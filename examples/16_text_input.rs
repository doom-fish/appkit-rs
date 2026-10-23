use core::ffi::c_void;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use appkit::event::{EventType, ModifierFlags};
use appkit::host_view::{HostedView, MetalLayer};
use appkit::host_window::{HostedWindow, HostedWindowConfiguration, WindowStyleMask};
use appkit::text_input::{TextInputClient, TextInputHandler, TextInputRect, TextInputSubstring};
use appkit::{CGPoint, CGRect, CallbackTableHeader, Event, MainThreadMarker, Utf16Range};

extern "C" {
    fn appkit_retain(object: *mut c_void);
    fn appkit_release(object: *mut c_void);
    fn appkit_text_input_install(
        view: *mut c_void,
        token: u64,
        table: *const CallbackTableHeader,
    ) -> bool;
    fn appkit_text_input_example_attributes_empty(view: *mut c_void) -> bool;
    fn appkit_text_input_example_protocol(view: *mut c_void) -> bool;
    fn appkit_text_input_example_hidden(view: *mut c_void) -> bool;
    fn appkit_text_input_example_has_marked_text(view: *mut c_void) -> bool;
    fn appkit_text_input_example_range(
        view: *mut c_void,
        selector: u32,
        out: *mut Utf16Range,
    ) -> bool;
    fn appkit_text_input_example_first_rect(
        view: *mut c_void,
        requested: *const Utf16Range,
        actual: *mut Utf16Range,
        rect: *mut CGRect,
    ) -> bool;
    fn appkit_text_input_example_attributed_substring(
        view: *mut c_void,
        requested: *const Utf16Range,
        actual: *mut Utf16Range,
        context: *mut c_void,
        copy: Option<extern "C" fn(*mut c_void, *const u8, usize) -> bool>,
    ) -> bool;
    fn appkit_text_input_example_character_index(view: *mut c_void, x: f64, y: f64) -> usize;
    fn appkit_text_input_example_insert(
        view: *mut c_void,
        bytes: *const u8,
        length: usize,
        replacement: *const Utf16Range,
        attributed: bool,
    ) -> bool;
    fn appkit_text_input_example_mark(
        view: *mut c_void,
        bytes: *const u8,
        length: usize,
        selection: *const Utf16Range,
        replacement: *const Utf16Range,
        attributed: bool,
    ) -> bool;
    fn appkit_text_input_example_unmark(view: *mut c_void) -> bool;
    fn appkit_text_input_example_command(
        view: *mut c_void,
        bytes: *const u8,
        length: usize,
    ) -> bool;
    fn appkit_text_input_example_has_context(view: *mut c_void) -> bool;
}

#[derive(Clone, Debug, PartialEq)]
enum Call {
    HasMarked,
    MarkedRange,
    SelectedRange,
    FirstRect(Utf16Range),
    Substring(Utf16Range),
    Index(CGPoint),
    Insert(String, Utf16Range),
    Mark(String, Utf16Range, Utf16Range),
    Unmark,
    Command(String),
}

struct TextState {
    text: String,
    selection: Utf16Range,
    marked: Option<Utf16Range>,
    calls: Vec<Call>,
}

struct RecordingHandler(Rc<RefCell<TextState>>);

impl TextInputHandler for RecordingHandler {
    fn has_marked_text(&mut self) -> bool {
        let mut state = self.0.borrow_mut();
        state.calls.push(Call::HasMarked);
        state.marked.is_some()
    }

    fn marked_range(&mut self) -> Option<Utf16Range> {
        let mut state = self.0.borrow_mut();
        state.calls.push(Call::MarkedRange);
        state.marked
    }

    fn selected_range(&mut self) -> Option<Utf16Range> {
        let mut state = self.0.borrow_mut();
        state.calls.push(Call::SelectedRange);
        Some(state.selection)
    }

    fn first_rect_for_character_range(&mut self, range: Utf16Range) -> Option<TextInputRect> {
        self.0.borrow_mut().calls.push(Call::FirstRect(range));
        let actual_range = match (range.location, range.length) {
            (1 | 2, 2) => Utf16Range::new(1, 2)?,
            _ => return None,
        };
        Some(TextInputRect {
            bounds: CGRect::new(4.0, 5.0, 20.0, 10.0),
            actual_range,
        })
    }

    fn insert_text(&mut self, text: &str, replacement: Utf16Range) {
        let mut state = self.0.borrow_mut();
        state.calls.push(Call::Insert(text.to_owned(), replacement));
        if let Some(utf8_range) = replacement.to_utf8_range(&state.text) {
            state.text.replace_range(utf8_range, text);
            state.selection =
                Utf16Range::new(replacement.location + text.encode_utf16().count(), 0)
                    .expect("a valid selection");
        }
        state.marked = None;
    }

