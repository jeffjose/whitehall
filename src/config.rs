use anyhow::Result;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub project: ProjectConfig,
    pub android: AndroidConfig,
    #[serde(default)]
    pub build: BuildConfig,
    #[serde(default)]
    pub toolchain: ToolchainConfig,
    #[serde(default)]
    pub ffi: FfiConfig,
}

#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct AndroidConfig {
    pub min_sdk: u32,
    pub target_sdk: u32,
    pub package: String,
}

#[derive(Debug, Deserialize)]
pub struct BuildConfig {
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
    #[serde(default = "default_optimize_level")]
    pub optimize_level: String,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            output_dir: default_output_dir(),
            optimize_level: default_optimize_level(),
        }
    }
}

fn default_output_dir() -> String {
    "build".to_string()
}

fn default_optimize_level() -> String {
    "default".to_string()
}

#[derive(Debug, Deserialize, Clone)]
pub struct ToolchainConfig {
    #[serde(default = "default_java")]
    pub java: String,
    #[serde(default = "default_gradle")]
    pub gradle: String,
    #[serde(default = "default_agp")]
    pub agp: String,
    #[serde(default = "default_kotlin")]
    pub kotlin: String,
}

impl Default for ToolchainConfig {
    fn default() -> Self {
        Self {
            java: default_java(),
            gradle: default_gradle(),
            agp: default_agp(),
            kotlin: default_kotlin(),
        }
    }
}

fn default_java() -> String {
    crate::toolchain::DEFAULT_JAVA.to_string()
}

fn default_gradle() -> String {
    crate::toolchain::DEFAULT_GRADLE.to_string()
}

fn default_agp() -> String {
    crate::toolchain::DEFAULT_AGP.to_string()
}

fn default_kotlin() -> String {
    crate::toolchain::DEFAULT_KOTLIN.to_string()
}

#[derive(Debug, Deserialize, Clone)]
pub struct FfiConfig {
    /// Enable or disable FFI. If None, auto-detect based on directory existence
    pub enabled: Option<bool>,
    #[serde(default)]
    pub cpp: CppConfig,
    #[serde(default)]
    pub rust: RustConfig,
}

impl Default for FfiConfig {
    fn default() -> Self {
        Self {
            enabled: None, // Auto-detect by default
            cpp: CppConfig::default(),
            rust: RustConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct CppConfig {
    #[serde(default = "default_cpp_standard")]
    pub standard: String,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub libraries: Vec<String>,
}

impl Default for CppConfig {
    fn default() -> Self {
        Self {
            standard: default_cpp_standard(),
            flags: Vec::new(),
            libraries: Vec::new(),
        }
    }
}

fn default_cpp_standard() -> String {
    "17".to_string()
}

#[derive(Debug, Deserialize, Clone)]
pub struct RustConfig {
    #[serde(default = "default_rust_profile")]
    pub profile: String,
    #[serde(default = "default_rust_targets")]
    pub targets: Vec<String>,
}

impl Default for RustConfig {
    fn default() -> Self {
        Self {
            profile: default_rust_profile(),
            targets: default_rust_targets(),
        }
    }
}

fn default_rust_profile() -> String {
    "release".to_string()
}

fn default_rust_targets() -> Vec<String> {
    vec![
        "aarch64-linux-android".to_string(),
        "armv7-linux-androideabi".to_string(),
        "x86_64-linux-android".to_string(),
        "i686-linux-android".to_string(),
    ]
}

/// Load and parse whitehall.toml configuration file
pub fn load_config(path: &str) -> Result<Config> {
    let content = fs::read_to_string(path)
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow::anyhow!("could not find `{}` in current directory", path)
            } else {
                anyhow::anyhow!("failed to read `{}`: {}", path, e)
            }
        })?;

    let config: Config = toml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("failed to parse `{}`: {}", path, e))?;

    // Validate Android package name
    validate_package_name(&config.android.package)?;

    // Validate toolchain compatibility
    let validator_config = crate::toolchain::validator::ToolchainConfig {
        java: config.toolchain.java.clone(),
        gradle: config.toolchain.gradle.clone(),
        agp: config.toolchain.agp.clone(),
    };
    crate::toolchain::validate_compatibility(&validator_config)?;

    Ok(config)
}

/// Validate Android package name format
fn validate_package_name(package: &str) -> Result<()> {
    // Must have at least two parts (e.g., com.example)
    let parts: Vec<&str> = package.split('.').collect();
    if parts.len() < 2 {
        anyhow::bail!(
            "Invalid Android package name '{}'. Must have at least two parts (e.g., 'com.example')",
            package
        );
    }

    // Each part must start with lowercase letter and contain only lowercase, digits, underscores
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            anyhow::bail!(
                "Invalid Android package name '{}'. Part {} is empty",
                package,
                i + 1
            );
        }

        let first_char = part.chars().next().unwrap();
        if !first_char.is_ascii_lowercase() {
            anyhow::bail!(
                "Invalid Android package name '{}'. Part '{}' must start with a lowercase letter",
                package,
                part
            );
        }

        for ch in part.chars() {
            if !ch.is_ascii_lowercase() && !ch.is_ascii_digit() && ch != '_' {
                anyhow::bail!(
                    "Invalid Android package name '{}'. Part '{}' contains invalid character '{}'",
                    package,
                    part,
                    ch
                );
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_package_name_valid() {
        assert!(validate_package_name("com.example.app").is_ok());
        assert!(validate_package_name("com.example.my_app").is_ok());
        assert!(validate_package_name("com.example.app123").is_ok());
    }

    #[test]
    fn test_validate_package_name_invalid() {
        // Too few parts
        assert!(validate_package_name("com").is_err());

        // Starts with uppercase
        assert!(validate_package_name("Com.example.app").is_err());
        assert!(validate_package_name("com.Example.app").is_err());

        // Contains invalid characters
        assert!(validate_package_name("com.example.my-app").is_err());
        assert!(validate_package_name("com.example.my app").is_err());

        // Empty part
        assert!(validate_package_name("com..app").is_err());
    }

    #[test]
    fn test_default_build_config() {
        let config = BuildConfig::default();
        assert_eq!(config.output_dir, "build");
        assert_eq!(config.optimize_level, "default");
    }
}
