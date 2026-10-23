use core::ffi::{c_char, c_void};
use std::ffi::CString;

use crate::error::AppKitError;
use crate::ffi;
use crate::image::Image;
use crate::main_thread::MainThreadMarker;
use crate::object::Handle;
use crate::util::optional_c_string;

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

    pub fn order_front_standard_about_panel(&self, options: &AboutPanelOptions) -> Result<(), AppKitError> {
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
