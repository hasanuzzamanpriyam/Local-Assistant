# Phase 2 — Computer Control Design Specification

**Project Path:** `C:\Users\CT\Desktop\Assistant`  
**Date:** 2026-11-11  
**Phase:** 2 of 4 (Core → Computer Control → Overlay/Polish → Voice)

---

## 1. Overview

Phase 2 adds **computer control** to the assistant: the AI can observe the screen, click, type, scroll, and run shell commands on the user's desktop. A Python sidecar process handles screen capture, input automation, OCR, and an agent loop. Tauri manages the sidecar lifecycle and bridges NDJSON messages between the frontend and the Python process.

**Target:** HP EliteBook 845 G8 — Ryzen 5000, 16 GB RAM, Windows 11. All-local, no cloud dependencies.

---

## 2. Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Tauri App (Rust + WebView2)                │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │  Frontend   │◄─►│  Tauri Core│◄─►│ Sidecar     │              │
│  │  (Svelte 5) │  │  (Commands, │  │  Manager    │              │
│  │             │  │   Events)   │  │  (spawn,    │              │
│  └─────────────┘  └──────┬──────┘  │  monitor,   │              │
│                         │         │  NDJSON)    │              │
│                         ▼         └──────┬──────┘              │
│              ┌─────────────────────┐    │                     │
│              │  Sidecar Store / API│    │                     │
│              │  (TS frontend)      │    │                     │
│              └─────────────────────┘    │                     │
└─────────────────────────────────────────┼────────────────────┘
                                          │ NDJSON (stdin/stdout)
                                          ▼
                    ┌─────────────────────────────────────┐
                    │        Python Sidecar Process       │
                    │  ┌─────────┐ ┌─────────┐           │
                    │  │ runner  │ │ protocol│           │
                    │  │ (entry) │ │ (NDJSON)│           │
                    │  └─────────┘ └─────────┘           │
                    │  ┌─────────┐ ┌─────────┐           │
                    │  │ screen  │ │  input  │           │
                    │  │ (mss)   │ │(pyauto) │           │
                    │  └─────────┘ └─────────┘           │
                    │  ┌─────────┐ ┌─────────┐           │
                    │  │   ocr   │ │ actions │           │
                    │  │(tesseract)│ │executor │           │
                    │  └─────────┘ └─────────┘           │
                    │  ┌─────────────────────────┐       │
                    │  │      agent.py           │       │
                    │  │  observe → plan → act   │       │
                    │  └─────────────────────────┘       │
                    └─────────────────────────────────────┘
