//! The four pins Task 11 requires on the `base/keyword` extractor:
//! conservation, absolute committed literals, a real floor, and the ooTest
//! revision the literals were measured at.
//!
//! # Why conservation alone would be a tautology, and what makes it one here
//!
//! `rows + dropped == calls` holds trivially at `0 + 0 == 0` (nothing
//! scanned) and at `0 + N == N` (everything dropped). What makes it
//! non-vacuous is that `calls` is counted **independently of the extractor**,
//! by [`rexx_extract::keyword::count_assert_same`], which does not parse Rexx
//! and only counts a substring -- so a scanner that silently sees nothing
//! cannot satisfy it, and neither can one that drops everything, because
//! [`the_row_floor`] below states a real lower bound on the rows side.
//! Those two plus [`base_keyword_yields_the_measured_counts`]'s absolute
//! literals are the four-part requirement.
//!
//! # The denominator is the *exact* `assertSame` spelling
//!
//! `base/keyword` contains 4,567 `self~assert*` occurrences of all
//! spellings. **2,561 match the prefix `self~assertSame`, but only 2,441 are
//! the method `assertSame`** -- the other 120 are `assertSameList`, a
//! different method that compares two lists and which a prefix test
//! swallows. This extractor's denominator is the **2,441**, and
//! `count_assert_same` implements exactly that rule.
//!
//! Taking the prefix instead would inflate the denominator to 2,561 and
//! silently classify 120 `assertSameList` calls as dropped `assertSame`
//! calls -- reporting a shortfall against a population that never existed.
//! The cost of the narrower choice is that those 120 are not counted at all,
//! anywhere, which is correct: they are a different assertion.
//!
//! **2,126 of the group's 4,567 assertions (46.6%) are deliberately outside
//! this population** -- every spelling other than `assertSame`, the largest
//! being `assertTrue` (797), `assertSyntaxError` (400) and `assertEquals`
//! (361). Nothing here claims anything about them.

use rexx_extract::find_test_groups;
use rexx_extract::keyword::{count_assert_same, extract_keyword};
use std::path::{Path, PathBuf};

/// The ooTest revision every absolute literal below was measured at. Named
/// in each failure message so a red count is diagnosable as `svn up` rather
/// than as a regression in this repository.
///
/// `ootest/` is **not** checked-in test data: it is git-ignored
/// (`.gitignore:6`), has zero tracked files, and exists only as an SVN
/// working copy of `svn.code.sf.net/p/oorexx/code-0/test/trunk`. Read it
/// back with `svn info ootest`.
const OOTEST_REVISION: &str = "r13178";

/// What a red absolute-literal test most likely means, appended to every
/// such failure.
fn provenance() -> String {
    format!(
        "\n\nThese literals were measured against ooTest {OOTEST_REVISION}. `ootest/` is an SVN \
         working copy, not checked-in test data, so the first thing to check is `svn info \
         ootest`: if it no longer reads {OOTEST_REVISION}, the corpus moved underneath this \
         test and the fix is to re-measure and re-commit these numbers, not to relax them. If \
         the revision still matches, this is a real change in what the extractor accepts."
    )
}

fn suite_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../ootest/ooRexx/base/keyword")
}

/// The committed list of `.testGroup` files the extractor scans, pinned the
/// way `rust/corpus/phase-4a.txt` is pinned.
fn committed_group_list() -> Vec<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/keyword-groups.txt");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect()
}

/// Every group's `(name, calls, rows, dropped)`, in `find_test_groups`
/// order.
fn measure() -> Vec<(String, usize, usize, usize)> {
    let mut out = Vec::new();
    for path in find_test_groups(&suite_root()) {
        let bytes =
            std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        let source = String::from_utf8_lossy(&bytes);
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("group")
            .to_string();
        let extraction = extract_keyword(&name, &source);
        out.push((
            name,
            count_assert_same(&source),
            extraction.rows(),
            extraction.dropped(),
        ));
    }
    out
}

