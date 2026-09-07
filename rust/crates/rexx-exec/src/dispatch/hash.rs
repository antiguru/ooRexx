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

/// The five pool entries one store occupies.
///
/// A `Directory` holds two of these at the same scope: the contents every
/// mapped collection has, and the method table `setMethod` fills
/// (`classes/DirectoryClass.cpp:496`). They are the same geometry, which is
/// why this is a name set rather than a second structure -- measured, a
/// directory given the methods `AAA` and `MMM` enumerates them `MMM AAA`,
/// which is the order a plain directory gives those two names as ordinary
/// entries.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Half {
    indexes: &'static [u8],
    items: &'static [u8],
    next: &'static [u8],
    buckets: &'static [u8],
    free: &'static [u8],
}

/// The store every mapped collection has.
const CONTENTS: Half = Half {
    indexes: HASH_INDEXES,
    items: HASH_ITEMS,
    next: HASH_NEXT,
    buckets: HASH_BUCKETS,
    free: HASH_FREE,
};

/// `DirectoryClass::methodTable`, which only a `Directory` ever installs.
const METHODS: Half = Half {
    indexes: b"METHODINDEXES",
    items: b"METHODITEMS",
    next: b"METHODNEXT",
    buckets: b"METHODBUCKETS",
    free: b"METHODFREE",
};

