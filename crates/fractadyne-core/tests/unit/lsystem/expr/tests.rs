use super::*;

fn ev(text: &str) -> f64 {
    parse(text, &[], &[]).unwrap_or_else(|e| panic!("{text}: {e:?}")).eval(&[])
}

#[test]
fn arithmetic_follows_the_usual_precedence() {
    assert_eq!(ev("1 + 2 * 3"), 7.0);
    assert_eq!(ev("(1 + 2) * 3"), 9.0);
    assert_eq!(ev("2 ^ 3 ^ 2"), 512.0, "right-associative");
    assert_eq!(ev("-2 ^ 2"), -4.0, "the power binds tighter than the minus");
    assert_eq!(ev("2 ^ -1"), 0.5);
    assert_eq!(ev("7 % 4 - 10 / 4"), 0.5);
    assert_eq!(ev("1e-3 * 1000"), 1.0);
    assert_eq!(ev(".5 + 0.25"), 0.75);
}

#[test]
fn comparisons_and_logic_are_one_or_zero() {
    for (text, want) in [
        ("3 <= 3", 1.0),
        ("3 < 3", 0.0),
        ("4 > 3 & 2 > 1", 1.0),
        ("4 > 3 && 2 < 1", 0.0),
        ("0 | 1", 1.0),
        ("0 || 0", 0.0),
        ("!(1 > 2)", 1.0),
        ("2 = 2", 1.0),
        ("2 == 3", 0.0),
        ("2 != 3", 1.0),
        ("1 + 1 == 2", 1.0),
    ] {
        assert_eq!(ev(text), want, "{text}");
    }
}

#[test]
fn trigonometry_is_in_degrees() {
    assert!((ev("sin(30)") - 0.5).abs() < 1e-15);
    assert!((ev("cos(60)") - 0.5).abs() < 1e-15);
    assert!((ev("atan2(1, 1)") - 45.0).abs() < 1e-12);
    assert_eq!(ev("max(2, min(5, 3))"), 3.0);
    assert_eq!(ev("sqrt(16) + abs(-1) + floor(2.7) + sign(-3)"), 6.0);
}

#[test]
fn names_are_parameters_then_constants() {
    let vars = ["x".to_string(), "y".to_string()];
    let consts = [("R".to_string(), 1.5), ("x".to_string(), 99.0)];
    let e = parse("x * R + y", &vars, &consts).unwrap();
    assert_eq!(e.eval(&[2.0, 1.0]), 4.0, "the parameter x shadows the constant x");
    assert!(parse("x > 3 & y <= 3", &vars, &consts).unwrap().holds(&[4.0, 3.0]));
    assert!(!parse("x / y > 0", &vars, &consts).unwrap().holds(&[0.0, 0.0]), "NaN does not hold");
}

#[test]
fn refusals_say_where() {
    let e = parse("x + ", &["x".into()], &[]).unwrap_err();
    assert!(e.message.contains("ends"), "{e:?}");
    let e = parse("2 * q", &[], &[]).unwrap_err();
    assert_eq!(e.at, 4);
    assert!(e.message.contains("'q'"), "{e:?}");
    let e = parse("frob(2)", &[], &[]).unwrap_err();
    assert!(e.message.contains("no function"), "{e:?}");
    let e = parse("min(2)", &[], &[]).unwrap_err();
    assert!(e.message.contains("2 arguments"), "{e:?}");
    let e = parse("(1 + 2", &[], &[]).unwrap_err();
    assert!(e.message.contains("never closed"), "{e:?}");
    let e = parse("1 2", &[], &[]).unwrap_err();
    assert_eq!(e.at, 2);
    let deep = format!("{}1{}", "(".repeat(200), ")".repeat(200));
    assert!(parse(&deep, &[], &[]).unwrap_err().message.contains("deeply"));
    let deep = format!("{}1", "-".repeat(200));
    assert!(parse(&deep, &[], &[]).unwrap_err().message.contains("deeply"));
}
