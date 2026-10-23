# Changelog

This file records the important changes to `appkit-rs`.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- The main-thread callback registry uses numeric tokens and weak state handles.
- Version 1 callback tables connect the Swift owner to borrowed events and copied UTF-8 text.
- `EventRef<'_>`, retained `Event`, `Utf16Range`, and `HandlerSlot` define the callback lifetime and re-entrant handler rules.
- Rust callback dispatch contains panics and gives a safe result for a late callback.

### Changed

- The Swift bridge requires macOS 12.0 or later.

## [0.1.1] - 2026-10-23

### Added

- `Window::new`, which makes a titled window that the user can close, minimize and resize.
- `Window::make_key_and_order_front` and `Window::close`.
- `ActivationPolicy`, and `Application::set_activation_policy`, `Application::activate`, `Application::run` and `Application::terminate`.
- The `03_window` example.

## [0.1.0] - 2026-10-23

### Added

- `MainThreadMarker`, which the functions that make or find main-thread objects need.
- `View` for `NSView`: the window, the bounds, the frame, the layer pointer and the layer setting.
- `Window` for `NSWindow`: the content view, the appearance, the title and its visibility, the titlebar separator style, the frame, the content layout rectangle, the backing scale factor, the screen, the alpha value with and without animation, the animation behavior and `orderFront:`.
- `Screen` for `NSScreen`: the main screen, all screens, the frames, the backing scale factor, the display identifier and the localized name.
- `Application` for `NSApplication`: the shared application, the application icon and the standard about panel.
- `Image` for `NSImage`: an image from data and its size.
- `Scroller::preferred_scroller_style` for `NSScroller`.
- `Workspace` for the accessibility display settings and the VoiceOver setting of `NSWorkspace`.
- `AppearanceName` for the standard `NSAppearance` names.
