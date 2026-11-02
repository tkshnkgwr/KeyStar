//! アプリケーション設定の永続化（`config.toml`）および GUI 設定ダイアログモジュール。
//!
//! ホットキー、Windows スタートアップ自動起動、監視ディレクトリ、対象拡張子、最大表示候補数、
//! ウィンドウ横幅、フォントサイズ、電卓コピー書式の読み込み・保存および `egui` による
//! 独立設定ウィンドウ描画を提供します。

use crate::calc::CalcResult;
use crate::hotkey::HotKeyBinding;
use crate::indexer::TARGET_EXTENSIONS;
use crate::ui::state::{
    BG_COLOR, CALC_ACCENT_COLOR, DEFAULT_MAX_CANDIDATES, TEXT_COLOR, WINDOW_WIDTH,
};
use eframe::egui;
use global_hotkey::hotkey::{Code, Modifiers};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// 設定ダイアログウィンドウの標準横幅（ピクセル）。
pub const SETTINGS_WINDOW_WIDTH: f32 = 480.0;

/// 設定ダイアログウィンドウの標準高さ（ピクセル）。
pub const SETTINGS_WINDOW_HEIGHT: f32 = 460.0;

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

/// 電卓計算結果のクリップボードコピー書式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CalcCopyFormat {
    /// カンマを含まない生数値プレーンテキスト（例: `"1408"`）
    #[default]
    Plain,
    /// 3桁カンマ区切り付きの数値テキスト（例: `"1,408"`）
    CommaSeparated,
}

impl CalcCopyFormat {
    /// 計算結果 [`CalcResult`] から設定書式に応じたコピー用文字列を取り出します。
    ///
    /// # Panics
    ///
    /// パニックは発生しません。
    ///
    /// # Examples
    ///
    /// ```
    /// use KeyStar::calc::evaluate;
    /// use KeyStar::settings::CalcCopyFormat;
    ///
    /// let res = evaluate("1280 * 1.1").unwrap();
    /// assert_eq!(CalcCopyFormat::Plain.format_for_copy(&res), "1408");
    /// assert_eq!(CalcCopyFormat::CommaSeparated.format_for_copy(&res), "1,408");
    /// ```
    #[must_use]
    pub fn format_for_copy(self, calc: &CalcResult) -> String {
        match self {
            Self::Plain => calc.copy_text.clone(),
            Self::CommaSeparated => calc.formatted_result.clone(),
        }
    }
}

/// `config.toml` に永続化される KeyStar の環境設定スキーマ。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// グローバルホットキー文字列表現（例: `"Alt + Space"`, `"Ctrl + Space"`, `"Win + Space"`）
    pub hotkey: String,
    /// Windows 起動時にバックグラウンド常駐で自動起動するか
    pub auto_start: bool,
    /// ショートカット監視ディレクトリ一覧（環境変数 `%USERPROFILE%` 等の記述に対応）
    pub watch_directories: Vec<String>,
    /// 候補ドロップダウンの最大表示件数（3〜15件）
    pub max_candidates: usize,
    /// スキャン対象とする拡張子一覧（小文字、ドットなし: `"lnk"`, `"url"` 等）
    pub target_extensions: Vec<String>,
    /// ランチャー入力ウィンドウの横幅（400.0〜800.0 px）
    pub window_width: f32,
    /// ランチャー入力欄のフォントサイズ（12.0〜20.0 pt）
    pub font_size: f32,
    /// 電卓計算結果のクリップボードコピー形式
    pub calc_copy_format: CalcCopyFormat,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            hotkey: DEFAULT_HOTKEY_STR.to_string(),
            auto_start: false,
            watch_directories: vec![
                "%USERPROFILE%\\commands".to_string(),
                "./shortcuts".to_string(),
            ],
            max_candidates: DEFAULT_MAX_CANDIDATES,
            target_extensions: TARGET_EXTENSIONS.iter().map(|s| (*s).to_string()).collect(),
            window_width: WINDOW_WIDTH,
            font_size: DEFAULT_FONT_SIZE,
            calc_copy_format: CalcCopyFormat::Plain,
        }
    }
}

