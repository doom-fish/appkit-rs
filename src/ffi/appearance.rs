use core::ffi::c_void;

extern "C" {
    pub fn appkit_appearance_named(name: i32) -> *mut c_void;
    pub fn appkit_appearance_name(appearance: *mut c_void) -> i32;
    pub fn appkit_appearance_window_effective(window: *mut c_void) -> *mut c_void;
    pub fn appkit_appearance_view_effective(view: *mut c_void) -> *mut c_void;
    pub fn appkit_appearance_view_override(view: *mut c_void) -> *mut c_void;
    pub fn appkit_appearance_set_view_override(view: *mut c_void, appearance: i32);
}
