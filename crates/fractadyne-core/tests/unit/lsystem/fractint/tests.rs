use super::*;
use crate::lsystem::{Role, Tok};

/// A small `.l` file in Fractint's format, written for this test.
const FILE: &str = "\
; a comment before any entry
Snowflake {   ; three Koch curves
  Angle 6
  Axiom F--F--F
  F=F+F--F+F
  }

Lower
{
  angle 4
  axiom fx
  x=x+yf+
  y=-fx-y
}

Commands { Angle 8
  Axiom F@IQ2\\45/90C5<2>|!
}
Broken {
  Angle 6
  Axiom F[
}
NoAngle {
  Axiom F
}
Open {
  Angle 4
";

#[test]
fn every_entry_is_read_and_a_bad_one_does_not_stop_the_rest() {
    let entries = parse_l_file(FILE);
    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["Snowflake", "Lower", "Commands", "Broken", "NoAngle", "Open"]);

    let s = entries[0].system.as_ref().unwrap();
    assert_eq!(s.angle, Angle::Division(6));
    assert_eq!(s.name, "Snowflake");
    assert_eq!(s.rule(b'F').unwrap().len(), 8);
    assert_eq!(entries[0].line, 2);

    // Case is ignored: `f` draws (it is `F`), `x` and `X` are one symbol.
    let s = entries[1].system.as_ref().unwrap();
    assert_eq!(s.axiom, vec![Tok::Sym(b'F'), Tok::Sym(b'X')]);
    assert!(s.rule(b'X').is_some() && s.rule(b'Y').is_some());
    assert_eq!(s.roles[b'F' as usize], Role::Draw);
    assert_eq!(entries[1].line, 8, "the name on the line before the brace");

    let s = entries[2].system.as_ref().unwrap();
    assert_eq!(
        s.axiom,
        vec![
            Tok::Sym(b'F'),
            Tok::Scale(1.0 / 2f64.sqrt()),
            Tok::TurnBy(45.0),
            Tok::TurnBy(-90.0),
            Tok::SetColour(5),
            Tok::AddColour(2),
            Tok::AddColour(-1),
            Tok::Around,
            Tok::Reverse,
        ]
    );

    let e = entries[3].system.as_ref().unwrap_err();
    assert_eq!(e.line, 21);
    assert!(e.message.contains("never closed"), "{e}");
    let e = entries[4].system.as_ref().unwrap_err();
    assert!(e.message.contains("no Angle"), "{e}");
    let e = entries[5].system.as_ref().unwrap_err();
    assert!(e.message.contains("never closed by a '}'"), "{e}");
}

#[test]
fn a_one_line_entry_and_crlf_endings_read() {
    let entries = parse_l_file("Tri { Angle 3 }\r\nT2 {\r\nAngle 3\r\nAxiom F+F+F\r\n}\r\n");
    assert_eq!(entries.len(), 2);
    assert!(entries[0].system.as_ref().unwrap_err().message.contains("no Axiom"));
    assert_eq!(entries[1].system.as_ref().unwrap().axiom.len(), 5);
}

#[test]
fn a_production_line_must_name_one_symbol() {
    let entries = parse_l_file("X {\nAngle 4\nAxiom F\nFF=F\n}\n");
    let e = entries[0].system.as_ref().unwrap_err();
    assert_eq!(e.line, 4);
    assert!(e.message.contains("one symbol"), "{e}");
}
