use super::spread_set;

/// One `RelationProcessorCore` entry as `GetLogicalProcessorInformationEx` lays it out on x64:
/// 48 bytes, flags (1 = SMT), efficiency class, one group, the core's logical-CPU mask, group 0.
fn core(mask: u64, smt: bool, class: u8, group: u16) -> Vec<u8> {
    let mut e = vec![0u8; 48];
    e[0..4].copy_from_slice(&0u32.to_le_bytes()); // RelationProcessorCore
    e[4..8].copy_from_slice(&48u32.to_le_bytes());
    e[8] = u8::from(smt);
    e[9] = class;
    e[30..32].copy_from_slice(&1u16.to_le_bytes());
    e[32..40].copy_from_slice(&mask.to_le_bytes());
    e[40..42].copy_from_slice(&group.to_le_bytes());
    e
}

/// The 3950X's layout (siblings 2k and 2k+1) gives the measured mask, 0x55555555: the lowest
/// logical CPU of each of its 16 cores.
#[test]
fn an_smt_cpu_spreads_to_the_lowest_logical_cpu_of_each_core() {
    let buf: Vec<u8> = (0..16).flat_map(|k| core(0b11 << (2 * k), true, 0, 0)).collect();
    assert_eq!(spread_set(&buf), Ok(0x5555_5555));
    // Siblings numbered apart (k and k+4, as some layouts number them): still one per core.
    let buf: Vec<u8> = (0..4).flat_map(|k| core((1 << k) | (1 << (k + 4)), true, 0, 0)).collect();
    assert_eq!(spread_set(&buf), Ok(0b1111));
}

/// The cases left to the scheduler, each with its reason.
#[test]
fn the_declined_cases_say_why() {
    let no_smt: Vec<u8> = (0..8).flat_map(|k| core(1 << k, false, 0, 0)).collect();
    assert_eq!(spread_set(&no_smt), Err("no core has two logical CPUs"));
    let hybrid: Vec<u8> = (0..4).flat_map(|k| core(0b11 << (2 * k), true, 1, 0)).chain(core(1 << 8, false, 0, 0)).collect();
    assert_eq!(spread_set(&hybrid), Err("a hybrid CPU"));
    assert_eq!(spread_set(&core(0b11, true, 0, 1)), Err("more than one processor group"));
    assert_eq!(spread_set(&[]), Err("no cores listed"));
    let mut cut = core(0b11, true, 0, 0);
    cut.truncate(40);
    assert_eq!(spread_set(&cut), Err("an entry of an impossible size"));
    // Entries of other kinds (a cache's, here 32 bytes) are skipped over by their size.
    let mut other = vec![0u8; 32];
    other[0..4].copy_from_slice(&2u32.to_le_bytes());
    other[4..8].copy_from_slice(&32u32.to_le_bytes());
    let mixed: Vec<u8> = other.iter().copied().chain(core(0b11, true, 0, 0)).chain(core(0b1100, true, 0, 0)).collect();
    assert_eq!(spread_set(&mixed), Ok(0b0101));
}
