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

use super::super::dispatch;
use super::{
    BASE64, HEX_DIGITS, base64_digit, decode_base64_bytes, encode_base64_bytes, hex_value,
};
use crate::plan::{BodyKey, ProgramId};
use crate::{Activation, Interp, error::Failure, error::Raised};
use rexx_parse::parse_program;
use std::rc::Rc;

/// An interpreter with a live top-level activation at `NUMERIC DIGITS
/// digits`.
fn interp_at(digits: &str) -> Interp {
    let mut interp = Interp::new();
    let program = Rc::new(parse_program(b"nop".to_vec()).expect("a NOP program parses"));
    let program_id = ProgramId(interp.programs.len());
    interp.programs.push(Rc::clone(&program));
    let plan = interp.plan_for(
        BodyKey {
            program: program_id,
            directive: None,
        },
        &program.main,
        &program.symbols,
        &program.source,
    );
    let frame = interp.roots.push_slots(plan.len());
    let activation = interp.next_activation_id();
    interp.push_activation(Activation::new(
        activation, program, program_id, plan, frame,
    ));
    interp
        .activation_mut()
        .settings
        .set_digits_str(digits)
        .expect("a legal DIGITS setting");
    interp
}

/// Runs `name` over `arguments` at `digits`, each `None` standing for an
/// omitted interior position, and answers the result's own bytes.
fn call_at(digits: &str, name: &[u8], arguments: &[Option<&[u8]>]) -> Result<Vec<u8>, Failure> {
    let mut interp = interp_at(digits);
    let args: Vec<_> = arguments
        .iter()
        .map(|argument| argument.map(|bytes| interp.text(bytes)))
        .collect();
    let result = dispatch(&mut interp, name, &args).expect("a builtin name")?;
    Ok(interp.to_text(result).into_owned())
}

/// [`call_at`] at the default precision.
fn call(name: &[u8], arguments: &[Option<&[u8]>]) -> Result<Vec<u8>, Failure> {
    call_at("9", name, arguments)
}

/// [`call`], for the cases whose answer is the bytes and nothing else.
fn answer(name: &[u8], arguments: &[&[u8]]) -> Vec<u8> {
    let arguments: Vec<_> = arguments.iter().map(|bytes| Some(*bytes)).collect();
    call(name, &arguments).expect("this call succeeds")
}

/// [`answer`] at a chosen precision.
fn answer_at(digits: &str, name: &[u8], arguments: &[&[u8]]) -> Vec<u8> {
    let arguments: Vec<_> = arguments.iter().map(|bytes| Some(*bytes)).collect();
    call_at(digits, name, &arguments).expect("this call succeeds")
}

/// The `(major, sub)` and substitutions of the condition `name` raises.
fn raised(name: &[u8], arguments: &[Option<&[u8]>]) -> (u16, u16, Vec<Vec<u8>>) {
    raised_at("9", name, arguments)
}

/// [`raised`] at a chosen precision.
fn raised_at(digits: &str, name: &[u8], arguments: &[Option<&[u8]>]) -> (u16, u16, Vec<Vec<u8>>) {
    let failure = call_at(digits, name, arguments).expect_err("this call raises");
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    (raised.number, raised.sub, raised.additional)
}

/// The alphabet every case set below draws from: the null string, a byte
/// at or above `0x80`, a control byte, a NUL, and the two that separate
/// groups.
const BYTE_ALPHABET: &[&[u8]] = &[
    b"",
    b"a",
    b"AB",
    &[0x00],
    &[0x01],
    &[0x09],
    &[0x20],
    &[0x7f],
    &[0x80],
    &[0xff],
    &[0x00, 0xff, 0x7f, 0x80],
    &[0x61, 0x00, 0xe9],
];

/// `C2X` writes two upper-case digits per byte over the whole byte range,
/// and `X2C` reverses it.
#[test]
fn c2x_and_x2c_are_inverse_over_every_byte() {
    for byte in 0..=u8::MAX {
        let hex = answer(b"C2X", &[&[byte]]);
        let expected = format!("{byte:02X}").into_bytes();
        assert_eq!(hex, expected, "byte {byte:#04x} did not render");
        assert_eq!(answer(b"X2C", &[&hex]), vec![byte]);
        // Lower case on the way in, upper case on the way out.
        let lowered: Vec<u8> = hex.iter().map(u8::to_ascii_lowercase).collect();
        assert_eq!(answer(b"X2C", &[&lowered]), vec![byte]);
    }
    for subject in BYTE_ALPHABET {
        let hex = answer(b"C2X", &[subject]);
        assert_eq!(hex.len(), subject.len() * 2);
        assert_eq!(&answer(b"X2C", &[&hex])[..], *subject);
    }
    assert_eq!(answer(b"C2X", &[b""]), b"");
    assert_eq!(answer(b"X2C", &[b""]), b"");
    assert_eq!(answer(b"C2X", &[b"Ab"]), b"4162");
    assert_eq!(answer(b"X2C", &[b"616263"]), b"abc");
}

/// The grouping rule: the first group fixes a residue and every later one
/// must be an exact multiple, with the first group left-padded.
#[test]
fn a_grouped_string_carries_its_residue_to_the_end() {
    assert_eq!(answer(b"X2C", &[b"414"]), b"\x04\x14");
    assert_eq!(answer(b"X2C", &[b"4 1424"]), b"\x04\x14\x24");
    assert_eq!(answer(b"X2C", &[b"414 2434"]), b"\x04\x14\x24\x34");
    assert_eq!(answer(b"X2C", &[b"41 42 43"]), b"ABC");
    assert_eq!(answer(b"X2C", &[b"4\t1424"]), b"\x04\x14\x24");
    assert_eq!(raised(b"X2C", &[Some(b"414 243")]), (93, 976, Vec::new()));
    assert_eq!(raised(b"X2C", &[Some(b"4 4")]), (93, 976, Vec::new()));
    assert_eq!(raised(b"X2C", &[Some(b"44 4")]), (93, 976, Vec::new()));

    assert_eq!(answer(b"B2X", &[b"101 0000"]), b"50");
    assert_eq!(answer(b"B2X", &[b"1 0000"]), b"10");
    assert_eq!(answer(b"B2X", &[b"1 0000 0011"]), b"103");
    assert_eq!(answer(b"B2X", &[b"11 0000 0011"]), b"303");
    assert_eq!(answer(b"B2X", &[b"1\t0000"]), b"10");
    assert_eq!(raised(b"B2X", &[Some(b"101 000")]), (93, 977, Vec::new()));
    assert_eq!(raised(b"B2X", &[Some(b"10 10")]), (93, 977, Vec::new()));
    assert_eq!(raised(b"B2X", &[Some(b"1 0 0000")]), (93, 977, Vec::new()));
}

