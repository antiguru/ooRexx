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

use std::cell::Cell;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::rc::Rc;

use crate::layout::{
    CSTRING, CallContextInterface, MethodContextInterface, Owned, POINTER, RexxArrayObject,
    RexxBufferObject, RexxBufferStringObject, RexxCallContext_, RexxCondition, RexxDirectoryObject,
    RexxInstance_, RexxInstanceInterface, RexxMethodContext_, RexxMutableBufferObject,
    RexxObjectPtr, RexxPackageEntry, RexxPointerObject, RexxStringObject, RexxThreadContext_,
    RexxThreadInterface, ValueDescriptor, logical_t, stringsize_t, wholenumber_t,
};
use crate::values::{Activation, Converted, MAX_WHOLENUMBER, Repr, Value};

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
/// hands out ([`Contexts::method`], [`ThreadContext::new`]) and never hands out
/// one it did not build.
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
    table.ThrowException0 = throw_exception0::<RexxMethodContext_>;
    table.ThrowException1 = throw_exception1::<RexxMethodContext_>;
    table.ThrowException2 = throw_exception2::<RexxMethodContext_>;
    table.ThrowException = throw_exception::<RexxMethodContext_>;
    table.ThrowCondition = throw_condition::<RexxMethodContext_>;
    table.SetObjectVariable = set_object_variable;
    table.DropObjectVariable = drop_object_variable;
    table.GetCSelf = get_cself;
    table.GetObjectVariable = variables::get_object_variable;
    table.GetObjectVariableReference = variables::get_object_variable_reference;
    table.SetGuardOn = variables::set_guard;
    table.SetGuardOff = variables::set_guard;
    table.GetArguments = messages::method_arguments;
    table.GetArgument = messages::method_argument;
    table.GetMessageName = messages::message_name;
    table.GetMethod = messages::current_method;
    table.GetSelf = messages::get_self;
    table.GetSuper = messages::get_super;
    table.GetScope = messages::get_scope;
    table.ForwardMessage = messages::forward_message;
    table.FindContextClass = messages::method_find_context_class;
    table.AllocateObjectMemory = allocate_object_memory;
    table.FreeObjectMemory = free_object_memory;
    table.ReallocateObjectMemory = reallocate_object_memory;
    table
};

/// The call-context table a routine's stub is handed
/// (`Activity::callContextFunctions`,
/// `interpreter/api/CallContextStubs.cpp:678`), at one address for the process
/// as [`METHOD_CONTEXT`] is.
pub static CALL_CONTEXT: CallContextInterface = {
    let mut table = CallContextInterface::REFUSING;
    table.ThrowException0 = throw_exception0::<RexxCallContext_>;
    table.ThrowException1 = throw_exception1::<RexxCallContext_>;
    table.ThrowException2 = throw_exception2::<RexxCallContext_>;
    table.ThrowException = throw_exception::<RexxCallContext_>;
    table.ThrowCondition = throw_condition::<RexxCallContext_>;
    table.GetContextDigits = get_context_digits;
    table.GetContextFuzz = get_context_fuzz;
    table.GetContextForm = get_context_form;
    table.GetContextVariable = variables::get_context_variable;
    table.SetContextVariable = variables::set_context_variable;
    table.DropContextVariable = variables::drop_context_variable;
    table.GetAllContextVariables = variables::get_all_context_variables;
    table.ResolveStemVariable = variables::resolve_stem_variable;
    table.GetContextVariableReference = variables::get_context_variable_reference;
    table.GetArguments = messages::call_arguments;
    table.GetArgument = messages::call_argument;
    table.GetRoutineName = messages::routine_name;
    table.GetRoutine = messages::current_routine;
    table.GetCallerContext = messages::caller_context;
    table.FindContextClass = messages::call_find_context_class;
    table.InvalidRoutine = messages::invalid_routine;
    table
};

/// The instance table a thread context's `instance` addresses
/// (`InterpreterInstance::interfaceVector`,
/// `interpreter/api/InterpreterInstanceStubs.cpp:106`), at one address for the
/// process as [`METHOD_CONTEXT`] is.
pub static INSTANCE: RexxInstanceInterface = {
    let mut table = RexxInstanceInterface::REFUSING;
    table.InterpreterVersion = interpreter_version;
    table.LanguageLevel = language_level;
    table.AttachThread = attach_thread;
    table
};

/// The thread table each [`ThreadContext`] copies
/// (`Activity::threadContextFunctions`,
/// `interpreter/api/ThreadContextStubs.cpp:2098`), its data members null until
/// the first [`ThreadContext::enter`] writes them.
pub const THREAD: RexxThreadInterface = {
    let mut table = RexxThreadInterface::REFUSING;
    table.WholeNumberToObject = whole_number_to_object;
    table.StringData = string_data;
    table.StringLength = string_length;
    table.NewPointer = new_pointer;
    table.DoubleToObjectWithPrecision = double_to_object_with_precision;
    table.RaiseException0 = raise_exception0;
    table.UintptrToObject = uintptr_to_object;
    table.IntptrToObject = intptr_to_object;
    table.StringSizeToObject = string_size_to_object;
    table.Int64ToObject = int64_to_object;
    table.UnsignedInt64ToObject = unsigned_int64_to_object;
    table.Int32ToObject = int32_to_object;
    table.UnsignedInt32ToObject = unsigned_int32_to_object;
    table.ObjectToWholeNumber = object_to_whole_number;
    table.ObjectToStringSize = object_to_string_size;
    table.ObjectToInt64 = object_to_int64;
    table.ObjectToUnsignedInt64 = object_to_unsigned_int64;
    table.ObjectToInt32 = object_to_int32;
    table.ObjectToUnsignedInt32 = object_to_unsigned_int32;
    table.ObjectToUintptr = object_to_uintptr;
    table.ObjectToIntptr = object_to_intptr;
    table.ObjectToLogical = object_to_logical;
    table.LogicalToObject = logical_to_object;
    table.DoubleToObject = double_to_object;
    table.ObjectToDouble = object_to_double;
    table.NewString = new_string;
    table.NewStringFromAsciiz = new_string_from_asciiz;
    table.ValueToObject = value_to_object;
    table.ValuesToObject = values_to_object;
    table.ObjectToValue = object_to_value;
    table.RaiseException1 = raise_exception1;
    table.RaiseException2 = raise_exception2;
    table.RaiseException = raise_exception;
    table.RaiseCondition = raise_condition;
    table.CheckCondition = check_condition;
    table.ClearCondition = clear_condition;
    table.GetConditionInfo = get_condition_info;
    table.DisplayCondition = display_condition;
    table.DecodeConditionInfo = decode_condition_info;
    table.ObjectToString = object_to_string;
    table.ObjectToStringValue = object_to_string_value;
    table.StringGet = string_get;
    table.StringUpper = string_upper;
    table.StringLower = string_lower;
    table.IsString = is_string;
    table.NewBufferString = new_buffer_string;
    table.BufferStringLength = buffer_string_length;
    table.BufferStringData = buffer_string_data;
    table.FinishBufferString = finish_buffer_string;
    table.NewBuffer = new_buffer;
    table.BufferData = buffer_data;
    table.BufferLength = buffer_length;
    table.IsBuffer = is_buffer;
    table.PointerValue = pointer_value;
    table.IsPointer = is_pointer;
    table.NewMutableBuffer = new_mutable_buffer;
    table.MutableBufferData = mutable_buffer_data;
    table.MutableBufferLength = mutable_buffer_length;
    table.MutableBufferCapacity = mutable_buffer_capacity;
    table.SetMutableBufferLength = set_mutable_buffer_length;
    table.SetMutableBufferCapacity = set_mutable_buffer_capacity;
    table.IsMutableBuffer = is_mutable_buffer;
    table.ObjectToCSelf = object_to_cself;
    table.ObjectToCSelfScoped = object_to_cself_scoped;
    table.DirectoryPut = collections::directory_put;
    table.DirectoryAt = collections::directory_at;
    table.DirectoryRemove = collections::directory_remove;
    table.NewDirectory = collections::new_directory;
    table.IsDirectory = collections::is_directory;
    table.StringTablePut = collections::string_table_put;
    table.StringTableAt = collections::string_table_at;
    table.StringTableRemove = collections::string_table_remove;
    table.NewStringTable = collections::new_string_table;
    table.IsStringTable = collections::is_string_table;
    table.ArrayAt = collections::array_at;
    table.ArrayPut = collections::array_put;
    table.ArrayAppend = collections::array_append;
    table.ArrayAppendString = collections::array_append_string;
    table.ArraySize = collections::array_size;
    table.ArrayItems = collections::array_items;
    table.ArrayDimension = collections::array_dimension;
    table.NewArray = collections::new_array;
    table.ArrayOfOne = collections::array_of_one;
    table.ArrayOfTwo = collections::array_of_two;
    table.ArrayOfThree = collections::array_of_three;
    table.ArrayOfFour = collections::array_of_four;
    table.IsArray = collections::is_array;
    table.SupplierItem = collections::supplier_item;
    table.SupplierIndex = collections::supplier_index;
    table.SupplierAvailable = collections::supplier_available;
    table.SupplierNext = collections::supplier_next;
    table.NewSupplier = collections::new_supplier;
    table.NewStem = collections::new_stem;
    table.SetStemElement = collections::set_stem_element;
    table.GetStemElement = collections::get_stem_element;
    table.DropStemElement = collections::drop_stem_element;
    table.SetStemArrayElement = collections::set_stem_array_element;
    table.GetStemArrayElement = collections::get_stem_array_element;
    table.DropStemArrayElement = collections::drop_stem_array_element;
    table.GetAllStemElements = collections::get_all_stem_elements;
    table.GetStemValue = collections::get_stem_value;
    table.IsStem = collections::is_stem;
    table.VariableReferenceName = variables::variable_reference_name;
    table.VariableReferenceValue = variables::variable_reference_value;
    table.SetVariableReferenceValue = variables::set_variable_reference_value;
    table.IsVariableReference = variables::is_variable_reference;
    table.SendMessage = messages::send_message;
    table.SendMessage0 = messages::send_message0;
    table.SendMessage1 = messages::send_message1;
    table.SendMessage2 = messages::send_message2;
    table.SendMessageScoped = messages::send_message_scoped;
    table.GetLocalEnvironment = messages::get_local_environment;
    table.GetGlobalEnvironment = messages::get_global_environment;
    table.IsInstanceOf = messages::is_instance_of;
    table.IsOfType = messages::is_of_type;
    table.HasMethod = messages::has_method;
    table.FindClass = messages::find_class;
    table.FindPackageClass = messages::find_package_class;
    table.IsMethod = messages::is_method;
    table.IsRoutine = messages::is_routine;
    table.LoadPackage = packages::load_package;
    table.LoadPackageFromData = packages::load_package_from_data;
    table.LoadLibrary = packages::load_library;
    table.GetPackageRoutines = packages::package_routines;
    table.GetPackagePublicRoutines = packages::package_public_routines;
    table.GetPackageClasses = packages::package_classes;
    table.GetPackagePublicClasses = packages::package_public_classes;
    table.GetPackageMethods = packages::package_methods;
    table.GetRoutinePackage = packages::routine_package;
    table.GetMethodPackage = packages::method_package;
    table.CallRoutine = packages::call_routine;
    table.CallProgram = packages::call_program;
    table.NewMethod = packages::new_method;
    table.NewRoutine = packages::new_routine;
    table.RequestGlobalReference = request_global_reference;
    table.ReleaseGlobalReference = release_global_reference;
    table.ReleaseLocalReference = release_local_reference;
    table.RegisterLibrary = register_library;
    table.GetInterpreterInstance = get_interpreter_instance;
    table.DetachThread = detach_thread;
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

/// The thread context an extension is handed, which it may keep for as long as
/// the interpreter runs: the oracle's belongs to the activity
/// (`interpreter/concurrency/ActivationApiContexts.hpp:64-68`), and
/// `RexxPackageLoader` is handed it (`api/oorexxapi.h:257`).
///
/// Its callbacks reach the innermost native call in flight, which is what
/// `contextToActivation` does for a thread context
/// (`interpreter/concurrency/Activity.hpp:458`). A clone addresses the same
/// context; the allocation lives until the last clone is dropped.
#[derive(Clone)]
pub struct ThreadContext {
    home: Rc<Home>,
}

/// Owns the allocation a [`ThreadContext`] hands out addresses into. Nothing
/// forms a reference to the whole of it, so an address an extension kept is
/// never invalidated by one.
struct Home(NonNull<Thread>);

impl Drop for Home {
    fn drop(&mut self) {
        // SAFETY: the pointer came from `Box::into_raw` in
        // `ThreadContext::new`, and only this `Home`, which is not `Clone`,
        // frees it.
        drop(unsafe { Box::from_raw(self.0.as_ptr()) });
    }
}

/// The public structs a thread context links, its table, and the native
/// call its callbacks reach.
struct Thread {
    thread: Owned<RexxThreadContext_, Innermost>,
    instance: Owned<RexxInstance_, Innermost>,
    table: RexxThreadInterface,
    innermost: Innermost,
    /// The thread the interpreter runs on, which is the only one an
    /// extension may attach to it.
    home: std::thread::ThreadId,
}

/// The activation of the innermost native call in flight, null where none is.
///
/// The lifetime is erased: the pointer is written only by
/// [`ThreadContext::enter`], for the length of a borrow of the activation,
/// and put back before that borrow ends.
struct Innermost(Cell<*const Activation<'static>>);

impl ThreadContext {
    /// A thread context with no native call in flight.
    #[must_use]
    pub fn new() -> ThreadContext {
        let raw = Box::into_raw(Box::new(Thread {
            thread: Owned {
                context: RexxThreadContext_ {
                    instance: std::ptr::null_mut(),
                    functions: std::ptr::null_mut(),
                },
                owner: std::ptr::null_mut(),
            },
            instance: Owned {
                context: RexxInstance_ {
                    functions: std::ptr::null_mut(),
                    applicationData: std::ptr::null_mut(),
                },
                owner: std::ptr::null_mut(),
            },
            table: THREAD,
            innermost: Innermost(Cell::new(std::ptr::null())),
            home: std::thread::current().id(),
        }));
        // SAFETY: `raw` is the allocation just made and nothing else addresses
        // it yet. Every link is taken from `raw` itself, so its provenance is
        // the whole allocation's, which is what lets `owner_of` read `owner`
        // beside a public struct.
        unsafe {
            let innermost = &raw mut (*raw).innermost;
            (*raw).thread.context.instance = (&raw mut (*raw).instance).cast();
            (*raw).thread.context.functions = &raw mut (*raw).table;
            (*raw).thread.owner = innermost;
            (*raw).instance.context.functions = std::ptr::from_ref(&INSTANCE).cast_mut();
            (*raw).instance.owner = innermost;
        }
        ThreadContext {
            home: Rc::new(Home(
                NonNull::new(raw).expect("Box::into_raw answers a non-null pointer"),
            )),
        }
    }

    /// The address an extension is handed.
    pub(crate) fn pointer(&self) -> *mut RexxThreadContext_ {
        let raw = self.home.0.as_ptr();
        // SAFETY: the allocation is live while `self` is, and naming a
        // field's address reads and writes nothing.
        unsafe { (&raw mut (*raw).thread).cast() }
    }

    /// Runs `body` with `activation` as the innermost native call, and puts
    /// back the call that was innermost before, however `body` ends.
    ///
    /// The thread table's data members are written from `activation`'s
    /// constants on the first call and read on every later one.
    ///
    /// # Panics
    /// In a debug build, if `activation`'s constants are not the handles the
    /// first call wrote.
    pub fn enter<'h, R>(
        &self,
        activation: &Activation<'h>,
        body: impl FnOnce(&mut Contexts<'_, 'h>) -> R,
    ) -> R {
        let raw = self.home.0.as_ptr();
        let constants = activation.constants();
        // SAFETY: the allocation is live while `self` is. The data members
        // are written only while they are still null, which is before any
        // extension has been handed this context, so nothing is reading them.
        unsafe {
            if (*raw).table.RexxNil.is_null() {
                (*raw).table.RexxNil = constants.nil;
                (*raw).table.RexxTrue = constants.true_object;
                (*raw).table.RexxFalse = constants.false_object;
                (*raw).table.RexxNullString = constants.null_string.cast();
            }
        }
        debug_assert_eq!(self.constants(), constants, "a constant's handle moved");
        // SAFETY: as above; the reference covers only the cell.
        let innermost = unsafe { &(*raw).innermost };
        let _entered = Entered {
            innermost,
            previous: innermost
                .0
                .replace(std::ptr::from_ref(activation).cast::<Activation<'static>>()),
        };
        let mut contexts = Contexts {
            thread: self.pointer(),
            method: Owned {
                context: RexxMethodContext_ {
                    threadContext: std::ptr::null_mut(),
                    functions: std::ptr::null_mut(),
                    arguments: std::ptr::null_mut(),
                },
                owner: std::ptr::from_ref(activation).cast_mut(),
            },
            call: Owned {
                context: RexxCallContext_ {
                    threadContext: std::ptr::null_mut(),
                    functions: std::ptr::null_mut(),
                    arguments: std::ptr::null_mut(),
                },
                owner: std::ptr::from_ref(activation).cast_mut(),
            },
            activation: PhantomData,
        };
        body(&mut contexts)
    }

    /// The handles the thread table's data members carry, null before the
    /// first [`ThreadContext::enter`].
    #[must_use]
    pub fn constants(&self) -> crate::values::Constants<RexxObjectPtr> {
        let raw = self.home.0.as_ptr();
        // SAFETY: the allocation is live while `self` is, and a data member
        // is written only before any extension could read it.
        unsafe {
            crate::values::Constants {
                nil: (*raw).table.RexxNil,
                true_object: (*raw).table.RexxTrue,
                false_object: (*raw).table.RexxFalse,
                null_string: (*raw).table.RexxNullString.cast(),
            }
        }
    }
}

impl Default for ThreadContext {
    fn default() -> ThreadContext {
        ThreadContext::new()
    }
}

/// Puts back the native call that was innermost before an
/// [`ThreadContext::enter`].
struct Entered<'a> {
    innermost: &'a Innermost,
    previous: *const Activation<'static>,
}

impl Drop for Entered<'_> {
    fn drop(&mut self) {
        self.innermost.0.set(self.previous);
    }
}

