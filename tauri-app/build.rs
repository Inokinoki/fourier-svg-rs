fn main() {
    // Ensure OUT_DIR is available during compilation for the macro
    let out_dir = std::env::var("OUT_DIR").unwrap();
    println!("cargo:rustc-env=OUT_DIR={}", out_dir);
    println!("cargo:rerun-if-changed=tauri.conf.json");
}
