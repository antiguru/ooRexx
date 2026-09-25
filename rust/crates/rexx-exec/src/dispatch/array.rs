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

//! Array slot and subscript arithmetic, and `Array`'s primitive methods.

use super::{
    BehaviourId, Body, Cleared, Decoded, Failure, INIT, Interp, Loud, ObjRef, Raised,
    class_receiver, collection, required_string_argument,
};

use super::new_instance;
use super::{Arity, NativeMethod};

// `Array`'s sort family, chained into `ObjectModel::build`.
pub(super) mod sort;

// `Array`'s collection surface, chained into `ObjectModel::build` after its
// sort family.
pub(super) mod surface;

/// An array receiver's own slots, borrowed, or the refusal for a receiver that
/// is not one.
pub(super) fn array_slots(interp: &Interp, receiver: ObjRef) -> Result<&[Option<ObjRef>], Failure> {
    let receiver = collection_store(interp, receiver);
    interp
        .array_slots(receiver)
        .ok_or_else(|| Loud::receiver_class("a value that is not an array").into())
}

/// The array that actually holds `receiver`'s slots: the receiver itself when
/// it carries a `Body::Array`, and otherwise the store its own pool holds.
fn collection_store(interp: &Interp, receiver: ObjRef) -> ObjRef {
    if interp.array_slots(receiver).is_some() {
        return receiver;
    }
    let Some(model) = interp.object_model.as_ref() else {
        return receiver;
    };
    let scope = model.array;
    match interp.heap.get(receiver).map(|object| &object.body) {
        Some(Body::Instance { pools, .. }) => pools.get(scope, b"ITEMS").unwrap_or(receiver),
        _ => receiver,
    }
}

/// [`array_slots`] as an owned copy, for a caller that renders the slots and
/// so needs `interp` back.
pub(super) fn array_slots_owned(
    interp: &Interp,
    receiver: ObjRef,
) -> Result<Vec<Option<ObjRef>>, Failure> {
    Ok(array_slots(interp, receiver)?.to_vec())
}

/// An array receiver's dimensions array as an owned copy, or the refusal
/// [`array_slots`] gives for a receiver that is not an array.
pub(super) fn array_dimensions(
    interp: &Interp,
    receiver: ObjRef,
) -> Result<Option<Vec<usize>>, Failure> {
    let receiver = collection_store(interp, receiver);
    match interp.array_body(receiver) {
        Some((_, dimensions)) => Ok(dimensions.map(<[usize]>::to_vec)),
        None => Err(Loud::receiver_class("a value that is not an array").into()),
    }
}

/// `ArrayClass::isFixedDimension` (`classes/ArrayClass.hpp:223`): an array
/// that can no longer take a shape from a subscript list.
fn array_is_fixed_dimension(interp: &Interp, receiver: ObjRef) -> Result<bool, Failure> {
    let receiver = collection_store(interp, receiver);
    match interp.array_body(receiver) {
        Some((slots, dimensions)) => Ok(dimensions.is_some() || !slots.is_empty()),
        None => Err(Loud::receiver_class("a value that is not an array").into()),
    }
}

/// `ArrayClass::MaxFixedArraySize` (`classes/ArrayClass.hpp:329`).
pub(super) const MAX_FIXED_ARRAY_SIZE: usize = 100_000_000_000_000_000;

/// `size` empty slots, or the 5.0 the oracle raises when the allocator refuses
/// -- [`Raised::system_resources`] carries why that refusal is asked of the
/// allocator rather than of a size limit.
fn empty_slots(size: usize) -> Result<Vec<Option<ObjRef>>, Failure> {
    let mut slots = Vec::new();
    slots
        .try_reserve_exact(size)
        .map_err(|_| Failure::from(Raised::system_resources()))?;
    slots.resize(size, None);
    Ok(slots)
}

/// The bounds policy a subscript list is validated under -- `IndexAccess`
/// and `IndexUpdate` (`classes/ArrayClass.hpp:62`-`:63`).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum IndexUse {
    /// `getRexx`: a subscript past the end answers `.nil`.
    Get,
    /// `putRexx`: a subscript past the end grows the array.
    Put,
}

impl IndexUse {
    /// The argument list position the subscript list starts at, which every
    /// refusal a subscript raises counts from: `putRexx`'s first argument is
    /// the value, so its list starts one later than `getRexx`'s.
    fn arg_position(self) -> usize {
        match self {
            IndexUse::Get => 1,
            IndexUse::Put => 2,
        }
    }
}

