use crate::ffi;
use crate::main_thread::MainThreadMarker;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScrollerStyle {
    Legacy,
    Overlay,
}

#[derive(Debug, Clone, Copy)]
pub struct Scroller;

impl Scroller {
    #[must_use]
    pub fn preferred_scroller_style(_main_thread: MainThreadMarker) -> ScrollerStyle {
        if unsafe { ffi::appkit_scroller_preferred_scroller_style() } == 0 {
            ScrollerStyle::Legacy
        } else {
            ScrollerStyle::Overlay
        }
    }
}
