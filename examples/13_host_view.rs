use core::ffi::c_void;
use core::ptr::NonNull;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use appkit::event::{EventType, ModifierFlags};
use appkit::host_view::{
    HostViewEvent, HostViewNotification, HostViewQuery, HostedView, MetalLayer, TrackingAreaOptions,
};
use appkit::{CGPoint, CGRect, CGSize, Event, MainThreadMarker, Window};

extern "C" {
    fn appkit_retain(object: *mut c_void);
    fn appkit_release(object: *mut c_void);
    fn appkit_host_view_example_arrow_cursor() -> *mut c_void;
    fn appkit_host_view_send_event(view: *mut c_void, selector: u32, event: *mut c_void) -> bool;
    fn appkit_host_view_notify_appearance(view: *mut c_void);
    fn appkit_host_view_display_layer(view: *mut c_void);
    fn appkit_host_view_accepts_first_mouse(view: *mut c_void, event: *mut c_void) -> bool;
    fn appkit_host_view_accepts_first_responder(view: *mut c_void) -> bool;
}

struct Fixture {
    main_thread: MainThreadMarker,
    window: Window,
    view: HostedView,
    events: Rc<RefCell<Vec<(HostViewEvent, EventType)>>>,
    notifications: Rc<RefCell<Vec<HostViewNotification>>>,
    queries: Rc<RefCell<Vec<(HostViewQuery, bool)>>>,
    handled: Rc<Cell<bool>>,
    first_mouse: Rc<Cell<bool>>,
    first_responder: Rc<Cell<bool>>,
}

impl Fixture {
    fn new() -> Self {
        let main_thread = MainThreadMarker::new().expect("the main thread");
        let layer = MetalLayer::new(main_thread).expect("a metal layer");
        let layer_ptr = layer.as_ptr();
        let window = Window::new(
            main_thread,
            CGRect::new(0.0, 0.0, 120.0, 90.0),
            "Hidden hosted view",
        )
        .expect("a hidden window");
        assert!(unsafe {
            HostedView::new(
                main_thread,
                CGRect::new(f64::NAN, 0.0, 120.0, 90.0),
                layer_ptr,
            )
        }
        .is_none());
        let view =
            unsafe { HostedView::new(main_thread, CGRect::new(0.0, 0.0, 120.0, 90.0), layer_ptr) }
                .expect("a hosted view");
        let parent = window.content_view().expect("a content view");
        assert!(view.attach_to(&parent));
        assert!(!view.attach_to(&view.as_view()));
        assert_eq!(
            view.as_view().window().map(|attached| attached.as_ptr()),
            Some(window.as_ptr())
        );
        assert!(view.as_view().wants_layer());
        assert_eq!(view.layer_ptr(), Some(layer_ptr));
        drop(layer);
        assert_eq!(view.layer_ptr(), Some(layer_ptr));

        let events = Rc::new(RefCell::new(Vec::new()));
        let handled = Rc::new(Cell::new(true));
        view.set_event_handler(Some(Box::new({
            let events = events.clone();
            let handled = handled.clone();
            move |selector, event| {
                events.borrow_mut().push((selector, event.event_type()));
                handled.get()
            }
        })));
        let notifications = Rc::new(RefCell::new(Vec::new()));
        view.set_notification_handler(Some(Box::new({
            let notifications = notifications.clone();
            move |notification| {
                notifications.borrow_mut().push(notification);
            }
        })));
        let queries = Rc::new(RefCell::new(Vec::new()));
        let first_mouse = Rc::new(Cell::new(true));
        let first_responder = Rc::new(Cell::new(true));
        view.set_query_handler(Some(Box::new({
            let queries = queries.clone();
            let first_mouse = first_mouse.clone();
            let first_responder = first_responder.clone();
            move |query, event| {
                queries.borrow_mut().push((query, event.is_some()));
                match query {
                    HostViewQuery::AcceptsFirstMouse => first_mouse.get(),
                    HostViewQuery::AcceptsFirstResponder => first_responder.get(),
                }
            }
        })));
        Self {
            main_thread,
            window,
            view,
            events,
            notifications,
            queries,
            handled,
            first_mouse,
            first_responder,
        }
    }

    fn verify_tracking(&self) {
        let options = TrackingAreaOptions::MOUSE_ENTERED_AND_EXITED
            | TrackingAreaOptions::MOUSE_MOVED
            | TrackingAreaOptions::ACTIVE_ALWAYS
            | TrackingAreaOptions::IN_VISIBLE_RECT;
        assert!(self.view.set_tracking_options(options));
        assert_eq!(self.view.tracking_options(), options);
        assert!(!self
            .view
            .set_tracking_options(TrackingAreaOptions::MOUSE_MOVED));
        assert_eq!(self.view.tracking_options(), options);
        assert!(!self
            .view
            .set_tracking_options(options | TrackingAreaOptions::ACTIVE_IN_KEY_WINDOW));
        assert_eq!(self.view.tracking_options(), options);
        let key_window_options = TrackingAreaOptions::MOUSE_MOVED
            | TrackingAreaOptions::ACTIVE_IN_KEY_WINDOW
            | TrackingAreaOptions::IN_VISIBLE_RECT;
        assert!(self.view.set_tracking_options(key_window_options));
        assert_eq!(self.view.tracking_options(), key_window_options);
    }

