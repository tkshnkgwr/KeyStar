# KeyStar: システムアーキテクチャ設計書 (ARCHITECTURE)

## 1. プロジェクト目的 & 設計思想

本プロジェクト（`KeyStar`）は、Windows向けの名作キーボードランチャー「**CraftLaunch 2**」の原点思想（1行・爆速・キーボード特化・登録ショートカット即起動）を受け継ぎ、モダンかつ安全・高速なプログラミング言語である **Rust (Edition 2024)** でゼロから再構築する超軽量キーボードランチャーです。

### 🎯 3大設計思想
1. **極限のミニマリズム（100〜300行の凝縮美）**:
   - PC全域を常時インデックス走査してメモリを食い潰す肥大化ツールではなく、「自分が決めたフォルダに置いたショートカットだけを最短キーストロークで叩く」という究極のシンプルさを追求します。
2. **クリーンルーム開発（内部実装は見ない）**:
   - 本家CraftLaunch 3が陥った「Pythonランタイム組み込みによる肥大化と設定複雑化」の罠を完全回避するため、Python拡張や内部構造は一切参照せず、**「1行テキスト枠・候補リストの外観」と「キーボード操作感（ホットキー、インクリメンタル補完、Enter/Esc）」のみ** をRustで忠実に再現します。
3. **完全なキーボード完結性**:
   - マウス操作を一切必要とせず、呼び出しから候補選択、アプリ起動、計算結果コピー、ウィンドウ非表示までがすべてキーボードのみで完結します。

---

## 2. システム全体構成

```mermaid
flowchart TD
    subgraph OS ["Windows OS 連携"]
        HotKey["グローバルホットキー (Alt+Space)"]
        FS["監視フォルダ (%USERPROFILE%\\commands)"]
        Clipboard["OS クリップボード"]
        ShellExecute["ShellExecuteW / open::that_detached"]
        TaskMgr["taskmgr.exe (タスクマネージャ)"]
    end

    subgraph App ["KeyStar アプリケーション本体"]
        Guard["二重起動防止 (SingleInstanceGuard)"]
        Tray["システムトレイ常駐 (tray-icon)"]
        GUI["1行極小ウィンドウ (eframe / egui)"]
        
        subgraph Core ["コアエンジン & モジュール群"]
            Indexer["ショートカットインデクサ (src/indexer.rs)"]
            Calc["数式評価器 (src/calc.rs)"]
            Cmd["ビルトインコマンド (src/command.rs)"]
            Search["インクリメンタル補完 (src/search.rs)"]
        end
    end

    Guard -->|多重起動抑止| App
    HotKey -->|トグルシグナル| GUI
    Tray -->|終了 / フォルダ開く| App
    FS -->|起動時 / タイムスタンプ検知| Indexer
    
    GUI -->|入力文字列| Search
    GUI -->|入力文字列| Calc
    GUI -->|/ で始まるクエリ| Cmd
    Indexer -->|キャッシュデータ| Search
    Calc -->|計算結果| Search
    Cmd -->|コマンド候補| Search
    
    Search -->|候補リスト| GUI
    GUI -->|Enter (ショートカット/引数)| ShellExecute
    GUI -->|Enter (計算/日付/IP)| Clipboard
    GUI -->|Enter (/task)| TaskMgr
```

---

## 3. モジュール構成と責務分離

| モジュール名      | 役割・責務                                                                    | 主要採用技術                  |
| :---------------- | :---------------------------------------------------------------------------- | :---------------------------- |
| `calc`            | 数式パース、四則演算・実数除算評価、3桁カンマ区切り整形、クリップボード用変換 | `evalexpr`                    |
| `indexer`         | `%USERPROFILE%\commands` および `./shortcuts` の走査、更新タイムスタンプ検知  | `std::fs`, `PathBuf`          |
| `search`          | 入力文字列とショートカット名の前方/部分一致スコアリング、電卓・コマンドマージ | 自前スコアリング              |
| `command`         | スラッシュコマンド定義、インクリメンタル検索、Windows APIシステム制御         | `windows-sys`, Win32 API      |
| `single_instance` | Windows名前付きミューテックスによる多重起動防止ガード                         | Win32 Named Mutex             |
| `hotkey`          | `global-hotkey` によるOS全体ショートカット監視、省電力ウェイクアップフック    | `global-hotkey`               |
| `tray`            | `tray-icon` によるトレイ常駐、プロシージャルRGBAアイコン生成、メニュー制御    | `tray-icon`                   |
| `clipboard`       | `arboard` によるOSクリップボードへの確実な同期書き込み                        | `arboard`                     |
| `ui`              | 1行極小入力テキストボックス、候補ドロップダウン描画、キーストローク状態機械   | `eframe`, `egui`              |
| `main`            | アプリケーションライフサイクル、ホットキー監視、システムトレイ常駐制御        | `eframe`, Windowsサブシステム |

