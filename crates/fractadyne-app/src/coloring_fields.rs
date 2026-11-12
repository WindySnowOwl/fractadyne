//! The coloring field table (coloring plan B4): every coloring setting a view carries, one row each,
//! with what the `.fdn` writer and reader need — its key, its kind and range, the settings it hangs
//! off and when the picture depends on it. The writer ([`view_lines`]) and the reader
//! ([`apply_view`]) walk the table on a [`ColoringSpec`], the struct the session saves and a farm
//! job carries, so a setting gets its view key by gaining a row here.
//!
//! The rules are A1's (`view_coloring`, which is now a typed front on this table):
//! - ⭐**A view carries what its picture depends on, and nothing else.** A key is written only when
//!   its setting changes the picture ([`Field::shows`]) and differs from the default, so an ordinary
//!   Smooth view is written byte for byte as before these keys existed.
//! - ⭐**Reading follows `iter_div`.** A file that states its coloring (`cycle=`) and leaves a key
//!   out was written at that key's default, so the setting returns to it. A setting that hangs off
//!   others ([`Field::parents`]: the stripe density off the method, the light's angle off the light)
//!   returns to its default only when those were read from this file (or reset by it) and the
//!   picture depends on it; when one of them is unreadable (an unknown method), what hangs off it is
//!   left alone. A file that states no coloring leaves every setting it does not name alone.
//! - A value out of range is clamped and named; an unreadable one keeps the current setting.

use crate::{slider_range, ColorMethod, TrapType};
use fractadyne_state::ColoringSpec;

/// How a row reads and writes its setting on a [`ColoringSpec`], by kind.
#[derive(Clone, Copy)]
pub(crate) enum Access {
    /// `1` or `0`.
    Flag(fn(&ColoringSpec) -> bool, fn(&mut ColoringSpec, bool)),
    /// A number, clamped into `lo..=hi` on reading (written with `{}`, the shortest decimal that
    /// reads back to the same bits).
    Real(fn(&ColoringSpec) -> f32, fn(&mut ColoringSpec, f32), f32, f32),
    /// A whole number, clamped into `lo..=hi` on reading.
    Count(fn(&ColoringSpec) -> u32, fn(&mut ColoringSpec, u32), u32, u32),
    /// One of the keys `choices` lists.
    Choice(fn(&ColoringSpec) -> &str, fn(&mut ColoringSpec, &str), fn() -> Vec<&'static str>),
    /// Three numbers `r,g,b` in 0..=1 (display space), clamped on reading.
    Rgb(fn(&ColoringSpec) -> [f32; 3], fn(&mut ColoringSpec, [f32; 3])),
}

/// One coloring setting a view carries.
pub(crate) struct Field {
    /// Its key in a view (`.fdn`, a bookmark, a Share text, an image's embedded view).
    pub(crate) key: &'static str,
    /// What a message calls it when its value had to be clamped.
    pub(crate) name: &'static str,
    pub(crate) access: Access,
    /// The rows it hangs off (earlier in the table): it is reset to its default only when they
    /// were read from the same file.
    pub(crate) parents: &'static [&'static str],
    /// Whether the picture depends on it, given the rest of the coloring.
    pub(crate) shows: fn(&ColoringSpec) -> bool,
}

fn method(s: &ColoringSpec) -> ColorMethod {
    ColorMethod::from_key(&s.color_method)
}

fn always(_: &ColoringSpec) -> bool {
    true
}

/// "Normalize deep colors" decides only for Smooth (`live_norm_window`).
fn uses_normalize_live(s: &ColoringSpec) -> bool {
    method(s) == ColorMethod::Smooth
}

/// "Always fit range" and the log scale act on Smooth's escape range and on the orbit trap's values
/// (`live_norm_window`, `norm_map_for`).
fn uses_fit(s: &ColoringSpec) -> bool {
    matches!(method(s), ColorMethod::Smooth | ColorMethod::OrbitTrap)
}

fn two_color_on(s: &ColoringSpec) -> bool {
    s.use_binary || s.use_duotone
}

