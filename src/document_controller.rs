use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use crate::error::AppKitError;
use crate::ffi::document_controller as ffi;
use crate::main_thread::MainThreadMarker;

pub struct DocumentController;

impl DocumentController {
    pub fn note_new_recent_document(
        _main_thread: MainThreadMarker,
        path: &Path,
    ) -> Result<(), AppKitError> {
        let bytes = path.as_os_str().as_bytes();
        if bytes.is_empty() || bytes.contains(&0) || std::str::from_utf8(bytes).is_err() {
            return Err(AppKitError::InvalidArgument(
                "a recent document path must be non-empty UTF-8 without null bytes".to_owned(),
            ));
        }
        if unsafe {
            ffi::appkit_document_controller_note_new_recent_document(bytes.as_ptr(), bytes.len())
        } {
            Ok(())
        } else {
            Err(AppKitError::InvalidArgument(
                "AppKit did not record the recent document".to_owned(),
            ))
        }
    }
}
