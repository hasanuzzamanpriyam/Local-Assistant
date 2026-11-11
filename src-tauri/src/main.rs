mod config;
mod model;
mod commands;
mod overlay;
mod sidecar;

use std::sync::Arc;
use crate::model::ModelManager;
use crate::commands::shell::JobRegistry;
use crate::sidecar::SidecarManager;

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

    tauri::Builder::default()
        .manage(model_manager)
        .manage(job_registry)
        .manage(sidecar_manager)
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}