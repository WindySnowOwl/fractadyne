//! The custom gradient carried inside a `.fdn` file, and the shapes the reader must refuse.
//!
//! ⭐**Why this field exists at all.** A view's `palette=` is an INDEX into the built-in presets, so
//! a file saved while a hand-built gradient was live reopened in whatever preset happened to sit at
//! that index — the geometry survived and the colour did not. `palette_custom=` carries the
//! segments themselves, so the file is self-contained.
//!
//! ⚠⚠**The metadata container is `key=value`, one pair per LINE, Latin-1** (it is also written into
//! a PNG `tEXt` chunk). So the encoded value must contain no newline and no `=`, and that is not a
//! stylistic preference — a value carrying either silently truncates or invents a key. The tests
//! below pin both properties on a deliberately hostile gradient rather than on a tidy one.

use super::{decode_palette_segments, encode_palette_segments};
use fractadyne_state::PaletteSegment;

fn seg(
    left: f32,
    mid: f32,
    right: f32,
    lc: [f32; 4],
    rc: [f32; 4],
    blend: u8,
    space: u8,
    bp: [f32; 4],
) -> PaletteSegment {
    PaletteSegment {
        left,
        mid,
        right,
        left_color: lc,
        right_color: rc,
        blend,
        space,
        blend_params: bp,
    }
}

/// Not a tidy two-stop ramp: off-centre midpoints, every blend kind we ship, both HSV directions,
/// partial alpha, and a Bézier segment whose control points are the whole point of the field.
fn rich() -> Vec<PaletteSegment> {
    vec![
        seg(
            0.0,
            0.113_712_3,
            0.25,
            [0.0, 0.0, 0.0, 1.0],
            [0.937_254_9, 0.203_921_6, 0.101_960_8, 1.0],
            0,
            0,
            [0.0; 4],
        ),
        seg(
            0.25,
            0.4,
            0.5,
            [0.937_254_9, 0.203_921_6, 0.101_960_8, 1.0],
            [0.101_960_8, 0.878_431_4, 0.427_450_9, 0.5],
            1,
            1,
            [0.0; 4],
        ),
        seg(
            0.5,
            0.5,
            0.812_5,
            [0.101_960_8, 0.878_431_4, 0.427_450_9, 0.5],
            [0.043_137_3, 0.180_392_2, 0.941_176_4, 1.0],
            3,
            2,
            [0.0; 4],
        ),
        seg(
            0.812_5,
            0.9,
            1.0,
            [0.043_137_3, 0.180_392_2, 0.941_176_4, 1.0],
            [1.0, 1.0, 1.0, 1.0],
            5,
            0,
            [0.17, 0.67, 0.83, 0.33],
        ),
    ]
}

/// ⭐⭐**Every field, bit for bit** — not "looks the same". Rust's `Display` for `f32` emits the
/// shortest decimal that parses back to the identical bits, so an exact comparison is the right
/// assertion here; anything weaker would hide a precision loss until someone's gradient drifted.
#[test]
fn rich_gradient_round_trips_exactly() {
    let before = rich();
    let back = decode_palette_segments(&encode_palette_segments(&before))
        .expect("a gradient we just encoded must decode");
    assert_eq!(back.len(), before.len());
    for (i, (a, b)) in before.iter().zip(back.iter()).enumerate() {
        assert_eq!(a.left.to_bits(), b.left.to_bits(), "segment {i} left");
        assert_eq!(a.mid.to_bits(), b.mid.to_bits(), "segment {i} mid");
        assert_eq!(a.right.to_bits(), b.right.to_bits(), "segment {i} right");
        for c in 0..4 {
            assert_eq!(
                a.left_color[c].to_bits(),
                b.left_color[c].to_bits(),
                "segment {i} left_color[{c}]"
            );
            assert_eq!(
                a.right_color[c].to_bits(),
                b.right_color[c].to_bits(),
                "segment {i} right_color[{c}]"
            );
            assert_eq!(
                a.blend_params[c].to_bits(),
                b.blend_params[c].to_bits(),
                "segment {i} blend_params[{c}]"
            );
        }
        assert_eq!(a.blend, b.blend, "segment {i} blend");
        assert_eq!(a.space, b.space, "segment {i} space");
    }
}

/// ⚠⚠The container's two reserved characters. A gradient is written as one line of a `key=value`
/// file, so a value containing either would corrupt the *file*, not merely this field.
#[test]
fn encoding_is_safe_for_the_key_value_container() {
    let v = encode_palette_segments(&rich());
    assert!(!v.contains('\n'), "a newline would truncate the value");
    assert!(!v.contains('\r'), "a CR would truncate the value");
    assert!(!v.contains('='), "an '=' would invent a key");
    assert!(
        v.is_ascii(),
        "the container is Latin-1 and PNG tEXt is Latin-1"
    );
}

/// ⭐A file written before this field existed, or one saved from a preset, carries no
/// `palette_custom` at all — the decoder must not manufacture a gradient from nothing.
#[test]
fn empty_is_not_a_gradient() {
    assert!(decode_palette_segments("").is_none());
    assert!(decode_palette_segments("   ").is_none());
}