/// `ArrayClass::validateIndex` (`classes/ArrayClass.cpp:1211`): the flattened
/// 1-based slot `args` names in `receiver`, or `None` for a subscript out of
/// bounds under [`IndexUse::Get`].
pub(super) fn array_position(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    index_use: IndexUse,
) -> Result<Option<usize>, Failure> {
    let spread;
    let subscripts = match args {
        [Some(only)] => match interp.array_slots(*only) {
            Some(slots) => {
                let items = slots.iter().flatten().count();
                spread = slots[..items].to_vec();
                &spread[..]
            }
            None => args,
        },
        _ => args,
    };
    match array_dimensions(interp, receiver)? {
        Some(dimensions) if dimensions.len() != 1 => {
            multi_dimension_position(interp, receiver, subscripts, index_use, &dimensions)
        }
        _ => single_dimension_position(interp, receiver, subscripts, index_use),
    }
}

/// `ArrayClass::validateSingleDimensionIndex` (`classes/ArrayClass.cpp:1258`).
fn single_dimension_position(
    interp: &mut Interp,
    receiver: ObjRef,
    subscripts: &[Option<ObjRef>],
    index_use: IndexUse,
) -> Result<Option<usize>, Failure> {
    match subscripts {
        [] => Err(Raised::not_enough_method_arguments(index_use.arg_position()).into()),
        [Some(only)] => {
            let position = positive_index(interp, *only, index_use.arg_position())?;
            if position <= array_slots(interp, receiver)?.len() {
                return Ok(Some(position));
            }
            match index_use {
                IndexUse::Get => Ok(None),
                IndexUse::Put if position > MAX_FIXED_ARRAY_SIZE => {
                    Err(Raised::array_too_big(MAX_FIXED_ARRAY_SIZE).into())
                }
                IndexUse::Put => {
                    array_resize(interp, receiver, position)?;
                    Ok(Some(position))
                }
            }
        }
        // Only the spread above can produce this: an argument list of its own
        // drops a trailing omission, so `~at(,)` arrives as no argument at all
        // and is 93.901 above.
        [None] => Err(Loud::array_index_hole().into()),
        _ => {
            if array_is_fixed_dimension(interp, receiver)? {
                return Err(Raised::too_many_subscripts(1).into());
            }
            match index_use {
                IndexUse::Get => Ok(None),
                IndexUse::Put => {
                    let dimensions = array_extend_multi(interp, receiver, subscripts)?;
                    multi_dimension_position(interp, receiver, subscripts, index_use, &dimensions)
                }
            }
        }
    }
}

/// `ArrayClass::validateMultiDimensionIndex` (`classes/ArrayClass.cpp:1361`),
/// whose offset takes the **first** subscript as the fastest-moving one
/// (`:1408`-`:1410`).
fn multi_dimension_position(
    interp: &mut Interp,
    receiver: ObjRef,
    subscripts: &[Option<ObjRef>],
    index_use: IndexUse,
    dimensions: &[usize],
) -> Result<Option<usize>, Failure> {
    if subscripts.len() < dimensions.len() {
        return Err(Raised::not_enough_subscripts(dimensions.len()).into());
    }
    if subscripts.len() > dimensions.len() {
        return Err(Raised::too_many_subscripts(dimensions.len()).into());
    }
    let mut multiplier = 1;
    let mut offset = 0;
    for (at, (subscript, dimension)) in subscripts.iter().zip(dimensions).enumerate() {
        let position = position_index(interp, *subscript, index_use.arg_position() + at + 1)?;
        if position > *dimension {
            return match index_use {
                IndexUse::Get => Ok(None),
                IndexUse::Put => {
                    let grown = array_extend_multi(interp, receiver, subscripts)?;
                    multi_dimension_position(interp, receiver, subscripts, index_use, &grown)
                }
            };
        }
        offset += multiplier * (position - 1);
        multiplier *= dimension;
    }
    Ok(Some(offset + 1))
}

/// Whether `value` is a whole number at `Numerics::ARGUMENT_DIGITS`, which is
/// what `numberArgument` accepts.
pub(super) fn is_whole_method_argument(interp: &mut Interp, value: ObjRef) -> bool {
    interp
        .to_number(value)
        .ok()
        .and_then(|number| number.whole_value(rexx_num::ARGUMENT_DIGITS))
        .is_some()
}

