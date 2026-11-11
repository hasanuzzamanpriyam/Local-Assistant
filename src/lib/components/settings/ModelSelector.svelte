<script lang="ts">
  import { allModels } from '../../stores/settings';
  import { modelScanLocal, modelSetDefault } from '../../api/llama';
  import { onMount } from 'svelte';

  let { onClose = () => {} } = $props();

  let isScanning = $state(false);
  let error: string | null = $state(null);

  async function scanModels() {
    isScanning = true;
    error = null;
    try {
      const result = await modelScanLocal();
      // The stores are updated by the scan command result - we just trigger a refresh
      // The derived store will pick up the changes
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to scan models';
      console.error('Model scan failed:', e);
    } finally {
      isScanning = false;
    }
  }

  async function selectModel(modelName: string) {
    try {
      await modelSetDefault(modelName);
      // The settings store will be updated by the command
      // Close the settings panel after selection
      onClose();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to set default model';
      console.error('Set default model failed:', e);
    }
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }

  // Initial scan on mount
  onMount(() => {
    scanModels();
  });
</script>

<div class="model-selector">
  <div class="selector-header">
    <h3>Select Model</h3>
    <button class="refresh-btn" onclick={scanModels} disabled={isScanning}>
      {isScanning ? '⟳ Scanning...' : '⟳ Refresh'}
    </button>
  </div>

  {#if error}
    <div class="error-message">{error}</div>
  {/if}

  <div class="model-list">
    {#each $allModels as model}
      <button
        class="model-item {model.selected ? 'selected' : ''} {model.source}"
        onclick={() => selectModel(model.name)}
      >
        <div class="model-info">
          <span class="model-name">{model.name}</span>
          <span class="model-source">{model.source}</span>
        </div>
        {#if model.selected}
          <span class="checkmark">✓</span>
        {/if}
      </button>
    {:else}
      <div class="no-models">
        {isScanning ? 'Scanning for models...' : 'No models found. Click Refresh to scan.'}
      </div>
    {/each}
  </div>

  <div class="model-hint">
    Local models: %APPDATA%/assistant/models/*.gguf<br>
    Remote models: from running llama.cpp server
  </div>
</div>

<style>
  .model-selector {
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-width: 320px;
    max-width: 480px;
  }

  .selector-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border);
  }

  .selector-header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--fg);
  }

  .refresh-btn {
    padding: 6px 12px;
    background: var(--bg-hover);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--fg);
    font-size: 12px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .refresh-btn:hover:not(:disabled) {
    background: var(--accent);
    border-color: var(--accent);
    color: white;
  }

  .refresh-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .error-message {
    padding: 8px 12px;
    background: var(--error-bg, #fee);
    border: 1px solid var(--error-border, #fcc);
    border-radius: var(--radius);
    color: var(--error-fg, #c00);
    font-size: 12px;
  }

  .model-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 300px;
    overflow-y: auto;
    padding: 4px 0;
  }

  .model-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 12px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--fg);
    font-size: 13px;
    cursor: pointer;
    transition: all 0.15s;
    text-align: left;
    width: 100%;
  }

  .model-item:hover {
    background: var(--bg-hover);
    border-color: var(--accent);
  }

  .model-item.selected {
    background: var(--accent-bg, rgba(0, 122, 255, 0.1));
    border-color: var(--accent);
  }

  .model-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
  }

  .model-name {
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .model-source {
    font-size: 11px;
    color: var(--fg-muted);
    text-transform: capitalize;
  }

  .model-item.local .model-source::before {
    content: '💾 ';
  }

  .model-item.remote .model-source::before {
    content: '☁️ ';
  }

  .checkmark {
    color: var(--accent);
    font-weight: bold;
    font-size: 16px;
  }

  .no-models {
    padding: 24px;
    text-align: center;
    color: var(--fg-muted);
    font-size: 13px;
  }

  .model-hint {
    margin-top: 8px;
    padding: 8px 12px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    font-size: 11px;
    color: var(--fg-muted);
    line-height: 1.5;
  }

  .model-list::-webkit-scrollbar {
    width: 8px;
  }

  .model-list::-webkit-scrollbar-track {
    background: transparent;
  }

  .model-list::-webkit-scrollbar-thumb {
    background: var(--border);
    border-radius: 4px;
  }

  .model-list::-webkit-scrollbar-thumb:hover {
    background: var(--fg-muted);
  }
</style>