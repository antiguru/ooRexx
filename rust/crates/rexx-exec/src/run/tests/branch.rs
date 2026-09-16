/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 2026 Rexx Language Association. All rights reserved.          */
/*                                                                            */
/* This program and the accompanying materials are made available under       */
/* the terms of the Common Public License v1.0 which accompanies this         */
/* distribution. A copy is also available at the following address:           */
/* https://www.oorexx.org/license.html                                        */
/*                                                                            */
/*----------------------------------------------------------------------------*/

use super::*;

// ---- LABEL and NOP ----

#[test]
fn a_label_is_a_traced_no_op() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"here: say 'hit'"),
        b"hit\n".to_vec(),
        "the label itself produces no output of its own"
    );
}

#[test]
fn nop_is_a_no_op() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"nop\nsay 'after'"),
        b"after\n".to_vec()
    );
}

// ---- IF/THEN/ELSE ----

/// The discriminating shape for a wrong `false_target`/`then_exit`: a
/// true condition whose `ELSE` branch has a **different** side effect
/// than the `THEN` branch. A version that lets the true path fall
/// through into the `ELSE` marker without skipping it (the naive
/// fallthrough this task's whole design exists to avoid -- see
/// `run_bounded`'s doc comment) runs *both* assignments and prints
/// `abXc`. A version that never runs the `ELSE` branch on the false path
/// at all prints `aXc` for the companion case below. Only the correct
/// wiring prints `abc` and `aXc` respectively.
#[test]
fn if_then_else_runs_exactly_one_branch() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"a = 'a'\nif 1 = 1 then a = a || 'b'\nelse a = a || 'X'\na = a || 'c'\nsay a"
        ),
        b"abc\n".to_vec(),
        "true: runs the THEN branch and skips the ELSE branch entirely"
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"a = 'a'\nif 1 = 0 then a = a || 'b'\nelse a = a || 'X'\na = a || 'c'\nsay a"
        ),
        b"aXc\n".to_vec(),
        "false: runs the ELSE branch and skips the THEN branch entirely"
    );
}

#[test]
fn if_then_with_no_else_falls_through_on_false() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"a = 'a'\nif 1 = 0 then a = a || 'b'\na = a || 'c'\nsay a"
        ),
        b"ac\n".to_vec()
    );
}

/// The `ast.rs`/`block.rs`-derived discriminating case: an `IF`/`ELSE
/// IF`/`ELSE` chain (no `DO`, which Task 11 owns) where each link's
/// `false_target` is the next link's own condition and the whole
/// chain's resume point sits after all of them. Run once per branch so
/// a wrong `false_target` (skipping or re-testing a link) and a wrong
/// `then_exit`/resume (landing inside a later link, matching
/// `if_else_chain.rex`'s own framing) are both visible.
#[test]
fn if_else_if_chain_takes_exactly_one_link() {
    let source = |n: &str| -> Vec<u8> {
        format!(
            "n = {n}\na = ''\nif n = 1 then a = a || 'one'\nelse if n = 2 then a = a || 'two'\nelse if n = 3 then a = a || 'three'\nelse a = a || 'other'\na = a || '-after'\nsay a"
        )
        .into_bytes()
    };
    for (n, expected) in [
        ("1", "one-after\n"),
        ("2", "two-after\n"),
        ("3", "three-after\n"),
        ("4", "other-after\n"),
    ] {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, &source(n)),
            expected.as_bytes().to_vec(),
            "n = {n}"
        );
    }
}

#[test]
fn if_condition_that_is_not_0_or_1_raises_34_1() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"if 'x' then nop").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (34, 1));
    assert_eq!(raised.additional, vec![b"x".to_vec()]);
}

/// A comma list is 34.6 regardless of which element fails, never 34.1.
/// Measured against the oracle (brief's own transcript).
#[test]
fn if_condition_that_is_a_comma_list_raises_34_6_not_34_1() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"if 'x', 1 then nop").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (34, 6));
    assert_eq!(raised.additional, vec![b"x".to_vec()]);
}

#[test]
fn a_true_comma_list_condition_is_an_and_of_its_parts() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"if 1, 1 then say 'hit'\nelse say 'miss'"),
        b"hit\n".to_vec()
    );
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"if 1, 0 then say 'hit'\nelse say 'miss'"),
        b"miss\n".to_vec()
    );
}

