//! `egui` / `eframe` による 1 行極小入力ウィンドウおよびキーボード操作モジュール。
//!
//! 名作「CraftLaunch 2」の佇まい（タイトルバーなし・常時最前面・極小1行入力枠・動的候補ドロップダウン）と、
//! キーボード完結の操作体系（`Enter`, `Esc`, `Tab`/`Shift+Tab`, `Ctrl+n`/`Ctrl+p`, `Up`/`Down`）を提供します。

use crate::calc::evaluate;
use crate::clipboard::{ClipboardProvider, SystemClipboard};
use crate::hotkey::{HotKeyService, install_hotkey_wakeup_hook};
use crate::indexer::{ShortcutIndexer, ShortcutItem};
use crate::search::{Candidate, CandidateKind, filter_candidates};
use crate::tray::{TrayCommand, TrayService, install_tray_wakeup_hook};
use eframe::egui;
use std::path::PathBuf;

/// ランチャーウィンドウの標準横幅（ピクセル）。
pub const WINDOW_WIDTH: f32 = 500.0;

/// 入力欄のみ表示時の基本ウィンドウ高さ（ピクセル）。
pub const BASE_WINDOW_HEIGHT: f32 = 42.0;

/// 候補ドロップダウン1行あたりの高さ（ピクセル）。
pub const CANDIDATE_ROW_HEIGHT: f32 = 26.0;

/// 候補ドロップダウンに表示する標準最大件数。
pub const DEFAULT_MAX_CANDIDATES: usize = 8;

/// 背景色（ダークチャコール `#1e1e1e`）。
pub const BG_COLOR: egui::Color32 = egui::Color32::from_rgb(0x1e, 0x1e, 0x1e);

/// メインテキスト色（高コントラスト白 `#ffffff`）。
pub const TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(0xff, 0xff, 0xff);

/// 電卓計算結果の強調アクセントカラー（ティールグリーン `#4ec9b0`）。
pub const CALC_ACCENT_COLOR: egui::Color32 = egui::Color32::from_rgb(0x4e, 0xc9, 0xb0);

/// 選択中（アクティブ）候補行のハイライト背景色（アクセントブルー `#264f78`）。
pub const SELECTED_BG_COLOR: egui::Color32 = egui::Color32::from_rgb(0x26, 0x4f, 0x78);

/// 非選択ショートカット候補行のテキスト色（ライトグレー `#d4d4d4`）。
pub const SHORTCUT_TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(0xd4, 0xd4, 0xd4);

/// ランチャー画面で受け付けるキーボード操作コマンド。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCommand {
    /// 次の候補を選択（`Tab`, `Ctrl+n`, `Down`）
    SelectNext,
    /// 前の候補を選択（`Shift+Tab`, `Ctrl+p`, `Up`）
    SelectPrev,
    /// 選択中の候補を実行・確定（`Enter`）
    Confirm,
    /// キャンセルしてウィンドウを非表示化（`Esc`）
    Cancel,
}

/// キー操作や候補選択の結果として発火する外部アクション。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LauncherAction {
    /// アクションなし（入力継続・選択移動のみ）
    None,
    /// ショートカット・実行ファイルを非同期起動してウィンドウを非表示にする
    ExecuteShortcut(PathBuf),
    /// 電卓の計算結果テキストをクリップボードへコピーしてウィンドウを非表示にする
    CopyCalcResult(String),
    /// 何も実行せずウィンドウを非表示にする
    Dismiss,
}

/// ランチャーのUI状態（入力クエリ・候補リスト・選択インデックス・表示状態）を管理する構造体。
///
/// GUI描画フレームワーク（`egui`）から独立して状態遷移とキーバインドを検証可能にします。
#[derive(Debug, Clone)]
pub struct LauncherState {
    /// 現在の1行入力テキスト
    query: String,
    /// 現在の絞り込み候補リスト
    candidates: Vec<Candidate>,
    /// 現在選択されている候補のインデックス（0始まり）
    selected_index: usize,
    /// 表示する最大候補件数
    max_candidates: usize,
    /// クエリが空の際にも明示的にドロップダウンを展開しているかどうか
    force_dropdown_open: bool,
    /// ランチャーウィンドウが現在表示状態かどうか（ホットキートグル制御用）
    window_visible: bool,
    /// 入力欄へのフォーカス強制が要求されているかどうか
    focus_requested: bool,
}

