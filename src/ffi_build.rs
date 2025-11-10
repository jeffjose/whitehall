use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::config::Config;
use crate::ffi_parser::cpp::discover_cpp_ffi;
use crate::ffi_parser::rust::discover_rust_ffi;
use crate::ffi_codegen::{
    generate_kotlin_object, generate_jni_bridge, generate_cmake,
    generate_kotlin_object_rust, generate_rust_bridge,
};
use crate::toolchain::Toolchain;

/// Build FFI components if enabled
pub fn build_ffi(config: &Config, project_root: &Path) -> Result<()> {
    // Check if FFI is enabled
    if !should_build_ffi(config, project_root) {
        return Ok(());
    }

    let ffi_dir = project_root.join("src/ffi");
    let build_dir = project_root.join(&config.build.output_dir);

    // Build C++ FFI if present
    if ffi_dir.join("cpp").exists() {
        build_cpp_ffi(config, &ffi_dir, &build_dir)?;
    }

    // Build Rust FFI if present
    if ffi_dir.join("rust").exists() {
        build_rust_ffi(config, &ffi_dir, &build_dir)?;
    }

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

/// Convert kebab-case or snake_case to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split(|c| c == '-' || c == '_')
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

/// Parse project name from CMakeLists.txt
/// Looks for: project("name") or project(name)
fn parse_cmake_project_name(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)
        .context(format!("Failed to read CMakeLists.txt: {}", path.display()))?;

    // Match: project("name") or project(name) or project( "name" )
    let re = regex::Regex::new(r#"project\s*\(\s*["']?([^"')\s]+)["']?\s*\)"#)
        .context("Failed to compile regex")?;

    if let Some(cap) = re.captures(&content) {
        return Ok(cap[1].to_string());
    }

    anyhow::bail!("No project() declaration found in CMakeLists.txt")
}

/// Parse package name from Cargo.toml
fn parse_cargo_toml_name(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)
        .context(format!("Failed to read Cargo.toml: {}", path.display()))?;

    let cargo_toml: toml::Value = toml::from_str(&content)
        .context("Failed to parse Cargo.toml")?;

    cargo_toml
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("No package.name found in Cargo.toml"))
}

/// Get C++ library name with precedence:
/// 1. whitehall.toml override
/// 2. src/ffi/cpp/CMakeLists.txt project name
/// 3. project name fallback
fn get_cpp_library_name(config: &Config, ffi_dir: &Path) -> String {
    // 1. Check whitehall.toml override
    if let Some(name) = &config.ffi.cpp.library_name {
        return name.clone();
    }

    // 2. Check user-provided CMakeLists.txt
    let user_cmake = ffi_dir.join("cpp/CMakeLists.txt");
    if user_cmake.exists() {
        if let Ok(name) = parse_cmake_project_name(&user_cmake) {
            return name;
        }
    }

    // 3. Fallback to project name
    config.project.name.clone()
}

/// Get Rust library name with precedence:
/// 1. whitehall.toml override
/// 2. src/ffi/rust/Cargo.toml package name
/// 3. project name fallback
fn get_rust_library_name(config: &Config, ffi_dir: &Path) -> String {
    // 1. Check whitehall.toml override
    if let Some(name) = &config.ffi.rust.library_name {
        return name.clone();
    }

    // 2. Check Cargo.toml
    let cargo_toml = ffi_dir.join("rust/Cargo.toml");
    if cargo_toml.exists() {
        if let Ok(name) = parse_cargo_toml_name(&cargo_toml) {
            return name;
        }
    }

    // 3. Fallback to project name
    config.project.name.clone()
}