// ---- SELECT / WHEN ----

#[test]
fn select_when_runs_exactly_the_first_matching_when() {
    let source = |n: &str| -> Vec<u8> {
        format!(
            "n = {n}\nr = ''\nselect\n  when n = 1 then r = r || 'one'\n  when n = 2 then r = r || 'two'\n  when n = 3 then r = r || 'three'\n  otherwise r = r || 'other'\nend\nr = r || '-after'\nsay r"
        )
        .into_bytes()
    };
    for (n, expected) in [
        ("1", "one-after\n"),
        ("2", "two-after\n"),
        ("3", "three-after\n"),
        ("4", "other-after\n"),
    ] {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, &source(n)),
            expected.as_bytes().to_vec(),
            "n = {n}"
        );
    }
}

/// The `select_when_bodies.rex` discriminating shape, reproduced without
/// `DO` (Task 11's): each `WHEN`'s consequence is a nested `SELECT`
/// spanning several flat instructions of its own
/// (`Select`/`When`/`Then`/two assignments/`End`), so a wrong exit that
/// lands even one instruction into a *later* `WHEN`'s span, rather than
/// cleanly past the whole outer `SELECT`, shows up as an extra or
/// missing accumulator update. Run for every `n` so a wrong exit wired
/// to (for instance) always resume after the *first* `WHEN` would still
/// be caught on `n = 2` or `n = 3`.
#[test]
fn select_when_wrong_exit_would_land_in_a_later_whens_multi_instruction_body() {
    let source = |n: &str| -> Vec<u8> {
        format!(
            "n = {n}\nr = ''\nselect\n  when n = 1 then select\n    when 1 = 1 then r = r || 'w1a'\n    otherwise nop\n  end\n  when n = 2 then select\n    when 1 = 1 then r = r || 'w2a'\n    otherwise nop\n  end\n  when n = 3 then select\n    when 1 = 1 then r = r || 'w3a'\n    otherwise nop\n  end\n  otherwise r = r || 'w4a'\nend\nr = r || '-done'\nsay r"
        )
        .into_bytes()
    };
    for (n, expected) in [
        ("1", "w1a-done\n"),
        ("2", "w2a-done\n"),
        ("3", "w3a-done\n"),
        ("4", "w4a-done\n"),
    ] {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, &source(n)),
            expected.as_bytes().to_vec(),
            "n = {n}: exactly one nested SELECT's body ran, and nothing else did"
        );
    }
}

/// `when 1 = 1 then` followed immediately by `when 2 = 2 then n = 42` is
/// the second `WHEN`'s absorption into the first's (empty) consequence
/// (`ast.rs`'s own doc comment on `Select::whens`): the second `WHEN` is
/// never collected, and its own `exit` is permanently `None`. Measured
/// against the oracle: prints `0`, rc 0 -- the absorbed `WHEN`'s own
/// condition being true (`2 = 2`) must not run `n = 42`, and `OTHERWISE`
/// must not run either, because one true `WHEN` (the outer one) already
/// ended the whole `SELECT`.
#[test]
fn an_absorbed_when_runs_neither_its_own_consequence_nor_otherwise() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"n = 0\nselect\n  when 1 = 1 then\n    when 2 = 2 then n = 42\n  otherwise\n    n = 99\nend\nsay n"
        ),
        b"0\n".to_vec()
    );
}

/// The true-condition-variant is accepted at rc 0 (`ast.rs:776`, and the
/// brief's own transcript) -- the *false*-condition variant is a
/// separate, upstream oracle segfault (SF #2018) and is deliberately not
/// probed here.
#[test]
fn when_absorbing_a_when_parses_and_runs_at_rc_0() {
    let mut interp = Interp::new();
    assert_eq!(
        run_source(
            &mut interp,
            b"select\n  when 1 = 1 then\n    when 2 = 2 then nop\nend"
        )
        .expect("accepted, rc 0"),
        None
    );
}

