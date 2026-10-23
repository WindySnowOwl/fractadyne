//! `--uitest`'s text-layout check (design/multilingual.md §10): every piece of text a frame painted,
//! read back from egui's paint lists, and judged for the faults a translation of another length
//! causes — text cut off by the edge of its container, text running off the screen, two texts drawn
//! over each other, and text shortened to "…".
//!
//! It reads what was PAINTED, not what a layout meant to do, so it sees every widget, window and
//! painter-drawn legend alike. Run the walk in the pseudo-language (`--language en-XA`: every message
//! 40% longer and in brackets) and the check knows which text is the interface's: a fault on a
//! bracketed text is a translation that will not fit. Text without brackets in that run is text
//! the English file does not hold yet; the walk lists it beside the screenshots.

use egui::{LayerId, Rect, Shape};

/// One text the frame painted.
#[derive(Clone, Debug)]
pub(crate) struct TextItem {
    pub(crate) layer: LayerId,
    /// The rectangle it was clipped to (its container's).
    pub(crate) clip: Rect,
    /// Where its glyphs are.
    pub(crate) rect: Rect,
    pub(crate) text: String,
    /// Shortened to fit, with "…".
    pub(crate) elided: bool,
}

/// What can be wrong with a text's place on screen.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Fault {
    /// Cut off at its container's left or right edge (by this many points).
    Clipped(f32),
    /// Running past the screen's edge (by this many points).
    OffScreen(f32),
    /// Drawn over another text (this one).
    Overlap(String),
    /// Shortened to "…" to fit.
    Elided,
}

#[derive(Clone, Debug)]
pub(crate) struct Finding {
    pub(crate) text: String,
    pub(crate) fault: Fault,
}

/// Tolerance in points: anti-aliasing and rounding put glyph edges a fraction past a rectangle.
const SLACK: f32 = 1.0;

/// Every visible text in the frame being built, back to front. Call after the frame's UI is drawn.
pub(crate) fn collect(ctx: &egui::Context) -> Vec<TextItem> {
    let mut layers: Vec<LayerId> = ctx.memory(|m| m.layer_ids().collect());
    if !layers.contains(&LayerId::background()) {
        layers.insert(0, LayerId::background());
    }
    let mut out = Vec::new();
    ctx.graphics(|g| {
        for layer in &layers {
            if let Some(list) = g.get(*layer) {
                for cs in list.all_entries() {
                    walk(*layer, cs.clip_rect, &cs.shape, &mut out);
                }
            }
        }
    });
    out
}

fn walk(layer: LayerId, clip: Rect, shape: &Shape, out: &mut Vec<TextItem>) {
    match shape {
        Shape::Vec(v) => {
            for s in v {
                walk(layer, clip, s, out);
            }
        }
        Shape::Text(t) => {
            let text = t.galley.job.text.clone();
            if text.trim().is_empty() || !visible(t) {
                return;
            }
            out.push(TextItem { layer, clip, rect: t.visual_bounding_rect(), text, elided: t.galley.elided });
        }
        _ => {}
    }
}

/// Whether any of the text can be seen: a slot reserved by painting its widest text transparent
/// (the status bar's) is not text on screen.
fn visible(t: &egui::epaint::TextShape) -> bool {
    if t.opacity_factor <= 0.0 {
        return false;
    }
    if let Some(c) = t.override_text_color {
        return c.a() > 0;
    }
    let fallback = t.fallback_color.a() > 0;
    t.galley.job.sections.iter().any(|s| {
        if s.format.color == egui::Color32::PLACEHOLDER {
            fallback
        } else {
            s.format.color.a() > 0
        }
    })
}

/// Whether `text` is the pseudo-language's (bracketed, accented): the interface's own text.
pub(crate) fn is_pseudo(text: &str) -> bool {
    text.contains('[') && text.chars().any(|c| "åçéĝîñöšüýžÅÇÉÎÑÖŠÜŽ~".contains(c))
}

/// Whether `text` reads as words. A text with no letter in it is data or notation — a number in a
/// field, a masked key, the pieces of a typeset formula — which a field cuts and a formula stacks on
/// purpose, whatever the language.
fn has_words(text: &str) -> bool {
    text.chars().any(char::is_alphabetic)
}

/// The faults of `items` on a screen `screen`.
pub(crate) fn analyse(screen: Rect, items: &[TextItem]) -> Vec<Finding> {
    let mut out = Vec::new();
    for (i, t) in items.iter().enumerate() {
        if !has_words(&t.text) {
            continue;
        }
        let seen = t.rect.intersect(t.clip);
        // Wholly outside its clip (scrolled out of view, a collapsed section): not on screen at all.
        if seen.width() <= 0.0 || seen.height() <= 0.0 {
            continue;
        }
        // Only the sides: a list scrolled part-way cuts its top and bottom rows by design.
        let cut = (t.clip.left() - t.rect.left()).max(t.rect.right() - t.clip.right());
        if cut > SLACK {
            out.push(Finding { text: t.text.clone(), fault: Fault::Clipped(cut) });
        }
        let off = (screen.left() - seen.left()).max(seen.right() - screen.right());
        if off > SLACK {
            out.push(Finding { text: t.text.clone(), fault: Fault::OffScreen(off) });
        }
        if t.elided {
            out.push(Finding { text: t.text.clone(), fault: Fault::Elided });
        }
        for u in &items[i + 1..] {
            // Text over text in one layer. A text drawn twice at an offset (an engraved legend's
            // shadow and highlight) is one text, and a popup or tooltip is another layer.
            if u.layer != t.layer || u.text == t.text || !has_words(&u.text) {
                continue;
            }
            let both = seen.intersect(u.rect.intersect(u.clip));
            let h = t.rect.height().min(u.rect.height());
            if both.width() > 2.0 * SLACK && both.height() > 0.4 * h {
                out.push(Finding { text: t.text.clone(), fault: Fault::Overlap(u.text.clone()) });
            }
        }
    }
    out
}

/// One line for the report.
pub(crate) fn describe(f: &Finding) -> String {
    let t: String = f.text.chars().take(60).collect();
    let more = if f.text.chars().count() > 60 { "…" } else { "" };
    match &f.fault {
        Fault::Clipped(by) => format!("cut off by {by:.0} pt: \"{t}{more}\""),
        Fault::OffScreen(by) => format!("{by:.0} pt off screen: \"{t}{more}\""),
        Fault::Overlap(o) => {
            let o: String = o.chars().take(40).collect();
            format!("drawn over \"{o}\": \"{t}{more}\"")
        }
        Fault::Elided => format!("shortened to fit: \"{t}{more}\""),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/harness/text_layout/tests.rs"]
mod tests;
