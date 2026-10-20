# Re:CraftLaunch: Development Roadmap & Checklist (TODO)

## 0. Development Philosophy

`Re:CraftLaunch` revives the core essence of the legendary "CraftLaunch 2" (single-line, instant response, keyboard-driven, registered shortcuts execution) in modern **Rust (Edition 2024)**.

---

## 📊 Phase Matrix Table

| Phase                     | Core Focus & Deliverables                            | Key Tech Stack              | Status          |
| :------------------------ | :--------------------------------------------------- | :-------------------------- | :-------------: |
| **Phase 0: Environment**  | Agent rules, skills, CI & formatting baselines       | Rust 2024, `.agents`        | **Completed**   |
| **Phase 1: Core Logic**   | Math engine, shortcut indexer, completion engine     | `evalexpr`, `std::fs`       | **Completed**   |
| **Phase 2: GUI Prototype**| Single-line input frame, candidate dropdown rendering| `eframe`, `egui`            | Pending         |
| **Phase 3: OS Integration**| Global hotkeys, clipboard copy, system tray         | `global-hotkey`, `tray-icon`| Pending         |
| **Phase 4: Optimization** | Single-binary LTO optimization, Scoop package        | Cargo Release, Scoop        | Pending         |

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

### Phase 2: GUI Window & Keyboard Handling (Pending)
- [ ] **Minimal 1-Line Input Window (`src/ui.rs`)**:
  - [ ] Borderless frameless window with `egui`
  - [ ] Automatic focus on single-line TextEdit
  - [ ] Dynamic candidate dropdown (shows 5–10 items only when query exists)
- [ ] **Keyboard Interaction Handling**:
  - [ ] `Enter`: Launch shortcut or copy math result
  - [ ] `Esc`: Dismiss window immediately
  - [ ] `Tab` / `Shift+Tab` & `Ctrl+n` / `Ctrl+p`: Candidate navigation

### Phase 3: OS Integration & Tray Resident (Pending)
- [ ] Global hotkey `Alt + Space` via `global-hotkey`
- [ ] System tray resident via `tray-icon`
- [ ] Clipboard copy integration
