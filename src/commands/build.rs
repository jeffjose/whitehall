use anyhow::{Context, Result};
use std::env;
use std::path::Path;

use crate::build_pipeline;
use crate::config;

pub fn execute(manifest_path: &str) -> Result<()> {
    println!("🔨 Building Whitehall project...\n");

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
        env::set_current_dir(&project_dir)
            .context(format!("Failed to change to directory: {}", project_dir.display()))?;
    }

    // 2. Load configuration
    let manifest_file = manifest_path.file_name().unwrap().to_str().unwrap();
    let config = config::load_config(manifest_file)
        .context(format!("Failed to load {}. Are you in a Whitehall project directory?", manifest_file))?;

    // 3. Run build pipeline (with clean)
    let result = build_pipeline::execute_build(&config, true)?;

    // 4. Restore original directory if we changed it
    if project_dir != original_dir {
        env::set_current_dir(&original_dir)?;
    }

    // 5. Report results
    if !result.errors.is_empty() {
        eprintln!("❌ Build failed with {} error(s):\n", result.errors.len());
        for error in &result.errors {
            eprintln!("  {} - {}", error.file.display(), error.message);
        }
        anyhow::bail!("Build failed");
    }

    println!("✅ Build complete!");
    println!("   Transpiled {} file(s)", result.files_transpiled);
    println!("\n📦 Next steps:");

    // Make the output path relative to where the user ran the command
    let output_path = project_dir.join(&result.output_dir);
    let display_path = if output_path.starts_with(&original_dir) {
        output_path.strip_prefix(&original_dir).unwrap().to_path_buf()
    } else {
        output_path
    };

    println!("   cd {}", display_path.display());
    println!("   gradle wrapper  # Generate Gradle wrapper (first time only)");
    println!("   ./gradlew assembleDebug");
    println!("\n   APK will be in: app/build/outputs/apk/debug/");

    Ok(())
}
