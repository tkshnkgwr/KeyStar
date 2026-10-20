# KeyStar: Contribution Guidelines (CONTRIBUTING)

Thank you for your interest in contributing to `KeyStar`!
This project prioritizes preserving the core spirit of CraftLaunch 2 (ultra-lightweight, blazing fast, keyboard-first) while sustaining a clean, highly reliable Rust codebase.

---

## 1. Scope Policy & Guiding Principles

- **Preserving Minimalism**:
  - Feature proposals are welcome, but turning into a bloated, heavily configured launcher (the trap CraftLaunch 3 fell into) is strictly avoided.
  - Proposals for "full-drive indexing", "heavy web tech / Electron runtimes", or "complex scripting runtimes" will be politely declined.
- **Clean-Room Development**:
  - We do not copy internal legacy proprietary code. All features are built from scratch in Rust, emulating only the visual minimalism and keyboard ergonomics.

---

## 2. Commit Message Guidelines

Format commit messages as follows:

```text
<prefix>: <Clear summary of changes>

[Optional detailed description]
```

### Commit Prefixes
- `feat:` New features or modules
- `fix:` Bug fixes
- `docs:` Documentation changes only
- `refactor:` Code restructuring without functional changes
- `test:` Adding or updating tests
- `chore:` Build configuration, dependencies, or maintenance

---

## 3. Pull Request (PR) Checklist

Before submitting a Pull Request, verify all checks below in your local environment:

- [ ] `cargo check --all-targets` passes with zero errors.
- [ ] `cargo test --all-targets` and `cargo test --doc` pass 100%.
- [ ] `cargo clippy --all-targets -- -D warnings` outputs zero warnings.
- [ ] `cargo fmt --check` passes with zero formatting diffs.
- [ ] All Rust source files remain strictly below the 1,000-line ceiling.
- [ ] Public items include Rustdoc documentation with `# Examples` and `# Panics`.
- [ ] Markdown tables maintain strict vertical pipe `|` and hyphen `-` alignment.
- [ ] Absolute paths are normalized using `%USERPROFILE%`.
