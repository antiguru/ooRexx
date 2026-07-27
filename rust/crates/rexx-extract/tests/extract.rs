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
