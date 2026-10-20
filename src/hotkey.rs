//! グローバルホットキー監視・制御モジュール。
//!
//! `global-hotkey` クレートを用いて OS 全体での `Alt + Space`（または任意のキーバインド）の
//! 押下を監視し、バックグラウンド待機中のランチャーウィンドウを即座にウェイクアップ・トグルします。

use eframe::egui;
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};
use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex, Once, OnceLock};

/// イベントハンドラ経由で受信したホットキーイベントを保持する内部キュー。
static HOTKEY_EVENT_QUEUE: LazyLock<Mutex<VecDeque<GlobalHotKeyEvent>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));

/// バックグラウンド待機中の `egui` イベントループを即座に起こすためのコンテキスト保持セル。
static WAKEUP_EGUI_CTX: OnceLock<Mutex<Option<egui::Context>>> = OnceLock::new();

/// `GlobalHotKeyEvent::set_event_handler` の初期化をプロセス内で1回のみ保証するガード。
static HANDLER_INIT: Once = Once::new();

/// グローバルホットキーのキー組み合わせ定義。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HotKeyBinding {
    /// 修飾キー（`Alt`, `Ctrl`, `Shift`, `Super` 等）
    pub modifiers: Option<Modifiers>,
    /// メインキーコード（`Code::Space` 等）
    pub code: Code,
}

impl Default for HotKeyBinding {
    fn default() -> Self {
        Self::default_alt_space()
    }
}

impl HotKeyBinding {
    /// 指定された修飾キーとキーコードで新しい [`HotKeyBinding`] を生成します。
    ///
    /// # Panics
    ///
    /// パニックは発生しません。
    ///
    /// # Examples
    ///
    /// ```
    /// use KeyStar::hotkey::HotKeyBinding;
    /// use global_hotkey::hotkey::{Code, Modifiers};
    ///
    /// let binding = HotKeyBinding::new(Some(Modifiers::ALT), Code::Space);
    /// assert_eq!(binding, HotKeyBinding::default_alt_space());
    /// ```
    #[must_use]
    pub const fn new(modifiers: Option<Modifiers>, code: Code) -> Self {
        Self { modifiers, code }
    }

    /// KeyStar 標準のホットキー（`Alt + Space`）を返します。
    ///
    /// # Panics
    ///
    /// パニックは発生しません。
    ///
    /// # Examples
    ///
    /// ```
    /// use KeyStar::hotkey::HotKeyBinding;
    ///
    /// let binding = HotKeyBinding::default_alt_space();
    /// assert!(binding.hotkey_id() > 0);
    /// ```
    #[must_use]
    pub const fn default_alt_space() -> Self {
        Self {
            modifiers: Some(Modifiers::ALT),
            code: Code::Space,
        }
    }

    /// `global_hotkey::hotkey::HotKey` オブジェクトへ変換します。
    #[must_use]
    pub fn to_hotkey(self) -> HotKey {
        HotKey::new(self.modifiers, self.code)
    }

    /// このキー組み合わせに対応する一意なホットキーIDを返します。
    #[must_use]
    pub fn hotkey_id(self) -> u32 {
        self.to_hotkey().id()
    }
}

/// 受信したホットキーイベントが対象ホットキーの「押下（Pressed）」であるかを判定します。
///
/// キーを離した際（`HotKeyState::Released`）に二重トグルが発生するのを防止するため、
/// `HotKeyState::Pressed` の場合のみ `true` を返します。
///
/// # Panics
///
/// パニックは発生しません。
///
/// # Examples
///
/// ```
/// use KeyStar::hotkey::should_trigger_hotkey;
/// use global_hotkey::HotKeyState;
///
/// assert!(should_trigger_hotkey(42, 42, HotKeyState::Pressed));
/// assert!(!should_trigger_hotkey(42, 42, HotKeyState::Released));
/// assert!(!should_trigger_hotkey(42, 99, HotKeyState::Pressed));
/// ```
#[must_use]
pub fn should_trigger_hotkey(target_id: u32, event_id: u32, state: HotKeyState) -> bool {
    target_id == event_id && state == HotKeyState::Pressed
}

