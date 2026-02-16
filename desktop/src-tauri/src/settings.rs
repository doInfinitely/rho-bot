//! Application settings with file-based persistence.
//!
//! Settings are stored as JSON in `~/.config/rho-bot/settings.json`
//! (or the platform-appropriate config directory).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub server_url: String,
    pub auth_token: String,
    pub capture_interval_ms: u64,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            server_url: "ws://localhost:8000/ws/agent".into(),
            auth_token: String::new(),
            capture_interval_ms: 500,
        }
    }
}

impl AppSettings {
    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("rho-bot").join("settings.json"))
    }

    /// Load settings from disk. Returns `None` if the file doesn't exist.
    pub fn load() -> Option<Self> {
        let path = Self::config_path()?;
        let data = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&data).ok()
    }

    /// Persist current settings to disk.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path()
            .ok_or_else(|| "Could not determine config directory".to_string())?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {}", e))?;
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        std::fs::write(&path, json)
            .map_err(|e| format!("Failed to write settings: {}", e))?;

        log::info!("Settings saved to {:?}", path);
        Ok(())
    }
}
