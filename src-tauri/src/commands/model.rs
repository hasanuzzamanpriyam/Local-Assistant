use crate::model::{ModelManager, ModelInfo};
use crate::model::client::{ChatCompletionRequest, ChatCompletionResponse};
use tauri::{State, command, Emitter};
use futures::StreamExt;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    pub healthy: bool,
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