impl Default for LauncherState {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_CANDIDATES)
    }
}

impl LauncherState {
    /// 指定された最大候補件数で新しい [`LauncherState`] を生成します。
    ///
    /// # Panics
    ///
    /// パニックは発生しません。
    ///
    /// # Examples
    ///
    /// ```
    /// use KeyStar::ui::LauncherState;
    ///
    /// let state = LauncherState::new(8);
    /// assert_eq!(state.query(), "");
    /// assert!(!state.is_dropdown_visible());
    /// assert!(state.is_window_visible());
    /// ```
    #[must_use]
    pub fn new(max_candidates: usize) -> Self {
        Self {
            query: String::new(),
            candidates: Vec::new(),
            selected_index: 0,
            max_candidates: max_candidates.max(1),
            force_dropdown_open: false,
            window_visible: true,
            focus_requested: true,
        }
    }

    /// ランチャーウィンドウが現在表示状態であるかを返します。
    #[must_use]
    pub fn is_window_visible(&self) -> bool {
        self.window_visible
    }

    /// ランチャーウィンドウの表示状態を更新します。
    ///
    /// `visible == true` に遷移する際は、入力テキストボックスへのフォーカス強制フラグも自動的に有効化されます。
    pub fn set_window_visible(&mut self, visible: bool) {
        self.window_visible = visible;
        self.focus_requested = visible;
    }

    /// 入力欄へのフォーカス強制が要求されているかを返します。
    #[must_use]
    pub fn is_focus_requested(&self) -> bool {
        self.focus_requested
    }

    /// フォーカス強制要求フラグを取得し、同時にフラグを消費（`false` にリセット）します。
    pub fn take_focus_request(&mut self) -> bool {
        let req = self.focus_requested;
        self.focus_requested = false;
        req
    }

    /// グローバルホットキー（`Alt + Space`）やトレイクリックによるウィンドウ表示 / 非表示をトグルします。
    ///
    /// トグル後の新しい表示状態（`true` = 表示、`false` = 非表示）を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use KeyStar::ui::LauncherState;
    ///
    /// let mut state = LauncherState::new(8);
    /// assert!(state.is_window_visible());
    ///
    /// // 非表示へトグル
    /// assert!(!state.toggle_visibility(&[]));
    /// assert!(!state.is_window_visible());
    ///
    /// // 表示へトグル（フォーカス要求も復活）
    /// assert!(state.toggle_visibility(&[]));
    /// assert!(state.is_window_visible());
    /// assert!(state.is_focus_requested());
    /// ```
    pub fn toggle_visibility(&mut self, shortcuts: &[ShortcutItem]) -> bool {
        self.reset(shortcuts);
        let next_visible = !self.window_visible;
        self.set_window_visible(next_visible);
        next_visible
    }

    /// 現在の入力文字列を返します。
    #[must_use]
    pub fn query(&self) -> &str {
        &self.query
    }

    /// 現在保持している候補リスト全体を返します。
    #[must_use]
    pub fn candidates(&self) -> &[Candidate] {
        &self.candidates
    }

    /// 現在選択されている候補のインデックスを返します。
    #[must_use]
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// 現在選択されている候補アイテムへの参照を返します。
    ///
    /// ドロップダウンが非表示の場合や候補が0件の場合は `None` を返します。
    #[must_use]
    pub fn selected_candidate(&self) -> Option<&Candidate> {
        if !self.is_dropdown_visible() {
            return None;
        }
        self.candidates.get(self.selected_index)
    }

    /// 候補ドロップダウンパネルを展開表示すべきかどうかを判定します。
    ///
    /// 入力欄に1文字以上入力があり候補が存在する場合、またはキー操作（`Down`/`Tab`等）で
    /// 明示的に展開された場合に `true` を返します。
    #[must_use]
    pub fn is_dropdown_visible(&self) -> bool {
        !self.candidates.is_empty() && (!self.query.trim().is_empty() || self.force_dropdown_open)
    }

    /// 画面上に描画すべき候補スライスを返します。
    ///
    /// ドロップダウン非表示時は空スライスを返します。
    #[must_use]
    pub fn visible_candidates(&self) -> &[Candidate] {
        if self.is_dropdown_visible() {
            &self.candidates
        } else {
            &[]
        }
    }

