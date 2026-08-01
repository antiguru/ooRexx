use rexx_extract::{Form, RaiseExpectation, extract_assertions, find_test_groups};
use std::path::Path;

/// Two `assertSame` calls in one method, a `NUMERIC DIGITS` change between
/// them -- the exact shape `SUBTRACTION.testGroup`'s `test_2` uses. This is
/// the brief's own named risk: if the extractor read the file's settings in
/// isolation rather than carrying state sequentially, both rows could come
/// out with the same (wrong) digits and this test would not catch it, so it
/// asserts the two values are *different* and each matches its own call.
#[test]
fn digits_changing_mid_method_is_carried_sequentially_not_read_in_isolation() {
    let source = r#"
::class "D.testGroup" subclass ooTestCase public

::method "test_2"
   numeric digits 9
   self~assertSame(1.23456789012345E-13 - 0, 123.456789E-15)
   numeric digits 18
   self~assertSame(1.23456789012345E-13 - 0, 0.000000000000123456789012345)
"#;
    let out = extract_assertions("D", source);
    assert_eq!(out.rows.len(), 2);
    assert_eq!(out.rows[0].digits, 9);
    assert_eq!(out.rows[0].expected, "123.456789E-15");
    assert_eq!(out.rows[1].digits, 18);
    assert_eq!(out.rows[1].expected, "0.000000000000123456789012345");
    assert!(out.blocked.is_empty());
}

/// `NUMERIC FORM` is carried exactly the same way `DIGITS` is, and defaults
/// to `Scientific` at the start of a fresh method. Not a case the brief
/// named, but the same risk: `ADDITION.testGroup`'s `test_198` sets
/// `ENGINEERING` before an assertion whose expected value
/// (`20.000E+12`) is only correct in that notation.
#[test]
fn form_changing_mid_method_is_carried_sequentially() {
    let source = r#"
::class "F.testGroup" subclass ooTestCase public

::method "test_1"
   self~assertSame(1 + 1, '2')
   Numeric Form ENGINEERING
   Numeric Digits 5
   self~assertSame(9999999999999 + 9999999999999, 20.000E+12)
"#;
    let out = extract_assertions("F", source);
    assert_eq!(out.rows.len(), 2);
    assert_eq!(out.rows[0].form, Form::Scientific);
    assert_eq!(out.rows[0].digits, 9);
    assert_eq!(out.rows[1].form, Form::Engineering);
    assert_eq!(out.rows[1].digits, 5);
}

/// `CONCATENATION.testGroup`'s actual shape: an `a`..`g` prelude, then many
/// assertions against it. A row without the prelude cannot be evaluated
/// meaningfully -- the brief's own named silent-pass trap -- so every row
/// from this method must carry it.
#[test]
fn a_method_with_an_assignment_prelude_attaches_it_to_every_row() {
    let source = r#"
::class "C.testGroup" subclass ooTestCase public

::method "test_1"
   a="abcdefg"'00'x
   b="abcdefgh"
   self~assertSame((a==a) (b==a), '1 0')
   self~assertSame((a==b) (b==b), '0 1')
"#;
    let out = extract_assertions("C", source);
    assert_eq!(out.rows.len(), 2);
    let expected_prelude = vec![
        r#"a="abcdefg"'00'x"#.to_string(),
        r#"b="abcdefgh""#.to_string(),
    ];
    assert_eq!(out.rows[0].prelude, expected_prelude);
    assert_eq!(out.rows[1].prelude, expected_prelude);
    assert_eq!(out.rows[0].expr, "(a==a) (b==a)");
    assert_eq!(out.rows[0].expected, "'1 0'");
}

/// A method with no prelude at all -- the common case, and every
/// `PRECEDENCE.testGroup` method -- gets an empty prelude, not a missing or
/// magic sentinel value.
#[test]
fn a_self_contained_method_gets_an_empty_prelude() {
    let source = r#"
::class "P.testGroup" subclass ooTestCase public

::method "test_1"
   self~assertSame(0*0, '0')
"#;
    let out = extract_assertions("P", source);
    assert_eq!(out.rows.len(), 1);
    assert!(out.rows[0].prelude.is_empty());
}

