use core::ffi::c_void;
use std::cell::{Cell, RefCell};
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;
use std::rc::Rc;

use appkit::drag::{
    DragContext, DragEvent, DragItem, DragOperation, DragRegistration, DragSelector,
};
use appkit::event::{EventType, ModifierFlags};
use appkit::host_view::{HostedView, MetalLayer};
use appkit::host_window::{
    HostedPanelOptions, HostedWindow, HostedWindowConfiguration, WindowStyleMask,
};
use appkit::pasteboard::{Pasteboard, TYPE_FILE_URL};
use appkit::{CGPoint, CGRect, Event, MainThreadMarker};

extern "C" {
    fn appkit_drag_verify_destination(
        window: *mut c_void,
        pasteboard: *mut c_void,
        x: f64,
        y: f64,
        source_operations: u64,
        same_window: bool,
        phase: u32,
    ) -> u64;
    fn appkit_drag_verify_source_end(window: *mut c_void, x: f64, y: f64, operation: u64) -> bool;
}

fn hidden_host(main_thread: MainThreadMarker, panel: bool) -> (HostedWindow, HostedView) {
    let configuration = HostedWindowConfiguration::new(
        CGRect::new(-10_000.0, -10_000.0, 200.0, 140.0),
        WindowStyleMask::TITLED | WindowStyleMask::CLOSABLE,
    );
    let host = if panel {
        HostedWindow::new_panel(main_thread, configuration, HostedPanelOptions::default())
    } else {
        HostedWindow::new(main_thread, configuration)
    }
    .expect("a hidden hosted window");
    let layer = MetalLayer::new(main_thread).expect("a metal layer");
    let view = unsafe {
        HostedView::new(
            main_thread,
            CGRect::new(0.0, 0.0, 200.0, 140.0),
            layer.as_ptr(),
        )
    }
    .expect("a hosted view");
    assert!(view.attach_to(&host.as_window().content_view().expect("a content view")));
    (host, view)
}

fn response(event: &DragEvent, accepts: bool) -> DragOperation {
    match event {
        DragEvent::Entered(info) | DragEvent::Updated(info) if accepts && info.paths.is_some() => {
            if info.is_source_window {
                DragOperation::MOVE
            } else {
                DragOperation::COPY
            }
        }
        DragEvent::Performed(info) if accepts && info.paths.is_some() => DragOperation::COPY,
        DragEvent::SourceMask(DragContext::OutsideApplication) => DragOperation::COPY,
        DragEvent::SourceMask(DragContext::WithinApplication | DragContext::Other(_)) => {
            DragOperation::COPY | DragOperation::MOVE
        }
        _ => DragOperation::NONE,
    }
}

fn expect_destination(
    drag: &DragRegistration,
    board: &Pasteboard,
    position: CGPoint,
    source: (DragOperation, bool),
    phase: DragSelector,
    expected: u64,
) {
    assert_eq!(
        drag.verify_destination(board, position, source.0, source.1, phase),
        expected
    );
}

fn exercise_destination_and_source(main_thread: MainThreadMarker) {
    let (host, view) = hidden_host(main_thread, false);
    let board = Rc::new(Pasteboard::unique(main_thread).expect("a unique pasteboard"));
    let paths = vec![
        PathBuf::from("/example/first name.txt"),
        PathBuf::from("/example/café.txt"),
    ];
    assert!(board.set_filenames(&paths).expect("UTF-8 file names"));
    let drag = Rc::new(DragRegistration::new(main_thread, &host, &view).expect("a drag host"));
    assert!(DragRegistration::new(main_thread, &host, &view).is_none());

    let events = Rc::new(RefCell::new(Vec::new()));
    let accept = Rc::new(Cell::new(true));
    let reenter = Rc::new(Cell::new(true));
    let observed = Rc::clone(&events);
    let allowed = Rc::clone(&accept);
    let nested_once = Rc::clone(&reenter);
    let nested_drag = Rc::downgrade(&drag);
    let nested_board = Rc::clone(&board);
    drag.set_handler(Some(Box::new(move |event| {
        observed.borrow_mut().push(event.clone());
        if matches!(event, DragEvent::Entered(_)) && nested_once.replace(false) {
            assert_eq!(
                nested_drag
                    .upgrade()
                    .expect("an active drag")
                    .verify_destination(
                        &nested_board,
                        CGPoint::new(33.0, 29.0),
                        DragOperation::COPY,
                        false,
                        DragSelector::Updated,
                    ),
                0
            );
            assert_eq!(
                nested_drag
                    .upgrade()
                    .expect("an active drag")
                    .verify_source_mask(DragContext::OutsideApplication),
                DragOperation::NONE
            );
        }
        response(&event, allowed.get())
    })));

    verify_entered(&drag, &board, &events, &paths);
    verify_performed(&drag, &board, &events);
    verify_local_and_rejected(&drag, &board, &accept);
    verify_file_url(&drag, &board, &events);
    verify_source(&drag, &events);
    verify_items(main_thread, &drag, &paths);
    let count = events.borrow().len();
    drag.deactivate_callbacks();
    host.deactivate_callbacks();
    view.deactivate_callbacks();
    drag.deactivate_callbacks();
    expect_destination(
        &drag,
        &board,
        CGPoint::new(7.0, 8.0),
        (DragOperation::COPY, false),
        DragSelector::Conclude,
        0,
    );
    assert_eq!(
        drag.verify_source_mask(DragContext::OutsideApplication),
        DragOperation::NONE
    );
    assert_eq!(events.borrow().len(), count);
}

