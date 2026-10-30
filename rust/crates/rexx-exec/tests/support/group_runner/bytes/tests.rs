use super::*;

#[test]
fn empty_needle_has_the_substring_contract() {
    assert!(contains(b"", b""));
    assert!(contains(b"abc", b""));
    assert!(!contains(b"", b"a"));
    assert!(contains(b"abc", b"bc"));
    assert!(!contains(b"abc", b"bd"));
}

#[test]
fn skip_distinguishes_invalid_bytes_from_a_replacement_character() {
    let input = b"::method 'test_\xff'\n::method 'test_\xef\xbf\xbd'\n::MeThOd test_ascii\n::method untouched\n";
    let wanted = BTreeSet::from(["GROUP.TEST_\u{fffd}", "GROUP.TEST_ASCII"]);
    let (rewritten, skipped) = renamed(input, "GROUP", &wanted);
    assert_eq!(rewritten, b"::method 'test_\xff'\n::method 'SKIPPED_test_\xef\xbf\xbd'\n::MeThOd SKIPPED_test_ascii\n::method untouched\n");
    assert_eq!(skipped, BTreeSet::from(["GROUP.TEST_\u{fffd}".into(), "GROUP.TEST_ASCII".into()]));
}
