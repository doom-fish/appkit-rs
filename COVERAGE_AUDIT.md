# AppKit coverage audit

Date: 2026-11-01. SDK: macOS 27.0.

This audit compares the application and pasteboard additions with the SDK symbol names.
It does not claim complete AppKit coverage.
The other domains remain in `COVERAGE.md`.

| SDK symbol | Rust surface | Difference |
| --- | --- | --- |
| `NSPasteboard.writeObjects:` | `Pasteboard::write_file_urls` | File URL objects only. No general pasteboard writer wrapper. |
| `NSURL.initFileURLWithFileSystemRepresentation:isDirectory:relativeToURL:` | Internal file URL conversion | Preserves path bytes. No general URL constructor. |
| `NSApplicationDelegate.applicationShouldTerminate:` | `ApplicationHostHandlers::{should_terminate_reply, should_terminate}` | The reply handler takes priority. The boolean handler remains available. |
| `NSTerminateNow`, `NSTerminateCancel`, `NSTerminateLater` | `TerminateReply::{Now, Cancel, Later}` | All three native reply values. |
| `NSApplication.replyToApplicationShouldTerminate:` | `Application::reply_to_application_should_terminate` | The caller must reply once after `Later`. |
| `NSPasteboard.readObjectsForClasses:options:`, `canReadObjectForClasses:options:` | Not implemented | No general pasteboard object reader or file URL reader. |
| `NSApplication.stop:` | Not implemented | `Application::run` has no safe stop wrapper. |

The application callback table keeps version 1 and its existing layout.
Its termination callback already returns a 32-bit reply value.
The Swift bridge now accepts the native deferred reply value.

## Verification

Clippy for all targets and the documentation build passed.
All 36 existing tests and three documentation examples passed.
The `terminate_later` and `pasteboard_files` examples compiled. Only the owner runs them.
The README lists the owner's manual steps.
The minimum supported macOS version requires a separate check.
