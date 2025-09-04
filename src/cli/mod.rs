pub mod commands;
pub mod templates;
pub mod context;
pub mod utils;

use anyhow::Result;
use std::path::Path;

pub fn is_synapse_project() -> bool {
    Path::new("Cargo.toml").exists() && 
    Path::new(".synapse").exists()
}

pub fn ensure_synapse_directory() -> Result<()> {
    let synapse_dir = Path::new(".synapse");
    if !synapse_dir.exists() {
        std::fs::create_dir_all(synapse_dir)?;
        std::fs::create_dir_all(synapse_dir.join("templates"))?;
        std::fs::create_dir_all(synapse_dir.join("rules"))?;
        std::fs::create_dir_all(synapse_dir.join("architecture"))?;
        std::fs::create_dir_all(synapse_dir.join("decisions"))?;
    }
    Ok(())
}