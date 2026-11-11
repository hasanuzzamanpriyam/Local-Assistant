<script lang="ts">
  import { sidecarStore } from '../../stores/sidecar';

  let status: any = {};
  sidecarStore.subscribe((s) => { status = s; })();

  async function start() {
    await sidecarStore.start();
  }

  async function stop() {
    await sidecarStore.stop();
  }
</script>

<div class="sidecar-status">
  <span class="indicator" class:ready={status.ready} class:running={status.running}></span>
  <span class="label">{status.ready ? 'Ready' : status.running ? 'Starting...' : 'Stopped'}</span>
  {#if !status.ready}
    <button onclick={start}>Start Sidecar</button>
  {:else}
    <button onclick={stop}>Stop Sidecar</button>
  {/if}
</div>

<style>
  .sidecar-status {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
  }
  .indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--fg-muted);
  }
  .indicator.ready { background: #00c851; box-shadow: 0 0 8px #00c851; }
  .indicator.running { background: var(--accent); box-shadow: 0 0 8px var(--accent); }
  .label { color: var(--fg-muted); }
  button {
    padding: 4px 10px;
    font-size: 12px;
    border: 1px solid var(--border);
    background: var(--bg-elevated);
    color: var(--fg);
    border-radius: var(--radius);
    cursor: pointer;
  }
  button:hover { background: var(--bg-hover); }
</style>