    /// 現在の展開状態に応じたウィンドウの目標高さ（ピクセル）を計算します。
    ///
    /// # Examples
    ///
    /// ```
    /// use KeyStar::ui::{BASE_WINDOW_HEIGHT, LauncherState};
    ///
    /// let state = LauncherState::default();
    /// assert_eq!(state.window_height(), BASE_WINDOW_HEIGHT);
    /// ```
    #[must_use]
    pub fn window_height(&self) -> f32 {
        let visible_count = self.visible_candidates().len();
        if visible_count == 0 {
            BASE_WINDOW_HEIGHT
        } else {
            BASE_WINDOW_HEIGHT + (visible_count as f32 * CANDIDATE_ROW_HEIGHT) + 8.0
        }
    }

    /// 入力クエリを更新し、電卓評価および候補フィルタリングを再実行します。
    ///
    /// クエリが変更された場合、選択インデックスは先頭（`0`）にリセットされます。
    ///
    /// # Panics
    ///
    /// パニックは発生しません。
    pub fn update_query(&mut self, new_query: impl Into<String>, shortcuts: &[ShortcutItem]) {
        let new_query = new_query.into();
        self.query = new_query;
        self.selected_index = 0;
        self.force_dropdown_open = false;
        self.refresh_candidates(shortcuts);
    }

    /// 現在のクエリのまま候補リストを再構築します。
    pub fn refresh_candidates(&mut self, shortcuts: &[ShortcutItem]) {
        let calc_res = evaluate(&self.query);
        self.candidates = filter_candidates(&self.query, shortcuts, calc_res, self.max_candidates);
        if self.candidates.is_empty() {
            self.selected_index = 0;
        } else if self.selected_index >= self.candidates.len() {
            self.selected_index = self.candidates.len() - 1;
        }
    }

    /// 選択インデックスを指定位置に設定します（マウスクリック等での選択用）。
    pub fn select_index(&mut self, index: usize) {
        if index < self.candidates.len() {
            self.selected_index = index;
        }
    }

    /// 入力状態と選択状態を初期状態にリセットします。
    pub fn reset(&mut self, shortcuts: &[ShortcutItem]) {
        self.query.clear();
        self.selected_index = 0;
        self.force_dropdown_open = false;
        self.refresh_candidates(shortcuts);
    }

    /// キーボード操作コマンドを処理し、対応する [`LauncherAction`] を返します。
    ///
    /// - [`KeyCommand::SelectNext`] — 次の候補へ循環移動（末尾の次は先頭 `0`）
    /// - [`KeyCommand::SelectPrev`] — 前の候補へ循環移動（先頭 `0` の前は末尾）
    /// - [`KeyCommand::Confirm`] — 選択中のショートカット起動または計算結果コピーを返す
    /// - [`KeyCommand::Cancel`] — 入力をクリアして [`LauncherAction::Dismiss`] を返す
    ///
    /// # Panics
    ///
    /// 候補が空の場合でもゼロ除算や境界外アクセスを起こさず安全に処理され、パニックは発生しません。
    pub fn handle_key_command(
        &mut self,
        cmd: KeyCommand,
        shortcuts: &[ShortcutItem],
    ) -> LauncherAction {
        match cmd {
            KeyCommand::SelectNext => {
                if self.query.trim().is_empty() && !self.force_dropdown_open {
                    self.force_dropdown_open = true;
                    self.refresh_candidates(shortcuts);
                    self.selected_index = 0;
                } else if !self.candidates.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.candidates.len();
                }
                LauncherAction::None
            }
            KeyCommand::SelectPrev => {
                if self.query.trim().is_empty() && !self.force_dropdown_open {
                    self.force_dropdown_open = true;
                    self.refresh_candidates(shortcuts);
                    self.selected_index = self.candidates.len().saturating_sub(1);
                } else if !self.candidates.is_empty() {
                    self.selected_index = if self.selected_index == 0 {
                        self.candidates.len() - 1
                    } else {
                        self.selected_index - 1
                    };
                }
                LauncherAction::None
            }
            KeyCommand::Confirm => {
                let action = match self.selected_candidate() {
                    Some(cand) => match &cand.kind {
                        CandidateKind::Calc(calc) => {
                            LauncherAction::CopyCalcResult(calc.copy_text.clone())
                        }
                        CandidateKind::Shortcut(item) => {
                            LauncherAction::ExecuteShortcut(item.path.clone())
                        }
                    },
                    None => LauncherAction::Dismiss,
                };
                self.reset(shortcuts);
                action
            }
            KeyCommand::Cancel => {
                self.reset(shortcuts);
                LauncherAction::Dismiss
            }
        }
    }
}

