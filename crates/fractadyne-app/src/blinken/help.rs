//! The panel's help cards (design/blinkenlights.md, "Where it sits and how it is called"): for each
//! instrument, lamp and readout, what it shows, what healthy looks like, and what signals trouble.
//! A card is built when it is shown, in the language in use; its key is an id, never translated.

/// One help card.
pub(crate) struct Card {
    /// Shown as it is: the message holds the capitals the panel's legends are engraved in.
    pub(crate) title: String,
    pub(crate) what: String,
    pub(crate) note: Option<String>,
    pub(crate) good: Vec<String>,
    pub(crate) bad: Vec<String>,
}

/// The card for a key: an instrument's name, a readout's, or `lamp:` and the lamp's.
pub(crate) fn card(key: &str) -> Option<Card> {
    CARDS.iter().find(|(k, _)| *k == key).map(|(_, build)| build())
}

fn c(title: String, what: String, note: Option<String>, good: Vec<String>, bad: Vec<String>) -> Card {
    Card { title, what, note, good, bad }
}

/// The cards by key, each built in the language in use when it is shown.
pub(crate) static CARDS: &[(&str, fn() -> Card)] = &[
    ("walk", || c(
        tr!(live_title_walk),
        tr!(live_card_walk_what),
        Some(tr!(live_card_walk_note)),
        vec![tr!(live_card_walk_good_sweep), tr!(live_card_walk_good_arcs), tr!(live_card_walk_good_still)],
        vec![tr!(live_card_walk_bad_red), tr!(live_card_walk_bad_restart), tr!(live_card_walk_bad_hollow)])),
    ("age", || c(
        tr!(live_title_age),
        tr!(live_card_age_what),
        None,
        vec![tr!(live_card_age_good_sawtooth), tr!(live_card_age_good_still)],
        vec![tr!(live_card_age_bad_red), tr!(live_card_age_bad_ramp)])),
    ("tiles", || c(
        tr!(live_title_tiles),
        tr!(live_card_tiles_what),
        None,
        vec![tr!(live_card_tiles_good_whole), tr!(live_card_tiles_good_grid)],
        vec![tr!(live_card_tiles_bad_small), tr!(live_card_tiles_bad_strips), tr!(live_card_tiles_bad_splits), tr!(live_card_tiles_bad_dark)])),
    ("scope", || c(
        tr!(live_title_scope),
        tr!(live_card_scope_what),
        None,
        vec![tr!(live_card_scope_good_under), tr!(live_card_scope_good_pin), tr!(live_card_scope_good_samples)],
        vec![tr!(live_card_scope_bad_over), tr!(live_card_scope_bad_seconds), tr!(live_card_scope_bad_hollow)])),
    ("timers", || c(
        tr!(live_title_timers),
        tr!(live_card_timers_what),
        None,
        vec![tr!(live_card_timers_good)],
        vec![tr!(live_card_timers_bad_stuck), tr!(live_card_timers_bad_dark)])),
    ("dial", || c(
        tr!(live_title_dial),
        tr!(live_card_dial_what),
        None,
        vec![tr!(live_card_dial_good_ahead), tr!(live_card_dial_good_crosses)],
        vec![tr!(live_card_dial_bad_passing), tr!(live_card_dial_bad_crosses), tr!(live_card_dial_bad_crossing)])),
    ("bank", || c(tr!(live_title_bank), tr!(live_card_bank_what), Some(tr!(live_card_bank_note)), vec![], vec![])),
    ("depth", || c(tr!(live_title_depth), tr!(live_card_depth_what), Some(tr!(live_card_depth_note)), vec![], vec![])),
    ("iter", || c(
        tr!(live_title_iterations),
        tr!(live_card_iter_what),
        None,
        vec![tr!(live_card_iter_good)],
        vec![tr!(live_card_iter_bad_low), tr!(live_card_iter_bad_high)])),
    ("ms", || c(
        tr!(live_title_frame_ms),
        tr!(live_card_ms_what),
        None,
        vec![tr!(live_card_ms_good)],
        vec![tr!(live_card_ms_bad_stutter), tr!(live_card_ms_bad_spikes)])),
    ("clock", || c(tr!(live_title_clock), tr!(live_card_clock_what), Some(tr!(live_card_clock_note)), vec![], vec![])),
    ("tach", || c(
        tr!(live_title_steps),
        tr!(live_card_tach_what),
        None,
        vec![tr!(live_card_tach_good)],
        vec![tr!(live_card_tach_bad_together), tr!(live_card_tach_bad_red)])),
    ("engines", || c(
        tr!(live_title_engines),
        tr!(live_card_engines_what),
        None,
        vec![tr!(live_card_engines_good)],
        vec![tr!(live_card_engines_bad_pinned), tr!(live_card_engines_bad_idle)])),
    ("memory", || c(
        tr!(live_title_memory),
        tr!(live_card_memory_what),
        None,
        vec![tr!(live_card_memory_good)],
        vec![tr!(live_card_memory_bad_leak), tr!(live_card_memory_bad_vram)])),
    ("cores", || c(
        tr!(live_title_cores),
        tr!(live_card_cores_what),
        None,
        vec![tr!(live_card_cores_good)],
        vec![tr!(live_card_cores_bad_every), tr!(live_card_cores_bad_own)])),
    ("pixels", || c(tr!(live_title_pixels), tr!(live_card_pixels_what), None, vec![tr!(live_card_pixels_good)], vec![tr!(live_card_pixels_bad)])),
    ("lamp:plan", || c(tr!(live_lamp_plan), tr!(live_card_lamp_plan_what), None, vec![tr!(live_card_lamp_plan_good)], vec![tr!(live_card_lamp_plan_bad)])),
    ("lamp:dispatch", || c(tr!(live_lamp_dispatch), tr!(live_card_lamp_dispatch_what), None, vec![tr!(live_card_lamp_dispatch_good)], vec![tr!(live_card_lamp_dispatch_bad)])),
    ("lamp:live", || c(tr!(live_lamp_live), tr!(live_card_lamp_live_what), Some(tr!(live_card_lamp_live_note)), vec![tr!(live_card_lamp_live_good)], vec![])),
    ("lamp:held", || c(
        tr!(live_lamp_held),
        tr!(live_card_lamp_held_what),
        None,
        vec![tr!(live_card_lamp_held_good_dive), tr!(live_card_lamp_held_good_still)],
        vec![tr!(live_card_lamp_held_bad)])),
    ("lamp:reproj", || c(tr!(live_lamp_reproj), tr!(live_card_lamp_reproj_what), None, vec![tr!(live_card_lamp_reproj_good)], vec![tr!(live_card_lamp_reproj_bad)])),
    ("lamp:reading", || c(tr!(live_lamp_reading), tr!(live_card_lamp_reading_what), None, vec![tr!(live_card_lamp_reading_good)], vec![tr!(live_card_lamp_reading_bad)])),
    ("lamp:pass", || c(tr!(live_lamp_pin_pass), tr!(live_card_lamp_pin_pass_what), None, vec![tr!(live_card_lamp_pin_pass_good)], vec![tr!(live_card_lamp_pin_pass_bad)])),
    ("lamp:split", || c(tr!(live_lamp_split), tr!(live_card_lamp_split_what), None, vec![tr!(live_card_lamp_split_good)], vec![tr!(live_card_lamp_split_bad)])),
    ("lamp:adopt", || c(tr!(live_lamp_adopt), tr!(live_card_lamp_adopt_what), None, vec![tr!(live_card_lamp_adopt_good)], vec![tr!(live_card_lamp_adopt_bad)])),
    ("lamp:abandon", || c(tr!(live_lamp_abandon), tr!(live_card_lamp_abandon_what), None, vec![tr!(live_card_lamp_abandon_good)], vec![tr!(live_card_lamp_abandon_bad)])),
    ("lamp:refLive", || c(
        tr!(live_lamp_ref_live),
        tr!(live_card_lamp_ref_live_what),
        None,
        vec![tr!(live_card_lamp_ref_live_good)],
        vec![tr!(live_card_lamp_ref_live_bad_long), tr!(live_card_lamp_ref_live_bad_lookahead)])),
    ("lamp:refAhead", || c(tr!(live_lamp_ref_ahead), tr!(live_card_lamp_ref_ahead_what), None, vec![tr!(live_card_lamp_ref_ahead_good)], vec![tr!(live_card_lamp_ref_ahead_bad)])),
    ("lamp:refHold", || c(
        tr!(live_lamp_ref_hold),
        tr!(live_card_lamp_ref_hold_what),
        Some(tr!(live_card_lamp_ref_hold_note)),
        vec![tr!(live_card_lamp_ref_hold_good)],
        vec![tr!(live_card_lamp_ref_hold_bad)])),
    ("lamp:series", || c(tr!(live_lamp_series), tr!(live_card_lamp_series_what), None, vec![tr!(live_card_lamp_series_good)], vec![tr!(live_card_lamp_series_bad)])),
    ("lamp:worker", || c(tr!(live_lamp_worker), tr!(live_card_lamp_worker_what), None, vec![tr!(live_card_lamp_worker_good)], vec![tr!(live_card_lamp_worker_bad)])),
    ("lamp:tile", || c(tr!(live_lamp_tile), tr!(live_card_lamp_tile_what), None, vec![tr!(live_card_lamp_tile_good)], vec![tr!(live_card_lamp_tile_bad)])),
    ("lamp:ssaa", || c(tr!(live_lamp_ssaa), tr!(live_card_lamp_ssaa_what), None, vec![tr!(live_card_lamp_ssaa_good)], vec![tr!(live_card_lamp_ssaa_bad)])),
    ("lamp:palette", || c(tr!(live_lamp_palette), tr!(live_card_lamp_palette_what), None, vec![tr!(live_card_lamp_palette_good)], vec![tr!(live_card_lamp_palette_bad)])),
    ("lamp:mode", || c(tr!(live_lamp_mode), tr!(live_card_lamp_mode_what), None, vec![tr!(live_card_lamp_mode_good)], vec![tr!(live_card_lamp_mode_bad)])),
];

#[cfg(test)]
#[path = "../../tests/unit/blinken/help/tests.rs"]
mod tests;
