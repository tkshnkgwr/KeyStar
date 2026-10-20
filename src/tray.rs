//! システムトレイ常駐・コンテキストメニュー管理モジュール。
//!
//! `tray-icon` クレートを用いて Windows システムトレイ（通知領域）への常駐アイコン表示と、
//! 右クリックメニュー（「設定」「コマンドフォルダを開く」「終了」）および左クリックによる
//! ランチャー表示トグルを提供します。

use eframe::egui;
use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex, Once, OnceLock};
use tray_icon::{
    Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

/// システムトレイアイコンの標準解像度（ピクセル幅・高さ）。
pub const DEFAULT_ICON_SIZE: u32 = 32;

/// 右クリックメニュー「設定」の識別ID。
pub const MENU_ID_SETTINGS: &str = "settings";

/// 右クリックメニュー「コマンドフォルダを開く」の識別ID。
pub const MENU_ID_OPEN_COMMANDS: &str = "open_commands";

/// 右クリックメニュー「終了」の識別ID。
pub const MENU_ID_EXIT: &str = "exit";

/// システムトレイアイコンの標準ツールチップ文字列。
pub const DEFAULT_TRAY_TOOLTIP: &str = "KeyStar (Alt + Space)";

/// メニューイベントを保持する内部キュー。
static TRAY_MENU_QUEUE: LazyLock<Mutex<VecDeque<MenuEvent>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));

/// トレイアイコンクリックイベントを保持する内部キュー。
static TRAY_ICON_QUEUE: LazyLock<Mutex<VecDeque<TrayIconEvent>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));

/// バックグラウンド待機中の `egui` イベントループを即座に起こすためのコンテキスト保持セル。
static TRAY_WAKEUP_CTX: OnceLock<Mutex<Option<egui::Context>>> = OnceLock::new();

/// トレイイベントハンドラの初期化をプロセス内で1回のみ保証するガード。
static TRAY_HANDLER_INIT: Once = Once::new();

/// システムトレイ操作によって発火するコマンド。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayCommand {
    /// ランチャーウィンドウの表示 / 非表示をトグル（トレイアイコン左クリック時）
    ToggleWindow,
    /// 「設定」を選択（ランチャー表示およびショートカット設定ディレクトリの確認）
    OpenSettings,
    /// 「コマンドフォルダを開く」を選択（`%USERPROFILE%\commands` をエクスプローラーで開く）
    OpenCommandsFolder,
    /// 「終了」を選択（常駐プロセスを完全終了）
    Exit,
}

/// メニュー項目ID文字列から対応する [`TrayCommand`] を解決します。
///
/// # Panics
///
/// パニックは発生しません。
///
/// # Examples
///
/// ```
/// use KeyStar::tray::{
///     MENU_ID_EXIT, MENU_ID_OPEN_COMMANDS, MENU_ID_SETTINGS, TrayCommand, resolve_menu_command,
/// };
///
/// assert_eq!(resolve_menu_command(MENU_ID_SETTINGS), Some(TrayCommand::OpenSettings));
/// assert_eq!(resolve_menu_command(MENU_ID_OPEN_COMMANDS), Some(TrayCommand::OpenCommandsFolder));
/// assert_eq!(resolve_menu_command(MENU_ID_EXIT), Some(TrayCommand::Exit));
/// assert_eq!(resolve_menu_command("unknown"), None);
/// ```
#[must_use]
pub fn resolve_menu_command(menu_id: &str) -> Option<TrayCommand> {
    match menu_id {
        MENU_ID_SETTINGS => Some(TrayCommand::OpenSettings),
        MENU_ID_OPEN_COMMANDS => Some(TrayCommand::OpenCommandsFolder),
        MENU_ID_EXIT => Some(TrayCommand::Exit),
        _ => None,
    }
}

/// トレイアイコンイベントが「左クリックのリリース（確定）」であるかを判定します。
#[must_use]
pub fn is_tray_left_click_release(event: &TrayIconEvent) -> bool {
    matches!(
        event,
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        }
    )
}

