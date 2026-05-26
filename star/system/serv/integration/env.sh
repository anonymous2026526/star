#!/usr/bin/env bash

# Reproducible environment for README commands.
#
# This file intentionally owns the non-secret environment used by the example
# and integration scripts. It may be sourced from a nix-shell that provides
# compiler/OpenSSL development inputs, but it does not depend on
# /etc/set-environment.

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
SGX_TARGET=x86_64-fortanix-unknown-sgx

prepend_path() {
    [ -d "${1:-}" ] || return 0
    case ":${PATH:-}:" in
        *":$1:"*) ;;
        *) export PATH="$1${PATH:+:$PATH}" ;;
    esac
}

prepend_ld_library_path() {
    [ -d "${1:-}" ] || return 0
    case ":${LD_LIBRARY_PATH:-}:" in
        *":$1:"*) ;;
        *) export LD_LIBRARY_PATH="$1${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" ;;
    esac
}

prepend_path /run/current-system/sw/bin
prepend_path /run/wrappers/bin

export QCNL_CONF_PATH=/etc/sgx_default_qcnl.conf
export SGX_DCAP_QPL=/run/current-system/sw/lib/libdcap_quoteprov.so.1
export SGX_DCAP_QVL=/run/current-system/sw/lib/libsgx_dcap_quoteverify.so
if [ -e /tmp/psw/ae/data/prebuilt/libsgx_qve.signed.so ]; then
    export SGX_DCAP_QVE=/tmp/psw/ae/data/prebuilt/libsgx_qve.signed.so
fi

export AESM_SOCKET=/run/aesmd/aesm.socket
export AESM_PROXY=127.0.0.1:5555
export PCCS_URL=https://pccs.phala.network
export NO_PROXY=127.0.0.1,localhost

export STAR_ALLOW_DEBUG_ENCLAVES=1
export STAR_ALLOW_ADVISORY_ENCLAVES=1
export STAR_TRUST_SAME_SIGNER_ENCLAVES=1

CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-"$ROOT/target"}
case "$CARGO_TARGET_DIR" in
    /*) ;;
    *) CARGO_TARGET_DIR="$ROOT/$CARGO_TARGET_DIR" ;;
esac
export CARGO_TARGET_DIR

export STAR_ENCLAVE_SGXS=${STAR_ENCLAVE_SGXS:-"$CARGO_TARGET_DIR/$SGX_TARGET/release/enclave.sgxs"}
export STAR_KEY_MANAGER_SGXS=${STAR_KEY_MANAGER_SGXS:-"$CARGO_TARGET_DIR/$SGX_TARGET/release/key-manager.sgxs"}
export STAR_KEY_MANAGER_SEALED_KEYS=${STAR_KEY_MANAGER_SEALED_KEYS:-"$ROOT/integration/key-manager.keys.sealed"}

export STAR_KEY_MANAGER_ADDR=127.0.0.1:8090
export STAR_KEY_MANAGER_URL=http://127.0.0.1:8090

export STAR_API_ADDR=127.0.0.1:18080
export STAR_PROXY_ADDR=127.0.0.1:18081
export STAR_UPSTREAM_ADDR=127.0.0.1:13000
export STAR_API_BASE=http://$STAR_API_ADDR
export STAR_UPSTREAM_HOST=${STAR_PROXY_ADDR%:*}
export STAR_UPSTREAM_PORT=${STAR_PROXY_ADDR##*:}
export STAR_UPSTREAM_SNI=localhost
export STAR_CLIENT_PROXY_LISTEN=127.0.0.1:18082
export STAR_CLIENT_SMOKE_TIMEOUT=900
export STAR_MAX_COUNT=100

export TLS_CERT_PATH=
export TLS_KEY_PATH=

if command -v pkg-config >/dev/null 2>&1 && pkg-config --exists openssl; then
    export PKG_CONFIG_FOR_TARGET=${PKG_CONFIG_FOR_TARGET:-pkg-config}
    export PKG_CONFIG_PATH_FOR_TARGET=${PKG_CONFIG_PATH_FOR_TARGET:-${PKG_CONFIG_PATH:-}}
    export OPENSSL_INCLUDE_DIR=${OPENSSL_INCLUDE_DIR:-$(pkg-config --variable=includedir openssl)}
    export OPENSSL_LIB_DIR=${OPENSSL_LIB_DIR:-$(pkg-config --variable=libdir openssl)}
fi
export OPENSSL_NO_VENDOR=${OPENSSL_NO_VENDOR:-1}

if command -v gcc >/dev/null 2>&1; then
    export CC=${CC:-$(command -v gcc)}
fi
if command -v ar >/dev/null 2>&1; then
    export AR=${AR:-$(command -v ar)}
fi
if command -v ranlib >/dev/null 2>&1; then
    export RANLIB=${RANLIB:-$(command -v ranlib)}
fi

if [ -n "${SGX_DCAP_QPL:-}" ]; then
    prepend_ld_library_path "$(dirname "$SGX_DCAP_QPL")"
fi
if [ -n "${SGX_DCAP_QVL:-}" ]; then
    prepend_ld_library_path "$(dirname "$SGX_DCAP_QVL")"
fi
if [ -n "${OPENSSL_LIB_DIR:-}" ]; then
    prepend_ld_library_path "$OPENSSL_LIB_DIR"
elif command -v openssl >/dev/null 2>&1 && command -v ldd >/dev/null 2>&1; then
    _env_openssl_libdir=$(
        ldd "$(command -v openssl)" 2>/dev/null |
            awk '/libssl\.so|libcrypto\.so/ && $3 ~ /^\// { sub("/[^/]+$", "", $3); print $3; exit }'
    )
    prepend_ld_library_path "$_env_openssl_libdir"
    unset _env_openssl_libdir
fi

unset -f prepend_path
unset -f prepend_ld_library_path
