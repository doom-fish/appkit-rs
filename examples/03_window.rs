use appkit::prelude::*;

fn main() -> Result<(), AppKitError> {
    let main_thread = MainThreadMarker::new().expect("an example starts on the main thread");
    let application = Application::shared(main_thread);
    assert!(application.set_activation_policy(ActivationPolicy::Regular));
    let window = Window::new(main_thread, CGRect::new(200.0, 200.0, 480.0, 320.0), "appkit-rs")?;
    window.make_key_and_order_front();
    application.activate();
    println!("{} is open. Press Control-C to stop.", window.title());
    application.run();
    Ok(())
}
