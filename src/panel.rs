use core::any::Any;
use core::cell::RefCell;
use core::ffi::c_void;
use core::ops::Deref;
use core::ptr::{self, NonNull};
use std::collections::HashMap;
use std::ffi::{CString, OsString};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use doom_fish_utils::panic_safe::catch_user_panic_result;

use crate::callbacks::{
    CallbackRegistration, CallbackTableHeader, CallbackTableV1, HandlerSlot, RegistrationToken,
};
use crate::error::AppKitError;
use crate::ffi;
use crate::main_thread::MainThreadMarker;
use crate::object::Handle;
use crate::util::{c_string, optional_c_string};

thread_local! {
    static DETACHED: RefCell<HashMap<RegistrationToken, Box<dyn Any>>> = RefCell::new(HashMap::new());
}

pub(crate) struct CompletionGuard<T: 'static> {
    registration: CallbackRegistration<T>,
    handle: Handle,
}

impl<T: 'static> CompletionGuard<T> {
    pub(crate) fn new(main_thread: MainThreadMarker, handle: Handle, state: Rc<T>) -> Self {
        Self {
            registration: CallbackRegistration::new(main_thread, state),
            handle,
        }
    }

    pub(crate) const fn token(&self) -> RegistrationToken {
        self.registration.token()
    }

    pub(crate) const fn as_ptr(&self) -> *mut c_void {
        self.handle.as_ptr()
    }

    pub(crate) fn detach(self) {
        let token = self.token();
        let previous =
            DETACHED.with(|detached| detached.borrow_mut().insert(token, Box::new(self)));
        drop(previous);
    }

    pub(crate) fn finish(token: RegistrationToken, main_thread: MainThreadMarker) -> Option<Rc<T>> {
        let state = CallbackRegistration::<T>::snapshot(token, main_thread);
        let detached = DETACHED
            .try_with(|detached| detached.borrow_mut().remove(&token))
            .ok()
            .flatten();
        drop(detached);
        state
    }
}

pub(crate) extern "C" fn deactivate_completion(raw_token: u64) {
    let _ = catch_user_panic_result("AppKit completion deactivation", || {
        if MainThreadMarker::new().is_none() {
            return;
        }
        if let Some(token) = RegistrationToken::from_raw(raw_token) {
            let detached = DETACHED
                .try_with(|detached| detached.borrow_mut().remove(&token))
                .ok()
                .flatten();
            drop(detached);
            (CallbackTableV1::for_state::<()>().deactivate)(raw_token);
        }
    });
}

type PanelHandler = Box<dyn FnMut(Option<Vec<PathBuf>>)>;

struct PanelCallbackState {
    handler: HandlerSlot<PanelHandler>,
}

impl PanelCallbackState {
    fn new(callback: impl FnOnce(Option<Vec<PathBuf>>) + 'static) -> Rc<Self> {
        let mut callback = Some(callback);
        let state = Rc::new(Self {
            handler: HandlerSlot::new(),
        });
        let _ = state.handler.replace(Some(Box::new(move |result| {
            if let Some(callback) = callback.take() {
                callback(result);
            }
        })));
        state
    }
}

#[derive(Debug)]
pub struct FilePanel {
    handle: Handle,
    main_thread: MainThreadMarker,
}

impl FilePanel {
    pub fn set_directory(&self, directory: Option<&Path>) -> Result<(), AppKitError> {
        let directory = directory
            .map(|path| CString::new(path.as_os_str().as_bytes()))
            .transpose()
            .map_err(|error| AppKitError::InvalidArgument(error.to_string()))?;
        let success = unsafe {
            ffi::panel::appkit_panel_set_directory(
                self.handle.as_ptr(),
                directory
                    .as_ref()
                    .map_or(ptr::null(), |path| path.as_ptr().cast()),
                directory.as_ref().map_or(0, |path| path.as_bytes().len()),
            )
        };
        if success {
            Ok(())
        } else {
            Err(AppKitError::InvalidArgument("panel directory".into()))
        }
    }

    pub fn set_prompt(&self, prompt: Option<&str>) -> Result<(), AppKitError> {
        let prompt = optional_c_string(prompt)?;
        unsafe {
            ffi::panel::appkit_panel_set_prompt(
                self.handle.as_ptr(),
                prompt
                    .as_ref()
                    .map_or(ptr::null(), |prompt| prompt.as_ptr()),
            );
        }
        Ok(())
    }

    pub fn set_message(&self, message: Option<&str>) -> Result<(), AppKitError> {
        let message = optional_c_string(message)?;
        unsafe {
            ffi::panel::appkit_panel_set_message(
                self.handle.as_ptr(),
                message
                    .as_ref()
                    .map_or(ptr::null(), |message| message.as_ptr()),
            );
        }
        Ok(())
    }