    fn set_marked_text(&mut self, text: &str, selected: Utf16Range, replacement: Utf16Range) {
        let mut state = self.0.borrow_mut();
        state
            .calls
            .push(Call::Mark(text.to_owned(), selected, replacement));
        state.marked = Utf16Range::new(1, text.encode_utf16().count());
        state.selection = selected
            .location
            .checked_add(1)
            .and_then(|location| Utf16Range::new(location, selected.length))
            .unwrap_or(Utf16Range::INVALID);
    }

    fn unmark_text(&mut self) {
        let mut state = self.0.borrow_mut();
        state.calls.push(Call::Unmark);
        state.marked = None;
    }

    fn attributed_substring_for_proposed_range(
        &mut self,
        range: Utf16Range,
    ) -> Option<TextInputSubstring> {
        let mut state = self.0.borrow_mut();
        state.calls.push(Call::Substring(range));
        let adjusted = if range == Utf16Range::new(2, 2)? {
            Utf16Range::new(1, 2)?
        } else {
            range
        };
        let utf8_range = adjusted.to_utf8_range(&state.text)?;
        if utf8_range.is_empty() {
            return None;
        }
        Some(TextInputSubstring {
            text: state.text.get(utf8_range)?.to_owned(),
            actual_range: adjusted,
        })
    }

    fn character_index_for_point(&mut self, point: CGPoint) -> Option<usize> {
        self.0.borrow_mut().calls.push(Call::Index(point));
        (point == CGPoint::new(7.0, 9.0)).then_some(3)
    }

    fn do_command_by_selector(&mut self, selector: &str) {
        self.0
            .borrow_mut()
            .calls
            .push(Call::Command(selector.to_owned()));
    }
}

extern "C" fn copy_text(context: *mut c_void, bytes: *const u8, length: usize) -> bool {
    let Some(output) = (unsafe { context.cast::<Vec<u8>>().as_mut() }) else {
        return false;
    };
    if length != 0 {
        let Some(bytes) = (unsafe { bytes.as_ref() }) else {
            return false;
        };
        output.extend_from_slice(unsafe { core::slice::from_raw_parts(bytes, length) });
    }
    true
}

fn range(view: *mut c_void, selector: u32) -> Utf16Range {
    let mut result = Utf16Range::INVALID;
    assert!(unsafe { appkit_text_input_example_range(view, selector, &raw mut result) });
    result
}

fn rect(view: *mut c_void, requested: Utf16Range) -> (CGRect, Utf16Range) {
    let mut actual = Utf16Range::new(123, 1).expect("a sentinel");
    let mut bounds = CGRect::new(-1.0, -1.0, -1.0, -1.0);
    assert!(unsafe {
        appkit_text_input_example_first_rect(
            view,
            &raw const requested,
            &raw mut actual,
            &raw mut bounds,
        )
    });
    (bounds, actual)
}

fn substring(view: *mut c_void, requested: Utf16Range) -> (Option<String>, Utf16Range) {
    let mut actual = Utf16Range::new(123, 1).expect("a sentinel");
    let mut bytes = Vec::<u8>::new();
    let returned = unsafe {
        appkit_text_input_example_attributed_substring(
            view,
            &raw const requested,
            &raw mut actual,
            (&raw mut bytes).cast(),
            Some(copy_text),
        )
    };
    let text = returned.then(|| String::from_utf8(bytes).expect("UTF-8 attributed text"));
    (text, actual)
}

fn verify_null_actual_range(view: *mut c_void, requested: Utf16Range) {
    let mut screen = CGRect::new(-1.0, -1.0, -1.0, -1.0);
    assert!(unsafe {
        appkit_text_input_example_first_rect(
            view,
            &raw const requested,
            core::ptr::null_mut(),
            &raw mut screen,
        )
    });
    assert_eq!(screen, rect(view, requested).0);
    let mut bytes = Vec::<u8>::new();
    assert!(unsafe {
        appkit_text_input_example_attributed_substring(
            view,
            &raw const requested,
            core::ptr::null_mut(),
            (&raw mut bytes).cast(),
            Some(copy_text),
        )
    });
    assert_eq!(String::from_utf8(bytes).unwrap(), "😀");
}

fn insert(view: *mut c_void, text: &str, replacement: Utf16Range, attributed: bool) -> bool {
    unsafe {
        appkit_text_input_example_insert(
            view,
            text.as_ptr(),
            text.len(),
            &raw const replacement,
            attributed,
        )
    }
}

