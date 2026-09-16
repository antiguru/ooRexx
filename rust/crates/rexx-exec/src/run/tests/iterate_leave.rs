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

// ---- LEAVE/ITERATE: the block-stack rules ----

#[test]
fn bare_leave_stops_the_innermost_loop() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"do i = 1 to 5\nif i = 3 then leave\nsay i\nend"
        ),
        b"1\n2\n".to_vec()
    );
}

#[test]
fn bare_iterate_skips_the_rest_of_the_current_pass() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"do i = 1 to 4\nif i = 2 then iterate\nsay i\nend"
        ),
        b"1\n3\n4\n".to_vec()
    );
}

/// A bare `LEAVE`/`ITERATE` skips **transparently past** an unlabelled
/// `DO` block on its way to the nearest enclosing loop -- measured
/// against the oracle: this program prints `1` then `after`, not
/// `unreached`, because the innermost `DO` (a block, not a loop) never
/// intercepts the bare `LEAVE` at all.
#[test]
fn bare_leave_passes_transparently_through_an_unlabelled_simple_block() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"do i = 1 to 3\ndo\nsay i\nleave\nsay 'unreached'\nend\nend\nsay 'after'"
        ),
        b"1\nafter\n".to_vec()
    );
}

/// **Bare `LEAVE` in a simple `DO` block is 28.1** (not a loop, and
/// unlabelled, so nothing on the enclosing chain ever matches) -- but a
/// **labelled** simple block is leavable by its own explicit name,
/// which the next test pins.
#[test]
fn bare_leave_in_an_unlabelled_simple_block_with_nothing_enclosing_it_raises_28_1() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"do\nleave\nend").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (28, 1));
}

#[test]
fn a_labelled_simple_block_is_leavable_by_its_own_name() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"do label blk\nsay 'a'\nleave blk\nsay 'b'\nend\nsay 'after'"
        ),
        b"a\nafter\n".to_vec()
    );
}

/// **An ordinary clause label does not name a loop or a block.**
/// `outer:` here is a plain `Label` instruction, entirely separate from
/// the `DO`'s own `label` field (which is `i`, the control variable,
/// since no `LABEL` keyword was written) -- measured, `leave outer` is
/// 28.3, not a hit, exactly as `leave_nested_outer.rex`'s own comment
/// states.
#[test]
fn an_ordinary_clause_label_does_not_name_the_loop_it_precedes() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"outer: do i = 1 to 3\nleave outer\nend").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (28, 3));
    assert_eq!(raised.additional, vec![b"OUTER".to_vec()]);

    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"outer: do i = 1 to 3\niterate outer\nend").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (28, 4));
    assert_eq!(raised.additional, vec![b"OUTER".to_vec()]);
}

/// **What does name a loop for `LEAVE`/`ITERATE`**: a controlled loop's
/// own control variable, as an automatic label, with no `LABEL` keyword
/// needed at all. `leave i`/`iterate i` from inside a *nested* loop
/// reaches the outer one and unwinds the inner one on the way --
/// `leave_nested_outer.rex`'s own transcript, reproduced here.
#[test]
fn a_controlled_loops_own_control_variable_is_an_automatic_label() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"do outer = 1 to 3\ndo inner = 1 to 3\nif inner = 2 then leave outer\nsay 'inner' outer inner\nend\nsay 'after inner' outer\nend\nsay 'after outer'"
        ),
        b"inner 1 1\nafter outer\n".to_vec()
    );
}

#[test]
fn iterate_naming_the_outer_loops_control_variable_cuts_every_outer_pass_short() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"do outer = 1 to 3\ndo inner = 1 to 3\nif inner = 2 then iterate outer\nsay 'l' outer inner\nend\nsay 'outer-after' outer\nend"
        ),
        b"l 1 1\nl 2 1\nl 3 1\n".to_vec(),
        "outer-after never prints for any pass, since every one is cut short at inner = 2"
    );
}

