import AppKit
import AppKitExceptionShim
import QuartzCore

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

private final class AppKitClearBlurView: NSVisualEffectView {
    override func updateLayer() {
        super.updateLayer()
        guard let layer else { return }
        appkitRemoveBlurTint(from: layer)
    }
}

private func appkitRemoveBlurTint(from layer: CALayer) {
    layer.backgroundColor = nil
    if NSStringFromClass(type(of: layer)).hasSuffix("CAChameleonLayer") {
        layer.isHidden = true
        return
    }
    if let filters = layer.filters {
        let retained = filters.filter { !appkitSaturationDescription(String(describing: $0)) }
        if retained.count != filters.count {
            _ = appkit_try { layer.filters = retained }
        }
    }
    layer.sublayers?.forEach { appkitRemoveBlurTint(from: $0) }
}

private func appkitSaturationDescription(_ description: String) -> Bool {
    description.range(of: "saturat", options: .caseInsensitive) != nil
}

@_cdecl("appkit_blur_description_is_saturation")
public func appkit_blur_description_is_saturation(_ description: UnsafePointer<CChar>) -> Bool {
    appkitSaturationDescription(String(cString: description))
}

@_cdecl("appkit_blur_new_for_window")
public func appkit_blur_new_for_window(
    _ window: UnsafeMutableRawPointer, _ clear: Bool
) -> UnsafeMutableRawPointer? {
    let window: NSWindow = appkitObject(window)
    guard let contentView = window.contentView else { return nil }
    var effect: NSVisualEffectView?
    guard appkit_try({
        let view: NSVisualEffectView = clear
            ? AppKitClearBlurView(frame: contentView.bounds)
            : NSVisualEffectView(frame: contentView.bounds)
        view.autoresizingMask = [.width, .height]
        view.material = .underWindowBackground
        view.blendingMode = .behindWindow
        view.state = .active
        effect = view
    }) == nil else { return nil }
    return appkitRetained(effect)
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
    _ hosted: UnsafeMutableRawPointer
) -> Bool {
    let view: NSVisualEffectView = appkitObject(handle)
    let hosted: NSView = appkitObject(hosted)
    guard let window = hosted.window,
          let contentView = window.contentView,
          hosted.superview === contentView else {
        return false
    }
    return appkit_try {
        view.frame = contentView.bounds
        contentView.addSubview(view, positioned: .below, relativeTo: hosted)
    } == nil
}

@_cdecl("appkit_blur_is_clear_blur")
public func appkit_blur_is_clear_blur(_ handle: UnsafeMutableRawPointer) -> Bool {
    let view: NSVisualEffectView = appkitObject(handle)
    return view is AppKitClearBlurView
}

@_cdecl("appkit_blur_detach")
public func appkit_blur_detach(_ handle: UnsafeMutableRawPointer) {
    let view: NSVisualEffectView = appkitObject(handle)
    _ = appkit_try { view.removeFromSuperview() }
}
