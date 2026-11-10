//! ⭐**A CRASH THE PANIC HOOK NEVER SEES IS REPORTED TOO.** An access violation, an illegal
//! instruction or another hardware exception (above all inside a graphics driver) is not a Rust
//! panic: the process just ended, and the window vanished without a word. The laptop Eris
//! (2026-11-09: Intel HD Graphics 4400, OpenGL only) died that way 7 s into every start-up, with no
//! report, no dialog and a log that stopped at the adapter line; only the next launch noticed an
//! unclean session, and could not say what killed it.
//!
//! On Windows the process's top-level exception filter now does what the panic hook does: writes a
//! crash report that names the exception, the module it happened in and the crashed thread's native
//! stack (module+offset), notes in the session marker that this session crashed (so the next launch
//! files no second report), flushes the log, and shows the crash dialog to a person who launched the
//! app themselves. Then it lets Windows go on as before (Windows Error Reporting, the same exit code).
//!
//! What runs on the crashed thread is only what must: the facts are copied into static storage with
//! no allocation and no lock of ours (it may have crashed holding one: an allocator, the log). The
//! report and the dialog are a helper thread's, waited for with time limits, so a lock the crashed
//! thread holds can delay the end but never hang it.

/// The exception's name, for the codes a process dies of.
pub(crate) fn exception_name(code: u32) -> Option<&'static str> {
    Some(match code {
        0xC000_0005 => "access violation",
        0xC000_0006 => "in-page error",
        0xC000_001D => "illegal instruction",
        0xC000_0025 => "noncontinuable exception",
        0xC000_008C => "array bounds exceeded",
        0xC000_008D..=0xC000_0093 => "floating-point exception",
        0xC000_0094 => "integer divide by zero",
        0xC000_0095 => "integer overflow",
        0xC000_0096 => "privileged instruction",
        0xC000_00FD => "stack overflow",
        0xC000_0374 => "heap corruption",
        0xC000_0409 => "stack buffer overrun (fail-fast)",
        0x8000_0003 => "breakpoint",
        0xE06D_7363 => "C++ exception",
        _ => return None,
    })
}

/// Whether `module` (a file name) is a graphics driver's or a graphics API's own library.
pub(crate) fn graphics_module(module: &str) -> bool {
    const PREFIXES: &[&str] = &[
        // Intel (ig75icd64 = Haswell's OpenGL, igxelpicd64, igd10iumd64, igc64 = its compiler, igvk64)
        "ig7", "igx", "igd", "igc", "igvk", "igfx", "igil",
        // NVIDIA
        "nvogl", "nvwgf", "nvd3d", "nvldumd", "nvgpucomp", "nvcuda", "nvvk",
        // AMD
        "atio", "atig", "atid", "aticfx", "amdvlk", "amdxc", "amdihk", "amdenc",
        // the APIs themselves
        "opengl32", "vulkan-1", "d3d12", "d3d11", "d3d10", "dxgi", "d3dcompiler", "dxcompiler", "dxil",
    ];
    let m = module.to_ascii_lowercase();
    PREFIXES.iter().any(|p| m.starts_with(p))
}

