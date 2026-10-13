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

//! The `REXXUTIL` routines over stem arrays and the caller's variables:
//! `interpreter/runtime/RexxUtilCommon.cpp` and `StemClass::sort`.

use std::cmp::Ordering;

use rexx_core::{Decoded, ObjRef};
use rexx_num::{DIGITS64, Number};

use super::{answer, c_string, check_arity, marshalled, optional_text, os_path, required};
use crate::error::Raised;
use crate::{Failure, Interp};

/// `Numerics::MAX_WHOLENUMBER`, the top of a native whole-number argument.
const MAX_WHOLENUMBER: i64 = 999_999_999_999_999_999;

/// `Numerics::ARGUMENT_DIGITS`, the precision `WholeNumber` converts under.
const ARGUMENT_DIGITS: usize = 18;

/// The largest count `StemClass::sort` reads from element 0: an unsigned
/// value at the default nine digits (`Numerics::maxValueForDigits(9)`).
const SORT_COUNT_MAXIMUM: u64 = 999_999_999;

/// A `RexxStemObject` argument: a stem itself, or a string naming a stem
/// variable of the caller's.
fn stem_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    position: usize,
) -> Result<ObjRef, Failure> {
    let value = required(args, position)?;
    if interp.is_stem_object(value) {
        return Ok(value);
    }
    let text = interp.required_string_or_nil(value)?;
    if text != ObjRef::NIL {
        let name = interp.to_text(text).into_owned();
        if let Some(stem) = interp.context_stem_named(&name) {
            interp.roots.activity_mut().push_temp(stem);
            return Ok(stem);
        }
    }
    let found = interp.string_value_text(value);
    Err(marshalled(Raised::native_argument_not_a_stem(
        false, position, &found,
    )))
}

/// A whole number from `minimum` to [`MAX_WHOLENUMBER`], as
/// `objectToSignedInteger` converts a native argument.
fn whole_value(interp: &mut Interp, value: ObjRef, minimum: i64) -> Result<Option<i64>, Failure> {
    let number = match value.decode() {
        Decoded::SmallInt(number) => Some(number),
        _ => {
            let text = interp.required_string_or_nil(value)?;
            Number::parse_bytes(&interp.to_text(text))
                .and_then(|number| number.int64_value(DIGITS64))
        }
    };
    Ok(number.filter(|number| (minimum..=MAX_WHOLENUMBER).contains(number)))
}

/// An `OPTIONAL_positive_wholenumber_t` argument: 88.905 where it is not
/// a whole number from one up.
fn optional_positive(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    position: usize,
) -> Result<Option<i64>, Failure> {
    let Some(value) = args.get(position - 1).copied().flatten() else {
        return Ok(None);
    };
    match whole_value(interp, value, 1)? {
        Some(number) => Ok(Some(number)),
        None => {
            let found = interp.string_value_text(value);
            Err(marshalled(Raised::native_argument_not_positive(
                position, &found,
            )))
        }
    }
}

/// [`optional_positive`] for an argument that is required.
pub(super) fn positive(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    position: usize,
) -> Result<i64, Failure> {
    required(args, position)?;
    Ok(optional_positive(interp, args, position)?.expect("the argument is present"))
}

/// An `OPTIONAL_nonnegative_wholenumber_t` argument: 88.904 where it is not
/// a whole number from zero up.
fn optional_nonnegative(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    position: usize,
) -> Result<Option<i64>, Failure> {
    let Some(value) = args.get(position - 1).copied().flatten() else {
        return Ok(None);
    };
    match whole_value(interp, value, 0)? {
        Some(number) => Ok(Some(number)),
        None => {
            let found = interp.string_value_text(value);
            Err(marshalled(Raised::native_argument_not_nonnegative(
                position, &found,
            )))
        }
    }
}

/// The tail `CompoundVariableTail(index)` builds: the number in decimal.
fn index_tail(index: i64) -> Vec<u8> {
    index.to_string().into_bytes()
}

/// `GetStemArrayElement`: the element's own value, `None` where it is unset
/// or dropped. The stem's default never answers.
fn element(interp: &Interp, stem: ObjRef, index: i64) -> Option<ObjRef> {
    interp.stem_object_tail(stem, &index_tail(index))
}

