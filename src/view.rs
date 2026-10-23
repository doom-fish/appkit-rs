use core::ffi::c_void;
use core::ptr::NonNull;

use apple_cf::cg::CGRect;

use crate::ffi;
use crate::main_thread::MainThreadMarker;
use crate::object::Handle;
use crate::util::read_rect;
use crate::window::Window;

#[derive(Debug, Clone)]
pub struct View {
    handle: Handle,
}

impl View {
    #[must_use]
    pub unsafe fn from_raw_borrowed(view: NonNull<c_void>, _main_thread: MainThreadMarker) -> Self {
        Self {
            handle: unsafe { Handle::retain(view) },
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
    pub fn window(&self) -> Option<Window> {
        unsafe { Handle::from_retained(ffi::appkit_view_window(self.as_ptr())) }.map(Window::from_handle)
    }

    #[must_use]
    pub fn bounds(&self) -> CGRect {
        read_rect(|out| unsafe { ffi::appkit_view_bounds(self.as_ptr(), out) })
    }

    #[must_use]
    pub fn frame(&self) -> CGRect {
        read_rect(|out| unsafe { ffi::appkit_view_frame(self.as_ptr(), out) })
    }

    #[must_use]
    pub fn layer_ptr(&self) -> Option<NonNull<c_void>> {
        NonNull::new(unsafe { ffi::appkit_view_layer(self.as_ptr()) })
    }

    #[must_use]
    pub fn wants_layer(&self) -> bool {
        unsafe { ffi::appkit_view_wants_layer(self.as_ptr()) }
    }

    pub fn set_wants_layer(&self, wants: bool) {
        unsafe { ffi::appkit_view_set_wants_layer(self.as_ptr(), wants) };
    }
}
