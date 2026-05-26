#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

pub const CHALLENGE_LEN: usize = 64;
pub const MEASUREMENT_LEN: usize = 32;
pub const PUBLIC_KEY_LEN: usize = 32;

const QUOTE_HEADER_LEN: usize = 48;
const SGX_REPORT_BODY_LEN: usize = 384;
const QUOTE_V3: u16 = 3;
const QUOTE_V5: u16 = 5;
const QUOTE_V5_BODY_TYPE_OFFSET: usize = 48;
const QUOTE_V5_BODY_SIZE_OFFSET: usize = 50;
const QUOTE_V5_BODY_OFFSET: usize = 54;
const SGX_QL_ALG_ECDSA_P256: u16 = 2;
const TEE_TYPE_SGX: u32 = 0;
const QUOTE_BODY_TYPE_SGX: u16 = 1;

pub type Result<T> = core::result::Result<T, String>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedQuote {
    pub mrenclave: [u8; MEASUREMENT_LEN],
    pub mrsigner: [u8; MEASUREMENT_LEN],
    pub report_data: [u8; CHALLENGE_LEN],
    pub debug: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotePolicy {
    pub expected_mrenclave: Option<[u8; MEASUREMENT_LEN]>,
    pub expected_mrsigner: Option<[u8; MEASUREMENT_LEN]>,
    pub allow_debug: bool,
    pub allow_advisory: bool,
}

pub fn report_data_for_public_key(public_key: [u8; PUBLIC_KEY_LEN]) -> [u8; CHALLENGE_LEN] {
    let mut report_data = [0u8; CHALLENGE_LEN];
    report_data[..PUBLIC_KEY_LEN].copy_from_slice(&public_key);
    report_data
}

pub fn public_key_from_report_data(parsed: &ParsedQuote) -> Result<[u8; PUBLIC_KEY_LEN]> {
    if parsed.report_data[PUBLIC_KEY_LEN..]
        .iter()
        .any(|byte| *byte != 0)
    {
        return Err("report_data is not public_key || zero padding".into());
    }

    parsed.report_data[..PUBLIC_KEY_LEN]
        .try_into()
        .map_err(|_| "report_data is too short for public key".into())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawQuoteEvidence<'a> {
    pub quote: &'a [u8],
    pub collateral: &'a [u8],
}

pub fn encode_raw_quote_evidence(evidence: &RawQuoteEvidence<'_>) -> Vec<u8> {
    let mut out = Vec::new();
    append_blob(&mut out, evidence.quote);
    append_blob(&mut out, evidence.collateral);
    out
}

pub fn decode_raw_quote_evidence(payload: &[u8]) -> Result<RawQuoteEvidence<'_>> {
    let mut remaining = payload;
    let quote = take_blob(&mut remaining)?;
    let collateral = take_blob(&mut remaining)?;
    if !remaining.is_empty() {
        return Err("trailing raw quote evidence bytes".into());
    }
    Ok(RawQuoteEvidence { quote, collateral })
}

pub fn verify_quote_public_key(quote: &[u8], policy: &QuotePolicy) -> Result<[u8; PUBLIC_KEY_LEN]> {
    let parsed = parse_quote(quote)?;
    verify_policy(&parsed, policy)?;
    public_key_from_report_data(&parsed)
}

pub fn verify_policy(parsed: &ParsedQuote, policy: &QuotePolicy) -> Result<()> {
    let mut has_identity_policy = false;

    if let Some(expected_mrenclave) = policy.expected_mrenclave {
        has_identity_policy = true;
        if parsed.mrenclave != expected_mrenclave {
            return Err("MRENCLAVE does not match the trusted enclave measurement".into());
        }
    }

    if let Some(expected_mrsigner) = policy.expected_mrsigner {
        has_identity_policy = true;
        if parsed.mrsigner != expected_mrsigner {
            return Err("MRSIGNER does not match the trusted enclave signer".into());
        }
    }

    if !has_identity_policy {
        return Err("no trusted enclave identity policy is configured".into());
    }

    if parsed.debug && !policy.allow_debug {
        return Err("debug enclave is not trusted".into());
    }

    Ok(())
}

