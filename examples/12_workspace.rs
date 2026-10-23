use std::cell::{Cell, RefCell};
use std::ptr;
use std::rc::Rc;
use std::time::Duration;

use appkit::appearance::Appearance;
use appkit::application::PresentationOptions;
use appkit::blur::{
    VisualEffectBlendingMode, VisualEffectMaterial, VisualEffectState, VisualEffectView,
};
use appkit::ffi::workspace::{
    appkit_workspace_power_observer_deliver, appkit_workspace_power_observer_notify,
};
use appkit::workspace::{PowerEvent, SchemeRegistrationError};
use appkit::{
    AppKitError, AppearanceName, Application, CGRect, MainThreadMarker, Window, Workspace,
};
use apple_cf::cf::CFRunLoop;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let main_thread = MainThreadMarker::new().expect("the example runs on the main thread");
    let application = Application::shared(main_thread);
    let window = Window::new(
        main_thread,
        CGRect::new(0.0, 0.0, 320.0, 240.0),
        "Workspace example",
    )?;
    check_hidden_objects(main_thread, &application, &window)?;
    check_power_observers(main_thread);

    assert!(matches!(
        Workspace::register_default_url_scheme(main_thread, "9invalid", |_| {}),
        Err(SchemeRegistrationError::InvalidScheme)
    ));
    println!("The workspace callback and hidden AppKit objects are ready.");
    Ok(())
}

fn check_hidden_objects(
    main_thread: MainThreadMarker,
    application: &Application,
    window: &Window,
) -> Result<(), AppKitError> {
    let content = window
        .content_view()
        .expect("the hidden window has a content view");
    let _ = application.key_window();
    let _ = application.main_window();
    let _ = application.ordered_windows();
    let _ = application.is_active();
    let _ = application.presentation_options();
    let _ = application.appearance();
    let _ = application.effective_appearance();
    let _ = Workspace::icon_for_file_type(main_thread, "public.folder")?;
    let presentation = application.presentation_options();
    assert!(!application.set_presentation_options(PresentationOptions::HIDE_MENU_BAR));
    assert_eq!(application.presentation_options(), presentation);
    let _ = application.change_windows_item(window, "/an/ordinary/path")?;

    let aqua = Appearance::named(main_thread, AppearanceName::Aqua).expect("Aqua is available");
    assert_eq!(aqua.name(), Some(AppearanceName::Aqua));
    assert_eq!(
        AppearanceName::from_native_name(AppearanceName::Aqua.native_name()),
        Some(AppearanceName::Aqua)
    );
    assert!(Appearance::effective_for_window(window).is_some());
    assert!(Appearance::effective_for_view(&content).is_some());
    assert!(
        Appearance::named(main_thread, AppearanceName::AccessibilityHighContrastAqua).is_none()
    );
    let application_appearance = application.appearance();
    assert!(!application.set_appearance(Some(AppearanceName::AccessibilityHighContrastAqua)));
    assert_eq!(application.appearance(), application_appearance);
    assert!(!Appearance::set_view_override(
        &content,
        Some(AppearanceName::AccessibilityHighContrastAqua)
    ));
    assert!(Appearance::view_override(&content).is_none());

    let blur = VisualEffectView::new_for_window(main_thread, window)
        .expect("the hidden window has a content area");
    let clear = VisualEffectView::new_clear_for_window(main_thread, window)
        .expect("the hidden window has a content area");
    assert!(!blur.is_clear_blur());
    assert!(clear.is_clear_blur());
    assert_eq!(
        blur.material(),
        Some(VisualEffectMaterial::UnderWindowBackground)
    );
    assert_eq!(
        blur.blending_mode(),
        Some(VisualEffectBlendingMode::BehindWindow)
    );
    assert_eq!(blur.state(), Some(VisualEffectState::Active));
    blur.set_state(VisualEffectState::FollowsWindowActiveState);
    assert_eq!(
        blur.state(),
        Some(VisualEffectState::FollowsWindowActiveState)
    );
    blur.set_emphasized(true);
    assert!(blur.is_emphasized());
    blur.set_emphasized(false);
    assert!(!blur.attach_below(&content));
    assert!(!clear.attach_below(&content));
    blur.detach();
    clear.detach();
    Ok(())
}

fn check_power_observers(main_thread: MainThreadMarker) {
    let delivered = Rc::new(RefCell::new(Vec::new()));
    let observer_address = Rc::new(Cell::new(ptr::null_mut()));
    let events = Rc::clone(&delivered);
    let address = Rc::clone(&observer_address);
    let observer = Workspace::observe_power(main_thread, move |event| {
        events.borrow_mut().push(event);
        if event == PowerEvent::WillSleep {
            unsafe {
                appkit_workspace_power_observer_deliver(address.get(), PowerEvent::DidWake as i32);
            };
        }
    })
    .expect("the workspace accepts a power observer");
    observer_address.set(observer.as_ptr());
    unsafe {
        appkit_workspace_power_observer_deliver(observer.as_ptr(), PowerEvent::WillSleep as i32);
    };
    assert_eq!(
        delivered.borrow().as_slice(),
        &[PowerEvent::WillSleep, PowerEvent::DidWake]
    );
    drop(observer);

    let closing_events = Rc::new(RefCell::new(Vec::new()));
    let closing_observer = Rc::new(RefCell::new(None::<appkit::workspace::PowerObserver>));
    let received = Rc::clone(&closing_events);
    let slot = Rc::clone(&closing_observer);
    let observer = Workspace::observe_power(main_thread, move |event| {
        received.borrow_mut().push(event);
        if event == PowerEvent::WillSleep {
            let address = slot
                .borrow()
                .as_ref()
                .expect("the observer is live")
                .as_ptr();
            unsafe {
                appkit_workspace_power_observer_deliver(address, PowerEvent::DidWake as i32);
            }
            slot.borrow_mut().take();
        }
    })
    .expect("the workspace accepts a power observer");
    let address = observer.as_ptr();
    *closing_observer.borrow_mut() = Some(observer);
    unsafe {
        appkit_workspace_power_observer_deliver(address, PowerEvent::WillSleep as i32);
    }
    assert!(closing_observer.borrow().is_none());
    assert_eq!(
        closing_events.borrow().as_slice(),
        &[PowerEvent::WillSleep, PowerEvent::DidWake]
    );

    let deferred = Rc::new(Cell::new(0_usize));
    let calls = Rc::clone(&deferred);
    let observer = Workspace::observe_power(main_thread, move |_| calls.set(calls.get() + 1))
        .expect("the workspace accepts a power observer");
    unsafe {
        appkit_workspace_power_observer_notify(observer.as_ptr(), PowerEvent::WillSleep as i32);
    }
    assert_eq!(deferred.get(), 0);
    for _ in 0..10 {
        if deferred.get() == 1 {
            break;
        }
        let _ = CFRunLoop::run_in_default_mode(Duration::from_millis(20), false);
    }
    assert_eq!(deferred.get(), 1);
    unsafe {
        appkit_workspace_power_observer_notify(observer.as_ptr(), PowerEvent::DidWake as i32);
    }
    drop(observer);
    let _ = CFRunLoop::run_in_default_mode(Duration::from_millis(20), false);
    assert_eq!(deferred.get(), 1);
}
