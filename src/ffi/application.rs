use core::ffi::{c_char, c_void};

extern "C" {
    pub fn appkit_application_key_window(application: *mut c_void) -> *mut c_void;
    pub fn appkit_application_main_window(application: *mut c_void) -> *mut c_void;
    pub fn appkit_application_ordered_windows(application: *mut c_void) -> *mut c_void;
    pub fn appkit_application_window_array_count(array: *mut c_void) -> usize;
    pub fn appkit_application_window_array_at(array: *mut c_void, index: usize) -> *mut c_void;
    pub fn appkit_application_is_active(application: *mut c_void) -> bool;
    pub fn appkit_application_is_hidden(application: *mut c_void) -> bool;
    pub fn appkit_application_activate_ignoring_other_apps(application: *mut c_void, ignore: bool);
    pub fn appkit_application_hide(application: *mut c_void);
    pub fn appkit_application_unhide(application: *mut c_void);
    pub fn appkit_application_hide_other_applications(application: *mut c_void);
    pub fn appkit_application_unhide_all_applications(application: *mut c_void);
    pub fn appkit_application_presentation_options(application: *mut c_void) -> u64;
    pub fn appkit_application_set_presentation_options(application: *mut c_void, options: u64);
    pub fn appkit_application_request_user_attention(application: *mut c_void, kind: i32) -> i64;
    pub fn appkit_application_cancel_user_attention(application: *mut c_void, request: i64);
    pub fn appkit_application_appearance(application: *mut c_void) -> i32;
    pub fn appkit_application_effective_appearance(application: *mut c_void) -> i32;
    pub fn appkit_application_set_appearance(application: *mut c_void, appearance: i32);
    pub fn appkit_application_order_front_character_palette(
        application: *mut c_void,
        sender: *mut c_void,
    );
    pub fn appkit_application_change_windows_item(
        application: *mut c_void,
        window: *mut c_void,
        title: *const c_char,
        is_filename: bool,
    );
    pub fn appkit_application_current_event(application: *mut c_void) -> *mut c_void;
}
