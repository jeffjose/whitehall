pub mod init;
pub mod build;
pub mod watch;
pub mod run;
pub mod install;
pub mod compile;
pub mod toolchain;
pub mod emulator;
pub mod doctor;

use std::path::Path;

/// Represents the type of target we're working with
#[derive(Debug, Clone)]
pub enum Target {
    /// A Whitehall project directory (contains whitehall.toml)
    Project(String), // manifest path
    /// A single .wh file
    SingleFile(String), // file path
}

/// Detect whether the target is a project directory or a single file
pub fn detect_target(target: &str) -> Target {
    let path = Path::new(target);

    // Check if it's a .wh file
    if target.ends_with(".wh") {
        return Target::SingleFile(target.to_string());
    }

    // Check if it's a directory
    if path.is_dir() {
        // Look for whitehall.toml in the directory
        let manifest_path = path.join("whitehall.toml");
        if manifest_path.exists() {
            return Target::Project(manifest_path.to_str().unwrap().to_string());
        }
        // Default to whitehall.toml if directory exists
        return Target::Project("whitehall.toml".to_string());
    }

    // If target is "." or similar, assume project mode
    if target == "." || target == "./" {
        return Target::Project("whitehall.toml".to_string());
    }

    // Check if it looks like a manifest path
    if target.ends_with("whitehall.toml") {
        return Target::Project(target.to_string());
    }

    // Default to project mode
    Target::Project(target.to_string())
}
