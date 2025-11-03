use anyhow::{Context, Result};

use crate::build_pipeline;
use crate::config;

pub fn execute() -> Result<()> {
    println!("🔨 Building Whitehall project...\n");

    // 1. Load configuration
    let config = config::load_config("whitehall.toml")
        .context("Failed to load whitehall.toml. Are you in a Whitehall project directory?")?;

    // 2. Run build pipeline (with clean)
    let result = build_pipeline::execute_build(&config, true)?;

    // 3. Report results
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
    println!("   cd {}", result.output_dir.display());
    println!("   gradle wrapper  # Generate Gradle wrapper (first time only)");
    println!("   ./gradlew assembleDebug");
    println!("\n   APK will be in: app/build/outputs/apk/debug/");

    Ok(())
}
