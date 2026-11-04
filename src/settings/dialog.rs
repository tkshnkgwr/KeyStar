//! GUI 環境設定ダイアログ状態および `egui` ビューポート描画モジュール。
//!
//! 独立した設定ウィンドウの表示・非表示、ユーザー入力操作、キー入力キャプチャ、
//! および設定値の適用・キャンセル等のアクション生成を提供します。

use crate::hotkey::HotKeyBinding;
use crate::indexer::TARGET_EXTENSIONS;
use crate::ui::state::{BG_COLOR, CALC_ACCENT_COLOR, TEXT_COLOR};
use eframe::egui;

use super::config::{AppConfig, CalcCopyFormat};
use super::os::{
    collapse_env_path, collapse_env_path_str, egui_key_to_code, format_hotkey_binding,
    pick_folder_dialog,
};
use super::{
    HOTKEY_PRESETS, MAX_FONT_SIZE, MAX_MAX_CANDIDATES, MAX_WINDOW_WIDTH, MIN_FONT_SIZE,
    MIN_MAX_CANDIDATES, MIN_WINDOW_WIDTH, SETTINGS_WINDOW_HEIGHT, SETTINGS_WINDOW_WIDTH,
};

/// 設定ダイアログでのユーザー操作結果を表すアクション。
#[derive(Debug, Clone, PartialEq)]
pub enum SettingsDialogAction {
    /// アクションなし（ダイアログ操作継続）
    None,
    /// 設定を保存・適用してダイアログを閉じる
    SaveAndApply(AppConfig),
    /// 変更を破棄してダイアログを閉じる
    Cancel,
}

/// GUI 環境設定ダイアログの状態機械。
#[derive(Debug, Clone)]
pub struct SettingsDialog {
    /// ダイアログが現在開いているか
    is_open: bool,
    /// 編集中の設定ドラフト
    draft: AppConfig,
    /// 監視ディレクトリ一覧で現在選択中のインデックス
    selected_dir_index: Option<usize>,
    /// 手動追加用のディレクトリパス入力欄
    new_dir_input: String,
    /// ホットキーのキー押下キャプチャ待機中か
    capturing_hotkey: bool,
}

