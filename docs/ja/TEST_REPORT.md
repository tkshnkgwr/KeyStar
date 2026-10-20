# Re:CraftLaunch: テスト結果報告書 (TEST_REPORT)

本ドキュメントは、`Re:CraftLaunch` の各開発フェーズにおける品質検証およびテスト実行結果を記録した公式レポートです。

---

## 📋 検証実施サマリー (v0.1.0-alpha.1 Phase 1 完了時点)

| 検証項目                   | 対象スコープ                  | 実行コマンド                       | 結果ステータス  |
| :------------------------- | :---------------------------- | :--------------------------------- | :-------------: |
| **型チェック & 構文検証**  | 全ターゲット (`--all-targets`) | `cargo check --all-targets`        | **合格 (PASS)** |
| **内部単体テスト**         | `src/*.rs` (`#[cfg(test)]`)   | `cargo test --lib`                 | **15/15 PASS**  |
| **公開API統合テスト**      | `tests/*.rs`                  | `cargo test --test *`              | **12/12 PASS**  |
| **Docテスト**              | `/// # Examples`              | `cargo test --doc`                 | **7/7 PASS**    |
| **静的リント解析**         | Clippy 警告ゼロ検査           | `cargo clippy -- -D warnings`      | **0警告 (PASS)**|
| **コードフォーマット**     | Rust標準コードスタイル        | `cargo fmt --check`                | **0差分 (PASS)**|
| **コード規模 (1000行制限)** | 全 `.rs` ファイル             | 行数計測スクリプト                 | **最大316行 (適合)** |

---

## 🧪 詳細テスト実行結果

### 1. 内部単体テスト (`src/lib.rs`) — 15 passed / 0 failed
```text
running 15 tests
test calc::tests::test_is_evaluable ... ok
test calc::tests::test_format_integer_with_commas ... ok
test calc::tests::test_evaluate_large_numbers_and_decimals ... ok
test calc::tests::test_evaluate_basic_arithmetic ... ok
test calc::tests::test_evaluate_invalid_cases ... ok
test calc::tests::test_evaluate_negative_numbers ... ok
test calc::tests::test_evaluate_nested_parentheses ... ok
test search::tests::test_filter_substring_matching ... ok
test search::tests::test_filter_prefix_matching ... ok
test search::tests::test_filter_with_calc_result ... ok
test search::tests::test_filter_empty_query_and_limit ... ok
test search::tests::test_filter_no_match ... ok
test indexer::tests::test_shortcut_item_from_path ... ok
test indexer::tests::test_indexer_scan_and_dedup ... ok
test indexer::tests::test_indexer_non_existent_dir ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

### 2. 公開API統合テスト (`tests/`) — 12 passed / 0 failed
```text
running 5 tests (tests/calc_integration.rs)
test test_calc_basic_operations ... ok
test test_calc_compound_and_parentheses ... ok
test test_calc_decimal_and_comma_formatting ... ok
test test_calc_negative_and_zero_division ... ok
test test_calc_invalid_and_edge_cases ... ok
test result: ok. 5 passed; 0 failed; finished in 0.00s

running 3 tests (tests/indexer_integration.rs)
test test_shortcut_item_filtering ... ok
test test_indexer_lifecycle ... ok
test test_indexer_default_directories_exist ... ok
test result: ok. 3 passed; 0 failed; finished in 0.01s

running 4 tests (tests/search_integration.rs)
test test_search_calculator_priority_merge ... ok
test test_search_prefix_vs_substring ... ok
test test_search_case_insensitivity ... ok
test test_search_max_results_limit ... ok
test result: ok. 4 passed; 0 failed; finished in 0.00s
```

### 3. ドキュメンテーションテスト (`cargo test --doc`) — 7 passed / 0 failed
```text
running 7 tests
test src/calc.rs - calc::is_evaluable (line 32) ... ok
test src/calc.rs - calc::evaluate (line 84) ... ok
test src/indexer.rs - indexer::ShortcutItem::from_path (line 36) ... ok
test src/indexer.rs - indexer::ShortcutIndexer::new (line 86) ... ok
test src/indexer.rs - indexer::ShortcutIndexer::with_default_directories (line 107) ... ok
test src/search.rs - search::Candidate::from_calc (line 38) ... ok
test src/search.rs - search::filter_candidates (line 91) ... ok
test result: ok. 7 passed; 0 failed; finished in 0.08s
```
