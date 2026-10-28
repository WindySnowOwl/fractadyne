//! Frames piped straight into ffmpeg, encoded as they arrive (`design/zoom-video.md` §6.4): an
//! assembled zoom video never exists as a folder of PNGs.
//!
//! The tour export writes PNGs and runs ffmpeg once at the end ([`crate::scripting::assemble_mp4`]),
//! which suits a render whose frames are the expensive, resumable part. Assembling frames from
//! strips is cheap and fast, so the encoder is the bound and a PNG round trip would double the
//! work: here each frame goes to ffmpeg's stdin as raw RGBA, through a writer thread and a short
//! queue, so the GPU assembles the next frame while ffmpeg encodes this one.
//!
//! Frames are the colour pass's display (sRGB) values, quantized with the same ordered dither as
//! every 8-bit deliverable ([`fractadyne_export::to_srgb8_dithered`]): a video frame holds the
//! bytes the PNG of the same frame would. The video is written beside its final name and renamed
//! into place only when ffmpeg has finished cleanly, so a file under the final name is complete.

use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread::JoinHandle;

/// Frames waiting for the encoder: enough to ride out a slow frame either side, little enough
/// that a 4K queue stays well under a gigabyte (each frame is `w·h·16` bytes before quantizing).
const QUEUE: usize = 3;

/// ffmpeg's container for an output file name (`-f`), from its extension; mp4 when unknown.
pub(crate) fn container_for(out: &Path) -> &'static str {
    match out.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()).as_deref() {
        Some("mkv") => "matroska",
        Some("mov") => "mov",
        _ => "mp4",
    }
}

/// Where the video is written while it is being encoded: beside `out`, renamed into place when
/// ffmpeg has finished. The container is given with `-f`, so the name's extension does not matter.
pub(crate) fn partial_video_path(out: &Path) -> PathBuf {
    let mut name = out.file_name().map(|n| n.to_os_string()).unwrap_or_else(|| OsString::from("video"));
    name.push(".partial");
    out.with_file_name(name)
}

/// ffmpeg's arguments for raw RGBA frames of `width × height` at `fps` on stdin, H.264 at `crf`
/// into `file` (as the tour export encodes: odd sizes padded to even, `yuv420p`).
pub(crate) fn ffmpeg_args(width: u32, height: u32, fps: f64, crf: u32, container: &str, file: &Path) -> Vec<OsString> {
    let mut a: Vec<OsString> = [
        "-y",
        "-hide_banner",
        "-loglevel",
        "error",
        "-f",
        "rawvideo",
        "-pix_fmt",
        "rgba",
        "-s",
        &format!("{width}x{height}"),
        "-framerate",
        &format!("{fps}"),
        "-i",
        "-",
        "-vf",
        "pad=ceil(iw/2)*2:ceil(ih/2)*2",
        "-c:v",
        "libx264",
        "-pix_fmt",
        "yuv420p",
        "-crf",
        &format!("{crf}"),
    ]
    .iter()
    .map(OsString::from)
    .collect();
    if container == "mp4" || container == "mov" {
        // The index at the front, so a player can start before the whole file has arrived.
        a.extend(["-movflags", "+faststart"].map(OsString::from));
    }
    a.extend(["-f", container].map(OsString::from));
    a.push(file.as_os_str().to_os_string());
    a
}

/// An ffmpeg process taking frames on its stdin.
pub(crate) struct VideoPipe {
    child: Child,
    tx: Option<SyncSender<Vec<f32>>>,
    writer: Option<JoinHandle<Result<u64, String>>>,
    stderr: Option<JoinHandle<String>>,
    partial: PathBuf,
    out: PathBuf,
    /// The ffmpeg that runs, for messages: a surprising binary should be visible.
    pub(crate) ffmpeg: PathBuf,
}

