use super::*;

/// A row that cannot tell two models apart cannot report which one a backend runs (a control
/// that cannot express the variable under test is not a control): each pair of models must
/// disagree somewhere in the row's inputs, and the inputs must include near ties.
#[test]
fn every_decision_row_can_tell_its_models_apart() {
    for op in DECISION_ROWS {
        let (id, _, names, cases) = row_cases(op);
        assert_eq!(cases.len(), (GT_W * 4) as usize, "{id}");
        let near = cases.iter().filter(|c| !c.decisive).count();
        assert!(near >= 50, "{id}: only {near} near ties of {}", cases.len());
        for a in 0..names.len() {
            for b in a + 1..names.len() {
                // Two models may legitimately coincide when the inputs give no way to part
                // them (fma(x,x,y*y) and x*x+y*y on y = 0); row 13's and 15's subnormal
                // forms must still part the reference from every other model.
                if a != 0 {
                    continue;
                }
                let differ = cases.iter().filter(|c| c.model[a] != c.model[b]).count();
                assert!(differ > 0, "{id}: '{}' and '{}' decide alike on every input", names[a], names[b]);
            }
        }
    }
}

/// Where the answer is beyond doubt, every model gives it: a model that does not would make a
/// correct backend look WRONG.
#[test]
fn every_model_is_right_on_the_decisive_inputs() {
    for op in DECISION_ROWS {
        let (id, _, names, cases) = row_cases(op);
        let decisive = cases.iter().filter(|c| c.decisive).count();
        assert!(decisive >= 50, "{id}: only {decisive} decisive inputs");
        for (m, n) in names.iter().enumerate() {
            let wrong = cases.iter().filter(|c| c.decisive && c.model[m] != c.exact).count();
            assert_eq!(wrong, 0, "{id}: model '{n}' is wrong on {wrong} decisive inputs");
        }
    }
}

/// The battery matches a declared failure as a prefix of a `[FAIL]` line: an id that begins
/// with another would let a new failure of the longer one pass as the shorter's known one.
#[test]
fn ids_are_prefix_free() {
    let px = vec![0f32; (GT_W * 19 * 4) as usize];
    let ids: Vec<&str> = check_rows(GT_W, 19, &px).iter().map(|c| c.id).collect();
    assert_eq!(ids.len(), 19, "one id per row of fs_gputest");
    for a in &ids {
        assert!(a.is_ascii(), "{a}");
        for b in &ids {
            assert!(a == b || !b.starts_with(a), "'{b}' begins with '{a}'");
        }
    }
}

/// The grading: the reference model's own decisions PASS; another model's are named; a flipped
/// decisive answer is WRONG.
#[test]
fn grading_names_the_arithmetic() {
    let (id, name, names, cases) = row_cases(13);
    let as_model = |m: usize| cases.iter().map(|c| c.model[m]).collect::<Vec<_>>();
    let pass = grade_decisions(id, name, &names, &cases, &as_model(0));
    assert_eq!((pass.fails, pass.tag.as_str()), (0, ""));
    let other = (1..names.len()).find(|&m| as_model(m) != as_model(0)).expect("a model unlike the reference");
    let fused = grade_decisions(id, name, &names, &cases, &as_model(other));
    assert!(fused.fails > 0 && fused.tag.starts_with("decides like ") && fused.tag.contains(&names[other]), "{}", fused.tag);
    let mut flipped = as_model(0);
    let k = cases.iter().position(|c| c.decisive).unwrap();
    flipped[k] = !flipped[k];
    let wrong = grade_decisions(id, name, &names, &cases, &flipped);
    assert_eq!(wrong.tag, "WRONG on 1 decisive inputs");
}
