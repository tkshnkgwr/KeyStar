//! Re:CraftLaunch メインエントリポイント
//!
//! Phase 1 ではコアロジック（電卓評価、ショートカット走査、補完検索）の動作確認をCLIで実施できます。
//! Phase 2 にて eframe / egui による 1 行極小 GUI ウィンドウへと統合されます。

use Re_CraftLaunch::calc::evaluate;
use Re_CraftLaunch::indexer::ShortcutIndexer;
use Re_CraftLaunch::search::filter_candidates;

fn main() {
    println!("=== Re:CraftLaunch (v0.1.0-alpha.1) Phase 1 Core Engine ===");

    // 1. ショートカットインデクサの初期化と走査
    let mut indexer = ShortcutIndexer::with_default_directories();
    indexer.ensure_directories_exist();
    let count = indexer.scan();
    println!("監視ディレクトリ:");
    for dir in indexer.directories() {
        println!("  - {}", dir.display());
    }
    println!("ロードされたショートカット件数: {count} 件\n");

    // 2. 電卓評価のテストデモ
    let sample_math = "1280 * 1.1";
    if let Some(calc) = evaluate(sample_math) {
        println!("電卓デモ: '{sample_math}' -> {}", calc.display_text);
    }

    // 3. インクリメンタル検索のデモ
    let sample_query = "1 + 2";
    let calc = evaluate(sample_query);
    let candidates = filter_candidates(sample_query, indexer.items(), calc, 5);
    println!("\n検索デモ (クエリ: '{sample_query}'):");
    for (i, c) in candidates.iter().enumerate() {
        println!("  [{i}] {}", c.label);
    }

    println!("\nPhase 1 コアエンジン正常稼働中。Phase 2 (egui GUI) 準備完了。");
}
