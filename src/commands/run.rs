use anyhow::{Context, Result};
use colored::Colorize;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use indicatif::{ProgressBar, ProgressStyle};
use notify::{Event, RecursiveMode, Watcher};
use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::build_pipeline;
use crate::config;
use crate::keyboard::{self, KeyAction, RawModeGuard};
use crate::single_file;
use crate::commands::{detect_target, Target};
use crate::commands::device;
use crate::toolchain::Toolchain;

pub fn execute(target: &str, device_query: Option<&str>, watch: bool) -> Result<()> {
    // Smart argument detection:
    // If target doesn't exist as a file/dir, but whitehall.toml exists in current dir,
    // treat target as device_query instead
    let path = Path::new(target);
    let is_current_dir = target == "." || target == "./";
    let target_exists = path.exists() || target.ends_with(".wh") || target.ends_with("whitehall.toml");
    let has_local_project = Path::new("whitehall.toml").exists();

    let (actual_target, actual_device) = if !is_current_dir && !target_exists && has_local_project {
        // Target doesn't exist but we have a local project - treat target as device
        (".", Some(target))
    } else {
        (target, device_query)
    };

    // Detect if we're running a project or single file
    match detect_target(actual_target) {
        Target::Project(manifest_path) => {
            if watch {
                execute_project_watch(&manifest_path, actual_device)
            } else {
                execute_project(&manifest_path, actual_device)
            }
        }
        Target::SingleFile(file_path) => {
            if watch {
                execute_single_file_watch(&file_path, actual_device)
            } else {
                execute_single_file(&file_path, actual_device)
            }
        }
    }
}

/// Run a single .wh file
fn execute_single_file(file_path: &str, device_query: Option<&str>) -> Result<()> {
    // Parse frontmatter
    let file_path_obj = Path::new(file_path);
    let content = fs::read_to_string(file_path_obj)
        .context(format!("Failed to read {}", file_path_obj.display()))?;

    let (single_config, code) = single_file::parse_frontmatter(&content)
        .context("Failed to parse frontmatter")?;

    // Generate temporary project
    let temp_project_dir = single_file::generate_temp_project(file_path_obj, &single_config, &code)
        .context("Failed to generate temporary project")?;

    // Change to temp project directory
    let original_dir = env::current_dir()?;
    env::set_current_dir(&temp_project_dir)
        .context("Failed to change to temp project directory")?;

    // Load config from generated whitehall.toml
    let config = config::load_config("whitehall.toml")
        .context("Failed to load generated whitehall.toml")?;

    // Build project
    let result = build_pipeline::execute_build(&config, false)?;

    if !result.errors.is_empty() {
        env::set_current_dir(&original_dir)?;
        eprintln!("{} build failed with {} error(s)", "error:".red().bold(), result.errors.len());
        for error in &result.errors {
            eprintln!("  {} - {}", error.file.display(), error.message);
        }
        anyhow::bail!("Build failed");
    }

    println!("   {} `{}` ({})",
        "Finished".green().bold(),
        single_config.app.name,
        single_config.app.package
    );

    // Initialize toolchain manager
    let toolchain = Toolchain::new()?;

    // Ensure all toolchains are ready (download in parallel if needed)
    toolchain.ensure_all_for_build(&config.toolchain.java, &config.toolchain.gradle)?;

    // Resolve device
    let device = device::resolve_device(&toolchain, device_query)?;
    println!("    {} {}", "Device".cyan(), device.display_name());

    // Continue with gradle, install, and launch
    build_with_gradle(&toolchain, &config, &result.output_dir)?;
    install_apk(&toolchain, &result.output_dir, &device.id, &config.android.package)?;
    clear_logcat(&toolchain, &device.id)?;
    launch_app(&toolchain, &config.android.package, &device.id)?;

    println!("  {} on {}", "Running".green().bold(), device.short_name());
    println!();

    // Stream logcat
    stream_logcat(&toolchain, &config.android.package, &device.id)?;

    // Restore original directory
    env::set_current_dir(&original_dir)?;

    Ok(())
}

