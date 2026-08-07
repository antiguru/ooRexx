//! The pins on the `base/bif` extractor: conservation, absolute committed
//! literals, a real floor, and the ooTest revision they were measured at.
//!
//! # Why conservation alone would be a tautology, and what makes it one here
//!
//! `rows + dropped == calls` holds trivially at `0 + 0 == 0` (nothing scanned)
//! and at `0 + N == N` (everything dropped). What makes it non-vacuous is that
//! `calls` is counted **independently of the extractor**, by
//! [`rexx_extract::keyword::count_assert_same`], which does not parse Rexx and
//! only counts a substring -- so a scanner that silently sees nothing cannot
//! satisfy it, and neither can one that drops everything, because
//! [`the_row_floor`] states a real lower bound on the rows side.
//!
//! # `raises` is inside `dropped`, not beside it
//!
//! A [`rexx_extract::bif::RaiseRow`] is built from a real `self~assertSame`
//! call, and that call is *also* counted dropped
//! ([`rexx_extract::bif::DropReason::ExpectedRaise`]). So the law is
//! `rows + dropped == calls` with `raises` outside it, and the raise rows are a
//! second, smaller population carved out of the dropped side rather than a
//! third term. Stating it the other way would double-count them.
//!
//! # The denominator is the *exact* `assertSame` spelling
//!
//! `base/bif` holds 6,293 exact `self~assertSame` occurrences and 5
//! `self~assertSameList`, a different method that a prefix test would swallow.
//! The 424-call `assertTrue`/`assertEquals`/`assertFalse` tail is a different
//! assertion again and nothing here claims anything about it.

use rexx_extract::bif::{DropReason, extract_bif};
use rexx_extract::find_test_groups;
use rexx_extract::keyword::count_assert_same;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The ooTest revision every absolute literal below was measured at. Named in
/// each failure message so a red count is diagnosable as `svn up` rather than
/// as a regression in this repository.
///
/// `ootest/` is **not** checked-in test data: it is git-ignored, has zero
/// tracked files, and exists only as an SVN working copy of
/// `svn.code.sf.net/p/oorexx/code-0/test/trunk`. Read it back with
/// `svn info ootest`.
const OOTEST_REVISION: &str = "r13178";

/// What a red absolute-literal test most likely means, appended to every such
/// failure.
fn provenance() -> String {
    format!(
        "\n\nThese literals were measured against ooTest {OOTEST_REVISION}. `ootest/` is an SVN \
         working copy, not checked-in test data, so the first thing to check is `svn info \
         ootest`: if it no longer reads {OOTEST_REVISION}, the corpus moved underneath this test \
         and the fix is to re-measure and re-commit these numbers, not to relax them. If the \
         revision still matches, this is a real change in what the extractor accepts."
    )
}

fn suite_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../ootest/ooRexx/base/bif")
}

/// Every group's `(name, calls, rows, raises)`, in `find_test_groups` order.
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
        let extraction = extract_bif(&name, &source);
        out.push((
            name,
            count_assert_same(&source),
            extraction.rows.len(),
            extraction.raises.len(),
        ));
    }
    out
}

