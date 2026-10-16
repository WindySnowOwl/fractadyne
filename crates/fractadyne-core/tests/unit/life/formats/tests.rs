use super::*;

const GLIDER: [(i64, i64, u8); 5] = [(1, 0, 1), (2, 1, 1), (0, 2, 1), (1, 2, 1), (2, 2, 1)];

#[test]
fn rle_reads_header_comments_and_runs() {
    let text = "#N Glider\n#C A comment\n#O Someone\nx = 3, y = 3, rule = B3/S23\nbo$2bo$3o!\n";
    let p = parse_rle(text).unwrap();
    assert_eq!(p.cells, GLIDER);
    assert_eq!(p.rule.as_deref(), Some("B3/S23"));
    assert_eq!(p.name.as_deref(), Some("Glider"));
    assert_eq!(p.comments, ["A comment", "Someone"]);
    // The body may span lines, carry spaces, and stop at '!' whatever follows.
    let split = "x = 3, y = 3\nbo$2b\no$3o\n!  trailing words\n#C ignored";
    assert_eq!(parse_rle(split).unwrap().cells, GLIDER);
    // Empty rows by count, and trailing dead cells omitted or written.
    let rows = parse_rle("x = 2, y = 4\n2o3$2o!").unwrap();
    assert_eq!(rows.cells, [(0, 0, 1), (1, 0, 1), (0, 3, 1), (1, 3, 1)]);
    assert_eq!(parse_rle("o2b$o!").unwrap().cells, [(0, 0, 1), (0, 1, 1)]);
}

#[test]
fn rle_reads_positions_generations_and_states() {
    let p = parse_rle("#CXRLE Pos=-5,7 Gen=1234\nx = 1, y = 1, rule = B2/S/C3\nA$.B!").unwrap();
    assert_eq!(p.cells, [(-5, 7, 1), (-4, 8, 2)]);
    assert_eq!(p.generation, Some(1234));
    let q = parse_rle("#P 10 -3\no!").unwrap();
    assert_eq!(q.cells, [(10, -3, 1)]);
    // States past 24 take a prefix: pA = 25, pX = 48, qA = 49, yO = 255.
    let r = parse_rle("XpApXqAyO!").unwrap();
    assert_eq!(r.cells.iter().map(|c| c.2).collect::<Vec<_>>(), [24, 25, 48, 49, 255]);
}

#[test]
fn rle_refuses_what_it_cannot_read() {
    for (text, says) in [
        ("bo$2bz!", "not an RLE run"),
        ("99999999999999999999o!", "too large"),
        ("20000000o!", "live cells"),
        ("pZ!", "followed by A-X"),
        ("yX!", "past 255"),
        ("x = a, y = 3\no!", "not a size"),
        ("#CXRLE Pos=1\no!", "not a position"),
        ("3", "ends inside a run"),
        ("4611686018427387903b2o!", "extent"),
    ] {
        let e = parse_rle(text).expect_err(text);
        assert!(e.message.contains(says), "'{text}': {e}");
    }
}

/// What the writer writes, the reader reads back — positions, states, the generation; and no line
/// passes 70 columns.
#[test]
fn rle_round_trips() {
    let mut seed = 99u64;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    for states in [2u16, 3, 30, 256] {
        let mut cells: Vec<(i64, i64, u8)> = Vec::new();
        for j in -20..40i64 {
            for i in -50..90i64 {
                if next() % 3 == 0 {
                    cells.push((i, j, 1 + (next() % u64::from(states - 1)) as u8));
                }
            }
        }
        // A long run, to exercise wrapping.
        cells.extend((300..500).map(|x| (x, 100, 1)));
        cells.sort_unstable_by_key(|&(x, y, _)| (y, x));
        let text = write_rle(&cells, "B3/S23", states > 2, Some("soup"), Some(77));
        assert!(text.lines().all(|l| l.len() <= 70), "a line passes 70 columns");
        let p = parse_rle(&text).unwrap();
        assert_eq!(p.cells, cells, "{states} states");
        assert_eq!((p.generation, p.name.as_deref(), p.rule.as_deref()), (Some(77), Some("soup"), Some("B3/S23")));
    }
    assert_eq!(parse_rle(&write_rle(&[], "B3/S23", false, None, None)).unwrap().cells, []);
}

#[test]
fn plaintext_and_life_formats_read_and_write() {
    let cells_text = "!Name: Glider\n!A comment\n.O\n..O\nOOO\n";
    let p = parse_plaintext(cells_text).unwrap();
    assert_eq!(p.cells, GLIDER);
    assert_eq!(p.name.as_deref(), Some("Glider"));
    assert_eq!(parse_plaintext(&write_plaintext(&GLIDER, Some("g"))).unwrap().cells, GLIDER);
    assert!(parse_plaintext("..X\n").is_err());

    let l105 = "#Life 1.05\n#D A glider\n#N\n#P -1 -1\n.*\n..*\n***\n";
    let q = parse_life105(l105).unwrap();
    assert_eq!(q.cells, GLIDER.map(|(x, y, s)| (x - 1, y - 1, s)));
    assert_eq!(q.rule.as_deref(), Some("B3/S23"));

    let l106 = "#Life 1.06\n0 -1\n1 0\n-1 1\n0 1\n1 1\n";
    assert_eq!(parse_life106(l106).unwrap().cells, GLIDER.map(|(x, y, s)| (x - 1, y - 1, s)));
    assert_eq!(parse_life106(&write_life106(&GLIDER)).unwrap().cells, GLIDER);
}

#[test]
fn the_format_is_recognised() {
    for (text, want) in [
        ("x = 3, y = 3\nbo$2bo$3o!", PatternFormat::Rle),
        ("bo$2bo$3o!", PatternFormat::Rle),
        ("#N name\n#C c\n3o!", PatternFormat::Rle),
        ("!Name: g\n.O\n..O\nOOO", PatternFormat::Plaintext),
        ("OOO", PatternFormat::Plaintext),
        ("#Life 1.05\n*", PatternFormat::Life105),
        ("#Life 1.06\n0 0", PatternFormat::Life106),
    ] {
        let (_, f) = parse_pattern(text).unwrap_or_else(|e| panic!("{text}: {e}"));
        assert_eq!(f, want, "{text}");
    }
}
