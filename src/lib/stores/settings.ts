import { writable, derived } from 'svelte/store';
import type { ModelInfo } from '../api/llama';

// Settings interface matching config.toml ModelConfig plus system prompt
export interface Settings {
  model: string;
  systemPrompt: string;
  temperature: number;
  topP: number;
  contextSize: number;
}

// Default settings
const DEFAULT_SETTINGS: Settings = {
  model: 'Qwen2.5-3B-Instruct-Q4_K_M.gguf',
  systemPrompt: 'You are a helpful AI assistant.',
  temperature: 0.3,
  topP: 0.9,
  contextSize: 4096,
};

const STORAGE_KEY = 'assistant_settings';

// Load settings from localStorage
function loadSettings(): Settings {
  if (typeof localStorage !== 'undefined') {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        const parsed = JSON.parse(stored);
        return { ...DEFAULT_SETTINGS, ...parsed };
      }
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  }
  return DEFAULT_SETTINGS;
}

// Create the settings store
function createSettingsStore() {
  const initialSettings = loadSettings();
  
  const { subscribe, set, update } = writable<Settings>(initialSettings);

  // Persist to localStorage whenever settings change
  subscribe(($settings) => {
    if (typeof localStorage !== 'undefined') {
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify($settings));
      } catch (e) {
        console.error('Failed to save settings:', e);
      }
    }
  });

  return {
    subscribe,
    set,
    update,
    
    // Update a single setting
    updateSetting<K extends keyof Settings>(key: K, value: Settings[K]) {
      update(($settings) => ({ ...$settings, [key]: value }));
    },
    
    // Reset to defaults
    reset() {
      set(DEFAULT_SETTINGS);
    },
  };
}

export const settingsStore = createSettingsStore();

// Derived store for available local models
export const localModels = writable<ModelInfo[]>([]);

// Derived store for available remote models
export const remoteModels = writable<string[]>([]);

// Derived store for all available models (local + remote)
export const allModels = derived(
  [localModels, remoteModels, settingsStore],
  ([$localModels, $remoteModels, $settings]) => {
    const models: { name: string; source: 'local' | 'remote'; selected: boolean }[] = [];
    
    for (const model of $localModels) {
      models.push({
        name: model.name,
        source: 'local',
        selected: model.name === $settings.model,
      });
    }
    
    for (const model of $remoteModels) {
      // Don't duplicate if already in local
      if (!$localModels.some((m) => m.name === model)) {
        models.push({
          name: model,
          source: 'remote',
          selected: model === $settings.model,
        });
      }
    }
    
    return models;
  }
);