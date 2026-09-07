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

//! The mapped collections' store, and the two classes that are the store with
//! nothing on top.
//!
//! A child of [`super`] for [`super::collection`]'s reason: the rows point at
//! `NativeMethod`s, whose parameter list names types only that module can.
//!
//! # The geometry is observable, and that is why it is ported rather than
//! # invented
//!
//! Iteration order is bucket order. Measured on the oracle, a `Table` given
//! the keys `zebra apple Mango q longer-key-name '' '  ' Z` in that order
//! answers `allIndexes` as `'' '  ' longer-key-name Z zebra Mango apple q`;
//! given the same keys in reverse it answers a different order again; and
//! `.Table~new(100)` with the same keys answers a third. Forty integer keys
//! come back `40,20,21,...` So a store that keeps insertion order diverges on
//! `allIndexes`, `allItems`, `makeArray`, `supplier` and `do over` for every
//! class in this phase, on any receiver holding more than a couple of
//! entries.
//!
//! The rules are all in the C++ and all small:
//!
//! * `calculateBucketSize` is `max(capacity, 17)` forced odd, capped at
//!   `1 << 30` (`classes/support/HashCollection.cpp:150`,
//!   `HashCollection.hpp:126`);
//! * the table is `bucketSize` primary slots plus an equal overflow area --
//!   `allocateContents(bucketSize, bucketSize * 2)` (`HashCollection.cpp:84`);
//! * the free chain runs upward from `bucketSize`
//!   (`HashContents.cpp:145`), `put` appends at the END of a bucket's chain
//!   (`:226`, `:275`), and the table is full when the free chain empties and
//!   not when the item count reaches the total (`HashContents.hpp:297`);
//! * growth doubles the TOTAL size and recalculates
//!   (`HashCollection.cpp:96`), then `reMerge` re-adds in old bucket order
//!   (`HashContents.cpp:1238`);
//! * iteration walks buckets `0..bucketSize` and follows each chain
//!   (`HashContents.hpp:104`-`:124`).
//!
//! A Python simulation of exactly that predicted the oracle's answer on four
//! independent cases before any of this was written: the eight keys above,
//! the same eight reversed, the same eight at `.Table~new(100)`, and forty
//! integer keys -- which is the one that exercises growth, and which needed
//! the free-chain fullness test to come out right.

use super::{
    Arity, BehaviourId, Body, Cleared, Decoded, Failure, Interp, Loud, NativeMethod, ObjRef,
    Raised, array_slots, new_instance,
};

/// The five pool entries a hash store is, bound in the receiver's own pool
/// under the `MapCollection` class as scope.
///
/// A pool rather than a new `Body` variant, for [`super::collection`]'s
/// reason: `ScopePools` is already walked by the collector, so a store built
/// out of `Array` objects is GC-visible the moment it exists -- which is spec
/// D101's requirement, met by construction rather than by new collector code.
///
/// `INDEXES`, `ITEMS` and `NEXT` are parallel arrays of `totalSize` slots.
/// An `INDEXES` hole is an available slot. `NEXT` holds a chain link, with
/// `totalSize` itself standing for `NoMore` -- no slot has that number.
const HASH_INDEXES: &[u8] = b"HASHINDEXES";
const HASH_ITEMS: &[u8] = b"HASHITEMS";
const HASH_NEXT: &[u8] = b"HASHNEXT";
const HASH_BUCKETS: &[u8] = b"HASHBUCKETS";
const HASH_FREE: &[u8] = b"HASHFREE";

/// `HashCollection::MinimumBucketSize` (`classes/support/HashCollection.hpp:126`).
const MINIMUM_BUCKET_SIZE: usize = 17;

/// `HashCollection::calculateBucketSize` (`classes/support/HashCollection.cpp:150`).
fn calculate_bucket_size(capacity: usize) -> usize {
    if capacity >= 1 << 30 {
        return 1 << 30;
    }
    if capacity < MINIMUM_BUCKET_SIZE {
        return MINIMUM_BUCKET_SIZE;
    }
    // `(capacity & 1) == 0` upstream: an odd bucket count is preferred.
    if capacity.is_multiple_of(2) {
        capacity + 1
    } else {
        capacity
    }
}

/// The class the store's entries are bound under.
///
/// `Table` rather than `MapCollection`, which is the class every receiver in
/// this phase really derives from: `MapCollection` is donated by
/// `CoreClasses.orx` and the class registry does not answer `lookup` for it,
/// so reaching for it here panicked on the first `t['k'] = 'v'`. The scope is
/// only a namespace key -- `Queue`'s store hangs under `Array` for the same
/// reason -- so a native class the registry always has is the right one.
fn hash_scope(interp: &mut Interp) -> ObjRef {
    interp
        .classes()
        .lookup("Table")
        .expect("Table is a native class")
}

/// The two classes this task owns.
///
/// **Narrower than the set of receivers these bodies are reached for**, and
/// that is the point. `Setup.cpp` donates `IdentityTable`'s `At`, `Put` and
/// `[]` rows to `StringTable` and that whole set on to `Directory`
/// (`memory/Setup.cpp:881`, `:933`), and `Set`, `Bag` and `Relation` take
/// theirs the same way, so one method identity here serves seven classes
/// exactly as one C++ function serves them upstream. Registering a body for
/// `Table` registers it for all of them.
///
/// Each of the other five has its own semantics and its own task -- `Set`'s
/// index-only rule, `Bag` and `Relation`'s multi-value `put`, the string
/// classes' entry map -- so a receiver of one of them gets back exactly the
/// refusal it had before this file existed, by way of [`not_this_task`].
/// Measured as the instrument for that: with the guard missing, the
/// method-body table reported 38 rows regressing from `answers` to
/// `diverge`.
const OWNED: &[&str] = &["Table", "IdentityTable", "Set", "Relation", "Bag"];

/// The classes whose `put` takes its index from its value --
/// `IndexOnlyHashCollection`, whose only two subclasses are `Set` and `Bag`
/// (`classes/support/HashCollection.cpp:1129`).
const INDEX_ONLY: &[&str] = &["Set", "Bag"];

