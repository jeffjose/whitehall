/// Markdown-based transpiler tests
///
/// This test module parses markdown files from tests/transpiler-examples/
/// and validates transpilation output against expected Kotlin code.

use std::fs;
use std::path::PathBuf;

/// Test metadata extracted from ## Metadata section
#[derive(Debug)]
struct TestMetadata {
    file: String,
    package: String,
    type_hint: Option<String>,
}

/// Represents a parsed transpiler test case
#[derive(Debug)]
struct TranspilerTest {
    name: String,
    metadata: TestMetadata,
    input: String,
    expected_output: String,
}

/// Parse a markdown test file
fn parse_test_file(content: &str, filename: &str) -> Result<TranspilerTest, String> {
    let mut lines = content.lines().peekable();
    let mut name = String::new();
    let mut input = String::new();
    let mut expected_output = String::new();
    let mut metadata_content = String::new();

    let mut in_input_section = false;
    let mut in_output_section = false;
    let mut in_metadata_section = false;
    let mut in_code_block = false;

    // Extract test name from first heading
    while let Some(line) = lines.peek() {
        if line.starts_with("# ") {
            name = line[2..].trim().to_string();
            lines.next();
            break;
        }
        lines.next();
    }

    if name.is_empty() {
        return Err(format!("No title found in {}", filename));
    }

    // Parse sections
    for line in lines {
        if line.starts_with("## Input") {
            in_input_section = true;
            in_output_section = false;
            in_metadata_section = false;
            continue;
        } else if line.starts_with("## Output") {
            in_input_section = false;
            in_output_section = true;
            in_metadata_section = false;
            continue;
        } else if line.starts_with("## Metadata") {
            in_input_section = false;
            in_output_section = false;
            in_metadata_section = true;
            continue;
        } else if line.starts_with("## ") {
            // Other sections (like Notes) - skip
            in_input_section = false;
            in_output_section = false;
            in_metadata_section = false;
            continue;
        }

        // Handle code blocks
        if line.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        // Collect code block content
        if in_code_block {
            if in_input_section {
                input.push_str(line);
                input.push('\n');
            } else if in_output_section {
                expected_output.push_str(line);
                expected_output.push('\n');
            } else if in_metadata_section {
                metadata_content.push_str(line);
                metadata_content.push('\n');
            }
        }
    }

    if input.is_empty() {
        return Err(format!("No input section found in {}", filename));
    }

    if expected_output.is_empty() {
        return Err(format!("No output section found in {}", filename));
    }

    if metadata_content.is_empty() {
        return Err(format!("No metadata section found in {}", filename));
    }

    // Parse metadata key-value pairs
    let metadata = parse_metadata(&metadata_content, filename)?;

    Ok(TranspilerTest {
        name,
        metadata,
        input: input.trim().to_string(),
        expected_output: expected_output.trim().to_string(),
    })
}

/// Parse metadata content (simple key: value format)
fn parse_metadata(content: &str, filename: &str) -> Result<TestMetadata, String> {
    let mut file = None;
    let mut package = None;
    let mut type_hint = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() != 2 {
            continue;
        }

        let key = parts[0].trim();
        let value = parts[1].trim().to_string();

        match key {
            "file" => file = Some(value),
            "package" => package = Some(value),
            "type" => type_hint = Some(value),
            _ => {} // Ignore unknown keys
        }
    }

    let file = file.ok_or_else(|| format!("Missing 'file' in metadata for {}", filename))?;
    let package = package.ok_or_else(|| format!("Missing 'package' in metadata for {}", filename))?;

    Ok(TestMetadata {
        file,
        package,
        type_hint,
    })
}

/// Load all test files from the transpiler-examples directory
fn load_test_files() -> Vec<(String, String)> {
    let test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("transpiler-examples");

    let mut tests = Vec::new();

    if let Ok(entries) = fs::read_dir(&test_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    // Skip README
                    if filename == "README.md" {
                        continue;
                    }

                    if let Ok(content) = fs::read_to_string(&path) {
                        tests.push((filename.to_string(), content));
                    }
                }
            }
        }
    }

    // Sort for deterministic test order
    tests.sort_by(|a, b| a.0.cmp(&b.0));
    tests
}

