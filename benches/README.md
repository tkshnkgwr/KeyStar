# Re:CraftLaunch Benchmarks (性能測定・極限ベンチマーク置き場)

本ディレクトリは、`Re:CraftLaunch` の電卓エンジン、補完検索エンジン、ショートカットインデクサの処理速度を、マイクロ秒（$\mu s$）〜ナノ秒（$ns$）単位で厳密に測定・検証するための **ベンチマーク（Benchmarks）** 置き場です（Phase 4 導入予定）。

---

## 📌 追加予定ベンチマーク一覧

| ベンチマーク名     | 測定対象         | 比較・検証内容                                                      |
| :----------------- | :--------------- | :------------------------------------------------------------------ |
| `calc_bench.rs`    | `src/calc.rs`    | 四則演算・実数除算・カンマ整形のパースおよび評価速度（目標 <0.1ms） |
| `search_bench.rs`  | `src/search.rs`  | 1,000件のショートカットに対する前方・部分一致スコアリング速度       |
| `indexer_bench.rs` | `src/indexer.rs` | ディレクトリ内ファイル走査と更新検知（`needs_rescan`）の処理速度    |

---

## 🛠️ 推奨フレームワーク（Criterion.rs）

統計的解析および HTML レポート生成に対応した **`criterion`** クレートを採用予定です。

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "search_bench"
harness = false
```

---

## 🚀 実行コマンド（導入後）

```powershell
# ワークスペース全体のベンチマーク実行
cargo bench

# 特定のベンチマークのみ実行
cargo bench --bench search_bench
```
