use core::ffi::c_void;

use crate::ffi::blur as native;
use crate::main_thread::MainThreadMarker;
use crate::object::Handle;
use crate::view::View;
use crate::window::Window;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VisualEffectMaterial {
    UnderWindowBackground,
    WindowBackground,
    Sidebar,
    Titlebar,
    Selection,
    ContentBackground,
}

impl VisualEffectMaterial {
    const fn from_raw(raw: i32) -> Option<Self> {
        match raw {
            0 => Some(Self::UnderWindowBackground),
            1 => Some(Self::WindowBackground),
            2 => Some(Self::Sidebar),
            3 => Some(Self::Titlebar),
            4 => Some(Self::Selection),
            5 => Some(Self::ContentBackground),
            _ => None,
        }
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VisualEffectBlendingMode {
    BehindWindow,
    WithinWindow,
}

impl VisualEffectBlendingMode {
    const fn from_raw(raw: i32) -> Option<Self> {
        match raw {
            0 => Some(Self::BehindWindow),
            1 => Some(Self::WithinWindow),
            _ => None,
        }
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VisualEffectState {
    FollowsWindowActiveState,
    Active,
    Inactive,
}

impl VisualEffectState {
    const fn from_raw(raw: i32) -> Option<Self> {
        match raw {
            0 => Some(Self::FollowsWindowActiveState),
            1 => Some(Self::Active),
            2 => Some(Self::Inactive),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct VisualEffectView {
    handle: Handle,
}

impl VisualEffectView {
    #[must_use]
    pub fn new_for_window(_main_thread: MainThreadMarker, window: &Window) -> Option<Self> {
        let handle = unsafe {
            Handle::from_retained(native::appkit_blur_new_for_window(window.as_ptr(), false))
        }?;
        Some(Self { handle })
    }

    #[must_use]
    pub fn new_clear_for_window(_main_thread: MainThreadMarker, window: &Window) -> Option<Self> {
        let handle = unsafe {
            Handle::from_retained(native::appkit_blur_new_for_window(window.as_ptr(), true))
        }?;
        Some(Self { handle })
    }

    #[must_use]
    pub const fn as_ptr(&self) -> *mut c_void {
        self.handle.as_ptr()
    }

    #[must_use]
    pub fn material(&self) -> Option<VisualEffectMaterial> {
        VisualEffectMaterial::from_raw(unsafe { native::appkit_blur_material(self.as_ptr()) })
    }

    pub fn set_material(&self, material: VisualEffectMaterial) {
        unsafe { native::appkit_blur_set_material(self.as_ptr(), material as i32) };
    }

    #[must_use]
    pub fn blending_mode(&self) -> Option<VisualEffectBlendingMode> {
        VisualEffectBlendingMode::from_raw(unsafe {
            native::appkit_blur_blending_mode(self.as_ptr())
        })
    }

    pub fn set_blending_mode(&self, mode: VisualEffectBlendingMode) {
        unsafe { native::appkit_blur_set_blending_mode(self.as_ptr(), mode as i32) };
    }

    #[must_use]
    pub fn state(&self) -> Option<VisualEffectState> {
        VisualEffectState::from_raw(unsafe { native::appkit_blur_state(self.as_ptr()) })
    }

    pub fn set_state(&self, state: VisualEffectState) {
        unsafe { native::appkit_blur_set_state(self.as_ptr(), state as i32) };
    }

    #[must_use]
    pub fn is_emphasized(&self) -> bool {
        unsafe { native::appkit_blur_is_emphasized(self.as_ptr()) }
    }

    pub fn set_emphasized(&self, emphasized: bool) {
        unsafe { native::appkit_blur_set_emphasized(self.as_ptr(), emphasized) };
    }

    #[must_use]
    pub fn attach_below(&self, hosted_view: &View) -> bool {
        unsafe { native::appkit_blur_attach_below(self.as_ptr(), hosted_view.as_ptr()) }
    }

    #[must_use]
    pub fn is_clear_blur(&self) -> bool {
        unsafe { native::appkit_blur_is_clear_blur(self.as_ptr()) }
    }

    pub fn detach(&self) {
        unsafe { native::appkit_blur_detach(self.as_ptr()) };
    }
}

impl Drop for VisualEffectView {
    fn drop(&mut self) {
        self.detach();
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use super::{native, Handle, VisualEffectView};

    #[test]
    fn visual_effect_handle_does_not_retain_a_window() {
        assert_eq!(size_of::<VisualEffectView>(), size_of::<Handle>());
    }

    #[test]
    fn saturation_filter_detection_preserves_other_filters() {
        for (description, expected) in [
            ("colorSaturate", true),
            ("inputSaturation = 1.2", true),
            ("GaussianBlur", false),
            ("colorTint", false),
        ] {
            let description = CString::new(description).expect("the text has no null byte");
            assert_eq!(
                unsafe { native::appkit_blur_description_is_saturation(description.as_ptr()) },
                expected
            );
        }
    }
}
