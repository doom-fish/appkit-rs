use core::cell::{Cell, RefCell};
use core::ffi::c_void;
use std::collections::VecDeque;
use std::rc::Rc;

use apple_cf::cg::{CGPoint, CGRect};
use doom_fish_utils::panic_safe::catch_user_panic_result;

use crate::callbacks::{
    copy_utf8, CallbackRegistration, CallbackTableHeader, CallbackTableV1, HandlerSlot,
    RegistrationToken, Utf16Range,
};
use crate::event::EventRef;
use crate::ffi::text_input::{
    self as ffi, CopyUtf8, TextInputCallbackTableV1, TextInputQueryResult,
};
use crate::host_view::HostedView;
use crate::main_thread::MainThreadMarker;
use crate::view::View;

pub struct TextInputRect {
    pub bounds: CGRect,
    pub actual_range: Utf16Range,
}

pub struct TextInputSubstring {
    pub text: String,
    pub actual_range: Utf16Range,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextInputEventStatus {
    NotConsumed = 0,
    Consumed = 1,
    HandledCommand = 2,
    UnhandledCommand = 3,
}

impl TextInputEventStatus {
    const fn from_raw(raw: u32) -> Self {
        match raw {
            1 => Self::Consumed,
            2 => Self::HandledCommand,
            3 => Self::UnhandledCommand,
            _ => Self::NotConsumed,
        }
    }

    #[must_use]
    pub const fn is_handled(self) -> bool {
        matches!(self, Self::Consumed | Self::HandledCommand)
    }
}

pub trait TextInputHandler {
    fn has_marked_text(&mut self) -> bool {
        self.marked_range().is_some()
    }

    fn marked_range(&mut self) -> Option<Utf16Range> {
        None
    }

    fn selected_range(&mut self) -> Option<Utf16Range> {
        None
    }

    fn first_rect_for_character_range(&mut self, _range: Utf16Range) -> Option<TextInputRect> {
        None
    }

    fn insert_text(&mut self, _text: &str, _replacement_range: Utf16Range) {}

    fn set_marked_text(
        &mut self,
        _text: &str,
        _selected_range: Utf16Range,
        _replacement_range: Utf16Range,
    ) {
    }

    fn unmark_text(&mut self) {}

    fn attributed_substring_for_proposed_range(
        &mut self,
        _range: Utf16Range,
    ) -> Option<TextInputSubstring> {
        None
    }

    fn character_index_for_point(&mut self, _point: CGPoint) -> Option<usize> {
        None
    }

    fn do_command_by_selector(&mut self, _selector: &str) -> bool {
        false
    }
}

#[repr(u32)]
#[derive(Clone, Copy)]
enum Query {
    HasMarkedText = 1,
    MarkedRange = 2,
    SelectedRange = 3,
    FirstRect = 4,
    AttributedSubstring = 5,
    CharacterIndex = 6,
}

impl Query {
    const fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            1 => Some(Self::HasMarkedText),
            2 => Some(Self::MarkedRange),
            3 => Some(Self::SelectedRange),
            4 => Some(Self::FirstRect),
            5 => Some(Self::AttributedSubstring),
            6 => Some(Self::CharacterIndex),
            _ => None,
        }
    }
}

#[repr(u32)]
enum ActionKind {
    Insert = 1,
    Mark = 2,
    Unmark = 3,
}

impl ActionKind {
    const fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            1 => Some(Self::Insert),
            2 => Some(Self::Mark),
            3 => Some(Self::Unmark),
            _ => None,
        }
    }
}

enum Action {
    Insert(String, Utf16Range),
    Mark(String, Utf16Range, Utf16Range),
    Unmark,
}

enum Response {
    Marked(bool),
    Range(Utf16Range),
    Rect(TextInputRect),
    Substring(TextInputSubstring),
    Index(usize),
}

#[derive(Default)]
struct TextInputState {
    handler: HandlerSlot<Box<dyn TextInputHandler>>,
    handler_present: Cell<bool>,
    pending: RefCell<VecDeque<Action>>,
    delivering: Cell<bool>,
    query_depth: Cell<usize>,
}

struct ActionGuard<'a>(&'a TextInputState);

impl Drop for ActionGuard<'_> {
    fn drop(&mut self) {
        self.0.delivering.set(false);
    }
}