/// ホットキー発火時に `egui` のイベントループを即座にウェイクアップ（再描画要求）するフックを登録します。
///
/// ウィンドウが非表示（`Visible(false)`）でバックグラウンド待機している際にも、
/// ホットキー押下と同時に遅延なく `LauncherApp::ui()` を呼び出せるようにします。
pub fn install_hotkey_wakeup_hook(ctx: egui::Context) {
    let cell = WAKEUP_EGUI_CTX.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = cell.lock() {
        *guard = Some(ctx);
    }

    HANDLER_INIT.call_once(|| {
        GlobalHotKeyEvent::set_event_handler(Some(|event: GlobalHotKeyEvent| {
            if let Ok(mut queue) = HOTKEY_EVENT_QUEUE.lock() {
                queue.push_back(event);
            }
            if let Some(cell) = WAKEUP_EGUI_CTX.get()
                && let Ok(guard) = cell.lock()
                && let Some(ctx) = guard.as_ref()
            {
                ctx.request_repaint();
            }
        }));
    });
}

/// OS グローバルホットキーの登録・解除およびイベントポーリングを管理するサービス構造体。
pub struct HotKeyService {
    /// OS ホットキーマネージャ
    manager: GlobalHotKeyManager,
    /// 登録済みのホットキー実体
    hotkey: HotKey,
    /// キーバインド設定
    binding: HotKeyBinding,
}

impl HotKeyService {
    /// 標準ホットキー（`Alt + Space`）を OS に登録して [`HotKeyService`] を生成します。
    ///
    /// # Errors
    ///
    /// 他のアプリケーションが既に同一ホットキーを占有している場合、または OS マネージャの
    /// 初期化に失敗した場合にエラーメッセージを返します。
    pub fn new_default() -> Result<Self, String> {
        Self::with_binding(HotKeyBinding::default_alt_space())
    }

    /// 指定された [`HotKeyBinding`] を OS に登録して [`HotKeyService`] を生成します。
    ///
    /// # Errors
    ///
    /// ホットキーマネージャの作成またはホットキー登録に失敗した場合にエラーメッセージを返します。
    pub fn with_binding(binding: HotKeyBinding) -> Result<Self, String> {
        let manager = GlobalHotKeyManager::new()
            .map_err(|e| format!("グローバルホットキーマネージャの初期化に失敗しました: {e}"))?;
        let hotkey = binding.to_hotkey();
        manager
            .register(hotkey)
            .map_err(|e| format!("グローバルホットキーの登録に失敗しました: {e}"))?;

        Ok(Self {
            manager,
            hotkey,
            binding,
        })
    }

    /// 登録されている [`HotKeyBinding`] を返します。
    #[must_use]
    pub const fn binding(&self) -> HotKeyBinding {
        self.binding
    }

    /// 登録されているホットキーの一意なIDを返します。
    #[must_use]
    pub fn hotkey_id(&self) -> u32 {
        self.hotkey.id()
    }

    /// キューに蓄積されたホットキーイベントを走査し、対象ホットキーが押下されたかを返します。
    ///
    /// 複数回イベントが蓄積されていた場合でも、1フレームにつき1回のトグル判定として消化します。
    #[must_use]
    pub fn poll_pressed(&self) -> bool {
        let target_id = self.hotkey.id();
        let mut triggered = false;

        // 1. ウェイクアップフック経由の内部キューを確認
        if let Ok(mut queue) = HOTKEY_EVENT_QUEUE.lock() {
            while let Some(event) = queue.pop_front() {
                if should_trigger_hotkey(target_id, event.id, event.state) {
                    triggered = true;
                }
            }
        }

        // 2. クレート標準レシーバに残っているイベントもドレイン確認
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if should_trigger_hotkey(target_id, event.id, event.state) {
                triggered = true;
            }
        }

        triggered
    }
}

impl Drop for HotKeyService {
    fn drop(&mut self) {
        let _ = self.manager.unregister(self.hotkey);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_alt_space_binding() {
        let binding = HotKeyBinding::default_alt_space();
        assert_eq!(binding.modifiers, Some(Modifiers::ALT));
        assert_eq!(binding.code, Code::Space);
        assert_eq!(binding, HotKeyBinding::default());

        let hk = binding.to_hotkey();
        assert_eq!(binding.hotkey_id(), hk.id());
    }

    #[test]
    fn test_should_trigger_hotkey_only_on_pressed() {
        let target_id = HotKeyBinding::default_alt_space().hotkey_id();
        let other_id = HotKeyBinding::new(Some(Modifiers::CONTROL), Code::KeyK).hotkey_id();

        assert!(should_trigger_hotkey(
            target_id,
            target_id,
            HotKeyState::Pressed
        ));
        assert!(!should_trigger_hotkey(
            target_id,
            target_id,
            HotKeyState::Released
        ));
        assert!(!should_trigger_hotkey(
            target_id,
            other_id,
            HotKeyState::Pressed
        ));
    }
}