/// The residue is checked **at every gap**, not only once at the end, and
/// these are the strings that can tell the two apart.
#[test]
fn the_residue_is_checked_at_every_gap_and_not_only_at_the_end() {
    for subject in [b"41 4 1 42".as_slice(), b"414 2 434", b"41 4 1 4 2"] {
        assert_eq!(
            raised(b"X2C", &[Some(subject)]),
            (93, 976, Vec::new()),
            "{} broke the residue at an interior gap and was not refused",
            String::from_utf8_lossy(subject)
        );
    }
    for subject in [b"1010 10 10".as_slice(), b"1010 101 0101"] {
        assert_eq!(
            raised(b"B2X", &[Some(subject)]),
            (93, 977, Vec::new()),
            "{} broke the residue at an interior gap and was not refused",
            String::from_utf8_lossy(subject)
        );
    }
    // The adjacent successes, which is what pins the refusals to the
    // interior gap rather than to having more than two groups: each of
    // these holds the same residue at every gap it has.
    assert_eq!(answer(b"X2C", &[b"4 14 24"]), b"\x04\x14\x24");
    assert_eq!(answer(b"X2C", &[b"41 42 43"]), b"ABC");
    assert_eq!(answer(b"B2X", &[b"101 0000 0000"]), b"500");
    assert_eq!(answer(b"B2X", &[b"1010 1010 1010"]), b"AAA");
}

/// Whitespace is blank and tab; every other byte below `0x20`, and every
/// byte at or above `0x80`, is an invalid digit.
#[test]
fn only_blank_and_tab_separate_groups() {
    for byte in 0..=u8::MAX {
        let subject = [b'4', b'1', byte, b'4', b'2'];
        let result = call(b"X2C", &[Some(&subject)]);
        if byte == b' ' || byte == b'\t' {
            assert_eq!(result.expect("a separator converts"), b"AB");
        } else if hex_value(byte).is_some() {
            assert!(result.is_ok(), "byte {byte:#04x} is a hexadecimal digit");
        } else {
            let failure = result.expect_err("this byte is not a hexadecimal digit");
            let Failure::Raised(raised) = failure else {
                panic!("expected Raised");
            };
            assert_eq!((raised.number, raised.sub), (93, 933));
            assert_eq!(raised.additional, vec![vec![byte]]);
        }
    }
    // The binary twin names its own sub-code and carries the byte too.
    assert_eq!(
        raised(b"B2X", &[Some(b"1012")]),
        (93, 934, vec![b"2".to_vec()])
    );
    assert_eq!(
        raised(b"B2X", &[Some(&[b'1', 0xff])]),
        (93, 934, vec![vec![0xff]])
    );
    assert_eq!(answer(b"B2X", &[b"1\t0000"]), b"10");
}

/// Whitespace at either end is the misplaced-whitespace error, and the
/// position it names is the *last* byte of a trailing run.
#[test]
fn whitespace_at_either_end_names_its_position() {
    assert_eq!(
        raised(b"X2C", &[Some(b" 4142")]),
        (93, 931, vec![b"1".to_vec()])
    );
    assert_eq!(
        raised(b"X2C", &[Some(b"4142 ")]),
        (93, 931, vec![b"5".to_vec()])
    );
    assert_eq!(
        raised(b"X2C", &[Some(b"41 42  ")]),
        (93, 931, vec![b"7".to_vec()])
    );
    assert_eq!(
        raised(b"X2C", &[Some(b"4142\t")]),
        (93, 931, vec![b"5".to_vec()])
    );
    assert_eq!(
        raised(b"X2D", &[Some(b" ")]),
        (93, 931, vec![b"1".to_vec()])
    );
    assert_eq!(
        raised(b"B2X", &[Some(b" 1010")]),
        (93, 932, vec![b"1".to_vec()])
    );
    assert_eq!(
        raised(b"B2X", &[Some(b"1010 ")]),
        (93, 932, vec![b"5".to_vec()])
    );
    // The adjacent successes: the same strings with the outer whitespace
    // removed convert, so the refusal is about position and not content.
    assert_eq!(answer(b"X2C", &[b"4142"]), b"AB");
    assert_eq!(answer(b"B2X", &[b"1010"]), b"A");
}

/// `X2C` pads an odd number of nibbles up to a byte and `X2B` does not
/// pad at all, which is the one place the two disagree.
#[test]
fn x2c_pads_an_odd_nibble_where_x2b_does_not() {
    assert_eq!(answer(b"X2C", &[b"4"]), b"\x04");
    assert_eq!(answer(b"X2C", &[b"414"]), b"\x04\x14");
    assert_eq!(answer(b"X2B", &[b"4"]), b"0100");
    assert_eq!(answer(b"X2B", &[b"414"]), b"010000010100");
    assert_eq!(answer(b"X2B", &[b"c3"]), b"11000011");
    assert_eq!(answer(b"X2B", &[b"4 1424"]), b"01000001010000100100");
    assert_eq!(answer(b"X2B", &[b""]), b"");
    // `B2X` is `X2C`-shaped: the leading group is padded up to four bits.
    assert_eq!(answer(b"B2X", &[b"1"]), b"1");
    assert_eq!(answer(b"B2X", &[b"11"]), b"3");
    assert_eq!(answer(b"B2X", &[b"100"]), b"4");
    assert_eq!(answer(b"B2X", &[b"11000011"]), b"C3");
    assert_eq!(answer(b"B2X", &[b"0000"]), b"0");
    assert_eq!(answer(b"B2X", &[b""]), b"");
    // Every nibble value, both ways round.
    for value in 0..16u8 {
        let hex = [HEX_DIGITS[usize::from(value)]];
        let bits: Vec<u8> = (0..4).rev().map(|b| b'0' + ((value >> b) & 1)).collect();
        assert_eq!(answer(b"X2B", &[&hex]), bits);
        assert_eq!(answer(b"B2X", &[&bits]), hex);
    }
}