/// `DirectoryClass::unknownMethod`: the one method `setMethod` keeps out of
/// the method table, because `UNKNOWN` is not an entry -- it is what answers
/// when there is none. Measured: with it set, `items` is still 1 for a
/// directory holding one ordinary entry and `allIndexes` does not name it.
const UNKNOWN_METHOD: &[u8] = b"METHODUNKNOWN";

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
const OWNED: &[&str] = &[
    "Table",
    "IdentityTable",
    "Set",
    "Relation",
    "Bag",
    "Directory",
    "StringTable",
    "Properties",
];

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
pub(crate) fn owns(interp: &mut Interp, receiver: ObjRef) -> bool {
    // **A `Body::Native` receiver is never one of these**, however its class
    // reads. `.environment` and `.local` are `Directory`s the bootstrap built
    // on `NativeObject`'s map and keeps there, so they answer the entry
    // family and not the store -- see [`store_of`].
    if matches!(
        interp.heap.get(receiver).map(|object| &object.body),
        Some(Body::Native(_))
    ) {
        return false;
    }
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
fn install_store(interp: &mut Interp, receiver: ObjRef, half: Half, buckets: usize) -> Store {
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
    interp.set_pool_variable(receiver, scope, half.indexes, indexes);
    interp.set_pool_variable(receiver, scope, half.items, items);
    interp.set_pool_variable(receiver, scope, half.next, next);
    interp.set_pool_variable(receiver, scope, half.buckets, buckets_value);
    interp.set_pool_variable(receiver, scope, half.free, free_value);
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
/// The store `half` names, or `None` when the receiver has not installed it.
///
/// A `Directory` installs its method table only when `setMethod` is first
/// sent, so every other class and every directory that has never been sent
/// one pays a single absent-slot read per merged operation.
fn read_store(interp: &mut Interp, receiver: ObjRef, half: Half) -> Result<Option<Store>, Failure> {
    let scope = hash_scope(interp);
    if let (Some(indexes), Some(items), Some(next), Some(buckets), Some(free)) = (
        pool_variable(interp, receiver, scope, half.indexes),
        pool_variable(interp, receiver, scope, half.items),
        pool_variable(interp, receiver, scope, half.next),
        counted_pool_variable(interp, receiver, scope, half.buckets),
        counted_pool_variable(interp, receiver, scope, half.free),
    ) {
        let total = array_slots(interp, indexes)?.len();
        return Ok(Some(Store {
            indexes,
            items,
            next,
            buckets,
            free,
            total,
        }));
    }
    Ok(None)
}

/// The store `half` names, installing an empty one if it is not there yet.
fn store_in(interp: &mut Interp, receiver: ObjRef, half: Half) -> Result<Store, Failure> {
    if half == CONTENTS {
        return store_of(interp, receiver);
    }
    match read_store(interp, receiver, half)? {
        Some(store) => Ok(store),
        None => Ok(install_store(interp, receiver, half, MINIMUM_BUCKET_SIZE)),
    }
}

fn store_of(interp: &mut Interp, receiver: ObjRef) -> Result<Store, Failure> {
    if let Some(store) = read_store(interp, receiver, CONTENTS)? {
        return Ok(store);
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
    Ok(install_store(
        interp,
        receiver,
        CONTENTS,
        MINIMUM_BUCKET_SIZE,
    ))
}

fn set_free(interp: &mut Interp, receiver: ObjRef, half: Half, free: usize) {
    let scope = hash_scope(interp);
    let value = interp.counted(free);
    interp.set_pool_variable(receiver, scope, half.free, value);
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
    /// `StringHashContents`: the index's STRING VALUE, compared as bytes and
    /// hashed by the string hash.
    ///
    /// Measured on a `.Directory`: `d[1] = 'one'` answers to `d['1']` and
    /// reports its index as `1`, so a number and its digits are one key; and
    /// `d[.Array~new] = 'x'` is accepted rather than refused, so a non-string
    /// index is taken by its string value rather than turned away.
    StringValue,
}

/// The classes whose keys are string values.
const STRING_KEYED: &[&str] = &["Directory", "StringTable", "Properties"];

/// Whether `receiver`'s class is one of [`STRING_KEYED`].
pub(crate) fn string_keyed(interp: &mut Interp, receiver: ObjRef) -> bool {
    let Some(class) = interp.class_of_value(receiver) else {
        return false;
    };
    for name in STRING_KEYED {
        let Some(base) = interp.classes().lookup(name) else {
            continue;
        };
        if interp.classes().is_a(class, base) {
            return true;
        }
    }
    false
}

fn keys_of(interp: &mut Interp, receiver: ObjRef) -> Keys {
    let Some(class) = interp.class_of_value(receiver) else {
        return Keys::Equality;
    };
    for name in STRING_KEYED {
        let Some(base) = interp.classes().lookup(name) else {
            continue;
        };
        if interp.classes().is_a(class, base) {
            return Keys::StringValue;
        }
    }
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
    if keys == Keys::StringValue {
        let text = interp.to_text(index).into_owned();
        return Ok(super::string_hash(&text));
    }
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
        Keys::StringValue => {
            let wanted = interp.to_text(index).into_owned();
            let found = interp.to_text(held).into_owned();
            Ok(wanted == found)
        }
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
    probe_in(interp, receiver, store, index)
}

/// [`probe`] against a store the caller has already read, so that a
/// `Directory`'s method table can be searched with the same chain walk.
fn probe_in(
    interp: &mut Interp,
    receiver: ObjRef,
    store: Store,
    index: ObjRef,
) -> Result<(Store, Probe), Failure> {
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
    walk_in(interp, &store)
}

/// [`walk`] over a store the caller has already read.
fn walk_in(interp: &Interp, store: &Store) -> Result<Vec<usize>, Failure> {
    let mut order = Vec::new();
    for bucket in 0..store.buckets {
        let mut slot = bucket;
        while slot < store.total && slot_at(interp, store.indexes, slot)?.is_some() {
            order.push(slot);
            slot = link_at(interp, store, slot)?;
        }
    }
    Ok(order)
}

/// Grows the table and re-adds every entry in old bucket order --
/// `HashCollection::expandContents` and `HashContents::reMerge`.
fn expand(interp: &mut Interp, receiver: ObjRef, half: Half) -> Result<(), Failure> {
    let store = store_in(interp, receiver, half)?;
    let order = walk_in(interp, &store)?;
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
    install_store(interp, receiver, half, buckets);
    for (index, item) in carried {
        // **Never `insert`**, which replaces an index the table already
        // holds: a `Relation` carries duplicate indexes and would lose one
        // per pair on every growth. `reMerge` adds rather than puts
        // (`classes/support/HashContents.cpp:1238`), and adding in the old
        // walk order is what keeps each index's chain in its order.
        append_entry(interp, receiver, half, index, item)?;
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
    half: Half,
    index: ObjRef,
    item: Option<ObjRef>,
) -> Result<(), Failure> {
    let store = store_in(interp, receiver, half)?;
    if store.free >= store.total {
        expand(interp, receiver, half)?;
        return append_entry(interp, receiver, half, index, item);
    }
    let store = store_in(interp, receiver, half)?;
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
    set_free(interp, receiver, half, next_free);
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
        expand(interp, receiver, CONTENTS)?;
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
    set_free(interp, receiver, CONTENTS, next_free);
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
    // `DirectoryClass::put`: a put replaces any method of the same name.
    // Measured: `setMethod('M', 'return 2')` then `d['M'] = 5` answers 5, and
    // `unsetMethod('M')` afterwards still answers 5.
    if method_store(interp, receiver)?.is_some() {
        take_in(interp, receiver, METHODS, index)?;
    }
    insert_in(interp, receiver, CONTENTS, index, item)
}

/// [`insert`] into either half of a `Directory`.
fn insert_in(
    interp: &mut Interp,
    receiver: ObjRef,
    half: Half,
    index: ObjRef,
    item: Option<ObjRef>,
) -> Result<(), Failure> {
    // **The fullness test comes first, and it is the free chain rather than
    // the item count** (`classes/support/HashContents.hpp:297`): a table can
    // run its overflow dry while primary buckets stand empty, and upstream
    // expands before every `put` rather than only when one needs a slot.
    // Measured through the simulation this was written from: testing the item
    // count instead runs the free chain out on the forty-integer case.
    let store = store_in(interp, receiver, half)?;
    if store.free >= store.total {
        expand(interp, receiver, half)?;
    }
    let store = store_in(interp, receiver, half)?;
    let (store, found) = probe_in(interp, receiver, store, index)?;
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
    set_free(interp, receiver, half, next_free);
    write_slot(interp, store.indexes, slot, Some(index));
    write_slot(interp, store.items, slot, item);
    write_link(interp, &store, last, slot);
    write_link(interp, &store, slot, store.no_more());
    Ok(())
}

/// `HashContents::remove`: unlinks the entry and returns its slot to the free
/// chain, answering the item it held.
fn take(interp: &mut Interp, receiver: ObjRef, index: ObjRef) -> Result<Option<ObjRef>, Failure> {
    take_in(interp, receiver, CONTENTS, index)
}

/// [`take`] from either half of a `Directory`.
fn take_in(
    interp: &mut Interp,
    receiver: ObjRef,
    half: Half,
    index: ObjRef,
) -> Result<Option<ObjRef>, Failure> {
    let Some(store) = read_store(interp, receiver, half)? else {
        return Ok(None);
    };
    let (store, found) = probe_in(interp, receiver, store, index)?;
    let Some(slot) = found.found else {
        return Ok(None);
    };
    let previous = found.last.filter(|last| *last != slot);
    remove_at(interp, receiver, half, &store, slot, previous)
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
    remove_at(interp, receiver, CONTENTS, &store, slot, previous)
}

/// Unlinks the entry at `slot`, whose chain predecessor is `previous`, and
/// answers the item it held.
fn remove_at(
    interp: &mut Interp,
    receiver: ObjRef,
    half: Half,
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
            free_slot(interp, receiver, half, store, next);
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
    free_slot(interp, receiver, half, store, slot);
    Ok(item)
}

fn free_slot(interp: &mut Interp, receiver: ObjRef, half: Half, store: &Store, slot: usize) {
    write_slot(interp, store.indexes, slot, None);
    write_slot(interp, store.items, slot, None);
    let free = store.free;
    write_link(interp, store, slot, free);
    set_free(interp, receiver, half, slot);
}

// ---- `Directory`'s method table ----

/// The method table's store, or `None` when `setMethod` has never been sent
/// to this receiver.
fn method_store(interp: &mut Interp, receiver: ObjRef) -> Result<Option<Store>, Failure> {
    read_store(interp, receiver, METHODS)
}

/// `DirectoryClass::unknownMethod`, or `None` when there is none.
///
/// Cleared by writing `.nil` rather than by dropping the pool entry, so that
/// `unsetMethod('UNKNOWN')` needs no delete the pool does not offer.
fn unknown_method(interp: &mut Interp, receiver: ObjRef) -> Option<ObjRef> {
    let scope = hash_scope(interp);
    pool_variable(interp, receiver, scope, UNKNOWN_METHOD).filter(|held| *held != ObjRef::NIL)
}

fn set_unknown_method(interp: &mut Interp, receiver: ObjRef, held: Option<ObjRef>) {
    let scope = hash_scope(interp);
    let value = held.unwrap_or(ObjRef::NIL);
    interp.set_pool_variable(receiver, scope, UNKNOWN_METHOD, value);
}

/// Whether the name is the one `setMethod` keeps outside the table.
fn is_unknown_name(interp: &mut Interp, name: ObjRef) -> bool {
    interp.to_text(name).as_ref() == b"UNKNOWN"
}

/// Runs a stored method with the directory as the receiver.
///
/// The item slot holds the minted [`rexx_core::MethodId`] rather than the
/// `Method` object: nothing Rexx-visible reads the object back out --
/// `Directory` has no method that answers one -- and minting per read would
/// grow `method_bodies` without bound.
fn run_stored_method(
    interp: &mut Interp,
    receiver: ObjRef,
    name: &[u8],
    stored: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let Decoded::SmallInt(id) = stored.decode() else {
        return Err(
            Loud::method_from_source("a directory method entry that holds no method").into(),
        );
    };
    let resolution = super::Resolution {
        scope: ObjRef::NIL,
        method: rexx_core::MethodId(id as u32),
    };
    Ok(interp
        .invoke(resolution, receiver, name, args)?
        .unwrap_or(ObjRef::NIL))
}

/// `DirectoryClass::methodTableValue`: the result of the method stored under
/// `index`, run with NO arguments and under its own name.
fn method_table_value(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
) -> Result<Option<ObjRef>, Failure> {
    let Some(store) = method_store(interp, receiver)? else {
        return Ok(None);
    };
    let (store, found) = probe_in(interp, receiver, store, index)?;
    let Some(slot) = found.found else {
        return Ok(None);
    };
    let Some(stored) = slot_at(interp, store.items, slot)? else {
        return Ok(None);
    };
    let name = interp.to_text(index).to_vec();
    run_stored_method(interp, receiver, &name, stored, &[]).map(Some)
}

/// `DirectoryClass::unknownValue`: run under the name `UNKNOWN` and with the
/// index as its ONE argument -- unlike a method table entry, which is run
/// with none.
fn unknown_value(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
) -> Result<Option<ObjRef>, Failure> {
    let Some(stored) = unknown_method(interp, receiver) else {
        return Ok(None);
    };
    run_stored_method(interp, receiver, b"UNKNOWN", stored, &[Some(index)]).map(Some)
}

/// `DirectoryClass::get`: the contents, then the method table, then the
/// unknown method.
fn merged_get(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
) -> Result<Option<ObjRef>, Failure> {
    let (store, found) = probe(interp, receiver, index)?;
    if let Some(slot) = found.found {
        return Ok(Some(
            slot_at(interp, store.items, slot)?.unwrap_or(ObjRef::NIL),
        ));
    }
    if let Some(value) = method_table_value(interp, receiver, index)? {
        return Ok(Some(value));
    }
    unknown_value(interp, receiver, index)
}

/// `DirectoryClass::hasIndex`: either half, and **not** the unknown method.
/// Measured, with an `UNKNOWN` method set: `hasIndex('nosuch')` is 0 while
/// `d['nosuch']` answers what the method returns.
fn merged_has_index(interp: &mut Interp, receiver: ObjRef, index: ObjRef) -> Result<bool, Failure> {
    let (_, found) = probe(interp, receiver, index)?;
    if found.found.is_some() {
        return Ok(true);
    }
    let Some(store) = method_store(interp, receiver)? else {
        return Ok(false);
    };
    let (_, found) = probe_in(interp, receiver, store, index)?;
    Ok(found.found.is_some())
}

/// Every (index, result) the method table answers, in its own store order.
///
/// Every entry is read out of the table before any of them runs: a body may
/// write to the directory -- measured, a method that increments an entry
/// answers 1, 2 and 3 over three reads -- and a walk interleaved with that
/// would be following a chain its own callee had moved.
fn method_pairs(interp: &mut Interp, receiver: ObjRef) -> Result<Vec<(ObjRef, ObjRef)>, Failure> {
    let Some(store) = method_store(interp, receiver)? else {
        return Ok(Vec::new());
    };
    let mut held = Vec::new();
    for slot in walk_in(interp, &store)? {
        let Some(index) = slot_at(interp, store.indexes, slot)? else {
            continue;
        };
        let Some(stored) = slot_at(interp, store.items, slot)? else {
            continue;
        };
        interp.roots.push_temp(index);
        held.push((index, stored));
    }
    let mut pairs = Vec::with_capacity(held.len());
    for (index, stored) in held {
        let name = interp.to_text(index).to_vec();
        let item = run_stored_method(interp, receiver, &name, stored, &[])?;
        interp.roots.push_temp(item);
        pairs.push((index, item));
    }
    Ok(pairs)
}

/// How many entries the method table holds, WITHOUT running any of them --
/// `DirectoryClass::items` adds the two counts. The unknown method is not one
/// of them: measured, a directory holding one ordinary entry and an `UNKNOWN`
/// method answers `items` as 1.
fn method_count(interp: &mut Interp, receiver: ObjRef) -> Result<usize, Failure> {
    let Some(store) = method_store(interp, receiver)? else {
        return Ok(0);
    };
    Ok(walk_in(interp, &store)?.len())
}

/// `DirectoryClass::remove`: answers what `get` would -- which may run a
/// method, or the unknown method -- and then drops the name from both halves.
///
/// Measured: with an `UNKNOWN` method set, `d~remove('nosuch')` answers what
/// it returns, while `d~removeItem` over the same value answers `.nil`,
/// because `removeItem` goes by `getIndex` and that does not consult it.
fn take_merged(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
) -> Result<Option<ObjRef>, Failure> {
    if method_store(interp, receiver)?.is_none() && unknown_method(interp, receiver).is_none() {
        return take(interp, receiver, index);
    }
    let old = merged_get(interp, receiver, index)?;
    take(interp, receiver, index)?;
    take_in(interp, receiver, METHODS, index)?;
    Ok(old)
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
    Ok(Some(
        merged_get(interp, receiver, index)?.unwrap_or(ObjRef::NIL),
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
    // `DirectoryClass::allIndexes` and its siblings append the method table
    // AFTER the contents, each half in its own store order. Measured: a
    // directory given `AAA`(method), `zzz`, `MMM`(method), `bbb` answers
    // `zzz bbb MMM AAA`.
    pairs.extend(method_pairs(interp, receiver)?);
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
    let count = walk(interp, receiver)?.len() + method_count(interp, receiver)?;
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
    let empty = walk(interp, receiver)?.is_empty() && method_count(interp, receiver)? == 0;
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
    install_store(interp, receiver, CONTENTS, store.buckets);
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
    let held = merged_has_index(interp, receiver, index)?;
    Ok(Some(crate::eval::logical(held)))
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
    Ok(Some(
        take_merged(interp, receiver, index)?.unwrap_or(ObjRef::NIL),
    ))
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
            take_merged(interp, receiver, index)?;
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
    install_store(interp, object, CONTENTS, calculate_bucket_size(capacity));
    let caller = interp.caller();
    interp.send_message(object, super::INIT, None, args, caller)?;
    Ok(Some(object))
}

// ---- `Stem` ----
//
// **`Stem` shares no entry point with the rest of this file**, which is spec
// section 7's reason for giving it a task of its own rather than widening the
// protocol: its store is the language's tails, it is a `MapCollection` by
// inheritance alone, and `Body::Stem` already tells a dropped tail from an
// absent one -- the distinction the collection surface has to respect.

/// The stem's tails, as (name, value) pairs in the map's own order, with a
/// dropped tail's `None` kept.
fn stem_tails(interp: &Interp, receiver: ObjRef) -> Vec<(Vec<u8>, Option<ObjRef>)> {
    match interp.heap.get(receiver).map(|object| &object.body) {
        Some(Body::Stem { tails, .. }) => {
            let mut held: Vec<(usize, Vec<u8>, Option<ObjRef>)> = tails
                .iter()
                .map(|(name, (at, value))| (*at, name.clone(), *value))
                .collect();
            // Insertion order, which is what the tree was built from.
            held.sort_by_key(|(at, ..)| *at);
            held.into_iter()
                .map(|(_, name, value)| (name, value))
                .collect()
        }
        _ => Vec::new(),
    }
}

/// The value a tail read answers.
///
/// **Three cases and not two.** A tail that holds something answers it; a
/// tail that was DROPPED answers its own derived name, because
/// `Body::Stem` keeps the tombstone; and a tail that was never assigned
/// answers the stem's default, or its derived name when there is none.
/// Measured on `s. = 'dflt'` with `s.b` dropped: `at('B')` is `S.B` and
/// `at('ZZ')` is `dflt`.
fn stem_read(interp: &mut Interp, receiver: ObjRef, tail: &[u8]) -> ObjRef {
    let (held, default, name) = match interp.heap.get(receiver).map(|object| &object.body) {
        Some(Body::Stem {
            tails,
            default,
            name,
        }) => (
            tails.get(tail).map(|(_, value)| *value),
            *default,
            name.as_ref().to_vec(),
        ),
        _ => return ObjRef::NIL,
    };
    match held {
        Some(Some(value)) => value,
        Some(None) => {
            let mut derived = name;
            derived.extend_from_slice(tail);
            interp.text_built(derived)
        }
        None => default.unwrap_or_else(|| {
            let mut derived = name;
            derived.extend_from_slice(tail);
            interp.text_built(derived)
        }),
    }
}

fn native_stem_at(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !is_stem(interp, receiver) {
        return Err(stem_refusal(interp, receiver, b"[]"));
    }
    let tail = stem_tail(interp, args)?;
    Ok(Some(stem_read(interp, receiver, &tail)))
}

/// `StemClass::bracketEqual`: the value is argument one and the subscripts
/// follow it, as `Array~put`'s do.
fn native_stem_put(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !is_stem(interp, receiver) {
        return Err(stem_refusal(interp, receiver, b"[]="));
    }
    let value = super::collection::item_argument(args)?;
    let tail = stem_tail(interp, args.get(1..).unwrap_or_default())?;
    stem_write(interp, receiver, tail, Some(value));
    Ok(None)
}

/// Whether `receiver` is a stem, which is what every row below needs and what
/// the shared bodies must not be given.
fn is_stem(interp: &Interp, receiver: ObjRef) -> bool {
    matches!(
        interp.heap.get(receiver).map(|object| &object.body),
        Some(Body::Stem { .. })
    )
}

fn stem_refusal(interp: &mut Interp, receiver: ObjRef, method: &[u8]) -> Failure {
    not_this_task(interp, receiver, method)
}

/// One node of the stem's tail tree, laid out as
/// `CompoundTableElement` is: two links, a parent, and a depth per side
/// rather than one height.
struct TailNode {
    tail: usize,
    left: Option<usize>,
    right: Option<usize>,
    parent: Option<usize>,
    left_depth: u16,
    right_depth: u16,
}

/// `CompoundVariableTail::compare` (`classes/support/CompoundVariableTail.hpp:170`):
/// **length first, bytes second**, which is why the tree's order is not
/// alphabetical.
fn tail_order(left: &[u8], right: &[u8]) -> std::cmp::Ordering {
    left.len().cmp(&right.len()).then_with(|| left.cmp(right))
}

/// The order a stem answers its tails in: post-order over the tree the
/// insertions built.
///
/// **Ported rather than invented, and validated before it was written.**
/// `CompoundVariableTable` is a balanced binary tree, not a hash table: keyed
/// by [`tail_order`], rebalanced by a SINGLE rotation with a depth counter per
/// side (`CompoundVariableTable.cpp:197`, `:273` -- there is no double
/// rotation, which is where a textbook AVL and this part company), and walked
/// in post-order by `first`/`next` (`:343`, `:405`).
///
/// A simulation of exactly that predicted the oracle on four independent
/// cases before any of this existed: five string tails, twenty-five integer
/// tails, seven mixed-length tails, and the five strings inserted backwards --
/// which answers a different order from the same set, and is why the
/// insertion ordinal has to be kept at all.
///
/// **Dropped tails keep their node**, so they shape the tree even though
/// `allIndexes` does not name them.
fn stem_order(tails: &[(Vec<u8>, Option<ObjRef>)]) -> Vec<usize> {
    let mut nodes: Vec<TailNode> = Vec::with_capacity(tails.len());
    let mut root: Option<usize> = None;
    for (at, (tail, _)) in tails.iter().enumerate() {
        let mut anchor = root;
        let mut previous = None;
        let mut side = std::cmp::Ordering::Equal;
        while let Some(node) = anchor {
            side = tail_order(tail, &tails[nodes[node].tail].0);
            match side {
                std::cmp::Ordering::Greater => {
                    previous = Some(node);
                    anchor = nodes[node].right;
                }
                std::cmp::Ordering::Less => {
                    previous = Some(node);
                    anchor = nodes[node].left;
                }
                std::cmp::Ordering::Equal => break,
            }
        }
        if anchor.is_some() {
            continue;
        }
        nodes.push(TailNode {
            tail: at,
            left: None,
            right: None,
            parent: previous,
            left_depth: 0,
            right_depth: 0,
        });
        let fresh = nodes.len() - 1;
        match previous {
            None => root = Some(fresh),
            Some(parent) => {
                if side == std::cmp::Ordering::Greater {
                    nodes[parent].right = Some(fresh);
                } else {
                    nodes[parent].left = Some(fresh);
                }
                rebalance(&mut nodes, &mut root, fresh);
            }
        }
    }
    let Some(root) = root else {
        return Vec::new();
    };
    let mut order = Vec::with_capacity(nodes.len());
    let mut cursor = Some(first_tail(&nodes, root));
    while let Some(node) = cursor {
        order.push(nodes[node].tail);
        let Some(parent) = nodes[node].parent else {
            break;
        };
        cursor = if nodes[parent].right == Some(node) {
            Some(parent)
        } else if let Some(right) = nodes[parent].right {
            Some(first_tail(&nodes, right))
        } else {
            Some(parent)
        };
    }
    order
}

/// `CompoundVariableTable::findLeaf` (`:367`): as far left as possible, then
/// one step right, repeatedly -- the first node of a post-order walk.
fn first_tail(nodes: &[TailNode], mut node: usize) -> usize {
    loop {
        while let Some(left) = nodes[node].left {
            node = left;
        }
        match nodes[node].right {
            None => return node,
            Some(right) => node = right,
        }
    }
}

/// `CompoundVariableTable::moveNode` (`:273`): one rotation, and the depth
/// bookkeeping upstream does with it.
fn move_tail(
    nodes: &mut [TailNode],
    root: &mut Option<usize>,
    anchor: usize,
    toright: bool,
) -> usize {
    let original = nodes[anchor].parent;
    let work = if toright {
        let work = nodes[anchor].left.expect("a left child to rotate");
        let moved = nodes[work].right;
        nodes[anchor].left = moved;
        nodes[anchor].left_depth = nodes[work].right_depth;
        if let Some(moved) = moved {
            nodes[moved].parent = Some(anchor);
        }
        nodes[work].right = Some(anchor);
        nodes[work].right_depth += 1;
        work
    } else {
        let work = nodes[anchor].right.expect("a right child to rotate");
        let moved = nodes[work].left;
        nodes[anchor].right = moved;
        nodes[anchor].right_depth = nodes[work].left_depth;
        if let Some(moved) = moved {
            nodes[moved].parent = Some(anchor);
        }
        nodes[work].left = Some(anchor);
        nodes[work].left_depth += 1;
        work
    };
    nodes[work].parent = original;
    nodes[anchor].parent = Some(work);
    match original {
        None => *root = Some(work),
        Some(parent) => {
            if nodes[parent].left == Some(anchor) {
                nodes[parent].left = Some(work);
            } else {
                nodes[parent].right = Some(work);
            }
        }
    }
    work
}

/// `CompoundVariableTable::balance` (`:197`): walk up from the new node,
/// recording the depth on the side it came from and rotating once wherever
/// that side has run more than one deeper than the other.
fn rebalance(nodes: &mut [TailNode], root: &mut Option<usize>, node: usize) {
    if *root == Some(node) {
        return;
    }
    let mut node = node;
    let mut parent = nodes[node].parent;
    let mut depth: u16 = 1;
    while let Some(mut current) = parent {
        // **Upstream's early return is dead and is not reproduced.** The C++
        // writes `if (depth > workingDepth) { move } else { if (workingDepth
        // < depth) return; }` (`CompoundVariableTable.cpp:221`, `:244`), and
        // the inner test is the outer one again -- it can never hold in the
        // `else`, so the walk always runs to the root. The simulation this was
        // ported from carried the same dead branch and matched the oracle on
        // four cases either way, so dropping it changes no shape; clippy
        // names it as `same condition` and it would be noise to keep.
        if nodes[current].right == Some(node) {
            nodes[current].right_depth = depth;
            if depth > nodes[current].left_depth + 1 {
                current = move_tail(nodes, root, current, false);
                depth = nodes[current].right_depth;
            }
        } else {
            nodes[current].left_depth = depth;
            if depth > nodes[current].right_depth + 1 {
                current = move_tail(nodes, root, current, true);
                depth = nodes[current].left_depth;
            }
        }
        depth += 1;
        node = current;
        parent = nodes[current].parent;
    }
}

/// Every tail that holds something, which is what `items` counts and
/// `allIndexes` names.
///
/// **A dropped tail is not one of them.** Measured, `s.a = 1`, `s.b = 2`,
/// `s.c = 3` then `drop s.b` leaves `items` at 2 and `allIndexes` at `A,C`.
fn stem_live(interp: &Interp, receiver: ObjRef) -> Vec<(Vec<u8>, ObjRef)> {
    let tails = stem_tails(interp, receiver);
    stem_order(&tails)
        .into_iter()
        .filter_map(|at| {
            let (name, value) = &tails[at];
            Some((name.clone(), (*value)?))
        })
        .collect()
}

fn native_stem_all_indexes(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !is_stem(interp, receiver) {
        return Err(stem_refusal(interp, receiver, b"ALLINDEXES"));
    }
    // **Rooted as they are built**, for `ordered_pairs`'s reason: each name is
    // a fresh allocation and the next one collects, so an unrooted earlier
    // name is swept before the array holds it. Measured -- without these,
    // `collect_stress` panics at `not_in_arena`'s "a live value".
    let mut names = Vec::new();
    for (name, _) in stem_live(interp, receiver) {
        let name = interp.text_built(name);
        interp.roots.push_temp(name);
        names.push(name);
    }
    Ok(Some(super::collection::array_of(interp, names)))
}

fn native_stem_all_items(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !is_stem(interp, receiver) {
        return Err(stem_refusal(interp, receiver, b"ALLITEMS"));
    }
    let items: Vec<ObjRef> = stem_live(interp, receiver)
        .into_iter()
        .map(|(_, value)| value)
        .collect();
    // The items are the stem's own and are reachable through it, but the
    // array below allocates, so they are held for that.
    for item in &items {
        interp.roots.push_temp(*item);
    }
    Ok(Some(super::collection::array_of(interp, items)))
}

/// `makeArray` answers the INDEXES here as it does for every other class in
/// this phase -- measured, `A,C` for a stem holding `A` and `C`, not `1,3`.
fn native_stem_make_array(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    native_stem_all_indexes(interp, cleared, receiver, args)
}

fn native_stem_supplier(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !is_stem(interp, receiver) {
        return Err(stem_refusal(interp, receiver, b"SUPPLIER"));
    }
    let live = stem_live(interp, receiver);
    let items: Vec<ObjRef> = live.iter().map(|(_, value)| *value).collect();
    for item in &items {
        interp.roots.push_temp(*item);
    }
    // Rooted as they are built -- see [`native_stem_all_indexes`].
    let mut indexes = Vec::new();
    for (name, _) in live {
        let name = interp.text_built(name);
        interp.roots.push_temp(name);
        indexes.push(name);
    }
    let items = super::collection::array_of(interp, items);
    let indexes = super::collection::array_of(interp, indexes);
    super::collection::new_supplier(interp, items, indexes).map(Some)
}

fn native_stem_items(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !is_stem(interp, receiver) {
        return Err(stem_refusal(interp, receiver, b"ITEMS"));
    }
    let count = stem_live(interp, receiver).len();
    Ok(Some(interp.counted(count)))
}

fn native_stem_is_empty(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !is_stem(interp, receiver) {
        return Err(stem_refusal(interp, receiver, b"ISEMPTY"));
    }
    let empty = stem_live(interp, receiver).is_empty();
    Ok(Some(crate::eval::logical(empty)))
}

/// The tail a subscript list names: the arguments' string values joined with
/// `.`, **exactly as written**.
///
/// **The method path does not upper-case and the language path does.** A
/// symbol in `s.a` is upper-cased when it is parsed, so the tail is `A`; a
/// subscript in `obj['a']` is a string and stays `a`. Measured, and every
/// line of it separates the two: after `s.a = 1`, `obj['A']` is 1 while
/// `obj['a']` is the unset `S.a`; `obj['b'] = 2` leaves `allIndexes` reading
/// `b,A`; and `s.b` afterwards is `S.B`, a different tail from the `b` the
/// method wrote.
///
/// Task 5 upper-cased here and every probe it had used upper-case literals,
/// so nothing saw it. Task 6's populated receiver -- `.Stem~new~~put('v1',
/// 'k1')` -- is what made the method-body table report `allIndexes` and
/// `makeArray` diverging.
fn stem_tail(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<Vec<u8>, Failure> {
    let mut tail = Vec::new();
    for (at, argument) in args.iter().enumerate() {
        let Some(argument) = *argument else {
            return Err(Raised::missing_method_argument(at + 1).into());
        };
        if !tail.is_empty() {
            tail.push(b'.');
        }
        tail.extend_from_slice(&interp.to_text(argument));
    }
    Ok(tail)
}

fn native_stem_has_index(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !is_stem(interp, receiver) {
        return Err(stem_refusal(interp, receiver, b"HASINDEX"));
    }
    // **No subscripts is true.** Measured, `obj~hasIndex()` on a stem holding
    // one tail answers `1` -- the stem itself is the index a bare `hasIndex`
    // asks about, and none of `Stem`'s rows treats a missing subscript as an
    // error the way the hash classes do.
    if args.is_empty() {
        return Ok(Some(crate::eval::logical(true)));
    }
    let tail = stem_tail(interp, args)?;
    let held = stem_live(interp, receiver)
        .into_iter()
        .any(|(name, _)| name == tail);
    Ok(Some(crate::eval::logical(held)))
}

fn native_stem_has_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !is_stem(interp, receiver) {
        return Err(stem_refusal(interp, receiver, b"HASITEM"));
    }
    // Optional, unlike every other class's, where a missing item is 93.903.
    //
    // **Measured against an EMPTIED stem, and it cannot be measured any other
    // way.** `hasItem()` and `index()` with no argument are a SIGSEGV on the
    // oracle whenever the stem holds a tail -- both are in
    // `corpus/oracle-crashes.txt`, and the probe that answered `0` here got
    // that answer only because an earlier line in it had emptied the stem.
    // So these two limbs are written from the one shape the oracle survives
    // and nothing in the tree witnesses them; `stem_collection.rex` says so
    // and leaves them out rather than asserting an answer it cannot check.
    let Some(wanted) = args.first().copied().flatten() else {
        return Ok(Some(crate::eval::logical(false)));
    };
    for (_, value) in stem_live(interp, receiver) {
        if super::collection::same_item(interp, wanted, value)? {
            return Ok(Some(crate::eval::logical(true)));
        }
    }
    Ok(Some(crate::eval::logical(false)))
}

fn native_stem_index(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !is_stem(interp, receiver) {
        return Err(stem_refusal(interp, receiver, b"INDEX"));
    }
    let Some(wanted) = args.first().copied().flatten() else {
        return Ok(Some(ObjRef::NIL));
    };
    for (name, value) in stem_live(interp, receiver) {
        if super::collection::same_item(interp, wanted, value)? {
            return Ok(Some(interp.text_built(name)));
        }
    }
    Ok(Some(ObjRef::NIL))
}

/// Writes one tail, `None` dropping it.
fn stem_write(interp: &mut Interp, receiver: ObjRef, tail: Vec<u8>, value: Option<ObjRef>) {
    if let Some(Body::Stem { tails, .. }) = interp.heap.get_mut(receiver).map(|held| &mut held.body)
    {
        // The ordinal survives a drop and an overwrite -- see `Body::Stem`.
        let next = tails.len();
        let ordinal = tails.get(&tail).map_or(next, |(at, _)| *at);
        tails.insert(tail, (ordinal, value));
    }
}

/// `remove(tail)`: drops the tail and answers what it held, or `.nil`.
///
/// **A drop and not a delete.** `Body::Stem` keeps the tombstone, which is
/// what makes a dropped tail read as its own derived name where an absent one
/// reads as the stem's default -- measured, `drop s.b` then `s~at('B')` is
/// `S.B` while `s~at('ZZ')` is the default.
fn native_stem_remove(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !is_stem(interp, receiver) {
        return Err(stem_refusal(interp, receiver, b"REMOVE"));
    }
    // No subscripts names the stem itself, which is not a tail: measured,
    // `obj~remove()` answers `S.` and takes nothing out, where
    // `obj~remove('ZZ')` answers `.nil`.
    if args.is_empty() {
        return Ok(Some(stem_read(interp, receiver, b"")));
    }
    let tail = stem_tail(interp, args)?;
    let held = stem_live(interp, receiver)
        .into_iter()
        .find(|(name, _)| *name == tail)
        .map(|(_, value)| value);
    if held.is_some() {
        stem_write(interp, receiver, tail, None);
    }
    Ok(Some(held.unwrap_or(ObjRef::NIL)))
}

fn native_stem_remove_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !is_stem(interp, receiver) {
        return Err(stem_refusal(interp, receiver, b"REMOVEITEM"));
    }
    let wanted = super::collection::item_argument(args)?;
    for (name, value) in stem_live(interp, receiver) {
        if super::collection::same_item(interp, wanted, value)? {
            stem_write(interp, receiver, name, None);
            return Ok(Some(value));
        }
    }
    Ok(Some(ObjRef::NIL))
}

/// `empty`: drops every tail and **keeps the default** -- measured, `u. = 5`
/// with one tail set answers `items` 0 after `empty` and `u~at('K')` still
/// answers `5`.
fn native_stem_empty(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !is_stem(interp, receiver) {
        return Err(stem_refusal(interp, receiver, b"EMPTY"));
    }
    // **Deleted, not dropped.** A tombstone reads as the tail's own derived
    // name; the oracle's `empty` leaves the tails never-assigned, so they
    // read as the stem's default. Measured: `u. = 5` with one tail set
    // answers `items` 0 after `empty` and `u~at('K')` still answers `5`,
    // where writing tombstones answered `U.K`.
    let default = match interp.heap.get(receiver).map(|object| &object.body) {
        Some(Body::Stem { default, .. }) => *default,
        _ => None,
    };
    if let Some(Body::Stem { tails, .. }) = interp.heap.get_mut(receiver).map(|held| &mut held.body)
    {
        *tails = rexx_core::NameMap::default();
    }
    let _ = default;
    // Answers the receiver, as `Array~empty` and `List~empty` do -- measured,
    // `(obj~empty == obj)` is `1`.
    Ok(Some(receiver))
}

// ---- the string-keyed classes' own accessors ----

/// The name an entry method looks up, which is **upper-cased** where the
/// index family's is not.
///
/// Measured on a `.Directory`: `d~setEntry('viaEntry', 9)` leaves
/// `allIndexes` naming `VIAENTRY`, `d['viaEntry']` answering `.nil` and
/// `d['VIAENTRY']` answering 9, and `d~entry('lower')` after `d['lower'] = 1`
/// is `.nil` -- so the two families read one store through different names.
fn entry_name(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    let Some(name) = args.first().copied().flatten() else {
        return Err(Raised::missing_named_argument("index").into());
    };
    let upper = interp.to_text(name).to_ascii_uppercase();
    Ok(interp.text_built(upper))
}

/// Every index the receiver's store holds, in the store's own order, or an
/// empty list for a receiver that has no store -- what
/// [`crate::Interp::native_keys`] reads for a method table a program built.
pub(crate) fn store_indexes(interp: &mut Interp, receiver: ObjRef) -> Vec<ObjRef> {
    let scope = hash_scope(interp);
    if pool_variable(interp, receiver, scope, HASH_INDEXES).is_none() {
        return Vec::new();
    }
    let Ok(store) = store_of(interp, receiver) else {
        return Vec::new();
    };
    let Ok(order) = walk(interp, receiver) else {
        return Vec::new();
    };
    order
        .into_iter()
        .filter_map(|slot| slot_at(interp, store.indexes, slot).ok().flatten())
        .collect()
}

/// The item the receiver's store holds under `index`.
pub(crate) fn store_item(interp: &mut Interp, receiver: ObjRef, index: ObjRef) -> Option<ObjRef> {
    let (store, found) = probe(interp, receiver, index).ok()?;
    let slot = found.found?;
    slot_at(interp, store.items, slot).ok().flatten()
}

/// The `UNKNOWN` entry read, over the store rather than the environment's map.
pub(super) fn store_entry_read(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(
        merged_get(interp, receiver, index)?.unwrap_or(ObjRef::NIL),
    ))
}

/// [`store_entry_read`]'s other half.
pub(super) fn store_entry_write(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
    item: ObjRef,
) -> Result<Option<ObjRef>, Failure> {
    insert(interp, receiver, index, Some(item))?;
    Ok(None)
}

fn native_hash_entry(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"ENTRY"));
    }
    let name = entry_name(interp, args)?;
    Ok(Some(
        merged_get(interp, receiver, name)?.unwrap_or(ObjRef::NIL),
    ))
}

fn native_hash_has_entry(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"HASENTRY"));
    }
    let name = entry_name(interp, args)?;
    let held = merged_has_index(interp, receiver, name)?;
    Ok(Some(crate::eval::logical(held)))
}