```

**Key Decisions:**
- **Python sidecar** — Separate process, spawned by Tauri. NDJSON over stdin/stdout for IPC.
- **Embedded venv via `uv`** — Self-contained Python environment, no system Python required. `uv` installs `mss`, `pyautogui`, `pywin32`, `pytesseract`.
- **Per-step screenshots** — Agent captures a screenshot on every step for visual context. Trade-off: slower than on-demand, but enables the agent to see UI changes after each action.
- **Agent loop** — observe (screenshot + OCR) → plan (LLM decides next action) → act (execute via pyautogui) → repeat until task complete or max_steps reached.

---

## 3. NDJSON Protocol (Tauri ↔ Python Sidecar)

### 3.1 Tauri → Sidecar (stdin)

| Command | Payload | Description |
|---------|---------|-------------|
| `run` | `{cmd:"run", id:string, task:string, max_steps:number}` | Start a computer control task |
| `stop` | `{cmd:"stop", id:string}` | Stop a running task |
| `ping` | `{cmd:"ping"}` | Health check |
| `shutdown` | `{cmd:"shutdown"}` | Graceful shutdown |

### 3.2 Sidecar → Tauri (stdout)

| Event | Payload | Description |
|-------|---------|-------------|
| `booting` | `{event:"booting", screen:{width,height}, pid:number}` | Sidecar starting |
| `loading` | `{event:"loading", elapsed:number, stage:string}` | Loading progress |
| `ready` | `{event:"ready", screen:{width,height}, seconds:number}` | Sidecar ready |
| `load_error` | `{event:"load_error", message:string}` | Startup failure |
| `task_started` | `{event:"task_started", id:string}` | Task began |
| `step` | `{event:"step", id:string, step:number, max_steps:number, thought:string, action:string, screenshot_base64?:string}` | Agent step result |
| `task_complete` | `{event:"task_complete", id:string, success:boolean, result:string, steps:number, duration_ms:number}` | Task finished |
| `task_stopped` | `{event:"task_stopped", id:string, reason:string}` | Task stopped by user |
| `pong` | `{event:"pong"}` | Response to ping |
| `error` | `{event:"error", message:string, fatal:boolean}` | Error (fatal or recoverable) |

---

## 4. Native Commands (Tauri ↔ Frontend)

| Command | Input | Output |
|---------|-------|--------|
| `sidecar.status` | — | `{running:boolean, pid?:number, ready:boolean}` |
| `sidecar.start` | — | `{ok:boolean}` |
| `sidecar.stop` | — | `{ok:boolean}` |
| `sidecar.run_task` | `{task:string, max_steps?:number}` | `{task_id:string}` |
| `sidecar.stop_task` | `{id:string}` | `{ok:boolean}` |
| `sidecar.task_events` | `{task_id:string}` | Stream of NDJSON task events (Tauri event) |

---

## 5. Python Sidecar Modules

### 5.1 `sidecar/protocol.py`
NDJSON message types, encoding/decoding, validation. Shared schema between Tauri and Python.

### 5.2 `sidecar/runner.py`
Main entry point. NDJSON stdin/stdout loop. Command dispatch:
- `run` → spawn agent task
- `stop` → cancel task
- `ping` → respond `pong`
- `shutdown` → exit cleanly

Manages a registry of active tasks, each running its own agent loop.

### 5.3 `sidecar/screen.py`
Screen capture using `mss`:
- `capture()` → base64 PNG of primary monitor
- `list_monitors()` → `[{id, x, y, width, height}]`
- `capture_region(x, y, w, h)` → base64 PNG of region

### 5.4 `sidecar/input.py`
Mouse/keyboard automation using `pyautogui` + `pywin32`:
- `click(x, y)`, `double_click(x, y)`, `right_click(x, y)`
- `type(text)`, `press(key)`, `hotkey(*keys)`
- `scroll(x, y, clicks)`, `drag(x1, y1, x2, y2)`
- `move(x, y)`
- `get_cursor_pos()` → `{x, y}`

### 5.5 `sidecar/ocr.py`
Text extraction using `pytesseract`:
- `extract_text(image)` → list of `{text, x, y, width, height, confidence}`
- `find_text(image, query)` → list of matches with coordinates

### 5.6 `sidecar/actions.py`
High-level action executor:
- `click_element(selector_or_text)` — find by OCR or coordinates
- `type_into(text, target)` — focus then type
- `scroll_page(direction)` — scroll active window
- `wait(seconds)` — pause
- `screenshot()` — capture current screen
- `hotkey_combo(keys)` — keyboard shortcuts

### 5.7 `sidecar/agent.py`
Agent loop (ReAct pattern):
```
observe → think → act → observe → ...
```
- `observe()`: capture screenshot + OCR
- `think(task, observations, history)`: call LLM (via llama.cpp through Tauri? No — sidecar needs its own LLM access)

Wait — the sidecar needs to call the LLM for the agent loop. The sidecar can make HTTP requests directly to the llama.cpp server (same one Tauri manages), OR Tauri can proxy. The cleanest approach: sidecar makes HTTP requests directly to the llama.cpp server on localhost (it knows the port from Tauri's startup info, or discovers it).

Actually, looking at the current architecture: Tauri starts the llama.cpp server and manages it. The sidecar is a separate process. For the agent loop, the sidecar needs to call the LLM. Options:

1. **Sidecar calls llama.cpp directly** — Sidecar knows the port (could be passed as an argument from Tauri). Simplest, no extra complexity.
2. **Tauri proxies LLM calls** — Sidecar sends NDJSON to Tauri asking for LLM completion, Tauri forwards to llama.cpp. More architecture but keeps LLM access centralized.

I'll go with **option 1** (sidecar calls llama.cpp directly) — Tauri passes the llama.cpp server URL as a command-line argument when spawning the sidecar. This keeps the agent loop self-contained in the sidecar.

Agent loop max: `max_steps` (default 10). On each step:
1. Capture screenshot
2. Run OCR
3. Build prompt: system + task + screenshot description + OCR text + history
4. Call LLM → get next action (click, type, scroll, wait, done)
5. Execute action
6. Repeat

---

## 6. Frontend Components

| Component | Responsibility |
|-----------|---------------|
| `src/lib/stores/sidecar.ts` | Reactive store: sidecar status, active tasks, task events |
| `src/lib/api/sidecar.ts` | Tauri invoke wrappers: status, start, stop, run_task, stop_task |
| `src/components/sidecar/SidecarStatus.svelte` | Sidecar health indicator (running/ready/error) |
| `src/components/sidecar/TaskInput.svelte` | Input to start a new computer control task |
| `src/components/sidecar/TaskFeed.svelte` | Live stream of step events with screenshots |
| `src/components/sidecar/ScreenshotViewer.svelte` | Display base64 screenshot from agent step |

### 6.1 Sidecar Store

```typescript
export interface SidecarStatus {
  running: boolean;
  ready: boolean;
  pid?: number;
}

