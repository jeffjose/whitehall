use anyhow::{Context, Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{AndroidConfig, BuildConfig, Config, ProjectConfig, ToolchainConfig};

/// Configuration extracted from single-file frontmatter
#[derive(Debug, Deserialize)]
pub struct SingleFileConfig {
    pub app: AppSection,
}

#[derive(Debug, Deserialize)]
pub struct AppSection {
    pub name: String,
    #[serde(default = "default_package")]
    pub package: String,
    #[serde(default = "default_min_sdk")]
    pub min_sdk: u32,
    #[serde(default = "default_target_sdk")]
    pub target_sdk: u32,
}

fn default_package() -> String {
    // Will be generated from app name if not provided
    String::new()
}

fn default_min_sdk() -> u32 {
    24
}

fn default_target_sdk() -> u32 {
    34
}

/// Parse frontmatter from a single .wh file
/// Returns (config, code_without_frontmatter)
pub fn parse_frontmatter(content: &str) -> Result<(SingleFileConfig, String)> {
    let mut frontmatter_lines = Vec::new();
    let mut code_lines = Vec::new();
    let mut in_frontmatter = false;
    let mut found_frontmatter = false;

    for line in content.lines() {
        // Skip shebang if present
        if line.starts_with("#!") {
            continue;
        }

        if line.trim().starts_with("///") {
            // Extract frontmatter (remove /// prefix)
            let toml_line = line.trim().trim_start_matches("///").trim();
            if !toml_line.is_empty() || in_frontmatter {
                frontmatter_lines.push(toml_line.to_string());
                in_frontmatter = true;
                found_frontmatter = true;
            }
        } else if in_frontmatter && line.trim().is_empty() {
            // Continue frontmatter through empty lines
            continue;
        } else {
            // We've left the frontmatter section
            in_frontmatter = false;
            code_lines.push(line);
        }
    }

    if !found_frontmatter {
        anyhow::bail!(
            "No frontmatter found. Single-file mode requires frontmatter.\n\
            \n\
            Example:\n\
            /// [app]\n\
            /// name = \"MyApp\"\n\
            /// package = \"com.example.myapp\"\n\
            \n\
            <your code here>"
        );
    }

    // Parse TOML frontmatter
    let frontmatter_toml = frontmatter_lines.join("\n");
    let mut config: SingleFileConfig = toml::from_str(&frontmatter_toml)
        .context("Failed to parse frontmatter TOML")?;

    // Generate package name from app name if not provided
    if config.app.package.is_empty() {
        config.app.package = generate_package_name(&config.app.name);
    }

    // Validate package name
    validate_package_name(&config.app.package)?;

    let code = code_lines.join("\n");

    Ok((config, code))
}

/// Generate a package name from app name
/// "My Counter" -> "com.example.my_counter"
fn generate_package_name(app_name: &str) -> String {
    let sanitized = app_name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>();

    format!("com.example.{}", sanitized)
}

/// Validate Android package name (same logic as config.rs)
fn validate_package_name(package: &str) -> Result<()> {
    let parts: Vec<&str> = package.split('.').collect();
    if parts.len() < 2 {
        anyhow::bail!(
            "Invalid Android package name '{}'. Must have at least two parts (e.g., 'com.example')",
            package
        );
    }

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

/// Convert SingleFileConfig to full Config struct
pub fn to_config(single_file_config: &SingleFileConfig, output_dir: &str) -> Config {
    Config {
        project: ProjectConfig {
            name: single_file_config.app.name.clone(),
            version: "0.1.0".to_string(),
        },
        android: AndroidConfig {
            min_sdk: single_file_config.app.min_sdk,
            target_sdk: single_file_config.app.target_sdk,
            package: single_file_config.app.package.clone(),
        },
        build: BuildConfig {
            output_dir: output_dir.to_string(),
            optimize_level: "default".to_string(),
        },
        toolchain: ToolchainConfig::default(),
    }
}

/// Generate SHA256 hash of content for cache key
pub fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Get the cache directory for a given content hash
pub fn get_cache_dir(content_hash: &str) -> Result<PathBuf> {
    let cache_dir = if let Ok(custom_cache) = std::env::var("WHITEHALL_CACHE_DIR") {
        PathBuf::from(custom_cache)
    } else {
        dirs::cache_dir()
            .context("Failed to determine system cache directory")?
            .join("whitehall")
    };

    Ok(cache_dir.join(&content_hash[..16])) // Use first 16 chars of hash
}

/// Generate temporary project structure for single-file mode
/// Returns the path to the generated project directory
pub fn generate_temp_project(
    file_path: &Path,
    config: &SingleFileConfig,
    code: &str,
) -> Result<PathBuf> {
    // Read original file content to compute hash (including frontmatter)
    let original_content = fs::read_to_string(file_path)
        .context("Failed to read single-file")?;
    let content_hash = hash_content(&original_content);

    // Get cache directory
    let cache_dir = get_cache_dir(&content_hash)?;

    // Check if cache already exists
    let whitehall_toml_path = cache_dir.join("whitehall.toml");
    if whitehall_toml_path.exists() {
        return Ok(cache_dir);
    }

    // Create cache directory
    fs::create_dir_all(&cache_dir)
        .context("Failed to create cache directory")?;

    // Generate whitehall.toml
    let toml_content = format!(
        r#"[project]
name = "{}"
version = "0.1.0"

[android]
min_sdk = {}
target_sdk = {}
package = "{}"

[build]
output_dir = "build"
"#,
        config.app.name, config.app.min_sdk, config.app.target_sdk, config.app.package
    );

    fs::write(&whitehall_toml_path, toml_content)
        .context("Failed to write whitehall.toml")?;

    // Create src/ directory and write main.wh
    let src_dir = cache_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    let main_wh_path = src_dir.join("main.wh");
    fs::write(&main_wh_path, code)
        .context("Failed to write src/main.wh")?;

    Ok(cache_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter_valid() {
        let content = r#"#!/usr/bin/env whitehall
/// [app]
/// name = "Counter"
/// package = "com.example.counter"

var count = 0
<Text>{count}</Text>
"#;

        let (config, code) = parse_frontmatter(content).unwrap();
        assert_eq!(config.app.name, "Counter");
        assert_eq!(config.app.package, "com.example.counter");
        assert_eq!(config.app.min_sdk, 24); // default
        assert_eq!(config.app.target_sdk, 34); // default
        assert!(code.contains("var count = 0"));
    }

    #[test]
    fn test_parse_frontmatter_with_sdk_versions() {
        let content = r#"/// [app]
/// name = "MyApp"
/// package = "com.test.app"
/// min_sdk = 26
/// target_sdk = 33

<Text>Hello</Text>
"#;

        let (config, _) = parse_frontmatter(content).unwrap();
        assert_eq!(config.app.min_sdk, 26);
        assert_eq!(config.app.target_sdk, 33);
    }

    #[test]
    fn test_parse_frontmatter_missing() {
        let content = "var x = 5\n<Text>Hello</Text>";
        assert!(parse_frontmatter(content).is_err());
    }

    #[test]
    fn test_generate_package_name() {
        assert_eq!(generate_package_name("Counter"), "com.example.counter");
        assert_eq!(generate_package_name("My App"), "com.example.my_app");
        assert_eq!(generate_package_name("Todo List!"), "com.example.todo_list_");
    }

    #[test]
    fn test_hash_content() {
        let content1 = "hello world";
        let content2 = "hello world";
        let content3 = "different";

        assert_eq!(hash_content(content1), hash_content(content2));
        assert_ne!(hash_content(content1), hash_content(content3));
    }

    #[test]
    fn test_validate_package_name() {
        assert!(validate_package_name("com.example.app").is_ok());
        assert!(validate_package_name("com").is_err()); // too short
        assert!(validate_package_name("Com.example").is_err()); // uppercase
    }
}
