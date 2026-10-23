use core::any::Any;
use core::cell::RefCell;
use core::ffi::c_void;
use core::num::NonZeroU64;
use core::ops::Range;
use core::ptr::NonNull;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use doom_fish_utils::panic_safe::{catch_user_panic_result, catch_user_panic_result_with_cleanup};

use crate::event::EventRef;
use crate::ffi;
use crate::main_thread::MainThreadMarker;
use crate::object::Handle;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RegistrationToken(NonZeroU64);

impl RegistrationToken {
    #[must_use]
    pub const fn from_raw(token: u64) -> Option<Self> {
        match NonZeroU64::new(token) {
            Some(token) => Some(Self(token)),
            None => None,
        }
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Default)]
struct Registry {
    next: u64,
    states: HashMap<RegistrationToken, Weak<dyn Any>>,
}

thread_local! {
    static REGISTRY: RefCell<Registry> = RefCell::new(Registry::default());
}

pub struct CallbackRegistration<T: 'static> {
    token: RegistrationToken,
    state: Rc<T>,
    _main_thread: MainThreadMarker,
}

impl<T: 'static> CallbackRegistration<T> {
    #[must_use]
    pub fn new(main_thread: MainThreadMarker, state: Rc<T>) -> Self {
        let erased: Rc<dyn Any> = state.clone();
        let weak = Rc::downgrade(&erased);
        let token = REGISTRY.with(|registry| {
            let mut registry = registry.borrow_mut();
            registry.next = registry
                .next
                .checked_add(1)
                .expect("AppKit registration tokens are exhausted");
            let token =
                RegistrationToken::from_raw(registry.next).expect("a new token is not zero");
            registry.states.insert(token, weak);
            token
        });
        Self {
            token,
            state,
            _main_thread: main_thread,
        }
    }

    #[must_use]
    pub const fn token(&self) -> RegistrationToken {
        self.token
    }

    #[must_use]
    pub fn state(&self) -> &T {
        &self.state
    }

    #[must_use]
    pub fn snapshot(token: RegistrationToken, _main_thread: MainThreadMarker) -> Option<Rc<T>> {
        let weak = REGISTRY
            .try_with(|registry| registry.borrow().states.get(&token).cloned())
            .ok()
            .flatten()?;
        weak.upgrade()?.downcast::<T>().ok()
    }

    #[must_use]
    pub fn deactivate(&self) -> bool {
        REGISTRY
            .try_with(|registry| registry.borrow_mut().states.remove(&self.token).is_some())
            .unwrap_or(false)
    }
}

impl<T: 'static> Drop for CallbackRegistration<T> {
    fn drop(&mut self) {
        let _ = self.deactivate();
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CallbackTableHeader {
    pub version: u32,
    pub size: usize,
}

pub type EventCallback = extern "C" fn(u64, *mut c_void) -> bool;
pub type TextCallback = extern "C" fn(u64, *const u8, usize);

pub trait CallbackState: 'static {
    fn handle_event(&self, _event: EventRef<'_>) -> bool {
        false
    }

    fn handle_text(&self, _text: String) {}
}

impl CallbackState for () {}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CallbackTableV1 {
    pub header: CallbackTableHeader,
    pub deactivate: extern "C" fn(u64),
    pub event: Option<EventCallback>,
    pub text: Option<TextCallback>,
}

impl CallbackTableV1 {
    pub const VERSION: u32 = 1;

    #[must_use]
    pub const fn for_state<T: CallbackState>() -> Self {
        Self::new(Some(event_trampoline::<T>), Some(text_trampoline::<T>))
    }

    pub(crate) const fn new(event: Option<EventCallback>, text: Option<TextCallback>) -> Self {
        Self {
            header: CallbackTableHeader {
                version: Self::VERSION,
                size: size_of::<Self>(),
            },
            deactivate: deactivate_from_swift,
            event,
            text,
        }
    }
}

extern "C" fn event_trampoline<T: CallbackState>(raw_token: u64, event: *mut c_void) -> bool {
    unsafe {
        dispatch_event::<T>(raw_token, event, "AppKit event callback", |state, event| {
            state.handle_event(event)
        })
    }
}

extern "C" fn text_trampoline<T: CallbackState>(raw_token: u64, bytes: *const u8, length: usize) {
    let _ = unsafe {
        dispatch_text::<T>(
            raw_token,
            bytes,
            length,
            "AppKit text callback",
            |state, text| {
                state.handle_text(text);
            },
        )
    };
}

