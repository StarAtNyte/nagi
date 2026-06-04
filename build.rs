fn main() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    match target_os.as_str() {
        "linux" => {
            // libmpv.so symlink in project root; also search system paths
            println!("cargo:rustc-link-search=native={}", manifest);
            println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu");
            println!("cargo:rustc-link-search=native=/usr/lib");
        }
        "windows" => {
            // mpv.lib (MSVC import library) placed in wix/ by CI
            println!("cargo:rustc-link-search=native={}/wix", manifest);
        }
        _ => {}
    }
}
