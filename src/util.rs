use core::ffi::c_char;
use std::ffi::{CStr, CString};

use apple_cf::cg::{CGRect, CGSize};

use crate::error::AppKitError;
use crate::ffi;

pub fn c_string(value: &str) -> Result<CString, AppKitError> {
    CString::new(value).map_err(|error| AppKitError::InvalidArgument(error.to_string()))
}

pub fn optional_c_string(value: Option<&str>) -> Result<Option<CString>, AppKitError> {
    value.map(c_string).transpose()
}

pub unsafe fn take_string(pointer: *mut c_char) -> Option<String> {
    if pointer.is_null() {
        return None;
    }
    let value = unsafe { CStr::from_ptr(pointer) }.to_string_lossy().into_owned();
    unsafe { ffi::appkit_string_free(pointer) };
    Some(value)
}

pub fn read_rect(read: impl FnOnce(*mut CGRect)) -> CGRect {
    let mut rect = CGRect::new(0.0, 0.0, 0.0, 0.0);
    read(&raw mut rect);
    rect
}

pub fn read_size(read: impl FnOnce(*mut CGSize)) -> CGSize {
    let mut size = CGSize::new(0.0, 0.0);
    read(&raw mut size);
    size
}