fn mark(
    view: *mut c_void,
    text: &str,
    selection: Utf16Range,
    replacement: Utf16Range,
    attributed: bool,
) -> bool {
    unsafe {
        appkit_text_input_example_mark(
            view,
            text.as_ptr(),
            text.len(),
            &raw const selection,
            &raw const replacement,
            attributed,
        )
    }
}

struct Fixture {
    main_thread: MainThreadMarker,
    window: HostedWindow,
    view: HostedView,
    client: TextInputClient,
}

impl Fixture {
    fn new() -> Self {
        let main_thread = MainThreadMarker::new().expect("the main thread");
        let mut configuration = HostedWindowConfiguration::new(
            CGRect::new(20480.0, 20480.0, 120.0, 90.0),
            WindowStyleMask::BORDERLESS,
        );
        configuration.can_become_key = false;
        configuration.can_become_main = false;
        let window = HostedWindow::new(main_thread, configuration).expect("a hidden window");
        let layer = MetalLayer::new(main_thread).expect("a metal layer");
        let view = unsafe {
            HostedView::new(
                main_thread,
                CGRect::new(0.0, 0.0, 120.0, 90.0),
                layer.as_ptr(),
            )
        }
        .expect("a hosted view");
        assert!(view.attach_to(&window.as_window().content_view().expect("a content view")));
        let incompatible = CallbackTableHeader {
            version: 1,
            size: size_of::<CallbackTableHeader>(),
        };
        assert!(!unsafe { appkit_text_input_install(view.as_ptr(), 1, &raw const incompatible) });
        let client = TextInputClient::install(main_thread, &view).expect("a text input client");
        assert!(TextInputClient::install(main_thread, &view).is_none());
        assert!(unsafe { appkit_text_input_example_hidden(view.as_ptr()) });
        Self {
            main_thread,
            window,
            view,
            client,
        }
    }
}

fn verify_no_handler(view: *mut c_void, requested: Utf16Range) {
    assert!(unsafe { appkit_text_input_example_attributes_empty(view) });
    assert!(unsafe { appkit_text_input_example_protocol(view) });
    assert!(!unsafe { appkit_text_input_example_has_marked_text(view) });
    assert_eq!(range(view, 1), Utf16Range::INVALID);
    assert_eq!(range(view, 2), Utf16Range::INVALID);
    assert_eq!(
        rect(view, requested),
        (CGRect::new(0.0, 0.0, 0.0, 0.0), Utf16Range::INVALID)
    );
    assert_eq!(substring(view, requested), (None, Utf16Range::INVALID));
    assert_eq!(
        unsafe { appkit_text_input_example_character_index(view, 0.0, 0.0) },
        Utf16Range::INVALID.location
    );
}

fn verify_queries(fixture: &Fixture) -> Rc<RefCell<TextState>> {
    let view = fixture.view.as_ptr();
    let requested = Utf16Range::new(1, 2).expect("the emoji range");
    verify_no_handler(view, requested);
    let state = Rc::new(RefCell::new(TextState {
        text: "A😀e\u{0301}Z".to_owned(),
        selection: Utf16Range::new(1, 2).expect("the selection"),
        marked: None,
        calls: Vec::new(),
    }));
    fixture
        .client
        .set_handler(Some(Box::new(RecordingHandler(state.clone()))));
    assert!(!unsafe { appkit_text_input_example_has_marked_text(view) });
    assert_eq!(range(view, 1), Utf16Range::INVALID);
    assert_eq!(range(view, 2), requested);

    let (screen, actual) = rect(view, requested);
    let frame = fixture.window.as_window().frame();
    assert_eq!(
        screen,
        CGRect::new(
            frame.origin.x + 4.0,
            frame.origin.y + 90.0 - 5.0 - 10.0,
            20.0,
            10.0
        )
    );
    assert_eq!(actual, requested);
    verify_null_actual_range(view, requested);
    let adjusted = Utf16Range::new(2, 2).expect("a surrogate split");
    assert_eq!(rect(view, adjusted).1, requested);
    assert_eq!(
        substring(view, requested),
        (Some("😀".to_owned()), requested)
    );
    assert_eq!(
        substring(view, adjusted),
        (Some("😀".to_owned()), requested)
    );
    let combining = Utf16Range::new(3, 2).expect("a combined character");
    assert_eq!(
        substring(view, combining),
        (Some("e\u{0301}".to_owned()), combining)
    );
    assert_eq!(
        substring(
            view,
            Utf16Range::new(99, 4).expect("an out-of-range request")
        ),
        (None, Utf16Range::INVALID)
    );
    assert_eq!(
        substring(view, Utf16Range::INVALID),
        (None, Utf16Range::INVALID)
    );
    assert_eq!(
        rect(view, Utf16Range::INVALID),
        (CGRect::new(0.0, 0.0, 0.0, 0.0), Utf16Range::INVALID)
    );

    let screen_point = CGPoint::new(frame.origin.x + 7.0, frame.origin.y + 90.0 - 9.0);
    assert_eq!(
        unsafe { appkit_text_input_example_character_index(view, screen_point.x, screen_point.y) },
        3
    );
    assert_eq!(
        unsafe { appkit_text_input_example_character_index(view, f64::NAN, 9.0) },
        Utf16Range::INVALID.location
    );
    assert!(state
        .borrow()
        .calls
        .contains(&Call::Index(CGPoint::new(7.0, 9.0))));
    assert!(state.borrow().calls.contains(&Call::HasMarked));
    assert!(state.borrow().calls.contains(&Call::MarkedRange));
    assert!(state.borrow().calls.contains(&Call::SelectedRange));
    assert!(state.borrow().calls.contains(&Call::FirstRect(requested)));
    assert!(state.borrow().calls.contains(&Call::Substring(adjusted)));
    assert!(state
        .borrow()
        .calls
        .contains(&Call::Substring(Utf16Range::INVALID)));
    assert!(!state
        .borrow()
        .calls
        .contains(&Call::Index(CGPoint::new(f64::NAN, 9.0))));
    state
}

