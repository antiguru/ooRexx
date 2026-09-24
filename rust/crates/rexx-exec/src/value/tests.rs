/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 2026 Rexx Language Association. All rights reserved.          */
/*                                                                            */
/* This program and the accompanying materials are made available under       */
/* the terms of the Common Public License v1.0 which accompanies this         */
/* distribution. A copy is also available at the following address:           */
/* https://www.oorexx.org/license.html                                        */
/*                                                                            */
/*----------------------------------------------------------------------------*/

use super::*;
use rexx_core::INLINE_TEXT;
use rexx_num::DivOp;

/// A buffer holding `bytes` at the constructor's default capacity.
fn buffer_state(bytes: &[u8]) -> BufferState {
    BufferState {
        bytes: bytes.to_vec(),
        capacity: bytes.len().max(256),
        default_size: 256,
    }
}

/// `to_text` renders a tagged integer exactly as `i64`'s `Display` does,
/// and the scratch it renders into is wide enough for every one of them.
#[test]
fn a_tagged_integer_renders_as_its_own_display_does() {
    let mut interp = Interp::new();
    for case in [
        0i64,
        1,
        -1,
        9,
        -9,
        10,
        -10,
        99,
        -100,
        1000,
        i64::from(i32::MIN),
        i64::from(u32::MAX),
        SMALL_INT_MAX,
        SMALL_INT_MIN,
    ] {
        let value = ObjRef::small_int(case).expect("inside the tagged range");
        assert_eq!(
            interp.to_text(value).as_ref(),
            case.to_string().as_bytes(),
            "{case}"
        );
    }
    assert_eq!(
        SMALL_INT_MIN.to_string().len(),
        TEXT_SCRATCH,
        "the widest tagged rendering is what fixes `TEXT_SCRATCH`; if this \
         moved, the buffer has slack or is about to overrun"
    );
}

/// A remembered parse answers what a fresh one answers, for every
/// spelling a handle can carry -- numeric and not, at every slot the
/// table has.
#[test]
fn a_remembered_parse_answers_what_a_fresh_one_does() {
    let spellings: Vec<Vec<u8>> = [
        "0", "1", "-1", "7", "05", "+5", " 5 ", "1.1", "2.2", "1.50", "99.7", "1e2", "1E-2", "67",
        "1234567", "-999999", "foobar", "Key", "?", "string", "", "1.2.3", "0x1f", "e", ".5",
        "-.5", "1.",
    ]
    .iter()
    .map(|s| s.as_bytes().to_vec())
    .collect();
    // Every one of them has to fit the handle, or it would reach
    // `heap_to_number` and this would be testing the wrong cache.
    for spelling in &spellings {
        assert!(
            spelling.len() <= INLINE_TEXT,
            "{spelling:?} does not fit a handle"
        );
    }

    let mut interp = Interp::new();
    let mut answered = 0usize;
    for _pass in 0..3 {
        for spelling in &spellings {
            for other in &spellings {
                let handle = ObjRef::inline_text(other).expect("checked to fit the handle above");
                let _ = interp.to_number(handle);
            }
            let handle = ObjRef::inline_text(spelling).expect("checked to fit the handle above");
            let expected = Number::parse_bytes(spelling).ok_or(NotNumeric);
            assert_eq!(
                interp.to_number(handle),
                expected,
                "{:?}",
                String::from_utf8_lossy(spelling)
            );
            answered += 1;
        }
    }
    // A floor, so a grid that stopped generating cases fails rather than
    // passes empty.
    assert_eq!(answered, spellings.len() * 3);
}

/// Two spellings that land in the same slot each keep their own answer.
#[test]
fn two_spellings_sharing_a_slot_each_keep_their_own_answer() {
    let candidates: Vec<Vec<u8>> = (0..4000u32)
        .map(|n| n.to_string().into_bytes())
        .filter(|s| s.len() <= INLINE_TEXT)
        .collect();
    let mut collision = None;
    'search: for (at, first) in candidates.iter().enumerate() {
        let first_handle = ObjRef::inline_text(first).expect("short enough");
        for second in &candidates[at + 1..] {
            let second_handle = ObjRef::inline_text(second).expect("short enough");
            if TextNumbers::slot(first_handle) == TextNumbers::slot(second_handle) {
                collision = Some((first_handle, first.clone(), second_handle, second.clone()));
                break 'search;
            }
        }
    }
    let (first_handle, first, second_handle, second) =
        collision.expect("two of four thousand spellings share one of thirty-two slots");

    let mut interp = Interp::new();
    for _round in 0..3 {
        assert_eq!(
            interp.to_number(first_handle),
            Number::parse_bytes(&first).ok_or(NotNumeric),
            "{:?}",
            String::from_utf8_lossy(&first)
        );
        assert_eq!(
            interp.to_number(second_handle),
            Number::parse_bytes(&second).ok_or(NotNumeric),
            "{:?}",
            String::from_utf8_lossy(&second)
        );
    }
}

