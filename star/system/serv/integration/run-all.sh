#!/usr/bin/env bash
set -Eeuo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
ENV_FILE="$ROOT/integration/env.sh"
LOG_DIR="$ROOT/integration/logs"
RUN_ID=$(date +%Y%m%d-%H%M%S)
LOG_FILE="$LOG_DIR/integration-$RUN_ID.log"
APP_LOG="$LOG_DIR/star-app-attest-$RUN_ID.log"
KEY_MANAGER_LOG="$LOG_DIR/key-manager-$RUN_ID.log"
INSPECT_LOG="$LOG_DIR/inspect-$RUN_ID.log"
VERIFY_LOG="$LOG_DIR/verify-$RUN_ID.log"
SGX_TARGET=x86_64-fortanix-unknown-sgx

mkdir -p "$LOG_DIR"
cd "$ROOT"

# shellcheck source=integration/env.sh
. "$ENV_FILE"

export STAR_ENCLAVE_SGXS="$CARGO_TARGET_DIR/$SGX_TARGET/release/enclave-bench-max-count.sgxs"
export STAR_KEY_MANAGER_SEALED_KEYS="$LOG_DIR/key-manager-$RUN_ID.keys.sealed"

exec > >(tee -a "$LOG_FILE") 2>&1

AESM_PROXY_PID=
KEY_MANAGER_PID=
APP_PID=

cleanup() {
    stop_pid "$APP_PID"
    stop_pid "$KEY_MANAGER_PID"
    stop_pid "$AESM_PROXY_PID"
}

stop_pid() {
    local pid="${1:-}"
    [ -n "$pid" ] || return 0
    kill -0 "$pid" 2>/dev/null || return 0

    kill "$pid" 2>/dev/null || true
    for _ in $(seq 1 10); do
        if ! kill -0 "$pid" 2>/dev/null; then
            wait "$pid" 2>/dev/null || true
            return 0
        fi
        sleep 1
    done

    kill -9 "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
}

trap cleanup EXIT

section() {
    printf '\n== %s ==\n' "$1"
}

run() {
    section "$1"
    shift
    printf '+'
    printf ' %q' "$@"
    printf '\n'
    "$@"
}

run_with_stdin() {
    local title="$1"
    local stdin_file="$2"
    shift 2
    section "$title"
    printf '+'
    printf ' %q' "$@"
    printf ' < %q\n' "$stdin_file"
    "$@" < "$stdin_file"
}

default_parallelism() {
    getconf _NPROCESSORS_ONLN 2>/dev/null || printf '1\n'
}

build_sgxs() {
    local manifest_path="$1"
    local binary_name="$2"
    local output_path="$3"
    local heap_size="$4"
    local stack_size="$5"
    local threads="$6"
    local features="${7:-}"

    command -v ftxsgx-elf2sgxs >/dev/null 2>&1 || {
        echo "required command not found: ftxsgx-elf2sgxs" >&2
        exit 1
    }

    if [ -n "$features" ]; then
        cargo build --quiet --release --target "$SGX_TARGET" --manifest-path "$manifest_path" --features "$features"
    else
        cargo build --quiet --release --target "$SGX_TARGET" --manifest-path "$manifest_path"
    fi

    local elf_path="$CARGO_TARGET_DIR/$SGX_TARGET/release/$binary_name"
    if [ ! -f "$elf_path" ]; then
        echo "cargo built $binary_name, but $elf_path was not found" >&2
        exit 1
    fi

    mkdir -p "$(dirname "$output_path")"
    ftxsgx-elf2sgxs "$elf_path" \
        --library \
        --heap-size "$heap_size" \
        --stack-size "$stack_size" \
        --threads "$threads" \
        --output "$output_path"
}

