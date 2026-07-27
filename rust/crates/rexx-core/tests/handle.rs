use rexx_core::{Decoded, ObjRef};

#[test]
fn heap_handles_round_trip() {
    for slot in [0u32, 1, 1000, u32::MAX] {
        for generation in [0u32, 1, 7, rexx_core::GENERATION_MAX] {
            assert_eq!(
                ObjRef::heap(slot, generation).decode(),
                Decoded::Heap { slot, generation }
            );
        }
    }
}

#[test]
fn the_same_slot_at_different_generations_is_a_different_handle() {
    assert_ne!(ObjRef::heap(4, 0), ObjRef::heap(4, 1));
}

#[test]
fn small_integers_are_encoded_inline_without_allocating() {
    for value in [0i64, 1, -1, 42, -42, (1 << 60) - 1, -(1 << 60)] {
        let r = ObjRef::small_int(value).expect("fits in the tagged range");
        assert_eq!(r.decode(), Decoded::SmallInt(value));
    }
}

#[test]
fn integers_outside_the_tagged_range_are_rejected_rather_than_truncated() {
    assert_eq!(ObjRef::small_int(i64::MAX), None);
    assert_eq!(ObjRef::small_int(i64::MIN), None);
}

#[test]
fn nil_is_distinct_from_every_heap_slot_and_every_integer() {
    assert_eq!(ObjRef::NIL.decode(), Decoded::Nil);
    assert_ne!(ObjRef::NIL, ObjRef::heap(0, 0));
    assert_ne!(ObjRef::NIL, ObjRef::small_int(0).unwrap());
}
