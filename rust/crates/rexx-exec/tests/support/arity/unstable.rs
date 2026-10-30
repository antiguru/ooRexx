use super::*;

#[derive(Clone, Copy)]
enum IdentityProperty {
    RepeatableHash,
    UniqueIdentity,
}

fn identity_property(class: &str, method: &str, arm: &str) -> Option<IdentityProperty> {
    match (class, method, arm) {
        ("Object", "hashCode", "instance") => Some(IdentityProperty::RepeatableHash),
        ("Object", "identityHash", "instance") => Some(IdentityProperty::UniqueIdentity),
        _ => None,
    }
}

pub(super) fn classified(class: &str, method: &str, arm: &str) -> bool {
    identity_property(class, method, arm).is_some()
}

fn property_program(receiver: &Receiver, method: &str, property: IdentityProperty) -> String {
    let predicate = match property {
        IdentityProperty::RepeatableHash => "vv == r~hashCode",
        IdentityProperty::UniqueIdentity => "(vv == r~identityHash) & (vv \\== other~identityHash)",
    };
    // Hold both receivers until the identity comparison finishes. A hash
    // collision between unequal objects is allowed by the hashCode contract.
    let mut setup = receiver.setup.clone();
    if matches!(property, IdentityProperty::UniqueIdentity) {
        setup.push_str(" | other = r~class~new");
    }
    let receiver = Receiver {
        setup,
        wrapper: receiver.wrapper.clone(),
        directives: receiver.directives.clone(),
    };
    program_rendering(&receiver, method, NONE, true, &format!("({predicate})"))
}

fn property_holds(answer: &Three) -> bool {
    answer == &(0, "SETUP-OK\nSENT\nVALUE 1\n".into(), String::new())
}

/// The identity-derived rows whose documented property did not hold, and
/// markers lacking an explicit property classification.
pub fn invalid_identity_markers(layout: &Layout) -> Vec<String> {
    let receivers = layout.receivers();
    let dir = probe_dir(layout.probe_prefix);
    let path = dir.join("probe.rex");
    if layout.fixture {
        fs::write(dir.join(FIXTURE), "return 1\n").expect("the fixture is writable");
    }
    let oracle = oracle::oracle_root();
    let mut invalid = Vec::new();
    for (class, method, arm, list) in layout.arguments() {
        if !list.starts_with(UNSTABLE) {
            continue;
        }
        let Some(property) = identity_property(&class, &method, &arm) else {
            invalid.push(format!(
                "{class}~{method} ({arm}) has an unclassified {UNSTABLE} marker"
            ));
            continue;
        };
        let receiver = receivers
            .get(&(class.clone(), arm.clone()))
            .unwrap_or_else(|| panic!("{} has no setup for {class} ({arm})", layout.receivers));
        fs::write(&path, property_program(receiver, &method, property))
            .expect("the probe is writable");
        let answer = run(&mut oracle_command(&oracle, &path, &dir));
        if !property_holds(&answer) {
            invalid.push(format!(
                "{class}~{method} ({arm}) identity property failed: {answer:?}"
            ));
        }
    }
    fs::remove_dir_all(dir).expect("remove identity probes");
    invalid
}

#[cfg(test)]
mod tests;
