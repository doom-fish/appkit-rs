# COVERAGE

- Framework: `AppKit.framework`
- Scope: The crate covers windows, views, delegates, events, text input, drag operations, pasteboards, menus, panels, cursors and workspace services.
- Scope: It also covers appearance, alerts, screens, secure window state and main-thread callback tokens.
- Limits: The crate does not call private selectors. Space restoration is not guaranteed without private restoration options.
- Legend: `✅` implemented, `⏭️` not implemented

## Callback ABI

| Contract | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| Main-thread tokens and weak state handles | ✅ | `RegistrationToken`, `CallbackRegistration<T>` | A late callback gets no state. |
| Swift owner and versioned callback table | ✅ | `CallbackState`, `NativeCallbackRegistration<T>`, `CallbackTableHeader`, `CallbackTableV1`, `EventCallback`, `TextCallback` | Drop deactivates the token before Swift releases the owner. |
| Reentrant `FnMut` handler | ✅ | `HandlerSlot<H>` | A recursive call gets `None`. A new handler replaces the old handler. |
| Synchronous `NSEvent` borrow | ✅ | `EventRef<'_>`, `EventRef::retain`, `Event` | Retain the event before you store it. |
| `NSRange` and UTF-16 offsets | ✅ | `Utf16Range::{is_valid, from_range, to_range, from_utf8_range, to_utf8_range, INVALID}` | `INVALID` represents `NSNotFound`. |
| UTF-8 pointer and length | ✅ | `copy_utf8`, `dispatch_text` | Rust copies the bytes before it calls the handler. |
| Main-thread callback dispatch | ✅ | `dispatch_event`, `dispatch_text` | Inactive tokens and calls from other threads get safe results. Rust contains handler panics. |

## Dock menu inspection

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `NSApplicationDelegate.applicationDockMenu` | Implemented | `Menu::application_dock_menu` | The method retains the result without activation. |
| `NSMenu.performActionForItem` | Implemented | `Menu::perform_action_at` | An invalid, disabled, separator, or actionless item returns `false`. |

## NSAccessibility

| Contract | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| Per-host callback table | Implemented | `accessibility::AccessibilityCallbacksV1` | Version 1, complete size, main thread, and all callbacks are required. |
| Virtual native elements | Implemented | `ffi::accessibility::{appkit_a11y_node_new, appkit_a11y_retain, appkit_a11y_release}` | The consumer owns node identity and semantics. |
| Hosted view children, focus and hit tests | Implemented | `appkit_a11y_view_ref_new`, `appkit_a11y_subclass_attach`, `appkit_a11y_subclass_detach` | Each attachment retains its own callback table and a weak view reference. |
| Names, roles, values, states and bounds | Implemented | String, number, flag, value, list and rectangle callbacks | Native selectors use consumer results. |
| Text ranges and attributed text | Implemented | `NodeRange`, `NodeTextRuns`, `TextRun` | Native ranges use UTF-16 offsets. |
| Actions and text selection | Implemented | `NodeAction` | The consumer receives the action and payload. |
| Native change notifications | Implemented | `appkit_a11y_post` | Focus, value, selected text, title, selected rows and destruction. |
| Live announcements | Implemented | `appkit_a11y_announce` | Assertive uses high priority. Polite uses medium priority. |

The bridge does not construct or compare semantic trees.
Hidden-window provider checks do not establish screen-reader speech parity.

## NSView

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| an existing `NSView` | ✅ | `View::from_raw_borrowed`, `HostedView::as_view`, `WindowChromeView::as_view` | Each path retains the view. Raw import needs a `MainThreadMarker`. |
| `-initWithFrame:` with a `CAMetalLayer` | ✅ | `HostedView::new`, `MetalLayer::new` | The native view retains the borrowed layer. |
| `window` | ✅ | `View::window` |  |
| `bounds` | ✅ | `View::bounds` |  |
| `frame`, `-setFrame:`, `-setFrameOrigin:` | ✅ | `View::frame`, `WindowChromeView::{frame, set_frame, set_frame_origin}` | Chrome views expose frame setters. |
| `superview` | ✅ | `WindowChromeView::superview` |  |
| `hidden` | ✅ | `WindowChromeView::{is_hidden, set_hidden}` |  |
| `-addSubview:`, `-removeFromSuperview` | ✅ | `HostedView::attach_to`, `HostedView` drop | Drop detaches the hosted view. |
| `layer` | ✅ | `View::layer_ptr`, `HostedView::layer_ptr` | The layer pointer is not retained. |
| `wantsLayer` | ✅ | `View::wants_layer`, `View::set_wants_layer` |  |
| `-setFrameSize:` | ✅ | `HostedView::set_frame_size`, `HostViewNotification::FrameSizeChanged` | The size must be finite and nonnegative. |
| `-setNeedsDisplay:` | ✅ | `View::set_needs_display`, `WindowChromeView::set_needs_display`, `Window::set_content_view_needs_display` |  |
| `-viewDidChangeBackingProperties` | ✅ | `HostViewNotification::BackingPropertiesChanged`, `HostedView::{notify_backing_properties_changed, set_notification_handler}`, `HostViewNotificationHandler` |  |
| `-viewDidChangeEffectiveAppearance` | ✅ | `HostViewNotification::EffectiveAppearanceChanged`, `HostedView::notify_effective_appearance_changed` |  |
| `-updateTrackingAreas` | ✅ | `WindowChromeView::update_tracking_areas`, `HostedView::set_tracking_options` | The hosted view updates its area after a size change. |
| `-resetCursorRects`, `-addCursorRect:cursor:` | ✅ | `HostViewNotification::ResetCursorRects`, `HostedView::{invalidate_cursor_rects, add_cursor_rect_borrowed}` | The cursor pointer must be live during the call. |
| `-acceptsFirstMouse:`, `acceptsFirstResponder` | ✅ | `HostViewQuery`, `HostViewQueryHandler`, `HostedView::{accepts_first_mouse, accepts_first_responder, set_query_handler}` | A missing handler uses the default native answer. |
| `_opaqueRectForWindowMoveWhenInTitlebar` | ✅ | `HostedView::set_owns_titlebar_drag` | AppKit calls this override hook. The crate does not call the selector. |
| hosted view callback deactivation | ✅ | `HostedView::deactivate_callbacks` | Stops callbacks without detaching the native view. |
| Other subview and `drawRect:` operations | ⏭️ |  | Hosted views supply a specific layer and callback surface. |