/// **Critical, found by review, not by this task's own probes.** The
/// mutation this kills is exactly the one the old `Ok(Flow::Next)`
/// arm *was*: treating an absorbed `WHEN`'s own condition as never
/// evaluated at all, rather than evaluated and discarded. A version
/// that reverts `InstructionKind::When`'s arm to a bare `Ok(Flow::
/// Next)` makes this test hang around `run_source` returning `Ok`
/// (prints `after`, rc 0) where the oracle raises 42.3 at rc 214 --
/// `an_absorbed_when_runs_neither_its_own_consequence_nor_otherwise`,
/// just above, cannot distinguish the two models (both give `n = 0`
/// for a side-effect-free true condition), which is exactly why every
/// probe before this review used one and missed this.
#[test]
fn an_absorbed_whens_raising_condition_escapes_even_though_its_own_consequence_never_runs() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"select\n  when 1 = 1 then\n    when 1 / 0 then nop\n  otherwise nop\nend\nsay 'after'",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (42, 3));
}

/// The companion half: a **true** absorbed condition with a printable
/// side effect still never runs its own consequence (matching the
/// existing `n = 0` test, restated with `SAY` so a wrong "the
/// absorbed WHEN's branch is taken" model would be caught by output
/// content rather than only by a variable's final value) -- measured,
/// this task's own report: `ABSORBED-RAN` never prints, only `after`.
#[test]
fn an_absorbed_whens_true_condition_still_never_runs_its_own_consequence() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"select\n  when 1 = 1 then\n    when 2 = 2 then say 'ABSORBED-RAN'\nend\nsay 'after'"
        ),
        b"after\n".to_vec(),
        "a reverted fix would print ABSORBED-RAN first"
    );
}

/// F3, found by review: unlike a plain `WHEN`'s absorbed form, a
/// `WHEN CASE`'s own absorbed form branches to its own `false_target`
/// on a false match. The mutation this kills: reverting
/// `InstructionKind::WhenCase`'s arm to the old evaluate-and-discard
/// shape (matching `When`'s own, still correct for `When`) makes this
/// program print only `after`, where the oracle -- and this fix --
/// print `O` then `after`. Verified by mutation: reverting made this
/// test fail with exactly that wrong output, while
/// `an_absorbed_whens_true_condition_still_never_runs_its_own_
/// consequence` (a `WHEN`, not a `WHEN CASE`) stayed green, confirming
/// the fix is scoped to `WhenCase` alone.
#[test]
fn an_absorbed_whencases_false_condition_branches_to_its_own_false_target() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"select case 2\n  when 2 then\n    when 3 then nop\n  otherwise say 'O'\nend\nsay 'after'"
        ),
        b"O\nafter\n".to_vec()
    );
}

/// The companion true-match case, matching the coordinator's own
/// phrase "matches on both sides": a true absorbed `WhenCase` still
/// never runs its own consequence, exactly like a plain `WHEN`'s own
/// true-absorbed case -- this is *not* new behaviour F3 introduced, it
/// is what this crate already did before the fix, re-pinned here so a
/// future change to the false-path fix cannot silently start running
/// the true path's own consequence too.
#[test]
fn an_absorbed_whencases_true_condition_still_never_runs_its_own_consequence() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"select case 2\n  when 2 then\n    when 2 then say 'INNER'\n  otherwise say 'O'\nend\nsay 'after'"
        ),
        b"after\n".to_vec()
    );
}

#[test]
fn select_with_no_when_true_and_no_otherwise_raises_7_3() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"select\n  when 1 = 0 then nop\nend").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (7, 3));
    assert_eq!(raised.additional, Vec::<Vec<u8>>::new());
}

/// F3's own perimeter, found by review: an absorbed `WhenCase`'s
/// false-branch escape landing directly on `END` reports 7.3 at the
/// escape's own residual indent (4, the absorbed condition's own
/// depth of 6 minus 2), not `END`'s own lexical `static_indent` (0,
/// top-level `SELECT`). The mutation this kills: removing the
/// `self.pending_escape_indent = Some(...)` assignment in
/// `InstructionKind::WhenCase`'s own false-branch arm (or reverting
/// `record_failure_site` to ignore it) makes this test's own `indent`
/// read back `0` instead of `4`, while `select_with_no_when_true_and_
/// no_otherwise_raises_7_3`, just above -- the *ordinary*, non-
/// absorbed 7.3 path, unaffected by this fix -- stays green either
/// way, confirming the fix is scoped to the escape path alone.
#[test]
fn an_absorbed_whencases_escaping_false_branch_reports_end_at_its_own_residual_indent() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"select case 2\n  when 2 then\n    when 3 then nop\nend\nsay 'after'",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (7, 3));
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(indent, 4);
}

