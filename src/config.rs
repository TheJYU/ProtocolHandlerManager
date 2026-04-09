use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Config {
    pub mappings: HashMap<String, String>,
}

impl Config {
    fn config_path() -> Result<PathBuf, String> {
        let base_dirs = BaseDirs::new().ok_or("Failed to get base directories")?;
        let mut path = base_dirs.config_dir().to_path_buf();
        path.push("protocol-handler-manager");
        
        fs::create_dir_all(&path).map_err(|e| format!("Failed to create config directory: {}", e))?;
        Ok(path.join("mappings.json"))
    }

    pub fn load() -> Self {
        if let Ok(path) = Self::config_path() {
            if path.exists() {
                if let Ok(data) = fs::read_to_string(path) {
                    return serde_json::from_str(&data).unwrap_or_default();
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path()?;
        let data = serde_json::to_string_pretty(self).map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(path, data).map_err(|e| format!("Failed to write config file: {}", e))?;
        Ok(())
    }

    pub fn add_mapping(&mut self, protocol: String, target: String) -> Result<(), String> {
        self.mappings.insert(protocol, target);
        self.save()
    }

    pub fn remove_mapping(&mut self, protocol: &str) -> Result<(), String> {
        self.mappings.remove(protocol);
        self.save()
    }
}
