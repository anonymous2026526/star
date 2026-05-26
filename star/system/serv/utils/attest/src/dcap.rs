#![cfg(any(unix, windows))]

use crate::Result;
use parity_scale_codec::Encode;
use std::time::{SystemTime, UNIX_EPOCH};
use std::vec::Vec;

pub fn raw_evidence_for_verifier(quote: &[u8]) -> Result<Vec<u8>> {
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|err| format!("create Tokio runtime for DCAP collateral fetch: {err}"))?;
    runtime.block_on(raw_evidence_for_verifier_async(quote))
}

pub async fn raw_evidence_for_verifier_async(quote: &[u8]) -> Result<Vec<u8>> {
    let client = dcap_qvl::collateral::CollateralClient::from_env()
        .map_err(|err| format!("create PCCS client: {err}"))?;

    let collateral = client
        .fetch(quote)
        .await
        .map_err(|err| format!("fetch DCAP collateral: {err}"))?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| format!("system clock is before UNIX_EPOCH: {err}"))?
        .as_secs();
    dcap_qvl::verify::QuoteVerifier::new_prod()
        .allow_debug(true)
        .verify(quote, &collateral, now)
        .map_err(|err| format!("verify fetched DCAP collateral: {err}"))?;

    let encoded_collateral = collateral.encode();
    Ok(crate::encode_raw_quote_evidence(&crate::RawQuoteEvidence {
        quote,
        collateral: &encoded_collateral,
    }))
}
