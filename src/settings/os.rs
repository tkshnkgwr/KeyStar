//! OS ネイティブ連携、環境変数パス処理、およびキーコード相互変換モジュール。
//!
//! Windows レジストリ（スタートアップ登録）、ネイティブフォルダ選択ダイアログ（`SHBrowseForFolderW`）、
//! `%USERPROFILE%` 等の環境変数パス展開・短縮、およびホットキー文字列とキーコードの変換を提供します。

use crate::hotkey::HotKeyBinding;
use eframe::egui;
use global_hotkey::hotkey::{Code, Modifiers};
use std::path::{Path, PathBuf};

/// 環境変数プレフィックス（`%USERPROFILE%`, `%APPDATA%`, `%LOCALAPPDATA%`, `$HOME`）を
/// 現在のプロセス環境変数で展開し、[`PathBuf`] に変換します。
///
/// # Panics
///
/// パニックは発生しません。
///
/// # Examples
///
/// ```
/// use KeyStar::settings::expand_env_path;
///
/// let p = expand_env_path("./shortcuts");
/// assert_eq!(p.to_str().unwrap(), "./shortcuts");
/// ```
#[must_use]
pub fn expand_env_path(raw: &str) -> PathBuf {
    let trimmed = raw.trim();
    let userprofile = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok();

    if let Some(ref profile) = userprofile {
        if let Some(rest) = strip_prefix_case_insensitive(trimmed, "%USERPROFILE%") {
            let suffix = rest.trim_start_matches(['\\', '/']);
            return if suffix.is_empty() {
                PathBuf::from(profile)
            } else {
                PathBuf::from(profile).join(suffix)
            };
        }
        if let Some(rest) = trimmed.strip_prefix("$HOME") {
            let suffix = rest.trim_start_matches(['\\', '/']);
            return if suffix.is_empty() {
                PathBuf::from(profile)
            } else {
                PathBuf::from(profile).join(suffix)
            };
        }
    }

    if let Ok(appdata) = std::env::var("APPDATA")
        && let Some(rest) = strip_prefix_case_insensitive(trimmed, "%APPDATA%")
    {
        let suffix = rest.trim_start_matches(['\\', '/']);
        return if suffix.is_empty() {
            PathBuf::from(appdata)
        } else {
            PathBuf::from(appdata).join(suffix)
        };
    }

    if let Ok(localappdata) = std::env::var("LOCALAPPDATA")
        && let Some(rest) = strip_prefix_case_insensitive(trimmed, "%LOCALAPPDATA%")
    {
        let suffix = rest.trim_start_matches(['\\', '/']);
        return if suffix.is_empty() {
            PathBuf::from(localappdata)
        } else {
            PathBuf::from(localappdata).join(suffix)
        };
    }

    PathBuf::from(trimmed)
}

/// 絶対パス内に現在のユーザープロファイルパスが含まれる場合、プライバシー保護および
/// ポータビリティ向上のため `%USERPROFILE%` 環境変数表記へ自動的に折りたたみます。
///
/// # Panics
///
/// パニックは発生しません。
#[must_use]
pub fn collapse_env_path(path: &Path) -> String {
    collapse_env_path_str(&path.to_string_lossy())
}

/// パス文字列内のユーザープロファイルディレクトリ部分を `%USERPROFILE%` に置換します。
#[must_use]
pub fn collapse_env_path_str(raw_path: &str) -> String {
    let trimmed = raw_path.trim();
    if let Ok(profile) = std::env::var("USERPROFILE")
        && !profile.trim().is_empty()
        && let Some(rest) = strip_prefix_case_insensitive(trimmed, profile.trim())
    {
        if rest.is_empty() {
            return "%USERPROFILE%".to_string();
        }
        if rest.starts_with('\\') || rest.starts_with('/') {
            return format!("%USERPROFILE%{rest}");
        }
    }
    trimmed.to_string()
}

pub(crate) fn strip_prefix_case_insensitive<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    if text.len() < prefix.len() {
        return None;
    }
    let (head, tail) = text.split_at(prefix.len());
    if head.eq_ignore_ascii_case(prefix) {
        Some(tail)
    } else {
        None
    }
}

