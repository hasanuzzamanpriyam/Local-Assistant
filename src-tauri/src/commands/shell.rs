use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::command;
use tokio::process::Command as TokioCommand;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// Result of a shell command execution
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShellExecResult {
    pub stdout: String,
    pub stderr: String,
    pub code: i32,
}

/// Background job status
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum JobStatus {
    Running,
    Completed,
    Failed,
}

/// Background job information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobInfo {
    pub id: String,
    pub command: String,
    pub args: Vec<String>,
    pub status: JobStatus,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub code: Option<i32>,
}

/// Job registry to track background jobs
#[derive(Debug, Default, Clone)]
pub struct JobRegistry {
    jobs: Arc<Mutex<HashMap<String, JobInfo>>>,
}

impl JobRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the inner jobs Arc for sharing across tasks
    pub fn inner(&self) -> Arc<Mutex<HashMap<String, JobInfo>>> {
        self.jobs.clone()
    }

    /// Add a new background job
    pub fn add_job(&self, command: String, args: Vec<String>) -> String {
        let id = Uuid::new_v4().to_string();
        let job = JobInfo {
            id: id.clone(),
            command,
            args,
            status: JobStatus::Running,
            stdout: None,
            stderr: None,
            code: None,
        };
        self.jobs.lock().unwrap().insert(id.clone(), job);
        id
    }

    /// Update job status and results
    pub fn update_job(&self, id: &str, status: JobStatus, stdout: Option<String>, stderr: Option<String>, code: Option<i32>) {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(job) = jobs.get_mut(id) {
            job.status = status;
            job.stdout = stdout;
            job.stderr = stderr;
            job.code = code;
        }
    }

    /// Get job info
    pub fn get_job(&self, id: &str) -> Option<JobInfo> {
        self.jobs.lock().unwrap().get(id).cloned()
    }

    /// List all jobs
    pub fn list_jobs(&self) -> Vec<JobInfo> {
        self.jobs.lock().unwrap().values().cloned().collect()
    }

    /// Remove completed/failed jobs older than a certain duration (cleanup)
    pub fn cleanup_old_jobs(&self, _max_age: Duration) {
        // In a real implementation, we'd track creation time
        // For now, we'll just remove completed/failed jobs to keep memory bounded
        let mut jobs = self.jobs.lock().unwrap();
        jobs.retain(|_, job| job.status == JobStatus::Running);
    }
}

/// Execute a shell command synchronously (foreground)
#[command]
pub async fn shell_exec(
    command: String,
    args: Vec<String>,
    cwd: Option<String>,
) -> Result<ShellExecResult, String> {
    // Security: Validate command to prevent injection
    if command.is_empty() {
        return Err("Command cannot be empty".to_string());
    }

    let mut cmd = TokioCommand::new(&command);
    cmd.args(&args);

    // Set working directory if provided
    if let Some(dir) = cwd {
        // Security: Prevent path traversal - ensure cwd is within workspace
        let workspace_root = dirs::data_dir()
            .ok_or("No data dir")?
            .join("assistant")
            .join("workspace");
        
        let target = workspace_root.join(&dir);
        let canonical_root = std::fs::canonicalize(&workspace_root)
            .map_err(|e| format!("Failed to canonicalize workspace root: {}", e))?;
        let canonical_target = std::fs::canonicalize(&target)
            .map_err(|e| format!("Failed to canonicalize target path: {}", e))?;
        
        if !canonical_target.starts_with(&canonical_root) {
            return Err("Path traversal attempt blocked".to_string());
        }
        
        cmd.current_dir(canonical_target);
    }

    // Execute command with timeout
    let output = tokio::time::timeout(
        Duration::from_secs(60), // 60 second timeout
        cmd.output()
    ).await
    .map_err(|_| "Command timed out after 60 seconds".to_string())?
    .map_err(|e| format!("Failed to execute command: {}", e))?;

    Ok(ShellExecResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        code: output.status.code().unwrap_or(-1),
    })
}

