use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::project::{discover_files, FileType, WhitehallFile};
use crate::transpiler;

/// Represents the result of a build operation
#[derive(Debug)]
pub struct BuildResult {
    pub files_transpiled: usize,
    pub output_dir: PathBuf,
    pub errors: Vec<BuildError>,
}

#[derive(Debug)]
pub struct BuildError {
    pub file: PathBuf,
    pub message: String,
}

/// Core build pipeline - used by build, watch, and run commands
///
/// # Arguments
/// * `config` - Parsed whitehall.toml configuration
/// * `clean` - If true, remove and recreate output directory
///
/// # Returns
/// BuildResult with success count and any errors
pub fn execute_build(config: &Config, clean: bool) -> Result<BuildResult> {
    let output_dir = Path::new(&config.build.output_dir);

    // 1. Clean output directory if requested
    if clean && output_dir.exists() {
        fs::remove_dir_all(output_dir)
            .context("Failed to clean output directory")?;
    }
    fs::create_dir_all(output_dir)
        .context("Failed to create output directory")?;

    // 2. Discover .wh files
    let files = discover_files(config)
        .context("Failed to discover source files")?;

    // 3. Transpile each file
    let mut errors = Vec::new();
    let mut success_count = 0;

    for file in &files {
        match transpile_file(file, config, output_dir) {
            Ok(_) => success_count += 1,
            Err(e) => errors.push(BuildError {
                file: file.path.clone(),
                message: e.to_string(),
            }),
        }
    }

    // 4. Generate MainActivity if all files transpiled successfully
    if errors.is_empty() {
        generate_main_activity(config, output_dir, &files)?;
    }

    Ok(BuildResult {
        files_transpiled: success_count,
        output_dir: output_dir.to_path_buf(),
        errors,
    })
}

/// Transpile a single .wh file to Kotlin
fn transpile_file(
    file: &WhitehallFile,
    _config: &Config,
    output_dir: &Path,
) -> Result<()> {
    // Read source file
    let source = fs::read_to_string(&file.path)
        .context(format!("Failed to read {}", file.path.display()))?;

    // Determine component type for transpiler
    let component_type = match file.file_type {
        FileType::Screen => Some("screen"),
        _ => None,
    };

    // Transpile to Kotlin
    let kotlin_code = transpiler::transpile(
        &source,
        &file.package_path,
        &file.component_name,
        component_type,
    )
    .map_err(|e| anyhow::anyhow!("Transpilation error: {}", e))?;

    // Write output
    let output_path = get_kotlin_output_path(output_dir, file);
    fs::create_dir_all(output_path.parent().unwrap())
        .context("Failed to create output directories")?;
    fs::write(&output_path, kotlin_code)
        .context(format!("Failed to write {}", output_path.display()))?;

    Ok(())
}

/// Get the output path for a transpiled Kotlin file
fn get_kotlin_output_path(output_dir: &Path, file: &WhitehallFile) -> PathBuf {
    let package_path = file.package_path.replace('.', "/");
    output_dir
        .join("app/src/main/kotlin")
        .join(package_path)
        .join(format!("{}.kt", file.component_name))
}

/// Generate MainActivity.kt
fn generate_main_activity(
    config: &Config,
    output_dir: &Path,
    files: &[WhitehallFile],
) -> Result<()> {
    // Check if there's a main.wh file
    let main_file = files.iter().find(|f| f.file_type == FileType::Main);

    let main_content = if let Some(main_file) = main_file {
        // Use transpiled main.wh content as the App composable
        let source = fs::read_to_string(&main_file.path)?;
        transpiler::transpile(&source, &config.android.package, "App", None)
            .map_err(|e| anyhow::anyhow!(e))?
    } else {
        // Generate default MainActivity with basic content
        generate_default_main_activity(config)
    };

    // If we transpiled main.wh, we need to wrap it in MainActivity
    let activity_content = if main_file.is_some() {
        format!(
            r#"package {}

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.material3.MaterialTheme

class MainActivity : ComponentActivity() {{
    override fun onCreate(savedInstanceState: Bundle?) {{
        super.onCreate(savedInstanceState)
        setContent {{
            MaterialTheme {{
                App()
            }}
        }}
    }}
}}

{}"#,
            config.android.package, main_content
        )
    } else {
        main_content
    };

    // Write MainActivity.kt
    let package_path = config.android.package.replace('.', "/");
    let output_path = output_dir
        .join("app/src/main/kotlin")
        .join(package_path)
        .join("MainActivity.kt");

    fs::create_dir_all(output_path.parent().unwrap())?;
    fs::write(output_path, activity_content)?;

    Ok(())
}

/// Generate a default MainActivity with basic "Hello, Whitehall!" content
fn generate_default_main_activity(config: &Config) -> String {
    format!(
        r#"package {}

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier

class MainActivity : ComponentActivity() {{
    override fun onCreate(savedInstanceState: Bundle?) {{
        super.onCreate(savedInstanceState)
        setContent {{
            MaterialTheme {{
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {{
                    Text("Hello, Whitehall!")
                }}
            }}
        }}
    }}
}}
"#,
        config.android.package
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_kotlin_output_path() {
        let file = WhitehallFile {
            path: PathBuf::from("src/components/Button.wh"),
            file_type: FileType::Component,
            component_name: "Button".to_string(),
            package_path: "com.example.app.components".to_string(),
        };

        let output_dir = Path::new("build");
        let result = get_kotlin_output_path(output_dir, &file);

        assert_eq!(
            result,
            PathBuf::from("build/app/src/main/kotlin/com/example/app/components/Button.kt")
        );
    }

    #[test]
    fn test_default_main_activity_generation() {
        use crate::config::{AndroidConfig, BuildConfig, Config, ProjectConfig};

        let config = Config {
            project: ProjectConfig {
                name: "test".to_string(),
                version: "0.1.0".to_string(),
            },
            android: AndroidConfig {
                min_sdk: 24,
                target_sdk: 34,
                package: "com.example.test".to_string(),
            },
            build: BuildConfig::default(),
        };

        let content = generate_default_main_activity(&config);

        assert!(content.contains("package com.example.test"));
        assert!(content.contains("class MainActivity"));
        assert!(content.contains("Hello, Whitehall!"));
    }
}
