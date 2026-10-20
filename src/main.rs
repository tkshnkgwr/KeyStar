#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! KeyStar メインエントリポイント
//!
//! `eframe` / `egui` を用いた 1 行極小キーボードランチャー GUI を起動し、
//! グローバルホットキー（`Alt + Space`）およびシステムトレイ常駐を有効化します。

use KeyStar::indexer::ShortcutIndexer;
use KeyStar::ui::{LauncherApp, build_native_options};

fn main() -> eframe::Result<()> {
    let options = build_native_options();

    eframe::run_native(
        "KeyStar",
        options,
        Box::new(|cc| {
            LauncherApp::configure_style(&cc.egui_ctx);
            let indexer = ShortcutIndexer::with_default_directories();
            // Phase 3 常駐モード: Alt+Space トグル、システムトレイ常駐、Esc/Enter 時は非表示待機
            Ok(Box::new(LauncherApp::new_resident(indexer, &cc.egui_ctx)))
        }),
    )
}
