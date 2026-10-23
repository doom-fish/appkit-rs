use appkit::application_host::MENU_ITEM_ACTION_SELECTOR;
use appkit::menu::{KeyEquivalentModifiers, Menu, MenuItem, MenuItemAction, MenuItemState};
use appkit::MainThreadMarker;

fn main() {
    let main_thread = MainThreadMarker::new().expect("the main thread");
    let menu = Menu::new(main_thread, "File").expect("a menu");
    assert_eq!(menu.title().as_deref(), Some("File"));
    assert!(menu.is_empty());
    assert!(menu.autoenables_items());
    assert!(menu.set_autoenables_items(false));
    assert!(!menu.autoenables_items());
    assert!(menu.set_title("Café\0Menu"));
    assert_eq!(menu.title().as_deref(), Some("Café\0Menu"));

    let item = check_action_item(main_thread);
    check_menu_contents(main_thread, &menu, item);
    check_standard_actions(main_thread);

    let dock_menu = Menu::new(main_thread, "Dock").expect("a Dock menu");
    let retained = dock_menu.retain_for_dock();
    assert_eq!(retained.as_ptr(), dock_menu.as_ptr());
    drop(dock_menu);
    drop(retained);
    println!("The menu properties and the responder actions are correct.");
}

fn check_action_item(main_thread: MainThreadMarker) -> MenuItem {
    let item = MenuItem::new(main_thread, "Open", MenuItemAction::ApplicationHost)
        .expect("an action item");
    assert_eq!(item.title().as_deref(), Some("Open"));
    assert_eq!(item.action(), MenuItemAction::ApplicationHost);
    assert_eq!(
        item.action_selector().as_deref(),
        Some(MENU_ITEM_ACTION_SELECTOR)
    );
    assert!(!item.has_explicit_target());
    assert!(item.is_enabled());
    assert_eq!(item.state(), Some(MenuItemState::Off));
    assert_eq!(item.tag(), Some(0));
    assert_eq!(item.represented_integer(), None);
    assert_eq!(item.key_equivalent().as_deref(), Some(""));
    assert_eq!(
        item.key_equivalent_modifiers(),
        KeyEquivalentModifiers::COMMAND
    );
    assert!(item.allows_automatic_key_equivalent_localization());

    assert!(item.set_title("Ca\u{0301}fé\0Item"));
    assert_eq!(item.title().as_deref(), Some("Ca\u{0301}fé\0Item"));
    assert!(item.set_tag(-42));
    assert_eq!(item.tag(), Some(-42));
    assert!(item.set_represented_integer(Some(0)));
    assert_eq!(item.represented_integer(), Some(0));
    assert!(item.set_represented_integer(Some(i64::MIN)));
    assert_eq!(item.represented_integer(), Some(i64::MIN));
    assert!(item.set_represented_integer(None));
    assert_eq!(item.represented_integer(), None);
    assert!(item.set_state(MenuItemState::Mixed));
    assert_eq!(item.state(), Some(MenuItemState::Mixed));
    assert!(item.set_state(MenuItemState::On));
    assert_eq!(item.state(), Some(MenuItemState::On));
    assert!(item.set_enabled(false));
    assert!(!item.is_enabled());

    let modifiers = KeyEquivalentModifiers::COMMAND | KeyEquivalentModifiers::SHIFT;
    assert!(item.set_key_equivalent(Some(("k", modifiers))));
    assert_eq!(item.key_equivalent().as_deref(), Some("k"));
    assert_eq!(item.key_equivalent_modifiers(), modifiers);
    assert!(!item.allows_automatic_key_equivalent_localization());
    assert!(item.set_key_equivalent(Some(("\u{F700}", KeyEquivalentModifiers::CONTROL,))));
    assert_eq!(item.key_equivalent().as_deref(), Some("\u{F700}"));
    assert_eq!(
        item.key_equivalent_modifiers(),
        KeyEquivalentModifiers::CONTROL
    );
    assert!(item.set_key_equivalent(None));
    assert_eq!(item.key_equivalent().as_deref(), Some(""));
    assert_eq!(
        item.key_equivalent_modifiers(),
        KeyEquivalentModifiers::COMMAND
    );
    assert!(item.allows_automatic_key_equivalent_localization());

    assert!(item.set_action(MenuItemAction::Copy));
    assert_eq!(item.action_selector().as_deref(), Some("copy:"));
    assert!(!item.has_explicit_target());
    assert!(item.set_action(MenuItemAction::ApplicationHost));
    assert_eq!(
        item.action_selector().as_deref(),
        Some(MENU_ITEM_ACTION_SELECTOR)
    );
    assert!(!item.set_action(MenuItemAction::Other));

    item
}

