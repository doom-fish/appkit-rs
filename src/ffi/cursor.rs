use core::ffi::c_void;

extern "C" {
    pub fn appkit_cursor_system(kind: i32) -> *mut c_void;
    pub fn appkit_cursor_check_legacy_diagonals() -> bool;
    pub fn appkit_cursor_current() -> *mut c_void;
    pub fn appkit_cursor_from_image(
        image: *mut c_void,
        width: f64,
        height: f64,
        hot_spot_x: f64,
        hot_spot_y: f64,
    ) -> *mut c_void;
    pub fn appkit_cursor_set(cursor: *mut c_void) -> bool;
    pub fn appkit_cursor_hide() -> bool;
    pub fn appkit_cursor_unhide() -> bool;
    pub fn appkit_cursor_set_hidden_until_mouse_moves(hidden: bool) -> bool;
}