/// `SetStemArrayElement`.
fn set_element(interp: &mut Interp, stem: ObjRef, index: i64, value: ObjRef) {
    interp.stem_object_write(stem, &index_tail(index), value);
}

/// `SetStemArrayElement` of a whole number.
fn set_count(interp: &mut Interp, stem: ObjRef, index: i64, count: i64) {
    let value = answer(interp, count);
    set_element(interp, stem, index, value);
}

/// `WholeNumber` over element 0: `None` where it is unset, `Some(None)` where
/// it is not a whole number.
fn element_count(interp: &mut Interp, stem: ObjRef) -> Result<Option<Option<i64>>, Failure> {
    let Some(value) = element(interp, stem, 0) else {
        return Ok(None);
    };
    let number = match value.decode() {
        Decoded::SmallInt(number) => Some(number),
        _ => {
            let text = interp.required_string_or_nil(value)?;
            Number::parse_bytes(&interp.to_text(text))
                .and_then(|number| number.int64_value(ARGUMENT_DIGITS))
        }
    };
    Ok(Some(number.filter(|number| {
        (-MAX_WHOLENUMBER..=MAX_WHOLENUMBER).contains(number)
    })))
}

/// Element 0 as a count, 40.922 where it is unset or not a whole number.
fn required_count(interp: &mut Interp, stem: ObjRef) -> Result<i64, Failure> {
    element_count(interp, stem)?
        .flatten()
        .ok_or_else(|| Raised::stem_count_invalid().into())
}

/// `SysStemDelete(stem, start [, count])`: removes `count` elements from
/// `start`, moving the rest down and dropping the vacated tails.
pub(crate) fn delete(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 3)?;
    let stem = stem_argument(interp, args, 1)?;
    let start = positive(interp, args, 2)?;
    let count = optional_positive(interp, args, 3)?.unwrap_or(1);
    let items = match element_count(interp, stem)?.flatten() {
        Some(items) if items >= 0 => items,
        _ => return Err(Raised::stem_count_invalid().into()),
    };
    if start + count - 1 > items {
        return Err(Raised::stem_range(items).into());
    }
    let mut index = start;
    while index + count <= items {
        let Some(value) = element(interp, stem, index + count) else {
            return Err(Raised::stem_element_unset(index).into());
        };
        set_element(interp, stem, index, value);
        index += 1;
    }
    for index in items - count + 1..=items {
        interp.stem_drop_element(stem, &index_tail(index));
    }
    set_count(interp, stem, 0, items - count);
    Ok(answer(interp, 0))
}

/// `SysStemInsert(stem, position, value)`: moves the elements from
/// `position` up by one and stores `value` there.
pub(crate) fn insert(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 3)?;
    let stem = stem_argument(interp, args, 1)?;
    let position = positive(interp, args, 2)?;
    let value = required(args, 3)?;
    let count = required_count(interp, stem)?;
    if position > count + 1 {
        return Err(Raised::stem_range(count).into());
    }
    let mut index = count;
    while index >= position {
        let Some(moved) = element(interp, stem, index) else {
            return Err(Raised::stem_element_unset(index).into());
        };
        set_element(interp, stem, index + 1, moved);
        index -= 1;
    }
    set_element(interp, stem, position, value);
    set_count(interp, stem, 0, count + 1);
    Ok(answer(interp, 0))
}