struct QueryGuard<'a>(&'a TextInputState);

impl Drop for QueryGuard<'_> {
    fn drop(&mut self) {
        let depth = self.0.query_depth.get() - 1;
        self.0.query_depth.set(depth);
        if depth == 0 {
            self.0.drain();
        }
    }
}

impl TextInputState {
    fn query(&self, selector: Query, range: Utf16Range, point: CGPoint) -> Option<Response> {
        self.query_depth.set(self.query_depth.get() + 1);
        let _guard = QueryGuard(self);
        self.handler
            .invoke("AppKit text input query", |handler| match selector {
                Query::HasMarkedText => Some(Response::Marked(handler.has_marked_text())),
                Query::MarkedRange => handler
                    .marked_range()
                    .filter(|range| range.is_valid())
                    .map(Response::Range),
                Query::SelectedRange => handler
                    .selected_range()
                    .filter(|range| range.is_valid())
                    .map(Response::Range),
                Query::FirstRect => handler
                    .first_rect_for_character_range(range)
                    .filter(|result| {
                        let rect = result.bounds;
                        result.actual_range.is_valid()
                            && rect.origin.x.is_finite()
                            && rect.origin.y.is_finite()
                            && rect.size.width.is_finite()
                            && rect.size.height.is_finite()
                            && rect.size.width >= 0.0
                            && rect.size.height >= 0.0
                    })
                    .map(Response::Rect),
                Query::AttributedSubstring => handler
                    .attributed_substring_for_proposed_range(range)
                    .filter(|result| result.actual_range.is_valid())
                    .map(Response::Substring),
                Query::CharacterIndex => handler
                    .character_index_for_point(point)
                    .filter(|index| *index < Utf16Range::INVALID.location)
                    .map(Response::Index),
            })
            .flatten()
    }

    fn action(&self, action: Action) {
        self.pending.borrow_mut().push_back(action);
        self.drain();
    }

    fn command(&self, selector: &str) -> bool {
        if !self.handler_present.get() || self.delivering.get() || self.query_depth.get() != 0 {
            return false;
        }
        let handled = {
            self.delivering.set(true);
            let _guard = ActionGuard(self);
            self.handler
                .invoke("AppKit text input command", |handler| {
                    handler.do_command_by_selector(selector)
                })
                .unwrap_or(false)
        };
        self.drain();
        handled
    }

    fn drain(&self) {
        if self.delivering.get() || self.query_depth.get() != 0 || self.pending.borrow().is_empty()
        {
            return;
        }
        self.delivering.set(true);
        let _guard = ActionGuard(self);
        loop {
            if !self.handler_present.get() {
                self.pending.borrow_mut().clear();
                break;
            }
            let next = self.pending.borrow_mut().pop_front();
            let Some(action) = next else {
                break;
            };
            let _ = self
                .handler
                .invoke("AppKit text input action", |handler| match action {
                    Action::Insert(text, replacement) => handler.insert_text(&text, replacement),
                    Action::Mark(text, selected, replacement) => {
                        handler.set_marked_text(&text, selected, replacement);
                    }
                    Action::Unmark => handler.unmark_text(),
                });
        }
    }
}

fn with_state<R>(raw_token: u64, site: &str, call: impl FnOnce(&TextInputState) -> R) -> Option<R> {
    catch_user_panic_result(site, || {
        let main_thread = MainThreadMarker::new()?;
        let token = RegistrationToken::from_raw(raw_token)?;
        let state = CallbackRegistration::<TextInputState>::snapshot(token, main_thread)?;
        Some(call(&state))
    })
    .flatten()
}

