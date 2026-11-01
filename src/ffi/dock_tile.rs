use core::ffi::{c_char, c_void};

extern "C" {
    pub fn appkit_dock_tile_set_progress(tile: *mut c_void, kind: i32, fraction: f64) -> bool;
    pub fn appkit_dock_tile_set_badge_label(tile: *mut c_void, label: *const c_char) -> bool;
    pub fn appkit_dock_tile_badge_label(tile: *mut c_void) -> *mut c_char;
    pub fn appkit_dock_tile_display(tile: *mut c_void);
}