/// `SysStemCopy(from, to [, start [, at [, count [, mode]]]])`: copies
/// `count` elements of `from` from `start` over `to` from `at`, or inserts
/// them there under mode `I`.
///
/// A missing source element is 40.924 naming its offset into the copy, not
/// its index, as the oracle passes its loop counter.
pub(crate) fn copy(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 6)?;
    let from = stem_argument(interp, args, 1)?;
    let to = stem_argument(interp, args, 2)?;
    let start = optional_positive(interp, args, 3)?;
    let at = optional_positive(interp, args, 4)?;
    let given = optional_nonnegative(interp, args, 5)?;
    let mode = optional_text(interp, args, 6)?;
    let inserting = match mode.as_deref() {
        None => false,
        Some(mode) => match mode.first().map(u8::to_ascii_uppercase) {
            Some(b'I') => true,
            Some(b'O') => false,
            _ => {
                return Err(Raised::invalid_routine_option(
                    "SysStemCopy",
                    "sort type",
                    "'I' or 'O'",
                    mode,
                )
                .into());
            }
        },
    };
    let from_count = required_count(interp, from)?;
    let start = start.unwrap_or(1);
    let at = at.unwrap_or(1);
    let count = match given {
        Some(count) if count > from_count - start + 1 || from_count == 0 => {
            return Err(Raised::stem_range(from_count).into());
        }
        Some(count) => count,
        None => from_count - start + 1,
    };
    let mut to_count = match element_count(interp, to)? {
        None => 0,
        Some(Some(count)) => count,
        Some(None) => return Err(Raised::stem_count_invalid().into()),
    };
    if at > to_count + 1 {
        return Err(Raised::stem_range(to_count).into());
    }
    if inserting {
        let mut index = to_count;
        while index >= at {
            let Some(moved) = element(interp, to, index) else {
                return Err(Raised::stem_element_unset(index).into());
            };
            set_element(interp, to, index + count, moved);
            index -= 1;
        }
        to_count += count;
        set_count(interp, to, 0, to_count);
    }
    for offset in 0..count {
        let Some(value) = element(interp, from, start + offset) else {
            return Err(Raised::stem_element_unset(offset).into());
        };
        set_element(interp, to, at + offset, value);
    }
    if at + count - 1 > to_count {
        set_count(interp, to, 0, at + count - 1);
    }
    Ok(answer(interp, 0))
}

/// How `SysStemSort` orders two values.
#[derive(Clone, Copy)]
struct SortOrder {
    descending: bool,
    caseless: bool,
    /// The zero-based first column and the column count, where a column
    /// range was given.
    columns: Option<(usize, usize)>,
}

impl SortOrder {
    /// The comparator `StemClass::sort` picks, as `-1`, `0` or `1`.
    fn compare(self, left: &[u8], right: &[u8]) -> i32 {
        let ordering = match self.columns {
            None => compare_whole(left, right, self.caseless),
            Some((start, length)) => compare_columns(left, right, start, length, self.caseless),
        };
        let sign = match ordering {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        };
        if self.descending { -sign } else { sign }
    }
}

/// One byte as `StringUtil::caselessCompare` reads it: upcased, then taken as
/// a signed `char`, so a byte from `0x80` up orders below every ASCII byte.
fn caseless_byte(byte: u8) -> i8 {
    i8::from_ne_bytes([byte.to_ascii_uppercase()])
}

/// Compares two byte runs of one length, case-sensitively as `memcmp` does or
/// caselessly as `caselessCompare` does.
fn compare_run(
    left: impl Iterator<Item = u8>,
    right: impl Iterator<Item = u8>,
    caseless: bool,
) -> Ordering {
    for (left, right) in left.zip(right) {
        let ordering = if caseless {
            caseless_byte(left).cmp(&caseless_byte(right))
        } else {
            left.cmp(&right)
        };
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    Ordering::Equal
}

/// `RexxString::sortCompare` and `sortCaselessCompare` with no columns: the
/// shared prefix, then the shorter first.
fn compare_whole(left: &[u8], right: &[u8], caseless: bool) -> Ordering {
    let shared = left.len().min(right.len());
    compare_run(
        left[..shared].iter().copied(),
        right[..shared].iter().copied(),
        caseless,
    )
    .then(left.len().cmp(&right.len()))
}

/// The column forms of the same two comparisons.
///
/// The run compared reaches one byte past the shorter value, where the C++
/// reads the string's terminating NUL.
fn compare_columns(
    left: &[u8],
    right: &[u8],
    start: usize,
    length: usize,
    caseless: bool,
) -> Ordering {
    if start >= left.len() || start >= right.len() {
        return left.len().cmp(&right.len());
    }
    let reachable = left.len().min(right.len()) - start + 1;
    let compared = length.min(reachable);
    let ordering = compare_run(
        terminated(left, start, compared),
        terminated(right, start, compared),
        caseless,
    );
    if ordering == Ordering::Equal && reachable < length {
        return left.len().cmp(&right.len());
    }
    ordering
}

/// `count` bytes of `value` from `start`, a NUL standing for each byte past
/// its end.
fn terminated(value: &[u8], start: usize, count: usize) -> impl Iterator<Item = u8> + '_ {
    (start..start + count).map(|at| value.get(at).copied().unwrap_or(0))
}

