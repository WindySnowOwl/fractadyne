use super::*;

/// Japanese and Chinese each ask for a font of their own typography; the bundled fonts serve the
/// rest.
#[test]
fn each_han_script_asks_for_its_own_font() {
    assert_eq!(Need::for_script("Jpan"), Some(Need::Japanese));
    assert_eq!(Need::for_script("Hans"), Some(Need::ChineseSimplified));
    assert_eq!(Need::for_script("Hant"), Some(Need::ChineseTraditional));
    assert_eq!(Need::for_script("Kore"), Some(Need::Korean));
    assert_eq!(Need::for_script("Latn"), None);
    assert_eq!(Need::for_script("Cyrl"), None);
    let all = [Need::Japanese, Need::ChineseSimplified, Need::ChineseTraditional, Need::Korean];
    for (i, a) in all.iter().enumerate() {
        for b in &all[i + 1..] {
            assert_ne!(a.key(), b.key());
        }
    }
}

/// A font found is one that draws its language: the face chosen holds the probe character. (A
/// machine without the font finds none, which the language switch reports instead of drawing boxes.)
#[test]
fn a_system_font_found_draws_its_language() {
    for (need, probe) in [(Need::Japanese, 'あ'), (Need::ChineseSimplified, '简'), (Need::ChineseTraditional, '體'), (Need::Korean, '한')] {
        if let Some(f) = system_font(need) {
            let face = ttf_parser::Face::parse(&f.data.font[..], f.data.index).expect("the chosen face parses");
            assert!(face.glyph_index(probe).is_some(), "{need:?}: {} lacks {probe}", f.name);
        }
    }
}
