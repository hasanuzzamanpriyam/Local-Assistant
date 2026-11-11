use std::path::PathBuf;
use std::sync::Arc;
use std::pin::Pin;
use tokio::sync::Mutex;
use anyhow::Result;
use crate::config::Config;
use futures::Stream;

pub mod server;
pub mod client;

use server::LlamaServer;
use client::LlamaClient;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub downloaded: bool,
}

#[derive(Debug, Clone)]
pub struct ModelManager {
    config: Config,
    server: Arc<Mutex<Option<LlamaServer>>>,
    client: Arc<Mutex<Option<LlamaClient>>>,
    models_dir: PathBuf,
}

impl ModelManager {
    pub fn new(config: Config) -> Result<Self> {
        let models_dir = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("No data dir"))?
            .join("assistant")
            .join("models");
        
        std::fs::create_dir_all(&models_dir)?;

        Ok(Self {
            config,
            server: Arc::new(Mutex::new(None)),
            client: Arc::new(Mutex::new(None)),
            models_dir,
        })
    }

    pub async fn start_server(&self) -> Result<()> {
        let mut server_guard = self.server.lock().await;
        
        if server_guard.is_some() {
            return Ok(());
        }

        let model_path = self.models_dir.join(&self.config.model.default);
        if !model_path.exists() {
            return Err(anyhow::anyhow!("Model not found: {}", model_path.display()));
        }

        let mut server = LlamaServer::new(
            model_path,
            self.config.model.context_size,
            self.config.model.threads,
        ).await?;
        
        server.start().await?;
        server.wait_healthy().await?;

        let client = LlamaClient::new(server.base_url().clone());
        
        *server_guard = Some(server);
        *self.client.lock().await = Some(client);

        Ok(())
    }

    pub async fn stop_server(&self) -> Result<()> {
        let mut server_guard = self.server.lock().await;
        
        if let Some(mut server) = server_guard.take() {
            server.stop().await?;
        }
        
        *self.client.lock().await = None;
        
        Ok(())
    }

    pub async fn is_healthy(&self) -> bool {
        if let Some(client) = self.client.lock().await.as_ref() {
            client.health_check().await.unwrap_or(false)
        } else {
            false
        }
    }

    pub async fn list_models(&self) -> Vec<ModelInfo> {
        let mut models = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir(&self.models_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.ends_with(".gguf") {
                            models.push(ModelInfo {
                                name: name.clone(),
                                path: entry.path(),
                                size: metadata.len(),
                                downloaded: true,
                            });
                        }
                    }
                }
            }
        }
        
        models
    }

    pub async fn chat_completion(
        &self,
        request: client::ChatCompletionRequest,
    ) -> Result<client::ChatCompletionResponse> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Server not running"))?;
        
        client.chat_completion(request).await
    }

    pub async fn chat_completion_stream(
        &self,
        request: client::ChatCompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<client::ChatCompletionChunk>> + Send + 'static>>> {
        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Server not running"))?;
        
        client.chat_completion_stream(request).await
    }

    #[allow(dead_code)]
    pub fn models_dir(&self) -> &PathBuf {
        &self.models_dir
    }
}