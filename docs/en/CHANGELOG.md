# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0-alpha.1] - 2026-10-20

### Added
- **Initial Project Setup & Environment Baseline (Phase 0)**:
  - **Agent & Skill Automation Framework (`.agents/`)**:
    - Strict prohibition of automatic Git commit/push policy (`git_policy.md`).
    - Pre-commit verification checklist (`rust_pre_commit.md`).
    - Test separation architecture guide (`tests/` integration vs `src/` unit tests) (`rust_testing.md`).
    - Rustdoc documentation standard in Japanese (`rustdoc.md`).
    - Safe Git commit & push workflow (`git-commit` skill).
    - Markdown table vertical alignment guide for gVim readability (`markdown-table` skill).
    - High-speed Rust quality check workflow (`rust-verification` skill).
    - Obsidian Vault fleeting note persistence workflow (`vault-memo` skill).
  - **Project Configuration & Code Style**:
    - Workspace directives (`RULES.md`) and AI master rules (`GEMINI.md`).
    - Standard Rust formatter configuration (`rustfmt.toml`: Edition 2024, max_width 100, tab_spaces 4).
    - Cross-editor formatting rules (`.editorconfig`: UTF-8, LF, space indent 4).
    - Git ignore rules (`.gitignore`).
  - **Documentation & Licensing**:
    - Bilingual documentation: English (`README.md`) and Japanese (`README_JA.md`).
    - Open source licensing under the MIT License (`LICENSE`).
    - Comprehensive technical specification (`docs/ja/SPECIFICATION.md`).
    - Development roadmap & task checklist (`docs/ja/TODO.md`, `TODO.md`).
  - **Core Dependencies & Verification**:
    - Added `eframe` (v0.37.0), `evalexpr` (v13.1.0), and `open` (v5.4.4).
    - Verified compilation, testing, clippy lints, and formatting with 100% pass rate.
