# Assistant Full Clone — Design Specification

**Project Path:** `C:\Users\CT\Desktop\Assistant`  
**Date:** 2026-11-11  
**Target Hardware:** HP EliteBook 845 G8 — Ryzen 5000 (Zen 3), 16 GB RAM, 500 GB NVMe, integrated Radeon, Windows 11  
**Constraint:** All-local, free, smooth on modest hardware (no discrete GPU)

---

## 1. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Tauri App (Rust + WebView2)              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │  Frontend   │  │  Tauri Core │  │  Native     │             │
│  │  (Svelte 5) │◄─►│  (Commands, │◄─►│  Commands   │             │
│  │             │  │   Events)   │  │  (FS, Shell,│             │
│  └─────────────┘  └──────┬──────┘  │   Screen,   │             │
│                         │         │   Model)    │             │
│                         ▼         └──────┬──────┘             │
│              ┌─────────────────────┐    │                    │
│              │  Sidecar Manager    │    │                    │
│              │  (spawns/monitors   │    │                    │
│              │   Python sidecar)   │    │                    │
│              └──────────┬──────────┘    │                    │
│                         │ NDJSON        │                    │
│                         ▼               │                    │
│              ┌─────────────────────┐    │                    │
│              │  Python Sidecar     │    │                    │
│              │  • Screen capture   │    │                    │
│              │  • Mouse/Keyboard   │    │                    │
│              │  • OCR (tesseract)  │    │                    │
│              │  • Action executor  │    │                    │
│              └─────────────────────┘    │                    │
└─────────────────────────────────────────│────────────────────┘
                                          │
                    ┌─────────────────────┴─────────────────────┐
                    ▼                                           ▼
            ┌───────────────┐                           ┌───────────────┐
            │  llama.cpp    │                           │  System APIs  │
            │  Server       │                           │  (Win32,      │
            │  (GGUF models)│                           │   uiautomation)│
            └───────────────┘                           └───────────────┘
```

**Key Decisions:**
- **Tauri over Electron** — ~100 MB vs ~300 MB; WebView2 uses system Edge; Rust native commands for heavy lifting
- **llama.cpp server** — Lightest inference; GGUF models; OpenAI-compatible API; runs as separate process
- **Svelte 5** — Compiled, no VDOM, smallest bundle (~20 KB gzipped), great for Tauri
- **Python sidecar** — Only for computer control (screen, input, OCR); ~80 MB with deps
- **4B model default (Qwen2.5-3B-Instruct or Phi-3.5-mini)** — ~2.5 GB RAM, 30-50 tok/s on CPU; 7B optional upgrade

---

## 2. Components

### 2.1 Tauri Application (`src-tauri/`)

| Module | Responsibility |
|--------|----------------|
| `main.rs` | App entry, sidecar lifecycle, global state |
| `commands/` | Native commands exposed to frontend: `fs`, `shell`, `screen`, `model`, `sidecar` |
| `sidecar/` | Spawns Python sidecar, manages NDJSON stdin/stdout, handles reconnection |
| `model/` | llama.cpp server management (start/stop/health), chat completion proxy |
| `ipc/` | NDJSON protocol types, serialization, request/response correlation |
| `overlay/` | (Phase 3) Native overlay window for cursor takeover, glow, HUD |
| `esc_guard/` | (Phase 3) Low-level keyboard hook (WH_KEYBOARD_LL) for real Esc detection |

### 2.2 Frontend (`src/` — Svelte 5 + TypeScript)

| Module | Responsibility |
|--------|----------------|
| `lib/stores/` | Reactive state: chat, tools, sidecar status, model status, settings |
| `lib/api/` | Tauri command wrappers, llama.cpp HTTP client, sidecar NDJSON client |
| `components/chat/` | Message list, input, streaming renderer, tool call display |
| `components/tools/` | Tool confirmation UI, file tree, terminal output, screenshot viewer |
| `components/overlay/` | (Phase 3) HUD overlay components, cursor preview |
| `components/settings/` | Model selection, sidecar config, prompts, keybindings |

### 2.3 Python Sidecar (`sidecar/`)

| Module | Responsibility |
|--------|----------------|
| `runner.py` | Main entry, NDJSON loop, command dispatch, agent loop |
| `screen.py` | `mss` screenshot capture, monitor enumeration, region capture |
| `input.py` | `pyautogui` + `pywin32` mouse/keyboard/actions |
| `ocr.py` | `pytesseract` text extraction, screen search |
| `actions.py` | High-level actions: `click`, `type`, `scroll`, `drag`, `wait`, `find_text` |
| `agent.py` | Minimal agent loop: observe → plan → act → observe (tool-calling LLM) |
| `protocol.py` | NDJSON message types, validation, encoding |

### 2.4 llama.cpp Server (External Process)

- Managed by Tauri (start/stop on demand)
- Model: `Qwen2.5-3B-Instruct-Q4_K_M.gguf` (default) or `Phi-3.5-mini-instruct-Q4_K_M.gguf`
- OpenAI-compatible `/v1/chat/completions` endpoint
- Context: 4096 tokens (configurable)
- Threads: `n_threads = physical_cores` (8 on 845 G8)

---

## 3. Data Flow

### 3.1 Chat + Tool Use (Text Mode)

```
User types message
       │
       ▼
