# Re:CraftLaunch: 変更履歴 (CHANGELOG)

本プロジェクトのすべての主要な変更を記録します。フォーマットは [Keep a Changelog](https://keepachangelog.com/) に準拠し、バージョン番号は [セマンティックバージョニング (SemVer)](https://semver.org/) を採用しています。

---

## [0.1.0-alpha.1] - 2026-10-20

### Added
- **初期開発環境・規約基盤の整備 (Phase 0)**:
  - **エージェント・スキル運用基盤 (`.agents/`)**:
    - 自動コミット/Push厳禁ポリシーの策定 (`git_policy.md`)
    - コミット前必須チェック規約 (`rust_pre_commit.md`)
    - テスト責務分離規約（`tests/` 統合テスト vs `src/` 単体テスト）(`rust_testing.md`)
    - 日本語Rustdoc生成規約 (`rustdoc.md`)
    - 安全なGitコミット・Pushワークフロー (`git-commit` スキル)
    - gVim視認性を担保するテーブル垂直整列スキル (`markdown-table` スキル)
    - 高速Rustコード事前検証スキル (`rust-verification` スキル)
    - Obsidian Vault受信箱への知見保存スキル (`vault-memo` スキル)
  - **プロジェクト設定 & コードスタイル**:
    - `RULES.md`（開発共通絶対遵守指令）および `GEMINI.md`（AI秘書最優先ルール）の配置
    - Rust標準フォーマッター設定 (`rustfmt.toml`: Edition 2024, max_width 100, tab_spaces 4)
    - エディタ設定 (`.editorconfig`: UTF-8, LF, スペース4)
    - Git管理除外設定 (`.gitignore`)
  - **ドキュメント & ライセンス体系**:
    - 英語README (`README.md`) および日本語README (`README_JA.md`) の作成
    - MITライセンス (`LICENSE`) の配置
    - 基本仕様書 (`docs/ja/SPECIFICATION.md`) の策定
    - 開発ロードマップ & タスク一覧 (`docs/ja/TODO.md`, `TODO.md`) の策定
  - **主要クレートの導入 & 動作検証**:
    - `eframe` (v0.37.0) / `egui` (GUI基盤)
    - `evalexpr` (v13.1.0) (軽量数式評価エンジン)
    - `open` (v5.4.4) (ショートカット・関連付け起動)
    - 初回 `cargo check`, `cargo test`, `cargo clippy`, `cargo fmt` の全通過確認