/// **`leave sel` exits a `SELECT` only when it was written
/// `SELECT LABEL sel`.** An ordinary clause label in front of a
/// `SELECT` does not make it leavable (28.3, exactly like a loop).
#[test]
fn leave_by_name_exits_a_select_only_when_it_was_given_that_label() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"s: select label s\nwhen 1 = 1 then\ndo\nleave s\nend\notherwise\nnop\nend\nsay 'after'"
        ),
        b"after\n".to_vec()
    );

    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"select\nwhen 1 = 1 then\ndo\nleave sel\nend\notherwise\nnop\nend",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (28, 3));
    assert_eq!(raised.additional, vec![b"SEL".to_vec()]);
}

/// A named `LEAVE` reaching a `SELECT LABEL` from inside its
/// `OTHERWISE` branch -- the fix that routes `OTHERWISE`'s own body
/// through `Select`'s own `run_bounded`/`leave_select`, not the plain
/// `Goto` it used before Task 11 (`Select`'s own arm has the full
/// argument). Kills a version that still lets `OTHERWISE` fall through
/// to the outer loop directly: that version would propagate this
/// `LEAVE` all the way to `run_activation`'s own top level and raise
/// 28.3 instead of the clean exit this test asserts.
#[test]
fn leave_by_name_reaches_a_select_label_from_inside_its_otherwise_branch() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"select label s\nwhen 1 = 0 then nop\notherwise\nsay 'o'\nleave s\nsay 'unreached'\nend\nsay 'after'"
        ),
        b"o\nafter\n".to_vec()
    );
}

/// **`ITERATE` never accepts a labelled block or a labelled `SELECT`,
/// only a loop -- 28.5, not silently skipped, when the name matches but
/// the kind is wrong.**
#[test]
fn a_named_iterate_matching_a_labelled_simple_block_raises_28_5_not_28_4() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"do label x\nsay 1\niterate x\nend").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (28, 5));
    assert_eq!(raised.additional, vec![b"X".to_vec()]);
}

#[test]
fn a_named_iterate_matching_a_select_label_raises_28_5() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"select label s\nwhen 1 = 1 then\ndo\niterate s\nend\notherwise\nnop\nend",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (28, 5));
    assert_eq!(raised.additional, vec![b"S".to_vec()]);
}

#[test]
fn iterate_from_inside_a_select_nested_in_a_loop_skips_only_the_current_pass() {
    // iterate_from_select.rex's own transcript: an ITERATE inside a
    // SELECT's WHEN body must skip straight to the loop's own next
    // pass, past both the SELECT's own resume point and back up to the
    // enclosing DO, printing exactly one "skip"/"keep" pair per
    // iteration and never both for the same i.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"do i = 1 to 4\nselect\nwhen i = 2 then do\nsay 'skip' i\niterate\nend\notherwise nop\nend\nsay 'keep' i\nend"
        ),
        b"keep 1\nskip 2\nkeep 3\nkeep 4\n".to_vec()
    );
}

/// Two real, unnamed loops nested two deep, neither matching `zz`: both
/// own a search frame (`is_loop` true for a `Controlled` loop), so both
/// get popped and both reset the residual to their own `static_indent`
/// -- the outer one last, to `0` (top level), which is what survives to
/// the exhausted-search report. This is **not** "28.1-28.4 always
/// report zero" (that rule was wrong -- see `n1`/`n2`/`n3` below, and
/// `LeaveOrigin`'s own doc comment for the corrected one): it is zero
/// here specifically because the outermost popped frame happens to sit
/// at top level, the same way it did in every one of this task's own
/// original probes, which is exactly how the wrong rule looked right
/// for as long as it did.
#[test]
fn leave_no_match_through_two_real_loops_resets_to_the_outer_ones_own_indent() {
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"do i = 1 to 3\ndo j = 1 to 3\nleave zz\nend\nend",
    )
    .unwrap_err();
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(indent, 0);
}

/// The one intervening construct is an *unlabelled* `Simple` block,
/// which owns no search frame and is fully transparent -- so nothing
/// ever resets the residual, and it stays at the `ITERATE`'s own full
/// lexical depth all the way to the match (the outer labelled block,
/// which does not reset on a match either). Two real loops in
/// `leave_no_match_through_two_real_loops_resets_to_the_outer_ones_own_indent`
/// above land on the *same* final answer as this test for an entirely
/// different reason -- neither test tells the two rules apart, which is
/// this task's own original mistake and why the corrected rule below has
/// its own dedicated tests.
#[test]
fn iterate_wrong_kind_through_a_transparent_unlabelled_block_reports_full_lexical_depth() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"do label x\ndo\niterate x\nend\nend").unwrap_err();
    let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(indent, 4, "two DO frames deep, matching the oracle");
}