/// The absolute committed literals: per-group row counts, and the four
/// totals. Measured at [`OOTEST_REVISION`].
///
/// Each row is `(group, assertSame calls, rows)`. `dropped` is not repeated
/// per group -- conservation makes it `calls - rows` exactly, and a second
/// column that is arithmetically forced by the other two would pin nothing
/// the first two do not already pin.
const PER_GROUP: &[(&str, usize, usize)] = &[
    ("ADDRESS", 222, 11),
    ("ASSIGNMENT", 265, 265),
    ("Assignments", 0, 0),
    ("CALL", 128, 50),
    ("DO", 510, 507),
    ("DoControlled", 0, 0),
    ("DoOther", 0, 0),
    ("DoOver", 4, 0),
    ("DoWith", 6, 0),
    ("EXPOSE", 24, 0),
    ("FORWARD", 0, 0),
    ("GUARD", 15, 0),
    ("IF", 159, 29),
    ("INTERPRET", 6, 3),
    ("ITERATE", 21, 20),
    ("LABEL", 0, 0),
    ("LEAVE", 17, 17),
    ("LOOP", 0, 0),
    ("LOSTDIGITS", 0, 0),
    ("LabelOption", 0, 0),
    ("LoopControlled", 0, 0),
    ("LoopOther", 0, 0),
    ("LoopOver", 3, 0),
    ("LoopWith", 6, 0),
    ("NOP", 3, 3),
    ("NUMERIC", 68, 63),
    ("PARSE", 792, 778),
    ("RAISE", 1, 0),
    ("REPLY", 11, 0),
    ("SAY", 3, 0),
    ("SELECT", 12, 8),
    ("SIGNAL", 27, 0),
    ("SelectCase", 5, 1),
    ("ShortCircuitAnd", 0, 0),
    ("TRACE", 49, 13),
    ("TRACE_TraceObject", 11, 0),
    ("USE", 20, 0),
    ("USELOCAL", 14, 0),
    ("VarRef", 39, 5),
];

/// `.testGroup` files scanned.
const TOTAL_FILES: usize = 39;
/// Exact-spelling `self~assertSame` occurrences across all of them.
const TOTAL_CALLS: usize = 2441;
/// Calls carried by a body this extractor turned into a runnable program.
const TOTAL_ROWS: usize = 1773;
/// Calls in a method this extractor could not turn into one.
const TOTAL_DROPPED: usize = 668;

/// **Criterion 2, the absolute literals.** Everything a percentage would
/// hide: the file count, the independently-counted call total, the row and
/// drop totals, and every group's own row count by name.
#[test]
fn base_keyword_yields_the_measured_counts() {
    let measured = measure();

    assert_eq!(
        measured.len(),
        TOTAL_FILES,
        "scanned {} .testGroup files, expected {TOTAL_FILES}{}",
        measured.len(),
        provenance()
    );

    let names: Vec<&str> = measured.iter().map(|(n, ..)| n.as_str()).collect();
    let expected_names: Vec<&str> = PER_GROUP.iter().map(|&(n, ..)| n).collect();
    assert_eq!(
        names,
        expected_names,
        "the set or order of scanned groups changed{}",
        provenance()
    );

    for (&(name, calls, rows), measured) in PER_GROUP.iter().zip(&measured) {
        assert_eq!(
            (measured.1, measured.2),
            (calls, rows),
            "{name}: measured {} calls / {} rows, committed {calls} / {rows}{}",
            measured.1,
            measured.2,
            provenance()
        );
    }

    let calls: usize = measured.iter().map(|m| m.1).sum();
    let rows: usize = measured.iter().map(|m| m.2).sum();
    let dropped: usize = measured.iter().map(|m| m.3).sum();
    assert_eq!(
        (calls, rows, dropped),
        (TOTAL_CALLS, TOTAL_ROWS, TOTAL_DROPPED),
        "totals moved: measured {calls} calls / {rows} rows / {dropped} dropped, committed \
         {TOTAL_CALLS} / {TOTAL_ROWS} / {TOTAL_DROPPED}{}",
        provenance()
    );
}

