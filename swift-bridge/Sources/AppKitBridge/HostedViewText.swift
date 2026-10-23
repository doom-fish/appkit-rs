import AppKit
import AppKitCallbackABI
import AppKitExceptionShim

private enum TextQuery: UInt32 {
    case hasMarkedText = 1
    case markedRange = 2
    case selectedRange = 3
    case firstRect = 4
    case attributedSubstring = 5
    case characterIndex = 6
}

private enum TextAction: UInt32 {
    case insertText = 1
    case setMarkedText = 2
    case unmarkText = 3
    case doCommand = 4
}

private let invalidTextRange = AppKitUtf16Range(location: NSNotFound, length: 0)

private func textRange(_ range: NSRange) -> AppKitUtf16Range {
    AppKitUtf16Range(location: range.location, length: range.length)
}

private func nativeTextRange(_ range: AppKitUtf16Range) -> NSRange {
    guard range.location >= 0,
          range.location < NSNotFound,
          range.length >= 0,
          range.length <= Int.max - range.location else {
        return NSRange(location: NSNotFound, length: 0)
    }
    return NSRange(location: range.location, length: range.length)
}

private func validTextRect(_ rect: CGRect) -> Bool {
    rect.origin.x.isFinite && rect.origin.y.isFinite &&
    rect.size.width.isFinite && rect.size.height.isFinite &&
    rect.size.width >= 0 && rect.size.height >= 0
}

private func textFromInput(_ value: Any) -> String? {
    if let attributed = value as? NSAttributedString {
        return attributed.string
    }
    return value as? String
}

private final class TextInputCallbackOwner {
    let token: UInt64
    let table: AppKitTextInputCallbackTableV1

    init?(token: UInt64, table: AppKitTextInputCallbackTableV1) {
        guard Thread.isMainThread,
              token != 0,
              table.header.version == 1,
              table.header.size >= MemoryLayout<AppKitTextInputCallbackTableV1>.size,
              table.deactivate != nil,
              table.query != nil,
              table.action != nil else {
            return nil
        }
        self.token = token
        self.table = table
    }

    deinit {
        table.deactivate?(token)
    }

    func query(
        _ selector: TextQuery,
        range: NSRange = NSRange(location: NSNotFound, length: 0),
        point: NSPoint = .zero
    ) -> AppKitTextInputQueryResult? {
        guard Thread.isMainThread else { return nil }
        var result = AppKitTextInputQueryResult(
            range: invalidTextRange,
            rect: .zero,
            character_index: NSNotFound,
            has_marked_text: false
        )
        guard table.query?(
            token, selector.rawValue, textRange(range),
            Double(point.x), Double(point.y), &result, nil, nil
        ) ?? false else { return nil }
        return result
    }

    func substring(_ range: NSRange) -> (String, AppKitUtf16Range)? {
        guard Thread.isMainThread else { return nil }
        var result = AppKitTextInputQueryResult(
            range: invalidTextRange,
            rect: .zero,
            character_index: NSNotFound,
            has_marked_text: false
        )
        var copied: String?
        let succeeded = withUnsafeMutablePointer(to: &copied) { pointer in
            table.query?(
                token, TextQuery.attributedSubstring.rawValue, textRange(range),
                0, 0, &result, UnsafeMutableRawPointer(pointer),
                { context, bytes, length in
                    guard let context, length >= 0, length == 0 || bytes != nil else {
                        return false
                    }
                    let buffer = UnsafeBufferPointer(start: bytes, count: length)
                    let text = String(bytes: buffer, encoding: .utf8)
                    context.assumingMemoryBound(to: String?.self).pointee = text
                    return text != nil
                }
            ) ?? false
        }
        guard succeeded, let copied else { return nil }
        return (copied, result.range)
    }