## CALayerDelegate

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `-displayLayer:` | ✅ | `HostViewNotification::DisplayLayer`, `HostedView::display_layer` | The hosted view sends a notification for its layer. |

## NSResponder

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `-performKeyEquivalent:` | ✅ | `HostedView::perform_key_equivalent`, `HostViewEvent::PerformKeyEquivalent` | A recursive query returns `false`. |
| `-keyDown:` | ✅ | `HostViewEvent::KeyDown`, `HostedView::{set_event_handler, send_event}`, `HostViewEventHandler` | The event handler receives a borrowed `EventRef`. |
| `-keyUp:` | ✅ | `HostViewEvent::KeyUp` |  |
| `-flagsChanged:` | ✅ | `HostViewEvent::FlagsChanged` |  |
| `-mouseDown:`, `-mouseUp:` | ✅ | `HostViewEvent::{LeftMouseDown, LeftMouseUp}` |  |
| `-rightMouseDown:`, `-rightMouseUp:` | ✅ | `HostViewEvent::{RightMouseDown, RightMouseUp}` |  |
| `-otherMouseDown:`, `-otherMouseUp:` | ✅ | `HostViewEvent::{OtherMouseDown, OtherMouseUp}` |  |
| `-mouseDragged:` | ✅ | `HostViewEvent::LeftMouseDragged` |  |
| `-rightMouseDragged:` | ✅ | `HostViewEvent::RightMouseDragged` |  |
| `-otherMouseDragged:` | ✅ | `HostViewEvent::OtherMouseDragged` |  |
| `-mouseMoved:` | ✅ | `HostViewEvent::MouseMoved` |  |
| `-mouseEntered:`, `-mouseExited:` | ✅ | `HostViewEvent::{MouseEntered, MouseExited}` |  |
| `-scrollWheel:` | ✅ | `HostViewEvent::ScrollWheel` |  |
| `-pressureChange:` | ✅ | `HostViewEvent::PressureChanged` |  |
| `-magnify:`, `-swipe:` | ✅ | `HostViewEvent::{Magnified, Swiped}` |  |
| `-smartMagnify:` | ✅ | `HostViewEvent::SmartMagnified` |  |
| Other responder-chain events and commands | ⏭️ |  |  |

## NSTrackingArea

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `-initWithRect:options:owner:userInfo:` | ✅ | `HostedView::{set_tracking_options, tracking_options}`, `TrackingAreaOptions` | Nonzero options need `IN_VISIBLE_RECT` and one active-state option. |
| Arbitrary tracking area owners | ⏭️ |  |  |

## NSTextInputClient

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| protocol installation on `NSView` | ✅ | `TextInputClient::install`, `TextInputClient::set_handler` | Installs callbacks on a `HostedView`. Drop removes them. |
| `-validAttributesForMarkedText` | ✅ | `TextInputClient::install` | The native view returns an empty list. |
| `-hasMarkedText` | ✅ | `TextInputHandler::has_marked_text` |  |
| `-markedRange` | ✅ | `TextInputHandler::marked_range` |  |
| `-selectedRange` | ✅ | `TextInputHandler::selected_range` |  |
| `-firstRectForCharacterRange:actualRange:` | ✅ | `TextInputHandler::first_rect_for_character_range`, `TextInputRect` | The native view converts the rectangle to screen coordinates. |
| `-insertText:replacementRange:` | ✅ | `TextInputHandler::insert_text` |  |
| `-setMarkedText:selectedRange:replacementRange:` | ✅ | `TextInputHandler::set_marked_text` |  |
| `-unmarkText` | ✅ | `TextInputHandler::unmark_text` |  |
| `-attributedSubstringForProposedRange:actualRange:` | ✅ | `TextInputHandler::attributed_substring_for_proposed_range`, `TextInputSubstring` | The native view returns plain text as an attributed string. |
| `-characterIndexForPoint:` | ✅ | `TextInputHandler::character_index_for_point` |  |
| `-doCommandBySelector:` | ✅ | `TextInputHandler::do_command_by_selector` | The input context reports whether the handler accepts the command. |
| Other text input attributes | ⏭️ |  |  |

## NSTextInputContext

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `-handleEvent:` | ✅ | `TextInputClient::handle_event`, `TextInputEventStatus` | Reports consumed events and handled or unhandled commands. |
| `-discardMarkedText` | ✅ | `TextInputClient::discard_marked_text` |  |
| `-invalidateCharacterCoordinates` | ✅ | `TextInputClient::invalidate_character_coordinates` |  |
| Other input context methods | ⏭️ |  |  |

