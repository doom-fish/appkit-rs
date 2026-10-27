import AppKit
import AppKitCallbackABI
import AppKitExceptionShim

@_cdecl("appkit_menu_application_dock_menu")
public func appkit_menu_application_dock_menu() -> UnsafeMutableRawPointer? {
    guard Thread.isMainThread else { return nil }
    let application = NSApplication.shared
    var menu: NSMenu?
    guard appkit_try({
        menu = application.delegate?.applicationDockMenu?(application)
    }) == nil else { return nil }
    return appkitRetained(menu)
}

@_cdecl("appkit_menu_perform_action_at")
public func appkit_menu_perform_action_at(
    _ handle: UnsafeMutableRawPointer,
    _ index: Int
) -> Bool {
    guard Thread.isMainThread else { return false }
    let menu: NSMenu = appkitObject(handle)
    guard index >= 0, index < menu.numberOfItems,
          let item = menu.item(at: index), !item.isSeparatorItem,
          item.isEnabled, item.action != nil else { return false }
    return appkit_try({ menu.performActionForItem(at: index) }) == nil
}

private let appkitMenuSelectors: [String?] = [
    nil,
    "handleAppKitMenuItem:",
    "cut:",
    "copy:",
    "paste:",
    "selectAll:",
    "undo:",
    "redo:",
    "performClose:",
    "performMiniaturize:",
    "performZoom:",
    "hide:",
    "hideOtherApplications:",
    "unhideAllApplications:",
    "terminate:",
    "toggleFullScreen:",
    "toggleTabBar:",
    "selectNextTab:",
    "selectPreviousTab:",
    "moveTabToNewWindow:",
    "mergeAllWindows:",
    "toggleTabOverview:",
    "arrangeInFront:",
    "orderFrontStandardAboutPanel:",
    "showHelp:",
]

private func appkitMenuString(_ bytes: UnsafePointer<UInt8>?, _ length: Int) -> String? {
    guard length >= 0, length == 0 || bytes != nil else { return nil }
    return String(bytes: UnsafeBufferPointer(start: bytes, count: length), encoding: .utf8)
}

private func appkitMenuWrite(_ text: String, _ out: UnsafeMutablePointer<AppKitMenuBytes>) -> Bool {
    let bytes = Array(text.utf8)
    guard let storage = malloc(max(1, bytes.count)) else { return false }
    bytes.withUnsafeBytes { source in
        if let base = source.baseAddress {
            storage.copyMemory(from: base, byteCount: source.count)
        }
    }
    out.pointee = AppKitMenuBytes(
        bytes: storage.assumingMemoryBound(to: UInt8.self),
        length: bytes.count
    )
    return true
}

private func appkitMenuSelector(_ action: Int32) -> Selector? {
    guard action >= 0, Int(action) < appkitMenuSelectors.count,
          let name = appkitMenuSelectors[Int(action)] else {
        return nil
    }
    return NSSelectorFromString(name)
}

private func appkitMenuWouldCycle(_ submenu: NSMenu, under menu: NSMenu?) -> Bool {
    var ancestors = Set<ObjectIdentifier>()
    var current = menu
    while let ancestor = current {
        if ancestor === submenu || !ancestors.insert(ObjectIdentifier(ancestor)).inserted {
            return true
        }
        current = ancestor.supermenu
    }
    return false
}

@_cdecl("appkit_menu_new")
public func appkit_menu_new(
    _ titleBytes: UnsafePointer<UInt8>?,
    _ titleLength: Int
) -> UnsafeMutableRawPointer? {
    guard Thread.isMainThread, let title = appkitMenuString(titleBytes, titleLength) else {
        return nil
    }
    var menu: NSMenu?
    guard appkit_try({ menu = NSMenu(title: title) }) == nil else { return nil }
    return appkitRetained(menu)
}

@_cdecl("appkit_menu_title")
public func appkit_menu_title(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<AppKitMenuBytes>
) -> Bool {
    guard Thread.isMainThread else { return false }
    let menu: NSMenu = appkitObject(handle)
    var title: String?
    guard appkit_try({ title = menu.title }) == nil, let title else { return false }
    return appkitMenuWrite(title, out)
}