/// Run a project (existing behavior)
fn execute_project(manifest_path: &str, device_query: Option<&str>) -> Result<()> {
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
        env::set_current_dir(&project_dir)?;
    }

    // 2. Load configuration
    let manifest_file = manifest_path.file_name().unwrap().to_str().unwrap();
    let config = config::load_config(manifest_file)?;

    // 3. Build project
    let result = build_pipeline::execute_build(&config, false)?;

    if !result.errors.is_empty() {
        eprintln!("{} build failed with {} error(s)", "error:".red().bold(), result.errors.len());
        for error in &result.errors {
            eprintln!("  {} - {}", error.file.display(), error.message);
        }
        anyhow::bail!("Build failed");
    }

    println!("   {} `{}` v{} ({})",
        "Finished".green().bold(),
        config.project.name,
        config.project.version,
        config.android.package
    );

    // Initialize toolchain manager
    let toolchain = Toolchain::new()?;

    // 3.5. Ensure all toolchains are ready (download in parallel if needed)
    toolchain.ensure_all_for_build(&config.toolchain.java, &config.toolchain.gradle)?;

    // 4. Resolve device
    let device = device::resolve_device(&toolchain, device_query)?;
    println!("    {} {}", "Device".cyan(), device.display_name());

    // 5. Build APK with Gradle
    build_with_gradle(&toolchain, &config, &result.output_dir)?;

    // 6. Install on device (skipped if APK unchanged)
    install_apk(&toolchain, &result.output_dir, &device.id, &config.android.package)?;

    // 7. Clear logcat and launch app
    clear_logcat(&toolchain, &device.id)?;
    launch_app(&toolchain, &config.android.package, &device.id)?;

    println!("  {} on {}", "Running".green().bold(), device.short_name());
    println!();

    // 8. Stream logcat filtered to this app
    stream_logcat(&toolchain, &config.android.package, &device.id)?;

    // Restore original directory
    if project_dir != original_dir {
        env::set_current_dir(&original_dir)?;
    }

    Ok(())
}

fn build_with_gradle(toolchain: &Toolchain, config: &crate::config::Config, output_dir: &Path) -> Result<()> {
    // Create a spinner to show progress
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.dim} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
    );
    pb.set_message("Building APK with Gradle...");
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let mut gradle = toolchain.gradle_cmd(&config.toolchain.java, &config.toolchain.gradle)?;

    let status = gradle
        .current_dir(output_dir)
        .args(&["assembleDebug", "--console=plain", "--quiet"])
        .status()
        .context("Failed to run Gradle")?;

    // Clear the progress bar (it disappears)
    pb.finish_and_clear();

    if !status.success() {
        anyhow::bail!("Gradle build failed");
    }

    Ok(())
}

fn install_apk(toolchain: &Toolchain, output_dir: &Path, device_id: &str, package: &str) -> Result<()> {
    let apk_path = output_dir.join("app/build/outputs/apk/debug/app-debug.apk");

    if !apk_path.exists() {
        anyhow::bail!("APK not found at {}", apk_path.display());
    }

    // Check if app is installed on device
    let is_installed = is_app_installed(toolchain, device_id, package)?;

    // Calculate APK hash to check if install is needed
    let apk_hash = calculate_file_hash(&apk_path)?;
    // Store hash in .whitehall directory (not in gradle's build output which may be cleaned)
    let whitehall_dir = output_dir.join(".whitehall");
    let _ = fs::create_dir_all(&whitehall_dir);
    let hash_file = whitehall_dir.join(format!("installed-{}.hash", device_id));

    // Skip install if: app is installed AND hash matches
    if is_installed {
        if let Ok(stored_hash) = fs::read_to_string(&hash_file) {
            if stored_hash.trim() == apk_hash {
                // APK unchanged and app is installed, skip installation
                // Use \r\n for raw mode compatibility
                print!("  {} (APK unchanged)\r\n", "Skipped install".dimmed());
                return Ok(());
            }
        }
    }

    let status = toolchain
        .adb_cmd()?
        .args(["-s", device_id, "install", "-r", apk_path.to_str().unwrap()])
        .status()
        .context("Failed to install APK")?;

    if !status.success() {
        anyhow::bail!("APK installation failed");
    }

    // Store hash for next comparison
    let _ = fs::write(&hash_file, &apk_hash);

    Ok(())
}

