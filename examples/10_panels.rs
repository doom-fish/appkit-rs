use std::env;

use appkit::alert::{Alert, AlertStyle};
use appkit::panel::{OpenPanel, SavePanel};
use appkit::{AppKitError, MainThreadMarker};

fn main() -> Result<(), AppKitError> {
    let main_thread = MainThreadMarker::new().expect("an example starts on the main thread");
    let directory = env::current_dir().expect("the current directory");

    let open = OpenPanel::new(main_thread);
    open.set_can_choose_files(true);
    open.set_can_choose_directories(true);
    open.set_allows_multiple_selection(true);
    open.set_can_create_directories(true);
    open.set_resolves_aliases(false);
    open.set_shows_hidden_files(false);
    open.set_directory(Some(&directory))?;
    open.set_prompt(Some("Select"))?;
    open.set_message(Some("Select a file or a directory."))?;

    let save = SavePanel::new(main_thread);
    save.set_can_create_directories(true);
    save.set_directory(Some(&directory))?;
    save.set_suggested_name("example.txt")?;
    save.set_prompt(Some("Save"))?;

    let alert = Alert::new(main_thread);
    alert.set_style(AlertStyle::Warning);
    alert.set_message("Save the file?")?;
    alert.set_informative_text(Some("Select a response."))?;
    assert_eq!(alert.add_button("Save", false)?, 0);
    assert_eq!(alert.add_button("Do Not Save", false)?, 1);
    assert_eq!(alert.add_button("Cancel", true)?, 2);

    println!("The open panel, save panel, and alert are configured but not visible.");
    Ok(())
}
