//! Palette tags: words a palette can be found by — its colors, warm or cool, calm or frantic,
//! dark or light, smooth or banded, a theme, and whether it is built for reading structure or for
//! looks.
//!
//! ⭐**Most tags are MEASURED, not assigned.** Color, temperature, energy, brightness and shape
//! come from the gradient itself (`computed_tags`), so they are consistent across the built-in
//! palettes and apply to a gradient the user saved or imported without anyone tagging it. Theme
//! and purpose are judgments, so the built-in palettes carry them by hand (`PRESET_TAGS`) and a
//! user adds them to their own.
//!
//! The measures run in OKLab on a 256-entry table baked like the app's. Their thresholds were set
//! against the first fifty built-in palettes, each placed in a clear gap between values, so a tiny change
//! in arithmetic cannot flip a palette across one (`scratchpad` prototype `tags_proto.py`, kept in
//! step with this file by `computed_tags_match_the_reference`).

use crate::segment::Gradient;

/// What a tag describes. Each group is one row of filters.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TagGroup {
    Colour,
    Temperature,
    Energy,
    Brightness,
    Shape,
    Theme,
    Purpose,
}

impl TagGroup {
    /// Every group, in the order the filters show them.
    pub const ALL: [TagGroup; 7] = [
        TagGroup::Colour,
        TagGroup::Temperature,
        TagGroup::Energy,
        TagGroup::Brightness,
        TagGroup::Shape,
        TagGroup::Theme,
        TagGroup::Purpose,
    ];

    /// Groups the gradient's colors decide; the others are given by hand.
    pub fn is_measured(self) -> bool {
        !matches!(self, TagGroup::Theme | TagGroup::Purpose)
    }
}

/// One tag of the shared vocabulary. The `id` is what a file stores; the app shows a translated
/// name for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TagDef {
    pub id: &'static str,
    pub group: TagGroup,
}

const fn t(id: &'static str, group: TagGroup) -> TagDef {
    TagDef { id, group }
}

/// The shared vocabulary, in display order.
pub const TAGS: &[TagDef] = &[
    t("reds", TagGroup::Colour),
    t("oranges", TagGroup::Colour),
    t("yellows", TagGroup::Colour),
    t("greens", TagGroup::Colour),
    t("blues", TagGroup::Colour),
    t("purples", TagGroup::Colour),
    t("pinks", TagGroup::Colour),
    t("browns", TagGroup::Colour),
    t("neutrals", TagGroup::Colour),
    t("warm", TagGroup::Temperature),
    t("cool", TagGroup::Temperature),
    t("calm", TagGroup::Energy),
    t("lively", TagGroup::Energy),
    t("frantic", TagGroup::Energy),
    t("dark", TagGroup::Brightness),
    t("light", TagGroup::Brightness),
    t("pastel", TagGroup::Brightness),
    t("vivid", TagGroup::Brightness),
    t("smooth", TagGroup::Shape),
    t("banded", TagGroup::Shape),
    t("single-hue", TagGroup::Shape),
    t("rainbow", TagGroup::Shape),
    t("holidays", TagGroup::Theme),
    t("seasons", TagGroup::Theme),
    t("nature", TagGroup::Theme),
    t("retro", TagGroup::Theme),
    t("art", TagGroup::Theme),
    t("materials", TagGroup::Theme),
    t("science", TagGroup::Theme),
    t("food", TagGroup::Theme),
    t("minerals", TagGroup::Theme),
    t("plants", TagGroup::Theme),
    t("animals", TagGroup::Theme),
    t("biomes", TagGroup::Theme),
    t("places", TagGroup::Theme),
    t("analytical", TagGroup::Purpose),
    t("aesthetic", TagGroup::Purpose),
];

/// The vocabulary entry for `id`, if it is one.
pub fn tag_def(id: &str) -> Option<&'static TagDef> {
    TAGS.iter().find(|d| d.id == id)
}

