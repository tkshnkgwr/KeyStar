//! アプリケーション設定スキーマおよび `config.toml` 永続化モジュール。
//!
//! `AppConfig` スキーマの定義、TOML 入出力、入力値サニタイズ・クランプ、
//! およびポータブル・システム標準パスの解決ロジックを提供します。

use crate::calc::CalcResult;
use crate::hotkey::HotKeyBinding;
use crate::indexer::TARGET_EXTENSIONS;
use crate::ui::state::{DEFAULT_MAX_CANDIDATES, WINDOW_WIDTH};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use super::os::{
    collapse_env_path_str, expand_env_path, format_hotkey_binding, parse_hotkey_binding,
};
use super::{
    DEFAULT_FONT_SIZE, DEFAULT_HOTKEY_STR, MAX_FONT_SIZE, MAX_MAX_CANDIDATES, MAX_WINDOW_WIDTH,
    MIN_FONT_SIZE, MIN_MAX_CANDIDATES, MIN_WINDOW_WIDTH,
};

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
