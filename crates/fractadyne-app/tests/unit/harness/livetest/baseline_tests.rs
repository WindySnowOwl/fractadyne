use super::baseline_name;
use crate::ColorMethod;

/// Smooth keeps the name every baseline had before methods (so they stay valid); every other
/// method has its own, by its key, and no two methods share one.
#[test]
fn smooth_keeps_the_old_name_and_each_method_has_its_own() {
    let tour = std::path::Path::new("tours/coloring-methods.toml");
    assert_eq!(baseline_name(tour, [480, 270], ColorMethod::Smooth), "livetest-coloring-methods-480x270.json");
    assert_eq!(baseline_name(tour, [480, 270], ColorMethod::Fringe), "livetest-coloring-methods-fringe-480x270.json");
    let names: std::collections::BTreeSet<String> =
        ColorMethod::ALL.iter().map(|m| baseline_name(tour, [480, 270], *m)).collect();
    assert_eq!(names.len(), ColorMethod::ALL.len());
}
