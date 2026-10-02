use super::*;

fn err(text: &str) -> ParseError {
    LSystem::parse(text).expect_err("should be refused")
}

#[test]
fn the_native_format_reads_every_key() {
    let s = LSystem::parse(
        "# a comment\nname Test\nangle /6\nheading 90\ndraw G\nmove F\nvariables D\ncolour depth\norder 5\naxiom F[+G]\nG = GG\n",
    )
    .unwrap();
    assert_eq!(s.name, "Test");
    assert_eq!(s.angle, Angle::Division(6));
    assert_eq!(s.heading, 90.0);
    assert_eq!(s.roles[b'G' as usize], Role::Draw);
    assert_eq!(s.roles[b'F' as usize], Role::Move);
    assert_eq!(s.roles[b'D' as usize], Role::None);
    assert_eq!(s.roles[b'M' as usize], Role::Move, "an untouched default stays");
    assert_eq!(s.colour, Some(Colouring::Depth));
    assert_eq!(s.order, Some(5));
    assert_eq!(s.axiom, vec![Tok::Sym(b'F'), Tok::Push, Tok::Turn(1), Tok::Sym(b'G'), Tok::Pop]);
    assert_eq!(s.rule(b'G'), Some(&[Tok::Sym(b'G'), Tok::Sym(b'G')][..]));
    assert_eq!(s.rule(b'F'), None);
}

#[test]
fn every_command_reads_as_fractint_documents_it() {
    let w = parse_word("F G D M + - | ! [ ] @2 @I2 @Q4 @IQ4 \\30 /22.5 C5 <2 > >3 X C", 1, 1, false).unwrap();
    assert_eq!(
        w,
        vec![
            Tok::Sym(b'F'),
            Tok::Sym(b'G'),
            Tok::Sym(b'D'),
            Tok::Sym(b'M'),
            Tok::Turn(1),
            Tok::Turn(-1),
            Tok::Around,
            Tok::Reverse,
            Tok::Push,
            Tok::Pop,
            Tok::Scale(2.0),
            Tok::Scale(0.5),
            Tok::Scale(2.0),
            Tok::Scale(0.5),
            Tok::TurnBy(30.0),
            Tok::TurnBy(-22.5),
            Tok::SetColour(5),
            Tok::AddColour(2),
            Tok::AddColour(-1),
            Tok::AddColour(-3),
            Tok::Sym(b'X'),
            // `C` with no number after it is a symbol (Paul Bourke's kolams use it as one).
            Tok::Sym(b'C'),
        ]
    );
}

#[test]
fn a_number_ends_at_a_space_so_digit_symbols_survive_the_round_trip() {
    // Penrose-style systems use digits as symbols: `@2` then the symbol `7`.
    // And a number then a `.` vertex would read as `3.`.
    let w = vec![Tok::Scale(2.0), Tok::Sym(b'7'), Tok::Sym(b'C'), Tok::Sym(b'5'), Tok::SetColour(3), Tok::Vertex];
    let text = word_text(&w);
    assert_eq!(parse_word(&text, 1, 1, false).unwrap(), w, "{text}");
}

#[test]
fn every_library_system_round_trips_through_its_text() {
    for e in super::super::library::SYSTEMS {
        let s = e.system().unwrap();
        let back = LSystem::parse(&s.to_text()).unwrap_or_else(|err| panic!("{}: {err}\n{}", e.name, s.to_text()));
        assert_eq!(back, s, "{}", e.name);
    }
}

#[test]
fn odd_values_round_trip_exactly() {
    let mut s = LSystem::new("Odd");
    s.angle = Angle::Degrees(25.7);
    s.heading = -12.5;
    s.axiom = vec![Tok::Scale(1.0 / 3.0), Tok::TurnBy(-0.1), Tok::AddColour(-7), Tok::Sym(b'F')];
    s.rules[b'F' as usize] = Some(vec![Tok::Scale(std::f64::consts::SQRT_2), Tok::Sym(b'F')]);
    s.roles[b'F' as usize] = Role::Move;
    s.roles[b'A' as usize] = Role::Draw;
    assert_eq!(LSystem::parse(&s.to_text()).unwrap(), s, "{}", s.to_text());
}

