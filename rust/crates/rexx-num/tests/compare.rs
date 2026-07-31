use rexx_num::{ArithError, CompareOp, Number, compare, compare_bytes, compare_decoded};

fn cmp(a: &str, b: &str, digits: u64, fuzz: u64, op: CompareOp) -> bool {
    compare(a, b, digits, fuzz, op).unwrap()
}

/// Default DIGITS (9), FUZZ 0 -- the common case most tests below use.
fn c(a: &str, b: &str, op: CompareOp) -> bool {
    cmp(a, b, 9, 0, op)
}

fn cb(a: &[u8], b: &[u8], digits: u64, fuzz: u64, op: CompareOp) -> bool {
    compare_bytes(a, b, digits, fuzz, op).unwrap()
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
    for (a, b) in [
        ("123456789", "123456780"),
        ("-123456789", "-123456780"),
        ("1000000000", "1000000009"),
    ] {
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

/// The nine discriminating transcripts, measured directly against
/// `build/bin/rexx` (Task 8a's report has the wrapped run). The first row is
/// the one that matters: an earlier spec draft described the rule as
/// "blank-pad the shorter operand on the right", which this contradicts --
/// padding `"a"` on the right to match `" a"` gives `"a "`, whose first byte
/// (`'a'`) does not match `" a"`'s first byte (a space), so that rule would
/// answer `0` where the interpreter answers `1`. Stripping *leading* blanks
/// instead (what `string_order` actually does) reduces both operands to
/// `"a"`, which is what makes this the row that tells the two rules apart --
/// none of the other eight can.
#[test]
fn the_nine_transcripts_that_pin_leading_not_padded_blank_stripping() {
    use CompareOp::*;
    let eq = |a: &[u8], b: &[u8]| cb(a, b, 9, 0, Equal);
    assert!(eq(b" a", b"a"), "leading blank stripped, not right-padded");
    assert!(eq(b"\ta", b"a"), "leading tab stripped the same way");
    assert!(eq(b"a", b"a\t"), "a lone trailing tab is blank padding");
    assert!(
        eq(b"a", b"a "),
        "a lone trailing space is blank padding too"
    );
    assert!(
        !eq(b"a b", b"a  b"),
        "interior blanks are content, never collapsed"
    );
    assert!(eq(b"", b" "), "empty against all-blank");
    assert!(
        eq(b"01", b"1"),
        "numeric: leading zero does not change the value"
    );
    assert!(eq(b" 1 ", b"1"), "numeric: surrounding blanks on a number");
    assert!(
        !eq(b"a", b"1"),
        "non-numeric left side falls back to string compare"
    );
}

/// The case `compare`'s `&str` signature cannot express at all -- a Rexx
/// string that is not valid UTF-8 (D14). `'C3'x` alone is an incomplete
/// UTF-8 continuation sequence, invalid on its own; measured against the
/// oracle (task report) that comparison still works on it exactly as on any
/// other byte string, with no UTF-8 requirement anywhere in the actual rule.
#[test]
fn a_non_utf8_operand_compares_correctly_where_str_could_not_express_it() {
    use CompareOp::*;
    // 0xC3 is a two-byte UTF-8 lead byte with no continuation byte after
    // it, invalid on its own -- rustc's own `invalid_from_utf8` lint proves
    // this at compile time for a literal this shape, which is exactly why
    // there is no runtime `from_utf8` self-check here to trigger it.
    let c3 = [0xC3u8];
    let c4 = [0xC4u8];
    assert!(cb(&c3, &c3, 9, 0, Equal), "identical non-UTF-8 bytes");
    assert!(!cb(&c3, &c4, 9, 0, Equal), "different non-UTF-8 bytes");
    assert!(cb(&c3, &c3, 9, 0, StrictEqual));

    // A leading blank in front of a non-UTF-8 byte is still stripped: the
    // rule inspects individual bytes, never a decoded character, so this
    // works with no special-casing for the byte that follows the blank.
    let blank_then_c3 = [b' ', 0xC3];
    assert!(cb(&blank_then_c3, &c3, 9, 0, Equal));
    assert!(!cb(&blank_then_c3, &c3, 9, 0, StrictEqual));

    // Non-UTF-8 bytes can never be a Rexx number (one is ASCII by
    // definition), so a numeric operator on one falls back to the string
    // rule exactly as non-numeric ASCII text does, not to an error.
    assert_eq!(compare_bytes(&c3, b"1", 9, 0, Equal), Ok(false));
}

/// `compare`, `compare_bytes` and `compare_decoded` (with nothing
/// pre-parsed) must answer identically for every operand and operator here,
/// because all three are required to reach the same `numeric_order`/
/// `string_order` rather than each carrying its own copy of it. Checked
/// computationally rather than only asserted in a doc comment.
#[test]
fn compare_and_compare_bytes_and_compare_decoded_agree_on_every_case_above() {
    use CompareOp::*;
    let cases: &[(&str, &str, CompareOp)] = &[
        (" a", "a", Equal),
        ("a b", "a  b", Equal),
        ("01", "1", Equal),
        (" 1 ", "1", Equal),
        ("a", "1", Equal),
        ("123456789", "123456780", Greater),
        ("100", "100.001", StrictEqual),
    ];
    for &(a, b, op) in cases {
        let via_str = compare(a, b, 9, 0, op).unwrap();
        let via_bytes = compare_bytes(a.as_bytes(), b.as_bytes(), 9, 0, op).unwrap();
        let via_decoded =
            compare_decoded(a.as_bytes(), None, b.as_bytes(), None, 9, 0, op).unwrap();
        assert_eq!(
            via_str, via_bytes,
            "{a:?} {op:?} {b:?}: compare vs compare_bytes"
        );
        assert_eq!(
            via_str, via_decoded,
            "{a:?} {op:?} {b:?}: compare vs compare_decoded"
        );
    }
}

/// `compare_decoded` genuinely uses a caller-supplied `Number` rather than
/// silently re-deriving one from the bytes -- proven, not merely claimed, by
/// supplying a `Number` that disagrees with what parsing the bytes would
/// give and checking the *supplied* value is what decided the answer.
/// `b"999"` alone would compare unequal to `"1"`; passed in as a pre-parsed
/// `1`, it compares equal instead, which could only happen if the parameter
/// was actually consulted.
#[test]
fn compare_decoded_uses_the_supplied_number_not_a_fresh_parse_of_the_bytes() {
    use CompareOp::*;
    // Sanity check first: without the override, the bytes really do parse
    // to 999 and really do compare unequal to 1.
    assert!(!cb(b"999", b"1", 9, 0, Equal));

    let one = Number::parse("1").expect("\"1\" parses");
    let overridden = compare_decoded(b"999", Some(&one), b"1", None, 9, 0, Equal).unwrap();
    assert!(
        overridden,
        "the supplied Number (1) must be used in place of parsing b\"999\" (which is 999)"
    );

    // Symmetric check on the right-hand side, and with a Greater comparison
    // so the direction of the override is pinned too, not only equality.
    let five = Number::parse("5").expect("\"5\" parses");
    let overridden_right = compare_decoded(b"1", None, b"999", Some(&five), 9, 0, Less).unwrap();
    assert!(
        overridden_right,
        "1 < 5 (the supplied override), even though b\"999\" alone would make 1 < 999 true too \
         -- this only pins the override if the *value* 5 was used, which the next assertion checks"
    );
    let not_less_than_five = compare_decoded(b"6", None, b"999", Some(&five), 9, 0, Less).unwrap();
    assert!(
        !not_less_than_five,
        "6 is not less than the supplied override 5, even though 6 < 999 (the raw bytes) is true"
    );
}

/// Passing a pre-parsed `Number` must not change the *strict* family's
/// answer: the strict operators compare an operand's own text, never a
/// value derived from it, so a supplied `Number` (which the numeric family
/// alone consumes) must be ignored by the strict path -- `"007"` and `"7"`
/// parse to the same `Number` but must not strict-compare equal.
#[test]
fn a_supplied_number_does_not_affect_strict_comparison() {
    use CompareOp::*;
    // "007" and "7" parse to the same Number (7) but are different bytes.
    let seven = Number::parse("007").expect("\"007\" parses");
    // Numeric: the supplied Number (and the bytes-derived one on the right)
    // agree, so this is equal.
    assert!(compare_decoded(b"007", Some(&seven), b"7", None, 9, 0, Equal).unwrap());
    // Strict: must still be false, exactly as comparing the two byte strings
    // directly would be -- if the supplied Number leaked into the strict
    // path, "both sides are numerically 7" could wrongly make this true.
    assert!(!compare_decoded(b"007", Some(&seven), b"7", None, 9, 0, StrictEqual).unwrap());
    assert_eq!(
        compare_decoded(b"007", Some(&seven), b"7", None, 9, 0, StrictEqual),
        compare_bytes(b"007", b"7", 9, 0, StrictEqual),
        "the override must not change the strict answer from what the plain bytes give"
    );
}