/// `StemClass::mergeSort` over `order`, one index per element: the oracle's
/// own algorithm, an insertion sort below eight elements and an exponential
/// search in the merge, so the order it leaves is the oracle's whatever the
/// comparator answers.
fn merge_sort(
    order: &mut [usize],
    working: &mut [usize],
    left: usize,
    right: usize,
    compare: &dyn Fn(usize, usize) -> i32,
) {
    if right - left < 7 {
        for i in left + 1..=right {
            let current = order[i];
            let mut previous = order[i - 1];
            if compare(current, previous) < 0 {
                let mut j = i;
                loop {
                    order[j] = previous;
                    j -= 1;
                    if j <= left {
                        break;
                    }
                    previous = order[j - 1];
                    if compare(current, previous) >= 0 {
                        break;
                    }
                }
                order[j] = current;
            }
        }
        return;
    }
    let middle = (left + right) / 2;
    merge_sort(order, working, left, middle, compare);
    merge_sort(order, working, middle + 1, right, compare);
    merge(order, working, left, middle + 1, right, compare);
}

/// `StemClass::merge`: joins the sorted runs `left..middle` and
/// `middle..=right`, copying each stretch past an exponential search.
fn merge(
    order: &mut [usize],
    working: &mut [usize],
    left: usize,
    middle: usize,
    right: usize,
    compare: &dyn Fn(usize, usize) -> i32,
) {
    let left_end = middle - 1;
    if compare(order[left_end], order[middle]) <= 0 {
        return;
    }
    let (mut left_cursor, mut right_cursor, mut out) = (left, middle, left);
    loop {
        let from_value = order[left_cursor];
        let right_value = order[right_cursor];
        if compare(from_value, right_value) <= 0 {
            let insertion = find(order, right_value, -1, left_cursor + 1, left_end, compare);
            let stretch = insertion + 1 - left_cursor;
            working[out..out + stretch].copy_from_slice(&order[left_cursor..left_cursor + stretch]);
            out += stretch;
            working[out] = right_value;
            out += 1;
            right_cursor += 1;
            left_cursor = insertion + 1;
        } else {
            let insertion = find(order, from_value, 0, right_cursor + 1, right, compare);
            let stretch = insertion + 1 - right_cursor;
            working[out..out + stretch]
                .copy_from_slice(&order[right_cursor..right_cursor + stretch]);
            out += stretch;
            working[out] = from_value;
            out += 1;
            left_cursor += 1;
            right_cursor = insertion + 1;
        }
        if !(right >= right_cursor && middle > left_cursor) {
            break;
        }
    }
    if left_cursor < middle {
        let stretch = middle - left_cursor;
        working[out..out + stretch].copy_from_slice(&order[left_cursor..middle]);
    } else {
        let stretch = right + 1 - right_cursor;
        working[out..out + stretch].copy_from_slice(&order[right_cursor..=right]);
    }
    order[left..=right].copy_from_slice(&working[left..=right]);
}

/// `StemClass::find`: the last index from `left` to `right` whose element
/// `value` compares above `limit` against, or `left - 1` where there is none.
fn find(
    order: &[usize],
    value: usize,
    limit: i32,
    left: usize,
    right: usize,
    compare: &dyn Fn(usize, usize) -> i32,
) -> usize {
    // Signed, because the C++'s `right = checkPoint - 1` may pass below
    // `left` and the loops end on that.
    let (mut left, mut right) = (left as isize, right as isize);
    let mut check = left;
    let mut delta = 1;
    while check <= right {
        if compare(value, order[check as usize]) > limit {
            left = check + 1;
        } else {
            right = check - 1;
            break;
        }
        check += delta;
        delta *= 2;
    }
    while left <= right {
        check = (left + right) / 2;
        if compare(value, order[check as usize]) > limit {
            left = check + 1;
        } else {
            right = check - 1;
        }
    }
    (left - 1) as usize
}