ensure_sgxs() {
    section "Build SGX Artifacts"

    if [ "${STAR_README_REBUILD:-0}" = "1" ] || [ ! -f "$STAR_KEY_MANAGER_SGXS" ]; then
        echo "building $STAR_KEY_MANAGER_SGXS"
        build_sgxs \
            "$ROOT/utils/manage-key/enclave/Cargo.toml" \
            key-manager \
            "$STAR_KEY_MANAGER_SGXS" \
            "${STAR_KEY_MANAGER_HEAP_SIZE:-33554432}" \
            "${STAR_KEY_MANAGER_STACK_SIZE:-131072}" \
            "${STAR_KEY_MANAGER_THREADS:-1}"
    else
        echo "using $STAR_KEY_MANAGER_SGXS"
    fi

    if [ "${STAR_README_REBUILD:-0}" = "1" ] || [ ! -f "$STAR_ENCLAVE_SGXS" ]; then
        echo "building $STAR_ENCLAVE_SGXS"
        build_sgxs \
            "$ROOT/enclave/Cargo.toml" \
            enclave \
            "$STAR_ENCLAVE_SGXS" \
            "${STAR_ENCLAVE_HEAP_SIZE:-33554432}" \
            "${STAR_ENCLAVE_STACK_SIZE:-131072}" \
            "${STAR_ENCLAVE_THREADS:-$(default_parallelism)}" \
            bench-max-count
    else
        echo "using $STAR_ENCLAVE_SGXS"
    fi
}

wait_for_file_text() {
    local file="$1"
    local text="$2"
    local seconds="$3"
    local i
    for i in $(seq 1 "$seconds"); do
        if grep -q "$text" "$file" 2>/dev/null; then
            return 0
        fi
        if [ -n "${APP_PID:-}" ] && ! kill -0 "$APP_PID" 2>/dev/null; then
            return 1
        fi
        sleep 1
    done
    return 1
}

wait_for_http() {
    local url="$1"
    local seconds="$2"
    local i
    for i in $(seq 1 "$seconds"); do
        if curl -fsS "$url" >/dev/null 2>&1; then
            return 0
        fi
        if [ -n "${KEY_MANAGER_PID:-}" ] && ! kill -0 "$KEY_MANAGER_PID" 2>/dev/null; then
            return 1
        fi
        sleep 1
    done
    return 1
}

start_aesm_proxy() {
    section "Start AESM TCP proxy"
    if nc -z 127.0.0.1 5555 >/dev/null 2>&1; then
        echo "AESM proxy already listens on 127.0.0.1:5555"
        return 0
    fi

    perl -MIO::Socket::INET -MIO::Socket::UNIX -e '
        use strict;
        use warnings;
        my $listen = $ENV{AESM_PROXY} || "127.0.0.1:5555";
        my ($host, $port) = split /:/, $listen, 2;
        my $socket_path = $ENV{AESM_SOCKET} || "/run/aesmd/aesm.socket";
        my $server = IO::Socket::INET->new(
            LocalAddr => $host,
            LocalPort => $port,
            Listen => 16,
            ReuseAddr => 1,
            Proto => "tcp",
        ) or die "listen $listen: $!";
        while (my $client = $server->accept) {
            my $pid = fork();
            die "fork: $!" unless defined $pid;
            if ($pid == 0) {
                close $server;
                eval {
                    while (1) {
                        read_exact($client, 4, my $len_buf) or last;
                        my $len = unpack("L", $len_buf);
                        die "AESM frame too large: $len" if $len > 16 * 1024 * 1024;
                        read_exact($client, $len, my $body) or last;
                        my $unix = IO::Socket::UNIX->new(Type => SOCK_STREAM(), Peer => $socket_path)
                            or die "connect $socket_path: $!";
                        print {$unix} $len_buf, $body;
                        read_exact($unix, 4, my $res_len_buf) or die "AESM closed early";
                        my $res_len = unpack("L", $res_len_buf);
                        die "AESM response too large: $res_len" if $res_len > 16 * 1024 * 1024;
                        read_exact($unix, $res_len, my $res_body) or die "short AESM response";
                        print {$client} $res_len_buf, $res_body;
                        close $unix;
                    }
                };
                warn "$@\n" if $@;
                close $client;
                exit 0;
            }
            close $client;
            while (waitpid(-1, 1) > 0) {}
        }
        sub read_exact {
            my ($fh, $want, undef) = @_;
            my $buf = "";
            while (length($buf) < $want) {
                my $n = sysread($fh, my $chunk, $want - length($buf));
                return 0 unless defined($n) && $n > 0;
                $buf .= $chunk;
            }
            $_[2] = $buf;
            return 1;
        }
    ' &
    AESM_PROXY_PID=$!
    sleep 1
    if ! kill -0 "$AESM_PROXY_PID" 2>/dev/null; then
        wait "$AESM_PROXY_PID"
    fi
    echo "AESM proxy pid: $AESM_PROXY_PID"
}