    func action(
        _ selector: TextAction,
        text: String = "",
        selection: NSRange = NSRange(location: NSNotFound, length: 0),
        replacement: NSRange = NSRange(location: NSNotFound, length: 0)
    ) {
        guard Thread.isMainThread else { return }
        let bytes = Array(text.utf8)
        bytes.withUnsafeBufferPointer { buffer in
            table.action?(
                token, selector.rawValue, buffer.baseAddress, buffer.count,
                textRange(selection), textRange(replacement)
            )
        }
    }
}

extension HostedView: NSTextInputClient {
    func validAttributesForMarkedText() -> [NSAttributedString.Key] {
        []
    }

    func hasMarkedText() -> Bool {
        withExtendedLifetime(self) {
            (textInputOwner as? TextInputCallbackOwner)?.query(.hasMarkedText)?.has_marked_text ?? false
        }
    }

    func markedRange() -> NSRange {
        withExtendedLifetime(self) {
            guard let range = (textInputOwner as? TextInputCallbackOwner)?.query(.markedRange)?.range else {
                return NSRange(location: NSNotFound, length: 0)
            }
            return nativeTextRange(range)
        }
    }

    func selectedRange() -> NSRange {
        withExtendedLifetime(self) {
            guard let range = (textInputOwner as? TextInputCallbackOwner)?.query(.selectedRange)?.range else {
                return NSRange(location: NSNotFound, length: 0)
            }
            return nativeTextRange(range)
        }
    }

    func firstRect(forCharacterRange range: NSRange, actualRange: NSRangePointer?) -> NSRect {
        withExtendedLifetime(self) {
            actualRange?.pointee = NSRange(location: NSNotFound, length: 0)
            guard let result = (textInputOwner as? TextInputCallbackOwner)?.query(.firstRect, range: range),
                  let window,
                  validTextRect(result.rect),
                  result.rect.maxY.isFinite,
                  bounds.height.isFinite else {
                return .zero
            }
            let local = NSRect(
                x: bounds.minX + result.rect.origin.x,
                y: bounds.maxY - result.rect.maxY,
                width: result.rect.width,
                height: result.rect.height
            )
            guard validTextRect(local) else { return .zero }
            var screen: NSRect = .zero
            guard appkit_try({
                screen = window.convertToScreen(convert(local, to: nil))
            }) == nil, validTextRect(screen) else {
                return .zero
            }
            actualRange?.pointee = nativeTextRange(result.range)
            return screen
        }
    }

    func insertText(_ string: Any, replacementRange: NSRange) {
        withExtendedLifetime(self) {
            guard Thread.isMainThread else { return }
            var text: String?
            guard appkit_try({ text = textFromInput(string) }) == nil, let text else { return }
            (textInputOwner as? TextInputCallbackOwner)?.action(
                .insertText, text: text, replacement: replacementRange
            )
        }
    }

    func setMarkedText(_ string: Any, selectedRange: NSRange, replacementRange: NSRange) {
        withExtendedLifetime(self) {
            guard Thread.isMainThread else { return }
            var text: String?
            guard appkit_try({ text = textFromInput(string) }) == nil, let text else { return }
            (textInputOwner as? TextInputCallbackOwner)?.action(
                .setMarkedText, text: text, selection: selectedRange, replacement: replacementRange
            )
        }
    }

    func unmarkText() {
        withExtendedLifetime(self) {
            (textInputOwner as? TextInputCallbackOwner)?.action(.unmarkText)
        }
    }

    func attributedSubstring(
        forProposedRange range: NSRange,
        actualRange: NSRangePointer?
    ) -> NSAttributedString? {
        withExtendedLifetime(self) {
            actualRange?.pointee = NSRange(location: NSNotFound, length: 0)
            guard let (text, adjusted) = (textInputOwner as? TextInputCallbackOwner)?.substring(range) else {
                return nil
            }
            var attributed: NSAttributedString?
            guard appkit_try({
                attributed = NSAttributedString(string: text)
            }) == nil, let attributed else {
                return nil
            }
            actualRange?.pointee = nativeTextRange(adjusted)
            return attributed
        }
    }

