<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
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
  <main class="content">App loaded</main>
</div>

<style>
  .app { display: flex; flex-direction: column; height: 100vh; background: var(--bg); color: var(--fg); }
  .titlebar { display: flex; justify-content: space-between; align-items: center; height: 36px; padding: 0 12px; background: var(--bg-elevated); border-bottom: 1px solid var(--border); user-select: none; }
  .titlebar-drag { -webkit-app-region: drag; font-weight: 600; font-size: 13px; }
  .titlebar-buttons { display: flex; gap: 4px; }
  .titlebar-buttons button { width: 28px; height: 28px; border: none; background: transparent; color: var(--fg); border-radius: 4px; cursor: pointer; }
  .titlebar-buttons button:hover { background: var(--bg-hover); }
  .content { flex: 1; overflow: auto; }
</style>