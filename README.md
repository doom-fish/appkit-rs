# appkit-rs

Safe Rust bindings for Apple's [AppKit](https://developer.apple.com/documentation/appkit) framework on macOS. This release makes windows, gives access to windows and views that other code makes, runs the event loop, and gives access to screens, the shared application, images and the accessibility settings of the display.

The crate requires macOS 12.0 or later.

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

`AppKit` objects belong to the main thread.

- `MainThreadMarker::new` gives a marker only on the main thread. The functions that make or find `View`, `Window`, `Screen` and `Application` values need a marker.
- These values are not `Send` and not `Sync`. Thus they stay on the main thread.
- `Image` and `Workspace` do not need a marker.

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

## Callbacks and lifetime

`CallbackRegistration::new(main_thread, state)` assigns a numeric token to a main-thread `Rc<T>` state.
The registry keeps a weak state handle. Swift keeps only the token and a copy of `CallbackTableV1`.
`CallbackRegistration::snapshot(token, main_thread)` gets a strong state handle before the callback runs.
Implement `CallbackState` for a host state, and pass its `Rc` to `NativeCallbackRegistration::new`.
Drop the registration to deactivate the token.
`NativeCallbackRegistration<T>` deactivates the token before it releases its Swift owner.
Stop frame delivery and detach other owners before you release a hosted view or window.

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
Use `ffi::appkit_release` to release an owned object.
Use `ffi::appkit_string_free` to release a string.
The callback table uses the C types in `AppKitCallbackABI.h`.

## Examples

```bash
cargo run --example 01_accessibility
cargo run --example 02_screens
cargo run --example 03_window
cargo run --example 04_callbacks
```

`04_callbacks` checks callback ownership without a window.

## License

MIT OR Apache-2.0
