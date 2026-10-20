# KeyStar: Specification (SPECIFICATION)

## 1. Application Overview

| Item              | Details                                                                  |
| :---------------- | :----------------------------------------------------------------------- |
| **Name**          | KeyStar                                                                  |
| **Category**      | Ultra-Lightweight Single-Line Windows Keyboard Launcher                  |
| **Language**      | Rust (Edition 2024)                                                      |
| **GUI Framework** | `eframe` / `egui`                                                        |
| **Target OS**     | Windows 11 (x86_64)                                                      |
| **Design Spirit** | Revival of "CraftLaunch 2" (Single-line, blazing fast, keyboard-centric) |

---

## 2. Core Functional Requirements

### 2.1 Incremental Shortcut Completion (Feature 1)
1. **Monitored Directories**:
   - Primary: `%USERPROFILE%\commands` (automatically expanded).
   - Portable: `./shortcuts` adjacent to binary.
   - Extensions: `.lnk`, `.url`, `.bat`, `.cmd`, `.exe`, `.ps1`.
2. **In-Memory Caching**:
   - Maps file stems (name without extension) to full paths in memory.
   - Dynamic update via directory timestamp comparison (`needs_rescan`).
3. **Filtering**:
   - Case-insensitive prefix and substring matching.
   - Shows top 5–10 candidates in a clean dropdown list.
4. **Execution**:
   - `Enter` launches selected item via `open::that_detached(path)` and immediately hides launcher window.

### 2.2 Instant Arithmetic Calculator (Feature 2)
1. **Real-Time Evaluation**:
   - Evaluates mathematical expressions using `evalexpr` whenever arithmetic operators (`+ - * / % ( )`) and numbers are detected.
   - Evaluates division as real numbers (`10 / 4 = 2.5`).
   - Displays result at top candidate position as `= <Result>`.
2. **Clipboard Copy**:
   - Pressing `Enter` on a math candidate copies numerical value to OS clipboard and hides window.

---

## 3. UI / UX Design

### 3.1 Window Appearance
- **Dimensions**: Width 450–550px, single-line height ~36–42px.
- **Frameless**: Borderless, no title bar.
- **System Tray**: Hidden from taskbar, sits quietly in notification area.
- **Always on Top**: Surfaces above all application windows upon hotkey invocation.

### 3.2 Keyboard Keybindings

| Keystroke           | Behavior                                               |
| :------------------ | :----------------------------------------------------- |
| `Alt + Space`       | Toggle launcher visibility (Global Hotkey)             |
| `Enter`             | Execute selected item or copy math result to clipboard |
| `Esc`               | Dismiss launcher window immediately                    |
| `Tab` / `Shift+Tab` | Cycle through candidate list                           |
| `Ctrl+n` / `Ctrl+p` | Vim-friendly candidate navigation                      |
| `Down` / `Up`       | Arrow key candidate navigation                         |
