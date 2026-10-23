import AppKit
import AppKitCallbackABI

fileprivate enum AppKitHostMenuAction: Int32 {
    case other = -1
    case menuItem = 0
    case cut = 1
    case copy = 2
    case paste = 3
    case selectAll = 4
    case undo = 5
    case redo = 6
}

final class AppKitApplicationDelegate: NSResponder, NSApplicationDelegate, NSMenuDelegate, NSMenuItemValidation {
    let token: UInt64
    private let table: AppKitApplicationHostCallbacksV1
    private var active = true

    init?(token: UInt64, table: AppKitApplicationHostCallbacksV1) {
        guard Thread.isMainThread,
              token != 0,
              table.header.version == 1,
              table.header.size >= MemoryLayout<AppKitApplicationHostCallbacksV1>.size,
              table.deactivate != nil else {
            return nil
        }
        self.token = token
        self.table = table
        super.init()
    }

    required init?(coder: NSCoder) {
        return nil
    }

    deinit {
        disconnect()
    }

    func disconnect() {
        guard active else { return }
        active = false
        NotificationCenter.default.removeObserver(
            self,
            name: NSApplication.didChangeScreenParametersNotification,
            object: nil
        )
        table.deactivate(token)
    }

    private func deliver<Result>(_ fallback: Result, _ callback: () -> Result) -> Result {
        guard active, Thread.isMainThread else { return fallback }
        return withExtendedLifetime(self, callback)
    }

    func applicationWillFinishLaunching(_ notification: Notification) {
        deliver(()) {
            _ = table.will_finish_launching?(token)
        }
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        deliver(()) {
            _ = table.did_finish_launching?(token)
        }
    }

    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows: Bool) -> Bool {
        deliver(false) {
            table.should_handle_reopen?(token, hasVisibleWindows) ?? false
        }
    }

    func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
        let shouldTerminate = deliver(true) {
            table.should_terminate?(token) ?? true
        }
        return shouldTerminate ? .terminateNow : .terminateCancel
    }

    func applicationWillTerminate(_ notification: Notification) {
        deliver(()) {
            _ = table.will_terminate?(token)
        }
    }

    func application(_ application: NSApplication, open urls: [URL]) {
        deliver(()) {
            guard let callback = table.open_urls else { return }
            var bytes: [UInt8] = []
            var ranges: [AppKitApplicationHostUtf8Range] = []
            for url in urls {
                let value = Array(url.absoluteString.utf8)
                ranges.append(AppKitApplicationHostUtf8Range(offset: bytes.count, length: value.count))
                bytes.append(contentsOf: value)
            }
            bytes.withUnsafeBufferPointer { buffer in
                ranges.withUnsafeBufferPointer { offsets in
                    callback(token, buffer.baseAddress, buffer.count, offsets.baseAddress, offsets.count)
                }
            }
        }
    }

    func applicationDockMenu(_ sender: NSApplication) -> NSMenu? {
        deliver(nil as NSMenu?) {
            guard let retainedMenu = table.dock_menu_retained?(token) else { return nil }
            return Unmanaged<NSMenu>.fromOpaque(UnsafeMutableRawPointer(retainedMenu)).takeRetainedValue()
        }
    }

    @objc(handleAppKitMenuItem:)
    func handleAppKitMenuItem(_ sender: Any?) {
        sendMenuItemAction(.menuItem, sender: sender)
    }

    @objc(cut:)
    func cut(_ sender: Any?) {
        sendMenuItemAction(.cut, sender: sender)
    }

    @objc(copy:)
    func copy(_ sender: Any?) {
        sendMenuItemAction(.copy, sender: sender)
    }

    @objc(paste:)
    func paste(_ sender: Any?) {
        sendMenuItemAction(.paste, sender: sender)
    }

    @objc(selectAll:)
    override func selectAll(_ sender: Any?) {
        sendMenuItemAction(.selectAll, sender: sender)
    }

    @objc(undo:)
    func undo(_ sender: Any?) {
        sendMenuItemAction(.undo, sender: sender)
    }

    @objc(redo:)
    func redo(_ sender: Any?) {
        sendMenuItemAction(.redo, sender: sender)
    }

    private func sendMenuItemAction(_ action: AppKitHostMenuAction, sender: Any?) {
        guard let item = sender as? NSMenuItem else { return }
        deliver(()) {
            _ = table.menu_item_action?(token, Int64(item.tag), action.rawValue)
        }
    }

    func validateMenuItem(_ menuItem: NSMenuItem) -> Bool {
        deliver(false) {
            table.validate_menu_item?(token, Int64(menuItem.tag), menuAction(for: menuItem.action).rawValue) ?? false
        }
    }

    func menuWillOpen(_ menu: NSMenu) {
        deliver(()) {
            _ = table.menu_will_open?(token)
        }
    }

    @objc
    func screenParametersChanged(_ notification: Notification) {
        deliver(()) {
            _ = table.screen_parameters_changed?(token)
        }
    }

    private func menuAction(for selector: Selector?) -> AppKitHostMenuAction {
        switch selector {
        case #selector(handleAppKitMenuItem(_:)): .menuItem
        case #selector(cut(_:)): .cut
        case #selector(copy(_:)): .copy
        case #selector(paste(_:)): .paste
        case #selector(selectAll(_:)): .selectAll
        case #selector(undo(_:)): .undo
        case #selector(redo(_:)): .redo
        default: .other
        }
    }

    fileprivate func selector(for action: AppKitHostMenuAction) -> Selector {
        switch action {
        case .menuItem: #selector(handleAppKitMenuItem(_:))
        case .cut: #selector(cut(_:))
        case .copy: #selector(copy(_:))
        case .paste: #selector(paste(_:))
        case .selectAll: #selector(selectAll(_:))
        case .undo: #selector(undo(_:))
        case .redo: #selector(redo(_:))
        case .other: NSSelectorFromString("unsupportedAction:")
        }
    }

    fileprivate func invokeMenuItemAction(_ action: AppKitHostMenuAction, item: NSMenuItem) {
        switch action {
        case .menuItem: handleAppKitMenuItem(item)
        case .cut: cut(item)
        case .copy: copy(item)
        case .paste: paste(item)
        case .selectAll: selectAll(item)
        case .undo: undo(item)
        case .redo: redo(item)
        case .other: break
        }
    }
}

