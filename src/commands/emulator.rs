use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashMap;

use crate::config;
use crate::toolchain::Toolchain;

/// AVD info with short ID and status
struct AvdInfo {
    name: String,
    short_id: String,
    status: AvdStatus,
}

#[derive(Clone)]
enum AvdStatus {
    Ok,
    Error(String),
}

/// Get list of AVDs with their info and status
fn get_avds(toolchain: &Toolchain, config: &crate::config::Config) -> Result<Vec<AvdInfo>> {
    let android_home = toolchain.root().join("android");
    let emulator_bin = android_home.join("emulator/emulator");

    if !emulator_bin.exists() {
        return Ok(vec![]);
    }

    // Get list of AVD names from emulator
    let output = std::process::Command::new(&emulator_bin)
        .env("ANDROID_HOME", &android_home)
        .env("ANDROID_SDK_ROOT", &android_home)
        .args(["-list-avds"])
        .output()
        .context("Failed to run emulator -list-avds")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let avd_names: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();

    // Get detailed status from avdmanager
    let errors = get_avd_errors(toolchain, config)?;

    let avds: Vec<AvdInfo> = avd_names
        .iter()
        .map(|name| {
            let short_id = generate_short_id(name);
            let status = errors.get(*name)
                .map(|e| AvdStatus::Error(e.clone()))
                .unwrap_or(AvdStatus::Ok);
            AvdInfo {
                name: name.to_string(),
                short_id,
                status,
            }
        })
        .collect();

    Ok(avds)
}

/// Parse avdmanager output to get error messages for broken AVDs
fn get_avd_errors(toolchain: &Toolchain, config: &crate::config::Config) -> Result<HashMap<String, String>> {
    let android_home = toolchain.root().join("android");
    let avdmanager = android_home.join("cmdline-tools/latest/bin/avdmanager");

    if !avdmanager.exists() {
        return Ok(HashMap::new());
    }

    let java_home = toolchain.root().join(format!("java/{}", config.toolchain.java));
    let java_home = if cfg!(target_os = "macos") {
        java_home.join("Contents/Home")
    } else {
        java_home
    };

    let output = std::process::Command::new(&avdmanager)
        .env("ANDROID_HOME", &android_home)
        .env("ANDROID_SDK_ROOT", &android_home)
        .env("JAVA_HOME", &java_home)
        .args(["list", "avd"])
        .output()
        .context("Failed to run avdmanager list avd")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut errors = HashMap::new();

    // Parse output - look for Name: and Error: pairs
    let mut current_name: Option<String> = None;
    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("Name:") {
            current_name = Some(line.trim_start_matches("Name:").trim().to_string());
        } else if line.starts_with("Error:") {
            if let Some(name) = current_name.take() {
                let error = line.trim_start_matches("Error:").trim().to_string();
                errors.insert(name, error);
            }
        }
    }

    Ok(errors)
}

/// Generate a short ID from AVD name (first 8 chars of hash)
fn generate_short_id(name: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    format!("{:08x}", hasher.finish() as u32)
}

/// Find AVD by partial match (short_id prefix or name substring)
fn find_avd<'a>(avds: &'a [AvdInfo], query: &str) -> Result<&'a AvdInfo> {
    let query_lower = query.to_lowercase();

    // First try exact short_id prefix match
    let matches: Vec<_> = avds
        .iter()
        .filter(|avd| avd.short_id.starts_with(&query_lower))
        .collect();

    if matches.len() == 1 {
        return Ok(matches[0]);
    }

    if matches.len() > 1 {
        anyhow::bail!(
            "Ambiguous ID '{}'. Matches:\n{}",
            query,
            matches.iter().map(|a| format!("  {} {}", a.short_id, a.name)).collect::<Vec<_>>().join("\n")
        );
    }

    // Try name substring match (case-insensitive)
    let name_matches: Vec<_> = avds
        .iter()
        .filter(|avd| avd.name.to_lowercase().contains(&query_lower))
        .collect();

    if name_matches.len() == 1 {
        return Ok(name_matches[0]);
    }

    if name_matches.len() > 1 {
        anyhow::bail!(
            "Ambiguous name '{}'. Matches:\n{}",
            query,
            name_matches.iter().map(|a| format!("  {} {}", a.short_id, a.name)).collect::<Vec<_>>().join("\n")
        );
    }

    anyhow::bail!("No emulator found matching '{}'", query);
}

