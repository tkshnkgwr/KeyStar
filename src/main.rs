#![windows_subsystem = "windows"]

//! KeyStar メインエントリポイント
//!
//! `eframe` / `egui` を用いた 1 行極小キーボードランチャー GUI を起動し、
//! グローバルホットキー（`Alt + Space`）およびシステムトレイ常駐を有効化します。

use KeyStar::indexer::ShortcutIndexer;
use KeyStar::single_instance::SingleInstanceGuard;
use KeyStar::ui::{LauncherApp, build_native_options};

fn main() -> eframe::Result<()> {
    // 多重起動防止: 同一セッション内で既に KeyStar が起動している場合は即座に終了する
    let _instance_guard = match SingleInstanceGuard::acquire("Local\\KeyStar_SingleInstance_Mutex")
    {
        Some(guard) => guard,
        None => {
            // 既に KeyStar が起動中のため、2重起動せずに終了
            return Ok(());
        }
    };

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
