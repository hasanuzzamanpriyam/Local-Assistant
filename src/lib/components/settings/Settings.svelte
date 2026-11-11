<script lang="ts">
  import { settingsStore } from '../../stores/settings';
  import ModelSelector from './ModelSelector.svelte';
  import { onMount } from 'svelte';
  import { modelScanLocal } from '../../api/llama';

  let { isOpen = false, onClose = () => {} } = $props();

  let activeTab = $state<'model' | 'parameters' | 'system'>('model');
  let localSystemPrompt = $state('');
  let localTemperature = $state(0.3);
  let localTopP = $state(0.9);
  let localContextSize = $state(4096);

  // Sync local state with store
  function syncFromStore() {
    const s = settingsStoreValue;
    localSystemPrompt = s.systemPrompt;
    localTemperature = s.temperature;
    localTopP = s.topP;
    localContextSize = s.contextSize;
  }

  // Store value reference for reactive access
  let settingsStoreValue: any;
  settingsStore.subscribe((val) => { settingsStoreValue = val; });
  
  // Sync when store changes
  $effect(() => {
    syncFromStore();
  });

  function handleOverlayClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      onClose();
    }
  }

  function updateSetting<K extends keyof typeof settingsStoreValue>(key: K, value: any) {
    settingsStore.updateSetting(key, value);
  }

  function saveSystemPrompt() {
    updateSetting('systemPrompt', localSystemPrompt);
  }

  function saveTemperature() {
    updateSetting('temperature', localTemperature);
  }

  function saveTopP() {
    updateSetting('topP', localTopP);
  }

  function saveContextSize() {
    updateSetting('contextSize', localContextSize);
  }

  // Scan models when panel opens
  $effect(() => {
    if (isOpen) {
      modelScanLocal().catch(console.error);
    }
  });
</script>

