use super::{accelerated_asset_url, accelerated_package_exists_for_platform};

/// The artifact name each platform's packager actually produces. Renaming either side without the
/// other kills the in-app link silently, and a dead download link in a menu is worse than none.
///   Windows: `scripts/build-accelerated.ps1` → `…-windows-x64-accelerated.zip`
///   Linux:   `scripts/build-accelerated.sh`  → `…-linux-x64-accelerated.tar.gz`
const EXPECTED_SUFFIX: &str = if cfg!(target_os = "windows") {
    "-windows-x64-accelerated.zip"
} else {
    "-linux-x64-accelerated.tar.gz"
};

#[test]
fn the_download_url_is_well_formed_and_version_matched() {
    let u = accelerated_asset_url("0.2.40-beta.156 (build 2076)");
    let want = format!(
        "https://github.com/WindySnowOwl/fractadyne/releases/download/\
         v0.2.40-beta.156/fractadyne-v0.2.40-beta.156{EXPECTED_SUFFIX}"
    )
    .replace(' ', "");
    assert_eq!(u, want);
    // A URL containing a space is a dead link, and a `\` continuation inside a Rust string
    // literal is how one gets there. This is the assertion that catches it.
    assert!(!u.contains(' '), "URL contains a space: {u}");
    assert!(u.starts_with("https://"), "{u}");
    // Must match the artifact name this platform's packaging script builds.
    assert!(u.ends_with(EXPECTED_SUFFIX), "{u}");
}

#[test]
fn a_bare_version_without_a_build_suffix_also_works() {
    let u = accelerated_asset_url("0.3.0");
    assert!(u.contains("/v0.3.0/"), "{u}");
    assert!(!u.contains(' '), "{u}");
}

/// ⭐The link must be PLATFORM-matched, not just version-matched: handing a Linux user the
/// Windows `.zip` resolves, downloads, and is useless — the same class of failure as the wrong
/// version, which is what the rest of this file exists to prevent.
#[test]
fn the_url_names_this_platforms_package() {
    let u = accelerated_asset_url("0.3.0");
    if cfg!(target_os = "windows") {
        assert!(u.ends_with(".zip"), "windows must be offered the zip: {u}");
        assert!(!u.contains("linux"), "{u}");
    } else {
        assert!(u.ends_with(".tar.gz"), "linux must be offered the tarball: {u}");
        assert!(!u.contains("windows"), "{u}");
    }
}

/// Both platforms that `release.yml` builds a package for must advertise one; the publish step
/// blocks on both jobs, so a published release always carries the asset the link names.
#[test]
fn the_built_platforms_offer_a_download() {
    assert_eq!(
        accelerated_package_exists_for_platform(),
        cfg!(any(target_os = "windows", target_os = "linux"))
    );
}
