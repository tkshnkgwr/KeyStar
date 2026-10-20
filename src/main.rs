//! Re:CraftLaunch メインエントリポイント
//!
//! `eframe` / `egui` を用いた 1 行極小キーボードランチャー GUI を起動します。

use Re_CraftLaunch::indexer::ShortcutIndexer;
use Re_CraftLaunch::ui::{LauncherApp, build_native_options};

fn main() -> eframe::Result<()> {
    let options = build_native_options();

    eframe::run_native(
        "Re:CraftLaunch",
        options,
        Box::new(|cc| {
            LauncherApp::configure_style(&cc.egui_ctx);
            let indexer = ShortcutIndexer::with_default_directories();
            // Phase 2 単体起動モードでは Esc / Enter 確定時にウィンドウを閉じて終了する（false）
            Ok(Box::new(LauncherApp::new(indexer, false)))
        }),
    )
}
