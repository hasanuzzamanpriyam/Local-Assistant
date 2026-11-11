<script lang="ts">
  import { onMount } from 'svelte';
  import Message from './Message.svelte';
  import MessageInput from './MessageInput.svelte';
  import { chatStore, sendMessage } from '../../stores/chat';
  import { settingsStore } from '../../stores/settings';

  let messagesContainer: HTMLDivElement | null = null;
  let isAutoScroll = $state(true);
  let systemPrompt = $state('');

  // Load system prompt from settings store
  function loadSystemPrompt() {
    settingsStore.subscribe((s) => {
      systemPrompt = s.systemPrompt || '';
    })();
  }

  function scrollToBottom() {
    if (messagesContainer && isAutoScroll) {
      messagesContainer.scrollTop = messagesContainer.scrollHeight;
    }
  }

  function handleScroll() {
    if (!messagesContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = messagesContainer;
    // Consider at bottom if within 50px of bottom
    isAutoScroll = scrollHeight - scrollTop - clientHeight < 50;
  }

  function handleSend(content: string) {
    sendMessage(content, systemPrompt);
    // Scroll to bottom after a tick to allow message to render
    setTimeout(scrollToBottom, 0);
  }

  function clearChat() {
    chatStore.clear();
  }

  $effect(() => {
    // Auto-scroll when new messages arrive
    const messages = $chatStore;
    if (messages.length > 0) {
      setTimeout(scrollToBottom, 0);
    }
  });

  onMount(async () => {
    await loadSystemPrompt();
    // Initial scroll
    setTimeout(scrollToBottom, 100);
  });
</script>

<div class="chat-container">
  <div class="chat-header">
    <h2>Assistant</h2>
    <div class="chat-actions">
      <button class="icon-button" onclick={clearChat} title="Clear chat" aria-label="Clear chat">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="3 6 5 6 21 6"></polyline>
          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
        </svg>
      </button>
    </div>
  </div>

  <div 
    class="messages-container" 
    bind:this={messagesContainer}
    onscroll={handleScroll}
    role="log"
    aria-live="polite"
    aria-label="Chat messages"
  >
    {#each $chatStore as message (message.id)}
      <Message message={message} />
    {/each}
    
    {#if $chatStore.length === 0}
      <div class="empty-state">
        <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
        </svg>
        <p>Start a conversation</p>
        <span>Type a message below to begin</span>
      </div>
    {/if}
  </div>

  <MessageInput onSend={handleSend} />
</div>

<style>
  .chat-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg);
  }

  .chat-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 52px;
    padding: 0 16px;
    background: var(--bg-elevated);
    border-bottom: 1px solid var(--border);
  }

  .chat-header h2 {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    color: var(--fg);
  }

  .chat-actions {
    display: flex;
    gap: 4px;
  }

  .icon-button {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: var(--radius);
    color: var(--fg-muted);
    cursor: pointer;
    transition: background-color 0.2s, color 0.2s;
  }

  .icon-button:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }

  .messages-container {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--fg-muted);
    text-align: center;
    gap: 12px;
  }

  .empty-state p {
    margin: 0;
    font-size: 16px;
    font-weight: 500;
    color: var(--fg);
  }

  .empty-state span {
    font-size: 13px;
  }
</style>