/// A subscript converted under `Numerics::ARGUMENT_DIGITS` rather than under
/// the activation's own `NUMERIC DIGITS`, or `None` for one that is not a
/// whole number of at least 1.
fn whole_index(interp: &mut Interp, value: ObjRef) -> Option<usize> {
    // A tagged integer already is the answer when it is narrow enough that
    // the rounding rule would change nothing, the same shortcut
    // `builtin::whole_number` takes against the same width.
    if let Decoded::SmallInt(small) = value.decode()
        && let Some(whole) = rexx_num::whole_i64(small, rexx_num::ARGUMENT_DIGITS)
        && let Ok(index) = usize::try_from(whole)
        && index > 0
    {
        return Some(index);
    }
    let number = interp.to_number(value).ok()?;
    let index = usize::try_from(number.whole_value(rexx_num::ARGUMENT_DIGITS)?).ok()?;
    (index > 0).then_some(index)
}

/// [`whole_index`]'s conversion admitting zero, for the one index in the
/// language that counts from it: `ListClass::validateIndex`
/// (`classes/ListClass.cpp:195`) reads a list handle with
/// `unsignedNumberValue` under the same `Numerics::ARGUMENT_DIGITS`, and a
/// list's first handle is `0`.
pub(super) fn unsigned_index(interp: &mut Interp, value: ObjRef) -> Option<usize> {
    if let Decoded::SmallInt(small) = value.decode()
        && let Some(whole) = rexx_num::whole_i64(small, rexx_num::ARGUMENT_DIGITS)
    {
        return usize::try_from(whole).ok();
    }
    let number = interp.to_number(value).ok()?;
    usize::try_from(number.whole_value(rexx_num::ARGUMENT_DIGITS)?).ok()
}

/// A comparison result as `numberValue` reads it -- the conversion both sort
/// comparators make on what the Rexx method answered
/// (`classes/ArrayClass.cpp:2907`, `classes/ObjectClass.cpp:243`), at
/// `Numerics::DEFAULT_DIGITS` rather than at the subscript precision.
fn whole_comparison(interp: &mut Interp, value: ObjRef) -> Option<i64> {
    let digits = rexx_num::DEFAULT_DIGITS as usize;
    if let Decoded::SmallInt(small) = value.decode() {
        return rexx_num::whole_i64(small, digits);
    }
    interp.to_number(value).ok()?.whole_value(digits)
}

/// One subscript as `RexxInternalObject::requiredPositive`
/// (`classes/ObjectClass.cpp:1564`) reads it, `position` naming its place in
/// the method's own argument list.
pub(super) fn positive_index(
    interp: &mut Interp,
    value: ObjRef,
    position: usize,
) -> Result<usize, Failure> {
    match whole_index(interp, value) {
        Some(index) => Ok(index),
        None => {
            let found = interp.string_value_text(value);
            Err(Raised::method_argument_not_positive(position, &found).into())
        }
    }
}

/// One subscript of a multidimensional index as `positionArgument`
/// (`classes/StringClassUtil.cpp:209`) reads it -- the same conversion
/// [`positive_index`] makes, under a different pair of errors.
fn position_index(
    interp: &mut Interp,
    subscript: Option<ObjRef>,
    position: usize,
) -> Result<usize, Failure> {
    let Some(value) = subscript else {
        return Err(Raised::missing_method_argument(position).into());
    };
    match whole_index(interp, value) {
        Some(index) => Ok(index),
        None => {
            let found = interp.string_value_text(value);
            Err(Raised::invalid_position(&found).into())
        }
    }
}

/// `ArrayClass::extend` (`classes/ArrayClass.cpp:2034`): grow the receiver to
/// `size` slots, the added ones empty.
fn array_resize(interp: &mut Interp, receiver: ObjRef, size: usize) -> Result<(), Failure> {
    let receiver = collection_store(interp, receiver);
    match interp.heap.get_mut(receiver).map(|object| &mut object.body) {
        Some(Body::Array { slots, .. }) => {
            if let Some(extra) = size.checked_sub(slots.len()) {
                slots
                    .try_reserve_exact(extra)
                    .map_err(|_| Failure::from(Raised::system_resources()))?;
            }
            slots.resize(size, None);
            Ok(())
        }
        _ => Err(Loud::receiver_class("a value that is not an array").into()),
    }
}

