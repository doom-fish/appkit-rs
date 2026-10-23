use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::rc::Rc;

use appkit::application_host::{
    ApplicationHost, ApplicationHostHandlers, MenuAction, RetainedMenu, MENU_ITEM_ACTION_SELECTOR,
};
use appkit::{ffi, Application, MainThreadMarker};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Calls {
    lifecycle: Vec<&'static str>,
    reopen: Vec<bool>,
    should_terminate: usize,
    urls: Vec<Vec<String>>,
    dock_menu: usize,
    menu_actions: Vec<(MenuAction, i64)>,
    menu_validation: Vec<(MenuAction, i64)>,
    menu_will_open: usize,
    screen_parameters: usize,
}

fn probe(
    delegate: *mut c_void,
    method: i32,
    action: i32,
    tag: i64,
    has_visible_windows: bool,
) -> i64 {
    let result = unsafe {
        ffi::application_host::appkit_application_host_probe(
            delegate,
            method,
            action,
            tag,
            has_visible_windows,
        )
    };
    assert_ne!(result, -1, "invalid application host probe");
    result
}

fn configured_handlers(
    main_thread: MainThreadMarker,
    calls: &Rc<RefCell<Calls>>,
    delegate_ptr: &Rc<Cell<*mut c_void>>,
) -> ApplicationHostHandlers {
    let handlers = ApplicationHostHandlers::default();

    let seen = Rc::clone(calls);
    let nested = Rc::clone(delegate_ptr);
    let _ = handlers
        .will_finish_launching
        .replace(Some(Box::new(move || {
            seen.borrow_mut().lifecycle.push("will finish");
            assert_eq!(probe(nested.get(), 3, 0, 0, false), 0);
            assert_eq!(probe(nested.get(), 2, 0, 0, false), 1);
            assert_eq!(probe(nested.get(), 12, 0, 0, false), 0);
        })));

    let seen = Rc::clone(calls);
    let _ = handlers
        .did_finish_launching
        .replace(Some(Box::new(move || {
            seen.borrow_mut().lifecycle.push("did finish");
        })));

    let seen = Rc::clone(calls);
    let _ = handlers
        .should_handle_reopen
        .replace(Some(Box::new(move |has_visible_windows| {
            seen.borrow_mut().reopen.push(has_visible_windows);
            !has_visible_windows
        })));

    let seen = Rc::clone(calls);
    let nested = Rc::clone(delegate_ptr);
    let _ = handlers.should_terminate.replace(Some(Box::new(move || {
        let attempts = {
            let mut calls = seen.borrow_mut();
            calls.should_terminate += 1;
            calls.should_terminate
        };
        if attempts == 1 {
            assert_eq!(probe(nested.get(), 12, 0, 0, false), 1);
        }
        attempts == 2
    })));

    let seen = Rc::clone(calls);
    let _ = handlers.will_terminate.replace(Some(Box::new(move || {
        seen.borrow_mut().lifecycle.push("will terminate");
    })));

    let seen = Rc::clone(calls);
    let _ = handlers.open_urls.replace(Some(Box::new(move |urls| {
        seen.borrow_mut().urls.push(urls);
    })));

    let menu = unsafe {
        RetainedMenu::from_retained_ptr(
            ffi::application_host::appkit_application_host_probe_retained_menu(),
            main_thread,
        )
    }
    .expect("a retained Dock menu");
    let seen = Rc::clone(calls);
    let _ = handlers.dock_menu_retained.replace(Some(Box::new(move || {
        seen.borrow_mut().dock_menu += 1;
        Some(menu.clone())
    })));

    let seen = Rc::clone(calls);
    let _ = handlers
        .menu_item_action
        .replace(Some(Box::new(move |action, tag| {
            seen.borrow_mut().menu_actions.push((action, tag));
        })));

    let seen = Rc::clone(calls);
    let _ = handlers
        .validate_menu_item
        .replace(Some(Box::new(move |action, tag| {
            seen.borrow_mut().menu_validation.push((action, tag));
            action != MenuAction::Other && tag > 0
        })));

    let seen = Rc::clone(calls);
    let _ = handlers.menu_will_open.replace(Some(Box::new(move || {
        seen.borrow_mut().menu_will_open += 1;
    })));

    let seen = Rc::clone(calls);
    let _ = handlers
        .screen_parameters_changed
        .replace(Some(Box::new(move || {
            seen.borrow_mut().screen_parameters += 1;
        })));

    handlers
}