/// The method and call contexts one native call hands an extension, each
/// linking the interpreter's [`ThreadContext`]. Built only by
/// [`ThreadContext::enter`], for the length of the call.
pub struct Contexts<'a, 'h> {
    thread: *mut RexxThreadContext_,
    method: Owned<RexxMethodContext_, Activation<'h>>,
    call: Owned<RexxCallContext_, Activation<'h>>,
    /// Ties this wrapper to the activation its contexts address, so that no
    /// context it hands out can outlive the state behind it.
    activation: PhantomData<&'a Activation<'h>>,
}

impl Contexts<'_, '_> {
    /// The method context, linking the thread context.
    ///
    /// The links are written here rather than at construction because the
    /// context pointer is the address of a field of `self`, which moving
    /// `self` changes. It is taken from the whole wrapper, which is what lets
    /// [`owner_of`] read the `owner` beside the public struct.
    pub fn method(&mut self) -> MethodContext<'_> {
        self.method.context.threadContext = self.thread;
        self.method.context.functions = std::ptr::from_ref(&METHOD_CONTEXT).cast_mut();
        MethodContext {
            pointer: (&raw mut self.method).cast::<RexxMethodContext_>(),
            wrapper: PhantomData,
        }
    }

    /// The call context, linked as [`Contexts::method`] links the method
    /// context.
    pub fn call(&mut self) -> CallContext<'_> {
        self.call.context.threadContext = self.thread;
        self.call.context.functions = std::ptr::from_ref(&CALL_CONTEXT).cast_mut();
        CallContext {
            pointer: (&raw mut self.call).cast::<RexxCallContext_>(),
            wrapper: PhantomData,
        }
    }

    /// The thread context a package loader or unloader is handed.
    pub(crate) fn thread(&self) -> *mut RexxThreadContext_ {
        self.thread
    }
}

/// A method or call context, which links its thread context first.
trait CallLinked {
    /// The `ThrowException0`, `1` and `2` members' names in this context's
    /// table, as a refusal records them.
    const THROW: [&'static str; 3];

    /// The thread context `context` links.
    ///
    /// # Safety
    /// `context` is a live context of this type.
    unsafe fn thread_of(context: *mut Self) -> *mut RexxThreadContext_;
}

impl CallLinked for RexxMethodContext_ {
    const THROW: [&'static str; 3] = [
        "MethodContextInterface.ThrowException0",
        "MethodContextInterface.ThrowException1",
        "MethodContextInterface.ThrowException2",
    ];

    unsafe fn thread_of(context: *mut Self) -> *mut RexxThreadContext_ {
        // SAFETY: the caller guarantees `context` is live.
        unsafe { (*context).threadContext }
    }
}

impl CallLinked for RexxCallContext_ {
    const THROW: [&'static str; 3] = [
        "CallContextInterface.ThrowException0",
        "CallContextInterface.ThrowException1",
        "CallContextInterface.ThrowException2",
    ];

    unsafe fn thread_of(context: *mut Self) -> *mut RexxThreadContext_ {
        // SAFETY: the caller guarantees `context` is live.
        unsafe { (*context).threadContext }
    }
}

/// The activation a method or call context addresses, or, while a call
/// nested inside that one holds its conversion state, the innermost
/// activation, which is the one the oracle's non-blocking members answer
/// for (`ApiContext(RexxCallContext *, bool)`, `interpreter/api/ContextApi.hpp:135`).
///
/// # Safety
/// `context` is a method or call context a [`Contexts`] handed out, used
/// during the call it was handed to, whose thread context link was written.
unsafe fn activation_of<'a, C: CallLinked>(context: *mut C) -> &'a Activation<'a> {
    // SAFETY: the caller guarantees `context` came from a live `Contexts`,
    // whose wrappers `ThreadContext::enter` built with `owner` pointing at the
    // `&'a Activation` it was given. That reference is shared, so forming
    // another one here aliases nothing: every write past it goes through the
    // activation's own cells.
    let own = unsafe { &*owner_of::<C, Activation<'a>>(context) };
    if !own.is_busy() {
        return own;
    }
    // SAFETY: the caller guarantees the link to a live thread context, and a
    // held conversion means a call is in flight on it.
    unsafe { innermost_activation(C::thread_of(context), "nested call") }
}

/// The activation of the innermost native call in flight, reached through a
/// thread context.
///
/// # Panics
/// Where no native call is in flight. The oracle aborts there too: measured,
/// an extension's destructor calling `WholeNumberToObject` through a thread
/// context it kept, run once the interpreter has terminated, ends the oracle
/// at rc 134 with `terminate called after throwing an instance of
/// 'NativeActivation*'`. The caller is an `extern "C"` frame, so the panic
/// aborts.
///
/// # Safety
/// `context` is a thread context a [`ThreadContext`] handed out, some clone
/// of which is alive. A caller holding the innermost call's conversion state
/// is caught by its `RefCell`, a panic.
unsafe fn innermost_activation<'a>(
    context: *mut RexxThreadContext_,
    slot: &str,
) -> &'a Activation<'a> {
    // SAFETY: the caller guarantees `context` came from a live
    // `ThreadContext`, whose wrapper's `owner` is its `Innermost`, a field of
    // the same allocation.
    let innermost = unsafe { &*owner_of::<RexxThreadContext_, Innermost>(context) };
    let activation = innermost.0.get();
    assert!(
        !activation.is_null(),
        "RexxThreadInterface.{slot} was called with no native call in flight"
    );
    // SAFETY: a non-null pointer is the activation `ThreadContext::enter` is
    // running a call for, which is borrowed for as long as the pointer is
    // there. The reference is shared, as in `activation_of`.
    unsafe { &*activation.cast::<Activation<'a>>() }
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
/// `context` is a method context a [`Contexts`] handed out, used during the
/// call it was handed to, and a non-null `name` is a NUL-terminated string.
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
/// `context` is a thread context a live [`ThreadContext`] handed out.
unsafe extern "C" fn whole_number_to_object(
    context: *mut RexxThreadContext_,
    value: wholenumber_t,
) -> RexxObjectPtr {
    // SAFETY: the caller guarantees the context, and `invoke::run` and
    // `invoke::hook` hold no conversion state across the call they make.
    unsafe { innermost_activation(context, "WholeNumberToObject") }.whole_number(value)
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn string_data(
    context: *mut RexxThreadContext_,
    string: RexxStringObject,
) -> CSTRING {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "StringData") }.string_data(string.cast())
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn string_length(
    context: *mut RexxThreadContext_,
    string: RexxStringObject,
) -> usize {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "StringLength") }.string_length(string.cast())
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn new_pointer(
    context: *mut RexxThreadContext_,
    value: POINTER,
) -> RexxPointerObject {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "NewPointer") }
        .new_pointer(value)
        .cast()
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
    unsafe { innermost_activation(context, "DoubleToObjectWithPrecision") }
        .double_object(value, precision)
}

/// # Safety
/// `context` is a call context a [`Contexts`] handed out, used during the
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
    unsafe { innermost_activation(context, "RaiseException0") }.raise_syntax(number);
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn raise_exception1(
    context: *mut RexxThreadContext_,
    number: usize,
    first: RexxObjectPtr,
) {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "RaiseException1") }.raise_with(
        "RexxThreadInterface.RaiseException1",
        number,
        &[first],
    );
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn raise_exception2(
    context: *mut RexxThreadContext_,
    number: usize,
    first: RexxObjectPtr,
    second: RexxObjectPtr,
) {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "RaiseException2") }.raise_with(
        "RexxThreadInterface.RaiseException2",
        number,
        &[first, second],
    );
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn raise_exception(
    context: *mut RexxThreadContext_,
    number: usize,
    substitutions: RexxArrayObject,
) {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "RaiseException") }
        .raise_with_array(number, substitutions.cast());
}

/// # Safety
/// As [`whole_number_to_object`], and a non-null `name` is NUL-terminated.
unsafe extern "C" fn raise_condition(
    context: *mut RexxThreadContext_,
    name: CSTRING,
    description: RexxStringObject,
    additional: RexxObjectPtr,
    result: RexxObjectPtr,
) {
    // SAFETY: as `whole_number_to_object`.
    let activation = unsafe { innermost_activation(context, "RaiseCondition") };
    // SAFETY: the caller guarantees the terminator.
    let name = unsafe { name_of(name) }.unwrap_or_default();
    activation.raise_condition(name, description.cast(), additional, result);
}

/// The payload a `Throw` member unwinds the extension with, which the call's
/// boundary (`load::call_stub`) accepts and nothing else carries.
pub(crate) struct Thrown;

/// Leaves the extension as the oracle's `Throw` stubs do by C++ `throw`
/// (`interpreter/api/CallContextStubs.cpp:207-213`): the condition is already
/// recorded, and the call answers once the unwind reaches its boundary.
fn unwind() -> ! {
    std::panic::resume_unwind(Box::new(Thrown))
}

/// `ThrowException0`: [`raise_exception0`], then out of the extension.
///
/// # Safety
/// As [`activation_of`].
unsafe extern "C-unwind" fn throw_exception0<C: CallLinked>(context: *mut C, number: usize) {
    // SAFETY: as `activation_of`.
    unsafe { activation_of(context) }.raise_syntax(number);
    unwind()
}

/// `ThrowException1`: [`raise_exception1`], then out of the extension.
///
/// # Safety
/// As [`activation_of`].
unsafe extern "C-unwind" fn throw_exception1<C: CallLinked>(
    context: *mut C,
    number: usize,
    first: RexxObjectPtr,
) {
    // SAFETY: as `activation_of`.
    unsafe { activation_of(context) }.raise_with(C::THROW[1], number, &[first]);
    unwind()
}

/// `ThrowException2`: [`raise_exception2`], then out of the extension.
///
/// # Safety
/// As [`activation_of`].
unsafe extern "C-unwind" fn throw_exception2<C: CallLinked>(
    context: *mut C,
    number: usize,
    first: RexxObjectPtr,
    second: RexxObjectPtr,
) {
    // SAFETY: as `activation_of`.
    unsafe { activation_of(context) }.raise_with(C::THROW[2], number, &[first, second]);
    unwind()
}

/// `ThrowException`: [`raise_exception`], then out of the extension.
///
/// # Safety
/// As [`activation_of`].
unsafe extern "C-unwind" fn throw_exception<C: CallLinked>(
    context: *mut C,
    number: usize,
    substitutions: RexxArrayObject,
) {
    // SAFETY: as `activation_of`.
    unsafe { activation_of(context) }.raise_with_array(number, substitutions.cast());
    unwind()
}

/// `ThrowCondition`: [`raise_condition`], then out of the extension.
///
/// # Safety
/// As [`activation_of`], and a non-null `name` is NUL-terminated.
unsafe extern "C-unwind" fn throw_condition<C: CallLinked>(
    context: *mut C,
    name: CSTRING,
    description: RexxStringObject,
    additional: RexxObjectPtr,
    result: RexxObjectPtr,
) {
    // SAFETY: as `activation_of`.
    let activation = unsafe { activation_of(context) };
    // SAFETY: the caller guarantees the terminator.
    let name = unsafe { name_of(name) }.unwrap_or_default();
    activation.raise_condition(name, description.cast(), additional, result);
    unwind()
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn check_condition(context: *mut RexxThreadContext_) -> logical_t {
    // SAFETY: as `whole_number_to_object`.
    logical_t::from(unsafe { innermost_activation(context, "CheckCondition") }.check_condition())
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn clear_condition(context: *mut RexxThreadContext_) {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "ClearCondition") }.clear_condition();
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn get_condition_info(context: *mut RexxThreadContext_) -> RexxDirectoryObject {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "GetConditionInfo") }
        .condition_info()
        .cast()
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn display_condition(context: *mut RexxThreadContext_) -> wholenumber_t {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "DisplayCondition") }.display_condition()
}

