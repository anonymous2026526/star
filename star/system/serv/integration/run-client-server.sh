#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
SERV_ROOT=$(cd "$SCRIPT_DIR/.." && pwd)
SYSTEM_ROOT=$(cd "$SERV_ROOT/.." && pwd)
ENV_FILE="${STAR_INTEGRATION_ENV:-$SERV_ROOT/integration/env.sh}"
LOG_DIR="$SERV_ROOT/integration/logs"
RUN_ID=$(date +%Y%m%d-%H%M%S)
LOG_FILE="$LOG_DIR/client-server-integration-$RUN_ID.log"
KEY_MANAGER_LOG="$LOG_DIR/client-server-key-manager-$RUN_ID.log"
SERVER_LOG="$LOG_DIR/client-server-star-app-$RUN_ID.log"
CLIENT_LOG="$LOG_DIR/client-server-client-proxy-$RUN_ID.log"
UPSTREAM_LOG="$LOG_DIR/client-server-upstream-$RUN_ID.log"

mkdir -p "$LOG_DIR"

# shellcheck source=integration/env.sh
. "$ENV_FILE"

TMP_DIR=$(mktemp -d "${TMPDIR:-/tmp}/star-client-server.XXXXXX")

exec > >(tee -a "$LOG_FILE") 2>&1

AESM_PROXY_PID=
KEY_MANAGER_PID=
SERVER_PID=
CLIENT_PID=
UPSTREAM_PID=

cleanup() {
    local status=$?

    if [ "$status" -ne 0 ]; then
        section "Failure Logs"
        tail_log "upstream" "$UPSTREAM_LOG"
        tail_log "key-manager" "$KEY_MANAGER_LOG"
        tail_log "server" "$SERVER_LOG"
        tail_log "client" "$CLIENT_LOG"
    fi

    stop_pid "$CLIENT_PID"
    stop_pid "$SERVER_PID"
    stop_pid "$KEY_MANAGER_PID"
    stop_pid "$UPSTREAM_PID"
    stop_pid "$AESM_PROXY_PID"
    rm -rf "$TMP_DIR"
    return "$status"
}

stop_pid() {
    local pid="${1:-}"
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
        kill "$pid" 2>/dev/null || true
        wait "$pid" 2>/dev/null || true
    fi
}

trap cleanup EXIT

section() {
    printf '\n== %s ==\n' "$1"
}

fail() {
    echo "error: $*" >&2
    exit 1
}

tail_log() {
    local name="$1"
    local file="$2"
    if [ -f "$file" ]; then
        printf '\n-- %s: %s --\n' "$name" "$file"
        tail -n 120 "$file"
    fi
}

need_cmd() {
    command -v "$1" >/dev/null 2>&1 || fail "missing required command: $1"
}

split_host_port() {
    local addr="$1"
    local host_var="$2"
    local port_var="$3"
    local host="${addr%:*}"
    local port="${addr##*:}"

    if [ -z "$host" ] || [ -z "$port" ] || [ "$host" = "$addr" ]; then
        fail "expected host:port address, got: $addr"
    fi

    printf -v "$host_var" '%s' "$host"
    printf -v "$port_var" '%s' "$port"
}

tcp_ready() {
    local host="$1"
    local port="$2"

    if command -v nc >/dev/null 2>&1; then
        nc -z "$host" "$port" >/dev/null 2>&1
        return
    fi

    perl -MIO::Socket::INET -e '
        my ($host, $port) = @ARGV;
        my $sock = IO::Socket::INET->new(
            PeerAddr => $host,
            PeerPort => $port,
            Proto => "tcp",
            Timeout => 1,
        );
        exit($sock ? 0 : 1);
    ' "$host" "$port"
}

wait_for_tcp() {
    local name="$1"
    local addr="$2"
    local seconds="$3"
    local pid="${4:-}"
    local host
    local port
    local i

    split_host_port "$addr" host port
    for i in $(seq 1 "$seconds"); do
        if tcp_ready "$host" "$port"; then
            echo "$name is listening on $addr"
            return 0
        fi
        if [ -n "$pid" ] && ! kill -0 "$pid" 2>/dev/null; then
            echo "$name exited before listening on $addr"
            return 1
        fi
        sleep 1
    done

    echo "timed out waiting for $name on $addr"
    return 1
}