/// `~setEntry(name [, value])`: stores under the upper-cased name, and
/// **removes the entry when the value is omitted** -- measured,
/// `d~setEntry('beta')` after `d~setEntry('beta', 2)` leaves `items` at 0.
fn native_hash_set_entry(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"SETENTRY"));
    }
    let name = entry_name(interp, args)?;
    match args.get(1).copied().flatten() {
        Some(value) => insert(interp, receiver, name, Some(value))?,
        None => {
            take_merged(interp, receiver, name)?;
        }
    }
    Ok(None)
}

/// `DirectoryClass::setMethodRexx` (`classes/DirectoryClass.cpp:480`):
/// attaches a method whose RESULT is the entry's item, recomputed on every
/// read.
///
/// The index is `stringArgument(name, "index")->upper()`, so an omitted one
/// is the named 88.901 and a number is fine -- measured, `setMethod()` is 88
/// while `setMethod(5, 'return 1')` is rc 0. A third argument is 93, which
/// the `Fixed(2)` arity reports.
///
/// An omitted method is a REMOVAL rather than an error, and either way the
/// name is dropped from the contents: `contents->remove(entryname)` runs on
/// both branches. Measured: `g['AAA'] = 'value'` then
/// `setMethod('AAA', 'return 1')` then `unsetMethod('AAA')` leaves the
/// directory empty, because the ordinary entry was destroyed rather than
/// shadowed.
fn native_directory_set_method(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"SETMETHOD"));
    }
    let name = entry_name(interp, args)?;
    interp.roots.push_temp(name);
    let unknown = is_unknown_name(interp, name);
    match args.get(1).copied().flatten() {
        Some(source) => {
            let body = super::run_method_body(interp, source)?;
            let method = interp.classes().mint_method_id();
            interp.method_bodies.insert(method, body);
            let stored = interp.counted(method.0 as usize);
            if unknown {
                set_unknown_method(interp, receiver, Some(stored));
            } else {
                insert_in(interp, receiver, METHODS, name, Some(stored))?;
            }
        }
        None if unknown => set_unknown_method(interp, receiver, None),
        None => {
            take_in(interp, receiver, METHODS, name)?;
        }
    }
    take(interp, receiver, name)?;
    Ok(None)
}

