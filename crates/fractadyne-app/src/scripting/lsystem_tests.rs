use super::parse_tour_text;
use crate::FractalKind;

fn tour(body: &str) -> Result<super::Playback, String> {
    parse_tour_text(&format!("format_version = 2\nname = \"t\"\n{body}"))
}

/// Why `body` is refused.
fn refused(body: &str) -> String {
    match tour(body) {
        Ok(_) => panic!("should be refused:\n{body}"),
        Err(e) => e,
    }
}

/// An L-system tour: its system by library name, a zoom below 1× (its own coordinates), the angle
/// turning and the drawing advancing along the glide, the order stepped at the keyframe.
#[test]
fn an_lsystem_keyframe_carries_its_system_order_angle_and_drawing() {
    let p = tour(
        "[[keyframe]]\nt = 0.0\nfractal = \"L-system\"\nlsystem = \"Koch curve\"\nre = \"0.5\"\nim = \"0.1\"\n\
         zoom = 0.5\norder = 4\nangle = 60\ndraw = 0.0\nhold = 1.0\n\
         [[keyframe]]\nt = 3.0\nzoom = 8\norder = \"auto\"\nangle = 80\ndraw = 1.0\nease = \"linear\"\n",
    )
    .expect("an L-system tour parses");
    let (a, mid, b) = (p.sample(0.5), p.sample(2.0), p.sample(3.0));
    assert_eq!(a.fractal, FractalKind::LSystem);
    assert_eq!(a.ls.system.as_ref().unwrap().name, "Koch curve");
    assert!((a.logmag - 0.5f64.ln()).abs() < 1e-12, "below 1×: {}", a.logmag);
    assert_eq!((a.ls.order, a.ls.angle, a.ls.draw), (Some(4), Some(60.0), Some(0.0)));
    // Halfway along a linear glide: half the turn, half the drawing; the order still the first's.
    assert!((mid.ls.angle.unwrap() - 70.0).abs() < 1e-9, "{:?}", mid.ls.angle);
    assert!((mid.ls.draw.unwrap() - 0.5).abs() < 1e-6, "{:?}", mid.ls.draw);
    assert_eq!(mid.ls.order, Some(4), "stepped at the keyframe");
    assert_eq!((b.ls.order, b.ls.angle, b.ls.draw), (None, Some(80.0), Some(1.0)), "\"auto\" follows the zoom");
    assert!(std::sync::Arc::ptr_eq(a.ls.system.as_ref().unwrap(), b.ls.system.as_ref().unwrap()), "inherited");
}

#[test]
fn an_lsystem_given_as_text_reads_and_a_wrong_one_is_refused() {
    let p = tour(
        "[[keyframe]]\nt = 0.0\nfractal = \"L-system\"\nre = \"0\"\nim = \"0\"\nzoom = 1\n\
         lsystem = '''\nangle 90\naxiom F\nF = F+F-F-F+F\n'''\n",
    )
    .unwrap();
    assert_eq!(p.sample(0.0).ls.system.as_ref().unwrap().rule(b'F').unwrap().len(), 9);
    let e = refused("[[keyframe]]\nt = 0.0\nfractal = \"L-system\"\nre = \"0\"\nim = \"0\"\nzoom = 1\nlsystem = \"No such system\"\n");
    assert!(e.contains("lsystem"), "{e}");
    let e = refused("[[keyframe]]\nt = 0.0\nfractal = \"L-system\"\nre = \"0\"\nim = \"0\"\nzoom = 1\norder = 2.5\n");
    assert!(e.contains("order"), "{e}");
    let e = refused("[[keyframe]]\nt = 0.0\nfractal = \"L-system\"\nre = \"0\"\nim = \"0\"\nzoom = 1\nangle = 900\n");
    assert!(e.contains("angle"), "{e}");
    // Its own coordinates are required: the Mandelbrot's default centre means nothing to it.
    let e = refused("[[keyframe]]\nt = 0.0\nfractal = \"L-system\"\nlsystem = \"Koch curve\"\n");
    assert!(e.contains("re`, `im` and `zoom"), "{e}");
    // An escape-time family is still held at 1× or more.
    let p = tour("[[keyframe]]\nt = 0.0\nzoom = 0.5\n").unwrap();
    assert_eq!(p.sample(0.0).logmag, 0.0);
}
