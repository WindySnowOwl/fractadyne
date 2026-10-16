//! In-app update check against the public GitHub Releases API. `check()` does a single blocking
//! HTTPS GET (run on a background thread by the caller) and reports whether a newer release exists
//! on the chosen track. No auto-install — the UI just offers a link to the release page.

const REPO: &str = "WindySnowOwl/fractadyne";

/// The canonical releases page. Used as the safe fallback whenever a release's `html_url` fails
/// validation (F-07) — always a known-good destination the user can act on.
pub const RELEASES_URL: &str = "https://github.com/WindySnowOwl/fractadyne/releases";

/// Return `url` only when it is an `https` URL on the `github.com` host; otherwise the canonical
/// [`RELEASES_URL`].
///
/// ⭐**Why (F-07).** A release's `html_url` comes from a GitHub API response we do not fully trust
/// to stay well-formed — a proxy, an unexpected field, or compromised release metadata could put
/// an arbitrary string here, and it is handed straight to the OS URL handler (`ctx.open_url`). A
/// non-`https` scheme (`file:`, a custom handler) or a foreign host must never reach the handler,
/// so validate scheme AND host and fall back to the hard-coded releases page on anything else.
pub fn safe_release_url(url: &str) -> String {
    if is_trusted_github_https(url) {
        url.to_string()
    } else {
        RELEASES_URL.to_string()
    }
}

/// `https` scheme, host exactly `github.com`. A dependency-free parse deliberately robust to the
/// classic spoofs: userinfo (`https://github.com@evil.com/`), a foreign host with `github.com` in
/// the path, and look-alike hosts (`github.com.evil.com`).
fn is_trusted_github_https(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("https://") else {
        return false; // scheme must be exactly https (rejects http:, file:, custom handlers)
    };
    // Authority is everything up to the first '/', '?' or '#'.
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    // Any userinfo precedes the LAST '@'; the real host is what follows it.
    let host_port = authority.rsplit('@').next().unwrap_or("");
    // Drop a :port suffix, then compare case-insensitively.
    let host = host_port.split(':').next().unwrap_or("").to_ascii_lowercase();
    host == "github.com"
}

/// Which release track the update check follows.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
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
    /// A newer release is available on the track: its version tag, release-page URL, and whether
    /// it's a pre-release (a Beta-track user can be offered a *stable* build — see `fetch_latest`).
    Available { version: String, url: String, prerelease: bool },
    /// Running the newest release available on the track.
    UpToDate,
    /// The check couldn't complete (offline, API/rate-limit error, …).
    Error(String),
}

/// UI word for the channel a release came from ("beta" for a pre-release, "stable" otherwise).
pub(crate) fn channel_word(prerelease: bool) -> &'static str {
    if prerelease {
        "beta"
    } else {
        "stable"
    }
}

/// The version to compare against releases — normally the compiled version, but overridable via
/// the `FRACTADYNE_FAKE_VERSION` env var so the "update available" path (CLI and the in-app
/// prompt) can be exercised while the running build is already current.
pub(crate) fn running_version() -> String {
    std::env::var("FRACTADYNE_FAKE_VERSION").unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string())
}

/// Is `version` a pre-release (`0.2.41-beta.78`, `0.3.0-rc.1`), by the same reading the
/// comparator uses — a `-` suffix after the core triple. A leading `v` is tolerated.
pub(crate) fn is_prerelease(version: &str) -> bool {
    parse_ver(strip_v(version)).1.is_some()
}

/// The track a build with NO saved choice starts on: **Beta for a pre-release build, Stable
/// otherwise.** The first public announcement points at a beta, and someone who downloads a beta
/// should hear about the next one without first discovering View ▸ Settings ▸ Updates; someone on
/// a stable build did not ask for pre-releases and is not moved onto them. Applied only where no
/// choice exists — a fresh install, an unreadable session, a bare `--check-updates` — never over a
/// saved track.
pub(crate) fn default_track_for(version: &str) -> UpdateTrack {
    if is_prerelease(version) {
        UpdateTrack::Beta
    } else {
        UpdateTrack::Stable
    }
}

/// Blocking update check. `current` is the running semver (e.g. `env!("CARGO_PKG_VERSION")`).
pub(crate) fn check(track: UpdateTrack, current: &str) -> UpdateStatus {
    match fetch_latest(track) {
        Ok(Some(rel)) => {
            if version_gt(strip_v(&rel.tag), current) {
                UpdateStatus::Available {
                    version: rel.tag,
                    url: rel.url,
                    prerelease: rel.prerelease,
                }
            } else {
                UpdateStatus::UpToDate
            }
        }
        Ok(None) => UpdateStatus::UpToDate, // no release on this track
        Err(e) => UpdateStatus::Error(e),
    }
}

/// A release candidate.
struct Rel {
    tag: String,
    url: String,
    prerelease: bool,
}