fn verify_entered(
    drag: &DragRegistration,
    board: &Pasteboard,
    events: &RefCell<Vec<DragEvent>>,
    paths: &[PathBuf],
) {
    expect_destination(
        drag,
        board,
        CGPoint::new(11.0, 19.0),
        (DragOperation::COPY, false),
        DragSelector::Entered,
        1,
    );
    let observed = events.borrow();
    assert!(matches!(
        observed.as_slice(),
        [DragEvent::Entered(_), DragEvent::Updated(_)]
    ));
    for (event, position) in observed
        .iter()
        .zip([CGPoint::new(11.0, 19.0), CGPoint::new(33.0, 29.0)])
    {
        let (DragEvent::Entered(info) | DragEvent::Updated(info)) = event else {
            panic!("the destination sends enter and update");
        };
        assert_eq!(info.position_in_window, position);
        assert_eq!(info.source_operations, DragOperation::COPY);
        assert!(!info.is_source_window);
        assert_eq!(info.paths.as_deref(), Some(paths));
    }
}

fn verify_performed(drag: &DragRegistration, board: &Pasteboard, events: &RefCell<Vec<DragEvent>>) {
    let position = CGPoint::new(33.0, 29.0);
    let source = (DragOperation::COPY, false);
    expect_destination(drag, board, position, source, DragSelector::Prepare, 1);
    expect_destination(drag, board, position, source, DragSelector::Perform, 1);
    assert!(matches!(
        events.borrow().last(),
        Some(DragEvent::Performed(_))
    ));
    for phase in [DragSelector::Exited, DragSelector::Conclude] {
        expect_destination(drag, board, position, source, phase, 0);
    }
    assert!(matches!(
        events.borrow().last(),
        Some(DragEvent::Concluded(_))
    ));
    expect_destination(
        drag,
        board,
        CGPoint::new(0.0, 0.0),
        source,
        DragSelector::Prepare,
        0,
    );
}

fn verify_local_and_rejected(drag: &DragRegistration, board: &Pasteboard, accept: &Cell<bool>) {
    let position = CGPoint::new(0.0, 0.0);
    let source = (DragOperation::COPY | DragOperation::MOVE, true);
    expect_destination(
        drag,
        board,
        position,
        source,
        DragSelector::Entered,
        DragOperation::MOVE.bits(),
    );
    expect_destination(
        drag,
        board,
        position,
        source,
        DragSelector::Updated,
        DragOperation::MOVE.bits(),
    );
    expect_destination(
        drag,
        board,
        position,
        (DragOperation::COPY, true),
        DragSelector::Updated,
        0,
    );
    accept.set(false);
    expect_destination(
        drag,
        board,
        CGPoint::new(1.0, 2.0),
        (DragOperation::COPY, false),
        DragSelector::Entered,
        0,
    );
    expect_destination(
        drag,
        board,
        CGPoint::new(1.0, 2.0),
        (DragOperation::COPY, false),
        DragSelector::Perform,
        0,
    );
    accept.set(true);
}

fn verify_file_url(drag: &DragRegistration, board: &Pasteboard, events: &RefCell<Vec<DragEvent>>) {
    assert!(board.declare_types(&[TYPE_FILE_URL]) >= 0);
    assert!(board.set_string_for_type(TYPE_FILE_URL, "file:///example/file%20URL.txt"));
    let start = events.borrow().len();
    expect_destination(
        drag,
        board,
        CGPoint::new(5.0, 6.0),
        (DragOperation::COPY, false),
        DragSelector::Entered,
        1,
    );
    let observed = events.borrow();
    let DragEvent::Entered(info) = &observed[start] else {
        panic!("the file URL sends an enter event");
    };
    assert_eq!(info.position_in_window, CGPoint::new(5.0, 6.0));
    assert_eq!(info.source_operations, DragOperation::COPY);
    assert!(!info.is_source_window);
    assert_eq!(
        info.paths.as_deref(),
        Some([PathBuf::from("/example/file URL.txt")].as_slice())
    );
    drop(observed);
    assert!(board.clear_contents() >= 0);
    let start = events.borrow().len();
    expect_destination(
        drag,
        board,
        CGPoint::new(7.0, 8.0),
        (DragOperation::COPY, false),
        DragSelector::Entered,
        0,
    );
    assert!(matches!(
        &events.borrow()[start],
        DragEvent::Entered(info) if info.paths.is_none()
    ));
}

