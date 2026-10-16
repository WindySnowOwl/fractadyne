use super::*;

#[test]
fn latex_says_what_the_typeset_form_shows() {
    assert_eq!(latex("z = z^2 + c"), "z = z^{2} + c");
    assert_eq!(latex("z = c/(z + 1)"), "z = \\frac{c}{z + 1}");
    // The truthful notation: |z| is the squared modulus, sqr a square, exp stays exp.
    assert_eq!(latex("|z| + cabs(z)"), "\\left| z \\right|^{2} + \\left| z \\right|");
    assert_eq!(latex("sqr(z + c)"), "\\left( z + c \\right)^{2}");
    assert_eq!(latex("exp(z)"), "\\exp \\left( z \\right)");
    assert_eq!(latex("conj(t)*p1"), "\\overline{t} p_{1}");
    assert_eq!(latex("real(z)"), "\\operatorname{Re} \\left( z \\right)");
    assert_eq!(latex("z*c + pi*pixel"), "z \\cdot c + \\pi \\mathit{pixel}");
    assert_eq!(latex("1e-5*z"), "1 \\times 10^{- 5} z");
    // Several statements align at their `=`; comments go.
    assert_eq!(
        latex("t = sqr(z) ; square\nz = t + c"),
        "\\begin{aligned}\nt &= z^{2} \\\\\nz &= t + c\n\\end{aligned}"
    );
}
