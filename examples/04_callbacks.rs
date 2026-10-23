use std::rc::Rc;

use appkit::{CallbackRegistration, MainThreadMarker, NativeCallbackRegistration};

fn main() {
    let main_thread = MainThreadMarker::new().expect("the main thread");
    let owner =
        NativeCallbackRegistration::new(main_thread, Rc::new(())).expect("a Swift callback owner");
    let token = owner.registration().token();
    assert!(!owner.as_ptr().is_null());
    assert!(CallbackRegistration::<()>::snapshot(token, main_thread).is_some());

    drop(owner);
    assert!(CallbackRegistration::<()>::snapshot(token, main_thread).is_none());
    println!("The callback token is inactive after the Swift owner closes.");
}
