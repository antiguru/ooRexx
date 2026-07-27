use rexx_core::{ObjRef, RootSet};

#[test]
fn globals_are_always_roots() {
    let mut roots = RootSet::new();
    let env = ObjRef::heap(1, 0);
    roots.add_global(".ENVIRONMENT", env);
    assert!(roots.iter().any(|r| r == env));
}

#[test]
fn temporaries_stop_being_roots_when_their_frame_is_popped() {
    let mut roots = RootSet::new();
    let tmp = ObjRef::heap(5, 0);
    let frame = roots.push_frame();
    roots.push_temp(tmp);
    assert!(roots.iter().any(|r| r == tmp));
    roots.pop_frame(frame);
    assert!(!roots.iter().any(|r| r == tmp));
}

#[test]
fn popping_an_outer_frame_discards_the_inner_frames_it_contains() {
    let mut roots = RootSet::new();
    let outer = roots.push_frame();
    roots.push_temp(ObjRef::heap(1, 0));
    let _inner = roots.push_frame();
    roots.push_temp(ObjRef::heap(2, 0));
    roots.pop_frame(outer);
    assert_eq!(roots.iter().count(), 0);
}

#[test]
fn rebinding_a_global_replaces_it_rather_than_adding_a_second_root() {
    let mut roots = RootSet::new();
    let first = ObjRef::heap(1, 0);
    let second = ObjRef::heap(2, 0);
    roots.add_global(".LOCAL", first);
    roots.add_global(".LOCAL", second);
    assert_eq!(roots.iter().count(), 1);
    assert!(roots.iter().any(|r| r == second));
}