/// # Safety
/// As [`whole_number_to_object`], and a non-null `condition` is valid for a
/// write.
unsafe extern "C" fn decode_condition_info(
    context: *mut RexxThreadContext_,
    directory: RexxDirectoryObject,
    condition: *mut RexxCondition,
) {
    // SAFETY: as `whole_number_to_object`.
    let activation = unsafe { innermost_activation(context, "DecodeConditionInfo") };
    if condition.is_null() {
        return;
    }
    let decoded = activation.decode_condition(directory.cast());
    let written = decoded.map_or(
        RexxCondition {
            code: 0,
            rc: 0,
            position: 0,
            conditionName: std::ptr::null_mut(),
            message: std::ptr::null_mut(),
            errortext: std::ptr::null_mut(),
            program: std::ptr::null_mut(),
            description: std::ptr::null_mut(),
            additional: std::ptr::null_mut(),
        },
        |decoded| RexxCondition {
            code: decoded.code,
            rc: decoded.rc,
            position: decoded.position,
            conditionName: decoded.name.cast(),
            message: decoded.message.cast(),
            errortext: decoded.errortext.cast(),
            program: decoded.program.cast(),
            description: decoded.description.cast(),
            additional: decoded.additional.cast(),
        },
    );
    // SAFETY: the caller guarantees the write.
    unsafe { condition.write(written) };
}

/// Writes `value` through `out` and answers true, or answers false for no
/// value. A null `out` is not written.
///
/// # Safety
/// A non-null `out` is valid for a write of `T`.
unsafe fn answer_through<T>(out: *mut T, value: Option<T>) -> logical_t {
    let Some(value) = value else {
        return 0;
    };
    if !out.is_null() {
        // SAFETY: the caller guarantees a non-null `out` is writable.
        unsafe { out.write(value) };
    }
    1
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn uintptr_to_object(
    context: *mut RexxThreadContext_,
    value: usize,
) -> RexxObjectPtr {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "UintptrToObject") }.unsigned_object(value as u64)
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn intptr_to_object(
    context: *mut RexxThreadContext_,
    value: isize,
) -> RexxObjectPtr {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "IntptrToObject") }.signed_object(value as i64)
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn string_size_to_object(
    context: *mut RexxThreadContext_,
    value: stringsize_t,
) -> RexxObjectPtr {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "StringSizeToObject") }.unsigned_object(value as u64)
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn int64_to_object(
    context: *mut RexxThreadContext_,
    value: i64,
) -> RexxObjectPtr {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "Int64ToObject") }.signed_object(value)
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn unsigned_int64_to_object(
    context: *mut RexxThreadContext_,
    value: u64,
) -> RexxObjectPtr {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "UnsignedInt64ToObject") }.unsigned_object(value)
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn int32_to_object(
    context: *mut RexxThreadContext_,
    value: i32,
) -> RexxObjectPtr {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "Int32ToObject") }.signed_object(i64::from(value))
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn unsigned_int32_to_object(
    context: *mut RexxThreadContext_,
    value: u32,
) -> RexxObjectPtr {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "UnsignedInt32ToObject") }
        .unsigned_object(u64::from(value))
}

/// # Safety
/// As [`whole_number_to_object`], and a non-null `result` is valid for a
/// write.
unsafe extern "C" fn object_to_whole_number(
    context: *mut RexxThreadContext_,
    object: RexxObjectPtr,
    result: *mut wholenumber_t,
) -> logical_t {
    // SAFETY: as `whole_number_to_object`.
    let found = unsafe { innermost_activation(context, "ObjectToWholeNumber") }.signed_value(
        object,
        -MAX_WHOLENUMBER,
        MAX_WHOLENUMBER,
    );
    // SAFETY: the caller guarantees `result`; the value is within the range.
    unsafe { answer_through(result, found.map(|number| number as isize)) }
}

/// # Safety
/// As [`object_to_whole_number`].
unsafe extern "C" fn object_to_string_size(
    context: *mut RexxThreadContext_,
    object: RexxObjectPtr,
    result: *mut stringsize_t,
) -> logical_t {
    // SAFETY: as `whole_number_to_object`.
    let found = unsafe { innermost_activation(context, "ObjectToStringSize") }
        .unsigned_value(object, MAX_WHOLENUMBER.unsigned_abs());
    // SAFETY: the caller guarantees `result`; the value is within the range.
    unsafe { answer_through(result, found.map(|number| number as usize)) }
}

/// # Safety
/// As [`object_to_whole_number`].
unsafe extern "C" fn object_to_int64(
    context: *mut RexxThreadContext_,
    object: RexxObjectPtr,
    result: *mut i64,
) -> logical_t {
    // SAFETY: as `whole_number_to_object`.
    let found = unsafe { innermost_activation(context, "ObjectToInt64") }.signed_value(
        object,
        i64::MIN,
        i64::MAX,
    );
    // SAFETY: the caller guarantees `result`.
    unsafe { answer_through(result, found) }
}

/// # Safety
/// As [`object_to_whole_number`].
unsafe extern "C" fn object_to_unsigned_int64(
    context: *mut RexxThreadContext_,
    object: RexxObjectPtr,
    result: *mut u64,
) -> logical_t {
    // SAFETY: as `whole_number_to_object`.
    let found = unsafe { innermost_activation(context, "ObjectToUnsignedInt64") }
        .unsigned_value(object, u64::MAX);
    // SAFETY: the caller guarantees `result`.
    unsafe { answer_through(result, found) }
}

/// # Safety
/// As [`object_to_whole_number`].
unsafe extern "C" fn object_to_int32(
    context: *mut RexxThreadContext_,
    object: RexxObjectPtr,
    result: *mut i32,
) -> logical_t {
    // SAFETY: as `whole_number_to_object`.
    let found = unsafe { innermost_activation(context, "ObjectToInt32") }.signed_value(
        object,
        i64::from(i32::MIN),
        i64::from(i32::MAX),
    );
    // SAFETY: the caller guarantees `result`; the value is within the range.
    unsafe { answer_through(result, found.map(|number| number as i32)) }
}

/// # Safety
/// As [`object_to_whole_number`].
unsafe extern "C" fn object_to_unsigned_int32(
    context: *mut RexxThreadContext_,
    object: RexxObjectPtr,
    result: *mut u32,
) -> logical_t {
    // SAFETY: as `whole_number_to_object`.
    let found = unsafe { innermost_activation(context, "ObjectToUnsignedInt32") }
        .unsigned_value(object, u64::from(u32::MAX));
    // SAFETY: the caller guarantees `result`; the value is within the range.
    unsafe { answer_through(result, found.map(|number| number as u32)) }
}

/// # Safety
/// As [`object_to_whole_number`].
unsafe extern "C" fn object_to_uintptr(
    context: *mut RexxThreadContext_,
    object: RexxObjectPtr,
    result: *mut usize,
) -> logical_t {
    // SAFETY: as `whole_number_to_object`.
    let found = unsafe { innermost_activation(context, "ObjectToUintptr") }
        .unsigned_value(object, u64::MAX);
    // SAFETY: the caller guarantees `result`; a pointer is 64 bits.
    unsafe { answer_through(result, found.map(|number| number as usize)) }
}

/// # Safety
/// As [`object_to_whole_number`].
unsafe extern "C" fn object_to_intptr(
    context: *mut RexxThreadContext_,
    object: RexxObjectPtr,
    result: *mut isize,
) -> logical_t {
    // SAFETY: as `whole_number_to_object`.
    let found = unsafe { innermost_activation(context, "ObjectToIntptr") }.signed_value(
        object,
        i64::MIN,
        i64::MAX,
    );
    // SAFETY: the caller guarantees `result`; a pointer is 64 bits.
    unsafe { answer_through(result, found.map(|number| number as isize)) }
}

/// # Safety
/// As [`object_to_whole_number`].
unsafe extern "C" fn object_to_logical(
    context: *mut RexxThreadContext_,
    object: RexxObjectPtr,
    result: *mut logical_t,
) -> logical_t {
    // SAFETY: as `whole_number_to_object`.
    let found = unsafe { innermost_activation(context, "ObjectToLogical") }.logical_value(object);
    // SAFETY: the caller guarantees `result`.
    unsafe { answer_through(result, found.map(logical_t::from)) }
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn logical_to_object(
    context: *mut RexxThreadContext_,
    value: logical_t,
) -> RexxObjectPtr {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "LogicalToObject") }.logical_object(value != 0)
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn double_to_object(
    context: *mut RexxThreadContext_,
    value: f64,
) -> RexxObjectPtr {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "DoubleToObject") }.double_default_object(value)
}

/// # Safety
/// As [`object_to_whole_number`].
unsafe extern "C" fn object_to_double(
    context: *mut RexxThreadContext_,
    object: RexxObjectPtr,
    result: *mut f64,
) -> logical_t {
    // SAFETY: as `whole_number_to_object`.
    let found = unsafe { innermost_activation(context, "ObjectToDouble") }.double_value(object);
    // SAFETY: the caller guarantees `result`.
    unsafe { answer_through(result, found) }
}

/// The bytes `length` long at `data`, or `None` for a null `data` with a
/// length.
///
/// # Safety
/// A non-null `data` is valid for reads of `length` bytes.
unsafe fn bytes_of<'a>(data: CSTRING, length: usize) -> Option<&'a [u8]> {
    if data.is_null() {
        return (length == 0).then_some(&[]);
    }
    // SAFETY: the caller guarantees the range is readable.
    Some(unsafe { std::slice::from_raw_parts(data.cast::<u8>(), length) })
}

/// # Safety
/// As [`whole_number_to_object`], and a non-null `data` is valid for reads
/// of `length` bytes.
unsafe extern "C" fn new_string(
    context: *mut RexxThreadContext_,
    data: CSTRING,
    length: usize,
) -> RexxStringObject {
    // SAFETY: as `whole_number_to_object`.
    let activation = unsafe { innermost_activation(context, "NewString") };
    // SAFETY: the caller guarantees the range.
    match unsafe { bytes_of(data, length) } {
        Some(bytes) => activation.string_object(bytes).cast(),
        None => std::ptr::null_mut(),
    }
}

/// # Safety
/// As [`whole_number_to_object`], and a non-null `text` is NUL-terminated.
unsafe extern "C" fn new_string_from_asciiz(
    context: *mut RexxThreadContext_,
    text: CSTRING,
) -> RexxStringObject {
    // SAFETY: as `whole_number_to_object`.
    let activation = unsafe { innermost_activation(context, "NewStringFromAsciiz") };
    // SAFETY: the caller guarantees the terminator.
    match unsafe { name_of(text) } {
        Some(bytes) => activation.string_object(bytes).cast(),
        None => std::ptr::null_mut(),
    }
}

/// The code a descriptor declares, its value read as that code's member,
/// and the bytes a `CSTRING` value points at.
///
/// # Safety
/// `descriptor` is valid for reads, and the member its type names was
/// written; a `CSTRING` value that is not null is NUL-terminated.
unsafe fn described(descriptor: *const ValueDescriptor) -> (u16, Value, Option<Vec<u8>>) {
    // SAFETY: the caller guarantees the read.
    let descriptor = unsafe { &*descriptor };
    let declared = descriptor.r#type;
    let Some(repr) = crate::values::repr(declared) else {
        return (declared, Value::Omitted, None);
    };
    // SAFETY: the caller guarantees the member `repr` names was written.
    let value = unsafe { value_of(descriptor, repr) };
    let text = match value {
        // SAFETY: the caller guarantees the terminator.
        Value::CString(text) => unsafe { name_of(text) }.map(<[u8]>::to_vec),
        _ => None,
    };
    (declared, value, text)
}

/// # Safety
/// As [`whole_number_to_object`], and a non-null `descriptor` is as
/// [`described`] asks.
unsafe extern "C" fn value_to_object(
    context: *mut RexxThreadContext_,
    descriptor: *mut ValueDescriptor,
) -> RexxObjectPtr {
    // SAFETY: as `whole_number_to_object`.
    let activation = unsafe { innermost_activation(context, "ValueToObject") };
    if descriptor.is_null() {
        return std::ptr::null_mut();
    }
    // SAFETY: the caller guarantees the descriptor.
    let (declared, value, text) = unsafe { described(descriptor) };
    activation.value_to_object(declared, value, text.as_deref())
}

/// # Safety
/// As [`whole_number_to_object`], and a non-null `descriptors` is `count`
/// descriptors, each as [`described`] asks.
unsafe extern "C" fn values_to_object(
    context: *mut RexxThreadContext_,
    descriptors: *mut ValueDescriptor,
    count: usize,
) -> RexxArrayObject {
    // SAFETY: as `whole_number_to_object`.
    let activation = unsafe { innermost_activation(context, "ValuesToObject") };
    if descriptors.is_null() && count != 0 {
        return std::ptr::null_mut();
    }
    let described: Vec<(u16, Value, Option<Vec<u8>>)> = (0..count)
        // SAFETY: the caller guarantees `count` descriptors.
        .map(|at| unsafe { described(descriptors.add(at)) })
        .collect();
    activation.values_to_object(&described).cast()
}

/// # Safety
/// As [`whole_number_to_object`], and a non-null `descriptor` is valid for
/// reads and writes.
unsafe extern "C" fn object_to_value(
    context: *mut RexxThreadContext_,
    object: RexxObjectPtr,
    descriptor: *mut ValueDescriptor,
) -> logical_t {
    // SAFETY: as `whole_number_to_object`.
    let activation = unsafe { innermost_activation(context, "ObjectToValue") };
    if descriptor.is_null() {
        return 0;
    }
    // SAFETY: the caller guarantees the descriptor.
    let declared = unsafe { (*descriptor).r#type };
    let Some(value) = activation.object_to_value(object, declared) else {
        return 0;
    };
    let written = crate::values::descriptor(declared, Converted { value, flags: 0 });
    // SAFETY: the caller guarantees the write; only the value word changes.
    unsafe { (*descriptor).value = written.value };
    1
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn object_to_string(
    context: *mut RexxThreadContext_,
    object: RexxObjectPtr,
) -> RexxStringObject {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "ObjectToString") }
        .object_to_string(object)
        .cast()
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn object_to_string_value(
    context: *mut RexxThreadContext_,
    object: RexxObjectPtr,
) -> CSTRING {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "ObjectToStringValue") }.object_to_string_value(object)
}