/// **Criterion 1, conservation**, stated over the wider population the brief
/// requires: `calls` counts **every** exact-spelling occurrence, including
/// the 403 that are not at the start of a line, the 3 inside comments and
/// the 2 in a method whose name does not begin `test`. A law stated only
/// over line-start occurrences would be satisfied by an extractor that
/// silently ignored all of those, which is precisely the blindness this
/// exists to catch: the `base/expressions` extractor pointed at this group
/// yields 54 rows from 2,561 prefix-matched calls and panics its own
/// conservation assertion on the second file.
#[test]
fn every_assert_same_is_a_row_or_an_accounted_for_drop() {
    for (name, calls, rows, dropped) in measure() {
        assert_eq!(
            rows + dropped,
            calls,
            "{name}: {rows} rows + {dropped} dropped != {calls} assertSame calls -- an \
             occurrence went neither into a body nor into a drop, which means the scanner \
             cannot see it at all{}",
            provenance()
        );
    }
}

/// **Criterion 3, the floor.** A real number, not `> 0`.
///
/// `> 0` would call a 54-row extraction a pass, and 54 is exactly what the
/// `base/expressions` extractor already yields on this group -- so a floor
/// that low would certify the very result this task exists to replace. The
/// number below is the measured ceiling's own scale rather than the measured
/// value: it is deliberately *not* [`TOTAL_ROWS`], because a floor equal to
/// the committed literal would be that literal a second time and would move
/// whenever it moved. This one survives an intentional re-measurement and
/// only fires if the extractor collapses.
#[test]
fn the_row_floor() {
    /// Below this, the body-shaped extractor has failed rather than drifted.
    const ROW_FLOOR: usize = 1500;

    let rows: usize = measure().iter().map(|m| m.2).sum();
    assert!(
        rows >= ROW_FLOOR,
        "only {rows} of {TOTAL_CALLS} assertSame calls were carried by an extracted body, \
         below the floor of {ROW_FLOOR}. A whole-body extractor that yields as few rows as \
         the prelude-shaped one it replaces (54) has not under-performed, it has failed{}",
        provenance()
    );
}

/// **Criterion 4, revision pinning.** The literals above are only meaningful
/// against a known corpus, and `ootest/` can move under `svn up` with
/// nothing in this repository changing. This asserts the committed file list
/// still matches the checkout, and every failure message in this file names
/// [`OOTEST_REVISION`] so a red run is diagnosable as a corpus move rather
/// than as a regression.
#[test]
fn the_committed_group_list_matches_the_checkout() {
    let committed = committed_group_list();
    let found: Vec<String> = find_test_groups(&suite_root())
        .iter()
        .map(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string()
        })
        .collect();

    assert_eq!(
        found,
        committed,
        "the .testGroup files under {} no longer match rust/corpus/keyword-groups.txt{}",
        suite_root().display(),
        provenance()
    );
    assert_eq!(committed.len(), TOTAL_FILES);
}

// ---------------------------------------------------------------------------
// The mechanics, on hand-written sources. Each pins one shape `base/keyword`
// contains and `base/expressions` does not, which is why the row-shaped
// extractor never had to model any of them.
// ---------------------------------------------------------------------------

fn one_body(source: &str) -> rexx_extract::keyword::KeywordExtraction {
    extract_keyword("G", source)
}

/// The blindness that made a third extractor necessary: `base/keyword`
/// writes 403 assertions somewhere other than the start of their line, as
/// multi-clause lines like this one. `extract_assertions` tests
/// `trimmed.starts_with("self~assertsame")` and sees none of them.
#[test]
fn an_assertion_after_a_semicolon_is_found_not_only_a_line_starting_one() {
    let out =
        one_body("::method test_1\n   id=0010; ABBREV     =0010; self~assertSame(abbrev, id)\n");
    assert_eq!(out.rows(), 1, "{:?}", out.blocked);
    assert!(
        out.bodies[0]
            .program
            .contains("id=0010; ABBREV     =0010; say ")
    );
}

