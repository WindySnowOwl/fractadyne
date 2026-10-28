use super::*;

struct Tmp(PathBuf);
impl Tmp {
    fn new(tag: &str) -> Self {
        let d = std::env::temp_dir().join(format!("fractadyne_farm_manifest_{}_{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        Self(d)
    }
}
impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn job(script: &str) -> JobIdentity {
    JobIdentity::new("Gate", "gate.toml", script, "{}", None, "0.3.0-beta.18", "gabc", (320, 180), 3.0, 1, "gate", 19, 1)
}

fn rec(index: u64, bytes: u64) -> DoneRecord {
    DoneRecord { index, bytes, sha256: "0".repeat(64), machine: "A".into(), ms: 10, at_unix: 1, reference: None }
}

#[test]
fn the_job_id_follows_everything_that_decides_the_pictures() {
    let a = job("format_version = 2");
    assert_eq!(a.job_id.len(), 16);
    assert_eq!(a.job_id, job("format_version = 2").job_id, "not deterministic");
    assert_ne!(a.job_id, job("format_version = 2 ").job_id, "the script did not count");
    let other_size = JobIdentity::new("Gate", "gate.toml", "format_version = 2", "{}", None, "0.3.0-beta.18", "gabc", (640, 360), 3.0, 1, "gate", 19, 1);
    assert_ne!(a.job_id, other_size.job_id, "the size did not count");
    let other_build = JobIdentity::new("Gate", "gate.toml", "format_version = 2", "{}", None, "0.3.0-beta.18", "gdef", (320, 180), 3.0, 1, "gate", 19, 1);
    assert_ne!(a.job_id, other_build.job_id, "the commit did not count");
    // The created time and display file name do not change what is rendered.
    let later = JobIdentity::new("Gate", "elsewhere/gate.toml", "format_version = 2", "{}", None, "0.3.0-beta.18", "gabc", (320, 180), 3.0, 1, "gate", 19, 999);
    assert_eq!(a.job_id, later.job_id);
}

/// ⭐The folder alone resumes: recorded frames still on disk at their size are done; a changed or
/// missing one is rendered again; an unrecorded frame is adopted only if it passes the check; a
/// torn last line and `.part` leftovers are survived.
#[test]
fn a_resume_trusts_records_only_for_files_still_there() {
    let t = Tmp::new("resume");
    let j = job("s");
    {
        let (m, r) = Manifest::open(&t.0, j.clone(), &|_| true).expect("create");
        assert_eq!(r, Resume::default());
        for (i, size) in [(0u64, 5u64), (1, 5), (2, 5), (3, 5)] {
            std::fs::write(m.frame_path(i), vec![1u8; size as usize]).unwrap();
            m.record_done(&rec(i, size)).unwrap();
        }
    }
    std::fs::write(t.0.join("gate_00002.png"), b"changed").unwrap(); // size differs from its record
    std::fs::remove_file(t.0.join("gate_00003.png")).unwrap(); // recorded, now missing
    std::fs::write(t.0.join("gate_00005.png"), b"adopt me").unwrap(); // unrecorded, passes
    std::fs::write(t.0.join("gate_00006.png"), b"reject me").unwrap(); // unrecorded, fails the check
    std::fs::write(t.0.join("gate_00007.png.part"), b"half").unwrap();
    std::fs::write(t.0.join("gate_00099.png"), b"past the end").unwrap();
    let mut f = std::fs::OpenOptions::new().append(true).open(t.0.join("farm").join("done.jsonl")).unwrap();
    f.write_all(b"{\"index\":4,\"by").unwrap(); // a torn last line
    drop(f);
    let (_, r) = Manifest::open(&t.0, j, &|p| !p.to_string_lossy().ends_with("00006.png")).expect("resume");
    assert_eq!(r.done, vec![0, 1, 5]);
    assert_eq!(r.adopted, vec![5]);
    assert_eq!(r.lost, vec![2, 3]);
    assert_eq!(r.partials_removed, 1);
    assert_eq!(r.bad_lines, 1);
    assert!(!t.0.join("gate_00007.png.part").exists());
}

/// A zoom video's strips (`design/zoom-video.md` §8.5): the job's id follows the dive (its
/// manifest) and is never a tour's; its units are `strip_NNNNNN.exr`, and a resume reads, adopts
/// and cleans those — not a tour's PNGs, which may share the folder.
#[test]
fn a_strip_job_resumes_its_own_strips() {
    let strips = |dive: &str| JobIdentity::strips("dive", dive, "{}", "0.3.0-beta.22", "gabc", (4614, 509), 12, 1);
    let a = strips("{\"dive\":1}");
    assert_eq!(a.kind, crate::names::UnitKind::Strip);
    assert_eq!(a.job_id, strips("{\"dive\":1}").job_id, "not deterministic");
    assert_ne!(a.job_id, strips("{\"dive\":2}").job_id, "the dive did not count");
    let as_tour = JobIdentity::new("dive", "(zoom video dive)", "{\"dive\":1}", "{}", None, "0.3.0-beta.22", "gabc", (4614, 509), 0.0, 1, "strip", 12, 1);
    assert_ne!(a.job_id, as_tour.job_id, "a strip job took a tour's id");
    let t = Tmp::new("strips");
    {
        let (m, _) = Manifest::open(&t.0, a.clone(), &|_| true).expect("create");
        assert!(m.frame_path(3).ends_with("strip_000003.exr"), "{}", m.frame_path(3).display());
        assert!(m.quarantine_path(3, "PC 1").to_string_lossy().ends_with(".PC_1.exr"));
        for i in [0u64, 1] {
            std::fs::write(m.frame_path(i), vec![1u8; 7]).unwrap();
            m.record_done(&rec(i, 7)).unwrap();
        }
    }
    std::fs::write(t.0.join("strip_000002.exr"), b"adopt me").unwrap(); // a single machine's kept strip
    std::fs::write(t.0.join("strip_000004.exr.part"), b"half").unwrap();
    std::fs::write(t.0.join("strip_00005.png"), b"not a strip").unwrap();
    let (m, r) = Manifest::open(&t.0, a, &|p| p.extension().is_some_and(|e| e == "exr")).expect("resume");
    assert_eq!(r.done, vec![0, 1, 2]);
    assert_eq!(r.adopted, vec![2]);
    assert_eq!(r.partials_removed, 1);
    assert!(!t.0.join("strip_000004.exr.part").exists());
    // `job.toml` says what the folder holds: a later run reads it back as strips.
    let text = std::fs::read_to_string(m.farm_dir().join("job.toml")).unwrap();
    assert!(text.contains("kind = \"strip\""), "{text}");
}

/// A `job.toml` written before strips existed has no `kind`: it is a tour's.
#[test]
fn a_job_file_from_before_strips_is_a_tours() {
    let mut text = toml::to_string(&job("s")).unwrap();
    text = text.lines().filter(|l| !l.starts_with("kind")).collect::<Vec<_>>().join("\n");
    let j: JobIdentity = toml::from_str(&text).expect("an old job.toml reads");
    assert_eq!(j.kind, crate::names::UnitKind::Frame);
}

#[test]
fn a_folder_holding_another_job_is_refused_naming_both() {
    let t = Tmp::new("other");
    Manifest::open(&t.0, job("first"), &|_| true).expect("create");
    let e = Manifest::open(&t.0, job("second"), &|_| true).err().expect("must refuse");
    assert!(e.contains("different farm job") && e.contains(&job("first").job_id) && e.contains(&job("second").job_id), "{e}");
}

#[test]
fn quarantine_and_diag_names_are_safe_whatever_the_machine_is_called() {
    let t = Tmp::new("names");
    let (m, _) = Manifest::open(&t.0, job("s"), &|_| true).unwrap();
    let q = m.quarantine_path(12, "PLUTO (RX 6800 XT)/../x");
    assert_eq!(q.file_name().unwrap().to_string_lossy(), "gate_00012.PLUTO__RX_6800_XT_____x.png");
    assert_eq!(q.parent().unwrap(), m.farm_dir().join("bad"));
    let d = m.diag_dir("..", 5);
    assert_eq!(d.parent().unwrap(), m.farm_dir().join("diag"));
    assert_eq!(d.file_name().unwrap().to_string_lossy(), "__-5");
}

#[test]
fn status_events_and_metrics_are_written() {
    let t = Tmp::new("logs");
    let (m, _) = Manifest::open(&t.0, job("s"), &|_| true).unwrap();
    m.record_event(1, "A joined").unwrap();
    m.record_event(2, "A left").unwrap();
    m.record_metrics(&serde_json::json!({"fps": 1.5})).unwrap();
    m.write_status("done = 3\n").unwrap();
    let ev = std::fs::read_to_string(m.farm_dir().join("events.jsonl")).unwrap();
    assert_eq!(ev.lines().count(), 2);
    assert!(ev.contains("A joined"));
    assert_eq!(std::fs::read_to_string(m.farm_dir().join("status.toml")).unwrap(), "done = 3\n");
    assert!(!m.farm_dir().join("status.toml.part").exists());
}
