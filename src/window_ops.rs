use core::ffi::{c_char, c_void, CStr};
use core::mem::size_of;
use core::ptr::{self, NonNull};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ffi::{CString, OsStr};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use apple_cf::cg::{CGPoint, CGRect, CGSize};
use doom_fish_utils::panic_safe::catch_user_panic_result;

use crate::appearance::AppearanceName;
use crate::callbacks::{CallbackRegistration, CallbackTableHeader, HandlerSlot, RegistrationToken};
use crate::error::AppKitError;
use crate::event::EventRef;
use crate::ffi::{self, window_ops as native};
use crate::main_thread::MainThreadMarker;
use crate::object::Handle;
use crate::util::{optional_c_string, read_rect, read_size, take_string};
use crate::view::View;
use crate::window::Window;

macro_rules! window_flags {
    ($name:ident { $($flag:ident = $value:expr),* $(,)? }) => {
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
        pub struct $name(u64);

        impl $name {
            $(pub const $flag: Self = Self($value);)*

            #[must_use]
            pub const fn from_bits_retain(bits: u64) -> Self {
                Self(bits)
            }

            #[must_use]
            pub const fn bits(self) -> u64 {
                self.0
            }

            #[must_use]
            pub const fn contains(self, other: Self) -> bool {
                self.0 & other.0 == other.0
            }
        }

        impl core::ops::BitOr for $name {
            type Output = Self;

            fn bitor(self, other: Self) -> Self {
                Self(self.0 | other.0)
            }
        }

        impl core::ops::BitOrAssign for $name {
            fn bitor_assign(&mut self, other: Self) {
                self.0 |= other.0;
            }
        }
    };
}

window_flags!(WindowStyleMask {
    BORDERLESS = 0,
    TITLED = 1 << 0,
    CLOSABLE = 1 << 1,
    MINIATURIZABLE = 1 << 2,
    RESIZABLE = 1 << 3,
    UTILITY_WINDOW = 1 << 4,
    NONACTIVATING_PANEL = 1 << 7,
    FULL_SCREEN = 1 << 14,
    FULL_SIZE_CONTENT_VIEW = 1 << 15,
});

window_flags!(WindowCollectionBehavior {
    DEFAULT = 0,
    CAN_JOIN_ALL_SPACES = 1 << 0,
    MOVE_TO_ACTIVE_SPACE = 1 << 1,
    MANAGED = 1 << 2,
    TRANSIENT = 1 << 3,
    STATIONARY = 1 << 4,
    PARTICIPATES_IN_CYCLE = 1 << 5,
    IGNORES_CYCLE = 1 << 6,
    FULL_SCREEN_PRIMARY = 1 << 7,
    FULL_SCREEN_AUXILIARY = 1 << 8,
    FULL_SCREEN_NONE = 1 << 9,
});

