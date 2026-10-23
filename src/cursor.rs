use core::ffi::c_void;

use apple_cf::cg::{CGPoint, CGSize};

use crate::ffi;
use crate::image::Image;
use crate::main_thread::MainThreadMarker;
use crate::object::Handle;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CursorKind {
    Arrow = 0,
    IBeam = 1,
    IBeamForVerticalLayout = 2,
    Crosshair = 3,
    OpenHand = 4,
    ClosedHand = 5,
    PointingHand = 6,
    ResizeLeftRight = 7,
    ResizeUpDown = 8,
    ResizeLeft = 9,
    ResizeRight = 10,
    ResizeUp = 11,
    ResizeDown = 12,
    ResizeColumn = 13,
    ResizeRow = 14,
    ResizeUpLeftDownRight = 15,
    ResizeUpRightDownLeft = 16,
    OperationNotAllowed = 17,
    DragLink = 18,
    DragCopy = 19,
    ContextualMenu = 20,
}

#[derive(Debug, Clone)]
pub struct Cursor {
    handle: Handle,
    _main_thread: MainThreadMarker,
}

impl Cursor {
    #[must_use]
    pub fn system(main_thread: MainThreadMarker, kind: CursorKind) -> Self {
        let handle =
            unsafe { Handle::from_retained(ffi::cursor::appkit_cursor_system(kind as i32)) }
                .unwrap_or_else(|| panic!("AppKit provides a {kind:?} system cursor"));
        Self {
            handle,
            _main_thread: main_thread,
        }
    }

    #[must_use]
    pub fn current(main_thread: MainThreadMarker) -> Option<Self> {
        let handle = unsafe { Handle::from_retained(ffi::cursor::appkit_cursor_current()) }?;
        Some(Self {
            handle,
            _main_thread: main_thread,
        })
    }

    #[must_use]
    pub fn from_image(
        main_thread: MainThreadMarker,
        image: &Image,
        size: CGSize,
        hot_spot: CGPoint,
    ) -> Option<Self> {
        let handle = unsafe {
            Handle::from_retained(ffi::cursor::appkit_cursor_from_image(
                image.as_ptr(),
                size.width,
                size.height,
                hot_spot.x,
                hot_spot.y,
            ))
        }?;
        Some(Self {
            handle,
            _main_thread: main_thread,
        })
    }

    #[must_use]
    pub const fn as_ptr(&self) -> *mut c_void {
        self.handle.as_ptr()
    }

    pub fn set(&self) {
        unsafe { ffi::cursor::appkit_cursor_set(self.as_ptr()) };
    }

    pub fn hide(_main_thread: MainThreadMarker) {
        unsafe { ffi::cursor::appkit_cursor_hide() };
    }

    pub fn unhide(_main_thread: MainThreadMarker) {
        unsafe { ffi::cursor::appkit_cursor_unhide() };
    }

    pub fn set_hidden_until_mouse_moves(_main_thread: MainThreadMarker, hidden: bool) {
        unsafe { ffi::cursor::appkit_cursor_set_hidden_until_mouse_moves(hidden) };
    }
}
