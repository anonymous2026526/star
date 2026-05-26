# STAR SGX Attestation Tools

This crate verifies remote SGX attestation quotes produced by the service. The
service prints the quote on startup as URL-safe base64:

```text
STAR_ATTESTATION_QUOTE_BASE64=<quote>
```

The quote `report_data` is `public_key || 32 zero bytes`, so verification binds
the enclave measurement to the enclave public key.

## Commands

Inspect measurements and the attested public key:

```bash
cargo run --manifest-path tools/verifier/Cargo.toml -- inspect < quote.txt
```

Verify a quote:

```bash
cargo run --manifest-path tools/verifier/Cargo.toml -- verify \
  --mrsigner <32-byte hex MRSIGNER> \
  --mrenclave <32-byte hex MRENCLAVE> \
  --public-key <32-byte hex public key> \
  --allow-debug \
  --allow-advisory \
  --pccs-url <PCCS URL> \
  < quote.txt
```

`--mrenclave` and `--public-key` are optional if signer-only policy and
displaying the attested key are acceptable. `--allow-debug` must only be used for
development enclaves. `--allow-advisory` accepts Intel DCAP QVL results such as
`ConfigurationNeeded`, `OutOfDate`, and `SWHardeningNeeded`; omit it when the
policy requires an up-to-date `UpToDate` result.

The verifier uses the pure Rust `dcap-qvl` crate and fetches collateral from
`--pccs-url`, `PCCS_URL`, or the crate's default PCCS URL.
