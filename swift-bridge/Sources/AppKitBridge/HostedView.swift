import AppKit
import AppKitCallbackABI
import AppKitExceptionShim
import QuartzCore

private func validHostViewSize(_ size: CGSize) -> Bool {
    size.width.isFinite && size.height.isFinite && size.width >= 0 && size.height >= 0
}

private func validHostViewRect(_ rect: CGRect) -> Bool {
    rect.origin.x.isFinite && rect.origin.y.isFinite && validHostViewSize(rect.size)
}

private enum HostViewSelector: UInt32 {
    case performKeyEquivalent = 1
    case keyDown = 2
    case keyUp = 3
    case flagsChanged = 4
    case leftMouseDown = 5
    case leftMouseUp = 6
    case rightMouseDown = 7
    case rightMouseUp = 8
    case otherMouseDown = 9
    case otherMouseUp = 10
    case leftMouseDragged = 11
    case rightMouseDragged = 12
    case otherMouseDragged = 13
    case mouseMoved = 14
    case mouseEntered = 15
    case mouseExited = 16
    case scrollWheel = 17
    case pressureChanged = 18
    case magnified = 19
    case swiped = 20
    case smartMagnified = 21
}

private enum HostViewNotification: UInt32 {
    case backingPropertiesChanged = 1
    case frameSizeChanged = 2
    case effectiveAppearanceChanged = 3
    case displayLayer = 4
    case resetCursorRects = 5
}

private enum HostViewQuery: UInt32 {
    case acceptsFirstMouse = 1
    case acceptsFirstResponder = 2
}

final class HostViewCallbackOwner {
    let token: UInt64
    let table: AppKitHostViewCallbackTableV1

    init?(token: UInt64, table: AppKitHostViewCallbackTableV1) {
        guard Thread.isMainThread,
              token != 0,
              table.header.version == 1,
              table.header.size >= MemoryLayout<AppKitHostViewCallbackTableV1>.size,
              table.deactivate != nil else {
            return nil
        }
        self.token = token
        self.table = table
    }

    deinit {
        table.deactivate?(token)
    }

    fileprivate func event(_ selector: HostViewSelector, _ event: NSEvent) -> Bool {
        guard Thread.isMainThread else { return false }
        return withExtendedLifetime(event) {
            table.event?(token, selector.rawValue, Unmanaged.passUnretained(event).toOpaque()) ?? false
        }
    }

    fileprivate func notification(_ selector: HostViewNotification, _ size: CGSize = .zero) {
        guard Thread.isMainThread else { return }
        table.notification?(token, selector.rawValue, Double(size.width), Double(size.height))
    }

    fileprivate func query(_ selector: HostViewQuery, _ event: NSEvent? = nil) -> Bool {
        guard Thread.isMainThread else { return false }
        return withExtendedLifetime(event) {
            table.query?(token, selector.rawValue, event.map { Unmanaged.passUnretained($0).toOpaque() }) ?? false
        }
    }
}

final class HostViewWeakReference {
    weak var view: HostedView?

    init(_ view: HostedView) {
        self.view = view
    }
}

final class OffscreenHostViewWindow: NSWindow {
    override var canBecomeKey: Bool { false }
    override var canBecomeMain: Bool { false }

    override func constrainFrameRect(_ frameRect: NSRect, to screen: NSScreen?) -> NSRect {
        frameRect
    }
}

class HostedView: NSView, CALayerDelegate {
    private let hostedLayer: CAMetalLayer
    private var callbackOwner: HostViewCallbackOwner?
    fileprivate let titlebarRefreshView: NSView
    fileprivate var ownsTitlebarDrag = false
    fileprivate var titlebarDragQueryCount: UInt64 = 0
    fileprivate var lastTitlebarDragRect: NSRect = .zero
    fileprivate var firstResponderBecomeCount: UInt64 = 0
    fileprivate var firstResponderResignCount: UInt64 = 0
    fileprivate var trackingOptions: NSTrackingArea.Options = []
    private var hostedTrackingArea: NSTrackingArea?
    var textInputOwner: AnyObject?