---

## 4. コンポーネント詳細仕様

### 4.1 電卓エンジン (`src/calc.rs`)
- **安全なリアルタイム数式評価**:
  - `evalexpr` をラップし、ユーザーが入力するたびに即時評価。
  - 数字と演算子が含まれるか判定する `is_evaluable` により、通常の英単語入力や単なる数字での誤作動を完全防止。
- **実数除算の自動昇格**:
  - `evalexpr` 標準の整数除算（`10 / 4` $\to$ `2`）を補正し、除算を含む場合は自動的に浮動小数点数として評価（`10 / 4` $\to$ `2.5`）。
- **出力整形**:
  - 整数部は3桁カンマ区切り（`1280 * 1.1` $\to$ `= 1,408`）。
  - クリップボード用生数値（`1408`）を別保持。

### 4.2 ショートカットインデクサ (`src/indexer.rs`)
- **監視ディレクトリの多重管理**:
  1. `%USERPROFILE%\commands`（最優先・標準）
  2. `./shortcuts`（ポータブル利用向け）
- **対象拡張子**:
  - `.lnk`, `.url`, `.bat`, `.cmd`, `.exe`, `.ps1`（大文字小文字不問）
- **高速性・低負荷の担保**:
  - 起動時に全走査してメモリ上にファイル名とパスをキャッシュ。
  - 毎回のホットキー呼出時は、ディレクトリの更新タイムスタンプ（`modified`）のみを軽量チェックし、変更がない限り再スキャンを省略。

### 4.3 補完・検索エンジン (`src/search.rs`)
- **優先度制御**:
  - クエリが `/` で始まる場合、専用のスラッシュコマンド検索モードへ切り替え。
  - 電卓の計算結果が存在する場合、スコア `10,000` を付与して候補リストの index 0 に固定配置。
  - 前方一致（スコア `1000 - 文字数差`）を最優先。
  - 部分一致（スコア `500 - 出現位置 - 文字数差`）を次点優先。
- **大文字・小文字透過**:
  - クエリとファイル名の双方を小文字化してマッチング。
- **コマンドと引数の分離**:
  - `split_command_and_args` により、`cmd /k dir` などの引数入力中も先頭コマンド単語で候補リストを展開維持。

### 4.4 ビルトインコマンド (`src/command.rs`)
- **スラッシュコマンド群**:
  - `/task`, `/config`, `/cmd`, `/reload`, `/restart`, `/date`, `/today`, `/time`, `/ip`, `/lock`, `/sleep`, `/shutdown`, `/help`, `/exit` の全14種を静的テーブル定義。
- **Windows API 直接連携**:
  - 画面ロック: `LockWorkStation`
  - 日時取得: `GetLocalTime`
  - IPアドレス解決: パケット送信なしのOSルーティングテーブル直接参照

### 4.5 多重起動抑止 (`src/single_instance.rs`)
- **名前付きミューテックス**:
  - `CreateMutexW` による `Local\KeyStar_SingleInstance_Mutex` の排他獲得。
  - 多重起動検知時は即座にプロセスをクリーン終了し、二重常駐を完全防止。

---

## 5. 技術選定の理由と制約

1. **`eframe` / `egui` (即時描画GUI)**:
   - Web技術（Tauri/Electron）のような重厚なブラウザエンジンを排し、GPU即時描画による起動速度 0.001 秒・メモリ消費数MBの超軽量性を実現。
   - 1行の入力欄と数行の候補リストという極小UIに最適。
2. **`evalexpr`**:
   - 外部ランタイム（PythonやLua）を埋め込むことなく、Rust単体バイナリ内で数式評価を完結。
3. **`open` & `ShellExecuteW`**:
   - Windowsシェルの標準関連付けに従い、`.lnk` や `.url`、引数付きスクリプトを安全かつ確実に非同期起動。
