use super::*;

#[test]
fn only_the_documented_identity_rows_can_skip_value_comparison() {
    assert!(identity_property("Object", "hashCode", "instance").is_some());
    assert!(identity_property("Object", "identityHash", "instance").is_some());
    for (class, method, arm) in [("String", "hashCode", "instance"), ("Object", "string", "instance"), ("Object", "identityHash", "class")] {
        assert!(identity_property(class, method, arm).is_none());
    }
}

#[test]
fn identity_properties_reject_stable_overrides_and_failed_sends() {
    let receiver = Receiver { setup: "r = .K~new".into(), wrapper: NONE.into(), directives: "::class K subclass Object | ::method identityHash | return 1".into() };
    let property = identity_property("Object", "identityHash", "instance").unwrap();
    let dir = probe_dir("identity-property-negative");
    let path = dir.join("probe.rex");
    fs::write(&path, property_program(&receiver, "identityHash", property)).unwrap();
    let answer = run(&mut oracle_command(&oracle::oracle_root(), &path, &dir));
    assert_eq!(answer, (0, "SETUP-OK\nSENT\nVALUE 0\n".into(), String::new()));
    assert!(!property_holds(&answer), "{answer:?}");
    assert!(!property_holds(&(0, "SETUP-OK\nSYNTAX 97.1\n".into(), String::new())));
    fs::remove_dir_all(dir).unwrap();
}