/// The table, in the order a view writes its keys. A row's parents come before it.
pub(crate) const FIELDS: [Field; 17] = [
    Field {
        key: "method",
        name: "coloring method",
        access: Access::Choice(|s| &s.color_method, |s, v| s.color_method = v.to_string(), || {
            ColorMethod::ALL.iter().map(|m| m.key()).collect()
        }),
        parents: &[],
        shows: always,
    },
    Field {
        key: "stripe_freq",
        name: "stripe density",
        access: Access::Real(
            |s| s.stripe_freq,
            |s, v| s.stripe_freq = v,
            *slider_range::STRIPE_FREQ.start(),
            *slider_range::STRIPE_FREQ.end(),
        ),
        parents: &["method"],
        shows: |s| method(s) == ColorMethod::Stripe,
    },
    Field {
        key: "stripe_tail",
        name: "stripe tail",
        access: Access::Flag(|s| s.stripe_tail, |s, v| s.stripe_tail = v),
        parents: &["method"],
        shows: |s| method(s) == ColorMethod::Stripe,
    },
    Field {
        key: "stripe_tail_len",
        name: "tail length",
        access: Access::Count(
            |s| s.stripe_tail_len,
            |s, v| s.stripe_tail_len = v,
            *slider_range::STRIPE_TAIL_LEN.start(),
            *slider_range::STRIPE_TAIL_LEN.end(),
        ),
        parents: &["method", "stripe_tail"],
        shows: |s| method(s) == ColorMethod::Stripe && s.stripe_tail,
    },
    Field {
        key: "trap",
        name: "orbit trap",
        access: Access::Choice(|s| &s.trap_type, |s, v| s.trap_type = v.to_string(), || {
            TrapType::ALL.iter().map(|t| t.key()).collect()
        }),
        parents: &["method"],
        shows: |s| method(s) == ColorMethod::OrbitTrap,
    },
    Field {
        key: "light",
        name: "relief light",
        access: Access::Flag(|s| s.light, |s, v| s.light = v),
        parents: &[],
        shows: always,
    },
    Field {
        key: "light_angle",
        name: "light angle",
        access: Access::Real(|s| s.light_angle, |s, v| s.light_angle = v, f32::MIN, f32::MAX),
        parents: &["light"],
        shows: |s| s.light,
    },
    Field {
        key: "light_height",
        name: "relief",
        access: Access::Real(
            |s| s.light_height,
            |s, v| s.light_height = v,
            *slider_range::LIGHT_HEIGHT.start(),
            *slider_range::LIGHT_HEIGHT.end(),
        ),
        parents: &["light"],
        shows: |s| s.light,
    },
    Field {
        key: "glow",
        name: "distance glow",
        access: Access::Flag(|s| s.de, |s, v| s.de = v),
        parents: &[],
        shows: always,
    },
    Field {
        key: "glow_strength",
        name: "glow",
        access: Access::Real(
            |s| s.de_strength,
            |s, v| s.de_strength = v,
            *slider_range::GLOW_STRENGTH.start(),
            *slider_range::GLOW_STRENGTH.end(),
        ),
        parents: &["glow"],
        shows: |s| s.de,
    },
    Field {
        key: "glow_width",
        name: "band width",
        access: Access::Real(
            |s| s.de_width,
            |s, v| s.de_width = v,
            *slider_range::GLOW_WIDTH.start(),
            *slider_range::GLOW_WIDTH.end(),
        ),
        parents: &["glow"],
        shows: |s| s.de,
    },
    // Duotone or Binary: one choice (`palette_mode`), held in the session as two switches. Off is
    // the absence of the key, and never written; Binary wins when both are set, as
    // `chosen_gradient` reads them.
    Field {
        key: "palette_mode",
        name: "palette mode",
        access: Access::Choice(
            |s| {
                if s.use_binary {
                    "binary"
                } else if s.use_duotone {
                    "duotone"
                } else {
                    "off"
                }
            },
            |s, v| {
                s.use_binary = v == "binary";
                s.use_duotone = v == "duotone";
            },
            || vec!["duotone", "binary"],
        ),
        parents: &[],
        shows: always,
    },
    Field {
        key: "palette_lo",
        name: "palette colors",
        access: Access::Rgb(|s| s.duotone_lo, |s, v| s.duotone_lo = v),
        parents: &["palette_mode"],
        shows: two_color_on,
    },
    Field {
        key: "palette_hi",
        name: "palette colors",
        access: Access::Rgb(|s| s.duotone_hi, |s, v| s.duotone_hi = v),
        parents: &["palette_mode"],
        shows: two_color_on,
    },
    Field {
        key: "normalize_live",
        name: "normalize deep colors",
        access: Access::Flag(|s| s.normalize_live, |s, v| s.normalize_live = v),
        parents: &["method"],
        shows: uses_normalize_live,
    },
    Field {
        key: "normalize_fit",
        name: "always fit range",
        access: Access::Flag(|s| s.normalize_fit, |s, v| s.normalize_fit = v),
        parents: &["method"],
        shows: uses_fit,
    },
    Field {
        key: "log_scale",
        name: "log color scale",
        access: Access::Flag(|s| s.log_palette, |s, v| s.log_palette = v),
        parents: &["method"],
        shows: uses_fit,
    },
];

/// Every key the table writes and reads, in its order. `export.rs`'s reader lists them too
/// (`KNOWN_VIEW_KEYS`, `TYPED_VIEW_KEYS`), and a unit test holds the lists together.
pub(crate) const KEYS: [&str; FIELDS.len()] = {
    let mut k = [""; FIELDS.len()];
    let mut i = 0;
    while i < FIELDS.len() {
        k[i] = FIELDS[i].key;
        i += 1;
    }
    k
};

