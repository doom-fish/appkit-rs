# COVERAGE

- Framework: `AppKit.framework`
- Scope: The released API makes titled windows and runs the event loop. It gives access to views, screens, the shared application, images, the scroller style, and display settings.
- Unreleased scope: The callback ABI adds tokens, borrowed events, text bytes, and UTF-16 ranges. It does not make hosted views or menus.
- Legend: `✅` implemented, `⏭️` not implemented

## Callback ABI (unreleased)

| Contract | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| Main-thread tokens and weak state handles | ✅ | `RegistrationToken`, `CallbackRegistration<T>` | A late callback gets no state. |
| Swift owner and versioned callback table | ✅ | `CallbackState`, `NativeCallbackRegistration<T>`, `CallbackTableV1` | Drop deactivates the token before Swift releases the owner. |
| Re-entrant `FnMut` handler | ✅ | `HandlerSlot<H>` | A recursive call gets `None`. A new handler replaces the old handler. |
| Synchronous `NSEvent` borrow | ✅ | `EventRef<'_>`, `Event` | Retain the event before you store it. |
| `NSRange` and UTF-16 offsets | ✅ | `Utf16Range` | `INVALID` represents `NSNotFound`. |
| UTF-8 pointer and length | ✅ | `copy_utf8`, `dispatch_text` | Rust copies the bytes before it calls the handler. |
| Hosted view and window callbacks | ⏭️ |  | The later domain modules provide these objects. |

## NSView

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| an existing `NSView` | ✅ | `View::from_raw_borrowed` | Retains the view. Needs a `MainThreadMarker`. |
| `window` | ✅ | `View::window` |  |
| `bounds` | ✅ | `View::bounds` |  |
| `frame` | ✅ | `View::frame` |  |
| `layer` | ✅ | `View::layer_ptr` | A pointer that is not retained. |
| `wantsLayer` | ✅ | `View::wants_layer`, `View::set_wants_layer` |  |
| subviews, drawing, events, tracking areas | ⏭️ |  |  |

## NSWindow

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `-initWithContentRect:styleMask:backing:defer:` | ✅ | `Window::new` | A titled window that the user can close, minimize and resize. `releasedWhenClosed` is `NO`. |
| an existing `NSWindow` | ✅ | `Window::from_raw_borrowed` | Retains the window. Needs a `MainThreadMarker`. |
| `contentView` | ✅ | `Window::content_view` |  |
| `appearance` | ✅ | `Window::appearance`, `Window::set_appearance` | The standard appearance names only. |
| `title` | ✅ | `Window::title`, `Window::set_title` |  |
| `titleVisibility` | ✅ | `Window::title_visibility`, `Window::set_title_visibility` |  |
| `titlebarSeparatorStyle` | ✅ | `Window::titlebar_separator_style`, `Window::set_titlebar_separator_style` |  |
| `frame` | ✅ | `Window::frame` |  |
| `contentLayoutRect` | ✅ | `Window::content_layout_rect` |  |
| `backingScaleFactor` | ✅ | `Window::backing_scale_factor` |  |
| `screen` | ✅ | `Window::screen` |  |
| `alphaValue` | ✅ | `Window::alpha_value`, `Window::set_alpha_value` |  |
| `animator.alphaValue` | ✅ | `Window::animate_alpha_value` |  |
| `animationBehavior` | ✅ | `Window::animation_behavior`, `Window::set_animation_behavior` |  |
| `-orderFront:` | ✅ | `Window::order_front` |  |
| `-makeKeyAndOrderFront:` | ✅ | `Window::make_key_and_order_front` |  |
| `-close` | ✅ | `Window::close` |  |
| `-setFrame:display:`, other style masks, tabs, sheets | ⏭️ |  |  |

## NSScreen

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `+mainScreen` | ✅ | `Screen::main` |  |
| `+screens` | ✅ | `Screen::screens` |  |
| `frame` | ✅ | `Screen::frame` |  |
| `visibleFrame` | ✅ | `Screen::visible_frame` |  |
| `backingScaleFactor` | ✅ | `Screen::backing_scale_factor` |  |
| `deviceDescription[NSScreenNumber]` | ✅ | `Screen::display_id` |  |
| `localizedName` | ✅ | `Screen::localized_name` |  |
| `colorSpace`, `maximumFramesPerSecond`, `safeAreaInsets`, `auxiliaryTopLeftArea` | ⏭️ |  |  |

## NSApplication

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `+sharedApplication` | ✅ | `Application::shared` |  |
| `applicationIconImage` | ✅ | `Application::set_application_icon_image` |  |
| `-orderFrontStandardAboutPanelWithOptions:` | ✅ | `Application::order_front_standard_about_panel` | The application name, the application version and the version. |
| `-setActivationPolicy:` | ✅ | `Application::set_activation_policy` |  |
| `-activate`, `-activateIgnoringOtherApps:` | ✅ | `Application::activate` | `-activate` on macOS 14 and later. |
| `-run` | ✅ | `Application::run` |  |
| `-terminate:` | ✅ | `Application::terminate` |  |
| `-stop:`, menus, the dock tile | ⏭️ |  |  |

## Other classes

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `-[NSImage initWithData:]` | ✅ | `Image::from_data` | Data that AppKit cannot read gives `AppKitError::InvalidImageData`. |
| `NSImage.size` | ✅ | `Image::size` |  |
| `+[NSScroller preferredScrollerStyle]` | ✅ | `Scroller::preferred_scroller_style` |  |
| `NSWorkspace.accessibilityDisplayShouldReduceMotion` | ✅ | `Workspace::accessibility_display_should_reduce_motion` |  |
| `NSWorkspace.accessibilityDisplayShouldReduceTransparency` | ✅ | `Workspace::accessibility_display_should_reduce_transparency` |  |
| `NSWorkspace.accessibilityDisplayShouldIncreaseContrast` | ✅ | `Workspace::accessibility_display_should_increase_contrast` |  |
| `NSWorkspace.accessibilityDisplayShouldDifferentiateWithoutColor` | ✅ | `Workspace::accessibility_display_should_differentiate_without_color` |  |
| `NSWorkspace.accessibilityDisplayShouldInvertColors` | ✅ | `Workspace::accessibility_display_should_invert_colors` |  |
| `NSWorkspace.isVoiceOverEnabled` | ✅ | `Workspace::is_voice_over_enabled` |  |
| the other `NSWorkspace` functions, `NSAppearance` drawing, `NSColor`, `NSFont`, `NSMenu`, `NSPasteboard` | ⏭️ |  |  |
