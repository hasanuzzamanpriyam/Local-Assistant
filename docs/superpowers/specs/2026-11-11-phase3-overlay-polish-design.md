# Phase 3 — Overlay & Polish Design Specification

**Project Path:** `C:\Users\CT\Desktop\Assistant`  
**Date:** 2026-11-11  
**Phase:** 3 of 4 (Core → Computer Control → Overlay/Polish → Voice)

---

## 1. Overview

Phase 3 adds a **system-level overlay window** that provides visual feedback during computer control tasks, plus an **Esc Guard** to stop runaway agent tasks. The overlay draws a glowing aura, a comet trail following the cursor, and a HUD showing task progress — all floating above every other application.

**Target:** Windows 11, HP EliteBook 845 G8 (16 GB RAM). All-local, no cloud dependencies.

---

## 2. Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Tauri App (Rust + WebView2)                │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │  Main App   │◄─►│  Tauri Core │◄─►│  Overlay    │              │
│  │  (Svelte 5) │  │  (Events)   │  │  Window     │              │
│  └─────────────┘  └──────┬──────┘  │  (2nd WV2)  │              │
│                         │         └──────┬──────┘              │
│                         ▼              │                       │
│              ┌─────────────────────┐    │                       │
│              │  Global Shortcut    │◄───┘                       │
│              │  (Ctrl+Alt+Esc)     │    Tauri events             │
│              └─────────────────────┘    overlay-show/hide/update│
│                         │                                          │
│                         ▼                                          │
│              ┌─────────────────────┐                              │
│              │  Sidecar Manager    │                              │
│              │  (task lifecycle)   │                              │
│              └─────────────────────┘                              │
└─────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
                    ┌─────────────────────────────┐
                    │   Python Sidecar Process    │
                    │   (agent task running)      │
                    └─────────────────────────────┘
```

**Key Decisions:**
- **Separate overlay window** (not in-app layer) — required to draw over *all* applications during agent control
- **Ctrl+Alt+Esc global shortcut** via Tauri `global-shortcut` plugin — simpler than WH_KEYBOARD_LL hook, reliable, defer native hook to later if needed
- **Tauri events for IPC** — main app → overlay window via `window.emit` / `listen`
- **CSS animations** for aura glow + HTML5 Canvas for comet trail — GPU-accelerated via WebView2

---

## 3. Overlay Window Configuration

| Property | Value | Reason |
|----------|-------|--------|
| `title` | "Assistant Overlay" | Taskbar hidden, title irrelevant |
| `width`/`height` | full screen (primary monitor) | Covers entire screen |
| `alwaysOnTop` | true | Floats above all apps |
| `decorations` | false | No titlebar/border |
| `transparent` | true | See through to apps underneath |
| `skipTaskbar` | true | Don't clutter taskbar |
| `focus` / `focused` | false | Don't steal keyboard focus |
| `cursor` | none | Passthrough to underlying apps |
| `ignoreCursorEvents` | true | Click-through (passthrough clicks) |

---

## 4. Tauri Events (Main App ↔ Overlay)

| Event | Direction | Payload | Description |
|-------|-----------|---------|-------------|
| `overlay-show` | Main → Overlay | `{position: {x,y}, config: {aura_color, comet_color, hud_position}}` | Show overlay, position at cursor |
| `overlay-hide` | Main → Overlay | `{}` | Hide overlay |
| `overlay-update` | Main → Overlay | `{step, max_steps, thought, action, elapsed_ms}` | Update HUD + reposition aura/comet |

Overlay → Main: none needed for Phase 3 MVP (all control flows from main app).

---

## 5. Native Commands (Tauri ↔ Frontend)

| Command | Input | Output |
|---------|-------|--------|
| `overlay.show` | `{x, y}` | `{ok: boolean}` |
| `overlay.hide` | — | `{ok: boolean}` |
| `global_shortcut.register` | — | `{ok: boolean}` (registers Ctrl+Alt+Esc) |
| `global_shortcut.unregister` | — | `{ok: boolean}` |

Note: `overlay.show/hide` are implemented by emitting Tauri events to the overlay window, not direct Rust calls. The commands trigger the event emission.

---

## 6. Frontend Components

### 6.1 `src/lib/components/overlay/OverlayController.svelte`
Coordinates overlay lifecycle: listens to sidecar task events, shows/hides overlay window, positions aura at cursor.

### 6.2 `src/lib/components/overlay/OverlayWindow.svelte`
The overlay window content itself (runs in the separate Tauri window): aura glow canvas, comet trail canvas, HUD div.

### 6.3 `src/lib/stores/overlay.ts`
Reactive store: `isVisible`, `cursorPosition`, `currentStep`, `hudConfig`, `taskStatus`.

### 6.4 `src/lib/api/overlay.ts`
Tauri invoke wrappers: `overlayShow(x, y)`, `overlayHide()`.

---

## 7. Data Flow (During Agent Task)

```
Sidecar emits step event (NDJSON)
       │
       ▼
Main app TaskFeed receives event → updates overlay store
       │
       ▼
Main app emits `overlay-update` Tauri event to overlay window
       │
       ▼
Overlay window: reposition aura to cursor, update HUD text, draw comet frame
       │
       └─┬─ User presses Ctrl+Alt+Esc
         │   Tauri global shortcut fires
         ▼   Main app stops sidecar task + emits `overlay-hide`
```

---

## 8. Error Handling & Resilience

| Scenario | Handling |
|----------|----------|
| Overlay window fails to create | Log warning, continue without overlay (headless mode) |
| Overlay event send fails | Silently ignore (overlay may not be loaded yet) |
| Global shortcut registration fails | Log warning, Esc Guard unavailable; task still stoppable via UI |
| Config overlay.disabled = true | Skip overlay entirely |

---

## 9. Configuration

Add to `config.toml` under `[overlay]`:

```toml
[overlay]
enabled = true
aura_color = "#007AFF"
comet_color = "#00D4FF"
hud_position = "top-center"
```

Already present in existing config — read via `Config` struct. No new config keys needed.

---

## 10. Success Criteria

| Metric | Target |
|--------|--------|
| Overlay show/hide latency | < 200 ms |
| HUD update latency (step event → overlay) | < 100 ms |
| Overlay window creation | < 1 s |
| Ctrl+Alt+Esc response (stop task) | < 500 ms |
| RAM overhead (overlay window) | < 100 MB |

---

## 11. Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| WebView2 overlay can't be transparent | Low | Overlay looks wrong | Fallback to semi-transparent background |
| Global shortcut conflicts with other apps | Low | Shortcut doesn't fire | Document limitation; use UI stop button as backup |
| Overlay steals focus from target app | Medium | Disrupts agent control | Set `focus: false`, `ignoreCursorEvents: true` |
| Multiple overlay windows spawned | Low | Resource leak | Track window state, prevent duplicate creation |

---

## 12. Out of Scope (Phase 3)

- Custom system cursor (`SetSystemCursor`, .cur resource) — deferred
- WH_KEYBOARD_LL hook for real-vs-injected Esc detection — deferred (using Ctrl+Alt+Esc fallback)
- Multi-monitor overlay sync — deferred
- Overlay persistence across app restarts — not needed