//! ランチャーの UI 状態機械・キーコマンド定義モジュール。
//!
//! 入力テキスト、補完候補リスト、選択インデックス、ウィンドウ表示状態の管理、
//! およびキーボード操作（`Tab`, `Ctrl+n`, `Enter`, `Esc` 等）による状態遷移を提供します。

use crate::calc::evaluate;
use crate::command::BuiltinCommand;
use crate::indexer::ShortcutItem;
use crate::search::{Candidate, CandidateKind, filter_candidates, split_command_and_args};
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
    /// ショートカット・実行ファイルを指定引数で非同期起動してウィンドウを非表示にする
    ExecuteShortcut {
        /// 実行対象のファイルパス（`.lnk`, `.bat`, `.cmd`, `.exe`, `.ps1` 等）
        path: PathBuf,
        /// コマンドライン引数文字列（未指定時は空文字列）
        args: String,
    },
    /// 電卓の計算結果テキストをクリップボードへコピーしてウィンドウを非表示にする
    CopyCalcResult(String),
    /// ビルトイン（スラッシュ）コマンドを実行してウィンドウを非表示にする
    ExecuteBuiltin(BuiltinCommand),
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

    /// 入力欄へのフォーカス強制要求を取得し、内部フラグを消費（`false` にリセット）します。
    pub fn take_focus_request(&mut self) -> bool {
        let req = self.focus_requested;
        self.focus_requested = false;
        req
    }

    /// 表示状態と非表示待機状態をトグル切り替えします。
    ///
    /// 非表示化する際は、次回の呼び出しに備えてクエリやドロップダウン状態を完全にクリーンリセットします。
    /// 復帰（表示化）する際は、常に先頭の入力欄へフォーカスが当たるようフォーカス強制フラグを有効化します。
    ///
    /// # Examples
    ///
    /// ```
    /// use KeyStar::ui::LauncherState;
    ///
    /// let mut state = LauncherState::new(8);
    /// assert!(state.is_window_visible());
    ///
    /// // 非表示化
    /// let is_visible = state.toggle_visibility(&[]);
    /// assert!(!is_visible);
    /// assert!(!state.is_window_visible());
    ///
    /// // 再表示（復帰）
    /// let is_visible = state.toggle_visibility(&[]);
    /// assert!(is_visible);
    /// assert!(state.is_window_visible());
    /// assert!(state.is_focus_requested());
    /// ```
    pub fn toggle_visibility(&mut self, shortcuts: &[ShortcutItem]) -> bool {
        if self.window_visible {
            self.window_visible = false;
            self.focus_requested = false;
            self.reset(shortcuts);
        } else {
            self.window_visible = true;
            self.focus_requested = true;
            self.reset(shortcuts);
        }
        self.window_visible
    }

    /// 現在の1行入力テキストを返します。
    #[must_use]
    pub fn query(&self) -> &str {
        &self.query
    }

    /// 現在絞り込まれている候補リストのスライスを返します。
    #[must_use]
    pub fn candidates(&self) -> &[Candidate] {
        &self.candidates
    }

    /// 候補リストがドロップダウンとして展開表示されるべきかを返します。
    #[must_use]
    pub fn is_dropdown_visible(&self) -> bool {
        !self.query.trim().is_empty() || self.force_dropdown_open
    }

    /// ドロップダウンに実際に表示される候補一覧のスライスを返します。
    #[must_use]
    pub fn visible_candidates(&self) -> &[Candidate] {
        if self.is_dropdown_visible() {
            &self.candidates
        } else {
            &[]
        }
    }

    /// 現在選択されている候補のインデックスを返します。
    #[must_use]
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// 現在選択されている候補アイテムを返します。
    #[must_use]
    pub fn selected_candidate(&self) -> Option<&Candidate> {
        self.candidates.get(self.selected_index)
    }

    /// 現在の候補リストの件数に応じたウィンドウの高さを計算して返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use KeyStar::ui::{BASE_WINDOW_HEIGHT, LauncherState};
    ///
    /// let state = LauncherState::new(8);
    /// assert_eq!(state.window_height(), BASE_WINDOW_HEIGHT);
    /// ```
    #[must_use]
    pub fn window_height(&self) -> f32 {
        let count = self.visible_candidates().len();
        if count == 0 {
            BASE_WINDOW_HEIGHT
        } else {
            BASE_WINDOW_HEIGHT + (count as f32 * CANDIDATE_ROW_HEIGHT) + 8.0
        }
    }

    /// 入力文字列が変更された際に呼び出し、リアルタイム電卓評価と候補リスト絞り込みを行います。
    pub fn update_query(&mut self, query: impl Into<String>, shortcuts: &[ShortcutItem]) {
        self.query = query.into();
        self.force_dropdown_open = false;
        self.refresh_candidates(shortcuts);
    }

    /// 現在の入力クエリに基づいて候補リストを再計算します。
    pub fn refresh_candidates(&mut self, shortcuts: &[ShortcutItem]) {
        let calc_result = evaluate(&self.query);
        self.candidates =
            filter_candidates(&self.query, shortcuts, calc_result, self.max_candidates);

        if self.selected_index >= self.candidates.len() {
            self.selected_index = 0;
        }
    }

    /// キーボード操作コマンドを処理し、選択移動またはアクション生成を行います。
    pub fn handle_key_command(
        &mut self,
        cmd: KeyCommand,
        shortcuts: &[ShortcutItem],
    ) -> LauncherAction {
        match cmd {
            KeyCommand::SelectNext => {
                if !self.is_dropdown_visible() {
                    self.force_dropdown_open = true;
                    self.refresh_candidates(shortcuts);
                    self.selected_index = 0;
                    return LauncherAction::None;
                }
                let len = self.candidates.len();
                if len > 0 {
                    self.selected_index = (self.selected_index + 1) % len;
                    self.reflect_selected_to_query();
                }
                LauncherAction::None
            }
            KeyCommand::SelectPrev => {
                if !self.is_dropdown_visible() {
                    self.force_dropdown_open = true;
                    self.refresh_candidates(shortcuts);
                    self.selected_index = self.candidates.len().saturating_sub(1);
                    return LauncherAction::None;
                }
                let len = self.candidates.len();
                if len > 0 {
                    self.selected_index = (self.selected_index + len - 1) % len;
                    self.reflect_selected_to_query();
                }
                LauncherAction::None
            }
            KeyCommand::Confirm => {
                let action = match self.selected_candidate() {
                    Some(candidate) => match &candidate.kind {
                        CandidateKind::Calc(calc) => {
                            LauncherAction::CopyCalcResult(calc.copy_text.clone())
                        }
                        CandidateKind::Shortcut(item) => {
                            let (_, args) = split_command_and_args(&self.query);
                            LauncherAction::ExecuteShortcut {
                                path: item.path.clone(),
                                args: args.to_string(),
                            }
                        }
                        CandidateKind::Builtin(cmd) => LauncherAction::ExecuteBuiltin(*cmd),
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

    /// 候補リストの特定インデックスを直接選択状態にします（マウスクリック等）。
    pub fn select_index(&mut self, index: usize) {
        if index < self.candidates.len() {
            self.selected_index = index;
        }
    }

    /// 現在選択中の候補アイテム名を入力クエリへインクリメンタル補完として反映します。
    fn reflect_selected_to_query(&mut self) {
        if let Some(Candidate {
            kind: CandidateKind::Shortcut(item),
            ..
        }) = self.selected_candidate()
        {
            let (leading_ws, query_body) = {
                let len_before = self.query.len();
                let trimmed_left = self.query.trim_start();
                let ws_len = len_before - trimmed_left.len();
                (&self.query[..ws_len], trimmed_left)
            };

            let (_, args) = split_command_and_args(query_body);

            if args.is_empty() {
                self.query = format!("{leading_ws}{}", item.name);
            } else {
                self.query = format!("{leading_ws}{} {args}", item.name);
            }
        }
    }

    /// 入力欄を初期状態（空文字列・非ドロップダウン）にリセットします。
    pub fn reset(&mut self, shortcuts: &[ShortcutItem]) {
        self.query.clear();
        self.selected_index = 0;
        self.force_dropdown_open = false;
        self.refresh_candidates(shortcuts);
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

        // ショートカット確定（引数なし）
        state.update_query("obs", &shortcuts);
        let action = state.handle_key_command(KeyCommand::Confirm, &shortcuts);
        assert_eq!(
            action,
            LauncherAction::ExecuteShortcut {
                path: PathBuf::from("C:\\commands\\Obsidian.lnk"),
                args: String::new(),
            }
        );
        assert_eq!(state.query(), ""); // 確定後はリセット

        // ショートカット確定（引数あり）
        state.update_query("code C:\\project", &shortcuts);
        let action_with_args = state.handle_key_command(KeyCommand::Confirm, &shortcuts);
        assert_eq!(
            action_with_args,
            LauncherAction::ExecuteShortcut {
                path: PathBuf::from("C:\\commands\\Code.lnk"),
                args: "C:\\project".to_string(),
            }
        );
        assert_eq!(state.query(), "");

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