/// Check if an app is installed on the device
fn is_app_installed(toolchain: &Toolchain, device_id: &str, package: &str) -> Result<bool> {
    let output = toolchain
        .adb_cmd()?
        .args(["-s", device_id, "shell", "pm", "list", "packages", package])
        .output()
        .context("Failed to check installed packages")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    // pm list packages returns "package:com.example.app" if installed
    Ok(stdout.contains(&format!("package:{}", package)))
}

/// Calculate SHA256 hash of a file
fn calculate_file_hash(path: &Path) -> Result<String> {
    use std::io::Read;
    use sha2::{Sha256, Digest};

    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Force stop the app if running
fn stop_app(toolchain: &Toolchain, package: &str, device_id: &str) -> Result<()> {
    let _ = toolchain
        .adb_cmd()?
        .args(["-s", device_id, "shell", "am", "force-stop", package])
        .output();
    Ok(())
}

fn launch_app(toolchain: &Toolchain, package: &str, device_id: &str) -> Result<()> {
    let activity = format!("{}/.MainActivity", package);

    // Capture output and reprint with \r\n for raw mode compatibility
    let output = toolchain
        .adb_cmd()?
        .args(["-s", device_id, "shell", "am", "start", "-n", &activity])
        .output()
        .context("Failed to launch app")?;

    // Print stdout with proper line endings
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        print!("{}\r\n", line);
    }

    // Print stderr with proper line endings
    let stderr = String::from_utf8_lossy(&output.stderr);
    for line in stderr.lines() {
        eprint!("{}\r\n", line);
    }

    if !output.status.success() {
        anyhow::bail!("App launch failed");
    }

    Ok(())
}

fn stream_logcat(toolchain: &Toolchain, package: &str, device_id: &str) -> Result<()> {
    use std::io::{BufRead, BufReader};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    println!("{} (press Ctrl+C to stop app and exit)", "Streaming logcat".cyan().bold());
    println!("{}", "─".repeat(80).dimmed());

    // Set up Ctrl+C handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let pkg = package.to_string();
    let dev = device_id.to_string();
    let tc = toolchain.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);

        // Force-stop the app on the device
        println!("\n{}", "Stopping app...".yellow());
        let _ = tc
            .adb_cmd()
            .map(|mut cmd| {
                cmd.args(["-s", &dev, "shell", "am", "force-stop", &pkg])
                    .output()
            });
    }).expect("Error setting Ctrl-C handler");

    // Note: logcat is cleared before launch_app() so we capture init logs
    // Stream logcat with brief format - we'll filter in Rust
    let mut child = toolchain
        .adb_cmd()?
        .args(["-s", device_id, "logcat", "-v", "brief"])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .context("Failed to start logcat")?;

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut app_pids: std::collections::HashSet<String> = std::collections::HashSet::new();

        for line in reader.lines() {
            // Check if we should stop
            if !running.load(Ordering::SeqCst) {
                break;
            }

            match line {
                Ok(line) => {
                    // Track our app's PID from ActivityManager logs
                    // Format: "Start proc 15171:com.example.picsum/..."
                    if line.contains("Start proc") && line.contains(package) {
                        if let Some(pid_start) = line.find("Start proc ") {
                            let after_proc = &line[pid_start + 11..];
                            if let Some(colon_pos) = after_proc.find(':') {
                                let pid = after_proc[..colon_pos].trim();
                                app_pids.insert(pid.to_string());
                            }
                        }
                    }

                    // Extract PID from logcat line (brief format: "L/Tag( PID): Message")
                    let line_pid = extract_pid_from_logcat(&line);

                    // Filter: show line if it's from our app's PID, contains package name, or is a crash
                    let relevant = line_pid.as_ref().map_or(false, |pid| app_pids.contains(pid))
                        || line.contains(package)
                        || line.contains("AndroidRuntime")  // Runtime errors/crashes
                        || (line.contains("ActivityManager") && line.contains(package));

                    if !relevant {
                        continue;
                    }

                    // Skip noisy logs that aren't helpful for debugging
                    if line.contains("AiAiEcho")
                        || line.contains("PackageUpdatedTask")
                        || line.contains("ApkAssets")
                        || line.contains("ziparchive")
                        || line.contains("nativeloader")
                        || line.contains("ProximityAuth")
                        || line.contains("SQLiteLog")
                        || (line.contains("ActivityThread") && line.contains("REPLACED"))
                        || line.contains("Finsky")
                        || line.contains("InputManager-JNI")
                        || line.contains("CoreBackPreview")
                    {
                        continue;
                    }

                    // Parse and colorize logcat line
                    // Format: "L/Tag(PID): Message" where L is the log level
                    print_colorized_logcat(&line);
                }
                Err(_) => break,
            }
        }
    }

    // Kill the logcat process when we exit
    let _ = child.kill();

    Ok(())
}

