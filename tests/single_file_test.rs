use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use whitehall::single_file::{parse_frontmatter, generate_temp_project, hash_content};

#[test]
fn test_parse_hello_world() -> Result<()> {
    let content = fs::read_to_string("tests/single-file-examples/hello.wh")?;

    let (config, code) = parse_frontmatter(&content)?;

    // Check frontmatter was parsed correctly
    assert_eq!(config.app.name, "Hello World");
    assert_eq!(config.app.package, "com.example.hello");
    assert_eq!(config.app.min_sdk, 24); // default
    assert_eq!(config.app.target_sdk, 34); // default

    // Check code was extracted (without frontmatter)
    assert!(!code.contains("///"));
    assert!(!code.contains("#!"));
    assert!(code.contains("<Column"));
    assert!(code.contains("Hello, Whitehall!"));

    Ok(())
}

#[test]
fn test_parse_todo_app() -> Result<()> {
    let content = fs::read_to_string("tests/single-file-examples/todo-simple.wh")?;

    let (config, code) = parse_frontmatter(&content)?;

    // Check frontmatter
    assert_eq!(config.app.name, "Simple Todo");
    assert_eq!(config.app.package, "com.example.simpletodo");
    assert_eq!(config.app.min_sdk, 26);
    assert_eq!(config.app.target_sdk, 34);

    // Check code extraction
    assert!(!code.contains("///"));
    assert!(code.contains("var todos"));
    assert!(code.contains("fun addTodo"));
    assert!(code.contains("@for"));

    Ok(())
}

#[test]
fn test_missing_frontmatter() {
    let content = r#"
var count = 0

<Text>{count}</Text>
"#;

    let result = parse_frontmatter(content);
    assert!(result.is_err());

    let err = result.unwrap_err().to_string();
    assert!(err.contains("No frontmatter found"));
}

#[test]
fn test_invalid_frontmatter_toml() {
    let content = r#"
/// [app]
/// name = "Test
/// this is invalid TOML

<Text>Hello</Text>
"#;

    let result = parse_frontmatter(content);
    assert!(result.is_err());
}

#[test]
fn test_package_name_generation() -> Result<()> {
    // Test with app name that needs sanitization
    let content = r#"
/// [app]
/// name = "My Cool App!"

<Text>Hello</Text>
"#;

    let (config, _) = parse_frontmatter(content)?;

    // Package should be auto-generated and sanitized
    assert_eq!(config.app.package, "com.example.my_cool_app_");

    Ok(())
}

#[test]
fn test_content_hashing() {
    let content1 = "var x = 5";
    let content2 = "var x = 5";
    let content3 = "var y = 10";

    let hash1 = hash_content(content1);
    let hash2 = hash_content(content2);
    let hash3 = hash_content(content3);

    // Same content = same hash
    assert_eq!(hash1, hash2);

    // Different content = different hash
    assert_ne!(hash1, hash3);

    // Hash should be hex string
    assert!(hash1.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
#[ignore] // Ignoring by default as it creates files on disk
fn test_temp_project_generation() -> Result<()> {
    let file_path = PathBuf::from("tests/single-file-examples/hello.wh");
    let content = fs::read_to_string(&file_path)?;
    let (config, code) = parse_frontmatter(&content)?;

    // Generate temp project
    let project_dir = generate_temp_project(&file_path, &config, &code)?;

    // Check structure was created
    assert!(project_dir.exists());
    assert!(project_dir.join("whitehall.toml").exists());
    assert!(project_dir.join("src").exists());
    assert!(project_dir.join("src/main.wh").exists());

    // Check whitehall.toml content
    let toml_content = fs::read_to_string(project_dir.join("whitehall.toml"))?;
    assert!(toml_content.contains("name = \"Hello World\""));
    assert!(toml_content.contains("package = \"com.example.hello\""));

    // Check main.wh has code without frontmatter
    let main_wh = fs::read_to_string(project_dir.join("src/main.wh"))?;
    assert!(!main_wh.contains("///"));
    assert!(main_wh.contains("<Column"));

    Ok(())
}

#[test]
#[ignore] // Ignoring by default as it creates files on disk
fn test_cache_reuse() -> Result<()> {
    let file_path = PathBuf::from("tests/single-file-examples/hello.wh");
    let content = fs::read_to_string(&file_path)?;
    let (config, code) = parse_frontmatter(&content)?;

    // Generate temp project first time
    let project_dir1 = generate_temp_project(&file_path, &config, &code)?;

    // Generate again - should reuse cache
    let project_dir2 = generate_temp_project(&file_path, &config, &code)?;

    // Should be the same directory
    assert_eq!(project_dir1, project_dir2);

    Ok(())
}
