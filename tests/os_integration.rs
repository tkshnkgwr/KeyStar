//! Phase 3 OS連携・常駐機能（グローバルホットキー・システムトレイ・クリップボード連携）の統合テスト。

use KeyStar::clipboard::{ClipboardProvider, MemoryClipboard};
use KeyStar::hotkey::{HotKeyBinding, should_trigger_hotkey};
use KeyStar::indexer::ShortcutItem;
use KeyStar::tray::{
    DEFAULT_ICON_SIZE, MENU_ID_EXIT, MENU_ID_OPEN_COMMANDS, MENU_ID_SETTINGS, TrayCommand,
    build_default_tray_icon, generate_default_icon_rgba, resolve_menu_command,
};
use KeyStar::ui::{KeyCommand, LauncherAction, LauncherState};
use global_hotkey::{
    HotKeyState,
    hotkey::{Code, Modifiers},
};
use std::path::PathBuf;

#[test]
fn test_global_hotkey_binding_and_press_filtering() {
    let alt_space = HotKeyBinding::default_alt_space();
    let ctrl_space = HotKeyBinding::new(Some(Modifiers::CONTROL), Code::Space);

    // 異なる修飾キーの組み合わせは異なるホットキーIDを持つこと
    assert_ne!(alt_space.hotkey_id(), ctrl_space.hotkey_id());

    let target_id = alt_space.hotkey_id();
    // Pressed のみトグル発火し、Released や別キーIDは無視されること
    assert!(should_trigger_hotkey(
        target_id,
        target_id,
        HotKeyState::Pressed
    ));
    assert!(!should_trigger_hotkey(
        target_id,
        target_id,
        HotKeyState::Released
    ));
    assert!(!should_trigger_hotkey(
        target_id,
        ctrl_space.hotkey_id(),
        HotKeyState::Pressed
    ));
}

#[test]
fn test_tray_menu_commands_and_embedded_icon() {
    // 右クリックメニュー3項目（「設定」「コマンドフォルダを開く」「終了」）の解決検証
    assert_eq!(
        resolve_menu_command(MENU_ID_SETTINGS),
        Some(TrayCommand::OpenSettings)
    );
    assert_eq!(
        resolve_menu_command(MENU_ID_OPEN_COMMANDS),
        Some(TrayCommand::OpenCommandsFolder)
    );
    assert_eq!(resolve_menu_command(MENU_ID_EXIT), Some(TrayCommand::Exit));
    assert_eq!(resolve_menu_command("non_existent"), None);

    // 単一バイナリ内蔵のトレイアイコン生成検証
    let rgba = generate_default_icon_rgba(DEFAULT_ICON_SIZE, DEFAULT_ICON_SIZE);
    assert_eq!(
        rgba.len(),
        (DEFAULT_ICON_SIZE * DEFAULT_ICON_SIZE * 4) as usize
    );
    assert!(build_default_tray_icon().is_ok());
}

#[test]
fn test_clipboard_provider_and_calc_copy_pipeline() {
    let shortcuts: Vec<ShortcutItem> = Vec::new();
    let mut state = LauncherState::new(8);
    let mut clipboard = MemoryClipboard::new();

    // 1. 電卓式を入力して Enter 確定
    state.update_query("(1280 + 720) * 1.1", &shortcuts);
    let action = state.handle_key_command(KeyCommand::Confirm, &shortcuts);
    assert_eq!(action, LauncherAction::CopyCalcResult("2200".to_string()));

    // 2. クリップボードプロバイダへ書き込み
    if let LauncherAction::CopyCalcResult(text) = action {
        clipboard.copy_text(&text).expect("clipboard copy succeeds");
    }
    assert_eq!(clipboard.last_copied_text(), Some("2200"));
}

#[test]
fn test_hotkey_visibility_toggle_and_focus_lifecycle() {
    let shortcuts = vec![ShortcutItem {
        name: "Terminal".to_string(),
        path: PathBuf::from("C:\\commands\\Terminal.lnk"),
        extension: "lnk".to_string(),
    }];

    let mut state = LauncherState::new(8);
    assert!(state.is_window_visible());
    assert!(state.take_focus_request());
    assert!(!state.is_focus_requested());

    // 入力中に Alt + Space で非表示化 ➔ クエリはクリアされウィンドウ非表示へ
    state.update_query("term", &shortcuts);
    assert!(state.is_dropdown_visible());
    let visible_after_hide = state.toggle_visibility(&shortcuts);
    assert!(!visible_after_hide);
    assert!(!state.is_window_visible());
    assert_eq!(state.query(), "");
    assert!(!state.is_dropdown_visible());

    // バックグラウンド待機中から Alt + Space で再表示 ➔ ウィンドウ表示 & 入力欄フォーカス強制要求ON
    let visible_after_show = state.toggle_visibility(&shortcuts);
    assert!(visible_after_show);
    assert!(state.is_window_visible());
    assert!(state.is_focus_requested());
}
