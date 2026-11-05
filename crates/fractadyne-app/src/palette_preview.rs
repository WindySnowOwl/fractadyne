//! Hover preview for the palette pickers (Color ▸ Palette and the panel's Palette list).
//!
//! Rest the pointer on a gradient's row and, after a short pause, the view shows that gradient in
//! place of the chosen one. Choose it and it becomes the palette; close the picker any other way
//! (Escape, a click elsewhere) and the view goes back to what was chosen.
//!
//! ⭐**A preview is never a choice.** Nothing here touches the coloring state: the view reads
//! [`PalettePreview::shown`] where it reads the palette (`active_gradient`), so there is nothing to
//! put back on cancel and nothing a session save could pick up by mistake.
//!
//! ⭐**It lives only while a picker says it is open, every frame.** The pickers call
//! [`PalettePreview::picker_open`] from inside their popups, and [`PalettePreview::end_frame`] drops
//! the preview on the first frame none did. So every way a menu can close reverts it, including
//! the ones nobody thought to list: the rule is "shown while open", not "cleared on cancel".

/// How long the pointer must rest on a row before the view switches to it. Short enough that
/// stopping on a row feels immediate; long enough that sweeping the pointer down a list on the
/// way to one row does not flash every gradient it crosses.
pub(crate) const DELAY_S: f64 = 0.3;

/// A row of a palette picker: a built-in preset, or one of the user's saved gradients (both by
/// index, as the pickers list them).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum PaletteRow {
    Preset(usize),
    Saved(usize),
}

/// What [`PalettePreview::end_frame`] asks of the caller.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Settle {
    /// The previewed gradient changed this frame, so the view needs repainting.
    pub changed: bool,
    /// The pointer is resting on a row whose pause has not run out: wake after this many seconds,
    /// or a still pointer would never see its preview (egui repaints only on input).
    pub wake_in: Option<f64>,
}

#[derive(Debug, Default)]
pub(crate) struct PalettePreview {
    /// What the view shows in place of the chosen palette.
    shown: Option<PaletteRow>,
    /// The row the pointer was last seen on (`None`: on no row), and when it got there.
    target: Option<PaletteRow>,
    since: f64,
    /// This frame's reports from the pickers, consumed by `end_frame`.
    open: bool,
    hover: Option<PaletteRow>,
    drawing: Vec<(PaletteRow, egui::Rect)>,
    /// Where each row sat on the last frame: what `--uitest` aims its pointer at.
    drawn: Vec<(PaletteRow, egui::Rect)>,
    /// How many frames running a picker has been open, up to the frame before this one.
    open_frames: u32,
}

impl PalettePreview {
    /// The gradient to show in place of the chosen palette, if any.
    pub(crate) fn shown(&self) -> Option<PaletteRow> {
        self.shown
    }

    /// A picker is open this frame. Call from inside its popup, every frame it draws.
    pub(crate) fn picker_open(&mut self) {
        self.open = true;
    }

    /// A picker is in its first frames open: the time to bring the chosen row into view. Two
    /// frames rather than one, because a popup's first frame can be egui's invisible sizing pass.
    pub(crate) fn opening(&self) -> bool {
        self.open_frames < 2
    }

    /// A list is being opened by request (`Dialogs::open_palette_dropdown`): its opening frames start
    /// again. ⚠Needed because such a request can close a list and open it again in one frame, as
    /// `--uitest` does between two steps, and then no frame ever saw it closed.
    pub(crate) fn reopened(&mut self) {
        self.open_frames = 0;
    }

    /// A picker drew `row` at `rect` this frame, with the pointer on it or not.
    pub(crate) fn row(&mut self, row: PaletteRow, rect: egui::Rect, hovered: bool) {
        self.drawing.push((row, rect));
        if hovered {
            self.hover = Some(row);
        }
    }

    /// Where `row` was drawn on the last frame, if a picker drew it.
    pub(crate) fn row_rect(&self, row: PaletteRow) -> Option<egui::Rect> {
        self.drawn.iter().find(|(r, _)| *r == row).map(|(_, rect)| *rect)
    }

    /// Settle what the view shows, after this frame's pickers have drawn. `now` is the UI clock in
    /// seconds.
    ///
    /// The view follows the row under the pointer once it has rested there for [`DELAY_S`]; off
    /// every row (the pointer on a separator, or on the menu's other items) it goes back to the
    /// chosen palette after the same pause. ⚠Moving from one row to the next keeps the first row's
    /// preview until the second's pause runs out, rather than flashing the chosen palette between
    /// the two.
    pub(crate) fn end_frame(&mut self, now: f64) -> Settle {
        let open = std::mem::take(&mut self.open);
        let hover = self.hover.take();
        self.drawn = std::mem::take(&mut self.drawing);
        if !open {
            let changed = self.shown.is_some();
            *self = Self::default();
            return Settle { changed, wake_in: None };
        }
        self.open_frames = self.open_frames.saturating_add(1);
        if hover != self.target {
            self.target = hover;
            self.since = now;
        }
        if self.shown == self.target {
            return Settle { changed: false, wake_in: None };
        }
        let left = self.since + DELAY_S - now;
        if left > 0.0 {
            return Settle { changed: false, wake_in: Some(left) };
        }
        self.shown = self.target;
        Settle { changed: true, wake_in: None }
    }
}

#[cfg(test)]
#[path = "../tests/unit/palette_preview/tests.rs"]
mod tests;
