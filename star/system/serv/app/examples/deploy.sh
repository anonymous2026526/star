#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT=$(cd "$SCRIPT_DIR/../.." && pwd)
CLIENT_MANIFEST=${STAR_CLIENT_MANIFEST:-"$ROOT/../client/Cargo.toml"}
SGX_TARGET=x86_64-fortanix-unknown-sgx

usage() {
    cat <<EOF
Usage: $(basename "$0") [--no-build] [--rebuild] [--no-aesm-proxy] [--no-client]

Deploy the STAR app example locally:
  - builds missing enclave .sgxs files
  - starts the key-manager service
  - starts app/examples/main.rs
  - starts the STAR client proxy
  - verifies the path with one request through the client proxy

Useful environment overrides:
  STAR_ENV_FILE=/path/to/env.sh       Source an SGX/Nix environment first.
  AESM_PROXY=127.0.0.1:5555          TCP AESM proxy address.
  AESM_SOCKET=/run/aesmd/aesm.socket Local AESM Unix socket.
  PCCS_URL=https://pccs.phala.network
  STAR_KEY_MANAGER_ADDR=127.0.0.1:8090
  STAR_KEY_MANAGER_URL=http://127.0.0.1:8090
  STAR_API_ADDR=127.0.0.1:8080
  STAR_PROXY_ADDR=127.0.0.1:18081
  STAR_EXAMPLE_CONTENT_ADDR=127.0.0.1:3000
  STAR_CLIENT_PROXY_LISTEN=127.0.0.1:18082
  STAR_CLIENT_SMOKE_TIMEOUT=900
  STAR_EXAMPLE_LOG_DIR=$ROOT/app/examples/logs

The app API listens on STAR_API_ADDR. The example content server is placed
behind the STAR TLS proxy at STAR_PROXY_ADDR.
EOF
}

NO_BUILD=0
REBUILD=0
NO_AESM_PROXY=0
NO_CLIENT=0

while [ "$#" -gt 0 ]; do
    case "$1" in
        --help|-h)
            usage
            exit 0
            ;;
        --no-build)
            NO_BUILD=1
            ;;
        --rebuild)
            REBUILD=1
            ;;
        --no-aesm-proxy)
            NO_AESM_PROXY=1
            ;;
        --no-client)
            NO_CLIENT=1
            ;;
        *)
            echo "unknown argument: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
    shift
done

die() {
    echo "error: $*" >&2
    exit 1
}

section() {
    printf '\n== %s ==\n' "$1"
}

