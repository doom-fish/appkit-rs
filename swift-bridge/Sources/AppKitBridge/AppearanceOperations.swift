import AppKit

let appkitAppearanceNames: [NSAppearance.Name] = [
    .aqua,
    .darkAqua,
    .vibrantLight,
    .vibrantDark,
    .accessibilityHighContrastAqua,
    .accessibilityHighContrastDarkAqua,
    .accessibilityHighContrastVibrantLight,
    .accessibilityHighContrastVibrantDark,
]

func appkitAppearance(_ index: Int32) -> NSAppearance? {
    guard index >= 0, Int(index) < appkitAppearanceNames.count else { return nil }
    return NSAppearance(named: appkitAppearanceNames[Int(index)])
}

func appkitAppearanceIndex(_ appearance: NSAppearance?) -> Int32 {
    guard let appearance,
          let index = appkitAppearanceNames.firstIndex(of: appearance.name) else {
        return -1
    }
    return Int32(index)
}

@_cdecl("appkit_appearance_named")
public func appkit_appearance_named(_ name: Int32) -> UnsafeMutableRawPointer? {
    appkitRetained(appkitAppearance(name))
}

@_cdecl("appkit_appearance_name")
public func appkit_appearance_name(_ handle: UnsafeMutableRawPointer) -> Int32 {
    let appearance: NSAppearance = appkitObject(handle)
    return appkitAppearanceIndex(appearance)
}

@_cdecl("appkit_appearance_window_effective")
public func appkit_appearance_window_effective(
    _ handle: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    let window: NSWindow = appkitObject(handle)
    return appkitRetained(window.effectiveAppearance)
}

@_cdecl("appkit_appearance_view_effective")
public func appkit_appearance_view_effective(
    _ handle: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    let view: NSView = appkitObject(handle)
    return appkitRetained(view.effectiveAppearance)
}

@_cdecl("appkit_appearance_view_override")
public func appkit_appearance_view_override(
    _ handle: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    let view: NSView = appkitObject(handle)
    return appkitRetained(view.appearance)
}

@_cdecl("appkit_appearance_set_view_override")
public func appkit_appearance_set_view_override(
    _ handle: UnsafeMutableRawPointer, _ appearance: Int32
) {
    let view: NSView = appkitObject(handle)
    view.appearance = appkitAppearance(appearance)
}
