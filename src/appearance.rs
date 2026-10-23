#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppearanceName {
    Aqua,
    DarkAqua,
    VibrantLight,
    VibrantDark,
    AccessibilityHighContrastAqua,
    AccessibilityHighContrastDarkAqua,
    AccessibilityHighContrastVibrantLight,
    AccessibilityHighContrastVibrantDark,
}

const NAMES: [AppearanceName; 8] = [
    AppearanceName::Aqua,
    AppearanceName::DarkAqua,
    AppearanceName::VibrantLight,
    AppearanceName::VibrantDark,
    AppearanceName::AccessibilityHighContrastAqua,
    AppearanceName::AccessibilityHighContrastDarkAqua,
    AppearanceName::AccessibilityHighContrastVibrantLight,
    AppearanceName::AccessibilityHighContrastVibrantDark,
];

impl AppearanceName {
    pub(crate) const fn raw(self) -> i32 {
        self as i32
    }

    pub(crate) fn from_raw(raw: i32) -> Option<Self> {
        usize::try_from(raw).ok().and_then(|index| NAMES.get(index).copied())
    }
}
