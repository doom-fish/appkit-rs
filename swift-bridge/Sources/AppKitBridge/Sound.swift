import AppKit

@_cdecl("appkit_sound_beep")
public func appkit_sound_beep() {
    guard Thread.isMainThread else { return }
    NSSound.beep()
}
