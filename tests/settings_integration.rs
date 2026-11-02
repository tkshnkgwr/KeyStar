//! Phase 4 GUI設定画面・設定永続化 (`config.toml`) の公開API統合テスト。

use KeyStar::indexer::{ShortcutIndexer, ShortcutItem};
use KeyStar::settings::{
    AppConfig, CalcCopyFormat, DEFAULT_FONT_SIZE, DEFAULT_HOTKEY_STR, MAX_FONT_SIZE,
    MAX_MAX_CANDIDATES, MAX_WINDOW_WIDTH, MIN_FONT_SIZE, MIN_MAX_CANDIDATES, MIN_WINDOW_WIDTH,
    SettingsDialog, collapse_env_path_str, expand_env_path, format_hotkey_binding,
    parse_hotkey_binding,
};
use KeyStar::ui::{KeyCommand, LauncherAction, LauncherState, build_native_options_with_config};
use global_hotkey::hotkey::{Code, Modifiers};
use std::fs::{self, File};
use std::path::PathBuf;

#[test]
fn test_config_toml_roundtrip_and_file_persistence() {
    let temp_dir = std::env::temp_dir().join("keystar_test_settings_persistence");
    let _ = fs::remove_dir_all(&temp_dir);
    let config_path = temp_dir.join("sub").join("config.toml");

    let custom_cfg = AppConfig {
        hotkey: "Ctrl + Space".to_string(),
        auto_start: true,
        watch_directories: vec![
            "%USERPROFILE%\\commands".to_string(),
            "./my_shortcuts".to_string(),
        ],
        max_candidates: 12,
        target_extensions: vec!["lnk".to_string(), "bat".to_string(), "ps1".to_string()],
        window_width: 640.0,
        font_size: 18.0,
        calc_copy_format: CalcCopyFormat::CommaSeparated,
    };

    // 親ディレクトリごと自動作成して TOML 保存
    custom_cfg
        .save_to_path(&config_path)
        .expect("save config.toml");
    assert!(config_path.is_file());

    // ファイルから再読み込みして全項目が一致すること
    let loaded = AppConfig::load_from_path(&config_path).expect("load config.toml");
    assert_eq!(loaded, custom_cfg);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_config_sanitization_and_clamping() {
    let mut cfg = AppConfig {
        hotkey: "NotAValidKey".to_string(),
        auto_start: false,
        watch_directories: vec!["   ".to_string()],
        max_candidates: 999,
        target_extensions: vec![".LNK".to_string(), "lnk".to_string(), " .Bat ".to_string()],
        window_width: 100.0,
        font_size: 99.0,
        calc_copy_format: CalcCopyFormat::Plain,
    };

    cfg = cfg.sanitized();
    assert_eq!(cfg.hotkey, DEFAULT_HOTKEY_STR);
    assert_eq!(cfg.max_candidates, MAX_MAX_CANDIDATES);
    assert_eq!(cfg.window_width, MIN_WINDOW_WIDTH);
    assert_eq!(cfg.font_size, MAX_FONT_SIZE);
    // 空のディレクトリ指定はデフォルトの2件に復元される
    assert_eq!(cfg.watch_directories.len(), 2);
    // 拡張子は小文字・ドットなし・重複排除される
    assert_eq!(
        cfg.target_extensions,
        vec!["lnk".to_string(), "bat".to_string()]
    );

    // 下限・上限の逆方向クランプ検証
    let low_cfg = AppConfig {
        max_candidates: 0,
        window_width: 2000.0,
        font_size: 4.0,
        ..AppConfig::default()
    }
    .sanitized();
    assert_eq!(low_cfg.max_candidates, MIN_MAX_CANDIDATES);
    assert_eq!(low_cfg.window_width, MAX_WINDOW_WIDTH);
    assert_eq!(low_cfg.font_size, MIN_FONT_SIZE);
}

#[test]
fn test_hotkey_parsing_and_formatting() {
    let win_space = parse_hotkey_binding("Win + Space").expect("Win + Space parses");
    assert_eq!(win_space.modifiers, Some(Modifiers::META));
    assert_eq!(win_space.code, Code::Space);
    assert_eq!(format_hotkey_binding(win_space), "Win + Space");

    let ctrl_alt_k = parse_hotkey_binding("Ctrl + Alt + K").expect("Ctrl + Alt + K parses");
    assert_eq!(
        ctrl_alt_k.modifiers,
        Some(Modifiers::CONTROL | Modifiers::ALT)
    );
    assert_eq!(ctrl_alt_k.code, Code::KeyK);
    assert_eq!(format_hotkey_binding(ctrl_alt_k), "Ctrl + Alt + K");
}

#[test]
fn test_env_path_expansion_and_userprofile_collapsing() {
    if let Ok(profile) = std::env::var("USERPROFILE") {
        let expanded = expand_env_path("%USERPROFILE%\\commands");
        assert_eq!(expanded, PathBuf::from(&profile).join("commands"));

        let raw_abs = format!("{profile}\\my_tools");
        let collapsed = collapse_env_path_str(&raw_abs);
        assert_eq!(collapsed, "%USERPROFILE%\\my_tools");
    }
}

#[test]
fn test_settings_dialog_state_transitions() {
    let mut dialog = SettingsDialog::new();
    assert!(!dialog.is_open());

    let initial = AppConfig {
        window_width: 620.0,
        ..AppConfig::default()
    };
    dialog.open(initial);
    assert!(dialog.is_open());
    assert_eq!(dialog.draft().window_width, 620.0);

    // 監視ディレクトリの追加・重複無視・削除
    dialog.add_watch_directory("./custom_tools");
    dialog.add_watch_directory("./CUSTOM_TOOLS"); // 大文字小文字違いの重複は無視
    assert_eq!(dialog.draft().watch_directories.len(), 3);

    dialog.remove_selected_directory();
    assert_eq!(dialog.draft().watch_directories.len(), 2);

    // 対象拡張子の ON / OFF 切り替え
    dialog.set_extension_enabled("ps1", false);
    assert!(
        !dialog
            .draft()
            .target_extensions
            .contains(&"ps1".to_string())
    );
    dialog.set_extension_enabled(".PS1", true);
    assert!(
        dialog
            .draft()
            .target_extensions
            .contains(&"ps1".to_string())
    );

    // 初期値リセット
    dialog.reset_to_default();
    assert_eq!(dialog.draft().font_size, DEFAULT_FONT_SIZE);
    assert_eq!(dialog.draft().max_candidates, 8);

    dialog.close();
    assert!(!dialog.is_open());
}

#[test]
fn test_indexer_extension_filter_and_calc_format_with_settings() {
    let temp_dir = std::env::temp_dir().join("keystar_test_settings_indexer");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    File::create(temp_dir.join("Browser.lnk")).unwrap();
    File::create(temp_dir.join("Deploy.ps1")).unwrap();

    // .lnk のみを対象拡張子にした場合、.ps1 は除外される
    let mut indexer = ShortcutIndexer::new(vec![temp_dir.clone()]);
    indexer.set_target_extensions(vec!["lnk".to_string()]);
    assert_eq!(indexer.scan(), 1);
    assert_eq!(indexer.items()[0].name, "Browser");

    let _ = fs::remove_dir_all(&temp_dir);

    // 電卓コピー書式の切り替え（生数値 vs カンマ区切り）と最大候補数設定の検証
    let shortcuts: Vec<ShortcutItem> = Vec::new();
    let mut state = LauncherState::new(5);
    assert_eq!(state.max_candidates(), 5);

    state.set_calc_copy_format(CalcCopyFormat::CommaSeparated);
    state.update_query("1000 * 1.408", &shortcuts);
    let action = state.handle_key_command(KeyCommand::Confirm, &shortcuts);
    assert_eq!(action, LauncherAction::CopyCalcResult("1,408".to_string()));

    // NativeOptions への横幅設定反映検証
    let cfg = AppConfig {
        window_width: 650.0,
        ..AppConfig::default()
    };
    let opts = build_native_options_with_config(&cfg);
    assert_eq!(
        opts.viewport.inner_size,
        Some(eframe::egui::vec2(650.0, KeyStar::ui::BASE_WINDOW_HEIGHT))
    );
}
