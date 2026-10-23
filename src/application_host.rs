use core::cell::{Cell, RefCell};
use core::ffi::c_void;
use core::ptr::NonNull;
use std::collections::VecDeque;
use std::rc::Rc;

use doom_fish_utils::panic_safe::catch_user_panic_result;

use crate::callbacks::{CallbackRegistration, CallbackTableHeader, HandlerSlot, RegistrationToken};
use crate::ffi;
use crate::main_thread::MainThreadMarker;
use crate::object::Handle;

pub const MENU_ITEM_ACTION_SELECTOR: &str = "handleAppKitMenuItem:";

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuAction {
    Other = -1,
    MenuItem = 0,
    Cut = 1,
    Copy = 2,
    Paste = 3,
    SelectAll = 4,
    Undo = 5,
    Redo = 6,
}

impl MenuAction {
    const fn from_raw(value: i32) -> Self {
        match value {
            0 => Self::MenuItem,
            1 => Self::Cut,
            2 => Self::Copy,
            3 => Self::Paste,
            4 => Self::SelectAll,
            5 => Self::Undo,
            6 => Self::Redo,
            _ => Self::Other,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RetainedMenu {
    handle: Handle,
    _main_thread: MainThreadMarker,
}

impl RetainedMenu {
    #[must_use]
    pub unsafe fn from_retained_ptr(
        menu: *mut c_void,
        main_thread: MainThreadMarker,
    ) -> Option<Self> {
        Some(Self {
            handle: unsafe { Handle::from_retained(menu) }?,
            _main_thread: main_thread,
        })
    }

    #[must_use]
    pub const fn as_ptr(&self) -> *mut c_void {
        self.handle.as_ptr()
    }

    const fn into_retained_ptr(self) -> *mut c_void {
        let pointer = self.as_ptr();
        core::mem::forget(self);
        pointer
    }
}

#[derive(Default)]
pub struct ApplicationHostHandlers {
    pub will_finish_launching: HandlerSlot<Box<dyn FnMut()>>,
    pub did_finish_launching: HandlerSlot<Box<dyn FnMut()>>,
    pub should_handle_reopen: HandlerSlot<Box<dyn FnMut(bool) -> bool>>,
    pub should_terminate: HandlerSlot<Box<dyn FnMut() -> bool>>,
    pub will_terminate: HandlerSlot<Box<dyn FnMut()>>,
    pub open_urls: HandlerSlot<Box<dyn FnMut(Vec<String>)>>,
    pub dock_menu_retained: HandlerSlot<Box<dyn FnMut() -> Option<RetainedMenu>>>,
    pub menu_item_action: HandlerSlot<Box<dyn FnMut(MenuAction, i64)>>,
    pub validate_menu_item: HandlerSlot<Box<dyn FnMut(MenuAction, i64) -> bool>>,
    pub menu_will_open: HandlerSlot<Box<dyn FnMut()>>,
    pub screen_parameters_changed: HandlerSlot<Box<dyn FnMut()>>,
}

enum DeferredNotification {
    WillFinishLaunching,
    DidFinishLaunching,
    WillTerminate,
    OpenUrls(Vec<String>),
    MenuItemAction(MenuAction, i64),
    MenuWillOpen,
    ScreenParametersChanged,
}

struct ApplicationHostState {
    handlers: ApplicationHostHandlers,
    alive: Cell<bool>,
    running: Cell<bool>,
    pending: RefCell<VecDeque<DeferredNotification>>,
}

struct CallbackTurn<'a>(&'a ApplicationHostState);

impl Drop for CallbackTurn<'_> {
    fn drop(&mut self) {
        self.0.pending.borrow_mut().clear();
        self.0.running.set(false);
    }
}

impl ApplicationHostState {
    fn enter(&self) -> Option<CallbackTurn<'_>> {
        if !self.alive.get() || self.running.replace(true) {
            return None;
        }
        Some(CallbackTurn(self))
    }

    fn notify(&self, event: DeferredNotification) {
        if !self.alive.get() {
            return;
        }
        if self.running.get() {
            self.pending.borrow_mut().push_back(event);
            return;
        }
        let Some(_turn) = self.enter() else {
            return;
        };
        self.invoke_notification(event);
        self.drain_pending();
    }

    fn query<R>(&self, handler: impl FnOnce(&ApplicationHostHandlers) -> R) -> Option<R> {
        let _turn = self.enter()?;
        let result = handler(&self.handlers);
        self.drain_pending();
        self.alive.get().then_some(result)
    }

    fn drain_pending(&self) {
        while self.alive.get() {
            let Some(event) = self.pending.borrow_mut().pop_front() else {
                break;
            };
            self.invoke_notification(event);
        }
    }

