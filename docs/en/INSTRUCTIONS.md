# KeyStar: Developer & Agent Instructions (INSTRUCTIONS)

This document provides operational guidelines, architectural tenets, and working instructions for developers and AI agents working on `KeyStar`.

---

## 1. Top-Priority Guardrails (RULES.md Compliance)

All activities in this workspace are strictly bound by the repository root [**`RULES.md`**](../../RULES.md) and [**`GEMINI.md`**](../../GEMINI.md).

1. **Strictly No Automatic Commits/Pushes**:
   - Never run `git commit` or `git push` without explicit instruction from the boss (user).
2. **Markdown Table Vertical Alignment**:
   - Align all table pipes `|` and hyphens `-` vertically to preserve legibility in plain-text editors like gVim.
3. **Addressing Protocol & Complete Localization**:
   - Always address the user as "ボス" (Boss) and deliver all conversational responses, logs, questions, and descriptions in 100% natural Japanese.
4. **Environment Variable Path Normalization**:
   - Never hardcode raw absolute user paths (`C:\Users\<user>\...`). Always standardize on `%USERPROFILE%` (or `$env:USERPROFILE`).
5. **High-Speed CLI Toolchain**:
   - Prioritize `rg` for text searching, `fd` for path finding, `bat --paging=never` for previewing, and `jq` for JSON processing.
6. **Code Size Ceiling (1,000-Line Limit)**:
   - No `.rs` file may exceed 1,000 lines. Refactor and modularize code before it breaches this threshold.

---

## 2. Project Philosophy & Clean-Room Directive

- **Reincarnation of CraftLaunch 2**:
  - Rather than following the bloated, Python-embedded direction of Version 3, faithfully resurrect the **Version 2 baseline (single-line, blazingly fast, keyboard-first, instant shortcut dispatch)** in Rust (Edition 2024).
  - Avoid heavy, full-drive indexing; maintain extreme minimalism by dispatching only shortcuts explicitly placed in designated folders.
- **Clean-Room Development**:
  - Do not inspect legacy internal implementation code or Python extension documentation. Emulate solely the **minimalist GUI appearance (1-line input bar & candidate dropdown)** and **keyboard ergonomics (global hotkey, incremental completion, Enter/Esc)**.

---

## 3. Mandatory Pre-Commit Verification Pipeline

Before committing any Rust changes (`*.rs`), execute the following quality checks in order (per `.agents/skills/rust-verification/`):

```powershell
# 1. Type check and syntax verification
cargo check --all-targets

# 2. Execute all unit and integration tests
cargo test --all-targets

# 3. Execute all documentation tests
cargo test --doc

# 4. Lint analysis (zero warnings enforced)
cargo clippy --all-targets -- -D warnings

# 5. Code formatting check
cargo fmt --check

# 6. File line count inspection (under 1,000 lines)
fd -e rs --exec-batch wc -l
```

---

## 4. Versioning & CHANGELOG Guidelines

- Adhere strictly to [Semantic Versioning (SemVer)](https://semver.org/). Bump `Cargo.toml`, `README.md`, and `README_JA.md` version badges upon milestone completion.
- Keep `CHANGELOG.md`, `docs/ja/CHANGELOG.md`, and `docs/en/CHANGELOG.md` synchronized across all releases.
- Always create annotated git tags (`git tag -a vX.Y.Z -m "..."`).