/// Build C++ FFI components
fn build_cpp_ffi(config: &Config, ffi_dir: &Path, build_dir: &Path) -> Result<()> {
    use colored::Colorize;

    // 1. Discover FFI functions
    eprintln!("{}", "Discovering FFI functions...".dimmed());
    let functions = discover_cpp_ffi(ffi_dir)
        .context("Failed to discover C++ FFI functions")?;

    if functions.is_empty() {
        eprintln!("{}", "No FFI functions found".yellow());
        return Ok(());
    }

    eprintln!("{} {} FFI function(s)", "Found".green(), functions.len());

    // Get library name with precedence: whitehall.toml > CMakeLists.txt > project.name
    let library_name = get_cpp_library_name(config, ffi_dir);
    eprintln!("{} {}", "Library name:".dimmed(), library_name);

    // Generate PascalCase object name for Kotlin
    let object_name = to_pascal_case(&library_name);

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
        &library_name,
        &object_name,
    );

    let kotlin_file = kotlin_package_dir.join(format!("{}.kt", object_name));
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

    let jni_file = jni_dir.join(format!("{}_bridge.cpp", library_name));
    fs::write(&jni_file, jni_code)
        .context(format!("Failed to write JNI bridge: {}", jni_file.display()))?;

    // 4. CMakeLists.txt: Use user-provided or auto-generate
    let cmake_dir = build_dir.join("cmake");
    fs::create_dir_all(&cmake_dir)
        .context("Failed to create CMake output directory")?;

    let cmake_file = cmake_dir.join("CMakeLists.txt");
    let user_cmake = ffi_dir.join("cpp/CMakeLists.txt");

    if user_cmake.exists() {
        // User provided their own CMakeLists.txt - copy it
        fs::copy(&user_cmake, &cmake_file)
            .context(format!("Failed to copy user CMakeLists.txt from {}", user_cmake.display()))?;
    } else {
        // Auto-generate CMakeLists.txt with absolute paths
        let project_root_abs = ffi_dir.parent().unwrap().canonicalize()
            .context("Failed to canonicalize project root")?;

        // Convert relative source paths to absolute
        let absolute_source_files: Vec<String> = source_files
            .iter()
            .map(|rel_path| {
                project_root_abs.join(rel_path).display().to_string()
            })
            .collect();

        let cmake_code = generate_cmake(
            &library_name,
            &absolute_source_files,
            &jni_file.canonicalize()
                .context("Failed to canonicalize JNI bridge path")?
                .display()
                .to_string(),
            &config.ffi.cpp.standard,
            &config.ffi.cpp.flags,
            &config.ffi.cpp.libraries,
        );

        fs::write(&cmake_file, cmake_code)
            .context(format!("Failed to write CMakeLists.txt: {}", cmake_file.display()))?;
    }

    // 5. Build native library using CMake + NDK
    let project_root = ffi_dir.parent().unwrap();
    build_native_library(
        &library_name,
        &cmake_file,
        project_root,
        build_dir,
        config.android.min_sdk,
    )?;

    Ok(())
}

