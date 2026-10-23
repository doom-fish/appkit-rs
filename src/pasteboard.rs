use core::ffi::c_void;
use core::slice;
use std::ffi::OsString;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::PathBuf;

use crate::error::AppKitError;
use crate::ffi::pasteboard as ffi;
use crate::main_thread::MainThreadMarker;
use crate::object::Handle;

pub const TYPE_STRING: &str = "public.utf8-plain-text";
pub const TYPE_PNG: &str = "public.png";
pub const TYPE_TIFF: &str = "public.tiff";
pub const TYPE_LEGACY_FILENAMES: &str = "NSFilenamesPboardType";
pub const TYPE_FILE_URL: &str = "public.file-url";
pub const TYPE_JPEG: &str = "public.jpeg";
pub const TYPE_WEBP: &str = "org.webmproject.webp";
pub const TYPE_GIF: &str = "com.compuserve.gif";
pub const TYPE_BMP: &str = concat!("com.micro", "soft.bmp");
pub const TYPE_SVG: &str = "public.svg-image";
pub const TYPE_ICO: &str = concat!("com.micro", "soft.ico");
pub const TYPE_PNM: &str = "public.pbm";

struct OwnedBytes(ffi::AppKitPasteboardBytes);

impl OwnedBytes {
    const fn as_slice(&self) -> &[u8] {
        if self.0.length == 0 {
            &[]
        } else {
            unsafe { slice::from_raw_parts(self.0.bytes, self.0.length) }
        }
    }
}

impl Drop for OwnedBytes {
    fn drop(&mut self) {
        unsafe { ffi::appkit_pasteboard_bytes_free(self.0.bytes) };
    }
}

struct OwnedList(ffi::AppKitPasteboardBytesList);

impl OwnedList {
    const fn items(&self) -> &[ffi::AppKitPasteboardBytes] {
        if self.0.count == 0 {
            &[]
        } else {
            unsafe { slice::from_raw_parts(self.0.items, self.0.count) }
        }
    }

    fn bytes(&self) -> impl Iterator<Item = &[u8]> {
        self.items().iter().map(|item| {
            if item.length == 0 {
                &[][..]
            } else {
                unsafe { slice::from_raw_parts(item.bytes, item.length) }
            }
        })
    }
}

impl Drop for OwnedList {
    fn drop(&mut self) {
        unsafe { ffi::appkit_pasteboard_list_free(&raw mut self.0) };
    }
}

#[derive(Debug)]
pub struct Pasteboard {
    handle: Handle,
    unique: bool,
    _main_thread: MainThreadMarker,
}

impl Pasteboard {
    #[must_use]
    pub fn general(main_thread: MainThreadMarker) -> Self {
        let handle = unsafe { Handle::from_retained(ffi::appkit_pasteboard_general()) }
            .expect("AppKit gives the general pasteboard");
        Self {
            handle,
            unique: false,
            _main_thread: main_thread,
        }
    }

    #[must_use]
    pub fn find(main_thread: MainThreadMarker) -> Self {
        let handle = unsafe { Handle::from_retained(ffi::appkit_pasteboard_find()) }
            .expect("AppKit gives the find pasteboard");
        Self {
            handle,
            unique: false,
            _main_thread: main_thread,
        }
    }

    #[must_use]
    pub fn unique(main_thread: MainThreadMarker) -> Option<Self> {
        unsafe { Handle::from_retained(ffi::appkit_pasteboard_unique()) }.map(|handle| Self {
            handle,
            unique: true,
            _main_thread: main_thread,
        })
    }

    #[must_use]
    pub const fn as_ptr(&self) -> *mut c_void {
        self.handle.as_ptr()
    }

    #[must_use]
    pub fn types(&self) -> Vec<String> {
        let mut list = ffi::AppKitPasteboardBytesList {
            items: core::ptr::null_mut(),
            count: 0,
        };
        if !unsafe { ffi::appkit_pasteboard_types(self.as_ptr(), &raw mut list) } {
            return Vec::new();
        }
        let list = OwnedList(list);
        list.bytes()
            .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
            .collect()
    }

    #[must_use]
    pub fn data_for_type(&self, kind: &str) -> Option<Vec<u8>> {
        Self::read_bytes(|out| unsafe {
            ffi::appkit_pasteboard_data_for_type(self.as_ptr(), kind.as_ptr(), kind.len(), out)
        })
    }

    #[must_use]
    pub fn string_for_type(&self, kind: &str) -> Option<String> {
        Self::read_bytes(|out| unsafe {
            ffi::appkit_pasteboard_string_for_type(self.as_ptr(), kind.as_ptr(), kind.len(), out)
        })
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
    }

