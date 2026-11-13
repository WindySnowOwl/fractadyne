use super::*;

/// Every vocabulary tag has a message: one that fell through to `tag_label`'s last arm would show
/// its bare id ("single-hue") in every language.
#[test]
fn every_vocabulary_tag_has_a_name() {
    for d in fractadyne_color::tags::TAGS {
        let l = tag_label(d.id);
        assert!(!l.is_empty() && l != d.id, "{}: no message, shown as its id", d.id);
    }
}

/// A typed tag that names a vocabulary tag, by id or by its name, IS that tag, so filtering by
/// "Calm" finds a gradient the user tagged "calm"; anything else is kept as typed, tidied.
#[test]
fn a_typed_tag_becomes_a_vocabulary_tag_when_it_names_one() {
    assert_eq!(normalize_user_tag("  Calm "), Some("calm".into()));
    assert_eq!(normalize_user_tag("single-hue"), Some("single-hue".into()));
    assert_eq!(normalize_user_tag("Single hue"), Some("single-hue".into()));
    assert_eq!(normalize_user_tag("  my   favorites "), Some("my favorites".into()));
    assert_eq!(normalize_user_tag("   "), None);
    assert_eq!(normalize_user_tag(&"x".repeat(100)).map(|s| s.chars().count()), Some(MAX_TAG_CHARS));
}

/// A filter keeps a palette only when it carries EVERY chosen tag, case ignored.
#[test]
fn a_filter_keeps_only_palettes_with_every_tag() {
    let t = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert!(passes(&t(&["blues", "calm"]), &t(&["blues"])));
    assert!(passes(&t(&["blues", "calm"]), &t(&["blues", "calm"])));
    assert!(!passes(&t(&["blues"]), &t(&["blues", "calm"])));
    assert!(passes(&t(&["Favourites"]), &t(&["favourites"])));
    assert!(passes(&t(&["blues"]), &t(&[])), "no filter keeps everything");
}

/// ⭐**The thumbnail atlas fits the device's texture limit at any count and view shape.** Eight
/// fixed columns put 190 presets in a strip 24 cells tall: 11,520 texels for a tall view, past the
/// 8192 some GPUs allow, and the browser drew no thumbnails. A layout that fits keeps the cell's
/// size; one that cannot shrinks the cell, keeping its shape.
#[test]
fn the_atlas_fits_the_texture_limit_and_keeps_the_cell_when_it_can() {
    let tall = [320, 480];
    assert!(24 * tall[1] > 8192, "the old eight-column strip of 190 did not fit");
    assert_eq!(atlas_layout(190, tall, 8192).0, tall, "190 tall cells fit without shrinking");
    for n in [1, 2, 50, 190, 300, 2000] {
        for aspect in [0.2, 0.625, 1.0, 1.5, 5.0] {
            let cell = [320, ((320.0 * aspect) as f64).round() as u32];
            for max in [2048, 4096, 8192, 16384] {
                let ([w, h], cols) = atlas_layout(n, cell, max);
                let rows = n.div_ceil(cols);
                let case = format!("{n} cells of {cell:?} within {max}: {cols} x {rows} of {w}x{h}");
                assert!(cols >= 1 && cols <= n.max(1), "{case}");
                assert!(cols * w <= max && rows * h <= max, "{case}: too big");
                assert!(w <= cell[0] && h <= cell[1], "{case}: grew");
                let (r0, r1) = (cell[1] as f64 / cell[0] as f64, h as f64 / w as f64);
                assert!((r1 - r0).abs() <= r0 * 0.25 + 2.0 / w as f64, "{case}: shape changed");
                if w < cell[0] {
                    assert!(cols * cell[0] > max || rows * cell[1] > max, "{case}: shrank though it fit");
                }
            }
        }
    }
}

/// Outside the dual view a thumbnail shows the one view, whatever was chosen; in it, the chosen
/// panes, left to right as the screen has them.
#[test]
fn the_thumbnails_show_the_panes_chosen_in_the_dual_view() {
    for p in [BrowserPanes::Set, BrowserPanes::Julia, BrowserPanes::Both] {
        assert_eq!(p.views(false), [0], "{p:?} outside the dual view");
    }
    assert_eq!(BrowserPanes::Set.views(true), [0]);
    assert_eq!(BrowserPanes::Julia.views(true), [1]);
    assert_eq!(BrowserPanes::Both.views(true), [0, 1]);
    assert_eq!(BrowserPanes::default(), BrowserPanes::Both, "the dual view opens on both, as the screen shows them");
}