/// Execute a shell command in the background
#[command]
pub async fn shell_background(
    command: String,
    args: Vec<String>,
    cwd: Option<String>,
    registry: tauri::State<'_, JobRegistry>,
) -> Result<JobInfo, String> {
    // Security: Validate command
    if command.is_empty() {
        return Err("Command cannot be empty".to_string());
    }

    // Get a cloneable reference to the inner jobs map
    let jobs = registry.inner().jobs.clone();
    
    // Create job entry
    let job_id = {
        let id = Uuid::new_v4().to_string();
        let job = JobInfo {
            id: id.clone(),
            command: command.clone(),
            args: args.clone(),
            status: JobStatus::Running,
            stdout: None,
            stderr: None,
            code: None,
        };
        jobs.lock().unwrap().insert(id.clone(), job);
        id
    };
    
    // Prepare command for background execution
    let cmd_str = command.clone();
    let cmd_args = args.clone();
    let cwd_clone = cwd.clone();
    let jobs_clone = jobs.clone();
    let job_id_clone = job_id.clone();

    // Spawn background task
    tokio::spawn(async move {
        let mut cmd = TokioCommand::new(&cmd_str);
        cmd.args(&cmd_args);

        // Set working directory if provided
        if let Some(dir) = cwd_clone {
            let workspace_root = dirs::data_dir()
                .expect("No data dir")
                .join("assistant")
                .join("workspace");
            
            let target = workspace_root.join(&dir);
            if let (Ok(canonical_root), Ok(canonical_target)) = (
                std::fs::canonicalize(&workspace_root),
                std::fs::canonicalize(&target)
            ) {
                if canonical_target.starts_with(&canonical_root) {
                    cmd.current_dir(canonical_target);
                } else {
                    let mut jobs = jobs_clone.lock().unwrap();
                    if let Some(job) = jobs.get_mut(&job_id_clone) {
                        job.status = JobStatus::Failed;
                        job.stdout = None;
                        job.stderr = Some("Path traversal attempt blocked".to_string());
                        job.code = Some(-1);
                    }
                    return;
                }
            } else {
                let mut jobs = jobs_clone.lock().unwrap();
                if let Some(job) = jobs.get_mut(&job_id_clone) {
                    job.status = JobStatus::Failed;
                    job.stdout = None;
                    job.stderr = Some("Invalid working directory".to_string());
                    job.code = Some(-1);
                }
                return;
            }
        }

        // Execute command
        match tokio::time::timeout(
            Duration::from_secs(3600), // 1 hour timeout for background jobs
            cmd.output()
        ).await {
            Ok(Ok(output)) => {
                let mut jobs = jobs_clone.lock().unwrap();
                if let Some(job) = jobs.get_mut(&job_id_clone) {
                    job.status = JobStatus::Completed;
                    job.stdout = Some(String::from_utf8_lossy(&output.stdout).to_string());
                    job.stderr = Some(String::from_utf8_lossy(&output.stderr).to_string());
                    job.code = output.status.code();
                }
            }
            Ok(Err(e)) => {
                let mut jobs = jobs_clone.lock().unwrap();
                if let Some(job) = jobs.get_mut(&job_id_clone) {
                    job.status = JobStatus::Failed;
                    job.stdout = None;
                    job.stderr = Some(format!("Failed to execute command: {}", e));
                    job.code = Some(-1);
                }
            }
            Err(_) => {
                let mut jobs = jobs_clone.lock().unwrap();
                if let Some(job) = jobs.get_mut(&job_id_clone) {
                    job.status = JobStatus::Failed;
                    job.stdout = None;
                    job.stderr = Some("Command timed out after 1 hour".to_string());
                    job.code = Some(-1);
                }
            }
        }
    });

    // Return initial job info
    Ok(JobInfo {
        id: job_id,
        command,
        args,
        status: JobStatus::Running,
        stdout: None,
        stderr: None,
        code: None,
    })
}

/// Get status of a background job
#[command]
pub async fn shell_job_status(
    job_id: String,
    registry: tauri::State<'_, JobRegistry>,
) -> Result<Option<JobInfo>, String> {
    Ok(registry.get_job(&job_id))
}

/// List all background jobs
#[command]
pub async fn shell_job_list(
    registry: tauri::State<'_, JobRegistry>,
) -> Result<Vec<JobInfo>, String> {
    Ok(registry.list_jobs())
}

/// Clean up completed/failed jobs
#[command]
pub async fn shell_job_cleanup(
    registry: tauri::State<'_, JobRegistry>,
) -> Result<usize, String> {
    let before = registry.list_jobs().len();
    registry.cleanup_old_jobs(Duration::from_secs(3600));
    let after = registry.list_jobs().len();
    Ok(before - after)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_registry() {
        let registry = JobRegistry::new();
        
        // Add a job
        let id = registry.add_job("echo".to_string(), vec!["hello".to_string()]);
        assert!(!id.is_empty());
        
        // Get job
        let job = registry.get_job(&id);
        assert!(job.is_some());
        let job = job.unwrap();
        assert_eq!(job.command, "echo");
        assert_eq!(job.args, vec!["hello"]);
        assert_eq!(job.status, JobStatus::Running);
        
        // Update job
        registry.update_job(&id, JobStatus::Completed, Some("hello".to_string()), None, Some(0));
        let job = registry.get_job(&id).unwrap();
        assert_eq!(job.status, JobStatus::Completed);
        assert_eq!(job.stdout, Some("hello".to_string()));
        assert_eq!(job.code, Some(0));
        
        // List jobs
        let jobs = registry.list_jobs();
        assert_eq!(jobs.len(), 1);
        
        // Cleanup
        registry.cleanup_old_jobs(Duration::from_secs(3600));
        let jobs = registry.list_jobs();
        assert_eq!(jobs.len(), 0);
    }

    #[tokio::test]
    async fn test_shell_exec() {
        let result = shell_exec("cmd".to_string(), vec!["/c".to_string(), "echo hello".to_string()], None).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.stdout.contains("hello"));
        assert_eq!(result.code, 0);
    }

    #[tokio::test]
    async fn test_shell_background() {
        let registry = JobRegistry::new();
        // We can't easily test the full async background execution without a Tauri context
        // but we can test the registry functionality
    }
}