    fn verify_events(&self) -> (Event, Event) {
        let key = Event::synthetic_key(
            self.main_thread,
            EventType::KeyDown,
            ModifierFlags::COMMAND,
            12,
            "q",
            "q",
            false,
        )
        .expect("a key event");
        let key_up = Event::synthetic_key(
            self.main_thread,
            EventType::KeyUp,
            ModifierFlags::COMMAND,
            12,
            "q",
            "q",
            false,
        )
        .expect("a key-up event");
        let mouse = Event::synthetic_mouse(
            self.main_thread,
            EventType::MouseMoved,
            CGPoint::new(8.0, 13.0),
            ModifierFlags::default(),
            0,
            0.0,
        )
        .expect("a mouse event");
        assert!(self.view.perform_key_equivalent(key.as_ref()));
        self.handled.set(false);
        assert!(!self.view.perform_key_equivalent(key.as_ref()));
        self.handled.set(true);
        self.view.send_event(HostViewEvent::KeyDown, key.as_ref());
        self.view.send_event(HostViewEvent::KeyUp, key_up.as_ref());
        self.view
            .send_event(HostViewEvent::FlagsChanged, key.as_ref());
        self.verify_mouse_events(&mouse);
        assert_eq!(self.events.borrow().len(), 21);
        (key, mouse)
    }

    fn verify_mouse_events(&self, mouse: &Event) {
        let mouse_selectors = [
            (HostViewEvent::LeftMouseDown, EventType::LeftMouseDown),
            (HostViewEvent::LeftMouseUp, EventType::LeftMouseUp),
            (HostViewEvent::RightMouseDown, EventType::RightMouseDown),
            (HostViewEvent::RightMouseUp, EventType::RightMouseUp),
            (HostViewEvent::OtherMouseDown, EventType::OtherMouseDown),
            (HostViewEvent::OtherMouseUp, EventType::OtherMouseUp),
            (HostViewEvent::LeftMouseDragged, EventType::LeftMouseDragged),
            (
                HostViewEvent::RightMouseDragged,
                EventType::RightMouseDragged,
            ),
            (
                HostViewEvent::OtherMouseDragged,
                EventType::OtherMouseDragged,
            ),
            (HostViewEvent::MouseMoved, EventType::MouseMoved),
        ];
        for (selector, event_type) in mouse_selectors {
            let event = Event::synthetic_mouse(
                self.main_thread,
                event_type,
                CGPoint::new(8.0, 13.0),
                ModifierFlags::default(),
                1,
                0.5,
            )
            .expect("a synthetic mouse event");
            self.view.send_event(selector, event.as_ref());
            assert_eq!(
                self.events.borrow().last().copied(),
                Some((selector, event_type))
            );
        }
        for selector in [
            HostViewEvent::MouseEntered,
            HostViewEvent::MouseExited,
            HostViewEvent::ScrollWheel,
            HostViewEvent::PressureChanged,
            HostViewEvent::Magnified,
            HostViewEvent::Swiped,
        ] {
            self.view.send_event(selector, mouse.as_ref());
            assert_eq!(
                self.events.borrow().last().copied(),
                Some((selector, EventType::MouseMoved))
            );
        }
    }

    fn verify_queries(&self, mouse: &Event) {
        assert!(self.view.accepts_first_mouse(mouse.as_ref()));
        assert!(self.view.accepts_first_responder());
        self.first_mouse.set(false);
        self.first_responder.set(false);
        assert!(!self.view.accepts_first_mouse(mouse.as_ref()));
        assert!(!self.view.accepts_first_responder());
        assert_eq!(
            &self.queries.borrow()[..],
            &[
                (HostViewQuery::AcceptsFirstMouse, true),
                (HostViewQuery::AcceptsFirstResponder, false),
                (HostViewQuery::AcceptsFirstMouse, true),
                (HostViewQuery::AcceptsFirstResponder, false),
            ]
        );
        self.view.set_query_handler(None);
        assert!(self.view.accepts_first_mouse(mouse.as_ref()));
        assert!(self.view.accepts_first_responder());
        assert!(unsafe {
            appkit_host_view_accepts_first_mouse(self.view.as_ptr(), core::ptr::null_mut())
        });
        let native_view = self.view.as_ptr();
        self.view.set_query_handler(Some(Box::new(move |query, _| {
            if query == HostViewQuery::AcceptsFirstResponder {
                assert!(!unsafe { appkit_host_view_accepts_first_responder(native_view) });
            }
            true
        })));
        assert!(self.view.accepts_first_responder());
        self.view.set_query_handler(Some(Box::new({
            let queries = self.queries.clone();
            move |query, event| {
                queries.borrow_mut().push((query, event.is_some()));
                false
            }
        })));
    }

