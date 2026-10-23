use core::ptr;
use std::rc::Rc;

use doom_fish_utils::panic_safe::catch_user_panic_result;

use crate::callbacks::{CallbackTableHeader, HandlerSlot, RegistrationToken};
use crate::error::AppKitError;
use crate::ffi;
use crate::main_thread::MainThreadMarker;
use crate::object::Handle;
use crate::panel::{deactivate_completion, CompletionGuard};
use crate::util::{c_string, optional_c_string};
use crate::window::Window;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertStyle {
    Warning = 0,
    Informational = 1,
    Critical = 2,
}

#[must_use]
pub enum AlertSheetStart {
    ParentMayBeShown(AlertSession),
    AppKitDidNotStart,
}

struct AlertCallbackState {
    handler: HandlerSlot<Box<dyn FnMut(Option<usize>)>>,
}

impl AlertCallbackState {
    fn new(callback: impl FnOnce(Option<usize>) + 'static) -> Rc<Self> {
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
pub struct Alert {
    handle: Handle,
    main_thread: MainThreadMarker,
}

impl Alert {
    #[must_use]
    pub fn new(main_thread: MainThreadMarker) -> Self {
        let handle = unsafe { Handle::from_retained(ffi::alert::appkit_alert_new()) }
            .expect("AppKit creates an alert");
        Self {
            handle,
            main_thread,
        }
    }

    #[must_use]
    pub fn set_style(&self, style: AlertStyle) -> bool {
        unsafe { ffi::alert::appkit_alert_set_style(self.handle.as_ptr(), style as i32) }
    }

    pub fn set_message(&self, message: &str) -> Result<(), AppKitError> {
        let message = c_string(message)?;
        unsafe { ffi::alert::appkit_alert_set_message(self.handle.as_ptr(), message.as_ptr()) }
            .then_some(())
            .ok_or_else(|| AppKitError::InvalidArgument("alert message".into()))
    }

    pub fn set_informative_text(&self, text: Option<&str>) -> Result<(), AppKitError> {
        let text = optional_c_string(text)?;
        let success = unsafe {
            ffi::alert::appkit_alert_set_informative_text(
                self.handle.as_ptr(),
                text.as_ref().map_or(ptr::null(), |text| text.as_ptr()),
            )
        };
        success
            .then_some(())
            .ok_or_else(|| AppKitError::InvalidArgument("alert informative text".into()))
    }

    pub fn add_button(&self, title: &str, is_cancel: bool) -> Result<usize, AppKitError> {
        let title = c_string(title)?;
        let index = unsafe {
            ffi::alert::appkit_alert_add_button(self.handle.as_ptr(), title.as_ptr(), is_cancel)
        };
        usize::try_from(index).map_err(|_| AppKitError::InvalidArgument("alert button".into()))
    }

    pub fn begin_sheet(
        self,
        window: &Window,
        callback: impl FnOnce(Option<usize>) + 'static,
    ) -> AlertSheetStart {
        let guard = CompletionGuard::new(self.main_thread, AlertCallbackState::new(callback));
        let callbacks = ffi::alert::AlertCallbacksV1 {
            header: CallbackTableHeader {
                version: 1,
                size: size_of::<ffi::alert::AlertCallbacksV1>(),
            },
            deactivate: deactivate_completion,
            complete: alert_completed,
        };
        let token = guard.token();
        guard.retain();
        let started = unsafe {
            ffi::alert::appkit_alert_begin_sheet(
                self.handle.as_ptr(),
                window.as_ptr(),
                token.get(),
                &raw const callbacks,
            )
        };
        if !started {
            deactivate_completion(token.get());
            return AlertSheetStart::AppKitDidNotStart;
        }
        AlertSheetStart::ParentMayBeShown(AlertSession {
            handle: self.handle,
            token,
            main_thread: self.main_thread,
        })
    }
}

pub struct AlertSession {
    handle: Handle,
    token: RegistrationToken,
    main_thread: MainThreadMarker,
}

impl AlertSession {
    #[must_use]
    pub fn cancel(&self) -> bool {
        CompletionGuard::<AlertCallbackState>::is_active(self.token, self.main_thread)
            && unsafe { ffi::alert::appkit_alert_cancel(self.token.get()) }
    }

    pub fn detach(self) {
        drop(self.handle);
    }
}

extern "C" fn alert_completed(raw_token: u64, button_index: i64) {
    let _ = catch_user_panic_result("AppKit alert completion", || {
        let Some(main_thread) = MainThreadMarker::new() else {
            return;
        };
        let Some(token) = RegistrationToken::from_raw(raw_token) else {
            return;
        };
        complete_on_main(main_thread, token, button_index);
    });
}

fn complete_on_main(main_thread: MainThreadMarker, token: RegistrationToken, button_index: i64) {
    let Some(state) = CompletionGuard::<AlertCallbackState>::finish(token, main_thread) else {
        return;
    };
    let index = usize::try_from(button_index).ok();
    let _ = state
        .handler
        .invoke("AppKit alert handler", |callback| callback(index));
}

#[cfg(test)]
mod tests {
    use core::cell::{Cell, RefCell};
    use std::rc::Rc;

    use super::{complete_on_main, AlertCallbackState, CompletionGuard};
    use crate::callbacks::CallbackRegistration;
    use crate::MainThreadMarker;

    #[test]
    fn completion_returns_button_index_once_and_allows_reentry() {
        let main_thread = unsafe { MainThreadMarker::new_unchecked() };
        let received = Rc::new(RefCell::new(Vec::new()));
        let reentered = Rc::new(Cell::new(false));
        let first = Rc::clone(&received);
        let second = Rc::clone(&received);
        let entered = Rc::clone(&reentered);
        let registration = CallbackRegistration::new(
            main_thread,
            AlertCallbackState::new(move |index| {
                first.borrow_mut().push(index);
                let nested = CallbackRegistration::new(
                    main_thread,
                    AlertCallbackState::new(move |index| {
                        second.borrow_mut().push(index);
                    }),
                );
                complete_on_main(main_thread, nested.token(), 0);
                entered.set(true);
            }),
        );

        let token = registration.token();
        complete_on_main(main_thread, token, 2);
        complete_on_main(main_thread, token, 2);
        drop(registration);
        complete_on_main(main_thread, token, 2);
        assert!(reentered.get());
        assert_eq!(&*received.borrow(), &[Some(2), Some(0)]);
    }

    #[test]
    fn invalid_response_returns_none() {
        let main_thread = unsafe { MainThreadMarker::new_unchecked() };
        let received = Rc::new(Cell::new(Some(0)));
        let result = Rc::clone(&received);
        let registration = CallbackRegistration::new(
            main_thread,
            AlertCallbackState::new(move |index| result.set(index)),
        );
        complete_on_main(main_thread, registration.token(), -1);
        assert_eq!(received.get(), None);
    }

    #[test]
    fn parent_close_aborts_pending_sheet_once() {
        let main_thread = unsafe { MainThreadMarker::new_unchecked() };
        let received = Rc::new(RefCell::new(Vec::new()));
        let result = Rc::clone(&received);
        let state = AlertCallbackState::new(move |index| result.borrow_mut().push(index));
        let weak = Rc::downgrade(&state);
        let pending = CompletionGuard::new(main_thread, state);
        let token = pending.token();
        pending.retain();

        complete_on_main(main_thread, token, -1001);
        complete_on_main(main_thread, token, -1001);
        assert_eq!(&*received.borrow(), &[None]);
        assert!(weak.upgrade().is_none());
        assert!(!CompletionGuard::<AlertCallbackState>::is_active(
            token,
            main_thread
        ));
    }
}
