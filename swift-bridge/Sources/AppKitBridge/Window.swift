import AppKit

private let appearanceNames: [NSAppearance.Name] = [
    .aqua,
    .darkAqua,
    .vibrantLight,
    .vibrantDark,
    .accessibilityHighContrastAqua,
    .accessibilityHighContrastDarkAqua,
    .accessibilityHighContrastVibrantLight,
    .accessibilityHighContrastVibrantDark,
]

@_cdecl("appkit_window_new")
public func appkit_window_new(
    _ x: Double,
    _ y: Double,
    _ width: Double,
    _ height: Double,
    _ title: UnsafePointer<CChar>
) -> UnsafeMutableRawPointer {
    let window = NSWindow(
        contentRect: NSRect(x: x, y: y, width: width, height: height),
        styleMask: [.titled, .closable, .miniaturizable, .resizable],
        backing: .buffered,
        defer: false
    )
    window.isReleasedWhenClosed = false
    window.title = String(cString: title)
    return Unmanaged.passRetained(window).toOpaque()
}

@_cdecl("appkit_window_content_view")
public func appkit_window_content_view(_ handle: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let window: NSWindow = appkitObject(handle)
    return appkitRetained(window.contentView)
}

@_cdecl("appkit_window_appearance")
public func appkit_window_appearance(_ handle: UnsafeMutableRawPointer) -> Int32 {
    let window: NSWindow = appkitObject(handle)
    guard let name = window.appearance?.name, let index = appearanceNames.firstIndex(of: name) else {
        return -1
    }
    return Int32(index)
}

@_cdecl("appkit_window_set_appearance")
public func appkit_window_set_appearance(_ handle: UnsafeMutableRawPointer, _ appearance: Int32) {
    let window: NSWindow = appkitObject(handle)
    guard appearance >= 0, Int(appearance) < appearanceNames.count else {
        window.appearance = nil
        return
    }
    window.appearance = NSAppearance(named: appearanceNames[Int(appearance)])
}

@_cdecl("appkit_window_title")
public func appkit_window_title(_ handle: UnsafeMutableRawPointer) -> UnsafeMutablePointer<CChar>? {
    let window: NSWindow = appkitObject(handle)
    return strdup(window.title)
}

@_cdecl("appkit_window_set_title")
public func appkit_window_set_title(_ handle: UnsafeMutableRawPointer, _ title: UnsafePointer<CChar>) {
    let window: NSWindow = appkitObject(handle)
    window.title = String(cString: title)
}

@_cdecl("appkit_window_title_visibility")
public func appkit_window_title_visibility(_ handle: UnsafeMutableRawPointer) -> Int64 {
    let window: NSWindow = appkitObject(handle)
    return Int64(window.titleVisibility.rawValue)
}

@_cdecl("appkit_window_set_title_visibility")
public func appkit_window_set_title_visibility(_ handle: UnsafeMutableRawPointer, _ visibility: Int64) {
    let window: NSWindow = appkitObject(handle)
    window.titleVisibility = NSWindow.TitleVisibility(rawValue: Int(visibility)) ?? .visible
}

@_cdecl("appkit_window_titlebar_separator_style")
public func appkit_window_titlebar_separator_style(_ handle: UnsafeMutableRawPointer) -> Int64 {
    let window: NSWindow = appkitObject(handle)
    return Int64(window.titlebarSeparatorStyle.rawValue)
}

@_cdecl("appkit_window_set_titlebar_separator_style")
public func appkit_window_set_titlebar_separator_style(_ handle: UnsafeMutableRawPointer, _ style: Int64) {
    let window: NSWindow = appkitObject(handle)
    window.titlebarSeparatorStyle = NSTitlebarSeparatorStyle(rawValue: Int(style)) ?? .automatic
}

@_cdecl("appkit_window_frame")
public func appkit_window_frame(_ handle: UnsafeMutableRawPointer, _ out: UnsafeMutablePointer<CGRect>) {
    let window: NSWindow = appkitObject(handle)
    out.pointee = window.frame
}

@_cdecl("appkit_window_content_layout_rect")
public func appkit_window_content_layout_rect(_ handle: UnsafeMutableRawPointer, _ out: UnsafeMutablePointer<CGRect>) {
    let window: NSWindow = appkitObject(handle)
    out.pointee = window.contentLayoutRect
}

@_cdecl("appkit_window_backing_scale_factor")
public func appkit_window_backing_scale_factor(_ handle: UnsafeMutableRawPointer) -> Double {
    let window: NSWindow = appkitObject(handle)
    return Double(window.backingScaleFactor)
}

@_cdecl("appkit_window_screen")
public func appkit_window_screen(_ handle: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let window: NSWindow = appkitObject(handle)
    return appkitRetained(window.screen)
}

@_cdecl("appkit_window_alpha_value")
public func appkit_window_alpha_value(_ handle: UnsafeMutableRawPointer) -> Double {
    let window: NSWindow = appkitObject(handle)
    return Double(window.alphaValue)
}

@_cdecl("appkit_window_set_alpha_value")
public func appkit_window_set_alpha_value(_ handle: UnsafeMutableRawPointer, _ alpha: Double) {
    let window: NSWindow = appkitObject(handle)
    window.alphaValue = CGFloat(alpha)
}

@_cdecl("appkit_window_animate_alpha_value")
public func appkit_window_animate_alpha_value(_ handle: UnsafeMutableRawPointer, _ alpha: Double) {
    let window: NSWindow = appkitObject(handle)
    window.animator().alphaValue = CGFloat(alpha)
}

@_cdecl("appkit_window_animation_behavior")
public func appkit_window_animation_behavior(_ handle: UnsafeMutableRawPointer) -> Int64 {
    let window: NSWindow = appkitObject(handle)
    return Int64(window.animationBehavior.rawValue)
}

@_cdecl("appkit_window_set_animation_behavior")
public func appkit_window_set_animation_behavior(_ handle: UnsafeMutableRawPointer, _ behavior: Int64) {
    let window: NSWindow = appkitObject(handle)
    window.animationBehavior = NSWindow.AnimationBehavior(rawValue: Int(behavior)) ?? .default
}

@_cdecl("appkit_window_order_front")
public func appkit_window_order_front(_ handle: UnsafeMutableRawPointer) {
    let window: NSWindow = appkitObject(handle)
    window.orderFront(nil)
}

@_cdecl("appkit_window_make_key_and_order_front")
public func appkit_window_make_key_and_order_front(_ handle: UnsafeMutableRawPointer) {
    let window: NSWindow = appkitObject(handle)
    window.makeKeyAndOrderFront(nil)
}

@_cdecl("appkit_window_close")
public func appkit_window_close(_ handle: UnsafeMutableRawPointer) {
    let window: NSWindow = appkitObject(handle)
    window.close()
}
