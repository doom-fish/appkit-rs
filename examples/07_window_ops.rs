use std::cell::Cell;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

use appkit::ffi::window_ops::{
    appkit_window_ops_probe_closed_simple_full_screen_responder,
    appkit_window_ops_probe_sheet_completion, SheetCallbackTableV1,
};
use appkit::window_ops::{
    RgbaColor, SheetResponse, WindowButton, WindowCollectionBehavior, WindowLevel, WindowStyleMask,
    WindowTabbingMode,
};
use appkit::{
    AppKitError, AppearanceName, CGPoint, CGRect, CGSize, CallbackTableHeader, MainThreadMarker,
    Window,
};

thread_local! {
    static PROBE_RESPONSE: Cell<Option<(u64, i64)>> = const { Cell::new(None) };
    static PROBE_DEACTIVATED: Cell<u64> = const { Cell::new(0) };
}

extern "C" fn probe_complete(token: u64, response: i64) {
    let _ = PROBE_RESPONSE.try_with(|result| result.set(Some((token, response))));
}

extern "C" fn probe_deactivate(token: u64) {
    let _ = PROBE_DEACTIVATED.try_with(|result| result.set(token));
}

static PROBE_CALLBACKS: SheetCallbackTableV1 = SheetCallbackTableV1 {
    header: CallbackTableHeader {
        version: 1,
        size: size_of::<SheetCallbackTableV1>(),
    },
    deactivate: probe_deactivate,
    complete: probe_complete,
};

fn check_window_properties(window: &Window) -> Result<(), AppKitError> {
    assert!(!window.is_visible());
    assert!(!window.is_key_window());
    assert!(!window.is_main_window());
    window.retain_when_closed();
    assert!(!window.is_released_when_closed());
    assert!(!window.is_simple_full_screen());
    assert!(window.attached_sheet().is_none());
    assert!(window.sheet_parent().is_none());
    assert!(window.style_mask().contains(WindowStyleMask::TITLED));

    let original_mask = window.style_mask();
    window.set_style_mask(original_mask | WindowStyleMask::FULL_SIZE_CONTENT_VIEW)?;
    assert!(window
        .style_mask()
        .contains(WindowStyleMask::FULL_SIZE_CONTENT_VIEW));
    window.set_style_mask(original_mask)?;
    assert!(window
        .set_style_mask(original_mask | WindowStyleMask::FULL_SCREEN)
        .is_err());
    assert_eq!(window.style_mask(), original_mask);

    let original_level = window.level();
    window.set_level(WindowLevel::Floating)?;
    assert_eq!(window.level(), WindowLevel::Floating);
    window.set_level(original_level)?;

    let original_behavior = window.collection_behavior();
    window.set_collection_behavior(WindowCollectionBehavior::CAN_JOIN_ALL_SPACES)?;
    assert!(window
        .collection_behavior()
        .contains(WindowCollectionBehavior::CAN_JOIN_ALL_SPACES));
    assert!(window
        .set_collection_behavior(
            WindowCollectionBehavior::CAN_JOIN_ALL_SPACES
                | WindowCollectionBehavior::MOVE_TO_ACTIVE_SPACE
        )
        .is_err());
    assert_eq!(
        window.collection_behavior(),
        WindowCollectionBehavior::CAN_JOIN_ALL_SPACES
    );
    window.set_collection_behavior(original_behavior)?;

    let default_tabbing_identifier = window.tabbing_identifier();
    assert!(default_tabbing_identifier
        .as_ref()
        .is_some_and(|identifier| !identifier.is_empty()));
    let original_mode = window.tabbing_mode();
    window.set_tabbing_mode(WindowTabbingMode::Disallowed)?;
    assert_eq!(window.tabbing_mode(), WindowTabbingMode::Disallowed);
    window.set_tabbing_mode(original_mode)?;
    window.set_tabbing_identifier(Some("hidden-window-ops"))?;
    assert_eq!(
        window.tabbing_identifier().as_deref(),
        Some("hidden-window-ops")
    );
    window.set_tabbing_identifier(None)?;
    assert_eq!(window.tabbing_identifier(), default_tabbing_identifier);

    window.set_frame(CGRect::new(200.0, 220.0, 420.0, 300.0), false, false)?;
    let valid_frame = window.frame();
    assert!(window
        .set_frame(CGRect::new(f64::NAN, 220.0, 420.0, 300.0), false, false)
        .is_err());
    assert!(window
        .set_frame(
            CGRect::new(2_147_483_649.0, 220.0, 420.0, 300.0),
            false,
            false
        )
        .is_err());
    assert_eq!(window.frame(), valid_frame);
    window.set_content_size(CGSize::new(380.0, 240.0))?;
    assert_eq!(window.content_size(), CGSize::new(380.0, 240.0));
    assert!(window
        .set_content_size(CGSize::new(f64::NAN, 240.0))
        .is_err());
    assert_eq!(window.content_size(), CGSize::new(380.0, 240.0));
    window.set_minimum_content_size(CGSize::new(200.0, 160.0))?;
    assert_eq!(window.minimum_content_size(), CGSize::new(200.0, 160.0));

    let content_rect = CGRect::new(30.0, 40.0, 250.0, 180.0);
    let frame_rect = window.frame_rect_for_content_rect(content_rect);
    assert_eq!(window.content_rect_for_frame_rect(frame_rect), content_rect);
    let _ = window.convert_point_to_screen(CGPoint::new(8.0, 12.0));
    let _ = window.convert_rect_to_screen(content_rect);
    let _ = window.mouse_location_outside_event_stream();
    Ok(())
}

