use core::ffi::c_void;

#[repr(C)]
pub(crate) struct AppKitMenuBytes {
    pub bytes: *mut u8,
    pub length: usize,
}

extern "C" {
    pub(crate) fn appkit_menu_new(title: *const u8, title_length: usize) -> *mut c_void;
    pub(crate) fn appkit_menu_title(menu: *mut c_void, out: *mut AppKitMenuBytes) -> bool;
    pub(crate) fn appkit_menu_set_title(
        menu: *mut c_void,
        title: *const u8,
        title_length: usize,
    ) -> bool;
    pub(crate) fn appkit_menu_item_count(menu: *mut c_void) -> usize;
    pub(crate) fn appkit_menu_item_at(menu: *mut c_void, index: usize) -> *mut c_void;
    pub(crate) fn appkit_menu_add_item(menu: *mut c_void, item: *mut c_void) -> bool;
    pub(crate) fn appkit_menu_insert_item(
        menu: *mut c_void,
        item: *mut c_void,
        index: usize,
    ) -> bool;
    pub(crate) fn appkit_menu_remove_item_at(menu: *mut c_void, index: usize) -> bool;
    pub(crate) fn appkit_menu_autoenables_items(menu: *mut c_void) -> bool;
    pub(crate) fn appkit_menu_set_autoenables_items(menu: *mut c_void, enabled: bool) -> bool;
    pub(crate) fn appkit_menu_set_application_host(
        menu: *mut c_void,
        delegate: *mut c_void,
    ) -> bool;
    pub(crate) fn appkit_menu_set_application_menu(role: i32, menu: *mut c_void) -> bool;
    pub(crate) fn appkit_menu_application_menu(role: i32) -> *mut c_void;
    pub(crate) fn appkit_menu_application_dock_menu() -> *mut c_void;
    pub(crate) fn appkit_menu_perform_action_at(menu: *mut c_void, index: usize) -> bool;

    pub(crate) fn appkit_menu_item_new(
        title: *const u8,
        title_length: usize,
        action: i32,
    ) -> *mut c_void;
    pub(crate) fn appkit_menu_item_separator() -> *mut c_void;
    pub(crate) fn appkit_menu_item_title(item: *mut c_void, out: *mut AppKitMenuBytes) -> bool;
    pub(crate) fn appkit_menu_item_set_title(
        item: *mut c_void,
        title: *const u8,
        title_length: usize,
    ) -> bool;
    pub(crate) fn appkit_menu_item_is_separator(item: *mut c_void) -> bool;
    pub(crate) fn appkit_menu_item_submenu(item: *mut c_void) -> *mut c_void;
    pub(crate) fn appkit_menu_item_set_submenu(item: *mut c_void, menu: *mut c_void) -> bool;
    pub(crate) fn appkit_menu_item_enabled(item: *mut c_void) -> bool;
    pub(crate) fn appkit_menu_item_set_enabled(item: *mut c_void, enabled: bool) -> bool;
    pub(crate) fn appkit_menu_item_state(item: *mut c_void, out: *mut i32) -> bool;
    pub(crate) fn appkit_menu_item_set_state(item: *mut c_void, state: i32) -> bool;
    pub(crate) fn appkit_menu_item_tag(item: *mut c_void, out: *mut i64) -> bool;
    pub(crate) fn appkit_menu_item_set_tag(item: *mut c_void, tag: i64) -> bool;
    pub(crate) fn appkit_menu_item_represented_integer(item: *mut c_void, out: *mut i64) -> bool;
    pub(crate) fn appkit_menu_item_set_represented_integer(
        item: *mut c_void,
        present: bool,
        value: i64,
    ) -> bool;
    pub(crate) fn appkit_menu_item_key_equivalent(
        item: *mut c_void,
        out: *mut AppKitMenuBytes,
    ) -> bool;
    pub(crate) fn appkit_menu_item_key_modifiers(item: *mut c_void) -> u64;
    pub(crate) fn appkit_menu_item_set_key_equivalent(
        item: *mut c_void,
        key: *const u8,
        key_length: usize,
        present: bool,
        modifiers: u64,
    ) -> bool;
    pub(crate) fn appkit_menu_item_allows_automatic_key_equivalent_localization(
        item: *mut c_void,
    ) -> bool;
    pub(crate) fn appkit_menu_item_action(item: *mut c_void) -> i32;
    pub(crate) fn appkit_menu_item_set_action(item: *mut c_void, action: i32) -> bool;
    pub(crate) fn appkit_menu_item_action_selector(
        item: *mut c_void,
        out: *mut AppKitMenuBytes,
    ) -> bool;
    pub(crate) fn appkit_menu_item_has_explicit_target(item: *mut c_void) -> bool;
    pub(crate) fn appkit_menu_bytes_free(bytes: *mut u8);
    pub fn appkit_menu_probe_window_responds_to_item(
        window: *mut c_void,
        item: *mut c_void,
    ) -> bool;
}