/// ホットキー文字列表現（例: `"Alt + Space"`, `"Ctrl + Space"`, `"Win + Space"`）を解析し、
/// [`HotKeyBinding`] に変換します。
///
/// 修飾キーが含まれない場合や未対応のキー名が含まれる場合は `None` を返します。
///
/// # Panics
///
/// パニックは発生しません。
///
/// # Examples
///
/// ```
/// use KeyStar::settings::parse_hotkey_binding;
/// use global_hotkey::hotkey::{Code, Modifiers};
///
/// let alt_space = parse_hotkey_binding("Alt + Space").unwrap();
/// assert_eq!(alt_space.modifiers, Some(Modifiers::ALT));
/// assert_eq!(alt_space.code, Code::Space);
///
/// let ctrl_shift_k = parse_hotkey_binding("Ctrl + Shift + K").unwrap();
/// assert_eq!(ctrl_shift_k.modifiers, Some(Modifiers::CONTROL | Modifiers::SHIFT));
/// assert_eq!(ctrl_shift_k.code, Code::KeyK);
///
/// assert!(parse_hotkey_binding("InvalidKey").is_none());
/// ```
#[must_use]
pub fn parse_hotkey_binding(text: &str) -> Option<HotKeyBinding> {
    let parts: Vec<&str> = text
        .split('+')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if parts.len() < 2 {
        return None;
    }

    let mut modifiers = Modifiers::empty();
    let mut main_code: Option<Code> = None;

    for (idx, part) in parts.iter().enumerate() {
        let lower = part.to_lowercase();
        let is_last = idx == parts.len() - 1;

        match lower.as_str() {
            "alt" | "option" if !is_last => modifiers |= Modifiers::ALT,
            "ctrl" | "control" if !is_last => modifiers |= Modifiers::CONTROL,
            "shift" if !is_last => modifiers |= Modifiers::SHIFT,
            "win" | "windows" | "super" | "meta" | "cmd" if !is_last => {
                modifiers |= Modifiers::META;
            }
            _ => {
                if !is_last {
                    return None;
                }
                main_code = parse_key_code(&lower);
            }
        }
    }

    if modifiers.is_empty() {
        return None;
    }

    main_code.map(|code| HotKeyBinding::new(Some(modifiers), code))
}

/// [`HotKeyBinding`] を正規化された人間可読な文字列（例: `"Alt + Space"`）へ変換します。
///
/// # Panics
///
/// パニックは発生しません。
///
/// # Examples
///
/// ```
/// use KeyStar::hotkey::HotKeyBinding;
/// use KeyStar::settings::format_hotkey_binding;
///
/// assert_eq!(format_hotkey_binding(HotKeyBinding::default_alt_space()), "Alt + Space");
/// ```
#[must_use]
pub fn format_hotkey_binding(binding: HotKeyBinding) -> String {
    let mut parts = Vec::new();
    if let Some(mods) = binding.modifiers {
        if mods.contains(Modifiers::CONTROL) {
            parts.push("Ctrl".to_string());
        }
        if mods.contains(Modifiers::ALT) {
            parts.push("Alt".to_string());
        }
        if mods.contains(Modifiers::SHIFT) {
            parts.push("Shift".to_string());
        }
        if mods.contains(Modifiers::META) {
            parts.push("Win".to_string());
        }
    }
    parts.push(format_key_code(binding.code));
    parts.join(" + ")
}