@_cdecl("appkit_application_host_install_v1")
public func appkit_application_host_install_v1(
    _ token: UInt64,
    _ table: UnsafePointer<AppKitApplicationHostCallbacksV1>?
) -> UnsafeMutableRawPointer? {
    guard let table, let delegate = AppKitApplicationDelegate(token: token, table: table.pointee) else {
        return nil
    }
    let application = NSApplication.shared
    if let previous = application.delegate as? AppKitApplicationDelegate {
        previous.disconnect()
    }
    application.delegate = delegate
    NotificationCenter.default.addObserver(
        delegate,
        selector: #selector(AppKitApplicationDelegate.screenParametersChanged(_:)),
        name: NSApplication.didChangeScreenParametersNotification,
        object: nil
    )
    return Unmanaged.passRetained(delegate).toOpaque()
}

@_cdecl("appkit_application_host_uninstall")
public func appkit_application_host_uninstall(_ handle: UnsafeMutableRawPointer) {
    guard Thread.isMainThread else { return }
    let delegate: AppKitApplicationDelegate = appkitObject(handle)
    delegate.disconnect()
    let application = NSApplication.shared
    if application.delegate === delegate {
        application.delegate = nil
    }
}

@_cdecl("appkit_application_host_probe")
public func appkit_application_host_probe(
    _ handle: UnsafeMutableRawPointer,
    _ method: Int32,
    _ action: Int32,
    _ tag: Int64,
    _ hasVisibleWindows: Bool
) -> Int64 {
    guard Thread.isMainThread else { return -1 }
    let delegate: AppKitApplicationDelegate = appkitObject(handle)
    let application = NSApplication.shared
    switch method {
    case 0:
        delegate.applicationWillFinishLaunching(Notification(name: NSApplication.willFinishLaunchingNotification))
    case 1:
        delegate.applicationDidFinishLaunching(Notification(name: NSApplication.didFinishLaunchingNotification))
    case 2:
        return delegate.applicationShouldHandleReopen(application, hasVisibleWindows: hasVisibleWindows) ? 1 : 0
    case 3:
        delegate.applicationWillTerminate(Notification(name: NSApplication.willTerminateNotification))
    case 4:
        guard let first = URL(string: "https://example.invalid/first"),
              let second = URL(string: "https://example.invalid/second%20item") else {
            return -1
        }
        delegate.application(application, open: [first, second])
    case 5:
        return delegate.applicationDockMenu(application) == nil ? 0 : 1
    case 6, 7:
        let menuAction = AppKitHostMenuAction(rawValue: action) ?? .other
        let item = NSMenuItem(title: "Probe", action: delegate.selector(for: menuAction), keyEquivalent: "")
        item.tag = Int(tag)
        if method == 6 {
            delegate.invokeMenuItemAction(menuAction, item: item)
        } else {
            return delegate.validateMenuItem(item) ? 1 : 0
        }
    case 8:
        delegate.menuWillOpen(NSMenu())
    case 9:
        NotificationCenter.default.post(name: NSApplication.didChangeScreenParametersNotification, object: nil)
    case 10:
        return NSStringFromSelector(#selector(AppKitApplicationDelegate.handleAppKitMenuItem(_:))) ==
            "handleAppKitMenuItem:" ? 1 : 0
    case 11:
        return application.delegate === delegate ? 1 : 0
    case 12:
        return delegate.applicationShouldTerminate(application) == .terminateNow ? 1 : 0
    case 13:
        delegate.screenParametersChanged(Notification(name: NSApplication.didChangeScreenParametersNotification))
    case 14:
        let selectors = [
            "applicationWillFinishLaunching:",
            "applicationDidFinishLaunching:",
            "applicationShouldHandleReopen:hasVisibleWindows:",
            "applicationShouldTerminate:",
            "applicationWillTerminate:",
            "application:openURLs:",
            "applicationDockMenu:",
            "handleAppKitMenuItem:",
            "cut:",
            "copy:",
            "paste:",
            "selectAll:",
            "undo:",
            "redo:",
            "validateMenuItem:",
            "menuWillOpen:",
            "screenParametersChanged:",
        ]
        for (index, selector) in selectors.enumerated() {
            if !delegate.responds(to: NSSelectorFromString(selector)) {
                return Int64(index + 2)
            }
        }
        return 1
    default:
        return -1
    }
    return 0
}

@_cdecl("appkit_application_host_probe_retained_menu")
public func appkit_application_host_probe_retained_menu() -> UnsafeMutableRawPointer {
    Unmanaged.passRetained(NSMenu(title: "Probe Dock")).toOpaque()
}
