use core::ffi::{c_char, c_void};

#[repr(C)]
pub(crate) struct AppKitWindowStateBytes {
    pub bytes: *mut u8,
    pub length: usize,
}

extern "C" {
    pub(crate) fn appkit_window_state_encode(
        window: *mut c_void,
        out: *mut AppKitWindowStateBytes,
    ) -> bool;
    pub(crate) fn appkit_window_state_restore(
        window: *mut c_void,
        bytes: *const u8,
        length: usize,
        error: *mut *mut c_char,
    ) -> bool;
    pub(crate) fn appkit_window_state_bytes_free(bytes: *mut u8);
}
