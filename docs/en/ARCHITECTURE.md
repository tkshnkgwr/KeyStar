# KeyStar: System Architecture (ARCHITECTURE)

## 1. Project Objectives & Design Philosophy

`KeyStar` is an ultra-lightweight keyboard launcher built in **Rust (Edition 2024)**, reincarnating the core design principles of the classic Windows keyboard launcher "**CraftLaunch 2**" (single-line, instant response, keyboard-driven, registered shortcuts execution).

### 🎯 Three Core Pillars
1. **Ultimate Minimalism (100–300 Lines of Elegance)**:
   - Avoids indexing the entire hard drive or consuming background memory. Simply executes explicitly placed shortcuts in designated folders with minimum keystrokes.
2. **Cleanroom Implementation**:
   - Strictly avoids inspecting internal legacy Python structures from CraftLaunch 3. Preserves and reconstructs solely the **minimalist UI layout** and **tactile keyboard ergonomics** (hotkey, completion dropdown, Enter/Esc) in clean Rust.
3. **Pure Keyboard-Driven Ergonomics**:
   - Zero mouse dependency. Launch, search, execute, copy arithmetic evaluation, and dismiss window entirely via keyboard.

---

## 2. Overall Architecture

```mermaid
flowchart TD
    subgraph OS ["Windows OS Integration"]
        HotKey["Global Hotkey (Alt+Space)"]
        FS["Monitored Folder (%USERPROFILE%\\commands)"]
        Clipboard["OS Clipboard"]
        ShellExecute["ShellExecuteW / open::that_detached"]
        TaskMgr["taskmgr.exe (Task Manager)"]
    end

    subgraph App ["KeyStar Application"]
        Guard["Single Instance Guard (Named Mutex)"]
        Tray["System Tray Resident (tray-icon)"]
        GUI["Single-Line Minimal Window (eframe / egui)"]
        
        subgraph Core ["Core Engines & Modules"]
            Indexer["Shortcut Indexer (src/indexer.rs)"]
            Calc["Arithmetic Evaluator (src/calc.rs)"]
            Cmd["Built-in Commands (src/command.rs)"]
            Search["Incremental Completion (src/search.rs)"]
        end
    end

    Guard -->|Duplicate Check| App
    HotKey -->|Toggle Signal| GUI
    Tray -->|Exit / Open Folder| App
    FS -->|Startup / Timestamp Detection| Indexer
    
    GUI -->|Input Query| Search
    GUI -->|Input Query| Calc
    GUI -->|Starts with /| Cmd
    Indexer -->|Cached Items| Search
    Calc -->|Math Result| Search
    Cmd -->|Built-in Candidates| Search
    
    Search -->|Candidate List| GUI
    GUI -->|Enter (Shortcut/Args)| ShellExecute
    GUI -->|Enter (Math/Date/IP)| Clipboard
    GUI -->|Enter (/task)| TaskMgr
```

---

## 3. Module Responsibilities

| Module Name       | Responsibilities & Roles                                                     | Key Tech                    |
| :---------------- | :--------------------------------------------------------------------------- | :-------------------------- |
| `calc`            | Parses expressions, evaluates math, handles decimal division, comma formats  | `evalexpr`                  |
| `indexer`         | Scans `%USERPROFILE%\commands` and `./shortcuts`, checks timestamps          | `std::fs`, `PathBuf`        |
| `search`          | Scores prefix/substring matches against shortcut names, prioritizes math/cmd | Custom scoring engine       |
| `command`         | Defines built-in slash commands, incremental matching, Win32 API controls    | `windows-sys`, Win32 API    |
| `single_instance` | Prevents duplicate processes via Windows Named Mutex                         | Win32 Named Mutex           |
| `hotkey`          | Listens to system-wide hotkeys, wakes up GUI repaint thread                  | `global-hotkey`             |
| `tray`            | Controls system tray icon, procedural RGBA generation, right-click context   | `tray-icon`                 |
| `clipboard`       | Synchronously writes evaluation results or utility text to OS clipboard      | `arboard`                   |
| `ui`              | Renders single-line text input, candidate dropdown, handles keystrokes       | `eframe`, `egui`            |
| `main`            | Application lifecycle, process entry point, Windows subsystem controls       | `eframe`, Windows subsystem |

---

## 4. Component Details

### 4.1 Calculator Engine (`src/calc.rs`)
- Evaluates mathematical expressions in real-time (`evalexpr`).
- Automatically promotes integer division to real-number division (`10 / 4` $\to$ `2.5`).
- Formats integers with comma separators (`1280 * 1.1` $\to$ `= 1,408`).
- Preserves clean raw numerical string for clipboard copying (`1408`).

### 4.2 Shortcut Indexer (`src/indexer.rs`)
- Multi-directory monitoring: `%USERPROFILE%\commands` (default) and `./shortcuts` (portable).
- Target extensions: `.lnk`, `.url`, `.bat`, `.cmd`, `.exe`, `.ps1`.
- Fast, low-overhead timestamp checking (`needs_rescan`) prevents redundant disk I/O.

### 4.3 Completion Engine (`src/search.rs`)
- Merges calculator results to the top position (index 0) with top priority score (10,000).
- Switches to dedicated slash command search mode when input begins with `/`.
- Splits queries into command stems and arguments via `split_command_and_args`.
- Scores prefix matches higher than substring matches.
- Case-insensitive filtering.

### 4.4 Built-in Slash Commands (`src/command.rs`)
- Provides 14 built-in slash commands: `/task`, `/config`, `/cmd`, `/reload`, `/restart`, `/date`, `/today`, `/time`, `/ip`, `/lock`, `/sleep`, `/shutdown`, `/help`, `/exit`.
- Directly interfaces with Windows native APIs for screen locking (`LockWorkStation`), local time (`GetLocalTime`), and network routing resolution.

### 4.5 Single Instance Guard (`src/single_instance.rs`)
- Uses Windows Named Mutex `Local\KeyStar_SingleInstance_Mutex` to ensure single active instance.
- Exits secondary processes immediately without consuming system resources.

---

## 5. Technology Choices & Constraints

1. **`eframe` / `egui` (Immediate Mode GUI)**:
   - Eliminates heavyweight browser engines (Tauri/Electron), achieving sub-millisecond cold start and minimal RAM consumption (<15MB).
   - Ideal for single-line frameless text edit and dynamic candidate dropdowns.
2. **`evalexpr`**:
   - High-performance in-binary expression evaluator without requiring external runtime engines (Python/Lua).
3. **`open` & `ShellExecuteW`**:
   - Reliable async execution of Windows file associations and command-line arguments.
