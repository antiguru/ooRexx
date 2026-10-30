use super::*;

fn groups() -> Vec<(String, String)> {
    vec![("base".into(), "A".into()), ("base/bif".into(), "B".into())]
}

#[test]
fn absent_selector_keeps_every_group() {
    assert_eq!(selected(groups(), None), groups());
}

#[test]
fn named_groups_are_selected_in_discovery_order() {
    assert_eq!(selected(groups(), Some("base/bif/B")), vec![("base/bif".into(), "B".into())]);
    assert_eq!(selected(groups(), Some(" base/bif/B, base/A ")), groups());
}

#[test]
fn empty_selectors_fail_visibly() {
    for selector in ["", "  ", ",", " , , "] {
        let failure = std::panic::catch_unwind(|| selected(groups(), Some(selector)));
        let error = failure.expect_err("a present selector must name a group");
        let message = error.downcast_ref::<String>().map(String::as_str)
            .or_else(|| error.downcast_ref::<&str>().copied()).expect("panic message");
        assert!(message.contains("REXX_SUITE_GROUPS"), "{message}");
    }
}

#[test]
fn unknown_groups_keep_the_named_diagnostic() {
    let failure = std::panic::catch_unwind(|| selected(groups(), Some("missing/Group")));
    let error = failure.expect_err("an unknown group must fail");
    let message = error.downcast_ref::<String>().expect("panic message");
    assert!(message.contains("REXX_SUITE_GROUPS names no group"), "{message}");
    assert!(message.contains("missing/Group"), "{message}");
}
