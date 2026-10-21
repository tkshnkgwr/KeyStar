# KeyStar: User Guide (USER_GUIDE)

User manual and setup guide for `KeyStar`, the ultra-lightweight keyboard launcher reviving the spirit of CraftLaunch 2.

---

## 1. Quick Start

### Step 1: Launch
- Run `KeyStar.exe`.
- It sits quietly in the **system tray** without cluttering your taskbar. Monitored folders are created automatically.

### Step 2: Invoke
- Press **`Alt + Space`** to summon the minimal single-line input frame at the top/center of your screen.

### Step 3: Exit
- Right-click the tray icon and select "Exit".

---

## 2. Registering Shortcuts (Folder Drop System)

There are no complex configuration GUIs or fragile scripting setups.  
**Just drop your shortcuts into your designated folder.**

### Monitored Locations
- **Standard Folder**: `%USERPROFILE%\commands`  
  (e.g., `C:\Users\<YourUsername>\commands`)
- **Portable Folder**: `shortcuts\` folder adjacent to the executable

### How to Register
1. Open `%USERPROFILE%\commands` in Windows Explorer.
2. Drop `.lnk`, `.url`, `.bat`, `.cmd`, `.exe`, or `.ps1` files into it.
3. They are instantly searchable without restarting the app.

> 💡 **Productivity Tip**:
> Rename shortcuts to short mnemonics like `c.lnk` for Chrome or `n.lnk` for Notepad. You can launch them with 1–2 keystrokes and `Enter`.

---

## 3. Instant Calculator

Type any arithmetic expression directly into the input box to evaluate it in real-time.

| Input Example     | Display Result | Action on `Enter`                             |
| :---------------- | :------------- | :-------------------------------------------- |
| `1280 * 1.1`      | `= 1,408`      | Copies `1408` to clipboard and hides launcher |
| `10 / 4`          | `= 2.5`        | Copies `2.5` to clipboard and hides launcher  |
| `(100 + 200) * 3` | `= 900`        | Copies `900` to clipboard and hides launcher  |
| `10 % 3`          | `= 1`          | Copies `1` to clipboard and hides launcher    |

---

## 4. Keyboard Shortcuts

| Keystroke         | Action Description                                    |
| :---------------- | :---------------------------------------------------- |
| **`Alt + Space`** | Toggle launcher visibility (Global Hotkey)            |
| **`Enter`**       | Launch selected item or copy math result to clipboard |
| **`Esc`**         | Dismiss launcher immediately without executing        |
| **`Tab`**         | Select next candidate                                 |
| **`Shift + Tab`** | Select previous candidate                             |
| **`Ctrl + n`**    | Select next candidate (Vim-style navigation)          |
| **`Ctrl + p`**    | Select previous candidate (Vim-style navigation)      |
| **`Down` / `Up`** | Arrow key candidate navigation                        |

---

## 5. Launch Arguments (Command Execution)

Type an executable or shortcut name followed by a space and parameters to pass arguments directly to the target program.

| Example Input               | Behavior                                                    |
| :-------------------------- | :---------------------------------------------------------- |
| `cmd /k ipconfig`           | Opens command prompt and executes `/k ipconfig` immediately |
| `code .`                    | Launches VS Code in current directory                       |
| `chrome https://github.com` | Opens designated URL directly in Google Chrome              |

> 💡 **Tip**:
> As you type arguments, the candidate dropdown keeps matching the first command word (e.g. `cmd` or `code`).

---

## 6. Built-in Slash Commands

Type **`/`** into the input box to access built-in system utilities, launcher controls, and productivity tools with incremental filtering.

| Command                    | Feature            | Description                                                   |
| :------------------------- | :----------------- | :------------------------------------------------------------ |
| **`/task`**                | Task Manager       | Launches Windows `taskmgr.exe` asynchronously                 |
| **`/config`**              | Settings Directory | Opens shortcut settings folder in Windows Explorer            |
| **`/cmd`**                 | Commands Directory | Opens `%USERPROFILE%\commands` in Windows Explorer            |
| **`/reload`**              | Rescan Shortcuts   | Immediately scans directories and updates in-memory index     |
| **`/restart`**             | Restart KeyStar    | Gracefully terminates and relaunches KeyStar                  |
| **`/date`** / **`/today`** | Copy Current Date  | Copies `YYYY-MM-DD` (e.g. `2026-10-21`) to OS clipboard       |
| **`/time`**                | Copy Current Time  | Copies `HH:MM:SS` (e.g. `17:30:00`) to OS clipboard           |
| **`/ip`**                  | Copy Local IP      | Resolves and copies primary local IP address to clipboard     |
| **`/lock`**                | Lock Workstation   | Instantly locks Windows workstation (equivalent to `Win + L`) |
| **`/sleep`**               | Sleep Mode         | Puts PC into sleep mode                                       |
| **`/shutdown`**            | Shutdown PC        | Powers off Windows system                                     |
| **`/help`**                | Command List       | Shows all available slash commands in candidate list          |
| **`/exit`**                | Exit KeyStar       | Completely shuts down KeyStar process                         |