/// Build native library using CMake and Android NDK
///
/// Compiles C++ code to .so files for all Android ABIs
///
/// # Arguments
/// * `library_name` - Name of the native library (used for .so filename)
/// * `cmake_file` - Path to CMakeLists.txt
/// * `project_root` - Root directory of the project
/// * `build_dir` - Build output directory
/// * `min_sdk` - Minimum Android SDK version
fn build_native_library(
    library_name: &str,
    cmake_file: &Path,
    _project_root: &Path,
    build_dir: &Path,
    min_sdk: u32,
) -> Result<()> {
    println!("{}", format!("Building native library: {}", library_name).cyan());

    // Initialize toolchain
    let toolchain = Toolchain::new()
        .context("Failed to initialize toolchain")?;

    // Ensure NDK is installed
    let ndk_path = toolchain.ensure_ndk()
        .context("Failed to ensure Android NDK")?;

    // Android ABIs to build for (cover most devices)
    let abis = vec![
        "arm64-v8a",     // Modern 64-bit ARM (most common)
        "armeabi-v7a",   // Legacy 32-bit ARM
        "x86_64",        // 64-bit x86 (emulators)
        "x86",           // 32-bit x86 (old emulators)
    ];

    // Build for each ABI
    for abi in &abis {
        println!("  {} {}", "→".dimmed(), format!("Building for {}", abi).dimmed());

        // Create build directory for this ABI
        let abi_build_dir = build_dir.join("cmake-build").join(abi);
        fs::create_dir_all(&abi_build_dir)
            .context(format!("Failed to create build directory for {}", abi))?;

        // Run CMake configure
        let mut cmake_cmd = toolchain.cmake_cmd(&ndk_path, abi, min_sdk)?;

        // Set source directory (parent of CMakeLists.txt)
        let source_dir = cmake_file.parent().unwrap();

        cmake_cmd
            .arg("-S").arg(source_dir)
            .arg("-B").arg(&abi_build_dir)
            .arg("-DCMAKE_BUILD_TYPE=Release");

        let output = cmake_cmd.output()
            .context(format!("Failed to run CMake configure for {}", abi))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            eprintln!("{}", stdout);
            eprintln!("{}", stderr);
            anyhow::bail!("CMake configure failed for ABI: {}", abi);
        }

        // Run CMake build
        let build_output = Command::new(toolchain.ensure_cmake()?)
            .arg("--build")
            .arg(&abi_build_dir)
            .arg("--config").arg("Release")
            .output()
            .context(format!("Failed to build for {}", abi))?;

        if !build_output.status.success() {
            let stderr = String::from_utf8_lossy(&build_output.stderr);
            let stdout = String::from_utf8_lossy(&build_output.stdout);
            eprintln!("{}", stdout);
            eprintln!("{}", stderr);
            anyhow::bail!("CMake build failed for ABI: {}", abi);
        }

        // Copy .so file to jniLibs
        let so_file = abi_build_dir.join(format!("lib{}.so", library_name));
        if !so_file.exists() {
            anyhow::bail!(
                "Built library not found: {}. Expected at: {}",
                library_name,
                so_file.display()
            );
        }

        // Destination: build/app/src/main/jniLibs/{abi}/lib{name}.so
        let jnilib_dir = build_dir.join("app/src/main/jniLibs").join(abi);
        fs::create_dir_all(&jnilib_dir)
            .context(format!("Failed to create jniLibs directory for {}", abi))?;

        let dest_so = jnilib_dir.join(format!("lib{}.so", library_name));
        fs::copy(&so_file, &dest_so)
            .context(format!("Failed to copy .so file for {}", abi))?;

        println!("    {} {}", "✓".green(), format!("{} built successfully", abi).dimmed());
    }

    println!("{}", format!("Native library '{}' built successfully for {} ABIs", library_name, abis.len()).green());

    Ok(())
}

