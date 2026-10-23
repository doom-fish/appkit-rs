import AppKit
import AppKitCallbackABI
import AppKitExceptionShim

private func appkitPasteboardInputString(_ bytes: UnsafePointer<UInt8>?, _ length: Int) -> String? {
    guard length >= 0, length == 0 || bytes != nil else { return nil }
    return String(bytes: UnsafeBufferPointer(start: bytes, count: length), encoding: .utf8)
}

private func appkitPasteboardType(_ bytes: UnsafePointer<UInt8>?, _ length: Int) -> NSPasteboard.PasteboardType? {
    guard let name = appkitPasteboardInputString(bytes, length),
          !name.isEmpty, !name.utf8.contains(0) else { return nil }
    return NSPasteboard.PasteboardType(name)
}

private func appkitPasteboardOwnedBytes(_ data: Data) -> AppKitPasteboardBytes? {
    guard let storage = malloc(max(1, data.count)) else { return nil }
    data.withUnsafeBytes { buffer in
        if let base = buffer.baseAddress {
            storage.copyMemory(from: base, byteCount: buffer.count)
        }
    }
    return AppKitPasteboardBytes(bytes: storage.assumingMemoryBound(to: UInt8.self), length: data.count)
}

private func appkitPasteboardOwnedList(_ values: [Data]) -> AppKitPasteboardBytesList? {
    guard let items = calloc(max(1, values.count), MemoryLayout<AppKitPasteboardBytes>.stride)?
        .assumingMemoryBound(to: AppKitPasteboardBytes.self) else {
        return nil
    }
    for (index, value) in values.enumerated() {
        guard let bytes = appkitPasteboardOwnedBytes(value) else {
            for previous in 0..<index {
                free(items[previous].bytes)
            }
            free(items)
            return nil
        }
        items[index] = bytes
    }
    return AppKitPasteboardBytesList(items: items, count: values.count)
}

@_cdecl("appkit_pasteboard_general")
public func appkit_pasteboard_general() -> UnsafeMutableRawPointer? {
    var pasteboard: NSPasteboard?
    guard appkit_try({ pasteboard = NSPasteboard.general }) == nil else { return nil }
    return appkitRetained(pasteboard)
}

@_cdecl("appkit_pasteboard_find")
public func appkit_pasteboard_find() -> UnsafeMutableRawPointer? {
    var pasteboard: NSPasteboard?
    guard appkit_try({ pasteboard = NSPasteboard(name: .find) }) == nil else { return nil }
    return appkitRetained(pasteboard)
}

@_cdecl("appkit_pasteboard_unique")
public func appkit_pasteboard_unique() -> UnsafeMutableRawPointer? {
    var pasteboard: NSPasteboard?
    guard appkit_try({
        pasteboard = NSPasteboard(name: .init("appkit-rs.unique.\(UUID().uuidString)"))
    }) == nil else { return nil }
    return appkitRetained(pasteboard)
}

@_cdecl("appkit_pasteboard_release_globally")
public func appkit_pasteboard_release_globally(_ handle: UnsafeMutableRawPointer) {
    let pasteboard: NSPasteboard = appkitObject(handle)
    _ = appkit_try { pasteboard.releaseGlobally() }
}

@_cdecl("appkit_pasteboard_types")
public func appkit_pasteboard_types(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<AppKitPasteboardBytesList>
) -> Bool {
    let pasteboard: NSPasteboard = appkitObject(handle)
    var types: [Data] = []
    guard appkit_try({
        types = pasteboard.types?.map { Data($0.rawValue.utf8) } ?? []
    }) == nil else { return false }
    guard let list = appkitPasteboardOwnedList(types) else { return false }
    out.pointee = list
    return true
}

@_cdecl("appkit_pasteboard_data_for_type")
public func appkit_pasteboard_data_for_type(
    _ handle: UnsafeMutableRawPointer,
    _ kind: UnsafePointer<UInt8>?,
    _ kindLength: Int,
    _ out: UnsafeMutablePointer<AppKitPasteboardBytes>
) -> Bool {
    let pasteboard: NSPasteboard = appkitObject(handle)
    var data: Data?
    guard let type = appkitPasteboardType(kind, kindLength),
          appkit_try({ data = pasteboard.data(forType: type) }) == nil,
          let data,
          let bytes = appkitPasteboardOwnedBytes(data) else {
        return false
    }
    out.pointee = bytes
    return true
}

@_cdecl("appkit_pasteboard_string_for_type")
public func appkit_pasteboard_string_for_type(
    _ handle: UnsafeMutableRawPointer,
    _ kind: UnsafePointer<UInt8>?,
    _ kindLength: Int,
    _ out: UnsafeMutablePointer<AppKitPasteboardBytes>
) -> Bool {
    let pasteboard: NSPasteboard = appkitObject(handle)
    var text: String?
    guard let type = appkitPasteboardType(kind, kindLength),
          appkit_try({ text = pasteboard.string(forType: type) }) == nil,
          let text,
          let bytes = appkitPasteboardOwnedBytes(Data(text.utf8)) else {
        return false
    }
    out.pointee = bytes
    return true
}

