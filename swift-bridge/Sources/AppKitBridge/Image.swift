import AppKit

@_cdecl("appkit_image_from_data")
public func appkit_image_from_data(_ bytes: UnsafePointer<UInt8>?, _ length: Int) -> UnsafeMutableRawPointer? {
    guard let bytes, length > 0 else {
        return nil
    }
    return appkitRetained(NSImage(data: Data(bytes: bytes, count: length)))
}

@_cdecl("appkit_image_size")
public func appkit_image_size(_ handle: UnsafeMutableRawPointer, _ out: UnsafeMutablePointer<CGSize>) {
    let image: NSImage = appkitObject(handle)
    out.pointee = image.size
}