    func characterIndex(for point: NSPoint) -> Int {
        withExtendedLifetime(self) {
            guard Thread.isMainThread, let window else { return NSNotFound }
            var local: NSPoint?
            guard appkit_try({
                let windowPoint = window.convertPoint(fromScreen: point)
                let converted = convert(windowPoint, from: nil)
                local = NSPoint(x: converted.x - bounds.minX, y: bounds.maxY - converted.y)
            }) == nil, let local, local.x.isFinite, local.y.isFinite else {
                return NSNotFound
            }
            return (textInputOwner as? TextInputCallbackOwner)?
                .query(.characterIndex, point: local)?.character_index ?? NSNotFound
        }
    }

    @objc(doCommandBySelector:)
    func handleTextCommand(_ selector: Selector) {
        withExtendedLifetime(self) {
            guard Thread.isMainThread else { return }
            (textInputOwner as? TextInputCallbackOwner)?.action(
                .doCommand, text: NSStringFromSelector(selector)
            )
        }
    }
}

@_cdecl("appkit_text_input_install")
public func appkit_text_input_install(
    _ handle: UnsafeMutableRawPointer,
    _ token: UInt64,
    _ table: UnsafePointer<AppKitTextInputCallbackTableV1>?
) -> Bool {
    guard Thread.isMainThread, token != 0, let table else { return false }
    let header = table.withMemoryRebound(to: AppKitCallbackHeader.self, capacity: 1) {
        $0.pointee
    }
    guard header.version == 1,
          header.size >= MemoryLayout<AppKitTextInputCallbackTableV1>.size,
          let owner = TextInputCallbackOwner(token: token, table: table.pointee) else {
        return false
    }
    let view: HostedView = appkitObject(handle)
    guard view.textInputOwner == nil else { return false }
    view.textInputOwner = owner
    return true
}

@_cdecl("appkit_text_input_uninstall")
public func appkit_text_input_uninstall(_ handle: UnsafeMutableRawPointer, _ token: UInt64) {
    guard Thread.isMainThread else { return }
    let view: HostedView = appkitObject(handle)
    if (view.textInputOwner as? TextInputCallbackOwner)?.token == token {
        view.textInputOwner = nil
    }
}

@_cdecl("appkit_text_input_handle_event")
public func appkit_text_input_handle_event(
    _ handle: UnsafeMutableRawPointer,
    _ token: UInt64,
    _ eventHandle: UnsafeMutableRawPointer
) -> Bool {
    guard Thread.isMainThread else { return false }
    let view: HostedView = appkitObject(handle)
    guard (view.textInputOwner as? TextInputCallbackOwner)?.token == token else { return false }
    let event: NSEvent = appkitObject(eventHandle)
    return withExtendedLifetime(view) {
        var handled = false
        guard appkit_try({
            if let context = view.inputContext {
                handled = context.handleEvent(event)
            }
        }) == nil else { return false }
        return handled
    }
}

@_cdecl("appkit_text_input_discard_marked_text")
public func appkit_text_input_discard_marked_text(_ handle: UnsafeMutableRawPointer, _ token: UInt64) -> Bool {
    guard Thread.isMainThread else { return false }
    let view: HostedView = appkitObject(handle)
    guard (view.textInputOwner as? TextInputCallbackOwner)?.token == token else { return false }
    return withExtendedLifetime(view) {
        var discarded = false
        guard appkit_try({
            if let context = view.inputContext {
                context.discardMarkedText()
                discarded = true
            }
        }) == nil else { return false }
        return discarded
    }
}

@_cdecl("appkit_text_input_invalidate_character_coordinates")
public func appkit_text_input_invalidate_character_coordinates(
    _ handle: UnsafeMutableRawPointer,
    _ token: UInt64
) -> Bool {
    guard Thread.isMainThread else { return false }
    let view: HostedView = appkitObject(handle)
    guard (view.textInputOwner as? TextInputCallbackOwner)?.token == token else { return false }
    return withExtendedLifetime(view) {
        var invalidated = false
        guard appkit_try({
            if let context = view.inputContext {
                context.invalidateCharacterCoordinates()
                invalidated = true
            }
        }) == nil else { return false }
        return invalidated
    }
}