/// The other half of that blindness: an assertion as a `THEN` target.
///
/// The replacement must be **one instruction**, and specifically not an
/// `IF` -- an `IF` here would capture a following `ELSE` that belongs to the
/// outer one, silently changing which branch the program takes. Asserting
/// the shape of the emitted text is what pins that choice; a test that only
/// counted rows would pass just as well with the broken rewrite.
#[test]
fn a_then_target_assertion_becomes_a_single_say_not_a_nested_if() {
    let out = one_body("::method test_1\n   If a=1 Then self~assertSame(1, b)\n   Else c=2\n");
    assert_eq!(out.rows(), 1, "{:?}", out.blocked);
    let program = &out.bodies[0].program;
    assert!(program.contains("If a=1 Then say '@@ASSERTSAME 1' ((1) == (b))"));
    assert!(
        !program.to_ascii_lowercase().contains("then if"),
        "the replacement introduced a nested IF, which would rebind the ELSE below it: \
         {program}"
    );
}

/// `self~assertSameList` shares the whole of `assertSame`'s spelling as a
/// prefix and is a different method. It must not be counted as an
/// `assertSame`, and it must not be rewritten as one -- it leaves its `~`
/// behind, which blocks its body like any other message send.
///
/// `base/expressions` contains zero of these, which is the only reason the
/// row-shaped extractor's prefix test has never mattered; `base/keyword`
/// contains 120.
#[test]
fn assert_same_list_is_neither_counted_nor_rewritten() {
    let source = "::method test_1\n   self~assertSameList(a, b)\n";
    assert_eq!(count_assert_same(source), 0);
    let out = one_body(source);
    assert_eq!(out.rows(), 0);
    assert_eq!(
        out.dropped(),
        0,
        "nothing to drop: there is no assertSame here"
    );

    // The sharper case: a real assertSame in the same body as one of these.
    // The body cannot run, and the real call is a drop rather than a row --
    // never a row whose program still contains the un-rewritable send.
    let mixed = "::method test_1\n   self~assertSameList(a, b)\n   self~assertSame(1, c)\n";
    assert_eq!(count_assert_same(mixed), 1);
    let out = one_body(mixed);
    assert_eq!((out.rows(), out.dropped()), (0, 1));
}

/// An `assertSame` written inside a comment is text, not a call.
/// `TRACE.testGroup` quotes two of them in a block comment showing that
/// method's own expected trace output. `count_assert_same` does not know
/// what a comment is -- deliberately, it is the independent denominator --
/// so conservation only holds if these get an explicit drop.
#[test]
fn an_assertion_inside_a_comment_is_accounted_as_a_drop_not_rewritten() {
    let source = "::method test_1\n   self~assertSame(1, a)\n   /* nnn *-* self~assertSame(\"?A\", trace())\n   */\n";
    assert_eq!(count_assert_same(source), 2);
    let out = one_body(source);
    assert_eq!((out.rows(), out.dropped()), (1, 1));
    assert!(out.blocked[0].reason.contains("inside a comment"));
    assert!(
        !out.bodies[0].program.contains("?A"),
        "the commented-out call was rewritten as if it were code"
    );
}

/// A `~` anywhere in a body stops the whole body, not just the clause it is
/// on -- the body runs as one program. This is the all-or-nothing rule that
/// distinguishes this extractor from the row-shaped one, which keeps rows
/// from before a blocking statement.
#[test]
fn a_message_send_anywhere_blocks_the_whole_body_including_earlier_assertions() {
    let out = one_body(
        "::method test_1\n   self~assertSame(1, a)\n   b = c~copies(2)\n   self~assertSame(2, b)\n",
    );
    assert_eq!((out.rows(), out.dropped()), (0, 2));
    assert!(out.blocked[0].reason.contains("c~copies(2)"));
}

