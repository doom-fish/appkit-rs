use core::ffi::c_void;

extern "C" {
    pub fn appkit_blur_new_for_window(window: *mut c_void) -> *mut c_void;
    pub fn appkit_blur_material(view: *mut c_void) -> i32;
    pub fn appkit_blur_set_material(view: *mut c_void, material: i32);
    pub fn appkit_blur_blending_mode(view: *mut c_void) -> i32;
    pub fn appkit_blur_set_blending_mode(view: *mut c_void, mode: i32);
    pub fn appkit_blur_state(view: *mut c_void) -> i32;
    pub fn appkit_blur_set_state(view: *mut c_void, state: i32);
    pub fn appkit_blur_is_emphasized(view: *mut c_void) -> bool;
    pub fn appkit_blur_set_emphasized(view: *mut c_void, emphasized: bool);
    pub fn appkit_blur_attach_below(
        view: *mut c_void,
        window: *mut c_void,
        hosted_view: *mut c_void,
    ) -> bool;
    pub fn appkit_blur_detach(view: *mut c_void);
}
