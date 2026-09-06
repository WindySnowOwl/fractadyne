//! A screen-wide colour picker: sample the pixel under the OS cursor, anywhere on the desktop.
//!
//! ⭐**Why this reaches outside our own window.** The colours people want in a fractal palette come
//! from somewhere else — a photograph, another fractal program, a palette on a web page. Restricting
//! the dropper to our window would make it a novelty; the whole value is picking the orange out of
//! a sunset that is open in another application.
//!
//! ⚠⚠**Windows only, and it says so rather than pretending.** The Win32 arm is raw FFI, matching
//! `sysinfo.rs`. X11 would need a real dependency and Wayland forbids reading other surfaces
//! outright (it needs a portal and a user consent dialog), so on any other platform
//! [`supported`] returns `false` and the button is disabled with a reason. A silently dead button
//! would be worse than an absent one.
//!
//! ⚠The polling is deliberately over `GetAsyncKeyState` rather than egui events: while picking, the
//! pointer is usually over some other application's window and our process receives no input at all.

/// Unpack a Win32 `COLORREF` into a display-referred RGB triple, or `None` for `CLR_INVALID`.
///
/// ⭐⭐**A `COLORREF` is `0x00bbggrr` — BLUE in the high byte.** Reading it as RGB swaps red and
/// blue and still produces a plausible-looking colour, which is exactly how the `.ugr` importer's
/// byte order went unnoticed until it was checked at the pixel. That is why this is a separate
/// function with a test naming the two channels, rather than three shifts inlined at the call site.
///
/// ⚠`GetPixel` reports failure as `CLR_INVALID` (`0xffffffff`), which is otherwise a legal-looking
/// colour value — white with the top byte set. Treating it as a colour would silently paint a stop
/// white whenever the sample failed, so it is rejected here rather than at the caller.
pub(crate) fn colorref_rgb(c: u32) -> Option<[f32; 3]> {
    const CLR_INVALID: u32 = 0xffff_ffff;
    if c == CLR_INVALID {
        return None;
    }
    let ch = |shift: u32| f32::from(((c >> shift) & 0xff) as u8) / 255.0;
    Some([ch(0), ch(8), ch(16)])
}

/// Can this platform sample an arbitrary screen pixel?
pub(crate) fn supported() -> bool {
    cfg!(windows)
}

/// Why not, for the disabled button's tooltip. `None` when it is supported.
pub(crate) fn unsupported_reason() -> Option<&'static str> {
    (!supported()).then_some(
        "Picking a colour from elsewhere on screen needs a platform screen-capture API. \
         Only the Windows build has one; on Linux this would need an X11 dependency, and \
         Wayland requires a portal with its own permission prompt.",
    )
}

/// The colour of the screen pixel under the OS cursor right now.
#[cfg(windows)]
pub(crate) fn sample_under_cursor() -> Option<[f32; 3]> {
    #[repr(C)]
    struct Point {
        x: i32,
        y: i32,
    }
    extern "system" {
        fn GetCursorPos(p: *mut Point) -> i32;
        fn GetDC(hwnd: isize) -> isize;
        fn ReleaseDC(hwnd: isize, hdc: isize) -> i32;
        fn GetPixel(hdc: isize, x: i32, y: i32) -> u32;
    }
    // SAFETY: `Point` is `#[repr(C)]` matching POINT and is fully written by `GetCursorPos` before
    // it is read (we bail if the call reports failure). `GetDC(0)` asks for the SCREEN device
    // context; it is released on every path out of this function, including the failure ones, so
    // repeated polling cannot leak DCs. `GetPixel` takes the DC by value and returns a plain u32.
    unsafe {
        let mut p = Point { x: 0, y: 0 };
        if GetCursorPos(&mut p) == 0 {
            return None;
        }
        let hdc = GetDC(0);
        if hdc == 0 {
            return None;
        }
        let c = GetPixel(hdc, p.x, p.y);
        ReleaseDC(0, hdc);
        colorref_rgb(c)
    }
}

#[cfg(not(windows))]
pub(crate) fn sample_under_cursor() -> Option<[f32; 3]> {
    None
}

