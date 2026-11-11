use super::*;

fn no_fdn(_: &str) -> Result<String, String> {
    Err("no view files in this test".into())
}

fn one(extra: &str) -> Result<ViewSet, String> {
    parse_views(&format!("[[view]]\nname = \"a\"\niter = 100\n{extra}"), &no_fdn)
}

/// Every method of the table gets a cell, the trap once per shape and the stripe once more over its
/// tail; the names come from the table's keys.
#[test]
fn the_variants_walk_the_tables() {
    let keys: Vec<String> = variants().into_iter().map(Variant::key).collect();
    assert_eq!(
        keys,
        ["smooth", "stripe", "stripe-tail", "triangle", "trap-point", "trap-cross", "trap-circle", "distance", "decomposition"]
    );
    for m in ColorMethod::ALL {
        assert!(variants().iter().any(|v| v.method == m), "{m:?} has no cell");
    }
    for t in TrapType::ALL {
        assert!(variants().iter().any(|v| v.method == ColorMethod::OrbitTrap && v.trap == t), "{t:?} has no cell");
    }
}

/// The shipped view set reads, its view files included, and covers what the plan asks of it: every
/// built-in escape-time family, Julia, a custom formula, all three bands and a quick subset.
#[test]
fn the_shipped_view_set_reads() {
    let text = std::fs::read_to_string(crate::harness::selftest::anchored(VIEWS_FILE)).expect("views file");
    let set = parse_views(&text, &|p| std::fs::read_to_string(crate::harness::selftest::anchored(p)).map_err(|e| e.to_string()))
        .expect("views parse");
    for k in FractalKind::ALL {
        assert!(set.views.iter().any(|v| v.fractal == k && v.julia_c.is_none()), "no view of {}", k.name());
    }
    assert!(set.views.iter().any(|v| v.julia_c.is_some()), "no Julia view");
    assert!(set.views.iter().any(|v| v.formula.is_some()), "no custom formula view");
    // Direct, df32 and floatexp, by magnification (the request decides; these are the thresholds).
    assert!(set.views.iter().any(|v| v.zoom_log2 < 1e4f64.log2()));
    assert!(set.views.iter().any(|v| (1e4f64.log2()..1e28f64.log2()).contains(&v.zoom_log2)));
    assert!(set.views.iter().any(|v| v.zoom_log2 >= 1e28f64.log2()));
    let quick: Vec<&str> = set.views.iter().filter(|v| v.quick).map(|v| v.name.as_str()).collect();
    assert_eq!(quick.len(), 9, "{quick:?}");
    // A view file's center and zoom come through as text, untouched.
    let m = set.views.iter().find(|v| v.name == "mandelbrot-misiurewicz-7.4e143").expect("misiurewicz view");
    assert!(m.center.0.starts_with("-0.101096363845622161025785445738622565463805442826253483876931177660780840740470584274821219810516779033404531908556741193971546144260911882355703907680301615758418143435454482306810666402941937826741189160781952644899539699533446682339312669900981755715042520653886699186134433396160061056277538514425960956070356822164327995613303692671476"));
    assert_eq!(m.zoom, "7.399700e+143");
    assert!(set.views.iter().any(|v| v.opt_in));
}

#[test]
fn a_view_reads_its_keys() {
    let set = one("fractal = \"Burning Ship\"\ncenter = [\"-1.75\", \"-0.03\"]\nzoom = \"1e6\"\nquick = true\nnote = \"n\"").unwrap();
    let v = &set.views[0];
    assert_eq!(v.fractal, FractalKind::BurningShip);
    assert_eq!(v.center, ("-1.75".to_string(), "-0.03".to_string()));
    assert!((v.zoom_log2 - 1e6f64.log2()).abs() < 1e-9);
    assert!(v.quick && !v.opt_in);
    // Defaults: Mandelbrot at its own center, 1x.
    let d = &one("").unwrap().views[0];
    assert_eq!(d.fractal, FractalKind::Mandelbrot);
    assert_eq!(d.center, ("-0.5".to_string(), "0.0".to_string()));
    assert_eq!(d.zoom_log2, 0.0);
    // A formula is the Custom family; a Julia c is two numbers.
    assert_eq!(one("formula = \"z = z^3 + c\"").unwrap().views[0].fractal, FractalKind::Custom);
    assert_eq!(one("julia_c = [\"-0.75\", \"0.1\"]").unwrap().views[0].julia_c, Some((-0.75, 0.1)));
}

