use super::*;
use crate::{preset_by_id, PRESETS};

/// ⭐⭐**The measured tags of every built-in palette, as the prototype computed them.** The
/// thresholds were set on the prototype (`tags_proto.py`), looking at the palettes; this table is
/// its output, so a port that measures differently, or a threshold edited here alone, shows as the
/// palette whose tags moved. A deliberate retune changes the prototype and this table together.
const REFERENCE: &[(&str, &[&str])] = &[
    ("ember", &["reds", "oranges", "yellows", "browns", "warm", "calm", "smooth"]),
    ("ice", &["blues", "cool", "calm", "smooth", "single-hue"]),
    ("nebula", &["purples", "pinks", "lively", "vivid", "smooth"]),
    ("grayscale", &["neutrals", "calm", "smooth", "single-hue"]),
    ("four-seasons", &["oranges", "warm", "lively", "smooth"]),
    ("royalty", &["reds", "blues", "purples", "lively", "dark", "vivid", "smooth"]),
    ("rave", &["greens", "blues", "pinks", "frantic", "vivid", "smooth"]),
    ("emerald-isle", &["greens", "blues", "cool", "lively", "smooth", "single-hue"]),
    ("seventies", &["oranges", "yellows", "browns", "warm", "lively", "smooth"]),
    ("eighties", &["blues", "purples", "lively", "vivid", "smooth", "rainbow"]),
    ("mondrian", &["reds", "blues", "neutrals", "frantic", "banded"]),
    ("dragon-scales", &["greens", "browns", "lively", "dark", "smooth"]),
    ("machine-age", &["yellows", "neutrals", "calm", "smooth", "single-hue"]),
    ("bronze-age", &["oranges", "greens", "browns", "warm", "calm", "smooth"]),
    ("unity", &["reds", "greens", "blues", "lively", "vivid", "smooth", "rainbow"]),
    ("spice", &["oranges", "yellows", "browns", "warm", "calm", "smooth"]),
    ("veggies", &["reds", "greens", "warm", "lively", "smooth"]),
    ("mosaic", &["oranges", "yellows", "blues", "frantic", "banded"]),
    ("coral-reef", &["oranges", "blues", "purples", "lively", "smooth", "rainbow"]),
    ("aurora", &["greens", "blues", "purples", "cool", "lively", "dark", "smooth"]),
    ("desert", &["yellows", "blues", "lively", "smooth", "rainbow"]),
    ("candy", &["reds", "greens", "blues", "lively", "pastel", "smooth", "rainbow"]),
    ("blueprint", &["blues", "cool", "lively", "dark", "smooth", "single-hue"]),
    ("sepia", &["oranges", "yellows", "browns", "warm", "calm", "smooth", "single-hue"]),
    ("thermal", &["reds", "yellows", "purples", "warm", "calm", "vivid", "smooth"]),
    ("stained-glass", &["reds", "oranges", "greens", "blues", "purples", "frantic", "vivid", "banded"]),
    ("noir", &["neutrals", "calm", "dark", "smooth", "single-hue"]),
    ("cherry-blossom", &["reds", "greens", "pinks", "warm", "calm", "smooth"]),
    ("black-opal", &["greens", "blues", "purples", "cool", "frantic", "dark", "vivid", "smooth"]),
    ("lavender-fields", &["yellows", "purples", "cool", "lively", "smooth"]),
    ("lunar-new-year", &["reds", "yellows", "warm", "lively", "vivid", "smooth"]),
    ("valentines-day", &["reds", "pinks", "warm", "calm", "vivid", "smooth", "single-hue"]),
    ("mardi-gras", &["yellows", "greens", "purples", "cool", "lively", "dark", "smooth"]),
    ("holi", &["reds", "yellows", "greens", "lively", "light", "vivid", "smooth", "rainbow"]),
    ("nowruz", &["greens", "purples", "cool", "lively", "smooth"]),
    ("eid-al-fitr", &["yellows", "blues", "lively", "smooth"]),
    ("easter", &["yellows", "greens", "blues", "purples", "pinks", "lively", "pastel", "banded"]),
    ("vesak", &["oranges", "purples", "pinks", "warm", "lively", "smooth"]),
    ("midsummer", &["greens", "blues", "cool", "lively", "pastel", "smooth"]),
    ("matariki", &["blues", "cool", "lively", "dark", "smooth"]),
    ("mid-autumn", &["yellows", "blues", "warm", "lively", "smooth"]),
    ("meskel", &["oranges", "yellows", "greens", "browns", "warm", "lively", "smooth"]),
    ("halloween", &["oranges", "greens", "purples", "lively", "dark", "smooth"]),
    ("day-of-the-dead", &["oranges", "blues", "purples", "lively", "smooth"]),
    ("diwali", &["yellows", "blues", "purples", "lively", "smooth", "rainbow"]),
    ("thanksgiving", &["reds", "oranges", "yellows", "warm", "lively", "smooth"]),
    ("hanukkah", &["yellows", "blues", "cool", "lively", "smooth"]),
    ("christmas", &["reds", "greens", "lively", "smooth"]),
    ("kwanzaa", &["reds", "greens", "calm", "dark", "smooth"]),
    ("new-years-eve", &["purples", "frantic", "dark", "smooth"]),
    ("amethyst", &["purples", "cool", "calm", "smooth", "single-hue"]),
    ("malachite", &["greens", "cool", "frantic", "smooth", "single-hue"]),
    ("lapis-lazuli", &["blues", "cool", "lively", "dark", "smooth", "single-hue"]),
    ("turquoise", &["blues", "cool", "lively", "pastel", "smooth", "single-hue"]),
    ("obsidian", &["neutrals", "lively", "dark", "smooth", "single-hue"]),
    ("rose-quartz", &["pinks", "warm", "calm", "pastel", "smooth", "single-hue"]),
    ("tigers-eye", &["oranges", "yellows", "browns", "warm", "lively", "dark", "smooth", "single-hue"]),
    ("azurite", &["greens", "blues", "cool", "lively", "dark", "vivid", "smooth"]),
    ("rhodochrosite", &["reds", "pinks", "warm", "lively", "smooth", "single-hue"]),
    ("labradorite", &["blues", "neutrals", "lively", "dark", "smooth"]),
    ("moonstone", &["blues", "neutrals", "cool", "calm", "pastel", "smooth", "single-hue"]),
    ("jade", &["greens", "cool", "calm", "smooth", "single-hue"]),
    ("marble", &["neutrals", "lively", "pastel", "smooth", "single-hue"]),
    ("granite", &["neutrals", "frantic", "smooth", "single-hue"]),
    ("agate", &["oranges", "browns", "warm", "frantic", "smooth", "single-hue"]),
    ("fluorite", &["greens", "blues", "purples", "cool", "lively", "smooth"]),
    ("bismuth", &["oranges", "yellows", "blues", "lively", "smooth", "rainbow"]),
    ("sulfur", &["yellows", "warm", "calm", "smooth", "single-hue"]),
    ("cinnabar", &["reds", "neutrals", "warm", "lively", "vivid", "smooth", "single-hue"]),
    ("pyrite", &["yellows", "neutrals", "warm", "lively", "smooth", "single-hue"]),
    ("sunflower", &["yellows", "greens", "browns", "warm", "calm", "smooth"]),
    ("lotus", &["blues", "pinks", "lively", "smooth"]),
    ("moss", &["greens", "cool", "calm", "dark", "smooth", "single-hue"]),
    ("bamboo", &["greens", "cool", "frantic", "smooth", "single-hue"]),
    ("maple", &["reds", "oranges", "browns", "warm", "lively", "vivid", "smooth", "single-hue"]),
    ("ginkgo", &["yellows", "warm", "calm", "light", "smooth", "single-hue"]),
    ("wisteria", &["purples", "cool", "lively", "smooth"]),
    ("poppy", &["reds", "browns", "warm", "lively", "vivid", "smooth"]),
    ("iris", &["purples", "cool", "lively", "dark", "vivid", "smooth"]),
    ("orchid", &["pinks", "warm", "calm", "vivid", "smooth", "single-hue"]),
    ("cactus-flower", &["greens", "lively", "smooth"]),
    ("succulent", &["blues", "cool", "lively", "smooth"]),
    ("tulip-fields", &["reds", "oranges", "yellows", "greens", "purples", "pinks", "warm", "frantic", "vivid", "banded"]),
    ("redwood", &["oranges", "greens", "browns", "warm", "lively", "dark", "smooth"]),
    ("olive-grove", &["yellows", "greens", "lively", "smooth", "single-hue"]),
    ("fly-agaric", &["reds", "browns", "warm", "frantic", "vivid", "banded", "single-hue"]),
    ("chanterelle", &["oranges", "yellows", "greens", "warm", "lively", "smooth"]),
    ("foxfire", &["greens", "neutrals", "cool", "calm", "dark", "smooth", "single-hue"]),
    ("turkey-tail", &["oranges", "browns", "warm", "frantic", "smooth", "single-hue"]),
    ("lichen", &["yellows", "greens", "neutrals", "lively", "smooth", "single-hue"]),
    ("peacock", &["greens", "blues", "cool", "lively", "dark", "smooth"]),
    ("flamingo", &["reds", "pinks", "warm", "calm", "smooth", "single-hue"]),
    ("tiger", &["oranges", "warm", "lively", "banded", "single-hue"]),
    ("zebra", &["neutrals", "frantic", "banded", "single-hue"]),
    ("poison-dart-frog", &["blues", "cool", "lively", "vivid", "banded", "single-hue"]),
    ("monarch", &["oranges", "neutrals", "warm", "frantic", "dark", "banded", "single-hue"]),
    ("blue-morpho", &["blues", "browns", "cool", "lively", "dark", "vivid", "smooth"]),
    ("koi", &["reds", "blues", "neutrals", "warm", "lively", "vivid", "banded"]),
    ("mandarin-duck", &["oranges", "warm", "frantic", "smooth"]),
    ("scarlet-macaw", &["reds", "blues", "lively", "vivid", "smooth"]),
    ("toucan", &["neutrals", "warm", "lively", "dark", "smooth"]),
    ("chameleon", &["greens", "blues", "cool", "lively", "vivid", "smooth"]),
    ("jellyfish", &["blues", "purples", "pinks", "cool", "lively", "dark", "smooth"]),
    ("hummingbird", &["reds", "greens", "frantic", "vivid", "smooth"]),
    ("snow-leopard", &["neutrals", "lively", "pastel", "banded", "single-hue"]),
    ("clownfish", &["oranges", "neutrals", "warm", "frantic", "light", "vivid", "banded", "single-hue"]),
    ("fireflies", &["blues", "neutrals", "cool", "lively", "dark", "smooth"]),
    ("kingfisher", &["oranges", "blues", "lively", "smooth"]),
    ("honeybee", &["yellows", "neutrals", "warm", "lively", "banded", "single-hue"]),
    ("scarab", &["greens", "cool", "lively", "dark", "smooth", "rainbow"]),
    ("coffee", &["oranges", "browns", "warm", "calm", "dark", "smooth", "single-hue"]),
    ("chocolate", &["browns", "warm", "calm", "dark", "banded", "single-hue"]),
    ("citrus", &["oranges", "yellows", "greens", "warm", "lively", "light", "smooth"]),
    ("watermelon", &["reds", "greens", "warm", "lively", "vivid", "smooth"]),
    ("matcha", &["greens", "cool", "calm", "smooth", "single-hue"]),
    ("sushi", &["greens", "neutrals", "warm", "lively", "pastel", "banded"]),
    ("berries", &["reds", "purples", "pinks", "lively", "dark", "vivid", "smooth"]),
    ("red-wine", &["reds", "warm", "calm", "dark", "smooth", "single-hue"]),
    ("honey", &["oranges", "yellows", "browns", "warm", "calm", "smooth", "single-hue"]),
    ("peach", &["reds", "oranges", "yellows", "warm", "lively", "light", "smooth", "single-hue"]),
    ("avocado", &["yellows", "greens", "lively", "smooth", "single-hue"]),
    ("pomegranate", &["reds", "warm", "lively", "vivid", "smooth", "single-hue"]),
    ("neapolitan", &["pinks", "browns", "warm", "calm", "pastel", "banded", "single-hue"]),
    ("pistachio", &["yellows", "greens", "lively", "pastel", "smooth"]),
    ("lemon-meringue", &["yellows", "warm", "lively", "pastel", "smooth", "single-hue"]),
    ("dragon-fruit", &["reds", "neutrals", "warm", "frantic", "light", "vivid", "banded"]),
    ("kiwi", &["greens", "cool", "lively", "vivid", "smooth", "single-hue"]),
    ("fig", &["reds", "purples", "pinks", "warm", "lively", "dark", "smooth"]),
    ("blood-orange", &["reds", "oranges", "browns", "warm", "lively", "vivid", "smooth", "single-hue"]),
    ("bubble-tea", &["oranges", "purples", "browns", "warm", "lively", "smooth"]),
    ("botticelli", &["oranges", "greens", "blues", "lively", "smooth"]),
    ("rembrandt", &["browns", "warm", "lively", "dark", "smooth", "single-hue"]),
    ("vermeer", &["yellows", "blues", "neutrals", "lively", "dark", "smooth"]),
    ("hokusai", &["yellows", "blues", "cool", "lively", "dark", "smooth"]),
    ("hiroshige", &["blues", "cool", "lively", "dark", "smooth"]),
    ("wang-ximeng", &["yellows", "greens", "blues", "lively", "smooth"]),
    ("behzad", &["yellows", "frantic", "smooth"]),
    ("turner", &["oranges", "yellows", "warm", "lively", "pastel", "smooth", "single-hue"]),
    ("monet", &["blues", "purples", "cool", "lively", "smooth"]),
    ("van-gogh", &["blues", "cool", "lively", "dark", "smooth"]),
    ("seurat", &["oranges", "yellows", "greens", "frantic", "smooth"]),
    ("gauguin", &["reds", "yellows", "greens", "lively", "smooth"]),
    ("rousseau", &["greens", "cool", "lively", "dark", "smooth"]),
    ("klimt", &["yellows", "browns", "warm", "frantic", "smooth"]),
    ("munch", &["oranges", "blues", "warm", "lively", "smooth"]),
    ("mucha", &["yellows", "greens", "warm", "lively", "pastel", "smooth"]),
    ("kandinsky", &["yellows", "blues", "frantic", "smooth"]),
    ("hilma-af-klint", &["oranges", "yellows", "purples", "pinks", "warm", "lively", "pastel", "smooth"]),
    ("amrita-sher-gil", &["oranges", "yellows", "browns", "warm", "frantic", "smooth", "single-hue"]),
    ("torres-garcia", &["yellows", "browns", "neutrals", "frantic", "banded"]),
    ("rainforest", &["greens", "cool", "calm", "dark", "smooth", "single-hue"]),
    ("cloud-forest", &["greens", "cool", "lively", "smooth"]),
    ("mangrove", &["yellows", "greens", "frantic", "smooth", "single-hue"]),
    ("savanna", &["oranges", "yellows", "warm", "lively", "smooth", "single-hue"]),
    ("prairie", &["yellows", "greens", "lively", "smooth"]),
    ("taiga", &["greens", "neutrals", "cool", "calm", "dark", "smooth", "single-hue"]),
    ("tundra", &["oranges", "yellows", "browns", "warm", "lively", "smooth"]),
    ("glacier", &["blues", "cool", "lively", "smooth", "single-hue"]),
    ("alpine", &["greens", "blues", "cool", "lively", "smooth"]),
    ("heath", &["purples", "pinks", "lively", "smooth"]),
    ("chaparral", &["yellows", "greens", "lively", "smooth"]),
    ("wetland", &["yellows", "greens", "blues", "lively", "smooth"]),
    ("kelp-forest", &["blues", "lively", "dark", "smooth"]),
    ("lagoon", &["blues", "cool", "lively", "smooth"]),
    ("deep-sea", &["blues", "neutrals", "cool", "lively", "dark", "smooth"]),
    ("hydrothermal-vent", &["reds", "neutrals", "warm", "lively", "dark", "smooth"]),
    ("volcano", &["browns", "neutrals", "warm", "lively", "dark", "smooth", "single-hue"]),
    ("canyon", &["reds", "oranges", "browns", "warm", "frantic", "smooth", "single-hue"]),
    ("salt-flat", &["blues", "pinks", "neutrals", "lively", "pastel", "smooth"]),
    ("cave", &["blues", "neutrals", "lively", "dark", "smooth"]),
    ("paris", &["oranges", "neutrals", "calm", "smooth", "single-hue"]),
    ("venice", &["oranges", "blues", "lively", "smooth"]),
    ("santorini", &["blues", "lively", "smooth"]),
    ("chefchaouen", &["blues", "cool", "lively", "smooth"]),
    ("marrakech", &["reds", "oranges", "frantic", "smooth"]),
    ("jaipur", &["reds", "warm", "lively", "pastel", "smooth"]),
    ("istanbul", &["blues", "cool", "frantic", "smooth"]),
    ("taj-mahal", &["neutrals", "warm", "calm", "pastel", "smooth"]),
    ("petra", &["reds", "browns", "warm", "frantic", "smooth", "single-hue"]),
    ("giza", &["yellows", "blues", "lively", "smooth"]),
    ("kyoto", &["reds", "greens", "warm", "frantic", "vivid", "smooth"]),
    ("tokyo", &["blues", "purples", "frantic", "smooth"]),
    ("forbidden-city", &["reds", "warm", "lively", "vivid", "smooth"]),
    ("angkor-wat", &["oranges", "pinks", "warm", "lively", "smooth"]),
    ("machu-picchu", &["greens", "neutrals", "cool", "lively", "smooth"]),
    ("havana", &["blues", "lively", "pastel", "smooth"]),
    ("burano", &["blues", "lively", "vivid", "banded"]),
    ("new-york", &["yellows", "blues", "lively", "smooth"]),
    ("golden-gate", &["reds", "blues", "warm", "lively", "smooth"]),
    ("st-basils", &["reds", "yellows", "greens", "blues", "frantic", "vivid", "banded"]),
];