/// The report's one-line account of the crash: what, at which address, in which module. `params`
/// are the exception's parameters, `module` the file name and offset of the faulting address.
///
/// For a fault in no module (a call through a null or stale function pointer), `caller` is the
/// module that made the call: the file name and offset of the return address the call left on the
/// stack. Venus (2026-11-10, NVIDIA GeForce GT 650M, a 2017 driver) crashed executing address 0x0
/// while listing graphics adapters, and the report could not say whose call that was.
pub(crate) fn describe_called(
    code: u32,
    address: usize,
    params: &[usize],
    module: Option<(&str, usize)>,
    caller: Option<(&str, usize)>,
    thread: &str,
) -> String {
    let what = match exception_name(code) {
        Some(n) => format!("{n} (0x{code:08x})"),
        None => format!("exception 0x{code:08x}"),
    };
    // An access violation's and an in-page error's first two parameters: read/write/execute, and
    // the address it touched.
    let touched = match (code, params) {
        (0xC000_0005 | 0xC000_0006, [kind, at, ..]) => {
            let verb = match kind {
                0 => "reading",
                1 => "writing",
                8 => "executing",
                _ => "touching",
            };
            format!(" {verb} address 0x{at:x}")
        }
        _ => String::new(),
    };
    let place = match module {
        Some((m, off)) if graphics_module(m) => format!(", in the graphics driver: {m}+0x{off:x}"),
        Some((m, off)) => format!(", in {m}+0x{off:x}"),
        None => {
            let by = match caller {
                Some((m, off)) if graphics_module(m) => format!(", called from the graphics driver: {m}+0x{off:x}"),
                Some((m, off)) => format!(", called from {m}+0x{off:x}"),
                None => String::new(),
            };
            format!(", at 0x{address:x} (in no loaded module){by}")
        }
    };
    format!("a native crash: {what}{touched}{place}, on thread '{thread}'")
}

/// The file name of a module path.
pub(crate) fn file_name(path: &str) -> &str {
    path.rsplit(['\\', '/']).next().unwrap_or(path)
}

#[cfg(windows)]
pub(crate) use imp::{install, raise_for_test, reinstall};

#[cfg(not(windows))]
pub(crate) fn install() {}

#[cfg(not(windows))]
pub(crate) fn reinstall(_after: &str) {}

#[cfg(not(windows))]
pub(crate) fn raise_for_test(_on_thread: bool, _null_call: bool) {}

#[cfg(windows)]
mod imp {
    use std::cell::UnsafeCell;
    use std::ffi::c_void;
    use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
    use std::time::Duration;

    /// `EXCEPTION_RECORD` (x64 layout; repr(C) pads `n_params` to `info` as C does).
    #[repr(C)]
    struct ExceptionRecord {
        code: u32,
        flags: u32,
        record: *mut ExceptionRecord,
        address: *mut c_void,
        n_params: u32,
        info: [usize; 15],
    }

    /// `EXCEPTION_POINTERS`.
    #[repr(C)]
    struct ExceptionPointers {
        record: *mut ExceptionRecord,
        context: *mut c_void,
    }

    type Filter = unsafe extern "system" fn(*const ExceptionPointers) -> i32;

    unsafe extern "system" {
        fn SetUnhandledExceptionFilter(filter: Option<Filter>) -> Option<Filter>;
        fn RtlCaptureStackBackTrace(skip: u32, count: u32, frames: *mut *mut c_void, hash: *mut u32) -> u16;
        fn GetModuleHandleExW(flags: u32, address: *const u16, module: *mut *mut c_void) -> i32;
        fn GetModuleFileNameW(module: *mut c_void, name: *mut u16, size: u32) -> u32;
        fn GetCurrentThreadId() -> u32;
        fn RaiseException(code: u32, flags: u32, n_args: u32, args: *const usize);
        fn GetCurrentThreadStackLimits(low: *mut usize, high: *mut usize);
    }

    const EXCEPTION_CONTINUE_SEARCH: i32 = 0;
    const FROM_ADDRESS: u32 = 0x4;
    const UNCHANGED_REFCOUNT: u32 = 0x2;
    const FRAMES: usize = 48;
    const MODULES: usize = 24;
    /// Stack words scanned from the fault's stack pointer (4 KB), and the most kept.
    const STACK_SCAN_WORDS: usize = 512;
    const STACK_HITS: usize = 12;
    const NAME: usize = 260;
    const NO_MODULE: u8 = u8::MAX;

