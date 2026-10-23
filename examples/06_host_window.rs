use std::cell::{Cell, RefCell};
use std::rc::Rc;

use appkit::host_window::{
    HostedPanelOptions, HostedWindow, HostedWindowConfiguration, HostedWindowEvent, WindowStyleMask,
};
use appkit::{CGRect, MainThreadMarker, Screen};

fn main() {
    let main_thread = MainThreadMarker::new().expect("the main thread");
    exercise_window(main_thread);
    exercise_panel(main_thread);
    println!("The hidden window and panel stop callbacks after close.");
}

fn exercise_window(main_thread: MainThreadMarker) {
    let screen = Screen::main(main_thread);
    let mut configuration = HostedWindowConfiguration::new(
        CGRect::new(0.0, 0.0, 320.0, 200.0),
        WindowStyleMask::TITLED | WindowStyleMask::CLOSABLE | WindowStyleMask::RESIZABLE,
    );
    configuration.screen = screen.as_ref();
    let host =
        Rc::new(HostedWindow::new(main_thread, configuration).expect("a hidden hosted window"));
    assert!(host.can_become_main());
    host.set_can_become_main(false);
    assert!(!host.can_become_main());
    host.set_can_become_key(false);
    assert!(!host.can_become_key());
    host.set_can_become_key(true);

    let events = Rc::new(RefCell::new(Vec::new()));
    let observed = Rc::clone(&events);
    let nested = Rc::downgrade(&host);
    host.set_handler(move |event| {
        observed.borrow_mut().push(event);
        if event == HostedWindowEvent::DidMove {
            assert!(nested
                .upgrade()
                .unwrap()
                .verify_event(HostedWindowEvent::DidResize));
        }
    });
    assert!(host.verify_event(HostedWindowEvent::DidMove));
    assert_eq!(
        *events.borrow(),
        [HostedWindowEvent::DidMove, HostedWindowEvent::DidResize]
    );

    for event in [
        HostedWindowEvent::DidChangeScreen,
        HostedWindowEvent::DidChangeBackingProperties,
        HostedWindowEvent::DidBecomeKey,
        HostedWindowEvent::DidResignKey,
        HostedWindowEvent::DidChangeOcclusionState,
        HostedWindowEvent::WillEnterFullScreen,
        HostedWindowEvent::DidEnterFullScreen,
        HostedWindowEvent::DidFailToEnterFullScreen,
        HostedWindowEvent::WillExitFullScreen,
        HostedWindowEvent::DidExitFullScreen,
        HostedWindowEvent::DidFailToExitFullScreen,
        HostedWindowEvent::AddedTitlebarAccessory,
        HostedWindowEvent::MoveTabToNewWindow,
        HostedWindowEvent::MergeAllWindows,
        HostedWindowEvent::SelectNextTab,
        HostedWindowEvent::SelectPreviousTab,
        HostedWindowEvent::ToggleTabBar,
    ] {
        assert!(host.verify_event(event));
        assert_eq!(*events.borrow().last().unwrap(), event);
    }

    let close_calls = Rc::new(Cell::new(0));
    let calls = Rc::clone(&close_calls);
    let nested = Rc::downgrade(&host);
    host.set_should_close_handler(move || {
        calls.set(calls.get() + 1);
        assert_eq!(nested.upgrade().unwrap().verify_should_close(), Some(false));
        false
    });
    assert_eq!(host.verify_should_close(), Some(false));
    assert_eq!(close_calls.get(), 1);
    host.clear_should_close_handler();
    assert_eq!(host.verify_should_close(), Some(true));

    let window = host.as_window();
    window.close();
    assert!(events.borrow().contains(&HostedWindowEvent::Close));
    assert!(events.borrow().contains(&HostedWindowEvent::WillClose));
    let count = events.borrow().len();
    assert_eq!(host.verify_should_close(), None);
    assert!(!host.verify_event(HostedWindowEvent::DidMove));
    drop(host);
    window.close();
    assert_eq!(events.borrow().len(), count);
}

fn exercise_panel(main_thread: MainThreadMarker) {
    let panel_configuration = HostedWindowConfiguration::new(
        CGRect::new(0.0, 0.0, 200.0, 120.0),
        WindowStyleMask::TITLED | WindowStyleMask::CLOSABLE | WindowStyleMask::NONACTIVATING_PANEL,
    );
    let panel = HostedWindow::new_panel(
        main_thread,
        panel_configuration,
        HostedPanelOptions {
            becomes_key_only_if_needed: Some(true),
            hides_on_deactivate: Some(false),
            ..HostedPanelOptions::default()
        },
    )
    .expect("a hidden hosted panel");
    let panel_calls = Rc::new(Cell::new(0));
    let calls = Rc::clone(&panel_calls);
    panel.set_handler(move |_| calls.set(calls.get() + 1));
    assert!(panel.can_become_key());
    assert!(panel.verify_event(HostedWindowEvent::DidResize));
    assert_eq!(panel_calls.get(), 1);
    let native_panel = panel.as_window();
    drop(panel);
    native_panel.close();
    assert_eq!(panel_calls.get(), 1);
}