/// What would render something other than what the file says is an error, never a default.
#[test]
fn a_view_that_cannot_be_read_is_an_error() {
    for (extra, why) in [
        ("zom = \"1e6\"", "unknown field"),
        ("fractal = \"Burning Boat\"", "unknown fractal"),
        ("fractal = \"Life\"", "unknown fractal"),
        ("fractal = \"Custom\"", "Custom needs a formula"),
        ("fractal = \"Newton\"\nformula = \"z = z^3 + c\"", "the Custom family"),
        ("fractal = \"Newton\"\njulia_c = [\"0\", \"0\"]", "no Julia form"),
        ("julia_c = [\"x\", \"0\"]", "two numbers"),
        ("zoom = \"lots\"", "unreadable zoom"),
        ("center = [\"0.5\", \"i\"]", "unreadable center"),
        ("fdn = \"x.fdn\"\nzoom = \"2\"", "not both"),
        ("fdn = \"x.fdn\"", "no view files"),
    ] {
        let e = match one(extra) {
            Err(e) => e,
            Ok(_) => panic!("{extra:?} was accepted"),
        };
        assert!(e.contains(why), "{extra:?}: {e}");
    }
    let iter = |n: u32| parse_views(&format!("[[view]]\nname = \"a\"\niter = {n}\n"), &no_fdn);
    assert!(iter(15).is_err() && iter(16).is_ok() && iter(crate::MAX_ITER_LIMIT + 1).is_err());
    let twice = "[[view]]\nname = \"a\"\niter = 100\n[[view]]\nname = \"a\"\niter = 100\n";
    assert!(parse_views(twice, &no_fdn).unwrap_err().contains("named twice"));
    assert!(parse_views("[[view]]\nname = \"A b\"\niter = 100\n", &no_fdn).is_err());
    assert!(parse_views("", &no_fdn).unwrap_err().contains("no [[view]]"));
}

#[test]
fn a_view_file_gives_the_family_center_and_zoom() {
    let fdn = "# comment\n# ----- BEGIN FRACTADYNE VIEW -----\nfractal=Tricorn\ncenter_re=-1.25\ncenter_im=0.5e-3\nzoom=3.5e12\n# ----- END FRACTADYNE VIEW -----\n";
    let set = parse_views("[[view]]\nname = \"a\"\nfdn = \"v.fdn\"\niter = 100\n", &|p| {
        assert_eq!(p, "v.fdn");
        Ok(fdn.to_string())
    })
    .unwrap();
    let v = &set.views[0];
    assert_eq!(v.fractal, FractalKind::Tricorn);
    assert_eq!(v.center, ("-1.25".to_string(), "0.5e-3".to_string()));
    assert_eq!(v.zoom, "3.5e12");
}

#[test]
fn a_known_red_cell_names_a_view_a_variant_and_a_check() {
    let base = "[[view]]\nname = \"a\"\niter = 100\n";
    let known = |cell: &str, check: &str, reason: &str| {
        parse_views(&format!("{base}[[known]]\ncell = \"{cell}\"\ncheck = \"{check}\"\nreason = \"{reason}\"\n"), &no_fdn)
    };
    let set = known("a/trap-cross", "flat", "why").unwrap();
    assert!(set.known[0].covers("a", "trap-cross", "flat"));
    assert!(!set.known[0].covers("a", "trap-cross", "bla"));
    assert!(!set.known[0].covers("b", "trap-cross", "flat"));
    assert!(known("*/stripe", "bla", "why").unwrap().known[0].covers("anything", "stripe", "bla"));
    assert!(known("b/stripe", "flat", "why").unwrap_err().contains("no view"));
    assert!(known("a/trap", "flat", "why").unwrap_err().contains("no variant"));
    assert!(known("a/stripe", "flatness", "why").unwrap_err().contains("no check"));
    assert!(known("a/stripe", "flat", " ").unwrap_err().contains("reason"));
    assert!(known("a-stripe", "flat", "why").unwrap_err().contains("view/variant"));
}