/// The same shape, one `DO` level deeper -- `indent_offset`'s own doc
/// comment (`lib.rs`) has the full argument for why the answer stays
/// `6`, not `8`: the offset past an *ordinary* `SELECT`-level
/// construct's own depth is the constant `4`, not a function of how
/// deep the absorbed condition itself sits, and both grow by the same
/// amount together under nesting. The mutation this kills: reverting
/// `indent_offset`'s own assignment to `self.clause_state.current_value_indent.
/// saturating_sub(2)` (an earlier, wrong version of this same fix)
/// makes this test read back `6` (`8 - 2`) instead of `4`, while the
/// non-nested version just above still reads back the right answer
/// either way (`6 - 2` and the constant `4` coincide at the top
/// level) -- this is the one test that distinguishes them.
#[test]
fn an_absorbed_whencases_escape_to_end_reports_the_same_constant_offset_nested() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"do i = 1 to 1\n  select case 2\n    when 2 then\n      when 3 then nop\n  end\nend\nsay 'after'",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (7, 3));
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(indent, 6);
}

/// **F-EX1, Important, found by the whole-branch review, not by this
/// task's own probes.** An absorbed `WhenCase`'s own false-branch
/// escape landing on `OTHERWISE` used to leave `Select`'s own arm
/// through a bare `Flow::Goto` that `leave_select` had no way to
/// recognise, so `OTHERWISE`'s own body ran under whichever *outer*
/// construct received that `Goto` -- with no `SELECT` frame on the
/// search a `LEAVE` naming the enclosing `SELECT LABEL` needs to find.
/// The mutation this kills: reverting the escape-redirect check in
/// `Select`'s own arm (the `if let Flow::Goto(target) = flow && *
/// otherwise == Some(target)` branch, calling `Op::EnterOtherwise` instead
/// of forwarding the bare `Goto`) makes `leave s` search *outward* from
/// outside this `SELECT` and find nothing, raising 28.3 at rc 228
/// instead of resuming past the `SELECT` cleanly -- reproduced by
/// actually reverting it (this task's report has the transcript).
#[test]
fn an_absorbed_whencases_escape_to_otherwise_still_finds_the_enclosing_selects_own_label() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"select label s case 2\n  when 2 then\n    when 3 then nop\n  otherwise say 'O'\n  \
              leave s\nend\nsay 'after'"
        ),
        b"O\nafter\n".to_vec()
    );
}

/// The companion half of F-EX1: a **named `ITERATE`** inside the same
/// escaped `OTHERWISE`, naming the enclosing `SELECT LABEL`, is 28.5
/// (matches the name, but a `SELECT` is never a repetitive block) --
/// not 28.4 (no match at all), which is what it read as before this
/// fix, because the search never reached `leave_select`'s own name
/// check at all. Distinguishes "the frame is restored" from "the frame
/// is restored, and the specific consumption rule inside it still
/// applies", which the `LEAVE` test above alone cannot.
#[test]
fn an_absorbed_whencases_escape_to_otherwise_reports_a_named_iterate_as_28_5_not_28_4() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"select label s case 2\n  when 2 then\n    when 3 then nop\n  otherwise say 'O'\n  \
          iterate s\nend\nsay 'after'",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (28, 5));
}

#[test]
fn select_with_no_when_true_and_an_otherwise_runs_it_without_error() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"select\n  when 1 = 0 then nop\n  otherwise say 'lo'\nend"
        ),
        b"lo\n".to_vec()
    );
}

#[test]
fn when_condition_that_is_not_0_or_1_raises_34_2() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"select\n  when 'x' then nop\nend").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (34, 2));
    assert_eq!(raised.additional, vec![b"x".to_vec()]);
}

