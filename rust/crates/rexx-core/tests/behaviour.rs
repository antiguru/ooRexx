use rexx_core::{BehaviourId, BehaviourTable, MethodId};

#[test]
fn a_method_defined_on_a_behaviour_is_found() {
    let mut t = BehaviourTable::new();
    t.define(BehaviourId::STRING, "LENGTH", MethodId(7));
    assert_eq!(t.lookup(BehaviourId::STRING, "LENGTH"), Some(MethodId(7)));
}

#[test]
fn lookup_is_case_insensitive_because_rexx_message_names_are_uppercased() {
    let mut t = BehaviourTable::new();
    t.define(BehaviourId::STRING, "LENGTH", MethodId(7));
    assert_eq!(t.lookup(BehaviourId::STRING, "length"), Some(MethodId(7)));
}

#[test]
fn lookup_walks_to_the_superclass() {
    let mut t = BehaviourTable::new();
    t.define(BehaviourId::OBJECT, "CLASS", MethodId(1));
    t.set_superclass(BehaviourId::STRING, BehaviourId::OBJECT);
    assert_eq!(t.lookup(BehaviourId::STRING, "CLASS"), Some(MethodId(1)));
}

#[test]
fn a_subclass_method_overrides_the_superclass() {
    let mut t = BehaviourTable::new();
    t.define(BehaviourId::OBJECT, "STRING", MethodId(1));
    t.define(BehaviourId::STRING, "STRING", MethodId(2));
    t.set_superclass(BehaviourId::STRING, BehaviourId::OBJECT);
    assert_eq!(t.lookup(BehaviourId::STRING, "STRING"), Some(MethodId(2)));
}

#[test]
fn an_unknown_message_is_a_miss_rather_than_a_panic() {
    let mut t = BehaviourTable::new();
    t.define(BehaviourId::STRING, "LENGTH", MethodId(7));
    assert_eq!(t.lookup(BehaviourId::STRING, "NOSUCHTHING"), None);
}

#[test]
fn a_superclass_cycle_terminates_instead_of_looping_forever() {
    // Bootstrap genuinely creates cycles -- .class is an instance of itself --
    // so lookup must not depend on the chain being acyclic.
    let mut t = BehaviourTable::new();
    t.set_superclass(BehaviourId::STRING, BehaviourId::OBJECT);
    t.set_superclass(BehaviourId::OBJECT, BehaviourId::STRING);
    assert_eq!(t.lookup(BehaviourId::STRING, "ANYTHING"), None);
}
