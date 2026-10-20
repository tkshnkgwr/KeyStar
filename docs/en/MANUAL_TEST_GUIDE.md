# KeyStar: Manual Acceptance Test Checklist (MANUAL_TEST_GUIDE)

This document provides a comprehensive acceptance checklist for human testers and developers to verify the look, feel, and keyboard ergonomics of `KeyStar` on Windows.

---

## 🚀 Preparation & Launch

```powershell
# 1. Build optimized release binary
cargo build --release

# 2. Run
.\target\release\KeyStar.exe
```

---

## 1. Startup, Resident Daemon, and Taskbar Suppression

| No. | Test Item             | Procedure                                            | Expected Outcome                                            | Done |
| :-- | :-------------------- | :--------------------------------------------------- | :---------------------------------------------------------- | :--: |
| 1-1 | Taskbar Suppression   | Launch `KeyStar.exe`                                 | Window opens without a taskbar button, leaving screen clean | [ ]  |
| 1-2 | System Tray Residency | Inspect the Windows notification area (system tray)  | Embedded icon (`>_`) appears with tooltip on mouse hover    | [ ]  |
| 1-3 | Folder Initialization | Check `%USERPROFILE%\commands` in File Explorer      | Folder is automatically created on startup if absent        | [ ]  |
| 1-4 | Safe Multi-Launch     | Launch another instance while one is already running | Runs smoothly without socket collision or panic             | [ ]  |

---

## 2. Global Hotkey Activation & Visibility Toggle

| No. | Test Item             | Procedure                                        | Expected Outcome                                            | Done |
| :-- | :-------------------- | :----------------------------------------------- | :---------------------------------------------------------- | :--: |
| 2-1 | Global Hotkey Trigger | Press `Alt + Space` from any external window     | Minimal 1-line frame instantly pops up at screen center/top | [ ]  |
| 2-2 | Automatic Input Focus | Type characters immediately after `Alt + Space`  | Keystrokes register immediately without mouse clicking      | [ ]  |
| 2-3 | Tray Click Toggle     | Left-click the system tray icon                  | Launcher toggles between visible and hidden states          | [ ]  |
| 2-4 | Hotkey Dismiss Toggle | Press `Alt + Space` while launcher is visible    | Input resets and window hides immediately                   | [ ]  |
| 2-5 | Idle CPU Load         | Inspect `KeyStar.exe` CPU in Task Manager        | CPU usage remains steadily at 0.0% during background sleep  | [ ]  |

---

## 3. Incremental Completion & Dynamic Dropdown

| No. | Test Item             | Procedure                                           | Expected Outcome                                           | Done |
| :-- | :-------------------- | :-------------------------------------------------- | :--------------------------------------------------------- | :--: |
| 3-1 | Candidate Dropdown    | Type 1 character in the input box (e.g., `c`)       | Candidate panel expands downward, increasing window height | [ ]  |
| 3-2 | Prefix Match Priority | Verify ranking when typing `c` (e.g., `Chrome.lnk`) | Prefix matches rank strictly higher than substring matches | [ ]  |
| 3-3 | Case Insensitivity    | Type `chrome`, `CHROME`, and `Chrome`               | Exact same shortcuts match regardless of letter case       | [ ]  |
| 3-4 | Shrink on Empty Query | Backspace all text until input is empty             | Candidates vanish and window shrinks back to ~42px height  | [ ]  |
| 3-5 | Down Key Expansion    | Press `Down` key while input box is empty           | All registered shortcuts expand downward                   | [ ]  |

---

## 4. Keyboard Navigation Ergonomics

| No. | Test Item              | Procedure                                          | Expected Outcome                                          | Done |
| :-- | :--------------------- | :------------------------------------------------- | :-------------------------------------------------------- | :--: |
| 4-1 | Next Candidate (`Tab`) | Press `Tab` with candidates listed                 | Highlight bar moves down to the next candidate            | [ ]  |
| 4-2 | Prev Candidate (S-Tab) | Press `Shift + Tab`                                | Highlight bar moves up to the previous candidate          | [ ]  |
| 4-3 | Arrow Key Movement     | Press `Down` / `Up` keys                           | Smooth candidate navigation via standard arrow keys       | [ ]  |
| 4-4 | Vim-Style Navigation   | Press `Ctrl + n` / `Ctrl + p`                      | Smooth candidate navigation via Vim keys                  | [ ]  |
| 4-5 | Cyclic Wrapping        | Press `Down` on bottom row or `Up` on top row      | Seamlessly wraps around from bottom to top and vice versa | [ ]  |
| 4-6 | In-Place Completion    | Press `Tab` to cycle without executing immediately | Input buffer reflects selected candidate name             | [ ]  |