/// The absolute committed literals, measured at [`OOTEST_REVISION`]. Each row
/// is `(group, assertSame calls, value rows, raise rows)`.
///
/// `dropped` is not a fourth column: conservation makes it `calls - rows`
/// exactly, so a column arithmetically forced by the others would pin nothing
/// they do not already pin.
const PER_GROUP: &[(&str, usize, usize, usize)] = &[
    ("ABBREV", 130, 109, 4),
    ("ABS", 49, 44, 5),
    ("ADDRESS", 2, 0, 0),
    ("ARG", 64, 0, 0),
    ("B2X", 41, 24, 13),
    ("BEEP", 3, 3, 0),
    ("BITAND", 101, 83, 18),
    ("BITOR", 124, 105, 19),
    ("BITXOR", 136, 111, 25),
    ("C2D", 127, 69, 33),
    ("C2X", 34, 32, 1),
    ("CENTER", 97, 74, 22),
    ("CENTRE", 97, 74, 22),
    ("CHANGESTR", 8, 8, 0),
    ("CHARIN", 16, 3, 0),
    ("CHAROUT", 184, 1, 1),
    ("CHARS", 26, 0, 1),
    ("COMPARE", 132, 119, 12),
    ("CONDITION", 47, 0, 0),
    ("COPIES", 479, 447, 0),
    ("COUNTSTR", 4, 4, 0),
    ("D2C", 50, 50, 0),
    ("D2X", 57, 57, 0),
    ("DATATYPE", 9, 6, 0),
    ("DATE", 763, 716, 0),
    ("DELSTR", 87, 79, 0),
    ("DELWORD", 69, 65, 0),
    ("DIGITS", 20, 5, 0),
    ("ERRORTEXT", 101, 101, 0),
    ("FILESPEC", 76, 0, 0),
    ("FORM", 3, 2, 0),
    ("FORMAT", 721, 710, 10),
    ("FUZZ", 0, 0, 0),
    ("GC", 0, 0, 0),
    ("INSERT", 124, 80, 0),
    ("LASTPOS", 184, 184, 0),
    ("LEFT", 32, 32, 0),
    ("LENGTH", 32, 32, 0),
    ("LINEIN", 9, 0, 0),
    ("LINEOUT", 27, 0, 0),
    ("LINES", 53, 6, 0),
    ("LOWER", 5, 5, 0),
    ("MAX", 16, 16, 0),
    ("MIN", 12, 12, 0),
    ("OVERLAY", 62, 62, 0),
    ("POS", 93, 69, 0),
    ("QUALIFY", 21, 2, 0),
    ("QUEUED", 5, 1, 0),
    ("RANDOM", 3, 0, 0),
    ("REVERSE", 27, 26, 0),
    ("RIGHT", 36, 36, 0),
    ("RXQUEUE", 43, 5, 0),
    ("SIGN", 36, 36, 0),
    ("SOURCELINE", 6, 0, 0),
    ("SPACE", 77, 69, 0),
    ("STREAM", 137, 32, 0),
    ("STRIP", 110, 83, 0),
    ("SUBSTR", 64, 64, 0),
    ("SUBWORD", 130, 122, 0),
    ("SYMBOL", 28, 28, 0),
    ("TIME", 217, 122, 0),
    ("TRANSLATE", 27, 27, 0),
    ("TRUNC", 39, 39, 0),
    ("UPPER", 5, 5, 0),
    ("VALUE", 67, 40, 0),
    ("VAR", 0, 0, 0),
    ("VERIFY", 86, 84, 0),
    ("WORD", 60, 56, 0),
    ("WORDINDEX", 59, 55, 0),
    ("WORDLENGTH", 59, 55, 0),
    ("WORDPOS", 177, 177, 0),
    ("WORDS", 39, 39, 0),
    ("X2B", 49, 49, 0),
    ("X2C", 31, 31, 0),
    ("X2D", 66, 66, 0),
    ("XRANGE", 83, 51, 0),
];

/// Conservation, per group and in total. The property the whole extractor is
/// built around: every `self~assertSame` occurrence is either a row or is
/// accounted for by a named [`DropReason`], with nothing falling between.
#[test]
fn every_call_is_a_row_or_a_counted_drop() {
    for path in find_test_groups(&suite_root()) {
        let bytes =
            std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        let source = String::from_utf8_lossy(&bytes);
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("group");
        let extraction = extract_bif(name, &source);
        let calls = count_assert_same(&source);
        assert_eq!(
            extraction.rows.len() + extraction.dropped(),
            calls,
            "{name}: {} rows + {} dropped != {calls} assertSame calls -- some occurrence is \
             neither extracted nor accounted for",
            extraction.rows.len(),
            extraction.dropped()
        );
    }
}

