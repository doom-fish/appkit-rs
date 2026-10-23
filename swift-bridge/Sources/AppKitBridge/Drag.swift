import AppKit
import AppKitCallbackABI
import AppKitExceptionShim
import ObjectiveC
import UniformTypeIdentifiers

private enum DragPhase: UInt32 {
    case entered = 1
    case updated = 2
    case exited = 3
    case performed = 4
    case concluded = 5
}

private var dragOwnerKey: UInt8 = 0

private func dragOwner(_ window: NSWindow) -> DragCallbackOwner? {
    objc_getAssociatedObject(window, &dragOwnerKey) as? DragCallbackOwner
}

private final class DragCallbackOwner {
    let token: UInt64
    let table: AppKitDragCallbackTableV1
    weak var window: NSWindow?
    weak var view: HostedView?
    private var active = true
    private var paths: [[UInt8]]?
    private var pathsLoaded = false
    private var position: NSPoint = .zero
    private var sourceOperations: UInt64 = 0
    private var fromSourceWindow = false
    private var acceptedOperation: NSDragOperation = []
    private var mouseDown: NSEvent?

    init?(token: UInt64, table: AppKitDragCallbackTableV1, window: NSWindow, view: HostedView) {
        guard token != 0,
              table.header.version == 1,
              table.header.size >= MemoryLayout<AppKitDragCallbackTableV1>.size,
              table.deactivate != nil,
              table.destination != nil,
              table.source_mask != nil,
              table.source_end != nil else {
            return nil
        }
        self.token = token
        self.table = table
        self.window = window
        self.view = view
    }

    deinit {
        table.deactivate?(token)
    }

    func detach() {
        active = false
        mouseDown = nil
        paths = nil
        window = nil
        view = nil
    }

    var acceptsCallbacks: Bool {
        guard Thread.isMainThread, active,
              let window, let view, view.window === window,
              let host = window as? (NSWindow & HostedWindowOwner),
              !host.isClosing,
              host.callbackDelegate?.acceptsCallbacks == true else {
            return false
        }
        return true
    }

    private func readPaths(_ pasteboard: NSPasteboard) -> [[UInt8]]? {
        if let filenames = pasteboard.propertyList(
            forType: NSPasteboard.PasteboardType("NSFilenamesPboardType")
        ) as? [String] {
            return filenames.map { Array($0.utf8) }
        }
        guard let urls = pasteboard.readObjects(
            forClasses: [NSURL.self],
            options: [.urlReadingFileURLsOnly: true]
        ) as? [NSURL], !urls.isEmpty else {
            return nil
        }
        return urls.compactMap { url in
            guard url.isFileURL else { return nil }
            return (url as URL).withUnsafeFileSystemRepresentation { representation in
                guard let representation else { return nil }
                let bytes = UnsafeRawPointer(representation).assumingMemoryBound(to: UInt8.self)
                return Array(UnsafeBufferPointer(start: bytes, count: strlen(representation)))
            }
        }
    }

    private func load(_ sender: NSDraggingInfo, phase: DragPhase) -> Bool {
        guard let window else { return false }
        if phase == .entered {
            pathsLoaded = false
            paths = nil
            acceptedOperation = []
        }
        var location = NSPoint.zero
        var operations: UInt64 = 0
        var sourceWindow = false
        var loadedPaths: [[UInt8]]?
        guard appkit_try({
            location = sender.draggingLocation
            operations = UInt64(sender.draggingSourceOperationMask.rawValue)
            sourceWindow = (sender.draggingSource as AnyObject?) === window
            if !pathsLoaded {
                loadedPaths = readPaths(sender.draggingPasteboard)
            }
        }) == nil,
              location.x.isFinite,
              location.y.isFinite else {
            return false
        }
        position = location
        sourceOperations = operations
        fromSourceWindow = sourceWindow
        if !pathsLoaded {
            paths = loadedPaths
            pathsLoaded = true
        }
        return true
    }

