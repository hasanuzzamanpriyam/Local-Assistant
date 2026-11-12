<script lang="ts">
  import ToolCallDisplay from './ToolCallDisplay.svelte';
  import type { ChatMessageExtended, ToolResult } from '../../../stores/chat';

  interface Props {
    message: ChatMessageExtended;
  }

  let { message }: Props = $props();

  // Simple markdown rendering
  function renderMarkdown(content: string): string {
    if (!content) return '';
    
    let html = content
      // Escape HTML
      .replace(/&/g, '&')
      .replace(/</g, '<')
      .replace(/>/g, '>')
      // Code blocks (before inline code)
      .replace(/```(\w+)?\n([\s\S]*?)```/g, (match, lang, code) => {
        const language = lang || '';
        return `<pre><code class="language-${language}">${code.trim()}</code></pre>`;
      })
      // Inline code
      .replace(/`([^`]+)`/g, '<code>$1</code>')
      // Bold
      .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
      // Italic
      .replace(/\*([^*]+)\*/g, '<em>$1</em>')
      // Headers
      .replace(/^### (.*$)/gm, '<h3>$1</h3>')
      .replace(/^## (.*$)/gm, '<h2>$1</h2>')
      .replace(/^# (.*$)/gm, '<h1>$1</h1>')
      // Links
      .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" rel="noopener">$1</a>')
      // Line breaks
      .replace(/\n/g, '<br>');
    
    return html;
  }

  function formatTime(timestamp: number): string {
    const date = new Date(timestamp);
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }

  function getRoleClass(role: string): string {
    switch (role) {
      case 'user': return 'message-user';
      case 'assistant': return 'message-assistant';
      case 'system': return 'message-system';
      case 'tool': return 'message-tool';
      default: return '';
    }
  }

  function getRoleLabel(role: string): string {
    switch (role) {
      case 'user': return 'You';
      case 'assistant': return 'Assistant';
      case 'system': return 'System';
      case 'tool': return 'Tool';
      default: return role;
    }
  }
</script>

<div class="message {getRoleClass(message.role)}" id={message.id}>
  <div class="message-header">
    <span class="message-role">{getRoleLabel(message.role)}</span>
    <span class="message-time">{formatTime(message.timestamp)}</span>
  </div>
  
  <div class="message-content" style="color: var(--fg); min-height: 1.5em;">
    {#if message.content}
      {@html renderMarkdown(message.content)}
    {:else}
      <span class="empty-content-placeholder">...</span>
    {/if}
  </div>
  
  {#if message.tool_calls && message.tool_calls.length > 0}
    <ToolCallDisplay 
      toolCalls={message.tool_calls} 
      toolResults={message.tool_results || []} 
    />
  {/if}
  
  {#if message.tool_results && message.tool_results.length > 0 && (!message.tool_calls || message.tool_calls.length === 0)}
    <ToolCallDisplay 
      toolCalls={[]} 
      toolResults={message.tool_results} 
    />
  {/if}
  
  {#if message.isStreaming}
    <span class="streaming-indicator">▊</span>
  {/if}
</div>

<style>
  .message {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 12px 16px;
    animation: fadeIn 0.2s ease-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .message-user {
    background: var(--bg-elevated);
    margin-left: 48px;
    border-radius: var(--radius) var(--radius) 4px var(--radius);
  }

  .message-assistant {
    margin-right: 48px;
    border-radius: var(--radius) var(--radius) var(--radius) 4px;
  }

  .message-system {
    opacity: 0.7;
    font-size: 12px;
    margin: 0 48px;
  }

  .message-tool {
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    margin: 0 48px;
    font-size: 12px;
  }

  .message-header {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 11px;
    color: var(--fg-muted);
  }

  .message-role {
    font-weight: 600;
    text-transform: capitalize;
    color: var(--accent);
  }

  .message-user .message-role {
    color: var(--fg);
  }

  .message-time {
    margin-left: auto;
    opacity: 0.6;
  }

  .message-content {
    font-size: 14px;
    line-height: 1.6;
    word-wrap: break-word;
  }
  
  /* Global markdown styles - these need :global() to apply to innerHTML content */
  :global(.message-content pre) {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 12px;
    margin: 8px 0;
    overflow-x: auto;
  }

  :global(.message-content code) {
    font-family: var(--font-mono);
    font-size: 13px;
  }

  :global(.message-content pre code) {
    background: transparent;
    padding: 0;
    font-size: 12px;
    line-height: 1.5;
  }

  :global(.message-content code:not(pre code)) {
    background: var(--bg);
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 12px;
  }

  :global(.message-content h1),
  :global(.message-content h2),
  :global(.message-content h3) {
    margin: 12px 0 8px;
    font-weight: 600;
    line-height: 1.3;
  }

  :global(.message-content h1) { font-size: 1.4em; }
  :global(.message-content h2) { font-size: 1.25em; }
  :global(.message-content h3) { font-size: 1.1em; }

  :global(.message-content p) {
    margin: 8px 0;
  }

  :global(.message-content ul),
  :global(.message-content ol) {
    margin: 8px 0;
    padding-left: 24px;
  }

  :global(.message-content li) {
    margin: 4px 0;
  }

  :global(.message-content blockquote) {
    margin: 8px 0;
    padding-left: 12px;
    border-left: 3px solid var(--accent);
    color: var(--fg-muted);
    font-style: italic;
  }

  :global(.message-content a) {
    color: var(--accent);
    text-decoration: underline;
  }

  :global(.message-content hr) {
    border: none;
    border-top: 1px solid var(--border);
    margin: 16px 0;
  }

  .streaming-indicator {
    display: inline-block;
    width: 8px;
    height: 16px;
    background: var(--accent);
    animation: blink 1s infinite;
    margin-top: 4px;
    align-self: flex-start;
  }

  @keyframes blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0; }
  }
</style>