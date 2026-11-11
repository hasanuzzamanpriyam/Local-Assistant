import { writable, derived, get } from 'svelte/store';
import type { ChatMessage, ToolCall, ChatCompletionRequest, ChatCompletionChunk } from '../api/llama';
import { modelChatStream } from '../api/llama';
import { fsRead, fsWrite, fsEdit, fsGlob, fsGrep } from '../api/fs';
import { shellExec } from '../api/shell';

const STORAGE_KEY = 'assistant_chat_history';
const MAX_MESSAGES = 100;

export interface ToolResult {
  tool_call_id: string;
  name: string;
  arguments: string;
  result: string;
  error?: string;
}

export interface ChatMessageExtended extends ChatMessage {
  id: string;
  timestamp: number;
  isStreaming?: boolean;
  tool_results?: ToolResult[];
}

// Create the chat store
function createChatStore() {
  // Load from localStorage
  let initialMessages: ChatMessageExtended[] = [];
  if (typeof localStorage !== 'undefined') {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        initialMessages = JSON.parse(stored);
      }
    } catch (e) {
      console.error('Failed to load chat history:', e);
    }
  }

  const { subscribe, set, update } = writable<ChatMessageExtended[]>(initialMessages);

  // Persist to localStorage whenever messages change
  subscribe((messages) => {
    if (typeof localStorage !== 'undefined') {
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(messages.slice(-MAX_MESSAGES)));
      } catch (e) {
        console.error('Failed to save chat history:', e);
      }
    }
  });

  return {
    subscribe,
    set,
    update,

    // Add a new message
    addMessage(message: Omit<ChatMessageExtended, 'id' | 'timestamp'>) {
      const newMessage: ChatMessageExtended = {
        ...message,
        id: crypto.randomUUID(),
        timestamp: Date.now(),
      };
      update((messages) => [...messages, newMessage]);
      return newMessage;
    },

    // Update a message by ID
    updateMessage(id: string, updates: Partial<ChatMessageExtended>) {
      update((messages) =>
        messages.map((msg) => (msg.id === id ? { ...msg, ...updates } : msg))
      );
    },

    // Append content to a streaming message
    appendToMessage(id: string, content: string) {
      update((messages) =>
        messages.map((msg) =>
          msg.id === id
            ? { ...msg, content: msg.content + content, isStreaming: true }
            : msg
        )
      );
    },

    // Mark message as done streaming
    finishStreaming(id: string) {
      update((messages) =>
        messages.map((msg) =>
          msg.id === id ? { ...msg, isStreaming: false } : msg
        )
      );
    },

    // Add tool results to a message
    addToolResults(id: string, toolResults: ToolResult[]) {
      update((messages) =>
        messages.map((msg) =>
          msg.id === id
            ? { ...msg, tool_results: [...(msg.tool_results || []), ...toolResults] }
            : msg
        )
      );
    },

    // Clear all messages
    clear() {
      set([]);
    },

    // Get messages for LLM (without internal fields)
    getLLMMessages(): ChatMessage[] {
      let messages: ChatMessageExtended[] = [];
      // Use subscribe once to get current value
      const unsubscribe = subscribe((msgs) => { messages = msgs; });
      unsubscribe();
      return messages.map(({ id, timestamp, isStreaming, tool_results, ...msg }) => msg);
    },
  };
}

export const chatStore = createChatStore();

