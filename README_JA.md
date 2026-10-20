<div align="center">

# 🚀 Re:CraftLaunch

**名作「CraftLaunch 2」の原点思想を受け継ぐ、Rust製の超軽量1行キーボードランチャー**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust 2024](https://img.shields.io/badge/Rust-2024%20Edition-orange.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-v0.1.0--alpha.5-brightgreen.svg)](Cargo.toml)
[![Platform](https://img.shields.io/badge/Platform-Windows-blue.svg)](https://github.com/tkshnkgwr/KeyStar)

[English](./README.md) | [日本語](./README_JA.md)

</div>

---

```text
+-------------------------------------------------------+
| > [ ここに入力（1行の極小テキストボックス）         ] |
+-------------------------------------------------------+
| = 1,408 (四則演算の即時結果) |
| :--------------------------- |
| * Chrome.lnk                 |
+-------------------------------------------------------+
```

## 背景と開発方針

本ランチャー『Re:CraftLaunch』は、Windows向けの名作キーボードランチャー「CraftLaunch 2」のミニマリズムとキーボード完結の操作感をルーツとしています。

PC全体をインデックス検索する現代の巨大なランチャーとは異なり、**「自分が決めたショートカットだけを、最短キーストロークで即座に叩く」** というCraftLaunch 2の使い心地は、四半世紀近く経った今なお色褪せない至高の道具でした。

筆者は長年CraftLaunchを愛用してまいりましたが、現代のWindows 11環境において、単一バイナリ・超高速・依存関係ゼロで永久に壊れず動き続ける環境を自ら用意したいと考え、Rustによるクリーンルーム再構築（UIの見た目とキーボード操作感のみを踏襲）に着手いたしました。

* **開発体制**:  
  Rustによるコード実装は、AI（Gemini / 大賢者）と相談しながら進めています。
* **現在のステータス**:  
  現在は `v0.1.0-alpha.5`（α版完結マイルストーン）です。Phase 0（環境・規約）、Phase 1（コアロジック）、Phase 2（egui 1行GUI）、Phase 3（OS連携常駐：Alt+Spaceホットキー・トレイ・クリップボード）、および全15種の日英完全対称ドキュメント体系がすべて整備完了しました。ここから先の機能追加・修正（Phase 4: バイナリ最適化・Scoopパッケージング等）は **β版（Beta）** フェーズへ移行します。

---

## 🙏 謝辞とリスペクト (Special Thanks & Respect)

本プロジェクト『Re:CraftLaunch』は、**craftware** 氏が開発されたWindows向け不朽の名作コマンドラインランチャー「**CraftLaunch**（CraftLaunch 2 / 3）」に、最大限の敬意と心からの感謝を表して開発されています。

- **オリジナル**: CraftLaunch (clnch)
- **作者**: **craftware** 氏
- **公式サイト**: [craftware (Google Sites)](https://sites.google.com/site/craftware/clnch)
- **リポジトリ**: [GitHub (crftwr/clnch)](https://github.com/crftwr/clnch)

画面上部にポンと現れる極小の1行入力枠、前方一致による電光石火のインクリメンタル補完、そして計算結果の即時コピー。craftware 氏が生み出した CraftLaunch は、多くのパワーユーザーやプログラマーに「キーボードランチャーの真の快適さと感動」を教え、PC操作の文化そのものを大きく変えた偉大な金字塔です。

長年にわたり素晴らしいソフトウェアを生み出し、キーボード主体のデスクトップ環境に計り知れない恩恵をもたらしてくださった craftware 氏に、心より深く御礼申し上げます。

---

## 🎯 コンセプト & 開発動機

- **「CraftLaunch 2」の魂を受け継ぐ現代リメイク**:
  - 過剰に多機能化・Python埋め込みで肥大化した「CraftLaunch 3」ではなく、全ユーザーが熱狂した **Ver2の原点（1行・爆速・キーボード特化・登録ショートカット即起動）** をRustで蘇らせます。
  - PC全体を無駄に検索せず、**「自分が決めたショートカットだけを、最短キーストロークで即座に呼び出す」** という原点にして頂点の使い勝手を提供します。
- **研ぎ澄まされた2大機能**:
  1. **特定フォルダのファイル名インクリメンタル補完起動**:
     - `%USERPROFILE%\commands`（またはアプリ同梱の `shortcuts/` フォルダ）に `.lnk` / `.bat` / `.url` 等のショートカットを配置するだけ。
     - 高速メモリキャッシュと前方・部分一致で即座に絞り込み、`Enter` で即座にキック起動。
  2. **入力欄での即時四則演算（電卓）**:
     - 入力文字列に数値と演算子（`+ - * / % ( )`）が含まれている場合、数式として即座に評価（`evalexpr`）。
     - `Enter` を押すと計算結果の数値をOSクリップボードに自動コピーしてウィンドウを閉じます。
- **クリーンルーム開発**:
  - 本家の内部実装（クローズドまたはVer3のPython前提構造）には引きずられず、**「GUIの見た目（1行の極小枠、候補リスト）」と「キーボード操作感（ホットキー、インクリメンタル補完、Enter/Esc）」のみ** を踏襲してクリーンに構築します。

---

## 🛠️ 技術スタック

| 構成要素       | 採用クレート / 技術     | 選定理由                                                      |
| :------------- | :---------------------- | :------------------------------------------------------------ |
| **言語**       | **Rust (Edition 2024)** | メモリ安全性、超高速、ガベージコレクションなし                |
| **GUI**        | **`eframe` / `egui`**   | 依存が少なく単一exeにビルド可能。1行テキスト入力に最適        |
| **数式計算**   | **`evalexpr`**          | 軽量・高速な数式評価クレート。1行で四則演算を安全にパース可能 |
| **起動処理**   | **`open`**              | Windows標準の関連付けに従って `.lnk` や `.url` を確実にキック |
| **ホットキー** | **`global-hotkey`**     | バックグラウンド常駐時のOS全体ショートカット検知              |
| **トレイ常駐** | **`tray-icon`**         | タスクバーを汚さず、システムトレイに最小化常駐                |

---

## ⌨️ 操作体系

| 操作キー            | 動作内容                                                |
| :------------------ | :------------------------------------------------------ |
| `Alt + Space`       | ランチャーの表示 / 非表示トグル（グローバルホットキー） |
| `Enter`             | 選択候補の実行、または計算結果をコピーして閉じる        |
| `Esc`               | 何もせずウィンドウを非表示にする                        |
| `Tab` / `Shift+Tab` | 候補リストの次 / 前の項目を選択                         |
| `Ctrl+n` / `Ctrl+p` | Vim風の次 / 前の候補選択                                |
| `Down` / `Up`       | 矢印キーによる次 / 前の候補選択                         |

---

## 🚀 ビルド & 実行方法

### 必要環境
- Rust 1.85+ (Edition 2024 対応)
- Cargo

### ローカルビルド & 実行
```powershell
# リポジトリのクローン
git clone https://github.com/tkshnkgwr/KeyStar.git Re-CraftLaunch
cd Re-CraftLaunch

# 依存クレートのビルド & 実行
cargo run

# 最適化リリースバイナリのビルド
cargo build --release
```

---

## 📚 設計・開発ドキュメント一式

プロジェクトの詳細な設計仕様、開発規約、手動テスト手順、リソース指標は `docs/` 配下に体系的に整備されています：

| ドキュメント                                               | 内容・役割                                                 |
| :--------------------------------------------------------- | :--------------------------------------------------------- |
| [**ARCHITECTURE.md**](./docs/ja/ARCHITECTURE.md)           | 全体システム構成、データフロー、モジュール責務分離         |
| [**SPECIFICATION.md**](./docs/ja/SPECIFICATION.md)         | コア機能、検索スコアリング、電卓、ウィンドウ基本仕様       |
| [**GUI_DESIGN.md**](./docs/ja/GUI_DESIGN.md)               | CraftLaunch 2の佇まい、ウィンドウ寸法、配色、キー操作体系  |
| [**DEVELOPMENT.md**](./docs/ja/DEVELOPMENT.md)             | 開発前提環境、ビルド手順、コミット前検証コマンド           |
| [**USER_GUIDE.md**](./docs/ja/USER_GUIDE.md)               | 初回クイックスタート、ショートカット配置方法、操作一覧     |
| [**INSTRUCTIONS.md**](./docs/ja/INSTRUCTIONS.md)           | 開発者・AIエージェント行動規範、RULES.md連動指示書         |
| [**FOOTPRINTS.md**](./docs/ja/FOOTPRINTS.md)               | メモリ消費・CPU負荷・起動時間指標および実測値              |
| [**MANUAL_TEST_GUIDE.md**](./docs/ja/MANUAL_TEST_GUIDE.md) | 実機での手動受入テスト（8大セクション）チェックリスト      |
| [**TESTING.md**](./docs/ja/TESTING.md)                     | 3層テスト設計（単体・統合・Docテスト）方針                 |
| [**TEST_REPORT.md**](./docs/ja/TEST_REPORT.md)             | 全64テスト完全合格の公式テスト結果報告書                   |
| [**RELEASE.md**](./docs/ja/RELEASE.md)                     | リリース前チェックリスト、最適化ビルド、パッケージング手順 |
| [**CONTRIBUTING.md**](./docs/ja/CONTRIBUTING.md)           | スコープ方針、クリーンルーム開発規約、PR提出チェックリスト |
| [**SECURITY.md**](./docs/ja/SECURITY.md)                   | 権限境界、プロセス分離、防御的入力評価ポリシー             |
| [**TODO.md**](./docs/ja/TODO.md)                           | 各実装フェーズの進捗状況とタスク一覧                       |
| [**CHANGELOG.md**](./docs/ja/CHANGELOG.md)                 | バージョンごとの更新履歴（Keep a Changelog 準拠）          |

---

## 📂 ディレクトリ構成

```text
Re-CraftLaunch/
├── .agents/
│   ├── rules/              # Git規約、テスト規約、Rustdoc規約
│   └── skills/             # 検証・コミット・Markdown整形・Vault連携スキル
├── docs/                   # 仕様書・設計書・TODO
│   ├── en/                 # 英語版ドキュメント
│   └── ja/                 # 日本語版ドキュメント
│       ├── SPECIFICATION.md# 詳細機能仕様書
│       └── TODO.md         # 開発ロードマップ & タスク一覧
├── src/                    # ソースコード
│   └── main.rs
├── tests/                  # 統合・シナリオテスト
├── Cargo.toml              # クレート設定
├── CHANGELOG.md            # 変更履歴 (英語)
├── GEMINI.md               # AI秘書最優先ルール
├── LICENSE                 # MIT License
├── README.md               # 英語ドキュメント
├── README_JA.md            # 日本語ドキュメント
├── RULES.md                # 開発共通絶対遵守指令
└── rustfmt.toml            # フォーマッター設定
```

---

## 📄 ライセンス

本プロジェクトは **[MIT License](LICENSE)** の下で公開されています。

Copyright (c) 2026 tkshnkgwr  
Re-CraftLaunch is an independent tribute project inspired by the design and UI concepts of CraftLaunch by craftware.
