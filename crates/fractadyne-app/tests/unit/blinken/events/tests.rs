use super::*;

fn ev(kind: Kind, a: u32) -> Event {
    Event { seq: 0, frame: 7, t_us: 0, view: 0, kind, a, b: 0, f: 0.0 }
}

#[test]
fn a_ring_hands_back_its_events_oldest_first_with_their_sequence() {
    let mut r = Ring::new(8);
    for a in 0..5 {
        r.push(ev(Kind::PinPass, a));
    }
    let mut out = Vec::new();
    let next = r.since(0, &mut out);
    assert_eq!(next, 5);
    assert_eq!(out.iter().map(|e| e.a).collect::<Vec<_>>(), vec![0, 1, 2, 3, 4]);
    assert_eq!(out.iter().map(|e| e.seq).collect::<Vec<_>>(), vec![0, 1, 2, 3, 4]);
}

#[test]
fn a_reader_gets_only_what_came_after_the_last_it_saw() {
    let mut r = Ring::new(8);
    for a in 0..3 {
        r.push(ev(Kind::PinPass, a));
    }
    let mut out = Vec::new();
    let next = r.since(0, &mut out);
    out.clear();
    r.push(ev(Kind::PinAdopt, 9));
    let next2 = r.since(next, &mut out);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].kind, Kind::PinAdopt);
    assert_eq!(next2, 4);
    // Nothing new: nothing read, the cursor stays.
    out.clear();
    assert_eq!(r.since(next2, &mut out), 4);
    assert!(out.is_empty());
}

#[test]
fn a_full_ring_overwrites_its_oldest_and_a_late_reader_gets_what_is_left() {
    let mut r = Ring::new(4);
    for a in 0..10 {
        r.push(ev(Kind::TileSent, a));
    }
    let mut out = Vec::new();
    let next = r.since(0, &mut out);
    assert_eq!(next, 10);
    assert_eq!(out.iter().map(|e| e.a).collect::<Vec<_>>(), vec![6, 7, 8, 9]);
    assert_eq!(out.iter().map(|e| e.seq).collect::<Vec<_>>(), vec![6, 7, 8, 9]);
    assert_eq!(r.pushed(), 10);
}

#[test]
fn a_reader_partway_through_a_wrapped_ring_resumes_where_it_stopped() {
    let mut r = Ring::new(4);
    for a in 0..6 {
        r.push(ev(Kind::TileSent, a));
    }
    let mut out = Vec::new();
    // Saw up to seq 4 (exclusive): gets 4 and 5, both still held.
    r.since(4, &mut out);
    assert_eq!(out.iter().map(|e| e.a).collect::<Vec<_>>(), vec![4, 5]);
}

#[test]
fn the_shared_ring_stamps_the_frame_and_keeps_the_values() {
    // The process-wide ring is shared with every other test: read only what this test pushed.
    let from = pushed();
    set_frame(1234);
    emit(1, Kind::SplitPass, 2, 3, 0.5);
    emit(5, Kind::GridDone, 0, 0, 0.0);
    let mut out = Vec::new();
    read_since(from, &mut out);
    let mine: Vec<&Event> = out.iter().filter(|e| e.frame == 1234).collect();
    assert!(mine.iter().any(|e| e.kind == Kind::SplitPass && e.view == 1 && e.a == 2 && e.b == 3 && e.f == 0.5));
    // A view past the second is clamped to it.
    assert!(mine.iter().any(|e| e.kind == Kind::GridDone && e.view == 1));
}