/// 1行極小ウィンドウ用の `eframe::NativeOptions` を構築します。
///
/// タイトルバーなし（Borderless）、常時最前面（Always on Top）、リサイズ不可、
/// 画面中央初期配置の設定を適用します。
#[must_use]
pub fn build_native_options() -> eframe::NativeOptions {
    let viewport = egui::ViewportBuilder::default()
        .with_title("KeyStar")
        .with_inner_size([WINDOW_WIDTH, BASE_WINDOW_HEIGHT])
        .with_decorations(false)
        .with_always_on_top()
        .with_resizable(false)
        .with_taskbar(false);

    eframe::NativeOptions {
        viewport,
        centered: true,
        ..eframe::NativeOptions::default()
    }
}

/// `eframe::App` を実装するメインランチャーGUIアプリケーション。
pub struct LauncherApp {
    /// UI状態機械
    state: LauncherState,
    /// ショートカット監視インデクサ
    indexer: ShortcutIndexer,
    /// 前フレームで設定したウィンドウ高さ
    last_window_height: f32,
    /// 非表示化時にプロセスを終了せずウィンドウのみ隠すか（Phase 3 常駐連携用）
    hide_only_on_dismiss: bool,
    /// OS グローバルホットキー監視サービス（`Alt + Space`）
    hotkey_service: Option<HotKeyService>,
    /// OS システムトレイ常駐サービス
    tray_service: Option<TrayService>,
    /// クリップボード書き込みプロバイダ（`arboard` OS連携 / テスト用モック）
    clipboard: Box<dyn ClipboardProvider>,
}

impl LauncherApp {
    /// 指定されたインデクサを用いて新しい [`LauncherApp`] を生成します。
    #[must_use]
    pub fn new(mut indexer: ShortcutIndexer, hide_only_on_dismiss: bool) -> Self {
        indexer.ensure_directories_exist();
        indexer.scan();

        let mut state = LauncherState::default();
        state.refresh_candidates(indexer.items());

        Self {
            state,
            indexer,
            last_window_height: BASE_WINDOW_HEIGHT,
            hide_only_on_dismiss,
            hotkey_service: None,
            tray_service: None,
            clipboard: Box::new(SystemClipboard::new()),
        }
    }

    /// グローバルホットキー（`Alt + Space`）およびシステムトレイ常駐を有効化した
    /// 常駐モードの [`LauncherApp`] を生成します。
    #[must_use]
    pub fn new_resident(indexer: ShortcutIndexer, ctx: &egui::Context) -> Self {
        install_hotkey_wakeup_hook(ctx.clone());
        install_tray_wakeup_hook(ctx.clone());

        let mut app = Self::new(indexer, true);
        app.hotkey_service = HotKeyService::new_default().ok();
        app.tray_service = TrayService::new_default().ok();
        app
    }

    /// カスタムの [`ClipboardProvider`] を差し替えます（単体・統合テスト用）。
    #[must_use]
    pub fn with_clipboard(mut self, clipboard: Box<dyn ClipboardProvider>) -> Self {
        self.clipboard = clipboard;
        self
    }

    /// 内部の [`LauncherState`] への不変参照を返します。
    #[must_use]
    pub fn state(&self) -> &LauncherState {
        &self.state
    }

    /// 内部の [`LauncherState`] への可変参照を返します。
    pub fn state_mut(&mut self) -> &mut LauncherState {
        &mut self.state
    }

    /// 直近にクリップボードプロバイダへコピーされた文字列を返します。
    #[must_use]
    pub fn last_copied_text(&self) -> Option<&str> {
        self.clipboard.last_copied_text()
    }