@_cdecl("appkit_menu_set_title")
public func appkit_menu_set_title(
    _ handle: UnsafeMutableRawPointer,
    _ titleBytes: UnsafePointer<UInt8>?,
    _ titleLength: Int
) -> Bool {
    guard Thread.isMainThread, let title = appkitMenuString(titleBytes, titleLength) else {
        return false
    }
    let menu: NSMenu = appkitObject(handle)
    return appkit_try({ menu.title = title }) == nil
}

@_cdecl("appkit_menu_item_count")
public func appkit_menu_item_count(_ handle: UnsafeMutableRawPointer) -> Int {
    guard Thread.isMainThread else { return 0 }
    let menu: NSMenu = appkitObject(handle)
    var count = 0
    guard appkit_try({ count = menu.numberOfItems }) == nil else { return 0 }
    return count
}

@_cdecl("appkit_menu_item_at")
public func appkit_menu_item_at(
    _ handle: UnsafeMutableRawPointer,
    _ index: Int
) -> UnsafeMutableRawPointer? {
    guard Thread.isMainThread, index >= 0 else { return nil }
    let menu: NSMenu = appkitObject(handle)
    var item: NSMenuItem?
    guard appkit_try({
        if index < menu.numberOfItems {
            item = menu.item(at: index)
        }
    }) == nil else {
        return nil
    }
    return appkitRetained(item)
}

@_cdecl("appkit_menu_add_item")
public func appkit_menu_add_item(
    _ handle: UnsafeMutableRawPointer,
    _ itemHandle: UnsafeMutableRawPointer
) -> Bool {
    guard Thread.isMainThread else { return false }
    let menu: NSMenu = appkitObject(handle)
    let item: NSMenuItem = appkitObject(itemHandle)
    var added = false
    guard appkit_try({
        if let submenu = item.submenu, appkitMenuWouldCycle(submenu, under: menu) {
            return
        }
        menu.addItem(item)
        added = true
    }) == nil else {
        return false
    }
    return added
}

@_cdecl("appkit_menu_insert_item")
public func appkit_menu_insert_item(
    _ handle: UnsafeMutableRawPointer,
    _ itemHandle: UnsafeMutableRawPointer,
    _ index: Int
) -> Bool {
    guard Thread.isMainThread, index >= 0 else { return false }
    let menu: NSMenu = appkitObject(handle)
    let item: NSMenuItem = appkitObject(itemHandle)
    var inserted = false
    guard appkit_try({
        guard index <= menu.numberOfItems else { return }
        if let submenu = item.submenu, appkitMenuWouldCycle(submenu, under: menu) {
            return
        }
        menu.insertItem(item, at: index)
        inserted = true
    }) == nil else {
        return false
    }
    return inserted
}

@_cdecl("appkit_menu_remove_item_at")
public func appkit_menu_remove_item_at(
    _ handle: UnsafeMutableRawPointer,
    _ index: Int
) -> Bool {
    guard Thread.isMainThread, index >= 0 else { return false }
    let menu: NSMenu = appkitObject(handle)
    var removed = false
    guard appkit_try({
        if index < menu.numberOfItems {
            menu.removeItem(at: index)
            removed = true
        }
    }) == nil else {
        return false
    }
    return removed
}

@_cdecl("appkit_menu_autoenables_items")
public func appkit_menu_autoenables_items(_ handle: UnsafeMutableRawPointer) -> Bool {
    guard Thread.isMainThread else { return false }
    let menu: NSMenu = appkitObject(handle)
    var autoenables = false
    guard appkit_try({ autoenables = menu.autoenablesItems }) == nil else { return false }
    return autoenables
}

@_cdecl("appkit_menu_set_autoenables_items")
public func appkit_menu_set_autoenables_items(
    _ handle: UnsafeMutableRawPointer,
    _ autoenables: Bool
) -> Bool {
    guard Thread.isMainThread else { return false }
    let menu: NSMenu = appkitObject(handle)
    return appkit_try({ menu.autoenablesItems = autoenables }) == nil
}

@_cdecl("appkit_menu_set_application_host")
public func appkit_menu_set_application_host(
    _ handle: UnsafeMutableRawPointer,
    _ delegateHandle: UnsafeMutableRawPointer?
) -> Bool {
    guard Thread.isMainThread else { return false }
    let menu: NSMenu = appkitObject(handle)
    let delegate: AppKitApplicationDelegate? = delegateHandle.map { appkitObject($0) }
    return appkit_try({ menu.delegate = delegate }) == nil
}