    init(frame: CGRect, layer: CAMetalLayer, callbacks: HostViewCallbackOwner) {
        hostedLayer = layer
        callbackOwner = callbacks
        titlebarRefreshView = NSView(frame: CGRect(origin: .zero, size: frame.size))
        super.init(frame: frame)
        autoresizingMask = [.width, .height]
        wantsLayer = true
        layerContentsRedrawPolicy = .duringViewResize
        hostedLayer.delegate = self
        titlebarRefreshView.autoresizingMask = [.width, .height]
        titlebarRefreshView.isHidden = true
        addSubview(titlebarRefreshView)
    }

    required init?(coder: NSCoder) {
        return nil
    }

    deinit {
        callbackOwner = nil
        textInputOwner = nil
    }

    override func makeBackingLayer() -> CALayer {
        hostedLayer
    }

    override var wantsUpdateLayer: Bool {
        true
    }

    @objc(displayLayer:) func display(_ layer: CALayer) {
        notify(.displayLayer)
    }

    override func viewDidChangeBackingProperties() {
        super.viewDidChangeBackingProperties()
        notify(.backingPropertiesChanged)
    }

    override func setFrameSize(_ newSize: NSSize) {
        guard validHostViewSize(newSize) else { return }
        let oldSize = frame.size
        guard appkit_try({ super.setFrameSize(newSize) }) == nil else { return }
        if oldSize != newSize {
            notify(.frameSizeChanged, newSize)
        }
    }

    override func viewDidChangeEffectiveAppearance() {
        super.viewDidChangeEffectiveAppearance()
        notify(.effectiveAppearanceChanged)
    }

    override func resetCursorRects() {
        super.resetCursorRects()
        notify(.resetCursorRects)
    }

    @objc(_opaqueRectForWindowMoveWhenInTitlebar)
    func opaqueRectForWindowMoveWhenInTitlebar() -> NSRect {
        let rect = ownsTitlebarDrag ? bounds : .zero
        titlebarDragQueryCount &+= 1
        lastTitlebarDragRect = rect
        return rect
    }

    override var acceptsFirstResponder: Bool {
        withExtendedLifetime(self) {
            callbackOwner?.query(.acceptsFirstResponder) ?? super.acceptsFirstResponder
        }
    }

    override func becomeFirstResponder() -> Bool {
        firstResponderBecomeCount &+= 1
        return super.becomeFirstResponder()
    }

    override func resignFirstResponder() -> Bool {
        firstResponderResignCount &+= 1
        return super.resignFirstResponder()
    }

    override func acceptsFirstMouse(for event: NSEvent?) -> Bool {
        withExtendedLifetime(self) {
            callbackOwner?.query(.acceptsFirstMouse, event) ?? super.acceptsFirstMouse(for: event)
        }
    }

    override func performKeyEquivalent(with event: NSEvent) -> Bool {
        withExtendedLifetime(self) {
            if let callbackOwner {
                return callbackOwner.event(.performKeyEquivalent, event)
            }
            var handled = false
            _ = appkit_try {
                handled = super.performKeyEquivalent(with: event)
            }
            return handled
        }
    }

    override func keyDown(with event: NSEvent) {
        _ = forward(.keyDown, event)
    }

    override func keyUp(with event: NSEvent) {
        _ = forward(.keyUp, event)
    }

    override func flagsChanged(with event: NSEvent) {
        _ = forward(.flagsChanged, event)
    }

    override func mouseDown(with event: NSEvent) {
        _ = forward(.leftMouseDown, event)
    }

    override func mouseUp(with event: NSEvent) {
        _ = forward(.leftMouseUp, event)
    }

    override func rightMouseDown(with event: NSEvent) {
        _ = forward(.rightMouseDown, event)
    }

    override func rightMouseUp(with event: NSEvent) {
        _ = forward(.rightMouseUp, event)
    }

    override func otherMouseDown(with event: NSEvent) {
        _ = forward(.otherMouseDown, event)
    }

    override func otherMouseUp(with event: NSEvent) {
        _ = forward(.otherMouseUp, event)
    }

    override func mouseDragged(with event: NSEvent) {
        _ = forward(.leftMouseDragged, event)
    }

    override func rightMouseDragged(with event: NSEvent) {
        _ = forward(.rightMouseDragged, event)
    }

