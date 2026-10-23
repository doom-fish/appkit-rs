import AppKit

@_cdecl("appkit_application_shared")
public func appkit_application_shared() -> UnsafeMutableRawPointer {
    Unmanaged.passRetained(NSApplication.shared).toOpaque()
}

@_cdecl("appkit_application_set_application_icon_image")
public func appkit_application_set_application_icon_image(
    _ handle: UnsafeMutableRawPointer,
    _ image: UnsafeMutableRawPointer?
) {
    let application: NSApplication = appkitObject(handle)
    let icon: NSImage? = image.map { appkitObject($0) }
    application.applicationIconImage = icon
}

@_cdecl("appkit_application_order_front_standard_about_panel")
public func appkit_application_order_front_standard_about_panel(
    _ handle: UnsafeMutableRawPointer,
    _ applicationName: UnsafePointer<CChar>?,
    _ applicationVersion: UnsafePointer<CChar>?,
    _ version: UnsafePointer<CChar>?
) {
    let application: NSApplication = appkitObject(handle)
    var options: [NSApplication.AboutPanelOptionKey: Any] = [:]
    if let applicationName = appkitString(applicationName) {
        options[.applicationName] = applicationName
    }
    if let applicationVersion = appkitString(applicationVersion) {
        options[.applicationVersion] = applicationVersion
    }
    if let version = appkitString(version) {
        options[.version] = version
    }
    application.orderFrontStandardAboutPanel(options: options)
}

@_cdecl("appkit_application_set_activation_policy")
public func appkit_application_set_activation_policy(_ handle: UnsafeMutableRawPointer, _ policy: Int64) -> Bool {
    let application: NSApplication = appkitObject(handle)
    guard let policy = NSApplication.ActivationPolicy(rawValue: Int(policy)) else {
        return false
    }
    return application.setActivationPolicy(policy)
}

@_cdecl("appkit_application_activate")
public func appkit_application_activate(_ handle: UnsafeMutableRawPointer) {
    let application: NSApplication = appkitObject(handle)
    if #available(macOS 14.0, *) {
        application.activate()
    } else {
        application.activate(ignoringOtherApps: true)
    }
}

@_cdecl("appkit_application_run")
public func appkit_application_run(_ handle: UnsafeMutableRawPointer) {
    let application: NSApplication = appkitObject(handle)
    application.run()
}

@_cdecl("appkit_application_terminate")
public func appkit_application_terminate(_ handle: UnsafeMutableRawPointer) {
    let application: NSApplication = appkitObject(handle)
    application.terminate(nil)
}
