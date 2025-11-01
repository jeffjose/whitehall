use std::fs;
use tempfile::TempDir;
use serial_test::serial;

// Import the init command
use whitehall::commands::init;

#[test]
#[serial]
fn test_init_creates_project_structure() {
    let temp_dir = TempDir::new().unwrap();
    let project_name = "test-project";
    let project_path = temp_dir.path().join(project_name);

    // Change to temp directory
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Execute init command
    let result = init::execute(project_name);
    assert!(result.is_ok(), "Init command should succeed");

    // Verify directory structure
    assert!(project_path.exists(), "Project directory should exist");
    assert!(project_path.is_dir(), "Project path should be a directory");

    // Verify files exist
    assert!(
        project_path.join("whitehall.toml").exists(),
        "whitehall.toml should exist"
    );
    assert!(
        project_path.join("src").exists(),
        "src directory should exist"
    );
    assert!(
        project_path.join("src/main.wh").exists(),
        "src/main.wh should exist"
    );
    assert!(
        project_path.join(".gitignore").exists(),
        ".gitignore should exist"
    );

    // Verify src is a directory
    assert!(
        project_path.join("src").is_dir(),
        "src should be a directory"
    );
}

#[test]
#[serial]
fn test_init_substitutes_project_name() {
    let temp_dir = TempDir::new().unwrap();
    let project_name = "my-awesome-app";
    let project_path = temp_dir.path().join(project_name);

    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Execute init command
    init::execute(project_name).unwrap();

    // Read whitehall.toml
    let manifest_content = fs::read_to_string(project_path.join("whitehall.toml")).unwrap();

    // Verify project name substitution
    assert!(
        manifest_content.contains(&format!("name = \"{}\"", project_name)),
        "Manifest should contain project name"
    );

    // Verify snake_case package name substitution
    assert!(
        manifest_content.contains("package = \"com.example.my_awesome_app\""),
        "Manifest should contain snake_case package name"
    );
}

#[test]
#[serial]
fn test_init_handles_different_naming_conventions() {
    let test_cases = vec![
        ("simple", "simple"),
        ("with-dashes", "with_dashes"),
        ("MixedCase", "mixedcase"),
        ("with spaces", "with_spaces"),
    ];

    for (input_name, expected_snake) in test_cases {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join(input_name);

        std::env::set_current_dir(temp_dir.path()).unwrap();

        init::execute(input_name).unwrap();

        let manifest_content = fs::read_to_string(project_path.join("whitehall.toml")).unwrap();

        assert!(
            manifest_content.contains(&format!("package = \"com.example.{}\"", expected_snake)),
            "Package name should be snake_case for input '{}'",
            input_name
        );
    }
}

#[test]
#[serial]
fn test_init_fails_for_existing_directory() {
    let temp_dir = TempDir::new().unwrap();
    let project_name = "existing-project";
    let project_path = temp_dir.path().join(project_name);

    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Create directory manually
    fs::create_dir(&project_path).unwrap();

    // Try to init - should fail
    let result = init::execute(project_name);
    assert!(result.is_err(), "Init should fail for existing directory");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("already exists"),
        "Error message should mention directory already exists"
    );
}

#[test]
#[serial]
fn test_init_creates_valid_gitignore() {
    let temp_dir = TempDir::new().unwrap();
    let project_name = "test-gitignore";
    let project_path = temp_dir.path().join(project_name);

    std::env::set_current_dir(temp_dir.path()).unwrap();

    init::execute(project_name).unwrap();

    let gitignore_content = fs::read_to_string(project_path.join(".gitignore")).unwrap();

    // Verify it contains expected patterns (not exact content)
    assert!(
        gitignore_content.contains(".whitehall"),
        ".gitignore should ignore .whitehall directory"
    );
    assert!(
        gitignore_content.contains("*.apk") || gitignore_content.contains(".apk"),
        ".gitignore should ignore APK files"
    );
}

#[test]
#[serial]
fn test_init_creates_main_wh_file() {
    let temp_dir = TempDir::new().unwrap();
    let project_name = "test-wh-file";
    let project_path = temp_dir.path().join(project_name);

    std::env::set_current_dir(temp_dir.path()).unwrap();

    init::execute(project_name).unwrap();

    let main_wh_path = project_path.join("src/main.wh");

    // Verify file exists
    assert!(main_wh_path.exists(), "main.wh should exist");

    // Verify it's not empty
    let content = fs::read_to_string(main_wh_path).unwrap();
    assert!(!content.is_empty(), "main.wh should not be empty");

    // Verify it's a valid text file (not checking syntax, just that it's readable)
    assert!(content.len() > 0, "main.wh should have content");
}
