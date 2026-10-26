# appkit-rs

Safe Rust bindings for Apple's [AppKit](https://developer.apple.com/documentation/appkit) framework on macOS.
The crate creates and hosts windows and views, and it runs the event loop.
It gives access to menus, dialogs, cursors, pasteboards, text input, drag operations, screens and workspace services.
It also forwards application and window callbacks and saves secure window state.

The crate requires macOS 12.0 or later.

## Accessibility display options

Read the current options with the `Workspace::accessibility_display_should_*` methods.
Keep a `workspace::AccessibilityDisplayObserver` from `Workspace::observe_accessibility_display` to receive changes on the main thread.
Read the options again in the callback.
Drop the observer to remove the registration and cancel queued Rust callbacks.

## Quick start

```rust,no_run
use appkit::prelude::*;

fn main() {
    let main_thread = MainThreadMarker::new().expect("main thread");
    if Workspace::accessibility_display_should_reduce_motion() {
        println!("the user prefers less motion");
    }
    for screen in Screen::screens(main_thread) {
        println!("{}: {:?}", screen.localized_name(), screen.display_id());
    }
    let application = Application::shared(main_thread);
    application
        .order_front_standard_about_panel(&AboutPanelOptions {
            application_name: Some("Example".to_owned()),
            ..AboutPanelOptions::default()
        })
        .unwrap();
}
```

## The main thread

Use `AppKit` UI objects on the main thread.

- `MainThreadMarker::new` gives a marker only on the main thread. Constructors and callbacks for hosted UI objects need a marker.
- Object handles are not `Send` or `Sync`. Keep each handle on its creation thread.
- `Image::from_data` and the accessibility queries on `Workspace` need no marker. `Workspace::reveal_paths` also accepts calls from other threads.
- Other `Workspace` operations, such as `open_url` and `observe_power`, need a marker.

## Windows and views from other code

A window library, for example one that gives a `raw-window-handle`, can supply an `NSView` pointer. `View::from_raw_borrowed` retains this view. Then `View::window` gives its window.

```rust,no_run
use core::ffi::c_void;
use core::ptr::NonNull;

use appkit::prelude::*;

fn style(ns_view: NonNull<c_void>) {
    let main_thread = MainThreadMarker::new().expect("main thread");
    let view = unsafe { View::from_raw_borrowed(ns_view, main_thread) };
    let Some(window) = view.window() else { return };
    window.set_appearance(Some(AppearanceName::DarkAqua));
    window.set_titlebar_separator_style(TitlebarSeparatorStyle::Line);
    let chrome = window.frame().size.height - window.content_layout_rect().size.height;
    println!("title bar height: {chrome}");
}
```

- `View::from_raw_borrowed` and `Window::from_raw_borrowed` are `unsafe`. The pointer must point to a live `NSView` or `NSWindow`.
- `Appearance::from_raw_borrowed` retains a live `NSAppearance`. `RetainedMenu::from_retained_ptr` adopts a caller-owned `NSMenu`.
- `Event::from_raw_retained` takes ownership of a retained event. `EventRef::from_raw_borrowed` borrows a live event.
- `View::layer_ptr` gives the `CALayer` of the view, or `None`. The pointer is not retained. It stays valid while the view keeps this layer.
- `Window::animate_alpha_value` changes the alpha value through the animator of the window.
- Geometry values are `CGRect`, `CGSize` and `CGPoint` from `apple-cf`. This crate exports these types again.

## Windows and the event loop

`Window::new` makes a titled window that the user can close, minimize and resize. The window stays in memory after the user closes it.

```rust,no_run
use appkit::prelude::*;

fn main() -> Result<(), AppKitError> {
    let main_thread = MainThreadMarker::new().expect("main thread");
    let application = Application::shared(main_thread);
    assert!(application.set_activation_policy(ActivationPolicy::Regular));
    let window = Window::new(main_thread, CGRect::new(200.0, 200.0, 480.0, 320.0), "Example")?;
    window.make_key_and_order_front();
    application.activate();
    application.run();
    Ok(())
}
```

- `Application::run` returns only after a call to `-stop:`. `Application::terminate` ends the process.
- `Window::as_ptr` gives the `NSWindow` pointer for other libraries, for example an accessibility adapter.

## Hosted UI and window controls

Use `host_window::HostedWindow` with `HostedWindowConfiguration` to receive window events and close requests.
`HostedWindow::new_panel` makes an `NSPanel` with `HostedPanelOptions`.
Use `host_view::HostedView` to host a `CAMetalLayer` and receive input and view notifications.
Its constructor borrows a live layer pointer. The native view retains the layer.

The `window_ops` module supplies typed style masks, levels, geometry, sheets, tabs and titlebar buttons.
Setters that return `Result` report native failures as `AppKitError`.
`Window::encode_restorable_state` copies secure native window state. `Window::restore_state` rejects invalid data.
Space restoration is not guaranteed without private restoration options. This crate does not use those options.

`HostedView::set_owns_titlebar_drag` controls the view's titlebar drag area.
`AppKit` calls the `_opaqueRectForWindowMoveWhenInTitlebar` override hook. This crate does not call private selectors.

## Menus, dialogs and services

`menu::Menu` and `menu::MenuItem` support responder actions, shortcuts, submenus and application menu roles.
`Menu::retain_for_dock` gives `ApplicationHost` an owned Dock menu. A submenu cannot contain its own menu.
`ApplicationHost` forwards launch, reopen, quit, URL, screen and menu events to its handlers.
`ApplicationHost::delegate_borrowed_ptr` stays valid while the host lives. Do not release this pointer.