/// The stem `SysStemSort` sorts and the tail prefix its elements carry: the
/// argument resolved as a stem, else a name like `s.foo` split at its first
/// period into the stem `s.` and the prefix `FOO`.
fn sort_target(interp: &mut Interp, argument: ObjRef) -> Result<(ObjRef, Vec<u8>), Failure> {
    if interp.is_stem_object(argument) {
        return Ok((argument, Vec::new()));
    }
    let text = interp.required_string_value(argument)?;
    let name = interp.to_text(text).into_owned();
    if let Some(stem) = interp.context_stem_named(&name) {
        interp.roots.activity_mut().push_temp(stem);
        return Ok((stem, Vec::new()));
    }
    let unresolved = || Failure::from(Raised::stem_argument_unresolved(1, &name));
    let Some(period) = name.iter().position(|byte| *byte == b'.') else {
        return Err(unresolved());
    };
    if period + 1 == name.len() {
        return Err(unresolved());
    }
    let Some(stem) = interp.context_stem_named(&name[..=period]) else {
        return Err(unresolved());
    };
    interp.roots.activity_mut().push_temp(stem);
    let mut prefix = name[period + 1..].to_ascii_uppercase();
    if prefix.last() != Some(&b'.') {
        prefix.push(b'.');
    }
    Ok((stem, prefix))
}

/// `SysStemSort(stem [, order [, type [, first [, last [, firstcol [,
/// lastcol]]]]]])`: sorts elements `first` to `last` by their string values,
/// which replace them.
///
/// `last` and `lastcol` of `999999999999999999` mean "to the end" whether
/// given or defaulted, as in the C++, which marks the default with that value.
pub(crate) fn sort(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 7)?;
    let argument = required(args, 1)?;
    let order = optional_text(interp, args, 2)?;
    let kind = optional_text(interp, args, 3)?;
    let first = optional_positive(interp, args, 4)?;
    let last = optional_positive(interp, args, 5)?;
    let first_column = optional_positive(interp, args, 6)?;
    let last_column = optional_positive(interp, args, 7)?;
    let (stem, prefix) = sort_target(interp, argument)?;
    let descending = match order.as_deref() {
        None => false,
        Some(order) => match order.first().map(u8::to_ascii_uppercase) {
            Some(b'A') => false,
            Some(b'D') => true,
            _ => {
                return Err(Raised::invalid_routine_option(
                    "SysStemSort",
                    "sort order",
                    "'A' or 'D'",
                    order,
                )
                .into());
            }
        },
    };
    let caseless = match kind.as_deref() {
        None => false,
        Some(kind) => match kind.first().map(u8::to_ascii_uppercase) {
            Some(b'C') => false,
            Some(b'I') => true,
            _ => {
                return Err(Raised::invalid_routine_option(
                    "SysStemSort",
                    "sort type",
                    "'C' or 'I'",
                    kind,
                )
                .into());
            }
        },
    };
    let first = first.unwrap_or(1);
    let last = last.unwrap_or(MAX_WHOLENUMBER);
    if last < first {
        return Err(Raised::arguments_out_of_order("last", last, "first", first).into());
    }
    let first_column = first_column.unwrap_or(1);
    let last_column = last_column.unwrap_or(MAX_WHOLENUMBER);
    if last_column < first_column {
        return Err(Raised::arguments_out_of_order(
            "last column",
            last_column,
            "first column",
            first_column,
        )
        .into());
    }

    let tail = |index: i64| {
        let mut tail = prefix.clone();
        tail.extend_from_slice(&index_tail(index));
        tail
    };
    let count = interp
        .stem_object_tail(stem, &tail(0))
        .map(|value| sort_count(interp, value))
        .transpose()?
        .flatten()
        .ok_or_else(|| Failure::from(Raised::stem_count_invalid()))?;
    if count == 0 {
        return Ok(answer(interp, 0));
    }
    let last = if last == MAX_WHOLENUMBER { count } else { last };
    if first > count || last > count {
        return Err(Raised::stem_range(count).into());
    }
    let mut values = Vec::new();
    let mut texts = Vec::new();
    for index in first..=last {
        let Some(value) = interp.stem_object_tail(stem, &tail(index)) else {
            return Err(Raised::stem_element_unset(index).into());
        };
        let value = interp.required_string_value(value)?;
        interp.roots.activity_mut().push_temp(value);
        texts.push(interp.to_text(value).into_owned());
        values.push(value);
    }
    let columns = (first_column != 1 || last_column != MAX_WHOLENUMBER).then(|| {
        let start = usize::try_from(first_column - 1).expect("a positive column");
        let length = usize::try_from(last_column - first_column + 1).expect("an ordered range");
        (start, length)
    });
    let sorting = SortOrder {
        descending,
        caseless,
        columns,
    };
    let compare = |left: usize, right: usize| sorting.compare(&texts[left], &texts[right]);
    let mut order: Vec<usize> = (0..values.len()).collect();
    let mut working = vec![0; order.len()];
    let right = order.len() - 1;
    merge_sort(&mut order, &mut working, 0, right, &compare);
    for (index, at) in (first..=last).zip(order) {
        interp.stem_object_write(stem, &tail(index), values[at]);
    }
    Ok(answer(interp, 0))
}

