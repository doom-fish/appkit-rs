use core::ffi::{c_char, c_void};

use apple_cf::cg::{CGRect, CGSize};

pub mod alert;
pub mod appearance;
pub mod application;
pub mod application_host;
pub mod blur;
pub mod callbacks;
pub mod cursor;
pub mod drag;
pub mod event;
pub mod host_view;
pub mod host_window;
pub mod menu;
pub mod panel;
pub mod pasteboard;
pub mod text_input;
pub mod window_ops;
pub mod window_state;
pub mod workspace;

extern "C" {
    pub fn appkit_retain(object: *mut c_void);
    pub fn appkit_release(object: *mut c_void);
    pub fn appkit_string_free(string: *mut c_char);

    pub fn appkit_view_window(view: *mut c_void) -> *mut c_void;
    pub fn appkit_view_bounds(view: *mut c_void, out: *mut CGRect);
    pub fn appkit_view_frame(view: *mut c_void, out: *mut CGRect);
    pub fn appkit_view_layer(view: *mut c_void) -> *mut c_void;
    pub fn appkit_view_wants_layer(view: *mut c_void) -> bool;
    pub fn appkit_view_set_wants_layer(view: *mut c_void, wants: bool);

    pub fn appkit_window_new(
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        title: *const c_char,
    ) -> *mut c_void;
    pub fn appkit_window_content_view(window: *mut c_void) -> *mut c_void;
    pub fn appkit_window_appearance(window: *mut c_void) -> i32;
    pub fn appkit_window_set_appearance(window: *mut c_void, appearance: i32);
    pub fn appkit_window_title(window: *mut c_void) -> *mut c_char;
    pub fn appkit_window_set_title(window: *mut c_void, title: *const c_char);
    pub fn appkit_window_title_visibility(window: *mut c_void) -> i64;
    pub fn appkit_window_set_title_visibility(window: *mut c_void, visibility: i64);
    pub fn appkit_window_titlebar_separator_style(window: *mut c_void) -> i64;
    pub fn appkit_window_set_titlebar_separator_style(window: *mut c_void, style: i64);
    pub fn appkit_window_frame(window: *mut c_void, out: *mut CGRect);
    pub fn appkit_window_content_layout_rect(window: *mut c_void, out: *mut CGRect);
    pub fn appkit_window_backing_scale_factor(window: *mut c_void) -> f64;
    pub fn appkit_window_screen(window: *mut c_void) -> *mut c_void;
    pub fn appkit_window_alpha_value(window: *mut c_void) -> f64;
    pub fn appkit_window_set_alpha_value(window: *mut c_void, alpha: f64);
    pub fn appkit_window_animate_alpha_value(window: *mut c_void, alpha: f64);
    pub fn appkit_window_animation_behavior(window: *mut c_void) -> i64;
    pub fn appkit_window_set_animation_behavior(window: *mut c_void, behavior: i64);
    pub fn appkit_window_order_front(window: *mut c_void);
    pub fn appkit_window_make_key_and_order_front(window: *mut c_void);
    pub fn appkit_window_close(window: *mut c_void);

    pub fn appkit_screen_main() -> *mut c_void;
    pub fn appkit_screen_count() -> i64;
    pub fn appkit_screen_at(index: i64) -> *mut c_void;
    pub fn appkit_screen_frame(screen: *mut c_void, out: *mut CGRect);
    pub fn appkit_screen_visible_frame(screen: *mut c_void, out: *mut CGRect);
    pub fn appkit_screen_backing_scale_factor(screen: *mut c_void) -> f64;
    pub fn appkit_screen_display_id(screen: *mut c_void, out: *mut u32) -> bool;
    pub fn appkit_screen_localized_name(screen: *mut c_void) -> *mut c_char;

    pub fn appkit_application_shared() -> *mut c_void;
    pub fn appkit_application_set_application_icon_image(
        application: *mut c_void,
        image: *mut c_void,
    );
    pub fn appkit_application_set_activation_policy(application: *mut c_void, policy: i64) -> bool;
    pub fn appkit_application_activate(application: *mut c_void);
    pub fn appkit_application_run(application: *mut c_void);
    pub fn appkit_application_terminate(application: *mut c_void);
    pub fn appkit_application_order_front_standard_about_panel(
        application: *mut c_void,
        application_name: *const c_char,
        application_version: *const c_char,
        version: *const c_char,
    );

    pub fn appkit_image_from_data(bytes: *const u8, length: usize) -> *mut c_void;
    pub fn appkit_image_size(image: *mut c_void, out: *mut CGSize);

    pub fn appkit_scroller_preferred_scroller_style() -> i64;

    pub fn appkit_workspace_accessibility_display_should_reduce_motion() -> bool;
    pub fn appkit_workspace_accessibility_display_should_reduce_transparency() -> bool;
    pub fn appkit_workspace_accessibility_display_should_increase_contrast() -> bool;
    pub fn appkit_workspace_accessibility_display_should_differentiate_without_color() -> bool;
    pub fn appkit_workspace_accessibility_display_should_invert_colors() -> bool;
    pub fn appkit_workspace_is_voice_over_enabled() -> bool;
}
