import AppKit

func appkitObject<T: AnyObject>(_ handle: UnsafeMutableRawPointer) -> T {
    Unmanaged<T>.fromOpaque(handle).takeUnretainedValue()
}

func appkitRetained(_ object: AnyObject?) -> UnsafeMutableRawPointer? {
    object.map { Unmanaged.passRetained($0).toOpaque() }
}

func appkitString(_ string: UnsafePointer<CChar>?) -> String? {
    string.map { String(cString: $0) }
}

@_cdecl("appkit_retain")
public func appkit_retain(_ handle: UnsafeMutableRawPointer) {
    _ = Unmanaged<AnyObject>.fromOpaque(handle).retain()
}

@_cdecl("appkit_release")
public func appkit_release(_ handle: UnsafeMutableRawPointer) {
    Unmanaged<AnyObject>.fromOpaque(handle).release()
}

@_cdecl("appkit_string_free")
public func appkit_string_free(_ string: UnsafeMutablePointer<CChar>?) {
    free(string)
}
