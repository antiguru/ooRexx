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
use rexx_extract::keyword::{DropReason, count_assert_same, extract_keyword};
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
/// that low would certify the very result this task exists to replace.
///
/// **Derived from what is actually achieved, and set just under it.** The
/// extractor carries [`TOTAL_ROWS`] = 1,773 of 2,441 calls (72.6%); the
/// floor is 1,750, about 1.3% below. It is deliberately *not* `TOTAL_ROWS`
/// itself, which would be that literal a second time and would move
/// whenever it moved -- the point of a separate floor is that it survives a
/// deliberate re-measurement of the exact counts while still failing on any
/// real collapse. A gap this narrow means the two tests fail together for a
/// large regression and the absolute one fails alone for a small
/// deliberate change, which is the intended division of labour.
#[test]
fn the_row_floor() {
    /// Below this, the body-shaped extractor has failed rather than drifted.
    const ROW_FLOOR: usize = 1750;

    let rows: usize = measure().iter().map(|m| m.2).sum();
    assert!(
        rows >= ROW_FLOOR,
        "only {rows} of {TOTAL_CALLS} assertSame calls were carried by an extracted body, \
         below the floor of {ROW_FLOOR}. A whole-body extractor that yields as few rows as \
         the prelude-shaped one it replaces (54) has not under-performed, it has failed{}",
        provenance()
    );
}

/// **The 668 calls outside the population, accounted for by reason.**
///
/// `calls - rows` as a single number says only that something was lost.
/// This pins what kind and how much of each, so a category that starts
/// growing is visible on its own rather than absorbed into a total that was
/// always going to be large. Every variant is listed, including the ones at
/// zero: a category pinned at zero fails the first time the corpus grows
/// one.
///
/// Two rows here are load-bearing beyond bookkeeping.
///
/// `OtherAssertion` (169) is exactly the price of this extractor's
/// population choice -- the calls a wider rule would admit by rewriting
/// other `self~assert*` spellings to `NOP`, which would report those bodies
/// as passing after deleting the checks they were written to make. Having
/// it as a committed number rather than a claim is the point.
///
/// `AssertSameList` reads zero, and **not** because no body mixes the two
/// spellings -- five do. See [`DropReason::AssertSameList`]'s own doc for
/// which, and why `MessageSend` claims them first.
#[test]
fn the_drop_reasons_account_for_every_call_outside_the_population() {
    /// `(reason, methods, calls)`, measured at [`OOTEST_REVISION`].
    const BY_REASON: &[(DropReason, usize, usize)] = &[
        (DropReason::OutsideTestMethod, 1, 2),
        (DropReason::InsideComment, 3, 3),
        (DropReason::ContinuedLine, 2, 3),
        (DropReason::AssertSameList, 0, 0),
        (DropReason::OtherAssertion, 138, 169),
        (DropReason::MessageSend, 96, 491),
        (DropReason::UnparsedCallShape, 0, 0),
        (DropReason::NotAClause, 0, 0),
    ];

    let mut methods: std::collections::BTreeMap<DropReason, usize> = Default::default();
    let mut calls: std::collections::BTreeMap<DropReason, usize> = Default::default();
    for path in find_test_groups(&suite_root()) {
        let bytes =
            std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        let source = String::from_utf8_lossy(&bytes);
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("group");
        for blocked in extract_keyword(name, &source).blocked {
            *methods.entry(blocked.reason).or_default() += 1;
            *calls.entry(blocked.reason).or_default() += blocked.dropped;
        }
    }

    // Every variant is covered by the committed table, so a new one cannot
    // be added to the enum and left unmeasured here.
    let listed: Vec<DropReason> = BY_REASON.iter().map(|&(r, ..)| r).collect();
    assert_eq!(listed, DropReason::ALL.to_vec());

    for &(reason, want_methods, want_calls) in BY_REASON {
        let got = (
            methods.get(&reason).copied().unwrap_or(0),
            calls.get(&reason).copied().unwrap_or(0),
        );
        assert_eq!(
            got,
            (want_methods, want_calls),
            "{}: measured {} entries / {} calls, committed {want_methods} / {want_calls}{}",
            reason.label(),
            got.0,
            got.1,
            provenance()
        );
    }

    // The accounting closes: the breakdown is the whole of `calls - rows`,
    // not a sample of it.
    let total: usize = BY_REASON.iter().map(|&(.., c)| c).sum();
    assert_eq!(total, TOTAL_DROPPED);
    assert_eq!(TOTAL_ROWS + TOTAL_DROPPED, TOTAL_CALLS);
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
    // And it lands in its own category rather than the general message-send
    // bucket. This matters because that category reads zero against the
    // corpus: a category at zero has to be demonstrably *reachable*, or it
    // is indistinguishable from one that can never fire, and pinning it at
    // zero would then prove nothing.
    assert_eq!(out.blocked[0].reason, DropReason::AssertSameList);
}