/// ⭐**Theme and purpose for the built-in palettes, by hand.** `analytical`: built to read the
/// structure — lightness in order with the iteration count, or hues that mark bands evenly.
/// `aesthetic`: chosen for how it looks. A palette can be both.
pub const PRESET_TAGS: &[(&str, &[&str])] = &[
    ("ember", &["nature", "analytical", "aesthetic"]),
    ("ice", &["nature", "analytical", "aesthetic"]),
    ("nebula", &["nature", "science", "aesthetic"]),
    ("grayscale", &["science", "analytical"]),
    ("four-seasons", &["seasons", "nature", "aesthetic"]),
    ("royalty", &["art", "aesthetic"]),
    ("rave", &["retro", "aesthetic"]),
    ("emerald-isle", &["nature", "places", "aesthetic"]),
    ("seventies", &["retro", "aesthetic"]),
    ("eighties", &["retro", "aesthetic"]),
    ("mondrian", &["art", "aesthetic"]),
    ("dragon-scales", &["materials", "aesthetic"]),
    ("machine-age", &["materials", "retro", "aesthetic"]),
    ("bronze-age", &["materials", "aesthetic"]),
    ("unity", &["science", "analytical"]),
    ("spice", &["food", "aesthetic"]),
    ("veggies", &["food", "aesthetic"]),
    ("mosaic", &["art", "aesthetic"]),
    ("coral-reef", &["nature", "biomes", "aesthetic"]),
    ("aurora", &["nature", "aesthetic"]),
    ("desert", &["nature", "biomes", "aesthetic"]),
    ("candy", &["food", "aesthetic"]),
    ("blueprint", &["science", "analytical"]),
    ("sepia", &["retro", "analytical", "aesthetic"]),
    ("thermal", &["science", "analytical"]),
    ("stained-glass", &["art", "materials", "aesthetic"]),
    ("noir", &["retro", "art", "aesthetic"]),
    ("cherry-blossom", &["seasons", "nature", "plants", "aesthetic"]),
    ("black-opal", &["materials", "minerals", "aesthetic"]),
    ("lavender-fields", &["seasons", "nature", "plants", "aesthetic"]),
    ("lunar-new-year", &["holidays", "aesthetic"]),
    ("valentines-day", &["holidays", "aesthetic"]),
    ("mardi-gras", &["holidays", "aesthetic"]),
    ("holi", &["holidays", "aesthetic"]),
    ("nowruz", &["holidays", "aesthetic"]),
    ("eid-al-fitr", &["holidays", "aesthetic"]),
    ("easter", &["holidays", "aesthetic"]),
    ("vesak", &["holidays", "aesthetic"]),
    ("midsummer", &["holidays", "seasons", "aesthetic"]),
    ("matariki", &["holidays", "aesthetic"]),
    ("mid-autumn", &["holidays", "seasons", "aesthetic"]),
    ("meskel", &["holidays", "aesthetic"]),
    ("halloween", &["holidays", "aesthetic"]),
    ("day-of-the-dead", &["holidays", "aesthetic"]),
    ("diwali", &["holidays", "aesthetic"]),
    ("thanksgiving", &["holidays", "aesthetic"]),
    ("hanukkah", &["holidays", "aesthetic"]),
    ("christmas", &["holidays", "aesthetic"]),
    ("kwanzaa", &["holidays", "aesthetic"]),
    ("new-years-eve", &["holidays", "aesthetic"]),
    ("amethyst", &["nature", "minerals", "aesthetic"]),
    ("malachite", &["nature", "minerals", "aesthetic"]),
    ("lapis-lazuli", &["nature", "minerals", "aesthetic"]),
    ("turquoise", &["nature", "minerals", "aesthetic"]),
    ("obsidian", &["nature", "minerals", "aesthetic"]),
    ("rose-quartz", &["nature", "minerals", "aesthetic"]),
    ("tigers-eye", &["nature", "minerals", "aesthetic"]),
    ("azurite", &["nature", "minerals", "aesthetic"]),
    ("rhodochrosite", &["nature", "minerals", "aesthetic"]),
    ("labradorite", &["nature", "minerals", "aesthetic"]),
    ("moonstone", &["nature", "minerals", "aesthetic"]),
    ("jade", &["nature", "minerals", "aesthetic"]),
    ("marble", &["nature", "minerals", "aesthetic"]),
    ("granite", &["nature", "minerals", "aesthetic"]),
    ("agate", &["nature", "minerals", "aesthetic"]),
    ("fluorite", &["nature", "minerals", "aesthetic"]),
    ("bismuth", &["nature", "minerals", "aesthetic"]),
    ("sulfur", &["nature", "minerals", "aesthetic"]),
    ("cinnabar", &["nature", "minerals", "aesthetic"]),
    ("pyrite", &["nature", "minerals", "aesthetic"]),
    ("sunflower", &["nature", "plants", "aesthetic"]),
    ("lotus", &["nature", "plants", "aesthetic"]),
    ("moss", &["nature", "plants", "aesthetic"]),
    ("bamboo", &["nature", "plants", "aesthetic"]),
    ("maple", &["nature", "plants", "aesthetic"]),
    ("ginkgo", &["nature", "plants", "aesthetic"]),
    ("wisteria", &["nature", "plants", "aesthetic"]),
    ("poppy", &["nature", "plants", "aesthetic"]),
    ("iris", &["nature", "plants", "aesthetic"]),
    ("orchid", &["nature", "plants", "aesthetic"]),
    ("cactus-flower", &["nature", "plants", "aesthetic"]),
    ("succulent", &["nature", "plants", "aesthetic"]),
    ("tulip-fields", &["nature", "plants", "aesthetic"]),
    ("redwood", &["nature", "plants", "aesthetic"]),
    ("olive-grove", &["nature", "plants", "aesthetic"]),
    ("fly-agaric", &["nature", "plants", "aesthetic"]),
    ("chanterelle", &["nature", "plants", "aesthetic"]),
    ("foxfire", &["nature", "plants", "aesthetic"]),
    ("turkey-tail", &["nature", "plants", "aesthetic"]),
    ("lichen", &["nature", "plants", "aesthetic"]),
    ("peacock", &["nature", "animals", "aesthetic"]),
    ("flamingo", &["nature", "animals", "aesthetic"]),
    ("tiger", &["nature", "animals", "aesthetic"]),
    ("zebra", &["nature", "animals", "aesthetic"]),
    ("poison-dart-frog", &["nature", "animals", "aesthetic"]),
    ("monarch", &["nature", "animals", "aesthetic"]),
    ("blue-morpho", &["nature", "animals", "aesthetic"]),
    ("koi", &["nature", "animals", "aesthetic"]),
    ("mandarin-duck", &["nature", "animals", "aesthetic"]),
    ("scarlet-macaw", &["nature", "animals", "aesthetic"]),
    ("toucan", &["nature", "animals", "aesthetic"]),
    ("chameleon", &["nature", "animals", "aesthetic"]),
    ("jellyfish", &["nature", "animals", "aesthetic"]),
    ("hummingbird", &["nature", "animals", "aesthetic"]),
    ("snow-leopard", &["nature", "animals", "aesthetic"]),
    ("clownfish", &["nature", "animals", "aesthetic"]),
    ("fireflies", &["nature", "animals", "aesthetic"]),
    ("kingfisher", &["nature", "animals", "aesthetic"]),
    ("honeybee", &["nature", "animals", "aesthetic"]),
    ("scarab", &["nature", "animals", "aesthetic"]),
    ("coffee", &["food", "aesthetic"]),
    ("chocolate", &["food", "aesthetic"]),
    ("citrus", &["food", "aesthetic"]),
    ("watermelon", &["food", "aesthetic"]),
    ("matcha", &["food", "aesthetic"]),
    ("sushi", &["food", "aesthetic"]),
    ("berries", &["food", "aesthetic"]),
    ("red-wine", &["food", "aesthetic"]),
    ("honey", &["food", "aesthetic"]),
    ("peach", &["food", "aesthetic"]),
    ("avocado", &["food", "aesthetic"]),
    ("pomegranate", &["food", "aesthetic"]),
    ("neapolitan", &["food", "aesthetic"]),
    ("pistachio", &["food", "aesthetic"]),
    ("lemon-meringue", &["food", "aesthetic"]),
    ("dragon-fruit", &["food", "aesthetic"]),
    ("kiwi", &["food", "aesthetic"]),
    ("fig", &["food", "aesthetic"]),
    ("blood-orange", &["food", "aesthetic"]),
    ("bubble-tea", &["food", "aesthetic"]),
    ("botticelli", &["art", "aesthetic"]),
    ("rembrandt", &["art", "aesthetic"]),
    ("vermeer", &["art", "aesthetic"]),
    ("hokusai", &["art", "aesthetic"]),
    ("hiroshige", &["art", "aesthetic"]),
    ("wang-ximeng", &["art", "aesthetic"]),
    ("behzad", &["art", "aesthetic"]),
    ("turner", &["art", "aesthetic"]),
    ("monet", &["art", "aesthetic"]),
    ("van-gogh", &["art", "aesthetic"]),
    ("seurat", &["art", "aesthetic"]),
    ("gauguin", &["art", "aesthetic"]),
    ("rousseau", &["art", "aesthetic"]),
    ("klimt", &["art", "aesthetic"]),
    ("munch", &["art", "aesthetic"]),
    ("mucha", &["art", "aesthetic"]),
    ("kandinsky", &["art", "aesthetic"]),
    ("hilma-af-klint", &["art", "aesthetic"]),
    ("amrita-sher-gil", &["art", "aesthetic"]),
    ("torres-garcia", &["art", "aesthetic"]),
    ("rainforest", &["nature", "biomes", "aesthetic"]),
    ("cloud-forest", &["nature", "biomes", "aesthetic"]),
    ("mangrove", &["nature", "biomes", "aesthetic"]),
    ("savanna", &["nature", "biomes", "aesthetic"]),
    ("prairie", &["nature", "biomes", "aesthetic"]),
    ("taiga", &["nature", "biomes", "aesthetic"]),
    ("tundra", &["nature", "biomes", "aesthetic"]),
    ("glacier", &["nature", "biomes", "aesthetic"]),
    ("alpine", &["nature", "biomes", "aesthetic"]),
    ("heath", &["nature", "biomes", "aesthetic"]),
    ("chaparral", &["nature", "biomes", "aesthetic"]),
    ("wetland", &["nature", "biomes", "aesthetic"]),
    ("kelp-forest", &["nature", "biomes", "aesthetic"]),
    ("lagoon", &["nature", "biomes", "aesthetic"]),
    ("deep-sea", &["nature", "biomes", "aesthetic"]),
    ("hydrothermal-vent", &["nature", "biomes", "aesthetic"]),
    ("volcano", &["nature", "biomes", "aesthetic"]),
    ("canyon", &["nature", "biomes", "aesthetic"]),
    ("salt-flat", &["nature", "biomes", "aesthetic"]),
    ("cave", &["nature", "biomes", "aesthetic"]),
    ("paris", &["places", "aesthetic"]),
    ("venice", &["places", "aesthetic"]),
    ("santorini", &["places", "aesthetic"]),
    ("chefchaouen", &["places", "aesthetic"]),
    ("marrakech", &["places", "aesthetic"]),
    ("jaipur", &["places", "aesthetic"]),
    ("istanbul", &["places", "aesthetic"]),
    ("taj-mahal", &["places", "aesthetic"]),
    ("petra", &["places", "aesthetic"]),
    ("giza", &["places", "aesthetic"]),
    ("kyoto", &["places", "aesthetic"]),
    ("tokyo", &["places", "aesthetic"]),
    ("forbidden-city", &["places", "aesthetic"]),
    ("angkor-wat", &["places", "aesthetic"]),
    ("machu-picchu", &["places", "aesthetic"]),
    ("havana", &["places", "aesthetic"]),
    ("burano", &["places", "aesthetic"]),
    ("new-york", &["places", "aesthetic"]),
    ("golden-gate", &["places", "aesthetic"]),
    ("st-basils", &["places", "aesthetic"]),
];

