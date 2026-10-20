# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0-beta.1] - 2026-10-20

### Changed
- **Transition to Beta Phase & Directory Structure Tree Modernization**:
  - Upgraded project version from `v0.1.0-alpha.5` to `v0.1.0-beta.1`.
  - Modernized repository folder structure tree in `README.md` and `README_JA.md` to fully reflect all source modules, tests, bilingual documentation, helper scripts, benchmarks, and examples.
  - Completed comprehensive codebase verification auditing all modules (`calc`, `indexer`, `search`, `clipboard`, `hotkey`, `tray`, `ui`, `main`), confirming zero stub/mock implementations and 100% real production logic across all 64 tests (27 unit, 20 integration, 17 doc-tests).

## [0.1.0-alpha.5] - 2026-10-20

### Added
- **Alpha Milestone Completion & Comprehensive Documentation Suite**:
  - **Symmetric 15-Document Bilingual Architecture**:
    - Complete synchronization across `docs/ja/` and `docs/en/` (15 docs in each language).
    - `FOOTPRINTS.md`: Detailed resource benchmarks (<50ms cold start, 0.0% idle CPU, <15MB resident RAM, <1ms hotkey wakeup) and complexity analysis.
    - `INSTRUCTIONS.md`: Strict developer and AI agent directives, RULES.md enforcement, clean-room protocol, and verification steps.
    - `MANUAL_TEST_GUIDE.md`: Comprehensive 8-section manual acceptance testing checklist for Windows workstations.
    - `RELEASE.md`: Pre-release verification checklist, optimized LTO build, tagging, and distribution ZIP workflow.
    - `GUI_DESIGN.md`, `CONTRIBUTING.md`, `SECURITY.md`, `TESTING.md`, and `TEST_REPORT.md`: Fully mirrored in English.
  - **Benchmarking & Example Workspaces**:
    - `benches/README.md`: Architecture guide for upcoming `criterion` benchmark suites (calc, search, indexer).
    - `examples/README.md`: Standalone headless CLI examples for instant testing (`calc_cli`, `search_cli`, `icon_gen`).
  - **Table Alignment Automation**:
    - `scripts/align_table.py`: Python East Asian Width vertical alignment tool applied to all markdown tables repository-wide.
  - **Milestone Transition**:
    - Formal conclusion of the Alpha development phase. All subsequent functional development (Phase 4: Release LTO optimizations & Scoop packaging) transitions to the **Beta** phase.

## [0.1.0-alpha.4] - 2026-10-20

### Added
- **Phase 3 OS Integration & Background Resident Daemon**:
  - **Global Hotkey Daemon (`src/hotkey.rs`)**:
    - System-wide `Alt + Space` hotkey registration and monitoring via `global-hotkey`.
    - Key-release filtering (`should_trigger_hotkey`) preventing duplicate toggling on `HotKeyState::Released`.
    - `install_hotkey_wakeup_hook`: Wakes up the sleeping `egui` event loop (`request_repaint()`) from background with zero input latency while maintaining 0% CPU usage.
  - **System Tray Integration & Embedded Icon (`src/tray.rs`)**:
    - Notification area resident icon with left-click toggle via `tray-icon`.
    - Context menu with 3 options: "Settings" (open shortcuts/settings folder & activate), "Open Commands Folder" (`%USERPROFILE%\commands`), and "Exit" (clean application shutdown).
    - `generate_default_icon_rgba`: Procedural in-memory 32x32 RGBA icon (dark charcoal `#1e1e1e` with teal green `#4ec9b0` `>_` prompt glyph), eliminating external `.ico` assets.
  - **OS Synchronous Clipboard Provider (`src/clipboard.rs`)**:
    - Immediate and reliable OS clipboard writing via `arboard` upon math calculation confirmation (`Enter`).
    - Decoupled `ClipboardProvider` trait with `SystemClipboard` and headless `MemoryClipboard` for deterministic testing.
  - **Resident Window Lifecycle & Focus Forcing (`src/ui.rs` & `src/main.rs`)**:
    - `LauncherState::toggle_visibility`: Seamless toggle between foreground active and hidden background states.
    - Automatic focus restoration (`take_focus_request`) and `ViewportCommand::Visible(true)` / `Focus` on hotkey activation.
    - Windows subsystem configuration (`windows_subsystem = "windows"`) preventing console window flash.
  - **OS Integration Test Suite (`tests/os_integration.rs`)**:
    - 4 end-to-end integration tests covering global hotkey filtering, tray menu commands, clipboard copy pipeline, and visibility/focus lifecycle.
    - Total test suite expanded to 64 tests (27 unit, 20 integration, 17 doc tests, 100% PASS).

## [0.1.0-alpha.3] - 2026-10-20