---

## 5. Shortcut Execution

| No. | Test Item          | Procedure                                             | Expected Outcome                                               | Done |
| :-- | :----------------- | :---------------------------------------------------- | :------------------------------------------------------------- | :--: |
| 5-1 | Application Launch | Select a shortcut (e.g. `Chrome.lnk`) and press Enter | Target application launches detached; launcher hides instantly | [ ]  |
| 5-2 | Web URL Shortcut   | Select a `.url` shortcut and press Enter              | Target webpage opens in default browser                        | [ ]  |
| 5-3 | Script Execution   | Select a `.bat` or `.ps1` script and press Enter      | Script executes properly in background                         | [ ]  |
| 5-4 | Parameter Passing  | Type arguments after shortcut name and press Enter    | Program launches with parameters passed                        | [ ]  |

---

## 6. Real-Time Math Evaluation & Clipboard Copy

| No. | Test Item               | Procedure                                         | Expected Outcome                                               | Done |
| :-- | :---------------------- | :------------------------------------------------ | :------------------------------------------------------------- | :--: |
| 6-1 | Real-Time Arithmetic    | Type `1280 * 1.1`                                 | Top row displays `= 1,408` in teal accent color                | [ ]  |
| 6-2 | Real Division Promotion | Type `10 / 4`                                     | Evaluates as floating-point `= 2.5` rather than integer `2`    | [ ]  |
| 6-3 | Compound Parentheses    | Type `(100 + 200) * 3`                            | Correct operator precedence yields `= 900`                     | [ ]  |
| 6-4 | False-Positive Guard    | Type plain words (`apple`) or raw numbers (`100`) | Calculator row does not trigger erroneously                    | [ ]  |
| 6-5 | Copy to Clipboard       | Press `Enter` on active calculation result        | Window hides, and pasting (`Ctrl+V`) outputs raw number string | [ ]  |

---

## 7. Escape Handling (CraftLaunch 2 Heritage)

| No. | Test Item            | Procedure                                             | Expected Outcome                                          | Done |
| :-- | :------------------- | :---------------------------------------------------- | :-------------------------------------------------------- | :--: |
| 7-1 | First Press Clear    | Press `Esc` once while query text is typed            | Clears input buffer and hides candidate dropdown          | [ ]  |
| 7-2 | Second Press Dismiss | Press `Esc` while input box is empty                  | Window dismisses immediately into background sleep        | [ ]  |
| 7-3 | Clean Re-Invocation  | Press `Alt + Space` right after dismissing with `Esc` | Opens cleanly with an empty input buffer ready for typing | [ ]  |

---

## 8. Directory Dynamic Rescan & Tray Menu

| No. | Test Item            | Procedure                                       | Expected Outcome                                          | Done |
| :-- | :------------------- | :---------------------------------------------- | :-------------------------------------------------------- | :--: |
| 8-1 | Dynamic Addition     | Drop a new `.lnk` into `%USERPROFILE%\commands` | Recognized on next keystroke without restarting launcher  | [ ]  |
| 8-2 | Tray Context Menu    | Right-click the system tray icon                | Menu shows "Settings", "Open Commands Folder", and "Exit" | [ ]  |
| 8-3 | Open Commands Folder | Select "Open Commands Folder" from tray menu    | File Explorer immediately opens `%USERPROFILE%\commands`  | [ ]  |
| 8-4 | Open Settings        | Select "Settings" from tray menu                | Opens shortcut directory and brings launcher to front     | [ ]  |
| 8-5 | Clean Exit           | Select "Exit" from tray menu                    | Tray icon is removed and process terminates cleanly       | [ ]  |
