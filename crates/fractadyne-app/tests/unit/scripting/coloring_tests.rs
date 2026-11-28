//! A tour's `[render.coloring]` (coloring plan B4): the keys a saved view carries, checked when the
//! script is read, so a misspelled coloring stops the tour instead of rendering in the viewer's.
use super::parse_tour_text;

fn tour(render: &str) -> Result<super::Playback, String> {
    parse_tour_text(&format!(
        "format_version = 3\nname = \"t\"\n{render}\n[[keyframe]]\nt = 0.0\nzoom = 1.0\nhold = 1.0\n[[keyframe]]\nt = 2.0\nzoom = 4.0\n"
    ))
}

#[test]
fn each_key_reads_in_its_own_toml_shape() {
    let p = tour(
        "[render.coloring]\nmethod = \"stripe\"\nstripe_freq = 8\nstripe_tail = true\nstripe_tail_len = 64\n\
         light = true\nlight_angle = 0.5\npalette_mode = \"duotone\"\npalette_lo = [0.1, 0.2, 0.3]\n",
    )
    .expect("a tour with a coloring parses");
    let mut got = p.render.coloring.clone();
    got.sort();
    let mut want: Vec<(String, String)> = [
        ("method", "stripe"),
        ("stripe_freq", "8"),
        ("stripe_tail", "1"),
        ("stripe_tail_len", "64"),
        ("light", "1"),
        ("light_angle", "0.5"),
        ("palette_mode", "duotone"),
        ("palette_lo", "0.1,0.2,0.3"),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();
    want.sort();
    assert_eq!(got, want);
}

/// A format-2 tour states the light's angle in radians: it is kept in degrees, as format 3 states it
/// (user, 2026-11-28), so the same script draws the same light.
#[test]
fn a_format_2_tour_states_the_light_angle_in_radians() {
    let v2 = parse_tour_text(
        "format_version = 2\nname = \"t\"\n[render.coloring]\nlight = true\nlight_angle = 0.5\n[[keyframe]]\nt = 0.0\nzoom = 1.0\n",
    )
    .expect("a v2 tour parses");
    let angle = v2.render.coloring.iter().find(|(k, _)| k == "light_angle").map(|(_, v)| v.clone());
    assert_eq!(angle.as_deref(), Some("28.64789"), "0.5 radians");
    let v3 = tour("[render.coloring]\nlight = true\nlight_angle = 0.5\n").expect("a v3 tour parses");
    let angle = v3.render.coloring.iter().find(|(k, _)| k == "light_angle").map(|(_, v)| v.clone());
    assert_eq!(angle.as_deref(), Some("0.5"), "0.5 degrees");
}

#[test]
fn no_table_is_no_coloring() {
    assert!(tour("").expect("parses").render.coloring.is_empty());
    assert!(tour("[render]\nfps = 24\n").expect("parses").render.coloring.is_empty());
}

#[test]
fn an_unknown_key_is_refused_with_the_keys() {
    let e = tour("[render.coloring]\nmethd = \"stripe\"\n").err().expect("refused");
    assert!(e.contains("has no key `methd`") && e.contains("method, band_texture, band_texture_amount, band_texture_near, stat_layer, stat_layer_blend, stat_layer_contrast, stat_layer_opacity, stat_bump, stat_bump_height, stat_gloss, stat_gloss_amount, stripe_freq"), "{e}");
}

/// The wrong shape, a value the key cannot take, and a number out of its slider's range are each
/// refused, naming the key and what it expects.
#[test]
fn a_value_it_cannot_take_is_refused() {
    for (body, expects) in [
        ("stripe_freq = 99", "float, 1 to 24"),
        ("stripe_freq = \"many\"", "float, 1 to 24"),
        ("light = \"yes\"", "bool"),
        ("light = 2", "bool"),
        ("method = \"plasma\"", "smooth \\| stripe"),
        ("trap = \"triangle\"", "point \\| cross \\| circle \\| line \\| square \\| hexagon \\| star \\| sine \\| s-curve"),
        ("palette_mode = \"off\"", "duotone \\| binary"),
        ("palette_lo = [0.1, 0.2]", "[r, g, b]"),
        ("palette_lo = [0.1, 0.2, 7.0]", "[r, g, b]"),
        ("stripe_tail_len = 2.5", "int, 4 to"),
    ] {
        let e = tour(&format!("[render.coloring]\n{body}\n")).err().unwrap_or_else(|| panic!("accepted {body}"));
        let key = body.split(' ').next().unwrap();
        assert!(e.contains(&format!("[render.coloring] {key} =")) && e.contains(expects), "{body}: {e}");
    }
}
