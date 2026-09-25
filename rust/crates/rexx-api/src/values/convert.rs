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

//! The per-code conversions the rows of `TABLE` name: one function for each
//! `REXX_VALUE_*` code and direction, and the helpers they share.

use std::ffi::c_int;

use rexx_core::ObjRef;

use super::{Class, Conversion, Failure, MAX_WHOLENUMBER, RESULT_DIGITS, Value, pointer_string};

/// `REXX_VALUE_ARGLIST` (`NativeActivation.cpp:306`).
pub(super) fn arglist_to_native(
    cx: &mut Conversion<'_>,
    _argument: ObjRef,
    _position: usize,
) -> Result<Value, Failure> {
    let arguments = cx.host.arguments();
    Ok(Value::Object(cx.host.locals().register(arguments)))
}

/// `REXX_VALUE_NAME` (`NativeActivation.cpp:317`).
pub(super) fn name_to_native(
    cx: &mut Conversion<'_>,
    _argument: ObjRef,
    _position: usize,
) -> Result<Value, Failure> {
    let name = cx.host.message_name();
    Ok(Value::CString(cx.strings.intern(&name)))
}

/// `REXX_VALUE_SCOPE` (`NativeActivation.cpp:266`).
pub(super) fn scope_to_native(
    cx: &mut Conversion<'_>,
    _argument: ObjRef,
    _position: usize,
) -> Result<Value, Failure> {
    if !cx.host.is_method() {
        return Err(Failure::Signature);
    }
    let scope = cx.host.scope();
    Ok(Value::Object(cx.host.locals().register(scope)))
}

/// `REXX_VALUE_CSELF` (`NativeActivation.cpp:294`).
pub(super) fn cself_to_native(
    cx: &mut Conversion<'_>,
    _argument: ObjRef,
    _position: usize,
) -> Result<Value, Failure> {
    if !cx.host.is_method() {
        return Err(Failure::Signature);
    }
    Ok(Value::Pointer(
        cx.host.cself().unwrap_or(std::ptr::null_mut()),
    ))
}

/// `REXX_VALUE_OSELF` (`NativeActivation.cpp:251`).
pub(super) fn oself_to_native(
    cx: &mut Conversion<'_>,
    _argument: ObjRef,
    _position: usize,
) -> Result<Value, Failure> {
    if !cx.host.is_method() {
        return Err(Failure::Signature);
    }
    let receiver = cx.host.receiver();
    Ok(Value::Object(cx.host.locals().register(receiver)))
}

/// `REXX_VALUE_SUPER` (`NativeActivation.cpp:281`).
pub(super) fn super_to_native(
    cx: &mut Conversion<'_>,
    _argument: ObjRef,
    _position: usize,
) -> Result<Value, Failure> {
    if !cx.host.is_method() {
        return Err(Failure::Signature);
    }
    let scope = cx.host.super_scope();
    Ok(Value::Object(cx.host.locals().register(scope)))
}

/// `REXX_VALUE_RexxObjectPtr` (`NativeActivation.cpp:335`).
pub(super) fn object_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    _position: usize,
) -> Result<Value, Failure> {
    Ok(Value::Object(cx.host.locals().register(argument)))
}

/// A row `signedIntegerValue(argument, position, max, min)` converts
/// (`NativeActivation.cpp:1901`), answering the value as the C type `T`.
fn signed<T: TryFrom<i64>>(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
    min: i64,
    max: i64,
) -> Result<T, Failure> {
    cx.host
        .signed_integer(argument, min, max)?
        .filter(|number| (min..=max).contains(number))
        .and_then(|number| T::try_from(number).ok())
        .ok_or(Failure::OutOfRange {
            position,
            min: i128::from(min),
            max: i128::from(max),
            argument,
        })
}

/// A row `unsignedIntegerValue(argument, position, max)` converts
/// (`NativeActivation.cpp:1966`), answering the value as the C type `T`.
fn unsigned<T: TryFrom<u64>>(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
    max: u64,
) -> Result<T, Failure> {
    cx.host
        .unsigned_integer(argument, max)?
        .filter(|number| *number <= max)
        .and_then(|number| T::try_from(number).ok())
        .ok_or(Failure::OutOfRange {
            position,
            min: 0,
            max: i128::from(max),
            argument,
        })
}

/// `REXX_VALUE_int` (`NativeActivation.cpp:341`).
pub(super) fn int_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    let (min, max) = (i64::from(c_int::MIN), i64::from(c_int::MAX));
    signed(cx, argument, position, min, max).map(Value::Int)
}

