use core::ffi::c_void;

use crate::callbacks::CallbackTableV1;

extern "C" {
    pub fn appkit_callbacks_create_v1(token: u64, table: *const CallbackTableV1) -> *mut c_void;
}
