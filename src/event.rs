use core::ffi::c_void;
use core::marker::PhantomData;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};
use core::ptr::NonNull;

use apple_cf::cg::CGPoint;

use crate::ffi::event as ffi;
use crate::main_thread::MainThreadMarker;
use crate::object::Handle;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventType {
    LeftMouseDown,
    LeftMouseUp,
    RightMouseDown,
    RightMouseUp,
    MouseMoved,
    LeftMouseDragged,
    RightMouseDragged,
    MouseEntered,
    MouseExited,
    KeyDown,
    KeyUp,
    FlagsChanged,
    AppKitDefined,
    SystemDefined,
    ApplicationDefined,
    Periodic,
    CursorUpdate,
    Rotate,
    BeginGesture,
    EndGesture,
    ScrollWheel,
    TabletPoint,
    TabletProximity,
    OtherMouseDown,
    OtherMouseUp,
    OtherMouseDragged,
    Gesture,
    Magnify,
    Swipe,
    SmartMagnify,
    QuickLook,
    Pressure,
    DirectTouch,
    ChangeMode,
    MouseCancelled,
    Other(u64),
}

impl EventType {
    #[must_use]
    pub const fn from_raw(value: u64) -> Self {
        match value {
            1 => Self::LeftMouseDown,
            2 => Self::LeftMouseUp,
            3 => Self::RightMouseDown,
            4 => Self::RightMouseUp,
            5 => Self::MouseMoved,
            6 => Self::LeftMouseDragged,
            7 => Self::RightMouseDragged,
            8 => Self::MouseEntered,
            9 => Self::MouseExited,
            10 => Self::KeyDown,
            11 => Self::KeyUp,
            12 => Self::FlagsChanged,
            13 => Self::AppKitDefined,
            14 => Self::SystemDefined,
            15 => Self::ApplicationDefined,
            16 => Self::Periodic,
            17 => Self::CursorUpdate,
            18 => Self::Rotate,
            19 => Self::BeginGesture,
            20 => Self::EndGesture,
            22 => Self::ScrollWheel,
            23 => Self::TabletPoint,
            24 => Self::TabletProximity,
            25 => Self::OtherMouseDown,
            26 => Self::OtherMouseUp,
            27 => Self::OtherMouseDragged,
            29 => Self::Gesture,
            30 => Self::Magnify,
            31 => Self::Swipe,
            32 => Self::SmartMagnify,
            33 => Self::QuickLook,
            34 => Self::Pressure,
            37 => Self::DirectTouch,
            38 => Self::ChangeMode,
            40 => Self::MouseCancelled,
            _ => Self::Other(value),
        }
    }

