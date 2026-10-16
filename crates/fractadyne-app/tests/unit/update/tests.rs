use super::{
    cmp_ver, default_track_for, is_prerelease, safe_release_url, version_gt, UpdateTrack,
    RELEASES_URL,
};
use std::cmp::Ordering;

#[test]
fn a_normal_github_release_url_passes_through_untouched() {
    let url = "https://github.com/WindySnowOwl/fractadyne/releases/tag/v0.2.41-beta.105";
    assert_eq!(safe_release_url(url), url);
    // A path or query on github.com is still fine.
    let q = "https://github.com/WindySnowOwl/fractadyne/releases?page=2#latest";
    assert_eq!(safe_release_url(q), q);
}

#[test]
fn a_hostile_or_malformed_html_url_falls_back_to_the_releases_page() {
    for bad in [
        "http://github.com/WindySnowOwl/fractadyne/releases", // not https
        "file:///C:/Windows/System32/calc.exe",              // dangerous scheme
        "javascript:alert(1)",                               // dangerous scheme
        "https://evil.com/WindySnowOwl/fractadyne/releases", // foreign host
        "https://github.com@evil.com/pwn",                   // userinfo spoof — real host evil.com
        "https://github.com.evil.com/pwn",                   // look-alike host
        "https://notgithub.com/x",                           // suffix confusion
        "",                                                   // empty
        "ftp://github.com/x",                                // wrong scheme
    ] {
        assert_eq!(safe_release_url(bad), RELEASES_URL, "must not open {bad:?} verbatim");
    }
}

#[test]
fn a_prerelease_build_starts_on_the_beta_track_and_a_stable_one_does_not() {
    assert_eq!(default_track_for("0.2.41-beta.78"), UpdateTrack::Beta);
    assert_eq!(default_track_for("v0.2.41-beta.78"), UpdateTrack::Beta);
    assert_eq!(default_track_for("0.3.0-rc.1"), UpdateTrack::Beta);
    assert_eq!(default_track_for("0.2.40"), UpdateTrack::Stable);
    assert_eq!(default_track_for("1.0.0"), UpdateTrack::Stable);
    // The reading must be the comparator's: what `version_gt` ranks as a pre-release of its
    // core is exactly what starts on the beta track, so the two can never disagree.
    for v in ["0.2.41-beta.78", "0.3.0-rc.1", "0.3.0-beta"] {
        assert!(is_prerelease(v));
        assert!(version_gt(v.split('-').next().unwrap(), v), "{v} is not below its own core");
    }
    // And THIS build decides by its own version string — pin the mapping so a change in how
    // the version is spelled cannot silently move a beta's first launch onto Stable.
    let compiled = env!("CARGO_PKG_VERSION");
    assert_eq!(is_prerelease(compiled), compiled.contains('-'));
}

#[test]
fn beta_picks_newest_of_either_channel() {
    // Graduation: once stable ships it outranks its own prerelease, so a Beta user moves to it.
    assert_eq!(cmp_ver("0.2.40", "0.2.40-beta.3"), Ordering::Greater);
    // But a newer beta line still wins over the shipped stable.
    assert_eq!(cmp_ver("0.2.41-beta.1", "0.2.40"), Ordering::Greater);
    // Same version → equal (max_by keeps either; version_gt vs `current` decides Available).
    assert_eq!(cmp_ver("0.2.40", "0.2.40"), Ordering::Equal);
    // Highest-semver, not newest-by-date: an out-of-order older prerelease loses to newer stable.
    assert_eq!(cmp_ver("0.2.39-beta.4", "0.2.40"), Ordering::Less);
}
#[test]
fn semver_ordering() {
    assert!(version_gt("0.3.0", "0.2.38"));
    assert!(version_gt("1.0.0", "0.9.9"));
    assert!(!version_gt("0.2.38", "0.2.38"));
    assert!(!version_gt("0.2.37", "0.2.38"));
    // Prerelease ordering.
    assert!(version_gt("0.3.0", "0.3.0-beta.1")); // release > its prerelease
    assert!(!version_gt("0.3.0-beta.1", "0.3.0")); // prerelease < release
    assert!(version_gt("0.3.0-beta.2", "0.3.0-beta.1"));
    assert!(version_gt("0.3.0-beta.1", "0.2.38")); // newer core, even as a prerelease
}

#[test]
fn multi_digit_prereleases_compare_numerically() {
    // ⭐THE FIELD BUG, verbatim: 0.2.40-beta.105 was offered 0.2.40-beta.78 as an "update",
    // because a String comparison puts "beta.78" above "beta.105" ('7' > '1'). Every test above
    // uses single-digit betas, which is precisely why this survived to beta.105.
    assert!(version_gt("0.2.40-beta.105", "0.2.40-beta.78"));
    assert!(!version_gt("0.2.40-beta.78", "0.2.40-beta.105"));
    assert_eq!(cmp_ver("0.2.40-beta.78", "0.2.40-beta.105"), Ordering::Less);

    // The boundary where lexical ordering first breaks: 9 vs 10.
    assert!(version_gt("0.2.40-beta.10", "0.2.40-beta.9"));
    assert!(!version_gt("0.2.40-beta.9", "0.2.40-beta.10"));

    // And the case that actually matters for the shelf: the newest beta must win the Beta
    // track, whatever else is published alongside it.
    let published = ["0.2.36", "0.2.40-beta.78", "0.2.40-beta.104", "0.2.40-beta.105"];
    let newest = published.iter().copied().max_by(|a, b| cmp_ver(a, b)).unwrap();
    assert_eq!(newest, "0.2.40-beta.105");
    // A build ON that newest release must be told there is nothing to get.
    assert!(!version_gt(newest, "0.2.40-beta.105"));
}

#[test]
fn prerelease_identifier_kinds_order_per_semver() {
    // rc outranks beta (alphanumeric, lexical), and numeric sorts below alphanumeric.
    assert!(version_gt("0.3.0-rc.1", "0.3.0-beta.9"));
    assert!(version_gt("0.3.0-rc.1", "0.3.0-beta.105"));
    // Fewer identifiers sorts lower when the common prefix matches (semver §11).
    assert!(version_gt("0.3.0-beta.1", "0.3.0-beta"));
    assert!(!version_gt("0.3.0-beta", "0.3.0-beta.1"));
}