pub fn parse_measurement_hex(input: &str, name: &str) -> Result<[u8; MEASUREMENT_LEN]> {
    let mut out = [0u8; MEASUREMENT_LEN];
    let mut nibbles = 0usize;

    for byte in input.bytes() {
        if matches!(byte, b':' | b'_' | b' ' | b'\t' | b'\r' | b'\n') {
            continue;
        }
        if nibbles == MEASUREMENT_LEN * 2 {
            return Err(format!("{name} is longer than 32 bytes"));
        }
        let value = hex_nibble(byte).ok_or_else(|| format!("{name} contains non-hex data"))?;
        if nibbles % 2 == 0 {
            out[nibbles / 2] = value << 4;
        } else {
            out[nibbles / 2] |= value;
        }
        nibbles += 1;
    }

    if nibbles != MEASUREMENT_LEN * 2 {
        return Err(format!("{name} must be 32 bytes of hex"));
    }

    Ok(out)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

const POLICY_HAS_MRENCLAVE: u8 = 1;
const POLICY_HAS_MRSIGNER: u8 = 2;

pub fn encode_policy(policy: &QuotePolicy) -> Vec<u8> {
    let mut identity_flags = 0u8;
    let mut capacity = 3;
    if policy.expected_mrenclave.is_some() {
        identity_flags |= POLICY_HAS_MRENCLAVE;
        capacity += MEASUREMENT_LEN;
    }
    if policy.expected_mrsigner.is_some() {
        identity_flags |= POLICY_HAS_MRSIGNER;
        capacity += MEASUREMENT_LEN;
    }

    let mut out = Vec::with_capacity(capacity);
    out.push(identity_flags);
    out.push(u8::from(policy.allow_debug));
    out.push(u8::from(policy.allow_advisory));
    if let Some(expected_mrenclave) = policy.expected_mrenclave {
        out.extend_from_slice(&expected_mrenclave);
    }
    if let Some(expected_mrsigner) = policy.expected_mrsigner {
        out.extend_from_slice(&expected_mrsigner);
    }
    out
}

pub fn encode_policy_and_quote(policy: &QuotePolicy, quote: &[u8]) -> Vec<u8> {
    let mut out = encode_policy(policy);
    append_blob(&mut out, quote);
    out
}

pub fn decode_policy_and_quote(payload: &[u8]) -> Result<(QuotePolicy, &[u8])> {
    let mut remaining = payload;
    let policy = take_policy(&mut remaining)?;
    let quote = take_blob(&mut remaining)?;
    if !remaining.is_empty() {
        return Err("trailing policy payload bytes".into());
    }

    Ok((policy, quote))
}

pub fn take_policy(payload: &mut &[u8]) -> Result<QuotePolicy> {
    let min_len = 3;
    if payload.len() < min_len {
        return Err("policy payload is too short".into());
    }

    let identity_flags = payload[0];
    if identity_flags & !(POLICY_HAS_MRENCLAVE | POLICY_HAS_MRSIGNER) != 0 {
        return Err("invalid policy identity flags".into());
    }
    let allow_debug = match payload[1] {
        0 => false,
        1 => true,
        _ => return Err("invalid debug policy flag".into()),
    };
    let allow_advisory = match payload[2] {
        0 => false,
        1 => true,
        _ => return Err("invalid advisory policy flag".into()),
    };

    let mut remaining = &payload[min_len..];
    let expected_mrenclave = if identity_flags & POLICY_HAS_MRENCLAVE != 0 {
        Some(take_measurement(&mut remaining, "MRENCLAVE")?)
    } else {
        None
    };
    let expected_mrsigner = if identity_flags & POLICY_HAS_MRSIGNER != 0 {
        Some(take_measurement(&mut remaining, "MRSIGNER")?)
    } else {
        None
    };

    *payload = remaining;

    Ok(QuotePolicy {
        expected_mrenclave,
        expected_mrsigner,
        allow_debug,
        allow_advisory,
    })
}

fn take_measurement(payload: &mut &[u8], name: &str) -> Result<[u8; MEASUREMENT_LEN]> {
    if payload.len() < MEASUREMENT_LEN {
        return Err(format!("policy payload is too short for {name}"));
    }
    let measurement = payload[..MEASUREMENT_LEN]
        .try_into()
        .map_err(|_| format!("bad {name} length"))?;
    *payload = &payload[MEASUREMENT_LEN..];
    Ok(measurement)
}

pub fn append_blob(payload: &mut Vec<u8>, blob: &[u8]) {
    payload.extend_from_slice(&(blob.len() as u32).to_be_bytes());
    payload.extend_from_slice(blob);
}

pub fn take_blob<'a>(payload: &mut &'a [u8]) -> Result<&'a [u8]> {
    let len: [u8; 4] = payload
        .get(..4)
        .ok_or_else(|| "missing blob length".to_string())?
        .try_into()
        .map_err(|_| "bad blob length".to_string())?;

    let len = u32::from_be_bytes(len) as usize;
    let blob = payload
        .get(4..4 + len)
        .ok_or_else(|| "truncated blob".to_string())?;
    *payload = &payload[4 + len..];
    Ok(blob)
}

#[cfg(all(feature = "host-dcap", any(unix, windows)))]
pub mod dcap;

