use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::time::{sleep, timeout};
use reqwest::Client;
use url::Url;
use anyhow::Result;
use portpicker;

#[derive(Debug)]
pub struct LlamaServer {
    model_path: PathBuf,
    context_size: u32,
    threads: u32,
    port: u16,
    base_url: Url,
    child: Option<Child>,
    client: Client,
}

impl LlamaServer {
    pub async fn new(
        model_path: PathBuf,
        context_size: u32,
        threads: u32,
    ) -> Result<Self> {
        let port = portpicker::pick_unused_port()
            .ok_or_else(|| anyhow::anyhow!("Failed to pick unused port"))?;
        
        let base_url = Url::parse(&format!("http://127.0.0.1:{}/v1", port))?;
        
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;

        Ok(Self {
            model_path,
            context_size,
            threads,
            port,
            base_url,
            child: None,
            client,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        // Note: In production, this would use the llama.cpp server binary
        // For now, we'll simulate the server start
        // The actual llama.cpp binary is not bundled yet
        
        let mut cmd = Command::new("echo");
        cmd.arg("llama.cpp server would start here")
            .arg("--model").arg(self.model_path.to_string_lossy().to_string())
            .arg("--port").arg(self.port.to_string())
            .arg("--ctx-size").arg(self.context_size.to_string())
            .arg("--threads").arg(self.threads.to_string())
            .arg("--host").arg("127.0.0.1")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // In real implementation, this would be:
        // let child = cmd.spawn()?;
        // self.child = Some(child);
        
        // For now, simulate by not actually spawning
        // This is a placeholder until llama.cpp binary is bundled
        
        tracing::info!("LlamaServer would start on port {} with model {}", self.port, self.model_path.display());
        
        Ok(())
    }

    pub async fn wait_healthy(&self) -> Result<()> {
        let max_wait = Duration::from_secs(15);
        let interval = Duration::from_millis(500);
        
        let start = std::time::Instant::now();
        
        while start.elapsed() < max_wait {
            if self.health_check().await? {
                return Ok(());
            }
            sleep(interval).await;
        }
        
        Err(anyhow::anyhow!("Health check timeout after 15s"))
    }

    pub async fn health_check(&self) -> Result<bool> {
        let url = self.base_url.join("/health")?;
        
        match timeout(Duration::from_secs(2), self.client.get(url).send()).await {
            Ok(Ok(resp)) => Ok(resp.status().is_success()),
            _ => Ok(false),
        }
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            child.kill().await?;
            child.wait().await?;
        }
        Ok(())
    }

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    #[allow(dead_code)]
    pub fn port(&self) -> u16 {
        self.port
    }
}