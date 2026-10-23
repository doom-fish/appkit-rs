import AppKit
import AppKitCallbackABI

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
        panel.directoryURL = nil
        return true
    }
    guard length > 0, length < Int.max else { return false }
    var bytes = Array(UnsafeBufferPointer(start: path, count: length))
    guard !bytes.contains(0) else { return false }
    bytes.append(0)
    panel.directoryURL = bytes.withUnsafeBufferPointer { buffer in
        buffer.baseAddress!.withMemoryRebound(to: CChar.self, capacity: buffer.count) { representation in
            URL(fileURLWithFileSystemRepresentation: representation, isDirectory: true, relativeTo: nil)
        }
    }
    return true
}

@_cdecl("appkit_panel_set_prompt")
public func appkit_panel_set_prompt(_ handle: UnsafeMutableRawPointer, _ prompt: UnsafePointer<CChar>?) {
    let panel: NSSavePanel = appkitObject(handle)
    panel.prompt = appkitString(prompt)
}

@_cdecl("appkit_panel_set_message")
public func appkit_panel_set_message(_ handle: UnsafeMutableRawPointer, _ message: UnsafePointer<CChar>?) {
    let panel: NSSavePanel = appkitObject(handle)
    panel.message = appkitString(message)
}

@_cdecl("appkit_panel_set_can_create_directories")
public func appkit_panel_set_can_create_directories(_ handle: UnsafeMutableRawPointer, _ canCreate: Bool) {
    let panel: NSSavePanel = appkitObject(handle)
    panel.canCreateDirectories = canCreate
}

@_cdecl("appkit_panel_set_shows_hidden_files")
public func appkit_panel_set_shows_hidden_files(_ handle: UnsafeMutableRawPointer, _ shows: Bool) {
    let panel: NSSavePanel = appkitObject(handle)
    panel.showsHiddenFiles = shows
}

@_cdecl("appkit_open_panel_set_can_choose_files")
public func appkit_open_panel_set_can_choose_files(_ handle: UnsafeMutableRawPointer, _ canChoose: Bool) {
    let panel: NSOpenPanel = appkitObject(handle)
    panel.canChooseFiles = canChoose
}

@_cdecl("appkit_open_panel_set_can_choose_directories")
public func appkit_open_panel_set_can_choose_directories(_ handle: UnsafeMutableRawPointer, _ canChoose: Bool) {
    let panel: NSOpenPanel = appkitObject(handle)
    panel.canChooseDirectories = canChoose
}

@_cdecl("appkit_open_panel_set_allows_multiple_selection")
public func appkit_open_panel_set_allows_multiple_selection(_ handle: UnsafeMutableRawPointer, _ allows: Bool) {
    let panel: NSOpenPanel = appkitObject(handle)
    panel.allowsMultipleSelection = allows
}

@_cdecl("appkit_open_panel_set_resolves_aliases")
public func appkit_open_panel_set_resolves_aliases(_ handle: UnsafeMutableRawPointer, _ resolves: Bool) {
    let panel: NSOpenPanel = appkitObject(handle)
    panel.resolvesAliases = resolves
}

@_cdecl("appkit_save_panel_set_suggested_name")
public func appkit_save_panel_set_suggested_name(_ handle: UnsafeMutableRawPointer, _ name: UnsafePointer<CChar>) {
    let panel: NSSavePanel = appkitObject(handle)
    panel.nameFieldStringValue = String(cString: name)
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
          let complete = callbacks.complete,
          let owner = AppKitCompletionOwner(token: token, deactivate: deactivate) else {
        return false
    }

    let panel: NSSavePanel = appkitObject(handle)
    panel.begin { [panel, owner, complete] response in
        let deliver = {
            guard owner.finish() else { return }
            guard response == .OK else {
                complete(token, false, nil, 0)
                return
            }
            let urls = (panel as? NSOpenPanel)?.urls ?? panel.url.map { [$0] } ?? []
            let representations: [UnsafeMutablePointer<CChar>] = urls.compactMap { url in
                guard url.isFileURL else { return nil }
                return url.withUnsafeFileSystemRepresentation { representation in
                    representation.flatMap { strdup($0) }
                }
            }
            defer {
                for representation in representations {
                    free(representation)
                }
            }
            let paths = representations.map { representation in
                AppKitFileSystemPath(
                    bytes: UnsafeRawPointer(representation).assumingMemoryBound(to: UInt8.self),
                    length: strlen(representation)
                )
            }
            paths.withUnsafeBufferPointer { buffer in
                complete(token, true, buffer.baseAddress, buffer.count)
            }
        }
        if Thread.isMainThread {
            deliver()
        } else {
            DispatchQueue.main.async(execute: deliver)
        }
    }
    return true
}
