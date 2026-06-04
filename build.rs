fn main() {
    // Project root contains libmpv.so symlink; also check system lib path
    println!("cargo:rustc-link-search=native={}", env!("CARGO_MANIFEST_DIR"));
    println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu");
    println!("cargo:rustc-link-search=native=/usr/lib");
}
