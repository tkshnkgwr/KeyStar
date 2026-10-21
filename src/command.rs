//! スラッシュコマンド（ビルトインコマンド）モジュール。
//!
//! 入力欄に `/` から始まる文字列を入力した際に、ランチャー自身の制御や
//! Windows のシステム操作、生産性ユーティリティを即座に実行する機能を提供します。

/// ビルトイン（スラッシュ）コマンドの識別子。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinCommand {
    /// コマンド置き場フォルダ（`%USERPROFILE%\commands`）を開く
    Cmd,
    /// 設定・ショートカット置き場フォルダを開く
    Config,
    /// 今日の日付（`YYYY-MM-DD`）をクリップボードにコピー
    Date,
    /// KeyStar を終了
    Exit,
    /// 利用可能なスラッシュコマンド一覧のヘルプを表示
    Help,
    /// 自身のローカル IP アドレスをクリップボードにコピー
    Ip,
    /// Windows 画面を即座にロック（`Win + L` 相当）
    Lock,
    /// ショートカット一覧を即座に再スキャン（再読み込み）
    Reload,
    /// KeyStar を再起動
    Restart,
    /// PC をシャットダウン
    Shutdown,
    /// PC をスリープ
    Sleep,
    /// タスクマネージャーを起動
    Task,
    /// 現在時刻（`HH:MM:SS`）をクリップボードにコピー
    Time,
}

/// スラッシュコマンドの定義情報。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinCommandDef {
    /// コマンド文字列（例: `"/task"`）
    pub name: &'static str,
    /// ドロップダウンに表示する機能説明
    pub description: &'static str,
    /// コマンド識別子
    pub command: BuiltinCommand,
}

/// KeyStar で利用可能な全スラッシュコマンドの定義一覧。
pub static ALL_BUILTIN_COMMANDS: &[BuiltinCommandDef] = &[
    BuiltinCommandDef {
        name: "/cmd",
        description: "コマンド置き場フォルダを開く",
        command: BuiltinCommand::Cmd,
    },
    BuiltinCommandDef {
        name: "/config",
        description: "設定・ショートカット置き場を開く",
        command: BuiltinCommand::Config,
    },
    BuiltinCommandDef {
        name: "/date",
        description: "今日の日付をクリップボードにコピー",
        command: BuiltinCommand::Date,
    },
    BuiltinCommandDef {
        name: "/exit",
        description: "KeyStar を終了",
        command: BuiltinCommand::Exit,
    },
    BuiltinCommandDef {
        name: "/help",
        description: "スラッシュコマンド一覧を表示",
        command: BuiltinCommand::Help,
    },
    BuiltinCommandDef {
        name: "/ip",
        description: "ローカルIPアドレスをクリップボードにコピー",
        command: BuiltinCommand::Ip,
    },
    BuiltinCommandDef {
        name: "/lock",
        description: "画面をロック (Win + L)",
        command: BuiltinCommand::Lock,
    },
    BuiltinCommandDef {
        name: "/reload",
        description: "ショートカット一覧を即時再スキャン",
        command: BuiltinCommand::Reload,
    },
    BuiltinCommandDef {
        name: "/restart",
        description: "KeyStar を再起動",
        command: BuiltinCommand::Restart,
    },
    BuiltinCommandDef {
        name: "/shutdown",
        description: "PC をシャットダウン",
        command: BuiltinCommand::Shutdown,
    },
    BuiltinCommandDef {
        name: "/sleep",
        description: "PC をスリープ",
        command: BuiltinCommand::Sleep,
    },
    BuiltinCommandDef {
        name: "/task",
        description: "タスクマネージャーを起動",
        command: BuiltinCommand::Task,
    },
    BuiltinCommandDef {
        name: "/time",
        description: "現在時刻をクリップボードにコピー",
        command: BuiltinCommand::Time,
    },
    BuiltinCommandDef {
        name: "/today",
        description: "今日の日付をクリップボードにコピー (/date)",
        command: BuiltinCommand::Date,
    },
];