#[cfg(feature = "dcap-verify")]
pub fn verify_raw_quote_evidence_public_key(
    payload: &[u8],
    policy: &QuotePolicy,
) -> Result<[u8; PUBLIC_KEY_LEN]> {
    let evidence = decode_raw_quote_evidence(payload)?;
    let mut collateral_bytes = evidence.collateral;
    let collateral =
        <dcap_qvl::QuoteCollateralV3 as parity_scale_codec::Decode>::decode(&mut collateral_bytes)
            .map_err(|err| format!("parse DCAP collateral: {err}"))?;
    if !collateral_bytes.is_empty() {
        return Err("trailing DCAP collateral bytes".into());
    }

    let parsed = parse_quote(evidence.quote)?;
    verify_policy(&parsed, policy)?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| format!("system clock is before UNIX_EPOCH: {err}"))?
        .as_secs();
    let verified = dcap_qvl::verify::QuoteVerifier::new_prod()
        .allow_debug(policy.allow_debug)
        .verify(evidence.quote, &collateral, now)
        .map_err(|err| format!("verify DCAP quote: {err}"))?;

    let final_status = verified.platform_status.clone().merge(&verified.qe_status);
    validate_dcap_status(final_status.status, policy.allow_advisory)?;

    public_key_from_report_data(&parsed)
}

#[cfg(feature = "dcap-verify")]
fn validate_dcap_status(status: dcap_qvl::tcb_info::TcbStatus, allow_advisory: bool) -> Result<()> {
    use dcap_qvl::tcb_info::TcbStatus;

    match status {
        TcbStatus::UpToDate => Ok(()),
        TcbStatus::Revoked => Err("quote TCB status is Revoked".into()),
        _ if allow_advisory => Ok(()),
        _ => Err(format!(
            "quote TCB status is {status}; advisory status is not allowed"
        )),
    }
}

pub fn parse_quote(quote: &[u8]) -> Result<ParsedQuote> {
    let report_body_offset = sgx_report_body_offset(quote)?;

    let attributes_offset = report_body_offset + 48;
    let mrenclave_offset = report_body_offset + 64;
    let mrsigner_offset = report_body_offset + 128;
    let report_data_offset = report_body_offset + 320;

    let attributes_flags = read_u64_le(quote, attributes_offset)?;

    Ok(ParsedQuote {
        mrenclave: read_array(quote, mrenclave_offset, "MRENCLAVE")?,
        mrsigner: read_array(quote, mrsigner_offset, "MRSIGNER")?,
        report_data: read_array(quote, report_data_offset, "report_data")?,
        debug: attributes_flags & 0x2 != 0,
    })
}

fn sgx_report_body_offset(quote: &[u8]) -> Result<usize> {
    if quote.len() < QUOTE_HEADER_LEN {
        return Err("quote is too short".into());
    }

    let version = read_u16_le(quote, 0)?;
    let attestation_key_type = read_u16_le(quote, 2)?;
    if attestation_key_type != SGX_QL_ALG_ECDSA_P256 {
        return Err("quote attestation key is not ECDSA/P-256".into());
    }

    let report_body_offset = match version {
        QUOTE_V3 => {
            let attestation_key_data = read_u32_le(quote, 4)?;
            if attestation_key_data != 0 {
                return Err("SGX quote v3 attestation key data must be zero".into());
            }
            QUOTE_HEADER_LEN
        }
        QUOTE_V5 => {
            let tee_type = read_u32_le(quote, 4)?;
            if tee_type != TEE_TYPE_SGX {
                return Err("quote is not for SGX".into());
            }
            let body_type = read_u16_le(quote, QUOTE_V5_BODY_TYPE_OFFSET)?;
            if body_type != QUOTE_BODY_TYPE_SGX {
                return Err("quote v5 body is not an SGX report".into());
            }
            let body_size = read_u32_le(quote, QUOTE_V5_BODY_SIZE_OFFSET)? as usize;
            if body_size < SGX_REPORT_BODY_LEN {
                return Err("quote v5 SGX body is too short".into());
            }
            if quote.len() < QUOTE_V5_BODY_OFFSET + body_size {
                return Err("quote v5 body size exceeds quote length".into());
            }
            QUOTE_V5_BODY_OFFSET
        }
        4 => return Err("TDX quote v4 is not an SGX enclave quote".into()),
        _ => return Err(format!("unsupported quote version: {version}")),
    };

    if quote.len() < report_body_offset + SGX_REPORT_BODY_LEN {
        return Err("quote is too short".into());
    }

    Ok(report_body_offset)
}

fn read_array<const N: usize>(bytes: &[u8], offset: usize, name: &str) -> Result<[u8; N]> {
    bytes
        .get(offset..offset + N)
        .ok_or_else(|| format!("quote is too short for {name}"))?
        .try_into()
        .map_err(|_| format!("quote is too short for {name}"))
}

fn read_u16_le(bytes: &[u8], offset: usize) -> Result<u16> {
    let bytes: [u8; 2] = read_array(bytes, offset, "u16")?;
    Ok(u16::from_le_bytes(bytes))
}

fn read_u32_le(bytes: &[u8], offset: usize) -> Result<u32> {
    let bytes: [u8; 4] = read_array(bytes, offset, "u32")?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_u64_le(bytes: &[u8], offset: usize) -> Result<u64> {
    let bytes: [u8; 8] = read_array(bytes, offset, "u64")?;
    Ok(u64::from_le_bytes(bytes))
}

#[path = "lib_test.rs"]
#[cfg(test)]
mod tests;