/// A body blocked only by *other* `self~assert*` spellings is its own
/// category, not a message send. That column is the measured price of this
/// extractor's population choice, so it has to mean exactly "modelling more
/// assertions would unblock this body" and nothing looser.
#[test]
fn a_body_blocked_only_by_other_assertion_spellings_is_its_own_category() {
    let out = one_body("::method test_1\n   self~assertTrue(a)\n   self~assertSame(1, c)\n");
    assert_eq!((out.rows(), out.dropped()), (0, 1));
    assert_eq!(out.blocked[0].reason, DropReason::OtherAssertion);

    // The adjacent case that pins it to the whole body rather than to the
    // first offending line: an `assertTrue` first, a real message send
    // later. Modelling assertions would not unblock this one, so it must
    // read `MessageSend` even though the line named in `detail` is the
    // assertion.
    let later = one_body(
        "::method test_1\n   self~assertTrue(a)\n   b = c~copies(2)\n   self~assertSame(1, b)\n",
    );
    assert_eq!(later.blocked[0].reason, DropReason::MessageSend);
    assert!(
        later.blocked[0].detail.contains("assertTrue"),
        "detail should still name the first offending line: {:?}",
        later.blocked[0].detail
    );
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
    assert_eq!(out.blocked[0].reason, DropReason::InsideComment);

    // The comment itself survives into the program verbatim -- a Rexx
    // comment ends a token without inserting a blank, so a rewriter that
    // deleted it would join two tokens and one that replaced it with a
    // space would concatenate them with a blank. What must not survive is
    // any *rewrite* of the call inside it.
    let program = &out.bodies[0].program;
    assert!(program.contains("/* nnn *-* self~assertSame(\"?A\", trace())"));
    assert_eq!(
        program
            .matches(rexx_extract::keyword::ASSERTION_MARKER)
            .count(),
        1,
        "the commented-out call was rewritten as if it were code: {program}"
    );
}

/// The measurement behind keeping comments rather than stripping them, as a
/// test rather than only as prose in [`rexx_extract::keyword`]'s own doc.
///
/// `ITERATE.testGroup`'s `test_11` and `LEAVE.testGroup`'s `test_10` both
/// write an expected value as `(11/**/ 1/**irrelevant**/05  10/*...*/)`,
/// relying on the comment to end a token *without* contributing a blank.
/// Measured on the oracle: `say '['1/**/05']'` prints `[105]` and
/// `say '['1 /**/ 05']'` prints `[1 05]`. So the two rewrites below must
/// differ, and the first must keep the operand's bytes exactly as written --
/// an earlier draft replaced each comment with a space, and both of those
/// two bodies then disagreed with the oracle.
#[test]
fn a_comment_inside_an_operand_is_kept_verbatim_not_turned_into_a_blank() {
    let abutted = one_body("::method test_1\n   self~assertSame(x, (1/**/05))\n");
    assert_eq!(abutted.rows(), 1, "{:?}", abutted.blocked);
    assert!(
        abutted.bodies[0].program.contains("((1/**/05))"),
        "{}",
        abutted.bodies[0].program
    );

    let spaced = one_body("::method test_1\n   self~assertSame(x, (1 /**/ 05))\n");
    assert!(spaced.bodies[0].program.contains("((1 /**/ 05))"));
}

