// Example program to test the toolchain manager
// Run with: cargo run --example test-toolchain

use whitehall::toolchain::{Toolchain, Platform, validate_compatibility};
use whitehall::toolchain::validator::ToolchainConfig;

fn main() {
    println!("=== Whitehall Toolchain Manager - Phase 1 Test ===\n");

    // Test 1: Platform Detection
    println!("1. Platform Detection:");
    match Platform::detect() {
        Ok(platform) => {
            println!("   ✓ Detected: {}", platform);
            let (os, arch) = platform.as_download_strings();
            println!("   - OS: {}, Arch: {}", os, arch);
        }
        Err(e) => println!("   ✗ Error: {}", e),
    }

    // Test 2: Version Validation (Valid)
    println!("\n2. Version Validation (Valid Config):");
    let valid = ToolchainConfig {
        java: "21".to_string(),
        gradle: "8.4".to_string(),
        agp: "8.2.0".to_string(),
    };
    match validate_compatibility(&valid) {
        Ok(_) => println!("   ✓ Java 21 + Gradle 8.4 + AGP 8.2.0 is compatible"),
        Err(e) => println!("   ✗ Error: {}", e),
    }

    // Test 3: Version Validation (Invalid)
    println!("\n3. Version Validation (Invalid Config):");
    let invalid = ToolchainConfig {
        java: "11".to_string(),
        gradle: "8.4".to_string(),
        agp: "8.2.0".to_string(),
    };
    match validate_compatibility(&invalid) {
        Ok(_) => println!("   ✗ Should have failed!"),
        Err(e) => {
            println!("   ✓ Correctly rejected:");
            println!("   {}", e);
        }
    }

    // Test 4: Toolchain Manager
    println!("\n4. Toolchain Manager:");
    match Toolchain::new() {
        Ok(toolchain) => {
            println!("   ✓ Initialized at: {}", toolchain.root().display());

            // Try to find Java 21
            println!("\n   Checking for Java 21...");
            match toolchain.ensure_java("21") {
                Ok(java_home) => {
                    println!("   ✓ Found Java 21 at: {}", java_home.display());
                }
                Err(e) => {
                    println!("   ✗ Not installed (expected in Phase 1):");
                    println!("   {}", e);
                }
            }

            // Try to find Gradle 8.4
            println!("\n   Checking for Gradle 8.4...");
            match toolchain.ensure_gradle("8.4") {
                Ok(gradle_bin) => {
                    println!("   ✓ Found Gradle 8.4 at: {}", gradle_bin.display());
                }
                Err(e) => {
                    println!("   ✗ Not installed (expected in Phase 1):");
                    println!("   {}", e);
                }
            }

            // Try to find Android SDK
            println!("\n   Checking for Android SDK...");
            match toolchain.ensure_android_sdk() {
                Ok(sdk_root) => {
                    println!("   ✓ Found Android SDK at: {}", sdk_root.display());
                }
                Err(e) => {
                    println!("   ✗ Not installed (expected in Phase 1):");
                    println!("   {}", e);
                }
            }
        }
        Err(e) => println!("   ✗ Error: {}", e),
    }

    println!("\n=== Test Complete ===");
    println!("\nPhase 1 Status: ✓ Foundation working");
    println!("Next: Phase 2 will add automatic downloads");
}
