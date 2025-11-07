use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::ffi_parser::cpp::{discover_cpp_ffi, CppFfiFunction};
use crate::ffi_codegen::{generate_kotlin_object, generate_jni_bridge, generate_cmake};

/// Build FFI components if enabled
pub fn build_ffi(config: &Config, project_root: &Path) -> Result<()> {
    // Check if FFI is enabled
    if !should_build_ffi(config, project_root) {
        return Ok(());
    }

    let ffi_dir = project_root.join("src/ffi");
    let build_dir = project_root.join(&config.build.output_dir);

    // Phase 1: Only support C++ primitives
    if ffi_dir.join("cpp").exists() {
        build_cpp_ffi(config, &ffi_dir, &build_dir)?;
    }

    // TODO: Phase 4: Rust support
    // if ffi_dir.join("rust").exists() {
    //     build_rust_ffi(config, &ffi_dir, &build_dir)?;
    // }

    Ok(())
}

/// Determine if FFI should be built
fn should_build_ffi(config: &Config, project_root: &Path) -> bool {
    match config.ffi.enabled {
        Some(true) => true,
        Some(false) => false,
        None => {
            // Auto-detect: check if src/ffi/cpp/ or src/ffi/rust/ exists
            let ffi_dir = project_root.join("src/ffi");
            ffi_dir.join("cpp").exists() || ffi_dir.join("rust").exists()
        }
    }
}

/// Build C++ FFI components
fn build_cpp_ffi(config: &Config, ffi_dir: &Path, build_dir: &Path) -> Result<()> {
    // 1. Discover FFI functions
    let functions = discover_cpp_ffi(ffi_dir)
        .context("Failed to discover C++ FFI functions")?;

    if functions.is_empty() {
        return Ok(());
    }

    // 2. Generate Kotlin bindings
    let kotlin_dir = build_dir.join("generated/kotlin");
    let package_path = config.android.package.replace('.', "/");
    let kotlin_package_dir = kotlin_dir.join(&package_path).join("ffi");

    fs::create_dir_all(&kotlin_package_dir)
        .context("Failed to create Kotlin output directory")?;

    let kotlin_package = format!("{}.ffi", config.android.package);
    let kotlin_code = generate_kotlin_object(
        &functions,
        &kotlin_package,
        "math", // TODO: Make this configurable
        "Math",
    );

    let kotlin_file = kotlin_package_dir.join("Math.kt");
    fs::write(&kotlin_file, kotlin_code)
        .context(format!("Failed to write Kotlin binding: {}", kotlin_file.display()))?;

    // 3. Generate JNI bridge
    let jni_dir = build_dir.join("generated/jni");
    fs::create_dir_all(&jni_dir)
        .context("Failed to create JNI output directory")?;

    let source_files: Vec<String> = functions
        .iter()
        .map(|f| {
            // Get relative path from project root
            let rel_path = f.source_file.strip_prefix(ffi_dir.parent().unwrap()).unwrap();
            rel_path.to_string_lossy().to_string()
        })
        .collect::<std::collections::HashSet<_>>() // Remove duplicates
        .into_iter()
        .collect();

    let jni_code = generate_jni_bridge(&functions, &kotlin_package, &source_files);

    let jni_file = jni_dir.join("math_bridge.cpp");
    fs::write(&jni_file, jni_code)
        .context(format!("Failed to write JNI bridge: {}", jni_file.display()))?;

    // 4. Generate CMakeLists.txt
    let cmake_dir = build_dir.join("cmake");
    fs::create_dir_all(&cmake_dir)
        .context("Failed to create CMake output directory")?;

    let cmake_code = generate_cmake(
        "math", // TODO: Make this configurable
        &source_files,
        jni_file.to_string_lossy().as_ref(),
        &config.ffi.cpp.standard,
        &config.ffi.cpp.flags,
        &config.ffi.cpp.libraries,
    );

    let cmake_file = cmake_dir.join("CMakeLists.txt");
    fs::write(&cmake_file, cmake_code)
        .context(format!("Failed to write CMakeLists.txt: {}", cmake_file.display()))?;

    // 5. TODO: Run CMake build (Phase 1.6)
    // This would require:
    // - Finding Android NDK
    // - Running cmake with proper toolchain file
    // - Running make/ninja
    // - Copying .so files to correct location

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AndroidConfig, BuildConfig, FfiConfig, ProjectConfig, ToolchainConfig};

    fn create_test_config(ffi_enabled: Option<bool>) -> Config {
        Config {
            project: ProjectConfig {
                name: "test".to_string(),
                version: "0.1.0".to_string(),
            },
            android: AndroidConfig {
                min_sdk: 24,
                target_sdk: 34,
                package: "com.example.test".to_string(),
            },
            build: BuildConfig {
                output_dir: "build".to_string(),
                optimize_level: "default".to_string(),
            },
            toolchain: ToolchainConfig::default(),
            ffi: FfiConfig {
                enabled: ffi_enabled,
                ..Default::default()
            },
        }
    }

    #[test]
    fn test_should_build_ffi_explicit_enabled() {
        let config = create_test_config(Some(true));
        let temp_dir = tempfile::tempdir().unwrap();
        assert!(should_build_ffi(&config, temp_dir.path()));
    }

    #[test]
    fn test_should_build_ffi_explicit_disabled() {
        let config = create_test_config(Some(false));
        let temp_dir = tempfile::tempdir().unwrap();
        // Even if directory exists, should not build if explicitly disabled
        fs::create_dir_all(temp_dir.path().join("src/ffi/cpp")).unwrap();
        assert!(!should_build_ffi(&config, temp_dir.path()));
    }

    #[test]
    fn test_should_build_ffi_auto_detect_cpp() {
        let config = create_test_config(None);
        let temp_dir = tempfile::tempdir().unwrap();

        // Without directory, should not build
        assert!(!should_build_ffi(&config, temp_dir.path()));

        // With cpp directory, should build
        fs::create_dir_all(temp_dir.path().join("src/ffi/cpp")).unwrap();
        assert!(should_build_ffi(&config, temp_dir.path()));
    }

    #[test]
    fn test_should_build_ffi_auto_detect_rust() {
        let config = create_test_config(None);
        let temp_dir = tempfile::tempdir().unwrap();

        // With rust directory, should build
        fs::create_dir_all(temp_dir.path().join("src/ffi/rust")).unwrap();
        assert!(should_build_ffi(&config, temp_dir.path()));
    }
}
