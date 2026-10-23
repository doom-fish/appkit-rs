use core::ffi::c_void;

use apple_cf::cg::CGRect;

use crate::ffi;
use crate::main_thread::MainThreadMarker;
use crate::object::Handle;
use crate::util::{read_rect, take_string};

#[derive(Debug, Clone)]
pub struct Screen {
    handle: Handle,
}

impl Screen {
    #[must_use]
    pub fn main(_main_thread: MainThreadMarker) -> Option<Self> {
        unsafe { Handle::from_retained(ffi::appkit_screen_main()) }.map(Self::from_handle)
    }

    #[must_use]
    pub fn screens(_main_thread: MainThreadMarker) -> Vec<Self> {
        let count = unsafe { ffi::appkit_screen_count() };
        (0..count)
            .filter_map(|index| unsafe { Handle::from_retained(ffi::appkit_screen_at(index)) })
            .map(Self::from_handle)
            .collect()
    }

    pub(crate) const fn from_handle(handle: Handle) -> Self {
        Self { handle }
    }

    #[must_use]
    pub const fn as_ptr(&self) -> *mut c_void {
        self.handle.as_ptr()
    }

    #[must_use]
    pub fn frame(&self) -> CGRect {
        read_rect(|out| unsafe { ffi::appkit_screen_frame(self.as_ptr(), out) })
    }

    #[must_use]
    pub fn visible_frame(&self) -> CGRect {
        read_rect(|out| unsafe { ffi::appkit_screen_visible_frame(self.as_ptr(), out) })
    }

    #[must_use]
    pub fn backing_scale_factor(&self) -> f64 {
        unsafe { ffi::appkit_screen_backing_scale_factor(self.as_ptr()) }
    }

    #[must_use]
    pub fn display_id(&self) -> Option<u32> {
        let mut display = 0;
        unsafe { ffi::appkit_screen_display_id(self.as_ptr(), &raw mut display) }.then_some(display)
    }

    #[must_use]
    pub fn localized_name(&self) -> String {
        unsafe { take_string(ffi::appkit_screen_localized_name(self.as_ptr())) }.unwrap_or_default()
    }

    #[must_use]
    pub fn maximum_frames_per_second(&self) -> i64 {
        unsafe { ffi::appkit_screen_maximum_frames_per_second(self.as_ptr()) }
    }
}
