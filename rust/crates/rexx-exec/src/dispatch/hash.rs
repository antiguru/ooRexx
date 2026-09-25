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

use super::collection;
use super::required_string_named_argument;
use super::{
    Arity, BehaviourId, Body, Cleared, Decoded, Failure, Interp, Loud, NativeMethod, ObjRef,
    Raised, array_slots, new_instance, required_string_argument, unconverted_array_argument,
};

// `Relation`'s and `Bag`'s own primitive methods, chained into
// `ObjectModel::build` after this module's own.
pub(super) mod relation;

// `Stem`'s primitive methods, chained into `ObjectModel::build` after this
// module's own.
pub(super) mod stem;

/// The five pool entries a hash store is, bound in the receiver's own pool
/// under the `MapCollection` class as scope.
const HASH_INDEXES: &[u8] = b"HASHINDEXES";
const HASH_ITEMS: &[u8] = b"HASHITEMS";
const HASH_NEXT: &[u8] = b"HASHNEXT";
const HASH_BUCKETS: &[u8] = b"HASHBUCKETS";
const HASH_FREE: &[u8] = b"HASHFREE";

/// The five pool entries one store occupies.
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

const METHOD_INDEXES: &[u8] = b"METHODINDEXES";
const METHOD_ITEMS: &[u8] = b"METHODITEMS";
const METHOD_NEXT: &[u8] = b"METHODNEXT";
const METHOD_BUCKETS: &[u8] = b"METHODBUCKETS";
const METHOD_FREE: &[u8] = b"METHODFREE";

