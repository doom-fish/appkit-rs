import AppKit

@_cdecl("appkit_view_window")
public func appkit_view_window(_ handle: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let view: NSView = appkitObject(handle)
    return appkitRetained(view.window)
}

@_cdecl("appkit_view_bounds")
public func appkit_view_bounds(_ handle: UnsafeMutableRawPointer, _ out: UnsafeMutablePointer<CGRect>) {
    let view: NSView = appkitObject(handle)
    out.pointee = view.bounds
}

@_cdecl("appkit_view_frame")
public func appkit_view_frame(_ handle: UnsafeMutableRawPointer, _ out: UnsafeMutablePointer<CGRect>) {
    let view: NSView = appkitObject(handle)
    out.pointee = view.frame
}

@_cdecl("appkit_view_layer")
public func appkit_view_layer(_ handle: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let view: NSView = appkitObject(handle)
    return view.layer.map { Unmanaged.passUnretained($0).toOpaque() }
}

@_cdecl("appkit_view_wants_layer")
public func appkit_view_wants_layer(_ handle: UnsafeMutableRawPointer) -> Bool {
    let view: NSView = appkitObject(handle)
    return view.wantsLayer
}

@_cdecl("appkit_view_set_wants_layer")
public func appkit_view_set_wants_layer(_ handle: UnsafeMutableRawPointer, _ wants: Bool) {
    let view: NSView = appkitObject(handle)
    view.wantsLayer = wants
}
