import AppKit
import AppKitExceptionShim

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
    guard index >= 0, index < 4 else { return nil }
    var appearance: NSAppearance?
    guard appkit_try({
        appearance = NSAppearance(named: appkitAppearanceNames[Int(index)])
    }) == nil else { return nil }
    return appearance
}

func appkitAppearanceIndex(_ appearance: NSAppearance?) -> Int32 {
    guard let appearance,
          let name = appearance.bestMatch(from: appkitAppearanceNames),
          let index = appkitAppearanceNames.firstIndex(of: name) else {
        return -1
    }
    return Int32(index)
}

@_cdecl("appkit_appearance_name_raw_value")
public func appkit_appearance_name_raw_value(_ name: Int32) -> UnsafeMutablePointer<CChar>? {
    guard name >= 0, Int(name) < appkitAppearanceNames.count else { return nil }
    return strdup(appkitAppearanceNames[Int(name)].rawValue)
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
) -> Bool {
    let view: NSView = appkitObject(handle)
    let override: NSAppearance?
    if appearance == -1 {
        override = nil
    } else {
        guard let named = appkitAppearance(appearance) else { return false }
        override = named
    }
    return appkit_try {
        view.appearance = override
    } == nil
}
