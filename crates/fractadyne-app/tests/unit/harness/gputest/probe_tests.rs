use super::*;

/// Four plain terms have 15 association orders (5!!), each listed once.
#[test]
fn four_values_have_fifteen_orders() {
    let t = [(Term::V(1.0), "a"), (Term::V(2.0), "b"), (Term::V(3.0), "c"), (Term::V(4.0), "d")];
    let e = evals(&t, 0b1111);
    assert_eq!(e.len(), 15);
    let mut labels: Vec<&str> = e.iter().map(|x| x.label.as_str()).collect();
    labels.sort();
    labels.dedup();
    assert_eq!(labels.len(), 15, "no order listed twice");
    assert!(labels.contains(&"((a+b)+(c+d))"));
}

/// The as-written orders the probe looks up are among the enumerated ones, and a fused product
/// gives the fma's value, not the rounded product's.
#[test]
fn the_written_orders_and_fusion_are_enumerated() {
    let add = [(Term::V(1.0), "ax"), (Term::V(2.0), "bx"), (Term::V(3.0), "ay"), (Term::V(4.0), "by")];
    assert!(evals(&add, 0b1111).iter().any(|e| e.label == "((ax+bx)+(ay+by))"));
    let mul = [(Term::P(1.0, 1.0), "P"), (Term::V(0.0), "PE"), (Term::P(1.0, 2.0), "X"), (Term::P(3.0, 1.0), "Y")];
    assert!(evals(&mul, 0b1111).iter().any(|e| e.label == "(P+(PE+(X+Y)))"));
    // (1 + 2^-12)^2 = 1 + 2^-11 + 2^-24: the rounded product drops the 2^-24 (a tie, to even);
    // fused with -1 it survives (2^-11 + 2^-24 fits in a mantissa).
    let x = 1.0f32 + 1.0 / 4096.0;
    let fused = evals(&[(Term::P(x, x), "P"), (Term::V(-1.0), "m")], 0b11);
    let f = fused.iter().find(|e| e.label == "fma[P](m)").unwrap();
    let r = fused.iter().find(|e| e.label == "(P+m)").unwrap();
    assert_ne!(f.val, r.val, "fusion changes the value here");
    assert_eq!(f.val, x.mul_add(x, -1.0));
}
