use appkit::prelude::*;

fn main() {
    let main_thread = MainThreadMarker::new().expect("an example starts on the main thread");
    println!(
        "reduce motion:               {}",
        Workspace::accessibility_display_should_reduce_motion()
    );
    println!(
        "reduce transparency:         {}",
        Workspace::accessibility_display_should_reduce_transparency()
    );
    println!(
        "increase contrast:           {}",
        Workspace::accessibility_display_should_increase_contrast()
    );
    println!(
        "differentiate without color: {}",
        Workspace::accessibility_display_should_differentiate_without_color()
    );
    println!(
        "invert colors:               {}",
        Workspace::accessibility_display_should_invert_colors()
    );
    println!("VoiceOver:                   {}", Workspace::is_voice_over_enabled());
    println!(
        "preferred scroller style:    {:?}",
        Scroller::preferred_scroller_style(main_thread)
    );
}
