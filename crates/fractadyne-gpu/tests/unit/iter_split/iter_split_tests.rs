//! The split iterate module: both halves cut from the fixed module (and from a strip module), each
//! valid, each holding its modes' text exactly as the whole module does, and much smaller.

use super::*;

/// The renderer's module text, as every pipeline starts from it (the coloring ids appended).
fn fixed_module() -> &'static str {
    crate::renderer_wgsl()
}

fn validate(name: &str, src: &str) -> egui_wgpu::wgpu::naga::Module {
    use egui_wgpu::wgpu::naga;
    let module = naga::front::wgsl::parse_str(src).unwrap_or_else(|e| panic!("{name}: {}", e.emit_to_string(src)));
    naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
        .validate(&module)
        .unwrap_or_else(|e| panic!("{name}: {}", e.emit_to_string(src)));
    module
}

/// The text between two marker lines of `src`, markers excluded.
fn inside<'a>(src: &'a str, (begin, end): (&str, &str)) -> &'a str {
    let b = src.find(begin).unwrap();
    let from = b + src[b..].find('\n').unwrap() + 1;
    let to = src[..src.find(end).unwrap()].rfind('\n').unwrap() + 1;
    &src[from..to]
}

#[test]
fn the_fixed_module_splits_into_two_valid_halves_that_keep_each_modes_code() {
    let (without_fe, fe_only) = split_sources(fixed_module()).expect("the fixed module carries every marker");
    for (name, half) in [("without mode 2", &without_fe), ("mode 2 only", &fe_only)] {
        let m = validate(name, half);
        assert!(m.entry_points.iter().any(|e| e.name == "fs_iterate"), "{name}");
        assert!(m.entry_points.iter().any(|e| e.name == "fs_iterate_gather"), "{name}");
    }
    // Each mode's code is in its half byte for byte, and not in the other.
    let direct = inside(fixed_module(), DIRECT);
    let fe = inside(fixed_module(), FE);
    let pert = inside(fixed_module(), PERT);
    assert!(direct.contains("if (iu.mode == 1u) {") && fe.contains("else if (iu.mode == 2u) {") && pert.contains("else {"));
    assert!(without_fe.contains(direct) && without_fe.contains(pert) && !without_fe.contains(fe));
    assert!(fe_only.contains(fe) && !fe_only.contains(direct) && !fe_only.contains(pert));
    // Outside the cuts, a half is the whole module: take its stand-in branch out and it is the
    // whole with the cut regions (markers included) taken out.
    let (f0, f1) = region(fixed_module(), FE).unwrap();
    let stub = format!("    else if (iu.mode == 2u) {ABSENT}\n");
    assert_eq!(without_fe.replacen(&stub, "", 1), format!("{}{}", &fixed_module()[..f0], &fixed_module()[f1..]));
    let ((d0, d1), (p0, p1)) = (region(fixed_module(), DIRECT).unwrap(), region(fixed_module(), PERT).unwrap());
    let rest = fe_only.replacen(&format!("    if (iu.mode == 1u) {ABSENT}\n"), "", 1).replacen(&format!("    else {ABSENT}\n"), "", 1);
    assert_eq!(rest, format!("{}{}{}", &fixed_module()[..d0], &fixed_module()[d1..p0], &fixed_module()[p1..]));
    // Each is smaller than the whole. (By the text, only somewhat; by the code a compiler inlines,
    // about half: see the module docs.)
    assert!(without_fe.len() < fixed_module().len() && fe_only.len() < fixed_module().len());
}

#[test]
fn a_strip_module_splits_too() {
    let strip = crate::strip::source().expect("the strip module's slots");
    let (a, b) = split_sources(&strip).expect("a strip module carries every marker");
    validate("strip, without mode 2", &a);
    validate("strip, mode 2 only", &b);
}

#[test]
fn a_source_without_its_markers_is_not_cut() {
    assert_eq!(split_sources(&fixed_module().replace("// @@SPLIT_PERT_END", "// gone")), None);
    assert_eq!(split_sources(&fixed_module().replace("// @@SPLIT_DIRECT_BEGIN", "// gone")), None);
    // A marker twice is as bad as none.
    let twice = fixed_module().replacen("// @@SPLIT_PERT_END", "// @@SPLIT_PERT_END\n    // @@SPLIT_PERT_END", 1);
    assert_eq!(split_sources(&twice), None);
    assert_eq!(split_sources("fn main() {}"), None);
}