/// An unsupported statement (here, the `DO`/`END` loop shape
/// `Literals.testGroup` and `MULTIPLICATION.testGroup` both use) blocks
/// every `assertSame` from that point on in the same method, but does not
/// retroactively invalidate assertions already turned into rows earlier in
/// the method: nothing about their state actually changed.
#[test]
fn an_unsupported_statement_blocks_only_what_follows_it() {
    let source = r#"
::class "L.testGroup" subclass ooTestCase public

::method "test_1"
   self~assertSame(1 + 1, '2')
   do n = 1 to 2
     self~assertSame(n, n)
   end
"#;
    let out = extract_assertions("L", source);
    assert_eq!(
        out.rows.len(),
        1,
        "the assertion before the loop still counts"
    );
    assert_eq!(out.rows[0].expr, "1 + 1");
    assert_eq!(out.blocked.len(), 1);
    assert_eq!(out.blocked[0].method, "test_1");
    assert_eq!(
        out.blocked[0].dropped, 1,
        "the one assertSame inside the loop"
    );
    assert!(out.blocked[0].reason.contains("do n = 1 to 2"));
}

/// A trailing bare `return`, with nothing after it in the method -- the
/// shape `ADDITION.testGroup`'s `test_198` and `REMAINDER.testGroup`'s
/// `test_293` both end on -- is technically an unsupported statement too,
/// but since no `assertSame` follows it, nothing is ever dropped and the
/// method must not appear in `blocked` at all: a `blocked` entry with
/// `dropped == 0` would misreport a method that lost nothing.
#[test]
fn a_trailing_return_with_nothing_after_it_reports_no_blocked_method() {
    let source = r#"
::class "R.testGroup" subclass ooTestCase public

::method "test_198"
   Numeric Form ENGINEERING
   Numeric Digits 5
   self~assertSame(9999999999999 + 9999999999999, 20.000E+12)
return
"#;
    let out = extract_assertions("R", source);
    assert_eq!(out.rows.len(), 1);
    assert!(
        out.blocked.is_empty(),
        "the return follows the only assertion, so nothing was dropped"
    );
}

/// A non-`assertSame` assertion (`assertTrue`, `expectSyntax`, ...) carries
/// no variables and must not be mistaken for an unsupported statement or
/// change `NUMERIC` state: it is simply not a row.
#[test]
fn other_assertion_kinds_are_ignored_without_blocking_or_changing_state() {
    let source = r#"
::class "M.testGroup" subclass ooTestCase public

::method "test_1"
   self~assertTrue(1 = 1)
   numeric digits 5
   self~assertSame(1/3, '0.33333')
   self~expectSyntax(42.3)
"#;
    let out = extract_assertions("M", source);
    assert_eq!(out.rows.len(), 1);
    assert_eq!(out.rows[0].digits, 5);
    assert!(out.blocked.is_empty());
}

/// `DIVISION.testGroup`'s own `test_262` shape: `self~expectSyntax` is not
/// even the line immediately before the assertion (`Numeric Digits 5` sits
/// between them), so a row after it must still pick up the expectation --
/// a same-line or previous-line check would have missed this.
#[test]
fn an_expect_syntax_marker_turns_the_following_row_into_a_raise_expectation() {
    let source = r#"
::class "S.testGroup" subclass ooTestCase public

::method "test_262"
   self~expectSyntax(26.11)
   Numeric Digits 5
   self~assertSame("-5678932" % "-37", 1)
"#;
    let out = extract_assertions("S", source);
    assert_eq!(out.rows.len(), 1);
    assert_eq!(
        out.rows[0].expect_raise,
        Some(RaiseExpectation { major: 26, sub: 11 })
    );
    assert_eq!(out.rows[0].digits, 5, "NUMERIC DIGITS is still carried too");
    assert!(out.blocked.is_empty());
}

/// A row before any `expectSyntax` in the same method is an ordinary value
/// row; only what follows the marker changes meaning. Mirrors
/// `an_unsupported_statement_blocks_only_what_follows_it`'s own shape for
/// blocking, but this marker does not block -- it converts.
#[test]
fn a_row_before_the_expect_syntax_marker_is_unaffected() {
    let source = r#"
::class "T.testGroup" subclass ooTestCase public

::method "test_1"
   self~assertSame(1 + 1, '2')
   self~expectSyntax(26.11)
   self~assertSame(1 % 0, 1)
"#;
    let out = extract_assertions("T", source);
    assert_eq!(out.rows.len(), 2);
    assert_eq!(out.rows[0].expect_raise, None);
    assert_eq!(
        out.rows[1].expect_raise,
        Some(RaiseExpectation { major: 26, sub: 11 })
    );
}

/// Carried sequentially, exactly like `digits`/`form`: a second
/// `expectSyntax` changes the expectation for whatever follows *it*, rather
/// than the first marker sticking for the rest of the method. No method in
/// `base/expressions` does this today (checked), but the state-carrying
/// mechanism is the same one `digits`/`form` already need to get right, so
/// it is proven directly rather than left as an unexercised claim.
#[test]
fn a_second_expect_syntax_marker_replaces_the_first_for_what_follows_it() {
    let source = r#"
::class "U.testGroup" subclass ooTestCase public

::method "test_1"
   self~expectSyntax(26.11)
   self~assertSame(1 % 0, 1)
   self~expectSyntax(42.3)
   self~assertSame(1 % 0, 1)
"#;
    let out = extract_assertions("U", source);
    assert_eq!(out.rows.len(), 2);
    assert_eq!(
        out.rows[0].expect_raise,
        Some(RaiseExpectation { major: 26, sub: 11 })
    );
    assert_eq!(
        out.rows[1].expect_raise,
        Some(RaiseExpectation { major: 42, sub: 3 })
    );
}

