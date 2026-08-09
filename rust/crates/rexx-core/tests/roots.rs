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

// ---------------------------------------------------------------------------
// Where an IR chunk's register file can live.
//
// Three candidate homes were on the table. These decide between them by
// running them rather than by arguing, because the argument for the losing
// one was sound and the premise it was missing is only visible here.
// ---------------------------------------------------------------------------

/// The rejected design, and why: a register `SlotFrame` of its own makes
/// every run-time variable introduction underneath it panic.
///
/// `grow_slots` asserts its target is the top frame, and its own doc says
/// that assertion "is not a placeholder awaiting a later relaxation". A
/// program reaches this on `INTERPRET` introducing a name, on `DROP (v)`,
/// and on the first `CALL` in a program that never writes `RESULT` -- none
/// of them exotic.
#[test]
#[should_panic(expected = "grow_slots on a frame that is not the top one")]
fn a_register_slot_frame_makes_the_variable_frame_beneath_it_ungrowable() {
    let mut roots = RootSet::new();
    let variables = roots.push_slots(2);
    let _registers = roots.push_slots(4);
    roots.grow_slots(variables);
}

/// The chosen design: registers as an indexable temporaries region, which
/// the variable frame beneath is free to grow through.
///
/// The neighbouring success to the refusal above, which is what pins the
/// fix to the property it is supposed to have rather than to a coincidence.
#[test]
fn registers_in_the_temps_region_leave_the_variable_frame_growable() {
    let mut roots = RootSet::new();
    let variables = roots.push_slots(2);
    let registers = roots.reserve_temps(4);
    let live = ObjRef::heap(7, 0);
    roots.set_temp(registers, 3, live);

    // The growth the rejected design could not survive.
    let grown = roots.grow_slots(variables);
    roots.set_slot(variables, grown, ObjRef::heap(8, 0));

    assert_eq!(roots.temp_at(registers, 3), live);
    assert!(roots.iter().any(|r| r == live));
}

/// A register is a root for as long as its region is open, and stops being
/// one when the region closes -- the property that makes intermediates safe
/// to hold in registers at all.
#[test]
fn registers_are_roots_until_their_region_is_truncated() {
    let mut roots = RootSet::new();
    let outer = roots.push_frame();
    let registers = roots.reserve_temps(2);
    let held = ObjRef::heap(9, 0);
    roots.set_temp(registers, 1, held);
    assert!(roots.iter().any(|r| r == held));
    roots.pop_frame(outer);
    assert!(!roots.iter().any(|r| r == held));
}

/// Register regions nest, which is what a fragment chunk compiled and run
/// inside an already-running chunk needs.
///
/// `INTERPRET` pushes no frame of its own and runs inside the enclosing
/// activation, so a fragment's registers cannot be a second frame and must
/// not disturb the outer chunk's.
#[test]
fn a_nested_register_region_leaves_the_enclosing_one_intact() {
    let mut roots = RootSet::new();
    let outer_registers = roots.reserve_temps(3);
    let outer_value = ObjRef::heap(11, 0);
    roots.set_temp(outer_registers, 2, outer_value);

    let fragment = roots.push_frame();
    let inner_registers = roots.reserve_temps(2);
    roots.set_temp(inner_registers, 0, ObjRef::heap(12, 0));
    assert_eq!(roots.temp_at(outer_registers, 2), outer_value);
    roots.pop_frame(fragment);

    assert_eq!(roots.temp_at(outer_registers, 2), outer_value);
}