impl AppConfig {
    /// 各設定項目を有効範囲内にクランプ・正規化した新しい [`AppConfig`] を返します。
    ///
    /// 不正なホットキー文字列が指定されていた場合は標準の `"Alt + Space"` にフォールバックし、
    /// 監視ディレクトリが空の場合はデフォルトの監視ディレクトリ一覧を復元します。
    ///
    /// # Panics
    ///
    /// パニックは発生しません。
    ///
    /// # Examples
    ///
    /// ```
    /// use KeyStar::settings::AppConfig;
    ///
    /// let mut cfg = AppConfig::default();
    /// cfg.max_candidates = 99;
    /// cfg.window_width = 100.0;
    /// cfg.font_size = 50.0;
    /// cfg.hotkey = "InvalidKeyCombo".to_string();
    ///
    /// let sanitized = cfg.sanitized();
    /// assert_eq!(sanitized.max_candidates, 15);
    /// assert_eq!(sanitized.window_width, 400.0);
    /// assert_eq!(sanitized.font_size, 20.0);
    /// assert_eq!(sanitized.hotkey, "Alt + Space");
    /// ```
    #[must_use]
    pub fn sanitized(mut self) -> Self {
        self.max_candidates = self
            .max_candidates
            .clamp(MIN_MAX_CANDIDATES, MAX_MAX_CANDIDATES);
        self.window_width = self.window_width.clamp(MIN_WINDOW_WIDTH, MAX_WINDOW_WIDTH);
        self.font_size = self.font_size.clamp(MIN_FONT_SIZE, MAX_FONT_SIZE);

        if let Some(binding) = parse_hotkey_binding(&self.hotkey) {
            self.hotkey = format_hotkey_binding(binding);
        } else {
            self.hotkey = DEFAULT_HOTKEY_STR.to_string();
        }

        // 監視ディレクトリの正規化（空白除去・重複排除）
        let mut clean_dirs = Vec::new();
        for dir in self.watch_directories {
            let trimmed = dir.trim();
            if !trimmed.is_empty() {
                let collapsed = collapse_env_path_str(trimmed);
                if !clean_dirs
                    .iter()
                    .any(|existing: &String| existing.eq_ignore_ascii_case(&collapsed))
                {
                    clean_dirs.push(collapsed);
                }
            }
        }
        if clean_dirs.is_empty() {
            clean_dirs = Self::default().watch_directories;
        }
        self.watch_directories = clean_dirs;

        // 対象拡張子の正規化（小文字化・先頭ドット除去・重複排除）
        let mut clean_exts = Vec::new();
        for ext in self.target_extensions {
            let normalized = ext.trim().trim_start_matches('.').to_lowercase();
            if !normalized.is_empty() && !clean_exts.contains(&normalized) {
                clean_exts.push(normalized);
            }
        }
        self.target_extensions = clean_exts;

        self
    }

    /// 設定内の `watch_directories` に含まれる環境変数（`%USERPROFILE%` 等）を展開し、
    /// 実体 [`PathBuf`] のリストとして返します。
    ///
    /// # Panics
    ///
    /// パニックは発生しません。
    ///
    /// # Examples
    ///
    /// ```
    /// use KeyStar::settings::AppConfig;
    ///
    /// let cfg = AppConfig::default();
    /// let dirs = cfg.resolved_watch_directories();
    /// assert_eq!(dirs.len(), 2);
    /// ```
    #[must_use]
    pub fn resolved_watch_directories(&self) -> Vec<PathBuf> {
        self.watch_directories
            .iter()
            .map(|raw| expand_env_path(raw))
            .collect()
    }

    /// 設定内の `hotkey` 文字列を解析して [`HotKeyBinding`] を返します。
    ///
    /// 解析できない文字列の場合は標準の `Alt + Space` を返します。
    #[must_use]
    pub fn resolved_hotkey_binding(&self) -> HotKeyBinding {
        parse_hotkey_binding(&self.hotkey).unwrap_or_else(HotKeyBinding::default_alt_space)
    }

