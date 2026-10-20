use super::*;

/// A colour's relative luminance (WCAG 2): its sRGB channels linearized and weighted.
fn luminance(c: Color32) -> f32 {
    let lin = |v: u8| {
        let v = v as f32 / 255.0;
        if v <= 0.040_45 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
    };
    let [r, g, b, _] = c.to_srgba_unmultiplied();
    0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b)
}

/// The WCAG contrast ratio between two colours (1 to 21).
fn contrast(a: Color32, b: Color32) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

fn tile() -> egui::ColorImage {
    hammertone(HAMMER_TILE, 0x4a11_e7)
}

fn lum_at(img: &egui::ColorImage, x: usize, y: usize) -> f32 {
    luminance(img.pixels[y * img.size[0] + x])
}

/// The `q`-quantile (0..1) of the tile's texel luminances.
fn quantile(img: &egui::ColorImage, q: f32) -> f32 {
    let mut l: Vec<f32> = img.pixels.iter().map(|&c| luminance(c)).collect();
    l.sort_by(f32::total_cmp);
    l[((l.len() - 1) as f32 * q) as usize]
}

/// The texel whose luminance is the `q`-quantile.
fn texel_at_quantile(img: &egui::ColorImage, q: f32) -> Color32 {
    let target = quantile(img, q);
    *img.pixels.iter().min_by(|a, b| (luminance(**a) - target).abs().total_cmp(&(luminance(**b) - target).abs())).unwrap()
}

/// `c` under the legend halo's black at `HALO_ALPHA`, blended as egui blends (in gamma space).
fn under_halo(c: Color32) -> Color32 {
    let k = 1.0 - HALO_ALPHA as f32 / 255.0;
    let [r, g, b, _] = c.to_srgba_unmultiplied();
    Color32::from_rgb((r as f32 * k).round() as u8, (g as f32 * k).round() as u8, (b as f32 * k).round() as u8)
}

#[test]
fn the_hammertone_is_the_same_every_time() {
    assert_eq!(tile().pixels, tile().pixels);
    assert_ne!(tile().pixels, hammertone(HAMMER_TILE, 1).pixels, "the seed matters");
}

#[test]
fn the_hammertone_tiles_without_a_seam() {
    // The step across the wrap (last column to first, last row to first) is no rougher than the
    // tile's own steps between neighbouring columns and rows.
    let img = tile();
    let n = HAMMER_TILE;
    let col_step = |a: usize, b: usize| (0..n).map(|y| (lum_at(&img, a, y) - lum_at(&img, b, y)).abs()).sum::<f32>() / n as f32;
    let row_step = |a: usize, b: usize| (0..n).map(|x| (lum_at(&img, x, a) - lum_at(&img, x, b)).abs()).sum::<f32>() / n as f32;
    let mut cols: Vec<f32> = (0..n - 1).map(|x| col_step(x, x + 1)).collect();
    let mut rows: Vec<f32> = (0..n - 1).map(|y| row_step(y, y + 1)).collect();
    cols.sort_by(f32::total_cmp);
    rows.sort_by(f32::total_cmp);
    let p99 = |v: &[f32]| v[(v.len() - 1) * 99 / 100];
    assert!(col_step(n - 1, 0) <= p99(&cols), "column seam {} vs interior p99 {}", col_step(n - 1, 0), p99(&cols));
    assert!(row_step(n - 1, 0) <= p99(&rows), "row seam {} vs interior p99 {}", row_step(n - 1, 0), p99(&rows));
}

#[test]
fn the_hammertone_is_a_finish_not_a_flat_colour() {
    // A guard that cannot go red is not a gate: the legibility test below would pass on a flat
    // tile, so the tile must actually vary.
    let img = tile();
    let (lo, hi) = (quantile(&img, 0.01), quantile(&img, 0.99));
    assert!(hi / lo > 2.0, "luminance p1 {lo} to p99 {hi}: too flat to read as hammered");
}

#[test]
fn every_legend_on_the_case_reads_against_its_brightest_texels() {
    let img = tile();
    let median = texel_at_quantile(&img, 0.5);
    let bright = texel_at_quantile(&img, 0.995);
    for (name, c) in [("ENGRAVE", ink::ENGRAVE), ("ENGRAVE_DIM", ink::ENGRAVE_DIM)] {
        // WCAG AA for small text, against the typical texel and against a glint under the halo
        // every engraved legend carries.
        let typical = contrast(c, median);
        let glint = contrast(c, under_halo(bright));
        assert!(typical >= 4.5, "{name} against the median texel {median:?}: {typical:.2}:1");
        assert!(glint >= 4.5, "{name} against the p99.5 texel {bright:?} under its halo: {glint:.2}:1");
    }
    // The maker's mark is large text on its own badge: 3:1.
    let mark = contrast(ink::MAKER_RED, ink::FACE_DARK);
    assert!(mark >= 3.0, "maker's mark on its badge: {mark:.2}:1");
}

#[test]
fn contrast_is_wcags_ratio() {
    assert!((contrast(Color32::BLACK, Color32::WHITE) - 21.0).abs() < 0.01);
    assert!((contrast(Color32::WHITE, Color32::WHITE) - 1.0).abs() < 1e-6);
    // #767676 on white is the textbook 4.54:1.
    assert!((contrast(Color32::from_rgb(0x76, 0x76, 0x76), Color32::WHITE) - 4.54).abs() < 0.01);
}