## NSWindow

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `-initWithContentRect:styleMask:backing:defer:` | ✅ | `Window::new`, `HostedWindow::new`, `HostedWindowConfiguration`, `BackingStore` | The constructors retain the window after close. The hosted constructor accepts a style mask and screen. |
| an existing `NSWindow` | ✅ | `Window::from_raw_borrowed`, `HostedWindow::as_window` | Each path retains the window. Raw import needs a `MainThreadMarker`. |
| `contentView` | ✅ | `Window::content_view` |  |
| `appearance` | ✅ | `Window::appearance`, `Window::set_appearance` | The standard appearance names only. |
| `effectiveAppearance` | ✅ | `Window::effective_appearance`, `Appearance::effective_for_window` |  |
| `title` | ✅ | `Window::title`, `Window::set_title` |  |
| `titleVisibility` | ✅ | `Window::title_visibility`, `Window::set_title_visibility`, `TitleVisibility` |  |
| `titlebarSeparatorStyle` | ✅ | `Window::titlebar_separator_style`, `Window::set_titlebar_separator_style` |  |
| `styleMask` | ✅ | `Window::{style_mask, set_style_mask}`, `window_ops::WindowStyleMask` | The hosted constructor uses `host_window::WindowStyleMask`. |
| `level` | ✅ | `Window::{level, set_level}`, `WindowLevel` |  |
| `collectionBehavior` | ✅ | `Window::{collection_behavior, set_collection_behavior}`, `WindowCollectionBehavior` |  |
| `-makeFirstResponder:` | ✅ | `Window::make_first_responder` |  |
| `initialFirstResponder` | ✅ | `Window::set_initial_first_responder` |  |
| `frame`, `-setFrame:display:animate:` | ✅ | `Window::{frame, set_frame}` | The setter reports invalid geometry as an error. |
| `-setFrameTopLeftPoint:` | ✅ | `Window::set_frame_top_left_point` |  |
| `contentSize`, `-setContentSize:` | ✅ | `Window::{content_size, set_content_size}` |  |
| `-frameRectForContentRect:`, `-contentRectForFrameRect:` | ✅ | `Window::{frame_rect_for_content_rect, content_rect_for_frame_rect}` |  |
| `-convertPointToScreen:`, `-convertRectToScreen:` | ✅ | `Window::{convert_point_to_screen, convert_rect_to_screen}` |  |
| `contentMinSize` | ✅ | `Window::{minimum_content_size, set_minimum_content_size}` |  |
| `contentLayoutRect` | ✅ | `Window::content_layout_rect` |  |
| `backingScaleFactor` | ✅ | `Window::backing_scale_factor` |  |
| `screen` | ✅ | `Window::screen` |  |
| `mouseLocationOutsideOfEventStream` | ✅ | `Window::mouse_location_outside_event_stream` |  |
| `alphaValue` | ✅ | `Window::alpha_value`, `Window::set_alpha_value` |  |
| `animator.alphaValue` | ✅ | `Window::animate_alpha_value` |  |
| `animationBehavior` | ✅ | `Window::animation_behavior`, `Window::set_animation_behavior`, `AnimationBehavior` |  |
| `isKeyWindow`, `isMainWindow` | ✅ | `Window::{is_key_window, is_main_window}` |  |
| `isVisible`, `occlusionState` | ✅ | `Window::{is_visible, occlusion_state}`, `WindowOcclusionState` |  |
| `isMiniaturized`, `-miniaturize:`, `-deminiaturize:` | ✅ | `Window::{is_miniaturized, miniaturize, deminiaturize}` |  |
| `isZoomed`, `-zoom:` | ✅ | `Window::{is_zoomed, zoom}` |  |
| `-toggleFullScreen:` | ✅ | `Window::toggle_full_screen` |  |
| `styleMask`, `frame`, `NSApplication.presentationOptions` | ✅ | `Window::{is_simple_full_screen, toggle_simple_full_screen}` | Implements simple full screen with public APIs. |
| `-beginSheet:completionHandler:` | ✅ | `Window::begin_sheet`, `SheetCompletion` | Drop does not stop the sheet or callback. Closing the parent aborts the sheet. |
| sheet completion cancellation | ✅ | `SheetCompletion::cancel` | Stops the completion callback but keeps the sheet open. |
| `-endSheet:returnCode:` | ✅ | `Window::end_sheet`, `SheetResponse` |  |
| `attachedSheet`, `sheetParent` | ✅ | `Window::{attached_sheet, sheet_parent}` |  |
| `+allowsAutomaticWindowTabbing` | ✅ | `Window::{allows_automatic_window_tabbing, set_allows_automatic_window_tabbing}` |  |
| `tabbingIdentifier`, `tabbingMode` | ✅ | `Window::{tabbing_identifier, set_tabbing_identifier, tabbing_mode, set_tabbing_mode}`, `WindowTabbingMode` |  |
| `-addTabbedWindow:ordered:`, `tabbedWindows` | ✅ | `Window::{add_tabbed_window, tabbed_windows}` |  |
| `-toggleTabOverview:`, `-moveTabToNewWindow:` | ✅ | `Window::{toggle_tab_overview, move_tab_to_new_window}` |  |
| `-mergeAllWindows:`, `-selectNextTab:`, `-selectPreviousTab:`, `-toggleTabBar:` | ✅ | `Window::{merge_all_windows, select_next_tab, select_previous_tab, toggle_tab_bar}` |  |
| `-standardWindowButton:` | ✅ | `Window::standard_window_button`, `WindowButton`, `WindowChromeView` | Supports close, minimize and zoom buttons. |
| `backgroundColor` | ✅ | `Window::{background_color, set_background_color}`, `RgbaColor` |  |
| `isOpaque`, `hasShadow`, `-invalidateShadow` | ✅ | `Window::{is_opaque, set_opaque, has_shadow, set_has_shadow, invalidate_shadow}` |  |
| `titlebarAppearsTransparent` | ✅ | `Window::{titlebar_appears_transparent, set_titlebar_appears_transparent}` |  |
| `isMovable`, `isMovableByWindowBackground` | ✅ | `Window::{is_movable, set_movable, is_movable_by_window_background, set_movable_by_window_background}` |  |
| `acceptsMouseMovedEvents`, `-performWindowDragWithEvent:` | ✅ | `Window::{accepts_mouse_moved_events, set_accepts_mouse_moved_events, perform_window_drag}` | The drag method borrows an `EventRef`. |
| `isDocumentEdited`, `representedFilename` | ✅ | `Window::{is_document_edited, set_document_edited, represented_filename, set_represented_filename}` |  |
| `-orderFront:` | ✅ | `Window::order_front` |  |
| `-makeKeyAndOrderFront:` | ✅ | `Window::make_key_and_order_front` |  |
| `-close` | ✅ | `Window::close` |  |
| `-orderOut:`, `-orderBack:`, `-orderWindow:relativeTo:` | ✅ | `Window::{order_out, order_back, order_window}`, `WindowOrderingMode` |  |
| `windowNumber`, `-displayIfNeeded` | ✅ | `Window::{window_number, display_if_needed}` |  |
| `-invalidateCursorRectsForView:`, `-resignKeyWindow` | ✅ | `Window::{invalidate_cursor_rects_for_view, resign_key_window}` |  |
| `isReleasedWhenClosed` | ✅ | `Window::{is_released_when_closed, retain_when_closed}` | `retain_when_closed` sets the native value to `false`. |
| `canBecomeMain`, `canBecomeKey` | ✅ | `HostedWindow::{can_become_main, set_can_become_main, can_become_key, set_can_become_key}` | The hosted subclass overrides these properties. |
| `-close`, `-addTitlebarAccessoryViewController:` | ✅ | `HostedWindowEvent::{Close, AddedTitlebarAccessory}` | A hosted window sends these events through its handler. |
| tab action overrides | ✅ | `HostedWindowEvent::{MoveTabToNewWindow, MergeAllWindows, SelectNextTab, SelectPreviousTab, ToggleTabBar}` |  |
| `-encodeRestorableStateWithCoder:` | ✅ | `Window::encode_restorable_state` | Uses a secure coder and copies its data. |
| `-restoreStateWithCoder:` | ✅ | `Window::restore_state` | Invalid state gives `AppKitError`. The original Space is not guaranteed. |
| Private Space restoration options | ⏭️ |  | The crate does not call private selectors. |
| Other window animations, tab groups and titlebar customization | ⏭️ |  |  |

