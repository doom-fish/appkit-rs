import AppKit
import AppKitCallbackABI
import ObjectiveC

private var simpleFullScreenKey: UInt8 = 0
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
            NSApplication.shared.presentationOptions = savedPresentationOptions
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

private final class SheetCompletionOwner {
    let token: UInt64
    let table: AppKitWindowSheetCallbackTableV1
    private var completed = false

    init?(token: UInt64, table: AppKitWindowSheetCallbackTableV1) {
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
    }

    func finish(_ response: NSApplication.ModalResponse) {
        guard Thread.isMainThread, !completed else { return }
        completed = true
        table.complete(token, Int64(response.rawValue))
    }

    deinit {
        table.deactivate(token)
    }
}

@_cdecl("appkit_window_ops_style_mask")
public func appkit_window_ops_style_mask(_ handle: UnsafeMutableRawPointer) -> UInt64 {
    let window: NSWindow = appkitObject(handle)
    return UInt64(window.styleMask.rawValue)
}

@_cdecl("appkit_window_ops_set_style_mask")
public func appkit_window_ops_set_style_mask(_ handle: UnsafeMutableRawPointer, _ mask: UInt64) {
    let window: NSWindow = appkitObject(handle)
    window.styleMask = NSWindow.StyleMask(rawValue: UInt(mask))
}

@_cdecl("appkit_window_ops_level")
public func appkit_window_ops_level(_ handle: UnsafeMutableRawPointer) -> Int64 {
    let window: NSWindow = appkitObject(handle)
    return Int64(window.level.rawValue)
}

@_cdecl("appkit_window_ops_set_level")
public func appkit_window_ops_set_level(_ handle: UnsafeMutableRawPointer, _ level: Int64) {
    let window: NSWindow = appkitObject(handle)
    window.level = NSWindow.Level(rawValue: Int(level))
}

@_cdecl("appkit_window_ops_collection_behavior")
public func appkit_window_ops_collection_behavior(_ handle: UnsafeMutableRawPointer) -> UInt64 {
    let window: NSWindow = appkitObject(handle)
    return UInt64(window.collectionBehavior.rawValue)
}

@_cdecl("appkit_window_ops_set_collection_behavior")
public func appkit_window_ops_set_collection_behavior(_ handle: UnsafeMutableRawPointer, _ behavior: UInt64) {
    let window: NSWindow = appkitObject(handle)
    window.collectionBehavior = NSWindow.CollectionBehavior(rawValue: UInt(behavior))
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
) {
    let window: NSWindow = appkitObject(handle)
    window.initialFirstResponder = viewHandle.map { appkitObject($0) as NSView }
}

@_cdecl("appkit_window_ops_set_frame")
public func appkit_window_ops_set_frame(
    _ handle: UnsafeMutableRawPointer,
    _ frame: UnsafePointer<CGRect>,
    _ display: Bool,
    _ animate: Bool
) {
    let window: NSWindow = appkitObject(handle)
    window.setFrame(frame.pointee, display: display, animate: animate)
}

@_cdecl("appkit_window_ops_set_frame_top_left_point")
public func appkit_window_ops_set_frame_top_left_point(
    _ handle: UnsafeMutableRawPointer,
    _ point: UnsafePointer<CGPoint>
) {
    let window: NSWindow = appkitObject(handle)
    window.setFrameTopLeftPoint(point.pointee)
}

@_cdecl("appkit_window_ops_content_size")
public func appkit_window_ops_content_size(_ handle: UnsafeMutableRawPointer, _ out: UnsafeMutablePointer<CGSize>) {
    let window: NSWindow = appkitObject(handle)
    out.pointee = window.contentView?.frame.size ?? window.contentRect(forFrameRect: window.frame).size
}

