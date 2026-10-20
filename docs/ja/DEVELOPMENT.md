# KeyStar: 開発ガイド (DEVELOPMENT)

本ドキュメントは、`KeyStar` の開発環境構築、ビルド、テスト、およびコード品質維持のための開発者向けガイドラインです。

---

## 1. 開発前提環境

| 項目           | 推奨・必須要件                                        |
| :------------- | :---------------------------------------------------- |
| **OS**         | Windows 11 (x86_64)                                   |
| **Rust**       | Rust 1.85+（**Edition 2024** 対応）                   |
| **Cargo**      | 付属の最新版                                          |
| **CLI ツール** | `ripgrep` (`rg`), `fd`, `bat`, `jq`（高速調査・推奨） |
| **エディタ**   | VS Code (rust-analyzer) または gVim                   |

---

## 2. 環境構築 & 初回セットアップ

```powershell
# 1. リポジトリのクローン
git clone https://github.com/tkshnkgwr/KeyStar.git KeyStar
cd KeyStar

# 2. 依存関係の取得とコンパイルチェック
cargo check --all-targets

# 3. テストの全実行
cargo test --all-targets
```

---

## 3. ビルドコマンド一覧

### 通常開発ビルド & 実行
```powershell
# 開発ビルドで起動
cargo run

# 最適化リリースバイナリのビルド
cargo build --release
```

### コミット前必須検証コマンド
コードを変更した後は、コミット前に以下のコマンド群を順次実行し、エラーおよび警告が 0 件であることを確認してください：

```powershell
# 1. コードフォーマット検証
cargo fmt --check

# 2. Clippy による静的リント検証（警告ゼロ必須）
cargo clippy --all-targets -- -D warnings

# 3. 単体・統合・Docテストの全実行
cargo test --all-targets
cargo test --doc
```

※ フォーマット違反がある場合は `cargo fmt` を実行して自動整形を適用してください。

---

## 4. プロジェクト特有の絶対遵守規約

1. **自動コミット/Pushの禁止**:
   - AIエージェント自身による自律的なコミットやプッシュは固く禁止されています。必ずボスの明示的な指示（「コミットして」等）があった場合のみ実行してください。
2. **パス表記の環境変数化**:
   - ユーザー固有の生パス（`C:\Users\<user>\...`）をコードやドキュメントにハードコードすることは禁止です。必ず `%USERPROFILE%`（または `$env:USERPROFILE`）を使用してください。
3. **Markdownテーブルの垂直整列**:
   - 表組みはgVim等の生テキストエディタで閲覧した際にも美しく揃うよう、パイプ `|` とハイフン `-` を完全に垂直整列させてください。
4. **テストコードの責務分離**:
   - クレート外部から利用可能な公開仕様は `tests/*.rs` に配置し、内部の非公開ロジック・エッジケースのみを `src/*.rs` 内の `#[cfg(test)] mod tests` に留めてください。
5. **日本語Rustdocの記述**:
   - 公開シンボルには自然な日本語で目的、前提条件、副作用、`# Panics`、`# Examples` を記述してください。
