# Re:CraftLaunch: Testing Architecture & Quality Guidelines (TESTING)

This document establishes the testing architecture, isolation conventions, and quality verification pipelines for `Re:CraftLaunch`.

---

## 1. 3-Layer Testing Architecture

```
                    ┌─────────────────────────────────────────┐
                    │          Rust Testing Framework         │
                    └────────────────────┬────────────────────┘
                                         │
       ┌─────────────────────────────────┼─────────────────────────────────┐
       ▼                                 ▼                                 ▼
[Internal Unit Tests]             [Public API Integration]          [Documentation Tests]
• Path: src/*.rs                  • Path: tests/*.rs                • Path: /// # Examples
• Viewpoint: White-box            • Viewpoint: Black-box (external) • Viewpoint: Contract validation
• Target: Private items & edges   • Target: Public API items (pub)  • Target: Representative usage
```

| Layer                      | Location                      | Primary Verification Objective                              | Command                    |
| :------------------------- | :---------------------------- | :---------------------------------------------------------- | :------------------------- |
| **Internal Unit Tests**    | `src/*.rs` (`mod tests`)      | Pinpoint testing for private algorithms, parsing, and edges | `cargo test --lib`         |
| **Public API Integration** | `tests/*.rs`                  | Full scenarios and multi-module behaviors from consumer eye | `cargo test --test <name>` |
| **Doc Tests**              | `src/*.rs` (`/// # Examples`) | Assuring code examples in documentation execute correctly   | `cargo test --doc`         |

---

## 2. Test Suite Organization

### 2.1 Calculator Suite (`src/calc.rs` & `tests/calc_integration.rs`)
- **Unit Tests**:
  - `is_evaluable`: Operator detection, exclusion of raw numbers, whitespace handling.
  - `format_integer_with_commas`: Thousands separator formatting, zero, and large numbers.
  - `promote_integers_to_floats`: Safety token promotion for real division.
- **Integration Tests**:
  - Full arithmetic (+, -, *, /, %).
  - Real division promotion (`10 / 4 = 2.5`).
  - Zero-division non-panic safety validation.

### 2.2 Indexer Suite (`src/indexer.rs` & `tests/indexer_integration.rs`)
- **Unit Tests**:
  - `ShortcutItem::from_path`: Extension filtering (`.lnk`, `.url`, `.bat`, etc.).
  - `scan`: Duplicate exclusion, case-insensitive normalization.
  - `needs_rescan`: Avoiding redundant rescans when folder timestamps are unchanged.
- **Integration Tests**:
  - Real filesystem scanning with temp directories.
  - Uppercase extension handling (`.EXE`, `.BAT`).

### 2.3 Search Suite (`src/search.rs` & `tests/search_integration.rs`)
- **Unit Tests**:
  - Prefix vs substring score weighting.
  - Fixed index 0 placement for active calculation results.
  - Empty query handling and `max_results` limits.
- **Integration Tests**:
  - Case insensitivity verification (`vscode` / `VSCODE`).
  - Combined shortcut and calculator candidate ranking.

### 2.4 UI & Keyboard Suite (`src/ui.rs` & `tests/ui_integration.rs`)
- **Unit Tests**:
  - Initial state (dropdown hidden, 42px base height).
  - Dynamic expansion and height calculation on query entry.
  - Keyboard navigation (Tab/Shift+Tab, Down/Up, circular wrapping).
  - Confirm (Enter) and Cancel (Esc) input resets.
- **Integration Tests**:
  - Incremental completion and shortcut execution scenario.
  - Real-time arithmetic evaluation and copy scenario.
  - Empty query explicit dropdown expansion and escape dismissal.
  - Frameless native window options verification.

### 2.5 OS Integration Suite (`src/hotkey.rs`, `src/tray.rs`, `src/clipboard.rs` & `tests/os_integration.rs`)
- **Unit Tests**:
  - `HotKeyBinding::default_alt_space` and press-only state filtering.
  - Tray context menu command resolution (Settings, Open Commands Folder, Exit).
  - In-memory procedural 32x32 RGBA icon generation.
  - `MemoryClipboard` in-memory text storage and empty string rejection.
- **Integration Tests**:
  - Global hotkey modifier distinction and key-release rejection.
  - Tray menu command dispatch and procedural icon creation.
  - End-to-end pipeline from arithmetic typing to clipboard storage.
  - Hotkey visibility toggle and automatic input focus restoration.

---

## 3. Test Execution Pipeline

```powershell
# 1. Execute all unit and integration tests
cargo test --all-targets

# 2. Execute documentation tests
cargo test --doc

# 3. Static lint analysis (zero warnings enforced)
cargo clippy --all-targets -- -D warnings

# 4. Code formatting verification
cargo fmt --check
```
