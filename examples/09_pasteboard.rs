use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;

use appkit::pasteboard::{Pasteboard, TYPE_LEGACY_FILENAMES, TYPE_PNG, TYPE_STRING};
use appkit::MainThreadMarker;

fn main() {
    let main_thread = MainThreadMarker::new().expect("the main thread");
    let pasteboard = Pasteboard::unique(main_thread).expect("a unique pasteboard");
    assert_eq!(pasteboard.types(), Vec::<String>::new());
    assert!(pasteboard.data_for_type(TYPE_STRING).is_none());
    assert!(pasteboard.string_for_type(TYPE_STRING).is_none());
    assert_eq!(pasteboard.legacy_filenames(), Vec::<PathBuf>::new());

    let previous_change = pasteboard.change_count();
    let metadata_type = "example.metadata";
    let next_change =
        pasteboard.declare_types(&[TYPE_LEGACY_FILENAMES, TYPE_STRING, TYPE_PNG, metadata_type]);
    assert!(next_change > previous_change);
    assert_eq!(pasteboard.change_count(), next_change);
    let types = pasteboard.types();
    assert!(types.iter().any(|kind| kind == TYPE_LEGACY_FILENAMES));
    assert!(types.iter().any(|kind| kind == TYPE_STRING));
    assert!(types.iter().any(|kind| kind == TYPE_PNG));
    assert!(types.iter().any(|kind| kind == metadata_type));

    let paths = vec![
        PathBuf::from("/example/file name.txt"),
        PathBuf::from("/example/café.png"),
    ];
    assert!(pasteboard.set_filenames(&paths).expect("valid UTF-8 paths"));
    assert_eq!(pasteboard.legacy_filenames(), paths);

    let paths_change = pasteboard.change_count();
    let invalid_path = PathBuf::from(OsString::from_vec(b"/example/\xff.png".to_vec()));
    assert!(pasteboard
        .set_filenames(&[paths[0].clone(), invalid_path])
        .is_err());
    assert!(pasteboard.set_filenames(&[PathBuf::new()]).is_err());
    assert!(pasteboard
        .set_filenames(&[PathBuf::from("/example/part\0name")])
        .is_err());
    assert_eq!(pasteboard.change_count(), paths_change);
    assert_eq!(pasteboard.legacy_filenames(), paths);

    let empty_filename_list = br#"<plist version="1.0"><array><string></string></array></plist>"#;
    assert!(pasteboard.set_data_for_type(TYPE_LEGACY_FILENAMES, empty_filename_list));
    assert_eq!(pasteboard.legacy_filenames(), [PathBuf::new()]);
    assert!(pasteboard.set_data_for_type(TYPE_LEGACY_FILENAMES, b"not a property list"));
    assert_eq!(pasteboard.legacy_filenames(), Vec::<PathBuf>::new());
    let valid_type_change = pasteboard.change_count();
    assert!(!pasteboard.set_string_for_type("bad\0type", "value"));
    assert_eq!(pasteboard.declare_types(&[""]), -1);
    assert_eq!(pasteboard.change_count(), valid_type_change);

    assert!(pasteboard.set_string_for_type(TYPE_STRING, ""));
    assert_eq!(pasteboard.string_for_type(TYPE_STRING).as_deref(), Some(""));
    assert!(pasteboard.set_string_for_type(TYPE_STRING, "café\0text"));
    assert_eq!(
        pasteboard.string_for_type(TYPE_STRING).as_deref(),
        Some("café\0text")
    );
    assert!(pasteboard.set_data_for_type(metadata_type, &[0, 255, 1, 0]));
    assert_eq!(
        pasteboard.data_for_type(metadata_type),
        Some(vec![0, 255, 1, 0])
    );
    assert!(pasteboard.set_data_for_type(TYPE_PNG, &[]));
    assert_eq!(pasteboard.data_for_type(TYPE_PNG), Some(Vec::new()));
    assert!(pasteboard.data_for_type("example.absent").is_none());

    let cleared_change = pasteboard.clear_contents();
    assert!(cleared_change > next_change);
    assert_eq!(pasteboard.change_count(), cleared_change);
    assert_eq!(pasteboard.types(), Vec::<String>::new());
    assert_eq!(pasteboard.legacy_filenames(), Vec::<PathBuf>::new());
    assert!(pasteboard.data_for_type(metadata_type).is_none());
    println!("The unique pasteboard releases its data when this example ends.");
}
