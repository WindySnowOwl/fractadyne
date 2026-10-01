//! The formula editor's textbook mode (design/formula-textbook-editor.md): the formula typeset as
//! LaTeX would set it, by TeX's layout rules driven by the math font's OpenType MATH table.
//!
//! - [`font`]: Fractadyne Math (a Latin Modern Math subset) and its MATH table.
//! - [`layout`]: a TeX-style box layout of a math list.
//! - [`paint`]: drawing a laid-out formula with egui.
//! - [`model`]: the formula as rows of atoms — read from the text, printed back, typeset.
//! - [`edit`]: the document and the editing commands; [`editor`]: the widget.

pub(crate) mod edit;
pub(crate) mod editor;
pub(crate) mod font;
pub(crate) mod layout;
pub(crate) mod model;
pub(crate) mod paint;

use layout::{Ctx, Node};

/// One formula per construct the layout sets — the uitest's `textbook-specimen` screen shows them,
/// so a change to the layout or the font shows up as a picture to look at.
pub(crate) fn specimen() -> Vec<Vec<Node>> {
    use layout::Class;
    let v = Node::var;
    let n = Node::num;
    let sup = |base: Node, s: Vec<Node>| Node::Scripts { base: Box::new(base), sup: Some(s), sub: None };
    let sub = |base: Node, s: Vec<Node>| Node::Scripts { base: Box::new(base), sup: None, sub: Some(s) };
    let frac = |num: Vec<Node>, den: Vec<Node>| Node::Frac { num, den };
    let paren = |body: Vec<Node>| Node::Fenced { open: '(', close: ')', body };
    let (plus, minus, eq) = (Node::bin('+'), Node::bin('\u{2212}'), Node::rel('='));
    let p1 = || sub(v("p"), vec![n("1")]);
    let mut cf = vec![v("z")];
    for _ in 0..3 {
        cf = vec![n("1"), plus.clone(), frac(vec![n("1")], cf)];
    }
    vec![
        vec![v("z"), eq.clone(), sup(v("z"), vec![n("2")]), plus.clone(), v("c")],
        vec![
            v("z"),
            eq.clone(),
            frac(vec![sup(v("z"), vec![n("3")]), minus.clone(), p1(), v("z")], vec![sup(v("z"), vec![n("2")]), plus.clone(), n("1")]),
            plus.clone(),
            v("c"),
        ],
        vec![v("t"), eq.clone(), Node::Radical(vec![sup(v("z"), vec![n("4")]), plus.clone(), v("c")])],
        vec![
            v("w"),
            eq.clone(),
            sup(Node::Fenced { open: '|', close: '|', body: vec![v("z")] }, vec![n("2")]),
            plus.clone(),
            Node::Overline(vec![v("z")]),
            plus.clone(),
            Node::op("Re"),
            paren(vec![v("z")]),
        ],
        vec![
            v("z"),
            eq.clone(),
            Node::op("sin"),
            paren(vec![v("z")]),
            plus.clone(),
            Node::op("exp"),
            paren(vec![frac(vec![n("1")], vec![n("1"), plus.clone(), frac(vec![n("1")], vec![v("z")])])]),
        ],
        vec![v("z"), eq.clone(), sup(paren(cf), vec![n("2")])],
        vec![
            v("z"),
            eq.clone(),
            sup(v("z"), vec![n("2.2"), plus.clone(), n("0.3"), v("i")]),
            plus.clone(),
            Node::Slot(None),
        ],
        vec![
            v("u"),
            eq.clone(),
            n("1"),
            Node::glyphs("\u{00D7}", Class::Bin),
            sup(n("10"), vec![minus, n("5")]),
            v("z"),
        ],
    ]
}

/// A window showing [`specimen`] (the uitest opens it; nothing in the app does).
pub(crate) fn specimen_window(ctx: &egui::Context, open: &mut bool) {
    egui::Window::new("Textbook specimen").open(open).default_pos([40.0, 60.0]).show(ctx, |ui| {
        let laid = layout::rows(&specimen(), &Ctx { size_pt: 20.0, ppp: ctx.pixels_per_point() });
        let (rect, _) = ui.allocate_exact_size(laid.size + egui::vec2(16.0, 16.0), egui::Sense::hover());
        let v = ui.visuals();
        paint::paint(ui.painter(), rect.min + egui::vec2(8.0, 8.0), &laid, v.text_color(), v.weak_text_color());
    });
}