    fn invoke_notification(&self, event: DeferredNotification) {
        match event {
            DeferredNotification::WillFinishLaunching => {
                let _ = self
                    .handlers
                    .will_finish_launching
                    .invoke("AppKit will finish launching handler", |handler| handler());
            }
            DeferredNotification::DidFinishLaunching => {
                let _ = self
                    .handlers
                    .did_finish_launching
                    .invoke("AppKit did finish launching handler", |handler| handler());
            }
            DeferredNotification::WillTerminate => {
                let _ = self
                    .handlers
                    .will_terminate
                    .invoke("AppKit will terminate handler", |handler| handler());
            }
            DeferredNotification::OpenUrls(urls) => {
                let _ = self
                    .handlers
                    .open_urls
                    .invoke("AppKit open URLs handler", |handler| handler(urls));
            }
            DeferredNotification::MenuItemAction(action, tag) => {
                let _ = self.handlers.menu_item_action.invoke(
                    "AppKit menu item action handler",
                    |handler| {
                        handler(action, tag);
                    },
                );
            }
            DeferredNotification::MenuWillOpen => {
                let _ = self
                    .handlers
                    .menu_will_open
                    .invoke("AppKit menu will open handler", |handler| handler());
            }
            DeferredNotification::ScreenParametersChanged => {
                let _ = self
                    .handlers
                    .screen_parameters_changed
                    .invoke("AppKit screen parameters handler", |handler| handler());
            }
        }
    }
}

pub struct ApplicationHost {
    registration: CallbackRegistration<ApplicationHostState>,
    delegate: Handle,
    _main_thread: MainThreadMarker,
}

impl ApplicationHost {
    #[must_use]
    pub fn install(
        main_thread: MainThreadMarker,
        handlers: ApplicationHostHandlers,
    ) -> Option<Self> {
        let state = Rc::new(ApplicationHostState {
            handlers,
            alive: Cell::new(true),
            running: Cell::new(false),
            pending: RefCell::new(VecDeque::new()),
        });
        let registration = CallbackRegistration::new(main_thread, state);
        let table = ApplicationHostCallbackTableV1::new();
        let delegate = unsafe {
            Handle::from_retained(ffi::application_host::appkit_application_host_install_v1(
                registration.token().get(),
                &raw const table,
            ))
        }?;
        Some(Self {
            registration,
            delegate,
            _main_thread: main_thread,
        })
    }

    #[must_use]
    pub fn handlers(&self) -> &ApplicationHostHandlers {
        &self.registration.state().handlers
    }

    #[must_use]
    pub const fn delegate_borrowed_ptr(&self) -> *mut c_void {
        self.delegate.as_ptr()
    }
}

