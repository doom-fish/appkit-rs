import AppKit
import AppKitCallbackABI

@_cdecl("appkit_alert_new")
public func appkit_alert_new() -> UnsafeMutableRawPointer {
    Unmanaged.passRetained(NSAlert()).toOpaque()
}

@_cdecl("appkit_alert_set_style")
public func appkit_alert_set_style(_ handle: UnsafeMutableRawPointer, _ style: Int32) {
    let alert: NSAlert = appkitObject(handle)
    alert.alertStyle = UInt(exactly: style).flatMap(NSAlert.Style.init(rawValue:)) ?? .informational
}

@_cdecl("appkit_alert_set_message")
public func appkit_alert_set_message(_ handle: UnsafeMutableRawPointer, _ message: UnsafePointer<CChar>) {
    let alert: NSAlert = appkitObject(handle)
    alert.messageText = String(cString: message)
}

@_cdecl("appkit_alert_set_informative_text")
public func appkit_alert_set_informative_text(_ handle: UnsafeMutableRawPointer, _ text: UnsafePointer<CChar>?) {
    let alert: NSAlert = appkitObject(handle)
    alert.informativeText = appkitString(text) ?? ""
}

@_cdecl("appkit_alert_add_button")
public func appkit_alert_add_button(
    _ handle: UnsafeMutableRawPointer,
    _ title: UnsafePointer<CChar>,
    _ isCancel: Bool
) -> Int64 {
    let alert: NSAlert = appkitObject(handle)
    let button = alert.addButton(withTitle: String(cString: title))
    guard let index = alert.buttons.firstIndex(where: { $0 === button }) else { return -1 }
    button.tag = index
    if isCancel {
        button.keyEquivalent = "\u{1b}"
    } else if index == 0 {
        button.keyEquivalent = "\r"
    }
    return Int64(index)
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
          let complete = callbacks.complete,
          let owner = AppKitCompletionOwner(token: token, deactivate: deactivate) else {
        return false
    }

    let alert: NSAlert = appkitObject(handle)
    let parent: NSWindow = appkitObject(window)
    if let focus = alert.buttons.enumerated().reversed().first(where: {
        $0.offset > 0 && $0.element.keyEquivalent != "\u{1b}"
    }) {
        alert.window.initialFirstResponder = focus.element
    }

    alert.beginSheetModal(for: parent) { [alert, owner, weak parent, complete] response in
        let deliver = { [alert] in
            withExtendedLifetime(alert) {
                guard owner.finish() else { return }
                complete(token, parent == nil ? -1 : Int64(response.rawValue))
            }
        }
        if Thread.isMainThread {
            deliver()
        } else {
            DispatchQueue.main.async(execute: deliver)
        }
    }
    return true
}
