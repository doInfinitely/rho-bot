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
    use serde_json::Value;
    use tauri::State;

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
