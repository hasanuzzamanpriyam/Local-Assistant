# Phase 4 — Voice Design Specification

**Project Path:** `C:\Users\CT\Desktop\Assistant`  
**Date:** 2026-11-11  
**Phase:** 4 of 4 (Core → Computer Control → Overlay/Polish → Voice)

---

## 1. Overview

Phase 4 adds **local voice I/O** to the assistant: Speech-to-Text (STT) for user input and Text-to-Speech (TTS) for assistant responses. All processing runs locally — no cloud APIs.

**Target:** Windows 11, HP EliteBook 845 G8 (16 GB RAM, Ryzen 5000, integrated Radeon). All-local, no cloud dependencies.

---

## 2. Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Tauri App (Rust + WebView2)                │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │  Frontend   │◄─►│  Tauri Core│◄─►│  Whisper    │              │
│  │  (Svelte 5) │  │  (Commands, │  │  (STT proc) │              │
│  │  Web Audio  │  │   Events)   │  │  Piper      │              │
│  │  (Mic +     │  │             │  │  (TTS proc) │              │
│  │   Playback) │  └──────┬──────┘  └──────┬──────┘              │
│  └─────────────┘         │             │                       │
│                         ▼             ▼                       │
│              ┌─────────────────────┐                           │
│              │  STT Command        │                           │
│              │  (audio → text)     │                           │
│              │  TTS Command        │                           │
│              │  (text → audio)     │                           │
│              └─────────────────────┘                           │
└─────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
                    ┌─────────────────────────────┐
                    │   Existing Chat Pipeline    │
                    │  (llama.cpp + tools + UI)   │
                    └─────────────────────────────┘
```

**Key Decisions:**
- **STT:** Whisper.cpp Small via Tauri-managed process (same pattern as llama.cpp server). Audio captured in frontend (Web Audio API), sent to Tauri command, Whisper.cpp transcribes, text returned.
- **TTS:** Piper TTS — lightweight, fast, good quality, runs locally. Text from chat response → Tauri command → Piper synthesizes → audio played in frontend.
- **VAD:** Silero VAD for voice activity detection — determines when user starts/stops speaking, enabling hands-free turn-taking.
- **Audio:** Web Audio API in frontend — `MediaRecorder`/`AudioContext` for mic capture, `AudioElement`/`AudioWorklet` for playback.
- **Push-to-talk default** (simpler than VAD for MVP), with VAD as optional enhancement.

---

## 3. Native Commands (Tauri ↔ Frontend)

| Command | Input | Output |
|---------|-------|--------|
| `voice.stt` | `{audio_base64: string, format: string}` | `{text: string}` |
| `voice.tts` | `{text: string, voice?: string}` | `{audio_base64: string, format: string}` |
| `voice.stt_start` | — | `{ok: boolean}` (start Whisper process) |
| `voice.stt_stop` | — | `{ok: boolean}` |
| `voice.tts_start` | — | `{ok: boolean}` (start Piper process) |
| `voice.tts_stop` | — | `{ok: boolean}` |
| `voice.vad_start` | — | `{ok: boolean}` (start Silero VAD) |
| `voice.vad_stop` | — | `{ok: boolean}` |

---

## 4. Frontend Components

| Component | Responsibility |
|-----------|---------------|
| `src/lib/stores/voice.ts` | Reactive store: isListening, isSpeaking, sttStatus, ttsStatus, transcript, audioLevel |
| `src/lib/api/voice.ts` | Tauri invoke wrappers: stt, tts, start/stop processes |
| `src/lib/components/voice/VoiceButton.svelte` | Push-to-talk button (hold to speak, release to transcribe) |
| `src/lib/components/voice/VoiceStatus.svelte` | Visual indicator: listening/speaking/idle states |
| `src/lib/components/voice/VoiceWaveform.svelte` | Real-time audio level visualization |

---

## 5. Data Flow (Voice Input)

```
User speaks → Frontend mic capture (Web Audio API)
       │
       ▼
Audio chunks → VAD detects speech start
       │
       ▼
User releases push-to-talk (or VAD detects silence)
       │
       ▼
Frontend: audio_base64 → `voice.stt` Tauri command
       │
       ▼
Whisper.cpp: audio → text
       │
       ▼
Text → existing chat pipeline (sendMessage) → Assistant response
       │
       ▼
Response text → `voice.tts` Tauri command
       │
       ▼
Piper: text → audio_base64
       │
       ▼
Frontend: play audio via Web Audio API
```

---

## 6. Error Handling & Resilience

| Scenario | Handling |
|----------|----------|
| Whisper.cpp not available | Fall back to text-only input, notify user |
| Microphone permission denied | Show error, offer text input fallback |
| TTS synthesis fails | Display text response without audio |
| Audio device unavailable | Log warning, skip voice features |
| STT/TTS process crash | Auto-respawn (max 3), notify frontend |

---

## 7. Configuration

Add to `config.toml` under `[voice]`:

```toml
[voice]
enabled = true
stt_model = "whisper-small"
tts_voice = "en_US-amy-medium"
vad_enabled = false
push_to_talk = true
```

---

## 8. Success Criteria

| Metric | Target |
|--------|--------|
| STT latency (audio → text) | < 2 s |
| TTS latency (text → audio) | < 1 s |
| End-to-end voice response | < 5 s |
| Whisper.cpp RAM | < 1 GB |
| Piper RAM | < 200 MB |
| Microphone permission | Requested on first voice use |

---

## 9. Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Whisper.cpp binary not bundled | High | STT unavailable | Download model binary on first run, clear error message |
| Microphone permissions | Medium | Voice input blocked | Graceful fallback to text input |
| Audio latency | High | Poor UX | Stream audio in chunks, optimize buffer sizes |
| Piper voice quality | Low | Robotic voice | Allow voice selection, document options |
| Concurrent STT+TTS | Medium | Audio feedback | Disable mic while TTS playing (echo prevention) |

---

## 10. Out of Scope (Phase 4)

- Wake word detection (always-listening)
- Voice cloning / custom voices
- Multi-language STT/TTS (English only initially)
- Voice commands for system control (separate from chat)
- Real-time streaming STT (chunked transcription)