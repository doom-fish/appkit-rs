import AppKit
import AppKitCallbackABI

@_cdecl("appkit_cursor_system")
public func appkit_cursor_system(_ kind: Int32) -> UnsafeMutableRawPointer? {
    _ = NSApplication.shared
    let cursor: NSCursor
    switch kind {
    case Int32(AppKitCursorKindArrow.rawValue):
        cursor = .arrow
    case Int32(AppKitCursorKindIBeam.rawValue):
        cursor = .iBeam
    case Int32(AppKitCursorKindIBeamForVerticalLayout.rawValue):
        cursor = .iBeamCursorForVerticalLayout
    case Int32(AppKitCursorKindCrosshair.rawValue):
        cursor = .crosshair
    case Int32(AppKitCursorKindOpenHand.rawValue):
        cursor = .openHand
    case Int32(AppKitCursorKindClosedHand.rawValue):
        cursor = .closedHand
    case Int32(AppKitCursorKindPointingHand.rawValue):
        cursor = .pointingHand
    case Int32(AppKitCursorKindResizeLeftRight.rawValue),
         Int32(AppKitCursorKindResizeColumn.rawValue):
        cursor = .resizeLeftRight
    case Int32(AppKitCursorKindResizeUpDown.rawValue),
         Int32(AppKitCursorKindResizeRow.rawValue):
        cursor = .resizeUpDown
    case Int32(AppKitCursorKindResizeLeft.rawValue):
        cursor = .resizeLeft
    case Int32(AppKitCursorKindResizeRight.rawValue):
        cursor = .resizeRight
    case Int32(AppKitCursorKindResizeUp.rawValue):
        cursor = .resizeUp
    case Int32(AppKitCursorKindResizeDown.rawValue):
        cursor = .resizeDown
    case Int32(AppKitCursorKindResizeUpLeftDownRight.rawValue):
        if #available(macOS 15.0, *) {
            cursor = .frameResize(position: .topLeft, directions: .all)
        } else {
            cursor = .resizeLeftRight
        }
    case Int32(AppKitCursorKindResizeUpRightDownLeft.rawValue):
        if #available(macOS 15.0, *) {
            cursor = .frameResize(position: .topRight, directions: .all)
        } else {
            cursor = .resizeUpDown
        }
    case Int32(AppKitCursorKindOperationNotAllowed.rawValue):
        cursor = .operationNotAllowed
    case Int32(AppKitCursorKindDragLink.rawValue):
        cursor = .dragLink
    case Int32(AppKitCursorKindDragCopy.rawValue):
        cursor = .dragCopy
    case Int32(AppKitCursorKindContextualMenu.rawValue):
        cursor = .contextualMenu
    default:
        return nil
    }
    return appkitRetained(cursor)
}

@_cdecl("appkit_cursor_current")
public func appkit_cursor_current() -> UnsafeMutableRawPointer? {
    _ = NSApplication.shared
    return appkitRetained(NSCursor.current)
}

@_cdecl("appkit_cursor_from_image")
public func appkit_cursor_from_image(
    _ handle: UnsafeMutableRawPointer,
    _ width: Double,
    _ height: Double,
    _ hotSpotX: Double,
    _ hotSpotY: Double
) -> UnsafeMutableRawPointer? {
    guard width.isFinite, height.isFinite, width > 0, height > 0,
          hotSpotX.isFinite, hotSpotY.isFinite else {
        return nil
    }
    _ = NSApplication.shared
    let image: NSImage = appkitObject(handle)
    guard let scaled = image.copy() as? NSImage else { return nil }
    scaled.size = NSSize(width: CGFloat(width), height: CGFloat(height))
    return appkitRetained(NSCursor(image: scaled, hotSpot: NSPoint(x: CGFloat(hotSpotX), y: CGFloat(hotSpotY))))
}

@_cdecl("appkit_cursor_set")
public func appkit_cursor_set(_ handle: UnsafeMutableRawPointer) {
    let cursor: NSCursor = appkitObject(handle)
    cursor.set()
}

@_cdecl("appkit_cursor_hide")
public func appkit_cursor_hide() {
    NSCursor.hide()
}

@_cdecl("appkit_cursor_unhide")
public func appkit_cursor_unhide() {
    NSCursor.unhide()
}

@_cdecl("appkit_cursor_set_hidden_until_mouse_moves")
public func appkit_cursor_set_hidden_until_mouse_moves(_ hidden: Bool) {
    NSCursor.setHiddenUntilMouseMoves(hidden)
}