    func destination(_ phase: DragPhase, _ sender: NSDraggingInfo?) -> NSDragOperation {
        guard acceptsCallbacks, let window, let view,
              let callback = table.destination else { return [] }
        if let sender, !load(sender, phase: phase) {
            return []
        }
        var storage: [NSData] = []
        var entries: [AppKitDragPath] = []
        guard appkit_try({
            storage = (paths ?? []).map { NSData(bytes: $0, length: $0.count) }
            entries = storage.map {
                AppKitDragPath(bytes: $0.bytes.assumingMemoryBound(to: UInt8.self), length: $0.length)
            }
        }) == nil else { return [] }
        var info = AppKitDragInfo(
            x: Double(position.x),
            y: Double(position.y),
            source_operation_mask: sourceOperations,
            is_source_window: fromSourceWindow,
            has_paths: paths != nil,
            paths: nil,
            path_count: entries.count
        )
        let offeredOperations = sourceOperations
        let raw = withExtendedLifetime((self, window, view, storage)) {
            entries.withUnsafeBufferPointer { buffer in
                info.paths = buffer.baseAddress
                return withUnsafePointer(to: &info) {
                    callback(token, phase.rawValue, $0)
                }
            }
        }
        let operation = NSDragOperation(rawValue: UInt(raw))
        switch phase {
        case .entered, .updated:
            acceptedOperation = operation.intersection(NSDragOperation(rawValue: UInt(offeredOperations)))
            return acceptsCallbacks ? acceptedOperation : []
        case .exited, .concluded:
            paths = nil
            pathsLoaded = false
            acceptedOperation = []
            return []
        case .performed:
            return operation
        }
    }

    func prepare() -> Bool {
        acceptsCallbacks && !acceptedOperation.isEmpty
    }

    func sourceMask(_ context: NSDraggingContext) -> NSDragOperation {
        guard acceptsCallbacks, let window, let view,
              let callback = table.source_mask else { return [] }
        return withExtendedLifetime((self, window, view)) {
            NSDragOperation(rawValue: UInt(callback(token, Int64(context.rawValue))))
        }
    }

    @discardableResult
    func sourceEnded(at point: NSPoint, operation: NSDragOperation) -> Bool {
        defer { mouseDown = nil }
        guard acceptsCallbacks, point.x.isFinite, point.y.isFinite,
              let window, let view,
              let callback = table.source_end else {
            return false
        }
        withExtendedLifetime((self, window, view)) {
            callback(token, Double(point.x), Double(point.y), UInt64(operation.rawValue))
        }
        return true
    }

    func makeItems(_ input: UnsafePointer<AppKitDragItem>?, count: Int, event: NSEvent) -> [NSDraggingItem] {
        guard count > 0, let input else { return [] }
        let location = event.locationInWindow
        guard location.x.isFinite, location.y.isFinite else { return [] }
        let frame = NSRect(x: location.x - 16, y: location.y - 16, width: 32, height: 32)
        var items: [NSDraggingItem] = []
        items.reserveCapacity(count)
        for value in UnsafeBufferPointer(start: input, count: count) {
            guard value.length >= 0, value.length == 0 || value.bytes != nil else { continue }
            let bytes = UnsafeBufferPointer(start: value.bytes, count: value.length)
            let writer: NSPasteboardWriting
            let iconType: String
            switch value.kind {
            case UInt32(APPKIT_DRAG_FILE):
                guard !bytes.isEmpty, !bytes.contains(0) else { continue }
                let representation = bytes.map { CChar(bitPattern: $0) } + [0]
                let url = representation.withUnsafeBufferPointer {
                    URL(fileURLWithFileSystemRepresentation: $0.baseAddress!, isDirectory: value.is_directory, relativeTo: nil)
                }
                writer = url as NSURL
                iconType = value.is_directory ? "public.folder" : (url.pathExtension.isEmpty ? "public.data" : url.pathExtension)
            case UInt32(APPKIT_DRAG_TEXT):
                guard let text = String(bytes: bytes, encoding: .utf8) else { continue }
                writer = text as NSString
                iconType = "public.plain-text"
            default:
                continue
            }
            let item = NSDraggingItem(pasteboardWriter: writer)
            item.draggingFrame = frame
            item.imageComponentsProvider = {
                var components: [NSDraggingImageComponent] = []
                _ = appkit_try {
                    let component = NSDraggingImageComponent(key: .icon)
                    let contentType = UTType(iconType) ?? UTType(filenameExtension: iconType) ?? .data
                    component.contents = NSWorkspace.shared.icon(for: contentType)
                    component.frame = NSRect(origin: .zero, size: frame.size)
                    components = [component]
                }
                return components
            }
            items.append(item)
        }
        return items
    }

