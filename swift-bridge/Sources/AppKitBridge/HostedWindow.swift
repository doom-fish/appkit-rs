import AppKit
import AppKitCallbackABI
import AppKitExceptionShim

final class HostedWindowDelegate: NSObject, NSWindowDelegate {
    let token: UInt64
    private let table: AppKitHostWindowCallbackTableV1
    private var active = true
    private var registered = true
    private weak var window: NSWindow?

    init?(token: UInt64, table: AppKitHostWindowCallbackTableV1) {
        guard Thread.isMainThread,
              token != 0,
              table.header.version == 1,
              table.header.size >= MemoryLayout<AppKitHostWindowCallbackTableV1>.size,
              table.deactivate != nil,
              table.should_close != nil,
              table.event != nil else {
            return nil
        }
        self.token = token
        self.table = table
    }

    deinit {
        deactivate()
    }

    func deactivate() {
        guard registered else { return }
        registered = false
        active = false
        table.deactivate?(token)
    }

    func suspend() {
        active = false
    }

    func attach(to window: NSWindow) {
        self.window = window
    }

    func resumeIfShown() {
        guard registered, !active, let window,
              !((window as? HostedWindowOwner)?.isClosing ?? false),
              window.isVisible || window.isKeyWindow else {
            return
        }
        active = true
    }

    var acceptsCallbacks: Bool {
        guard Thread.isMainThread, registered else { return false }
        resumeIfShown()
        return active
    }

    @discardableResult
    func event(_ kind: UInt32) -> Bool {
        guard acceptsCallbacks, let callback = table.event else { return false }
        withExtendedLifetime(self) {
            callback(token, kind)
        }
        return true
    }

    func windowShouldClose(_ sender: NSWindow) -> Bool {
        guard acceptsCallbacks else { return false }
        return withExtendedLifetime(self) {
            table.should_close?(token) ?? false
        }
    }

    func windowWillClose(_ notification: Notification) {
        event(UInt32(APPKIT_HOST_WINDOW_EVENT_WILL_CLOSE))
    }

    func windowDidResize(_ notification: Notification) {
        event(UInt32(APPKIT_HOST_WINDOW_EVENT_DID_RESIZE))
    }

    func windowDidMove(_ notification: Notification) {
        event(UInt32(APPKIT_HOST_WINDOW_EVENT_DID_MOVE))
    }

    func windowDidChangeScreen(_ notification: Notification) {
        event(UInt32(APPKIT_HOST_WINDOW_EVENT_DID_CHANGE_SCREEN))
    }

    func windowDidChangeBackingProperties(_ notification: Notification) {
        event(UInt32(APPKIT_HOST_WINDOW_EVENT_DID_CHANGE_BACKING_PROPERTIES))
    }

    func windowDidBecomeKey(_ notification: Notification) {
        event(UInt32(APPKIT_HOST_WINDOW_EVENT_DID_BECOME_KEY))
    }

    func windowDidResignKey(_ notification: Notification) {
        event(UInt32(APPKIT_HOST_WINDOW_EVENT_DID_RESIGN_KEY))
    }

    func windowDidChangeOcclusionState(_ notification: Notification) {
        event(UInt32(APPKIT_HOST_WINDOW_EVENT_DID_CHANGE_OCCLUSION_STATE))
    }

    func windowWillEnterFullScreen(_ notification: Notification) {
        event(UInt32(APPKIT_HOST_WINDOW_EVENT_WILL_ENTER_FULL_SCREEN))
    }

    func windowDidEnterFullScreen(_ notification: Notification) {
        event(UInt32(APPKIT_HOST_WINDOW_EVENT_DID_ENTER_FULL_SCREEN))
    }

    func windowDidFailToEnterFullScreen(_ window: NSWindow) {
        event(UInt32(APPKIT_HOST_WINDOW_EVENT_DID_FAIL_TO_ENTER_FULL_SCREEN))
    }

