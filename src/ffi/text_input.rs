use core::ffi::c_void;

use apple_cf::cg::CGRect;

use crate::callbacks::{CallbackTableHeader, Utf16Range};

#[repr(C)]
pub(crate) struct TextInputQueryResult {
    pub range: Utf16Range,
    pub rect: CGRect,
    pub character_index: usize,
    pub has_marked_text: bool,
}

pub(crate) type CopyUtf8 = extern "C" fn(*mut c_void, *const u8, usize) -> bool;

#[repr(C)]
pub(crate) struct TextInputCallbackTableV1 {
    pub header: CallbackTableHeader,
    pub deactivate: extern "C" fn(u64),
    pub query: Option<
        extern "C" fn(
            u64,
            u32,
            Utf16Range,
            f64,
            f64,
            *mut TextInputQueryResult,
            *mut c_void,
            Option<CopyUtf8>,
        ) -> bool,
    >,
    pub action: Option<extern "C" fn(u64, u32, *const u8, usize, Utf16Range, Utf16Range)>,
    pub command: Option<extern "C" fn(u64, *const u8, usize) -> bool>,
}

extern "C" {
    pub(crate) fn appkit_text_input_install(
        view: *mut c_void,
        token: u64,
        table: *const TextInputCallbackTableV1,
    ) -> bool;
    pub(crate) fn appkit_text_input_uninstall(view: *mut c_void, token: u64);
    pub(crate) fn appkit_text_input_handle_event(
        view: *mut c_void,
        token: u64,
        event: *mut c_void,
    ) -> u32;
    pub(crate) fn appkit_text_input_discard_marked_text(view: *mut c_void, token: u64) -> bool;
    pub(crate) fn appkit_text_input_invalidate_character_coordinates(
        view: *mut c_void,
        token: u64,
    ) -> bool;
}