/// `DirectoryClass::unsetMethodRexx`: drops a method, and unlike `setMethod`
/// leaves the contents alone.
fn native_directory_unset_method(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"UNSETMETHOD"));
    }
    let name = entry_name(interp, args)?;
    if is_unknown_name(interp, name) {
        set_unknown_method(interp, receiver, None);
    } else {
        take_in(interp, receiver, METHODS, name)?;
    }
    Ok(None)
}

fn native_hash_remove_entry(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"REMOVEENTRY"));
    }
    let name = entry_name(interp, args)?;
    Ok(Some(
        take_merged(interp, receiver, name)?.unwrap_or(ObjRef::NIL),
    ))
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
            let same = match keys {
                Keys::Identity => *held == index,
                Keys::Equality => super::collection::same_item(interp, index, *held)?,
                Keys::StringValue => {
                    let wanted = interp.to_text(index).into_owned();
                    let found = interp.to_text(*held).into_owned();
                    wanted == found
                }
            };
            if same {
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
    install_store(interp, object, CONTENTS, MINIMUM_BUCKET_SIZE);
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
    install_store(interp, object, CONTENTS, MINIMUM_BUCKET_SIZE);
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
    // `StringHashCollection`'s four, which read the same store the index
    // family reads and upper-case the name on the way in.
    ("Directory", "ENTRY", Arity::Fixed(1), native_hash_entry),
    (
        "Directory",
        "HASENTRY",
        Arity::Fixed(1),
        native_hash_has_entry,
    ),
    (
        "Directory",
        "SETENTRY",
        Arity::Fixed(2),
        native_hash_set_entry,
    ),
    (
        "Directory",
        "REMOVEENTRY",
        Arity::Fixed(1),
        native_hash_remove_entry,
    ),
    // `Directory`'s own two, which override `Object`'s private pair. Neither
    // reaches `StringTable`: `setMethodRexx` is declared only in
    // `DirectoryClass.cpp`, and a `setMethod` send to a `StringTable` falls
    // through to its `unknown` and answers `.nil`.
    (
        "Directory",
        "SETMETHOD",
        Arity::Fixed(2),
        native_directory_set_method,
    ),
    (
        "Directory",
        "UNSETMETHOD",
        Arity::Fixed(1),
        native_directory_unset_method,
    ),
    ("Stem", "AT", Arity::Counted, native_stem_at),
    ("Stem", "[]", Arity::Counted, native_stem_at),
    ("Stem", "PUT", Arity::Counted, native_stem_put),
    ("Stem", "[]=", Arity::Counted, native_stem_put),
    ("Stem", "ITEMS", Arity::Fixed(0), native_stem_items),
    (
        "Stem",
        "ALLINDEXES",
        Arity::Fixed(0),
        native_stem_all_indexes,
    ),
    ("Stem", "ALLITEMS", Arity::Fixed(0), native_stem_all_items),
    ("Stem", "MAKEARRAY", Arity::Fixed(0), native_stem_make_array),
    ("Stem", "SUPPLIER", Arity::Fixed(0), native_stem_supplier),
    ("Stem", "ISEMPTY", Arity::Fixed(0), native_stem_is_empty),
    ("Stem", "HASINDEX", Arity::Counted, native_stem_has_index),
    ("Stem", "HASITEM", Arity::Fixed(1), native_stem_has_item),
    ("Stem", "INDEX", Arity::Fixed(1), native_stem_index),
    ("Stem", "REMOVE", Arity::Counted, native_stem_remove),
    (
        "Stem",
        "REMOVEITEM",
        Arity::Fixed(1),
        native_stem_remove_item,
    ),
    ("Stem", "EMPTY", Arity::Fixed(0), native_stem_empty),
    ("Bag", "HASITEM", Arity::Fixed(2), native_relation_has_item),
    (
        "Bag",
        "REMOVEITEM",
        Arity::Fixed(2),
        native_relation_remove_item,
    ),
];
