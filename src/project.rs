use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::Config;

#[derive(Debug, Clone, PartialEq)]
pub struct WhitehallFile {
    pub path: PathBuf,           // src/components/Button.wh
    pub file_type: FileType,     // Component, Screen, or Main
    pub component_name: String,  // Button
    pub package_path: String,    // com.example.app.components
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    Component,
    Screen,
    Main,  // src/main.wh
}

/// Discover all .wh files in the src/ directory
pub fn discover_files(config: &Config) -> Result<Vec<WhitehallFile>> {
    let src_dir = Path::new("src");

    if !src_dir.exists() {
        anyhow::bail!(
            "Source directory 'src/' not found. Are you in a Whitehall project root?"
        );
    }

    let mut files = Vec::new();

    for entry in WalkDir::new(src_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Only process .wh files
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("wh") {
            let file = classify_file(path, config)
                .context(format!("Failed to classify file: {}", path.display()))?;
            files.push(file);
        }
    }

    if files.is_empty() {
        anyhow::bail!("No .wh files found in src/ directory");
    }

    Ok(files)
}

/// Classify a .wh file and determine its type and package
fn classify_file(path: &Path, config: &Config) -> Result<WhitehallFile> {
    // Get component name from filename (without extension)
    let component_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid filename: {}", path.display()))?
        .to_string();

    // Determine file type and package based on path
    let (file_type, package_suffix) = if path.ends_with("src/main.wh") {
        (FileType::Main, None)
    } else if is_under_directory(path, "src/components") {
        (FileType::Component, Some("components"))
    } else if is_under_directory(path, "src/screens") {
        (FileType::Screen, Some("screens"))
    } else {
        // Default: treat as component in base package
        (FileType::Component, None)
    };

    // Build full package path
    let package_path = if let Some(suffix) = package_suffix {
        format!("{}.{}", config.android.package, suffix)
    } else {
        config.android.package.clone()
    };

    Ok(WhitehallFile {
        path: path.to_path_buf(),
        file_type,
        component_name,
        package_path,
    })
}

/// Check if a path is under a specific directory
fn is_under_directory(path: &Path, dir: &str) -> bool {
    path.starts_with(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AndroidConfig, BuildConfig, ProjectConfig};

    fn make_test_config() -> Config {
        Config {
            project: ProjectConfig {
                name: "test-app".to_string(),
                version: "0.1.0".to_string(),
            },
            android: AndroidConfig {
                min_sdk: 24,
                target_sdk: 34,
                package: "com.example.testapp".to_string(),
            },
            build: BuildConfig::default(),
        }
    }

    #[test]
    fn test_classify_component() {
        let config = make_test_config();
        let path = Path::new("src/components/Button.wh");

        let file = classify_file(path, &config).unwrap();

        assert_eq!(file.component_name, "Button");
        assert_eq!(file.file_type, FileType::Component);
        assert_eq!(file.package_path, "com.example.testapp.components");
    }

    #[test]
    fn test_classify_screen() {
        let config = make_test_config();
        let path = Path::new("src/screens/HomeScreen.wh");

        let file = classify_file(path, &config).unwrap();

        assert_eq!(file.component_name, "HomeScreen");
        assert_eq!(file.file_type, FileType::Screen);
        assert_eq!(file.package_path, "com.example.testapp.screens");
    }

    #[test]
    fn test_classify_main() {
        let config = make_test_config();
        let path = Path::new("src/main.wh");

        let file = classify_file(path, &config).unwrap();

        assert_eq!(file.component_name, "main");
        assert_eq!(file.file_type, FileType::Main);
        assert_eq!(file.package_path, "com.example.testapp");
    }

    #[test]
    fn test_classify_root_level_component() {
        let config = make_test_config();
        let path = Path::new("src/App.wh");

        let file = classify_file(path, &config).unwrap();

        assert_eq!(file.component_name, "App");
        assert_eq!(file.file_type, FileType::Component);
        assert_eq!(file.package_path, "com.example.testapp");
    }

    #[test]
    fn test_nested_component() {
        let config = make_test_config();
        let path = Path::new("src/components/ui/Button.wh");

        let file = classify_file(path, &config).unwrap();

        assert_eq!(file.component_name, "Button");
        assert_eq!(file.file_type, FileType::Component);
        // Still maps to .components, not .components.ui
        assert_eq!(file.package_path, "com.example.testapp.components");
    }
}
