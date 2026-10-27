// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

import AppKit
import ObjectiveC

final class AppKitAccessibilityAttachment: NSObject {
    var state: UnsafeMutableRawPointer?
    let previousClass: AnyClass
    let reference: AppKitAccessibilityViewRef

    init(state: UnsafeMutableRawPointer, previousClass: AnyClass, reference: AppKitAccessibilityViewRef) {
        self.reference = reference
        self.state = state
        self.previousClass = previousClass
        super.init()
    }
}

private let attachmentKey = UnsafeMutableRawPointer.allocate(byteCount: 1, alignment: 1)

private var subclasses: [ObjectIdentifier: AnyClass] = [:]

private func attachment(of view: AnyObject) -> AppKitAccessibilityAttachment? {
    objc_getAssociatedObject(view, attachmentKey) as? AppKitAccessibilityAttachment
}

private func implementation(_ block: Any) -> IMP {
    imp_implementationWithBlock(block)
}

private func subclass(of previous: AnyClass) -> AnyClass {
    if let existing = subclasses[ObjectIdentifier(previous)] {
        return existing
    }
    let name = "AppKitAccessibilityRsSubclassOf" + NSStringFromClass(previous)
    guard let created = objc_allocateClassPair(previous, name, 0) else {
        fatalError("accesskit-rs could not make the class \(name)")
    }
    let superclass: @convention(block) (AnyObject) -> AnyClass? = { view in
        attachment(of: view).flatMap { class_getSuperclass($0.previousClass) }
    }
    let children: @convention(block) (NSView) -> NSArray? = { view in
        guard let attached = attachment(of: view), let state = attached.state else {
            return nil
        }
        var pointer: UnsafePointer<UnsafeMutableRawPointer?>?
        var count: UInt = 0
        attached.reference.callbacks.viewChildren(state, &pointer, &count)
        return appKitAccessibilityNodes(pointer, count) as NSArray
    }
    let focus: @convention(block) (NSView) -> AnyObject? = { view in
        guard let attached = attachment(of: view), let state = attached.state, let node = attached.reference.callbacks.viewFocus(state) else {
            return nil
        }
        return Unmanaged<AnyObject>.fromOpaque(node).takeUnretainedValue()
    }
    let hitTest: @convention(block) (NSView, NSPoint) -> AnyObject? = { view, point in
        guard let attached = attachment(of: view), let state = attached.state, let window = view.window else {
            return nil
        }
        let pixel = appKitAccessibilityPixelPoint(view: view, window: window, screenPoint: point)
        guard let node = attached.reference.callbacks.viewHitTest(state, pixel.x, pixel.y) else {
            return nil
        }
        return Unmanaged<AnyObject>.fromOpaque(node).takeUnretainedValue()
    }
    class_addMethod(created, NSSelectorFromString("superclass"), implementation(superclass), "#@:")
    class_addMethod(created, NSSelectorFromString("accessibilityChildren"), implementation(children), "@@:")
    class_addMethod(created, NSSelectorFromString("accessibilityFocusedUIElement"), implementation(focus), "@@:")
    class_addMethod(created, NSSelectorFromString("accessibilityHitTest:"), implementation(hitTest), "@@:{CGPoint=dd}")
    objc_registerClassPair(created)
    subclasses[ObjectIdentifier(previous)] = created
    return created
}

@_cdecl("appkit_a11y_window_content_view")
public func appkit_a11y_window_content_view(_ window: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer? {
    let window: NSWindow = appKitAccessibilityObject(window)
    return window.contentView.map { Unmanaged.passRetained($0).toOpaque() }
}

@_cdecl("appkit_a11y_subclass_attach")
public func appkit_a11y_subclass_attach(
    _ referencePointer: UnsafeMutableRawPointer,
    _ state: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    let reference: AppKitAccessibilityViewRef = appKitAccessibilityObject(referencePointer)
    guard let view = reference.view, attachment(of: view) == nil,
          let previous = object_getClass(view) else {
        return nil
    }
    let attached = AppKitAccessibilityAttachment(state: state, previousClass: previous, reference: reference)
    objc_setAssociatedObject(view, attachmentKey, attached, .OBJC_ASSOCIATION_RETAIN_NONATOMIC)
    object_setClass(view, subclass(of: previous))
    return Unmanaged.passRetained(attached).toOpaque()
}

@_cdecl("appkit_a11y_subclass_detach")
public func appkit_a11y_subclass_detach(
    _ view: UnsafeMutableRawPointer,
    _ attachment: UnsafeMutableRawPointer
) {
    let view: NSView = appKitAccessibilityObject(view)
    let attached: AppKitAccessibilityAttachment = appKitAccessibilityObject(attachment)
    attached.state = nil
    object_setClass(view, attached.previousClass)
    objc_setAssociatedObject(view, attachmentKey, nil, .OBJC_ASSOCIATION_RETAIN_NONATOMIC)
}

@_cdecl("appkit_a11y_add_focus_forwarder")
public func appkit_a11y_add_focus_forwarder(_ className: UnsafePointer<CChar>) -> Int32 {
    guard let windowClass = NSClassFromString(String(cString: className)) else {
        return 1
    }
    let forward: @convention(block) (NSWindow) -> AnyObject? = { window in
        window.contentView?.accessibilityFocusedUIElement as AnyObject?
    }
    let added = class_addMethod(
        windowClass,
        NSSelectorFromString("accessibilityFocusedUIElement"),
        implementation(forward),
        "@@:"
    )
    return added ? 0 : 2
}
