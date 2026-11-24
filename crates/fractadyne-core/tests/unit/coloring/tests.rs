use super::*;

/// The keys are what files store: they never change, whatever the table's order.
#[test]
fn the_keys_are_the_persisted_ones() {
    let keys: Vec<_> = ColorMethod::ALL.iter().map(|m| m.key()).collect();
    assert_eq!(keys, ["smooth", "stripe", "triangle", "trap", "distance", "fringe", "decomposition"]);
    let traps: Vec<_> = TrapType::ALL.iter().map(|t| t.key()).collect();
    assert_eq!(traps, ["point", "cross", "circle"]);
}

/// Each variant's row is its own, and the GPU number is the row's position.
#[test]
fn every_variant_has_its_own_row_and_number() {
    assert_eq!(ColorMethod::ALL.len(), METHODS.len());
    for (i, m) in ColorMethod::ALL.into_iter().enumerate() {
        assert_eq!(m.gpu_id() as usize, i);
        // The row's contents, not its address: `METHODS` is a `const`, so `&METHODS[i]` points at a
        // fresh copy, the same address as `spec()`'s only when the optimizer merges them (it does in
        // release, not in a debug test build).
        assert_eq!((m.spec().key, m.spec().wgsl), (METHODS[i].key, METHODS[i].wgsl));
        assert_eq!(ColorMethod::from_gpu_id(m.gpu_id()), Some(m));
        assert_eq!(ColorMethod::parse_key(m.key()), Some(m));
        assert!(m.spec().wgsl.starts_with("METHOD_"), "{}", m.spec().wgsl);
    }
    assert_eq!(TrapType::ALL.len(), TRAPS.len());
    for (i, t) in TrapType::ALL.into_iter().enumerate() {
        assert_eq!(t.gpu_id() as usize, i);
        assert_eq!(TrapType::from_gpu_id(t.gpu_id()), Some(t));
        assert_eq!(TrapType::parse_key(t.key()), Some(t));
        assert!(t.spec().wgsl.starts_with("TRAP_"), "{}", t.spec().wgsl);
    }
    assert_eq!(ColorMethod::from_gpu_id(ColorMethod::ALL.len() as u32), None);
    assert_eq!(TrapType::from_gpu_id(TrapType::ALL.len() as u32), None);
}

#[test]
fn keys_and_shader_names_are_unique() {
    let mut seen = std::collections::HashSet::new();
    for m in ColorMethod::ALL {
        assert!(seen.insert(m.key()), "key {} twice", m.key());
        assert!(seen.insert(m.spec().wgsl), "shader name {} twice", m.spec().wgsl);
    }
    for t in TrapType::ALL {
        assert!(seen.insert(t.key()), "key {} twice", t.key());
        assert!(seen.insert(t.spec().wgsl), "shader name {} twice", t.spec().wgsl);
    }
    // A statistic's key is for logs only (never stored), so it may share a method's ("trap"); its
    // shader name may not share anything.
    let mut stat_keys = std::collections::HashSet::new();
    for s in Stat::ALL {
        assert!(stat_keys.insert(s.spec().key), "statistic key {} twice", s.spec().key);
        assert!(seen.insert(s.spec().wgsl), "shader name {} twice", s.spec().wgsl);
    }
}

/// Each statistic's row is its own and its bit is its position: one bit each, none shared.
#[test]
fn every_statistic_has_its_own_row_and_bit() {
    assert_eq!(Stat::ALL.len(), STATS.len());
    let mut all = 0u32;
    for (i, s) in Stat::ALL.into_iter().enumerate() {
        assert_eq!((s.spec().key, s.spec().wgsl), (STATS[i].key, STATS[i].wgsl)); // contents: see above
        assert_eq!(s.bit(), 1 << i);
        assert_eq!(all & s.bit(), 0, "{s:?} shares a bit");
        all |= s.bit();
        assert!(s.spec().wgsl.starts_with("STAT_"), "{}", s.spec().wgsl);
    }
}

