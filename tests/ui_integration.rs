//! UI状態機械・キーボード操作系の公開API統合テスト。

use Re_CraftLaunch::indexer::ShortcutItem;
use Re_CraftLaunch::ui::{
    BASE_WINDOW_HEIGHT, KeyCommand, LauncherAction, LauncherState, WINDOW_WIDTH,
    build_native_options,
};
use std::path::PathBuf;

fn sample_shortcuts() -> Vec<ShortcutItem> {
    vec![
        ShortcutItem {
            name: "Chrome".to_string(),
            path: PathBuf::from("C:\\commands\\Chrome.lnk"),
            extension: "lnk".to_string(),
        },
        ShortcutItem {
            name: "CmdPrompt".to_string(),
            path: PathBuf::from("C:\\commands\\CmdPrompt.bat"),
            extension: "bat".to_string(),
        },
        ShortcutItem {
            name: "Obsidian".to_string(),
            path: PathBuf::from("C:\\commands\\Obsidian.lnk"),
            extension: "lnk".to_string(),
        },
        ShortcutItem {
            name: "VSCode".to_string(),
            path: PathBuf::from("C:\\commands\\VSCode.lnk"),
            extension: "lnk".to_string(),
        },
    ]
}

#[test]
fn test_ui_incremental_completion_and_launch_scenario() {
    let shortcuts = sample_shortcuts();
    let mut state = LauncherState::new(8);
    state.refresh_candidates(&shortcuts);

    // 初期状態（未入力）ではドロップダウンは非表示で高さは1行分
    assert!(!state.is_dropdown_visible());
    assert_eq!(state.window_height(), BASE_WINDOW_HEIGHT);

    // "c" を入力 -> Chrome (前方), CmdPrompt (前方), VSCode (部分) の3件が展開
    state.update_query("c", &shortcuts);
    assert!(state.is_dropdown_visible());
    assert_eq!(state.visible_candidates().len(), 3);
    assert!(state.window_height() > BASE_WINDOW_HEIGHT);

    // Tab / Ctrl+n 相当で2番目の候補 (CmdPrompt) を選択
    let nav_action = state.handle_key_command(KeyCommand::SelectNext, &shortcuts);
    assert_eq!(nav_action, LauncherAction::None);
    assert_eq!(state.selected_index(), 1);

    // Enter で確定 -> CmdPrompt.bat の起動アクションが返り、状態は初期リセット
    let confirm_action = state.handle_key_command(KeyCommand::Confirm, &shortcuts);
    assert_eq!(
        confirm_action,
        LauncherAction::ExecuteShortcut(PathBuf::from("C:\\commands\\CmdPrompt.bat"))
    );
    assert_eq!(state.query(), "");
    assert!(!state.is_dropdown_visible());
    assert_eq!(state.window_height(), BASE_WINDOW_HEIGHT);
}

#[test]
fn test_ui_calculator_evaluation_and_copy_scenario() {
    let shortcuts = sample_shortcuts();
    let mut state = LauncherState::new(8);

    // 計算式を入力 -> 最上位（index 0）に電卓結果が配置される
    state.update_query("(100 + 200) * 4", &shortcuts);
    assert!(state.is_dropdown_visible());
    let selected = state.selected_candidate().unwrap();
    assert_eq!(selected.label, "= 1,200");

    // Enter 押下 -> クリップボードへのプレーン数値コピーアクションが返る
    let action = state.handle_key_command(KeyCommand::Confirm, &shortcuts);
    assert_eq!(action, LauncherAction::CopyCalcResult("1200".to_string()));
}

#[test]
fn test_ui_empty_query_explicit_dropdown_and_escape() {
    let shortcuts = sample_shortcuts();
    let mut state = LauncherState::new(8);
    state.refresh_candidates(&shortcuts);

    // 空クエリ時に SelectNext (Down / Tab) を押すと強制展開される
    state.handle_key_command(KeyCommand::SelectNext, &shortcuts);
    assert!(state.is_dropdown_visible());
    assert_eq!(state.visible_candidates().len(), 4);
    assert_eq!(state.selected_index(), 0);

    // 先頭で SelectPrev (Shift+Tab / Ctrl+p / Up) を押すと末尾（index 3）へ循環
    state.handle_key_command(KeyCommand::SelectPrev, &shortcuts);
    assert_eq!(state.selected_index(), 3);

    // Esc でキャンセル -> Dismiss アクションが返り、ドロップダウンが閉じる
    let esc_action = state.handle_key_command(KeyCommand::Cancel, &shortcuts);
    assert_eq!(esc_action, LauncherAction::Dismiss);
    assert!(!state.is_dropdown_visible());
}

#[test]
fn test_native_options_frameless_configuration() {
    let opts = build_native_options();
    assert!(opts.centered);
    assert_eq!(opts.viewport.decorations, Some(false));
    assert_eq!(opts.viewport.resizable, Some(false));
    assert_eq!(opts.viewport.taskbar, Some(false));
    assert_eq!(
        opts.viewport.inner_size,
        Some(eframe::egui::vec2(WINDOW_WIDTH, BASE_WINDOW_HEIGHT))
    );
}
