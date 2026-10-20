# KeyStar: Development Roadmap & Checklist (TODO)

## 0. Development Philosophy

`KeyStar` revives the core essence of the legendary "CraftLaunch 2" (single-line, instant response, keyboard-driven, registered shortcuts execution) in modern **Rust (Edition 2024)**.

---

## 📊 Phase Matrix Table

| Phase                       | Core Focus & Deliverables                             | Key Tech Stack               |    Status     |
| :-------------------------- | :---------------------------------------------------- | :--------------------------- | :-----------: |
| **Phase 0: Environment**    | Agent rules, skills, CI & formatting baselines        | Rust 2024, `.agents`         | **Completed** |
| **Phase 1: Core Logic**     | Math engine, shortcut indexer, completion engine      | `evalexpr`, `std::fs`        | **Completed** |
| **Phase 2: GUI Prototype**  | Single-line input frame, candidate dropdown rendering | `eframe`, `egui`             | **Completed** |
| **Phase 3: OS Integration** | Global hotkeys, clipboard copy, system tray           | `global-hotkey`, `tray-icon` | **Completed** |
| **Phase 4: Settings GUI**   | Settings dialog, config persistence (`config.toml`)   | `egui`, `serde`, `toml`      |    Pending    |
| **Phase 5: Optimization**   | Single-binary LTO optimization, Scoop package         | Cargo Release, Scoop         |    Pending    |

---

## 📝 Task Checklist

### Phase 0: Project Setup & Baseline (Completed)
- [x] Establish Git policies and pre-commit checks (`RULES.md`, `.agents/rules/git_policy.md`)
- [x] Markdown table vertical alignment skill (`.agents/skills/markdown-table/`)
- [x] Rust verification workflow (`.agents/skills/rust-verification/`)
- [x] Obsidian Vault fleeting note persistence skill (`.agents/skills/vault-memo/`)
- [x] MIT License (`LICENSE`)
- [x] Bilingual README (`README.md` & `README_JA.md`)
- [x] Core dependencies (`eframe`, `evalexpr`, `open`) verified

### Phase 1: Core Logic Implementation (Completed)
- [x] **Math Evaluation Engine (`src/calc.rs`)**:
  - [x] `is_evaluable(expr: &str) -> bool` expression validation
  - [x] Basic arithmetic, parentheses, and real division (`10 / 4 = 2.5`)
  - [x] Comma formatting (`1280 * 1.1` $\to$ `= 1,408`)
  - [x] Unit & integration tests for zero division and edge cases
- [x] **Shortcut Indexer (`src/indexer.rs`)**:
  - [x] `%USERPROFILE%\commands` and `./shortcuts` path resolution
  - [x] Supported extensions (`.lnk`, `.url`, `.bat`, `.cmd`, `.exe`, `.ps1`)
  - [x] In-memory file cache with deduplication
  - [x] Directory timestamp change detection (`needs_rescan`)
- [x] **Incremental Completion Engine (`src/search.rs`)**:
  - [x] Prefix match and substring match scoring
  - [x] Case-insensitive filtering
  - [x] Calculator result prioritized to index 0

### Phase 2: GUI Window & Keyboard Handling (Completed)
- [x] **Minimal 1-Line Input Window (`src/ui.rs`)**:
  - [x] Borderless frameless window with `egui`
  - [x] Automatic focus on single-line TextEdit
  - [x] Dynamic candidate dropdown (shows 5–10 items only when query exists)
  - [x] Visual highlighting for calculator evaluation rows (`= 1,408`)
- [x] **Keyboard Interaction Handling**:
  - [x] `Enter`: Launch shortcut or copy math result
  - [x] `Esc`: Dismiss window immediately
  - [x] `Tab` / `Shift+Tab` & `Ctrl+n` / `Ctrl+p` & `Down` / `Up`: Candidate navigation

