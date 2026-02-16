//! The agent loop: capture -> send -> receive -> execute -> repeat.

use crate::capture;
use crate::accessibility;
use crate::events::EventBuffer;
use crate::executor::{self, Action};
use crate::platform;
use crate::settings::AppSettings;
use crate::ws_client::WsClient;

use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, Notify};
use uuid::Uuid;

/// Maximum number of reconnect attempts before giving up.
const MAX_RECONNECT_ATTEMPTS: u32 = 5;

/// Shared handle the React frontend uses to control the agent.
pub struct AgentHandle {
    state: Arc<Mutex<String>>,
    last_error: Arc<Mutex<String>>,
    settings: Arc<Mutex<AppSettings>>,
    event_buffer: Arc<EventBuffer>,
    recent_actions: Arc<Mutex<Vec<Value>>>,
    stop_signal: Arc<Notify>,
    running: Arc<Mutex<bool>>,
    recording_stop: Arc<Notify>,
    recording: Arc<Mutex<bool>>,
}

impl AgentHandle {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new("disconnected".into())),
            last_error: Arc::new(Mutex::new(String::new())),
            settings: Arc::new(Mutex::new(AppSettings::default())),
            event_buffer: Arc::new(EventBuffer::new(100)),
            recent_actions: Arc::new(Mutex::new(Vec::new())),
            stop_signal: Arc::new(Notify::new()),
            running: Arc::new(Mutex::new(false)),
            recording_stop: Arc::new(Notify::new()),
            recording: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn state(&self) -> String {
        self.state.lock().await.clone()
    }

    pub async fn last_error(&self) -> String {
        self.last_error.lock().await.clone()
    }

    pub async fn start(&self) -> Result<(), String> {
        let mut running = self.running.lock().await;
        if *running {
            return Err("Agent is already running".into());
        }
        *running = true;
        drop(running);

        // Clear previous error
        *self.last_error.lock().await = String::new();

        let state = self.state.clone();
        let last_error = self.last_error.clone();
        let settings = self.settings.clone();
        let event_buffer = self.event_buffer.clone();
        let recent_actions = self.recent_actions.clone();
        let stop_signal = self.stop_signal.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let result = run_agent_loop(
                state.clone(),
                last_error.clone(),
                settings,
                event_buffer,
                recent_actions,
                stop_signal,
            )
            .await;

            if let Err(e) = result {
                log::error!("Agent loop error: {}", e);
                *last_error.lock().await = e;
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

    pub async fn start_recording(&self) -> Result<(), String> {
        let running = self.running.lock().await;
        if *running {
            return Err("Cannot record while agent is running".into());
        }
        drop(running);

        let mut recording = self.recording.lock().await;
        if *recording {
            return Err("Already recording".into());
        }
        *recording = true;
        drop(recording);

        let state = self.state.clone();
        let settings = self.settings.clone();
        let event_buffer = self.event_buffer.clone();
        let stop_signal = self.recording_stop.clone();
        let recording = self.recording.clone();

        tokio::spawn(async move {
            let result = run_recording_loop(
                state.clone(),
                settings,
                event_buffer,
                stop_signal,
            )
            .await;

            if let Err(e) = result {
                log::error!("Recording loop error: {}", e);
            }

            *state.lock().await = "disconnected".into();
            *recording.lock().await = false;
        });

        Ok(())
    }

    pub async fn stop_recording(&self) {
        self.recording_stop.notify_one();
        *self.recording.lock().await = false;
    }

    pub async fn update_settings(&self, new: AppSettings) {
        *self.settings.lock().await = new;
    }

    /// Access the shared event buffer (e.g. to wire up input monitoring).
    pub fn event_buffer(&self) -> Arc<EventBuffer> {
        self.event_buffer.clone()
    }
}

fn now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

/// Check for stop signal without blocking.
async fn should_stop(stop_signal: &Notify) -> bool {
    tokio::time::timeout(std::time::Duration::from_millis(1), stop_signal.notified())
        .await
        .is_ok()
}

async fn run_agent_loop(
    state: Arc<Mutex<String>>,
    last_error: Arc<Mutex<String>>,
    settings: Arc<Mutex<AppSettings>>,
    event_buffer: Arc<EventBuffer>,
    recent_actions: Arc<Mutex<Vec<Value>>>,
    stop_signal: Arc<Notify>,
) -> Result<(), String> {
    let session_id = Uuid::new_v4().to_string();

    let (ws_url, token, interval) = {
        let s = settings.lock().await;
        (s.ws_agent_url(), s.auth_token.clone(), s.capture_interval_ms)
    };

    log::info!("Agent connecting to {}", ws_url);

    // ---- connect with retries ----
    let mut client: Option<WsClient> = None;
    let mut attempts: u32 = 0;

    while attempts < MAX_RECONNECT_ATTEMPTS {
        if should_stop(&stop_signal).await {
            return Ok(());
        }

        *state.lock().await = format!("connecting (attempt {})", attempts + 1);

        match WsClient::connect(&ws_url, &token).await {
            Ok(c) => {
                log::info!("WebSocket connected on attempt {}", attempts + 1);
                client = Some(c);
                break;
            }
            Err(e) => {
                attempts += 1;
                let msg = format!("Connect failed (attempt {}): {}", attempts, e);
                log::warn!("{}", msg);
                *last_error.lock().await = msg;

                if attempts < MAX_RECONNECT_ATTEMPTS {
                    let backoff = std::time::Duration::from_secs(2u64.pow(attempts.min(4)));
                    tokio::time::sleep(backoff).await;
                }
            }
        }
    }

    let mut client = client.ok_or_else(|| {
        format!("Failed to connect after {} attempts to {}", MAX_RECONNECT_ATTEMPTS, ws_url)
    })?;

    *state.lock().await = "running".into();
    *last_error.lock().await = String::new();

    let mut consecutive_errors: u32 = 0;

    loop {
        if should_stop(&stop_signal).await {
            break;
        }

        // 1. Capture screenshot
        let screenshot = match capture::capture_screen() {
            Ok(s) => { consecutive_errors = 0; s }
            Err(e) => {
                log::warn!("Screenshot failed: {}", e);
                consecutive_errors += 1;
                String::new()
            }
        };

        // 2. Read accessibility tree
        let tree = accessibility::read_frontmost_tree();

        // 3. Gather recent events
        let events = event_buffer.drain().await;

        // 4. Build context bundle
        let (app_name, _pid) = platform::frontmost_app();
        let (wx, wy, ww, wh) = platform::focused_window_bounds();

        let context = json!({
            "session_id": session_id,
            "timestamp": now(),
            "screenshot_b64": screenshot,
            "accessibility_tree": tree,
            "recent_events": events,
            "active_app": app_name,
            "window_bounds": {"x": wx, "y": wy, "width": ww, "height": wh}
        });

        // 5. Send & receive (with one reconnect attempt on failure)
        match client.send_context(&context).await {
            Ok(action_json) => {
                consecutive_errors = 0;

                // 6. Check for server-side errors
                if let Ok(action) = serde_json::from_value::<Action>(action_json.clone()) {
                    if let Some(ref error_msg) = action.error {
                        if error_msg.starts_with("Server error:") {
                            // Transient server error — log it but keep the loop going
                            log::warn!("Server processing error (will retry): {}", error_msg);
                            *last_error.lock().await = error_msg.clone();
                            consecutive_errors += 1;
                        } else {
                            // Quota / billing error — stop the agent
                            log::warn!("Server denied action: {}", error_msg);
                            *state.lock().await = "quota_exceeded".into();
                            *last_error.lock().await = error_msg.clone();

                            let mut log = recent_actions.lock().await;
                            log.push(action_json);
                            break;
                        }
                    } else {
                        // 7. Execute
                        if action.action_type != "noop" {
                            if let Err(e) = executor::execute(&action) {
                                log::error!("Action execution failed: {}", e);
                            }
                        }
                    }
                }

                // 8. Log for the UI
                let mut log = recent_actions.lock().await;
                log.push(action_json);
                let len = log.len();
                if len > 50 {
                    let drain_count = len - 50;
                    log.drain(..drain_count);
                }
            }
            Err(e) => {
                log::error!("Server communication failed: {}", e);
                *last_error.lock().await = e.clone();
                consecutive_errors += 1;

                // Try to reconnect once
                log::info!("Attempting reconnect...");
                *state.lock().await = "reconnecting".into();

                match WsClient::connect(&ws_url, &token).await {
                    Ok(new_client) => {
                        log::info!("Reconnected successfully");
                        client = new_client;
                        *state.lock().await = "running".into();
                        *last_error.lock().await = String::new();
                        // Continue the loop — the context we failed to send is lost,
                        // but we'll capture fresh data next iteration.
                        continue;
                    }
                    Err(reconnect_err) => {
                        let msg = format!("Reconnect also failed: {}. Original error: {}", reconnect_err, e);
                        log::error!("{}", msg);
                        *last_error.lock().await = msg.clone();
                        return Err(msg);
                    }
                }
            }
        }

        // 9. Sleep (back off if capture keeps failing)
        let sleep_ms = if consecutive_errors > 5 {
            interval.max(5000)
        } else {
            interval
        };
        tokio::time::sleep(std::time::Duration::from_millis(sleep_ms)).await;
    }

    client.close().await;
    Ok(())
}

/// Passive recording loop: captures context, observes user actions, and
/// sends context/action pairs to the server for training data collection.
async fn run_recording_loop(
    state: Arc<Mutex<String>>,
    settings: Arc<Mutex<AppSettings>>,
    event_buffer: Arc<EventBuffer>,
    stop_signal: Arc<Notify>,
) -> Result<(), String> {
    let session_id = Uuid::new_v4().to_string();

    let (ws_url, token, interval) = {
        let s = settings.lock().await;
        (s.ws_record_url(), s.auth_token.clone(), s.capture_interval_ms)
    };

    // Connect
    *state.lock().await = "connected".into();
    let mut client = WsClient::connect(&ws_url, &token).await?;
    *state.lock().await = "recording".into();

    loop {
        if should_stop(&stop_signal).await {
            break;
        }

        // 1. Capture the current screen state
        let screenshot = match capture::capture_screen() {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Screenshot failed: {}", e);
                String::new()
            }
        };

        let tree = accessibility::read_frontmost_tree();
        let (app_name, _pid) = platform::frontmost_app();
        let (wx, wy, ww, wh) = platform::focused_window_bounds();

        let context = json!({
            "session_id": session_id,
            "timestamp": now(),
            "screenshot_b64": screenshot,
            "accessibility_tree": tree,
            "recent_events": [],
            "active_app": app_name,
            "window_bounds": {"x": wx, "y": wy, "width": ww, "height": wh}
        });

        // 2. Wait for the user to act
        tokio::time::sleep(std::time::Duration::from_millis(interval)).await;

        // 3. Drain input events that occurred during the interval
        let user_actions = event_buffer.drain().await;

        // 4. Send the context/action pair to the server
        let payload = json!({
            "context": context,
            "user_actions": user_actions,
        });

        if let Err(e) = client.send_training_pair(&payload).await {
            log::error!("Failed to send training pair: {}", e);
            break;
        }
    }

    client.close().await;
    Ok(())
}
