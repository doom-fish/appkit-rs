import AppKit
import AppKitCallbackABI

private extension NSEvent {
    var appkitHasMouseButton: Bool {
        switch type.rawValue {
        case 1, 2, 3, 4, 6, 7, 25, 26, 27: return true
        default: return false
        }
    }

    var appkitHasLocation: Bool {
        switch type.rawValue {
        case 1...9, 17, 22...27, 29...31, 34: return true
        default: return false
        }
    }

    var appkitHasKeyCharacters: Bool {
        type.rawValue == 10 || type.rawValue == 11
    }

    var appkitHasKeyCode: Bool {
        appkitHasKeyCharacters || type.rawValue == 12
    }

    var appkitHasPhase: Bool {
        type.rawValue == 22 || type.rawValue == 30 || type.rawValue == 31
    }

    var appkitHasPressure: Bool {
        appkitHasMouseButton || type.rawValue == 23 || type.rawValue == 34
    }
}

private func appkitEventString(_ bytes: UnsafePointer<UInt8>?, _ length: Int) -> String? {
    guard length >= 0, length == 0 || bytes != nil else { return nil }
    return String(decoding: UnsafeBufferPointer(start: bytes, count: length), as: UTF8.self)
}

@_cdecl("appkit_event_current")
public func appkit_event_current() -> UnsafeMutableRawPointer? {
    appkitRetained(NSApp?.currentEvent)
}

@_cdecl("appkit_event_pressed_mouse_buttons")
public func appkit_event_pressed_mouse_buttons() -> UInt64 {
    UInt64(NSEvent.pressedMouseButtons)
}

@_cdecl("appkit_event_class_modifier_flags")
public func appkit_event_class_modifier_flags() -> UInt64 {
    UInt64(NSEvent.modifierFlags.rawValue)
}

@_cdecl("appkit_event_type")
public func appkit_event_type(_ handle: UnsafeMutableRawPointer) -> UInt64 {
    let event: NSEvent = appkitObject(handle)
    return UInt64(event.type.rawValue)
}

@_cdecl("appkit_event_modifier_flags")
public func appkit_event_modifier_flags(_ handle: UnsafeMutableRawPointer) -> UInt64 {
    let event: NSEvent = appkitObject(handle)
    return UInt64(event.modifierFlags.rawValue)
}

@_cdecl("appkit_event_location_in_window")
public func appkit_event_location_in_window(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<CGPoint>
) -> Bool {
    let event: NSEvent = appkitObject(handle)
    guard event.appkitHasLocation else { return false }
    out.pointee = event.locationInWindow
    return true
}

@_cdecl("appkit_event_button_number")
public func appkit_event_button_number(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<Int64>
) -> Bool {
    let event: NSEvent = appkitObject(handle)
    guard event.appkitHasMouseButton else { return false }
    out.pointee = Int64(event.buttonNumber)
    return true
}

@_cdecl("appkit_event_click_count")
public func appkit_event_click_count(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<Int64>
) -> Bool {
    let event: NSEvent = appkitObject(handle)
    guard event.appkitHasMouseButton else { return false }
    out.pointee = Int64(event.clickCount)
    return true
}

@_cdecl("appkit_event_is_repeat")
public func appkit_event_is_repeat(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<Bool>
) -> Bool {
    let event: NSEvent = appkitObject(handle)
    guard event.appkitHasKeyCharacters else { return false }
    out.pointee = event.isARepeat
    return true
}

@_cdecl("appkit_event_key_code")
public func appkit_event_key_code(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<UInt16>
) -> Bool {
    let event: NSEvent = appkitObject(handle)
    guard event.appkitHasKeyCode else { return false }
    out.pointee = event.keyCode
    return true
}

@_cdecl("appkit_event_characters")
public func appkit_event_characters(
    _ handle: UnsafeMutableRawPointer,
    _ ignoringModifiers: Bool,
    _ out: UnsafeMutablePointer<AppKitEventString>
) -> Bool {
    let event: NSEvent = appkitObject(handle)
    guard event.appkitHasKeyCharacters,
          let characters = ignoringModifiers ? event.charactersIgnoringModifiers : event.characters,
          let pointer = malloc(max(1, characters.utf8.count))?.assumingMemoryBound(to: UInt8.self) else {
        return false
    }
    let bytes = Array(characters.utf8)
    bytes.withUnsafeBufferPointer { buffer in
        if let base = buffer.baseAddress {
            pointer.update(from: base, count: buffer.count)
        }
    }
    out.pointee = AppKitEventString(bytes: pointer, length: bytes.count)
    return true
}

@_cdecl("appkit_event_string_free")
public func appkit_event_string_free(_ bytes: UnsafeMutablePointer<UInt8>?) {
    free(bytes)
}

@_cdecl("appkit_event_scrolling_delta")
public func appkit_event_scrolling_delta(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<CGPoint>
) -> Bool {
    let event: NSEvent = appkitObject(handle)
    guard event.type.rawValue == 22 else { return false }
    out.pointee = CGPoint(x: event.scrollingDeltaX, y: event.scrollingDeltaY)
    return true
}