/// Every tag of built-in palette `i` (its position in [`crate::PRESETS`]): measured ones first,
/// then its theme and purpose, in vocabulary order. Measured once per run.
pub fn preset_tags(i: usize) -> &'static [&'static str] {
    static ALL: std::sync::OnceLock<Vec<Vec<&'static str>>> = std::sync::OnceLock::new();
    let all = ALL.get_or_init(|| {
        crate::PRESETS
            .iter()
            .map(|p| {
                let mut tags = computed_tags(&Gradient::from_stops(p.name, p.stops));
                if let Some((_, hand)) = PRESET_TAGS.iter().find(|(id, _)| *id == p.id) {
                    tags.extend(hand.iter().copied());
                }
                in_vocabulary_order(tags)
            })
            .collect()
    });
    all.get(i).map(Vec::as_slice).unwrap_or(&[])
}

/// `tags` sorted into vocabulary order, duplicates dropped.
fn in_vocabulary_order(mut tags: Vec<&'static str>) -> Vec<&'static str> {
    let rank = |id: &str| TAGS.iter().position(|d| d.id == id).unwrap_or(usize::MAX);
    tags.sort_by_key(|id| rank(id));
    tags.dedup();
    tags
}

// ------------------------------------------------------------------------------------- measures

const N: usize = 256;
/// Chroma below which a color reads as black, white or gray.
const NEUTRAL_C: f64 = 0.035;
/// Below this lightness a color reads as black whatever its tint.
const BLACK_L: f64 = 0.18;

/// The measured tags a gradient earns (color, temperature, energy, brightness, shape), in
/// vocabulary order.
pub fn computed_tags(g: &Gradient) -> Vec<&'static str> {
    let m = Measures::of(g);
    let mut out = Vec::new();
    for (k, id) in FAMILIES.iter().enumerate() {
        let need = if *id == "neutrals" { 0.35 } else { 0.15 };
        if m.share[k] >= need {
            out.push(*id);
        }
    }
    let tot = m.warm + m.cool;
    if tot > 1e-9 && m.chrom_share >= 0.4 {
        if m.warm / tot >= 0.7 {
            out.push("warm");
        } else if m.cool / tot >= 0.7 {
            out.push("cool");
        }
    }
    // ⭐**Energy is reversals, not distance.** A gradient that sweeps steadily through many colors
    // (Unity, a single black-to-white ramp) is not busy; one whose lightness keeps swinging, whose
    // hue keeps doubling back, is. Calm below 2.7 (the 50 presets leave a gap from 2.5 to 2.8);
    // frantic at six lightness swings or an energy of 9 (a gap from 8.2 to 9.8).
    let energy = m.l_swings as f64 + m.hue_turns as f64 + m.hue_travel;
    out.push(if m.l_swings >= 6 || energy >= 9.0 {
        "frantic"
    } else if energy < 2.7 {
        "calm"
    } else {
        "lively"
    });
    if m.dark_share >= 0.45 {
        out.push("dark");
    } else if m.mean_l >= 0.72 {
        out.push(if m.chroma_of_chromatic < 0.13 { "pastel" } else { "light" });
    }
    if m.p75_chroma >= 0.172 {
        out.push("vivid");
    }
    out.push(if m.hard_edges >= 3 { "banded" } else { "smooth" });
    if m.hue_coherence >= 0.93 || m.share[NEUTRALS] >= 0.9 {
        out.push("single-hue");
    }
    if m.hue_sectors >= 9 {
        out.push("rainbow");
    }
    in_vocabulary_order(out)
}