fn check_window_chrome_and_document(window: &Window) -> Result<(), AppKitError> {
    let original_appearance = window.appearance();
    window.set_appearance(Some(AppearanceName::Aqua))?;
    for appearance in [
        AppearanceName::AccessibilityHighContrastAqua,
        AppearanceName::AccessibilityHighContrastDarkAqua,
        AppearanceName::AccessibilityHighContrastVibrantLight,
        AppearanceName::AccessibilityHighContrastVibrantDark,
    ] {
        assert!(window.set_appearance(Some(appearance)).is_err());
        assert_eq!(window.appearance(), Some(AppearanceName::Aqua));
    }
    window.set_appearance(original_appearance)?;

    window.set_background_color(RgbaColor::new(0.25, 0.5, 0.75, 1.0))?;
    assert!(window.background_color().is_some());
    assert!(window
        .set_background_color(RgbaColor::new(f64::NAN, 0.5, 0.75, 1.0))
        .is_err());
    window.set_document_edited(true);
    assert!(window.is_document_edited());
    window.set_document_edited(false);

    let filename = std::env::current_dir()
        .expect("the example has a worktree")
        .join(OsStr::from_bytes(b"R\xc3\xa4kning.txt"));
    window.set_represented_filename(Some(&filename))?;
    let represented = window
        .represented_filename()
        .expect("the window has a represented filename");
    assert!(matches!(
        represented.file_name(),
        Some(name) if name == OsStr::new("Räkning.txt") || name == OsStr::new("Ra\u{308}kning.txt")
    ));
    let bytes_filename = std::env::current_dir()
        .expect("the example has a worktree")
        .join(OsStr::from_bytes(b"hidden-window-\xff.txt"));
    assert!(window
        .set_represented_filename(Some(&bytes_filename))
        .is_err());
    assert_eq!(window.represented_filename(), Some(represented));
    window.set_represented_filename(None)?;
    assert!(window.represented_filename().is_none());

    if let Some(close_button) = window.standard_window_button(WindowButton::Close) {
        let frame = close_button.frame();
        assert!(close_button
            .set_frame(CGRect::new(f64::NAN, 0.0, 10.0, 10.0))
            .is_err());
        assert_eq!(close_button.frame(), frame);
        let _ = close_button.is_hidden();
        let _ = close_button.superview();
    }

    assert!(!window
        .occlusion_state()
        .contains(appkit::window_ops::WindowOcclusionState::VISIBLE));
    assert!(!window.is_visible());
    assert!(!window.is_key_window());
    Ok(())
}