    /// What the crashed thread copies out before anything else runs: static, so a thread out of
    /// stack (a stack overflow) still has room, and nothing here allocates.
    struct Facts {
        code: u32,
        address: usize,
        n_params: usize,
        params: [usize; 15],
        frames: [usize; FRAMES],
        frame_module: [u8; FRAMES],
        n_frames: usize,
        fault_module: u8,
        /// The stack pointer at the fault (x64 `CONTEXT.Rsp`), 0 when unknown.
        rsp: usize,
        /// The first stack words from `rsp` that point into a loaded module: (offset from `rsp`,
        /// value, module index). For a fault in no module, a call through a null pointer, the one at
        /// offset 0 is the call's return address: the caller. The rest are candidates (a pointer to
        /// a module's data reads the same as a return address).
        stack_hits: [(usize, usize, u8); STACK_HITS],
        n_stack_hits: usize,
        /// Each module seen: its base, and its path as UTF-16 (length, buffer).
        modules: [(usize, usize, [u16; NAME]); MODULES],
        n_modules: usize,
    }

    struct Slot(UnsafeCell<Facts>);
    // SAFETY: written by exactly one thread (the one that wins `ENTERED`), then only read, by the
    // helper it starts after the writes.
    unsafe impl Sync for Slot {}

    static SLOT: Slot = Slot(UnsafeCell::new(Facts {
        code: 0,
        address: 0,
        n_params: 0,
        params: [0; 15],
        frames: [0; FRAMES],
        frame_module: [NO_MODULE; FRAMES],
        n_frames: 0,
        fault_module: NO_MODULE,
        rsp: 0,
        stack_hits: [(0, 0, NO_MODULE); STACK_HITS],
        n_stack_hits: 0,
        modules: [(0, 0, [0; NAME]); MODULES],
        n_modules: 0,
    }));

    /// The thread handling a crash (its OS id), 0 before any.
    static HANDLER_THREAD: AtomicU32 = AtomicU32::new(0);
    static ENTERED: AtomicBool = AtomicBool::new(false);
    /// The filter that was in place before ours, if it was not ours: called after ours.
    static PREVIOUS: AtomicUsize = AtomicUsize::new(0);

    /// Install the filter (from `diag::init`, before any window or graphics device exists).
    pub(crate) fn install() {
        // SAFETY: registers a `extern "system"` fn of the right signature; the returned previous
        // filter is kept only as an address, called only through the same signature.
        let prev = unsafe { SetUnhandledExceptionFilter(Some(on_unhandled)) };
        keep_previous(prev);
    }

    /// Install the filter again after `after` (a graphics driver, say) may have put its own in its
    /// place, and say so in the log when one had.
    pub(crate) fn reinstall(after: &str) {
        // SAFETY: as in `install`.
        let prev = unsafe { SetUnhandledExceptionFilter(Some(on_unhandled)) };
        if let Some(p) = prev {
            if p as usize != ours() {
                let module = module_name_of(p as usize).unwrap_or_else(|| "an unknown module".into());
                crate::diag::log_line(
                    "diag",
                    &format!("{after} had replaced the crash handler (with one in {module}); restored, and it is called after ours"),
                );
                keep_previous(prev);
            }
        }
    }

    /// Our filter's address, to tell it from another.
    fn ours() -> usize {
        on_unhandled as Filter as usize
    }

    fn keep_previous(prev: Option<Filter>) {
        if let Some(p) = prev {
            if p as usize != ours() {
                PREVIOUS.store(p as usize, Ordering::SeqCst);
            }
        }
    }

    /// A deliberate crash (`FRACTADYNE_NATIVE_CRASH_AT_START`): an access violation's code raised
    /// on this thread, or on a thread of its own, to see the report and the dialog on a machine
    /// that does not crash. Raised, not a wild write: the same dispatch, and no undefined behavior.
    /// `null_call`: a real call to address 0 instead (x64), the crash Venus had, so the report's
    /// caller can be checked: it must name this function's module, fractadyne.exe.
    pub(crate) fn raise_for_test(on_thread: bool, null_call: bool) {
        let raise = move || {
            #[cfg(target_arch = "x86_64")]
            if null_call {
                // SAFETY: deliberately not: it executes address 0, which faults before anything
                // runs there; the process ends in the crash handler.
                unsafe { std::arch::asm!("call {f}", f = in(reg) 0usize, clobber_abi("system")) };
            }
            let _ = null_call;
            // Writing address 0x10, as a null struct pointer's field would.
            let args = [1usize, 0x10];
            // SAFETY: a documented Win32 call with a two-element argument array that outlives it.
            unsafe { RaiseException(0xC000_0005, 0, 2, args.as_ptr()) };
        };
        if on_thread {
            if let Ok(h) = std::thread::Builder::new().name("fd-crash-test".into()).spawn(raise) {
                let _ = h.join();
            }
        } else {
            raise();
        }
    }

