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

//! [`BufferBytes`], a `MutableBuffer`'s bytes as native calls may hold their
//! address (spec 2026-09-29 2.5).

use rexx_core::{BufferBytes, BufferState};

fn lent(bytes: &[u8]) -> BufferBytes {
    let mut lent = BufferBytes::from(bytes.to_vec());
    lent.lend();
    lent
}

/// **Growth of lent bytes moves them and keeps the old storage**, holding
/// what it held, until the last call that held it ends.
#[test]
fn growth_of_lent_bytes_keeps_the_old_storage_until_the_last_release() {
    let mut bytes = lent(b"abc");
    bytes.lend();
    let old = bytes.as_ptr();
    bytes.extend_from_slice(&[b'x'; 4096]);
    assert_ne!(
        bytes.as_ptr(),
        old,
        "the growth fitted; the test proves nothing"
    );
    assert_eq!(bytes.retired(), 1);
    assert_eq!(&bytes[..3], b"abc");
    bytes.release();
    assert_eq!(
        bytes.retired(),
        1,
        "the first release freed a call's storage"
    );
    bytes.release();
    assert_eq!(bytes.retired(), 0);
}

/// **Every growth path of lent bytes moves them**: a reservation, an
/// extension and a resize past the capacity each keep the storage they left.
#[test]
fn each_growth_path_moves_lent_bytes() {
    let grow: [fn(&mut BufferBytes); 3] = [
        |bytes| bytes.try_reserve_exact(4096).expect("room"),
        |bytes| bytes.extend_from_slice(&[0; 4096]),
        |bytes| bytes.resize(4096, 0),
    ];
    for (path, grow) in grow.into_iter().enumerate() {
        let mut bytes = lent(b"abc");
        grow(&mut bytes);
        assert_eq!(bytes.retired(), 1, "growth path {path}");
    }
}

/// **Bytes no call holds grow in place, and a fitting growth of lent ones
/// keeps their storage**, so writes through the lent address stay seen.
#[test]
fn unlent_or_fitting_growth_keeps_the_storage() {
    let mut bytes = BufferBytes::from(b"abc".to_vec());
    bytes.extend_from_slice(&[b'x'; 4096]);
    assert_eq!(bytes.retired(), 0);
    let mut bytes = BufferBytes::from(Vec::with_capacity(64));
    bytes.lend();
    let address = bytes.as_ptr();
    bytes.extend_from_slice(&[b'x'; 64]);
    assert_eq!((bytes.as_ptr(), bytes.retired()), (address, 0));
}

/// **A lent buffer's shrink keeps its storage**: `setBufferSize` takes the
/// capacity down without reallocating under a call.
#[test]
fn a_lent_buffer_set_smaller_keeps_its_storage() {
    let mut state = BufferState {
        bytes: Vec::with_capacity(4096).into(),
        capacity: 4096,
        default_size: 16,
    };
    state.bytes.extend_from_slice(&[b'x'; 4096]);
    state.bytes.lend();
    let address = state.bytes.as_ptr();
    state.set_buffer_size(0).expect("room");
    assert_eq!(state.capacity, 16);
    assert_eq!(state.bytes.as_ptr(), address);
}

/// **A copy of lent bytes is lent to nobody**: it grows in place.
#[test]
fn a_copy_of_lent_bytes_is_not_lent() {
    let mut copy = lent(b"abc").clone();
    copy.extend_from_slice(&[0; 4096]);
    assert_eq!(copy.retired(), 0);
}

/// **An edit that grows lent bytes is refused** rather than freeing the
/// storage under the call.
#[test]
#[should_panic(expected = "a buffer edit reallocated lent storage")]
fn an_edit_growing_lent_bytes_panics() {
    let mut bytes = lent(b"abc");
    bytes.edit(|bytes| bytes.extend_from_slice(&[0; 4096]));
}