    override func otherMouseDragged(with event: NSEvent) {
        _ = forward(.otherMouseDragged, event)
    }

    override func mouseMoved(with event: NSEvent) {
        _ = forward(.mouseMoved, event)
    }

    override func mouseEntered(with event: NSEvent) {
        _ = forward(.mouseEntered, event)
    }

    override func mouseExited(with event: NSEvent) {
        _ = forward(.mouseExited, event)
    }

    override func scrollWheel(with event: NSEvent) {
        _ = forward(.scrollWheel, event)
    }

    override func pressureChange(with event: NSEvent) {
        _ = forward(.pressureChanged, event)
    }

    override func magnify(with event: NSEvent) {
        _ = forward(.magnified, event)
    }

    override func swipe(with event: NSEvent) {
        _ = forward(.swiped, event)
    }

    override func smartMagnify(with event: NSEvent) {
        _ = forward(.smartMagnified, event)
    }

    private func forward(_ selector: HostViewSelector, _ event: NSEvent) -> Bool {
        withExtendedLifetime(self) {
            callbackOwner?.event(selector, event) ?? false
        }
    }

    private func notify(_ selector: HostViewNotification, _ size: CGSize = .zero) {
        withExtendedLifetime(self) {
            callbackOwner?.notification(selector, size)
        }
    }

    override func updateTrackingAreas() {
        _ = appkit_try { super.updateTrackingAreas() }
        _ = installTrackingArea()
    }

    func setTrackingOptions(_ raw: UInt64) -> Bool {
        let eventBits: UInt64 = 0x7
        let activeBits: UInt64 = 0xf0
        let allowedBits: UInt64 = 0x7f7
        let visibleRectBit = UInt64(NSTrackingArea.Options.inVisibleRect.rawValue)
        let activity = raw & activeBits
        guard raw == 0 ||
              (raw & ~allowedBits == 0 &&
               raw & eventBits != 0 &&
               raw & visibleRectBit != 0 &&
               activity.nonzeroBitCount == 1) else {
            return false
        }
        let previous = trackingOptions
        trackingOptions = NSTrackingArea.Options(rawValue: UInt(raw))
        if installTrackingArea() {
            return true
        }
        trackingOptions = previous
        _ = installTrackingArea()
        return false
    }

    private func installTrackingArea() -> Bool {
        if let hostedTrackingArea {
            guard appkit_try({ removeTrackingArea(hostedTrackingArea) }) == nil else { return false }
            self.hostedTrackingArea = nil
        }
        guard !trackingOptions.isEmpty else { return true }
        return appkit_try {
            let area = NSTrackingArea(rect: .zero, options: trackingOptions, owner: self, userInfo: nil)
            hostedTrackingArea = area
            addTrackingArea(area)
        } == nil
    }

    func detach() {
        callbackOwner = nil
        textInputOwner = nil
        if hostedLayer.delegate === self {
            hostedLayer.delegate = nil
        }
        trackingOptions = []
        _ = installTrackingArea()
        _ = appkit_try { removeFromSuperview() }
    }
}

@_cdecl("appkit_host_view_new")
public func appkit_host_view_new(
    _ token: UInt64,
    _ table: UnsafePointer<AppKitHostViewCallbackTableV1>?,
    _ borrowedLayer: UnsafeMutableRawPointer?,
    _ frame: UnsafePointer<CGRect>?
) -> UnsafeMutableRawPointer? {
    guard Thread.isMainThread,
          let table,
          let borrowedLayer,
          let frame,
          validHostViewRect(frame.pointee) else {
        return nil
    }
    let layer: CAMetalLayer = appkitObject(borrowedLayer)
    guard !(layer.delegate is HostedView),
          let owner = HostViewCallbackOwner(token: token, table: table.pointee) else {
        return nil
    }
    var view: HostedView?
    guard appkit_try({
        view = HostedView(frame: frame.pointee, layer: layer, callbacks: owner)
    }) == nil else {
        return nil
    }
    return appkitRetained(view)
}