    func windowWillExitFullScreen(_ notification: Notification) {
        event(UInt32(APPKIT_HOST_WINDOW_EVENT_WILL_EXIT_FULL_SCREEN))
    }

    func windowDidExitFullScreen(_ notification: Notification) {
        event(UInt32(APPKIT_HOST_WINDOW_EVENT_DID_EXIT_FULL_SCREEN))
    }

    func windowDidFailToExitFullScreen(_ window: NSWindow) {
        event(UInt32(APPKIT_HOST_WINDOW_EVENT_DID_FAIL_TO_EXIT_FULL_SCREEN))
    }
}

protocol HostedWindowOwner: AnyObject {
    var callbackDelegate: HostedWindowDelegate? { get set }
    var allowsMainWindow: Bool { get set }
    var allowsKeyWindow: Bool { get set }
    var isClosing: Bool { get set }
}

extension HostedWindowOwner where Self: NSWindow {
    func attach(_ callback: HostedWindowDelegate, configuration: AppKitHostedWindowConfiguration) {
        allowsMainWindow = configuration.can_become_main
        allowsKeyWindow = configuration.can_become_key
        isReleasedWhenClosed = false
        callback.attach(to: self)
        callbackDelegate = callback
        delegate = callback
    }

    func detach() {
        callbackDelegate?.deactivate()
        let _ = appkit_try { delegate = nil }
        callbackDelegate = nil
    }

    func suspendAfterClose() {
        callbackDelegate?.suspend()
    }

    func resumeForOrder(_ place: NSWindow.OrderingMode) {
        guard place != .out else { return }
        callbackDelegate?.resumeIfShown()
    }

    func hostedClose(_ close: () -> Bool) {
        guard !isClosing else { return }
        isClosing = true
        withExtendedLifetime(self) {
            defer { isClosing = false }
            callbackDelegate?.event(UInt32(APPKIT_HOST_WINDOW_EVENT_CLOSE))
            if close() {
                suspendAfterClose()
            }
        }
    }

    func hostedAccessory(_ controller: NSTitlebarAccessoryViewController, add: () -> Bool) {
        guard add(), appkit_try({
            let accessory = controller.view
            accessory.isHidden = true
            var frame = accessory.frame
            frame.size.height = 0
            accessory.frame = frame
        }) == nil else {
            return
        }
        callbackDelegate?.event(UInt32(APPKIT_HOST_WINDOW_EVENT_ADDED_TITLEBAR_ACCESSORY))
    }
}

@objc(DFHostedWindow)
class HostedWindow: NSWindow, HostedWindowOwner {
    var callbackDelegate: HostedWindowDelegate?
    var allowsMainWindow = true
    var allowsKeyWindow = true
    var isClosing = false

    override var canBecomeMain: Bool { allowsMainWindow }
    override var canBecomeKey: Bool { allowsKeyWindow }

    override func close() {
        hostedClose { appkit_try { super.close() } == nil }
    }

    override func order(_ place: NSWindow.OrderingMode, relativeTo otherWin: Int) {
        guard appkit_try({ super.order(place, relativeTo: otherWin) }) == nil else { return }
        resumeForOrder(place)
    }

    override func orderFrontRegardless() {
        guard appkit_try({ super.orderFrontRegardless() }) == nil else { return }
        callbackDelegate?.resumeIfShown()
    }

    override func becomeKey() {
        guard appkit_try({ super.becomeKey() }) == nil else { return }
        callbackDelegate?.resumeIfShown()
    }

    override func addTitlebarAccessoryViewController(_ childViewController: NSTitlebarAccessoryViewController) {
        hostedAccessory(childViewController) {
            appkit_try { super.addTitlebarAccessoryViewController(childViewController) } == nil
        }
    }

    override func moveTabToNewWindow(_ sender: Any?) {
        guard appkit_try({ super.moveTabToNewWindow(sender) }) == nil else { return }
        callbackDelegate?.event(UInt32(APPKIT_HOST_WINDOW_EVENT_MOVE_TAB_TO_NEW_WINDOW))
    }

