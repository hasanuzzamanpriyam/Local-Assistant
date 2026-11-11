<script lang="ts">
  import { sidecarStore } from '../../stores/sidecar';
  import ScreenshotViewer from './ScreenshotViewer.svelte';

  let events = $state<any[]>([]);
  sidecarStore.subscribeEvents((evts) => { events = evts; })();

  let { onTaskStarted, onTaskStopped }: { onTaskStarted?: (id: string) => void; onTaskStopped?: () => void } = $props();

  let unlisten: (() => void) | null = null;

  async function startListening() {
    unlisten = await sidecarStore.listen((event) => {
      events = [...events, event];

      if (event.event === 'task_started' && event.id) {
        onTaskStarted?.(event.id);
      }
      if ((event.event === 'task_complete' || event.event === 'task_stopped' || event.event === 'error') && event.id) {
        onTaskStopped?.();
      }
    });
  }

  function clearEvents() {
    events = [];
  }
</script>

<div class="task-feed" onmount={startListening}>
  <div class="feed-header">
    <h3>Task Activity</h3>
    <button onclick={clearEvents} class="clear-btn">Clear</button>
  </div>
  <div class="feed-list">
    {#each events as event (event.id + '-' + event.event)}
      <div class="feed-item {event.event}">
        <div class="feed-time">{new Date().toLocaleTimeString()}</div>
        <div class="feed-event">{event.event}</div>
        {#if event.thought}
          <div class="feed-thought">💭 {event.thought}</div>
        {/if}
        {#if event.action}
          <div class="feed-action">⚡ {event.action}</div>
        {/if}
        {#if event.screenshot_base64}
          <ScreenshotViewer src={event.screenshot_base64} />
        {/if}
        {#if event.result}
          <div class="feed-result">{event.success ? '✅' : '❌'} {event.result}</div>
        {/if}
        {#if event.error}
          <div class="feed-error">❌ {event.error}</div>
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .task-feed {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .feed-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-elevated);
  }
  .feed-header h3 {
    margin: 0;
    font-size: 14px;
    color: var(--fg);
  }
  .clear-btn {
    padding: 4px 10px;
    font-size: 11px;
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--fg-muted);
    border-radius: var(--radius);
    cursor: pointer;
  }
  .feed-list {
    flex: 1;
    overflow-y: auto;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .feed-item {
    padding: 12px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .feed-time {
    font-size: 11px;
    color: var(--fg-muted);
  }
  .feed-event {
    font-size: 12px;
    font-weight: 600;
    color: var(--accent);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .feed-thought {
    font-size: 13px;
    color: var(--fg);
    font-style: italic;
  }
  .feed-action {
    font-size: 13px;
    color: var(--fg-muted);
    font-family: var(--font-mono);
  }
  .feed-result {
    font-size: 13px;
    color: #00c851;
    margin-top: 4px;
  }
  .feed-error {
    font-size: 13px;
    color: #ff4444;
    margin-top: 4px;
  }
</style>
