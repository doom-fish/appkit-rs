// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

import AppKit
import AppKitCallbackABI

public typealias AppKitAccessibilityNodeStringCallback = @convention(c) (
    UnsafeMutableRawPointer,
    Int32,
    UInt,
    UInt,
    UnsafeMutablePointer<UnsafePointer<UInt8>?>,
    UnsafeMutablePointer<UInt>
) -> Int32

public typealias AppKitAccessibilityNodeNumberCallback = @convention(c) (
    UnsafeMutableRawPointer,
    Int32,
    UnsafeMutablePointer<Double>
) -> Bool

public typealias AppKitAccessibilityNodeIntegerCallback = @convention(c) (
    UnsafeMutableRawPointer,
    Int32,
    Int,
    UnsafeMutablePointer<Int>
) -> Bool

public typealias AppKitAccessibilityNodeFlagCallback = @convention(c) (
    UnsafeMutableRawPointer,
    Int32,
    Int32
) -> Bool

public typealias AppKitAccessibilityNodeValueCallback = @convention(c) (
    UnsafeMutableRawPointer,
    UnsafeMutablePointer<Double>,
    UnsafeMutablePointer<UnsafePointer<UInt8>?>,
    UnsafeMutablePointer<UInt>
) -> Int32

public typealias AppKitAccessibilityNodeListCallback = @convention(c) (
    UnsafeMutableRawPointer,
    Int32,
    UnsafeMutablePointer<UnsafePointer<UnsafeMutableRawPointer?>?>,
    UnsafeMutablePointer<UInt>
) -> Int32

public typealias AppKitAccessibilityNodeRectCallback = @convention(c) (
    UnsafeMutableRawPointer,
    Int32,
    UInt,
    UInt,
    UnsafeMutablePointer<Double>
) -> Int32

public typealias AppKitAccessibilityNodeRangeCallback = @convention(c) (
    UnsafeMutableRawPointer,
    Int32,
    Int,
    Double,
    Double,
    UnsafeMutablePointer<UInt>,
    UnsafeMutablePointer<UInt>
) -> Bool

public typealias AppKitAccessibilityTextRunCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafePointer<UInt8>?,
    UInt,
    UInt64,
    UInt64,
    UnsafePointer<UInt8>?,
    UInt,
    Double,
    Bool,
    Bool,
    UInt64,
    UInt64,
    UnsafePointer<UInt8>?,
    UInt,
    Int32
) -> Void

public typealias AppKitAccessibilityNodeTextRunsCallback = @convention(c) (
    UnsafeMutableRawPointer,
    UInt,
    UInt,
    UnsafeMutableRawPointer?,
    AppKitAccessibilityTextRunCallback
) -> Bool

public typealias AppKitAccessibilityNodeActionCallback = @convention(c) (
    UnsafeMutableRawPointer,
    Int32,
    Double,
    UInt,
    UInt,
    UnsafePointer<UInt8>?,
    UInt
) -> Bool

public typealias AppKitAccessibilityNodeReleaseCallback = @convention(c) (UnsafeMutableRawPointer) -> Void

public typealias AppKitAccessibilityViewChildrenCallback = @convention(c) (
    UnsafeMutableRawPointer,
    UnsafeMutablePointer<UnsafePointer<UnsafeMutableRawPointer?>?>,
    UnsafeMutablePointer<UInt>
) -> Void

public typealias AppKitAccessibilityViewFocusCallback = @convention(c) (UnsafeMutableRawPointer) -> UnsafeMutableRawPointer?

public typealias AppKitAccessibilityViewHitTestCallback = @convention(c) (
    UnsafeMutableRawPointer,
    Double,
    Double
) -> UnsafeMutableRawPointer?