    override func mergeAllWindows(_ sender: Any?) {
        guard appkit_try({ super.mergeAllWindows(sender) }) == nil else { return }
        callbackDelegate?.event(UInt32(APPKIT_HOST_WINDOW_EVENT_MERGE_ALL_WINDOWS))
    }

    override func selectNextTab(_ sender: Any?) {
        callbackDelegate?.event(UInt32(APPKIT_HOST_WINDOW_EVENT_SELECT_NEXT_TAB))
    }

    override func selectPreviousTab(_ sender: Any?) {
        callbackDelegate?.event(UInt32(APPKIT_HOST_WINDOW_EVENT_SELECT_PREVIOUS_TAB))
    }

    override func toggleTabBar(_ sender: Any?) {
        guard appkit_try({ super.toggleTabBar(sender) }) == nil else { return }
        callbackDelegate?.event(UInt32(APPKIT_HOST_WINDOW_EVENT_TOGGLE_TAB_BAR))
    }
}

@objc(DFHostedPanel)
class HostedPanel: NSPanel, HostedWindowOwner {
    var callbackDelegate: HostedWindowDelegate?
    var allowsMainWindow = true
    var allowsKeyWindow = true
    var isClosing = false

    override var canBecomeMain: Bool { allowsMainWindow }
    override var canBecomeKey: Bool { allowsKeyWindow }

    override func close() {
        hostedClose { appkit_try { super.close() } == nil }
    }

    override func order(_ place: NSWindow.OrderingMode, relativeTo otherWin: Int) {
        guard appkit_try({ super.order(place, relativeTo: otherWin) }) == nil else { return }
        resumeForOrder(place)
    }

    override func orderFrontRegardless() {
        guard appkit_try({ super.orderFrontRegardless() }) == nil else { return }
        callbackDelegate?.resumeIfShown()
    }

    override func becomeKey() {
        guard appkit_try({ super.becomeKey() }) == nil else { return }
        callbackDelegate?.resumeIfShown()
    }

    override func addTitlebarAccessoryViewController(_ childViewController: NSTitlebarAccessoryViewController) {
        hostedAccessory(childViewController) {
            appkit_try { super.addTitlebarAccessoryViewController(childViewController) } == nil
        }
    }

    override func moveTabToNewWindow(_ sender: Any?) {
        guard appkit_try({ super.moveTabToNewWindow(sender) }) == nil else { return }
        callbackDelegate?.event(UInt32(APPKIT_HOST_WINDOW_EVENT_MOVE_TAB_TO_NEW_WINDOW))
    }

    override func mergeAllWindows(_ sender: Any?) {
        guard appkit_try({ super.mergeAllWindows(sender) }) == nil else { return }
        callbackDelegate?.event(UInt32(APPKIT_HOST_WINDOW_EVENT_MERGE_ALL_WINDOWS))
    }

    override func selectNextTab(_ sender: Any?) {
        callbackDelegate?.event(UInt32(APPKIT_HOST_WINDOW_EVENT_SELECT_NEXT_TAB))
    }

    override func selectPreviousTab(_ sender: Any?) {
        callbackDelegate?.event(UInt32(APPKIT_HOST_WINDOW_EVENT_SELECT_PREVIOUS_TAB))
    }

    override func toggleTabBar(_ sender: Any?) {
        guard appkit_try({ super.toggleTabBar(sender) }) == nil else { return }
        callbackDelegate?.event(UInt32(APPKIT_HOST_WINDOW_EVENT_TOGGLE_TAB_BAR))
    }
}

