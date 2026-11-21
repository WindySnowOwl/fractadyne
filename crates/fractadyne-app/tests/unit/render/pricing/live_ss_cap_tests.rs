//! The live G-buffer's share of the card's memory (`live_ss_cap`).

use super::*;

/// The texels a view's G-buffer holds at `ss`.
fn texels(px: u64, ss: u32) -> u64 {
    px * (ss as u64) * (ss as u64)
}

#[test]
fn the_dual_view_that_ran_out_of_memory_is_held_to_ss_3() {
    // The user's 2026-11-20 session: a 10 GB RTX 3080, two panes of a 5048-pixel-wide window.
    let (v0, v1) = (2638u64 * 2060, 1937u64 * 2060);
    assert_eq!(live_ss_cap(10_240, v0, 2), 3);
    assert_eq!(live_ss_cap(10_240, v1, 2), 3);
    // At rest both together take at most a third of the card.
    let bytes = (texels(v0, 3) + texels(v1, 3)) * crate::tunables::LIVE_GBUFFER_TEXEL_BYTES;
    assert!(bytes <= (10_240u64 << 20) / crate::tunables::LIVE_GBUFFER_VRAM_DIVISOR, "{bytes}");
}

#[test]
fn an_ordinary_window_keeps_its_full_supersampling() {
    // A single 1920x1200 view on the same card: ss 4 is 1.1 GB, well inside a third.
    assert!(live_ss_cap(10_240, 1920 * 1200, 1) >= 4);
    // A 4K view: ss 4 would be 4.2 GB, ss 3 is 2.4 GB.
    assert_eq!(live_ss_cap(10_240, 3840 * 2160, 1), 3);
}

#[test]
fn a_small_card_gets_less() {
    // A 2 GB card (the GT 710 class) at 1920x1080: ss 4 would be 1.06 GB alone, ss 3 is 0.6 GB.
    assert_eq!(live_ss_cap(2_048, 1920 * 1080, 1), 3);
    assert_eq!(live_ss_cap(2_048, 1280 * 720, 1), 4);
    // The same card's dual view: each pane gets half the share.
    assert_eq!(live_ss_cap(2_048, 960 * 1080, 2), 3);
    assert_eq!(live_ss_cap(2_048, 1920 * 1080, 2), 2);
}

#[test]
fn it_is_the_exact_floor_of_the_root() {
    // Construct a pixel count where texels/px is a perfect square, then one texel short of it.
    let vram_mb = 3 * 32; // one third → 32 MB → 1 Mi texels for one view
    let texels_total = ((vram_mb as u64) << 20) / crate::tunables::LIVE_GBUFFER_VRAM_DIVISOR / crate::tunables::LIVE_GBUFFER_TEXEL_BYTES;
    assert_eq!(texels_total, 1 << 20);
    assert_eq!(live_ss_cap(vram_mb, (1 << 20) / 16, 1), 4); // exactly 16 texels a pixel
    assert_eq!(live_ss_cap(vram_mb, (1 << 20) / 16 + 1, 1), 3); // a hair under 16
}

#[test]
fn never_below_one_and_unknown_memory_is_a_modest_card() {
    assert_eq!(live_ss_cap(64, 16_384 * 16_384, 1), 1);
    assert_eq!(live_ss_cap(10_240, 0, 1), live_ss_cap(10_240, 1, 1), "an empty panel is one pixel");
    // Unknown memory is taken as 4 GB.
    assert_eq!(live_ss_cap(0, 1920 * 1080, 1), live_ss_cap(crate::tunables::LIVE_GBUFFER_UNKNOWN_VRAM_MB, 1920 * 1080, 1));
    // Two panes share one card's share.
    assert!(live_ss_cap(10_240, 2560 * 1440, 2) <= live_ss_cap(10_240, 2560 * 1440, 1));
}
