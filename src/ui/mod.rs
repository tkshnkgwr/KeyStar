//! `egui` / `eframe` による 1 行極小入力ウィンドウおよびキーボード操作モジュール。
//!
//! 名作「CraftLaunch 2」の佇まい（タイトルバーなし・常時最前面・極小1行入力枠・動的候補ドロップダウン）と、
//! キーボード完結の操作体系（`Enter`, `Esc`, `Tab`/`Shift+Tab`, `Ctrl+n`/`Ctrl+p`, `Up`/`Down`）を提供します。

pub mod executor;
pub mod state;

pub use executor::{execute_builtin_command, launch_target};
pub use state::{
    BASE_WINDOW_HEIGHT, BG_COLOR, CALC_ACCENT_COLOR, CANDIDATE_ROW_HEIGHT, DEFAULT_MAX_CANDIDATES,
    KeyCommand, LauncherAction, LauncherState, SELECTED_BG_COLOR, SHORTCUT_TEXT_COLOR, TEXT_COLOR,
    WINDOW_WIDTH,
};

use crate::clipboard::{ClipboardProvider, SystemClipboard};
use crate::command::BuiltinCommand;
use crate::hotkey::{HotKeyService, install_hotkey_wakeup_hook};
use crate::indexer::ShortcutIndexer;
use crate::search::CandidateKind;
use crate::tray::{TrayCommand, TrayService, install_tray_wakeup_hook};
use eframe::egui;

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
                    if let Some(first_dir) = self.indexer.directories().first() {
                        let _ = open::that_detached(first_dir);
                    }
                    self.dismiss_window(ctx);
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

    /// [`LauncherAction`] を実行し、ウィンドウ非表示などの副次効果を適用します。
    pub fn execute_action(&mut self, ctx: &egui::Context, action: LauncherAction) {
        match action {
            LauncherAction::None => {}
            LauncherAction::ExecuteShortcut { path, args } => {
                launch_target(&path, &args);
                self.dismiss_window(ctx);
            }
            LauncherAction::CopyCalcResult(text) => {
                let _ = self.clipboard.copy_text(&text);
                ctx.copy_text(text);
                self.dismiss_window(ctx);
            }
            LauncherAction::ExecuteBuiltin(cmd) => {
                self.execute_builtin(ctx, cmd);
            }
            LauncherAction::Dismiss => {
                self.dismiss_window(ctx);
            }
        }
    }

    /// ビルトイン（スラッシュ）コマンドを実行します。
    fn execute_builtin(&mut self, ctx: &egui::Context, cmd: BuiltinCommand) {
        let is_help = cmd == BuiltinCommand::Help;
        execute_builtin_command(
            ctx,
            cmd,
            &mut self.indexer,
            &mut self.state,
            self.clipboard.as_mut(),
        );

        if is_help {
            self.show_window(ctx);
        } else if cmd != BuiltinCommand::Exit && cmd != BuiltinCommand::Restart {
            self.dismiss_window(ctx);
        }
    }

    /// バックグラウンド待機状態からランチャーウィンドウを最前面に復帰・アクティブ化します。
    fn show_window(&mut self, ctx: &egui::Context) {
        if self.indexer.needs_rescan() {
            self.indexer.scan();
            self.state.refresh_candidates(self.indexer.items());
        }
        self.state.set_window_visible(true);
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
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
