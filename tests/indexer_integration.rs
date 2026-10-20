//! ショートカットインデクサの公開API統合テスト。

use Re_CraftLaunch::indexer::{ShortcutIndexer, ShortcutItem};
use std::fs::{self, File};

#[test]
fn test_indexer_lifecycle() {
    let temp_dir = std::env::temp_dir().join("recraftlaunch_test_indexer_integration");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    // テスト用ショートカットファイル作成
    File::create(temp_dir.join("Obsidian.lnk")).unwrap();
    File::create(temp_dir.join("Google Chrome.url")).unwrap();
    File::create(temp_dir.join("Terminal.bat")).unwrap();
    File::create(temp_dir.join("Script.ps1")).unwrap();
    File::create(temp_dir.join("App.EXE")).unwrap(); // 大文字拡張子
    File::create(temp_dir.join("README.txt")).unwrap(); // 対象外拡張子

    let mut indexer = ShortcutIndexer::new(vec![temp_dir.clone()]);
    let count = indexer.scan();

    assert_eq!(count, 5);
    assert_eq!(indexer.items().len(), 5);

    let names: Vec<String> = indexer
        .items()
        .iter()
        .map(|item| item.name.clone())
        .collect();
    assert!(names.contains(&"Obsidian".to_string()));
    assert!(names.contains(&"Google Chrome".to_string()));
    assert!(names.contains(&"Terminal".to_string()));
    assert!(names.contains(&"Script".to_string()));
    assert!(names.contains(&"App".to_string()));
    assert!(!names.contains(&"README".to_string()));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_shortcut_item_filtering() {
    let lnk = std::path::Path::new("MyTool.lnk");
    // 実体ファイルなしでは from_path は None
    assert!(ShortcutItem::from_path(lnk).is_none());
}

#[test]
fn test_indexer_default_directories_exist() {
    let dirs = ShortcutIndexer::default_directories();
    assert!(!dirs.is_empty());
    // shortcuts ディレクトリが含まれる
    assert!(dirs.iter().any(|d| d.ends_with("shortcuts")));
}