#[test]
fn angles_that_divide_the_circle_are_divisions() {
    assert_eq!(Angle::Degrees(60.0).division(), Some(6));
    assert_eq!(Angle::Degrees(22.5).division(), Some(16));
    assert_eq!(Angle::Degrees(36.0).division(), Some(10));
    assert_eq!(Angle::Degrees(25.7).division(), None);
    assert_eq!(Angle::Degrees(0.0).division(), None);
    assert_eq!(Angle::Degrees(-90.0).division(), None, "a negative angle mirrors; it is not a division");
    assert_eq!(Angle::Division(5).division(), Some(5));
    assert_eq!(Angle::Division(8).degrees(), 45.0);
}

#[test]
fn a_unicode_minus_from_a_book_is_a_minus() {
    let s = LSystem::parse("angle 60\naxiom F\nF = F+F\u{2212}\u{2212}F+F\n").unwrap();
    assert_eq!(s.rule(b'F').unwrap()[3], Tok::Turn(-1));
    // And a lone CR ends a line.
    let s = LSystem::parse("angle 90\raxiom F\rF = FF\r").unwrap();
    assert_eq!(s.rule(b'F').unwrap().len(), 2);
}

#[test]
fn refusals_say_what_and_where() {
    let e = err("angle 90\naxiom F\nF = F[+F\n");
    assert_eq!((e.line, e.message.as_str()), (3, "a '[' is never closed"));
    let e = err("angle 90\naxiom F]\n");
    assert_eq!((e.line, e.col), (2, 8));
    assert!(e.message.contains("without a '['"));
    let e = err("angle 90\naxiom F@\n");
    assert_eq!((e.line, e.col), (2, 8));
    assert!(e.message.contains("needs a number"));
    let e = err("angle 90\naxiom F\nF = F\nF = FF\n");
    assert_eq!(e.line, 4);
    assert!(e.message.contains("second production"));
    let e = err("angle 90\naxiom F\n+ = F\n");
    assert!(e.message.contains("cannot have a production"), "{e}");
    let e = err("angle 90\nwibble F\n");
    assert!(e.message.contains("unknown key"), "{e}");
    let e = err("angle 90\n");
    assert!(e.message.contains("no axiom"), "{e}");
    let e = err("axiom F\n");
    assert!(e.message.contains("no angle"), "{e}");
    let e = err("angle sixty\naxiom F\n");
    assert_eq!((e.line, e.col), (1, 7));
    let e = err("angle /0\naxiom F\n");
    assert_eq!(e.line, 1);
    let e = err("angle 90\naxiom F\u{e9}\n");
    assert!(e.message.contains("ASCII"), "{e}");
    let e = err("angle 90\naxiom F@0\n");
    assert!(e.message.contains("between"), "{e}");
    let e = err("angle 90\norder 99999\naxiom F\n");
    assert!(e.message.contains("order"), "{e}");
    let long = format!("angle 90\naxiom {}\n", "F".repeat(MAX_WORD + 1));
    assert!(err(&long).message.contains("at most"));
}

#[test]
fn the_default_colouring_follows_the_system() {
    let plain = LSystem::parse("angle 60\naxiom F\nF = F+F--F+F\n").unwrap();
    assert_eq!(plain.colouring(), Colouring::Position);
    let plant = LSystem::parse("angle 20\naxiom F\nF = F[+F]F\n").unwrap();
    assert_eq!(plant.colouring(), Colouring::Depth);
    let indexed = LSystem::parse("angle 20\naxiom F\nF = F[+F<1]F\n").unwrap();
    assert_eq!(indexed.colouring(), Colouring::Index);
    let own = LSystem::parse("angle 20\ncolour plain\naxiom F\nF = F[+F<1]F\n").unwrap();
    assert_eq!(own.colouring(), Colouring::Plain);
}
