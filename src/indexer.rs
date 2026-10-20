//! ショートカット・コマンド監視フォルダ走査モジュール。
//!
//! `%USERPROFILE%\commands` やアプリケーションローカルの `shortcuts/` フォルダを走査し、
//! 起動対象となるショートカット（`.lnk`）やバッチ（`.bat`）、URLファイル（`.url`）等の
//! 一覧をメモリ内にインデックス・キャッシュします。

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// 対象とするファイル拡張子の一覧（小文字）。
pub const TARGET_EXTENSIONS: &[&str] = &["lnk", "url", "bat", "cmd", "exe", "ps1"];

/// 検出されたショートカット・実行可能アイテム。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutItem {
    /// 拡張子を除いたファイル名（例: `"Chrome"`, `"Obsidian"`）
    pub name: String,
    /// ファイルの実体フルパス
    pub path: PathBuf,
    /// 小文字化された拡張子（例: `"lnk"`, `"bat"`）
    pub extension: String,
}

impl ShortcutItem {
    /// ファイルパスから [`ShortcutItem`] を生成します。
    ///
    /// 対象外の拡張子、ファイルが存在しない場合、またはファイル名が取得できない場合は `None` を返します。
    ///
    /// # Panics
    ///
    /// パニックは発生しません。
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use KeyStar::indexer::ShortcutItem;
    ///
    /// // 実在しないファイルパスでは None
    /// assert!(ShortcutItem::from_path(Path::new("not_exist.lnk")).is_none());
    /// ```
    #[must_use]
    pub fn from_path(path: &Path) -> Option<Self> {
        if !path.is_file() {
            return None;
        }

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_lowercase)?;

        if !TARGET_EXTENSIONS.contains(&extension.as_str()) {
            return None;
        }

        let file_stem = path.file_stem()?.to_str()?.to_string();

        Some(Self {
            name: file_stem,
            path: path.to_path_buf(),
            extension,
        })
    }
}

/// ショートカットの走査・キャッシュを担うインデクサ。
#[derive(Debug, Clone, Default)]
pub struct ShortcutIndexer {
    /// 監視対象のディレクトリリスト
    directories: Vec<PathBuf>,
    /// キャッシュされたアイテム一覧
    items: Vec<ShortcutItem>,
    /// 最終走査時刻
    last_scanned: Option<SystemTime>,
    /// ディレクトリごとの最終更新時刻キャッシュ
    dir_modified_times: Vec<(PathBuf, SystemTime)>,
}

impl ShortcutIndexer {
    /// 指定された監視対象ディレクトリリストで新しいインデクサを生成します。
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use KeyStar::indexer::ShortcutIndexer;
    ///
    /// let indexer = ShortcutIndexer::new(vec![PathBuf::from("shortcuts")]);
    /// assert_eq!(indexer.directories().len(), 1);
    /// ```
    #[must_use]
    pub fn new(directories: Vec<PathBuf>) -> Self {
        Self {
            directories,
            items: Vec::new(),
            last_scanned: None,
            dir_modified_times: Vec::new(),
        }
    }

    /// 標準の監視対象ディレクトリ（`%USERPROFILE%\commands` および `./shortcuts`）でインデクサを生成します。
    ///
    /// # Examples
    ///
    /// ```
    /// use KeyStar::indexer::ShortcutIndexer;
    ///
    /// let indexer = ShortcutIndexer::with_default_directories();
    /// assert!(!indexer.directories().is_empty());
    /// ```
    #[must_use]
    pub fn with_default_directories() -> Self {
        Self::new(Self::default_directories())
    }