fn installed_default_host(main_thread: MainThreadMarker) -> ApplicationHost {
    let defaults = ApplicationHost::install(main_thread, ApplicationHostHandlers::default())
        .expect("the default delegate");
    let default_delegate = defaults.delegate_borrowed_ptr();
    assert_eq!(probe(default_delegate, 11, 0, 0, false), 1);
    for method in [0, 1, 3, 4, 6, 8, 9, 13] {
        assert_eq!(probe(default_delegate, method, 0, 0, false), 0);
    }
    assert_eq!(probe(default_delegate, 2, 0, 0, false), 1);
    assert_eq!(probe(default_delegate, 5, 0, 0, false), 0);
    assert_eq!(probe(default_delegate, 7, 0, 0, false), 0);
    assert_eq!(probe(default_delegate, 12, 0, 0, false), 1);
    assert_eq!(probe(default_delegate, 14, 0, 0, false), 1);
    defaults
}

fn close_during_callback(main_thread: MainThreadMarker) {
    let holder = Rc::new(RefCell::new(None));
    let weak = Rc::downgrade(&holder);
    let calls = Rc::new(Cell::new(0));
    let seen = Rc::clone(&calls);
    let handlers = ApplicationHostHandlers::default();
    let _ = handlers.menu_will_open.replace(Some(Box::new(move || {
        seen.set(seen.get() + 1);
        if let Some(holder) = weak.upgrade() {
            let host = holder.borrow_mut().take();
            drop(host);
        }
    })));
    let host = ApplicationHost::install(main_thread, handlers).expect("a disposable host");
    let delegate = host.delegate_borrowed_ptr();
    unsafe { ffi::appkit_retain(delegate) };
    holder.replace(Some(host));
    assert_eq!(probe(delegate, 8, 0, 0, false), 0);
    assert!(holder.borrow().is_none());
    assert_eq!(probe(delegate, 8, 0, 0, false), 0);
    assert_eq!(calls.get(), 1);
    unsafe { ffi::appkit_release(delegate) };
}

fn termination_from_menu_action(main_thread: MainThreadMarker, has_handler: bool) {
    let events = Rc::new(RefCell::new(Vec::new()));
    let delegate_ptr = Rc::new(Cell::new(core::ptr::null_mut()));
    let handlers = ApplicationHostHandlers::default();

    let seen = Rc::clone(&events);
    let _ = handlers.will_terminate.replace(Some(Box::new(move || {
        seen.borrow_mut().push("will terminate");
    })));

    if has_handler {
        let seen = Rc::clone(&events);
        let _ = handlers.should_terminate.replace(Some(Box::new(move || {
            seen.borrow_mut().push("should terminate");
            true
        })));
    }

    let seen = Rc::clone(&events);
    let nested = Rc::clone(&delegate_ptr);
    let _ = handlers
        .menu_item_action
        .replace(Some(Box::new(move |_, _| {
            seen.borrow_mut().push("menu start");
            assert_eq!(probe(nested.get(), 17, 0, 0, false), 1);
            seen.borrow_mut().push("menu end");
        })));

    let host = ApplicationHost::install(main_thread, handlers).expect("a menu delegate");
    let delegate = host.delegate_borrowed_ptr();
    delegate_ptr.set(delegate);
    assert_eq!(probe(delegate, 6, 0, 10, false), 0);
    if has_handler {
        assert_eq!(
            *events.borrow(),
            [
                "menu start",
                "should terminate",
                "will terminate",
                "menu end"
            ]
        );
    } else {
        assert_eq!(
            *events.borrow(),
            ["menu start", "will terminate", "menu end"]
        );
    }
}

