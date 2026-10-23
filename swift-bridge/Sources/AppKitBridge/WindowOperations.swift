import AppKit
import AppKitCallbackABI
import AppKitExceptionShim
import ObjectiveC

private let windowCoordinateLimit = Double(Int32.max)

func windowOperationError(_ action: () -> Void) -> UnsafeMutablePointer<CChar>? {
    guard let exception = appkit_try(action) else { return nil }
    return strdup(exception as String)
}

private func validWindowSize(_ size: CGSize) -> Bool {
    let dimensions = [Double(size.width), Double(size.height)]
    return dimensions.allSatisfy { $0.isFinite && $0 >= 0 && $0 <= windowCoordinateLimit }
}

private func validWindowPoint(_ point: CGPoint) -> Bool {
    [Double(point.x), Double(point.y)]
        .allSatisfy { $0.isFinite && abs($0) <= windowCoordinateLimit }
}

private func validWindowFrame(_ frame: CGRect) -> Bool {
    guard validWindowSize(frame.size), validWindowPoint(frame.origin) else { return false }
    let x = Double(frame.origin.x) + Double(frame.size.width)
    let y = Double(frame.origin.y) + Double(frame.size.height)
    return [x, y]
        .allSatisfy { $0.isFinite && abs($0) <= windowCoordinateLimit }
}

private var simpleFullScreenKey: UInt8 = 0
private var sheetCompletionKey: UInt8 = 0
private var simpleFullScreenCount = 0
private var savedPresentationOptions: NSApplication.PresentationOptions = []

private final class SimpleFullScreenState {
    let frame: CGRect
    let styleMask: NSWindow.StyleMask
    var transitioning = true
    private(set) var closed = false
    private var active = true
    private var closeObserver: NSObjectProtocol?

    init(window: NSWindow) {
        frame = window.frame
        styleMask = window.styleMask
        closeObserver = NotificationCenter.default.addObserver(
            forName: NSWindow.willCloseNotification,
            object: window,
            queue: nil
        ) { [weak self] _ in
            self?.closed = true
            self?.releasePresentation()
        }
    }

    func releasePresentation() {
        guard active else { return }
        active = false
        simpleFullScreenCount -= 1
        if simpleFullScreenCount == 0 {
            _ = appkit_try {
                NSApplication.shared.presentationOptions = savedPresentationOptions
            }
        }
    }

    deinit {
        if let closeObserver {
            NotificationCenter.default.removeObserver(closeObserver)
        }
        releasePresentation()
    }
}

private func simpleFullScreenState(for window: NSWindow) -> SimpleFullScreenState? {
    withUnsafePointer(to: &simpleFullScreenKey) {
        objc_getAssociatedObject(window, $0) as? SimpleFullScreenState
    }
}

private func setSimpleFullScreenState(_ state: SimpleFullScreenState?, for window: NSWindow) {
    withUnsafePointer(to: &simpleFullScreenKey) {
        objc_setAssociatedObject(window, $0, state, .OBJC_ASSOCIATION_RETAIN_NONATOMIC)
    }
}

private final class SimpleFullScreenResponderProbeWindow: NSWindow {
    var pretendVisible = false
    var keyRequests = 0
    var responderRequests = 0
    var actualVisible: Bool { super.isVisible }

    override var isVisible: Bool { pretendVisible }

    override func makeKeyAndOrderFront(_ sender: Any?) {
        keyRequests += 1
    }

    override func orderFront(_ sender: Any?) {}

    override func makeFirstResponder(_ responder: NSResponder?) -> Bool {
        responderRequests += 1
        return true
    }
}

private final class SheetCompletionOwner {
    let token: UInt64
    let table: AppKitWindowSheetCallbackTableV1
    private weak var parent: NSWindow?
    private weak var sheet: NSWindow?
    private var observers: [NSObjectProtocol] = []
    private var completed = false
    private var ending = false
    private var aborting = false
    var started = false

    init?(parent: NSWindow, sheet: NSWindow, token: UInt64, table: AppKitWindowSheetCallbackTableV1) {
        guard Thread.isMainThread,
              token != 0,
              table.header.version == 1,
              table.header.size >= MemoryLayout<AppKitWindowSheetCallbackTableV1>.size,
              table.deactivate != nil,
              table.complete != nil else {
            return nil
        }
        self.token = token
        self.table = table
        self.parent = parent
        self.sheet = sheet

        let center = NotificationCenter.default
        observers.append(center.addObserver(
            forName: NSWindow.willCloseNotification,
            object: parent,
            queue: nil
        ) { [weak self] _ in
            self?.abort()
        })
        observers.append(center.addObserver(
            forName: NSWindow.willCloseNotification,
            object: sheet,
            queue: nil
        ) { [weak self] _ in
            self?.abort()
        })
    }

    func finish(_ response: NSApplication.ModalResponse) {
        guard Thread.isMainThread, !completed else { return }
        completed = true
        discardObservers()
        table.complete(token, Int64(response.rawValue))
    }

    func end(on window: NSWindow, sheet: NSWindow, response: NSApplication.ModalResponse) -> Bool {
        guard parent === window, !completed, !ending else { return false }
        ending = true
        let ended = appkit_try {
            window.endSheet(sheet, returnCode: response)
        } == nil
        if !ended && !completed {
            ending = false
        }
        return ended
    }

    func discard() {
        started = false
        discardObservers()
    }

