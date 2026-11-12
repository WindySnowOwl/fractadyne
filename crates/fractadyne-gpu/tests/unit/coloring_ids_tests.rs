//! The shader names a coloring method or a trap shape only by the constants generated from
//! `fractadyne_core::coloring` (coloring plan step A10), and every module the renderer builds
//! starts from text that carries them.

use fractadyne_core::coloring::{wgsl_ids, ColorMethod, TrapType};

/// The shader file as written: it compares `color_method` and `trap_type` only with names.
#[test]
fn the_shader_compares_methods_and_traps_only_by_name() {
    let wgsl = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/mandelbrot.wgsl"));
    let mut numeric = Vec::new();
    for (i, line) in wgsl.lines().enumerate() {
        let code = line.split("//").next().unwrap_or("");
        for subject in ["color_method", "trap_type", "method"] {
            for op in ["==", "!="] {
                let mut rest = code;
                while let Some(at) = rest.find(&format!("{subject} {op} ")) {
                    let after = &rest[at + subject.len() + op.len() + 2..];
                    if after.starts_with(|c: char| c.is_ascii_digit()) {
                        numeric.push(format!("line {}: {}", i + 1, line.trim()));
                    }
                    rest = after;
                }
            }
        }
    }
    numeric.dedup(); // `color_method == 4u` is also `method == 4u`
    assert!(numeric.is_empty(), "compare with METHOD_* / TRAP_* instead:\n{}", numeric.join("\n"));
    // The file does not define the names itself: they come from the table.
    for m in ColorMethod::ALL {
        assert!(!wgsl.contains(&format!("const {}", m.spec().wgsl)), "{} defined by hand", m.spec().wgsl);
    }
    // And it uses them (a shader that stopped reading the method would make this test vacuous).
    for name in ["METHOD_STRIPE", "METHOD_TRIANGLE", "METHOD_TRAP", "METHOD_DISTANCE", "METHOD_DECOMPOSITION", "TRAP_CROSS", "TRAP_CIRCLE"] {
        assert!(wgsl.contains(name), "the shader no longer reads {name}");
    }
}

/// The renderer's source is the file plus the generated block, once, at the end.
#[test]
fn the_renderer_source_carries_the_generated_ids_once() {
    let src = crate::renderer_wgsl();
    let ids = wgsl_ids();
    assert!(src.ends_with(&ids), "the ids are appended at the end");
    assert_eq!(src.matches(ids.as_str()).count(), 1);
    for t in TrapType::ALL {
        assert_eq!(src.matches(&format!("const {}:", t.spec().wgsl)).count(), 1);
    }
    // The strip module starts from it too (custom formulas: `custom` tests validate theirs).
    let strip = crate::strip::source().expect("the strip module's slots");
    assert!(strip.contains(&ids));
}

/// The iteration accumulates what it is asked to and what the method on screen reads, so a caller
/// that set only the method (as every caller did before B2) gets exactly what it got then, and
/// one that set only the set gets no less.
#[test]
fn the_iteration_accumulates_the_set_and_what_the_method_reads() {
    use fractadyne_core::coloring::{Stat, StatSet};
    for m in ColorMethod::ALL {
        assert_eq!(crate::iter_stats(StatSet::NONE, m.gpu_id()), m.orbit_stats(), "{m:?}");
        assert_eq!(crate::iter_stats(m.orbit_stats(), ColorMethod::Smooth.gpu_id()), m.orbit_stats(), "{m:?}");
        let more = StatSet::of(Stat::Trap);
        assert_eq!(crate::iter_stats(more, m.gpu_id()), more.union(m.orbit_stats()), "{m:?}");
    }
    // A number that is no method reads nothing.
    assert_eq!(crate::iter_stats(StatSet::NONE, ColorMethod::ALL.len() as u32), StatSet::NONE);
    // The two structs ask the same rule.
    let p = crate::MandelbrotParams {
        color_method: ColorMethod::Stripe.gpu_id(),
        orbit_stats: StatSet::of(Stat::Angle),
        ..crate::MandelbrotParams::default()
    };
    assert_eq!(p.iter_stats(), StatSet::of(Stat::Stripe).with(Stat::Angle));
}

/// The shader masks `orbit_stats` only with the generated STAT_* names, and reads it where the
/// statistics are accumulated: per statistic in `aux_step` and the BLA fold, the stripe tail, and
/// the trap histogram.
#[test]
fn the_shader_masks_the_set_only_by_name() {
    let wgsl = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/mandelbrot.wgsl"));
    let mut numeric = Vec::new();
    for (i, line) in wgsl.lines().enumerate() {
        let code = line.split("//").next().unwrap_or("");
        for subject in ["orbit_stats & ", "(st & "] {
            let mut rest = code;
            while let Some(at) = rest.find(subject) {
                let after = &rest[at + subject.len()..];
                if after.starts_with(|c: char| c.is_ascii_digit()) {
                    numeric.push(format!("line {}: {}", i + 1, line.trim()));
                }
                rest = after;
            }
        }
    }
    assert!(numeric.is_empty(), "mask with STAT_* instead:\n{}", numeric.join("\n"));
    for s in fractadyne_core::coloring::Stat::ALL {
        assert!(!wgsl.contains(&format!("const {}", s.spec().wgsl)), "{} defined by hand", s.spec().wgsl);
    }
    // aux_step and aux_fold_run each test the three running statistics; the tail and the
    // histogram test theirs.
    for (name, at_least) in [("STAT_STRIPE", 3), ("STAT_TRIANGLE", 2), ("STAT_TRAP", 3)] {
        let n = wgsl.lines().filter(|l| l.split("//").next().unwrap_or("").contains(name)).count();
        assert!(n >= at_least, "{name} read on {n} lines, expected at least {at_least}");
    }
    // The iteration no longer reads the method on screen.
    assert!(!wgsl.contains("iu.color_method"), "the iteration reads the displayed method again");
}
