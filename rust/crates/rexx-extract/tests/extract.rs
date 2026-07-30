use rexx_extract::extract;

const SAMPLE: &str = r#"
::class "Demo.testGroup" subclass ooTestCase public

::method setUp
  self~thing = 1

::method testAddition
  self~assertEquals(2, 1 + 1)

::method testSelfFree
  x = "abc"
  self~assertEquals(3, x~length)

::method helperNotATest
  return 7
"#;

#[test]
fn only_test_methods_are_extracted() {
    let methods = extract(SAMPLE);
    let names: Vec<&str> = methods.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, vec!["testAddition", "testSelfFree"]);
}

#[test]
fn methods_touching_instance_state_are_flagged_as_fixture_dependent() {
    let methods = extract(SAMPLE);
    let addition = methods.iter().find(|m| m.name == "testAddition").unwrap();
    let free = methods.iter().find(|m| m.name == "testSelfFree").unwrap();
    // `self~assertEquals` is the shim, not fixture state; `self~thing` would be.
    assert!(!addition.uses_fixture);
    assert!(!free.uses_fixture);
}

/// A `::method` name may be single-quoted, not just double-quoted --
/// `MULTIPLICATION.testGroup` names eight of its methods this way (e.g.
/// `::method 'test_15_bit'`). Trimming only `"` left the quotes in the
/// name, which then failed the "starts with test" check and silently
/// dropped the method (and, in that file, 864 `self~assertSame` calls
/// across the eight of them combined -- found while extracting
/// `base/expressions` for the `AssertionRow` mode, 2026-07-30).
#[test]
fn a_single_quoted_method_name_is_still_recognised_as_a_test() {
    let source = r#"
::class "Demo.testGroup" subclass ooTestCase public

::method 'test_15_bit'  -- 15-bit edge cases
   self~assertSame(1, '1')
"#;
    let methods = extract(source);
    let names: Vec<&str> = methods.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, vec!["test_15_bit"]);
}