    func abort() {
        guard started, !completed, !aborting else { return }
        aborting = true
        ending = true
        if let parent, let sheet {
            _ = appkit_try {
                parent.endSheet(sheet, returnCode: NSApplication.ModalResponse(rawValue: -1001))
            }
        }
        finish(NSApplication.ModalResponse(rawValue: -1001))
    }

    private func discardObservers() {
        let current = observers
        observers.removeAll()
        let center = NotificationCenter.default
        for observer in current {
            center.removeObserver(observer)
        }
        if let sheet, sheetCompletionOwner(for: sheet) === self {
            setSheetCompletionOwner(nil, for: sheet)
        }
    }

    deinit {
        discardObservers()
        if started && !completed && Thread.isMainThread {
            table.complete(token, -1001)
        }
        table.deactivate(token)
    }
}

private func sheetCompletionOwner(for sheet: NSWindow) -> SheetCompletionOwner? {
    withUnsafePointer(to: &sheetCompletionKey) {
        objc_getAssociatedObject(sheet, $0) as? SheetCompletionOwner
    }
}

private func setSheetCompletionOwner(_ owner: SheetCompletionOwner?, for sheet: NSWindow) {
    withUnsafePointer(to: &sheetCompletionKey) {
        objc_setAssociatedObject(sheet, $0, owner, .OBJC_ASSOCIATION_RETAIN_NONATOMIC)
    }
}

@_cdecl("appkit_window_ops_style_mask")
public func appkit_window_ops_style_mask(_ handle: UnsafeMutableRawPointer) -> UInt64 {
    let window: NSWindow = appkitObject(handle)
    return UInt64(window.styleMask.rawValue)
}

@_cdecl("appkit_window_ops_set_style_mask")
public func appkit_window_ops_set_style_mask(
    _ handle: UnsafeMutableRawPointer,
    _ mask: UInt64
) -> UnsafeMutablePointer<CChar>? {
    let window: NSWindow = appkitObject(handle)
    let styleMask = NSWindow.StyleMask(rawValue: UInt(mask))
    guard styleMask.contains(.fullScreen) == window.styleMask.contains(.fullScreen) else {
        return strdup("AppKit controls the full-screen style bit")
    }
    return windowOperationError { window.styleMask = styleMask }
}

@_cdecl("appkit_window_ops_level")
public func appkit_window_ops_level(_ handle: UnsafeMutableRawPointer) -> Int64 {
    let window: NSWindow = appkitObject(handle)
    return Int64(window.level.rawValue)
}

@_cdecl("appkit_window_ops_set_level")
public func appkit_window_ops_set_level(
    _ handle: UnsafeMutableRawPointer,
    _ level: Int64
) -> UnsafeMutablePointer<CChar>? {
    let window: NSWindow = appkitObject(handle)
    return windowOperationError { window.level = NSWindow.Level(rawValue: Int(level)) }
}

@_cdecl("appkit_window_ops_collection_behavior")
public func appkit_window_ops_collection_behavior(_ handle: UnsafeMutableRawPointer) -> UInt64 {
    let window: NSWindow = appkitObject(handle)
    return UInt64(window.collectionBehavior.rawValue)
}

@_cdecl("appkit_window_ops_set_collection_behavior")
public func appkit_window_ops_set_collection_behavior(
    _ handle: UnsafeMutableRawPointer,
    _ behavior: UInt64
) -> UnsafeMutablePointer<CChar>? {
    let window: NSWindow = appkitObject(handle)
    let collectionBehavior = NSWindow.CollectionBehavior(rawValue: UInt(behavior))
    guard !collectionBehavior.contains([.canJoinAllSpaces, .moveToActiveSpace]) else {
        return strdup("AppKit does not permit both space behaviors")
    }
    return windowOperationError { window.collectionBehavior = collectionBehavior }
}

@_cdecl("appkit_window_ops_make_first_responder")
public func appkit_window_ops_make_first_responder(
    _ handle: UnsafeMutableRawPointer,
    _ viewHandle: UnsafeMutableRawPointer
) -> Bool {
    let window: NSWindow = appkitObject(handle)
    let view: NSView = appkitObject(viewHandle)
    return window.makeFirstResponder(view)
}

@_cdecl("appkit_window_ops_set_initial_first_responder")
public func appkit_window_ops_set_initial_first_responder(
    _ handle: UnsafeMutableRawPointer,
    _ viewHandle: UnsafeMutableRawPointer?
) -> UnsafeMutablePointer<CChar>? {
    let window: NSWindow = appkitObject(handle)
    return windowOperationError {
        window.initialFirstResponder = viewHandle.map { appkitObject($0) as NSView }
    }
}

@_cdecl("appkit_window_ops_set_frame")
public func appkit_window_ops_set_frame(
    _ handle: UnsafeMutableRawPointer,
    _ frame: UnsafePointer<CGRect>,
    _ display: Bool,
    _ animate: Bool
) -> UnsafeMutablePointer<CChar>? {
    let window: NSWindow = appkitObject(handle)
    guard validWindowFrame(frame.pointee) else { return strdup("the window frame is invalid") }
    return windowOperationError {
        window.setFrame(frame.pointee, display: display, animate: animate)
    }
}

@_cdecl("appkit_window_ops_set_frame_top_left_point")
public func appkit_window_ops_set_frame_top_left_point(
    _ handle: UnsafeMutableRawPointer,
    _ point: UnsafePointer<CGPoint>
) -> UnsafeMutablePointer<CChar>? {
    let window: NSWindow = appkitObject(handle)
    guard validWindowPoint(point.pointee) else {
        return strdup("the window frame origin is invalid")
    }
    return windowOperationError { window.setFrameTopLeftPoint(point.pointee) }
}

