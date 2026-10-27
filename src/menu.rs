use core::ffi::c_void;
use core::ops::{BitOr, BitOrAssign};
use core::ptr::NonNull;
use core::slice;

use crate::application_host::{ApplicationHost, RetainedMenu};
use crate::ffi::menu as ffi;
use crate::main_thread::MainThreadMarker;
use crate::object::Handle;

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplicationMenuRole {
    Main = 0,
    Services = 1,
    Windows = 2,
    Help = 3,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuItemAction {
    Other = -1,
    None = 0,
    ApplicationHost = 1,
    Cut = 2,
    Copy = 3,
    Paste = 4,
    SelectAll = 5,
    Undo = 6,
    Redo = 7,
    PerformClose = 8,
    PerformMiniaturize = 9,
    PerformZoom = 10,
    Hide = 11,
    HideOtherApplications = 12,
    UnhideAllApplications = 13,
    Terminate = 14,
    ToggleFullScreen = 15,
    ToggleTabBar = 16,
    SelectNextTab = 17,
    SelectPreviousTab = 18,
    MoveTabToNewWindow = 19,
    MergeAllWindows = 20,
    ToggleTabOverview = 21,
    ArrangeInFront = 22,
    OrderFrontStandardAboutPanel = 23,
    ShowHelp = 24,
}

impl MenuItemAction {
    const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::None,
            1 => Self::ApplicationHost,
            2 => Self::Cut,
            3 => Self::Copy,
            4 => Self::Paste,
            5 => Self::SelectAll,
            6 => Self::Undo,
            7 => Self::Redo,
            8 => Self::PerformClose,
            9 => Self::PerformMiniaturize,
            10 => Self::PerformZoom,
            11 => Self::Hide,
            12 => Self::HideOtherApplications,
            13 => Self::UnhideAllApplications,
            14 => Self::Terminate,
            15 => Self::ToggleFullScreen,
            16 => Self::ToggleTabBar,
            17 => Self::SelectNextTab,
            18 => Self::SelectPreviousTab,
            19 => Self::MoveTabToNewWindow,
            20 => Self::MergeAllWindows,
            21 => Self::ToggleTabOverview,
            22 => Self::ArrangeInFront,
            23 => Self::OrderFrontStandardAboutPanel,
            24 => Self::ShowHelp,
            _ => Self::Other,
        }
    }
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuItemState {
    Mixed = -1,
    Off = 0,
    On = 1,
}

