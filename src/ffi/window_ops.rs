use core::ffi::{c_char, c_void};

use apple_cf::cg::{CGPoint, CGRect, CGSize};

use crate::callbacks::CallbackTableHeader;
use crate::window_ops::RgbaColor;

#[repr(C)]
pub struct SheetCallbackTableV1 {
    pub header: CallbackTableHeader,
    pub deactivate: extern "C" fn(u64),
    pub complete: extern "C" fn(u64, i64),
}

extern "C" {
    pub fn appkit_window_ops_set_appearance(window: *mut c_void, appearance: i32) -> *mut c_char;
    pub fn appkit_window_ops_style_mask(window: *mut c_void) -> u64;
    pub fn appkit_window_ops_set_style_mask(window: *mut c_void, mask: u64) -> *mut c_char;
    pub fn appkit_window_ops_level(window: *mut c_void) -> i64;
    pub fn appkit_window_ops_set_level(window: *mut c_void, level: i64) -> *mut c_char;
    pub fn appkit_window_ops_collection_behavior(window: *mut c_void) -> u64;
    pub fn appkit_window_ops_set_collection_behavior(
        window: *mut c_void,
        behavior: u64,
    ) -> *mut c_char;
    pub fn appkit_window_ops_make_first_responder(window: *mut c_void, view: *mut c_void) -> bool;
    pub fn appkit_window_ops_set_initial_first_responder(
        window: *mut c_void,
        view: *mut c_void,
    ) -> *mut c_char;
    pub fn appkit_window_ops_set_frame(
        window: *mut c_void,
        frame: *const CGRect,
        display: bool,
        animate: bool,
    ) -> *mut c_char;
    pub fn appkit_window_ops_set_frame_top_left_point(
        window: *mut c_void,
        point: *const CGPoint,
    ) -> *mut c_char;
    pub fn appkit_window_ops_content_size(window: *mut c_void, out: *mut CGSize);
    pub fn appkit_window_ops_set_content_size(
        window: *mut c_void,
        size: *const CGSize,
    ) -> *mut c_char;
    pub fn appkit_window_ops_frame_rect_for_content_rect(
        window: *mut c_void,
        rect: *const CGRect,
        out: *mut CGRect,
    );
    pub fn appkit_window_ops_content_rect_for_frame_rect(
        window: *mut c_void,
        rect: *const CGRect,
        out: *mut CGRect,
    );
    pub fn appkit_window_ops_convert_point_to_screen(
        window: *mut c_void,
        point: *const CGPoint,
        out: *mut CGPoint,
    );
    pub fn appkit_window_ops_convert_rect_to_screen(
        window: *mut c_void,
        rect: *const CGRect,
        out: *mut CGRect,
    );
    pub fn appkit_window_ops_content_min_size(window: *mut c_void, out: *mut CGSize);
    pub fn appkit_window_ops_set_content_min_size(
        window: *mut c_void,
        size: *const CGSize,
    ) -> *mut c_char;
    pub fn appkit_window_ops_mouse_location_outside_event_stream(
        window: *mut c_void,
        out: *mut CGPoint,
    );
    pub fn appkit_window_ops_is_key_window(window: *mut c_void) -> bool;
    pub fn appkit_window_ops_is_main_window(window: *mut c_void) -> bool;
    pub fn appkit_window_ops_is_visible(window: *mut c_void) -> bool;
    pub fn appkit_window_ops_occlusion_state(window: *mut c_void) -> u64;
    pub fn appkit_window_ops_is_miniaturized(window: *mut c_void) -> bool;
    pub fn appkit_window_ops_miniaturize(window: *mut c_void);
    pub fn appkit_window_ops_deminiaturize(window: *mut c_void);
    pub fn appkit_window_ops_is_zoomed(window: *mut c_void) -> bool;
    pub fn appkit_window_ops_zoom(window: *mut c_void);
    pub fn appkit_window_ops_toggle_full_screen(window: *mut c_void);
    pub fn appkit_window_ops_is_simple_full_screen(window: *mut c_void) -> bool;
    #[doc(hidden)]
    pub fn appkit_window_ops_probe_closed_simple_full_screen_responder() -> bool;
    pub fn appkit_window_ops_toggle_simple_full_screen(
        window: *mut c_void,
        view: *mut c_void,
    ) -> bool;
    pub fn appkit_window_ops_begin_sheet(
        window: *mut c_void,
        sheet: *mut c_void,
        token: u64,
        table: *const SheetCallbackTableV1,
    ) -> bool;
    #[doc(hidden)]
    pub fn appkit_window_ops_probe_sheet_completion(
        window: *mut c_void,
        sheet: *mut c_void,
        token: u64,
        table: *const SheetCallbackTableV1,
    ) -> bool;
    pub fn appkit_window_ops_end_sheet(
        window: *mut c_void,
        sheet: *mut c_void,
        response: i64,
    ) -> bool;
    pub fn appkit_window_ops_attached_sheet(window: *mut c_void) -> *mut c_void;
    pub fn appkit_window_ops_sheet_parent(window: *mut c_void) -> *mut c_void;
    pub fn appkit_window_ops_allows_automatic_window_tabbing() -> bool;
    pub fn appkit_window_ops_set_allows_automatic_window_tabbing(allows: bool);
    pub fn appkit_window_ops_tabbing_identifier(window: *mut c_void) -> *mut c_char;
    pub fn appkit_window_ops_set_tabbing_identifier(
        window: *mut c_void,
        identifier: *const c_char,
    ) -> *mut c_char;
    pub fn appkit_window_ops_tabbing_mode(window: *mut c_void) -> i64;
    pub fn appkit_window_ops_set_tabbing_mode(window: *mut c_void, mode: i64) -> *mut c_char;
    pub fn appkit_window_ops_add_tabbed_window(
        window: *mut c_void,
        other: *mut c_void,
        ordered: i64,
    ) -> bool;
    pub fn appkit_window_ops_tabbed_windows(window: *mut c_void) -> *mut c_void;
    pub fn appkit_window_ops_tabbed_windows_count(windows: *mut c_void) -> usize;
    pub fn appkit_window_ops_tabbed_window_at(windows: *mut c_void, index: usize) -> *mut c_void;
    pub fn appkit_window_ops_is_tab_bar_visible(window: *mut c_void) -> bool;
    pub fn appkit_window_ops_selected_tab(window: *mut c_void) -> *mut c_void;
    pub fn appkit_window_ops_select_tab(window: *mut c_void, selected: *mut c_void) -> bool;
    pub fn appkit_window_ops_toggle_tab_overview(window: *mut c_void);
    pub fn appkit_window_ops_move_tab_to_new_window(window: *mut c_void);
    pub fn appkit_window_ops_merge_all_windows(window: *mut c_void);
    pub fn appkit_window_ops_select_next_tab(window: *mut c_void);
    pub fn appkit_window_ops_select_previous_tab(window: *mut c_void);
    pub fn appkit_window_ops_toggle_tab_bar(window: *mut c_void);
    pub fn appkit_window_ops_standard_window_button(
        window: *mut c_void,
        button: i64,
    ) -> *mut c_void;
    pub fn appkit_window_ops_chrome_superview(view: *mut c_void) -> *mut c_void;
    pub fn appkit_window_ops_chrome_frame(view: *mut c_void, out: *mut CGRect);
    pub fn appkit_window_ops_chrome_set_frame(
        view: *mut c_void,
        frame: *const CGRect,
    ) -> *mut c_char;
    pub fn appkit_window_ops_chrome_set_frame_origin(
        view: *mut c_void,
        origin: *const CGPoint,
    ) -> *mut c_char;
    pub fn appkit_window_ops_chrome_is_hidden(view: *mut c_void) -> bool;
    pub fn appkit_window_ops_chrome_set_hidden(view: *mut c_void, hidden: bool);
    pub fn appkit_window_ops_chrome_update_tracking_areas(view: *mut c_void);
    pub fn appkit_window_ops_view_set_needs_display(view: *mut c_void, needs_display: bool);
    pub fn appkit_window_ops_background_color(window: *mut c_void, out: *mut RgbaColor) -> bool;
    pub fn appkit_window_ops_set_background_color(
        window: *mut c_void,
        color: *const RgbaColor,
    ) -> *mut c_char;
    pub fn appkit_window_ops_is_opaque(window: *mut c_void) -> bool;
    pub fn appkit_window_ops_set_opaque(window: *mut c_void, opaque: bool);
    pub fn appkit_window_ops_has_shadow(window: *mut c_void) -> bool;
    pub fn appkit_window_ops_set_has_shadow(window: *mut c_void, shadow: bool);
    pub fn appkit_window_ops_invalidate_shadow(window: *mut c_void);
    pub fn appkit_window_ops_titlebar_appears_transparent(window: *mut c_void) -> bool;
    pub fn appkit_window_ops_set_titlebar_appears_transparent(
        window: *mut c_void,
        transparent: bool,
    );
    pub fn appkit_window_ops_is_movable(window: *mut c_void) -> bool;
    pub fn appkit_window_ops_set_movable(window: *mut c_void, movable: bool);
    pub fn appkit_window_ops_is_movable_by_window_background(window: *mut c_void) -> bool;
    pub fn appkit_window_ops_set_movable_by_window_background(window: *mut c_void, movable: bool);
    pub fn appkit_window_ops_accepts_mouse_moved_events(window: *mut c_void) -> bool;
    pub fn appkit_window_ops_set_accepts_mouse_moved_events(window: *mut c_void, accepts: bool);
    pub fn appkit_window_ops_perform_window_drag(
        window: *mut c_void,
        event: *mut c_void,
    ) -> *mut c_char;
    pub fn appkit_window_ops_is_document_edited(window: *mut c_void) -> bool;
    pub fn appkit_window_ops_set_document_edited(window: *mut c_void, edited: bool);
    pub fn appkit_window_ops_represented_filename(window: *mut c_void) -> *mut c_char;
    pub fn appkit_window_ops_set_represented_filename(
        window: *mut c_void,
        bytes: *const c_char,
        length: usize,
    ) -> *mut c_char;
    pub fn appkit_window_ops_order_out(window: *mut c_void);
    pub fn appkit_window_ops_order_back(window: *mut c_void);
    pub fn appkit_window_ops_order_window(
        window: *mut c_void,
        ordered: i64,
        relative_to: i64,
    ) -> *mut c_char;
    pub fn appkit_window_ops_window_number(window: *mut c_void) -> i64;
    pub fn appkit_window_ops_display_if_needed(window: *mut c_void);
    pub fn appkit_window_ops_set_content_view_needs_display(
        window: *mut c_void,
        needs_display: bool,
    );
    pub fn appkit_window_ops_invalidate_cursor_rects_for_view(
        window: *mut c_void,
        view: *mut c_void,
    );
    pub fn appkit_window_ops_resign_key_window(window: *mut c_void);
    pub fn appkit_window_ops_is_released_when_closed(window: *mut c_void) -> bool;
    pub fn appkit_window_ops_retain_when_closed(window: *mut c_void);
    pub fn appkit_window_ops_effective_appearance(window: *mut c_void) -> i32;
}