/// The set's operations, and what it says about the iteration.
#[test]
fn a_set_of_statistics() {
    use Stat::*;
    let st = StatSet::of(Stripe);
    assert!(StatSet::NONE.is_empty() && StatSet::default().is_empty() && !st.is_empty());
    assert!(st.has(Stripe) && !st.has(Trap));
    let both = st.with(Trap);
    assert_eq!(both, StatSet::of(Trap).union(st));
    assert_eq!(both.iter().collect::<Vec<_>>(), [Stripe, Trap]);
    assert_eq!(both.bits(), Stripe.bit() | Trap.bit());
    assert_eq!(both.to_string(), "stripe+trap");
    assert_eq!(StatSet::NONE.to_string(), "none");
    // Every running statistic blocks the series and is folded by BLA; the final angle, read off
    // the final z at escape (B8), does neither and needs neither: it rides both and walks.
    assert!(!StatSet::NONE.blocks_iter_skip() && StatSet::NONE.rides_bla() && !StatSet::NONE.accumulates());
    for s in Stat::ALL {
        let one = StatSet::of(s);
        assert_eq!(one.blocks_iter_skip(), s != Angle, "{s:?}");
        assert!(one.rides_bla(), "{s:?}");
        assert_eq!(one.accumulates(), s != Angle, "{s:?}");
        assert_eq!(s.spec().at_escape, s == Angle, "{s:?}");
        assert_eq!(s.spec().bla_aggregate, s != Angle, "{s:?}: only a running statistic has a node aggregate");
    }
    let every = Stat::ALL.into_iter().fold(StatSet::NONE, StatSet::with);
    assert!(every.blocks_iter_skip() && every.rides_bla() && every.accumulates());
    // The angle beside a running statistic changes none of its rules.
    for s in [Stripe, Triangle, Trap] {
        let with = StatSet::of(s).with(Angle);
        assert_eq!(
            (with.blocks_iter_skip(), with.rides_bla(), with.accumulates()),
            (true, true, true),
            "{s:?} and the angle"
        );
    }
}

/// The full G-buffer (B3) is what BLA folds, and adding it to a frame that accumulates anything
/// changes none of that frame's rules: the series stays off and BLA stays as it was. That is what
/// makes the bundle cost only its operations, and the coloring matrix's `gbuffer` check rests on it
/// (the two renders must then agree channel for channel).
#[test]
fn the_gbuffer_changes_no_rule_of_a_frame_that_accumulates() {
    use Stat::*;
    assert_eq!(StatSet::GBUFFER, StatSet::of(Stripe).with(Triangle).with(Trap));
    for bits in 1u32..(1 << Stat::ALL.len()) {
        let s = Stat::ALL.into_iter().filter(|x| bits & x.bit() != 0).fold(StatSet::NONE, StatSet::with);
        if !s.accumulates() {
            // The angle alone: the G-buffer would add the running statistics' rules (below).
            continue;
        }
        let g = s.union(StatSet::GBUFFER);
        assert_eq!(g.blocks_iter_skip(), s.blocks_iter_skip(), "{s}");
        assert_eq!(g.rides_bla(), s.rides_bla(), "{s}");
        assert_eq!(g.accumulates(), s.accumulates(), "{s}");
    }
    // Every method that reads a statistic reads one in the G-buffer, or the angle.
    for m in ColorMethod::ALL {
        if let Some(r) = m.reads() {
            assert!(StatSet::GBUFFER.has(r) || r == Angle, "{m:?}");
        }
    }
    // For a frame that accumulates nothing as it runs it is not free: it turns the series and the
    // walk off, for the angle alone too (which is why Decomposition does not carry it, B8).
    for s in [StatSet::NONE, StatSet::of(Angle)] {
        assert!(!s.blocks_iter_skip() && !s.accumulates(), "{s}");
        let g = s.union(StatSet::GBUFFER);
        assert!(g.blocks_iter_skip() && g.accumulates(), "{s}");
    }
    assert!(!StatSet::GBUFFER.has(Angle));
}