wait_for_http() {
    local name="$1"
    local url="$2"
    local seconds="$3"
    local pid="${4:-}"
    local i

    for i in $(seq 1 "$seconds"); do
        if curl -fsS "$url" >/dev/null 2>&1; then
            echo "$name is ready at $url"
            return 0
        fi
        if [ -n "$pid" ] && ! kill -0 "$pid" 2>/dev/null; then
            echo "$name exited before becoming ready at $url"
            return 1
        fi
        sleep 1
    done

    echo "timed out waiting for $name at $url"
    return 1
}

wait_for_http_body() {
    local name="$1"
    local url="$2"
    local expected="$3"
    local seconds="$4"
    local pid="${5:-}"
    local body
    local i

    for i in $(seq 1 "$seconds"); do
        body=$(curl -fsS "$url" 2>/dev/null || true)
        if [[ "$body" == *"$expected"* ]]; then
            echo "$name returned expected body"
            return 0
        fi
        if [ -n "$pid" ] && ! kill -0 "$pid" 2>/dev/null; then
            echo "$name exited before returning expected body"
            return 1
        fi
        sleep 1
    done

    echo "timed out waiting for expected body from $name"
    return 1
}

start_aesm_proxy() {
    section "Start AESM TCP proxy"
    if tcp_ready 127.0.0.1 5555; then
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
    wait_for_tcp "AESM proxy" "${AESM_PROXY:-127.0.0.1:5555}" 10 "$AESM_PROXY_PID"
}

start_upstream() {
    section "Start upstream HTTP server"
    local host
    local port

    split_host_port "$STAR_UPSTREAM_ADDR" host port
    printf '%s\n' "$EXPECTED_BODY" > "$TMP_DIR/index.html"
    python3 -m http.server "$port" --bind "$host" --directory "$TMP_DIR" > "$UPSTREAM_LOG" 2>&1 &
    UPSTREAM_PID=$!
    echo "upstream pid: $UPSTREAM_PID"
    wait_for_http "upstream" "http://$STAR_UPSTREAM_ADDR/" 30 "$UPSTREAM_PID"
}

start_key_manager() {
    section "Start key-manager"
    cargo run --manifest-path "$SERV_ROOT/utils/manage-key/service/Cargo.toml" --release > "$KEY_MANAGER_LOG" 2>&1 &
    KEY_MANAGER_PID=$!
    echo "key-manager pid: $KEY_MANAGER_PID"
    wait_for_http "key-manager" "$STAR_KEY_MANAGER_URL/star/key-manager/public_key" 900 "$KEY_MANAGER_PID"
}

start_server() {
    section "Start STAR server"
    cargo run --manifest-path "$SERV_ROOT/app/Cargo.toml" --release --bin star_app > "$SERVER_LOG" 2>&1 &
    SERVER_PID=$!
    echo "server pid: $SERVER_PID"
    wait_for_http "STAR API" "http://$STAR_API_ADDR/star/public_key" 900 "$SERVER_PID"
    wait_for_tcp "STAR TLS proxy" "$STAR_PROXY_ADDR" 60 "$SERVER_PID"
}

start_client() {
    section "Start STAR client proxy"
    cargo run --manifest-path "$SYSTEM_ROOT/client/Cargo.toml" --release > "$CLIENT_LOG" 2>&1 &
    CLIENT_PID=$!
    echo "client pid: $CLIENT_PID"
    wait_for_tcp "STAR client proxy" "$STAR_CLIENT_PROXY_LISTEN" 120 "$CLIENT_PID"
}

test_client_preflight() {
    section "Client CORS preflight"
    local status
    status=$(curl -sS -o /dev/null -w '%{http_code}' \
        -X OPTIONS \
        -H 'Access-Control-Request-Method: GET' \
        "http://$STAR_CLIENT_PROXY_LISTEN/")
    [ "$status" = "204" ] || fail "expected preflight status 204, got $status"
    echo "preflight status: $status"
}

