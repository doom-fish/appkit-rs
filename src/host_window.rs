use core::cell::{Cell, RefCell};
use core::ops::{BitOr, BitOrAssign};
use core::ptr::NonNull;
use std::collections::VecDeque;
use std::rc::Rc;

use apple_cf::cg::CGRect;
use doom_fish_utils::panic_safe::catch_user_panic_result;

use crate::callbacks::{
    CallbackRegistration, CallbackTableHeader, CallbackTableV1, HandlerSlot, RegistrationToken,
};
use crate::ffi::host_window::{
    self as ffi, HostPanelOptions, HostWindowCallbackTableV1, HostWindowConfiguration,
};
use crate::main_thread::MainThreadMarker;
use crate::object::Handle;
use crate::screen::Screen;
use crate::window::Window;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowStyleMask(u64);

impl WindowStyleMask {
    pub const BORDERLESS: Self = Self(0);
    pub const TITLED: Self = Self(1 << 0);
    pub const CLOSABLE: Self = Self(1 << 1);
    pub const MINIATURIZABLE: Self = Self(1 << 2);
    pub const RESIZABLE: Self = Self(1 << 3);
    pub const NONACTIVATING_PANEL: Self = Self(1 << 7);
    pub const FULL_SIZE_CONTENT_VIEW: Self = Self(1 << 15);

    #[must_use]
    pub const fn bits(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }
}

impl BitOr for WindowStyleMask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for WindowStyleMask {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[repr(i64)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackingStore {
    Retained = 0,
    Nonretained = 1,
    Buffered = 2,
}

#[derive(Clone, Copy, Debug)]
pub struct HostedWindowConfiguration<'a> {
    pub content_rect: CGRect,
    pub style_mask: WindowStyleMask,
    pub backing_store: BackingStore,
    pub defer_creation: bool,
    pub screen: Option<&'a Screen>,
    pub can_become_main: bool,
    pub can_become_key: bool,
}

impl HostedWindowConfiguration<'_> {
    #[must_use]
    pub const fn new(content_rect: CGRect, style_mask: WindowStyleMask) -> Self {
        Self {
            content_rect,
            style_mask,
            backing_store: BackingStore::Buffered,
            defer_creation: false,
            screen: None,
            can_become_main: true,
            can_become_key: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HostedPanelOptions {
    pub is_floating_panel: Option<bool>,
    pub becomes_key_only_if_needed: Option<bool>,
    pub works_when_modal: Option<bool>,
    pub hides_on_deactivate: Option<bool>,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostedWindowEvent {
    Close = 1,
    WillClose = 2,
    DidResize = 3,
    DidMove = 4,
    DidChangeScreen = 5,
    DidChangeBackingProperties = 6,
    DidBecomeKey = 7,
    DidResignKey = 8,
    DidChangeOcclusionState = 9,
    WillEnterFullScreen = 10,
    DidEnterFullScreen = 11,
    DidFailToEnterFullScreen = 12,
    WillExitFullScreen = 13,
    DidExitFullScreen = 14,
    DidFailToExitFullScreen = 15,
    AddedTitlebarAccessory = 16,
    MoveTabToNewWindow = 17,
    MergeAllWindows = 18,
    SelectNextTab = 19,
    SelectPreviousTab = 20,
    ToggleTabBar = 21,
}

impl HostedWindowEvent {
    const fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            1 => Some(Self::Close),
            2 => Some(Self::WillClose),
            3 => Some(Self::DidResize),
            4 => Some(Self::DidMove),
            5 => Some(Self::DidChangeScreen),
            6 => Some(Self::DidChangeBackingProperties),
            7 => Some(Self::DidBecomeKey),
            8 => Some(Self::DidResignKey),
            9 => Some(Self::DidChangeOcclusionState),
            10 => Some(Self::WillEnterFullScreen),
            11 => Some(Self::DidEnterFullScreen),
            12 => Some(Self::DidFailToEnterFullScreen),
            13 => Some(Self::WillExitFullScreen),
            14 => Some(Self::DidExitFullScreen),
            15 => Some(Self::DidFailToExitFullScreen),
            16 => Some(Self::AddedTitlebarAccessory),
            17 => Some(Self::MoveTabToNewWindow),
            18 => Some(Self::MergeAllWindows),
            19 => Some(Self::SelectNextTab),
            20 => Some(Self::SelectPreviousTab),
            21 => Some(Self::ToggleTabBar),
            _ => None,
        }
    }
}

type EventHandler = Box<dyn FnMut(HostedWindowEvent)>;
type ShouldCloseHandler = Box<dyn FnMut() -> bool>;

#[derive(Default)]
struct WindowHandlers {
    event: HandlerSlot<EventHandler>,
    should_close: HandlerSlot<ShouldCloseHandler>,
    has_event: Cell<bool>,
    has_should_close: Cell<bool>,
    delivering: Cell<bool>,
    pending: RefCell<VecDeque<HostedWindowEvent>>,
}

struct EventDeliveryGuard<'a>(&'a WindowHandlers);

impl Drop for EventDeliveryGuard<'_> {
    fn drop(&mut self) {
        self.0.pending.borrow_mut().clear();
        self.0.delivering.set(false);
    }
}

