use anyhow::{Context, Result};
use colored::Colorize;
use std::env;
use std::fs;
use std::path::Path;
use std::time::Instant;

use crate::build_pipeline;
use crate::config;
use crate::single_file;
use crate::commands::{detect_target, Target};
use crate::toolchain::Toolchain;

pub fn execute(target: &str) -> Result<()> {
    // Detect if we're building a project or single file
    match detect_target(target) {
        Target::Project(manifest_path) => execute_project(&manifest_path),
        Target::SingleFile(file_path) => execute_single_file(&file_path),
    }
}

/// Build a single .wh file
fn execute_single_file(file_path: &str) -> Result<()> {
    let start = Instant::now();

    // Parse frontmatter
    let file_path = Path::new(file_path);
    let content = fs::read_to_string(file_path)
        .map_err(|e| anyhow::anyhow!("failed to read `{}`: {}", file_path.display(), e))?;

    let (single_config, code) = single_file::parse_frontmatter(&content)?;

    // Generate temporary project
    let temp_project_dir = single_file::generate_temp_project(file_path, &single_config, &code)?;

    // Change to temp project directory
    let original_dir = env::current_dir()?;
    env::set_current_dir(&temp_project_dir)?;

    // Load config from generated whitehall.toml
    let config = config::load_config("whitehall.toml")?;

    // Run build pipeline
    let result = build_pipeline::execute_build(&config, true)?;

    // Report results
    if !result.errors.is_empty() {
        env::set_current_dir(&original_dir)?;
        eprintln!("{} build failed with {} error(s)", "error:".red().bold(), result.errors.len());
        for error in &result.errors {
            eprintln!("  {} - {}", error.file.display(), error.message);
        }
        anyhow::bail!("Build failed");
    }

    println!("   {} transpilation for `{}`",
        "Finished".green().bold(),
        single_config.app.name
    );

    // Build APK (skip if FFI-only mode)
    if !config.ffi.ffi_only {
        // Initialize toolchain and build APK
        let toolchain = Toolchain::new()?;
        toolchain.ensure_all_for_build(&config.toolchain.java, &config.toolchain.gradle)?;

        build_with_gradle(&toolchain, &config, &result.output_dir)?;

        let elapsed = start.elapsed();
        println!("   {} APK for `{}` v{} ({}) in {:.2}s",
            "Built".green().bold(),
            single_config.app.name,
            config.project.version,
            single_config.app.package,
            elapsed.as_secs_f64()
        );
    } else {
        let elapsed = start.elapsed();
        println!("   {} FFI code generation for `{}` in {:.2}s (FFI-only mode)",
            "Completed".green().bold(),
            single_config.app.name,
            elapsed.as_secs_f64()
        );
    }

    // Restore original directory
    env::set_current_dir(&original_dir)?;

    Ok(())
}

/// Build a project (existing behavior)
fn execute_project(manifest_path: &str) -> Result<()> {
    let start = Instant::now();

    // 1. Determine project directory from manifest path
    let manifest_path = Path::new(manifest_path);
    let original_dir = env::current_dir()?;

    // If manifest_path is just "whitehall.toml" (default), use cwd
    // Otherwise, change to the directory containing the manifest
    let project_dir = if manifest_path == Path::new("whitehall.toml") {
        original_dir.clone()
    } else {
        let dir = manifest_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        // Make it absolute
        if dir.is_relative() {
            original_dir.join(dir)
        } else {
            dir
        }
    };

    // Change to project directory if needed
    if project_dir != original_dir {
        env::set_current_dir(&project_dir)?;
    }

    // 2. Load configuration
    let manifest_file = manifest_path.file_name().unwrap().to_str().unwrap();
    let config = config::load_config(manifest_file)?;

    // 3. Run build pipeline (with clean)
    let result = build_pipeline::execute_build(&config, true)?;

    // 4. Report results
    if !result.errors.is_empty() {
        if project_dir != original_dir {
            env::set_current_dir(&original_dir)?;
        }
        eprintln!("{} build failed with {} error(s)", "error:".red().bold(), result.errors.len());
        for error in &result.errors {
            eprintln!("  {} - {}", error.file.display(), error.message);
        }
        anyhow::bail!("Build failed");
    }

    println!("   {} transpilation for `{}`",
        "Finished".green().bold(),
        config.project.name
    );

    // 5. Build APK (skip if FFI-only mode)
    if !config.ffi.ffi_only {
        // Initialize toolchain and build APK
        let toolchain = Toolchain::new()?;
        toolchain.ensure_all_for_build(&config.toolchain.java, &config.toolchain.gradle)?;

        build_with_gradle(&toolchain, &config, &result.output_dir)?;

        let elapsed = start.elapsed();
        println!("   {} APK for `{}` v{} ({}) in {:.2}s",
            "Built".green().bold(),
            config.project.name,
            config.project.version,
            config.android.package,
            elapsed.as_secs_f64()
        );
    } else {
        let elapsed = start.elapsed();
        println!("   {} FFI code generation for `{}` in {:.2}s (FFI-only mode)",
            "Completed".green().bold(),
            config.project.name,
            elapsed.as_secs_f64()
        );
    }

    // 6. Restore original directory if we changed it
    if project_dir != original_dir {
        env::set_current_dir(&original_dir)?;
    }

    Ok(())
}

fn build_with_gradle(toolchain: &Toolchain, config: &crate::config::Config, output_dir: &Path) -> Result<()> {
    let mut gradle = toolchain.gradle_cmd(&config.toolchain.java, &config.toolchain.gradle)?;

    let status = gradle
        .current_dir(output_dir)
        .args(&["assembleDebug", "--console=plain", "--quiet"])
        .status()
        .context("Failed to run Gradle")?;

    if !status.success() {
        anyhow::bail!("Gradle build failed");
    }

    Ok(())
}
