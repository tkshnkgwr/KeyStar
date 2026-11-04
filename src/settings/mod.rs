//! アプリケーション設定の永続化（`config.toml`）および GUI 設定ダイアログモジュール。
//!
//! ホットキー、Windows スタートアップ自動起動、監視ディレクトリ、対象拡張子、最大表示候補数、
//! ウィンドウ横幅、フォントサイズ、電卓コピー書式の読み込み・保存および `egui` による
//! 独立設定ウィンドウ描画を提供します。

pub mod config;
pub mod dialog;
pub mod os;

pub use config::{AppConfig, CalcCopyFormat};
pub use dialog::{SettingsDialog, SettingsDialogAction};
pub use os::{
    collapse_env_path, collapse_env_path_str, expand_env_path, format_hotkey_binding,
    parse_hotkey_binding, pick_folder_dialog, sync_windows_auto_start,
};

/// 設定ダイアログウィンドウの標準横幅（ピクセル）。
pub const SETTINGS_WINDOW_WIDTH: f32 = 480.0;

/// 設定ダイアログウィンドウの標準高さ（ピクセル）。
pub const SETTINGS_WINDOW_HEIGHT: f32 = 560.0;

/// 最大候補表示件数の設定可能な下限値。
pub const MIN_MAX_CANDIDATES: usize = 3;

/// 最大候補表示件数の設定可能な上限値。
pub const MAX_MAX_CANDIDATES: usize = 15;

/// ランチャーウィンドウ横幅の設定可能な下限値（ピクセル）。
pub const MIN_WINDOW_WIDTH: f32 = 400.0;

/// ランチャーウィンドウ横幅の設定可能な上限値（ピクセル）。
pub const MAX_WINDOW_WIDTH: f32 = 800.0;

/// フォントサイズの設定可能な下限値（ポイント）。
pub const MIN_FONT_SIZE: f32 = 12.0;

/// フォントサイズの設定可能な上限値（ポイント）。
pub const MAX_FONT_SIZE: f32 = 20.0;

/// 標準のフォントサイズ（ポイント）。
pub const DEFAULT_FONT_SIZE: f32 = 15.0;

/// 標準のグローバルホットキー文字列表現。
pub const DEFAULT_HOTKEY_STR: &str = "Alt + Space";

/// 設定画面のドロップダウンで選択可能な代表的なホットキープリセット一覧。
pub const HOTKEY_PRESETS: &[&str] = &[
    "Alt + Space",
    "Ctrl + Space",
    "Win + Space",
    "Alt + Shift + Space",
    "Ctrl + Shift + Space",
    "Ctrl + Alt + Space",
];
