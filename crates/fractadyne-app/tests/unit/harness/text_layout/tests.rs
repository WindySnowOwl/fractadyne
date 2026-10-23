use super::*;
use egui::{pos2, Id, Order};

fn item(text: &str, clip: Rect, rect: Rect) -> TextItem {
    TextItem { layer: LayerId::background(), clip, rect, text: text.into(), elided: false }
}

fn r(x0: f32, y0: f32, x1: f32, y1: f32) -> Rect {
    Rect::from_min_max(pos2(x0, y0), pos2(x1, y1))
}

const SCREEN: Rect = Rect { min: egui::Pos2 { x: 0.0, y: 0.0 }, max: egui::Pos2 { x: 1000.0, y: 800.0 } };

/// Text that fits its container and the screen, side by side, has nothing to report.
#[test]
fn text_that_fits_passes() {
    let clip = r(0.0, 0.0, 400.0, 300.0);
    let items = [item("File", clip, r(10.0, 10.0, 40.0, 24.0)), item("Edit", clip, r(50.0, 10.0, 80.0, 24.0))];
    assert!(analyse(SCREEN, &items).is_empty());
}

/// Each fault goes red on a planted case: a check that cannot fail is not a check.
#[test]
fn each_fault_is_found() {
    let clip = r(0.0, 0.0, 400.0, 300.0);
    // Cut off at the right edge by 30 points.
    let f = analyse(SCREEN, &[item("[Šåvé åš… ~~~~]", clip, r(300.0, 10.0, 430.0, 24.0))]);
    assert!(matches!(f[..], [Finding { fault: Fault::Clipped(by), .. }] if (by - 30.0).abs() < 0.01), "{f:?}");
    // Past the screen's edge, in a container that is too.
    let wide = r(600.0, 0.0, 1200.0, 300.0);
    let f = analyse(SCREEN, &[item("[Ŝéttîñĝš ~~~]", wide, r(900.0, 10.0, 1100.0, 24.0))]);
    assert!(matches!(f[..], [Finding { fault: Fault::OffScreen(by), .. }] if (by - 100.0).abs() < 0.01), "{f:?}");
    // Two texts over each other.
    let f = analyse(SCREEN, &[item("[Ŝåmplé ~~]", clip, r(10.0, 10.0, 90.0, 24.0)), item("Next", clip, r(60.0, 10.0, 100.0, 24.0))]);
    assert!(matches!(&f[..], [Finding { fault: Fault::Overlap(o), .. }] if o == "Next"), "{f:?}");
    // Shortened to fit.
    let mut e = item("[Lööñĝ…", clip, r(10.0, 10.0, 90.0, 24.0));
    e.elided = true;
    assert!(matches!(analyse(SCREEN, &[e])[..], [Finding { fault: Fault::Elided, .. }]));
}

/// What a container cuts on purpose is not a fault: rows scrolled part-way out of a list, rows
/// scrolled wholly out of it, a legend drawn twice for its engraving, and a popup over a window.
#[test]
fn what_is_cut_or_stacked_on_purpose_passes() {
    let clip = r(0.0, 100.0, 400.0, 300.0);
    let items = [
        // The top row of a scrolled list, half above its viewport.
        item("Row one", clip, r(10.0, 92.0, 80.0, 108.0)),
        // A row scrolled wholly out of view, and past the screen's edge as well.
        item("Row zero", clip, r(10.0, -40.0, 1500.0, -24.0)),
        // An engraved legend: the same text, one point apart.
        item("PLAN", clip, r(10.0, 150.0, 50.0, 164.0)),
        item("PLAN", clip, r(11.0, 151.0, 51.0, 165.0)),
    ];
    assert!(analyse(SCREEN, &items).is_empty(), "{:?}", analyse(SCREEN, &items));
    // Data longer than its field (a masked key, a number), and a formula's stacked glyphs.
    let field = r(0.0, 100.0, 200.0, 300.0);
    let data = [
        item("\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}", field, r(4.0, 110.0, 260.0, 124.0)),
        item("0.3125008988", field, r(150.0, 130.0, 248.0, 144.0)),
        item("\u{e010}", field, r(10.0, 200.0, 30.0, 220.0)),
        item("\u{e011}", field, r(12.0, 200.0, 32.0, 220.0)),
    ];
    assert!(analyse(SCREEN, &data).is_empty(), "{:?}", analyse(SCREEN, &data));
    let mut popup = item("Tooltip text", clip, r(20.0, 150.0, 120.0, 164.0));
    popup.layer = LayerId::new(Order::Tooltip, Id::new("tip"));
    assert!(analyse(SCREEN, &[item("Under it", clip, r(20.0, 150.0, 120.0, 164.0)), popup]).is_empty());
}

/// The pseudo-language marks the interface's text: brackets and accented letters (or its padding).
#[test]
fn the_pseudo_languages_text_is_recognised() {
    assert!(is_pseudo("[Ŝåvé îmåĝé… ~~~~]"));
    assert!(is_pseudo("[OK ~]"));
    assert!(!is_pseudo("Mandelbrot"));
    assert!(!is_pseudo("[x]"));
    assert!(!is_pseudo("-0.743643887 + 0.131825904i"));
    assert_eq!(crate::i18n::pseudo::transform("Save image"), "[Šåvé îmåĝé ~~~~]");
    assert!(is_pseudo(&crate::i18n::pseudo::transform("Close")));
}
