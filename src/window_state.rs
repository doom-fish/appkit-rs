use core::ptr;
use core::slice;

use crate::error::AppKitError;
use crate::ffi::window_state::{self as ffi, AppKitWindowStateBytes};
use crate::util::take_string;
use crate::window::Window;

struct OwnedBytes(AppKitWindowStateBytes);

impl Drop for OwnedBytes {
    fn drop(&mut self) {
        unsafe { ffi::appkit_window_state_bytes_free(self.0.bytes) };
    }
}

impl Window {
    #[must_use]
    pub fn encode_restorable_state(&self) -> Option<Vec<u8>> {
        let mut encoded = AppKitWindowStateBytes {
            bytes: ptr::null_mut(),
            length: 0,
        };
        if !unsafe { ffi::appkit_window_state_encode(self.as_ptr(), &raw mut encoded) } {
            return None;
        }
        let encoded = OwnedBytes(encoded);
        if encoded.0.bytes.is_null() {
            return None;
        }
        Some(unsafe { slice::from_raw_parts(encoded.0.bytes, encoded.0.length) }.to_vec())
    }

    pub fn restore_state(&self, state: &[u8]) -> Result<(), AppKitError> {
        let mut error = ptr::null_mut();
        if unsafe {
            ffi::appkit_window_state_restore(
                self.as_ptr(),
                state.as_ptr(),
                state.len(),
                &raw mut error,
            )
        } {
            return Ok(());
        }
        Err(AppKitError::InvalidArgument(
            unsafe { take_string(error) }
                .unwrap_or_else(|| "AppKit cannot restore the native window state".to_owned()),
        ))
    }
}
