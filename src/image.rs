use core::ffi::c_void;

use apple_cf::cg::CGSize;

use crate::error::AppKitError;
use crate::ffi;
use crate::object::Handle;
use crate::util::read_size;

#[derive(Debug, Clone)]
pub struct Image {
    handle: Handle,
}

impl Image {
    pub fn from_data(data: &[u8]) -> Result<Self, AppKitError> {
        unsafe { Handle::from_retained(ffi::appkit_image_from_data(data.as_ptr(), data.len())) }
            .map(|handle| Self { handle })
            .ok_or(AppKitError::InvalidImageData)
    }

    #[must_use]
    pub const fn as_ptr(&self) -> *mut c_void {
        self.handle.as_ptr()
    }

    #[must_use]
    pub fn size(&self) -> CGSize {
        read_size(|out| unsafe { ffi::appkit_image_size(self.as_ptr(), out) })
    }
}
