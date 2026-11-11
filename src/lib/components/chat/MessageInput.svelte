<script lang="ts">
  import { onMount } from 'svelte';

  interface Props {
    disabled?: boolean;
    onSend: (content: string) => void;
  }

  let { disabled = false, onSend }: Props = $props();

  let textarea: HTMLTextareaElement | null = null;
  let content = $state('');
  let isComposing = $state(false);

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey && !isComposing) {
      event.preventDefault();
      sendMessage();
    }
  }

  function handleCompositionStart() {
    isComposing = true;
  }

  function handleCompositionEnd(event: CompositionEvent) {
    isComposing = false;
    // Handle the case where Enter was pressed during composition
    const target = event.target as HTMLTextareaElement;
    if (target.value.endsWith('\n')) {
      target.value = target.value.slice(0, -1);
      sendMessage();
    }
  }

  function sendMessage() {
    const trimmed = content.trim();
    if (trimmed && !disabled) {
      onSend(trimmed);
      content = '';
      // Reset height after sending
      if (textarea) {
        textarea.style.height = 'auto';
        textarea.style.height = Math.min(textarea.scrollHeight, 200) + 'px';
      }
    }
  }

  function autoResize() {
    if (textarea) {
      textarea.style.height = 'auto';
      const newHeight = Math.min(textarea.scrollHeight, 200);
      textarea.style.height = newHeight + 'px';
    }
  }

  // Auto-resize on content change
  $effect(() => {
    if (content !== undefined) {
      autoResize();
    }
  });
</script>

<div class="message-input-container">
  <div class="input-wrapper">
    <textarea
      bind:this={textarea}
      bind:value={content}
      placeholder="Message Assistant... (Shift+Enter for new line)"
      class="message-input"
      disabled={disabled}
      onkeydown={handleKeydown}
      oncompositionstart={handleCompositionStart}
      oncompositionend={handleCompositionEnd}
      oninput={autoResize}
      rows={1}
      spellcheck="false"
      aria-label="Message input"
    ></textarea>
  </div>
  <div class="input-actions">
    <button 
      class="send-button" 
      onclick={sendMessage}
      disabled={!content.trim() || disabled}
      aria-label="Send message"
    >
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="22" y1="2" x2="11" y2="13"></line>
        <polygon points="22 2 15 22 11 13 2 9 22 2"></polygon>
      </svg>
    </button>
  </div>
</div>

<style>
  .message-input-container {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 16px;
    background: var(--bg-elevated);
    border-top: 1px solid var(--border);
  }

  .input-wrapper {
    position: relative;
  }

  .message-input {
    width: 100%;
    min-height: 44px;
    max-height: 200px;
    padding: 10px 14px;
    font-size: 14px;
    line-height: 1.5;
    resize: none;
    overflow-y: auto;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--fg);
    transition: border-color 0.2s, box-shadow 0.2s;
  }

  .message-input:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 2px rgba(0, 122, 255, 0.2);
  }

  .message-input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .message-input::placeholder {
    color: var(--fg-muted);
  }

  .input-actions {
    display: flex;
    justify-content: flex-end;
  }

  .send-button {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--accent);
    color: white;
    border: none;
    border-radius: var(--radius);
    cursor: pointer;
    transition: background-color 0.2s, transform 0.1s;
  }

  .send-button:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .send-button:active:not(:disabled) {
    transform: scale(0.95);
  }

  .send-button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .send-button svg {
    display: block;
  }
</style>