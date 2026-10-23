use core::ffi::{c_char, c_void};

use crate::callbacks::CallbackTableHeader;

#[repr(C)]
pub struct FileSystemPath {
    pub bytes: *const c_char,
}

#[repr(C)]
pub struct PowerTableV1 {
    pub header: CallbackTableHeader,
    pub event: Option<extern "C" fn(u64, i32)>,
}

#[repr(C)]
pub struct SchemeTableV1 {
    pub header: CallbackTableHeader,
    pub complete: Option<extern "C" fn(u64, bool, *const u8, usize)>,
}

extern "C" {
    pub fn appkit_workspace_open_url(url: *const c_char) -> bool;
    pub fn appkit_workspace_open_file(path: *const c_char) -> bool;
    pub fn appkit_workspace_reveal_paths(paths: *const FileSystemPath, count: usize);
    pub fn appkit_workspace_icon_for_file(path: *const c_char) -> *mut c_void;
    pub fn appkit_workspace_icon_for_file_type(file_type: *const c_char) -> *mut c_void;
    pub fn appkit_workspace_icon_for_application(bundle_id: *const c_char) -> *mut c_void;
    pub fn appkit_workspace_power_observer_new(
        token: u64,
        table: *const PowerTableV1,
    ) -> *mut c_void;
    pub fn appkit_workspace_power_observer_notify(observer: *mut c_void, kind: i32);
    pub fn appkit_workspace_register_url_scheme(
        scheme: *const c_char,
        token: u64,
        table: *const SchemeTableV1,
        status: *mut i32,
    ) -> *mut c_void;
}