/// The classes whose `put` adds a second entry under an existing index
/// instead of replacing it -- `MultiValueContents`, whose `put` is
/// `addFront` (`classes/support/HashContents.cpp:1650`).
const MULTI_VALUE: &[&str] = &["Relation", "Bag"];

/// Whether `receiver` holds more than one item per index.
fn multi_value(interp: &mut Interp, receiver: ObjRef) -> bool {
    let Some(class) = interp.class_of_value(receiver) else {
        return false;
    };
    for name in MULTI_VALUE {
        let Some(base) = interp.classes().lookup(name) else {
            continue;
        };
        if interp.classes().is_a(class, base) {
            return true;
        }
    }
    false
}

/// Whether `receiver`'s `put` is the index-only one.
fn index_only(interp: &mut Interp, receiver: ObjRef) -> bool {
    let Some(class) = interp.class_of_value(receiver) else {
        return false;
    };
    for name in INDEX_ONLY {
        let Some(base) = interp.classes().lookup(name) else {
            continue;
        };
        if interp.classes().is_a(class, base) {
            return true;
        }
    }
    false
}

/// Whether `receiver`'s class is one of [`OWNED`].
pub(super) fn owns(interp: &mut Interp, receiver: ObjRef) -> bool {
    let Some(class) = interp.class_of_value(receiver) else {
        return false;
    };
    for name in OWNED {
        let Some(base) = interp.classes().lookup(name) else {
            continue;
        };
        if interp.classes().is_a(class, base) {
            return true;
        }
    }
    false
}

/// The refusal a receiver another task owns gets, which is the one it got
/// before this file existed: the class's own name and the method's.
fn not_this_task(interp: &mut Interp, receiver: ObjRef, method: &[u8]) -> Failure {
    match interp.receiver_class_id(receiver) {
        Some(id) => Loud::native_method(method, &id).into(),
        None => Loud::receiver_class("a value that is not a hash collection").into(),
    }
}

/// The classes whose instances get a store, named rather than derived for
/// [`hash_scope`]'s reason.
///
/// Measured, oracle and crate agreeing, that this is the same set
/// `MapCollection` would have given: these nine all answer `1` to
/// `isSubClassOf(.MapCollection)` where `.Array`, `.List` and `.Queue` answer
/// `0`.
const STORE_HOLDERS: &[&str] = &[
    "Table",
    "IdentityTable",
    "Set",
    "Bag",
    "Relation",
    "Directory",
    "StringTable",
    "Properties",
    "Stem",
];

/// Whether `receiver` is an instance of a class this file gives a store to.
fn holds_hash_store(interp: &mut Interp, receiver: ObjRef) -> bool {
    let Some(class) = interp.class_of_value(receiver) else {
        return false;
    };
    for name in STORE_HOLDERS {
        let Some(base) = interp.classes().lookup(name) else {
            continue;
        };
        if interp.classes().is_a(class, base) {
            return true;
        }
    }
    false
}

/// A hash store's three arrays and its two scalars, read out together.
struct Store {
    indexes: ObjRef,
    items: ObjRef,
    next: ObjRef,
    buckets: usize,
    free: usize,
    total: usize,
}

impl Store {
    /// The chain terminator, which is one past the last slot.
    fn no_more(&self) -> usize {
        self.total
    }
}

fn pool_variable(interp: &Interp, owner: ObjRef, scope: ObjRef, name: &[u8]) -> Option<ObjRef> {
    match interp.heap.get(owner).map(|object| &object.body) {
        Some(Body::Instance { pools, .. }) => pools.get(scope, name),
        _ => None,
    }
}

fn counted_pool_variable(
    interp: &Interp,
    owner: ObjRef,
    scope: ObjRef,
    name: &[u8],
) -> Option<usize> {
    match pool_variable(interp, owner, scope, name)?.decode() {
        Decoded::SmallInt(value) if value >= 0 => Some(value as usize),
        _ => None,
    }
}

/// Builds a store of `buckets` primary slots and as many overflow slots, and
/// binds it to `receiver`.
fn install_store(interp: &mut Interp, receiver: ObjRef, buckets: usize) -> Store {
    let total = buckets * 2;
    let scope = hash_scope(interp);
    let indexes = interp.alloc_with(BehaviourId::ARRAY, Body::array(vec![None; total]));
    interp.roots.push_temp(indexes);
    let items = interp.alloc_with(BehaviourId::ARRAY, Body::array(vec![None; total]));
    interp.roots.push_temp(items);
    // `initializeFreeChain` (`classes/support/HashContents.cpp:145`): every
    // bucket slot ends its own chain, and the overflow slots are chained
    // upward from `bucketSize` with the last one ending the chain.
    let mut links: Vec<Option<ObjRef>> = Vec::with_capacity(total);
    for slot in 0..total {
        let link = if slot < buckets || slot + 1 == total {
            total
        } else {
            slot + 1
        };
        links.push(Some(interp.counted(link)));
    }
    let next = interp.alloc_with(BehaviourId::ARRAY, Body::array(links));
    interp.roots.push_temp(next);
    let buckets_value = interp.counted(buckets);
    let free_value = interp.counted(buckets);
    interp.set_pool_variable(receiver, scope, HASH_INDEXES, indexes);
    interp.set_pool_variable(receiver, scope, HASH_ITEMS, items);
    interp.set_pool_variable(receiver, scope, HASH_NEXT, next);
    interp.set_pool_variable(receiver, scope, HASH_BUCKETS, buckets_value);
    interp.set_pool_variable(receiver, scope, HASH_FREE, free_value);
    Store {
        indexes,
        items,
        next,
        buckets,
        free: buckets,
        total,
    }
}

