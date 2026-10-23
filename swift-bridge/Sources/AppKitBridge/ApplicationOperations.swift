import AppKit
import AppKitExceptionShim

@_cdecl("appkit_application_key_window")
public func appkit_application_key_window(_ handle: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let application: NSApplication = appkitObject(handle)
    return appkitRetained(application.keyWindow)
}

@_cdecl("appkit_application_main_window")
public func appkit_application_main_window(_ handle: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let application: NSApplication = appkitObject(handle)
    return appkitRetained(application.mainWindow)
}

@_cdecl("appkit_application_ordered_windows")
public func appkit_application_ordered_windows(_ handle: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    let application: NSApplication = appkitObject(handle)
    return Unmanaged.passRetained(application.orderedWindows as NSArray).toOpaque()
}

@_cdecl("appkit_application_window_array_count")
public func appkit_application_window_array_count(_ handle: UnsafeMutableRawPointer) -> Int {
    let windows: NSArray = appkitObject(handle)
    return windows.count
}

@_cdecl("appkit_application_window_array_at")
public func appkit_application_window_array_at(
    _ handle: UnsafeMutableRawPointer, _ index: Int
) -> UnsafeMutableRawPointer? {
    let windows: NSArray = appkitObject(handle)
    guard index >= 0, index < windows.count else { return nil }
    return appkitRetained(windows.object(at: index) as? NSWindow)
}

@_cdecl("appkit_application_is_active")
public func appkit_application_is_active(_ handle: UnsafeMutableRawPointer) -> Bool {
    let application: NSApplication = appkitObject(handle)
    return application.isActive
}

@_cdecl("appkit_application_is_hidden")
public func appkit_application_is_hidden(_ handle: UnsafeMutableRawPointer) -> Bool {
    let application: NSApplication = appkitObject(handle)
    return application.isHidden
}

@_cdecl("appkit_application_activate_ignoring_other_apps")
public func appkit_application_activate_ignoring_other_apps(
    _ handle: UnsafeMutableRawPointer, _ ignore: Bool
) {
    let application: NSApplication = appkitObject(handle)
    application.activate(ignoringOtherApps: ignore)
}

@_cdecl("appkit_application_hide")
public func appkit_application_hide(_ handle: UnsafeMutableRawPointer) {
    let application: NSApplication = appkitObject(handle)
    application.hide(nil)
}

@_cdecl("appkit_application_unhide")
public func appkit_application_unhide(_ handle: UnsafeMutableRawPointer) {
    let application: NSApplication = appkitObject(handle)
    application.unhide(nil)
}

@_cdecl("appkit_application_hide_other_applications")
public func appkit_application_hide_other_applications(_ handle: UnsafeMutableRawPointer) {
    let application: NSApplication = appkitObject(handle)
    application.hideOtherApplications(nil)
}

@_cdecl("appkit_application_unhide_all_applications")
public func appkit_application_unhide_all_applications(_ handle: UnsafeMutableRawPointer) {
    let application: NSApplication = appkitObject(handle)
    application.unhideAllApplications(nil)
}

@_cdecl("appkit_application_presentation_options")
public func appkit_application_presentation_options(_ handle: UnsafeMutableRawPointer) -> UInt64 {
    let application: NSApplication = appkitObject(handle)
    return UInt64(application.presentationOptions.rawValue)
}

private func appkitValidPresentationOptions(_ options: NSApplication.PresentationOptions) -> Bool {
    let dockIsHidden = options.contains(.hideDock)
    let dockAutoHides = options.contains(.autoHideDock)
    let menuIsHidden = options.contains(.hideMenuBar)
    let menuAutoHides = options.contains(.autoHideMenuBar)
    if dockIsHidden && dockAutoHides || menuIsHidden && menuAutoHides {
        return false
    }
    if menuIsHidden && !dockIsHidden {
        return false
    }
    let requiresDock: NSApplication.PresentationOptions = [
        .disableProcessSwitching,
        .disableForceQuit,
        .disableSessionTermination,
        .disableMenuBarTransparency,
    ]
    if !options.intersection(requiresDock).isEmpty && !(dockIsHidden || dockAutoHides) {
        return false
    }
    if options.contains(.autoHideToolbar)
        && !(options.contains(.fullScreen) && menuAutoHides) {
        return false
    }
    return true
}

@_cdecl("appkit_application_validate_presentation_options")
public func appkit_application_validate_presentation_options(_ raw: UInt64) -> Bool {
    appkitValidPresentationOptions(NSApplication.PresentationOptions(rawValue: UInt(raw)))
}

@_cdecl("appkit_application_set_presentation_options")
public func appkit_application_set_presentation_options(
    _ handle: UnsafeMutableRawPointer, _ options: UInt64
) -> Bool {
    let requested = NSApplication.PresentationOptions(rawValue: UInt(options))
    guard appkitValidPresentationOptions(requested) else { return false }
    let application: NSApplication = appkitObject(handle)
    return appkit_try {
        application.presentationOptions = requested
    } == nil
}

@_cdecl("appkit_application_request_user_attention")
public func appkit_application_request_user_attention(
    _ handle: UnsafeMutableRawPointer, _ kind: Int32
) -> Int64 {
    let application: NSApplication = appkitObject(handle)
    return Int64(application.requestUserAttention(kind == 0 ? .informationalRequest : .criticalRequest))
}

@_cdecl("appkit_application_cancel_user_attention")
public func appkit_application_cancel_user_attention(
    _ handle: UnsafeMutableRawPointer, _ request: Int64
) {
    let application: NSApplication = appkitObject(handle)
    application.cancelUserAttentionRequest(Int(request))
}

@_cdecl("appkit_application_appearance")
public func appkit_application_appearance(_ handle: UnsafeMutableRawPointer) -> Int32 {
    let application: NSApplication = appkitObject(handle)
    return appkitAppearanceIndex(application.appearance)
}

@_cdecl("appkit_application_effective_appearance")
public func appkit_application_effective_appearance(_ handle: UnsafeMutableRawPointer) -> Int32 {
    let application: NSApplication = appkitObject(handle)
    return appkitAppearanceIndex(application.effectiveAppearance)
}

@_cdecl("appkit_application_set_appearance")
public func appkit_application_set_appearance(_ handle: UnsafeMutableRawPointer, _ appearance: Int32) -> Bool {
    let application: NSApplication = appkitObject(handle)
    let override: NSAppearance?
    if appearance == -1 {
        override = nil
    } else {
        guard let named = appkitAppearance(appearance) else { return false }
        override = named
    }
    return appkit_try {
        application.appearance = override
    } == nil
}

@_cdecl("appkit_application_order_front_character_palette")
public func appkit_application_order_front_character_palette(
    _ handle: UnsafeMutableRawPointer, _ sender: UnsafeMutableRawPointer?
) {
    let application: NSApplication = appkitObject(handle)
    let object: NSWindow? = sender.map { appkitObject($0) }
    application.orderFrontCharacterPalette(object)
}

@_cdecl("appkit_application_change_windows_item")
public func appkit_application_change_windows_item(
    _ handle: UnsafeMutableRawPointer,
    _ window: UnsafeMutableRawPointer,
    _ title: UnsafePointer<CChar>
) -> Bool {
    let application: NSApplication = appkitObject(handle)
    let window: NSWindow = appkitObject(window)
    return appkit_try {
        application.changeWindowsItem(window, title: String(cString: title), filename: false)
    } == nil
}

@_cdecl("appkit_application_current_event")
public func appkit_application_current_event(_ handle: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let application: NSApplication = appkitObject(handle)
    return appkitRetained(application.currentEvent)
}
