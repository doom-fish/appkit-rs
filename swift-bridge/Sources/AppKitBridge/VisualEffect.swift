import AppKit

private let appkitVisualEffectMaterials: [NSVisualEffectView.Material] = [
    .underWindowBackground,
    .windowBackground,
    .sidebar,
    .titlebar,
    .selection,
    .contentBackground,
]

private let appkitVisualEffectBlendingModes: [NSVisualEffectView.BlendingMode] = [
    .behindWindow,
    .withinWindow,
]

private let appkitVisualEffectStates: [NSVisualEffectView.State] = [
    .followsWindowActiveState,
    .active,
    .inactive,
]

@_cdecl("appkit_blur_new_for_window")
public func appkit_blur_new_for_window(_ window: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let window: NSWindow = appkitObject(window)
    guard let contentView = window.contentView else { return nil }
    let view = NSVisualEffectView(frame: contentView.bounds)
    view.autoresizingMask = [.width, .height]
    view.material = .underWindowBackground
    view.blendingMode = .behindWindow
    view.state = .active
    return Unmanaged.passRetained(view).toOpaque()
}

@_cdecl("appkit_blur_material")
public func appkit_blur_material(_ handle: UnsafeMutableRawPointer) -> Int32 {
    let view: NSVisualEffectView = appkitObject(handle)
    return appkitVisualEffectMaterials.firstIndex(of: view.material).map(Int32.init) ?? -1
}

@_cdecl("appkit_blur_set_material")
public func appkit_blur_set_material(_ handle: UnsafeMutableRawPointer, _ material: Int32) {
    let view: NSVisualEffectView = appkitObject(handle)
    guard material >= 0, Int(material) < appkitVisualEffectMaterials.count else { return }
    view.material = appkitVisualEffectMaterials[Int(material)]
}

@_cdecl("appkit_blur_blending_mode")
public func appkit_blur_blending_mode(_ handle: UnsafeMutableRawPointer) -> Int32 {
    let view: NSVisualEffectView = appkitObject(handle)
    return appkitVisualEffectBlendingModes.firstIndex(of: view.blendingMode).map(Int32.init) ?? -1
}

@_cdecl("appkit_blur_set_blending_mode")
public func appkit_blur_set_blending_mode(_ handle: UnsafeMutableRawPointer, _ mode: Int32) {
    let view: NSVisualEffectView = appkitObject(handle)
    guard mode >= 0, Int(mode) < appkitVisualEffectBlendingModes.count else { return }
    view.blendingMode = appkitVisualEffectBlendingModes[Int(mode)]
}

@_cdecl("appkit_blur_state")
public func appkit_blur_state(_ handle: UnsafeMutableRawPointer) -> Int32 {
    let view: NSVisualEffectView = appkitObject(handle)
    return appkitVisualEffectStates.firstIndex(of: view.state).map(Int32.init) ?? -1
}

@_cdecl("appkit_blur_set_state")
public func appkit_blur_set_state(_ handle: UnsafeMutableRawPointer, _ state: Int32) {
    let view: NSVisualEffectView = appkitObject(handle)
    guard state >= 0, Int(state) < appkitVisualEffectStates.count else { return }
    view.state = appkitVisualEffectStates[Int(state)]
}

@_cdecl("appkit_blur_is_emphasized")
public func appkit_blur_is_emphasized(_ handle: UnsafeMutableRawPointer) -> Bool {
    let view: NSVisualEffectView = appkitObject(handle)
    return view.isEmphasized
}

@_cdecl("appkit_blur_set_emphasized")
public func appkit_blur_set_emphasized(_ handle: UnsafeMutableRawPointer, _ emphasized: Bool) {
    let view: NSVisualEffectView = appkitObject(handle)
    view.isEmphasized = emphasized
}

@_cdecl("appkit_blur_attach_below")
public func appkit_blur_attach_below(
    _ handle: UnsafeMutableRawPointer,
    _ window: UnsafeMutableRawPointer,
    _ hosted: UnsafeMutableRawPointer
) -> Bool {
    let view: NSVisualEffectView = appkitObject(handle)
    let window: NSWindow = appkitObject(window)
    let hosted: NSView = appkitObject(hosted)
    guard let contentView = window.contentView,
          hosted.superview === contentView else {
        return false
    }
    view.frame = contentView.bounds
    contentView.addSubview(view, positioned: .below, relativeTo: hosted)
    return true
}

@_cdecl("appkit_blur_detach")
public func appkit_blur_detach(_ handle: UnsafeMutableRawPointer) {
    let view: NSVisualEffectView = appkitObject(handle)
    view.removeFromSuperview()
}
