use core::cell::{Cell, RefCell};
use core::ffi::c_void;
use std::collections::VecDeque;
use std::ffi::CString;
use std::fmt;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::rc::Rc;

use apple_cf::cg::CGSize;
use doom_fish_utils::panic_safe::catch_user_panic_result;

use crate::callbacks::{
    copy_utf8, CallbackRegistration, CallbackTableHeader, HandlerSlot, RegistrationToken,
};
use crate::error::AppKitError;
use crate::ffi::{self, workspace as native};
use crate::main_thread::MainThreadMarker;
use crate::object::Handle;
use crate::util::{c_string, read_size};

#[derive(Debug, Clone, Copy)]
pub struct Workspace;

#[derive(Debug, Clone)]
pub struct WorkspaceIcon {
    handle: Handle,
}

impl WorkspaceIcon {
    #[must_use]
    pub const fn as_ptr(&self) -> *mut c_void {
        self.handle.as_ptr()
    }

    #[must_use]
    pub fn size(&self) -> CGSize {
        read_size(|out| unsafe { ffi::appkit_image_size(self.as_ptr(), out) })
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerEvent {
    WillSleep,
    DidWake,
}

struct PowerState {
    handler: HandlerSlot<Box<dyn FnMut(PowerEvent)>>,
    queued: RefCell<VecDeque<PowerEvent>>,
    delivering: Cell<bool>,
}

impl PowerState {
    fn notify(&self, event: PowerEvent) {
        self.queued.borrow_mut().push_back(event);
        if self.delivering.replace(true) {
            return;
        }
        loop {
            let event = self.queued.borrow_mut().pop_front();
            let Some(event) = event else {
                break;
            };
            let _ = self
                .handler
                .invoke("AppKit workspace power notification", |handler| {
                    handler(event);
                });
        }
        self.delivering.set(false);
    }
}

#[must_use = "the observer must remain alive to receive notifications"]
pub struct PowerObserver {
    registration: CallbackRegistration<PowerState>,
    handle: Handle,
}

impl PowerObserver {
    #[must_use]
    pub const fn as_ptr(&self) -> *mut c_void {
        self.handle.as_ptr()
    }
}

impl Drop for PowerObserver {
    fn drop(&mut self) {
        let _ = self.registration.deactivate();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemeRegistrationError {
    InvalidScheme,
    UnsupportedSystem,
    MissingBundleIdentifier,
    ApplicationNotInstalled,
    NativeFailure,
}

impl fmt::Display for SchemeRegistrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidScheme => formatter.write_str("the URL scheme is not valid"),
            Self::UnsupportedSystem => formatter.write_str("macOS 12 or later is required"),
            Self::MissingBundleIdentifier => {
                formatter.write_str("the application has no bundle identifier")
            }
            Self::ApplicationNotInstalled => {
                formatter.write_str("the application is not installed")
            }
            Self::NativeFailure => {
                formatter.write_str("the workspace cannot register the URL scheme")
            }
        }
    }
}

impl std::error::Error for SchemeRegistrationError {}

type SchemeCompletion = Box<dyn FnMut(Result<(), String>)>;

struct SchemeState {
    completion: HandlerSlot<SchemeCompletion>,
}

#[must_use = "the registration must remain alive until the completion callback"]
pub struct SchemeRegistration {
    registration: CallbackRegistration<SchemeState>,
    handle: Handle,
}

impl SchemeRegistration {
    #[must_use]
    pub const fn as_ptr(&self) -> *mut c_void {
        self.handle.as_ptr()
    }
}

impl Drop for SchemeRegistration {
    fn drop(&mut self) {
        let _ = self.registration.deactivate();
    }
}

extern "C" fn power_event(raw_token: u64, kind: i32) {
    let _ = catch_user_panic_result("AppKit workspace power callback", || {
        let Some(main_thread) = MainThreadMarker::new() else {
            return;
        };
        let Some(token) = RegistrationToken::from_raw(raw_token) else {
            return;
        };
        let Some(state) = CallbackRegistration::<PowerState>::snapshot(token, main_thread) else {
            return;
        };
        let event = match kind {
            0 => PowerEvent::WillSleep,
            1 => PowerEvent::DidWake,
            _ => return,
        };
        state.notify(event);
    });
}

extern "C" fn scheme_complete(raw_token: u64, success: bool, bytes: *const u8, length: usize) {
    let _ = catch_user_panic_result("AppKit workspace scheme callback", || {
        let Some(main_thread) = MainThreadMarker::new() else {
            return;
        };
        let Some(token) = RegistrationToken::from_raw(raw_token) else {
            return;
        };
        let Some(state) = CallbackRegistration::<SchemeState>::snapshot(token, main_thread) else {
            return;
        };
        let result = if success {
            Ok(())
        } else {
            Err(unsafe { copy_utf8(bytes, length) }
                .filter(|message| !message.is_empty())
                .unwrap_or_else(|| "the URL scheme registration failed".to_owned()))
        };
        let _ = state
            .completion
            .invoke("AppKit workspace scheme completion", |completion| {
                completion(result);
            });
    });
}

fn file_system_path(path: &Path) -> Result<CString, AppKitError> {
    CString::new(path.as_os_str().as_bytes())
        .map_err(|error| AppKitError::InvalidArgument(error.to_string()))
}

impl Workspace {
    pub fn open_url(_main_thread: MainThreadMarker, url: &str) -> Result<bool, AppKitError> {
        let url = c_string(url)?;
        Ok(unsafe { native::appkit_workspace_open_url(url.as_ptr()) })
    }