/// **The corrected 28.x indent rule, all fourteen points**: nine from
/// the reviewer's own probe (`p1`/`p2`/`p5`/`p8`/`p9`/`p10`/`p11`/`p12`/
/// `p13`, their own naming, kept so the report's cross-reference to
/// this table still resolves) plus five re-measured independently by
/// this task against the oracle before touching any code (`n1`-`n3`
/// entirely new shapes, `p1`/`p11` re-run to confirm the two that
/// already matched still do). Every row was captured with `cat -A`
/// against `build/bin/rexx`, byte for byte, not inferred.
#[test]
fn the_corrected_28x_indent_rule_matches_all_fourteen_probed_shapes() {
    for (name, source, expect) in [
        ("p5", &b"if 1=1 then leave"[..], 4),
        ("p9", &b"if 1=1 then do\nleave\nend"[..], 6),
        ("p12", &b"do\nleave\nend"[..], 2),
        ("p8", &b"if 1=1 then do i=1 to 3\nleave zz\nend"[..], 4),
        (
            "p2",
            &b"do label x\nselect\nwhen 1=1 then iterate x\notherwise nop\nend\nend"[..],
            2,
        ),
        (
            "p10",
            &b"do label x\nselect\nwhen 1=1 then do\niterate x\nend\notherwise nop\nend\nend"[..],
            2,
        ),
        (
            "p13",
            &b"do label x\ndo label y\niterate x\nend\nend"[..],
            2,
        ),
        (
            "p1",
            &b"do label x\nif 1=1 then do\niterate x\nend\nend"[..],
            8,
        ),
        (
            "p11",
            &b"select label s\nwhen 1=1 then iterate s\notherwise nop\nend"[..],
            6,
        ),
        // Independently added by this task, not in the reviewer's own
        // table: a bare LEAVE reaching only an unlabelled SELECT
        // (SELECT owns a frame unconditionally, even unlabelled, so
        // the pop still happens and still resets to its own indent).
        (
            "n1",
            &b"select\nwhen 1=1 then leave\notherwise nop\nend"[..],
            0,
        ),
        // Three real loops nested three deep, none matching: each pop
        // resets in turn, the outermost (top level) wins.
        (
            "n2",
            &b"do i=1 to 3\ndo j=1 to 3\ndo k=1 to 3\nleave zz\nend\nend\nend"[..],
            0,
        ),
        // A named ITERATE crossing one unlabelled real loop and one
        // unlabelled SELECT, matching neither: both pop, the outer
        // loop's own indent (0, top level) is what survives.
        (
            "n3",
            &b"do i=1 to 3\nselect\nwhen 1=1 then iterate zz\notherwise nop\nend\nend"[..],
            0,
        ),
    ] {
        let mut interp = Interp::new();
        run_source(&mut interp, source).unwrap_err();
        let FailureSite::Clause { indent, .. } = interp.failure_site.expect("a site was resolved")
        else {
            panic!("not a clause site")
        };
        assert_eq!(indent, expect, "{name}: {source:?}");
    }
}

/// **The `run_bounded` `Goto`-absorption trap, named in `Flow::Leave`'s
/// own doc comment.** A `DO` with an `ITERATE` in its body, nested
/// inside an `IF`'s `THEN`, run enough times that a version which
/// implemented repetition as a `Goto` back to the loop's own top
/// (rather than an internal `run_repeating` loop that never returns
/// mid-construct) would have that `Goto`'s target absorbed by the
/// enclosing `IF`'s own `run_bounded` -- which owns the whole `THEN`
/// range the `DO` sits inside -- and re-enter the `DO`'s own arm as a
/// fresh first pass, with its running total reset to `Simple`/`Count`'s
/// own initial state every time. That bug's own observable signature
/// is `n` staying stuck at whatever the first `ITERATE`d pass produced,
/// forever, or (depending on exactly how the re-entry is shaped)
/// hanging outright, rather than the small, exact total this test
/// asserts.
#[test]
fn leave_and_iterate_survive_a_do_nested_in_an_ifs_then_iterating_repeatedly() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"n = 0\nif 1 = 1 then do i = 1 to 5\nif i // 2 = 0 then iterate\nn = n + i\nend\nsay n"
        ),
        b"9\n".to_vec(),
        "1 + 3 + 5, the odd i's only -- a Goto-based re-entry would not \
         accumulate this total across the DO's own five iterations"
    );
}