`panel::OpenPanel` and `panel::SavePanel` give owned file paths. `alert::Alert` gives a button index from a sheet.
Callbacks run once, even if you drop the session. Call `cancel` to close the dialog.
If `Alert::begin_sheet` returns `AlertSheetStart::ParentMayBeShown`, show the parent window to present the sheet.
A parent window that closes aborts its sheet.
If you drop a `window_ops::SheetCompletion`, its sheet stays open. Its `cancel` method stops only the completion callback.

`pasteboard::Pasteboard` reads and writes the general, find and unique pasteboards.
`Pasteboard::unique` gives a UUID name and releases the pasteboard globally on drop.
Panel and file-URL drag paths keep their file-system bytes. Legacy pasteboard file names use UTF-8 bytes.
`Pasteboard::set_filenames` and `DocumentController::note_new_recent_document` require UTF-8 paths.
`Workspace` opens URLs and files, reads icons, observes power events and registers URL schemes.
`appearance::Appearance` gives standard appearances and view overrides. High-contrast appearance names are for matches only.
`blur::VisualEffectView` supplies standard and clear blur. `cursor::Cursor` supplies system and image cursors.
Use `sound::beep` for the system alert sound.

## Events, text input and drag operations

`EventRef<'_>` gives event details during a callback. Retain it as an `Event` if it must live after the callback.
`Event` also supplies synthetic key and mouse events.
`text_input::TextInputClient::install` attaches a `TextInputHandler` to a `HostedView`.
Text ranges count UTF-16 code units. Check `TextInputEventStatus::is_handled` after `handle_event`.
`drag::DragRegistration` receives file drops on a hosted window and starts file or text drags from a hosted view.
`DragRegistration::start` needs a left mouse-down `Event` from that window.

## Callbacks and lifetime

`CallbackRegistration::new(main_thread, state)` assigns a numeric token to a main-thread `Rc<T>` state.
The registry keeps a weak state handle. Swift keeps only the token and a copy of `CallbackTableV1`.
`CallbackRegistration::snapshot(token, main_thread)` gets a strong state handle before the callback runs.
Implement `CallbackState` for a host state, and pass its `Rc` to `NativeCallbackRegistration::new`.
Drop the registration to deactivate the token.
`NativeCallbackRegistration<T>` deactivates the token before it releases its Swift owner.
Stop frame delivery before you release a hosted view or window.
Drop the `TextInputClient` and `DragRegistration` before you release their view and window.
`HostedView::deactivate_callbacks` and `HostedWindow::deactivate_callbacks` stop callbacks but do not detach native objects.
`DragRegistration::deactivate_callbacks` also detaches its native registration.

Each `CallbackTableV1` has a version and a byte size.
Its `event` callback receives a borrowed `NSEvent` pointer for one synchronous call.
`EventRef<'_>` keeps that borrow in Rust. Use `EventRef::retain` to keep an owned `Event`.
Its `text` callback receives UTF-8 bytes as a pointer and a length.
`dispatch_text` copies those bytes before it calls the handler. The pointer expires when the callback returns.
Use `HandlerSlot::replace` and `HandlerSlot::invoke` for each `FnMut` handler.
`invoke` removes the handler during the call and restores it only if no replacement enters the slot.
A recursive call returns `None`. Use a safe value for a synchronous query.
`dispatch_event` and `dispatch_text` reject inactive tokens and calls outside the main thread, and contain Rust panics.

`Utf16Range` counts UTF-16 code units. `Utf16Range::INVALID` represents `NSNotFound`.
Use `from_utf8_range` and `to_utf8_range` to convert Rust byte ranges at character boundaries.
`EventRef::from_raw_borrowed` needs a live `NSEvent` for the full borrow.
`copy_utf8` and `dispatch_text` need a readable byte buffer of the specified length.
The C callbacks must use the dispatch functions before they call Rust handlers.

## Raw functions

The `ffi` module declares the Swift bridge functions.
The caller owns each object pointer that a bridge function returns, except for the pointer from `appkit_view_layer`.
An `as_ptr` method gives a borrowed pointer. Do not release it unless you retain it first.
Use `ffi::appkit_release` to release an owned object.
Use `ffi::appkit_string_free` to release a string.
The callback table uses the C types in `AppKitCallbackABI.h`.
The Swift bridge uses `appkit_try` to catch Objective-C exceptions in guarded calls.
Fallible wrappers return `false`, `None` or `AppKitError` if those calls fail.
The guard does not protect an `ffi` call unless its bridge function uses it.

## Examples

```bash
cargo run --example 01_accessibility
cargo run --example 02_screens
cargo run --example 03_window
cargo run --example 04_callbacks
cargo run --example 05_events
cargo run --example 06_host_window
cargo run --example 07_window_ops
cargo run --example 08_application_host
cargo run --example 09_pasteboard
cargo run --example 10_panels
cargo run --example 11_cursor
cargo run --example 12_workspace
cargo run --example 13_host_view
cargo run --example 14_menu
cargo run --example 15_window_state
cargo run --example 16_text_input
cargo run --example 17_drag
```

`04_callbacks` checks callback ownership without a window. The other examples show each `AppKit` domain.

## License

MIT OR Apache-2.0
