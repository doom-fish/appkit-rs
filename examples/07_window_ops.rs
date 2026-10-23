use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

use appkit::window_ops::{
    RgbaColor, WindowButton, WindowCollectionBehavior, WindowLevel, WindowStyleMask,
    WindowTabbingMode,
};
use appkit::{AppKitError, CGPoint, CGRect, CGSize, MainThreadMarker, Window};

fn main() -> Result<(), AppKitError> {
    let main_thread = MainThreadMarker::new().expect("the example starts on the main thread");
    let window = Window::new(
        main_thread,
        CGRect::new(120.0, 180.0, 400.0, 280.0),
        "Hidden window operations",
    )?;

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
    window.set_style_mask(original_mask | WindowStyleMask::FULL_SIZE_CONTENT_VIEW);
    assert!(window
        .style_mask()
        .contains(WindowStyleMask::FULL_SIZE_CONTENT_VIEW));
    window.set_style_mask(original_mask);

    let original_level = window.level();
    window.set_level(WindowLevel::Floating);
    assert_eq!(window.level(), WindowLevel::Floating);
    window.set_level(original_level);

    let original_behavior = window.collection_behavior();
    window.set_collection_behavior(WindowCollectionBehavior::CAN_JOIN_ALL_SPACES);
    assert!(window
        .collection_behavior()
        .contains(WindowCollectionBehavior::CAN_JOIN_ALL_SPACES));
    window.set_collection_behavior(original_behavior);

    let original_mode = window.tabbing_mode();
    window.set_tabbing_mode(WindowTabbingMode::Disallowed);
    assert_eq!(window.tabbing_mode(), WindowTabbingMode::Disallowed);
    window.set_tabbing_mode(original_mode);
    window.set_tabbing_identifier(Some("hidden-window-ops"))?;
    assert_eq!(
        window.tabbing_identifier().as_deref(),
        Some("hidden-window-ops")
    );
    window.set_tabbing_identifier(None)?;
    assert!(window.tabbing_identifier().is_none());

    window.set_frame(CGRect::new(200.0, 220.0, 420.0, 300.0), false, false);
    window.set_content_size(CGSize::new(380.0, 240.0));
    assert_eq!(window.content_size(), CGSize::new(380.0, 240.0));
    window.set_minimum_content_size(CGSize::new(200.0, 160.0));
    assert_eq!(window.minimum_content_size(), CGSize::new(200.0, 160.0));

    let content_rect = CGRect::new(30.0, 40.0, 250.0, 180.0);
    let frame_rect = window.frame_rect_for_content_rect(content_rect);
    assert_eq!(window.content_rect_for_frame_rect(frame_rect), content_rect);
    let _ = window.convert_point_to_screen(CGPoint::new(8.0, 12.0));
    let _ = window.convert_rect_to_screen(content_rect);
    let _ = window.mouse_location_outside_event_stream();

    window.set_background_color(RgbaColor::new(0.25, 0.5, 0.75, 1.0));
    assert!(window.background_color().is_some());
    window.set_document_edited(true);
    assert!(window.is_document_edited());
    window.set_document_edited(false);

    let filename = std::env::current_dir()
        .expect("the example has a worktree")
        .join("Cargo.toml");
    window.set_represented_filename(Some(&filename))?;
    assert_eq!(
        window.represented_filename().as_deref(),
        Some(filename.as_path())
    );
    let bytes_filename = std::env::current_dir()
        .expect("the example has a worktree")
        .join(OsStr::from_bytes(b"hidden-window-\xff.txt"));
    if window
        .set_represented_filename(Some(&bytes_filename))
        .is_ok()
    {
        assert_eq!(
            window.represented_filename().as_deref(),
            Some(bytes_filename.as_path())
        );
    } else {
        assert_eq!(
            window.represented_filename().as_deref(),
            Some(filename.as_path())
        );
    }
    window.set_represented_filename(None)?;
    assert!(window.represented_filename().is_none());

    if let Some(close_button) = window.standard_window_button(WindowButton::Close) {
        let _ = close_button.frame();
        let _ = close_button.is_hidden();
        let _ = close_button.superview();
    }

    assert!(!window
        .occlusion_state()
        .contains(appkit::window_ops::WindowOcclusionState::VISIBLE));
    assert!(!window.is_visible());
    assert!(!window.is_key_window());
    println!("Window {} stays hidden.", window.window_number());
    Ok(())
}
