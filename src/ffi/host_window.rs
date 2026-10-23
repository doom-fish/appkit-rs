use core::ffi::c_void;

use crate::callbacks::CallbackTableHeader;

#[repr(C)]
pub struct HostWindowCallbackTableV1 {
    pub header: CallbackTableHeader,
    pub deactivate: extern "C" fn(u64),
    pub should_close: extern "C" fn(u64) -> bool,
    pub event: extern "C" fn(u64, u32),
}

#[repr(C)]
pub struct HostWindowConfiguration {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub style_mask: u64,
    pub backing_store: i64,
    pub screen: *mut c_void,
    pub defer_creation: bool,
    pub can_become_main: bool,
    pub can_become_key: bool,
}

#[repr(C)]
pub struct HostPanelOptions {
    pub is_floating_panel: i8,
    pub becomes_key_only_if_needed: i8,
    pub works_when_modal: i8,
    pub hides_on_deactivate: i8,
}

extern "C" {
    pub fn appkit_host_window_new(
        configuration: *const HostWindowConfiguration,
        token: u64,
        table: *const HostWindowCallbackTableV1,
    ) -> *mut c_void;
    pub fn appkit_host_panel_new(
        configuration: *const HostWindowConfiguration,
        options: *const HostPanelOptions,
        token: u64,
        table: *const HostWindowCallbackTableV1,
    ) -> *mut c_void;
    pub fn appkit_host_window_detach(window: *mut c_void);
    pub fn appkit_host_window_set_can_become_main(window: *mut c_void, enabled: bool);
    pub fn appkit_host_window_set_can_become_key(window: *mut c_void, enabled: bool);
    pub fn appkit_host_window_can_become_main(window: *mut c_void) -> bool;
    pub fn appkit_host_window_can_become_key(window: *mut c_void) -> bool;
    pub fn appkit_host_window_verify_should_close(window: *mut c_void) -> i8;
    pub fn appkit_host_window_verify_event(window: *mut c_void, event: u32) -> bool;
}
