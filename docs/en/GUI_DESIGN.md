# KeyStar: GUI Design & UI/UX Specifications (GUI_DESIGN)

## 1. Design Philosophy (Spirit of CraftLaunch 2)

```text
+-------------------------------------------------------+
| > [ Type here (Minimal single-line input box)       ] |
+-------------------------------------------------------+
| = 1,408 (Instant arithmetic evaluation result) |
| :--------------------------------------------- |
| * Chrome.lnk                                   |
+-------------------------------------------------------+
```

The greatest appeal of the classic "CraftLaunch 2" was its unobtrusive presence: **"It occupies zero unnecessary screen estate, never gets in your way, and is immediately there as your typing target the millisecond you summon it."**
`KeyStar` intentionally rejects modern visual bloat (translucent acrylic blurs, oversized search cards, heavy thumbnails) to prioritize **razor-sharp tactile speed and immediate legibility**.

---

## 2. Window Geometry & Placement

| Parameter              | Specification                                                |
| :--------------------- | :----------------------------------------------------------- |
| **Window Frame**       | Borderless / Frameless (no title bar or system decorations)  |
| **Window Level**       | Always on Top (appears above all other application windows)  |
| **Taskbar Presence**   | Excluded from taskbar (background resident in system tray)   |
| **Initial Position**   | Screen center or top edge of primary monitor                 |
| **Dimensions**         | Width: 450–550 px / Height (input bar only): 36–42 px        |
| **Dropdown Expansion** | Dynamic vertical expansion downwards (max 5–10 rows, ~180px) |

---

## 3. UI Component Architecture

### 3.1 Single-Line Input Bar (`TextEdit`)
- Hint Text: `> Type here... (Shortcut name or calculation)`
- Instant automatic focus upon window reveal (zero keystroke loss).
- Background: Dark charcoal (`#1e1e1e`).
- Foreground Text: High-contrast white (`#ffffff`).
- Monospace font for exact visual alignment.

### 3.2 Candidate Dropdown Panel
- **Expansion Trigger**:
  - Automatically unfolds when $\ge 1$ character is typed or when `Down` / `Tab` is pressed.
  - Automatically contracts to single-line height when the input buffer is empty.
- **Visual Row Styling**:
  - **Calculation Result Row**:
    - Highlighted at index 0 with teal accent prefix (`= 1,408`).
  - **Shortcut Candidate Rows**:
    - Clean asterisk or icon prefix with file name (`* Chrome.lnk`).
  - **Selected Row (Active Navigation)**:
    - Inverted solid background highlight (accent blue `#264f78`).

---

## 4. Keyboard Interaction Specification

| Keystroke           | Action / Behavior                                                                                                      |
| :------------------ | :--------------------------------------------------------------------------------------------------------------------- |
| `Alt + Space`       | Global hotkey toggling launcher between visible and hidden                                                             |
| `Enter`             | **Execute Selection**: <br>• Calculation: Copy to OS clipboard & hide<br>• Shortcut: Launch detached via `open` & hide |
| `Esc`               | First press clears text; second press hides launcher window                                                            |
| `Tab` / `Shift+Tab` | Move candidate selection forward / backward (and completes)                                                            |
| `Ctrl+n` / `Ctrl+p` | Vim-style candidate navigation (down / up)                                                                             |
| `Down` / `Up`       | Arrow key candidate navigation                                                                                         |

---

## 5. `eframe` / `egui` Implementation Strategy

- **Native Options**:
  - Configure `decorated: false`, `always_on_top: true`, `taskbar: false` in `ViewportBuilder`.
- **Background Event Loop**:
  - When hidden (`Visible(false)`), winit sleeps with zero CPU overhead until `egui::Context::request_repaint()` is triggered by hotkey or tray hooks.
- **Focus Locking**:
  - Call `TextEdit::lock_focus(true)` and preemptively consume keys via `ui.input_mut().consume_key` to prevent focus loss.
