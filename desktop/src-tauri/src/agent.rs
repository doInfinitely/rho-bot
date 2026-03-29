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
    /// The task description for the marionette agent.
    task: Arc<Mutex<String>>,
    screen_stop: Arc<Notify>,
    screen_publishing: Arc<Mutex<bool>>,
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
            task: Arc::new(Mutex::new(String::new())),
            screen_stop: Arc::new(Notify::new()),
            screen_publishing: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn state(&self) -> String {
        self.state.lock().await.clone()
    }

    pub async fn last_error(&self) -> String {
        self.last_error.lock().await.clone()
    }

    /// Set the task for the marionette agent to execute.
    pub async fn set_task(&self, task: String) {
        *self.task.lock().await = task;
    }

    /// Get the current task.
    pub async fn get_task(&self) -> String {
        self.task.lock().await.clone()
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

        // Check if we should use marionette mode
        let (use_marionette, task) = {
            let s = settings.lock().await;
            (s.use_marionette, self.task.lock().await.clone())
        };

        if use_marionette && task.is_empty() {
            // Standby mode: connect and wait for tasks from iOS
            // Auto-restarts on error so the desktop stays available
            tokio::spawn(async move {
                loop {
                    let result = run_marionette_standby_loop(
                        state.clone(),
                        last_error.clone(),
                        settings.clone(),
                        event_buffer.clone(),
                        recent_actions.clone(),
                        stop_signal.clone(),
                    )
                    .await;

                    // Check if we were deliberately stopped
                    if should_stop(&stop_signal).await {
                        log::info!("Marionette standby stopped by user");
                        break;
                    }

                    match result {
                        Ok(()) => {
                            log::info!("Marionette standby exited cleanly, restarting in 5s");
                        }
                        Err(e) => {
                            log::error!("Marionette standby error: {}, restarting in 5s", e);
                            *last_error.lock().await = e;
                        }
                    }

                    *state.lock().await = "reconnecting".into();
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }

                *state.lock().await = "disconnected".into();
                *running.lock().await = false;
            });
        } else if use_marionette {
            tokio::spawn(async move {
                let result = run_marionette_loop(
                    state.clone(),
                    last_error.clone(),
                    settings,
                    event_buffer,
                    recent_actions,
                    stop_signal,
                    task,
                )
                .await;

                if let Err(e) = result {
                    log::error!("Marionette loop error: {}", e);
                    *last_error.lock().await = e;
                }

                *state.lock().await = "disconnected".into();
                *running.lock().await = false;
            });
        } else {
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
        }

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

    pub async fn start_screen_publisher(&self) -> Result<(), String> {
        let mut publishing = self.screen_publishing.lock().await;
        if *publishing {
            return Err("Already publishing screen".into());
        }
        *publishing = true;
        drop(publishing);

        let settings = self.settings.clone();
        let stop_signal = self.screen_stop.clone();
        let publishing = self.screen_publishing.clone();

        tokio::spawn(async move {
            let result = run_screen_publisher_loop(settings, stop_signal).await;

            if let Err(e) = result {
                log::error!("Screen publisher loop error: {}", e);
            }

            *publishing.lock().await = false;
        });

        Ok(())
    }

    pub async fn stop_screen_publisher(&self) {
        self.screen_stop.notify_one();
        *self.screen_publishing.lock().await = false;
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
    let mut session_id = Uuid::new_v4().to_string();
    let mut was_idle = false;

    let (ws_url, token, interval, idle_timeout) = {
        let s = settings.lock().await;
        (s.ws_agent_url(), s.auth_token.clone(), s.capture_interval_ms, s.idle_timeout_secs)
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

        // Idle detection: skip capture when user is inactive
        let idle_secs = event_buffer.idle_seconds();
        if idle_secs > idle_timeout as f64 {
            if !was_idle {
                log::info!(
                    "Agent idle: no input for {:.0}s (threshold {}s), pausing capture",
                    idle_secs, idle_timeout
                );
                was_idle = true;
                // Free memory held by recent_actions
                let mut actions = recent_actions.lock().await;
                actions.clear();
                actions.shrink_to_fit();
            }
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            continue;
        }

        // Idle → active transition: chop session
        if was_idle {
            log::info!("Agent resumed after idle, starting new session");
            session_id = Uuid::new_v4().to_string();
            was_idle = false;
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
                log::info!("Received action JSON: {}", action_json);
                match serde_json::from_value::<Action>(action_json.clone()) {
                    Err(e) => log::error!("Failed to deserialize action: {}. JSON: {}", e, action_json),
                    _ => {}
                }
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

/// Marionette remote agent loop: captures desktop state, sends to marionette
/// server which calls LLM, receives actions back, executes them locally.
async fn run_marionette_loop(
    state: Arc<Mutex<String>>,
    last_error: Arc<Mutex<String>>,
    settings: Arc<Mutex<AppSettings>>,
    event_buffer: Arc<EventBuffer>,
    recent_actions: Arc<Mutex<Vec<Value>>>,
    stop_signal: Arc<Notify>,
    task: String,
) -> Result<(), String> {
    if task.is_empty() {
        return Err("No task provided for marionette agent".into());
    }

    let session_id = Uuid::new_v4().to_string();

    let (ws_url, interval) = {
        let s = settings.lock().await;
        (s.ws_marionette_url(), s.capture_interval_ms)
    };

    log::info!("Marionette connecting to {} for task: {}", ws_url, task);

    // Connect with retries (no auth needed for marionette)
    let mut client: Option<WsClient> = None;
    let mut attempts: u32 = 0;

    while attempts < MAX_RECONNECT_ATTEMPTS {
        if should_stop(&stop_signal).await {
            return Ok(());
        }

        *state.lock().await = format!("connecting (attempt {})", attempts + 1);

        match WsClient::connect_no_auth(&ws_url).await {
            Ok(c) => {
                log::info!("Marionette WebSocket connected on attempt {}", attempts + 1);
                client = Some(c);
                break;
            }
            Err(e) => {
                attempts += 1;
                let msg = format!("Marionette connect failed (attempt {}): {}", attempts, e);
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
        format!("Failed to connect to marionette after {} attempts", MAX_RECONNECT_ATTEMPTS)
    })?;

    // Send start message with task and settings
    *state.lock().await = "starting".into();
    let agent_settings = json!({
        "llm_provider": "anthropic",
        "model": "claude-sonnet-4-20250514",
        "max_steps": 50,
    });

    if let Err(e) = client.send_start(&task, &agent_settings).await {
        return Err(format!("Failed to start marionette task: {}", e));
    }

    log::info!("Marionette task started, entering agent loop");
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

        // 5. Send context and receive response
        match client.send_marionette_context(&context).await {
            Ok(response) => {
                consecutive_errors = 0;
                let resp_type = response.get("type").and_then(|t| t.as_str()).unwrap_or("");

                log::info!("Marionette response type: {}", resp_type);

                match resp_type {
                    "actions" => {
                        // Execute the action batch
                        if let Some(actions_arr) = response.get("actions").and_then(|a| a.as_array()) {
                            let actions: Vec<Action> = actions_arr
                                .iter()
                                .filter_map(|a| serde_json::from_value::<Action>(a.clone()).ok())
                                .collect();

                            if !actions.is_empty() {
                                log::info!("Executing {} actions", actions.len());
                                if let Err(e) = executor::execute_batch(&actions) {
                                    log::error!("Batch execution failed: {}", e);
                                }
                            }
                        }

                        // Log for UI
                        let mut log_actions = recent_actions.lock().await;
                        log_actions.push(response);
                        let len = log_actions.len();
                        if len > 50 {
                            log_actions.drain(..len - 50);
                        }
                    }

                    "done" => {
                        let result = response.get("result")
                            .and_then(|r| r.as_str())
                            .unwrap_or("Task completed");
                        log::info!("Marionette task done: {}", result);

                        let mut log_actions = recent_actions.lock().await;
                        log_actions.push(response);
                        break;
                    }

                    "ask_user" => {
                        // TODO: Surface this to the desktop UI for user input
                        let message = response.get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("Agent is asking for input");
                        log::info!("Marionette ask_user: {}", message);

                        let mut log_actions = recent_actions.lock().await;
                        log_actions.push(response);
                        // For now, auto-respond with "yes" — in the future, show UI
                        if let Err(e) = client.send_chat("yes, proceed").await {
                            log::error!("Failed to send chat response: {}", e);
                        }
                    }

                    "step" => {
                        // Progress update — log it
                        let mut log_actions = recent_actions.lock().await;
                        log_actions.push(response);
                        let len = log_actions.len();
                        if len > 50 {
                            log_actions.drain(..len - 50);
                        }
                    }

                    "error" => {
                        let msg = response.get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("Unknown error");
                        log::error!("Marionette error: {}", msg);
                        *last_error.lock().await = msg.to_string();
                        consecutive_errors += 1;

                        let mut log_actions = recent_actions.lock().await;
                        log_actions.push(response);
                    }

                    other => {
                        log::warn!("Unknown marionette response type: {}", other);
                    }
                }
            }
            Err(e) => {
                log::error!("Marionette communication failed: {}", e);
                *last_error.lock().await = e.clone();
                consecutive_errors += 1;

                // Try to reconnect once
                log::info!("Attempting marionette reconnect...");
                *state.lock().await = "reconnecting".into();

                match WsClient::connect_no_auth(&ws_url).await {
                    Ok(mut new_client) => {
                        // Re-send start message
                        if let Err(e) = new_client.send_start(&task, &agent_settings).await {
                            let msg = format!("Reconnected but start failed: {}", e);
                            log::error!("{}", msg);
                            *last_error.lock().await = msg.clone();
                            return Err(msg);
                        }
                        log::info!("Marionette reconnected successfully");
                        client = new_client;
                        *state.lock().await = "running".into();
                        *last_error.lock().await = String::new();
                        continue;
                    }
                    Err(reconnect_err) => {
                        let msg = format!(
                            "Marionette reconnect failed: {}. Original: {}",
                            reconnect_err, e
                        );
                        log::error!("{}", msg);
                        *last_error.lock().await = msg.clone();
                        return Err(msg);
                    }
                }
            }
        }

        // Sleep between iterations
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

/// Marionette standby loop: connects to the server in standby mode and waits
/// for tasks forwarded from the iOS app. When a task arrives, enters the
/// capture/execute loop. On task completion, returns to standby.
///
/// Wraps everything in a reconnect loop so connection drops are recovered
/// automatically. Sends periodic pings to prevent idle timeouts.
async fn run_marionette_standby_loop(
    state: Arc<Mutex<String>>,
    last_error: Arc<Mutex<String>>,
    settings: Arc<Mutex<AppSettings>>,
    event_buffer: Arc<EventBuffer>,
    recent_actions: Arc<Mutex<Vec<Value>>>,
    stop_signal: Arc<Notify>,
) -> Result<(), String> {
    // Outer reconnect loop — keeps trying to stay connected
    loop {
        if should_stop(&stop_signal).await {
            return Ok(());
        }

        let (ws_url, interval) = {
            let s = settings.lock().await;
            (s.ws_marionette_url(), s.capture_interval_ms)
        };

        log::info!("Marionette standby: connecting to {}", ws_url);

        // Connect with retries
        let mut client: Option<WsClient> = None;
        let mut attempts: u32 = 0;

        while attempts < MAX_RECONNECT_ATTEMPTS {
            if should_stop(&stop_signal).await {
                return Ok(());
            }

            *state.lock().await = format!("connecting (attempt {})", attempts + 1);

            match WsClient::connect_no_auth(&ws_url).await {
                Ok(c) => {
                    log::info!("Marionette standby connected on attempt {}", attempts + 1);
                    client = Some(c);
                    break;
                }
                Err(e) => {
                    attempts += 1;
                    let msg = format!("Standby connect failed (attempt {}): {}", attempts, e);
                    log::warn!("{}", msg);
                    *last_error.lock().await = msg;

                    if attempts < MAX_RECONNECT_ATTEMPTS {
                        let backoff = std::time::Duration::from_secs(2u64.pow(attempts.min(4)));
                        tokio::time::sleep(backoff).await;
                    }
                }
            }
        }

        let mut client = match client {
            Some(c) => c,
            None => {
                log::error!("Failed to connect after {} attempts, will retry in 10s", MAX_RECONNECT_ATTEMPTS);
                *state.lock().await = "disconnected".into();
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                continue; // Retry the outer reconnect loop
            }
        };

        // Register as standby
        if let Err(e) = client.send_register().await {
            log::error!("Failed to register as standby: {}, will reconnect", e);
            *last_error.lock().await = e;
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            continue; // Retry the outer reconnect loop
        }

        log::info!("Marionette standby: registered, waiting for tasks");
        *state.lock().await = "standby".into();
        *last_error.lock().await = String::new();

        // Track when we last sent a keepalive
        let mut last_ping = std::time::Instant::now();
        let ping_interval = std::time::Duration::from_secs(30);

        // Inner loop: wait for tasks (breaks on connection error to trigger reconnect)
        let mut should_reconnect = false;
        loop {
            if should_stop(&stop_signal).await {
                client.close().await;
                return Ok(());
            }

            // Send keepalive ping if idle too long
            if last_ping.elapsed() >= ping_interval {
                if let Err(e) = client.send_ping().await {
                    log::warn!("Keepalive ping failed: {}, reconnecting", e);
                    should_reconnect = true;
                    break;
                }
                last_ping = std::time::Instant::now();
            }

            // Wait for a run_task message from the server (with timeout for ping checks)
            let msg = tokio::select! {
                _ = async {
                    // Check stop signal periodically
                    loop {
                        if should_stop(&stop_signal).await {
                            return;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    }
                } => {
                    client.close().await;
                    return Ok(());
                }
                _ = tokio::time::sleep(ping_interval) => {
                    // Time to send a keepalive — loop back
                    continue;
                }
                msg = client.receive_message() => msg,
            };

            last_ping = std::time::Instant::now(); // Got activity

            let msg = match msg {
                Ok(m) => m,
                Err(e) => {
                    log::error!("Standby receive failed: {}, will reconnect", e);
                    *last_error.lock().await = e;
                    should_reconnect = true;
                    break;
                }
            };

            let msg_type = msg.get("type").and_then(|t| t.as_str()).unwrap_or("");

            match msg_type {
                "run_task" => {
                    let task = msg.get("task").and_then(|t| t.as_str()).unwrap_or("").to_string();
                    if task.is_empty() {
                        log::warn!("Received run_task with empty task");
                        continue;
                    }

                    log::info!("Marionette standby: received task: {}", task);
                    *state.lock().await = "running".into();

                    let session_id = Uuid::new_v4().to_string();
                    let mut consecutive_errors: u32 = 0;

                    // Inner capture/execute loop for this task
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

                        // 5. Send context and receive response
                        match client.send_marionette_context(&context).await {
                            Ok(response) => {
                                consecutive_errors = 0;
                                let resp_type = response.get("type").and_then(|t| t.as_str()).unwrap_or("");

                                match resp_type {
                                    "actions" => {
                                        if let Some(actions_arr) = response.get("actions").and_then(|a| a.as_array()) {
                                            let actions: Vec<Action> = actions_arr
                                                .iter()
                                                .filter_map(|a| serde_json::from_value::<Action>(a.clone()).ok())
                                                .collect();

                                            if !actions.is_empty() {
                                                log::info!("Executing {} actions", actions.len());
                                                if let Err(e) = executor::execute_batch(&actions) {
                                                    log::error!("Batch execution failed: {}", e);
                                                }
                                            }
                                        }

                                        let mut log_actions = recent_actions.lock().await;
                                        log_actions.push(response);
                                        let len = log_actions.len();
                                        if len > 50 { log_actions.drain(..len - 50); }
                                    }

                                    "done" => {
                                        let result = response.get("result")
                                            .and_then(|r| r.as_str())
                                            .unwrap_or("Task completed");
                                        log::info!("Task done: {}", result);

                                        let mut log_actions = recent_actions.lock().await;
                                        log_actions.push(response);
                                        break; // Exit inner loop, return to standby
                                    }

                                    "stop" => {
                                        log::info!("Task stopped by server");
                                        break;
                                    }

                                    "ask_user" => {
                                        let message = response.get("message")
                                            .and_then(|m| m.as_str())
                                            .unwrap_or("Agent is asking for input");
                                        log::info!("Ask user: {}", message);
                                        // Auto-respond for now
                                        if let Err(e) = client.send_chat("yes, proceed").await {
                                            log::error!("Failed to send chat response: {}", e);
                                        }
                                    }

                                    "error" => {
                                        let msg = response.get("message")
                                            .and_then(|m| m.as_str())
                                            .unwrap_or("Unknown error");
                                        log::error!("Server error: {}", msg);
                                        *last_error.lock().await = msg.to_string();
                                        consecutive_errors += 1;
                                    }

                                    _ => {
                                        log::warn!("Unknown response type: {}", resp_type);
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("Context send failed: {}", e);
                                *last_error.lock().await = e;
                                consecutive_errors += 1;
                                if consecutive_errors > 3 {
                                    break; // Give up on this task
                                }
                            }
                        }

                        // Sleep between iterations
                        let sleep_ms = if consecutive_errors > 5 {
                            interval.max(5000)
                        } else {
                            interval
                        };
                        tokio::time::sleep(std::time::Duration::from_millis(sleep_ms)).await;
                    }

                    // Task finished or failed — re-register as standby
                    log::info!("Task finished, re-registering as standby");
                    *state.lock().await = "standby".into();
                    if let Err(e) = client.send_register().await {
                        log::error!("Failed to re-register: {}, will reconnect", e);
                        should_reconnect = true;
                        break;
                    }
                }

                "stop" => {
                    log::info!("Stop received in standby mode");
                    client.close().await;
                    return Ok(());
                }

                other => {
                    log::warn!("Unexpected message in standby: {}", other);
                }
            }
        }

        // Connection broke — retry after a brief pause
        if should_reconnect {
            log::info!("Connection lost, reconnecting in 3s...");
            *state.lock().await = "reconnecting".into();
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }
}

/// Passive recording loop: captures context, observes user actions, and
/// sends context/action pairs to the server for training data collection.
/// Runs forever, retrying connections with backoff. Re-reads settings each
/// attempt so it picks up credentials after login.
async fn run_recording_loop(
    state: Arc<Mutex<String>>,
    settings: Arc<Mutex<AppSettings>>,
    event_buffer: Arc<EventBuffer>,
    stop_signal: Arc<Notify>,
) -> Result<(), String> {
    let mut session_id = Uuid::new_v4().to_string();
    let mut was_idle = false;

    loop {
        if should_stop(&stop_signal).await {
            return Ok(());
        }

        // Re-read settings each connection attempt (picks up token after login)
        let (ws_url, token, interval, idle_timeout) = {
            let s = settings.lock().await;
            (s.ws_record_url(), s.auth_token.clone(), s.capture_interval_ms, s.idle_timeout_secs)
        };

        // Try to connect
        *state.lock().await = "connecting".into();
        let client = WsClient::connect(&ws_url, &token).await;

        let mut client = match client {
            Ok(c) => {
                log::info!("Recording WebSocket connected");
                c
            }
            Err(e) => {
                log::warn!("Recording connect failed (will retry): {}", e);
                *state.lock().await = "disconnected".into();
                // Wait before retrying — check for stop signal during wait
                for _ in 0..10 {
                    if should_stop(&stop_signal).await {
                        return Ok(());
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
                continue;
            }
        };

        *state.lock().await = "recording".into();

        // Inner capture loop — runs until connection breaks
        loop {
            if should_stop(&stop_signal).await {
                client.close().await;
                return Ok(());
            }

            // Idle detection: skip capture when user is inactive
            let idle_secs = event_buffer.idle_seconds();
            if idle_secs > idle_timeout as f64 {
                if !was_idle {
                    log::info!(
                        "Recording idle: no input for {:.0}s (threshold {}s), pausing capture",
                        idle_secs, idle_timeout
                    );
                    was_idle = true;
                }
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }

            // Idle → active transition: chop session
            if was_idle {
                log::info!("Recording resumed after idle, starting new session");
                session_id = Uuid::new_v4().to_string();
                was_idle = false;
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

            // Read current auth state each iteration
            let (user_email, logged_in) = {
                let s = settings.lock().await;
                (s.user_email.clone(), s.is_logged_in())
            };

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

            // 3. Snapshot input events
            let user_actions = event_buffer.snapshot().await;

            // 4. Send the context/action pair to the server
            let payload = json!({
                "context": context,
                "user_actions": user_actions,
                "user_email": user_email,
                "logged_in": logged_in,
            });

            if let Err(e) = client.send_training_pair(&payload).await {
                log::error!("Recording send failed (will reconnect): {}", e);
                break; // break inner loop to reconnect
            }
        }

        *state.lock().await = "disconnected".into();
        // Brief pause before reconnecting
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}

/// Screen publisher loop: connects to /ws/screen, waits for start_stream,
/// then captures and sends JPEG frames to the server for relay to iOS subscribers.
/// Reconnects with backoff. Re-reads settings each attempt.
async fn run_screen_publisher_loop(
    settings: Arc<Mutex<AppSettings>>,
    stop_signal: Arc<Notify>,
) -> Result<(), String> {
    loop {
        if should_stop(&stop_signal).await {
            return Ok(());
        }

        // Re-read settings each connection attempt (picks up token after login)
        let (ws_url, token) = {
            let s = settings.lock().await;
            (s.ws_screen_url(), s.auth_token.clone())
        };

        // Skip if not logged in
        if token.is_empty() {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            continue;
        }

        log::info!("Screen publisher: connecting to {}", ws_url);

        let mut client = match WsClient::connect(&ws_url, &token).await {
            Ok(c) => {
                log::info!("Screen publisher WebSocket connected");
                c
            }
            Err(e) => {
                log::warn!("Screen publisher connect failed (will retry): {}", e);
                for _ in 0..10 {
                    if should_stop(&stop_signal).await {
                        return Ok(());
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
                continue;
            }
        };

        // Register as publisher
        if let Err(e) = client.send_json(&json!({"type": "register", "role": "publisher"})).await {
            log::error!("Screen publisher register failed: {}", e);
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            continue;
        }

        log::info!("Screen publisher: registered, waiting for start_stream");
        let mut last_ping = std::time::Instant::now();
        let ping_interval = std::time::Duration::from_secs(30);
        let mut streaming = false;
        let mut capture_interval_ms: u64 = 1000;
        let mut should_reconnect = false;

        // Inner loop: wait for commands, stream when requested
        loop {
            if should_stop(&stop_signal).await {
                client.close().await;
                return Ok(());
            }

            // Keepalive ping
            if last_ping.elapsed() >= ping_interval {
                if let Err(e) = client.send_ping().await {
                    log::warn!("Screen publisher ping failed: {}, reconnecting", e);
                    should_reconnect = true;
                    break;
                }
                last_ping = std::time::Instant::now();
            }

            if streaming {
                // Capture and send a frame
                let screenshot = match capture::capture_screen() {
                    Ok(s) => s,
                    Err(e) => {
                        log::warn!("Screen capture failed: {}", e);
                        tokio::time::sleep(std::time::Duration::from_millis(capture_interval_ms)).await;
                        continue;
                    }
                };

                // Get screen dimensions (reuse window bounds as approximation)
                let (_, _, ww, wh) = platform::focused_window_bounds();

                let frame = json!({
                    "type": "frame",
                    "data": screenshot,
                    "timestamp": now(),
                    "width": ww,
                    "height": wh,
                });

                if let Err(e) = client.send_json(&frame).await {
                    log::error!("Screen publisher frame send failed: {}", e);
                    should_reconnect = true;
                    break;
                }

                // Check for incoming messages (stop_stream, set_interval) without blocking
                match client.try_receive_message(std::time::Duration::from_millis(1)).await {
                    Ok(Some(msg)) => {
                        match msg.get("type").and_then(|t| t.as_str()) {
                            Some("stop_stream") => {
                                log::info!("Screen publisher: stop_stream received, pausing");
                                streaming = false;
                            }
                            Some("set_interval") => {
                                if let Some(ms) = msg.get("interval_ms").and_then(|v| v.as_u64()) {
                                    log::info!("Screen publisher: interval changed to {}ms", ms);
                                    capture_interval_ms = ms.max(100); // minimum 100ms
                                }
                            }
                            _ => {}
                        }
                    }
                    Ok(None) => {} // no message (timeout)
                    Err(e) => {
                        log::warn!("Screen publisher receive error: {}", e);
                        should_reconnect = true;
                        break;
                    }
                }

                // Sleep for the capture interval (minus the time spent capturing)
                tokio::time::sleep(std::time::Duration::from_millis(capture_interval_ms)).await;
            } else {
                // Not streaming — wait for start_stream with ping keepalive
                match client.try_receive_message(std::time::Duration::from_secs(5)).await {
                    Ok(Some(msg)) => {
                        match msg.get("type").and_then(|t| t.as_str()) {
                            Some("start_stream") => {
                                if let Some(ms) = msg.get("interval_ms").and_then(|v| v.as_u64()) {
                                    capture_interval_ms = ms.max(100);
                                }
                                log::info!("Screen publisher: start_stream received, interval={}ms", capture_interval_ms);
                                streaming = true;
                            }
                            Some("set_interval") => {
                                if let Some(ms) = msg.get("interval_ms").and_then(|v| v.as_u64()) {
                                    capture_interval_ms = ms.max(100);
                                }
                            }
                            _ => {}
                        }
                    }
                    Ok(None) => {} // timeout — loop back to check stop & ping
                    Err(e) => {
                        log::warn!("Screen publisher connection lost: {}", e);
                        should_reconnect = true;
                        break;
                    }
                }
            }
        }

        // Connection broke — retry after a brief pause
        if should_reconnect {
            log::info!("Screen publisher: connection lost, reconnecting in 3s...");
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }
}