fn verify_actions(fixture: &Fixture, state: &Rc<RefCell<TextState>>) {
    let view = fixture.view.as_ptr();
    let selection = Utf16Range::new(2, 1).expect("the marked selection");
    assert!(mark(
        view,
        "😀e\u{0301}",
        selection,
        Utf16Range::INVALID,
        true
    ));
    assert!(unsafe { appkit_text_input_example_has_marked_text(view) });
    assert_eq!(range(view, 1), Utf16Range::new(1, 4).unwrap());
    assert_eq!(range(view, 2), Utf16Range::new(3, 1).unwrap());
    assert!(unsafe { appkit_text_input_example_unmark(view) });
    assert!(!unsafe { appkit_text_input_example_has_marked_text(view) });

    let replacement = Utf16Range::new(1, 2).expect("the emoji replacement");
    assert!(insert(view, "🌊", replacement, true));
    assert_eq!(state.borrow().text, "A🌊e\u{0301}Z");
    assert_eq!(range(view, 2), Utf16Range::new(3, 0).unwrap());
    let requested = Utf16Range::new(1, 2).expect("the emoji range");
    assert_eq!(
        substring(view, requested),
        (Some("🌊".to_owned()), requested)
    );
    let out_of_range = Utf16Range::new(99, 17).expect("a far range");
    assert!(insert(view, "e\u{0301}", out_of_range, false));
    assert!(insert(view, "unplaced", Utf16Range::INVALID, false));
    let invalid_length = Utf16Range {
        location: Utf16Range::INVALID.location,
        length: 5,
    };
    assert!(insert(view, "invalid-length", invalid_length, false));
    let invalid_selection = Utf16Range {
        location: Utf16Range::INVALID.location,
        length: 7,
    };
    assert!(mark(
        view,
        "e\u{0301}",
        invalid_selection,
        out_of_range,
        false
    ));
    assert_eq!(range(view, 2), Utf16Range::INVALID);
    assert!(unsafe { appkit_text_input_example_unmark(view) });
    assert_eq!(state.borrow().text, "A🌊e\u{0301}Z");
    let selector = "moveLeft:";
    assert!(unsafe { appkit_text_input_example_command(view, selector.as_ptr(), selector.len()) });
    let actions: Vec<Call> = state
        .borrow()
        .calls
        .iter()
        .filter(|call| {
            matches!(
                call,
                Call::Insert(..) | Call::Mark(..) | Call::Unmark | Call::Command(..)
            )
        })
        .cloned()
        .collect();
    assert_eq!(
        actions,
        [
            Call::Mark("😀e\u{0301}".to_owned(), selection, Utf16Range::INVALID),
            Call::Unmark,
            Call::Insert("🌊".to_owned(), replacement),
            Call::Insert("e\u{0301}".to_owned(), out_of_range),
            Call::Insert("unplaced".to_owned(), Utf16Range::INVALID),
            Call::Insert("invalid-length".to_owned(), invalid_length),
            Call::Mark("e\u{0301}".to_owned(), invalid_selection, out_of_range),
            Call::Unmark,
            Call::Command(selector.to_owned()),
        ]
    );
}

