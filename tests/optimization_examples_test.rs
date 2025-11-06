/// Optimization validation tests
///
/// Tests that optimization examples match expected output:
/// - Static collections should generate RecyclerView (Optimized Output)
/// - Dynamic collections should stay Compose (Unoptimized Output)

use std::fs;
use std::path::PathBuf;
use similar::{ChangeTag, TextDiff};

/// Test metadata extracted from ## Metadata section
#[derive(Debug)]
struct TestMetadata {
    file: String,
    package: String,
}

/// Represents a parsed optimization test case
#[derive(Debug)]
struct OptimizationTest {
    name: String,
    metadata: TestMetadata,
    input: String,
    optimized_output: String,
}

/// Parse a markdown test file
fn parse_test_file(content: &str, filename: &str) -> Result<OptimizationTest, String> {
    let mut lines = content.lines().peekable();
    let mut name = String::new();
    let mut input = String::new();
    let mut optimized_output = String::new();
    let mut metadata_content = String::new();

    let mut in_input_section = false;
    let mut in_optimized_section = false;
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
        return Err(format!("{}: Missing test name (# heading)", filename));
    }

    // Parse sections
    while let Some(line) = lines.next() {
        if line.starts_with("## Input") {
            in_input_section = true;
            in_optimized_section = false;
            in_metadata_section = false;
            continue;
        } else if line.starts_with("## Optimized Output") {
            in_input_section = false;
            in_optimized_section = true;
            in_metadata_section = false;
            continue;
        } else if line.starts_with("## Metadata") {
            in_input_section = false;
            in_optimized_section = false;
            in_metadata_section = true;
            continue;
        } else if line.starts_with("## ") {
            // Other section (like Unoptimized Output), skip it
            in_input_section = false;
            in_optimized_section = false;
            in_metadata_section = false;
            continue;
        }

        // Handle code blocks
        if line.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        if in_code_block {
            if in_input_section {
                input.push_str(line);
                input.push('\n');
            } else if in_optimized_section {
                optimized_output.push_str(line);
                optimized_output.push('\n');
            } else if in_metadata_section {
                metadata_content.push_str(line);
                metadata_content.push('\n');
            }
        }
    }

    // Parse metadata
    let mut file = String::new();
    let mut package = String::new();

    for line in metadata_content.lines() {
        if let Some(value) = line.strip_prefix("file:") {
            file = value.trim().to_string();
        } else if let Some(value) = line.strip_prefix("package:") {
            package = value.trim().to_string();
        }
    }

    if file.is_empty() || package.is_empty() {
        return Err(format!("{}: Missing metadata (file or package)", filename));
    }

    Ok(OptimizationTest {
        name,
        metadata: TestMetadata { file, package },
        input: input.trim().to_string(),
        optimized_output: optimized_output.trim().to_string(),
    })
}

/// Run transpiler with optimizations enabled
fn transpile_with_optimizations(
    input: &str,
    package: &str,
    component_name: &str,
) -> Result<String, String> {
    whitehall::transpiler::transpile(input, package, component_name, None)
        .map(|result| result.primary_content().to_string())
}

/// Normalize whitespace in Kotlin code for comparison
fn normalize_kotlin(code: &str) -> String {
    code.lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_examples() {
        let examples_dir = PathBuf::from("tests/optimization-examples");

        // Find all markdown files
        let mut test_files: Vec<_> = fs::read_dir(&examples_dir)
            .expect("Failed to read optimization-examples directory")
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "md" && path.file_name()? != "README.md" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        test_files.sort();

        let mut passed = 0;
        let mut failed = 0;

        for test_file in test_files {
            let filename = test_file.file_name().unwrap().to_string_lossy().to_string();
            let content = fs::read_to_string(&test_file)
                .unwrap_or_else(|e| panic!("Failed to read {}: {}", filename, e));

            let test = match parse_test_file(&content, &filename) {
                Ok(t) => t,
                Err(e) => {
                    println!("\x1b[31m✗\x1b[0m [PARSE ERROR] {}: {}", filename, e);
                    failed += 1;
                    continue;
                }
            };

            println!("Testing: {} ({})", test.name, filename);

            // Extract component name from metadata file
            let component_name = test
                .metadata
                .file
                .strip_suffix(".wh")
                .unwrap_or(&test.metadata.file);

            // Transpile with optimizations
            let result = match transpile_with_optimizations(
                &test.input,
                &test.metadata.package,
                component_name,
            ) {
                Ok(output) => output,
                Err(e) => {
                    println!("\x1b[31m✗\x1b[0m [TRANSPILE ERROR] {}: {}", filename, e);
                    failed += 1;
                    continue;
                }
            };

            // Normalize both outputs for comparison
            let actual = normalize_kotlin(&result);
            let expected = normalize_kotlin(&test.optimized_output);

            if actual == expected {
                println!("\x1b[32m✓\x1b[0m [PASS] {}", filename);
                passed += 1;
            } else {
                println!("\x1b[31m✗\x1b[0m [FAIL] {}", filename);
                println!("\n===== Expected vs Actual Diff =====");

                let diff = TextDiff::from_lines(&expected, &actual);
                for change in diff.iter_all_changes() {
                    let (sign, style) = match change.tag() {
                        ChangeTag::Delete => ("-", "\x1b[31m"),
                        ChangeTag::Insert => ("+", "\x1b[32m"),
                        ChangeTag::Equal => (" ", ""),
                    };
                    print!("{}{}{}\x1b[0m", style, sign, change);
                }
                println!("====================================\n");
                failed += 1;
            }
        }

        println!(
            "\n\x1b[1;32m✓ {}/{} tests passed!\x1b[0m",
            passed,
            passed + failed
        );

        assert_eq!(
            failed,
            0,
            "{} optimization example test(s) failed",
            failed
        );
    }
}