const FAMILIES: [&str; 9] =
    ["reds", "oranges", "yellows", "greens", "blues", "purples", "pinks", "browns", "neutrals"];
const NEUTRALS: usize = 8;

fn family(l: f64, c: f64, h: f64) -> usize {
    let at = |id: &str| FAMILIES.iter().position(|f| *f == id).unwrap_or(NEUTRALS);
    if c < NEUTRAL_C || l < BLACK_L {
        return NEUTRALS;
    }
    if l < 0.52 && c < 0.16 && (25.0..100.0).contains(&h) {
        return at("browns");
    }
    if l >= 0.70 && (h >= 320.0 || h < 30.0) {
        return at("pinks");
    }
    at(match h {
        h if h < 40.0 || h >= 350.0 => "reds",
        h if h < 75.0 => "oranges",
        h if h < 115.0 => "yellows",
        h if h < 175.0 => "greens",
        h if h < 270.0 => "blues",
        h if h < 330.0 => "purples",
        _ => "pinks",
    })
}

struct Measures {
    share: [f64; 9],
    chrom_share: f64,
    warm: f64,
    cool: f64,
    l_swings: u32,
    hue_turns: u32,
    hue_travel: f64,
    dark_share: f64,
    mean_l: f64,
    chroma_of_chromatic: f64,
    p75_chroma: f64,
    hard_edges: u32,
    hue_coherence: f64,
    hue_sectors: u32,
}