Frontend: append to chat store, stream indicator
       │
       ▼
Tauri command: `model.chat_completion(messages, tools)`
       │
       ▼
llama.cpp server: completion with tool schema
       │
       ▼
Tool calls returned → Tauri executes native commands
       │
       ▼
Results appended to messages → next completion
       │
       ▼
Final response streamed to frontend
```

### 3.2 Computer Control Task

```
User: "Open Notepad and type hello"
       │
       ▼
Frontend → Tauri `sidecar.run_task(task, max_steps)`
       │
       ▼
Sidecar Manager: NDJSON `{"cmd":"run","id":"t1","task":"..."}` → Python stdin
       │
       ▼
Python Runner: Agent loop (screenshot → LLM → action → repeat)
       │
       ▼
Step events: NDJSON `{"event":"step",...}` → Tauri stdout → Frontend (live HUD)
       │
       ▼
Task complete → `{"event":"task_complete",...}` → Frontend shows result
```

### 3.3 Overlay & Esc Guard (Phase 3)

```
Task starts
       │
       ├─► Tauri: Create overlay window (WS_EX_LAYERED | WS_EX_TRANSPARENT | WDA_EXCLUDEFROMCAPTURE)
       │        ├─► Render aura glow (animated, GPU via WebView2)
       │        ├─► Render comet trail (follows cursor)
       │        └─► Render HUD (click-through, top-center)
       │
       ├─► Tauri: Install WH_KEYBOARD_LL hook (Esc Guard)
       │        ├─► Real Esc (LLKHF_INJECTED=0) → stop task, restore cursor
       │        └─► Injected Esc → pass through
       │
       └─► Python: SetSystemCursor → agent cursor (custom .cur)
