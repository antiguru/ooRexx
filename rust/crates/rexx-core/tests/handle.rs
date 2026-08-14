use rexx_core::{Decoded, GENERATION_MAX, INLINE_TEXT, ObjRef};

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

/// Every length the encoding admits, over bytes chosen to break a decoder
/// that assumes text.
///
/// **`0x00` and `0xFF` are in the alphabet on purpose.** A Rexx string may
/// contain a NUL -- `'00'x` is an ordinary one-byte value -- so an encoding
/// that recovered the length from a terminator rather than from its own
/// field would pass over ASCII and lose data here. That is why the length is
/// stored, and this is what says so.
#[test]
fn a_short_string_round_trips_through_the_handle_at_every_length() {
    let alphabet: [u8; 6] = [0x00, 0x01, b'a', b'~', 0x7f, 0xff];
    for len in 0..=INLINE_TEXT {
        for (offset, _) in alphabet.iter().enumerate() {
            let source: Vec<u8> = (0..len).map(|i| alphabet[(i + offset) % 6]).collect();
            let handle = ObjRef::inline_text(&source).expect("fits");
            let Decoded::Text(inline) = handle.decode() else {
                panic!("carried in the handle at {len}, got {:?}", handle.decode())
            };
            assert_eq!(&*inline, &source[..], "bytes at {len} from {offset}");
            assert_eq!(inline.len(), len);
        }
    }
}

/// One byte more is refused rather than truncated.
///
/// The adjacent success is the length above it, so this pins a boundary and
/// not merely "long strings are refused".
#[test]
fn a_string_one_byte_too_long_is_refused_rather_than_truncated() {
    let fits = vec![b'q'; INLINE_TEXT];
    let over = vec![b'q'; INLINE_TEXT + 1];
    assert!(ObjRef::inline_text(&fits).is_some());
    assert_eq!(ObjRef::inline_text(&over), None);
}

/// Distinct strings are distinct handles, including strings that differ only
/// in length.
///
/// **`""`, `"\0"` and `"\0\0"` are the case that matters**: their byte
/// payloads are all zero, so only the length field tells them apart, and an
/// encoding that dropped it would collapse all three onto one handle -- and
/// onto `NIL`, whose payload is also zero.
#[test]
fn strings_differing_only_in_length_are_different_handles() {
    let handles: Vec<ObjRef> = [&b""[..], &b"\0"[..], &b"\0\0"[..], &b"a"[..], &b"aa"[..]]
        .iter()
        .map(|s| ObjRef::inline_text(s).expect("fits"))
        .collect();
    for (i, left) in handles.iter().enumerate() {
        for (j, right) in handles.iter().enumerate() {
            assert_eq!(i == j, left == right, "handles {i} and {j}");
        }
    }
}

/// The new tag does not collide with the three that were already there.
///
/// `NIL` is the one to watch: the null string's payload is entirely zero,
/// and `NIL`'s is too, so only the tag separates them.
#[test]
fn an_inline_string_is_not_a_slot_an_integer_or_nil() {
    let empty = ObjRef::inline_text(b"").expect("fits");
    assert_ne!(empty, ObjRef::NIL);
    assert_eq!(ObjRef::NIL.decode(), Decoded::Nil);
    for handle in [empty, ObjRef::inline_text(b"abc").expect("fits")] {
        assert!(matches!(handle.decode(), Decoded::Text(_)));
        assert_ne!(handle, ObjRef::heap(0, 0));
        assert_ne!(handle, ObjRef::heap(u32::MAX, GENERATION_MAX));
        assert_ne!(handle, ObjRef::small_int(0).expect("fits"));
        assert_ne!(handle, ObjRef::small_int(-1).expect("fits"));
    }
}