/// The receiver's store, built at the minimum size on demand.
///
/// On demand for [`super::collection::store_of`]'s reason: upstream the
/// contents belong to the object the allocator returns, so a subclass whose
/// `INIT` does not forward still has them.
fn store_of(interp: &mut Interp, receiver: ObjRef) -> Result<Store, Failure> {
    let scope = hash_scope(interp);
    if let (Some(indexes), Some(items), Some(next), Some(buckets), Some(free)) = (
        pool_variable(interp, receiver, scope, HASH_INDEXES),
        pool_variable(interp, receiver, scope, HASH_ITEMS),
        pool_variable(interp, receiver, scope, HASH_NEXT),
        counted_pool_variable(interp, receiver, scope, HASH_BUCKETS),
        counted_pool_variable(interp, receiver, scope, HASH_FREE),
    ) {
        let total = array_slots(interp, indexes)?.len();
        return Ok(Store {
            indexes,
            items,
            next,
            buckets,
            free,
            total,
        });
    }
    // **A `Body::Native` receiver is not this store's**, and the check has to
    // be here because the two are one method. `Setup.cpp` donates
    // `IdentityTable`'s `At`, `Put` and `[]` rows to `StringTable` and then
    // `StringTable`'s whole set to `Directory` (`memory/Setup.cpp:881`,
    // `:933`), so all four classes share one method identity and one body --
    // which is why registering these names for `Table` alone reached a
    // `StringTable` during the bootstrap. `Directory`, `StringTable` and
    // `Properties` keep the entry map they already have until the task that
    // owns them.
    if matches!(
        interp.heap.get(receiver).map(|object| &object.body),
        Some(Body::Native(_))
    ) || !holds_hash_store(interp, receiver)
    {
        return Err(Loud::receiver_class("a value that is not a hash collection").into());
    }
    Ok(install_store(interp, receiver, MINIMUM_BUCKET_SIZE))
}

fn set_free(interp: &mut Interp, receiver: ObjRef, free: usize) {
    let scope = hash_scope(interp);
    let value = interp.counted(free);
    interp.set_pool_variable(receiver, scope, HASH_FREE, value);
}

fn slot_at(interp: &Interp, array: ObjRef, slot: usize) -> Result<Option<ObjRef>, Failure> {
    Ok(array_slots(interp, array)?.get(slot).copied().flatten())
}

fn link_at(interp: &Interp, store: &Store, slot: usize) -> Result<usize, Failure> {
    match slot_at(interp, store.next, slot)?.map(|value| value.decode()) {
        Some(Decoded::SmallInt(link)) if link >= 0 => Ok(link as usize),
        _ => Ok(store.no_more()),
    }
}

fn write_slot(interp: &mut Interp, array: ObjRef, slot: usize, value: Option<ObjRef>) {
    if let Some(Body::Array { slots, .. }) =
        interp.heap.get_mut(array).map(|object| &mut object.body)
        && let Some(cell) = slots.get_mut(slot)
    {
        *cell = value;
    }
}

fn write_link(interp: &mut Interp, store: &Store, slot: usize, link: usize) {
    let value = interp.counted(link);
    write_slot(interp, store.next, slot, Some(value));
}

// ---- the key protocols ----

/// Which key protocol a receiver's class uses.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Keys {
    /// `IdentityHashContents`: reference identity, hashed by `getHashValue()`
    /// rather than by the identity hash -- `HashContents.hpp:197`'s own
    /// comment gives the reason, which is that a saved image would otherwise
    /// lose its lookups.
    Identity,
    /// `EqualityHashContents`: `equalValue`, hashed by `hash()`
    /// (`HashContents.hpp:441`).
    Equality,
}

fn keys_of(interp: &mut Interp, receiver: ObjRef) -> Keys {
    let Some(class) = interp.class_of_value(receiver) else {
        return Keys::Equality;
    };
    let Some(identity) = interp.classes().lookup("IdentityTable") else {
        return Keys::Equality;
    };
    if interp.classes().is_a(class, identity) {
        Keys::Identity
    } else {
        Keys::Equality
    }
}

/// Whether `value`'s behaviour is a primitive one, which is what
/// `RexxObject::hash` branches on (`classes/ObjectClass.cpp:416`).
///
/// **"Has a string value" and "is a base-class object" are different
/// questions**, which is spec D96's trap: a `String` subclass has a string
/// value and is not a base class, so its `HASHCODE` override is reached where
/// a plain string's -- which cannot exist, since `.String~define` is 98.985 --
/// would not be.
fn is_base_class(interp: &mut Interp, value: ObjRef) -> bool {
    let Some(class) = interp.class_of_value(value) else {
        return true;
    };
    let id = interp.classes().id_string(class).to_string();
    interp.classes().system_lookup(&id) == Some(class)
}

/// `RexxInternalObject::getHashValue()`, which is what the identity contents
/// hashes with and what the equality contents falls back on for a base class.
fn get_hash_value(interp: &mut Interp, value: ObjRef) -> u64 {
    super::hash_value(interp, value)
}

/// `RexxString::getObjectHashCode` (`classes/StringClass.cpp:220`): a
/// `HASHCODE` answer turned back into a number.
///
/// Empty is 1; eight bytes or more are read as a little-endian `size_t`;
/// anything shorter is read as a **signed** two-byte short, which for a
/// one-character answer picks up the terminating null -- so a single byte
/// `'80'x` is negative there and sign-extends.
fn object_hash_code(text: &[u8]) -> u64 {
    if text.is_empty() {
        return 1;
    }
    if text.len() >= 8 {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&text[..8]);
        return u64::from_le_bytes(bytes);
    }
    let low = text[0] as u16;
    let high = if text.len() > 1 { text[1] as u16 } else { 0 };
    ((low | (high << 8)) as i16) as i64 as u64
}

/// The hash a store looks an index up by.
fn hash_of(interp: &mut Interp, keys: Keys, index: ObjRef) -> Result<u64, Failure> {
    if keys == Keys::Identity || is_base_class(interp, index) {
        return Ok(get_hash_value(interp, index));
    }
    let caller = interp.caller();
    let answer = interp.send_message(index, b"HASHCODE", None, &[], caller)?;
    let Some(answer) = answer else {
        return Err(Raised::no_result(b"HASHCODE").into());
    };
    let text = interp.to_text(answer).into_owned();
    Ok(object_hash_code(&text))
}

/// Whether the entry at `slot` is the one `index` names.
fn same_index(
    interp: &mut Interp,
    store: &Store,
    keys: Keys,
    slot: usize,
    index: ObjRef,
) -> Result<bool, Failure> {
    let Some(held) = slot_at(interp, store.indexes, slot)? else {
        return Ok(false);
    };
    match keys {
        Keys::Identity => Ok(held == index),
        Keys::Equality => super::collection::same_item(interp, index, held),
    }
}