@_cdecl("appkit_menu_set_application_menu")
public func appkit_menu_set_application_menu(
    _ role: Int32,
    _ menuHandle: UnsafeMutableRawPointer?
) -> Bool {
    guard Thread.isMainThread, (0...3).contains(role) else { return false }
    let menu: NSMenu? = menuHandle.map { appkitObject($0) }
    return appkit_try({
        let application = NSApplication.shared
        switch role {
        case 0: application.mainMenu = menu
        case 1: application.servicesMenu = menu
        case 2: application.windowsMenu = menu
        case 3: application.helpMenu = menu
        default: break
        }
    }) == nil
}

@_cdecl("appkit_menu_application_menu")
public func appkit_menu_application_menu(_ role: Int32) -> UnsafeMutableRawPointer? {
    guard Thread.isMainThread, (0...3).contains(role) else { return nil }
    var menu: NSMenu?
    guard appkit_try({
        let application = NSApplication.shared
        switch role {
        case 0: menu = application.mainMenu
        case 1: menu = application.servicesMenu
        case 2: menu = application.windowsMenu
        case 3: menu = application.helpMenu
        default: break
        }
    }) == nil else {
        return nil
    }
    return appkitRetained(menu)
}

@_cdecl("appkit_menu_item_new")
public func appkit_menu_item_new(
    _ titleBytes: UnsafePointer<UInt8>?,
    _ titleLength: Int,
    _ action: Int32
) -> UnsafeMutableRawPointer? {
    guard Thread.isMainThread,
          action >= 0, Int(action) < appkitMenuSelectors.count,
          let title = appkitMenuString(titleBytes, titleLength) else {
        return nil
    }
    var item: NSMenuItem?
    guard appkit_try({
        item = NSMenuItem(
            title: title,
            action: appkitMenuSelector(action),
            keyEquivalent: ""
        )
    }) == nil else {
        return nil
    }
    return appkitRetained(item)
}

@_cdecl("appkit_menu_item_separator")
public func appkit_menu_item_separator() -> UnsafeMutableRawPointer? {
    guard Thread.isMainThread else { return nil }
    var item: NSMenuItem?
    guard appkit_try({ item = NSMenuItem.separator() }) == nil else { return nil }
    return appkitRetained(item)
}

@_cdecl("appkit_menu_item_title")
public func appkit_menu_item_title(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<AppKitMenuBytes>
) -> Bool {
    guard Thread.isMainThread else { return false }
    let item: NSMenuItem = appkitObject(handle)
    var title: String?
    guard appkit_try({ title = item.title }) == nil, let title else { return false }
    return appkitMenuWrite(title, out)
}

@_cdecl("appkit_menu_item_set_title")
public func appkit_menu_item_set_title(
    _ handle: UnsafeMutableRawPointer,
    _ titleBytes: UnsafePointer<UInt8>?,
    _ titleLength: Int
) -> Bool {
    guard Thread.isMainThread, let title = appkitMenuString(titleBytes, titleLength) else {
        return false
    }
    let item: NSMenuItem = appkitObject(handle)
    return appkit_try({ item.title = title }) == nil
}

@_cdecl("appkit_menu_item_is_separator")
public func appkit_menu_item_is_separator(_ handle: UnsafeMutableRawPointer) -> Bool {
    guard Thread.isMainThread else { return false }
    let item: NSMenuItem = appkitObject(handle)
    var separator = false
    guard appkit_try({ separator = item.isSeparatorItem }) == nil else { return false }
    return separator
}

@_cdecl("appkit_menu_item_submenu")
public func appkit_menu_item_submenu(
    _ handle: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    guard Thread.isMainThread else { return nil }
    let item: NSMenuItem = appkitObject(handle)
    var submenu: NSMenu?
    guard appkit_try({ submenu = item.submenu }) == nil else { return nil }
    return appkitRetained(submenu)
}

@_cdecl("appkit_menu_item_set_submenu")
public func appkit_menu_item_set_submenu(
    _ handle: UnsafeMutableRawPointer,
    _ menuHandle: UnsafeMutableRawPointer?
) -> Bool {
    guard Thread.isMainThread else { return false }
    let item: NSMenuItem = appkitObject(handle)
    let submenu: NSMenu? = menuHandle.map { appkitObject($0) }
    var assigned = false
    guard appkit_try({
        if let submenu, appkitMenuWouldCycle(submenu, under: item.menu) {
            return
        }
        item.submenu = submenu
        assigned = true
    }) == nil else {
        return false
    }
    return assigned
}