private func createHostedWindow<T: NSWindow & HostedWindowOwner>(
    _ configuration: AppKitHostedWindowConfiguration,
    token: UInt64,
    table: AppKitHostWindowCallbackTableV1,
    make: (NSRect, NSWindow.StyleMask, NSWindow.BackingStoreType, Bool, NSScreen?) -> T
) -> T? {
    guard Thread.isMainThread,
          configuration.x.isFinite,
          configuration.y.isFinite,
          configuration.width.isFinite,
          configuration.height.isFinite,
          configuration.width >= 0,
          configuration.height >= 0,
          configuration.backing_store >= 0,
          let callback = HostedWindowDelegate(token: token, table: table),
          let backing = NSWindow.BackingStoreType(rawValue: UInt(configuration.backing_store)) else {
        return nil
    }
    let style = NSWindow.StyleMask(rawValue: UInt(configuration.style_mask))
    guard !style.contains(.fullScreen) else { return nil }
    let screen: NSScreen? = configuration.screen.map { appkitObject($0) }
    var created: T?
    guard appkit_try({
        created = make(
            NSRect(x: configuration.x, y: configuration.y, width: configuration.width, height: configuration.height),
            style,
            backing,
            configuration.defer_creation,
            screen
        )
    }) == nil, let window = created else {
        return nil
    }
    guard appkit_try({ window.attach(callback, configuration: configuration) }) == nil else {
        window.detach()
        return nil
    }
    return window
}

@_cdecl("appkit_host_window_new")
public func appkit_host_window_new(
    _ configuration: UnsafePointer<AppKitHostedWindowConfiguration>?,
    _ token: UInt64,
    _ table: UnsafePointer<AppKitHostWindowCallbackTableV1>?
) -> UnsafeMutableRawPointer? {
    guard let configuration, let table else { return nil }
    let window: HostedWindow? = createHostedWindow(configuration.pointee, token: token, table: table.pointee) {
        HostedWindow(contentRect: $0, styleMask: $1, backing: $2, defer: $3, screen: $4)
    }
    return window.map { Unmanaged.passRetained($0).toOpaque() }
}

@_cdecl("appkit_host_panel_new")
public func appkit_host_panel_new(
    _ configuration: UnsafePointer<AppKitHostedWindowConfiguration>?,
    _ options: UnsafePointer<AppKitHostedPanelOptions>?,
    _ token: UInt64,
    _ table: UnsafePointer<AppKitHostWindowCallbackTableV1>?
) -> UnsafeMutableRawPointer? {
    guard let configuration, let options, let table else { return nil }
    let panel: HostedPanel? = createHostedWindow(configuration.pointee, token: token, table: table.pointee) {
        HostedPanel(contentRect: $0, styleMask: $1, backing: $2, defer: $3, screen: $4)
    }
    guard let panel else { return nil }
    let settings = options.pointee
    guard appkit_try({
        if settings.is_floating_panel >= 0 {
            panel.isFloatingPanel = settings.is_floating_panel != 0
        }
        if settings.becomes_key_only_if_needed >= 0 {
            panel.becomesKeyOnlyIfNeeded = settings.becomes_key_only_if_needed != 0
        }
        if settings.works_when_modal >= 0 {
            panel.worksWhenModal = settings.works_when_modal != 0
        }
        if settings.hides_on_deactivate >= 0 {
            panel.hidesOnDeactivate = settings.hides_on_deactivate != 0
        }
    }) == nil else {
        panel.detach()
        return nil
    }
    return Unmanaged.passRetained(panel).toOpaque()
}

@_cdecl("appkit_host_window_detach")
public func appkit_host_window_detach(_ handle: UnsafeMutableRawPointer) {
    guard Thread.isMainThread else { return }
    let window: NSWindow = appkitObject(handle)
    (window as? (NSWindow & HostedWindowOwner))?.detach()
}

@_cdecl("appkit_host_window_set_can_become_main")
public func appkit_host_window_set_can_become_main(_ handle: UnsafeMutableRawPointer, _ enabled: Bool) {
    guard Thread.isMainThread else { return }
    let window: NSWindow = appkitObject(handle)
    (window as? (NSWindow & HostedWindowOwner))?.allowsMainWindow = enabled
}

