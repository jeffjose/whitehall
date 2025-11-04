use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::transpiler;

/// Compile a single .wh file to Kotlin
/// This is a lightweight operation that only transpiles - no Android project generation
pub fn execute(file_path: &str, package: Option<&str>, no_package: bool) -> Result<()> {
    // Validate file exists and has .wh extension
    let path = Path::new(file_path);
    if !path.exists() {
        anyhow::bail!("File not found: {}", file_path);
    }
    if !file_path.ends_with(".wh") {
        anyhow::bail!("File must have .wh extension: {}", file_path);
    }

    // Read source file
    let source = fs::read_to_string(path)
        .context(format!("Failed to read {}", file_path))?;

    // Strip frontmatter if present (for compile, we don't need it)
    let code = strip_frontmatter(&source);

    // Get component name from filename
    let component_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| {
            // Capitalize first letter for component name
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .unwrap_or_else(|| "Component".to_string());

    // Determine package name
    let package_name = package.unwrap_or("com.example.app");

    // Transpile to Kotlin
    let kotlin_code = transpiler::transpile(&code, package_name, &component_name, None)
        .map_err(|e| anyhow::anyhow!("Transpilation error: {}", e))?;

    // Output the Kotlin code
    if no_package {
        // Strip package declaration for pasting into existing files
        let code_without_package = kotlin_code
            .lines()
            .skip_while(|line| line.trim().is_empty() || line.starts_with("package "))
            .collect::<Vec<_>>()
            .join("\n");
        println!("{}", code_without_package);
    } else {
        println!("{}", kotlin_code);
    }

    Ok(())
}

/// Strip frontmatter (/// comments) from source code
fn strip_frontmatter(content: &str) -> String {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            // Skip shebang and frontmatter lines
            !trimmed.starts_with("#!") && !trimmed.starts_with("///")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_frontmatter() {
        let source = r#"#!/usr/bin/env whitehall
/// [app]
/// name = "Test"

var x = 5
<Text>Hello</Text>
"#;

        let result = strip_frontmatter(source);
        assert!(!result.contains("#!/"));
        assert!(!result.contains("///"));
        assert!(result.contains("var x = 5"));
        assert!(result.contains("<Text>Hello</Text>"));
    }
}