/// Parse and print a colorized logcat line.
/// Format: "L/Tag(PID): Message" where L is the log level (V, D, I, W, E, F)
fn print_colorized_logcat(line: &str) {
    // Try to parse the brief format: "L/Tag( PID): Message"
    if line.len() >= 2 && line.chars().nth(1) == Some('/') {
        let level = line.chars().next().unwrap();

        // Find the end of tag (look for the colon after the closing paren)
        if let Some(colon_pos) = line.find("): ") {
            let tag_part = &line[2..colon_pos + 1]; // "Tag(PID)"
            let message = &line[colon_pos + 3..];   // Everything after "): "

            // Color the level indicator
            let level_colored = match level {
                'E' | 'F' => format!("{}", level.to_string().red().bold()),
                'W' => format!("{}", level.to_string().yellow().bold()),
                'I' => format!("{}", level.to_string().green().bold()),
                'D' => format!("{}", level.to_string().blue().bold()),
                'V' => format!("{}", level.to_string().white().dimmed()),
                _ => level.to_string(),
            };

            // Color the tag (cyan)
            let tag_colored = format!("{}", tag_part.cyan());

            // Color the message based on level
            let message_colored = match level {
                'E' | 'F' => format!("{}", message.red()),
                'W' => format!("{}", message.yellow()),
                'I' => message.to_string(),
                'D' => format!("{}", message.dimmed()),
                'V' => format!("{}", message.dimmed()),
                _ => message.to_string(),
            };

            // Use \r\n for raw mode compatibility (raw mode doesn't auto-add \r)
            print!("{}/{} {}\r\n", level_colored, tag_colored, message_colored);
            return;
        }
    }

    // Fallback: print as-is if we couldn't parse
    // Use \r\n for raw mode compatibility
    print!("{}\r\n", line);
}

// ============================================================================
// Watch mode implementations
// ============================================================================

