//! `--uitest`'s seed for the tour-render dialog: a tour with chapters, as if opened from its
//! player.
//! Attached to `ui::tour_render` (`#[path]`): it fills the window's private fields. Moved
//! from `ui/tour_render.rs`.

use super::*;

impl FractadyneApp {
    /// The UI walk's view of this dialog: a tour with chapters, as if opened from its player —
    /// the walk plays no tour, and reading the player made this step photograph an empty screen.
    pub(crate) fn uitest_seed_tour_render(&mut self) {
        let t = &mut self.tour_render;
        t.script = Some(std::path::PathBuf::from("tours/grand-tour.toml"));
        t.tour_name = "Grand tour".into();
        t.total = 331.0;
        t.chapters = vec![("Seahorse valley".into(), 0.0, 96.0), ("Into the spiral".into(), 96.0, 214.0), ("The deep end".into(), 214.0, 331.0)];
        (t.width, t.height, t.custom_size, t.fps, t.ss, t.segment) = (1920, 1080, false, 30.0, 2, 0);
        t.prefix = "grand-tour".into();
        // An absolute sample path: the default "frames" resolves against the working directory,
        // and the walk's screenshot then showed this machine's home folder.
        t.out = "C:/Users/robin/Videos/grand-tour-frames".into();
        t.progress.clear();
        t.progress_frames = None;
        t.status = None;
        // A two-card machine, so the walk photographs "Use all graphics cards" whatever runs it.
        t.cards = Some(2);
        t.cards_rx = None;
        t.open = true;
    }
}
