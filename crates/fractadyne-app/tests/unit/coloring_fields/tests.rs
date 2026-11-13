use super::*;

/// The spec's fields as a JSON object: what a field is called and holds, for comparing two specs
/// field by field without listing them.
fn fields_of(s: &ColoringSpec) -> serde_json::Map<String, serde_json::Value> {
    match serde_json::to_value(s).expect("a spec serializes") {
        serde_json::Value::Object(m) => m,
        v => panic!("a spec is not an object: {v}"),
    }
}

/// The spec fields that differ between `a` and `b`.
fn changed(a: &ColoringSpec, b: &ColoringSpec) -> Vec<String> {
    let (a, b) = (fields_of(a), fields_of(b));
    a.keys().filter(|k| a.get(*k) != b.get(*k)).cloned().collect()
}

/// A value for `f` other than the default's, as a view would write it.
fn other_value(f: &Field) -> &'static str {
    match f.access {
        Access::Flag(get, _) => {
            if get(&ColoringSpec::default()) {
                "0"
            } else {
                "1"
            }
        }
        Access::Real(..) => "2.5",
        Access::Count(..) => "77",
        Access::Choice(..) => match f.key {
            "method" => "stripe",
            "trap" => "circle",
            "palette_mode" => "binary",
            "interior" => "trap",
            "interior_palette" => "ember",
            k => panic!("no other value for the choice {k}"),
        },
        Access::Rgb(..) => "0.25,0.5,0.75",
    }
}

/// The spec fields a view does not carry through the table: `export.rs` writes and reads them under
/// its older keys (`palette`/`palette_id`, `cycle`, `iter_div`, `offset`, `palette_custom`).
const CARRIED_ELSEWHERE: &[&str] = &[
    "palette_idx",
    "palette_id",
    "cycle",
    "iter_div",
    "offset",
    "custom_palette",
    "custom_palette_flat",
    "custom_segments",
    "use_custom_palette",
];

/// ⭐**Every coloring setting the session saves is accounted for in a view** (coloring plan B4): it
/// has a row in the table, or it is one of the older keys `export.rs` carries. A field added to
/// [`ColoringSpec`] without either fails here, so a setting cannot be saved in the session and sent
/// to a farm yet silently left out of a `.fdn`, a bookmark or a Share text.
#[test]
fn every_spec_field_has_a_row_or_is_carried_elsewhere() {
    let d = ColoringSpec::default();
    let mut covered: Vec<String> = CARRIED_ELSEWHERE.iter().map(|s| s.to_string()).collect();
    for f in &FIELDS {
        // A choice tries each of its values (Duotone and Binary set different switches).
        let values: Vec<&str> = match f.access {
            Access::Choice(_, _, choices) => choices(),
            _ => vec![other_value(f)],
        };
        let mut touched: Vec<String> = Vec::new();
        for v in values {
            let mut s = d.clone();
            assert!(f.access.read(v, f.name, &mut s, &mut Vec::new()), "{} did not read {v:?}", f.key);
            touched.extend(changed(&d, &s));
        }
        touched.sort();
        touched.dedup();
        assert!(!touched.is_empty(), "{} changed no spec field", f.key);
        for t in touched {
            assert!(!covered.contains(&t), "{t} is set by two rows ({})", f.key);
            covered.push(t);
        }
    }
    covered.sort();
    covered.dedup();
    let mut all: Vec<String> = fields_of(&d).keys().cloned().collect();
    all.sort();
    assert_eq!(covered, all, "a coloring field has no view row and is not carried elsewhere");
}

/// A row's parents exist and come before it, which the reader relies on (it decides a row's reset
/// from the rows it has already read); the keys are unique.
#[test]
fn parents_come_first_and_keys_are_unique() {
    for (i, f) in FIELDS.iter().enumerate() {
        assert!(!FIELDS[..i].iter().any(|g| g.key == f.key), "{} twice", f.key);
        for p in f.parents {
            assert!(FIELDS[..i].iter().any(|g| g.key == *p), "{}'s parent {p} is not before it", f.key);
        }
    }
    assert_eq!(KEYS.len(), FIELDS.len());
    assert!(KEYS.iter().zip(&FIELDS).all(|(k, f)| *k == f.key));
}

/// The default coloring writes nothing (an ordinary view's metadata is what it was before these
/// keys existed), and each row written reads back to the same value onto a different coloring.
#[test]
fn the_default_writes_nothing_and_each_row_reads_back() {
    assert_eq!(view_lines(&ColoringSpec::default()), "");
    // Every row at once, with what each hangs off switched on.
    let mut s = ColoringSpec::default();
    for k in ["method=stripe", "stripe_tail=1", "interior=trap", "light=1", "glow=1", "palette_mode=duotone"] {
        let (key, v) = k.split_once('=').unwrap();
        let f = FIELDS.iter().find(|f| f.key == key).unwrap();
        assert!(f.access.read(v, f.name, &mut s, &mut Vec::new()));
    }
    for f in &FIELDS {
        if f.parents.is_empty() {
            continue;
        }
        assert!(f.access.read(other_value(f), f.name, &mut s, &mut Vec::new()), "{}", f.key);
    }
    let text = view_lines(&s);
    let lines: std::collections::HashMap<String, String> =
        text.lines().filter_map(|l| l.split_once('=')).map(|(k, v)| (k.to_string(), v.to_string())).collect();
    let mut back = ColoringSpec { light: false, de: false, use_duotone: false, ..ColoringSpec::default() };
    let mut clamped = Vec::new();
    apply_view(&mut back, &|k| lines.get(k).cloned(), true, &mut clamped);
    assert!(clamped.is_empty(), "{clamped:?}");
    for f in &FIELDS {
        if (f.shows)(&s) {
            assert!(!f.access.differs(&s, &back), "{} did not read back ({text})", f.key);
        }
    }
}