    pub fn set_can_create_directories(&self, can_create: bool) {
        unsafe {
            ffi::panel::appkit_panel_set_can_create_directories(self.handle.as_ptr(), can_create);
        }
    }

    pub fn set_shows_hidden_files(&self, shows: bool) {
        unsafe { ffi::panel::appkit_panel_set_shows_hidden_files(self.handle.as_ptr(), shows) };
    }
}

#[derive(Debug)]
pub struct OpenPanel(FilePanel);

impl OpenPanel {
    #[must_use]
    pub fn new(main_thread: MainThreadMarker) -> Self {
        let handle = unsafe { Handle::from_retained(ffi::panel::appkit_open_panel_new()) }
            .expect("AppKit creates an open panel");
        Self(FilePanel {
            handle,
            main_thread,
        })
    }

    pub fn set_can_choose_files(&self, can_choose: bool) {
        unsafe {
            ffi::panel::appkit_open_panel_set_can_choose_files(self.0.handle.as_ptr(), can_choose);
        };
    }

    pub fn set_can_choose_directories(&self, can_choose: bool) {
        unsafe {
            ffi::panel::appkit_open_panel_set_can_choose_directories(
                self.0.handle.as_ptr(),
                can_choose,
            );
        }
    }

    pub fn set_allows_multiple_selection(&self, allows: bool) {
        unsafe {
            ffi::panel::appkit_open_panel_set_allows_multiple_selection(
                self.0.handle.as_ptr(),
                allows,
            );
        }
    }

    pub fn set_resolves_aliases(&self, resolves: bool) {
        unsafe {
            ffi::panel::appkit_open_panel_set_resolves_aliases(self.0.handle.as_ptr(), resolves);
        };
    }

    #[must_use]
    pub fn begin(
        self,
        callback: impl FnOnce(Option<Vec<PathBuf>>) + 'static,
    ) -> Option<PanelSession> {
        let guard = CompletionGuard::new(
            self.0.main_thread,
            self.0.handle,
            PanelCallbackState::new(callback),
        );
        begin_panel(guard)
    }
}

impl Deref for OpenPanel {
    type Target = FilePanel;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct SavePanel(FilePanel);

impl SavePanel {
    #[must_use]
    pub fn new(main_thread: MainThreadMarker) -> Self {
        let handle = unsafe { Handle::from_retained(ffi::panel::appkit_save_panel_new()) }
            .expect("AppKit creates a save panel");
        Self(FilePanel {
            handle,
            main_thread,
        })
    }

    pub fn set_suggested_name(&self, name: &str) -> Result<(), AppKitError> {
        let name = c_string(name)?;
        unsafe {
            ffi::panel::appkit_save_panel_set_suggested_name(self.0.handle.as_ptr(), name.as_ptr());
        }
        Ok(())
    }