// ---- the store's own operations ----

/// Where `index` belongs, and where it is: the bucket, the slot holding it if
/// any, and the last slot on that bucket's chain.
struct Probe {
    bucket: usize,
    found: Option<usize>,
    last: Option<usize>,
}

fn probe(interp: &mut Interp, receiver: ObjRef, index: ObjRef) -> Result<(Store, Probe), Failure> {
    let store = store_of(interp, receiver)?;
    let keys = keys_of(interp, receiver);
    let hash = hash_of(interp, keys, index)?;
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
        if same_index(interp, &store, keys, slot, index)? {
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

/// Every occupied slot, in bucket order following each chain --
/// `HashContents::TableIterator`.
fn walk(interp: &mut Interp, receiver: ObjRef) -> Result<Vec<usize>, Failure> {
    let store = store_of(interp, receiver)?;
    let mut order = Vec::new();
    for bucket in 0..store.buckets {
        let mut slot = bucket;
        while slot < store.total && slot_at(interp, store.indexes, slot)?.is_some() {
            order.push(slot);
            slot = link_at(interp, &store, slot)?;
        }
    }
    Ok(order)
}

/// Grows the table and re-adds every entry in old bucket order --
/// `HashCollection::expandContents` and `HashContents::reMerge`.
fn expand(interp: &mut Interp, receiver: ObjRef) -> Result<(), Failure> {
    let store = store_of(interp, receiver)?;
    let order = walk(interp, receiver)?;
    let mut carried = Vec::with_capacity(order.len());
    for slot in order {
        let index = slot_at(interp, store.indexes, slot)?;
        let item = slot_at(interp, store.items, slot)?;
        if let Some(index) = index {
            interp.roots.push_temp(index);
            if let Some(item) = item {
                interp.roots.push_temp(item);
            }
            carried.push((index, item));
        }
    }
    let buckets = calculate_bucket_size(store.total * 2);
    install_store(interp, receiver, buckets);
    for (index, item) in carried {
        // **Never `insert`**, which replaces an index the table already
        // holds: a `Relation` carries duplicate indexes and would lose one
        // per pair on every growth. `reMerge` adds rather than puts
        // (`classes/support/HashContents.cpp:1238`), and adding in the old
        // walk order is what keeps each index's chain in its order.
        append_entry(interp, receiver, index, item)?;
    }
    Ok(())
}

/// Appends an entry at the end of its bucket's chain without looking for an
/// index the table already holds -- `HashContents::append`'s half of `put`.
///
/// The caller has already made room.
fn append_entry(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
    item: Option<ObjRef>,
) -> Result<(), Failure> {
    let store = store_of(interp, receiver)?;
    if store.free >= store.total {
        expand(interp, receiver)?;
        return append_entry(interp, receiver, index, item);
    }
    let store = store_of(interp, receiver)?;
    let keys = keys_of(interp, receiver);
    let hash = hash_of(interp, keys, index)?;
    let bucket = (hash % store.buckets as u64) as usize;
    if slot_at(interp, store.indexes, bucket)?.is_none() {
        write_slot(interp, store.indexes, bucket, Some(index));
        write_slot(interp, store.items, bucket, item);
        write_link(interp, &store, bucket, store.no_more());
        return Ok(());
    }
    let mut last = bucket;
    loop {
        let next = link_at(interp, &store, last)?;
        if next >= store.total {
            break;
        }
        last = next;
    }
    let slot = store.free;
    let next_free = link_at(interp, &store, slot)?;
    set_free(interp, receiver, next_free);
    write_slot(interp, store.indexes, slot, Some(index));
    write_slot(interp, store.items, slot, item);
    write_link(interp, &store, last, slot);
    write_link(interp, &store, slot, store.no_more());
    Ok(())
}

/// `HashContents::addFront` (`classes/support/HashContents.cpp:1650`): a new
/// entry for an index the table already holds goes to the FRONT of that
/// index's chain.
///
/// The bucket slot cannot move, so the entry that was there is copied into a
/// free slot and chained behind the new one -- `HashContents::insert`
/// (`:313`). Measured: a `Relation` given `k -> v1` then `k -> v2` answers
/// `allAt('k')` as `v2,v1` and `at('k')` as `v2`.
fn insert_front(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
    item: Option<ObjRef>,
) -> Result<(), Failure> {
    let store = store_of(interp, receiver)?;
    if store.free >= store.total {
        expand(interp, receiver)?;
        return insert_front(interp, receiver, index, item);
    }
    let store = store_of(interp, receiver)?;
    let keys = keys_of(interp, receiver);
    let hash = hash_of(interp, keys, index)?;
    let bucket = (hash % store.buckets as u64) as usize;
    if slot_at(interp, store.indexes, bucket)?.is_none() {
        write_slot(interp, store.indexes, bucket, Some(index));
        write_slot(interp, store.items, bucket, item);
        write_link(interp, &store, bucket, store.no_more());
        return Ok(());
    }
    let moved = store.free;
    let next_free = link_at(interp, &store, moved)?;
    set_free(interp, receiver, next_free);
    let head_index = slot_at(interp, store.indexes, bucket)?;
    let head_item = slot_at(interp, store.items, bucket)?;
    let head_next = link_at(interp, &store, bucket)?;
    write_slot(interp, store.indexes, moved, head_index);
    write_slot(interp, store.items, moved, head_item);
    write_link(interp, &store, moved, head_next);
    write_slot(interp, store.indexes, bucket, Some(index));
    write_slot(interp, store.items, bucket, item);
    write_link(interp, &store, bucket, moved);
    Ok(())
}

/// `HashContents::put`: replaces the value under an index the table already
/// holds, and otherwise appends to the end of the bucket's chain, expanding
/// first if the free chain has run out.
fn insert(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
    item: Option<ObjRef>,
) -> Result<(), Failure> {
    // **The fullness test comes first, and it is the free chain rather than
    // the item count** (`classes/support/HashContents.hpp:297`): a table can
    // run its overflow dry while primary buckets stand empty, and upstream
    // expands before every `put` rather than only when one needs a slot.
    // Measured through the simulation this was written from: testing the item
    // count instead runs the free chain out on the forty-integer case.
    let store = store_of(interp, receiver)?;
    if store.free >= store.total {
        expand(interp, receiver)?;
    }
    let (store, found) = probe(interp, receiver, index)?;
    if let Some(slot) = found.found {
        write_slot(interp, store.items, slot, item);
        return Ok(());
    }
    let Some(last) = found.last else {
        write_slot(interp, store.indexes, found.bucket, Some(index));
        write_slot(interp, store.items, found.bucket, item);
        write_link(interp, &store, found.bucket, store.no_more());
        return Ok(());
    };
    let slot = store.free;
    let next_free = link_at(interp, &store, slot)?;
    set_free(interp, receiver, next_free);
    write_slot(interp, store.indexes, slot, Some(index));
    write_slot(interp, store.items, slot, item);
    write_link(interp, &store, last, slot);
    write_link(interp, &store, slot, store.no_more());
    Ok(())
}

/// `HashContents::remove`: unlinks the entry and returns its slot to the free
/// chain, answering the item it held.
fn take(interp: &mut Interp, receiver: ObjRef, index: ObjRef) -> Result<Option<ObjRef>, Failure> {
    let (store, found) = probe(interp, receiver, index)?;
    let Some(slot) = found.found else {
        return Ok(None);
    };
    let previous = found.last.filter(|last| *last != slot);
    remove_at(interp, receiver, &store, slot, previous)
}

/// [`take`] for an entry the caller has already found, named by its slot
/// rather than by its index -- what `removeItem` needs, since two entries
/// under one index differ only by their item.
fn take_at(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
    slot: usize,
) -> Result<Option<ObjRef>, Failure> {
    let store = store_of(interp, receiver)?;
    let keys = keys_of(interp, receiver);
    let hash = hash_of(interp, keys, index)?;
    let bucket = (hash % store.buckets as u64) as usize;
    let mut previous = None;
    let mut cursor = bucket;
    while cursor < store.total && cursor != slot {
        previous = Some(cursor);
        cursor = link_at(interp, &store, cursor)?;
    }
    if cursor != slot {
        return Ok(None);
    }
    remove_at(interp, receiver, &store, slot, previous)
}

/// Unlinks the entry at `slot`, whose chain predecessor is `previous`, and
/// answers the item it held.
fn remove_at(
    interp: &mut Interp,
    receiver: ObjRef,
    store: &Store,
    slot: usize,
    previous: Option<usize>,
) -> Result<Option<ObjRef>, Failure> {
    let item = slot_at(interp, store.items, slot)?;
    let next = link_at(interp, store, slot)?;
    if slot < store.buckets {
        // A bucket slot cannot be freed, so the chain is closed by copying
        // the next entry into it -- `HashContents::closeChain`.
        if next < store.total {
            let moved_index = slot_at(interp, store.indexes, next)?;
            let moved_item = slot_at(interp, store.items, next)?;
            let moved_next = link_at(interp, store, next)?;
            write_slot(interp, store.indexes, slot, moved_index);
            write_slot(interp, store.items, slot, moved_item);
            write_link(interp, store, slot, moved_next);
            free_slot(interp, receiver, store, next);
        } else {
            write_slot(interp, store.indexes, slot, None);
            write_slot(interp, store.items, slot, None);
            write_link(interp, store, slot, store.no_more());
        }
        return Ok(item);
    }
    if let Some(previous) = previous {
        write_link(interp, store, previous, next);
    }
    free_slot(interp, receiver, store, slot);
    Ok(item)
}

fn free_slot(interp: &mut Interp, receiver: ObjRef, store: &Store, slot: usize) {
    write_slot(interp, store.indexes, slot, None);
    write_slot(interp, store.items, slot, None);
    let free = store.free;
    write_link(interp, store, slot, free);
    set_free(interp, receiver, slot);
}

// ---- the shared surface ----

/// The index argument the store's index-taking methods require.
///
/// **88.901 and not 93.903**, which is the split measured on a `.Table~new`:
/// `at()`, `put()`, `put('v')`, `hasIndex()` and `remove()` all report
/// `Missing argument; argument index is required.` -- the named form
/// `HashCollection` raises -- while `hasItem()`, `index()` and
/// `removeItem()`, which take an ITEM, report the positional 93.903.
fn index_argument(args: &[Option<ObjRef>], position: usize) -> Result<ObjRef, Failure> {
    args.get(position - 1)
        .copied()
        .flatten()
        .ok_or_else(|| Raised::missing_named_argument("index").into())
}

/// `HashCollection::getRexx` and `[]`: the item under that index, or `.nil`.
pub(super) fn store_at(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let index = index_argument(args, 1)?;
    let (store, found) = probe(interp, receiver, index)?;
    let Some(slot) = found.found else {
        return Ok(Some(ObjRef::NIL));
    };
    Ok(Some(
        slot_at(interp, store.items, slot)?.unwrap_or(ObjRef::NIL),
    ))
}

/// `HashCollection::putRexx` and `[]=`: **the item comes first and the index
/// second**, the opposite of the reading order of `t['k'] = 'v'`.
pub(super) fn store_put(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(item) = args.first().copied().flatten() else {
        return Err(Raised::missing_named_argument("item").into());
    };
    // **A `Set`'s index is its value.**
    // `IndexOnlyHashCollection::validateValueIndex`
    // (`classes/support/HashCollection.cpp:1129`): the value is required, the
    // index is optional, and an index that is given must equal the value or
    // the send is 93.949. Measured: `s~put('a')` twice leaves `items` at 1,
    // `s~put('b','b')` is accepted, and `s~put('c','d')` raises.
    let index = if index_only(interp, receiver) {
        match args.get(1).copied().flatten() {
            // `isIndexEqual(value, index)` -- the VALUE receives the `==`.
            Some(given) if !super::collection::same_item(interp, item, given)? => {
                return Err(Raised::index_does_not_match().into());
            }
            _ => item,
        }
    } else {
        index_argument(args, 2)?
    };
    if multi_value(interp, receiver) {
        insert_front(interp, receiver, index, Some(item))?;
    } else {
        insert(interp, receiver, index, Some(item))?;
    }
    Ok(None)
}

/// Every (index, item) pair the receiver holds, in bucket order.
fn pairs(interp: &mut Interp, receiver: ObjRef) -> Result<Vec<(ObjRef, ObjRef)>, Failure> {
    let store = store_of(interp, receiver)?;
    let mut pairs = Vec::new();
    for slot in walk(interp, receiver)? {
        let Some(index) = slot_at(interp, store.indexes, slot)? else {
            continue;
        };
        let item = slot_at(interp, store.items, slot)?.unwrap_or(ObjRef::NIL);
        // Rooted for `ordered_pairs`'s reason: a caller sends `==` per pair
        // and the callback may empty the collection.
        interp.roots.push_temp(index);
        interp.roots.push_temp(item);
        pairs.push((index, item));
    }
    Ok(pairs)
}

fn native_hash_all_indexes(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"ALLINDEXES"));
    }
    let indexes = pairs(interp, receiver)?
        .into_iter()
        .map(|(index, _)| index)
        .collect();
    Ok(Some(super::collection::array_of(interp, indexes)))
}