/// Where a frame carries the G-buffer: in floatexp, for the methods that read one of its
/// statistics; everywhere else, and for every other method, what the method reads. A change here
/// changes what every deep stripe, triangle or trap frame computes, so it should be deliberate.
#[test]
fn a_deep_frame_of_a_gbuffer_method_carries_the_gbuffer() {
    use ColorMethod::*;
    for m in ColorMethod::ALL {
        assert_eq!(m.orbit_stats_at(false), m.orbit_stats(), "{m:?} outside floatexp");
        let deep = m.orbit_stats_at(true);
        match m {
            Stripe | TriangleIneq | OrbitTrap => assert_eq!(deep, StatSet::GBUFFER, "{m:?}"),
            _ => assert_eq!(deep, m.orbit_stats(), "{m:?}"),
        }
        // It changes none of the frame's rules (see the test above).
        assert_eq!(deep.blocks_iter_skip(), m.orbit_stats().blocks_iter_skip(), "{m:?}");
        assert_eq!(deep.rides_bla(), m.orbit_stats().rides_bla(), "{m:?}");
        assert_eq!(deep.accumulates(), m.orbit_stats().accumulates(), "{m:?}");
        assert!(deep.has(m.reads().unwrap_or(Stat::Angle)) || m.reads().is_none(), "{m:?}");
    }
}

/// An unknown key is `None` to the strict reader and the default to the lenient one.
#[test]
fn an_unknown_key_is_none_or_the_default() {
    assert_eq!(ColorMethod::parse_key("Stripe"), None);
    assert_eq!(ColorMethod::parse_key(""), None);
    assert_eq!(ColorMethod::from_key("plasma"), ColorMethod::Smooth);
    assert_eq!(TrapType::parse_key("square"), None);
    assert_eq!(TrapType::from_key("square"), TrapType::Point);
}

/// The predicate sets as they were when each lived in its own `matches!` (main.rs and the GPU
/// crate before A10), and as the method rows said them before B2 moved the iteration's to
/// [`StatSet`]: through a method's own set they are the same sets. A change here changes what a
/// method computes or which accelerators it keeps, so it should be deliberate.
#[test]
fn the_predicate_sets_are_the_established_ones() {
    use ColorMethod::*;
    let set = |f: fn(ColorMethod) -> bool| -> Vec<ColorMethod> { ColorMethod::ALL.into_iter().filter(|m| f(*m)).collect() };
    // needs_aux
    assert_eq!(set(ColorMethod::reads_aux), [Stripe, TriangleIneq, OrbitTrap, Decomposition]);
    assert_eq!(set(|m| !m.orbit_stats().is_empty()), [Stripe, TriangleIneq, OrbitTrap, Decomposition]);
    // blocks_iter_skip: the running statistics (the final angle rides the series since B8)
    assert_eq!(set(|m| m.orbit_stats().blocks_iter_skip()), [Stripe, TriangleIneq, OrbitTrap]);
    // accumulated as the orbit runs (the aux machinery, no walk in passes)
    assert_eq!(set(|m| m.orbit_stats().accumulates()), [Stripe, TriangleIneq, OrbitTrap]);
    assert_eq!(set(ColorMethod::bounded_value), [Stripe, TriangleIneq, Fringe, Decomposition]);
    assert_eq!(set(ColorMethod::needs_distance_estimate), [Distance, Fringe]);
    // rides BLA: every method since B8 (the final angle needs no fold)
    assert_eq!(set(|m| m.orbit_stats().rides_bla()), ColorMethod::ALL);
    // Each reads its own statistic, and accumulates exactly it.
    assert_eq!(Stripe.reads(), Some(Stat::Stripe));
    assert_eq!(TriangleIneq.reads(), Some(Stat::Triangle));
    assert_eq!(OrbitTrap.reads(), Some(Stat::Trap));
    assert_eq!(Decomposition.reads(), Some(Stat::Angle));
    for m in ColorMethod::ALL {
        assert_eq!(m.orbit_stats().iter().collect::<Vec<_>>(), m.reads().into_iter().collect::<Vec<_>>(), "{m:?}");
    }
}

