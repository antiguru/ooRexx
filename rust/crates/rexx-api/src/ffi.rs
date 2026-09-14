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

use crate::layout::{Owned, ValueDescriptor};
use crate::values::{Repr, Value};

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
/// `repr` is the conversion table's answer for the declared type
/// (`values::repr`), and the stub writes the member that same type names
/// (`api/oorexxapi.h:4282`), so the two agree by construction.
pub fn value_of(descriptor: &ValueDescriptor, repr: Repr) -> Value {
    // SAFETY: the member read is the one the declared type names, and it is
    // the member that was written: the stub writes through that same type
    // (`api/oorexxapi.h:4282`) and the caller derives `repr` from it through
    // the table, whose rows the frozen header establishes. Every member is an
    // integer, a float or a pointer, so once written it holds a valid value
    // of its own type.
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

#[cfg(test)]
mod tests {
    use super::owner_of;
    use crate::layout::{Owned, RexxCallContext_, RexxMethodContext_, RexxThreadContext_};

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
