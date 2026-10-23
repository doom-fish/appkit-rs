use core::ffi::c_int;
use core::marker::PhantomData;

extern "C" {
    fn pthread_main_np() -> c_int;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MainThreadMarker {
    not_send: PhantomData<*mut ()>,
}

impl MainThreadMarker {
    #[must_use]
    pub fn new() -> Option<Self> {
        (unsafe { pthread_main_np() } != 0).then_some(Self { not_send: PhantomData })
    }

    #[must_use]
    pub const unsafe fn new_unchecked() -> Self {
        Self { not_send: PhantomData }
    }
}