@_cdecl("appkit_window_ops_content_size")
public func appkit_window_ops_content_size(_ handle: UnsafeMutableRawPointer, _ out: UnsafeMutablePointer<CGSize>) {
    let window: NSWindow = appkitObject(handle)
    out.pointee = window.contentView?.frame.size ?? window.contentRect(forFrameRect: window.frame).size
}

@_cdecl("appkit_window_ops_set_content_size")
public func appkit_window_ops_set_content_size(
    _ handle: UnsafeMutableRawPointer,
    _ size: UnsafePointer<CGSize>
) -> UnsafeMutablePointer<CChar>? {
    let window: NSWindow = appkitObject(handle)
    guard validWindowSize(size.pointee) else { return strdup("the window content size is invalid") }
    return windowOperationError { window.setContentSize(size.pointee) }
}

@_cdecl("appkit_window_ops_frame_rect_for_content_rect")
public func appkit_window_ops_frame_rect_for_content_rect(
    _ handle: UnsafeMutableRawPointer,
    _ rect: UnsafePointer<CGRect>,
    _ out: UnsafeMutablePointer<CGRect>
) {
    let window: NSWindow = appkitObject(handle)
    out.pointee = window.frameRect(forContentRect: rect.pointee)
}

@_cdecl("appkit_window_ops_content_rect_for_frame_rect")
public func appkit_window_ops_content_rect_for_frame_rect(
    _ handle: UnsafeMutableRawPointer,
    _ rect: UnsafePointer<CGRect>,
    _ out: UnsafeMutablePointer<CGRect>
) {
    let window: NSWindow = appkitObject(handle)
    out.pointee = window.contentRect(forFrameRect: rect.pointee)
}

@_cdecl("appkit_window_ops_convert_point_to_screen")
public func appkit_window_ops_convert_point_to_screen(
    _ handle: UnsafeMutableRawPointer,
    _ point: UnsafePointer<CGPoint>,
    _ out: UnsafeMutablePointer<CGPoint>
) {
    let window: NSWindow = appkitObject(handle)
    out.pointee = window.convertPoint(toScreen: point.pointee)
}

@_cdecl("appkit_window_ops_convert_rect_to_screen")
public func appkit_window_ops_convert_rect_to_screen(
    _ handle: UnsafeMutableRawPointer,
    _ rect: UnsafePointer<CGRect>,
    _ out: UnsafeMutablePointer<CGRect>
) {
    let window: NSWindow = appkitObject(handle)
    out.pointee = window.convertToScreen(rect.pointee)
}

@_cdecl("appkit_window_ops_content_min_size")
public func appkit_window_ops_content_min_size(_ handle: UnsafeMutableRawPointer, _ out: UnsafeMutablePointer<CGSize>) {
    let window: NSWindow = appkitObject(handle)
    out.pointee = window.contentMinSize
}

@_cdecl("appkit_window_ops_set_content_min_size")
public func appkit_window_ops_set_content_min_size(
    _ handle: UnsafeMutableRawPointer,
    _ size: UnsafePointer<CGSize>
) -> UnsafeMutablePointer<CChar>? {
    let window: NSWindow = appkitObject(handle)
    guard validWindowSize(size.pointee) else { return strdup("the minimum content size is invalid") }
    return windowOperationError { window.contentMinSize = size.pointee }
}

@_cdecl("appkit_window_ops_mouse_location_outside_event_stream")
public func appkit_window_ops_mouse_location_outside_event_stream(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<CGPoint>
) {
    let window: NSWindow = appkitObject(handle)
    out.pointee = window.mouseLocationOutsideOfEventStream
}

@_cdecl("appkit_window_ops_is_key_window")
public func appkit_window_ops_is_key_window(_ handle: UnsafeMutableRawPointer) -> Bool {
    let window: NSWindow = appkitObject(handle)
    return window.isKeyWindow
}

@_cdecl("appkit_window_ops_is_main_window")
public func appkit_window_ops_is_main_window(_ handle: UnsafeMutableRawPointer) -> Bool {
    let window: NSWindow = appkitObject(handle)
    return window.isMainWindow
}

@_cdecl("appkit_window_ops_is_visible")
public func appkit_window_ops_is_visible(_ handle: UnsafeMutableRawPointer) -> Bool {
    let window: NSWindow = appkitObject(handle)
    return window.isVisible
}

@_cdecl("appkit_window_ops_occlusion_state")
public func appkit_window_ops_occlusion_state(_ handle: UnsafeMutableRawPointer) -> UInt64 {
    let window: NSWindow = appkitObject(handle)
    return UInt64(window.occlusionState.rawValue)
}

@_cdecl("appkit_window_ops_is_miniaturized")
public func appkit_window_ops_is_miniaturized(_ handle: UnsafeMutableRawPointer) -> Bool {
    let window: NSWindow = appkitObject(handle)
    return window.isMiniaturized
}

@_cdecl("appkit_window_ops_miniaturize")
public func appkit_window_ops_miniaturize(_ handle: UnsafeMutableRawPointer) {
    let window: NSWindow = appkitObject(handle)
    window.miniaturize(nil)
}

@_cdecl("appkit_window_ops_deminiaturize")
public func appkit_window_ops_deminiaturize(_ handle: UnsafeMutableRawPointer) {
    let window: NSWindow = appkitObject(handle)
    window.deminiaturize(nil)
}

@_cdecl("appkit_window_ops_is_zoomed")
public func appkit_window_ops_is_zoomed(_ handle: UnsafeMutableRawPointer) -> Bool {
    let window: NSWindow = appkitObject(handle)
    return window.isZoomed
}

