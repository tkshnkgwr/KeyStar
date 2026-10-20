# Re:CraftLaunch: User Guide (USER_GUIDE)

User manual and setup guide for `Re:CraftLaunch`, the ultra-lightweight keyboard launcher reviving the spirit of CraftLaunch 2.

---

## 1. Quick Start

### Step 1: Launch
- Run `Re-CraftLaunch.exe`.
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
