use appkit::alert::{Alert, AlertSheetStart};
use appkit::application_host::{ApplicationHost, ApplicationHostHandlers};
use appkit::prelude::*;

fn main() -> Result<(), AppKitError> {
    let main_thread = MainThreadMarker::new().expect("the main thread");
    let application = Application::shared(main_thread);
    assert!(application.set_activation_policy(ActivationPolicy::Regular));
    let window = Window::new(
        main_thread,
        CGRect::new(200.0, 200.0, 480.0, 320.0),
        "Terminate later example",
    )?;
    let handlers = ApplicationHostHandlers::default();
    let reply_application = application.clone();
    let reply_window = window.clone();
    let _ = handlers
        .should_terminate_reply
        .replace(Some(Box::new(move || {
            let alert = Alert::new(main_thread);
            if alert.set_message("Quit the example?").is_err()
                || alert
                    .set_informative_text(Some("The application waits for your reply."))
                    .is_err()
                || alert.add_button("Quit", false).is_err()
                || alert.add_button("Cancel", true).is_err()
            {
                return TerminateReply::Cancel;
            }
            let application = reply_application.clone();
            match alert.begin_sheet(&reply_window, move |button| {
                let allowed = button == Some(0);
                println!("Allow termination: {allowed}");
                application.reply_to_application_should_terminate(allowed);
            }) {
                AlertSheetStart::ParentMayBeShown(session) => {
                    session.detach();
                    reply_window.make_key_and_order_front();
                    reply_application.activate();
                    println!("The application returned TerminateReply::Later.");
                    TerminateReply::Later
                }
                AlertSheetStart::AppKitDidNotStart => TerminateReply::Cancel,
            }
        })));
    let _host = ApplicationHost::install(main_thread, handlers).expect("an application delegate");
    window.make_key_and_order_front();
    application.activate();
    println!("Select Quit in the Dock menu. Select Cancel, then repeat and select Quit.");
    application.run();
    Ok(())
}
