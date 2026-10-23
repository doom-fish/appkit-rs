import AppKit

@_cdecl("appkit_screen_main")
public func appkit_screen_main() -> UnsafeMutableRawPointer? {
    appkitRetained(NSScreen.main)
}

@_cdecl("appkit_screen_count")
public func appkit_screen_count() -> Int64 {
    Int64(NSScreen.screens.count)
}

@_cdecl("appkit_screen_at")
public func appkit_screen_at(_ index: Int64) -> UnsafeMutableRawPointer? {
    let screens = NSScreen.screens
    guard index >= 0, index < Int64(screens.count) else {
        return nil
    }
    return appkitRetained(screens[Int(index)])
}

@_cdecl("appkit_screen_frame")
public func appkit_screen_frame(_ handle: UnsafeMutableRawPointer, _ out: UnsafeMutablePointer<CGRect>) {
    let screen: NSScreen = appkitObject(handle)
    out.pointee = screen.frame
}

@_cdecl("appkit_screen_visible_frame")
public func appkit_screen_visible_frame(_ handle: UnsafeMutableRawPointer, _ out: UnsafeMutablePointer<CGRect>) {
    let screen: NSScreen = appkitObject(handle)
    out.pointee = screen.visibleFrame
}

@_cdecl("appkit_screen_backing_scale_factor")
public func appkit_screen_backing_scale_factor(_ handle: UnsafeMutableRawPointer) -> Double {
    let screen: NSScreen = appkitObject(handle)
    return Double(screen.backingScaleFactor)
}

@_cdecl("appkit_screen_display_id")
public func appkit_screen_display_id(_ handle: UnsafeMutableRawPointer, _ out: UnsafeMutablePointer<UInt32>) -> Bool {
    let screen: NSScreen = appkitObject(handle)
    guard let number = screen.deviceDescription[NSDeviceDescriptionKey("NSScreenNumber")] as? NSNumber else {
        return false
    }
    out.pointee = number.uint32Value
    return true
}

@_cdecl("appkit_screen_localized_name")
public func appkit_screen_localized_name(_ handle: UnsafeMutableRawPointer) -> UnsafeMutablePointer<CChar>? {
    let screen: NSScreen = appkitObject(handle)
    return strdup(screen.localizedName)
}
