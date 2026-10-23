use appkit::event::{EventPhase, EventType, ModifierFlags, NS_F35_FUNCTION_KEY};
use appkit::{CGPoint, Event, MainThreadMarker};

fn main() {
    let main_thread = MainThreadMarker::new().expect("the main thread");
    let modifiers = ModifierFlags::COMMAND | ModifierFlags::SHIFT;
    let key = Event::synthetic_key(
        main_thread,
        EventType::KeyDown,
        modifiers,
        12,
        "é\0z",
        "e\0z",
        true,
    )
    .expect("an in-memory key event");
    let key_ref = key.as_ref();
    assert_eq!(key_ref.event_type(), EventType::KeyDown);
    assert_eq!(key_ref.modifier_flags(), modifiers);
    assert_eq!(key_ref.key_code(), Some(12));
    assert_eq!(key_ref.characters().as_deref(), Some("é\0z"));
    assert_eq!(
        key_ref.characters_ignoring_modifiers().as_deref(),
        Some("e\0z")
    );
    assert_eq!(key_ref.is_repeat(), Some(true));
    assert!(key_ref.location_in_window().is_none());
    assert!(key_ref.button_number().is_none());
    assert!(key_ref.click_count().is_none());
    assert!(key_ref.scrolling_delta().is_none());
    assert!(key_ref.has_precise_scrolling_deltas().is_none());
    assert!(key_ref.phase().is_none());
    assert!(key_ref.momentum_phase().is_none());
    assert!(key_ref.pressure().is_none());
    assert!(key_ref.stage().is_none());
    assert!(key_ref.magnification().is_none());
    assert!(key_ref.swipe_delta().is_none());
    assert_eq!(key_ref.window_number(), 0);
    assert!(key_ref.timestamp().is_finite());

    let mouse = Event::synthetic_mouse(
        main_thread,
        EventType::LeftMouseDown,
        CGPoint::new(12.0, 34.0),
        ModifierFlags::CONTROL,
        2,
        0.5,
    )
    .expect("an in-memory mouse event");
    let mouse_ref = mouse.as_ref();
    assert_eq!(mouse_ref.event_type(), EventType::LeftMouseDown);
    assert_eq!(
        mouse_ref.location_in_window(),
        Some(CGPoint::new(12.0, 34.0))
    );
    assert_eq!(mouse_ref.button_number(), Some(0));
    assert_eq!(mouse_ref.click_count(), Some(2));
    assert_eq!(mouse_ref.pressure(), Some(0.5));
    assert!(mouse_ref.key_code().is_none());
    assert!(mouse_ref.characters().is_none());
    assert!(mouse_ref.characters_ignoring_modifiers().is_none());
    assert!(mouse_ref.is_repeat().is_none());
    assert!(mouse_ref.phase().is_none());
    assert!(mouse_ref.stage().is_none());
    assert!(mouse_ref.magnification().is_none());
    let retained = mouse_ref.retain();
    drop(mouse);
    assert_eq!(retained.as_ref().click_count(), Some(2));

    assert_eq!(EventType::from_raw(35), EventType::Other(35));
    assert_eq!(EventType::Other(35).raw_value(), 35);
    assert_eq!(EventPhase::MAY_BEGIN.bits(), 1 << 5);
    assert_eq!(NS_F35_FUNCTION_KEY, 0xF726);
    let _ = Event::current(main_thread);
    let _ = Event::pressed_mouse_buttons(main_thread);
    let _ = Event::modifier_flags(main_thread);
    println!("The event accessors keep key and mouse data safe.");
}