/// Two reads in a row each answer their own value.
#[test]
fn a_second_tagged_read_does_not_show_the_first_ones_tail() {
    let mut interp = Interp::new();
    let long = ObjRef::small_int(SMALL_INT_MIN).expect("inside the tagged range");
    let short = ObjRef::small_int(7).expect("inside the tagged range");
    assert_eq!(
        interp.to_text(long).to_vec(),
        SMALL_INT_MIN.to_string().into_bytes()
    );
    assert_eq!(interp.to_text(short).to_vec(), b"7".to_vec());
}

/// `write_text` must append exactly the bytes `to_text` renders, for every
/// value that reaches its hand-written formatter.
#[test]
fn write_text_writes_what_to_text_renders() {
    let mut interp = Interp::new();
    let cases: [i64; 13] = [
        0,
        1,
        -1,
        9,
        -9,
        10,
        -10,
        99,
        -100,
        i64::from(u32::MAX),
        -i64::from(u32::MAX),
        SMALL_INT_MAX,
        SMALL_INT_MIN,
    ];
    for case in cases {
        let value = ObjRef::small_int(case).expect("inside the tagged range");
        let expected = interp.to_text(value).to_vec();
        assert_eq!(
            expected,
            case.to_string().into_bytes(),
            "the case itself must render as its own decimal spelling: {case}"
        );
        let mut out = b"KEY.".to_vec();
        interp.write_text(value, &mut out);
        assert_eq!(
            out,
            [b"KEY.".as_slice(), &expected].concat(),
            "write_text must append what to_text renders, for {case}"
        );
    }
}

/// A test-only shorthand: every literal here is a number by construction,
/// so a parse failure is this test's own bug, not a case to handle.
fn n(text: &str) -> Number {
    Number::parse(text).expect("test literal parses")
}

