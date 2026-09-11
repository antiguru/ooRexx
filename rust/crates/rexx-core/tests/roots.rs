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

/// The rejected design, and why: a register `SlotFrame` of its own makes
/// every run-time variable introduction underneath it panic.
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
#[test]
fn registers_in_the_temps_region_leave_the_variable_frame_growable() {
    let mut roots = RootSet::new();
    let variables = roots.push_slots(2);
    let registers = roots.reserve_temps(4);
    let live = ObjRef::heap(7, 0);
    roots.set_temp(registers, 3, live);

    // The growth the rejected design could not survive.
    let grown = roots.grow_slots(variables);
    roots.set_frame_slot(variables, grown, ObjRef::heap(8, 0));

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

/// A promoted variable keeps answering by name, through the redirect the
/// promotion leaves behind.
#[test]
fn a_promoted_variable_still_reads_and_writes_by_name() {
    let mut roots = RootSet::new();
    let frame = roots.push_slots(2);
    let first = ObjRef::heap(1, 0);
    roots.set_frame_slot(frame, 0, first);
    let cell = roots.promote(frame, 0);
    assert_eq!(roots.frame_slot(frame, 0), Some(first));
    assert_eq!(roots.slot_value(cell), Some(first));
    let second = ObjRef::heap(2, 0);
    roots.set_frame_slot(frame, 0, second);
    assert_eq!(roots.slot_value(cell), Some(second));
    roots.set_slot_value(cell, first);
    assert_eq!(roots.frame_slot(frame, 0), Some(first));
}

/// Promotion is idempotent, which is what two `>p` terms on one variable
/// need: a second one answers the same storage rather than a second copy.
#[test]
fn promoting_a_variable_twice_answers_one_cell() {
    let mut roots = RootSet::new();
    let frame = roots.push_slots(1);
    roots.set_frame_slot(frame, 0, ObjRef::heap(1, 0));
    assert_eq!(roots.promote(frame, 0), roots.promote(frame, 0));
}

/// A cell survives the frame that declared the variable, and stays a root --
/// the whole reason promotion exists, since a reference is an ordinary value
/// and may outlive the activation.
#[test]
fn a_cell_outlives_the_frame_it_was_promoted_out_of() {
    let mut roots = RootSet::new();
    let outer = roots.push_slots(1);
    let inner = roots.push_slots(1);
    let held = ObjRef::heap(7, 0);
    roots.set_frame_slot(inner, 0, held);
    let cell = roots.promote(inner, 0);
    roots.pop_slots(inner);
    assert_eq!(roots.slot_value(cell), Some(held));
    assert!(roots.iter().any(|r| r == held));
    let written = ObjRef::heap(8, 0);
    roots.set_slot_value(cell, written);
    assert_eq!(roots.slot_value(cell), Some(written));
    roots.pop_slots(outer);
}

/// An exposed name promotes the storage it was exposed *from*, so the
/// exposing frame sees a write through the cell.
#[test]
fn promoting_an_exposed_name_promotes_the_storage_behind_it() {
    let mut roots = RootSet::new();
    let outer = roots.push_slots(1);
    let inner = roots.push_slots(1);
    roots.alias_slot(inner, 0, roots.slot_ref(outer, 0));
    let cell = roots.promote(inner, 0);
    let written = ObjRef::heap(3, 0);
    roots.set_slot_value(cell, written);
    assert_eq!(roots.frame_slot(outer, 0), Some(written));
    assert_eq!(roots.frame_slot(inner, 0), Some(written));
}

/// An alias bound *before* the promotion still reaches the cell, which is
/// the case `resolve`'s walk exists for: the redirect it recorded points at
/// a position that is now itself redirected.
#[test]
fn an_alias_taken_before_a_promotion_still_reaches_the_cell() {
    let mut roots = RootSet::new();
    let outer = roots.push_slots(1);
    let inner = roots.push_slots(1);
    roots.alias_slot(inner, 0, roots.slot_ref(outer, 0));
    let cell = roots.promote(outer, 0);
    let written = ObjRef::heap(4, 0);
    roots.set_slot_value(cell, written);
    assert_eq!(roots.frame_slot(inner, 0), Some(written));
}
