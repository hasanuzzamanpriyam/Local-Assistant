use tauri::command;
use serde::{Deserialize, Serialize};

const BASE64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(input: &[u8]) -> String {
    let mut result = String::new();
    for chunk in input.chunks(3) {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        let triple = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);
        result.push(BASE64_CHARS[((triple >> 18) & 63) as usize] as char);
        result.push(BASE64_CHARS[((triple >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            result.push(BASE64_CHARS[((triple >> 6) & 63) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(BASE64_CHARS[(triple & 63) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

#[derive(Deserialize)]
pub struct SttRequest {
    pub audio_base64: String,
    pub format: Option<String>,
}

#[derive(Serialize)]
pub struct SttResponse {
    pub text: String,
}

#[derive(Deserialize)]
pub struct TtsRequest {
    pub text: String,
    pub voice: Option<String>,
}

#[derive(Serialize)]
pub struct TtsResponse {
    pub audio_base64: String,
    pub format: String,
}

#[command]
pub async fn voice_stt(request: SttRequest) -> Result<SttResponse, String> {
    // Whisper.cpp STT — placeholder until binary is bundled
    let text = format!("{} chars received (Whisper not yet bundled)", request.audio_base64.len());
    Ok(SttResponse { text })
}

#[command]
pub async fn voice_tts(request: TtsRequest) -> Result<TtsResponse, String> {
    // Piper TTS — placeholder until binary is bundled
    let audio_bytes = request.text.as_bytes();
    let audio_base64 = base64_encode(audio_bytes);
    Ok(TtsResponse {
        audio_base64,
        format: "wav".to_string(),
    })
}

#[command]
pub async fn voice_stt_start() -> Result<(), String> {
    Ok(())
}

#[command]
pub async fn voice_stt_stop() -> Result<(), String> {
    Ok(())
}

#[command]
pub async fn voice_tts_start() -> Result<(), String> {
    Ok(())
}

#[command]
pub async fn voice_tts_stop() -> Result<(), String> {
    Ok(())
}
