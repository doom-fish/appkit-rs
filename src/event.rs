use core::ffi::c_void;
use core::marker::PhantomData;
use core::ptr::NonNull;

use crate::main_thread::MainThreadMarker;
use crate::object::Handle;

#[derive(Clone, Copy, Debug)]
pub struct EventRef<'event> {
    raw: NonNull<c_void>,
    main_thread: MainThreadMarker,
    borrowed: PhantomData<&'event ()>,
}

impl EventRef<'_> {
    #[must_use]
    pub const unsafe fn from_raw_borrowed(
        event: NonNull<c_void>,
        main_thread: MainThreadMarker,
    ) -> Self {
        Self {
            raw: event,
            main_thread,
            borrowed: PhantomData,
        }
    }

    #[must_use]
    pub const fn as_ptr(self) -> *mut c_void {
        self.raw.as_ptr()
    }

    #[must_use]
    pub fn retain(self) -> Event {
        Event {
            handle: unsafe { Handle::retain(self.raw) },
            main_thread: self.main_thread,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Event {
    handle: Handle,
    main_thread: MainThreadMarker,
}

impl Event {
    #[must_use]
    pub unsafe fn from_raw_retained(event: NonNull<c_void>, main_thread: MainThreadMarker) -> Self {
        Self {
            handle: unsafe { Handle::from_retained(event.as_ptr()) }
                .expect("the event pointer is not null"),
            main_thread,
        }
    }

    #[must_use]
    pub fn as_ref(&self) -> EventRef<'_> {
        EventRef {
            raw: NonNull::new(self.handle.as_ptr()).expect("an event handle is not null"),
            main_thread: self.main_thread,
            borrowed: PhantomData,
        }
    }

    #[must_use]
    pub const fn as_ptr(&self) -> *mut c_void {
        self.handle.as_ptr()
    }
}