/// # Safety
/// As [`whole_number_to_object`], and a non-null `buffer` is valid for
/// writes of `length` bytes.
unsafe extern "C" fn string_get(
    context: *mut RexxThreadContext_,
    string: RexxStringObject,
    offset: usize,
    buffer: POINTER,
    length: usize,
) -> usize {
    // SAFETY: as `whole_number_to_object`.
    let activation = unsafe { innermost_activation(context, "StringGet") };
    let Some(bytes) = activation.string_bytes(string.cast()) else {
        return 0;
    };
    // `RexxString::copyData` (`interpreter/classes/StringClass.cpp:599`),
    // whose start is one-based.
    let start = offset.wrapping_sub(1);
    if start >= bytes.len() || buffer.is_null() {
        return 0;
    }
    let copied = length.min(bytes.len() - start);
    // SAFETY: the caller guarantees `length` writable bytes, and `copied` is
    // no more than that.
    unsafe {
        std::ptr::copy_nonoverlapping(bytes[start..].as_ptr(), buffer.cast::<u8>(), copied);
    }
    copied
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn string_upper(
    context: *mut RexxThreadContext_,
    string: RexxStringObject,
) -> RexxStringObject {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "StringUpper") }
        .string_case(string.cast(), true)
        .cast()
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn string_lower(
    context: *mut RexxThreadContext_,
    string: RexxStringObject,
) -> RexxStringObject {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "StringLower") }
        .string_case(string.cast(), false)
        .cast()
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn is_string(
    context: *mut RexxThreadContext_,
    object: RexxObjectPtr,
) -> logical_t {
    // SAFETY: as `whole_number_to_object`.
    logical_t::from(unsafe { innermost_activation(context, "IsString") }.is_string(object))
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn new_buffer_string(
    context: *mut RexxThreadContext_,
    length: usize,
) -> RexxBufferStringObject {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "NewBufferString") }
        .new_buffer_string(length)
        .cast()
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn buffer_string_length(
    context: *mut RexxThreadContext_,
    string: RexxBufferStringObject,
) -> usize {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "BufferStringLength") }
        .buffer_string_length(string.cast())
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn buffer_string_data(
    context: *mut RexxThreadContext_,
    string: RexxBufferStringObject,
) -> POINTER {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "BufferStringData") }.buffer_string_data(string.cast())
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn finish_buffer_string(
    context: *mut RexxThreadContext_,
    string: RexxBufferStringObject,
    length: usize,
) -> RexxStringObject {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "FinishBufferString") }
        .finish_buffer_string(string.cast(), length)
        .cast()
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn new_buffer(
    context: *mut RexxThreadContext_,
    length: usize,
) -> RexxBufferObject {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "NewBuffer") }
        .new_buffer(length)
        .cast()
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn buffer_data(
    context: *mut RexxThreadContext_,
    buffer: RexxBufferObject,
) -> POINTER {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "BufferData") }
        .buffer_data("RexxThreadInterface.BufferData", buffer.cast())
        .map_or(std::ptr::null_mut(), |(address, _)| address)
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn buffer_length(
    context: *mut RexxThreadContext_,
    buffer: RexxBufferObject,
) -> usize {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "BufferLength") }
        .buffer_data("RexxThreadInterface.BufferLength", buffer.cast())
        .map_or(0, |(_, length)| length)
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn is_buffer(
    context: *mut RexxThreadContext_,
    object: RexxObjectPtr,
) -> logical_t {
    // SAFETY: as `whole_number_to_object`.
    logical_t::from(
        unsafe { innermost_activation(context, "IsBuffer") }.is_of_class(
            "RexxThreadInterface.IsBuffer",
            object,
            "Buffer",
        ),
    )
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn pointer_value(
    context: *mut RexxThreadContext_,
    pointer: RexxPointerObject,
) -> POINTER {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "PointerValue") }.pointer_value(pointer.cast())
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn is_pointer(
    context: *mut RexxThreadContext_,
    object: RexxObjectPtr,
) -> logical_t {
    // SAFETY: as `whole_number_to_object`.
    logical_t::from(
        unsafe { innermost_activation(context, "IsPointer") }.is_of_class(
            "RexxThreadInterface.IsPointer",
            object,
            "Pointer",
        ),
    )
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn new_mutable_buffer(
    context: *mut RexxThreadContext_,
    capacity: usize,
) -> RexxMutableBufferObject {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "NewMutableBuffer") }
        .new_mutable_buffer(capacity)
        .cast()
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn mutable_buffer_data(
    context: *mut RexxThreadContext_,
    buffer: RexxMutableBufferObject,
) -> POINTER {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "MutableBufferData") }
        .mutable_buffer("RexxThreadInterface.MutableBufferData", buffer.cast())
        .map_or(std::ptr::null_mut(), |(address, _, _)| address)
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn mutable_buffer_length(
    context: *mut RexxThreadContext_,
    buffer: RexxMutableBufferObject,
) -> usize {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "MutableBufferLength") }
        .mutable_buffer("RexxThreadInterface.MutableBufferLength", buffer.cast())
        .map_or(0, |(_, length, _)| length)
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn mutable_buffer_capacity(
    context: *mut RexxThreadContext_,
    buffer: RexxMutableBufferObject,
) -> usize {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "MutableBufferCapacity") }
        .mutable_buffer("RexxThreadInterface.MutableBufferCapacity", buffer.cast())
        .map_or(0, |(_, _, capacity)| capacity)
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn set_mutable_buffer_length(
    context: *mut RexxThreadContext_,
    buffer: RexxMutableBufferObject,
    length: usize,
) -> usize {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "SetMutableBufferLength") }
        .set_mutable_buffer_length(buffer.cast(), length)
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn set_mutable_buffer_capacity(
    context: *mut RexxThreadContext_,
    buffer: RexxMutableBufferObject,
    capacity: usize,
) -> POINTER {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "SetMutableBufferCapacity") }
        .set_mutable_buffer_capacity(buffer.cast(), capacity)
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn is_mutable_buffer(
    context: *mut RexxThreadContext_,
    object: RexxObjectPtr,
) -> logical_t {
    // SAFETY: as `whole_number_to_object`.
    logical_t::from(
        unsafe { innermost_activation(context, "IsMutableBuffer") }.is_of_class(
            "RexxThreadInterface.IsMutableBuffer",
            object,
            "MutableBuffer",
        ),
    )
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn object_to_cself(
    context: *mut RexxThreadContext_,
    object: RexxObjectPtr,
) -> POINTER {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "ObjectToCSelf") }.object_cself(object, None)
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn object_to_cself_scoped(
    context: *mut RexxThreadContext_,
    object: RexxObjectPtr,
    scope: RexxObjectPtr,
) -> POINTER {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "ObjectToCSelfScoped") }
        .object_cself(object, Some(scope))
}

/// # Safety
/// As [`set_object_variable`].
unsafe extern "C" fn get_cself(context: *mut RexxMethodContext_) -> POINTER {
    // SAFETY: as `set_object_variable`.
    unsafe { activation_of(context) }.cself()
}

/// The thread-table members over a collection, each of whose bodies is one
/// callback on the innermost activation; `name` arguments are
/// NUL-terminated strings the caller keeps for the call.
mod collections {
    use super::{innermost_activation, name_of};
    use crate::callbacks::Argument;
    use crate::layout::{
        CSTRING, RexxArrayObject, RexxDirectoryObject, RexxObjectPtr, RexxStemObject,
        RexxStringTableObject, RexxSupplierObject, RexxThreadContext_, logical_t,
    };

    /// # Safety
    /// As [`super::whole_number_to_object`], and a non-null `name` is
    /// NUL-terminated.
    pub(super) unsafe extern "C" fn directory_put(
        context: *mut RexxThreadContext_,
        table: RexxDirectoryObject,
        item: RexxObjectPtr,
        name: CSTRING,
    ) {
        // SAFETY: as `whole_number_to_object`; the caller guarantees `name`.
        let (activation, name) = unsafe {
            (
                innermost_activation(context, "DirectoryPut"),
                name_of(name).unwrap_or_default(),
            )
        };
        activation.table_put("RexxThreadInterface.DirectoryPut", table.cast(), item, name);
    }

    /// # Safety
    /// As [`directory_put`].
    pub(super) unsafe extern "C" fn directory_at(
        context: *mut RexxThreadContext_,
        table: RexxDirectoryObject,
        name: CSTRING,
    ) -> RexxObjectPtr {
        // SAFETY: as `directory_put`.
        let (activation, name) = unsafe {
            (
                innermost_activation(context, "DirectoryAt"),
                name_of(name).unwrap_or_default(),
            )
        };
        activation.table_at("RexxThreadInterface.DirectoryAt", table.cast(), name)
    }

    /// # Safety
    /// As [`directory_put`].
    pub(super) unsafe extern "C" fn directory_remove(
        context: *mut RexxThreadContext_,
        table: RexxDirectoryObject,
        name: CSTRING,
    ) -> RexxObjectPtr {
        // SAFETY: as `directory_put`.
        let (activation, name) = unsafe {
            (
                innermost_activation(context, "DirectoryRemove"),
                name_of(name).unwrap_or_default(),
            )
        };
        activation.table_remove("RexxThreadInterface.DirectoryRemove", table.cast(), name)
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn new_directory(
        context: *mut RexxThreadContext_,
    ) -> RexxDirectoryObject {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "NewDirectory") }
            .new_instance_of("RexxThreadInterface.NewDirectory", "Directory", &[])
            .cast()
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn is_directory(
        context: *mut RexxThreadContext_,
        object: RexxObjectPtr,
    ) -> logical_t {
        // SAFETY: as `whole_number_to_object`.
        logical_t::from(
            unsafe { innermost_activation(context, "IsDirectory") }.is_of_class(
                "RexxThreadInterface.IsDirectory",
                object,
                "Directory",
            ),
        )
    }

    /// # Safety
    /// As [`directory_put`].
    pub(super) unsafe extern "C" fn string_table_put(
        context: *mut RexxThreadContext_,
        table: RexxStringTableObject,
        item: RexxObjectPtr,
        name: CSTRING,
    ) {
        // SAFETY: as `directory_put`.
        let (activation, name) = unsafe {
            (
                innermost_activation(context, "StringTablePut"),
                name_of(name).unwrap_or_default(),
            )
        };
        activation.table_put(
            "RexxThreadInterface.StringTablePut",
            table.cast(),
            item,
            name,
        );
    }

    /// # Safety
    /// As [`directory_put`].
    pub(super) unsafe extern "C" fn string_table_at(
        context: *mut RexxThreadContext_,
        table: RexxStringTableObject,
        name: CSTRING,
    ) -> RexxObjectPtr {
        // SAFETY: as `directory_put`.
        let (activation, name) = unsafe {
            (
                innermost_activation(context, "StringTableAt"),
                name_of(name).unwrap_or_default(),
            )
        };
        activation.table_at("RexxThreadInterface.StringTableAt", table.cast(), name)
    }

