import { invoke } from '@tauri-apps/api/core';

export interface ShellExecResult {
  stdout: string;
  stderr: string;
  code: number;
}

export type JobStatus = 'Running' | 'Completed' | 'Failed';

export interface JobInfo {
  id: string;
  command: string;
  args: string[];
  status: JobStatus;
  stdout?: string;
  stderr?: string;
  code?: number;
}

/**
 * Execute a shell command synchronously (foreground)
 * @param command - The command to execute (e.g., 'cmd', 'powershell', 'bash')
 * @param args - Arguments to pass to the command
 * @param cwd - Optional working directory (relative to workspace root)
 * @returns ShellExecResult with stdout, stderr, and exit code
 */
export async function shellExec(
  command: string,
  args: string[],
  cwd?: string
): Promise<ShellExecResult> {
  return invoke('shell_exec', { command, args, cwd });
}

/**
 * Execute a shell command in the background
 * @param command - The command to execute (e.g., 'cmd', 'powershell', 'bash')
 * @param args - Arguments to pass to the command
 * @param cwd - Optional working directory (relative to workspace root)
 * @returns JobInfo with job ID and initial status
 */
export async function shellBackground(
  command: string,
  args: string[],
  cwd?: string
): Promise<JobInfo> {
  return invoke('shell_background', { command, args, cwd });
}

/**
 * Get the status of a background job
 * @param jobId - The job ID returned by shellBackground
 * @returns JobInfo or null if job not found
 */
export async function shellJobStatus(jobId: string): Promise<JobInfo | null> {
  return invoke('shell_job_status', { job_id: jobId });
}

/**
 * List all background jobs
 * @returns Array of JobInfo
 */
export async function shellJobList(): Promise<JobInfo[]> {
  return invoke('shell_job_list', {});
}

/**
 * Clean up completed/failed jobs
 * @returns Number of jobs cleaned up
 */
export async function shellJobCleanup(): Promise<number> {
  return invoke('shell_job_cleanup', {});
}

/**
 * Wait for a background job to complete
 * @param jobId - The job ID returned by shellBackground
 * @param pollIntervalMs - How often to poll for status (default: 500ms)
 * @param timeoutMs - Maximum time to wait (default: 60000ms)
 * @returns Final JobInfo when completed or failed
 */
export async function shellJobWait(
  jobId: string,
  pollIntervalMs: number = 500,
  timeoutMs: number = 60000
): Promise<JobInfo> {
  const startTime = Date.now();
  
  while (true) {
    const job = await shellJobStatus(jobId);
    
    if (!job) {
      throw new Error(`Job ${jobId} not found`);
    }
    
    if (job.status === 'Completed' || job.status === 'Failed') {
      return job;
    }
    
    if (Date.now() - startTime > timeoutMs) {
      throw new Error(`Timeout waiting for job ${jobId}`);
    }
    
    await new Promise(resolve => setTimeout(resolve, pollIntervalMs));
  }
}