/// **The coordinator's own finding, fixed after the first round.** A
/// `WHEN`'s own `step` arm is a pure no-op (`Select`'s own arm reads it
/// as data instead), so a raise while evaluating its *condition* never
/// went through a `Op::Clause`'s region call for the `WHEN` itself --
/// the first version of this task attributed it to the enclosing
/// `SELECT`, wrong clause *and* wrong line, measured against the
/// oracle. `record_failure_site`'s own doc comment on `Select`'s call
/// sites has the fix. Checks `failure_site` directly (line and clause
/// text), which `raised.number`/`.sub` alone -- every other test in
/// this file -- cannot: both were already unaffected by the bug, since
/// the bug is entirely in *which clause* gets echoed, not in which
/// condition failed.
#[test]
fn a_when_conditions_own_failure_is_attributed_to_the_when_not_the_select() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"select\nwhen 'x' then nop\nend").unwrap_err();
    let FailureSite::Clause { line, text, .. } = interp
        .failure_site
        .expect("a raised condition always resolves a site when source is Some")
    else {
        panic!("not a clause site")
    };
    assert_eq!(line, 2, "the WHEN's own line, not the SELECT's (line 1)");
    assert_eq!(
        text,
        b"when 'x' ".to_vec(),
        "the WHEN's own clause text, not \"select\""
    );
}

/// The line has to move with the failing `WHEN`, not merely differ from
/// the `SELECT`'s -- a test whose expected line is the first `WHEN`'s
/// cannot tell a correct resolution from one that defaults to
/// "whichever `WHEN` this loop happens to be looking at first".
#[test]
fn the_second_of_two_whens_own_failure_moves_the_line_with_it() {
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"select\nwhen 1 = 0 then nop\nwhen 'x' then nop\nend",
    )
    .unwrap_err();
    let FailureSite::Clause { line, text, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(line, 3, "the second WHEN's own line, not the first's (2)");
    assert_eq!(text, b"when 'x' ".to_vec());
}

/// A `SELECT CASE` expression that itself raises is the `SELECT`'s own
/// clause -- confirmed against the oracle rather than assumed, since
/// `case` is evaluated directly inside `Select`'s own `step` call and
/// the coordinator asked this be checked, not taken on faith.
#[test]
fn a_select_cases_own_expression_failure_is_attributed_to_the_select() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"select case (1/0)\nwhen 1 then nop\nend").unwrap_err();
    let FailureSite::Clause { line, text, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(line, 1);
    assert_eq!(text, b"select case (1/0)".to_vec());
}

/// A `WhenCase` value expression that raises is the `WHEN`'s own
/// clause, the same rule as a plain `WHEN`'s condition -- both go
/// through `Select`'s own explicit-match-and-record path, never
/// through `Op::Clause`'s region for the `When`/`WhenCase` node itself.
#[test]
fn a_whencase_values_own_failure_is_attributed_to_the_when_not_the_select() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"select case 1\nwhen (1/0) then nop\nend").unwrap_err();
    let FailureSite::Clause { line, text, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(line, 2);
    assert_eq!(text, b"when (1/0) ".to_vec());
}

/// A raise inside an `OTHERWISE` branch was already correct before this
/// round's fix (`OTHERWISE`'s own body runs through the outer loop's
/// ordinary `Op::Clause`'s region, never through `Select`'s own
/// explicit-match path), and stays that way -- checked because the
/// coordinator asked for it explicitly, not assumed from the `WHEN` fix.
#[test]
fn a_raise_inside_an_otherwise_branch_is_attributed_to_its_own_clause() {
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"select\nwhen 1 = 0 then nop\notherwise\n  say 1/0\nend",
    )
    .unwrap_err();
    let FailureSite::Clause { line, text, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(line, 4);
    assert_eq!(text, b"say 1/0".to_vec());
}

