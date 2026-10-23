use appkit::host_window::{HostedWindow, HostedWindowConfiguration, WindowStyleMask};
use appkit::{AppKitError, CGRect, MainThreadMarker};

fn main() -> Result<(), AppKitError> {
    let main_thread = MainThreadMarker::new().expect("the example starts on the main thread");
    let configuration = HostedWindowConfiguration::new(
        CGRect::new(120.0, 180.0, 400.0, 280.0),
        WindowStyleMask::TITLED | WindowStyleMask::CLOSABLE | WindowStyleMask::RESIZABLE,
    );
    let host =
        HostedWindow::new(main_thread, configuration).expect("AppKit creates a hidden window");
    let window = host.as_window();
    let original = window.frame();
    let state = window
        .encode_restorable_state()
        .expect("AppKit archives the native window state");

    window.set_frame(CGRect::new(240.0, 260.0, 360.0, 240.0), false, false)?;
    assert_ne!(window.frame(), original);
    window.restore_state(&state)?;
    assert_eq!(window.frame(), original);

    assert!(window
        .restore_state(b"corrupt native window state")
        .is_err());
    assert!(window.restore_state(&state[..state.len() / 2]).is_err());
    assert!(window.restore_state(&[]).is_err());
    assert!(!window.is_visible());
    assert!(!window.is_key_window());
    println!("The hidden window restores its frame and rejects corrupt state.");
    Ok(())
}