/// A comment cannot smuggle structure into a call: a comma or a paren
/// inside one belongs to the comment, not to the argument list. The
/// structural scan runs on the blanked view for exactly this reason, while
/// the emitted text comes from the original.
#[test]
fn a_comma_inside_a_comment_is_not_an_argument_separator() {
    let out = one_body("::method test_1\n   self~assertSame(a /* , ) */, b)\n");
    assert_eq!(out.rows(), 1, "{:?}", out.blocked);
    assert!(
        out.bodies[0].program.contains("((a /* , ) */) == (b))"),
        "{}",
        out.bodies[0].program
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
    assert_eq!(out.blocked[0].reason, DropReason::MessageSend);
    assert!(out.blocked[0].detail.contains("c~copies(2)"));
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
    assert_eq!(before.blocked[0].reason, DropReason::ContinuedLine);

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
    assert_eq!(out.blocked[0].reason, DropReason::NotAClause);
}

/// Conservation has exactly one hole, and this pins that it is **loud**.
///
/// `count_assert_same` counts a substring and does not know what a string
/// literal is; `rewrite_line` tracks string state and skips one. No
/// [`DropReason`] covers the difference, so such an occurrence makes
/// `rows + dropped` come out short. That is a red conservation test naming
/// the group, not a wrong number reported quietly -- which is the behaviour
/// to want from a hole -- but it is a hole, and
/// `count_assert_same`'s own doc now says so rather than claiming the
/// accounting is total.
///
/// No such literal exists in `base/keyword` at r13178, which is why
/// `every_assert_same_is_a_row_or_an_accounted_for_drop` passes over the
/// real corpus. This constructs one so the property is checked rather than
/// merely believed.
#[test]
fn a_call_inside_a_string_literal_breaks_conservation_loudly() {
    let source = "::method test_1\n   s = 'self~assertSame(1, 2)'\n   self~assertSame(1, 1)\n";
    assert_eq!(
        count_assert_same(source),
        2,
        "the counter counts substrings, including the one in the literal"
    );
    let out = one_body(source);
    assert_eq!(
        (out.rows(), out.dropped()),
        (1, 0),
        "the scanner sees only the real call, and has no drop reason for the literal"
    );
    assert_ne!(
        out.rows() + out.dropped(),
        count_assert_same(source),
        "conservation must FAIL here -- if this ever holds, either the counter learned about \
         strings or a DropReason was added, and this test should be replaced by the ordinary \
         accounting rather than deleted"
    );

    // The adjacent success: the same body without the literal conserves.
    let clean = "::method test_1\n   self~assertSame(1, 1)\n";
    let out = one_body(clean);
    assert_eq!(out.rows() + out.dropped(), count_assert_same(clean));
}

/// [`DropReason::UnparsedCallShape`] is pinned at zero against the corpus,
/// and a category pinned at zero proves nothing unless it can fire. Three
/// shapes reach it.
///
/// This is the same requirement `assert_same_list_is_neither_counted_nor_
/// rewritten` meets for the other zero-valued category. `NotAClause` has
/// its own witness in
/// `an_assertion_used_as_an_operand_blocks_rather_than_producing_invalid_rexx`.
#[test]
fn the_unparsed_call_shape_category_is_reachable() {
    for source in [
        "::method test_1\n   self~assertSame(a)\n", // too few arguments
        "::method test_1\n   self~assertSame(a,b,c,d)\n", // too many
        "::method test_1\n   self~assertSame\n",    // no argument list at all
    ] {
        let out = one_body(source);
        assert_eq!((out.rows(), out.dropped()), (0, 1), "{source:?}");
        assert_eq!(
            out.blocked[0].reason,
            DropReason::UnparsedCallShape,
            "{source:?}"
        );
    }

    // The adjacent success, so this is pinned to the argument count and not
    // to "anything unusual blocks": two and three arguments both parse.
    for source in [
        "::method test_1\n   self~assertSame(a,b)\n",
        "::method test_1\n   self~assertSame(a,b,\"msg\")\n",
    ] {
        assert_eq!(one_body(source).rows(), 1, "{source:?}");
    }
}

/// The population choice's price, re-derived by a **different rule** and
/// checked against the committed [`DropReason::OtherAssertion`] column.
///
/// The extractor decides that column with a per-body cascade over blanked
/// lines (`classify_sends`). This re-derives the same quantity the way the
/// pre-implementation estimate did: split methods with `extract`, strip
/// comments with a local stripper, and ask whether deleting every
/// `self~assert*`/`self~expect*` token leaves a `~` behind. Two independent
/// routes to one number is what makes it a cross-check rather than the same
/// rule reported twice -- which is precisely the objection this answers.
///
/// The wider population is the set of bodies whose only sends are ooTest
/// assertions of any spelling; the narrower one, which this extractor takes,
/// requires the only send to be `assertSame` exactly. The difference is what
/// admitting the wider rule would buy, and it must equal the committed
/// column.
#[test]
fn the_population_choices_price_is_reproduced_by_an_independent_rule() {
    fn strip(body: &str) -> String {
        let (mut out, mut depth, mut in_str) = (String::new(), 0usize, None::<char>);
        let mut chars = body.chars().peekable();
        while let Some(c) = chars.next() {
            if depth > 0 {
                match c {
                    '*' if chars.peek() == Some(&'/') => {
                        chars.next();
                        depth -= 1;
                    }
                    '/' if chars.peek() == Some(&'*') => {
                        chars.next();
                        depth += 1;
                    }
                    '\n' => out.push('\n'),
                    _ => {}
                }
                continue;
            }
            if let Some(q) = in_str {
                out.push(c);
                if c == q {
                    in_str = None;
                }
                continue;
            }
            match c {
                '\'' | '"' => {
                    in_str = Some(c);
                    out.push(c);
                }
                '/' if chars.peek() == Some(&'*') => {
                    chars.next();
                    depth = 1;
                }
                '-' if chars.peek() == Some(&'-') => {
                    while chars.peek().is_some_and(|&n| n != '\n') {
                        chars.next();
                    }
                }
                _ => out.push(c),
            }
        }
        out
    }

    let (mut narrow_m, mut narrow_c, mut wide_m, mut wide_c) = (0usize, 0usize, 0usize, 0usize);
    for path in find_test_groups(&suite_root()) {
        let bytes =
            std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        let source = String::from_utf8_lossy(&bytes);
        for method in rexx_extract::extract(&source) {
            let body = strip(&method.body);
            let calls = count_assert_same(&body);
            if calls == 0 {
                continue;
            }
            let lower = body.to_ascii_lowercase();
            // Narrow: nothing but `self~assertSame` sends anything. The
            // `\u{1}` stand-in keeps `assertSameList` from being consumed by
            // its own prefix, which would wrongly admit the body.
            let narrow = lower
                .replace("self~assertsamelist", "\u{1}")
                .split("self~assertsame")
                .collect::<Vec<_>>()
                .join(" ");
            if !narrow.contains('~') && !narrow.contains('\u{1}') {
                narrow_m += 1;
                narrow_c += calls;
            }
            // Wide: nothing but ooTest assertions of any spelling.
            let mut wide = lower.clone();
            for token in ["self~assert", "self~expect"] {
                wide = wide.split(token).collect::<Vec<_>>().join(" ");
            }
            if !wide.contains('~') {
                wide_m += 1;
                wide_c += calls;
            }
        }
    }

    assert_eq!(
        narrow_c,
        TOTAL_ROWS,
        "the independent narrow rule should reproduce the extractor's own row total{}",
        provenance()
    );

    let committed = PER_GROUP.len();
    assert!(committed > 0);
    let gain_methods = wide_m - narrow_m;
    let gain_calls = wide_c - narrow_c;
    assert_eq!(
        (gain_methods, gain_calls),
        (138, 169),
        "the wider population's gain over the narrower one moved{}",
        provenance()
    );

    // And that gain is exactly what the extractor's own cascade attributes
    // to OtherAssertion -- the two rules agreeing is the whole point.
    let mut other = 0usize;
    for path in find_test_groups(&suite_root()) {
        let bytes = std::fs::read(&path).unwrap();
        let source = String::from_utf8_lossy(&bytes);
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("group");
        for blocked in extract_keyword(name, &source).blocked {
            if blocked.reason == DropReason::OtherAssertion {
                other += blocked.dropped;
            }
        }
    }
    assert_eq!(
        other,
        gain_calls,
        "the extractor's OtherAssertion column and the independent rule disagree, so one of \
         them is wrong{}",
        provenance()
    );
}
