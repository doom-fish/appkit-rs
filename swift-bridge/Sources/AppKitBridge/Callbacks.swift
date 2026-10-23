import AppKit
import AppKitCallbackABI

final class AppKitCallbackOwner {
    let token: UInt64
    private let table: AppKitCallbackTableV1

    init?(token: UInt64, table: AppKitCallbackTableV1) {
        guard Thread.isMainThread,
              token != 0,
              table.header.version == 1,
              table.header.size >= MemoryLayout<AppKitCallbackTableV1>.size,
              table.deactivate != nil else {
            return nil
        }
        self.token = token
        self.table = table
    }

    deinit {
        table.deactivate(token)
    }

    func event(_ event: NSEvent) -> Bool {
        guard Thread.isMainThread else { return false }
        return table.event?(token, Unmanaged.passUnretained(event).toOpaque()) ?? false
    }

    func text(_ text: String) {
        guard Thread.isMainThread, let callback = table.text else { return }
        let bytes = Array(text.utf8)
        bytes.withUnsafeBufferPointer { buffer in
            callback(token, buffer.baseAddress, buffer.count)
        }
    }
}

extension AppKitUtf16Range {
    init(_ range: NSRange) {
        guard range.location >= 0,
              range.location != NSNotFound,
              range.length >= 0,
              range.length <= Int.max - range.location else {
            self.init(location: NSNotFound, length: 0)
            return
        }
        self.init(location: range.location, length: range.length)
    }

    var nsRange: NSRange {
        guard location >= 0,
              location != NSNotFound,
              length >= 0,
              length <= Int.max - location else {
            return NSRange(location: NSNotFound, length: 0)
        }
        return NSRange(location: location, length: length)
    }
}

@_cdecl("appkit_callbacks_create_v1")
public func appkit_callbacks_create_v1(
    _ token: UInt64,
    _ table: UnsafePointer<AppKitCallbackTableV1>?
) -> UnsafeMutableRawPointer? {
    guard let table, let owner = AppKitCallbackOwner(token: token, table: table.pointee) else {
        return nil
    }
    return Unmanaged.passRetained(owner).toOpaque()
}