/// `*` stands for any run of characters, none included; without one a pattern is the text itself.
#[test]
fn a_known_pattern_matches_cell_ids() {
    assert!(glob("*-deep-*/trap-*", "burning-ship-3-deep-2e28/trap-cross"));
    assert!(glob("*/stripe-tail", "mandelbrot-overview/stripe-tail"));
    assert!(!glob("*/stripe", "mandelbrot-overview/stripe-tail"));
    assert!(glob("newton-overview/*", "newton-overview/decomposition"));
    assert!(glob("a*b", "ab") && glob("a*b", "axxb") && !glob("a*b", "axxbc") && !glob("a*b*b", "ab"));
    assert!(glob("exact", "exact") && !glob("exact", "exact2"));
    let with_known = |view: &str, cell: &str| {
        parse_views(
            &format!("[[view]]\nname = \"{view}\"\niter = 100\n[[known]]\ncell = \"{cell}\"\ncheck = \"flat\"\nreason = \"why\"\n"),
            &no_fdn,
        )
    };
    let set = with_known("x-deep-1e6", "*-deep-*/trap-*").unwrap();
    assert!(set.known[0].covers("x-deep-1e6", "trap-point", "flat"));
    assert!(!set.known[0].covers("x-deep-1e6", "stripe", "flat"));
    // A pattern that matches no view or no variant is a typo.
    let bad = |cell: &str| with_known("a", cell);
    assert!(bad("*-deep-*/stripe").unwrap_err().contains("no view matches"));
    assert!(bad("a/trapp-*").unwrap_err().contains("no variant matches"));
}

#[test]
fn a_run_selects_its_cells() {
    let set = parse_views(
        "[[view]]\nname = \"m\"\niter = 100\nquick = true\n\
         [[view]]\nname = \"b\"\nfractal = \"Burning Ship\"\niter = 100\n\
         [[view]]\nname = \"deep\"\niter = 100\nopt_in = true\n",
        &no_fdn,
    )
    .unwrap();
    let n = variants().len();
    let ids = |cells: Vec<(usize, Variant)>| -> Vec<String> {
        cells.into_iter().map(|(v, var)| format!("{}/{}", set.views[v].name, var.key())).collect()
    };
    // Everything but the opt-in view.
    assert_eq!(select(&set, false, None).len(), 2 * n);
    assert_eq!(select(&set, true, None).len(), n);
    // --only picks cells by id; the view's Smooth cell comes along for the comparison.
    assert_eq!(ids(select(&set, false, Some("trap-cross"))), ["m/smooth", "m/trap-cross", "b/smooth", "b/trap-cross"]);
    assert_eq!(select(&set, false, Some("b/")).len(), n);
    // An opt-in view runs only when --only names it.
    assert_eq!(ids(select(&set, false, Some("deep/stripe-tail"))), ["deep/smooth", "deep/stripe-tail"]);
    assert!(select(&set, false, Some("nothing")).is_empty());
}

/// A 10 x 10 image, every pixel `f(x, y)` in r, g and b.
fn img(f: impl Fn(usize, usize) -> f32) -> Vec<f32> {
    (0..100).flat_map(|i| {
        let v = f(i % 10, i / 10);
        [v, v, v, 1.0]
    })
    .collect()
}

/// A 10 x 10 iteration texture, the count of every pixel `f(x, y)` (-1 interior, -2 a glitch).
fn counts(f: impl Fn(usize, usize) -> f32) -> Vec<f32> {
    (0..100).flat_map(|i| [f(i % 10, i / 10), 0.0, 0.0, 1e30]).collect()
}