fn check_menu_contents(main_thread: MainThreadMarker, menu: &Menu, item: MenuItem) {
    let separator = MenuItem::separator(main_thread).expect("a separator");
    assert!(separator.is_separator());
    assert!(!separator.set_action(MenuItemAction::Cut));
    assert!(menu.add_item(&item));
    assert!(menu.add_item(&separator));
    assert_eq!(menu.len(), 2);
    drop(item);
    let retained_item = menu.item_at(0).expect("a retained item");
    assert_eq!(retained_item.tag(), Some(-42));
    assert!(menu.item_at(usize::MAX).is_none());
    assert!(!menu.insert_item(usize::MAX, &retained_item));

    let submenu = Menu::new(main_thread, "Services").expect("a submenu");
    let submenu_item =
        MenuItem::new(main_thread, "Services", MenuItemAction::None).expect("a submenu item");
    assert!(submenu_item.action_selector().is_none());
    assert!(submenu_item.set_submenu(Some(&submenu)));
    drop(submenu);
    assert_eq!(
        submenu_item
            .submenu()
            .and_then(|menu| menu.title())
            .as_deref(),
        Some("Services")
    );
    assert!(menu.insert_item(0, &submenu_item));
    assert_eq!(
        menu.item_at(0).and_then(|item| item.title()).as_deref(),
        Some("Services")
    );
    assert!(submenu_item.set_submenu(None));
    assert!(submenu_item.submenu().is_none());
    assert!(!menu.remove_item_at(usize::MAX));
    assert!(menu.remove_item_at(0));
}

fn check_standard_actions(main_thread: MainThreadMarker) {
    for (action, selector) in [
        (MenuItemAction::Cut, "cut:"),
        (MenuItemAction::Copy, "copy:"),
        (MenuItemAction::Paste, "paste:"),
        (MenuItemAction::SelectAll, "selectAll:"),
        (MenuItemAction::Undo, "undo:"),
        (MenuItemAction::Redo, "redo:"),
        (MenuItemAction::PerformClose, "performClose:"),
        (MenuItemAction::PerformMiniaturize, "performMiniaturize:"),
        (MenuItemAction::PerformZoom, "performZoom:"),
        (MenuItemAction::Hide, "hide:"),
        (
            MenuItemAction::HideOtherApplications,
            "hideOtherApplications:",
        ),
        (
            MenuItemAction::UnhideAllApplications,
            "unhideAllApplications:",
        ),
        (MenuItemAction::Terminate, "terminate:"),
        (MenuItemAction::ToggleFullScreen, "toggleFullScreen:"),
        (MenuItemAction::ToggleTabBar, "toggleTabBar:"),
        (MenuItemAction::SelectNextTab, "selectNextTab:"),
        (MenuItemAction::SelectPreviousTab, "selectPreviousTab:"),
        (MenuItemAction::MoveTabToNewWindow, "moveTabToNewWindow:"),
        (MenuItemAction::MergeAllWindows, "mergeAllWindows:"),
        (MenuItemAction::ShowAllTabs, "showAllTabs:"),
        (MenuItemAction::ArrangeInFront, "arrangeInFront:"),
        (
            MenuItemAction::OrderFrontStandardAboutPanel,
            "orderFrontStandardAboutPanel:",
        ),
        (MenuItemAction::ShowHelp, "showHelp:"),
    ] {
        let system_item = MenuItem::new(main_thread, "System", action).expect("a system item");
        assert_eq!(system_item.action(), action);
        assert_eq!(system_item.action_selector().as_deref(), Some(selector));
        assert!(!system_item.has_explicit_target());
    }
}