    fn verify_notifications(&self) -> NonNull<c_void> {
        self.notifications.borrow_mut().clear();
        self.view.notify_backing_properties_changed();
        assert!(self.view.set_frame_size(CGSize::new(100.0, 70.0)));
        assert!(!self.view.set_frame_size(CGSize::new(f64::NAN, 70.0)));
        assert!(!self.view.set_frame_size(CGSize::new(100.0, f64::INFINITY)));
        assert!(!self.view.set_frame_size(CGSize::new(-1.0, 70.0)));
        assert_eq!(self.view.as_view().frame().size, CGSize::new(100.0, 70.0));
        self.view.notify_effective_appearance_changed();
        assert!(self.view.display_layer());
        assert!(self.view.invalidate_cursor_rects());
        let received = self.notifications.borrow();
        for notification in [
            HostViewNotification::BackingPropertiesChanged,
            HostViewNotification::FrameSizeChanged(CGSize::new(100.0, 70.0)),
            HostViewNotification::EffectiveAppearanceChanged,
            HostViewNotification::DisplayLayer,
            HostViewNotification::ResetCursorRects,
        ] {
            assert!(received.contains(&notification), "{notification:?}");
        }
        drop(received);
        let cursor = NonNull::new(unsafe { appkit_host_view_example_arrow_cursor() })
            .expect("an arrow cursor");
        assert!(unsafe {
            self.view
                .add_cursor_rect_borrowed(self.view.as_view().bounds(), cursor)
        });
        assert!(!unsafe {
            self.view
                .add_cursor_rect_borrowed(CGRect::new(f64::NAN, 0.0, 10.0, 10.0), cursor)
        });
        assert!(self
            .view
            .set_tracking_options(TrackingAreaOptions::default()));
        assert_eq!(self.view.tracking_options(), TrackingAreaOptions::default());
        cursor
    }

    fn verify_reentrant_teardown(self, key: Event, mouse: &Event, cursor: NonNull<c_void>) {
        let native_view = NonNull::new(self.view.as_ptr()).expect("a native view");
        unsafe { appkit_retain(native_view.as_ptr()) };
        let view_slot = Rc::new(RefCell::new(Some(self.view)));
        let weak_slot = Rc::downgrade(&view_slot);
        let queued_events = Rc::new(RefCell::new(Vec::new()));
        let queued_mouse = mouse.clone();
        view_slot
            .borrow()
            .as_ref()
            .expect("an active view")
            .set_event_handler(Some(Box::new({
                let queued_events = queued_events.clone();
                move |selector, _| {
                    queued_events.borrow_mut().push(selector);
                    if selector == HostViewEvent::KeyDown {
                        assert!(!unsafe {
                            appkit_host_view_send_event(
                                native_view.as_ptr(),
                                HostViewEvent::PerformKeyEquivalent as u32,
                                key.as_ptr(),
                            )
                        });
                        unsafe {
                            appkit_host_view_send_event(
                                native_view.as_ptr(),
                                HostViewEvent::MouseMoved as u32,
                                queued_mouse.as_ptr(),
                            );
                        }
                        let dropped = weak_slot
                            .upgrade()
                            .expect("a view slot")
                            .borrow_mut()
                            .take();
                        drop(dropped);
                    }
                    true
                }
            })));
        let trigger = Event::synthetic_key(
            self.main_thread,
            EventType::KeyDown,
            ModifierFlags::default(),
            12,
            "x",
            "x",
            false,
        )
        .expect("a reentrant key event");
        unsafe {
            appkit_host_view_send_event(
                native_view.as_ptr(),
                HostViewEvent::KeyDown as u32,
                trigger.as_ptr(),
            );
        }
        assert!(view_slot.borrow().is_none());
        assert_eq!(
            &queued_events.borrow()[..],
            &[HostViewEvent::KeyDown, HostViewEvent::MouseMoved]
        );
        let event_count = self.events.borrow().len();
        let notification_count = self.notifications.borrow().len();
        let query_count = self.queries.borrow().len();
        unsafe {
            appkit_host_view_send_event(
                native_view.as_ptr(),
                HostViewEvent::KeyDown as u32,
                trigger.as_ptr(),
            );
            appkit_host_view_notify_appearance(native_view.as_ptr());
            appkit_host_view_display_layer(native_view.as_ptr());
            appkit_host_view_accepts_first_mouse(native_view.as_ptr(), mouse.as_ptr());
            appkit_host_view_accepts_first_responder(native_view.as_ptr());
        }
        assert_eq!(self.events.borrow().len(), event_count);
        assert_eq!(self.notifications.borrow().len(), notification_count);
        assert_eq!(self.queries.borrow().len(), query_count);
        self.window.close();
        unsafe {
            appkit_release(cursor.as_ptr());
            appkit_release(native_view.as_ptr());
        }
    }
}

fn main() {
    let fixture = Fixture::new();
    fixture.verify_tracking();
    let (key, mouse) = fixture.verify_events();
    fixture.verify_queries(&mouse);
    let cursor = fixture.verify_notifications();
    fixture.verify_reentrant_teardown(key, &mouse, cursor);
    println!("The hidden hosted view delivered every callback and closed safely.");
}
