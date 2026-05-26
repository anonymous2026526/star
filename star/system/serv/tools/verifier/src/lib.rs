use std::time::{SystemTime, UNIX_EPOCH};

use base64::{
    Engine as _,
    engine::general_purpose::{URL_SAFE, URL_SAFE_NO_PAD},
};
use dcap_qvl::collateral::CollateralClient;
use dcap_qvl::quote::Quote;
use dcap_qvl::tcb_info::TcbStatus;
use dcap_qvl::verify::{QuoteVerifier, VerifiedReport};
use parity_scale_codec::Decode;

pub const CHALLENGE_LEN: usize = 64;
pub const MEASUREMENT_LEN: usize = 32;
pub const PUBLIC_KEY_LEN: usize = 32;

const SGX_QL_ALG_ECDSA_P256: u16 = 2;

pub type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedQuote {
    pub mrenclave: [u8; MEASUREMENT_LEN],
    pub mrsigner: [u8; MEASUREMENT_LEN],
    pub report_data: [u8; CHALLENGE_LEN],
    pub debug: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DcapVerification {
    pub status: TcbStatus,
    pub advisory_ids: Vec<String>,
    pub qe_status: TcbStatus,
    pub platform_status: TcbStatus,
}

#[derive(Debug, Clone)]
pub struct QuoteVerificationPolicy<'a> {
    pub expected_mrsigner: &'a [u8; MEASUREMENT_LEN],
    pub expected_mrenclave: Option<&'a [u8; MEASUREMENT_LEN]>,
    pub expected_public_key: Option<&'a [u8; PUBLIC_KEY_LEN]>,
    pub allow_debug: bool,
    pub allow_advisory: bool,
    pub pccs_url: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedQuote {
    pub parsed: ParsedQuote,
    pub public_key: [u8; PUBLIC_KEY_LEN],
    pub dcap: DcapVerification,
}

pub fn verify_quote(quote: &[u8], policy: &QuoteVerificationPolicy<'_>) -> Result<VerifiedQuote> {
    let parsed = parse_quote(quote)?;
    let public_key = public_key_from_report_data(&parsed)?;
    if let Some(expected_public_key) = policy.expected_public_key {
        if &public_key != expected_public_key {
            return Err("attested public key does not match expected public key".to_string());
        }
    }
    verify_policy(
        &parsed,
        policy.expected_mrsigner,
        policy.expected_mrenclave,
        policy.allow_debug,
    )?;

    let dcap = verify_quote_with_dcap_qvl(quote, policy.pccs_url, policy.allow_debug)?;
    validate_dcap_status(dcap.status, policy.allow_advisory)?;

    Ok(VerifiedQuote {
        parsed,
        public_key,
        dcap,
    })
}

pub fn verify_quote_with_dcap_qvl(
    quote: &[u8],
    pccs_url: Option<&str>,
    allow_debug: bool,
) -> Result<DcapVerification> {
    let verified = fetch_collateral_and_verify_quote(quote, pccs_url, allow_debug)?;
    let final_status = verified.platform_status.clone().merge(&verified.qe_status);

    Ok(DcapVerification {
        status: final_status.status,
        advisory_ids: final_status.advisory_ids,
        qe_status: verified.qe_status.status,
        platform_status: verified.platform_status.status,
    })
}

fn fetch_collateral_and_verify_quote(
    quote: &[u8],
    pccs_url: Option<&str>,
    allow_debug: bool,
) -> Result<VerifiedReport> {
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|err| format!("create Tokio runtime for DCAP collateral fetch: {err}"))?;

    runtime.block_on(async {
        let client = match pccs_url {
            Some(url) => CollateralClient::with_default_http(url)
                .map_err(|err| format!("create PCCS client: {err}"))?,
            None => {
                CollateralClient::from_env().map_err(|err| format!("create PCCS client: {err}"))?
            }
        };
        let collateral = client
            .fetch(quote)
            .await
            .map_err(|err| format!("fetch DCAP collateral: {err}"))?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| format!("system clock is before UNIX_EPOCH: {err}"))?
            .as_secs();

        QuoteVerifier::new_prod()
            .allow_debug(allow_debug)
            .verify(quote, &collateral, now)
            .map_err(|err| format!("verify DCAP quote with dcap-qvl: {err}"))
    })
}

pub fn parse_quote(quote: &[u8]) -> Result<ParsedQuote> {
    let quote = decode_dcap_quote(quote)?;
    if quote.header.attestation_key_type != SGX_QL_ALG_ECDSA_P256 {
        return Err("quote attestation key is not ECDSA/P-256".to_string());
    }

    let report = quote
        .report
        .as_sgx()
        .ok_or_else(|| "quote is not for an SGX enclave".to_string())?;
    Ok(ParsedQuote {
        mrenclave: report.mr_enclave,
        mrsigner: report.mr_signer,
        report_data: report.report_data,
        debug: report.attributes[0] & 0x02 != 0,
    })
}

fn decode_dcap_quote(quote: &[u8]) -> Result<Quote> {
    Quote::decode(&mut &quote[..]).map_err(|err| format!("parse DCAP quote: {err}"))
}

pub fn public_key_from_report_data(parsed: &ParsedQuote) -> Result<[u8; PUBLIC_KEY_LEN]> {
    if parsed.report_data[PUBLIC_KEY_LEN..]
        .iter()
        .any(|byte| *byte != 0)
    {
        return Err("report_data is not public_key || zero padding".to_string());
    }

    parsed.report_data[..PUBLIC_KEY_LEN]
        .try_into()
        .map_err(|_| "report_data is too short for public key".to_string())
}