@_cdecl("appkit_host_window_set_can_become_key")
public func appkit_host_window_set_can_become_key(_ handle: UnsafeMutableRawPointer, _ enabled: Bool) {
    guard Thread.isMainThread else { return }
    let window: NSWindow = appkitObject(handle)
    (window as? (NSWindow & HostedWindowOwner))?.allowsKeyWindow = enabled
}

@_cdecl("appkit_host_window_can_become_main")
public func appkit_host_window_can_become_main(_ handle: UnsafeMutableRawPointer) -> Bool {
    guard Thread.isMainThread else { return false }
    let window: NSWindow = appkitObject(handle)
    return window.canBecomeMain
}

@_cdecl("appkit_host_window_can_become_key")
public func appkit_host_window_can_become_key(_ handle: UnsafeMutableRawPointer) -> Bool {
    guard Thread.isMainThread else { return false }
    let window: NSWindow = appkitObject(handle)
    return window.canBecomeKey
}

@_cdecl("appkit_host_window_verify_should_close")
public func appkit_host_window_verify_should_close(_ handle: UnsafeMutableRawPointer) -> Int8 {
    guard Thread.isMainThread else { return -1 }
    let window: NSWindow = appkitObject(handle)
    guard let callback = window.delegate as? HostedWindowDelegate, callback.acceptsCallbacks else { return -1 }
    return callback.windowShouldClose(window) ? 1 : 0
}

@_cdecl("appkit_host_window_verify_hidden_reshow")
public func appkit_host_window_verify_hidden_reshow(_ handle: UnsafeMutableRawPointer) -> Bool {
    guard Thread.isMainThread else { return false }
    let window: NSWindow = appkitObject(handle)
    guard !window.isVisible else { return false }
    let application = NSApplication.shared
    let wasActive = application.isActive
    let alpha = window.alphaValue
    let ignoresMouseEvents = window.ignoresMouseEvents
    defer {
        if window.isVisible {
            let _ = appkit_try { window.orderOut(nil) }
        }
        let _ = appkit_try { window.alphaValue = alpha }
        let _ = appkit_try { window.ignoresMouseEvents = ignoresMouseEvents }
    }
    guard appkit_try({
        window.alphaValue = 0
        window.ignoresMouseEvents = true
    }) == nil else {
        return false
    }
    guard appkit_try({ window.orderBack(nil) }) == nil else { return false }
    let shown = window.isVisible
    guard appkit_try({ window.close() }) == nil else { return false }
    return shown && !window.isVisible && application.isActive == wasActive
}

@_cdecl("appkit_host_window_verify_hidden_reshow_with_veto")
public func appkit_host_window_verify_hidden_reshow_with_veto(
    _ handle: UnsafeMutableRawPointer,
    _ path: Int32
) -> Bool {
    guard Thread.isMainThread else { return false }
    let window: NSWindow = appkitObject(handle)
    guard !window.isVisible else { return false }
    let application = NSApplication.shared
    let wasActive = application.isActive
    let alpha = window.alphaValue
    let ignoresMouseEvents = window.ignoresMouseEvents
    defer {
        if window.isVisible {
            let _ = appkit_try { window.orderOut(nil) }
        }
        let _ = appkit_try { window.alphaValue = alpha }
        let _ = appkit_try { window.ignoresMouseEvents = ignoresMouseEvents }
    }
    guard appkit_try({
        window.alphaValue = 0
        window.ignoresMouseEvents = true
    }) == nil else {
        return false
    }
    switch path {
    case 0:
        guard appkit_try({ window.orderFrontRegardless() }) == nil else { return false }
    case 1:
        var session: NSApplication.ModalSession?
        guard appkit_try({ session = application.beginModalSession(for: window) }) == nil,
              let session,
              appkit_try({ application.endModalSession(session) }) == nil else {
            return false
        }
    default:
        return false
    }
    let shownWithoutFocus = window.isVisible && !window.isKeyWindow && application.isActive == wasActive
    guard appkit_try({ window.performClose(nil) }) == nil else { return false }
    let vetoed = window.isVisible
    let callback = window.delegate as? HostedWindowDelegate
    callback?.windowDidResize(Notification(name: NSWindow.didResizeNotification, object: window))
    let notified = callback?.acceptsCallbacks ?? false
    guard appkit_try({ window.performClose(nil) }) == nil else { return false }
    return shownWithoutFocus && vetoed && notified && !window.isVisible && application.isActive == wasActive
}