@_cdecl("appkit_host_view_metal_layer_new")
public func appkit_host_view_metal_layer_new() -> UnsafeMutableRawPointer? {
    guard Thread.isMainThread else { return nil }
    var layer: CAMetalLayer?
    guard appkit_try({ layer = CAMetalLayer() }) == nil else { return nil }
    return appkitRetained(layer)
}

@_cdecl("appkit_host_view_detach")
public func appkit_host_view_detach(_ handle: UnsafeMutableRawPointer) {
    let view: HostedView = appkitObject(handle)
    view.detach()
}

@_cdecl("appkit_host_view_attach")
public func appkit_host_view_attach(_ handle: UnsafeMutableRawPointer, _ parentHandle: UnsafeMutableRawPointer) -> Bool {
    let view: HostedView = appkitObject(handle)
    let parent: NSView = appkitObject(parentHandle)
    guard Thread.isMainThread, parent !== view, !parent.isDescendant(of: view) else { return false }
    return appkit_try { parent.addSubview(view) } == nil
}

@_cdecl("appkit_host_view_set_frame_size")
public func appkit_host_view_set_frame_size(_ handle: UnsafeMutableRawPointer, _ width: Double, _ height: Double) -> Bool {
    let view: HostedView = appkitObject(handle)
    let size = CGSize(width: width, height: height)
    guard Thread.isMainThread, validHostViewSize(size) else { return false }
    guard appkit_try({ view.setFrameSize(size) }) == nil else { return false }
    return view.frame.size == size
}

@_cdecl("appkit_host_view_set_tracking_options")
public func appkit_host_view_set_tracking_options(_ handle: UnsafeMutableRawPointer, _ options: UInt64) -> Bool {
    let view: HostedView = appkitObject(handle)
    return view.setTrackingOptions(options)
}

@_cdecl("appkit_host_view_tracking_options")
public func appkit_host_view_tracking_options(_ handle: UnsafeMutableRawPointer) -> UInt64 {
    let view: HostedView = appkitObject(handle)
    return UInt64(view.trackingOptions.rawValue)
}

@_cdecl("appkit_host_view_set_owns_titlebar_drag")
public func appkit_host_view_set_owns_titlebar_drag(_ handle: UnsafeMutableRawPointer, _ owns: Bool) -> Bool {
    guard Thread.isMainThread else { return false }
    let view: HostedView = appkitObject(handle)
    guard view.ownsTitlebarDrag != owns else { return true }
    let previous = view.ownsTitlebarDrag
    view.ownsTitlebarDrag = owns
    guard view.window != nil, !view.isHidden else { return true }
    guard appkit_try({
        view.titlebarRefreshView.isHidden = false
        view.titlebarRefreshView.isHidden = true
        CATransaction.flush()
    }) == nil else {
        view.ownsTitlebarDrag = previous
        _ = appkit_try { view.titlebarRefreshView.isHidden = true }
        return false
    }
    return true
}

@_cdecl("appkit_host_view_invalidate_cursor_rects")
public func appkit_host_view_invalidate_cursor_rects(_ handle: UnsafeMutableRawPointer) -> Bool {
    let view: HostedView = appkitObject(handle)
    guard Thread.isMainThread else { return false }
    return appkit_try {
        if let window = view.window {
            window.invalidateCursorRects(for: view)
        }
        view.resetCursorRects()
    } == nil
}

@_cdecl("appkit_host_view_add_cursor_rect")
public func appkit_host_view_add_cursor_rect(
    _ handle: UnsafeMutableRawPointer,
    _ rect: UnsafePointer<CGRect>,
    _ cursorHandle: UnsafeMutableRawPointer
) -> Bool {
    guard Thread.isMainThread, validHostViewRect(rect.pointee) else { return false }
    let view: HostedView = appkitObject(handle)
    let cursor: NSCursor = appkitObject(cursorHandle)
    return appkit_try { view.addCursorRect(rect.pointee, cursor: cursor) } == nil
}

