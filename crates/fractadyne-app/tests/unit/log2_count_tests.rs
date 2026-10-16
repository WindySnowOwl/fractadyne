use super::fmt_log2_count;

/// ⭐The reported case: the "view-widths away" message rendered 23 as `2.3184272282471518e1`,
/// because it reused the MAGNIFICATION formatter. A count is not a magnification — small values
/// are the common case here and they have to read like numbers.
#[test]
fn a_count_reads_like_a_number() {
    assert_eq!(fmt_log2_count(23.184272282471518f64.log2()), "23");
    assert_eq!(fmt_log2_count(0.0), "1");
    assert_eq!(fmt_log2_count(3.0), "8");
    assert_eq!(fmt_log2_count(10.0), "1,024");
    // ...and stays readable where the count is astronomical, which is the other real case:
    // the hand-typed (5,332) at a 2.77e89x dendrite was 2^187 view-widths out.
    assert_eq!(fmt_log2_count(187.0), "2.0e56");
    // Past 2^1024 a linear count is not even representable, so this must not go through f64.
    assert_eq!(fmt_log2_count(3_000.0), "1.2e903");
}
