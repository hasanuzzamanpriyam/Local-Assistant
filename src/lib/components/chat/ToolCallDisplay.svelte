<script lang="ts">
  import type { ToolCall, ToolResult } from '../../../stores/chat';

  interface Props {
    toolCalls?: ToolCall[];
    toolResults?: ToolResult[];
  }

  let { toolCalls = [], toolResults = [] }: Props = $props();

  function formatArguments(args: string): string {
    try {
      const parsed = JSON.parse(args);
      return JSON.stringify(parsed, null, 2);
    } catch {
      return args;
    }
  }

  function getResultForToolCall(toolCallId: string): ToolResult | undefined {
    return toolResults.find((r) => r.tool_call_id === toolCallId);
  }
</script>

{#if toolCalls.length > 0 || toolResults.length > 0}
  <div class="tool-calls-container">
    {#each toolCalls as toolCall (toolCall.id)}
      <div class="tool-call">
        <div class="tool-call-header">
          <span class="tool-call-icon">⚙️</span>
          <span class="tool-call-name">{toolCall.function.name}</span>
          <span class="tool-call-id">{toolCall.id.slice(0, 8)}...</span>
        </div>
        <div class="tool-call-arguments">
          <pre>{formatArguments(toolCall.function.arguments)}</pre>
        </div>
        {#if getResultForToolCall(toolCall.id) as result}
          <div class="tool-call-result" class:error={!!result.error}>
            <div class="tool-result-header">
              <span class="tool-result-label">{result.error ? 'Error' : 'Result'}</span>
            </div>
            <div class="tool-result-content">
              <pre>{result.error ? result.error : result.result}</pre>
            </div>
          </div>
        {/if}
      </div>
    {/each}
    
    {#each toolResults as result (result.tool_call_id)}
      {#if !toolCalls.find(tc => tc.id === result.tool_call_id)}
        <div class="tool-call tool-result-only">
          <div class="tool-call-header">
            <span class="tool-call-icon">📤</span>
            <span class="tool-call-name">{result.name}</span>
            <span class="tool-call-id">{result.tool_call_id.slice(0, 8)}...</span>
          </div>
          <div class="tool-call-arguments">
            <pre>{formatArguments(result.arguments)}</pre>
          </div>
          <div class="tool-call-result" class:error={!!result.error}>
            <div class="tool-result-header">
              <span class="tool-result-label">{result.error ? 'Error' : 'Result'}</span>
            </div>
            <div class="tool-result-content">
              <pre>{result.error ? result.error : result.result}</pre>
            </div>
          </div>
        </div>
      {/if}
    {/each}
  </div>
{/if}

<style>
  .tool-calls-container {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 8px;
    padding: 12px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius);
  }

  .tool-call {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .tool-call:not(:last-child) {
    border-bottom: 1px solid var(--border);
    padding-bottom: 8px;
  }

  .tool-call-header {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--fg-muted);
  }

  .tool-call-name {
    font-weight: 600;
    color: var(--fg);
    font-family: var(--font-mono);
  }

  .tool-call-id {
    margin-left: auto;
    opacity: 0.6;
    font-family: var(--font-mono);
  }

  .tool-call-arguments,
  .tool-call-result {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    overflow: hidden;
  }

  .tool-call-arguments pre,
  .tool-call-result pre {
    margin: 0;
    padding: 10px;
    font-size: 11px;
    line-height: 1.5;
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-word;
    color: var(--fg-muted);
  }

  .tool-call-result {
    border-left: 3px solid var(--accent);
  }

  .tool-call-result.error {
    border-left-color: #ff4444;
  }

  .tool-result-header {
    padding: 8px 10px;
    background: var(--bg-elevated);
    border-bottom: 1px solid var(--border);
    font-size: 11px;
    font-weight: 600;
    color: var(--fg-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .tool-result-content pre {
    color: var(--fg);
  }

  .tool-call-result.error .tool-result-content pre {
    color: #ff6b6b;
  }
</style>