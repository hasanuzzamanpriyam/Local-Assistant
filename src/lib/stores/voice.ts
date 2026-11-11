import { writable } from 'svelte/store';

export interface VoiceStatus {
  isListening: boolean;
  isSpeaking: boolean;
  transcript: string;
  audioLevel: number;
  sttStatus: 'idle' | 'ready' | 'listening' | 'processing' | 'error';
  ttsStatus: 'idle' | 'ready' | 'speaking' | 'error';
}

const isListening = writable<boolean>(false);
const isSpeaking = writable<boolean>(false);
const transcript = writable<string>('');
const audioLevel = writable<number>(0);
const sttStatus = writable<'idle' | 'ready' | 'listening' | 'processing' | 'error'>('idle');
const ttsStatus = writable<'idle' | 'ready' | 'speaking' | 'error'>('idle');

function createVoiceStore() {
  return {
    subscribe: isListening.subscribe,
    subscribeIsSpeaking: isSpeaking.subscribe,
    subscribeTranscript: transcript.subscribe,
    subscribeAudioLevel: audioLevel.subscribe,
    subscribeSttStatus: sttStatus.subscribe,
    subscribeTtsStatus: ttsStatus.subscribe,

    startListening() {
      isListening.set(true);
      sttStatus.set('listening');
      transcript.set('');
    },

    stopListening() {
      isListening.set(false);
      sttStatus.set('idle');
    },

    startSpeaking() {
      isSpeaking.set(true);
      ttsStatus.set('speaking');
    },

    stopSpeaking() {
      isSpeaking.set(false);
      ttsStatus.set('idle');
    },

    setTranscript(text: string) {
      transcript.set(text);
    },

    setAudioLevel(level: number) {
      audioLevel.set(level);
    },

    setSttStatus(status: 'idle' | 'ready' | 'listening' | 'processing' | 'error') {
      sttStatus.set(status);
    },

    setTtsStatus(status: 'idle' | 'ready' | 'speaking' | 'error') {
      ttsStatus.set(status);
    },
  };
}

export const voiceStore = createVoiceStore();
