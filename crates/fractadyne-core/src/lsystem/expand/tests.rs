use super::*;
use crate::lsystem::walk::Segment;

fn sys(text: &str) -> LSystem {
    LSystem::parse(text).unwrap_or_else(|e| panic!("{e}\n{text}"))
}

/// The words at orders 0..=n.
fn words(text: &str, n: u32) -> Vec<String> {
    let s = sys(text);
    let e = s.expanded.clone().expect("a parametric system");
    (0..=n).map(|k| expand(&s, &e, k, EXPAND_BUDGET).word.text()).collect()
}

fn segments(s: &LSystem, order: u32) -> Vec<Segment> {
    let e = s.expanded.clone().unwrap();
    let x = expand(s, &e, order, EXPAND_BUDGET);
    let mut out = Vec::new();
    draw(s, &x, &View::EVERYTHING, u64::MAX, &mut |d| {
        if let Drawn::Segment(g) = d {
            out.push(*g)
        }
    });
    out
}

/// ABOP's parametric example, equation 1.7, and the derivation its Figure 1.34 shows.
#[test]
fn the_abop_parametric_example_derives_as_printed() {
    let w = words(
        "angle 90\naxiom B(2)A(4,4)\n\
         A(x,y) : y <= 3 = A(x*2, x+y)\n\
         A(x,y) : y > 3 = B(x)A(x/y, 0)\n\
         B(x) : x < 1 = C\n\
         B(x) : x >= 1 = B(x-1)\n",
        5,
    );
    assert_eq!(w[0], "B(2)A(4,4)");
    assert_eq!(w[1], "B(1)B(4)A(1,0)");
    assert_eq!(w[2], "B(0)B(3)A(2,1)");
    assert_eq!(w[3], "CB(2)A(4,3)");
    assert_eq!(w[4], "CB(1)A(8,7)");
    assert_eq!(w[5], format!("CB(0)B(8)A({},0)", 8.0 / 7.0));
}

/// ABOP §1.10.2's context-sensitive production, applied to `A(4)B(5)C(6)`: B(5) becomes
/// E(4.5)F(5.5); A and C, which no production matches, stay.
#[test]
fn a_parametric_context_binds_its_neighbours_parameters() {
    let w = words("angle 90\naxiom A(4)B(5)C(6)\nA(x) < B(y) > C(z) : x + y + z > 10 = E((x+y)/2)F((y+z)/2)\n", 1);
    assert_eq!(w[1], "A(4)E(4.5)F(5.5)C(6)");
    // The condition false: nothing happens.
    let w = words("angle 90\naxiom A(1)B(2)C(3)\nA(x) < B(y) > C(z) : x + y + z > 10 = E\n", 1);
    assert_eq!(w[1], "A(1)B(2)C(3)");
}

/// ABOP Figure 1.30: a signal moving up a branching structure (left context) and down it (right
/// context). The left context steps over the branches to the left and out of the branch a module
/// is in; the right context steps over branches, and a branch's end has none (the signal coming
/// down never enters a branch).
#[test]
fn signals_travel_through_brackets_as_abop_draws_them() {
    let up = words("angle 45\nignore +-\naxiom b[+a]a[-a]a[+a]a\nb < a = b\n", 3);
    assert_eq!(up[1], "b[+b]b[-a]a[+a]a");
    assert_eq!(up[2], "b[+b]b[-b]b[+a]a");
    assert_eq!(up[3], "b[+b]b[-b]b[+b]b");
    let down = words("angle 45\nignore +-\naxiom a[+a]a[-a]a[+a]b\na > b = b\n", 3);
    assert_eq!(down[1], "a[+a]a[-a]b[+a]b");
    assert_eq!(down[2], "a[+a]b[-a]b[+a]b");
    assert_eq!(down[3], "b[+a]b[-a]b[+a]b");
}

/// Hogeweg and Hesper's plant, ABOP Figure 1.31a, verbatim; its first five words worked by hand
/// from the productions (the `+` and `-` swap every generation).
#[test]
fn the_hogeweg_plant_grows_as_worked_by_hand() {
    let text = "angle 22.5\norder 30\nignore +-F\naxiom F1F1F1\n\
                0 < 0 > 0 = 0\n0 < 0 > 1 = 1[+F1F1]\n0 < 1 > 0 = 1\n0 < 1 > 1 = 1\n\
                1 < 0 > 0 = 0\n1 < 0 > 1 = 1F1\n1 < 1 > 0 = 0\n1 < 1 > 1 = 0\n\
                * < + > * = -\n* < - > * = +\n";
    let w = words(text, 5);
    assert_eq!(&w[1..], ["F1F0F1", "F1F1F1F1", "F1F0F0F1", "F1F0F1[+F1F1]F1", "F1F1F1F1[-F0F1]F1"]);
    // And at the figure's order it draws a plant, within the budget.
    let s = sys(text);
    let x = expand(&s, s.expanded.as_ref().unwrap(), 30, EXPAND_BUDGET);
    assert!(!x.short() && x.segments > 100 && x.max_brackets >= 2, "{} segments, {} brackets", x.segments, x.max_brackets);
}

/// ABOP's "row of trees" (Figure 1.37a), with `define`s that use earlier ones: at a right angle
/// its generator's edges are p, h, h, q with p + q = c, so the curve ends where it would as a
/// straight line — at every order.
#[test]
fn the_row_of_trees_ends_where_its_base_does() {
    let text = "angle 90\ndefine c 1\ndefine p 0.3\ndefine q c - p\ndefine h (p*q)^0.5\naxiom F(1)\n\
                F(x) = F(x*p)+F(x*h)--F(x*h)+F(x*q)\n";
    let s = sys(text);
    for order in 0..6 {
        let segs = segments(&s, order);
        assert_eq!(segs.len(), 4usize.pow(order));
        let end = segs.last().unwrap().b;
        assert!((end[0] - 1.0).abs() < 1e-12 && end[1].abs() < 1e-12, "order {order}: ends at {end:?}");
    }
}

