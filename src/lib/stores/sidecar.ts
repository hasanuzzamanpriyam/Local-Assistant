import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export interface SidecarStatus {
  running: boolean;
  ready: boolean;
  pid?: number;
}

export interface TaskEvent {
  id: string;
  event: string;
  step?: number;
  max_steps?: number;
  thought?: string;
  action?: string;
  screenshot_base64?: string;
  result?: string;
  success?: boolean;
  steps?: number;
  duration_ms?: number;
  error?: string;
}

export interface ActiveTask {
  id: string;
  task: string;
  max_steps: number;
  status: 'running' | 'completed' | 'failed' | 'stopped';
  events: TaskEvent[];
}

const sidecarStatus = writable<SidecarStatus>({ running: false, ready: false });
const tasks = writable<ActiveTask[]>([]);
const events = writable<TaskEvent[]>([]);

function createSidecarStore() {
  return {
    subscribe: sidecarStatus.subscribe,
    subscribeTasks: tasks.subscribe,
    subscribeEvents: events.subscribe,

    async status(): Promise<SidecarStatus> {
      return invoke('sidecar_status');
    },

    async start(): Promise<void> {
      await invoke('sidecar_start');
    },

    async stop(): Promise<void> {
      await invoke('sidecar_stop');
    },

    async runTask(task: string, maxSteps = 10): Promise<string> {
      const result: { task_id: string } = await invoke('sidecar_run_task', { task, max_steps: maxSteps });
      return result.task_id;
    },

    async stopTask(taskId: string): Promise<void> {
      await invoke('sidecar_stop_task', { task_id: taskId });
    },

    async sendCommand(command: Record<string, any>): Promise<void> {
      await invoke('sidecar_send_command', { command });
    },

    async listen(callback: (event: TaskEvent) => void): Promise<() => void> {
      try {
        const unlisten = await listen('sidecar-event', (e) => {
          callback(e.payload as TaskEvent);
        });
        return unlisten;
      } catch {
        return () => {};
      }
    },
  };
}

export const sidecarStore = createSidecarStore();
