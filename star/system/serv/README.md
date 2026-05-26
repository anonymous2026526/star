# STAR Rate-Limiting Service

This repository contains a rate-limiting service built with STAR anonymous
tokens and Intel SGX remote attestation. The service lets a client obtain a
rate-limit credential, attach an anonymous STAR token to traffic, and have a
proxy enforce token validity and replay resistance without learning the
client's registration identity.

## Components

| Repository path | Formal name | Purpose |
| --- | --- | --- |
| `enclave/` | Auth SGX Enclave | SGX enclave that implements the STAR authorization logic and produces attested keys. |
| `app/` | Filtering Proxy | Host application and proxy that run the enclaves, expose registration/public-key endpoints, and filter upstream traffic using STAR tokens. |
| `app/examples/` | Web App Server | Example upstream web server and deployment script used for the local end-to-end demo. |

Supporting code:

- `utils/manage-key/`: key-manager service and SGX enclave used to provision key material to the Auth SGX Enclave.
- `tools/verifier/`: quote inspection and DCAP verification helper.
- `integration/`: reproducible README environment and full integration runner.
- `certs/`: local development certificates used by the example deployment.

## Requirements

The commands below are intended to work from a fresh shell on this machine.
Non-secret environment variables are kept in `integration/env.sh` so the
same setup can be reproduced without manually sourcing `/etc/set-environment`.
That system file may still be used as a reference, but it is not required for
the documented flow.

The machine must provide:

- Intel SGX hardware and DCAP quote support.
- Running AESM service, normally `aesmd.service`.
- Access to SGX device nodes, for example `/dev/sgx_enclave` and
  `/dev/sgx_provision`.
- PCCS/DCAP collateral access. The checked-in README environment uses
  `https://pccs.phala.network` by default.

Following dependencies are required by this project:

- Rust with `rustup`
- Rust target `x86_64-fortanix-unknown-sgx`
- `sgxs-tools`
- `fortanix-sgx-tools`
- Intel SGX PSW / AESM
- Intel SGX DCAP quote provider and quote verification libraries
- Intel SGX SDK
- `protobuf`
- `zlib`
- `cmake`
- `curl`
- `git`
- `clang`
- `openssl`
- C/C++ runtime from the system compiler toolchain
- Docker-backed PCCS service or another reachable PCCS endpoint

Additional build tools may be needed when starting from an otherwise empty
shell. The verified commands use `nix-shell` to provide them:

```sh
nix-shell -p pkg-config openssl.dev openssl gcc gnumake clang
```

If the SGX Rust target is not installed yet, install it once:

```sh
rustup target add x86_64-fortanix-unknown-sgx
```

## Environment

The reproducible environment file is:

```sh
integration/env.sh
```

It sets the local SGX/DCAP library paths, AESM proxy address, demo ports, STAR
policy flags, target directories, and default certificate paths. It intentionally
does not contain secrets.

The README environment enables local-development policy:

- `STAR_ALLOW_DEBUG_ENCLAVES=1`
- `STAR_ALLOW_ADVISORY_ENCLAVES=1`
- `STAR_TRUST_SAME_SIGNER_ENCLAVES=1`

These settings are suitable for this local demo and integration verification.
For production, pin the expected enclave measurements/signers and disable broad
debug/advisory/same-signer trust.

## 1. Run The Example Environment

Use `app/examples/deploy.sh` to run the Web App Server, Filtering Proxy,
key-manager service, AESM TCP proxy, and local client proxy.

From a fresh shell:

```sh
cd /home/anonymous2/star/system/serv

nix-shell -p pkg-config openssl.dev openssl gcc gnumake clang --run 'bash -lc "
  . integration/env.sh
  export STAR_KEY_MANAGER_SEALED_KEYS=/tmp/star-key-manager-example-\$(date +%s).keys.sealed
  ./app/examples/deploy.sh --rebuild
"'
```

Use `--rebuild` for the first run or whenever SGX artifacts should be rebuilt.
After the `.sgxs` artifacts already exist, a faster repeated run can use:

```sh
cd /home/anonymous2/star/system/serv

nix-shell -p pkg-config openssl.dev openssl gcc gnumake clang --run 'bash -lc "
  . integration/env.sh
  export STAR_KEY_MANAGER_SEALED_KEYS=/tmp/star-key-manager-example-\$(date +%s).keys.sealed
  ./app/examples/deploy.sh --no-build
"'
```

The script performs a client smoke request. A successful run prints the HTML
response from the Web App Server, then keeps the demo services running until
interrupted with `Ctrl-C`.

Default local endpoints:

- Web App Server: `127.0.0.1:18081`
- Filtering Proxy API: `http://127.0.0.1:18080`
- Filtering Proxy TLS listener: `127.0.0.1:18443`
- Client proxy listener: `127.0.0.1:18180`

Logs are written under:

```sh
app/examples/logs/
```

## 2. Run All Functionality

Use `integration/run-all.sh` to exercise the full README verification path:

- source the reproducible README environment
- build missing SGX `.sgxs` artifacts
- start the AESM TCP proxy
- start the key-manager service
- run release tests
- run Criterion benchmarks
- start `star_app` for attestation
- inspect the generated quote
- verify the quote with the DCAP verifier

From a fresh shell:

```sh
cd /home/anonymous2/star/system/serv

nix-shell -p pkg-config openssl.dev openssl gcc gnumake clang --run './integration/run-all.sh'
```

A successful run ends with quote verification and local policy validation, for
example:

```text
report_data public key binding OK
local policy OK
```

Logs are written under:

```sh
integration/logs/
```

## Troubleshooting

If OpenSSL headers or `gcc` are missing, enter the documented `nix-shell`.
Typical errors include missing `opensslconf.h`, missing `pkg-config`, or
`failed to find tool "gcc"`.

If AESM cannot be reached, check:

```sh
systemctl status aesmd.service
```

If the key-manager fails while unsealing a previous key bundle, use a fresh
sealed-key path:

```sh
export STAR_KEY_MANAGER_SEALED_KEYS=/tmp/star-key-manager-$(date +%s).keys.sealed
```

If a port is already in use, stop the previous demo run or edit the port values
in `integration/env.sh`.