    func start(_ view: HostedView, event: NSEvent, input: UnsafePointer<AppKitDragItem>?, count: Int) -> Bool {
        guard acceptsCallbacks, mouseDown == nil, event.type == .leftMouseDown,
              let window, event.window === window else {
            return false
        }
        var items: [NSDraggingItem] = []
        guard appkit_try({ items = makeItems(input, count: count, event: event) }) == nil,
              !items.isEmpty,
              let source = window as? NSDraggingSource else {
            return false
        }
        mouseDown = event
        var session: NSDraggingSession?
        guard appkit_try({
            session = view.beginDraggingSession(with: items, event: event, source: source)
        }) == nil, session != nil else {
            mouseDown = nil
            return false
        }
        return true
    }
}

private func entered(_ window: NSWindow, _ sender: NSDraggingInfo) -> NSDragOperation {
    dragOwner(window)?.destination(.entered, sender) ?? []
}

private func updated(_ window: NSWindow, _ sender: NSDraggingInfo) -> NSDragOperation {
    dragOwner(window)?.destination(.updated, sender) ?? []
}

private func exited(_ window: NSWindow, _ sender: NSDraggingInfo?) {
    _ = dragOwner(window)?.destination(.exited, sender)
}

private func performed(_ window: NSWindow, _ sender: NSDraggingInfo) -> Bool {
    !(dragOwner(window)?.destination(.performed, sender) ?? []).isEmpty
}

private func concluded(_ window: NSWindow, _ sender: NSDraggingInfo?) {
    _ = dragOwner(window)?.destination(.concluded, sender)
}

extension HostedWindow: NSDraggingDestination, NSDraggingSource {
    func draggingEntered(_ sender: NSDraggingInfo) -> NSDragOperation { entered(self, sender) }
    func draggingUpdated(_ sender: NSDraggingInfo) -> NSDragOperation { updated(self, sender) }
    func draggingExited(_ sender: NSDraggingInfo?) { exited(self, sender) }
    func prepareForDragOperation(_ sender: NSDraggingInfo) -> Bool { dragOwner(self)?.prepare() ?? false }
    func performDragOperation(_ sender: NSDraggingInfo) -> Bool { performed(self, sender) }
    func concludeDragOperation(_ sender: NSDraggingInfo?) { concluded(self, sender) }
    func wantsPeriodicDraggingUpdates() -> Bool { true }

    func draggingSession(_ session: NSDraggingSession, sourceOperationMaskFor context: NSDraggingContext) -> NSDragOperation {
        dragOwner(self)?.sourceMask(context) ?? []
    }

    func draggingSession(_ session: NSDraggingSession, endedAt screenPoint: NSPoint, operation: NSDragOperation) {
        dragOwner(self)?.sourceEnded(at: screenPoint, operation: operation)
    }
}

extension HostedPanel: NSDraggingDestination, NSDraggingSource {
    func draggingEntered(_ sender: NSDraggingInfo) -> NSDragOperation { entered(self, sender) }
    func draggingUpdated(_ sender: NSDraggingInfo) -> NSDragOperation { updated(self, sender) }
    func draggingExited(_ sender: NSDraggingInfo?) { exited(self, sender) }
    func prepareForDragOperation(_ sender: NSDraggingInfo) -> Bool { dragOwner(self)?.prepare() ?? false }
    func performDragOperation(_ sender: NSDraggingInfo) -> Bool { performed(self, sender) }
    func concludeDragOperation(_ sender: NSDraggingInfo?) { concluded(self, sender) }
    func wantsPeriodicDraggingUpdates() -> Bool { true }

