use core::ffi::c_void;

#[repr(C)]
pub(crate) struct AppKitPasteboardInput {
    pub bytes: *const u8,
    pub length: usize,
}

#[repr(C)]
pub(crate) struct AppKitPasteboardBytes {
    pub bytes: *mut u8,
    pub length: usize,
}

#[repr(C)]
pub(crate) struct AppKitPasteboardBytesList {
    pub items: *mut AppKitPasteboardBytes,
    pub count: usize,
}

extern "C" {
    pub(crate) fn appkit_pasteboard_general() -> *mut c_void;
    pub(crate) fn appkit_pasteboard_find() -> *mut c_void;
    pub(crate) fn appkit_pasteboard_unique() -> *mut c_void;
    pub(crate) fn appkit_pasteboard_release_globally(pasteboard: *mut c_void);
    pub(crate) fn appkit_pasteboard_types(
        pasteboard: *mut c_void,
        out: *mut AppKitPasteboardBytesList,
    ) -> bool;
    pub(crate) fn appkit_pasteboard_data_for_type(
        pasteboard: *mut c_void,
        kind: *const u8,
        kind_length: usize,
        out: *mut AppKitPasteboardBytes,
    ) -> bool;
    pub(crate) fn appkit_pasteboard_string_for_type(
        pasteboard: *mut c_void,
        kind: *const u8,
        kind_length: usize,
        out: *mut AppKitPasteboardBytes,
    ) -> bool;
    pub(crate) fn appkit_pasteboard_filenames(
        pasteboard: *mut c_void,
        out: *mut AppKitPasteboardBytesList,
    ) -> bool;
    pub(crate) fn appkit_pasteboard_clear_contents(pasteboard: *mut c_void) -> i64;
    pub(crate) fn appkit_pasteboard_declare_types(
        pasteboard: *mut c_void,
        kinds: *const AppKitPasteboardInput,
        count: usize,
    ) -> i64;
    pub(crate) fn appkit_pasteboard_set_data_for_type(
        pasteboard: *mut c_void,
        kind: *const u8,
        kind_length: usize,
        data: *const u8,
        data_length: usize,
    ) -> bool;
    pub(crate) fn appkit_pasteboard_set_string_for_type(
        pasteboard: *mut c_void,
        kind: *const u8,
        kind_length: usize,
        value: *const u8,
        value_length: usize,
    ) -> bool;
    pub(crate) fn appkit_pasteboard_set_filenames(
        pasteboard: *mut c_void,
        paths: *const AppKitPasteboardInput,
        count: usize,
    ) -> bool;
    pub(crate) fn appkit_pasteboard_change_count(pasteboard: *mut c_void) -> i64;
    pub(crate) fn appkit_pasteboard_bytes_free(bytes: *mut u8);
    pub(crate) fn appkit_pasteboard_list_free(list: *mut AppKitPasteboardBytesList);
}
