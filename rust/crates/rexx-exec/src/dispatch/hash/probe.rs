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

/// Where `index` belongs, and where it is: the bucket, the slot holding it if
/// any, and the last slot on that bucket's chain.
pub(super) struct Probe {
    pub(super) bucket: usize,
    pub(super) found: Option<usize>,
    pub(super) last: Option<usize>,
}

pub(super) fn probe(interp: &mut Interp, receiver: ObjRef, index: ObjRef) -> Result<(Store, Probe), Failure> {
    let store = store_of(interp, receiver)?;
    probe_in(interp, receiver, CONTENTS, store, index)
}

/// [`probe`] against a store the caller has already read, so that a
/// `Directory`'s method table can be searched with the same chain walk.
pub(super) fn probe_in(
    interp: &mut Interp,
    receiver: ObjRef,
    half: Half,
    mut store: Store,
    index: ObjRef,
) -> Result<(Store, Probe), Failure> {
    let keys = keys_of(interp, receiver);
    let generation = interp.store_generation;
    let hash = hash_of(interp, keys, index)?;
    if generation != interp.store_generation {
        store = store_in(interp, receiver, half)?;
    }
    'search: loop {
        let bucket = (hash % store.buckets as u64) as usize;
        if slot_at(interp, store.indexes, bucket)?.is_none() {
            return Ok((
                store,
                Probe {
                    bucket,
                    found: None,
                    last: None,
                },
            ));
        }
        let mut slot = bucket;
        let mut last = bucket;
        loop {
            let generation = interp.store_generation;
            let matches = same_index(interp, &store, keys, slot, index)?;
            if generation != interp.store_generation {
                let current = store_in(interp, receiver, half)?;
                if current != store {
                    store = current;
                    continue 'search;
                }
            }
            if matches {
                return Ok((
                    store,
                    Probe {
                        bucket,
                        found: Some(slot),
                        last: Some(last),
                    },
                ));
            }
            last = slot;
            slot = link_at(interp, &store, slot)?;
            if slot >= store.total {
                return Ok((
                    store,
                    Probe {
                        bucket,
                        found: None,
                        last: Some(last),
                    },
                ));
            }
        }
    }
}