@_cdecl("appkit_text_input_example_attributes_empty")
public func appkit_text_input_example_attributes_empty(_ handle: UnsafeMutableRawPointer) -> Bool {
    let view: HostedView = appkitObject(handle)
    return Thread.isMainThread && view.validAttributesForMarkedText().isEmpty
}

@_cdecl("appkit_text_input_example_protocol")
public func appkit_text_input_example_protocol(_ handle: UnsafeMutableRawPointer) -> Bool {
    guard Thread.isMainThread, let textProtocol = NSProtocolFromString("NSTextInputClient") else {
        return false
    }
    let view: HostedView = appkitObject(handle)
    let selectors = [
        "validAttributesForMarkedText", "hasMarkedText", "markedRange", "selectedRange",
        "firstRectForCharacterRange:actualRange:", "insertText:replacementRange:",
        "setMarkedText:selectedRange:replacementRange:", "unmarkText",
        "attributedSubstringForProposedRange:actualRange:", "characterIndexForPoint:",
        "doCommandBySelector:"
    ]
    return view.conforms(to: textProtocol) &&
        selectors.allSatisfy { view.responds(to: NSSelectorFromString($0)) }
}

@_cdecl("appkit_text_input_example_hidden")
public func appkit_text_input_example_hidden(_ handle: UnsafeMutableRawPointer) -> Bool {
    let view: HostedView = appkitObject(handle)
    return Thread.isMainThread && !(view.window?.isVisible ?? true)
}

@_cdecl("appkit_text_input_example_has_marked_text")
public func appkit_text_input_example_has_marked_text(_ handle: UnsafeMutableRawPointer) -> Bool {
    let view: HostedView = appkitObject(handle)
    return Thread.isMainThread && view.hasMarkedText()
}

@_cdecl("appkit_text_input_example_range")
public func appkit_text_input_example_range(
    _ handle: UnsafeMutableRawPointer,
    _ selector: UInt32,
    _ out: UnsafeMutablePointer<AppKitUtf16Range>
) -> Bool {
    guard Thread.isMainThread else { return false }
    let view: HostedView = appkitObject(handle)
    let range: NSRange
    switch selector {
    case 1: range = view.markedRange()
    case 2: range = view.selectedRange()
    default: return false
    }
    out.pointee = textRange(range)
    return true
}

@_cdecl("appkit_text_input_example_first_rect")
public func appkit_text_input_example_first_rect(
    _ handle: UnsafeMutableRawPointer,
    _ requested: UnsafePointer<AppKitUtf16Range>,
    _ actualOut: UnsafeMutablePointer<AppKitUtf16Range>?,
    _ rectOut: UnsafeMutablePointer<CGRect>
) -> Bool {
    guard Thread.isMainThread else { return false }
    let view: HostedView = appkitObject(handle)
    var actual = NSRange(location: 123, length: 1)
    rectOut.pointee = withUnsafeMutablePointer(to: &actual) { actualPointer in
        view.firstRect(
            forCharacterRange: NSRange(location: requested.pointee.location, length: requested.pointee.length),
            actualRange: actualOut == nil ? nil : actualPointer
        )
    }
    actualOut?.pointee = textRange(actual)
    return true
}

@_cdecl("appkit_text_input_example_attributed_substring")
public func appkit_text_input_example_attributed_substring(
    _ handle: UnsafeMutableRawPointer,
    _ requested: UnsafePointer<AppKitUtf16Range>,
    _ actualOut: UnsafeMutablePointer<AppKitUtf16Range>?,
    _ context: UnsafeMutableRawPointer?,
    _ copy: AppKitTextInputCopyUTF8?
) -> Bool {
    guard Thread.isMainThread else { return false }
    let view: HostedView = appkitObject(handle)
    var actual = NSRange(location: 123, length: 1)
    let attributed = withUnsafeMutablePointer(to: &actual) { actualPointer in
        view.attributedSubstring(
            forProposedRange: NSRange(location: requested.pointee.location, length: requested.pointee.length),
            actualRange: actualOut == nil ? nil : actualPointer
        )
    }
    actualOut?.pointee = textRange(actual)
    guard let attributed, let copy else { return false }
    let bytes = Array(attributed.string.utf8)
    return bytes.withUnsafeBufferPointer { buffer in
        copy(context, buffer.baseAddress, buffer.count)
    }
}