impl VideoPipe {
    /// Start ffmpeg for frames of `width × height` at `fps`, writing `out`.
    pub(crate) fn start(out: &Path, width: u32, height: u32, fps: f64, crf: u32) -> Result<Self, String> {
        let ffmpeg = crate::exec_resolve::external("ffmpeg")
            .ok_or("could not find ffmpeg (set FRACTADYNE_FFMPEG or add it to PATH)")?;
        let partial = partial_video_path(out);
        let mut child = Command::new(&ffmpeg)
            .args(ffmpeg_args(width, height, fps, crf, container_for(out), &partial))
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("could not run {}: {e}", ffmpeg.display()))?;
        let mut stdin = child.stdin.take().ok_or("ffmpeg's stdin was not piped")?;
        let mut err = child.stderr.take().ok_or("ffmpeg's stderr was not piped")?;
        // ffmpeg's messages are drained as they come (a full stderr pipe would stall it) and kept
        // for the error, if there is one.
        let stderr = std::thread::Builder::new()
            .name("ffmpeg-stderr".into())
            .spawn(move || {
                let mut s = String::new();
                let _ = err.read_to_string(&mut s);
                s
            })
            .map_err(|e| format!("could not start the encoder's reader: {e}"))?;
        let (tx, rx) = sync_channel::<Vec<f32>>(QUEUE);
        let frame_len = width as usize * height as usize * 4;
        let writer = std::thread::Builder::new()
            .name("ffmpeg-writer".into())
            .spawn(move || -> Result<u64, String> {
                let mut n = 0u64;
                for frame in rx {
                    if frame.len() != frame_len {
                        return Err(format!("frame {n} has {} values, not {frame_len}", frame.len()));
                    }
                    let bytes = fractadyne_export::to_srgb8_dithered(&frame, width);
                    stdin.write_all(&bytes).map_err(|e| format!("ffmpeg stopped taking frames at frame {n}: {e}"))?;
                    n += 1;
                }
                // Dropping stdin closes it: ffmpeg finishes the file.
                Ok(n)
            })
            .map_err(|e| format!("could not start the encoder's writer: {e}"))?;
        Ok(VideoPipe { child, tx: Some(tx), writer: Some(writer), stderr: Some(stderr), partial, out: out.to_path_buf(), ffmpeg })
    }

    /// Queue one frame: `width × height` RGBA display values, row-major. Blocks while the queue is
    /// full. An error means ffmpeg has stopped; [`VideoPipe::finish`] says why.
    pub(crate) fn push(&self, frame: Vec<f32>) -> Result<(), String> {
        let tx = self.tx.as_ref().ok_or("the video is already finished")?;
        tx.send(frame).map_err(|_| "the encoder stopped taking frames".to_string())
    }

    /// Close the input, wait for ffmpeg, and move the video into place: the frames written. On
    /// failure the partial file is removed and the error carries ffmpeg's last words.
    pub(crate) fn finish(mut self) -> Result<u64, String> {
        self.tx = None;
        let written = self.writer.take().map(|w| w.join().unwrap_or_else(|_| Err("the encoder's writer panicked".into())));
        let result = (|| {
            let status = self.child.wait().map_err(|e| format!("ffmpeg: {e}"))?;
            let messages = self.stderr.take().and_then(|h| h.join().ok()).unwrap_or_default();
            let tail = tail_lines(&messages, 12);
            let n = match written {
                Some(Ok(n)) => n,
                Some(Err(e)) => return Err(format!("{e}{tail}")),
                None => 0,
            };
            if !status.success() {
                return Err(format!("ffmpeg exited with {status} after {n} frames{tail}"));
            }
            std::fs::rename(&self.partial, &self.out)
                .map_err(|e| format!("cannot move {} to {}: {e}", self.partial.display(), self.out.display()))?;
            Ok(n)
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&self.partial);
        }
        result
    }
}

impl Drop for VideoPipe {
    /// Abandoned (an error before [`VideoPipe::finish`]): stop ffmpeg and remove its partial file,
    /// so neither a process nor a broken video outlives the run.
    fn drop(&mut self) {
        if self.writer.is_none() {
            return; // finished
        }
        self.tx = None;
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(w) = self.writer.take() {
            let _ = w.join();
        }
        if let Some(h) = self.stderr.take() {
            let _ = h.join();
        }
        let _ = std::fs::remove_file(&self.partial);
    }
}

/// The last `n` non-empty lines of ffmpeg's messages, as a suffix for an error ("" when none).
fn tail_lines(s: &str, n: usize) -> String {
    let lines: Vec<&str> = s.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return String::new();
    }
    format!("; ffmpeg said:\n  {}", lines[lines.len().saturating_sub(n)..].join("\n  "))
}

#[cfg(test)]
#[path = "../tests/unit/video_pipe/tests.rs"]
mod tests;