/// Watch and run a single .wh file
fn execute_single_file_watch(file_path: &str, device_query: Option<&str>) -> Result<()> {
    let file_path_buf = PathBuf::from(file_path);
    let file_path_obj = Path::new(file_path);
    let original_dir = env::current_dir()?;

    // Load gitignore from the file's directory
    let watch_dir = file_path_buf.parent().unwrap_or(Path::new("."));
    let gitignore = load_gitignore(watch_dir);

    // Parse frontmatter for initial build
    let content = fs::read_to_string(file_path_obj)
        .context(format!("Failed to read {}", file_path_obj.display()))?;
    let (single_config, code) = single_file::parse_frontmatter(&content)?;

    // Generate temporary project
    let temp_project_dir = single_file::generate_temp_project(file_path_obj, &single_config, &code)?;

    // Change to temp project directory
    env::set_current_dir(&temp_project_dir)?;

    // Load config
    let config = config::load_config("whitehall.toml")?;

    // Initialize toolchain
    let toolchain = Toolchain::new()?;
    toolchain.ensure_all_for_build(&config.toolchain.java, &config.toolchain.gradle)?;

    // Resolve device once at start
    let device = device::resolve_device(&toolchain, device_query)?;
    println!("    {} {}", "Device".cyan(), device.display_name());

    // Initial build and run
    let start = Instant::now();
    let mut logcat_handle: Option<LogcatHandle> = None;
    match run_full_cycle(&toolchain, &config, &device.id, &config.android.package) {
        Ok(_) => {
            print_build_status(start.elapsed());
            // Start logcat after successful build
            println!("{}", "─".repeat(60).dimmed());
            if let Ok(handle) = start_logcat_background(&toolchain, &config.android.package, &device.id) {
                logcat_handle = Some(handle);
            }
        }
        Err(e) => {
            eprintln!("{} {}", "error:".red().bold(), e);
        }
    }

    // Restore to original directory for watching
    env::set_current_dir(&original_dir)?;

    keyboard::print_shortcuts();

    // Set up file watcher
    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res {
            let _ = tx.send(event);
        }
    })?;

    // Watch the single .wh file
    watcher.watch(&file_path_buf, RecursiveMode::NonRecursive)?;

    // Enable raw mode for keyboard input
    let _raw_guard = RawModeGuard::new()?;

    // Helper closure to run a full rebuild cycle
    let run_rebuild = |logcat_handle: &mut Option<LogcatHandle>| -> Result<()> {
        // Stop current logcat
        if let Some(ref mut handle) = logcat_handle {
            handle.stop();
        }

        // Re-read and regenerate project
        let content = fs::read_to_string(&file_path_buf)?;
        let (single_config, code) = single_file::parse_frontmatter(&content)?;
        let temp_project_dir = single_file::generate_temp_project(&file_path_buf, &single_config, &code)?;

        env::set_current_dir(&temp_project_dir)?;
        let config = config::load_config("whitehall.toml")?;

        print!("{}\r\n", "─".repeat(60).dimmed());
        let start = Instant::now();
        match run_full_cycle(&toolchain, &config, &device.id, &config.android.package) {
            Ok(_) => {
                print_build_status(start.elapsed());
                // Restart logcat
                print!("{}\r\n", "─".repeat(60).dimmed());
                if let Ok(handle) = start_logcat_background(&toolchain, &config.android.package, &device.id) {
                    *logcat_handle = Some(handle);
                }
            }
            Err(e) => {
                eprint!("{} {}\r\n", "error:".red().bold(), e);
            }
        }

        env::set_current_dir(&original_dir)?;
        Ok(())
    };

    // Watch loop with debouncing
    let mut last_build = Instant::now();
    loop {
        // Check for keyboard input first
        match keyboard::poll_key(Duration::from_millis(100))? {
            KeyAction::Quit => {
                if let Some(ref mut handle) = logcat_handle {
                    handle.stop();
                }
                print!("\r\n   Exiting watch mode\r\n");
                return Ok(());
            }
            KeyAction::Rebuild => {
                print!("\r\n   Rebuilding...\r\n");
                run_rebuild(&mut logcat_handle)?;
                last_build = Instant::now();
                continue;
            }
            KeyAction::None => {}
        }

        // Check for file system events (non-blocking)
        while let Ok(event) = rx.try_recv() {
            if should_rebuild(&event, &gitignore) {
                // Debounce - wait for file saves to settle
                if last_build.elapsed() < Duration::from_millis(500) {
                    continue;
                }
                while rx.try_recv().is_ok() {}

                run_rebuild(&mut logcat_handle)?;

                // Check if more changes came in during build
                // If so, don't reset timer - allow immediate rebuild
                if rx.try_recv().is_err() {
                    last_build = Instant::now();
                } else {
                    // Drain remaining events, will rebuild on next iteration
                    while rx.try_recv().is_ok() {}
                }
            }
        }
    }
}