    #[must_use]
    pub fn begin(self, callback: impl FnOnce(Option<PathBuf>) + 'static) -> Option<PanelSession> {
        let guard = CompletionGuard::new(
            self.0.main_thread,
            self.0.handle,
            PanelCallbackState::new(move |paths| {
                callback(paths.and_then(|paths| paths.into_iter().next()));
            }),
        );
        begin_panel(guard)
    }
}

impl Deref for SavePanel {
    type Target = FilePanel;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct PanelSession {
    guard: CompletionGuard<PanelCallbackState>,
}

impl PanelSession {
    pub fn detach(self) {
        self.guard.detach();
    }
}

fn begin_panel(guard: CompletionGuard<PanelCallbackState>) -> Option<PanelSession> {
    let callbacks = ffi::panel::PanelCallbacksV1 {
        header: CallbackTableHeader {
            version: 1,
            size: size_of::<ffi::panel::PanelCallbacksV1>(),
        },
        deactivate: deactivate_completion,
        complete: panel_completed,
    };
    let started = unsafe {
        ffi::panel::appkit_panel_begin(guard.as_ptr(), guard.token().get(), &raw const callbacks)
    };
    started.then_some(PanelSession { guard })
}

extern "C" fn panel_completed(
    raw_token: u64,
    accepted: bool,
    paths: *const ffi::panel::FileSystemPath,
    count: usize,
) {
    let _ = catch_user_panic_result("AppKit panel completion", || {
        let Some(main_thread) = MainThreadMarker::new() else {
            return;
        };
        let Some(token) = RegistrationToken::from_raw(raw_token) else {
            return;
        };
        complete_on_main(main_thread, token, accepted, paths, count);
    });
}

fn complete_on_main(
    main_thread: MainThreadMarker,
    token: RegistrationToken,
    accepted: bool,
    paths: *const ffi::panel::FileSystemPath,
    count: usize,
) {
    let Some(state) = CompletionGuard::<PanelCallbackState>::finish(token, main_thread) else {
        return;
    };
    let result = if accepted {
        unsafe { copy_paths(paths, count) }
    } else {
        None
    };
    let _ = state
        .handler
        .invoke("AppKit panel handler", |callback| callback(result));
}

unsafe fn copy_paths(
    paths: *const ffi::panel::FileSystemPath,
    count: usize,
) -> Option<Vec<PathBuf>> {
    if count == 0 {
        return Some(Vec::new());
    }
    if count > isize::MAX as usize / size_of::<ffi::panel::FileSystemPath>() {
        return None;
    }
    let paths = NonNull::new(paths.cast_mut())?;
    let paths = unsafe { core::slice::from_raw_parts(paths.as_ptr(), count) };
    paths
        .iter()
        .map(|path| {
            if path.length == 0 || path.length > isize::MAX as usize {
                return None;
            }
            let bytes = NonNull::new(path.bytes.cast_mut())?;
            let bytes = unsafe { core::slice::from_raw_parts(bytes.as_ptr(), path.length) };
            Some(PathBuf::from(OsString::from_vec(bytes.to_vec())))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use core::cell::{Cell, RefCell};
    use core::ptr;
    use std::os::unix::ffi::OsStrExt;
    use std::path::{Path, PathBuf};
    use std::rc::Rc;

    use super::{complete_on_main, PanelCallbackState};
    use crate::callbacks::CallbackRegistration;
    use crate::ffi::panel::FileSystemPath;
    use crate::MainThreadMarker;

    #[test]
    fn completion_copies_paths_once_and_allows_reentry() {
        let main_thread = unsafe { MainThreadMarker::new_unchecked() };
        let received = Rc::new(RefCell::new(Vec::new()));
        let reentered = Rc::new(Cell::new(false));
        let first = Rc::clone(&received);
        let second = Rc::clone(&received);
        let entered = Rc::clone(&reentered);
        let registration = CallbackRegistration::new(
            main_thread,
            PanelCallbackState::new(move |paths| {
                first.borrow_mut().push(paths);
                let nested = CallbackRegistration::new(
                    main_thread,
                    PanelCallbackState::new(move |paths| {
                        second.borrow_mut().push(paths);
                    }),
                );
                let nested_bytes = b"/private/second";
                let nested_path = FileSystemPath {
                    bytes: nested_bytes.as_ptr(),
                    length: nested_bytes.len(),
                };
                complete_on_main(main_thread, nested.token(), true, &raw const nested_path, 1);
                entered.set(true);
            }),
        );

        let mut original = b"/private/\xff".to_vec();
        let path = FileSystemPath {
            bytes: original.as_ptr(),
            length: original.len(),
        };
        let token = registration.token();
        complete_on_main(main_thread, token, true, &raw const path, 1);
        original.fill(b'x');
        complete_on_main(main_thread, token, true, &raw const path, 1);
        drop(registration);
        complete_on_main(main_thread, token, false, ptr::null(), 0);

        assert!(reentered.get());
        assert_eq!(
            &*received.borrow(),
            &[
                Some(vec![PathBuf::from(Path::new(std::ffi::OsStr::from_bytes(
                    b"/private/\xff"
                )))]),
                Some(vec![PathBuf::from("/private/second")]),
            ]
        );
    }

    #[test]
    fn cancellation_and_invalid_bytes_have_no_paths() {
        let main_thread = unsafe { MainThreadMarker::new_unchecked() };
        let received = Rc::new(RefCell::new(Vec::new()));
        let results = Rc::clone(&received);
        let registration = CallbackRegistration::new(
            main_thread,
            PanelCallbackState::new(move |paths| results.borrow_mut().push(paths)),
        );
        let token = registration.token();
        complete_on_main(main_thread, token, false, ptr::null(), 0);
        assert_eq!(&*received.borrow(), &[None]);

        let invalid = FileSystemPath {
            bytes: ptr::null(),
            length: 1,
        };
        let results = Rc::clone(&received);
        let registration = CallbackRegistration::new(
            main_thread,
            PanelCallbackState::new(move |paths| results.borrow_mut().push(paths)),
        );
        complete_on_main(
            main_thread,
            registration.token(),
            true,
            &raw const invalid,
            1,
        );
        assert_eq!(&*received.borrow(), &[None, None]);
    }
}