/// List available AVDs
pub fn execute_list(manifest_path: &str) -> Result<()> {
    let config = config::load_config(manifest_path)?;
    let toolchain = Toolchain::new()?;

    // Ensure emulator is installed
    toolchain.ensure_android_sdk_with_emulator()?;

    let avds = get_avds(&toolchain, &config)?;

    if avds.is_empty() {
        println!("{}", "No emulators found.".yellow());
        println!();
        println!("Create one with:");
        println!("  whitehall emulator create");
    } else {
        // Calculate column widths
        let max_name_len = avds.iter().map(|a| a.name.len()).max().unwrap_or(0);

        // Print header
        println!(
            "{:<8}  {:<6}  {:<width$}  {}",
            "ID".dimmed(),
            "STATUS".dimmed(),
            "NAME".dimmed(),
            "ERROR".dimmed(),
            width = max_name_len
        );

        for avd in &avds {
            let (status_str, error_str) = match &avd.status {
                AvdStatus::Ok => (format!("{}    ", "ok".green()), "".to_string()),
                AvdStatus::Error(e) => (format!("{}", "error".red()), e.clone()),
            };
            println!(
                "{}  {}  {:<width$}  {}",
                avd.short_id.yellow(),
                status_str,
                avd.name,
                error_str.dimmed(),
                width = max_name_len
            );
        }
    }

    Ok(())
}

/// Start an emulator by name or ID
pub fn execute_start(manifest_path: &str, query: &str) -> Result<()> {
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

    // Find the AVD
    let avds = get_avds(&toolchain, &config)?;
    let avd = find_avd(&avds, query)?;

    println!("{} emulator '{}' ({})", "Starting".green().bold(), avd.name, avd.short_id.yellow());

    // Run emulator in background
    let child = std::process::Command::new(&emulator_bin)
        .env("ANDROID_HOME", &android_home)
        .env("ANDROID_SDK_ROOT", &android_home)
        .args(["-avd", &avd.name])
        .spawn()
        .context(format!("Failed to start emulator '{}'", avd.name))?;

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
    let system_image = format!("system-images;android-{};google_apis_playstore;x86_64", target_sdk);

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

    // Get the short ID for the new AVD
    let short_id = generate_short_id(avd_name);

    println!();
    println!("{} Created emulator '{}' ({})", "✓".green().bold(), avd_name, short_id.yellow());
    println!();
    println!("Start it with:");
    println!("  whitehall emulator start {}", short_id);

    Ok(())
}

/// Delete an AVD
pub fn execute_delete(manifest_path: &str, query: &str) -> Result<()> {
    let config = config::load_config(manifest_path)?;
    let toolchain = Toolchain::new()?;

    let android_home = toolchain.root().join("android");
    let avdmanager = android_home.join("cmdline-tools/latest/bin/avdmanager");

    if !avdmanager.exists() {
        anyhow::bail!("avdmanager not found. Run 'whitehall toolchain install' first.");
    }

    // Find the AVD
    let avds = get_avds(&toolchain, &config)?;
    let avd = find_avd(&avds, query)?;

    // Set JAVA_HOME for avdmanager
    let java_home = toolchain.ensure_java(&config.toolchain.java)?;

    let output = std::process::Command::new(&avdmanager)
        .env("ANDROID_HOME", &android_home)
        .env("ANDROID_SDK_ROOT", &android_home)
        .env("JAVA_HOME", &java_home)
        .args(["delete", "avd", "--name", &avd.name])
        .output()
        .context("Failed to delete AVD")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to delete emulator '{}': {}", avd.name, stderr.trim());
    }

    println!("  {} {}", "Deleted".green().bold(), avd.name);

    Ok(())
}
