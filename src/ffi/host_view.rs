use core::ffi::c_void;

use apple_cf::cg::CGRect;

use crate::callbacks::CallbackTableHeader;

#[repr(C)]
pub(crate) struct HostViewCallbackTableV1 {
    pub header: CallbackTableHeader,
    pub deactivate: extern "C" fn(u64),
    pub event: Option<extern "C" fn(u64, u32, *mut c_void) -> bool>,
    pub notification: Option<extern "C" fn(u64, u32, f64, f64)>,
    pub query: Option<extern "C" fn(u64, u32, *mut c_void) -> bool>,
}

extern "C" {
    pub(crate) fn appkit_host_view_new(
        token: u64,
        table: *const HostViewCallbackTableV1,
        layer: *mut c_void,
        frame: *const CGRect,
    ) -> *mut c_void;
    pub(crate) fn appkit_host_view_metal_layer_new() -> *mut c_void;
    pub(crate) fn appkit_host_view_detach(view: *mut c_void);
    pub(crate) fn appkit_host_view_attach(view: *mut c_void, parent: *mut c_void) -> bool;
    pub(crate) fn appkit_host_view_set_frame_size(
        view: *mut c_void,
        width: f64,
        height: f64,
    ) -> bool;
    pub(crate) fn appkit_host_view_set_tracking_options(view: *mut c_void, options: u64) -> bool;
    pub(crate) fn appkit_host_view_tracking_options(view: *mut c_void) -> u64;
    pub(crate) fn appkit_host_view_set_owns_titlebar_drag(view: *mut c_void, owns: bool) -> bool;
    pub(crate) fn appkit_host_view_invalidate_cursor_rects(view: *mut c_void) -> bool;
    pub(crate) fn appkit_host_view_add_cursor_rect(
        view: *mut c_void,
        rect: *const CGRect,
        cursor: *mut c_void,
    ) -> bool;
    pub(crate) fn appkit_host_view_send_event(
        view: *mut c_void,
        selector: u32,
        event: *mut c_void,
    ) -> bool;
    pub(crate) fn appkit_host_view_notify_backing_properties(view: *mut c_void);
    pub(crate) fn appkit_host_view_notify_appearance(view: *mut c_void);
    pub(crate) fn appkit_host_view_display_layer(view: *mut c_void) -> bool;
    pub(crate) fn appkit_host_view_accepts_first_mouse(
        view: *mut c_void,
        event: *mut c_void,
    ) -> bool;
    pub(crate) fn appkit_host_view_accepts_first_responder(view: *mut c_void) -> bool;
}