    #[must_use]
    pub const fn raw_value(self) -> u64 {
        match self {
            Self::LeftMouseDown => 1,
            Self::LeftMouseUp => 2,
            Self::RightMouseDown => 3,
            Self::RightMouseUp => 4,
            Self::MouseMoved => 5,
            Self::LeftMouseDragged => 6,
            Self::RightMouseDragged => 7,
            Self::MouseEntered => 8,
            Self::MouseExited => 9,
            Self::KeyDown => 10,
            Self::KeyUp => 11,
            Self::FlagsChanged => 12,
            Self::AppKitDefined => 13,
            Self::SystemDefined => 14,
            Self::ApplicationDefined => 15,
            Self::Periodic => 16,
            Self::CursorUpdate => 17,
            Self::Rotate => 18,
            Self::BeginGesture => 19,
            Self::EndGesture => 20,
            Self::ScrollWheel => 22,
            Self::TabletPoint => 23,
            Self::TabletProximity => 24,
            Self::OtherMouseDown => 25,
            Self::OtherMouseUp => 26,
            Self::OtherMouseDragged => 27,
            Self::Gesture => 29,
            Self::Magnify => 30,
            Self::Swipe => 31,
            Self::SmartMagnify => 32,
            Self::QuickLook => 33,
            Self::Pressure => 34,
            Self::DirectTouch => 37,
            Self::ChangeMode => 38,
            Self::MouseCancelled => 40,
            Self::Other(value) => value,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ModifierFlags(u64);

impl ModifierFlags {
    pub const CAPS_LOCK: Self = Self(1 << 16);
    pub const SHIFT: Self = Self(1 << 17);
    pub const CONTROL: Self = Self(1 << 18);
    pub const OPTION: Self = Self(1 << 19);
    pub const COMMAND: Self = Self(1 << 20);
    pub const NUMERIC_PAD: Self = Self(1 << 21);
    pub const HELP: Self = Self(1 << 22);
    pub const FUNCTION: Self = Self(1 << 23);
    pub const DEVICE_INDEPENDENT_FLAGS_MASK: Self = Self(0xffff_0000);

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

impl BitOr for ModifierFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for ModifierFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for ModifierFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for ModifierFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct EventPhase(u64);

impl EventPhase {
    pub const NONE: Self = Self(0);
    pub const BEGAN: Self = Self(1 << 0);
    pub const STATIONARY: Self = Self(1 << 1);
    pub const CHANGED: Self = Self(1 << 2);
    pub const ENDED: Self = Self(1 << 3);
    pub const CANCELLED: Self = Self(1 << 4);
    pub const MAY_BEGIN: Self = Self(1 << 5);

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

impl BitOr for EventPhase {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for EventPhase {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for EventPhase {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for EventPhase {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

pub const NS_UP_ARROW_FUNCTION_KEY: u16 = 0xF700;
pub const NS_DOWN_ARROW_FUNCTION_KEY: u16 = 0xF701;
pub const NS_LEFT_ARROW_FUNCTION_KEY: u16 = 0xF702;
pub const NS_RIGHT_ARROW_FUNCTION_KEY: u16 = 0xF703;
pub const NS_F1_FUNCTION_KEY: u16 = 0xF704;
pub const NS_F2_FUNCTION_KEY: u16 = 0xF705;
pub const NS_F3_FUNCTION_KEY: u16 = 0xF706;
pub const NS_F4_FUNCTION_KEY: u16 = 0xF707;
pub const NS_F5_FUNCTION_KEY: u16 = 0xF708;
pub const NS_F6_FUNCTION_KEY: u16 = 0xF709;
pub const NS_F7_FUNCTION_KEY: u16 = 0xF70A;
pub const NS_F8_FUNCTION_KEY: u16 = 0xF70B;
pub const NS_F9_FUNCTION_KEY: u16 = 0xF70C;
pub const NS_F10_FUNCTION_KEY: u16 = 0xF70D;
pub const NS_F11_FUNCTION_KEY: u16 = 0xF70E;
pub const NS_F12_FUNCTION_KEY: u16 = 0xF70F;
pub const NS_F13_FUNCTION_KEY: u16 = 0xF710;
pub const NS_F14_FUNCTION_KEY: u16 = 0xF711;
pub const NS_F15_FUNCTION_KEY: u16 = 0xF712;
pub const NS_F16_FUNCTION_KEY: u16 = 0xF713;
pub const NS_F17_FUNCTION_KEY: u16 = 0xF714;
pub const NS_F18_FUNCTION_KEY: u16 = 0xF715;
pub const NS_F19_FUNCTION_KEY: u16 = 0xF716;
pub const NS_F20_FUNCTION_KEY: u16 = 0xF717;
pub const NS_F21_FUNCTION_KEY: u16 = 0xF718;
pub const NS_F22_FUNCTION_KEY: u16 = 0xF719;
pub const NS_F23_FUNCTION_KEY: u16 = 0xF71A;
pub const NS_F24_FUNCTION_KEY: u16 = 0xF71B;
pub const NS_F25_FUNCTION_KEY: u16 = 0xF71C;
pub const NS_F26_FUNCTION_KEY: u16 = 0xF71D;
pub const NS_F27_FUNCTION_KEY: u16 = 0xF71E;
pub const NS_F28_FUNCTION_KEY: u16 = 0xF71F;
pub const NS_F29_FUNCTION_KEY: u16 = 0xF720;
pub const NS_F30_FUNCTION_KEY: u16 = 0xF721;
pub const NS_F31_FUNCTION_KEY: u16 = 0xF722;
pub const NS_F32_FUNCTION_KEY: u16 = 0xF723;
pub const NS_F33_FUNCTION_KEY: u16 = 0xF724;
pub const NS_F34_FUNCTION_KEY: u16 = 0xF725;
pub const NS_F35_FUNCTION_KEY: u16 = 0xF726;
pub const NS_INSERT_FUNCTION_KEY: u16 = 0xF727;
pub const NS_DELETE_FUNCTION_KEY: u16 = 0xF728;
pub const NS_HOME_FUNCTION_KEY: u16 = 0xF729;
pub const NS_BEGIN_FUNCTION_KEY: u16 = 0xF72A;
pub const NS_END_FUNCTION_KEY: u16 = 0xF72B;
pub const NS_PAGE_UP_FUNCTION_KEY: u16 = 0xF72C;
pub const NS_PAGE_DOWN_FUNCTION_KEY: u16 = 0xF72D;
pub const NS_PRINT_SCREEN_FUNCTION_KEY: u16 = 0xF72E;
pub const NS_SCROLL_LOCK_FUNCTION_KEY: u16 = 0xF72F;
pub const NS_PAUSE_FUNCTION_KEY: u16 = 0xF730;
pub const NS_SYS_REQ_FUNCTION_KEY: u16 = 0xF731;
pub const NS_BREAK_FUNCTION_KEY: u16 = 0xF732;
pub const NS_RESET_FUNCTION_KEY: u16 = 0xF733;
pub const NS_STOP_FUNCTION_KEY: u16 = 0xF734;
pub const NS_MENU_FUNCTION_KEY: u16 = 0xF735;
pub const NS_USER_FUNCTION_KEY: u16 = 0xF736;
pub const NS_SYSTEM_FUNCTION_KEY: u16 = 0xF737;
pub const NS_PRINT_FUNCTION_KEY: u16 = 0xF738;
pub const NS_CLEAR_LINE_FUNCTION_KEY: u16 = 0xF739;
pub const NS_CLEAR_DISPLAY_FUNCTION_KEY: u16 = 0xF73A;
pub const NS_INSERT_LINE_FUNCTION_KEY: u16 = 0xF73B;
pub const NS_DELETE_LINE_FUNCTION_KEY: u16 = 0xF73C;
pub const NS_INSERT_CHAR_FUNCTION_KEY: u16 = 0xF73D;
pub const NS_DELETE_CHAR_FUNCTION_KEY: u16 = 0xF73E;
pub const NS_PREV_FUNCTION_KEY: u16 = 0xF73F;
pub const NS_NEXT_FUNCTION_KEY: u16 = 0xF740;
pub const NS_SELECT_FUNCTION_KEY: u16 = 0xF741;
pub const NS_EXECUTE_FUNCTION_KEY: u16 = 0xF742;
pub const NS_UNDO_FUNCTION_KEY: u16 = 0xF743;
pub const NS_REDO_FUNCTION_KEY: u16 = 0xF744;
pub const NS_FIND_FUNCTION_KEY: u16 = 0xF745;
pub const NS_HELP_FUNCTION_KEY: u16 = 0xF746;
pub const NS_MODE_SWITCH_FUNCTION_KEY: u16 = 0xF747;

#[derive(Clone, Copy, Debug)]
pub struct EventRef<'event> {
    raw: NonNull<c_void>,
    main_thread: MainThreadMarker,
    borrowed: PhantomData<&'event ()>,
}

impl EventRef<'_> {
    #[must_use]
    pub const unsafe fn from_raw_borrowed(
        event: NonNull<c_void>,
        main_thread: MainThreadMarker,
    ) -> Self {
        Self {
            raw: event,
            main_thread,
            borrowed: PhantomData,
        }
    }

    #[must_use]
    pub const fn as_ptr(self) -> *mut c_void {
        self.raw.as_ptr()
    }

    #[must_use]
    pub fn event_type(self) -> EventType {
        EventType::from_raw(unsafe { ffi::appkit_event_type(self.as_ptr()) })
    }

    #[must_use]
    pub fn modifier_flags(self) -> ModifierFlags {
        ModifierFlags::from_bits_retain(unsafe { ffi::appkit_event_modifier_flags(self.as_ptr()) })
    }

    #[must_use]
    pub fn location_in_window(self) -> Option<CGPoint> {
        self.read(CGPoint::new(0.0, 0.0), |event, out| unsafe {
            ffi::appkit_event_location_in_window(event, out)
        })
    }

    #[must_use]
    pub fn button_number(self) -> Option<i64> {
        self.read(0, |event, out| unsafe {
            ffi::appkit_event_button_number(event, out)
        })
    }

    #[must_use]
    pub fn click_count(self) -> Option<i64> {
        self.read(0, |event, out| unsafe {
            ffi::appkit_event_click_count(event, out)
        })
    }

    #[must_use]
    pub fn is_repeat(self) -> Option<bool> {
        self.read(false, |event, out| unsafe {
            ffi::appkit_event_is_repeat(event, out)
        })
    }

    #[must_use]
    pub fn key_code(self) -> Option<u16> {
        self.read(0, |event, out| unsafe {
            ffi::appkit_event_key_code(event, out)
        })
    }

    #[must_use]
    pub fn characters(self) -> Option<String> {
        let mut string = ffi::AppKitEventString {
            bytes: core::ptr::null_mut(),
            length: 0,
        };
        if !unsafe { ffi::appkit_event_characters(self.as_ptr(), false, &raw mut string) } {
            return None;
        }
        Some(unsafe { Self::take_string(&string) })
    }

    #[must_use]
    pub fn characters_ignoring_modifiers(self) -> Option<String> {
        let mut string = ffi::AppKitEventString {
            bytes: core::ptr::null_mut(),
            length: 0,
        };
        if !unsafe { ffi::appkit_event_characters(self.as_ptr(), true, &raw mut string) } {
            return None;
        }
        Some(unsafe { Self::take_string(&string) })
    }

    #[must_use]
    pub fn scrolling_delta(self) -> Option<CGPoint> {
        self.read(CGPoint::new(0.0, 0.0), |event, out| unsafe {
            ffi::appkit_event_scrolling_delta(event, out)
        })
    }

    #[must_use]
    pub fn has_precise_scrolling_deltas(self) -> Option<bool> {
        self.read(false, |event, out| unsafe {
            ffi::appkit_event_has_precise_scrolling_deltas(event, out)
        })
    }

    #[must_use]
    pub fn phase(self) -> Option<EventPhase> {
        self.read(0, |event, out| unsafe {
            ffi::appkit_event_phase(event, out)
        })
        .map(EventPhase::from_bits_retain)
    }

    #[must_use]
    pub fn momentum_phase(self) -> Option<EventPhase> {
        self.read(0, |event, out| unsafe {
            ffi::appkit_event_momentum_phase(event, out)
        })
        .map(EventPhase::from_bits_retain)
    }

    #[must_use]
    pub fn pressure(self) -> Option<f32> {
        self.read(0.0, |event, out| unsafe {
            ffi::appkit_event_pressure(event, out)
        })
    }

    #[must_use]
    pub fn stage(self) -> Option<i64> {
        self.read(0, |event, out| unsafe {
            ffi::appkit_event_stage(event, out)
        })
    }

    #[must_use]
    pub fn magnification(self) -> Option<f64> {
        self.read(0.0, |event, out| unsafe {
            ffi::appkit_event_magnification(event, out)
        })
    }

    #[must_use]
    pub fn swipe_delta(self) -> Option<CGPoint> {
        self.read(CGPoint::new(0.0, 0.0), |event, out| unsafe {
            ffi::appkit_event_swipe_delta(event, out)
        })
    }

    #[must_use]
    pub fn timestamp(self) -> f64 {
        unsafe { ffi::appkit_event_timestamp(self.as_ptr()) }
    }

    #[must_use]
    pub fn window_number(self) -> i64 {
        unsafe { ffi::appkit_event_window_number(self.as_ptr()) }
    }

    #[must_use]
    pub fn retain(self) -> Event {
        Event {
            handle: unsafe { Handle::retain(self.raw) },
            main_thread: self.main_thread,
        }
    }

    fn read<T: Copy>(
        self,
        initial: T,
        getter: impl FnOnce(*mut c_void, *mut T) -> bool,
    ) -> Option<T> {
        let mut value = initial;
        getter(self.as_ptr(), &raw mut value).then_some(value)
    }

    unsafe fn take_string(string: &ffi::AppKitEventString) -> String {
        let bytes = unsafe { core::slice::from_raw_parts(string.bytes, string.length) };
        let value = String::from_utf8_lossy(bytes).into_owned();
        unsafe { ffi::appkit_event_string_free(string.bytes) };
        value
    }
}

#[derive(Clone, Debug)]
pub struct Event {
    handle: Handle,
    main_thread: MainThreadMarker,
}

impl Event {
    #[must_use]
    pub fn current(main_thread: MainThreadMarker) -> Option<Self> {
        unsafe { Handle::from_retained(ffi::appkit_event_current()) }.map(|handle| Self {
            handle,
            main_thread,
        })
    }

    #[must_use]
    pub fn pressed_mouse_buttons(_main_thread: MainThreadMarker) -> u64 {
        unsafe { ffi::appkit_event_pressed_mouse_buttons() }
    }

    #[must_use]
    pub fn modifier_flags(_main_thread: MainThreadMarker) -> ModifierFlags {
        ModifierFlags::from_bits_retain(unsafe { ffi::appkit_event_class_modifier_flags() })
    }

    #[must_use]
    pub fn synthetic_key(
        main_thread: MainThreadMarker,
        event_type: EventType,
        modifiers: ModifierFlags,
        key_code: u16,
        characters: &str,
        characters_ignoring_modifiers: &str,
        is_repeat: bool,
    ) -> Option<Self> {
        if !matches!(event_type, EventType::KeyDown | EventType::KeyUp) {
            return None;
        }
        let event = unsafe {
            ffi::appkit_event_synthetic_key(
                event_type.raw_value(),
                modifiers.bits(),
                key_code,
                characters.as_ptr(),
                characters.len(),
                characters_ignoring_modifiers.as_ptr(),
                characters_ignoring_modifiers.len(),
                is_repeat,
            )
        };
        unsafe { Handle::from_retained(event) }.map(|handle| Self {
            handle,
            main_thread,
        })
    }

    #[must_use]
    pub fn synthetic_mouse(
        main_thread: MainThreadMarker,
        event_type: EventType,
        location: CGPoint,
        modifiers: ModifierFlags,
        click_count: i64,
        pressure: f32,
    ) -> Option<Self> {
        if !matches!(
            event_type,
            EventType::LeftMouseDown
                | EventType::LeftMouseUp
                | EventType::RightMouseDown
                | EventType::RightMouseUp
                | EventType::MouseMoved
                | EventType::LeftMouseDragged
                | EventType::RightMouseDragged
                | EventType::OtherMouseDown
                | EventType::OtherMouseUp
                | EventType::OtherMouseDragged
        ) {
            return None;
        }
        let event = unsafe {
            ffi::appkit_event_synthetic_mouse(
                event_type.raw_value(),
                &raw const location,
                modifiers.bits(),
                click_count,
                pressure,
            )
        };
        unsafe { Handle::from_retained(event) }.map(|handle| Self {
            handle,
            main_thread,
        })
    }

    #[must_use]
    pub unsafe fn from_raw_retained(event: NonNull<c_void>, main_thread: MainThreadMarker) -> Self {
        Self {
            handle: unsafe { Handle::from_retained(event.as_ptr()) }
                .expect("the event pointer is not null"),
            main_thread,
        }
    }

    #[must_use]
    pub fn as_ref(&self) -> EventRef<'_> {
        EventRef {
            raw: NonNull::new(self.handle.as_ptr()).expect("an event handle is not null"),
            main_thread: self.main_thread,
            borrowed: PhantomData,
        }
    }

    #[must_use]
    pub const fn as_ptr(&self) -> *mut c_void {
        self.handle.as_ptr()
    }
}

#[cfg(test)]
mod tests {
    use super::{EventPhase, EventType, ModifierFlags, NS_F35_FUNCTION_KEY};

    #[test]
    fn event_types_round_trip_unknown_values() {
        for raw in 0..=41 {
            assert_eq!(EventType::from_raw(raw).raw_value(), raw);
        }
        assert_eq!(EventType::from_raw(u64::MAX), EventType::Other(u64::MAX));
        assert_eq!(EventType::from_raw(21), EventType::Other(21));
        assert_eq!(EventType::from_raw(35), EventType::Other(35));
    }

    #[test]
    fn modifier_flags_preserve_device_bits() {
        let flags = ModifierFlags::from_bits_retain(0x43 | ModifierFlags::COMMAND.bits());
        assert!(flags.contains(ModifierFlags::COMMAND));
        assert!(!flags.contains(ModifierFlags::SHIFT));
        assert_eq!(
            (flags & ModifierFlags::DEVICE_INDEPENDENT_FLAGS_MASK).bits(),
            1 << 20
        );
        assert_eq!(
            (ModifierFlags::CONTROL | ModifierFlags::OPTION).bits(),
            (1 << 18) | (1 << 19)
        );
    }

    #[test]
    fn phases_and_function_keys_match_appkit() {
        assert_eq!(EventPhase::MAY_BEGIN.bits(), 1 << 5);
        assert_eq!(EventPhase::ENDED.bits(), 1 << 3);
        assert!(EventPhase::from_bits_retain(8).contains(EventPhase::ENDED));
        assert_eq!(NS_F35_FUNCTION_KEY, 0xF726);
    }
}