    /// 設定を TOML 形式の文字列へシリアライズします。
    ///
    /// # Errors
    ///
    /// TOML シリアライズに失敗した場合にエラーメッセージを返します。
    pub fn to_toml_string(&self) -> Result<String, String> {
        let clean = self.clone().sanitized();
        toml::to_string_pretty(&clean)
            .map_err(|e| format!("設定のTOMLシリアライズに失敗しました: {e}"))
    }

    /// TOML 形式の文字列から [`AppConfig`] をデシリアライズし、有効範囲へ正規化して返します。
    ///
    /// # Errors
    ///
    /// TOML の構文が不正な場合にエラーメッセージを返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use KeyStar::settings::{AppConfig, CalcCopyFormat};
    ///
    /// let toml_str = r#"
    /// hotkey = "Ctrl + Space"
    /// max_candidates = 10
    /// calc_copy_format = "comma_separated"
    /// "#;
    /// let cfg = AppConfig::from_toml_str(toml_str).unwrap();
    /// assert_eq!(cfg.hotkey, "Ctrl + Space");
    /// assert_eq!(cfg.max_candidates, 10);
    /// assert_eq!(cfg.calc_copy_format, CalcCopyFormat::CommaSeparated);
    /// ```
    pub fn from_toml_str(toml_str: &str) -> Result<Self, String> {
        let parsed: Self = toml::from_str(toml_str)
            .map_err(|e| format!("設定ファイルのパースに失敗しました: {e}"))?;
        Ok(parsed.sanitized())
    }

    /// 指定されたファイルパスから設定を読み込みます。
    ///
    /// # Errors
    ///
    /// ファイルが存在しない場合、読み取りに失敗した場合、または TOML パースに失敗した場合にエラーを返します。
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| {
            format!(
                "設定ファイル ({}) の読み込みに失敗しました: {e}",
                path.display()
            )
        })?;
        Self::from_toml_str(&content)
    }

    /// 指定されたファイルパスへ設定を TOML 形式で保存します。
    ///
    /// 親ディレクトリが存在しない場合は自動的に作成します。
    ///
    /// # Errors
    ///
    /// ディレクトリ作成またはファイル書き込みに失敗した場合にエラーメッセージを返します。
    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
            && !parent.exists()
        {
            fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "設定保存先ディレクトリ ({}) の作成に失敗しました: {e}",
                    parent.display()
                )
            })?;
        }
        let toml_str = self.to_toml_string()?;
        fs::write(path, toml_str).map_err(|e| {
            format!(
                "設定ファイル ({}) の書き込みに失敗しました: {e}",
                path.display()
            )
        })
    }

    /// 標準の設定ファイルパスを解決します。
    ///
    /// 1. カレントディレクトリに `./config.toml` が存在する場合はポータブルモードとして優先採用します。
    /// 2. 存在しない場合は `%APPDATA%\KeyStar\config.toml` を返します。
    /// 3. 環境変数 `APPDATA` が取得できない場合は `./config.toml` にフォールバックします。
    #[must_use]
    pub fn resolve_config_path() -> PathBuf {
        let portable_path = PathBuf::from("config.toml");
        if portable_path.is_file() {
            return portable_path;
        }

        if let Ok(appdata) = std::env::var("APPDATA")
            && !appdata.trim().is_empty()
        {
            return PathBuf::from(appdata).join("KeyStar").join("config.toml");
        }

        portable_path
    }

    /// 標準の設定ファイルパスから設定を読み込み、存在しない場合やエラー時はデフォルト設定を返します。
    #[must_use]
    pub fn load_or_default() -> Self {
        let path = Self::resolve_config_path();
        Self::load_from_path(&path).unwrap_or_default()
    }

    /// 標準の設定ファイルパスへ設定を保存し、保存先のパスを返します。
    ///
    /// # Errors
    ///
    /// ファイル書き込みに失敗した場合にエラーメッセージを返します。
    pub fn save_default_location(&self) -> Result<PathBuf, String> {
        let path = Self::resolve_config_path();
        self.save_to_path(&path)?;
        Ok(path)
    }
}

