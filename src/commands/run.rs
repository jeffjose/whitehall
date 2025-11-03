use anyhow::{Context, Result};
use std::env;
use std::path::Path;
use std::process::Command;

use crate::build_pipeline;
use crate::config;

pub fn execute(manifest_path: &str) -> Result<()> {
    println!("🚀 Building and running Whitehall app...\n");

    // 1. Determine project directory from manifest path (same as build command)
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
        env::set_current_dir(&project_dir)
            .context(format!("Failed to change to directory: {}", project_dir.display()))?;
    }

    // 2. Load configuration
    let manifest_file = manifest_path.file_name().unwrap().to_str().unwrap();
    let config = config::load_config(manifest_file)
        .context(format!("Failed to load {}. Are you in a Whitehall project directory?", manifest_file))?;

    // 3. Build project
    println!("🔨 Step 1/4: Building...");
    let result = build_pipeline::execute_build(&config, true)?;

    if !result.errors.is_empty() {
        eprintln!("❌ Build failed with {} error(s):", result.errors.len());
        for error in &result.errors {
            eprintln!("  {} - {}", error.file.display(), error.message);
        }
        anyhow::bail!("Build failed");
    }

    println!("✅ Build complete\n");

    // 4. Check if device/emulator is connected
    println!("📱 Step 2/4: Checking for connected devices...");
    check_device_connected()?;

    // 5. Build APK with Gradle
    println!("🔧 Step 3/4: Building APK...");
    build_with_gradle(&result.output_dir)?;

    // 6. Install on device
    println!("📲 Step 4/4: Installing and launching app...");
    install_apk(&result.output_dir)?;

    // 7. Launch app
    launch_app(&config.android.package)?;

    // Restore original directory
    if project_dir != original_dir {
        env::set_current_dir(&original_dir)?;
    }

    println!("\n✅ App running on device!");

    Ok(())
}

fn check_device_connected() -> Result<()> {
    let output = Command::new("adb")
        .args(&["devices"])
        .output()
        .context("Failed to run 'adb devices'. Is Android SDK installed and adb in PATH?")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let device_count = stdout
        .lines()
        .skip(1) // Skip header
        .filter(|line| line.contains("device") && !line.contains("offline"))
        .count();

    if device_count == 0 {
        anyhow::bail!(
            "No devices connected. Please:\n  \
            1. Connect a device via USB with USB debugging enabled, or\n  \
            2. Start an emulator"
        );
    }

    println!("   Found {} device(s)", device_count);
    Ok(())
}

fn build_with_gradle(output_dir: &Path) -> Result<()> {
    // Check if gradlew exists
    let gradlew_path = if cfg!(windows) {
        output_dir.join("gradlew.bat")
    } else {
        output_dir.join("gradlew")
    };

    if !gradlew_path.exists() {
        anyhow::bail!(
            "Gradle wrapper not found. Please run:\n  \
            cd {}\n  \
            gradle wrapper\n\n  \
            Then try 'whitehall run' again.",
            output_dir.display()
        );
    }

    let gradle_cmd = if cfg!(windows) {
        "gradlew.bat"
    } else {
        "./gradlew"
    };

    let status = Command::new(gradle_cmd)
        .current_dir(output_dir)
        .args(&["assembleDebug"])
        .status()
        .context("Failed to run Gradle")?;

    if !status.success() {
        anyhow::bail!("Gradle build failed");
    }

    Ok(())
}

fn install_apk(output_dir: &Path) -> Result<()> {
    let apk_path = output_dir.join("app/build/outputs/apk/debug/app-debug.apk");

    if !apk_path.exists() {
        anyhow::bail!("APK not found at {}", apk_path.display());
    }

    let status = Command::new("adb")
        .args(&["install", "-r", apk_path.to_str().unwrap()])
        .status()
        .context("Failed to install APK")?;

    if !status.success() {
        anyhow::bail!("APK installation failed");
    }

    Ok(())
}

fn launch_app(package: &str) -> Result<()> {
    let activity = format!("{}/.MainActivity", package);

    let status = Command::new("adb")
        .args(&["shell", "am", "start", "-n", &activity])
        .status()
        .context("Failed to launch app")?;

    if !status.success() {
        anyhow::bail!("App launch failed");
    }

    Ok(())
}