/// The absolute per-group literals, and the four totals.
#[test]
fn base_bif_yields_the_measured_counts() {
    let measured = measure();
    let committed: Vec<(String, usize, usize, usize)> = PER_GROUP
        .iter()
        .map(|&(name, calls, rows, raises)| (name.to_string(), calls, rows, raises))
        .collect();
    assert_eq!(
        measured,
        committed,
        "the per-group extraction no longer matches the committed table.{}",
        provenance()
    );

    let calls: usize = measured.iter().map(|r| r.1).sum();
    let rows: usize = measured.iter().map(|r| r.2).sum();
    let raises: usize = measured.iter().map(|r| r.3).sum();
    assert_eq!(
        (calls, rows, raises, calls - rows),
        (6293, 4999, 186, 1294),
        "the totals moved.{}",
        provenance()
    );
}

/// Every [`DropReason`]'s own count, including the ones standing at zero.
///
/// A category pinned at zero fails loudly the first time the corpus grows one,
/// which is exactly when a reader needs to know -- and it is the only thing
/// that tells "this shape does not occur" apart from "this shape is not
/// detected".
#[test]
fn the_drop_reasons_account_for_every_call_outside_the_population() {
    const EXPECTED: &[(DropReason, usize, usize)] = &[
        (DropReason::OutsideTestMethod, 2, 2),
        (DropReason::RoutineBody, 1, 5),
        (DropReason::InsideComment, 3, 25),
        (DropReason::ContinuedLine, 0, 0),
        (DropReason::NotAClause, 0, 0),
        (DropReason::UnparsedCallShape, 0, 0),
        (DropReason::ExpectedRaise, 189, 195),
        (DropReason::MessageSend, 101, 418),
        (DropReason::UnsupportedStatement, 112, 467),
        (DropReason::NumericNotConstant, 9, 14),
        (DropReason::UnresolvedFixture, 121, 121),
        (DropReason::InFileRoutine, 1, 1),
        (DropReason::NoValueUnderOptions, 10, 28),
        (DropReason::OtherExpectation, 0, 0),
        (DropReason::NonUtf8Source, 6, 11),
        (DropReason::SideEffectingAssertion, 1, 6),
        (DropReason::ClockDependent, 1, 1),
    ];
    assert_eq!(
        EXPECTED.len(),
        DropReason::ALL.len(),
        "a DropReason variant was added or removed without this table moving with it"
    );

    let mut measured: BTreeMap<DropReason, (usize, usize)> = BTreeMap::new();
    for reason in DropReason::ALL {
        measured.insert(*reason, (0, 0));
    }
    for path in find_test_groups(&suite_root()) {
        let bytes =
            std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        let source = String::from_utf8_lossy(&bytes);
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("group");
        for blocked in extract_bif(name, &source).blocked {
            let entry = measured.entry(blocked.reason).or_default();
            entry.0 += 1;
            entry.1 += blocked.dropped;
        }
    }

    let committed: BTreeMap<DropReason, (usize, usize)> = EXPECTED
        .iter()
        .map(|&(reason, bodies, calls)| (reason, (bodies, calls)))
        .collect();
    assert_eq!(
        measured,
        committed,
        "the per-reason drop table moved.{}",
        provenance()
    );
    let dropped: usize = measured.values().map(|v| v.1).sum();
    assert_eq!(dropped, 1294, "the drop total moved.{}", provenance());
}

/// A real lower bound on the rows side, so that conservation cannot be
/// satisfied by an extractor that drops everything.
///
/// Deliberately far below the committed figure: this is the floor, and
/// [`base_bif_yields_the_measured_counts`] is what pins the exact number. A
/// floor that tracked the exact count would go red for every improvement, which
/// is the opposite of what a floor is for.
#[test]
fn the_row_floor() {
    const FLOOR: usize = 4000;
    let rows: usize = measure().iter().map(|r| r.2).sum();
    assert!(
        rows >= FLOOR,
        "only {rows} rows extracted, below the {FLOOR} floor -- conservation would still hold \
         with every call dropped, and this is what makes it mean something.{}",
        provenance()
    );
}

