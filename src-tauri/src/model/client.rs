use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use url::Url;
use futures::{Stream, stream};
use std::pin::Pin;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatChunkChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChunkChoice {
    pub index: u32,
    pub delta: ChatMessageDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallDelta>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallDelta {
    pub index: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<FunctionCallDelta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

#[derive(Debug)]
pub struct LlamaClient {
    base_url: Url,
    client: Client,
}

impl LlamaClient {
    pub fn new(base_url: Url) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        
        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            base_url,
            client,
        }
    }

    pub async fn health_check(&self) -> Result<bool> {
        let url = self.base_url.join("/health")?;
        
        match self.client.get(url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    pub async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        let url = self.base_url.join("/chat/completions")?;
        
        let response = self.client
            .post(url)
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("API error: {}", error_text));
        }
        
        let completion: ChatCompletionResponse = response.json().await?;
        Ok(completion)
    }

    pub async fn chat_completion_stream(
        &self,
        mut request: ChatCompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk>> + Send + 'static>>> {
        request.stream = Some(true);
        
        let url = self.base_url.join("/chat/completions")?;
        
        let response = self.client
            .post(url)
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("API error: {}", error_text));
        }
        
        // Get the full response body as bytes and parse SSE
        let body = response.bytes().await?;
        let chunks = Self::parse_sse_chunk(&body);
        
        let stream = stream::iter(chunks.into_iter().map(Ok));
        
        Ok(Box::pin(stream))
    }

    fn parse_sse_chunk(chunk: &[u8]) -> Vec<ChatCompletionChunk> {
        let text = String::from_utf8_lossy(chunk);
        let mut results = Vec::new();
        
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with("data: ") {
                let data = &line[6..];
                if data == "[DONE]" {
                    continue;
                }
                if let Ok(chunk) = serde_json::from_str::<ChatCompletionChunk>(data) {
                    results.push(chunk);
                }
            }
        }
        
        results
    }

    #[allow(dead_code)]
    pub async fn list_models(&self) -> Result<Vec<String>> {
        let url = self.base_url.join("/models")?;
        
        let response = self.client.get(url).send().await?;
        
        if !response.status().is_success() {
            return Ok(vec![]);
        }
        
        #[derive(Deserialize)]
        struct ModelsResponse {
            data: Vec<ModelData>,
        }
        
        #[derive(Deserialize)]
        struct ModelData {
            id: String,
        }
        
        let models: ModelsResponse = response.json().await?;
        Ok(models.data.into_iter().map(|m| m.id).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_serialization() {
        let msg = ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        };
        
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"content\":\"Hello\""));
    }

    #[test]
    fn test_chat_completion_request_serialization() {
        let req = ChatCompletionRequest {
            model: "test-model".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            }],
            temperature: Some(0.7),
            top_p: Some(0.9),
            max_tokens: Some(100),
            stream: Some(false),
            tools: None,
            tool_choice: None,
        };
        
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"model\":\"test-model\""));
        assert!(json.contains("\"temperature\":0.7"));
        assert!(json.contains("\"stream\":false"));
    }

    #[test]
    fn test_chat_completion_response_deserialization() {
        let json = r#"{
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created": 1234567890,
            "model": "test-model",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello there!"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5,
                "total_tokens": 15
            }
        }"#;
        
        let resp: ChatCompletionResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.id, "chatcmpl-123");
        assert_eq!(resp.choices[0].message.content, "Hello there!");
        assert_eq!(resp.usage.as_ref().unwrap().total_tokens, 15);
    }

    #[test]
    fn test_tool_call_serialization() {
        let tool_call = ToolCall {
            id: "call_123".to_string(),
            tool_type: "function".to_string(),
            function: FunctionCall {
                name: "get_weather".to_string(),
                arguments: "{\"location\":\"Tokyo\"}".to_string(),
            },
        };
        
        let json = serde_json::to_string(&tool_call).unwrap();
        assert!(json.contains("\"id\":\"call_123\""));
        assert!(json.contains("\"name\":\"get_weather\""));
    }
}