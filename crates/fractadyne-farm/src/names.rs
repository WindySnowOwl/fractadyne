//! The only strings from the network that ever become part of a file name: a job id, a frame
//! prefix, and a machine's display name.
//!
//! ⛔No path crosses the wire as a path (design §3). The controller sends a job id and a prefix;
//! each side builds every path from those, a frame index and its OWN configured folders. That rule
//! is only as strong as these checks: one character outside the set below and a "name" could
//! climb out of its folder (`..`), name a device (`CON`), or address a stream (`a:b`).

/// Longest job id or prefix accepted.
pub const MAX_NAME: usize = 64;

/// A job id or frame prefix: 1–64 of `A-Z a-z 0-9 _ -`, not starting with `-` (an option to every
/// tool that later sees the file name), and not a Windows device name (`CON`, `NUL`, `COM1`, …),
/// which no folder can hold whatever follows it.
pub fn check_file_part(what: &str, s: &str) -> Result<(), String> {
    if s.is_empty() || s.len() > MAX_NAME {
        return Err(format!("{what} must be 1 to {MAX_NAME} characters (got {})", s.len()));
    }
    if let Some(c) = s.chars().find(|c| !(c.is_ascii_alphanumeric() || *c == '_' || *c == '-')) {
        return Err(format!("{what} may use only letters, digits, '_' and '-' (found {c:?})"));
    }
    if s.starts_with('-') {
        return Err(format!("{what} must not start with '-'"));
    }
    let upper = s.to_ascii_uppercase();
    let reserved = ["CON", "PRN", "AUX", "NUL"].contains(&upper.as_str())
        || ((upper.starts_with("COM") || upper.starts_with("LPT"))
            && upper.len() == 4
            && upper.as_bytes()[3].is_ascii_digit());
    if reserved {
        return Err(format!("{what} \"{s}\" is a reserved device name on Windows"));
    }
    Ok(())
}

/// A machine's display name (shown in the controller's table, used to pin its key): printable,
/// 1–64 characters, no control characters. Never used in a path.
pub fn check_display_name(s: &str) -> Result<(), String> {
    let n = s.chars().count();
    if n == 0 || n > MAX_NAME {
        return Err(format!("a machine name must be 1 to {MAX_NAME} characters (got {n})"));
    }
    if s.chars().any(char::is_control) {
        return Err("a machine name must not contain control characters".into());
    }
    Ok(())
}

/// Where a client puts a job's frames in share mode, under ITS OWN mapping of the shared drive:
/// `<root>/fractadyne-farm/<job_id>/<machine>/`. The controller computes the same folder under its
/// own mapping. Nothing here comes off the wire as a path: the job id is checked by
/// [`check_file_part`] and the machine name is made safe with [`crate::manifest::safe_name`].
pub fn share_dir(root: &std::path::Path, job_id: &str, machine: &str) -> Result<std::path::PathBuf, String> {
    check_file_part("job id", job_id)?;
    Ok(root.join("fractadyne-farm").join(job_id).join(crate::manifest::safe_name(machine)))
}

/// The file a frame of a job is written under: `<prefix>_NNNNN.png`, the `--render-tour` naming.
/// Both sides compute it; neither accepts one from the other.
pub fn frame_file_name(prefix: &str, index: u64) -> String {
    format!("{prefix}_{index:05}.png")
}

#[cfg(test)]
mod tests;