test_end_to_end_request() {
    section "Client to server end-to-end request"
    wait_for_http_body \
        "client proxy end-to-end request" \
        "http://$STAR_CLIENT_PROXY_LISTEN/" \
        "$EXPECTED_BODY" \
        60 \
        "$CLIENT_PID"
}

test_server_rejects_plain_client() {
    section "Server rejects clients without STAR TLS extension"
    if curl -kfsS --max-time 10 "https://$STAR_PROXY_ADDR/" >/dev/null 2>&1; then
        fail "server proxy accepted a TLS client without STAR extension"
    fi
    echo "direct HTTPS request without STAR extension was rejected"
}

section "Prerequisites"
need_cmd cargo
need_cmd curl
need_cmd perl
need_cmd python3
echo "env file: $ENV_FILE"
echo "main log: $LOG_FILE"
echo "key-manager log: $KEY_MANAGER_LOG"
echo "server log: $SERVER_LOG"
echo "client log: $CLIENT_LOG"
echo "upstream log: $UPSTREAM_LOG"

EXPECTED_BODY="STAR_CLIENT_SERVER_INTEGRATION_OK $RUN_ID"

export STAR_KEY_MANAGER_SEALED_KEYS="$TMP_DIR/key-manager.keys.sealed"
export TLS_CERT_PATH="${TLS_CERT_PATH:-$SERV_ROOT/app/examples/certs/server.crt}"
export TLS_KEY_PATH="${TLS_KEY_PATH:-$SERV_ROOT/app/examples/certs/server.key}"
export STAR_API_THREADS="${STAR_API_THREADS:-1}"
export STAR_PROXY_THREADS="${STAR_PROXY_THREADS:-1}"
export STAR_CLIENT_PROXY_LISTEN="${STAR_CLIENT_PROXY_LISTEN:-127.0.0.1:18082}"
export STAR_API_BASE="${STAR_CLIENT_API_BASE:-http://$STAR_API_ADDR}"

split_host_port "${STAR_CLIENT_UPSTREAM_ADDR:-$STAR_PROXY_ADDR}" STAR_CLIENT_UPSTREAM_HOST STAR_CLIENT_UPSTREAM_PORT
export STAR_UPSTREAM_HOST="$STAR_CLIENT_UPSTREAM_HOST"
export STAR_UPSTREAM_PORT="$STAR_CLIENT_UPSTREAM_PORT"
export STAR_UPSTREAM_SNI="${STAR_UPSTREAM_SNI:-localhost}"
export STAR_MAX_COUNT="${STAR_MAX_COUNT:-100}"

section "Environment"
echo "STAR_KEY_MANAGER_URL=$STAR_KEY_MANAGER_URL"
echo "STAR_API_ADDR=$STAR_API_ADDR"
echo "STAR_PROXY_ADDR=$STAR_PROXY_ADDR"
echo "STAR_UPSTREAM_ADDR=$STAR_UPSTREAM_ADDR"
echo "STAR_CLIENT_PROXY_LISTEN=$STAR_CLIENT_PROXY_LISTEN"
echo "STAR_API_BASE=$STAR_API_BASE"
echo "STAR_UPSTREAM_HOST=$STAR_UPSTREAM_HOST"
echo "STAR_UPSTREAM_PORT=$STAR_UPSTREAM_PORT"
echo "STAR_UPSTREAM_SNI=$STAR_UPSTREAM_SNI"
echo "STAR_KEY_MANAGER_SEALED_KEYS=$STAR_KEY_MANAGER_SEALED_KEYS"
echo "TLS_CERT_PATH=$TLS_CERT_PATH"
echo "TLS_KEY_PATH=$TLS_KEY_PATH"

start_aesm_proxy
start_upstream
start_key_manager
start_server
start_client

test_client_preflight
test_end_to_end_request
test_server_rejects_plain_client

section "Done"
echo "client/server integration test passed"
echo "main log: $LOG_FILE"
echo "key-manager log: $KEY_MANAGER_LOG"
echo "server log: $SERVER_LOG"
echo "client log: $CLIENT_LOG"
echo "upstream log: $UPSTREAM_LOG"
