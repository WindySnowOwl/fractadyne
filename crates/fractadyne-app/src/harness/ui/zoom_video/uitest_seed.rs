//! `--uitest`'s seed for the Render zoom video dialog: a deep view, normalized, with strips kept,
//! as if a render were part way through — so the walk photographs the summary, the memory and disk
//! lines and the progress bar. It starts no render.
//! Attached to `ui::zoom_video` (`#[path]`): it fills the window's fields.

use super::*;

impl FractadyneApp {
    /// The UI walk's view of this dialog: the e98 star dive at 1080p60, a third of the way through.
    pub(crate) fn uitest_seed_zoom_video(&mut self) {
        let z = &mut self.zoom_video;
        z.centre = ("-5.622026215230372127449695962629619e-1".into(), "6.428171490727752488996246566278309e-1".into());
        z.end_log2 = 331.1;
        z.fractal = "Mandelbrot".into();
        z.unsupported = None;
        // An absolute sample path: a default would show this machine's own folders.
        z.out = "C:/Users/robin/Videos/star-e98-dive.mp4".into();
        (z.width, z.height, z.custom_size, z.fps, z.speed, z.start_log2, z.rotate, z.ss) = (1920, 1080, false, 60.0, 1.0, 0.0, 0.0, 1);
        z.normalize = true;
        z.keep = true;
        z.keep_dir = "C:/Users/robin/Videos/star-e98-strips".into();
        z.progress = "frame 6624/19867  (11m02s elapsed, 22m04s left, 10.0 fps)".into();
        z.progress_frames = Some((6624, 19867));
        z.status = None;
        z.error = None;
        z.open = true;
    }
}