/// A `self~expectSyntax` suppresses every `assertSame` at or after it, and
/// **only** those.
///
/// The refusal and its adjacent success in one test, because the rule is about
/// ordering and a test of the refusal alone would also pass for an extractor
/// that suppressed the whole body unconditionally. `LINEOUT.testGroup:207` is
/// the corpus's one real instance of the second ordering.
#[test]
fn expect_syntax_suppresses_only_what_follows_it() {
    let after =
        "::method 'test_after'\n   self~expectSyntax(40.5)\n   self~assertSame(C2D(,-1), '-1')\n";
    let extraction = extract_bif("SYNTHETIC", after);
    assert!(
        extraction.rows.is_empty(),
        "an assertSame after an expectSyntax became a value row, which states a return value \
         where the body requires a raise: {:?}",
        extraction.rows
    );
    assert_eq!(extraction.raises.len(), 1);
    assert_eq!(extraction.raises[0].expect.major, 40);
    assert_eq!(extraction.raises[0].expect.sub, 5);
    assert_eq!(
        extraction.raises[0].operands,
        vec!["C2D(,-1)".to_string(), "'-1'".to_string()],
        "a raise row must carry both arguments in evaluation order -- base/bif writes the raiser \
         in either position"
    );
    assert_eq!(
        extraction.dropped(),
        1,
        "the suppressed call must still be counted"
    );

    // The adjacent success: the same two clauses the other way round is an
    // ordinary value row, and the expectation applies to nothing.
    let before = "::method 'test_before'\n   self~assertSame(1, 1)\n   self~expectSyntax(40.5)\n";
    let extraction = extract_bif("SYNTHETIC", before);
    assert_eq!(extraction.rows.len(), 1);
    assert!(extraction.raises.is_empty());
    assert_eq!(extraction.dropped(), 0);
}

/// A body is segmented at the next `::` directive of any kind, not at the next
/// `::method`.
///
/// The corpus case is `CONDITION.testGroup`'s three-line
/// `test_novalue_override`, which is followed by three `::routine`s. Segmenting
/// at `^::method` swallows them into it, which both invents assertions in the
/// method and hides the ones that are genuinely a trap handler's.
#[test]
fn a_routine_after_a_method_is_not_part_of_it() {
    let source = "::method 'test_short'\n   self~assertSame(1, 1)\n\n\
                  ::routine handler\n   self~assertSame(2, 2)\n";
    let extraction = extract_bif("SYNTHETIC", source);
    assert_eq!(extraction.rows.len(), 1, "{:?}", extraction.rows);
    assert_eq!(
        extraction
            .blocked
            .iter()
            .filter(|b| b.reason == DropReason::RoutineBody)
            .map(|b| b.dropped)
            .sum::<usize>(),
        1,
        "the ::routine's own assertion was not attributed to the routine"
    );
}

/// Under `::options novalue` an unassigned symbol raises, so a row reading one
/// is dropped; without it the same row resolves.
///
/// Both directions in one test, because the whole hazard is that the body text
/// is identical and only a file-level directive decides what it means.
/// `::options all` counts as `novalue` -- verified on the oracle, where a
/// program whose only directive is `::options all syntax` fails `say abc` with
/// `Error 98.986`.
#[test]
fn options_novalue_inverts_whether_an_unassigned_symbol_is_resolvable() {
    const BODY: &str = "::method 'test_nv'\n   self~assertSame('NV', nv)\n";
    let open = extract_bif("SYNTHETIC", BODY);
    assert_eq!(open.rows.len(), 1, "{:?}", open.blocked);

    for directive in ["::options novalue error", "::options all syntax"] {
        let source = format!("{directive}\n{BODY}");
        let closed = extract_bif("SYNTHETIC", &source);
        assert!(
            closed.rows.is_empty(),
            "{directive}: the symbol-equals-own-name resolution was applied in a file where \
             reading an unassigned symbol raises: {:?}",
            closed.rows
        );
        assert_eq!(
            closed
                .blocked
                .iter()
                .filter(|b| b.reason == DropReason::NoValueUnderOptions)
                .map(|b| b.dropped)
                .sum::<usize>(),
            1
        );
    }
}