/// `ArrayClass::extendMulti` (`classes/ArrayClass.cpp:2434`): grow the
/// receiver so that every subscript is within bounds, answering the shape it
/// now has.
fn array_extend_multi(
    interp: &mut Interp,
    receiver: ObjRef,
    subscripts: &[Option<ObjRef>],
) -> Result<Vec<usize>, Failure> {
    let held = array_dimensions(interp, receiver)?;
    let old = held.filter(|dimensions| dimensions.len() == subscripts.len());
    let mut dimensions = Vec::with_capacity(subscripts.len());
    let mut size = 1usize;
    for (at, subscript) in subscripts.iter().enumerate() {
        let position = position_index(interp, *subscript, at + 1)?;
        let dimension = match &old {
            Some(old) => position.max(old[at]),
            None => position,
        };
        size = size
            .checked_mul(dimension)
            .filter(|size| *size <= MAX_FIXED_ARRAY_SIZE)
            .ok_or_else(|| Failure::from(Raised::array_too_big(MAX_FIXED_ARRAY_SIZE)))?;
        dimensions.push(dimension);
    }
    array_reshape(interp, receiver, &dimensions, size, old.as_deref())?;
    Ok(dimensions)
}

/// The element move `ArrayClass::extendMulti` performs: each filled slot goes
/// to the offset its multidimensional index has under `dimensions`.
fn array_reshape(
    interp: &mut Interp,
    receiver: ObjRef,
    dimensions: &[usize],
    size: usize,
    old: Option<&[usize]>,
) -> Result<(), Failure> {
    let slots = array_slots_owned(interp, receiver)?;
    debug_assert!(
        old.is_some() || slots.iter().all(Option::is_none),
        "a reshape with no source shape must have nothing to move"
    );
    let mut grown = empty_slots(size)?;
    if let Some(old) = old {
        for (position, item) in slots.iter().enumerate() {
            if item.is_none() {
                continue;
            }
            let mut rest = position;
            let mut offset = 0;
            let mut multiplier = 1;
            for (extent, dimension) in old.iter().zip(dimensions) {
                offset += multiplier * (rest % extent);
                rest /= extent;
                multiplier *= dimension;
            }
            grown[offset] = *item;
        }
    }
    match interp.heap.get_mut(receiver).map(|object| &mut object.body) {
        Some(Body::Array {
            slots,
            dimensions: held,
        }) => {
            *slots = grown;
            *held = Some(dimensions.into());
            Ok(())
        }
        _ => Err(Loud::receiver_class("a value that is not an array").into()),
    }
}

/// `Array~at(index)` and `Array~[index]`: the item at `index`, or `.nil` for
/// an empty slot and for a subscript past the end -- `ArrayClass::getRexx`
/// (`classes/ArrayClass.cpp:979`), whose out-of-bounds answer is
/// `TheNilObject` and whose in-bounds answer is `resultOrNil(get(position))`.
pub(super) fn native_array_at(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    native_array_at_for(interp, cleared, receiver, args)
}

/// [`native_array_at`] against a named store, which is what `Queue`'s own row
/// needs: its slots live in an `Array` the instance holds rather than in the
/// receiver.
pub(super) fn native_array_at_for(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // The subscript first, so the slots are borrowed rather than copied: the
    // conversion needs `&mut interp` and the read does not, and reading one
    // slot must not cost a copy of the whole array.
    let Some(index) = array_position(interp, receiver, args, IndexUse::Get)? else {
        return Ok(Some(ObjRef::NIL));
    };
    Ok(Some(match array_slots(interp, receiver)?.get(index - 1) {
        Some(Some(item)) => *item,
        Some(None) | None => ObjRef::NIL,
    }))
}

/// `Array~put(value, index...)` and `Array~[index...] = value`:
/// `ArrayClass::putRexx` (`classes/ArrayClass.cpp:590`), which requires the
/// value, validates the rest as the subscript list under `IndexUpdate` and
/// answers nothing.
pub(super) fn native_array_put(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if args.len() < 2 {
        return Err(Raised::not_enough_method_arguments(2).into());
    }
    let Some(value) = args[0] else {
        return Err(Raised::missing_method_argument(1).into());
    };
    let position = array_position(interp, receiver, &args[1..], IndexUse::Put)?
        .expect("IndexUse::Put grows the array rather than answering out of bounds");
    let receiver = collection_store(interp, receiver);
    match interp.heap.get_mut(receiver).map(|object| &mut object.body) {
        Some(Body::Array { slots, .. }) => {
            slots[position - 1] = Some(value);
            Ok(None)
        }
        _ => Err(Loud::receiver_class("a value that is not an array").into()),
    }
}