impl Default for SettingsDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsDialog {
    /// 非表示状態の新しい [`SettingsDialog`] を生成します。
    ///
    /// # Panics
    ///
    /// パニックは発生しません。
    ///
    /// # Examples
    ///
    /// ```
    /// use KeyStar::settings::SettingsDialog;
    ///
    /// let dialog = SettingsDialog::new();
    /// assert!(!dialog.is_open());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            is_open: false,
            draft: AppConfig::default(),
            selected_dir_index: None,
            new_dir_input: String::new(),
            capturing_hotkey: false,
        }
    }

    /// 現在の設定 [`AppConfig`] をドラフトに読み込んで設定ダイアログを開きます。
    ///
    /// # Examples
    ///
    /// ```
    /// use KeyStar::settings::{AppConfig, SettingsDialog};
    ///
    /// let mut dialog = SettingsDialog::new();
    /// dialog.open(AppConfig::default());
    /// assert!(dialog.is_open());
    /// ```
    pub fn open(&mut self, current_config: AppConfig) {
        self.draft = current_config.sanitized();
        self.selected_dir_index = if self.draft.watch_directories.is_empty() {
            None
        } else {
            Some(0)
        };
        self.new_dir_input.clear();
        self.capturing_hotkey = false;
        self.is_open = true;
    }

    /// 設定ダイアログを閉じます。
    pub fn close(&mut self) {
        self.is_open = false;
        self.capturing_hotkey = false;
    }

    /// 設定ダイアログが開いているかを返します。
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// 編集中のドラフト [`AppConfig`] への不変参照を返します。
    #[must_use]
    pub fn draft(&self) -> &AppConfig {
        &self.draft
    }

    /// 編集中のドラフト [`AppConfig`] への可変参照を返します。
    pub fn draft_mut(&mut self) -> &mut AppConfig {
        &mut self.draft
    }

    /// 編集中のドラフトを初期値（[`AppConfig::default()`]）にリセットします。
    pub fn reset_to_default(&mut self) {
        self.draft = AppConfig::default();
        self.selected_dir_index = Some(0);
        self.new_dir_input.clear();
        self.capturing_hotkey = false;
    }

    /// 監視ディレクトリをドラフトに追加します。
    ///
    /// ユーザープロファイル配下の絶対パスが渡された場合は、自動的に `%USERPROFILE%` 表記へ変換して追加します。
    pub fn add_watch_directory(&mut self, raw_dir: &str) {
        let collapsed = collapse_env_path_str(raw_dir);
        if collapsed.is_empty() {
            return;
        }
        if !self
            .draft
            .watch_directories
            .iter()
            .any(|d| d.eq_ignore_ascii_case(&collapsed))
        {
            self.draft.watch_directories.push(collapsed);
            self.selected_dir_index = Some(self.draft.watch_directories.len() - 1);
        }
    }

    /// 現在選択されている監視ディレクトリをドラフトから削除します。
    pub fn remove_selected_directory(&mut self) {
        if let Some(idx) = self.selected_dir_index
            && idx < self.draft.watch_directories.len()
        {
            self.draft.watch_directories.remove(idx);
            if self.draft.watch_directories.is_empty() {
                self.selected_dir_index = None;
            } else {
                self.selected_dir_index =
                    Some(idx.min(self.draft.watch_directories.len().saturating_sub(1)));
            }
        }
    }

    /// 指定された拡張子のスキャン対象 ON / OFF を切り替えます。
    pub fn set_extension_enabled(&mut self, ext: &str, enabled: bool) {
        let norm = ext.trim().trim_start_matches('.').to_lowercase();
        if enabled {
            if !self.draft.target_extensions.contains(&norm) {
                self.draft.target_extensions.push(norm);
            }
        } else {
            self.draft.target_extensions.retain(|e| e != &norm);
        }
    }

    /// 独立した `egui` ビューポートとして設定ダイアログを描画し、ユーザーの確定・キャンセル操作を返します。
    pub fn show_viewport(&mut self, ctx: &egui::Context) -> SettingsDialogAction {
        if !self.is_open {
            return SettingsDialogAction::None;
        }

        let mut action = SettingsDialogAction::None;
        let viewport_id = egui::ViewportId::from_hash_of("keystar_settings_dialog");
        let builder = egui::ViewportBuilder::default()
            .with_title("KeyStar 設定")
            .with_inner_size([SETTINGS_WINDOW_WIDTH, SETTINGS_WINDOW_HEIGHT])
            .with_decorations(true)
            .with_resizable(false)
            .with_always_on_top();

        ctx.show_viewport_immediate(viewport_id, builder, |ui, _class| {
            if ui.ctx().input(|i| i.viewport().close_requested()) {
                action = SettingsDialogAction::Cancel;
                return;
            }

            let panel_frame = egui::Frame::central_panel(&ui.ctx().global_style())
                .fill(BG_COLOR)
                .inner_margin(egui::Margin::symmetric(12, 10));

            egui::CentralPanel::default()
                .frame(panel_frame)
                .show(ui, |ui| {
                    action = self.render_contents(ui);
                });
        });

        if action != SettingsDialogAction::None {
            self.close();
        }

        action
    }

    /// 設定ダイアログ内部の UI コンポーネント群を描画します。
    pub fn render_contents(&mut self, ui: &mut egui::Ui) -> SettingsDialogAction {
        // キー入力キャプチャ処理
        if self.capturing_hotkey {
            let mut captured: Option<HotKeyBinding> = None;
            let mut cancel_capture = false;

            ui.input(|i| {
                if i.key_pressed(egui::Key::Escape) {
                    cancel_capture = true;
                    return;
                }

                let mut mods = global_hotkey::hotkey::Modifiers::empty();
                if i.modifiers.ctrl {
                    mods |= global_hotkey::hotkey::Modifiers::CONTROL;
                }
                if i.modifiers.alt {
                    mods |= global_hotkey::hotkey::Modifiers::ALT;
                }
                if i.modifiers.shift {
                    mods |= global_hotkey::hotkey::Modifiers::SHIFT;
                }
                if i.modifiers.mac_cmd || i.modifiers.command && !i.modifiers.ctrl {
                    mods |= global_hotkey::hotkey::Modifiers::META;
                }

                if !mods.is_empty() {
                    for event in &i.events {
                        if let egui::Event::Key {
                            key, pressed: true, ..
                        } = event
                            && let Some(code) = egui_key_to_code(*key)
                        {
                            captured = Some(HotKeyBinding::new(Some(mods), code));
                            break;
                        }
                    }
                }
            });

            if cancel_capture {
                self.capturing_hotkey = false;
            } else if let Some(binding) = captured {
                self.draft.hotkey = format_hotkey_binding(binding);
                self.capturing_hotkey = false;
            }
        } else if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            return SettingsDialogAction::Cancel;
        }

        let mut final_action = SettingsDialogAction::None;

        // 下部アクションボタンを固定フッターとして最下部に配置（絶対に見切れないよう保護）
        egui::Panel::bottom("settings_footer_panel")
            .frame(egui::Frame::NONE.inner_margin(egui::Margin {
                left: 0,
                right: 0,
                top: 8,
                bottom: 2,
            }))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("初期値に戻す").clicked() {
                        self.reset_to_default();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let save_btn = egui::Button::new(
                            egui::RichText::new("保存して適用")
                                .color(TEXT_COLOR)
                                .strong(),
                        )
                        .fill(egui::Color32::from_rgb(0x26, 0x4f, 0x78));

                        if ui.add(save_btn).clicked() {
                            final_action =
                                SettingsDialogAction::SaveAndApply(self.draft.clone().sanitized());
                        }

                        if ui.button("キャンセル").clicked() {
                            final_action = SettingsDialogAction::Cancel;
                        }
                    });
                });
            });

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // ============================================================
                // セクション 1: 全般 (General)
                // ============================================================
                ui.group(|ui| {
                    ui.set_width(ui.available_width());
                    ui.colored_label(
                        CALC_ACCENT_COLOR,
                        egui::RichText::new("全般 (General)").strong().size(14.0),
                    );
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label("グローバルホットキー:");
                        egui::ComboBox::from_id_salt("hotkey_preset_combo")
                            .selected_text(&self.draft.hotkey)
                            .width(170.0)
                            .show_ui(ui, |ui| {
                                for &preset in HOTKEY_PRESETS {
                                    ui.selectable_value(
                                        &mut self.draft.hotkey,
                                        preset.to_string(),
                                        preset,
                                    );
                                }
                            });

                        let capture_label = if self.capturing_hotkey {
                            "キー押下待機中..."
                        } else {
                            "キー入力キャプチャ"
                        };
                        if ui.button(capture_label).clicked() {
                            self.capturing_hotkey = !self.capturing_hotkey;
                        }
                    });

                    ui.add_space(2.0);
                    ui.checkbox(
                        &mut self.draft.auto_start,
                        "Windows 起動時にバックグラウンド常駐で自動起動する",
                    );
                });

                ui.add_space(6.0);

                // ============================================================
                // セクション 2: ショートカット & 検索 (Shortcuts & Search)
                // ============================================================
                ui.group(|ui| {
                    ui.set_width(ui.available_width());
                    ui.colored_label(
                        CALC_ACCENT_COLOR,
                        egui::RichText::new("ショートカット & 検索 (Shortcuts & Search)")
                            .strong()
                            .size(14.0),
                    );
                    ui.add_space(4.0);

                    ui.label("監視ディレクトリ:");
                    egui::Frame::NONE
                        .fill(egui::Color32::from_rgb(0x25, 0x25, 0x26))
                        .inner_margin(egui::Margin::symmetric(6, 4))
                        .corner_radius(egui::CornerRadius::same(3))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            for (idx, dir) in self.draft.watch_directories.iter().enumerate() {
                                let is_selected = self.selected_dir_index == Some(idx);
                                let label = format!("• {dir}");
                                if ui.selectable_label(is_selected, label).clicked() {
                                    self.selected_dir_index = Some(idx);
                                }
                            }
                        });

                    ui.horizontal(|ui| {
                        let text_resp = ui.add(
                            egui::TextEdit::singleline(&mut self.new_dir_input)
                                .hint_text("フォルダパスを入力...")
                                .desired_width(210.0),
                        );
                        let enter_pressed =
                            text_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        if (ui.button("+ 追加").clicked() || enter_pressed)
                            && !self.new_dir_input.trim().is_empty()
                        {
                            let input = self.new_dir_input.clone();
                            self.add_watch_directory(&input);
                            self.new_dir_input.clear();
                        }

                        if ui.button("参照...").clicked()
                            && let Some(folder) = pick_folder_dialog()
                        {
                            let collapsed = collapse_env_path(&folder);
                            self.add_watch_directory(&collapsed);
                        }

                        let can_remove = self.selected_dir_index.is_some()
                            && self.draft.watch_directories.len() > 1;
                        if ui
                            .add_enabled(can_remove, egui::Button::new("選択項目を削除"))
                            .clicked()
                        {
                            self.remove_selected_directory();
                        }
                    });

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label("最大候補表示件数:");
                        ui.add(
                            egui::Slider::new(
                                &mut self.draft.max_candidates,
                                MIN_MAX_CANDIDATES..=MAX_MAX_CANDIDATES,
                            )
                            .suffix(" 件"),
                        );
                    });

                    ui.add_space(2.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.label("対象拡張子:");
                        for &ext in TARGET_EXTENSIONS {
                            let mut enabled = self
                                .draft
                                .target_extensions
                                .iter()
                                .any(|e| e.eq_ignore_ascii_case(ext));
                            if ui.checkbox(&mut enabled, format!(".{ext}")).changed() {
                                self.set_extension_enabled(ext, enabled);
                            }
                        }
                    });
                });

                ui.add_space(6.0);

                // ============================================================
                // セクション 3: 外観 & 電卓 (Appearance & Calculator)
                // ============================================================
                ui.group(|ui| {
                    ui.set_width(ui.available_width());
                    ui.colored_label(
                        CALC_ACCENT_COLOR,
                        egui::RichText::new("外観 & 電卓 (Appearance & Calculator)")
                            .strong()
                            .size(14.0),
                    );
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label("入力枠の横幅:    ");
                        ui.add(
                            egui::Slider::new(
                                &mut self.draft.window_width,
                                MIN_WINDOW_WIDTH..=MAX_WINDOW_WIDTH,
                            )
                            .step_by(10.0)
                            .suffix(" px"),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("フォントサイズ:  ");
                        ui.add(
                            egui::Slider::new(
                                &mut self.draft.font_size,
                                MIN_FONT_SIZE..=MAX_FONT_SIZE,
                            )
                            .step_by(1.0)
                            .suffix(" pt"),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("電卓コピー形式:  ");
                        ui.radio_value(
                            &mut self.draft.calc_copy_format,
                            CalcCopyFormat::Plain,
                            "生数値 (1408)",
                        );
                        ui.radio_value(
                            &mut self.draft.calc_copy_format,
                            CalcCopyFormat::CommaSeparated,
                            "カンマ付き (1,408)",
                        );
                    });
                });
            });

        final_action
    }
}
