import AppKit
import AppKitCallbackABI
import AppKitExceptionShim

final class AppKitCompletionOwner {
    let token: UInt64
    private let deactivate: @convention(c) (UInt64) -> Void
    private var completed = false

    init?(token: UInt64, deactivate: @escaping @convention(c) (UInt64) -> Void) {
        guard Thread.isMainThread, token != 0 else { return nil }
        self.token = token
        self.deactivate = deactivate
    }

    deinit {
        let token = token
        let deactivate = deactivate
        if Thread.isMainThread {
            deactivate(token)
        } else {
            DispatchQueue.main.async {
                deactivate(token)
            }
        }
    }

    func finish() -> Bool {
        guard Thread.isMainThread, !completed else { return false }
        completed = true
        return true
    }
}

private var panelOwners: [UInt64: AppKitPanelCompletionOwner] = [:]

private final class AppKitPanelCompletionOwner {
    let panel: NSSavePanel
    let callback: AppKitCompletionOwner
    let callbacks: AppKitPanelCallbacksV1
    private var cancelled = false

    init(panel: NSSavePanel, callback: AppKitCompletionOwner, callbacks: AppKitPanelCallbacksV1) {
        self.panel = panel
        self.callback = callback
        self.callbacks = callbacks
    }

    func begin() -> Bool {
        panelOwners[callback.token] = self
        guard appkit_try({
            panel.begin { [weak self] response in
                self?.nativeResponse(response)
            }
        }) == nil else {
            panelOwners.removeValue(forKey: callback.token)
            return false
        }
        return true
    }

    func nativeResponse(_ response: NSApplication.ModalResponse) {
        DispatchQueue.main.async { [self] in complete(response) }
    }

    func complete(_ response: NSApplication.ModalResponse) {
        guard callback.finish() else { return }
        withExtendedLifetime(self) {
            panelOwners.removeValue(forKey: callback.token)
            guard !cancelled, response == .OK, let paths = panelPaths(panel) else {
                callbacks.complete?(callback.token, false, nil, 0)
                return
            }
            var offsets: [Int] = []
            var bytes: [UInt8] = []
            for path in paths {
                offsets.append(bytes.count)
                bytes.append(contentsOf: path)
            }
            bytes.withUnsafeBufferPointer { data in
                let records = zip(paths, offsets).map { path, offset in
                    AppKitFileSystemPath(bytes: data.baseAddress?.advanced(by: offset), length: path.count)
                }
                records.withUnsafeBufferPointer { buffer in
                    callbacks.complete?(callback.token, true, buffer.baseAddress, buffer.count)
                }
            }
        }
    }

    func cancel() -> Bool {
        guard !cancelled else { return true }
        cancelled = true
        guard appkit_try({ panel.cancel(nil) }) == nil else {
            cancelled = false
            return false
        }
        DispatchQueue.main.async { [self] in complete(.cancel) }
        return true
    }
}

@_cdecl("appkit_open_panel_new")
public func appkit_open_panel_new() -> UnsafeMutableRawPointer {
    Unmanaged.passRetained(NSOpenPanel()).toOpaque()
}

@_cdecl("appkit_save_panel_new")
public func appkit_save_panel_new() -> UnsafeMutableRawPointer {
    Unmanaged.passRetained(NSSavePanel()).toOpaque()
}

@_cdecl("appkit_panel_set_directory")
public func appkit_panel_set_directory(
    _ handle: UnsafeMutableRawPointer,
    _ path: UnsafePointer<UInt8>?,
    _ length: Int
) -> Bool {
    let panel: NSSavePanel = appkitObject(handle)
    guard let path else {
        guard length == 0 else { return false }
        return appkit_try { panel.directoryURL = nil } == nil
    }
    guard length > 0, length < Int.max else { return false }
    let bytes = UnsafeBufferPointer(start: path, count: length)
    guard !bytes.contains(0), let directory = String(bytes: bytes, encoding: .utf8),
          !directory.isEmpty else { return false }
    var url: URL?
    guard appkit_try({ url = URL(fileURLWithPath: directory, isDirectory: true) }) == nil,
          let url else { return false }
    return appkit_try { panel.directoryURL = url } == nil
}

@_cdecl("appkit_panel_set_prompt")
public func appkit_panel_set_prompt(_ handle: UnsafeMutableRawPointer, _ prompt: UnsafePointer<CChar>?) -> Bool {
    let panel: NSSavePanel = appkitObject(handle)
    return appkit_try { panel.prompt = appkitString(prompt) } == nil
}

@_cdecl("appkit_panel_set_message")
public func appkit_panel_set_message(_ handle: UnsafeMutableRawPointer, _ message: UnsafePointer<CChar>?) -> Bool {
    let panel: NSSavePanel = appkitObject(handle)
    return appkit_try { panel.message = appkitString(message) } == nil
}

@_cdecl("appkit_panel_set_can_create_directories")
public func appkit_panel_set_can_create_directories(_ handle: UnsafeMutableRawPointer, _ canCreate: Bool) -> Bool {
    let panel: NSSavePanel = appkitObject(handle)
    return appkit_try { panel.canCreateDirectories = canCreate } == nil
}

