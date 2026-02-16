mod agent;
mod capture;
mod accessibility;
mod events;
mod event_monitor;
mod executor;
mod platform;
mod settings;
mod ws_client;

use std::sync::Arc;
use tauri::Manager;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::menu::{MenuBuilder, MenuItemBuilder};

use agent::AgentHandle;
use settings::AppSettings;

/// Tauri commands exposed to the React frontend.
mod commands {
    use super::*;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use tauri::State;

    #[derive(Deserialize)]
    pub struct AuthCredentials {
        pub email: String,
        pub password: String,
        pub server_url: Option<String>,
    }

    #[derive(Deserialize)]
    struct TokenResponse {
        access_token: String,
        token_type: String,
    }

    #[derive(Serialize)]
    pub struct AuthResult {
        pub email: String,
        pub token: String,
    }

    #[tauri::command]
    pub async fn login(
        settings_state: State<'_, Arc<tokio::sync::Mutex<AppSettings>>>,
        handle: State<'_, Arc<AgentHandle>>,
        creds: AuthCredentials,
    ) -> Result<AuthResult, String> {
        let base_url = {
            let s = settings_state.lock().await;
            creds.server_url.clone().unwrap_or_else(|| s.server_url.clone())
        };
        let url = format!("{}/auth/login", base_url.trim_end_matches('/'));

        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .json(&serde_json::json!({
                "email": creds.email,
                "password": creds.password,
            }))
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Login failed ({}): {}", status, body));
        }

        let token_resp: TokenResponse = resp
            .json()
            .await
            .map_err(|e| format!("Invalid response: {}", e))?;

        // Persist to settings
        let mut settings = settings_state.lock().await;
        settings.auth_token = token_resp.access_token.clone();
        settings.user_email = creds.email.clone();
        if let Some(ref url) = creds.server_url {
            settings.server_url = url.clone();
        }
        settings.save()?;
        handle.update_settings(settings.clone()).await;

        Ok(AuthResult {
            email: creds.email,
            token: token_resp.access_token,
        })
    }

    #[tauri::command]
    pub async fn signup(
        settings_state: State<'_, Arc<tokio::sync::Mutex<AppSettings>>>,
        handle: State<'_, Arc<AgentHandle>>,
        creds: AuthCredentials,
    ) -> Result<AuthResult, String> {
        let base_url = {
            let s = settings_state.lock().await;
            creds.server_url.clone().unwrap_or_else(|| s.server_url.clone())
        };
        let url = format!("{}/auth/signup", base_url.trim_end_matches('/'));

        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .json(&serde_json::json!({
                "email": creds.email,
                "password": creds.password,
            }))
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Signup failed ({}): {}", status, body));
        }

        let token_resp: TokenResponse = resp
            .json()
            .await
            .map_err(|e| format!("Invalid response: {}", e))?;

        // Persist to settings
        let mut settings = settings_state.lock().await;
        settings.auth_token = token_resp.access_token.clone();
        settings.user_email = creds.email.clone();
        if let Some(ref url) = creds.server_url {
            settings.server_url = url.clone();
        }
        settings.save()?;
        handle.update_settings(settings.clone()).await;

        Ok(AuthResult {
            email: creds.email,
            token: token_resp.access_token,
        })
    }

    #[tauri::command]
    pub async fn logout(
        settings_state: State<'_, Arc<tokio::sync::Mutex<AppSettings>>>,
        handle: State<'_, Arc<AgentHandle>>,
    ) -> Result<(), String> {
        handle.stop().await;

        let mut settings = settings_state.lock().await;
        settings.auth_token = String::new();
        settings.user_email = String::new();
        settings.save()?;
        handle.update_settings(settings.clone()).await;

        Ok(())
    }

    #[tauri::command]
    pub async fn get_auth_state(
        settings: State<'_, Arc<tokio::sync::Mutex<AppSettings>>>,
    ) -> Result<Value, String> {
        let s = settings.lock().await;
        Ok(serde_json::json!({
            "logged_in": s.is_logged_in(),
            "email": s.user_email,
            "server_url": s.server_url,
        }))
    }

    #[tauri::command]
    pub async fn get_agent_state(handle: State<'_, Arc<AgentHandle>>) -> Result<String, String> {
        Ok(handle.state().await)
    }

    #[tauri::command]
    pub async fn start_agent(handle: State<'_, Arc<AgentHandle>>) -> Result<(), String> {
        handle.start().await.map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub async fn stop_agent(handle: State<'_, Arc<AgentHandle>>) -> Result<(), String> {
        handle.stop().await;
        Ok(())
    }

    #[tauri::command]
    pub async fn start_recording(handle: State<'_, Arc<AgentHandle>>) -> Result<(), String> {
        handle.start_recording().await.map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub async fn stop_recording(handle: State<'_, Arc<AgentHandle>>) -> Result<(), String> {
        handle.stop_recording().await;
        Ok(())
    }

    #[tauri::command]
    pub async fn get_recent_actions(
        handle: State<'_, Arc<AgentHandle>>,
    ) -> Result<Vec<Value>, String> {
        Ok(handle.recent_actions().await)
    }

    #[tauri::command]
    pub async fn get_settings(
        settings: State<'_, Arc<tokio::sync::Mutex<AppSettings>>>,
    ) -> Result<AppSettings, String> {
        Ok(settings.lock().await.clone())
    }

    #[tauri::command]
    pub async fn save_settings(
        settings_state: State<'_, Arc<tokio::sync::Mutex<AppSettings>>>,
        handle: State<'_, Arc<AgentHandle>>,
        settings: AppSettings,
    ) -> Result<(), String> {
        settings.save()?;
        let mut current = settings_state.lock().await;
        *current = settings.clone();
        handle.update_settings(settings).await;
        Ok(())
    }
}

pub fn run() {
    env_logger::init();

    let loaded_settings = settings::AppSettings::load().unwrap_or_default();
    let settings = Arc::new(tokio::sync::Mutex::new(loaded_settings));
    let agent = Arc::new(AgentHandle::new());

    let agent_for_setup = agent.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(move |app| {
            // Start global event monitor on a background thread.
            // This is safe even if Accessibility permissions aren't granted yet.
            event_monitor::start_event_monitor(agent_for_setup.event_buffer());

            // Build tray menu
            let quit = MenuItemBuilder::with_id("quit", "Quit rho-bot").build(app)?;
            let show = MenuItemBuilder::with_id("show", "Show Window").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&show, &quit]).build()?;

            // Build tray icon using the bundled icon
            let icon = app.default_window_icon().cloned();
            let mut builder = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("rho-bot agent")
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "quit" => std::process::exit(0),
                        "show" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                });

            if let Some(icon) = icon {
                builder = builder.icon(icon);
            }

            builder.build(app)?;

            Ok(())
        })
        .manage(settings)
        .manage(agent)
        .invoke_handler(tauri::generate_handler![
            commands::login,
            commands::signup,
            commands::logout,
            commands::get_auth_state,
            commands::get_agent_state,
            commands::start_agent,
            commands::stop_agent,
            commands::start_recording,
            commands::stop_recording,
            commands::get_recent_actions,
            commands::get_settings,
            commands::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
