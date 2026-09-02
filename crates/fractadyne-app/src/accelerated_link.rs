use super::accelerated_asset_url;

#[test]
fn the_download_url_is_well_formed_and_version_matched() {
    let u = accelerated_asset_url("0.2.40-beta.156 (build 2076)");
    assert_eq!(
        u,
        "https://github.com/WindySnowOwl/fractadyne/releases/download/\
         v0.2.40-beta.156/fractadyne-v0.2.40-beta.156-windows-x64-accelerated.zip"
            .replace(' ', "")
    );
    // A URL containing a space is a dead link, and a `\` continuation inside a Rust string
    // literal is how one gets there. This is the assertion that catches it.
    assert!(!u.contains(' '), "URL contains a space: {u}");
    assert!(u.starts_with("https://"), "{u}");
    // Must match the artifact name `scripts/build-accelerated.ps1` builds.
    assert!(u.ends_with("-windows-x64-accelerated.zip"), "{u}");
}

#[test]
fn a_bare_version_without_a_build_suffix_also_works() {
    let u = accelerated_asset_url("0.3.0");
    assert!(u.contains("/v0.3.0/"), "{u}");
    assert!(!u.contains(' '), "{u}");
}
