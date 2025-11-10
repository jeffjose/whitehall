// Auto-generated build script for Whitehall FFI
// This copies the generated JNI bridge from build/ to OUT_DIR

use std::{env, fs, path::Path};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let bridge_src = Path::new(r"/home/jeffjose/scripts/whitehall/examples/ffi-rust/build/generated/jni/rust/jni_bridge.rs");
    let bridge_dst = Path::new(&out_dir).join("jni_bridge.rs");

    // Copy generated bridge to OUT_DIR
    fs::copy(bridge_src, bridge_dst)
        .expect("Failed to copy jni_bridge.rs to OUT_DIR");

    // Re-run if the bridge changes
    println!("cargo:rerun-if-changed={}", bridge_src.display());
}