@_cdecl("appkit_panel_set_shows_hidden_files")
public func appkit_panel_set_shows_hidden_files(_ handle: UnsafeMutableRawPointer, _ shows: Bool) -> Bool {
    let panel: NSSavePanel = appkitObject(handle)
    return appkit_try { panel.showsHiddenFiles = shows } == nil
}

@_cdecl("appkit_open_panel_set_can_choose_files")
public func appkit_open_panel_set_can_choose_files(_ handle: UnsafeMutableRawPointer, _ canChoose: Bool) -> Bool {
    let panel: NSOpenPanel = appkitObject(handle)
    return appkit_try { panel.canChooseFiles = canChoose } == nil
}

@_cdecl("appkit_open_panel_set_can_choose_directories")
public func appkit_open_panel_set_can_choose_directories(_ handle: UnsafeMutableRawPointer, _ canChoose: Bool) -> Bool {
    let panel: NSOpenPanel = appkitObject(handle)
    return appkit_try { panel.canChooseDirectories = canChoose } == nil
}

@_cdecl("appkit_open_panel_set_allows_multiple_selection")
public func appkit_open_panel_set_allows_multiple_selection(_ handle: UnsafeMutableRawPointer, _ allows: Bool) -> Bool {
    let panel: NSOpenPanel = appkitObject(handle)
    return appkit_try { panel.allowsMultipleSelection = allows } == nil
}

@_cdecl("appkit_open_panel_set_resolves_aliases")
public func appkit_open_panel_set_resolves_aliases(_ handle: UnsafeMutableRawPointer, _ resolves: Bool) -> Bool {
    let panel: NSOpenPanel = appkitObject(handle)
    return appkit_try { panel.resolvesAliases = resolves } == nil
}

@_cdecl("appkit_save_panel_set_suggested_name")
public func appkit_save_panel_set_suggested_name(_ handle: UnsafeMutableRawPointer, _ name: UnsafePointer<CChar>) -> Bool {
    let panel: NSSavePanel = appkitObject(handle)
    return appkit_try { panel.nameFieldStringValue = String(cString: name) } == nil
}

private func panelPaths(_ panel: NSSavePanel) -> [[UInt8]]? {
    var paths: [[UInt8]] = []
    let failure = appkit_try {
        let urls = (panel as? NSOpenPanel)?.urls ?? panel.url.map { [$0] } ?? []
        paths = urls.compactMap { url in
            guard url.isFileURL else { return nil }
            return url.withUnsafeFileSystemRepresentation { representation in
                guard let representation else { return nil }
                let bytes = UnsafeRawPointer(representation).assumingMemoryBound(to: UInt8.self)
                return Array(UnsafeBufferPointer(start: bytes, count: strlen(representation)))
            }
        }
    }
    return failure == nil ? paths : nil
}

@_cdecl("appkit_panel_begin")
public func appkit_panel_begin(
    _ handle: UnsafeMutableRawPointer,
    _ token: UInt64,
    _ callbacks: UnsafePointer<AppKitPanelCallbacksV1>?
) -> Bool {
    guard Thread.isMainThread,
          let callbacks = callbacks?.pointee,
          callbacks.header.version == 1,
          callbacks.header.size >= MemoryLayout<AppKitPanelCallbacksV1>.size,
          let deactivate = callbacks.deactivate,
          callbacks.complete != nil,
          let owner = AppKitCompletionOwner(token: token, deactivate: deactivate) else {
        return false
    }

    let panel: NSSavePanel = appkitObject(handle)
    return AppKitPanelCompletionOwner(panel: panel, callback: owner, callbacks: callbacks).begin()
}

@_cdecl("appkit_panel_cancel")
public func appkit_panel_cancel(_ token: UInt64) -> Bool {
    guard Thread.isMainThread, let operation = panelOwners[token] else { return false }
    return operation.cancel()
}

@_cdecl("appkit_panel_check_completion_after_drop")
public func appkit_panel_check_completion_after_drop(
    _ token: UInt64,
    _ callbacks: UnsafePointer<AppKitPanelCallbacksV1>?
) -> Bool {
    guard Thread.isMainThread,
          let callbacks = callbacks?.pointee,
          callbacks.header.version == 1,
          callbacks.header.size >= MemoryLayout<AppKitPanelCallbacksV1>.size,
          let deactivate = callbacks.deactivate,
          callbacks.complete != nil,
          let owner = AppKitCompletionOwner(token: token, deactivate: deactivate) else {
        return false
    }
    let operation: AppKitPanelCompletionOwner
    let wasHidden: Bool
    do {
        var panel: NSSavePanel?
        guard appkit_try({ panel = NSOpenPanel() }) == nil, let panel else { return false }
        operation = AppKitPanelCompletionOwner(panel: panel, callback: owner, callbacks: callbacks)
        wasHidden = !panel.isVisible
    }
    panelOwners[token] = operation
    operation.nativeResponse(.OK)
    guard operation.cancel() else {
        operation.complete(.cancel)
        return false
    }
    let deferred = panelOwners[token] != nil
    return wasHidden && deferred && !operation.panel.isVisible
}