extern "C" fn deactivate_from_swift(raw_token: u64) {
    let _ = catch_user_panic_result("AppKit callback deactivation", || {
        if MainThreadMarker::new().is_none() {
            return;
        }
        if let Some(token) = RegistrationToken::from_raw(raw_token) {
            let _ = REGISTRY.try_with(|registry| registry.borrow_mut().states.remove(&token));
        }
    });
}

pub struct NativeCallbackRegistration<T: 'static> {
    registration: CallbackRegistration<T>,
    handle: Handle,
}

impl<T: CallbackState> NativeCallbackRegistration<T> {
    #[must_use]
    pub fn new(main_thread: MainThreadMarker, state: Rc<T>) -> Option<Self> {
        Self::with_table(main_thread, state, CallbackTableV1::for_state::<T>())
    }
}

impl<T: 'static> NativeCallbackRegistration<T> {
    #[must_use]
    pub(crate) fn with_table(
        main_thread: MainThreadMarker,
        state: Rc<T>,
        table: CallbackTableV1,
    ) -> Option<Self> {
        let registration = CallbackRegistration::new(main_thread, state);
        let raw = unsafe {
            ffi::callbacks::appkit_callbacks_create_v1(registration.token.get(), &raw const table)
        };
        let handle = unsafe { Handle::from_retained(raw) }?;
        Some(Self {
            registration,
            handle,
        })
    }

    #[must_use]
    pub const fn registration(&self) -> &CallbackRegistration<T> {
        &self.registration
    }

    #[must_use]
    pub const fn as_ptr(&self) -> *mut c_void {
        self.handle.as_ptr()
    }
}

impl<T: 'static> Drop for NativeCallbackRegistration<T> {
    fn drop(&mut self) {
        let _ = self.registration.deactivate();
    }
}

struct SlotState<H> {
    handler: Option<H>,
    invoking: bool,
    replaced: bool,
}

pub struct HandlerSlot<H> {
    inner: RefCell<SlotState<H>>,
}

impl<H> HandlerSlot<H> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inner: RefCell::new(SlotState {
                handler: None,
                invoking: false,
                replaced: false,
            }),
        }
    }

    pub fn replace(&self, handler: Option<H>) -> Option<H> {
        let mut inner = self.inner.borrow_mut();
        if inner.invoking {
            inner.replaced = true;
        }
        core::mem::replace(&mut inner.handler, handler)
    }

    pub fn invoke<R>(&self, site: &str, call: impl FnOnce(&mut H) -> R) -> Option<R> {
        let handler = {
            let mut inner = self.inner.borrow_mut();
            if inner.invoking {
                return None;
            }
            let handler = inner.handler.take()?;
            inner.invoking = true;
            inner.replaced = false;
            handler
        };

        let mut call = Some(call);
        catch_user_panic_result_with_cleanup(
            site,
            Some(handler),
            |handler| {
                call.take().expect("a handler is invoked once")(
                    handler.as_mut().expect("an active slot has a handler"),
                )
            },
            |handler| {
                let mut inner = self.inner.borrow_mut();
                if !inner.replaced && inner.handler.is_none() {
                    inner.handler = handler.take();
                }
                inner.invoking = false;
                inner.replaced = false;
            },
        )
    }
}

impl<H> Default for HandlerSlot<H> {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Utf16Range {
    pub location: usize,
    pub length: usize,
}

impl Utf16Range {
    pub const INVALID: Self = Self {
        location: isize::MAX as usize,
        length: 0,
    };

    #[must_use]
    pub const fn new(location: usize, length: usize) -> Option<Self> {
        if location >= Self::INVALID.location || length > Self::INVALID.location - location {
            None
        } else {
            Some(Self { location, length })
        }
    }

    #[must_use]
    pub const fn is_valid(self) -> bool {
        Self::new(self.location, self.length).is_some()
    }

    #[must_use]
    pub fn from_range(range: Range<usize>) -> Option<Self> {
        Self::new(range.start, range.end.checked_sub(range.start)?)
    }

    #[must_use]
    pub fn to_range(self) -> Option<Range<usize>> {
        self.is_valid()
            .then(|| self.location..self.location + self.length)
    }

