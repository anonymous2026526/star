fn main() {
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("sgx") {
        println!("cargo:rustc-link-arg=--allow-multiple-definition");
    }
}
