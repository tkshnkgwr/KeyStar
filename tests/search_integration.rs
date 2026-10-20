//! 補完・検索エンジンの公開API統合テスト。

use Re_CraftLaunch::calc::evaluate;
use Re_CraftLaunch::indexer::ShortcutItem;
use Re_CraftLaunch::search::{CandidateKind, filter_candidates};
use std::path::PathBuf;

fn create_sample_shortcuts() -> Vec<ShortcutItem> {
    vec![
        ShortcutItem {
            name: "Chrome".to_string(),
            path: PathBuf::from("C:\\commands\\Chrome.lnk"),
            extension: "lnk".to_string(),
        },
        ShortcutItem {
            name: "Calculator".to_string(),
            path: PathBuf::from("C:\\commands\\Calculator.lnk"),
            extension: "lnk".to_string(),
        },
        ShortcutItem {
            name: "Google Drive".to_string(),
            path: PathBuf::from("C:\\commands\\Google Drive.url"),
            extension: "url".to_string(),
        },
        ShortcutItem {
            name: "VSCode".to_string(),
            path: PathBuf::from("C:\\commands\\VSCode.lnk"),
            extension: "lnk".to_string(),
        },
    ]
}

#[test]
fn test_search_prefix_vs_substring() {
    let shortcuts = create_sample_shortcuts();

    // "c" で前方一致する Chrome と Calculator が上位にくる
    let candidates = filter_candidates("c", &shortcuts, None, 10);
    assert!(candidates.len() >= 2);
    let top_names: Vec<&str> = candidates.iter().map(|c| c.label.as_str()).collect();
    assert!(top_names[0].starts_with("C"));
    assert!(top_names[1].starts_with("C"));
}

#[test]
fn test_search_calculator_priority_merge() {
    let shortcuts = create_sample_shortcuts();
    let query = "25 * 4";
    let calc = evaluate(query);

    let candidates = filter_candidates(query, &shortcuts, calc, 5);
    assert!(!candidates.is_empty());
    // 最初の候補は電卓結果
    assert_eq!(candidates[0].label, "= 100");
    assert!(matches!(candidates[0].kind, CandidateKind::Calc(_)));
}

#[test]
fn test_search_case_insensitivity() {
    let shortcuts = create_sample_shortcuts();

    let lower = filter_candidates("vscode", &shortcuts, None, 5);
    let upper = filter_candidates("VSCODE", &shortcuts, None, 5);
    let mixed = filter_candidates("VsCoDe", &shortcuts, None, 5);

    assert_eq!(lower.len(), 1);
    assert_eq!(upper.len(), 1);
    assert_eq!(mixed.len(), 1);
    assert_eq!(lower[0].label, upper[0].label);
    assert_eq!(lower[0].label, mixed[0].label);
}

#[test]
fn test_search_max_results_limit() {
    let shortcuts = create_sample_shortcuts();
    let candidates = filter_candidates("", &shortcuts, None, 2);
    assert_eq!(candidates.len(), 2);
}
