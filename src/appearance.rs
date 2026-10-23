use core::ffi::c_void;
use core::ptr::NonNull;

use crate::ffi::appearance as native;
use crate::main_thread::MainThreadMarker;
use crate::object::Handle;
use crate::view::View;
use crate::window::Window;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppearanceName {
    Aqua,
    DarkAqua,
    VibrantLight,
    VibrantDark,
    AccessibilityHighContrastAqua,
    AccessibilityHighContrastDarkAqua,
    AccessibilityHighContrastVibrantLight,
    AccessibilityHighContrastVibrantDark,
}

const NAMES: [AppearanceName; 8] = [
    AppearanceName::Aqua,
    AppearanceName::DarkAqua,
    AppearanceName::VibrantLight,
    AppearanceName::VibrantDark,
    AppearanceName::AccessibilityHighContrastAqua,
    AppearanceName::AccessibilityHighContrastDarkAqua,
    AppearanceName::AccessibilityHighContrastVibrantLight,
    AppearanceName::AccessibilityHighContrastVibrantDark,
];

impl AppearanceName {
    #[must_use]
    pub const fn native_name(self) -> &'static str {
        match self {
            Self::Aqua => "NSAppearanceNameAqua",
            Self::DarkAqua => "NSAppearanceNameDarkAqua",
            Self::VibrantLight => "NSAppearanceNameVibrantLight",
            Self::VibrantDark => "NSAppearanceNameVibrantDark",
            Self::AccessibilityHighContrastAqua => "NSAppearanceNameAccessibilityHighContrastAqua",
            Self::AccessibilityHighContrastDarkAqua => {
                "NSAppearanceNameAccessibilityHighContrastDarkAqua"
            }
            Self::AccessibilityHighContrastVibrantLight => {
                "NSAppearanceNameAccessibilityHighContrastVibrantLight"
            }
            Self::AccessibilityHighContrastVibrantDark => {
                "NSAppearanceNameAccessibilityHighContrastVibrantDark"
            }
        }
    }

    #[must_use]
    pub fn from_native_name(name: &str) -> Option<Self> {
        NAMES
            .iter()
            .copied()
            .find(|appearance| appearance.native_name() == name)
    }

    pub(crate) const fn raw(self) -> i32 {
        self as i32
    }

    pub(crate) fn from_raw(raw: i32) -> Option<Self> {
        usize::try_from(raw)
            .ok()
            .and_then(|index| NAMES.get(index).copied())
    }
}

#[derive(Debug, Clone)]
pub struct Appearance {
    handle: Handle,
}

impl Appearance {
    #[must_use]
    pub fn named(_main_thread: MainThreadMarker, name: AppearanceName) -> Option<Self> {
        unsafe { Handle::from_retained(native::appkit_appearance_named(name.raw())) }
            .map(|handle| Self { handle })
    }

    #[must_use]
    pub unsafe fn from_raw_borrowed(
        appearance: NonNull<c_void>,
        _main_thread: MainThreadMarker,
    ) -> Self {
        Self {
            handle: unsafe { Handle::retain(appearance) },
        }
    }

    #[must_use]
    pub const fn as_ptr(&self) -> *mut c_void {
        self.handle.as_ptr()
    }

    #[must_use]
    pub fn name(&self) -> Option<AppearanceName> {
        AppearanceName::from_raw(unsafe { native::appkit_appearance_name(self.as_ptr()) })
    }

    #[must_use]
    pub fn effective_for_window(window: &Window) -> Option<Self> {
        unsafe {
            Handle::from_retained(native::appkit_appearance_window_effective(window.as_ptr()))
        }
        .map(|handle| Self { handle })
    }

    #[must_use]
    pub fn effective_for_view(view: &View) -> Option<Self> {
        unsafe { Handle::from_retained(native::appkit_appearance_view_effective(view.as_ptr())) }
            .map(|handle| Self { handle })
    }

    #[must_use]
    pub fn view_override(view: &View) -> Option<Self> {
        unsafe { Handle::from_retained(native::appkit_appearance_view_override(view.as_ptr())) }
            .map(|handle| Self { handle })
    }

    pub fn set_view_override(view: &View, appearance: Option<AppearanceName>) {
        unsafe {
            native::appkit_appearance_set_view_override(
                view.as_ptr(),
                appearance.map_or(-1, AppearanceName::raw),
            );
        }
    }
}