    func draggingSession(_ session: NSDraggingSession, sourceOperationMaskFor context: NSDraggingContext) -> NSDragOperation {
        dragOwner(self)?.sourceMask(context) ?? []
    }

    func draggingSession(_ session: NSDraggingSession, endedAt screenPoint: NSPoint, operation: NSDragOperation) {
        dragOwner(self)?.sourceEnded(at: screenPoint, operation: operation)
    }
}

@_cdecl("appkit_drag_attach")
public func appkit_drag_attach(
    _ windowHandle: UnsafeMutableRawPointer?,
    _ viewHandle: UnsafeMutableRawPointer?,
    _ token: UInt64,
    _ table: UnsafePointer<AppKitDragCallbackTableV1>?
) -> Bool {
    guard Thread.isMainThread, let windowHandle, let viewHandle, let table else { return false }
    let window: NSWindow = appkitObject(windowHandle)
    let view: NSView = appkitObject(viewHandle)
    guard window is HostedWindow || window is HostedPanel,
          let hostedView = view as? HostedView,
          hostedView.window === window,
          dragOwner(window) == nil,
          let owner = DragCallbackOwner(token: token, table: table.pointee, window: window, view: hostedView),
          appkit_try({
              window.registerForDraggedTypes([.fileURL, NSPasteboard.PasteboardType("NSFilenamesPboardType")])
          }) == nil else {
        return false
    }
    objc_setAssociatedObject(window, &dragOwnerKey, owner, .OBJC_ASSOCIATION_RETAIN_NONATOMIC)
    return true
}

@_cdecl("appkit_drag_detach")
public func appkit_drag_detach(_ windowHandle: UnsafeMutableRawPointer?) {
    guard Thread.isMainThread, let windowHandle else { return }
    let window: NSWindow = appkitObject(windowHandle)
    guard let owner = dragOwner(window) else { return }
    owner.detach()
    objc_setAssociatedObject(window, &dragOwnerKey, nil, .OBJC_ASSOCIATION_RETAIN_NONATOMIC)
    _ = appkit_try { window.unregisterDraggedTypes() }
}

@_cdecl("appkit_drag_start")
public func appkit_drag_start(
    _ windowHandle: UnsafeMutableRawPointer?,
    _ viewHandle: UnsafeMutableRawPointer?,
    _ eventHandle: UnsafeMutableRawPointer?,
    _ input: UnsafePointer<AppKitDragItem>?,
    _ count: Int
) -> Bool {
    guard Thread.isMainThread, let windowHandle, let viewHandle, let eventHandle else { return false }
    let window: NSWindow = appkitObject(windowHandle)
    let view: HostedView = appkitObject(viewHandle)
    let event: NSEvent = appkitObject(eventHandle)
    guard let owner = dragOwner(window), owner.view === view else { return false }
    return owner.start(view, event: event, input: input, count: count)
}

@_cdecl("appkit_drag_verify_items")
public func appkit_drag_verify_items(
    _ windowHandle: UnsafeMutableRawPointer?,
    _ viewHandle: UnsafeMutableRawPointer?,
    _ eventHandle: UnsafeMutableRawPointer?,
    _ input: UnsafePointer<AppKitDragItem>?,
    _ count: Int
) -> Int {
    guard Thread.isMainThread, let windowHandle, let viewHandle, let eventHandle else { return 0 }
    let window: NSWindow = appkitObject(windowHandle)
    let view: HostedView = appkitObject(viewHandle)
    let event: NSEvent = appkitObject(eventHandle)
    guard let owner = dragOwner(window), owner.view === view, owner.acceptsCallbacks else { return 0 }
    var items: [NSDraggingItem] = []
    guard appkit_try({ items = owner.makeItems(input, count: count, event: event) }) == nil else { return 0 }
    return items.count
}

