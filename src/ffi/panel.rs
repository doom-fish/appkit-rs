use core::ffi::{c_char, c_void};

use crate::callbacks::CallbackTableHeader;

#[repr(C)]
pub struct FileSystemPath {
    pub bytes: *const u8,
    pub length: usize,
}

#[repr(C)]
pub struct PanelCallbacksV1 {
    pub header: CallbackTableHeader,
    pub deactivate: extern "C" fn(u64),
    pub complete: extern "C" fn(u64, bool, *const FileSystemPath, usize),
}

extern "C" {
    pub fn appkit_open_panel_new() -> *mut c_void;
    pub fn appkit_save_panel_new() -> *mut c_void;
    pub fn appkit_panel_set_directory(panel: *mut c_void, path: *const u8, length: usize) -> bool;
    pub fn appkit_panel_set_prompt(panel: *mut c_void, prompt: *const c_char);
    pub fn appkit_panel_set_message(panel: *mut c_void, message: *const c_char);
    pub fn appkit_panel_set_can_create_directories(panel: *mut c_void, can_create: bool);
    pub fn appkit_panel_set_shows_hidden_files(panel: *mut c_void, shows: bool);
    pub fn appkit_open_panel_set_can_choose_files(panel: *mut c_void, can_choose: bool);
    pub fn appkit_open_panel_set_can_choose_directories(panel: *mut c_void, can_choose: bool);
    pub fn appkit_open_panel_set_allows_multiple_selection(panel: *mut c_void, allows: bool);
    pub fn appkit_open_panel_set_resolves_aliases(panel: *mut c_void, resolves: bool);
    pub fn appkit_save_panel_set_suggested_name(panel: *mut c_void, name: *const c_char);
    pub fn appkit_panel_begin(
        panel: *mut c_void,
        token: u64,
        callbacks: *const PanelCallbacksV1,
    ) -> bool;
}