struct AppKitAccessibilityCallbacks {
    let string: AppKitAccessibilityNodeStringCallback
    let number: AppKitAccessibilityNodeNumberCallback
    let integer: AppKitAccessibilityNodeIntegerCallback
    let flag: AppKitAccessibilityNodeFlagCallback
    let value: AppKitAccessibilityNodeValueCallback
    let list: AppKitAccessibilityNodeListCallback
    let rect: AppKitAccessibilityNodeRectCallback
    let range: AppKitAccessibilityNodeRangeCallback
    let textRuns: AppKitAccessibilityNodeTextRunsCallback
    let action: AppKitAccessibilityNodeActionCallback
    let release: AppKitAccessibilityNodeReleaseCallback
    let viewChildren: AppKitAccessibilityViewChildrenCallback
    let viewFocus: AppKitAccessibilityViewFocusCallback
    let viewHitTest: AppKitAccessibilityViewHitTestCallback

    init?(_ table: AppKitAccessibilityCallbackTableV1) {
        guard Thread.isMainThread,
              table.header.version == 1,
              table.header.size >= MemoryLayout<AppKitAccessibilityCallbackTableV1>.size,
              let string = table.string,
              let number = table.number,
              let integer = table.integer,
              let flag = table.flag,
              let value = table.value,
              let list = table.list,
              let rect = table.rect,
              let range = table.range,
              let textRuns = table.textRuns,
              let action = table.action,
              let release = table.release,
              let viewChildren = table.viewChildren,
              let viewFocus = table.viewFocus,
              let viewHitTest = table.viewHitTest else { return nil }
        self.string = string
        self.number = number
        self.integer = integer
        self.flag = flag
        self.value = value
        self.list = list
        self.rect = rect
        self.range = range
        self.textRuns = textRuns
        self.action = action
        self.release = release
        self.viewChildren = viewChildren
        self.viewFocus = viewFocus
        self.viewHitTest = viewHitTest
    }

}

func appKitAccessibilityObject<T: AnyObject>(_ pointer: UnsafeMutableRawPointer) -> T {
    Unmanaged<T>.fromOpaque(pointer).takeUnretainedValue()
}

func appKitAccessibilityString(_ pointer: UnsafePointer<UInt8>?, _ length: UInt) -> String? {
    guard let pointer else {
        return nil
    }
    return String(decoding: UnsafeBufferPointer(start: pointer, count: Int(length)), as: UTF8.self)
}

func appKitAccessibilityNodes(_ pointer: UnsafePointer<UnsafeMutableRawPointer?>?, _ count: UInt) -> [AppKitAccessibilityNode] {
    guard let pointer else {
        return []
    }
    return UnsafeBufferPointer(start: pointer, count: Int(count)).compactMap { element in
        element.map { Unmanaged<AppKitAccessibilityNode>.fromOpaque($0).takeUnretainedValue() }
    }
}

func appKitAccessibilityPixelPoint(view: NSView, window: NSWindow, screenPoint: NSPoint) -> NSPoint {
    let point = view.convert(window.convertPoint(fromScreen: screenPoint), from: nil)
    let factor = window.backingScaleFactor
    let y = view.isFlipped ? point.y * factor : (view.bounds.size.height - point.y) * factor
    return NSPoint(x: point.x * factor, y: y)
}

final class AppKitAccessibilityViewRef: NSObject {
    weak var view: NSView?
    let callbacks: AppKitAccessibilityCallbacks

    init(view: NSView, callbacks: AppKitAccessibilityCallbacks) {
        self.view = view
        self.callbacks = callbacks
        super.init()
    }

    func screenRect(_ rect: [Double]) -> NSRect {
        guard let view, let window = view.window else {
            return .zero
        }
        let factor = window.backingScaleFactor
        let y = view.isFlipped ? rect[1] / factor : view.bounds.size.height - rect[3] / factor
        let local = NSRect(
            x: rect[0] / factor,
            y: y,
            width: (rect[2] - rect[0]) / factor,
            height: (rect[3] - rect[1]) / factor
        )
        return window.convertToScreen(view.convert(local, to: nil))
    }

