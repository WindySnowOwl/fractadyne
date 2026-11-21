use super::*;

const SMOOTH: &str = "fn custom_color(l: Lanes) -> vec3<f32> {\n    if (l.inside) { return l.color; }\n    return palette(palette_map(l.count));\n}\n";
const NAMED: &str = "// @param p1 Gain 0 4 2\nfn custom_color(l: Lanes) -> vec3<f32> { return l.color * p1(); }\n";

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("fd-hook-test-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("hook.wgsl")
}

#[test]
fn params_read_and_write_four_numbers() {
    assert_eq!(parse_params("1, 0.5,-2,3e-3"), Some(vec![1.0, 0.5, -2.0, 0.003]));
    assert_eq!(parse_params("1,2,3"), None);
    assert_eq!(parse_params("1,2,3,NaN"), None);
    assert_eq!(parse_params("1,2,x,4"), None);
    let p = [0.1f32, 1.0 / 3.0, -7.25, 1e-7];
    assert_eq!(parse_params(&params_line(&p)), Some(p.to_vec()), "bit for bit");
}

#[test]
fn a_hook_takes_its_defaults_unless_given_four_values() {
    let mut s = HookState::default();
    s.set_text(NAMED, None, None).unwrap();
    assert_eq!(s.params, [2.0, 0.5, 0.5, 0.5]);
    s.set_text(NAMED, Some(&[3.0, 0.0, 0.0, 1.0]), None).unwrap();
    assert_eq!(s.params, [3.0, 0.0, 0.0, 1.0]);
    s.set_text(NAMED, Some(&[3.0]), None).unwrap();
    assert_eq!(s.params, [2.0, 0.5, 0.5, 0.5], "a wrong count is the defaults");
    assert_eq!(s.gpu_params(), [2.0, 0.5, 0.5, 0.5]);
}

#[test]
fn a_text_that_does_not_build_keeps_the_hook_in_use() {
    let mut s = HookState::default();
    s.set_text(SMOOTH, None, None).unwrap();
    let (rev, key) = (s.rev, s.active.as_ref().unwrap().key);
    let e = s.set_text("fn custom_color(l: Lanes) -> vec3<f32> {\n  return l.count\n}\n", None, None).unwrap_err();
    assert!(e.starts_with("line "), "{e}");
    assert_eq!(s.active.as_ref().unwrap().key, key, "the last hook that built stays");
    assert_eq!(s.rev, rev);
    assert_eq!(s.error.as_deref(), Some(e.as_str()));
    s.set_text(SMOOTH, None, None).unwrap();
    assert!(s.error.is_none(), "a text that builds clears the error");
}

#[test]
fn turning_off_drops_the_hook_and_the_file() {
    let mut s = HookState::default();
    s.set_text(SMOOTH, None, None).unwrap();
    let rev = s.rev;
    s.turn_off();
    assert!(s.active.is_none() && s.path.is_none() && s.error.is_none());
    assert_eq!(s.rev, rev + 1);
    assert_eq!(s.gpu_params(), [0.0; MAX_PARAMS]);
    assert!(s.stats().is_empty());
}

#[test]
fn the_statistics_are_the_hooks() {
    let mut s = HookState::default();
    s.set_text("fn custom_color(l: Lanes) -> vec3<f32> { return palette(l.trap); }\n", None, None).unwrap();
    assert_eq!(s.stats(), crate::StatSet::of(fractadyne_core::coloring::Stat::Trap));
}

#[test]
fn a_file_too_long_or_not_text_is_refused_by_name() {
    let p = scratch("long");
    std::fs::write(&p, vec![b' '; color_hook::MAX_TEXT + 1]).unwrap();
    assert!(read_hook_file(&p).unwrap_err().contains("too long"));
    std::fs::write(&p, [0xff, 0xfe, 0x00]).unwrap();
    assert!(read_hook_file(&p).unwrap_err().contains("UTF-8"));
    // A byte-order mark is not part of the text.
    std::fs::write(&p, format!("\u{feff}{SMOOTH}")).unwrap();
    assert_eq!(read_hook_file(&p).unwrap(), SMOOTH);
    let _ = std::fs::remove_dir_all(p.parent().unwrap());
}

#[test]
fn a_saved_file_is_picked_up_and_a_broken_save_keeps_the_hook() {
    let p = scratch("watch");
    std::fs::write(&p, SMOOTH).unwrap();
    let mut s = HookState::default();
    s.load_file(&p).unwrap();
    assert_eq!(s.path.as_deref(), Some(p.as_path()));
    s.params[0] = 0.25;
    let first = s.active.as_ref().unwrap().key;
    // Wait for the watcher until `deadline`, polling as the UI does.
    let wait = |s: &mut HookState, done: &dyn Fn(&HookState) -> bool| {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while std::time::Instant::now() < deadline {
            s.poll(None);
            if done(s) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        false
    };
    // Modification times can be coarse: a different length is seen whatever the clock.
    std::fs::write(&p, format!("{SMOOTH}// saved again\n")).unwrap();
    assert!(wait(&mut s, &|s| s.active.as_ref().unwrap().key != first), "the save was not picked up");
    assert_eq!(s.params[0], 0.25, "a reload keeps the sliders");
    let second = s.active.as_ref().unwrap().key;
    std::fs::write(&p, "fn custom_color(l: Lanes) -> vec3<f32> { oops }\n").unwrap();
    assert!(wait(&mut s, &|s| s.error.is_some()), "the broken save was not reported");
    assert_eq!(s.active.as_ref().unwrap().key, second, "a broken save keeps the hook");
    s.turn_off();
    let _ = std::fs::remove_dir_all(p.parent().unwrap());
}