```

---

## 4. NDJSON Protocol (Tauri ↔ Python Sidecar)

### 4.1 Electron → Sidecar (stdin)

| Command | Payload |
|---------|---------|
| `run` | `{cmd:"run", id:string, task:string, max_steps:number, overlay:boolean, fast:boolean}` |
| `stop` | `{cmd:"stop", id:string}` |
| `ping` | `{cmd:"ping"}` |
| `shutdown` | `{cmd:"shutdown"}` |

### 4.2 Sidecar → Electron (stdout)

| Event | Payload |
|-------|---------|
| `booting` | `{event:"booting", screen:{width,height}, pid:number}` |
| `loading` | `{event:"loading", elapsed:number, stage:string}` |
| `ready` | `{event:"ready", screen:{width,height}, seconds:number}` |
| `load_error` | `{event:"load_error", message:string}` |
| `task_started` | `{event:"task_started", id:string}` |
| `step` | `{event:"step", id:string, step:number, max_steps:number, thought:string, action:string, screenshot_base64?:string}` |
| `task_complete` | `{event:"task_complete", id:string, success:boolean, result:string, steps:number, duration_ms:number}` |
| `task_stopped` | `{event:"task_stopped", id:string, reason:string}` |
| `pong` | `{event:"pong"}` |
| `error` | `{event:"error", message:string, fatal:boolean}` |

---

## 5. Native Commands (Tauri ↔ Frontend)

| Command | Input | Output |
|---------|-------|--------|
| `fs.read` | `path:string` | `string` |
| `fs.write` | `path:string, content:string` | `void` |
| `fs.edit` | `path:string, old:string, new:string` | `void` |
| `fs.glob` | `pattern:string` | `string[]` |
| `fs.grep` | `pattern:string, path?:string` | `{file:string, line:number, text:string}[]` |
| `shell.exec` | `cmd:string, args:string[], cwd?:string` | `{stdout:string, stderr:string, code:number}` |
| `shell.background` | `cmd:string, args:string[]` | `job_id:string` |
| `screen.capture` | `monitor?:number, region?:{x,y,w,h}` | `base64_png` |
| `screen.monitors` | — | `{id,x,y,width,height}[]` |
| `model.chat` | `messages:Message[], tools?:Tool[], stream?:boolean` | `Message` or `AsyncIterable<Chunk>` |
| `model.list` | — | `{name,size,path}[]` |
| `model.pull` | `name:string` | `progress` events |
| `sidecar.status` | — | `{running:boolean, pid?:number, ready:boolean}` |
| `sidecar.run_task` | `task:string, max_steps?:number` | `TaskResult` |
| `sidecar.stop_task` | `id:string` | `void` |

---

## 6. Models (Local GGUF)

| Model | Size (Q4_K_M) | RAM | Speed (tok/s) | Best For |
|-------|---------------|-----|---------------|----------|
| **Qwen2.5-3B-Instruct** | ~2.0 GB | 2.5 GB | 40-55 | General chat, tool use, coding |
| **Phi-3.5-mini-instruct** | ~2.3 GB | 2.8 GB | 35-50 | Reasoning, instruction following |
| **SmolLM2-3B-Instruct** | ~1.9 GB | 2.4 GB | 45-60 | Speed, simple tasks |
| **Qwen2.5-7B-Instruct** | ~4.4 GB | 5.0 GB | 18-28 | Complex reasoning, better tool use |
| **Llama-3.2-3B-Instruct** | ~2.0 GB | 2.5 GB | 40-55 | Balanced |

**Default:** Qwen2.5-3B-Instruct-Q4_K_M (best tool use in 3B class)  
**Download:** Hugging Face `bartowski/Qwen2.5-3B-Instruct-GGUF` → `Qwen2.5-3B-Instruct-Q4_K_M.gguf`

---

## 7. Phased Implementation

### Phase 1 — Core (Weeks 1-2)
- [ ] Tauri + Svelte 5 project setup
- [ ] llama.cpp server management (start/stop/health, model download)
- [ ] Chat UI with streaming, message history (local JSONL)
- [ ] Native FS commands (read/write/edit/glob/grep)
- [ ] Native Shell commands (exec, background)
- [ ] Model chat completion (OpenAI-compatible API)
- [ ] Settings: model selection, system prompt, temperature

### Phase 2 — Computer Control (Weeks 2-3)
- [ ] Python sidecar structure (NDJSON, runner, protocol)
- [ ] Screen capture (mss), monitor enumeration
- [ ] Mouse/keyboard actions (pyautogui + pywin32)
- [ ] OCR (pytesseract) — find text, click text
- [ ] Action executor (click, type, scroll, drag, wait, find_text)
- [ ] Minimal agent loop (observe → LLM → act)
- [ ] Tauri sidecar manager (spawn, monitor, NDJSON bridge)
- [ ] Frontend: task input, live step feed, screenshot viewer

### Phase 3 — Overlay & Polish (Weeks 3-4)
- [ ] Native overlay window (WebView2, layered, capture-excluded)
- [ ] Aura glow (CSS animations, GPU accelerated)
- [ ] Comet trail (canvas, follows cursor)
- [ ] HUD (step, thought, time, expandable details)
- [ ] Custom system cursor (SetSystemCursor, .cur resource)
- [ ] Esc Guard (WH_KEYBOARD_LL hook, real vs injected detection)
- [ ] Cleanup: restore cursor on exit, atexit, Ctrl+Alt+Esc fallback

### Phase 4 — Voice (Optional, Later)
- [ ] Whisper.cpp (local STT) + Piper/Kokoro (local TTS)
- [ ] WebRTC or WebSocket audio streaming
- [ ] VAD (Silero) + turn detection
- [ ] Same agent brain, voice I/O

---

## 8. Error Handling & Resilience

| Scenario | Handling |
|----------|----------|
| Sidecar crashes | Tauri detects stdout close → auto-respawn (max 3), notify frontend |
| llama.cpp server down | Health check every 10s → auto-restart, queue pending requests |
| Model OOM | Catch `CUDA out of memory` / alloc failure → fallback to smaller model, notify |
| Action fails (click missed) | Agent retries with offset; after 3 failures → report to LLM for replan |
| Overlay creation fails | Log, continue without overlay (headless mode) |
| Esc Guard hook fails | Fallback to Ctrl+Alt+Esc (Tauri global shortcut) |

---

## 9. Configuration

**`config.toml` (Tauri app dir):**
```toml
[model]
default = "Qwen2.5-3B-Instruct-Q4_K_M.gguf"
context_size = 4096
threads = 8
temperature = 0.3
top_p = 0.9

