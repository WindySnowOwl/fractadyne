use super::*;

const SMOOTH: &str = "fn custom_color(l: Lanes) -> vec3<f32> {\n    if (l.inside) { return l.color; }\n    return palette(palette_map(l.count));\n}\n";

fn line_of_error(text: &str) -> HookError {
    build(text).expect_err("the hook should not build")
}

#[test]
fn every_example_builds() {
    for (name, text) in EXAMPLES {
        let h = build(text).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(h.params.len(), MAX_PARAMS, "{name}");
        assert!(h.stats.is_empty(), "{name} reads no statistic");
        assert!(h.source.ends_with(&format!("{text}\n")), "{name}: the text is last, as written");
    }
}

#[test]
fn the_examples_params_are_their_lines() {
    let fringe = build(EXAMPLES.iter().find(|(n, _)| *n == "fringe.wgsl").unwrap().1).unwrap();
    assert_eq!(fringe.params[0], HookParam { name: "Line width (pixels)".into(), min: 0.25, max: 4.0, default: 1.0 });
    assert_eq!(fringe.params[1], HookParam::unnamed(1));
    assert_eq!(fringe.defaults(), [1.0, 0.5, 0.5, 0.5]);
}

#[test]
fn the_coloring_pass_is_cut_with_both_shading_functions_renamed() {
    let p = prefix().unwrap();
    assert!(p.contains("fn builtin_shade_aa(") && p.contains("fn builtin_shade("));
    // The interface's own two, once each.
    assert_eq!(p.matches("fn shade_aa(").count(), 1);
    assert_eq!(p.matches("fn shade(").count(), 1);
    assert!(p.contains("fn fs_color(") && p.contains("fn vs_main("));
    assert!(!p.contains("fn fs_iterate("), "no iteration in a hook's module");
    assert!(p.contains("const LUT2_BASE") && p.contains("const METHOD_SMOOTH"));
    assert!(p.ends_with(&format!("{USER_MARK}\n")));
}

#[test]
fn the_key_is_the_module() {
    let a = build(SMOOTH).unwrap();
    let b = build(&format!("{SMOOTH}// another comment\n")).unwrap();
    assert_ne!(a.key, b.key);
    assert_eq!(a.key, build(SMOOTH).unwrap().key);
}

#[test]
fn a_syntax_error_names_the_texts_own_line() {
    let e = line_of_error("// line 1\n\nfn custom_color(l: Lanes) -> vec3<f32> {\n    let x = 1.0 +;\n    return l.color;\n}\n");
    assert_eq!(e.line, Some(4), "{e}");
    assert!(e.to_string().starts_with("line 4, column "), "{e}");
}

#[test]
fn a_type_error_names_the_texts_own_line() {
    // Validation, not parsing: the return type is wrong.
    let e = line_of_error("fn custom_color(l: Lanes) -> vec3<f32> {\n    return l.count;\n}\n");
    assert_eq!(e.line, Some(2), "{e}");
    // Without naga's handle numbers.
    assert!(!e.message.contains('[') && !e.message.contains("Some("), "{e}");
    assert!(e.message.contains("'custom_color'"), "{e}");
}

#[test]
fn handle_numbers_are_tidied_away() {
    assert_eq!(
        tidy("Function [22] 'f' is invalid: The `return` value Some([1]) does not match"),
        "Function 'f' is invalid: The `return` value does not match"
    );
    assert_eq!(tidy("index [x] and a[3]"), "index [x] and a[3]");
}

#[test]
fn a_missing_hook_function_is_said_plainly() {
    let e = line_of_error("// fn custom_color(l: Lanes) -> vec3<f32> is in a comment only\nfn other() {}\n");
    assert_eq!(e.line, None);
    assert!(e.message.contains("defines no `fn custom_color"), "{e}");
    // A longer name is not the hook either.
    assert!(line_of_error("fn custom_color2(l: Lanes) -> vec3<f32> { return l.color; }\n").message.contains("defines no"));
}

#[test]
fn a_hook_with_the_wrong_signature_is_an_error_not_a_panic() {
    let e = line_of_error("fn custom_color(x: f32) -> vec3<f32> {\n    return vec3<f32>(x);\n}\n");
    assert!(!e.message.is_empty(), "{e}");
}

#[test]
fn every_kind_of_loop_is_refused_at_its_line() {
    let body = |lp: &str| format!("fn custom_color(l: Lanes) -> vec3<f32> {{\n    var s = 0.0;\n    {lp}\n    return vec3<f32>(s);\n}}\n");
    for lp in [
        "for (var i = 0; i < 4; i++) { s += 1.0; }",
        "while (s < 4.0) { s += 1.0; }",
        "loop { s += 1.0; if (s > 4.0) { break; } }",
        "if (l.inside) { for (var i = 0; i < 4; i++) { s += 1.0; } }",
    ] {
        let e = line_of_error(&body(lp));
        assert_eq!(e.line, Some(3), "{lp}: {e}");
        assert!(e.message.starts_with("a loop"), "{lp}: {e}");
    }
    // In a helper the hook calls, too.
    let e = line_of_error("fn helper() -> f32 {\n    var s = 0.0;\n    loop { s += 1.0; if (s > 2.0) { break; } }\n    return s;\n}\nfn custom_color(l: Lanes) -> vec3<f32> { return vec3<f32>(helper()); }\n");
    assert_eq!(e.line, Some(3), "{e}");
}