@_cdecl("appkit_window_ops_set_content_size")
public func appkit_window_ops_set_content_size(_ handle: UnsafeMutableRawPointer, _ size: UnsafePointer<CGSize>) {
    let window: NSWindow = appkitObject(handle)
    window.setContentSize(size.pointee)
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
public func appkit_window_ops_set_content_min_size(_ handle: UnsafeMutableRawPointer, _ size: UnsafePointer<CGSize>) {
    let window: NSWindow = appkitObject(handle)
    window.contentMinSize = size.pointee
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
    return simpleFullScreenState(for: window).map { !$0.closed } ?? false
}

@_cdecl("appkit_window_ops_toggle_simple_full_screen")
public func appkit_window_ops_toggle_simple_full_screen(
    _ handle: UnsafeMutableRawPointer,
    _ viewHandle: UnsafeMutableRawPointer
) -> Bool {
    let window: NSWindow = appkitObject(handle)
    let view: NSView = appkitObject(viewHandle)
    guard Thread.isMainThread, view.window === window else { return false }

    if let saved = simpleFullScreenState(for: window) {
        guard !saved.transitioning else { return false }
        saved.transitioning = true
        saved.releasePresentation()
        if !saved.closed {
            window.styleMask = saved.styleMask
        }
        if !saved.closed {
            window.setFrame(saved.frame, display: true)
        }
        if !saved.closed {
            window.makeKeyAndOrderFront(nil)
        }
        if !saved.closed {
            _ = window.makeFirstResponder(view)
        }
        setSimpleFullScreenState(nil, for: window)
        return !saved.closed
    } else {
        guard !window.styleMask.contains(.fullScreen), let screen = window.screen else {
            return false
        }
        let state = SimpleFullScreenState(window: window)
        setSimpleFullScreenState(state, for: window)
        if simpleFullScreenCount == 0 {
            savedPresentationOptions = NSApplication.shared.presentationOptions
        }
        simpleFullScreenCount += 1
        if simpleFullScreenCount == 1 {
            NSApplication.shared.presentationOptions = [.autoHideDock, .autoHideMenuBar]
        }
        if state.closed {
            setSimpleFullScreenState(nil, for: window)
            return false
        }
        window.styleMask = .borderless
        if !state.closed {
            window.setFrame(screen.frame, display: true)
        }
        if !state.closed {
            window.makeKeyAndOrderFront(nil)
        }
        if !state.closed {
            _ = window.makeFirstResponder(view)
        }
        if state.closed {
            setSimpleFullScreenState(nil, for: window)
            return false
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
    guard Thread.isMainThread, window !== sheet, sheet.sheetParent == nil else { return false }
    if token == 0 && table == nil {
        window.beginSheet(sheet)
        return true
    }
    guard let table, let owner = SheetCompletionOwner(token: token, table: table.pointee) else {
        return false
    }
    window.beginSheet(sheet) { response in
        owner.finish(response)
    }
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
    guard sheet.sheetParent === window else { return false }
    window.endSheet(sheet, returnCode: NSApplication.ModalResponse(rawValue: Int(response)))
    return true
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
) {
    let window: NSWindow = appkitObject(handle)
    window.tabbingIdentifier = identifier.map { String(cString: $0) } ?? ""
}

@_cdecl("appkit_window_ops_tabbing_mode")
public func appkit_window_ops_tabbing_mode(_ handle: UnsafeMutableRawPointer) -> Int64 {
    let window: NSWindow = appkitObject(handle)
    return Int64(window.tabbingMode.rawValue)
}

@_cdecl("appkit_window_ops_set_tabbing_mode")
public func appkit_window_ops_set_tabbing_mode(_ handle: UnsafeMutableRawPointer, _ mode: Int64) {
    let window: NSWindow = appkitObject(handle)
    window.tabbingMode = NSWindow.TabbingMode(rawValue: Int(mode)) ?? .automatic
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
    window.addTabbedWindow(other, ordered: mode)
    return true
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
public func appkit_window_ops_chrome_set_frame(_ handle: UnsafeMutableRawPointer, _ frame: UnsafePointer<CGRect>) {
    let view: NSView = appkitObject(handle)
    view.frame = frame.pointee
}

@_cdecl("appkit_window_ops_chrome_set_frame_origin")
public func appkit_window_ops_chrome_set_frame_origin(
    _ handle: UnsafeMutableRawPointer,
    _ origin: UnsafePointer<CGPoint>
) {
    let view: NSView = appkitObject(handle)
    view.setFrameOrigin(origin.pointee)
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
) {
    let window: NSWindow = appkitObject(handle)
    let color = color.pointee
    window.backgroundColor = NSColor(
        srgbRed: CGFloat(color.red),
        green: CGFloat(color.green),
        blue: CGFloat(color.blue),
        alpha: CGFloat(color.alpha)
    )
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
) {
    let window: NSWindow = appkitObject(handle)
    let event: NSEvent = appkitObject(eventHandle)
    guard event.window === window else { return }
    window.performDrag(with: event)
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

private func fileSystemBytes(_ path: String) -> [CChar]? {
    let string = path as NSString
    let maximum = string.maximumLengthOfBytes(using: String.Encoding.utf8.rawValue)
    guard maximum < Int.max / 4 else { return nil }
    var buffer = [CChar](repeating: 0, count: max(256, maximum * 4 + 1))
    let converted = buffer.withUnsafeMutableBufferPointer {
        string.getFileSystemRepresentation($0.baseAddress!, maxLength: $0.count)
    }
    return converted ? buffer : nil
}

private func hasFileSystemBytes(_ path: String, equalTo bytes: UnsafePointer<CChar>, length: Int) -> Bool {
    guard let representation = fileSystemBytes(path) else { return false }
    return representation.withUnsafeBufferPointer {
        guard let start = $0.baseAddress else { return false }
        return strlen(start) == length && memcmp(start, bytes, length) == 0
    }
}

@_cdecl("appkit_window_ops_represented_filename")
public func appkit_window_ops_represented_filename(_ handle: UnsafeMutableRawPointer) -> UnsafeMutablePointer<CChar>? {
    let window: NSWindow = appkitObject(handle)
    guard !window.representedFilename.isEmpty else { return nil }
    guard let bytes = fileSystemBytes(window.representedFilename) else { return nil }
    return bytes.withUnsafeBufferPointer { strdup($0.baseAddress!) }
}

@_cdecl("appkit_window_ops_set_represented_filename")
public func appkit_window_ops_set_represented_filename(
    _ handle: UnsafeMutableRawPointer,
    _ bytes: UnsafePointer<CChar>?,
    _ length: Int
) -> Bool {
    let window: NSWindow = appkitObject(handle)
    guard length >= 0 else { return false }
    if let bytes {
        guard String(validatingUTF8: bytes) != nil else { return false }
        let filename = FileManager.default.string(
            withFileSystemRepresentation: bytes,
            length: length
        )
        guard hasFileSystemBytes(filename, equalTo: bytes, length: length) else {
            return false
        }
        let previous = window.representedFilename
        window.representedFilename = filename
        guard hasFileSystemBytes(window.representedFilename, equalTo: bytes, length: length) else {
            window.representedFilename = previous
            return false
        }
    } else {
        guard length == 0 else { return false }
        window.representedFilename = ""
    }
    return true
}

@_cdecl("appkit_window_ops_order_out")
public func appkit_window_ops_order_out(_ handle: UnsafeMutableRawPointer) {
    let window: NSWindow = appkitObject(handle)
    window.orderOut(nil)
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
) {
    let window: NSWindow = appkitObject(handle)
    guard let mode = NSWindow.OrderingMode(rawValue: Int(ordered)) else { return }
    window.order(mode, relativeTo: Int(relativeTo))
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