@_cdecl("appkit_event_has_precise_scrolling_deltas")
public func appkit_event_has_precise_scrolling_deltas(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<Bool>
) -> Bool {
    let event: NSEvent = appkitObject(handle)
    guard event.type.rawValue == 22 else { return false }
    out.pointee = event.hasPreciseScrollingDeltas
    return true
}

@_cdecl("appkit_event_phase")
public func appkit_event_phase(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<UInt64>
) -> Bool {
    let event: NSEvent = appkitObject(handle)
    guard event.appkitHasPhase else { return false }
    out.pointee = UInt64(event.phase.rawValue)
    return true
}

@_cdecl("appkit_event_momentum_phase")
public func appkit_event_momentum_phase(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<UInt64>
) -> Bool {
    let event: NSEvent = appkitObject(handle)
    guard event.type.rawValue == 22 else { return false }
    out.pointee = UInt64(event.momentumPhase.rawValue)
    return true
}

@_cdecl("appkit_event_pressure")
public func appkit_event_pressure(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<Float>
) -> Bool {
    let event: NSEvent = appkitObject(handle)
    guard event.appkitHasPressure else { return false }
    out.pointee = event.pressure
    return true
}

@_cdecl("appkit_event_stage")
public func appkit_event_stage(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<Int64>
) -> Bool {
    let event: NSEvent = appkitObject(handle)
    guard event.type.rawValue == 34 else { return false }
    out.pointee = Int64(event.stage)
    return true
}

@_cdecl("appkit_event_magnification")
public func appkit_event_magnification(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<Double>
) -> Bool {
    let event: NSEvent = appkitObject(handle)
    guard event.type.rawValue == 30 else { return false }
    out.pointee = Double(event.magnification)
    return true
}

@_cdecl("appkit_event_swipe_delta")
public func appkit_event_swipe_delta(
    _ handle: UnsafeMutableRawPointer,
    _ out: UnsafeMutablePointer<CGPoint>
) -> Bool {
    let event: NSEvent = appkitObject(handle)
    guard event.type.rawValue == 31 else { return false }
    out.pointee = CGPoint(x: event.deltaX, y: event.deltaY)
    return true
}

@_cdecl("appkit_event_timestamp")
public func appkit_event_timestamp(_ handle: UnsafeMutableRawPointer) -> Double {
    let event: NSEvent = appkitObject(handle)
    return event.timestamp
}

@_cdecl("appkit_event_window_number")
public func appkit_event_window_number(_ handle: UnsafeMutableRawPointer) -> Int64 {
    let event: NSEvent = appkitObject(handle)
    return Int64(event.windowNumber)
}

@_cdecl("appkit_event_synthetic_key")
public func appkit_event_synthetic_key(
    _ type: UInt64,
    _ modifiers: UInt64,
    _ keyCode: UInt16,
    _ characters: UnsafePointer<UInt8>?,
    _ charactersLength: Int,
    _ ignoringModifiers: UnsafePointer<UInt8>?,
    _ ignoringModifiersLength: Int,
    _ isRepeat: Bool
) -> UnsafeMutableRawPointer? {
    let kind: NSEvent.EventType
    switch type {
    case 10: kind = .keyDown
    case 11: kind = .keyUp
    default: return nil
    }
    guard let characters = appkitEventString(characters, charactersLength),
          let ignoringModifiers = appkitEventString(ignoringModifiers, ignoringModifiersLength) else {
        return nil
    }
    let event = NSEvent.keyEvent(
        with: kind,
        location: .zero,
        modifierFlags: NSEvent.ModifierFlags(rawValue: UInt(modifiers)),
        timestamp: 0,
        windowNumber: 0,
        context: nil,
        characters: characters,
        charactersIgnoringModifiers: ignoringModifiers,
        isARepeat: isRepeat,
        keyCode: keyCode
    )
    return appkitRetained(event)
}

@_cdecl("appkit_event_synthetic_mouse")
public func appkit_event_synthetic_mouse(
    _ type: UInt64,
    _ location: UnsafePointer<CGPoint>,
    _ modifiers: UInt64,
    _ clickCount: Int64,
    _ pressure: Float
) -> UnsafeMutableRawPointer? {
    let kind: NSEvent.EventType
    switch type {
    case 1: kind = .leftMouseDown
    case 2: kind = .leftMouseUp
    case 3: kind = .rightMouseDown
    case 4: kind = .rightMouseUp
    case 5: kind = .mouseMoved
    case 6: kind = .leftMouseDragged
    case 7: kind = .rightMouseDragged
    case 25: kind = .otherMouseDown
    case 26: kind = .otherMouseUp
    case 27: kind = .otherMouseDragged
    default: return nil
    }
    guard clickCount >= 0, pressure.isFinite, (0...1).contains(pressure),
          location.pointee.x.isFinite, location.pointee.y.isFinite else {
        return nil
    }
    let event = NSEvent.mouseEvent(
        with: kind,
        location: location.pointee,
        modifierFlags: NSEvent.ModifierFlags(rawValue: UInt(modifiers)),
        timestamp: 0,
        windowNumber: 0,
        context: nil,
        eventNumber: 0,
        clickCount: Int(clickCount),
        pressure: pressure
    )
    return appkitRetained(event)
}