    func pixelPoint(screen point: NSPoint) -> NSPoint? {
        guard let view, let window = view.window else {
            return nil
        }
        return appKitAccessibilityPixelPoint(view: view, window: window, screenPoint: point)
    }
}

@_cdecl("appkit_a11y_retain")
public func appkit_a11y_retain(_ object: UnsafeMutableRawPointer) {
    _ = Unmanaged<AnyObject>.fromOpaque(object).retain()
}

@_cdecl("appkit_a11y_release")
public func appkit_a11y_release(_ object: UnsafeMutableRawPointer) {
    Unmanaged<AnyObject>.fromOpaque(object).release()
}

@_cdecl("appkit_a11y_autorelease")
public func appkit_a11y_autorelease(_ object: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    Unmanaged<AnyObject>.fromOpaque(object).retain().autorelease().toOpaque()
}

@_cdecl("appkit_a11y_array")
public func appkit_a11y_array(
    _ objects: UnsafePointer<UnsafeMutableRawPointer?>?,
    _ count: UInt
) -> UnsafeMutableRawPointer {
    let array = appKitAccessibilityNodes(objects, count) as NSArray
    return Unmanaged.passRetained(array).autorelease().toOpaque()
}

@_cdecl("appkit_a11y_view_ref_new")
public func appkit_a11y_view_ref_new(
    _ view: UnsafeMutableRawPointer,
    _ table: UnsafePointer<AppKitAccessibilityCallbackTableV1>?
) -> UnsafeMutableRawPointer? {
    guard let table, let callbacks = AppKitAccessibilityCallbacks(table.pointee) else {
        return nil
    }
    return Unmanaged.passRetained(AppKitAccessibilityViewRef(
        view: appKitAccessibilityObject(view),
        callbacks: callbacks
    )).toOpaque()
}

@_cdecl("appkit_a11y_view_ref_pixel_point")
public func appkit_a11y_view_ref_pixel_point(
    _ reference: UnsafeMutableRawPointer,
    _ x: Double,
    _ y: Double,
    _ out: UnsafeMutablePointer<Double>
) -> Bool {
    let reference: AppKitAccessibilityViewRef = appKitAccessibilityObject(reference)
    guard let point = reference.pixelPoint(screen: NSPoint(x: x, y: y)) else {
        return false
    }
    out[0] = point.x
    out[1] = point.y
    return true
}

@_cdecl("appkit_a11y_node_new")
public func appkit_a11y_node_new(
    _ reference: UnsafeMutableRawPointer,
    _ node: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer {
    Unmanaged.passRetained(AppKitAccessibilityNode(reference: appKitAccessibilityObject(reference), node: node)).toOpaque()
}

private let appKitAccessibilityNotifications: [NSAccessibility.Notification] = [
    .focusedUIElementChanged,
    .valueChanged,
    .selectedTextChanged,
    .titleChanged,
    .selectedRowsChanged,
    .uiElementDestroyed,
]

@_cdecl("appkit_a11y_post")
public func appkit_a11y_post(_ element: UnsafeMutableRawPointer, _ notification: Int32) {
    let element: AnyObject = appKitAccessibilityObject(element)
    NSAccessibility.post(element: element, notification: appKitAccessibilityNotifications[Int(notification)])
}

@_cdecl("appkit_a11y_announce")
public func appkit_a11y_announce(
    _ reference: UnsafeMutableRawPointer,
    _ text: UnsafePointer<UInt8>?,
    _ length: UInt,
    _ assertive: Bool
) {
    let reference: AppKitAccessibilityViewRef = appKitAccessibilityObject(reference)
    guard let window = reference.view?.window, let text = appKitAccessibilityString(text, length) else {
        return
    }
    let priority: NSAccessibilityPriorityLevel = assertive ? .high : .medium
    NSAccessibility.post(
        element: window,
        notification: .announcementRequested,
        userInfo: [.announcement: text, .priority: priority.rawValue]
    )
}
