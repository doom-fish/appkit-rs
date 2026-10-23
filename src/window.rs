use core::ffi::c_void;
use core::ptr::NonNull;

use apple_cf::cg::CGRect;

use crate::appearance::AppearanceName;
use crate::error::AppKitError;
use crate::ffi;
use crate::main_thread::MainThreadMarker;
use crate::object::Handle;
use crate::screen::Screen;
use crate::util::{c_string, read_rect, take_string};
use crate::view::View;
use crate::window_ops::check_window_operation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TitleVisibility {
    Visible,
    Hidden,
    Unknown(i64),
}

impl TitleVisibility {
    const fn raw(self) -> i64 {
        match self {
            Self::Visible => 0,
            Self::Hidden => 1,
            Self::Unknown(value) => value,
        }
    }

    const fn from_raw(raw: i64) -> Self {
        match raw {
            0 => Self::Visible,
            1 => Self::Hidden,
            _ => Self::Unknown(raw),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TitlebarSeparatorStyle {
    Automatic,
    None,
    Line,
    Shadow,
    Unknown(i64),
}

impl TitlebarSeparatorStyle {
    const fn raw(self) -> i64 {
        match self {
            Self::Automatic => 0,
            Self::None => 1,
            Self::Line => 2,
            Self::Shadow => 3,
            Self::Unknown(value) => value,
        }
    }

    const fn from_raw(raw: i64) -> Self {
        match raw {
            1 => Self::None,
            2 => Self::Line,
            3 => Self::Shadow,
            0 => Self::Automatic,
            _ => Self::Unknown(raw),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimationBehavior {
    Default,
    None,
    DocumentWindow,
    UtilityWindow,
    AlertPanel,
    Unknown(i64),
}

impl AnimationBehavior {
    const fn raw(self) -> i64 {
        match self {
            Self::Default => 0,
            Self::None => 2,
            Self::DocumentWindow => 3,
            Self::UtilityWindow => 4,
            Self::AlertPanel => 5,
            Self::Unknown(value) => value,
        }
    }

    const fn from_raw(raw: i64) -> Self {
        match raw {
            2 => Self::None,
            3 => Self::DocumentWindow,
            4 => Self::UtilityWindow,
            5 => Self::AlertPanel,
            0 => Self::Default,
            _ => Self::Unknown(raw),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Window {
    handle: Handle,
}

impl Window {
    pub fn new(
        _main_thread: MainThreadMarker,
        content_rect: CGRect,
        title: &str,
    ) -> Result<Self, AppKitError> {
        let title = c_string(title)?;
        let window = unsafe {
            ffi::appkit_window_new(
                content_rect.origin.x,
                content_rect.origin.y,
                content_rect.size.width,
                content_rect.size.height,
                title.as_ptr(),
            )
        };
        let handle = unsafe { Handle::from_retained(window) };
        Ok(Self {
            handle: handle.expect("NSWindow gives a window"),
        })
    }

    #[must_use]
    pub unsafe fn from_raw_borrowed(
        window: NonNull<c_void>,
        _main_thread: MainThreadMarker,
    ) -> Self {
        Self {
            handle: unsafe { Handle::retain(window) },
        }
    }

    pub(crate) const fn from_handle(handle: Handle) -> Self {
        Self { handle }
    }

    #[must_use]
    pub const fn as_ptr(&self) -> *mut c_void {
        self.handle.as_ptr()
    }

    #[must_use]
    pub fn content_view(&self) -> Option<View> {
        unsafe { Handle::from_retained(ffi::appkit_window_content_view(self.as_ptr())) }
            .map(View::from_handle)
    }

    #[must_use]
    pub fn appearance(&self) -> Option<AppearanceName> {
        AppearanceName::from_raw(unsafe { ffi::appkit_window_appearance(self.as_ptr()) })
    }

    pub fn set_appearance(&self, appearance: Option<AppearanceName>) -> Result<(), AppKitError> {
        if matches!(
            appearance,
            Some(
                AppearanceName::AccessibilityHighContrastAqua
                    | AppearanceName::AccessibilityHighContrastDarkAqua
                    | AppearanceName::AccessibilityHighContrastVibrantLight
                    | AppearanceName::AccessibilityHighContrastVibrantDark
            )
        ) {
            return Err(AppKitError::InvalidArgument(
                "high-contrast appearances are match-only".to_owned(),
            ));
        }
        check_window_operation(unsafe {
            ffi::window_ops::appkit_window_ops_set_appearance(
                self.as_ptr(),
                appearance.map_or(-1, AppearanceName::raw),
            )
        })
    }

    #[must_use]
    pub fn title(&self) -> String {
        unsafe { take_string(ffi::appkit_window_title(self.as_ptr())) }.unwrap_or_default()
    }

    pub fn set_title(&self, title: &str) -> Result<(), AppKitError> {
        let title = c_string(title)?;
        unsafe { ffi::appkit_window_set_title(self.as_ptr(), title.as_ptr()) };
        Ok(())
    }

    #[must_use]
    pub fn title_visibility(&self) -> TitleVisibility {
        TitleVisibility::from_raw(unsafe { ffi::appkit_window_title_visibility(self.as_ptr()) })
    }

    pub fn set_title_visibility(&self, visibility: TitleVisibility) {
        unsafe { ffi::appkit_window_set_title_visibility(self.as_ptr(), visibility.raw()) };
    }

    #[must_use]
    pub fn titlebar_separator_style(&self) -> TitlebarSeparatorStyle {
        TitlebarSeparatorStyle::from_raw(unsafe {
            ffi::appkit_window_titlebar_separator_style(self.as_ptr())
        })
    }

    pub fn set_titlebar_separator_style(&self, style: TitlebarSeparatorStyle) {
        unsafe { ffi::appkit_window_set_titlebar_separator_style(self.as_ptr(), style.raw()) };
    }

    #[must_use]
    pub fn frame(&self) -> CGRect {
        read_rect(|out| unsafe { ffi::appkit_window_frame(self.as_ptr(), out) })
    }

    #[must_use]
    pub fn content_layout_rect(&self) -> CGRect {
        read_rect(|out| unsafe { ffi::appkit_window_content_layout_rect(self.as_ptr(), out) })
    }

    #[must_use]
    pub fn backing_scale_factor(&self) -> f64 {
        unsafe { ffi::appkit_window_backing_scale_factor(self.as_ptr()) }
    }

    #[must_use]
    pub fn screen(&self) -> Option<Screen> {
        unsafe { Handle::from_retained(ffi::appkit_window_screen(self.as_ptr())) }
            .map(Screen::from_handle)
    }

    #[must_use]
    pub fn alpha_value(&self) -> f64 {
        unsafe { ffi::appkit_window_alpha_value(self.as_ptr()) }
    }

    pub fn set_alpha_value(&self, alpha: f64) {
        unsafe { ffi::appkit_window_set_alpha_value(self.as_ptr(), alpha) };
    }

    pub fn animate_alpha_value(&self, alpha: f64) {
        unsafe { ffi::appkit_window_animate_alpha_value(self.as_ptr(), alpha) };
    }

    #[must_use]
    pub fn animation_behavior(&self) -> AnimationBehavior {
        AnimationBehavior::from_raw(unsafe { ffi::appkit_window_animation_behavior(self.as_ptr()) })
    }

    pub fn set_animation_behavior(&self, behavior: AnimationBehavior) {
        unsafe { ffi::appkit_window_set_animation_behavior(self.as_ptr(), behavior.raw()) };
    }

    pub fn order_front(&self) {
        unsafe { ffi::appkit_window_order_front(self.as_ptr()) };
    }

    pub fn make_key_and_order_front(&self) {
        unsafe { ffi::appkit_window_make_key_and_order_front(self.as_ptr()) };
    }

    pub fn close(&self) {
        unsafe { ffi::appkit_window_close(self.as_ptr()) };
    }
}
