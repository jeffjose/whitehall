// Auto-generated build script for Whitehall FFI
// This copies the generated JNI bridge from build/ to OUT_DIR

use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    // CARGO_MANIFEST_DIR points to the directory containing Cargo.toml (src/ffi/rust/)
    // Navigate up to project root, then to the generated bridge file
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let project_root = manifest_dir.ancestors().nth(3).unwrap(); // src/ffi/rust -> src/ffi -> src -> project_root
    let bridge_src = project_root.join("build/generated/jni/rust/jni_bridge.rs");
    let bridge_dst = PathBuf::from(&out_dir).join("jni_bridge.rs");

    // Copy generated bridge to OUT_DIR
    fs::copy(&bridge_src, &bridge_dst)
        .expect("Failed to copy jni_bridge.rs to OUT_DIR");

    // Re-run if the bridge changes
    println!("cargo:rerun-if-changed={}", bridge_src.display());
}