impl MenuItemState {
    const fn from_raw(raw: i32) -> Option<Self> {
        match raw {
            -1 => Some(Self::Mixed),
            0 => Some(Self::Off),
            1 => Some(Self::On),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct KeyEquivalentModifiers(u64);

impl KeyEquivalentModifiers {
    pub const NONE: Self = Self(0);
    pub const SHIFT: Self = Self(1 << 17);
    pub const CONTROL: Self = Self(1 << 18);
    pub const OPTION: Self = Self(1 << 19);
    pub const COMMAND: Self = Self(1 << 20);
    pub const NUMERIC_PAD: Self = Self(1 << 21);
    pub const FUNCTION: Self = Self(1 << 23);

    #[must_use]
    pub const fn from_bits_retain(bits: u64) -> Self {
        Self(bits)
    }

    #[must_use]
    pub const fn bits(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl BitOr for KeyEquivalentModifiers {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl BitOrAssign for KeyEquivalentModifiers {
    fn bitor_assign(&mut self, other: Self) {
        self.0 |= other.0;
    }
}

struct OwnedBytes(ffi::AppKitMenuBytes);

impl OwnedBytes {
    fn as_slice(&self) -> Option<&[u8]> {
        if self.0.length == 0 {
            Some(&[])
        } else {
            let bytes = NonNull::new(self.0.bytes)?;
            isize::try_from(self.0.length)
                .is_ok()
                .then(|| unsafe { slice::from_raw_parts(bytes.as_ptr(), self.0.length) })
        }
    }
}

impl Drop for OwnedBytes {
    fn drop(&mut self) {
        unsafe { ffi::appkit_menu_bytes_free(self.0.bytes) };
    }
}

fn read_string(read: impl FnOnce(*mut ffi::AppKitMenuBytes) -> bool) -> Option<String> {
    let mut bytes = ffi::AppKitMenuBytes {
        bytes: core::ptr::null_mut(),
        length: 0,
    };
    if !read(&raw mut bytes) {
        return None;
    }
    String::from_utf8(OwnedBytes(bytes).as_slice()?.to_vec()).ok()
}

#[derive(Debug, Clone)]
pub struct Menu {
    handle: Handle,
    main_thread: MainThreadMarker,
}

impl Menu {
    #[must_use]
    pub fn new(main_thread: MainThreadMarker, title: &str) -> Option<Self> {
        let handle =
            unsafe { Handle::from_retained(ffi::appkit_menu_new(title.as_ptr(), title.len())) }?;
        Some(Self {
            handle,
            main_thread,
        })
    }

    #[must_use]
    pub const fn as_ptr(&self) -> *mut c_void {
        self.handle.as_ptr()
    }

    #[must_use]
    pub fn title(&self) -> Option<String> {
        read_string(|out| unsafe { ffi::appkit_menu_title(self.as_ptr(), out) })
    }

    #[must_use]
    pub fn set_title(&self, title: &str) -> bool {
        unsafe { ffi::appkit_menu_set_title(self.as_ptr(), title.as_ptr(), title.len()) }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        unsafe { ffi::appkit_menu_item_count(self.as_ptr()) }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub fn item_at(&self, index: usize) -> Option<MenuItem> {
        let handle =
            unsafe { Handle::from_retained(ffi::appkit_menu_item_at(self.as_ptr(), index)) }?;
        Some(MenuItem {
            handle,
            main_thread: self.main_thread,
        })
    }

    #[must_use]
    pub fn add_item(&self, item: &MenuItem) -> bool {
        unsafe { ffi::appkit_menu_add_item(self.as_ptr(), item.as_ptr()) }
    }

    #[must_use]
    pub fn insert_item(&self, index: usize, item: &MenuItem) -> bool {
        unsafe { ffi::appkit_menu_insert_item(self.as_ptr(), item.as_ptr(), index) }
    }

    #[must_use]
    pub fn remove_item_at(&self, index: usize) -> bool {
        unsafe { ffi::appkit_menu_remove_item_at(self.as_ptr(), index) }
    }

    #[must_use]
    pub fn autoenables_items(&self) -> bool {
        unsafe { ffi::appkit_menu_autoenables_items(self.as_ptr()) }
    }

    #[must_use]
    pub fn set_autoenables_items(&self, autoenables: bool) -> bool {
        unsafe { ffi::appkit_menu_set_autoenables_items(self.as_ptr(), autoenables) }
    }

    #[must_use]
    pub fn set_application_host(&self, host: Option<&ApplicationHost>) -> bool {
        let delegate = host.map_or(
            core::ptr::null_mut(),
            ApplicationHost::delegate_borrowed_ptr,
        );
        unsafe { ffi::appkit_menu_set_application_host(self.as_ptr(), delegate) }
    }

    #[must_use]
    pub fn application_menu(
        main_thread: MainThreadMarker,
        role: ApplicationMenuRole,
    ) -> Option<Self> {
        let handle =
            unsafe { Handle::from_retained(ffi::appkit_menu_application_menu(role as i32)) }?;
        Some(Self {
            handle,
            main_thread,
        })
    }

    #[must_use]
    pub fn application_dock_menu(main_thread: MainThreadMarker) -> Option<Self> {
        let handle =
            unsafe { Handle::from_retained(ffi::appkit_menu_application_dock_menu()) }?;
        Some(Self {
            handle,
            main_thread,
        })
    }

    #[must_use]
    pub fn perform_action_at(&self, index: usize) -> bool {
        unsafe { ffi::appkit_menu_perform_action_at(self.as_ptr(), index) }
    }

    #[must_use]
    pub fn set_application_menu(
        _main_thread: MainThreadMarker,
        role: ApplicationMenuRole,
        menu: Option<&Self>,
    ) -> bool {
        let pointer = menu.map_or(core::ptr::null_mut(), Self::as_ptr);
        unsafe { ffi::appkit_menu_set_application_menu(role as i32, pointer) }
    }

    #[must_use]
    pub fn retain_for_dock(&self) -> RetainedMenu {
        let retained = self.handle.clone();
        let pointer = retained.as_ptr();
        core::mem::forget(retained);
        unsafe { RetainedMenu::from_retained_ptr(pointer, self.main_thread) }
            .expect("a retained menu has a non-null pointer")
    }
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    handle: Handle,
    main_thread: MainThreadMarker,
}

impl MenuItem {
    #[must_use]
    pub fn new(main_thread: MainThreadMarker, title: &str, action: MenuItemAction) -> Option<Self> {
        if action == MenuItemAction::Other {
            return None;
        }
        let handle = unsafe {
            Handle::from_retained(ffi::appkit_menu_item_new(
                title.as_ptr(),
                title.len(),
                action as i32,
            ))
        }?;
        Some(Self {
            handle,
            main_thread,
        })
    }

    #[must_use]
    pub fn separator(main_thread: MainThreadMarker) -> Option<Self> {
        let handle = unsafe { Handle::from_retained(ffi::appkit_menu_item_separator()) }?;
        Some(Self {
            handle,
            main_thread,
        })
    }

    #[must_use]
    pub const fn as_ptr(&self) -> *mut c_void {
        self.handle.as_ptr()
    }

    #[must_use]
    pub fn title(&self) -> Option<String> {
        read_string(|out| unsafe { ffi::appkit_menu_item_title(self.as_ptr(), out) })
    }

    #[must_use]
    pub fn set_title(&self, title: &str) -> bool {
        unsafe { ffi::appkit_menu_item_set_title(self.as_ptr(), title.as_ptr(), title.len()) }
    }

    #[must_use]
    pub fn is_separator(&self) -> bool {
        unsafe { ffi::appkit_menu_item_is_separator(self.as_ptr()) }
    }

    #[must_use]
    pub fn submenu(&self) -> Option<Menu> {
        let handle =
            unsafe { Handle::from_retained(ffi::appkit_menu_item_submenu(self.as_ptr())) }?;
        Some(Menu {
            handle,
            main_thread: self.main_thread,
        })
    }

    #[must_use]
    pub fn set_submenu(&self, menu: Option<&Menu>) -> bool {
        let pointer = menu.map_or(core::ptr::null_mut(), Menu::as_ptr);
        unsafe { ffi::appkit_menu_item_set_submenu(self.as_ptr(), pointer) }
    }

    #[must_use]
    pub fn is_enabled(&self) -> bool {
        unsafe { ffi::appkit_menu_item_enabled(self.as_ptr()) }
    }

    #[must_use]
    pub fn set_enabled(&self, enabled: bool) -> bool {
        unsafe { ffi::appkit_menu_item_set_enabled(self.as_ptr(), enabled) }
    }

    #[must_use]
    pub fn state(&self) -> Option<MenuItemState> {
        let mut state = 0;
        unsafe { ffi::appkit_menu_item_state(self.as_ptr(), &raw mut state) }
            .then(|| MenuItemState::from_raw(state))
            .flatten()
    }

    #[must_use]
    pub fn set_state(&self, state: MenuItemState) -> bool {
        unsafe { ffi::appkit_menu_item_set_state(self.as_ptr(), state as i32) }
    }

    #[must_use]
    pub fn tag(&self) -> Option<i64> {
        let mut tag = 0;
        unsafe { ffi::appkit_menu_item_tag(self.as_ptr(), &raw mut tag) }.then_some(tag)
    }

    #[must_use]
    pub fn set_tag(&self, tag: i64) -> bool {
        unsafe { ffi::appkit_menu_item_set_tag(self.as_ptr(), tag) }
    }

    #[must_use]
    pub fn represented_integer(&self) -> Option<i64> {
        let mut value = 0;
        unsafe { ffi::appkit_menu_item_represented_integer(self.as_ptr(), &raw mut value) }
            .then_some(value)
    }

    #[must_use]
    pub fn set_represented_integer(&self, value: Option<i64>) -> bool {
        unsafe {
            ffi::appkit_menu_item_set_represented_integer(
                self.as_ptr(),
                value.is_some(),
                value.unwrap_or_default(),
            )
        }
    }

    #[must_use]
    pub fn key_equivalent(&self) -> Option<String> {
        read_string(|out| unsafe { ffi::appkit_menu_item_key_equivalent(self.as_ptr(), out) })
    }

    #[must_use]
    pub fn key_equivalent_modifiers(&self) -> KeyEquivalentModifiers {
        KeyEquivalentModifiers::from_bits_retain(unsafe {
            ffi::appkit_menu_item_key_modifiers(self.as_ptr())
        })
    }

    #[must_use]
    pub fn set_key_equivalent(&self, shortcut: Option<(&str, KeyEquivalentModifiers)>) -> bool {
        let (bytes, length, present, modifiers) = shortcut
            .map_or((core::ptr::null(), 0, false, 0), |(key, modifiers)| {
                (key.as_ptr(), key.len(), true, modifiers.bits())
            });
        unsafe {
            ffi::appkit_menu_item_set_key_equivalent(
                self.as_ptr(),
                bytes,
                length,
                present,
                modifiers,
            )
        }
    }

    #[must_use]
    pub fn allows_automatic_key_equivalent_localization(&self) -> bool {
        unsafe { ffi::appkit_menu_item_allows_automatic_key_equivalent_localization(self.as_ptr()) }
    }

    #[must_use]
    pub fn action(&self) -> MenuItemAction {
        MenuItemAction::from_raw(unsafe { ffi::appkit_menu_item_action(self.as_ptr()) })
    }

    #[must_use]
    pub fn set_action(&self, action: MenuItemAction) -> bool {
        action != MenuItemAction::Other
            && unsafe { ffi::appkit_menu_item_set_action(self.as_ptr(), action as i32) }
    }

    #[must_use]
    pub fn action_selector(&self) -> Option<String> {
        read_string(|out| unsafe { ffi::appkit_menu_item_action_selector(self.as_ptr(), out) })
    }

    #[must_use]
    pub fn has_explicit_target(&self) -> bool {
        unsafe { ffi::appkit_menu_item_has_explicit_target(self.as_ptr()) }
    }
}