start_key_manager() {
    section "Run Key-Manager Separately"
    cargo run --manifest-path utils/manage-key/service/Cargo.toml --release > "$KEY_MANAGER_LOG" 2>&1 &
    KEY_MANAGER_PID=$!
    echo "key-manager pid: $KEY_MANAGER_PID"
    if ! wait_for_http "$STAR_KEY_MANAGER_URL/star/key-manager/public_key" 900; then
        sed -n '1,160p' "$KEY_MANAGER_LOG"
        return 1
    fi
}

start_star_app() {
    section "Start With Attestation"
    cargo run --manifest-path app/Cargo.toml --release --bin star_app > "$APP_LOG" 2>&1 &
    APP_PID=$!
    echo "star_app pid: $APP_PID"
    if ! wait_for_file_text "$APP_LOG" "STAR_ISSUE_ENCLAVE_DCAP_EVIDENCE_FOR_KEY_MANAGER_BASE64=" 900; then
        sed -n '1,200p' "$APP_LOG"
        return 1
    fi
    stop_pid "$APP_PID"
    APP_PID=
    grep -E '^(STAR_ATTESTATION_PUBLIC_KEY_BASE64|STAR_ATTESTATION_QUOTE_BASE64|STAR_KEY_MANAGER_QUOTE_BASE64|STAR_FILTER_ENCLAVE_TRANSFER_QUOTE_BASE64|STAR_ISSUE_ENCLAVE_TRANSFER_QUOTE_BASE64|STAR_FILTER_ENCLAVE_DCAP_EVIDENCE_FOR_KEY_MANAGER_BASE64|STAR_ISSUE_ENCLAVE_DCAP_EVIDENCE_FOR_KEY_MANAGER_BASE64|STAR_KEY_MANAGER_DCAP_EVIDENCE_FOR_FILTER_ENCLAVE_BASE64|STAR_KEY_MANAGER_DCAP_EVIDENCE_FOR_ISSUE_ENCLAVE_BASE64|use plain|use tls)' "$APP_LOG" |
        sed -E 's/=.*/=<base64 omitted>/'
}

extract_field() {
    local name="$1"
    awk -v key="$name" '$1 == key { print $NF; exit }' "$INSPECT_LOG"
}

section "Environment"
    echo "env file: $ENV_FILE"
    echo "main log: $LOG_FILE"
    echo "app log: $APP_LOG"
    echo "key-manager log: $KEY_MANAGER_LOG"
    echo "inspect log: $INSPECT_LOG"
    echo "verify log: $VERIFY_LOG"
    env | sort | grep -E '^(AESM|PCCS|SGX|STAR|TLS|NO_PROXY|PKG_CONFIG|OPENSSL|LD_LIBRARY_PATH|PATH|CC|AR|RANLIB)='

ensure_sgxs

start_aesm_proxy

start_key_manager

run "README Test" cargo test --release

run "README Reproduce SGX Benchmarks" cargo bench

start_star_app

run_with_stdin "README Inspect A Quote" "$APP_LOG" \
    cargo run --manifest-path tools/verifier/Cargo.toml --release -- inspect \
    > "$INSPECT_LOG"
cat "$INSPECT_LOG"

PUBLIC_KEY=$(extract_field "public")
MRENCLAVE=$(extract_field "mrenclave:")
MRSIGNER=$(extract_field "mrsigner")

if [ -z "$PUBLIC_KEY" ] || [ -z "$MRENCLAVE" ] || [ -z "$MRSIGNER" ]; then
    echo "failed to extract verifier measurements"
    exit 1
fi

run_with_stdin "README Verify A Quote" "$APP_LOG" \
    cargo run --manifest-path tools/verifier/Cargo.toml --release -- verify \
        --mrsigner "$MRSIGNER" \
        --mrenclave "$MRENCLAVE" \
        --public-key "$PUBLIC_KEY" \
        --allow-debug \
        --allow-advisory \
        --pccs-url "$PCCS_URL" \
    > "$VERIFY_LOG"
cat "$VERIFY_LOG"

section "Done"
echo "main log: $LOG_FILE"
echo "app log: $APP_LOG"
echo "key-manager log: $KEY_MANAGER_LOG"
echo "inspect log: $INSPECT_LOG"
echo "verify log: $VERIFY_LOG"