/// `REXX_VALUE_int8_t` (`NativeActivation.cpp:348`).
pub(super) fn int8_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    let (min, max) = (i64::from(i8::MIN), i64::from(i8::MAX));
    signed(cx, argument, position, min, max).map(Value::Int8)
}

/// `REXX_VALUE_int16_t` (`NativeActivation.cpp:354`).
pub(super) fn int16_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    let (min, max) = (i64::from(i16::MIN), i64::from(i16::MAX));
    signed(cx, argument, position, min, max).map(Value::Int16)
}

/// `REXX_VALUE_int32_t` (`NativeActivation.cpp:360`).
pub(super) fn int32_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    let (min, max) = (i64::from(i32::MIN), i64::from(i32::MAX));
    signed(cx, argument, position, min, max).map(Value::Int32)
}

/// `REXX_VALUE_int64_t` (`NativeActivation.cpp:366`), whose `int64Value`
/// converts at the same twenty digits as `signedIntegerValue`.
pub(super) fn int64_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    signed(cx, argument, position, i64::MIN, i64::MAX).map(Value::Int64)
}

/// `REXX_VALUE_ssize_t` and `REXX_VALUE_intptr_t` (`NativeActivation.cpp:372`,
/// `:378`), both the whole range of a pointer-sized signed integer.
pub(super) fn signed_word_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    signed(cx, argument, position, i64::MIN, i64::MAX).map(Value::Isize)
}

/// `REXX_VALUE_wholenumber_t` (`NativeActivation.cpp:427`).
pub(super) fn whole_number_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    signed(cx, argument, position, -MAX_WHOLENUMBER, MAX_WHOLENUMBER).map(Value::Isize)
}

/// `REXX_VALUE_uint8_t` (`NativeActivation.cpp:384`).
pub(super) fn uint8_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    unsigned(cx, argument, position, u64::from(u8::MAX)).map(Value::Uint8)
}

/// `REXX_VALUE_uint16_t` (`NativeActivation.cpp:390`).
pub(super) fn uint16_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    unsigned(cx, argument, position, u64::from(u16::MAX)).map(Value::Uint16)
}

/// `REXX_VALUE_uint32_t` (`NativeActivation.cpp:396`).
pub(super) fn uint32_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    unsigned(cx, argument, position, u64::from(u32::MAX)).map(Value::Uint32)
}

/// `REXX_VALUE_uint64_t` (`NativeActivation.cpp:402`), whose
/// `unsignedInt64Value` converts at the same twenty digits as
/// `unsignedIntegerValue`.
pub(super) fn uint64_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    unsigned(cx, argument, position, u64::MAX).map(Value::Uint64)
}

/// `REXX_VALUE_size_t` and `REXX_VALUE_uintptr_t` (`NativeActivation.cpp:408`,
/// `:414`), both the whole range of a pointer-sized unsigned integer.
pub(super) fn unsigned_word_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    unsigned(cx, argument, position, u64::MAX).map(Value::Usize)
}

/// `REXX_VALUE_stringsize_t` (`NativeActivation.cpp:448`).
pub(super) fn string_size_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    let max = MAX_WHOLENUMBER.unsigned_abs();
    unsigned(cx, argument, position, max).map(Value::Usize)
}

/// `REXX_VALUE_positive_wholenumber_t` (`NativeActivation.cpp:434`).
pub(super) fn positive_whole_number_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    cx.host
        .signed_integer(argument, 1, MAX_WHOLENUMBER)?
        .filter(|number| (1..=MAX_WHOLENUMBER).contains(number))
        .and_then(|number| isize::try_from(number).ok())
        .map(Value::Isize)
        .ok_or(Failure::NotPositive { position, argument })
}

/// `REXX_VALUE_nonnegative_wholenumber_t` (`NativeActivation.cpp:441`).
pub(super) fn nonnegative_whole_number_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    cx.host
        .signed_integer(argument, 0, MAX_WHOLENUMBER)?
        .filter(|number| (0..=MAX_WHOLENUMBER).contains(number))
        .and_then(|number| isize::try_from(number).ok())
        .map(Value::Isize)
        .ok_or(Failure::NotNonnegative { position, argument })
}

/// `REXX_VALUE_logical_t` (`NativeActivation.cpp:420`).
pub(super) fn logical_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    _position: usize,
) -> Result<Value, Failure> {
    match cx.host.logical(argument)? {
        Ok(truth) => Ok(Value::Usize(usize::from(truth))),
        Err(found) => Err(Failure::NotLogical { found }),
    }
}

/// `REXX_VALUE_double` (`NativeActivation.cpp:454`).
pub(super) fn double_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    cx.host
        .double_value(argument)?
        .map(Value::Double)
        .ok_or(Failure::InvalidDouble { position, argument })
}

