//! `--lanes LIST` (coloring plan B9): the statistics a lanes render accumulates besides the method's
//! own. Every key the table has is read, the shorthands mean what the help says, and an unknown or
//! empty list is an error naming the keys, never a silent "none".
use super::lanes_stat_set;
use fractadyne_core::coloring::{Stat, StatSet};

#[test]
fn every_statistic_is_read_by_its_key() {
    for s in Stat::ALL {
        assert_eq!(lanes_stat_set(s.spec().key), Ok(StatSet::of(s)), "{}", s.spec().key);
    }
}

#[test]
fn lists_and_shorthands() {
    let stripe_trap = StatSet::of(Stat::Stripe).with(Stat::Trap);
    assert_eq!(lanes_stat_set("stripe,trap"), Ok(stripe_trap));
    assert_eq!(lanes_stat_set("trap+stripe"), Ok(stripe_trap));
    assert_eq!(lanes_stat_set(" stripe , trap "), Ok(stripe_trap));
    assert_eq!(lanes_stat_set("gbuffer"), Ok(StatSet::GBUFFER));
    assert_eq!(lanes_stat_set("all"), Ok(StatSet::GBUFFER.with(Stat::Angle)));
    assert_eq!(lanes_stat_set("gbuffer,angle"), lanes_stat_set("all"));
    assert_eq!(lanes_stat_set("none"), Ok(StatSet::NONE));
    // `gbuffer` is what BLA folds: the angle, which needs no fold, is not in it.
    assert!(!StatSet::GBUFFER.has(Stat::Angle));
}

#[test]
fn an_unknown_or_empty_list_is_an_error_that_names_the_keys() {
    for bad in ["stripes", "stripe,tia", "", " , "] {
        let why = lanes_stat_set(bad).expect_err(bad);
        for s in Stat::ALL {
            assert!(why.contains(s.spec().key), "{bad:?}: {why}");
        }
        assert!(why.contains("gbuffer") && why.contains("all"), "{bad:?}: {why}");
    }
}
