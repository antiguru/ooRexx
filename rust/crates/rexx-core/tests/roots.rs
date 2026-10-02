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
    let frame = roots.activity_mut().push_frame();
    roots.activity_mut().push_temp(tmp);
    assert!(roots.iter().any(|r| r == tmp));
    roots.activity_mut().pop_frame(frame);
    assert!(!roots.iter().any(|r| r == tmp));
}

#[test]
fn popping_an_outer_frame_discards_the_inner_frames_it_contains() {
    let mut roots = RootSet::new();
    let outer = roots.activity_mut().push_frame();
    roots.activity_mut().push_temp(ObjRef::heap(1, 0));
    let _inner = roots.activity_mut().push_frame();
    roots.activity_mut().push_temp(ObjRef::heap(2, 0));
    roots.activity_mut().pop_frame(outer);
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

/// Registers live in the frame arena, so the variable frame beneath them is
/// free to grow.
#[test]
fn registers_leave_the_variable_frame_growable() {
    let mut roots = RootSet::new();
    let variables = roots.push_slots(2);
    let arena = roots.activity().frames();
    let registers = arena.reserve(4);
    let live = ObjRef::heap(7, 0);
    registers.set(3, live);

    let grown = roots.activity_mut().grow_slots_of(variables, 1);
    roots.set_frame_slot(variables, grown, ObjRef::heap(8, 0));

    assert_eq!(registers.get(3), live);
    assert!(roots.iter().any(|r| r == live));
    arena.release(registers);
}

/// A register is a root for as long as its frame is reserved, and stops being
/// one when the frame is released -- the property that makes intermediates
/// safe to hold in registers at all.
#[test]
fn registers_are_roots_until_their_frame_is_released() {
    let roots = RootSet::new();
    let arena = roots.activity().frames();
    let registers = arena.reserve(2);
    let held = ObjRef::heap(9, 0);
    registers.set(1, held);
    assert!(roots.iter().any(|r| r == held));
    arena.release(registers);
    assert!(!roots.iter().any(|r| r == held));
}

/// Register frames nest, which is what a fragment chunk compiled and run
/// inside an already-running chunk needs.
#[test]
fn a_nested_register_frame_leaves_the_enclosing_one_intact() {
    let roots = RootSet::new();
    let arena = roots.activity().frames();
    let outer = arena.reserve(3);
    let outer_value = ObjRef::heap(11, 0);
    outer.set(2, outer_value);

    let inner = arena.reserve(2);
    inner.set(0, ObjRef::heap(12, 0));
    assert_eq!(outer.get(2), outer_value);
    arena.release(inner);

    assert_eq!(outer.get(2), outer_value);
    assert!(roots.iter().any(|r| r == outer_value));
    assert!(!roots.iter().any(|r| r == ObjRef::heap(12, 0)));
    arena.release(outer);
}

/// The block size is fixed before the first frame; afterwards it is refused.
#[test]
#[should_panic(expected = "the frame block size is set before the first frame")]
fn the_frame_block_size_cannot_change_under_a_reserved_frame() {
    let mut roots = RootSet::new();
    roots
        .activity_mut()
        .set_frame_block(rexx_core::FrameBlock::new(8).expect("in range"));
    let arena = roots.activity().frames();
    arena.release(arena.reserve(1));
    drop(arena);
    roots
        .activity_mut()
        .set_frame_block(rexx_core::FrameBlock::DEFAULT);
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
    roots.activity_mut().pop_slots(inner);
    assert_eq!(roots.slot_value(cell), Some(held));
    assert!(roots.iter().any(|r| r == held));
    let written = ObjRef::heap(8, 0);
    roots.set_slot_value(cell, written);
    assert_eq!(roots.slot_value(cell), Some(written));
    roots.activity_mut().pop_slots(outer);
}

/// An exposed name promotes the storage it was exposed *from*, so the
/// exposing frame sees a write through the cell.
#[test]
fn promoting_an_exposed_name_promotes_the_storage_behind_it() {
    let mut roots = RootSet::new();
    let outer = roots.push_slots(1);
    let inner = roots.push_slots(1);
    let target = roots.activity().slot_ref(outer, 0);
    roots.activity_mut().alias_slot(inner, 0, target);
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
    let target = roots.activity().slot_ref(outer, 0);
    roots.activity_mut().alias_slot(inner, 0, target);
    let cell = roots.promote(outer, 0);
    let written = ObjRef::heap(4, 0);
    roots.set_slot_value(cell, written);
    assert_eq!(roots.frame_slot(inner, 0), Some(written));
}

/// A frame beneath another grows, and an alias the upper frame holds into
/// it still reaches the same variable afterwards.
#[test]
fn a_frame_beneath_another_grows_and_aliases_into_it_hold() {
    let mut roots = RootSet::new();
    let outer = roots.push_slots(2);
    let inner = roots.push_slots(2);
    let target = roots.activity().slot_ref(outer, 1);
    roots.activity_mut().alias_slot(inner, 0, target);
    roots.set_frame_slot(inner, 1, ObjRef::heap(2, 0));

    let grown = roots.activity_mut().grow_slots_of(outer, 2);
    assert_eq!(grown, 2);
    roots.set_frame_slot(inner, 0, ObjRef::heap(1, 0));
    roots.set_frame_slot_of(outer, grown + 1, ObjRef::heap(3, 0));

    assert_eq!(roots.frame_slot_of(outer, 1), Some(ObjRef::heap(1, 0)));
    assert_eq!(roots.frame_slot(inner, 1), Some(ObjRef::heap(2, 0)));
    assert_eq!(
        roots.frame_slot_of(outer, grown + 1),
        Some(ObjRef::heap(3, 0))
    );
    assert_eq!(roots.frame_slot_of(outer, grown), None);
}

/// The top-frame accessors serve a frame below the top one through its own
/// record.
#[test]
fn the_top_accessors_reach_a_lower_frame() {
    let mut roots = RootSet::new();
    let outer = roots.push_slots(1);
    let inner = roots.push_slots(1);
    roots.set_frame_slot(outer, 0, ObjRef::heap(4, 0));
    roots.set_frame_slot(inner, 0, ObjRef::heap(5, 0));
    assert_eq!(roots.frame_slot(outer, 0), Some(ObjRef::heap(4, 0)));
    assert_eq!(roots.frame_slot(inner, 0), Some(ObjRef::heap(5, 0)));
    roots.clear_frame_slot(outer, 0);
    assert_eq!(roots.frame_slot(outer, 0), None);
    assert_eq!(roots.frame_slot(inner, 0), Some(ObjRef::heap(5, 0)));
}

/// A handle kept past its frame's pop is refused against the frame opened
/// at the same depth afterwards, by the top-frame accessors as well.
#[test]
#[should_panic(expected = "resolved against another frame's record")]
fn a_closed_frames_handle_is_refused_by_its_successor() {
    let mut roots = RootSet::new();
    let first = roots.push_slots(1);
    roots.activity_mut().pop_slots(first);
    let _second = roots.push_slots(1);
    roots.frame_slot(first, 0);
}

/// An alias must lead to an older slot, which is what ends the chase.
#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "does not descend")]
fn an_alias_that_does_not_descend_is_refused() {
    let mut roots = RootSet::new();
    let outer = roots.push_slots(1);
    let inner = roots.push_slots(1);
    let target = roots.activity().slot_ref(inner, 0);
    roots.activity_mut().alias_slot(outer, 0, target);
    roots.frame_slot(outer, 0);
}

/// A moved frame keeps its values, its promoted cells and its serial, and
/// leaves the frame beneath it in the source untouched.
#[test]
fn a_moved_frame_keeps_its_values_and_cells() {
    let mut roots = RootSet::new();
    let outer = roots.push_slots(1);
    let kept = ObjRef::heap(1, 0);
    roots.set_frame_slot(outer, 0, kept);
    let frame = roots.push_slots(3);
    let plain = ObjRef::heap(2, 0);
    let celled = ObjRef::heap(3, 0);
    roots.set_frame_slot(frame, 0, plain);
    roots.set_frame_slot(frame, 2, celled);
    let cell = roots.promote(frame, 2);
    let mut other = rexx_core::ActivityRoots::new();
    let moved = roots.activity_mut().move_frame(frame, &mut other);
    assert_eq!(roots.activity().live_frames(), 1);
    assert_eq!(roots.frame_slot(outer, 0), Some(kept));
    std::mem::swap(roots.activity_mut(), &mut other);
    assert_eq!(roots.activity().frame_len(moved), 3);
    assert_eq!(roots.frame_slot(moved, 0), Some(plain));
    assert_eq!(roots.frame_slot(moved, 1), None);
    assert_eq!(roots.frame_slot(moved, 2), Some(celled));
    let written = ObjRef::heap(4, 0);
    roots.set_frame_slot(moved, 2, written);
    assert_eq!(roots.slot_value(cell), Some(written));
    assert!(roots.iter().any(|r| r == plain));
}

/// The handle a frame had before its move is refused by its new record.
#[test]
#[should_panic]
fn a_moved_frames_old_handle_is_refused() {
    let mut roots = RootSet::new();
    let _outer = roots.push_slots(1);
    let frame = roots.push_slots(1);
    let mut other = rexx_core::ActivityRoots::new();
    roots.activity_mut().move_frame(frame, &mut other);
    std::mem::swap(roots.activity_mut(), &mut other);
    roots.frame_slot(frame, 0);
}

/// A frame one of whose slots aliases another slot cannot move.
#[test]
#[should_panic(expected = "aliases a slot")]
fn a_frame_aliasing_a_slot_does_not_move() {
    let mut roots = RootSet::new();
    let outer = roots.push_slots(1);
    let inner = roots.push_slots(1);
    let target = roots.activity().slot_ref(outer, 0);
    roots.activity_mut().alias_slot(inner, 0, target);
    let mut other = rexx_core::ActivityRoots::new();
    roots.activity_mut().move_frame(inner, &mut other);
}
