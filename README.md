<div align="center">

# 🚀 KeyStar

**Ultra-Lightweight, Single-Line Keyboard Launcher in Rust, Reincarnating the Spirit of the Classic "CraftLaunch 2"**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust 2024](https://img.shields.io/badge/Rust-2024%20Edition-orange.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-v0.1.0--beta.2-brightgreen.svg)](Cargo.toml)
[![Platform](https://img.shields.io/badge/Platform-Windows-blue.svg)](https://github.com/tkshnkgwr/KeyStar)

[English](./README.md) | [日本語](./README_JA.md)

</div>

---

```text
+-------------------------------------------------------+
| > [ Type here (Minimal single-line input box)       ] |
+-------------------------------------------------------+
| = 1,408 (Instant arithmetic evaluation result) |
| :--------------------------------------------- |
| * Chrome.lnk                                   |
+-------------------------------------------------------+
```

## Background & Philosophy

`KeyStar` is rooted in the minimalist design philosophy and keyboard-first ergonomics of the legendary Windows command-line launcher "**CraftLaunch 2**".

Unlike modern, resource-heavy desktop launchers that index entire drives and pollute search results, the core principle of CraftLaunch 2 remains timeless: **"Trigger only the shortcuts you explicitly placed in your designated folder, with minimal keystrokes."**

Having relied on CraftLaunch for many years, the author wanted to create an everlasting, zero-dependency, single-binary successor tailored for modern Windows 11. By using modern Rust and a cleanroom approach (reconstructing solely the UI layout and tactile keyboard ergonomics), `KeyStar` aims to be an indestructible daily utility.

* **Development Approach**:  
  Implemented in Rust in collaborative pair programming with AI (Gemini).
* **Current Status**:  
  Currently at `v0.1.0-beta.2`. Phase 0 (Setup & Specs), Phase 1 (Core Logic), Phase 2 (egui minimal 1-line GUI), Phase 3 (OS integration: Alt+Space global hotkey, system tray daemon, and arboard clipboard copy), and the complete 15-document bilingual specification suite are fully delivered and verified. All stub implementations have been audited and confirmed zero. Subsequent packaging and optimizations (Phase 4) are conducted under the **Beta** phase.

---

## 🙏 Special Thanks & Respect

The `KeyStar` project is developed with the utmost respect and heartfelt gratitude to **craftware**, the creator of the timeless, legendary Windows keyboard launcher "**CraftLaunch** (CraftLaunch 2 / 3)".

- **Original Software**: CraftLaunch (clnch)
- **Author**: **craftware**
- **Official Site**: [craftware (Google Sites)](https://sites.google.com/site/craftware/clnch)
- **Repository**: [GitHub (crftwr/clnch)](https://github.com/crftwr/clnch)

The unobtrusive single-line input frame popping up at the top of the screen, lightning-fast prefix-based incremental completion, and instant math evaluation copying—CraftLaunch pioneered the true joy and speed of keyboard-driven launcher workflows for countless power users and developers.

We express our deepest appreciation to craftware for creating such an exceptional masterpiece and enriching the keyboard-first desktop experience for decades.

---

## 🎯 Concept & Design Highlights

- **Modern Remake of the Classic "CraftLaunch 2"**:
  - Rather than recreating the bloated, Python-embedded path of "CraftLaunch 3", `KeyStar` revives the widely celebrated essence of **CraftLaunch 2 (single-line, instant response, keyboard-driven, registered shortcuts execution)** in modern Rust.
  - Adheres to the ultimate minimalism: **"Trigger only the shortcuts you placed in your designated folder, with minimal keystrokes."**
- **Two Sharpened Core Features**:
  1. **Incremental Shortcut Completion & Launch**:
     - Simply drop `.lnk`, `.bat`, `.cmd`, or `.url` files into `%USERPROFILE%\commands` (or the local `shortcuts/` folder).
     - Filter candidates in real-time with in-memory caching and prefix/substring matching; press `Enter` to launch immediately.
  2. **Instant Inline Calculator**:
     - Whenever mathematical expressions containing numbers and operators (`+ - * / % ( )`) are entered, the engine evaluates them in real-time (`evalexpr`).
     - Pressing `Enter` automatically copies the calculated value to the OS clipboard and hides the window.
- **Cleanroom Implementation**:
  - Built without inspecting internal legacy Python structures. Only the **minimalist UI layout** and **tactile keyboard ergonomics** (hotkey, completion dropdown, Enter/Esc) are preserved and reconstructed cleanly in Rust.

---

## 🛠️ Tech Stack

| Component           | Technology / Crate      | Rationales & Selection Criteria                              |
| :------------------ | :---------------------- | :----------------------------------------------------------- |
| **Language**        | **Rust (Edition 2024)** | High performance, memory safety, zero garbage collection     |
| **GUI Framework**   | **`eframe` / `egui`**   | Minimal overhead, builds into a single self-contained binary |
| **Math Evaluation** | **`evalexpr`**          | Fast and secure arithmetic expression evaluator              |
| **Shell Launcher**  | **`open`**              | Reliably launches Windows shortcuts and URL associations     |
| **Global Hotkey**   | **`global-hotkey`**     | Detects global invocation keys while running in background   |
| **Tray Resident**   | **`tray-icon`**         | Sits quietly in the system tray without polluting taskbar    |

---

## ⌨️ Keyboard Operations

| Key Combination     | Action Description                                         |
| :------------------ | :--------------------------------------------------------- |
| `Alt + Space`       | Toggle launcher visibility (Global Hotkey)                 |
| `Enter`             | Execute selected shortcut or copy math result to clipboard |
| `Esc`               | Dismiss launcher window immediately                        |
| `Tab` / `Shift+Tab` | Select next / previous candidate in dropdown list          |
| `Ctrl+n` / `Ctrl+p` | Vim-friendly next / previous candidate selection           |
| `Down` / `Up`       | Arrow key navigation for candidate list                    |

---

## 🚀 Build & Run

### Prerequisites
- Rust 1.85+ (Edition 2024 support)
- Cargo

### Local Build & Execution
```powershell
# Clone the repository
git clone https://github.com/tkshnkgwr/KeyStar.git KeyStar
cd KeyStar

# Run development build
cargo run

# Build optimized release binary
cargo build --release
```

---

## 📚 Documentation Suite

Comprehensive specifications, architectural guidelines, manual testing checklists, and footprints are documented under `docs/`:

| Document                                                   | Description & Scope                                            |
| :--------------------------------------------------------- | :------------------------------------------------------------- |
| [**ARCHITECTURE.md**](./docs/en/ARCHITECTURE.md)           | System components, data flows, and module responsibilities     |
| [**SPECIFICATION.md**](./docs/en/SPECIFICATION.md)         | Core requirements, search ranking, and math engine specs       |
| [**GUI_DESIGN.md**](./docs/en/GUI_DESIGN.md)               | CraftLaunch 2 aesthetics, dimensions, palette, and keybindings |
| [**DEVELOPMENT.md**](./docs/en/DEVELOPMENT.md)             | Prerequisites, build instructions, and verification commands   |
| [**USER_GUIDE.md**](./docs/en/USER_GUIDE.md)               | Quick start, shortcut folder setup, and complete cheat sheet   |
| [**INSTRUCTIONS.md**](./docs/en/INSTRUCTIONS.md)           | Developer and AI agent directives linked with RULES.md         |
| [**FOOTPRINTS.md**](./docs/en/FOOTPRINTS.md)               | Memory footprint, CPU usage, and cold start benchmarks         |
| [**MANUAL_TEST_GUIDE.md**](./docs/en/MANUAL_TEST_GUIDE.md) | Manual acceptance testing checklist across 8 core sections     |
| [**TESTING.md**](./docs/en/TESTING.md)                     | 3-layer testing architecture (unit, integration, doc tests)    |
| [**TEST_REPORT.md**](./docs/en/TEST_REPORT.md)             | Official test execution report covering 64 passing tests       |
| [**RELEASE.md**](./docs/en/RELEASE.md)                     | Release checklist, optimization build, and packaging steps     |
| [**CONTRIBUTING.md**](./docs/en/CONTRIBUTING.md)           | Contribution policy, clean-room rules, and PR checklist        |
| [**SECURITY.md**](./docs/en/SECURITY.md)                   | Privilege boundaries, process isolation, and input safety      |
| [**TODO.md**](./docs/en/TODO.md)                           | Development roadmap and implementation milestones              |
| [**CHANGELOG.md**](./docs/en/CHANGELOG.md)                 | Version release history (Keep a Changelog standard)            |

---

## 📂 Project Structure

```text
KeyStar/
├── .agents/
│   ├── rules/                  # Git, Rust testing, and Rustdoc specifications
│   └── skills/                 # Verification, commit, table format, and memo skills
├── benches/                    # Benchmarking design workspace (Criterion)
│   └── README.md
├── docs/                       # Comprehensive bilingual documentation suite (15 each)
│   ├── en/                     # English documentation (15 docs)
│   │   ├── ARCHITECTURE.md     # System architecture and data flow
│   │   ├── CHANGELOG.md        # Release history (Keep a Changelog)
│   │   ├── CONTRIBUTING.md     # Contribution guidelines & PR checklist
│   │   ├── DEVELOPMENT.md      # Development setup & build commands
│   │   ├── FOOTPRINTS.md       # Resource consumption metrics & benchmarks
│   │   ├── GUI_DESIGN.md       # GUI design specs & UI/UX layout
│   │   ├── INSTRUCTIONS.md     # Developer and AI agent directives
│   │   ├── MANUAL_TEST_GUIDE.md# Manual acceptance testing checklist
│   │   ├── RELEASE.md          # Release checklist & packaging steps
│   │   ├── SECURITY.md         # Privilege boundaries & security policy
│   │   ├── SPECIFICATION.md    # Detailed functional specification
│   │   ├── TESTING.md          # 3-layer test architecture & guidelines
│   │   ├── TEST_REPORT.md      # Official test execution report
│   │   ├── TODO.md             # Development roadmap & task matrix
│   │   └── USER_GUIDE.md       # Complete user operation manual
│   └── ja/                     # Japanese documentation (15 docs)
│       ├── ARCHITECTURE.md
│       ├── CHANGELOG.md
│       ├── CONTRIBUTING.md
│       ├── DEVELOPMENT.md
│       ├── FOOTPRINTS.md
│       ├── GUI_DESIGN.md
│       ├── INSTRUCTIONS.md
│       ├── MANUAL_TEST_GUIDE.md
│       ├── RELEASE.md
│       ├── SECURITY.md
│       ├── SPECIFICATION.md
│       ├── TESTING.md
│       ├── TEST_REPORT.md
│       ├── TODO.md
│       └── USER_GUIDE.md
├── examples/                   # Headless standalone CLI examples
│   └── README.md
├── scripts/                    # Automation scripts
│   └── align_table.py          # East Asian Width markdown table aligner
├── src/                        # Rust source code (all < 1,000 lines)
│   ├── main.rs                 # Main entrypoint (resident daemon & window subsystem)
│   ├── lib.rs                  # Library root & public module exports
│   ├── calc.rs                 # Real-time arithmetic math engine (evalexpr)
│   ├── indexer.rs              # Shortcut directory scanner & cache
│   ├── search.rs               # Incremental search & dynamic scoring
│   ├── ui.rs                   # Minimal 1-line window & state machine (egui)
│   ├── hotkey.rs               # Global hotkey daemon for Alt+Space (global-hotkey)
│   ├── tray.rs                 # System tray resident & context menu (tray-icon)
│   └── clipboard.rs            # OS clipboard provider (arboard)
├── tests/                      # Public API integration test suite (20 tests)
│   ├── calc_integration.rs    # Arithmetic & edge-case integration tests
│   ├── indexer_integration.rs # Real filesystem scan & cache tests
│   ├── search_integration.rs  # Case-insensitivity & ranking tests
│   ├── ui_integration.rs      # Completion & window state transition tests
│   └── os_integration.rs      # Hotkey, tray, and clipboard integration tests
├── Cargo.lock                  # Dependency lockfile
├── Cargo.toml                  # Rust crate manifest
├── CHANGELOG.md                # Changelog root
├── GEMINI.md                   # AI assistant master directives
├── LICENSE                     # MIT License
├── README.md                   # English project documentation
├── README_JA.md                # Japanese project documentation
├── RULES.md                    # Workspace common absolute rules
└── rustfmt.toml                # Formatter configuration
```

---

## 📄 License

This project is open-sourced under the **[MIT License](LICENSE)**.

Copyright (c) 2026 tkshnkgwr  
KeyStar is an independent tribute project inspired by the design and UI concepts of CraftLaunch by craftware.