/// `REXX_VALUE_float` (`NativeActivation.cpp:461`).
pub(super) fn float_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the C++'s own `(float)` cast, which is the conversion measured"
    )]
    let narrow = |value: f64| Value::Float(value as f32);
    cx.host
        .double_value(argument)?
        .map(narrow)
        .ok_or(Failure::InvalidDouble { position, argument })
}

/// `REXX_VALUE_CSTRING` (`NativeActivation.cpp:467`).
pub(super) fn cstring_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    let string = cx
        .host
        .string_value(argument)?
        .ok_or(Failure::NoStringValue { position })?;
    let bytes = cx
        .host
        .string_bytes(string)
        .ok_or(Failure::NoStringValue { position })?;
    Ok(Value::CString(cx.strings.intern(&bytes)))
}

/// `REXX_VALUE_RexxStringObject` (`NativeActivation.cpp:473`).
pub(super) fn string_object_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    let string = cx
        .host
        .string_value(argument)?
        .ok_or(Failure::NoStringValue { position })?;
    // The C++ registers a local reference only for a string the conversion
    // had to create. Minting a handle is registering it, so here every one is
    // rooted and the "was it created" question does not arise.
    Ok(Value::Object(cx.host.locals().register(string)))
}

/// `REXX_VALUE_RexxArrayObject` (`NativeActivation.cpp:488`).
pub(super) fn array_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    _position: usize,
) -> Result<Value, Failure> {
    let array = cx
        .host
        .array_value(argument)?
        .ok_or(Failure::NotArray { argument })?;
    Ok(Value::Object(cx.host.locals().register(array)))
}

/// `REXX_VALUE_RexxStemObject` (`NativeActivation.cpp:502`): a stem, or in a
/// call the name of the caller's.
pub(super) fn stem_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    let stem = if cx.host.is_stem(argument) {
        argument
    } else if cx.host.is_method() {
        return Err(Failure::NoStem { position, argument });
    } else {
        cx.host
            .context_stem(argument)?
            .ok_or(Failure::NoStem { position, argument })?
    };
    Ok(Value::Object(cx.host.locals().register(stem)))
}

/// An argument that must be an instance of `class`, handed over as it is.
fn instance(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
    class: Class,
) -> Result<Value, Failure> {
    if !cx.host.is_instance_of(argument, class) {
        return Err(Failure::NotInstance { position, class });
    }
    Ok(Value::Object(cx.host.locals().register(argument)))
}

/// `REXX_VALUE_RexxClassObject` (`NativeActivation.cpp:549`).
pub(super) fn class_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    instance(cx, argument, position, Class::Class)
}

/// `REXX_VALUE_RexxMutableBufferObject` (`NativeActivation.cpp:577`).
pub(super) fn mutable_buffer_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    instance(cx, argument, position, Class::MutableBuffer)
}

/// `REXX_VALUE_RexxVariableReferenceObject` (`NativeActivation.cpp:588`).
pub(super) fn variable_reference_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    instance(cx, argument, position, Class::VariableReference)
}

/// `REXX_VALUE_POINTER` (`NativeActivation.cpp:560`).
pub(super) fn pointer_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    cx.host
        .pointer_value(argument)
        .map(Value::Pointer)
        .ok_or(Failure::NotInstance {
            position,
            class: Class::Pointer,
        })
}

/// `REXX_VALUE_POINTERSTRING` (`NativeActivation.cpp:571`).
pub(super) fn pointer_string_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    let text = cx.host.string_value_text(argument);
    pointer_string(&text)
        .map(|address| Value::Pointer(std::ptr::without_provenance_mut(address)))
        .ok_or(Failure::NotPointerString { position, argument })
}

/// `valueToObject` for the object codes (`NativeActivation.cpp:722`).
pub(super) fn object_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Object(handle) = value else {
        return Err(Failure::ResultSignature);
    };
    if handle.is_null() {
        return Ok(None);
    }
    cx.host
        .locals()
        .resolve(handle)
        .map(Some)
        .ok_or(Failure::StaleHandle)
}

/// `valueToObject` for `REXX_VALUE_POINTER` (`NativeActivation.cpp:839`),
/// which wraps the address in a `.Pointer` and does not register it: the
/// answer is the call's result, which the caller roots.
pub(super) fn pointer_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Pointer(address) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(cx.host.new_pointer(address)))
}

/// `valueToObject` for `REXX_VALUE_POINTERSTRING` (`NativeActivation.cpp:842`).
pub(super) fn pointer_string_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Pointer(address) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(
        cx.host.new_string(&rexx_core::pointer_to_string(address)),
    ))
}

