# STAR

STAR: Scalable Anonymous Rate-Limiting with a Server-Side TEE

STAR is an experimental implementation of anonymous rate limiting using a server-side TEE.

It allows a service to enforce per-user access limits without linking authentication requests to the same user. STAR is designed around three security goals: rate-limiting, unforgeability, and unlinkability. :contentReference[oaicite:0]{index=0}

## What is included

| Path | Role |
| --- | --- |
| [`core/`](core/) | Core Library of the STAR |
| [`core/analysis/`](core/analysis/) | Side-channel checks / Timecop-style analysis |
| [`system/client/`](system/client/) | Client-side Proxy |
| [`system/serv/`](system/serv/) | Rate-Limiting Service |
| [`system/serv/app/`](system/serv/app/) | Filtering Proxy / Registration API |
| [`system/serv/app/examples/`](system/serv/app/examples/) | Web App Server and local demo |
| [`system/serv/enclave/`](system/serv/enclave/) | Auth SGX Enclave |
| [`system/serv/integration/`](system/serv/integration/) | Integration tests / reproduction scripts |


## Start here

### Read

| Purpose | Entry point |
| --- | --- |
| Core library | [`core/README.md`](core/README.md) |
| Client proxy | [`system/client/README.md`](system/client/README.md) |
| Server-side components | [`system/serv/README.md`](system/serv/README.md) |

### Run / verify

| Purpose | Entry point |
| --- | --- |
| Side-channel checks | [`core/analysis/run.sh`](core/analysis/run.sh) |
| Local demo | [`system/serv/app/examples/deploy.sh`](system/serv/app/examples/deploy.sh) |
| Integration flow | [`system/serv/integration/run-all.sh`](system/serv/integration/run-all.sh) |
| SGX quote inspection / verification | [`system/serv/tools/verifier/`](system/serv/tools/verifier/) |