fn native_hash_all_items(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"ALLITEMS"));
    }
    let items = pairs(interp, receiver)?
        .into_iter()
        .map(|(_, item)| item)
        .collect();
    Ok(Some(super::collection::array_of(interp, items)))
}

/// `makeArray` answers the INDEXES, where `Array`'s answers the items -- spec
/// D98. Measured, a `.Directory` holding `k1` and `k2` answers `makeArray` as
/// `k1 k2` against `allItems` as `v1 v2`.
fn native_hash_make_array(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"MAKEARRAY"));
    }
    native_hash_all_indexes(interp, cleared, receiver, args)
}

fn native_hash_items(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"ITEMS"));
    }
    let count = walk(interp, receiver)?.len();
    Ok(Some(interp.counted(count)))
}

fn native_hash_is_empty(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"ISEMPTY"));
    }
    let empty = walk(interp, receiver)?.is_empty();
    Ok(Some(crate::eval::logical(empty)))
}

/// `HashCollection::emptyRexx`: drops every entry and answers the receiver.
fn native_hash_empty(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"EMPTY"));
    }
    let store = store_of(interp, receiver)?;
    install_store(interp, receiver, store.buckets);
    Ok(Some(receiver))
}

fn native_hash_has_index(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"HASINDEX"));
    }
    let index = index_argument(args, 1)?;
    let (_, found) = probe(interp, receiver, index)?;
    Ok(Some(crate::eval::logical(found.found.is_some())))
}