/// 環境変数プレフィックス（`%USERPROFILE%`, `%APPDATA%`, `%LOCALAPPDATA%`, `$HOME`）を
/// 現在のプロセス環境変数で展開し、[`PathBuf`] に変換します。
///
/// # Panics
///
/// パニックは発生しません。
///
/// # Examples
///
/// ```
/// use KeyStar::settings::expand_env_path;
///
/// let p = expand_env_path("./shortcuts");
/// assert_eq!(p.to_str().unwrap(), "./shortcuts");
/// ```
#[must_use]
pub fn expand_env_path(raw: &str) -> PathBuf {
    let trimmed = raw.trim();
    let userprofile = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok();

    if let Some(ref profile) = userprofile {
        if let Some(rest) = strip_prefix_case_insensitive(trimmed, "%USERPROFILE%") {
            let suffix = rest.trim_start_matches(['\\', '/']);
            return if suffix.is_empty() {
                PathBuf::from(profile)
            } else {
                PathBuf::from(profile).join(suffix)
            };
        }
        if let Some(rest) = trimmed.strip_prefix("$HOME") {
            let suffix = rest.trim_start_matches(['\\', '/']);
            return if suffix.is_empty() {
                PathBuf::from(profile)
            } else {
                PathBuf::from(profile).join(suffix)
            };
        }
    }

    if let Ok(appdata) = std::env::var("APPDATA")
        && let Some(rest) = strip_prefix_case_insensitive(trimmed, "%APPDATA%")
    {
        let suffix = rest.trim_start_matches(['\\', '/']);
        return if suffix.is_empty() {
            PathBuf::from(appdata)
        } else {
            PathBuf::from(appdata).join(suffix)
        };
    }

    if let Ok(localappdata) = std::env::var("LOCALAPPDATA")
        && let Some(rest) = strip_prefix_case_insensitive(trimmed, "%LOCALAPPDATA%")
    {
        let suffix = rest.trim_start_matches(['\\', '/']);
        return if suffix.is_empty() {
            PathBuf::from(localappdata)
        } else {
            PathBuf::from(localappdata).join(suffix)
        };
    }

    PathBuf::from(trimmed)
}

/// 絶対パス内に現在のユーザープロファイルパスが含まれる場合、プライバシー保護および
/// ポータビリティ向上のため `%USERPROFILE%` 環境変数表記へ自動的に折りたたみます。
///
/// # Panics
///
/// パニックは発生しません。
#[must_use]
pub fn collapse_env_path(path: &Path) -> String {
    collapse_env_path_str(&path.to_string_lossy())
}

/// パス文字列内のユーザープロファイルディレクトリ部分を `%USERPROFILE%` に置換します。
#[must_use]
pub fn collapse_env_path_str(raw_path: &str) -> String {
    let trimmed = raw_path.trim();
    if let Ok(profile) = std::env::var("USERPROFILE")
        && !profile.trim().is_empty()
        && let Some(rest) = strip_prefix_case_insensitive(trimmed, profile.trim())
    {
        if rest.is_empty() {
            return "%USERPROFILE%".to_string();
        }
        if rest.starts_with('\\') || rest.starts_with('/') {
            return format!("%USERPROFILE%{rest}");
        }
    }
    trimmed.to_string()
}

fn strip_prefix_case_insensitive<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    if text.len() < prefix.len() {
        return None;
    }
    let (head, tail) = text.split_at(prefix.len());
    if head.eq_ignore_ascii_case(prefix) {
        Some(tail)
    } else {
        None
    }
}

