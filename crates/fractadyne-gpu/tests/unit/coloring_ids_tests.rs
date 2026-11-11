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

/// The GPU crate's aux test is the table's.
#[test]
fn method_needs_aux_reads_the_table() {
    for m in ColorMethod::ALL {
        assert_eq!(crate::method_needs_aux(m.gpu_id()), m.needs_aux(), "{m:?}");
    }
    assert!(!crate::method_needs_aux(ColorMethod::ALL.len() as u32));
}