// ---- the LEAVE/ITERATE search stops at the run_fragment boundary ----

/// The measured rule, all four families at once: a `LEAVE`/`ITERATE`
/// inside `INTERPRET` text never sees the enclosing loop, so an
/// enclosing `DO` that would have consumed it does not, and it is the
/// exhausted search instead. `run_fragment`'s own doc comment has the
/// oracle transcripts these numbers come from.
#[test]
fn a_fragments_leave_or_iterate_never_reaches_the_enclosing_loop() {
    for (source, number, sub, additional) in [
        (
            &b"do kk = 1 to 3\ninterpret \"leave\"\nend\n"[..],
            28u16,
            1u16,
            Vec::new(),
        ),
        (
            &b"do kk = 1 to 3\ninterpret \"iterate\"\nend\n"[..],
            28,
            2,
            Vec::new(),
        ),
        (
            &b"do label outer while 1\ninterpret \"leave outer\"\nend\n"[..],
            28,
            3,
            vec![b"OUTER".to_vec()],
        ),
        (
            &b"do label outer kk = 1 to 3\ninterpret \"iterate outer\"\nend\n"[..],
            28,
            4,
            vec![b"OUTER".to_vec()],
        ),
    ] {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, source).unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised for {source:?}, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (number, sub), "{source:?}");
        assert_eq!(raised.additional, additional, "{source:?}");
    }
}

/// The name in 28.3/28.4 is resolved against the **fragment's** symbol
/// table, which is the half of F-EX2 that survives the rule above.
#[test]
fn a_fragments_named_leave_is_resolved_against_the_fragments_own_table() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"bar = 1\ninterpret \"leave foo\"\n").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (28, 3));
    assert_eq!(raised.additional, vec![b"FOO".to_vec()]);
}

/// Review finding I1(a): `INTERPRET` traces `>>>` on the text it is about
/// to run, like every other value-producing arm, and it shipped without
/// doing so.
/// ```text
///      1 *-* trace r
///      2 *-* zz = 'nop'
///        >>>   "nop"
///      3 *-* interpret zz
///        >>>   "nop"
///      3 *-* nop
/// ```
#[test]
fn interpret_traces_the_text_it_is_about_to_run() {
    let mut interp = Interp::new();
    say_output_traced(&mut interp, b"zz = 'nop'\ninterpret zz");
    assert_eq!(
        interp.trace,
        concat!(
            "     1 *-* zz = 'nop'\n",
            "       >>>   \"nop\"\n",
            "     2 *-* interpret zz\n",
            "       >>>   \"nop\"\n",
            "     2 *-* nop\n",
        )
        .as_bytes()
    );

    let mut interp = Interp::new();
    say_output_traced(&mut interp, b"do kk = 1 to 1\ninterpret \"nop\"\nend");
    for expected in [&b"       >>>     \"nop\"\n"[..], &b"     2 *-*   nop\n"[..]] {
        assert!(
            interp
                .trace
                .windows(expected.len())
                .any(|window| window == expected),
            "expected a line carrying the enclosing DO's own two spaces \
             ({:?}), got {:?}",
            String::from_utf8_lossy(expected),
            String::from_utf8_lossy(&interp.trace)
        );
    }
}

/// The other side of the boundary rule: a loop written *inside* the
/// fragment consumes its own `LEAVE` normally, so the rule above is a
/// statement about crossing the boundary and not a blanket refusal.
/// Measured: this program prints two lines and exits 0.
#[test]
fn a_leave_inside_the_fragments_own_loop_is_consumed_there() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"interpret \"do jj = 1 to 5; say 'frag' jj; if jj = 2 then leave; end\"\nsay 'after'\n"
        ),
        b"frag 1\nfrag 2\nafter\n".to_vec()
    );
}
