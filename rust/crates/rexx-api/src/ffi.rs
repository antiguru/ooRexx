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
    CSTRING, CallContextInterface, MethodContextInterface, Owned, POINTER, RexxCallContext_,
    RexxInstance_, RexxInstanceInterface, RexxMethodContext_, RexxObjectPtr, RexxPointerObject,
    RexxStringObject, RexxThreadContext_, RexxThreadInterface, ValueDescriptor, logical_t,
    stringsize_t, wholenumber_t,
};
use crate::values::{Activation, Repr, Value};

/// The state that owns the context `context` addresses.
///
/// `context` must be the public struct at the head of an [`Owned`] we handed
/// out, which is how the C++ recovers the owning activation
/// (`interpreter/concurrency/Activity.hpp:503`).
///
/// # Safety
/// `context` is a pointer to a live `Owned<C, T>` cast to its `context`
/// field, derived from the whole wrapper rather than from that field, so that
/// its provenance covers `owner`. Nothing about the pointer witnesses that:
/// an extension holds it as an opaque address and can hand back any value at
/// all, so the guarantee is the interpreter's, which mints every context it
/// hands out ([`Contexts::method`]) and never hands out one it did not build.
pub unsafe fn owner_of<C, T>(context: *mut C) -> *mut T {
    let owned = context.cast::<Owned<C, T>>();
    // SAFETY: the caller guarantees `owned` was derived from a live
    // `Owned<C, T>` as a whole, so its provenance covers the `owner` field,
    // and `context` is first by value, so the cast is the identity on the
    // address.
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
///
/// A member trusts the context it is handed, so calling one is `unsafe`:
///
/// ```compile_fail,E0133
/// # use rexx_api::ffi::METHOD_CONTEXT;
/// # use rexx_api::layout::RexxMethodContext_;
/// let mut bare = RexxMethodContext_ {
///     threadContext: std::ptr::null_mut(),
///     functions: std::ptr::null_mut(),
///     arguments: std::ptr::null_mut(),
/// };
/// (METHOD_CONTEXT.SetObjectVariable)(&raw mut bare, c"X".as_ptr(), std::ptr::null_mut());
/// ```
pub static METHOD_CONTEXT: MethodContextInterface = {
    let mut table = MethodContextInterface::REFUSING;
    table.SetObjectVariable = set_object_variable;
    table.DropObjectVariable = drop_object_variable;
    table
};

/// The call-context table a routine's stub is handed
/// (`Activity::callContextFunctions`,
/// `interpreter/api/CallContextStubs.cpp:678`), at one address for the process
/// as [`METHOD_CONTEXT`] is.
pub static CALL_CONTEXT: CallContextInterface = {
    let mut table = CallContextInterface::REFUSING;
    table.GetContextDigits = get_context_digits;
    table.GetContextFuzz = get_context_fuzz;
    table.GetContextForm = get_context_form;
    table
};

/// The instance table a thread context's `instance` addresses
/// (`InterpreterInstance::interfaceVector`,
/// `interpreter/api/InterpreterInstanceStubs.cpp:105`), at one address for the
/// process as [`METHOD_CONTEXT`] is.
pub static INSTANCE: RexxInstanceInterface = {
    let mut table = RexxInstanceInterface::REFUSING;
    table.InterpreterVersion = interpreter_version;
    table.LanguageLevel = language_level;
    table
};

/// The method context a stub is handed: a pointer to the public struct at the
/// head of an `Owned` wrapper, derived from the whole wrapper, and borrowed
/// from that wrapper for as long as it is used.
pub struct MethodContext<'a> {
    pointer: *mut RexxMethodContext_,
    wrapper: PhantomData<&'a mut RexxMethodContext_>,
}

