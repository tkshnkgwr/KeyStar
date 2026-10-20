# Re:CraftLaunch: 変更履歴 (CHANGELOG)

本プロジェクトのすべての主要な変更を記録します。フォーマットは [Keep a Changelog](https://keepachangelog.com/) に準拠し、バージョン番号は [セマンティックバージョニング (SemVer)](https://semver.org/) を採用しています。

---

## [0.1.0-alpha.2] - 2026-10-20

### Added
- **Phase 1 コアエンジンの実装**:
  - **電卓エンジン (`src/calc.rs`)**:
    - `evalexpr` によるリアルタイム数式パースおよび四則演算・剰余・括弧評価。
    - 実数除算（`10 / 4 = 2.5`）の自動昇格・判定アルゴリズム。
    - 3桁カンマ区切り整形（`1,408`）およびクリップボード用プレーン文字列（`1408`）の生成。
    - 単なる数字や英単語での誤作動を防ぐ `is_evaluable` 判定。
  - **ショートカットインデクサ (`src/indexer.rs`)**:
    - `%USERPROFILE%\commands` および `./shortcuts` の自動走査・存在保証。
    - `.lnk`, `.url`, `.bat`, `.cmd`, `.exe`, `.ps1` の大文字小文字透過スキャンとパス重複排除。
    - 監視ディレクトリのタイムスタンプ比較による超軽量な更新検知 (`needs_rescan`)。
  - **インクリメンタル補完エンジン (`src/search.rs`)**:
    - 前方一致（最優先）および部分一致の動的スコアリング。
    - 電卓結果が存在する場合の最優先（index 0）固定配置。
    - 空クエリ時の件数制御および最大件数制限。
  - **3層テストスイート & 品質検証 (`tests/`)**:
    - 内部単体テスト 15件、公開API統合テスト 12件、Docテスト 7件（全34件テスト 100% PASS）。
    - 全ソースコードの規模検証（最大316行、1000行ルール完全適合）。
- **日英バイリンガル設計資料群の全面配備 (`docs/`)**:
  - `ARCHITECTURE.md` (システム構成・データフロー)
  - `SPECIFICATION.md` (詳細機能仕様)
  - `GUI_DESIGN.md` (1行ウィンドウ・候補パネルUI/UX仕様)
  - `DEVELOPMENT.md` (開発環境構築・ビルド手順・品質規約)
  - `USER_GUIDE.md` (ユーザー向け操作マニュアル)
  - `TESTING.md` & `TEST_REPORT.md` (テスト設計方針 & 公式テストレポート)
  - `CONTRIBUTING.md` & `SECURITY.md` (開発スコープ規約 & セキュリティ方針)

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
