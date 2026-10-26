import AppKit
import AppKitCallbackABI

private final class AppKitAccessibilityDisplayObserver {
    let center = NSWorkspace.shared.notificationCenter
    var observer: NSObjectProtocol?

    init?(token: UInt64, table: AppKitWorkspaceAccessibilityDisplayTableV1) {
        guard Thread.isMainThread, token != 0, table.header.version == 1,
              table.header.size >= MemoryLayout<AppKitWorkspaceAccessibilityDisplayTableV1>.size,
              let changed = table.changed else {
            return nil
        }
        observer = center.addObserver(
            forName: NSWorkspace.accessibilityDisplayOptionsDidChangeNotification,
            object: nil, queue: nil
        ) { _ in
            DispatchQueue.main.async {
                changed(token)
            }
        }
    }

    deinit {
        if let observer {
            center.removeObserver(observer)
        }
    }
}

@_cdecl("appkit_workspace_accessibility_display_observer_new")
public func appkit_workspace_accessibility_display_observer_new(
    _ token: UInt64, _ table: UnsafePointer<AppKitWorkspaceAccessibilityDisplayTableV1>?
) -> UnsafeMutableRawPointer? {
    guard let table,
          let observer = AppKitAccessibilityDisplayObserver(token: token, table: table.pointee) else {
        return nil
    }
    return Unmanaged.passRetained(observer).toOpaque()
}

@_cdecl("appkit_scroller_preferred_scroller_style")
public func appkit_scroller_preferred_scroller_style() -> Int64 {
    Int64(NSScroller.preferredScrollerStyle.rawValue)
}

@_cdecl("appkit_workspace_accessibility_display_should_reduce_motion")
public func appkit_workspace_accessibility_display_should_reduce_motion() -> Bool {
    NSWorkspace.shared.accessibilityDisplayShouldReduceMotion
}

@_cdecl("appkit_workspace_accessibility_display_should_reduce_transparency")
public func appkit_workspace_accessibility_display_should_reduce_transparency() -> Bool {
    NSWorkspace.shared.accessibilityDisplayShouldReduceTransparency
}

@_cdecl("appkit_workspace_accessibility_display_should_increase_contrast")
public func appkit_workspace_accessibility_display_should_increase_contrast() -> Bool {
    NSWorkspace.shared.accessibilityDisplayShouldIncreaseContrast
}

@_cdecl("appkit_workspace_accessibility_display_should_differentiate_without_color")
public func appkit_workspace_accessibility_display_should_differentiate_without_color() -> Bool {
    NSWorkspace.shared.accessibilityDisplayShouldDifferentiateWithoutColor
}

@_cdecl("appkit_workspace_accessibility_display_should_invert_colors")
public func appkit_workspace_accessibility_display_should_invert_colors() -> Bool {
    NSWorkspace.shared.accessibilityDisplayShouldInvertColors
}

@_cdecl("appkit_workspace_is_voice_over_enabled")
public func appkit_workspace_is_voice_over_enabled() -> Bool {
    NSWorkspace.shared.isVoiceOverEnabled
}
