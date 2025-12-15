use anyhow::Result;
use colored::Colorize;
use std::env;
use std::fs;
use std::path::Path;

use crate::commands::{detect_target, Target};
use crate::config;

/// Clean build artifacts
pub fn execute(target: &str) -> Result<()> {
    match detect_target(target) {
        Target::Project(manifest_path) => execute_project(&manifest_path),
        Target::SingleFile(_) => {
            println!("{} Single file mode has no build artifacts to clean", "info:".blue().bold());
            Ok(())
        }
    }
}

/// Clean a project's build artifacts
fn execute_project(manifest_path: &str) -> Result<()> {
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

    // Load configuration to get output directory
    let manifest_file = manifest_path.file_name().unwrap().to_str().unwrap();
    let config = config::load_config(manifest_file)?;

    let output_dir = Path::new(&config.build.output_dir);
    let mut cleaned = false;

    // Remove build output directory
    if output_dir.exists() {
        fs::remove_dir_all(output_dir)?;
        println!("  {} {}", "Removed".green(), output_dir.display());
        cleaned = true;
    }

    // Restore original directory if we changed it
    if project_dir != original_dir {
        env::set_current_dir(&original_dir)?;
    }

    if cleaned {
        println!("   {} build artifacts", "Cleaned".green().bold());
    } else {
        println!("{} No build artifacts to clean", "info:".blue().bold());
    }

    Ok(())
}
