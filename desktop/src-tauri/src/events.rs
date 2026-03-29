//! User-input event logger.
//!
//! Keeps a rolling buffer of recent mouse/keyboard events so the agent
//! can include them in context bundles sent to the server.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
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

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Thread-safe rolling buffer of recent events.
pub struct EventBuffer {
    events: Arc<Mutex<Vec<InputEvent>>>,
    max_size: usize,
    /// Epoch-millis of the most recent user input event (lock-free).
    last_activity_ms: Arc<AtomicU64>,
}

impl EventBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::with_capacity(max_size))),
            max_size,
            last_activity_ms: Arc::new(AtomicU64::new(now_ms())),
        }
    }

    pub async fn push(&self, event: InputEvent) {
        // Update last-activity from the event timestamp (seconds → millis)
        let event_ms = (event.timestamp * 1000.0) as u64;
        self.last_activity_ms.fetch_max(event_ms, Ordering::Relaxed);

        let mut events = self.events.lock().await;
        if events.len() >= self.max_size {
            events.remove(0);
        }
        events.push(event);
    }

    pub async fn drain(&self) -> Vec<InputEvent> {
        let mut events = self.events.lock().await;
        let taken = std::mem::take(&mut *events);
        events.shrink_to_fit();
        taken
    }

    pub async fn snapshot(&self) -> Vec<InputEvent> {
        self.events.lock().await.clone()
    }

    /// Seconds since the last user input event. Lock-free.
    pub fn idle_seconds(&self) -> f64 {
        let last = self.last_activity_ms.load(Ordering::Relaxed);
        let now = now_ms();
        (now.saturating_sub(last)) as f64 / 1000.0
    }
}

impl Default for EventBuffer {
    fn default() -> Self {
        Self::new(100)
    }
}
