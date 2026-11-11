import { invoke } from '@tauri-apps/api/core';

// Extend Window interface for Tauri event system
declare global {
  interface Window {
    __TAURI__: {
      event: {
        listen: (event: string, handler: (event: any) => void) => Promise<() => void>;
      };
    };
  }
}

export interface ChatMessage {
  role: string;
  content: string;
  name?: string;
  tool_calls?: ToolCall[];
  tool_call_id?: string;
}

export interface ToolCall {
  id: string;
  type: string;
  function: FunctionCall;
}

export interface FunctionCall {
  name: string;
  arguments: string;
}

export interface ChatCompletionRequest {
  model: string;
  messages: ChatMessage[];
  temperature?: number;
  top_p?: number;
  max_tokens?: number;
  stream?: boolean;
  tools?: Tool[];
  tool_choice?: any;
}

export interface Tool {
  type: string;
  function: ToolFunction;
}

export interface ToolFunction {
  name: string;
  description: string;
  parameters: any;
}

export interface ChatCompletionResponse {
  id: string;
  object: string;
  created: number;
  model: string;
  choices: ChatChoice[];
  usage?: Usage;
}

export interface ChatChoice {
  index: number;
  message: ChatMessage;
  finish_reason?: string;
}

export interface Usage {
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
}

export interface ChatCompletionChunk {
  id: string;
  object: string;
  created: number;
  model: string;
  choices: ChatChunkChoice[];
}

export interface ChatChunkChoice {
  index: number;
  delta: ChatMessageDelta;
  finish_reason?: string;
}

export interface ChatMessageDelta {
  role?: string;
  content?: string;
  tool_calls?: ToolCallDelta[];
}

export interface ToolCallDelta {
  index: number;
  id?: string;
  type?: string;
  function?: FunctionCallDelta;
}

export interface FunctionCallDelta {
  name?: string;
  arguments?: string;
}

export interface ModelInfo {
  name: string;
  path: string;
  size: number;
  downloaded: boolean;
}

export interface HealthResponse {
  healthy: boolean;
}

export async function modelHealth(): Promise<HealthResponse> {
  return invoke('model_health');
}

export async function modelList(): Promise<ModelInfo[]> {
  return invoke('model_list');
}

export async function modelChat(request: ChatCompletionRequest): Promise<ChatCompletionResponse> {
  return invoke('model_chat', { request });
}

export async function modelChatStream(
  request: ChatCompletionRequest,
  onChunk: (chunk: ChatCompletionChunk) => void,
  onDone: () => void,
  onError: (error: Error) => void
): Promise<() => void> {
  // For streaming, we'll use the event system
  // The actual streaming is handled by the backend emitting events
  const unlistenChunk = await window.__TAURI__.event.listen('model-chat-chunk', (event) => {
    onChunk(event.payload as ChatCompletionChunk);
  });
  
  const unlistenDone = await window.__TAURI__.event.listen('model-chat-done', () => {
    onDone();
  });
  
  const unlistenError = await window.__TAURI__.event.listen('model-chat-error', (event) => {
    onError(new Error(event.payload as string));
  });
  
  // Start the stream
  invoke('model_chat_stream', { request }).catch((err) => {
    onError(new Error(err));
  });
  
  // Return cleanup function
  return () => {
    unlistenChunk();
    unlistenDone();
    unlistenError();
  };
}

export async function modelStart(): Promise<void> {
  return invoke('model_start');
}

export async function modelStop(): Promise<void> {
  return invoke('model_stop');
}