    unsafe extern "system" fn on_unhandled(p: *const ExceptionPointers) -> i32 {
        // SAFETY: plain Win32 calls; `p` is the system's for the duration of this call.
        let me = unsafe { GetCurrentThreadId() };
        if ENTERED.swap(true, Ordering::SeqCst) {
            if HANDLER_THREAD.load(Ordering::SeqCst) == me {
                // Our own handling crashed: let it end.
                return EXCEPTION_CONTINUE_SEARCH;
            }
            // Another thread crashed while the first is being reported: wait for that to finish
            // (the process ends when it does) rather than end it under the dialog.
            for _ in 0..600 {
                std::thread::sleep(Duration::from_secs(1));
            }
            return EXCEPTION_CONTINUE_SEARCH;
        }
        HANDLER_THREAD.store(me, Ordering::SeqCst);
        // SAFETY: only this thread gets here (it won `ENTERED`); the helper reads after these writes.
        let facts = unsafe { &mut *SLOT.0.get() };
        // SAFETY: the system's pointers, null-checked.
        unsafe { capture(facts, p) };
        let thread = std::thread::current().name().unwrap_or("<unnamed>").to_string();
        report_on_helper(thread);
        call_previous(p)
    }

    /// Copy the exception and this thread's stack into `f`, and the modules they are in.
    unsafe fn capture(f: &mut Facts, p: *const ExceptionPointers) {
        // SAFETY: the caller's pointers, each checked for null before use.
        if let Some(rec) = unsafe { p.as_ref().and_then(|p| p.record.as_ref()) } {
            f.code = rec.code;
            f.address = rec.address as usize;
            f.n_params = (rec.n_params as usize).min(15);
            f.params = rec.info;
        }
        let mut raw = [std::ptr::null_mut::<c_void>(); FRAMES];
        // SAFETY: a buffer of FRAMES pointers. The capture runs through the exception dispatch
        // into the frames that faulted (x64 unwinds through KiUserExceptionDispatcher).
        let n = unsafe { RtlCaptureStackBackTrace(0, FRAMES as u32, raw.as_mut_ptr(), std::ptr::null_mut()) } as usize;
        f.n_frames = n.min(FRAMES);
        for i in 0..f.n_frames {
            f.frames[i] = raw[i] as usize;
            f.frame_module[i] = module_index(f, raw[i] as usize);
        }
        let at = f.address;
        f.fault_module = module_index(f, at);
        // SAFETY: the system's pointers, null-checked inside.
        unsafe { scan_stack(f, p) };
    }