/// A family offers what it computes: Distance only with a distance estimate, the methods that read
/// the orbit's statistics only where the iteration records them (not Newton).
#[test]
fn a_family_offers_what_it_computes() {
    use crate::formula::{caps, BURNING_SHIP, CUSTOM, MANDELBROT, NEWTON, PHOENIX};
    use ColorMethod::*;
    let offered = |f: u32| -> Vec<ColorMethod> { ColorMethod::ALL.into_iter().filter(|m| m.offered_for(caps(f))).collect() };
    assert_eq!(offered(MANDELBROT), ColorMethod::ALL);
    assert_eq!(offered(PHOENIX), ColorMethod::ALL);
    assert_eq!(offered(BURNING_SHIP), [Smooth, Stripe, TriangleIneq, OrbitTrap, Decomposition]);
    assert_eq!(offered(CUSTOM), [Smooth, Stripe, TriangleIneq, OrbitTrap, Decomposition]);
    assert_eq!(offered(NEWTON), [Smooth]);
    // Each with its own reason.
    assert_eq!(Distance.lacks(caps(BURNING_SHIP)), Some(Lacks::DistanceEstimate));
    assert_eq!(Stripe.lacks(caps(NEWTON)), Some(Lacks::OrbitStatistics));
    assert_eq!(Decomposition.lacks(caps(NEWTON)), Some(Lacks::OrbitStatistics));
    assert_eq!(Distance.lacks(caps(NEWTON)), Some(Lacks::DistanceEstimate));
    assert_eq!(Fringe.lacks(caps(BURNING_SHIP)), Some(Lacks::DistanceEstimate));
    assert_eq!(Smooth.lacks(caps(NEWTON)), None);
}

/// The shader gets one constant per method and per trap, named as the table says, with the
/// table's number.
#[test]
fn the_shader_constants_are_the_table() {
    let ids = wgsl_ids();
    for m in ColorMethod::ALL {
        let line = format!("const {}: u32 = {}u;", m.spec().wgsl, m.gpu_id());
        assert_eq!(ids.matches(&line).count(), 1, "{line}");
    }
    for t in TrapType::ALL {
        let line = format!("const {}: u32 = {}u;", t.spec().wgsl, t.gpu_id());
        assert_eq!(ids.matches(&line).count(), 1, "{line}");
    }
    for s in Stat::ALL {
        let line = format!("const {}: u32 = {}u;", s.spec().wgsl, s.bit());
        assert_eq!(ids.matches(&line).count(), 1, "{line}");
    }
    // ...and one per interior shading (coloring plan B5).
    for (i, name) in [(InteriorShading::Method, "INTERIOR_METHOD"), (InteriorShading::Solid, "INTERIOR_SOLID"), (InteriorShading::Trap, "INTERIOR_TRAP")] {
        let line = format!("const {name}: u32 = {}u;", i.gpu_id());
        assert_eq!(ids.matches(&line).count(), 1, "{line}");
    }
    let consts = ids.lines().filter(|l| l.starts_with("const ")).count();
    assert_eq!(consts, ColorMethod::ALL.len() + TrapType::ALL.len() + Stat::ALL.len() + InteriorShading::ALL.len());
    // Every other line is a comment or blank, so the block can go anywhere at module scope.
    for l in ids.lines().filter(|l| !l.starts_with("const ")) {
        assert!(l.is_empty() || l.starts_with("//"), "{l:?}");
    }
}

/// The capability table as it stands (coloring plan B4): the methods colored from the main lane go
/// everywhere; the running statistics keep BLA but lose the series, the walk, glitch correction,
/// the normalized export and strips; decomposition's final angle, read at escape, keeps BLA, the
/// series and the walk (B8) and loses what reads or patches the main lane only. A change here
/// changes what a menu or a dialog offers, so it should be deliberate.
#[test]
fn the_capability_table_is_the_established_one() {
    use Capability::*;
    use ColorMethod::*;
    for m in ColorMethod::ALL {
        let can: Vec<Capability> = Capability::ALL.into_iter().filter(|c| m.can(*c)).collect();
        let want: &[Capability] = match m {
            Smooth | Distance | Fringe => &Capability::ALL,
            Stripe | TriangleIneq | OrbitTrap => &[Bla],
            Decomposition => &[LiveWalk, Series, Bla],
        };
        assert_eq!(can, want, "{m:?}");
        // The deep G-buffer changes none of it: the iteration's answers read the same off the
        // deep frame's statistics.
        let deep = m.orbit_stats_at(true);
        assert_eq!(m.can(Series), !deep.blocks_iter_skip(), "{m:?}");
        assert_eq!(m.can(Bla), deep.rides_bla(), "{m:?}");
        assert_eq!(m.can(LiveWalk), !deep.accumulates(), "{m:?}");
    }
    let keys: Vec<&str> = Capability::ALL.iter().map(|c| c.key()).collect();
    let mut unique = keys.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), keys.len(), "two capabilities share a key");
}