/// ホットキー文字列表現（例: `"Alt + Space"`, `"Ctrl + Space"`, `"Win + Space"`）を解析し、
/// [`HotKeyBinding`] に変換します。
///
/// 修飾キーが含まれない場合や未対応のキー名が含まれる場合は `None` を返します。
///
/// # Panics
///
/// パニックは発生しません。
///
/// # Examples
///
/// ```
/// use KeyStar::settings::parse_hotkey_binding;
/// use global_hotkey::hotkey::{Code, Modifiers};
///
/// let alt_space = parse_hotkey_binding("Alt + Space").unwrap();
/// assert_eq!(alt_space.modifiers, Some(Modifiers::ALT));
/// assert_eq!(alt_space.code, Code::Space);
///
/// let ctrl_shift_k = parse_hotkey_binding("Ctrl + Shift + K").unwrap();
/// assert_eq!(ctrl_shift_k.modifiers, Some(Modifiers::CONTROL | Modifiers::SHIFT));
/// assert_eq!(ctrl_shift_k.code, Code::KeyK);
///
/// assert!(parse_hotkey_binding("InvalidKey").is_none());
/// ```
#[must_use]
pub fn parse_hotkey_binding(text: &str) -> Option<HotKeyBinding> {
    let parts: Vec<&str> = text
        .split('+')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if parts.len() < 2 {
        return None;
    }

    let mut modifiers = Modifiers::empty();
    let mut main_code: Option<Code> = None;

    for (idx, part) in parts.iter().enumerate() {
        let lower = part.to_lowercase();
        let is_last = idx == parts.len() - 1;

        match lower.as_str() {
            "alt" | "option" if !is_last => modifiers |= Modifiers::ALT,
            "ctrl" | "control" if !is_last => modifiers |= Modifiers::CONTROL,
            "shift" if !is_last => modifiers |= Modifiers::SHIFT,
            "win" | "windows" | "super" | "meta" | "cmd" if !is_last => {
                modifiers |= Modifiers::META;
            }
            _ => {
                if !is_last {
                    return None;
                }
                main_code = parse_key_code(&lower);
            }
        }
    }

    if modifiers.is_empty() {
        return None;
    }

    main_code.map(|code| HotKeyBinding::new(Some(modifiers), code))
}

/// [`HotKeyBinding`] を正規化された人間可読な文字列（例: `"Alt + Space"`）へ変換します。
///
/// # Panics
///
/// パニックは発生しません。
///
/// # Examples
///
/// ```
/// use KeyStar::hotkey::HotKeyBinding;
/// use KeyStar::settings::format_hotkey_binding;
///
/// assert_eq!(format_hotkey_binding(HotKeyBinding::default_alt_space()), "Alt + Space");
/// ```
#[must_use]
pub fn format_hotkey_binding(binding: HotKeyBinding) -> String {
    let mut parts = Vec::new();
    if let Some(mods) = binding.modifiers {
        if mods.contains(Modifiers::CONTROL) {
            parts.push("Ctrl".to_string());
        }
        if mods.contains(Modifiers::ALT) {
            parts.push("Alt".to_string());
        }
        if mods.contains(Modifiers::SHIFT) {
            parts.push("Shift".to_string());
        }
        if mods.contains(Modifiers::META) {
            parts.push("Win".to_string());
        }
    }
    parts.push(format_key_code(binding.code));
    parts.join(" + ")
}

fn parse_key_code(lower: &str) -> Option<Code> {
    match lower {
        "space" => Some(Code::Space),
        "enter" | "return" => Some(Code::Enter),
        "tab" => Some(Code::Tab),
        "backspace" => Some(Code::Backspace),
        "a" => Some(Code::KeyA),
        "b" => Some(Code::KeyB),
        "c" => Some(Code::KeyC),
        "d" => Some(Code::KeyD),
        "e" => Some(Code::KeyE),
        "f" => Some(Code::KeyF),
        "g" => Some(Code::KeyG),
        "h" => Some(Code::KeyH),
        "i" => Some(Code::KeyI),
        "j" => Some(Code::KeyJ),
        "k" => Some(Code::KeyK),
        "l" => Some(Code::KeyL),
        "m" => Some(Code::KeyM),
        "n" => Some(Code::KeyN),
        "o" => Some(Code::KeyO),
        "p" => Some(Code::KeyP),
        "q" => Some(Code::KeyQ),
        "r" => Some(Code::KeyR),
        "s" => Some(Code::KeyS),
        "t" => Some(Code::KeyT),
        "u" => Some(Code::KeyU),
        "v" => Some(Code::KeyV),
        "w" => Some(Code::KeyW),
        "x" => Some(Code::KeyX),
        "y" => Some(Code::KeyY),
        "z" => Some(Code::KeyZ),
        "0" | "digit0" => Some(Code::Digit0),
        "1" | "digit1" => Some(Code::Digit1),
        "2" | "digit2" => Some(Code::Digit2),
        "3" | "digit3" => Some(Code::Digit3),
        "4" | "digit4" => Some(Code::Digit4),
        "5" | "digit5" => Some(Code::Digit5),
        "6" | "digit6" => Some(Code::Digit6),
        "7" | "digit7" => Some(Code::Digit7),
        "8" | "digit8" => Some(Code::Digit8),
        "9" | "digit9" => Some(Code::Digit9),
        "f1" => Some(Code::F1),
        "f2" => Some(Code::F2),
        "f3" => Some(Code::F3),
        "f4" => Some(Code::F4),
        "f5" => Some(Code::F5),
        "f6" => Some(Code::F6),
        "f7" => Some(Code::F7),
        "f8" => Some(Code::F8),
        "f9" => Some(Code::F9),
        "f10" => Some(Code::F10),
        "f11" => Some(Code::F11),
        "f12" => Some(Code::F12),
        _ => None,
    }
}

