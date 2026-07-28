use rexx_num::{compare, ArithError, CompareOp};

fn cmp(a: &str, b: &str, digits: u64, fuzz: u64, op: CompareOp) -> bool {
    compare(a, b, digits, fuzz, op).unwrap()
}

/// Default DIGITS (9), FUZZ 0 -- the common case most tests below use.
fn c(a: &str, b: &str, op: CompareOp) -> bool {
    cmp(a, b, 9, 0, op)
}

#[test]
fn numeric_equal_ignores_form_but_strict_equal_does_not() {
    use CompareOp::*;
    assert!(c("1", "1.0", Equal));
    assert!(!c("1", "1.0", StrictEqual));
    assert!(c("1e2", "100", Equal));
    assert!(!c("1e2", "100", StrictEqual));
}

#[test]
fn strict_comparison_does_not_strip_blanks() {
    use CompareOp::*;
    assert!(!c(" 1", "1", StrictEqual));
    // but the numeric comparison does, since both sides still convert.
    assert!(c(" 1", "1", Equal));
}

#[test]
fn zero_and_negative_zero_are_numerically_equal() {
    use CompareOp::*;
    for zero in ["-0", "0.0", "-0.0", "00", "-00", "0e9", "-0e9"] {
        assert!(c("0", zero, Equal), "0 = {zero}");
    }
}

#[test]
fn strict_ordering_is_plain_byte_ordering() {
    use CompareOp::*;
    assert!(c("a", "b", StrictLess));
    assert!(c("b", "a", StrictGreater));
    assert!(c("ab", "a", StrictGreater)); // shared prefix, longer wins
    assert!(!c("a", "a", StrictLess));
    assert!(c("a", "a", StrictGreaterEqual));
    assert!(c("a", "a", StrictLessEqual));
}

#[test]
fn not_equal_holds_whenever_equal_does_not() {
    use CompareOp::*;
    // `\=`, `<>` and `><` are three spellings of the same operator in the
    // interpreter's grammar (`StringClass.cpp:2391-2410` repeats the same
    // method pointer for all three); this crate models that as one
    // `NotEqual` variant, and the harness's token-to-variant mapping
    // (`bin/muldiv.rs::compare_op`) is what collapses the spellings.
    assert!(c("1", "2", NotEqual));
    assert!(!c("1", "1", NotEqual));
}

#[test]
fn non_numeric_operands_fall_back_to_string_comparison_instead_of_erroring() {
    use CompareOp::*;
    // Neither side converts: plain byte-ish comparison (leading blanks
    // stripped, trailing treated as padding).
    assert!(!c("abc", "abd", Equal));
    assert!(c("abc", "abd", Less));
    // One side numeric, one not: still falls to string comparison, not E41.
    assert!(!c("1", "abc", Equal));
    assert_eq!(compare("1", "abc", 9, 0, Equal), Ok(false));
    // Trailing blanks on the non-numeric side are padding, not content...
    assert!(c("abc", "abc  ", Equal));
    // ... but strict comparison does not treat them that way.
    assert!(!c("abc", "abc  ", StrictEqual));
    // Leading blanks are stripped on both sides for the non-strict compare.
    assert!(c("  abc", "abc", Equal));
    // An empty operand is non-numeric too, and still just compares as a string.
    assert!(c("", "", Equal));
    assert!(c("", "x", Less));
}

#[test]
fn a_borrow_or_carry_never_overflows_an_opposite_sign_comparison() {
    // Comparing two huge, opposite-signed, individually in-range operands
    // must be decided from sign alone. A naive "always subtract" comparison
    // would add their magnitudes together and overflow; the real
    // interpreter never attempts the computation. See `numeric_order`'s doc
    // comment in `compare.rs`.
    use CompareOp::*;
    let big = "9.999999999e999999999";
    let neg_big = "-9.999999999e999999999";
    assert_eq!(compare(big, neg_big, 9, 0, Greater), Ok(true));
    assert_eq!(compare(neg_big, big, 9, 0, Less), Ok(true));
    assert_eq!(compare(big, neg_big, 9, 0, Equal), Ok(false));
}