    #[must_use]
    pub fn from_utf8_range(text: &str, range: Range<usize>) -> Option<Self> {
        let prefix = text.get(..range.start)?;
        let selected = text.get(range)?;
        Self::new(
            prefix.encode_utf16().count(),
            selected.encode_utf16().count(),
        )
    }

    #[must_use]
    pub fn to_utf8_range(self, text: &str) -> Option<Range<usize>> {
        let range = self.to_range()?;
        Some(utf8_offset(text, range.start)?..utf8_offset(text, range.end)?)
    }
}

fn utf8_offset(text: &str, utf16_offset: usize) -> Option<usize> {
    let mut position = 0_usize;
    for (byte, character) in text.char_indices() {
        if position == utf16_offset {
            return Some(byte);
        }
        position += character.len_utf16();
        if position > utf16_offset {
            return None;
        }
    }
    (position == utf16_offset).then_some(text.len())
}

#[must_use]
pub unsafe fn copy_utf8(bytes: *const u8, length: usize) -> Option<String> {
    if length == 0 {
        return Some(String::new());
    }
    if length > isize::MAX as usize {
        return None;
    }
    let bytes = NonNull::new(bytes.cast_mut())?;
    let slice = unsafe { core::slice::from_raw_parts(bytes.as_ptr(), length) };
    core::str::from_utf8(slice).ok().map(str::to_owned)
}

#[must_use]
pub unsafe fn dispatch_event<T: 'static>(
    raw_token: u64,
    event: *mut c_void,
    site: &str,
    callback: impl FnOnce(&T, EventRef<'_>) -> bool,
) -> bool {
    let Some(main_thread) = MainThreadMarker::new() else {
        return false;
    };
    unsafe { dispatch_event_on_main(main_thread, raw_token, event, site, callback) }
}

unsafe fn dispatch_event_on_main<T: 'static>(
    main_thread: MainThreadMarker,
    raw_token: u64,
    event: *mut c_void,
    site: &str,
    callback: impl FnOnce(&T, EventRef<'_>) -> bool,
) -> bool {
    catch_user_panic_result(site, || {
        let state = CallbackRegistration::<T>::snapshot(
            RegistrationToken::from_raw(raw_token)?,
            main_thread,
        )?;
        let event = unsafe { EventRef::from_raw_borrowed(NonNull::new(event)?, main_thread) };
        Some(callback(&state, event))
    })
    .flatten()
    .unwrap_or(false)
}

#[must_use]
pub unsafe fn dispatch_text<T: 'static>(
    raw_token: u64,
    bytes: *const u8,
    length: usize,
    site: &str,
    callback: impl FnOnce(&T, String),
) -> bool {
    let Some(main_thread) = MainThreadMarker::new() else {
        return false;
    };
    unsafe { dispatch_text_on_main(main_thread, raw_token, bytes, length, site, callback) }
}

unsafe fn dispatch_text_on_main<T: 'static>(
    main_thread: MainThreadMarker,
    raw_token: u64,
    bytes: *const u8,
    length: usize,
    site: &str,
    callback: impl FnOnce(&T, String),
) -> bool {
    catch_user_panic_result(site, || {
        let state = CallbackRegistration::<T>::snapshot(
            RegistrationToken::from_raw(raw_token)?,
            main_thread,
        )?;
        let text = unsafe { copy_utf8(bytes, length)? };
        callback(&state, text);
        Some(())
    })
    .flatten()
    .is_some()
}

#[cfg(test)]
mod tests {
    use core::ptr;
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use super::{
        copy_utf8, dispatch_event_on_main, dispatch_text, dispatch_text_on_main,
        CallbackRegistration, CallbackTableV1, HandlerSlot, RegistrationToken, Utf16Range,
    };
    use crate::MainThreadMarker;

    #[derive(Default)]
    struct TestState {
        text: HandlerSlot<Box<dyn FnMut(String)>>,
    }

    fn deliver_text(token: u64, bytes: &[u8]) -> bool {
        unsafe {
            dispatch_text_on_main::<TestState>(
                MainThreadMarker::new_unchecked(),
                token,
                bytes.as_ptr(),
                bytes.len(),
                "test text delivery",
                |state, text| {
                    let _ = state
                        .text
                        .invoke("test text handler", |handler| handler(text));
                },
            )
        }
    }

