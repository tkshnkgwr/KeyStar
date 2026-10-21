//! 多重起動防止（Single Instance）ガードモジュール
//!
//! Windows 名前付きミューテックス（Named Mutex）を用いて、
//! 同一ユーザーセッション内での KeyStar プロセスの多重起動を確実に検知・抑止します。

/// 多重起動を防止するための RAII ガード構造体。
///
/// このガードがスコープ内に保持されている間、ミューテックスの所有権が維持されます。
/// ガードが破棄（ドロップ）またはプロセス終了時に自動的に解放されます。
pub struct SingleInstanceGuard {
    #[cfg(windows)]
    handle: windows_sys::Win32::Foundation::HANDLE,
}

impl SingleInstanceGuard {
    /// 指定された識別名で多重起動ガードの獲得を試みます。
    ///
    /// 既に他のインスタンスが起動している場合は `None` を返し、
    /// 自身が最初のインスタンスである場合はガードオブジェクト `Some(SingleInstanceGuard)` を返します。
    pub fn acquire(name: &str) -> Option<Self> {
        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError};
            use windows_sys::Win32::System::Threading::CreateMutexW;

            // UTF-16 文字列（終端 NULL 含む）へ変換
            let wide_name: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();

            // CreateMutexW: 初期所有権フラグを 1 (TRUE) に指定
            let handle = unsafe { CreateMutexW(std::ptr::null(), 1, wide_name.as_ptr()) };

            // ハンドルの生成失敗（0 / NULL）
            if handle.is_null() {
                return None;
            }

            // 既にミューテックスが存在していた場合（GetLastError == ERROR_ALREADY_EXISTS）
            if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
                unsafe {
                    windows_sys::Win32::Foundation::CloseHandle(handle);
                }
                return None;
            }

            Some(Self { handle })
        }

        #[cfg(not(windows))]
        {
            let _ = name;
            Some(Self {})
        }
    }
}

#[cfg(windows)]
impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                windows_sys::Win32::Foundation::CloseHandle(self.handle);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_instance_guard_acquire_and_release() {
        let mutex_name = "Local\\KeyStar_Test_Mutex_Unique_12345";
        let guard1 = SingleInstanceGuard::acquire(mutex_name);
        assert!(guard1.is_some(), "初回獲得は成功するはずです");

        // 同一プロセス内または別スレッドからの再獲得試行（二重起動の模擬）
        let guard2 = SingleInstanceGuard::acquire(mutex_name);
        assert!(guard2.is_none(), "既に獲得済みの場合は None になるはずです");

        // guard1 を解放
        drop(guard1);

        // 解放後は再度獲得可能になるはず
        let guard3 = SingleInstanceGuard::acquire(mutex_name);
        assert!(guard3.is_some(), "解放後は再度獲得できるはずです");
    }
}