/// Normalize whitespace for comparison (ignores minor formatting differences)
fn normalize_whitespace(s: &str) -> String {
    s.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_files() {
        let test_files = load_test_files();

        assert!(
            !test_files.is_empty(),
            "No test files found in tests/transpiler-examples/"
        );

        for (filename, content) in test_files {
            let result = parse_test_file(&content, &filename);
            assert!(
                result.is_ok(),
                "Failed to parse {}: {:?}",
                filename,
                result.err()
            );

            let test = result.unwrap();
            assert!(!test.name.is_empty(), "Test name is empty in {}", filename);
            assert!(!test.input.is_empty(), "Input is empty in {}", filename);
            assert!(
                !test.expected_output.is_empty(),
                "Expected output is empty in {}",
                filename
            );
            assert!(
                !test.metadata.file.is_empty(),
                "Metadata file is empty in {}",
                filename
            );
            assert!(
                !test.metadata.package.is_empty(),
                "Metadata package is empty in {}",
                filename
            );

            println!("✓ Parsed {}: {} (file: {}, package: {})",
                filename,
                test.name,
                test.metadata.file,
                test.metadata.package
            );
        }
    }

    #[test]
    fn test_transpile_all_examples() {
        use whitehall::transpiler::transpile;

        let test_files = load_test_files();
        let mut failures = Vec::new();

        for (filename, content) in test_files {
            let test = parse_test_file(&content, &filename).expect("Failed to parse test file");

            // Derive component name from metadata.file
            let component_name = test.metadata.file.trim_end_matches(".wh");

            println!("Testing: {} ({})", test.name, filename);

            // Call the transpiler
            match transpile(&test.input, &test.metadata.package, component_name) {
                Ok(actual_output) => {
                    if normalize_whitespace(&actual_output) != normalize_whitespace(&test.expected_output) {
                        println!("\n=== MISMATCH in {} ===", filename);
                        println!("Expected:\n{}", test.expected_output);
                        println!("\nActual:\n{}", actual_output);
                        println!("=========================\n");
                        failures.push(filename.clone());
                    } else {
                        println!("✓ {}", filename);
                    }
                }
                Err(e) => {
                    println!("\n=== TRANSPILATION ERROR in {} ===", filename);
                    println!("Error: {}", e);
                    println!("Input:\n{}", test.input);
                    println!("=========================\n");
                    failures.push(filename.clone());
                }
            }
        }

        if !failures.is_empty() {
            panic!("\n\n{} tests failed:\n{}\n", failures.len(), failures.join("\n"));
        }
    }

    #[test]
    fn test_basic_component_structure() {
        // Test that we can parse the basic component example
        let content = include_str!("transpiler-examples/01-basic-component.md");
        let test = parse_test_file(content, "01-basic-component.md")
            .expect("Failed to parse basic component test");

        assert_eq!(test.name, "Basic Component with Props");
        assert_eq!(test.metadata.file, "Avatar.wh");
        assert_eq!(test.metadata.package, "com.example.app.components");
        assert!(test.input.contains("@prop val url: String"));
        assert!(test.expected_output.contains("@Composable"));
        assert!(test.expected_output.contains("fun Avatar("));
    }

    #[test]
    fn test_control_flow_if_structure() {
        let content = include_str!("transpiler-examples/02-control-flow-if.md");
        let test = parse_test_file(content, "02-control-flow-if.md")
            .expect("Failed to parse if/else test");

        assert_eq!(test.name, "Control Flow: If/Else");
        assert_eq!(test.metadata.file, "StatusView.wh");
        assert!(test.input.contains("@if (isLoading)"));
        assert!(test.expected_output.contains("if (isLoading)"));
    }

    #[test]
    fn test_for_loop_structure() {
        let content = include_str!("transpiler-examples/03-control-flow-for.md");
        let test = parse_test_file(content, "03-control-flow-for.md")
            .expect("Failed to parse for loop test");

        assert_eq!(test.name, "Control Flow: For Loop with Key");
        assert_eq!(test.metadata.file, "PostList.wh");
        assert!(test.input.contains("@for (post in posts, key = { it.id })"));
        assert!(test.expected_output.contains("posts.forEach"));
        assert!(test.expected_output.contains("key(post.id)"));
    }

    #[test]
    fn test_routing_screen_type() {
        let content = include_str!("transpiler-examples/07-routing-simple.md");
        let test = parse_test_file(content, "07-routing-simple.md")
            .expect("Failed to parse routing test");

        assert_eq!(test.name, "Routing: Simple Navigation");
        assert_eq!(test.metadata.file, "WelcomeScreen.wh");
        assert_eq!(test.metadata.package, "com.example.app.screens");
        assert_eq!(test.metadata.type_hint, Some("screen".to_string()));
    }
}
