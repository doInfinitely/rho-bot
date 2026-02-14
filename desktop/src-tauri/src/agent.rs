//! The agent loop: capture -> send -> receive -> execute -> repeat.

use crate::capture;
use crate::accessibility;
use crate::events::EventBuffer;
use crate::executor::{self, Action};
use crate::settings::AppSettings;
use crate::ws_client::WsClient;

use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, Notify};
use uuid::Uuid;

/// Shared handle the React frontend uses to control the agent.
pub struct AgentHandle {
    state: Arc<Mutex<String>>,
    settings: Arc<Mutex<AppSettings>>,
    event_buffer: Arc<EventBuffer>,
    recent_actions: Arc<Mutex<Vec<Value>>>,
    stop_signal: Arc<Notify>,
    running: Arc<Mutex<bool>>,
}

impl AgentHandle {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new("disconnected".into())),
            settings: Arc::new(Mutex::new(AppSettings::default())),
            event_buffer: Arc::new(EventBuffer::new(100)),
            recent_actions: Arc::new(Mutex::new(Vec::new())),
            stop_signal: Arc::new(Notify::new()),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn state(&self) -> String {
        self.state.lock().await.clone()
    }

    pub async fn start(&self) -> Result<(), String> {
        let mut running = self.running.lock().await;
        if *running {
            return Err("Agent is already running".into());
        }
        *running = true;
        drop(running);

        let state = self.state.clone();
        let settings = self.settings.clone();
        let event_buffer = self.event_buffer.clone();
        let recent_actions = self.recent_actions.clone();
        let stop_signal = self.stop_signal.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let result = run_agent_loop(
                state.clone(),
                settings,
                event_buffer,
                recent_actions,
                stop_signal,
            )
            .await;

            if let Err(e) = result {
                log::error!("Agent loop error: {}", e);
            }

            *state.lock().await = "disconnected".into();
            *running.lock().await = false;
        });

        Ok(())
    }

    pub async fn stop(&self) {
        self.stop_signal.notify_one();
        *self.running.lock().await = false;
    }

    pub async fn recent_actions(&self) -> Vec<Value> {
        self.recent_actions.lock().await.clone()
    }

    pub async fn update_settings(&self, new: AppSettings) {
        *self.settings.lock().await = new;
    }
}

fn now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

async fn run_agent_loop(
    state: Arc<Mutex<String>>,
    settings: Arc<Mutex<AppSettings>>,
    event_buffer: Arc<EventBuffer>,
    recent_actions: Arc<Mutex<Vec<Value>>>,
    stop_signal: Arc<Notify>,
) -> Result<(), String> {
    let session_id = Uuid::new_v4().to_string();

    let (server_url, token, interval) = {
        let s = settings.lock().await;
        (s.server_url.clone(), s.auth_token.clone(), s.capture_interval_ms)
    };

    // Connect
    *state.lock().await = "connected".into();
    let mut client = WsClient::connect(&server_url, &token).await?;
    *state.lock().await = "running".into();

    loop {
        // Check for stop signal (non-blocking)
        if tokio::time::timeout(std::time::Duration::from_millis(1), stop_signal.notified())
            .await
            .is_ok()
        {
            break;
        }

        // 1. Capture screenshot
        let screenshot = match capture::capture_screen() {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Screenshot failed: {}", e);
                String::new()
            }
        };

        // 2. Read accessibility tree
        let tree = accessibility::read_frontmost_tree();

        // 3. Gather recent events
        let events = event_buffer.drain().await;

        // 4. Build context bundle
        let context = json!({
            "session_id": session_id,
            "timestamp": now(),
            "screenshot_b64": screenshot,
            "accessibility_tree": tree,
            "recent_events": events,
            "active_app": "",
            "window_bounds": {"x": 0, "y": 0, "width": 1920, "height": 1080}
        });

        // 5. Send & receive
        match client.send_context(&context).await {
            Ok(action_json) => {
                // 6. Execute
                if let Ok(action) = serde_json::from_value::<Action>(action_json.clone()) {
                    if action.action_type != "noop" {
                        if let Err(e) = executor::execute(&action) {
                            log::error!("Action execution failed: {}", e);
                        }
                    }
                }

                // 7. Log for the UI
                let mut log = recent_actions.lock().await;
                log.push(action_json);
                // Keep only the last 50 entries
                let len = log.len();
                if len > 50 {
                    let drain_count = len - 50;
                    log.drain(..drain_count);
                }
            }
            Err(e) => {
                log::error!("Server communication failed: {}", e);
                break;
            }
        }

        // 8. Sleep
        tokio::time::sleep(std::time::Duration::from_millis(interval)).await;
    }

    client.close().await;
    Ok(())
}