/// The method list's headings (coloring plan B6, design 9.1): the grouping the plan settled, every
/// method under exactly one heading, and no heading without a method (the list would show an empty
/// title).
#[test]
fn every_method_is_under_one_heading_and_no_heading_is_empty() {
    let listed: Vec<(MethodGroup, Vec<&str>)> =
        MethodGroup::ALL.iter().map(|g| (*g, g.methods().map(|m| m.key()).collect())).collect();
    assert_eq!(
        listed,
        [
            (MethodGroup::EscapeCount, vec!["smooth"]),
            (MethodGroup::Distance, vec!["distance", "fringe"]),
            (MethodGroup::OrbitStatistics, vec!["stripe", "triangle", "trap"]),
            (MethodGroup::Angle, vec!["decomposition"]),
        ]
    );
    let total: usize = listed.iter().map(|(_, ms)| ms.len()).sum();
    assert_eq!(total, ColorMethod::ALL.len(), "a method is under no heading or under two");
}

/// What each shape takes from its placement (F4): the cross turns, the circle has a size, the
/// point neither.
#[test]
fn each_trap_shape_says_what_it_uses() {
    let uses: Vec<_> = TrapType::ALL.iter().map(|t| (t.key(), t.spec().turns, t.spec().sized)).collect();
    assert_eq!(uses, [("point", false, false), ("cross", true, false), ("circle", false, true)]);
}

/// At the default placement every shape measures exactly as it always did, bit for bit: the
/// views and goldens from before F4 keep every pixel.
#[test]
fn the_default_placement_is_the_unit_shape_exactly() {
    let place = TrapPlace::default();
    assert!(place.is_default());
    for t in TrapType::ALL {
        for (x, y) in [(0.3, -0.7), (-1.9, 0.01), (1.0e-40, 3.0e-41), (0.0, 0.0), (250.0, -12.5)] {
            assert_eq!(place.dist(t, x, y).to_bits(), t.unit_dist(x, y).to_bits(), "{t:?} at ({x}, {y})");
        }
    }
}

/// The placed shapes, measured in the orbit's plane: the point at its center, the cross as the
/// two lines through its center turned counterclockwise, the circle with its size as the radius.
#[test]
fn a_placed_trap_is_measured_in_the_orbits_plane() {
    let c = [0.3f32, -0.2f32];
    let (cx, cy) = (f64::from(c[0]), f64::from(c[1]));
    let near = |a: f64, b: f64| (a - b).abs() < 1e-6;
    // The point: |z - center|, whatever the turn and size, which it does not use.
    let point = TrapPlace { center: c, rotation: 30.0, size: 3.0 }.for_shape(TrapType::Point);
    assert_eq!((point.rotation, point.size), (0.0, 1.0));
    assert!(near(point.dist(TrapType::Point, cx + 0.3, cy + 0.4), 0.5));
    // The circle: radius = size, centered; a turn changes nothing.
    let circle = TrapPlace { center: c, rotation: 77.0, size: 0.5 }.for_shape(TrapType::Circle);
    assert_eq!(circle.rotation, 0.0);
    for a in [0.0f64, 1.0, 2.5, 4.0] {
        assert!(near(circle.dist(TrapType::Circle, cx + 0.5 * a.cos(), cy + 0.5 * a.sin()), 0.0), "on the rim at {a}");
    }
    assert!(near(circle.dist(TrapType::Circle, cx, cy), 0.5), "the center is a radius away");
    assert!(near(circle.dist(TrapType::Circle, cx + 2.0, cy), 1.5));
    // The cross turned by 30 degrees: its arms run at 30 and 120 degrees through the center, and a
    // point at 75 degrees, between them, is r * sin 45 degrees from either.
    let cross = TrapPlace { center: c, rotation: 30.0, size: 9.0 }.for_shape(TrapType::Cross);
    assert_eq!(cross.size, 1.0);
    let at = |deg: f64, r: f64| {
        let a = deg.to_radians();
        cross.dist(TrapType::Cross, cx + r * a.cos(), cy + r * a.sin())
    };
    for deg in [30.0, 120.0, 210.0, 300.0] {
        assert!(near(at(deg, 0.8), 0.0), "on the arm at {deg} degrees");
    }
    assert!(near(at(75.0, 1.0), std::f64::consts::FRAC_1_SQRT_2));
    // Unturned, the cross through the center is the axes moved there.
    let plain = TrapPlace { center: c, rotation: 0.0, size: 1.0 };
    assert!(near(plain.dist(TrapType::Cross, cx + 0.25, cy - 0.5), 0.25));
}

