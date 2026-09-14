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

// D-U1, 2026-09-14
#![allow(unsafe_code)]

//! The inbound FFI boundary: caller-supplied pointers into validated handles.

use std::ffi::CStr;
use std::marker::PhantomData;

use crate::layout::{
    CSTRING, MethodContextInterface, Owned, POINTER, RexxMethodContext_, RexxObjectPtr,
    RexxPointerObject, RexxStringObject, RexxThreadContext_, RexxThreadInterface, ValueDescriptor,
    wholenumber_t,
};
use crate::values::{Activation, Repr, Value};

/// The state that owns the context `context` addresses.
///
/// `context` must be the public struct at the head of an [`Owned`] we handed
/// out, which is how the C++ recovers the owning activation
/// (`interpreter/concurrency/Activity.hpp:503`).
///
/// # Safety
/// `context` came from [`Owned::<C, T>::context`] on a wrapper that is still
/// alive, with the same `C` and `T`. Nothing about the pointer witnesses
/// that: an extension holds it as an opaque address and can hand back any
/// value at all, so the guarantee is the interpreter's, which mints every
/// context it hands out and never hands out one it did not build.
pub unsafe fn owner_of<C, T>(context: *mut C) -> *mut T {
    let owned = context.cast::<Owned<C, T>>();
    // SAFETY: the caller guarantees `owned` addresses a live `Owned<C, T>`,
    // whose `context` is first by value, so the cast is the identity on the
    // address and `owner` is in bounds.
    unsafe { (*owned).owner }
}

/// The value `descriptor` carries, read as the union member `repr` names.
///
/// Safe code can build a descriptor whose written member is narrower than
/// the one `repr` names, which is why the read is `unsafe` to ask for:
///
/// ```compile_fail,E0133
/// # use rexx_api::ffi::value_of;
/// # use rexx_api::layout::{ValueDescriptor, ValueUnion};
/// # use rexx_api::values::Repr;
/// let narrow = ValueDescriptor {
///     value: ValueUnion { value_float: 1.5 },
///     r#type: 0,
///     flags: 0,
/// };
/// let read = value_of(&narrow, Repr::Double);
/// ```
///
/// # Safety
/// Every byte of the member `repr` names is initialised: `descriptor`'s word
/// was written in full, as every union [`crate::values::descriptor`] builds is,
/// or that member itself was written.
pub unsafe fn value_of(descriptor: &ValueDescriptor, repr: Repr) -> Value {
    // SAFETY: the caller guarantees the member's bytes are initialised, and
    // every member is an integer, a float or a pointer, for which any
    // initialised bytes are a valid value.
    unsafe {
        match repr {
            Repr::Object => Value::Object(descriptor.value.value_RexxObjectPtr),
            Repr::CString => Value::CString(descriptor.value.value_CSTRING),
            Repr::Pointer => Value::Pointer(descriptor.value.value_POINTER),
            Repr::Int => Value::Int(descriptor.value.value_int),
            Repr::Int8 => Value::Int8(descriptor.value.value_int8_t),
            Repr::Int16 => Value::Int16(descriptor.value.value_int16_t),
            Repr::Int32 => Value::Int32(descriptor.value.value_int32_t),
            Repr::Int64 => Value::Int64(descriptor.value.value_int64_t),
            Repr::Uint8 => Value::Uint8(descriptor.value.value_uint8_t),
            Repr::Uint16 => Value::Uint16(descriptor.value.value_uint16_t),
            Repr::Uint32 => Value::Uint32(descriptor.value.value_uint32_t),
            Repr::Uint64 => Value::Uint64(descriptor.value.value_uint64_t),
            Repr::Isize => Value::Isize(descriptor.value.value_wholenumber_t),
            Repr::Usize => Value::Usize(descriptor.value.value_stringsize_t),
            Repr::Double => Value::Double(descriptor.value.value_double),
            Repr::Float => Value::Float(descriptor.value.value_float),
        }
    }
}

/// The method-context table an extension is handed
/// (`Activity::methodContextFunctions`,
/// `interpreter/api/MethodContextStubs.cpp:374`).
///
/// A plain initializer nothing patches, so it lives at one address for the
/// process; the members this phase has not written still refuse loudly.
pub static METHOD_CONTEXT: MethodContextInterface = {
    let mut table = MethodContextInterface::REFUSING;
    table.SetObjectVariable = set_object_variable;
    table.DropObjectVariable = drop_object_variable;
    table
};

