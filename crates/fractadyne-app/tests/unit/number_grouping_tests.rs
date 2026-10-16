//! Comma grouping for the Controls panel, and the reader that lets a grouped value box be edited.

use crate::{commas, fmt_coord, grouped_count, parse_grouped_number};

#[test]
fn the_julia_parameter_rows_group_in_fives_like_the_centre() {
    // The Performance section's `julia c.re` / `julia c.im` rows now call `fmt_coord`, the status
    // bar's formatter, instead of `{:+.15}`. These are the parameter values from the 1.08e66 view.
    assert_eq!(fmt_coord(-1.63181519400247554e-1), "-0.16318 15194 00248");
    assert_eq!(fmt_coord(6.50116979262094108e-1), "+0.65011 69792 62094");
    // Same digits as before, only grouped: fifteen decimals either way.
    let old = format!("{:+.15}", 6.50116979262094108e-1_f64);
    assert_eq!(fmt_coord(6.50116979262094108e-1).replace(' ', ""), old);
}

#[test]
fn integers_group_in_threes() {
    assert_eq!(commas("7"), "7");
    assert_eq!(commas("999"), "999");
    assert_eq!(commas("1000"), "1,000");
    assert_eq!(commas("257280"), "257,280");
    assert_eq!(commas("10000000"), "10,000,000");
    assert_eq!(commas("-7452445"), "-7,452,445");
}

#[test]
fn only_the_integer_part_of_a_decimal_is_grouped() {
    // The Performance section formats milliseconds as `{:.2}`; an idle gap measured 92 seconds.
    assert_eq!(commas("92151.25"), "92,151.25");
    assert_eq!(commas("12.24"), "12.24");
    assert_eq!(commas("-1234.5"), "-1,234.5");
    // ⭐Fractional digits are never comma-grouped — a long fraction stays one run.
    assert_eq!(commas("1234.56789"), "1,234.56789");
}

#[test]
fn a_slider_count_rounds_and_groups() {
    assert_eq!(grouped_count(250000.0), "250,000");
    assert_eq!(grouped_count(10_000_000.0), "10,000,000");
    assert_eq!(grouped_count(1023.6), "1,024");
    assert_eq!(grouped_count(64.0), "64");
}

#[test]
fn the_value_box_reads_back_what_it_shows() {
    // ⭐The round trip is the property: whatever the box displays must parse to the same value.
    for n in [64.0, 1024.0, 250_000.0, 10_000_000.0, 123_456_789.0] {
        assert_eq!(parse_grouped_number(&grouped_count(n)), Some(n), "{n}");
    }
    // And the forms a person types still work.
    assert_eq!(parse_grouped_number("250000"), Some(250_000.0));
    assert_eq!(parse_grouped_number(" 1,024 it"), Some(1024.0));
    assert_eq!(parse_grouped_number("1e6"), Some(1_000_000.0));
    assert_eq!(parse_grouped_number("2_000"), Some(2000.0));
}

#[test]
fn garbage_is_refused_rather_than_misread() {
    // `None` makes egui keep the previous value — never a surprise number.
    assert_eq!(parse_grouped_number(""), None);
    assert_eq!(parse_grouped_number("lots"), None);
    assert_eq!(parse_grouped_number("1,2,3.4.5"), None);
}