/// `expectSyntax`'s array form (`(major.sub, message inserts...)`) is not
/// one of the 19 real shapes this scanner was built against, and rather
/// than guess which comma-separated item is the code, it blocks -- the
/// same conservative choice `parse_assert_same` makes for an `assertSame`
/// shape it does not recognise.
#[test]
fn expect_syntax_with_message_inserts_blocks_rather_than_guesses() {
    let source = r#"
::class "V.testGroup" subclass ooTestCase public

::method "test_1"
   self~expectSyntax((15.1, 1))
   self~assertSame(1, 1)
"#;
    let out = extract_assertions("V", source);
    assert!(out.rows.is_empty());
    assert_eq!(out.blocked.len(), 1);
    assert_eq!(out.blocked[0].dropped, 1);
}

/// `self~assertSyntaxError`/`self~assertRuntimeError` call `expectSyntax`
/// internally but check the raise *within that same call* -- confirmed by
/// reading `OOREXXUNIT.CLS` -- so unlike a bare `self~expectSyntax`, they
/// leave nothing pending for a later `self~assertSame` in the same method.
/// This is the existing "other assertion kinds are inert" behaviour and
/// must not regress into treating them like `expectSyntax`.
#[test]
fn assert_syntax_error_does_not_leave_a_pending_raise_expectation() {
    let source = r#"
::class "W.testGroup" subclass ooTestCase public

::method "test_1"
   self~assertSyntaxError(15.1, self~hex(" "))
   self~assertSame(1 + 1, '2')
"#;
    let out = extract_assertions("W", source);
    assert_eq!(out.rows.len(), 1);
    assert_eq!(out.rows[0].expect_raise, None);
    assert!(out.blocked.is_empty());
}

/// `self~expectCondition` defers a raise-check the same way `expectSyntax`
/// does, but for an arbitrary condition *name* rather than a numbered
/// `SYNTAX` one -- this row schema has nowhere to carry a bare name, so it
/// blocks rather than silently falling into the generic "other assertion
/// kind, inert" branch, which would reproduce the exact bug this file's
/// other tests exist to close. Not a case `base/expressions` has today
/// (checked); this is a forward guard.
#[test]
fn expect_condition_blocks_rather_than_silently_passing_through() {
    let source = r#"
::class "X.testGroup" subclass ooTestCase public

::method "test_1"
   self~expectCondition("USER SOMETHING")
   self~assertSame(1, 1)
"#;
    let out = extract_assertions("X", source);
    assert!(out.rows.is_empty());
    assert_eq!(out.blocked.len(), 1);
    assert_eq!(out.blocked[0].dropped, 1);
}

/// A method whose name is not `test`-prefixed is not run by the ooTest
/// framework at all (matching `extract`'s own existing filter), so an
/// `assertSame` inside one must not produce a row.
#[test]
fn a_non_test_method_produces_no_rows() {
    let source = r#"
::class "N.testGroup" subclass ooTestCase public

::method helper
   self~assertSame(1, '1')
"#;
    let out = extract_assertions("N", source);
    assert!(out.rows.is_empty());
    assert!(out.blocked.is_empty());
}

/// Every row and every blocked entry carries the caller-supplied group
/// label verbatim: `extract_assertions` has no notion of files, so the
/// walker is what is trusted to pass the right one.
#[test]
fn rows_and_blocked_entries_carry_the_supplied_group_label() {
    let source = r#"
::class "G.testGroup" subclass ooTestCase public

::method "test_1"
   self~assertSame(1, '1')
   do n = 1 to 1
     self~assertSame(n, n)
   end
"#;
    let out = extract_assertions("MYGROUP", source);
    assert_eq!(out.rows[0].group, "MYGROUP");
    assert_eq!(out.blocked[0].group, "MYGROUP");
}