/// The contexts one native call hands an extension, wired to the state behind
/// them.
///
/// The thread table is owned here and not shared, because its object members
/// are handles this activation minted
/// (`interpreter/concurrency/Activity.cpp:1846-1849`).
pub struct Contexts<'a, 'h> {
    thread: Owned<RexxThreadContext_, Activation<'h>>,
    method: Owned<RexxMethodContext_, Activation<'h>>,
    table: RexxThreadInterface,
    /// Ties this wrapper to the activation its tables address, so that no
    /// context it hands out can outlive the state behind it.
    activation: PhantomData<&'a Activation<'h>>,
}

impl<'a, 'h> Contexts<'a, 'h> {
    /// The contexts for a call whose state is `activation`.
    pub fn new(activation: &'a Activation<'h>) -> Contexts<'a, 'h> {
        let owner = std::ptr::from_ref(activation).cast_mut();
        let constants = activation.constants();
        let mut table = RexxThreadInterface::REFUSING;
        table.WholeNumberToObject = whole_number_to_object;
        table.StringData = string_data;
        table.StringLength = string_length;
        table.NewPointer = new_pointer;
        table.RaiseException0 = raise_exception0;
        table.RexxNil = constants.nil;
        table.RexxTrue = constants.true_object;
        table.RexxFalse = constants.false_object;
        table.RexxNullString = constants.null_string.cast();
        Contexts {
            thread: Owned {
                context: RexxThreadContext_ {
                    instance: std::ptr::null_mut(),
                    functions: std::ptr::null_mut(),
                },
                owner,
            },
            method: Owned {
                context: RexxMethodContext_ {
                    threadContext: std::ptr::null_mut(),
                    functions: std::ptr::null_mut(),
                    arguments: std::ptr::null_mut(),
                },
                owner,
            },
            table,
            activation: PhantomData,
        }
    }

    /// The method context, addressing this wrapper's own thread context and
    /// tables.
    ///
    /// The links are written here rather than at construction because each
    /// one is the address of a field of `self`, which moving `self` changes.
    pub fn method(&mut self) -> &mut RexxMethodContext_ {
        self.thread.context.functions = &raw mut self.table;
        self.method.context.threadContext = (&raw mut self.thread).cast::<RexxThreadContext_>();
        self.method.context.functions = std::ptr::from_ref(&METHOD_CONTEXT).cast_mut();
        &mut self.method.context
    }

    /// The handles the thread table's data members carry.
    pub fn constants(&self) -> crate::values::Constants<RexxObjectPtr> {
        crate::values::Constants {
            nil: self.table.RexxNil,
            true_object: self.table.RexxTrue,
            false_object: self.table.RexxFalse,
            null_string: self.table.RexxNullString.cast(),
        }
    }
}

/// The activation `context` addresses.
///
/// # Safety
/// `context` is a context handed out by a [`Contexts`] that is still alive,
/// and no caller holds its conversion state for the duration of the call this
/// reference is used in.
unsafe fn activation_of<'a, C>(context: *mut C) -> &'a Activation<'a> {
    // SAFETY: the caller guarantees `context` came from a live `Contexts`,
    // which builds both of its wrappers with `owner` pointing at the
    // `&'a Activation` it was given. That reference is shared, so forming
    // another one here aliases nothing: every write past it goes through the
    // activation's own cells.
    unsafe { &*owner_of::<C, Activation<'a>>(context) }
}

/// The bytes of a name an extension passed, or `None` for a null pointer.
///
/// # Safety
/// A non-null `name` is a NUL-terminated string that outlives the call.
unsafe fn name_of<'a>(name: CSTRING) -> Option<&'a [u8]> {
    if name.is_null() {
        return None;
    }
    // SAFETY: the caller guarantees a non-null `name` is a live
    // NUL-terminated string; the extension's own literal is one
    // (`extensions/rxregexp/rxregexp.cpp:88`).
    Some(unsafe { CStr::from_ptr(name) }.to_bytes())
}

extern "C" fn set_object_variable(
    context: *mut RexxMethodContext_,
    name: CSTRING,
    value: RexxObjectPtr,
) {
    // SAFETY: this is reached only through `METHOD_CONTEXT`, which only a
    // `Contexts` publishes, and only for the length of the call it made; the
    // name is the extension's own string.
    let (activation, Some(name)) = (unsafe { activation_of(context) }, unsafe { name_of(name) })
    else {
        return;
    };
    activation.set_object_variable(name, value);
}