extern "C" fn on_query(
    token: u64,
    raw_selector: u32,
    range: Utf16Range,
    x: f64,
    y: f64,
    out: *mut TextInputQueryResult,
    context: *mut c_void,
    copy_text: Option<CopyUtf8>,
) -> bool {
    let Some(selector) = Query::from_raw(raw_selector) else {
        return false;
    };
    if out.is_null()
        || (matches!(selector, Query::CharacterIndex) && (!x.is_finite() || !y.is_finite()))
    {
        return false;
    }
    let Some(response) = with_state(token, "AppKit text input query callback", |state| {
        state.query(selector, range, CGPoint::new(x, y))
    })
    .flatten() else {
        return false;
    };
    let mut result = TextInputQueryResult {
        range: Utf16Range::INVALID,
        rect: CGRect::new(0.0, 0.0, 0.0, 0.0),
        character_index: Utf16Range::INVALID.location,
        has_marked_text: false,
    };
    match response {
        Response::Marked(marked) => result.has_marked_text = marked,
        Response::Range(range) => result.range = range,
        Response::Rect(rect) => {
            result.range = rect.actual_range;
            result.rect = rect.bounds;
        }
        Response::Substring(substring) => {
            let Some(copy_text) = copy_text else {
                return false;
            };
            if !copy_text(context, substring.text.as_ptr(), substring.text.len()) {
                return false;
            }
            result.range = substring.actual_range;
        }
        Response::Index(index) => result.character_index = index,
    }
    unsafe { out.write(result) };
    true
}

extern "C" fn on_action(
    token: u64,
    raw_selector: u32,
    bytes: *const u8,
    length: usize,
    selection: Utf16Range,
    replacement: Utf16Range,
) {
    let Some(selector) = ActionKind::from_raw(raw_selector) else {
        return;
    };
    let _ = with_state(token, "AppKit text input action callback", |state| {
        let action = match selector {
            ActionKind::Unmark => Action::Unmark,
            ActionKind::Insert | ActionKind::Mark => {
                let Some(text) = (unsafe { copy_utf8(bytes, length) }) else {
                    return;
                };
                match selector {
                    ActionKind::Insert => Action::Insert(text, replacement),
                    ActionKind::Mark => Action::Mark(text, selection, replacement),
                    ActionKind::Unmark => unreachable!(),
                }
            }
        };
        state.action(action);
    });
}

extern "C" fn on_command(token: u64, bytes: *const u8, length: usize) -> bool {
    with_state(token, "AppKit text input command callback", |state| {
        (unsafe { copy_utf8(bytes, length) }).is_some_and(|selector| state.command(&selector))
    })
    .unwrap_or(false)
}

pub struct TextInputClient {
    registration: CallbackRegistration<TextInputState>,
    view: View,
}

impl TextInputClient {
    #[must_use]
    pub fn install(main_thread: MainThreadMarker, view: &HostedView) -> Option<Self> {
        let registration =
            CallbackRegistration::new(main_thread, Rc::new(TextInputState::default()));
        let table = TextInputCallbackTableV1 {
            header: CallbackTableHeader {
                version: 1,
                size: size_of::<TextInputCallbackTableV1>(),
            },
            deactivate: CallbackTableV1::new(None, None).deactivate,
            query: Some(on_query),
            action: Some(on_action),
            command: Some(on_command),
        };
        if !unsafe {
            ffi::appkit_text_input_install(
                view.as_ptr(),
                registration.token().get(),
                &raw const table,
            )
        } {
            return None;
        }
        Some(Self {
            registration,
            view: view.as_view(),
        })
    }

    pub fn set_handler(&self, handler: Option<Box<dyn TextInputHandler>>) {
        let state = self.registration.state();
        state.handler_present.set(handler.is_some());
        let _ = state.handler.replace(handler);
    }

    #[must_use]
    pub fn handle_event(&self, event: EventRef<'_>) -> TextInputEventStatus {
        TextInputEventStatus::from_raw(unsafe {
            ffi::appkit_text_input_handle_event(
                self.view.as_ptr(),
                self.registration.token().get(),
                event.as_ptr(),
            )
        })
    }

    #[must_use]
    pub fn discard_marked_text(&self) -> bool {
        unsafe {
            ffi::appkit_text_input_discard_marked_text(
                self.view.as_ptr(),
                self.registration.token().get(),
            )
        }
    }

    #[must_use]
    pub fn invalidate_character_coordinates(&self) -> bool {
        unsafe {
            ffi::appkit_text_input_invalidate_character_coordinates(
                self.view.as_ptr(),
                self.registration.token().get(),
            )
        }
    }
}

impl Drop for TextInputClient {
    fn drop(&mut self) {
        let _ = self.registration.deactivate();
        unsafe {
            ffi::appkit_text_input_uninstall(self.view.as_ptr(), self.registration.token().get());
        }
    }
}
