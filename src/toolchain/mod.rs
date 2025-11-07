mod defaults;
mod downloader;
mod platform;
pub mod validator;

pub use defaults::*;
pub use platform::Platform;
pub use validator::validate_compatibility;

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

/// Core toolchain manager for Whitehall
///
/// Manages Java, Gradle, and Android SDK installations in ~/.whitehall/toolchains/
/// Each project specifies required versions in [toolchain] section of whitehall.toml
pub struct Toolchain {
    root: PathBuf,
}

impl Toolchain {
    /// Create new toolchain manager
    ///
    /// Uses ~/.whitehall/toolchains/ as the root directory for all toolchains
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

        let root = home.join(".whitehall").join("toolchains");

        // Ensure root directory exists
        std::fs::create_dir_all(&root)
            .context("Failed to create toolchain directory")?;

        Ok(Self { root })
    }

    /// Ensure Java is installed for the given version
    ///
    /// Downloads and installs Java if not present
    ///
    /// # Arguments
    /// * `version` - Java version (e.g., "11", "17", "21")
    pub fn ensure_java(&self, version: &str) -> Result<PathBuf> {
        let java_home = self.root.join(format!("java/{}", version));

        if !java_home.exists() {
            // Download Java
            self.download_java(version)?;
        }

        // Verify java binary exists
        let java_bin = if cfg!(target_os = "macos") {
            java_home.join("Contents/Home/bin/java")
        } else {
            java_home.join("bin/java")
        };

        if !java_bin.exists() {
            anyhow::bail!(
                "Java {} installation corrupt: binary not found at {}",
                version,
                java_bin.display()
            );
        }

        // Return JAVA_HOME path (not bin/java)
        if cfg!(target_os = "macos") {
            Ok(java_home.join("Contents/Home"))
        } else {
            Ok(java_home)
        }
    }

    /// Download and install Java
    fn download_java(&self, version: &str) -> Result<()> {
        let platform = Platform::detect()?;
        let url = downloader::get_java_download_url(version, platform)?;

        let java_dir = self.root.join("java");
        std::fs::create_dir_all(&java_dir)?;

        // Download to temporary file
        let archive_path = java_dir.join(format!("java-{}.tar.gz", version));
        downloader::download_with_retry(&url, &archive_path)?;

        // Extract
        downloader::extract_tar_gz(&archive_path, &java_dir)?;

        // The extracted directory structure varies, need to rename properly
        // Adoptium extracts to jdk-VERSION/ directory
        let extracted_dirs: Vec<_> = std::fs::read_dir(&java_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter(|e| e.file_name().to_str().unwrap().starts_with("jdk-"))
            .collect();

        if let Some(extracted_dir) = extracted_dirs.first() {
            let target = java_dir.join(version);
            std::fs::rename(extracted_dir.path(), &target)?;
        }

        // Clean up archive
        std::fs::remove_file(&archive_path)?;

        Ok(())
    }

    /// Ensure Gradle is installed for the given version
    ///
    /// Downloads and installs Gradle if not present
    ///
    /// # Arguments
    /// * `version` - Gradle version (e.g., "7.6", "8.0", "8.4")
    pub fn ensure_gradle(&self, version: &str) -> Result<PathBuf> {
        let gradle_home = self.root.join(format!("gradle/{}", version));

        if !gradle_home.exists() {
            self.download_gradle(version)?;
        }

        let gradle_bin = gradle_home.join("bin/gradle");

        if !gradle_bin.exists() {
            anyhow::bail!(
                "Gradle {} installation corrupt: binary not found at {}",
                version,
                gradle_bin.display()
            );
        }

        Ok(gradle_bin)
    }

    /// Download and install Gradle
    fn download_gradle(&self, version: &str) -> Result<()> {
        let url = downloader::get_gradle_download_url(version);

        let gradle_dir = self.root.join("gradle");
        std::fs::create_dir_all(&gradle_dir)?;

        // Download to temporary file
        let archive_path = gradle_dir.join(format!("gradle-{}.zip", version));
        downloader::download_with_retry(&url, &archive_path)?;

        // Extract
        downloader::extract_zip(&archive_path, &gradle_dir)?;

        // Gradle extracts to gradle-VERSION/ directory, rename to VERSION/
        let extracted = gradle_dir.join(format!("gradle-{}", version));
        let target = gradle_dir.join(version);
        if extracted.exists() {
            std::fs::rename(&extracted, &target)?;
        }

        // Clean up archive
        std::fs::remove_file(&archive_path)?;

        Ok(())
    }

    /// Ensure Android SDK is installed
    ///
    /// Downloads and installs Android SDK if not present
    pub fn ensure_android_sdk(&self) -> Result<PathBuf> {
        let sdk_root = self.root.join("android");

        if !sdk_root.exists() {
            self.download_android_sdk()?;
        }

        // Verify critical components exist
        let platform_tools = sdk_root.join("platform-tools/adb");
        if !platform_tools.exists() {
            anyhow::bail!(
                "Android SDK installation corrupt: platform-tools not found at {}",
                platform_tools.display()
            );
        }

        Ok(sdk_root)
    }

    /// Download and install Android SDK
    fn download_android_sdk(&self) -> Result<()> {
        let platform = Platform::detect()?;
        let url = downloader::get_android_cmdline_tools_url(platform)?;

        let sdk_root = self.root.join("android");
        std::fs::create_dir_all(&sdk_root)?;

        // Download cmdline-tools
        let archive_path = sdk_root.join("cmdline-tools.zip");
        downloader::download_with_retry(&url, &archive_path)?;

        // Extract to cmdline-tools/latest/
        // IMPORTANT: Must be in "latest" subdirectory for sdkmanager to work
        let cmdline_tools_dir = sdk_root.join("cmdline-tools");
        std::fs::create_dir_all(&cmdline_tools_dir)?;

        downloader::extract_zip(&archive_path, &cmdline_tools_dir)?;

        // Rename extracted "cmdline-tools" to "latest"
        let extracted = cmdline_tools_dir.join("cmdline-tools");
        let latest = cmdline_tools_dir.join("latest");
        if extracted.exists() {
            std::fs::rename(&extracted, &latest)?;
        }

        // Clean up archive
        std::fs::remove_file(&archive_path)?;

        // Now use sdkmanager to install essential components
        self.install_sdk_components()?;

        Ok(())
    }

    /// Use sdkmanager to install essential SDK components
    fn install_sdk_components(&self) -> Result<()> {
        let sdk_root = self.root.join("android");
        let sdkmanager = sdk_root.join("cmdline-tools/latest/bin/sdkmanager");

        if !sdkmanager.exists() {
            anyhow::bail!("sdkmanager not found at {}", sdkmanager.display());
        }

        // Accept licenses first using yes command to auto-accept
        let status = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "yes | {} --sdk_root={} --licenses 2>&1 > /dev/null",
                sdkmanager.display(),
                sdk_root.display()
            ))
            .env("ANDROID_HOME", &sdk_root)
            .status()
            .context("Failed to accept SDK licenses")?;

        if !status.success() {
            anyhow::bail!("Failed to accept SDK licenses");
        }

        // Install essential components
        let components = vec![
            "platform-tools",        // adb, fastboot
            "build-tools;34.0.0",    // aapt, dx, etc.
            "platforms;android-34",  // Android 14 platform
            "emulator",              // Android emulator
        ];

        for component in components {
            let output = Command::new(&sdkmanager)
                .arg(format!("--sdk_root={}", sdk_root.display()))
                .arg(component)
                .env("ANDROID_HOME", &sdk_root)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .output()
                .with_context(|| format!("Failed to run sdkmanager for {}", component))?;

            if !output.status.success() {
                anyhow::bail!("Failed to install {}", component);
            }
        }

        Ok(())
    }

    /// Get configured gradle Command for this toolchain
    ///
    /// Returns a Command with proper JAVA_HOME and ANDROID_HOME environment variables set
    ///
    /// # Arguments
    /// * `java_version` - Java version to use
    /// * `gradle_version` - Gradle version to use
    pub fn gradle_cmd(&self, java_version: &str, gradle_version: &str) -> Result<Command> {
        let java_home = self.ensure_java(java_version)?;
        let gradle_bin = self.ensure_gradle(gradle_version)?;
        let android_home = self.ensure_android_sdk()?;

        let mut cmd = Command::new(gradle_bin);
        cmd.env("JAVA_HOME", java_home);
        cmd.env("ANDROID_HOME", &android_home);

        // Unset ANDROID_SDK_ROOT to prevent conflicts with ANDROID_HOME
        // (Gradle complains if both are set to different paths)
        cmd.env_remove("ANDROID_SDK_ROOT");

        // Isolate Gradle daemon per version to prevent conflicts
        let gradle_user_home = self.root.join(format!("gradle-home/{}", gradle_version));
        std::fs::create_dir_all(&gradle_user_home)
            .context("Failed to create GRADLE_USER_HOME")?;
        cmd.env("GRADLE_USER_HOME", gradle_user_home);

        Ok(cmd)
    }

    /// Get configured adb Command
    ///
    /// Returns a Command for adb with proper ANDROID_HOME set
    pub fn adb_cmd(&self) -> Result<Command> {
        let android_home = self.ensure_android_sdk()?;
        let adb_bin = android_home.join("platform-tools/adb");

        let mut cmd = Command::new(adb_bin);
        cmd.env("ANDROID_HOME", android_home);

        Ok(cmd)
    }

    /// Get the root toolchain directory
    ///
    /// Useful for debugging and toolchain management commands
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Get system image package name based on target SDK
    ///
    /// # Arguments
    /// * `target_sdk` - Target SDK version (e.g., 34, 35)
    ///
    /// Returns the system image package name to install via sdkmanager
    pub fn get_system_image_package(target_sdk: u32) -> Result<String> {
        let platform = Platform::detect()?;

        // Determine architecture
        let arch = match platform {
            Platform::LinuxX64 | Platform::MacX64 => "x86_64",
            Platform::MacAarch64 => "arm64-v8a", // Apple Silicon
            Platform::LinuxAarch64 => "arm64-v8a", // Linux ARM64
        };

        // Use google_apis_playstore for most common use case
        // Format: system-images;android-{SDK};{variant};{arch}
        Ok(format!(
            "system-images;android-{};google_apis_playstore;{}",
            target_sdk, arch
        ))
    }

    /// Ensure Android NDK is installed
    ///
    /// Downloads and installs NDK if not present. Required for native C++/Rust compilation.
    ///
    /// # Returns
    /// Path to NDK installation directory
    pub fn ensure_ndk(&self) -> Result<PathBuf> {
        let sdk_root = self.ensure_android_sdk()?;
        let ndk_version = DEFAULT_NDK;
        let ndk_path = sdk_root.join("ndk").join(ndk_version);

        // Check if already installed
        if ndk_path.exists() && ndk_path.join("build/cmake/android.toolchain.cmake").exists() {
            return Ok(ndk_path);
        }

        println!("Installing Android NDK {} (required for native compilation, ~1GB download)...", ndk_version);

        // Install using sdkmanager
        let sdkmanager = sdk_root.join("cmdline-tools/latest/bin/sdkmanager");
        let package = format!("ndk;{}", ndk_version);

        let output = Command::new(&sdkmanager)
            .arg(format!("--sdk_root={}", sdk_root.display()))
            .arg(&package)
            .env("ANDROID_HOME", &sdk_root)
            .stdin(std::process::Stdio::null())
            .output()
            .with_context(|| format!("Failed to install NDK: {}", package))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!(
                "Failed to install NDK {}:\n{}",
                package,
                stderr
            );
        }

        println!("Android NDK installed successfully");
        Ok(ndk_path)
    }

    /// Ensure CMake is installed
    ///
    /// Downloads and installs CMake if not present. Required for building native C++ code.
    ///
    /// # Returns
    /// Path to CMake bin directory
    pub fn ensure_cmake(&self) -> Result<PathBuf> {
        let cmake_version = DEFAULT_CMAKE;
        let cmake_home = self.root.join(format!("cmake/{}", cmake_version));

        if !cmake_home.exists() {
            self.download_cmake(cmake_version)?;
        }

        let cmake_bin = cmake_home.join("bin/cmake");

        if !cmake_bin.exists() {
            anyhow::bail!(
                "CMake {} installation corrupt: binary not found at {}",
                cmake_version,
                cmake_bin.display()
            );
        }

        Ok(cmake_bin)
    }

    /// Download and install CMake
    fn download_cmake(&self, version: &str) -> Result<()> {
        let platform = Platform::detect()?;
        let url = downloader::get_cmake_download_url(version, platform)?;

        let cmake_dir = self.root.join("cmake");
        fs::create_dir_all(&cmake_dir)?;

        // Download to temporary file
        let archive_path = cmake_dir.join(format!("cmake-{}.tar.gz", version));
        downloader::download_with_retry(&url, &archive_path)?;

        // Extract
        downloader::extract_tar_gz(&archive_path, &cmake_dir)?;

        // CMake extracts to cmake-VERSION-OS-ARCH/ directory
        // Find the extracted directory
        let extracted_dirs: Vec<_> = fs::read_dir(&cmake_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter(|e| {
                let name = e.file_name();
                let name_str = name.to_str().unwrap();
                name_str.starts_with(&format!("cmake-{}", version))
            })
            .collect();

        if let Some(extracted_dir) = extracted_dirs.first() {
            let target = cmake_dir.join(version);
            fs::rename(extracted_dir.path(), &target)?;
        }

        // Clean up archive
        fs::remove_file(&archive_path)?;

        Ok(())
    }

    /// Get configured cmake Command
    ///
    /// Returns a Command with proper NDK toolchain configuration
    ///
    /// # Arguments
    /// * `ndk_path` - Path to Android NDK
    /// * `android_abi` - Target Android ABI (e.g., "arm64-v8a", "x86_64")
    /// * `android_platform` - Minimum Android API level (e.g., 24)
    pub fn cmake_cmd(&self, ndk_path: &Path, android_abi: &str, android_platform: u32) -> Result<Command> {
        let cmake_bin = self.ensure_cmake()?;

        let mut cmd = Command::new(cmake_bin);

        // Set Android NDK toolchain file
        let toolchain_file = ndk_path.join("build/cmake/android.toolchain.cmake");
        if !toolchain_file.exists() {
            anyhow::bail!(
                "NDK toolchain file not found at {}",
                toolchain_file.display()
            );
        }

        cmd.arg(format!("-DCMAKE_TOOLCHAIN_FILE={}", toolchain_file.display()));
        cmd.arg(format!("-DANDROID_ABI={}", android_abi));
        cmd.arg(format!("-DANDROID_PLATFORM=android-{}", android_platform));

        Ok(cmd)
    }

    /// Ensure system image is installed for the given target SDK
    ///
    /// Downloads and installs system image if not present
    ///
    /// # Arguments
    /// * `target_sdk` - Target SDK version (e.g., 34, 35)
    pub fn ensure_system_image(&self, target_sdk: u32) -> Result<()> {
        let package = Self::get_system_image_package(target_sdk)?;
        let sdk_root = self.ensure_android_sdk()?;

        // Parse the package name to get the directory path
        // system-images;android-35;google_apis_playstore;x86_64
        // -> system-images/android-35/google_apis_playstore/x86_64/
        let parts: Vec<&str> = package.split(';').collect();
        if parts.len() != 4 || parts[0] != "system-images" {
            anyhow::bail!("Invalid system image package format: {}", package);
        }

        let system_image_path = sdk_root
            .join("system-images")
            .join(parts[1])
            .join(parts[2])
            .join(parts[3]);

        // Check if already installed
        if system_image_path.exists() && system_image_path.join("system.img").exists() {
            return Ok(()); // Already installed
        }

        // Create styled progress bar matching other components (40 chars wide like java/gradle/emulator)
        use colored::Colorize;
        use indicatif::{ProgressBar, ProgressStyle};

        let pb = ProgressBar::new(100);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(&format!("{{msg:20}} {}{{bar:40.dim}}{} \x1b[2mdownloading (~1GB)\x1b[0m", "[".dimmed(), "]".dimmed()))
                .unwrap()
                .progress_chars("=> "),
        );
        pb.set_message(format!("{}", format!("system-image-{}", target_sdk).dimmed()));

        // Start a thread to slowly increment progress (fake progress since sdkmanager doesn't provide it)
        use std::sync::atomic::{AtomicBool, Ordering};
        let pb_clone = pb.clone();
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = Arc::clone(&done);

        let progress_thread = std::thread::spawn(move || {
            let mut i = 0u64;
            // Slowly increment progress, but stop if download completes
            while i < 95 && !done_clone.load(Ordering::Relaxed) {
                pb_clone.set_position(i);
                std::thread::sleep(std::time::Duration::from_millis(600));
                i += 1;
            }
        });

        // Install using sdkmanager with yes piped for license acceptance
        let sdkmanager = sdk_root.join("cmdline-tools/latest/bin/sdkmanager");

        let status = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "yes 2>/dev/null | {} --sdk_root={} '{}' > /dev/null 2>&1",
                sdkmanager.display(),
                sdk_root.display(),
                package
            ))
            .env("ANDROID_HOME", &sdk_root)
            .status()
            .with_context(|| format!("Failed to install system image: {}", package))?;

        // Signal thread to stop and wait for it
        done.store(true, Ordering::Relaxed);
        let _ = progress_thread.join();

        // Complete the progress bar
        pb.set_position(100);
        pb.finish_and_clear();

        if !status.success() {
            anyhow::bail!(
                "Failed to install system image {}",
                package
            );
        }

        Ok(())
    }

    /// Ensure all toolchains in parallel for faster installation
    ///
    /// Downloads Java, Gradle, and Android SDK simultaneously
    ///
    /// # Arguments
    /// * `java_version` - Java version to ensure
    /// * `gradle_version` - Gradle version to ensure
    pub fn ensure_all_parallel(&self, java_version: &str, gradle_version: &str) -> Result<(PathBuf, PathBuf, PathBuf)> {
        use indicatif::MultiProgress;
        use std::sync::{mpsc, Arc, Mutex};
        use std::thread;

        let root = self.root.clone();
        let java_ver = java_version.to_string();
        let gradle_ver = gradle_version.to_string();

        // Shared results wrapped in Arc<Mutex>
        let results: Arc<Mutex<Vec<Result<String>>>> = Arc::new(Mutex::new(Vec::new()));

        // Create MultiProgress for coordinated display
        let multi = Arc::new(MultiProgress::new());

        // Create all 3 progress bars upfront so they all show immediately
        use colored::Colorize;
        use indicatif::{ProgressBar, ProgressStyle};

        let pb_java = multi.add(ProgressBar::new(100));
        pb_java.set_style(
            ProgressStyle::default_bar()
                .template(&format!("{{msg:20}} {}{{bar:40.dim}}{} \x1b[2m{{bytes:>10}}/{{total_bytes:>10}}\x1b[0m", "[".dimmed(), "]".dimmed()))
                .unwrap()
                .progress_chars("=> "),
        );
        pb_java.set_message(format!("{}", format!("java-{}", java_ver).dimmed()));

        let pb_gradle = multi.add(ProgressBar::new(100));
        pb_gradle.set_style(
            ProgressStyle::default_bar()
                .template(&format!("{{msg:20}} {}{{bar:40.dim}}{} \x1b[2m{{bytes:>10}}/{{total_bytes:>10}}\x1b[0m", "[".dimmed(), "]".dimmed()))
                .unwrap()
                .progress_chars("=> "),
        );
        pb_gradle.set_message(format!("{}", format!("gradle-{}", gradle_ver).dimmed()));

        let pb_android = multi.add(ProgressBar::new(100));
        pb_android.set_style(
            ProgressStyle::default_bar()
                .template(&format!("{{msg:20}} {}{{bar:40.dim}}{} \x1b[2m{{bytes:>10}}/{{total_bytes:>10}}\x1b[0m", "[".dimmed(), "]".dimmed()))
                .unwrap()
                .progress_chars("=> "),
        );
        pb_android.set_message(format!("{}", "android-sdk".dimmed()));

        // Wrap progress bars in Arc for sharing
        let pb_java = Arc::new(pb_java);
        let pb_gradle = Arc::new(pb_gradle);
        let pb_android = Arc::new(pb_android);

        // Create a semaphore-like channel with 2 slots (2 concurrent downloads max)
        let (permit_tx, permit_rx) = mpsc::sync_channel::<()>(2);

        // Fill the semaphore with 2 permits
        permit_tx.send(()).unwrap();
        permit_tx.send(()).unwrap();

        // Wrap receiver in Arc<Mutex> for sharing across threads
        let permit_rx = Arc::new(Mutex::new(permit_rx));

        let mut handles = vec![];

        // Thread 1: Java download
        {
            let java_ver = java_ver.clone();
            let root = root.clone();
            let results = Arc::clone(&results);
            let pb = Arc::clone(&pb_java);
            let permit_rx = Arc::clone(&permit_rx);
            let permit_tx = permit_tx.clone();

            let handle = thread::spawn(move || {
                let _permit = permit_rx.lock().unwrap().recv().unwrap(); // Acquire permit
                let result = Self::download_java_with_pb(&root, &java_ver, pb);
                let mut res_lock = results.lock().unwrap();
                res_lock.push(result.map(|_| format!("java-{}", java_ver)));
                drop(res_lock);
                permit_tx.send(()).unwrap(); // Release permit
            });
            handles.push(handle);
        }

        // Thread 2: Gradle download
        {
            let gradle_ver = gradle_ver.clone();
            let root = root.clone();
            let results = Arc::clone(&results);
            let pb = Arc::clone(&pb_gradle);
            let permit_rx = Arc::clone(&permit_rx);
            let permit_tx = permit_tx.clone();

            let handle = thread::spawn(move || {
                let _permit = permit_rx.lock().unwrap().recv().unwrap(); // Acquire permit
                let result = Self::download_gradle_with_pb(&root, &gradle_ver, pb);
                let mut res_lock = results.lock().unwrap();
                res_lock.push(result.map(|_| format!("gradle-{}", gradle_ver)));
                drop(res_lock);
                permit_tx.send(()).unwrap(); // Release permit
            });
            handles.push(handle);
        }

        // Thread 3: Android SDK download
        {
            let root = root.clone();
            let results = Arc::clone(&results);
            let pb = Arc::clone(&pb_android);
            let permit_rx = Arc::clone(&permit_rx);
            let permit_tx = permit_tx.clone();

            let handle = thread::spawn(move || {
                let _permit = permit_rx.lock().unwrap().recv().unwrap(); // Acquire permit (will wait for slot)
                let result = Self::download_android_sdk_with_pb(&root, pb);
                let mut res_lock = results.lock().unwrap();
                res_lock.push(result.map(|_| "android-sdk".to_string()));
                drop(res_lock);
                permit_tx.send(()).unwrap(); // Release permit
            });
            handles.push(handle);
        }

        // Wait for all downloads to complete
        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        // Clear the MultiProgress display
        multi.clear().ok();

        // Check results
        let results_vec = results.lock().unwrap();
        let mut errors = Vec::new();

        for result in results_vec.iter() {
            if let Err(e) = result {
                errors.push(format!("{}", e));
            }
        }

        if !errors.is_empty() {
            anyhow::bail!("Some downloads failed:\n  {}", errors.join("\n  "));
        }

        // Return paths to installed components
        Ok((
            self.ensure_java(&java_ver)?,
            self.ensure_gradle(&gradle_ver)?,
            self.ensure_android_sdk()?,
        ))
    }

    // Static helper functions for parallel downloads with pre-created progress bars

    fn download_java_with_pb(root: &Path, version: &str, pb: Arc<indicatif::ProgressBar>) -> Result<()> {
        use colored::Colorize;
        use indicatif::ProgressStyle;

        let java_home = root.join(format!("java/{}", version));

        if java_home.exists() {
            pb.finish_and_clear();
            return Ok(()); // Already installed
        }

        let platform = Platform::detect()?;
        let url = downloader::get_java_download_url(version, platform)?;

        let java_dir = root.join("java");
        std::fs::create_dir_all(&java_dir)?;

        let archive_path = java_dir.join(format!("java-{}.tar.gz", version));

        // Phase 1: Download (0-100% of file size)
        downloader::download_with_retry_and_bar(&url, &archive_path, Some(&pb))?;

        // Phase 2: Extract (reset bar, show "extracting")
        pb.set_style(
            ProgressStyle::default_bar()
                .template(&format!("{{msg:20}} {}{{bar:40.dim}}{} \x1b[2mextracting\x1b[0m", "[".dimmed(), "]".dimmed()))
                .unwrap()
                .progress_chars("=> "),
        );
        pb.set_length(100);
        pb.set_position(0);

        downloader::extract_tar_gz(&archive_path, &java_dir)?;
        pb.set_position(100);

        // Rename extracted directory
        let extracted_dirs: Vec<_> = std::fs::read_dir(&java_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter(|e| e.file_name().to_str().unwrap().starts_with("jdk-"))
            .collect();

        if let Some(extracted_dir) = extracted_dirs.first() {
            let target = java_dir.join(version);
            std::fs::rename(extracted_dir.path(), &target)?;
        }

        // Clean up archive
        std::fs::remove_file(&archive_path)?;

        pb.finish_and_clear();
        Ok(())
    }

    fn download_gradle_with_pb(root: &Path, version: &str, pb: Arc<indicatif::ProgressBar>) -> Result<()> {
        use colored::Colorize;
        use indicatif::ProgressStyle;

        let gradle_home = root.join(format!("gradle/{}", version));

        if gradle_home.exists() {
            pb.finish_and_clear();
            return Ok(()); // Already installed
        }

        let url = downloader::get_gradle_download_url(version);

        let gradle_dir = root.join("gradle");
        std::fs::create_dir_all(&gradle_dir)?;

        let archive_path = gradle_dir.join(format!("gradle-{}.zip", version));

        // Phase 1: Download (0-100% of file size)
        downloader::download_with_retry_and_bar(&url, &archive_path, Some(&pb))?;

        // Phase 2: Extract (reset bar, show "extracting")
        pb.set_style(
            ProgressStyle::default_bar()
                .template(&format!("{{msg:20}} {}{{bar:40.dim}}{} \x1b[2mextracting\x1b[0m", "[".dimmed(), "]".dimmed()))
                .unwrap()
                .progress_chars("=> "),
        );
        pb.set_length(100);
        pb.set_position(0);

        downloader::extract_zip(&archive_path, &gradle_dir)?;
        pb.set_position(100);

        // Rename extracted directory
        let extracted = gradle_dir.join(format!("gradle-{}", version));
        let target = gradle_dir.join(version);
        if extracted.exists() {
            std::fs::rename(&extracted, &target)?;
        }

        // Clean up archive
        std::fs::remove_file(&archive_path)?;

        pb.finish_and_clear();
        Ok(())
    }

    fn download_android_sdk_with_pb(root: &Path, pb: Arc<indicatif::ProgressBar>) -> Result<()> {
        use colored::Colorize;
        use indicatif::ProgressStyle;

        let sdk_root = root.join("android");

        if sdk_root.join("platform-tools/adb").exists() {
            pb.finish_and_clear();
            return Ok(()); // Already installed
        }

        let platform = Platform::detect()?;
        let url = downloader::get_android_cmdline_tools_url(platform)?;

        std::fs::create_dir_all(&sdk_root)?;

        let archive_path = sdk_root.join("cmdline-tools.zip");

        // Phase 1: Download cmdline-tools (0-100% of file size)
        downloader::download_with_retry_and_bar(&url, &archive_path, Some(&pb))?;

        // Phase 2: Extract (reset bar to 0-100%)
        pb.set_style(
            ProgressStyle::default_bar()
                .template(&format!("{{msg:20}} {}{{bar:40.dim}}{} extracting", "[".dimmed(), "]".dimmed()))
                .unwrap()
                .progress_chars("=> "),
        );
        pb.set_length(100);
        pb.set_position(0);

        let cmdline_tools_dir = sdk_root.join("cmdline-tools");
        std::fs::create_dir_all(&cmdline_tools_dir)?;

        downloader::extract_zip(&archive_path, &cmdline_tools_dir)?;

        // Rename to latest
        let extracted = cmdline_tools_dir.join("cmdline-tools");
        let latest = cmdline_tools_dir.join("latest");
        if extracted.exists() {
            std::fs::rename(&extracted, &latest)?;
        }

        std::fs::remove_file(&archive_path)?;
        pb.set_position(100);

        // Phase 3: Install SDK components (reset bar to 0-100%)
        pb.set_style(
            ProgressStyle::default_bar()
                .template(&format!("{{msg:20}} {}{{bar:40.dim}}{} \x1b[2minstalling\x1b[0m", "[".dimmed(), "]".dimmed()))
                .unwrap()
                .progress_chars("=> "),
        );
        pb.set_length(100);
        pb.set_position(0);

        // Accept licenses
        use std::process::Command;
        let sdkmanager = sdk_root.join("cmdline-tools/latest/bin/sdkmanager");
        let status = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "yes | {} --sdk_root={} --licenses 2>&1 > /dev/null",
                sdkmanager.display(),
                sdk_root.display()
            ))
            .env("ANDROID_HOME", &sdk_root)
            .status()
            .context("Failed to accept SDK licenses")?;

        if !status.success() {
            anyhow::bail!("Failed to accept SDK licenses");
        }

        pb.set_position(10); // License acceptance done

        // Install essential components
        let components = vec![
            "platform-tools",
            "build-tools;34.0.0",
            "platforms;android-34",
            "emulator",
        ];

        let progress_per_component = 90 / components.len() as u64; // 90% total for all components

        for (i, component) in components.iter().enumerate() {
            let output = Command::new(&sdkmanager)
                .arg(format!("--sdk_root={}", sdk_root.display()))
                .arg(component)
                .env("ANDROID_HOME", &sdk_root)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .output()
                .with_context(|| format!("Failed to install {}", component))?;

            if !output.status.success() {
                anyhow::bail!("Failed to install {}", component);
            }

            // Update progress: 10% for licenses + proportional progress for each component
            pb.set_position(10 + ((i + 1) as u64 * progress_per_component));
        }

        pb.finish_and_clear();
        Ok(())
    }

    #[allow(dead_code)]
    fn download_java_static(root: &Path, version: &str, multi: Option<Arc<indicatif::MultiProgress>>) -> Result<()> {
        use colored::Colorize;
        use indicatif::{ProgressBar, ProgressStyle};

        let java_home = root.join(format!("java/{}", version));

        if java_home.exists() {
            return Ok(()); // Already installed
        }

        let platform = Platform::detect()?;
        let url = downloader::get_java_download_url(version, platform)?;

        let java_dir = root.join("java");
        std::fs::create_dir_all(&java_dir)?;

        let archive_path = java_dir.join(format!("java-{}.tar.gz", version));

        if let Some(ref m) = multi {
            // Create ONE progress bar that resets between phases
            let pb = m.add(ProgressBar::new(100));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{msg:20} [{bar:40.dim}] {bytes:>10}/{total_bytes:>10}")
                    .unwrap()
                    .progress_chars("=> "),
            );
            pb.set_message(format!("{}", format!("java-{}", version).dimmed()));

            // Phase 1: Download (0-100% of file size)
            downloader::download_with_retry_and_bar(&url, &archive_path, Some(&pb))?;

            // Phase 2: Extract (reset bar, show "extracting")
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{msg:20} [{bar:40.dim}] extracting")
                    .unwrap()
                    .progress_chars("=> "),
            );
            pb.set_length(100);
            pb.set_position(0);
            pb.set_message(format!("{}", format!("java-{}", version).dimmed()));

            downloader::extract_tar_gz(&archive_path, &java_dir)?;
            pb.set_position(100);

            pb.finish_and_clear();
        } else {
            downloader::download_with_retry(&url, &archive_path)?;
            downloader::extract_tar_gz(&archive_path, &java_dir)?;
        }

        // Rename extracted directory
        let extracted_dirs: Vec<_> = std::fs::read_dir(&java_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter(|e| e.file_name().to_str().unwrap().starts_with("jdk-"))
            .collect();

        if let Some(extracted_dir) = extracted_dirs.first() {
            let target = java_dir.join(version);
            std::fs::rename(extracted_dir.path(), &target)?;
        }

        std::fs::remove_file(&archive_path)?;

        Ok(())
    }

    #[allow(dead_code)]
    fn download_gradle_static(root: &Path, version: &str, multi: Option<Arc<indicatif::MultiProgress>>) -> Result<()> {
        use colored::Colorize;
        use indicatif::{ProgressBar, ProgressStyle};

        let gradle_home = root.join(format!("gradle/{}", version));

        if gradle_home.exists() {
            return Ok(()); // Already installed
        }

        let url = downloader::get_gradle_download_url(version);

        let gradle_dir = root.join("gradle");
        std::fs::create_dir_all(&gradle_dir)?;

        let archive_path = gradle_dir.join(format!("gradle-{}.zip", version));

        if let Some(ref m) = multi {
            // Create ONE progress bar that resets between phases
            let pb = m.add(ProgressBar::new(100));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{msg:20} [{bar:40.dim}] {bytes:>10}/{total_bytes:>10}")
                    .unwrap()
                    .progress_chars("=> "),
            );
            pb.set_message(format!("{}", format!("gradle-{}", version).dimmed()));

            // Phase 1: Download (0-100% of file size)
            downloader::download_with_retry_and_bar(&url, &archive_path, Some(&pb))?;

            // Phase 2: Extract (reset bar, show "extracting")
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{msg:20} [{bar:40.dim}] extracting")
                    .unwrap()
                    .progress_chars("=> "),
            );
            pb.set_length(100);
            pb.set_position(0);
            pb.set_message(format!("{}", format!("gradle-{}", version).dimmed()));

            downloader::extract_zip(&archive_path, &gradle_dir)?;
            pb.set_position(100);

            pb.finish_and_clear();
        } else {
            downloader::download_with_retry(&url, &archive_path)?;
            downloader::extract_zip(&archive_path, &gradle_dir)?;
        }

        // Rename extracted directory
        let extracted = gradle_dir.join(format!("gradle-{}", version));
        let target = gradle_dir.join(version);
        if extracted.exists() {
            std::fs::rename(&extracted, &target)?;
        }

        std::fs::remove_file(&archive_path)?;

        Ok(())
    }

    #[allow(dead_code)]
    fn download_android_sdk_static(root: &Path, multi: Option<Arc<indicatif::MultiProgress>>) -> Result<()> {
        use colored::Colorize;
        use indicatif::{ProgressBar, ProgressStyle};

        let sdk_root = root.join("android");

        if sdk_root.join("platform-tools/adb").exists() {
            return Ok(()); // Already installed
        }

        let platform = Platform::detect()?;
        let url = downloader::get_android_cmdline_tools_url(platform)?;

        std::fs::create_dir_all(&sdk_root)?;

        let archive_path = sdk_root.join("cmdline-tools.zip");

        if let Some(ref m) = multi {
            // Create ONE progress bar that resets between phases
            let pb = m.add(ProgressBar::new(100));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{msg:20} [{bar:40.dim}] {bytes:>10}/{total_bytes:>10}")
                    .unwrap()
                    .progress_chars("=> "),
            );
            pb.set_message(format!("{}", "android-sdk".dimmed()));

            // Phase 1: Download cmdline-tools (0-100% of file size)
            downloader::download_with_retry_and_bar(&url, &archive_path, Some(&pb))?;

            // Phase 2: Extract (reset bar to 0-100%)
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{msg:20} [{bar:40.dim}] extracting")
                    .unwrap()
                    .progress_chars("=> "),
            );
            pb.set_length(100);
            pb.set_position(0);

            let cmdline_tools_dir = sdk_root.join("cmdline-tools");
            std::fs::create_dir_all(&cmdline_tools_dir)?;

            downloader::extract_zip(&archive_path, &cmdline_tools_dir)?;

            // Rename to latest
            let extracted = cmdline_tools_dir.join("cmdline-tools");
            let latest = cmdline_tools_dir.join("latest");
            if extracted.exists() {
                std::fs::rename(&extracted, &latest)?;
            }

            std::fs::remove_file(&archive_path)?;
            pb.set_position(100);

            // Phase 3: Install SDK components (reset bar to 0-100%)
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{msg:20} [{bar:40.dim}] installing")
                    .unwrap()
                    .progress_chars("=> "),
            );
            pb.set_length(100);
            pb.set_position(0);

            // Accept licenses
            use std::process::Command;
            let sdkmanager = sdk_root.join("cmdline-tools/latest/bin/sdkmanager");
            let status = Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "yes | {} --sdk_root={} --licenses 2>&1 > /dev/null",
                    sdkmanager.display(),
                    sdk_root.display()
                ))
                .env("ANDROID_HOME", &sdk_root)
                .status()
                .context("Failed to accept SDK licenses")?;

            if !status.success() {
                anyhow::bail!("Failed to accept SDK licenses");
            }

            pb.set_position(10); // License acceptance done

            // Install essential components
            let components = vec![
                "platform-tools",
                "build-tools;34.0.0",
                "platforms;android-34",
                "emulator",
            ];

            let progress_per_component = 90 / components.len() as u64; // 90% total for all components

            for (i, component) in components.iter().enumerate() {
                let output = Command::new(&sdkmanager)
                    .arg(format!("--sdk_root={}", sdk_root.display()))
                    .arg(component)
                    .env("ANDROID_HOME", &sdk_root)
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .output()
                    .with_context(|| format!("Failed to install {}", component))?;

                if !output.status.success() {
                    anyhow::bail!("Failed to install {}", component);
                }

                // Update progress: 10% for licenses + proportional progress for each component
                pb.set_position(10 + ((i + 1) as u64 * progress_per_component));
            }

            pb.finish_and_clear();
        } else {
            downloader::download_with_retry(&url, &archive_path)?;

            let cmdline_tools_dir = sdk_root.join("cmdline-tools");
            std::fs::create_dir_all(&cmdline_tools_dir)?;

            downloader::extract_zip(&archive_path, &cmdline_tools_dir)?;

            // Rename to latest
            let extracted = cmdline_tools_dir.join("cmdline-tools");
            let latest = cmdline_tools_dir.join("latest");
            if extracted.exists() {
                std::fs::rename(&extracted, &latest)?;
            }

            std::fs::remove_file(&archive_path)?;

            // Install SDK components using sdkmanager
            let sdkmanager = sdk_root.join("cmdline-tools/latest/bin/sdkmanager");

            // Accept licenses
            use std::process::Command;
            let status = Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "yes | {} --sdk_root={} --licenses 2>&1 > /dev/null",
                    sdkmanager.display(),
                    sdk_root.display()
                ))
                .env("ANDROID_HOME", &sdk_root)
                .status()
                .context("Failed to accept SDK licenses")?;

            if !status.success() {
                anyhow::bail!("Failed to accept SDK licenses");
            }

            // Install essential components
            let components = vec![
                "platform-tools",
                "build-tools;34.0.0",
                "platforms;android-34",
                "emulator",
            ];

            for component in components {
                let output = Command::new(&sdkmanager)
                    .arg(format!("--sdk_root={}", sdk_root.display()))
                    .arg(component)
                    .env("ANDROID_HOME", &sdk_root)
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .output()
                    .with_context(|| format!("Failed to install {}", component))?;

                if !output.status.success() {
                    anyhow::bail!("Failed to install {}", component);
                }
            }
        }

        Ok(())
    }
}

impl Default for Toolchain {
    fn default() -> Self {
        Self::new().expect("Failed to initialize toolchain")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolchain_root_creation() {
        let toolchain = Toolchain::new().unwrap();
        let root = toolchain.root();

        // Root should be ~/.whitehall/toolchains
        assert!(root.ends_with(".whitehall/toolchains"));

        // Directory should exist
        assert!(root.exists());
    }

    // Phase 1 tests removed - Phase 2 now downloads automatically
    // To test downloads, run: cargo run --example test-toolchain
}
