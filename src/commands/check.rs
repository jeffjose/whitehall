use anyhow::{Context, Result};
use colored::Colorize;
use std::env;
use std::fs;
use std::path::Path;
use std::time::Instant;

use crate::commands::{detect_target, Target};
use crate::config::{self, Config};
use crate::transpiler;

/// Check syntax of .wh files without building
pub fn execute(target: &str) -> Result<()> {
    match detect_target(target) {
        Target::Project(manifest_path) => execute_project(&manifest_path),
        Target::SingleFile(file_path) => execute_single_file(&file_path),
    }
}

/// Check all .wh files in a project
fn execute_project(manifest_path: &str) -> Result<()> {
    let start = Instant::now();

    let manifest_path = Path::new(manifest_path);
    let original_dir = env::current_dir()?;

    let project_dir = if manifest_path == Path::new("whitehall.toml") {
        original_dir.clone()
    } else {
        let dir = manifest_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

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

    // Load configuration
    let manifest_file = manifest_path.file_name().unwrap().to_str().unwrap();
    let config = config::load_config(manifest_file)?;

    // Find all .wh files
    let files = find_wh_files(&config)?;

    if files.is_empty() {
        println!("{} No .wh files found in src/", "warning:".yellow().bold());
        return Ok(());
    }

    let mut errors = Vec::new();
    let mut checked = 0;

    for file_path in &files {
        match check_file(file_path, &config) {
            Ok(()) => {
                checked += 1;
            }
            Err(e) => {
                errors.push((file_path.clone(), e.to_string()));
            }
        }
    }

    // Restore original directory if we changed it
    if project_dir != original_dir {
        env::set_current_dir(&original_dir)?;
    }

    let elapsed = start.elapsed();

    if !errors.is_empty() {
        eprintln!("{} {} error(s) found:\n", "error:".red().bold(), errors.len());
        for (file, error) in &errors {
            eprintln!("  {} {}", file.display().to_string().yellow(), error);
        }
        anyhow::bail!("Check failed with {} error(s)", errors.len());
    }

    println!("   {} {} file(s) in {:.2}s",
        "Checked".green().bold(),
        checked,
        elapsed.as_secs_f64()
    );

    Ok(())
}

/// Check a single .wh file
fn execute_single_file(file_path: &str) -> Result<()> {
    let start = Instant::now();

    let path = Path::new(file_path);
    if !path.exists() {
        anyhow::bail!("File not found: {}", file_path);
    }
    if !file_path.ends_with(".wh") {
        anyhow::bail!("File must have .wh extension: {}", file_path);
    }

    let source = fs::read_to_string(path)
        .context(format!("Failed to read {}", file_path))?;

    let code = strip_frontmatter(&source);

    let component_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .unwrap_or_else(|| "Component".to_string());

    // Try to transpile - this validates syntax
    transpiler::transpile(&code, "com.example.app", &component_name, None)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let elapsed = start.elapsed();
    println!("   {} {} in {:.2}s",
        "Checked".green().bold(),
        file_path,
        elapsed.as_secs_f64()
    );

    Ok(())
}

/// Find all .wh files in the project's src directory
fn find_wh_files(_config: &Config) -> Result<Vec<std::path::PathBuf>> {
    let src_dir = Path::new("src");
    if !src_dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    collect_wh_files(src_dir, &mut files)?;

    // Sort for consistent output
    files.sort();

    Ok(files)
}

fn collect_wh_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_wh_files(&path, files)?;
        } else if path.extension().map_or(false, |ext| ext == "wh") {
            files.push(path);
        }
    }
    Ok(())
}

/// Check a single file for syntax errors
fn check_file(file_path: &Path, config: &Config) -> Result<()> {
    let source = fs::read_to_string(file_path)
        .context(format!("Failed to read {}", file_path.display()))?;

    let code = strip_frontmatter(&source);

    let component_name = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .unwrap_or_else(|| "Component".to_string());

    // Try to transpile - this validates syntax
    transpiler::transpile(&code, &config.android.package, &component_name, None)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(())
}

/// Strip frontmatter (/// comments) from source code
fn strip_frontmatter(content: &str) -> String {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with("#!") && !trimmed.starts_with("///")
        })
        .collect::<Vec<_>>()
        .join("\n")
}
