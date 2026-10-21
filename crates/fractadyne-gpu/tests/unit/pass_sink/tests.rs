use super::*;
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn a_sink_takes_this_threads_reports_only_inside_its_scope() {
    let got = Rc::new(Cell::new(0.0));
    report(5.0); // no sink: dropped
    let g = got.clone();
    let out = with(move |ms| g.set(g.get() + ms), || {
        report(2.5);
        report(f64::NAN); // never a reading
        report(-1.0);
        report(1.5);
        7
    });
    assert_eq!(out, 7);
    assert_eq!(got.get(), 4.0);
    report(3.0); // the scope is over
    assert_eq!(got.get(), 4.0);
    // Another thread's reports are its own.
    with(|_| panic!("another thread reported here"), || std::thread::spawn(|| report(1.0)).join().unwrap());
}

#[test]
fn a_nested_scope_restores_the_outer_sink_and_drops_its_own_on_the_way_out() {
    struct Flush(Rc<Cell<u32>>);
    impl Drop for Flush {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let (outer, inner, flushed) = (Rc::new(Cell::new(0.0)), Rc::new(Cell::new(0.0)), Rc::new(Cell::new(0)));
    let (o, i, f) = (outer.clone(), inner.clone(), Flush(flushed.clone()));
    with(move |ms| o.set(o.get() + ms), || {
        report(1.0);
        with(
            move |ms| {
                let _keep = &f;
                i.set(i.get() + ms);
            },
            || report(2.0),
        );
        assert_eq!(flushed.get(), 1, "the inner sink is dropped when its scope ends");
        report(4.0);
    });
    assert_eq!((outer.get(), inner.get()), (5.0, 2.0));
}
