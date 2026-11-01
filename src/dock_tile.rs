use core::ffi::c_void;

use crate::error::AppKitError;
use crate::ffi::dock_tile as ffi;
use crate::object::Handle;
use crate::util::{optional_c_string, take_string};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DockProgress {
    Fraction(f64),
    Busy,
}

#[derive(Debug, Clone)]
pub struct DockTile {
    pub(crate) handle: Handle,
}

impl DockTile {
    #[must_use]
    pub const fn as_ptr(&self) -> *mut c_void {
        self.handle.as_ptr()
    }

    pub fn set_progress(&self, progress: Option<DockProgress>) -> Result<(), AppKitError> {
        let (kind, fraction) = match progress {
            None => (0, 0.0),
            Some(DockProgress::Fraction(value)) => {
                if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                    return Err(AppKitError::InvalidArgument(
                        "a Dock progress fraction must be finite and between zero and one".into(),
                    ));
                }
                (1, value)
            }
            Some(DockProgress::Busy) => (2, 0.0),
        };
        unsafe { ffi::appkit_dock_tile_set_progress(self.as_ptr(), kind, fraction) }
            .then_some(())
            .ok_or_else(|| AppKitError::InvalidArgument("Dock progress".into()))
    }

    pub fn set_badge_label(&self, label: Option<&str>) -> Result<(), AppKitError> {
        let label = optional_c_string(label)?;
        unsafe {
            ffi::appkit_dock_tile_set_badge_label(
                self.as_ptr(),
                label.as_ref().map_or(core::ptr::null(), |label| label.as_ptr()),
            )
        }
        .then_some(())
        .ok_or_else(|| AppKitError::InvalidArgument("Dock badge label".into()))
    }

    #[must_use]
    pub fn badge_label(&self) -> Option<String> {
        unsafe { take_string(ffi::appkit_dock_tile_badge_label(self.as_ptr())) }
    }

    pub fn display(&self) {
        unsafe { ffi::appkit_dock_tile_display(self.as_ptr()) };
    }
}
