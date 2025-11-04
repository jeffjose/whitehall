mod defaults;
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
    /// Returns the JAVA_HOME path if toolchain exists
    /// Returns error if toolchain is not installed (Phase 2 will download)
    ///
    /// # Arguments
    /// * `version` - Java version (e.g., "11", "17", "21")
    pub fn ensure_java(&self, version: &str) -> Result<PathBuf> {
        let java_home = self.root.join(format!("java/{}", version));

        if !java_home.exists() {
            anyhow::bail!(
                "Java {} not installed. Download will be implemented in Phase 2.\n\
                Expected location: {}",
                version,
                java_home.display()
            );
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

    /// Ensure Gradle is installed for the given version
    ///
    /// Returns the path to the gradle executable
    /// Returns error if toolchain is not installed (Phase 2 will download)
    ///
    /// # Arguments
    /// * `version` - Gradle version (e.g., "7.6", "8.0", "8.4")
    pub fn ensure_gradle(&self, version: &str) -> Result<PathBuf> {
        let gradle_home = self.root.join(format!("gradle/{}", version));

        if !gradle_home.exists() {
            anyhow::bail!(
                "Gradle {} not installed. Download will be implemented in Phase 2.\n\
                Expected location: {}",
                version,
                gradle_home.display()
            );
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

    /// Ensure Android SDK is installed
    ///
    /// Returns the ANDROID_HOME path
    /// Returns error if SDK is not installed (Phase 2 will download)
    pub fn ensure_android_sdk(&self) -> Result<PathBuf> {
        let sdk_root = self.root.join("android");

        if !sdk_root.exists() {
            anyhow::bail!(
                "Android SDK not installed. Download will be implemented in Phase 2.\n\
                Expected location: {}",
                sdk_root.display()
            );
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
        cmd.env("ANDROID_HOME", android_home);

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

    #[test]
    fn test_java_path_structure() {
        let toolchain = Toolchain::new().unwrap();

        // Should error when Java not installed
        let result = toolchain.ensure_java("21");
        assert!(result.is_err());

        let err = result.unwrap_err().to_string();
        assert!(err.contains("Java 21 not installed"));
    }

    #[test]
    fn test_gradle_path_structure() {
        let toolchain = Toolchain::new().unwrap();

        // Should error when Gradle not installed
        let result = toolchain.ensure_gradle("8.4");
        assert!(result.is_err());

        let err = result.unwrap_err().to_string();
        assert!(err.contains("Gradle 8.4 not installed"));
    }

    #[test]
    fn test_android_sdk_path_structure() {
        let toolchain = Toolchain::new().unwrap();

        // Should error when SDK not installed
        let result = toolchain.ensure_android_sdk();
        assert!(result.is_err());

        let err = result.unwrap_err().to_string();
        assert!(err.contains("Android SDK not installed"));
    }
}
