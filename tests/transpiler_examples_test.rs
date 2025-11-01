/// Markdown-based transpiler tests
///
/// This test module parses markdown files from tests/transpiler-examples/
/// and validates transpilation output against expected Kotlin code.

use std::fs;
use std::path::PathBuf;

/// Represents a parsed transpiler test case
#[derive(Debug)]
struct TranspilerTest {
    name: String,
    input: String,
    expected_output: String,
}

/// Parse a markdown test file
fn parse_test_file(content: &str, filename: &str) -> Result<TranspilerTest, String> {
    let mut lines = content.lines().peekable();
    let mut name = String::new();
    let mut input = String::new();
    let mut expected_output = String::new();

    let mut in_input_section = false;
    let mut in_output_section = false;
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
            continue;
        } else if line.starts_with("## Output") {
            in_input_section = false;
            in_output_section = true;
            continue;
        } else if line.starts_with("## ") {
            // Other sections (like Notes) - skip
            in_input_section = false;
            in_output_section = false;
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
            }
        }
    }

    if input.is_empty() {
        return Err(format!("No input section found in {}", filename));
    }

    if expected_output.is_empty() {
        return Err(format!("No output section found in {}", filename));
    }

    Ok(TranspilerTest {
        name,
        input: input.trim().to_string(),
        expected_output: expected_output.trim().to_string(),
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

            println!("✓ Parsed {}: {}", filename, test.name);
        }
    }

    #[test]
    #[ignore] // TODO: Enable once transpiler is implemented
    fn test_transpile_all_examples() {
        let test_files = load_test_files();

        for (filename, content) in test_files {
            let test = parse_test_file(&content, &filename).expect("Failed to parse test file");

            // TODO: Call the actual transpiler once implemented
            // let actual_output = transpile(&test.input).expect("Transpilation failed");

            // For now, just verify the test structure
            println!("Would transpile: {}", test.name);

            // TODO: Enable this assertion once transpiler exists:
            // assert_eq!(
            //     normalize_whitespace(&actual_output),
            //     normalize_whitespace(&test.expected_output),
            //     "Transpilation mismatch in {}:\n\nInput:\n{}\n\nExpected:\n{}\n\nActual:\n{}",
            //     filename,
            //     test.input,
            //     test.expected_output,
            //     actual_output
            // );
        }
    }

    #[test]
    fn test_basic_component_structure() {
        // Test that we can parse the basic component example
        let content = include_str!("transpiler-examples/01-basic-component.md");
        let test = parse_test_file(content, "01-basic-component.md")
            .expect("Failed to parse basic component test");

        assert_eq!(test.name, "Basic Component with Props");
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
        assert!(test.input.contains("@if (isLoading)"));
        assert!(test.expected_output.contains("if (isLoading)"));
    }

    #[test]
    fn test_for_loop_structure() {
        let content = include_str!("transpiler-examples/03-control-flow-for.md");
        let test = parse_test_file(content, "03-control-flow-for.md")
            .expect("Failed to parse for loop test");

        assert_eq!(test.name, "Control Flow: For Loop with Key");
        assert!(test.input.contains("@for (post in posts, key = { it.id })"));
        assert!(test.expected_output.contains("posts.forEach"));
        assert!(test.expected_output.contains("key(post.id)"));
    }
}