extern "C" fn drop_object_variable(context: *mut RexxMethodContext_, name: CSTRING) {
    // SAFETY: as `set_object_variable`.
    let (activation, Some(name)) = (unsafe { activation_of(context) }, unsafe { name_of(name) })
    else {
        return;
    };
    activation.drop_object_variable(name);
}

extern "C" fn whole_number_to_object(
    context: *mut RexxThreadContext_,
    value: wholenumber_t,
) -> RexxObjectPtr {
    // SAFETY: this is reached only through the thread table a `Contexts`
    // owns, and only for the length of the call it made.
    unsafe { activation_of(context) }.whole_number(value)
}

extern "C" fn string_data(context: *mut RexxThreadContext_, string: RexxStringObject) -> CSTRING {
    // SAFETY: as `whole_number_to_object`.
    unsafe { activation_of(context) }.string_data(string.cast())
}

extern "C" fn string_length(context: *mut RexxThreadContext_, string: RexxStringObject) -> usize {
    // SAFETY: as `whole_number_to_object`.
    unsafe { activation_of(context) }.string_length(string.cast())
}

extern "C" fn new_pointer(context: *mut RexxThreadContext_, value: POINTER) -> RexxPointerObject {
    // SAFETY: as `whole_number_to_object`.
    unsafe { activation_of(context) }.new_pointer(value).cast()
}

extern "C" fn raise_exception0(context: *mut RexxThreadContext_, number: usize) {
    // SAFETY: as `whole_number_to_object`.
    unsafe { activation_of(context) }.raise(number);
}

/// What a stub read through the context, which is the channel
/// `argumentExists` uses (`api/oorexxapi.h:4276`).
#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Seen {
    /// Whether the context published the array the stub was handed.
    pub same_array: bool,
    /// The `flags` word of the argument at index one, read through the
    /// published array; zero where the context published another array.
    pub flags: u16,
}

#[cfg(test)]
const SAW_NOTHING: Seen = Seen {
    same_array: false,
    flags: 0,
};

#[cfg(test)]
thread_local! {
    static SEEN: std::cell::Cell<Seen> = const { std::cell::Cell::new(SAW_NOTHING) };
}

/// What [`reading_stub`] read on its most recent call.
#[cfg(test)]
pub(crate) fn seen() -> Seen {
    SEEN.get()
}

#[cfg(test)]
pub(crate) fn forget_seen() {
    SEEN.set(SAW_NOTHING);
}

/// The signature [`reading_stub`] publishes: an `int` result and one
/// `CSTRING` parameter.
#[cfg(test)]
static READING_TYPES: [u16; 3] = [
    crate::values::code::INT,
    crate::values::code::CSTRING,
    crate::values::ARGUMENT_TERMINATOR,
];

/// A method stub that reads its argument through the context rather than
/// through the parameter, as an extension using `argumentExists` does.
#[cfg(test)]
pub(crate) extern "C" fn reading_stub(
    context: *mut crate::layout::RexxMethodContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    if arguments.is_null() {
        return READING_TYPES.as_ptr().cast_mut();
    }
    // SAFETY: `context` is the one `invoke::method` handed this stub, which
    // holds it as a `&mut RexxMethodContext_` across the call, so the field
    // read is in bounds.
    let published = unsafe { (*context).arguments };
    let same_array = published == arguments;
    let flags = if same_array {
        // SAFETY: the published array is the one this stub was handed, which
        // is `invoke::method`'s own array of `MAX_NATIVE_ARGUMENTS`
        // descriptors, so the element after the return value is inside it.
        // Where the two differ nothing is read.
        unsafe { (*published.add(1)).flags }
    } else {
        0
    };
    SEEN.set(Seen { same_array, flags });
    std::ptr::null_mut()
}

#[cfg(test)]
mod tests {
    use super::{owner_of, value_of};
    use crate::layout::{Owned, RexxCallContext_, RexxMethodContext_, RexxThreadContext_};
    use crate::values::{Converted, Repr, Value, code, descriptor, repr, rows};

