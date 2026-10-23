use core::ffi::{c_char, c_void};

use crate::callbacks::CallbackTableHeader;

#[repr(C)]
pub struct AlertCallbacksV1 {
    pub header: CallbackTableHeader,
    pub deactivate: extern "C" fn(u64),
    pub complete: extern "C" fn(u64, i64),
}

extern "C" {
    pub fn appkit_alert_new() -> *mut c_void;
    pub fn appkit_alert_set_style(alert: *mut c_void, style: i32);
    pub fn appkit_alert_set_message(alert: *mut c_void, message: *const c_char);
    pub fn appkit_alert_set_informative_text(alert: *mut c_void, text: *const c_char);
    pub fn appkit_alert_add_button(
        alert: *mut c_void,
        title: *const c_char,
        is_cancel: bool,
    ) -> i64;
    pub fn appkit_alert_begin_sheet(
        alert: *mut c_void,
        window: *mut c_void,
        token: u64,
        callbacks: *const AlertCallbacksV1,
    ) -> bool;
}
