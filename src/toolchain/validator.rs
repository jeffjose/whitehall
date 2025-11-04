use anyhow::{Context, Result};

/// Toolchain configuration from whitehall.toml
#[derive(Debug, Clone)]
pub struct ToolchainConfig {
    pub java: String,
    pub gradle: String,
    pub agp: String,
}

/// Validate that toolchain versions are compatible with each other
///
/// Checks:
/// - AGP version requires compatible Java version
/// - AGP version requires compatible Gradle version
///
/// # Version Compatibility Matrix
///
/// | AGP Version | Required Gradle | Required Java |
/// |-------------|-----------------|---------------|
/// | 7.4.x       | 7.5+            | 11+           |
/// | 8.0.x       | 8.0+            | 17+           |
/// | 8.1.x       | 8.0+            | 17+           |
/// | 8.2.x       | 8.2+            | 17+           |
/// | 8.3.x       | 8.4+            | 17+           |
/// | 8.4.x       | 8.6+            | 17+           |
/// | 9.0.x       | 8.6+            | 21+           |
///
pub fn validate_compatibility(config: &ToolchainConfig) -> Result<()> {
    // Parse versions
    let java_version = parse_java_version(&config.java)?;
    let gradle_version = parse_gradle_version(&config.gradle)?;
    let agp_version = parse_agp_version(&config.agp)?;

    // Validate AGP <-> Java compatibility
    validate_agp_java(agp_version, java_version, &config.java, &config.agp)?;

    // Validate AGP <-> Gradle compatibility
    validate_agp_gradle(agp_version, gradle_version, &config.gradle, &config.agp)?;

    Ok(())
}

/// Parse Java version (e.g., "21" -> 21)
fn parse_java_version(version: &str) -> Result<u32> {
    version
        .parse::<u32>()
        .with_context(|| format!("Invalid Java version: '{}'. Expected format: '11', '17', '21'", version))
}

/// Parse Gradle version (e.g., "8.4" -> (8, 4))
fn parse_gradle_version(version: &str) -> Result<(u32, u32)> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 2 {
        anyhow::bail!(
            "Invalid Gradle version: '{}'. Expected format: 'X.Y' (e.g., '8.4')",
            version
        );
    }

    let major = parts[0].parse::<u32>().with_context(|| {
        format!("Invalid Gradle major version: '{}'", parts[0])
    })?;

    let minor = parts[1].parse::<u32>().with_context(|| {
        format!("Invalid Gradle minor version: '{}'", parts[1])
    })?;

    Ok((major, minor))
}

/// Parse AGP version (e.g., "8.2.0" -> (8, 2, 0))
fn parse_agp_version(version: &str) -> Result<(u32, u32, u32)> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 3 {
        anyhow::bail!(
            "Invalid AGP version: '{}'. Expected format: 'X.Y.Z' (e.g., '8.2.0')",
            version
        );
    }

    let major = parts[0].parse::<u32>().with_context(|| {
        format!("Invalid AGP major version: '{}'", parts[0])
    })?;

    let minor = parts[1].parse::<u32>().with_context(|| {
        format!("Invalid AGP minor version: '{}'", parts[1])
    })?;

    let patch = parts[2].parse::<u32>().with_context(|| {
        format!("Invalid AGP patch version: '{}'", parts[2])
    })?;

    Ok((major, minor, patch))
}

/// Validate AGP and Java version compatibility
fn validate_agp_java(
    agp: (u32, u32, u32),
    java: u32,
    java_str: &str,
    agp_str: &str,
) -> Result<()> {
    let (agp_major, agp_minor, _) = agp;

    let required_java = match (agp_major, agp_minor) {
        // AGP 7.4.x requires Java 11+
        (7, 4) => 11,
        // AGP 8.0-8.x requires Java 17+
        (8, 0..=8) => 17,
        // AGP 9.0+ requires Java 21+
        (9, _) => 21,
        // Unknown AGP version - require Java 17 as safe default
        _ => 17,
    };

    if java < required_java {
        anyhow::bail!(
            "Incompatible toolchain configuration:\n\
            AGP {} requires Java {} or higher, but java = \"{}\" specified.\n\
            Suggestion: Update to java = \"{}\" or java = \"21\"",
            agp_str,
            required_java,
            java_str,
            required_java
        );
    }

    Ok(())
}

