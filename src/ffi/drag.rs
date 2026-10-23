use core::ffi::c_void;

use crate::callbacks::CallbackTableHeader;

#[repr(C)]
pub(crate) struct DragPath {
    pub bytes: *const u8,
    pub length: usize,
}

#[repr(C)]
pub(crate) struct DragInfo {
    pub x: f64,
    pub y: f64,
    pub source_operation_mask: u64,
    pub is_source_window: bool,
    pub has_paths: bool,
    pub paths: *const DragPath,
    pub path_count: usize,
}

#[repr(C)]
pub(crate) struct DragCallbackTableV1 {
    pub header: CallbackTableHeader,
    pub deactivate: extern "C" fn(u64),
    pub destination: extern "C" fn(u64, u32, *const DragInfo) -> u64,
    pub source_mask: extern "C" fn(u64, i64) -> u64,
    pub source_end: extern "C" fn(u64, f64, f64, u64),
}

#[repr(C)]
pub(crate) struct DragItem {
    pub kind: u32,
    pub bytes: *const u8,
    pub length: usize,
    pub is_directory: bool,
}

extern "C" {
    pub(crate) fn appkit_drag_attach(
        window: *mut c_void,
        view: *mut c_void,
        token: u64,
        table: *const DragCallbackTableV1,
    ) -> bool;
    pub(crate) fn appkit_drag_detach(window: *mut c_void, token: u64);
    pub(crate) fn appkit_drag_start(
        window: *mut c_void,
        view: *mut c_void,
        event: *mut c_void,
        items: *const DragItem,
        count: usize,
    ) -> bool;
    pub(crate) fn appkit_drag_verify_items(
        window: *mut c_void,
        view: *mut c_void,
        event: *mut c_void,
        items: *const DragItem,
        count: usize,
    ) -> usize;
    pub(crate) fn appkit_drag_verify_destination(
        window: *mut c_void,
        pasteboard: *mut c_void,
        x: f64,
        y: f64,
        source_operations: u64,
        same_window: bool,
        phase: u32,
    ) -> u64;
    pub(crate) fn appkit_drag_verify_source_mask(window: *mut c_void, context: i64) -> u64;
    pub(crate) fn appkit_drag_verify_source_end(
        window: *mut c_void,
        x: f64,
        y: f64,
        operation: u64,
    ) -> bool;
}
