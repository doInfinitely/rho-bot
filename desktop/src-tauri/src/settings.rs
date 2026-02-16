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
    #[serde(default)]
    pub user_email: String,
    pub capture_interval_ms: u64,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            server_url: "https://rho-bot-production.up.railway.app".into(),
            auth_token: String::new(),
            user_email: String::new(),
            capture_interval_ms: 500,
        }
    }
}

impl AppSettings {
    /// Derive the WebSocket agent URL from the REST base URL.
    pub fn ws_agent_url(&self) -> String {
        let base = self.server_url.trim_end_matches('/');
        let ws_base = if base.starts_with("https://") {
            base.replacen("https://", "wss://", 1)
        } else if base.starts_with("http://") {
            base.replacen("http://", "ws://", 1)
        } else {
            format!("wss://{}", base)
        };
        format!("{}/ws/agent", ws_base)
    }

    /// Derive the WebSocket record URL from the REST base URL.
    pub fn ws_record_url(&self) -> String {
        let base = self.server_url.trim_end_matches('/');
        let ws_base = if base.starts_with("https://") {
            base.replacen("https://", "wss://", 1)
        } else if base.starts_with("http://") {
            base.replacen("http://", "ws://", 1)
        } else {
            format!("wss://{}", base)
        };
        format!("{}/ws/record", ws_base)
    }

    pub fn is_logged_in(&self) -> bool {
        !self.auth_token.is_empty()
    }

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