@_cdecl("appkit_window_ops_zoom")
public func appkit_window_ops_zoom(_ handle: UnsafeMutableRawPointer) {
    let window: NSWindow = appkitObject(handle)
    window.zoom(nil)
}

@_cdecl("appkit_window_ops_toggle_full_screen")
public func appkit_window_ops_toggle_full_screen(_ handle: UnsafeMutableRawPointer) {
    let window: NSWindow = appkitObject(handle)
    window.toggleFullScreen(nil)
}

@_cdecl("appkit_window_ops_is_simple_full_screen")
public func appkit_window_ops_is_simple_full_screen(_ handle: UnsafeMutableRawPointer) -> Bool {
    let window: NSWindow = appkitObject(handle)
    return simpleFullScreenState(for: window) != nil
}

@_cdecl("appkit_window_ops_probe_closed_simple_full_screen_responder")
public func appkit_window_ops_probe_closed_simple_full_screen_responder() -> Bool {
    guard Thread.isMainThread else { return false }
    let window = SimpleFullScreenResponderProbeWindow(
        contentRect: CGRect(x: 200, y: 200, width: 280, height: 190),
        styleMask: [.titled, .closable],
        backing: .buffered,
        defer: false
    )
    window.isReleasedWhenClosed = false
    guard let contentView = window.contentView else { return false }
    let view = NSView(frame: contentView.bounds)
    contentView.addSubview(view)
    let originalFrame = window.frame
    let originalStyle = window.styleMask
    let handle = Unmanaged.passUnretained(window).toOpaque()
    let viewHandle = Unmanaged.passUnretained(view).toOpaque()
    guard appkit_window_ops_toggle_simple_full_screen(handle, viewHandle) else { return false }
    guard !window.actualVisible, !window.isKeyWindow else { return false }
    window.close()
    guard let saved = simpleFullScreenState(for: window), saved.closed else { return false }

    let keyRequests = window.keyRequests
    let responderRequests = window.responderRequests
    window.pretendVisible = true
    let restored = appkit_window_ops_toggle_simple_full_screen(handle, viewHandle)
    window.pretendVisible = false
    return restored
        && window.keyRequests == keyRequests + 1
        && window.responderRequests == responderRequests + 1
        && window.styleMask == originalStyle
        && window.frame == originalFrame
        && simpleFullScreenState(for: window) == nil
        && !window.actualVisible
        && !window.isKeyWindow
}

@_cdecl("appkit_window_ops_toggle_simple_full_screen")
public func appkit_window_ops_toggle_simple_full_screen(
    _ handle: UnsafeMutableRawPointer,
    _ viewHandle: UnsafeMutableRawPointer
) -> Bool {
    let window: NSWindow = appkitObject(handle)
    let view: NSView = appkitObject(viewHandle)
    guard Thread.isMainThread, !window.styleMask.contains(.fullScreen) else { return false }
    let wasVisible = window.isVisible

    if let saved = simpleFullScreenState(for: window) {
        guard !saved.transitioning, saved.closed || view.window === window else { return false }
        saved.transitioning = true
        saved.releasePresentation()
        let error = appkit_try {
            window.styleMask = saved.styleMask
            window.setFrame(saved.frame, display: true)
        }
        if error != nil {
            saved.transitioning = false
            return false
        }
        if wasVisible {
            _ = appkit_try {
                window.makeKeyAndOrderFront(nil)
                _ = window.makeFirstResponder(view)
            }
        }
        setSimpleFullScreenState(nil, for: window)
        return true
    } else {
        guard view.window === window, let screen = window.screen,
              validWindowFrame(screen.frame) else {
            return false
        }
        let state = SimpleFullScreenState(window: window)
        setSimpleFullScreenState(state, for: window)
        if simpleFullScreenCount == 0 {
            savedPresentationOptions = NSApplication.shared.presentationOptions
        }
        simpleFullScreenCount += 1
        if simpleFullScreenCount == 1 {
            guard appkit_try({
                NSApplication.shared.presentationOptions = [.autoHideDock, .autoHideMenuBar]
            }) == nil else {
                setSimpleFullScreenState(nil, for: window)
                return false
            }
        }
        if state.closed {
            setSimpleFullScreenState(nil, for: window)
            return false
        }
        let error = appkit_try {
            window.styleMask = .borderless
            window.setFrame(screen.frame, display: true)
        }
        if error != nil || state.closed {
            _ = appkit_try {
                window.styleMask = state.styleMask
                window.setFrame(state.frame, display: true)
            }
            setSimpleFullScreenState(nil, for: window)
            return false
        }
        if wasVisible {
            _ = appkit_try {
                window.makeKeyAndOrderFront(nil)
                _ = window.makeFirstResponder(view)
            }
        }
        state.transitioning = false
        return true
    }
}

@_cdecl("appkit_window_ops_begin_sheet")
public func appkit_window_ops_begin_sheet(
    _ handle: UnsafeMutableRawPointer,
    _ sheetHandle: UnsafeMutableRawPointer,
    _ token: UInt64,
    _ table: UnsafePointer<AppKitWindowSheetCallbackTableV1>?
) -> Bool {
    let window: NSWindow = appkitObject(handle)
    let sheet: NSWindow = appkitObject(sheetHandle)
    guard Thread.isMainThread, window !== sheet, sheet.sheetParent == nil,
          sheetCompletionOwner(for: sheet) == nil else {
        return false
    }
    if token == 0 && table == nil {
        return appkit_try { window.beginSheet(sheet) } == nil
    }
    guard let table,
          let owner = SheetCompletionOwner(parent: window, sheet: sheet, token: token, table: table.pointee) else {
        return false
    }
    setSheetCompletionOwner(owner, for: sheet)
    owner.started = true
    let begun = appkit_try {
        window.beginSheet(sheet) { response in
            owner.finish(response)
        }
    } == nil
    if !begun {
        owner.discard()
    }
    return begun
}