    /// ダークテーマおよび余白を極小化したカスタムビジュアルスタイルを適用します。
    pub fn configure_style(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = BG_COLOR;
        visuals.window_fill = BG_COLOR;
        visuals.extreme_bg_color = egui::Color32::from_rgb(0x25, 0x25, 0x26);
        visuals.override_text_color = Some(TEXT_COLOR);
        ctx.set_visuals(visuals);

        let mut style = (*ctx.global_style()).clone();
        style.spacing.item_spacing = egui::vec2(6.0, 4.0);
        style.spacing.window_margin = egui::Margin::same(6);
        ctx.set_global_style(style);
    }

    /// `egui::InputState` からキーボードイベントを読み取り、[`KeyCommand`] に変換します。
    fn poll_key_command(ui: &mut egui::Ui) -> Option<KeyCommand> {
        ui.input_mut(|i| {
            // Shift+Tab は Tab 単体より先に判定する（Modifiers::matches_logically の仕様対応）
            if i.consume_key(egui::Modifiers::SHIFT, egui::Key::Tab)
                || i.consume_key(egui::Modifiers::CTRL, egui::Key::P)
                || i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)
            {
                Some(KeyCommand::SelectPrev)
            } else if i.consume_key(egui::Modifiers::NONE, egui::Key::Tab)
                || i.consume_key(egui::Modifiers::CTRL, egui::Key::N)
                || i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)
            {
                Some(KeyCommand::SelectNext)
            } else if i.consume_key(egui::Modifiers::NONE, egui::Key::Enter) {
                Some(KeyCommand::Confirm)
            } else if i.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                Some(KeyCommand::Cancel)
            } else {
                None
            }
        })
    }

    /// グローバルホットキーおよびシステムトレイからの OS イベントをポーリングして処理します。
    fn poll_os_events(&mut self, ctx: &egui::Context) {
        // 1. グローバルホットキー (Alt + Space) による表示 / 非表示トグル
        let hotkey_pressed = self
            .hotkey_service
            .as_ref()
            .is_some_and(HotKeyService::poll_pressed);
        if hotkey_pressed {
            let now_visible = self.state.toggle_visibility(self.indexer.items());
            if now_visible {
                self.show_window(ctx);
            } else {
                self.dismiss_window(ctx);
            }
        }

        // 2. システムトレイのメニュー選択およびクリックイベント処理
        while let Some(cmd) = self
            .tray_service
            .as_ref()
            .and_then(TrayService::poll_command)
        {
            match cmd {
                TrayCommand::ToggleWindow => {
                    let now_visible = self.state.toggle_visibility(self.indexer.items());
                    if now_visible {
                        self.show_window(ctx);
                    } else {
                        self.dismiss_window(ctx);
                    }
                }
                TrayCommand::OpenCommandsFolder => {
                    if let Some(primary_dir) = self.indexer.directories().first() {
                        let _ = open::that_detached(primary_dir);
                    }
                }
                TrayCommand::OpenSettings => {
                    if let Some(settings_dir) = self
                        .indexer
                        .directories()
                        .get(1)
                        .or_else(|| self.indexer.directories().first())
                    {
                        let _ = open::that_detached(settings_dir);
                    }
                    self.show_window(ctx);
                }
                TrayCommand::Exit => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }

    /// 確定された [`LauncherAction`] をOS/ウィンドウシステムに対して実行します。
    fn execute_action(&mut self, ctx: &egui::Context, action: LauncherAction) {
        match action {
            LauncherAction::None => {}
            LauncherAction::ExecuteShortcut(path) => {
                let _ = open::that_detached(&path);
                self.dismiss_window(ctx);
            }
            LauncherAction::CopyCalcResult(text) => {
                // arboard による OS クリップボードへの同期的書き込み + egui フォールバック
                let _ = self.clipboard.copy_text(&text);
                ctx.copy_text(text);
                self.dismiss_window(ctx);
            }
            LauncherAction::Dismiss => {
                self.dismiss_window(ctx);
            }
        }
    }

    /// バックグラウンド待機状態からランチャーウィンドウを最前面に復帰・アクティブ化します。
    fn show_window(&mut self, ctx: &egui::Context) {
        if self.indexer.needs_rescan() {
            self.indexer.scan();
        }
        self.state.reset(self.indexer.items());
        self.state.set_window_visible(true);

        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
            egui::WindowLevel::AlwaysOnTop,
        ));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }

    /// ウィンドウを非表示化（または終了）します。
    fn dismiss_window(&mut self, ctx: &egui::Context) {
        self.state.set_window_visible(false);
        if self.hide_only_on_dismiss {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

impl eframe::App for LauncherApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // 0. グローバルホットキー・システムトレイ等の OS 常駐イベント処理
        self.poll_os_events(&ctx);

        // 監視フォルダの更新があれば軽量再スキャン
        if self.indexer.needs_rescan() {
            self.indexer.scan();
            self.state.refresh_candidates(self.indexer.items());
        }

        // 1. キーボードコマンドの先読み消費（TextEditにTabや矢印が奪われるのを防ぐ）
        if let Some(cmd) = Self::poll_key_command(ui) {
            let action = self.state.handle_key_command(cmd, self.indexer.items());
            if action != LauncherAction::None {
                self.execute_action(&ctx, action);
                return;
            }
        }

        let panel_frame = egui::Frame::central_panel(&ctx.global_style())
            .fill(BG_COLOR)
            .inner_margin(egui::Margin::symmetric(8, 6))
            .stroke(egui::Stroke::new(
                1.0,
                egui::Color32::from_rgb(0x3c, 0x3c, 0x3c),
            ));

        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ui, |ui| {
                // 2. 1行入力テキストボックスの描画
                let mut current_query = self.state.query().to_string();
                ui.horizontal(|ui| {
                    ui.colored_label(
                        CALC_ACCENT_COLOR,
                        egui::RichText::new(">").monospace().strong().size(16.0),
                    );

                    let text_edit = egui::TextEdit::singleline(&mut current_query)
                        .hint_text("ここに入力... (ショートカット名 または 計算式)")
                        .font(egui::FontId::monospace(15.0))
                        .desired_width(f32::INFINITY)
                        .lock_focus(true)
                        .frame(egui::Frame::NONE);

                    let response = ui.add(text_edit);
                    // ホットキー復帰時または非フォーカス時は1行テキストボックスへフォーカス強制
                    if self.state.take_focus_request() || !response.has_focus() {
                        response.request_focus();
                    }

                    if response.changed() {
                        self.state.update_query(current_query, self.indexer.items());
                    }
                });

                // 3. 候補ドロップダウンパネルの動的展開描画
                let visible_candidates = self.state.visible_candidates().to_vec();
                if !visible_candidates.is_empty() {
                    ui.add_space(4.0);
                    ui.separator();

                    let selected_idx = self.state.selected_index();
                    let mut clicked_action = LauncherAction::None;

                    for (idx, candidate) in visible_candidates.iter().enumerate() {
                        let is_selected = idx == selected_idx;
                        let is_calc = matches!(candidate.kind, CandidateKind::Calc(_));

                        let row_bg = if is_selected {
                            SELECTED_BG_COLOR
                        } else {
                            egui::Color32::TRANSPARENT
                        };

                        let text_color = if is_calc {
                            CALC_ACCENT_COLOR
                        } else if is_selected {
                            TEXT_COLOR
                        } else {
                            SHORTCUT_TEXT_COLOR
                        };

                        let prefix = if is_calc { " " } else { " * " };
                        let display_line = format!("{prefix}{}", candidate.label);

                        let row_frame = egui::Frame::NONE
                            .fill(row_bg)
                            .inner_margin(egui::Margin::symmetric(6, 3))
                            .corner_radius(egui::CornerRadius::same(3));

                        let row_response = row_frame
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(display_line)
                                            .color(text_color)
                                            .monospace()
                                            .size(14.0),
                                    )
                                    .selectable(false),
                                )
                            })
                            .inner;

                        if row_response.clicked() {
                            self.state.select_index(idx);
                            clicked_action = self
                                .state
                                .handle_key_command(KeyCommand::Confirm, self.indexer.items());
                        }
                    }

                    if clicked_action != LauncherAction::None {
                        self.execute_action(&ctx, clicked_action);
                    }
                }
            });

        // 4. 候補数に応じたウィンドウ高さの動的調整
        let target_height = self.state.window_height();
        if (target_height - self.last_window_height).abs() > 0.5 {
            self.last_window_height = target_height;
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                WINDOW_WIDTH,
                target_height,
            )));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_shortcuts() -> Vec<ShortcutItem> {
        vec![
            ShortcutItem {
                name: "Chrome".to_string(),
                path: PathBuf::from("C:\\commands\\Chrome.lnk"),
                extension: "lnk".to_string(),
            },
            ShortcutItem {
                name: "Code".to_string(),
                path: PathBuf::from("C:\\commands\\Code.lnk"),
                extension: "lnk".to_string(),
            },
            ShortcutItem {
                name: "Obsidian".to_string(),
                path: PathBuf::from("C:\\commands\\Obsidian.lnk"),
                extension: "lnk".to_string(),
            },
        ]
    }

    #[test]
    fn test_initial_state_dropdown_hidden() {
        let shortcuts = sample_shortcuts();
        let mut state = LauncherState::new(8);
        state.refresh_candidates(&shortcuts);

        assert_eq!(state.query(), "");
        assert!(!state.is_dropdown_visible());
        assert!(state.visible_candidates().is_empty());
        assert_eq!(state.window_height(), BASE_WINDOW_HEIGHT);
    }

    #[test]
    fn test_query_expands_dropdown_and_height() {
        let shortcuts = sample_shortcuts();
        let mut state = LauncherState::new(8);
        state.update_query("c", &shortcuts);

        assert!(state.is_dropdown_visible());
        assert_eq!(state.visible_candidates().len(), 2); // Chrome, Code
        assert!(state.window_height() > BASE_WINDOW_HEIGHT);
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_navigation_cycle_next_and_prev() {
        let shortcuts = sample_shortcuts();
        let mut state = LauncherState::new(8);
        state.update_query("c", &shortcuts); // 2 candidates: 0=Chrome, 1=Code

        assert_eq!(state.selected_index(), 0);

        // Next -> 1
        assert_eq!(
            state.handle_key_command(KeyCommand::SelectNext, &shortcuts),
            LauncherAction::None
        );
        assert_eq!(state.selected_index(), 1);

        // Next -> wraps to 0
        assert_eq!(
            state.handle_key_command(KeyCommand::SelectNext, &shortcuts),
            LauncherAction::None
        );
        assert_eq!(state.selected_index(), 0);

        // Prev -> wraps to 1
        assert_eq!(
            state.handle_key_command(KeyCommand::SelectPrev, &shortcuts),
            LauncherAction::None
        );
        assert_eq!(state.selected_index(), 1);
    }

    #[test]
    fn test_confirm_shortcut_and_calc() {
        let shortcuts = sample_shortcuts();
        let mut state = LauncherState::new(8);

        // ショートカット確定
        state.update_query("obs", &shortcuts);
        let action = state.handle_key_command(KeyCommand::Confirm, &shortcuts);
        assert_eq!(
            action,
            LauncherAction::ExecuteShortcut(PathBuf::from("C:\\commands\\Obsidian.lnk"))
        );
        assert_eq!(state.query(), ""); // 確定後はリセット

        // 電卓結果確定
        state.update_query("1280 * 1.1", &shortcuts);
        let calc_action = state.handle_key_command(KeyCommand::Confirm, &shortcuts);
        assert_eq!(
            calc_action,
            LauncherAction::CopyCalcResult("1408".to_string())
        );
    }

    #[test]
    fn test_cancel_resets_and_dismisses() {
        let shortcuts = sample_shortcuts();
        let mut state = LauncherState::new(8);
        state.update_query("chrome", &shortcuts);

        let action = state.handle_key_command(KeyCommand::Cancel, &shortcuts);
        assert_eq!(action, LauncherAction::Dismiss);
        assert_eq!(state.query(), "");
        assert!(!state.is_dropdown_visible());
    }

    #[test]
    fn test_toggle_visibility_and_focus_request() {
        let shortcuts = sample_shortcuts();
        let mut state = LauncherState::new(8);
        assert!(state.is_window_visible());
        assert!(state.take_focus_request());
        assert!(!state.is_focus_requested());

        // 入力途中にホットキーで非表示へトグル ➔ 入力はリセットされ非表示になる
        state.update_query("obs", &shortcuts);
        assert!(!state.toggle_visibility(&shortcuts));
        assert!(!state.is_window_visible());
        assert!(!state.is_focus_requested());
        assert_eq!(state.query(), "");

        // 再度ホットキーで表示へトグル ➔ 表示状態になりフォーカス強制フラグが立つ
        assert!(state.toggle_visibility(&shortcuts));
        assert!(state.is_window_visible());
        assert!(state.is_focus_requested());
    }
}
