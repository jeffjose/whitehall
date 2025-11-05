use anyhow::Result;
use colored::Colorize;
use std::path::Path;

use crate::config;
use crate::toolchain::Toolchain;

/// Check system health and toolchain status
pub fn execute(manifest_path: &str) -> Result<()> {
    println!("{}", "Whitehall System Health Check".cyan().bold());
    println!("{}", "=".repeat(50));
    println!();

    let mut all_ok = true;

    // 1. Check Whitehall installation
    print_check("Whitehall installation");
    let version = env!("CARGO_PKG_VERSION");
    println!("  {} Version: {}", "✓".green(), version);
    println!();

    // 2. Check if we're in a project (optional)
    let in_project = Path::new(manifest_path).exists();

    if in_project {
        print_check("Project configuration");
        match config::load_config(manifest_path) {
            Ok(config) => {
                println!("  {} Project: {} v{}", "✓".green(), config.project.name, config.project.version);
                println!("  {} Package: {}", "✓".green(), config.android.package);
                println!("  {} Min SDK: {} | Target SDK: {}",
                    "✓".green(), config.android.min_sdk, config.android.target_sdk);
                println!("  {} Toolchain:", "✓".green());
                println!("    - Java: {}", config.toolchain.java);
                println!("    - Gradle: {}", config.toolchain.gradle);
                println!("    - AGP: {}", config.toolchain.agp);
                println!("    - Kotlin: {}", config.toolchain.kotlin);
                println!();

                // 3. Check toolchain availability
                print_check("Toolchain availability");
                let toolchain = Toolchain::new()?;

                // Check Java
                match toolchain.ensure_java(&config.toolchain.java) {
                    Ok(java_home) => {
                        println!("  {} Java {} installed at {}",
                            "✓".green(), config.toolchain.java, java_home.display());
                    }
                    Err(e) => {
                        println!("  {} Java {} not available: {}",
                            "✗".red(), config.toolchain.java, e);
                        all_ok = false;
                    }
                }

                // Check Gradle
                match toolchain.ensure_gradle(&config.toolchain.gradle) {
                    Ok(gradle_bin) => {
                        println!("  {} Gradle {} installed at {}",
                            "✓".green(), config.toolchain.gradle, gradle_bin.display());
                    }
                    Err(e) => {
                        println!("  {} Gradle {} not available: {}",
                            "✗".red(), config.toolchain.gradle, e);
                        all_ok = false;
                    }
                }

                // Check Android SDK
                match toolchain.ensure_android_sdk() {
                    Ok(sdk_root) => {
                        println!("  {} Android SDK installed at {}",
                            "✓".green(), sdk_root.display());
                    }
                    Err(e) => {
                        println!("  {} Android SDK not available: {}",
                            "✗".red(), e);
                        all_ok = false;
                    }
                }
                println!();
            }
            Err(e) => {
                println!("  {} Failed to load project config: {}", "✗".red(), e);
                all_ok = false;
                println!();
            }
        }
    } else {
        println!("{}", "  (Not in a Whitehall project)".dimmed());
        println!();
    }

    // 4. Check toolchain storage
    print_check("Toolchain storage");
    let toolchain = Toolchain::new()?;
    let root = toolchain.root();

    if root.exists() {
        let size = get_dir_size(root)?;
        println!("  {} Location: {}", "✓".green(), root.display());
        println!("  {} Size: {}", "✓".green(), human_size(size));

        // List installed components
        let java_count = count_versions(&root.join("java"))?;
        let gradle_count = count_versions(&root.join("gradle"))?;
        let has_sdk = root.join("android").exists();

        println!("  {} Installed components:", "✓".green());
        println!("    - Java: {} version(s)", java_count);
        println!("    - Gradle: {} version(s)", gradle_count);
        println!("    - Android SDK: {}", if has_sdk { "Yes" } else { "No" });
    } else {
        println!("  {} No toolchains installed yet", "ℹ".yellow());
        println!("    Run 'whitehall toolchain install' in a project to download");
    }
    println!();

    // 5. Check disk space
    print_check("Disk space");
    if let Ok(available) = get_available_space(root) {
        println!("  {} Available: {}", "✓".green(), human_size(available));

        if available < 1_000_000_000 {  // Less than 1GB
            println!("  {} Warning: Low disk space", "⚠".yellow());
            all_ok = false;
        }
    } else {
        println!("  {} Could not determine available space", "⚠".yellow());
    }
    println!();

    // Summary
    println!("{}", "=".repeat(50));
    if all_ok {
        println!("{} All checks passed!", "✓".green().bold());
        if !in_project {
            println!();
            println!("To get started:");
            println!("  whitehall init myapp");
            println!("  cd myapp");
            println!("  whitehall build");
        }
    } else {
        println!("{} Some issues detected", "⚠".yellow().bold());
        println!();
        println!("To install missing toolchains:");
        println!("  whitehall toolchain install");
    }

    Ok(())
}

fn print_check(name: &str) {
    println!("{} {}...", "Checking".cyan().bold(), name);
}

fn get_dir_size(path: &Path) -> Result<u64> {
    let mut size = 0;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                size += get_dir_size(&entry.path())?;
            } else {
                size += metadata.len();
            }
        }
    }
    Ok(size)
}

fn count_versions(dir: &Path) -> Result<usize> {
    if !dir.exists() {
        return Ok(0);
    }

    let count = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .count();

    Ok(count)
}

fn get_available_space(path: &Path) -> Result<u64> {
    use std::process::Command;

    // Use df command to get available space
    let output = Command::new("df")
        .arg("-k")  // Output in KB
        .arg(path)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse df output (skip header, get second line, 4th column)
    if let Some(line) = stdout.lines().nth(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            if let Ok(kb) = parts[3].parse::<u64>() {
                return Ok(kb * 1024);  // Convert KB to bytes
            }
        }
    }

    anyhow::bail!("Could not parse df output")
}

fn human_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_index])
}