fn verify_source(drag: &DragRegistration, events: &RefCell<Vec<DragEvent>>) {
    assert_eq!(
        drag.verify_source_mask(DragContext::OutsideApplication),
        DragOperation::COPY
    );
    assert_eq!(
        drag.verify_source_mask(DragContext::WithinApplication),
        DragOperation::COPY | DragOperation::MOVE
    );
    assert!(drag.verify_source_end(CGPoint::new(88.0, 44.0), DragOperation::MOVE));
    assert!(matches!(
        events.borrow().last(),
        Some(DragEvent::SourceEnded {
            screen_position,
            operation: DragOperation::MOVE,
        }) if *screen_position == CGPoint::new(88.0, 44.0)
    ));
}

fn verify_items(main_thread: MainThreadMarker, drag: &DragRegistration, paths: &[PathBuf]) {
    let mouse = Event::synthetic_mouse(
        main_thread,
        EventType::LeftMouseDown,
        CGPoint::new(18.0, 24.0),
        ModifierFlags::default(),
        1,
        0.0,
    )
    .expect("a retained mouse-down event");
    let moved = Event::synthetic_mouse(
        main_thread,
        EventType::MouseMoved,
        CGPoint::new(18.0, 24.0),
        ModifierFlags::default(),
        0,
        0.0,
    )
    .expect("a retained mouse-move event");
    let opaque = PathBuf::from(OsString::from_vec(b"/example/\xff.bin".to_vec()));
    let payload = [
        DragItem::File {
            path: &paths[0],
            is_directory: false,
        },
        DragItem::File {
            path: &opaque,
            is_directory: true,
        },
        DragItem::Text("text with café"),
    ];
    assert_eq!(drag.verify_items(&mouse, &payload), 3);
    assert_eq!(drag.verify_items(&moved, &payload), 0);
    assert!(!drag.start(&moved, &payload));
    let invalid = PathBuf::from("/example/bad\0file");
    assert_eq!(
        drag.verify_items(
            &mouse,
            &[DragItem::File {
                path: &invalid,
                is_directory: false,
            }],
        ),
        0
    );
    assert_eq!(drag.verify_items(&mouse, &[]), 0);
}

fn exercise_panel_and_teardown(main_thread: MainThreadMarker) {
    let (host, view) = hidden_host(main_thread, true);
    let board = Pasteboard::unique(main_thread).expect("a unique pasteboard");
    assert!(board
        .set_filenames(&[PathBuf::from("/example/panel.txt")])
        .expect("a valid path"));
    let drag = DragRegistration::new(main_thread, &host, &view).expect("a hosted panel drag");
    let calls = Rc::new(Cell::new(0));
    let seen = Rc::clone(&calls);
    drag.set_handler(Some(Box::new(move |event| {
        seen.set(seen.get() + 1);
        response(&event, true)
    })));
    expect_destination(
        &drag,
        &board,
        CGPoint::new(1.0, 3.0),
        (DragOperation::COPY | DragOperation::MOVE, true),
        DragSelector::Entered,
        DragOperation::MOVE.bits(),
    );
    assert_eq!(calls.get(), 1);
    let native = host.as_window();
    drop(view);
    expect_destination(
        &drag,
        &board,
        CGPoint::new(1.0, 3.0),
        (DragOperation::MOVE, true),
        DragSelector::Updated,
        0,
    );
    assert_eq!(calls.get(), 1);
    drop(host);
    assert_eq!(
        drag.verify_source_mask(DragContext::WithinApplication),
        DragOperation::NONE
    );
    drop(drag);
    assert_eq!(
        unsafe {
            appkit_drag_verify_destination(native.as_ptr(), board.as_ptr(), 0.0, 0.0, 1, false, 6)
        },
        0
    );
    assert!(!unsafe { appkit_drag_verify_source_end(native.as_ptr(), 0.0, 0.0, 1) });
    assert_eq!(calls.get(), 1);
}