// Tool definitions for the LLM
export const availableTools = [
  {
    type: 'function',
    function: {
      name: 'fs_read',
      description: 'Read the contents of a file',
      parameters: {
        type: 'object',
        properties: {
          path: { type: 'string', description: 'Path to the file to read' },
        },
        required: ['path'],
      },
    },
  },
  {
    type: 'function',
    function: {
      name: 'fs_write',
      description: 'Write content to a file (creates or overwrites)',
      parameters: {
        type: 'object',
        properties: {
          path: { type: 'string', description: 'Path to the file to write' },
          content: { type: 'string', description: 'Content to write to the file' },
        },
        required: ['path', 'content'],
      },
    },
  },
  {
    type: 'function',
    function: {
      name: 'fs_edit',
      description: 'Edit a file by replacing old text with new text',
      parameters: {
        type: 'object',
        properties: {
          path: { type: 'string', description: 'Path to the file to edit' },
          old_text: { type: 'string', description: 'Text to replace' },
          new_text: { type: 'string', description: 'New text to insert' },
        },
        required: ['path', 'old_text', 'new_text'],
      },
    },
  },
  {
    type: 'function',
    function: {
      name: 'fs_glob',
      description: 'Find files matching a glob pattern',
      parameters: {
        type: 'object',
        properties: {
          pattern: { type: 'string', description: 'Glob pattern to match files' },
        },
        required: ['pattern'],
      },
    },
  },
  {
    type: 'function',
    function: {
      name: 'fs_grep',
      description: 'Search for a pattern in files',
      parameters: {
        type: 'object',
        properties: {
          pattern: { type: 'string', description: 'Regex pattern to search for' },
          path: { type: 'string', description: 'Optional path to search in' },
        },
        required: ['pattern'],
      },
    },
  },
  {
    type: 'function',
    function: {
      name: 'shell_exec',
      description: 'Execute a shell command',
      parameters: {
        type: 'object',
        properties: {
          command: { type: 'string', description: 'Command to execute (e.g., cmd, powershell, bash)' },
          args: { type: 'array', items: { type: 'string' }, description: 'Arguments to pass to the command' },
          cwd: { type: 'string', description: 'Optional working directory (relative to workspace root)' },
        },
        required: ['command', 'args'],
      },
    },
  },
];

