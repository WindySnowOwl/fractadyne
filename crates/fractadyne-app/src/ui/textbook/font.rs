//! Fractadyne Math — the subset of Latin Modern Math built by `scripts/subset_math_font.py` — and
//! what the layout needs from it: the OpenType MATH table's constants, each glyph's ink box and
//! advance, italic corrections, and the size variants and assembly parts of the stretchy glyphs.
//!
//! Every dimension here is in FONT UNITS (1000 to the em); the layout scales them. egui draws the
//! glyphs; the stretchy ones carry Private Use Area code points (given by the subset script) so that
//! egui, which draws by character, can draw them at all — [`MathFont::char_of`] maps a glyph back.

use std::collections::HashMap;
use std::sync::OnceLock;
use ttf_parser::{Face, GlyphId};

pub(crate) const FONT_BYTES: &[u8] = include_bytes!("../../../assets/fonts/FractadyneMath.otf");
/// The name the font is registered under in egui.
pub(crate) const FONT_NAME: &str = "FractadyneMath";

/// The egui font family that draws with it (and nothing else).
pub(crate) fn family() -> egui::FontFamily {
    egui::FontFamily::Name("math".into())
}

/// The MATH table constants the layout uses, in font units (percentages as they are).
#[derive(Clone, Debug)]
pub(crate) struct Constants {
    pub(crate) script_percent: f32,
    pub(crate) script_script_percent: f32,
    pub(crate) delimited_sub_formula_min_height: f32,
    pub(crate) axis_height: f32,
    pub(crate) subscript_shift_down: f32,
    pub(crate) subscript_top_max: f32,
    pub(crate) subscript_baseline_drop_min: f32,
    pub(crate) superscript_shift_up: f32,
    pub(crate) superscript_shift_up_cramped: f32,
    pub(crate) superscript_bottom_min: f32,
    pub(crate) superscript_baseline_drop_max: f32,
    pub(crate) sub_superscript_gap_min: f32,
    pub(crate) superscript_bottom_max_with_subscript: f32,
    pub(crate) space_after_script: f32,
    pub(crate) fraction_numerator_shift_up: f32,
    pub(crate) fraction_numerator_display_style_shift_up: f32,
    pub(crate) fraction_denominator_shift_down: f32,
    pub(crate) fraction_denominator_display_style_shift_down: f32,
    pub(crate) fraction_numerator_gap_min: f32,
    pub(crate) fraction_num_display_style_gap_min: f32,
    pub(crate) fraction_rule_thickness: f32,
    pub(crate) fraction_denominator_gap_min: f32,
    pub(crate) fraction_denom_display_style_gap_min: f32,
    pub(crate) overbar_vertical_gap: f32,
    pub(crate) overbar_rule_thickness: f32,
    pub(crate) overbar_extra_ascender: f32,
    pub(crate) radical_vertical_gap: f32,
    pub(crate) radical_display_style_vertical_gap: f32,
    pub(crate) radical_rule_thickness: f32,
    pub(crate) radical_extra_ascender: f32,
    pub(crate) min_connector_overlap: f32,
}

/// One part of a stretchy glyph's assembly, bottom to top as the MATH table lists them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Part {
    pub(crate) ch: char,
    pub(crate) start_connector: f32,
    pub(crate) end_connector: f32,
    pub(crate) full_advance: f32,
    pub(crate) extender: bool,
}

pub(crate) struct MathFont {
    face: Face<'static>,
    pub(crate) units_per_em: f32,
    /// Ascender − descender, in font units: what ab_glyph calls the font's height and egui scales
    /// by when it turns a size into pixels (see [`MathFont::rendered_em`]).
    height_unscaled: f32,
    pub(crate) constants: Constants,
    to_char: HashMap<u16, char>,
}

/// The font, parsed once.
pub(crate) fn font() -> &'static MathFont {
    static FONT: OnceLock<MathFont> = OnceLock::new();
    FONT.get_or_init(|| MathFont::parse(FONT_BYTES).expect("the bundled math font parses and has a MATH table"))
}