## NSPanel

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `-initWithContentRect:styleMask:backing:defer:` | ✅ | `HostedWindow::new_panel`, `HostedWindowConfiguration` | Makes a hosted `NSPanel`. |
| `isFloatingPanel`, `becomesKeyOnlyIfNeeded` | ✅ | `HostedPanelOptions` | Set each option during construction. |
| `worksWhenModal`, `hidesOnDeactivate` | ✅ | `HostedPanelOptions` |  |
| Other panel behavior | ⏭️ |  |  |

## NSWindowDelegate

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| delegate installation and removal | ✅ | `HostedWindow::{set_handler, clear_handler, deactivate_callbacks}` | `deactivate_callbacks` keeps the native window but stops Rust callbacks. |
| `-windowShouldClose:` | ✅ | `HostedWindow::{set_should_close_handler, clear_should_close_handler}` | Without a handler, the active delegate allows close. |
| `-windowWillClose:` | ✅ | `HostedWindowEvent::WillClose` |  |
| `-windowDidResize:` | ✅ | `HostedWindowEvent::DidResize` |  |
| `-windowDidMove:` | ✅ | `HostedWindowEvent::DidMove` |  |
| `-windowDidChangeScreen:` | ✅ | `HostedWindowEvent::DidChangeScreen` |  |
| `-windowDidChangeBackingProperties:` | ✅ | `HostedWindowEvent::DidChangeBackingProperties` |  |
| `-windowDidBecomeKey:`, `-windowDidResignKey:` | ✅ | `HostedWindowEvent::{DidBecomeKey, DidResignKey}` |  |
| `-windowDidChangeOcclusionState:` | ✅ | `HostedWindowEvent::DidChangeOcclusionState` |  |
| `-windowWillEnterFullScreen:`, `-windowDidEnterFullScreen:` | ✅ | `HostedWindowEvent::{WillEnterFullScreen, DidEnterFullScreen}` |  |
| `-windowDidFailToEnterFullScreen:` | ✅ | `HostedWindowEvent::DidFailToEnterFullScreen` |  |
| `-windowWillExitFullScreen:`, `-windowDidExitFullScreen:` | ✅ | `HostedWindowEvent::{WillExitFullScreen, DidExitFullScreen}` |  |
| `-windowDidFailToExitFullScreen:` | ✅ | `HostedWindowEvent::DidFailToExitFullScreen` |  |
| Other delegate methods | ⏭️ |  |  |