/// `BASE64` and `base64_digit` are two spellings of one table and only the
/// first is ever used to encode, so they are crossed both ways here --
/// over every digit, and over every byte that is not one. An alphabet
/// gaining a character it cannot decode, or losing one it emits, reddens
/// here rather than waiting for a witness whose text happens to use it.
#[test]
fn every_base64_digit_decodes_to_the_value_that_encoded_it() {
    for value in 0..64u8 {
        assert_eq!(base64_digit(BASE64[usize::from(value)]), Some(value));
    }
    for byte in 0..=u8::MAX {
        assert_eq!(base64_digit(byte).is_some(), BASE64.contains(&byte));
    }
    // Padding is not a digit, which is what lets the decoder rather than
    // the table decide where it is allowed.
    assert_eq!(base64_digit(b'='), None);
}

/// Every byte value through a full group, a padded pair and a padded
/// single, which is the axis the corpus witness cannot sweep.
#[test]
fn every_byte_survives_the_base64_round_trip() {
    for length in 1..=3usize {
        for value in 0..=u8::MAX {
            let source = vec![value; length];
            let encoded = encode_base64_bytes(&source);
            assert_eq!(encoded.len(), 4, "{source:?}");
            assert_eq!(decode_base64_bytes(&encoded).as_deref(), Some(&source[..]));
        }
    }
    assert_eq!(encode_base64_bytes(b""), b"");
    assert_eq!(decode_base64_bytes(b""), Some(Vec::new()));
}

/// The bit builtins pass the longer string's tail through when no pad is
/// supplied, and combine it with the pad when one is.
#[test]
fn an_omitted_pad_leaves_the_longer_strings_tail_alone() {
    assert_eq!(answer(b"BITAND", &[b"\xff\xff", b"\x00"]), b"\x00\xff");
    assert_eq!(answer(b"BITAND", &[b"\x00", b"\xff\xff"]), b"\x00\xff");
    assert_eq!(
        answer(b"BITAND", &[b"\xff\xff", b"\x00", b"\x00"]),
        b"\x00\x00"
    );
    assert_eq!(answer(b"BITOR", &[b"\xff\xff", b"\x00"]), b"\xff\xff");
    assert_eq!(answer(b"BITOR", &[b"\x00\x00", b"\xff"]), b"\xff\x00");
    assert_eq!(answer(b"BITXOR", &[b"\xff\xff", b"\x00"]), b"\xff\xff");
    assert_eq!(answer(b"BITXOR", &[b"\x00\x00", b"\xff"]), b"\xff\x00");
    // One argument: every byte reaches the pad path.
    assert_eq!(answer(b"BITAND", &[b"\xff\xff"]), b"\xff\xff");
    assert_eq!(answer(b"BITOR", &[b"\x00\x00"]), b"\x00\x00");
    assert_eq!(answer(b"BITXOR", &[b"\x00\x00"]), b"\x00\x00");
    assert_eq!(answer(b"BITAND", &[b"abc"]), b"abc");
    // A null second string is the same thing as no second string.
    for name in [b"BITAND".as_slice(), b"BITOR", b"BITXOR"] {
        assert_eq!(answer(name, &[b"abc", b""]), b"abc");
        assert_eq!(answer(name, &[b"", b"abc"]), b"abc");
        assert_eq!(answer(name, &[b"", b""]), b"");
        assert_eq!(answer(name, &[b""]), b"");
    }
    // The documented examples, which pin the operations themselves.
    assert_eq!(answer(b"BITAND", &[b"cat", b"DOG"]), b"@AD");
    assert_eq!(answer(b"BITOR", &[b"cat", b"DOG"]), b"gow");
    assert_eq!(answer(b"BITXOR", &[b"cat", b"   "]), b"CAT");
    // The pad crossed with the tail, over the byte alphabet.
    for pad in [0x00u8, 0xff, 0x0f, 0x80, 0x01] {
        for subject in BYTE_ALPHABET {
            let and = answer(b"BITAND", &[subject, b"", &[pad]]);
            let or = answer(b"BITOR", &[subject, b"", &[pad]]);
            let xor = answer(b"BITXOR", &[subject, b"", &[pad]]);
            let expect = |op: fn(u8, u8) -> u8| -> Vec<u8> {
                subject.iter().map(|byte| op(*byte, pad)).collect()
            };
            assert_eq!(and, expect(|a, b| a & b));
            assert_eq!(or, expect(|a, b| a | b));
            assert_eq!(xor, expect(|a, b| a ^ b));
        }
    }
    // An interior omission past the minimum is legal and reaches the pad.
    assert_eq!(
        call(b"BITAND", &[Some(b"\xff\xff"), None, Some(b"\x00")])
            .expect("an omitted second string is legal"),
        b"\x00\x00"
    );
}