extern "C" fn should_close_callback(raw_token: u64) -> bool {
    catch_user_panic_result("AppKit hosted window should close", || {
        let main_thread = MainThreadMarker::new()?;
        let token = RegistrationToken::from_raw(raw_token)?;
        let state = CallbackRegistration::<WindowHandlers>::snapshot(token, main_thread)?;
        if !state.has_should_close.get() {
            return Some(true);
        }
        Some(
            state
                .should_close
                .invoke("AppKit hosted window close handler", |handler| handler())
                .unwrap_or(false),
        )
    })
    .flatten()
    .unwrap_or(false)
}

extern "C" fn event_callback(raw_token: u64, raw_event: u32) {
    let _ = catch_user_panic_result("AppKit hosted window event", || {
        let Some(main_thread) = MainThreadMarker::new() else {
            return;
        };
        let Some(token) = RegistrationToken::from_raw(raw_token) else {
            return;
        };
        let Some(state) = CallbackRegistration::<WindowHandlers>::snapshot(token, main_thread)
        else {
            return;
        };
        let Some(event) = HostedWindowEvent::from_raw(raw_event) else {
            return;
        };
        if !state.has_event.get() {
            return;
        }
        state.pending.borrow_mut().push_back(event);
        if state.delivering.replace(true) {
            return;
        }
        let _delivery = EventDeliveryGuard(&state);
        loop {
            let next = state.pending.borrow_mut().pop_front();
            let Some(next) = next else {
                break;
            };
            if CallbackRegistration::<WindowHandlers>::snapshot(token, main_thread).is_none() {
                break;
            }
            if state.has_event.get() {
                let _ = state
                    .event
                    .invoke("AppKit hosted window event handler", |handler| {
                        handler(next);
                    });
            }
        }
    });
}

pub struct HostedWindow {
    handle: Handle,
    registration: CallbackRegistration<WindowHandlers>,
    main_thread: MainThreadMarker,
}

impl HostedWindow {
    #[must_use]
    pub fn new(
        main_thread: MainThreadMarker,
        configuration: HostedWindowConfiguration<'_>,
    ) -> Option<Self> {
        Self::create(main_thread, configuration, None)
    }

    #[must_use]
    pub fn new_panel(
        main_thread: MainThreadMarker,
        configuration: HostedWindowConfiguration<'_>,
        options: HostedPanelOptions,
    ) -> Option<Self> {
        Self::create(main_thread, configuration, Some(options))
    }