#[test]
fn truncation_to_digits_can_make_longer_operands_compare_equal_without_fuzz() {
    // At DIGITS 5 both operands truncate to their first 6 (DIGITS + 1)
    // digits before any comparison happens, so `123456789` and
    // `123456780` -- which differ only in the 9th digit -- become
    // identical (123456000) well before FUZZ enters the picture.
    // Verified against the interpreter (`build/bin/rexx`, DIGITS 5).
    use CompareOp::*;
    assert!(cmp("123456789", "123456780", 5, 0, Equal));
    assert!(!cmp("123456789", "123456780", 9, 0, Equal));
    assert!(cmp("123456789", "123456780", 9, 0, Greater));
}

/// Table reproduced from a direct run of `build/bin/rexx` against a small
/// probe script (`numeric digits dd; numeric fuzz ff; say a op b` for each
/// pair/operator/setting below; see the task report for the full script).
/// NUMERIC FUZZ has no column in the `digits|a|op|b` case format, so it
/// cannot go through the usual oracle harness; this pins what the probe
/// showed instead.
#[test]
fn numeric_fuzz_relaxes_the_numeric_operators_but_never_the_strict_ones() {
    use CompareOp::*;

    // digits=9 fuzz=0: only the truly-identical pair compares equal.
    assert!(!cmp("123456789", "123456780", 9, 0, Equal));
    assert!(cmp("123456789", "123456780", 9, 0, Greater));
    assert!(cmp("123456789", "123456789", 9, 0, Equal));
    assert!(cmp("-123456789", "-123456780", 9, 0, Less));
    assert!(!cmp("1000000000", "1000000009", 9, 0, Equal));
    assert!(!cmp("100", "100.001", 9, 0, Equal));
    // strict comparisons of the same text at fuzz 0, for a baseline.
    assert!(!cmp("123456789", "123456780", 9, 0, StrictLess));
    assert!(cmp("123456789", "123456780", 9, 0, StrictGreater));

    // digits=9 fuzz=1 (working precision 8): the two 10-digit operands
    // truncate to 9 working digits inside `sub` (DIGITS + 1); the digit
    // they differ in is the one that truncation drops, so they become
    // identical before any rounding happens. The 9-digit pairs are one
    // fuzz level short of that: truncating to 9 digits keeps all of theirs,
    // so their difference survives.
    assert!(!cmp("123456789", "123456780", 9, 1, Equal));
    assert!(cmp("1000000000", "1000000009", 9, 1, Equal));
    assert!(!cmp("100", "100.001", 9, 1, Equal));

    // digits=9 fuzz=2 (working precision 7): now the 9-digit pairs also
    // truncate down to 8 digits, dropping their one differing digit too.
    for (a, b) in [("123456789", "123456780"), ("-123456789", "-123456780"), ("1000000000", "1000000009")] {
        assert!(cmp(a, b, 9, 2, Equal), "{a} = {b} at fuzz 2");
    }
    assert!(!cmp("100", "100.001", 9, 2, Equal));

    // digits=9 fuzz=8 (working precision 1): everything collapses, including
    // the decimal pair.
    assert!(cmp("100", "100.001", 9, 8, Equal));

    // digits=5 fuzz=0: DIGITS itself already truncates every pair here to
    // the point of exact equality (see the dedicated truncation test above);
    // FUZZ changes nothing further at fuzz=4.
    for (a, b) in [
        ("123456789", "123456780"),
        ("-123456789", "-123456780"),
        ("1000000000", "1000000009"),
        ("100", "100.001"),
    ] {
        assert!(cmp(a, b, 5, 0, Equal), "{a} = {b} at digits 5 fuzz 0");
        assert!(cmp(a, b, 5, 4, Equal), "{a} = {b} at digits 5 fuzz 4");
    }

    // Strict comparisons are unaffected by FUZZ at any of the above settings.
    assert!(!cmp("123456789", "123456780", 9, 8, StrictEqual));
    assert!(!cmp("100", "100.001", 5, 4, StrictEqual));
}

#[test]
fn compare_never_errors_for_ordinary_operands() {
    // Comparison can in principle propagate an ArithError from the
    // underlying subtraction; ordinary operands never trigger it.
    let r: Result<bool, ArithError> = compare("1", "2", 9, 0, CompareOp::Less);
    assert_eq!(r, Ok(true));
}