/// A length argument is a right-aligned window that truncates from the
/// left, and the top of what survives is a sign bit.
#[test]
fn a_length_makes_the_read_a_signed_window() {
    assert_eq!(answer(b"C2D", &[b"\x80"]), b"128");
    assert_eq!(answer(b"C2D", &[b"\x80", b"1"]), b"-128");
    assert_eq!(answer(b"C2D", &[b"\x80", b"2"]), b"128");
    assert_eq!(answer(b"X2D", &[b"80"]), b"128");
    assert_eq!(answer(b"X2D", &[b"80", b"1"]), b"0");
    assert_eq!(answer(b"X2D", &[b"80", b"2"]), b"-128");

    assert_eq!(answer(b"C2D", &[b"\x01\x02\x03\x04", b"2"]), b"772");
    assert_eq!(answer(b"C2D", &[b"\x7f", b"1"]), b"127");
    assert_eq!(answer(b"C2D", &[b"\xff\x00", b"2"]), b"-256");
    assert_eq!(answer(b"C2D", &[b"\xff\x00", b"1"]), b"0");
    assert_eq!(answer(b"C2D", &[b"\x00\x80", b"2"]), b"128");
    assert_eq!(answer(b"C2D", &[b"\x01", b"5"]), b"1");
    assert_eq!(answer(b"C2D", &[b"\xff", b"5"]), b"255");

    assert_eq!(answer(b"X2D", &[b"8f", b"1"]), b"-1");
    assert_eq!(answer(b"X2D", &[b"18f", b"1"]), b"-1");
    assert_eq!(answer(b"X2D", &[b"18f", b"2"]), b"-113");
    assert_eq!(answer(b"X2D", &[b"18f", b"3"]), b"399");
    assert_eq!(answer(b"X2D", &[b"ff00", b"2"]), b"0");
    assert_eq!(answer(b"X2D", &[b"ff00", b"3"]), b"-256");
    assert_eq!(answer(b"X2D", &[b"ff00", b"4"]), b"-256");
    assert_eq!(answer(b"X2D", &[b"ff00", b"5"]), b"65280");
    assert_eq!(answer(b"X2D", &[b"f", b"2"]), b"15");
    assert_eq!(answer(b"X2D", &[b"ff", b"5"]), b"255");

    // A length of zero answers zero without validating anything, which is
    // the one shape that separates the shortcut from the scan.
    assert_eq!(answer(b"C2D", &[b"abc", b"0"]), b"0");
    assert_eq!(answer(b"X2D", &[b"zz", b"0"]), b"0");
    assert_eq!(answer(b"C2D", &[b""]), b"0");
    assert_eq!(answer(b"X2D", &[b""]), b"0");
    assert_eq!(raised(b"X2D", &[Some(b"zz"), Some(b"4")]).1, 933);
    // The adjacent legality: an omitted length past the minimum.
    assert_eq!(
        call(b"C2D", &[Some(b"\xff"), None]).expect("an omitted length is legal"),
        b"255"
    );
}

/// A length is the width of a `D2X`/`D2C` result, padded on the left with
/// `0` or `F` and truncated on the left when it is too small.
#[test]
fn a_d2x_length_pads_and_truncates_on_the_left() {
    assert_eq!(answer(b"D2X", &[b"1"]), b"1");
    assert_eq!(answer(b"D2X", &[b"255"]), b"FF");
    assert_eq!(answer(b"D2X", &[b"255", b"1"]), b"F");
    assert_eq!(answer(b"D2X", &[b"255", b"5"]), b"000FF");
    assert_eq!(answer(b"D2X", &[b"4096", b"2"]), b"00");
    assert_eq!(answer(b"D2X", &[b"-1", b"3"]), b"FFF");
    assert_eq!(answer(b"D2X", &[b"-255", b"3"]), b"F01");
    assert_eq!(answer(b"D2X", &[b"-1", b"1"]), b"F");
    assert_eq!(answer(b"D2X", &[b"-16", b"1"]), b"0");
    assert_eq!(answer(b"D2X", &[b"-16", b"2"]), b"F0");
    assert_eq!(answer(b"D2X", &[b"0"]), b"0");
    assert_eq!(answer(b"D2X", &[b"0", b"3"]), b"000");
    assert_eq!(answer(b"D2X", &[b"0", b"0"]), b"");
    assert_eq!(answer(b"D2X", &[b"255", b"0"]), b"");

    assert_eq!(answer(b"D2C", &[b"1"]), b"\x01");
    assert_eq!(answer(b"D2C", &[b"0"]), b"\x00");
    assert_eq!(answer(b"D2C", &[b"256"]), b"\x01\x00");
    assert_eq!(answer(b"D2C", &[b"16706"]), b"AB");
    assert_eq!(answer(b"D2C", &[b"255", b"1"]), b"\xff");
    assert_eq!(answer(b"D2C", &[b"255", b"3"]), b"\x00\x00\xff");
    assert_eq!(answer(b"D2C", &[b"-1", b"1"]), b"\xff");
    assert_eq!(answer(b"D2C", &[b"-1", b"3"]), b"\xff\xff\xff");
    assert_eq!(answer(b"D2C", &[b"-65536", b"4"]), b"\xff\xff\x00\x00");
    assert_eq!(answer(b"D2C", &[b"0", b"3"]), b"\x00\x00\x00");
    assert_eq!(answer(b"D2C", &[b"255", b"0"]), b"");

    // A negative value with no length at all, both of them.
    assert_eq!(raised(b"D2X", &[Some(b"-1")]), (93, 927, Vec::new()));
    assert_eq!(raised(b"D2C", &[Some(b"-1")]), (93, 927, Vec::new()));
    // The adjacent success is the same value with any length at all.
    assert_eq!(answer(b"D2X", &[b"-1", b"1"]), b"F");
    // And a non-negative value needs none.
    assert_eq!(answer(b"D2X", &[b"1"]), b"1");
    assert_eq!(
        call(b"D2X", &[Some(b"1"), None]).expect("an omitted length is legal"),
        b"1"
    );
}