#[test]
fn the_interfaces_own_loops_are_fine() {
    // with_lightness bisects in a fixed loop of its own.
    build("fn custom_color(l: Lanes) -> vec3<f32> { return with_lightness(l.color, 0.5); }\n").unwrap();
}

#[test]
fn module_scope_vars_overrides_and_entry_points_are_refused() {
    let hook = "fn custom_color(l: Lanes) -> vec3<f32> { return l.color; }\n";
    let e = line_of_error(&format!("{hook}var<private> state: f32;\n"));
    assert_eq!(e.line, Some(2), "{e}");
    assert!(e.message.contains("module-scope `var`"), "{e}");
    let e = line_of_error(&format!("{hook}@group(0) @binding(7) var<uniform> mine: vec4<f32>;\n"));
    assert_eq!(e.line, Some(2), "{e}");
    let e = line_of_error(&format!("{hook}override GAIN: f32 = 1.0;\n"));
    assert_eq!(e.line, Some(2), "{e}");
    assert!(e.message.contains("override"), "{e}");
    let e = line_of_error(&format!("{hook}@fragment\nfn fs_mine() -> @location(0) vec4<f32> {{ return vec4<f32>(1.0); }}\n"));
    assert_eq!(e.line, Some(3), "{e}");
    assert!(e.message.contains("entry point"), "{e}");
}

#[test]
fn a_name_the_coloring_pass_uses_is_reported_in_the_text() {
    let e = line_of_error("fn custom_color(l: Lanes) -> vec3<f32> { return l.color; }\nfn palette(t: f32) -> vec3<f32> { return vec3<f32>(t); }\n");
    assert_eq!(e.line, Some(2), "{e}");
    assert!(e.message.contains("choose another"), "{e}");
}

#[test]
fn the_statistics_read_are_found_in_code_only() {
    let h = build("// l.angle in a comment is not a read\nfn custom_color(l: Lanes) -> vec3<f32> {\n    return palette(l.stripe + l.trap);\n}\n").unwrap();
    assert_eq!(h.stats, StatSet::of(Stat::Stripe).with(Stat::Trap));
    let h = build("/* l.triangle /* nested */ still a comment */\nfn custom_color(l: Lanes) -> vec3<f32> { return palette(l.angle); }\n").unwrap();
    assert_eq!(h.stats, StatSet::of(Stat::Angle));
    assert!(build(SMOOTH).unwrap().stats.is_empty());
}

#[test]
fn parameter_lines_read_and_refuse_by_line() {
    let p = params_of("// @param p2 Gain 0 10 2\n//@param p4 \"Two words\" -1 1\n// @param p1 \"Only a name\"\n").unwrap();
    assert_eq!(p[0], HookParam { name: "Only a name".into(), min: 0.0, max: 1.0, default: 0.5 });
    assert_eq!(p[1], HookParam { name: "Gain".into(), min: 0.0, max: 10.0, default: 2.0 });
    assert_eq!(p[2], HookParam::unnamed(2));
    assert_eq!(p[3], HookParam { name: "Two words".into(), min: -1.0, max: 1.0, default: 0.0 });
    // A default outside the range is held to it.
    assert_eq!(params_of("// @param p1 x 0 1 5\n").unwrap()[0].default, 1.0);
    for (bad, line) in [
        ("\n// @param p5 x 0 1\n", 2),
        ("// @param p1 \"unclosed 0 1\n", 1),
        ("// @param p1 x 1 0\n", 1),
        ("// @param p1 x zero 1\n", 1),
        ("// @param p1 x 0\n", 1),
        ("// @param p1 x 0 1 0.5 9\n", 1),
    ] {
        let e = params_of(bad).expect_err(bad);
        assert_eq!(e.line, Some(line), "{bad}: {e}");
    }
}

#[test]
fn comments_are_blanked_with_their_offsets_kept() {
    let t = "a // b\nc /* d\n e */ f";
    let c = code_only(t);
    assert_eq!(c.len(), t.len());
    assert_eq!(c.matches('\n').count(), 2);
    assert!(c.starts_with("a ") && c.ends_with(" f") && !c.contains('b') && !c.contains('d') && !c.contains('e'));
}

#[test]
fn text_that_is_not_a_hook_is_refused_before_parsing() {
    assert!(build(&"x".repeat(MAX_TEXT + 1)).unwrap_err().message.contains("at most"));
    assert!(build("fn custom_color(l: Lanes) -> vec3<f32> { return l.color; }\0").unwrap_err().message.contains("NUL"));
}
