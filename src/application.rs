use core::ffi::{c_char, c_void};
use core::ops::{BitOr, BitOrAssign};
use core::ptr::NonNull;
use std::ffi::CString;

use crate::appearance::AppearanceName;
use crate::error::AppKitError;
use crate::event::Event;
use crate::ffi;
use crate::ffi::application as native;
use crate::image::Image;
use crate::main_thread::MainThreadMarker;
use crate::object::Handle;
use crate::util::{c_string, optional_c_string};
use crate::window::Window;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AboutPanelOptions {
    pub application_name: Option<String>,
    pub application_version: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActivationPolicy {
    Regular,
    Accessory,
    Prohibited,
}

impl ActivationPolicy {
    const fn raw(self) -> i64 {
        match self {
            Self::Regular => 0,
            Self::Accessory => 1,
            Self::Prohibited => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct PresentationOptions(u64);

impl PresentationOptions {
    pub const NONE: Self = Self(0);
    pub const AUTO_HIDE_DOCK: Self = Self(1);
    pub const HIDE_DOCK: Self = Self(1 << 1);
    pub const AUTO_HIDE_MENU_BAR: Self = Self(1 << 2);
    pub const HIDE_MENU_BAR: Self = Self(1 << 3);
    pub const DISABLE_APPLE_MENU: Self = Self(1 << 4);
    pub const DISABLE_PROCESS_SWITCHING: Self = Self(1 << 5);
    pub const DISABLE_FORCE_QUIT: Self = Self(1 << 6);
    pub const DISABLE_SESSION_TERMINATION: Self = Self(1 << 7);
    pub const DISABLE_HIDE_APPLICATION: Self = Self(1 << 8);
    pub const DISABLE_MENU_BAR_TRANSPARENCY: Self = Self(1 << 9);
    pub const FULL_SCREEN: Self = Self(1 << 10);
    pub const AUTO_HIDE_TOOLBAR: Self = Self(1 << 11);
    pub const DISABLE_CURSOR_LOCATION_ASSISTANCE: Self = Self(1 << 12);

    #[must_use]
    pub const fn from_bits_retain(bits: u64) -> Self {
        Self(bits)
    }

    #[must_use]
    pub const fn bits(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn is_valid(self) -> bool {
        let dock = self.0 & (Self::AUTO_HIDE_DOCK.0 | Self::HIDE_DOCK.0);
        let menu = self.0 & (Self::AUTO_HIDE_MENU_BAR.0 | Self::HIDE_MENU_BAR.0);
        let requires_dock = Self::DISABLE_PROCESS_SWITCHING.0
            | Self::DISABLE_FORCE_QUIT.0
            | Self::DISABLE_SESSION_TERMINATION.0
            | Self::DISABLE_MENU_BAR_TRANSPARENCY.0;
        if dock == Self::AUTO_HIDE_DOCK.0 | Self::HIDE_DOCK.0
            || menu == Self::AUTO_HIDE_MENU_BAR.0 | Self::HIDE_MENU_BAR.0
        {
            return false;
        }
        if self.0 & Self::HIDE_MENU_BAR.0 != 0 && dock != Self::HIDE_DOCK.0 {
            return false;
        }
        if self.0 & requires_dock != 0 && dock == 0 {
            return false;
        }
        if self.0 & Self::AUTO_HIDE_TOOLBAR.0 != 0
            && self.0 & (Self::FULL_SCREEN.0 | Self::AUTO_HIDE_MENU_BAR.0)
                != Self::FULL_SCREEN.0 | Self::AUTO_HIDE_MENU_BAR.0
        {
            return false;
        }
        true
    }
}

impl BitOr for PresentationOptions {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for PresentationOptions {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UserAttentionType {
    Informational,
    Critical,
}

#[derive(Debug, Clone)]
pub struct Application {
    handle: Handle,
}

impl Application {
    #[must_use]
    pub fn shared(_main_thread: MainThreadMarker) -> Self {
        let handle = unsafe { Handle::from_retained(ffi::appkit_application_shared()) };
        Self {
            handle: handle.expect("every process has a shared application"),
        }
    }

    #[must_use]
    pub const fn as_ptr(&self) -> *mut c_void {
        self.handle.as_ptr()
    }

    #[must_use]
    pub fn set_activation_policy(&self, policy: ActivationPolicy) -> bool {
        unsafe { ffi::appkit_application_set_activation_policy(self.as_ptr(), policy.raw()) }
    }

    pub fn activate(&self) {
        unsafe { ffi::appkit_application_activate(self.as_ptr()) };
    }

    pub fn run(&self) {
        unsafe { ffi::appkit_application_run(self.as_ptr()) };
    }

    pub fn terminate(&self) {
        unsafe { ffi::appkit_application_terminate(self.as_ptr()) };
    }

    pub fn set_application_icon_image(&self, image: Option<&Image>) {
        let image = image.map_or(core::ptr::null_mut(), Image::as_ptr);
        unsafe { ffi::appkit_application_set_application_icon_image(self.as_ptr(), image) };
    }

    #[must_use]
    pub fn key_window(&self) -> Option<Window> {
        unsafe { Handle::from_retained(native::appkit_application_key_window(self.as_ptr())) }
            .map(Window::from_handle)
    }

    #[must_use]
    pub fn main_window(&self) -> Option<Window> {
        unsafe { Handle::from_retained(native::appkit_application_main_window(self.as_ptr())) }
            .map(Window::from_handle)
    }

    #[must_use]
    pub fn ordered_windows(&self) -> Vec<Window> {
        let Some(array) = (unsafe {
            Handle::from_retained(native::appkit_application_ordered_windows(self.as_ptr()))
        }) else {
            return Vec::new();
        };
        let count = unsafe { native::appkit_application_window_array_count(array.as_ptr()) };
        (0..count)
            .filter_map(|index| unsafe {
                Handle::from_retained(native::appkit_application_window_array_at(
                    array.as_ptr(),
                    index,
                ))
            })
            .map(Window::from_handle)
            .collect()
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        unsafe { native::appkit_application_is_active(self.as_ptr()) }
    }

    #[must_use]
    pub fn is_hidden(&self) -> bool {
        unsafe { native::appkit_application_is_hidden(self.as_ptr()) }
    }

    pub fn activate_ignoring_other_apps(&self, ignore: bool) {
        unsafe { native::appkit_application_activate_ignoring_other_apps(self.as_ptr(), ignore) };
    }

    pub fn hide(&self) {
        unsafe { native::appkit_application_hide(self.as_ptr()) };
    }

    pub fn unhide(&self) {
        unsafe { native::appkit_application_unhide(self.as_ptr()) };
    }

    pub fn hide_other_applications(&self) {
        unsafe { native::appkit_application_hide_other_applications(self.as_ptr()) };
    }

    pub fn unhide_all_applications(&self) {
        unsafe { native::appkit_application_unhide_all_applications(self.as_ptr()) };
    }

    #[must_use]
    pub fn presentation_options(&self) -> PresentationOptions {
        PresentationOptions::from_bits_retain(unsafe {
            native::appkit_application_presentation_options(self.as_ptr())
        })
    }

    #[must_use]
    pub fn set_presentation_options(&self, options: PresentationOptions) -> bool {
        options.is_valid()
            && unsafe {
                native::appkit_application_set_presentation_options(self.as_ptr(), options.bits())
            }
    }

    #[must_use]
    pub fn request_user_attention(&self, kind: UserAttentionType) -> i64 {
        let kind = match kind {
            UserAttentionType::Informational => 0,
            UserAttentionType::Critical => 1,
        };
        unsafe { native::appkit_application_request_user_attention(self.as_ptr(), kind) }
    }

    pub fn cancel_user_attention(&self, request: i64) {
        unsafe { native::appkit_application_cancel_user_attention(self.as_ptr(), request) };
    }

    #[must_use]
    pub fn appearance(&self) -> Option<AppearanceName> {
        AppearanceName::from_raw(unsafe { native::appkit_application_appearance(self.as_ptr()) })
    }

    #[must_use]
    pub fn effective_appearance(&self) -> Option<AppearanceName> {
        AppearanceName::from_raw(unsafe {
            native::appkit_application_effective_appearance(self.as_ptr())
        })
    }

    pub fn set_appearance(&self, appearance: Option<AppearanceName>) -> bool {
        appearance.is_none_or(AppearanceName::is_settable)
            && unsafe {
                native::appkit_application_set_appearance(
                    self.as_ptr(),
                    appearance.map_or(-1, AppearanceName::raw),
                )
            }
    }

    pub fn order_front_character_palette(&self, sender: Option<&Window>) {
        let sender = sender.map_or(core::ptr::null_mut(), Window::as_ptr);
        unsafe { native::appkit_application_order_front_character_palette(self.as_ptr(), sender) };
    }

    pub fn change_windows_item(&self, window: &Window, title: &str) -> Result<bool, AppKitError> {
        let title = c_string(title)?;
        Ok(unsafe {
            native::appkit_application_change_windows_item(
                self.as_ptr(),
                window.as_ptr(),
                title.as_ptr(),
            )
        })
    }

    #[must_use]
    pub fn current_event(&self) -> Option<Event> {
        let main_thread = MainThreadMarker::new()?;
        let event =
            NonNull::new(unsafe { native::appkit_application_current_event(self.as_ptr()) })?;
        Some(unsafe { Event::from_raw_retained(event, main_thread) })
    }

    pub fn order_front_standard_about_panel(
        &self,
        options: &AboutPanelOptions,
    ) -> Result<(), AppKitError> {
        let application_name = optional_c_string(options.application_name.as_deref())?;
        let application_version = optional_c_string(options.application_version.as_deref())?;
        let version = optional_c_string(options.version.as_deref())?;
        unsafe {
            ffi::appkit_application_order_front_standard_about_panel(
                self.as_ptr(),
                pointer(application_name.as_ref()),
                pointer(application_version.as_ref()),
                pointer(version.as_ref()),
            );
        }
        Ok(())
    }
}

fn pointer(value: Option<&CString>) -> *const c_char {
    value.map_or(core::ptr::null(), |value| value.as_ptr())
}

#[cfg(test)]
mod tests {
    use super::{AppKitError, Application, PresentationOptions, Window};
    use crate::ffi::application as native;

    #[test]
    fn presentation_options_follow_appkit_rules() {
        let valid = [
            PresentationOptions::NONE,
            PresentationOptions::AUTO_HIDE_DOCK,
            PresentationOptions::AUTO_HIDE_MENU_BAR,
            PresentationOptions::AUTO_HIDE_MENU_BAR | PresentationOptions::DISABLE_APPLE_MENU,
            PresentationOptions::AUTO_HIDE_DOCK | PresentationOptions::AUTO_HIDE_MENU_BAR,
            PresentationOptions::HIDE_DOCK | PresentationOptions::HIDE_MENU_BAR,
            PresentationOptions::HIDE_DOCK | PresentationOptions::DISABLE_PROCESS_SWITCHING,
            PresentationOptions::FULL_SCREEN
                | PresentationOptions::AUTO_HIDE_DOCK
                | PresentationOptions::AUTO_HIDE_MENU_BAR
                | PresentationOptions::AUTO_HIDE_TOOLBAR,
        ];
        let invalid = [
            PresentationOptions::AUTO_HIDE_DOCK | PresentationOptions::HIDE_DOCK,
            PresentationOptions::HIDE_DOCK
                | PresentationOptions::AUTO_HIDE_MENU_BAR
                | PresentationOptions::HIDE_MENU_BAR,
            PresentationOptions::HIDE_MENU_BAR,
            PresentationOptions::AUTO_HIDE_DOCK | PresentationOptions::HIDE_MENU_BAR,
            PresentationOptions::DISABLE_PROCESS_SWITCHING,
            PresentationOptions::DISABLE_FORCE_QUIT,
            PresentationOptions::DISABLE_SESSION_TERMINATION,
            PresentationOptions::DISABLE_MENU_BAR_TRANSPARENCY,
            PresentationOptions::AUTO_HIDE_TOOLBAR,
            PresentationOptions::FULL_SCREEN | PresentationOptions::AUTO_HIDE_TOOLBAR,
        ];
        for options in valid {
            assert!(options.is_valid(), "{options:?}");
        }
        for options in invalid {
            assert!(!options.is_valid(), "{options:?}");
        }
        for bits in 0..1 << 13 {
            let options = PresentationOptions::from_bits_retain(bits);
            assert_eq!(
                options.is_valid(),
                unsafe { native::appkit_application_validate_presentation_options(bits) },
                "{options:?}"
            );
        }
    }

    #[test]
    fn window_menu_titles_have_no_filename_flag() {
        let _: fn(&Application, &Window, &str) -> Result<bool, AppKitError> =
            Application::change_windows_item;
    }
}
