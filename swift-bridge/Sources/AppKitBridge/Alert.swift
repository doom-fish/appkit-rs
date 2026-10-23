import AppKit
import AppKitCallbackABI
import AppKitExceptionShim

private var alertOwners: [UInt64: AppKitAlertSheetOwner] = [:]

private final class AppKitOffscreenTestWindow: NSWindow {
    private func offscreen(_ frame: NSRect) -> NSRect {
        var frame = frame
        frame.origin = NSPoint(x: -20000, y: -20000)
        return frame
    }

    override func constrainFrameRect(_ frameRect: NSRect, to screen: NSScreen?) -> NSRect {
        offscreen(frameRect)
    }

    override func setFrame(_ frameRect: NSRect, display flag: Bool) {
        super.setFrame(offscreen(frameRect), display: flag)
    }

    override func setFrame(_ frameRect: NSRect, display flag: Bool, animate animateFlag: Bool) {
        super.setFrame(offscreen(frameRect), display: flag, animate: animateFlag)
    }

    override func setFrameOrigin(_ point: NSPoint) {
        super.setFrameOrigin(NSPoint(x: -20000, y: -20000))
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
        window?.alphaValue = 0
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
    private var responseQueued = false

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
                _ = self?.abort()
            }
        }) == nil, let observer else { return false }
        closeObserver = observer
        alertOwners[callback.token] = self
        return true
    }

    func begin() -> Bool {
        guard let parent, observeParent() else { return false }
        guard appkit_try({
            if parent is AppKitOffscreenTestWindow {
                alert.window.alphaValue = 0
            }
            alert.beginSheetModal(for: parent) { [weak self] response in
                guard let self else { return }
                let result = Int64(response.rawValue)
                if Thread.isMainThread {
                    self.responseQueued = true
                    DispatchQueue.main.async { [self] in self.complete(result) }
                } else {
                    DispatchQueue.main.async { [self] in
                        self.responseQueued = true
                        self.complete(result)
                    }
                }
            }
        }) == nil else {
            removeObserver()
            alertOwners.removeValue(forKey: callback.token)
            return false
        }
        DispatchQueue.main.async { [weak self] in self?.abortIfOrphaned() }
        return true
    }

    private func abortIfOrphaned() {
        guard !isEnding, !responseQueued else { return }
        guard let parent else {
            _ = abort()
            return
        }
        var attached = false
        var hidden = false
        guard appkit_try({
            attached = parent.attachedSheet === alert.window
            hidden = !parent.isVisible && !parent.isMiniaturized
        }) == nil else {
            _ = abort()
            return
        }
        if attached && hidden {
            _ = abort()
        }
    }

    func abort() -> Bool {
        guard Thread.isMainThread else {
            DispatchQueue.main.async { [self] in _ = abort() }
            return true
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

@_cdecl("appkit_alert_begin_sheet")
public func appkit_alert_begin_sheet(
    _ handle: UnsafeMutableRawPointer,
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

@_cdecl("appkit_alert_prepare_offscreen_parent")
public func appkit_alert_prepare_offscreen_parent(
    _ window: UnsafeMutableRawPointer,
    _ companion: UnsafeMutableRawPointer?,
    _ kind: Int32
) -> Bool {
    guard Thread.isMainThread else { return false }
    let parent: NSWindow = appkitObject(window)
    guard parent is AppKitOffscreenTestWindow, isOffscreen(parent) else { return false }
    switch kind {
    case 0:
        return !parent.isVisible
    case 1:
        return appkit_try({ parent.close() }) == nil && !parent.isVisible
    case 2:
        return appkit_try({ parent.orderFront(nil); parent.orderOut(nil) }) == nil
            && !parent.isVisible && isOffscreen(parent)
    case 3:
        return appkit_try({ parent.orderFront(nil); NSApplication.shared.hide(nil) }) == nil
            && NSApplication.shared.isHidden && !parent.isVisible && isOffscreen(parent)
    case 4:
        guard let companion else { return false }
        let second: NSWindow = appkitObject(companion)
        guard second is AppKitOffscreenTestWindow, isOffscreen(second) else { return false }
        let failure = appkit_try {
            parent.tabbingMode = .preferred
            second.tabbingMode = .preferred
            parent.tabbingIdentifier = "appkit-rs.alert-offscreen"
            second.tabbingIdentifier = parent.tabbingIdentifier
            parent.orderFront(nil)
            second.orderFront(nil)
            parent.addTabbedWindow(second, ordered: .above)
        }
        guard failure == nil, isOffscreen(parent), isOffscreen(second),
              let group = parent.tabGroup, group.windows.count == 2 else { return false }
        return group.selectedWindow === second
    default:
        return false
    }
}

@_cdecl("appkit_alert_finish_offscreen_sheet")
public func appkit_alert_finish_offscreen_sheet(_ window: UnsafeMutableRawPointer, _ buttonIndex: Int64) -> Bool {
    guard Thread.isMainThread else { return false }
    let parent: NSWindow = appkitObject(window)
    guard parent is AppKitOffscreenTestWindow, isOffscreen(parent) else { return false }
    var sheet: NSWindow?
    for _ in 0..<3 {
        guard appkit_try({ sheet = parent.attachedSheet }) == nil else { return false }
        if sheet != nil { break }
        guard appkit_try({ RunLoop.main.run(until: Date(timeIntervalSinceNow: 0.02)) }) == nil else { return false }
    }
    guard let sheet else { return false }
    guard isOffscreen(sheet) else {
        let _ = appkit_try { parent.endSheet(sheet, returnCode: .abort) }
        return false
    }
    return appkit_try({
        parent.endSheet(sheet, returnCode: NSApplication.ModalResponse(rawValue: Int(buttonIndex)))
    }) == nil
}

@_cdecl("appkit_alert_run_test_main_queue")
public func appkit_alert_run_test_main_queue() -> Bool {
    guard Thread.isMainThread else { return false }
    return appkit_try({ RunLoop.main.run(until: Date(timeIntervalSinceNow: 0.05)) }) == nil
}

@_cdecl("appkit_alert_close_offscreen_test_window")
public func appkit_alert_close_offscreen_test_window(_ window: UnsafeMutableRawPointer) -> Bool {
    guard Thread.isMainThread else { return false }
    let parent: NSWindow = appkitObject(window)
    guard parent is AppKitOffscreenTestWindow, isOffscreen(parent) else { return false }
    let closed = appkit_try { parent.close() } == nil
    let restored = appkit_try {
        if NSApplication.shared.isHidden { NSApplication.shared.unhideWithoutActivation() }
    } == nil
    return closed && restored
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
    guard parent is AppKitOffscreenTestWindow, isOffscreen(parent) else { return false }
    var shown = false
    guard appkit_try({
        parent.orderFront(nil)
        shown = parent.isVisible
    }) == nil, shown, isOffscreen(parent) else {
        let _ = appkit_try { parent.orderOut(nil) }
        return false
    }
    var alert: NSAlert?
    guard appkit_try({ alert = NSAlert() }) == nil, let alert else { return false }
    let operation = AppKitAlertSheetOwner(alert: alert, parent: parent, callback: callback, callbacks: callbacks)
    guard operation.begin() else { return false }
    guard appkit_try({ parent.close() }) == nil else {
        _ = operation.abort()
        return false
    }
    let deferred = alertOwners[token] != nil
    guard appkit_try({ RunLoop.main.run(until: Date(timeIntervalSinceNow: 0.03)) }) == nil else { return false }
    return deferred && alertOwners[token] == nil
}

@_cdecl("appkit_alert_check_will_close_start_sheet")
public func appkit_alert_check_will_close_start_sheet(
    _ window: UnsafeMutableRawPointer,
    _ token: UInt64,
    _ callbacks: UnsafePointer<AppKitAlertCallbacksV1>?
) -> Bool {
    guard Thread.isMainThread,
          let callbacks = callbacks?.pointee,
          callbacks.header.version == 1,
          callbacks.header.size >= MemoryLayout<AppKitAlertCallbacksV1>.size,
          callbacks.deactivate != nil,
          callbacks.complete != nil else { return false }
    let parent: NSWindow = appkitObject(window)
    guard parent is AppKitOffscreenTestWindow, isOffscreen(parent),
          appkit_try({ parent.orderFront(nil) }) == nil,
          parent.isVisible, isOffscreen(parent) else { return false }
    var alert: NSAlert?
    guard appkit_try({ alert = NSAlert() }) == nil, let alert else { return false }
    var started = false
    var observer: NSObjectProtocol?
    guard appkit_try({
        observer = NotificationCenter.default.addObserver(
            forName: NSWindow.willCloseNotification,
            object: parent,
            queue: nil
        ) { _ in
            var table = callbacks
            started = withUnsafePointer(to: &table) { pointer in
                appkit_alert_begin_sheet(
                    Unmanaged.passUnretained(alert).toOpaque(),
                    Unmanaged.passUnretained(parent).toOpaque(),
                    token,
                    pointer
                )
            }
        }
    }) == nil, let observer else { return false }
    let closed = appkit_try { parent.close() } == nil
    let _ = appkit_try { NotificationCenter.default.removeObserver(observer) }
    guard closed else {
        if started { _ = appkit_alert_cancel(token) }
        return false
    }
    let deferred = alertOwners[token] != nil
    guard appkit_try({ RunLoop.main.run(until: Date(timeIntervalSinceNow: 0.05)) }) == nil else { return false }
    return started && deferred && alertOwners[token] == nil
}