@_cdecl("appkit_window_ops_probe_sheet_completion")
public func appkit_window_ops_probe_sheet_completion(
    _ handle: UnsafeMutableRawPointer,
    _ sheetHandle: UnsafeMutableRawPointer,
    _ token: UInt64,
    _ table: UnsafePointer<AppKitWindowSheetCallbackTableV1>?
) -> Bool {
    let window: NSWindow = appkitObject(handle)
    let sheet: NSWindow = appkitObject(sheetHandle)
    guard let table,
          let owner = SheetCompletionOwner(parent: window, sheet: sheet, token: token, table: table.pointee),
          sheetCompletionOwner(for: sheet) == nil else {
        return false
    }
    setSheetCompletionOwner(owner, for: sheet)
    owner.started = true
    return true
}

@_cdecl("appkit_window_ops_end_sheet")
public func appkit_window_ops_end_sheet(
    _ handle: UnsafeMutableRawPointer,
    _ sheetHandle: UnsafeMutableRawPointer,
    _ response: Int64
) -> Bool {
    let window: NSWindow = appkitObject(handle)
    let sheet: NSWindow = appkitObject(sheetHandle)
    let result = NSApplication.ModalResponse(rawValue: Int(response))
    if let owner = sheetCompletionOwner(for: sheet) {
        return owner.end(on: window, sheet: sheet, response: result)
    }
    guard sheet.sheetParent === window else { return false }
    return appkit_try {
        window.endSheet(sheet, returnCode: result)
    } == nil
}

@_cdecl("appkit_window_ops_attached_sheet")
public func appkit_window_ops_attached_sheet(_ handle: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let window: NSWindow = appkitObject(handle)
    return appkitRetained(window.attachedSheet)
}

@_cdecl("appkit_window_ops_sheet_parent")
public func appkit_window_ops_sheet_parent(_ handle: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let window: NSWindow = appkitObject(handle)
    return appkitRetained(window.sheetParent)
}

@_cdecl("appkit_window_ops_allows_automatic_window_tabbing")
public func appkit_window_ops_allows_automatic_window_tabbing() -> Bool {
    NSWindow.allowsAutomaticWindowTabbing
}

@_cdecl("appkit_window_ops_set_allows_automatic_window_tabbing")
public func appkit_window_ops_set_allows_automatic_window_tabbing(_ allows: Bool) {
    NSWindow.allowsAutomaticWindowTabbing = allows
}

@_cdecl("appkit_window_ops_tabbing_identifier")
public func appkit_window_ops_tabbing_identifier(_ handle: UnsafeMutableRawPointer) -> UnsafeMutablePointer<CChar>? {
    let window: NSWindow = appkitObject(handle)
    guard !window.tabbingIdentifier.isEmpty else { return nil }
    return strdup(window.tabbingIdentifier)
}

@_cdecl("appkit_window_ops_set_tabbing_identifier")
public func appkit_window_ops_set_tabbing_identifier(
    _ handle: UnsafeMutableRawPointer,
    _ identifier: UnsafePointer<CChar>?
) -> UnsafeMutablePointer<CChar>? {
    let window: NSWindow = appkitObject(handle)
    if let identifier {
        return windowOperationError {
            window.tabbingIdentifier = String(cString: identifier)
        }
    }
    return windowOperationError {
        _ = window.perform(Selector(("setTabbingIdentifier:")), with: nil)
    }
}

@_cdecl("appkit_window_ops_tabbing_mode")
public func appkit_window_ops_tabbing_mode(_ handle: UnsafeMutableRawPointer) -> Int64 {
    let window: NSWindow = appkitObject(handle)
    return Int64(window.tabbingMode.rawValue)
}

@_cdecl("appkit_window_ops_set_tabbing_mode")
public func appkit_window_ops_set_tabbing_mode(
    _ handle: UnsafeMutableRawPointer,
    _ mode: Int64
) -> UnsafeMutablePointer<CChar>? {
    let window: NSWindow = appkitObject(handle)
    return windowOperationError {
        window.tabbingMode = NSWindow.TabbingMode(rawValue: Int(mode)) ?? .automatic
    }
}

@_cdecl("appkit_window_ops_add_tabbed_window")
public func appkit_window_ops_add_tabbed_window(
    _ handle: UnsafeMutableRawPointer,
    _ otherHandle: UnsafeMutableRawPointer,
    _ ordered: Int64
) -> Bool {
    let window: NSWindow = appkitObject(handle)
    let other: NSWindow = appkitObject(otherHandle)
    guard window !== other,
          ordered == -1 || ordered == 1,
          let mode = NSWindow.OrderingMode(rawValue: Int(ordered)) else {
        return false
    }
    return appkit_try { window.addTabbedWindow(other, ordered: mode) } == nil
}

@_cdecl("appkit_window_ops_tabbed_windows")
public func appkit_window_ops_tabbed_windows(_ handle: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let window: NSWindow = appkitObject(handle)
    guard let windows = window.tabbedWindows else { return nil }
    return appkitRetained(windows as NSArray)
}