@_cdecl("appkit_host_view_send_event")
public func appkit_host_view_send_event(
    _ handle: UnsafeMutableRawPointer,
    _ rawSelector: UInt32,
    _ eventHandle: UnsafeMutableRawPointer
) -> Bool {
    guard let selector = HostViewSelector(rawValue: rawSelector) else { return false }
    let view: HostedView = appkitObject(handle)
    let event: NSEvent = appkitObject(eventHandle)
    switch selector {
    case .performKeyEquivalent: return view.performKeyEquivalent(with: event)
    case .keyDown: view.keyDown(with: event)
    case .keyUp: view.keyUp(with: event)
    case .flagsChanged: view.flagsChanged(with: event)
    case .leftMouseDown: view.mouseDown(with: event)
    case .leftMouseUp: view.mouseUp(with: event)
    case .rightMouseDown: view.rightMouseDown(with: event)
    case .rightMouseUp: view.rightMouseUp(with: event)
    case .otherMouseDown: view.otherMouseDown(with: event)
    case .otherMouseUp: view.otherMouseUp(with: event)
    case .leftMouseDragged: view.mouseDragged(with: event)
    case .rightMouseDragged: view.rightMouseDragged(with: event)
    case .otherMouseDragged: view.otherMouseDragged(with: event)
    case .mouseMoved: view.mouseMoved(with: event)
    case .mouseEntered: view.mouseEntered(with: event)
    case .mouseExited: view.mouseExited(with: event)
    case .scrollWheel: view.scrollWheel(with: event)
    case .pressureChanged: view.pressureChange(with: event)
    case .magnified: view.magnify(with: event)
    case .swiped: view.swipe(with: event)
    case .smartMagnified: view.smartMagnify(with: event)
    }
    return true
}

@_cdecl("appkit_host_view_notify_backing_properties")
public func appkit_host_view_notify_backing_properties(_ handle: UnsafeMutableRawPointer) {
    let view: HostedView = appkitObject(handle)
    view.viewDidChangeBackingProperties()
}

@_cdecl("appkit_host_view_notify_appearance")
public func appkit_host_view_notify_appearance(_ handle: UnsafeMutableRawPointer) {
    let view: HostedView = appkitObject(handle)
    view.viewDidChangeEffectiveAppearance()
}

@_cdecl("appkit_host_view_display_layer")
public func appkit_host_view_display_layer(_ handle: UnsafeMutableRawPointer) -> Bool {
    let view: HostedView = appkitObject(handle)
    guard Thread.isMainThread, let layer = view.layer else { return false }
    return appkit_try { layer.display() } == nil
}

@_cdecl("appkit_host_view_accepts_first_mouse")
public func appkit_host_view_accepts_first_mouse(
    _ handle: UnsafeMutableRawPointer,
    _ eventHandle: UnsafeMutableRawPointer?
) -> Bool {
    let view: HostedView = appkitObject(handle)
    let event: NSEvent? = eventHandle.map(appkitObject)
    return view.acceptsFirstMouse(for: event)
}

@_cdecl("appkit_host_view_accepts_first_responder")
public func appkit_host_view_accepts_first_responder(_ handle: UnsafeMutableRawPointer) -> Bool {
    let view: HostedView = appkitObject(handle)
    return view.acceptsFirstResponder
}

@_cdecl("appkit_host_view_example_arrow_cursor")
public func appkit_host_view_example_arrow_cursor() -> UnsafeMutableRawPointer {
    Unmanaged.passRetained(NSCursor.arrow).toOpaque()
}

@_cdecl("appkit_host_view_example_titlebar_drag_rect")
public func appkit_host_view_example_titlebar_drag_rect(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<CGRect>
) -> Bool {
    let view: HostedView = appkitObject(handle)
    guard Thread.isMainThread else { return false }
    out.pointee = view.opaqueRectForWindowMoveWhenInTitlebar()
    return true
}

@_cdecl("appkit_host_view_example_autoreleasepool")
public func appkit_host_view_example_autoreleasepool(
    _ callback: @convention(c) (UnsafeMutableRawPointer?) -> Void,
    _ context: UnsafeMutableRawPointer?
) {
    autoreleasepool {
        callback(context)
    }
}

@_cdecl("appkit_host_view_example_weak_reference")
public func appkit_host_view_example_weak_reference(_ handle: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    let view: HostedView = appkitObject(handle)
    return Unmanaged.passRetained(HostViewWeakReference(view)).toOpaque()
}

