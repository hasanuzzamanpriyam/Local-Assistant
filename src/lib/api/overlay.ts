import { invoke } from '@tauri-apps/api/core';

export async function overlayShow(x: number, y: number): Promise<void> {
  return invoke('overlay_show', { x, y });
}

export async function overlayHide(): Promise<void> {
  return invoke('overlay_hide', {});
}