@_cdecl("appkit_window_ops_tabbed_windows_count")
public func appkit_window_ops_tabbed_windows_count(_ handle: UnsafeMutableRawPointer) -> Int {
    let windows: NSArray = appkitObject(handle)
    return windows.count
}

@_cdecl("appkit_window_ops_tabbed_window_at")
public func appkit_window_ops_tabbed_window_at(
    _ handle: UnsafeMutableRawPointer,
    _ index: Int
) -> UnsafeMutableRawPointer? {
    let windows: NSArray = appkitObject(handle)
    guard index >= 0, index < windows.count else { return nil }
    return appkitRetained(windows.object(at: index) as? NSWindow)
}

@_cdecl("appkit_window_ops_is_tab_bar_visible")
public func appkit_window_ops_is_tab_bar_visible(_ handle: UnsafeMutableRawPointer) -> Bool {
    let window: NSWindow = appkitObject(handle)
    return window.tabGroup?.isTabBarVisible ?? false
}

@_cdecl("appkit_window_ops_selected_tab")
public func appkit_window_ops_selected_tab(_ handle: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let window: NSWindow = appkitObject(handle)
    return appkitRetained(window.tabGroup?.selectedWindow)
}

@_cdecl("appkit_window_ops_select_tab")
public func appkit_window_ops_select_tab(
    _ handle: UnsafeMutableRawPointer,
    _ selectedHandle: UnsafeMutableRawPointer
) -> Bool {
    let window: NSWindow = appkitObject(handle)
    let selected: NSWindow = appkitObject(selectedHandle)
    guard let group = window.tabGroup, selected.tabGroup === group else { return false }
    group.selectedWindow = selected
    return true
}

@_cdecl("appkit_window_ops_toggle_tab_overview")
public func appkit_window_ops_toggle_tab_overview(_ handle: UnsafeMutableRawPointer) {
    let window: NSWindow = appkitObject(handle)
    window.toggleTabOverview(nil)
}

@_cdecl("appkit_window_ops_move_tab_to_new_window")
public func appkit_window_ops_move_tab_to_new_window(_ handle: UnsafeMutableRawPointer) {
    let window: NSWindow = appkitObject(handle)
    window.moveTabToNewWindow(nil)
}

@_cdecl("appkit_window_ops_merge_all_windows")
public func appkit_window_ops_merge_all_windows(_ handle: UnsafeMutableRawPointer) {
    let window: NSWindow = appkitObject(handle)
    window.mergeAllWindows(nil)
}

@_cdecl("appkit_window_ops_select_next_tab")
public func appkit_window_ops_select_next_tab(_ handle: UnsafeMutableRawPointer) {
    let window: NSWindow = appkitObject(handle)
    window.selectNextTab(nil)
}

@_cdecl("appkit_window_ops_select_previous_tab")
public func appkit_window_ops_select_previous_tab(_ handle: UnsafeMutableRawPointer) {
    let window: NSWindow = appkitObject(handle)
    window.selectPreviousTab(nil)
}

@_cdecl("appkit_window_ops_toggle_tab_bar")
public func appkit_window_ops_toggle_tab_bar(_ handle: UnsafeMutableRawPointer) {
    let window: NSWindow = appkitObject(handle)
    window.toggleTabBar(nil)
}

@_cdecl("appkit_window_ops_standard_window_button")
public func appkit_window_ops_standard_window_button(
    _ handle: UnsafeMutableRawPointer,
    _ button: Int64
) -> UnsafeMutableRawPointer? {
    let window: NSWindow = appkitObject(handle)
    guard button >= 0, let button = NSWindow.ButtonType(rawValue: UInt(button)) else { return nil }
    return appkitRetained(window.standardWindowButton(button))
}

@_cdecl("appkit_window_ops_chrome_superview")
public func appkit_window_ops_chrome_superview(_ handle: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let view: NSView = appkitObject(handle)
    return appkitRetained(view.superview)
}

@_cdecl("appkit_window_ops_chrome_frame")
public func appkit_window_ops_chrome_frame(_ handle: UnsafeMutableRawPointer, _ out: UnsafeMutablePointer<CGRect>) {
    let view: NSView = appkitObject(handle)
    out.pointee = view.frame
}

@_cdecl("appkit_window_ops_chrome_set_frame")
public func appkit_window_ops_chrome_set_frame(
    _ handle: UnsafeMutableRawPointer,
    _ frame: UnsafePointer<CGRect>
) -> UnsafeMutablePointer<CChar>? {
    let view: NSView = appkitObject(handle)
    guard validWindowFrame(frame.pointee) else { return strdup("the chrome frame is invalid") }
    return windowOperationError { view.frame = frame.pointee }
}

@_cdecl("appkit_window_ops_chrome_set_frame_origin")
public func appkit_window_ops_chrome_set_frame_origin(
    _ handle: UnsafeMutableRawPointer,
    _ origin: UnsafePointer<CGPoint>
) -> UnsafeMutablePointer<CChar>? {
    let view: NSView = appkitObject(handle)
    guard validWindowPoint(origin.pointee) else { return strdup("the chrome origin is invalid") }
    return windowOperationError { view.setFrameOrigin(origin.pointee) }
}

@_cdecl("appkit_window_ops_chrome_is_hidden")
public func appkit_window_ops_chrome_is_hidden(_ handle: UnsafeMutableRawPointer) -> Bool {
    let view: NSView = appkitObject(handle)
    return view.isHidden
}