## NSWindowTabGroup

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `isTabBarVisible` | ✅ | `Window::is_tab_bar_visible` |  |
| `selectedWindow` | ✅ | `Window::{selected_tab, select_tab}` |  |
| Other tab group methods | ⏭️ |  |  |

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
| `maximumFramesPerSecond` | ✅ | `Screen::maximum_frames_per_second` | Gives the highest refresh rate of the screen. |
| `colorSpace`, `safeAreaInsets`, `auxiliaryTopLeftArea` | ⏭️ |  |  |

## NSApplication

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `+sharedApplication` | ✅ | `Application::shared` |  |
| `applicationIconImage` | ✅ | `Application::set_application_icon_image` |  |
| `-orderFrontStandardAboutPanelWithOptions:` | ✅ | `Application::order_front_standard_about_panel` | The application name, the application version and the version. |
| `-setActivationPolicy:` | ✅ | `Application::set_activation_policy` |  |
| `-activate`, `-activateIgnoringOtherApps:` | ✅ | `Application::activate` | `-activate` on macOS 14 and later. |
| `-activateIgnoringOtherApps:` | ✅ | `Application::activate_ignoring_other_apps` | Accepts a boolean argument. |
| `-run` | ✅ | `Application::run` |  |
| `-terminate:` | ✅ | `Application::terminate` |  |
| `keyWindow`, `mainWindow` | ✅ | `Application::{key_window, main_window}` |  |
| `orderedWindows` | ✅ | `Application::ordered_windows` | Returns retained `Window` values. |
| `isActive`, `isHidden` | ✅ | `Application::{is_active, is_hidden}` |  |
| `-hide:`, `-unhide:` | ✅ | `Application::{hide, unhide}` |  |
| `-hideOtherApplications:`, `-unhideAllApplications:` | ✅ | `Application::{hide_other_applications, unhide_all_applications}` |  |
| `presentationOptions` | ✅ | `Application::{presentation_options, set_presentation_options}`, `PresentationOptions` | Invalid option combinations return `false`. |
| `-requestUserAttention:`, `-cancelUserAttentionRequest:` | ✅ | `Application::{request_user_attention, cancel_user_attention}`, `UserAttentionType` |  |
| `appearance`, `effectiveAppearance` | ✅ | `Application::{appearance, effective_appearance, set_appearance}` | High-contrast names are for matches only. |
| `-orderFrontCharacterPalette:` | ✅ | `Application::order_front_character_palette` |  |
| `-changeWindowsItem:title:filename:` | ✅ | `Application::change_windows_item` | Sets the menu title without a filename flag. |
| `currentEvent` | ✅ | `Application::current_event`, `Event::current` | Returns a retained event. |
| `-stop:`, dock tile customization | ⏭️ |  | Menus have a separate Rust surface. |

## NSApplicationDelegate

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| delegate installation and removal | ✅ | `ApplicationHost::{install, delegate_borrowed_ptr}`, `ApplicationHost` drop | The host deactivates its token before it removes its native delegate. |
| `-applicationWillFinishLaunching:` | ✅ | `ApplicationHostHandlers::will_finish_launching` |  |
| `-applicationDidFinishLaunching:` | ✅ | `ApplicationHostHandlers::did_finish_launching` |  |
| `-applicationShouldHandleReopen:hasVisibleWindows:` | ✅ | `ApplicationHostHandlers::should_handle_reopen` | Defaults to `true`. |
| `-applicationShouldTerminate:` | ✅ | `ApplicationHostHandlers::should_terminate` | A request inside another callback gets an immediate answer. |
| `-applicationWillTerminate:` | ✅ | `ApplicationHostHandlers::will_terminate` |  |
| `-application:openURLs:` | ✅ | `ApplicationHostHandlers::open_urls` | Copies URL strings before the callback returns. |
| `-applicationDockMenu:` | ✅ | `ApplicationHostHandlers::dock_menu_retained`, `Menu::retain_for_dock` | The callback returns a retained menu. |
| `NSApplication.didChangeScreenParametersNotification` | ✅ | `ApplicationHostHandlers::screen_parameters_changed` |  |
| Other application delegate methods | ⏭️ |  |  |

## NSMenu

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `-initWithTitle:` | ✅ | `Menu::new` |  |
| `title` | ✅ | `Menu::{title, set_title}` |  |
| `numberOfItems`, `-itemAtIndex:` | ✅ | `Menu::{len, is_empty, item_at}` |  |
| `-addItem:`, `-insertItem:atIndex:`, `-removeItemAtIndex:` | ✅ | `Menu::{add_item, insert_item, remove_item_at}` |  |
| `autoenablesItems` | ✅ | `Menu::{autoenables_items, set_autoenables_items}` |  |
| `delegate` | ✅ | `Menu::set_application_host` | Connects the menu to application host callbacks. |
| `NSApplication.mainMenu`, `servicesMenu`, `windowsMenu`, `helpMenu` | ✅ | `Menu::{application_menu, set_application_menu}`, `ApplicationMenuRole` |  |
| retained Dock menu | ✅ | `Menu::retain_for_dock`, `RetainedMenu::from_retained_ptr` | The constructor adopts an owned pointer. The host receives a retained menu. |
| Other menu presentation methods | ⏭️ |  |  |

