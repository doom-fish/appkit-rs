import AppKit
import AppKitCallbackABI
import AppKitExceptionShim

private var legacyDiagonalCursors: [Int32: NSCursor] = [:]

private func legacyDiagonalCursor(_ kind: Int32) -> NSCursor? {
    guard kind == Int32(AppKitCursorKindResizeUpLeftDownRight.rawValue)
        || kind == Int32(AppKitCursorKindResizeUpRightDownLeft.rawValue) else { return nil }
    if let cached = legacyDiagonalCursors[kind] { return cached }

    var source: NSCursor?
    var size = NSSize.zero
    var bitmap: NSBitmapImageRep?
    var graphics: NSGraphicsContext?
    guard appkit_try({
        source = .resizeLeftRight
        size = source?.image.size ?? .zero
        guard size.width.isFinite, size.height.isFinite,
              size.width > 0, size.height > 0,
              size.width <= 512, size.height <= 512 else { return }
        bitmap = NSBitmapImageRep(
            bitmapDataPlanes: nil,
            pixelsWide: Int(size.width.rounded(.up)),
            pixelsHigh: Int(size.height.rounded(.up)),
            bitsPerSample: 8,
            samplesPerPixel: 4,
            hasAlpha: true,
            isPlanar: false,
            colorSpaceName: .deviceRGB,
            bytesPerRow: 0,
            bitsPerPixel: 0
        )
        if let bitmap {
            bitmap.size = size
            graphics = NSGraphicsContext(bitmapImageRep: bitmap)
        }
    }) == nil, let source, let bitmap, let graphics else { return nil }

    let previous = NSGraphicsContext.current
    let degrees: CGFloat = kind == Int32(AppKitCursorKindResizeUpLeftDownRight.rawValue) ? -45 : 45
    let drawFailure = appkit_try {
        NSGraphicsContext.current = graphics
        let transform = NSAffineTransform()
        transform.translateX(by: size.width / 2, yBy: size.height / 2)
        transform.rotate(byDegrees: degrees)
        transform.translateX(by: -size.width / 2, yBy: -size.height / 2)
        transform.concat()
        source.image.draw(in: NSRect(origin: .zero, size: size), from: .zero, operation: .sourceOver, fraction: 1)
        graphics.flushGraphics()
    }
    let restoreFailure = appkit_try { NSGraphicsContext.current = previous }
    guard drawFailure == nil, restoreFailure == nil else { return nil }

    var cursor: NSCursor?
    guard appkit_try({
        let image = NSImage(size: size)
        image.addRepresentation(bitmap)
        cursor = NSCursor(image: image, hotSpot: NSPoint(x: size.width / 2, y: size.height / 2))
    }) == nil, let cursor else { return nil }
    legacyDiagonalCursors[kind] = cursor
    return cursor
}

@_cdecl("appkit_cursor_system")
public func appkit_cursor_system(_ kind: Int32) -> UnsafeMutableRawPointer? {
    guard Thread.isMainThread else { return nil }
    var cursor: NSCursor?
    guard appkit_try({
        _ = NSApplication.shared
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
                cursor = legacyDiagonalCursor(kind)
            }
        case Int32(AppKitCursorKindResizeUpRightDownLeft.rawValue):
            if #available(macOS 15.0, *) {
                cursor = .frameResize(position: .topRight, directions: .all)
            } else {
                cursor = legacyDiagonalCursor(kind)
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
            break
        }
    }) == nil else { return nil }
    return appkitRetained(cursor)
}

@_cdecl("appkit_cursor_check_legacy_diagonals")
public func appkit_cursor_check_legacy_diagonals() -> Bool {
    guard Thread.isMainThread else { return false }
    var valid = false
    guard appkit_try({
        _ = NSApplication.shared
        let leftKind = Int32(AppKitCursorKindResizeUpLeftDownRight.rawValue)
        let rightKind = Int32(AppKitCursorKindResizeUpRightDownLeft.rawValue)
        guard let left = legacyDiagonalCursor(leftKind),
              let right = legacyDiagonalCursor(rightKind),
              left !== right,
              left === legacyDiagonalCursor(leftKind),
              right === legacyDiagonalCursor(rightKind) else { return }
        var patterns: [[Bool]] = []
        valid = [left, right].allSatisfy { cursor in
            let image = cursor.image
            let size = image.size
            let point = cursor.hotSpot
            guard point.x == size.width / 2, point.y == size.height / 2,
                  let bitmap = image.representations.first as? NSBitmapImageRep else { return false }
            var pattern: [Bool] = []
            for y in 0..<bitmap.pixelsHigh {
                for x in 0..<bitmap.pixelsWide {
                    pattern.append((bitmap.colorAt(x: x, y: y)?.alphaComponent ?? 0) > 0)
                }
            }
            patterns.append(pattern)
            return pattern.contains(true)
        }
        valid = valid && patterns.count == 2 && patterns[0] != patterns[1]
    }) == nil else { return false }
    return valid
}

@_cdecl("appkit_cursor_current")
public func appkit_cursor_current() -> UnsafeMutableRawPointer? {
    guard Thread.isMainThread else { return nil }
    var cursor: NSCursor?
    guard appkit_try({
        _ = NSApplication.shared
        cursor = NSCursor.current
    }) == nil else { return nil }
    return appkitRetained(cursor)
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
          hotSpotX.isFinite, hotSpotY.isFinite,
          hotSpotX >= 0, hotSpotX <= width,
          hotSpotY >= 0, hotSpotY <= height,
          Thread.isMainThread else {
        return nil
    }
    var cursor: NSCursor?
    guard appkit_try({
        _ = NSApplication.shared
        let image: NSImage = appkitObject(handle)
        guard let scaled = image.copy() as? NSImage else { return }
        scaled.size = NSSize(width: CGFloat(width), height: CGFloat(height))
        cursor = NSCursor(image: scaled, hotSpot: NSPoint(x: CGFloat(hotSpotX), y: CGFloat(hotSpotY)))
    }) == nil else { return nil }
    return appkitRetained(cursor)
}

@_cdecl("appkit_cursor_set")
public func appkit_cursor_set(_ handle: UnsafeMutableRawPointer) -> Bool {
    guard Thread.isMainThread else { return false }
    let cursor: NSCursor = appkitObject(handle)
    return appkit_try { cursor.set() } == nil
}

@_cdecl("appkit_cursor_hide")
public func appkit_cursor_hide() -> Bool {
    guard Thread.isMainThread else { return false }
    return appkit_try { NSCursor.hide() } == nil
}

@_cdecl("appkit_cursor_unhide")
public func appkit_cursor_unhide() -> Bool {
    guard Thread.isMainThread else { return false }
    return appkit_try { NSCursor.unhide() } == nil
}

@_cdecl("appkit_cursor_set_hidden_until_mouse_moves")
public func appkit_cursor_set_hidden_until_mouse_moves(_ hidden: Bool) -> Bool {
    guard Thread.isMainThread else { return false }
    return appkit_try { NSCursor.setHiddenUntilMouseMoves(hidden) } == nil
}