    #[test]
    fn registration_uses_unique_tokens_and_weak_snapshots() {
        let main_thread = unsafe { MainThreadMarker::new_unchecked() };
        let state = Rc::new(TestState::default());
        let weak = Rc::downgrade(&state);
        let registration = CallbackRegistration::new(main_thread, Rc::clone(&state));
        let token = registration.token();
        assert_ne!(token.get(), 0);
        assert_eq!(RegistrationToken::from_raw(token.get()), Some(token));
        assert_eq!(RegistrationToken::from_raw(0), None);

        let snapshot = CallbackRegistration::<TestState>::snapshot(token, main_thread).unwrap();
        assert!(Rc::ptr_eq(&state, &snapshot));
        assert!(registration.deactivate());
        assert!(!registration.deactivate());
        assert!(CallbackRegistration::<TestState>::snapshot(token, main_thread).is_none());

        drop(registration);
        drop(state);
        assert!(weak.upgrade().is_some());
        drop(snapshot);
        assert!(weak.upgrade().is_none());

        let next = CallbackRegistration::new(main_thread, Rc::new(TestState::default()));
        assert_ne!(next.token(), token);
    }

    #[test]
    fn a_late_callback_does_not_read_deactivated_state() {
        let main_thread = unsafe { MainThreadMarker::new_unchecked() };
        let count = Rc::new(Cell::new(0));
        let registration = CallbackRegistration::new(main_thread, Rc::new(TestState::default()));
        let callback_count = Rc::clone(&count);
        registration.state().text.replace(Some(Box::new(move |_| {
            callback_count.set(callback_count.get() + 1);
        })));

        let token = registration.token().get();
        assert!(deliver_text(token, b"first"));
        assert!(deliver_text(token, b"second"));
        assert_eq!(count.get(), 2);
        assert!(registration.deactivate());
        assert!(!deliver_text(token, b"late"));
        assert_eq!(count.get(), 2);
    }

    #[test]
    fn reentrant_replacement_preserves_the_new_handler() {
        let main_thread = unsafe { MainThreadMarker::new_unchecked() };
        let state = Rc::new(TestState::default());
        let weak = Rc::downgrade(&state);
        let registration = CallbackRegistration::new(main_thread, state);
        let received = Rc::new(RefCell::new(Vec::new()));
        let first = Rc::clone(&received);
        let nested_registered = Rc::new(Cell::new(false));
        let registered = Rc::clone(&nested_registered);

        registration
            .state()
            .text
            .replace(Some(Box::new(move |text| {
                first.borrow_mut().push(text);
                let nested = CallbackRegistration::new(main_thread, Rc::new(TestState::default()));
                registered.set(
                    CallbackRegistration::<TestState>::snapshot(nested.token(), main_thread)
                        .is_some(),
                );
                let state = weak.upgrade().unwrap();
                let second = Rc::clone(&first);
                state.text.replace(Some(Box::new(move |text| {
                    second.borrow_mut().push(text);
                })));
                assert!(state
                    .text
                    .invoke("recursive query", |handler| handler("recursive".into()))
                    .is_none());
            })));

        let token = registration.token().get();
        assert!(deliver_text(token, b"first"));
        assert!(deliver_text(token, b"second"));
        assert!(nested_registered.get());
        assert_eq!(&*received.borrow(), &["first", "second"]);
    }