    /// # Safety
    /// As [`directory_put`].
    pub(super) unsafe extern "C" fn string_table_remove(
        context: *mut RexxThreadContext_,
        table: RexxStringTableObject,
        name: CSTRING,
    ) -> RexxObjectPtr {
        // SAFETY: as `directory_put`.
        let (activation, name) = unsafe {
            (
                innermost_activation(context, "StringTableRemove"),
                name_of(name).unwrap_or_default(),
            )
        };
        activation.table_remove("RexxThreadInterface.StringTableRemove", table.cast(), name)
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn new_string_table(
        context: *mut RexxThreadContext_,
    ) -> RexxStringTableObject {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "NewStringTable") }
            .new_instance_of("RexxThreadInterface.NewStringTable", "StringTable", &[])
            .cast()
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn is_string_table(
        context: *mut RexxThreadContext_,
        object: RexxObjectPtr,
    ) -> logical_t {
        // SAFETY: as `whole_number_to_object`.
        logical_t::from(
            unsafe { innermost_activation(context, "IsStringTable") }.is_of_class(
                "RexxThreadInterface.IsStringTable",
                object,
                "StringTable",
            ),
        )
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn array_at(
        context: *mut RexxThreadContext_,
        array: RexxArrayObject,
        index: usize,
    ) -> RexxObjectPtr {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "ArrayAt") }.array_at(array.cast(), index)
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn array_put(
        context: *mut RexxThreadContext_,
        array: RexxArrayObject,
        item: RexxObjectPtr,
        index: usize,
    ) {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "ArrayPut") }.array_put(array.cast(), item, index);
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn array_append(
        context: *mut RexxThreadContext_,
        array: RexxArrayObject,
        item: RexxObjectPtr,
    ) -> usize {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "ArrayAppend") }.array_append(
            "RexxThreadInterface.ArrayAppend",
            array.cast(),
            Argument::Handle(item),
        )
    }

    /// # Safety
    /// As [`super::whole_number_to_object`], and a non-null `data` is valid
    /// for reads of `length` bytes.
    pub(super) unsafe extern "C" fn array_append_string(
        context: *mut RexxThreadContext_,
        array: RexxArrayObject,
        data: CSTRING,
        length: usize,
    ) -> usize {
        // SAFETY: as `whole_number_to_object`.
        let activation = unsafe { innermost_activation(context, "ArrayAppendString") };
        // SAFETY: the caller guarantees the range.
        let Some(bytes) = (unsafe { super::bytes_of(data, length) }) else {
            return 0;
        };
        activation.array_append(
            "RexxThreadInterface.ArrayAppendString",
            array.cast(),
            Argument::Text(bytes),
        )
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn array_size(
        context: *mut RexxThreadContext_,
        array: RexxArrayObject,
    ) -> usize {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "ArraySize") }.array_count(
            "RexxThreadInterface.ArraySize",
            array.cast(),
            b"SIZE",
        )
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn array_items(
        context: *mut RexxThreadContext_,
        array: RexxArrayObject,
    ) -> usize {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "ArrayItems") }.array_count(
            "RexxThreadInterface.ArrayItems",
            array.cast(),
            b"ITEMS",
        )
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn array_dimension(
        context: *mut RexxThreadContext_,
        array: RexxArrayObject,
    ) -> usize {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "ArrayDimension") }.array_dimension(array.cast())
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn new_array(
        context: *mut RexxThreadContext_,
        size: usize,
    ) -> RexxArrayObject {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "NewArray") }
            .new_array_sized(size)
            .cast()
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn array_of_one(
        context: *mut RexxThreadContext_,
        first: RexxObjectPtr,
    ) -> RexxArrayObject {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "ArrayOfOne") }
            .array_of("RexxThreadInterface.ArrayOfOne", &[first])
            .cast()
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn array_of_two(
        context: *mut RexxThreadContext_,
        first: RexxObjectPtr,
        second: RexxObjectPtr,
    ) -> RexxArrayObject {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "ArrayOfTwo") }
            .array_of("RexxThreadInterface.ArrayOfTwo", &[first, second])
            .cast()
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn array_of_three(
        context: *mut RexxThreadContext_,
        first: RexxObjectPtr,
        second: RexxObjectPtr,
        third: RexxObjectPtr,
    ) -> RexxArrayObject {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "ArrayOfThree") }
            .array_of("RexxThreadInterface.ArrayOfThree", &[first, second, third])
            .cast()
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn array_of_four(
        context: *mut RexxThreadContext_,
        first: RexxObjectPtr,
        second: RexxObjectPtr,
        third: RexxObjectPtr,
        fourth: RexxObjectPtr,
    ) -> RexxArrayObject {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "ArrayOfFour") }
            .array_of(
                "RexxThreadInterface.ArrayOfFour",
                &[first, second, third, fourth],
            )
            .cast()
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn is_array(
        context: *mut RexxThreadContext_,
        object: RexxObjectPtr,
    ) -> logical_t {
        // SAFETY: as `whole_number_to_object`.
        logical_t::from(
            unsafe { innermost_activation(context, "IsArray") }.is_of_class(
                "RexxThreadInterface.IsArray",
                object,
                "Array",
            ),
        )
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn supplier_item(
        context: *mut RexxThreadContext_,
        supplier: RexxSupplierObject,
    ) -> RexxObjectPtr {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "SupplierItem") }.supplier_object(
            "RexxThreadInterface.SupplierItem",
            supplier.cast(),
            b"ITEM",
        )
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn supplier_index(
        context: *mut RexxThreadContext_,
        supplier: RexxSupplierObject,
    ) -> RexxObjectPtr {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "SupplierIndex") }.supplier_object(
            "RexxThreadInterface.SupplierIndex",
            supplier.cast(),
            b"INDEX",
        )
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn supplier_available(
        context: *mut RexxThreadContext_,
        supplier: RexxSupplierObject,
    ) -> logical_t {
        // SAFETY: as `whole_number_to_object`.
        logical_t::from(
            unsafe { innermost_activation(context, "SupplierAvailable") }
                .supplier_available(supplier.cast()),
        )
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn supplier_next(
        context: *mut RexxThreadContext_,
        supplier: RexxSupplierObject,
    ) {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "SupplierNext") }.supplier_next(supplier.cast());
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn new_supplier(
        context: *mut RexxThreadContext_,
        values: RexxArrayObject,
        names: RexxArrayObject,
    ) -> RexxSupplierObject {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "NewSupplier") }
            .new_supplier(values.cast(), names.cast())
            .cast()
    }

    /// # Safety
    /// As [`directory_put`].
    pub(super) unsafe extern "C" fn new_stem(
        context: *mut RexxThreadContext_,
        name: CSTRING,
    ) -> RexxStemObject {
        // SAFETY: as `directory_put`.
        let (activation, name) =
            unsafe { (innermost_activation(context, "NewStem"), name_of(name)) };
        let arguments: &[Argument<'_>] = match &name {
            Some(name) => &[Argument::Text(name)],
            None => &[],
        };
        activation
            .new_instance_of("RexxThreadInterface.NewStem", "Stem", arguments)
            .cast()
    }

    /// # Safety
    /// As [`directory_put`].
    pub(super) unsafe extern "C" fn set_stem_element(
        context: *mut RexxThreadContext_,
        stem: RexxStemObject,
        tail: CSTRING,
        value: RexxObjectPtr,
    ) {
        // SAFETY: as `directory_put`.
        let (activation, tail) = unsafe {
            (
                innermost_activation(context, "SetStemElement"),
                name_of(tail).unwrap_or_default(),
            )
        };
        activation.stem_set(
            "RexxThreadInterface.SetStemElement",
            stem.cast(),
            Argument::Text(tail),
            value,
        );
    }

    /// # Safety
    /// As [`directory_put`].
    pub(super) unsafe extern "C" fn get_stem_element(
        context: *mut RexxThreadContext_,
        stem: RexxStemObject,
        tail: CSTRING,
    ) -> RexxObjectPtr {
        // SAFETY: as `directory_put`.
        let (activation, tail) = unsafe {
            (
                innermost_activation(context, "GetStemElement"),
                name_of(tail).unwrap_or_default(),
            )
        };
        activation.stem_get(
            "RexxThreadInterface.GetStemElement",
            stem.cast(),
            Argument::Text(tail),
        )
    }

    /// # Safety
    /// As [`directory_put`].
    pub(super) unsafe extern "C" fn drop_stem_element(
        context: *mut RexxThreadContext_,
        stem: RexxStemObject,
        tail: CSTRING,
    ) {
        // SAFETY: as `directory_put`.
        let (activation, tail) = unsafe {
            (
                innermost_activation(context, "DropStemElement"),
                name_of(tail).unwrap_or_default(),
            )
        };
        activation.stem_drop(
            "RexxThreadInterface.DropStemElement",
            stem.cast(),
            Argument::Text(tail),
        );
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn set_stem_array_element(
        context: *mut RexxThreadContext_,
        stem: RexxStemObject,
        index: usize,
        value: RexxObjectPtr,
    ) {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "SetStemArrayElement") }.stem_set(
            "RexxThreadInterface.SetStemArrayElement",
            stem.cast(),
            Argument::Number(index),
            value,
        );
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn get_stem_array_element(
        context: *mut RexxThreadContext_,
        stem: RexxStemObject,
        index: usize,
    ) -> RexxObjectPtr {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "GetStemArrayElement") }.stem_get(
            "RexxThreadInterface.GetStemArrayElement",
            stem.cast(),
            Argument::Number(index),
        )
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn drop_stem_array_element(
        context: *mut RexxThreadContext_,
        stem: RexxStemObject,
        index: usize,
    ) {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "DropStemArrayElement") }.stem_drop(
            "RexxThreadInterface.DropStemArrayElement",
            stem.cast(),
            Argument::Number(index),
        );
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn get_all_stem_elements(
        context: *mut RexxThreadContext_,
        stem: RexxStemObject,
    ) -> RexxDirectoryObject {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "GetAllStemElements") }
            .stem_whole(
                "RexxThreadInterface.GetAllStemElements",
                stem.cast(),
                b"TODIRECTORY",
            )
            .cast()
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn get_stem_value(
        context: *mut RexxThreadContext_,
        stem: RexxStemObject,
    ) -> RexxObjectPtr {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "GetStemValue") }.stem_whole(
            "RexxThreadInterface.GetStemValue",
            stem.cast(),
            b"[]",
        )
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn is_stem(
        context: *mut RexxThreadContext_,
        object: RexxObjectPtr,
    ) -> logical_t {
        // SAFETY: as `whole_number_to_object`.
        logical_t::from(
            unsafe { innermost_activation(context, "IsStem") }.is_of_class(
                "RexxThreadInterface.IsStem",
                object,
                "Stem",
            ),
        )
    }
}

/// The members over variables: the calling activation's through a call
/// context, the running method's object variables through a method context,
/// and a `VariableReference` through the thread table.
mod variables {
    use super::{activation_of, innermost_activation, name_of};
    use crate::layout::{
        CSTRING, RexxCallContext_, RexxDirectoryObject, RexxMethodContext_, RexxObjectPtr,
        RexxStemObject, RexxStringObject, RexxThreadContext_, RexxVariableReferenceObject,
        logical_t,
    };

    /// # Safety
    /// `context` is a call context a [`super::Contexts`] handed out, used
    /// during the call it was handed to, and a non-null `name` is
    /// NUL-terminated.
    pub(super) unsafe extern "C" fn get_context_variable(
        context: *mut RexxCallContext_,
        name: CSTRING,
    ) -> RexxObjectPtr {
        // SAFETY: the caller guarantees the context and the name.
        let (activation, name) = unsafe { (activation_of(context), name_of(name)) };
        name.map_or(std::ptr::null_mut(), |name| {
            activation.context_variable(name)
        })
    }

    /// # Safety
    /// As [`get_context_variable`].
    pub(super) unsafe extern "C" fn set_context_variable(
        context: *mut RexxCallContext_,
        name: CSTRING,
        value: RexxObjectPtr,
    ) {
        // SAFETY: as `get_context_variable`.
        let (activation, name) = unsafe { (activation_of(context), name_of(name)) };
        if let Some(name) = name {
            activation.set_context_variable(name, value);
        }
    }

    /// # Safety
    /// As [`get_context_variable`].
    pub(super) unsafe extern "C" fn drop_context_variable(
        context: *mut RexxCallContext_,
        name: CSTRING,
    ) {
        // SAFETY: as `get_context_variable`.
        let (activation, name) = unsafe { (activation_of(context), name_of(name)) };
        if let Some(name) = name {
            activation.drop_context_variable(name);
        }
    }

    /// # Safety
    /// As [`get_context_variable`].
    pub(super) unsafe extern "C" fn get_all_context_variables(
        context: *mut RexxCallContext_,
    ) -> RexxDirectoryObject {
        // SAFETY: as `get_context_variable`.
        unsafe { activation_of(context) }.context_variables().cast()
    }

    /// # Safety
    /// As [`get_context_variable`].
    pub(super) unsafe extern "C" fn resolve_stem_variable(
        context: *mut RexxCallContext_,
        object: RexxObjectPtr,
    ) -> RexxStemObject {
        // SAFETY: as `get_context_variable`.
        unsafe { activation_of(context) }
            .resolve_stem(object)
            .cast()
    }

    /// # Safety
    /// As [`get_context_variable`].
    pub(super) unsafe extern "C" fn get_context_variable_reference(
        context: *mut RexxCallContext_,
        name: CSTRING,
    ) -> RexxVariableReferenceObject {
        // SAFETY: as `get_context_variable`.
        let (activation, name) = unsafe { (activation_of(context), name_of(name)) };
        name.map_or(std::ptr::null_mut(), |name| {
            activation
                .variable_reference(
                    "CallContextInterface.GetContextVariableReference",
                    name,
                    false,
                )
                .cast()
        })
    }

    /// # Safety
    /// As [`super::set_object_variable`].
    pub(super) unsafe extern "C" fn get_object_variable(
        context: *mut RexxMethodContext_,
        name: CSTRING,
    ) -> RexxObjectPtr {
        // SAFETY: as `set_object_variable`.
        let (activation, name) = unsafe { (activation_of(context), name_of(name)) };
        name.map_or(std::ptr::null_mut(), |name| {
            activation.object_variable(name)
        })
    }

    /// # Safety
    /// As [`super::set_object_variable`].
    pub(super) unsafe extern "C" fn get_object_variable_reference(
        context: *mut RexxMethodContext_,
        name: CSTRING,
    ) -> RexxVariableReferenceObject {
        // SAFETY: as `set_object_variable`.
        let (activation, name) = unsafe { (activation_of(context), name_of(name)) };
        name.map_or(std::ptr::null_mut(), |name| {
            activation
                .variable_reference(
                    "MethodContextInterface.GetObjectVariableReference",
                    name,
                    true,
                )
                .cast()
        })
    }

    /// `SetGuardOn` and `SetGuardOff`, which do what this interpreter's own
    /// `GUARD ON` and `GUARD OFF` do with no other activity to exclude:
    /// nothing a program can see.
    ///
    /// # Safety
    /// As [`super::set_object_variable`]; the context is not read.
    pub(super) unsafe extern "C" fn set_guard(_context: *mut RexxMethodContext_) {}

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn variable_reference_name(
        context: *mut RexxThreadContext_,
        reference: RexxVariableReferenceObject,
    ) -> RexxStringObject {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "VariableReferenceName") }
            .reference_answer(
                "RexxThreadInterface.VariableReferenceName",
                reference.cast(),
                b"NAME",
            )
            .cast()
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn variable_reference_value(
        context: *mut RexxThreadContext_,
        reference: RexxVariableReferenceObject,
    ) -> RexxObjectPtr {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "VariableReferenceValue") }.reference_answer(
            "RexxThreadInterface.VariableReferenceValue",
            reference.cast(),
            b"VALUE",
        )
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn set_variable_reference_value(
        context: *mut RexxThreadContext_,
        reference: RexxVariableReferenceObject,
        value: RexxObjectPtr,
    ) {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "SetVariableReferenceValue") }
            .set_reference_value(reference.cast(), value);
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn is_variable_reference(
        context: *mut RexxThreadContext_,
        object: RexxObjectPtr,
    ) -> logical_t {
        // SAFETY: as `whole_number_to_object`.
        logical_t::from(
            unsafe { innermost_activation(context, "IsVariableReference") }.is_of_class(
                "RexxThreadInterface.IsVariableReference",
                object,
                "VariableReference",
            ),
        )
    }
}

/// The members that send messages, find classes and describe the running
/// call: the thread table's, and the method and call contexts' own.
mod messages {
    use super::{activation_of, innermost_activation, name_of};
    use crate::callbacks::MethodObject;
    use crate::layout::{
        CSTRING, RexxArrayObject, RexxCallContext_, RexxClassObject, RexxDirectoryObject,
        RexxMethodContext_, RexxMethodObject, RexxObjectPtr, RexxPackageObject, RexxRoutineObject,
        RexxThreadContext_, logical_t,
    };

    /// # Safety
    /// As [`super::whole_number_to_object`], and a non-null `name` is
    /// NUL-terminated.
    pub(super) unsafe extern "C" fn send_message(
        context: *mut RexxThreadContext_,
        receiver: RexxObjectPtr,
        name: CSTRING,
        arguments: RexxArrayObject,
    ) -> RexxObjectPtr {
        // SAFETY: as `whole_number_to_object`; the caller guarantees `name`.
        let (activation, name) =
            unsafe { (innermost_activation(context, "SendMessage"), name_of(name)) };
        let (Some(name), Some(arguments)) = (name, activation.arguments_of(arguments.cast()))
        else {
            return std::ptr::null_mut();
        };
        activation.send_message(
            "RexxThreadInterface.SendMessage",
            receiver,
            name,
            None,
            arguments,
        )
    }

    /// # Safety
    /// As [`send_message`].
    pub(super) unsafe extern "C" fn send_message_scoped(
        context: *mut RexxThreadContext_,
        receiver: RexxObjectPtr,
        name: CSTRING,
        scope: RexxClassObject,
        arguments: RexxArrayObject,
    ) -> RexxObjectPtr {
        // SAFETY: as `send_message`.
        let (activation, name) = unsafe {
            (
                innermost_activation(context, "SendMessageScoped"),
                name_of(name),
            )
        };
        let (Some(name), Some(arguments)) = (name, activation.arguments_of(arguments.cast()))
        else {
            return std::ptr::null_mut();
        };
        activation.send_message(
            "RexxThreadInterface.SendMessageScoped",
            receiver,
            name,
            Some(scope.cast()),
            arguments,
        )
    }

    /// # Safety
    /// As [`send_message`].
    pub(super) unsafe extern "C" fn send_message0(
        context: *mut RexxThreadContext_,
        receiver: RexxObjectPtr,
        name: CSTRING,
    ) -> RexxObjectPtr {
        // SAFETY: as `send_message`.
        let (activation, name) =
            unsafe { (innermost_activation(context, "SendMessage0"), name_of(name)) };
        let Some(name) = name else {
            return std::ptr::null_mut();
        };
        activation.send_message(
            "RexxThreadInterface.SendMessage0",
            receiver,
            name,
            None,
            Vec::new(),
        )
    }

