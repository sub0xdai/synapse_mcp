pub mod generic;
pub mod rust;
pub mod python;
pub mod typescript;

use anyhow::Result;
use std::fs;
use std::path::Path;

pub async fn write_template_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

pub fn replace_placeholders(template: &str, project_name: &str) -> String {
    template
        .replace("{{PROJECT_NAME}}", project_name)
        .replace("{{PROJECT_NAME_UPPER}}", &project_name.to_uppercase())
        .replace("{{PROJECT_NAME_LOWER}}", &project_name.to_lowercase())
        .replace("{{YEAR}}", &chrono::Utc::now().format("%Y").to_string())
        .replace("{{DATE}}", &chrono::Utc::now().format("%Y-%m-%d").to_string())
}