/// What reaches the iteration is finite and in range: an unreadable number takes its default, the
/// size is clamped, and a negative zero is a zero (keys hash the bits).
#[test]
fn a_placement_for_a_shape_is_finite_and_clamped() {
    let wild = TrapPlace { center: [f32::NAN, -0.0], rotation: f32::INFINITY, size: 0.0 };
    let c = wild.for_shape(TrapType::Circle);
    assert_eq!(c, TrapPlace { center: [0.0, 0.0], rotation: 0.0, size: TrapPlace::SIZE_MIN });
    assert_eq!(c.center[1].to_bits(), 0.0f32.to_bits(), "no negative zero");
    let x = wild.for_shape(TrapType::Cross);
    assert_eq!((x.rotation, x.size), (0.0, 1.0));
    assert_eq!(TrapPlace { size: 1.0e9, ..TrapPlace::DEFAULT }.for_shape(TrapType::Circle).size, TrapPlace::SIZE_MAX);
    // A placement the shape ignores entirely is the default, so it computes as before.
    assert!(TrapPlace { rotation: 45.0, size: 2.0, ..TrapPlace::DEFAULT }.for_shape(TrapType::Point).is_default());
}

/// Placing the trap on a cycle (F4 part 2): its outline through a cycle point, the nearest first,
/// then each next one in orbit order, round the cycle. The period-2 bulb's cycle at the study's
/// view U: z+ near 0, z− at |z−| = 1.0047.
#[test]
fn the_trap_walks_the_cycle_it_is_placed_on() {
    let zp = [-0.02483685875720304, -0.2416742977846682];
    let zm = [-0.975163141242797, 0.2416742977846682];
    let cycle = [zp, zm];
    let f = |z: [f64; 2]| [z[0] as f32, z[1] as f32];
    // The point from the origin: z+ is nearer; then z−; then z+ again.
    let (p1, k1) = TrapPlace::DEFAULT.on_cycle(TrapType::Point, &cycle);
    assert_eq!((k1, p1.center), (0, f(zp)));
    let (p2, k2) = p1.on_cycle(TrapType::Point, &cycle);
    assert_eq!((k2, p2.center), (1, f(zm)));
    let (p3, k3) = p2.on_cycle(TrapType::Point, &cycle);
    assert_eq!((k3, p3.center), (0, f(zp)));
    // The cross is centered the same way and keeps its turn.
    let turned = TrapPlace { rotation: 30.0, ..TrapPlace::DEFAULT };
    let (x, _) = turned.on_cycle(TrapType::Cross, &cycle);
    assert_eq!((x.center, x.rotation), (f(zp), 30.0));
    // The circle keeps its center and passes through a cycle point: the unit circle is nearest z−
    // (|z−| − 1 = 0.0047 against 1 − |z+| = 0.757), then z+.
    let (c1, j1) = TrapPlace::DEFAULT.on_cycle(TrapType::Circle, &cycle);
    assert_eq!((j1, c1.center), (1, [0.0, 0.0]));
    assert!((f64::from(c1.size) - zm[0].hypot(zm[1])).abs() < 1e-6, "{}", c1.size);
    let (c2, j2) = c1.on_cycle(TrapType::Circle, &cycle);
    assert_eq!(j2, 0);
    assert!((f64::from(c2.size) - zp[0].hypot(zp[1])).abs() < 1e-6, "{}", c2.size);
    // Each placed trap passes through the point it took.
    assert!(p1.dist(TrapType::Point, zp[0], zp[1]) < 1e-7);
    assert!(x.dist(TrapType::Cross, zp[0], zp[1]) < 1e-7);
    assert!(c1.dist(TrapType::Circle, zm[0], zm[1]) < 1e-6);
}