@_cdecl("appkit_host_window_verify_event")
public func appkit_host_window_verify_event(_ handle: UnsafeMutableRawPointer, _ kind: UInt32) -> Bool {
    guard Thread.isMainThread else { return false }
    let window: NSWindow = appkitObject(handle)
    guard let callback = window.delegate as? HostedWindowDelegate, callback.acceptsCallbacks else { return false }
    let notification = Notification(name: NSWindow.didMoveNotification, object: window)
    switch kind {
    case UInt32(APPKIT_HOST_WINDOW_EVENT_WILL_CLOSE):
        callback.windowWillClose(notification)
    case UInt32(APPKIT_HOST_WINDOW_EVENT_DID_RESIZE):
        callback.windowDidResize(notification)
    case UInt32(APPKIT_HOST_WINDOW_EVENT_DID_MOVE):
        callback.windowDidMove(notification)
    case UInt32(APPKIT_HOST_WINDOW_EVENT_DID_CHANGE_SCREEN):
        callback.windowDidChangeScreen(notification)
    case UInt32(APPKIT_HOST_WINDOW_EVENT_DID_CHANGE_BACKING_PROPERTIES):
        callback.windowDidChangeBackingProperties(notification)
    case UInt32(APPKIT_HOST_WINDOW_EVENT_DID_BECOME_KEY):
        callback.windowDidBecomeKey(notification)
    case UInt32(APPKIT_HOST_WINDOW_EVENT_DID_RESIGN_KEY):
        callback.windowDidResignKey(notification)
    case UInt32(APPKIT_HOST_WINDOW_EVENT_DID_CHANGE_OCCLUSION_STATE):
        callback.windowDidChangeOcclusionState(notification)
    case UInt32(APPKIT_HOST_WINDOW_EVENT_WILL_ENTER_FULL_SCREEN):
        callback.windowWillEnterFullScreen(notification)
    case UInt32(APPKIT_HOST_WINDOW_EVENT_DID_ENTER_FULL_SCREEN):
        callback.windowDidEnterFullScreen(notification)
    case UInt32(APPKIT_HOST_WINDOW_EVENT_DID_FAIL_TO_ENTER_FULL_SCREEN):
        callback.windowDidFailToEnterFullScreen(window)
    case UInt32(APPKIT_HOST_WINDOW_EVENT_WILL_EXIT_FULL_SCREEN):
        callback.windowWillExitFullScreen(notification)
    case UInt32(APPKIT_HOST_WINDOW_EVENT_DID_EXIT_FULL_SCREEN):
        callback.windowDidExitFullScreen(notification)
    case UInt32(APPKIT_HOST_WINDOW_EVENT_DID_FAIL_TO_EXIT_FULL_SCREEN):
        callback.windowDidFailToExitFullScreen(window)
    case UInt32(APPKIT_HOST_WINDOW_EVENT_CLOSE),
         UInt32(APPKIT_HOST_WINDOW_EVENT_ADDED_TITLEBAR_ACCESSORY),
         UInt32(APPKIT_HOST_WINDOW_EVENT_MOVE_TAB_TO_NEW_WINDOW),
         UInt32(APPKIT_HOST_WINDOW_EVENT_MERGE_ALL_WINDOWS),
         UInt32(APPKIT_HOST_WINDOW_EVENT_SELECT_NEXT_TAB),
         UInt32(APPKIT_HOST_WINDOW_EVENT_SELECT_PREVIOUS_TAB),
         UInt32(APPKIT_HOST_WINDOW_EVENT_TOGGLE_TAB_BAR):
        return callback.event(kind)
    default:
        return false
    }
    return true
}