fn format_key_code(code: Code) -> String {
    match code {
        Code::Space => "Space".to_string(),
        Code::Enter => "Enter".to_string(),
        Code::Tab => "Tab".to_string(),
        Code::Backspace => "Backspace".to_string(),
        Code::KeyA => "A".to_string(),
        Code::KeyB => "B".to_string(),
        Code::KeyC => "C".to_string(),
        Code::KeyD => "D".to_string(),
        Code::KeyE => "E".to_string(),
        Code::KeyF => "F".to_string(),
        Code::KeyG => "G".to_string(),
        Code::KeyH => "H".to_string(),
        Code::KeyI => "I".to_string(),
        Code::KeyJ => "J".to_string(),
        Code::KeyK => "K".to_string(),
        Code::KeyL => "L".to_string(),
        Code::KeyM => "M".to_string(),
        Code::KeyN => "N".to_string(),
        Code::KeyO => "O".to_string(),
        Code::KeyP => "P".to_string(),
        Code::KeyQ => "Q".to_string(),
        Code::KeyR => "R".to_string(),
        Code::KeyS => "S".to_string(),
        Code::KeyT => "T".to_string(),
        Code::KeyU => "U".to_string(),
        Code::KeyV => "V".to_string(),
        Code::KeyW => "W".to_string(),
        Code::KeyX => "X".to_string(),
        Code::KeyY => "Y".to_string(),
        Code::KeyZ => "Z".to_string(),
        Code::Digit0 => "0".to_string(),
        Code::Digit1 => "1".to_string(),
        Code::Digit2 => "2".to_string(),
        Code::Digit3 => "3".to_string(),
        Code::Digit4 => "4".to_string(),
        Code::Digit5 => "5".to_string(),
        Code::Digit6 => "6".to_string(),
        Code::Digit7 => "7".to_string(),
        Code::Digit8 => "8".to_string(),
        Code::Digit9 => "9".to_string(),
        Code::F1 => "F1".to_string(),
        Code::F2 => "F2".to_string(),
        Code::F3 => "F3".to_string(),
        Code::F4 => "F4".to_string(),
        Code::F5 => "F5".to_string(),
        Code::F6 => "F6".to_string(),
        Code::F7 => "F7".to_string(),
        Code::F8 => "F8".to_string(),
        Code::F9 => "F9".to_string(),
        Code::F10 => "F10".to_string(),
        Code::F11 => "F11".to_string(),
        Code::F12 => "F12".to_string(),
        other => format!("{other:?}"),
    }
}

