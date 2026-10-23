# Changelog

This file records the important changes to `appkit-rs`.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `HostedWindow::deactivate_callbacks` and `HostedView::deactivate_callbacks` stop all callbacks to Rust without detaching or releasing the native object, so an owner can tear down in steps. A second call does nothing.
- `HostedView` is an `NSView` that hosts a renderer's `CAMetalLayer`. It forwards key, mouse, scroll, pressure, magnify and swipe events, display, size, backing-property and appearance changes, cursor rectangles and mouse tracking through token callbacks, and it can own titlebar drags.
- `Menu` and `MenuItem` build menus with responder actions, key equivalents, check states, submenus and application menu roles (main, Services, Windows and Help), and give a retained Dock menu to the application host. A submenu that would contain its own menu is refused.
- A unique pasteboard gets its own UUID name, and pasteboard calls return a failure value instead of a native exception.
- `Window::encode_restorable_state` saves the secure native state of a window, and `Window::restore_state` restores it. Invalid data gives an error. Restore does not guarantee the original Space.
- Window operations: typed style masks, levels and collection behaviours, geometry and screen conversions, key and visibility state, full screen and simple full screen, sheets, tabs, traffic-light buttons, background, shadow, document state and ordering. Setters return an error for values that AppKit rejects. A sheet completion runs after the caller drops its handle, and closing or ordering out a window aborts its sheet.
- `Workspace` opens URLs and files, reveals paths in Finder from any thread, reads file and application icons, observes sleep and wake, and registers the app for a URL scheme. A dropped registration handle still delivers its completion.
- `Application` reads and changes key, main and ordered windows, activation, visibility, presentation options, user attention and the app appearance. Invalid presentation options give an error.
- `Appearance` converts appearance names, including the four accessibility names that only match, and sets or clears view overrides.
- `VisualEffectView` attaches a standard or a clear blur below a hosted view without a strong reference to its window.
- `ApplicationHost` installs an application delegate on the shared `NSApplication`. It forwards launch, reopen, quit, URL, Dock menu, screen and menu-item events through token callbacks. Reopen defaults to YES, and a quit request from inside a callback gets an answer at once.
- `HostedWindow` makes `NSWindow` and `NSPanel` subclasses with a retained delegate and token callbacks for close, focus, movement, size, screen, occlusion, full-screen, tab and titlebar events. A window that closes and shows again keeps its callbacks.
- `Screen::maximum_frames_per_second` reads the highest refresh rate of a screen, for example 120 on a ProMotion display.
- `EventRef` reads the event type, the modifier flags, positions, mouse buttons, key codes, characters, scroll and swipe deltas, phases, pressure and magnification. A field that the event type does not have gives `None`.
- `Event::current`, the class queries for modifier flags and pressed mouse buttons, and synthetic key and mouse events.
- `Pasteboard` reads and writes the general, find and unique pasteboards: types, data, strings, legacy file names and the change count. File names keep their UTF-8 bytes.
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