/// The operands are inspected too: a send hidden inside an assertion's own
/// argument blocks exactly like a bare one. Checked after rewriting rather
/// than before, which is what makes this fall out instead of needing its own
/// rule.
#[test]
fn a_message_send_inside_an_assertions_own_argument_blocks_it() {
    let out = one_body("::method test_1\n   self~assertSame(1, x~y)\n");
    assert_eq!((out.rows(), out.dropped()), (0, 1));
}

/// ...and the adjacent success, so the rule above is pinned to "a message
/// send" rather than to "the character `~`": a `~` inside a string literal
/// is data, and must not block. `TRACE`'s expected-output strings contain
/// them.
#[test]
fn a_tilde_inside_a_string_literal_does_not_block() {
    let out = one_body("::method test_1\n   s = 'a~b'\n   self~assertSame('a~b', s)\n");
    assert_eq!(out.rows(), 1, "{:?}", out.blocked);
}

/// `assertSame(expected, actual, msg = "")` -- the third argument is a
/// failure-report message and is read and discarded, never compared. Two
/// calls in this group pass one.
#[test]
fn the_optional_third_message_argument_is_discarded_not_compared() {
    let out = one_body("::method test_1\n   self~assertSame(a, b, \"a should equal b\")\n");
    assert_eq!(out.rows(), 1, "{:?}", out.blocked);
    let program = &out.bodies[0].program;
    assert!(program.contains("((a) == (b))"));
    assert!(
        !program.contains("should equal"),
        "the message argument reached the comparison: {program}"
    );
}

/// A continued line is one clause spread over two, so an assertion on
/// either side of the join is not a clause of its own and a `SAY` cannot
/// stand in its place. Both `,` and `-` continue in ooRexx.
#[test]
fn an_assertion_on_a_continued_line_blocks_from_either_side() {
    let before = one_body("::method test_1\n   x = 1 -\n   self~assertSame(1, x)\n");
    assert_eq!((before.rows(), before.dropped()), (0, 1));
    assert!(before.blocked[0].reason.contains("continued line"));

    let after = one_body("::method test_1\n   self~assertSame(1, x) ,\n   y\n");
    assert_eq!((after.rows(), after.dropped()), (0, 1));
}

/// A call in a method whose name does not begin `test` is never yielded by
/// `extract` at all, so nothing downstream can see it -- `GUARD.testGroup`'s
/// `waiter_multiple` has two. Conservation is stated over the file, so these
/// need their own drop or they would vanish from both columns, which is
/// exactly how an unseen population hides.
#[test]
fn calls_outside_a_test_prefixed_method_are_accounted_as_drops() {
    let source = "::method test_1\n   self~assertSame(1, a)\n\n::method waiter\n   self~assertSame(0, var)\n";
    assert_eq!(count_assert_same(source), 2);
    let out = one_body(source);
    assert_eq!((out.rows(), out.dropped()), (1, 1));
    assert!(out.blocked[0].method.contains("outside any test-prefixed"));
}

/// An assertion used as an operand rather than as a clause cannot become a
/// `SAY`. Nothing in `base/keyword` writes one -- all 2,441 calls sit at a
/// clause boundary, measured -- so this is a forward guard against the
/// corpus growing one, not a case seen today. Without it, such a call would
/// be spliced into the middle of someone else's clause and the body would
/// become a syntactically invalid program that fails for a reason no report
/// could explain.
#[test]
fn an_assertion_used_as_an_operand_blocks_rather_than_producing_invalid_rexx() {
    let out = one_body("::method test_1\n   x = self~assertSame(1, a)\n");
    assert_eq!((out.rows(), out.dropped()), (0, 1));
    assert!(out.blocked[0].reason.contains("not a clause of its own"));
}
