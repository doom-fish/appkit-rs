extern "C" {
    pub fn appkit_document_controller_note_new_recent_document(
        bytes: *const u8,
        length: usize,
    ) -> bool;
}
