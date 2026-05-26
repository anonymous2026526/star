use std::env;
use std::io::{self, Read};

use star_tools::{
    DcapVerification, MEASUREMENT_LEN, PUBLIC_KEY_LEN, ParsedQuote, QuoteVerificationPolicy,
    VerifiedQuote, dcap_status_is_ok, dcap_status_name, decode_quote_input, hex, parse_hex_array,
    parse_quote, public_key_from_report_data, verify_quote,
};

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        print_usage();
        return Err("missing command".to_string());
    }

    let command = args.remove(0);
    match command.as_str() {
        "verify" => command_verify(args),
        "inspect" => command_inspect(args),
        "help" | "--help" | "-h" => {
            print_usage();
            Ok(())
        }
        _ => {
            print_usage();
            Err(format!("unknown command {command:?}"))
        }
    }
}

fn command_verify(mut args: Vec<String>) -> Result<(), String> {
    let expected_mrsigner = required_hex::<MEASUREMENT_LEN>(&mut args, "--mrsigner", "MRSIGNER")?;
    let expected_mrenclave =
        optional_hex::<MEASUREMENT_LEN>(&mut args, "--mrenclave", "MRENCLAVE")?;
    let expected_public_key =
        optional_hex::<PUBLIC_KEY_LEN>(&mut args, "--public-key", "public key")?;
    let allow_debug = take_flag(&mut args, "--allow-debug");
    let allow_advisory = take_flag(&mut args, "--allow-advisory");
    let pccs_url = take_value(&mut args, "--pccs-url")?;
    reject_extra(&args)?;

    let quote = read_stdin_quote()?;
    let policy = QuoteVerificationPolicy {
        expected_mrsigner: &expected_mrsigner,
        expected_mrenclave: expected_mrenclave.as_ref(),
        expected_public_key: expected_public_key.as_ref(),
        allow_debug,
        allow_advisory,
        pccs_url: pccs_url.as_deref(),
    };
    let verified = verify_quote(&quote, &policy)?;
    print_verified_quote_summary(&quote, &verified);
    print_dcap_status(&verified.dcap);
    println!("report_data public key binding OK");
    println!("local policy OK");
    Ok(())
}

fn command_inspect(args: Vec<String>) -> Result<(), String> {
    reject_extra(&args)?;

    let quote = read_stdin_quote()?;
    print_quote_summary(&quote)?;
    Ok(())
}

fn print_dcap_status(dcap: &DcapVerification) {
    if dcap_status_is_ok(dcap.status) {
        println!(
            "DCAP quote verification OK: {}",
            dcap_status_name(dcap.status)
        );
    } else {
        println!(
            "DCAP quote verification advisory accepted: {}",
            dcap_status_name(dcap.status)
        );
    }
    if !dcap.advisory_ids.is_empty() {
        println!("advisory IDs: {}", dcap.advisory_ids.join(","));
    }
}

fn print_quote_summary(quote: &[u8]) -> Result<ParsedQuote, String> {
    let parsed = parse_quote(quote)?;
    let public_key = public_key_from_report_data(&parsed)?;
    print_summary_fields(quote.len(), &parsed, &public_key);
    Ok(parsed)
}

fn print_verified_quote_summary(quote: &[u8], verified: &VerifiedQuote) {
    print_summary_fields(quote.len(), &verified.parsed, &verified.public_key);
}

fn print_summary_fields(quote_len: usize, parsed: &ParsedQuote, public_key: &[u8; PUBLIC_KEY_LEN]) {
    println!("quote size: {quote_len} bytes");
    println!("public key: {}", hex(public_key));
    println!("mrenclave: {}", hex(&parsed.mrenclave));
    println!("mrsigner : {}", hex(&parsed.mrsigner));
    println!("debug    : {}", parsed.debug);
}

fn read_stdin_quote() -> Result<Vec<u8>, String> {
    let mut input = Vec::new();
    io::stdin()
        .read_to_end(&mut input)
        .map_err(|err| format!("read quote from stdin: {err}"))?;
    decode_quote_input(&input)
}

fn required_hex<const N: usize>(
    args: &mut Vec<String>,
    flag: &str,
    name: &str,
) -> Result<[u8; N], String> {
    let value = required_value(args, flag)?;
    parse_hex_array(&value, name)
}

fn optional_hex<const N: usize>(
    args: &mut Vec<String>,
    flag: &str,
    name: &str,
) -> Result<Option<[u8; N]>, String> {
    take_value(args, flag)?
        .map(|value| parse_hex_array(&value, name))
        .transpose()
}

fn required_value(args: &mut Vec<String>, flag: &str) -> Result<String, String> {
    take_value(args, flag)?.ok_or_else(|| format!("missing required {flag}"))
}

fn take_value(args: &mut Vec<String>, flag: &str) -> Result<Option<String>, String> {
    if let Some(index) = args.iter().position(|arg| arg == flag) {
        args.remove(index);
        if index >= args.len() {
            return Err(format!("{flag} needs a value"));
        }
        return Ok(Some(args.remove(index)));
    }

    let prefix = format!("{flag}=");
    if let Some(index) = args.iter().position(|arg| arg.starts_with(&prefix)) {
        let value = args.remove(index)[prefix.len()..].to_string();
        return Ok(Some(value));
    }

    Ok(None)
}

fn take_flag(args: &mut Vec<String>, flag: &str) -> bool {
    if let Some(index) = args.iter().position(|arg| arg == flag) {
        args.remove(index);
        true
    } else {
        false
    }
}

fn reject_extra(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(format!("unexpected arguments: {}", args.join(" ")))
    }
}

fn print_usage() {
    eprintln!(
        "Usage:
  star_tools inspect < quote_or_startup_log.txt
  star_tools verify --mrsigner HEX [--mrenclave HEX] [--public-key HEX] [--allow-debug] [--allow-advisory] [--pccs-url URL] < quote_or_startup_log.txt"
    );
}
