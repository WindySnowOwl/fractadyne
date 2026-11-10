use super::*;

#[cfg(all(windows, target_arch = "x86_64"))]
mod windows {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    unsafe extern "system" {
        fn GetCurrentThreadId() -> u32;
    }

    /// A thread parked in a sleep, its id sent back; it ends when `stop` is dropped.
    fn parked() -> (u32, mpsc::Sender<()>, std::thread::JoinHandle<()>) {
        let (id_tx, id_rx) = mpsc::channel();
        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        let h = std::thread::spawn(move || {
            // SAFETY: takes no arguments.
            id_tx.send(unsafe { GetCurrentThreadId() }).unwrap();
            let _ = stop_rx.recv_timeout(Duration::from_secs(30));
        });
        let id = id_rx.recv().unwrap();
        std::thread::sleep(Duration::from_millis(100));
        (id, stop_tx, h)
    }

    #[test]
    fn a_parked_threads_stack_reaches_the_system_and_this_program() {
        let (id, stop, h) = parked();
        let frames = thread_stack(id).expect("supported here").expect("readable");
        drop(stop);
        h.join().unwrap();
        assert!(frames.len() >= 3, "{frames:?}");
        assert!(frames.len() <= MAX_FRAMES);
        // A wait ends in the kernel's own module, and the thread began in this program.
        assert!(frames.iter().any(|f| f.to_ascii_lowercase().starts_with("ntdll.dll+0x")), "{frames:?}");
        assert!(frames.iter().any(|f| f.contains(".exe+0x")), "{frames:?}");
        assert!(frames.iter().all(|f| f.contains("+0x")), "{frames:?}");
    }

    #[test]
    fn the_thread_resumes_after_the_walk() {
        let (id, stop, h) = parked();
        for _ in 0..5 {
            assert!(thread_stack(id).expect("supported here").is_ok());
        }
        drop(stop);
        // It ran on to the end: a thread left suspended would never be joined.
        let (done_tx, done_rx) = mpsc::channel();
        std::thread::spawn(move || {
            h.join().unwrap();
            done_tx.send(()).unwrap();
        });
        assert!(done_rx.recv_timeout(Duration::from_secs(10)).is_ok());
    }

    #[test]
    fn a_thread_cannot_walk_itself_and_a_missing_one_says_so() {
        // SAFETY: takes no arguments.
        let me = unsafe { GetCurrentThreadId() };
        assert!(thread_stack(me).expect("supported here").is_err());
        assert!(thread_stack(u32::MAX - 3).expect("supported here").is_err());
    }
}

#[test]
fn no_ui_thread_noted_means_no_stack() {
    // The tests run without a window, so no UI thread is noted (start_watchdog never ran).
    assert!(ui_thread_stack().is_none());
}
