use std::path::PathBuf;

use appkit::pasteboard::Pasteboard;
use appkit::MainThreadMarker;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let paths: Vec<_> = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .map(|path| path.canonicalize())
        .collect::<Result<_, _>>()?;
    if paths.is_empty() {
        return Err("Pass at least one file path.".into());
    }
    let main_thread = MainThreadMarker::new().expect("the main thread");
    let pasteboard = Pasteboard::general(main_thread);
    if pasteboard.clear_contents() < 0 || !pasteboard.write_file_urls(&paths)? {
        return Err("AppKit did not write the file URLs.".into());
    }
    println!("The general pasteboard contains {} file URLs.", paths.len());
    println!("Paste the files into Finder to verify the result.");
    Ok(())
}