/// A raise inside a matched `WHEN`'s **body** (not its condition) has to
/// go through `run_bounded`'s own `Op::Clause`'s region calls with
/// `source` actually threaded through, which is a different path from
/// every other test in this section: those all check a condition/value
/// expression `Select`'s own arm evaluates directly, and
/// `a_raise_inside_an_otherwise_branch_is_attributed_to_its_own_clause`
/// runs through the *outer* loop's `Op::Clause`'s region, never through a
/// `run_bounded` nested inside `If`/`Select`'s own arm. That last
/// distinction matters and an earlier wording of it was wrong: in the
/// test harness `run_source` routes everything through its own top-level
/// `run_bounded` with `source` supplied directly, so "never through
/// `run_bounded` at all" is true of the production outer loop only.
#[test]
fn a_raise_inside_a_matched_whens_body_is_attributed_to_its_own_clause() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"select\nwhen 1 = 1 then\n  say 1/0\nend").unwrap_err();
    let FailureSite::Clause { line, text, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(
        line, 3,
        "the WHEN's own body clause, not the SELECT's (line 1)"
    );
    assert_eq!(text, b"say 1/0".to_vec());
}

/// The `IF` analogue of the `WHEN`-body test above: a raise inside the
/// matched `THEN` branch's own body, which likewise only ever reaches
/// `Op::Clause`'s region through `run_bounded`. Confirmed by the same
/// mutation (`None` for `source` at `If`'s own `run_bounded` call site
/// made this fail too, alongside the `WHEN`-body test, both restored
/// after).
#[test]
fn a_raise_inside_an_ifs_then_body_is_attributed_to_its_own_clause() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"if 1 = 1 then\n  say 1/0").unwrap_err();
    let FailureSite::Clause { line, text, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(
        line, 2,
        "the THEN branch's own body clause, not the IF's (line 1)"
    );
    assert_eq!(text, b"say 1/0".to_vec());
}

// ---- SELECT CASE / WhenCase ----

/// **The central `WhenCase` rule.** `select case 2` / `when 1, 2 then`
/// matches (an OR of `==`), while a plain `select` / `when 1, 2` on the
/// same non-logical value is 34.6 (an AND, each element checked for
/// `0`/`1`) -- the two commas parse into the same-looking node and mean
/// opposites (`ast.rs:801-815`, and the brief's own framing).
#[test]
fn whencase_comma_is_an_or_of_equals_the_opposite_of_a_plain_whens_and() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"select case 2\n  when 1, 2 then say 'hit'\n  otherwise say 'miss'\nend"
        ),
        b"hit\n".to_vec(),
        "SELECT CASE: an OR of == comparisons"
    );

    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"select\n  when 1, 2 then nop\nend").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!(
        (raised.number, raised.sub),
        (34, 6),
        "plain SELECT: 2 is not a logical value, an AND-with-a-check, not an OR-of-=="
    );
    assert_eq!(raised.additional, vec![b"2".to_vec()]);
}

/// `==` is strict: byte-for-byte, no padding, no numeric awareness.
/// Measured (D15's own example, restated in the brief): `'007'` does not
/// match `when 7`, because the two are not byte-identical.
#[test]
fn select_case_compares_with_strict_equality_not_numeric_equality() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"select case '007'\n  when 7 then say 'hit'\n  otherwise say 'miss'\nend"
        ),
        b"miss\n".to_vec()
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"select case 7\n  when 7 then say 'hit'\n  otherwise say 'miss'\nend"
        ),
        b"hit\n".to_vec()
    );
}

#[test]
fn select_case_evaluates_its_own_expression_and_runs_exactly_the_matching_when() {
    let source = |v: &str| -> Vec<u8> {
        format!("select case {v}\n  when 1 then say 'one'\n  when 2 then say 'two'\n  otherwise say 'other'\nend")
            .into_bytes()
    };
    for (v, expected) in [("1", "one\n"), ("2", "two\n"), ("3", "other\n")] {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, &source(v)),
            expected.as_bytes().to_vec(),
            "v = {v}"
        );
    }
}

// ---- SELECT with a LABEL, and a nested SELECT/IF ----

#[test]
fn a_labelled_select_with_otherwise_runs_normally() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"select label s\n  when 1 = 0 then nop\n  otherwise say 'lo'\nend"
        ),
        b"lo\n".to_vec()
    );
}

#[test]
fn a_select_nested_inside_an_ifs_then_branch_is_fully_resolved_before_resuming() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"a = 'a'\nif 1 = 1 then select\n  when 1 = 1 then a = a || 'b'\nend\na = a || 'c'\nsay a"
        ),
        b"abc\n".to_vec()
    );
}