    fn create(
        main_thread: MainThreadMarker,
        configuration: HostedWindowConfiguration<'_>,
        panel: Option<HostedPanelOptions>,
    ) -> Option<Self> {
        let registration =
            CallbackRegistration::new(main_thread, Rc::new(WindowHandlers::default()));
        let rect = configuration.content_rect;
        let native_configuration = HostWindowConfiguration {
            x: rect.origin.x,
            y: rect.origin.y,
            width: rect.size.width,
            height: rect.size.height,
            style_mask: configuration.style_mask.bits(),
            backing_store: configuration.backing_store as i64,
            screen: configuration
                .screen
                .map_or(core::ptr::null_mut(), Screen::as_ptr),
            defer_creation: configuration.defer_creation,
            can_become_main: configuration.can_become_main,
            can_become_key: configuration.can_become_key,
        };
        let table = HostWindowCallbackTableV1 {
            header: CallbackTableHeader {
                version: 1,
                size: size_of::<HostWindowCallbackTableV1>(),
            },
            deactivate: CallbackTableV1::for_state::<()>().deactivate,
            should_close: should_close_callback,
            event: event_callback,
        };
        let raw = panel.map_or_else(
            || unsafe {
                ffi::appkit_host_window_new(
                    &raw const native_configuration,
                    registration.token().get(),
                    &raw const table,
                )
            },
            |options| {
                let native_options = HostPanelOptions {
                    is_floating_panel: option_bool(options.is_floating_panel),
                    becomes_key_only_if_needed: option_bool(options.becomes_key_only_if_needed),
                    works_when_modal: option_bool(options.works_when_modal),
                    hides_on_deactivate: option_bool(options.hides_on_deactivate),
                };
                unsafe {
                    ffi::appkit_host_panel_new(
                        &raw const native_configuration,
                        &raw const native_options,
                        registration.token().get(),
                        &raw const table,
                    )
                }
            },
        );
        let handle = unsafe { Handle::from_retained(raw) }?;
        Some(Self {
            handle,
            registration,
            main_thread,
        })
    }

    #[must_use]
    pub fn as_window(&self) -> Window {
        let pointer =
            NonNull::new(self.handle.as_ptr()).expect("a hosted window has a native window");
        unsafe { Window::from_raw_borrowed(pointer, self.main_thread) }
    }

    #[must_use]
    pub const fn token(&self) -> RegistrationToken {
        self.registration.token()
    }

    pub fn set_handler(&self, handler: impl FnMut(HostedWindowEvent) + 'static) {
        let state = self.registration.state();
        let _ = state.event.replace(Some(Box::new(handler)));
        state.has_event.set(true);
    }

    pub fn clear_handler(&self) {
        let state = self.registration.state();
        state.has_event.set(false);
        let _ = state.event.replace(None);
    }

    pub fn set_should_close_handler(&self, handler: impl FnMut() -> bool + 'static) {
        let state = self.registration.state();
        let _ = state.should_close.replace(Some(Box::new(handler)));
        state.has_should_close.set(true);
    }

    pub fn clear_should_close_handler(&self) {
        let state = self.registration.state();
        state.has_should_close.set(false);
        let _ = state.should_close.replace(None);
    }

    pub fn set_can_become_main(&self, enabled: bool) {
        unsafe { ffi::appkit_host_window_set_can_become_main(self.handle.as_ptr(), enabled) };
    }

    pub fn set_can_become_key(&self, enabled: bool) {
        unsafe { ffi::appkit_host_window_set_can_become_key(self.handle.as_ptr(), enabled) };
    }

    #[must_use]
    pub fn can_become_main(&self) -> bool {
        unsafe { ffi::appkit_host_window_can_become_main(self.handle.as_ptr()) }
    }

    #[must_use]
    pub fn can_become_key(&self) -> bool {
        unsafe { ffi::appkit_host_window_can_become_key(self.handle.as_ptr()) }
    }

    #[doc(hidden)]
    #[must_use]
    pub fn verify_should_close(&self) -> Option<bool> {
        match unsafe { ffi::appkit_host_window_verify_should_close(self.handle.as_ptr()) } {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }

    #[doc(hidden)]
    #[must_use]
    pub fn verify_event(&self, event: HostedWindowEvent) -> bool {
        unsafe { ffi::appkit_host_window_verify_event(self.handle.as_ptr(), event as u32) }
    }
}

impl Drop for HostedWindow {
    fn drop(&mut self) {
        let _ = self.registration.deactivate();
        unsafe { ffi::appkit_host_window_detach(self.handle.as_ptr()) };
    }
}

const fn option_bool(value: Option<bool>) -> i8 {
    match value {
        None => -1,
        Some(false) => 0,
        Some(true) => 1,
    }
}