{#if isOpen}
  <div 
    class="settings-overlay" 
    onclick={handleOverlayClick}
    onkeydown={handleKeyDown}
    role="dialog"
    aria-modal="true"
    aria-labelledby="settings-title"
    tabindex="-1"
  >
    <div class="settings-panel">
      <header class="settings-header">
        <h2 id="settings-title">Settings</h2>
        <button class="close-btn" onclick={onClose} aria-label="Close settings">×</button>
      </header>

      <div class="settings-tabs" role="tablist">
        <button 
          class="tab {activeTab === 'model' ? 'active' : ''}" 
          role="tab" 
          aria-selected={activeTab === 'model'}
          onclick={() => activeTab = 'model'}
        >Model</button>
        <button 
          class="tab {activeTab === 'parameters' ? 'active' : ''}" 
          role="tab" 
          aria-selected={activeTab === 'parameters'}
          onclick={() => activeTab = 'parameters'}
        >Parameters</button>
        <button 
          class="tab {activeTab === 'system' ? 'active' : ''}" 
          role="tab" 
          aria-selected={activeTab === 'system'}
          onclick={() => activeTab = 'system'}
        >System Prompt</button>
      </div>

      {#if activeTab === 'model'}
        <div class="settings-content">
          <div class="tab-content">
            <ModelSelector onClose={onClose} />
          </div>
        </div>
      {:else if activeTab === 'parameters'}
        <div class="settings-content">
          <div class="tab-content">
            <div class="parameters-panel">
              <div class="param-group">
                <label for="temperature">Temperature</label>
                <div class="slider-container">
                  <input
                    type="range"
                    id="temperature"
                    min="0"
                    max="2"
                    step="0.1"
                    bind:value={localTemperature}
                    onchange={saveTemperature}
                    aria-label="Temperature"
                  >
                  <span class="slider-value">{localTemperature.toFixed(1)}</span>
                </div>
                <p class="param-description">Controls randomness. Lower = more focused, Higher = more creative.</p>
              </div>

              <div class="param-group">
                <label for="top-p">Top P</label>
                <div class="slider-container">
                  <input
                    type="range"
                    id="top-p"
                    min="0"
                    max="1"
                    step="0.05"
                    bind:value={localTopP}
                    onchange={saveTopP}
                    aria-label="Top P"
                  >
                  <span class="slider-value">{localTopP.toFixed(2)}</span>
                </div>
                <p class="param-description">Nucleus sampling. Limits token selection to top probability mass.</p>
              </div>

              <div class="param-group">
                <label for="context-size">Context Size</label>
                <div class="slider-container">
                  <input
                    type="range"
                    id="context-size"
                    min="512"
                    max="32768"
                    step="512"
                    bind:value={localContextSize}
                    onchange={saveContextSize}
                    aria-label="Context Size"
                  >
                  <span class="slider-value">{localContextSize.toLocaleString()}</span>
                </div>
                <p class="param-description">Maximum tokens in context window. Requires model restart to take effect.</p>
              </div>

              <div class="param-note">
                Note: Context size changes require restarting the model.
              </div>
            </div>
          </div>
        </div>
      {:else if activeTab === 'system'}
        <div class="settings-content">
          <div class="tab-content">
            <div class="system-prompt-panel">
              <label for="system-prompt">System Prompt</label>
              <textarea
                id="system-prompt"
                bind:value={localSystemPrompt}
                onblur={saveSystemPrompt}
                placeholder="Enter system prompt..."
                rows="12"
                aria-label="System prompt"
              ></textarea>
              <p class="param-description">Instructions that guide the model's behavior for all conversations.</p>
              <div class="system-prompt-actions">
                <button class="secondary-btn" onclick={() => { localSystemPrompt = 'You are a helpful AI assistant.'; saveSystemPrompt(); }}>Reset to Default</button>
                <button class="primary-btn" onclick={saveSystemPrompt}>Save</button>
              </div>
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .settings-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    animation: fadeIn 0.15s ease-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .settings-panel {
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg, 12px);
    width: 90%;
    max-width: 560px;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: slideUp 0.2s ease-out;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
  }

  @keyframes slideUp {
    from { 
      opacity: 0;
      transform: translateY(20px);
    }
    to { 
      opacity: 1;
      transform: translateY(0);
    }
  }

  .settings-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border);
  }

  .settings-header h2 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--fg);
  }

  .close-btn {
    width: 32px;
    height: 32px;
    border: none;
    background: transparent;
    color: var(--fg-muted);
    font-size: 20px;
    border-radius: var(--radius);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
  }

  .close-btn:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }

  .settings-tabs {
    display: flex;
    border-bottom: 1px solid var(--border);
    background: var(--bg);
  }

  .tab {
    flex: 1;
    padding: 12px 16px;
    border: none;
    background: transparent;
    color: var(--fg-muted);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    position: relative;
    transition: color 0.15s;
  }

  .tab:hover {
    color: var(--fg);
  }

  .tab.active {
    color: var(--accent);
  }

  .tab.active::after {
    content: '';
    position: absolute;
    bottom: -1px;
    left: 0;
    right: 0;
    height: 2px;
    background: var(--accent);
  }

  .settings-content {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
  }

  .tab-content {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .parameters-panel,
  .system-prompt-panel {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .param-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .param-group label {
    font-size: 13px;
    font-weight: 500;
    color: var(--fg);
  }

  .slider-container {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .slider-container input[type="range"] {
    flex: 1;
    height: 6px;
    -webkit-appearance: none;
    appearance: none;
    background: var(--bg);
    border-radius: 3px;
    outline: none;
  }

  .slider-container input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 18px;
    height: 18px;
    background: var(--accent);
    border-radius: 50%;
    cursor: pointer;
    transition: transform 0.15s, box-shadow 0.15s;
  }

  .slider-container input[type="range"]::-webkit-slider-thumb:hover {
    transform: scale(1.2);
    box-shadow: 0 0 0 3px var(--accent-bg, rgba(0, 122, 255, 0.2));
  }

  .slider-container input[type="range"]::-moz-range-thumb {
    width: 18px;
    height: 18px;
    background: var(--accent);
    border: none;
    border-radius: 50%;
    cursor: pointer;
  }

  .slider-value {
    min-width: 60px;
    text-align: right;
    font-size: 13px;
    font-weight: 500;
    color: var(--fg);
    font-variant-numeric: tabular-nums;
  }

  .param-description {
    margin: 0;
    font-size: 12px;
    color: var(--fg-muted);
    line-height: 1.4;
  }

  .param-note {
    padding: 12px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    font-size: 12px;
    color: var(--fg-muted);
  }

  .system-prompt-panel textarea {
    width: 100%;
    padding: 12px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--fg);
    font-size: 13px;
    font-family: inherit;
    line-height: 1.5;
    resize: vertical;
    min-height: 180px;
    outline: none;
    transition: border-color 0.15s;
  }

  .system-prompt-panel textarea:focus {
    border-color: var(--accent);
  }

  .system-prompt-panel textarea::placeholder {
    color: var(--fg-muted);
  }

  .system-prompt-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 4px;
  }

  .primary-btn,
  .secondary-btn {
    padding: 8px 16px;
    border-radius: var(--radius);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s;
    border: 1px solid transparent;
  }

  .primary-btn {
    background: var(--accent);
    color: white;
  }

  .primary-btn:hover {
    background: var(--accent-hover);
  }

  .secondary-btn {
    background: var(--bg-hover);
    color: var(--fg);
    border-color: var(--border);
  }

  .secondary-btn:hover {
    background: var(--bg);
  }

  /* Scrollbar styling */
  .settings-content::-webkit-scrollbar {
    width: 8px;
  }

  .settings-content::-webkit-scrollbar-track {
    background: transparent;
  }

  .settings-content::-webkit-scrollbar-thumb {
    background: var(--border);
    border-radius: 4px;
  }

  .settings-content::-webkit-scrollbar-thumb:hover {
    background: var(--fg-muted);
  }
</style>