window_flags!(WindowOcclusionState {
    VISIBLE = 1 << 1,
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowLevel {
    Normal,
    Floating,
    ModalPanel,
    PopUpMenu,
    Other(i64),
}

impl WindowLevel {
    const fn from_raw(raw: i64) -> Self {
        match raw {
            0 => Self::Normal,
            3 => Self::Floating,
            8 => Self::ModalPanel,
            101 => Self::PopUpMenu,
            _ => Self::Other(raw),
        }
    }

    const fn raw(self) -> i64 {
        match self {
            Self::Normal => 0,
            Self::Floating => 3,
            Self::ModalPanel => 8,
            Self::PopUpMenu => 101,
            Self::Other(raw) => raw,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowOrderingMode {
    Out,
    Above,
    Below,
}

impl WindowOrderingMode {
    const fn raw(self) -> i64 {
        match self {
            Self::Out => 0,
            Self::Above => 1,
            Self::Below => -1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowTabbingMode {
    Automatic,
    Preferred,
    Disallowed,
    Other(i64),
}

impl WindowTabbingMode {
    const fn from_raw(raw: i64) -> Self {
        match raw {
            0 => Self::Automatic,
            1 => Self::Preferred,
            2 => Self::Disallowed,
            _ => Self::Other(raw),
        }
    }

    const fn raw(self) -> i64 {
        match self {
            Self::Automatic => 0,
            Self::Preferred => 1,
            Self::Disallowed => 2,
            Self::Other(raw) => raw,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowButton {
    Close,
    Miniaturize,
    Zoom,
}

impl WindowButton {
    const fn raw(self) -> i64 {
        match self {
            Self::Close => 0,
            Self::Miniaturize => 1,
            Self::Zoom => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SheetResponse {
    Ok,
    Cancel,
    Stopped,
    Aborted,
    Continue,
    Other(i64),
}

impl SheetResponse {
    #[must_use]
    pub const fn from_raw(raw: i64) -> Self {
        match raw {
            1 => Self::Ok,
            0 => Self::Cancel,
            -1000 => Self::Stopped,
            -1001 => Self::Aborted,
            -1002 => Self::Continue,
            _ => Self::Other(raw),
        }
    }

    #[must_use]
    pub const fn raw(self) -> i64 {
        match self {
            Self::Ok => 1,
            Self::Cancel => 0,
            Self::Stopped => -1000,
            Self::Aborted => -1001,
            Self::Continue => -1002,
            Self::Other(raw) => raw,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RgbaColor {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
    pub alpha: f64,
}

impl RgbaColor {
    #[must_use]
    pub const fn new(red: f64, green: f64, blue: f64, alpha: f64) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

fn check_window_operation(error: *mut c_char) -> Result<(), AppKitError> {
    unsafe { take_string(error) }
        .map_or(Ok(()), |message| Err(AppKitError::InvalidArgument(message)))
}

#[derive(Debug, Clone)]
pub struct WindowChromeView {
    handle: Handle,
}

impl WindowChromeView {
    #[must_use]
    pub const fn as_ptr(&self) -> *mut c_void {
        self.handle.as_ptr()
    }

    #[must_use]
    pub fn as_view(&self) -> View {
        View::from_handle(self.handle.clone())
    }

    #[must_use]
    pub fn superview(&self) -> Option<Self> {
        unsafe { Handle::from_retained(native::appkit_window_ops_chrome_superview(self.as_ptr())) }
            .map(|handle| Self { handle })
    }

    #[must_use]
    pub fn frame(&self) -> CGRect {
        read_rect(|out| unsafe { native::appkit_window_ops_chrome_frame(self.as_ptr(), out) })
    }

    pub fn set_frame(&self, frame: CGRect) -> Result<(), AppKitError> {
        check_window_operation(unsafe {
            native::appkit_window_ops_chrome_set_frame(self.as_ptr(), &raw const frame)
        })
    }

    pub fn set_frame_origin(&self, origin: CGPoint) -> Result<(), AppKitError> {
        check_window_operation(unsafe {
            native::appkit_window_ops_chrome_set_frame_origin(self.as_ptr(), &raw const origin)
        })
    }

    #[must_use]
    pub fn is_hidden(&self) -> bool {
        unsafe { native::appkit_window_ops_chrome_is_hidden(self.as_ptr()) }
    }

    pub fn set_hidden(&self, hidden: bool) {
        unsafe { native::appkit_window_ops_chrome_set_hidden(self.as_ptr(), hidden) };
    }

    pub fn update_tracking_areas(&self) {
        unsafe { native::appkit_window_ops_chrome_update_tracking_areas(self.as_ptr()) };
    }

    pub fn set_needs_display(&self, needs_display: bool) {
        unsafe { native::appkit_window_ops_view_set_needs_display(self.as_ptr(), needs_display) };
    }
}

struct SheetCompletionState {
    handler: HandlerSlot<Box<dyn FnMut(SheetResponse)>>,
    delivering: Cell<bool>,
}

thread_local! {
    static SHEET_REGISTRATIONS: RefCell<HashMap<RegistrationToken, CallbackRegistration<SheetCompletionState>>> =
        RefCell::new(HashMap::new());
}

#[must_use]
pub struct SheetCompletion {
    token: RegistrationToken,
    _main_thread: MainThreadMarker,
}

impl SheetCompletion {
    #[must_use]
    pub fn cancel(&self) -> bool {
        release_sheet_completion(self.token)
    }
}

fn register_sheet_completion(
    main_thread: MainThreadMarker,
    callback: Box<dyn FnOnce(SheetResponse)>,
) -> SheetCompletion {
    let mut callback = Some(callback);
    let state = Rc::new(SheetCompletionState {
        handler: HandlerSlot::new(),
        delivering: Cell::new(false),
    });
    let _ = state.handler.replace(Some(Box::new(move |response| {
        if let Some(callback) = callback.take() {
            callback(response);
        }
    })));
    let registration = CallbackRegistration::new(main_thread, state);
    let token = registration.token();
    SHEET_REGISTRATIONS.with(|registrations| {
        let old = registrations.borrow_mut().insert(token, registration);
        assert!(old.is_none(), "sheet tokens are unique");
    });
    SheetCompletion {
        token,
        _main_thread: main_thread,
    }
}

fn release_sheet_completion(token: RegistrationToken) -> bool {
    let registration = SHEET_REGISTRATIONS
        .try_with(|registrations| {
            registrations
                .try_borrow_mut()
                .ok()
                .and_then(|mut registrations| registrations.remove(&token))
        })
        .ok()
        .flatten();
    let Some(registration) = registration else {
        return false;
    };
    let _ = registration.deactivate();
    drop(registration);
    true
}

fn deliver_sheet_completion(
    token: RegistrationToken,
    response: SheetResponse,
    main_thread: MainThreadMarker,
) {
    let Some(state) = CallbackRegistration::<SheetCompletionState>::snapshot(token, main_thread)
    else {
        return;
    };
    if state.delivering.replace(true) {
        return;
    }
    let _ = state
        .handler
        .invoke("AppKit sheet handler", |handler| handler(response));
    let _ = release_sheet_completion(token);
}

static SHEET_CALLBACKS: native::SheetCallbackTableV1 = native::SheetCallbackTableV1 {
    header: CallbackTableHeader {
        version: 1,
        size: size_of::<native::SheetCallbackTableV1>(),
    },
    deactivate: sheet_deactivate_from_swift,
    complete: sheet_complete,
};

extern "C" fn sheet_complete(raw_token: u64, raw_response: i64) {
    let _ = catch_user_panic_result("AppKit sheet completion", || {
        let Some(main_thread) = MainThreadMarker::new() else {
            return;
        };
        let Some(token) = RegistrationToken::from_raw(raw_token) else {
            return;
        };
        deliver_sheet_completion(token, SheetResponse::from_raw(raw_response), main_thread);
    });
}

extern "C" fn sheet_deactivate_from_swift(raw_token: u64) {
    let _ = catch_user_panic_result("AppKit sheet deactivation", || {
        if MainThreadMarker::new().is_none() {
            return;
        }
        if let Some(token) = RegistrationToken::from_raw(raw_token) {
            let _ = release_sheet_completion(token);
        }
    });
}

impl Window {
    #[must_use]
    pub fn style_mask(&self) -> WindowStyleMask {
        WindowStyleMask::from_bits_retain(unsafe {
            native::appkit_window_ops_style_mask(self.as_ptr())
        })
    }

    pub fn set_style_mask(&self, mask: WindowStyleMask) -> Result<(), AppKitError> {
        check_window_operation(unsafe {
            native::appkit_window_ops_set_style_mask(self.as_ptr(), mask.bits())
        })
    }

    #[must_use]
    pub fn level(&self) -> WindowLevel {
        WindowLevel::from_raw(unsafe { native::appkit_window_ops_level(self.as_ptr()) })
    }

    pub fn set_level(&self, level: WindowLevel) -> Result<(), AppKitError> {
        check_window_operation(unsafe {
            native::appkit_window_ops_set_level(self.as_ptr(), level.raw())
        })
    }

    #[must_use]
    pub fn collection_behavior(&self) -> WindowCollectionBehavior {
        WindowCollectionBehavior::from_bits_retain(unsafe {
            native::appkit_window_ops_collection_behavior(self.as_ptr())
        })
    }

    pub fn set_collection_behavior(
        &self,
        behavior: WindowCollectionBehavior,
    ) -> Result<(), AppKitError> {
        check_window_operation(unsafe {
            native::appkit_window_ops_set_collection_behavior(self.as_ptr(), behavior.bits())
        })
    }

    #[must_use]
    pub fn make_first_responder(&self, view: &View) -> bool {
        unsafe { native::appkit_window_ops_make_first_responder(self.as_ptr(), view.as_ptr()) }
    }

    pub fn set_initial_first_responder(&self, view: Option<&View>) -> Result<(), AppKitError> {
        check_window_operation(unsafe {
            native::appkit_window_ops_set_initial_first_responder(
                self.as_ptr(),
                view.map_or(ptr::null_mut(), View::as_ptr),
            )
        })
    }

    pub fn set_frame(
        &self,
        frame: CGRect,
        display: bool,
        animate: bool,
    ) -> Result<(), AppKitError> {
        check_window_operation(unsafe {
            native::appkit_window_ops_set_frame(self.as_ptr(), &raw const frame, display, animate)
        })
    }

    pub fn set_frame_top_left_point(&self, point: CGPoint) -> Result<(), AppKitError> {
        check_window_operation(unsafe {
            native::appkit_window_ops_set_frame_top_left_point(self.as_ptr(), &raw const point)
        })
    }

    #[must_use]
    pub fn content_size(&self) -> CGSize {
        read_size(|out| unsafe { native::appkit_window_ops_content_size(self.as_ptr(), out) })
    }

    pub fn set_content_size(&self, size: CGSize) -> Result<(), AppKitError> {
        check_window_operation(unsafe {
            native::appkit_window_ops_set_content_size(self.as_ptr(), &raw const size)
        })
    }

    #[must_use]
    pub fn frame_rect_for_content_rect(&self, rect: CGRect) -> CGRect {
        read_rect(|out| unsafe {
            native::appkit_window_ops_frame_rect_for_content_rect(
                self.as_ptr(),
                &raw const rect,
                out,
            );
        })
    }

    #[must_use]
    pub fn content_rect_for_frame_rect(&self, rect: CGRect) -> CGRect {
        read_rect(|out| unsafe {
            native::appkit_window_ops_content_rect_for_frame_rect(
                self.as_ptr(),
                &raw const rect,
                out,
            );
        })
    }

    #[must_use]
    pub fn convert_point_to_screen(&self, point: CGPoint) -> CGPoint {
        let mut result = CGPoint::new(0.0, 0.0);
        unsafe {
            native::appkit_window_ops_convert_point_to_screen(
                self.as_ptr(),
                &raw const point,
                &raw mut result,
            );
        };
        result
    }

    #[must_use]
    pub fn convert_rect_to_screen(&self, rect: CGRect) -> CGRect {
        read_rect(|out| unsafe {
            native::appkit_window_ops_convert_rect_to_screen(self.as_ptr(), &raw const rect, out);
        })
    }

    #[must_use]
    pub fn minimum_content_size(&self) -> CGSize {
        read_size(|out| unsafe { native::appkit_window_ops_content_min_size(self.as_ptr(), out) })
    }

    pub fn set_minimum_content_size(&self, size: CGSize) -> Result<(), AppKitError> {
        check_window_operation(unsafe {
            native::appkit_window_ops_set_content_min_size(self.as_ptr(), &raw const size)
        })
    }

    #[must_use]
    pub fn mouse_location_outside_event_stream(&self) -> CGPoint {
        let mut result = CGPoint::new(0.0, 0.0);
        unsafe {
            native::appkit_window_ops_mouse_location_outside_event_stream(
                self.as_ptr(),
                &raw mut result,
            );
        };
        result
    }

    #[must_use]
    pub fn effective_appearance(&self) -> Option<AppearanceName> {
        AppearanceName::from_raw(unsafe {
            native::appkit_window_ops_effective_appearance(self.as_ptr())
        })
    }

    #[must_use]
    pub fn is_key_window(&self) -> bool {
        unsafe { native::appkit_window_ops_is_key_window(self.as_ptr()) }
    }

    #[must_use]
    pub fn is_main_window(&self) -> bool {
        unsafe { native::appkit_window_ops_is_main_window(self.as_ptr()) }
    }

    #[must_use]
    pub fn is_visible(&self) -> bool {
        unsafe { native::appkit_window_ops_is_visible(self.as_ptr()) }
    }

    #[must_use]
    pub fn occlusion_state(&self) -> WindowOcclusionState {
        WindowOcclusionState::from_bits_retain(unsafe {
            native::appkit_window_ops_occlusion_state(self.as_ptr())
        })
    }

    #[must_use]
    pub fn is_miniaturized(&self) -> bool {
        unsafe { native::appkit_window_ops_is_miniaturized(self.as_ptr()) }
    }

    pub fn miniaturize(&self) {
        unsafe { native::appkit_window_ops_miniaturize(self.as_ptr()) };
    }

    pub fn deminiaturize(&self) {
        unsafe { native::appkit_window_ops_deminiaturize(self.as_ptr()) };
    }

    #[must_use]
    pub fn is_zoomed(&self) -> bool {
        unsafe { native::appkit_window_ops_is_zoomed(self.as_ptr()) }
    }

    pub fn zoom(&self) {
        unsafe { native::appkit_window_ops_zoom(self.as_ptr()) };
    }

    pub fn toggle_full_screen(&self) {
        unsafe { native::appkit_window_ops_toggle_full_screen(self.as_ptr()) };
    }

    #[must_use]
    pub fn is_simple_full_screen(&self) -> bool {
        unsafe { native::appkit_window_ops_is_simple_full_screen(self.as_ptr()) }
    }

    #[must_use]
    pub fn toggle_simple_full_screen(&self, view: &View) -> bool {
        unsafe { native::appkit_window_ops_toggle_simple_full_screen(self.as_ptr(), view.as_ptr()) }
    }

    pub fn begin_sheet(
        &self,
        sheet: &Self,
        main_thread: MainThreadMarker,
        completion: Option<Box<dyn FnOnce(SheetResponse)>>,
    ) -> Result<Option<SheetCompletion>, AppKitError> {
        if self.as_ptr() == sheet.as_ptr() {
            return Err(AppKitError::InvalidArgument(
                "a window cannot be its own sheet".to_owned(),
            ));
        }

        let completion =
            completion.map(|callback| register_sheet_completion(main_thread, callback));
        let token = completion
            .as_ref()
            .map_or(0, |completion| completion.token.get());
        let table = if token == 0 {
            ptr::null()
        } else {
            &raw const SHEET_CALLBACKS
        };
        if unsafe {
            native::appkit_window_ops_begin_sheet(self.as_ptr(), sheet.as_ptr(), token, table)
        } {
            Ok(completion)
        } else {
            if let Some(completion) = completion {
                let _ = completion.cancel();
            }
            Err(AppKitError::InvalidArgument(
                "the window cannot begin this sheet".to_owned(),
            ))
        }
    }

    #[must_use]
    pub fn end_sheet(&self, sheet: &Self, response: SheetResponse) -> bool {
        unsafe {
            native::appkit_window_ops_end_sheet(self.as_ptr(), sheet.as_ptr(), response.raw())
        }
    }

    #[must_use]
    pub fn attached_sheet(&self) -> Option<Self> {
        unsafe { Handle::from_retained(native::appkit_window_ops_attached_sheet(self.as_ptr())) }
            .map(Self::from_handle)
    }

    #[must_use]
    pub fn sheet_parent(&self) -> Option<Self> {
        unsafe { Handle::from_retained(native::appkit_window_ops_sheet_parent(self.as_ptr())) }
            .map(Self::from_handle)
    }
}

impl Window {
    #[must_use]
    pub fn allows_automatic_window_tabbing(_main_thread: MainThreadMarker) -> bool {
        unsafe { native::appkit_window_ops_allows_automatic_window_tabbing() }
    }

    pub fn set_allows_automatic_window_tabbing(_main_thread: MainThreadMarker, allows: bool) {
        unsafe { native::appkit_window_ops_set_allows_automatic_window_tabbing(allows) };
    }

    #[must_use]
    pub fn tabbing_identifier(&self) -> Option<String> {
        unsafe { take_string(native::appkit_window_ops_tabbing_identifier(self.as_ptr())) }
    }

    pub fn set_tabbing_identifier(&self, identifier: Option<&str>) -> Result<(), AppKitError> {
        let identifier = optional_c_string(identifier)?;
        check_window_operation(unsafe {
            native::appkit_window_ops_set_tabbing_identifier(
                self.as_ptr(),
                identifier
                    .as_ref()
                    .map_or(ptr::null(), |value| value.as_ptr()),
            )
        })
    }

    #[must_use]
    pub fn tabbing_mode(&self) -> WindowTabbingMode {
        WindowTabbingMode::from_raw(unsafe {
            native::appkit_window_ops_tabbing_mode(self.as_ptr())
        })
    }

    pub fn set_tabbing_mode(&self, mode: WindowTabbingMode) -> Result<(), AppKitError> {
        check_window_operation(unsafe {
            native::appkit_window_ops_set_tabbing_mode(self.as_ptr(), mode.raw())
        })
    }

    #[must_use]
    pub fn add_tabbed_window(&self, other: &Self, order: WindowOrderingMode) -> bool {
        unsafe {
            native::appkit_window_ops_add_tabbed_window(self.as_ptr(), other.as_ptr(), order.raw())
        }
    }

    #[must_use]
    pub fn tabbed_windows(&self) -> Option<Vec<Self>> {
        let windows = unsafe {
            Handle::from_retained(native::appkit_window_ops_tabbed_windows(self.as_ptr()))
        }?;
        let count = unsafe { native::appkit_window_ops_tabbed_windows_count(windows.as_ptr()) };
        Some(
            (0..count)
                .filter_map(|index| unsafe {
                    Handle::from_retained(native::appkit_window_ops_tabbed_window_at(
                        windows.as_ptr(),
                        index,
                    ))
                })
                .map(Self::from_handle)
                .collect(),
        )
    }

    #[must_use]
    pub fn is_tab_bar_visible(&self) -> bool {
        unsafe { native::appkit_window_ops_is_tab_bar_visible(self.as_ptr()) }
    }

    #[must_use]
    pub fn selected_tab(&self) -> Option<Self> {
        unsafe { Handle::from_retained(native::appkit_window_ops_selected_tab(self.as_ptr())) }
            .map(Self::from_handle)
    }

    #[must_use]
    pub fn select_tab(&self, selected: &Self) -> bool {
        unsafe { native::appkit_window_ops_select_tab(self.as_ptr(), selected.as_ptr()) }
    }

    pub fn toggle_tab_overview(&self) {
        unsafe { native::appkit_window_ops_toggle_tab_overview(self.as_ptr()) };
    }

    pub fn move_tab_to_new_window(&self) {
        unsafe { native::appkit_window_ops_move_tab_to_new_window(self.as_ptr()) };
    }

    pub fn merge_all_windows(&self) {
        unsafe { native::appkit_window_ops_merge_all_windows(self.as_ptr()) };
    }

    pub fn select_next_tab(&self) {
        unsafe { native::appkit_window_ops_select_next_tab(self.as_ptr()) };
    }

    pub fn select_previous_tab(&self) {
        unsafe { native::appkit_window_ops_select_previous_tab(self.as_ptr()) };
    }

    pub fn toggle_tab_bar(&self) {
        unsafe { native::appkit_window_ops_toggle_tab_bar(self.as_ptr()) };
    }

    #[must_use]
    pub fn standard_window_button(&self, button: WindowButton) -> Option<WindowChromeView> {
        unsafe {
            Handle::from_retained(native::appkit_window_ops_standard_window_button(
                self.as_ptr(),
                button.raw(),
            ))
        }
        .map(|handle| WindowChromeView { handle })
    }
}

impl Window {
    #[must_use]
    pub fn background_color(&self) -> Option<RgbaColor> {
        let mut color = RgbaColor::new(0.0, 0.0, 0.0, 0.0);
        unsafe { native::appkit_window_ops_background_color(self.as_ptr(), &raw mut color) }
            .then_some(color)
    }

    pub fn set_background_color(&self, color: RgbaColor) -> Result<(), AppKitError> {
        check_window_operation(unsafe {
            native::appkit_window_ops_set_background_color(self.as_ptr(), &raw const color)
        })
    }

    #[must_use]
    pub fn is_opaque(&self) -> bool {
        unsafe { native::appkit_window_ops_is_opaque(self.as_ptr()) }
    }

    pub fn set_opaque(&self, opaque: bool) {
        unsafe { native::appkit_window_ops_set_opaque(self.as_ptr(), opaque) };
    }

    #[must_use]
    pub fn has_shadow(&self) -> bool {
        unsafe { native::appkit_window_ops_has_shadow(self.as_ptr()) }
    }

    pub fn set_has_shadow(&self, shadow: bool) {
        unsafe { native::appkit_window_ops_set_has_shadow(self.as_ptr(), shadow) };
    }

    pub fn invalidate_shadow(&self) {
        unsafe { native::appkit_window_ops_invalidate_shadow(self.as_ptr()) };
    }

    #[must_use]
    pub fn titlebar_appears_transparent(&self) -> bool {
        unsafe { native::appkit_window_ops_titlebar_appears_transparent(self.as_ptr()) }
    }

    pub fn set_titlebar_appears_transparent(&self, transparent: bool) {
        unsafe {
            native::appkit_window_ops_set_titlebar_appears_transparent(self.as_ptr(), transparent);
        };
    }

    #[must_use]
    pub fn is_movable(&self) -> bool {
        unsafe { native::appkit_window_ops_is_movable(self.as_ptr()) }
    }

    pub fn set_movable(&self, movable: bool) {
        unsafe { native::appkit_window_ops_set_movable(self.as_ptr(), movable) };
    }

    #[must_use]
    pub fn is_movable_by_window_background(&self) -> bool {
        unsafe { native::appkit_window_ops_is_movable_by_window_background(self.as_ptr()) }
    }

    pub fn set_movable_by_window_background(&self, movable: bool) {
        unsafe {
            native::appkit_window_ops_set_movable_by_window_background(self.as_ptr(), movable);
        };
    }

    #[must_use]
    pub fn accepts_mouse_moved_events(&self) -> bool {
        unsafe { native::appkit_window_ops_accepts_mouse_moved_events(self.as_ptr()) }
    }

    pub fn set_accepts_mouse_moved_events(&self, accepts: bool) {
        unsafe { native::appkit_window_ops_set_accepts_mouse_moved_events(self.as_ptr(), accepts) };
    }

    pub fn perform_window_drag(&self, event: EventRef<'_>) -> Result<(), AppKitError> {
        check_window_operation(unsafe {
            native::appkit_window_ops_perform_window_drag(self.as_ptr(), event.as_ptr())
        })
    }

    #[must_use]
    pub fn is_document_edited(&self) -> bool {
        unsafe { native::appkit_window_ops_is_document_edited(self.as_ptr()) }
    }

    pub fn set_document_edited(&self, edited: bool) {
        unsafe { native::appkit_window_ops_set_document_edited(self.as_ptr(), edited) };
    }

    #[must_use]
    pub fn represented_filename(&self) -> Option<PathBuf> {
        let filename = unsafe { native::appkit_window_ops_represented_filename(self.as_ptr()) };
        let filename = NonNull::new(filename)?;
        let path = PathBuf::from(OsStr::from_bytes(
            unsafe { CStr::from_ptr(filename.as_ptr()) }.to_bytes(),
        ));
        unsafe { ffi::appkit_string_free(filename.as_ptr()) };
        Some(path)
    }

    pub fn set_represented_filename(&self, filename: Option<&Path>) -> Result<(), AppKitError> {
        let bytes = filename
            .map(|path| CString::new(path.as_os_str().as_bytes()))
            .transpose()
            .map_err(|error| AppKitError::InvalidArgument(error.to_string()))?;
        check_window_operation(unsafe {
            native::appkit_window_ops_set_represented_filename(
                self.as_ptr(),
                bytes.as_ref().map_or(ptr::null(), |value| value.as_ptr()),
                bytes.as_ref().map_or(0, |value| value.as_bytes().len()),
            )
        })
    }

    pub fn order_out(&self) {
        unsafe { native::appkit_window_ops_order_out(self.as_ptr()) };
    }

    pub fn order_back(&self) {
        unsafe { native::appkit_window_ops_order_back(self.as_ptr()) };
    }

    pub fn order_window(
        &self,
        mode: WindowOrderingMode,
        relative_to: i64,
    ) -> Result<(), AppKitError> {
        check_window_operation(unsafe {
            native::appkit_window_ops_order_window(self.as_ptr(), mode.raw(), relative_to)
        })
    }

    #[must_use]
    pub fn window_number(&self) -> i64 {
        unsafe { native::appkit_window_ops_window_number(self.as_ptr()) }
    }

    pub fn display_if_needed(&self) {
        unsafe { native::appkit_window_ops_display_if_needed(self.as_ptr()) };
    }

    pub fn set_content_view_needs_display(&self, needs_display: bool) {
        unsafe {
            native::appkit_window_ops_set_content_view_needs_display(self.as_ptr(), needs_display);
        };
    }

    pub fn invalidate_cursor_rects_for_view(&self, view: &View) {
        unsafe {
            native::appkit_window_ops_invalidate_cursor_rects_for_view(
                self.as_ptr(),
                view.as_ptr(),
            );
        };
    }

    pub fn resign_key_window(&self) {
        unsafe { native::appkit_window_ops_resign_key_window(self.as_ptr()) };
    }

    #[must_use]
    pub fn is_released_when_closed(&self) -> bool {
        unsafe { native::appkit_window_ops_is_released_when_closed(self.as_ptr()) }
    }

    pub fn retain_when_closed(&self) {
        unsafe { native::appkit_window_ops_retain_when_closed(self.as_ptr()) };
    }
}

impl View {
    pub fn set_needs_display(&self, needs_display: bool) {
        unsafe { native::appkit_window_ops_view_set_needs_display(self.as_ptr(), needs_display) };
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use super::{
        deliver_sheet_completion, register_sheet_completion, release_sheet_completion,
        SheetCompletionState, SheetResponse,
    };
    use crate::callbacks::CallbackRegistration;
    use crate::MainThreadMarker;

    #[test]
    fn sheet_completion_survives_the_returned_handle() {
        let main_thread = unsafe { MainThreadMarker::new_unchecked() };
        let received = Rc::new(Cell::new(None));
        let result = Rc::clone(&received);
        let completion = register_sheet_completion(
            main_thread,
            Box::new(move |response| result.set(Some(response))),
        );
        let token = completion.token;
        drop(completion);

        assert!(
            CallbackRegistration::<SheetCompletionState>::snapshot(token, main_thread).is_some()
        );
        deliver_sheet_completion(token, SheetResponse::Ok, main_thread);
        assert_eq!(received.get(), Some(SheetResponse::Ok));
        assert!(
            CallbackRegistration::<SheetCompletionState>::snapshot(token, main_thread).is_none()
        );
        deliver_sheet_completion(token, SheetResponse::Cancel, main_thread);
        assert_eq!(received.get(), Some(SheetResponse::Ok));
    }

    #[test]
    fn sheet_completion_handles_reentry_and_explicit_cancellation() {
        let main_thread = unsafe { MainThreadMarker::new_unchecked() };
        let token = Rc::new(Cell::new(None));
        let nested_token = Rc::clone(&token);
        let count = Rc::new(Cell::new(0));
        let invoked = Rc::clone(&count);
        let completion = register_sheet_completion(
            main_thread,
            Box::new(move |_| {
                invoked.set(invoked.get() + 1);
                deliver_sheet_completion(
                    nested_token.get().expect("the callback has a token"),
                    SheetResponse::Other(42),
                    main_thread,
                );
            }),
        );
        token.set(Some(completion.token));
        drop(completion);
        deliver_sheet_completion(token.get().unwrap(), SheetResponse::Ok, main_thread);
        assert_eq!(count.get(), 1);
        assert!(!release_sheet_completion(token.get().unwrap()));

        let cancelled = register_sheet_completion(
            main_thread,
            Box::new(move |_| panic!("a cancelled sheet delivered a callback")),
        );
        let cancelled_token = cancelled.token;
        assert!(cancelled.cancel());
        drop(cancelled);
        deliver_sheet_completion(cancelled_token, SheetResponse::Ok, main_thread);
        assert!(CallbackRegistration::<SheetCompletionState>::snapshot(
            cancelled_token,
            main_thread
        )
        .is_none());
    }
}
