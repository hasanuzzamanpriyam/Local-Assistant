# SDD ledger — plan: C:\Users\CT\Desktop\Assistant\docs\superpowers\plans\2026-11-11-assistant-full-clone-implementation.md

## Pre-flight Scan

| Check | Tasks | Interface/File | Finding | Ruling |
|-------|-------|----------------|---------|--------|
| Missing main.rs | 1.1 vs 1.2 | src-tauri/src/main.rs | Task 1.1 creates frontend main.ts but not Tauri main.rs; Task 1.2 modifies main.rs:1-50 | **Ruling**: Task 1.1 must create minimal Tauri main.rs entry point. Add to 1.1 Step 14. |
| Commands dir structure | 1.1 vs 1.3 | src-tauri/src/commands/ | Task 1.1 doesn't create commands/ dir; Task 1.3 creates mod.rs as new | **Ruling**: Task 1.1 should create empty commands/mod.rs with register() stub. |
| Multiple main.rs mods | 1.2, 1.3, 3.7 | src-tauri/src/main.rs | Three tasks modify main.rs | **Ruling**: Each task adds its own state/setup; they're additive. Ensure order: 1.2 (config), 1.3 (model), 3.7 (cleanup). |
| Multiple App.svelte mods | 1.1, 1.6, 1.7 | src/App.svelte | Three tasks modify App.svelte | **Ruling**: Additive - 1.1 creates base, 1.6 adds Chat, 1.7 adds Settings + tabs. Compatible. |
| Multiple runner.py mods | 2.1, 2.2, 2.6 | sidecar/runner.py | Three tasks modify runner.py | **Ruling**: Additive - 2.1 creates skeleton, 2.2 adds screen, 2.6 adds agent loop. Compatible. |
| Model commands split | 1.3 vs 1.7 | src-tauri/src/commands/model.rs | 1.3 creates model.rs with chat/list/health; 1.7 adds scan_local/set_default | **Ruling**: 1.7 should add to existing model.rs, not replace. Update 1.7 to "Modify". |
| Sidecar protocol sync | 2.1 | sidecar/protocol.py ↔ protocol.rs | Must match exactly | **Ruling**: Keep as single source of truth in plan; both files generated from spec §4. |
| Overlay HTML entry | 3.1 | src-tauri/resources/overlay.html | Needs to exist before overlay components | **Ruling**: 3.1 creates overlay.html and overlay Svelte entry. |
| Cursor file | 3.5 | src-tauri/resources/cursors/agent.cur | Binary file needs to exist | **Ruling**: 3.5 Step 1 creates .cur file via ImageMagick or placeholder. |
| Config path consistency | All | config.toml location | Plan says root config.toml but Tauri expects app data dir | **Ruling**: config.rs handles this correctly - loads from app data dir, creates default from embedded template. |
| llama.cpp binary | 1.3 | resources/llama.cpp/llama-server.exe | Plan assumes binary exists | **Ruling**: Need to add download step in 1.3 or document as prerequisite. |

**Ruling Summary**: All conflicts are resolvable with minor task adjustments. No structural contradictions. Proceed with Task 1.1 with the added main.rs creation.