pub(crate) fn parse_key_code(lower: &str) -> Option<Code> {
    match lower {
        "space" => Some(Code::Space),
        "enter" | "return" => Some(Code::Enter),
        "tab" => Some(Code::Tab),
        "backspace" => Some(Code::Backspace),
        "a" => Some(Code::KeyA),
        "b" => Some(Code::KeyB),
        "c" => Some(Code::KeyC),
        "d" => Some(Code::KeyD),
        "e" => Some(Code::KeyE),
        "f" => Some(Code::KeyF),
        "g" => Some(Code::KeyG),
        "h" => Some(Code::KeyH),
        "i" => Some(Code::KeyI),
        "j" => Some(Code::KeyJ),
        "k" => Some(Code::KeyK),
        "l" => Some(Code::KeyL),
        "m" => Some(Code::KeyM),
        "n" => Some(Code::KeyN),
        "o" => Some(Code::KeyO),
        "p" => Some(Code::KeyP),
        "q" => Some(Code::KeyQ),
        "r" => Some(Code::KeyR),
        "s" => Some(Code::KeyS),
        "t" => Some(Code::KeyT),
        "u" => Some(Code::KeyU),
        "v" => Some(Code::KeyV),
        "w" => Some(Code::KeyW),
        "x" => Some(Code::KeyX),
        "y" => Some(Code::KeyY),
        "z" => Some(Code::KeyZ),
        "0" | "digit0" => Some(Code::Digit0),
        "1" | "digit1" => Some(Code::Digit1),
        "2" | "digit2" => Some(Code::Digit2),
        "3" | "digit3" => Some(Code::Digit3),
        "4" | "digit4" => Some(Code::Digit4),
        "5" | "digit5" => Some(Code::Digit5),
        "6" | "digit6" => Some(Code::Digit6),
        "7" | "digit7" => Some(Code::Digit7),
        "8" | "digit8" => Some(Code::Digit8),
        "9" | "digit9" => Some(Code::Digit9),
        "f1" => Some(Code::F1),
        "f2" => Some(Code::F2),
        "f3" => Some(Code::F3),
        "f4" => Some(Code::F4),
        "f5" => Some(Code::F5),
        "f6" => Some(Code::F6),
        "f7" => Some(Code::F7),
        "f8" => Some(Code::F8),
        "f9" => Some(Code::F9),
        "f10" => Some(Code::F10),
        "f11" => Some(Code::F11),
        "f12" => Some(Code::F12),
        _ => None,
    }
}

pub(crate) fn format_key_code(code: Code) -> String {
    match code {
        Code::Space => "Space".to_string(),
        Code::Enter => "Enter".to_string(),
        Code::Tab => "Tab".to_string(),
        Code::Backspace => "Backspace".to_string(),
        Code::KeyA => "A".to_string(),
        Code::KeyB => "B".to_string(),
        Code::KeyC => "C".to_string(),
        Code::KeyD => "D".to_string(),
        Code::KeyE => "E".to_string(),
        Code::KeyF => "F".to_string(),
        Code::KeyG => "G".to_string(),
        Code::KeyH => "H".to_string(),
        Code::KeyI => "I".to_string(),
        Code::KeyJ => "J".to_string(),
        Code::KeyK => "K".to_string(),
        Code::KeyL => "L".to_string(),
        Code::KeyM => "M".to_string(),
        Code::KeyN => "N".to_string(),
        Code::KeyO => "O".to_string(),
        Code::KeyP => "P".to_string(),
        Code::KeyQ => "Q".to_string(),
        Code::KeyR => "R".to_string(),
        Code::KeyS => "S".to_string(),
        Code::KeyT => "T".to_string(),
        Code::KeyU => "U".to_string(),
        Code::KeyV => "V".to_string(),
        Code::KeyW => "W".to_string(),
        Code::KeyX => "X".to_string(),
        Code::KeyY => "Y".to_string(),
        Code::KeyZ => "Z".to_string(),
        Code::Digit0 => "0".to_string(),
        Code::Digit1 => "1".to_string(),
        Code::Digit2 => "2".to_string(),
        Code::Digit3 => "3".to_string(),
        Code::Digit4 => "4".to_string(),
        Code::Digit5 => "5".to_string(),
        Code::Digit6 => "6".to_string(),
        Code::Digit7 => "7".to_string(),
        Code::Digit8 => "8".to_string(),
        Code::Digit9 => "9".to_string(),
        Code::F1 => "F1".to_string(),
        Code::F2 => "F2".to_string(),
        Code::F3 => "F3".to_string(),
        Code::F4 => "F4".to_string(),
        Code::F5 => "F5".to_string(),
        Code::F6 => "F6".to_string(),
        Code::F7 => "F7".to_string(),
        Code::F8 => "F8".to_string(),
        Code::F9 => "F9".to_string(),
        Code::F10 => "F10".to_string(),
        Code::F11 => "F11".to_string(),
        Code::F12 => "F12".to_string(),
        other => format!("{other:?}"),
    }
}

