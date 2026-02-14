//! User-input event logger.
//!
//! Keeps a rolling buffer of recent mouse/keyboard events so the agent
//! can include them in context bundles sent to the server.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// A recorded input event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub key: Option<String>,
    pub modifiers: Vec<String>,
    pub timestamp: f64,
}

/// Thread-safe rolling buffer of recent events.
pub struct EventBuffer {
    events: Arc<Mutex<Vec<InputEvent>>>,
    max_size: usize,
}

impl EventBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::with_capacity(max_size))),
            max_size,
        }
    }

    pub async fn push(&self, event: InputEvent) {
        let mut events = self.events.lock().await;
        if events.len() >= self.max_size {
            events.remove(0);
        }
        events.push(event);
    }

    pub async fn drain(&self) -> Vec<InputEvent> {
        let mut events = self.events.lock().await;
        std::mem::take(&mut *events)
    }

    pub async fn snapshot(&self) -> Vec<InputEvent> {
        self.events.lock().await.clone()
    }
}

impl Default for EventBuffer {
    fn default() -> Self {
        Self::new(100)
    }
}
