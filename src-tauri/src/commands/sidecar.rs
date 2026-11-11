use tauri::{command, State};
use crate::sidecar::SidecarManager;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RunTaskRequest {
    pub task: String,
    #[serde(default = "default_max_steps")]
    pub max_steps: u32,
}

fn default_max_steps() -> u32 { 10 }

#[derive(Serialize)]
pub struct RunTaskResponse {
    pub task_id: String,
}

#[command]
pub fn sidecar_status(manager: State<'_, SidecarManager>) -> crate::sidecar::SidecarStatus {
    manager.status()
}

#[command]
pub fn sidecar_start(manager: State<'_, SidecarManager>) -> Result<(), String> {
    manager.start().map_err(|e| e.to_string())
}

#[command]
pub fn sidecar_stop(manager: State<'_, SidecarManager>) -> Result<(), String> {
    manager.stop().map_err(|e| e.to_string())
}

#[command]
pub fn sidecar_run_task(
    manager: State<'_, SidecarManager>,
    request: RunTaskRequest,
) -> Result<RunTaskResponse, String> {
    let task_id = uuid::Uuid::new_v4().to_string();
    manager.send_command(serde_json::json!({
        "cmd": "run",
        "id": task_id,
        "task": request.task,
        "max_steps": request.max_steps,
    })).map_err(|e| e.to_string())?;
    Ok(RunTaskResponse { task_id })
}

#[command]
pub fn sidecar_stop_task(
    manager: State<'_, SidecarManager>,
    task_id: String,
) -> Result<(), String> {
    manager.send_command(serde_json::json!({
        "cmd": "stop",
        "id": task_id,
    })).map_err(|e| e.to_string())
}

#[command]
pub fn sidecar_send_command(
    manager: State<'_, SidecarManager>,
    command: serde_json::Value,
) -> Result<(), String> {
    manager.send_command(command).map_err(|e| e.to_string())
}