#[test]
fn computed_tags_match_the_reference() {
    assert_eq!(REFERENCE.len(), PRESETS.len(), "every preset is in the reference");
    let mut wrong = Vec::new();
    for (id, want) in REFERENCE {
        let i = preset_by_id(id).unwrap_or_else(|| panic!("{id}: no such preset"));
        let p = &PRESETS[i];
        let got = computed_tags(&Gradient::from_stops(p.name, p.stops));
        if got != *want {
            wrong.push(format!("{id}: got {got:?}, want {want:?}"));
        }
    }
    assert!(wrong.is_empty(), "measured tags moved:
{}", wrong.join("
"));
}

/// Tag ids are unique and plain (a file stores them), and every hand table entry names a real
/// preset with real tags of the two hand groups.
#[test]
fn the_vocabulary_and_the_hand_tags_are_well_formed() {
    let mut seen = std::collections::HashSet::new();
    for d in TAGS {
        assert!(seen.insert(d.id), "{}: twice", d.id);
        assert!(d.id.bytes().all(|b| b.is_ascii_lowercase() || b == b'-'), "{:?}", d.id);
    }
    for g in TagGroup::ALL {
        assert!(TAGS.iter().any(|d| d.group == g), "{g:?} has no tags");
    }
    let mut covered = std::collections::HashSet::new();
    for (id, hand) in PRESET_TAGS {
        assert!(preset_by_id(id).is_some(), "{id}: no such preset");
        assert!(covered.insert(*id), "{id}: twice in PRESET_TAGS");
        for t in *hand {
            let d = tag_def(t).unwrap_or_else(|| panic!("{id}: {t} is not a tag"));
            assert!(!d.group.is_measured(), "{id}: {t} is measured, not given by hand");
        }
    }
}

/// Every built-in palette has hand tags and a purpose (analytical, aesthetic or both), exactly one
/// energy and one of smooth/banded, and never two temperatures or two brightnesses.
#[test]
fn every_preset_is_tagged_sensibly() {
    let count = |tags: &[&str], ids: &[&str]| tags.iter().filter(|t| ids.contains(t)).count();
    for (i, p) in PRESETS.iter().enumerate() {
        assert!(PRESET_TAGS.iter().any(|(id, _)| *id == p.id), "{}: no hand tags", p.id);
        let tags = preset_tags(i);
        assert!(count(tags, &["analytical", "aesthetic"]) >= 1, "{}: no purpose", p.id);
        assert_eq!(count(tags, &["calm", "lively", "frantic"]), 1, "{}: energy {tags:?}", p.id);
        assert_eq!(count(tags, &["smooth", "banded"]), 1, "{}: shape {tags:?}", p.id);
        assert!(count(tags, &["warm", "cool"]) <= 1, "{}: {tags:?}", p.id);
        assert!(count(tags, &["dark", "light", "pastel"]) <= 1, "{}: {tags:?}", p.id);
        let rank: Vec<usize> = tags.iter().map(|t| TAGS.iter().position(|d| d.id == *t).unwrap()).collect();
        assert!(rank.windows(2).all(|w| w[0] < w[1]), "{}: not in vocabulary order {tags:?}", p.id);
    }
    assert!(preset_tags(PRESETS.len()).is_empty(), "past the presets is no tags");
}

/// A gradient nobody tagged gets measured like a preset: what a user's saved or imported gradient
/// relies on.
#[test]
fn an_untagged_gradient_is_measured() {
    let grey = Gradient::from_stops("grey", &[(0.0, [0.0; 3]), (0.5, [1.0; 3]), (1.0, [0.0; 3])]);
    let tags = computed_tags(&grey);
    for want in ["neutrals", "calm", "smooth", "single-hue"] {
        assert!(tags.contains(&want), "black-white-black: {want} missing from {tags:?}");
    }
    // Hard bands of saturated color, each meeting the next at an edge.
    let bands: Vec<(f32, [f32; 3])> = [[1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 1.0, 0.0], [0.0, 0.6, 0.0]]
        .iter()
        .enumerate()
        .flat_map(|(k, c)| [(k as f32 / 4.0, *c), ((k + 1) as f32 / 4.0, *c)])
        .collect();
    let tags = computed_tags(&Gradient::from_stops("bands", &bands));
    assert!(tags.contains(&"banded") && !tags.contains(&"smooth"), "four hard bands: {tags:?}");
}
