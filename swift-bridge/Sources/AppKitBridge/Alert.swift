import AppKit
import AppKitCallbackABI
import AppKitExceptionShim

private final class AppKitClosedWindowRegistry {
    private let closed = NSHashTable<NSWindow>.weakObjects()
    private let closing = NSHashTable<NSWindow>.weakObjects()
    private var closeObserver: NSObjectProtocol?
    private var exposeObserver: NSObjectProtocol?

    func ensureInstalled() -> Bool {
        if closeObserver != nil, exposeObserver != nil { return true }
        let center = NotificationCenter.default
        var closeToken: NSObjectProtocol?
        var exposeToken: NSObjectProtocol?
        let failure = appkit_try {
            closeToken = center.addObserver(forName: NSWindow.willCloseNotification, object: nil, queue: nil) {
                [weak self] notification in
                guard let window = notification.object as? NSWindow else { return }
                self?.markClosing(window)
            }
            exposeToken = center.addObserver(forName: NSWindow.didExposeNotification, object: nil, queue: nil) {
                [weak self] notification in
                guard let window = notification.object as? NSWindow else { return }
                self?.markShown(window)
            }
        }
        guard failure == nil, let closeToken, let exposeToken else {
            if let closeToken { let _ = appkit_try { center.removeObserver(closeToken) } }
            if let exposeToken { let _ = appkit_try { center.removeObserver(exposeToken) } }
            return false
        }
        closeObserver = closeToken
        exposeObserver = exposeToken
        return true
    }

    func markClosing(_ window: NSWindow) {
        let _ = appkit_try {
            closed.add(window)
            guard !closing.contains(window) else { return }
            closing.add(window)
            DispatchQueue.main.async { [weak self, weak window] in
                guard let self, let window else { return }
                let _ = appkit_try { self.closing.remove(window) }
            }
        }
    }

    func markShown(_ window: NSWindow) {
        let _ = appkit_try {
            guard !closing.contains(window), window.isVisible else { return }
            closed.remove(window)
        }
    }

    func canBegin(on window: NSWindow) -> Bool {
        guard !closing.contains(window) else { return false }
        guard closed.contains(window) else { return true }
        guard window.isVisible else { return false }
        closed.remove(window)
        return true
    }
}

private let closedWindows = AppKitClosedWindowRegistry()
private var alertOwners: [UInt64: AppKitAlertSheetOwner] = [:]

private final class AppKitOffscreenTestWindow: NSWindow {
    override func constrainFrameRect(_ frameRect: NSRect, to screen: NSScreen?) -> NSRect {
        frameRect
    }
}

private func offscreenTestWindow() -> AppKitOffscreenTestWindow? {
    var window: AppKitOffscreenTestWindow?
    guard appkit_try({
        window = AppKitOffscreenTestWindow(
            contentRect: NSRect(x: -20000, y: -20000, width: 80, height: 80),
            styleMask: [.titled, .closable, .resizable],
            backing: .buffered,
            defer: false
        )
        window?.isReleasedWhenClosed = false
        window?.setFrameOrigin(NSPoint(x: -20000, y: -20000))
    }) == nil else { return nil }
    guard let window, isOffscreen(window) else { return nil }
    return window
}

private func isOffscreen(_ window: NSWindow) -> Bool {
    var offscreen = false
    guard appkit_try({
        let frame = window.frame
        offscreen = NSScreen.screens.allSatisfy { !$0.frame.intersects(frame) }
    }) == nil else { return false }
    return offscreen
}

private final class AppKitAlertSheetOwner {
    let alert: NSAlert
    let callback: AppKitCompletionOwner
    let callbacks: AppKitAlertCallbacksV1
    private weak var parent: NSWindow?
    private var closeObserver: NSObjectProtocol?
    private var isEnding = false

    init(alert: NSAlert, parent: NSWindow, callback: AppKitCompletionOwner, callbacks: AppKitAlertCallbacksV1) {
        self.alert = alert
        self.parent = parent
        self.callback = callback
        self.callbacks = callbacks
    }

    deinit {
        removeObserver()
    }

    fileprivate func observeParent() -> Bool {
        guard let parent else { return false }
        var observer: NSObjectProtocol?
        guard appkit_try({
            observer = NotificationCenter.default.addObserver(
                forName: NSWindow.willCloseNotification,
                object: parent,
                queue: nil
            ) { [weak self] _ in
                _ = self?.abort(parentClosing: true)
            }
        }) == nil, let observer else { return false }
        closeObserver = observer
        alertOwners[callback.token] = self
        return true
    }

    func begin() -> Bool {
        guard let parent else { return false }
        var available = false
        guard appkit_try({ available = closedWindows.canBegin(on: parent) }) == nil,
              available, observeParent() else { return false }
        guard appkit_try({
            alert.beginSheetModal(for: parent) { [weak self] response in
                guard let self else { return }
                let result = Int64(response.rawValue)
                DispatchQueue.main.async { [self] in self.complete(result) }
            }
        }) == nil else {
            removeObserver()
            alertOwners.removeValue(forKey: callback.token)
            return false
        }
        return true
    }