@_cdecl("appkit_text_input_example_character_index")
public func appkit_text_input_example_character_index(
    _ handle: UnsafeMutableRawPointer,
    _ x: Double,
    _ y: Double
) -> Int {
    let view: HostedView = appkitObject(handle)
    guard Thread.isMainThread else { return NSNotFound }
    return view.characterIndex(for: NSPoint(x: x, y: y))
}

private func textInputExampleString(_ bytes: UnsafePointer<UInt8>?, _ length: Int) -> String? {
    guard length >= 0, length == 0 || bytes != nil else { return nil }
    return String(bytes: UnsafeBufferPointer(start: bytes, count: length), encoding: .utf8)
}

@_cdecl("appkit_text_input_example_insert")
public func appkit_text_input_example_insert(
    _ handle: UnsafeMutableRawPointer,
    _ bytes: UnsafePointer<UInt8>?,
    _ length: Int,
    _ replacement: UnsafePointer<AppKitUtf16Range>,
    _ attributed: Bool
) -> Bool {
    guard Thread.isMainThread, let text = textInputExampleString(bytes, length) else { return false }
    let view: HostedView = appkitObject(handle)
    let range = NSRange(location: replacement.pointee.location, length: replacement.pointee.length)
    return appkit_try {
        if attributed {
            view.insertText(NSAttributedString(string: text), replacementRange: range)
        } else {
            view.insertText(text, replacementRange: range)
        }
    } == nil
}

@_cdecl("appkit_text_input_example_mark")
public func appkit_text_input_example_mark(
    _ handle: UnsafeMutableRawPointer,
    _ bytes: UnsafePointer<UInt8>?,
    _ length: Int,
    _ selection: UnsafePointer<AppKitUtf16Range>,
    _ replacement: UnsafePointer<AppKitUtf16Range>,
    _ attributed: Bool
) -> Bool {
    guard Thread.isMainThread, let text = textInputExampleString(bytes, length) else { return false }
    let view: HostedView = appkitObject(handle)
    let selected = NSRange(location: selection.pointee.location, length: selection.pointee.length)
    let replaced = NSRange(location: replacement.pointee.location, length: replacement.pointee.length)
    return appkit_try {
        if attributed {
            view.setMarkedText(NSAttributedString(string: text), selectedRange: selected, replacementRange: replaced)
        } else {
            view.setMarkedText(text, selectedRange: selected, replacementRange: replaced)
        }
    } == nil
}

@_cdecl("appkit_text_input_example_unmark")
public func appkit_text_input_example_unmark(_ handle: UnsafeMutableRawPointer) -> Bool {
    guard Thread.isMainThread else { return false }
    let view: HostedView = appkitObject(handle)
    return appkit_try { view.unmarkText() } == nil
}

@_cdecl("appkit_text_input_example_command")
public func appkit_text_input_example_command(
    _ handle: UnsafeMutableRawPointer,
    _ bytes: UnsafePointer<UInt8>?,
    _ length: Int
) -> Bool {
    guard Thread.isMainThread, let name = textInputExampleString(bytes, length) else { return false }
    let view: HostedView = appkitObject(handle)
    return appkit_try { view.doCommand(by: NSSelectorFromString(name)) } == nil
}

@_cdecl("appkit_text_input_example_has_context")
public func appkit_text_input_example_has_context(_ handle: UnsafeMutableRawPointer) -> Bool {
    guard Thread.isMainThread else { return false }
    let view: HostedView = appkitObject(handle)
    var available = false
    guard appkit_try({ available = view.inputContext != nil }) == nil else { return false }
    return available
}
