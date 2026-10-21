# KeyStar: リリース手順書 (RELEASE)

本書は、`KeyStar` の公式リリース手順、バージョンタグ付与、最適化バイナリビルド、およびパッケージング手順を定めます。

---

## 1. リリース前チェックリスト

リリースビルドおよびタグ作成を行う前に、以下の項目がすべて満たされていることを確認します：

- [ ] すべてのテストが成功すること (`cargo test --all-targets` & `cargo test --doc`)
- [ ] 静的リント解析で警告が 0 件であること (`cargo clippy --all-targets -- -D warnings`)
- [ ] コードフォーマットが整っていること (`cargo fmt --check`)
- [ ] 全ソースコードの規模が 1000 行制限に適合していること (`fd -e rs --exec-batch wc -l`)
- [ ] `Cargo.toml` のバージョン番号（SemVer）が更新されていること
- [ ] `README.md` および `README_JA.md` のバッジ・ステータス記述が更新されていること
- [ ] `docs/ja/CHANGELOG.md` および `docs/en/CHANGELOG.md` の全更新履歴が同期されていること
- [ ] 手動受入テスト（`docs/ja/MANUAL_TEST_GUIDE.md`）の主要項目が実機で確認されていること

---

## 2. 最適化リリースビルド手順

Windows 向け単一スタンドアロン実行ファイルをビルドします：

```powershell
# 1. 最適化リリースバイナリのビルド
cargo build --release

# 2. 生成バイナリの確認
ls target/release/KeyStar.exe
```

---

## 3. バージョンタグ付与 & Git プッシュ手順

> [!IMPORTANT]
> Git コミットおよびプッシュは、必ずボスの明示的な指示のもとで行います（`RULES.md` 遵守）。

```powershell
# 1. コミットの作成
git add .
git commit -m "chore: release vX.Y.Z"

# 2. 注釈付きタグの作成
git tag -a vX.Y.Z -m "vX.Y.Z: リリース要約"

# 3. リモートリポジトリへ反映
git push origin main
git push origin --tags
```

---

## 4. 配布パッケージング (Scoop / ZIP)

```powershell
# リリース用 ZIP アーカイブの生成
Compress-Archive -Path target/release/KeyStar.exe, LICENSE, README_JA.md -DestinationPath target/release/KeyStar-vX.Y.Z-windows-x64.zip
```
