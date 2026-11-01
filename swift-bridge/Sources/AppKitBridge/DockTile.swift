import AppKit
import AppKitExceptionShim

private enum AppKitDockProgress {
    case fraction(Double)
    case busy
}

private final class AppKitDockProgressView: NSView {
    let previousContentView: NSView?
    var progress: AppKitDockProgress

    init(frame: NSRect, previousContentView: NSView?, progress: AppKitDockProgress) {
        self.previousContentView = previousContentView
        self.progress = progress
        super.init(frame: frame)
    }

    required init?(coder: NSCoder) {
        return nil
    }

    override func draw(_ dirtyRect: NSRect) {
        guard bounds.width > 0, bounds.height > 0 else { return }
        let icon = NSApplication.shared.applicationIconImage
            ?? NSImage(named: NSImage.Name("NSApplicationIcon"))
        icon?.draw(in: bounds, from: .zero, operation: .sourceOver, fraction: 1)

        let bar = NSRect(
            x: bounds.width * 0.12,
            y: bounds.height * 0.10,
            width: bounds.width * 0.76,
            height: bounds.height * 0.10
        )
        let track = NSBezierPath(roundedRect: bar, xRadius: bar.height / 2, yRadius: bar.height / 2)
        NSColor.black.withAlphaComponent(0.72).setFill()
        track.fill()
        NSColor.white.withAlphaComponent(0.9).setStroke()
        track.lineWidth = bar.height * 0.1
        track.stroke()

        let interior = bar.insetBy(dx: bar.height * 0.15, dy: bar.height * 0.15)
        var fill = interior
        switch progress {
        case .fraction(let value):
            fill.size.width *= value
        case .busy:
            fill.size.width *= 0.35
            fill.origin.x += (interior.width - fill.width) / 2
        }
        guard fill.width > 0 else { return }
        NSColor.controlAccentColor.setFill()
        NSBezierPath(
            roundedRect: fill,
            xRadius: fill.height / 2,
            yRadius: fill.height / 2
        ).fill()
    }
}

@_cdecl("appkit_application_dock_tile")
public func appkit_application_dock_tile(_ handle: UnsafeMutableRawPointer) -> UnsafeMutableRawPointer {
    let application: NSApplication = appkitObject(handle)
    return Unmanaged.passRetained(application.dockTile).toOpaque()
}

@_cdecl("appkit_dock_tile_set_progress")
public func appkit_dock_tile_set_progress(
    _ handle: UnsafeMutableRawPointer,
    _ kind: Int32,
    _ fraction: Double
) -> Bool {
    guard Thread.isMainThread else { return false }
    let progress: AppKitDockProgress?
    switch kind {
    case 0:
        progress = nil
    case 1:
        guard fraction.isFinite, (0...1).contains(fraction) else { return false }
        progress = .fraction(fraction)
    case 2:
        progress = .busy
    default:
        return false
    }
    let tile: NSDockTile = appkitObject(handle)
    return appkit_try({
        if let progress {
            let frame = NSRect(origin: .zero, size: tile.size)
            if let view = tile.contentView as? AppKitDockProgressView {
                view.progress = progress
                view.frame = frame
                view.needsDisplay = true
            } else {
                tile.contentView = AppKitDockProgressView(
                    frame: frame,
                    previousContentView: tile.contentView,
                    progress: progress
                )
            }
        } else if let view = tile.contentView as? AppKitDockProgressView {
            tile.contentView = view.previousContentView
        }
    }) == nil
}

@_cdecl("appkit_dock_tile_set_badge_label")
public func appkit_dock_tile_set_badge_label(
    _ handle: UnsafeMutableRawPointer,
    _ label: UnsafePointer<CChar>?
) -> Bool {
    guard Thread.isMainThread else { return false }
    let tile: NSDockTile = appkitObject(handle)
    return appkit_try({ tile.badgeLabel = appkitString(label) }) == nil
}

@_cdecl("appkit_dock_tile_badge_label")
public func appkit_dock_tile_badge_label(_ handle: UnsafeMutableRawPointer) -> UnsafeMutablePointer<CChar>? {
    guard Thread.isMainThread else { return nil }
    let tile: NSDockTile = appkitObject(handle)
    var label: String?
    guard appkit_try({ label = tile.badgeLabel }) == nil, let label else { return nil }
    return strdup(label)
}

@_cdecl("appkit_dock_tile_display")
public func appkit_dock_tile_display(_ handle: UnsafeMutableRawPointer) {
    guard Thread.isMainThread else { return }
    let tile: NSDockTile = appkitObject(handle)
    _ = appkit_try { tile.display() }
}
