use core::ffi::c_void;
use core::ptr::NonNull;

use crate::ffi;

#[derive(Debug)]
pub struct Handle(NonNull<c_void>);

impl Handle {
    pub unsafe fn from_retained(object: *mut c_void) -> Option<Self> {
        NonNull::new(object).map(Self)
    }

    pub unsafe fn retain(object: NonNull<c_void>) -> Self {
        unsafe { ffi::appkit_retain(object.as_ptr()) };
        Self(object)
    }

    pub const fn as_ptr(&self) -> *mut c_void {
        self.0.as_ptr()
    }
}

impl Clone for Handle {
    fn clone(&self) -> Self {
        unsafe { Self::retain(self.0) }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { ffi::appkit_release(self.0.as_ptr()) };
    }
}
