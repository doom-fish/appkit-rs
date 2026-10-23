use core::ffi::c_void;
use core::ptr::NonNull;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use appkit::event::{EventType, ModifierFlags};
use appkit::host_view::{
    HostViewEvent, HostViewNotification, HostViewQuery, HostedView, MetalLayer, TrackingAreaOptions,
};
use appkit::host_window::{HostedWindow, HostedWindowConfiguration, WindowStyleMask};
use appkit::{CGPoint, CGRect, CGSize, Event, MainThreadMarker, Screen, Window};

extern "C" {
    fn appkit_retain(object: *mut c_void);
    fn appkit_release(object: *mut c_void);
    fn appkit_host_view_example_arrow_cursor() -> *mut c_void;
    fn appkit_host_view_example_autoreleasepool(
        callback: extern "C" fn(*mut c_void),
        context: *mut c_void,
    );
    fn appkit_host_view_example_weak_reference(view: *mut c_void) -> *mut c_void;
    fn appkit_host_view_example_weak_reference_is_alive(reference: *mut c_void) -> bool;
    fn appkit_host_view_example_move_offscreen(view: *mut c_void, x: f64, y: f64) -> bool;
    fn appkit_host_view_example_shown_offscreen(view: *mut c_void) -> bool;
    fn appkit_host_view_example_application_is_active() -> bool;
    fn appkit_host_view_example_redraw(view: *mut c_void, resize: bool) -> bool;
    fn appkit_host_view_example_titlebar_query(view: *mut c_void, out: *mut CGRect) -> u64;
    fn appkit_host_view_example_offscreen_titled_window_new(
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> *mut c_void;
    fn appkit_host_view_example_titlebar_drag_rect(view: *mut c_void, out: *mut CGRect) -> bool;
    fn appkit_host_view_send_event(view: *mut c_void, selector: u32, event: *mut c_void) -> bool;
    fn appkit_host_view_set_tracking_options(view: *mut c_void, options: u64) -> bool;
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
        let missing_visible_rect = TrackingAreaOptions::MOUSE_ENTERED_AND_EXITED
            | TrackingAreaOptions::MOUSE_MOVED
            | TrackingAreaOptions::ACTIVE_ALWAYS;
        assert!(!self.view.set_tracking_options(missing_visible_rect));
        assert!(!unsafe {
            appkit_host_view_set_tracking_options(self.view.as_ptr(), missing_visible_rect.bits())
        });
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

    fn verify_titlebar_drag(&self) {
        let empty = CGRect::new(0.0, 0.0, 0.0, 0.0);
        let mut rect = CGRect::new(-1.0, -1.0, -1.0, -1.0);
        assert!(unsafe {
            appkit_host_view_example_titlebar_drag_rect(self.view.as_ptr(), &raw mut rect)
        });
        assert_eq!(rect, empty);

        assert!(self.view.set_owns_titlebar_drag(true));
        assert!(unsafe {
            appkit_host_view_example_titlebar_drag_rect(self.view.as_ptr(), &raw mut rect)
        });
        assert_eq!(rect, self.view.as_view().bounds());
        assert_eq!(rect.size, CGSize::new(100.0, 70.0));

        assert!(self.view.set_owns_titlebar_drag(false));
        assert!(unsafe {
            appkit_host_view_example_titlebar_drag_rect(self.view.as_ptr(), &raw mut rect)
        });
        assert_eq!(rect, empty);
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

struct SharedLayerCase {
    main_thread: MainThreadMarker,
    layer: MetalLayer,
    first_window: Window,
    second_window: HostedWindow,
    replacement: Option<HostedView>,
    weak_original: Option<NonNull<c_void>>,
    displays: Rc<Cell<usize>>,
    detached_display_delivered: bool,
    error: Option<&'static str>,
}

fn build_shared_layer_case(case: &mut SharedLayerCase) -> Result<(), &'static str> {
    let frame = CGRect::new(0.0, 0.0, 80.0, 60.0);
    let original = unsafe { HostedView::new(case.main_thread, frame, case.layer.as_ptr()) }
        .ok_or("the original view is missing")?;
    let first_parent = case
        .first_window
        .content_view()
        .ok_or("the first content view is missing")?;
    if !original.attach_to(&first_parent) {
        return Err("the original view did not attach");
    }
    if unsafe { HostedView::new(case.main_thread, frame, case.layer.as_ptr()) }.is_some() {
        return Err("the shared layer accepted a second live view");
    }
    let original_ptr = original.as_ptr();
    let weak = NonNull::new(unsafe { appkit_host_view_example_weak_reference(original_ptr) })
        .ok_or("the weak view reference is missing")?;
    case.weak_original = Some(weak);
    unsafe { appkit_retain(original_ptr) };
    drop(original);
    case.first_window.close();

    let replacement = unsafe { HostedView::new(case.main_thread, frame, case.layer.as_ptr()) }
        .ok_or("the replacement view is missing")?;
    let second_parent = case
        .second_window
        .as_window()
        .content_view()
        .ok_or("the second content view is missing")?;
    if !replacement.attach_to(&second_parent)
        || replacement.layer_ptr() != Some(case.layer.as_ptr())
    {
        return Err("the replacement view did not get the shared layer");
    }
    replacement.set_notification_handler(Some(Box::new({
        let displays = case.displays.clone();
        move |notification| {
            if notification == HostViewNotification::DisplayLayer {
                displays.set(displays.get() + 1);
            }
        }
    })));

    case.replacement = Some(replacement);
    let before_display = case.displays.get();
    let replacement_displayed = case
        .replacement
        .as_ref()
        .is_some_and(HostedView::display_layer)
        && case.displays.get() > before_display;
    let original_alive = unsafe { appkit_host_view_example_weak_reference_is_alive(weak.as_ptr()) };
    case.detached_display_delivered = replacement_displayed && original_alive;
    unsafe { appkit_release(original_ptr) };
    Ok(())
}

extern "C" fn build_shared_layer_case_in_pool(context: *mut c_void) {
    if let Some(case) = unsafe { context.cast::<SharedLayerCase>().as_mut() } {
        case.error = build_shared_layer_case(case).err();
    }
}

fn offscreen_rect(main_thread: MainThreadMarker) -> CGRect {
    let screens = Screen::screens(main_thread);
    let x = screens
        .iter()
        .map(|screen| {
            let frame = screen.frame();
            frame.origin.x + frame.size.width
        })
        .reduce(f64::max)
        .expect("a connected screen")
        + 128.0;
    let y = screens
        .iter()
        .map(|screen| {
            let frame = screen.frame();
            frame.origin.y + frame.size.height
        })
        .reduce(f64::max)
        .expect("a connected screen")
        + 128.0;
    CGRect::new(x, y, 180.0, 120.0)
}

fn offscreen_window(main_thread: MainThreadMarker, style: WindowStyleMask) -> HostedWindow {
    let mut configuration = HostedWindowConfiguration::new(offscreen_rect(main_thread), style);
    configuration.can_become_main = false;
    configuration.can_become_key = false;
    let window = HostedWindow::new(main_thread, configuration).expect("a hidden off-screen window");
    assert!(!window.can_become_key());
    assert!(!window.can_become_main());
    window.as_window().set_alpha_value(0.0);
    window
}

fn show_offscreen(view: &HostedView, window: &Window, main_thread: MainThreadMarker) {
    assert!(!unsafe { appkit_host_view_example_application_is_active() });
    window.order_front();
    if !unsafe { appkit_host_view_example_shown_offscreen(view.as_ptr()) } {
        let origin = offscreen_rect(main_thread).origin;
        let moved =
            unsafe { appkit_host_view_example_move_offscreen(view.as_ptr(), origin.x, origin.y) };
        assert!(
            moved,
            "the window frame {:?} did not reach {origin:?}",
            window.frame()
        );
    }
    window.set_alpha_value(1.0);
    assert_eq!(window.alpha_value(), 1.0);
    assert!(unsafe { appkit_host_view_example_shown_offscreen(view.as_ptr()) });
    assert!(!unsafe { appkit_host_view_example_application_is_active() });
}

fn verify_shared_layer_delegate() {
    let main_thread = MainThreadMarker::new().expect("the main thread");
    let mut case = SharedLayerCase {
        main_thread,
        layer: MetalLayer::new(main_thread).expect("a shared metal layer"),
        first_window: Window::new(
            main_thread,
            CGRect::new(0.0, 0.0, 80.0, 60.0),
            "Hidden original view",
        )
        .expect("a hidden window"),
        second_window: offscreen_window(main_thread, WindowStyleMask::BORDERLESS),
        replacement: None,
        weak_original: None,
        displays: Rc::new(Cell::new(0)),
        detached_display_delivered: false,
        error: Some("the pool callback did not run"),
    };
    unsafe {
        appkit_host_view_example_autoreleasepool(
            build_shared_layer_case_in_pool,
            (&raw mut case).cast(),
        );
    }
    assert!(case.error.is_none(), "{:?}", case.error);
    assert!(case.detached_display_delivered);
    let weak_original = case.weak_original.expect("a weak view reference");
    assert!(!unsafe { appkit_host_view_example_weak_reference_is_alive(weak_original.as_ptr()) });
    let after_pool = case.displays.get();
    assert!(case
        .replacement
        .as_ref()
        .is_some_and(HostedView::display_layer));
    assert!(case.displays.get() > after_pool);
    let replacement = case.replacement.as_ref().expect("a replacement view");
    show_offscreen(replacement, &case.second_window.as_window(), main_thread);
    assert!(unsafe {
        HostedView::new(
            main_thread,
            replacement.as_view().bounds(),
            case.layer.as_ptr(),
        )
    }
    .is_none());
    let before_needs_display = case.displays.get();
    assert!(unsafe { appkit_host_view_example_redraw(replacement.as_ptr(), false) });
    assert!(case.displays.get() > before_needs_display);
    let before_resize = case.displays.get();
    assert!(unsafe { appkit_host_view_example_redraw(replacement.as_ptr(), true) });
    assert!(case.displays.get() > before_resize);
    drop(case.replacement.take());
    case.second_window.as_window().close();
    unsafe { appkit_release(weak_original.as_ptr()) };
}

fn verify_offscreen_titlebar_drag() {
    let main_thread = MainThreadMarker::new().expect("the main thread");
    let frame = offscreen_rect(main_thread);
    let raw = NonNull::new(unsafe {
        appkit_host_view_example_offscreen_titled_window_new(
            frame.origin.x,
            frame.origin.y,
            frame.size.width,
            frame.size.height,
        )
    })
    .expect("a titled off-screen window");
    let native = unsafe { Window::from_raw_borrowed(raw, main_thread) };
    let parent = native.content_view().expect("a titlebar content view");
    let layer = MetalLayer::new(main_thread).expect("a titlebar metal layer");
    let view = unsafe { HostedView::new(main_thread, parent.bounds(), layer.as_ptr()) }
        .expect("a titlebar view");
    assert!(view.attach_to(&parent));
    assert!(view.set_owns_titlebar_drag(true));
    show_offscreen(&view, &native, main_thread);

    let mut cached_rect = CGRect::new(-1.0, -1.0, -1.0, -1.0);
    let before =
        unsafe { appkit_host_view_example_titlebar_query(view.as_ptr(), &raw mut cached_rect) };
    assert!(before > 0);
    assert_eq!(cached_rect, view.as_view().bounds());

    assert!(view.set_owns_titlebar_drag(false));
    let after_releasing =
        unsafe { appkit_host_view_example_titlebar_query(view.as_ptr(), &raw mut cached_rect) };
    assert!(after_releasing > before);
    assert_eq!(cached_rect, CGRect::new(0.0, 0.0, 0.0, 0.0));

    assert!(view.set_owns_titlebar_drag(true));
    let after_owning =
        unsafe { appkit_host_view_example_titlebar_query(view.as_ptr(), &raw mut cached_rect) };
    assert!(after_owning > after_releasing);
    assert_eq!(cached_rect, view.as_view().bounds());

    drop(view);
    native.close();
    drop(native);
    unsafe { appkit_release(raw.as_ptr()) };
}

fn main() {
    let fixture = Fixture::new();
    fixture.verify_tracking();
    let (key, mouse) = fixture.verify_events();
    fixture.verify_queries(&mouse);
    let cursor = fixture.verify_notifications();
    fixture.verify_titlebar_drag();
    fixture.verify_reentrant_teardown(key, &mouse, cursor);
    verify_shared_layer_delegate();
    verify_offscreen_titlebar_drag();
    println!("Hosted views delivered callbacks, updated titlebar regions, and closed safely.");
}
