//! A tour's deepest frame's reference (`DeepReference`): which frames it is offered to.

use super::{parse_tour_text, DeepReference};

fn pb(body: &str) -> super::Playback {
    let text = format!("format_version = 2\nname = \"t\"\n{body}");
    parse_tour_text(&text).expect("script should parse")
}

/// A reference is the orbit of one formula: the deepest frame's is offered to every frame of the
/// same fractal and Julia setting (the try at reuse then decides by the frame's view), and never to
/// another fractal, another Julia c, or a dual view.
#[test]
fn the_deep_reference_is_offered_only_to_frames_of_its_own_formula() {
    let p = pb(r#"
[[keyframe]]
t = 0
zoom = 1.0
fractal = "Mandelbrot"

[[keyframe]]
t = 10
zoom = 1e40
fractal = "Mandelbrot"

[[keyframe]]
t = 20
zoom = 10.0
fractal = "Burning Ship"

[[keyframe]]
t = 30
zoom = 10.0
fractal = "Mandelbrot"
dual = true
"#);
    let deep = p.sample(10.0);
    let key = DeepReference::key(&deep);
    assert!(DeepReference::may_take(&key, &deep), "the deepest frame itself");
    assert!(DeepReference::may_take(&key, &p.sample(0.0)), "a shallower frame of the same dive");
    assert!(DeepReference::may_take(&key, &p.sample(5.0)), "a frame on the way down");
    assert!(!DeepReference::may_take(&key, &p.sample(20.0)), "another fractal");
    assert!(!DeepReference::may_take(&key, &p.sample(30.0)), "a dual view");

    // Julia sets: only the same c.
    let j = pb(r#"
[[keyframe]]
t = 0
zoom = 1e20
fractal = "Mandelbrot"
julia = true
julia_re = -0.75
julia_im = 0.11

[[keyframe]]
t = 10
zoom = 1.0
fractal = "Mandelbrot"
julia = true
julia_re = -0.75
julia_im = 0.11

[[keyframe]]
t = 20
zoom = 1.0
fractal = "Mandelbrot"
julia = true
julia_re = 0.285
julia_im = 0.01
"#);
    let jkey = DeepReference::key(&j.sample(0.0));
    assert!(DeepReference::may_take(&jkey, &j.sample(10.0)), "the same Julia set");
    assert!(!DeepReference::may_take(&jkey, &j.sample(20.0)), "another c");
    assert!(!DeepReference::may_take(&key, &j.sample(10.0)), "a Julia frame never takes the Mandelbrot dive's");
}