@_cdecl("appkit_host_view_example_weak_reference_is_alive")
public func appkit_host_view_example_weak_reference_is_alive(_ handle: UnsafeMutableRawPointer) -> Bool {
    let reference: HostViewWeakReference = appkitObject(handle)
    return reference.view != nil
}

@_cdecl("appkit_host_view_example_titlebar_query")
public func appkit_host_view_example_titlebar_query(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<CGRect>
) -> UInt64 {
    let view: HostedView = appkitObject(handle)
    out.pointee = view.lastTitlebarDragRect
    return view.titlebarDragQueryCount
}

@_cdecl("appkit_host_view_example_titlebar_refresh_view_fills_bounds")
public func appkit_host_view_example_titlebar_refresh_view_fills_bounds(_ handle: UnsafeMutableRawPointer) -> Bool {
    let view: HostedView = appkitObject(handle)
    return view.titlebarRefreshView.isHidden && view.titlebarRefreshView.frame == view.bounds
}

@_cdecl("appkit_host_view_example_first_responder_state")
public func appkit_host_view_example_first_responder_state(
    _ handle: UnsafeMutableRawPointer,
    _ becomes: UnsafeMutablePointer<UInt64>,
    _ resigns: UnsafeMutablePointer<UInt64>
) -> Bool {
    let view: HostedView = appkitObject(handle)
    becomes.pointee = view.firstResponderBecomeCount
    resigns.pointee = view.firstResponderResignCount
    return view.window?.firstResponder === view
}

@_cdecl("appkit_host_view_example_offscreen_titled_window_new")
public func appkit_host_view_example_offscreen_titled_window_new(
    _ x: Double,
    _ y: Double,
    _ width: Double,
    _ height: Double
) -> UnsafeMutableRawPointer? {
    let frame = CGRect(x: x, y: y, width: width, height: height)
    guard Thread.isMainThread, validHostViewRect(frame) else { return nil }
    var window: OffscreenHostViewWindow?
    guard appkit_try({
        window = OffscreenHostViewWindow(
            contentRect: frame,
            styleMask: [.titled, .resizable, .fullSizeContentView],
            backing: .buffered,
            defer: false
        )
        window?.isReleasedWhenClosed = false
        window?.titlebarAppearsTransparent = true
        window?.titleVisibility = .hidden
        window?.alphaValue = 0.0
    }) == nil else { return nil }
    return appkitRetained(window)
}

@_cdecl("appkit_host_view_example_shown_offscreen")
public func appkit_host_view_example_shown_offscreen(_ handle: UnsafeMutableRawPointer) -> Bool {
    let view: HostedView = appkitObject(handle)
    guard let window = view.window, window.isVisible, !NSScreen.screens.isEmpty else { return false }
    return NSScreen.screens.allSatisfy { !window.frame.intersects($0.frame) }
}

@_cdecl("appkit_host_view_example_application_is_active")
public func appkit_host_view_example_application_is_active() -> Bool {
    NSApp?.isActive ?? false
}

@_cdecl("appkit_host_view_example_move_offscreen")
public func appkit_host_view_example_move_offscreen(
    _ handle: UnsafeMutableRawPointer,
    _ x: Double,
    _ y: Double
) -> Bool {
    let view: HostedView = appkitObject(handle)
    guard Thread.isMainThread, x.isFinite, y.isFinite, let window = view.window else { return false }
    guard appkit_try({ window.setFrameOrigin(NSPoint(x: x, y: y)) }) == nil else { return false }
    return appkit_host_view_example_shown_offscreen(handle)
}

@_cdecl("appkit_host_view_example_redraw")
public func appkit_host_view_example_redraw(_ handle: UnsafeMutableRawPointer, _ resize: Bool) -> Bool {
    let view: HostedView = appkitObject(handle)
    guard Thread.isMainThread, appkit_host_view_example_shown_offscreen(handle),
          let window = view.window else { return false }
    return appkit_try {
        if resize {
            let size = view.frame.size
            view.setFrameSize(NSSize(width: size.width + 1, height: size.height + 1))
        } else {
            view.needsDisplay = true
        }
        window.displayIfNeeded()
        CATransaction.flush()
    } == nil
}