fn native_hash_has_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"HASITEM"));
    }
    let wanted = super::collection::item_argument(args)?;
    for (_, item) in pairs(interp, receiver)? {
        if super::collection::same_item(interp, wanted, item)? {
            return Ok(Some(crate::eval::logical(true)));
        }
    }
    Ok(Some(crate::eval::logical(false)))
}

fn native_hash_index(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"INDEX"));
    }
    let wanted = super::collection::item_argument(args)?;
    for (index, item) in pairs(interp, receiver)? {
        if super::collection::same_item(interp, wanted, item)? {
            return Ok(Some(index));
        }
    }
    Ok(Some(ObjRef::NIL))
}

fn native_hash_remove(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"REMOVE"));
    }
    let index = index_argument(args, 1)?;
    Ok(Some(take(interp, receiver, index)?.unwrap_or(ObjRef::NIL)))
}

fn native_hash_remove_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"REMOVEITEM"));
    }
    let wanted = super::collection::item_argument(args)?;
    for (index, item) in pairs(interp, receiver)? {
        if super::collection::same_item(interp, wanted, item)? {
            take(interp, receiver, index)?;
            return Ok(Some(item));
        }
    }
    Ok(Some(ObjRef::NIL))
}

fn native_hash_supplier(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"SUPPLIER"));
    }
    let held = pairs(interp, receiver)?;
    let items: Vec<ObjRef> = held.iter().map(|(_, item)| *item).collect();
    let indexes: Vec<ObjRef> = held.into_iter().map(|(index, _)| index).collect();
    let items = super::collection::array_of(interp, items);
    let indexes = super::collection::array_of(interp, indexes);
    super::collection::new_supplier(interp, items, indexes).map(Some)
}

/// `TableClass::newRexx` and its siblings: the optional first argument is the
/// table's initial capacity, and it is **observable** -- it decides the
/// bucket count, which decides the iteration order. Measured, the same eight
/// keys in the same order come back differently from `.Table~new` and
/// `.Table~new(100)`, and both orders are what this store's geometry
/// predicts.
pub(super) fn native_hash_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = super::class_receiver(interp, receiver)?;
    let capacity = super::optional_length_argument(interp, args, 0)?.unwrap_or(0);
    let object = new_instance(interp, class)?;
    interp.roots.push_temp(object);
    install_store(interp, object, calculate_bucket_size(capacity));
    let caller = interp.caller();
    interp.send_message(object, super::INIT, None, args, caller)?;
    Ok(Some(object))
}

// ---- `Relation` and `Bag`'s own surface ----

/// Every slot whose index matches, in chain order.
fn slots_for(interp: &mut Interp, receiver: ObjRef, index: ObjRef) -> Result<Vec<usize>, Failure> {
    let store = store_of(interp, receiver)?;
    let keys = keys_of(interp, receiver);
    let mut found = Vec::new();
    for slot in walk(interp, receiver)? {
        if same_index(interp, &store, keys, slot, index)? {
            found.push(slot);
        }
    }
    Ok(found)
}

/// `RelationClass::allAt(index)`: every item under that index, newest first.
fn native_relation_all_at(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"ALLAT"));
    }
    // **93.903 and not 88.901.** `RelationClass`'s own methods take their
    // index positionally where `HashCollection`'s take it as a named
    // argument: measured on a `.Relation~new`, `allAt()` and `removeAll()`
    // report `Missing argument in method; argument 1 is required.` where
    // `at()` and `remove()` report `Missing argument; argument index is
    // required.`
    let index = super::collection::item_argument(args)?;
    let store = store_of(interp, receiver)?;
    let mut items = Vec::new();
    for slot in slots_for(interp, receiver, index)? {
        items.push(slot_at(interp, store.items, slot)?.unwrap_or(ObjRef::NIL));
    }
    Ok(Some(super::collection::array_of(interp, items)))
}