@_cdecl("appkit_pasteboard_filenames")
public func appkit_pasteboard_filenames(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<AppKitPasteboardBytesList>
) -> Bool {
    let pasteboard: NSPasteboard = appkitObject(handle)
    var filenames: [String] = []
    guard appkit_try({
        filenames = pasteboard.propertyList(
            forType: NSPasteboard.PasteboardType("NSFilenamesPboardType")
        ) as? [String] ?? []
    }) == nil else { return false }
    let paths = filenames.map { Data($0.utf8) }
    guard let list = appkitPasteboardOwnedList(paths) else { return false }
    out.pointee = list
    return true
}

@_cdecl("appkit_pasteboard_clear_contents")
public func appkit_pasteboard_clear_contents(_ handle: UnsafeMutableRawPointer) -> Int64 {
    let pasteboard: NSPasteboard = appkitObject(handle)
    var count: Int64 = -1
    guard appkit_try({ count = Int64(pasteboard.clearContents()) }) == nil else { return -1 }
    return count
}

@_cdecl("appkit_pasteboard_declare_types")
public func appkit_pasteboard_declare_types(
    _ handle: UnsafeMutableRawPointer,
    _ kinds: UnsafePointer<AppKitPasteboardInput>,
    _ count: Int
) -> Int64 {
    let pasteboard: NSPasteboard = appkitObject(handle)
    guard count >= 0 else { return -1 }
    var types: [NSPasteboard.PasteboardType] = []
    types.reserveCapacity(count)
    for index in 0..<count {
        guard let type = appkitPasteboardType(kinds[index].bytes, kinds[index].length) else {
            return -1
        }
        types.append(type)
    }
    var count: Int64 = -1
    guard appkit_try({ count = Int64(pasteboard.declareTypes(types, owner: nil)) }) == nil else {
        return -1
    }
    return count
}

@_cdecl("appkit_pasteboard_set_data_for_type")
public func appkit_pasteboard_set_data_for_type(
    _ handle: UnsafeMutableRawPointer,
    _ kind: UnsafePointer<UInt8>?,
    _ kindLength: Int,
    _ data: UnsafePointer<UInt8>?,
    _ dataLength: Int
) -> Bool {
    let pasteboard: NSPasteboard = appkitObject(handle)
    guard let type = appkitPasteboardType(kind, kindLength), dataLength >= 0,
          dataLength == 0 || data != nil else {
        return false
    }
    var saved = false
    return appkit_try({
        let bytes = dataLength == 0 ? Data() : Data(bytes: data!, count: dataLength)
        saved = pasteboard.setData(bytes, forType: type)
    }) == nil && saved
}

@_cdecl("appkit_pasteboard_set_string_for_type")
public func appkit_pasteboard_set_string_for_type(
    _ handle: UnsafeMutableRawPointer,
    _ kind: UnsafePointer<UInt8>?,
    _ kindLength: Int,
    _ value: UnsafePointer<UInt8>?,
    _ valueLength: Int
) -> Bool {
    let pasteboard: NSPasteboard = appkitObject(handle)
    guard let type = appkitPasteboardType(kind, kindLength),
          let text = appkitPasteboardInputString(value, valueLength) else {
        return false
    }
    var saved = false
    return appkit_try({ saved = pasteboard.setString(text, forType: type) }) == nil && saved
}

@_cdecl("appkit_pasteboard_set_filenames")
public func appkit_pasteboard_set_filenames(
    _ handle: UnsafeMutableRawPointer,
    _ paths: UnsafePointer<AppKitPasteboardInput>,
    _ count: Int
) -> Bool {
    let pasteboard: NSPasteboard = appkitObject(handle)
    guard count >= 0 else { return false }
    var filenames: [String] = []
    filenames.reserveCapacity(count)
    for index in 0..<count {
        let path = paths[index]
        guard path.length > 0, let bytes = path.bytes,
              !UnsafeBufferPointer(start: bytes, count: path.length).contains(0),
              let filename = String(
                  bytes: UnsafeBufferPointer(start: bytes, count: path.length),
                  encoding: .utf8
              ),
              !filename.isEmpty else {
            return false
        }
        filenames.append(filename)
    }
    var saved = false
    return appkit_try({
        saved = pasteboard.setPropertyList(
            filenames,
            forType: NSPasteboard.PasteboardType("NSFilenamesPboardType")
        )
    }) == nil && saved
}

@_cdecl("appkit_pasteboard_change_count")
public func appkit_pasteboard_change_count(_ handle: UnsafeMutableRawPointer) -> Int64 {
    let pasteboard: NSPasteboard = appkitObject(handle)
    var count: Int64 = -1
    guard appkit_try({ count = Int64(pasteboard.changeCount) }) == nil else { return -1 }
    return count
}

@_cdecl("appkit_pasteboard_bytes_free")
public func appkit_pasteboard_bytes_free(_ bytes: UnsafeMutablePointer<UInt8>?) {
    free(bytes)
}

@_cdecl("appkit_pasteboard_list_free")
public func appkit_pasteboard_list_free(_ list: UnsafeMutablePointer<AppKitPasteboardBytesList>) {
    if let items = list.pointee.items {
        for index in 0..<list.pointee.count {
            free(items[index].bytes)
        }
        free(items)
    }
    list.pointee.items = nil
    list.pointee.count = 0
}