impl Measures {
    fn of(g: &Gradient) -> Self {
        let lab: Vec<[f64; 3]> = (0..N)
            .map(|i| {
                let c = g.eval((i as f32 + 0.5) / N as f32);
                oklab([c[0] as f64, c[1] as f64, c[2] as f64])
            })
            .collect();
        let l: Vec<f64> = lab.iter().map(|p| p[0]).collect();
        let c: Vec<f64> = lab.iter().map(|p| p[1].hypot(p[2])).collect();
        let h: Vec<f64> = lab.iter().map(|p| (p[2].atan2(p[1]).to_degrees() + 360.0) % 360.0).collect();
        let fam: Vec<usize> = (0..N).map(|i| family(l[i], c[i], h[i])).collect();

        let mut share = [0.0; 9];
        for f in &fam {
            share[*f] += 1.0 / N as f64;
        }
        let is_warm = |f: usize| matches!(FAMILIES[f], "reds" | "oranges" | "yellows" | "pinks" | "browns");
        let is_cool = |f: usize| matches!(FAMILIES[f], "greens" | "blues" | "purples");
        let warm: f64 = (0..N).filter(|i| is_warm(fam[*i])).map(|i| c[i]).sum();
        let cool: f64 = (0..N).filter(|i| is_cool(fam[*i])).map(|i| c[i]).sum();
        let chromatic: Vec<usize> = (0..N).filter(|i| c[*i] >= NEUTRAL_C).collect();
        let chrom_share = chromatic.len() as f64 / N as f64;
        let chroma_of_chromatic = if chromatic.is_empty() {
            0.0
        } else {
            chromatic.iter().map(|i| c[*i]).sum::<f64>() / chromatic.len() as f64
        };
        let hue_coherence = if chromatic.len() >= 4 {
            let (mut x, mut y, mut w) = (0.0, 0.0, 0.0);
            for i in &chromatic {
                let a = h[*i].to_radians();
                x += c[*i] * a.cos();
                y += c[*i] * a.sin();
                w += c[*i];
            }
            x.hypot(y) / w
        } else {
            1.0
        };
        let hard_edges = (0..N)
            .filter(|i| {
                let (a, b) = (lab[*i], lab[(*i + 1) % N]);
                ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() > 0.12
            })
            .count() as u32;
        let mut sector_counts = [0u32; 12];
        for i in 0..N {
            if c[i] >= 0.05 && l[i] >= BLACK_L {
                sector_counts[((h[i] / 30.0) as usize).min(11)] += 1;
            }
        }
        let hue_sectors = sector_counts.iter().filter(|n| **n as f64 >= 0.03 * N as f64).count() as u32;
        let mut sorted = c.clone();
        sorted.sort_by(f64::total_cmp);
        let rank = 0.75 * (N - 1) as f64;
        let (lo, frac) = (rank.floor() as usize, rank.fract());
        let p75_chroma = sorted[lo] + frac * (sorted[(lo + 1).min(N - 1)] - sorted[lo]);
        let (hue_turns, hue_travel) = hue_reversals(&h, &c, &l, 45.0);
        Self {
            share,
            chrom_share,
            warm,
            cool,
            l_swings: swings(&l, 0.15),
            hue_turns,
            hue_travel,
            dark_share: l.iter().filter(|v| **v < 0.45).count() as f64 / N as f64,
            mean_l: l.iter().sum::<f64>() / N as f64,
            chroma_of_chromatic,
            p75_chroma,
            hard_edges,
            hue_coherence,
            hue_sectors,
        }
    }
}

