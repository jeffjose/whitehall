use anyhow::{Context, Result};
use colored::Colorize;

use crate::config;
use crate::toolchain::Toolchain;

/// List available AVDs
pub fn execute_list(manifest_path: &str) -> Result<()> {
    let _config = config::load_config(manifest_path)?;
    let toolchain = Toolchain::new()?;

    // Ensure emulator is installed
    toolchain.ensure_android_sdk_with_emulator()?;

    let android_home = toolchain.root().join("android");
    let emulator_bin = android_home.join("emulator/emulator");

    if !emulator_bin.exists() {
        anyhow::bail!("Emulator not installed. Run 'whitehall toolchain install' first.");
    }

    let output = std::process::Command::new(&emulator_bin)
        .env("ANDROID_HOME", &android_home)
        .env("ANDROID_SDK_ROOT", &android_home)
        .args(["-list-avds"])
        .output()
        .context("Failed to run emulator -list-avds")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let avds: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();

    if avds.is_empty() {
        println!("{}", "No emulators found.".yellow());
        println!();
        println!("Create one with:");
        println!("  whitehall emulator create");
    } else {
        println!("{}", "Available emulators:".green().bold());
        for avd in avds {
            println!("  {}", avd);
        }
    }

    Ok(())
}

/// Start an emulator by name
pub fn execute_start(manifest_path: &str, name: &str) -> Result<()> {
    let config = config::load_config(manifest_path)?;
    let toolchain = Toolchain::new()?;

    // Ensure emulator and system image are installed
    toolchain.ensure_android_sdk_with_emulator()?;
    toolchain.ensure_system_image(config.android.target_sdk)?;

    let android_home = toolchain.root().join("android");
    let emulator_bin = android_home.join("emulator/emulator");

    if !emulator_bin.exists() {
        anyhow::bail!("Emulator not installed. Run 'whitehall toolchain install' first.");
    }

    println!("{} emulator '{}'", "Starting".green().bold(), name);

    // Run emulator in background
    let child = std::process::Command::new(&emulator_bin)
        .env("ANDROID_HOME", &android_home)
        .env("ANDROID_SDK_ROOT", &android_home)
        .args(["-avd", name])
        .spawn()
        .context(format!("Failed to start emulator '{}'", name))?;

    // Detach - don't wait for it
    std::mem::forget(child);

    println!("Emulator starting in background...");

    Ok(())
}

/// Create a new AVD
pub fn execute_create(manifest_path: &str, name: Option<&str>) -> Result<()> {
    let config = config::load_config(manifest_path)?;
    let toolchain = Toolchain::new()?;

    // Ensure emulator and system image are installed
    toolchain.ensure_android_sdk_with_emulator()?;
    toolchain.ensure_system_image(config.android.target_sdk)?;

    let android_home = toolchain.root().join("android");
    let avdmanager = android_home.join("cmdline-tools/latest/bin/avdmanager");

    if !avdmanager.exists() {
        anyhow::bail!("avdmanager not found. Run 'whitehall toolchain install' first.");
    }

    let avd_name = name.unwrap_or("whitehall");
    let target_sdk = config.android.target_sdk;
    let system_image = format!("system-images;android-{};google_apis;x86_64", target_sdk);

    println!("{} emulator '{}'", "Creating".green().bold(), avd_name);
    println!("  Target SDK: {}", target_sdk);
    println!("  System image: {}", system_image);

    // Set JAVA_HOME for avdmanager
    let java_home = toolchain.ensure_java(&config.toolchain.java)?;

    let status = std::process::Command::new(&avdmanager)
        .env("ANDROID_HOME", &android_home)
        .env("ANDROID_SDK_ROOT", &android_home)
        .env("JAVA_HOME", &java_home)
        .args([
            "create", "avd",
            "--name", avd_name,
            "--package", &system_image,
            "--device", "pixel_6",
            "--force",
        ])
        .status()
        .context("Failed to create AVD")?;

    if !status.success() {
        anyhow::bail!("Failed to create emulator '{}'", avd_name);
    }

    println!();
    println!("{} Created emulator '{}'", "✓".green().bold(), avd_name);
    println!();
    println!("Start it with:");
    println!("  whitehall emulator start {}", avd_name);

    Ok(())
}