/// `text_len` answers what `to_text` measures, for every value kind that
/// can be built here.
#[test]
fn text_len_agrees_with_to_text() {
    let mut interp = Interp::new();
    let mut values: Vec<ObjRef> = vec![ObjRef::NIL];
    for case in [0i64, 1, -1, 9, -10, 1000, SMALL_INT_MAX, SMALL_INT_MIN] {
        values.push(ObjRef::small_int(case).expect("inside the tagged range"));
    }
    for spelling in ["", "a", "abcdef", "abcdefg"] {
        values.push(interp.text(spelling.as_bytes()));
    }
    // Past `INLINE_TEXT` and past `INLINE_BYTES`, so both a slot-inlined
    // and a heap-allocated `Body::Text` are measured.
    values.push(interp.text(&[b'q'; INLINE_TEXT + 1]));
    values.push(interp.text(&[b'q'; INLINE_BYTES + 1]));

    let mut numbers = 0usize;
    for spelling in [
        "0",
        "1.50",
        "-1.50",
        "0.05",
        "-0.05",
        "12.4",
        "99.6",
        "1E+9",
        "1E-9",
        "1E+30",
        "1E-30",
        "10E-19",
        "123456789012",
        "-123456789012",
        "0.000012345",
        "123456789012345678901234567890",
    ] {
        for digits in [1u32, 2, 5, 9, 18, 40] {
            for form in [Form::Scientific, Form::Engineering] {
                let value = interp.number(n(spelling), digits, form);
                // A `Body::Num`, not a tagged integer, is what this arm
                // is about -- the tag path is already in the list above.
                if matches!(value.decode(), Decoded::Heap { .. }) {
                    numbers += 1;
                }
                values.push(value);
            }
        }
    }

    // A stem with no default renders its own name, and one with a default
    // renders through it -- a second `text_len` hop, which is the only
    // recursive arm.
    let bare = interp.alloc_with(
        BehaviourId::STEM,
        Body::Stem {
            name: b"Q.".to_vec().into(),
            default: None,
            tails: rexx_core::NameMap::default(),
        },
    );
    values.push(bare);
    let default = interp.number(n("1.50"), 9, Form::Scientific);
    let defaulted = interp.alloc_with(
        BehaviourId::STEM,
        Body::Stem {
            name: b"W.".to_vec().into(),
            default: Some(default),
            tails: rexx_core::NameMap::default(),
        },
    );
    values.push(defaulted);

    // An array's string value is joined on demand and cached nowhere,
    // which is the redirect arm both functions take.
    let joined = interp.text(b"1");
    let array = interp.alloc_with(BehaviourId::ARRAY, Body::array(vec![Some(joined), None]));
    values.push(array);

    // Both shapes of instance, because `Redirect::of` answers only for
    // the unnamed one: the named one reaches the body match, which is the
    // half of the mirror a redirect arm cannot stand in for. A buffer in
    // each shape, because it takes neither: its own arm answers the
    // contents whether or not the instance is named, and a `.Pointer` is
    // in that same position with a rendering nothing stores.
    let class = interp
        .classes()
        .lookup("Object")
        .expect("the Object class is registered");
    let behaviour = interp.classes().instance_behaviour_handle(class);
    for name in [None, Some(b"123".to_vec().into_boxed_slice())] {
        for native in [
            None,
            Some(Box::new(rexx_core::NativeState::Buffer(buffer_state(
                b"held for long enough to need a slot",
            )))),
            Some(Box::new(rexx_core::NativeState::Pointer(
                std::ptr::null_mut(),
            ))),
            Some(Box::new(rexx_core::NativeState::Pointer(
                std::ptr::without_provenance_mut(0x7f01_2345_6789),
            ))),
        ] {
            let instance = interp.alloc_with(
                BehaviourId::OBJECT,
                Body::Instance {
                    class,
                    behaviour,
                    name: name.clone(),
                    pools: rexx_core::ScopePools::new(),
                    own: None,
                    native,
                },
            );
            values.push(instance);
        }
    }

    // **`text_len` is asked first, before anything renders**, so that a
    // `Body::Num` reaches its arm with an empty `text` cache at least
    // once. Asking `to_text` first would fill that cache, after which
    // both functions read the same stored bytes and the rendering half of
    // the arm is never exercised -- measured, a mutation replacing its
    // `created_digits` with a constant left this test green when the two
    // calls were the other way round. The second pass is the cached
    // reading, deliberately, and it is the *second*.
    for pass in 0..2 {
        for value in &values {
            let got = interp.text_len(*value);
            let expected = interp.to_text(*value).len();
            assert_eq!(got, expected, "pass {pass}, {value:?}");
        }
    }
    // Floors, because every assertion above is inside a loop.
    assert!(
        values.len() > 100,
        "only {} values were built",
        values.len()
    );
    assert!(
        numbers > 50,
        "only {numbers} of the grid became a `Body::Num`"
    );
}

/// The tag decision is the rendering's, at the tag's own boundary.
#[test]
fn the_tag_test_agrees_with_parsing_the_same_bytes() {
    fn reference(bytes: &[u8]) -> Option<i64> {
        let (negative, digits) = match bytes.split_first() {
            Some((b'-', rest)) => (true, rest),
            _ => (false, bytes),
        };
        if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
            return None;
        }
        if digits[0] == b'0' && (digits.len() > 1 || negative) {
            return None;
        }
        let magnitude: i64 = std::str::from_utf8(digits).ok()?.parse().ok()?;
        let value = if negative { -magnitude } else { magnitude };
        (SMALL_INT_MIN..=SMALL_INT_MAX)
            .contains(&value)
            .then_some(value)
    }

    let mut cases: Vec<Vec<u8>> = vec![
        b"".to_vec(),
        b"-".to_vec(),
        b"0".to_vec(),
        b"-0".to_vec(),
        b"00".to_vec(),
        b"05".to_vec(),
        b"-05".to_vec(),
        b"1".to_vec(),
        b"-1".to_vec(),
        b"9".to_vec(),
        b"10".to_vec(),
        b"123456789".to_vec(),
        b"-123456789".to_vec(),
        b"+1".to_vec(),
        b"1.0".to_vec(),
        b"1e5".to_vec(),
        b"1a".to_vec(),
        b"a1".to_vec(),
        b" 1".to_vec(),
        b"1 ".to_vec(),
        b"1\xff".to_vec(),
        b"\xff1".to_vec(),
        b"99999999999999999999999999".to_vec(),
        b"-99999999999999999999999999".to_vec(),
    ];
    for edge in [
        i64::MAX,
        i64::MIN,
        SMALL_INT_MAX,
        SMALL_INT_MIN,
        SMALL_INT_MAX - 1,
        SMALL_INT_MIN + 1,
    ] {
        cases.push(edge.to_string().into_bytes());
    }
    // One past each tag limit, and one past `i64::MAX`, spelled out
    // because they are not themselves representable in the type.
    for spelled in [
        "2305843009213693952",
        "-2305843009213693953",
        "9223372036854775808",
        "-9223372036854775809",
    ] {
        cases.push(spelled.as_bytes().to_vec());
    }

    let mut answered = 0usize;
    for case in &cases {
        let got = canonical_small_int(case);
        assert_eq!(got, reference(case), "{:?}", String::from_utf8_lossy(case));
        if got.is_some() {
            answered += 1;
        }
    }
    assert!(
        answered > 0,
        "every case was refused, so the agreement above is vacuous"
    );
}