fn verify_input_context(fixture: &Fixture) {
    let context = unsafe { appkit_text_input_example_has_context(fixture.view.as_ptr()) };
    assert_eq!(fixture.client.invalidate_character_coordinates(), context);
    assert_eq!(fixture.client.discard_marked_text(), context);
    let key = Event::synthetic_key(
        fixture.main_thread,
        EventType::KeyDown,
        ModifierFlags::default(),
        122,
        "\u{f704}",
        "\u{f704}",
        false,
    )
    .expect("a synthetic function key");
    let handled = fixture.client.handle_event(key.as_ref());
    if !context {
        assert!(!handled);
    }
    println!("Hidden input context: available={context}, key handled={handled}.");
    assert!(unsafe { appkit_text_input_example_hidden(fixture.view.as_ptr()) });
}

struct ReentrantHandler {
    view: *mut c_void,
    client: Weak<RefCell<Option<TextInputClient>>>,
    hosted_view: Weak<RefCell<Option<HostedView>>>,
    window: Weak<RefCell<Option<HostedWindow>>>,
    delivered: Rc<RefCell<Vec<String>>>,
    safe_queries: Rc<Cell<bool>>,
}

impl TextInputHandler for ReentrantHandler {
    fn insert_text(&mut self, text: &str, _replacement: Utf16Range) {
        self.delivered.borrow_mut().push(text.to_owned());
        if text != "outer" {
            return;
        }
        let nested = insert(self.view, "inner-one", Utf16Range::INVALID, true)
            && insert(self.view, "inner-two", Utf16Range::INVALID, false);
        let requested = Utf16Range::new(1, 2).expect("an emoji range");
        let defaults = !unsafe { appkit_text_input_example_has_marked_text(self.view) }
            && range(self.view, 1) == Utf16Range::INVALID
            && range(self.view, 2) == Utf16Range::INVALID
            && rect(self.view, requested) == (CGRect::new(0.0, 0.0, 0.0, 0.0), Utf16Range::INVALID)
            && substring(self.view, requested) == (None, Utf16Range::INVALID)
            && unsafe { appkit_text_input_example_character_index(self.view, 0.0, 0.0) }
                == Utf16Range::INVALID.location;
        self.safe_queries.set(nested && defaults);
        drop(
            self.client
                .upgrade()
                .expect("the client")
                .borrow_mut()
                .take(),
        );
        drop(
            self.hosted_view
                .upgrade()
                .expect("the hosted view")
                .borrow_mut()
                .take(),
        );
        if let Some(window) = self
            .window
            .upgrade()
            .expect("the window")
            .borrow_mut()
            .take()
        {
            window.as_window().close();
        }
    }
}

fn verify_reentrant_teardown(fixture: Fixture) {
    let native_view = fixture.view.as_ptr();
    unsafe { appkit_retain(native_view) };
    let clients = Rc::new(RefCell::new(Some(fixture.client)));
    let views = Rc::new(RefCell::new(Some(fixture.view)));
    let windows = Rc::new(RefCell::new(Some(fixture.window)));
    let delivered = Rc::new(RefCell::new(Vec::new()));
    let safe_queries = Rc::new(Cell::new(false));
    clients
        .borrow()
        .as_ref()
        .expect("the active client")
        .set_handler(Some(Box::new(ReentrantHandler {
            view: native_view,
            client: Rc::downgrade(&clients),
            hosted_view: Rc::downgrade(&views),
            window: Rc::downgrade(&windows),
            delivered: delivered.clone(),
            safe_queries: safe_queries.clone(),
        })));
    assert!(insert(native_view, "outer", Utf16Range::INVALID, false));
    assert!(safe_queries.get());
    assert!(clients.borrow().is_none());
    assert!(views.borrow().is_none());
    assert!(windows.borrow().is_none());
    assert_eq!(
        &delivered.borrow()[..],
        &["outer", "inner-one", "inner-two"]
    );
    assert!(insert(native_view, "late", Utf16Range::INVALID, false));
    assert_eq!(delivered.borrow().len(), 3);
    assert!(!unsafe { appkit_text_input_example_has_marked_text(native_view) });
    assert_eq!(range(native_view, 1), Utf16Range::INVALID);
    unsafe { appkit_release(native_view) };
}

fn main() {
    let fixture = Fixture::new();
    let state = verify_queries(&fixture);
    verify_actions(&fixture, &state);
    verify_input_context(&fixture);
    verify_reentrant_teardown(fixture);
    println!("Hidden text input selectors preserve UTF-16 ranges and reentrant inserts.");
}