/// `DirectoryClass::methodTable`, which only a `Directory` ever installs.
const METHODS: Half = Half {
    indexes: METHOD_INDEXES,
    items: METHOD_ITEMS,
    next: METHOD_NEXT,
    buckets: METHOD_BUCKETS,
    free: METHOD_FREE,
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
fn hash_scope(interp: &mut Interp) -> ObjRef {
    interp.object_model().table
}

/// The two classes this task owns.
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
    // reads: a `Directory` built on `NativeObject`'s map, such as a condition
    // object, has no store -- see [`store_of`].
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
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

fn counted(value: ObjRef) -> Option<usize> {
    match value.decode() {
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
    bump_store_generation(interp);
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
fn read_store(interp: &mut Interp, receiver: ObjRef, half: Half) -> Result<Option<Store>, Failure> {
    let parts = read_parts(interp, receiver);
    let found = if half == CONTENTS {
        parts.contents
    } else {
        parts.methods
    };
    found.store(interp)
}

/// One half's pool entries, as [`read_parts`] found them.
#[derive(Clone, Copy, Default)]
struct Found {
    indexes: Option<ObjRef>,
    items: Option<ObjRef>,
    next: Option<ObjRef>,
    buckets: Option<usize>,
    free: Option<usize>,
}

impl Found {
    /// The store these entries make, or `None` when any is missing.
    fn store(self, interp: &Interp) -> Result<Option<Store>, Failure> {
        let (Some(indexes), Some(items), Some(next), Some(buckets), Some(free)) =
            (self.indexes, self.items, self.next, self.buckets, self.free)
        else {
            return Ok(None);
        };
        let total = array_slots(interp, indexes)?.len();
        Ok(Some(Store {
            indexes,
            items,
            next,
            buckets,
            free,
            total,
        }))
    }
}

/// Both halves' entries and the unknown method, from one pass over the pool.
struct Parts {
    contents: Found,
    methods: Found,
    unknown: Option<ObjRef>,
}

fn read_parts(interp: &mut Interp, receiver: ObjRef) -> Parts {
    let scope = hash_scope(interp);
    let mut parts = Parts {
        contents: Found::default(),
        methods: Found::default(),
        unknown: None,
    };
    let Some(Body::Instance { pools, .. }) = interp.heap.get(receiver).map(|object| &object.body)
    else {
        return parts;
    };
    for (name, value) in pools.entries(scope) {
        let value = *value;
        match name.as_ref() {
            HASH_INDEXES => parts.contents.indexes = Some(value),
            HASH_ITEMS => parts.contents.items = Some(value),
            HASH_NEXT => parts.contents.next = Some(value),
            HASH_BUCKETS => parts.contents.buckets = counted(value),
            HASH_FREE => parts.contents.free = counted(value),
            METHOD_INDEXES => parts.methods.indexes = Some(value),
            METHOD_ITEMS => parts.methods.items = Some(value),
            METHOD_NEXT => parts.methods.next = Some(value),
            METHOD_BUCKETS => parts.methods.buckets = counted(value),
            METHOD_FREE => parts.methods.free = counted(value),
            UNKNOWN_METHOD => parts.unknown = Some(value).filter(|held| *held != ObjRef::NIL),
            _ => {}
        }
    }
    parts
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
    bump_store_generation(interp);
}

/// Marks every [`StoreView`] taken before this call as stale.
fn bump_store_generation(interp: &mut Interp) {
    interp.store_generation = interp.store_generation.wrapping_add(1);
}

fn slot_at(interp: &Interp, array: ObjRef, slot: usize) -> Result<Option<ObjRef>, Failure> {
    Ok(array_slots(interp, array)?.get(slot).copied().flatten())
}

/// The item at `slot`, for a read that answers it or compares it rather than
/// moving it.
///
/// Refuses an entry `.environment` or `.local` holds on the oracle and this
/// crate does not build.
fn item_at(interp: &mut Interp, store: &Store, slot: usize) -> Result<Option<ObjRef>, Failure> {
    let item = slot_at(interp, store.items, slot)?;
    if let Some(held) = item
        && let Some(owner) = interp.owed_entry_owner(held)
    {
        let index = slot_at(interp, store.indexes, slot)?;
        let name = index
            .map(|index| interp.to_text(index).into_owned())
            .unwrap_or_default();
        return Err(Loud::environment_entry(&name, owner).into());
    }
    Ok(item)
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

/// `RexxInternalObject::getHashValue()`, which is what the identity contents
/// hashes with and what the equality contents falls back on for a base class.
fn get_hash_value(interp: &mut Interp, value: ObjRef) -> u64 {
    super::hash_value(interp, value)
}

/// `RexxString::getObjectHashCode` (`classes/StringClass.cpp:220`): a
/// `HASHCODE` answer turned back into a number.
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
    // `RexxObject::hash` branches on whether the behaviour is a primitive one
    // (`classes/ObjectClass.cpp:416`). **"Has a string value" and "is a
    // base-class object" are different questions**, which is spec D96's trap:
    // a `String` subclass has a string value and is not a base class, so its
    // `HASHCODE` override is reached where a plain string's -- which cannot
    // exist, since `.String~define` is 98.985 -- would not be.
    if keys == Keys::Identity || interp.is_base_class(index) {
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
    // `.local['OUTPUT'] = x` moves a route's far end.
    interp.bump_route_generation();
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
    interp.bump_route_generation();
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
fn unknown_method(interp: &mut Interp, receiver: ObjRef) -> Option<ObjRef> {
    let scope = hash_scope(interp);
    pool_variable(interp, receiver, scope, UNKNOWN_METHOD).filter(|held| *held != ObjRef::NIL)
}

fn set_unknown_method(interp: &mut Interp, receiver: ObjRef, held: Option<ObjRef>) {
    let scope = hash_scope(interp);
    let value = held.unwrap_or(ObjRef::NIL);
    interp.set_pool_variable(receiver, scope, UNKNOWN_METHOD, value);
    bump_store_generation(interp);
}

/// Whether the name is the one `setMethod` keeps outside the table.
fn is_unknown_name(interp: &mut Interp, name: ObjRef) -> bool {
    interp.to_text(name).as_ref() == b"UNKNOWN"
}

/// Runs a stored method with the directory as the receiver.
fn run_stored_method(
    interp: &mut Interp,
    receiver: ObjRef,
    name: &[u8],
    stored: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    // An entry holding an object rather than a method answers that object:
    // `.environment`'s `LOCAL`, which `Setup.cpp:1781` installs as a method
    // running `ActivityManager::getLocalRexx`.
    let Decoded::SmallInt(id) = stored.decode() else {
        return Ok(stored);
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
        return Ok(Some(item_at(interp, &store, slot)?.unwrap_or(ObjRef::NIL)));
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

/// Whether a removal refuses an owed entry before taking it: `Answered` for
/// one that hands the item to the program unread, `Discarded` for one that
/// drops it or whose caller has already read it through [`item_at`].
#[derive(Clone, Copy, PartialEq, Eq)]
enum Removal {
    Answered,
    Discarded,
}

/// `DirectoryClass::remove`: answers what `get` would -- which may run a
/// method, or the unknown method -- and then drops the name from both halves.
///
/// An [`Removal::Answered`] removal refuses an owed entry; a discarded one
/// takes it without reading it.
fn take_merged(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
    removal: Removal,
) -> Result<Option<ObjRef>, Failure> {
    if method_store(interp, receiver)?.is_none() && unknown_method(interp, receiver).is_none() {
        if removal == Removal::Answered {
            let (store, found) = probe(interp, receiver, index)?;
            if let Some(slot) = found.found {
                item_at(interp, &store, slot)?;
            }
        }
        return take(interp, receiver, index);
    }
    // `get` runs a method only where the contents miss, so a contents entry
    // that is discarded needs no read.
    let old = if removal == Removal::Discarded && probe(interp, receiver, index)?.1.found.is_some()
    {
        None
    } else {
        merged_get(interp, receiver, index)?
    };
    take(interp, receiver, index)?;
    take_in(interp, receiver, METHODS, index)?;
    Ok(old)
}

// ---- the shared surface ----

/// The index argument the store's index-taking methods require.
fn index_argument(args: &[Option<ObjRef>], position: usize) -> Result<ObjRef, Failure> {
    args.get(position - 1)
        .copied()
        .flatten()
        .ok_or_else(|| Raised::missing_named_argument("index").into())
}

/// `validateIndex`: a string-keyed collection takes its index as
/// `stringArgument(index, "index")` (`classes/support/HashCollection.cpp:1112`),
/// and every other collection takes it as given.
fn validated_index(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
) -> Result<ObjRef, Failure> {
    if keys_of(interp, receiver) == Keys::StringValue {
        return super::required_string_named_argument(interp, index, "index");
    }
    Ok(index)
}

/// `HashCollection::getRexx` and `[]`: the item under that index, or `.nil`.
pub(super) fn store_at(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let index = index_argument(args, 1)?;
    let index = validated_index(interp, receiver, index)?;
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
        let index = index_argument(args, 2)?;
        validated_index(interp, receiver, index)?
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
        let item = item_at(interp, &store, slot)?.unwrap_or(ObjRef::NIL);
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

/// Every index the receiver holds, in [`pairs`]'s order, **running no
/// method** -- `DirectoryClass::allIndexes` appends
/// `methodTable->allIndexes()`, which reads names only.
fn indexes(interp: &mut Interp, receiver: ObjRef) -> Result<Vec<ObjRef>, Failure> {
    let mut indexes = Vec::new();
    let store = store_of(interp, receiver)?;
    let methods = method_store(interp, receiver)?;
    for half in std::iter::once(store).chain(methods) {
        for slot in walk_in(interp, &half)? {
            if let Some(index) = slot_at(interp, half.indexes, slot)? {
                interp.roots.push_temp(index);
                indexes.push(index);
            }
        }
    }
    Ok(indexes)
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
    let indexes = indexes(interp, receiver)?;
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
    let index = validated_index(interp, receiver, index)?;
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
    let index = validated_index(interp, receiver, index)?;
    Ok(Some(
        take_merged(interp, receiver, index, Removal::Answered)?.unwrap_or(ObjRef::NIL),
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
            take_merged(interp, receiver, index, Removal::Discarded)?;
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

// ---- the string-keyed classes' own accessors ----

/// The name an entry method looks up, which is **upper-cased** where the
/// index family's is not.
fn entry_name(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    let Some(name) = args.first().copied().flatten() else {
        return Err(Raised::missing_named_argument("index").into());
    };
    let name = super::required_string_named_argument(interp, name, "index")?;
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

/// A fresh `Directory` whose store is sized for `capacity` entries.
pub(crate) fn new_directory(interp: &mut Interp, capacity: usize) -> Result<ObjRef, Failure> {
    let class = interp.object_model().directory;
    let directory = new_instance(interp, class)?;
    install_store(interp, directory, CONTENTS, calculate_bucket_size(capacity));
    Ok(directory)
}

/// `DirectoryClass::put` of `item` under the string `name`.
pub(crate) fn directory_put(
    interp: &mut Interp,
    directory: ObjRef,
    name: &[u8],
    item: ObjRef,
) -> Result<(), Failure> {
    let index = interp.text(name);
    interp.roots.push_temp(index);
    insert(interp, directory, index, Some(item))
}

/// Puts `value` into the method-table half under `name`, as an entry that
/// answers `value` when run.
pub(crate) fn directory_put_method_value(
    interp: &mut Interp,
    directory: ObjRef,
    name: &[u8],
    value: ObjRef,
) -> Result<(), Failure> {
    let index = interp.text(name);
    interp.roots.push_temp(index);
    insert_in(interp, directory, METHODS, index, Some(value))
}

/// What `DirectoryClass::get` finds for a name.
pub(crate) enum DirectoryEntry {
    Found(ObjRef),
    /// An entry the oracle holds and this crate does not build, with the
    /// phase owing it.
    Owed(&'static str),
    Absent,
}

/// A reading of one directory's pool entries: both halves and the unknown
/// method, valid while no store's pool entries have been written since.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct StoreView {
    generation: u64,
    contents: Store,
    methods: Option<Store>,
    unknown: Option<ObjRef>,
}

fn read_view(interp: &mut Interp, directory: ObjRef) -> Result<StoreView, Failure> {
    let parts = read_parts(interp, directory);
    let contents = match parts.contents.store(interp)? {
        Some(store) => store,
        None => store_of(interp, directory)?,
    };
    let methods = parts.methods.store(interp)?;
    Ok(StoreView {
        generation: interp.store_generation,
        contents,
        methods,
        unknown: parts.unknown,
    })
}

/// A name a string-keyed store is searched for, with its string hash.
#[derive(Clone, Copy)]
pub(crate) struct Key<'a> {
    name: &'a [u8],
    hash: u64,
}

impl<'a> Key<'a> {
    pub(crate) fn new(name: &'a [u8]) -> Key<'a> {
        Key {
            name,
            hash: super::string_hash(name),
        }
    }
}

/// `DirectoryClass::get` over a string-keyed store, by the name's bytes: the
/// contents, then the method table, then the unknown method.
///
/// `view` is a reading of `directory`'s pool from an earlier call, used when
/// still valid. Returns the answer, and the reading this call made when it
/// could not use `view`.
pub(crate) fn directory_get(
    interp: &mut Interp,
    directory: ObjRef,
    key: Key<'_>,
    view: Option<&StoreView>,
) -> Result<(DirectoryEntry, Option<StoreView>), Failure> {
    if let Some(view) = view
        && view.generation == interp.store_generation
    {
        debug_assert_eq!(
            Some(*view),
            read_view(interp, directory).ok(),
            "a StoreView was reused after its directory's pool changed without a \
             bump_store_generation"
        );
        return Ok((view_get(interp, directory, key, view)?, None));
    }
    let view = read_view(interp, directory)?;
    Ok((view_get(interp, directory, key, &view)?, Some(view)))
}

fn view_get(
    interp: &mut Interp,
    directory: ObjRef,
    key: Key<'_>,
    view: &StoreView,
) -> Result<DirectoryEntry, Failure> {
    let name = key.name;
    let store = view.contents;
    if let Some(slot) = text_slot(interp, &store, key)? {
        let item = slot_at(interp, store.items, slot)?.unwrap_or(ObjRef::NIL);
        return Ok(match interp.owed_entry_owner(item) {
            Some(owner) => DirectoryEntry::Owed(owner),
            None => DirectoryEntry::Found(item),
        });
    }
    if let Some(methods) = view.methods
        && let Some(slot) = text_slot(interp, &methods, key)?
        && let Some(stored) = slot_at(interp, methods.items, slot)?
    {
        return run_stored_method(interp, directory, name, stored, &[]).map(DirectoryEntry::Found);
    }
    let Some(stored) = view.unknown else {
        return Ok(DirectoryEntry::Absent);
    };
    let index = interp.text(name);
    interp.roots.push_temp(index);
    run_stored_method(interp, directory, b"UNKNOWN", stored, &[Some(index)])
        .map(DirectoryEntry::Found)
}

/// The slot holding `name` in a string-keyed store, found without building
/// an index object.
fn text_slot(interp: &mut Interp, store: &Store, key: Key<'_>) -> Result<Option<usize>, Failure> {
    let mut slot = (key.hash % store.buckets as u64) as usize;
    while slot < store.total {
        let Some(held) = slot_at(interp, store.indexes, slot)? else {
            return Ok(None);
        };
        if interp.to_text(held).as_ref() == key.name {
            return Ok(Some(slot));
        }
        slot = link_at(interp, store, slot)?;
    }
    Ok(None)
}

/// The phase owing an entry `table` still holds and this crate does not
/// build, or `None` when it holds none.
pub(crate) fn owed_table_owner(interp: &mut Interp, table: ObjRef) -> Option<&'static str> {
    let store = read_store(interp, table, CONTENTS).ok()??;
    let order = walk_in(interp, &store).ok()?;
    order.into_iter().find_map(|slot| {
        let item = slot_at(interp, store.items, slot).ok()??;
        interp.owed_entry_owner(item)
    })
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

/// The index argument a hash-collection method was given, as the bytes it is
/// stored and looked up under.
fn hash_index(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    position: usize,
) -> Result<Vec<u8>, Failure> {
    let Some(Some(argument)) = args.get(position - 1).copied() else {
        return Err(Raised::missing_named_argument("index").into());
    };
    let argument = required_string_named_argument(interp, argument, "index")?;
    Ok(interp.to_text(argument).to_vec())
}

/// `~at(index)` and `~[index]` on a `Directory` or a `StringTable`: the entry
/// stored under `index`, or `.nil` -- `HashCollection::getRexx`, donated to
/// each of them by an `InheritInstanceMethods`.
pub(super) fn native_hash_at(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // **One body, two stores.** `Setup.cpp` donates `IdentityTable`'s `At`,
    // `Put` and `[]` rows to `StringTable` and that whole set on to
    // `Directory` (`memory/Setup.cpp:881`, `:933`), so `Table`,
    // `IdentityTable`, `StringTable` and `Directory` share one method
    // identity here exactly as they share one function upstream. A table
    // this crate built on `NativeObject`'s map reads that map; everything
    // else reads the store.
    if !matches!(
        interp.heap.get(receiver).map(|object| &object.body),
        Some(Body::Native(_))
    ) && owns(interp, receiver)
    {
        return store_at(interp, receiver, args);
    }
    let index = hash_index(interp, args, 1)?;
    Ok(Some(interp.hash_entry_read(receiver, &index)))
}

/// `~put(item, index)` on a `Directory` or a `StringTable`: stores `item`
/// under `index`, replacing whatever was there -- `HashCollection::putRexx`.
pub(super) fn native_hash_put(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // [`native_hash_at`]'s split, for the same reason.
    if !matches!(
        interp.heap.get(receiver).map(|object| &object.body),
        Some(Body::Native(_))
    ) && owns(interp, receiver)
    {
        return store_put(interp, receiver, args);
    }
    let Some(Some(item)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("item").into());
    };
    let index = hash_index(interp, args, 2)?;
    interp.hash_entry_write(receiver, &index, item)?;
    Ok(None)
}

/// `~unknown(message, arguments)` on a `Directory` or a `StringTable`: **the
/// entry-method mechanism**, `StringHashCollection::unknown`
/// (`classes/support/HashCollection.cpp:1015`).
pub(super) fn native_hash_unknown(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(message)) = args.first().copied() else {
        return Err(Raised::missing_method_argument(1).into());
    };
    let message = required_string_argument(interp, message, 1)?;
    let name = interp.to_text(message).to_vec();
    let Some(Some(arguments)) = args.get(1).copied() else {
        return Err(Raised::missing_method_argument(2).into());
    };
    let Some(forwarded) = interp.array_slots_of(arguments) else {
        return Err(unconverted_array_argument(interp, arguments));
    };
    // [`native_hash_at`]'s split again: a store or `NativeObject`'s map.
    // Measured, a `.Directory` given
    // `setEntry('alpha', 42)` answers `d~alpha` as `42`, and `d~beta = 7`
    // stores under `BETA`.
    let store = !matches!(
        interp.heap.get(receiver).map(|object| &object.body),
        Some(Body::Native(_))
    ) && owns(interp, receiver);
    let Some(index) = name.strip_suffix(b"=") else {
        let index = name.to_ascii_uppercase();
        if store {
            let index = interp.text_built(index);
            return store_entry_read(interp, receiver, index);
        }
        return Ok(Some(interp.hash_entry_read(receiver, &index)));
    };
    let index = index.to_ascii_uppercase();
    let Some(Some(item)) = forwarded.first().copied() else {
        return Err(Loud::entry_method_without_a_value(&index).into());
    };
    if store {
        let index = interp.text_built(index);
        return store_entry_write(interp, receiver, index, item);
    }
    interp.hash_entry_write(receiver, &index, item)?;
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
            take_merged(interp, receiver, name, Removal::Discarded)?;
        }
    }
    Ok(None)
}

/// `DirectoryClass::setMethodRexx` (`classes/DirectoryClass.cpp:480`):
/// attaches a method whose RESULT is the entry's item, recomputed on every
/// read.
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
        take_merged(interp, receiver, name, Removal::Answered)?.unwrap_or(ObjRef::NIL),
    ))
}

// ---- `Relation` and `Bag`'s own surface ----

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
/// `ObjectModel::build` ahead of [`relation::NATIVE_METHODS`].
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
];

/// A fresh `Directory` holding one entry per pair, string-keyed --
/// `VariableDictionary::getVariableDirectory`
/// (`execution/VariableDictionary.cpp`), which `RexxContext~variables`
/// answers.
pub(super) fn directory_of(
    interp: &mut Interp,
    entries: Vec<(Vec<u8>, ObjRef)>,
) -> Result<ObjRef, Failure> {
    let class = interp.object_model().directory;
    let directory = new_instance(interp, class)?;
    let frame = interp.roots.push_frame();
    for (name, value) in entries {
        let key = interp.text_built(name);
        interp.roots.push_temp(key);
        insert(interp, directory, key, Some(value))?;
    }
    interp.roots.pop_frame(frame);
    Ok(directory)
}