    /// 標準の監視ディレクトリ一覧を取得します。
    ///
    /// 環境変数 `%USERPROFILE%`（または `$HOME`）を展開した `commands` ディレクトリと、
    /// アプリケーション相対の `shortcuts` ディレクトリを返します。
    #[must_use]
    pub fn default_directories() -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        // 1. %USERPROFILE%\commands
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            let commands_dir = PathBuf::from(userprofile).join("commands");
            dirs.push(commands_dir);
        } else if let Ok(home) = std::env::var("HOME") {
            let commands_dir = PathBuf::from(home).join("commands");
            dirs.push(commands_dir);
        }

        // 2. カレント/実行ファイル基準の ./shortcuts
        dirs.push(PathBuf::from("shortcuts"));

        dirs
    }

    /// 監視ディレクトリが存在しない場合に自動生成を試みます。
    ///
    /// 権限不足などで作成に失敗した場合は無視（スキップ）されます。
    pub fn ensure_directories_exist(&self) {
        for dir in &self.directories {
            if !dir.exists() {
                let _ = fs::create_dir_all(dir);
            }
        }
    }

    /// 監視ディレクトリを同期走査し、インデックスキャッシュを最新状態に更新します。
    ///
    /// 検出されたショートカット・実行可能アイテムの件数を返します。
    /// 同名アイテム（大文字小文字区別なし）かつ同名パスの場合は自動的に重複排除されます。
    ///
    /// # Panics
    ///
    /// ディレクトリが存在しない場合や読み取り不可の場合もエラーにはならず安全にスキップされ、パニックは発生しません。
    pub fn scan(&mut self) -> usize {
        let mut new_items = Vec::new();
        let mut new_mod_times = Vec::new();

        for dir in &self.directories {
            if !dir.exists() || !dir.is_dir() {
                continue;
            }

            // ディレクトリ自体の最終更新日時を記録
            if let Ok(modified) = fs::metadata(dir).and_then(|m| m.modified()) {
                new_mod_times.push((dir.clone(), modified));
            }

            // ディレクトリ内ファイルを走査
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(item) = ShortcutItem::from_path(&path) {
                        new_items.push(item);
                    }
                }
            }
        }

        // アイテム名順にソート（大文字小文字無視）
        new_items.sort_by_key(|a| a.name.to_lowercase());
        // 同名ファイルがある場合は重複排除（パスで一意化）
        new_items.dedup_by(|a, b| a.name.eq_ignore_ascii_case(&b.name) && a.path == b.path);

        self.items = new_items;
        self.last_scanned = Some(SystemTime::now());
        self.dir_modified_times = new_mod_times;

        self.items.len()
    }

    /// ディレクトリの最終更新日時を比較し、再スキャンが必要かどうかを判定します。
    ///
    /// 前回の走査以降にディレクトリの内容に変更（追加・削除・更新）があった場合は `true` を返します。
    #[must_use]
    pub fn needs_rescan(&self) -> bool {
        if self.last_scanned.is_none() {
            return true;
        }

        for (dir, last_mod) in &self.dir_modified_times {
            if let Ok(modified) = fs::metadata(dir).and_then(|m| m.modified())
                && modified > *last_mod
            {
                return true;
            }
        }

        false
    }

    /// キャッシュされているアイテム一覧の参照を返します。
    #[must_use]
    pub fn items(&self) -> &[ShortcutItem] {
        &self.items
    }

    /// 監視対象ディレクトリ一覧の参照を返します。
    #[must_use]
    pub fn directories(&self) -> &[PathBuf] {
        &self.directories
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_shortcut_item_from_path() {
        let temp_dir = std::env::temp_dir().join("recraftlaunch_test_indexer");
        let _ = fs::create_dir_all(&temp_dir);

        let lnk_path = temp_dir.join("Chrome.lnk");
        File::create(&lnk_path).unwrap();

        let item = ShortcutItem::from_path(&lnk_path).unwrap();
        assert_eq!(item.name, "Chrome");
        assert_eq!(item.extension, "lnk");
        assert_eq!(item.path, lnk_path);

        // 大文字拡張子の検証
        let bat_path = temp_dir.join("Tool.BAT");
        File::create(&bat_path).unwrap();
        let bat_item = ShortcutItem::from_path(&bat_path).unwrap();
        assert_eq!(bat_item.name, "Tool");
        assert_eq!(bat_item.extension, "bat");

        // 対象外拡張子
        let txt_path = temp_dir.join("Note.txt");
        File::create(&txt_path).unwrap();
        assert!(ShortcutItem::from_path(&txt_path).is_none());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_indexer_scan_and_dedup() {
        let temp_dir = std::env::temp_dir().join("recraftlaunch_test_scan");
        let _ = fs::create_dir_all(&temp_dir);

        File::create(temp_dir.join("App1.bat")).unwrap();
        File::create(temp_dir.join("App2.url")).unwrap();
        File::create(temp_dir.join("ignored.log")).unwrap();

        let mut indexer = ShortcutIndexer::new(vec![temp_dir.clone()]);
        let count = indexer.scan();

        assert_eq!(count, 2);
        assert_eq!(indexer.items().len(), 2);
        assert_eq!(indexer.items()[0].name, "App1");
        assert_eq!(indexer.items()[1].name, "App2");

        // 走査直後は needs_rescan が false
        assert!(!indexer.needs_rescan());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_indexer_non_existent_dir() {
        let non_existent = std::env::temp_dir().join("non_existent_dir_recraftlaunch_12345");
        let mut indexer = ShortcutIndexer::new(vec![non_existent]);
        assert_eq!(indexer.scan(), 0);
        assert!(indexer.items().is_empty());
    }
}