fn egui_key_to_code(key: egui::Key) -> Option<Code> {
    match key {
        egui::Key::Space => Some(Code::Space),
        egui::Key::Enter => Some(Code::Enter),
        egui::Key::Tab => Some(Code::Tab),
        egui::Key::Backspace => Some(Code::Backspace),
        egui::Key::A => Some(Code::KeyA),
        egui::Key::B => Some(Code::KeyB),
        egui::Key::C => Some(Code::KeyC),
        egui::Key::D => Some(Code::KeyD),
        egui::Key::E => Some(Code::KeyE),
        egui::Key::F => Some(Code::KeyF),
        egui::Key::G => Some(Code::KeyG),
        egui::Key::H => Some(Code::KeyH),
        egui::Key::I => Some(Code::KeyI),
        egui::Key::J => Some(Code::KeyJ),
        egui::Key::K => Some(Code::KeyK),
        egui::Key::L => Some(Code::KeyL),
        egui::Key::M => Some(Code::KeyM),
        egui::Key::N => Some(Code::KeyN),
        egui::Key::O => Some(Code::KeyO),
        egui::Key::P => Some(Code::KeyP),
        egui::Key::Q => Some(Code::KeyQ),
        egui::Key::R => Some(Code::KeyR),
        egui::Key::S => Some(Code::KeyS),
        egui::Key::T => Some(Code::KeyT),
        egui::Key::U => Some(Code::KeyU),
        egui::Key::V => Some(Code::KeyV),
        egui::Key::W => Some(Code::KeyW),
        egui::Key::X => Some(Code::KeyX),
        egui::Key::Y => Some(Code::KeyY),
        egui::Key::Z => Some(Code::KeyZ),
        egui::Key::Num0 => Some(Code::Digit0),
        egui::Key::Num1 => Some(Code::Digit1),
        egui::Key::Num2 => Some(Code::Digit2),
        egui::Key::Num3 => Some(Code::Digit3),
        egui::Key::Num4 => Some(Code::Digit4),
        egui::Key::Num5 => Some(Code::Digit5),
        egui::Key::Num6 => Some(Code::Digit6),
        egui::Key::Num7 => Some(Code::Digit7),
        egui::Key::Num8 => Some(Code::Digit8),
        egui::Key::Num9 => Some(Code::Digit9),
        egui::Key::F1 => Some(Code::F1),
        egui::Key::F2 => Some(Code::F2),
        egui::Key::F3 => Some(Code::F3),
        egui::Key::F4 => Some(Code::F4),
        egui::Key::F5 => Some(Code::F5),
        egui::Key::F6 => Some(Code::F6),
        egui::Key::F7 => Some(Code::F7),
        egui::Key::F8 => Some(Code::F8),
        egui::Key::F9 => Some(Code::F9),
        egui::Key::F10 => Some(Code::F10),
        egui::Key::F11 => Some(Code::F11),
        egui::Key::F12 => Some(Code::F12),
        _ => None,
    }
}

/// Windows レジストリ（`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`）へ
/// KeyStar のスタートアップ自動起動登録または解除を反映します。
///
/// # Errors
///
/// レジストリキーのオープンや値の書き込み・削除に失敗した場合にエラーメッセージを返します。
pub fn sync_windows_auto_start(enable: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
        use windows_sys::Win32::System::Registry::{
            HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE, REG_SZ, RegCloseKey, RegDeleteValueW,
            RegOpenKeyExW, RegSetValueExW,
        };

        let subkey: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Run\0"
            .encode_utf16()
            .collect();
        let value_name: Vec<u16> = "KeyStar\0".encode_utf16().collect();

        let mut hkey: HKEY = std::ptr::null_mut();
        let status = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                subkey.as_ptr(),
                0,
                KEY_SET_VALUE,
                &mut hkey,
            )
        };
        if status != ERROR_SUCCESS {
            return Err(format!(
                "スタートアップレジストリキーのオープンに失敗しました (code: {status})"
            ));
        }

        let result = if enable {
            let exe_path = std::env::current_exe()
                .map_err(|e| format!("実行ファイルパスの取得に失敗しました: {e}"))?;
            let quoted = format!("\"{}\"", exe_path.display());
            let wide_val: Vec<u16> = std::ffi::OsStr::new(&quoted)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            let byte_len = (wide_val.len() * std::mem::size_of::<u16>()) as u32;

            let set_status = unsafe {
                RegSetValueExW(
                    hkey,
                    value_name.as_ptr(),
                    0,
                    REG_SZ,
                    wide_val.as_ptr().cast::<u8>(),
                    byte_len,
                )
            };
            if set_status == ERROR_SUCCESS {
                Ok(())
            } else {
                Err(format!(
                    "スタートアップ登録の書き込みに失敗しました (code: {set_status})"
                ))
            }
        } else {
            let del_status = unsafe { RegDeleteValueW(hkey, value_name.as_ptr()) };
            if del_status == ERROR_SUCCESS || del_status == ERROR_FILE_NOT_FOUND {
                Ok(())
            } else {
                Err(format!(
                    "スタートアップ登録の解除に失敗しました (code: {del_status})"
                ))
            }
        };

        unsafe {
            let _ = RegCloseKey(hkey);
        }
        result
    }

    #[cfg(not(windows))]
    {
        let _ = enable;
        Ok(())
    }
}

