use serde::{Deserialize, Serialize};
use std::path::Path;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub requires_auth: bool,
    #[serde(default)]
    pub stars: u32,
    #[serde(default)]
    pub category: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LoadedSkill {
    pub metadata: SkillMetadata,
    pub instructions: String,
}

#[allow(dead_code)]
impl LoadedSkill {
    /// Parse a standard Agent Skills SKILL.md file
    pub fn parse(content: &str) -> Result<Self, String> {
        let trimmed = content.trim_start();
        if !trimmed.starts_with("---") {
            return Err("Missing YAML frontmatter delimiter (---)".to_string());
        }

        let parts: Vec<&str> = trimmed.splitn(3, "---").collect();
        if parts.len() < 3 {
            return Err("Invalid frontmatter structure".to_string());
        }

        let yaml_str = parts[1];
        let instructions = parts[2].trim().to_string();

        let metadata: SkillMetadata = serde_yaml::from_str(yaml_str)
            .map_err(|e| format!("Failed to parse YAML frontmatter: {}", e))?;

        Ok(Self {
            metadata,
            instructions,
        })
    }

    /// Load from a filesystem path
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read file {:?}: {}", path, e))?;
        Self::parse(&content)
    }
}
