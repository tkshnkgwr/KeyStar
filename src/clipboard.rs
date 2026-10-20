//! OS クリップボード連携モジュール。
//!
//! `arboard` クレートを用いて、電卓の計算結果テキストを Windows 等の OS クリップボードへ
//! 確実かつ即座に格納する機能を提供します。また、単体・統合テスト時に開発環境のクリップボードを
//! 汚染せずに検証可能な [`ClipboardProvider`] トレイトおよび [`MemoryClipboard`] を備えています。

use arboard::Clipboard;

/// クリップボードへのテキスト書き込みを抽象化するトレイト。
///
/// 実環境用の [`SystemClipboard`] とテスト検証用の [`MemoryClipboard`] を統一的に扱います。
pub trait ClipboardProvider: Send {
    /// 指定されたテキストをクリップボードへコピーします。
    ///
    /// # Errors
    ///
    /// OS クリップボードのオープンまたは書き込みに失敗した場合にエラーメッセージを返します。
    fn copy_text(&mut self, text: &str) -> Result<(), String>;

    /// 直近にこのプロバイダ経由でコピーされたテキストを返します（保持している場合）。
    #[must_use]
    fn last_copied_text(&self) -> Option<&str>;
}

/// `arboard` を用いて実際の OS クリップボードへテキストを書き込む標準実装。
#[derive(Debug, Default)]
pub struct SystemClipboard {
    /// 直近にコピーに成功したテキストの記録
    last_text: Option<String>,
}

impl SystemClipboard {
    /// 新しい [`SystemClipboard`] インスタンスを生成します。
    ///
    /// # Panics
    ///
    /// パニックは発生しません。
    ///
    /// # Examples
    ///
    /// ```
    /// use KeyStar::clipboard::{ClipboardProvider, SystemClipboard};
    ///
    /// let cb = SystemClipboard::new();
    /// assert_eq!(cb.last_copied_text(), None);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { last_text: None }
    }
}

impl ClipboardProvider for SystemClipboard {
    fn copy_text(&mut self, text: &str) -> Result<(), String> {
        copy_to_os_clipboard(text)?;
        self.last_text = Some(text.to_string());
        Ok(())
    }

    fn last_copied_text(&self) -> Option<&str> {
        self.last_text.as_deref()
    }
}

/// テストおよびヘッドレス環境向けのオンメモリ・クリップボード実装。
///
/// OS の実際のクリップボードを変更せず、コピーされた文字列をメモリ上に記録します。
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MemoryClipboard {
    /// メモリ上に保持されたクリップボード文字列
    buffer: Option<String>,
}

impl MemoryClipboard {
    /// 空の [`MemoryClipboard`] を生成します。
    ///
    /// # Panics
    ///
    /// パニックは発生しません。
    ///
    /// # Examples
    ///
    /// ```
    /// use KeyStar::clipboard::{ClipboardProvider, MemoryClipboard};
    ///
    /// let mut cb = MemoryClipboard::new();
    /// cb.copy_text("1408").unwrap();
    /// assert_eq!(cb.last_copied_text(), Some("1408"));
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { buffer: None }
    }
}

impl ClipboardProvider for MemoryClipboard {
    fn copy_text(&mut self, text: &str) -> Result<(), String> {
        if text.is_empty() {
            return Err("空の文字列はクリップボードにコピーできません".to_string());
        }
        self.buffer = Some(text.to_string());
        Ok(())
    }

    fn last_copied_text(&self) -> Option<&str> {
        self.buffer.as_deref()
    }
}

/// `arboard` を用いて OS クリップボードへ即座にテキストをコピーします。
///
/// ランチャーウィンドウが確定直後に非表示化（`Visible(false)`）されるケースでも、
/// OS レベルのクリップボードへ同期的に書き込むことで確実に結果を保持します。
///
/// # Errors
///
/// 文字列が空の場合、または OS クリップボードへのアクセスに失敗した場合にエラー文字列を返します。
///
/// # Panics
///
/// パニックは発生しません。
pub fn copy_to_os_clipboard(text: &str) -> Result<(), String> {
    if text.is_empty() {
        return Err("空の文字列はクリップボードにコピーできません".to_string());
    }

    let mut clipboard =
        Clipboard::new().map_err(|e| format!("OSクリップボードの初期化に失敗しました: {e}"))?;
    clipboard
        .set_text(text.to_owned())
        .map_err(|e| format!("OSクリップボードへの書き込みに失敗しました: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_clipboard_copy_and_overwrite() {
        let mut cb = MemoryClipboard::new();
        assert_eq!(cb.last_copied_text(), None);

        assert!(cb.copy_text("1408").is_ok());
        assert_eq!(cb.last_copied_text(), Some("1408"));

        assert!(cb.copy_text("2.5").is_ok());
        assert_eq!(cb.last_copied_text(), Some("2.5"));
    }

    #[test]
    fn test_clipboard_rejects_empty_text() {
        let mut cb = MemoryClipboard::new();
        assert!(cb.copy_text("").is_err());
        assert_eq!(cb.last_copied_text(), None);

        assert!(copy_to_os_clipboard("").is_err());
    }
}
