<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import Chat from './lib/components/chat/Chat.svelte';
  import { modelHealth, modelStart } from './lib/api/llama';

  let isModelReady = $state(false);
  let isLoading = $state(true);

  // Check model health on mount
  onMount(async () => {
    await checkModel();
  });

  async function checkModel() {
    isLoading = true;
    isModelReady = false;
    
    try {
      const health = await modelHealth();
      if (health.healthy) {
        isModelReady = true;
      } else {
        // Try to start the model
        await modelStart();
        // Wait a bit for startup
        await new Promise(r => setTimeout(r, 2000));
        const health2 = await modelHealth();
        isModelReady = health2.healthy;
      }
    } catch (error) {
      console.error('Model check failed:', error);
      isModelReady = false;
    } finally {
      isLoading = false;
    }
  }
</script>

<div class="app">
  <header class="titlebar">
    <div class="titlebar-drag" data-tauri-drag-region>Assistant</div>
    <div class="titlebar-buttons">
      <button data-tauri-minimize>−</button>
      <button data-tauri-maximize>□</button>
      <button data-tauri-close>×</button>
    </div>
  </header>
  
  <main class="content">
    {#if isLoading}
      <div class="loading-overlay">
        <div class="loading-spinner"></div>
        <p>Starting model...</p>
      </div>
    {:else if !isModelReady}
      <div class="error-state">
        <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <circle cx="12" cy="12" r="10"></circle>
          <line x1="12" y1="8" x2="12" y2="12"></line>
          <line x1="12" y1="16" x2="12.01" y2="16"></line>
        </svg>
        <h3>Model not ready</h3>
        <p>Please ensure the model is downloaded and try again</p>
        <button class="retry-button" onclick={checkModel}>Retry</button>
      </div>
    {:else}
      <Chat />
    {/if}
  </main>
</div>

<style>
  .app { display: flex; flex-direction: column; height: 100vh; background: var(--bg); color: var(--fg); }
  .titlebar { display: flex; justify-content: space-between; align-items: center; height: 36px; padding: 0 12px; background: var(--bg-elevated); border-bottom: 1px solid var(--border); user-select: none; }
  .titlebar-drag { -webkit-app-region: drag; font-weight: 600; font-size: 13px; }
  .titlebar-buttons { display: flex; gap: 4px; }
  .titlebar-buttons button { width: 28px; height: 28px; border: none; background: transparent; color: var(--fg); border-radius: 4px; cursor: pointer; }
  .titlebar-buttons button:hover { background: var(--bg-hover); }
  .content { flex: 1; overflow: hidden; position: relative; }

  .loading-overlay,
  .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 16px;
    color: var(--fg-muted);
    text-align: center;
    padding: 24px;
  }

  .loading-overlay p {
    margin: 0;
    color: var(--fg);
  }

  .loading-spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .error-state h3 {
    margin: 0;
    font-size: 18px;
    color: var(--fg);
  }

  .error-state p {
    margin: 0;
    font-size: 14px;
  }

  .retry-button {
    margin-top: 8px;
    padding: 10px 24px;
    background: var(--accent);
    color: white;
    border: none;
    border-radius: var(--radius);
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .retry-button:hover {
    background: var(--accent-hover);
  }
</style>