@_cdecl("appkit_window_ops_chrome_set_hidden")
public func appkit_window_ops_chrome_set_hidden(_ handle: UnsafeMutableRawPointer, _ hidden: Bool) {
    let view: NSView = appkitObject(handle)
    view.isHidden = hidden
}

@_cdecl("appkit_window_ops_chrome_update_tracking_areas")
public func appkit_window_ops_chrome_update_tracking_areas(_ handle: UnsafeMutableRawPointer) {
    let view: NSView = appkitObject(handle)
    view.updateTrackingAreas()
}

@_cdecl("appkit_window_ops_view_set_needs_display")
public func appkit_window_ops_view_set_needs_display(_ handle: UnsafeMutableRawPointer, _ needsDisplay: Bool) {
    let view: NSView = appkitObject(handle)
    view.needsDisplay = needsDisplay
}

@_cdecl("appkit_window_ops_background_color")
public func appkit_window_ops_background_color(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<AppKitRGBA>
) -> Bool {
    let window: NSWindow = appkitObject(handle)
    let background: NSColor? = window.backgroundColor
    guard let background, let color = background.usingColorSpace(.sRGB) else { return false }
    out.pointee = AppKitRGBA(
        red: Double(color.redComponent),
        green: Double(color.greenComponent),
        blue: Double(color.blueComponent),
        alpha: Double(color.alphaComponent)
    )
    return true
}

@_cdecl("appkit_window_ops_set_background_color")
public func appkit_window_ops_set_background_color(
    _ handle: UnsafeMutableRawPointer,
    _ color: UnsafePointer<AppKitRGBA>
) -> UnsafeMutablePointer<CChar>? {
    let window: NSWindow = appkitObject(handle)
    let color = color.pointee
    guard [color.red, color.green, color.blue, color.alpha]
        .allSatisfy({ $0.isFinite && $0 >= 0 && $0 <= 1 }) else {
        return strdup("the background color is invalid")
    }
    return windowOperationError {
        window.backgroundColor = NSColor(
            srgbRed: CGFloat(color.red),
            green: CGFloat(color.green),
            blue: CGFloat(color.blue),
            alpha: CGFloat(color.alpha)
        )
    }
}

@_cdecl("appkit_window_ops_is_opaque")
public func appkit_window_ops_is_opaque(_ handle: UnsafeMutableRawPointer) -> Bool {
    let window: NSWindow = appkitObject(handle)
    return window.isOpaque
}

@_cdecl("appkit_window_ops_set_opaque")
public func appkit_window_ops_set_opaque(_ handle: UnsafeMutableRawPointer, _ opaque: Bool) {
    let window: NSWindow = appkitObject(handle)
    window.isOpaque = opaque
}

@_cdecl("appkit_window_ops_has_shadow")
public func appkit_window_ops_has_shadow(_ handle: UnsafeMutableRawPointer) -> Bool {
    let window: NSWindow = appkitObject(handle)
    return window.hasShadow
}

@_cdecl("appkit_window_ops_set_has_shadow")
public func appkit_window_ops_set_has_shadow(_ handle: UnsafeMutableRawPointer, _ shadow: Bool) {
    let window: NSWindow = appkitObject(handle)
    window.hasShadow = shadow
}

@_cdecl("appkit_window_ops_invalidate_shadow")
public func appkit_window_ops_invalidate_shadow(_ handle: UnsafeMutableRawPointer) {
    let window: NSWindow = appkitObject(handle)
    window.invalidateShadow()
}

@_cdecl("appkit_window_ops_titlebar_appears_transparent")
public func appkit_window_ops_titlebar_appears_transparent(_ handle: UnsafeMutableRawPointer) -> Bool {
    let window: NSWindow = appkitObject(handle)
    return window.titlebarAppearsTransparent
}

@_cdecl("appkit_window_ops_set_titlebar_appears_transparent")
public func appkit_window_ops_set_titlebar_appears_transparent(
    _ handle: UnsafeMutableRawPointer,
    _ transparent: Bool
) {
    let window: NSWindow = appkitObject(handle)
    window.titlebarAppearsTransparent = transparent
}

@_cdecl("appkit_window_ops_is_movable")
public func appkit_window_ops_is_movable(_ handle: UnsafeMutableRawPointer) -> Bool {
    let window: NSWindow = appkitObject(handle)
    return window.isMovable
}

@_cdecl("appkit_window_ops_set_movable")
public func appkit_window_ops_set_movable(_ handle: UnsafeMutableRawPointer, _ movable: Bool) {
    let window: NSWindow = appkitObject(handle)
    window.isMovable = movable
}

@_cdecl("appkit_window_ops_is_movable_by_window_background")
public func appkit_window_ops_is_movable_by_window_background(_ handle: UnsafeMutableRawPointer) -> Bool {
    let window: NSWindow = appkitObject(handle)
    return window.isMovableByWindowBackground
}

@_cdecl("appkit_window_ops_set_movable_by_window_background")
public func appkit_window_ops_set_movable_by_window_background(
    _ handle: UnsafeMutableRawPointer,
    _ movable: Bool
) {
    let window: NSWindow = appkitObject(handle)
    window.isMovableByWindowBackground = movable
}

@_cdecl("appkit_window_ops_accepts_mouse_moved_events")
public func appkit_window_ops_accepts_mouse_moved_events(_ handle: UnsafeMutableRawPointer) -> Bool {
    let window: NSWindow = appkitObject(handle)
    return window.acceptsMouseMovedEvents
}

