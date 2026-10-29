//! `--order deepest`: the frames by decreasing depth, the depth exactly as a frame is sampled, and
//! the chain of deeper frames' references each next frame may take.

use super::{deepest_frame_order, parse_tour_text, FrameOrder, RefChain};

fn pb(body: &str) -> super::Playback {
    let text = format!("format_version = 2\nname = \"t\"\n{body}");
    parse_tour_text(&text).expect("script should parse")
}

#[test]
fn the_order_words_are_the_command_lines() {
    for (w, o) in FrameOrder::WORDS {
        assert_eq!(FrameOrder::parse(w), Some(o));
        assert_eq!(o.word(), w);
    }
    assert_eq!(FrameOrder::parse("Deepest"), Some(FrameOrder::Deepest), "case-insensitive");
    assert_eq!(FrameOrder::parse("deep"), None, "a typo is refused, never taken for sequential");
    assert_eq!(FrameOrder::default(), FrameOrder::Sequential);
}

/// Every frame exactly once (what `--resume` and the mp4 rest on), by decreasing depth, frames of
/// equal depth in time order.
#[test]
fn deepest_order_is_a_permutation_by_decreasing_depth() {
    // Two dives: in to 30 over frames 0-59, then out from 50 over 60-99, with a hold (equal depths).
    let depth = |i: u64| if i < 60 { (i as f64 / 2.0).min(25.0) } else { 50.0 - (i - 60) as f64 };
    for (first, last) in [(0u64, 99u64), (0, 0), (17, 83), (99, 0)] {
        let order = deepest_frame_order(first, last, depth);
        let (lo, hi) = (first.min(last), first.max(last));
        let mut sorted = order.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (lo..=hi).collect::<Vec<_>>(), "not a permutation of [{lo},{hi}]");
        for w in order.windows(2) {
            let (a, b) = (depth(w[0]), depth(w[1]));
            assert!(a > b || (a == b && w[0] < w[1]), "frame {} ({a}) before {} ({b})", w[0], w[1]);
        }
    }
    let order = deepest_frame_order(0, 99, depth);
    assert_eq!(&order[..3], &[60, 61, 62], "the deepest frame first, then the dive out from it");
    // The hold at 25 (frames 50-59), deeper than frame 85 (25): in time order, before it.
    let p = |f: u64| order.iter().position(|&x| x == f).unwrap();
    assert!(p(50) < p(51) && p(59) < p(85), "a hold in time order, and before a later frame of equal depth");
}

/// The depth the order is sorted by is the depth each frame renders at: `logmag_at` is
/// `sample().logmag` at every frame of tours with holds, easing, a pan and a zoom out.
#[test]
fn the_depth_a_frame_is_ordered_by_is_the_depth_it_renders_at() {
    let p = pb(r#"
[[keyframe]]
t = 0
zoom = 1.0
fractal = "Mandelbrot"

[[keyframe]]
t = 8
hold = 2
zoom = 1e30
ease = "smoother"

[[keyframe]]
t = 15
zoom = "1e12"
re = "-0.75"
im = "0.1"
ease = "out"

[[keyframe]]
t = 22
zoom = 4.0
fractal = "Burning Ship"
"#);
    let fps = 7.0;
    let frames = super::tour_frame_count(p.total, fps);
    assert!(frames > 100, "{frames} frames");
    for i in 0..frames {
        let t = (i as f64 / fps).min(p.total);
        assert_eq!(p.logmag_at(t).to_bits(), p.sample(t).logmag.to_bits(), "frame {i} at {t} s");
    }
    for t in [-1.0, 0.0, 8.0, 9.5, 10.0, 22.0, 100.0] {
        assert_eq!(p.logmag_at(t).to_bits(), p.sample(t).logmag.to_bits(), "at {t} s");
    }
}

fn reference(re: &str, prec: usize, len: usize) -> crate::render::ReuseRef {
    use fractadyne_core::parse_bf_prec;
    let bf = |s: &str| parse_bf_prec(s, prec).unwrap();
    crate::render::ReuseRef {
        point: [bf(re), bf("0.1")],
        prefix: std::sync::Arc::new(vec![[0.0; 4]; len]),
        tail: fractadyne_core::OrbitTail {
            zx: bf("0"),
            zy: bf("0"),
            zpx: bf("0"),
            zpy: bf("0"),
            escaped: false,
            period: None,
            backend: 0,
            sched: None,
        },
        prec,
    }
}

/// The chain offers a frame only references of its own formula, the most recent first; a reference
/// that comes back (reused and extended) replaces itself, and it holds `KEEP` dives at most.
#[test]
fn the_chain_keeps_one_reference_a_dive_and_offers_its_own_formulas() {
    let p = pb(r#"
[[keyframe]]
t = 0
zoom = 1e20
fractal = "Mandelbrot"

[[keyframe]]
t = 10
zoom = 1e20
fractal = "Burning Ship"
"#);
    let (m, ship) = (p.sample(0.0), p.sample(10.0));
    let mut chain = RefChain::default();
    chain.remember(&m, reference("-0.75", 128, 4));
    chain.remember(&ship, reference("-1.75", 128, 4));
    chain.remember(&m, reference("-0.5", 128, 4));
    let firsts = |c: &RefChain, s| c.candidates(s).iter().map(|r| fractadyne_core::to_f64(&r.point[0])).collect::<Vec<_>>();
    assert_eq!(firsts(&chain, &m), vec![-0.5, -0.75], "Mandelbrot's, the most recent first");
    assert_eq!(firsts(&chain, &ship), vec![-1.75], "the Burning Ship's alone");
    // The first dive's reference comes back extended (its point and precision, a longer orbit, as
    // `try_reuse_reference` returns it): it replaces itself and goes first.
    chain.remember(&m, reference("-0.75", 128, 9));
    assert_eq!(firsts(&chain, &m), vec![-0.75, -0.5]);
    assert_eq!(chain.candidates(&m)[0].prefix.len(), 9, "the extended one");
    assert_eq!(chain.candidates(&m).len() + chain.candidates(&ship).len(), 3);
    for k in 0..10 {
        chain.remember(&m, reference(&format!("-0.{k}1"), 128, 4));
    }
    assert_eq!(chain.candidates(&m).len() + chain.candidates(&ship).len(), RefChain::KEEP, "at most KEEP dives");
}