/// Watch and run a project
fn execute_project_watch(manifest_path: &str, device_query: Option<&str>) -> Result<()> {
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

    if project_dir != original_dir {
        env::set_current_dir(&project_dir)?;
    }

    // Load gitignore
    let gitignore = load_gitignore(&env::current_dir()?);

    // Load configuration
    let manifest_file = manifest_path.file_name().unwrap().to_str().unwrap();
    let config = config::load_config(manifest_file)?;

    // Initialize toolchain
    let toolchain = Toolchain::new()?;
    toolchain.ensure_all_for_build(&config.toolchain.java, &config.toolchain.gradle)?;

    // Resolve device once at start
    let device = device::resolve_device(&toolchain, device_query)?;
    println!("    {} {}", "Device".cyan(), device.display_name());

    // Initial build and run
    let start = Instant::now();
    let mut logcat_handle: Option<LogcatHandle> = None;
    match run_full_cycle(&toolchain, &config, &device.id, &config.android.package) {
        Ok(_) => {
            print_build_status(start.elapsed());
            // Start logcat after successful build
            println!("{}", "─".repeat(60).dimmed());
            if let Ok(handle) = start_logcat_background(&toolchain, &config.android.package, &device.id) {
                logcat_handle = Some(handle);
            }
        }
        Err(e) => {
            eprintln!("{} {}", "error:".red().bold(), e);
        }
    }

    keyboard::print_shortcuts();

    // Set up file watcher
    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res {
            let _ = tx.send(event);
        }
    })?;

    // Watch src/ directory and whitehall.toml
    watcher.watch(Path::new("src"), RecursiveMode::Recursive)?;
    watcher.watch(Path::new(manifest_file), RecursiveMode::NonRecursive)?;

    // Enable raw mode for keyboard input
    let _raw_guard = RawModeGuard::new()?;

    // Helper closure to run a full rebuild cycle
    let run_rebuild = |logcat_handle: &mut Option<LogcatHandle>| {
        // Stop current logcat
        if let Some(ref mut handle) = logcat_handle {
            handle.stop();
        }

        print!("{}\r\n", "─".repeat(60).dimmed());
        let start = Instant::now();
        match run_full_cycle(&toolchain, &config, &device.id, &config.android.package) {
            Ok(_) => {
                print_build_status(start.elapsed());
                // Restart logcat
                print!("{}\r\n", "─".repeat(60).dimmed());
                if let Ok(handle) = start_logcat_background(&toolchain, &config.android.package, &device.id) {
                    *logcat_handle = Some(handle);
                }
            }
            Err(e) => {
                eprint!("{} {}\r\n", "error:".red().bold(), e);
            }
        }
    };

    // Watch loop with debouncing
    let mut last_build = Instant::now();
    loop {
        // Check for keyboard input first
        match keyboard::poll_key(Duration::from_millis(100))? {
            KeyAction::Quit => {
                if let Some(ref mut handle) = logcat_handle {
                    handle.stop();
                }
                print!("\r\n   Exiting watch mode\r\n");
                return Ok(());
            }
            KeyAction::Rebuild => {
                print!("\r\n   Rebuilding...\r\n");
                run_rebuild(&mut logcat_handle);
                last_build = Instant::now();
                continue;
            }
            KeyAction::None => {}
        }

        // Check for file system events (non-blocking)
        while let Ok(event) = rx.try_recv() {
            if should_rebuild(&event, &gitignore) {
                // Debounce - wait for file saves to settle
                if last_build.elapsed() < Duration::from_millis(500) {
                    continue;
                }
                while rx.try_recv().is_ok() {}

                run_rebuild(&mut logcat_handle);

                // Check if more changes came in during build
                // If so, don't reset timer - allow immediate rebuild
                if rx.try_recv().is_err() {
                    last_build = Instant::now();
                } else {
                    // Drain remaining events, will rebuild on next iteration
                    while rx.try_recv().is_ok() {}
                }
            }
        }
    }
}

/// Run full build cycle: transpile, gradle, install, launch
fn run_full_cycle(
    toolchain: &Toolchain,
    config: &crate::config::Config,
    device_id: &str,
    package: &str,
) -> Result<()> {
    // 1. Run transpilation (incremental)
    let result = build_pipeline::execute_build(config, false)?;

    if !result.errors.is_empty() {
        for error in &result.errors {
            // Use \r\n for raw mode compatibility
            eprint!("  {} - {}\r\n", error.file.display(), error.message);
        }
        anyhow::bail!("Build failed with {} error(s)", result.errors.len());
    }

    // 2. Build APK with Gradle
    build_with_gradle(toolchain, config, &result.output_dir)?;

    // 3. Install APK (skipped if APK unchanged)
    install_apk(toolchain, &result.output_dir, device_id, package)?;

    // 4. Stop app, clear logcat, and launch fresh
    stop_app(toolchain, package, device_id)?;
    clear_logcat(toolchain, device_id)?;
    launch_app(toolchain, package, device_id)?;

    Ok(())
}

/// Print build status on a new line
fn print_build_status(elapsed: Duration) {
    let ms = elapsed.as_millis();
    // Use \r\n for raw mode compatibility
    print!("   {} in {}ms\r\n", "Finished".green().bold(), format!("{}", ms).cyan());
}

/// Clear logcat buffer before launching app to capture init logs
fn clear_logcat(toolchain: &Toolchain, device_id: &str) -> Result<()> {
    let _ = toolchain
        .adb_cmd()?
        .args(["-s", device_id, "logcat", "-c"])
        .output();
    Ok(())
}

