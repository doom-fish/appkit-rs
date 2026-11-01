use appkit::application_host::{ApplicationHost, ApplicationHostHandlers, MenuAction};
use appkit::menu::{Menu, MenuItem, MenuItemAction};
use appkit::prelude::*;

fn main() -> Result<(), AppKitError> {
    let main_thread = MainThreadMarker::new().expect("the main thread");
    let application = Application::shared(main_thread);
    assert!(application.set_activation_policy(ActivationPolicy::Regular));
    let window = Window::new(
        main_thread,
        CGRect::new(200.0, 200.0, 480.0, 320.0),
        "Dock tile example",
    )?;
    let tile = application.dock_tile();
    tile.set_progress(Some(DockProgress::Fraction(0.25)))?;
    tile.set_badge_label(None)?;
    tile.display();

    let menu = Menu::new(main_thread, "Dock progress").expect("a Dock menu");
    assert!(menu.set_autoenables_items(false));
    for (tag, title) in [
        (0, "0%"),
        (1, "50%"),
        (2, "100%"),
        (3, "Busy"),
        (4, "Clear progress"),
        (5, "Badge label"),
        (6, "Clear badge"),
    ] {
        let item = MenuItem::new(main_thread, title, MenuItemAction::ApplicationHost)
            .expect("a menu item");
        assert!(item.set_tag(tag));
        assert!(menu.add_item(&item));
    }
    let handlers = ApplicationHostHandlers::default();
    let _ = handlers
        .dock_menu_retained
        .replace(Some(Box::new(move || Some(menu.retain_for_dock()))));
    let _ = handlers
        .menu_item_action
        .replace(Some(Box::new(move |action, tag| {
            if action != MenuAction::MenuItem {
                return;
            }
            let result = match tag {
                0 => tile.set_progress(Some(DockProgress::Fraction(0.0))),
                1 => tile.set_progress(Some(DockProgress::Fraction(0.5))),
                2 => tile.set_progress(Some(DockProgress::Fraction(1.0))),
                3 => tile.set_progress(Some(DockProgress::Busy)),
                4 => tile.set_progress(None),
                5 => tile.set_badge_label(Some("1")),
                6 => tile.set_badge_label(None),
                _ => return,
            };
            if let Err(error) = result {
                eprintln!("{error}");
            }
            tile.display();
            println!("Dock badge label: {:?}", tile.badge_label());
        })));
    let _host = ApplicationHost::install(main_thread, handlers).expect("an application delegate");
    window.make_key_and_order_front();
    application.activate();
    println!("Use the Dock menu to select progress, Busy, Clear progress or a badge label.");
    application.run();
    Ok(())
}
