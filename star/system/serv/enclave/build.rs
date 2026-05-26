use std::{env, fs, path::PathBuf};

const DEFAULT_MAX_COUNT: u32 = 100;
const STAR_MAX_COUNT: &str = "STAR_MAX_COUNT";

fn main() {
    println!("cargo:rerun-if-env-changed={STAR_MAX_COUNT}");
    write_max_count_config();

    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("sgx") {
        println!("cargo:rustc-link-arg=--allow-multiple-definition");
    }
}

fn write_max_count_config() {
    let max_count = match env::var(STAR_MAX_COUNT) {
        Ok(value) => parse_max_count(&value),
        Err(env::VarError::NotPresent) => DEFAULT_MAX_COUNT,
        Err(env::VarError::NotUnicode(value)) => {
            panic!("{STAR_MAX_COUNT} must be valid UTF-8, got {value:?}")
        }
    };

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by cargo"));
    let config = format!("pub(crate) const MAX_COUNT: u32 = {max_count};\n");
    fs::write(out_dir.join("max_count.rs"), config).expect("write max count config");
}

fn parse_max_count(value: &str) -> u32 {
    let value = value.trim();
    assert!(!value.is_empty(), "{STAR_MAX_COUNT} must not be empty");
    value
        .parse::<u32>()
        .unwrap_or_else(|_| panic!("{STAR_MAX_COUNT} must be a decimal u32, got {value:?}"))
}