@_cdecl("appkit_menu_item_enabled")
public func appkit_menu_item_enabled(_ handle: UnsafeMutableRawPointer) -> Bool {
    guard Thread.isMainThread else { return false }
    let item: NSMenuItem = appkitObject(handle)
    var enabled = false
    guard appkit_try({ enabled = item.isEnabled }) == nil else { return false }
    return enabled
}

@_cdecl("appkit_menu_item_set_enabled")
public func appkit_menu_item_set_enabled(
    _ handle: UnsafeMutableRawPointer,
    _ enabled: Bool
) -> Bool {
    guard Thread.isMainThread else { return false }
    let item: NSMenuItem = appkitObject(handle)
    return appkit_try({ item.isEnabled = enabled }) == nil
}

@_cdecl("appkit_menu_item_state")
public func appkit_menu_item_state(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<Int32>
) -> Bool {
    guard Thread.isMainThread else { return false }
    let item: NSMenuItem = appkitObject(handle)
    var state: Int32?
    guard appkit_try({ state = Int32(exactly: item.state.rawValue) }) == nil,
          let state else {
        return false
    }
    out.pointee = state
    return true
}

@_cdecl("appkit_menu_item_set_state")
public func appkit_menu_item_set_state(
    _ handle: UnsafeMutableRawPointer,
    _ state: Int32
) -> Bool {
    guard Thread.isMainThread, (-1...1).contains(state) else { return false }
    let item: NSMenuItem = appkitObject(handle)
    return appkit_try({ item.state = NSControl.StateValue(rawValue: Int(state)) }) == nil
}

@_cdecl("appkit_menu_item_tag")
public func appkit_menu_item_tag(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<Int64>
) -> Bool {
    guard Thread.isMainThread else { return false }
    let item: NSMenuItem = appkitObject(handle)
    var tag = 0
    guard appkit_try({ tag = item.tag }) == nil else { return false }
    out.pointee = Int64(tag)
    return true
}

@_cdecl("appkit_menu_item_set_tag")
public func appkit_menu_item_set_tag(_ handle: UnsafeMutableRawPointer, _ tag: Int64) -> Bool {
    guard Thread.isMainThread else { return false }
    let item: NSMenuItem = appkitObject(handle)
    return appkit_try({ item.tag = Int(tag) }) == nil
}

@_cdecl("appkit_menu_item_represented_integer")
public func appkit_menu_item_represented_integer(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<Int64>
) -> Bool {
    guard Thread.isMainThread else { return false }
    let item: NSMenuItem = appkitObject(handle)
    var number: NSNumber?
    guard appkit_try({ number = item.representedObject as? NSNumber }) == nil,
          let number else {
        return false
    }
    var value: Int64 = 0
    guard appkit_try({ value = number.int64Value }) == nil else { return false }
    out.pointee = value
    return true
}

@_cdecl("appkit_menu_item_set_represented_integer")
public func appkit_menu_item_set_represented_integer(
    _ handle: UnsafeMutableRawPointer,
    _ present: Bool,
    _ value: Int64
) -> Bool {
    guard Thread.isMainThread else { return false }
    let item: NSMenuItem = appkitObject(handle)
    return appkit_try({ item.representedObject = present ? NSNumber(value: value) : nil }) == nil
}

@_cdecl("appkit_menu_item_key_equivalent")
public func appkit_menu_item_key_equivalent(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<AppKitMenuBytes>
) -> Bool {
    guard Thread.isMainThread else { return false }
    let item: NSMenuItem = appkitObject(handle)
    var key: String?
    guard appkit_try({ key = item.keyEquivalent }) == nil, let key else { return false }
    return appkitMenuWrite(key, out)
}

@_cdecl("appkit_menu_item_key_modifiers")
public func appkit_menu_item_key_modifiers(_ handle: UnsafeMutableRawPointer) -> UInt64 {
    guard Thread.isMainThread else { return 0 }
    let item: NSMenuItem = appkitObject(handle)
    var modifiers: UInt64 = 0
    guard appkit_try({ modifiers = UInt64(item.keyEquivalentModifierMask.rawValue) }) == nil else {
        return 0
    }
    return modifiers
}