    #[test]
    fn clearing_during_delivery_does_not_restore_the_old_handler() {
        let main_thread = unsafe { MainThreadMarker::new_unchecked() };
        let state = Rc::new(TestState::default());
        let weak = Rc::downgrade(&state);
        let registration = CallbackRegistration::new(main_thread, state);
        let calls = Rc::new(Cell::new(0));
        let called = Rc::clone(&calls);
        registration.state().text.replace(Some(Box::new(move |_| {
            called.set(called.get() + 1);
            weak.upgrade().unwrap().text.replace(None);
        })));

        let token = registration.token().get();
        assert!(deliver_text(token, b"first"));
        assert!(deliver_text(token, b"second"));
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn a_panicking_handler_does_not_unwind_across_the_callback_boundary() {
        let main_thread = unsafe { MainThreadMarker::new_unchecked() };
        let registration = CallbackRegistration::new(main_thread, Rc::new(TestState::default()));
        let attempts = Rc::new(Cell::new(0));
        let attempted = Rc::clone(&attempts);
        registration.state().text.replace(Some(Box::new(move |_| {
            attempted.set(attempted.get() + 1);
            assert_ne!(attempted.get(), 1, "handler panic");
        })));

        let token = registration.token().get();
        assert!(deliver_text(token, b"first"));
        assert!(deliver_text(token, b"second"));
        assert_eq!(attempts.get(), 2);

        let failed = unsafe {
            dispatch_text_on_main::<TestState>(
                main_thread,
                token,
                b"text".as_ptr(),
                4,
                "panicking trampoline",
                |_, _| panic!("trampoline panic"),
            )
        };
        assert!(!failed);

        let calls = Rc::new(Cell::new(0));
        let called = Rc::clone(&calls);
        registration.state().text.replace(Some(Box::new(move |_| {
            called.set(called.get() + 1);
        })));
        assert!(deliver_text(token, b"second"));
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn utf16_ranges_preserve_units_and_invalidity() {
        let range = Utf16Range::from_range(1..3).unwrap();
        assert_eq!(range.to_range(), Some(1..3));
        assert_eq!(Utf16Range::INVALID.location, isize::MAX as usize);
        assert_eq!(Utf16Range::INVALID.to_range(), None);
        assert!(!Utf16Range::INVALID.is_valid());
        assert_eq!(
            Utf16Range {
                location: usize::MAX,
                length: 1,
            }
            .to_range(),
            None
        );
        let reversed_start = 4;
        let reversed_end = 2;
        assert!(Utf16Range::from_range(reversed_start..reversed_end).is_none());
        assert!(Utf16Range::new(isize::MAX as usize - 1, 2).is_none());

        let text = "a🙂é";
        assert_eq!(Utf16Range::from_utf8_range(text, 1..5), Some(range));
        assert_eq!(range.to_utf8_range(text), Some(1..5));
        assert_eq!(Utf16Range::new(2, 1).unwrap().to_utf8_range(text), None);
        assert_eq!(
            Utf16Range::new(4, 0).unwrap().to_utf8_range(text),
            Some(7..7)
        );
        assert!(Utf16Range::from_utf8_range(text, 2..5).is_none());
        assert_eq!(Utf16Range::INVALID.to_utf8_range(text), None);
    }

    #[test]
    fn text_bytes_are_copied_before_the_buffer_changes() {
        let main_thread = unsafe { MainThreadMarker::new_unchecked() };
        let registration = CallbackRegistration::new(main_thread, Rc::new(TestState::default()));
        let received = Rc::new(RefCell::new(Vec::new()));
        let saved = Rc::clone(&received);
        registration
            .state()
            .text
            .replace(Some(Box::new(move |text| saved.borrow_mut().push(text))));
        let mut bytes = "a\0🙂".as_bytes().to_vec();
        assert!(deliver_text(registration.token().get(), &bytes));
        let copied = unsafe { copy_utf8(bytes.as_ptr(), bytes.len()) }.unwrap();
        bytes.fill(b'x');
        assert_eq!(copied, "a\0🙂");
        assert_eq!(&*received.borrow(), &["a\0🙂"]);
        assert_eq!(unsafe { copy_utf8(ptr::null(), 0) }, Some(String::new()));
        assert_eq!(unsafe { copy_utf8(ptr::null(), 1) }, None);
        assert_eq!(unsafe { copy_utf8([0xff].as_ptr(), 1) }, None);
        assert_eq!(unsafe { copy_utf8(bytes.as_ptr(), usize::MAX) }, None);
    }

    #[test]
    fn callback_tables_fail_closed_for_invalid_payloads_and_threads() {
        let main_thread = unsafe { MainThreadMarker::new_unchecked() };
        let registration = CallbackRegistration::new(main_thread, Rc::new(TestState::default()));
        let table = CallbackTableV1::new(None, None);
        assert_eq!(table.header.version, CallbackTableV1::VERSION);
        assert_eq!(table.header.size, size_of::<CallbackTableV1>());

        let token = registration.token().get();
        assert!(!unsafe {
            dispatch_event_on_main::<TestState>(
                main_thread,
                token,
                ptr::null_mut(),
                "null event",
                |_, _| true,
            )
        });
        assert!(!unsafe {
            dispatch_text::<TestState>(token, b"text".as_ptr(), 4, "wrong thread", |_, _| {
                panic!("a non-main callback reached its handler")
            })
        });
        assert!(!unsafe {
            dispatch_text_on_main::<TestState>(
                main_thread,
                token,
                ptr::null(),
                1,
                "null text",
                |_, _| panic!("invalid text reached its handler"),
            )
        });
    }
}