[sidecar]
python = "python"  # or full path to venv
auto_start = true
log_level = "INFO"

[overlay]
enabled = true
aura_color = "#007AFF"
comet_color = "#00D4FF"
hud_position = "top-center"

[esc_guard]
enabled = true
fallback_shortcut = "Ctrl+Alt+Esc"
```

---

## 10. Success Criteria

| Metric | Target |
|--------|--------|
| Cold start (app → ready) | < 3 s |
| Model first token (4B, CPU) | < 500 ms |
| Chat streaming latency | < 50 ms/token |
| Sidecar spawn + ready | < 5 s |
| Screenshot capture | < 100 ms |
| Action execution (click) | < 50 ms |
| RAM usage (idle, 4B model loaded) | < 7 GB |
| RAM usage (active task) | < 9 GB |

---

## 11. Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Rust learning curve | Medium | Delay Phase 1 | Start with `cargo tauri init` template; only ~200 lines Rust needed |
| WebView2 not installed | Low | App won't run | Tauri installer bundles WebView2 bootstrapper |
| llama.cpp server conflicts | Low | Port collision | Random port + file-based lock; health check |
| Python sidecar DLL hell | Medium | Import errors | Embedded venv via `uv` or `python-embedded` (like Stonic) |
| OCR accuracy (tesseract) | High | Missed text | Add `rapidocr_onnx` as optional upgrade; allow manual coordinate click |
| No GPU → slow 7B | High | Poor UX | Default 4B; 7B opt-in with warning |

---

## 12. Out of Scope (v1)

- Voice (Phase 4)
- Multi-monitor overlay sync
- Plugin/skill system (beyond built-in tools)
- Cloud sync / multi-device
- Mobile companion
- Advanced agent planning (ReAct only, no MCTS/tree search)

---

**Next Step:** Review this spec. If approved, I'll invoke `writing-plans` to create the detailed implementation plan with tasks, dependencies, and file-level breakdown.