/// ⚠The field is untrusted input. Each of these is a shape that would render as something nobody
/// chose, so the reader must reject the whole field and leave the live palette alone.
#[test]
fn malformed_shapes_are_refused() {
    let good = encode_palette_segments(&rich());

    // Truncated mid-value: the field count no longer matches.
    let cut = &good[..good.len() - 12];
    assert!(decode_palette_segments(cut).is_none(), "truncated");

    // A gap between two segments — `eval` would read an undefined band.
    let mut gapped = rich();
    gapped[1].right = 0.4;
    assert!(
        decode_palette_segments(&encode_palette_segments(&gapped)).is_none(),
        "gap between segments"
    );

    // Does not start at 0 / does not end at 1.
    let mut short = rich();
    short[0].left = 0.1;
    short[0].mid = 0.2;
    assert!(
        decode_palette_segments(&encode_palette_segments(&short)).is_none(),
        "does not cover 0"
    );
    let mut stop_early = rich();
    stop_early[3].right = 0.9;
    stop_early[3].mid = 0.85;
    assert!(
        decode_palette_segments(&encode_palette_segments(&stop_early)).is_none(),
        "does not cover 1"
    );

    // Zero-width segment: a division by zero waiting to happen.
    let mut flat = rich();
    flat[1].right = flat[1].left;
    flat[2].left = flat[1].left;
    assert!(
        decode_palette_segments(&encode_palette_segments(&flat)).is_none(),
        "zero-width segment"
    );

    // A midpoint outside its own segment — the one malformed shape the ordering checks miss.
    let mut bad_mid = rich();
    bad_mid[2].mid = 0.9;
    assert!(
        decode_palette_segments(&encode_palette_segments(&bad_mid)).is_none(),
        "midpoint outside its segment"
    );

    // Non-finite values, which `Display` will happily write and `parse` will happily read back.
    let mut nan = rich();
    nan[0].left_color[1] = f32::NAN;
    assert!(
        decode_palette_segments(&encode_palette_segments(&nan)).is_none(),
        "NaN colour"
    );
    let mut inf = rich();
    inf[0].blend_params[0] = f32::INFINITY;
    assert!(
        decode_palette_segments(&encode_palette_segments(&inf)).is_none(),
        "infinite blend parameter"
    );

    // Garbage that is not numbers at all.
    assert!(decode_palette_segments("hello").is_none());
    assert!(decode_palette_segments("0,0,1,0,0,0,1,1,1,1,1,x,0,0,0,0,0").is_none());
}

/// ⛔⭐⭐**`blend` and `space` are a FILE FORMAT, and file formats are append-only.** They are the
/// `.ggr` numbering, shared with the session and the importers. This test fails the moment someone
/// renumbers them, which is the point: a renumbering silently recolours every file already on
/// disk — there is no version stamp inside this field to catch it later.
#[test]
fn blend_and_space_numbering_is_pinned() {
    let s = encode_palette_segments(&[seg(
        0.0,
        0.5,
        1.0,
        [0.0, 0.0, 0.0, 1.0],
        [1.0, 1.0, 1.0, 1.0],
        5,
        2,
        [0.25, 0.5, 0.75, 1.0],
    )]);
    assert_eq!(
        s, "0,0.5,1,0,0,0,1,1,1,1,1,5,2,0.25,0.5,0.75,1",
        "the encoded form is the on-disk format; changing it strands existing files"
    );
}

/// A single full-width segment is the degenerate but legal case — a two-colour gradient.
#[test]
fn one_segment_is_legal() {
    let one = vec![seg(
        0.0,
        0.5,
        1.0,
        [0.1, 0.2, 0.3, 1.0],
        [0.9, 0.8, 0.7, 1.0],
        0,
        0,
        [0.0; 4],
    )];
    let back = decode_palette_segments(&encode_palette_segments(&one)).expect("legal");
    assert_eq!(back.len(), 1);
    assert_eq!(back[0].right_color[0].to_bits(), 0.9f32.to_bits());
}

/// ⭐The cap is a shared constant, and the reader honours it exactly. The writer's side of the
/// symmetry (not emitting a gradient it could not read back) is checked in `view_metadata`'s own
/// guard, which reads the same constant — there is no second number to drift.
#[test]
fn the_segment_cap_is_the_shared_constant() {
    let build = |n: usize| {
        (0..n)
            .map(|i| {
                let (a, b) = (i as f32 / n as f32, (i + 1) as f32 / n as f32);
                seg(a, 0.5 * (a + b), b, [0.0, 0.0, 0.0, 1.0], [1.0; 4], 0, 0, [0.0; 4])
            })
            .collect::<Vec<_>>()
    };
    let at = build(super::MAX_EMBEDDED_SEGMENTS);
    assert!(
        decode_palette_segments(&encode_palette_segments(&at)).is_some(),
        "a gradient exactly at the cap must be readable"
    );
    let over = build(super::MAX_EMBEDDED_SEGMENTS + 1);
    assert!(
        decode_palette_segments(&encode_palette_segments(&over)).is_none(),
        "one segment past the cap must be refused"
    );
}
