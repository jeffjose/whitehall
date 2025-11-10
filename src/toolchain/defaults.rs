/// Default toolchain versions used when creating new projects
///
/// These defaults are updated with each Whitehall release to track
/// current stable versions. Existing projects are not affected - they
/// keep the versions specified in their whitehall.toml.

/// Default Java version (LTS)
///
/// Java 21 is the current LTS (Long Term Support) release, stable until 2029
pub const DEFAULT_JAVA: &str = "21";

/// Default Gradle version
///
/// Latest stable Gradle release
pub const DEFAULT_GRADLE: &str = "8.4";

/// Default Android Gradle Plugin (AGP) version
///
/// Latest stable AGP release
pub const DEFAULT_AGP: &str = "8.2.0";

/// Default Kotlin version
///
/// Note: This is the Kotlin Gradle plugin version used by AGP,
/// not the standalone Kotlin compiler
pub const DEFAULT_KOTLIN: &str = "2.0.0";

/// Default Compose Compiler Extension version
///
/// IMPORTANT: This must be compatible with DEFAULT_KOTLIN
/// Version compatibility:
/// - Kotlin 1.9.x → Compose Compiler 1.5.4
/// - Kotlin 2.0.0 → Compose Compiler 2.0.0
/// See: https://developer.android.com/jetpack/androidx/releases/compose-kotlin
pub const DEFAULT_COMPOSE_COMPILER: &str = "2.0.0";

/// Default minimum Android SDK version
pub const DEFAULT_MIN_SDK: u32 = 24;

/// Default target Android SDK version
pub const DEFAULT_TARGET_SDK: u32 = 34;

/// Default build tools version
pub const DEFAULT_BUILD_TOOLS: &str = "34.0.0";

/// Default Android NDK version
///
/// NDK 26 is the current stable release with good CMake/Rust support
pub const DEFAULT_NDK: &str = "26.1.10909125";

/// Default CMake version
///
/// CMake 3.28.1 is stable and works well with NDK 26
pub const DEFAULT_CMAKE: &str = "3.28.1";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults_are_valid() {
        // Java version should be numeric
        assert!(DEFAULT_JAVA.parse::<u32>().is_ok());

        // Gradle version should have format X.Y
        assert!(DEFAULT_GRADLE.contains('.'));

        // AGP version should have format X.Y.Z
        let agp_parts: Vec<&str> = DEFAULT_AGP.split('.').collect();
        assert_eq!(agp_parts.len(), 3);

        // Kotlin version should have format X.Y.Z
        let kotlin_parts: Vec<&str> = DEFAULT_KOTLIN.split('.').collect();
        assert_eq!(kotlin_parts.len(), 3);

        // SDK versions should be reasonable
        assert!(DEFAULT_MIN_SDK >= 21); // Android 5.0+
        assert!(DEFAULT_TARGET_SDK >= 33); // Android 13+
        assert!(DEFAULT_TARGET_SDK >= DEFAULT_MIN_SDK);
    }
}
