# STAR Client Proxy

`system/client` is the local client-side proxy for STAR-protected traffic.
It accepts plain HTTP from local applications, obtains STAR tokens from the STAR
API, attaches the token as a custom TLS extension, and forwards the request to
the STAR server-side TLS proxy.

## Requirements

For the client proxy itself:

- Rust / Cargo
- OpenSSL development headers and libraries
- A running STAR API endpoint
- A running STAR server-side TLS proxy to use as the upstream

On the development machine used by the service README, the following shell is
sufficient for the Rust/OpenSSL build inputs:

```sh
nix-shell -p pkg-config openssl.dev openssl gcc gnumake clang
````

## Configuration

The binary reads configuration from environment variables at startup.

| Variable                   | Default                      | Description                                                             |
| -------------------------- | ---------------------------- | ----------------------------------------------------------------------- |
| `STAR_CLIENT_PROXY_LISTEN` | `127.0.0.1:18080`            | Local address where the client proxy listens.                           |
| `STAR_API_BASE`            | `http://127.0.0.1:8080`      | STAR API base URL. Must expose `/star/public_key` and `/star/register`. |
| `STAR_UPSTREAM_HOST`       | `localhost`                  | Hostname or IP address of the STAR server-side TLS proxy.               |
| `STAR_UPSTREAM_PORT`       | `8081`                       | Port of the STAR server-side TLS proxy.                                 |
| `STAR_UPSTREAM_SNI`        | same as `STAR_UPSTREAM_HOST` | TLS SNI used when connecting to the upstream proxy.                     |
| `STAR_MAX_COUNT`           | `10`                         | Maximum STAR token requests per minute for one registered credential.   |

## Build


```sh
cargo build --release
```

## Run

Start the STAR API and server-side TLS proxy first. Then start the client proxy:

```sh
export STAR_API_BASE=http://127.0.0.1:18080
export STAR_UPSTREAM_HOST=127.0.0.1
export STAR_UPSTREAM_PORT=18081
export STAR_UPSTREAM_SNI=localhost
export STAR_CLIENT_PROXY_LISTEN=127.0.0.1:18082
export STAR_MAX_COUNT=10

cargo run --manifest-path system/client/Cargo.toml --release
```

A successful startup keeps the process running and listens on
`STAR_CLIENT_PROXY_LISTEN`.

Send requests to the local client proxy, not directly to the server-side TLS
proxy:

```sh
curl -v http://127.0.0.1:18082/
```