fn check_simple_full_screen(main_thread: MainThreadMarker) -> Result<(), AppKitError> {
    let simple_window = Window::new(
        main_thread,
        CGRect::new(200.0, 180.0, 360.0, 240.0),
        "Hidden simple full screen",
    )?;
    let original_frame = simple_window.frame();
    let original_style = simple_window.style_mask();
    let view = simple_window
        .content_view()
        .expect("the window has a content view");
    assert!(simple_window.toggle_simple_full_screen(&view));
    assert!(simple_window.is_simple_full_screen());
    assert!(!simple_window.is_visible());
    assert!(!simple_window.is_key_window());
    simple_window.close();
    assert!(simple_window.is_simple_full_screen());
    assert!(simple_window.toggle_simple_full_screen(&view));
    assert!(!simple_window.is_simple_full_screen());
    assert_eq!(simple_window.style_mask(), original_style);
    assert_eq!(simple_window.frame(), original_frame);
    assert!(!simple_window.is_visible());
    assert!(!simple_window.is_key_window());
    assert!(unsafe { appkit_window_ops_probe_closed_simple_full_screen_responder() });
    Ok(())
}

fn check_sheet_abort(main_thread: MainThreadMarker) -> Result<(), AppKitError> {
    enum Stop {
        ParentClose,
        SheetClose,
        SheetOrderOut,
    }

    for (index, stop) in [Stop::ParentClose, Stop::SheetClose, Stop::SheetOrderOut]
        .into_iter()
        .enumerate()
    {
        let parent = Window::new(
            main_thread,
            CGRect::new(200.0, 200.0, 300.0, 200.0),
            "Hidden sheet parent",
        )?;
        let sheet = Window::new(
            main_thread,
            CGRect::new(200.0, 200.0, 240.0, 160.0),
            "Hidden sheet",
        )?;
        let token = (index + 1) as u64;
        PROBE_RESPONSE.with(|result| result.set(None));
        PROBE_DEACTIVATED.with(|result| result.set(0));
        assert!(unsafe {
            appkit_window_ops_probe_sheet_completion(
                parent.as_ptr(),
                sheet.as_ptr(),
                token,
                &raw const PROBE_CALLBACKS,
            )
        });
        assert!(!parent.is_visible());
        assert!(!sheet.is_visible());
        assert!(!parent.is_key_window());
        assert!(!sheet.is_key_window());

        match stop {
            Stop::ParentClose => parent.close(),
            Stop::SheetClose => sheet.close(),
            Stop::SheetOrderOut => sheet.order_out(),
        }
        assert_eq!(
            PROBE_RESPONSE.with(Cell::get),
            Some((token, SheetResponse::Aborted.raw()))
        );
        assert_eq!(PROBE_DEACTIVATED.with(Cell::get), token);
        assert!(!parent.is_visible());
        assert!(!sheet.is_visible());
        assert!(!parent.is_key_window());
        assert!(!sheet.is_key_window());
    }
    Ok(())
}

fn main() -> Result<(), AppKitError> {
    let main_thread = MainThreadMarker::new().expect("the example starts on the main thread");
    let window = Window::new(
        main_thread,
        CGRect::new(120.0, 180.0, 400.0, 280.0),
        "Hidden window operations",
    )?;
    check_window_properties(&window)?;
    check_window_chrome_and_document(&window)?;
    check_simple_full_screen(main_thread)?;
    check_sheet_abort(main_thread)?;
    println!("Window {} stays hidden.", window.window_number());
    Ok(())
}
