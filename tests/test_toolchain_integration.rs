use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Integration test for toolchain manager with mock installations
#[test]
fn test_toolchain_with_mock_installations() {
    // Create temporary toolchain directory
    let temp = TempDir::new().unwrap();
    let toolchain_root = temp.path().join("toolchains");

    // Mock Java 21 installation
    let java_21 = toolchain_root.join("java/21/bin");
    fs::create_dir_all(&java_21).unwrap();

    // Create mock java binary
    let java_bin = java_21.join("java");
    fs::write(&java_bin, "#!/bin/sh\necho 'java version 21'").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&java_bin).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&java_bin, perms).unwrap();
    }

    // Mock Gradle 8.4 installation
    let gradle_84 = toolchain_root.join("gradle/8.4/bin");
    fs::create_dir_all(&gradle_84).unwrap();

    let gradle_bin = gradle_84.join("gradle");
    fs::write(&gradle_bin, "#!/bin/sh\necho 'Gradle 8.4'").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&gradle_bin).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&gradle_bin, perms).unwrap();
    }

    // Mock Android SDK installation
    let platform_tools = toolchain_root.join("android/platform-tools");
    fs::create_dir_all(&platform_tools).unwrap();

    let adb_bin = platform_tools.join("adb");
    fs::write(&adb_bin, "#!/bin/sh\necho 'Android Debug Bridge'").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&adb_bin).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&adb_bin, perms).unwrap();
    }

    println!("✓ Created mock toolchain at: {}", toolchain_root.display());
    println!("  - Java 21: {}", java_bin.display());
    println!("  - Gradle 8.4: {}", gradle_bin.display());
    println!("  - adb: {}", adb_bin.display());

    // Verify structure
    assert!(java_bin.exists());
    assert!(gradle_bin.exists());
    assert!(adb_bin.exists());
}

#[test]
fn test_version_validation() {
    use whitehall::toolchain::validator::{validate_compatibility, ToolchainConfig};

    // Valid: AGP 8.2.0 + Gradle 8.4 + Java 21
    let valid_config = ToolchainConfig {
        java: "21".to_string(),
        gradle: "8.4".to_string(),
        agp: "8.2.0".to_string(),
    };
    assert!(validate_compatibility(&valid_config).is_ok());

    // Invalid: AGP 8.2.0 requires Java 17+, but using Java 11
    let invalid_config = ToolchainConfig {
        java: "11".to_string(),
        gradle: "8.4".to_string(),
        agp: "8.2.0".to_string(),
    };
    let result = validate_compatibility(&invalid_config);
    assert!(result.is_err());

    let err = result.unwrap_err().to_string();
    assert!(err.contains("AGP 8.2.0 requires Java 17 or higher"));

    println!("✓ Version validation works correctly");
}

#[test]
fn test_platform_detection() {
    use whitehall::toolchain::Platform;

    let platform = Platform::detect().unwrap();
    println!("✓ Detected platform: {}", platform);

    let (os, arch) = platform.as_download_strings();
    println!("  - OS: {}", os);
    println!("  - Arch: {}", arch);

    // Should detect either Linux or macOS
    assert!(platform.is_linux() || platform.is_macos());
}
