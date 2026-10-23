import AppKit
import AppKitCallbackABI
import AppKitExceptionShim

private var alertOwners: [UInt64: AppKitAlertSheetOwner] = [:]

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
            alert.beginSheetModal(for: parent) { [weak self] response in
                guard let self else { return }
                if Thread.isMainThread {
                    self.complete(Int64(response.rawValue))
                } else {
                    DispatchQueue.main.async { [self] in self.complete(Int64(response.rawValue)) }
                }
            }
        }) == nil else {
            removeObserver()
            alertOwners.removeValue(forKey: callback.token)
            return false
        }
        return true
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
            callbacks.complete?(callback.token, response)
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
    Unmanaged.passRetained(NSAlert()).toOpaque()
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
    guard Thread.isMainThread else { return false }
    let parent: NSWindow = appkitObject(window)
    var available = false
    guard appkit_try({ available = parent.isVisible || parent.isMiniaturized }) == nil else { return false }
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
    guard !appkit_alert_can_begin_sheet(window) else { return false }
    var alert: NSAlert?
    guard appkit_try({ alert = NSAlert() }) == nil, let alert else { return false }
    let operation = AppKitAlertSheetOwner(alert: alert, parent: parent, callback: callback, callbacks: callbacks)
    guard operation.observeParent() else { return false }
    guard appkit_try({ parent.close() }) == nil else {
        _ = operation.abort()
        return false
    }
    guard appkit_try({ RunLoop.main.run(until: Date(timeIntervalSinceNow: 0.03)) }) == nil else { return false }
    return alertOwners[token] == nil
}
