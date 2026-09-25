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

use super::find_byte;

/// [`find_byte`]'s word step against the byte-at-a-time answer it stands
/// in for, at every length from empty to past the step's own width.
#[test]
fn find_byte_agrees_with_position() {
    let alphabet = [0x00u8, 0x01, 0x80, b'a'];
    for length in 0..40usize {
        for seed in 0..500u32 {
            let mut hay = Vec::with_capacity(length);
            let mut state = seed.wrapping_mul(2_654_435_761).wrapping_add(length as u32);
            for _ in 0..length {
                state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                hay.push(alphabet[(state >> 16) as usize % alphabet.len()]);
            }
            for &wanted in &alphabet {
                assert_eq!(
                    find_byte(&hay, wanted),
                    hay.iter().position(|&byte| byte == wanted),
                    "hay {hay:02x?} wanted {wanted:#04x}"
                );
            }
        }
    }
}
