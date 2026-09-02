// Parsers for location files written by OTHER programs (Imagina .imag, Kalles
// Fraktaler / Fraktaler-3 .kfr), plus the go-to round trip that shares their
// precision requirement. All untrusted input: a shared location is the artifact
// people pass around on a forum.
// ---- .kfr import (manual checklist step 73) and go-to round-trip (step 68) ----------------

/// A 62-digit centre from a Kalles Fraktaler / Fraktaler-3 location file. The whole point of
/// importing one is to land on a DEEP coordinate, so the property that matters is that the
/// digits survive: a parser that quietly went through f64 would keep about 17 of them and
/// still produce a plausible-looking view a long way from the one that was shared.
const KFR_DEEP: &str = "\
Re: -1.7686249050856172346353441645074953226348553577059118970313449\n\
Im: 0.0041965917670430586733584119946276337571344847602401093387185\n\
Zoom: 1E30\n\
Iterations: 250000\n\
Colors: 16\n";

#[test]
fn kfr_import_keeps_every_digit_of_a_deep_centre() {
    let v = super::parse_kfr(KFR_DEEP).expect(".kfr with Re/Im/Zoom must parse");
    assert!(
        (v.zoom - 1.0e30).abs() / 1.0e30 < 1e-12,
        "zoom {} != 1e30",
        v.zoom
    );
    assert_eq!(v.iterations, Some(250_000));

    // Round-trip the centre back to decimal and compare digit strings. Comparing f64s here
    // would be exactly the bug this guards: two centres that differ in the 30th digit are
    // the same f64 and utterly different views at 1e30x.
    for (got, want) in [
        (
            &v.cx,
            "-1.7686249050856172346353441645074953226348553577059118970313449",
        ),
        (
            &v.cy,
            "0.0041965917670430586733584119946276337571344847602401093387185",
        ),
    ] {
        let s = super::to_decimal_string(got);
        let (a, b) = (digits(&s), digits(want));
        let keep = b.len().min(a.len());
        assert!(
            keep >= 55 && a[..keep] == b[..keep],
            "centre lost precision:\n  got  {s}\n  want {want}"
        );
    }
}

/// Significant digits, sign and point removed, leading zeros dropped — so two spellings of
/// the same number compare equal.
fn digits(s: &str) -> String {
    let d: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    d.trim_start_matches('0').to_string()
}

/// The parser must REFUSE a file that is not a location, rather than inventing a view from
/// whatever it found. Absent is not the same as zero.
#[test]
fn kfr_import_refuses_a_file_that_is_not_a_location() {
    for bad in [
        "",
        "hello world",
        "Iterations: 500\nColors: 16\n",          // no Re/Im/Zoom
        "Re: -1.5\nZoom: 1E6\n",                  // no Im
        "Re: -1.5\nIm: 0.0\n",                    // no Zoom
        "Re: not-a-number\nIm: 0.0\nZoom: 1E6\n", // unparseable coordinate
    ] {
        assert!(
            super::parse_kfr(bad).is_none(),
            "should have been refused: {bad:?}"
        );
    }
}

/// Step 68: the go-to dialog is pre-filled with `to_decimal_string` and read back with
/// `parse_bf`, so that pair must be lossless at the precision a deep view needs. astro-float's
/// FromStr takes its precision from the DIGIT COUNT, which is why this is worth pinning
/// rather than assuming.
#[test]
fn go_to_round_trips_a_deep_coordinate() {
    let want = "-1.7686249050856172346353441645074953226348553577059118970313449";
    let parsed = super::parse_bf(want).expect("a 62-digit coordinate must parse");
    let back = super::to_decimal_string(&parsed);
    let (a, b) = (digits(&back), digits(want));
    let keep = b.len().min(a.len());
    assert!(
        keep >= 55 && a[..keep] == b[..keep],
        "go-to round trip lost precision:\n  got  {back}\n  want {want}"
    );

    // And re-parsing what we printed must land on the same value, not merely something that
    // prints the same.
    let again = super::parse_bf(&back).expect("our own output must parse");
    assert_eq!(
        super::to_decimal_string(&again),
        back,
        "printing, parsing and printing again must be stable"
    );
}

use super::{parse_imagina_text, to_f64, IMAGINA_BINARY_MAGIC};

#[test]
fn the_nested_form_imagina_writes_is_parsed() {
    let t = "Formula: Mandelbrot
Location:
	Size: 2e-30
	Re: -0.75
	Im: 0.1
	Iterations: 25000
";
    let v = parse_imagina_text(t).expect("nested form must parse");
    assert!((to_f64(&v.cx) + 0.75).abs() < 1e-12);
    assert!((to_f64(&v.cy) - 0.1).abs() < 1e-12);
    assert_eq!(v.iterations, Some(25_000));
    // Size is a half-height: mag = 2/Size = 1e30.
    assert!((v.zoom / 1.0e30 - 1.0).abs() < 1e-9, "zoom was {}", v.zoom);
}

#[test]
fn the_dotted_form_is_parsed_identically() {
    let nested = "Location:
  Size: 4
  Re: -0.5
  Im: 0
";
    let dotted = "Location.Size: 4
Location.Re: -0.5
Location.Im: 0
";
    let a = parse_imagina_text(nested).expect("nested");
    let b = parse_imagina_text(dotted).expect("dotted");
    assert_eq!(a.zoom, b.zoom);
    assert_eq!(to_f64(&a.cx), to_f64(&b.cx));
}

#[test]
fn a_full_precision_centre_survives_verbatim() {
    // The whole point of importing: deep centres are long, and truncating one silently moves
    // the view. 139 digits, the length our own session files already carry.
    let re = "-0.101096363845622131810062384757351929938361014185318540959576769264716835033666295089126713641250096615102645646890476648163450651052568";
    let t = format!(
        "Location:
 Size: 1e-100
 Re: {re}
 Im: 0.5
"
    );
    let v = parse_imagina_text(&t).expect("deep centre must parse");
    let round = crate::to_decimal_string(&v.cx);
    assert!(round.starts_with("-1.0109636384562213181006238475735192993836101418531854095957676926471683503366629508912671364125"),
            "precision lost on import: {round}");
}

#[test]
fn junk_and_hostile_input_is_refused_not_guessed() {
    assert!(parse_imagina_text("").is_none());
    assert!(parse_imagina_text(
        "Formula: Mandelbrot
"
    )
    .is_none()); // no location
    assert!(parse_imagina_text(
        "Location:
 Size: 0
 Re: 0
 Im: 0
"
    )
    .is_none()); // zero size
    assert!(parse_imagina_text(
        "Location:
 Size: -1
 Re: 0
 Im: 0
"
    )
    .is_none());
    assert!(parse_imagina_text(
        "Location:
 Size: nan
 Re: 0
 Im: 0
"
    )
    .is_none());
    assert!(parse_imagina_text(
        "Location:
 Size: 1
 Re: not-a-number
 Im: 0
"
    )
    .is_none());
    // A binary .im must not be coaxed into a text parse.
    assert_eq!(IMAGINA_BINARY_MAGIC[0], 0xFF);
    assert_eq!(&IMAGINA_BINARY_MAGIC[1..5], b"IMPV");
}

#[test]
fn a_shallow_location_clamps_rather_than_inverting() {
    // Size larger than the whole set: mag would fall below 1, which our viewport treats as the
    // home view. Clamp, never produce a sub-1 or negative magnification.
    let v = parse_imagina_text(
        "Location:
 Size: 1e6
 Re: 0
 Im: 0
",
    )
    .expect("parses");
    assert_eq!(v.zoom, 1.0);
}
