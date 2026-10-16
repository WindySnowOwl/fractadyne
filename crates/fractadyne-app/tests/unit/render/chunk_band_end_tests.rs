use super::*;

#[test]
fn band_ends_are_the_octave_edges() {
    assert_eq!(chunk_band_end(0), 256);
    assert_eq!(chunk_band_end(255), 256);
    assert_eq!(chunk_band_end(256), 512);
    assert_eq!(chunk_band_end(511), 512);
    assert_eq!(chunk_band_end(512), 1024);
    assert_eq!(chunk_band_end(9_544), 16_384);
    assert_eq!(chunk_band_end(16_384), 32_768);
    // Every end is the start of the next band, and each start maps back to its band.
    for cur in [0u32, 1, 256, 300, 1_000, 9_544, 100_000, 3_000_000] {
        let end = chunk_band_end(cur);
        if end != u32::MAX {
            assert_eq!(chunk_band_of(end), chunk_band_of(cur) + 1, "cur {cur}");
            assert_eq!(chunk_band_of(end - 1), chunk_band_of(cur), "cur {cur}");
        }
    }
}
