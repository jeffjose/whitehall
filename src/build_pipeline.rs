use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::android_scaffold;
use crate::config::Config;
use crate::project::{discover_files, FileType, WhitehallFile};
use crate::routes;
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

    // 3. Generate Android scaffold (only if clean or missing)
    let scaffold_exists = output_dir.join("app/build.gradle.kts").exists();
    if clean || !scaffold_exists {
        android_scaffold::generate(config, output_dir)
            .context("Failed to generate Android project scaffold")?;
    }

    // 4. Transpile each file
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

    // 5. Generate Routes.kt from route structure
    if errors.is_empty() {
        generate_routes_file(config, output_dir)?;
    }

    // 6. Generate MainActivity if all files transpiled successfully
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
    // Skip main.wh - it's handled separately in MainActivity generation
    if file.file_type == FileType::Main {
        return Ok(());
    }

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
    // Discover routes to determine if we need NavHost setup
    let discovered_routes = routes::discover_routes()?;

    // Check if there's a main.wh file
    let main_file = files.iter().find(|f| f.file_type == FileType::Main);

    let main_content = if let Some(main_file) = main_file {
        // Use transpiled main.wh content as the App composable
        let source = fs::read_to_string(&main_file.path)?;
        transpiler::transpile(&source, &config.android.package, "App", None)
            .map_err(|e| anyhow::anyhow!(e))?
    } else if !discovered_routes.is_empty() {
        // Generate MainActivity with NavHost for routing
        generate_navhost_main_activity(config, &discovered_routes)
    } else {
        // Generate default MainActivity with basic content
        generate_default_main_activity(config)
    };

    // If we transpiled main.wh, we need to wrap it in MainActivity
    let activity_content = if main_file.is_some() {
        // Extract imports and code from transpiled main content
        let lines: Vec<&str> = main_content.lines().collect();
        let mut app_imports = Vec::new();
        let mut app_code_lines = Vec::new();
        let mut in_header = true;

        for line in lines {
            let trimmed = line.trim();
            if in_header && (trimmed.is_empty() || trimmed.starts_with("package ") || trimmed.starts_with("import ")) {
                if trimmed.starts_with("import ") {
                    app_imports.push(line.to_string());
                }
                // Skip package and empty lines in header
            } else {
                in_header = false;
                app_code_lines.push(line);
            }
        }

        let app_code = app_code_lines.join("\n");

        // Deduplicate app imports and remove MaterialTheme since it's in the template
        let mut unique_imports: Vec<String> = app_imports.into_iter()
            .filter(|imp| !imp.contains("androidx.compose.material3.MaterialTheme"))
            .collect();
        unique_imports.sort();
        unique_imports.dedup();

        let app_imports_str = if unique_imports.is_empty() {
            String::new()
        } else {
            format!("\n{}", unique_imports.join("\n"))
        };

        format!(
            r#"package {}

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.material3.MaterialTheme{}

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
            config.android.package, app_imports_str, app_code
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

/// Generate MainActivity with NavHost for routing
fn generate_navhost_main_activity(config: &Config, routes: &[routes::Route]) -> String {
    // Generate composable calls for each route
    let mut composable_entries = Vec::new();

    for route in routes {
        let screen_call = if route.params.is_empty() {
            // No parameters: ProfileScreen(navController)
            format!("{}(navController)", route.screen_name)
        } else {
            // With parameters: ProfileScreen(navController, it.id)
            let params_str = route
                .params
                .iter()
                .map(|p| format!("it.{}", p.name))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}(navController, {})", route.screen_name, params_str)
        };

        let composable = format!(
            "        composable<Routes.{}>{{ {} }}",
            route.name, screen_call
        );
        composable_entries.push(composable);
    }

    let composables = composable_entries.join("\n");

    // Find Home route for start destination (default to first route)
    let start_destination = routes
        .iter()
        .find(|r| r.name == "Home")
        .map(|r| format!("Routes.{}", r.name))
        .unwrap_or_else(|| format!("Routes.{}", routes[0].name));

    format!(
        r#"package {}

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.material3.MaterialTheme
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import {}.routes.Routes
import {}.screens.*

class MainActivity : ComponentActivity() {{
    override fun onCreate(savedInstanceState: Bundle?) {{
        super.onCreate(savedInstanceState)
        setContent {{
            MaterialTheme {{
                val navController = rememberNavController()
                NavHost(
                    navController = navController,
                    startDestination = {}
                ) {{
{}
                }}
            }}
        }}
    }}
}}
"#,
        config.android.package,
        config.android.package,
        config.android.package,
        start_destination,
        composables
    )
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

/// Generate Routes.kt file from route directory structure
fn generate_routes_file(config: &Config, output_dir: &Path) -> Result<()> {
    // Discover routes from src/routes/ directory
    let discovered_routes = routes::discover_routes()?;

    // If no routes found, skip generation
    if discovered_routes.is_empty() {
        return Ok(());
    }

    // Generate Routes.kt content
    let routes_content = routes::generate_routes_kt(&discovered_routes, &config.android.package);

    // Write to build/app/src/main/kotlin/{package}/routes/Routes.kt
    let package_path = config.android.package.replace('.', "/");
    let routes_dir = output_dir
        .join("app/src/main/kotlin")
        .join(package_path)
        .join("routes");

    fs::create_dir_all(&routes_dir)?;
    let routes_file = routes_dir.join("Routes.kt");
    fs::write(routes_file, routes_content)?;

    Ok(())
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
        use crate::config::{AndroidConfig, BuildConfig, Config, ProjectConfig, ToolchainConfig};

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
            toolchain: ToolchainConfig::default(),
        };

        let content = generate_default_main_activity(&config);

        assert!(content.contains("package com.example.test"));
        assert!(content.contains("class MainActivity"));
        assert!(content.contains("Hello, Whitehall!"));
    }
}