require_cmd() {
    command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

source_environment() {
    if [ -n "${STAR_ENV_FILE:-}" ]; then
        [ -f "$STAR_ENV_FILE" ] || die "STAR_ENV_FILE does not exist: $STAR_ENV_FILE"
        # shellcheck source=/dev/null
        set +u
        . "$STAR_ENV_FILE"
        set -u
    elif [ -f "$ROOT/integration/readme-env.sh" ]; then
        # shellcheck source=../../integration/readme-env.sh
        set +u
        . "$ROOT/integration/readme-env.sh"
        set -u
    elif [ -f /etc/set-environment ]; then
        # shellcheck source=/dev/null
        set +u
        . /etc/set-environment
        set -u
    fi
}

absolute_target_dir() {
    local dir=${CARGO_TARGET_DIR:-"$ROOT/target"}
    case "$dir" in
        /*) printf '%s\n' "$dir" ;;
        *) printf '%s\n' "$ROOT/$dir" ;;
    esac
}

default_parallelism() {
    getconf _NPROCESSORS_ONLN 2>/dev/null || printf '1\n'
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

KEY_MANAGER_PID=
STAR_EXAMPLE_PID=
CLIENT_PROXY_PID=
AESM_PROXY_PID=

cleanup() {
    stop_pid "$CLIENT_PROXY_PID"
    stop_pid "$STAR_EXAMPLE_PID"
    stop_pid "$KEY_MANAGER_PID"
    stop_pid "$AESM_PROXY_PID"
}

trap cleanup EXIT

wait_for_http() {
    local url="$1"
    local seconds="$2"
    local watched_pid="${3:-}"
    local i=0

    while [ "$i" -lt "$seconds" ]; do
        if curl -fsS "$url" >/dev/null 2>&1; then
            return 0
        fi
        if [ -n "$watched_pid" ] && ! kill -0 "$watched_pid" 2>/dev/null; then
            return 1
        fi
        sleep 1
        i=$((i + 1))
    done

    return 1
}

wait_for_demo_response() {
    local url="$1"
    local seconds="$2"
    local i=0
    local body

    while [ "$i" -lt "$seconds" ]; do
        if body=$(curl -fsS "$url" 2>/dev/null); then
            printf '%s\n' "$body"
            return 0
        fi
        if [ -n "$CLIENT_PROXY_PID" ] && ! kill -0 "$CLIENT_PROXY_PID" 2>/dev/null; then
            return 1
        fi
        sleep 1
        i=$((i + 1))
    done

    return 1
}

tcp_port_open() {
    local addr="$1"
    local host="${addr%:*}"
    local port="${addr##*:}"

    if command -v nc >/dev/null 2>&1; then
        nc -z "$host" "$port" >/dev/null 2>&1
    else
        return 1
    fi
}

start_aesm_proxy() {
    [ "$NO_AESM_PROXY" -eq 0 ] || return 0

    section "AESM proxy"
    if tcp_port_open "$AESM_PROXY"; then
        echo "using existing AESM proxy at $AESM_PROXY"
        return 0
    fi

    require_cmd perl

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
    ' > "$AESM_LOG" 2>&1 &

    AESM_PROXY_PID=$!
    sleep 1

    if ! kill -0 "$AESM_PROXY_PID" 2>/dev/null; then
        sed -n '1,120p' "$AESM_LOG" >&2 || true
        die "AESM proxy failed to start"
    fi

    echo "started AESM proxy at $AESM_PROXY (pid $AESM_PROXY_PID)"
}

generate_tls_cert() {
    if [ -n "${TLS_CERT_PATH:-}" ] || [ -n "${TLS_KEY_PATH:-}" ]; then
        [ -n "${TLS_CERT_PATH:-}" ] && [ -n "${TLS_KEY_PATH:-}" ] \
            || die "TLS_CERT_PATH and TLS_KEY_PATH must be set together"
        [ -f "$TLS_CERT_PATH" ] || die "TLS_CERT_PATH does not exist: $TLS_CERT_PATH"
        [ -f "$TLS_KEY_PATH" ] || die "TLS_KEY_PATH does not exist: $TLS_KEY_PATH"
        return 0
    fi

    require_cmd openssl

    local cert_dir="$SCRIPT_DIR/certs"
    mkdir -p "$cert_dir"
    export TLS_CERT_PATH="$cert_dir/server.crt"
    export TLS_KEY_PATH="$cert_dir/server.key"

    if [ -f "$TLS_CERT_PATH" ] && [ -f "$TLS_KEY_PATH" ]; then
        return 0
    fi

    section "TLS certificate"
    openssl req -x509 -newkey ed25519 -nodes \
        -keyout "$TLS_KEY_PATH" \
        -out "$TLS_CERT_PATH" \
        -days 365 \
        -subj "/CN=localhost" >/dev/null 2>&1
    echo "generated $TLS_CERT_PATH and $TLS_KEY_PATH"
}

build_sgxs() {
    local manifest_path="$1"
    local binary_name="$2"
    local output_path="$3"
    local heap_size="$4"
    local stack_size="$5"
    local threads="$6"

    require_cmd cargo
    require_cmd ftxsgx-elf2sgxs

    cargo build --quiet --release --target "$SGX_TARGET" --manifest-path "$manifest_path"

    local elf_path="$TARGET_DIR/$SGX_TARGET/release/$binary_name"
    [ -f "$elf_path" ] || die "cargo built $binary_name, but $elf_path was not found"

    mkdir -p "$(dirname "$output_path")"
    ftxsgx-elf2sgxs "$elf_path" \
        --library \
        --heap-size "$heap_size" \
        --stack-size "$stack_size" \
        --threads "$threads" \
        --output "$output_path"
}

ensure_sgxs() {
    [ "$NO_BUILD" -eq 0 ] || return 0

    section "SGX artifacts"

    if [ "$REBUILD" -eq 1 ] || [ ! -f "$STAR_KEY_MANAGER_SGXS" ]; then
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

    if [ "$REBUILD" -eq 1 ] || [ ! -f "$STAR_ENCLAVE_SGXS" ]; then
        echo "building $STAR_ENCLAVE_SGXS"
        build_sgxs \
            "$ROOT/enclave/Cargo.toml" \
            enclave \
            "$STAR_ENCLAVE_SGXS" \
            "${STAR_ENCLAVE_HEAP_SIZE:-33554432}" \
            "${STAR_ENCLAVE_STACK_SIZE:-131072}" \
            "${STAR_ENCLAVE_THREADS:-$(default_parallelism)}"
    else
        echo "using $STAR_ENCLAVE_SGXS"
    fi
}

start_key_manager() {
    section "Key-manager"
    cargo run --manifest-path "$ROOT/utils/manage-key/service/Cargo.toml" --release \
        > "$KEY_MANAGER_LOG" 2>&1 &
    KEY_MANAGER_PID=$!
    echo "started key-manager (pid $KEY_MANAGER_PID)"

    if ! wait_for_http "$STAR_KEY_MANAGER_URL/star/key-manager/public_key" 900 "$KEY_MANAGER_PID"; then
        sed -n '1,160p' "$KEY_MANAGER_LOG" >&2 || true
        die "key-manager did not become ready"
    fi
}

start_star_example() {
    section "STAR example"
    cargo run --manifest-path "$ROOT/app/Cargo.toml" --release --example main \
        > "$STAR_EXAMPLE_LOG" 2>&1 &
    STAR_EXAMPLE_PID=$!
    echo "started STAR example (pid $STAR_EXAMPLE_PID)"

    if ! wait_for_http "$STAR_API_BASE/star/public_key" 900 "$STAR_EXAMPLE_PID"; then
        sed -n '1,220p' "$STAR_EXAMPLE_LOG" >&2 || true
        die "STAR example did not become ready"
    fi
}

start_client_proxy() {
    [ "$NO_CLIENT" -eq 0 ] || return 0
    [ -f "$CLIENT_MANIFEST" ] || die "client manifest not found: $CLIENT_MANIFEST"

    section "Client proxy"
    cargo run --manifest-path "$CLIENT_MANIFEST" --release > "$CLIENT_PROXY_LOG" 2>&1 &
    CLIENT_PROXY_PID=$!
    echo "started client proxy (pid $CLIENT_PROXY_PID)"

    local body
    if ! body=$(wait_for_demo_response "$STAR_DEMO_URL" "${STAR_CLIENT_SMOKE_TIMEOUT:-900}"); then
        sed -n '1,220p' "$CLIENT_PROXY_LOG" >&2 || true
        die "client proxy did not return a demo response"
    fi

    echo "smoke response: $body"
}

source_environment

TARGET_DIR=$(absolute_target_dir)
export CARGO_TARGET_DIR="$TARGET_DIR"
RUN_ID=$(date +%Y%m%d-%H%M%S)
LOG_DIR=${STAR_EXAMPLE_LOG_DIR:-"$ROOT/app/examples/logs"}
mkdir -p "$LOG_DIR"

KEY_MANAGER_LOG="$LOG_DIR/key-manager-$RUN_ID.log"
STAR_EXAMPLE_LOG="$LOG_DIR/star-example-$RUN_ID.log"
CLIENT_PROXY_LOG="$LOG_DIR/client-proxy-$RUN_ID.log"
AESM_LOG="$LOG_DIR/aesm-proxy-$RUN_ID.log"

export AESM_SOCKET=${AESM_SOCKET:-/run/aesmd/aesm.socket}
export AESM_PROXY=${AESM_PROXY:-127.0.0.1:5555}
export PCCS_URL=${PCCS_URL:-https://pccs.phala.network}
export NO_PROXY=${NO_PROXY:-127.0.0.1,localhost}

export STAR_ALLOW_DEBUG_ENCLAVES=${STAR_ALLOW_DEBUG_ENCLAVES:-1}
export STAR_ALLOW_ADVISORY_ENCLAVES=${STAR_ALLOW_ADVISORY_ENCLAVES:-1}
export STAR_TRUST_SAME_SIGNER_ENCLAVES=${STAR_TRUST_SAME_SIGNER_ENCLAVES:-1}

export STAR_KEY_MANAGER_ADDR=${STAR_KEY_MANAGER_ADDR:-127.0.0.1:8090}
export STAR_KEY_MANAGER_URL=${STAR_KEY_MANAGER_URL:-http://127.0.0.1:8090}
export STAR_KEY_MANAGER_SGXS=${STAR_KEY_MANAGER_SGXS:-"$TARGET_DIR/$SGX_TARGET/release/key-manager.sgxs"}
export STAR_KEY_MANAGER_SEALED_KEYS=${STAR_KEY_MANAGER_SEALED_KEYS:-"$ROOT/app/examples/key-manager.keys.sealed"}

export STAR_ENCLAVE_SGXS=${STAR_ENCLAVE_SGXS:-"$TARGET_DIR/$SGX_TARGET/release/enclave.sgxs"}

export STAR_API_ADDR=${STAR_API_ADDR:-127.0.0.1:8080}
export STAR_PROXY_ADDR=${STAR_PROXY_ADDR:-127.0.0.1:18081}
export STAR_EXAMPLE_CONTENT_ADDR=${STAR_EXAMPLE_CONTENT_ADDR:-${STAR_EXAMPLE_HTML_ADDR:-${STAR_EXAMPLE_UPSTREAM_ADDR:-127.0.0.1:3000}}}
export STAR_API_BASE=${STAR_API_BASE:-http://$STAR_API_ADDR}
export STAR_UPSTREAM_HOST=${STAR_UPSTREAM_HOST:-${STAR_PROXY_ADDR%:*}}
export STAR_UPSTREAM_PORT=${STAR_UPSTREAM_PORT:-${STAR_PROXY_ADDR##*:}}
export STAR_UPSTREAM_SNI=${STAR_UPSTREAM_SNI:-localhost}
export STAR_CLIENT_PROXY_LISTEN=${STAR_CLIENT_PROXY_LISTEN:-127.0.0.1:18082}
export STAR_MAX_COUNT=${STAR_MAX_COUNT:-100}

STAR_DEMO_URL=${STAR_DEMO_URL:-"http://$STAR_CLIENT_PROXY_LISTEN/"}

require_cmd cargo
require_cmd curl

section "Configuration"
echo "logs: $LOG_DIR"
echo "key-manager: $STAR_KEY_MANAGER_URL"
echo "app API: $STAR_API_BASE"
echo "example STAR TLS proxy: $STAR_UPSTREAM_HOST:$STAR_UPSTREAM_PORT"
echo "example content server: $STAR_EXAMPLE_CONTENT_ADDR"
if [ "$NO_CLIENT" -eq 0 ]; then
    echo "client proxy: http://$STAR_CLIENT_PROXY_LISTEN"
fi

generate_tls_cert
ensure_sgxs
start_aesm_proxy
start_key_manager
start_star_example
start_client_proxy

section "Running"
echo "demo URL: $STAR_DEMO_URL"
echo "key-manager log: $KEY_MANAGER_LOG"
echo "STAR example log: $STAR_EXAMPLE_LOG"
if [ "$NO_CLIENT" -eq 0 ]; then
    echo "client proxy log: $CLIENT_PROXY_LOG"
fi
echo "press Ctrl-C to stop"

if [ "$NO_CLIENT" -eq 0 ]; then
    wait -n "$KEY_MANAGER_PID" "$STAR_EXAMPLE_PID" "$CLIENT_PROXY_PID"
else
    wait -n "$KEY_MANAGER_PID" "$STAR_EXAMPLE_PID"
fi
