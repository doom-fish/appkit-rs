use core::ffi::c_void;

use crate::application_host::ApplicationHostCallbackTableV1;

extern "C" {
    pub(crate) fn appkit_application_host_install_v1(
        token: u64,
        table: *const ApplicationHostCallbackTableV1,
    ) -> *mut c_void;
    pub(crate) fn appkit_application_host_uninstall(delegate: *mut c_void) -> bool;
    pub fn appkit_application_host_probe(
        delegate: *mut c_void,
        method: i32,
        action: i32,
        tag: i64,
        has_visible_windows: bool,
    ) -> i64;
    pub fn appkit_application_host_probe_retained_menu() -> *mut c_void;
}
