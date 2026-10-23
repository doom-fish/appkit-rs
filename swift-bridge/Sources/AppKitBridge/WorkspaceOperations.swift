import AppKit
import AppKitCallbackABI
import UniformTypeIdentifiers

@_cdecl("appkit_workspace_open_url")
public func appkit_workspace_open_url(_ url: UnsafePointer<CChar>) -> Bool {
    guard let url = URL(string: String(cString: url)) else { return false }
    return NSWorkspace.shared.open(url)
}

@_cdecl("appkit_workspace_open_file")
public func appkit_workspace_open_file(_ path: UnsafePointer<CChar>) -> Bool {
    NSWorkspace.shared.open(
        URL(fileURLWithFileSystemRepresentation: path, isDirectory: false, relativeTo: nil)
    )
}

@_cdecl("appkit_workspace_reveal_paths")
public func appkit_workspace_reveal_paths(
    _ paths: UnsafePointer<AppKitWorkspacePath>?,
    _ count: Int
) {
    guard let paths, count > 0 else { return }
    let urls = UnsafeBufferPointer(start: paths, count: count).compactMap { path in
        path.bytes.map {
            URL(fileURLWithFileSystemRepresentation: $0, isDirectory: false, relativeTo: nil)
        }
    }
    NSWorkspace.shared.activateFileViewerSelecting(urls)
}

@_cdecl("appkit_workspace_icon_for_file")
public func appkit_workspace_icon_for_file(_ path: UnsafePointer<CChar>) -> UnsafeMutableRawPointer? {
    let url = URL(fileURLWithFileSystemRepresentation: path, isDirectory: false, relativeTo: nil)
    return appkitRetained(NSWorkspace.shared.icon(forFile: url.path))
}

@_cdecl("appkit_workspace_icon_for_file_type")
public func appkit_workspace_icon_for_file_type(_ fileType: UnsafePointer<CChar>) -> UnsafeMutableRawPointer? {
    let fileType = String(cString: fileType)
    let contentType = fileType.contains(".")
        ? UTType(fileType)
        : UTType(filenameExtension: fileType)
    return appkitRetained(NSWorkspace.shared.icon(for: contentType ?? .data))
}

@_cdecl("appkit_workspace_icon_for_application")
public func appkit_workspace_icon_for_application(
    _ bundleIdentifier: UnsafePointer<CChar>
) -> UnsafeMutableRawPointer? {
    guard let url = NSWorkspace.shared.urlForApplication(
        withBundleIdentifier: String(cString: bundleIdentifier)
    ) else {
        return nil
    }
    return appkitRetained(NSWorkspace.shared.icon(forFile: url.path))
}

private enum AppKitWorkspacePowerKind: Int32 {
    case willSleep
    case didWake
}

private final class AppKitWorkspacePowerObserver {
    let token: UInt64
    let table: AppKitWorkspacePowerTableV1
    let center: NotificationCenter
    var observers: [NSObjectProtocol] = []

    init?(token: UInt64, table: AppKitWorkspacePowerTableV1) {
        guard Thread.isMainThread, token != 0, table.header.version == 1,
              table.header.size >= MemoryLayout<AppKitWorkspacePowerTableV1>.size,
              table.event != nil else {
            return nil
        }
        self.token = token
        self.table = table
        self.center = NSWorkspace.shared.notificationCenter
        observers.append(center.addObserver(
            forName: NSWorkspace.willSleepNotification, object: nil, queue: nil
        ) { [weak self] _ in self?.notify(.willSleep) })
        observers.append(center.addObserver(
            forName: NSWorkspace.didWakeNotification, object: nil, queue: nil
        ) { [weak self] _ in self?.notify(.didWake) })
    }

    deinit {
        for observer in observers {
            center.removeObserver(observer)
        }
    }