/// 外部画像ファイルに依存しない、単一バイナリ内蔵の RGBA8 アイコンデータを動的生成します。
///
/// ダークチャコール背景（`#1e1e1e`）にティールグリーン枠（`#4ec9b0`）および中央の
/// プロンプト象徴ドットを描画した `width * height * 4` バイトのバッファを返します。
///
/// # Panics
///
/// パニックは発生しません。
///
/// # Examples
///
/// ```
/// use KeyStar::tray::{DEFAULT_ICON_SIZE, generate_default_icon_rgba};
///
/// let rgba = generate_default_icon_rgba(DEFAULT_ICON_SIZE, DEFAULT_ICON_SIZE);
/// assert_eq!(rgba.len(), (DEFAULT_ICON_SIZE * DEFAULT_ICON_SIZE * 4) as usize);
/// ```
#[must_use]
pub fn generate_default_icon_rgba(width: u32, height: u32) -> Vec<u8> {
    let w = width.max(1);
    let h = height.max(1);
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);

    for y in 0..h {
        for x in 0..w {
            let is_border = x < 2 || x >= w.saturating_sub(2) || y < 2 || y >= h.saturating_sub(2);
            // 中央左寄りに '>' 風のシェブロン、右下に '_' 風のアンダースコアバーを描画
            let mid_y = h / 2;
            let dy = y.abs_diff(mid_y);
            let is_chevron = (6..14).contains(&x) && dy <= 5 && (x - 6) == (5 - dy);
            let is_underscore = (15..25).contains(&x) && (mid_y + 3..=mid_y + 5).contains(&y);

            let color: [u8; 4] = if is_border || is_chevron {
                // ティールグリーン (#4ec9b0)
                [0x4e, 0xc9, 0xb0, 0xff]
            } else if is_underscore {
                // 高コントラスト白 (#ffffff)
                [0xff, 0xff, 0xff, 0xff]
            } else {
                // ダークチャコール (#1e1e1e)
                [0x1e, 0x1e, 0x1e, 0xff]
            };

            rgba.extend_from_slice(&color);
        }
    }

    rgba
}

/// 標準サイズのシステムトレイ用 [`Icon`] を生成します。
///
/// # Errors
///
/// RGBA バッファのサイズ不正等で `tray_icon::Icon` の構築に失敗した場合にエラーを返します。
pub fn build_default_tray_icon() -> Result<Icon, String> {
    let rgba = generate_default_icon_rgba(DEFAULT_ICON_SIZE, DEFAULT_ICON_SIZE);
    Icon::from_rgba(rgba, DEFAULT_ICON_SIZE, DEFAULT_ICON_SIZE)
        .map_err(|e| format!("トレイアイコンの生成に失敗しました: {e}"))
}

/// トレイイベント発火時に `egui` のイベントループを即座にウェイクアップするフックを登録します。
pub fn install_tray_wakeup_hook(ctx: egui::Context) {
    let cell = TRAY_WAKEUP_CTX.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = cell.lock() {
        *guard = Some(ctx);
    }

    TRAY_HANDLER_INIT.call_once(|| {
        MenuEvent::set_event_handler(Some(|event: MenuEvent| {
            if let Ok(mut queue) = TRAY_MENU_QUEUE.lock() {
                queue.push_back(event);
            }
            if let Some(cell) = TRAY_WAKEUP_CTX.get()
                && let Ok(guard) = cell.lock()
                && let Some(ctx) = guard.as_ref()
            {
                ctx.request_repaint();
            }
        }));

        TrayIconEvent::set_event_handler(Some(|event: TrayIconEvent| {
            if let Ok(mut queue) = TRAY_ICON_QUEUE.lock() {
                queue.push_back(event);
            }
            if let Some(cell) = TRAY_WAKEUP_CTX.get()
                && let Ok(guard) = cell.lock()
                && let Some(ctx) = guard.as_ref()
            {
                ctx.request_repaint();
            }
        }));
    });
}

/// システムトレイアイコンおよび右クリックメニューのライフサイクルとイベント取得を管理する構造体。
pub struct TrayService {
    /// OS トレイアイコン実体（ドロップ時にトレイから消去されるため保持）
    _tray_icon: TrayIcon,
}

