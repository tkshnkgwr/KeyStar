#![windows_subsystem = "windows"]

//! KeyStar メインエントリポイント
//!
//! `eframe` / `egui` を用いた 1 行極小キーボードランチャー GUI を起動し、
//! グローバルホットキー（`Alt + Space` 等）およびシステムトレイ常駐を有効化します。

use KeyStar::indexer::ShortcutIndexer;
use KeyStar::settings::AppConfig;
use KeyStar::single_instance::SingleInstanceGuard;
use KeyStar::ui::{LauncherApp, build_native_options_with_config};

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

    let config = AppConfig::load_or_default();
    let options = build_native_options_with_config(&config);

    eframe::run_native(
        "KeyStar",
        options,
        Box::new(move |cc| {
            LauncherApp::configure_style(&cc.egui_ctx);
            let indexer = ShortcutIndexer::new(config.resolved_watch_directories());
            // Phase 3/4 常駐モード: 設定反映・ホットキートグル・トレイ常駐・GUI設定画面対応
            Ok(Box::new(LauncherApp::new_resident_with_config(
                indexer,
                config,
                &cc.egui_ctx,
            )))
        }),
    )
}