/// `Array~dimension([n])`: how many dimensions the array has, or the extent
/// of dimension `n` -- `ArrayClass::dimensionRexx`
/// (`classes/ArrayClass.cpp:1103`), which answers `0` for a dimension the
/// array does not have.
pub(super) fn native_array_dimension(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let size = array_slots(interp, receiver)?.len();
    let dimensions = array_dimensions(interp, receiver)?;
    let answer = match args.first().copied().flatten() {
        None => match &dimensions {
            Some(dimensions) => dimensions.len(),
            None if size == 0 => 0,
            None => 1,
        },
        Some(target) => {
            let position = positive_index(interp, target, 1)?;
            match &dimensions {
                Some(dimensions) if dimensions.len() != 1 => {
                    dimensions.get(position - 1).copied().unwrap_or(0)
                }
                _ if position == 1 => size,
                _ => 0,
            }
        }
    };
    Ok(Some(interp.counted(answer)))
}

/// `.Array~new([size])` and `.Array~new(dimension...)`:
/// `ArrayClass::newRexx` (`classes/ArrayClass.cpp:89`).
pub(super) fn native_array_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let spread;
    let body = match args {
        [] => Body::array(Vec::new()),
        [only] => match only.and_then(|value| interp.array_slots_of(value)) {
            Some(slots) => {
                let items = slots.iter().flatten().count();
                spread = slots[..items].to_vec();
                multidimensional_body(interp, &spread)?
            }
            None => {
                let size = array_size_argument(interp, *only, 1)?;
                Body::Array {
                    slots: empty_slots(size)?,
                    // `newRexx`'s own `if (totalSize == 0)` (`:125`-`:128`),
                    // whose one entry nothing reads: an explicit zero size
                    // fixes the shape, and the entry is not the extent.
                    dimensions: (size == 0).then(|| Box::from([0].as_slice())),
                }
            }
        },
        _ => multidimensional_body(interp, args)?,
    };
    let object = array_of_class(interp, class, body)?;
    let caller = interp.caller();
    interp.send_message(object, INIT, None, &[], caller)?;
    Ok(Some(object))
}

/// `body` as an object of `class`: a bare `Body::Array` for `.Array` itself,
/// and an instance carrying it as a store for any subclass.
fn array_of_class(interp: &mut Interp, class: ObjRef, body: Body) -> Result<ObjRef, Failure> {
    let store = interp.alloc_with(BehaviourId::ARRAY, body);
    if class == interp.object_model().array {
        interp.roots.push_temp(store);
        return Ok(store);
    }
    collection::instance_over_store(interp, class, store)
}

/// `.Array~of(item, ...)`: the arguments as an array's slots, in order --
/// `ArrayClass::ofRexx` (`classes/ArrayClass.cpp:150`), whose `INIT` send
/// carries no arguments.
pub(super) fn native_array_of(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let body = Body::Array {
        slots: args.to_vec(),
        dimensions: args.is_empty().then(|| Box::from([0].as_slice())),
    };
    let object = array_of_class(interp, class, body)?;
    let caller = interp.caller();
    interp.send_message(object, INIT, None, &[], caller)?;
    Ok(Some(object))
}

/// `ArrayClass::createMultidimensional` (`classes/ArrayClass.cpp:199`): a
/// body whose shape is `dimensions` and whose slots are all empty.
fn multidimensional_body(
    interp: &mut Interp,
    dimensions: &[Option<ObjRef>],
) -> Result<Body, Failure> {
    let mut shape = Vec::with_capacity(dimensions.len());
    let mut size = 1usize;
    for (at, dimension) in dimensions.iter().enumerate() {
        let extent = array_size_argument(interp, *dimension, at + 1)?;
        size = size
            .checked_mul(extent)
            .filter(|size| *size <= MAX_FIXED_ARRAY_SIZE)
            .ok_or_else(|| Failure::from(Raised::array_too_big(MAX_FIXED_ARRAY_SIZE)))?;
        shape.push(extent);
    }
    Ok(Body::Array {
        slots: empty_slots(size)?,
        dimensions: Some(shape.into()),
    })
}