#[test]
fn the_tag_decision_is_the_rendering_read_back() {
    fn by_rendering(value: &Number, created_digits: u32) -> Option<i64> {
        let rendered = value.format_form(u64::from(created_digits), Form::Scientific);
        if rendered.contains('.') || rendered.contains('E') {
            return None;
        }
        let whole: i64 = rendered.parse().ok()?;
        (SMALL_INT_MIN..=SMALL_INT_MAX)
            .contains(&whole)
            .then_some(whole)
    }

    let spellings = [
        "0",
        "0.00",
        "-0",
        "1",
        "-1",
        "5",
        "1.50",
        "150",
        "1.50E+2",
        "12.0",
        // Values whose *rounding* is the integer, which is the half of
        // the set `plain_integer` cannot see: dropped digit below and
        // above five, a carry, and both signs.
        "1.4",
        "2.5",
        "12.4",
        "12.5",
        "-12.4",
        "-12.5",
        "19.6",
        "99.4",
        "99.6",
        "9.996",
        "123.456",
        "1234.5678",
        "2305843009213693951.4",
        "2305843009213693952.4",
        "1.2E+3",
        "999",
        "999.5",
        "1000",
        "1E+9",
        "1E+18",
        "1E+19",
        "0.001",
        "1E-5",
        "-12345",
        "123456789012345678",
        // The tag's limits, either side, and one past every `i64`.
        "2305843009213693950",
        "2305843009213693951",
        "2305843009213693952",
        "-2305843009213693952",
        "-2305843009213693953",
        "9223372036854775807",
        "9999999999999999999999",
    ];
    let precisions: [u32; 12] = [1, 2, 3, 4, 5, 9, 18, 19, 20, 21, 22, 25];

    let mut tagged = 0usize;
    let mut tagged_beyond_plain_integer = 0usize;
    for spelling in spellings {
        for digits in precisions {
            let value = n(spelling);
            let decided = small_int_for(&value, digits);
            assert_eq!(
                decided,
                by_rendering(&value, digits),
                "{spelling} at DIGITS {digits}"
            );
            if decided.is_some() {
                tagged += 1;
                if value.plain_integer(u64::from(digits)).is_none() {
                    tagged_beyond_plain_integer += 1;
                }
            }
        }
    }
    assert!(tagged > 100, "only {tagged} of the grid was tagged at all");
    assert!(
        tagged_beyond_plain_integer > 10,
        "only {tagged_beyond_plain_integer} tagged cases are outside `plain_integer`, \
         so this asserts almost nothing about the values a narrower rule would drop"
    );
}

/// `text_owned` keeps the caller's buffer instead of copying it.
#[test]
fn text_owned_takes_the_buffer_rather_than_copying_it() {
    let long = vec![b'q'; INLINE_BYTES + 1];
    let mut interp = Interp::new();
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(long.len())
        .expect("the buffer is available");
    bytes.extend_from_slice(&long);
    let address = bytes.as_ptr();

    let value = interp.text_owned(bytes);
    assert_eq!(interp.to_text(value).as_ptr(), address);
    assert_eq!(&*interp.to_text(value), &long[..]);
}