// Execute a tool call and return the result
export async function executeToolCall(toolCall: ToolCall): Promise<ToolResult> {
  const { id, function: fn } = toolCall;
  const name = fn.name;
  let args: any = {};
  
  try {
    args = JSON.parse(fn.arguments);
  } catch (e) {
    return {
      tool_call_id: id,
      name,
      arguments: fn.arguments,
      result: '',
      error: `Failed to parse arguments: ${e}`,
    };
  }

  try {
    let result: string;
    
    switch (name) {
      case 'fs_read':
        result = await fsRead(args.path);
        break;
      case 'fs_write':
        await fsWrite(args.path, args.content);
        result = `File written: ${args.path}`;
        break;
      case 'fs_edit':
        await fsEdit(args.path, args.old_text, args.new_text);
        result = `File edited: ${args.path}`;
        break;
      case 'fs_glob':
        const files = await fsGlob(args.pattern);
        result = files.length > 0 ? files.join('\n') : 'No files found';
        break;
      case 'fs_grep':
        const matches = await fsGrep(args.pattern, args.path);
        result = matches.length > 0 
          ? matches.map(m => `${m.path}:${m.line_number}: ${m.line}`).join('\n')
          : 'No matches found';
        break;
      case 'shell_exec':
        const execResult = await shellExec(args.command, args.args, args.cwd);
        result = execResult.stdout || execResult.stderr || `Exit code: ${execResult.code}`;
        break;
      default:
        throw new Error(`Unknown tool: ${name}`);
    }

    return {
      tool_call_id: id,
      name,
      arguments: fn.arguments,
      result,
    };
  } catch (error) {
    return {
      tool_call_id: id,
      name,
      arguments: fn.arguments,
      result: '',
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

// Send a message and handle streaming response with tool execution loop
export async function sendMessage(content: string, systemPrompt: string = '') {
  // Add user message
  const userMessage = chatStore.addMessage({
    role: 'user',
    content,
  });

  // Create assistant message placeholder
  const assistantMessage = chatStore.addMessage({
    role: 'assistant',
    content: '',
    isStreaming: true,
  });

  // Prepare messages for LLM
  const messages = chatStore.getLLMMessages();
  
  // Add system prompt if provided
  const requestMessages: ChatMessage[] = [];
  if (systemPrompt) {
    requestMessages.push({ role: 'system', content: systemPrompt });
  }
  requestMessages.push(...messages);

  const request: ChatCompletionRequest = {
    model: 'default',
    messages: requestMessages,
    temperature: 0.3,
    top_p: 0.9,
    max_tokens: 4096,
    stream: true,
    tools: availableTools,
    tool_choice: 'auto',
  };

  let accumulatedContent = '';
  let accumulatedToolCalls: ToolCall[] = [];
  let currentToolCall: Partial<ToolCall> | null = null;
  let currentToolCallIndex = -1;

  // Cleanup function for streaming
  let cleanup: (() => void) | null = null;

  try {
    cleanup = await modelChatStream(
      request,
      (chunk: ChatCompletionChunk) => {
        const choice = chunk.choices[0];
        if (!choice) return;

        const delta = choice.delta;

        // Handle content streaming
        if (delta.content) {
          accumulatedContent += delta.content;
          chatStore.appendToMessage(assistantMessage.id, delta.content);
        }

        // Handle tool calls streaming
        if (delta.tool_calls && delta.tool_calls.length > 0) {
          for (const tcDelta of delta.tool_calls) {
            if (tcDelta.index !== currentToolCallIndex) {
              // New tool call started
              if (currentToolCall) {
                accumulatedToolCalls.push(currentToolCall as ToolCall);
              }
              currentToolCallIndex = tcDelta.index;
              currentToolCall = {
                id: tcDelta.id || '',
                type: tcDelta.type || 'function',
                function: { name: '', arguments: '' },
              };
            }

            if (tcDelta.function?.name) {
              currentToolCall!.function!.name += tcDelta.function.name;
            }
            if (tcDelta.function?.arguments) {
              currentToolCall!.function!.arguments += tcDelta.function.arguments;
            }
          }
        }

        // Check if done
        if (choice.finish_reason) {
          // Add any remaining tool call
          if (currentToolCall) {
            accumulatedToolCalls.push(currentToolCall as ToolCall);
          }

          // Update the message with tool calls
          if (accumulatedToolCalls.length > 0) {
            chatStore.updateMessage(assistantMessage.id, {
              content: accumulatedContent,
              tool_calls: accumulatedToolCalls,
              isStreaming: false,
            });
          } else {
            chatStore.finishStreaming(assistantMessage.id);
          }
        }
      },
      async () => {
        // Stream done - check for tool calls
        if (accumulatedToolCalls.length > 0) {
          // Execute tool calls
          const toolResults: ToolResult[] = [];
          
          for (const toolCall of accumulatedToolCalls) {
            const result = await executeToolCall(toolCall);
            toolResults.push(result);
          }

          // Add tool results to the message
          chatStore.addToolResults(assistantMessage.id, toolResults);

          // Now send tool results back to LLM for final response
          await continueConversationAfterTools(assistantMessage.id, toolResults, systemPrompt);
        } else {
          // No tool calls, just finish
          chatStore.finishStreaming(assistantMessage.id);
        }
      },
      (error: Error) => {
        console.error('Streaming error:', error);
        chatStore.updateMessage(assistantMessage.id, {
          content: accumulatedContent + `\n\n[Error: ${error.message}]`,
          isStreaming: false,
        });
      }
    );
  } catch (error) {
    console.error('Failed to start stream:', error);
    chatStore.updateMessage(assistantMessage.id, {
      content: `[Error: ${error instanceof Error ? error.message : String(error)}]`,
      isStreaming: false,
    });
  }
}

// Continue conversation after tool execution
async function continueConversationAfterTools(
  assistantMessageId: string,
  toolResults: ToolResult[],
  systemPrompt: string
) {
  // Create a new assistant message for the final response
  const finalMessage = chatStore.addMessage({
    role: 'assistant',
    content: '',
    isStreaming: true,
  });

  // Build messages including tool results
  const messages = chatStore.getLLMMessages();
  
  // Find the assistant message with tool calls and add tool results
  const messagesWithTools: ChatMessage[] = [];
  if (systemPrompt) {
    messagesWithTools.push({ role: 'system', content: systemPrompt });
  }

  // Get the full messages with extended properties
  let fullMessages: ChatMessageExtended[] = [];
  const unsubscribe = chatStore.subscribe((msgs) => { fullMessages = msgs; });
  unsubscribe();

  for (const msg of fullMessages) {
    if (msg.id === assistantMessageId && msg.tool_calls) {
      // Add the assistant message with tool calls
      messagesWithTools.push({
        role: 'assistant',
        content: msg.content,
        tool_calls: msg.tool_calls,
      });
      // Add tool results as tool messages
      for (const result of toolResults) {
        messagesWithTools.push({
          role: 'tool',
          content: result.error ? `Error: ${result.error}` : result.result,
          tool_call_id: result.tool_call_id,
          name: result.name,
        });
      }
    } else if (msg.role !== 'tool') {
      // Skip tool messages from history (they're added above)
      const { id, timestamp, isStreaming, tool_results, ...cleanMsg } = msg;
      messagesWithTools.push(cleanMsg);
    }
  }

  const request: ChatCompletionRequest = {
    model: 'default',
    messages: messagesWithTools,
    temperature: 0.3,
    top_p: 0.9,
    max_tokens: 4096,
    stream: true,
    tools: availableTools,
    tool_choice: 'auto',
  };

  let accumulatedContent = '';
  let accumulatedToolCalls: ToolCall[] = [];
  let currentToolCall: Partial<ToolCall> | null = null;
  let currentToolCallIndex = -1;
  let hasMoreToolCalls = false;

  try {
    await modelChatStream(
      request,
      (chunk: ChatCompletionChunk) => {
        const choice = chunk.choices[0];
        if (!choice) return;

        const delta = choice.delta;

        if (delta.content) {
          accumulatedContent += delta.content;
          chatStore.appendToMessage(finalMessage.id, delta.content);
        }

        if (delta.tool_calls && delta.tool_calls.length > 0) {
          for (const tcDelta of delta.tool_calls) {
            if (tcDelta.index !== currentToolCallIndex) {
              if (currentToolCall) {
                accumulatedToolCalls.push(currentToolCall as ToolCall);
              }
              currentToolCallIndex = tcDelta.index;
              currentToolCall = {
                id: tcDelta.id || '',
                type: tcDelta.type || 'function',
                function: { name: '', arguments: '' },
              };
            }

            if (tcDelta.function?.name) {
              currentToolCall!.function!.name += tcDelta.function.name;
            }
            if (tcDelta.function?.arguments) {
              currentToolCall!.function!.arguments += tcDelta.function.arguments;
            }
          }
        }

        if (choice.finish_reason) {
          if (currentToolCall) {
            accumulatedToolCalls.push(currentToolCall as ToolCall);
          }

          if (accumulatedToolCalls.length > 0) {
            // More tool calls needed - mark for execution in onDone
            hasMoreToolCalls = true;
            chatStore.updateMessage(finalMessage.id, {
              content: accumulatedContent,
              tool_calls: accumulatedToolCalls,
              isStreaming: false,
            });
          } else {
            // Done
            chatStore.finishStreaming(finalMessage.id);
          }
        }
      },
      async () => {
        // Stream done - check for tool calls
        if (hasMoreToolCalls && accumulatedToolCalls.length > 0) {
          // Execute new tool calls
          const newToolResults: ToolResult[] = [];
          for (const toolCall of accumulatedToolCalls) {
            const result = await executeToolCall(toolCall);
            newToolResults.push(result);
          }
          
          chatStore.addToolResults(finalMessage.id, newToolResults);
          
          // Continue the conversation loop
          await continueConversationAfterTools(finalMessage.id, newToolResults, systemPrompt);
        } else {
          chatStore.finishStreaming(finalMessage.id);
        }
      },
      (error: Error) => {
        console.error('Streaming error:', error);
        chatStore.updateMessage(finalMessage.id, {
          content: accumulatedContent + `\n\n[Error: ${error.message}]`,
          isStreaming: false,
        });
      }
    );
  } catch (error) {
    console.error('Failed to continue conversation:', error);
    chatStore.updateMessage(finalMessage.id, {
      content: accumulatedContent + `\n\n[Error: ${error instanceof Error ? error.message : String(error)}]`,
      isStreaming: false,
    });
  }
}