/// ユーザー入力クエリに合致するスラッシュコマンド定義を絞り込み検索します。
///
/// クエリが単に `"/"` の場合は全コマンドを返し、文字が続く場合は
/// コマンド名に対する前方一致または部分一致（大文字小文字不問）で検索します。
///
/// # Examples
///
/// ```
/// use KeyStar::command::{match_builtin_commands, BuiltinCommand};
///
/// let results = match_builtin_commands("/t");
/// assert!(results.iter().any(|c| c.command == BuiltinCommand::Task));
/// assert!(results.iter().any(|c| c.command == BuiltinCommand::Time));
/// ```
#[must_use]
pub fn match_builtin_commands(query: &str) -> Vec<BuiltinCommandDef> {
    let trimmed = query.trim();
    if !trimmed.starts_with('/') {
        return Vec::new();
    }

    if trimmed == "/" {
        return ALL_BUILTIN_COMMANDS.to_vec();
    }

    let lower_query = trimmed.to_lowercase();
    let mut matched = Vec::new();

    for def in ALL_BUILTIN_COMMANDS {
        let lower_name = def.name.to_lowercase();
        let lower_desc = def.description.to_lowercase();

        if lower_name.starts_with(&lower_query) {
            matched.push((def, 1000 - (lower_name.len() - lower_query.len()) as i32));
        } else if lower_name.contains(&lower_query) || lower_desc.contains(&lower_query[1..]) {
            matched.push((def, 500));
        }
    }

    matched.sort_by(|(a_def, a_score), (b_def, b_score)| {
        b_score
            .cmp(a_score)
            .then_with(|| a_def.name.cmp(b_def.name))
    });

    matched.into_iter().map(|(def, _)| *def).collect()
}

/// 現在のローカル日付文字列（`YYYY-MM-DD`）を取得します。
#[must_use]
pub fn current_local_date_string() -> String {
    #[cfg(windows)]
    {
        let st = get_win32_local_time();
        format!("{:04}-{:02}-{:02}", st.wYear, st.wMonth, st.wDay)
    }
    #[cfg(not(windows))]
    {
        "2026-10-21".to_string()
    }
}

/// 現在のローカル時刻文字列（`HH:MM:SS`）を取得します。
#[must_use]
pub fn current_local_time_string() -> String {
    #[cfg(windows)]
    {
        let st = get_win32_local_time();
        format!("{:02}:{:02}:{:02}", st.wHour, st.wMinute, st.wSecond)
    }
    #[cfg(not(windows))]
    {
        "12:00:00".to_string()
    }
}

/// 端末の主要ローカル IP アドレスを取得します。
///
/// 外部へのルーティング情報から自身の IP を瞬時に解決します（パケット送信なし）。
#[must_use]
pub fn get_local_ip_string() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    Some(socket.local_addr().ok()?.ip().to_string())
}

/// Windows 画面を即座にロックします（`Win + L` 相当）。
pub fn lock_workstation() {
    #[cfg(windows)]
    {
        #[link(name = "user32")]
        unsafe extern "system" {
            fn LockWorkStation() -> i32;
        }
        unsafe {
            let _ = LockWorkStation();
        }
    }
}

/// Windows をスリープ状態に移行させます。
pub fn system_sleep() {
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("rundll32.exe")
            .arg("powrprof.dll,SetSuspendState")
            .arg("0,1,0")
            .spawn();
    }
}

/// Windows をシャットダウンします。
pub fn system_shutdown() {
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("shutdown.exe")
            .arg("/s")
            .arg("/t")
            .arg("0")
            .spawn();
    }
}

#[cfg(windows)]
#[repr(C)]
#[allow(non_snake_case)]
struct SystemTime {
    wYear: u16,
    wMonth: u16,
    wDayOfWeek: u16,
    wDay: u16,
    wHour: u16,
    wMinute: u16,
    wSecond: u16,
    wMilliseconds: u16,
}

#[cfg(windows)]
fn get_win32_local_time() -> SystemTime {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetLocalTime(lpSystemTime: *mut SystemTime);
    }
    let mut st = unsafe { std::mem::zeroed() };
    unsafe { GetLocalTime(&mut st) };
    st
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_builtin_commands() {
        // "/" 単体で全件取得
        let all = match_builtin_commands("/");
        assert_eq!(all.len(), ALL_BUILTIN_COMMANDS.len());

        // "/ta" で "/task" が最上位
        let task_matches = match_builtin_commands("/ta");
        assert_eq!(task_matches[0].command, BuiltinCommand::Task);

        // "/ex" で "/exit" が最上位
        let exit_matches = match_builtin_commands("/ex");
        assert_eq!(exit_matches[0].command, BuiltinCommand::Exit);

        // "/" で始まらないクエリは空
        assert!(match_builtin_commands("task").is_empty());
    }

    #[test]
    fn test_date_and_time_format() {
        let date = current_local_date_string();
        assert_eq!(date.len(), 10);
        assert_eq!(&date[4..5], "-");
        assert_eq!(&date[7..8], "-");

        let time = current_local_time_string();
        assert_eq!(time.len(), 8);
        assert_eq!(&time[2..3], ":");
        assert_eq!(&time[5..6], ":");
    }
}