/// Build Rust FFI components
fn build_rust_ffi(config: &Config, ffi_dir: &Path, build_dir: &Path) -> Result<()> {
    // 1. Discover FFI functions
    let functions = discover_rust_ffi(ffi_dir)
        .context("Failed to discover Rust FFI functions")?;

    if functions.is_empty() {
        return Ok(());
    }

    // Get library name with precedence: whitehall.toml > Cargo.toml > project.name
    let library_name = get_rust_library_name(config, ffi_dir);

    // Generate PascalCase object name for Kotlin
    let object_name = to_pascal_case(&library_name);

    // 2. Generate Kotlin bindings in ffi.rust subpackage
    let kotlin_dir = build_dir.join("generated/kotlin");
    let package_path = config.android.package.replace('.', "/");
    let kotlin_package_dir = kotlin_dir.join(&package_path).join("ffi/rust");

    fs::create_dir_all(&kotlin_package_dir)
        .context("Failed to create Kotlin output directory")?;

    let kotlin_package = format!("{}.ffi.rust", config.android.package);
    let kotlin_code = generate_kotlin_object_rust(
        &functions,
        &kotlin_package,
        &library_name,
        &object_name,
    );

    let kotlin_file = kotlin_package_dir.join(format!("{}.kt", object_name));
    fs::write(&kotlin_file, kotlin_code)
        .context(format!("Failed to write Kotlin binding: {}", kotlin_file.display()))?;

    // 3. Generate Rust JNI bridge to build/generated/jni/rust/
    let jni_dir = build_dir.join("generated/jni/rust");
    fs::create_dir_all(&jni_dir)
        .context("Failed to create Rust JNI output directory")?;

    let rust_bridge_code = generate_rust_bridge(&functions, &kotlin_package);

    // Write generated bridge to build/ directory (not src/)
    let bridge_file = jni_dir.join("jni_bridge.rs");
    fs::write(&bridge_file, rust_bridge_code)
        .context(format!("Failed to write Rust JNI bridge: {}", bridge_file.display()))?;

    // 4. Generate local ffi_macro crate (instead of fragile path dependencies)
    let rust_project_dir = ffi_dir.join("rust");
    let ffi_macro_dir = rust_project_dir.join("ffi_macro");
    fs::create_dir_all(&ffi_macro_dir.join("src"))
        .context("Failed to create ffi_macro directory")?;

    // Generate ffi_macro/Cargo.toml
    let ffi_macro_cargo_toml = r#"[package]
name = "whitehall"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true

[dependencies]
"#;
    fs::write(ffi_macro_dir.join("Cargo.toml"), ffi_macro_cargo_toml)
        .context("Failed to write ffi_macro/Cargo.toml")?;

    // Generate ffi_macro/src/lib.rs
    let ffi_macro_lib = r#"//! Whitehall FFI Macro (auto-generated)
//!
//! Provides the #[ffi] attribute for marking functions to be exposed via JNI.
//! This is a marker attribute that is detected by the Whitehall build system
//! during code generation. The attribute itself does nothing at runtime.

extern crate proc_macro;
use proc_macro::TokenStream;

/// Marker attribute for functions to be exposed via FFI/JNI
///
/// # Example
/// ```
/// #[ffi]
/// pub fn add(a: i32, b: i32) -> i32 {
///     a + b
/// }
/// ```
#[proc_macro_attribute]
pub fn ffi(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Pass through unchanged - this is just a marker for whitehall's parser
    item
}
"#;
    fs::write(ffi_macro_dir.join("src/lib.rs"), ffi_macro_lib)
        .context("Failed to write ffi_macro/src/lib.rs")?;

    // 5. Update main Cargo.toml to use local ffi_macro
    let cargo_toml_path = rust_project_dir.join("Cargo.toml");
    if cargo_toml_path.exists() {
        let cargo_content = fs::read_to_string(&cargo_toml_path)?;

        // Replace any existing whitehall or whitehall-ffi-macro dependency with local path
        let updated_cargo = if cargo_content.contains("whitehall-ffi-macro") || cargo_content.contains(r#"whitehall = "#) {
            // Replace existing dependency line (handle both old and new names)
            let re1 = regex::Regex::new(r#"whitehall-ffi-macro\s*=\s*\{[^}]+\}"#).unwrap();
            let re2 = regex::Regex::new(r#"whitehall\s*=\s*\{[^}]+\}"#).unwrap();

            let temp = re1.replace(&cargo_content, r#"whitehall = { path = "./ffi_macro" }"#).to_string();
            re2.replace(&temp, r#"whitehall = { path = "./ffi_macro" }"#).to_string()
        } else {
            // Add dependency if missing
            cargo_content.replace(
                "[dependencies]",
                "[dependencies]\nwhitehall = { path = \"./ffi_macro\" }"
            )
        };

        fs::write(&cargo_toml_path, updated_cargo)
            .context("Failed to update Cargo.toml with local ffi_macro dependency")?;
    }

    // 6. Generate build.rs to copy bridge to OUT_DIR during Rust build
    let build_rs_path = rust_project_dir.join("build.rs");

    // Get absolute path to generated bridge
    // ffi_dir is src/ffi, so we need to go up twice to get project root
    let project_root = ffi_dir.parent().unwrap().parent().unwrap();
    let bridge_abs = project_root
        .canonicalize()
        .context("Failed to get absolute project root path")?
        .join(&config.build.output_dir)
        .join("generated/jni/rust/jni_bridge.rs");

    let build_rs_content = format!(
        r#"// Auto-generated build script for Whitehall FFI
// This copies the generated JNI bridge from build/ to OUT_DIR

use std::{{env, fs, path::Path}};

fn main() {{
    let out_dir = env::var("OUT_DIR").unwrap();
    let bridge_src = Path::new(r"{}");
    let bridge_dst = Path::new(&out_dir).join("jni_bridge.rs");

    // Copy generated bridge to OUT_DIR
    fs::copy(bridge_src, bridge_dst)
        .expect("Failed to copy jni_bridge.rs to OUT_DIR");

    // Re-run if the bridge changes
    println!("cargo:rerun-if-changed={{}}", bridge_src.display());
}}
"#,
        bridge_abs.display()
    );

    fs::write(&build_rs_path, build_rs_content)
        .context(format!("Failed to write build.rs: {}", build_rs_path.display()))?;

    // 7. Create minimal stub in src/jni_bridge.rs that includes from OUT_DIR
    let rust_src_dir = ffi_dir.join("rust/src");
    if !rust_src_dir.exists() {
        anyhow::bail!("Rust src directory not found: {}", rust_src_dir.display());
    }

    let stub_file = rust_src_dir.join("jni_bridge.rs");
    let stub_content = r#"// Auto-generated stub: includes JNI bridge from build directory
// The actual generated code is in build/generated/jni/rust/jni_bridge.rs
// and gets copied to OUT_DIR by build.rs during compilation

include!(concat!(env!("OUT_DIR"), "/jni_bridge.rs"));
"#;

    fs::write(&stub_file, stub_content)
        .context(format!("Failed to write jni_bridge.rs stub: {}", stub_file.display()))?;

    // 8. Ensure lib.rs includes the bridge module
    let lib_rs = rust_src_dir.join("lib.rs");
    if lib_rs.exists() {
        let content = fs::read_to_string(&lib_rs)?;
        if !content.contains("mod jni_bridge;") {
            // Append module declaration
            let updated = format!("{}\n\n// JNI bridge module (generated by Whitehall FFI)\nmod jni_bridge;\n", content);
            fs::write(&lib_rs, updated)?;
        }
    }

    // 9. Build Rust library using cargo
    build_rust_library(
        &library_name,
        &ffi_dir.join("rust"),
        build_dir,
        config.android.min_sdk,
    )?;

    Ok(())
}

/// Build Rust library using cargo for Android targets
///
/// Compiles Rust code to .so files for all Android ABIs
///
/// # Arguments
/// * `library_name` - Name of the Rust library (from Cargo.toml)
/// * `rust_project_dir` - Path to Rust project directory (contains Cargo.toml)
/// * `build_dir` - Build output directory
/// * `min_sdk` - Minimum Android SDK version
fn build_rust_library(
    library_name: &str,
    rust_project_dir: &Path,
    build_dir: &Path,
    _min_sdk: u32,
) -> Result<()> {
    println!("{}", format!("Building Rust library: {}", library_name).cyan());

    // Initialize toolchain to ensure NDK
    let toolchain = Toolchain::new()
        .context("Failed to initialize toolchain")?;

    let ndk_path = toolchain.ensure_ndk()
        .context("Failed to ensure Android NDK")?;

    // Rust target triples for Android
    let targets = vec![
        ("aarch64-linux-android", "arm64-v8a"),
        ("armv7-linux-androideabi", "armeabi-v7a"),
        ("x86_64-linux-android", "x86_64"),
        ("i686-linux-android", "x86"),
    ];

    // Add Android targets if not already installed
    for (target, _) in &targets {
        println!("  {} {}", "→".dimmed(), format!("Adding target {}", target).dimmed());
        let status = Command::new("rustup")
            .args(&["target", "add", target])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .context(format!("Failed to add Rust target: {}", target))?;

        if !status.success() {
            println!("    {} {}", "⚠".yellow(), format!("Could not add target {}, skipping", target).yellow());
        }
    }

    // Build for each target
    for (target, abi) in &targets {
        println!("  {} {}", "→".dimmed(), format!("Building for {}", abi).dimmed());

        // Set up environment for Android NDK
        let mut cmd = Command::new("cargo");
        cmd.current_dir(rust_project_dir);
        cmd.arg("build");
        cmd.arg("--release");
        cmd.arg("--target").arg(target);

        // Set environment variables for NDK
        cmd.env("ANDROID_NDK_HOME", &ndk_path);
        cmd.env("ANDROID_NDK_ROOT", &ndk_path);

        // Set linker based on target
        let linker = match *target {
            "aarch64-linux-android" => "aarch64-linux-android21-clang",
            "armv7-linux-androideabi" => "armv7a-linux-androideabi21-clang",
            "x86_64-linux-android" => "x86_64-linux-android21-clang",
            "i686-linux-android" => "i686-linux-android21-clang",
            _ => continue,
        };

        let linker_path = ndk_path
            .join("toolchains/llvm/prebuilt")
            .join(if cfg!(target_os = "linux") { "linux-x86_64" } else { "darwin-x86_64" })
            .join("bin")
            .join(linker);

        if linker_path.exists() {
            cmd.env(
                format!("CARGO_TARGET_{}_LINKER", target.to_uppercase().replace('-', "_")),
                linker_path,
            );
        }

        // Capture output so we can show errors
        let output = cmd.output()
            .context(format!("Failed to execute cargo build for {}", target))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);

            eprintln!();
            eprintln!("{} Rust build failed for target: {}", "error:".red().bold(), target);
            eprintln!();

            if !stdout.is_empty() {
                eprintln!("stdout:");
                eprintln!("{}", stdout);
            }

            if !stderr.is_empty() {
                eprintln!("stderr:");
                eprintln!("{}", stderr);
            }

            anyhow::bail!("Rust build failed for target: {}", target);
        }

        // Copy .so file to jniLibs
        let so_name = format!("lib{}.so", library_name.replace('-', "_"));
        let so_file = rust_project_dir
            .join("target")
            .join(target)
            .join("release")
            .join(&so_name);

        if !so_file.exists() {
            anyhow::bail!(
                "Built Rust library not found: {}. Expected at: {}",
                library_name,
                so_file.display()
            );
        }

        // Destination: build/app/src/main/jniLibs/{abi}/lib{name}.so
        let jnilib_dir = build_dir.join("app/src/main/jniLibs").join(abi);
        fs::create_dir_all(&jnilib_dir)
            .context(format!("Failed to create jniLibs directory for {}", abi))?;

        let dest_so = jnilib_dir.join(&so_name);
        fs::copy(&so_file, &dest_so)
            .context(format!("Failed to copy Rust .so file for {}", abi))?;

        println!("    {} {}", "✓".green(), format!("{} built successfully", abi).dimmed());
    }

    println!("{}", format!("Rust library '{}' built successfully for {} ABIs", library_name, targets.len()).green());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AndroidConfig, BuildConfig, FfiConfig, ProjectConfig, ToolchainConfig, CppConfig};

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("hello-world"), "HelloWorld");
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("my-awesome-lib"), "MyAwesomeLib");
        assert_eq!(to_pascal_case("ffi-cpp-example"), "FfiCppExample");
        assert_eq!(to_pascal_case("math"), "Math");
        assert_eq!(to_pascal_case("single"), "Single");
    }

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
