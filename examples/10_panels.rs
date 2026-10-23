use core::ptr::{self, NonNull};
use std::cell::{Cell, RefCell};
use std::env;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::rc::Rc;

use appkit::alert::{Alert, AlertSheetStart, AlertStyle};
use appkit::ffi::alert::AlertCallbacksV1;
use appkit::ffi::panel::PanelCallbacksV1;
use appkit::panel::{OpenPanel, SavePanel};
use appkit::{
    AppKitError, CallbackRegistration, CallbackTableHeader, CallbackTableV1, MainThreadMarker,
    RegistrationToken, Window,
};
use doom_fish_utils::panic_safe::catch_user_panic_result;

struct CloseResult {
    response: Cell<i64>,
    calls: Cell<usize>,
}

extern "C" fn record_alert_abort(raw_token: u64, response: i64) {
    let _ = catch_user_panic_result("hidden alert close", || {
        let main_thread = MainThreadMarker::new()?;
        let token = RegistrationToken::from_raw(raw_token)?;
        let result = CallbackRegistration::<CloseResult>::snapshot(token, main_thread)?;
        result.response.set(response);
        result.calls.set(result.calls.get() + 1);
        Some(())
    });
}

extern "C" fn record_panel_cancel(
    raw_token: u64,
    accepted: bool,
    paths: *const appkit::ffi::panel::FileSystemPath,
    count: usize,
) {
    let _ = catch_user_panic_result("hidden panel cancel", || {
        let main_thread = MainThreadMarker::new()?;
        let token = RegistrationToken::from_raw(raw_token)?;
        let received = CallbackRegistration::<Cell<i64>>::snapshot(token, main_thread)?;
        if !accepted && paths.is_null() && count == 0 {
            received.set(received.get() + 1);
        }
        Some(())
    });
}

fn offscreen_window(main_thread: MainThreadMarker) -> Window {
    let native = unsafe { appkit::ffi::alert::appkit_alert_new_offscreen_test_window() };
    let native = NonNull::new(native).expect("an off-screen parent window");
    let window = unsafe { Window::from_raw_borrowed(native, main_thread) };
    unsafe { appkit::ffi::appkit_release(native.as_ptr()) };
    window
}

fn check_sheet_kind(
    main_thread: MainThreadMarker,
    kind: i32,
    label: &str,
) -> Result<(), AppKitError> {
    let parent = offscreen_window(main_thread);
    let companion = (kind == 4).then(|| offscreen_window(main_thread));
    assert!(
        unsafe {
            appkit::ffi::alert::appkit_alert_prepare_offscreen_parent(
                parent.as_ptr(),
                companion.as_ref().map_or(ptr::null_mut(), Window::as_ptr),
                kind,
            )
        },
        "{label}"
    );

    let alert = Alert::new(main_thread);
    assert_eq!(alert.add_button("OK", false)?, 0);
    let received = Rc::new(RefCell::new(Vec::new()));
    let result = Rc::clone(&received);
    let session = match alert.begin_sheet(&parent, move |index| result.borrow_mut().push(index)) {
        AlertSheetStart::ParentMayBeShown(session) => session,
        AlertSheetStart::AppKitDidNotStart => panic!("AppKit did not start the {label} sheet"),
    };
    drop(session);
    assert!(unsafe { appkit::ffi::alert::appkit_alert_finish_offscreen_sheet(parent.as_ptr(), 0) });
    assert!(received.borrow().is_empty(), "{label}");
    assert!(unsafe { appkit::ffi::alert::appkit_alert_run_test_main_queue() });
    assert_eq!(&*received.borrow(), &[Some(0)], "{label}");
    assert!(unsafe {
        appkit::ffi::alert::appkit_alert_close_offscreen_test_window(parent.as_ptr())
    });
    if let Some(companion) = companion {
        assert!(unsafe {
            appkit::ffi::alert::appkit_alert_close_offscreen_test_window(companion.as_ptr())
        });
    }
    Ok(())
}

fn main() -> Result<(), AppKitError> {
    let main_thread = MainThreadMarker::new().expect("an example starts on the main thread");
    let directory = env::current_dir().expect("the current directory");

    let open = OpenPanel::new(main_thread);
    assert!(open.set_can_choose_files(true));
    assert!(open.set_can_choose_directories(true));
    assert!(open.set_allows_multiple_selection(true));
    assert!(open.set_can_create_directories(true));
    assert!(open.set_resolves_aliases(false));
    assert!(open.set_shows_hidden_files(false));
    assert!(open.set_directory(Some(Path::new(""))).is_err());
    assert!(open
        .set_directory(Some(Path::new(OsStr::from_bytes(b"/\xff"))))
        .is_err());
    open.set_directory(Some(&directory.join("é")))?;
    open.set_directory(None)?;
    open.set_directory(Some(&directory))?;
    open.set_prompt(None)?;
    open.set_message(None)?;
    open.set_prompt(Some("Select"))?;
    open.set_message(Some("Select a file or a directory."))?;

    let save = SavePanel::new(main_thread);
    assert!(save.set_can_create_directories(true));
    save.set_directory(Some(&directory))?;
    save.set_suggested_name("example.txt")?;
    save.set_prompt(None)?;
    save.set_prompt(Some("Save"))?;

    let alert = Alert::new(main_thread);
    assert!(alert.set_style(AlertStyle::Warning));
    alert.set_message("Save the file?")?;
    alert.set_informative_text(Some("Select a response."))?;
    assert_eq!(alert.add_button("Save", false)?, 0);
    assert_eq!(alert.add_button("Do Not Save", false)?, 1);
    assert_eq!(alert.add_button("Cancel", true)?, 2);

    let panel_received = Rc::new(Cell::new(0_i64));
    let panel_registration = CallbackRegistration::new(main_thread, Rc::clone(&panel_received));
    let panel_callbacks = PanelCallbacksV1 {
        header: CallbackTableHeader {
            version: 1,
            size: size_of::<PanelCallbacksV1>(),
        },
        deactivate: CallbackTableV1::for_state::<()>().deactivate,
        complete: record_panel_cancel,
    };
    assert!(unsafe {
        appkit::ffi::panel::appkit_panel_check_completion_after_drop(
            panel_registration.token().get(),
            &raw const panel_callbacks,
        )
    });
    assert_eq!(panel_received.get(), 0);
    assert!(unsafe { appkit::ffi::alert::appkit_alert_run_test_main_queue() });
    assert_eq!(panel_received.get(), 1);

    for (kind, label) in [
        (0, "never-shown"),
        (1, "closed"),
        (2, "ordered-out"),
        (3, "hidden-application"),
        (4, "background-tab"),
    ] {
        check_sheet_kind(main_thread, kind, label)?;
    }

    let parent = offscreen_window(main_thread);
    let received = Rc::new(CloseResult {
        response: Cell::new(0),
        calls: Cell::new(0),
    });
    let registration = CallbackRegistration::new(main_thread, Rc::clone(&received));
    let callbacks = AlertCallbacksV1 {
        header: CallbackTableHeader {
            version: 1,
            size: size_of::<AlertCallbacksV1>(),
        },
        deactivate: CallbackTableV1::for_state::<()>().deactivate,
        complete: record_alert_abort,
    };
    assert!(unsafe {
        appkit::ffi::alert::appkit_alert_check_close_observer(
            parent.as_ptr(),
            registration.token().get(),
            &raw const callbacks,
        )
    });
    assert_eq!(received.response.get(), -1001);
    assert_eq!(received.calls.get(), 1);

    println!("The panels stayed hidden. The alert sheets stayed off-screen.");
    Ok(())
}