/// `NUMERIC DIGITS` bounds `C2D`/`X2D` on the result and `D2C`/`D2X` on
/// the value, and the four sub-codes are distinct.
#[test]
fn the_precision_bounds_the_result_one_way_and_the_value_the_other() {
    assert_eq!(answer_at("9", b"C2D", &[b"\x00\x00\x00\x00\x00\x01"]), b"1");
    assert_eq!(
        raised_at("9", b"C2D", &[Some(b"\xff\xff\xff\xff")]),
        (93, 936, vec![b"9".to_vec()])
    );
    assert_eq!(answer_at("1", b"C2D", &[b"\xff", b"1"]), b"-1");
    assert_eq!(
        raised_at("1", b"C2D", &[Some(b"\x7f"), Some(b"1")]),
        (93, 936, vec![b"1".to_vec()])
    );
    assert_eq!(answer_at("1", b"C2D", &[b"\x00"]), b"0");
    assert_eq!(answer_at("1", b"C2D", &[b"\x00\x00\x00\x00\x00"]), b"0");
    assert_eq!(answer_at("3", b"C2D", &[b"\xff"]), b"255");
    assert_eq!(
        raised_at("3", b"C2D", &[Some(b"\xff\xff")]),
        (93, 936, vec![b"3".to_vec()])
    );
    assert_eq!(answer_at("3", b"X2D", &[b"ff"]), b"255");
    assert_eq!(
        raised_at("3", b"X2D", &[Some(b"ffff")]),
        (93, 935, vec![b"3".to_vec()])
    );
    assert_eq!(answer_at("1", b"X2D", &[b"ff", b"2"]), b"-1");
    assert_eq!(
        raised_at("1", b"X2D", &[Some(b"8f"), Some(b"2")]),
        (93, 935, vec![b"1".to_vec()])
    );

    assert_eq!(answer_at("3", b"D2X", &[b"000123"]), b"7B");
    assert_eq!(answer_at("3", b"D2X", &[b"999"]), b"3E7");
    assert_eq!(answer_at("3", b"D2X", &[b"1E2"]), b"64");
    assert_eq!(
        raised_at("3", b"D2X", &[Some(b"1000")]),
        (93, 928, vec![b"1000".to_vec()])
    );
    assert_eq!(
        raised_at("3", b"D2X", &[Some(b"1E3")]),
        (93, 928, vec![b"1E3".to_vec()])
    );
    assert_eq!(
        raised_at("3", b"D2X", &[Some(b"12300")]),
        (93, 928, vec![b"12300".to_vec()])
    );
    assert_eq!(
        raised_at("3", b"D2C", &[Some(b"12300")]),
        (93, 929, vec![b"12300".to_vec()])
    );
    assert_eq!(answer_at("5", b"D2X", &[b"12345"]), b"3039");
    assert_eq!(answer_at("9", b"D2X", &[b"1E8"]), b"5F5E100");
    assert_eq!(
        raised_at("9", b"D2X", &[Some(b"1E9")]),
        (93, 928, vec![b"1E9".to_vec()])
    );
}

/// A value with decimals converts when the decimals are insignificant
/// *within the current precision*, which makes the same argument legal at
/// one setting and an error at the next.
#[test]
fn decimals_are_significant_relative_to_the_precision() {
    assert_eq!(answer_at("1", b"D2X", &[b"1.4"]), b"1");
    assert_eq!(
        raised_at("2", b"D2X", &[Some(b"1.4")]),
        (93, 928, vec![b"1.4".to_vec()])
    );
    assert_eq!(
        raised_at("1", b"D2X", &[Some(b"1.6")]),
        (93, 928, vec![b"1.6".to_vec()])
    );
    assert_eq!(answer_at("2", b"D2X", &[b"1.04"]), b"1");
    assert_eq!(
        raised_at("3", b"D2X", &[Some(b"1.04")]),
        (93, 928, vec![b"1.04".to_vec()])
    );
    assert_eq!(answer(b"D2X", &[b"1.0"]), b"1");
    assert_eq!(answer(b"D2X", &[b"1.0000000000004"]), b"1");
    assert_eq!(
        raised(b"D2X", &[Some(b"1.50")]),
        (93, 928, vec![b"1.50".to_vec()])
    );
    assert_eq!(answer(b"D2X", &[b"1200E-2"]), b"C");
    assert_eq!(
        raised(b"D2X", &[Some(b"1234E-2")]),
        (93, 928, vec![b"1234E-2".to_vec()])
    );
    assert_eq!(answer(b"D2X", &[b"1.23E4"]), b"300C");
    assert_eq!(answer(b"D2X", &[b"12.00"]), b"C");
    // Every spelling of zero is the same value, and it is whole.
    for zero in [b"0".as_slice(), b"0.0", b"0.00000", b"-0.0", b"0E5"] {
        assert_eq!(answer_at("1", b"D2X", &[zero]), b"0");
    }
    // A non-zero magnitude below one tenth is never whole, whatever the
    // precision.
    for tiny in [
        b"0.5".as_slice(),
        b"0.01",
        b"1E-9",
        b"1E-100",
        b"0.0000000001",
        b"123E-4",
    ] {
        assert_eq!(raised_at("1", b"D2X", &[Some(tiny)]).1, 928);
        assert_eq!(raised_at("20", b"D2X", &[Some(tiny)]).1, 928);
    }
    // The generous spellings the conversion does accept.
    assert_eq!(answer(b"D2X", &[b"  12  "]), b"C");
    assert_eq!(answer(b"D2X", &[b"+12"]), b"C");
    assert_eq!(answer(b"D2X", &[b"- 12", b"4"]), b"FFF4");
    assert_eq!(answer(b"D2X", &[b"\t12\t"]), b"C");
}

/// The scan `D2X`/`D2C` split their argument with accepts every text the
/// crate's own number parser does.
#[test]
fn a_scan_takes_apart_every_text_the_number_parser_accepts() {
    let subjects: &[&[u8]] = &[
        b"",
        b"0",
        b"1",
        b"-1",
        b"+1",
        b"1.5",
        b".5",
        b"5.",
        b"1e2",
        b"1E+2",
        b"1E-2",
        b"1e",
        b"1e+",
        b"1.2.3",
        b"1 2",
        b"0x1f",
        b" 12 ",
        b"\t12\t",
        b"+ 3",
        b"  +   3  ",
        b"+ .5",
        b"- 12",
        b"+",
        b"-",
        b".",
        b"abc",
        b"1E999999999999",
        b"1E-999999999999",
        b"000123",
        b"12300",
        b"0.0",
        b"1\n2",
        &[0xff],
        &[b'1', 0x00],
        &[b'1', 0x80],
    ];
    let mut interp = interp_at("9");
    let mut parsed_count = 0usize;
    for subject in subjects {
        let value = interp.text(subject);
        let parsed = interp.to_number(value).is_ok();
        let scanned = super::scan_decimal(subject).is_some();
        assert!(
            scanned || !parsed,
            "{:?} is a number the scan cannot take apart",
            String::from_utf8_lossy(subject)
        );
        parsed_count += usize::from(parsed);
    }

    // Without this line the assertion above is satisfied by a parser that
    // accepts nothing: `!parsed` is then true for every subject and the
    // loop asserts an implication with a false antecedent every time,
    // staying green while the thing it exists to check is entirely
    // broken. The count is what makes the antecedent real. It is exact
    // rather than a floor because `subjects` is a literal in this same
    // function, so the number cannot move without someone editing the
    // list directly above it.
    assert_eq!(
        parsed_count,
        19,
        "{parsed_count} of the {} subjects parse as numbers; the agreement asserted above is \
         only worth having over the ones that do",
        subjects.len()
    );
}

