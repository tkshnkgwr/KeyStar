# Re:CraftLaunch: テスト結果報告書 (TEST_REPORT)

本ドキュメントは、`Re:CraftLaunch` の各開発フェーズにおける品質検証およびテスト実行結果を記録した公式レポートです。

---

## 📋 検証実施サマリー (Phase 3 OS連携・常駐完了時点)

| 検証項目                    | 対象スコープ                   | 実行コマンド                  |    結果ステータス    |
| :-------------------------- | :----------------------------- | :---------------------------- | :------------------: |
| **型チェック & 構文検証**   | 全ターゲット (`--all-targets`) | `cargo check --all-targets`   |   **合格 (PASS)**    |
| **内部単体テスト**          | `src/*.rs` (`#[cfg(test)]`)    | `cargo test --lib`            |    **27/27 PASS**    |
| **公開API統合テスト**       | `tests/*.rs`                   | `cargo test --test *`         |    **20/20 PASS**    |
| **Docテスト**               | `/// # Examples`               | `cargo test --doc`            |    **17/17 PASS**    |
| **静的リント解析**          | Clippy 警告ゼロ検査            | `cargo clippy -- -D warnings` |   **0警告 (PASS)**   |
| **コードフォーマット**      | Rust標準コードスタイル         | `cargo fmt --check`           |   **0差分 (PASS)**   |
| **コード規模 (1000行制限)** | 全 `.rs` ファイル              | 行数計測スクリプト            | **最大859行 (適合)** |

---

## 🧪 詳細テスト実行結果 (全 64 テスト完全合格)

### 1. 内部単体テスト (`src/lib.rs`) — 27 passed / 0 failed
```text
running 27 tests
test calc::tests::test_is_evaluable ... ok
test calc::tests::test_format_integer_with_commas ... ok
test calc::tests::test_evaluate_large_numbers_and_decimals ... ok
test calc::tests::test_evaluate_basic_arithmetic ... ok
test calc::tests::test_evaluate_invalid_cases ... ok
test calc::tests::test_evaluate_negative_numbers ... ok
test calc::tests::test_evaluate_nested_parentheses ... ok
test clipboard::tests::test_memory_clipboard_copy_and_overwrite ... ok
test clipboard::tests::test_clipboard_rejects_empty_text ... ok
test hotkey::tests::test_default_alt_space_binding ... ok
test hotkey::tests::test_should_trigger_hotkey_only_on_pressed ... ok
test indexer::tests::test_shortcut_item_from_path ... ok
test indexer::tests::test_indexer_scan_and_dedup ... ok
test indexer::tests::test_indexer_non_existent_dir ... ok
test search::tests::test_filter_substring_matching ... ok
test search::tests::test_filter_prefix_matching ... ok
test search::tests::test_filter_with_calc_result ... ok
test search::tests::test_filter_empty_query_and_limit ... ok
test search::tests::test_filter_no_match ... ok
test tray::tests::test_resolve_menu_commands ... ok
test tray::tests::test_generate_default_icon_rgba_and_build_icon ... ok
test ui::tests::test_initial_state_dropdown_hidden ... ok
test ui::tests::test_query_expands_dropdown_and_height ... ok
test ui::tests::test_navigation_cycle_next_and_prev ... ok
test ui::tests::test_confirm_shortcut_and_calc ... ok
test ui::tests::test_cancel_resets_and_dismisses ... ok
test ui::tests::test_toggle_visibility_and_focus_request ... ok

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

### 2. 公開API統合テスト (`tests/`) — 20 passed / 0 failed
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

running 4 tests (tests/os_integration.rs)
test test_global_hotkey_binding_and_press_filtering ... ok
test test_tray_menu_commands_and_embedded_icon ... ok
test test_clipboard_provider_and_calc_copy_pipeline ... ok
test test_hotkey_visibility_toggle_and_focus_lifecycle ... ok
test result: ok. 4 passed; 0 failed; finished in 0.00s

running 4 tests (tests/search_integration.rs)
test test_search_calculator_priority_merge ... ok
test test_search_prefix_vs_substring ... ok
test test_search_case_insensitivity ... ok
test test_search_max_results_limit ... ok
test result: ok. 4 passed; 0 failed; finished in 0.00s

running 4 tests (tests/ui_integration.rs)
test test_ui_incremental_completion_and_launch_scenario ... ok
test test_ui_calculator_evaluation_and_copy_scenario ... ok
test test_ui_empty_query_explicit_dropdown_and_escape ... ok
test test_native_options_frameless_configuration ... ok
test result: ok. 4 passed; 0 failed; finished in 0.00s
```

### 3. ドキュメンテーションテスト (`cargo test --doc`) — 17 passed / 0 failed
```text
running 17 tests
test src/calc.rs - calc::is_evaluable (line 32) ... ok
test src/calc.rs - calc::evaluate (line 84) ... ok
test src/clipboard.rs - clipboard::SystemClipboard::new (line 41) ... ok
test src/clipboard.rs - clipboard::MemoryClipboard::new (line 83) ... ok
test src/hotkey.rs - hotkey::HotKeyBinding::new (line 48) ... ok
test src/hotkey.rs - hotkey::HotKeyBinding::default_alt_space (line 68) ... ok
test src/hotkey.rs - hotkey::should_trigger_hotkey (line 106) ... ok
test src/indexer.rs - indexer::ShortcutItem::from_path (line 36) ... ok
test src/indexer.rs - indexer::ShortcutIndexer::new (line 86) ... ok
test src/indexer.rs - indexer::ShortcutIndexer::with_default_directories (line 107) ... ok
test src/search.rs - search::Candidate::from_calc (line 38) ... ok
test src/search.rs - search::filter_candidates (line 91) ... ok
test src/tray.rs - tray::resolve_menu_command (line 65) ... ok
test src/tray.rs - tray::generate_default_icon_rgba (line 109) ... ok
test src/ui.rs - ui::LauncherState::new (line 104) ... ok
test src/ui.rs - ui::LauncherState::toggle_visibility (line 162) ... ok
test src/ui.rs - ui::LauncherState::window_height (line 238) ... ok
test result: ok. 17 passed; 0 failed; finished in 0.32s
```