/// Validate AGP and Gradle version compatibility
fn validate_agp_gradle(
    agp: (u32, u32, u32),
    gradle: (u32, u32),
    gradle_str: &str,
    agp_str: &str,
) -> Result<()> {
    let (agp_major, agp_minor, _) = agp;
    let (gradle_major, gradle_minor) = gradle;

    let (required_major, required_minor) = match (agp_major, agp_minor) {
        // AGP 7.4.x requires Gradle 7.5+
        (7, 4) => (7, 5),
        // AGP 8.0-8.1 requires Gradle 8.0+
        (8, 0..=1) => (8, 0),
        // AGP 8.2 requires Gradle 8.2+
        (8, 2) => (8, 2),
        // AGP 8.3 requires Gradle 8.4+
        (8, 3) => (8, 4),
        // AGP 8.4+ requires Gradle 8.6+
        (8, 4..) => (8, 6),
        // AGP 9.0+ requires Gradle 8.6+
        (9, _) => (8, 6),
        // Unknown AGP version - require Gradle 8.0 as safe default
        _ => (8, 0),
    };

    // Check if Gradle version meets minimum requirement
    let gradle_sufficient = gradle_major > required_major
        || (gradle_major == required_major && gradle_minor >= required_minor);

    if !gradle_sufficient {
        anyhow::bail!(
            "Incompatible toolchain configuration:\n\
            AGP {} requires Gradle {}.{} or higher, but gradle = \"{}\" specified.\n\
            Suggestion: Update to gradle = \"{}.{}\" or gradle = \"8.6\"",
            agp_str,
            required_major,
            required_minor,
            gradle_str,
            required_major,
            required_minor
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_configuration() {
        // AGP 8.2.0 + Gradle 8.4 + Java 21 (all compatible)
        let config = ToolchainConfig {
            java: "21".to_string(),
            gradle: "8.4".to_string(),
            agp: "8.2.0".to_string(),
        };
        assert!(validate_compatibility(&config).is_ok());
    }

    #[test]
    fn test_invalid_java_version() {
        // AGP 8.2.0 requires Java 17+, but using Java 11
        let config = ToolchainConfig {
            java: "11".to_string(),
            gradle: "8.4".to_string(),
            agp: "8.2.0".to_string(),
        };
        let result = validate_compatibility(&config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("AGP 8.2.0 requires Java 17 or higher"));
    }

    #[test]
    fn test_invalid_gradle_version() {
        // AGP 8.2.0 requires Gradle 8.2+, but using Gradle 8.0
        let config = ToolchainConfig {
            java: "17".to_string(),
            gradle: "8.0".to_string(),
            agp: "8.2.0".to_string(),
        };
        let result = validate_compatibility(&config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("AGP 8.2.0 requires Gradle 8.2 or higher"));
    }

    #[test]
    fn test_agp_7_4_compatibility() {
        // AGP 7.4.x requires Gradle 7.5+ and Java 11+
        let config = ToolchainConfig {
            java: "11".to_string(),
            gradle: "7.6".to_string(),
            agp: "7.4.2".to_string(),
        };
        assert!(validate_compatibility(&config).is_ok());
    }

    #[test]
    fn test_agp_9_0_compatibility() {
        // AGP 9.0+ requires Gradle 8.6+ and Java 21+
        let config = ToolchainConfig {
            java: "21".to_string(),
            gradle: "8.6".to_string(),
            agp: "9.0.0".to_string(),
        };
        assert!(validate_compatibility(&config).is_ok());
    }

    #[test]
    fn test_parse_version_errors() {
        let config = ToolchainConfig {
            java: "invalid".to_string(),
            gradle: "8.4".to_string(),
            agp: "8.2.0".to_string(),
        };
        assert!(validate_compatibility(&config).is_err());

        let config = ToolchainConfig {
            java: "21".to_string(),
            gradle: "invalid".to_string(),
            agp: "8.2.0".to_string(),
        };
        assert!(validate_compatibility(&config).is_err());

        let config = ToolchainConfig {
            java: "21".to_string(),
            gradle: "8.4".to_string(),
            agp: "invalid".to_string(),
        };
        assert!(validate_compatibility(&config).is_err());
    }
}
