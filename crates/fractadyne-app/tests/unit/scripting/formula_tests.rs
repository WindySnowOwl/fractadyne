use super::parse_tour_text;
use crate::FractalKind;

fn tour(body: &str) -> Result<super::Playback, String> {
    parse_tour_text(&format!("format_version = 2\nname = \"t\"\n{body}"))
}

#[test]
fn a_keyframe_carries_a_custom_formula_and_later_ones_inherit_it() {
    let p = tour(
        "[[keyframe]]\nt = 0.0\nzoom = 1.0\nformula = '''\nt = sqr(z)\nz = t + p1*conj(t) + c'''\n\
         formula_params = [[0.25, -0.1]]\nhold = 1.0\n\
         [[keyframe]]\nt = 2.0\nzoom = 4.0\n",
    )
    .expect("a tour with a formula parses");
    let (a, b) = (p.sample(0.5), p.sample(2.0));
    assert_eq!(a.fractal, FractalKind::Custom, "a formula implies fractal = Custom");
    let c = a.custom.as_ref().expect("the frame carries its formula");
    assert_eq!(c.source, "t = sqr(z)\nz = t + p1*conj(t) + c");
    assert_eq!(c.params[0], (0.25, -0.1));
    assert_eq!(b.fractal, FractalKind::Custom, "inherited");
    assert!(std::sync::Arc::ptr_eq(c, b.custom.as_ref().unwrap()), "one compile, shared by both keyframes");
}

#[test]
fn parameters_alone_re_parameterize_the_formula_in_force() {
    let p = tour(
        "[[keyframe]]\nt = 0.0\nformula = \"z = z^3 - p1*z + c\"\nformula_params = [[0.5, 0.0]]\nhold = 1.0\n\
         [[keyframe]]\nt = 2.0\nformula_params = [[0.75, 0.0]]\nhold = 1.0\n\
         [[keyframe]]\nt = 4.0\nformula_params = [[0.5, 0.0]]\n",
    )
    .unwrap();
    let (a, b, c) = (p.sample(0.0), p.sample(2.5), p.sample(4.0));
    assert_eq!(b.custom.as_ref().unwrap().source, "z = z^3 - p1*z + c");
    assert_eq!(b.custom.as_ref().unwrap().params[0], (0.75, 0.0));
    // Stepped, not interpolated: a frame mid-glide shows the keyframe it left.
    assert_eq!(p.sample(1.5).custom.as_ref().unwrap().params[0], (0.5, 0.0));
    // Back to the first parameters: the SAME compiled formula, not a second one.
    assert!(std::sync::Arc::ptr_eq(a.custom.as_ref().unwrap(), c.custom.as_ref().unwrap()));
}

#[test]
fn a_built_in_family_after_a_formula_switches_away_and_back() {
    let p = tour(
        "[[keyframe]]\nt = 0.0\nformula = \"z = sin(z) + c\"\nhold = 1.0\n\
         [[keyframe]]\nt = 2.0\nfractal = \"Mandelbrot\"\nhold = 1.0\n\
         [[keyframe]]\nt = 4.0\nfractal = \"Custom\"\n",
    )
    .unwrap();
    assert_eq!(p.sample(2.5).fractal, FractalKind::Mandelbrot);
    assert_eq!(p.sample(4.0).fractal, FractalKind::Custom);
    assert_eq!(p.sample(4.0).custom.unwrap().source, "z = sin(z) + c", "fractal = Custom resumes the formula in force");
}

#[test]
fn what_a_tour_cannot_show_is_refused_with_the_keyframe_named() {
    let refused = |body: &str, says: &str| {
        let e = tour(body).err().unwrap_or_else(|| panic!("should be refused: {body}"));
        assert!(e.contains(says), "{e}");
    };
    refused("[[keyframe]]\nid = \"k\"\nt = 0.0\nfractal = \"Custom\"\n", "keyframe k: fractal = \"Custom\" needs a `formula`");
    refused(
        "[[keyframe]]\nid = \"k\"\nt = 0.0\nfractal = \"Mandelbrot\"\nformula = \"z = z^2 + c\"\n",
        "keyframe k: `formula` is a custom formula, but fractal = \"Mandelbrot\"",
    );
    refused("[[keyframe]]\nid = \"k\"\nt = 0.0\nformula_params = [[1.0, 0.0]]\n", "keyframe k: `formula_params` with no `formula`");
    refused("[[keyframe]]\nid = \"k\"\nt = 0.0\nformula = \"z = z^^2 + c\"\n", "keyframe k: the formula does not compile");
    refused(
        "[[keyframe]]\nid = \"k\"\nt = 0.0\nformula = \"z = z^2 + c\"\n\
         formula_params = [[1.0, 0.0], [1.0, 0.0], [1.0, 0.0], [1.0, 0.0], [1.0, 0.0], [1.0, 0.0]]\n",
        "keyframe k: 6 formula parameters",
    );
    refused("[[keyframe]]\nid = \"k\"\nt = 0.0\nformula = \"z = z^2 + p1\"\nformula_params = [[inf, 0.0]]\n", "not a finite number");
}
