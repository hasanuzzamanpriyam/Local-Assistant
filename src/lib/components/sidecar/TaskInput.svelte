<script lang="ts">
  import { sidecarStore } from '../../stores/sidecar';

  let taskInput = '';
  let maxSteps = 10;
  let sending = false;

  async function handleSend() {
    const task = taskInput.trim();
    if (!task || sending) return;
    sending = true;
    try {
      await sidecarStore.runTask(task, maxSteps);
      taskInput = '';
    } catch (e) {
      console.error('Failed to run task:', e);
    } finally {
      sending = false;
    }
  }
</script>

<div class="task-input">
  <input
    bind:value={taskInput}
    placeholder="What should I do on your computer? (e.g., Open Notepad and type hello)"
    disabled={sending}
  />
  <input
    type="number"
    bind:value={maxSteps}
    min="1"
    max="50"
    placeholder="Max steps"
    disabled={sending}
  />
  <button onclick={handleSend} disabled={sending || !taskInput.trim()}>
    {sending ? 'Running...' : 'Run Task'}
  </button>
</div>

<style>
  .task-input {
    display: flex;
    gap: 8px;
    padding: 16px;
    background: var(--bg-elevated);
    border-top: 1px solid var(--border);
  }
  .task-input input[type="text"] {
    flex: 1;
    padding: 8px 12px;
    background: var(--bg);
    border: 1px solid var(--border);
    color: var(--fg);
    border-radius: var(--radius);
  }
  .task-input input[type="number"] {
    width: 80px;
    padding: 8px 12px;
    background: var(--bg);
    border: 1px solid var(--border);
    color: var(--fg);
    border-radius: var(--radius);
  }
  button {
    padding: 8px 16px;
    background: var(--accent);
    color: white;
    border: none;
    border-radius: var(--radius);
    cursor: pointer;
    font-weight: 500;
  }
  button:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
