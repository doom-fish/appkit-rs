use crate::ffi;

#[derive(Debug, Clone, Copy)]
pub struct Workspace;

impl Workspace {
    #[must_use]
    pub fn accessibility_display_should_reduce_motion() -> bool {
        unsafe { ffi::appkit_workspace_accessibility_display_should_reduce_motion() }
    }

    #[must_use]
    pub fn accessibility_display_should_reduce_transparency() -> bool {
        unsafe { ffi::appkit_workspace_accessibility_display_should_reduce_transparency() }
    }

    #[must_use]
    pub fn accessibility_display_should_increase_contrast() -> bool {
        unsafe { ffi::appkit_workspace_accessibility_display_should_increase_contrast() }
    }

    #[must_use]
    pub fn accessibility_display_should_differentiate_without_color() -> bool {
        unsafe { ffi::appkit_workspace_accessibility_display_should_differentiate_without_color() }
    }

    #[must_use]
    pub fn accessibility_display_should_invert_colors() -> bool {
        unsafe { ffi::appkit_workspace_accessibility_display_should_invert_colors() }
    }

    #[must_use]
    pub fn is_voice_over_enabled() -> bool {
        unsafe { ffi::appkit_workspace_is_voice_over_enabled() }
    }
}
