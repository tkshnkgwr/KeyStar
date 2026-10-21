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
   - `Enter` launches selected item via `open::that_detached(path)` or `ShellExecuteW` and immediately hides launcher window.

### 2.2 Instant Arithmetic Calculator (Feature 2)
1. **Real-Time Evaluation**:
   - Evaluates mathematical expressions using `evalexpr` whenever arithmetic operators (`+ - * / % ( )`) and numbers are detected.
   - Evaluates division as real numbers (`10 / 4 = 2.5`).
   - Displays result at top candidate position as `= <Result>`.
2. **Clipboard Copy**:
   - Pressing `Enter` on a math candidate copies numerical value to OS clipboard and hides window.

### 2.3 Built-in Slash Commands (Feature 3)
1. **Dedicated Slash Matching Mode**:
   - Typing `/` triggers specialized built-in command search mode instead of shortcut search.
   - `/` displays all available commands; typing characters narrows down the list incrementally.
2. **Supported Commands**:
   - `/task`: Launches Windows Task Manager (`taskmgr.exe`).
   - `/config`: Opens shortcut configuration directory in Explorer.
   - `/cmd`: Opens `%USERPROFILE%\commands` in Explorer.
   - `/reload`: Immediately rescans monitored directories and updates cache.
   - `/restart`: Cleanly restarts KeyStar application.
   - `/date` / `/today`: Copies current date (`YYYY-MM-DD`) to clipboard.
   - `/time`: Copies current time (`HH:MM:SS`) to clipboard.
   - `/ip`: Copies local IP address to clipboard.
   - `/lock`: Locks Windows workstation screen (`Win + L`).
   - `/sleep`: Puts Windows into sleep mode.
   - `/shutdown`: Powers off Windows system.
   - `/help`: Displays all slash commands.
   - `/exit`: Completely terminates KeyStar.

### 2.4 Command Launch Arguments (Feature 4)
1. **Separation of Command and Arguments**:
   - Splits query into command word and arguments (`split_command_and_args`).
   - Preserves candidate dropdown suggestions while typing trailing arguments.
2. **Execution with Arguments**:
   - Passes parameters to `.lnk`, `.bat`, and `.exe` via `ShellExecuteW`.
   - Executes `.ps1` (PowerShell scripts) directly with `-ExecutionPolicy Bypass -File`.

### 2.5 Process Resident & Duplicate Prevention (Feature 5)
1. **Single Instance Guard**:
   - Acquired via Windows Named Mutex `Local\KeyStar_SingleInstance_Mutex`.
   - Secondary processes terminate immediately if another instance is already running.
2. **DOS Window Suppression**:
   - Applies `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` in release mode to eliminate console window on startup.

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
| `Esc`               | Dismiss launcher window immediately (clean return)     |
| `Tab` / `Shift+Tab` | Cycle through candidate list                           |
| `Ctrl+n` / `Ctrl+p` | Vim-friendly candidate navigation                      |
| `Down` / `Up`       | Arrow key candidate navigation                         |

---

## 4. Architectural Overview

```mermaid
flowchart TD
    subgraph OS ["Windows OS"]
        HotKey["Global Hotkey (Alt+Space)"]
        FS["Shortcut Directory (%USERPROFILE%\\commands)"]
        Clip["OS Clipboard"]
        Shell["ShellExecuteW (Launch with Args)"]
        TaskMgr["taskmgr.exe (Task Manager)"]
    end

    subgraph App ["KeyStar"]
        Tray["System Tray Resident (tray-icon)"]
        Window["1-Line egui/eframe Window"]
        Guard["Single Instance Guard (Named Mutex)"]
        
        subgraph Logic ["Internal Logic"]
            Index["Shortcut Indexer (src/indexer.rs)"]
            Calc["Math Evaluator (src/calc.rs)"]
            Cmd["Built-in Commands (src/command.rs)"]
            Filter["Incremental Search (src/search.rs)"]
        end
    end

    Guard -->|Duplicate Check| App
    HotKey -->|Wakeup Signal| Window
    Tray -->|Menu / Exit| App
    FS -->|Startup / Timestamp Poll| Index
    
    Window -->|Query Text| Filter
    Window -->|Query Text| Calc
    Window -->|Starts with /| Cmd
    Index -->|Cached Items| Filter
    Cmd -->|Built-in Candidates| Filter
    
    Filter -->|Candidate List| Window
    Calc -->|Math Result| Window
    
    Window -->|Enter (App/Args)| Shell
    Window -->|Enter (Calc/Date/IP)| Clip
    Window -->|Enter (/task)| TaskMgr
```