### Added
- **Phase 2 Minimal 1-Line GUI & Keyboard Navigation**:
  - **GUI Window & State Architecture (`src/ui.rs`)**:
    - `LauncherApp`: Minimal frameless, borderless single-line input bar (`eframe` / `egui`).
    - `LauncherState`: Decoupled state machine managing input buffer, candidate list, selected index, and action queue.
    - `KeyCommand` & `LauncherAction`: Type-safe command and execution intent decoupling (`LaunchShortcut`, `CopyCalculation`, `Close`, `Hide`).
  - **Keyboard Navigation System**:
    - `Tab` / `Shift+Tab`: Forward/backward candidate selection and incremental text completion while preserving argument strings.
    - `Down` / `Up` & `Ctrl+n` / `Ctrl+p`: Bidirectional cyclic candidate navigation.
    - `Enter`: Execute selected shortcut (`open::that_detached`) or copy arithmetic result to clipboard.
    - `Esc`: Clear input buffer on first press, dismiss/hide window on second press (faithful CraftLaunch 2 ergonomics).
    - Single-line `TextEdit` focus locking (`lock_focus(true)`) and preemptive key consumption preventing focus escape.
  - **Dynamic Dropdown Panel**:
    - Automatically displays up to 8 matching candidates below the single-line input box.
    - Visual indicator prefix (`=`) and distinct styling for instant calculation results.
    - Dark mode color scheme inspired by CraftLaunch 2 palette.
  - **Comprehensive UI Integration Tests (`tests/ui_integration.rs`)**:
    - 4 state-machine scenario integration tests covering incremental completion, calculation execution, empty query dropdown, and frameless native window options.
    - Total test suite expanded to 45 tests (20 unit, 16 integration, 9 doc tests, 100% PASS).

## [0.1.0-alpha.2] - 2026-10-20

### Added
- **Phase 1 Core Logic Implementation**:
  - **Calculator Engine (`src/calc.rs`)**:
    - Real-time arithmetic evaluation using `evalexpr`.
    - Automatic promotion to real division (`10 / 4 = 2.5`).
    - Thousands separator comma formatting (`1,408`) and clean clipboard numeric string (`1408`).
    - `is_evaluable` guard preventing unintended evaluations on plain numbers or words.
  - **Shortcut Indexer (`src/indexer.rs`)**:
    - Automatic resolution and creation of `%USERPROFILE%\commands` and `./shortcuts`.
    - Case-insensitive scanning for `.lnk`, `.url`, `.bat`, `.cmd`, `.exe`, `.ps1`.
    - Low-overhead directory timestamp comparison (`needs_rescan`).
  - **Completion Engine (`src/search.rs`)**:
    - Dynamic scoring favoring prefix matches over substring matches.
    - Highest priority placement (index 0) for active calculator results.
  - **3-Layer Test Suite & Quality Verification (`tests/`)**:
    - 15 unit tests, 12 public API integration tests, and 7 doc tests (34 tests 100% passing).
    - Verified line count limits (max 316 lines, well below 1000 lines).
- **Comprehensive Bilingual Documentation Suite (`docs/`)**:
  - `ARCHITECTURE.md` (System design & data flow)
  - `SPECIFICATION.md` (Functional requirements)
  - `GUI_DESIGN.md` (Single-line window & candidate dropdown design)
  - `DEVELOPMENT.md` (Developer guidelines & verification commands)
  - `USER_GUIDE.md` (User manual & shortcut registration guide)
  - `TESTING.md` & `TEST_REPORT.md` (Testing architecture & official test report)
  - `CONTRIBUTING.md` & `SECURITY.md` (Contribution scope & security boundaries)

---

## [0.1.0-alpha.1] - 2026-10-20

### Added
- **Initial Project Setup & Environment Baseline (Phase 0)**:
  - **Agent & Skill Automation Framework (`.agents/`)**:
    - Strict prohibition of automatic Git commit/push policy (`git_policy.md`).
    - Pre-commit verification checklist (`rust_pre_commit.md`).
    - Test separation architecture guide (`tests/` integration vs `src/` unit tests) (`rust_testing.md`).
    - Rustdoc documentation standard in Japanese (`rustdoc.md`).
    - Safe Git commit & push workflow (`git-commit` skill).
    - Markdown table vertical alignment guide for gVim readability (`markdown-table` skill).
    - High-speed Rust quality check workflow (`rust-verification` skill).
    - Obsidian Vault fleeting note persistence workflow (`vault-memo` skill).
  - **Project Configuration & Code Style**:
    - Workspace directives (`RULES.md`) and AI master rules (`GEMINI.md`).
    - Standard Rust formatter configuration (`rustfmt.toml`: Edition 2024, max_width 100, tab_spaces 4).
    - Cross-editor formatting rules (`.editorconfig`: UTF-8, LF, space indent 4).
    - Git ignore rules (`.gitignore`).
  - **Documentation & Licensing**:
    - Bilingual documentation: English (`README.md`) and Japanese (`README_JA.md`).
    - Open source licensing under the MIT License (`LICENSE`).
    - Comprehensive technical specification (`docs/ja/SPECIFICATION.md`).
    - Development roadmap & task checklist (`docs/ja/TODO.md`, `TODO.md`).
  - **Core Dependencies & Verification**:
    - Added `eframe` (v0.37.0), `evalexpr` (v13.1.0), and `open` (v5.4.4).
    - Verified compilation, testing, clippy lints, and formatting with 100% pass rate.