pub(crate) fn egui_key_to_code(key: egui::Key) -> Option<Code> {
    match key {
        egui::Key::Space => Some(Code::Space),
        egui::Key::Enter => Some(Code::Enter),
        egui::Key::Tab => Some(Code::Tab),
        egui::Key::Backspace => Some(Code::Backspace),
        egui::Key::A => Some(Code::KeyA),
        egui::Key::B => Some(Code::KeyB),
        egui::Key::C => Some(Code::KeyC),
        egui::Key::D => Some(Code::KeyD),
        egui::Key::E => Some(Code::KeyE),
        egui::Key::F => Some(Code::KeyF),
        egui::Key::G => Some(Code::KeyG),
        egui::Key::H => Some(Code::KeyH),
        egui::Key::I => Some(Code::KeyI),
        egui::Key::J => Some(Code::KeyJ),
        egui::Key::K => Some(Code::KeyK),
        egui::Key::L => Some(Code::KeyL),
        egui::Key::M => Some(Code::KeyM),
        egui::Key::N => Some(Code::KeyN),
        egui::Key::O => Some(Code::KeyO),
        egui::Key::P => Some(Code::KeyP),
        egui::Key::Q => Some(Code::KeyQ),
        egui::Key::R => Some(Code::KeyR),
        egui::Key::S => Some(Code::KeyS),
        egui::Key::T => Some(Code::KeyT),
        egui::Key::U => Some(Code::KeyU),
        egui::Key::V => Some(Code::KeyV),
        egui::Key::W => Some(Code::KeyW),
        egui::Key::X => Some(Code::KeyX),
        egui::Key::Y => Some(Code::KeyY),
        egui::Key::Z => Some(Code::KeyZ),
        egui::Key::Num0 => Some(Code::Digit0),
        egui::Key::Num1 => Some(Code::Digit1),
        egui::Key::Num2 => Some(Code::Digit2),
        egui::Key::Num3 => Some(Code::Digit3),
        egui::Key::Num4 => Some(Code::Digit4),
        egui::Key::Num5 => Some(Code::Digit5),
        egui::Key::Num6 => Some(Code::Digit6),
        egui::Key::Num7 => Some(Code::Digit7),
        egui::Key::Num8 => Some(Code::Digit8),
        egui::Key::Num9 => Some(Code::Digit9),
        egui::Key::F1 => Some(Code::F1),
        egui::Key::F2 => Some(Code::F2),
        egui::Key::F3 => Some(Code::F3),
        egui::Key::F4 => Some(Code::F4),
        egui::Key::F5 => Some(Code::F5),
        egui::Key::F6 => Some(Code::F6),
        egui::Key::F7 => Some(Code::F7),
        egui::Key::F8 => Some(Code::F8),
        egui::Key::F9 => Some(Code::F9),
        egui::Key::F10 => Some(Code::F10),
        egui::Key::F11 => Some(Code::F11),
        egui::Key::F12 => Some(Code::F12),
        _ => None,
    }
}

