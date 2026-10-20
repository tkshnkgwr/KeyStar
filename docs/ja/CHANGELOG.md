# Re:CraftLaunch: 変更履歴 (CHANGELOG)

本プロジェクトのすべての主要な変更を記録します。フォーマットは [Keep a Changelog](https://keepachangelog.com/) に準拠し、バージョン番号は [セマンティックバージョニング (SemVer)](https://semver.org/) を採用しています。

---

## [0.1.0-alpha.5] - 2026-10-20

### Added
- **α版完結マイルストーン達成 ＆ 日英対称ドキュメント体系の全面整備**:
  - **日英完全対称（全15種）ドキュメント体系の確立**:
    - `docs/ja/` および `docs/en/` を完全な1対1の対称構造（各15ファイル）へ拡充。
    - `FOOTPRINTS.md`: リソース消費指標（起動<50ms、待機CPU 0.0%、RAM<15MB、復帰<1ms）、アルゴリズム計算量、CraftLaunch 3とのメモリ比較の定義。
    - `INSTRUCTIONS.md`: RULES.md/GEMINI.md連動の最優先行動規範、クリーンルーム開発方針、事前品質検証プロトコル。
    - `MANUAL_TEST_GUIDE.md`: 実機での8大受入テスト（常駐・ホットキー・補完・ナビゲーション・起動・電卓・Esc・トレイ）チェックリスト。
    - `RELEASE.md`: リリース前チェックリスト、最適化ビルド、タグ付け・プッシュ、パッケージング手順。
    - 英語版資料の完全補完（`GUI_DESIGN.md`, `CONTRIBUTING.md`, `SECURITY.md`, `TESTING.md`, `TEST_REPORT.md`）。
  - **ベンチマーク ＆ 実例サンプル置き場の配備**:
    - `benches/README.md`: 将来導入予定の `criterion` ベンチマーク設計書（電卓・検索・インデクサ）。
    - `examples/README.md`: GUI不要で単体動作を確認できるCLIサンプルコード置き場。
  - **Markdownテーブル垂直整列の自動化**:
    - `scripts/align_table.py`: 全角幅2計算による表組み垂直整列ツールの配備と、全30ファイル超への一括整形適用。
  - **α版完結 ＆ β版移行の宣言**:
    - α版（Alpha）マイルストーンの正式な完了。ここから先の機能追加・最適化（Phase 4: リリースLTO最適化・Scoopパッケージング等）は **β版（Beta）** フェーズへ移行。

## [0.1.0-alpha.4] - 2026-10-20

### Added
- **Phase 3 OS連携 & バックグラウンド常駐デーモンの実装**:
  - **グローバルホットキー監視基盤 (`src/hotkey.rs`)**:
    - `global-hotkey` クレートによる OS 全体での **`Alt + Space`** キー押下検知とウィンドウ表示トグル。
    - キー離脱イベント（`HotKeyState::Released`）を除外する `should_trigger_hotkey` 判定による二重発火防止。
    - `install_hotkey_wakeup_hook`: バックグラウンド待機（CPU負荷 0%）から `egui::Context::request_repaint()` による遅延ゼロの即時ウェイクアップ。
  - **システムトレイ常駐 & 内蔵プロシージャルアイコン (`src/tray.rs`)**:
    - `tray-icon` クレートによる Windows 通知領域常駐、および左クリックでのウィンドウ表示トグル。
    - 右クリックコンテキストメニュー（「設定」「コマンドフォルダを開く」「終了」）の完全装備。
    - `generate_default_icon_rgba`: 外部画像ファイル不要の 32x32 プロシージャル RGBA アイコン生成（ダークチャコール `#1e1e1e` 背景にティールグリーン `#4ec9b0` の `>_` プロンプト）。
  - **OS クリップボード同期書き込み (`src/clipboard.rs`)**:
    - `arboard` による電卓結果（`Enter` 確定）の OS クリップボードへの即時・確実な同期書き込み。
    - テスト検証用 `MemoryClipboard` と実環境用 `SystemClipboard` を抽象化する `ClipboardProvider` トレイトの設計。
  - **常駐ライフサイクル & フォーカス強制統合 (`src/ui.rs` & `src/main.rs`)**:
    - `LauncherState::toggle_visibility`: 表示状態と非表示待機状態のスムーズな相互トグル。
    - ホットキー復帰時の最前面化（`AlwaysOnTop`）、アクティブ化（`Focus`）、および 1 行テキストボックスへのフォーカス強制（`take_focus_request`）。
    - `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` によるリリースビルド時のコンソール非表示化。
  - **OS連携統合テストスイートの配備 (`tests/os_integration.rs`)**:
    - ホットキー判定、トレイメニュー解決、クリップボード同期コピー、トグル＆フォーカス遷移の4統合テストを追加。
    - テスト総数 64件（内部単体27件、公開API統合20件、Doc17件）100%合格達成。

## [0.1.0-alpha.3] - 2026-10-20

### Added
- **Phase 2 極小1行GUIウィンドウ & キーボード操作体系の実装**:
  - **GUIウィンドウ構造・状態管理アーキテクチャ (`src/ui.rs`)**:
    - `LauncherApp`: `eframe`/`egui` による枠なし（Frameless/Borderless）の極小1行入力ウィンドウ（高さ約36px、初期幅480px）。
    - `LauncherState`: 入力バッファ、インデックス走査、補完候補、選択行、アクションキューを完全に分離・統合管理する状態マシン設計。
    - `KeyCommand` & `LauncherAction`: キーボード入力と実行処理（`LaunchShortcut`, `CopyCalculation`, `Close`, `Hide`）の疎結合化。
  - **CraftLaunch 2互換キーボード操作体系**:
    - `Tab` / `Shift+Tab`: 次/前の候補選択および入力欄へのインクリメンタル補完（引数や先頭空白を保護）。
    - `Down` / `Up` & `Ctrl+n` / `Ctrl+p`: 候補リストの上下循環ナビゲーション。
    - `Enter`: 選択候補の実行（電卓結果の場合はクリップボードコピー、ショートカットの場合はバックグラウンド起動）。
    - `Esc`: 1回目で入力欄クリア、2回目でウィンドウ閉鎖/非表示（CraftLaunch 2の直感的エスケープ操作の再現）。
    - `lock_focus(true)` と `ui.input_mut().consume_key` による入力フォーカス逸脱の完全防止。
  - **動的ドロップダウン候補パネル**:
    - 入力時または下キー押下時に最大8件の候補リストを下方向へ自動展開。
    - 電卓結果行（`= 1,408`）の視覚的強調表示。
    - CraftLaunch 2のクラシックな操作感を意識したダークテーマ配色（背景 `#1e1e1e`、ハイライト `#2d4f7c`、文字 `#dcdcdc`）。
  - **UI統合テストスイートの配備 (`tests/ui_integration.rs`)**:
    - インクリメンタル補完、電卓評価＆コピー、空クエリ展開＆Esc終了、ネイティブウィンドウ設定の4シナリオ統合テストを追加。
    - テスト総数 45件（単体20件、統合16件、Doc9件）100%合格達成。

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