    /// # Safety
    /// As [`send_message`].
    pub(super) unsafe extern "C" fn send_message1(
        context: *mut RexxThreadContext_,
        receiver: RexxObjectPtr,
        name: CSTRING,
        first: RexxObjectPtr,
    ) -> RexxObjectPtr {
        // SAFETY: as `send_message`.
        let (activation, name) =
            unsafe { (innermost_activation(context, "SendMessage1"), name_of(name)) };
        let Some(name) = name else {
            return std::ptr::null_mut();
        };
        let arguments = activation.objects(&[first]);
        activation.send_message(
            "RexxThreadInterface.SendMessage1",
            receiver,
            name,
            None,
            arguments,
        )
    }

    /// # Safety
    /// As [`send_message`].
    pub(super) unsafe extern "C" fn send_message2(
        context: *mut RexxThreadContext_,
        receiver: RexxObjectPtr,
        name: CSTRING,
        first: RexxObjectPtr,
        second: RexxObjectPtr,
    ) -> RexxObjectPtr {
        // SAFETY: as `send_message`.
        let (activation, name) =
            unsafe { (innermost_activation(context, "SendMessage2"), name_of(name)) };
        let Some(name) = name else {
            return std::ptr::null_mut();
        };
        let arguments = activation.objects(&[first, second]);
        activation.send_message(
            "RexxThreadInterface.SendMessage2",
            receiver,
            name,
            None,
            arguments,
        )
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn get_local_environment(
        context: *mut RexxThreadContext_,
    ) -> RexxDirectoryObject {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "GetLocalEnvironment") }
            .environment("RexxThreadInterface.GetLocalEnvironment", true)
            .cast()
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn get_global_environment(
        context: *mut RexxThreadContext_,
    ) -> RexxDirectoryObject {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "GetGlobalEnvironment") }
            .environment("RexxThreadInterface.GetGlobalEnvironment", false)
            .cast()
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn is_instance_of(
        context: *mut RexxThreadContext_,
        object: RexxObjectPtr,
        class: RexxClassObject,
    ) -> logical_t {
        // SAFETY: as `whole_number_to_object`.
        logical_t::from(
            unsafe { innermost_activation(context, "IsInstanceOf") }
                .is_instance_of(object, class.cast()),
        )
    }

    /// # Safety
    /// As [`send_message`].
    pub(super) unsafe extern "C" fn is_of_type(
        context: *mut RexxThreadContext_,
        object: RexxObjectPtr,
        name: CSTRING,
    ) -> logical_t {
        // SAFETY: as `send_message`.
        let (activation, name) =
            unsafe { (innermost_activation(context, "IsOfType"), name_of(name)) };
        logical_t::from(name.is_some_and(|name| activation.is_of_type(object, name)))
    }

    /// # Safety
    /// As [`send_message`].
    pub(super) unsafe extern "C" fn has_method(
        context: *mut RexxThreadContext_,
        object: RexxObjectPtr,
        name: CSTRING,
    ) -> logical_t {
        // SAFETY: as `send_message`.
        let (activation, name) =
            unsafe { (innermost_activation(context, "HasMethod"), name_of(name)) };
        logical_t::from(name.is_some_and(|name| activation.has_method(object, name)))
    }

    /// # Safety
    /// As [`send_message`].
    pub(super) unsafe extern "C" fn find_class(
        context: *mut RexxThreadContext_,
        name: CSTRING,
    ) -> RexxClassObject {
        // SAFETY: as `send_message`.
        let (activation, name) =
            unsafe { (innermost_activation(context, "FindClass"), name_of(name)) };
        name.map_or(std::ptr::null_mut(), |name| {
            activation
                .find_class("RexxThreadInterface.FindClass", name, true)
                .cast()
        })
    }

    /// # Safety
    /// As [`send_message`].
    pub(super) unsafe extern "C" fn find_package_class(
        context: *mut RexxThreadContext_,
        package: RexxPackageObject,
        name: CSTRING,
    ) -> RexxClassObject {
        // SAFETY: as `send_message`.
        let (activation, name) = unsafe {
            (
                innermost_activation(context, "FindPackageClass"),
                name_of(name),
            )
        };
        name.map_or(std::ptr::null_mut(), |name| {
            activation.find_package_class(package.cast(), name).cast()
        })
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn is_method(
        context: *mut RexxThreadContext_,
        object: RexxObjectPtr,
    ) -> logical_t {
        // SAFETY: as `whole_number_to_object`.
        logical_t::from(
            unsafe { innermost_activation(context, "IsMethod") }.is_executable(object, "Method"),
        )
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn is_routine(
        context: *mut RexxThreadContext_,
        object: RexxObjectPtr,
    ) -> logical_t {
        // SAFETY: as `whole_number_to_object`.
        logical_t::from(
            unsafe { innermost_activation(context, "IsRoutine") }.is_executable(object, "Routine"),
        )
    }

    /// # Safety
    /// As [`super::set_object_variable`].
    pub(super) unsafe extern "C" fn method_arguments(
        context: *mut RexxMethodContext_,
    ) -> RexxArrayObject {
        // SAFETY: as `set_object_variable`.
        unsafe { activation_of(context) }.arguments().cast()
    }

    /// # Safety
    /// As [`super::set_object_variable`].
    pub(super) unsafe extern "C" fn method_argument(
        context: *mut RexxMethodContext_,
        index: usize,
    ) -> RexxObjectPtr {
        // SAFETY: as `set_object_variable`.
        unsafe { activation_of(context) }.argument(index)
    }

    /// # Safety
    /// As [`super::set_object_variable`].
    pub(super) unsafe extern "C" fn message_name(context: *mut RexxMethodContext_) -> CSTRING {
        // SAFETY: as `set_object_variable`.
        unsafe { activation_of(context) }.message_name()
    }

    /// # Safety
    /// As [`super::set_object_variable`].
    pub(super) unsafe extern "C" fn current_method(
        context: *mut RexxMethodContext_,
    ) -> RexxMethodObject {
        // SAFETY: as `set_object_variable`.
        unsafe { activation_of(context) }
            .executable("MethodContextInterface.GetMethod")
            .cast()
    }

    /// # Safety
    /// As [`super::set_object_variable`].
    pub(super) unsafe extern "C" fn get_self(context: *mut RexxMethodContext_) -> RexxObjectPtr {
        // SAFETY: as `set_object_variable`.
        unsafe { activation_of(context) }.method_object(MethodObject::Receiver)
    }

    /// # Safety
    /// As [`super::set_object_variable`].
    pub(super) unsafe extern "C" fn get_super(context: *mut RexxMethodContext_) -> RexxClassObject {
        // SAFETY: as `set_object_variable`.
        unsafe { activation_of(context) }
            .method_object(MethodObject::Super)
            .cast()
    }

    /// # Safety
    /// As [`super::set_object_variable`].
    pub(super) unsafe extern "C" fn get_scope(context: *mut RexxMethodContext_) -> RexxObjectPtr {
        // SAFETY: as `set_object_variable`.
        unsafe { activation_of(context) }.method_object(MethodObject::Scope)
    }

    /// # Safety
    /// As [`super::set_object_variable`], and a non-null `name` is
    /// NUL-terminated.
    pub(super) unsafe extern "C" fn forward_message(
        context: *mut RexxMethodContext_,
        receiver: RexxObjectPtr,
        name: CSTRING,
        scope: RexxClassObject,
        arguments: RexxArrayObject,
    ) -> RexxObjectPtr {
        // SAFETY: as `set_object_variable`.
        let (activation, name) = unsafe { (activation_of(context), name_of(name)) };
        activation.forward_message(receiver, name, scope.cast(), arguments.cast())
    }

    /// # Safety
    /// As [`forward_message`].
    pub(super) unsafe extern "C" fn method_find_context_class(
        context: *mut RexxMethodContext_,
        name: CSTRING,
    ) -> RexxClassObject {
        // SAFETY: as `set_object_variable`.
        let (activation, name) = unsafe { (activation_of(context), name_of(name)) };
        name.map_or(std::ptr::null_mut(), |name| {
            activation
                .find_class("MethodContextInterface.FindContextClass", name, true)
                .cast()
        })
    }

    /// # Safety
    /// As [`super::get_context_digits`].
    pub(super) unsafe extern "C" fn call_arguments(
        context: *mut RexxCallContext_,
    ) -> RexxArrayObject {
        // SAFETY: as `get_context_digits`.
        unsafe { activation_of(context) }.arguments().cast()
    }

    /// # Safety
    /// As [`super::get_context_digits`].
    pub(super) unsafe extern "C" fn call_argument(
        context: *mut RexxCallContext_,
        index: usize,
    ) -> RexxObjectPtr {
        // SAFETY: as `get_context_digits`.
        unsafe { activation_of(context) }.argument(index)
    }

    /// # Safety
    /// As [`super::get_context_digits`].
    pub(super) unsafe extern "C" fn routine_name(context: *mut RexxCallContext_) -> CSTRING {
        // SAFETY: as `get_context_digits`.
        unsafe { activation_of(context) }.message_name()
    }

    /// # Safety
    /// As [`super::get_context_digits`].
    pub(super) unsafe extern "C" fn current_routine(
        context: *mut RexxCallContext_,
    ) -> RexxRoutineObject {
        // SAFETY: as `get_context_digits`.
        unsafe { activation_of(context) }
            .executable("CallContextInterface.GetRoutine")
            .cast()
    }

    /// # Safety
    /// As [`super::get_context_digits`].
    pub(super) unsafe extern "C" fn caller_context(
        context: *mut RexxCallContext_,
    ) -> RexxObjectPtr {
        // SAFETY: as `get_context_digits`.
        unsafe { activation_of(context) }.caller_context()
    }

    /// # Safety
    /// As [`super::get_context_digits`], and a non-null `name` is
    /// NUL-terminated.
    pub(super) unsafe extern "C" fn call_find_context_class(
        context: *mut RexxCallContext_,
        name: CSTRING,
    ) -> RexxClassObject {
        // SAFETY: as `get_context_digits`; the caller guarantees `name`.
        let (activation, name) = unsafe { (activation_of(context), name_of(name)) };
        name.map_or(std::ptr::null_mut(), |name| {
            activation
                .find_class("CallContextInterface.FindContextClass", name, false)
                .cast()
        })
    }

    /// # Safety
    /// As [`super::get_context_digits`].
    pub(super) unsafe extern "C" fn invalid_routine(context: *mut RexxCallContext_) {
        // SAFETY: as `get_context_digits`.
        unsafe { activation_of(context) }.invalid_routine();
    }
}

/// The members over packages, libraries, routines and methods.
mod packages {
    use super::{bytes_of, innermost_activation, name_of};
    use crate::layout::{
        CSTRING, RexxArrayObject, RexxDirectoryObject, RexxMethodObject, RexxObjectPtr,
        RexxPackageObject, RexxRoutineObject, RexxThreadContext_, logical_t,
    };

    /// # Safety
    /// As [`super::whole_number_to_object`], and a non-null `name` is
    /// NUL-terminated.
    pub(super) unsafe extern "C" fn load_package(
        context: *mut RexxThreadContext_,
        name: CSTRING,
    ) -> RexxPackageObject {
        // SAFETY: as `whole_number_to_object`; the caller guarantees `name`.
        let (activation, name) =
            unsafe { (innermost_activation(context, "LoadPackage"), name_of(name)) };
        name.map_or(std::ptr::null_mut(), |name| {
            activation.load_package(name).cast()
        })
    }

    /// # Safety
    /// As [`load_package`], and a non-null `data` is valid for reads of
    /// `length` bytes.
    pub(super) unsafe extern "C" fn load_package_from_data(
        context: *mut RexxThreadContext_,
        name: CSTRING,
        data: CSTRING,
        length: usize,
    ) -> RexxPackageObject {
        // SAFETY: as `load_package`; the caller guarantees the range.
        let (activation, name, source) = unsafe {
            (
                innermost_activation(context, "LoadPackageFromData"),
                name_of(name),
                bytes_of(data, length),
            )
        };
        let (Some(name), Some(source)) = (name, source) else {
            return std::ptr::null_mut();
        };
        activation.load_package_from_data(name, source).cast()
    }

    /// # Safety
    /// As [`load_package`].
    pub(super) unsafe extern "C" fn load_library(
        context: *mut RexxThreadContext_,
        name: CSTRING,
    ) -> logical_t {
        // SAFETY: as `load_package`.
        let (activation, name) =
            unsafe { (innermost_activation(context, "LoadLibrary"), name_of(name)) };
        logical_t::from(name.is_some_and(|name| activation.load_library(name)))
    }

    /// The package members that each answer one message of the package.
    macro_rules! package_answer {
        ($function:ident, $member:literal, $message:literal) => {
            /// # Safety
            /// As [`super::whole_number_to_object`].
            pub(super) unsafe extern "C" fn $function(
                context: *mut RexxThreadContext_,
                package: RexxPackageObject,
            ) -> RexxDirectoryObject {
                // SAFETY: as `whole_number_to_object`.
                unsafe { innermost_activation(context, $member) }
                    .package_answer(
                        concat!("RexxThreadInterface.", $member),
                        package.cast(),
                        $message,
                    )
                    .cast()
            }
        };
    }

    package_answer!(package_routines, "GetPackageRoutines", b"ROUTINES");
    package_answer!(
        package_public_routines,
        "GetPackagePublicRoutines",
        b"PUBLICROUTINES"
    );
    package_answer!(package_classes, "GetPackageClasses", b"CLASSES");
    package_answer!(
        package_public_classes,
        "GetPackagePublicClasses",
        b"PUBLICCLASSES"
    );
    package_answer!(package_methods, "GetPackageMethods", b"DEFINEDMETHODS");

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn routine_package(
        context: *mut RexxThreadContext_,
        routine: RexxRoutineObject,
    ) -> RexxPackageObject {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "GetRoutinePackage") }
            .package_answer(
                "RexxThreadInterface.GetRoutinePackage",
                routine.cast(),
                b"PACKAGE",
            )
            .cast()
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn method_package(
        context: *mut RexxThreadContext_,
        method: RexxMethodObject,
    ) -> RexxPackageObject {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "GetMethodPackage") }
            .package_answer(
                "RexxThreadInterface.GetMethodPackage",
                method.cast(),
                b"PACKAGE",
            )
            .cast()
    }

    /// # Safety
    /// As [`super::whole_number_to_object`].
    pub(super) unsafe extern "C" fn call_routine(
        context: *mut RexxThreadContext_,
        routine: RexxRoutineObject,
        arguments: RexxArrayObject,
    ) -> RexxObjectPtr {
        // SAFETY: as `whole_number_to_object`.
        unsafe { innermost_activation(context, "CallRoutine") }
            .call_routine(routine.cast(), arguments.cast())
    }

    /// # Safety
    /// As [`load_package`].
    pub(super) unsafe extern "C" fn call_program(
        context: *mut RexxThreadContext_,
        name: CSTRING,
        arguments: RexxArrayObject,
    ) -> RexxObjectPtr {
        // SAFETY: as `load_package`.
        let (activation, name) =
            unsafe { (innermost_activation(context, "CallProgram"), name_of(name)) };
        name.map_or(std::ptr::null_mut(), |name| {
            activation.call_program(name, arguments.cast())
        })
    }

    /// # Safety
    /// As [`load_package_from_data`].
    pub(super) unsafe extern "C" fn new_method(
        context: *mut RexxThreadContext_,
        name: CSTRING,
        source: CSTRING,
        length: usize,
    ) -> RexxMethodObject {
        // SAFETY: as `load_package_from_data`.
        let (activation, name, source) = unsafe {
            (
                innermost_activation(context, "NewMethod"),
                name_of(name),
                bytes_of(source, length),
            )
        };
        let (Some(name), Some(source)) = (name, source) else {
            return std::ptr::null_mut();
        };
        activation
            .new_executable("RexxThreadInterface.NewMethod", "Method", name, source)
            .cast()
    }

    /// # Safety
    /// As [`load_package_from_data`].
    pub(super) unsafe extern "C" fn new_routine(
        context: *mut RexxThreadContext_,
        name: CSTRING,
        source: CSTRING,
        length: usize,
    ) -> RexxRoutineObject {
        // SAFETY: as `load_package_from_data`.
        let (activation, name, source) = unsafe {
            (
                innermost_activation(context, "NewRoutine"),
                name_of(name),
                bytes_of(source, length),
            )
        };
        let (Some(name), Some(source)) = (name, source) else {
            return std::ptr::null_mut();
        };
        activation
            .new_executable("RexxThreadInterface.NewRoutine", "Routine", name, source)
            .cast()
    }
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn request_global_reference(
    context: *mut RexxThreadContext_,
    object: RexxObjectPtr,
) -> RexxObjectPtr {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "RequestGlobalReference") }
        .global_reference("RexxThreadInterface.RequestGlobalReference", object)
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn release_global_reference(
    context: *mut RexxThreadContext_,
    object: RexxObjectPtr,
) {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "ReleaseGlobalReference") }
        .global_reference("RexxThreadInterface.ReleaseGlobalReference", object);
}

