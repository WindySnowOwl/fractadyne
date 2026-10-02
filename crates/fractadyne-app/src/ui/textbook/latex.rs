//! Copy as LaTeX (design §4.4): the formula as LaTeX source for a forum post or a paper. Written
//! from the typeset form ([`model::math_row`]), not from the text, so the two cannot disagree: the
//! same truthful notation (|𝑧|² for `|z|`, exp for `exp`), the same parentheses, the same dots.

use super::edit::{Doc, Shown};
use super::layout::{Class, Node};
use super::model;

/// The formula as LaTeX: one statement as is, several in an `aligned` block, aligned at their `=`.
/// Comments and lines that do not read are left out.
pub(crate) fn latex(src: &str) -> String {
    let doc = Doc::read(src);
    let rows: Vec<Vec<Node>> = doc
        .shown()
        .into_iter()
        .filter_map(|s| match s {
            Shown::Stmt { row, .. } if !row.is_empty() => Some(model::math_row(row)),
            _ => None,
        })
        .collect();
    match rows.len() {
        0 => String::new(),
        1 => list(&rows[0], false),
        _ => {
            let body: Vec<String> = rows.iter().map(|r| list(r, true)).collect();
            format!("\\begin{{aligned}}\n{}\n\\end{{aligned}}", body.join(" \\\\\n"))
        }
    }
}

/// A math list; `align`: an `&` before its first relation.
fn list(nodes: &[Node], align: bool) -> String {
    let mut parts = Vec::new();
    let mut aligned = !align;
    for n in nodes {
        let mut t = node(n);
        if !aligned && matches!(n, Node::Glyphs { class: Class::Rel, .. }) {
            t.insert(0, '&');
            aligned = true;
        }
        if !t.is_empty() {
            parts.push(t);
        }
    }
    // Spaces are nothing in math mode; between items they keep a control word off a letter.
    parts.join(" ")
}

/// `{…}` round a list of more than one item, as a script or an argument needs.
fn group(nodes: &[Node]) -> String {
    format!("{{{}}}", list(nodes, false))
}

/// The operators LaTeX names; any other is `\operatorname`.
const NAMED: [&str; 10] = ["sin", "cos", "tan", "sinh", "cosh", "tanh", "exp", "log", "cot", "coth"];

fn glyph(ch: char) -> String {
    match ch {
        '\u{210E}' => "h".into(),
        '\u{1D434}'..='\u{1D44D}' => char::from_u32(ch as u32 - 0x1D434 + 'A' as u32).map_or(String::new(), String::from),
        '\u{1D44E}'..='\u{1D467}' => char::from_u32(ch as u32 - 0x1D44E + 'a' as u32).map_or(String::new(), String::from),
        '\u{1D70B}' => "\\pi".into(),
        '\u{2212}' => "-".into(),
        '\u{22C5}' => "\\cdot".into(),
        '\u{00D7}' => "\\times".into(),
        c => c.to_string(),
    }
}

fn node(n: &Node) -> String {
    match n {
        Node::Glyphs { text, class: Class::Op, .. } => {
            if NAMED.contains(&text.as_str()) {
                format!("\\{text}")
            } else {
                format!("\\operatorname{{{text}}}")
            }
        }
        Node::Glyphs { text, .. } => {
            let italic = |c: char| matches!(c, '\u{210E}' | '\u{1D434}'..='\u{1D467}');
            let s: String = text.chars().map(glyph).collect();
            // A name of several letters is one name, not a product of letters.
            if text.chars().count() > 1 && text.chars().all(italic) {
                format!("\\mathit{{{s}}}")
            } else {
                s
            }
        }
        Node::Frac { num, den } => format!("\\frac{}{}", group(num), group(den)),
        Node::Scripts { base, sup, sub } => {
            // A base of one symbol or one bracketed group takes its scripts as it is; anything
            // else is braced, so the script is on all of it.
            let sole = match &**base {
                Node::Row(ns) => {
                    let mut real = ns.iter().filter(|n| !matches!(n, Node::Mark(_)));
                    match (real.next(), real.next()) {
                        (Some(only), None) => only,
                        _ => &**base,
                    }
                }
                b => b,
            };
            let mut s = match sole {
                Node::Glyphs { .. } | Node::Fenced { .. } => node(sole),
                b => format!("{{{}}}", node(b)),
            };
            if let Some(b) = sub {
                s.push_str(&format!("_{}", group(b)));
            }
            if let Some(p) = sup {
                s.push_str(&format!("^{}", group(p)));
            }
            s
        }
        Node::Fenced { open, close, body } => {
            let side = |c: char| if c == '|' { "|".to_string() } else { c.to_string() };
            format!("\\left{} {} \\right{}", side(*open), list(body, false), side(*close))
        }
        Node::Radical(body) => format!("\\sqrt{}", group(body)),
        Node::Overline(body) => format!("\\overline{}", group(body)),
        Node::Row(ns) => list(ns, false),
        Node::Slot(_) => "\\square".into(),
        Node::Mark(_) => String::new(),
    }
}

#[cfg(test)]
mod tests {
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
}