/// Extract PID from a logcat line in brief format: "L/Tag( PID): Message"
fn extract_pid_from_logcat(line: &str) -> Option<String> {
    // Brief format: "I/Tag( 1234): message" or "I/Tag(12345): message"
    if let Some(paren_start) = line.find('(') {
        if let Some(paren_end) = line[paren_start..].find(')') {
            let pid = line[paren_start + 1..paren_start + paren_end].trim();
            if pid.chars().all(|c| c.is_ascii_digit()) {
                return Some(pid.to_string());
            }
        }
    }
    None
}

/// Load gitignore from the directory if it exists
fn load_gitignore(dir: &Path) -> Gitignore {
    let gitignore_path = dir.join(".gitignore");
    let mut builder = GitignoreBuilder::new(dir);

    if gitignore_path.exists() {
        let _ = builder.add(&gitignore_path);
    }

    builder.build().unwrap_or_else(|_| Gitignore::empty())
}

/// Check if an event should trigger a rebuild
fn should_rebuild(event: &notify::Event, gitignore: &Gitignore) -> bool {
    use notify::EventKind::*;

    match event.kind {
        Modify(_) | Create(_) | Remove(_) => {
            event.paths.iter().any(|p| {
                let is_relevant = p.extension().map_or(false, |ext| ext == "wh")
                    || p.file_name().map_or(false, |name| name == "whitehall.toml");

                if !is_relevant {
                    return false;
                }

                !gitignore.matched(p, p.is_dir()).is_ignore()
            })
        }
        _ => false,
    }
}

/// Handle for background logcat process
struct LogcatHandle {
    child: Child,
    running: Arc<AtomicBool>,
}

impl LogcatHandle {
    fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        let _ = self.child.kill();
    }
}

/// Start logcat streaming in background thread
fn start_logcat_background(
    toolchain: &Toolchain,
    package: &str,
    device_id: &str,
) -> Result<LogcatHandle> {
    // Note: logcat is cleared before launch_app() in run_full_cycle so we capture init logs
    // Start logcat process
    let mut child = toolchain
        .adb_cmd()?
        .args(["-s", device_id, "logcat", "-v", "brief"])
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to start logcat")?;

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    let package = package.to_string();

    // Take stdout and spawn reader thread
    if let Some(stdout) = child.stdout.take() {
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            let mut app_pids: std::collections::HashSet<String> = std::collections::HashSet::new();

            for line in reader.lines() {
                if !running_clone.load(Ordering::SeqCst) {
                    break;
                }
                if let Ok(line) = line {
                    // Track our app's PID from ActivityManager logs
                    // Format: "Start proc 15171:com.example.picsum/..."
                    if line.contains("Start proc") && line.contains(&package) {
                        if let Some(pid_start) = line.find("Start proc ") {
                            let after_proc = &line[pid_start + 11..];
                            if let Some(colon_pos) = after_proc.find(':') {
                                let pid = after_proc[..colon_pos].trim();
                                app_pids.insert(pid.to_string());
                            }
                        }
                    }

                    // Extract PID from logcat line (brief format: "L/Tag( PID): Message")
                    let line_pid = extract_pid_from_logcat(&line);

                    // Filter: show line if it's from our app's PID, contains package name, or is a crash
                    let relevant = line_pid.as_ref().map_or(false, |pid| app_pids.contains(pid))
                        || line.contains(&package)
                        || line.contains("AndroidRuntime")
                        || (line.contains("ActivityManager") && line.contains(&package));

                    if !relevant {
                        continue;
                    }

                    // Skip noisy logs
                    if line.contains("AiAiEcho")
                        || line.contains("PackageUpdatedTask")
                        || line.contains("ApkAssets")
                        || line.contains("ziparchive")
                        || line.contains("nativeloader")
                        || line.contains("ProximityAuth")
                        || line.contains("SQLiteLog")
                        || (line.contains("ActivityThread") && line.contains("REPLACED"))
                        || line.contains("Finsky")
                        || line.contains("InputManager-JNI")
                        || line.contains("CoreBackPreview")
                    {
                        continue;
                    }

                    print_colorized_logcat(&line);
                }
            }
        });
    }

    Ok(LogcatHandle { child, running })
}