/// `RelationClass::allIndexRexx(item)`: every index whose item matches.
fn native_relation_all_index(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"ALLINDEX"));
    }
    let wanted = super::collection::item_argument(args)?;
    let mut indexes = Vec::new();
    for (index, item) in pairs(interp, receiver)? {
        if super::collection::same_item(interp, wanted, item)? {
            indexes.push(index);
        }
    }
    Ok(Some(super::collection::array_of(interp, indexes)))
}

/// `RelationClass::uniqueIndexes`: each index once, in first-seen order.
fn native_relation_unique_indexes(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"UNIQUEINDEXES"));
    }
    let keys = keys_of(interp, receiver);
    let mut unique: Vec<ObjRef> = Vec::new();
    for (index, _) in pairs(interp, receiver)? {
        let mut seen = false;
        for held in &unique {
            if match keys {
                Keys::Identity => *held == index,
                Keys::Equality => super::collection::same_item(interp, index, *held)?,
            } {
                seen = true;
                break;
            }
        }
        if !seen {
            unique.push(index);
        }
    }
    Ok(Some(super::collection::array_of(interp, unique)))
}

/// `RelationClass::itemsRexx([index])`: the whole count, or the count under
/// one index. Measured, a relation holding `k -> v1`, `k -> v2` and `j -> w`
/// answers `items` 3, `items('k')` 2 and `items('z')` 0.
fn native_relation_items(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"ITEMS"));
    }
    let count = match args.first().copied().flatten() {
        Some(index) => slots_for(interp, receiver, index)?.len(),
        None => walk(interp, receiver)?.len(),
    };
    Ok(Some(interp.counted(count)))
}

/// `RelationClass::removeAll(index)`: takes every entry under that index out
/// and answers them as an array, newest first -- an empty `Array` for an
/// index the relation does not hold.
fn native_relation_remove_all(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"REMOVEALL"));
    }
    // **93.903 and not 88.901.** `RelationClass`'s own methods take their
    // index positionally where `HashCollection`'s take it as a named
    // argument: measured on a `.Relation~new`, `allAt()` and `removeAll()`
    // report `Missing argument in method; argument 1 is required.` where
    // `at()` and `remove()` report `Missing argument; argument index is
    // required.`
    let index = super::collection::item_argument(args)?;
    let mut removed = Vec::new();
    while let Some(item) = take(interp, receiver, index)? {
        interp.roots.push_temp(item);
        removed.push(item);
    }
    Ok(Some(super::collection::array_of(interp, removed)))
}

/// `RelationClass::supplierRexx([index])`: a supplier over the whole
/// relation, or over one index's items.
fn native_relation_supplier(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"SUPPLIER"));
    }
    let Some(index) = args.first().copied().flatten() else {
        return native_hash_supplier(interp, cleared, receiver, &[]);
    };
    let store = store_of(interp, receiver)?;
    let mut items = Vec::new();
    let mut indexes = Vec::new();
    for slot in slots_for(interp, receiver, index)? {
        items.push(slot_at(interp, store.items, slot)?.unwrap_or(ObjRef::NIL));
        indexes.push(slot_at(interp, store.indexes, slot)?.unwrap_or(ObjRef::NIL));
    }
    let items = super::collection::array_of(interp, items);
    let indexes = super::collection::array_of(interp, indexes);
    super::collection::new_supplier(interp, items, indexes).map(Some)
}

/// `RelationClass::hasItemRexx(item [, index])`: whether the relation holds
/// that item, optionally under that index. Measured, `hasItem('v1','k')` is
/// 1 and `hasItem('v1','j')` is 0 for a relation holding `k -> v1`.
fn native_relation_has_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"HASITEM"));
    }
    let wanted = super::collection::item_argument(args)?;
    let under = args.get(1).copied().flatten();
    let store = store_of(interp, receiver)?;
    let keys = keys_of(interp, receiver);
    for slot in walk(interp, receiver)? {
        let Some(item) = slot_at(interp, store.items, slot)? else {
            continue;
        };
        if !super::collection::same_item(interp, wanted, item)? {
            continue;
        }
        match under {
            None => return Ok(Some(crate::eval::logical(true))),
            Some(index) if same_index(interp, &store, keys, slot, index)? => {
                return Ok(Some(crate::eval::logical(true)));
            }
            Some(_) => {}
        }
    }
    Ok(Some(crate::eval::logical(false)))
}

/// `RelationClass::removeItemRexx(item [, index])`: takes out the first entry
/// holding that item, optionally under that index, and answers it.
fn native_relation_remove_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"REMOVEITEM"));
    }
    let wanted = super::collection::item_argument(args)?;
    let under = args.get(1).copied().flatten();
    let store = store_of(interp, receiver)?;
    let keys = keys_of(interp, receiver);
    for slot in walk(interp, receiver)? {
        let Some(item) = slot_at(interp, store.items, slot)? else {
            continue;
        };
        if !super::collection::same_item(interp, wanted, item)? {
            continue;
        }
        let held = slot_at(interp, store.indexes, slot)?;
        let matches = match under {
            None => true,
            Some(index) => same_index(interp, &store, keys, slot, index)?,
        };
        if matches && let Some(held) = held {
            take_at(interp, receiver, held, slot)?;
            return Ok(Some(item));
        }
    }
    Ok(Some(ObjRef::NIL))
}

/// `Bag~of(item, ...)`: `BagClass::ofRexx`, which unlike `Set~of` keeps the
/// duplicates -- measured, `.Bag~of('p','p','q')~items` is 3.
pub(super) fn native_bag_of(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = super::class_receiver(interp, receiver)?;
    for (at, argument) in args.iter().enumerate() {
        if argument.is_none() {
            return Err(Raised::missing_method_argument(at + 1).into());
        }
    }
    let object = new_instance(interp, class)?;
    interp.roots.push_temp(object);
    install_store(interp, object, MINIMUM_BUCKET_SIZE);
    let caller = interp.caller();
    interp.send_message(object, super::INIT, None, &[], caller)?;
    for argument in args.iter().flatten() {
        insert_front(interp, object, *argument, Some(*argument))?;
    }
    Ok(Some(object))
}

