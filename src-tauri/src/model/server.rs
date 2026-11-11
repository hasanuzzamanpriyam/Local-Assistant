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
    config_llama_server_path: Option<String>,
}

impl LlamaServer {
    pub async fn new(
        model_path: PathBuf,
        context_size: u32,
        threads: u32,
        config_llama_server_path: Option<String>,
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
            config_llama_server_path,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        // Find the llama-server binary
        let llama_server_path = self.find_llama_server()?;
        
        // Spawn the actual llama.cpp server binary
        let mut cmd = Command::new(llama_server_path);
        cmd.arg("-m").arg(self.model_path.to_string_lossy().to_string())
            .arg("--port").arg(self.port.to_string())
            .arg("--host").arg("127.0.0.1")
            .arg("--ctx-size").arg(self.context_size.to_string())
            .arg("--threads").arg(self.threads.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let child = cmd.spawn()?;
        self.child = Some(child);

        tracing::info!("LlamaServer started on port {} with model {}", self.port, self.model_path.display());
        
        Ok(())
    }

    /// Find the llama-server binary from config path, common locations, or PATH
    fn find_llama_server(&self) -> Result<String> {
        // 1. Check config path - could be a file or a directory containing llama-server.exe
        if let Some(configured_path) = &self.config_llama_server_path {
            let p = std::path::Path::new(configured_path);
            if p.is_file() {
                return Ok(configured_path.clone());
            }
            if p.is_dir() {
                let exe_path = p.join("llama-server.exe");
                if exe_path.is_file() {
                    return Ok(exe_path.to_string_lossy().to_string());
                }
            }
        }

        // 2. Check common Windows locations (file or directory)
        if cfg!(target_os = "windows") {
            let common_paths = [
                "C:\\llama.cpp\\llama-server.exe",
                "C:\\Program Files\\llama.cpp\\llama-server.exe",
                "C:\\tools\\llama-server.exe",
                "C:\\Users\\CT\\Desktop\\llama-b12289-bin-win-cpu-x64\\llama-server.exe",
                "llama-server.exe",
            ];
            for path in &common_paths {
                if std::path::Path::new(path).is_file() {
                    return Ok(path.to_string());
                }
            }
            // Also check directory-based paths
            let common_dirs = [
                "C:\\llama.cpp",
                "C:\\Program Files\\llama.cpp",
                "C:\\tools",
                "C:\\Users\\CT\\Desktop\\llama-b12289-bin-win-cpu-x64",
            ];
            for dir in &common_dirs {
                let exe_path = std::path::Path::new(dir).join("llama-server.exe");
                if exe_path.is_file() {
                    return Ok(exe_path.to_string_lossy().to_string());
                }
            }
        }

        // 3. Try PATH
        Ok("llama-server".to_string())
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