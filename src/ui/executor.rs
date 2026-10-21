//! 外部プロセス起動・シェル実行およびビルトインコマンド実行モジュール。
//!
//! ショートカット（`.lnk`）、バッチ（`.bat`）、PowerShell（`.ps1`）、EXE への引数付き起動、
//! および各種スラッシュコマンドの OS 連携処理を担当します。

use crate::clipboard::ClipboardProvider;
use crate::command::{
    BuiltinCommand, current_local_date_string, current_local_time_string, get_local_ip_string,
    lock_workstation, system_shutdown, system_sleep,
};
use crate::indexer::ShortcutIndexer;
use crate::ui::state::LauncherState;
use eframe::egui;
use std::path::Path;

/// 指定されたショートカットまたは実行可能ファイルを非同期起動します。
///
/// 引数が指定されている場合は、Windows ネイティブ API である `ShellExecuteW` を用いて
/// パラメータを確実に引き渡します。
/// PowerShell スクリプト（`.ps1`）の場合は、実行ポリシーを自動バイパス（`-ExecutionPolicy Bypass`）して
/// 直接実行します。
pub fn launch_target(path: &Path, args: &str) {
    #[cfg(windows)]
    {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // 1. PowerShell スクリプト (.ps1) の場合: powershell.exe 経由で実行ポリシーをバイパスして直接キック
        if ext == "ps1" {
            let mut cmd = std::process::Command::new("powershell.exe");
            cmd.arg("-ExecutionPolicy")
                .arg("Bypass")
                .arg("-NoProfile")
                .arg("-File")
                .arg(path);

            if !args.is_empty() {
                for arg in args.split_whitespace() {
                    cmd.arg(arg);
                }
            }
            let _ = cmd.spawn();
            return;
        }

        // 2. 引数が指定されている場合、または open で起動しなかった場合は ShellExecuteW を利用
        if !args.is_empty() {
            use std::os::windows::ffi::OsStrExt;
            use windows_sys::Win32::UI::Shell::ShellExecuteW;
            use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

            let wide_file: Vec<u16> = path
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            let wide_args: Vec<u16> = args.encode_utf16().chain(std::iter::once(0)).collect();

            let result = unsafe {
                ShellExecuteW(
                    std::ptr::null_mut(),
                    std::ptr::null(), // "open"
                    wide_file.as_ptr(),
                    wide_args.as_ptr(),
                    std::ptr::null(),
                    SW_SHOWNORMAL,
                )
            };

            if (result as isize) > 32 {
                return;
            }
        }

        // 3. 引数なし、またはフォールバック
        let _ = open::that_detached(path);
    }

    #[cfg(not(windows))]
    {
        let _ = args;
        let _ = open::that_detached(path);
    }
}

/// ビルトイン（スラッシュ）コマンドの実行ロジック。
pub fn execute_builtin_command(
    ctx: &egui::Context,
    cmd: BuiltinCommand,
    indexer: &mut ShortcutIndexer,
    state: &mut LauncherState,
    clipboard: &mut dyn ClipboardProvider,
) {
    match cmd {
        BuiltinCommand::Exit => {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        BuiltinCommand::Config => {
            if let Some(settings_dir) = indexer
                .directories()
                .get(1)
                .or_else(|| indexer.directories().first())
            {
                let _ = open::that_detached(settings_dir);
            }
        }
        BuiltinCommand::Cmd => {
            if let Some(primary_dir) = indexer.directories().first() {
                let _ = open::that_detached(primary_dir);
            }
        }
        BuiltinCommand::Reload => {
            indexer.scan();
            state.refresh_candidates(indexer.items());
        }
        BuiltinCommand::Restart => {
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                if let Ok(exe) = std::env::current_exe() {
                    let _ = std::process::Command::new("cmd.exe")
                        .arg("/c")
                        .arg(format!(
                            "timeout /t 1 /nobreak >nul & start \"\" \"{}\"",
                            exe.display()
                        ))
                        .creation_flags(0x08000000)
                        .spawn();
                }
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        BuiltinCommand::Task => {
            let _ = open::that_detached("taskmgr.exe");
        }
        BuiltinCommand::Lock => {
            lock_workstation();
        }
        BuiltinCommand::Sleep => {
            system_sleep();
        }
        BuiltinCommand::Shutdown => {
            system_shutdown();
        }
        BuiltinCommand::Date => {
            let date = current_local_date_string();
            let _ = clipboard.copy_text(&date);
            ctx.copy_text(date);
        }
        BuiltinCommand::Time => {
            let time = current_local_time_string();
            let _ = clipboard.copy_text(&time);
            ctx.copy_text(time);
        }
        BuiltinCommand::Ip => {
            let ip = get_local_ip_string().unwrap_or_else(|| "127.0.0.1".to_string());
            let _ = clipboard.copy_text(&ip);
            ctx.copy_text(ip);
        }
        BuiltinCommand::Help => {
            state.update_query("/", indexer.items());
        }
    }
}