    #[must_use]
    pub fn legacy_filenames(&self) -> Vec<PathBuf> {
        let mut list = ffi::AppKitPasteboardBytesList {
            items: core::ptr::null_mut(),
            count: 0,
        };
        if !unsafe { ffi::appkit_pasteboard_filenames(self.as_ptr(), &raw mut list) } {
            return Vec::new();
        }
        let list = OwnedList(list);
        list.bytes()
            .map(|bytes| PathBuf::from(OsString::from_vec(bytes.to_vec())))
            .collect()
    }

    #[must_use]
    pub fn clear_contents(&self) -> i64 {
        unsafe { ffi::appkit_pasteboard_clear_contents(self.as_ptr()) }
    }

    #[must_use]
    pub fn declare_types(&self, types: &[&str]) -> i64 {
        let inputs: Vec<_> = types
            .iter()
            .map(|kind| ffi::AppKitPasteboardInput {
                bytes: kind.as_ptr(),
                length: kind.len(),
            })
            .collect();
        unsafe {
            ffi::appkit_pasteboard_declare_types(self.as_ptr(), inputs.as_ptr(), inputs.len())
        }
    }

    #[must_use]
    pub fn set_data_for_type(&self, kind: &str, data: &[u8]) -> bool {
        unsafe {
            ffi::appkit_pasteboard_set_data_for_type(
                self.as_ptr(),
                kind.as_ptr(),
                kind.len(),
                data.as_ptr(),
                data.len(),
            )
        }
    }

    #[must_use]
    pub fn set_string_for_type(&self, kind: &str, value: &str) -> bool {
        unsafe {
            ffi::appkit_pasteboard_set_string_for_type(
                self.as_ptr(),
                kind.as_ptr(),
                kind.len(),
                value.as_ptr(),
                value.len(),
            )
        }
    }

    pub fn set_filenames(&self, paths: &[PathBuf]) -> Result<bool, AppKitError> {
        let inputs: Vec<_> = paths
            .iter()
            .map(|path| {
                let bytes = path.as_os_str().as_bytes();
                if bytes.is_empty() {
                    return Err(AppKitError::InvalidArgument("a file path is empty".into()));
                }
                if bytes.contains(&0) {
                    return Err(AppKitError::InvalidArgument(
                        "a file path contains a null byte".into(),
                    ));
                }
                std::str::from_utf8(bytes).map_err(|_| {
                    AppKitError::InvalidArgument("a file path is not valid UTF-8".into())
                })?;
                Ok(ffi::AppKitPasteboardInput {
                    bytes: bytes.as_ptr(),
                    length: bytes.len(),
                })
            })
            .collect::<Result<_, _>>()?;
        Ok(unsafe {
            ffi::appkit_pasteboard_set_filenames(self.as_ptr(), inputs.as_ptr(), inputs.len())
        })
    }

    #[must_use]
    pub fn change_count(&self) -> i64 {
        unsafe { ffi::appkit_pasteboard_change_count(self.as_ptr()) }
    }

    fn read_bytes(read: impl FnOnce(*mut ffi::AppKitPasteboardBytes) -> bool) -> Option<Vec<u8>> {
        let mut bytes = ffi::AppKitPasteboardBytes {
            bytes: core::ptr::null_mut(),
            length: 0,
        };
        if !read(&raw mut bytes) {
            return None;
        }
        Some(OwnedBytes(bytes).as_slice().to_vec())
    }
}

impl Drop for Pasteboard {
    fn drop(&mut self) {
        if self.unique {
            unsafe { ffi::appkit_pasteboard_release_globally(self.as_ptr()) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        TYPE_BMP, TYPE_FILE_URL, TYPE_GIF, TYPE_ICO, TYPE_JPEG, TYPE_LEGACY_FILENAMES, TYPE_PNG,
        TYPE_PNM, TYPE_STRING, TYPE_SVG, TYPE_TIFF, TYPE_WEBP,
    };

    #[test]
    fn type_identifiers_have_distinct_image_values() {
        let images = [
            TYPE_PNG, TYPE_JPEG, TYPE_TIFF, TYPE_WEBP, TYPE_GIF, TYPE_BMP, TYPE_SVG, TYPE_ICO,
            TYPE_PNM,
        ];
        for (index, image) in images.iter().enumerate() {
            assert!(images[index + 1..].iter().all(|other| image != other));
        }
        assert_eq!(TYPE_STRING, "public.utf8-plain-text");
        assert_eq!(TYPE_FILE_URL, "public.file-url");
        assert_eq!(TYPE_LEGACY_FILENAMES, "NSFilenamesPboardType");
    }
}