/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn release_local_reference(
    context: *mut RexxThreadContext_,
    object: RexxObjectPtr,
) {
    // SAFETY: as `whole_number_to_object`.
    unsafe { innermost_activation(context, "ReleaseLocalReference") }
        .release_local_reference(object);
}

/// # Safety
/// As [`whole_number_to_object`], a non-null `name` is NUL-terminated, and a
/// non-null `entry` is as [`crate::load::registered`] asks.
unsafe extern "C" fn register_library(
    context: *mut RexxThreadContext_,
    name: CSTRING,
    entry: *mut RexxPackageEntry,
) -> logical_t {
    // SAFETY: as `whole_number_to_object`; the caller guarantees `name`.
    let (activation, name) = unsafe {
        (
            innermost_activation(context, "RegisterLibrary"),
            name_of(name),
        )
    };
    let Some(name) = name else {
        return 0;
    };
    // SAFETY: the caller guarantees the entry.
    let library = unsafe { crate::load::registered(entry, &String::from_utf8_lossy(name)) };
    logical_t::from(activation.register_library(name, library))
}

/// # Safety
/// `context` is a thread context a live [`ThreadContext`] handed out.
unsafe extern "C" fn get_interpreter_instance(
    context: *mut RexxThreadContext_,
) -> *mut RexxInstance_ {
    // SAFETY: the caller guarantees the context, whose `instance` field its
    // `ThreadContext` wrote.
    unsafe { (*context).instance }
}

/// `AttachThread` from the thread the interpreter runs on while a native
/// call is in flight: that thread is attached already, so the context it
/// answers is the one the running call has.
///
/// # Panics
/// Called from another thread, or with no native call in flight, which is
/// embedding.
///
/// # Safety
/// `instance` is an instance a live [`ThreadContext`] links, and a non-null
/// `attached` is valid for a write.
unsafe extern "C" fn attach_thread(
    instance: *mut RexxInstance_,
    attached: *mut *mut RexxThreadContext_,
) -> logical_t {
    // SAFETY: the caller guarantees `instance` is the `instance` field of a
    // live `Thread`, and it was taken from that whole allocation, so stepping
    // back to the allocation's start stays inside its provenance.
    let thread = unsafe {
        instance
            .cast::<u8>()
            .sub(std::mem::offset_of!(Thread, instance))
            .cast::<Thread>()
    };
    let refused = "RexxInstanceInterface.AttachThread is not implemented (Phase 9)";
    // SAFETY: as above; `home` is written only when the thread is made, so
    // any thread may read it.
    let home = unsafe { (*thread).home };
    assert!(home == std::thread::current().id(), "{refused}");
    // SAFETY: as above, and this is the home thread, the only one that
    // touches the innermost cell.
    let idle = unsafe { (*thread).innermost.0.get().is_null() };
    assert!(!idle, "{refused}");
    if !attached.is_null() {
        // SAFETY: the caller guarantees the write; the context is the
        // allocation's own `thread` field.
        unsafe { attached.write((&raw mut (*thread).thread).cast()) };
    }
    1
}

/// `DetachThread` of the context [`attach_thread`] answered, which the
/// running call still holds, so nothing is released.
///
/// # Safety
/// As [`whole_number_to_object`].
unsafe extern "C" fn detach_thread(context: *mut RexxThreadContext_) {
    // SAFETY: as `whole_number_to_object`; the call aborts where no native
    // call is in flight.
    unsafe { innermost_activation(context, "DetachThread") };
}

/// # Safety
/// As [`set_object_variable`].
unsafe extern "C" fn allocate_object_memory(
    context: *mut RexxMethodContext_,
    size: usize,
) -> POINTER {
    // SAFETY: as `set_object_variable`.
    unsafe { activation_of(context) }.allocate_object_memory(size)
}

/// # Safety
/// As [`set_object_variable`].
unsafe extern "C" fn free_object_memory(context: *mut RexxMethodContext_, pointer: POINTER) {
    // SAFETY: as `set_object_variable`.
    unsafe { activation_of(context) }.free_object_memory(pointer);
}

