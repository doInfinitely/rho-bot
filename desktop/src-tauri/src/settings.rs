use serde::{Deserialize, Serialize};

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