/// The two construction entry points and the boundary between the arms,
/// held at the interpreter's own surface rather than only inside `Bytes`.
#[test]
fn a_short_value_holds_its_bytes_in_the_slot_and_a_long_one_does_not() {
    for len in [
        INLINE_TEXT + 1,
        INLINE_BYTES - 1,
        INLINE_BYTES,
        INLINE_BYTES + 1,
        400,
    ] {
        let source: Vec<u8> = (0..len).map(|i| b'a' + (i % 26) as u8).collect();
        let mut interp = Interp::new();

        for value in [interp.text(&source), interp.text_owned(source.clone())] {
            assert_eq!(&*interp.to_text(value), &source[..], "bytes at {len}");
            let Decoded::Heap { slot, .. } = value.decode() else {
                panic!("a heap string at {len}")
            };
            let inline = interp
                .heap
                .get(value)
                .is_some_and(|object| match &object.body {
                    Body::Text { bytes, .. } => bytes.is_inline(),
                    other => panic!("a Body::Text at {len}, got {other:?}"),
                });
            assert_eq!(inline, len <= INLINE_BYTES, "arm at {len}, slot {slot}");
        }
    }
}

// The plan's own draft sketched these five tests as calls to
// `interp.eval_str(...)`/`interp.eval_with(...)`, which do not exist and
// cannot without pulling Task 6's `Activation`/`Settings` and Task 7's
// arithmetic forward into this task -- scaffolding Task 7 would then have
// to unwind. So every `Number` below is built directly through
// `rexx-num`, which Phase 2 already tested; what these tests check is
// only what `text`/`number`/`to_text`/`to_number` do with it afterwards.
// The oracle transcripts remain the source of truth and are re-run
// against `build/bin/rexx` in the task report.

#[test]
fn a_numbers_rendering_is_fixed_when_it_is_created() {
    // numeric digits 9 ; y = 1 / 3 ; numeric digits 3 ; say y
    //   -> 0.333333333 (not 0.333)
    //                   z = 1 / 3 ; say z -> 0.333
    let mut interp = Interp::new();
    let y = interp.number(
        n("1").div(&n("3"), 9, DivOp::Divide).unwrap(),
        9,
        Form::Scientific,
    );
    let z = interp.number(
        n("1").div(&n("3"), 3, DivOp::Divide).unwrap(),
        3,
        Form::Scientific,
    );
    assert_eq!(&*interp.to_text(y), b"0.333333333");
    assert_eq!(&*interp.to_text(z), b"0.333");
    // Reading `y` again proves the cache is genuinely reused, not just
    // right the first time: nothing about creating or reading `z`
    // afterwards moved it.
    assert_eq!(&*interp.to_text(y), b"0.333333333");
}

#[test]
fn numeric_form_is_captured_at_creation_too() {
    // numeric form engineering ; x = 1e10 + 0 ; say x -> 10E+9
    // numeric form scientific  ;               say x -> 10E+9 (unchanged)
    //                            y = 1e10 + 0 ; say y -> 1E+10
    let mut interp = Interp::new();
    let sum = n("1e10").add(&n("0"), 9).unwrap();
    let x = interp.number(sum.clone(), 9, Form::Engineering);
    assert_eq!(&*interp.to_text(x), b"10E+9");
    let y = interp.number(sum, 9, Form::Scientific);
    assert_eq!(&*interp.to_text(x), b"10E+9", "x's own form must not move");
    assert_eq!(&*interp.to_text(y), b"1E+10");
}

#[test]
fn a_small_int_is_only_admissible_within_the_digits_of_its_own_operation() {
    // numeric digits 1 ; x = 15 + 0 ; x is 20, so x + 6 is 3E+1 while
    // 15 + 6 is 2E+1.
    let mut interp = Interp::new();
    let x_number = n("15").add(&n("0"), 1).unwrap();
    let x = interp.number(x_number.clone(), 1, Form::Scientific);
    assert_eq!(&*interp.to_text(x), b"2E+1");

    let sum = interp.number(x_number.add(&n("6"), 1).unwrap(), 1, Form::Scientific);
    assert_eq!(&*interp.to_text(sum), b"3E+1");

    let direct = interp.number(n("15").add(&n("6"), 1).unwrap(), 1, Form::Scientific);
    assert_eq!(&*interp.to_text(direct), b"2E+1");
}

