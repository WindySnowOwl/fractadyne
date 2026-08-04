//! In-app update check against the public GitHub Releases API. `check()` does a single blocking
//! HTTPS GET (run on a background thread by the caller) and reports whether a newer release exists
//! on the chosen track. No auto-install — the UI just offers a link to the release page.

const REPO: &str = "WindySnowOwl/fractadyne";

/// Which release track the update check follows.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum UpdateTrack {
    /// Latest stable release (GitHub "latest", excludes pre-releases).
    #[default]
    Stable,
    /// Newest release including pre-releases (the `-beta.N` / `-rc.N` track).
    Beta,
}
impl UpdateTrack {
    pub(crate) const ALL: [UpdateTrack; 2] = [UpdateTrack::Stable, UpdateTrack::Beta];
    pub(crate) fn label(self) -> &'static str {
        match self {
            UpdateTrack::Stable => "Stable",
            UpdateTrack::Beta => "Beta (pre-releases)",
        }
    }
    /// Persisted token in `SessionState`.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            UpdateTrack::Stable => "stable",
            UpdateTrack::Beta => "beta",
        }
    }
    pub(crate) fn from_str(s: &str) -> Self {
        match s {
            "beta" => UpdateTrack::Beta,
            _ => UpdateTrack::Stable,
        }
    }
}

/// Outcome of an update check (delivered from the worker thread to the UI).
pub(crate) enum UpdateStatus {
    /// A newer release is available on the track: its version tag + release-page URL.
    Available { version: String, url: String },
    /// Running the newest release available on the track.
    UpToDate,
    /// The check couldn't complete (offline, API/rate-limit error, …).
    Error(String),
}

/// Blocking update check. `current` is the running semver (e.g. `env!("CARGO_PKG_VERSION")`).
pub(crate) fn check(track: UpdateTrack, current: &str) -> UpdateStatus {
    match fetch_latest(track) {
        Ok(Some((tag, url))) => {
            if version_gt(strip_v(&tag), current) {
                UpdateStatus::Available { version: tag, url }
            } else {
                UpdateStatus::UpToDate
            }
        }
        Ok(None) => UpdateStatus::UpToDate, // no release on this track
        Err(e) => UpdateStatus::Error(e),
    }
}

/// The latest release `(tag_name, html_url)` for the track, or `None` if the track has no release.
fn fetch_latest(track: UpdateTrack) -> Result<Option<(String, String)>, String> {
    let ua = concat!("fractadyne/", env!("CARGO_PKG_VERSION"));
    match track {
        UpdateTrack::Stable => {
            let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
            match get(&url, ua)? {
                Some(body) => {
                    let v: serde_json::Value =
                        serde_json::from_str(&body).map_err(|e| e.to_string())?;
                    Ok(rel_of(&v))
                }
                None => Ok(None), // 404 = no stable release yet
            }
        }
        UpdateTrack::Beta => {
            // Newest of all releases (GitHub returns newest-first); skip drafts.
            let url = format!("https://api.github.com/repos/{REPO}/releases?per_page=15");
            match get(&url, ua)? {
                Some(body) => {
                    let arr: serde_json::Value =
                        serde_json::from_str(&body).map_err(|e| e.to_string())?;
                    let newest = arr.as_array().and_then(|a| {
                        a.iter().find(|r| {
                            !r.get("draft").and_then(|d| d.as_bool()).unwrap_or(false)
                        })
                    });
                    Ok(newest.and_then(rel_of))
                }
                None => Ok(None),
            }
        }
    }
}

fn rel_of(v: &serde_json::Value) -> Option<(String, String)> {
    let tag = v.get("tag_name")?.as_str()?.to_string();
    let url = v
        .get("html_url")
        .and_then(|u| u.as_str())
        .unwrap_or("")
        .to_string();
    Some((tag, url))
}

/// GitHub API GET. `Ok(None)` on 404 (not-found is a normal "no release" answer, not an error).
fn get(url: &str, ua: &str) -> Result<Option<String>, String> {
    match ureq::get(url)
        .set("User-Agent", ua) // GitHub rejects requests without one
        .set("Accept", "application/vnd.github+json")
        .timeout(std::time::Duration::from_secs(10))
        .call()
    {
        Ok(resp) => resp.into_string().map(Some).map_err(|e| e.to_string()),
        Err(ureq::Error::Status(404, _)) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

fn strip_v(s: &str) -> &str {
    s.strip_prefix('v').unwrap_or(s)
}

/// Parse `"0.3.0-beta.1"` → `((0,3,0), Some("beta.1"))`.
fn parse_ver(s: &str) -> ((u32, u32, u32), Option<String>) {
    let (core, pre) = match s.split_once('-') {
        Some((c, p)) => (c, Some(p.to_string())),
        None => (s, None),
    };
    let mut it = core.split('.').map(|x| x.trim().parse::<u32>().unwrap_or(0));
    (
        (it.next().unwrap_or(0), it.next().unwrap_or(0), it.next().unwrap_or(0)),
        pre,
    )
}

/// True if `cand` is a strictly newer semver than `cur`. Prerelease ordering (semver §11): a
/// release outranks its own pre-releases; two pre-releases of the same core compare by identifier
/// (`beta.2 > beta.1`, `rc.1 > beta.9` — lexical is adequate for our `beta.N`/`rc.N` scheme).
fn version_gt(cand: &str, cur: &str) -> bool {
    let (cc, cp) = parse_ver(cand);
    let (uc, up) = parse_ver(cur);
    if cc != uc {
        return cc > uc;
    }
    match (cp, up) {
        (None, None) => false,
        (None, Some(_)) => true,
        (Some(_), None) => false,
        (Some(a), Some(b)) => a > b,
    }
}

#[cfg(test)]
mod tests {
    use super::version_gt;
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
}
