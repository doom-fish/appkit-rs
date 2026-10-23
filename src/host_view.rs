use core::cell::{Cell, RefCell};
use core::ffi::c_void;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};
use core::ptr::NonNull;
use std::collections::VecDeque;
use std::rc::Rc;

use apple_cf::cg::{CGRect, CGSize};
use doom_fish_utils::panic_safe::catch_user_panic_result;

use crate::callbacks::{
    dispatch_event, CallbackRegistration, CallbackTableHeader, CallbackTableV1, HandlerSlot,
    RegistrationToken,
};
use crate::event::{Event, EventRef};
use crate::ffi::host_view::{self as ffi, HostViewCallbackTableV1};
use crate::main_thread::MainThreadMarker;
use crate::object::Handle;
use crate::view::View;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostViewEvent {
    PerformKeyEquivalent = 1,
    KeyDown = 2,
    KeyUp = 3,
    FlagsChanged = 4,
    LeftMouseDown = 5,
    LeftMouseUp = 6,
    RightMouseDown = 7,
    RightMouseUp = 8,
    OtherMouseDown = 9,
    OtherMouseUp = 10,
    LeftMouseDragged = 11,
    RightMouseDragged = 12,
    OtherMouseDragged = 13,
    MouseMoved = 14,
    MouseEntered = 15,
    MouseExited = 16,
    ScrollWheel = 17,
    PressureChanged = 18,
    Magnified = 19,
    Swiped = 20,
}

