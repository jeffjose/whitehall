/// Supported platforms for toolchain downloads
///
/// Note: Windows is not currently supported
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    LinuxX64,
    LinuxAarch64,
    MacX64,
    MacAarch64,
}

impl Platform {
    /// Detect the current platform
    ///
    /// Uses std::env::consts to determine OS and architecture
    pub fn detect() -> anyhow::Result<Self> {
        use std::env::consts::{ARCH, OS};

        match (OS, ARCH) {
            ("linux", "x86_64") => Ok(Platform::LinuxX64),
            ("linux", "aarch64") => Ok(Platform::LinuxAarch64),
            ("macos", "x86_64") => Ok(Platform::MacX64),
            ("macos", "aarch64") => Ok(Platform::MacAarch64),
            (os, arch) => anyhow::bail!(
                "Unsupported platform: {}-{}\n\
                Whitehall currently supports: Linux (x64, aarch64), macOS (x64, Apple Silicon)",
                os,
                arch
            ),
        }
    }

    /// Get the platform identifier string for download URLs
    ///
    /// # Returns
    /// Tuple of (os, arch) strings suitable for URL construction
    ///
    /// # Example
    /// ```
    /// let (os, arch) = Platform::LinuxX64.as_download_strings();
    /// // os = "linux", arch = "x64"
    /// ```
    pub fn as_download_strings(&self) -> (&'static str, &'static str) {
        match self {
            Platform::LinuxX64 => ("linux", "x64"),
            Platform::LinuxAarch64 => ("linux", "aarch64"),
            Platform::MacX64 => ("mac", "x64"),
            Platform::MacAarch64 => ("mac", "aarch64"),
        }
    }

    /// Check if this is a Linux platform
    pub fn is_linux(&self) -> bool {
        matches!(self, Platform::LinuxX64 | Platform::LinuxAarch64)
    }

    /// Check if this is a macOS platform
    pub fn is_macos(&self) -> bool {
        matches!(self, Platform::MacX64 | Platform::MacAarch64)
    }

    /// Check if this is an ARM64/aarch64 platform
    pub fn is_aarch64(&self) -> bool {
        matches!(self, Platform::LinuxAarch64 | Platform::MacAarch64)
    }

    /// Get platform-specific Java archive extension
    pub fn java_archive_ext(&self) -> &'static str {
        if self.is_linux() {
            "tar.gz"
        } else {
            "tar.gz" // macOS also uses tar.gz
        }
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (os, arch) = self.as_download_strings();
        write!(f, "{}-{}", os, arch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        // Should successfully detect current platform
        let platform = Platform::detect();
        assert!(platform.is_ok());
    }

    #[test]
    fn test_platform_strings() {
        assert_eq!(Platform::LinuxX64.as_download_strings(), ("linux", "x64"));
        assert_eq!(
            Platform::LinuxAarch64.as_download_strings(),
            ("linux", "aarch64")
        );
        assert_eq!(Platform::MacX64.as_download_strings(), ("mac", "x64"));
        assert_eq!(
            Platform::MacAarch64.as_download_strings(),
            ("mac", "aarch64")
        );
    }

    #[test]
    fn test_platform_checks() {
        assert!(Platform::LinuxX64.is_linux());
        assert!(!Platform::LinuxX64.is_macos());
        assert!(!Platform::LinuxX64.is_aarch64());

        assert!(Platform::MacAarch64.is_macos());
        assert!(!Platform::MacAarch64.is_linux());
        assert!(Platform::MacAarch64.is_aarch64());
    }

    #[test]
    fn test_platform_display() {
        assert_eq!(Platform::LinuxX64.to_string(), "linux-x64");
        assert_eq!(Platform::MacAarch64.to_string(), "mac-aarch64");
    }

    #[test]
    fn test_java_archive_ext() {
        // All supported platforms use tar.gz
        assert_eq!(Platform::LinuxX64.java_archive_ext(), "tar.gz");
        assert_eq!(Platform::MacAarch64.java_archive_ext(), "tar.gz");
    }
}
