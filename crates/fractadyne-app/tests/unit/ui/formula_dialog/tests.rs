use super::clipboard_text;
use fractadyne_core::ir::parse::parse;

#[test]
fn a_formula_without_parameters_copies_as_written() {
    assert_eq!(clipboard_text("z = z^2 + c\n\n", &[]), "z = z^2 + c");
    assert_eq!(clipboard_text("z = 1: z = cosh(z)/sinh(z^2) + c", &[]), "z = 1: z = cosh(z)/sinh(z^2) + c");
}

#[test]
fn its_parameters_copy_as_comments_and_the_text_reads_back_as_the_same_formula() {
    let src = "z = z^2 + p1*c + p2";
    let params = [("0.5".to_string(), " -0.25 ".to_string()), ("1".to_string(), String::new())];
    let text = clipboard_text(src, &params);
    assert_eq!(text, "z = z^2 + p1*c + p2\n; p1 = 0.5, -0.25\n; p2 = 1, 0");
    assert_eq!(parse(&text).expect("the copy reads"), parse(src).expect("the source reads"));
}