#[test]
fn small_int_admissibility_is_checked_once_against_the_producing_operations_digits() {
    // D15's own discriminating pair: two SmallInt-shaped results that
    // differ observably. Checking the actual `ObjRef` shape, not only the
    // rendered bytes, is the point -- a wrongly-admitted `SmallInt(20)`
    // and a correctly-refused one can render identically to `to_text` if
    // `to_text`'s two branches happen to agree, and only the tag itself
    // tells them apart.
    let mut interp = Interp::new();
    let a = interp.number(n("20"), 9, Form::Scientific);
    assert!(matches!(a.decode(), Decoded::SmallInt(20)));
    assert_eq!(&*interp.to_text(a), b"20");

    let b = interp.number(n("15").add(&n("0"), 1).unwrap(), 1, Form::Scientific);
    assert!(matches!(b.decode(), Decoded::Heap { .. }));
    assert_eq!(&*interp.to_text(b), b"2E+1");
}

#[test]
fn text_keeps_its_own_spelling_and_caches_an_exact_parse() {
    // x = '007' ; say x -> 007 ; say x + 0 -> 7
    let mut interp = Interp::new();
    let x = interp.text(b"007");
    assert_eq!(&*interp.to_text(x), b"007");

    let parsed = interp.to_number(x).unwrap();
    let converted = interp.number(parsed.add(&n("0"), 9).unwrap(), 9, Form::Scientific);
    assert_eq!(&*interp.to_text(converted), b"7");
}

#[test]
fn the_text_cache_holds_the_exact_parse_not_a_rounded_one() {
    // x = '1.234567890123456789'. The SAME stored parse renders 1.2346
    // at DIGITS 5 and the full nineteen-digit value at DIGITS 20.
    // Measured against the oracle: this is `x + 0` at each DIGITS, not
    // `say x` alone -- a `Body::Text`'s identity is its own bytes, so
    // `say x` never converts it at all and prints the literal unchanged
    // at every DIGITS (`t4b.rex`/`t4c.rex` in the task report).
    let mut interp = Interp::new();
    let x = interp.text(b"1.234567890123456789");

    let rounded_5 = interp.to_number(x).unwrap().add(&n("0"), 5).unwrap();
    let at_5 = interp.number(rounded_5, 5, Form::Scientific);
    assert_eq!(&*interp.to_text(at_5), b"1.2346");

    // Reads the cache a second time, under a completely different
    // DIGITS, to show it is still the exact original parse and not
    // whatever the first read happened to round it to.
    let rounded_20 = interp.to_number(x).unwrap().add(&n("0"), 20).unwrap();
    let at_20 = interp.number(rounded_20, 20, Form::Scientific);
    assert_eq!(&*interp.to_text(at_20), b"1.234567890123456789");
}

/// Every shape a value can take, read through the borrowing pair, gives
/// the bytes `to_text` gives.
#[test]
fn the_borrowing_accessor_answers_what_to_text_answers() {
    let mut interp = Interp::new();

    let short = interp.text(b"abc");
    let long = interp.text(&[b'z'; INLINE_BYTES + 40]);
    let small = interp.number(n("7"), 9, Form::Scientific);
    let heap_number = interp.number(n("1.5"), 9, Form::Scientific);
    let rendered_number = interp.number(n("2.5"), 9, Form::Scientific);
    let _ = interp.to_text(rendered_number);

    let five = interp.number(n("5"), 9, Form::Scientific);
    let with_default = interp.alloc_with(
        BehaviourId::STEM,
        Body::Stem {
            name: b"A.".to_vec().into(),
            default: Some(five),
            tails: rexx_core::NameMap::default(),
        },
    );
    let bare = interp.alloc_with(
        BehaviourId::STEM,
        Body::Stem {
            name: b"Q.".to_vec().into(),
            default: None,
            tails: rexx_core::NameMap::default(),
        },
    );
    let aliasing = interp.alloc_with(
        BehaviourId::STEM,
        Body::Stem {
            name: b"B.".to_vec().into(),
            default: Some(with_default),
            tails: rexx_core::NameMap::default(),
        },
    );

    for value in [
        ObjRef::NIL,
        short,
        long,
        small,
        heap_number,
        rendered_number,
        with_default,
        bare,
        aliasing,
    ] {
        let rendered = interp.render(value);
        let borrowed = rendered.text(&interp).to_vec();
        let expected = interp.to_text(value).into_owned();
        assert_eq!(
            String::from_utf8_lossy(&borrowed),
            String::from_utf8_lossy(&expected),
            "{value:?}"
        );
    }
}

