mod defaults;
mod downloader;
mod platform;
pub mod validator;

pub use defaults::*;
pub use platform::Platform;
pub use validator::validate_compatibility;

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

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
        downloader::download_with_progress(&url, &archive_path)?;

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

        println!("✓ Java {} installed", version);
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
        downloader::download_with_progress(&url, &archive_path)?;

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

        println!("✓ Gradle {} installed", version);
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
        downloader::download_with_progress(&url, &archive_path)?;

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

        println!("✓ Android cmdline-tools installed");

        // Now use sdkmanager to install essential components
        self.install_sdk_components()?;

        println!("✓ Android SDK installed");
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
        println!("Accepting Android SDK licenses...");
        let status = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "yes | {} --sdk_root={} --licenses",
                sdkmanager.display(),
                sdk_root.display()
            ))
            .env("ANDROID_HOME", &sdk_root)
            .status()
            .context("Failed to accept SDK licenses")?;

        if !status.success() {
            anyhow::bail!("Failed to accept SDK licenses");
        }

        println!("Installing Android SDK components...");

        // Install essential components
        let components = vec![
            "platform-tools",        // adb, fastboot
            "build-tools;34.0.0",    // aapt, dx, etc.
            "platforms;android-34",  // Android 14 platform
        ];

        for component in components {
            println!("  Installing {}...", component);
            let output = Command::new(&sdkmanager)
                .arg(format!("--sdk_root={}", sdk_root.display()))
                .arg(component)
                .env("ANDROID_HOME", &sdk_root)
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .output()
                .with_context(|| format!("Failed to run sdkmanager for {}", component))?;

            if !output.status.success() {
                anyhow::bail!("Failed to install {}", component);
            }
        }

        println!("✓ SDK components installed");
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