### Phase 3: OS Integration & Tray Resident (Completed)
- [x] Global hotkey `Alt + Space` via `global-hotkey` (`src/hotkey.rs`)
- [x] System tray resident via `tray-icon` (`src/tray.rs`)
- [x] Clipboard copy integration via `arboard` (`src/clipboard.rs`)

### Phase 4: GUI Settings Dialog (Beta Phase)
- [ ] **【Settings Window】 GUI Configuration Dialog (`src/settings.rs` / `src/ui.rs`)**:
  - [ ] **Tray Menu Wiring**: Open independent settings dialog upon selecting "Settings" from system tray menu
  - [ ] **Configuration Persistence (`config.toml`)**:
    - [ ] `serde` / `toml` loading and serialization (prefers portable `./config.toml`, fallbacks to `%APPDATA%\KeyStar\config.toml`)
    - [ ] Schema definition (hotkey bindings, monitored directories, max candidates, window dimensions, calculator formatting)
  - [ ] **Settings Dialog UI Design (egui)**:
    - [ ] Window size: 480 × 420 px (with title bar, Esc/close button support, dark theme)
    - [ ] Section 1 "General":
      - Global hotkey modification (`Alt + Space`, `Ctrl + Space`, `Win + Space`, or custom key capture)
      - Auto-start with Windows (`HKCU\...\Run` startup registry toggle)
    - [ ] Section 2 "Shortcuts & Search":
      - Monitored directory list (list view, Add/Remove buttons, folder picker dialog via `rfd`)
      - Max visible candidates slider (3–15 items, default: 8)
      - Target extension toggles (`.lnk`, `.url`, `.bat`, `.cmd`, `.exe`, `.ps1`)
    - [ ] Section 3 "Appearance & Calculator":
      - Launcher window width slider (400px–800px, default: 500px)
      - Font size selector (12px–20px, default: 15px)
      - Calculator clipboard format (Plain raw number `1408` vs Comma-separated `1,408`)
    - [ ] Bottom action buttons: "Save & Apply", "Cancel", "Reset to Defaults"

---

### Phase 5: Optimization & Distribution (Official Release Phase)
- [ ] **Release Binary Optimization**:
  - [ ] Configure `profile.release` in `Cargo.toml`: `lto = true`, `codegen-units = 1`, `panic = "abort"`, `strip = true`
  - [ ] Sub-millisecond cold start (<50ms) and minimal resident RAM (<10MB)
- [ ] **Package Distribution (Scoop / Winget / ZIP)**:
  - [ ] Standalone portable ZIP distribution
  - [ ] Custom Scoop bucket manifest configuration

---

## 🎨 GUI Settings Dialog Specification & Wireframe

```text
┌──────────────────────────────────────────────────────────────┐
│  KeyStar Settings                                         ✕  │
├──────────────────────────────────────────────────────────────┤
│ ┌─ General ────────────────────────────────────────────────┐ │
│ │ Global Hotkey:          [ Alt + Space             ▼ ]    │ │
│ │ [x] Launch automatically on Windows startup              │ │
│ └──────────────────────────────────────────────────────────┘ │
│ ┌─ Shortcuts & Search ─────────────────────────────────────┐ │
│ │ Monitored Directories:                                   │ │
│ │  • %USERPROFILE%\commands                                │ │
│ │  • ./shortcuts                                           │ │
│ │  [ + Add Folder... ]  [ Remove Selected ]                │ │
│ │ Max Candidates:   [=======●=============]  8 items       │ │
│ └──────────────────────────────────────────────────────────┘ │
│ ┌─ Appearance & Calculator ────────────────────────────────┐ │
│ │ Window Width:     [============●========]  500 px        │ │
│ │ Font Size:        [========●============]  15 pt         │ │
│ │ Calc Copy Format: (●) Raw Number (1408)  ( ) Comma-sep   │ │
│ └──────────────────────────────────────────────────────────┘ │
│                                                              │
│              [ Reset ]          [ Cancel ]    [ Save & Apply ] │
└──────────────────────────────────────────────────────────────┘
```
