use appkit::cursor::{Cursor, CursorKind};
use appkit::{CGPoint, CGSize, Image, MainThreadMarker};

const PNG: [u8; 70] = [
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4,
    0x89, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0xf8, 0xcf, 0xc0, 0xf0,
    0x1f, 0x00, 0x05, 0x00, 0x01, 0xff, 0x89, 0x99, 0x3d, 0x1d, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
    0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];

fn main() {
    let main_thread = MainThreadMarker::new().expect("an example starts on the main thread");
    let kinds = [
        CursorKind::Arrow,
        CursorKind::IBeam,
        CursorKind::IBeamForVerticalLayout,
        CursorKind::Crosshair,
        CursorKind::OpenHand,
        CursorKind::ClosedHand,
        CursorKind::PointingHand,
        CursorKind::ResizeLeftRight,
        CursorKind::ResizeUpDown,
        CursorKind::ResizeLeft,
        CursorKind::ResizeRight,
        CursorKind::ResizeUp,
        CursorKind::ResizeDown,
        CursorKind::ResizeColumn,
        CursorKind::ResizeRow,
        CursorKind::ResizeUpLeftDownRight,
        CursorKind::ResizeUpRightDownLeft,
        CursorKind::OperationNotAllowed,
        CursorKind::DragLink,
        CursorKind::DragCopy,
        CursorKind::ContextualMenu,
    ];
    for kind in kinds {
        let cursor = Cursor::system(main_thread, kind);
        assert!(!cursor.as_ptr().is_null());
    }
    let image = Image::from_data(&PNG).expect("a valid cursor image");
    let custom = Cursor::from_image(
        main_thread,
        &image,
        CGSize::new(16.0, 16.0),
        CGPoint::new(0.0, 0.0),
    )
    .expect("a custom cursor");
    assert!(!custom.as_ptr().is_null());
    println!("The system and custom cursors exist. The visible cursor did not change.");
}