@_cdecl("appkit_menu_item_set_key_equivalent")
public func appkit_menu_item_set_key_equivalent(
    _ handle: UnsafeMutableRawPointer,
    _ keyBytes: UnsafePointer<UInt8>?,
    _ keyLength: Int,
    _ present: Bool,
    _ modifiers: UInt64
) -> Bool {
    guard Thread.isMainThread else { return false }
    let key: String?
    if present {
        guard let value = appkitMenuString(keyBytes, keyLength) else { return false }
        key = value
    } else {
        key = nil
    }
    let item: NSMenuItem = appkitObject(handle)
    return appkit_try({
        item.keyEquivalent = key ?? ""
        item.keyEquivalentModifierMask = key == nil
            ? .command
            : NSEvent.ModifierFlags(rawValue: UInt(modifiers))
        item.allowsAutomaticKeyEquivalentLocalization = key == nil
    }) == nil
}

@_cdecl("appkit_menu_item_allows_automatic_key_equivalent_localization")
public func appkit_menu_item_allows_automatic_key_equivalent_localization(
    _ handle: UnsafeMutableRawPointer
) -> Bool {
    guard Thread.isMainThread else { return false }
    let item: NSMenuItem = appkitObject(handle)
    var allows = false
    guard appkit_try({ allows = item.allowsAutomaticKeyEquivalentLocalization }) == nil else {
        return false
    }
    return allows
}

@_cdecl("appkit_menu_item_action")
public func appkit_menu_item_action(_ handle: UnsafeMutableRawPointer) -> Int32 {
    guard Thread.isMainThread else { return -1 }
    let item: NSMenuItem = appkitObject(handle)
    var name: String?
    guard appkit_try({
        if let action = item.action {
            name = NSStringFromSelector(action)
        }
    }) == nil else {
        return -1
    }
    guard let name else { return 0 }
    guard let index = appkitMenuSelectors.firstIndex(where: { $0 == name }) else { return -1 }
    return Int32(index)
}

@_cdecl("appkit_menu_item_set_action")
public func appkit_menu_item_set_action(
    _ handle: UnsafeMutableRawPointer,
    _ action: Int32
) -> Bool {
    guard Thread.isMainThread, action >= 0, Int(action) < appkitMenuSelectors.count else {
        return false
    }
    let item: NSMenuItem = appkitObject(handle)
    var updated = false
    guard appkit_try({
        if !item.isSeparatorItem {
            item.action = appkitMenuSelector(action)
            item.target = nil
            updated = true
        }
    }) == nil else {
        return false
    }
    return updated
}

@_cdecl("appkit_menu_item_action_selector")
public func appkit_menu_item_action_selector(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<AppKitMenuBytes>
) -> Bool {
    guard Thread.isMainThread else { return false }
    let item: NSMenuItem = appkitObject(handle)
    var name: String?
    guard appkit_try({
        if let action = item.action {
            name = NSStringFromSelector(action)
        }
    }) == nil, let name else {
        return false
    }
    return appkitMenuWrite(name, out)
}

@_cdecl("appkit_menu_item_has_explicit_target")
public func appkit_menu_item_has_explicit_target(_ handle: UnsafeMutableRawPointer) -> Bool {
    guard Thread.isMainThread else { return false }
    let item: NSMenuItem = appkitObject(handle)
    var hasTarget = false
    guard appkit_try({ hasTarget = item.target != nil }) == nil else { return false }
    return hasTarget
}

@_cdecl("appkit_menu_bytes_free")
public func appkit_menu_bytes_free(_ bytes: UnsafeMutablePointer<UInt8>?) {
    free(bytes)
}

@_cdecl("appkit_menu_probe_window_responds_to_item")
public func appkit_menu_probe_window_responds_to_item(
    _ windowHandle: UnsafeMutableRawPointer,
    _ itemHandle: UnsafeMutableRawPointer
) -> Bool {
    guard Thread.isMainThread else { return false }
    let window: NSWindow = appkitObject(windowHandle)
    let item: NSMenuItem = appkitObject(itemHandle)
    var responds = false
    guard appkit_try({
        if let action = item.action {
            responds = window.responds(to: action)
        }
    }) == nil else {
        return false
    }
    return responds
}
