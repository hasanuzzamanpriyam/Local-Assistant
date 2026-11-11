use crate::model::{ModelManager, ModelInfo};
use crate::model::client::{ChatCompletionRequest, ChatCompletionResponse};
use tauri::{State, command, Emitter};
use futures::StreamExt;
use serde::{Serialize, Deserialize};

#[derive(Serialize)]
pub struct HealthResponse {
    pub healthy: bool,
}

#[derive(Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ModelScanResult {
    pub local: Vec<ModelInfo>,
    pub remote: Vec<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct SetDefaultModelRequest {
    pub model_name: String,
}

#[command]
pub async fn model_health(manager: State<'_, ModelManager>) -> Result<HealthResponse, String> {
    let healthy = manager.is_healthy().await;
    Ok(HealthResponse { healthy })
}

#[command]
pub async fn model_list(manager: State<'_, ModelManager>) -> Result<Vec<ModelInfo>, String> {
    let models = manager.list_models().await;
    Ok(models)
}

#[command]
#[allow(dead_code)]
pub async fn model_scan_local(manager: State<'_, ModelManager>) -> Result<ModelScanResult, String> {
    // Get local models from filesystem
    let local_models = manager.list_models().await;
    
    // Get remote models from llama.cpp server if running
    let remote_models = manager.list_remote_models().await;
    
    Ok(ModelScanResult {
        local: local_models,
        remote: remote_models,
    })
}

#[command]
#[allow(dead_code)]
pub async fn model_set_default(
    manager: State<'_, ModelManager>,
    request: SetDefaultModelRequest,
) -> Result<(), String> {
    manager.set_default_model(request.model_name).await.map_err(|e| e.to_string())
}

#[command]
pub async fn model_chat(
    manager: State<'_, ModelManager>,
    request: ChatCompletionRequest,
) -> Result<ChatCompletionResponse, String> {
    manager.chat_completion(request)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn model_chat_stream(
    manager: State<'_, ModelManager>,
    request: ChatCompletionRequest,
    window: tauri::Window,
) -> Result<(), String> {
    let stream = manager.chat_completion_stream(request)
        .await
        .map_err(|e| e.to_string())?;
    
    let mut stream = Box::pin(stream);
    
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(chunk) => {
                window.emit("model-chat-chunk", &chunk).map_err(|e| e.to_string())?;
            }
            Err(e) => {
                return Err(e.to_string());
            }
        }
    }
    
    window.emit("model-chat-done", ()).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[command]
pub async fn model_start(manager: State<'_, ModelManager>) -> Result<(), String> {
    manager.start_server().await.map_err(|e| e.to_string())
}

#[command]
pub async fn model_stop(manager: State<'_, ModelManager>) -> Result<(), String> {
    manager.stop_server().await.map_err(|e| e.to_string())
}