impl MathFont {
    fn parse(bytes: &'static [u8]) -> Option<MathFont> {
        let face = Face::parse(bytes, 0).ok()?;
        let math = face.tables().math?;
        let c = math.constants?;
        let v = |m: ttf_parser::math::MathValue| m.value as f32;
        let constants = Constants {
            script_percent: c.script_percent_scale_down() as f32,
            script_script_percent: c.script_script_percent_scale_down() as f32,
            delimited_sub_formula_min_height: c.delimited_sub_formula_min_height() as f32,
            axis_height: v(c.axis_height()),
            subscript_shift_down: v(c.subscript_shift_down()),
            subscript_top_max: v(c.subscript_top_max()),
            subscript_baseline_drop_min: v(c.subscript_baseline_drop_min()),
            superscript_shift_up: v(c.superscript_shift_up()),
            superscript_shift_up_cramped: v(c.superscript_shift_up_cramped()),
            superscript_bottom_min: v(c.superscript_bottom_min()),
            superscript_baseline_drop_max: v(c.superscript_baseline_drop_max()),
            sub_superscript_gap_min: v(c.sub_superscript_gap_min()),
            superscript_bottom_max_with_subscript: v(c.superscript_bottom_max_with_subscript()),
            space_after_script: v(c.space_after_script()),
            fraction_numerator_shift_up: v(c.fraction_numerator_shift_up()),
            fraction_numerator_display_style_shift_up: v(c.fraction_numerator_display_style_shift_up()),
            fraction_denominator_shift_down: v(c.fraction_denominator_shift_down()),
            fraction_denominator_display_style_shift_down: v(c.fraction_denominator_display_style_shift_down()),
            fraction_numerator_gap_min: v(c.fraction_numerator_gap_min()),
            fraction_num_display_style_gap_min: v(c.fraction_num_display_style_gap_min()),
            fraction_rule_thickness: v(c.fraction_rule_thickness()),
            fraction_denominator_gap_min: v(c.fraction_denominator_gap_min()),
            fraction_denom_display_style_gap_min: v(c.fraction_denom_display_style_gap_min()),
            overbar_vertical_gap: v(c.overbar_vertical_gap()),
            overbar_rule_thickness: v(c.overbar_rule_thickness()),
            overbar_extra_ascender: v(c.overbar_extra_ascender()),
            radical_vertical_gap: v(c.radical_vertical_gap()),
            radical_display_style_vertical_gap: v(c.radical_display_style_vertical_gap()),
            radical_rule_thickness: v(c.radical_rule_thickness()),
            radical_extra_ascender: v(c.radical_extra_ascender()),
            min_connector_overlap: math.variants.map_or(0.0, |vs| vs.min_connector_overlap as f32),
        };
        let mut to_char = HashMap::new();
        if let Some(cmap) = face.tables().cmap {
            for sub in cmap.subtables {
                if !sub.is_unicode() {
                    continue;
                }
                sub.codepoints(|cp| {
                    if let (Some(ch), Some(g)) = (char::from_u32(cp), sub.glyph_index(cp)) {
                        // The lowest code point wins: the ordinary character over a PUA alias.
                        to_char.entry(g.0).and_modify(|c: &mut char| *c = (*c).min(ch)).or_insert(ch);
                    }
                });
            }
        }
        Some(MathFont {
            units_per_em: face.units_per_em() as f32,
            height_unscaled: (face.ascender() as f32) - (face.descender() as f32),
            face,
            constants,
            to_char,
        })
    }

    pub(crate) fn glyph(&self, ch: char) -> Option<GlyphId> {
        self.face.glyph_index(ch)
    }

    /// The character that draws glyph `g` (a PUA code point for a stretchy variant or part).
    pub(crate) fn char_of(&self, g: GlyphId) -> Option<char> {
        self.to_char.get(&g.0).copied()
    }

    /// Whether the font draws `ch` (the layout emits nothing else).
    #[cfg(test)]
    pub(crate) fn has(&self, ch: char) -> bool {
        self.glyph(ch).is_some()
    }

    pub(crate) fn advance(&self, ch: char) -> f32 {
        self.glyph(ch).and_then(|g| self.face.glyph_hor_advance(g)).unwrap_or(0) as f32
    }

    /// The ink box's height above and depth below the baseline (font units, never negative).
    pub(crate) fn ink(&self, ch: char) -> (f32, f32) {
        match self.glyph(ch).and_then(|g| self.face.glyph_bounding_box(g)) {
            Some(r) => ((r.y_max as f32).max(0.0), (-(r.y_min as f32)).max(0.0)),
            None => (0.0, 0.0),
        }
    }

    pub(crate) fn italic_correction(&self, ch: char) -> f32 {
        let Some(g) = self.glyph(ch) else { return 0.0 };
        self.face
            .tables()
            .math
            .and_then(|m| m.glyph_info)
            .and_then(|gi| gi.italic_corrections)
            .and_then(|ic| ic.get(g))
            .map_or(0.0, |v| v.value as f32)
    }

    /// The vertical size variants of `ch`, smallest first, as (character, full height in font units).
    pub(crate) fn vertical_variants(&self, ch: char) -> Vec<(char, f32)> {
        let Some(c) = self.construction(ch) else { return Vec::new() };
        c.variants
            .into_iter()
            .filter_map(|v| Some((self.char_of(v.variant_glyph)?, v.advance_measurement as f32)))
            .collect()
    }

    /// The parts that build `ch` past its largest variant, bottom to top.
    pub(crate) fn vertical_assembly(&self, ch: char) -> Vec<Part> {
        let Some(a) = self.construction(ch).and_then(|c| c.assembly) else { return Vec::new() };
        a.parts
            .into_iter()
            .filter_map(|p| {
                Some(Part {
                    ch: self.char_of(p.glyph_id)?,
                    start_connector: p.start_connector_length as f32,
                    end_connector: p.end_connector_length as f32,
                    full_advance: p.full_advance as f32,
                    extender: p.part_flags.extender(),
                })
            })
            .collect()
    }

    fn construction(&self, ch: char) -> Option<ttf_parser::math::GlyphConstruction<'static>> {
        let g = self.glyph(ch)?;
        self.face.tables().math?.variants?.vertical_constructions.get(g)
    }

    /// The em size, in points, that egui actually draws a `size_pt` font with at `ppp` physical
    /// pixels per point. egui scales the font so its em is `size_pt`, but rounds the scale it hands
    /// the rasteriser — ascender−descender in whole pixels (epaint `Fonts::font_impl`) — so the em
    /// drawn is a little off the one asked for. The layout measures with this one, or its boxes and
    /// the glyphs drawn in them would disagree by up to half a pixel per em.
    pub(crate) fn rendered_em(&self, size_pt: f32, ppp: f32) -> f32 {
        let k = self.height_unscaled / self.units_per_em;
        let px = (ppp * size_pt * k).round().max(1.0);
        px / k / ppp
    }
}

#[cfg(test)]
#[path = "font_tests.rs"]
mod tests;
