/// The checked-in TOURS.md must match what the schema generates — regenerate it with
/// `fractadyne --dump-tour-schema > TOURS.md` after editing `TOUR_SCHEMA`. (Line endings are
/// normalized so a CRLF working copy still matches the LF the generator emits.)
#[test]
fn tour_schema_doc_current() {
    let generated = super::tour_schema_markdown();
    let committed = include_str!("../../../../TOURS.md").replace("\r\n", "\n");
    assert_eq!(
        generated, committed,
        "TOURS.md is stale — run `fractadyne --dump-tour-schema > TOURS.md` to regenerate"
    );
}