fn exercise_close_during_drop(main_thread: MainThreadMarker) {
    let (host, view) = hidden_host(main_thread, false);
    let board = Pasteboard::unique(main_thread).expect("a unique pasteboard");
    assert!(board
        .set_filenames(&[PathBuf::from("/example/drop.txt")])
        .expect("a valid path"));
    let drag = DragRegistration::new(main_thread, &host, &view).expect("a drag host");
    let native = host.as_window();
    let closing = native.clone();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let observed = Rc::clone(&seen);
    drag.set_handler(Some(Box::new(move |event| {
        observed.borrow_mut().push(event.clone());
        if matches!(event, DragEvent::Performed(_)) {
            closing.close();
        }
        response(&event, true)
    })));
    let position = CGPoint::new(10.0, 12.0);
    let source = (DragOperation::COPY, false);
    expect_destination(&drag, &board, position, source, DragSelector::Entered, 1);
    expect_destination(&drag, &board, position, source, DragSelector::Perform, 1);
    assert_eq!(seen.borrow().len(), 2);
    expect_destination(&drag, &board, position, source, DragSelector::Conclude, 0);
    assert_eq!(seen.borrow().len(), 2);
    assert!(!drag.verify_source_end(CGPoint::new(3.0, 4.0), DragOperation::COPY));
    drop(drag);
    assert_eq!(
        unsafe {
            appkit_drag_verify_destination(native.as_ptr(), board.as_ptr(), 0.0, 0.0, 1, false, 6)
        },
        0
    );
}

fn exercise_queued_event_during_close(main_thread: MainThreadMarker) {
    let (host, view) = hidden_host(main_thread, false);
    let board = Rc::new(Pasteboard::unique(main_thread).expect("a unique pasteboard"));
    assert!(board
        .set_filenames(&[PathBuf::from("/example/queued.txt")])
        .expect("a valid path"));
    let drag = Rc::new(DragRegistration::new(main_thread, &host, &view).expect("a drag host"));
    let weak = Rc::downgrade(&drag);
    let nested_board = Rc::clone(&board);
    let native = host.as_window();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let observed = Rc::clone(&seen);
    drag.set_handler(Some(Box::new(move |event| {
        let entered = matches!(event, DragEvent::Entered(_));
        observed.borrow_mut().push(event.clone());
        if entered {
            assert_eq!(
                weak.upgrade().expect("a live owner").verify_destination(
                    &nested_board,
                    CGPoint::new(2.0, 4.0),
                    DragOperation::COPY,
                    false,
                    DragSelector::Updated,
                ),
                0
            );
            native.close();
        }
        response(&event, true)
    })));
    expect_destination(
        &drag,
        &board,
        CGPoint::new(1.0, 3.0),
        (DragOperation::COPY, false),
        DragSelector::Entered,
        0,
    );
    assert!(matches!(
        seen.borrow().as_slice(),
        [DragEvent::Entered(_), DragEvent::Updated(_)]
    ));
    expect_destination(
        &drag,
        &board,
        CGPoint::new(1.0, 3.0),
        (DragOperation::COPY, false),
        DragSelector::Conclude,
        0,
    );
    assert_eq!(seen.borrow().len(), 2);
}

fn exercise_drop_inside_handler(main_thread: MainThreadMarker) {
    let (host, view) = hidden_host(main_thread, false);
    let board = Pasteboard::unique(main_thread).expect("a unique pasteboard");
    assert!(board
        .set_filenames(&[PathBuf::from("/example/detach.txt")])
        .expect("a valid path"));
    let stored = Rc::new(RefCell::new(None));
    let drag = DragRegistration::new(main_thread, &host, &view).expect("a drag host");
    let discarded = Rc::clone(&stored);
    let calls = Rc::new(Cell::new(0));
    let seen = Rc::clone(&calls);
    drag.set_handler(Some(Box::new(move |_| {
        seen.set(seen.get() + 1);
        drop(discarded.borrow_mut().take());
        DragOperation::COPY
    })));
    *stored.borrow_mut() = Some(drag);
    let window = host.as_window();
    assert_eq!(
        unsafe {
            appkit_drag_verify_destination(window.as_ptr(), board.as_ptr(), 1.0, 2.0, 1, false, 1)
        },
        0
    );
    assert!(stored.borrow().is_none());
    assert_eq!(calls.get(), 1);
    assert_eq!(
        unsafe {
            appkit_drag_verify_destination(window.as_ptr(), board.as_ptr(), 1.0, 2.0, 1, false, 6)
        },
        0
    );
    assert!(!unsafe { appkit_drag_verify_source_end(window.as_ptr(), 2.0, 3.0, 1) });
    assert_eq!(calls.get(), 1);
}

fn main() {
    let main_thread = MainThreadMarker::new().expect("the main thread");
    exercise_destination_and_source(main_thread);
    exercise_panel_and_teardown(main_thread);
    exercise_close_during_drop(main_thread);
    exercise_queued_event_during_close(main_thread);
    exercise_drop_inside_handler(main_thread);
    println!("Hidden drag callbacks, operations, paths, and teardown passed.");
}