export interface TaskEvent {
  id: string;
  event: string;
  step?: number;
  max_steps?: number;
  thought?: string;
  action?: string;
  screenshot_base64?: string;
  result?: string;
  success?: boolean;
  duration_ms?: number;
  error?: string;
}

export interface ActiveTask {
  id: string;
  task: string;
  max_steps: number;
  status: 'running' | 'completed' | 'failed' | 'stopped';
  events: TaskEvent[];
}
```

---

## 7. Data Flow (Computer Control Task)

```
User: "Open Notepad and type hello"
       │
       ▼
Frontend: sidecar.run_task({task, max_steps: 10})
       │
       ▼
Tauri command: sidecar_run_task → spawns/wakes sidecar
       │
       ▼
Tauri: NDJSON {"cmd":"run", id, task, max_steps} → Python stdin
       │
       ▼
Python Runner: dispatch to agent task
       │
       ▼
Python Agent Loop:
  1. screenshot (mss) → base64
  2. OCR (tesseract) → text
  3. LLM call (llama.cpp directly) → action
  4. Execute action (pyautogui)
  5. Emit step event via NDJSON stdout
  6. Repeat until complete or max_steps
       │
       ▼
Step events flow: NDJSON stdout → Tauri → Tauri event → Frontend store
       │
       ▼
Frontend: TaskFeed displays step events + screenshots live
       │
       ▼
task_complete → Frontend shows final result
```

---

## 8. Error Handling & Resilience

| Scenario | Handling |
|----------|----------|
| Sidecar crashes | Tauri detects stdout close → auto-respawn (max 3), notify frontend |
| Sidecar fails to start | Emit `load_error` event, frontend shows error |
| Action fails (click missed) | Agent retries with slight offset; after 3 failures → report to LLM for replan |
| LLM unavailable | Agent emits `error` event, task marked failed |
| max_steps reached | Task stopped, marked as incomplete with partial result |
| User stops task | `task_stopped` event, sidecar kills agent loop |
| OCR returns no text | Agent proceeds with screenshot-only context |

---

## 9. Python Dependencies (uv-managed venv)

| Package | Purpose |
|---------|---------|
| `mss` | Multi-monitor screenshot capture |
| `pyautogui` | Mouse/keyboard automation |
| `pywin32` | Windows-specific input APIs |
| `pytesseract` | OCR text extraction |
| `requests` | HTTP client for llama.cpp calls |
| `Pillow` | Image processing (screenshots) |

**Note:** Tesseract OCR engine binary must be installed on Windows separately (tesseract-ocr-w64). The sidecar gracefully degrades if tesseract is not found (screenshot-only agent mode).

---

## 10. Configuration

Sidecar settings added to `config.toml` under `[sidecar]`:

```toml
[sidecar]
python = "python"
auto_start = true
log_level = "INFO"
venv_path = ""  # empty = auto-create via uv
max_steps = 10
screenshot_quality = 85
```

---

## 11. Success Criteria

| Metric | Target |
|--------|--------|
| Sidecar spawn + ready | < 5 s |
| Screenshot capture | < 100 ms |
| Action execution (click) | < 50 ms |
| Agent step (observe→act) | < 2 s |
| Task completion (simple) | < 30 s |
| RAM overhead (sidecar) | < 500 MB |

---

## 12. Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Python deps missing | Medium | Sidecar won't start | `uv` venv auto-creation; clear error messages |
| Tesseract not installed | High | OCR unavailable | Graceful degradation to screenshot-only mode |
| pyautogui fails on secure desktop | Medium | Can't click admin apps | Document limitation; run as user |
| Agent infinite loop | Medium | High CPU/RAM | `max_steps` hard limit (default 10) |
| Sidecar zombie process | Low | Resource leak | Tauri kills sidecar on app exit |
| Port conflict for llama.cpp | Low | Agent can't call LLM | Tauri passes known port to sidecar |

---

## 13. Out of Scope (Phase 2)

- Custom system cursor / overlay (Phase 3)
- Esc Guard (Phase 3)
- Voice I/O (Phase 4)
- Multi-monitor simultaneous control
- Persistent task history across restarts