## NSMenuItem

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `-initWithTitle:action:keyEquivalent:` | ✅ | `MenuItem::new`, `MenuItemAction` | Supports responder actions and application host actions. |
| `+separatorItem`, `isSeparatorItem` | ✅ | `MenuItem::{separator, is_separator}` |  |
| `title` | ✅ | `MenuItem::{title, set_title}` |  |
| `submenu` | ✅ | `MenuItem::{submenu, set_submenu}` | A submenu cannot contain its own menu. |
| `isEnabled` | ✅ | `MenuItem::{is_enabled, set_enabled}` |  |
| `state` | ✅ | `MenuItem::{state, set_state}`, `MenuItemState` |  |
| `tag` | ✅ | `MenuItem::{tag, set_tag}` |  |
| `representedObject` as an integer | ✅ | `MenuItem::{represented_integer, set_represented_integer}` |  |
| `keyEquivalent`, `keyEquivalentModifierMask` | ✅ | `MenuItem::{key_equivalent, key_equivalent_modifiers, set_key_equivalent}`, `KeyEquivalentModifiers` |  |
| `allowsAutomaticKeyEquivalentLocalization` | ✅ | `MenuItem::allows_automatic_key_equivalent_localization` |  |
| `action`, `target` | ✅ | `MenuItem::{action, set_action, action_selector, has_explicit_target}`, `MenuAction` | `ApplicationHostHandlers::menu_item_action` receives the action and item tag. |
| Images, tooltips and custom item views | ⏭️ |  |  |

## NSMenuDelegate

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `-menuWillOpen:` | ✅ | `ApplicationHostHandlers::menu_will_open`, `Menu::set_application_host` |  |
| Other menu delegate methods | ⏭️ |  |  |

## NSMenuItemValidation

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `-validateMenuItem:` | ✅ | `ApplicationHostHandlers::validate_menu_item` | A missing handler returns `false`. |

## NSEvent

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| event types | ✅ | `EventType` | Unknown values remain available as `Other`. |
| `modifierFlags` | ✅ | `EventRef::modifier_flags`, `ModifierFlags` |  |
| `+modifierFlags`, `+pressedMouseButtons` | ✅ | `Event::{modifier_flags, pressed_mouse_buttons}` |  |
| `type` | ✅ | `EventRef::event_type` |  |
| `locationInWindow` | ✅ | `EventRef::location_in_window` |  |
| `buttonNumber`, `clickCount` | ✅ | `EventRef::{button_number, click_count}` |  |
| `isARepeat`, `keyCode` | ✅ | `EventRef::{is_repeat, key_code}` |  |
| `characters`, `charactersIgnoringModifiers` | ✅ | `EventRef::{characters, characters_ignoring_modifiers}` |  |
| `scrollingDeltaX`, `scrollingDeltaY` | ✅ | `EventRef::scrolling_delta` |  |
| `hasPreciseScrollingDeltas` | ✅ | `EventRef::has_precise_scrolling_deltas` |  |
| `phase`, `momentumPhase` | ✅ | `EventRef::{phase, momentum_phase}`, `EventPhase` |  |
| `pressure`, `stage` | ✅ | `EventRef::{pressure, stage}` |  |
| `magnification` | ✅ | `EventRef::magnification` |  |
| `deltaX`, `deltaY` for a swipe | ✅ | `EventRef::swipe_delta` |  |
| `timestamp`, `windowNumber` | ✅ | `EventRef::{timestamp, window_number}` |  |
| borrowed event and retain | ✅ | `EventRef::from_raw_borrowed`, `EventRef::retain`, `Event::{from_raw_retained, as_ref}` | Optional fields return `None` when the event type has no such field. |
| `+keyEventWithType:...`, `+mouseEventWithType:...` | ✅ | `Event::{synthetic_key, synthetic_mouse}` | Only the supported key and mouse types succeed. |
| Global monitors, local monitors and other event families | ⏭️ |  |  |

## NSPasteboard

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `+generalPasteboard` | ✅ | `Pasteboard::general` |  |
| `+pasteboardWithName:` for the find pasteboard | ✅ | `Pasteboard::find` |  |
| `+pasteboardWithName:` with a unique name | ✅ | `Pasteboard::unique` | Uses a UUID name. Drop calls `releaseGlobally`. |
| `types` | ✅ | `Pasteboard::types` | Returns copied type names. |
| `-dataForType:` | ✅ | `Pasteboard::data_for_type` | Returns owned bytes. |
| `-stringForType:` | ✅ | `Pasteboard::string_for_type` | Returns owned text. |
| `-propertyListForType:` with legacy file names | ✅ | `Pasteboard::legacy_filenames` | Returns `PathBuf` values from UTF-8 bytes. |
| `-clearContents` | ✅ | `Pasteboard::clear_contents` |  |
| `-declareTypes:owner:` | ✅ | `Pasteboard::declare_types` | Uses no owner. |
| `-setData:forType:` | ✅ | `Pasteboard::set_data_for_type` |  |
| `-setString:forType:` | ✅ | `Pasteboard::set_string_for_type` |  |
| `-setPropertyList:forType:` with legacy file names | ✅ | `Pasteboard::set_filenames` | Each path must contain valid UTF-8 without a null byte. |
| `changeCount` | ✅ | `Pasteboard::change_count` |  |
| Common pasteboard type identifiers | ✅ | `pasteboard::TYPE_STRING`, `TYPE_PNG`, `TYPE_FILE_URL`, `TYPE_LEGACY_FILENAMES` | The module also defines image type identifiers. |
| Other pasteboard items, promises and file URL writes | ⏭️ |  |  |

