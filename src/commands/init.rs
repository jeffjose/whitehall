use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

const MANIFEST_TEMPLATE: &str = include_str!("../../templates/whitehall.toml");
const MAIN_WH_TEMPLATE: &str = include_str!("../../templates/src/main.wh");
const GITIGNORE_TEMPLATE: &str = include_str!("../../templates/.gitignore");

pub fn execute(project_name: &str) -> Result<()> {
    let project_path = Path::new(project_name);

    // Check if directory already exists
    if project_path.exists() {
        anyhow::bail!("Directory '{}' already exists", project_name);
    }

    // Create project structure

    fs::create_dir_all(project_path.join("src"))
        .context("Failed to create project directories")?;

    // Generate whitehall.toml with substitutions
    let manifest_content = MANIFEST_TEMPLATE
        .replace("{{PROJECT_NAME}}", project_name)
        .replace("{{PROJECT_NAME_SNAKE}}", &to_snake_case(project_name));

    fs::write(project_path.join("whitehall.toml"), manifest_content)
        .context("Failed to write whitehall.toml")?;

    // Copy main.wh
    fs::write(project_path.join("src/main.wh"), MAIN_WH_TEMPLATE)
        .context("Failed to write src/main.wh")?;

    // Copy .gitignore
    fs::write(project_path.join(".gitignore"), GITIGNORE_TEMPLATE)
        .context("Failed to write .gitignore")?;

    println!(
        "     {} binary (application) `{}` package",
        "Created".green().bold(),
        project_name
    );

    Ok(())
}

fn to_snake_case(s: &str) -> String {
    s.replace('-', "_").replace(' ', "_").to_lowercase()
}