/// sRGB-encoded display color (0..1) to OKLab.
fn oklab(rgb: [f64; 3]) -> [f64; 3] {
    let lin = |v: f64| {
        let v = v.clamp(0.0, 1.0);
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    let (r, g, b) = (lin(rgb[0]), lin(rgb[1]), lin(rgb[2]));
    let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
    [
        0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    ]
}

/// Lightness turns of at least `prom` around the ring, walked from its darkest entry: a peak and
/// a trough count one each.
fn swings(l: &[f64], prom: f64) -> u32 {
    let n = l.len();
    let k0 = (0..n).min_by(|a, b| l[*a].total_cmp(&l[*b])).unwrap_or(0);
    let (mut count, mut up, mut ext) = (0u32, true, l[k0]);
    for j in 1..n {
        let x = l[(k0 + j) % n];
        if up {
            if x > ext {
                ext = x;
            } else if ext - x >= prom {
                count += 1;
                up = false;
                ext = x;
            }
        } else if x < ext {
            ext = x;
        } else if x - ext >= prom {
            count += 1;
            up = true;
            ext = x;
        }
    }
    count + u32::from(!up)
}

/// Turns of the hue's direction of at least `prom` degrees, over the chromatic entries in ring
/// order, and the total hue traveled in whole turns of the wheel.
fn hue_reversals(h: &[f64], c: &[f64], l: &[f64], prom: f64) -> (u32, f64) {
    let idx: Vec<usize> = (0..h.len()).filter(|i| c[*i] >= 0.05 && l[*i] >= BLACK_L).collect();
    if idx.len() < 8 {
        return (0, 0.0);
    }
    // Unwrap the hue (numpy's `unwrap` on radians) so a step across 0°/360° is a small step.
    use std::f64::consts::PI;
    let mut hs = Vec::with_capacity(idx.len());
    let mut acc = 0.0;
    let mut prev = h[idx[0]].to_radians();
    hs.push(prev);
    for i in &idx[1..] {
        let p = h[*i].to_radians();
        let dd = p - prev;
        let mut ddmod = (dd + PI).rem_euclid(2.0 * PI) - PI;
        if ddmod == -PI && dd > 0.0 {
            ddmod = PI;
        }
        if dd.abs() >= PI {
            acc += ddmod - dd;
        }
        hs.push(p + acc);
        prev = p;
    }
    let hs: Vec<f64> = hs.iter().map(|v| v.to_degrees()).collect();
    let travel: f64 = hs.windows(2).map(|w| (w[1] - w[0]).abs()).sum();
    let (mut count, mut direction, mut ext) = (0u32, 0i32, hs[0]);
    for x in &hs[1..] {
        let x = *x;
        if direction >= 0 {
            if x > ext {
                ext = x;
                if direction == 0 && x - hs[0] > 1.0 {
                    direction = 1;
                }
            } else if ext - x >= prom {
                count += 1;
                direction = -1;
                ext = x;
            }
        }
        if direction < 0 {
            if x < ext {
                ext = x;
            } else if x - ext >= prom {
                count += 1;
                direction = 1;
                ext = x;
            }
        }
    }
    (count, travel / 360.0)
}

#[cfg(test)]
#[path = "../tests/unit/tags/tests.rs"]
mod tests;
