//! ⭐WHERE A HUNG UI THREAD IS STUCK (2026-11-10). The watchdog (`start_watchdog`) can say THAT
//! the window stopped, and which phase it was in, but not WHERE: on Eris (Intel HD Graphics 4400,
//! OpenGL) an export rendering on its own device while the live view drew left the window frozen
//! for good at its 21st frame, inside some driver call, and the log could not say which. Without a
//! debugger on the machine, the app records it itself: when the watchdog first warns, it suspends
//! the UI thread for a moment, walks its stack with the system's own unwind data
//! (`RtlLookupFunctionEntry` / `RtlVirtualUnwind`, x64), resumes it, and logs each frame as
//! `module+offset`. Fractadyne's own frames are named offline from the build's .pdb
//! (`symbolize.py`); a driver's frames say which driver call it is in.
//!
//! ⚠While the UI thread is suspended this thread may hold no lock it could be holding: nothing
//! here allocates, logs or names a module until it is resumed (the heap lock is the usual one).
//! The frames go into a fixed array; a leaf frame's return address is read with
//! `ReadProcessMemory`, which fails instead of faulting on a bad stack pointer.

/// The most frames recorded.
pub(crate) const MAX_FRAMES: usize = 48;

/// Remember the calling thread as the UI thread (`start_watchdog` runs on it).
pub(crate) fn note_ui_thread() {
    imp::note_ui_thread();
}

/// The UI thread's stack as `module+0xoffset` frames, innermost first, or why it could not be
/// read. `None` where this is not supported (not Windows x64) or no UI thread was noted.
pub(crate) fn ui_thread_stack() -> Option<Result<Vec<String>, String>> {
    imp::ui_thread_stack()
}

/// `thread_id`'s stack as [`ui_thread_stack`] gives it (the unit tests walk a thread of their own).
pub(crate) fn thread_stack(thread_id: u32) -> Option<Result<Vec<String>, String>> {
    imp::thread_stack(thread_id)
}

#[cfg(all(windows, target_arch = "x86_64"))]
mod imp {
    use super::MAX_FRAMES;
    use std::ffi::c_void;
    use std::sync::atomic::{AtomicU32, Ordering};

    static UI_THREAD: AtomicU32 = AtomicU32::new(0);

    /// The x64 `CONTEXT` (1232 bytes, 16-aligned): the general registers, `Rip`, and the rest as
    /// bytes the unwinder reads and writes.
    #[repr(C, align(16))]
    struct Context {
        home: [u64; 6],
        context_flags: u32,
        mx_csr: u32,
        segs: [u16; 6],
        eflags: u32,
        dr: [u64; 6],
        /// Rax, Rcx, Rdx, Rbx, Rsp, Rbp, Rsi, Rdi, R8..R15.
        regs: [u64; 16],
        rip: u64,
        rest: [u8; 0x4D0 - 0x100],
    }
    const _: () = assert!(std::mem::size_of::<Context>() == 0x4D0);
    const RSP: usize = 4;

    const CONTEXT_AMD64: u32 = 0x0010_0000;
    const CONTEXT_FULL: u32 = CONTEXT_AMD64 | 0x1 | 0x2 | 0x8;
    const THREAD_SUSPEND_RESUME: u32 = 0x0002;
    const THREAD_GET_CONTEXT: u32 = 0x0008;
    const THREAD_QUERY_INFORMATION: u32 = 0x0040;
    const FROM_ADDRESS: u32 = 0x4;
    const UNCHANGED_REFCOUNT: u32 = 0x2;

    unsafe extern "system" {
        fn GetCurrentThreadId() -> u32;
        fn GetCurrentProcess() -> *mut c_void;
        fn OpenThread(access: u32, inherit: i32, id: u32) -> *mut c_void;
        fn CloseHandle(h: *mut c_void) -> i32;
        fn SuspendThread(h: *mut c_void) -> u32;
        fn ResumeThread(h: *mut c_void) -> u32;
        fn GetThreadContext(h: *mut c_void, ctx: *mut Context) -> i32;
        fn RtlLookupFunctionEntry(pc: u64, image_base: *mut u64, history: *mut c_void) -> *mut c_void;
        fn RtlVirtualUnwind(
            handler_type: u32,
            image_base: u64,
            pc: u64,
            entry: *mut c_void,
            ctx: *mut Context,
            handler_data: *mut *mut c_void,
            establisher: *mut u64,
            pointers: *mut c_void,
        ) -> *mut c_void;
        fn ReadProcessMemory(process: *mut c_void, at: *const c_void, buf: *mut c_void, size: usize, read: *mut usize) -> i32;
        fn GetModuleHandleExW(flags: u32, address: *const u16, module: *mut *mut c_void) -> i32;
        fn GetModuleFileNameW(module: *mut c_void, name: *mut u16, size: u32) -> u32;
    }

    pub(super) fn note_ui_thread() {
        // SAFETY: takes no arguments.
        UI_THREAD.store(unsafe { GetCurrentThreadId() }, Ordering::Relaxed);
    }