/// `SetClass::ofRexx`: a new `Set` of the receiver's own class holding the
/// arguments, each of which is its own index.
///
/// Measured: `.Set~of('x','y')~items` is 2 and `.Set~of('x','x')~items` is 1,
/// so the arguments go through the same index-only rule `put` applies rather
/// than being appended.
pub(super) fn native_set_of(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = super::class_receiver(interp, receiver)?;
    for (at, argument) in args.iter().enumerate() {
        if argument.is_none() {
            return Err(Raised::missing_method_argument(at + 1).into());
        }
    }
    let object = new_instance(interp, class)?;
    interp.roots.push_temp(object);
    install_store(interp, object, MINIMUM_BUCKET_SIZE);
    let caller = interp.caller();
    interp.send_message(object, super::INIT, None, &[], caller)?;
    for argument in args.iter().flatten() {
        insert(interp, object, *argument, Some(*argument))?;
    }
    Ok(Some(object))
}

/// The mapped collections' primitive methods, chained into
/// `ObjectModel::build` beside [`super::collection::NATIVE_METHODS`].
pub(super) const NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    (
        "Table",
        "ALLINDEXES",
        Arity::Fixed(0),
        native_hash_all_indexes,
    ),
    ("Table", "ALLITEMS", Arity::Fixed(0), native_hash_all_items),
    (
        "Table",
        "MAKEARRAY",
        Arity::Fixed(0),
        native_hash_make_array,
    ),
    ("Table", "ITEMS", Arity::Fixed(0), native_hash_items),
    ("Table", "ISEMPTY", Arity::Fixed(0), native_hash_is_empty),
    ("Table", "EMPTY", Arity::Fixed(0), native_hash_empty),
    ("Table", "HASINDEX", Arity::Fixed(1), native_hash_has_index),
    ("Table", "HASITEM", Arity::Fixed(1), native_hash_has_item),
    ("Table", "INDEX", Arity::Fixed(1), native_hash_index),
    ("Table", "REMOVE", Arity::Fixed(1), native_hash_remove),
    (
        "Table",
        "REMOVEITEM",
        Arity::Fixed(1),
        native_hash_remove_item,
    ),
    ("Table", "SUPPLIER", Arity::Fixed(0), native_hash_supplier),
    (
        "IdentityTable",
        "ALLINDEXES",
        Arity::Fixed(0),
        native_hash_all_indexes,
    ),
    (
        "IdentityTable",
        "ALLITEMS",
        Arity::Fixed(0),
        native_hash_all_items,
    ),
    (
        "IdentityTable",
        "MAKEARRAY",
        Arity::Fixed(0),
        native_hash_make_array,
    ),
    ("IdentityTable", "ITEMS", Arity::Fixed(0), native_hash_items),
    (
        "IdentityTable",
        "ISEMPTY",
        Arity::Fixed(0),
        native_hash_is_empty,
    ),
    ("IdentityTable", "EMPTY", Arity::Fixed(0), native_hash_empty),
    (
        "IdentityTable",
        "HASINDEX",
        Arity::Fixed(1),
        native_hash_has_index,
    ),
    (
        "IdentityTable",
        "HASITEM",
        Arity::Fixed(1),
        native_hash_has_item,
    ),
    ("IdentityTable", "INDEX", Arity::Fixed(1), native_hash_index),
    (
        "IdentityTable",
        "REMOVE",
        Arity::Fixed(1),
        native_hash_remove,
    ),
    (
        "IdentityTable",
        "REMOVEITEM",
        Arity::Fixed(1),
        native_hash_remove_item,
    ),
    (
        "IdentityTable",
        "SUPPLIER",
        Arity::Fixed(0),
        native_hash_supplier,
    ),
    // **`Set`'s two overrides route to the INDEX bodies.** `Setup.cpp` writes
    // `Set`'s `HasItem` as `IdentityTable::hasIndexRexx` and its `RemoveItem`
    // as `IdentityTable::removeRexx`, and `IndexOnlyHashCollection`'s own
    // comment gives the reason: for a collection whose index is its value,
    // searching by index is the faster operation and answers the same thing.
    // Rows of their own here because they are method identities of their own
    // -- `Set~hasItem` is not `Table~hasItem`, which is why adding `Set` to
    // [`OWNED`] alone left it refusing.
    ("Set", "HASITEM", Arity::Fixed(1), native_hash_has_index),
    ("Set", "REMOVEITEM", Arity::Fixed(1), native_hash_remove),
    // **`Relation` and `Bag` have identical native entry-point sets**, which
    // is why they are one task: `Setup.cpp` writes `Bag`'s `AllAt`,
    // `AllIndex`, `Items`, `RemoveAll`, `Supplier` and `UniqueIndexes` as
    // `RelationClass::` bodies, and only `HasItem` and `RemoveItem` are
    // `BagClass::` -- and those two answer the same thing here, because a
    // `Bag`'s index is its item.
    //
    // `ITEMS` and `SUPPLIER` take an optional index where every other class's
    // take none, so they shadow the shared bodies rather than sharing them.
    ("Relation", "ALLAT", Arity::Fixed(1), native_relation_all_at),
    (
        "Relation",
        "ALLINDEX",
        Arity::Fixed(1),
        native_relation_all_index,
    ),
    (
        "Relation",
        "UNIQUEINDEXES",
        Arity::Fixed(0),
        native_relation_unique_indexes,
    ),
    ("Relation", "ITEMS", Arity::Fixed(1), native_relation_items),
    (
        "Relation",
        "REMOVEALL",
        Arity::Fixed(1),
        native_relation_remove_all,
    ),
    (
        "Relation",
        "SUPPLIER",
        Arity::Fixed(1),
        native_relation_supplier,
    ),
    (
        "Relation",
        "HASITEM",
        Arity::Fixed(2),
        native_relation_has_item,
    ),
    (
        "Relation",
        "REMOVEITEM",
        Arity::Fixed(2),
        native_relation_remove_item,
    ),
    ("Bag", "HASITEM", Arity::Fixed(2), native_relation_has_item),
    (
        "Bag",
        "REMOVEITEM",
        Arity::Fixed(2),
        native_relation_remove_item,
    ),
];