/// Element 0 as `StemClass::sort` reads it: an unsigned whole number at nine
/// digits, `None` where it is not one.
fn sort_count(interp: &mut Interp, value: ObjRef) -> Result<Option<i64>, Failure> {
    let count = match value.decode() {
        Decoded::SmallInt(number) => u64::try_from(number).ok(),
        _ => {
            let text = interp.required_string_or_nil(value)?;
            Number::parse_bytes(&interp.to_text(text))
                .and_then(|number| number.unsigned_int64_value(rexx_num::DEFAULT_DIGITS as usize))
        }
    };
    Ok(count
        .filter(|count| *count <= SORT_COUNT_MAXIMUM)
        .map(|count| i64::try_from(count).expect("nine digits fit")))
}

/// `SysDumpVariables([file])`: every variable of the caller's, a line each
/// as `Name=NAME, Value='value'`, and after a stem's own line one per tail
/// that holds a value, which ends in CR LF where a simple variable's ends in
/// LF. Appended to `file`, or written to standard output.
///
/// A file that will not open is 40.1.
pub(crate) fn dump_variables(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    check_arity(args, 1)?;
    let target = optional_text(interp, args, 1)?;
    let mut file = match target {
        None => None,
        Some(text) => {
            let path = crate::builtin::platform::qualify_name(interp, c_string(&text));
            let opened = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(os_path(&path));
            match opened {
                Ok(file) => Some(file),
                Err(_) => {
                    return Err(Raised::routine_failed(&name.to_ascii_uppercase()).into());
                }
            }
        }
    };
    let variables = crate::dispatch::context::local_variable_entries(interp, 0)?;
    for (_, value) in &variables {
        interp.roots.activity_mut().push_temp(*value);
    }
    for (variable, value) in variables {
        if variable.last() == Some(&b'.') && interp.is_stem_object(value) {
            let own = interp.stem_own_value(value);
            dump_line(interp, &mut file, &variable, own, b"\n")?;
            let tails = interp.stem_live_tails(value);
            for (_, held) in &tails {
                interp.roots.activity_mut().push_temp(*held);
            }
            for (tail, held) in tails {
                let mut compound = variable.clone();
                compound.extend_from_slice(&tail);
                dump_line(interp, &mut file, &compound, held, b"\r\n")?;
            }
        } else {
            dump_line(interp, &mut file, &variable, value, b"\n")?;
        }
    }
    Ok(answer(interp, 0))
}

/// Writes one `Name=..., Value='...'` line to `file`, or to standard output
/// where there is none, as soon as the value's string value is in hand: a
/// `STRING` method's own output lands between the lines, as on the oracle.
fn dump_line(
    interp: &mut Interp,
    file: &mut Option<std::fs::File>,
    name: &[u8],
    value: ObjRef,
    end: &[u8],
) -> Result<(), Failure> {
    let text = interp.required_string_value(value)?;
    let mut line = b"Name=".to_vec();
    line.extend_from_slice(name);
    line.extend_from_slice(b", Value='");
    line.extend_from_slice(&interp.to_text(text));
    line.extend_from_slice(b"'");
    line.extend_from_slice(end);
    match file {
        Some(file) => {
            use std::io::Write as _;
            // The oracle ignores a failed write.
            let _ = file.write_all(&line);
        }
        None => interp.write_out(&line),
    }
    Ok(())
}

#[cfg(test)]
mod tests;