/// Windows レジストリ（`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`）へ
/// KeyStar のスタートアップ自動起動登録または解除を反映します。
///
/// # Errors
///
/// レジストリキーのオープンや値の書き込み・削除に失敗した場合にエラーメッセージを返します。
pub fn sync_windows_auto_start(enable: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
        use windows_sys::Win32::System::Registry::{
            HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE, REG_SZ, RegCloseKey, RegDeleteValueW,
            RegOpenKeyExW, RegSetValueExW,
        };

        let subkey: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Run\0"
            .encode_utf16()
            .collect();
        let value_name: Vec<u16> = "KeyStar\0".encode_utf16().collect();

        let mut hkey: HKEY = std::ptr::null_mut();
        let status = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                subkey.as_ptr(),
                0,
                KEY_SET_VALUE,
                &mut hkey,
            )
        };
        if status != ERROR_SUCCESS {
            return Err(format!(
                "スタートアップレジストリキーのオープンに失敗しました (code: {status})"
            ));
        }

        let result = if enable {
            let exe_path = std::env::current_exe()
                .map_err(|e| format!("実行ファイルパスの取得に失敗しました: {e}"))?;
            let quoted = format!("\"{}\"", exe_path.display());
            let wide_val: Vec<u16> = std::ffi::OsStr::new(&quoted)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            let byte_len = (wide_val.len() * std::mem::size_of::<u16>()) as u32;

            let set_status = unsafe {
                RegSetValueExW(
                    hkey,
                    value_name.as_ptr(),
                    0,
                    REG_SZ,
                    wide_val.as_ptr().cast::<u8>(),
                    byte_len,
                )
            };
            if set_status == ERROR_SUCCESS {
                Ok(())
            } else {
                Err(format!(
                    "スタートアップ登録の書き込みに失敗しました (code: {set_status})"
                ))
            }
        } else {
            let del_status = unsafe { RegDeleteValueW(hkey, value_name.as_ptr()) };
            if del_status == ERROR_SUCCESS || del_status == ERROR_FILE_NOT_FOUND {
                Ok(())
            } else {
                Err(format!(
                    "スタートアップ登録の解除に失敗しました (code: {del_status})"
                ))
            }
        };

        unsafe {
            let _ = RegCloseKey(hkey);
        }
        result
    }

    #[cfg(not(windows))]
    {
        let _ = enable;
        Ok(())
    }
}

/// Windows ネイティブのフォルダ選択ダイアログ（`SHBrowseForFolderW`）を表示し、
/// 選択されたフォルダパスを返します。キャンセル時は `None` を返します。
#[must_use]
pub fn pick_folder_dialog() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        use windows_sys::Win32::System::Com::CoTaskMemFree;
        use windows_sys::Win32::UI::Shell::{
            BIF_NEWDIALOGSTYLE, BIF_RETURNONLYFSDIRS, BROWSEINFOW, SHBrowseForFolderW,
            SHGetPathFromIDListW,
        };

        let title: Vec<u16> = "KeyStar 監視ディレクトリを選択してください\0"
            .encode_utf16()
            .collect();
        let mut display_name = [0u16; 260];

        let bi = BROWSEINFOW {
            hwndOwner: std::ptr::null_mut(),
            pidlRoot: std::ptr::null_mut(),
            pszDisplayName: display_name.as_mut_ptr(),
            lpszTitle: title.as_ptr(),
            ulFlags: BIF_RETURNONLYFSDIRS | BIF_NEWDIALOGSTYLE,
            lpfn: None,
            lParam: 0,
            iImage: 0,
        };

        let pidl = unsafe { SHBrowseForFolderW(&bi) };
        if pidl.is_null() {
            return None;
        }

        let mut path_buf = [0u16; 260];
        let ok = unsafe { SHGetPathFromIDListW(pidl, path_buf.as_mut_ptr()) };
        unsafe {
            CoTaskMemFree(pidl.cast());
        }

        if ok != 0 {
            let len = path_buf
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(path_buf.len());
            let os_str = OsString::from_wide(&path_buf[..len]);
            Some(PathBuf::from(os_str))
        } else {
            None
        }
    }

    #[cfg(not(windows))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_format_hotkey_binding() {
        let b1 = parse_hotkey_binding("Alt + Space").unwrap();
        assert_eq!(format_hotkey_binding(b1), "Alt + Space");

        let b2 = parse_hotkey_binding("ctrl + shift + space").unwrap();
        assert_eq!(format_hotkey_binding(b2), "Ctrl + Shift + Space");

        let b3 = parse_hotkey_binding("Win + Space").unwrap();
        assert_eq!(format_hotkey_binding(b3), "Win + Space");

        assert!(parse_hotkey_binding("Space").is_none());
        assert!(parse_hotkey_binding("Alt + UnknownKey").is_none());
    }

    #[test]
    fn test_strip_prefix_case_insensitive() {
        assert_eq!(
            strip_prefix_case_insensitive("%userprofile%\\commands", "%USERPROFILE%"),
            Some("\\commands")
        );
        assert!(strip_prefix_case_insensitive("other\\path", "%USERPROFILE%").is_none());
    }
}