impl HostViewEvent {
    const fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            1 => Some(Self::PerformKeyEquivalent),
            2 => Some(Self::KeyDown),
            3 => Some(Self::KeyUp),
            4 => Some(Self::FlagsChanged),
            5 => Some(Self::LeftMouseDown),
            6 => Some(Self::LeftMouseUp),
            7 => Some(Self::RightMouseDown),
            8 => Some(Self::RightMouseUp),
            9 => Some(Self::OtherMouseDown),
            10 => Some(Self::OtherMouseUp),
            11 => Some(Self::LeftMouseDragged),
            12 => Some(Self::RightMouseDragged),
            13 => Some(Self::OtherMouseDragged),
            14 => Some(Self::MouseMoved),
            15 => Some(Self::MouseEntered),
            16 => Some(Self::MouseExited),
            17 => Some(Self::ScrollWheel),
            18 => Some(Self::PressureChanged),
            19 => Some(Self::Magnified),
            20 => Some(Self::Swiped),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum HostViewNotification {
    BackingPropertiesChanged,
    FrameSizeChanged(CGSize),
    EffectiveAppearanceChanged,
    DisplayLayer,
    ResetCursorRects,
}

impl HostViewNotification {
    const fn from_raw(raw: u32, width: f64, height: f64) -> Option<Self> {
        match raw {
            1 => Some(Self::BackingPropertiesChanged),
            2 => Some(Self::FrameSizeChanged(CGSize::new(width, height))),
            3 => Some(Self::EffectiveAppearanceChanged),
            4 => Some(Self::DisplayLayer),
            5 => Some(Self::ResetCursorRects),
            _ => None,
        }
    }
}

impl PartialEq for HostViewNotification {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::FrameSizeChanged(left), Self::FrameSizeChanged(right)) => left == right,
            (Self::BackingPropertiesChanged, Self::BackingPropertiesChanged)
            | (Self::EffectiveAppearanceChanged, Self::EffectiveAppearanceChanged)
            | (Self::DisplayLayer, Self::DisplayLayer)
            | (Self::ResetCursorRects, Self::ResetCursorRects) => true,
            _ => false,
        }
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostViewQuery {
    AcceptsFirstMouse = 1,
    AcceptsFirstResponder = 2,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TrackingAreaOptions(u64);

impl TrackingAreaOptions {
    pub const MOUSE_ENTERED_AND_EXITED: Self = Self(1);
    pub const MOUSE_MOVED: Self = Self(1 << 1);
    pub const CURSOR_UPDATE: Self = Self(1 << 2);
    pub const ACTIVE_WHEN_FIRST_RESPONDER: Self = Self(1 << 4);
    pub const ACTIVE_IN_KEY_WINDOW: Self = Self(1 << 5);
    pub const ACTIVE_IN_ACTIVE_APP: Self = Self(1 << 6);
    pub const ACTIVE_ALWAYS: Self = Self(1 << 7);
    pub const ASSUME_INSIDE: Self = Self(1 << 8);
    pub const IN_VISIBLE_RECT: Self = Self(1 << 9);
    pub const ENABLED_DURING_MOUSE_DRAG: Self = Self(1 << 10);

    #[must_use]
    pub const fn from_bits_retain(bits: u64) -> Self {
        Self(bits)
    }

    #[must_use]
    pub const fn bits(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl BitOr for TrackingAreaOptions {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for TrackingAreaOptions {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for TrackingAreaOptions {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for TrackingAreaOptions {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

pub type HostViewEventHandler = Box<dyn for<'event> FnMut(HostViewEvent, EventRef<'event>) -> bool>;
pub type HostViewNotificationHandler = Box<dyn FnMut(HostViewNotification)>;
pub type HostViewQueryHandler =
    Box<dyn for<'event> FnMut(HostViewQuery, Option<EventRef<'event>>) -> bool>;

#[derive(Default)]
struct HostViewState {
    events: HandlerSlot<HostViewEventHandler>,
    notifications: HandlerSlot<HostViewNotificationHandler>,
    queries: HandlerSlot<HostViewQueryHandler>,
    event_active: Cell<bool>,
    notification_active: Cell<bool>,
    query_handler_present: Cell<bool>,
    pending_events: RefCell<VecDeque<(HostViewEvent, Event)>>,
    pending_notifications: RefCell<VecDeque<HostViewNotification>>,
}

struct DispatchGuard<'a>(&'a Cell<bool>);

impl Drop for DispatchGuard<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

impl HostViewState {
    fn event(&self, selector: HostViewEvent, event: EventRef<'_>) -> bool {
        if selector == HostViewEvent::PerformKeyEquivalent && self.event_active.get() {
            return false;
        }
        if self.event_active.replace(true) {
            self.pending_events
                .borrow_mut()
                .push_back((selector, event.retain()));
            return false;
        }
        let _guard = DispatchGuard(&self.event_active);
        let handled = self
            .events
            .invoke("AppKit hosted view event", |handler| {
                handler(selector, event)
            })
            .unwrap_or(false);
        loop {
            let next = self.pending_events.borrow_mut().pop_front();
            let Some((selector, event)) = next else {
                break;
            };
            let _ = self
                .events
                .invoke("AppKit queued hosted view event", |handler| {
                    handler(selector, event.as_ref())
                });
        }
        handled
    }

    fn notify(&self, notification: HostViewNotification) {
        if self.notification_active.replace(true) {
            self.pending_notifications
                .borrow_mut()
                .push_back(notification);
            return;
        }
        let _guard = DispatchGuard(&self.notification_active);
        let _ = self
            .notifications
            .invoke("AppKit hosted view notification", |handler| {
                handler(notification);
            });
        loop {
            let next = self.pending_notifications.borrow_mut().pop_front();
            let Some(notification) = next else {
                break;
            };
            let _ =
                self.notifications
                    .invoke("AppKit queued hosted view notification", |handler| {
                        handler(notification);
                    });
        }
    }

    fn query(&self, query: HostViewQuery, event: Option<EventRef<'_>>) -> bool {
        self.queries
            .invoke("AppKit hosted view query", |handler| handler(query, event))
            .unwrap_or_else(|| !self.query_handler_present.get())
    }
}

fn with_state<R>(raw_token: u64, site: &str, call: impl FnOnce(&HostViewState) -> R) -> Option<R> {
    catch_user_panic_result(site, || {
        let main_thread = MainThreadMarker::new()?;
        let token = RegistrationToken::from_raw(raw_token)?;
        let state = CallbackRegistration::<HostViewState>::snapshot(token, main_thread)?;
        Some(call(&state))
    })
    .flatten()
}

extern "C" fn on_event(raw_token: u64, raw_selector: u32, event: *mut c_void) -> bool {
    let Some(selector) = HostViewEvent::from_raw(raw_selector) else {
        return false;
    };
    unsafe {
        dispatch_event::<HostViewState>(
            raw_token,
            event,
            "AppKit hosted view event callback",
            |state, event| state.event(selector, event),
        )
    }
}

extern "C" fn on_notification(raw_token: u64, raw_selector: u32, width: f64, height: f64) {
    let Some(notification) = HostViewNotification::from_raw(raw_selector, width, height) else {
        return;
    };
    let _ = with_state(
        raw_token,
        "AppKit hosted view notification callback",
        |state| {
            state.notify(notification);
        },
    );
}

extern "C" fn on_query(raw_token: u64, raw_selector: u32, event: *mut c_void) -> bool {
    match raw_selector {
        1 if event.is_null() => with_state(
            raw_token,
            "AppKit hosted view first mouse callback",
            |state| state.query(HostViewQuery::AcceptsFirstMouse, None),
        )
        .unwrap_or(false),
        1 => unsafe {
            dispatch_event::<HostViewState>(
                raw_token,
                event,
                "AppKit hosted view first mouse callback",
                |state, event| state.query(HostViewQuery::AcceptsFirstMouse, Some(event)),
            )
        },
        2 => with_state(
            raw_token,
            "AppKit hosted view first responder callback",
            |state| state.query(HostViewQuery::AcceptsFirstResponder, None),
        )
        .unwrap_or(false),
        _ => false,
    }
}

pub struct MetalLayer {
    handle: Handle,
    _main_thread: MainThreadMarker,
}

impl MetalLayer {
    #[must_use]
    pub fn new(main_thread: MainThreadMarker) -> Option<Self> {
        let handle = unsafe { Handle::from_retained(ffi::appkit_host_view_metal_layer_new()) }?;
        Some(Self {
            handle,
            _main_thread: main_thread,
        })
    }

    #[must_use]
    pub fn as_ptr(&self) -> NonNull<c_void> {
        NonNull::new(self.handle.as_ptr()).expect("a metal layer handle is not null")
    }
}

pub struct HostedView {
    registration: CallbackRegistration<HostViewState>,
    handle: Handle,
}

impl HostedView {
    #[must_use]
    pub unsafe fn new(
        main_thread: MainThreadMarker,
        frame: CGRect,
        borrowed_metal_layer: NonNull<c_void>,
    ) -> Option<Self> {
        let registration =
            CallbackRegistration::new(main_thread, Rc::new(HostViewState::default()));
        let table = HostViewCallbackTableV1 {
            header: CallbackTableHeader {
                version: 1,
                size: size_of::<HostViewCallbackTableV1>(),
            },
            deactivate: CallbackTableV1::new(None, None).deactivate,
            event: Some(on_event),
            notification: Some(on_notification),
            query: Some(on_query),
        };
        let raw = unsafe {
            ffi::appkit_host_view_new(
                registration.token().get(),
                &raw const table,
                borrowed_metal_layer.as_ptr(),
                &raw const frame,
            )
        };
        let handle = unsafe { Handle::from_retained(raw) }?;
        Some(Self {
            registration,
            handle,
        })
    }

    #[must_use]
    pub const fn as_ptr(&self) -> *mut c_void {
        self.handle.as_ptr()
    }

    #[must_use]
    pub fn as_view(&self) -> View {
        View::from_handle(self.handle.clone())
    }

    #[must_use]
    pub fn layer_ptr(&self) -> Option<NonNull<c_void>> {
        NonNull::new(unsafe { crate::ffi::appkit_view_layer(self.as_ptr()) })
    }

    #[must_use]
    pub fn attach_to(&self, parent: &View) -> bool {
        unsafe { ffi::appkit_host_view_attach(self.as_ptr(), parent.as_ptr()) }
    }

    #[must_use]
    pub fn set_frame_size(&self, size: CGSize) -> bool {
        unsafe { ffi::appkit_host_view_set_frame_size(self.as_ptr(), size.width, size.height) }
    }

    #[must_use]
    pub fn set_tracking_options(&self, options: TrackingAreaOptions) -> bool {
        if options.bits() != 0 && !options.contains(TrackingAreaOptions::IN_VISIBLE_RECT) {
            return false;
        }
        unsafe { ffi::appkit_host_view_set_tracking_options(self.as_ptr(), options.bits()) }
    }

    #[must_use]
    pub fn set_owns_titlebar_drag(&self, owns: bool) -> bool {
        unsafe { ffi::appkit_host_view_set_owns_titlebar_drag(self.as_ptr(), owns) }
    }

    #[must_use]
    pub fn tracking_options(&self) -> TrackingAreaOptions {
        TrackingAreaOptions::from_bits_retain(unsafe {
            ffi::appkit_host_view_tracking_options(self.as_ptr())
        })
    }

    #[must_use]
    pub fn invalidate_cursor_rects(&self) -> bool {
        unsafe { ffi::appkit_host_view_invalidate_cursor_rects(self.as_ptr()) }
    }

    #[must_use]
    pub unsafe fn add_cursor_rect_borrowed(&self, rect: CGRect, cursor: NonNull<c_void>) -> bool {
        unsafe {
            ffi::appkit_host_view_add_cursor_rect(self.as_ptr(), &raw const rect, cursor.as_ptr())
        }
    }

    pub fn send_event(&self, selector: HostViewEvent, event: EventRef<'_>) {
        let _ = unsafe {
            ffi::appkit_host_view_send_event(self.as_ptr(), selector as u32, event.as_ptr())
        };
    }

    #[must_use]
    pub fn perform_key_equivalent(&self, event: EventRef<'_>) -> bool {
        unsafe {
            ffi::appkit_host_view_send_event(
                self.as_ptr(),
                HostViewEvent::PerformKeyEquivalent as u32,
                event.as_ptr(),
            )
        }
    }

    pub fn notify_backing_properties_changed(&self) {
        unsafe { ffi::appkit_host_view_notify_backing_properties(self.as_ptr()) };
    }

    pub fn notify_effective_appearance_changed(&self) {
        unsafe { ffi::appkit_host_view_notify_appearance(self.as_ptr()) };
    }

    #[must_use]
    pub fn display_layer(&self) -> bool {
        unsafe { ffi::appkit_host_view_display_layer(self.as_ptr()) }
    }

    #[must_use]
    pub fn accepts_first_mouse(&self, event: EventRef<'_>) -> bool {
        unsafe { ffi::appkit_host_view_accepts_first_mouse(self.as_ptr(), event.as_ptr()) }
    }

    #[must_use]
    pub fn accepts_first_responder(&self) -> bool {
        unsafe { ffi::appkit_host_view_accepts_first_responder(self.as_ptr()) }
    }

    pub fn set_event_handler(&self, handler: Option<HostViewEventHandler>) {
        let _ = self.registration.state().events.replace(handler);
    }

    pub fn set_notification_handler(&self, handler: Option<HostViewNotificationHandler>) {
        let _ = self.registration.state().notifications.replace(handler);
    }

    pub fn set_query_handler(&self, handler: Option<HostViewQueryHandler>) {
        let state = self.registration.state();
        state.query_handler_present.set(handler.is_some());
        let _ = state.queries.replace(handler);
    }
}

impl HostedView {
    pub fn deactivate_callbacks(&self) {
        let _ = self.registration.deactivate();
    }
}

impl Drop for HostedView {
    fn drop(&mut self) {
        let _ = self.registration.deactivate();
        unsafe { ffi::appkit_host_view_detach(self.as_ptr()) };
    }
}