    /// A value of each shape, chosen so that a read of the wrong width or the
    /// wrong member answers something else.
    fn sample(repr: Repr) -> Value {
        let address: *mut std::ffi::c_void =
            std::ptr::without_provenance_mut(0x0123_4567_89ab_cdef);
        match repr {
            Repr::Object => Value::Object(address.cast()),
            Repr::CString => Value::CString(address.cast()),
            Repr::Pointer => Value::Pointer(address),
            Repr::Int => Value::Int(-0x1234_5678),
            Repr::Int8 => Value::Int8(-0x12),
            Repr::Int16 => Value::Int16(-0x1234),
            Repr::Int32 => Value::Int32(-0x1234_5678),
            Repr::Int64 => Value::Int64(-0x0123_4567_89ab_cdef),
            Repr::Uint8 => Value::Uint8(0xfe),
            Repr::Uint16 => Value::Uint16(0xfedc),
            Repr::Uint32 => Value::Uint32(0xfedc_ba98),
            Repr::Uint64 => Value::Uint64(0xfedc_ba98_7654_3210),
            Repr::Isize => Value::Isize(-0x0123_4567_89ab_cdef),
            Repr::Usize => Value::Usize(0xfedc_ba98_7654_3210),
            Repr::Double => Value::Double(-1.5e300),
            Repr::Float => Value::Float(-1.5e30),
        }
    }

    /// Whatever `descriptor` writes for a value, `value_of` reads back
    /// through the row's own [`Repr`], so the two halves of the union agree.
    #[test]
    fn every_repr_reads_back_the_member_the_table_wrote() {
        for (code, name) in rows() {
            let repr = repr(code).expect("every row names a member");
            let value = sample(repr);
            let written = descriptor(code, Converted { value, flags: 0 });
            // SAFETY: `descriptor` writes the whole word.
            let read = unsafe { value_of(&written, repr) };
            assert_eq!(
                read, value,
                "REXX_VALUE_{name} does not read back what it wrote"
            );
        }
    }

    /// A narrow member is written over a zeroed word, so the wider read sees
    /// the member's bytes and zeros above them.
    #[test]
    fn a_narrow_value_is_written_over_a_zeroed_word() {
        let written = descriptor(
            code::INT64_T,
            Converted {
                value: Value::Uint8(0xfe),
                flags: 0,
            },
        );
        // SAFETY: `descriptor` writes the whole word.
        let read = unsafe { value_of(&written, Repr::Uint64) };
        assert_eq!(read, Value::Uint64(0xfe));
    }

    /// Stands in for the interpreter state a later task puts behind a context.
    #[derive(Debug, PartialEq, Eq)]
    struct Activation(u32);

    #[test]
    fn a_context_we_handed_out_recovers_its_owner() {
        let mut owner = Activation(0x5eed);
        let mut wrapper = Owned {
            context: RexxMethodContext_ {
                threadContext: std::ptr::null_mut(),
                functions: std::ptr::null_mut(),
                arguments: std::ptr::null_mut(),
            },
            owner: &raw mut owner,
        };
        let handed_out = &raw mut wrapper.context;
        // SAFETY: `handed_out` is the `context` field of a live
        // `Owned<RexxMethodContext_, Activation>`, which is exactly what
        // `owner_of` asks of its caller.
        let recovered: *mut Activation = unsafe { owner_of(handed_out) };
        assert_eq!(recovered, &raw mut owner);
        // SAFETY: `recovered` is `&raw mut owner`, and `owner` outlives it.
        assert_eq!(unsafe { &*recovered }, &Activation(0x5eed));
    }

    /// The cast is the identity on the address, which is the property that
    /// lets an extension hold the public struct and nothing else.
    #[test]
    fn the_public_struct_is_at_the_head_of_the_wrapper() {
        let mut thread = Owned::<RexxThreadContext_, u8> {
            context: RexxThreadContext_ {
                instance: std::ptr::null_mut(),
                functions: std::ptr::null_mut(),
            },
            owner: std::ptr::null_mut(),
        };
        assert_eq!(
            (&raw mut thread).cast::<u8>(),
            (&raw mut thread.context).cast::<u8>()
        );

        let mut call = Owned::<RexxCallContext_, u8> {
            context: RexxCallContext_ {
                threadContext: std::ptr::null_mut(),
                functions: std::ptr::null_mut(),
                arguments: std::ptr::null_mut(),
            },
            owner: std::ptr::null_mut(),
        };
        assert_eq!(
            (&raw mut call).cast::<u8>(),
            (&raw mut call.context).cast::<u8>()
        );
    }
}