    func notify(_ kind: AppKitWorkspacePowerKind) {
        if Thread.isMainThread {
            table.event?(token, kind.rawValue)
        } else {
            DispatchQueue.main.async { [weak self] in self?.notify(kind) }
        }
    }
}

@_cdecl("appkit_workspace_power_observer_new")
public func appkit_workspace_power_observer_new(
    _ token: UInt64, _ table: UnsafePointer<AppKitWorkspacePowerTableV1>?
) -> UnsafeMutableRawPointer? {
    guard let table, let observer = AppKitWorkspacePowerObserver(token: token, table: table.pointee) else {
        return nil
    }
    return Unmanaged.passRetained(observer).toOpaque()
}

@_cdecl("appkit_workspace_power_observer_notify")
public func appkit_workspace_power_observer_notify(
    _ handle: UnsafeMutableRawPointer, _ kind: Int32
) {
    guard Thread.isMainThread, let kind = AppKitWorkspacePowerKind(rawValue: kind) else { return }
    let observer: AppKitWorkspacePowerObserver = appkitObject(handle)
    withExtendedLifetime(observer) {
        observer.notify(kind)
    }
}

private enum AppKitWorkspaceSchemeStatus: Int32 {
    case success
    case unsupportedSystem
    case missingBundleIdentifier
    case applicationNotInstalled
    case nativeFailure
}

private final class AppKitWorkspaceSchemeOperation {
    let token: UInt64
    let table: AppKitWorkspaceSchemeTableV1
    var completed = false

    init?(token: UInt64, table: AppKitWorkspaceSchemeTableV1) {
        guard Thread.isMainThread, token != 0, table.header.version == 1,
              table.header.size >= MemoryLayout<AppKitWorkspaceSchemeTableV1>.size,
              table.complete != nil else {
            return nil
        }
        self.token = token
        self.table = table
    }

    func complete(_ error: Error?) {
        if !Thread.isMainThread {
            let message = error?.localizedDescription
            DispatchQueue.main.async { [self] in completeMessage(message) }
            return
        }
        completeMessage(error?.localizedDescription)
    }

    private func completeMessage(_ message: String?) {
        guard !completed else { return }
        completed = true
        guard let message else {
            table.complete?(token, true, nil, 0)
            return
        }
        Array(message.utf8).withUnsafeBufferPointer { bytes in
            table.complete?(token, false, bytes.baseAddress, bytes.count)
        }
    }
}

@_cdecl("appkit_workspace_register_url_scheme")
public func appkit_workspace_register_url_scheme(
    _ scheme: UnsafePointer<CChar>,
    _ token: UInt64,
    _ table: UnsafePointer<AppKitWorkspaceSchemeTableV1>?,
    _ status: UnsafeMutablePointer<Int32>
) -> UnsafeMutableRawPointer? {
    status.pointee = AppKitWorkspaceSchemeStatus.nativeFailure.rawValue
    guard Thread.isMainThread, let table,
          let operation = AppKitWorkspaceSchemeOperation(token: token, table: table.pointee) else {
        return nil
    }
    guard #available(macOS 12.0, *) else {
        status.pointee = AppKitWorkspaceSchemeStatus.unsupportedSystem.rawValue
        return nil
    }
    guard let bundleIdentifier = Bundle.main.bundleIdentifier else {
        status.pointee = AppKitWorkspaceSchemeStatus.missingBundleIdentifier.rawValue
        return nil
    }
    guard let appURL = NSWorkspace.shared.urlForApplication(
        withBundleIdentifier: bundleIdentifier
    ) else {
        status.pointee = AppKitWorkspaceSchemeStatus.applicationNotInstalled.rawValue
        return nil
    }
    NSWorkspace.shared.setDefaultApplication(
        at: appURL, toOpenURLsWithScheme: String(cString: scheme)
    ) { [operation] error in
        operation.complete(error)
    }
    status.pointee = AppKitWorkspaceSchemeStatus.success.rawValue
    return Unmanaged.passRetained(operation).toOpaque()
}
