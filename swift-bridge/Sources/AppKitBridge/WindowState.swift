import AppKit
import AppKitCallbackABI
import AppKitExceptionShim

private final class WindowStateArchiverDelegate: NSObject, NSKeyedArchiverDelegate {
    func archiver(_ archiver: NSKeyedArchiver, willEncode object: Any) -> Any? {
        guard let secure = object as? NSSecureCoding,
              type(of: secure).supportsSecureCoding else {
            return nil
        }
        return object
    }
}

@_cdecl("appkit_window_state_encode")
public func appkit_window_state_encode(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<AppKitWindowStateBytes>
) -> Bool {
    guard Thread.isMainThread else { return false }
    let window: NSWindow = appkitObject(handle)
    let delegate = WindowStateArchiverDelegate()
    var archiver: NSKeyedArchiver?
    var data: Data?
    let exception = appkit_try {
        let coder = NSKeyedArchiver(requiringSecureCoding: true)
        archiver = coder
        coder.delegate = delegate
        window.encodeRestorableState(with: coder)
        coder.finishEncoding()
        if coder.error == nil {
            data = coder.encodedData
        }
    }
    let detached = appkit_try { archiver?.delegate = nil } == nil
    guard exception == nil, detached, let data, !data.isEmpty,
          let storage = malloc(data.count) else {
        return false
    }
    let copied = data.withUnsafeBytes { buffer -> Bool in
        guard let bytes = buffer.baseAddress else { return false }
        storage.copyMemory(from: bytes, byteCount: buffer.count)
        return true
    }
    guard copied else {
        free(storage)
        return false
    }
    out.pointee = AppKitWindowStateBytes(
        bytes: storage.assumingMemoryBound(to: UInt8.self),
        length: data.count
    )
    return true
}

@_cdecl("appkit_window_state_restore")
public func appkit_window_state_restore(
    _ handle: UnsafeMutableRawPointer,
    _ bytes: UnsafePointer<UInt8>?,
    _ length: Int,
    _ errorOut: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Bool {
    errorOut.pointee = nil
    guard Thread.isMainThread else {
        errorOut.pointee = strdup("window state requires the main thread")
        return false
    }
    guard length > 0, let bytes else {
        errorOut.pointee = strdup("the native window state is empty")
        return false
    }
    let window: NSWindow = appkitObject(handle)
    var failure: String?
    let exception = appkit_try {
        do {
            let data = Data(bytes: bytes, count: length)
            let coder = try NSKeyedUnarchiver(forReadingFrom: data)
            coder.requiresSecureCoding = true
            coder.decodingFailurePolicy = .setErrorAndReturn
            window.restoreState(with: coder)
            coder.finishDecoding()
            failure = coder.error?.localizedDescription
        } catch {
            failure = error.localizedDescription
        }
    }
    if let exception {
        errorOut.pointee = strdup(exception as String)
        return false
    }
    if let failure {
        errorOut.pointee = strdup(failure)
        return false
    }
    return true
}

@_cdecl("appkit_window_state_bytes_free")
public func appkit_window_state_bytes_free(_ bytes: UnsafeMutablePointer<UInt8>?) {
    free(bytes)
}
