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
            Self::AccessibilityHighContrastAqua => "NSAppearanceNameAccessibilityAqua",
            Self::AccessibilityHighContrastDarkAqua => "NSAppearanceNameAccessibilityDarkAqua",
            Self::AccessibilityHighContrastVibrantLight => {
                "NSAppearanceNameAccessibilityVibrantLight"
            }
            Self::AccessibilityHighContrastVibrantDark => {
                "NSAppearanceNameAccessibilityVibrantDark"
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

    pub(crate) const fn is_settable(self) -> bool {
        matches!(
            self,
            Self::Aqua | Self::DarkAqua | Self::VibrantLight | Self::VibrantDark
        )
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
        if !name.is_settable() {
            return None;
        }
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

    pub fn set_view_override(view: &View, appearance: Option<AppearanceName>) -> bool {
        appearance.is_none_or(AppearanceName::is_settable)
            && unsafe {
                native::appkit_appearance_set_view_override(
                    view.as_ptr(),
                    appearance.map_or(-1, AppearanceName::raw),
                )
            }
    }
}

#[cfg(test)]
mod tests {
    use super::{native, AppearanceName, NAMES};
    use crate::util::take_string;

    #[test]
    fn appearance_names_match_the_sdk() {
        for name in NAMES {
            let raw = unsafe { take_string(native::appkit_appearance_name_raw_value(name.raw())) }
                .expect("the SDK has an appearance name");
            assert_eq!(name.native_name(), raw);
            assert_eq!(AppearanceName::from_native_name(&raw), Some(name));
        }
        assert_eq!(
            AppearanceName::from_native_name("NSAppearanceNameAccessibilityHighContrastAqua"),
            None
        );
    }

    #[test]
    fn match_only_appearances_cannot_be_set() {
        for name in &NAMES[..4] {
            assert!(name.is_settable());
        }
        for name in &NAMES[4..] {
            assert!(!name.is_settable());
            assert!(unsafe { native::appkit_appearance_named(name.raw()) }.is_null());
        }
    }
}