/// `ArrayClass::validateSize` (`classes/ArrayClass.cpp:178`) over
/// `nonNegativeArgument` (`classes/StringClassUtil.cpp:167`): a whole number
/// of at least zero, `MaxFixedArraySize` its upper bound.
pub(super) fn array_size_argument(
    interp: &mut Interp,
    argument: Option<ObjRef>,
    position: usize,
) -> Result<usize, Failure> {
    let Some(value) = argument else {
        return Err(Raised::missing_method_argument(position).into());
    };
    let size = interp
        .to_number(value)
        .ok()
        .and_then(|number| number.whole_value(rexx_num::ARGUMENT_DIGITS))
        .and_then(|whole| usize::try_from(whole).ok());
    match size {
        Some(size) if size <= MAX_FIXED_ARRAY_SIZE => Ok(size),
        Some(_) => Err(Raised::array_too_big(MAX_FIXED_ARRAY_SIZE).into()),
        None => {
            let found = interp.string_value_text(value);
            Err(Raised::argument_not_non_negative(position, &found).into())
        }
    }
}

/// `Array~size`: how many slots the array has, empty ones included --
/// `ArrayClass::sizeRexx`.
pub(super) fn native_array_size(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let size = array_slots(interp, receiver)?.len();
    Ok(Some(interp.counted(size)))
}

/// `Array~items`: how many slots hold an object -- `ArrayClass::itemsRexx`.
pub(super) fn native_array_items(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let items = array_slots(interp, receiver)?.iter().flatten().count();
    Ok(Some(interp.counted(items)))
}

/// `RexxObject::requestArray` (`runtime/MethodArguments.hpp:683`): the value
/// itself when it already is an array, and its `MAKEARRAY` otherwise.
pub(super) fn request_array(interp: &mut Interp, value: ObjRef) -> Result<ObjRef, Failure> {
    if interp.array_slots_of(value).is_some() {
        return Ok(value);
    }
    if interp.lookup(value, b"MAKEARRAY", None).is_none() {
        return Ok(value);
    }
    let caller = interp.caller();
    let sent = interp.send_message(value, b"MAKEARRAY", None, &[], caller)?;
    Ok(sent.unwrap_or(value))
}

/// The refusal for an `~UNKNOWN` argument list that is not already an `Array`.
pub(super) fn unconverted_array_argument(interp: &mut Interp, value: ObjRef) -> Failure {
    match interp.receiver_class_id(value) {
        Some(id) => Loud::native_method(b"MAKEARRAY", &id).into(),
        None => Loud::receiver_class("a value this phase builds no class for").into(),
    }
}

/// `Array~makeString(format, separator)` and `Array~toString(format,
/// separator)`: the array's items as one string -- `ArrayClass::toString`
/// (`classes/ArrayClass.cpp:1856`), which `MakeString` and `ToString` both
/// name at the same arity (`memory/Setup.cpp:733`-`:734`), which is why one
/// function answers both rows.
pub(super) fn native_array_make_string(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // `optionalOptionArgument(format, 'L', ARG_ONE)`: the option's first
    // character, upcased, with the whole argument omitted meaning `L`.
    let form = match args.first().copied().flatten() {
        None => b'L',
        Some(argument) => {
            let argument = required_string_argument(interp, argument, 1)?;
            let text = interp.to_text(argument).to_vec();
            match text.first().copied().map(|byte| byte.to_ascii_uppercase()) {
                Some(byte @ (b'L' | b'C')) => byte,
                _ => return Err(Raised::method_option_not_recognised("CL", &text).into()),
            }
        }
    };
    let separator = args.get(1).copied().flatten();
    if form == b'C' && separator.is_some() {
        return Err(Raised::too_many_method_arguments(1).into());
    }
    let separator = match separator {
        None if form == b'L' => b"\n".to_vec(),
        None => Vec::new(),
        Some(argument) => {
            let argument = required_string_argument(interp, argument, 2)?;
            interp.to_text(argument).to_vec()
        }
    };
    let slots = array_slots_owned(interp, receiver)?;
    // The join itself is `Interp::array_string` and not a second loop here:
    // `makeString` with no arguments is what a string context asks an array
    // for, so the two must not be able to disagree about an empty slot or
    // about a nested array's rendering.
    let out = interp.array_string(&slots, &separator);
    Ok(Some(interp.text_built(out)))
}