/// Windows ネイティブのフォルダ選択ダイアログ（`SHBrowseForFolderW`）を表示し、
/// 選択されたフォルダパスを返します。キャンセル時は `None` を返します。
#[must_use]
pub fn pick_folder_dialog() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        use windows_sys::Win32::System::Com::CoTaskMemFree;
        use windows_sys::Win32::UI::Shell::{
            BIF_NEWDIALOGSTYLE, BIF_RETURNONLYFSDIRS, BROWSEINFOW, SHBrowseForFolderW,
            SHGetPathFromIDListW,
        };

        let title: Vec<u16> = "KeyStar 監視ディレクトリを選択してください\0"
            .encode_utf16()
            .collect();
        let mut display_name = [0u16; 260];

        let bi = BROWSEINFOW {
            hwndOwner: std::ptr::null_mut(),
            pidlRoot: std::ptr::null_mut(),
            pszDisplayName: display_name.as_mut_ptr(),
            lpszTitle: title.as_ptr(),
            ulFlags: BIF_RETURNONLYFSDIRS | BIF_NEWDIALOGSTYLE,
            lpfn: None,
            lParam: 0,
            iImage: 0,
        };

        let pidl = unsafe { SHBrowseForFolderW(&bi) };
        if pidl.is_null() {
            return None;
        }

        let mut path_buf = [0u16; 260];
        let ok = unsafe { SHGetPathFromIDListW(pidl, path_buf.as_mut_ptr()) };
        unsafe {
            CoTaskMemFree(pidl.cast());
        }

        if ok != 0 {
            let len = path_buf
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(path_buf.len());
            let os_str = OsString::from_wide(&path_buf[..len]);
            Some(PathBuf::from(os_str))
        } else {
            None
        }
    }

    #[cfg(not(windows))]
    {
        None
    }
}

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

                let mut mods = Modifiers::empty();
                if i.modifiers.ctrl {
                    mods |= Modifiers::CONTROL;
                }
                if i.modifiers.alt {
                    mods |= Modifiers::ALT;
                }
                if i.modifiers.shift {
                    mods |= Modifiers::SHIFT;
                }
                if i.modifiers.mac_cmd || i.modifiers.command && !i.modifiers.ctrl {
                    mods |= Modifiers::META;
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

                ui.add_space(10.0);

                // ============================================================
                // 下部アクションボタン
                // ============================================================
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

        final_action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_format_hotkey_binding() {
        let b1 = parse_hotkey_binding("Alt + Space").unwrap();
        assert_eq!(format_hotkey_binding(b1), "Alt + Space");

        let b2 = parse_hotkey_binding("ctrl + shift + space").unwrap();
        assert_eq!(format_hotkey_binding(b2), "Ctrl + Shift + Space");

        let b3 = parse_hotkey_binding("Win + Space").unwrap();
        assert_eq!(format_hotkey_binding(b3), "Win + Space");

        assert!(parse_hotkey_binding("Space").is_none());
        assert!(parse_hotkey_binding("Alt + UnknownKey").is_none());
    }

    #[test]
    fn test_strip_prefix_case_insensitive() {
        assert_eq!(
            strip_prefix_case_insensitive("%userprofile%\\commands", "%USERPROFILE%"),
            Some("\\commands")
        );
        assert!(strip_prefix_case_insensitive("other\\path", "%USERPROFILE%").is_none());
    }
}
