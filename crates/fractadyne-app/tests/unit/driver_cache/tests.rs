use super::*;
use fractadyne_gpu::pipeline_compile::SlowRepeat;

fn gather() -> SlowRepeat {
    SlowRepeat { pipeline: "gather.pipeline fs_iterate_gather".into(), ms: 7_520, count: 6 }
}

/// The notice names what compiled again and how long it took, and tells the user what to clear:
/// on NVIDIA the cache folder with its size and the setting that keeps it from filling again.
#[test]
fn the_notice_says_what_compiled_again_and_what_to_clear() {
    let dir = PathBuf::from(r"C:\Users\robin\AppData\Local\NVIDIA\GLCache");
    let (title, body) = notice(&gather(), NVIDIA, Some((dir.as_path(), 1_074_000_000)));
    assert_eq!(title, "Graphics driver not keeping shaders");
    assert!(body.contains("compiled 6 of Fractadyne's shaders again"), "{body}");
    assert!(body.contains("gather.pipeline fs_iterate_gather, 7.5 s"), "{body}");
    assert!(body.contains(r"C:\Users\robin\AppData\Local\NVIDIA\GLCache   (1024 MB)"), "{body}");
    assert!(body.contains("Shader Cache Size"), "{body}");

    let (_, amd) = notice(&gather(), AMD, None);
    assert!(amd.contains("Reset Shader Cache"), "{amd}");
    let (_, other) = notice(&gather(), 0x8086, None);
    assert!(other.contains("clear the graphics driver's shader cache"), "{other}");
    let (_, nv_no_dir) = notice(&gather(), NVIDIA, None);
    assert!(nv_no_dir.contains("Shader Cache") && !nv_no_dir.contains("MB)"), "{nv_no_dir}");
}

/// The folder's size is the files under it, however deep.
#[test]
fn a_cache_folder_size_counts_every_file_under_it() {
    let root = std::env::temp_dir().join(format!("fd-driver-cache-{}", std::process::id()));
    let sub = root.join("2e40a44cdf65ba3d");
    std::fs::create_dir_all(&sub).unwrap();
    std::fs::write(root.join("a.toc"), [0u8; 10]).unwrap();
    std::fs::write(sub.join("b.bin"), [0u8; 1000]).unwrap();
    assert_eq!(dir_size(&root), 1010);
    std::fs::remove_dir_all(&root).unwrap();
}