impl Access {
    /// The setting as a view writes it.
    fn text(&self, s: &ColoringSpec) -> String {
        match *self {
            Access::Flag(get, _) => if get(s) { "1" } else { "0" }.to_string(),
            Access::Real(get, ..) => get(s).to_string(),
            Access::Count(get, ..) => get(s).to_string(),
            Access::Choice(get, ..) => get(s).to_string(),
            Access::Rgb(get, _) => {
                let c = get(s);
                format!("{},{},{}", c[0], c[1], c[2])
            }
        }
    }

    /// Whether `a` and `b` hold different values of this setting.
    fn differs(&self, a: &ColoringSpec, b: &ColoringSpec) -> bool {
        match *self {
            Access::Flag(get, _) => get(a) != get(b),
            Access::Real(get, ..) => get(a) != get(b),
            Access::Count(get, ..) => get(a) != get(b),
            Access::Choice(get, ..) => get(a) != get(b),
            Access::Rgb(get, _) => get(a) != get(b),
        }
    }

    /// Copy this setting from `from` into `to`.
    fn copy(&self, from: &ColoringSpec, to: &mut ColoringSpec) {
        match *self {
            Access::Flag(get, set) => set(to, get(from)),
            Access::Real(get, set, ..) => set(to, get(from)),
            Access::Count(get, set, ..) => set(to, get(from)),
            Access::Choice(get, set, _) => {
                let v = get(from).to_string();
                set(to, &v)
            }
            Access::Rgb(get, set) => set(to, get(from)),
        }
    }

    /// Read `text` into `to`: `Some(true)` when it was usable (clamped into range if it had to
    /// be, and named in `clamped` then), `Some(false)` when it was not (a number that is not
    /// finite is named too, as the cycle's reader does), and the setting is left alone.
    fn read(&self, text: &str, name: &'static str, to: &mut ColoringSpec, clamped: &mut Vec<&'static str>) -> bool {
        match *self {
            Access::Flag(_, set) => match text {
                "1" => {
                    set(to, true);
                    true
                }
                "0" => {
                    set(to, false);
                    true
                }
                _ => false,
            },
            Access::Real(_, set, lo, hi) => match text.parse::<f32>() {
                Ok(x) if x.is_finite() => {
                    let c = x.clamp(lo, hi);
                    if c != x {
                        clamped.push(name);
                    }
                    set(to, c);
                    true
                }
                Ok(_) => {
                    clamped.push(name);
                    false
                }
                Err(_) => false,
            },
            Access::Count(_, set, lo, hi) => match text.parse::<u64>() {
                Ok(n) => {
                    let c = n.clamp(u64::from(lo), u64::from(hi)) as u32;
                    if u64::from(c) != n {
                        clamped.push(name);
                    }
                    set(to, c);
                    true
                }
                Err(_) => false,
            },
            Access::Choice(_, set, choices) => {
                if choices().contains(&text) {
                    set(to, text);
                    true
                } else {
                    false
                }
            }
            Access::Rgb(_, set) => {
                let parts: Vec<Option<f32>> =
                    text.split(',').map(|t| t.trim().parse::<f32>().ok().filter(|x| x.is_finite())).collect();
                match parts.as_slice() {
                    [Some(r), Some(g), Some(b)] => {
                        let c = [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)];
                        if c != [*r, *g, *b] {
                            clamped.push(name);
                        }
                        set(to, c);
                        true
                    }
                    _ => false,
                }
            }
        }
    }
}

/// The view lines for this coloring: each setting the picture depends on that differs from the
/// default, one `key=value` per line, in the table's order. Empty for the default coloring.
pub(crate) fn view_lines(spec: &ColoringSpec) -> String {
    let d = ColoringSpec::default();
    let mut out = String::new();
    for f in &FIELDS {
        if (f.shows)(spec) && f.access.differs(spec, &d) {
            out.push_str(f.key);
            out.push('=');
            out.push_str(&f.access.text(spec));
            out.push('\n');
        }
    }
    out
}

/// Apply a view's coloring keys to `spec`, this session's coloring, by the rules at the top of this
/// module. `get` reads a key (`None` when the file does not have it); `states` says the file states
/// its coloring (it has `cycle=`). A value out of range is clamped and named in `clamped`; an
/// unreadable one keeps the current setting (the view reader's type table reports it with its line).
pub(crate) fn apply_view(spec: &mut ColoringSpec, get: &dyn Fn(&str) -> Option<String>, states: bool, clamped: &mut Vec<&'static str>) {
    let d = ColoringSpec::default();
    // The rows this file set (to its value, or back to the default).
    let mut settled: Vec<&'static str> = Vec::new();
    for f in &FIELDS {
        let read = match get(f.key) {
            Some(text) => f.access.read(&text, f.name, spec, clamped),
            None => {
                let reset = if f.parents.is_empty() {
                    states
                } else {
                    f.parents.iter().all(|p| settled.contains(p)) && (f.shows)(spec)
                };
                if reset {
                    f.access.copy(&d, spec);
                }
                reset
            }
        };
        if read {
            settled.push(f.key);
        }
    }
}

#[cfg(test)]
#[path = "../tests/unit/coloring_fields/tests.rs"]
mod tests;