private final class DragTestInfo: NSObject, NSDraggingInfo {
    let draggingDestinationWindow: NSWindow?
    let draggingSourceOperationMask: NSDragOperation
    let draggingLocation: NSPoint
    let draggedImageLocation: NSPoint
    let draggedImage: NSImage? = nil
    let draggingPasteboard: NSPasteboard
    let draggingSource: Any?
    let draggingSequenceNumber = 1
    var draggingFormation: NSDraggingFormation = .none
    var animatesToDestination = false
    var numberOfValidItemsForDrop = 0
    var springLoadingHighlight: NSSpringLoadingHighlight { .none }

    init(window: NSWindow, pasteboard: NSPasteboard, position: NSPoint, operations: NSDragOperation, sameWindow: Bool) {
        draggingDestinationWindow = window
        draggingSourceOperationMask = operations
        draggingLocation = position
        draggedImageLocation = position
        draggingPasteboard = pasteboard
        draggingSource = sameWindow ? window : nil
    }

    func slideDraggedImage(to screenPoint: NSPoint) {}

    override func namesOfPromisedFilesDropped(atDestination dropDestination: URL) -> [String]? {
        nil
    }

    func enumerateDraggingItems(
        options enumOpts: NSDraggingItemEnumerationOptions,
        for view: NSView?,
        classes classArray: [AnyClass],
        searchOptions: [NSPasteboard.ReadingOptionKey: Any],
        using block: @escaping (NSDraggingItem, Int, UnsafeMutablePointer<ObjCBool>) -> Void
    ) {}

    func resetSpringLoading() {}
}

@_cdecl("appkit_drag_verify_destination")
public func appkit_drag_verify_destination(
    _ windowHandle: UnsafeMutableRawPointer?,
    _ pasteboardHandle: UnsafeMutableRawPointer?,
    _ x: Double,
    _ y: Double,
    _ sourceOperations: UInt64,
    _ sameWindow: Bool,
    _ phase: UInt32
) -> UInt64 {
    guard Thread.isMainThread, let windowHandle, let pasteboardHandle,
          x.isFinite, y.isFinite else {
        return 0
    }
    let window: NSWindow = appkitObject(windowHandle)
    let pasteboard: NSPasteboard = appkitObject(pasteboardHandle)
    guard let destination = window as? NSDraggingDestination else { return 0 }
    let info = DragTestInfo(
        window: window,
        pasteboard: pasteboard,
        position: NSPoint(x: x, y: y),
        operations: NSDragOperation(rawValue: UInt(sourceOperations)),
        sameWindow: sameWindow
    )
    var result: UInt64 = 0
    guard appkit_try({
        switch phase {
        case 1: result = UInt64(destination.draggingEntered?(info).rawValue ?? 0)
        case 2: result = UInt64(destination.draggingUpdated?(info).rawValue ?? 0)
        case 3: destination.draggingExited?(info)
        case 4: result = destination.prepareForDragOperation?(info) == true ? 1 : 0
        case 5: result = destination.performDragOperation?(info) == true ? 1 : 0
        case 6: destination.concludeDragOperation?(info)
        default: break
        }
    }) == nil else {
        return 0
    }
    return result
}

@_cdecl("appkit_drag_verify_source_mask")
public func appkit_drag_verify_source_mask(_ windowHandle: UnsafeMutableRawPointer?, _ context: Int64) -> UInt64 {
    guard Thread.isMainThread, let windowHandle,
          let draggingContext = NSDraggingContext(rawValue: Int(context)) else {
        return 0
    }
    let window: NSWindow = appkitObject(windowHandle)
    return UInt64(dragOwner(window)?.sourceMask(draggingContext).rawValue ?? 0)
}

@_cdecl("appkit_drag_verify_source_end")
public func appkit_drag_verify_source_end(
    _ windowHandle: UnsafeMutableRawPointer?,
    _ x: Double,
    _ y: Double,
    _ operation: UInt64
) -> Bool {
    guard Thread.isMainThread, let windowHandle else { return false }
    let window: NSWindow = appkitObject(windowHandle)
    return dragOwner(window)?.sourceEnded(
        at: NSPoint(x: x, y: y),
        operation: NSDragOperation(rawValue: UInt(operation))
    ) ?? false
}