impl MethodContext<'_> {
    /// The address the stub is handed.
    pub(crate) fn as_ptr(&self) -> *mut RexxMethodContext_ {
        self.pointer
    }

    /// A context over a struct no `Contexts` built, for a test whose tables
    /// never recover an owner.
    #[cfg(test)]
    pub(crate) fn bare(context: &mut RexxMethodContext_) -> MethodContext<'_> {
        MethodContext {
            pointer: &raw mut *context,
            wrapper: PhantomData,
        }
    }
}

/// The call context a routine's stub is handed, shaped as [`MethodContext`].
pub struct CallContext<'a> {
    pointer: *mut RexxCallContext_,
    wrapper: PhantomData<&'a mut RexxCallContext_>,
}

impl CallContext<'_> {
    /// The address the stub is handed.
    pub(crate) fn as_ptr(&self) -> *mut RexxCallContext_ {
        self.pointer
    }

    /// A context over a struct no `Contexts` built, for a test whose tables
    /// never recover an owner.
    #[cfg(test)]
    pub(crate) fn bare(context: &mut RexxCallContext_) -> CallContext<'_> {
        CallContext {
            pointer: &raw mut *context,
            wrapper: PhantomData,
        }
    }
}

/// The contexts one native call hands an extension, wired to the state behind
/// them.
///
/// The thread table is owned here and not shared, because its object members
/// are handles this activation minted
/// (`interpreter/concurrency/Activity.cpp:1846-1849`).
pub struct Contexts<'a, 'h> {
    instance: Owned<RexxInstance_, Activation<'h>>,
    thread: Owned<RexxThreadContext_, Activation<'h>>,
    method: Owned<RexxMethodContext_, Activation<'h>>,
    call: Owned<RexxCallContext_, Activation<'h>>,
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
        table.DoubleToObjectWithPrecision = double_to_object_with_precision;
        table.RaiseException0 = raise_exception0;
        table.RexxNil = constants.nil;
        table.RexxTrue = constants.true_object;
        table.RexxFalse = constants.false_object;
        table.RexxNullString = constants.null_string.cast();
        Contexts {
            instance: Owned {
                context: RexxInstance_ {
                    functions: std::ptr::null_mut(),
                    applicationData: std::ptr::null_mut(),
                },
                owner,
            },
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
            call: Owned {
                context: RexxCallContext_ {
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
    /// Both context pointers are taken from their whole wrappers, which is
    /// what lets [`owner_of`] read the `owner` beside the public struct.
    pub fn method(&mut self) -> MethodContext<'_> {
        self.link_thread();
        self.method.context.threadContext = (&raw mut self.thread).cast::<RexxThreadContext_>();
        self.method.context.functions = std::ptr::from_ref(&METHOD_CONTEXT).cast_mut();
        MethodContext {
            pointer: (&raw mut self.method).cast::<RexxMethodContext_>(),
            wrapper: PhantomData,
        }
    }

    /// The call context, linked as [`Contexts::method`] links the method
    /// context.
    pub fn call(&mut self) -> CallContext<'_> {
        self.link_thread();
        self.call.context.threadContext = (&raw mut self.thread).cast::<RexxThreadContext_>();
        self.call.context.functions = std::ptr::from_ref(&CALL_CONTEXT).cast_mut();
        CallContext {
            pointer: (&raw mut self.call).cast::<RexxCallContext_>(),
            wrapper: PhantomData,
        }
    }

    /// Points the thread context at its table and at this wrapper's instance,
    /// and the instance at its table.
    fn link_thread(&mut self) {
        self.instance.context.functions = std::ptr::from_ref(&INSTANCE).cast_mut();
        self.thread.context.instance = (&raw mut self.instance).cast::<RexxInstance_>();
        self.thread.context.functions = &raw mut self.table;
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

/// # Safety
/// `context` is a method context a live [`Contexts`] handed out, used during
/// the call it was handed to, and a non-null `name` is a NUL-terminated string.
unsafe extern "C" fn set_object_variable(
    context: *mut RexxMethodContext_,
    name: CSTRING,
    value: RexxObjectPtr,
) {
    // SAFETY: the caller guarantees the context and the name, and
    // `invoke::method` holds no conversion state across the call the context
    // was handed to.
    let (activation, Some(name)) = (unsafe { activation_of(context) }, unsafe { name_of(name) })
    else {
        return;
    };
    activation.set_object_variable(name, value);
}

/// # Safety
/// As [`set_object_variable`].
unsafe extern "C" fn drop_object_variable(context: *mut RexxMethodContext_, name: CSTRING) {
    // SAFETY: as `set_object_variable`.
    let (activation, Some(name)) = (unsafe { activation_of(context) }, unsafe { name_of(name) })
    else {
        return;
    };
    activation.drop_object_variable(name);
}

/// # Safety
/// `context` is the thread context of a live [`Contexts`], used during the
/// call its method context was handed to.
unsafe extern "C" fn whole_number_to_object(
    context: *mut RexxThreadContext_,
    value: wholenumber_t,
) -> RexxObjectPtr {
    // SAFETY: the caller guarantees the context, and `invoke::method` holds no
    // conversion state across the call it was handed to.
    unsafe { activation_of(context) }.whole_number(value)
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn string_data(
    context: *mut RexxThreadContext_,
    string: RexxStringObject,
) -> CSTRING {
    // SAFETY: as `whole_number_to_object`.
    unsafe { activation_of(context) }.string_data(string.cast())
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn string_length(
    context: *mut RexxThreadContext_,
    string: RexxStringObject,
) -> usize {
    // SAFETY: as `whole_number_to_object`.
    unsafe { activation_of(context) }.string_length(string.cast())
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn new_pointer(
    context: *mut RexxThreadContext_,
    value: POINTER,
) -> RexxPointerObject {
    // SAFETY: as `whole_number_to_object`.
    unsafe { activation_of(context) }.new_pointer(value).cast()
}

/// `InterpreterVersion` (`interpreter/api/InterpreterInstanceStubs.cpp:79`),
/// which reads nothing through the instance.
extern "C" fn interpreter_version(_instance: *mut RexxInstance_) -> usize {
    usize::try_from(crate::load::CURRENT_INTERPRETER_VERSION)
        .expect("the interpreter version is positive")
}

/// `LanguageLevel` (`interpreter/api/InterpreterInstanceStubs.cpp:84`), which
/// reads nothing through the instance.
extern "C" fn language_level(_instance: *mut RexxInstance_) -> usize {
    crate::load::CURRENT_LANGUAGE_LEVEL
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn double_to_object_with_precision(
    context: *mut RexxThreadContext_,
    value: f64,
    precision: usize,
) -> RexxObjectPtr {
    // SAFETY: as `whole_number_to_object`.
    unsafe { activation_of(context) }.double_object(value, precision)
}

/// # Safety
/// `context` is a call context a live [`Contexts`] handed out, used during the
/// call it was handed to.
unsafe extern "C" fn get_context_digits(context: *mut RexxCallContext_) -> stringsize_t {
    // SAFETY: the caller guarantees the context, and `invoke::routine` holds no
    // conversion state across the call it was handed to.
    unsafe { activation_of(context) }.numeric().digits
}

/// # Safety
/// As [`get_context_digits`].
unsafe extern "C" fn get_context_fuzz(context: *mut RexxCallContext_) -> stringsize_t {
    // SAFETY: as `get_context_digits`.
    unsafe { activation_of(context) }.numeric().fuzz
}

/// # Safety
/// As [`get_context_digits`].
unsafe extern "C" fn get_context_form(context: *mut RexxCallContext_) -> logical_t {
    // SAFETY: as `get_context_digits`.
    logical_t::from(unsafe { activation_of(context) }.numeric().engineering)
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn raise_exception0(context: *mut RexxThreadContext_, number: usize) {
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
    // holds the `MethodContext` it came from across the call, so the field
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

/// The signature [`dropping_stub`] publishes: an `int` result and nothing
/// else.
#[cfg(test)]
static DROPPING_TYPES: [u16; 2] = [crate::values::code::INT, crate::values::ARGUMENT_TERMINATOR];

/// A method stub that drops `CSELF` through the context it was handed, which
/// is the call that recovers the activation with [`owner_of`].
#[cfg(test)]
pub(crate) extern "C" fn dropping_stub(
    context: *mut crate::layout::RexxMethodContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    if arguments.is_null() {
        return DROPPING_TYPES.as_ptr().cast_mut();
    }
    // SAFETY: `context` is the one `invoke::method` handed this stub, whose
    // `functions` a `Contexts` wrote, so the table read is in bounds.
    let functions = unsafe { (*context).functions };
    // SAFETY: as above, the table is `METHOD_CONTEXT`.
    let drop_variable = unsafe { (*functions).DropObjectVariable };
    // SAFETY: `context` is the one a `Contexts` handed this call, and the
    // name is a literal.
    unsafe { drop_variable(context, c"CSELF".as_ptr()) };
    std::ptr::null_mut()
}

/// The signature [`thread_table_stub`] publishes: an `int` result and one
/// `RexxStringObject` parameter.
#[cfg(test)]
static THREAD_TABLE_TYPES: [u16; 3] = [
    crate::values::code::INT,
    crate::values::code::REXX_STRING_OBJECT,
    crate::values::ARGUMENT_TERMINATOR,
];

/// The address [`thread_table_stub`] hands `NewPointer`.
#[cfg(test)]
pub(crate) const STUB_POINTER: usize = 0x5eed_0000;

/// The condition number [`thread_table_stub`] hands `RaiseException0`.
#[cfg(test)]
pub(crate) const STUB_CONDITION: usize = 40_001;

/// A method stub that calls `StringLength`, `StringData`,
/// `WholeNumberToObject`, `NewPointer` and `RaiseException0` through the
/// thread context its method context links, and stores what they answered in
/// object variables through the method table: `LENGTH` and `FIRST` for its
/// argument's length and first byte, and `POINTER` for [`STUB_POINTER`].
#[cfg(test)]
pub(crate) extern "C" fn thread_table_stub(
    context: *mut crate::layout::RexxMethodContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    if arguments.is_null() {
        return THREAD_TABLE_TYPES.as_ptr().cast_mut();
    }
    // SAFETY: `context` is the one a `Contexts` handed this call, which
    // linked its thread context and wrote both tables; element one of the
    // array is this call's argument, whose word `values::descriptor` wrote.
    let (thread, method, string) = unsafe {
        (
            (*context).threadContext,
            &*(*context).functions,
            (*arguments.add(1)).value.value_RexxStringObject,
        )
    };
    // SAFETY: as above.
    let table = unsafe { &*(*thread).functions };
    // SAFETY: every call passes the contexts this call was handed, a literal
    // name, and a handle this call was given or the thread table answered;
    // `StringData`'s bytes are read only where it answered some.
    unsafe {
        let length = (table.StringLength)(thread, string);
        let data = (table.StringData)(thread, string);
        let first = if data.is_null() || length == 0 {
            0
        } else {
            isize::from(*data.cast::<u8>())
        };
        let length = isize::try_from(length).unwrap_or(-1);
        let length = (table.WholeNumberToObject)(thread, length);
        (method.SetObjectVariable)(context, c"LENGTH".as_ptr(), length);
        let first = (table.WholeNumberToObject)(thread, first);
        (method.SetObjectVariable)(context, c"FIRST".as_ptr(), first);
        let pointer = (table.NewPointer)(thread, std::ptr::without_provenance_mut(STUB_POINTER));
        (method.SetObjectVariable)(context, c"POINTER".as_ptr(), pointer.cast());
        (table.RaiseException0)(thread, STUB_CONDITION);
    }
    std::ptr::null_mut()
}

/// A method stub that reads `InterpreterVersion` and `LanguageLevel` through
/// the instance its thread context links, and stores them in the object
/// variables `VERSION` and `LEVEL`.
#[cfg(test)]
pub(crate) extern "C" fn instance_stub(
    context: *mut crate::layout::RexxMethodContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    if arguments.is_null() {
        return DROPPING_TYPES.as_ptr().cast_mut();
    }
    // SAFETY: `context` is the one a `Contexts` handed this call, which
    // linked its thread context, that context's instance, and all three
    // tables.
    unsafe {
        let method = &*(*context).functions;
        let thread = (*context).threadContext;
        let table = &*(*thread).functions;
        let instance = (*thread).instance;
        let instance_table = &*(*instance).functions;
        let version = (instance_table.InterpreterVersion)(instance);
        let level = (instance_table.LanguageLevel)(instance);
        let version = (table.WholeNumberToObject)(thread, isize::try_from(version).unwrap_or(-1));
        (method.SetObjectVariable)(context, c"VERSION".as_ptr(), version);
        let level = (table.WholeNumberToObject)(thread, isize::try_from(level).unwrap_or(-1));
        (method.SetObjectVariable)(context, c"LEVEL".as_ptr(), level);
    }
    std::ptr::null_mut()
}

/// A method stub that reaches `HaltThread` through its thread context, which
/// nothing fills.
#[cfg(test)]
pub(crate) extern "C" fn refusing_stub(
    context: *mut crate::layout::RexxMethodContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    if arguments.is_null() {
        return DROPPING_TYPES.as_ptr().cast_mut();
    }
    // SAFETY: `context` is the one a `Contexts` handed this call, which linked
    // its thread context and that context's table.
    unsafe {
        let thread = (*context).threadContext;
        ((*(*thread).functions).HaltThread)(thread);
    }
    std::ptr::null_mut()
}

/// A method stub that reaches `NewStringFromAsciiz`, which nothing fills, then
/// `WholeNumberToObject`, which the host a nesting test builds answers by
/// running a native call of its own, then raises [`STUB_CONDITION`].
#[cfg(test)]
pub(crate) extern "C" fn nesting_stub(
    context: *mut crate::layout::RexxMethodContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    if arguments.is_null() {
        return DROPPING_TYPES.as_ptr().cast_mut();
    }
    // SAFETY: as `refusing_stub`; the name is a literal.
    unsafe {
        let thread = (*context).threadContext;
        let table = &*(*thread).functions;
        (table.NewStringFromAsciiz)(thread, c"units".as_ptr());
        (table.WholeNumberToObject)(thread, 3);
        (table.RaiseException0)(thread, STUB_CONDITION);
    }
    std::ptr::null_mut()
}

#[cfg(test)]
thread_local! {
    /// What [`numeric_stub`] read through its call context: digits, fuzz and
    /// form, in that order.
    static NUMERIC_SEEN: std::cell::Cell<Option<(usize, usize, usize)>> =
        const { std::cell::Cell::new(None) };
}

/// What [`numeric_stub`] read on its most recent call.
#[cfg(test)]
pub(crate) fn numeric_seen() -> Option<(usize, usize, usize)> {
    NUMERIC_SEEN.get()
}

/// The value [`numeric_stub`] hands `DoubleToObjectWithPrecision`.
#[cfg(test)]
pub(crate) const STUB_DOUBLE: f64 = 1.5;

/// The signature [`numeric_stub`] publishes: a `RexxObjectPtr` result and no
/// parameter.
#[cfg(test)]
static NUMERIC_TYPES: [u16; 2] = [
    crate::values::code::REXX_OBJECT_PTR,
    crate::values::ARGUMENT_TERMINATOR,
];

/// A routine stub that reads `GetContextDigits`, `GetContextFuzz` and
/// `GetContextForm` through the call-context table, and answers the object
/// `DoubleToObjectWithPrecision` builds for [`STUB_DOUBLE`] at the digits it
/// read, which is what `rxmath`'s `NumericFormatter` does
/// (`extensions/rxmath/rxmath.cpp:118-140`).
#[cfg(test)]
pub(crate) extern "C" fn numeric_stub(
    context: *mut RexxCallContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    if arguments.is_null() {
        return NUMERIC_TYPES.as_ptr().cast_mut();
    }
    // SAFETY: `context` is the one a `Contexts` handed this call, which
    // linked its thread context and wrote both tables, and element zero is
    // this call's result descriptor.
    unsafe {
        let calls = &*(*context).functions;
        let thread = (*context).threadContext;
        let table = &*(*thread).functions;
        let digits = (calls.GetContextDigits)(context);
        let fuzz = (calls.GetContextFuzz)(context);
        let form = (calls.GetContextForm)(context);
        NUMERIC_SEEN.set(Some((digits, fuzz, form)));
        let object = (table.DoubleToObjectWithPrecision)(thread, STUB_DOUBLE, digits);
        (*arguments).value.value_RexxObjectPtr = object;
    }
    std::ptr::null_mut()
}

#[cfg(test)]
mod tests {
    use super::{owner_of, value_of};
    use crate::layout::{
        Owned, RexxCallContext_, RexxMethodContext_, RexxThreadContext_, RexxThreadInterface,
        recording_refusals,
    };
    use crate::values::{Converted, Repr, Value, code, descriptor, repr, rows};

    /// The variable this test sets on the child it spawns, so that the child
    /// reaches the stub and this process does not.
    const CALL_A_STUB: &str = "REXX_API_CALL_AN_ABORTING_ENTRY";

    /// A member marked `aborts` is reached the way an extension would reach
    /// it, through the table, and ends the process naming itself.
    ///
    /// The call is in a child because an `extern "C"` frame aborts on panic, so
    /// there is no returning from it.
    #[test]
    #[cfg_attr(miri, ignore = "spawns a process")]
    fn an_aborting_entry_refuses_loudly() {
        if std::env::var_os(CALL_A_STUB).is_some() {
            // SAFETY: the stub reads none of its arguments.
            unsafe {
                (RexxThreadInterface::REFUSING.BufferData)(
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            };
            unreachable!("the stub returned");
        }

        let binary = std::env::current_exe().expect("this test binary's own path");
        let output = std::process::Command::new(binary)
            .args([
                "ffi::tests::an_aborting_entry_refuses_loudly",
                "--exact",
                "--nocapture",
            ])
            .env(CALL_A_STUB, "1")
            .output()
            .expect("the child runs");

        assert!(
            !output.status.success(),
            "the child returned from an aborting entry: {:?}",
            output.status
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("RexxThreadInterface.BufferData is not implemented (Phase 8)"),
            "the refusal did not name the entry and the phase that owes it:\n{stderr}"
        );
    }

    /// Every other unbuilt member records the first of itself the call reached
    /// and returns the oracle's failure value, whatever context it was handed,
    /// and the record in force before is back afterwards.
    #[test]
    fn a_refusing_entry_records_itself_and_returns() {
        let table = RexxThreadInterface::REFUSING;
        let thread = std::ptr::null_mut();
        let ((), outer) = recording_refusals(|| {
            let (answers, inner) = recording_refusals(|| {
                // SAFETY: none of these stubs reads its arguments.
                unsafe {
                    (table.HaltThread)(thread);
                    (
                        (table.NewStringFromAsciiz)(thread, std::ptr::null()),
                        (table.IsString)(thread, std::ptr::null_mut()),
                        (table.DisplayCondition)(thread),
                    )
                }
            });
            assert_eq!(answers, (std::ptr::null_mut(), 0, 49));
            assert_eq!(inner, Some("RexxThreadInterface.HaltThread"));
        });
        assert_eq!(outer, None, "the inner record leaked into the outer one");
    }

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
        let handed_out = (&raw mut wrapper).cast::<RexxMethodContext_>();
        // SAFETY: `handed_out` is a live `Owned<RexxMethodContext_, Activation>`
        // taken as a whole and cast to its `context` field, which is exactly
        // what `owner_of` asks of its caller.
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