/// `XRANGE`'s twelve class tables, including the one that begins with a
/// NUL.
#[test]
fn every_character_class_is_the_oracles_own_table() {
    let mut cntrl: Vec<u8> = (0x00..=0x1fu8).collect();
    cntrl.push(0x7f);
    assert_eq!(cntrl.len(), 33);
    assert_eq!(answer(b"XRANGE", &[b"cntrl"]), cntrl);
    assert_eq!(answer(b"XRANGE", &[b"CNTRL"]), cntrl);
    assert_eq!(answer(b"XRANGE", &[b"CnTrL"]), cntrl);

    assert_eq!(answer(b"XRANGE", &[b"digit"]), b"0123456789");
    assert_eq!(answer(b"XRANGE", &[b"blank"]), b"\t ");
    assert_eq!(answer(b"XRANGE", &[b"space"]), b"\t\n\x0b\x0c\r ");
    assert_eq!(
        answer(b"XRANGE", &[b"upper"]),
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    );
    assert_eq!(
        answer(b"XRANGE", &[b"lower"]),
        b"abcdefghijklmnopqrstuvwxyz"
    );
    assert_eq!(
        answer(b"XRANGE", &[b"alpha"]),
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
    );
    assert_eq!(
        answer(b"XRANGE", &[b"alnum"]),
        b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
    );
    assert_eq!(answer(b"XRANGE", &[b"xdigit"]), b"0123456789ABCDEFabcdef");
    assert_eq!(
        answer(b"XRANGE", &[b"punct"]),
        b"!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~"
    );
    // `graph` and `print` differ by exactly the leading blank.
    let graph = answer(b"XRANGE", &[b"graph"]);
    let print = answer(b"XRANGE", &[b"print"]);
    assert_eq!(graph.len(), 94);
    assert_eq!(print.len(), 95);
    assert_eq!(print[0], b' ');
    assert_eq!(&print[1..], &graph[..]);
    // Every table is in ascending byte order, which is the property the
    // oracle's own comment claims for all twelve.
    for (name, _) in super::CHARACTER_CLASSES {
        let table = answer(b"XRANGE", &[name]);
        assert!(
            table.windows(2).all(|pair| pair[0] < pair[1]),
            "{} is not in ascending byte order",
            String::from_utf8_lossy(name)
        );
    }
}

/// `XRANGE` is variadic over pairs, and the wrap-around is on the byte.
#[test]
fn xrange_concatenates_ranges_and_wraps_at_the_top() {
    assert_eq!(answer(b"XRANGE", &[b"a", b"e"]), b"abcde");
    assert_eq!(answer(b"XRANGE", &[b"a", b"a"]), b"a");
    assert_eq!(answer(b"XRANGE", &[b"a", b"b", b"c", b"d"]), b"abcd");
    assert_eq!(
        answer(b"XRANGE", &[b"a", b"b", b"c", b"d", b"e", b"f", b"g", b"h"]),
        b"abcdefgh"
    );
    assert_eq!(answer(b"XRANGE", &[b"\x7f", b"\x80"]), b"\x7f\x80");
    assert_eq!(answer(b"XRANGE", &[b"\xff", b"\x00"]), b"\xff\x00");
    assert_eq!(answer(b"XRANGE", &[b"\xfe", b"\x01"]), b"\xfe\xff\x00\x01");
    assert_eq!(answer(b"XRANGE", &[b"\xff", b"\xff"]), b"\xff");
    assert_eq!(answer(b"XRANGE", &[b"\x80", b"\x7f"]).len(), 256);

    let all: Vec<u8> = (0..=u8::MAX).collect();
    assert_eq!(call(b"XRANGE", &[]).expect("no arguments is legal"), all);
    assert_eq!(answer(b"XRANGE", &[b"\x00", b"\xff"]), all);
    // An omitted end is `'ff'x` and an omitted start is `'00'x`.
    assert_eq!(answer(b"XRANGE", &[b"\x00"]), all);
    assert_eq!(
        call(b"XRANGE", &[None, Some(b"\x04")]).expect("an omitted start is legal"),
        b"\x00\x01\x02\x03\x04"
    );
    assert_eq!(
        call(b"XRANGE", &[Some(b"a"), None, Some(b"c"), Some(b"d")])
            .expect("an omitted end is legal"),
        {
            let mut expected: Vec<u8> = (b'a'..=0xff).collect();
            expected.extend_from_slice(b"cd");
            expected
        }
    );
}

/// A class name and a start/end pair mix, and the oracle's own early
/// return discards a class when the whole call is two arguments.
#[test]
fn a_two_argument_call_ending_in_a_range_drops_a_preceding_class() {
    let from_z: Vec<u8> = (b'z'..=0xff).collect();
    assert_eq!(answer(b"XRANGE", &[b"digit", b"z"]), from_z);
    assert_eq!(answer(b"XRANGE", &[b"cntrl", b"z"]), from_z);
    assert_eq!(from_z.len(), 134);

    // Add a third argument and the class comes back.
    let mut expected = b"0123456789".to_vec();
    expected.extend((b'z'..=0xff).chain(0x00..=b'q'));
    assert_eq!(answer(b"XRANGE", &[b"digit", b"z", b"q"]), expected);

    // Two class names never reach the early return at all.
    let mut alpha = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ".to_vec();
    alpha.extend_from_slice(b"abcdefghijklmnopqrstuvwxyz");
    assert_eq!(answer(b"XRANGE", &[b"upper", b"lower"]), alpha);
    let mut alnum = b"0123456789".to_vec();
    alnum.extend_from_slice(&alpha);
    assert_eq!(answer(b"XRANGE", &[b"digit", b"Alpha"]), alnum);
    // A class after a range, and a range after a class, with three or
    // more arguments.
    let mut ab_digits = b"ab".to_vec();
    ab_digits.extend_from_slice(b"0123456789");
    assert_eq!(answer(b"XRANGE", &[b"a", b"b", b"digit"]), ab_digits);
    let mut upper_a_z = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ".to_vec();
    upper_a_z.extend_from_slice(b"abcdefghijklmnopqrstuvwxyz");
    assert_eq!(answer(b"XRANGE", &[b"upper", b"a", b"z"]), upper_a_z);
    // One class on its own answers the class -- and *only* when the call
    // really is one argument long, which is why `xrange('digit',)` is
    // ten bytes: the parser drops the trailing omission before the count
    // is taken.
    assert_eq!(answer(b"XRANGE", &[b"digit"]), b"0123456789");
    assert_eq!(
        call(b"XRANGE", &[Some(b"digit"), None])
            .expect("a second position that is present but omitted is legal")
            .len(),
        256,
        "an omitted second argument is a start/end pair, not a missing one"
    );
}

