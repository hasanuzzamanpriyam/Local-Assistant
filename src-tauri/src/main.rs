mod config;
mod model;
mod commands;
mod overlay;
mod sidecar;

use std::sync::Arc;
use tauri::Manager;
use crate::model::ModelManager;
use crate::commands::shell::JobRegistry;
use crate::sidecar::SidecarManager;
use crate::overlay::{OverlayManager, GlobalShortcut};

fn main() {
    let config = config::Config::load().expect("Failed to load config");
    
    // Create model directory on startup
    let models_dir = dirs::data_dir()
        .expect("No data dir")
        .join("assistant")
        .join("models");
    std::fs::create_dir_all(&models_dir).expect("Failed to create models directory");
    
    // Create workspace directory on startup
    let workspace_dir = dirs::data_dir()
        .expect("No data dir")
        .join("assistant")
        .join("workspace");
    std::fs::create_dir_all(&workspace_dir).expect("Failed to create workspace directory");

    let model_manager = Arc::new(ModelManager::new(config).expect("Failed to create ModelManager"));
    let job_registry = JobRegistry::new();
    let sidecar_manager = SidecarManager::new();
    let overlay_manager = Arc::new(OverlayManager::new());
    let global_shortcut = Arc::new(GlobalShortcut::new());

    let overlay_clone = overlay_manager.clone();
    let shortcut_handler = move || {
        let _ = overlay_clone.hide();
    };
    global_shortcut.set_handler(Box::new(shortcut_handler));

    tauri::Builder::default()
        .manage(model_manager)
        .manage(job_registry)
        .manage(sidecar_manager)
        .manage(overlay_manager.clone())
        .manage(global_shortcut.clone())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(move |app| {
            let app_handle = app.app_handle();
            if let Some(overlay_window) = app.get_webview_window("overlay") {
                let window = overlay::window::OverlayWindow { window: Some(overlay_window) };
                overlay_manager.set_window(window);
            }
            if let Err(e) = global_shortcut.register(app_handle) {
                eprintln!("Failed to register global shortcut: {}", e);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::model::model_health,
            commands::model::model_list,
            commands::model::model_chat,
            commands::model::model_chat_stream,
            commands::model::model_start,
            commands::model::model_stop,
            commands::fs::fs_read,
            commands::fs::fs_write,
            commands::fs::fs_edit,
            commands::fs::fs_glob,
            commands::fs::fs_grep,
            commands::shell::shell_exec,
            commands::shell::shell_background,
            commands::shell::shell_job_status,
            commands::shell::shell_job_list,
            commands::shell::shell_job_cleanup,
            commands::sidecar::sidecar_status,
            commands::sidecar::sidecar_start,
            commands::sidecar::sidecar_stop,
            commands::sidecar::sidecar_run_task,
            commands::sidecar::sidecar_stop_task,
            commands::sidecar::sidecar_send_command,
            commands::overlay::overlay_show,
            commands::overlay::overlay_hide,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}