/// Steep, as the self-test's oracle has it: a neighbor across the interior's edge or more than two
/// iterations away, or a glitch.
#[test]
fn steep_pixels_follow_the_oracles_rule() {
    let steep = |c: Vec<f32>| -> Vec<(usize, usize)> {
        steep_pixels(&c, 10).iter().enumerate().filter(|(_, s)| **s).map(|(i, _)| (i % 10, i / 10)).collect()
    };
    // Half an iteration a pixel: nothing steep.
    assert!(steep(counts(|x, _| x as f32 * 0.5)).is_empty());
    // An interior pixel and its four neighbors.
    assert_eq!(steep(counts(|x, y| if (x, y) == (4, 4) { -1.0 } else { 10.0 })), [(4, 3), (3, 4), (4, 4), (5, 4), (4, 5)]);
    // Three iterations across a column edge: both sides of it, every row.
    let jump = steep(counts(|x, _| if x < 5 { 10.0 } else { 13.0 }));
    assert_eq!(jump.len(), 20);
    assert!(jump.iter().all(|(x, _)| *x == 4 || *x == 5));
    // A glitch, and a NaN.
    assert_eq!(steep(counts(|x, y| if (x, y) == (0, 0) { -2.0 } else { 10.0 })), [(0, 0), (1, 0), (0, 1)]);
    assert!(steep(counts(|x, y| if (x, y) == (9, 9) { f32::NAN } else { 10.0 })).contains(&(9, 9)));
}

/// A difference counts only where the view is not steep: speckle that moves is not a defect, a
/// smooth region that moves is.
#[test]
fn an_accelerator_ab_judges_only_where_the_view_is_not_steep() {
    let flat = img(|_, _| 0.5);
    let calm = vec![false; 100];
    assert!(Ab::of(&flat, &flat, &calm).ok());
    // One pixel moved by 0.05: 1% of the pixels differ.
    let mut moved = flat.clone();
    moved[44 * 4] = 0.55;
    let ab = Ab::of(&flat, &moved, &calm);
    assert!(!ab.ok() && (ab.unexplained - 0.01).abs() < 1e-9 && ab.judged == 1.0, "{ab:?}");
    // Within AB_CHANNEL: no difference.
    moved[44 * 4] = 0.515;
    assert!(Ab::of(&flat, &moved, &calm).ok());
    // The same move at a steep pixel is not judged.
    moved[44 * 4] = 0.95;
    let mut steep = calm.clone();
    steep[44] = true;
    let ab = Ab::of(&flat, &moved, &steep);
    assert!(ab.ok() && ab.max > 0.4 && (ab.judged - 0.99).abs() < 1e-9, "{ab:?}");
    // No mask: every pixel judged.
    assert!(!Ab::of(&flat, &moved, &[]).ok());
    // Against a noise floor: no more pixels than a nudge changes passes, more fails.
    moved[44 * 4] = 0.55;
    assert!(Ab::of(&flat, &moved, &calm).with_floor(0.02).ok());
    assert!(Ab::of(&flat, &moved, &calm).with_floor(0.01).ok());
    assert!(!Ab::of(&flat, &moved, &calm).with_floor(0.005).ok());
    // A NaN counts, steep or not, whatever the floor; a size mismatch or a mask of the wrong size
    // never passes.
    let mut nan = flat.clone();
    nan[44 * 4 + 1] = f32::NAN;
    assert!(!Ab::of(&flat, &nan, &steep).ok());
    assert!(!Ab::of(&flat, &nan, &steep).with_floor(0.5).ok());
    assert!(!Ab::of(&flat, &flat[..396], &calm).ok());
    assert!(!Ab::of(&flat, &flat, &calm[..99]).ok());
}

/// Supersampling is judged on the smooth pixels; a view of speckle gives no verdict.
#[test]
fn supersampling_is_judged_on_smooth_pixels() {
    let ramp = img(|x, _| x as f32 * 0.01);
    let shifted = img(|x, _| x as f32 * 0.01 + 0.2);
    let (p, share) = smooth_p50_diff(&ramp, &shifted, 10);
    assert_eq!(share, 1.0);
    assert!((p.unwrap() - 0.2).abs() < 1e-6, "{p:?}");
    let speckle = img(|x, y| ((x + y) % 2) as f32);
    assert_eq!(smooth_p50_diff(&speckle, &ramp, 10), (None, 0.0));
}