    /// Read the fault's stack pointer from its CONTEXT and keep the stack words above it that point
    /// into a loaded module ([`Facts::stack_hits`]). Reads only inside this thread's own stack
    /// (`GetCurrentThreadStackLimits`; the filter runs on the crashed thread), word-aligned, so the
    /// scan cannot fault. x64 only (the CONTEXT layout).
    #[cfg(target_arch = "x86_64")]
    unsafe fn scan_stack(f: &mut Facts, p: *const ExceptionPointers) {
        // SAFETY: the caller's pointer, checked for null; the CONTEXT is the system's, valid for
        // the duration of the filter, and `Rsp` is at 0x98 in the x64 layout.
        let Some(ctx) = (unsafe { p.as_ref() }).map(|p| p.context as *const u8).filter(|c| !c.is_null()) else {
            return;
        };
        // SAFETY: as above.
        let rsp = unsafe { std::ptr::read_unaligned(ctx.add(0x98) as *const u64) } as usize;
        f.rsp = rsp;
        let (mut lo, mut hi) = (0usize, 0usize);
        // SAFETY: two out-parameters.
        unsafe { GetCurrentThreadStackLimits(&mut lo, &mut hi) };
        if rsp < lo || rsp >= hi || rsp % 8 != 0 {
            return;
        }
        let end = hi.min(rsp.saturating_add(STACK_SCAN_WORDS * 8));
        let mut a = rsp;
        while a + 8 <= end && f.n_stack_hits < STACK_HITS {
            // SAFETY: an aligned word inside this thread's committed stack, [rsp, hi).
            let v = unsafe { std::ptr::read_volatile(a as *const usize) };
            let m = module_index(f, v);
            if m != NO_MODULE {
                f.stack_hits[f.n_stack_hits] = (a - rsp, v, m);
                f.n_stack_hits += 1;
            }
            a += 8;
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    unsafe fn scan_stack(_f: &mut Facts, _p: *const ExceptionPointers) {}

    /// The index in `f.modules` of the module holding `addr`, adding it if new; `NO_MODULE` if none
    /// does or the table is full. Fixed buffers only: no allocation on the crashed thread.
    fn module_index(f: &mut Facts, addr: usize) -> u8 {
        let mut h: *mut c_void = std::ptr::null_mut();
        // SAFETY: FROM_ADDRESS reads `addr` as an address, not a string; UNCHANGED_REFCOUNT takes no
        // reference to release.
        if addr == 0 || unsafe { GetModuleHandleExW(FROM_ADDRESS | UNCHANGED_REFCOUNT, addr as *const u16, &mut h) } == 0 {
            return NO_MODULE;
        }
        let base = h as usize;
        if let Some(i) = f.modules[..f.n_modules].iter().position(|m| m.0 == base) {
            return i as u8;
        }
        if f.n_modules == MODULES {
            return NO_MODULE;
        }
        let m = &mut f.modules[f.n_modules];
        // SAFETY: a buffer of NAME u16s.
        let len = unsafe { GetModuleFileNameW(h, m.2.as_mut_ptr(), NAME as u32) } as usize;
        m.0 = base;
        m.1 = len.min(NAME);
        f.n_modules += 1;
        (f.n_modules - 1) as u8
    }

    fn module_name_of(addr: usize) -> Option<String> {
        let mut h: *mut c_void = std::ptr::null_mut();
        // SAFETY: as in `module_index`.
        if unsafe { GetModuleHandleExW(FROM_ADDRESS | UNCHANGED_REFCOUNT, addr as *const u16, &mut h) } == 0 {
            return None;
        }
        let mut buf = [0u16; NAME];
        // SAFETY: a buffer of NAME u16s.
        let len = unsafe { GetModuleFileNameW(h, buf.as_mut_ptr(), NAME as u32) } as usize;
        Some(super::file_name(&String::from_utf16_lossy(&buf[..len.min(NAME)])).to_string())
    }

    fn call_previous(p: *const ExceptionPointers) -> i32 {
        match PREVIOUS.load(Ordering::SeqCst) {
            0 => EXCEPTION_CONTINUE_SEARCH,
            addr => {
                // SAFETY: the address came from SetUnhandledExceptionFilter as a `Filter`.
                let prev: Filter = unsafe { std::mem::transmute::<usize, Filter>(addr) };
                // SAFETY: the system's pointers, passed on as the system handed them to us.
                unsafe { prev(p) }
            }
        }
    }

    /// Write the report, log and dialog on a helper thread, and wait for it: started within 3 s,
    /// the report within 30 s, then (with the dialog up) until the person answers. A lock the
    /// crashed thread holds can stall the helper; the limits make that a quiet end, not a hang.
    fn report_on_helper(thread: String) {
        let (tx, rx) = std::sync::mpsc::channel::<u8>();
        let spawned = std::thread::Builder::new().name("fd-native-crash".into()).spawn(move || {
            let _ = tx.send(0);
            // SAFETY: the crashed thread finished writing `SLOT` before it started this thread.
            let f = unsafe { &*SLOT.0.get() };
            let (reason, loc, stack, module) = render(f, &thread);
            let report = crate::diag::native_crash_report(&reason, &loc, &thread, &stack);
            let _ = tx.send(1);
            let failure = crate::diag::advice::Failure::Native { module };
            crate::diag::show_fatal_dialog(&failure, &crate::diag::crash_reason(&reason), report.as_deref());
            let _ = tx.send(2);
        });
        if spawned.is_err() {
            return;
        }
        if rx.recv_timeout(Duration::from_secs(3)).is_err() {
            return;
        }
        if rx.recv_timeout(Duration::from_secs(30)).is_err() {
            return;
        }
        let _ = rx.recv();
    }

    /// The reason line, the report's `at`, the native stack section and the faulting module's file
    /// name, from the copied facts.
    fn render(f: &Facts, thread: &str) -> (String, String, String, Option<String>) {
        let path = |i: u8| -> Option<String> {
            f.modules.get(i as usize).filter(|_| (i as usize) < f.n_modules).map(|m| String::from_utf16_lossy(&m.2[..m.1]))
        };
        let base = |i: u8| f.modules[i as usize].0;
        let fault = path(f.fault_module).map(|p| (super::file_name(&p).to_string(), f.address.wrapping_sub(base(f.fault_module))));
        // A fault in no module: the word at the stack pointer, when it points into one, is the
        // return address of the call that jumped there.
        let hit = |k: usize| {
            let (off, v, m) = f.stack_hits[k];
            path(m).map(|p| (off, super::file_name(&p).to_string(), v.wrapping_sub(base(m))))
        };
        let caller = match (&fault, f.n_stack_hits) {
            (None, n) if n > 0 => hit(0).filter(|(off, _, _)| *off == 0).map(|(_, m, o)| (m, o)),
            _ => None,
        };
        let reason = super::describe_called(
            f.code,
            f.address,
            &f.params[..f.n_params],
            fault.as_ref().map(|(m, o)| (m.as_str(), *o)),
            caller.as_ref().map(|(m, o)| (m.as_str(), *o)),
            thread,
        );
        let loc = match &fault {
            Some((m, off)) => format!("<native: {m}+0x{off:x}>"),
            None => format!("<native: 0x{:x}>", f.address),
        };
        let mut stack = String::new();
        for i in 0..f.n_frames {
            let line = match path(f.frame_module[i]) {
                Some(p) => format!("  {i:2}: {}+0x{:x}\n", super::file_name(&p), f.frames[i].wrapping_sub(base(f.frame_module[i]))),
                None => format!("  {i:2}: 0x{:x}\n", f.frames[i]),
            };
            stack.push_str(&line);
        }
        if f.n_stack_hits > 0 {
            stack.push_str(&format!(
                "stack words from the fault's rsp 0x{:x} that point into a module (for a call to a null pointer, +0x0 is \
                 its return address; the rest are candidates):\n",
                f.rsp
            ));
            for k in 0..f.n_stack_hits {
                if let Some((off, m, o)) = hit(k) {
                    stack.push_str(&format!("  rsp+0x{off:03x}: {m}+0x{o:x}\n"));
                }
            }
        }
        stack.push_str("modules:\n");
        for m in &f.modules[..f.n_modules] {
            stack.push_str(&format!("  0x{:x} {}\n", m.0, String::from_utf16_lossy(&m.2[..m.1])));
        }
        // The advice reads the module: the caller's, when the fault itself is in none.
        (reason, loc, stack, fault.map(|(m, _)| m).or(caller.map(|(m, _)| m)))
    }
}
