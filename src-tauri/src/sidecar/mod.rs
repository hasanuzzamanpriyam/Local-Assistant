use std::process::{Command, Stdio, Child};
use std::io::{Write, BufRead, BufReader};
use std::sync::{Arc, Mutex};
use std::thread;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tauri::Emitter;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarStatus {
    pub running: bool,
    pub ready: bool,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarEvent {
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_steps: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thought: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshot_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fatal: Option<bool>,
}

pub struct SidecarManager {
    child: Mutex<Option<Child>>,
    stdin: Mutex<Option<std::process::ChildStdin>>,
    status: Mutex<SidecarStatus>,
    respawn_count: Mutex<u32>,
}

impl SidecarManager {
    pub fn new() -> Self {
        Self {
            child: Mutex::new(None),
            stdin: Mutex::new(None),
            status: Mutex::new(SidecarStatus { running: false, ready: false, pid: None }),
            respawn_count: Mutex::new(0),
        }
    }

    pub fn is_running(&self) -> bool {
        self.status.lock().unwrap().running
    }

    pub fn status(&self) -> SidecarStatus {
        self.status.lock().unwrap().clone()
    }

    pub fn start(&self) -> Result<()> {
        let mut status = self.status.lock().unwrap();
        if status.running {
            return Ok(());
        }
        drop(status);

        let mut cmd = Command::new("python")
            .arg("-u")
            .arg("sidecar/runner.py")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()?;

        let pid = cmd.id();
        let stdin = cmd.stdin.take().unwrap();
        let stdout = cmd.stdout.take().unwrap();
        let stderr = cmd.stderr.take().unwrap();

        // Spawn stdout reader thread
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        println!("[sidecar-stdout] {}", line);
                    }
                    Err(e) => {
                        eprintln!("[sidecar-stdout] Error reading: {}", e);
                    }
                }
            }
        });

        // Spawn stderr reader thread
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        eprintln!("[sidecar-stderr] {}", line);
                    }
                    Err(e) => {
                        eprintln!("[sidecar-stderr] Error reading: {}", e);
                    }
                }
            }
        });

        let stdin = cmd.stdin.take().unwrap();

        *self.child.lock().unwrap() = Some(cmd);
        *self.stdin.lock().unwrap() = Some(stdin);

        *self.status.lock().unwrap() = SidecarStatus { running: true, ready: false, pid: Some(pid) };

        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        let mut child = self.child.lock().unwrap();
        if let Some(mut c) = child.take() {
            c.kill()?;
            c.wait()?;
        }
        *self.stdin.lock().unwrap() = None;
        *self.status.lock().unwrap() = SidecarStatus { running: false, ready: false, pid: None };
        Ok(())
    }

    pub fn send_command(&self, command: serde_json::Value) -> Result<()> {
        let mut stdin = self.stdin.lock().unwrap();
        if let Some(ref mut s) = *stdin {
            writeln!(s, "{}", serde_json::to_string(&command)?)?;
        }
        Ok(())
    }
}

impl Default for SidecarManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidecar_manager_creates() {
        let manager = SidecarManager::new();
        assert!(!manager.is_running());
    }
}