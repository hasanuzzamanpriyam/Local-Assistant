import { invoke } from '@tauri-apps/api/core';

export interface SidecarStatus {
  running: boolean;
  ready: boolean;
  pid?: number;
}

export interface RunTaskResponse {
  task_id: string;
}

export async function sidecarStatus(): Promise<SidecarStatus> {
  return invoke('sidecar_status');
}

export async function sidecarStart(): Promise<void> {
  return invoke('sidecar_start');
}

export async function sidecarStop(): Promise<void> {
  return invoke('sidecar_stop');
}

export async function sidecarRunTask(task: string, maxSteps?: number): Promise<RunTaskResponse> {
  return invoke('sidecar_run_task', { task, max_steps: maxSteps ?? 10 });
}

export async function sidecarStopTask(taskId: string): Promise<void> {
  return invoke('sidecar_stop_task', { task_id: taskId });
}
