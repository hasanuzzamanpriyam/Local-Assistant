import { invoke } from '@tauri-apps/api/core';

export interface SttResult {
  text: string;
}

export interface TtsResult {
  audioBase64: string;
  format: string;
}

export async function stt(audioBase64: string, format: string = 'webm'): Promise<SttResult> {
  return invoke('voice_stt', { audio_base64: audioBase64, format });
}

export async function tts(text: string, voice?: string): Promise<TtsResult> {
  return invoke('voice_tts', { text, voice });
}

export async function sttStart(): Promise<void> {
  return invoke('voice_stt_start');
}

export async function sttStop(): Promise<void> {
  return invoke('voice_stt_stop');
}

export async function ttsStart(): Promise<void> {
  return invoke('voice_tts_start');
}

export async function ttsStop(): Promise<void> {
  return invoke('voice_tts_stop');
}