/// The whole-corpus invariant: every `self~assertSame` call in
/// `base/expressions` is either a row or accounted for in `blocked`'s
/// `dropped` counts, never neither. This is what caught the single-quoted
/// `::method` name bug (`extract`, not this module, silently dropped eight
/// whole methods and 864 calls in `MULTIPLICATION.testGroup`): before that
/// fix this assertion failed for that one file. A shortfall here is always
/// a bug in the extractor, never a rounding error to wave off.
#[test]
fn every_assert_same_in_base_expressions_is_a_row_or_an_accounted_for_drop() {
    let suite =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../ootest/ooRexx/base/expressions");
    let groups = find_test_groups(&suite);
    assert!(
        !groups.is_empty(),
        "corpus went missing: {}",
        suite.display()
    );
    for path in &groups {
        let bytes = std::fs::read(path).expect("readable .testGroup file");
        let source = String::from_utf8_lossy(&bytes);
        let group_name = path.file_stem().and_then(|s| s.to_str()).unwrap();
        let out = extract_assertions(group_name, &source);
        let calls = source
            .to_ascii_lowercase()
            .matches("self~assertsame")
            .count();
        let dropped: usize = out.blocked.iter().map(|b| b.dropped).sum();
        assert_eq!(
            out.rows.len() + dropped,
            calls,
            "{}: {} rows + {dropped} dropped != {calls} assertSame calls",
            path.display(),
            out.rows.len()
        );
    }
}

/// The exact counts this mode was built for, named in the brief: 4,269
/// `self~assertSame` calls total in `base/expressions`, 1,226 of them in
/// `PRECEDENCE`, 388 in `CONCATENATION`. Pinning the totals (4,259
/// extractable, 10 blocked, across `Literals` (6, five `DO`/`OVER` and
/// `DO`-controlled loop methods), `MULTIPLICATION` (2, one more such loop)
/// and `SPECIAL` (2, a method that calls a local label as a pseudo-function)
/// means a change in what this scanner accepts shows up here immediately,
/// by name, rather than as a silent drift in a percentage.
#[test]
fn base_expressions_yields_the_measured_row_and_blocked_counts() {
    let suite =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../ootest/ooRexx/base/expressions");
    let groups = find_test_groups(&suite);
    let mut rows_by_group = std::collections::HashMap::new();
    let mut total_rows = 0usize;
    let mut total_dropped = 0usize;
    for path in &groups {
        let bytes = std::fs::read(path).expect("readable .testGroup file");
        let source = String::from_utf8_lossy(&bytes);
        let group_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap()
            .to_string();
        let out = extract_assertions(&group_name, &source);
        total_rows += out.rows.len();
        total_dropped += out.blocked.iter().map(|b| b.dropped).sum::<usize>();
        rows_by_group.insert(group_name, out.rows.len());
    }
    assert_eq!(total_rows, 4259);
    assert_eq!(total_dropped, 10);
    assert_eq!(rows_by_group["PRECEDENCE"], 1226);
    assert_eq!(rows_by_group["CONCATENATION"], 388);
    assert_eq!(rows_by_group["MULTIPLICATION"], 1048);
    assert_eq!(rows_by_group["Literals"], 39);
    assert_eq!(rows_by_group["SPECIAL"], 27);
}

/// The measured extent of the `self~expectSyntax` conversion: 184 of the
/// 4,259 rows are raise-expectations rather than value comparisons -- 60 in
/// `DIVISION`, 6 in `EXPONENT`, 118 in `REMAINDER`, zero everywhere else
/// (in particular, `PRECEDENCE`'s 139 `self~assertSyntaxError` calls are
/// all self-contained wrapping calls, never followed by a separate
/// `assertSame`, so none of its rows convert). The row/dropped totals
/// themselves are unchanged from the previous test -- converting a row
/// changes what it means, not whether it exists -- pinning that here too
/// makes it explicit that this is not a second, competing count.
#[test]
fn base_expressions_expect_syntax_conversion_counts() {
    let suite =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../ootest/ooRexx/base/expressions");
    let groups = find_test_groups(&suite);
    let mut raising_by_group = std::collections::HashMap::new();
    let mut total_rows = 0usize;
    let mut total_raising = 0usize;
    for path in &groups {
        let bytes = std::fs::read(path).expect("readable .testGroup file");
        let source = String::from_utf8_lossy(&bytes);
        let group_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap()
            .to_string();
        let out = extract_assertions(&group_name, &source);
        total_rows += out.rows.len();
        let raising = out.rows.iter().filter(|r| r.expect_raise.is_some()).count();
        total_raising += raising;
        raising_by_group.insert(group_name, raising);
    }
    assert_eq!(total_rows, 4259, "conversion must not change the row count");
    assert_eq!(total_raising, 184);
    assert_eq!(raising_by_group["DIVISION"], 60);
    assert_eq!(raising_by_group["EXPONENT"], 6);
    assert_eq!(raising_by_group["REMAINDER"], 118);
    assert_eq!(raising_by_group["PRECEDENCE"], 0);
    assert_eq!(raising_by_group["Literals"], 0);
}
