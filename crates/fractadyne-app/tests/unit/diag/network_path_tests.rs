use super::is_network_path;
use std::path::Path;

/// The shapes a share takes on Windows, and the local ones it must not be confused with.
#[cfg(windows)]
#[test]
fn a_unc_path_is_a_share_and_the_temp_dir_is_not() {
    assert!(is_network_path(Path::new(r"\\fileserver\share\Fractadyne\config\logs")));
    assert!(is_network_path(Path::new("//fileserver/share/x")));
    assert!(is_network_path(Path::new(r"\\?\UNC\fileserver\share\x")));
    let tmp = std::env::temp_dir();
    assert!(!is_network_path(&tmp), "{} read as a share", tmp.display());
    let verbatim = std::path::PathBuf::from(format!(r"\\?\{}", tmp.display()));
    assert!(!is_network_path(&verbatim), "a verbatim LOCAL path is not a share: {}", verbatim.display());
    assert!(!is_network_path(Path::new("relative\\logs")));
}

#[cfg(not(windows))]
#[test]
fn the_temp_dir_is_not_a_share() {
    assert!(!is_network_path(&std::env::temp_dir()));
    assert!(!is_network_path(Path::new("/definitely/not/a/path")));
}