/// The release a track should offer, or `None` if the track has no release.
///
/// * **Stable** → GitHub's "latest" (`/releases/latest`), which excludes pre-releases.
/// * **Beta** → the **highest-semver** non-draft release across `/releases` — stable OR
///   pre-release. Highest-semver (not newest-by-date) means a Beta user always gets the newest of
///   either channel: they graduate to `X.Y.Z` stable once it ships (it outranks its own
///   `X.Y.Z-beta.N`), yet still pick up a newer `X.Y.(Z+1)-beta.1` when one lands — regardless of
///   publish order.
fn fetch_latest(track: UpdateTrack) -> Result<Option<Rel>, String> {
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
            let url = format!("https://api.github.com/repos/{REPO}/releases?per_page=30");
            match get(&url, ua)? {
                Some(body) => {
                    let arr: serde_json::Value =
                        serde_json::from_str(&body).map_err(|e| e.to_string())?;
                    let best = arr
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter(|r| !r.get("draft").and_then(|d| d.as_bool()).unwrap_or(false))
                        .filter_map(rel_of)
                        .max_by(|x, y| cmp_ver(strip_v(&x.tag), strip_v(&y.tag)));
                    Ok(best)
                }
                None => Ok(None),
            }
        }
    }
}

fn rel_of(v: &serde_json::Value) -> Option<Rel> {
    let tag = v.get("tag_name")?.as_str()?.to_string();
    let url = v
        .get("html_url")
        .and_then(|u| u.as_str())
        .unwrap_or("")
        .to_string();
    // Prefer GitHub's explicit flag; fall back to the tag's `-` pre-release suffix.
    let prerelease = v
        .get("prerelease")
        .and_then(|b| b.as_bool())
        .unwrap_or_else(|| tag.contains('-'));
    Some(Rel { tag, url, prerelease })
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

/// Compare two prerelease strings by semver §11: dot-separated identifiers, left to right.
/// NUMERIC identifiers compare numerically; alphanumeric ones compare lexically; numeric sorts
/// below alphanumeric; and a shorter run of identifiers sorts below a longer one that matches so
/// far (`beta` < `beta.1`).
///
/// ⚠**The bug this replaces was a plain `String` comparison, and it inverted the update check.**
/// Reported from the field 2026-08-16: a build running **0.2.40-beta.105** offered
/// **0.2.40-beta.78** as an upgrade. Lexically `"beta.78" > "beta.105"` — at the first differing
/// character `'7' > '1'` — so an older release looked newer. The comment here used to claim
/// "lexical is adequate for our `beta.N`/`rc.N` scheme", which was true only while N was a single
/// digit: it has been wrong since beta.10 and silently mis-ranked every release for ninety-five
/// betas. A version comparator is exactly the kind of code where "adequate for now" ages into a
/// falsehood without anything failing loudly.
fn cmp_prerelease(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    let mut ai = a.split('.');
    let mut bi = b.split('.');
    loop {
        match (ai.next(), bi.next()) {
            (None, None) => return Equal,
            // Fewer identifiers sorts lower when everything so far is equal (semver §11).
            (None, Some(_)) => return Less,
            (Some(_), None) => return Greater,
            (Some(x), Some(y)) => {
                if x == y {
                    continue;
                }
                let ord = match (x.parse::<u64>(), y.parse::<u64>()) {
                    (Ok(nx), Ok(ny)) => nx.cmp(&ny), // the case that was broken
                    (Ok(_), Err(_)) => Less,         // numeric < alphanumeric
                    (Err(_), Ok(_)) => Greater,
                    (Err(_), Err(_)) => x.cmp(y),
                };
                if ord != Equal {
                    return ord;
                }
            }
        }
    }
}

/// True if `cand` is a strictly newer semver than `cur`. Prerelease ordering (semver §11): a
/// release outranks its own pre-releases, and two pre-releases of the same core compare via
/// `cmp_prerelease` (so `beta.105 > beta.78`, and `rc.1 > beta.9`).
pub(crate) fn version_gt(cand: &str, cur: &str) -> bool {
    let (cc, cp) = parse_ver(cand);
    let (uc, up) = parse_ver(cur);
    if cc != uc {
        return cc > uc;
    }
    match (cp, up) {
        (None, None) => false,
        (None, Some(_)) => true,
        (Some(_), None) => false,
        (Some(a), Some(b)) => cmp_prerelease(&a, &b) == std::cmp::Ordering::Greater,
    }
}

/// Total order on version strings (no `v` prefix) built from `version_gt` — used to pick the
/// highest-semver release for the Beta track.
fn cmp_ver(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    if version_gt(a, b) {
        Greater
    } else if version_gt(b, a) {
        Less
    } else {
        Equal
    }
}

#[cfg(test)]
#[path = "../tests/unit/update/tests.rs"]
mod tests;