    func abort(parentClosing: Bool = false) -> Bool {
        guard Thread.isMainThread else {
            DispatchQueue.main.async { [self] in _ = abort(parentClosing: parentClosing) }
            return true
        }
        if parentClosing, let parent {
            closedWindows.markClosing(parent)
        }
        guard !isEnding else { return false }
        isEnding = true
        if let parent {
            if appkit_try({ parent.endSheet(alert.window, returnCode: .abort) }) != nil {
                let _ = appkit_try { alert.window.close() }
            }
        }
        DispatchQueue.main.async { [self] in complete(Int64(NSApplication.ModalResponse.abort.rawValue)) }
        return true
    }

    func complete(_ response: Int64) {
        guard callback.finish() else { return }
        withExtendedLifetime(self) {
            removeObserver()
            alertOwners.removeValue(forKey: callback.token)
            let result = isEnding ? Int64(NSApplication.ModalResponse.abort.rawValue) : response
            callbacks.complete?(callback.token, result)
        }
    }

    private func removeObserver() {
        guard let observer = closeObserver else { return }
        closeObserver = nil
        let _ = appkit_try { NotificationCenter.default.removeObserver(observer) }
    }
}

@_cdecl("appkit_alert_new")
public func appkit_alert_new() -> UnsafeMutableRawPointer {
    _ = closedWindows.ensureInstalled()
    return Unmanaged.passRetained(NSAlert()).toOpaque()
}

@_cdecl("appkit_alert_set_style")
public func appkit_alert_set_style(_ handle: UnsafeMutableRawPointer, _ style: Int32) -> Bool {
    let alert: NSAlert = appkitObject(handle)
    guard let raw = UInt(exactly: style), let value = NSAlert.Style(rawValue: raw) else { return false }
    return appkit_try { alert.alertStyle = value } == nil
}

@_cdecl("appkit_alert_set_message")
public func appkit_alert_set_message(_ handle: UnsafeMutableRawPointer, _ message: UnsafePointer<CChar>) -> Bool {
    let alert: NSAlert = appkitObject(handle)
    return appkit_try { alert.messageText = String(cString: message) } == nil
}

@_cdecl("appkit_alert_set_informative_text")
public func appkit_alert_set_informative_text(_ handle: UnsafeMutableRawPointer, _ text: UnsafePointer<CChar>?) -> Bool {
    let alert: NSAlert = appkitObject(handle)
    return appkit_try { alert.informativeText = appkitString(text) ?? "" } == nil
}

@_cdecl("appkit_alert_add_button")
public func appkit_alert_add_button(
    _ handle: UnsafeMutableRawPointer,
    _ title: UnsafePointer<CChar>,
    _ isCancel: Bool
) -> Int64 {
    let alert: NSAlert = appkitObject(handle)
    var index: Int64 = -1
    guard appkit_try({
        let button = alert.addButton(withTitle: String(cString: title))
        guard let position = alert.buttons.firstIndex(where: { $0 === button }) else { return }
        button.tag = position
        if isCancel {
            button.keyEquivalent = "\u{1b}"
        } else if position == 0 {
            button.keyEquivalent = "\r"
        }
        index = Int64(position)
    }) == nil else { return -1 }
    return index
}

@_cdecl("appkit_alert_can_begin_sheet")
public func appkit_alert_can_begin_sheet(_ window: UnsafeMutableRawPointer) -> Bool {
    guard Thread.isMainThread, closedWindows.ensureInstalled() else { return false }
    let parent: NSWindow = appkitObject(window)
    var available = false
    guard appkit_try({ available = closedWindows.canBegin(on: parent) }) == nil else { return false }
    return available
}

@_cdecl("appkit_alert_begin_sheet")
public func appkit_alert_begin_sheet(
    _ handle: UnsafeMutableRawPointer,
    _ window: UnsafeMutableRawPointer,
    _ token: UInt64,
    _ callbacks: UnsafePointer<AppKitAlertCallbacksV1>?
) -> Bool {
    guard Thread.isMainThread,
          appkit_alert_can_begin_sheet(window),
          let callbacks = callbacks?.pointee,
          callbacks.header.version == 1,
          callbacks.header.size >= MemoryLayout<AppKitAlertCallbacksV1>.size,
          let deactivate = callbacks.deactivate,
          callbacks.complete != nil,
          let owner = AppKitCompletionOwner(token: token, deactivate: deactivate) else {
        return false
    }

    let alert: NSAlert = appkitObject(handle)
    let parent: NSWindow = appkitObject(window)
    guard appkit_try({
        if let focus = alert.buttons.enumerated().reversed().first(where: {
            $0.offset > 0 && $0.element.keyEquivalent != "\u{1b}"
        }) {
            alert.window.initialFirstResponder = focus.element
        }
    }) == nil else { return false }
    return AppKitAlertSheetOwner(alert: alert, parent: parent, callback: owner, callbacks: callbacks).begin()
}