/// `try_text` answers `None` for exactly two causes, and both are "there
/// is nothing to borrow" rather than "there is no text".
#[test]
fn try_text_answers_only_where_the_bytes_already_exist() {
    let mut interp = Interp::new();

    // Cause one: the digits of a tagged small integer are stored nowhere
    // at all, so nothing renders it into existence and `render` has to
    // carry them.
    let small = interp.number(n("7"), 9, Form::Scientific);
    assert!(matches!(small.decode(), Decoded::SmallInt(7)));
    assert_eq!(interp.try_text(small), None);
    let _ = interp.to_text(small);
    assert_eq!(
        interp.try_text(small),
        None,
        "rendering a SmallInt stores nothing"
    );

    // Cause two: a heap number's `text` cache is empty until something
    // fills it, and filling it is the `&mut` this pair exists to move.
    let number = interp.number(n("1.5"), 9, Form::Scientific);
    assert_eq!(interp.try_text(number), None);
    let _ = interp.to_text(number);
    assert_eq!(interp.try_text(number), Some(&b"1.5"[..]));

    // And a stem chasing a default is answered by the *default's* bytes,
    // which is what `to_text` has to copy and this does not -- so it
    // inherits the default's own answer, `None` included.
    let stem = interp.alloc_with(
        BehaviourId::STEM,
        Body::Stem {
            name: b"A.".to_vec().into(),
            default: Some(small),
            tails: rexx_core::NameMap::default(),
        },
    );
    assert_eq!(interp.try_text(stem), None, "the default is a SmallInt");
    // Long enough to have a slot: a shorter one lives in the handle and
    // so has nothing to borrow, which is the case asserted below.
    let text = interp.text(b"held for long enough to need a slot");
    let stem_of_text = interp.alloc_with(
        BehaviourId::STEM,
        Body::Stem {
            name: b"B.".to_vec().into(),
            default: Some(text),
            tails: rexx_core::NameMap::default(),
        },
    );
    assert_eq!(
        interp.try_text(stem_of_text),
        Some(&b"held for long enough to need a slot"[..])
    );

    // The third cause of `None`, and the one this encoding adds: the
    // bytes are in the handle, which is a `Copy` local, so a shared
    // borrow has nothing to point at. `to_text` still answers.
    let inline = interp.text(b"held");
    assert!(matches!(inline.decode(), Decoded::Text(_)));
    assert_eq!(interp.try_text(inline), None, "the bytes are the handle");
    assert_eq!(&*interp.to_text(inline), b"held");

    // A buffer's bytes are the object's own storage, borrowable whether
    // or not the instance is named -- the name never wins over them.
    let class = interp
        .classes()
        .lookup("Object")
        .expect("the Object class is registered");
    let behaviour = interp.classes().instance_behaviour_handle(class);
    for name in [None, Some(b"named".to_vec().into_boxed_slice())] {
        let buffer = interp.alloc_with(
            BehaviourId::OBJECT,
            Body::Instance {
                class,
                behaviour,
                name,
                pools: rexx_core::ScopePools::new(),
                own: None,
                native: Some(Box::new(rexx_core::NativeState::Buffer(buffer_state(
                    b"abcdef",
                )))),
            },
        );
        assert_eq!(interp.try_text(buffer), Some(&b"abcdef"[..]));
        assert_eq!(&*interp.to_text(buffer), b"abcdef");
        assert_eq!(interp.text_len(buffer), 6);
    }
}

/// The other side of that boundary: a value short enough occupies no
/// slot at all.
#[test]
fn a_value_short_enough_is_the_handle_and_has_no_slot() {
    for len in 0..=INLINE_TEXT {
        let source: Vec<u8> = (0..len).map(|i| b'a' + (i % 26) as u8).collect();
        let mut interp = Interp::new();
        let before = interp.heap.live_count();

        for value in [interp.text(&source), interp.text_owned(source.clone())] {
            assert!(
                matches!(value.decode(), Decoded::Text(_)),
                "carried in the handle at {len}, got {:?}",
                value.decode()
            );
            assert_eq!(&*interp.to_text(value), &source[..], "bytes at {len}");
        }
        assert_eq!(
            interp.heap.live_count(),
            before,
            "no slot taken at {len}, on either entry point"
        );
    }

    // One byte more is a heap object again, which is what says the test
    // above is reading a boundary rather than a constant.
    let over: Vec<u8> = vec![b'z'; INLINE_TEXT + 1];
    let mut interp = Interp::new();
    let value = interp.text(&over);
    assert!(matches!(value.decode(), Decoded::Heap { .. }));
}

