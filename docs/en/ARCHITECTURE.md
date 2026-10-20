# Re:CraftLaunch: System Architecture (ARCHITECTURE)

## 1. Project Objectives & Design Philosophy

`Re:CraftLaunch` is an ultra-lightweight keyboard launcher built in **Rust (Edition 2024)**, reincarnating the core design principles of the classic Windows keyboard launcher "**CraftLaunch 2**" (single-line, instant response, keyboard-driven, registered shortcuts execution).

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
        ShellExecute["open::that_detached (Shell Association)"]
    end

    subgraph App ["Re:CraftLaunch Application"]
        Tray["System Tray Resident (tray-icon)"]
        GUI["Single-Line Minimal Window (eframe / egui)"]
        
        subgraph Core ["Phase 1 Core Engines"]
            Indexer["Shortcut Indexer (src/indexer.rs)"]
            Calc["Arithmetic Evaluator (src/calc.rs)"]
            Search["Incremental Completion (src/search.rs)"]
        end
    end

    HotKey -->|Toggle Signal| GUI
    Tray -->|Exit / Open Folder| App
    FS -->|Startup / Timestamp Detection| Indexer
    
    GUI -->|Input Query| Search
    GUI -->|Input Query| Calc
    Indexer -->|Cached Items| Search
    Calc -->|Math Result| Search
    
    Search -->|Candidate List (Top = Math)| GUI
    GUI -->|Enter (Shortcut)| ShellExecute
    GUI -->|Enter (Math Result)| Clipboard
```

---

## 3. Module Responsibilities

| Module Name  | Responsibilities & Roles                                                     | Key Tech               |
| :----------- | :--------------------------------------------------------------------------- | :--------------------- |
| `calc`       | Parses expressions, evaluates math, handles decimal division, comma formats   | `evalexpr`             |
| `indexer`    | Scans `%USERPROFILE%\commands` and `./shortcuts`, checks timestamps           | `std::fs`, `PathBuf`   |
| `search`     | Scores prefix/substring matches against shortcut names, prioritizes math     | In-memory scoring      |
| `ui` (Phase 2)| Renders single-line text input, candidate dropdown, handles keystrokes       | `eframe`, `egui`       |
| `main`       | Application lifecycle, hotkey management, system tray resident control       | `global-hotkey`, `open`|

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
- Scores prefix matches higher than substring matches.
- Case-insensitive filtering.