/// A `.local` fixture is substituted stem-aware, and a name the file sets twice
/// is not substituted at all.
///
/// `WORD.testGroup`'s shape: `v8. = .v8` assigns a stem default and the body
/// then reads `v8.10`, a compound whose tail was never assigned. An exact-name
/// match on the assigned set calls `v8.10` unassigned and, in a `novalue` file,
/// drops a row that is fine.
#[test]
fn a_local_fixture_is_substituted_and_an_ambiguous_one_is_not() {
    let source = "::method 'test000'\n   .local~v8 = 'one two three four'\n\n\
                  ::method 'test_read'\n   v8. = .v8\n   self~assertSame('four', word((v8.10),4))\n";
    let extraction = extract_bif("SYNTHETIC", source);
    assert_eq!(extraction.rows.len(), 1, "{:?}", extraction.blocked);
    assert_eq!(
        extraction.rows[0].prelude,
        vec!["v8. = ('one two three four')".to_string()],
        "the fixture was not substituted, so the row asserts something about the literal text \
         `.V8` rather than about the value the file set"
    );

    // The other direction: a second, different assignment to the same name makes
    // which value is in force depend on the order the framework ran the bodies
    // in, so the row is dropped rather than resolved to a guess.
    let ambiguous = source.replace(
        "::method 'test_read'",
        "::method 'test001'\n   .local~v8 = 'something else'\n\n::method 'test_read'",
    );
    let extraction = extract_bif("SYNTHETIC", &ambiguous);
    assert!(extraction.rows.is_empty(), "{:?}", extraction.rows);
    assert_eq!(
        extraction
            .blocked
            .iter()
            .filter(|b| b.reason == DropReason::UnresolvedFixture)
            .map(|b| b.dropped)
            .sum::<usize>(),
        1
    );
}

/// An `assertSame` whose own arguments assign through `VALUE` invalidates the
/// rows behind it, and not the row it is part of.
///
/// `VALUE.testGroup`'s `test051` is the shape: `assertSame(17, value(n,'abc'))`
/// is itself true, because the setter answers the *old* value, and the next
/// assertion is true only because that call assigned.
#[test]
fn a_value_setter_blocks_the_assertions_behind_it() {
    let source = "::method 'test_setter'\n   v = 17\n   n = 'v'\n   \
                  self~assertSame(17, value(n,'abc'))\n   self~assertSame('abc', value(n))\n";
    let extraction = extract_bif("SYNTHETIC", source);
    assert_eq!(extraction.rows.len(), 1, "{:?}", extraction.rows);
    assert_eq!(extraction.rows[0].expected, "value(n,'abc')");
    assert_eq!(
        extraction
            .blocked
            .iter()
            .filter(|b| b.reason == DropReason::SideEffectingAssertion)
            .map(|b| b.dropped)
            .sum::<usize>(),
        1
    );

    // The adjacent success: the reader form assigns nothing and blocks nothing.
    let reader = source.replace("value(n,'abc')", "value(n)");
    assert_eq!(extract_bif("SYNTHETIC", &reader).rows.len(), 2);
}

/// A `DATE`/`TIME` call with an input value is deterministic and a bare one is
/// not, and only the second is dropped (decision D11).
///
/// This matters because the harness runs a value row's two operands as two
/// separate programs: a clock read that moved between them would report a
/// divergence that is nothing but the clock.
#[test]
fn a_clock_read_is_dropped_and_a_reformat_is_not() {
    let bare = "::method 'test_clock'\n   self~assertSame(8, length(date('S')))\n";
    let extraction = extract_bif("SYNTHETIC", bare);
    assert!(extraction.rows.is_empty(), "{:?}", extraction.rows);
    assert_eq!(
        extraction
            .blocked
            .iter()
            .filter(|b| b.reason == DropReason::ClockDependent)
            .map(|b| b.dropped)
            .sum::<usize>(),
        1
    );

    let supplied =
        "::method 'test_reformat'\n   self~assertSame('19961113', date('S','1996-11-13','I'))\n";
    assert_eq!(extract_bif("SYNTHETIC", supplied).rows.len(), 1);

    // An omitted second argument is a clock read too, which a "counts its
    // arguments" rule alone would wave through.
    let omitted = "::method 'test_omitted'\n   self~assertSame(8, length(date('S',,'I')))\n";
    assert!(extract_bif("SYNTHETIC", omitted).rows.is_empty());
}