/// `valueToObject` for `REXX_VALUE_int` (`NativeActivation.cpp:733`), built
/// here rather than through the host: a `c_int` is a tagged value, and asking
/// the host for it would add an allocation point the call does not have.
///
/// # Panics
/// Never for a `c_int`, whose whole range is a small integer.
pub(super) fn int_from_native(
    _cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Int(number) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(
        ObjRef::small_int(i64::from(number)).expect("a c_int is a small integer"),
    ))
}

/// `valueToObject` for `REXX_VALUE_int8_t` (`NativeActivation.cpp:738`).
pub(super) fn int8_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Int8(number) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(cx.host.whole_number(isize::from(number))))
}

/// `valueToObject` for `REXX_VALUE_int16_t` (`NativeActivation.cpp:743`).
pub(super) fn int16_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Int16(number) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(cx.host.whole_number(isize::from(number))))
}

/// `valueToObject` for `REXX_VALUE_int32_t` (`NativeActivation.cpp:748`).
pub(super) fn int32_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Int32(number) = value else {
        return Err(Failure::ResultSignature);
    };
    let number = isize::try_from(number).map_err(|_| Failure::ResultSignature)?;
    Ok(Some(cx.host.whole_number(number)))
}

/// `valueToObject` for `REXX_VALUE_int64_t` (`NativeActivation.cpp:753`),
/// whose `int64ToObject` answers what `wholenumberToObject` does for a
/// pointer-sized value.
pub(super) fn int64_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Int64(number) = value else {
        return Err(Failure::ResultSignature);
    };
    let number = isize::try_from(number).map_err(|_| Failure::ResultSignature)?;
    Ok(Some(cx.host.whole_number(number)))
}

/// `valueToObject` for the pointer-sized signed codes, `wholenumberToObject`
/// (`NativeActivation.cpp:758`, `:798`, `:803-805`).
pub(super) fn isize_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Isize(number) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(cx.host.whole_number(number)))
}

/// `valueToObject` for `REXX_VALUE_uint8_t` (`NativeActivation.cpp:763`).
pub(super) fn uint8_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Uint8(number) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(cx.host.unsigned_number(u64::from(number))))
}

/// `valueToObject` for `REXX_VALUE_uint16_t` (`NativeActivation.cpp:768`).
pub(super) fn uint16_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Uint16(number) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(cx.host.unsigned_number(u64::from(number))))
}

/// `valueToObject` for `REXX_VALUE_uint32_t` (`NativeActivation.cpp:773`).
pub(super) fn uint32_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Uint32(number) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(cx.host.unsigned_number(u64::from(number))))
}

/// `valueToObject` for `REXX_VALUE_uint64_t` (`NativeActivation.cpp:778`).
pub(super) fn uint64_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Uint64(number) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(cx.host.unsigned_number(number)))
}

/// `valueToObject` for the pointer-sized unsigned codes, `stringsizeToObject`
/// (`NativeActivation.cpp:783`, `:793`, `:810`).
pub(super) fn usize_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Usize(number) = value else {
        return Err(Failure::ResultSignature);
    };
    let number = u64::try_from(number).map_err(|_| Failure::ResultSignature)?;
    Ok(Some(cx.host.unsigned_number(number)))
}

/// `valueToObject` for `REXX_VALUE_logical_t` (`NativeActivation.cpp:788`):
/// any value but zero is true.
pub(super) fn logical_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Usize(number) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(cx.host.whole_number(isize::from(number != 0))))
}

/// `valueToObject` for `REXX_VALUE_double` (`NativeActivation.cpp:815`).
pub(super) fn double_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Double(number) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(cx.host.double_object(number, RESULT_DIGITS)))
}

/// `valueToObject` for `REXX_VALUE_float` (`NativeActivation.cpp:820`), which
/// widens to a `double`.
pub(super) fn float_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::Float(number) = value else {
        return Err(Failure::ResultSignature);
    };
    Ok(Some(
        cx.host.double_object(f64::from(number), RESULT_DIGITS),
    ))
}

/// `valueToObject` for `REXX_VALUE_CSTRING` (`NativeActivation.cpp:825`): no
/// object for a null pointer, and a string of the bytes before the first NUL
/// otherwise.
pub(super) fn cstring_from_native(
    cx: &mut Conversion<'_>,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let Value::CString(pointer) = value else {
        return Err(Failure::ResultSignature);
    };
    if pointer.is_null() {
        return Ok(None);
    }
    let Some(bytes) = cx.strings.bytes_at(pointer) else {
        return Err(Failure::StaleHandle);
    };
    let bytes = bytes.split(|byte| *byte == 0).next().unwrap_or_default();
    Ok(Some(cx.host.new_string(bytes)))
}
