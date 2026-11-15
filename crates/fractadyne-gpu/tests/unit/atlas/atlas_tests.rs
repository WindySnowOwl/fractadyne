use crate::atlas_tap_stride;

/// The samples a row of one pixel's block the color pass reads (`fs_color`'s tap loop): texels
/// `first, first + stride, …` below `n`, `first` = stride / 2.
fn taps_a_row(n: u32, stride: u32) -> u32 {
    let stride = stride.clamp(1, n.max(1));
    (stride / 2..n).step_by(stride as usize).count() as u32
}

/// A palette-browser thumbnail reads at most 2×2 samples a pixel, whatever the view's supersampling
/// (2026-11-15: all 64 at 8x were ~2e9 colorings for one pane's atlas, a second of GPU time), and
/// every sample at ss ≤ 2, so those thumbnails are the bytes they were.
#[test]
fn a_thumbnail_reads_at_most_two_by_two_samples_and_all_of_them_up_to_ss_2() {
    for ss in 1..=16 {
        let stride = atlas_tap_stride(ss);
        let n = taps_a_row(ss, stride);
        assert!((1..=2).contains(&n), "ss {ss}: stride {stride} reads {n} a row");
        if ss <= 2 {
            assert_eq!((stride, n), (1, ss), "ss {ss}: every sample, as before");
        }
    }
    // ss 8: texels 2 and 6 of each row of eight, centered on the block.
    assert_eq!(atlas_tap_stride(8), 4);
    assert_eq!((4 / 2..8).step_by(4).collect::<Vec<u32>>(), vec![2, 6]);
    // The view's own passes leave it at 0: every texel, from the first.
    assert_eq!(taps_a_row(8, 0), 8);
}