#[test]
fn nil_has_a_string_value_and_the_booleans_are_plain_strings() {
    // say .nil -> The NIL object ; .true is "1" ; .false is "0"
    let mut interp = Interp::new();
    assert_eq!(&*interp.to_text(ObjRef::NIL), b"The NIL object");

    // `.true`/`.false` need no representation of their own (D15): they
    // are the one-byte strings "1" and "0", built the same way any other
    // text value is.
    let true_value = interp.text(b"1");
    let false_value = interp.text(b"0");
    assert_eq!(&*interp.to_text(true_value), b"1");
    assert_eq!(&*interp.to_text(false_value), b"0");
}

#[test]
fn nonnumeric_text_and_nil_both_collapse_to_not_numeric() {
    let mut interp = Interp::new();
    let words = interp.text(b"not a number");
    assert_eq!(interp.to_number(words), Err(NotNumeric));
    assert_eq!(interp.to_number(ObjRef::NIL), Err(NotNumeric));
}

// Branch review F5 (Critical): `to_number` on a `Body::Stem` used to hit
// the `unreachable!` fallback and abort the process (rc 101) --
// `to_text` already redirected through a stem's default/name, `to_number`
// never learned to. These three tests build a `Body::Stem` directly
// (the same shape `stem.rs`'s own allocations use) and drive it straight
// through `to_number`, with no live activation needed for that alone.

#[test]
fn to_number_on_a_stem_redirects_through_its_numeric_default() {
    // a. = 5 ; say a. + 1 -> 6 (measured against the oracle). Kills the
    // mutation that reverts this arm to `unreachable!`: that mutant
    // panics this test instead of returning `Ok(5)`.
    let mut interp = Interp::new();
    let five = interp.number(n("5"), 9, Form::Scientific);
    let stem = interp.alloc_with(
        BehaviourId::STEM,
        Body::Stem {
            name: b"A.".to_vec().into(),
            default: Some(five),
            tails: rexx_core::NameMap::default(),
        },
    );
    let value = interp.to_number(stem).unwrap();
    let sum = interp.number(value.add(&n("1"), 9).unwrap(), 9, Form::Scientific);
    assert_eq!(&*interp.to_text(sum), b"6");
}

#[test]
fn to_number_on_a_defaultless_stem_parses_its_own_name_and_fails() {
    // say q. + 1 on a stem nobody ever assigned a default to: the
    // oracle raises 41.1, nonnumeric value "Q." (measured). Kills a
    // mutation that makes the `default: None` arm return some fixed
    // `Ok` (e.g. `Ok(0)`) instead of parsing the object's own name and
    // reporting `NotNumeric` when, as here, that name is not one.
    let mut interp = Interp::new();
    let stem = interp.alloc_with(
        BehaviourId::STEM,
        Body::Stem {
            name: b"Q.".to_vec().into(),
            default: None,
            tails: rexx_core::NameMap::default(),
        },
    );
    assert_eq!(interp.to_number(stem), Err(NotNumeric));
    // Rendering is untouched by this fix -- still the derived name.
    assert_eq!(&*interp.to_text(stem), b"Q.");
}

#[test]
fn to_number_on_a_stem_aliasing_another_stem_chases_through_both() {
    // a. = 5 ; b. = a. ; say b. + 1 -> 6, the aliasing shape
    // `stem_assign` actually produces (`stem.rs`): `b.`'s default is
    // itself a `Body::Stem`, never a copy of `a.`'s value. Kills a
    // mutation that only redirects one level deep (e.g. matching
    // `Body::Num`/`Body::Text` inline instead of recursing through
    // `to_number` again), which would panic on the second hop.
    let mut interp = Interp::new();
    let five = interp.number(n("5"), 9, Form::Scientific);
    let a = interp.alloc_with(
        BehaviourId::STEM,
        Body::Stem {
            name: b"A.".to_vec().into(),
            default: Some(five),
            tails: rexx_core::NameMap::default(),
        },
    );
    let b = interp.alloc_with(
        BehaviourId::STEM,
        Body::Stem {
            name: b"B.".to_vec().into(),
            default: Some(a),
            tails: rexx_core::NameMap::default(),
        },
    );
    let value = interp.to_number(b).unwrap();
    let sum = interp.number(value.add(&n("1"), 9).unwrap(), 9, Form::Scientific);
    assert_eq!(&*interp.to_text(sum), b"6");
}