/// # Safety
/// As [`set_object_variable`].
unsafe extern "C" fn reallocate_object_memory(
    context: *mut RexxMethodContext_,
    pointer: POINTER,
    size: usize,
) -> POINTER {
    // SAFETY: as `set_object_variable`.
    unsafe { activation_of(context) }.reallocate_object_memory(pointer, size)
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
pub(crate) extern "C-unwind" fn reading_stub(
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
pub(crate) extern "C-unwind" fn dropping_stub(
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
pub(crate) extern "C-unwind" fn thread_table_stub(
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
pub(crate) extern "C-unwind" fn instance_stub(
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
pub(crate) extern "C-unwind" fn refusing_stub(
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

/// A method stub that reaches `ValuesToObject`, which a host with no
/// [`crate::callbacks::Surface`] refuses, then
/// `WholeNumberToObject`, which the host a nesting test builds answers by
/// running a native call of its own, then raises [`STUB_CONDITION`].
#[cfg(test)]
pub(crate) extern "C-unwind" fn nesting_stub(
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
        (table.ValuesToObject)(thread, std::ptr::null_mut(), 0);
        (table.WholeNumberToObject)(thread, 3);
        (table.RaiseException0)(thread, STUB_CONDITION);
    }
    std::ptr::null_mut()
}

/// The signature [`cstring_stub`] publishes: a `CSTRING` result and no
/// parameter.
#[cfg(test)]
static CSTRING_TYPES: [u16; 2] = [
    crate::values::code::CSTRING,
    crate::values::ARGUMENT_TERMINATOR,
];

/// The bytes [`cstring_stub`] answers a pointer to, a NUL inside them.
#[cfg(test)]
pub(crate) static STUB_TEXT: [u8; 15] = *b"answered\0after\0";

/// A method stub answering [`STUB_TEXT`] as its `CSTRING` result.
#[cfg(test)]
pub(crate) extern "C-unwind" fn cstring_stub(
    _context: *mut crate::layout::RexxMethodContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    if arguments.is_null() {
        return CSTRING_TYPES.as_ptr().cast_mut();
    }
    // SAFETY: element zero is this call's result descriptor.
    unsafe { (*arguments).value.value_CSTRING = STUB_TEXT.as_ptr().cast() };
    std::ptr::null_mut()
}

/// A method stub answering a null `CSTRING` result, which it leaves as the
/// zeroed word it was handed.
#[cfg(test)]
pub(crate) extern "C-unwind" fn null_cstring_stub(
    _context: *mut crate::layout::RexxMethodContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    if arguments.is_null() {
        return CSTRING_TYPES.as_ptr().cast_mut();
    }
    std::ptr::null_mut()
}

/// The signature [`arglist_stub`] publishes: an `int` result, an `int`
/// parameter, then the argument list.
#[cfg(test)]
static ARGLIST_TYPES: [u16; 4] = [
    crate::values::code::INT,
    crate::values::code::INT,
    crate::values::code::ARGLIST,
    crate::values::ARGUMENT_TERMINATOR,
];

/// A method stub answering its `int` argument.
#[cfg(test)]
pub(crate) extern "C-unwind" fn arglist_stub(
    _context: *mut crate::layout::RexxMethodContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    if arguments.is_null() {
        return ARGLIST_TYPES.as_ptr().cast_mut();
    }
    // SAFETY: the array is `invoke::run`'s, whose element one this signature
    // declares an `int` that `values::descriptor` wrote in full.
    unsafe { (*arguments).value.value_int = (*arguments.add(1)).value.value_int };
    std::ptr::null_mut()
}

/// [`arglist_stub`] as a routine.
#[cfg(test)]
pub(crate) extern "C-unwind" fn arglist_routine_stub(
    _context: *mut RexxCallContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    arglist_stub(std::ptr::null_mut(), arguments)
}

/// The signature [`name_result_stub`] publishes: `NAME` as its result type.
#[cfg(test)]
static NAME_RESULT_TYPES: [u16; 2] = [
    crate::values::code::NAME,
    crate::values::ARGUMENT_TERMINATOR,
];

/// A method stub declaring `NAME` as its result type and writing a `CSTRING`
/// into it, which nothing reads.
#[cfg(test)]
pub(crate) extern "C-unwind" fn name_result_stub(
    _context: *mut crate::layout::RexxMethodContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    if arguments.is_null() {
        return NAME_RESULT_TYPES.as_ptr().cast_mut();
    }
    // SAFETY: element zero is this call's result descriptor.
    unsafe { (*arguments).value.value_CSTRING = STUB_TEXT.as_ptr().cast() };
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
pub(crate) extern "C-unwind" fn numeric_stub(
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
thread_local! {
    /// The thread context [`stashing_stub`] was last handed.
    static STASH: std::cell::Cell<*mut RexxThreadContext_> =
        const { std::cell::Cell::new(std::ptr::null_mut()) };
}

/// A method stub that keeps the thread context its method context links, as
/// an extension may.
#[cfg(test)]
pub(crate) extern "C-unwind" fn stashing_stub(
    context: *mut crate::layout::RexxMethodContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    if arguments.is_null() {
        return DROPPING_TYPES.as_ptr().cast_mut();
    }
    // SAFETY: `context` is the one a `Contexts` handed this call.
    STASH.set(unsafe { (*context).threadContext });
    std::ptr::null_mut()
}

/// The thread context [`stashing_stub`] kept.
#[cfg(test)]
pub(crate) fn stashed() -> *mut RexxThreadContext_ {
    STASH.get()
}

/// The value [`stash_using_stub`] hands `WholeNumberToObject`.
#[cfg(test)]
pub(crate) const STASHED_NUMBER: isize = 42;

/// A method stub answering what `WholeNumberToObject` builds for
/// [`STASHED_NUMBER`] through the thread context [`stashing_stub`] kept,
/// rather than through the one its own method context links.
#[cfg(test)]
pub(crate) extern "C-unwind" fn stash_using_stub(
    _context: *mut crate::layout::RexxMethodContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    if arguments.is_null() {
        return NUMERIC_TYPES.as_ptr().cast_mut();
    }
    let thread = STASH.get();
    // SAFETY: `thread` is a thread context a `ThreadContext` handed out, which
    // the test keeps alive, and element zero is this call's result.
    unsafe {
        let object = ((*(*thread).functions).WholeNumberToObject)(thread, STASHED_NUMBER);
        (*arguments).value.value_RexxObjectPtr = object;
    }
    std::ptr::null_mut()
}

/// A method stub that reaches `WholeNumberToObject`, which the host a nesting
/// test builds answers by running a native call of its own, and then raises
/// [`STUB_CONDITION`], both through its thread context.
#[cfg(test)]
pub(crate) extern "C-unwind" fn nest_then_raise_stub(
    context: *mut crate::layout::RexxMethodContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    if arguments.is_null() {
        return DROPPING_TYPES.as_ptr().cast_mut();
    }
    // SAFETY: as `refusing_stub`.
    unsafe {
        let thread = (*context).threadContext;
        let table = &*(*thread).functions;
        (table.WholeNumberToObject)(thread, 3);
        (table.RaiseException0)(thread, STUB_CONDITION);
    }
    std::ptr::null_mut()
}

#[cfg(test)]
thread_local! {
    /// What [`throwing_stub`] reached: its guard's drop, and the code after
    /// its `ThrowException0`.
    pub(crate) static THROWN: std::cell::Cell<(bool, bool)> =
        const { std::cell::Cell::new((false, false)) };
}

/// A method stub that throws [`STUB_CONDITION`] through its method context
/// with a guard alive, and records in [`THROWN`] what ran.
#[cfg(test)]
pub(crate) extern "C-unwind" fn throwing_stub(
    context: *mut crate::layout::RexxMethodContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            THROWN.set((true, THROWN.get().1));
        }
    }
    if arguments.is_null() {
        return DROPPING_TYPES.as_ptr().cast_mut();
    }
    let _guard = Guard;
    // SAFETY: as `refusing_stub`.
    unsafe {
        let table = &*(*context).functions;
        (table.ThrowException0)(context, STUB_CONDITION);
    }
    THROWN.set((THROWN.get().0, true));
    std::ptr::null_mut()
}

#[cfg(test)]
thread_local! {
    /// The thread context [`raising_hook`] or [`refusing_hook`] was last
    /// handed.
    static HOOKED: std::cell::Cell<*mut RexxThreadContext_> =
        const { std::cell::Cell::new(std::ptr::null_mut()) };
}

/// The thread context a hook stub was last handed.
#[cfg(test)]
pub(crate) fn hooked() -> *mut RexxThreadContext_ {
    HOOKED.get()
}

/// A package hook that keeps the thread context it is handed and raises
/// [`STUB_CONDITION`] through it.
#[cfg(test)]
pub(crate) extern "C" fn raising_hook(thread: *mut RexxThreadContext_) {
    HOOKED.set(thread);
    // SAFETY: `thread` is the context `invoke::hook` handed this call.
    unsafe { ((*(*thread).functions).RaiseException0)(thread, STUB_CONDITION) };
}

/// A package hook that reaches `ValuesToObject`, which a host with no
/// [`crate::callbacks::Surface`] refuses, and then raises [`STUB_CONDITION`].
#[cfg(test)]
pub(crate) extern "C" fn refusing_hook(thread: *mut RexxThreadContext_) {
    HOOKED.set(thread);
    // SAFETY: as `raising_hook`; the name is a literal.
    unsafe {
        let table = &*(*thread).functions;
        (table.ValuesToObject)(thread, std::ptr::null_mut(), 0);
        (table.RaiseException0)(thread, STUB_CONDITION);
    }
}

#[cfg(test)]
mod tests {
    use super::{ThreadContext, owner_of, value_of};
    use crate::callbacks::fake::FakeHost;
    use crate::layout::{
        Owned, RexxCallContext_, RexxMethodContext_, RexxThreadContext_, RexxThreadInterface,
        ValueDescriptor, ValueUnion, recording_refusals,
    };
    use crate::values::{
        self, CStringPool, Conversion, Converted, Repr, Value, code, descriptor, repr, rows,
    };

    /// Runs `body` with the thread context and table of a native call `host`
    /// serves, recording the first member it refused.
    fn with_thread<R>(
        host: &mut FakeHost,
        body: impl FnOnce(*mut RexxThreadContext_, &RexxThreadInterface) -> R,
    ) -> (R, Option<&'static str>) {
        let mut strings = CStringPool::new();
        let activation = values::Activation::new(Conversion {
            host,
            strings: &mut strings,
        });
        ThreadContext::new().enter(&activation, |contexts| {
            let context = contexts.method().as_ptr();
            // SAFETY: `context` is the one this `Contexts` built, which links
            // the thread context and its table.
            let (thread, table) = unsafe {
                let thread = (*context).threadContext;
                (thread, &*(*thread).functions)
            };
            recording_refusals(|| body(thread, table))
        })
    }

    /// A descriptor of `code` over `value`.
    fn described(code: u16, value: ValueUnion) -> ValueDescriptor {
        ValueDescriptor {
            value,
            r#type: code,
            flags: 0,
        }
    }

    /// **The construction members answer through the thread table**: a
    /// string from a length and from a terminator, a number read into a
    /// caller's variable, a descriptor's value as an object and an object as
    /// a descriptor's value, and an array of descriptors.
    #[test]
    fn the_construction_members_answer_through_the_thread_table() {
        let mut host = FakeHost::new();
        let seven = host.text(b"7");
        let seven = host.locals.register(seven);
        let letters = host.text(b"abc");
        let letters = host.locals.register(letters);
        let (answers, refused) = with_thread(&mut host, |thread, table| {
            let mut number = 0i32;
            let mut hello = described(
                code::CSTRING,
                ValueUnion {
                    value_CSTRING: c"hello".as_ptr(),
                },
            );
            let mut int = described(code::INT, ValueUnion { value_int64_t: 0 });
            let mut refused = described(code::INT, ValueUnion { value_int64_t: 0 });
            let mut pair = [
                described(code::INT, ValueUnion { value_int: 5 }),
                described(
                    code::CSTRING,
                    ValueUnion {
                        value_CSTRING: c"q".as_ptr(),
                    },
                ),
            ];
            // SAFETY: every pointer is to a live local or a literal, and each
            // descriptor's member is the one its code names.
            unsafe {
                let string = (table.NewString)(thread, c"abcdef".as_ptr(), 3);
                let asciiz = (table.NewStringFromAsciiz)(thread, c"xyz".as_ptr());
                let fits = (table.ObjectToInt32)(thread, seven, &raw mut number);
                let value = (table.ValueToObject)(thread, &raw mut hello);
                let converted = (table.ObjectToValue)(thread, seven, &raw mut int);
                let rejected = (table.ObjectToValue)(thread, letters, &raw mut refused);
                let array = (table.ValuesToObject)(thread, pair.as_mut_ptr(), 2);
                (
                    string,
                    asciiz,
                    (fits, number),
                    value,
                    (converted, int.value.value_int),
                    rejected,
                    array,
                )
            }
        });
        assert_eq!(refused, None);
        let (string, asciiz, number, value, int, rejected, array) = answers;
        let bytes = |host: &FakeHost, handle| host.bytes(host.locals.resolve(handle)?);
        assert_eq!(bytes(&host, string.cast()), Some(b"abc".to_vec()));
        assert_eq!(bytes(&host, asciiz.cast()), Some(b"xyz".to_vec()));
        assert_eq!(number, (1, 7));
        assert_eq!(bytes(&host, value), Some(b"hello".to_vec()));
        assert_eq!(int, (1, 7));
        assert_eq!(rejected, 0);
        assert_eq!(
            host.cleared, 1,
            "the refused conversion's condition was kept"
        );
        let items = host
            .items(
                host.locals
                    .resolve(array.cast())
                    .expect("the array is registered"),
            )
            .expect("an array");
        let items: Vec<Option<Vec<u8>>> = items.into_iter().map(|item| host.bytes(item?)).collect();
        assert_eq!(items, [Some(b"5".to_vec()), Some(b"q".to_vec())]);
    }

    /// **The condition members reach the host's held condition**: a raise
    /// with substitutions hands the host an array of them, `CheckCondition`
    /// and `ClearCondition` read and clear it, `GetConditionInfo` answers
    /// its object, and `DecodeConditionInfo` reads a directory's entries.
    #[test]
    fn the_condition_members_hold_read_and_clear_the_hosts_condition() {
        use crate::callbacks::fake::Held;
        use crate::layout::RexxCondition;

        let mut host = FakeHost::new();
        host.displayed = 40;
        let sub = host.text(b"sub");
        let sub = host.locals.register(sub);
        let directory = host.text(b"directory");
        host.condition = directory;
        for (name, text) in [
            (&b"CODE"[..], &b"88.917"[..]),
            (b"RC", b"88"),
            (b"POSITION", b"3"),
            (b"CONDITION", b"SYNTAX"),
            (b"MESSAGE", b"a message"),
        ] {
            let value = host.text(text);
            host.entries.push((directory, name.to_vec(), value));
        }
        let (answers, refused) = with_thread(&mut host, |thread, table| {
            let mut decoded = std::mem::MaybeUninit::<RexxCondition>::uninit();
            // SAFETY: every handle is registered or answered by the table,
            // the name is a literal, and `decoded` is written in full.
            unsafe {
                (table.RaiseException1)(thread, 88_917, sub);
                let raised = (table.CheckCondition)(thread);
                let info = (table.GetConditionInfo)(thread);
                (table.DecodeConditionInfo)(thread, info, decoded.as_mut_ptr());
                let displayed = (table.DisplayCondition)(thread);
                (table.ClearCondition)(thread);
                let cleared = (table.CheckCondition)(thread);
                (table.RaiseCondition)(
                    thread,
                    c"user thing".as_ptr(),
                    std::ptr::null_mut(),
                    sub,
                    std::ptr::null_mut(),
                );
                (raised, info, decoded.assume_init(), displayed, cleared)
            }
        });
        assert_eq!(refused, None);
        let (raised, info, decoded, displayed, cleared) = answers;
        assert_eq!((raised, cleared, displayed), (1, 0, 40));
        assert_eq!(host.locals.resolve(info.cast()), Some(directory));
        assert_eq!(
            (decoded.code, decoded.rc, decoded.position),
            (88_917, 88, 3)
        );
        let bytes = |handle: *mut crate::layout::RexxStringObject_| {
            host.bytes(host.locals.resolve(handle.cast())?)
        };
        assert_eq!(bytes(decoded.conditionName), Some(b"SYNTAX".to_vec()));
        assert_eq!(bytes(decoded.message), Some(b"a message".to_vec()));
        assert!(decoded.errortext.is_null() && decoded.additional.is_null());
        let sub = host.locals.resolve(sub);
        assert_eq!(
            host.held,
            Some(Held::Named(b"USER THING".to_vec(), [None, sub, None]))
        );
    }

    /// A raise's substitutions reach the host as one array, in order.
    #[test]
    fn a_raise_hands_the_host_its_substitutions_as_an_array() {
        use crate::callbacks::fake::Held;

        let mut host = FakeHost::new();
        let (first, second) = (host.text(b"first"), host.text(b"second"));
        let (first, second) = (host.locals.register(first), host.locals.register(second));
        let ((), refused) = with_thread(&mut host, |thread, table| {
            // SAFETY: both handles are registered.
            unsafe { (table.RaiseException2)(thread, 93_903, first, second) };
        });
        assert_eq!(refused, None);
        let Some(Held::Syntax(93_903, Some(array))) = host.held else {
            panic!("the host holds {:?}", host.held);
        };
        let items: Vec<Option<Vec<u8>>> = host
            .items(array)
            .expect("an array")
            .into_iter()
            .map(|item| host.bytes(item?))
            .collect();
        assert_eq!(items, [Some(b"first".to_vec()), Some(b"second".to_vec())]);
    }

    /// **The string and buffer members copy out and write through the
    /// addresses they answer**: a string's bytes from an offset, a buffer
    /// string finished shorter than it was made, a buffer's bytes, and a
    /// mutable buffer grown, lengthened and read back.
    #[test]
    fn the_string_and_buffer_members_copy_and_write_through_their_addresses() {
        let mut host = FakeHost::new();
        let source = host.text(b"abcdef");
        let source = host.locals.register(source);
        let ((copied, window, finished, buffer, grown), refused) =
            with_thread(&mut host, |thread, table| {
                let mut window = [0u8; 4];
                // SAFETY: every handle is registered or answered by the table,
                // each address written is one the table answered for at least
                // the bytes written, and `window` has room for four.
                unsafe {
                    let copied =
                        (table.StringGet)(thread, source.cast(), 3, window.as_mut_ptr().cast(), 4);
                    let string = (table.NewBufferString)(thread, 5);
                    let data = (table.BufferStringData)(thread, string).cast::<u8>();
                    std::ptr::copy_nonoverlapping(b"xyz".as_ptr(), data, 3);
                    let finished = (table.FinishBufferString)(thread, string, 3);
                    let buffer = (table.NewBuffer)(thread, 4);
                    (table.BufferData)(thread, buffer)
                        .cast::<u8>()
                        .write_bytes(7, (table.BufferLength)(thread, buffer));
                    let grown = (table.NewMutableBuffer)(thread, 2);
                    // `ensureCapacity` is asked for the difference over the
                    // capacity, six, which over an empty buffer is six.
                    let data = (table.SetMutableBufferCapacity)(thread, grown, 8).cast::<u8>();
                    (table.SetMutableBufferLength)(thread, grown, 6);
                    std::ptr::copy_nonoverlapping(b"mutabl".as_ptr(), data, 6);
                    (copied, window, finished, buffer, grown)
                }
            });
        assert_eq!(refused, None);
        assert_eq!((copied, &window), (4, b"cdef"));
        let finished = host.locals.resolve(finished.cast()).expect("registered");
        assert_eq!(host.bytes(finished), Some(b"xyz".to_vec()));
        let buffer = host.locals.resolve(buffer.cast()).expect("registered");
        let state = host.state(buffer).expect("a buffer");
        assert_eq!(state.data(), Some(&[7u8; 4][..]));
        let grown = host.locals.resolve(grown.cast()).expect("registered");
        let state = host.state(grown).expect("a mutable buffer");
        let contents = state.buffer().expect("a mutable buffer");
        assert_eq!(
            (contents.bytes.as_slice(), contents.capacity),
            (&b"mutabl"[..], 6)
        );
    }

    /// **A copied mutable buffer's bytes are writable up to the capacity it
    /// answers**, though the copy's contents were cloned at their length.
    #[test]
    fn a_copied_mutable_buffer_is_writable_to_its_capacity() {
        let mut host = FakeHost::new();
        let original = crate::callbacks::Surface::new_mutable_buffer(&mut host, 64);
        crate::callbacks::Surface::set_mutable_buffer_length(&mut host, original, 3);
        let copy = host.copy(original).expect("an object");
        let copy = host.locals.register(copy);
        let (capacity, refused) = with_thread(&mut host, |thread, table| {
            // SAFETY: `copy` is registered, and the address is written only
            // up to the capacity answered with it.
            unsafe {
                let data = (table.MutableBufferData)(thread, copy.cast()).cast::<u8>();
                let capacity = (table.MutableBufferCapacity)(thread, copy.cast());
                data.write_bytes(b'z', capacity);
                capacity
            }
        });
        assert_eq!((capacity, refused), (64, None));
        let copy = host.locals.resolve(copy).expect("registered");
        let state = host.state(copy).expect("a mutable buffer");
        let contents = state.buffer().expect("a mutable buffer");
        assert!(contents.bytes.capacity() >= contents.capacity);
        assert_eq!(contents.bytes, b"zzz");
    }

    /// A member that needs the host's surface refuses on a host with none,
    /// naming itself.
    #[test]
    fn a_surface_member_on_a_host_without_one_refuses() {
        let mut host = FakeHost::new();
        host.serves = false;
        let (array, refused) = with_thread(&mut host, |thread, table| {
            // SAFETY: an empty list reads no descriptor.
            unsafe { (table.ValuesToObject)(thread, std::ptr::null_mut(), 0) }
        });
        assert!(array.is_null());
        assert_eq!(refused, Some("RexxThreadInterface.ValuesToObject"));
    }

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

    /// The variable this test sets on the child it spawns.
    const CALL_WITH_NO_FRAME: &str = "REXX_API_CALL_WITH_NO_NATIVE_CALL";

    /// A thread context called through with no native call in flight ends
    /// the process naming the member, which is the oracle's rc 134.
    #[test]
    #[cfg_attr(miri, ignore = "spawns a process")]
    fn a_thread_context_with_no_call_in_flight_aborts() {
        if std::env::var_os(CALL_WITH_NO_FRAME).is_some() {
            let thread = super::ThreadContext::new();
            let context = thread.pointer();
            // SAFETY: `context` is the live `thread`'s, and the table is its own.
            unsafe { ((*(*context).functions).WholeNumberToObject)(context, 42) };
            unreachable!("the member returned");
        }

        let binary = std::env::current_exe().expect("this test binary's own path");
        let output = std::process::Command::new(binary)
            .args([
                "ffi::tests::a_thread_context_with_no_call_in_flight_aborts",
                "--exact",
                "--nocapture",
            ])
            .env(CALL_WITH_NO_FRAME, "1")
            .output()
            .expect("the child runs");

        assert_eq!(
            std::os::unix::process::ExitStatusExt::signal(&output.status),
            Some(6),
            "the child did not abort: {:?}",
            output.status
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(
                "RexxThreadInterface.WholeNumberToObject was called with no native call in flight"
            ),
            "the abort did not name the member:\n{stderr}"
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
