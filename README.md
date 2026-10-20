<div align="center">

# 🚀 Re:CraftLaunch

**Ultra-Lightweight, Single-Line Keyboard Launcher in Rust, Reincarnating the Spirit of the Classic "CraftLaunch 2"**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust 2024](https://img.shields.io/badge/Rust-2024%20Edition-orange.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-v0.1.0--alpha.2-brightgreen.svg)](Cargo.toml)
[![Platform](https://img.shields.io/badge/Platform-Windows-blue.svg)](https://github.com/tkshnkgwr/KeyStar)

[English](./README.md) | [日本語](./README_JA.md)

</div>

---

```text
+-------------------------------------------------------+
| > [ Type here (Minimal single-line input box)       ] |
+-------------------------------------------------------+
|  = 1,408 (Instant arithmetic evaluation result)       |
|  * Obsidian.lnk                                       |
|  * Chrome.lnk                                         |
+-------------------------------------------------------+
```

## Background & Philosophy

`Re:CraftLaunch` is rooted in the minimalist design philosophy and keyboard-first ergonomics of the legendary Windows command-line launcher "**CraftLaunch 2**".

Unlike modern, resource-heavy desktop launchers that index entire drives and pollute search results, the core principle of CraftLaunch 2 remains timeless: **"Trigger only the shortcuts you explicitly placed in your designated folder, with minimal keystrokes."**

Having relied on CraftLaunch for many years, the author wanted to create an everlasting, zero-dependency, single-binary successor tailored for modern Windows 11. By using modern Rust and a cleanroom approach (reconstructing solely the UI layout and tactile keyboard ergonomics), `Re:CraftLaunch` aims to be an indestructible daily utility.

* **Development Approach**:  
  Implemented in Rust in collaborative pair programming with AI (Gemini).
* **Current Status**:  
  Currently at `v0.1.0-alpha.2`. Phase 1 (Core Logic: calculator, indexer, and search completion) is complete, ready for Phase 2 (egui minimal 1-line GUI).

---

## 🙏 Special Thanks & Respect

The `Re:CraftLaunch` project is developed with the utmost respect and heartfelt gratitude to **craftware**, the creator of the timeless, legendary Windows keyboard launcher "**CraftLaunch** (CraftLaunch 2 / 3)".

- **Original Software**: CraftLaunch (clnch)
- **Author**: **craftware**
- **Official Site**: [craftware (Google Sites)](https://sites.google.com/site/craftware/clnch)
- **Repository**: [GitHub (crftwr/clnch)](https://github.com/crftwr/clnch)

The unobtrusive single-line input frame popping up at the top of the screen, lightning-fast prefix-based incremental completion, and instant math evaluation copying—CraftLaunch pioneered the true joy and speed of keyboard-driven launcher workflows for countless power users and developers.

We express our deepest appreciation to craftware for creating such an exceptional masterpiece and enriching the keyboard-first desktop experience for decades.

---

## 🎯 Concept & Design Highlights

- **Modern Remake of the Classic "CraftLaunch 2"**:
  - Rather than recreating the bloated, Python-embedded path of "CraftLaunch 3", `Re:CraftLaunch` revives the widely celebrated essence of **CraftLaunch 2 (single-line, instant response, keyboard-driven, registered shortcuts execution)** in modern Rust.
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

| Key Combination      | Action Description                                              |
| :------------------- | :-------------------------------------------------------------- |
| `Alt + Space`        | Toggle launcher visibility (Global Hotkey)                      |
| `Enter`              | Execute selected shortcut or copy math result to clipboard      |
| `Esc`                | Dismiss launcher window immediately                             |
| `Tab` / `Shift+Tab`  | Select next / previous candidate in dropdown list               |
| `Ctrl+n` / `Ctrl+p`  | Vim-friendly next / previous candidate selection                |
| `Down` / `Up`        | Arrow key navigation for candidate list                         |

---

## 🚀 Build & Run

### Prerequisites
- Rust 1.85+ (Edition 2024 support)
- Cargo

### Local Build & Execution
```powershell
# Clone the repository
git clone https://github.com/tkshnkgwr/KeyStar.git Re-CraftLaunch
cd Re-CraftLaunch

# Run development build
cargo run

# Build optimized release binary
cargo build --release
```

---

## 📂 Project Structure

```text
Re-CraftLaunch/
├── .agents/
│   ├── rules/              # Git, Rust testing, and Rustdoc specifications
│   └── skills/             # Verification, commit, table format, and memo skills
├── docs/                   # Specifications and development roadmap
│   ├── en/                 # English documentation
│   └── ja/                 # Japanese documentation
│       ├── SPECIFICATION.md# Detailed feature specification
│       └── TODO.md         # Development roadmap & task checklist
├── src/                    # Rust source code
│   └── main.rs
├── tests/                  # Integration & scenario tests
├── Cargo.toml              # Rust crate manifest
├── CHANGELOG.md            # Changelog (English)
├── GEMINI.md               # AI assistant master directives
├── LICENSE                 # MIT License
├── README.md               # English documentation
├── README_JA.md            # Japanese documentation
├── RULES.md                # Workspace common absolute rules
└── rustfmt.toml            # Formatter configuration
```

---

## 📄 License

This project is open-sourced under the **[MIT License](LICENSE)**.

Copyright (c) 2026 tkshnkgwr  
Re-CraftLaunch is an independent tribute project inspired by the design and UI concepts of CraftLaunch by craftware.