@_cdecl("appkit_window_ops_set_accepts_mouse_moved_events")
public func appkit_window_ops_set_accepts_mouse_moved_events(
    _ handle: UnsafeMutableRawPointer,
    _ accepts: Bool
) {
    let window: NSWindow = appkitObject(handle)
    window.acceptsMouseMovedEvents = accepts
}

@_cdecl("appkit_window_ops_perform_window_drag")
public func appkit_window_ops_perform_window_drag(
    _ handle: UnsafeMutableRawPointer,
    _ eventHandle: UnsafeMutableRawPointer
) -> UnsafeMutablePointer<CChar>? {
    let window: NSWindow = appkitObject(handle)
    let event: NSEvent = appkitObject(eventHandle)
    guard event.window === window else { return strdup("the drag event has another window") }
    return windowOperationError { window.performDrag(with: event) }
}

@_cdecl("appkit_window_ops_is_document_edited")
public func appkit_window_ops_is_document_edited(_ handle: UnsafeMutableRawPointer) -> Bool {
    let window: NSWindow = appkitObject(handle)
    return window.isDocumentEdited
}

@_cdecl("appkit_window_ops_set_document_edited")
public func appkit_window_ops_set_document_edited(_ handle: UnsafeMutableRawPointer, _ edited: Bool) {
    let window: NSWindow = appkitObject(handle)
    window.isDocumentEdited = edited
}

@_cdecl("appkit_window_ops_represented_filename")
public func appkit_window_ops_represented_filename(_ handle: UnsafeMutableRawPointer) -> UnsafeMutablePointer<CChar>? {
    let window: NSWindow = appkitObject(handle)
    guard !window.representedFilename.isEmpty else { return nil }
    return strdup(window.representedFilename)
}

@_cdecl("appkit_window_ops_set_represented_filename")
public func appkit_window_ops_set_represented_filename(
    _ handle: UnsafeMutableRawPointer,
    _ bytes: UnsafePointer<CChar>?,
    _ length: Int
) -> UnsafeMutablePointer<CChar>? {
    let window: NSWindow = appkitObject(handle)
    guard length >= 0 else { return strdup("the represented filename length is invalid") }
    if let bytes {
        guard strlen(bytes) == length, let filename = String(validatingUTF8: bytes) else {
            return strdup("the represented filename is not UTF-8")
        }
        return windowOperationError { window.representedFilename = filename }
    }
    guard length == 0 else { return strdup("the represented filename is missing") }
    return windowOperationError { window.representedFilename = "" }
}

@_cdecl("appkit_window_ops_order_out")
public func appkit_window_ops_order_out(_ handle: UnsafeMutableRawPointer) {
    let window: NSWindow = appkitObject(handle)
    sheetCompletionOwner(for: window)?.abort()
    _ = appkit_try { window.orderOut(nil) }
}

@_cdecl("appkit_window_ops_order_back")
public func appkit_window_ops_order_back(_ handle: UnsafeMutableRawPointer) {
    let window: NSWindow = appkitObject(handle)
    window.orderBack(nil)
}

@_cdecl("appkit_window_ops_order_window")
public func appkit_window_ops_order_window(
    _ handle: UnsafeMutableRawPointer,
    _ ordered: Int64,
    _ relativeTo: Int64
) -> UnsafeMutablePointer<CChar>? {
    let window: NSWindow = appkitObject(handle)
    guard let mode = NSWindow.OrderingMode(rawValue: Int(ordered)) else {
        return strdup("the window order is invalid")
    }
    if mode == .out {
        sheetCompletionOwner(for: window)?.abort()
    }
    return windowOperationError {
        window.order(mode, relativeTo: Int(relativeTo))
    }
}

@_cdecl("appkit_window_ops_window_number")
public func appkit_window_ops_window_number(_ handle: UnsafeMutableRawPointer) -> Int64 {
    let window: NSWindow = appkitObject(handle)
    return Int64(window.windowNumber)
}

@_cdecl("appkit_window_ops_display_if_needed")
public func appkit_window_ops_display_if_needed(_ handle: UnsafeMutableRawPointer) {
    let window: NSWindow = appkitObject(handle)
    window.displayIfNeeded()
}

@_cdecl("appkit_window_ops_set_content_view_needs_display")
public func appkit_window_ops_set_content_view_needs_display(
    _ handle: UnsafeMutableRawPointer,
    _ needsDisplay: Bool
) {
    let window: NSWindow = appkitObject(handle)
    window.contentView?.needsDisplay = needsDisplay
}

@_cdecl("appkit_window_ops_invalidate_cursor_rects_for_view")
public func appkit_window_ops_invalidate_cursor_rects_for_view(
    _ handle: UnsafeMutableRawPointer,
    _ viewHandle: UnsafeMutableRawPointer
) {
    let window: NSWindow = appkitObject(handle)
    let view: NSView = appkitObject(viewHandle)
    window.invalidateCursorRects(for: view)
}

@_cdecl("appkit_window_ops_resign_key_window")
public func appkit_window_ops_resign_key_window(_ handle: UnsafeMutableRawPointer) {
    let window: NSWindow = appkitObject(handle)
    window.resignKey()
}

@_cdecl("appkit_window_ops_is_released_when_closed")
public func appkit_window_ops_is_released_when_closed(_ handle: UnsafeMutableRawPointer) -> Bool {
    let window: NSWindow = appkitObject(handle)
    return window.isReleasedWhenClosed
}

@_cdecl("appkit_window_ops_retain_when_closed")
public func appkit_window_ops_retain_when_closed(_ handle: UnsafeMutableRawPointer) {
    let window: NSWindow = appkitObject(handle)
    window.isReleasedWhenClosed = false
}
