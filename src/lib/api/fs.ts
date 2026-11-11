import { invoke } from '@tauri-apps/api/core';

export interface GrepMatch {
  path: string;
  line_number: number;
  line: string;
}

export async function fsRead(path: string): Promise<string> {
  return invoke('fs_read', { path });
}

export async function fsWrite(path: string, content: string): Promise<void> {
  return invoke('fs_write', { path, content });
}

export async function fsEdit(path: string, oldText: string, newText: string): Promise<void> {
  return invoke('fs_edit', { path, old_text: oldText, new_text: newText });
}

export async function fsGlob(pattern: string): Promise<string[]> {
  return invoke('fs_glob', { pattern });
}

export async function fsGrep(pattern: string, path?: string): Promise<GrepMatch[]> {
  return invoke('fs_grep', { pattern, path });
}