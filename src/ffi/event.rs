use core::ffi::c_void;

use apple_cf::cg::CGPoint;

#[repr(C)]
pub(crate) struct AppKitEventString {
    pub bytes: *mut u8,
    pub length: usize,
}

extern "C" {
    pub(crate) fn appkit_event_current() -> *mut c_void;
    pub(crate) fn appkit_event_pressed_mouse_buttons() -> u64;
    pub(crate) fn appkit_event_class_modifier_flags() -> u64;
    pub(crate) fn appkit_event_type(event: *mut c_void) -> u64;
    pub(crate) fn appkit_event_modifier_flags(event: *mut c_void) -> u64;
    pub(crate) fn appkit_event_location_in_window(event: *mut c_void, out: *mut CGPoint) -> bool;
    pub(crate) fn appkit_event_button_number(event: *mut c_void, out: *mut i64) -> bool;
    pub(crate) fn appkit_event_click_count(event: *mut c_void, out: *mut i64) -> bool;
    pub(crate) fn appkit_event_is_repeat(event: *mut c_void, out: *mut bool) -> bool;
    pub(crate) fn appkit_event_key_code(event: *mut c_void, out: *mut u16) -> bool;
    pub(crate) fn appkit_event_characters(
        event: *mut c_void,
        ignoring_modifiers: bool,
        out: *mut AppKitEventString,
    ) -> bool;
    pub(crate) fn appkit_event_string_free(bytes: *mut u8);
    pub(crate) fn appkit_event_scrolling_delta(event: *mut c_void, out: *mut CGPoint) -> bool;
    pub(crate) fn appkit_event_has_precise_scrolling_deltas(
        event: *mut c_void,
        out: *mut bool,
    ) -> bool;
    pub(crate) fn appkit_event_phase(event: *mut c_void, out: *mut u64) -> bool;
    pub(crate) fn appkit_event_momentum_phase(event: *mut c_void, out: *mut u64) -> bool;
    pub(crate) fn appkit_event_pressure(event: *mut c_void, out: *mut f32) -> bool;
    pub(crate) fn appkit_event_stage(event: *mut c_void, out: *mut i64) -> bool;
    pub(crate) fn appkit_event_magnification(event: *mut c_void, out: *mut f64) -> bool;
    pub(crate) fn appkit_event_swipe_delta(event: *mut c_void, out: *mut CGPoint) -> bool;
    pub(crate) fn appkit_event_timestamp(event: *mut c_void) -> f64;
    pub(crate) fn appkit_event_window_number(event: *mut c_void) -> i64;
    pub(crate) fn appkit_event_synthetic_key(
        event_type: u64,
        modifiers: u64,
        key_code: u16,
        characters: *const u8,
        characters_length: usize,
        ignoring_modifiers: *const u8,
        ignoring_modifiers_length: usize,
        is_repeat: bool,
    ) -> *mut c_void;
    pub(crate) fn appkit_event_synthetic_mouse(
        event_type: u64,
        location: *const CGPoint,
        modifiers: u64,
        click_count: i64,
        pressure: f32,
    ) -> *mut c_void;
}
