use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

const MANIFEST_TEMPLATE: &str = include_str!("../../templates/whitehall.toml");
const MAIN_WH_TEMPLATE: &str = include_str!("../../templates/src/main.wh");
const HOME_SCREEN_TEMPLATE: &str = include_str!("../../templates/src/routes/+screen.wh");
const GITIGNORE_TEMPLATE: &str = include_str!("../../templates/.gitignore");

pub fn execute(project_name: &str) -> Result<()> {
    let project_path = Path::new(project_name);

    // Check if directory already exists
    if project_path.exists() {
        anyhow::bail!("Directory '{}' already exists", project_name);
    }

    // Create project structure
    fs::create_dir_all(project_path.join("src/routes"))
        .context("Failed to create project directories")?;

    // Get current username for package name
    let username = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "example".to_string())
        .to_lowercase()
        .replace('-', "_");

    // Generate whitehall.toml with substitutions
    let manifest_content = MANIFEST_TEMPLATE
        .replace("{{PROJECT_NAME}}", project_name)
        .replace("{{PROJECT_NAME_SNAKE}}", &to_snake_case(project_name))
        .replace("{{USER}}", &username);

    fs::write(project_path.join("whitehall.toml"), manifest_content)
        .context("Failed to write whitehall.toml")?;

    // Copy main.wh (app shell)
    fs::write(project_path.join("src/main.wh"), MAIN_WH_TEMPLATE)
        .context("Failed to write src/main.wh")?;

    // Copy home screen
    fs::write(project_path.join("src/routes/+screen.wh"), HOME_SCREEN_TEMPLATE)
        .context("Failed to write src/routes/+screen.wh")?;

    // Copy .gitignore
    fs::write(project_path.join(".gitignore"), GITIGNORE_TEMPLATE)
        .context("Failed to write .gitignore")?;

    println!(
        "     {} android app `{}`",
        "Created".green().bold(),
        project_name
    );

    Ok(())
}

fn to_snake_case(s: &str) -> String {
    s.replace('-', "_").replace(' ', "_").to_lowercase()
}
