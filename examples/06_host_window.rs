use std::cell::{Cell, RefCell};
use std::rc::Rc;

use appkit::host_window::{
    HiddenReshowPath, HostedPanelOptions, HostedWindow, HostedWindowConfiguration,
    HostedWindowEvent, WindowStyleMask,
};
use appkit::{CGRect, MainThreadMarker, Screen};

fn main() {
    let main_thread = MainThreadMarker::new().expect("the main thread");
    exercise_window(main_thread);
    exercise_panel(main_thread);
    exercise_self_close(main_thread);
    exercise_hidden_reshow(main_thread);
    exercise_alternate_reshow(main_thread, HiddenReshowPath::OrderFrontRegardless);
    exercise_alternate_reshow(main_thread, HiddenReshowPath::ModalSession);
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

fn exercise_self_close(main_thread: MainThreadMarker) {
    let configuration = HostedWindowConfiguration::new(
        CGRect::new(0.0, 0.0, 180.0, 100.0),
        WindowStyleMask::TITLED | WindowStyleMask::CLOSABLE,
    );
    let host = HostedWindow::new(main_thread, configuration).expect("a hidden hosted window");
    let native = host.as_window();
    let events = Rc::new(RefCell::new(Vec::new()));
    let observed = Rc::clone(&events);
    host.set_handler(move |event| {
        observed.borrow_mut().push(event);
        if event == HostedWindowEvent::DidResize {
            native.close();
        }
    });
    assert!(host.verify_event(HostedWindowEvent::DidResize));
    assert_eq!(
        *events.borrow(),
        [
            HostedWindowEvent::DidResize,
            HostedWindowEvent::Close,
            HostedWindowEvent::WillClose
        ]
    );
    assert!(!host.verify_event(HostedWindowEvent::DidMove));
}

fn exercise_hidden_reshow(main_thread: MainThreadMarker) {
    let configuration = HostedWindowConfiguration::new(
        CGRect::new(-10_000.0, -10_000.0, 180.0, 100.0),
        WindowStyleMask::TITLED | WindowStyleMask::CLOSABLE,
    );
    let host = HostedWindow::new(main_thread, configuration).expect("a hidden hosted window");
    let events = Rc::new(RefCell::new(Vec::new()));
    let observed = Rc::clone(&events);
    host.set_handler(move |event| observed.borrow_mut().push(event));
    host.as_window().close();
    assert_eq!(
        *events.borrow(),
        [HostedWindowEvent::Close, HostedWindowEvent::WillClose]
    );
    assert_eq!(host.verify_should_close(), None);
    assert!(host.verify_hidden_reshow());
    let close_events: Vec<_> = events
        .borrow()
        .iter()
        .copied()
        .filter(|event| {
            matches!(
                event,
                HostedWindowEvent::Close | HostedWindowEvent::WillClose
            )
        })
        .collect();
    assert_eq!(
        close_events,
        [
            HostedWindowEvent::Close,
            HostedWindowEvent::WillClose,
            HostedWindowEvent::Close,
            HostedWindowEvent::WillClose
        ]
    );
    assert_eq!(host.verify_should_close(), None);
}

fn exercise_alternate_reshow(main_thread: MainThreadMarker, path: HiddenReshowPath) {
    let mut style = WindowStyleMask::TITLED | WindowStyleMask::CLOSABLE;
    if path == HiddenReshowPath::ModalSession {
        style |= WindowStyleMask::NONACTIVATING_PANEL;
    }
    let mut configuration =
        HostedWindowConfiguration::new(CGRect::new(-10_000.0, -10_000.0, 180.0, 100.0), style);
    configuration.can_become_main = false;
    configuration.can_become_key = false;
    let host = match path {
        HiddenReshowPath::OrderFrontRegardless => HostedWindow::new(main_thread, configuration),
        HiddenReshowPath::ModalSession => {
            HostedWindow::new_panel(main_thread, configuration, HostedPanelOptions::default())
        }
    }
    .expect("a hidden hosted window");
    let events = Rc::new(RefCell::new(Vec::new()));
    let observed = Rc::clone(&events);
    host.set_handler(move |event| observed.borrow_mut().push(event));
    let should_close_calls = Rc::new(Cell::new(0));
    let calls = Rc::clone(&should_close_calls);
    host.set_should_close_handler(move || {
        calls.set(calls.get() + 1);
        calls.get() == 2
    });
    host.as_window().close();
    assert_eq!(
        *events.borrow(),
        [HostedWindowEvent::Close, HostedWindowEvent::WillClose]
    );
    assert_eq!(host.verify_should_close(), None);
    assert!(
        host.verify_hidden_reshow_with_veto(path),
        "{path:?} must preserve the close veto and the next close"
    );
    assert_eq!(should_close_calls.get(), 2);
    assert!(events.borrow().contains(&HostedWindowEvent::DidResize));
    let close_events: Vec<_> = events
        .borrow()
        .iter()
        .copied()
        .filter(|event| {
            matches!(
                event,
                HostedWindowEvent::Close | HostedWindowEvent::WillClose
            )
        })
        .collect();
    assert_eq!(
        close_events,
        [
            HostedWindowEvent::Close,
            HostedWindowEvent::WillClose,
            HostedWindowEvent::Close,
            HostedWindowEvent::WillClose
        ]
    );
    assert_eq!(host.verify_should_close(), None);
}