impl TrayService {
    /// 標準アイコンおよび右クリックメニュー（「設定」「コマンドフォルダを開く」「終了」）を
    /// 備えた [`TrayService`] を生成します。
    ///
    /// # Errors
    ///
    /// メニュー構築または OS システムトレイへの登録に失敗した場合にエラーメッセージを返します。
    pub fn new_default() -> Result<Self, String> {
        let icon = build_default_tray_icon()?;
        let menu = Menu::new();

        let item_settings = MenuItem::with_id(MENU_ID_SETTINGS, "設定", true, None);
        let item_open_commands =
            MenuItem::with_id(MENU_ID_OPEN_COMMANDS, "コマンドフォルダを開く", true, None);
        let separator = PredefinedMenuItem::separator();
        let item_exit = MenuItem::with_id(MENU_ID_EXIT, "終了", true, None);

        menu.append_items(&[&item_settings, &item_open_commands, &separator, &item_exit])
            .map_err(|e| format!("トレイメニュー項目の追加に失敗しました: {e}"))?;

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip(DEFAULT_TRAY_TOOLTIP)
            .with_icon(icon)
            .build()
            .map_err(|e| format!("システムトレイアイコンの登録に失敗しました: {e}"))?;

        Ok(Self {
            _tray_icon: tray_icon,
        })
    }

    /// 蓄積されたトレイメニューイベントおよびアイコンクリックイベントから次の [`TrayCommand`] を取得します。
    #[must_use]
    pub fn poll_command(&self) -> Option<TrayCommand> {
        poll_pending_tray_command()
    }
}

/// キューおよびレシーバに蓄積されている未処理のトレイイベントを1件取り出し、[`TrayCommand`] に変換します。
#[must_use]
pub fn poll_pending_tray_command() -> Option<TrayCommand> {
    // 1. 右クリックメニュー選択イベントの確認（内部キュー）
    if let Ok(mut queue) = TRAY_MENU_QUEUE.lock() {
        while let Some(event) = queue.pop_front() {
            if let Some(cmd) = resolve_menu_command(event.id().0.as_str()) {
                return Some(cmd);
            }
        }
    }

    // 2. 右クリックメニュー選択イベントの確認（標準レシーバ）
    while let Ok(event) = MenuEvent::receiver().try_recv() {
        if let Some(cmd) = resolve_menu_command(event.id().0.as_str()) {
            return Some(cmd);
        }
    }

    // 3. トレイアイコン左クリックイベントの確認（内部キュー）
    if let Ok(mut queue) = TRAY_ICON_QUEUE.lock() {
        while let Some(event) = queue.pop_front() {
            if is_tray_left_click_release(&event) {
                return Some(TrayCommand::ToggleWindow);
            }
        }
    }

    // 4. トレイアイコン左クリックイベントの確認（標準レシーバ）
    while let Ok(event) = TrayIconEvent::receiver().try_recv() {
        if is_tray_left_click_release(&event) {
            return Some(TrayCommand::ToggleWindow);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_menu_commands() {
        assert_eq!(
            resolve_menu_command(MENU_ID_SETTINGS),
            Some(TrayCommand::OpenSettings)
        );
        assert_eq!(
            resolve_menu_command(MENU_ID_OPEN_COMMANDS),
            Some(TrayCommand::OpenCommandsFolder)
        );
        assert_eq!(resolve_menu_command(MENU_ID_EXIT), Some(TrayCommand::Exit));
        assert_eq!(resolve_menu_command("invalid_id"), None);
    }

    #[test]
    fn test_generate_default_icon_rgba_and_build_icon() {
        let rgba = generate_default_icon_rgba(DEFAULT_ICON_SIZE, DEFAULT_ICON_SIZE);
        assert_eq!(
            rgba.len(),
            (DEFAULT_ICON_SIZE * DEFAULT_ICON_SIZE * 4) as usize
        );

        // 左上角 (0, 0) はボーダー色 (#4ec9b0)
        assert_eq!(&rgba[0..4], &[0x4e, 0xc9, 0xb0, 0xff]);

        // 内側背景 (4, 4) はダークチャコール (#1e1e1e)
        let idx = ((4 * DEFAULT_ICON_SIZE + 4) * 4) as usize;
        assert_eq!(&rgba[idx..idx + 4], &[0x1e, 0x1e, 0x1e, 0xff]);

        // Icon 構造体への変換が成功すること
        let icon = build_default_tray_icon();
        assert!(icon.is_ok());
    }
}