pub fn verify_policy(
    parsed: &ParsedQuote,
    expected_mrsigner: &[u8; MEASUREMENT_LEN],
    expected_mrenclave: Option<&[u8; MEASUREMENT_LEN]>,
    allow_debug: bool,
) -> Result<()> {
    if &parsed.mrsigner != expected_mrsigner {
        return Err("MRSIGNER does not match the trusted signer".to_string());
    }
    if let Some(expected_mrenclave) = expected_mrenclave {
        if &parsed.mrenclave != expected_mrenclave {
            return Err("MRENCLAVE does not match the trusted enclave measurement".to_string());
        }
    }
    if parsed.debug && !allow_debug {
        return Err(
            "debug enclave is not trusted; pass --allow-debug only for development".to_string(),
        );
    }
    Ok(())
}

pub fn parse_hex_array<const N: usize>(input: &str, name: &str) -> Result<[u8; N]> {
    let mut cleaned = String::with_capacity(input.len());
    let mut input = input.trim();
    if let Some(stripped) = input
        .strip_prefix("0x")
        .or_else(|| input.strip_prefix("0X"))
    {
        input = stripped;
    }

    for ch in input.chars() {
        if ch == ':' || ch == '_' || ch.is_ascii_whitespace() {
            continue;
        }
        if !ch.is_ascii_hexdigit() {
            return Err(format!("{name} contains non-hex character {ch:?}"));
        }
        cleaned.push(ch);
    }

    if cleaned.len() != N * 2 {
        return Err(format!(
            "{name} must be {} hex characters for {N} bytes, got {}",
            N * 2,
            cleaned.len()
        ));
    }

    let mut out = [0u8; N];
    for (index, byte) in out.iter_mut().enumerate() {
        let offset = index * 2;
        *byte = u8::from_str_radix(&cleaned[offset..offset + 2], 16)
            .map_err(|err| format!("parse {name} hex: {err}"))?;
    }
    Ok(out)
}

pub fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

pub fn decode_base64(input: &str) -> Result<Vec<u8>> {
    let input = input.trim();
    URL_SAFE
        .decode(input)
        .or_else(|_| URL_SAFE_NO_PAD.decode(input))
        .map_err(|err| format!("decode base64: {err}"))
}

pub fn decode_quote_input(input: &[u8]) -> Result<Vec<u8>> {
    if parse_quote(input).is_ok() {
        return Ok(input.to_vec());
    }

    let text = std::str::from_utf8(input)
        .map_err(|err| format!("stdin is neither a raw quote nor UTF-8 base64 text: {err}"))?;

    let trimmed = text.trim();
    if !trimmed.is_empty() {
        if let Ok(quote) = decode_base64(trimmed) {
            if parse_quote(&quote).is_ok() {
                return Ok(quote);
            }
        }
    }

    for token in text.split_whitespace().rev() {
        for candidate in quote_token_candidates(token) {
            if let Ok(quote) = decode_base64(candidate) {
                if parse_quote(&quote).is_ok() {
                    return Ok(quote);
                }
            }
        }
    }

    Err("could not find a parseable SGX quote in stdin".to_string())
}

fn quote_token_candidates(token: &str) -> impl Iterator<Item = &str> {
    let token = token.trim_matches(|ch: char| {
        ch == '"' || ch == '\'' || ch == ',' || ch == ';' || ch == '[' || ch == ']'
    });
    [
        Some(token),
        token.split_once('=').map(|(_, value)| value),
        token.rsplit_once(':').map(|(_, value)| value),
    ]
    .into_iter()
    .flatten()
    .filter(|candidate| !candidate.is_empty())
}

pub fn dcap_status_name(status: TcbStatus) -> &'static str {
    match status {
        TcbStatus::UpToDate => "UpToDate",
        TcbStatus::ConfigurationNeeded => "ConfigurationNeeded",
        TcbStatus::SWHardeningNeeded => "SWHardeningNeeded",
        TcbStatus::ConfigurationAndSWHardeningNeeded => "ConfigurationAndSWHardeningNeeded",
        TcbStatus::OutOfDate => "OutOfDate",
        TcbStatus::OutOfDateConfigurationNeeded => "OutOfDateConfigurationNeeded",
        TcbStatus::Revoked => "Revoked",
    }
}

pub fn dcap_status_is_ok(status: TcbStatus) -> bool {
    status == TcbStatus::UpToDate
}

pub fn dcap_status_is_advisory(status: TcbStatus) -> bool {
    matches!(
        status,
        TcbStatus::ConfigurationNeeded
            | TcbStatus::SWHardeningNeeded
            | TcbStatus::ConfigurationAndSWHardeningNeeded
            | TcbStatus::OutOfDate
            | TcbStatus::OutOfDateConfigurationNeeded
    )
}

pub fn validate_dcap_status(status: TcbStatus, allow_advisory: bool) -> Result<()> {
    if dcap_status_is_ok(status) || (allow_advisory && dcap_status_is_advisory(status)) {
        return Ok(());
    }

    Err(format!("quote TCB status is {}", dcap_status_name(status)))
}

#[path = "lib_test.rs"]
#[cfg(test)]
mod tests;