## NSDraggingDestination

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| destination registration on a window | ✅ | `DragRegistration::new`, `DragRegistration::deactivate_callbacks` | The registration needs a hosted window and hosted view. Deactivation detaches it. |
| `-draggingEntered:` | ✅ | `DragEvent::Entered`, `DragRegistration::set_handler`, `DragHandler` | The handler gives an accepted `DragOperation`. |
| `-draggingUpdated:` | ✅ | `DragEvent::Updated` |  |
| `-draggingExited:` | ✅ | `DragEvent::Exited` |  |
| `-prepareForDragOperation:` | ✅ | `DragRegistration::new` | The native owner checks the last accepted operation. |
| `-performDragOperation:` | ✅ | `DragEvent::Performed` |  |
| `-concludeDragOperation:` | ✅ | `DragEvent::Concluded` |  |
| `-wantsPeriodicDraggingUpdates` | ✅ | `DragRegistration::new` | The native window returns `true`. |
| Other destination methods | ⏭️ |  |  |

## NSDraggingSource

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `-draggingSession:sourceOperationMaskForDraggingContext:` | ✅ | `DragEvent::SourceMask`, `DragContext`, `DragOperation` | The handler selects the operation mask. |
| `-draggingSession:endedAtPoint:operation:` | ✅ | `DragEvent::SourceEnded` | The callback gives the screen position and operation. |
| Other source methods | ⏭️ |  |  |

## NSDraggingItem

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `-initWithPasteboardWriter:` | ✅ | `DragRegistration::start`, `DragItem::{File, Text}` | Needs a left mouse-down event from the hosted window. |
| `NSView.beginDraggingSessionWithItems:event:source:` | ✅ | `DragRegistration::start` | Supports file and text items. |
| Other drag item content | ⏭️ |  |  |

## NSDraggingInfo

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `draggingLocation` | ✅ | `DragInfo::position_in_window` |  |
| `draggingSourceOperationMask`, `draggingSource` | ✅ | `DragInfo::{source_operations, is_source_window}` |  |
| `draggingPasteboard` file paths | ✅ | `DragInfo::paths` | File URLs retain their file-system bytes. |
| Other pasteboard content during a drag | ⏭️ |  |  |

## NSOpenPanel

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| panel construction | ✅ | `OpenPanel::new` | Needs a `MainThreadMarker`. |
| `canChooseFiles`, `canChooseDirectories` | ✅ | `OpenPanel::{set_can_choose_files, set_can_choose_directories}` |  |
| `allowsMultipleSelection` | ✅ | `OpenPanel::set_allows_multiple_selection` |  |
| `resolvesAliases` | ✅ | `OpenPanel::set_resolves_aliases` |  |
| `-beginWithCompletionHandler:` | ✅ | `OpenPanel::begin`, `PanelSession` | Its callback gets owned file paths or `None`. Drop does not cancel the callback. |
| Other open panel filters and accessory views | ⏭️ |  |  |

## NSSavePanel

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| panel construction | ✅ | `SavePanel::new` | Needs a `MainThreadMarker`. |
| `directoryURL` | ✅ | `FilePanel::set_directory` | Open and save panels share this setter. |
| `prompt`, `message` | ✅ | `FilePanel::{set_prompt, set_message}` |  |
| `canCreateDirectories`, `showsHiddenFiles` | ✅ | `FilePanel::{set_can_create_directories, set_shows_hidden_files}` |  |
| `nameFieldStringValue` | ✅ | `SavePanel::set_suggested_name` |  |
| `-beginWithCompletionHandler:` | ✅ | `SavePanel::begin`, `PanelSession` | Its callback gets an owned file path or `None`. Drop does not cancel the callback. |
| `-cancel:` | ✅ | `PanelSession::cancel` | Closes the panel. The callback still runs once. |
| Other save panel options | ⏭️ |  |  |

## NSAlert

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `-init` | ✅ | `Alert::new` |  |
| `alertStyle` | ✅ | `Alert::set_style`, `AlertStyle` |  |
| `messageText` | ✅ | `Alert::set_message` |  |
| `informativeText` | ✅ | `Alert::set_informative_text` |  |
| `-addButtonWithTitle:` | ✅ | `Alert::add_button` | Returns the zero-based button index. |
| `-beginSheetModalForWindow:completionHandler:` | ✅ | `Alert::begin_sheet`, `AlertSheetStart`, `AlertSession` | Its callback gets a button index or `None`. Drop does not cancel the callback. |
| sheet cancellation | ✅ | `AlertSession::cancel` | Closes the alert. A closed parent aborts its sheet. |
| Other alerts and modal runs | ⏭️ |  |  |

## NSAppearance

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| standard `NSAppearanceName` values | ✅ | `AppearanceName::{native_name, from_native_name}` | Includes four accessibility names for matches only. |
| `+appearanceNamed:` | ✅ | `Appearance::named` | Only the four standard names can create an appearance. |
| `-bestMatchFromAppearancesWithNames:` | ✅ | `Appearance::name` | Returns the best match, not necessarily the original name. |
| `NSView.effectiveAppearance`, `NSWindow.effectiveAppearance` | ✅ | `Appearance::{effective_for_view, effective_for_window}` |  |
| `NSView.appearance` | ✅ | `Appearance::{view_override, set_view_override}` | `None` clears an override. High-contrast names cannot set one. |
| Other appearance operations | ⏭️ |  |  |

