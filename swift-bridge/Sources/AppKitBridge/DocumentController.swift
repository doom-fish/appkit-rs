import AppKit
import AppKitExceptionShim

@_cdecl("appkit_document_controller_note_new_recent_document")
public func appkit_document_controller_note_new_recent_document(
    _ bytes: UnsafePointer<UInt8>?,
    _ length: Int
) -> Bool {
    guard Thread.isMainThread, let bytes, length > 0 else { return false }
    let buffer = UnsafeBufferPointer(start: bytes, count: length)
    guard !buffer.contains(0), let path = String(bytes: buffer, encoding: .utf8) else { return false }
    let url = URL(fileURLWithPath: path)
    return appkit_try { NSDocumentController.shared.noteNewRecentDocumentURL(url) } == nil
}