    pub fn open_file(_main_thread: MainThreadMarker, path: &Path) -> Result<bool, AppKitError> {
        let path = file_system_path(path)?;
        Ok(unsafe { native::appkit_workspace_open_file(path.as_ptr()) })
    }

    pub fn reveal_paths(
        _main_thread: MainThreadMarker,
        paths: &[&Path],
    ) -> Result<(), AppKitError> {
        let strings = paths
            .iter()
            .map(|path| file_system_path(path))
            .collect::<Result<Vec<_>, _>>()?;
        let borrowed = strings
            .iter()
            .map(|path| native::FileSystemPath {
                bytes: path.as_ptr(),
            })
            .collect::<Vec<_>>();
        if !borrowed.is_empty() {
            unsafe { native::appkit_workspace_reveal_paths(borrowed.as_ptr(), borrowed.len()) };
        }
        Ok(())
    }

    pub fn icon_for_file(
        _main_thread: MainThreadMarker,
        path: &Path,
    ) -> Result<Option<WorkspaceIcon>, AppKitError> {
        let path = file_system_path(path)?;
        Ok(
            unsafe { Handle::from_retained(native::appkit_workspace_icon_for_file(path.as_ptr())) }
                .map(|handle| WorkspaceIcon { handle }),
        )
    }

    pub fn icon_for_file_type(
        _main_thread: MainThreadMarker,
        file_type: &str,
    ) -> Result<Option<WorkspaceIcon>, AppKitError> {
        let file_type = c_string(file_type)?;
        Ok(unsafe {
            Handle::from_retained(native::appkit_workspace_icon_for_file_type(
                file_type.as_ptr(),
            ))
        }
        .map(|handle| WorkspaceIcon { handle }))
    }

    pub fn icon_for_application(
        _main_thread: MainThreadMarker,
        bundle_identifier: &str,
    ) -> Result<Option<WorkspaceIcon>, AppKitError> {
        let bundle_identifier = c_string(bundle_identifier)?;
        Ok(unsafe {
            Handle::from_retained(native::appkit_workspace_icon_for_application(
                bundle_identifier.as_ptr(),
            ))
        }
        .map(|handle| WorkspaceIcon { handle }))
    }

    #[must_use]
    pub fn observe_power(
        main_thread: MainThreadMarker,
        handler: impl FnMut(PowerEvent) + 'static,
    ) -> Option<PowerObserver> {
        let state = Rc::new(PowerState {
            handler: HandlerSlot::new(),
            queued: RefCell::new(VecDeque::new()),
            delivering: Cell::new(false),
        });
        state.handler.replace(Some(Box::new(handler)));
        let registration = CallbackRegistration::new(main_thread, state);
        let table = native::PowerTableV1 {
            header: CallbackTableHeader {
                version: 1,
                size: size_of::<native::PowerTableV1>(),
            },
            event: Some(power_event),
        };
        let handle = unsafe {
            Handle::from_retained(native::appkit_workspace_power_observer_new(
                registration.token().get(),
                &raw const table,
            ))
        }?;
        Some(PowerObserver {
            registration,
            handle,
        })
    }

    pub fn register_default_url_scheme(
        main_thread: MainThreadMarker,
        scheme: &str,
        completion: impl FnOnce(Result<(), String>) + 'static,
    ) -> Result<SchemeRegistration, SchemeRegistrationError> {
        let mut characters = scheme.bytes();
        if !characters
            .next()
            .is_some_and(|first| first.is_ascii_alphabetic())
            || !characters.all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, b'+' | b'-' | b'.')
            })
        {
            return Err(SchemeRegistrationError::InvalidScheme);
        }
        let scheme = CString::new(scheme).map_err(|_| SchemeRegistrationError::InvalidScheme)?;
        let mut completion = Some(completion);
        let state = Rc::new(SchemeState {
            completion: HandlerSlot::new(),
        });
        state.completion.replace(Some(Box::new(move |result| {
            if let Some(completion) = completion.take() {
                completion(result);
            }
        })));
        let registration = CallbackRegistration::new(main_thread, state);
        let table = native::SchemeTableV1 {
            header: CallbackTableHeader {
                version: 1,
                size: size_of::<native::SchemeTableV1>(),
            },
            complete: Some(scheme_complete),
        };
        let mut status = 0;
        let handle = unsafe {
            native::appkit_workspace_register_url_scheme(
                scheme.as_ptr(),
                registration.token().get(),
                &raw const table,
                &raw mut status,
            )
        };
        let handle = unsafe { Handle::from_retained(handle) };
        match (handle, status) {
            (Some(handle), 0) => Ok(SchemeRegistration {
                registration,
                handle,
            }),
            (_, 1) => Err(SchemeRegistrationError::UnsupportedSystem),
            (_, 2) => Err(SchemeRegistrationError::MissingBundleIdentifier),
            (_, 3) => Err(SchemeRegistrationError::ApplicationNotInstalled),
            _ => Err(SchemeRegistrationError::NativeFailure),
        }
    }

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
