use crate::ffi::sound as ffi;
use crate::main_thread::MainThreadMarker;

pub fn beep(_main_thread: MainThreadMarker) {
    unsafe { ffi::appkit_sound_beep() };
}