fn termination_handler_can_close_host(main_thread: MainThreadMarker) {
    let holder = Rc::new(RefCell::new(None));
    let weak = Rc::downgrade(&holder);
    let handlers = ApplicationHostHandlers::default();
    let _ = handlers.should_terminate.replace(Some(Box::new(move || {
        if let Some(holder) = weak.upgrade() {
            let host = holder.borrow_mut().take();
            drop(host);
        }
        true
    })));

    let host = ApplicationHost::install(main_thread, handlers).expect("a disposable host");
    let delegate = host.delegate_borrowed_ptr();
    unsafe { ffi::appkit_retain(delegate) };
    holder.replace(Some(host));
    assert_eq!(probe(delegate, 12, 0, 0, false), 1);
    assert!(holder.borrow().is_none());
    assert_eq!(probe(delegate, 11, 0, 0, false), 0);
    assert_eq!(probe(delegate, 12, 0, 0, false), 1);
    unsafe { ffi::appkit_release(delegate) };
}

fn actual_termination_child(main_thread: MainThreadMarker, allowed: Option<bool>) {
    use std::io::Write as _;

    let handlers = ApplicationHostHandlers::default();
    let _ = handlers.will_terminate.replace(Some(Box::new(|| {
        println!("will terminate");
        std::io::stdout().flush().expect("a flushed callback");
    })));
    if let Some(allowed) = allowed {
        let _ = handlers.should_terminate.replace(Some(Box::new(move || {
            println!("should terminate");
            std::io::stdout().flush().expect("a flushed decision");
            allowed
        })));
    }
    let _ = handlers
        .menu_item_action
        .replace(Some(Box::new(move |_, _| {
            println!("menu action");
            std::io::stdout().flush().expect("a flushed action");
            Application::shared(main_thread).terminate();
            println!("menu action returned");
        })));
    let host = ApplicationHost::install(main_thread, handlers).expect("an application delegate");
    assert_eq!(probe(host.delegate_borrowed_ptr(), 6, 0, 10, false), 0);
    println!("child completed");
}

