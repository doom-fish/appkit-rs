use std::cell::Cell;
use std::env;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::rc::Rc;

use appkit::alert::{Alert, AlertStyle};
use appkit::ffi::alert::AlertCallbacksV1;
use appkit::ffi::panel::PanelCallbacksV1;
use appkit::panel::{OpenPanel, SavePanel};
use appkit::{
    AppKitError, CGRect, CallbackRegistration, CallbackTableHeader, CallbackTableV1,
    MainThreadMarker, RegistrationToken, Window,
};
use doom_fish_utils::panic_safe::catch_user_panic_result;

extern "C" fn record_alert_abort(raw_token: u64, response: i64) {
    let _ = catch_user_panic_result("hidden alert close", || {
        let main_thread = MainThreadMarker::new()?;
        let token = RegistrationToken::from_raw(raw_token)?;
        let received = CallbackRegistration::<Cell<i64>>::snapshot(token, main_thread)?;
        received.set(response);
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
    assert_eq!(panel_received.get(), 1);

    let parent = Window::new(
        main_thread,
        CGRect::new(-20000.0, -20000.0, 80.0, 80.0),
        "Hidden parent",
    )?;
    assert!(!unsafe { appkit::ffi::alert::appkit_alert_can_begin_sheet(parent.as_ptr()) });

    let received = Rc::new(Cell::new(0_i64));
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
    assert_eq!(received.get(), -1001);
    assert!(!unsafe { appkit::ffi::alert::appkit_alert_can_begin_sheet(parent.as_ptr()) });

    println!("The open panel, save panel, and alert are configured but not visible.");
    Ok(())
}