@_cdecl("appkit_alert_cancel")
public func appkit_alert_cancel(_ token: UInt64) -> Bool {
    guard Thread.isMainThread, let operation = alertOwners[token] else { return false }
    return operation.abort()
}

@_cdecl("appkit_alert_new_offscreen_test_window")
public func appkit_alert_new_offscreen_test_window() -> UnsafeMutableRawPointer? {
    guard Thread.isMainThread else { return nil }
    return appkitRetained(offscreenTestWindow())
}

@_cdecl("appkit_alert_check_offscreen_states")
public func appkit_alert_check_offscreen_states() -> UInt32 {
    guard Thread.isMainThread, closedWindows.ensureInstalled(),
          let first = offscreenTestWindow(),
          let second = offscreenTestWindow(),
          let reopened = offscreenTestWindow() else { return 0 }
    let application = NSApplication.shared
    let wasHidden = application.isHidden
    var result: UInt32 = 0

    if appkit_try({ first.orderFront(nil) }) == nil,
       first.isVisible,
       isOffscreen(first),
       closedWindows.canBegin(on: first) {
        if appkit_try({ first.orderOut(nil) }) == nil,
           !first.isVisible,
           closedWindows.canBegin(on: first) {
            result |= 2
        }
        if appkit_try({ first.orderFront(nil) }) == nil,
           appkit_try({ application.hide(nil) }) == nil,
           application.isHidden,
           !first.isVisible,
           closedWindows.canBegin(on: first) {
            result |= 1
        }
        if !wasHidden {
            let _ = appkit_try { application.unhideWithoutActivation() }
        }
        let tabFailure = appkit_try {
            first.tabbingMode = .preferred
            second.tabbingMode = .preferred
            first.tabbingIdentifier = "appkit-rs.alert-offscreen"
            second.tabbingIdentifier = first.tabbingIdentifier
            second.orderFront(nil)
            first.addTabbedWindow(second, ordered: .above)
            RunLoop.main.run(until: Date(timeIntervalSinceNow: 0.03))
        }
        if tabFailure == nil, isOffscreen(first), isOffscreen(second),
           appkit_try({ first.orderOut(nil) }) == nil,
           let group = first.tabGroup, group.windows.count == 2,
           group.selectedWindow === second, !first.isVisible,
           closedWindows.canBegin(on: first) {
            result |= 4
        }
    }

    if appkit_try({ reopened.orderFront(nil) }) == nil, isOffscreen(reopened),
       appkit_try({ reopened.close() }) == nil, !closedWindows.canBegin(on: reopened),
       appkit_try({ RunLoop.main.run(until: Date(timeIntervalSinceNow: 0.05)) }) == nil,
       appkit_try({ reopened.orderFront(nil) }) == nil,
       reopened.isVisible, isOffscreen(reopened), closedWindows.canBegin(on: reopened) {
        result |= 8
    }

    let _ = appkit_try { first.close() }
    let _ = appkit_try { second.close() }
    let _ = appkit_try { reopened.close() }
    if !wasHidden, application.isHidden {
        let _ = appkit_try { application.unhideWithoutActivation() }
    }
    return result
}

@_cdecl("appkit_alert_check_close_observer")
public func appkit_alert_check_close_observer(
    _ window: UnsafeMutableRawPointer,
    _ token: UInt64,
    _ callbacks: UnsafePointer<AppKitAlertCallbacksV1>?
) -> Bool {
    guard Thread.isMainThread,
          let callbacks = callbacks?.pointee,
          callbacks.header.version == 1,
          callbacks.header.size >= MemoryLayout<AppKitAlertCallbacksV1>.size,
          let deactivate = callbacks.deactivate,
          callbacks.complete != nil,
          let callback = AppKitCompletionOwner(token: token, deactivate: deactivate) else {
        return false
    }
    let parent: NSWindow = appkitObject(window)
    guard appkit_try({
        parent.setFrameOrigin(NSPoint(x: -20000, y: -20000))
    }) == nil, isOffscreen(parent) else { return false }
    var shown = false
    guard appkit_try({
        parent.orderFront(nil)
        shown = parent.isVisible
    }) == nil, shown, isOffscreen(parent), appkit_alert_can_begin_sheet(window) else {
        let _ = appkit_try { parent.orderOut(nil) }
        return false
    }
    var alert: NSAlert?
    guard appkit_try({ alert = NSAlert() }) == nil, let alert else { return false }
    let operation = AppKitAlertSheetOwner(alert: alert, parent: parent, callback: callback, callbacks: callbacks)
    guard operation.observeParent() else { return false }
    guard appkit_try({ parent.close() }) == nil else {
        _ = operation.abort()
        return false
    }
    let deferred = alertOwners[token] != nil
    guard appkit_try({ RunLoop.main.run(until: Date(timeIntervalSinceNow: 0.03)) }) == nil else { return false }
    return deferred && alertOwners[token] == nil
}