    pub(super) fn ui_thread_stack() -> Option<Result<Vec<String>, String>> {
        match UI_THREAD.load(Ordering::Relaxed) {
            0 => None,
            id => thread_stack(id),
        }
    }

    pub(super) fn thread_stack(id: u32) -> Option<Result<Vec<String>, String>> {
        // SAFETY: takes no arguments.
        if id == unsafe { GetCurrentThreadId() } {
            return Some(Err("a thread cannot walk its own stack this way".into()));
        }
        // SAFETY: plain handle request; checked for null.
        let h = unsafe { OpenThread(THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT | THREAD_QUERY_INFORMATION, 0, id) };
        if h.is_null() {
            return Some(Err(format!("could not open thread {id}: {}", std::io::Error::last_os_error())));
        }
        let mut frames = [0u64; MAX_FRAMES];
        // SAFETY: `h` is a thread handle with suspend, resume and context rights; `walk` touches
        // no lock (see the module header) and the thread is resumed whatever it returns.
        let walked = unsafe {
            if SuspendThread(h) == u32::MAX {
                Err(format!("could not suspend thread {id}: {}", std::io::Error::last_os_error()))
            } else {
                let n = walk(h, &mut frames);
                ResumeThread(h);
                n
            }
        };
        // SAFETY: closing the handle opened above.
        unsafe { CloseHandle(h) };
        Some(walked.map(|n| frames[..n].iter().map(|&pc| describe(pc)).collect()))
    }

    /// Walk the suspended thread `h` into `frames`; how many were recorded. Allocates nothing.
    unsafe fn walk(h: *mut c_void, frames: &mut [u64; MAX_FRAMES]) -> Result<usize, String> {
        // SAFETY: a zeroed CONTEXT with its flags set is what GetThreadContext fills.
        let mut ctx: Context = unsafe { std::mem::zeroed() };
        ctx.context_flags = CONTEXT_FULL;
        // SAFETY: `h` is suspended and `ctx` is a properly aligned CONTEXT.
        if unsafe { GetThreadContext(h, &mut ctx) } == 0 {
            return Err("could not read the thread's registers".into());
        }
        let mut n = 0;
        while n < MAX_FRAMES && ctx.rip != 0 {
            frames[n] = ctx.rip;
            n += 1;
            let mut base = 0u64;
            // SAFETY: a lookup in the loaded modules' unwind tables; a null history is allowed.
            let entry = unsafe { RtlLookupFunctionEntry(ctx.rip, &mut base, std::ptr::null_mut()) };
            if entry.is_null() {
                // A leaf function (no unwind data): its return address is at the stack pointer.
                let rsp = ctx.regs[RSP];
                let mut ret = 0u64;
                let mut got = 0usize;
                // SAFETY: ReadProcessMemory on this process fails rather than faulting.
                let ok = unsafe {
                    ReadProcessMemory(GetCurrentProcess(), rsp as *const c_void, (&mut ret as *mut u64).cast(), 8, &mut got)
                };
                if ok == 0 || got != 8 {
                    break;
                }
                ctx.rip = ret;
                ctx.regs[RSP] = rsp + 8;
            } else {
                let mut data = std::ptr::null_mut();
                let mut establisher = 0u64;
                // SAFETY: the entry and base come from the lookup above; `ctx` is the frame's
                // context, which this updates to the caller's (UNW_FLAG_NHANDLER = 0).
                unsafe { RtlVirtualUnwind(0, base, ctx.rip, entry, &mut ctx, &mut data, &mut establisher, std::ptr::null_mut()) };
            }
            if ctx.regs[RSP] == 0 {
                break;
            }
        }
        Ok(n)
    }

    /// `module+0xoffset` for a code address (`?+0x…` outside every module).
    fn describe(pc: u64) -> String {
        let mut module = std::ptr::null_mut();
        // SAFETY: FROM_ADDRESS treats the pointer as an address inside a module; no refcount.
        if unsafe { GetModuleHandleExW(FROM_ADDRESS | UNCHANGED_REFCOUNT, pc as *const u16, &mut module) } == 0 {
            return format!("?+{pc:#x}");
        }
        let mut buf = [0u16; 260];
        // SAFETY: `module` is a loaded module's handle; the buffer's length is passed.
        let len = unsafe { GetModuleFileNameW(module, buf.as_mut_ptr(), buf.len() as u32) } as usize;
        let path = String::from_utf16_lossy(&buf[..len.min(buf.len())]);
        let name = path.rsplit(['\\', '/']).next().unwrap_or(&path).to_string();
        format!("{name}+{:#x}", pc - module as u64)
    }
}

#[cfg(not(all(windows, target_arch = "x86_64")))]
mod imp {
    pub(super) fn note_ui_thread() {}
    pub(super) fn ui_thread_stack() -> Option<Result<Vec<String>, String>> {
        None
    }
    pub(super) fn thread_stack(_thread_id: u32) -> Option<Result<Vec<String>, String>> {
        None
    }
}

#[cfg(test)]
#[path = "../../tests/unit/diag/hang_stack/tests.rs"]
mod tests;
