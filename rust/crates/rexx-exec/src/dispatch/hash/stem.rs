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

//! `Stem`'s primitive methods, and the tail tree that orders a stem's tails the
//! way `CompoundTableElement`'s does.

use super::{
    Arity, ArrayArgument, Body, Cleared, Failure, Interp, NativeMethod, ObjRef, Raised,
    array_argument, insert, new_instance, refusal_outside, required_string_argument,
};

// ---- `Stem` ----

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
fn stem_read(interp: &mut Interp, receiver: ObjRef, tail: &[u8]) -> ObjRef {
    let (held, default, name) = match interp.heap.get(receiver).map(|object| &object.body) {
        Some(Body::Stem {
            tails,
            default,
            name,
            exposed,
        }) => (
            match crate::stem::exposed_home(exposed, tail) {
                // `findCompoundVariable` answers the exposed tail's
                // `realVariable`, as the variable read does.
                Some(home) => Some(interp.tail_value(home, tail)),
                None => tails.get(tail).map(|(_, value)| *value),
            },
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
    let receiver = stem_of(interp, receiver, b"[]")?;
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
    let receiver = stem_of(interp, receiver, b"[]=")?;
    let value = super::collection::item_argument(args)?;
    // No tail: `StemClass::bracketEqual` (`classes/StemClass.cpp:476`) makes
    // `value` the default and clears every tail.
    if args.len() == 1 {
        if is_stem(interp, value) {
            return Err(Raised::syntax(98, 976, Vec::new()).into());
        }
        interp.detach_exposed_tails(receiver);
        if let Some(Body::Stem {
            default,
            tails,
            exposed,
            ..
        }) = interp.heap.get_mut(receiver).map(|held| &mut held.body)
        {
            *default = Some(value);
            *tails = rexx_core::NameMap::default();
            *exposed = None;
        }
        return Ok(None);
    }
    let tail = stem_tail(interp, args.get(1..).unwrap_or_default())?;
    stem_write(interp, receiver, tail, Some(value));
    Ok(None)
}

/// The stem an `EXPOSE` made `tail` of `receiver` belong to, if one did.
fn exposed_tail(interp: &Interp, receiver: ObjRef, tail: &[u8]) -> Option<ObjRef> {
    match interp.heap.get(receiver).map(|object| &object.body) {
        Some(Body::Stem { exposed, .. }) => crate::stem::exposed_home(exposed, tail),
        _ => None,
    }
}

/// Whether `receiver` is a stem, which is what every row below needs and what
/// the shared bodies must not be given.
fn is_stem(interp: &Interp, receiver: ObjRef) -> bool {
    matches!(
        interp.heap.get(receiver).map(|object| &object.body),
        Some(Body::Stem { .. })
    )
}

/// The scope-pool entry, under `Stem`'s scope, that holds a `Stem` subclass
/// instance's `Body::Stem`.
pub(in crate::dispatch) const STEM_STORE: &[u8] = b"STEM";

impl Interp {
    /// The `Body::Stem` a `Stem` subclass instance holds, or `None` for any
    /// other value.
    pub(crate) fn stem_subclass_store(&self, value: ObjRef) -> Option<ObjRef> {
        match &self.heap.get(value)?.body {
            Body::Instance { pools, .. } => self.stem_store_in(pools),
            _ => None,
        }
    }

    /// [`Interp::stem_subclass_store`] over an instance's pools: the one
    /// entry the `Stem` scope's pool holds, found by the scope alone, which
    /// keeps the test free of a name comparison on the string-value path.
    pub(crate) fn stem_store_in(&self, pools: &rexx_core::ScopePools) -> Option<ObjRef> {
        let scope = self.object_model.as_ref()?.stem;
        pools.entries(scope).first().map(|(_, store)| *store)
    }

    /// `StemClass::dropElement`: drops `tail` where the stem holds it, and
    /// leaves a tail it never held absent.
    pub(crate) fn stem_drop_element(&mut self, stem: ObjRef, tail: &[u8]) {
        let held = exposed_tail(self, stem, tail).is_some()
            || matches!(
                self.heap.get(stem).map(|object| &object.body),
                Some(Body::Stem { tails, .. }) if tails.get(tail).is_some()
            );
        if held {
            stem_write(self, stem, tail.to_vec(), None);
        }
    }

    /// `StemClass::toDirectory`'s pairs: every tail that holds a value, in
    /// the order the stem answers its tails.
    pub(crate) fn stem_live_tails(&self, stem: ObjRef) -> Vec<(Vec<u8>, ObjRef)> {
        stem_live(self, stem)
    }

    /// `StemClass::getStemValue`: the stem's default, else its own name.
    pub(crate) fn stem_own_value(&mut self, stem: ObjRef) -> ObjRef {
        stem_value(self, stem)
    }
}

/// The `Body::Stem` a `Stem` method works on: the receiver itself, or the
/// store a subclass instance holds; anything else is refused.
fn stem_of(interp: &mut Interp, receiver: ObjRef, method: &[u8]) -> Result<ObjRef, Failure> {
    if is_stem(interp, receiver) {
        return Ok(receiver);
    }
    match interp.stem_subclass_store(receiver) {
        Some(store) => Ok(store),
        None => Err(stem_refusal(interp, receiver, method)),
    }
}

fn stem_refusal(interp: &mut Interp, receiver: ObjRef, method: &[u8]) -> Failure {
    refusal_outside(
        interp,
        receiver,
        method,
        &["Stem"],
        "a value that is not a stem",
    )
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
    let receiver = stem_of(interp, receiver, b"ALLINDEXES")?;
    // **Rooted as they are built**, for `ordered_pairs`'s reason: each name is
    // a fresh allocation and the next one collects, so an unrooted earlier
    // name is swept before the array holds it. Measured -- without these,
    // `collect_stress` panics at `not_in_arena`'s "a live value".
    let mut names = Vec::new();
    for (name, _) in stem_live(interp, receiver) {
        let name = interp.text_built(name);
        interp.roots.activity_mut().push_temp(name);
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
    let receiver = stem_of(interp, receiver, b"ALLITEMS")?;
    let items: Vec<ObjRef> = stem_live(interp, receiver)
        .into_iter()
        .map(|(_, value)| value)
        .collect();
    // The items are the stem's own and are reachable through it, but the
    // array below allocates, so they are held for that.
    for item in &items {
        interp.roots.activity_mut().push_temp(*item);
    }
    Ok(Some(super::collection::array_of(interp, items)))
}

/// `makeArray` answers the INDEXES here as it does for every other class in
/// this phase -- measured, `A,C` for a stem holding `A` and `C`, not `1,3`.
/// The stem's own value, which is what `StemClass`'s forwarding methods send
/// to -- the `value` field every one of them names.
fn stem_value(interp: &mut Interp, receiver: ObjRef) -> ObjRef {
    let (default, name) = match interp.heap.get(receiver).map(|object| &object.body) {
        Some(Body::Stem { default, name, .. }) => (*default, name.as_ref().to_vec()),
        _ => return ObjRef::NIL,
    };
    default.unwrap_or_else(|| interp.text_built(name))
}

/// Sends `message` with `args` to the stem's value on its behalf, which is
/// what `StemClass`'s forwarding methods do ([`Interp::forwarded`]).
fn forward_to_value(
    interp: &mut Interp,
    receiver: ObjRef,
    message: &[u8],
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let value = stem_value(interp, receiver);
    interp.roots.activity_mut().push_temp(value);
    let caller = interp.caller();
    let probe = 0u8;
    interp.forwarded(&raw const probe, receiver, |interp| {
        interp.send_message(value, message, None, args, caller)
    })
}

/// `StemClass::request(class)` (`classes/StemClass.cpp`): `'ARRAY'` answers
/// the stem's own `makeArray` -- which for a `Stem` is its TAILS -- and every
/// other class is forwarded to the stem's value.
fn native_stem_request(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let store = stem_of(interp, receiver, b"REQUEST")?;
    let Some(wanted) = args.first().copied().flatten() else {
        return Err(Raised::missing_method_argument(1).into());
    };
    let wanted = required_string_argument(interp, wanted, 1)?;
    let upper = interp.to_text(wanted).to_ascii_uppercase();
    if upper == b"ARRAY" {
        if interp.is_base_class(receiver) {
            return native_stem_make_array(interp, cleared, receiver, &[]);
        }
        let caller = interp.caller();
        return interp.send_message(receiver, b"MAKEARRAY", None, &[], caller);
    }
    let forwarded = interp.text_built(upper);
    interp.roots.activity_mut().push_temp(forwarded);
    forward_to_value(interp, store, b"REQUEST", &[Some(forwarded)])
}

/// `StemClass::toDirectory`: a fresh `Directory` holding one entry per tail
/// that HAS a value, keyed by the tail's name, added in the tail tree's own
/// `first`/`next` order.
fn native_stem_to_directory(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let receiver = stem_of(interp, receiver, b"TODIRECTORY")?;
    let class = interp
        .classes()
        .lookup("Directory")
        .expect("Directory is a native class");
    let object = new_instance(interp, class)?;
    interp.roots.activity_mut().push_temp(object);
    for (name, value) in stem_live(interp, receiver) {
        let index = interp.text_built(name);
        interp.roots.activity_mut().push_temp(index);
        insert(interp, object, index, Some(value))?;
    }
    Ok(Some(object))
}

/// `StemClass::unknownRexx(message, arguments)`: forwards the message and its
/// argument array to the stem's value.
fn native_stem_unknown(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    stem_unknown(interp, receiver, args)
}

/// [`native_stem_unknown`]'s work, which `processUnknown` also runs without a send.
pub(in crate::dispatch) fn stem_unknown(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let receiver = stem_of(interp, receiver, b"UNKNOWN")?;
    let Some(message) = args.first().copied().flatten() else {
        return Err(Raised::missing_method_argument(1).into());
    };
    let message = required_string_argument(interp, message, 1)?;
    let name = interp.to_text(message).to_vec();
    let Some(arguments) = args.get(1).copied().flatten() else {
        return Err(Raised::missing_method_argument(2).into());
    };
    let forwarded = array_argument(interp, arguments, ArrayArgument::Positional)?;
    forward_to_value(interp, receiver, &name, &forwarded)
}

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
    let receiver = stem_of(interp, receiver, b"SUPPLIER")?;
    let live = stem_live(interp, receiver);
    let items: Vec<ObjRef> = live.iter().map(|(_, value)| *value).collect();
    for item in &items {
        interp.roots.activity_mut().push_temp(*item);
    }
    // Rooted as they are built -- see [`native_stem_all_indexes`].
    let mut indexes = Vec::new();
    for (name, _) in live {
        let name = interp.text_built(name);
        interp.roots.activity_mut().push_temp(name);
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
    let receiver = stem_of(interp, receiver, b"ITEMS")?;
    let count = stem_live(interp, receiver).len();
    Ok(Some(interp.counted(count)))
}

fn native_stem_is_empty(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let receiver = stem_of(interp, receiver, b"ISEMPTY")?;
    let empty = stem_live(interp, receiver).is_empty();
    Ok(Some(crate::eval::logical(empty)))
}

/// The tail a subscript list names: the arguments' string values joined with
/// `.`, **exactly as written**.
fn stem_tail(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<Vec<u8>, Failure> {
    let mut tail = Vec::new();
    for (at, argument) in args.iter().enumerate() {
        if at > 0 {
            tail.push(b'.');
        }
        if let Some(argument) = *argument {
            let value = interp.required_string_value(argument)?;
            interp.roots.activity_mut().push_temp(value);
            tail.extend_from_slice(&interp.to_text(value));
        }
    }
    Ok(tail)
}

fn native_stem_has_index(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let receiver = stem_of(interp, receiver, b"HASINDEX")?;
    // **No subscripts is true.** Measured, `obj~hasIndex()` on a stem holding
    // one tail answers `1` -- the stem itself is the index a bare `hasIndex`
    // asks about, and none of `Stem`'s rows treats a missing subscript as an
    // error the way the hash classes do.
    if args.is_empty() {
        return Ok(Some(crate::eval::logical(true)));
    }
    let tail = stem_tail(interp, args)?;
    let held = match exposed_tail(interp, receiver, &tail) {
        Some(home) => interp.tail_value(home, &tail).is_some(),
        None => stem_live(interp, receiver)
            .into_iter()
            .any(|(name, _)| name == tail),
    };
    Ok(Some(crate::eval::logical(held)))
}

fn native_stem_has_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let receiver = stem_of(interp, receiver, b"HASITEM")?;
    // Optional, unlike every other class's, where a missing item is 93.903.
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
    let receiver = stem_of(interp, receiver, b"INDEX")?;
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
    if let Some(home) = exposed_tail(interp, receiver, &tail) {
        interp.set_tail(home, &tail, value);
        return;
    }
    if let Some(Body::Stem { tails, .. }) = interp.heap.get_mut(receiver).map(|held| &mut held.body)
    {
        // The ordinal survives a drop and an overwrite -- see `Body::Stem`.
        let next = tails.len();
        let ordinal = tails.get(&tail).map_or(next, |(at, _)| *at);
        tails.insert(tail, (ordinal, value));
    }
}

/// `remove(tail)`: drops the tail and answers what it held, or `.nil`.
fn native_stem_remove(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let receiver = stem_of(interp, receiver, b"REMOVE")?;
    // No subscripts names the stem's value: `StemClass::remove`
    // (`classes/StemClass.cpp:346`) answers it and makes the stem's name its
    // value, which an unset tail of a stem that had a value then reads as.
    // `obj~remove('ZZ')` answers `.nil`.
    if args.is_empty() {
        let value = stem_value(interp, receiver);
        let name = match interp.heap.get(receiver).map(|object| &object.body) {
            Some(Body::Stem {
                name,
                default: Some(_),
                ..
            }) => name.to_vec(),
            _ => return Ok(Some(value)),
        };
        interp.roots.activity_mut().push_temp(value);
        let name = interp.text_built(name);
        interp.roots.activity_mut().push_temp(name);
        if let Some(Body::Stem { default, .. }) =
            interp.heap.get_mut(receiver).map(|held| &mut held.body)
        {
            *default = Some(name);
        }
        return Ok(Some(value));
    }
    let tail = stem_tail(interp, args)?;
    let held = match exposed_tail(interp, receiver, &tail) {
        Some(home) => interp.tail_value(home, &tail),
        None => stem_live(interp, receiver)
            .into_iter()
            .find(|(name, _)| *name == tail)
            .map(|(_, value)| value),
    };
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
    let receiver = stem_of(interp, receiver, b"REMOVEITEM")?;
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
    let store = stem_of(interp, receiver, b"EMPTY")?;
    // **Deleted, not dropped.** A tombstone reads as the tail's own derived
    // name; the oracle's `empty` leaves the tails never-assigned, so they
    // read as the stem's default. Measured: `u. = 5` with one tail set
    // answers `items` 0 after `empty` and `u~at('K')` still answers `5`,
    // where writing tombstones answered `U.K`.
    interp.detach_exposed_tails(store);
    let default = match interp.heap.get(store).map(|object| &object.body) {
        Some(Body::Stem { default, .. }) => *default,
        _ => None,
    };
    if let Some(Body::Stem { tails, exposed, .. }) =
        interp.heap.get_mut(store).map(|held| &mut held.body)
    {
        *tails = rexx_core::NameMap::default();
        *exposed = None;
    }
    let _ = default;
    // Answers the receiver, as `Array~empty` and `List~empty` do -- measured,
    // `(obj~empty == obj)` is `1`.
    Ok(Some(receiver))
}

/// `Stem`'s primitive methods, chained into `ObjectModel::build` beside
/// [`super::NATIVE_METHODS`].
pub(in crate::dispatch) const NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
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
    ("Stem", "REQUEST", Arity::Fixed(1), native_stem_request),
    (
        "Stem",
        "TODIRECTORY",
        Arity::Fixed(0),
        native_stem_to_directory,
    ),
    ("Stem", "UNKNOWN", Arity::Fixed(2), native_stem_unknown),
];