impl Drop for ApplicationHost {
    fn drop(&mut self) {
        self.registration.state().alive.set(false);
        let _ = self.registration.deactivate();
        unsafe { ffi::application_host::appkit_application_host_uninstall(self.delegate.as_ptr()) };
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ApplicationHostCallbackTableV1 {
    header: CallbackTableHeader,
    deactivate: extern "C" fn(u64),
    will_finish_launching: extern "C" fn(u64),
    did_finish_launching: extern "C" fn(u64),
    should_handle_reopen: extern "C" fn(u64, bool) -> bool,
    should_terminate: extern "C" fn(u64) -> bool,
    will_terminate: extern "C" fn(u64),
    open_urls: extern "C" fn(u64, *const u8, usize, *const Utf8Range, usize),
    dock_menu_retained: extern "C" fn(u64) -> *mut c_void,
    menu_item_action: extern "C" fn(u64, i64, i32),
    validate_menu_item: extern "C" fn(u64, i64, i32) -> bool,
    menu_will_open: extern "C" fn(u64),
    screen_parameters_changed: extern "C" fn(u64),
}

impl ApplicationHostCallbackTableV1 {
    const fn new() -> Self {
        Self {
            header: CallbackTableHeader {
                version: 1,
                size: size_of::<Self>(),
            },
            deactivate: deactivate_from_swift,
            will_finish_launching,
            did_finish_launching,
            should_handle_reopen,
            should_terminate,
            will_terminate,
            open_urls,
            dock_menu_retained,
            menu_item_action,
            validate_menu_item,
            menu_will_open,
            screen_parameters_changed,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Utf8Range {
    offset: usize,
    length: usize,
}

fn with_state<R>(
    token: u64,
    default: R,
    site: &str,
    callback: impl FnOnce(&ApplicationHostState) -> R,
) -> R {
    catch_user_panic_result(site, || {
        let main_thread = MainThreadMarker::new()?;
        let token = RegistrationToken::from_raw(token)?;
        let state = CallbackRegistration::<ApplicationHostState>::snapshot(token, main_thread)?;
        state.alive.get().then(|| callback(&state))
    })
    .flatten()
    .unwrap_or(default)
}

extern "C" fn deactivate_from_swift(raw_token: u64) {
    let _ = catch_user_panic_result("AppKit application host deactivation", || {
        if let (Some(main_thread), Some(token)) = (
            MainThreadMarker::new(),
            RegistrationToken::from_raw(raw_token),
        ) {
            if let Some(state) =
                CallbackRegistration::<ApplicationHostState>::snapshot(token, main_thread)
            {
                state.alive.set(false);
            }
        }
    });
}

extern "C" fn will_finish_launching(token: u64) {
    with_state(
        token,
        (),
        "AppKit will finish launching callback",
        |state| {
            state.notify(DeferredNotification::WillFinishLaunching);
        },
    );
}

extern "C" fn did_finish_launching(token: u64) {
    with_state(token, (), "AppKit did finish launching callback", |state| {
        state.notify(DeferredNotification::DidFinishLaunching);
    });
}

extern "C" fn should_handle_reopen(token: u64, has_visible_windows: bool) -> bool {
    with_state(token, false, "AppKit reopen callback", |state| {
        state
            .query(|handlers| {
                handlers
                    .should_handle_reopen
                    .invoke("AppKit reopen handler", |handler| {
                        handler(has_visible_windows)
                    })
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    })
}

extern "C" fn should_terminate(token: u64) -> bool {
    with_state(token, true, "AppKit should terminate callback", |state| {
        state
            .query(|handlers| {
                handlers
                    .should_terminate
                    .invoke("AppKit should terminate handler", |handler| handler())
                    .unwrap_or(true)
            })
            .unwrap_or(false)
    })
}

extern "C" fn will_terminate(token: u64) {
    with_state(token, (), "AppKit will terminate callback", |state| {
        state.notify(DeferredNotification::WillTerminate);
    });
}

unsafe fn copy_urls(
    bytes: *const u8,
    byte_length: usize,
    ranges: *const Utf8Range,
    count: usize,
) -> Option<Vec<String>> {
    if byte_length > isize::MAX as usize || count > isize::MAX as usize / size_of::<Utf8Range>() {
        return None;
    }
    let bytes = if byte_length == 0 {
        &[][..]
    } else {
        let bytes = NonNull::new(bytes.cast_mut())?;
        unsafe { core::slice::from_raw_parts(bytes.as_ptr(), byte_length) }
    };
    let ranges = if count == 0 {
        &[][..]
    } else {
        let ranges = NonNull::new(ranges.cast_mut())?;
        unsafe { core::slice::from_raw_parts(ranges.as_ptr(), count) }
    };
    ranges
        .iter()
        .map(|range| {
            let end = range.offset.checked_add(range.length)?;
            core::str::from_utf8(bytes.get(range.offset..end)?)
                .ok()
                .map(str::to_owned)
        })
        .collect()
}

extern "C" fn open_urls(
    token: u64,
    bytes: *const u8,
    byte_length: usize,
    ranges: *const Utf8Range,
    count: usize,
) {
    with_state(token, (), "AppKit open URLs callback", |state| {
        if let Some(urls) = unsafe { copy_urls(bytes, byte_length, ranges, count) } {
            state.notify(DeferredNotification::OpenUrls(urls));
        }
    });
}

extern "C" fn dock_menu_retained(token: u64) -> *mut c_void {
    with_state(
        token,
        core::ptr::null_mut(),
        "AppKit Dock menu callback",
        |state| {
            state
                .query(|handlers| {
                    handlers
                        .dock_menu_retained
                        .invoke("AppKit Dock menu handler", |handler| handler())
                        .flatten()
                })
                .flatten()
                .map_or(core::ptr::null_mut(), RetainedMenu::into_retained_ptr)
        },
    )
}

extern "C" fn menu_item_action(token: u64, tag: i64, action: i32) {
    with_state(token, (), "AppKit menu item action callback", |state| {
        state.notify(DeferredNotification::MenuItemAction(
            MenuAction::from_raw(action),
            tag,
        ));
    });
}

extern "C" fn validate_menu_item(token: u64, tag: i64, action: i32) -> bool {
    with_state(token, false, "AppKit menu validation callback", |state| {
        state
            .query(|handlers| {
                handlers
                    .validate_menu_item
                    .invoke("AppKit menu validation handler", |handler| {
                        handler(MenuAction::from_raw(action), tag)
                    })
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    })
}

extern "C" fn menu_will_open(token: u64) {
    with_state(token, (), "AppKit menu will open callback", |state| {
        state.notify(DeferredNotification::MenuWillOpen);
    });
}

extern "C" fn screen_parameters_changed(token: u64) {
    with_state(token, (), "AppKit screen parameters callback", |state| {
        state.notify(DeferredNotification::ScreenParametersChanged);
    });
}