/// `XRANGE`'s two argument positions take different things, and say so
/// with different sub-codes.
#[test]
fn xrange_names_the_position_that_is_wrong_and_what_it_wanted() {
    assert_eq!(
        raised(b"XRANGE", &[Some(b"zork")]),
        (
            40,
            28,
            vec![b"XRANGE".to_vec(), b"1".to_vec(), b"zork".to_vec()]
        )
    );
    assert_eq!(
        raised(b"XRANGE", &[Some(b"")]),
        (40, 28, vec![b"XRANGE".to_vec(), b"1".to_vec(), Vec::new()])
    );
    assert_eq!(
        raised(b"XRANGE", &[Some(b"c1"), Some(b"c1")]),
        (
            40,
            28,
            vec![b"XRANGE".to_vec(), b"1".to_vec(), b"c1".to_vec()]
        )
    );
    assert_eq!(
        raised(b"XRANGE", &[Some(b"a"), Some(b"zz")]),
        (
            40,
            23,
            vec![b"XRANGE".to_vec(), b"2".to_vec(), b"zz".to_vec()]
        )
    );
    assert_eq!(
        raised(b"XRANGE", &[Some(b"a"), Some(b"")]),
        (40, 23, vec![b"XRANGE".to_vec(), b"2".to_vec(), Vec::new()])
    );
    // A class name is not a legal *end*, which is what makes the two
    // positions genuinely different rather than the same check twice.
    assert_eq!(
        raised(b"XRANGE", &[Some(b"a"), Some(b"upper")]),
        (
            40,
            23,
            vec![b"XRANGE".to_vec(), b"2".to_vec(), b"upper".to_vec()]
        )
    );
    // The position the message names is the call's own, past the first
    // pair.
    assert_eq!(
        raised(b"XRANGE", &[Some(b"a"), Some(b"b"), Some(b"zork")]),
        (
            40,
            28,
            vec![b"XRANGE".to_vec(), b"3".to_vec(), b"zork".to_vec()]
        )
    );
    assert_eq!(
        raised(
            b"XRANGE",
            &[Some(b"a"), Some(b"b"), Some(b"c"), Some(b"zz")]
        ),
        (
            40,
            23,
            vec![b"XRANGE".to_vec(), b"4".to_vec(), b"zz".to_vec()]
        )
    );
}

/// Every conversion of an argument runs before every check on its
/// content, and the two families answer at different exit codes.
#[test]
fn the_call_layer_is_checked_before_the_operation_layer() {
    // The *length*'s type error beats the value's content error.
    assert_eq!(
        raised(b"D2C", &[Some(b"abc"), Some(b"def")]),
        (
            40,
            12,
            vec![b"D2C".to_vec(), b"2".to_vec(), b"def".to_vec()]
        )
    );
    assert_eq!(
        raised(b"X2D", &[Some(b"ZZ"), Some(b"zz")]),
        (40, 12, vec![b"X2D".to_vec(), b"2".to_vec(), b"zz".to_vec()])
    );
    assert_eq!(
        raised(b"D2X", &[Some(b"abc"), Some(b"zz")]),
        (40, 12, vec![b"D2X".to_vec(), b"2".to_vec(), b"zz".to_vec()])
    );
    // With the length made legal, the content error the 40.12 was hiding.
    assert_eq!(raised(b"X2D", &[Some(b"ZZ"), Some(b"4")]).1, 933);
    assert_eq!(raised(b"D2C", &[Some(b"abc"), Some(b"1")]).1, 929);

    // A negative length is 93.923 -- and for `D2X`/`D2C` it loses to the
    // value's own check, where for `C2D`/`X2D` it wins over the string's.
    assert_eq!(
        raised(b"C2D", &[Some(b"abc"), Some(b"-1")]),
        (93, 923, vec![b"-1".to_vec()])
    );
    assert_eq!(
        raised(b"X2D", &[Some(b"ZZ"), Some(b"-1")]),
        (93, 923, vec![b"-1".to_vec()])
    );
    assert_eq!(
        raised(b"D2X", &[Some(b"1.5"), Some(b"-1")]),
        (93, 923, vec![b"-1".to_vec()])
    );
    assert_eq!(
        raised(b"D2X", &[Some(b"-1"), Some(b"-1")]),
        (93, 923, vec![b"-1".to_vec()])
    );
    assert_eq!(
        raised(b"D2C", &[Some(b"abc"), Some(b"-1")]),
        (93, 929, vec![b"abc".to_vec()])
    );
    assert_eq!(
        raised(b"D2X", &[Some(b"abc"), Some(b"-1")]),
        (93, 928, vec![b"abc".to_vec()])
    );
    assert_eq!(
        raised_at("3", b"D2X", &[Some(b"1000"), Some(b"-1")]),
        (93, 923, vec![b"-1".to_vec()])
    );
    // A pad's own type error, at the position the call spells it.
    assert_eq!(
        raised(b"BITAND", &[Some(b"ab"), Some(b"cd"), Some(b"xx")]),
        (
            40,
            23,
            vec![b"BITAND".to_vec(), b"3".to_vec(), b"xx".to_vec()]
        )
    );
    // A value needing more than the argument precision is not a whole
    // number, however it is spelled.
    assert_eq!(
        answer(b"C2D", &[b"abc", b"1.0000000000000000000004"]),
        b"99"
    );
    assert_eq!(
        raised(b"C2D", &[Some(b"abc"), Some(b"1E18")]),
        (
            40,
            12,
            vec![b"C2D".to_vec(), b"2".to_vec(), b"1E18".to_vec()]
        )
    );
}

