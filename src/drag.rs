use core::cell::{Cell, RefCell};
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};
use core::ptr::NonNull;
use core::slice;
use std::collections::VecDeque;
use std::ffi::OsString;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use apple_cf::cg::CGPoint;
use doom_fish_utils::panic_safe::catch_user_panic_result;

use crate::callbacks::{
    CallbackRegistration, CallbackTableHeader, CallbackTableV1, HandlerSlot, RegistrationToken,
};
use crate::event::{Event, EventType};
use crate::ffi::drag::{self as ffi, DragCallbackTableV1};
use crate::host_view::HostedView;
use crate::host_window::HostedWindow;
use crate::main_thread::MainThreadMarker;
use crate::pasteboard::Pasteboard;
use crate::view::View;
use crate::window::Window;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DragOperation(u64);

impl DragOperation {
    pub const NONE: Self = Self(0);
    pub const COPY: Self = Self(1);
    pub const LINK: Self = Self(2);
    pub const GENERIC: Self = Self(4);
    pub const PRIVATE: Self = Self(8);
    pub const MOVE: Self = Self(16);
    pub const DELETE: Self = Self(32);

    #[must_use]
    pub const fn bits(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn from_bits_retain(bits: u64) -> Self {
        Self(bits)
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl BitOr for DragOperation {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for DragOperation {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for DragOperation {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for DragOperation {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragContext {
    OutsideApplication,
    WithinApplication,
    Other(i64),
}

impl DragContext {
    const fn from_raw(context: i64) -> Self {
        match context {
            0 => Self::OutsideApplication,
            1 => Self::WithinApplication,
            other => Self::Other(other),
        }
    }

    const fn raw(self) -> i64 {
        match self {
            Self::OutsideApplication => 0,
            Self::WithinApplication => 1,
            Self::Other(context) => context,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DragInfo {
    pub position_in_window: CGPoint,
    pub source_operations: DragOperation,
    pub is_source_window: bool,
    pub paths: Option<Vec<PathBuf>>,
}

#[derive(Clone, Debug)]
pub enum DragEvent {
    Entered(DragInfo),
    Updated(DragInfo),
    Exited(DragInfo),
    Performed(DragInfo),
    Concluded(DragInfo),
    SourceMask(DragContext),
    SourceEnded {
        screen_position: CGPoint,
        operation: DragOperation,
    },
}

pub enum DragItem<'a> {
    File { path: &'a Path, is_directory: bool },
    Text(&'a str),
}

pub type DragHandler = Box<dyn FnMut(DragEvent) -> DragOperation>;

#[derive(Default)]
struct DragState {
    handler: HandlerSlot<DragHandler>,
    delivering: Cell<bool>,
    pending: RefCell<VecDeque<DragEvent>>,
}

struct DeliveryGuard<'a>(&'a DragState);

impl Drop for DeliveryGuard<'_> {
    fn drop(&mut self) {
        self.0.pending.borrow_mut().clear();
        self.0.delivering.set(false);
    }
}

impl DragState {
    fn deliver(
        &self,
        token: RegistrationToken,
        main_thread: MainThreadMarker,
        event: DragEvent,
    ) -> DragOperation {
        if self.delivering.get() && matches!(event, DragEvent::SourceMask(_)) {
            return DragOperation::NONE;
        }
        if self.delivering.replace(true) {
            self.pending.borrow_mut().push_back(event);
            return DragOperation::NONE;
        }
        let _guard = DeliveryGuard(self);
        let mut pending = Some(event);
        let mut first = true;
        let mut result = DragOperation::NONE;
        while let Some(event) = pending {
            if CallbackRegistration::<Self>::snapshot(token, main_thread).is_none() {
                break;
            }
            let response = self
                .handler
                .invoke("AppKit drag handler", |handler| handler(event))
                .unwrap_or(DragOperation::NONE);
            if first {
                result = response;
                first = false;
            }
            pending = self.pending.borrow_mut().pop_front();
        }
        result
    }
}

fn with_state<R>(
    raw_token: u64,
    callback: impl FnOnce(&DragState, RegistrationToken, MainThreadMarker) -> R,
) -> Option<R> {
    let main_thread = MainThreadMarker::new()?;
    let token = RegistrationToken::from_raw(raw_token)?;
    let state = CallbackRegistration::<DragState>::snapshot(token, main_thread)?;
    Some(callback(&state, token, main_thread))
}

extern "C" fn destination_callback(
    raw_token: u64,
    raw_phase: u32,
    raw_info: *const ffi::DragInfo,
) -> u64 {
    catch_user_panic_result("AppKit drag destination callback", || {
        let main_thread = MainThreadMarker::new()?;
        let token = RegistrationToken::from_raw(raw_token)?;
        let state = CallbackRegistration::<DragState>::snapshot(token, main_thread)?;
        let info = unsafe { DragInfo::copy_from_raw(raw_info) }?;
        let event = match raw_phase {
            1 => DragEvent::Entered(info),
            2 => DragEvent::Updated(info),
            3 => DragEvent::Exited(info),
            4 => DragEvent::Performed(info),
            5 => DragEvent::Concluded(info),
            _ => return None,
        };
        Some(state.deliver(token, main_thread, event).bits())
    })
    .flatten()
    .unwrap_or(0)
}

extern "C" fn source_mask_callback(raw_token: u64, context: i64) -> u64 {
    catch_user_panic_result("AppKit drag source mask callback", || {
        with_state(raw_token, |state, token, main_thread| {
            state
                .deliver(
                    token,
                    main_thread,
                    DragEvent::SourceMask(DragContext::from_raw(context)),
                )
                .bits()
        })
    })
    .flatten()
    .unwrap_or(0)
}

extern "C" fn source_end_callback(raw_token: u64, x: f64, y: f64, operation: u64) {
    let _ = catch_user_panic_result("AppKit drag source end callback", || {
        let _ = with_state(raw_token, |state, token, main_thread| {
            state.deliver(
                token,
                main_thread,
                DragEvent::SourceEnded {
                    screen_position: CGPoint::new(x, y),
                    operation: DragOperation::from_bits_retain(operation),
                },
            )
        });
    });
}

impl DragInfo {
    unsafe fn copy_from_raw(info: *const ffi::DragInfo) -> Option<Self> {
        let info = unsafe { info.as_ref() }?;
        if !info.x.is_finite() || !info.y.is_finite() {
            return None;
        }
        let paths = if info.has_paths {
            if info.path_count == 0 {
                Some(Vec::new())
            } else {
                let count = info.path_count;
                if count > isize::MAX as usize / size_of::<ffi::DragPath>() {
                    return None;
                }
                let paths = NonNull::new(info.paths.cast_mut())?;
                let paths = unsafe { slice::from_raw_parts(paths.as_ptr(), count) };
                Some(
                    paths
                        .iter()
                        .map(|path| {
                            if path.length == 0 {
                                return Some(PathBuf::new());
                            }
                            if path.length > isize::MAX as usize {
                                return None;
                            }
                            let bytes = NonNull::new(path.bytes.cast_mut())?;
                            let bytes =
                                unsafe { slice::from_raw_parts(bytes.as_ptr(), path.length) };
                            Some(PathBuf::from(OsString::from_vec(bytes.to_vec())))
                        })
                        .collect::<Option<Vec<_>>>()?,
                )
            }
        } else {
            None
        };
        Some(Self {
            position_in_window: CGPoint::new(info.x, info.y),
            source_operations: DragOperation::from_bits_retain(info.source_operation_mask),
            is_source_window: info.is_source_window,
            paths,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum DragSelector {
    Entered = 1,
    Updated = 2,
    Exited = 3,
    Prepare = 4,
    Perform = 5,
    Conclude = 6,
}

pub struct DragRegistration {
    registration: CallbackRegistration<DragState>,
    window: Window,
    view: View,
    _main_thread: MainThreadMarker,
}

impl DragRegistration {
    #[must_use]
    pub fn new(
        main_thread: MainThreadMarker,
        window: &HostedWindow,
        view: &HostedView,
    ) -> Option<Self> {
        let registration = CallbackRegistration::new(main_thread, Rc::new(DragState::default()));
        let table = DragCallbackTableV1 {
            header: CallbackTableHeader {
                version: 1,
                size: size_of::<DragCallbackTableV1>(),
            },
            deactivate: CallbackTableV1::new(None, None).deactivate,
            destination: destination_callback,
            source_mask: source_mask_callback,
            source_end: source_end_callback,
        };
        let window = window.as_window();
        let view = view.as_view();
        if !unsafe {
            ffi::appkit_drag_attach(
                window.as_ptr(),
                view.as_ptr(),
                registration.token().get(),
                &raw const table,
            )
        } {
            return None;
        }
        Some(Self {
            registration,
            window,
            view,
            _main_thread: main_thread,
        })
    }

    pub fn set_handler(&self, handler: Option<DragHandler>) {
        let _ = self.registration.state().handler.replace(handler);
    }

    pub fn deactivate_callbacks(&self) {
        let _ = self.registration.deactivate();
        unsafe { ffi::appkit_drag_detach(self.window.as_ptr()) };
    }

    #[must_use]
    pub fn start(&self, mouse_down: &Event, items: &[DragItem<'_>]) -> bool {
        if mouse_down.as_ref().event_type() != EventType::LeftMouseDown {
            return false;
        }
        let native = Self::native_items(items);
        !native.is_empty()
            && unsafe {
                ffi::appkit_drag_start(
                    self.window.as_ptr(),
                    self.view.as_ptr(),
                    mouse_down.as_ptr(),
                    native.as_ptr(),
                    native.len(),
                )
            }
    }

    #[doc(hidden)]
    #[must_use]
    pub fn verify_items(&self, mouse_down: &Event, items: &[DragItem<'_>]) -> usize {
        if mouse_down.as_ref().event_type() != EventType::LeftMouseDown {
            return 0;
        }
        let native = Self::native_items(items);
        unsafe {
            ffi::appkit_drag_verify_items(
                self.window.as_ptr(),
                self.view.as_ptr(),
                mouse_down.as_ptr(),
                native.as_ptr(),
                native.len(),
            )
        }
    }

    #[doc(hidden)]
    #[must_use]
    pub fn verify_destination(
        &self,
        pasteboard: &Pasteboard,
        position_in_window: CGPoint,
        source_operations: DragOperation,
        is_source_window: bool,
        selector: DragSelector,
    ) -> u64 {
        unsafe {
            ffi::appkit_drag_verify_destination(
                self.window.as_ptr(),
                pasteboard.as_ptr(),
                position_in_window.x,
                position_in_window.y,
                source_operations.bits(),
                is_source_window,
                selector as u32,
            )
        }
    }

    #[doc(hidden)]
    #[must_use]
    pub fn verify_source_mask(&self, context: DragContext) -> DragOperation {
        DragOperation::from_bits_retain(unsafe {
            ffi::appkit_drag_verify_source_mask(self.window.as_ptr(), context.raw())
        })
    }

    #[doc(hidden)]
    #[must_use]
    pub fn verify_source_end(&self, screen_position: CGPoint, operation: DragOperation) -> bool {
        unsafe {
            ffi::appkit_drag_verify_source_end(
                self.window.as_ptr(),
                screen_position.x,
                screen_position.y,
                operation.bits(),
            )
        }
    }

    fn native_items(items: &[DragItem<'_>]) -> Vec<ffi::DragItem> {
        items
            .iter()
            .filter_map(|item| {
                let (kind, bytes, is_directory) = match item {
                    DragItem::File { path, is_directory } => {
                        (1, path.as_os_str().as_bytes(), *is_directory)
                    }
                    DragItem::Text(text) => (2, text.as_bytes(), false),
                };
                (kind == 2 || (!bytes.is_empty() && !bytes.contains(&0))).then_some(ffi::DragItem {
                    kind,
                    bytes: bytes.as_ptr(),
                    length: bytes.len(),
                    is_directory,
                })
            })
            .collect()
    }
}

impl Drop for DragRegistration {
    fn drop(&mut self) {
        self.deactivate_callbacks();
    }
}