/// Is a virtual key held down right now, asked of the OS rather than of our own event queue?
///
/// ⚠**This is the only way to see the click that ends the pick.** While picking, the pointer is
/// over another application's window, so the press is delivered to that application and our
/// process never learns about it through egui.
#[cfg(windows)]
fn key_down(vk: i32) -> bool {
    extern "system" {
        fn GetAsyncKeyState(v: i32) -> i16;
    }
    // SAFETY: `GetAsyncKeyState` takes an integer and returns one; there are no pointers and no
    // resources involved. The high bit means "currently down"; the low bit ("pressed since the
    // last call") is deliberately ignored, because it is consumed by whoever reads it first and
    // this is polled from a render loop that may run alongside other readers.
    unsafe { (GetAsyncKeyState(vk) as u16 & 0x8000) != 0 }
}

#[cfg(not(windows))]
fn key_down(_vk: i32) -> bool {
    false
}

/// The primary mouse button, honouring a left-handed swap — `GetAsyncKeyState(VK_LBUTTON)` is
/// documented to track the PHYSICAL left button, so the logical primary is what we must ask for.
pub(crate) fn primary_button_down() -> bool {
    const VK_LBUTTON: i32 = 0x01;
    const VK_RBUTTON: i32 = 0x02;
    key_down(if swapped_buttons() { VK_RBUTTON } else { VK_LBUTTON })
}

pub(crate) fn escape_down() -> bool {
    const VK_ESCAPE: i32 = 0x1b;
    key_down(VK_ESCAPE)
}

#[cfg(windows)]
fn swapped_buttons() -> bool {
    extern "system" {
        fn GetSystemMetrics(index: i32) -> i32;
    }
    const SM_SWAPBUTTON: i32 = 23;
    // SAFETY: an integer in, an integer out; no pointers, no resources.
    unsafe { GetSystemMetrics(SM_SWAPBUTTON) != 0 }
}

#[cfg(not(windows))]
fn swapped_buttons() -> bool {
    false
}

/// One in-progress pick.
///
/// ⭐⭐**`armed` is the whole reason this is a state machine and not a boolean.** The click that
/// STARTS the pick is still physically down when the first poll runs, so a naive "commit when the
/// button is down" reads that same press as the pick and the dropper grabs the colour of its own
/// button. The gesture only becomes live once the button has been seen RELEASED.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Pick {
    /// The starting press has been released, so a new press means "take this one".
    pub armed: bool,
    /// The colour under the cursor as of the last poll, for the live preview.
    pub preview: Option<[f32; 3]>,
}

/// What a poll decided.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum PickStep {
    /// Still picking; `preview` on the returned state is the live colour.
    Continue(Pick),
    /// The user clicked: take this colour.
    Take([f32; 3]),
    /// Escape, or an unsampleable screen.
    Cancel,
}

/// Advance a pick by one frame, given what the OS reports.
///
/// Split from the FFI so the state machine — which is where the bugs are — is testable without a
/// desktop: `down`, `esc` and `sample` are exactly what the platform layer provides.
pub(crate) fn step(pick: Pick, down: bool, esc: bool, sample: Option<[f32; 3]>) -> PickStep {
    if esc {
        return PickStep::Cancel;
    }
    if !pick.armed {
        // Wait for the starting click to end before the next press can mean anything.
        return PickStep::Continue(Pick { armed: !down, preview: sample });
    }
    match (down, sample) {
        // ⚠Take the sample from THIS poll, not the previous one: between the last frame and the
        // click the pointer may have moved, and the colour the user is looking at is the one under
        // the cursor now.
        (true, Some(c)) => PickStep::Take(c),
        // A click we could not sample must not silently take the stale preview — that would paint
        // a stop with whatever the cursor last passed over.
        (true, None) => PickStep::Cancel,
        (false, s) => PickStep::Continue(Pick { armed: true, preview: s.or(pick.preview) }),
    }
}

#[cfg(test)]
// ⚠**`#[cfg(test)]` was lost when the `#[path]` attribute was added**, so this file was
// compiling into the RELEASE binary — caught by "unused import" warnings that could only
// appear if a test-only module was being built for real.
#[cfg(test)]
#[path = "eyedropper_tests.rs"]
mod eyedropper_tests;