/// A substitution carries the argument's own bytes, and a byte at or
/// above `0x80` survives to the report where a control byte becomes `?`.
#[test]
fn a_substitution_carries_bytes_and_the_report_makes_them_displayable() {
    assert_eq!(
        raised(b"X2C", &[Some(&[b'4', b'1', 0xff])]),
        (93, 933, vec![vec![0xff]])
    );
    assert_eq!(
        raised(b"X2C", &[Some(&[b'4', b'1', 0x01])]),
        (93, 933, vec![vec![0x01]])
    );
    assert_eq!(
        raised(b"X2C", &[Some(&[b'4', b'1', 0x00])]),
        (93, 933, vec![vec![0x00]])
    );
    assert_eq!(
        raised(b"XRANGE", &[Some(&[0xe9, 0xe9])]),
        (
            40,
            28,
            vec![b"XRANGE".to_vec(), b"1".to_vec(), vec![0xe9, 0xe9]]
        )
    );

    let site = crate::error::ClauseSite {
        sites: &[],
        path: "/p.rex",
    };
    let control = Raised::invalid_digit(crate::error::Notation::Hex, 0x01);
    assert!(
        control.report(&site).windows(3).any(|w| w == *b"\"?\""),
        "a control byte must reach the report as a question mark"
    );
    let high = Raised::invalid_digit(crate::error::Notation::Hex, 0xe9);
    assert!(
        high.report(&site)
            .windows(3)
            .any(|w| w == [b'"', 0xe9, b'"']),
        "a byte at or above 0x80 must reach the report unchanged"
    );
}

/// The arity rows, at both ends and at the interior omission each one
/// admits.
#[test]
fn the_arity_rows_are_the_oracles_own() {
    for (name, min, max) in [
        (b"B2X".as_slice(), 1usize, 1usize),
        (b"BITAND", 1, 3),
        (b"BITOR", 1, 3),
        (b"BITXOR", 1, 3),
        (b"C2D", 1, 2),
        (b"C2X", 1, 1),
        (b"D2C", 1, 2),
        (b"D2X", 1, 2),
        (b"X2B", 1, 1),
        (b"X2C", 1, 1),
        (b"X2D", 1, 2),
    ] {
        let short: Vec<Option<&[u8]>> = vec![Some(b"1"); min - 1];
        assert_eq!(
            raised(name, &short),
            (40, 3, vec![name.to_vec(), min.to_string().into_bytes()]),
            "{} did not name its minimum",
            String::from_utf8_lossy(name)
        );
        let long: Vec<Option<&[u8]>> = vec![Some(b"1"); max + 1];
        assert_eq!(
            raised(name, &long),
            (40, 4, vec![name.to_vec(), max.to_string().into_bytes()]),
            "{} did not name its maximum",
            String::from_utf8_lossy(name)
        );
        // An omission in the one required position is 40.5 -- but only
        // where a second argument fits at all. A row with a maximum of 1
        // answers 40.4 for the same call, since the maximum is checked
        // first: measured, `b2x(,'x')` is 40.4 and `c2d(,'1')` is 40.5.
        let expected = if max >= 2 { 5 } else { 4 };
        let substitutions = if max >= 2 {
            b"1".to_vec()
        } else {
            max.to_string().into_bytes()
        };
        assert_eq!(
            raised(name, &[None, Some(b"1")]),
            (40, expected, vec![name.to_vec(), substitutions]),
            "{} answered the wrong sub-code for an omitted first argument",
            String::from_utf8_lossy(name)
        );
    }
    // `XRANGE` has neither end: a minimum of 0 and no maximum at all, so
    // no argument list is ever too long and none is ever too short.
    assert_eq!(
        call(b"XRANGE", &[]).expect("no arguments is legal").len(),
        256
    );
    assert_eq!(
        call(b"XRANGE", &[Some(b"a".as_slice()); 12])
            .expect("twelve arguments are not too many")
            .len(),
        6
    );
    assert_eq!(
        call(b"XRANGE", &[None, Some(b"\x00")]).expect("an omitted first is legal"),
        b"\x00"
    );
}

/// Every result is text, so a later `NUMERIC DIGITS` cannot reshape it.
#[test]
fn a_converted_number_is_text_that_no_later_digits_setting_reshapes() {
    for (name, argument, expected) in [
        (
            b"C2D".as_slice(),
            b"\x0f\x42\x40".as_slice(),
            b"1000000".as_slice(),
        ),
        (b"X2D", b"0f4240", b"1000000"),
        (b"D2X", b"1000000", b"F4240"),
    ] {
        // A fresh interpreter per case, because the drop to `DIGITS 1`
        // below cannot be undone: a candidate setting is judged against
        // the precision in force, so `12` is not a whole number at 1.
        let mut interp = interp_at("12");
        let value = interp.text(argument);
        let result = dispatch(&mut interp, name, &[Some(value)])
            .expect("a builtin name")
            .expect("this call succeeds");
        interp
            .activation_mut()
            .settings
            .set_digits_str("1")
            .expect("a legal DIGITS setting");
        assert_eq!(
            interp.to_text(result).into_owned(),
            expected,
            "{} was reshaped by a later precision",
            String::from_utf8_lossy(name)
        );
    }
}

/// A result sized from an argument is refused rather than aborting the
/// process.
#[test]
fn a_result_too_large_to_allocate_raises_the_oracles_error_5() {
    for name in [b"D2X".as_slice(), b"D2C"] {
        let failure = call(name, &[Some(b"1"), Some(b"123456789012345678")])
            .expect_err("that length cannot be allocated");
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (5, 0));
    }
    // The adjacent success: a length that is merely large is honoured.
    assert_eq!(answer(b"D2X", &[b"1", b"1000"]).len(), 1000);
    // And a length larger than the string is not a size at all for the
    // pair that reads rather than writes.
    assert_eq!(answer(b"C2D", &[b"\xff", b"123456789012345678"]), b"255");
}