fn actual_termination_ends_the_process(mode: &str, expected_output: &[&str]) {
    use std::time::{Duration, Instant};

    let mut child = std::process::Command::new(std::env::current_exe().expect("the example path"))
        .arg(mode)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("a child process");
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if child.try_wait().expect("the child status").is_some() {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("the child did not terminate");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let child = child.wait_with_output().expect("the child output");
    assert!(child.status.success(), "the child process did not exit");
    let output = String::from_utf8(child.stdout).expect("the child output");
    assert_eq!(output.lines().collect::<Vec<_>>(), expected_output);
}

fn verify_launch_and_reentry(delegate: *mut c_void, calls: &Rc<RefCell<Calls>>) {
    assert_eq!(probe(delegate, 0, 0, 0, false), 0);
    assert_eq!(
        calls.borrow().lifecycle,
        vec!["will finish", "will terminate"]
    );
    assert_eq!(calls.borrow().reopen, []);
    assert_eq!(calls.borrow().should_terminate, 1);
    assert_eq!(probe(delegate, 1, 0, 0, false), 0);
}

fn run_child_mode(main_thread: MainThreadMarker, mode: &str) {
    let allowed = match mode {
        "--terminate-now" => None,
        "--terminate-with-handler" => Some(true),
        "--cancel-with-handler" => Some(false),
        _ => panic!("invalid termination mode"),
    };
    actual_termination_child(main_thread, allowed);
}

fn verify_termination_processes() {
    actual_termination_ends_the_process("--terminate-now", &["menu action", "will terminate"]);
    actual_termination_ends_the_process(
        "--terminate-with-handler",
        &["menu action", "should terminate", "will terminate"],
    );
    actual_termination_ends_the_process(
        "--cancel-with-handler",
        &[
            "menu action",
            "should terminate",
            "menu action returned",
            "child completed",
        ],
    );
}

fn main() {
    let main_thread = MainThreadMarker::new().expect("the main thread");
    if let Some(mode) = std::env::args().nth(1) {
        run_child_mode(main_thread, &mode);
        return;
    }
    let defaults = installed_default_host(main_thread);
    let default_delegate = defaults.delegate_borrowed_ptr();
    let calls = Rc::new(RefCell::new(Calls::default()));
    let delegate_ptr = Rc::new(Cell::new(core::ptr::null_mut()));
    let handlers = configured_handlers(main_thread, &calls, &delegate_ptr);
    let host = ApplicationHost::install(main_thread, handlers).expect("the application delegate");
    let delegate = host.delegate_borrowed_ptr();
    delegate_ptr.set(delegate);
    assert_eq!(probe(default_delegate, 11, 0, 0, false), 0);
    drop(defaults);
    assert_eq!(probe(delegate, 11, 0, 0, false), 1);
    assert_eq!(MENU_ITEM_ACTION_SELECTOR, "handleAppKitMenuItem:");
    assert_eq!(probe(delegate, 10, 0, 0, false), 1);

    verify_launch_and_reentry(delegate, &calls);
    assert_eq!(probe(delegate, 2, 0, 0, false), 1);
    assert_eq!(probe(delegate, 2, 0, 0, true), 0);
    assert_eq!(probe(delegate, 12, 0, 0, false), 1);
    assert_eq!(probe(delegate, 3, 0, 0, false), 0);
    assert_eq!(probe(delegate, 4, 0, 0, false), 0);
    assert_eq!(probe(delegate, 5, 0, 0, false), 1);
    assert_eq!(probe(delegate, 5, 0, 0, false), 1);

    for action in 0..=6 {
        let tag = 100 + i64::from(action);
        assert_eq!(probe(delegate, 6, action, tag, false), 0);
        assert_eq!(probe(delegate, 7, action, tag, false), 1);
    }
    assert_eq!(probe(delegate, 7, 1, -2, false), 0);
    assert_eq!(probe(delegate, 7, 1234, 10, false), 0);
    assert_eq!(probe(delegate, 8, 0, 0, false), 0);
    assert_eq!(probe(delegate, 9, 0, 0, false), 0);
    assert_eq!(probe(delegate, 13, 0, 0, false), 0);

    let expected_actions = [
        MenuAction::MenuItem,
        MenuAction::Cut,
        MenuAction::Copy,
        MenuAction::Paste,
        MenuAction::SelectAll,
        MenuAction::Undo,
        MenuAction::Redo,
    ];
    let expected_menu_actions = expected_actions
        .into_iter()
        .enumerate()
        .map(|(index, action)| (action, 100 + i64::try_from(index).unwrap()))
        .collect::<Vec<_>>();
    let snapshot = calls.borrow().clone();
    assert_eq!(
        snapshot.lifecycle,
        [
            "will finish",
            "will terminate",
            "did finish",
            "will terminate"
        ]
    );
    assert_eq!(snapshot.reopen, [false, true]);
    assert_eq!(snapshot.should_terminate, 2);
    assert_eq!(
        snapshot.urls,
        [vec![
            "https://example.invalid/first".to_string(),
            "https://example.invalid/second%20item".to_string(),
        ]]
    );
    assert_eq!(snapshot.dock_menu, 2);
    assert_eq!(snapshot.menu_actions, expected_menu_actions);
    assert_eq!(snapshot.menu_validation.len(), 9);
    assert_eq!(
        snapshot.menu_validation.last(),
        Some(&(MenuAction::Other, 10))
    );
    assert_eq!(snapshot.menu_will_open, 1);
    assert_eq!(snapshot.screen_parameters, 2);

    unsafe { ffi::appkit_retain(delegate) };
    drop(host);
    assert_eq!(probe(delegate, 11, 0, 0, false), 0);
    for method in [0, 1, 3, 4, 6, 8, 9, 13] {
        assert_eq!(probe(delegate, method, 0, 10, false), 0);
    }
    assert_eq!(probe(delegate, 2, 0, 0, false), 1);
    assert_eq!(probe(delegate, 5, 0, 0, false), 0);
    assert_eq!(probe(delegate, 7, 0, 10, false), 0);
    assert_eq!(probe(delegate, 12, 0, 0, false), 1);
    unsafe { ffi::appkit_release(delegate) };
    assert_eq!(*calls.borrow(), snapshot);
    close_during_callback(main_thread);
    termination_from_menu_action(main_thread, false);
    termination_from_menu_action(main_thread, true);
    termination_handler_can_close_host(main_thread);
    verify_termination_processes();
    println!("The application delegate remains inactive after its host closes.");
}
