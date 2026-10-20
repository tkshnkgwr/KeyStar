# KeyStar: Developer Guide (DEVELOPMENT)

This document provides developer guidelines for building, testing, and maintaining code quality in `KeyStar`.

---

## 1. Prerequisites

| Component     | Requirements & Recommendations                    |
| :------------ | :------------------------------------------------ |
| **OS**        | Windows 11 (x86_64)                               |
| **Rust**      | Rust 1.85+ (**Edition 2024** support)             |
| **Cargo**     | Latest bundled version                            |
| **CLI Tools** | `ripgrep` (`rg`), `fd`, `bat`, `jq` (Recommended) |
| **Editor**    | VS Code (rust-analyzer) or gVim                   |

---

## 2. Setup & First Run

```powershell
# 1. Clone repository
git clone https://github.com/tkshnkgwr/KeyStar.git KeyStar
cd KeyStar

# 2. Check compilation
cargo check --all-targets

# 3. Run test suite
cargo test --all-targets
```

---

## 3. Build & Test Commands

### Development Build
```powershell
# Run debug build
cargo run

# Build optimized release binary
cargo build --release
```

### Pre-Commit Verification Workflow
Ensure all of the following commands execute with zero errors and zero warnings:

```powershell
# 1. Formatting check
cargo fmt --check

# 2. Clippy lint check (zero warnings enforced)
cargo clippy --all-targets -- -D warnings

# 3. Unit, integration, and doc tests
cargo test --all-targets
cargo test --doc
```

---

## 4. Key Project Directives

1. **Strict Prohibition of Automatic Commit/Push**:
   - Commits and pushes must never be initiated automatically by AI assistants; explicit user approval is strictly required.
2. **Environment Variable Paths**:
   - Never hardcode raw user paths (e.g. `C:\Users\<user>\...`). Always use `%USERPROFILE%`.
3. **Markdown Table Vertical Alignment**:
   - Align table pipes `|` and hyphens `-` vertically for plain-text (gVim) readability.
4. **Test Separation Architecture**:
   - Public API specifications belong in `tests/*.rs`. Private logic and internal edge cases belong in `src/*.rs` under `#[cfg(test)]`.
