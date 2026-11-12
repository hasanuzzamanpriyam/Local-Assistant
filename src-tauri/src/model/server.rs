use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::time::{sleep, timeout};
use reqwest::Client;
use url::Url;
use anyhow::Result;

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
    // Fixed port for llama-server (standard llama.cpp port)
    const LLAMA_SERVER_PORT: u16 = 11434;

    pub async fn new(
        model_path: PathBuf,
        context_size: u32,
        threads: u32,
        config_llama_server_path: Option<String>,
    ) -> Result<Self> {
        let port = Self::LLAMA_SERVER_PORT;
        
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
        
        // Get model path
        let model_path_str = self.model_path.to_string_lossy().to_string();
        
        // Build the exact command that works
        let mut cmd = Command::new(&llama_server_path);
        cmd.arg("-m").arg(&model_path_str)
            .arg("--port").arg(self.port.to_string())
            .arg("--host").arg("127.0.0.1")
            .arg("--ctx-size").arg(self.context_size.to_string())
            .arg("--threads").arg(self.threads.to_string())
            // Critical: remove --mlock as it causes model loading to fail
            .arg("--n-gpu-layers").arg("0")
            // Note: --mlock REMOVED - causes model loading to fail
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Log the exact command for debugging
        let cmd_debug = format!("{} -m \"{}\" --port {} --host 127.0.0.1 --ctx-size {} --threads {} --n-gpu-layers 0", 
            llama_server_path, model_path_str, self.port, self.context_size, self.threads);
        tracing::info!("Starting llama-server: {}", cmd_debug);
        
        // EXPLICIT DEBUG: Print before spawn
        eprintln!("[LlamaServer] Starting llama-server with command: {}", cmd_debug);
        eprintln!("[LlamaServer] Model path: {}", model_path_str);
        eprintln!("[LlamaServer] llama-server path: {}", llama_server_path);
        eprintln!("[LlamaServer] Using fixed port: {}", self.port);
        
        let child = cmd.spawn()?;
        self.child = Some(child);

        tracing::info!("LlamaServer started on port {} with model {}", self.port, self.model_path.display());
        eprintln!("[LlamaServer] Spawned successfully, waiting for health check...");
        
        Ok(())
    }

    /// Find the llama-server binary from config path, common locations, or PATH
    fn find_llama_server(&self) -> Result<String> {
        // 1. Direct known path (user's desktop build)
        let direct = r"C:\Users\CT\Desktop\llama-b12289-bin-win-cpu-x64\llama-server.exe";
        if std::path::Path::new(direct).is_file() {
            return Ok(direct.to_string());
        }

        // 2. Check config path - could be a file or a directory containing llama-server.exe
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

        // 3. Check common Windows locations
        if cfg!(target_os = "windows") {
            let common_paths = [
                "C:\\llama.cpp\\llama-server.exe",
                "C:\\Program Files\\llama.cpp\\llama-server.exe",
                "C:\\tools\\llama-server.exe",
                "llama-server.exe",
            ];
            for path in &common_paths {
                if std::path::Path::new(path).is_file() {
                    return Ok(path.to_string());
                }
            }
        }

        Err(anyhow::anyhow!(
            "llama-server binary not found. Please install llama.cpp and set llama_server_path in config.toml"
        ))
    }

    pub async fn wait_healthy(&self) -> Result<()> {
        let max_wait = Duration::from_secs(120);
        let interval = Duration::from_millis(500);
        
        let start = std::time::Instant::now();
        
        while start.elapsed() < max_wait {
            if self.health_check().await? {
                return Ok(());
            }
            sleep(interval).await;
        }
        
        Err(anyhow::anyhow!("Health check timeout after 120s"))
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