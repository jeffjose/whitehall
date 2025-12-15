use anyhow::{Context, Result};
use colored::Colorize;
use std::env;
use std::fs;
use std::path::Path;
use std::time::Instant;

use crate::build_pipeline;
use crate::config;
use crate::transpiler;
use crate::commands::{detect_target, Target};

/// Compile a .wh file or project to Kotlin (transpile only, no APK)
pub fn execute(target: &str, package: Option<&str>, no_package: bool) -> Result<()> {
    // Detect if we're compiling a project or single file
    match detect_target(target) {
        Target::Project(manifest_path) => execute_project(&manifest_path),
        Target::SingleFile(file_path) => execute_single_file(&file_path, package, no_package),
    }
}

/// Compile a project (transpile only)
fn execute_project(manifest_path: &str) -> Result<()> {
    let start = Instant::now();

    // 1. Determine project directory from manifest path
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

    // 2. Load configuration
    let manifest_file = manifest_path.file_name().unwrap().to_str().unwrap();
    let config = config::load_config(manifest_file)?;

    // 3. Run build pipeline (transpile only)
    let result = build_pipeline::execute_build(&config, true)?;

    // 4. Restore original directory if we changed it
    if project_dir != original_dir {
        env::set_current_dir(&original_dir)?;
    }

    // 5. Report results
    if !result.errors.is_empty() {
        eprintln!("{} compilation failed with {} error(s)", "error:".red().bold(), result.errors.len());
        for error in &result.errors {
            eprintln!("  {} - {}", error.file.display(), error.message);
        }
        anyhow::bail!("Compilation failed");
    }

    let elapsed = start.elapsed();
    println!("   {} `{}` v{} ({}) in {:.2}s",
        "Compiled".green().bold(),
        config.project.name,
        config.project.version,
        config.android.package,
        elapsed.as_secs_f64()
    );

    Ok(())
}

/// Compile a single .wh file to Kotlin (print to stdout)
fn execute_single_file(file_path: &str, package: Option<&str>, no_package: bool) -> Result<()> {
    // Validate file exists and has .wh extension
    let path = Path::new(file_path);
    if !path.exists() {
        anyhow::bail!("File not found: {}", file_path);
    }
    if !file_path.ends_with(".wh") {
        anyhow::bail!("File must have .wh extension: {}", file_path);
    }

    // Read source file
    let source = fs::read_to_string(path)
        .context(format!("Failed to read {}", file_path))?;

    // Strip frontmatter if present (for compile, we don't need it)
    let code = strip_frontmatter(&source);

    // Get component name from filename
    let component_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| {
            // Capitalize first letter for component name
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .unwrap_or_else(|| "Component".to_string());

    // Determine package name
    let package_name = package.unwrap_or("com.example.app");

    // Transpile to Kotlin
    let result = transpiler::transpile(&code, package_name, &component_name, None)
        .map_err(|e| anyhow::anyhow!("Transpilation error: {}", e))?;

    // Get all output files
    let files = result.files();

    // Output all Kotlin files
    for (i, (suffix, kotlin_code)) in files.iter().enumerate() {
        // Add separator between multiple files
        if i > 0 {
            println!("\n{}", "// ─────────────────────────────────────────────────────────────────".dimmed());
            let file_label = if suffix.is_empty() {
                format!("// {}.kt", component_name)
            } else {
                format!("// {}{}.kt", component_name, suffix)
            };
            println!("{}\n", file_label.dimmed());
        }

        if no_package {
            // Strip package declaration for pasting into existing files
            let code_without_package = kotlin_code
                .lines()
                .skip_while(|line| line.trim().is_empty() || line.starts_with("package "))
                .collect::<Vec<_>>()
                .join("\n");
            println!("{}", code_without_package);
        } else {
            println!("{}", kotlin_code);
        }
    }

    Ok(())
}

/// Strip frontmatter (/// comments) from source code
fn strip_frontmatter(content: &str) -> String {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            // Skip shebang and frontmatter lines
            !trimmed.starts_with("#!") && !trimmed.starts_with("///")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_frontmatter() {
        let source = r#"#!/usr/bin/env whitehall
/// [app]
/// name = "Test"

var x = 5
<Text>Hello</Text>
"#;

        let result = strip_frontmatter(source);
        assert!(!result.contains("#!/"));
        assert!(!result.contains("///"));
        assert!(result.contains("var x = 5"));
        assert!(result.contains("<Text>Hello</Text>"));
    }
}
