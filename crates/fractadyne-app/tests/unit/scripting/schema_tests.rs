/// The checked-in TOURS.md must match what the schema generates — regenerate it with
/// `fractadyne --dump-tour-schema > TOURS.md` after editing `TOUR_SCHEMA`. (Line endings are
/// normalized so a CRLF working copy still matches the LF the generator emits.)
#[test]
fn tour_schema_doc_current() {
    let generated = super::tour_schema_markdown();
    let committed = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../TOURS.md")).replace("\r\n", "\n");
    assert_eq!(
        generated, committed,
        "TOURS.md is stale — run `fractadyne --dump-tour-schema > TOURS.md` to regenerate"
    );
}

/// The `[[keyframe]]` table's field names, from the schema `--script-schema` prints: what a
/// script is ABLE to set. A keyframe field that writes one of the viewer's own settings must also
/// be handed back when the tour ends, and the only way to keep that true as fields are added is to
/// check the two lists against each other.
fn keyframe_field_names() -> Vec<&'static str> {
    super::TOUR_SCHEMA
        .iter()
        .find(|t| t.toml == "[[keyframe]]")
        .map(|t| t.fields.iter().map(|f| f.name).collect())
        .unwrap_or_default()
}

/// Checklist step 84, "close the tour player: playback stops and normal interactive control
/// returns". A tour may drive the viewer's own settings, and `PlaybackRestore` is what hands
/// them back — so the invariant worth pinning is COMPLETENESS: every keyframe field that
/// writes a session setting must have somewhere to be restored from. Add a keyframe field
/// without adding it here and a played tour silently edits the viewer's session.
#[test]
fn stopping_playback_restores_interaction() {
    // Keyframe fields a script may set, split by what happens when the tour ends. The
    // "kept" list is not laziness: where the tour LEFT you is the view you keep exploring
    // from, and resetting it would throw away the thing the tour was showing you.
    const RESTORED: &[&str] = &["max_iter", "palette", "dual_split", "orbits", "minimap"];
    const KEPT: &[&str] = &[
        "id",
        "t",
        "hold",
        "ease",
        "transition",
        "transition_secs",
        "fade_out_secs",
        "location",
        "re",
        "im",
        "zoom",
        "fractal",
        // A tour that shows a custom formula leaves you on it, as it leaves you on its family.
        "formula",
        "formula_params",
        "julia",
        "dual",
        "julia_re",
        "julia_im",
        "orbit_re",
        "orbit_im",
        // An L-system tour leaves you on its system, as a formula tour leaves you on its formula.
        "lsystem",
        "order",
        "angle",
        "draw",
    ];
    let fields = keyframe_field_names();
    for f in &fields {
        assert!(
            RESTORED.contains(f) || KEPT.contains(f),
            "keyframe field {f:?} is neither restored when a tour ends nor listed as \
                 deliberately kept — decide which, or a tour silently edits the session"
        );
    }
    for f in RESTORED {
        assert!(
            fields.contains(f),
            "{f:?} is restored but is no longer a keyframe field"
        );
    }
    // And the record itself carries a slot for each restored field (plus the palette's three
    // companions, since choosing a preset overrides a custom gradient / binary / duotone).
    let r = crate::scripting::PlaybackRestore {
        max_iter: 12_345,
        auto_iter: false,
        palette_idx: 3,
        use_custom_palette: true,
        use_binary: true,
        use_duotone: true,
        minimap: true,
        show_orbits: true,
        dual_split: 0.34,
    };
    assert_eq!(r.max_iter, 12_345);
    assert!((r.dual_split - 0.34).abs() < 1.0e-6);
}
