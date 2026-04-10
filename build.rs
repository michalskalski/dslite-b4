fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "illumos" {
        println!("cargo:rustc-link-lib=dladm");
    }
}