/// ABOP's equations 1.9 and 1.10 make "a structure with identical proportions" — the first
/// appending ever shorter segments, the second lengthening the old ones by R each generation: at
/// order n the second is the first scaled by R^(n−1), segment for segment.
#[test]
fn equations_one_nine_and_one_ten_agree() {
    let a = sys("angle 85\ndefine R 1.456\naxiom A(1)\nA(s) = F(s)[+A(s/R)][-A(s/R)]\n");
    let b = sys("angle 85\ndefine R 1.456\naxiom A\nA = F(1)[+A][-A]\nF(s) = F(s*R)\n");
    for n in 1..9u32 {
        let (sa, sb) = (segments(&a, n), segments(&b, n));
        assert_eq!(sa.len(), sb.len());
        let k = 1.456f64.powi(n as i32 - 1);
        for (x, y) in sa.iter().zip(&sb) {
            for (p, q) in [(x.a, y.a), (x.b, y.b)] {
                assert!((p[0] * k - q[0]).abs() < 1e-9 * k && (p[1] * k - q[1]).abs() < 1e-9 * k, "order {n}: {p:?}·{k} vs {q:?}");
            }
        }
    }
}

/// `F(l)` steps l, `\(a)` and `/(a)` turn by a degrees, `+(a)` too, and `@(f)` scales the step.
#[test]
fn turtle_commands_take_arguments() {
    let s = sys("angle 90\ndefine a 30\naxiom F(2)\\(a)F/(a)+(45)F@(3)F\n");
    let segs = segments(&s, 0);
    let ends: Vec<[f64; 2]> = segs.iter().map(|g| g.b).collect();
    let (c30, s30, r) = (30f64.to_radians().cos(), 0.5, std::f64::consts::FRAC_1_SQRT_2);
    let want = [[2.0, 0.0], [2.0 + c30, s30], [2.0 + c30 + r, s30 + r], [2.0 + c30 + 4.0 * r, s30 + 4.0 * r]];
    assert_eq!(ends.len(), 4);
    for (g, w) in ends.iter().zip(want) {
        assert!((g[0] - w[0]).abs() < 1e-12 && (g[1] - w[1]).abs() < 1e-12, "{ends:?}");
    }
}

#[test]
fn a_word_past_the_budget_stops_at_the_last_order_that_fits() {
    let s = sys("angle 90\ndefine k 1\naxiom F\nF = FF\n");
    let x = expand(&s, s.expanded.as_ref().unwrap(), 30, EXPAND_BUDGET);
    assert!(x.short());
    assert_eq!(x.order, 20, "2^20 modules fit two million, 2^21 do not");
    assert_eq!(x.word.mods.len(), 1 << 20);
}

#[test]
fn a_parametric_system_round_trips_through_its_text() {
    for text in [
        "angle 85\ndefine R 1.456\naxiom A(1)\nA(s) = F(s)[+A(s/R)][-A(s/R)]\n",
        "angle 22.5\nignore +-F\naxiom F1F1F1\n0 < 0 > 1 = 1[+F1F1]\n* < + > * = -\n",
        // ABOP's `→` and `=` for equality in a condition.
        "angle 86\naxiom F(1,0)\nF(x,t) : t = 0 → F(x*0.3,2)+F(x*0.45,1)--F(x*0.45,1)+F(x*0.7,0)\nF(x,t) : t > 0 → F(x,t-1)\n",
    ] {
        let s = sys(text);
        let back = LSystem::parse(&s.to_text()).unwrap_or_else(|e| panic!("{e}\n{}", s.to_text()));
        assert_eq!(back, s, "{}", s.to_text());
    }
    // `#define` and `#ignore:` as ABOP writes them.
    let s = sys("angle 22.5\n#ignore: +-F\n#define x 2\naxiom F1\n1 < 1 = 0\n# a comment, with an = and a < in it\n");
    let e = s.expanded.as_ref().unwrap();
    assert_eq!(e.ignore, b"+-F");
    assert_eq!(e.rules.len(), 1);
}

#[test]
fn refusals_say_what_and_where() {
    let err = |text: &str| LSystem::parse(text).expect_err("refused");
    let e = err("angle 90\naxiom A(1)\nA(x) = F(y)\n");
    assert_eq!((e.line, e.col), (3, 10), "{e}");
    assert!(e.message.contains("'y'"), "{e}");
    // (`A(2) = F` alone is a stochastic weight; among parameters it is refused as one.)
    let e = err("angle 90\naxiom A(1)\nA(x, 3) = F\n");
    assert!(e.message.contains("parameter name"), "{e}");
    let e = err("angle 90\naxiom A(1)\nA(2) = F\nB(x) = F\n");
    assert!(e.message.contains("weighted"), "{e}");
    let e = err("angle 90\naxiom A\nA B : 1 = F\n");
    assert!(e.message.contains("one module"), "{e}");
    let e = err("angle 90\naxiom A(1\nA(x) = F\n");
    assert!(e.message.contains("never closed"), "{e}");
    let e = err("angle 90\ndefine x 1\nA = F\n");
    assert!(e.message.contains("no axiom"), "{e}");
    let e = err("angle 90\naxiom A(1)\nA(x) : x > = F\n");
    assert_eq!(e.line, 3);
    let e = err("angle 90\naxiom A\nA (0.5) = F\nB(x) = F\n");
    assert!(e.message.contains("weighted"), "{e}");
    let e = err("angle 90\naxiom A\n[A] < A = F\n");
    assert!(e.message.contains("bracket"), "{e}");
}