## NSVisualEffectView

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `-initWithFrame:` | ✅ | `VisualEffectView::{new_for_window, new_clear_for_window, is_clear_blur}` | A clear view removes tint from the blur. |
| `material` | ✅ | `VisualEffectView::{material, set_material}`, `VisualEffectMaterial` |  |
| `blendingMode` | ✅ | `VisualEffectView::{blending_mode, set_blending_mode}`, `VisualEffectBlendingMode` |  |
| `state` | ✅ | `VisualEffectView::{state, set_state}`, `VisualEffectState` |  |
| `isEmphasized` | ✅ | `VisualEffectView::{is_emphasized, set_emphasized}` |  |
| `-addSubview:positioned:relativeTo:`, `-removeFromSuperview` | ✅ | `VisualEffectView::{attach_below, detach}` | Drop detaches the blur view. It does not retain the window. |
| Other materials and effect filters | ⏭️ |  |  |

## NSCursor

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| standard cursor class methods | ✅ | `Cursor::system`, `CursorKind` | Before macOS 15, diagonal cursors use rotated copies of a public cursor. |
| `+currentCursor` | ✅ | `Cursor::current` |  |
| `-initWithImage:hotSpot:` | ✅ | `Cursor::from_image` | Accepts an image, size and hot spot. |
| `-set` | ✅ | `Cursor::set` |  |
| `+hide`, `+unhide` | ✅ | `Cursor::{hide, unhide}` |  |
| `+setHiddenUntilMouseMoves:` | ✅ | `Cursor::set_hidden_until_mouse_moves` |  |
| Custom cursor stacks | ⏭️ |  |  |

## NSWorkspace

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `-openURL:` | ✅ | `Workspace::open_url` |  |
| `-openURL:` with a file URL | ✅ | `Workspace::open_file` | Accepts a file-system path. |
| `-activateFileViewerSelectingURLs:` | ✅ | `Workspace::reveal_paths` | Accepts calls from any thread. |
| `-iconForFile:` | ✅ | `Workspace::icon_for_file`, `WorkspaceIcon::size` |  |
| `-iconForContentType:` | ✅ | `Workspace::icon_for_file_type` |  |
| `-URLForApplicationWithBundleIdentifier:`, `-iconForFile:` | ✅ | `Workspace::icon_for_application` | Needs an installed application. |
| `NSWorkspaceWillSleepNotification`, `NSWorkspaceDidWakeNotification` | ✅ | `Workspace::observe_power`, `PowerObserver`, `PowerEvent` | Drop the observer to stop callbacks. |
| `-setDefaultApplicationAtURL:toOpenURLsWithScheme:completionHandler:` | ✅ | `Workspace::register_default_url_scheme`, `SchemeRegistration`, `SchemeRegistrationError` | Needs an installed application and bundle identifier. Drop does not cancel the completion. |
| URL scheme completion cancellation | ✅ | `SchemeRegistration::cancel_completion` | Stops the callback, not the native registration request. |
| `accessibilityDisplayShouldReduceMotion` | ✅ | `Workspace::accessibility_display_should_reduce_motion` |  |
| `accessibilityDisplayOptionsDidChangeNotification` | ✅ | `Workspace::observe_accessibility_display`, `AccessibilityDisplayObserver` | Main-thread delivery. Drop removes the registration. |
| `accessibilityDisplayShouldReduceTransparency` | ✅ | `Workspace::accessibility_display_should_reduce_transparency` |  |
| `accessibilityDisplayShouldIncreaseContrast` | ✅ | `Workspace::accessibility_display_should_increase_contrast` |  |
| `accessibilityDisplayShouldDifferentiateWithoutColor` | ✅ | `Workspace::accessibility_display_should_differentiate_without_color` |  |
| `accessibilityDisplayShouldInvertColors` | ✅ | `Workspace::accessibility_display_should_invert_colors` |  |
| `isVoiceOverEnabled` | ✅ | `Workspace::is_voice_over_enabled` |  |
| Other workspace operations | ⏭️ |  |  |

## NSDocumentController

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `-noteNewRecentDocumentURL:` | ✅ | `DocumentController::note_new_recent_document` | The path must contain valid, nonempty UTF-8 without a null byte. |
| Other document controller operations | ⏭️ |  |  |

## NSSound

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `+beep` | ✅ | `sound::beep` | Plays the system alert sound on the main thread. |
| Other sound APIs | ⏭️ |  |  |

## NSImage

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `-initWithData:` | ✅ | `Image::from_data` | Data that AppKit cannot read gives `AppKitError::InvalidImageData`. |
| `size` | ✅ | `Image::size`, `WorkspaceIcon::size` |  |
| Other image creation and drawing | ⏭️ |  |  |

## NSScroller

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| `+preferredScrollerStyle` | ✅ | `Scroller::preferred_scroller_style`, `ScrollerStyle` |  |
| Other scroller methods | ⏭️ |  |  |

## NSColor

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| General color constructors and catalogs | ⏭️ |  | `RgbaColor` only sets the background color of a window. |

## NSFont

| Apple API | Status | Rust surface | Notes |
| --- | --- | --- | --- |
| Font selection and metrics | ⏭️ |  |  |
