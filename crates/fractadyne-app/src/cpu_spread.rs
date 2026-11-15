//! The platform half of `fractadyne_core::cpu_spread`: which logical CPUs are one per physical
//! core, and holding a thread to them.
//!
//! A split reference walk's threads meet at a barrier every step and spin between stages; two of
//! them on one core's two logical CPUs set every step's pace (measured on a Ryzen 9 3950X: 37 s
//! against 43 s for the 1e30000 path's reference; see the core module). The set is the lowest
//! logical CPU of every core, within the process's own affinity. Declined, leaving the scheduler
//! as it was: without SMT (nothing to gain), on a hybrid CPU (a barrier thread held to an
//! efficiency core would set the pace instead), and with more than one processor group (a
//! thread's mask covers one group). Windows only; elsewhere a no-op.

/// The spread set from a `GetLogicalProcessorInformationEx(RelationProcessorCore)` buffer: the
/// lowest logical CPU of each core, or why there is none. Pure, so the parsing is tested anywhere.
/// Layout per entry (x64): `Relationship: u32`, `Size: u32`, then `PROCESSOR_RELATIONSHIP`:
/// `Flags: u8` (1 = SMT) at +8, `EfficiencyClass: u8` at +9, `GroupCount: u16` at +30, and the
/// first `GROUP_AFFINITY` (`Mask: usize`, `Group: u16`) at +32 and +40.
pub(crate) fn spread_set(buf: &[u8]) -> Result<u64, &'static str> {
    let u32_at = |o: usize| buf.get(o..o + 4).map(|b| u32::from_le_bytes(b.try_into().expect("4 bytes")));
    let (mut off, mut set, mut smt) = (0usize, 0u64, false);
    let mut class: Option<u8> = None;
    while off < buf.len() {
        let (Some(rel), Some(size)) = (u32_at(off), u32_at(off + 4)) else { return Err("a truncated entry") };
        let size = size as usize;
        if size < 8 || off + size > buf.len() {
            return Err("an entry of an impossible size");
        }
        if rel == 0 {
            if size < 48 {
                return Err("a core entry without a group mask");
            }
            let (flags, eff) = (buf[off + 8], buf[off + 9]);
            let groups = u16::from_le_bytes([buf[off + 30], buf[off + 31]]);
            let mask = u64::from_le_bytes(buf[off + 32..off + 40].try_into().expect("8 bytes"));
            let group = u16::from_le_bytes([buf[off + 40], buf[off + 41]]);
            if groups != 1 || group != 0 {
                return Err("more than one processor group");
            }
            if mask == 0 {
                return Err("a core with no logical CPU");
            }
            match class {
                Some(c) if c != eff => return Err("a hybrid CPU"),
                _ => class = Some(eff),
            }
            smt |= flags & 1 == 1 && mask.count_ones() > 1;
            set |= mask & mask.wrapping_neg(); // the core's lowest logical CPU
        }
        off += size;
    }
    if set == 0 {
        return Err("no cores listed");
    }
    if !smt {
        return Err("no core has two logical CPUs");
    }
    Ok(set)
}

#[cfg(windows)]
mod win {
    use std::ffi::c_void;
    unsafe extern "system" {
        fn GetLogicalProcessorInformationEx(relationship: i32, buffer: *mut u8, length: *mut u32) -> i32;
        fn GetActiveProcessorGroupCount() -> u16;
        // As `sysinfo.rs` and `diag/hang_stack.rs` declare it (one signature crate-wide).
        fn GetCurrentProcess() -> isize;
        fn GetProcessAffinityMask(process: isize, process_mask: *mut usize, system_mask: *mut usize) -> i32;
        fn GetCurrentThread() -> *mut c_void;
        fn SetThreadAffinityMask(thread: *mut c_void, mask: usize) -> usize;
    }

    /// The spread mask for this process, worked out once (and said once in the log).
    pub(super) fn mask() -> Option<usize> {
        static MASK: std::sync::OnceLock<Option<usize>> = std::sync::OnceLock::new();
        *MASK.get_or_init(|| {
            let found = find();
            crate::diag::log_line(
                "ref",
                &match found {
                    Ok(m) => format!("split orbit walks: one logical CPU per core, mask {m:#x}"),
                    Err(why) => format!("split orbit walks: left to the scheduler ({why})"),
                },
            );
            found.ok()
        })
    }

    fn find() -> Result<usize, &'static str> {
        // SAFETY: plain queries. The buffer is sized by the first call, which writes only `length`
        // (it fails with ERROR_INSUFFICIENT_BUFFER); the second fills at most `length` bytes.
        unsafe {
            if GetActiveProcessorGroupCount() != 1 {
                return Err("more than one processor group");
            }
            let mut len = 0u32;
            GetLogicalProcessorInformationEx(0, std::ptr::null_mut(), &mut len);
            if len == 0 || len > 1 << 20 {
                return Err("no processor information");
            }
            let mut buf = vec![0u8; len as usize];
            if GetLogicalProcessorInformationEx(0, buf.as_mut_ptr(), &mut len) == 0 {
                return Err("no processor information");
            }
            buf.truncate(len as usize);
            let set = super::spread_set(&buf)? as usize;
            let (mut process, mut system) = (0usize, 0usize);
            if GetProcessAffinityMask(GetCurrentProcess(), &mut process, &mut system) == 0 {
                return Err("no process affinity");
            }
            let m = set & process;
            if m == 0 || m == process {
                return Err("the process's CPUs already have one per core");
            }
            Ok(m)
        }
    }

    pub(super) fn set(mask: usize) -> usize {
        // SAFETY: the pseudo-handle of the calling thread needs no closing; the mask is a subset of
        // the process's own affinity, so it is one the thread may take.
        unsafe { SetThreadAffinityMask(GetCurrentThread(), mask) }
    }
}

/// `fractadyne_core::cpu_spread`'s `spread` hook: hold the calling thread to the spread set,
/// returning the mask to restore.
pub(crate) fn spread_this_thread() -> Option<usize> {
    #[cfg(windows)]
    {
        let m = win::mask()?;
        let prev = win::set(m);
        (prev != 0).then_some(prev)
    }
    #[cfg(not(windows))]
    {
        None
    }
}

/// The `restore` hook: give the calling thread back its mask.
pub(crate) fn restore_this_thread(prev: usize) {
    #[cfg(windows)]
    {
        win::set(prev);
    }
    #[cfg(not(windows))]
    {
        let _ = prev;
    }
}

#[cfg(test)]
#[path = "../tests/unit/cpu_spread/tests.rs"]
mod tests;
