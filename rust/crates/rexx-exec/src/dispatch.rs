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

//! Message dispatch: the object model this crate resolves against, the
//! `resolve`/`invoke` pair, and the one chokepoint every invocation passes.
//!
//! **`resolve` and `invoke` are two steps, not one fused send** (D24). They
//! are separate functions with separate signatures for the same reason
//! `Interp::resolve_call` and `Interp::invoke_call` are: the resolution's
//! inputs and the invocation's are different, and a construct that has one
//! already in hand must be able to enter the other on its own.
//! [`Interp::send_message`] is their composition and not a third step --
//! `exec_call`'s relation to the call pair exactly. What it adds around them
//! is the search order's tail: a name `resolve` does not find reaches
//! [`Interp::unknown_or_nomethod`], which forwards to the receiver's own
//! `UNKNOWN` or raises the condition beneath it.
//!
//! **Resolution is dynamic and nothing caches it** (D28). Every send resolves
//! against the receiver's *current* behaviour; `crate::ir::CallSite` is a
//! classic-call cache and no send goes near it. `ClassGraph` carries a
//! per-behaviour monotonic version for a future cache to guard on, and this
//! module reads it nowhere.
//!
//! # The security seam (D45, site one of two)
//!
//! The oracle sends a protected method through
//! `RexxObject::processProtectedMethod` (`ObjectClass.cpp:976`), which asks
//! `getEffectiveSecurityManager()->checkProtectedMethod` before running
//! anything. There is no manager object in this phase and nothing installs
//! one; what this module owes is the **place** one would hook, and the
//! guarantee that there is exactly one of them.
//!
//! `PROTECTED` is what routes through it. [`seam::clear`] asks
//! [`Interp::method_is_protected`] and, for a method that is, puts the
//! manager's question at [`Interp::check_protected_method`] -- so the seam is
//! where the one access scope the oracle asks a manager about is decided,
//! rather than a place nothing reaches. With no manager the answer is
//! permission, so no program can tell the branch from its absence:
//! `tests/dispatch_seam.rs` pins the placement lexically and says so.
//!
//! # The access scopes
//!
//! `PRIVATE` and `PACKAGE` are decided in [`Interp::resolve`] and not at the
//! seam, because the oracle refuses the **lookup** rather than the
//! invocation: `messageSend` drops the method it found and enters
//! `processUnknown` with the refusal's own error code
//! (`ObjectClass.cpp:876`-`:889`, `:904`), so a refused private send reaches
//! the receiver's own `UNKNOWN` if it has one. Measured, oracle rc 0: a class
//! whose `m` is `PRIVATE` and which also answers `UNKNOWN` prints
//! `unknown saw M` for `.K~m` from outside. [`Miss`] is that error code and
//! [`Interp::check_private`] and [`Interp::check_package`] are the two
//! checks.
//!
//! **What the type system carries**: [`seam::Cleared`] has a private field
//! and is neither `Copy` nor `Clone`, and every function that runs a resolved
//! method takes one by value, so none of them runs without a value produced
//! inside [`mod seam`](seam). The compiler refuses the token's tuple-struct
//! constructor written anywhere else, `error[E0423]`.
//!
//! **What it does not carry** is how many producers that module holds: a
//! second `fn` inside it is as legal as the first. `tests/dispatch_seam.rs`
//! is what bounds that, by reading the module's items rather than by
//! counting two token spellings, and its own module doc states what the
//! reading still cannot see.
//!
//! # The native method table
//!
//! [`NATIVE_METHODS`] names the primitive methods this phase implements. A
//! name in a class's dictionary with no row here **resolves** and then fails
//! loudly on invocation, naming the owning phase -- D37's rule for a native
//! entry point, applied to a native method: the oracle's own answer is a
//! working method, so a Rexx condition here would let a program expecting one
//! pass against a gap.
//!
//! # The invocable kinds, and the one clearance
//!
//! [`Invocable`] is what a resolved [`rexx_classes::MethodId`] names, one
//! variant per table [`Interp::invocable`] reads, and its own doc says what
//! separates them. [`Interp::invoke`] picks between them **after** the seam
//! has been passed: a kind with a body of its own takes the [`Cleared`] token
//! by value, and a kind `invoke` answers inline is answered by a function
//! that already holds one. [`Interp::enter_method_body`] is written here
//! rather than beside the other activation machinery in `run.rs` for exactly
//! that reason: `Cleared` is private to this module, so a function that takes
//! one has to live here.

use std::collections::HashMap;
use std::rc::Rc;

use rexx_classes::{ClassKind, ClassRegistry, InheritRefusal, MethodId, MethodSlot};
use rexx_core::{
    BehaviourHandle, BehaviourId, Body, BufferState, Decoded, ObjRef, Object, ObjectMethod,
    ObjectMethods,
};
use rexx_parse::{Access, Expr, Operator};

use crate::activation::{
    Activation, DeferredReply, MethodIdentity, ReplyState, TraceEntry, body_of,
};
use crate::error::{FailureSite, Raised};
use crate::plan::{BodyKey, Package};
use crate::run::MAX_ACTIVATION_DEPTH;
use crate::{Failure, Interp, Loud};

/// The dispatch security seam.
///
/// Its own module so that [`Cleared`]'s field is private to the smallest
/// possible scope: nothing outside these few lines can build one, whatever
/// else `dispatch.rs` grows.
mod seam {
    use super::{Failure, Interp, MethodId, ObjRef};

    /// Evidence that a message send passed the dispatch security seam.
    ///
    /// Zero-sized, with a private field. [`clear`] is the only expression
    /// that can produce a value of this type, and every function that runs a
    /// resolved method takes one, so nothing runs without the seam having
    /// been passed first.
    pub(super) struct Cleared(());

    /// **The dispatch chokepoint (D45, site one).** Every method invocation
    /// passes here, whatever kind [`super::Invocable`] finds it to be, and a
    /// manager installed in a later phase gets its hook in this function's
    /// body.
    ///
    /// The oracle asks its manager only for a *protected* method:
    /// `RexxObject::messageSend` routes one through
    /// `processProtectedMethod` (`classes/ObjectClass.cpp:886`-`:889`), and
    /// every other method reaches `method->run` with nothing asked
    /// (`:896`-`:899`). So the question is asked here, which is the one place
    /// every invocation passes, and [`Interp::check_protected_method`] is the
    /// manager's own half of it.
    ///
    /// `PRIVATE` and `PACKAGE` are **not** asked here, and that is measured
    /// rather than chosen: they refuse the *lookup* and fall through to the
    /// receiver's own `UNKNOWN`, so [`Interp::resolve`] is where they belong.
    /// [`super::Miss`] carries the reading behind that.
    pub(super) fn clear(
        interp: &mut Interp,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
        method: MethodId,
    ) -> Result<Cleared, Failure> {
        if interp.method_is_protected(method) {
            interp.check_protected_method(receiver, name, args)?;
        }
        Ok(Cleared(()))
    }
}

use seam::Cleared;

// The `LIBRARY REXX` entry-point registry (D37): the names a
// `::METHOD ... EXTERNAL 'LIBRARY REXX name'` binds to at install, and which
// phase owes each of them a body. A child of this module rather than a
// sibling, because the table's rows point at `NativeMethod`s, whose
// parameter list names a type only this module can.
pub(crate) mod native;

// `String`'s primitive methods, whose rows are chained into
// `ObjectModel::build` beside `NATIVE_METHODS` rather than merged into it.
mod string;

// The collection classes' primitive methods, chained the same way.
mod collection;
pub(crate) mod hash;

/// One primitive method's implementation.
///
/// The [`Cleared`] parameter is the seam's own enforcement and is never read
/// by an implementation -- see this module's doc comment.
///
/// **`None` is a method that produced no value**, which is `OREF_NULL` off a
/// C++ method body: measured, `.environment~put('v','q')` is rc 0 as a whole
/// clause and 91.999 at rc 165 under `say`, exactly as a `::METHOD` body
/// ending in a bare `return` is.
type NativeMethod =
    fn(&mut Interp, Cleared, ObjRef, &[Option<ObjRef>]) -> Result<Option<ObjRef>, Failure>;

/// What a resolved [`MethodId`] runs.
///
/// The kinds are not interchangeable and are kept apart rather than
/// hidden behind one closure. What separates them is what [`Interp::invoke`]
/// has to do around the code: a declared argument count for the send to
/// check, a `Compiled method` traceback line to contribute, an activation to
/// push. `Native` and the implemented half of `External` are the kinds that
/// run compiled code: both take a count check and both contribute the
/// traceback line, and they differ in which error the count check raises,
/// which is measured on [`Raised::too_many_external_arguments`].
enum Invocable {
    Native(NativeEntry),
    Rexx(crate::InstalledMethodBody),
    Generated(crate::GeneratedMethod),
    /// A `::METHOD ... EXTERNAL 'LIBRARY REXX name'` bound at install to a
    /// row of [`native`]'s registry. Apart from the entry points this phase
    /// implements, running one is loud -- so this is the kind whose ordinary
    /// outcome is a refusal, which is why it is not folded into `Native`
    /// beside the primitives.
    External(&'static native::NativeExternal),
}

/// What [`Interp::invoke`] needs about one primitive method beyond its code.
#[derive(Copy, Clone)]
struct NativeEntry {
    arity: Arity,
    run: NativeMethod,
}

/// How many arguments an entry admits.
///
/// **Where the count comes from differs by entry kind.**
/// For a [`NativeEntry`] it is the second half of that method's `AddMethod`
/// row in `memory/Setup.cpp`; for a `native::NativeExternal` it is the `N` of
/// the `RexxMethod<N>` that defines the entry point, and the refusal is
/// 88.922 from inside `NativeActivation` rather than the 93.902 named below.
/// [`Raised::too_many_external_arguments`] carries that pair measured.
#[derive(Copy, Clone)]
enum Arity {
    /// A numeric count. For a primitive method `CPPCode::run` refuses more
    /// than this with 93.902 before the body is entered
    /// (`execution/CPPCode.cpp:151`-`:154`) and pads a shorter list out with
    /// nulls -- measured, `'abc'~length(1)` is `0 expected` and
    /// `'abc'~hasMethod('a','b')` is `1 expected`.
    Fixed(usize),
    /// `A_COUNT` (`execution/CPPCode.hpp:48`), which hands the body the whole
    /// argument list and count and refuses nothing here
    /// (`execution/CPPCode.cpp:144`-`:148`). The body's own check is what
    /// reports, and its error is not always 93.902 -- measured,
    /// `(1,2)~at(1,2)` is 93.926 `Too many subscripts for array; 1 expected.`
    /// where `.environment~at(1,2)`, whose row is a count, is 93.902.
    Counted,
}

/// The primitive methods every run of this crate implements, as (class id,
/// method name, declared parameter count, implementation).
///
/// [`SETUP_METHODS`] is the other table and is registered only while the
/// interpreter's own library is being run.
///
/// The class id is the `~id` string `rexx_classes::native_classes` registers,
/// and the method name is looked up in that class's **own** dictionary, so a
/// row here fixes the method's scope: `HASMETHOD` at `Object` is the one
/// entry a `String` receiver reaches too, because the flattened cascade
/// copies `Object`'s entry (its `MethodId` included) into `String`'s
/// behaviour.
static NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    // `ArrayClass::getRexx` under both of its names, `memory/Setup.cpp:713`
    // and `:715`, at `A_COUNT` in each row -- the same both-names-one-function
    // shape `MAKESTRING`/`TOSTRING` below have, and for the same reason: a
    // second function is where the two could come to disagree. Measured, the
    // two agree on every shape `array_list_expression.rex` and
    // `array_index_refusals.rex` ask.
    ("Array", "[]", Arity::Counted, native_array_at),
    ("Array", "AT", Arity::Counted, native_array_at),
    // `ArrayClass::putRexx` under both of its names, `memory/Setup.cpp:714`
    // and `:721`, the same shape again. `t[i] = v` sends `t~"[]="(v, i)`, so
    // the value leads the subscript list here too.
    ("Array", "[]=", Arity::Counted, native_array_put),
    ("Array", "PUT", Arity::Counted, native_array_put),
    // `ArrayClass::dimensionRexx`, `memory/Setup.cpp:716`, whose declared
    // count is 1 and whose argument is optional.
    (
        "Array",
        "DIMENSION",
        Arity::Fixed(1),
        native_array_dimension,
    ),
    ("Array", "ITEMS", Arity::Fixed(0), native_array_items),
    (
        "Array",
        "MAKESTRING",
        Arity::Fixed(2),
        native_array_make_string,
    ),
    // The same C++ function under a second name, `Setup.cpp:733`-`:734`
    // binding `ArrayClass::toString` twice with the same arity. Measured, the
    // two agree on every shape `array_make_string.rex` and
    // `array_make_string_refusals.rex` ask -- the default, `L` with and
    // without a separator, `C`, a lower-case option, an option omitted in
    // place, the empty list, 93.915 and the `C`-with-separator 93.902 -- and
    // the traceback frame names the message that was sent rather than the
    // implementation, so `~toString` reads `Compiled method "TOSTRING" with
    // scope "Array".` A second row here rather than a second function,
    // because a second function is where the two could come to disagree.
    (
        "Array",
        "TOSTRING",
        Arity::Fixed(2),
        native_array_make_string,
    ),
    ("Array", "SIZE", Arity::Fixed(0), native_array_size),
    // `HashCollection::initRexx`, in `Bag`'s own dictionary by
    // `InheritInstanceMethods(Relation)` (`memory/Setup.cpp:988`) out of
    // `IdentityTable`'s own row (`:841`). A donation puts the method in the
    // receiving class's own dictionary, which is what the scope in the
    // traceback says, so each donee needs a row of its own here.
    ("Bag", "INIT", Arity::Fixed(1), native_capacity_init),
    // `RexxObject::initRexx`, bound to `.Class` under this name by
    // `memory/Setup.cpp:497` whose own comment is "this is a NOP by default,
    // so we'll just use the object init method as a fill in". Every class
    // object answers it, because a class's class behaviour merges its
    // metaclass's instance behaviour, and `ClassDirective::activate`
    // (`instructions/ClassDirective.cpp:288`) sends it to every class the
    // package installed.
    ("Class", "ACTIVATE", Arity::Fixed(0), native_no_op),
    // `RexxClass::getAnnotationRexx` and `RexxClass::getAnnotations`
    // (`memory/Setup.cpp:499`, `:498`). The same pair is bound at `Method`
    // and `Routine` out of `BaseExecutable` (`:1112`/`:1111` and
    // `:1141`/`:1140`) and at `Package` out of `PackageClass` (`:1173`,
    // `:1172`); each C++ body differs from the others only in which field it
    // reaches for, so `Class`, `Method`, `Routine` and `Package` share one
    // implementation here for the reason `MAKESTRING`/`TOSTRING` share
    // theirs.
    ("Class", "ANNOTATION", Arity::Fixed(1), native_annotation),
    ("Class", "ANNOTATIONS", Arity::Fixed(0), native_annotations),
    ("Class", "BASECLASS", Arity::Fixed(0), native_base_class),
    (
        "Class",
        "DEFAULTNAME",
        Arity::Fixed(0),
        native_class_default_name,
    ),
    // The five mutators, each of which `Setup.cpp` declares
    // `AddProtectedMethod` (`:456`, `:457`, `:463`, `:478`) except `Inherit`
    // (`:466`), and each of which opens with the same `REXX_DEFINED` refusal.
    // `RexxClass::copyRexx` (`classes/ClassClass.cpp:166`), which is a
    // refusal and not a copy: `Setup.cpp:483` overrides `Object`'s row with
    // one that raises. Measured, oracle rc 163: `.K~copy` is `93.970 COPY
    // method is not supported for object The K class.`
    ("Class", "COPY", Arity::Fixed(0), native_class_copy),
    ("Class", "DEFINE", Arity::Fixed(2), native_define),
    (
        "Class",
        "DEFINEMETHODS",
        Arity::Fixed(1),
        native_define_methods,
    ),
    ("Class", "DELETE", Arity::Fixed(1), native_delete),
    ("Class", "ENHANCED", Arity::Counted, native_enhanced),
    ("Class", "HASHCODE", Arity::Fixed(0), native_hash_code),
    ("Class", "ID", Arity::Fixed(0), native_id),
    ("Class", "INHERIT", Arity::Fixed(2), native_class_inherit),
    (
        "Class",
        "ISSUBCLASSOF",
        Arity::Fixed(1),
        native_is_subclass_of,
    ),
    ("Class", "METACLASS", Arity::Fixed(0), native_metaclass),
    ("Class", "METHOD", Arity::Fixed(1), native_method),
    // `RexxClass::mixinClassRexx` and `RexxClass::subclassRexx`
    // (`memory/Setup.cpp:470`, `:474`), the class factory a program reaches
    // by message. Neither carries the `REXX_DEFINED` refusal the mutators
    // above open with: `RexxClass::subclass` does not test the flag, and what
    // it builds does not carry it either -- measured, oracle rc 0,
    // `.object~subclass("k")~inherit(.object~mixinclass("mx"))`.
    (
        "Class",
        "MIXINCLASS",
        Arity::Fixed(3),
        native_mixin_class_factory,
    ),
    ("Class", "PACKAGE", Arity::Fixed(0), native_package),
    ("Class", "SUBCLASS", Arity::Fixed(3), native_subclass),
    ("Class", "SUPERCLASS", Arity::Fixed(0), native_superclass),
    (
        "Class",
        "SUPERCLASSES",
        Arity::Fixed(0),
        native_superclasses,
    ),
    ("Class", "UNINHERIT", Arity::Fixed(1), native_uninherit),
    // `Directory`'s own row (`memory/Setup.cpp:935`), which names
    // `DirectoryClass::initRexx` -- the inherited `HashCollection::initRexx`,
    // since `DirectoryClass` declares none of its own.
    ("Directory", "INIT", Arity::Fixed(1), native_capacity_init),
    // `HashCollection::getRexx` under both of its names and
    // `HashCollection::putRexx`, donated to `Directory` by
    // `InheritInstanceMethods(StringTable)` (`memory/Setup.cpp:933`) out of
    // `IdentityTable`'s own rows (`:825`, `:828`, `:831`). The donation puts
    // them in `Directory`'s **own** dictionary, which is what the scope in the
    // traceback says: measured, `.environment~at()` reports `Compiled method
    // "AT" with scope "Directory".`, not `IdentityTable`.
    ("Directory", "[]", Arity::Fixed(1), native_hash_at),
    // `HashCollection::putRexx` under its second name, donated from
    // `IdentityTable` (`memory/Setup.cpp:826`) and so the same function with
    // the same argument order: `t[i] = v` sends `t~"[]="(v, i)`.
    ("Directory", "[]=", Arity::Fixed(2), native_hash_put),
    ("Directory", "AT", Arity::Fixed(1), native_hash_at),
    ("Directory", "PUT", Arity::Fixed(2), native_hash_put),
    // `StringHashCollection::unknownRexx`, `StringTable`'s own row
    // (`memory/Setup.cpp:883`) and donated to `Directory` by the same
    // `InheritInstanceMethods`. This is the entry-method mechanism: an entry
    // is reached by sending its name.
    ("Directory", "UNKNOWN", Arity::Fixed(2), native_hash_unknown),
    // `EventSemaphoreClass::close` (`memory/Setup.cpp:1330`), which closes an
    // operating system semaphore this crate never opened and answers no
    // value. A row here rather than nothing, because the class declares
    // `UNINIT` and so every instance of it is registered for the finalizer
    // sweep, where a method with no row is a loud refusal at collection time
    // rather than at the send.
    ("EventSemaphore", "UNINIT", Arity::Fixed(0), native_no_op),
    // `HashCollection::initRexx` at the class that declares it
    // (`memory/Setup.cpp:841`), and `ListClass::initRexx` (`:1011`), which
    // differs from it only in the default capacity.
    (
        "IdentityTable",
        "INIT",
        Arity::Fixed(1),
        native_capacity_init,
    ),
    (
        "Message",
        "COMPLETED",
        Arity::Fixed(0),
        native_message_completed,
    ),
    (
        "Message",
        "HASERROR",
        Arity::Fixed(0),
        native_message_has_error,
    ),
    ("Message", "RESULT", Arity::Fixed(0), native_message_result),
    ("Method", "ANNOTATION", Arity::Fixed(1), native_annotation),
    ("Method", "ANNOTATIONS", Arity::Fixed(0), native_annotations),
    // `MethodClass::getScopeRexx`, `memory/Setup.cpp:1113`. `Routine` and
    // `Package` carry no such row: `Setup.cpp` binds `Scope` at `Method`
    // alone, and the scope is a `MethodClass` field rather than a
    // `BaseExecutable` one (`classes/MethodClass.hpp:168`).
    ("Method", "SCOPE", Arity::Fixed(0), native_scope),
    // `MutableBuffer`'s rows in `memory/Setup.cpp:1416`-`:1479`, each at its
    // declared count.
    (
        "MutableBuffer",
        "[]",
        Arity::Fixed(2),
        native_mutable_buffer_brackets,
    ),
    (
        "MutableBuffer",
        "[]=",
        Arity::Fixed(3),
        native_mutable_buffer_bracketsequal,
    ),
    (
        "MutableBuffer",
        "APPEND",
        Arity::Counted,
        native_mutable_buffer_append,
    ),
    (
        "MutableBuffer",
        "CASELESSCHANGESTR",
        Arity::Fixed(3),
        native_mutable_buffer_caselesschangestr,
    ),
    (
        "MutableBuffer",
        "CASELESSCONTAINS",
        Arity::Fixed(3),
        native_mutable_buffer_caselesscontains,
    ),
    (
        "MutableBuffer",
        "CASELESSCONTAINSWORD",
        Arity::Fixed(2),
        native_mutable_buffer_caselesscontainsword,
    ),
    (
        "MutableBuffer",
        "CASELESSCOUNTSTR",
        Arity::Fixed(1),
        native_mutable_buffer_caselesscountstr,
    ),
    (
        "MutableBuffer",
        "CASELESSENDSWITH",
        Arity::Fixed(1),
        native_mutable_buffer_caselessendswith,
    ),
    (
        "MutableBuffer",
        "CASELESSLASTPOS",
        Arity::Fixed(3),
        native_mutable_buffer_caselesslastpos,
    ),
    (
        "MutableBuffer",
        "CASELESSMATCH",
        Arity::Fixed(4),
        native_mutable_buffer_caselessmatch,
    ),
    (
        "MutableBuffer",
        "CASELESSMATCHCHAR",
        Arity::Fixed(2),
        native_mutable_buffer_caselessmatchchar,
    ),
    (
        "MutableBuffer",
        "CASELESSPOS",
        Arity::Fixed(3),
        native_mutable_buffer_caselesspos,
    ),
    (
        "MutableBuffer",
        "CASELESSSTARTSWITH",
        Arity::Fixed(1),
        native_mutable_buffer_caselessstartswith,
    ),
    (
        "MutableBuffer",
        "CASELESSWORDPOS",
        Arity::Fixed(2),
        native_mutable_buffer_caselesswordpos,
    ),
    (
        "MutableBuffer",
        "CHANGESTR",
        Arity::Fixed(3),
        native_mutable_buffer_changestr,
    ),
    (
        "MutableBuffer",
        "CONTAINS",
        Arity::Fixed(3),
        native_mutable_buffer_contains,
    ),
    (
        "MutableBuffer",
        "CONTAINSWORD",
        Arity::Fixed(2),
        native_mutable_buffer_containsword,
    ),
    (
        "MutableBuffer",
        "COUNTSTR",
        Arity::Fixed(1),
        native_mutable_buffer_countstr,
    ),
    (
        "MutableBuffer",
        "DELETE",
        Arity::Fixed(2),
        native_mutable_buffer_delete,
    ),
    (
        "MutableBuffer",
        "DELSTR",
        Arity::Fixed(2),
        native_mutable_buffer_delstr,
    ),
    (
        "MutableBuffer",
        "DELWORD",
        Arity::Fixed(2),
        native_mutable_buffer_delword,
    ),
    (
        "MutableBuffer",
        "ENDSWITH",
        Arity::Fixed(1),
        native_mutable_buffer_endswith,
    ),
    (
        "MutableBuffer",
        "GETBUFFERSIZE",
        Arity::Fixed(0),
        native_mutable_buffer_getbuffersize,
    ),
    (
        "MutableBuffer",
        "INSERT",
        Arity::Fixed(4),
        native_mutable_buffer_insert,
    ),
    (
        "MutableBuffer",
        "LASTPOS",
        Arity::Fixed(3),
        native_mutable_buffer_lastpos,
    ),
    (
        "MutableBuffer",
        "LENGTH",
        Arity::Fixed(0),
        native_mutable_buffer_length,
    ),
    (
        "MutableBuffer",
        "LOWER",
        Arity::Fixed(2),
        native_mutable_buffer_lower,
    ),
    (
        "MutableBuffer",
        "MAKEARRAY",
        Arity::Fixed(1),
        native_mutable_buffer_makearray,
    ),
    (
        "MutableBuffer",
        "MAKESTRING",
        Arity::Fixed(0),
        native_mutable_buffer_makestring,
    ),
    (
        "MutableBuffer",
        "MATCH",
        Arity::Fixed(4),
        native_mutable_buffer_match,
    ),
    (
        "MutableBuffer",
        "MATCHCHAR",
        Arity::Fixed(2),
        native_mutable_buffer_matchchar,
    ),
    (
        "MutableBuffer",
        "OVERLAY",
        Arity::Fixed(4),
        native_mutable_buffer_overlay,
    ),
    (
        "MutableBuffer",
        "POS",
        Arity::Fixed(3),
        native_mutable_buffer_pos,
    ),
    (
        "MutableBuffer",
        "REPLACEAT",
        Arity::Fixed(4),
        native_mutable_buffer_replaceat,
    ),
    (
        "MutableBuffer",
        "SETBUFFERSIZE",
        Arity::Fixed(1),
        native_mutable_buffer_setbuffersize,
    ),
    (
        "MutableBuffer",
        "SETTEXT",
        Arity::Fixed(1),
        native_mutable_buffer_settext,
    ),
    (
        "MutableBuffer",
        "SPACE",
        Arity::Fixed(2),
        native_mutable_buffer_space,
    ),
    (
        "MutableBuffer",
        "STARTSWITH",
        Arity::Fixed(1),
        native_mutable_buffer_startswith,
    ),
    (
        "MutableBuffer",
        "STRING",
        Arity::Fixed(0),
        native_mutable_buffer_string,
    ),
    (
        "MutableBuffer",
        "SUBCHAR",
        Arity::Fixed(1),
        native_mutable_buffer_subchar,
    ),
    (
        "MutableBuffer",
        "SUBSTR",
        Arity::Fixed(3),
        native_mutable_buffer_substr,
    ),
    (
        "MutableBuffer",
        "SUBWORD",
        Arity::Fixed(2),
        native_mutable_buffer_subword,
    ),
    (
        "MutableBuffer",
        "SUBWORDS",
        Arity::Fixed(2),
        native_mutable_buffer_subwords,
    ),
    (
        "MutableBuffer",
        "TRANSLATE",
        Arity::Fixed(5),
        native_mutable_buffer_translate,
    ),
    (
        "MutableBuffer",
        "UPPER",
        Arity::Fixed(2),
        native_mutable_buffer_upper,
    ),
    (
        "MutableBuffer",
        "VERIFY",
        Arity::Fixed(4),
        native_mutable_buffer_verify,
    ),
    (
        "MutableBuffer",
        "WORD",
        Arity::Fixed(1),
        native_mutable_buffer_word,
    ),
    (
        "MutableBuffer",
        "WORDINDEX",
        Arity::Fixed(1),
        native_mutable_buffer_wordindex,
    ),
    (
        "MutableBuffer",
        "WORDLENGTH",
        Arity::Fixed(1),
        native_mutable_buffer_wordlength,
    ),
    (
        "MutableBuffer",
        "WORDPOS",
        Arity::Fixed(2),
        native_mutable_buffer_wordpos,
    ),
    (
        "MutableBuffer",
        "WORDS",
        Arity::Fixed(0),
        native_mutable_buffer_words,
    ),
    // `MutexSemaphoreClass::close`, `EventSemaphore`'s partner above.
    ("MutexSemaphore", "UNINIT", Arity::Fixed(0), native_no_op),
    // `Setup.cpp:521`-`:530`, each declared at count 1. These are what an
    // operator applied to an instance resolves against when its class defines
    // none of its own; `Interp::operator_message_receiver` is the send, and
    // the names are `Operator::spelling`'s, which
    // `corpus/lang/operator_methods.rex` is what holds against the oracle.
    ("Object", "=", Arity::Fixed(1), native_object_identical),
    ("Object", "==", Arity::Fixed(1), native_object_identical),
    ("Object", "\\=", Arity::Fixed(1), native_object_different),
    ("Object", "\\==", Arity::Fixed(1), native_object_different),
    ("Object", "<>", Arity::Fixed(1), native_object_different),
    ("Object", "><", Arity::Fixed(1), native_object_different),
    ("Object", "||", Arity::Fixed(1), native_object_concat),
    ("Object", "", Arity::Fixed(1), native_object_concat),
    ("Object", " ", Arity::Fixed(1), native_object_concat_blank),
    ("Object", "CLASS", Arity::Fixed(0), native_class),
    ("Object", "COPY", Arity::Fixed(0), native_copy),
    (
        "Object",
        "DEFAULTNAME",
        Arity::Fixed(0),
        native_default_name,
    ),
    ("Object", "HASMETHOD", Arity::Fixed(1), native_has_method),
    ("Object", "HASHCODE", Arity::Fixed(0), native_hash_code),
    (
        "Object",
        "IDENTITYHASH",
        Arity::Fixed(0),
        native_identity_hash,
    ),
    // `RexxObject::initRexx` under its own name, `memory/Setup.cpp:520`. The
    // same function `ACTIVATE` above names, and a second row rather than a
    // second implementation, because a second implementation is where the two
    // could come to disagree. `RexxClass::subclass` sends this to every class
    // it builds (`classes/ClassClass.cpp:1631`), so a `::CLASS` declaring no
    // `::METHOD init CLASS` of its own reaches this entry.
    ("Object", "INIT", Arity::Fixed(0), native_no_op),
    ("Object", "ISA", Arity::Fixed(1), native_is_a),
    ("Object", "ISNIL", Arity::Fixed(0), native_is_nil),
    ("Object", "OBJECTNAME", Arity::Fixed(0), native_object_name),
    (
        "Object",
        "OBJECTNAME=",
        Arity::Fixed(1),
        native_object_name_set,
    ),
    ("Object", "REQUEST", Arity::Fixed(1), native_request),
    // `AddPrivateMethod("Run", RexxObject::run, A_COUNT)`,
    // `memory/Setup.cpp:549`. Private, so a program context is refused by
    // the private check before this entry is reached, and restricted
    // besides -- D66 and [`check_restricted_method`].
    ("Object", "RUN", Arity::Counted, native_run),
    ("Object", "SEND", Arity::Counted, native_send),
    ("Object", "SENDWITH", Arity::Fixed(2), native_send_with),
    ("Object", "START", Arity::Counted, native_start),
    ("Object", "STARTWITH", Arity::Fixed(2), native_start_with),
    // `AddPrivateMethod("SetMethod", ..., 3)` and its partner at
    // `memory/Setup.cpp:550`-`:551`. Private, which is where the refusal a
    // program context meets comes from; `rexx_classes::native_classes` files
    // the access scope off the same rows.
    ("Object", "SETMETHOD", Arity::Fixed(3), native_set_method),
    ("Object", "STRING", Arity::Fixed(0), native_string),
    (
        "Object",
        "UNSETMETHOD",
        Arity::Fixed(1),
        native_unset_method,
    ),
    (
        "Package",
        "ADDCLASS",
        Arity::Fixed(2),
        native_package_add_class,
    ),
    (
        "Package",
        "ADDPUBLICCLASS",
        Arity::Fixed(2),
        native_package_add_public_class,
    ),
    ("Package", "ANNOTATION", Arity::Fixed(1), native_annotation),
    (
        "Package",
        "ANNOTATIONS",
        Arity::Fixed(0),
        native_annotations,
    ),
    ("Package", "LOCAL", Arity::Fixed(0), native_package_local),
    ("Package", "NAME", Arity::Fixed(0), native_package_name),
    (
        "Package",
        "PUBLICCLASSES",
        Arity::Fixed(0),
        native_package_public_classes,
    ),
    // `QueueClass::initRexx` (`memory/Setup.cpp:777`) and the donation
    // `InheritInstanceMethods(IdentityTable)` makes at `Relation` (`:958`).
    ("Relation", "INIT", Arity::Fixed(1), native_capacity_init),
    // `.context`'s own package, which is the running program's -- the one
    // route to it, since `Class~package` above answers `REXX` for every
    // class the bootstrap registered.
    (
        "RexxContext",
        "PACKAGE",
        Arity::Fixed(0),
        native_context_package,
    ),
    ("Routine", "ANNOTATION", Arity::Fixed(1), native_annotation),
    (
        "Routine",
        "ANNOTATIONS",
        Arity::Fixed(0),
        native_annotations,
    ),
    // The same donation at `Set` (`memory/Setup.cpp:908`).
    ("Set", "INIT", Arity::Fixed(1), native_capacity_init),
    ("String", "LENGTH", Arity::Fixed(0), native_length),
    (
        "String",
        "MAKEARRAY",
        Arity::Fixed(0),
        native_string_makearray,
    ),
    (
        "String",
        "MAKESTRING",
        Arity::Fixed(0),
        native_string_make_string,
    ),
    ("String", "REVERSE", Arity::Fixed(0), native_reverse),
    // `memory/Setup.cpp:648`, at count 0.
    ("String", "SIGN", Arity::Fixed(0), native_string_sign),
    ("String", "UPPER", Arity::Fixed(2), native_string_upper),
    // `.methods`, `.routines` and `.resources`. The same functions the
    // `Directory` rows above name, because the C++ is the same code reached
    // through the same donation: `StringTable` takes `[]`, `At` and `Put` from
    // `IdentityTable` (`memory/Setup.cpp:881`) and declares `Unknown` itself
    // (`:883`), and `Directory` then takes that whole set from `StringTable`.
    ("StringTable", "[]", Arity::Fixed(1), native_hash_at),
    ("StringTable", "[]=", Arity::Fixed(2), native_hash_put),
    ("StringTable", "AT", Arity::Fixed(1), native_hash_at),
    // `HashCollection::initRexx`, which `~new` sends and which validates the
    // initial-size argument, in `StringTable`'s own dictionary by
    // `InheritInstanceMethods(IdentityTable)` (`memory/Setup.cpp:881`) out of
    // that class's own row (`:841`).
    ("StringTable", "INIT", Arity::Fixed(1), native_capacity_init),
    ("StringTable", "PUT", Arity::Fixed(2), native_hash_put),
    (
        "StringTable",
        "UNKNOWN",
        Arity::Fixed(2),
        native_hash_unknown,
    ),
    // `SupplierClass::initRexx` (`memory/Setup.cpp:1626`), which is where
    // `.Supplier~new`'s refusal comes from: the two arrays are required and
    // the allocation above it is not what raises.
    // The same donation at `Table` (`memory/Setup.cpp:861`).
    ("Table", "INIT", Arity::Fixed(1), native_capacity_init),
    // `VariableReference`'s own rows (`memory/Setup.cpp:1298`-`:1302`), the
    // whole of what its class defines beyond `Object`'s.
    (
        "VariableReference",
        "NAME",
        Arity::Fixed(0),
        native_reference_name,
    ),
    (
        "VariableReference",
        "VALUE",
        Arity::Fixed(0),
        native_reference_value,
    ),
    (
        "VariableReference",
        "VALUE=",
        Arity::Fixed(1),
        native_reference_value_set,
    ),
    (
        "VariableReference",
        "UNKNOWN",
        Arity::Fixed(2),
        native_reference_unknown,
    ),
    (
        "VariableReference",
        "REQUEST",
        Arity::Fixed(1),
        native_reference_request,
    ),
];

/// The primitive methods bound to a class's **class** dictionary rather than
/// its instance one -- `memory/Setup.cpp`'s `AddClassMethod` rows.
///
/// A separate table because the two dictionaries are separate: a name in one
/// is not the name in the other, and [`ObjectModel::build`] resolves a row
/// here through `lookup_class_method`. `Setup.cpp`'s `AddMethod` on `.Class`
/// stays in [`NATIVE_METHODS`], because a class object's messages resolve
/// against `.Class`'s *instance* behaviour by way of the metaclass merge.
static NATIVE_CLASS_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    // `AddClassMethod("New", RexxObject::newRexx, A_COUNT)`,
    // `memory/Setup.cpp:514`, reached by every class whose own class
    // behaviour declares no `NEW` of its own.
    ("Object", "NEW", Arity::Counted, native_new),
    // `AddClassMethod("New", ArrayClass::newRexx, A_COUNT)`,
    // `memory/Setup.cpp:708`. A row of its own for the reason
    // `StringTable`'s below is one: the answer is a body this crate builds
    // rather than an instance.
    ("Array", "NEW", Arity::Counted, native_array_new),
    // `AddClassMethod("Of", ArrayClass::ofRexx, A_COUNT)`,
    // `memory/Setup.cpp:709`. The same body as the row above, filled from the
    // arguments rather than sized from them.
    ("Array", "OF", Arity::Counted, native_array_of),
    // `AddClassMethod("Of", QueueClass::ofRexx, A_COUNT)` and
    // `AddClassMethod("Of", ListClass::ofRexx, A_COUNT)`. One body: each
    // creates a collection of the receiver's own class and appends the
    // arguments to it (`classes/ListClass.cpp:1010`).
    (
        "Queue",
        "OF",
        Arity::Counted,
        collection::native_collection_of,
    ),
    (
        "List",
        "OF",
        Arity::Counted,
        collection::native_collection_of,
    ),
    // `AddClassMethod("New", StringTable::newRexx, A_COUNT)` and
    // `AddClassMethod("New", DirectoryClass::newRexx, A_COUNT)`,
    // `memory/Setup.cpp:875` and `:928`. Rows of their own rather than
    // `Object`'s, because each allocates a hash body rather than an instance.
    (
        "StringTable",
        "NEW",
        Arity::Counted,
        native_hash_collection_new,
    ),
    ("Directory", "NEW", Arity::Counted, native_directory_new),
    // Every class whose own `newRexx` is an allocation followed by
    // `completeNewObject` and nothing else -- `memory/Setup.cpp:766`, `:821`,
    // `:902`, `:953`, `:982`, `:1006`, `:1327`, `:1350`, `:1618`.
    // Each allocates a primitive body this crate does not model, so each
    // answers [`native_new`]'s plain instance: the class's own behaviour and
    // the `INIT` send, and nothing that would read the body.
    ("Bag", "NEW", Arity::Counted, hash::native_hash_new),
    // `AddClassMethod("Of", BagClass::ofRexx, A_COUNT)`, which unlike
    // `Set~of` keeps the duplicates.
    ("Bag", "OF", Arity::Counted, hash::native_bag_of),
    ("EventSemaphore", "NEW", Arity::Counted, native_new),
    // `TableClass::newRexx` and `IdentityTable::newRexx` take an optional
    // initial capacity, and it is observable: it decides the bucket count,
    // which decides the order every iteration answers in.
    (
        "IdentityTable",
        "NEW",
        Arity::Counted,
        hash::native_hash_new,
    ),
    ("List", "NEW", Arity::Counted, native_new),
    ("MutexSemaphore", "NEW", Arity::Counted, native_new),
    ("Queue", "NEW", Arity::Counted, native_new),
    ("Relation", "NEW", Arity::Counted, hash::native_hash_new),
    ("Set", "NEW", Arity::Counted, hash::native_hash_new),
    // `AddClassMethod("Of", SetClass::ofRexx, A_COUNT)`: `Set`'s own, unlike
    // the mapped classes whose `of` is `MapCollection~OF` in Rexx.
    ("Set", "OF", Arity::Counted, hash::native_set_of),
    ("Supplier", "NEW", Arity::Counted, native_new),
    ("Table", "NEW", Arity::Counted, hash::native_hash_new),
    // The classes whose own `newRexx` checks its arguments and then builds
    // something this crate does not -- a class object, an undispatched
    // message, a compiled executable, a loaded package, a weak reference
    // (`memory/Setup.cpp:450`, `:1052`, `:1091`, `:1128`, `:1156`, `:1683`).
    // Each checks exactly what the C++ checks before that point and refuses
    // loudly past it.
    ("Class", "NEW", Arity::Counted, native_class_new),
    ("Message", "NEW", Arity::Counted, native_message_new),
    ("Method", "NEW", Arity::Counted, native_executable_new),
    ("Package", "NEW", Arity::Counted, native_package_new),
    ("Routine", "NEW", Arity::Counted, native_executable_new),
    (
        "WeakReference",
        "NEW",
        Arity::Counted,
        native_weak_reference_new,
    ),
    // `AddClassMethod("New", RexxString::newRexx, A_COUNT)`,
    // `memory/Setup.cpp:572`. The one row here whose answer is a value rather
    // than an instance.
    ("String", "NEW", Arity::Counted, native_string_new),
    // `AddClassMethod("New", StemClass::newRexx, A_COUNT)`,
    // `memory/Setup.cpp:1371`. A row of its own because the answer is the
    // `Body::Stem` a bare stem read also produces, not an instance.
    ("Stem", "NEW", Arity::Counted, native_stem_new),
    // `AddClassMethod("New", MutableBuffer::newRexx, A_COUNT)`,
    // `memory/Setup.cpp:1418`. A row of its own because the arguments are
    // checked before the allocation rather than by `INIT`.
    (
        "MutableBuffer",
        "NEW",
        Arity::Counted,
        native_mutable_buffer_new,
    ),
];

/// The two methods `Setup.cpp` puts on `.Class` for the image build and
/// `removeSetupMethods` deletes before the image is saved (D39).
///
/// **Registered only when the model is built for the library bootstrap**
/// ([`ObjectModel::bootstrap_for_library`]), because the class dictionary
/// only holds their names then: `rexx_classes::native_classes` leaves them
/// out and `native_classes_for_bootstrap` puts them in, and a row here
/// naming a method the registry does not answer is a panic at model-build
/// time.
///
/// **Neither has an oracle transcript** and neither can get one: they do not
/// exist in any shipped interpreter, so every refusal below is this crate's
/// own judgement rather than a measurement. Each is [`Loud`] for that
/// reason -- a plausible Rexx condition here would be a wrong answer nobody
/// could check.
static SETUP_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    (
        "Class",
        "DEFINECLASSMETHOD",
        Arity::Fixed(2),
        native_define_class_method,
    ),
    (
        "Class",
        "INHERITINSTANCEMETHODS",
        Arity::Fixed(1),
        native_inherit_instance_methods,
    ),
];

/// `~defaultName` for an array -- `RexxObject::defaultName`'s article rule
/// (`classes/ObjectClass.cpp:1760`) applied to `.Array`'s id.
///
/// This is what `stringValue()` answers for an array: `RexxObject::stringValue`
/// sends `OBJECTNAME` (`classes/ObjectClass.cpp:1157`), which falls through to
/// `defaultName()` for an object nothing has named. Measured, `a =
/// .Array~superClasses`: `a~objectName` and `a~defaultName` are both `an
/// Array`.
///
/// A constant rather than a lookup because every array this crate builds is an
/// instance of `.Array` itself: `native_superclasses` is the one constructor.
pub(crate) const ARRAY_DEFAULT_NAME: &[u8] = b"an Array";

/// The message the search order's last step sends -- `GlobalNames::UNKNOWN`.
///
/// Upper case because every name a dictionary is keyed by is: the parser
/// upcases the name in a send, and `::METHOD unknown` is installed upcased,
/// so a lookup spelled any other way finds nothing.
const UNKNOWN: &[u8] = b"UNKNOWN";

/// The message name a `~run` body is entered under --
/// `GlobalNames::UNNAMED_METHOD` (`memory/GlobalNames.h:240`), which
/// `RexxObject::run` passes to `methobj->run`
/// (`classes/ObjectClass.cpp:2245`).
const UNNAMED_METHOD: &[u8] = b"*UNNAMED*";

/// The entry a `Message` object keeps the value its send answered under.
///
/// On the object rather than beside it in [`Interp::message_outcomes`]
/// because the collector walks a `Body::Native`'s entries and does not walk
/// that table.
const MESSAGE_RESULT: &[u8] = b"RESULT";

/// The message a class construction sends the class it just built --
/// `GlobalNames::INIT`, sent by `RexxClass::subclass`
/// (`classes/ClassClass.cpp:1631`). Upper case for [`UNKNOWN`]'s reason.
pub(crate) const INIT: &[u8] = b"INIT";

/// The message a finalizer delivery sends -- `GlobalNames::UNINIT`, sent by
/// `UninitDispatcher::run` (`memory/UninitDispatcher.cpp:52`). Upper case for
/// [`UNKNOWN`]'s reason.
const UNINIT: &[u8] = b"UNINIT";

/// How many times the termination sweep runs, from the number of times the
/// oracle's shutdown reaches `MemoryObject::runUninits`: `collectAndUninit`
/// at `runtime/InterpreterInstance.cpp:581` and `lastChanceUninit` at
/// `runtime/Interpreter.cpp:279`. See
/// [`Interp::run_termination_uninits`](Interp::run_termination_uninits).
const SWEEPS: usize = 2;

/// The message the last install pass sends every class the package built --
/// `GlobalNames::ACTIVATE`, sent by `ClassDirective::activate`
/// (`instructions/ClassDirective.cpp:288`).
pub(crate) const ACTIVATE: &[u8] = b"ACTIVATE";

/// The class model this crate dispatches against, and the implementations
/// its `MethodId`s name.
///
/// One struct rather than separate `Interp` fields because the table is
/// derived from the registry: the `MethodId`s in `natives` are minted by
/// `classes`, so a registry built without the matching table would resolve
/// every name and implement none.
pub(crate) struct ObjectModel {
    classes: ClassRegistry,
    natives: HashMap<MethodId, NativeEntry>,
    /// The classes a value this crate builds answers to, and `.Class`,
    /// resolved once at bootstrap. These are handles into `classes` and not a
    /// second model of it, and holding them buys two things: a send does not
    /// look its receiver's class up by name (which upcases and allocates) on
    /// every message, and neither a send nor a `::CLASS` install can be
    /// redirected by a program that declares a class of one of these names.
    string: ObjRef,
    object: ObjRef,
    metaclass: ObjRef,
    array: ObjRef,
    package: ObjRef,
    method: ObjRef,
    routine: ObjRef,
    directory: ObjRef,
    string_table: ObjRef,
    context: ObjRef,
    /// The class `Setup.cpp` registers with `addToSystem` and whose
    /// *instance* `.RexxInfo` answers -- see [`Primitive::RexxInfo`].
    /// Reached through `ClassRegistry::system_lookup`, since no `.NAME`
    /// resolves to it.
    rexx_info: ObjRef,
    /// The class `~start` and `~startWith` answer an instance of.
    message: ObjRef,
    /// The class a `>name` term answers an instance of.
    variable_reference: ObjRef,
    /// The class a `Body::Stem` answers, whether a bare stem read produced it
    /// or `.Stem~new` did.
    stem: ObjRef,
}

impl ObjectModel {
    /// The hash-collection class a `DO OVER` target may be, for a caller
    /// asking whether an object is one.
    ///
    /// **`StringTable` and not `Directory`**, which is a narrowing rather
    /// than an omission and is measured. A `StringTable` this crate hands out
    /// -- `.methods`, `.routines`, `.resources`, a package's
    /// `~publicClasses` -- holds exactly what the running program's own
    /// directives and sends put in it, so its membership is the oracle's:
    /// measured, a file with three unattached `::METHOD`s answers `3` for
    /// both. `.environment` and `.local` are `Directory`s this crate models
    /// as a subset of the oracle's, so iterating one would differ in
    /// *membership* and not only in order: measured, `.local` iterates ten
    /// entries on the oracle and none here.
    pub(crate) fn iterable_collection_class(&self) -> ObjRef {
        self.string_table
    }

    /// `Setup.cpp`'s native class set, plus the lookup from each implemented
    /// method's minted identity to its code.
    ///
    /// **Built on first use and not in `Interp::new`**, through
    /// `Interp::object_model`'s `get_or_insert_with`, which is its only
    /// caller. `Interp::bootstrap_library` assigns the model itself, so what
    /// this builds is the model of an `Interp` that never ran the bootstrap.
    fn bootstrap() -> ObjectModel {
        ObjectModel::build(rexx_classes::native_classes(), &[])
    }

    /// [`ObjectModel::bootstrap`] with the two setup-only methods present --
    /// the state the interpreter's own library runs in, and the one
    /// `Interp::bootstrap_library` closes with
    /// `rexx_classes::remove_setup_methods`.
    pub(crate) fn bootstrap_for_library() -> ObjectModel {
        ObjectModel::build(rexx_classes::native_classes_for_bootstrap(), SETUP_METHODS)
    }

    fn build(
        classes: rexx_classes::ClassRegistry,
        extra: &[(&str, &str, Arity, NativeMethod)],
    ) -> ObjectModel {
        let mut natives = HashMap::new();
        for (class_id, method_name, arity, run) in NATIVE_METHODS
            .iter()
            .chain(string::NATIVE_METHODS)
            .chain(hash::NATIVE_METHODS)
            .chain(collection::NATIVE_METHODS)
            .chain(extra)
        {
            let class = classes.lookup(class_id).unwrap_or_else(|| {
                panic!("NATIVE_METHODS names class {class_id:?}, which is not in the registry")
            });
            let (_, method) = classes
                .lookup_instance_method(class, method_name)
                .unwrap_or_else(|| {
                    panic!(
                        "NATIVE_METHODS names {class_id}~{method_name}, which that class's \
                         behaviour does not answer"
                    )
                });
            natives.insert(
                method,
                NativeEntry {
                    arity: *arity,
                    run: *run,
                },
            );
        }
        for (class_id, method_name, arity, run) in NATIVE_CLASS_METHODS {
            let class = classes.lookup(class_id).unwrap_or_else(|| {
                panic!(
                    "NATIVE_CLASS_METHODS names class {class_id:?}, which is not in the registry"
                )
            });
            let (_, method) = classes
                .lookup_class_method(class, method_name)
                .unwrap_or_else(|| {
                    panic!(
                        "NATIVE_CLASS_METHODS names {class_id}~{method_name}, which that class's \
                         class behaviour does not answer"
                    )
                });
            natives.insert(
                method,
                NativeEntry {
                    arity: *arity,
                    run: *run,
                },
            );
        }
        let string = classes.lookup("String").expect("String is a native class");
        let object = classes.lookup("Object").expect("Object is a native class");
        let metaclass = classes.lookup("Class").expect("Class is a native class");
        let array = classes.lookup("Array").expect("Array is a native class");
        let package = classes
            .lookup("Package")
            .expect("Package is a native class");
        let method = classes.lookup("Method").expect("Method is a native class");
        let routine = classes
            .lookup("Routine")
            .expect("Routine is a native class");
        let directory = classes
            .lookup("Directory")
            .expect("Directory is a native class");
        let string_table = classes
            .lookup("StringTable")
            .expect("StringTable is a native class");
        let context = classes
            .lookup("RexxContext")
            .expect("RexxContext is a native class");
        let rexx_info = classes
            .system_lookup("RexxInfo")
            .expect("RexxInfo is a native class in the kernel directory");
        let message = classes
            .lookup("Message")
            .expect("Message is a native class");
        let variable_reference = classes
            .lookup("VariableReference")
            .expect("VariableReference is a native class");
        let stem = classes.lookup("Stem").expect("Stem is a native class");
        ObjectModel {
            classes,
            natives,
            string,
            object,
            metaclass,
            array,
            package,
            method,
            routine,
            directory,
            string_table,
            context,
            rexx_info,
            message,
            variable_reference,
            stem,
        }
    }
}

/// What [`Interp::write_object_method`] does to one name in an object's own
/// dictionary, which decides which of [`ObjectMethods`]'s levels it reaches.
#[derive(Copy, Clone, Debug)]
enum ObjectMethodWrite {
    /// `setMethod`, whose `None` is the no-method form: it stores the
    /// oracle's `.nil` and hides the name rather than removing anything.
    Set(Option<ObjectMethod>),
    /// `Class~enhanced`, which no `unsetMethod` reaches.
    Enhance(ObjectMethod),
    /// `unsetMethod`, which reveals whatever is behind the entry it takes.
    Remove,
}

/// Which native class a value answers to -- the classes a value this crate
/// can build belongs to, and no others.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Primitive {
    /// Every string-valued receiver: a literal of any length, and a number.
    /// Measured, `~class~id` for `'abc'`, `12345` and `1.5` is `String` for
    /// all three -- `RexxInteger` and `NumberString` are
    /// `CLASS_CREATE_SPECIAL(..., "String", ...)` and answer `String` for
    /// their own class.
    String,
    /// A receiver whose whole value is in the handle's integer tag
    /// ([`Decoded::SmallInt`]) -- **D24's `SmallInt` behaviour arm**.
    ///
    /// **It changes nothing today, and it is not a cheaper route either.**
    /// The arm that answers [`Primitive::String`] for the handle's inline
    /// text decides from the tag too, so the split buys no arena read and no
    /// class-range test that was being paid before it; and every consumer
    /// folds this variant straight back into `String`'s answer
    /// ([`Interp::receiver_behaviour`], [`native_class`]). Measured against
    /// the oracle, a small integer and the same digits as a string are
    /// indistinguishable: `12345~class~id` is `String` for both
    /// (`CLASS_CREATE_SPECIAL(Integer, "String", RexxIntegerClass)`,
    /// `classes/IntegerClass.cpp:2066`), and so are `~length`, `~isA`,
    /// `~hasMethod` and `~reverse`.
    ///
    /// **What it is, then, is the named place** D24 asks for: a phase that
    /// gives a tagged integer behaviour of its own changes this arm's mapping
    /// and touches neither the inline-text receiver nor the arena's.
    /// `a_small_integer_receiver_takes_the_small_int_arm` is what keeps the
    /// two halves from collapsing back together -- a distinct kind, one
    /// shared behaviour.
    SmallInt,
    /// `.nil`, measured: `.nil~class~id` is `Object`.
    Object,
    /// A `Body::Array`. Measured, `.Array~superClasses~class~id` is `Array`.
    Array,
    /// A `Body::Native` whose class is `.Package` -- what `~package` answers.
    /// Measured, `.Array~package~class~id` is `Package`.
    Package,
    /// A `Body::Native` whose class is `.Method` -- what `Class~method`
    /// answers and what a `.METHODS` entry holds. Measured,
    /// `.K~method("M")~class` is `The Method class`.
    Method,
    /// A `Body::Native` whose class is `.Routine` -- what a `.ROUTINES` entry
    /// holds. Measured, `.routines~r~class` is `The Routine class`.
    ///
    /// **Kept apart from [`Primitive::Method`] beside it** for
    /// [`Primitive::StringTable`]'s reason: the two classes hold different
    /// names and the traceback says which was the receiver's -- measured,
    /// `.routines~r~annotation()` reports `Compiled method "ANNOTATION" with
    /// scope "Routine".` where `.K~method("M")~annotation()` reports
    /// `"Method"`.
    Routine,
    /// A `Body::Native` whose class is `.Directory` -- `.environment` and
    /// `.local`. Measured, `.environment~class~id` is `Directory`.
    ///
    /// **Identity and not descent, unlike [`Primitive::StringTable`] beside
    /// it**: [`native_directory_new`] gives a subclass an instance body
    /// rather than a native one.
    Directory,
    /// A `Body::Native` whose class is `.StringTable` or a subclass of it --
    /// `.methods`, `.routines` and `.resources`, and `.TraceObject~new`.
    /// Measured, `.methods~class` is `The StringTable class` and
    /// `.TraceObject~new~class~id` is `TraceObject`.
    ///
    /// **Carries the object's own class**, because a subclass answers its own
    /// method set: measured, oracle rc 0,
    /// `.TraceObject~new~hasMethod("makeString")` is `1` where
    /// `.StringTable~new~hasMethod("makeString")` is `0`.
    ///
    /// **Separate from [`Primitive::Directory`] even though every method
    /// either answers is the same C++ function**, because `Directory` and
    /// `StringTable` are separate behaviours: the traceback names the
    /// receiver's own class, measured -- `.methods~at()` reports `Compiled
    /// method "AT" with scope "StringTable".` where `.environment~at()`
    /// reports `"Directory"`.
    ///
    /// **`.context` is not this**; it is [`Primitive::Context`] below.
    StringTable(ObjRef),
    /// A `Body::Native` whose class is `.RexxContext` -- what `.context`
    /// answers. Measured, `.context~class` is `The RexxContext class`.
    Context,
    /// A `Body::Native` whose class is the `RexxInfo` one -- the single
    /// pre-built instance `.RexxInfo` answers (`Setup.cpp:1735`-`:1737`).
    /// Measured, `.RexxInfo~class~id` is `RexxInfo` and
    /// `.RexxInfo~isA(.Class)` is `0`.
    ///
    /// **The class object itself takes [`Primitive::Class`] and not this
    /// arm**, whatever route puts it in a program's hands:
    /// [`Interp::receiver_kind`] asks `ObjRef::class_id` before it reaches
    /// the arena at all, so a class handle never gets as far as the
    /// `Body::Native` guards below.
    ///
    /// `RexxInfo`'s instance behaviour here is `Setup.cpp`'s whole set, the
    /// position `.Package` above is in: a name it does not hold is a name
    /// the running oracle does not hold either, so 97.1 is the answer and
    /// not a guess -- measured, `.RexxInfo~id` is
    /// `97.1 Object "a RexxInfo" does not understand message "ID"`. A name
    /// it *does* hold with no [`NativeMethod`] behind it is this crate's own
    /// gap and refuses loudly, because the oracle answers those: measured,
    /// `.RexxInfo~digits` is `9` and `.RexxInfo~languageLevel` is `6.06`.
    RexxInfo,
    /// A `Body::Native` whose class is `.Message` -- what `~start` and
    /// `~startWith` answer. Measured, oracle rc 0: `o~start('M', 5)~class~id`
    /// is `Message` and `~string` is `a Message`.
    ///
    /// `Message`'s instance behaviour here is `Setup.cpp`'s whole set, the
    /// position `.Package` above is in, so a name it does not hold is 97.1
    /// and a name it holds with no [`NativeMethod`] behind it is this crate's
    /// own gap.
    Message,
    /// A `Body::VarRef` -- the object a `>name` term answers. Measured,
    /// `vr = 5; o = >vr; say o~class~id` is `VariableReference`.
    ///
    /// **Its behaviour hides `=`, `==`, `\\=`, `\\==`, `<>` and `><`**
    /// (`memory/Setup.cpp:1307`-`:1312`), so those names miss and reach
    /// `UNKNOWN`, which forwards them to the referenced value.
    VariableReference,
    /// A `Body::Stem` -- what a bare stem read answers and what `.Stem~new`
    /// builds. Measured, oracle rc 0: `s. = 'd'; o = s.; say o~class~id` is
    /// `Stem`.
    ///
    /// **Its string value is the stem's default, not `a Stem`**, which is
    /// `classify_string_conversion`'s `Body::Stem` arms rather than this one;
    /// `~objectName` and `~defaultName` are `a Stem` all the same, measured.
    Stem,
    /// The receiver **is** a class object, so its messages resolve against
    /// that class's own class behaviour rather than against any class's
    /// instance behaviour. Measured, `::class K` plus `::method m class`:
    /// `.K~m` runs the body, `.K~hasMethod('M')` is `1` and
    /// `.K~hasMethod('LENGTH')` is `0`.
    Class(ObjRef),
    /// What `~new` builds: an object carrying both the class it belongs to
    /// and the behaviour it was given at construction (D58). Measured,
    /// `::class K` and `o = .K~new`: `o~class~id` is `K` and `o~string` is
    /// `a K`.
    ///
    /// **The two are separate answers**, which is what the pair is for:
    /// `class` is what `~class` and the default rendering read, and
    /// `behaviour` is what every message resolves against. `~define` on `K`
    /// after this object exists moves the class's own behaviour and leaves
    /// this one where it was.
    Instance {
        class: ObjRef,
        behaviour: BehaviourHandle,
    },
}

/// The behaviour a receiver's messages resolve against.
///
/// An enum rather than a bare `ObjRef`, because a class object's messages
/// are answered by its **class** behaviour while every other receiver's are
/// answered by its class's **instance** behaviour, and those dictionaries
/// hold different names for the same class.
///
/// The instance arm names the dictionary itself rather than the class, so
/// that every reader of it -- the lookup, `~hasMethod`, the scope-override
/// check, `SUPER`, the `UNINIT` question -- answers from the behaviour the
/// receiver actually holds and none of them has to restate D58 for itself.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Behaviour {
    Instance {
        methods: BehaviourHandle,
        /// `behaviour->getOwningClass()`: the class the dictionary belongs
        /// to, which the copying mutators carry across unchanged.
        owner: ObjRef,
    },
    ClassSide(ObjRef),
}

/// One message term's own parts, as the two call sites already hold them.
///
/// A struct rather than a positional parameter list: `super_class` and
/// `assigned` are both `Option<&Expr>` and `cascade` is a bare `bool`, so a
/// positional signature invites exactly the transposition that would show up
/// as wrong output rather than as a compile error.
#[derive(Copy, Clone)]
pub(crate) struct MessageTerm<'a> {
    pub(crate) target: &'a Expr,
    /// Already upcased by the parser, and already carrying the trailing `=`
    /// for the message-assignment form.
    pub(crate) name: &'a [u8],
    /// The `:scope` override's own expression.
    pub(crate) super_class: Option<&'a Expr>,
    pub(crate) args: &'a [Option<Expr>],
    /// `~~`, which yields the target rather than the method's result.
    pub(crate) cascade: bool,
    /// The message-assignment form's value expression, which the oracle
    /// inserts as the **first** argument ahead of the term's own
    /// (`RexxInstructionMessage`'s two-argument constructor,
    /// `MessageInstruction.cpp:76-88`).
    pub(crate) assigned: Option<&'a Expr>,
}

/// The sending side of a send, which the receiver does not carry (D53).
///
/// **One input per access scope that reads one.** `PRIVATE` compares
/// the *caller's own receiver* against the receiver of the send --
/// `RexxObject::checkPrivate` reads `activation->getReceiver()`
/// (`classes/ObjectClass.cpp:616`), allows the send outright when the two are
/// the same object (`:617`-`:620`), and refuses when the caller has no
/// receiver at all (`:622`-`:626`). `PACKAGE` compares the method's package
/// against the *caller's* -- `RexxObject::checkPackage` reads
/// `activation->getPackage()` (`classes/ObjectClass.cpp:671`) and refuses
/// when there is no calling activation (`:665`-`:669`).
///
/// **A parameter of [`Interp::resolve`] rather than something read off
/// `Interp` inside it.** The oracle takes both off `getTopStackFrame()`, so
/// both are properties of the activation the send is written in, and a
/// resolution asked for outside a send -- by [`Interp::send_message`]'s own
/// callers, or by a test -- has to be able to say which caller it is asking
/// as, and a caller with neither half is a state both of the oracle's checks
/// refuse rather than a state they cannot be asked about.
///
/// The C++'s spelling of `PRIVATE`'s input is the caller's **receiver**, not
/// the caller's method scope: `:628` reads the scope off the *method* being
/// resolved, which [`Resolution`] already carries.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) struct Caller {
    /// The receiver of the call in progress, from the calling convention.
    ///
    /// `None` where the code the send is written in was not itself entered
    /// by a message send, which is `RexxActivation::getReceiver`'s
    /// `OREF_NULL` (`execution/RexxActivation.cpp:2342`-`:2349`). An
    /// `INTERPRET` fragment answers its enclosing activation's receiver there
    /// (`:2344`-`:2347`) and answers it here for the same reason: a fragment
    /// runs in the activation that interpreted it, so the calling convention
    /// it reads is that activation's.
    receiver: Option<ObjRef>,
    /// The package the sending code was translated in, which is the running
    /// activation's program.
    ///
    /// **A variant for "no activation" rather than an absent package**, which
    /// is the state `checkPackage` refuses before it reads anything
    /// (`classes/ObjectClass.cpp:665`-`:669`). [`crate::plan::Package`] has
    /// the other side of the same rule: an absent [`crate::plan::ProgramId`]
    /// there would mean the interpreter's own package, so the two absences
    /// would be one type with opposite meanings and a `PACKAGE` check
    /// comparing them directly would call `.Array`'s package "no package".
    package: CallerPackage,
}

/// The package a send is written in, as `RexxObject::checkPackage` reads it.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum CallerPackage {
    /// No activation is running. `checkPackage` refuses outright for it
    /// (`classes/ObjectClass.cpp:665`-`:669`), and a resolution asked for
    /// from outside any running program is in it.
    NoActivation,
    /// The running activation's own package, which is the package its clauses
    /// were translated in.
    Package(Package),
}

impl Caller {
    /// The receiver of the call the send is written in.
    ///
    /// Read by [`Interp::check_private`], which compares it against the
    /// receiver of the send.
    pub(crate) fn receiver(self) -> Option<ObjRef> {
        self.receiver
    }

    /// The package the send is written in -- [`Interp::check_package`]'s
    /// input.
    pub(crate) fn package(self) -> CallerPackage {
        self.package
    }
}

/// One method's access scope and protection, as `MethodClass`'s own flags
/// carry them (`classes/MethodClass.hpp:115`-`:118`).
///
/// **Only a method the oracle calls *special* has one of these.**
/// `messageSend` reads the flags at all only inside
/// `if (method_save->isSpecial())` (`classes/ObjectClass.cpp:876`), which is
/// `protected || private || package`, so a method with neither an access
/// keyword nor `PROTECTED` is dispatched without any of this being consulted.
/// `Interp::special_methods` holds a row for exactly the special ones and
/// [`Interp::access_scope_of`] answers `None` for every other method.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) struct AccessScope {
    /// `PRIVATE` or `PACKAGE`. `Access::Default` or `Access::Public` for a
    /// method whose row exists for `PROTECTED` alone.
    pub(crate) access: Access,
    /// `MethodClass::isProtected`, which is what the security-manager seam
    /// asks about.
    pub(crate) protected: bool,
    /// The package the method's own directive was translated in.
    ///
    /// `PACKAGE`'s other input: `MethodClass::isSamePackage` asks the
    /// method's code object, not its scope class
    /// (`classes/MethodClass.hpp:147`), so it is the package of the file the
    /// `::METHOD` was written in.
    pub(crate) package: Package,
}

/// Why a send's own lookup produced no method to run.
///
/// The oracle's `error` local in both `messageSend` overloads
/// (`classes/ObjectClass.cpp:872`, `:930`). An access check that refuses does
/// not raise: it replaces the found method with nothing and sets this, and
/// `processUnknown` carries it to `reportNomethod` only for a receiver whose
/// behaviour answers no `UNKNOWN` (`:904`, `:1009`).
///
/// **So a refusal here is not a failure the caller reports**, and the
/// difference is observable. Measured, oracle rc 0: `.K~m` where `m` is
/// `::METHOD m CLASS PRIVATE` and the class also declares
/// `::METHOD unknown CLASS` prints `unknown saw M with 0`, and the same
/// program with the `UNKNOWN` removed is the 97.2 report at rc 159.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum Miss {
    /// The receiver's behaviour answers no method of that name -- 97.1.
    NoMethod,
    /// The method is `PRIVATE` and `checkPrivate` refused this caller --
    /// 97.2.
    Private,
    /// The method is `PACKAGE` and the caller is in another package -- 97.3.
    PackageScope,
}

/// What a resolved message send names: the method, and the class its
/// definition came from.
///
/// The scope is not bookkeeping. It is the class whose `~id` the oracle
/// prints in a native method's traceback line (`Compiled method "LENGTH"
/// with scope "String".`), and it is where a further scope-override send
/// from inside that method would start.
#[derive(Copy, Clone)]
pub(crate) struct Resolution {
    pub(crate) scope: ObjRef,
    pub(crate) method: MethodId,
}

impl Interp {
    /// The [`Caller`] a send written in the running activation resolves as.
    ///
    /// **The receiver is read out of the calling convention**
    /// ([`crate::CallContext`]) rather than out of the running activation,
    /// and that is where the oracle reads it too: `checkPrivate` asks
    /// `getTopStackFrame()->getReceiver()` (`classes/ObjectClass.cpp:612`,
    /// `:616`), which is the frame's own receiver and is `OREF_NULL` for a
    /// routine or program frame. The convention is saved and restored around
    /// every call, so a send inside a method reads that method's receiver and
    /// a send after it returns reads the caller's again.
    ///
    /// The package is the running activation's program, which is the package
    /// the sending clause was translated in.
    pub(crate) fn caller(&self) -> Caller {
        Caller {
            receiver: self.call_context.receiver,
            package: match self.running_program() {
                None => CallerPackage::NoActivation,
                Some(program) => CallerPackage::Package(Package::Program(program)),
            },
        }
    }

    /// The object model, built on first use.
    pub(crate) fn object_model(&mut self) -> &mut ObjectModel {
        if self.object_model.is_none() {
            self.install_object_model(ObjectModel::bootstrap());
        }
        self.object_model
            .as_mut()
            .expect("the object model was just installed")
    }

    /// Stores a freshly built object model and files the access scopes of
    /// the natives `Setup.cpp` declares private.
    ///
    /// **The natives are recorded here rather than at their `NATIVE_METHODS`
    /// row** because the identity a row is filed under is the registry's
    /// mint, and only the registry that minted it knows which rows came from
    /// an `AddPrivateMethod`. Every identity minted afterwards is higher,
    /// which is what lets [`Interp::record_access_scope`] keep appending in
    /// mint order.
    pub(crate) fn install_object_model(&mut self, model: ObjectModel) {
        debug_assert!(
            self.special_methods.is_empty(),
            "the natives' access scopes are the first rows filed, so nothing may have filed one \
             before the model that mints them exists"
        );
        self.special_methods.extend(
            model
                .classes
                .private_native_methods()
                .iter()
                .map(|&method| {
                    (
                        method,
                        AccessScope {
                            access: Access::Private,
                            protected: false,
                            package: Package::Rexx,
                        },
                    )
                }),
        );
        self.object_model = Some(model);
    }

    /// The class registry: the one class model in this crate (R9), holding
    /// `Setup.cpp`'s native classes and whatever `::CLASS` directives have
    /// installed beside them.
    pub(crate) fn classes(&mut self) -> &mut ClassRegistry {
        &mut self.object_model().classes
    }

    /// The rendering of a handle the arena does not hold.
    ///
    /// **A class identity is what reaches this today**, because
    /// `rexx_core::CLASS_SLOT_BASE` puts it past every slot the arena can
    /// allocate, so `Heap::resolve` answers `None` for it
    /// (`rexx-core/src/heap.rs:292`-`299`) with no test of its own. Anything
    /// else arriving here is a handle whose object is gone, which is the
    /// `a live value` tripwire the callers used to carry alone -- so a later
    /// phase adding another handle kind outside the arena has to widen the
    /// assertion below rather than inherit it.
    ///
    /// **`Interp::to_text` and `Interp::try_text` reach this through the
    /// `None` their existing `Heap::get` already produces**, which is what
    /// keeps a class object off their hot path. A guard testing every heap
    /// operand instead measured 73 instructions per pass of the `strings`
    /// benchmark axis, on a program that names no class at all.
    ///
    /// **`~objectName` and not `~defaultName`**, which is what
    /// `RexxObject::stringValue` sends (`classes/ObjectClass.cpp:1157`): a
    /// class object something has renamed renders under the new name.
    /// Measured, oracle rc 0: after `.K~objectName = "renamed"`, `say .K`
    /// prints `renamed`.
    pub(crate) fn not_in_arena(&self, value: ObjRef) -> &[u8] {
        assert!(value.class_id().is_some(), "a live value");
        self.class_object_name(value)
    }

    /// `~objectName` for a class object, which is [`Interp::class_default_name`]
    /// until `~objectName=` stores one.
    pub(crate) fn class_object_name(&self, class: ObjRef) -> &[u8] {
        self.object_model
            .as_ref()
            .expect("a class handle can only have come from the object model")
            .classes
            .object_name(class)
            .as_bytes()
    }

    /// `~defaultName` for a class object -- `The <id> class` -- borrowed out
    /// of the registry that minted the identity.
    ///
    /// **`&self`, which is why the string is stored rather than built.**
    /// `Interp::try_text` hands back a borrow of the value's own bytes and has
    /// nowhere to put a freshly formatted one.
    pub(crate) fn class_default_name(&self, class: ObjRef) -> &[u8] {
        self.object_model
            .as_ref()
            .expect("a class handle can only have come from the object model")
            .classes
            .default_name(class)
            .as_bytes()
    }

    /// `behaviour->getOwningClass()->getId()` for any receiver: the id of the
    /// class whose behaviour answers this object's messages.
    ///
    /// `None` where this phase builds no behaviour for the receiver, which is
    /// [`Interp::receiver_kind`]'s own refusal.
    fn receiver_class_id(&mut self, receiver: ObjRef) -> Option<String> {
        let owner = match self.receiver_behaviour(receiver).ok()? {
            Behaviour::Instance { owner, .. } => owner,
            // A class object's messages resolve against its class behaviour,
            // whose owning class is the metaclass in play -- which is what
            // `ClassRegistry::class_of` answers, and what makes
            // `.Array~request("CLASS")` the receiver itself.
            Behaviour::ClassSide(class) => self.classes().class_of(class),
        };
        Some(self.classes().id_string(owner).to_string())
    }

    /// `~id` for a class object -- the name it was declared with, case
    /// unmodified.
    ///
    /// `&self`, for the reason [`Interp::class_default_name`] is: the caller
    /// is a `&self` reader that has a class handle and no way to reach the
    /// registry through [`Interp::classes`].
    pub(crate) fn class_id_text(&self, class: ObjRef) -> &str {
        self.object_model
            .as_ref()
            .expect("a class handle can only have come from the object model")
            .classes
            .id_string(class)
    }

    /// `.Object` and `.Class`: the superclass and metaclass
    /// `RexxClass::subclass` defaults to, which is what a bare `::CLASS`
    /// declares (`ClassClass.cpp:1562`).
    pub(crate) fn root_and_metaclass(&mut self) -> (ObjRef, ObjRef) {
        let model = self.object_model();
        (model.object, model.metaclass)
    }

    /// `.String`: the class every string-valued receiver answers to, and the
    /// scope a stem-forwarded operator's traceback frame names -- see
    /// `eval.rs`'s `Interp::is_stem_receiver`.
    pub(crate) fn string_class(&mut self) -> ObjRef {
        self.object_model().string
    }

    /// `.Package`: the class of the object `~package` answers, which
    /// `environment.rs` builds one of per package.
    pub(crate) fn package_class(&mut self) -> ObjRef {
        self.object_model().package
    }

    /// `.Method`: the class of the object `Class~method` answers, which
    /// `environment.rs` builds one of per instance dictionary entry a program
    /// asks for.
    pub(crate) fn method_class(&mut self) -> ObjRef {
        self.object_model().method
    }

    /// Which native class a value answers to, or the value's own shape when
    /// this phase builds no class for it.
    ///
    /// Every heap shape that has no class here gets a refusal naming itself
    /// rather than a shared one, so whichever task makes one reachable as a
    /// receiver gets a message that says which.
    ///
    /// A **class object** is the one heap-tagged handle that answers, and it
    /// answers as itself rather than as an instance of anything: its
    /// behaviour is that class's class behaviour, which is where `::METHOD
    /// ... CLASS` installs.
    ///
    /// An **array** answers, because `~superClasses` puts one in a program's
    /// hands. A name `.Array`'s behaviour here does not hold is the oracle's
    /// 97.1. `CoreClasses.orx:93` and `:97`
    /// are the same `~inherit` and both run: measured, `.String~superClasses`
    /// names `Comparable` and `.Array~superClasses` names
    /// `OrderedCollection` on this crate and on the oracle alike. What is
    /// missing is the mixin's own methods and not the edge -- measured,
    /// `'abc'~compareTo('abd')` is `-1` on the oracle and a Phase 5 refusal
    /// here.
    fn receiver_kind(&self, receiver: ObjRef) -> Result<Primitive, &'static str> {
        match receiver.decode() {
            Decoded::Nil => Ok(Primitive::Object),
            Decoded::SmallInt(_) => Ok(Primitive::SmallInt),
            Decoded::Text(_) => Ok(Primitive::String),
            // **Asked before the arena is**, which is the whole point of
            // `rexx_core::CLASS_SLOT_BASE`: a class identity is heap-tagged
            // and names no slot, so reaching for the arena with one answers
            // from whatever object happens to hold that index.
            Decoded::Heap { .. } if receiver.class_id().is_some() => Ok(Primitive::Class(receiver)),
            Decoded::Heap { .. } => match self.heap.get(receiver) {
                // A handle whose slot is gone. A receiver is rooted by the
                // term that evaluated it, so an expression cannot reach this
                // with a value it just produced; what can is a handle held
                // somewhere the collector's root set does not name, which is
                // a defect in the rooting rather than in the program.
                // Answered rather than panicked for the reason every error
                // path here is: an implementation gap must not become a
                // crash.
                None => Err("a value whose object is no longer live"),
                Some(object) => match &object.body {
                    Body::Text { .. } | Body::Num { .. } => Ok(Primitive::String),
                    Body::Stem { .. } => Ok(Primitive::Stem),
                    Body::Array { .. } => Ok(Primitive::Array),
                    Body::Instance {
                        class, behaviour, ..
                    } => Ok(Primitive::Instance {
                        class: *class,
                        behaviour: *behaviour,
                    }),
                    Body::WeakRef(_) => Err("a weak reference"),
                    // `>name`'s own object. Its behaviour is `Setup.cpp`'s
                    // whole set, hidden operators included, so a name it
                    // does not hold reaches `UNKNOWN` on both sides.
                    Body::VarRef(_) => Ok(Primitive::VariableReference),
                    // The package object `~package` answers. `.Package`'s
                    // instance behaviour here is `Setup.cpp`'s whole set, so a
                    // name it does not hold is a name the running oracle does
                    // not hold either, and 97.1 is the right answer rather than
                    // a guess.
                    //
                    // The prologue leaves `.Package` alone, and the claim is as
                    // wide as the pattern behind it: case-insensitive
                    // `\.package\b` over `interpreter/RexxClasses/`'s
                    // `CoreClasses.orx` and `StreamClasses.orx` matches nothing
                    // in either file, where the same pattern for `.array`
                    // matches in both.
                    Body::Native(native)
                        if self.object_model.as_ref().map(|model| model.package)
                            == Some(native.class()) =>
                    {
                        Ok(Primitive::Package)
                    }
                    // The `Method` and `Routine` objects `~method` and a
                    // package table put in a program's hands. Each class's
                    // instance behaviour here is `Setup.cpp`'s whole set, so
                    // a name it does not hold is a name the running oracle
                    // does not hold either -- the position `.Package` beside
                    // them is in.
                    Body::Native(native)
                        if self.object_model.as_ref().map(|model| model.method)
                            == Some(native.class()) =>
                    {
                        Ok(Primitive::Method)
                    }
                    Body::Native(native)
                        if self.object_model.as_ref().map(|model| model.routine)
                            == Some(native.class()) =>
                    {
                        Ok(Primitive::Routine)
                    }
                    Body::Native(native)
                        if self.object_model.as_ref().map(|model| model.directory)
                            == Some(native.class()) =>
                    {
                        Ok(Primitive::Directory)
                    }
                    // Descent rather than identity: `::class "TraceObject"
                    // subclass StringTable`'s class-side `NEW` forwards to
                    // this one, and the object it builds carries
                    // `TraceObject` as its class.
                    Body::Native(native)
                        if self.object_model.as_ref().is_some_and(|model| {
                            model.classes.is_a(native.class(), model.string_table)
                        }) =>
                    {
                        Ok(Primitive::StringTable(native.class()))
                    }
                    // `.context`. Its class is in the registry, so there is
                    // a behaviour to resolve against, and `RexxContext`'s
                    // instance behaviour here is `Setup.cpp`'s whole set --
                    // the position `.Package` above is in, so a name it does
                    // not hold is 97.1 and a name it holds with no row here
                    // is this crate's own gap.
                    Body::Native(native)
                        if self.object_model.as_ref().map(|model| model.context)
                            == Some(native.class()) =>
                    {
                        Ok(Primitive::Context)
                    }
                    // `.RexxInfo`. Its class is in the registry's kernel
                    // directory rather than the environment one, which
                    // changes nothing about the behaviour a send resolves
                    // against -- the position `.Package` above is in.
                    Body::Native(native)
                        if self.object_model.as_ref().map(|model| model.rexx_info)
                            == Some(native.class()) =>
                    {
                        Ok(Primitive::RexxInfo)
                    }
                    Body::Native(native)
                        if self.object_model.as_ref().map(|model| model.message)
                            == Some(native.class()) =>
                    {
                        Ok(Primitive::Message)
                    }
                    // A `Body::Native` of a class this crate builds no
                    // receiver arm for. Nothing constructs one today; loud
                    // rather than answered, this crate's rule for an internal
                    // inconsistency.
                    Body::Native(_) => Err("one of the interpreter's own objects"),
                },
            },
        }
    }

    /// The behaviour a value's messages resolve against, or the value's own
    /// shape when this phase builds no class for it.
    ///
    /// **Only [`Primitive::Instance`] carries a stored handle**; every other
    /// receiver's is read off its class here, and the two answer alike for
    /// them because the `REXX_DEFINED` lock refuses every mutator a program
    /// could send to a class this crate builds. Measured, oracle rc 158:
    /// `.String~define('ZORK', .methods~z)` is 98.985, "User additions are
    /// not allowed to the REXX language classes".
    fn receiver_behaviour(&mut self, receiver: ObjRef) -> Result<Behaviour, &'static str> {
        let kind = self.receiver_kind(receiver)?;
        let model = self.object_model();
        let live = match kind {
            // Both arms answer one behaviour, which is what makes
            // `Primitive::SmallInt` a distinction without a divergence: the
            // oracle gives `RexxInteger` the id `String`
            // (`classes/IntegerClass.cpp:2066`), so a small integer's messages
            // resolve against `String`'s instance behaviour exactly as a
            // literal's do. This fold is one of the two sites that would
            // change if that ever stopped being true.
            Primitive::String | Primitive::SmallInt => model.string,
            Primitive::Object => model.object,
            Primitive::Array => model.array,
            Primitive::Package => model.package,
            Primitive::Method => model.method,
            Primitive::Routine => model.routine,
            Primitive::Directory => model.directory,
            Primitive::StringTable(class) => class,
            Primitive::Context => model.context,
            Primitive::RexxInfo => model.rexx_info,
            Primitive::Message => model.message,
            Primitive::VariableReference => model.variable_reference,
            Primitive::Stem => model.stem,
            Primitive::Class(class) => return Ok(Behaviour::ClassSide(class)),
            Primitive::Instance { class, behaviour } => {
                return Ok(Behaviour::Instance {
                    methods: behaviour,
                    owner: class,
                });
            }
        };
        Ok(Behaviour::Instance {
            methods: self.classes().instance_behaviour_handle(live),
            owner: live,
        })
    }

    /// **Step one of a send** (D24): which method a name reaches on this
    /// receiver and from which scope, or why the send has none to run.
    ///
    /// This is the whole of what `RexxObject::messageSend` does before
    /// `method->run`: [`Interp::lookup`], then the access check its
    /// `isSpecial()` branch performs (`classes/ObjectClass.cpp:874`-`:894`).
    /// A method the check refuses is not an error raised here -- see
    /// [`Miss`], which is the oracle's own `error` local.
    ///
    /// `caller` is the sending side, which the access scopes read and the
    /// receiver does not carry -- [`Caller`] has each one's C++ site.
    ///
    /// Nothing here is cached, per D28. The answer depends on the receiver's
    /// behaviour as it stands at this instant, and a `~define` between two
    /// sends of the same name at the same call site must change the second
    /// one's answer.
    pub(crate) fn resolve(
        &mut self,
        receiver: ObjRef,
        name: &[u8],
        start_scope: Option<ObjRef>,
        caller: Caller,
    ) -> Result<Resolution, Miss> {
        let Some(resolution) = self.lookup(receiver, name, start_scope) else {
            return Err(Miss::NoMethod);
        };
        match self.access_scope_of(resolution.method) {
            None => Ok(resolution),
            Some(scope) => match scope.access {
                // A method whose row exists for `PROTECTED` alone. The seam
                // asks about that one, not this.
                Access::Default | Access::Public => Ok(resolution),
                // `isPrivate()` and `isPackageScope()` are the two arms of
                // one `else if` in the C++, and the parser makes a second
                // access keyword 25.902, so no method is in both.
                Access::Private => self
                    .check_private(resolution, receiver, caller)
                    .map(|()| resolution),
                Access::Package => Self::check_package(scope.package, caller).map(|()| resolution),
            },
        }
    }

    /// `RexxBehaviour::methodLookup`, or `RexxObject::superMethod(msgname,
    /// startscope)` for a `target~name:scope` override, which searches only
    /// the starting scope itself and the scopes folded in ahead of it.
    ///
    /// **The lookup with no access check on it**, which is the shape
    /// `processUnknown` uses for its own `UNKNOWN` lookup
    /// (`classes/ObjectClass.cpp:1004`). Measured, oracle rc 0: a class whose
    /// only method is `::METHOD unknown CLASS PRIVATE` answers `.K~zork` from
    /// outside the class, where the same directive under any other name is
    /// refused.
    fn lookup(
        &mut self,
        receiver: ObjRef,
        name: &[u8],
        start_scope: Option<ObjRef>,
    ) -> Option<Resolution> {
        // **The object's own methods come first**, which is where
        // `MethodDictionary::addInstanceMethod` puts them
        // (`behaviour/MethodDictionary.cpp:399`, `addFront`). A scope
        // override starts inside the class hierarchy and so never reaches
        // one: `RexxObject::superMethod` searches from a named scope, and a
        // one-off's scope is `.nil` or the class's own.
        if start_scope.is_none()
            && let Some(entry) = self.own_method_entry(receiver, name)
        {
            return entry.map(|ObjectMethod { method, scope }| Resolution { scope, method });
        }
        let behaviour = self.receiver_behaviour(receiver).ok()?;
        // Borrowed rather than owned wherever the name is UTF-8, which every
        // name a program can write is: `from_utf8_lossy` allocates only for
        // the bytes it has to replace.
        let name = String::from_utf8_lossy(name);
        let (scope, method) = match (behaviour, start_scope) {
            (Behaviour::Instance { methods, .. }, None) => {
                self.classes().lookup_at(methods, &name)?
            }
            (Behaviour::Instance { methods, .. }, Some(start)) => {
                self.classes().lookup_from_scope_at(methods, &name, start)?
            }
            (Behaviour::ClassSide(class), None) => {
                self.classes().lookup_class_method(class, &name)?
            }
            (Behaviour::ClassSide(class), Some(start)) => self
                .classes()
                .lookup_class_method_from_scope(class, &name, start)?,
        };
        Some(Resolution { scope, method })
    }

    /// What `receiver`'s own dictionary answers for `name`: `None` when it
    /// holds no entry under it, `Some(None)` for a name it hides, and
    /// `Some(Some(method))` for one it defines.
    ///
    /// **Gated on [`Interp::object_methods`]**, so a program that never
    /// sends `SETMETHOD` pays one load and one branch per send --
    /// [`Interp::special_methods`]'s shape, taken for its reason.
    pub(crate) fn own_method_entry(
        &self,
        receiver: ObjRef,
        name: &[u8],
    ) -> Option<Option<ObjectMethod>> {
        if !self.object_methods {
            return None;
        }
        match &self.heap.get(receiver)?.body {
            Body::Instance { own: Some(own), .. } => own.get(name),
            _ => None,
        }
    }

    /// Installs one entry in `receiver`'s own dictionary, or takes one away
    /// -- `RexxObject::defineInstanceMethod` (`classes/ObjectClass.cpp:2297`)
    /// and `deleteInstanceMethod` (`:2328`).
    ///
    /// A receiver with no dictionary of its own to hold one is loud rather
    /// than silent, for [`native_object_name_set`]'s reason: the oracle
    /// copies the behaviour of whatever it is given, and forgetting the
    /// definition would be a wrong answer where the oracle keeps it.
    fn write_object_method(
        &mut self,
        receiver: ObjRef,
        name: &[u8],
        write: ObjectMethodWrite,
    ) -> Result<(), Failure> {
        match self.receiver_kind(receiver) {
            Ok(Primitive::Instance { .. }) => {}
            Ok(_) => return Err(Loud::object_method("a receiver with no scope of its own").into()),
            Err(kind) => return Err(Loud::receiver_class(kind).into()),
        }
        let Some(Object {
            body: Body::Instance { own, .. },
            ..
        }) = self.heap.get_mut(receiver)
        else {
            unreachable!("an instance receiver is Body::Instance")
        };
        match write {
            ObjectMethodWrite::Remove => {
                if let Some(own) = own {
                    own.remove(name);
                }
            }
            ObjectMethodWrite::Set(entry) => {
                own.get_or_insert_with(|| Box::new(ObjectMethods::new()))
                    .set(name, entry);
                self.object_methods = true;
            }
            ObjectMethodWrite::Enhance(method) => {
                own.get_or_insert_with(|| Box::new(ObjectMethods::new()))
                    .enhance(name, method);
                self.object_methods = true;
            }
        }
        self.check_uninit(receiver, name);
        Ok(())
    }

    /// `RexxObject::checkUninit` (`classes/ObjectClass.cpp:2604`), which both
    /// mutators run: an object that answers `UNINIT` after the change is
    /// registered for finalization and one that does not is taken back out.
    ///
    /// Both directions are measured, oracle rc 0:
    /// `self~setMethod('UNINIT', 'say "one-off uninit"')` prints from the
    /// termination sweep and from a forced collection alike, and
    /// `self~setMethod('UNINIT')` on a class that declares one prints
    /// nothing.
    ///
    /// Asked only for the one name, which is what makes it free for every
    /// other definition.
    fn check_uninit(&mut self, receiver: ObjRef, name: &[u8]) {
        if !name.eq_ignore_ascii_case(UNINIT) {
            return;
        }
        if self.answers_uninit(receiver) {
            self.heap.set_uninit(receiver);
        } else {
            self.heap.clear_uninit_all(&[receiver]);
        }
    }

    /// `RexxObject::validateScopeOverride` (`classes/ObjectClass.cpp:1950`):
    /// whether `scope` was folded into the behaviour this receiver resolves
    /// against, which is `behaviour->hasScope(scope)`.
    ///
    /// **The behaviour is the receiver's own, so a class object asks its
    /// class side.** `.k~tag:.Array` and `'abc'~length:.Array` are both
    /// 93.957 and reach that answer through different dictionaries.
    ///
    /// A receiver whose class this phase does not build answers `false`, so
    /// the send is refused with the oracle's own 93.957 rather than
    /// resolving from a scope nothing here can search. Every receiver
    /// [`Interp::receiver_kind`] rejects is already refused by
    /// [`Interp::send_message`] before the message name is looked up.
    /// `RexxObject::validateScopeOverride` as the send itself runs it.
    ///
    /// The oracle asks it inside `RexxObject::messageSend`'s scope-override
    /// overload (`classes/ObjectClass.cpp:921`), whose comment says FORWARD
    /// relies on that placement, so every site here that hands
    /// [`Interp::send_message`] a start scope asks it first.
    ///
    /// `target` is the object the send is made to, which for a `FORWARD` is
    /// the `TO` value rather than `SELF` -- measured, `forward to (t) class
    /// (.Other)` reports `Target object "a TGT"`. `Ok(())` for a send that
    /// names no scope.
    pub(crate) fn validate_scope_override(
        &mut self,
        target: ObjRef,
        scope: Option<ObjRef>,
    ) -> Result<(), Failure> {
        let Some(scope) = scope else {
            return Ok(());
        };
        if self.receiver_has_scope(target, scope) {
            return Ok(());
        }
        let named_target = self.string_value_text(target);
        let named_scope = self.string_value_text(scope);
        Err(Raised::scope_override_not_a_scope(&named_target, &named_scope).into())
    }

    fn receiver_has_scope(&mut self, receiver: ObjRef, scope: ObjRef) -> bool {
        match self.receiver_behaviour(receiver) {
            Ok(Behaviour::Instance { methods, .. }) => {
                self.classes().behaviour_has_scope(methods, scope)
            }
            Ok(Behaviour::ClassSide(class)) => {
                self.classes().class_behaviour_has_scope(class, scope)
            }
            Err(_) => false,
        }
    }

    /// The scope a name would resolve to on this receiver, for a caller that
    /// reports what a send it is not making would have failed with.
    ///
    /// `None` is 97.1's case -- the behaviour does not answer the name --
    /// and `Some(id)` names the class the entry came from, which is what
    /// [`Loud::native_method`]'s message carries.
    fn lookup_for_refusal(&mut self, receiver: ObjRef, name: &[u8]) -> Option<String> {
        let resolution = self.lookup(receiver, name, None)?;
        Some(self.classes().id_string(resolution.scope).to_string())
    }

    /// The access scope and protection of a resolved method, for the methods
    /// that have one.
    ///
    /// **The emptiness test is what keeps an ordinary send off the search.**
    /// The oracle reads `isSpecial()` off a flag word on the method object,
    /// and this crate has no method object to hang one on, so the question
    /// costs a lookup wherever it is asked at all. A program that declares no
    /// `PRIVATE`, `PACKAGE` or `PROTECTED` method has no row and pays one
    /// load and one branch per send instead.
    ///
    /// [`Interp::special_methods`] carries why the rows are searched rather
    /// than hashed, with the measurement.
    fn access_scope_of(&self, method: MethodId) -> Option<AccessScope> {
        if self.special_methods.is_empty() {
            return None;
        }
        // Ordered on the identity's own `u32`, which `MethodId` does not
        // itself expose an `Ord` for: the ordering is this crate's internal
        // use of a mint counter and not a property `rexx-classes` publishes.
        let found = self
            .special_methods
            .binary_search_by_key(&method.0, |&(key, _)| key.0)
            .ok()
            .map(|at| self.special_methods[at].1);
        // **Checked against a scan of the same rows**, because the failure
        // mode of a broken order is silence: a search that misses reports a
        // `PRIVATE` method as an ordinary one, and the send it should have
        // refused is answered instead.
        debug_assert_eq!(
            found,
            self.special_methods
                .iter()
                .find(|&&(key, _)| key == method)
                .map(|&(_, scope)| scope),
            "the search over the access scopes disagrees with a scan of the same rows"
        );
        found
    }

    /// `MethodClass::isProtected` (`classes/MethodClass.hpp:116`), asked by
    /// the security-manager seam and by nothing else.
    fn method_is_protected(&self, method: MethodId) -> bool {
        self.access_scope_of(method)
            .is_some_and(|scope| scope.protected)
    }

    /// The security manager's `checkProtectedMethod`, asked for a method the
    /// send has found to be `PROTECTED`.
    ///
    /// **Nothing installs a manager in this phase and the answer is
    /// permission.** `SecurityManager::checkProtectedMethod` returns `false`
    /// -- "not handled, run the method" -- from its first statement when
    /// there is no manager object (`execution/SecurityManager.cpp:175`), and
    /// `processProtectedMethod` then runs the method
    /// (`classes/ObjectClass.cpp:989`). Measured, both engines and the
    /// oracle: `::METHOD m CLASS PROTECTED` sent from outside its class is
    /// rc 0 with identical stdout.
    ///
    /// So this cannot refuse, and no program can distinguish it from its own
    /// absence. What it is for is the *place*: a manager arrives by
    /// `~setSecurityManager` on a Package, Method or Routine object, every
    /// route to one of which is a loud refusal in this phase, and the task
    /// that lands the first route has one function to fill in rather than a
    /// dispatch path to find.
    fn check_protected_method(
        &mut self,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<(), Failure> {
        let _ = (self, receiver, name, args);
        Ok(())
    }

    /// `RexxObject::checkPrivate` (`classes/ObjectClass.cpp:608`-`:646`).
    ///
    /// **The input is the caller's own receiver, not the caller's scope.**
    /// The C++ reads `activation->getReceiver()` (`:616`) and compares it
    /// against the receiving object, which is why `self~m` is allowed from
    /// inside the defining class *and* from a subclass's own method: both
    /// send to the same object the caller was entered on. Measured, oracle
    /// rc 0 `inner` for each, where a rule written over the defining scope
    /// alone would refuse the subclass.
    ///
    /// **Every measurement behind this is on a class method**, because
    /// reaching an instance method needs `~new`. The `isInstanceOf` limb
    /// below therefore has no differential witness in this phase: a class
    /// object's own class is the metaclass, never a user class, so the limb
    /// that answers for a second instance of the defining class is written
    /// from the C++ and covered by an in-crate test alone.
    fn check_private(
        &mut self,
        resolution: Resolution,
        receiver: ObjRef,
        caller: Caller,
    ) -> Result<(), Miss> {
        // No calling activation, or one whose frame carries no receiver: a
        // routine or program context, which is `OREF_NULL` at `:622`-`:626`
        // and the same refusal as `activation == OREF_NULL` at `:611`.
        let Some(sender) = caller.receiver() else {
            return Err(Miss::Private);
        };
        // The sending and receiving object being the same is allowed
        // outright (`:617`-`:620`).
        if sender == receiver {
            return Ok(());
        }
        // Another instance of the class that defined the method (`:628`-
        // `:633`), which is `classObject()->isCompatibleWith(scope)`.
        if let Some(class) = self.class_of_value(sender)
            && self.classes().is_a(class, resolution.scope)
        {
            return Ok(());
        }
        // A class object anywhere in the defining scope's own hierarchy
        // (`:635`-`:643`). `isOfClassType(Class, sender)` first, because
        // `isCompatibleWith` is a method on a class and not on a value.
        if self.is_class_object(sender) && self.classes().is_a(sender, resolution.scope) {
            return Ok(());
        }
        Err(Miss::Private)
    }

    /// `RexxObject::checkPackage` (`classes/ObjectClass.cpp:658`-`:685`).
    ///
    /// `method_package` is the package the `::METHOD` directive was
    /// translated in, which is what `MethodClass::isSamePackage` compares
    /// against the caller's.
    ///
    /// **The cross-package refusal is not reachable in this phase**, and a
    /// second package existing does not change that: only `Access::Package`
    /// reaches this check, so a `::METHOD ... PACKAGE` has to have declared
    /// the method, and no embedded library file declares one -- measured,
    /// `/bin/grep -acinE "^\s*::method[^;]*\bpackage\b"` answers 0 for each
    /// of the three. A program's own `::METHOD ... PACKAGE` is in the
    /// caller's package by construction. The same-package arm is what a
    /// program can run, and an in-crate test is the whole instrument for the
    /// other.
    fn check_package(method_package: Package, caller: Caller) -> Result<(), Miss> {
        match caller.package() {
            // No calling activation at all (`:665`-`:669`).
            CallerPackage::NoActivation => Err(Miss::PackageScope),
            CallerPackage::Package(package) if package == method_package => Ok(()),
            CallerPackage::Package(_) => Err(Miss::PackageScope),
        }
    }

    /// `RexxObject::classObject()`, the class an arbitrary value is an
    /// instance of, or `None` for a value this phase builds no class for.
    ///
    /// A **class object** answers its own class rather than itself:
    /// `receiver_behaviour` reports the class-side behaviour for one, and the
    /// class of a class object is what `~class` answers, which is the
    /// metaclass. `RexxInternalObject::isInstanceOf` is unconditionally false
    /// (`classes/ObjectClass.cpp:258`-`:261`), which is the `None` arm.
    fn class_of_value(&mut self, value: ObjRef) -> Option<ObjRef> {
        match self.receiver_behaviour(value).ok()? {
            Behaviour::Instance { owner, .. } => Some(owner),
            Behaviour::ClassSide(class) => Some(self.classes().class_of(class)),
        }
    }

    /// `isOfClassType(Class, value)`: whether the value is a class object.
    ///
    /// Asked through `receiver_kind` rather than off the handle's own tag,
    /// because a heap-tagged handle with a class id is the one shape that
    /// names no arena slot and that distinction is that function's.
    pub(crate) fn is_class_object(&self, value: ObjRef) -> bool {
        matches!(self.receiver_kind(value), Ok(Primitive::Class(_)))
    }

    /// The scope a method found at `resolution` sees as `SUPER`, or `None`
    /// when it is the topmost scope in the receiver's behaviour --
    /// `RexxObject::superScope(scope)`, the second of the two locals
    /// `RexxActivation::run` sets on a method activation
    /// (`RexxActivation.cpp:535`-`536`).
    fn super_scope_for(&mut self, receiver: ObjRef, resolution: Resolution) -> Option<ObjRef> {
        match self.receiver_behaviour(receiver).ok()? {
            Behaviour::Instance { methods, .. } => {
                self.classes().super_scope_at(methods, resolution.scope)
            }
            Behaviour::ClassSide(class) => {
                self.classes().class_super_scope(class, resolution.scope)
            }
        }
    }

    /// What a resolved [`MethodId`] runs, or the refusal for one no table
    /// below names.
    ///
    /// Asked **before** the seam rather than after, so the seam stays a
    /// single call site with every invocable kind behind it.
    fn invocable(&mut self, resolution: Resolution, name: &[u8]) -> Result<Invocable, Failure> {
        if let Some(entry) = self.object_model().natives.get(&resolution.method).copied() {
            return Ok(Invocable::Native(entry));
        }
        if let Some(installed) = self.method_bodies.get(&resolution.method).copied() {
            return Ok(Invocable::Rexx(installed));
        }
        // **After the body table and never before it**, so a send to a
        // written body reads exactly the one table it read before generated
        // methods existed -- see `crate::GeneratedMethod`'s own doc for what
        // that ordering is worth on `dispatchclass`.
        if let Some(generated) = self.generated_methods.get(&resolution.method).copied() {
            return Ok(Invocable::Generated(generated));
        }
        // **Last, so the tables above are read exactly as they were before
        // an `EXTERNAL` method could be sent to.** A send that reaches here
        // has already missed every table a send to an ordinary body or a
        // primitive reads, so no path this table is not on pays for it.
        if let Some(entry) = self.native_externals.get(&resolution.method).copied() {
            return Ok(Invocable::External(entry));
        }
        // The scope's `~id` is rendered only where it is printed -- a
        // successful send has no use for it, and every send would otherwise
        // pay for the copy.
        let scope = self.classes().id_string(resolution.scope).to_string();
        Err(Loud::native_method(name, &scope).into())
    }

    /// **Step two of a send** (D24): run what [`Interp::resolve`] found.
    ///
    /// The seam is passed first and the argument count is checked after,
    /// which is the oracle's order: `messageSend` asks the security manager
    /// before `method->run`, and the count check is part of the method's own
    /// entry (`NativeActivation::run`), not of the send.
    ///
    /// **`None` is a send that produced no value**, and it is not one
    /// [`Invocable`] kind's property. Measured, `::method m class` ending in a
    /// bare `return`: as a whole clause it drops `RESULT` at rc 0, and in an
    /// expression it is 91.999 at rc 165. `.environment~put('v','q')` answers
    /// the identical pair from a [`NativeMethod`], and so does a generated
    /// setter -- `.k~a = 5` on `::attribute a class` is rc 0 as a whole clause
    /// and `say .k~'A='(5)` is `91.999 Message "A=" did not return a result.`
    /// at rc 165, oracle and both engines.
    pub(crate) fn invoke(
        &mut self,
        resolution: Resolution,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        let invocable = self.invocable(resolution, name)?;
        let cleared = seam::clear(self, receiver, name, args, resolution.method)?;
        match invocable {
            Invocable::Native(entry) => {
                let outcome = match entry.arity {
                    Arity::Fixed(arity) if args.len() > arity => {
                        Err(Raised::too_many_method_arguments(arity).into())
                    }
                    Arity::Fixed(_) | Arity::Counted => (entry.run)(self, cleared, receiver, args),
                };
                if outcome.is_err() {
                    let scope = self.classes().id_string(resolution.scope).to_string();
                    self.blame_native_method(name, &scope);
                }
                outcome
            }
            // **Neither this arm nor `Generated` blames the method**,
            // measured twice over -- a claim about those arms and not
            // about their position, since `External`'s implemented half sits
            // between them and does blame.
            // An untrapped `1/0` inside a `::METHOD` body reports
            // the method's own failing clause and then the sending clause,
            // with no `Compiled method` line between them, and
            // [`Interp::enter_method_body`] seals its own level for that,
            // exactly as `Interp::invoke_call` does. A generated accessor and
            // an `ABSTRACT` send report the *sending* clause and nothing
            // above it: measured, `.K~a(1)` on `::attribute a class` is
            // `93.902` with the sending clause alone on `stderr`, where
            // `'abc'~length(1)` -- a [`NativeMethod`] taking the same
            // refusal -- carries a `Compiled method "LENGTH"` line. The two
            // differ in the C++ because `AttributeGetterCode::run` and
            // `AbstractCode::run` raise directly where `CPPCode::run` raises
            // from inside a `NativeActivation` of its own
            // (`execution/CPPCode.cpp:280`, `:526`).
            Invocable::Rexx(installed) => {
                self.enter_method_body(cleared, installed, resolution, receiver, name, args)
            }
            // **The refusal is the ordinary outcome here**, so unlike the
            // `Native` arm above this one does not blame the method for it:
            // a `Loud` is a report about this crate and carries no traceback
            // at all. The implemented arm does blame, because its refusals
            // are the oracle's own -- measured, `.k~sep(1)` on a class method
            // bound to `file_separator` is `88.922 Too many arguments in
            // invocation; 0 expected.` at rc 168, under a `Compiled method
            // "SEP" with scope "K".` line.
            Invocable::External(entry) => match &entry.body {
                native::ExternalBody::Deferred => Err(native::deferred_send(entry).into()),
                native::ExternalBody::Implemented { arity, run } => {
                    let outcome = match arity {
                        Arity::Fixed(arity) if args.len() > *arity => {
                            Err(Raised::too_many_external_arguments(*arity).into())
                        }
                        Arity::Fixed(_) | Arity::Counted => run(self, cleared, receiver, args),
                    };
                    if outcome.is_err() {
                        let scope = self.classes().id_string(resolution.scope).to_string();
                        self.blame_native_method(name, &scope);
                    }
                    outcome
                }
            },
            Invocable::Generated(generated) => match generated.kind {
                crate::GeneratedKind::Getter => {
                    self.read_attribute(cleared, generated, resolution, receiver, args)
                }
                crate::GeneratedKind::Setter => {
                    self.write_attribute(cleared, generated, resolution, receiver, args)
                }
                crate::GeneratedKind::Abstract => Err(Raised::abstract_method(name).into()),
                crate::GeneratedKind::Constant => {
                    self.read_constant(cleared, generated, receiver, args)
                }
                crate::GeneratedKind::Delegate => {
                    self.send_to_delegate(cleared, generated, resolution, receiver, name, args)
                }
            },
        }
    }

    /// A generated getter: the value the attribute's variable holds in the
    /// declaring scope's pool on the receiver, or -- for a variable nothing
    /// has assigned -- the derived name, which is that variable's own
    /// spelling.
    ///
    /// **No activation and no frame**, which is what the oracle's own shape
    /// makes it: `AttributeGetterCode::run` reads the pool and returns
    /// (`execution/CPPCode.cpp:280`-`:302`). A traced send therefore shows
    /// the ordinary `>M>` result line and nothing from inside the accessor --
    /// measured, `trace i` with `say .K~a` on a stored `5` echoes
    /// `>M>   "A" => "5"` and no `>I>`/`<I<` pair.
    ///
    /// **`GUARDED` has no reachable effect here.** The C++ splits on
    /// `method->isGuarded()` only to reserve the variable dictionary against
    /// other activities before reading it, and this crate runs one activity,
    /// so the two arms of that `if` are the same read. `::ATTRIBUTE`'s
    /// `GUARDED` and `UNGUARDED` are separate table D rows and both reach
    /// this function, which is why neither can be read as evidence about the
    /// keyword.
    ///
    /// The argument bound is the C++'s own and is checked before the pool is
    /// touched: measured, `.K~a(1)` is `93.902` naming `0 expected`.
    fn read_attribute(
        &mut self,
        _cleared: Cleared,
        generated: crate::GeneratedMethod,
        resolution: Resolution,
        receiver: ObjRef,
        args: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        if !args.is_empty() {
            return Err(Raised::too_many_method_arguments(0).into());
        }
        let variable = self.accessor_variable(generated)?;
        let owner = self.pool_owner(receiver)?;
        let stored = self
            .pools_of(owner)
            .and_then(|pools| pools.get(resolution.scope, &variable));
        Ok(Some(match stored {
            Some(value) => value,
            None => self.text(&variable),
        }))
    }

    /// A generated setter: assigns the attribute's variable in the declaring
    /// scope's pool on the receiver, and answers nothing.
    ///
    /// **Answering nothing is observable**, and it is the same absence a
    /// `::METHOD` body ending in a bare `return` produces: measured,
    /// `r = .K~'A='(9)` is `91.999` at rc 165, `Message "A=" did not return
    /// a result.`
    ///
    /// Both argument bounds are the C++'s own and both are checked before the
    /// pool is touched (`execution/CPPCode.cpp:330`-`:342`). Measured:
    /// `.K~'A='(1,2)` is `93.902` naming `1 expected`, and `.K~'A='()` is
    /// `93.903` naming `argument 1`.
    fn write_attribute(
        &mut self,
        _cleared: Cleared,
        generated: crate::GeneratedMethod,
        resolution: Resolution,
        receiver: ObjRef,
        args: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        if args.len() > 1 {
            return Err(Raised::too_many_method_arguments(1).into());
        }
        let Some(Some(value)) = args.first().copied() else {
            return Err(Raised::missing_method_argument(1).into());
        };
        let variable = self.accessor_variable(generated)?;
        let owner = self.pool_owner(receiver)?;
        self.set_pool_variable(owner, resolution.scope, &variable, value);
        Ok(None)
    }

    /// A `::CONSTANT` accessor: the value the resolve-constants pass recorded
    /// for the directive, on whichever dictionary side the send resolved
    /// through.
    ///
    /// **No activation and no frame**, for the reason [`Interp::read_attribute`]
    /// has none: `ConstantGetterCode::run` checks the argument count and
    /// returns the stored value (`execution/CPPCode.cpp:438`-`:454`).
    ///
    /// The argument bound is that function's own and is checked before the
    /// value is looked at: measured, `.A~c(1)` on `::constant c 5` is 93.902
    /// naming `0 expected`.
    ///
    /// **A constant with no value yet is 97.4 and not 97.1**, and the
    /// distinction is the oracle's: `reportNomethod` is reached with
    /// `Error_No_method_constant` rather than through a dictionary miss, so
    /// the name resolves and the *value* is what is missing. See
    /// [`Raised::constant_not_initialized`] for when that is reachable. The
    /// `NOMETHOD` condition it offers carries the **constant's** name as its
    /// description, which is the `message` argument `reportNomethod` takes
    /// (`execution/CPPCode.cpp:450`), and not the name the send spelled.
    ///
    /// [`Raised::constant_not_initialized`]: crate::Raised::constant_not_initialized
    fn read_constant(
        &mut self,
        _cleared: Cleared,
        generated: crate::GeneratedMethod,
        receiver: ObjRef,
        args: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        if !args.is_empty() {
            return Err(Raised::too_many_method_arguments(0).into());
        }
        if let Some(value) = self.constant_value(generated) {
            return Ok(Some(value));
        }
        let declared = self.constant_name(generated)?;
        let target = self.message_target_text(receiver);
        let report = Raised::constant_not_initialized(&target, &declared);
        Err(
            if self.trap_for(b"NOMETHOD").is_some_and(|trap| !trap.call) {
                Raised::nomethod(report, &declared).into()
            } else {
                report.into()
            },
        )
    }

    /// A `DELEGATE` method: the message re-sent, under the name it arrived
    /// under and with the arguments it arrived with, to the value of the
    /// delegate variable in the declaring scope's pool on the receiver
    /// (`DelegateCode::run`, `execution/CPPCode.cpp:605`-`:628`).
    ///
    /// **No activation and no frame**, which [`Interp::read_attribute`] has
    /// for the same reason and which is measured here from the other side: a
    /// `1/0` inside the delegated-to method reports its own clause and then
    /// the *sending* clause, with nothing between them.
    ///
    /// **The variable is read exactly as a generated getter reads one**,
    /// uninitialised value included -- measured, `::method m delegate d` with
    /// nothing assigned to `d` is `97.1 Object "D" does not understand
    /// message "M".` at rc 159, the delegate variable rendering as its own
    /// derived name and the message name being the one the send used.
    ///
    /// **The caller of the re-sent message is this send's own caller**, not
    /// the delegate method: `DelegateCode::run` pushes no activation, so
    /// `getTopStackFrame()` inside it is still the sending code's.
    /// [`Interp::caller`] answers the same here, because this arm runs before
    /// any activation is pushed.
    ///
    /// **`GUARDED` has no reachable effect**, for the reason
    /// [`Interp::read_attribute`]'s own doc gives: the C++ splits on
    /// `method->isGuarded()` only to reserve the variable dictionary against
    /// other activities while it reads the target, and this crate runs one
    /// activity.
    ///
    /// There is no argument bound: the delegated message takes whatever the
    /// original send carried, and the method it reaches applies its own.
    fn send_to_delegate(
        &mut self,
        _cleared: Cleared,
        generated: crate::GeneratedMethod,
        resolution: Resolution,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        let variable = self.delegate_variable(generated)?;
        let owner = self.pool_owner(receiver)?;
        let stored = self
            .pools_of(owner)
            .and_then(|pools| pools.get(resolution.scope, &variable));
        let target = match stored {
            Some(value) => value,
            None => self.text(&variable),
        };
        self.roots.push_temp(target);
        let caller = self.caller();
        self.send_message(target, name, None, args, caller)
    }

    /// The variable a `DELEGATE` method addresses, refusing the name shapes
    /// whose storage this crate has no representation for -- see
    /// [`Loud::delegate_variable`].
    ///
    /// [`Loud::delegate_variable`]: crate::Loud::delegate_variable
    fn delegate_variable(&self, generated: crate::GeneratedMethod) -> Result<Box<[u8]>, Failure> {
        let program = &self.programs[generated.program.0];
        let Some(directive) = program.directives.get(generated.directive) else {
            return Err(Loud::missing_body().into());
        };
        let Some(symbol) = crate::delegate_variable(&directive.kind) else {
            return Err(Loud::missing_body().into());
        };
        let variable = program.symbols.name(symbol).as_bytes();
        if crate::run::shape_of(variable) != crate::run::NameShape::Simple {
            return Err(Loud::delegate_variable(variable).into());
        }
        Ok(variable.into())
    }

    /// The variable a generated accessor addresses, refusing the name shapes
    /// whose storage this crate has no representation for -- see
    /// [`Loud::accessor_variable`].
    ///
    /// [`Loud::accessor_variable`]: crate::Loud::accessor_variable
    fn accessor_variable(&self, generated: crate::GeneratedMethod) -> Result<Box<[u8]>, Failure> {
        let program = &self.programs[generated.program.0];
        // `get` rather than an index, and `None` rather than a panic, for the
        // reason `Interp::enter_method_body`'s own reads carry.
        let Some(directive) = program.directives.get(generated.directive) else {
            return Err(Loud::missing_body().into());
        };
        let Some(variable) = crate::accessor_variable(&directive.kind) else {
            return Err(Loud::missing_body().into());
        };
        if crate::run::shape_of(variable) != crate::run::NameShape::Simple {
            return Err(Loud::accessor_variable(variable).into());
        }
        Ok(variable.into())
    }

    /// Runs one `::METHOD` body in an activation of its own, and answers what
    /// it returned.
    ///
    /// **Written here rather than beside [`Interp::invoke_call`] because of
    /// the `cleared` parameter**: [`Cleared`]'s constructor is private to
    /// [`mod seam`](seam), so the only functions that can take one by value
    /// are in this module, and taking one by value is what makes a Rexx body
    /// as unreachable-without-the-seam as a [`NativeMethod`] is.
    ///
    /// **The activation is a `::ROUTINE`'s in every respect the caller can
    /// see**, and each of those is measured on the oracle rather than carried
    /// over by analogy -- [`Activation::method`] lists them. The two pieces
    /// that are this function's own are the `SELF`/`SUPER` bindings and the
    /// gap check below.
    ///
    /// **Every ending is a value the caller gets, including `EXIT`.**
    /// Measured, `::method m class` whose body is `exit 5`: the sending
    /// clause receives `5` and the program runs on, exactly as a `::ROUTINE`
    /// does.
    fn enter_method_body(
        &mut self,
        _cleared: Cleared,
        installed: crate::InstalledMethodBody,
        resolution: Resolution,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        let program = Rc::clone(&self.programs[installed.program.0]);
        // Both reads are `get`, not an index: an `InstalledMethodBody` can
        // only have come from `Interp::record_method_body` and so always
        // names a real directive, but this crate's rule for an internal
        // inconsistency is a loud refusal rather than a panic, and
        // `body_of`'s own `?` already follows it.
        let Some(directive) = program.directives.get(installed.directive) else {
            return Err(Loud::missing_body().into());
        };
        if let Some(gap) = crate::method_body_gap(&directive.kind) {
            return Err(gap.into());
        }
        let Some(body) = body_of(&program, Some(installed.directive)) else {
            return Err(Loud::missing_body().into());
        };
        // D19/I6, the same guard `Interp::invoke_call` takes and for the same
        // reason: a method that sends itself a message is an unbounded
        // recursion, and it must become a reportable condition rather than a
        // native abort.
        if self.activation_depth() >= MAX_ACTIVATION_DEPTH {
            return Err(Raised::insufficient_stack().into());
        }
        let plan = self.plan_for(
            BodyKey {
                program: installed.program,
                directive: Some(installed.directive),
            },
            body,
            &program.symbols,
            &program.source,
        );
        let frame = self.roots.push_slots(plan.len());
        let callee_id = self.next_activation_id();
        let super_scope = self.super_scope_for(receiver, resolution);
        // **The calling convention, entered before anything reads it.** The
        // receiver goes in here and is read back out below, so the identity
        // the activation carries, the `SELF` the body reads and the caller a
        // send inside the body resolves as all name one field rather than
        // each taking its own copy of this function's argument.
        //
        // Saved and restored with the level state further down -- which is
        // where `Interp::invoke_call` does all of it -- and replaced ahead of
        // that group because the bindings below read out of it.
        let saved_context = std::mem::replace(
            &mut self.call_context,
            crate::CallContext {
                name: name.to_vec(),
                arguments: args.to_vec(),
                receiver: Some(receiver),
            },
        );
        // **Shadows the parameter**, which is what makes the routing a
        // property of scope rather than of a comment: past this line the name
        // `receiver` is the convention's own field, so the bindings below
        // cannot read the argument even by accident.
        let receiver = self
            .call_context
            .receiver
            .expect("the calling convention replaced directly above carries the receiver");
        let mut callee = Activation::method(
            callee_id,
            program,
            installed.program,
            installed.directive,
            plan,
            frame,
            MethodIdentity {
                name: name.into(),
                scope: resolution.scope,
                receiver,
            },
        );
        // The package the `::METHOD` was declared in, exactly as a
        // `::ROUTINE` starts from its own. `running_activation` rather than
        // `activation` because a send made outside any activation has no
        // caller for `::OPTIONS NUMERIC INHERIT` to read.
        self.start_from_package(
            &mut callee,
            self.running_activation().map(|caller| &caller.settings),
        );
        self.push_activation(callee);
        self.trace_package_invocation_entry();

        // `SELF` and `SUPER`, the two locals `RexxActivation::run` sets on a
        // method activation before its first instruction
        // (`RexxActivation.cpp:535`-`536`). Measured in a class method of
        // `::class K`: `say self` is `The K class` and `say super` is
        // `The Class class`.
        //
        // **Through `Interp::slot_of` rather than through the plan alone**,
        // so that a body which never mentions either name still binds them:
        // an `INTERPRET` inside such a body resolves `self` through the same
        // function and would otherwise read an unset variable and answer
        // `SELF`. `slot_of` grows the frame only when the plan has no slot,
        // so a body that does mention them pays nothing.
        //
        // **`SELF` is the receiver of the send and not the scope the method
        // was found at**, and the two part wherever a method is inherited:
        // measured on `::class K` with a class method `m` and `::class J
        // subclass K`, `.J~m` returning `self` answers `The J class` where
        // the resolution's scope is `K`. Reading it out of the calling
        // convention is what makes that the receiver by construction.
        let self_slot = self.slot_of(b"SELF");
        self.set_variable(frame, self_slot, receiver);
        let super_slot = self.slot_of(b"SUPER");
        // `.nil` for the topmost scope, which is what `superScope` answers
        // there.
        self.set_variable(frame, super_slot, super_scope.unwrap_or(ObjRef::NIL));

        // The same level state `Interp::invoke_call` saves and restores, set
        // to the same values its `::ROUTINE` arm uses -- that function's own
        // comment enumerates the pieces and carries the measurement for
        // each. A method's clause echoes are at indent 0 whatever the sending
        // clause's own indent was: measured, a send from inside two nested
        // `DO` blocks echoes the method's clauses at 0.
        let saved_clause_state = self.save_clause_state();
        let saved_base = std::mem::replace(&mut self.activation_indent, 0);
        let saved_offset = std::mem::take(&mut self.indent_offset);
        let saved_line = std::mem::take(&mut self.clause_line_override);

        let ended = self.run_activation();

        self.trace_invocation_exit();
        let callee = self.pop_activation().expect("the activation just pushed");
        // Unconditionally, where `Interp::invoke_call` asks `owns_frame`
        // first: a method activation always owns its frame and nothing can
        // change that under it, because the one instruction that swaps a
        // frame in is `PROCEDURE` and `Entry::Method` is 17.1 for it.
        debug_assert!(callee.owns_frame, "a method activation owns its frame");
        // **The callee's own convention comes out as the caller's goes back
        // in**, because a parked body still owns it: `ARG()` and a send's
        // caller resolution inside the resumed half read the same convention
        // the first half did. A body that is not parked drops it here.
        let callee_context = std::mem::replace(&mut self.call_context, saved_context);
        // The frame is released either way; what a `REPLY` changes is where
        // its contents go first. `park_reply` reads them out and hands them to
        // the collector's parked set, so the values survive with no frame open
        // above the caller's own.
        if callee.reply == ReplyState::Owed {
            self.park_reply(callee, callee_context);
        } else {
            self.roots.pop_slots(callee.frame);
        }
        self.activation_indent = saved_base;
        self.indent_offset = saved_offset;
        self.clause_line_override = saved_line;
        self.restore_clause_state(saved_clause_state);

        match ended {
            Ok(ended) => Ok(ended.value()),
            Err(crate::Failure::Exited(value)) => Ok(value),
            Err(failure) => {
                // Seal before the failure leaves the callee, the same rule
                // `Interp::invoke_call` follows: without it the method's own
                // clause wins `record_failure_at`'s first-wins race and the
                // sending clause is never echoed. Measured, `say 1/0` in a
                // class method reports the method's clause and then the
                // send's.
                self.seal_site_level();
                Err(failure)
            }
        }
    }

    /// Takes a method activation whose `REPLY` has just handed a value out,
    /// releases its frame, and queues the rest of its body.
    ///
    /// **SCHEDULING.** The oracle continues the body on another activity from
    /// here; `Interp::run_deferred_replies` continues it after the main
    /// program has finished. `Interp::deferred`'s own doc has the measured
    /// interleaving that separates the two.
    ///
    /// The order below is the whole of the rooting: the values come out of the
    /// frame, everything they and the convention name goes to
    /// `RootSet::park`, and only then is the frame released. Nothing between
    /// the read and the park can allocate, so there is no window in which a
    /// value is named by neither.
    fn park_reply(&mut self, mut activation: Box<Activation>, context: crate::CallContext) {
        let frame = activation.frame;
        let len = self.roots.frame_len(frame);
        // The values are copied out, so an alias in this frame would come back
        // as the resumed body's own storage and stop sharing. Both routes to
        // one are refused inside a method body -- measured, `PROCEDURE` there
        // is 17.1 at rc 239 and `USE ARG >q` is 88.928 at rc 168, both
        // matching the oracle -- and this is that stated as a check rather
        // than as a sentence, because what makes it true is elsewhere.
        debug_assert_eq!(
            self.roots.frame_aliases(frame),
            0,
            "a parked method frame holds an alias, whose sharing a copy loses"
        );
        let slots: Vec<Option<ObjRef>> = (0..len)
            .map(|index| self.roots.frame_slot(frame, index))
            .collect();
        // The one redirect the assertion above does not count, saved rather
        // than lost: a `>name` taken on a method's own local moves that
        // variable into a cell, and a copy that came back as plain storage
        // would leave the reference reading the cell and the variable
        // reading the copy.
        let aliases = self.roots.take_frame_aliases(frame);
        let mut anchor = Vec::new();
        activation.object_roots(&mut anchor);
        context.object_roots(&mut anchor);
        anchor.extend(slots.iter().flatten().copied());
        let parked = self.roots.park(anchor);
        self.roots.pop_slots(frame);
        activation.reply = ReplyState::Issued;
        self.deferred.push_back(DeferredReply {
            activation,
            context,
            slots,
            aliases,
            parked,
        });
    }

    /// Runs every method body a `REPLY` has left owed, oldest first, and
    /// answers what each of them raised.
    ///
    /// **SCHEDULING, and Phase 6 owns the whole function.** There is nothing
    /// of the language in it: draining a queue after the main program has
    /// finished is this interpreter's stand-in for the activity the oracle
    /// spawns, and `Interp::deferred`'s own doc has what that does and does
    /// not reproduce.
    ///
    /// **A body queued by a body already in this loop is run too**, which is
    /// what the queue is drained rather than iterated for: a resumed
    /// remainder can send a message whose method replies in its turn.
    ///
    /// The failures come back rather than being reported here, because what a
    /// report needs -- the program's path -- belongs to `execute` and so does
    /// the decision about the exit status. Measured, oracle rc **7**: a
    /// program ending `exit 7` whose replied method then raises 98.936 prints
    /// the traceback and exits 7, so a raise here settles nothing.
    ///
    /// Each failure carries its **own** echo stack, drained at the resume that
    /// produced it: the sites accumulate on `self` and a second resumed body
    /// would otherwise report the first one's clauses under its own error.
    ///
    /// **A deadline stops the drain**, and it is the one failure that does.
    /// This loop is where the self-forward's non-termination lives -- each
    /// drained body queues its own successor, so a deadline that only
    /// reddened one entry would leave the loop running for ever collecting
    /// failures. See [`Failure::Deadline`] and
    /// [`Deadline`](crate::clause::Deadline).
    pub(crate) fn run_deferred_replies(&mut self) -> Vec<(Failure, Vec<FailureSite>)> {
        let mut failures = Vec::new();
        let mut abandoned = false;
        while let Some(deferred) = self.deferred.pop_front() {
            if let Err(failure) = self.resume_reply(deferred) {
                abandoned = matches!(failure, Failure::Deadline);
                let mut sites = std::mem::take(&mut self.failure_sites);
                sites.extend(self.failure_site.take());
                failures.push((failure, sites));
                if abandoned {
                    break;
                }
            }
        }
        // Every park is matched by the release its resume does, and this is
        // where the pairing can be seen: the queue is empty, so a parked entry
        // still rooting anything is a set of values kept alive for the rest of
        // the process. Cheap and once per run, unlike `RootSet::live_frames`'
        // own callers.
        //
        // Not asked of a drain the deadline cut short: the queue is not empty
        // there, so the entries still in it are parked and owed.
        debug_assert!(
            abandoned || self.roots.live_parked() == 0,
            "a replied method body's values are still parked with nothing owing them"
        );
        failures
    }

    /// Sends `UNINIT` to `object` and answers a loud refusal if one escaped.
    ///
    /// **A raised condition and an `EXIT` are both discarded**, which is what
    /// `UninitDispatcher` under `activity->run` does
    /// (`memory/RexxMemory.cpp:373`, and the dispatcher's two `handleError`
    /// overrides at `memory/UninitDispatcher.cpp:64` and `:77`). Measured,
    /// oracle rc 0 with empty stderr: `y = 1/0`, `raise syntax 40.900` and
    /// `exit 5` inside an `UNINIT` each print the finalizer's own output and
    /// nothing else, and the rest of the program runs on. A loud refusal is
    /// not discarded -- that is this crate saying it cannot run the
    /// construct, and the loud rule is what keeps it from becoming a silent
    /// wrong answer.
    ///
    /// **An object that no longer answers `UNINIT` is reached and runs
    /// nothing**, which is `RexxObject::uninit`'s own `hasMethod` test
    /// (`classes/ObjectClass.cpp:2581`). A class's registration is never
    /// undone -- `RexxClass::checkUninit` only sets
    /// (`classes/ClassClass.cpp:1211`) -- so for an instance of a class
    /// whose ancestry gained and lost a finalizer this test is the only
    /// thing that cancels one. Measured,
    /// oracle rc 0 with empty stderr: `.QQ~inherit(.MX)` then
    /// `.QQ~uninherit(.MX)` runs `MX`'s finalizer and not `QQ`'s, and an
    /// instance built between an `~inherit` and its `~uninherit` runs none.
    fn run_one_uninit(&mut self, object: ObjRef) -> Option<Loud> {
        if !self.answers_uninit(object) {
            return None;
        }
        let caller = self.caller();
        let outcome = self.send_message(object, UNINIT, None, &[], caller);
        self.failure_site = None;
        self.failure_sites.clear();
        match outcome {
            // A deadline is discarded here with the rest, and `execute`'s own
            // guard is what keeps that from being a silent answer: the flag
            // the check set outlives this match. Every later clause of this
            // run fails the same way, so the bounded sweep above finishes at
            // once rather than running the remaining finalizers.
            Ok(_) | Err(Failure::Raised(_) | Failure::Exited(_) | Failure::Deadline) => None,
            Err(Failure::Loud(loud)) => Some(*loud),
        }
    }

    /// Whether `object` still answers `UNINIT` -- `RexxObject::hasMethod`
    /// as `RexxObject::uninit` asks it (`classes/ObjectClass.cpp:2581`),
    /// resolved the way [`native_has_method`] resolves the `HASMETHOD`
    /// message.
    fn answers_uninit(&mut self, object: ObjRef) -> bool {
        self.answers_message(object, "UNINIT")
    }

    /// Whether `object`'s behaviour answers `name`, which must already be
    /// upper case. `RexxBehaviour::methodLookup` with no scope override,
    /// resolved the way [`native_has_method`] resolves `HASMETHOD`.
    pub(crate) fn answers_message(&mut self, object: ObjRef, name: &str) -> bool {
        if let Some(entry) = self.own_method_entry(object, name.as_bytes()) {
            return entry.is_some();
        }
        let Ok(behaviour) = self.receiver_behaviour(object) else {
            return false;
        };
        let classes = &self.object_model().classes;
        match behaviour {
            Behaviour::Instance { methods, .. } => classes.has_method_at(methods, name),
            Behaviour::ClassSide(class) => classes.class_has_method(class, name),
        }
    }

    /// Runs `UNINIT` on each of `batch`, oldest first, with the whole batch
    /// rooted for the length of the run.
    ///
    /// **The flags are cleared before this is called**, which is `runUninits`
    /// removing the table entry (`memory/RexxMemory.cpp:362`) before running
    /// the method (`:373`). The flag was the batch's only root, so the park
    /// is this loop's `ProtectedObject`: without it a collection inside one
    /// finalizer sweeps the members that have not run.
    fn run_uninit_batch(&mut self, batch: Vec<ObjRef>, loud: &mut Vec<Loud>) {
        let parked = self.roots.park(batch.clone());
        for object in batch {
            loud.extend(self.run_one_uninit(object));
        }
        self.roots.release(parked);
    }

    /// Runs the `UNINIT` of every object a collection has readied, oldest
    /// first -- oracle's `MemoryObject::runUninits`
    /// (`memory/RexxMemory.cpp:337`), reached from `collectAndUninit` and so
    /// from `GC('force')` (`expression/BuiltinFunctions.cpp:3033`).
    ///
    /// **One pass, not a fixed point.** `runUninits` walks the table once,
    /// so an object readied *during* the walk is reached only if its bucket
    /// is still ahead of the iterator -- and an instance's bucket is its
    /// address, so which side it falls on is not reproducible. This crate
    /// takes the deterministic side and leaves it for the next sweep.
    ///
    /// **Re-entering it runs nothing**, which is the interlock `runUninits`
    /// opens with (`memory/RexxMemory.cpp:341`-`:347`, cleared at `:383`).
    /// A `GC('force')` inside a finalizer therefore collects and marks and
    /// runs no method.
    ///
    /// **Only the interlock has a reproducible witness.** Measured on a
    /// finalizer that builds an instance of another `UNINIT` class, drops it
    /// and calls `GC('force')`: the oracle never runs the inner finalizer
    /// inline, twelve runs of twelve, rc 0 with empty stderr -- but *when* it
    /// does run is bimodal, before the program's next clause in 3 of 13 runs
    /// and at termination in the other 10, which is the iterator's cursor
    /// against an address-derived bucket. `corpus/lang/uninit_nested_collection.rex`
    /// prints the first and deliberately not the second. The single pass is
    /// chosen on the C++ rather than on that split, and it lands on the
    /// majority side.
    pub(crate) fn run_ready_uninits(&mut self) -> Vec<Loud> {
        if self.processing_uninits {
            return Vec::new();
        }
        self.processing_uninits = true;
        let mut loud = Vec::new();
        let ready = std::mem::take(&mut self.uninit_ready);
        self.heap.clear_uninit_all(&ready);
        self.run_uninit_batch(ready, &mut loud);
        self.processing_uninits = false;
        loud
    }

    /// The termination sweep: every live object still carrying the flag
    /// (D69), then every class object with a class-side `UNINIT` (D60) in the
    /// order [`ClassRegistry::take_uninit_classes_in_sweep_order`] gives,
    /// and then the same again once.
    ///
    /// **Not the same as [`Interp::run_ready_uninits`] and not buildable out
    /// of it.** Under D59 a class-scope `EXPOSE` roots an instance for ever,
    /// so it never becomes unreachable and a collection never readies it;
    /// the oracle fires those at termination anyway, measured. A class object
    /// is never in the arena at all.
    ///
    /// **[`SWEEPS`] passes, and then whatever is still flagged is
    /// discarded.** The oracle's shutdown reaches the sweep exactly twice --
    /// `collectAndUninit` at `runtime/InterpreterInstance.cpp:581` and
    /// `lastChanceUninit` at `runtime/Interpreter.cpp:279`, which ends
    /// `uninitTable->empty()` (`memory/RexxMemory.cpp:330`) -- and each pass
    /// runs only what its own `collect` readied (`:274`). Measured, oracle
    /// rc 0 with empty stderr: a finalizer that allocates one further
    /// instance of its own class per call prints `u 1` / `u 2` and stops,
    /// at a self-imposed limit of 4 and of 8 alike. Running to a fixed point
    /// instead does not terminate on a finalizer that always allocates,
    /// where the oracle exits rc 0.
    ///
    /// **The instance group runs before the class group, and no check may
    /// depend on that** (D61). The oracle's sweep is one table holding both,
    /// and an instance's position in it comes from its address: measured,
    /// twenty runs of one class-side `UNINIT` on `::CLASS C` beside one live
    /// instance answered `instance` before `C` nineteen times and after it
    /// once.
    ///
    /// [`ClassRegistry::take_uninit_classes_in_sweep_order`]:
    ///     rexx_classes::ClassRegistry::take_uninit_classes_in_sweep_order
    pub(crate) fn run_termination_uninits(&mut self) -> Vec<Loud> {
        if self.processing_uninits {
            return Vec::new();
        }
        self.processing_uninits = true;
        let mut loud = Vec::new();
        for _ in 0..SWEEPS {
            let flagged = self.heap.take_uninit_flagged();
            self.run_uninit_batch(flagged, &mut loud);
            let classes = self.classes().take_uninit_classes_in_sweep_order();
            for class in classes {
                loud.extend(self.run_one_uninit(class));
            }
        }
        self.uninit_ready.clear();
        self.processing_uninits = false;
        loud
    }

    /// Puts one parked method body back and runs the rest of it.
    ///
    /// **SCHEDULING, and Phase 6 owns the whole function**, for
    /// [`Interp::run_deferred_replies`]'s reason. Only the `>I>`/`<I<`
    /// handling below is measured against the C++, and the C++ line it
    /// follows is itself on the reply-resume path, so it moves with whatever
    /// replaces this.
    ///
    /// **The level state is entered at nothing rather than restored**, and
    /// that is the resumed body's own shape rather than an omission: it has no
    /// sending clause left to be indented under, so the clause echoes start at
    /// indent 0 exactly as they do on the first half
    /// ([`Interp::enter_method_body`]'s own comment carries that
    /// measurement).
    ///
    /// **The resumed body announces a second `>I>` exactly when the first
    /// half announced one**, which is `RexxActivation::run`'s own reply arm:
    /// `traceEntryAllowed = traceEntryDone; traceEntryDone = false`
    /// (`execution/RexxActivation.cpp:562`-`:563`), and then the unconditional
    /// `traceEntry()` at `:600`. So [`TraceEntry::Done`] becomes
    /// [`TraceEntry::Allowed`] and anything else becomes
    /// [`TraceEntry::Spent`], and the announcement is asked for here rather
    /// than waiting for a `TRACE` instruction the remainder need not contain.
    /// Measured under `trace r`: a class method whose body is `trace r` /
    /// `guard on` / `reply "v"` / `say "tail"` / `return` produces `>I>`, the
    /// clauses, `<I<`, then `>I>` again before `say "tail"` and a final
    /// `<I<`.
    ///
    /// **`first_instruction_pending` does not go back with it.** A resumed
    /// clause is not the first instruction executed, so a `PROCEDURE` or a
    /// `USE LOCAL` there is the ordinary refusal.
    fn resume_reply(&mut self, deferred: DeferredReply) -> Result<(), Failure> {
        let DeferredReply {
            mut activation,
            context,
            slots,
            aliases,
            parked,
        } = deferred;
        let frame = self.roots.push_slots(slots.len());
        // Before the values, so that a promoted variable's write lands in
        // its cell and not in the slot the redirect stands in front of.
        self.roots.put_frame_aliases(frame, &aliases);
        for (index, value) in slots.iter().enumerate() {
            if let Some(value) = value {
                self.roots.set_frame_slot(frame, index, *value);
            }
        }
        // Released only once the arena holds the values again.
        //
        // **The context object is rooted by neither mechanism between here
        // and the `push_activation` below.** The park named it while the
        // activation was off the stack; `Interp::collect_now`'s sweep names
        // it once it is back on. Nothing between the two lines allocates a
        // Rexx object, so no collection can happen in that window today --
        // which makes this latent rather than live, and makes an allocation
        // added here the thing that would turn it live.
        self.roots.release(parked);
        activation.frame = frame;
        activation.trace_entry = if activation.trace_entry == TraceEntry::Done {
            TraceEntry::Allowed
        } else {
            TraceEntry::Spent
        };
        activation.first_instruction_pending = false;
        let saved_context = std::mem::replace(&mut self.call_context, context);
        let saved_clause_state = self.save_clause_state();
        let saved_base = std::mem::replace(&mut self.activation_indent, 0);
        let saved_offset = std::mem::take(&mut self.indent_offset);
        let saved_line = std::mem::take(&mut self.clause_line_override);
        self.push_activation(*activation);
        // After the push, because the announcement reads the running
        // activation's own trace mode and subject.
        self.trace_invocation_entry();

        let ended = self.run_activation();

        self.trace_invocation_exit();
        let callee = self.pop_activation().expect("the activation just pushed");
        self.activation_indent = saved_base;
        self.indent_offset = saved_offset;
        self.clause_line_override = saved_line;
        self.restore_clause_state(saved_clause_state);
        self.call_context = saved_context;
        // A resumed body cannot park again: `Interp::exec_reply` raises 98.935
        // on a second `REPLY` before it can set the state, so the frame is
        // released here unconditionally. The assertion is what makes a state
        // that stopped being unreachable announce itself rather than silently
        // dropping the body this branch has no queue entry for.
        debug_assert_ne!(
            callee.reply,
            ReplyState::Owed,
            "a resumed method body asked to be parked a second time"
        );
        self.roots.pop_slots(callee.frame);
        match ended {
            // Every ending is the same ending here: nothing is waiting for a
            // value, and a resumed body's `EXIT` does not set the process's
            // status. Measured, oracle rc 0: `reply 'v'` then `say 'tail'`
            // then `exit "boom"` is 98.937 with the main body's own rc kept.
            Ok(_) | Err(crate::Failure::Exited(_)) => Ok(()),
            Err(failure) => {
                self.seal_site_level();
                Err(failure)
            }
        }
    }

    /// [`Interp::resolve`] then [`Interp::invoke`], with
    /// [`Interp::unknown_or_nomethod`] behind them for a name the receiver's
    /// behaviour does not answer.
    ///
    /// Not a third step and not a fusion of the two: it is the composition
    /// every caller wants, in the shape `Interp::exec_call` already gives the
    /// classic-call pair.
    ///
    /// **"This phase builds no class for that receiver" and "that class does
    /// not answer this name" are two different answers, and only the second
    /// is 97.1.** A stem is the reachable case of the first: the oracle
    /// answers `a. = 'dflt'; say a.~length` with `4`, forwarding through
    /// `StemClass`'s own `UNKNOWN`, so raising a condition here would let a
    /// program *expecting* 97.1 pass against a gap -- the same argument
    /// [`Loud::unresolved_call`] makes for an excluded builtin.
    pub(crate) fn send_message(
        &mut self,
        receiver: ObjRef,
        name: &[u8],
        start_scope: Option<ObjRef>,
        args: &[Option<ObjRef>],
        caller: Caller,
    ) -> Result<Option<ObjRef>, Failure> {
        if let Err(kind) = self.receiver_kind(receiver) {
            return Err(Loud::receiver_class(kind).into());
        }
        match self.resolve(receiver, name, start_scope, caller) {
            Ok(resolution) => self.invoke(resolution, receiver, name, args),
            Err(miss) => self.unknown_or_nomethod(receiver, name, args, miss),
        }
    }

    /// **What the documented search order has left after the class chain**:
    /// `UNKNOWN` on the receiver's own behaviour, and the `NOMETHOD`
    /// condition beneath it when the behaviour answers no `UNKNOWN` either --
    /// `RexxObject::processUnknown` (`classes/ObjectClass.cpp:1002`), reached
    /// from `messageSend` at `:904`.
    ///
    /// **The forward's arguments are the missed message name and then an
    /// `Array` of the send's own arguments**, in that order (`:1013`, then
    /// `:1018`-`:1019`).
    /// The array is the argument list as the send holds it, omissions
    /// included: measured, `.k~zork(1,,3)` reaches an `UNKNOWN` whose
    /// `arguments~items` is `2` and whose `~size` is `3`, while `.k~zork()`
    /// answers `0` for each.
    ///
    /// **The `UNKNOWN` lookup is the ordinary one, carries no start scope and
    /// is not access-checked**, whatever the missed send's own scope override
    /// or access refusal was: `processUnknown` asks
    /// `behaviour->methodLookup(GlobalNames::UNKNOWN)` directly wherever
    /// `messageSend` reaches it, so [`Interp::lookup`] is the call and not
    /// [`Interp::resolve`]. Measured, oracle rc 0: a class whose only method
    /// is `::METHOD unknown CLASS PRIVATE` answers `.K~zork` sent from
    /// outside the class.
    ///
    /// `miss` is why the send had no method, which decides the report the
    /// receiver with no `UNKNOWN` gets.
    ///
    /// Off every hot path by construction -- a send that resolves never
    /// arrives here -- so the body is out of line rather than folded into
    /// [`Interp::send_message`]'s `match`.
    #[cold]
    #[inline(never)]
    fn unknown_or_nomethod(
        &mut self,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
        miss: Miss,
    ) -> Result<Option<ObjRef>, Failure> {
        let Some(resolution) = self.lookup(receiver, UNKNOWN, None) else {
            return Err(self.nomethod(receiver, name, miss));
        };
        // **Each of the forward's arguments is rooted, and only the array's
        // root has a witness.** `alloc_with` collects *before* it allocates,
        // so the array's root is what carries it across the `text` below -- a
        // message name too long for a handle allocates there, and
        // `corpus/lang/message_send_unknown_forward.rex`'s last row is that
        // name. Measured with `collect_stress.rs`'s
        // collect-on-every-allocation and this root removed: that row's own
        // `a~items` then sends to a collected array and
        // `Interp::receiver_kind` refuses it, so the stress run exits 120
        // with `a message send to a value whose object is no longer live`
        // where the plain run prints the row. A message name of seven bytes
        // or fewer leaves the subset green, because `Interp::text` inlines it
        // and nothing allocates between the array and the send.
        //
        // `missed`'s root has no such witness and stays because it is the
        // only root the value has: the slice handed to `invoke` is not one,
        // nothing else holds the name, and `invoke` runs a whole method body.
        // Measured, its removal reddens nothing today, which says `invoke`
        // reaches its argument binding without allocating and not that the
        // value is safe unrooted.
        //
        // `args` themselves need nothing here: the send site evaluated them
        // and rooted each one as it went (`Interp::eval_traced_argument`),
        // which is the same rooting every other allocation reached from a
        // send already relies on.
        let frame = self.roots.push_frame();
        let arguments = self.alloc_with(BehaviourId::ARRAY, Body::array(args.to_vec()));
        self.roots.push_temp(arguments);
        let missed = self.text(name);
        self.roots.push_temp(missed);
        let forwarded = self.invoke(
            resolution,
            receiver,
            UNKNOWN,
            &[Some(missed), Some(arguments)],
        );
        self.roots.pop_frame(frame);
        forwarded
    }

    /// The condition a send raises when the receiver's behaviour answers
    /// neither the message nor `UNKNOWN`, and **which condition that is
    /// depends on what is armed**.
    ///
    /// **`miss` decides the syntax error and not the condition.**
    /// `reportNomethod` takes the error code `messageSend` set and offers the
    /// same `NOMETHOD` condition whichever it is (`:904`, `:1009`), so an
    /// access refusal that nothing traps is 97.2 or 97.3 in place of 97.1
    /// while a trapping handler reads the identical items. Measured, oracle
    /// rc 0 on a `PRIVATE` class method sent from the program body under
    /// `signal on nomethod`: `CONDITION('C')` `NOMETHOD`, `CONDITION('D')`
    /// `M`, `CONDITION('E')` the null string and `RC` untouched, which is
    /// 97.1's own row.
    ///
    /// `reportNomethod` (`concurrency/ActivityManager.hpp:509`) offers a
    /// `NOMETHOD` condition first and raises the 97.1 syntax error only when
    /// nothing took it, so they are separate answers a program can tell
    /// apart. Measured, `say 'abc'~nosuchmsg`: under `signal on nomethod` it
    /// traps with `CONDITION('C')` `NOMETHOD`, `CONDITION('D')` `NOSUCHMSG`,
    /// `CONDITION('E')` the null string and `RC` untouched; under `signal on
    /// syntax` alone it traps with `C` `SYNTAX`, `D` the null string, `E` `1`
    /// and `RC` `97`.
    ///
    /// **The offer is to the running activation's own table and goes no
    /// further** -- [`Interp::trap_for`]'s question, not a walk of the
    /// activation stack. An internal `CALL` inherits its caller's traps by
    /// copy ([`Activation::traps`]), so a caller's `SIGNAL ON NOMETHOD` does
    /// take a miss raised inside such a callee, and beats a `SIGNAL ON
    /// SYNTAX` that callee armed for itself. A `::METHOD` activation inherits
    /// none, and that is where the readings part: measured, with `signal on
    /// nomethod` and `signal on syntax` both armed in the main body and the
    /// miss inside a `::METHOD` body, the oracle runs the **`SYNTAX`**
    /// handler, where a version asking the whole stack runs the `NOMETHOD`
    /// one.
    ///
    /// **The choice belongs here and not in [`Interp::offer_to_trap`]**: that
    /// function sees each activation in turn as the failure unwinds, and a
    /// `NOMETHOD` condition declined by the activation that raised it must
    /// become the 97.1 *there*, in time for that same activation's `SYNTAX`
    /// trap to take it -- measured, a routine whose own `SIGNAL ON SYNTAX` is
    /// the only trap armed anywhere does take its own missed send.
    ///
    /// A `CALL ON` trap does not count. There is nothing to resume into once
    /// a clause has failed, and `TrapHandler::canHandle`
    /// (`execution/TrapHandler.cpp:118`) refuses this whole family to a `CALL
    /// ON ANY`: measured, `call on any name h` over `say 'abc'~nosuchmsg` is
    /// the untrapped 97.1 at rc 159 where `signal on any` traps it as
    /// `NOMETHOD`. Excluding it here is what makes the gate the same test
    /// `offer_to_trap` applies a moment later, which is the shape
    /// [`Interp::novalue_raised`] describes for its own condition.
    fn nomethod(&mut self, receiver: ObjRef, name: &[u8], miss: Miss) -> Failure {
        let target = self.message_target_text(receiver);
        let report = match miss {
            Miss::NoMethod => Raised::no_method(&target, name),
            Miss::Private => Raised::private_method(&target, name),
            Miss::PackageScope => Raised::package_scope_method(&target, name),
        };
        if self.trap_for(b"NOMETHOD").is_some_and(|trap| !trap.call) {
            Raised::nomethod(report, name).into()
        } else {
            report.into()
        }
    }

    /// The receiver as 97.1 names it: `stringValue()`, which is **not** the
    /// string value [`Interp::to_text`] answers for every receiver.
    ///
    /// An array is where the two part. `RexxObject::stringValue` for an array
    /// is [`ARRAY_DEFAULT_NAME`], while a string context reaches
    /// `ArrayClass::makeString` and gets the items joined
    /// (`classes/ArrayClass.cpp:1841`). Measured, `a = .Array~superClasses`:
    /// `say a + 1` reports `Object "an Array" does not understand message
    /// "+".` where `say 'x'a` prints the two class names on two lines.
    ///
    /// [`Interp::string_value_text`] is the whole of it, and this name is what
    /// says which question the 97.1 site is asking.
    fn message_target_text(&mut self, receiver: ObjRef) -> Vec<u8> {
        self.string_value_text(receiver)
    }

    /// One whole `target~name(...)` term: the receiver, the scope override,
    /// the arguments, the send, and `~~`'s replacement of the result by the
    /// target.
    ///
    /// **The evaluation order is the oracle's and is observable under
    /// `TRACE I`**: receiver, then the scope override, then the arguments
    /// left to right, each tracing its own `>A>` line -- omitted positions
    /// included, measured, `'abc'~nosuch(,1)` traces `>A>   ""` then
    /// `>A>   "1"`. The scope override is validated **before** the arguments
    /// are evaluated, so a bad scope refuses without them
    /// (`RexxExpressionMessage::evaluate`, `ExpressionMessage.cpp:158-181`).
    pub(crate) fn message_term(
        &mut self,
        code: &crate::Code<'_>,
        term: &MessageTerm<'_>,
    ) -> Result<Option<ObjRef>, Failure> {
        let MessageTerm {
            target,
            name,
            super_class,
            args,
            cascade,
            assigned,
        } = *term;
        let receiver = self.eval(code, target)?;
        self.roots.push_temp(receiver);

        let start_scope = match super_class {
            None => None,
            Some(super_class) => {
                // Evaluated for its own trace lines and its own failures
                // before either check below, which is the oracle's order.
                let scope = self.eval(code, super_class)?;
                self.roots.push_temp(scope);
                // `RexxExpressionMessage::evaluate`'s
                // `_super->isInstanceOf(TheClassClass)`, then
                // `_target->validateScopeOverride(_super)`. Both run before
                // the arguments are evaluated.
                if scope.class_id().is_none() {
                    return Err(Raised::scope_override_not_a_class().into());
                }
                self.validate_scope_override(receiver, Some(scope))?;
                Some(scope)
            }
        };

        let mut values = self.take_value_buffer();
        let evaluated = self.evaluate_message_arguments(code, args, assigned, &mut values);
        let caller = self.caller();
        let result = evaluated
            .and_then(|()| self.send_message(receiver, name, start_scope, &values, caller));
        self.give_value_buffer(values);
        let sent = result?;

        // `~~` replaces the result with the target **before** the send is
        // traced, so `>M>` shows the target for a cascade -- measured,
        // `'abc'~~length` traces `>M>   "LENGTH" => "abc"`
        // (`ExpressionMessage.cpp:202-219`). It also gives a cascade a value
        // where the send itself had none, measured: `.K~~m` on a method
        // ending in a bare `return` yields the target at rc 0.
        let value = if cascade { Some(receiver) } else { sent };
        // **The only `>M>` site.** Emitted here rather than from `eval`'s own
        // post-order `trace_intermediate` hook, where every other value
        // prefix lives, because the message-assignment form
        // (`Interp::exec_message`) never evaluates its term as an expression
        // and so never reaches that hook. Emitting from the one function
        // both forms do pass through is what keeps them from disagreeing.
        // The position is the same either way: this is the last thing the
        // term does before returning its value.
        //
        // **A send that produced no value traces no line at all**, measured
        // under `trace i`: a whole-clause `.K~m` on a method ending in a bare
        // `return` emits the clause echo and its `>E>` and nothing else.
        if let Some(value) = value
            && let Some(rendered) = self.intermediate_text(value)
        {
            self.trace_message(self.clause_state.current_value_indent, name, &rendered);
        }
        Ok(value)
    }

    /// The argument loop [`Interp::message_term`] runs, split out so the
    /// borrowed value buffer is returned on the failure path too.
    fn evaluate_message_arguments(
        &mut self,
        code: &crate::Code<'_>,
        args: &[Option<Expr>],
        assigned: Option<&Expr>,
        values: &mut Vec<Option<ObjRef>>,
    ) -> Result<(), Failure> {
        if let Some(expr) = assigned {
            values.push(Some(self.eval_traced_argument(code, expr)?));
        }
        for arg in args {
            match arg {
                // An omitted position traces an empty value line, not no
                // line -- the same rule a call's own argument list follows.
                None => {
                    self.trace_argument(self.clause_state.current_value_indent, b"");
                    values.push(None);
                }
                Some(expr) => values.push(Some(self.eval_traced_argument(code, expr)?)),
            }
        }
        Ok(())
    }

    /// Records the traceback line a failing native method contributes, and
    /// closes the level so the sending clause records its own.
    ///
    /// The oracle's catalogue entry
    /// (`Message_Translations_compiled_method_invocation`) is a whole line
    /// including its own `*-*` marker and the blank line-number field in
    /// front of it, so the bytes are rendered here rather than assembled by
    /// `trace::push_clause`. Measured, the line carries no indent even for a
    /// send two `DO` levels deep.
    ///
    /// `pub(crate)` rather than private to this module, because the rule for
    /// who calls this is not "who wrote a send term": **the line is owed
    /// wherever the oracle reached the failing native method's body by a
    /// message send**, whatever put that send there. A `~` in the source is
    /// the obvious route and not the only one -- a stem forwarding an
    /// operator to its default value reaches the method through
    /// `value->messageSend` with no send term anywhere in sight
    /// (`classes/StemClass.cpp:280`).
    ///
    /// **Being inside a native method's body is not the condition; having
    /// been sent to is**, and `ClassDirective::install` is the discriminating
    /// pair by itself. It reaches `INHERIT` by `classObject->sendMessage`
    /// (`instructions/ClassDirective.cpp:230`) and that send's refusals carry
    /// this line; it *calls* `subclass()` and `mixinClass()` (`:205`, `:200`)
    /// and the 99.927 those raise carries none -- both measured, on one
    /// directive, and `RexxClass::subclass` is the very body a `~subclass`
    /// send would have entered. So a caller reaching a native method by a
    /// direct call owes nothing here, and reading the condition as "raises
    /// from inside a native method" over-predicts exactly there.
    pub(crate) fn blame_native_method(&mut self, name: &[u8], scope: &str) {
        if self.failure_site.is_some() {
            return;
        }
        let line = Raised::compiled_method_line(name, scope);
        self.failure_site = Some(FailureSite::Rendered(line));
        self.seal_site_level();
    }
}

/// What the protocol's conversion limbs answered, before anything is built.
///
/// The [`StringConversion::Bytes`] split, one layer up: the caller that only
/// reads the bytes must not allocate an object to read them out of.
enum RequiredString {
    Object(ObjRef),
    Bytes(Vec<u8>),
}

/// The message `Object~request("STRING")` looks for, and the one limb of the
/// required-string protocol a program can write.
pub(crate) const MAKESTRING: &[u8] = b"MAKESTRING";

/// What `RexxObject::stringValue` sends (`classes/ObjectClass.cpp:1157`).
pub(crate) const OBJECTNAME: &[u8] = b"OBJECTNAME";

/// What `RexxObject::objectName` sends for an object nothing has named and
/// whose class is not a base class (`classes/ObjectClass.cpp:1712`).
pub(crate) const DEFAULTNAME: &[u8] = b"DEFAULTNAME";

/// The required-string protocol's last limb, past every conversion
/// (`RexxInternalObject::requiredString`, `classes/ObjectClass.cpp:1353`).
pub(crate) const STRING: &[u8] = b"STRING";

/// What a value's own string value is, before the protocol's fallbacks: the
/// value itself, the object a `makeString` answered, bytes no object holds, or
/// nothing.
///
/// The `Array` arm is why this is a value rather than a `bool`: an array's
/// string value is its items joined, which no object holds, so a caller that
/// wants one has to build it.
enum StringConversion {
    /// The object that *is* the string value: the value itself, or a stem's
    /// own default value, or what a `makeString` answered.
    Object(ObjRef),
    /// The string value as bytes no object holds -- an array's items joined.
    ///
    /// **Separate from the arm above rather than built into a string here**,
    /// so that a caller which only wants to read the bytes does not allocate
    /// an object to read them out of again. `Interp::required_string_value`'s
    /// own debug check is that caller, and a heap allocation there would
    /// change what `tests/collect_stress.rs` observes between a debug build
    /// and a release one.
    Bytes(Vec<u8>),
    /// The receiver's behaviour has a `makeString` to send.
    MakeString,
    /// None of those, which is `.nil` from `requestString`'s point of view.
    None,
}

impl Interp {
    /// The required-string protocol, D52: `request("STRING")`, then the
    /// receiver's `makeString`, then the NOSTRING condition if trapped, else
    /// `defaultName`. **This is the value every context `provide.xml`
    /// `reqstr` lists renders, and rendering it without coming through here
    /// is the silent wrong answer the protocol exists to prevent.**
    ///
    /// `RexxInternalObject::requestString` (`classes/ObjectClass.cpp:1235`).
    /// The answer is a *value*, not bytes, so the caller renders it with the
    /// same [`Interp::to_text`] it always did and nothing on the rendering
    /// path becomes fallible.
    ///
    /// **The latch is what keeps this off the hot path.** With no
    /// `makeString` installed anywhere and no NOSTRING trap ever armed, the
    /// protocol's first limb cannot answer differently from the value itself
    /// and its third cannot fire, so the value stands and the whole of the
    /// walk below is skipped. A debug build runs the walk anyway and insists
    /// it agrees, which is the check that would catch a latch that misses an
    /// arming route -- see [`Interp::reqstr_armed`].
    ///
    /// [`Interp::reqstr_armed`]: crate::Interp::reqstr_armed
    #[inline]
    pub(crate) fn required_string_value(&mut self, value: ObjRef) -> Result<ObjRef, Failure> {
        if self.reqstr_armed {
            return self.required_string_dispatch(value);
        }
        debug_assert!(
            self.required_string_latch_holds(value),
            "the required-string latch is off where the protocol would answer differently \
             or raise"
        );
        Ok(value)
    }

    /// Whether the fast path is the right answer for `value` with the latch
    /// clear -- one test per limb the latch claims cannot fire.
    ///
    /// **A wrongly clear latch is a wrong answer, not a slow one**, and it is
    /// wrong independently at limb 1 and at limb 3, so this asks about each.
    /// Limb 1 can answer a different string than the value renders as, and
    /// limb 3 can raise where the fast path renders. A check covering only the
    /// first is silent for a route that arms a trap, which is exactly what the
    /// limb-3 test below is for.
    ///
    /// **This detects; it does not protect.** It runs under `debug_assert`, so
    /// a release build has nothing here: release correctness rests entirely on
    /// [`Interp::reqstr_armed`]'s arming sites being complete, and this is the
    /// instrument that says whether they are when a debug build runs the same
    /// program.
    ///
    /// **The bytes and not the identity**, for limb 1. The protocol's last
    /// limb builds a fresh string out of `stringValue()`, so an object never
    /// comes back as itself even when nothing has changed; what the latch
    /// claims is that rendering the value the caller already holds gives the
    /// same answer, and that is what this compares.
    ///
    /// **Not `#[cfg(debug_assertions)]`**: `debug_assert!` type-checks its
    /// expression in every profile, so the function has to exist in a release
    /// build even though nothing there calls it.
    ///
    /// [`Interp::reqstr_armed`]: crate::Interp::reqstr_armed
    fn required_string_latch_holds(&mut self, value: ObjRef) -> bool {
        // Limb 3's route, and it is not about `value` at all: a trap that
        // could take NOSTRING while the latch is clear is wrong for every
        // rendered object, because `Interp::exec_condition_trap` is what arms
        // both and a clear latch here means it did not. The gate is the same
        // one `required_string_dispatch` applies, so this fires exactly where
        // that would have raised.
        if self.trap_for(b"NOSTRING").is_some_and(|trap| !trap.call)
            || self.condition_raises_syntax(b"NOSTRING")
        {
            return false;
        }
        // Limb 1's route: the conversion limbs, run in full, against the
        // rendering the fast path hands back instead.
        let expected = self.to_text(value).into_owned();
        let answered = match self.required_string_answer(value) {
            Ok(answered) => answered,
            Err(_) => return false,
        };
        let bytes = match answered {
            Some(RequiredString::Object(text)) => self.to_text(text).into_owned(),
            Some(RequiredString::Bytes(bytes)) => bytes,
            // `stringValue()`, which is what the fast path's own rendering of
            // an object with no string value comes to.
            None => self.string_value_text(value),
        };
        bytes == expected
    }

    /// **The answer is rooted here and not at the call sites.** A `makeString`
    /// can allocate and can collect, and both fallbacks below build a string
    /// of their own, so every value this returns is one the caller did not
    /// have a root for. Rooting once, on the arm that can produce a fresh
    /// object, is what keeps the gate above free of a temp push per rendered
    /// value.
    #[cold]
    #[inline(never)]
    fn required_string_dispatch(&mut self, value: ObjRef) -> Result<ObjRef, Failure> {
        match self.required_string_answer(value) {
            Ok(Some(RequiredString::Object(text))) => {
                self.roots.push_temp(text);
                return Ok(text);
            }
            Ok(Some(RequiredString::Bytes(bytes))) => {
                let text = self.text_built(bytes);
                self.roots.push_temp(text);
                return Ok(text);
            }
            Ok(None) => {}
            Err(failure) => {
                self.blame_request();
                return Err(failure);
            }
        }
        // `sendMessage(STRING)`. An instance's `STRING` can be a Rexx
        // method, so it is sent; the shortcut below is what the send answers
        // wherever `STRING` resolves to `native_string`. Measured, oracle
        // rc 0: with `::METHOD string` returning `from-string`, `say o`,
        // `'x' o` and `length(o)` all follow it.
        //
        // **No `REQUEST` frame when the send raises**, unlike the conversion
        // limbs above: measured, oracle rc 214, `::METHOD string` ending in
        // `1/0` reports the method's own clause and then the sending clause,
        // and nothing between them.
        let readable = if matches!(self.receiver_kind(value), Ok(Primitive::Instance { .. })) {
            let caller = self.caller();
            match self.send_message(value, STRING, None, &[], caller)? {
                Some(answered) => self.string_value_text(answered),
                None => self.string_value_text(value),
            }
        } else {
            self.string_value_text(value)
        };
        // Gated on the trap rather than raised unconditionally, the shape
        // `Interp::novalue_raised` describes: an untrapped NOSTRING resumes
        // with the readable rendering, so a raise nothing can take would
        // build a condition per rendered object and then throw it away. A
        // `CALL ON` trap is excluded because `CALL ON NOSTRING` is a parse
        // error and `CALL ON ANY` is measured not to catch a condition with
        // no resumption point.
        if self.trap_for(b"NOSTRING").is_some_and(|trap| !trap.call) {
            return Err(Raised::nostring(&readable).into());
        }
        // `::OPTIONS NOSTRING SYNTAX`, and only where nothing trapped: the
        // trap wins, measured -- `signal on nostring` over `say .array` in
        // such a file runs the handler, and `signal on syntax` takes 98.973.
        if self.condition_raises_syntax(b"NOSTRING") {
            return Err(Raised::nostring_syntax(&readable).into());
        }
        let readable = self.text_built(readable);
        self.roots.push_temp(readable);
        Ok(readable)
    }

    /// Every supplied argument of one builtin call **except the positions
    /// `crate::builtin::raw_argument_positions` exempts for `name`**, in
    /// position order, through the required-string protocol -- `None` when the
    /// protocol cannot change any of them, so the caller passes its own list
    /// on.
    ///
    /// **`provide.xml` `reqstr` names "arguments to built-in functions"
    /// wholesale, and the oracle converts a position when the builtin fetches
    /// it through a converting accessor** -- `ExpressionStack::requiredStringArg`
    /// (`expression/ExpressionStack.cpp:142`) and
    /// `ExpressionStack::optionalStringArg` (`:167`), each of which calls
    /// `argument->requestString()`. A position it fetches with `stack->peek`
    /// instead is never converted, and
    /// `crate::builtin::raw_argument_positions` is the enumeration of those.
    ///
    /// Converting the rest up front reproduces the order every fetch that
    /// follows position order sees, and it reaches an argument the builtin
    /// does not go on to use -- measured on `SUBSTR`'s pad, which the oracle
    /// fetches through `optional_pad` whether or not the length reaches past
    /// the subject: `substr('abc', 1, 2, .P)` with a class-side `makeString`
    /// on `.P` prints `pad asked` at rc 0.
    ///
    /// **Converting a position at most once is the oracle's own property, not
    /// an economy here**: both of those write the converted string back with
    /// `replace(position, newStr)` (`:154`, `:186`), so a builtin that fetches
    /// one position twice converts it once. Measured on
    /// `XRANGE`, whose loop passes over its arguments twice:
    /// `xrange('a', .K, 'x', 'z')` prints `K asked` exactly once on both
    /// sides at rc 0.
    ///
    /// **After the 40.x count checks and before the builtin's own argument
    /// validation**, measured on both sides of that line: `date('S', , .Z)`
    /// prints `Z asked` and *then* raises 40.5, while
    /// `substr(.A, .B)` with `.B` answering `'x'` converts both and then
    /// raises 40.12 naming `The B class` -- an argument's own error names the
    /// object, because the oracle's error path has the object in hand and not
    /// the conversion.
    pub(crate) fn required_string_arguments(
        &mut self,
        name: &'static [u8],
        args: &[Option<ObjRef>],
    ) -> Result<Option<Vec<Option<ObjRef>>>, Failure> {
        if !self.reqstr_armed {
            return Ok(None);
        }
        // Looked up behind the latch, not in front of it: an ordinary call
        // pays one bool test here and nothing else, where a lookup at the call
        // site would scan the exemption table on every builtin call in every
        // program.
        let raw = crate::builtin::raw_argument_positions(name);
        let mut converted = Vec::with_capacity(args.len());
        for (index, argument) in args.iter().enumerate() {
            let position = index + 1;
            converted.push(match argument {
                None => None,
                Some(value) if raw.contains(&position) => Some(*value),
                Some(value) => Some(self.required_string_value(*value)?),
            });
        }
        Ok(Some(converted))
    }

    /// The protocol's conversion limbs, answered without building anything --
    /// [`Interp::string_conversion`]'s own body with the materialisation left
    /// to the caller.
    ///
    /// **Its callers want different things from it.**
    /// [`Interp::required_string_dispatch`] wants an object, because its own
    /// caller renders one; the debug check behind
    /// [`Interp::required_string_value`]'s latch wants only the bytes, and
    /// allocating an object there would make a debug build collect where a
    /// release build does not -- which `tests/collect_stress.rs` compares
    /// against a committed list.
    fn required_string_answer(&mut self, value: ObjRef) -> Result<Option<RequiredString>, Failure> {
        Ok(match self.classify_string_conversion(value) {
            StringConversion::Object(text) => Some(RequiredString::Object(text)),
            StringConversion::Bytes(bytes) => Some(RequiredString::Bytes(bytes)),
            StringConversion::None => None,
            StringConversion::MakeString => {
                // `string_value = string_value->primitiveMakeString()`
                // (`:1257`, `:1353`): what `makeString` answered has to be a
                // real string, and an object that is not one converts here or
                // counts as no answer at all. Measured, oracle rc 0 on
                // `say .K` with a class-side `makeString`: `return 5` prints
                // `5`, `return .Object~superClasses` prints the empty line
                // an empty array joins to, and `return .array` prints
                // `The K class` -- the third fell back.
                match self.send_make_string(value)? {
                    Some(answered) => match self.classify_string_conversion(answered) {
                        StringConversion::Object(text) => Some(RequiredString::Object(text)),
                        StringConversion::Bytes(bytes) => Some(RequiredString::Bytes(bytes)),
                        StringConversion::MakeString | StringConversion::None => None,
                    },
                    None => None,
                }
            }
        })
    }

    /// `RexxInternalObject::requiredString()` (`classes/ObjectClass.cpp:1341`):
    /// the protocol's conversion limbs alone, with **no** `~string` fallback
    /// and **no** NOSTRING condition. `None` is the oracle's `.nil`.
    ///
    /// This is what a method argument that must be text gets
    /// ([`required_string_argument`]) and what `Object~request("STRING")`
    /// answers.
    pub(crate) fn string_conversion(&mut self, value: ObjRef) -> Result<Option<ObjRef>, Failure> {
        Ok(match self.required_string_answer(value)? {
            Some(RequiredString::Object(text)) => Some(text),
            Some(RequiredString::Bytes(bytes)) => Some(self.text_built(bytes)),
            None => None,
        })
    }

    /// [`Interp::string_conversion`] with the `REQUEST` traceback line on it,
    /// for a caller inside a native method's own activation -- see
    /// [`Interp::blame_request`] for why the two frames stack there.
    fn blamed_string_conversion(&mut self, value: ObjRef) -> Result<Option<ObjRef>, Failure> {
        let converted = self.string_conversion(value);
        if converted.is_err() {
            self.blame_request();
        }
        converted
    }

    /// Which limb of the protocol answers for `value`.
    ///
    /// **`primitiveMakeString` is asked first, where the oracle asks
    /// `isBaseClass()` first**, and the two split the same set for every value
    /// this phase builds: a receiver whose `MAKESTRING` is a
    /// [`NativeMethod`] is one of the primitives the arms below answer for,
    /// and its `primitiveMakeString` answers the same bytes that method would,
    /// so taking the primitive answer is the oracle's own shortcut and reaches
    /// the same string without a send. The only `MAKESTRING` a program can
    /// install is a Rexx one, and it can only be installed on a receiver no
    /// arm below answers for.
    fn classify_string_conversion(&mut self, value: ObjRef) -> StringConversion {
        let redirect = match value.decode() {
            Decoded::Nil => return self.make_string_or_none(value),
            // `RexxString::primitiveMakeString` and
            // `RexxInteger::primitiveMakeString`: a string and a number are
            // their own string value.
            Decoded::SmallInt(_) | Decoded::Text(_) => {
                return StringConversion::Object(value);
            }
            // Asked before the arena is, for the reason `receiver_kind`
            // gives: a class identity is heap-tagged and names no slot.
            Decoded::Heap { .. } if value.class_id().is_some() => {
                return self.make_string_or_none(value);
            }
            Decoded::Heap { .. } => match self.heap.get(value) {
                // A handle whose slot is gone, which `Interp::to_text` turns
                // into its own tripwire. Answering the value keeps that the
                // one report rather than adding a second.
                None => return StringConversion::Object(value),
                Some(object) => match &object.body {
                    Body::Text { .. } | Body::Num { .. } => {
                        return StringConversion::Object(value);
                    }
                    // `StemClass::makeString` forwards to the default value,
                    // and a stem holding none is its own derived name, which
                    // is what `to_text` renders for it.
                    Body::Stem { default: None, .. } => {
                        return StringConversion::Object(value);
                    }
                    Body::Stem {
                        default: Some(default),
                        ..
                    } => Some(*default),
                    // `ArrayClass::makeString` (`classes/ArrayClass.cpp:1841`),
                    // the items joined by a newline -- **not**
                    // `stringValue()`, which is `an Array`.
                    Body::Array { .. } => None,
                    // **The referent's `stringValue()`, not its
                    // conversion.** `requestString` looks `MAKESTRING` up in
                    // the receiver's own behaviour rather than sending it,
                    // and a reference's behaviour holds no such name, so
                    // limb 4 answers `VariableReference::stringValue`
                    // (`classes/VariableReference.cpp:249`), which is the
                    // referent's. Measured, oracle rc 0: `say 'p' >zz` over a
                    // two-item array is `p an Array` where `say 'p' zz` joins
                    // the items. `~request('STRING')` is a different route
                    // and does forward -- `native_reference_request`.
                    Body::VarRef(_) => {
                        let text = crate::value::string_value_of_reference(self, value);
                        return StringConversion::Bytes(text);
                    }
                    _ => return self.make_string_or_none(value),
                },
            },
        };
        match redirect {
            Some(default) => self.classify_string_conversion(default),
            None => StringConversion::Bytes(self.string_conversion_array_text(value)),
        }
    }

    /// [`Interp::classify_string_conversion`]'s array arm, split out so the
    /// borrow of the heap object above ends before the join runs.
    fn string_conversion_array_text(&mut self, value: ObjRef) -> Vec<u8> {
        let items = self.array_slots_of(value).unwrap_or_default();
        self.array_string(&items, b"\n")
    }

    /// Whether the receiver's own behaviour answers `MAKESTRING`.
    ///
    /// `RexxObject::requestRexx` (`classes/ObjectClass.cpp:1912`) forms
    /// `MAKE` + the class name and asks `behaviour->methodLookup` for it --
    /// the unchecked lookup, so a `PRIVATE makeString` is found here and
    /// refused by the send, which is that function's own order. A receiver
    /// this phase builds no behaviour for has none to ask and answers
    /// nothing.
    fn make_string_or_none(&mut self, value: ObjRef) -> StringConversion {
        match self.lookup(value, MAKESTRING, None) {
            Some(_) => StringConversion::MakeString,
            None => StringConversion::None,
        }
    }

    /// Sends `makeString` on the receiver's behalf.
    ///
    /// **No traceback frame of its own**, because the frame belongs to the
    /// `REQUEST` activation the oracle runs *around* this send and not to the
    /// send: a caller that is itself `Object~request` already gets that line
    /// from `Interp::invoke`, and one that reaches the protocol from a
    /// language context has no native activation and owes it --
    /// [`Interp::blame_request`] is that caller's own call.
    fn send_make_string(&mut self, receiver: ObjRef) -> Result<Option<ObjRef>, Failure> {
        // The sending side is the frame the conversion happens in, not the
        // conversion itself: `checkPrivate` asks
        // `getTopStackFrame()->getReceiver()`, and `requestString` runs no
        // frame of its own that could answer that question differently.
        let caller = self.caller();
        self.send_message(receiver, MAKESTRING, None, &[], caller)
    }

    /// The traceback line the oracle's own `REQUEST` activation contributes
    /// when a `makeString` reached through the protocol fails.
    ///
    /// **The frame is `REQUEST`'s and not `MAKESTRING`'s**, measured: with a
    /// class-side `makeString` whose body is `return 1/0`, `say .K~makeString`
    /// reports the failing clause and then the sending clause, while
    /// `say .K` and `say length(.K)` put
    /// `*-* Compiled method "REQUEST" with scope "Object".` between them.
    /// `requestString` reaches `makeString` through
    /// `sendMessage(GlobalNames::REQUEST, GlobalNames::STRING)`
    /// (`classes/ObjectClass.cpp:1256`), and that native activation is what
    /// owns the line.
    ///
    /// A method argument's conversion gets this line **and** the method's own
    /// on top of it, measured: `'abc'~hasMethod(.K)` reports the failing
    /// clause, then `REQUEST` with scope `Object`, then `HASMETHOD` with
    /// scope `Object`, then the sending clause.
    fn blame_request(&mut self) {
        self.blame_native_method(b"REQUEST", "Object");
    }
}

/// `RexxInternalObject::requiredString(position)`
/// (`classes/ObjectClass.cpp:1373`): a method argument the method needs as
/// text, converted through the required-string protocol, or 88.909 for a
/// value that has no string value at all.
///
/// This is `stringArgument` (`runtime/MethodArguments.hpp:136`), which is
/// `provide.xml` `reqstr`'s "for all other methods" rule: `request("STRING")`
/// and an error when it answers `.nil`. **`~string` and the NOSTRING
/// condition are `requestString`'s limbs and not this one's** --
/// `requiredString` stops where the conversion fails, which is why an object
/// with no string value is an error here and a readable rendering in a `SAY`.
///
/// Measured, three descriptors against the oracle:
///
/// ```text
/// 'abc'~hasMethod(5)                oracle `0` rc 0    a number has one
/// 'abc'~hasMethod(a.)               oracle `0` rc 0    an unset stem is its own name
/// 'abc'~hasMethod(.nil)             oracle 88.909 rc 168
/// 'abc'~hasMethod(.String)          oracle 88.909 rc 168
/// 'abc'~hasMethod(.environment)     oracle 88.909 rc 168
/// a. = .array; 'abc'~hasMethod(a.)  oracle 88.909 rc 168
/// a. = .nil;   'abc'~hasMethod(a.)  oracle 88.909 rc 168
/// ```
///
/// and the row a `makeString` puts on the other side of that line: with
/// `::CLASS K` plus `::METHOD makeString CLASS` returning `'LENGTH'`,
/// `'abc'~hasMethod(.K)` is `1` at rc 0.
///
/// **Not [`Interp::operator_operand_gap`], and the difference is `.nil`.**
/// That predicate passes `.nil` through as text on purpose, because an
/// operator there compares its rendering; `requiredString` refuses it.
///
/// **Scoped to this argument rather than to native arguments in general**:
/// the surface where each native checks its own is owned by the phase named
/// against the argument row in `docs/superpowers/plans/phase-4-exclusions.txt`.
fn required_string_argument(
    interp: &mut Interp,
    value: ObjRef,
    position: usize,
) -> Result<ObjRef, Failure> {
    match interp.blamed_string_conversion(value)? {
        Some(text) => Ok(text),
        None => Err(Raised::argument_needs_a_string_value(position).into()),
    }
}

/// [`required_string_argument`] for an argument the oracle's 88.909 names
/// rather than numbers.
fn required_string_named_argument(
    interp: &mut Interp,
    value: ObjRef,
    argument: &'static str,
) -> Result<ObjRef, Failure> {
    match interp.blamed_string_conversion(value)? {
        Some(text) => Ok(text),
        None => Err(Raised::named_argument_needs_a_string_value(argument).into()),
    }
}

/// `RexxObject::initRexx` (`classes/ObjectClass.cpp:2546`-`:2549`): it takes
/// no arguments, does nothing, and answers `OREF_NULL`.
///
/// **Answering nothing is observable and is what the oracle answers**:
/// measured, `say .K~init` under a lone `::CLASS K` is rc 165, `No result
/// object.` and `Message "INIT" did not return a result.`
fn native_no_op(
    _interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(None)
}

/// `Object~hasMethod(name)`: whether the receiver's behaviour answers `name`.
///
/// The argument is upcased before the lookup, measured:
/// `'abc'~hasMethod('length')` is `1`. An argument with no string value is
/// 88.909 rather than an answer of `0`, measured -- see
/// [`required_string_argument`] for which shapes those are.
fn native_has_method(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_method_argument(1).into());
    };
    let argument = required_string_argument(interp, argument, 1)?;
    let name = String::from_utf8_lossy(&interp.to_text(argument).to_ascii_uppercase()).into_owned();
    // **The object's own dictionary answers first**, so a `setMethod` name
    // reports `1` and a hidden one reports `0` even where the class defines
    // it -- measured, oracle rc 0 for the first and rc 159 for the send
    // after the second.
    if let Some(entry) = interp.own_method_entry(receiver, name.as_bytes()) {
        return Ok(Some(interp.counted(usize::from(entry.is_some()))));
    }
    // **A receiver with no class here is loud, not `0`.** `send_message`
    // screens for it before any method runs, so this arm is unreachable
    // today; answering `0` from it anyway would make this the one place in
    // the module where a gap becomes an answer, and the answer would be one
    // the oracle contradicts.
    // **Which dictionary is asked follows the receiver**, measured on
    // `::class K` plus `::method m class`: `.K~hasMethod('M')` is `1` and
    // `.K~hasMethod('LENGTH')` is `0`, so a class object is asked about its
    // class behaviour and not about `.Class`'s instances.
    let behaviour = match interp.receiver_behaviour(receiver) {
        Ok(behaviour) => behaviour,
        Err(kind) => return Err(Loud::receiver_class(kind).into()),
    };
    let classes = &interp.object_model().classes;
    let answers = match behaviour {
        Behaviour::Instance { methods, .. } => classes.has_method_at(methods, &name),
        Behaviour::ClassSide(class) => classes.class_has_method(class, &name),
    };
    Ok(Some(interp.counted(usize::from(answers))))
}

/// `Class~baseClass`: `RexxClass::getBaseClass`, bound as a method of
/// `.Class` by `Setup.cpp:455`.
///
/// A class declared `MIXINCLASS` answers its target's own base class and
/// every other class answers itself, which is the one thing about a class
/// that says whether it is a mixin -- measured, `::CLASS M MIXINCLASS
/// Object` and `::CLASS P` answer `The Object class` and `The P class`.
///
/// The non-class-object arm is unreachable, for the reason
/// [`class_receiver`] gives.
fn native_base_class(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    Ok(Some(interp.classes().base_class(class)))
}

/// The class object a receiver whose messages resolve against a class's
/// **class** behaviour is, or the refusal for one that is not a class object.
///
/// Every method in `.Class`'s own instance dictionary is in this position:
/// the only receiver whose behaviour that dictionary reaches is a class
/// object, and nothing this crate can build is an *instance* of `.Class`
/// without being one. Answering from the instance arm would mean picking a
/// class, so it refuses instead.
fn class_receiver(interp: &Interp, receiver: ObjRef) -> Result<ObjRef, Failure> {
    match interp.receiver_kind(receiver) {
        Ok(Primitive::Class(class)) => Ok(class),
        Ok(_) => Err(Loud::receiver_class("a value that is not a class object").into()),
        Err(kind) => Err(Loud::receiver_class(kind).into()),
    }
}

/// The argument a method that requires a class object was given --
/// `classArgument(other, TheClassClass, "class")`
/// (`runtime/MethodArguments.hpp:727`), which refuses an omitted argument
/// with 88.901 and a value that is not a class object with 88.914.
///
/// Measured at rc 168: `.Array~isSubclassOf()` reports `Missing argument;
/// argument class is required.` and `.Array~isA('abc')` reports `Argument
/// class must be an instance of the Class class.`
fn class_argument(args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("class").into());
    };
    match argument.class_id() {
        Some(_) => Ok(argument),
        None => Err(Raised::argument_not_a_class("class").into()),
    }
}

/// `Class~id`: the name the class was declared with, case unmodified --
/// `RexxClass::getId` (`classes/ClassClass.cpp:385`).
///
/// Measured, `.array~id` is `Array`, `::class "Foo"` makes `.Foo~id` `Foo`
/// and `::class Foo` makes it `FOO` -- an unquoted directive name is upcased
/// with every other symbol before the id is taken, and a quoted one is not.
/// `.object~subclass("Foo")~id` is `Foo`, because that id is a string
/// argument and no tokenizer sees it.
fn native_id(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let id = interp.class_id_text(class).as_bytes().to_vec();
    Ok(Some(interp.text_built(id)))
}

/// `Class~defaultName`: `The <id> class` -- `RexxClass::defaultNameRexx`
/// (`classes/ClassClass.cpp:614`).
///
/// **`~objectName=` does not move it**, which is what separates it from
/// [`native_object_name`]: measured, oracle rc 0, after `.K~objectName = 'zed'`
/// the class answers `zed` for `~objectName` and `~string` and `The K class`
/// for this.
fn native_class_default_name(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let name = interp.classes().default_name(class).as_bytes().to_vec();
    Ok(Some(interp.text_built(name)))
}

/// `Object~defaultName`: the receiver's class id with an article in front --
/// `RexxObject::defaultNameRexx` (`classes/ObjectClass.cpp:2868`) over
/// `RexxObject::defaultName` (`:1760`).
///
/// The receiver's own class and not the scope the method resolved at, so it
/// follows a subclass. Measured, oracle rc 0: `'abc'`, a small integer and
/// `1.5` all answer `a String`; `.nil` answers `an Object`; `.environment`
/// answers `a Directory`; and `.local~defaultName` is `a Directory` where its
/// `~objectName` is `The Local Directory`, so a stored name does not reach
/// this.
fn native_default_name(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(class) = native_class(interp, cleared, receiver, &[])? else {
        return Err(Loud::receiver_class("a value with no class of its own").into());
    };
    let name = crate::environment::default_object_name(interp.classes().id_string(class));
    Ok(Some(interp.text_built(name.into_bytes())))
}

/// `Class~metaClass`: `RexxClass::getMetaClass` (`classes/ClassClass.cpp:419`).
///
/// **Not `~class`**, and the pair parts iff the superclass is a metaclass and
/// is not the named-or-inherited metaclass -- `ClassRegistry::class_of` carries
/// the measured rows and the rule.
fn native_metaclass(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    Ok(Some(interp.classes().metaclass(class)))
}

/// `Class~superClass`: the class's own direct superclass, or `.nil` for
/// `.Object` -- `RexxClass::getSuperClass` (`classes/ClassClass.cpp:441`),
/// whose body is `superClasses->getFirstItem()`, so it reads the **first** entry
/// of the superclass list and not its last.
///
/// Measured, `.Array~superClass` is `The Object class` while
/// `.Array~superClasses` holds `The Object class` and `The OrderedCollection
/// class`, so the first entry and the whole list are different answers;
/// `.Object~superClass` is `The NIL object`.
fn native_superclass(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    Ok(Some(
        interp.classes().superclass(class).unwrap_or(ObjRef::NIL),
    ))
}

/// `Class~superClasses`: a **fresh** array of the class's own direct
/// superclasses -- `RexxClass::getSuperClasses`
/// (`classes/ClassClass.cpp:458`), whose body is `superClasses->copy()`, so it
/// hands out a copy rather than the class's own list.
///
/// Measured, `(.Array~superClasses == .Array~superClasses)` is `0`: two sends
/// answer two objects.
fn native_superclasses(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let items: Vec<Option<ObjRef>> = interp
        .classes()
        .superclasses(class)
        .iter()
        .map(|class| Some(*class))
        .collect();
    // Every item is a class identity, which lives outside the arena
    // (`rexx_core::CLASS_SLOT_BASE`), so the allocation below cannot collect
    // one of them out from under this array.
    Ok(Some(
        interp.alloc_with(BehaviourId::ARRAY, Body::array(items)),
    ))
}

/// `Object~class`: the class whose behaviour answers this receiver's
/// messages -- `RexxObject::classObject` (`classes/ObjectClass.cpp:1814`),
/// whose whole body is `behaviour->getOwningClass()`.
///
/// For a class object this is [`ClassRegistry::class_of`], the class the class
/// object's own behaviour belongs to, and **not** its metaclass; that
/// function's doc carries the measured rows where the two part. The C++ body
/// is what says which: the owning class is a field of the behaviour, and the
/// metaclass is a field of the class.
///
/// [`ClassRegistry::class_of`]: rexx_classes::ClassRegistry::class_of
fn native_class(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let kind = match interp.receiver_kind(receiver) {
        Ok(kind) => kind,
        Err(gap) => return Err(Loud::receiver_class(gap).into()),
    };
    let model = interp.object_model();
    Ok(Some(match kind {
        Primitive::String | Primitive::SmallInt => model.string,
        Primitive::Object => model.object,
        Primitive::Array => model.array,
        Primitive::Package => model.package,
        Primitive::Method => model.method,
        Primitive::Routine => model.routine,
        Primitive::Directory => model.directory,
        Primitive::StringTable(class) => class,
        Primitive::Context => model.context,
        Primitive::RexxInfo => model.rexx_info,
        Primitive::Message => model.message,
        Primitive::VariableReference => model.variable_reference,
        Primitive::Stem => model.stem,
        Primitive::Class(class) => model.classes.class_of(class),
        Primitive::Instance { class, .. } => class,
    }))
}

/// `Object~isA(class)`: whether the receiver's **class** is `class` or a
/// subclass of it -- `RexxObject::isInstanceOfRexx`
/// (`classes/ObjectClass.cpp:286`), which asks `classObject()` and not the
/// receiver.
///
/// The indirection through `~class` is the whole difference from
/// [`native_is_subclass_of`], and it is measurable on a class object as
/// receiver: `.Array~isA(.Class)` is `1` because `.Array~class` is `.Class`,
/// and `.Array~isA(.Array)` is `0` where `.Array~isSubclassOf(.Array)` is `1`.
fn native_is_a(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let other = class_argument(args)?;
    // `native_class` answers a class object for every receiver kind, so the
    // `None` a `NativeMethod` may now answer is not one of its outcomes -- and
    // a refusal rather than an `expect`, this crate's rule for an internal
    // inconsistency.
    let Some(own) = native_class(interp, cleared, receiver, &[])? else {
        return Err(Loud::receiver_class("a value with no class of its own").into());
    };
    let answer = interp.classes().is_a(own, other);
    Ok(Some(interp.counted(usize::from(answer))))
}

/// `Class~isSubclassOf(class)`: whether the receiver **is** `class` or derives
/// from it -- `RexxClass::isSubclassOf` (`classes/ClassClass.cpp:1692`),
/// which asks `isCompatibleWith` on the receiver itself.
fn native_is_subclass_of(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let other = class_argument(args)?;
    let class = class_receiver(interp, receiver)?;
    let answer = interp.classes().is_a(class, other);
    Ok(Some(interp.counted(usize::from(answer))))
}

/// `Object~identityHash`.
///
/// **Answers the handle**, which is deviation 4's licence read at this
/// message: identity in this crate is handle equality. The oracle's own
/// answer is derived from the object's address, so no differential row can
/// compare the two -- the corpus cannot witness this method and
/// `dispatch.rs`'s own tests are the instrument.
///
/// **The divergence is licensed rather than closed, and the oracle is why.**
/// `RexxObject::identityHash` is `((uintptr_t)this) ^ UINTPTR_MAX`
/// (`classes/ObjectClass.hpp:340`), rendered as a signed decimal. Measured
/// 2026-09-03, `say .Object~new~identityHash` over 10 runs answered 10
/// different values between `-139691328307761` and `-140519509881393`, so the
/// oracle does not reproduce its own answer across runs and there is nothing
/// to match; matching the width instead -- 16 characters in 20 of 20 runs --
/// would pin this machine's mmap address range into this crate.
fn native_identity_hash(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let bits = receiver.bits().to_string().into_bytes();
    Ok(Some(interp.text_built(bits)))
}

/// `RexxNilObject::getHashValue` (`classes/ObjectClass.cpp:2916`), whose
/// member is stamped with this sentinel rather than derived from anything.
const NIL_HASH: u64 = 0xdead_beef;

/// The string hash `RexxString::getStringHash` computes
/// (`classes/StringClass.hpp:328`), over the bytes rather than the text.
///
/// The accumulator is 64-bit and wraps, and **the byte is signed** -- `char`
/// on this platform -- which is the half a reimplementation gets wrong,
/// because it only shows above 0x7f. Measured: `'ff'x~hashCode` is eight `FF`
/// bytes, so the single byte contributed -1 rather than 255, and
/// `'80'x~hashCode` is `80FFFFFFFFFFFFFF`.
fn string_hash(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0;
    for byte in bytes {
        hash = hash
            .wrapping_mul(31)
            .wrapping_add(*byte as i8 as i64 as u64);
    }
    hash
}

/// `Object~hashCode`, `RexxObject::hashCode` (`classes/ObjectClass.cpp:398`):
/// `getHashValue()` rendered as its own eight bytes, little-endian.
///
/// `getHashValue` is virtual and overridden by `NilObject`, `String`,
/// `Pointer`, `Integer`, `NumberString` and `Class`; every other receiver
/// takes `identityHash()`. **The rule is the receiver's kind, not whether it
/// renders as text**: measured, two `.MutableBuffer~new('abc')` hash
/// differently from each other and from `'abc'`, and both move between the
/// oracle's own runs.
///
/// `Integer` and `NumberString` delegate to their string value's hash
/// (`IntegerClass.cpp:83`, `NumberStringClass.cpp:129`) and answer `String`
/// for their class, so [`Primitive::String`] covers all three: measured,
/// `5~hashCode` and `'5'~hashCode` are both `3500000000000000`, and
/// `(2**40)~hashCode` equals `'1.09951163E+12'~hashCode` -- the hash is of
/// the number as it renders, not of its digits.
///
/// The identity arm answers the handle, which is deviation 4's licence read
/// exactly as [`native_identity_hash`] reads it: the oracle's own value is
/// the complement of an address and does not reproduce across its own runs,
/// so no differential row can compare the two.
fn native_hash_code(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let value = hash_value(interp, receiver);
    Ok(Some(interp.text_built(value.to_le_bytes().to_vec())))
}

/// `RexxInternalObject::getHashValue()` as a number, which is what
/// [`native_hash_code`] renders and what the mapped collections' store hashes
/// an index by. Split out for the second caller, not changed.
pub(super) fn hash_value(interp: &mut Interp, receiver: ObjRef) -> u64 {
    if receiver == ObjRef::NIL {
        return NIL_HASH;
    }
    match interp.receiver_kind(receiver) {
        Ok(Primitive::String | Primitive::SmallInt) => string_hash(&interp.to_text(receiver)),
        Ok(Primitive::Class(class)) => string_hash(interp.classes().id_string(class).as_bytes()),
        _ => receiver.bits(),
    }
}

/// `Class~annotation(name)`, and the same method at `Method`, `Routine` and
/// `Package`: the annotation `name` holds, or `.nil`.
///
/// `RexxClass::getAnnotationRexx` (`classes/ClassClass.cpp:374`) and its two
/// siblings are each `resultOrNil(getAnnotation(stringArgument(name,
/// "name")))`, and every `getAnnotation` reads
/// `annotations->entry(name)`, which **upcases the index it is given**
/// (`StringHashCollection::entry`, `classes/support/HashCollection.cpp:824`).
/// Measured, oracle rc 0 under `::ANNOTATE CLASS K author 'moritz'`:
/// `.K~annotation("AUTHOR")` and `.K~annotation("author")` both answer
/// `moritz`, and `.K~annotation("ZZ")` answers `The NIL object`.
///
/// The argument is a required string named `name` in the message the oracle
/// reports: measured, `.K~annotation()` is 88.901 `Missing argument;
/// argument name is required.` and `.K~annotation(.array)` is 88.909
/// `Argument name must have a string value.`
fn native_annotation(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("name").into());
    };
    let argument = required_string_named_argument(interp, argument, "name")?;
    let name = interp.to_text(argument).to_ascii_uppercase();
    let table = interp.annotations_of(receiver)?;
    Ok(Some(
        interp.native_entry(table, &name).unwrap_or(ObjRef::NIL),
    ))
}

/// `Class~annotations`, and the same method at `Method`, `Routine` and
/// `Package`: the receiver's own annotation table.
///
/// `RexxClass::getAnnotations` (`classes/ClassClass.cpp:325`) creates an
/// empty `StringTable` on the first ask and stores it, so a target no
/// `::ANNOTATE` named still answers a table and a program can add to it --
/// [`Interp::annotation_table`] carries the measurement.
fn native_annotations(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.annotations_of(receiver)?))
}

/// `String~sign`: `RexxString::sign`, which is
/// `ArithmeticMethod(Sign(), "SIGN")` (`classes/StringClass.cpp:1084`) and so
/// is the `SIGN` builtin's own computation on the receiver.
///
/// **Landed here because `DateTime~compareTo` reaches it**:
/// `(utcTimeStamp - othertime)~sign` (`CoreClasses.orx:2517`) is the last step
/// of every ordering comparison on a `DateTime`, so that class's documented
/// comparison operators all refused on this one name.
///
/// Measured, oracle: `(-12)~sign` is `-1`, `'-0.0'~sign` is `0`, and
/// `'abc'~sign` is `93.943 SIGN method target must be a number; found "abc".`
/// at rc 163 under a `Compiled method "SIGN" with scope "String".` line.
fn native_string_sign(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Ok(value) = interp.to_number(receiver) else {
        let found = interp.string_value_text(receiver);
        return Err(Raised::method_target_not_a_number(b"SIGN", &found).into());
    };
    Ok(Some(crate::builtin::numeric::sign_of(interp, &value)))
}

/// The one operand every `Object` operator method requires, or 93.903.
///
/// `requiredArgument(other, ARG_ONE)` opens each of them
/// (`classes/ObjectClass.cpp:452`), and the count in `Setup.cpp:521`-`:530`
/// is 1 -- so a shorter list is padded and refused here rather than by
/// [`Interp::invoke`]. Measured, oracle rc 163: `o~'='()` is `93.903 Missing
/// argument in method; argument 1 is required.` under a `Compiled method "="
/// with scope "Object".` line.
fn operator_argument(args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    match args.first().copied() {
        Some(Some(argument)) => Ok(argument),
        _ => Err(Raised::missing_method_argument(1).into()),
    }
}

/// `Object~"="` and `Object~"=="`: `RexxObject::equal` and
/// `RexxObject::strictEqual` (`classes/ObjectClass.cpp:464`, `:448`), each a
/// direct identity test rather than a comparison of renderings.
///
/// Measured, oracle rc 0 on `::CLASS K`: `.K~new = .K~new` is `0`, `o = o` is
/// `1`, and `o = 'a K'` is `0` against the very text the instance renders as.
///
/// Identity here is handle equality, which is deviation 4's licence read the
/// same way [`native_identity_hash`] reads it.
fn native_object_identical(
    _interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let other = operator_argument(args)?;
    Ok(Some(crate::eval::logical(receiver == other)))
}

/// `Object~"\="`, `"\=="`, `"<>"` and `"><"`: `RexxObject::notEqual` and
/// `RexxObject::strictNotEqual` (`classes/ObjectClass.cpp:492`, `:478`),
/// [`native_object_identical`]'s negation.
fn native_object_different(
    _interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let other = operator_argument(args)?;
    Ok(Some(crate::eval::logical(receiver != other)))
}

/// `Object~"||"` and `Object~""`: `RexxObject::concatRexx`
/// (`classes/ObjectClass.cpp:2807`), which is `requestString()` on the
/// receiver and then that string's own concatenation.
///
/// Measured, oracle rc 0: an instance whose class defines `::METHOD string`
/// returning `'STR'` gives `STRx` for `o || 'x'`, and one that defines
/// `makeString` gives `MKSx`, where a class defining neither gives its
/// default name.
fn native_object_concat(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    concat_through_string_value(interp, Operator::Concatenate, receiver, args)
}

/// `Object~" "`: `RexxObject::concatBlank` (`classes/ObjectClass.cpp:2823`),
/// [`native_object_concat`] with the one separating space.
fn native_object_concat_blank(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    concat_through_string_value(interp, Operator::Blank, receiver, args)
}

/// The shared body of the two above: the receiver's string value, then
/// [`Interp::apply_binary`] on it, which is the C++'s
/// `alias->concatRexx(otherObj)` and converts the other operand exactly as
/// every other concatenation does.
///
/// The alias is rooted because the join allocates and the receiver's own
/// `STRING` method may have built it.
fn concat_through_string_value(
    interp: &mut Interp,
    op: Operator,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let other = operator_argument(args)?;
    let frame = interp.roots.push_frame();
    let joined = interp
        .required_string_value(receiver)
        .and_then(|alias| {
            interp.roots.push_temp(alias);
            interp.apply_binary(op, alias, other)
        })
        .map(Some);
    interp.roots.pop_frame(frame);
    joined
}

/// `Class~method(name)`: the method object `name` names **in this class's own
/// instance dictionary**, and 97.1 for anything else.
///
/// `RexxClass::method` (`classes/ClassClass.cpp:984`) retrieves from
/// `instanceMethodDictionary` directly, so an inherited name, a donated name
/// and a class-side name all raise. Measured, three descriptors:
/// `.Array~method("APPEND")` answers `a Method`; `.Array~method("STRING")`,
/// whose name `.Object` defines and `.Array`'s flattened behaviour holds, is
/// 97.1 at rc 159; and `.K~method("M")` for `::method m class` is 97.1 too,
/// while the same directive without `CLASS` answers.
///
/// **A hidden name answers `.nil` rather than raising**, which is the one
/// reader that tells hiding and removal apart -- the C++ says so in its own
/// comment at the raise (`:992`-`:993`: "Note that is could be there, but as
/// .nil.  We will return that value"). Measured at rc 0,
/// `.Stem~method("==")` and `.VariableReference~method("==")` both print
/// `The NIL object` where `.Queue~method("SORT")` raises 97.1.
fn native_method(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("method name").into());
    };
    let argument = required_string_named_argument(interp, argument, "method name")?;
    let name = interp.to_text(argument).to_ascii_uppercase();
    let class = class_receiver(interp, receiver)?;
    match interp
        .classes()
        .own_instance_slot(class, &String::from_utf8_lossy(&name))
    {
        None => {
            let target = interp.class_default_name(class).to_vec();
            Err(Raised::no_method(&target, &name).into())
        }
        Some(MethodSlot::Hidden) => Ok(Some(ObjRef::NIL)),
        Some(MethodSlot::Defined { scope, .. }) => {
            Ok(Some(interp.method_object(class, &name, scope)))
        }
    }
}

/// `Method~scope`: the class the method object was defined at, `.nil` for a
/// method object no class has taken -- [`Interp::method_scope`] carries the
/// C++ and the measurements.
///
/// **The scope is not the class the send went to**, which is the whole point
/// of the answer: measured, oracle rc 0, with `::method m` under a
/// `::class base` and an overriding `::method m` under `::class sub subclass
/// base`, `.base~method("M")~scope~id` is `BASE` and `.sub~method("M")~scope~id`
/// is `SUB`, so a reader that walked to the ancestor defining the name would
/// answer `BASE` twice. `corpus/gate-tables/concepts/xscope.rex` is that pair.
fn native_scope(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.method_scope(receiver)?))
}

/// The `REXX_DEFINED` refusal every class mutator opens with --
/// `isRexxDefined()` and its `reportException(Error_Execution_rexx_defined_class)`
/// (`classes/ClassClass.cpp:823`, `:522`, `:955`, `:1290`, `:1382`, one per
/// mutator).
///
/// **Checked before the arguments are**, which the five methods all do and
/// which is measured: `.Array~inherit()` reports 98.985 where the same send
/// to a class a `::CLASS` declared reports 88.901.
///
/// **The corpus is what catches a regression in this check**, and its rows
/// have to name a class from each half of what carries the flag: the classes
/// `Setup.cpp` builds and the classes the interpreter's own Rexx-written
/// library declares. A build that dropped the check would let
/// `.Array~define(...)` succeed at rc 0 where the oracle raises -- a
/// divergence a differential row sees, unlike a refusal the oracle does not
/// share.
///
/// Whether the flag is *set* on the library's half is a different question,
/// and a corpus row can only ask it of a class a program can name.
/// `every_class_the_library_declares_carries_the_rexx_defined_flag` asks it
/// over the whole table the bootstrap leaves behind, which is where the
/// classes declared without `PUBLIC` are.
fn rexx_defined_lock(interp: &mut Interp, class: ObjRef) -> Result<(), Failure> {
    // **Open while the interpreter's own library runs**, which is the state
    // `Setup.cpp` builds the image in: `CoreClasses.orx:93` onwards is a run
    // of `~inherit` clauses against exactly the classes this flag guards,
    // and the C++ sets `REXX_DEFINED` on them at image-save time
    // (`RexxClass::liveGeneral`, `ClassClass.cpp:136`-`:142`) rather than
    // before. This crate flags each library class as `Interp::install_class`
    // creates it, so the bypass is not a convenience: it is what lets the
    // prologue's own `~inherit` clauses run at all. See
    // `Interp::library_bootstrap` for what else the same flag opens and for
    // why no program can be inside it.
    if !interp.library_bootstrap && interp.classes().is_rexx_defined(class) {
        return Err(Raised::rexx_defined_class().into());
    }
    Ok(())
}

/// The physical lines a method source is compiled from, or the refusal for
/// a value this crate cannot read as one -- `processExecutableSource`
/// (`execution/BaseExecutable.cpp:169`).
///
/// **A string is one line and not a text to split.** The C++ wraps it in a
/// one-element array (`:174`-`:177`) and `ArrayProgramSource` gives one line
/// per element, so a terminator byte inside it is a character in the program:
/// measured, oracle rc 243, `.k~define("m", 'say 1' || '0a'x || 'say 2')` is
/// `Error 13.1: Incorrect character in program "\n" ('0A'X).` Joining the
/// lines and letting the scanner find the boundaries would compile that at
/// rc 0, which is why [`rexx_parse::parse_lines`] takes the elements rather
/// than a buffer.
///
/// **The walk ends at the last item**, which is `stringArrayArgument`'s own
/// `1..=lastIndex()` -- see [`Raised::method_source_not_all_strings`] for the
/// pair of measurements that separates a hole from a longer array.
///
/// [`Raised::method_source_not_all_strings`]: crate::Raised::method_source_not_all_strings
fn method_source_lines(
    interp: &mut Interp,
    source: ObjRef,
    position: &'static str,
) -> Result<Vec<Vec<u8>>, Failure> {
    if let Ok(Primitive::Array) = interp.receiver_kind(source) {
        let slots = interp.array_slots_of(source).unwrap_or_default();
        let last_item = slots
            .iter()
            .rposition(Option::is_some)
            .map_or(0, |at| at + 1);
        let mut lines = Vec::with_capacity(last_item);
        for slot in &slots[..last_item] {
            let item = slot.filter(|item| is_source_line(interp, *item));
            let Some(item) = item else {
                return Err(Raised::method_source_not_all_strings(position).into());
            };
            lines.push(interp.to_text(item).to_vec());
        }
        return Ok(lines);
    }
    if is_source_line(interp, source) {
        return Ok(vec![interp.to_text(source).to_vec()]);
    }
    Err(Loud::method_from_source("a method source that is neither a string nor an array").into())
}

/// Whether one value is a source line -- the C++'s `isString(source)` for
/// the whole source and its `makeString()` for an array's items
/// (`execution/BaseExecutable.cpp:174`, `classes/StringClassUtil.cpp:428`).
///
/// [`Primitive::SmallInt`] answers with [`Primitive::String`] because the
/// oracle makes no distinction to answer differently: measured,
/// `12345~class~id` is `String`, and `.k~define("m", 5)` is rc 0.
fn is_source_line(interp: &Interp, value: ObjRef) -> bool {
    matches!(
        interp.receiver_kind(value),
        Ok(Primitive::String | Primitive::SmallInt)
    )
}

/// `MethodClass::newMethodObject`'s compiling arm
/// (`classes/MethodClass.cpp:462`-`:485`): the `Method` object a source text
/// becomes, carrying no scope, for a caller that is about to install it.
///
/// **The body is filed under the object rather than under a dictionary
/// key**, which is [`Interp::record_compiled_body`]'s job: `~define` and
/// `~defineMethods` mint their own identity for what they install, and
/// `SETMETHOD` is what installs where a send can reach the body this crate
/// holds.
///
/// The parse also buys the oracle's **timing**: a source that does not parse
/// fails at `~define` time and not at send time, measured at rc 221 for a
/// body no send ever reaches.
///
/// `name` is the method's own name as the caller wrote it, which the oracle
/// keeps unchanged where the dictionary key is upcased
/// (`classes/ClassClass.cpp:830`-`:832`): it is the name a parse failure
/// reports the source under.
fn compile_method_source(
    interp: &mut Interp,
    name: &[u8],
    source: ObjRef,
    position: &'static str,
) -> Result<ObjRef, Failure> {
    let lines = method_source_lines(interp, source, position)?;
    let borrowed: Vec<&[u8]> = lines.iter().map(Vec::as_slice).collect();
    let parsed = rexx_parse::parse_lines(&borrowed).map_err(|error| {
        Failure::from(Loud::method_from_source(&format!(
            "reporting a method source that does not parse ({}, {error})",
            String::from_utf8_lossy(name)
        )))
    })?;
    if !parsed.directives.is_empty() {
        return Err(Loud::method_from_source("a method source that carries a directive").into());
    }
    let method_class = interp.method_class();
    let object = interp.native_instance(method_class);
    // Every `Method` object this crate builds carries an annotation table, so
    // that `~annotations` and `~annotation` answer from the object rather
    // than from a way back to a directive. A compiled method has no directive
    // and its table starts empty, which is the oracle's own answer: measured,
    // rc 0, `.k~define("m", 'return 1')` then `.k~method("M")~annotation('x')`
    // is `The NIL object`.
    let site = crate::environment::Annotated::Compiled(interp.compiled_methods);
    interp.compiled_methods += 1;
    interp.attach_annotations(object, site);
    interp.record_compiled_body(object, name, parsed);
    Ok(object)
}

/// The `method name` argument `~define`, `~delete` and `~method` share:
/// required, string-valued, and upcased before it reaches a dictionary --
/// `stringArgument(method_name, "method name")->upper()`
/// (`classes/ClassClass.cpp:831`-`:832`, `:961`, `:987`).
fn method_name_argument(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<Vec<u8>, Failure> {
    Ok(method_name_pair(interp, args)?.1)
}

/// [`method_name_argument`] with the spelling the caller wrote kept beside
/// the dictionary key.
///
/// `~define` needs both at once: the key is `method_name->upper()` and the
/// method object is built under `method_name` itself
/// (`classes/ClassClass.cpp:830`-`:832`, `:849`). What the oracle does with
/// the second is report a parse failure under it -- measured, rc 221,
/// `.k~define("bad", 'this is not rexx +++')` reports `Error 35 running bad
/// line 1:`, the name as written -- and that report is
/// [`compile_method_source`]'s refusal here, which names the method the same
/// way. Nothing else in this crate can see the difference: a dictionary key
/// is upcased again by `MethodDict` on insert and on lookup, so the two
/// spellings reach the same entry.
fn method_name_pair(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, Vec<u8>), Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("method name").into());
    };
    let argument = required_string_named_argument(interp, argument, "method name")?;
    let written = interp.to_text(argument).to_vec();
    let key = written.to_ascii_uppercase();
    Ok((written, key))
}

/// `Class~define(name, method)`: install one instance method on the receiver
/// -- `RexxClass::defineMethod` (`classes/ClassClass.cpp:819`).
///
/// **The second argument has three shapes and they are three answers**,
/// measured on the oracle for a `::class K` and read back through `~method`:
///
/// * a `Method` object -- installed, and `~method` answers **that** object;
/// * omitted -- a `.nil` tombstone, and `~method` answers `The NIL object`.
///   `.K~define("STRING")` does this for a name `.Object` supplies, so the
///   entry is created rather than overwritten;
/// * `.nil` -- the entry goes away, and `~method` raises 97.1. This is not
///   the tombstone: measured, `.K~define("Z", .methods~z)` followed by
///   `.K~define("Z", .nil)` leaves `.K~method("Z")` raising where the
///   omitted form leaves it answering `The NIL object`. The C++ leaves
///   `methodObject` at `OREF_NULL` for this arm alone (`:840`-`:849`, whose
///   two tests are `OREF_NULL == methodSource` and
///   `TheNilObject != methodSource`) and hands that to `replaceMethod`.
///   Modelled as the removal it reads as; the flattened behaviour is where a
///   stored null and an absent entry could still part, and no send this
///   phase can make reaches one, since `~new` is not built.
///
/// Anything else is source text for `newMethodObject` to compile, which is
/// [`compile_method_source`]. The compiled object carries no scope, so the
/// `newScope` inside [`Interp::define_method_object`] fills in this class and
/// keeps the object rather than copying it: measured, oracle rc 0,
/// `.cost~define("upper", 'return "U"')` then `.cost~method("UPPER")~scope~id`
/// is `COST`, while an unattached `::METHOD` reached through `.methods~z`
/// answers `The NIL object` until a class takes it.
fn native_define(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    rexx_defined_lock(interp, class)?;
    let (written, name) = method_name_pair(interp, args)?;
    match args.get(1).copied().flatten() {
        None => {
            interp
                .classes()
                .hide_instance_method(class, &String::from_utf8_lossy(&name));
            interp.drop_method_object(class, &name);
        }
        Some(source) if source == ObjRef::NIL => {
            interp
                .classes()
                .delete_instance_method(class, &String::from_utf8_lossy(&name));
            interp.drop_method_object(class, &name);
        }
        Some(source) => {
            let source = if interp.receiver_kind(source) == Ok(Primitive::Method) {
                source
            } else {
                compile_method_source(interp, &written, source, "method")?
            };
            interp
                .define_method_object(class, &name, source)
                .ok_or_else(|| {
                    Failure::from(Loud::receiver_class(
                        "a method object this crate did not build",
                    ))
                })?;
        }
    }
    Ok(None)
}

/// `Class~defineMethods(methods)`: install a whole table of instance methods
/// in one mutation -- `RexxClass::defineMethodsRexx`
/// (`classes/ClassClass.cpp:518`).
///
/// **The object stored is never the one the table held**, unlike `~define`
/// beside it -- see [`Interp::define_method_table`] for the two `newScope`
/// calls that make it so and the measurement.
///
/// The oracle reads the argument by sending it `SUPPLIER`
/// (`classes/ClassClass.cpp:1250`), so what a value that has no such method
/// gets is 97.1 naming that message, and this raises the same: measured,
/// `.K~defineMethods("abc")` is rc 159, `Object "abc" does not understand
/// message "SUPPLIER".` under the `DEFINEMETHODS` frame. A value whose
/// behaviour *does* answer `SUPPLIER` and which this phase cannot walk is
/// the refusal that send would have produced instead.
///
/// The two hash collections are read from their own entries rather than
/// through a supplier object, which this phase does not build. They are
/// walked in sorted name order: the order is not observable, and a stable
/// one is what keeps the run-to-run allocation sequence fixed.
fn native_define_methods(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    rexx_defined_lock(interp, class)?;
    let Some(Some(table)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("methods").into());
    };
    if !string_keyed_table(interp, table) {
        return Err(supplier_refusal(interp, table));
    }
    // **Asked before the walk**, because the walk cannot see the difference:
    // a collection whose entries this crate answers per name rather than
    // storing looks empty to `native_keys`, and a mutation over an empty walk
    // is a mutation that silently does nothing.
    if let Some(owner) = interp.unbuilt_collection_owner(table) {
        return Err(Loud::unreadable_collection(owner).into());
    }
    let mut names = interp.native_keys(table);
    names.sort();
    let mut entries: Vec<(Box<[u8]>, Option<ObjRef>)> = Vec::with_capacity(names.len());
    for name in names {
        let value = interp.native_entry(table, &name).unwrap_or(ObjRef::NIL);
        if value == ObjRef::NIL {
            entries.push((name, None));
            continue;
        }
        let value = if interp.receiver_kind(value) == Ok(Primitive::Method) {
            value
        } else {
            // The entry's own index is the method's name, as stored:
            // `createMethodDictionary` builds the object under
            // `supplier->index()->requestString()` and keys the dictionary
            // under its upcase (`classes/ClassClass.cpp:1255`-`:1258`).
            compile_method_source(interp, &name, value, "method source")?
        };
        entries.push((name, Some(value)));
    }
    interp.define_method_table(class, &entries).ok_or_else(|| {
        Failure::from(Loud::receiver_class(
            "a method object this crate did not build",
        ))
    })?;
    Ok(None)
}

/// Whether `value` is a table `~defineMethods`, `~enhanced` and `~setMethod`
/// can walk: one of this crate's own `Body::Native` directories, or a
/// `Directory` or `StringTable` a program made, which since Phase 5h Task 4
/// is a collection with a hash store.
///
/// **Both, and the second is the one that was missed.** Measured when this
/// asked only `receiver_kind`: `enhanced_scope.rex`, `enhanced_unset.rex` and
/// `usesem.rex` -- all three of which hand `Class~enhanced` a
/// `.StringTable~new` they filled -- fell through to the `SUPPLIER` refusal.
fn string_keyed_table(interp: &mut Interp, value: ObjRef) -> bool {
    if matches!(
        interp.receiver_kind(value),
        Ok(Primitive::Directory | Primitive::StringTable(_))
    ) {
        return true;
    }
    hash::owns(interp, value) && hash::string_keyed(interp, value)
}

/// What a `SUPPLIER` send to `table` would have answered, as the failure
/// `~defineMethods` reports for a value it cannot walk.
///
/// Asking the same lookup a send asks, rather than deciding from the value's
/// kind: a receiver whose behaviour has no `SUPPLIER` entry is the oracle's
/// own 97.1, and one that has an entry this crate implements no code for is
/// this crate's gap. Nothing is run either way -- no value this phase builds
/// answers `SUPPLIER` with a supplier object.
fn supplier_refusal(interp: &mut Interp, table: ObjRef) -> Failure {
    match interp.lookup_for_refusal(table, b"SUPPLIER") {
        Some(scope) => Loud::native_method(b"SUPPLIER", &scope).into(),
        None => {
            let target = interp.string_value_text(table);
            Raised::no_method(&target, b"SUPPLIER").into()
        }
    }
}

/// `Class~subclass(id, metaclass, classMethods)`: build a class derived from
/// the receiver and answer it -- `RexxClass::subclassRexx`
/// (`classes/ClassClass.cpp:1543`), which forwards to the same
/// `RexxClass::subclass` a `::CLASS` directive calls, with `OREF_NULL` where
/// the directive passes its package (`:1546`).
fn native_subclass(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    class_factory(interp, receiver, args, ClassKind::Regular)
}

/// `Class~mixinClass(id, metaclass, classMethods)`: the same factory, marking
/// what it builds a mixin -- `RexxClass::mixinClassRexx`
/// (`classes/ClassClass.cpp:1493`).
///
/// **`RexxClass::mixinClass` is `subclass` under the mixin flag** (`:1519`)
/// **and the base class taken from the receiver's own** (`:1522`) rather than
/// from the class being built. [`rexx_classes::ClassGraph::define_class`]
/// makes both from [`ClassKind::Mixin`], which is why one factory serves both
/// messages. Measured, oracle rc 0: `.array~mixinclass("mx")~baseClass`
/// is `The Array class` where `.array~subclass("s")~baseClass` is `The s
/// class`.
fn native_mixin_class_factory(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    class_factory(interp, receiver, args, ClassKind::Mixin)
}

/// `Object~new`: what every class that declares no `NEW` of its own answers.
///
/// `RexxObject::newRexx` (`classes/ObjectClass.cpp:2630`) allocates a plain
/// object and hands it to `RexxClass::completeNewObject`
/// (`classes/ClassClass.cpp:1882`), whose steps run in a fixed order:
/// `checkAbstract`, the behaviour, the `UNINIT` registration, the `INIT` send.
///
/// **The order is observable.** The abstract check precedes `INIT`, so an
/// abstract class whose `INIT` prints never prints; and `~new`'s frame is
/// still on the traceback while `INIT` runs -- measured, oracle rc 163,
/// `.Object~new(1)` reports `Compiled method "INIT" with scope "Object".`
/// above `Compiled method "NEW" with scope "Object".`
///
/// **`~new`'s arguments are `INIT`'s, and nothing chains them.** Measured,
/// oracle rc 0: a subclass whose `INIT` omits `self~init:super` leaves the
/// superclass's `INIT` unrun and its exposed variable reading its own name.
fn native_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let object = new_instance(interp, class)?;
    let caller = interp.caller();
    interp.send_message(object, INIT, None, args, caller)?;
    Ok(Some(object))
}

/// `RexxClass::completeNewObject` (`classes/ClassClass.cpp:1882`) up to but
/// not including the `INIT` send: the abstract check, the behaviour, the
/// rooting and the `UNINIT` registration, in that order.
///
/// Split from [`native_new`] because [`native_enhanced`] takes the same
/// steps and then puts the enhancing methods in before `INIT` runs, which is
/// what makes an enhancing `INIT` the one that runs.
fn new_instance(interp: &mut Interp, class: ObjRef) -> Result<ObjRef, Failure> {
    if interp.classes().is_abstract(class) {
        let id = interp.classes().id_string(class).as_bytes().to_vec();
        return Err(Raised::abstract_class(&id).into());
    }
    // The object's behaviour is set from the class here, once. What the class
    // holds later is a different behaviour or the same one rebuilt, and D58
    // turns on which.
    let behaviour = interp.classes().instance_behaviour_handle(class);
    let object = interp.alloc_with(
        rexx_core::BehaviourId::OBJECT,
        Body::Instance {
            class,
            behaviour,
            name: None,
            pools: rexx_core::ScopePools::new(),
            own: None,
            native: None,
        },
    );
    // `ProtectedObject p(newObj)` (`ObjectClass.cpp:2637`): the `INIT` send
    // allocates, and nothing else holds the object until it returns.
    interp.roots.push_temp(object);
    // See `Interp::reqstr_armed` for why an instance arms the protocol
    // outright rather than by the name a directive installed.
    interp.reqstr_armed = true;
    if interp.classes().has_uninit(class) {
        interp.heap.set_uninit(object);
    }
    Ok(object)
}

/// `RexxClass::subclass` (`classes/ClassClass.cpp:1562`), in its own order.
///
/// **The order is observable and each boundary is measured**, oracle, stdout
/// empty: the metaclass is resolved and tested first, so
/// `.object~subclass(, .Object)` is 99.927 and not the 88.901 its omitted id
/// would earn, while `.object~subclass(, .Class)` is that 88.901. The
/// enhancing methods are merged before the `INIT` send, so an enhancing
/// `INIT` is the one that runs -- measured, `::METHOD init` in `.methods`
/// prints from inside `.object~subclass("k", .Class, .methods)`.
///
/// **Nothing here consults `REXX_DEFINED` and nothing sets it.** The C++ is
/// the reason on both halves: no mutator [`rexx_defined_lock`] guards has its
/// `isRexxDefined()` test inside this function, and the flag is written by
/// `RexxClass::liveGeneral` at image-save time (`:136`-`:142`) rather than by
/// any constructor. Measured, oracle rc 0: `k = .object~subclass("k")` then
/// `k~inherit(.object~mixinclass("mx"))` answers `The Object class The mx
/// class`, where `.array~inherit(.object)` is 98.985.
///
/// The id string is `String::from_utf8_lossy`'d for the reason
/// [`Interp::install_class`] gives about a `::CLASS` name that is not UTF-8.
fn class_factory(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    kind: ClassKind,
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let metaclass = factory_metaclass(interp, class, args)?;
    let name = class_id_argument(interp, args)?;
    let id = interp.classes().define_unregistered_class(
        &String::from_utf8_lossy(&name),
        Some(class),
        kind,
        metaclass,
    );
    interp.record_packageless_class(id);
    if let Some(enhancing) = args.get(2).copied().flatten() {
        enhance_class_methods(interp, id, enhancing)?;
    }
    // `RexxClass::subclass`'s own tail, in its order (`:1615`-`:1637`), which
    // is the same sequence `Interp::install_class_at` makes for a directive.
    interp.classes().check_uninit(id);
    let caller = interp.caller();
    interp.send_message(id, INIT, None, &[], caller)?;
    interp.classes().refresh_parent_has_uninit(id);
    Ok(Some(id))
}

/// The metaclass a class factory builds from: the second argument, or the
/// receiver's own where the send omits it (`classes/ClassClass.cpp:1566`-
/// `:1569`).
///
/// **`.nil` is not an omission**, measured: `.object~subclass("k", .nil)` is
/// 99.927 naming `The NIL object`, where `.object~subclass("k")` builds. The
/// C++ tests `meta_class == OREF_NULL`, which an omitted argument is and a
/// supplied `.nil` is not.
///
/// **The test is `!isInstanceOf(TheClassClass) || !isMetaClass()`** (`:1572`),
/// so a value that is not a class object gets the same 99.927 with its own
/// rendering: measured, `.object~subclass("k", "abc")` reports `"abc" is not
/// a valid metaclass.`
fn factory_metaclass(
    interp: &mut Interp,
    class: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let metaclass = match args.get(1).copied().flatten() {
        None => interp.classes().metaclass(class),
        Some(named) => named,
    };
    let is_class = matches!(interp.receiver_kind(metaclass), Ok(Primitive::Class(_)));
    if !is_class || !interp.classes().is_metaclass(metaclass) {
        let shown = interp.string_value_text(metaclass);
        return Err(Raised::bad_metaclass(&shown).into());
    }
    // **The class object itself is built by a `NEW` send to the metaclass**
    // (`:1579`), so a metaclass carrying its own `NEW` decides what gets
    // built and what its arguments mean. This crate implements no `NEW` at
    // all; what it models is the one `.Class` declares, by constructing the
    // class in `class_factory` instead of sending. A resolution landing
    // anywhere else is a class this crate cannot build, and it refuses rather
    // than build the one `.Class` would have -- measured, oracle rc 0:
    // `::CLASS MyMeta SUBCLASS Class` with `::METHOD new CLASS` runs that
    // body for `.object~subclass("k", .MyMeta)`.
    let modelled = interp.object_model().metaclass;
    let resolved = interp
        .classes()
        .lookup_class_method(metaclass, "NEW")
        .map(|(scope, _)| scope);
    if resolved != Some(modelled) {
        let scope = interp.classes().id_string(metaclass).to_string();
        return Err(Loud::native_method(b"NEW", &scope).into());
    }
    Ok(metaclass)
}

/// The `class id` argument, and the traceback frame the oracle's own `NEW`
/// activation contributes when it refuses.
///
/// `RexxClass::newRexx` is where the checks live
/// (`classes/ClassClass.cpp:1786`, `stringArgument(class_id, "class id")`),
/// because `subclass` reaches it by
/// `meta_class->sendMessage(GlobalNames::NEW, class_id, p)` (`:1579`) -- so
/// the frame is
/// owed for the same reason [`Interp::blame_request`]'s is, and the method's
/// own frame goes above it from [`Interp::invoke`]. Measured, oracle rc 168:
/// `.object~subclass()` reports `Compiled method "NEW" with scope "Class".`
/// then `Compiled method "SUBCLASS" with scope "Class".` then the sending
/// clause, and `88.901 Missing argument; argument class id is required.`;
/// `.object~subclass(.environment)` is `88.909 Argument class id must have a
/// string value.` under the same pair.
fn class_id_argument(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<Vec<u8>, Failure> {
    let outcome = required_class_id(interp, args);
    if outcome.is_err() {
        interp.blame_native_method(b"NEW", "Class");
    }
    outcome
}

/// [`class_id_argument`] without the frame, so that every way of failing
/// takes it.
///
/// **The id keeps the spelling it was given**, unlike a method name, which
/// [`method_name_argument`] upcases: measured, `.object~subclass("k")~id` is
/// `k` and `~string` is `The k class`.
fn required_class_id(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<Vec<u8>, Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("class id").into());
    };
    let argument = required_string_named_argument(interp, argument, "class id")?;
    Ok(interp.to_text(argument).to_vec())
}

/// The third argument: class-side methods the new class is built with --
/// `createMethodDictionary(enhancing_methods, new_class)` merged into
/// `classMethodDictionary` (`classes/ClassClass.cpp:1602`-`:1608`).
///
/// **The class side and not the instance side**, measured, oracle rc 0: with
/// `::METHOD z` unattached, `k = .object~subclass("k", .Class, .methods)`
/// answers `k~z` and `k~hasMethod('Z')` is `1`, while `k~method('Z')` raises
/// 97.1 -- `~method` reads the instance dictionary and nothing was put there.
///
/// The argument is read the way [`native_define_methods`] reads its own, for
/// the reason that function gives: the oracle walks it by sending `SUPPLIER`,
/// so what a value with no such method gets is 97.1 naming that message.
fn enhance_class_methods(
    interp: &mut Interp,
    class: ObjRef,
    enhancing: ObjRef,
) -> Result<(), Failure> {
    if !string_keyed_table(interp, enhancing) {
        return Err(supplier_refusal(interp, enhancing));
    }
    if let Some(owner) = interp.unbuilt_collection_owner(enhancing) {
        return Err(Loud::unreadable_collection(owner).into());
    }
    let mut names = interp.native_keys(enhancing);
    names.sort();
    // One frame around the whole walk, for the reason
    // [`Interp::define_method_table`] has one: each entry allocates the copy
    // `newScope` makes, and the temporary rooting that copy carries until
    // `hold_method_object` roots it as a global has to be released somewhere.
    let frame = interp.roots.push_frame();
    let installed = install_enhancing_class_methods(interp, class, enhancing, &names);
    interp.roots.pop_frame(frame);
    installed
}

/// [`enhance_class_methods`]'s walk, split out so the root frame it runs
/// inside is released on the failure path too.
fn install_enhancing_class_methods(
    interp: &mut Interp,
    class: ObjRef,
    enhancing: ObjRef,
    names: &[Box<[u8]>],
) -> Result<(), Failure> {
    for name in names {
        let value = interp.native_entry(enhancing, name).unwrap_or(ObjRef::NIL);
        if interp.receiver_kind(value) != Ok(Primitive::Method) {
            // **Read as source before declining it**, so that a source the
            // oracle refuses is refused here the same way rather than
            // reaching the loud arm below: measured, oracle rc 163, a
            // literal array `('return 1', , 'nop')` in this table is
            // `93.952 Method argument method source is an array and does
            // not contain all string values.`
            method_source_lines(interp, value, "method source")?;
            return Err(Loud::method_from_source("a class method built from source text").into());
        }
        interp
            .define_class_method_object(class, name, value)
            .ok_or_else(|| {
                Failure::from(Loud::method_from_source(
                    "a class method whose body this crate does not hold",
                ))
            })?;
    }
    Ok(())
}

/// `Class~delete(name)`: take one instance method back off the receiver --
/// `RexxClass::deleteMethod` (`classes/ClassClass.cpp:952`).
///
/// A name the class does not define is not an error: measured, oracle rc 0
/// with nothing on either descriptor.
fn native_delete(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    rexx_defined_lock(interp, class)?;
    let name = method_name_argument(interp, args)?;
    interp
        .classes()
        .delete_instance_method(class, &String::from_utf8_lossy(&name));
    interp.drop_method_object(class, &name);
    Ok(None)
}

/// `Class~inherit(mixin, position)`: add a mixin to the receiver's
/// superclass list -- `RexxClass::inherit` (`classes/ClassClass.cpp:1287`),
/// the same function the `INHERIT` keyword of a `::CLASS` directive reaches
/// by sending this message ([`Interp::inherit_mixin`]).
///
/// `position` is optional and says where in the list the mixin lands --
/// [`rexx_classes::ClassGraph::inherit_at`] carries what "after" means
/// there.
fn native_class_inherit(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    rexx_defined_lock(interp, class)?;
    let mixin = mixin_class_argument(interp, args)?;
    // **The position is not type-checked, and that is the oracle's shape.**
    // `RexxClass::inherit` uses it in one place, `superClasses->indexOf(position)`
    // (`classes/ClassClass.cpp:1346`), and a value that is not in the list gives
    // index 0 and `Error_Execution_uninherit` at `:1350` whatever kind of
    // object it was. Measured at rc 158, `::class K` inheriting `.M1`:
    // `.K~inherit(.M2, "abc")` is `98.945`, `Class "The K class" has not
    // inherited class "abc".` -- the second argument's error and not the
    // first's.
    let position = args.get(1).copied().flatten();
    match interp.classes().inherit_at(class, mixin, position) {
        Ok(()) => Ok(None),
        Err(refusal) => Err(inherit_refusal(interp, class, mixin, refusal)),
    }
}

/// `Class~defineClassMethod(name, method)`: install `method` as a class-side
/// method of the receiver -- `RexxClass::defineClassMethod`
/// (`classes/ClassClass.cpp:883`), which writes the class behaviour and
/// `classMethodDictionary` from one `newScope` copy.
///
/// **No `REXX_DEFINED` lock**, and that is the C++: every mutator
/// [`rexx_defined_lock`] guards opens with `isRexxDefined()` and this one
/// does not, which is what lets `CoreClasses.orx:73` put its string
/// constants on `.String`.
///
/// **Deleted from every shipped image**, so nothing here can be measured
/// against the oracle: `removeSetupMethods` strips it (D39). The refusals
/// are loud for that reason -- see [`SETUP_METHODS`].
fn native_define_class_method(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let name = method_name_argument(interp, args)?;
    let Some(Some(source)) = args.get(1).copied() else {
        return Err(Loud::setup_method("defineClassMethod with no method object").into());
    };
    if interp.receiver_kind(source) != Ok(Primitive::Method) {
        return Err(
            Loud::setup_method("defineClassMethod with a value that is not a method").into(),
        );
    }
    interp
        .define_class_method_object(class, &name, source)
        .ok_or_else(|| {
            Failure::from(Loud::setup_method(
                "defineClassMethod with a method object whose body this crate did not record",
            ))
        })?;
    Ok(None)
}

/// `Class~inheritInstanceMethods(source)`: copy `source`'s own instance
/// methods into the receiver's dictionary at the receiver's scope, with no
/// superclass edge added -- `RexxClass::inheritInstanceMethods`
/// (`classes/ClassClass.cpp:558`), the "phony inherit" `CoreClasses.orx:77`
/// names in its own comment.
///
/// **No `REXX_DEFINED` lock**, for the reason
/// [`native_define_class_method`] gives, and no oracle transcript either.
fn native_inherit_instance_methods(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Loud::setup_method("inheritInstanceMethods with no source class").into());
    };
    let Some(source) = argument.class_id().map(|_| argument) else {
        return Err(
            Loud::setup_method("inheritInstanceMethods with a value that is not a class").into(),
        );
    };
    let source = class_receiver(interp, source)?;
    interp.classes().inherit_instance_methods(class, source);
    interp.classes().check_uninit(class);
    Ok(None)
}

/// `Class~uninherit(mixin)`: take a mixin back out of the receiver's
/// superclass list -- `RexxClass::uninherit` (`classes/ClassClass.cpp:1379`).
fn native_uninherit(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    rexx_defined_lock(interp, class)?;
    let mixin = mixin_class_argument(interp, args)?;
    match interp.classes().uninherit(class, mixin) {
        Ok(()) => Ok(None),
        Err(refusal) => Err(inherit_refusal(interp, class, mixin, refusal)),
    }
}

/// The `mixin class` argument `~inherit` and `~uninherit` share: required,
/// and a `MIXINCLASS` class object or 98.942 naming the value
/// (`classes/ClassClass.cpp:1298`-`:1301`, `:1391`-`:1394`).
///
/// **The mixin test is the graph's and the class-object test is here**,
/// because the two report the same error and only one of them has a class to
/// ask about. Measured at rc 158: `.K~uninherit('abc')` reports `Class "abc"
/// must be a MIXINCLASS for INHERIT.` and `.K~uninherit(.Object)` reports the
/// same sentence with `The Object class` in it.
fn mixin_class_argument(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("mixin class").into());
    };
    class_receiver(interp, argument).map_err(|_| {
        let shown = interp.string_value_text(argument);
        Failure::from(Raised::inherit_needs_a_mixinclass(&shown))
    })
}

/// One [`InheritRefusal`] as the condition the oracle reports in its place,
/// with the substitutions rendered the way the message renders an object:
/// `stringValue()`, which is `~objectName` and so follows a rename.
fn inherit_refusal(
    interp: &mut Interp,
    class: ObjRef,
    mixin: ObjRef,
    refusal: InheritRefusal,
) -> Failure {
    let class_name = interp.string_value_text(class);
    let mixin_name = interp.string_value_text(mixin);
    match refusal {
        InheritRefusal::NotAMixin => Raised::inherit_needs_a_mixinclass(&mixin_name).into(),
        InheritRefusal::Recursive => Raised::recursive_inherit(&class_name, &mixin_name).into(),
        InheritRefusal::BaseClass(base) => {
            let base_name = interp.string_value_text(base);
            Raised::inherit_base_class(&class_name, &mixin_name, &base_name).into()
        }
        InheritRefusal::NotInherited(other) => {
            let other_name = interp.string_value_text(other);
            Raised::not_inherited(&class_name, &other_name).into()
        }
    }
}

/// `RexxContext~package`: the package of the program the running activation
/// belongs to -- `RexxContext::getPackage` (`classes/ContextClass.cpp:160`).
///
/// **The one route to the running program's package object**, which is why
/// it is here rather than left to `Class~package`: measured,
/// `.Array~package~name` is `REXX`, so a class the bootstrap registered
/// reaches the interpreter's own package and not this one.
fn native_context_package(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // `checkValid` (`ContextClass.cpp:162`) is what refuses a context object
    // whose activation has returned; nothing here hands one out that
    // outlives its activation, and a context object exists only while one is
    // running.
    interp
        .running_package_object()
        .map(Some)
        .ok_or_else(|| Loud::receiver_class("a context object outside a running program").into())
}

/// `Package~addClass(name, class)` and `Package~addPublicClass(name, class)`
/// -- `PackageClass::addClassRexx` (`classes/PackageClass.cpp:1926`) and
/// `addPublicClassRexx` (`:1944`), which differ only in the flag they hand
/// `addInstalledClass`.
///
/// Both answer the package object itself (`return this`, `:1933`), which is
/// measured: `p~addClass("zz", .K) == p` is `1`.
fn native_package_add_class(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    add_installed_class(interp, cleared, receiver, args, false)
}

/// `Package~addPublicClass` -- see [`native_package_add_class`].
fn native_package_add_public_class(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    add_installed_class(interp, cleared, receiver, args, true)
}

/// The body both `~addClass` rows share, since a second implementation is
/// where the two could come to disagree.
///
/// The arguments are validated in the C++'s order: `stringArgument(name,
/// "name")`, then `classArgument(clazz, TheClassClass, "class")`, then
/// `checkRexxPackage`. Measured at rc 168, `~addClass()` is `Missing
/// argument; argument name is required.`, `~addClass("a")` is the same
/// sentence for `class`, and `~addClass("a", "b")` is 88.914 `Argument class
/// must be an instance of the Class class.`
fn add_installed_class(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    public: bool,
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(name)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("name").into());
    };
    let name = required_string_named_argument(interp, name, "name")?;
    let name = interp.to_text(name).to_vec();
    let Some(Some(class)) = args.get(1).copied() else {
        return Err(Raised::missing_named_argument("class").into());
    };
    if class.class_id().is_none() {
        return Err(Raised::argument_not_a_class("class").into());
    }
    match interp.which_package(receiver) {
        Some(Package::Program(program)) => {
            interp.add_installed_class(program, &name, class, public);
            Ok(Some(receiver))
        }
        Some(Package::Rexx) => Err(Raised::rexx_package_addition().into()),
        None => Err(Loud::receiver_class("a package object this crate did not build").into()),
    }
}

/// `Package~publicClasses`: a fresh `StringTable` of the classes this
/// package exports -- `PackageClass::getPublicClassesRexx`
/// (`classes/PackageClass.cpp:1563`).
///
/// The REXX package's own table is [`Loud::rexx_package_classes`], which
/// carries why.
fn native_package_public_classes(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    match interp.which_package(receiver) {
        Some(Package::Program(program)) => Ok(Some(interp.public_classes_table(program))),
        Some(Package::Rexx) => Err(Loud::rexx_package_classes().into()),
        None => Err(Loud::receiver_class("a package object this crate did not build").into()),
    }
}

/// An array receiver's own slots, borrowed, or the refusal for a receiver that
/// is not one.
///
/// The refusal is unreachable -- every row that reaches one of these callers
/// is in `.Array`'s own dictionary, and the only receiver whose behaviour that
/// dictionary reaches is a `Body::Array`. Loud rather than a panic, this
/// crate's rule for an internal inconsistency.
fn array_slots(interp: &Interp, receiver: ObjRef) -> Result<&[Option<ObjRef>], Failure> {
    let receiver = collection_store(interp, receiver);
    interp
        .array_slots(receiver)
        .ok_or_else(|| Loud::receiver_class("a value that is not an array").into())
}

/// The array that actually holds `receiver`'s slots: the receiver itself when
/// it carries a `Body::Array`, and otherwise the store its own pool holds.
///
/// **One resolution point for every Array-shaped receiver**, which is what
/// lets one body serve `.Array`, a `Queue`, a `CircularQueue` and a user
/// subclass of any of them. A `Body::Array` resolves to `Primitive::Array`
/// wherever it is asked and so cannot answer a subclass's `~class`; spec D90
/// and `collection::store_of` carry the whole argument.
///
/// Answers the receiver unchanged when there is no store, so the refusal a
/// caller already raises stays the one it raises.
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
fn array_slots_owned(interp: &Interp, receiver: ObjRef) -> Result<Vec<Option<ObjRef>>, Failure> {
    Ok(array_slots(interp, receiver)?.to_vec())
}

/// An array receiver's dimensions array as an owned copy, or the refusal
/// [`array_slots`] gives for a receiver that is not an array.
///
/// `None` is an array no dimension list was fixed for, which is not the same
/// as a one-element list -- see [`rexx_core::Body::Array`].
fn array_dimensions(interp: &Interp, receiver: ObjRef) -> Result<Option<Vec<usize>>, Failure> {
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
const MAX_FIXED_ARRAY_SIZE: usize = 100_000_000_000_000_000;

/// `size` empty slots, or the 5.0 the oracle raises when the allocator refuses
/// -- [`Raised::system_resources`] carries why that refusal is asked of the
/// allocator rather than of a size limit.
///
/// [`MAX_FIXED_ARRAY_SIZE`] is a different check and runs first: a size above
/// it is 93.959, and a size below it the allocator cannot satisfy is this.
/// `corpus/lang/array_allocation_refused.rex` holds both against the oracle.
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
enum IndexUse {
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
///
/// Under [`IndexUse::Put`] the receiver grows to hold the position, so the
/// answer is always `Some`.
///
/// **A lone array argument is the subscript list**, spread by taking its item
/// count alongside its slot array (`:1219`-`:1226`). Measured, that is item
/// count and not size: `(1,2)~at((1,))` answers `1` where `(1,2)~at((1,2))` is
/// 93.926 and `(1,2)~at((,))` is 93.901.
fn array_position(
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
///
/// A subscript past the end of the array is **not** an error under
/// [`IndexUse::Get`] -- measured, `(1,2)~at(100000000000000001)` answers
/// `The NIL object` even though that is past `MaxFixedArraySize` -- and only
/// the count and the conversion raise.
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
///
/// Measured, oracle rc 0: a 2 by 3 array with `m[i, j]` set to `i || j` at
/// every cell renders `11 21 12 22 13 23` through `~toString('l', ' ')`.
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

/// A subscript converted under `Numerics::ARGUMENT_DIGITS` rather than under
/// the activation's own `NUMERIC DIGITS`, or `None` for one that is not a
/// whole number of at least 1.
///
/// The fixed precision is measured rather than read off the default argument:
/// `numeric digits 3; say (1,2)~at(1000000)` answers `The NIL object` where a
/// conversion at 3 digits would have rounded the subscript.
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
///
/// Measured on `.List~of('a','b','c')`, whose handles are `0`, `1` and `2`:
/// `hasIndex('-0')` is `1`, `hasIndex('1e1')` is `0` -- it converts to ten,
/// which the list does not hold -- and `hasIndex('1e300')` raises, because no
/// `size_t` holds it.
fn unsigned_index(interp: &mut Interp, value: ObjRef) -> Option<usize> {
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
///
/// `None` is the 26.902/26.903 limb. Measured: `'1.0'`, `'-1.0'` and `'0.0'`
/// all convert and sort, where reading the answer's text for a sign left the
/// array in its original order.
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
///
/// Measured at rc 163: `(1,2)~at(0)` reports `Method argument 1 must be a
/// positive whole number; found "0".` and `(1,2)~put('v',0)` reports the same
/// for argument 2.
fn positive_index(interp: &mut Interp, value: ObjRef, position: usize) -> Result<usize, Failure> {
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
///
/// Measured at rc 163, `m = .array~new(2,3)`: `m[,2]` reports `Missing
/// argument in method; argument 2 is required.`, `m[1.5,1]` reports `Invalid
/// position argument specified; found "1.5".` and `m[.array,1]` reports the
/// same with `found "The Array class"`. **The second names no position and
/// renders the argument rather than its converted value.**
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
///
/// Each dimension becomes the larger of the subscript and the extent it had;
/// an array with no dimensions array takes the subscripts as its shape, and
/// the C++ reaches that only at size zero. **The subscripts are validated
/// against `i + 1` here** rather than against the position the caller counts
/// from (`:2457`, `:2515`), which is visible only in a `93.903`.
///
/// **The C++ bounds the product in `createMultidimensional` (`:217`-`:220`)
/// and not here**; this raises the same 93.959 in both places rather than
/// wrapping.
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
///
/// `old` is the shape the slots are laid out under, and is `None` for a
/// receiver that had no compatible shape -- which the C++ reaches only at
/// size zero, so nothing moves.
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
///
/// Measured, `a = (1,,3)`: `a[1]` is `1`, `a[2]` is `The NIL object`, `a[3]` is
/// `3` and `a[4]` is `The NIL object`.
fn native_array_at(
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
fn native_array_at_for(
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
///
/// Measured at rc 163: `(1,2)~put('v')` reports `Not enough arguments for
/// method; 2 expected.` and `(1,2)~put(,1)` reports `Missing argument in
/// method; argument 1 is required.` Measured at rc 0: `a = (1,2)` then
/// `a~put('v',5)` leaves `a~size` `5` and `a~items` `3`.
fn native_array_put(
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
///
/// Measured, oracle rc 0: `.array~new()~dimension` is `0` and its
/// `~dimension(1)` is `0`; `.array~new(0)~dimension` is `1`;
/// `.array~new(5)~dimension(1)` is `5` and its `~dimension(2)` is `0`;
/// `(1,2)~dimension` is `1`; and `.array~new(2,3)` answers `2`, `2`, `3` and
/// then `0` for `~dimension(3)`.
fn native_array_dimension(
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
///
/// One argument that is not an array is the slot count; a lone array
/// argument, or more than one argument, is the dimension list
/// (`createMultidimensional`, `:199`), spread the way a subscript list is;
/// and no argument at all is an empty array whose shape is not yet fixed.
/// `INIT` is sent with no arguments, because `completeNewObject(temp)`
/// (`classes/ClassClass.cpp:1882`) takes the default empty list.
///
/// Measured, oracle rc 0: `.array~new((2,3))~size` is `6`, and
/// `.array~new(0)~dimension` is `1` where `.array~new()~dimension` is `0` --
/// an explicit zero size fixes the shape where an omitted one does not.
fn native_array_new(
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
///
/// A `Body::Array` resolves to `Primitive::Array` wherever it is asked, so it
/// cannot answer a subclass's `~class` -- spec D90 and
/// `dispatch::collection::store_of` carry the whole of that argument.
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
///
/// An omitted argument is an empty slot, and an empty argument list fixes the
/// shape the way an explicit zero size does. Measured, oracle rc 0:
/// `.array~of(1,,3)` is `~size` `3` and `~items` `2`, and `.array~of()` is
/// `~dimension` `1` where `.array~new()` is `0`
/// (`ArrayClass::ArrayClass(objs, count)`, `:314`).
fn native_array_of(
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
///
/// A one-element dimension list is a single-dimensional array that still
/// carries a dimensions array, which is [`rexx_core::Body::Array`]'s own
/// distinction.
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
///
/// Measured at rc 163: `.array~new(-1)` reports `Method argument 1 must be
/// zero or a positive whole number; found "-1".`, `.array~new(2,'-1.0')`
/// reports the same for argument 2 and renders the argument rather than its
/// converted value, `.array~new((,3))` reports `Missing argument in method;
/// argument 1 is required.` and `.array~new(100000000000000001)` reports `An
/// array cannot contain more than 100000000000000000 elements.`
fn array_size_argument(
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
///
/// Measured, `(1,)~size` is `2`, `(,)~size` is `2` and `(1,,3)~size` is `3`:
/// the list expression's own array is `new_array(expressionCount)`
/// (`expression/ExpressionList.cpp:93`), so a trailing omission is a slot.
fn native_array_size(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let size = array_slots(interp, receiver)?.len();
    Ok(Some(interp.counted(size)))
}

/// `Array~items`: how many slots hold an object -- `ArrayClass::itemsRexx`.
///
/// **Not `~size`, and an explicit `.nil` is an item.** Measured,
/// `(1,,3)~items` is `2` while `(1,.nil,3)~items` is `3`, and both are
/// `~size` `3`.
fn native_array_items(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let items = array_slots(interp, receiver)?.iter().flatten().count();
    Ok(Some(interp.counted(items)))
}

/// The index argument a hash-collection method was given, as the bytes it is
/// stored and looked up under.
///
/// `stringArgument(index, "index")` (`runtime/MethodArguments.hpp:161`), which
/// raises 88.901 for an omitted argument and 88.909 for a value with no string
/// value. Measured at rc 168: `.environment~at()` reports `Missing argument;
/// argument index is required.` and `.environment~at(.nil)` reports `Argument
/// index must have a string value.`
///
/// **The bytes are not upcased.** An index is stored and matched verbatim --
/// measured, `d~put('v','kk')` leaves `d['kk']` `v` and `d['KK']`
/// `The NIL object`, and `.environment['array']` is `The NIL object` where
/// `.environment['ARRAY']` is `The Array class`. The entries `Setup.cpp`
/// registers are uppercase because `completeSystemClass` upcases the *name it
/// registers*, not because a lookup folds case. [`native_hash_unknown`] is
/// where the fold does happen, because `entry` and `setEntry` are the
/// upcasing pair and `get`/`put` are not.
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
///
/// Measured, `.environment['ARRAY']` is `The Array class`,
/// `.environment['x']` is `The NIL object`, and `.methods['Z']` is `a Method`
/// in a file whose only directive is `::method z`.
fn native_hash_at(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // **One body, two stores.** `Setup.cpp` donates `IdentityTable`'s `At`,
    // `Put` and `[]` rows to `StringTable` and that whole set on to
    // `Directory` (`memory/Setup.cpp:881`, `:933`), so `Table`,
    // `IdentityTable`, `StringTable` and `Directory` share one method
    // identity here exactly as they share one function upstream. The
    // string-keyed classes read the entry map they are built on; everything
    // else reads the object-keyed store.
    if !matches!(
        interp.heap.get(receiver).map(|object| &object.body),
        Some(Body::Native(_))
    ) && hash::owns(interp, receiver)
    {
        return hash::store_at(interp, receiver, args);
    }
    let index = hash_index(interp, args, 1)?;
    Ok(Some(interp.hash_entry_read(receiver, &index)?))
}

/// `~put(item, index)` on a `Directory` or a `StringTable`: stores `item`
/// under `index`, replacing whatever was there -- `HashCollection::putRexx`.
///
/// **The item is argument one and the index argument two**, which is the order
/// `CoreClasses.orx:65` writes (`.environment~put(class, name)`). Measured at
/// rc 168, the two refusals in the order the C++ checks them:
/// `.environment~put()` reports `Missing argument; argument item is required.`
/// and `.environment~put('a')` reports `... argument index is required.`
///
/// **Answers no value**, measured: `.environment~put('v','q')` is rc 0 as a
/// whole clause and 91.999 at rc 165 under `say`.
fn native_hash_put(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // [`native_hash_at`]'s split, for the same reason.
    if !matches!(
        interp.heap.get(receiver).map(|object| &object.body),
        Some(Body::Native(_))
    ) && hash::owns(interp, receiver)
    {
        return hash::store_put(interp, receiver, args);
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
///
/// A name ending in `=` stores the send's own first argument under the name
/// without it (`:1020` is the test); every other name reads the entry. So an entry
/// answers a message of its own name without being in the behaviour at all --
/// measured, `.environment~local~class` is `The Directory class` while
/// `.environment~hasMethod("LOCAL")` is `0`.
///
/// **Both directions fold the name to upper case**, which `~at` and `~put` do
/// not: `entry` and `setEntry` are `get` and `put` with `index->upper()`
/// (`:824`, `:854`). Measured, `.local~"mything="('v')` then `.local["MYTHING"]`
/// is `v` and `.local["mything"]` is `The NIL object`.
///
/// **The set form with no value argument is refused, and there is no oracle
/// behaviour to match.** `unknown` reads `arguments[0]` whatever the argument
/// count is, so a send that supplies none reads uninitialised memory:
/// measured, `d~mything = 'v'` then `say 'a' d["MYTHING"]` then
/// `d~"MYTHING="()` leaves `d["MYTHING"]` holding `a v` -- the string the
/// intervening `SAY` had just built. `d~"MYTHING="(,)` and `(,,)` do the same.
/// The message-assignment form always supplies a value, so nothing that
/// reaches this from `receiver~NAME = expr` takes the refusal.
///
/// `arguments` is an `Array` on every send the interpreter itself forwards
/// ([`Interp::unknown_or_nomethod`] builds one). A program sending `~UNKNOWN`
/// by hand may pass anything, and the oracle converts it with `requestArray`;
/// this crate has no `MAKEARRAY` for any receiver, so a value that is not
/// already an array is the same refusal `~request('ARRAY')` gives.
fn native_hash_unknown(
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
    // [`native_hash_at`]'s split again: a collection reads its store and the
    // environment reads its map. Measured, a `.Directory` given
    // `setEntry('alpha', 42)` answers `d~alpha` as `42`, and `d~beta = 7`
    // stores under `BETA`.
    let store = !matches!(
        interp.heap.get(receiver).map(|object| &object.body),
        Some(Body::Native(_))
    ) && hash::owns(interp, receiver);
    let Some(index) = name.strip_suffix(b"=") else {
        let index = name.to_ascii_uppercase();
        if store {
            let index = interp.text_built(index);
            return hash::store_entry_read(interp, receiver, index);
        }
        return Ok(Some(interp.hash_entry_read(receiver, &index)?));
    };
    let index = index.to_ascii_uppercase();
    let Some(Some(item)) = forwarded.first().copied() else {
        return Err(Loud::entry_method_without_a_value(&index).into());
    };
    if store {
        let index = interp.text_built(index);
        return hash::store_entry_write(interp, receiver, index, item);
    }
    interp.hash_entry_write(receiver, &index, item)?;
    Ok(None)
}

/// `VariableReference~name`: the referenced variable's own spelling --
/// `VariableReference::getName` (`classes/VariableReference.cpp:150`).
///
/// Measured, oracle rc 0: `vr = 5; say (>vr)~name` is `VR`, upper case
/// whatever the source spelled, and `s. = 0; say (>s.)~name` is `S.`.
fn native_reference_name(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(reference) = interp.as_variable_reference(receiver) else {
        return Err(Loud::receiver_class("a value that is not a variable reference").into());
    };
    let name = reference.name.to_vec();
    Ok(Some(interp.text_built(name)))
}

/// `VariableReference~value`: what the referenced variable holds --
/// `VariableReference::getValue` (`classes/VariableReference.cpp:161`), whose
/// `getResolvedValue` answers the variable's derived name when it holds
/// nothing.
///
/// Measured, oracle rc 0: with `vr` unassigned, `(>vr)~value` is `VR`; and
/// after `vr = 5`, `o = >vr`, `vr = 'changed'`, `o~value` is `changed` --
/// the read is at the ask and not at the reference.
fn native_reference_value(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(reference) = interp.as_variable_reference(receiver) else {
        return Err(Loud::receiver_class("a value that is not a variable reference").into());
    };
    match interp.referenced_value(reference) {
        Some(value) => Ok(Some(value)),
        None => {
            let name = reference.name.to_vec();
            Ok(Some(interp.text_built(name)))
        }
    }
}

/// `VariableReference~value=`: writes the referenced variable --
/// `VariableReference::setValueRexx` (`classes/VariableReference.cpp:190`),
/// whose `requiredArgument` is the 93.903 an omitted value raises.
///
/// Measured, oracle rc 0: `vr = 5; o = >vr; o~value = 7; say vr` is `7`, and
/// the same through a reference returned by a `procedure` after its frame is
/// gone. Its `requiredArgument(v, "VALUE")` names the argument rather than
/// numbering it: measured, oracle rc 168, `o~"VALUE="()` reports
/// `88.901 Missing argument; argument VALUE is required.`
fn native_reference_value_set(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(value)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("VALUE").into());
    };
    let Some(reference) = interp.as_variable_reference(receiver) else {
        return Err(Loud::receiver_class("a value that is not a variable reference").into());
    };
    let home = reference.home.clone();
    let name = reference.name.clone();
    // `RexxVariable::setValue` "sorts out the stem vs. simple assignment
    // bits" (`classes/VariableReference.cpp:184`), so a stem reference
    // assigns what the bare `stem. = value` assigns rather than storing the
    // value itself -- a plain value in a stem-named slot is a state nothing
    // else in this crate can produce.
    let value = if crate::run::shape_of(&name) == crate::run::NameShape::Stem {
        interp.stem_assignment_value(&name, value)
    } else {
        value
    };
    match home {
        rexx_core::VarRefHome::Cell(cell) => interp.roots.set_slot_value(cell, value),
        rexx_core::VarRefHome::Instance { owner, scope } => {
            interp.set_pool_variable(owner, scope, &name, value);
        }
    }
    Ok(None)
}

/// `VariableReference~unknown(name, arguments)`: forwards to the referenced
/// value -- `VariableReference::unknownRexx`
/// (`classes/VariableReference.cpp:205`), which is what makes every message
/// the class does not define read as one to the variable's value.
///
/// The hidden comparison operators reach this too
/// (`memory/Setup.cpp:1307`-`:1312`). Measured, oracle rc 0: `vr = 5;
/// o = >vr; say o~length` is `1` and `say o == 5` is `1`.
fn native_reference_unknown(
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
    let referenced = referenced_receiver(interp, receiver)?;
    let caller = interp.caller();
    interp.send_message(referenced, &name, None, &forwarded, caller)
}

/// `VariableReference~request(class)`: forwards to the referenced value --
/// `VariableReference::request` (`classes/VariableReference.cpp:359`), which
/// handles none of them itself.
///
/// Measured, oracle rc 0: `vr = 5; say (>vr)~request('STRING')~class~id` is
/// `String`.
fn native_reference_request(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let referenced = referenced_receiver(interp, receiver)?;
    let caller = interp.caller();
    interp.send_message(referenced, b"REQUEST", None, args, caller)
}

/// The value a reference receiver names, materialised as an object so that a
/// message can be sent to it.
///
/// `getResolvedValue`'s own answer for an unset variable is its derived name,
/// which for a simple or stem symbol is the reference's own spelling.
fn referenced_receiver(interp: &mut Interp, receiver: ObjRef) -> Result<ObjRef, Failure> {
    interp
        .referenced_object(receiver)
        .ok_or_else(|| Loud::receiver_class("a value that is not a variable reference").into())
}

/// `RexxObject::requestArray` (`runtime/MethodArguments.hpp:683`): the value
/// itself when it already is an array, and its `MAKEARRAY` otherwise.
///
/// Upstream this is a `REQUEST` send, which answers `.nil` for a receiver
/// whose behaviour has no `MAKEARRAY` and lets one that does raise through.
/// The `.nil` limb keeps [`unconverted_array_argument`]'s loud refusal here
/// rather than becoming upstream's `Error_Execution_noarray`: that gap is
/// unchanged by this function and is not what it is for.
///
/// It exists because the collection classes now answer `MAKEARRAY`, and the
/// refusal was written when no receiver did. Measured, oracle and crate
/// agreeing: `.Supplier~new(.List~of('a'), .List~of('h'))` answers
/// `1 a h`, where the direct slot read refused it.
fn request_array(interp: &mut Interp, value: ObjRef) -> Result<ObjRef, Failure> {
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
///
/// `arrayArgument` converts with `requestArray`, which is a `MAKEARRAY` send,
/// so both arms name a step of that send: the method for a value whose class
/// this crate has, and the send itself for a value it does not build a class
/// for at all. Either way the message reads like the one `~request('ARRAY')`
/// produces for the same value.
fn unconverted_array_argument(interp: &mut Interp, value: ObjRef) -> Failure {
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
///
/// `L` joins the items with `separator`, defaulting to the line ending; `C`
/// concatenates them and **refuses a separator**, which is where the method's
/// own declared arity of 2 and the 93.902 it raises for its own second
/// argument come apart. Measured at rc 163:
/// `.Array~superClasses~makeString('C', '-')` reports `1 expected` where
/// `.Array~superClasses~makeString('L', ' ', 'z')` reports `2 expected`, and
/// `makeString('X')` is 93.915 naming `"CL"`.
fn native_array_make_string(
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

/// `Class~package`: the package the class was defined in --
/// `RexxClass::getPackage` (`classes/ClassClass.cpp:1732`).
///
/// Measured, three descriptors: `.Array~package` renders `The REXX Package`
/// and `.Array~package~name` is `REXX`, while a `::CLASS` in a user file
/// answers `a Package` whose `~name` is that file's own path. Measured too,
/// `(.Array~package == .String~package)` is `1` and
/// `(.K~package == .Array~package)` is `0`.
fn native_package(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    Ok(Some(interp.package_object_for(class)))
}

/// `Package~local`: the package's own environment directory, which is
/// `rexxpg`'s step 5 of the environment-symbol search order and the one route
/// a program has to it.
///
/// Measured, oracle rc 0: empty on the first ask, the same object on every
/// ask, and an entry written into it answers a `.NAME` ahead of `.local` --
/// `PackageClass::findClass` reads `packageLocal` between the REXX package's
/// public classes and `ActivityManager::getLocalEnvironment`
/// (`classes/PackageClass.cpp:1122`).
fn native_package_local(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(package) = interp.which_package(receiver) else {
        return Err(Loud::receiver_class("a package object this crate did not build").into());
    };
    Ok(Some(interp.package_local(package)))
}

/// `Package~name`: the package's own name -- `PackageClass::getProgramName`
/// (`classes/PackageClass.hpp:147`), bound as `Name` by `memory/Setup.cpp:1189`.
///
/// `REXX` for the package the primitive classes belong to, and the program's
/// own path for a package a `::CLASS` installed into.
fn native_package_name(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // Loud rather than a panic where the receiver is a package handle this
    // crate did not build, which is `Interp::package_name`'s own `None`.
    let Some(name) = interp.package_name(receiver) else {
        return Err(Loud::receiver_class("a package object this crate did not build").into());
    };
    Ok(Some(interp.text_built(name)))
}

/// `Object~string`: the receiver's readable string representation --
/// `RexxObject::stringValue` (`classes/ObjectClass.cpp:1157`), which is an
/// `OBJECTNAME` send, except for a `String`, whose own override answers
/// itself.
///
/// **Not the required-string protocol's answer, and that is what this method
/// is for.** Measured, oracle rc 0, with a class-side `makeString` returning
/// `'K says hello'`: `say .K` prints `K says hello` where `say .K~string`
/// prints `The K class`. The documented example in `provide.xml` `reqstr` is
/// exactly that gap -- `substr(d~string,3,6)` reads the readable form where
/// `substr(d,5,7)` reads the converted one.
fn native_string(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // **Sent rather than shortcut for an instance**, whose `OBJECTNAME` a
    // program can replace: measured, oracle rc 0, `::METHOD defaultName`
    // returning `overridden` makes `o~string` answer `overridden`.
    if matches!(
        interp.receiver_kind(receiver),
        Ok(Primitive::Instance { .. })
    ) {
        let caller = interp.caller();
        let answered = interp.send_message(receiver, OBJECTNAME, None, &[], caller)?;
        if let Some(answered) = answered {
            let text = interp.string_value_text(answered);
            return Ok(Some(interp.text_built(text)));
        }
    }
    let text = interp.string_value_text(receiver);
    Ok(Some(interp.text_built(text)))
}

/// `Object~objectName`: the name something gave this object, or
/// `RexxObject::defaultName` for one nothing has named
/// (`classes/ObjectClass.cpp:1696`, `:1760`).
///
/// **`~string` and this part on a string**, which is the whole reason this is
/// not [`native_string`]: `RexxString::stringValue` answers the string itself
/// where its `defaultName` is the article rule applied to `String`. Measured,
/// oracle rc 0: `'abc'~string` is `abc` and `'abc'~objectName` is `a String`.
/// The receivers whose name is not derived carry it -- measured,
/// `.local~objectName` is `The Local Directory` and `.Array~package
/// ~objectName` is `The REXX Package`, both of which `CoreClasses.orx`
/// assigns -- and for every one of those `stringValue()` is already the
/// answer.
fn native_object_name(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let kind = interp
        .receiver_kind(receiver)
        .map_err(|kind| Failure::from(Loud::receiver_class(kind)))?;
    let name = match kind {
        // `defaultName` from `.String`'s own id, which no object of this kind
        // carries a stored name for -- there is nowhere on a string to put
        // one, which is also why `native_object_name_set` refuses it.
        Primitive::String | Primitive::SmallInt => b"a String".to_vec(),
        // **A set name, or the answer to a `DEFAULTNAME` send.** The oracle
        // reads the object's own `Object`-scope variable and, for anything
        // that is not a base class, sends rather than deriving -- so a class
        // overriding `defaultName` decides what an unnamed instance answers,
        // and a name set afterwards wins over it. Both measured, oracle rc 0.
        Primitive::Instance { .. } => match interp.instance_name(receiver) {
            Some(held) => held,
            None => {
                let caller = interp.caller();
                let answered = interp.send_message(receiver, DEFAULTNAME, None, &[], caller)?;
                match answered {
                    Some(answered) => interp.string_value_text(answered),
                    None => interp.string_value_text(receiver),
                }
            }
        },
        // Derived from the class id rather than read through
        // `string_value_text`, which for this receiver answers the
        // *referenced* value: `~objectName` is the reference's own and is
        // one of the names its `UNKNOWN` never sees. Measured, oracle rc 0:
        // `vr = 5; o = >vr; say o~objectName` is `a VariableReference` where
        // `say o~string` is `5`.
        Primitive::VariableReference => {
            crate::environment::default_object_name("VariableReference").into_bytes()
        }
        // Derived for the same reason, and the two answers part here too: a
        // stem's `string_value_text` is its default. Measured, oracle rc 0:
        // `s. = 'dflt'; o = s.; say o~objectName` is `a Stem` where
        // `say o~string` is `dflt`.
        Primitive::Stem => crate::environment::default_object_name("Stem").into_bytes(),
        Primitive::Object
        | Primitive::Array
        | Primitive::Class(_)
        | Primitive::Package
        | Primitive::Method
        | Primitive::Routine
        | Primitive::Directory
        | Primitive::StringTable(_)
        | Primitive::Context
        | Primitive::RexxInfo
        | Primitive::Message => interp.string_value_text(receiver),
    };
    Ok(Some(interp.text_built(name)))
}

/// `Object~objectName=`: replaces what `~objectName` answers, and with it
/// every rendering of the object -- `RexxObject::objectNameEquals`
/// (`classes/ObjectClass.cpp:1733`), whose own return is `OREF_NULL`.
///
/// Measured, oracle rc 0: after `.K~objectName = "renamed"`, `say .K`,
/// `say .K~objectName` and `say .K~string` all print `renamed`, while
/// `.K~request("STRING")` still answers `The NIL object` -- the rename moves
/// `stringValue()` and not the conversion.
///
/// **A receiver with nowhere to keep a name is loud rather than silent.** The
/// oracle stores this in the object's own variable pool at `Object` scope, and
/// a string, a number and a stem have no pool in this crate; answering rc 0
/// and forgetting the name would be a wrong answer where the oracle
/// remembers it.
fn native_object_name_set(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_method_argument(1).into());
    };
    let argument = required_string_argument(interp, argument, 1)?;
    let name = interp.to_text(argument).into_owned();
    let kind = interp
        .receiver_kind(receiver)
        .map_err(|kind| Failure::from(Loud::receiver_class(kind)))?;
    match kind {
        Primitive::Class(class) => {
            let name = String::from_utf8_lossy(&name).into_owned();
            interp.classes().set_object_name(class, &name);
        }
        Primitive::Instance { .. } => {
            let Some(object) = interp.heap.get_mut(receiver) else {
                return Err(Loud::receiver_class("a value whose object is no longer live").into());
            };
            match &mut object.body {
                Body::Instance { name: held, .. } => *held = Some(name.as_slice().into()),
                other => unreachable!("an instance receiver is Body::Instance, got {other:?}"),
            }
        }
        Primitive::Package
        | Primitive::Method
        | Primitive::Routine
        | Primitive::Directory
        | Primitive::StringTable(_)
        | Primitive::Context
        | Primitive::RexxInfo
        | Primitive::Message => {
            let Some(object) = interp.heap.get_mut(receiver) else {
                return Err(Loud::receiver_class("a value whose object is no longer live").into());
            };
            match &mut object.body {
                Body::Native(native) => native.set_rendered(&name),
                other => {
                    unreachable!("each of these receivers is Body::Native, got {other:?}")
                }
            }
        }
        Primitive::String
        | Primitive::SmallInt
        | Primitive::Object
        | Primitive::Array
        | Primitive::VariableReference
        | Primitive::Stem => {
            return Err(Loud::native_method(b"OBJECTNAME=", "Object").into());
        }
    }
    Ok(None)
}

/// `Object~setMethod(name, method, scope)`: attach one method to this object
/// alone -- `RexxObject::setMethod` (`classes/ObjectClass.cpp:1829`).
///
/// **The order of the steps is observable**, and it is the C++'s: the name,
/// then the method object, then the scope option, then the restricted check,
/// then the definition. Measured, oracle rc 168 from a class method of an
/// unrelated class: `self~setMethod('MM', 'return 1', 'BOGUS')` reports the
/// option's 88.916 and not the restricted check's 98.991, so the option is
/// read first.
///
/// **A second argument that is omitted hides the name**, storing the
/// oracle's `.nil` rather than removing anything: measured, oracle rc 159,
/// after `self~setMethod('MM')` on a class that defines `MM`,
/// `hasMethod('MM')` is `0` and the send is 97.1.
///
/// The third argument is D67 and [`rexx_core::ObjectMethod`] carries which
/// pool each of its values selects.
fn native_set_method(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let name = method_name_argument(interp, args)?;
    let source = match args.get(1).copied().flatten() {
        None => None,
        Some(source) if interp.receiver_kind(source) == Ok(Primitive::Method) => Some(source),
        Some(source) => Some(compile_method_source(interp, &name, source, "method")?),
    };
    let scope = set_method_scope(interp, receiver, args)?;
    check_restricted_method(interp, receiver, b"SETMETHOD")?;
    let entry = match source {
        None => None,
        Some(object) => {
            let Some(body) = interp.table_method_bodies.get(&object).copied() else {
                return Err(Loud::method_from_source(
                    "a one-off method whose body this crate does not hold",
                )
                .into());
            };
            let method = interp.classes().mint_method_id();
            interp.method_bodies.insert(method, body);
            Some(ObjectMethod { method, scope })
        }
    };
    interp.write_object_method(receiver, &name, ObjectMethodWrite::Set(entry))?;
    Ok(None)
}

/// `Object~unsetMethod(name)`: take back a `setMethod` definition --
/// `RexxObject::unsetMethod` (`classes/ObjectClass.cpp:1891`).
///
/// **A name this object never set is untouched**, and that is the whole
/// difference from `Class~delete`: measured, oracle rc 0,
/// `self~unsetMethod('MM')` for a class-defined `MM` leaves `o~mm` answering
/// the class's, and `self~unsetMethod('ZZZ')` for a name nothing defines is
/// not an error either.
fn native_unset_method(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let name = method_name_argument(interp, args)?;
    check_restricted_method(interp, receiver, b"UNSETMETHOD")?;
    interp.write_object_method(receiver, &name, ObjectMethodWrite::Remove)?;
    Ok(None)
}

/// `setMethod`'s third argument: which variable pool the new method's
/// `EXPOSE` reaches (D67).
///
/// `FLOAT`, the default, is `TheNilObject` -- one pool per object, shared by
/// all of that object's `FLOAT` methods and separate from the class's.
/// `OBJECT` is `classObject()`, the object's own class, so the pool is the
/// one the class's own methods use. Measured, oracle rc 0: two `FLOAT`
/// methods on one object read each other's writes while a second instance of
/// the same class sees the name uninitialised, and an `OBJECT` method's write
/// is what a `::METHOD` of the class reads back.
///
/// Anything else is 88.916 -- measured, oracle rc 168, `Argument 3 must be
/// one of "FLOAT" or "OBJECT"; found "BOGUS".`
fn set_method_scope(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let Some(Some(option)) = args.get(2).copied() else {
        return Ok(ObjRef::NIL);
    };
    let option = required_string_named_argument(interp, option, "scope option")?;
    let text = interp.to_text(option).to_vec();
    if text.eq_ignore_ascii_case(b"OBJECT") {
        return interp
            .class_of_value(receiver)
            .ok_or_else(|| Loud::object_method("a receiver with no class of its own").into());
    }
    if text.eq_ignore_ascii_case(b"FLOAT") {
        return Ok(ObjRef::NIL);
    }
    Err(Raised::not_one_of(3, b"\"FLOAT\" or \"OBJECT\"", &text).into())
}

/// `RexxObject::checkRestrictedMethod` (`classes/ObjectClass.cpp:697`), the
/// fourth access check (D66): `run`, `setMethod` and `unsetMethod` may be
/// sent only from a method of the receiving object itself or from a class
/// method of a class it is an instance of.
///
/// **It is not the private check and it has its own error.** The private
/// check runs first, at dispatch, and refuses a program context with `97.2
/// ... cannot accept private message` at rc 159 and no method frame; this
/// one refuses with `98.991 ... may only be invoked from a method of the
/// same object or one of its classes.` at rc 158, under a `Compiled method
/// "SETMETHOD" with scope "Object".` frame. Both measured, and the frame is
/// what tells them apart. The allowing arm is measured too: a class method
/// of the object's own class may send it, oracle rc 0.
///
/// The caller's *receiver* is the input, which is what makes this a check no
/// value of [`Access`] can express.
fn check_restricted_method(
    interp: &mut Interp,
    receiver: ObjRef,
    name: &[u8],
) -> Result<(), Failure> {
    let caller = interp.caller();
    // `sender == OREF_NULL` at `:715`-`:719`, a routine or program context,
    // which the private check refuses first: `run`, `setMethod` and
    // `unsetMethod` are each private, so `check_private`'s own
    // `caller.receiver()` arm has already answered for a caller with no
    // receiver.
    let Some(sender) = caller.receiver() else {
        return Err(Raised::restricted_method(name).into());
    };
    if sender == receiver {
        return Ok(());
    }
    // `isOfClassType(Class, sender)` then `isInstanceOf((RexxClass *)sender)`
    // (`:722`-`:729`): a class method of a class this object is an instance
    // of.
    if interp.is_class_object(sender)
        && let Some(class) = interp.class_of_value(receiver)
        && interp.classes().is_a(class, sender)
    {
        return Ok(());
    }
    Err(Raised::restricted_method(name).into())
}

/// `Object~copy`: a new object with the receiver's methods and an equivalent
/// set of object variables holding the same values -- `RexxObject::copyRexx`
/// (`classes/ObjectClass.cpp:2879`).
///
/// **Shallow, and the write-through is what makes that observable.**
/// Measured, oracle rc 0: `(o~identityHash == c~identityHash)` is `0`, the
/// copy reads the receiver's values back, and after `c~set('changed')` the
/// receiver's `~get` still answers `orig` where the copy's answers `changed`.
/// The objects the pools hold are shared and not copied.
///
/// The object's own `setMethod` and `Class~enhanced` dictionary travels with
/// it -- measured, oracle rc 0, a one-off set on the receiver answers on the
/// copy -- and so does the receiver's `UNINIT` registration: measured, one
/// instance with a finalizer, copied once and both dropped, prints
/// `uninit ran` twice.
///
/// A receiver with no scope of its own is loud rather than answered. A class
/// object never reaches here -- `memory/Setup.cpp:483` overrides this row and
/// [`native_class_copy`] is the refusal it installs.
fn native_copy(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // **An `Array` and a `Stem` are copyable too**, and each keeps its own
    // behaviour: measured, `.Array~of('x','y')~copy~class~id` is `Array` and a
    // stem's is `Stem`. The clone below is what makes both deep in the way
    // upstream's virtual `copy()` is -- the slots and the tails travel with
    // the body, so appending to the copy leaves the receiver alone
    // (`a~items` 2 against `b~items` 3) and a tail written on the copy does
    // not appear on the receiver.
    let behaviour = match interp.receiver_kind(receiver) {
        Ok(Primitive::Instance { .. }) => rexx_core::BehaviourId::OBJECT,
        Ok(Primitive::Array) => rexx_core::BehaviourId::ARRAY,
        Ok(Primitive::Stem) => rexx_core::BehaviourId::STEM,
        Ok(_) => return Err(Loud::native_method(b"COPY", "Object").into()),
        Err(kind) => return Err(Loud::receiver_class(kind).into()),
    };
    // The allocation below collects first, and while the cloned body is a
    // local the collector does not walk, every value in it is reachable from
    // the receiver and from nowhere else.
    interp.roots.push_temp(receiver);
    let Some(source) = interp.heap.get(receiver) else {
        return Err(Loud::receiver_class("a value whose object is no longer live").into());
    };
    let body = source.body.clone();
    let copy = interp.alloc_with(behaviour, body);
    interp.roots.push_temp(copy);
    duplicate_collection_stores(interp, copy);
    if interp.answers_uninit(copy) {
        interp.heap.set_uninit(copy);
    }
    Ok(Some(copy))
}

/// The pool entries a collection's CONTENTS live in, as (scope class, name).
///
/// A `Body::Instance` clone copies the pool map, so the copy's entries name
/// the same `Array` objects the receiver's do -- which is right for an
/// ordinary instance variable and wrong for a collection's contents.
/// Upstream draws the same line by overriding the virtual `copy()`:
/// `HashCollection::copy` copies the base object and then its contents
/// (`classes/support/HashCollection.cpp:237`), and `ArrayClass` and
/// `ListClass` do the same for theirs.
///
/// Measured before this existed: `q = .Queue~of('a','b')`, `c = q~copy`,
/// `c~queue('z')` left **both** at 3 items where the oracle answers 2 and 3,
/// and the same for `List`. It is why `Set~union` -- which is `self~copy` and
/// then a loop of `put` (`RexxClasses/CoreClasses.orx:480`) -- was mutating
/// its receiver, which is how this was found.
const COLLECTION_STORES: &[(&str, &[u8])] = &[
    ("Array", b"ITEMS"),
    ("List", b"ITEMS"),
    ("List", b"HANDLES"),
    ("List", b"FREE"),
    ("Supplier", b"ITEMS"),
    ("Supplier", b"INDEXES"),
    ("Table", b"HASHINDEXES"),
    ("Table", b"HASHITEMS"),
    ("Table", b"HASHNEXT"),
    // A `Directory`'s method table travels with the copy and is its own --
    // `DirectoryClass::copy` copies it rather than sharing it. Measured:
    // `g = f~copy` then `g~unsetMethod('M')` leaves `f['M']` answering.
    ("Table", b"METHODINDEXES"),
    ("Table", b"METHODITEMS"),
    ("Table", b"METHODNEXT"),
];

/// Replaces each of [`COLLECTION_STORES`] on `copy` with an array of its own.
///
/// The slots are copied across as they stand: the ITEMS are shared with the
/// receiver's, which is what upstream's shallow element copy does, and only
/// the array holding them is new.
fn duplicate_collection_stores(interp: &mut Interp, copy: ObjRef) {
    for (scope_name, entry) in COLLECTION_STORES {
        let Some(scope) = interp.classes().lookup(scope_name) else {
            continue;
        };
        let held = match interp.heap.get(copy).map(|object| &object.body) {
            Some(Body::Instance { pools, .. }) => pools.get(scope, entry),
            _ => return,
        };
        let Some(held) = held else { continue };
        let Some(slots) = interp.array_slots_of(held) else {
            continue;
        };
        let fresh = interp.alloc_with(BehaviourId::ARRAY, Body::array(slots));
        interp.roots.push_temp(fresh);
        interp.set_pool_variable(copy, scope, entry, fresh);
    }
}

/// `Class~copy`: the refusal `memory/Setup.cpp:483` puts over [`native_copy`]
/// -- `RexxClass::copyRexx` (`classes/ClassClass.cpp:166`), whose whole body
/// is the raise.
///
/// Measured, oracle rc 163: `.K~copy` is `93.970 COPY method is not supported
/// for object The K class.` under a `Compiled method "COPY" with scope
/// "Class".` frame.
fn native_class_copy(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let target = interp.string_value_text(receiver);
    Err(Raised::copy_not_supported(&target).into())
}

/// `Object~run(method [, option [, argument ...]])`: run a method on this
/// object as if the object had defined it -- `RexxObject::run`
/// (`classes/ObjectClass.cpp:2185`).
///
/// **The body's `EXPOSE` reaches the object's `FLOAT` pool** (D67), because
/// the method is built with `TheNilObject` for its scope (`:2201`). Measured,
/// oracle: `self~run('expose v; return v*10')` is `41.1` at rc 215 where the
/// class's own `v` is 7; a `FLOAT` one-off's write is what a run body reads
/// back; and two run bodies on one object read each other's writes.
///
/// **The steps are in the C++'s order and the order is observable**: the
/// method, then the option, then the restricted check. Measured, oracle rc
/// 163 from a class method of an unrelated class,
/// `o~run('return 1', 'BOGUS')` reports the option's 93.915 and not the
/// restricted check's 98.991.
///
/// **The body is named `RUN` and not for the file.** Measured, oracle rc 0,
/// `parse source` inside one answers `LINUX METHOD RUN`; and rc 214, a
/// failure inside one reports `Error 42 running RUN line 1` with no
/// `Compiled method` line of its own above the native `RUN` frame.
fn native_run(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(source)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("method").into());
    };
    let body = run_method_body(interp, source)?;
    let values = run_arguments(interp, args)?;
    check_restricted_method(interp, receiver, b"RUN")?;
    let method = interp.classes().mint_method_id();
    interp.method_bodies.insert(method, body);
    let resolution = Resolution {
        scope: ObjRef::NIL,
        method,
    };
    interp.invoke(resolution, receiver, UNNAMED_METHOD, &values)
}

/// `~run`'s first argument as a body this crate can enter --
/// `MethodClass::newMethodObject(GlobalNames::RUN, methobj, TheNilObject,
/// "method")` (`classes/ObjectClass.cpp:2201`).
///
/// A source string and an `Array` of source strings are
/// [`compile_method_source`]'s two shapes and are compiled here. A `Method`
/// object is taken when this crate holds its body, which is every object
/// `compile_method_source` built; one that came from `Class~method` names a
/// dictionary entry and carries no body a send could enter, so it is loud
/// rather than run under the wrong one.
///
/// The scope every arm runs at is `ObjRef::NIL`, which a `Method` object does
/// not override: measured, oracle rc 0, `self~run(.K~method('PROBE'))` on a
/// `PROBE` that exposes `v` answers the derived name `V` where the class's
/// own pool holds `class-pool`.
fn run_method_body(
    interp: &mut Interp,
    source: ObjRef,
) -> Result<crate::InstalledMethodBody, Failure> {
    let object = if interp.receiver_kind(source) == Ok(Primitive::Method) {
        source
    } else {
        compile_method_source(interp, b"RUN", source, "method")?
    };
    interp
        .table_method_bodies
        .get(&object)
        .copied()
        .ok_or_else(|| {
            Loud::method_from_source("a one-off method whose body this crate does not hold").into()
        })
}

/// `~run`'s `Individual`/`Array` option and the arguments behind it
/// (`classes/ObjectClass.cpp:2207`-`:2235`).
///
/// A send with no second argument passes none, which is the option being
/// absent rather than empty: measured, oracle rc 0, `self~run('return
/// "no-opt"')` answers, while `self~run('...', , 5)` is `88.901 Missing
/// argument; argument argument style is required.` at rc 168 and
/// `self~run('...', '', 5)` is `93.915 ... found "".` at rc 163.
///
/// Only the first letter is read and the rest ignored -- measured, oracle rc
/// 0, `'ignored-after-first'` passes its arguments individually.
fn run_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<Vec<Option<ObjRef>>, Failure> {
    let Some(option) = args.get(1).copied() else {
        return Ok(Vec::new());
    };
    let Some(option) = option else {
        return Err(Raised::missing_named_argument("argument style").into());
    };
    let option = required_string_named_argument(interp, option, "argument style")?;
    let text = interp.to_text(option).to_vec();
    match text.first().map(u8::to_ascii_uppercase) {
        Some(b'I') => Ok(args[2..].to_vec()),
        // `argCount < 3` and `argCount > 3` are separate raises with separate
        // numbers. Measured, oracle: `self~run('return 1', 'A')` is `88.901
        // ... argument argument array is required.` at rc 168, and a fourth
        // argument is `93.902 ... 3 expected.` at rc 163.
        Some(b'A') => {
            let Some(Some(array)) = args.get(2).copied() else {
                return Err(Raised::missing_named_argument("argument array").into());
            };
            if args.len() > 3 {
                return Err(Raised::too_many_method_arguments(3).into());
            }
            array_argument(interp, array, ArrayArgument::Named("argument array"))
        }
        _ => Err(Raised::method_option_not_recognised("AI", &text).into()),
    }
}

/// `Object~send(messagename [, argument ...])`: invoke a method on this
/// object under a name built at run time -- `RexxObject::send`
/// (`classes/ObjectClass.cpp:2008`).
///
/// The name may be an array whose first item is the message and whose second
/// is the class to start the method search from, which is the dynamic form of
/// `receiver~name:scope`. Measured, oracle rc 0: with `::class Sub subclass
/// Base` both defining `M`, `o~send('M')` answers the subclass's and
/// `o~send(('M', .Base))` the base's.
fn native_send(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (name, scope) = decode_message_name(interp, args.first().copied().flatten())?;
    dynamic_send(interp, receiver, &name, scope, &args[1..])
}

/// `Object~sendWith(messagename, arguments)`: [`native_send`] with the
/// arguments in an array -- `RexxObject::sendWith`
/// (`classes/ObjectClass.cpp:1972`).
///
/// **The name is decoded before the array is read**, which is the C++'s order
/// and is observable: measured, oracle rc 163, `o~sendWith(.nil)` is the
/// name's own 93.972 where the same call to `~startWith` reports the missing
/// second argument instead.
fn native_send_with(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (name, scope) = decode_message_name(interp, args.first().copied().flatten())?;
    let values = message_arguments(interp, args.get(1).copied().flatten())?;
    dynamic_send(interp, receiver, &name, scope, &values)
}

/// `Object~start(messagename [, argument ...])`: a `Message` object whose
/// send has been made -- `RexxObject::start`
/// (`classes/ObjectClass.cpp:2067`) through `startCommon` (`:2094`).
///
/// **The send runs before this answers**, where the oracle runs it on an
/// activity of its own, and nothing a check may assert separates the two
/// (D68): `~start`'s interleaving with the program that started it is not
/// reproducible on the oracle, and `~completed` sampled *before* `~result` is
/// part of that interleaving. What is reproducible, and what the corpus
/// asserts, is `~result`'s value and `~completed` after it -- measured,
/// oracle rc 0, `result ran 5` / `completed 1` / `haserror 0`.
///
/// **Phase 6 owes the scheduling.** Two consequences of running the send
/// here, both measured against the oracle and neither reachable by a check
/// this phase may write: a started method's own output lands before the
/// starting program's next clause rather than interleaved with it, and a
/// started method that raises reports its failure once, at `~result`, where
/// the oracle writes the same report twice in a transcript that does not
/// reproduce -- four runs of one program, two orderings.
///
/// A `Message` row with no [`NativeMethod`] behind it refuses loudly, so a
/// message this crate cannot drive never answers as though it could.
fn native_start(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(message) = args.first().copied().flatten() else {
        return Err(Raised::missing_named_argument("message name").into());
    };
    started_message(interp, receiver, message, &args[1..])
}

/// `Object~startWith(messagename, arguments)`: [`native_start`] with the
/// arguments in an array -- `RexxObject::startWith`
/// (`classes/ObjectClass.cpp:2046`).
///
/// **The array is read before the name is decoded**, the opposite of
/// [`native_send_with`], because `startWith` checks the message is merely
/// present and leaves the decoding to `startCommon` (`:2049`-`:2053`).
/// Measured, oracle rc 163, one call shape and two catalogue rows:
/// `o~startWith(.nil)` is `93.903 Missing argument in method; argument 2 is
/// required.` where `o~sendWith(.nil)` is the name's own 93.972.
fn native_start_with(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(message) = args.first().copied().flatten() else {
        return Err(Raised::missing_named_argument("message name").into());
    };
    let Some(arguments) = args.get(1).copied().flatten() else {
        return Err(Raised::missing_method_argument(2).into());
    };
    let values = array_argument(interp, arguments, ArrayArgument::Positional)?;
    started_message(interp, receiver, message, &values)
}

/// The send `~send` and `~sendWith` make, once the name has been decoded.
///
/// The scope override is validated before the send, which is where
/// `RexxObject::sendWith` puts it (`:1989`) and what makes an unrelated class
/// 93.957 rather than 97.1: measured, oracle rc 163, `o~send(('M', .Array))`
/// is `Target object "a K" is not a subclass of the message override scope
/// (The Array class).`
fn dynamic_send(
    interp: &mut Interp,
    receiver: ObjRef,
    name: &[u8],
    scope: Option<ObjRef>,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    interp.validate_scope_override(receiver, scope)?;
    let caller = interp.caller();
    interp.send_message(receiver, name, scope, args, caller)
}

/// The `Message` object `~start` and `~startWith` answer, with its send
/// already made -- `RexxObject::startCommon` (`classes/ObjectClass.cpp:2094`)
/// and `MessageClass::dispatch` (`classes/MessageClass.cpp:421`).
///
/// **Only a raised condition is caught.** `MessageClass::error` is handed a
/// condition object by the activation notifying it, so what a message records
/// is a Rexx condition and nothing else: a [`Loud`] is this crate saying it
/// cannot run something and has to reach the program rather than become a
/// message's `~hasError`, and `Failure::Exited` is not a failure at all.
fn started_message(
    interp: &mut Interp,
    receiver: ObjRef,
    message: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (name, scope) = decode_message_name(interp, Some(message))?;
    interp.validate_scope_override(receiver, scope)?;
    let class = interp.object_model().message;
    let object = interp.native_instance(class);
    let caller = interp.caller();
    let outcome = match interp.send_message(receiver, &name, scope, args, caller) {
        Ok(None) => None,
        Ok(Some(value)) => {
            interp.set_native_entry(object, MESSAGE_RESULT, value);
            None
        }
        Err(Failure::Raised(raised)) => Some(raised),
        Err(other) => return Err(other),
    };
    interp.message_outcomes.insert(object, outcome);
    Ok(Some(object))
}

/// `Message~result`: the value the send answered, `.nil` for one that
/// answered none, and the send's own condition raised again where it failed
/// -- `MessageClass::result` (`classes/MessageClass.cpp:279`).
///
/// Measured, oracle rc 0: `m~result` after `o~start('M', 5)` is the method's
/// value, and after a method ending in a bare `return` it is
/// `The NIL object`.
fn native_message_result(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    match interp.message_outcomes.get(&receiver) {
        Some(Some(raised)) => Err(Failure::Raised(raised.clone())),
        Some(None) => {
            let held = interp.native_entry(receiver, MESSAGE_RESULT);
            Ok(Some(held.unwrap_or(ObjRef::NIL)))
        }
        // A `Message~new` object, whose send has not been made.
        None => Err(Loud::unsent_message_result().into()),
    }
}

/// `Message~completed`: whether the send has ended, with a result or with an
/// error -- `MessageClass::completed` (`classes/MessageClass.cpp:722`), which
/// is `resultReturned() || raiseError()`.
///
/// `setResultReturned()` runs after the send whether or not it produced a
/// value (`:446`), so a method ending in a bare `return` completes like any
/// other. **Sampling this before `~result` is racy on the oracle**, which is
/// what D68 puts out of reach of every check.
fn native_message_completed(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let completed = interp.message_outcomes.contains_key(&receiver);
    Ok(Some(interp.counted(usize::from(completed))))
}

/// `Message~hasError`: whether the send ended by raising --
/// `MessageClass::hasError` (`classes/MessageClass.cpp:736`).
fn native_message_has_error(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let failed = matches!(interp.message_outcomes.get(&receiver), Some(Some(_)));
    Ok(Some(interp.counted(usize::from(failed))))
}

/// `RexxObject::decodeMessageName` (`classes/ObjectClass.cpp:2125`): a
/// message name, or a two-item array of a name and the class to start the
/// method search from.
///
/// The name is upcased, so `o~send('m')` reaches `::method M` -- measured,
/// oracle rc 0. Each refusal below differs from its neighbours in exit status
/// or catalogue row, and `corpus/lang/object_send_refusals.rex` is where they
/// are asserted rather than listed.
fn decode_message_name(
    interp: &mut Interp,
    message: Option<ObjRef>,
) -> Result<(Vec<u8>, Option<ObjRef>), Failure> {
    let Some(message) = message else {
        return Err(Raised::missing_named_argument("message name").into());
    };
    // `isString(message)` first, so a value that merely has a string value --
    // `.nil` is the reachable one -- takes the array path and its own 93.972
    // rather than being converted.
    if matches!(
        interp.receiver_kind(message),
        Ok(Primitive::String | Primitive::SmallInt)
    ) {
        return Ok((interp.to_text(message).to_ascii_uppercase(), None));
    }
    let Some(slots) = interp.array_slots_of(message) else {
        let rendered = interp.string_value_text(message);
        return Err(Raised::message_name_shape(&rendered).into());
    };
    // `messageArgCount() != 2` (`classes/ObjectClass.cpp:2143`), which is
    // `lastItem` (`classes/ArrayClass.hpp:305`) and not the slot count: a
    // trailing empty slot is not an element. Measured, oracle rc 163,
    // `o~send(('M',))` is 93.946 where the same array is `~size` 2.
    let slots = &slots[..message_argument_count(&slots)];
    if slots.len() != 2 {
        return Err(Raised::message_array_shape().into());
    }
    // `stringArgument` distinguishes an empty slot from a value with no
    // string value. Measured, oracle: `o~send((, .K))` is `88.901 ...
    // argument message name is required.` and `o~send((.nil, .K))` is
    // `88.909 Argument message name must have a string value.`, both rc 168.
    let Some(name) = slots[0] else {
        return Err(Raised::missing_named_argument("message name").into());
    };
    let name = required_string_named_argument(interp, name, "message name")?;
    let name = interp.to_text(name).to_ascii_uppercase();
    let scope = slots[1].filter(|scope| interp.is_class_object(*scope));
    if scope.is_none() {
        return Err(Raised::argument_not_a_class("SCOPE").into());
    }
    Ok((name, scope))
}

/// `ArrayClass::messageArgCount` (`classes/ArrayClass.hpp:305`), which is the
/// array's `lastItem`: the position of the last filled slot, so a trailing
/// empty slot is not an argument and an interior one is an omitted argument.
///
/// Measured, oracle rc 0, over `~sendWith` into a method reporting `arg()`:
/// `(5,)` passes `1`, `(5, , 7)` passes `3` with the second omitted, and
/// `(, 6)` passes `2` with the first omitted.
fn message_argument_count(slots: &[Option<ObjRef>]) -> usize {
    slots
        .iter()
        .rposition(Option::is_some)
        .map_or(0, |at| at + 1)
}

/// [`message_argument_count`] applied to an argument array, which is what
/// `messageArgs()`/`messageArgCount()` hand a send together.
fn message_argument_slots(slots: Vec<Option<ObjRef>>) -> Vec<Option<ObjRef>> {
    let count = message_argument_count(&slots);
    let mut slots = slots;
    slots.truncate(count);
    slots
}

/// The argument array `~sendWith` requires -- `arrayArgument(args, "message
/// arguments")` (`classes/ObjectClass.cpp:1980`).
fn message_arguments(
    interp: &mut Interp,
    arguments: Option<ObjRef>,
) -> Result<Vec<Option<ObjRef>>, Failure> {
    let Some(arguments) = arguments else {
        return Err(Raised::missing_named_argument("message arguments").into());
    };
    array_argument(interp, arguments, ArrayArgument::Named("message arguments"))
}

/// Which `arrayArgument` overload a site calls, which is what decides the
/// refusal a multi-dimensional array takes.
enum ArrayArgument {
    /// `arrayArgument(object, const char *name)`
    /// (`runtime/MethodArguments.hpp:703`), whose raise names the argument.
    Named(&'static str),
    /// `arrayArgument(object, size_t position)`
    /// (`runtime/MethodArguments.hpp:675`), whose raise renders the object.
    Positional,
}

/// `arrayArgument`: an argument's message-argument slots.
///
/// A value that is not already an `Array` takes
/// [`unconverted_array_argument`]'s refusal, for the reason `~UNKNOWN`'s own
/// argument list takes it: the C++ converts with `requestArray`, which is a
/// `MAKEARRAY` send this crate answers for no receiver.
///
/// Both overloads then test that answer for `isMultiDimensional`, and they
/// differ in the raise that carries. Measured, oracle:
/// `o~sendWith('M', .array~new(2,2))` is `88.913 Argument message arguments
/// must be a single-dimensional array.` at rc 168, and
/// `o~startWith('M', .array~new(2,2))` is `98.913 Unable to convert object
/// "an Array" to a single-dimensional array value.` at rc 158.
fn array_argument(
    interp: &mut Interp,
    value: ObjRef,
    overload: ArrayArgument,
) -> Result<Vec<Option<ObjRef>>, Failure> {
    let value = request_array(interp, value)?;
    let Some(slots) = interp.array_slots_of(value) else {
        return Err(unconverted_array_argument(interp, value));
    };
    if interp.is_multi_dimensional_array(value) {
        return Err(match overload {
            ArrayArgument::Named(argument) => {
                Raised::argument_not_single_dimensional(argument).into()
            }
            ArrayArgument::Positional => {
                let found = interp.string_value_text(value);
                Raised::object_not_single_dimensional(&found).into()
            }
        });
    }
    Ok(message_argument_slots(slots))
}

/// `Class~enhanced(methods, ...)`: an instance of the receiver carrying
/// methods of its own -- `RexxClass::enhanced`
/// (`classes/ClassClass.cpp:1440`).
///
/// **The enhancing methods are in place before `INIT` runs**, which is
/// observable and is why this builds the object rather than sending `~new`:
/// the C++ puts them in a dummy subclass's behaviour and then sends `NEW` to
/// that subclass (`:1454`-`:1470`). Measured, oracle rc 0: with `INIT` in the
/// table, the enhancing `INIT` runs and the class's own does not, and the
/// arguments after the table are its arguments.
///
/// **The dummy subclass is invisible.** `setOwningClass(this)` (`:1474`)
/// puts the object's class back to the receiver, so `~class~id` and `~isA`
/// answer for the receiver -- measured, `K` and `1`.
///
/// **The methods are a level of their own, not `setMethod`'s.** They are
/// added with `.nil` scope, "so that these additional methods will look like
/// they were added with setMethod" (`:1454`-`:1455`), which is D67's pool selection
/// and is measured: an enhancing method and a `FLOAT` one-off on the same
/// object read each other's `EXPOSE`d names, oracle rc 0. The oracle keeps
/// them in the dummy subclass's behaviour, where `unsetMethod` cannot reach
/// them, and [`ObjectMethods`] is where that level lives here.
fn native_enhanced(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    // The refusals are different errors and the C++ checks them in this
    // order (`:1443`-`:1451`). Measured: `.K~enhanced` and `.K~enhanced()`
    // are both `93.901 Not enough arguments for method; 1 expected.` at rc
    // 163, the second because a trailing omission is dropped from the count,
    // while `.K~enhanced(, 'x')` is `88.901 Missing argument; argument
    // methods is required.` at rc 168.
    if args.is_empty() {
        return Err(Raised::not_enough_method_arguments(1).into());
    }
    let Some(table) = args[0] else {
        return Err(Raised::missing_named_argument("methods").into());
    };
    if !string_keyed_table(interp, table) {
        return Err(supplier_refusal(interp, table));
    }
    if let Some(owner) = interp.unbuilt_collection_owner(table) {
        return Err(Loud::unreadable_collection(owner).into());
    }
    let mut names = interp.native_keys(table);
    names.sort();
    let object = new_instance(interp, class)?;
    let frame = interp.roots.push_frame();
    let installed = install_enhancing_object_methods(interp, object, table, &names);
    interp.roots.pop_frame(frame);
    installed?;
    // `dummy_subclass->sendMessage(GlobalNames::NEW, args + 1, ...)`
    // (`:1470`), whose `INIT` send is `completeNewObject`'s -- the enhancing
    // `INIT` by then, since it is already in the behaviour.
    let caller = interp.caller();
    interp.send_message(object, INIT, None, &args[1..], caller)?;
    Ok(Some(object))
}

/// [`native_enhanced`]'s walk, split out so the root frame it runs inside is
/// released on the failure path too --
/// [`install_enhancing_class_methods`]'s shape.
fn install_enhancing_object_methods(
    interp: &mut Interp,
    object: ObjRef,
    table: ObjRef,
    names: &[Box<[u8]>],
) -> Result<(), Failure> {
    for name in names {
        let value = interp.native_entry(table, name).unwrap_or(ObjRef::NIL);
        let source = if interp.receiver_kind(value) == Ok(Primitive::Method) {
            value
        } else {
            compile_method_source(interp, name, value, "method source")?
        };
        let Some(body) = interp.table_method_bodies.get(&source).copied() else {
            return Err(Loud::method_from_source(
                "an enhancing method whose body this crate does not hold",
            )
            .into());
        };
        let method = interp.classes().mint_method_id();
        interp.method_bodies.insert(method, body);
        interp.write_object_method(
            object,
            name,
            ObjectMethodWrite::Enhance(ObjectMethod {
                method,
                scope: ObjRef::NIL,
            }),
        )?;
    }
    Ok(())
}

/// `StringTable~new` and `Directory~new`: an empty hash collection --
/// `StringTable::newRexx` and `DirectoryClass::newRexx`
/// (`memory/Setup.cpp:875`, `:928`), each of which allocates and then sends
/// `INIT` with the whole argument list.
///
/// The `INIT` send is what validates the initial-size argument, and it is a
/// send rather than a check here so the traceback carries both frames --
/// measured, oracle rc 163 for `.stringtable~new('abc')`: `Compiled method
/// "INIT" with scope "StringTable".` above `Compiled method "NEW" with scope
/// "StringTable".`
fn native_hash_collection_new(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // **A `Directory` a program makes is a collection, not the environment.**
    // `NativeObject`'s map holds its keys already uppercased, by its callers,
    // and a `.Directory~new` does not: measured, `d['lower'] = 1` leaves
    // `allIndexes` reading `lower` and `d['LOWER']` answering `.nil`. So that
    // map is what `.environment` and `.local` are built on -- they are
    // `native_instance`s the bootstrap makes and keeps -- and a collection
    // gets the object-keyed store with a string-key protocol on top.
    hash::native_hash_new(interp, cleared, receiver, args)
}

/// `Directory~new`: the hash body above for `.Directory` itself, and
/// [`native_new`]'s plain instance for a subclass.
///
/// **The split is what a `Body::Native` has not got**: a variable pool and an
/// `UNINIT` registration, each of which a `Directory` subclass can reach and
/// the oracle answers. What the split costs is the subclass's entry writes,
/// which refuse. `a_directory_subclass_keeps_the_instance` is the pair.
fn native_directory_new(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    if class == interp.object_model().directory {
        native_hash_collection_new(interp, cleared, receiver, args)
    } else {
        native_new(interp, cleared, receiver, args)
    }
}

/// A collection's `~init(size)`: the initial-size argument, validated and
/// then dropped -- `HashCollection::initRexx`
/// (`classes/support/HashCollection.cpp:63`), and the `QueueClass::initRexx`
/// and `ListClass::initRexx` beside it (`classes/QueueClass.cpp:261`,
/// `classes/ListClass.cpp:102`), which differ only in the default capacity.
///
/// The size is a capacity hint and nothing observable depends on it, so it
/// is checked and not kept. Measured, oracle rc 163:
/// `.stringtable~new('abc')` is `93.923 Invalid length argument specified;
/// found "abc".`
fn native_capacity_init(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    optional_length_argument(interp, args, 0)?;
    Ok(None)
}

/// `optionalLengthArgument` (`runtime/MethodArguments.hpp:327`): `None` for
/// an omitted argument, and anything that is not a non-negative whole number
/// in range is 93.923.
fn optional_length_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
) -> Result<Option<usize>, Failure> {
    match whole_method_argument(interp, args, index, Raised::invalid_length)? {
        Some(size) if size >= 0 => Ok(Some(usize_or_refuse(
            interp,
            args,
            index,
            size,
            Raised::invalid_length,
        )?)),
        Some(_) => Err(refuse_method_argument(
            interp,
            args,
            index,
            Raised::invalid_length,
        )),
        None => Ok(None),
    }
}

/// `lengthArgument` (`runtime/MethodArguments.hpp`): [`optional_length_argument`]
/// with an omitted argument 93.903 -- measured, oracle rc 163:
/// `.MutableBuffer~new('abc')~setBufferSize` reports `argument 1 is required`.
fn required_length_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
) -> Result<usize, Failure> {
    match optional_length_argument(interp, args, index)? {
        Some(size) => Ok(size),
        None => Err(Raised::missing_method_argument(index + 1).into()),
    }
}

/// `optionalPositionArgument` (`runtime/MethodArguments.hpp:387`): `None` for
/// an omitted argument, and anything that is not a positive whole number in
/// range is 93.924.
fn optional_position_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
) -> Result<Option<usize>, Failure> {
    match whole_method_argument(interp, args, index, Raised::invalid_position)? {
        Some(position) if position > 0 => Ok(Some(usize_or_refuse(
            interp,
            args,
            index,
            position,
            Raised::invalid_position,
        )?)),
        Some(_) => Err(refuse_method_argument(
            interp,
            args,
            index,
            Raised::invalid_position,
        )),
        None => Ok(None),
    }
}

/// `positionArgument` (`runtime/MethodArguments.hpp`): [`optional_position_argument`]
/// with an omitted argument 93.903 -- measured, oracle rc 163:
/// `.MutableBuffer~new('abcabc')~substr` reports `argument 1 is required`.
fn required_position_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
) -> Result<usize, Failure> {
    match optional_position_argument(interp, args, index)? {
        Some(position) => Ok(position),
        None => Err(Raised::missing_method_argument(index + 1).into()),
    }
}

/// `stringArgument` by position (`runtime/MethodArguments.hpp:136`), the
/// bytes copied into the lent result buffer: omitted is 93.903 and a value
/// without a string value is 88.909.
fn string_method_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
) -> Result<Vec<u8>, Failure> {
    let Some(value) = args.get(index).copied().flatten() else {
        return Err(Raised::missing_method_argument(index + 1).into());
    };
    let text = required_string_argument(interp, value, index + 1)?;
    let mut bytes = interp.take_result_buffer();
    bytes.extend_from_slice(&interp.to_text(text));
    Ok(bytes)
}

/// `optionalPadArgument` (`classes/StringClassUtil.cpp:261`): `None` for an
/// omitted argument, 88.909 for one without a string value, and 93.922 for a
/// string that is not one byte -- measured, oracle rc 163:
/// `.MutableBuffer~new('abcabc')~substr(1, 2, 'xx')` reports `found "xx"`.
fn pad_method_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
) -> Result<Option<u8>, Failure> {
    let Some(value) = args.get(index).copied().flatten() else {
        return Ok(None);
    };
    let text = required_string_argument(interp, value, index + 1)?;
    let pad = match interp.to_text(text).as_ref() {
        [byte] => Some(*byte),
        _ => None,
    };
    match pad {
        Some(byte) => Ok(Some(byte)),
        None => Err(Raised::incorrect_pad(&interp.string_value_text(value)).into()),
    }
}

/// `optionalOptionArgument` with a valid set (`classes/StringClassUtil.cpp:341`):
/// `None` for an omitted argument, 88.909 for one without a string value,
/// and 93.915 for the null string or a first byte outside `valid`, compared
/// case-folded; the `0x00` byte is admitted as the builtin's is -- measured,
/// oracle rc 163: `.MutableBuffer~new('abcabc')~verify('a', 'X')` reports
/// `must be one of "MN"; found "X"`, and rc 0: `~verify('abc', '00'x)` is `1`.
fn option_method_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
    valid: &str,
) -> Result<Option<u8>, Failure> {
    let Some(value) = args.get(index).copied().flatten() else {
        return Ok(None);
    };
    let text = required_string_argument(interp, value, index + 1)?;
    let letter = interp
        .to_text(text)
        .first()
        .map(|byte| byte.to_ascii_uppercase());
    match letter {
        Some(letter) if letter == 0 || valid.as_bytes().contains(&letter) => Ok(Some(letter)),
        _ => Err(
            Raised::method_option_not_recognised(valid, &interp.string_value_text(value)).into(),
        ),
    }
}

/// `optionalStringArgument` by position (`runtime/MethodArguments.hpp:186`):
/// the null string for an omitted argument, and 88.909 for a value without a
/// string value -- measured, oracle rc 0:
/// `.MutableBuffer~new('abcdef')~translate('ABC')` answers six blanks, the
/// empty input table reading each byte as its own index.
fn optional_string_method_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
) -> Result<Vec<u8>, Failure> {
    let Some(value) = args.get(index).copied().flatten() else {
        return Ok(Vec::new());
    };
    let text = required_string_argument(interp, value, index + 1)?;
    Ok(interp.to_text(text).into_owned())
}

/// [`optional_string_method_argument`] where an omitted argument has to stay
/// distinguishable from the null string, which is
/// `StringUtil::makearray`'s own `separator != OREF_NULL` test
/// (`classes/support/StringUtil.cpp:552`): `None` for an omitted argument,
/// and 88.909 for a value without a string value.
fn optional_string_or_none_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
) -> Result<Option<Vec<u8>>, Failure> {
    let Some(value) = args.get(index).copied().flatten() else {
        return Ok(None);
    };
    let text = required_string_argument(interp, value, index + 1)?;
    Ok(Some(interp.to_text(text).into_owned()))
}

/// `nonNegativeArgument` (`classes/StringClassUtil.cpp:167`): `None` for an
/// omitted argument, and anything that is not a non-negative whole number in
/// range is 93.906 -- measured, oracle rc 163:
/// `.MutableBuffer~new('abcdef')~insert('a', -1)` reports `Method argument 2
/// must be zero or a positive whole number; found "-1"`.
fn optional_non_negative_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
) -> Result<Option<usize>, Failure> {
    let raise = move |found: &[u8]| Raised::argument_not_non_negative(index + 1, found);
    match whole_method_argument(interp, args, index, raise)? {
        Some(value) if value >= 0 => Ok(Some(usize_or_refuse(interp, args, index, value, raise)?)),
        Some(_) => Err(refuse_method_argument(interp, args, index, raise)),
        None => Ok(None),
    }
}

/// `stringArgument`'s named overload (`runtime/MethodArguments.hpp:161`), the
/// bytes copied into the lent result buffer: omitted is 88.901 and a value
/// without a string value is 88.909.
fn named_string_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
    argument: &'static str,
) -> Result<Vec<u8>, Failure> {
    let Some(value) = args.get(index).copied().flatten() else {
        return Err(Raised::missing_named_argument(argument).into());
    };
    let text = required_string_named_argument(interp, value, argument)?;
    let mut bytes = interp.take_result_buffer();
    bytes.extend_from_slice(&interp.to_text(text));
    Ok(bytes)
}

/// `positionArgument`'s named overload (`classes/StringClassUtil.cpp:225`):
/// omitted is 88.901 and anything that is not a positive whole number in
/// range is 88.912.
fn named_position_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
    argument: &'static str,
) -> Result<usize, Failure> {
    let raise = move |found: &[u8]| Raised::named_argument_invalid_position(argument, found);
    match whole_method_argument(interp, args, index, raise)? {
        Some(position) if position > 0 => usize_or_refuse(interp, args, index, position, raise),
        Some(_) => Err(refuse_method_argument(interp, args, index, raise)),
        None => Err(Raised::missing_named_argument(argument).into()),
    }
}

/// `optionalLengthArgument`'s named overload
/// (`runtime/MethodArguments.hpp:344`): `None` for an omitted argument, and
/// anything that is not a non-negative whole number in range is 88.911.
fn optional_named_length_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
    argument: &'static str,
) -> Result<Option<usize>, Failure> {
    let raise = move |found: &[u8]| Raised::named_argument_invalid_length(argument, found);
    match whole_method_argument(interp, args, index, raise)? {
        Some(size) if size >= 0 => Ok(Some(usize_or_refuse(interp, args, index, size, raise)?)),
        Some(_) => Err(refuse_method_argument(interp, args, index, raise)),
        None => Ok(None),
    }
}

/// `optionalPadArgument`'s named overload
/// (`classes/StringClassUtil.cpp:275`): `None` for an omitted argument,
/// 88.909 for one without a string value, and 88.910 for a string that is not
/// one byte.
fn named_pad_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
    argument: &'static str,
) -> Result<Option<u8>, Failure> {
    let Some(value) = args.get(index).copied().flatten() else {
        return Ok(None);
    };
    let text = required_string_named_argument(interp, value, argument)?;
    let pad = match interp.to_text(text).as_ref() {
        [byte] => Some(*byte),
        _ => None,
    };
    match pad {
        Some(byte) => Ok(Some(byte)),
        None => Err(
            Raised::named_argument_invalid_pad(argument, &interp.string_value_text(value)).into(),
        ),
    }
}

/// The refusal a constructor that has checked its arguments and cannot build
/// the primitive body answers.
///
/// [`Loud::native_method`] with the receiving class's own `~id`, so the
/// message names the class the send went to rather than the one the row is
/// filed under.
fn unbuilt_new(interp: &mut Interp, class: ObjRef) -> Failure {
    unbuilt_class_method(interp, class, b"NEW")
}

/// [`unbuilt_new`] for a class method under some other name.
fn unbuilt_class_method(interp: &mut Interp, class: ObjRef, name: &[u8]) -> Failure {
    let id = interp.classes().id_string(class).to_string();
    Loud::native_method(name, &id).into()
}

/// `MutableBuffer~new(string, size, ...)`: an instance carrying the string
/// and the capacities `MutableBuffer::newRexx` derives from its arguments
/// (`classes/MutableBufferClass.cpp:90`): `default_size` is the second
/// argument or `DEFAULT_BUFFER_LENGTH`, and the capacity is that or the
/// string's length, whichever is larger.
///
/// **The `INIT` send takes the arguments from the front with the last two
/// dropped from the count**, `completeNewObject(newBuffer, args, argc > 2 ?
/// argc - 2 : 0)` (`:134`), which is not the same list as "the arguments past
/// the second".
fn native_mutable_buffer_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let mut initial = interp.take_result_buffer();
    if let Some(text) = args.first().copied().flatten() {
        let text = required_string_argument(interp, text, 1)?;
        initial.extend_from_slice(&interp.to_text(text));
    }
    let default_size = optional_length_argument(interp, args, 1)?.unwrap_or(BUFFER_DEFAULT_LENGTH);
    let capacity = default_size.max(initial.len());
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(capacity)
        .map_err(|_| Failure::from(Raised::system_resources()))?;
    bytes.extend_from_slice(&initial);
    interp.give_result_buffer(initial);
    let object = new_instance(interp, class)?;
    let Some(Object {
        body: Body::Instance { native, .. },
        ..
    }) = interp.heap.get_mut(object)
    else {
        unreachable!("new_instance allocates a Body::Instance")
    };
    *native = Some(Box::new(BufferState {
        bytes,
        capacity,
        default_size,
    }));
    let caller = interp.caller();
    let kept = args.len().saturating_sub(2);
    let rest: Vec<Option<ObjRef>> = args.iter().take(kept).copied().collect();
    interp.send_message(object, INIT, None, &rest, caller)?;
    Ok(Some(object))
}

/// `MutableBuffer::DEFAULT_BUFFER_LENGTH` (`classes/MutableBufferClass.hpp:150`).
const BUFFER_DEFAULT_LENGTH: usize = 256;

/// The receiver's buffer state, or [`Loud::native_method`] for a receiver
/// that carries none.
fn buffer_state<'a>(
    interp: &'a Interp,
    receiver: ObjRef,
    name: &[u8],
) -> Result<&'a BufferState, Failure> {
    interp
        .buffer(receiver)
        .ok_or_else(|| Loud::native_method(name, "MutableBuffer").into())
}

/// [`buffer_state`] for a method that changes the contents.
fn buffer_state_mut<'a>(
    interp: &'a mut Interp,
    receiver: ObjRef,
    name: &[u8],
) -> Result<&'a mut BufferState, Failure> {
    interp
        .buffer_mut(receiver)
        .ok_or_else(|| Loud::native_method(name, "MutableBuffer").into())
}

/// `MutableBuffer::lengthRexx` (`classes/MutableBufferClass.cpp:310`).
fn native_mutable_buffer_length(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let length = buffer_state(interp, receiver, b"LENGTH")?.bytes.len();
    Ok(Some(interp.counted(length)))
}

/// `MutableBuffer::getBufferSize` (`classes/MutableBufferClass.hpp:91`).
fn native_mutable_buffer_getbuffersize(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let capacity = buffer_state(interp, receiver, b"GETBUFFERSIZE")?.capacity;
    Ok(Some(interp.counted(capacity)))
}

/// `MutableBuffer~string`, `RexxObject::makeStringRexx` reaching
/// `MutableBuffer::stringValue` (`classes/MutableBufferClass.cpp:740`): a
/// fresh string of the contents.
fn native_mutable_buffer_string(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let state = buffer_state(interp, receiver, b"STRING")?;
    let mut out = interp.take_result_buffer();
    out.extend_from_slice(&state.bytes);
    Ok(Some(interp.text_built(out)))
}

/// `MutableBuffer~makeString`, `RexxObject::makeStringRexx`
/// (`classes/ObjectClass.cpp:2846`) reaching `MutableBuffer::makeString`
/// (`classes/MutableBufferClass.cpp:717`): a fresh string of the contents.
/// `Setup.cpp:1446` and `:1473` bind that one C++ entry under both `String`
/// and `makeString`.
///
/// This is what the required-string protocol reaches, so it is what `say buf`
/// and `length(buf)` answer from. The receiver-side comparison does not come
/// here at all -- `buf == 'abc'` sends `==` to the buffer and
/// [`native_object_identical`] answers it, which is the oracle's `0` against
/// `'abc' == buf`'s `1`.
fn native_mutable_buffer_makestring(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let state = buffer_state(interp, receiver, b"MAKESTRING")?;
    let mut out = interp.take_result_buffer();
    out.extend_from_slice(&state.bytes);
    Ok(Some(interp.text_built(out)))
}

/// `MutableBuffer::endsWithRexx` (`classes/MutableBufferClass.cpp:1569`).
///
/// An empty `match` is `0`, `primitiveMatch`'s `len == 0` arm (`:1615`) --
/// measured, oracle rc 0: `.MutableBuffer~new('abc')~endsWith('')` is `0`.
fn native_mutable_buffer_endswith(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let needle = named_string_argument(interp, args, 0, "match")?;
    let state = buffer_state(interp, receiver, b"ENDSWITH")?;
    let answer = ends_with(&state.bytes, &needle, <[u8]>::eq);
    interp.give_result_buffer(needle);
    Ok(Some(interp.counted(usize::from(answer))))
}

/// `MutableBuffer::caselessEndsWithRexx` (`classes/MutableBufferClass.cpp:1590`):
/// `ENDSWITH`'s region compare, folded, an empty `match` likewise `0`.
fn native_mutable_buffer_caselessendswith(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let needle = named_string_argument(interp, args, 0, "match")?;
    let state = buffer_state(interp, receiver, b"CASELESSENDSWITH")?;
    let answer = ends_with(&state.bytes, &needle, crate::builtin::string::caseless_eq);
    interp.give_result_buffer(needle);
    Ok(Some(interp.counted(usize::from(answer))))
}

/// `MutableBuffer::appendRexx` (`classes/MutableBufferClass.cpp:323`): every
/// argument a required string, appended in turn; answers the receiver.
fn native_mutable_buffer_append(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if args.is_empty() {
        return Err(Raised::missing_method_argument(1).into());
    }
    for (index, argument) in args.iter().enumerate() {
        let Some(argument) = *argument else {
            return Err(Raised::missing_method_argument(index + 1).into());
        };
        let argument = required_string_argument(interp, argument, index + 1)?;
        let mut piece = interp.take_result_buffer();
        piece.extend_from_slice(&interp.to_text(argument));
        let state = buffer_state_mut(interp, receiver, b"APPEND")?;
        state
            .ensure_capacity(piece.len())
            .map_err(|_| Failure::from(Raised::system_resources()))?;
        state.bytes.extend_from_slice(&piece);
        interp.give_result_buffer(piece);
    }
    Ok(Some(receiver))
}

/// `MutableBuffer::mydelete` (`classes/MutableBufferClass.cpp:647`): the
/// position defaults to 1 and the length to the rest of the contents, and a
/// position past them deletes nothing; answers the receiver.
///
/// `Setup.cpp:1427` and `:1428` bind this one C++ method under both `Delete`
/// and `DelStr`, and `name` is what the two rows differ in.
fn buffer_delete(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    name: &[u8],
) -> Result<Option<ObjRef>, Failure> {
    let (begin, range) = delete_arguments(interp, args)?;
    let state = buffer_state_mut(interp, receiver, name)?;
    crate::builtin::string::delete_range(&mut state.bytes, begin, range);
    Ok(Some(receiver))
}

/// `MutableBuffer~delStr`, [`buffer_delete`].
fn native_mutable_buffer_delstr(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    buffer_delete(interp, receiver, args, b"DELSTR")
}

/// `MutableBuffer~delete`, [`buffer_delete`].
fn native_mutable_buffer_delete(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    buffer_delete(interp, receiver, args, b"DELETE")
}

/// `MutableBuffer::setBufferSize` (`classes/MutableBufferClass.cpp:679`),
/// whose rule [`BufferState::set_buffer_size`] carries; answers the receiver.
fn native_mutable_buffer_setbuffersize(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let size = required_length_argument(interp, args, 0)?;
    let state = buffer_state_mut(interp, receiver, b"SETBUFFERSIZE")?;
    state
        .set_buffer_size(size)
        .map_err(|_| Failure::from(Raised::system_resources()))?;
    Ok(Some(receiver))
}

/// `MutableBuffer::setTextRexx` (`classes/MutableBufferClass.cpp:355`)
/// reaching `setText` (`:369`): the contents become the argument's; answers
/// the receiver.
///
/// **The length is zeroed before the capacity is raised**, which is what
/// `setText` does before it appends, so the size `ensureCapacity` needs is
/// the argument's own length and not the sum -- measured, oracle rc 0:
/// `.MutableBuffer~new('abc', 10)~setText(copies('y',40))` reads
/// `getBufferSize` `40` where raising for the sum would read `43`.
fn native_mutable_buffer_settext(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let new = string_method_argument(interp, args, 0)?;
    let state = buffer_state_mut(interp, receiver, b"SETTEXT")?;
    state.bytes.clear();
    buffer_capacity(state, new.len())?;
    state.bytes.extend_from_slice(&new);
    interp.give_result_buffer(new);
    Ok(Some(receiver))
}

/// `substr`'s arguments: a 0-based start, an optional length, and a pad
/// defaulting to a blank.
///
/// **Shared by both receivers**, and the C++ says so at the site:
/// `RexxString::substr` (`classes/StringClassSub.cpp:598`) is one line under
/// the comment *"use the common code shared with MutableBuffer"*, and that
/// common code is `StringUtil::substr`'s padding overload
/// (`classes/support/StringUtil.cpp:66`), which `MutableBuffer::substr`
/// (`classes/MutableBufferClass.cpp:770`) calls too. A pad wider than one character is 93.922, which is this layer's and
/// not the core's.
pub(super) fn substr_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(usize, Option<usize>, u8), Failure> {
    let start = required_position_argument(interp, args, 0)? - 1;
    let length = optional_length_argument(interp, args, 1)?;
    let pad = pad_method_argument(interp, args, 2)?.unwrap_or(b' ');
    Ok((start, length, pad))
}

/// The optional length `C2D`, `D2C`, `D2X` and `X2D` take, as methods.
///
/// An ordinary `optionalLengthArgument`: `None` for omitted, 93.923 for
/// anything that is not a non-negative whole number in range. **The builtin
/// forms do not share it** -- there the same mistake is 40.x, so the two
/// conversions' cores read their length through the caller rather than
/// reading it themselves.
pub(super) fn conversion_length_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<Option<usize>, Failure> {
    optional_length_argument(interp, args, 0)
}

/// A count argument that is zero or a positive whole number, answered as the
/// `i64` the numeric cores take.
///
/// [`optional_non_negative_argument`] beside it narrows to `usize`, which is
/// right for an index into bytes and wrong for a width: `padding_width` reads
/// an `i64` and decides for itself which widths are absurd, and narrowing
/// first would move that decision.
///
/// 93.906 for a negative or non-whole value, the same sub-code `COPIES` uses
/// and not the 93.923 the pad family raises.
pub(super) fn count_method_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
) -> Result<Option<i64>, Failure> {
    let raise = move |found: &[u8]| Raised::argument_not_non_negative(index + 1, found);
    match whole_method_argument(interp, args, index, raise)? {
        Some(value) if value >= 0 => Ok(Some(value)),
        Some(_) => Err(refuse_method_argument(interp, args, index, raise)),
        None => Ok(None),
    }
}

/// `BITAND`/`BITOR`/`BITXOR`'s optional operand and optional pad.
///
/// Neither is required and the operand has no length rule, so the pad is the
/// only thing here that can be wrong: 93.922 for anything but exactly one
/// byte, 88.909 for a value with no string value. **The pad is answered
/// undefaulted**, because each operation's default is its own identity byte
/// and only the caller knows which operation it is.
pub(super) fn bit_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, Option<u8>), Failure> {
    let other = optional_string_method_argument(interp, args, 0)?;
    let pad = pad_method_argument(interp, args, 1)?;
    Ok((other, pad))
}

/// `DATATYPE`'s option letter, or `None` when it is omitted.
///
/// **Not [`option_method_argument`]**, which substitutes the whole option
/// string into its 93.915. `DATATYPE` substitutes the LETTER: measured,
/// `'5'~dataType('Zonk')` reports `found "Z"` where `'  ab  '~strip('Zonk')`
/// reports `found "Zonk"`. An empty option has no first letter and reports
/// the NUL byte, which the report renders `?`.
pub(super) fn datatype_option_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<Option<u8>, Failure> {
    let Some(value) = args.first().copied().flatten() else {
        return Ok(None);
    };
    let text = required_string_argument(interp, value, 1)?;
    let letter = interp
        .to_text(text)
        .first()
        .copied()
        .unwrap_or(0)
        .to_ascii_uppercase();
    Ok(Some(letter))
}

/// `STRIP`'s option and character set.
///
/// The option is `"BLT"`, defaulting to `B`, and an unrecognised letter is
/// 93.915 naming the accepted set -- measured, `'x'~strip('Z')` reports
/// `Method option must be one of "BLT"; found "Z".` An empty option string is
/// as wrong as a wrong letter.
///
/// **The set is answered as `Option`, because omitted and null are different
/// arguments.** Omitted takes the whitespace default; a null string is an
/// empty set that strips nothing. Documented, in rexxref's own words: "If
/// chars is a null string, then no characters are removed." There is no
/// length check on it at all -- `STRIP("12.0000", "T", '.0')` is the
/// documentation's own two-character example.
pub(super) fn strip_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(u8, Option<Vec<u8>>), Failure> {
    let option = option_method_argument(interp, args, 0, "BLT")?.unwrap_or(b'B');
    let set = optional_string_or_none_argument(interp, args, 1)?;
    Ok((option, set))
}

/// `EQUALS`'s and `CASELESSEQUALS`'s one operand, as its **string value**.
///
/// **Not [`string_method_argument`]**, which is 88.909 for a value with no
/// string value. `equals` has no such refusal -- measured, `'abc'~equals(.nil)`
/// and `'abc'~equals(.array)` are both `0`, because every object renders and
/// the rendering simply is not `abc`. An omitted operand is still 93.903.
pub(super) fn equals_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<Vec<u8>, Failure> {
    let Some(value) = args.first().copied().flatten() else {
        return Err(Raised::missing_method_argument(1).into());
    };
    Ok(interp.string_value_text(value))
}

/// `COMPARETO`'s and `CASELESSCOMPARETO`'s three arguments.
///
/// The other string is required and strict -- 88.909, unlike `equals` above --
/// the start is a `positionArgument` defaulting to 1 (93.924) and the length is
/// an `optionalLengthArgument` (93.923) whose default is
/// `max(len, other) - start + 1` and so is left to the caller, which is the one
/// piece of it that needs both strings.
pub(super) fn compare_to_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, usize, Option<usize>), Failure> {
    let other = string_method_argument(interp, args, 0)?;
    let start = optional_position_argument(interp, args, 1)?.unwrap_or(1);
    let length = optional_length_argument(interp, args, 2)?;
    Ok((other, start, length))
}

/// `ABBREV`'s candidate and its minimum length.
///
/// The candidate is required (93.903 omitted, 88.909 without a string value)
/// and the length is `optionalLengthArgument`, 93.923.
pub(super) fn abbrev_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, Option<usize>), Failure> {
    let info = string_method_argument(interp, args, 0)?;
    let minimum = optional_length_argument(interp, args, 1)?;
    Ok((info, minimum))
}

/// `COMPARE`'s other string and its pad.
///
/// The same first argument as [`abbrev_arguments`] over a second that is a pad
/// rather than a length, so the two differ only in which sub-code the second
/// argument raises: 93.922 here against 93.923 there.
pub(super) fn compare_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, u8), Failure> {
    let other = string_method_argument(interp, args, 0)?;
    let pad = pad_method_argument(interp, args, 1)?.unwrap_or(b' ');
    Ok((other, pad))
}

/// A width and a pad: `CENTER`/`CENTRE`/`LEFT`/`RIGHT`'s two arguments as
/// methods.
///
/// The width is `lengthArgument` and required, so an omitted one is 93.903 and
/// anything that is not a non-negative whole number in range is 93.923; the pad
/// is `optionalPadArgument`, 93.922 for anything but exactly one byte and
/// 88.909 -- an 88, not a 93 -- for a value with no string value at all. That
/// is the pair `RexxString::center` opens with
/// (`classes/StringClassSub.cpp:59`) and `left` and `right` repeat.
pub(super) fn pad_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(usize, u8), Failure> {
    let width = required_length_argument(interp, args, 0)?;
    let pad = pad_method_argument(interp, args, 1)?.unwrap_or(b' ');
    Ok((width, pad))
}

/// `COPIES`'s count: `nonNegativeArgument` (`classes/StringClassMisc.cpp:288`),
/// required.
///
/// **A different sub-code from the width above for the same shape of
/// mistake.** Measured, oracle: `'abc'~center(-1)` is 93.923 `Invalid length
/// argument specified` and `'abc'~copies(-1)` is 93.906 `Method argument 1
/// must be zero or a positive whole number`.
pub(super) fn copies_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<usize, Failure> {
    match optional_non_negative_argument(interp, args, 0)? {
        Some(count) => Ok(count),
        None => Err(Raised::missing_method_argument(1).into()),
    }
}

/// `verify`'s arguments: the reference set, the `M`/`N` option defaulting to
/// `N`, a 0-based start, and an optional range.
///
/// **The option is the one argument here with an error of its own** -- an
/// unrecognised letter is 93.915, which names the accepted set, and it is
/// raised before the start is looked at. Measured, oracle:
/// `'abcabc'~verify('ab', 'Z')` is 93.915 and so is the same send to a
/// `MutableBuffer`.
pub(super) fn verify_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, u8, usize, Option<usize>), Failure> {
    let reference = string_method_argument(interp, args, 0)?;
    let option = option_method_argument(interp, args, 1, "MN")?.unwrap_or(b'N');
    let start = optional_position_argument(interp, args, 2)?.unwrap_or(1) - 1;
    let range = optional_length_argument(interp, args, 3)?;
    Ok((reference, option, start, range))
}

/// `MutableBuffer::substr` (`classes/MutableBufferClass.cpp:770`),
/// `StringUtil::substr`'s padding form.
fn native_mutable_buffer_substr(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (start, length, pad) = substr_arguments(interp, args)?;
    let state = buffer_state(interp, receiver, b"SUBSTR")?;
    let mut out = interp.take_result_buffer();
    crate::builtin::string::substr_bytes(&mut out, &state.bytes, start, length, pad)?;
    Ok(Some(interp.text_built(out)))
}

/// `MutableBuffer::brackets` (`classes/MutableBufferClass.cpp:787`),
/// `StringUtil::substr`'s two-argument form: the length defaults to one
/// byte, is capped at the end of the contents, and nothing pads -- measured,
/// oracle rc 0: `.MutableBuffer~new('abcabc')[2]` is `b` and `[5, 10]` is `bc`.
fn native_mutable_buffer_brackets(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let start = required_position_argument(interp, args, 0)? - 1;
    let length = optional_length_argument(interp, args, 1)?.unwrap_or(1);
    let state = buffer_state(interp, receiver, b"[]")?;
    let capped = length.min(state.bytes.len().saturating_sub(start));
    let mut out = interp.take_result_buffer();
    crate::builtin::string::substr_bytes(&mut out, &state.bytes, start, Some(capped), b' ')?;
    Ok(Some(interp.text_built(out)))
}

/// `StringUtil::posRexx`'s argument handling: the needle, a 1-based start
/// defaulting to the first byte, and an optional range.
///
/// **Shared by both receivers, which is what the C++ does too.**
/// `RexxString::posRexx` (`classes/StringClassMisc.cpp:581`) and
/// `MutableBuffer::posRexx` (`classes/MutableBufferClass.cpp:803`) each
/// forward to `StringUtil::posRexx(getStringData(), getLength(), ...)`
/// (`classes/support/StringUtil.cpp:184`), so the two differ in where the
/// bytes come from and in nothing else. Every error a bad argument raises is
/// raised here, once.
pub(super) fn forward_search_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, usize, Option<usize>), Failure> {
    let needle = string_method_argument(interp, args, 0)?;
    let start = optional_position_argument(interp, args, 1)?.unwrap_or(1);
    let range = optional_length_argument(interp, args, 2)?;
    Ok((needle, start, range))
}

/// [`forward_search_arguments`]' search: the 1-based position of `needle`, or
/// 0, with an omitted range reaching the end of `haystack`.
///
/// `scan` is [`crate::builtin::string::find_forward`] or its caseless twin.
/// **The caseless one is not the plain scan with a folded compare** -- the two
/// oracle scans answer differently for the same arguments, measured at
/// `caseless_find_forward`'s own doc.
pub(super) fn forward_search(
    haystack: &[u8],
    needle: &[u8],
    start: usize,
    range: Option<usize>,
    scan: fn(&[u8], &[u8], usize, usize) -> usize,
) -> usize {
    let range = range.unwrap_or(haystack.len().saturating_sub(start) + 1);
    scan(haystack, needle, start - 1, range)
}

/// `StringUtil::lastPosRexx`'s argument handling: the needle, and a start and
/// range that each default to the receiver's whole length rather than to a
/// fixed number, so both stay `None` here.
pub(super) fn backward_search_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, Option<usize>, Option<usize>), Failure> {
    let needle = string_method_argument(interp, args, 0)?;
    let start = optional_position_argument(interp, args, 1)?;
    let range = optional_length_argument(interp, args, 2)?;
    Ok((needle, start, range))
}

/// [`backward_search_arguments`]' search, with both defaults taken from
/// `haystack`.
pub(super) fn backward_search(
    haystack: &[u8],
    needle: &[u8],
    start: Option<usize>,
    range: Option<usize>,
    scan: fn(&[u8], &[u8], usize, usize) -> usize,
) -> usize {
    let length = haystack.len();
    scan(
        haystack,
        needle,
        start.unwrap_or(length),
        range.unwrap_or(length),
    )
}

/// [`forward_search`] over a buffer's contents, for the `MutableBuffer` rows
/// that answer a position: `POS` and `CONTAINS`
/// (`classes/MutableBufferClass.cpp:803`, `:819`) and their caseless twins
/// (`:851`, `:869`).
fn buffer_pos(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    name: &[u8],
    scan: fn(&[u8], &[u8], usize, usize) -> usize,
) -> Result<usize, Failure> {
    let (needle, start, range) = forward_search_arguments(interp, args)?;
    let state = buffer_state(interp, receiver, name)?;
    let found = forward_search(&state.bytes, &needle, start, range, scan);
    interp.give_result_buffer(needle);
    Ok(found)
}

/// `MutableBuffer::posRexx` (`classes/MutableBufferClass.cpp:803`).
fn native_mutable_buffer_pos(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = buffer_pos(
        interp,
        receiver,
        args,
        b"POS",
        crate::builtin::string::find_forward,
    )?;
    Ok(Some(interp.counted(found)))
}

/// `MutableBuffer::containsRexx` (`classes/MutableBufferClass.cpp:819`).
fn native_mutable_buffer_contains(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = buffer_pos(
        interp,
        receiver,
        args,
        b"CONTAINS",
        crate::builtin::string::find_forward,
    )?;
    Ok(Some(interp.counted(usize::from(found > 0))))
}

/// `MutableBuffer::caselessPos` (`classes/MutableBufferClass.cpp:851`).
fn native_mutable_buffer_caselesspos(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = buffer_pos(
        interp,
        receiver,
        args,
        b"CASELESSPOS",
        crate::builtin::string::caseless_find_forward,
    )?;
    Ok(Some(interp.counted(found)))
}

/// `MutableBuffer::caselessContains` (`classes/MutableBufferClass.cpp:869`).
fn native_mutable_buffer_caselesscontains(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = buffer_pos(
        interp,
        receiver,
        args,
        b"CASELESSCONTAINS",
        crate::builtin::string::caseless_find_forward,
    )?;
    Ok(Some(interp.counted(usize::from(found > 0))))
}

/// `MutableBuffer::lastPos` (`classes/MutableBufferClass.cpp:835`) through
/// `StringUtil::lastPosRexx`: the start and the range each default to the
/// whole length.
fn native_mutable_buffer_lastpos(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (needle, start, range) = backward_search_arguments(interp, args)?;
    let state = buffer_state(interp, receiver, b"LASTPOS")?;
    let found = backward_search(
        &state.bytes,
        &needle,
        start,
        range,
        crate::builtin::string::find_backward,
    );
    interp.give_result_buffer(needle);
    Ok(Some(interp.counted(found)))
}

/// `MutableBuffer::caselessLastPos` (`classes/MutableBufferClass.cpp:890`):
/// `LASTPOS`'s scan and defaults, the compare folded.
fn native_mutable_buffer_caselesslastpos(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (needle, start, range) = backward_search_arguments(interp, args)?;
    let state = buffer_state(interp, receiver, b"CASELESSLASTPOS")?;
    let found = backward_search(
        &state.bytes,
        &needle,
        start,
        range,
        crate::builtin::string::caseless_find_backward,
    );
    interp.give_result_buffer(needle);
    Ok(Some(interp.counted(found)))
}

/// `MutableBuffer::countStrRexx` (`classes/MutableBufferClass.cpp:940`).
fn native_mutable_buffer_countstr(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let needle = string_method_argument(interp, args, 0)?;
    let state = buffer_state(interp, receiver, b"COUNTSTR")?;
    let count = crate::builtin::string::count_occurrences(&state.bytes, &needle, usize::MAX);
    interp.give_result_buffer(needle);
    Ok(Some(interp.counted(count)))
}

/// `MutableBuffer::caselessCountStrRexx` (`classes/MutableBufferClass.cpp:955`).
fn native_mutable_buffer_caselesscountstr(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let needle = string_method_argument(interp, args, 0)?;
    let state = buffer_state(interp, receiver, b"CASELESSCOUNTSTR")?;
    let count =
        crate::builtin::string::caseless_count_occurrences(&state.bytes, &needle, usize::MAX);
    interp.give_result_buffer(needle);
    Ok(Some(interp.counted(count)))
}

/// `MutableBuffer::verify` (`classes/MutableBufferClass.cpp:1744`) through
/// `StringUtil::verify`, which builds an integer on every path -- a start past
/// the end included, where the builtin answers text.
fn native_mutable_buffer_verify(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (reference, option, start, range) = verify_arguments(interp, args)?;
    let state = buffer_state(interp, receiver, b"VERIFY")?;
    let answer =
        crate::builtin::string::verify_bytes(&state.bytes, &reference, option, start, range);
    interp.give_result_buffer(reference);
    Ok(Some(interp.counted(answer)))
}

/// `MutableBuffer::subWord` (`classes/MutableBufferClass.cpp:1758`).
fn native_mutable_buffer_subword(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = required_position_argument(interp, args, 0)?;
    let count = optional_length_argument(interp, args, 1)?;
    let state = buffer_state(interp, receiver, b"SUBWORD")?;
    let found = crate::builtin::word::subword_range(&state.bytes, position, count);
    let mut out = interp.take_result_buffer();
    out.extend_from_slice(&state.bytes[found]);
    Ok(Some(interp.text_built(out)))
}

/// A fresh `Array` holding one string per element of `pieces`.
///
/// An empty result carries no dimensions, which is `~dimension` `0` --
/// measured, oracle rc 0: `.MutableBuffer~new('')~makeArray~dimension` is `0`
/// where a one-line buffer's is `1`.
///
/// Each string is rooted as it is built, because [`Interp::alloc_with`]
/// collects before it allocates and a string already made is reachable from
/// nothing until the array carries it.
fn array_of_texts(interp: &mut Interp, pieces: Vec<Vec<u8>>) -> Result<ObjRef, Failure> {
    let frame = interp.roots.push_frame();
    let mut slots = Vec::with_capacity(pieces.len());
    for piece in pieces {
        let value = interp.text_built(piece);
        interp.roots.push_temp(value);
        slots.push(Some(value));
    }
    let object = interp.alloc_with(
        BehaviourId::ARRAY,
        Body::Array {
            dimensions: None,
            slots,
        },
    );
    interp.roots.pop_frame(frame);
    interp.roots.push_temp(object);
    let caller = interp.caller();
    interp.send_message(object, INIT, None, &[], caller)?;
    Ok(object)
}

/// `MutableBuffer::subWords` (`classes/MutableBufferClass.cpp:1778`) over
/// `StringUtil::subWords` (`classes/support/StringUtil.cpp:1405`): an
/// **`Array`** of the words from `position`, at most `count` of them.
///
/// Both arguments are converted before the contents are looked at, so
/// `subWords(9, .nil)` is 93.923 rather than the empty array a start past the
/// last word answers -- measured, oracle rc 163.
fn native_mutable_buffer_subwords(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = optional_position_argument(interp, args, 0)?.unwrap_or(1);
    let count = optional_length_argument(interp, args, 1)?.unwrap_or(usize::MAX);
    let state = buffer_state(interp, receiver, b"SUBWORDS")?;
    let words: Vec<Vec<u8>> = crate::builtin::word::word_slices(&state.bytes)
        .into_iter()
        .skip(position - 1)
        .take(count)
        .map(<[u8]>::to_vec)
        .collect();
    Ok(Some(array_of_texts(interp, words)?))
}

/// `MutableBuffer::makeArrayRexx` (`classes/MutableBufferClass.cpp:927`) over
/// `StringUtil::makearray` (`classes/support/StringUtil.cpp:545`): an
/// `Array` of the contents split on line ends, or on `separator` where one is
/// given.
fn native_mutable_buffer_makearray(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let separator = optional_string_or_none_argument(interp, args, 0)?;
    let state = buffer_state(interp, receiver, b"MAKEARRAY")?;
    let pieces = match &separator {
        Some(separator) => crate::builtin::string::split_slices(&state.bytes, separator),
        None => crate::builtin::string::line_slices(&state.bytes),
    };
    let pieces: Vec<Vec<u8>> = pieces.into_iter().map(<[u8]>::to_vec).collect();
    Ok(Some(array_of_texts(interp, pieces)?))
}

/// `MutableBuffer::word` (`classes/MutableBufferClass.cpp:1791`).
fn native_mutable_buffer_word(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = required_position_argument(interp, args, 0)?;
    let state = buffer_state(interp, receiver, b"WORD")?;
    let found = crate::builtin::word::word_range(&state.bytes, position).unwrap_or(0..0);
    let mut out = interp.take_result_buffer();
    out.extend_from_slice(&state.bytes[found]);
    Ok(Some(interp.text_built(out)))
}

/// `MutableBuffer::wordIndex` (`classes/MutableBufferClass.cpp:1805`).
fn native_mutable_buffer_wordindex(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = required_position_argument(interp, args, 0)?;
    let state = buffer_state(interp, receiver, b"WORDINDEX")?;
    let index =
        crate::builtin::word::word_range(&state.bytes, position).map_or(0, |word| word.start + 1);
    Ok(Some(interp.counted(index)))
}

/// `MutableBuffer::wordLength` (`classes/MutableBufferClass.cpp:1820`).
fn native_mutable_buffer_wordlength(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = required_position_argument(interp, args, 0)?;
    let state = buffer_state(interp, receiver, b"WORDLENGTH")?;
    let length =
        crate::builtin::word::word_range(&state.bytes, position).map_or(0, |word| word.len());
    Ok(Some(interp.counted(length)))
}

/// `MutableBuffer::words` (`classes/MutableBufferClass.cpp:1830`).
fn native_mutable_buffer_words(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let count = crate::builtin::word::word_count(&buffer_state(interp, receiver, b"WORDS")?.bytes);
    Ok(Some(interp.counted(count)))
}

/// `StringUtil::wordPos`'s argument handling: the phrase, and a 1-based start
/// defaulting to the first word.
///
/// **Shared by both receivers**, as `StringUtil::wordPos`
/// (`classes/support/StringUtil.cpp:1565`) is: `RexxString::wordPos`
/// (`classes/StringClassWord.cpp:256`) and `MutableBuffer::wordPos`
/// (`classes/MutableBufferClass.cpp:1845`) pass it their own bytes and nothing
/// else.
pub(super) fn wordpos_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, usize), Failure> {
    let phrase = string_method_argument(interp, args, 0)?;
    let start = optional_position_argument(interp, args, 1)?.unwrap_or(1);
    Ok((phrase, start))
}

/// [`wordpos_arguments`]' search over a buffer's contents, shared by
/// `WORDPOS` and `CONTAINSWORD` (`classes/MutableBufferClass.cpp:1845`,
/// `:1859`) and their caseless twins (`:1873`, `:1887`).
fn buffer_wordpos(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    name: &[u8],
    scan: fn(&[u8], &[u8], usize) -> usize,
) -> Result<usize, Failure> {
    let (phrase, start) = wordpos_arguments(interp, args)?;
    let state = buffer_state(interp, receiver, name)?;
    let found = scan(&phrase, &state.bytes, start);
    interp.give_result_buffer(phrase);
    Ok(found)
}

/// `MutableBuffer::wordPos` (`classes/MutableBufferClass.cpp:1845`).
fn native_mutable_buffer_wordpos(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = buffer_wordpos(
        interp,
        receiver,
        args,
        b"WORDPOS",
        crate::builtin::word::wordpos_bytes,
    )?;
    Ok(Some(interp.counted(found)))
}

/// `MutableBuffer::containsWord` (`classes/MutableBufferClass.cpp:1859`).
fn native_mutable_buffer_containsword(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = buffer_wordpos(
        interp,
        receiver,
        args,
        b"CONTAINSWORD",
        crate::builtin::word::wordpos_bytes,
    )?;
    Ok(Some(interp.counted(usize::from(found > 0))))
}

/// `MutableBuffer::caselessWordPos` (`classes/MutableBufferClass.cpp:1873`).
fn native_mutable_buffer_caselesswordpos(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = buffer_wordpos(
        interp,
        receiver,
        args,
        b"CASELESSWORDPOS",
        crate::builtin::word::caseless_wordpos_bytes,
    )?;
    Ok(Some(interp.counted(found)))
}

/// `MutableBuffer::caselessContainsWord` (`classes/MutableBufferClass.cpp:1887`).
fn native_mutable_buffer_caselesscontainsword(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = buffer_wordpos(
        interp,
        receiver,
        args,
        b"CASELESSCONTAINSWORD",
        crate::builtin::word::caseless_wordpos_bytes,
    )?;
    Ok(Some(interp.counted(usize::from(found > 0))))
}

/// `STARTSWITH` and its caseless twin over any bytes: an empty `match`
/// answers `0` on both, which is why this is not `slice::starts_with`.
///
/// `matches` is `<[u8]>::eq` or [`crate::builtin::string::caseless_eq`], the
/// only difference between the two spellings.
pub(super) fn starts_with(
    haystack: &[u8],
    needle: &[u8],
    matches: fn(&[u8], &[u8]) -> bool,
) -> bool {
    !needle.is_empty()
        && haystack
            .get(..needle.len())
            .is_some_and(|front| matches(front, needle))
}

/// [`starts_with`] from the other end.
pub(super) fn ends_with(haystack: &[u8], needle: &[u8], matches: fn(&[u8], &[u8]) -> bool) -> bool {
    !needle.is_empty()
        && haystack
            .len()
            .checked_sub(needle.len())
            .is_some_and(|at| matches(&haystack[at..], needle))
}

/// `MutableBuffer::startsWithRexx` (`classes/MutableBufferClass.cpp:1541`):
/// `ENDSWITH`'s twin, an empty `match` likewise `0`.
fn native_mutable_buffer_startswith(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let needle = named_string_argument(interp, args, 0, "match")?;
    let state = buffer_state(interp, receiver, b"STARTSWITH")?;
    let answer = starts_with(&state.bytes, &needle, <[u8]>::eq);
    interp.give_result_buffer(needle);
    Ok(Some(interp.counted(usize::from(answer))))
}

/// `MutableBuffer::caselessStartsWithRexx` (`classes/MutableBufferClass.cpp:1555`):
/// `STARTSWITH`'s region compare, folded, an empty `match` likewise `0`.
fn native_mutable_buffer_caselessstartswith(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let needle = named_string_argument(interp, args, 0, "match")?;
    let state = buffer_state(interp, receiver, b"CASELESSSTARTSWITH")?;
    let answer = starts_with(&state.bytes, &needle, crate::builtin::string::caseless_eq);
    interp.give_result_buffer(needle);
    Ok(Some(interp.counted(usize::from(answer))))
}

/// `MutableBuffer::match` (`classes/MutableBufferClass.cpp:1460`): a start
/// past the contents is `0` before `other` is looked at -- measured, oracle
/// rc 0: `.MutableBuffer~new('abcabc')~match(99, .nil)` is `0`.
fn native_mutable_buffer_match(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let start = required_position_argument(interp, args, 0)?;
    if start > buffer_state(interp, receiver, b"MATCH")?.bytes.len() {
        return Ok(Some(interp.counted(0)));
    }
    let other = string_method_argument(interp, args, 1)?;
    let answer = match_region(interp, receiver, args, start, &other, b"MATCH", <[u8]>::eq);
    interp.give_result_buffer(other);
    Ok(Some(interp.counted(usize::from(answer?))))
}

/// `MutableBuffer::caselessMatch` (`classes/MutableBufferClass.cpp:1505`):
/// `MATCH`'s scan, `primitiveCaselessMatch` (`:1642`) the only difference,
/// and the same order -- measured, oracle rc 0:
/// `.MutableBuffer~new('aBcaBc')~caselessMatch(99, .nil)` is `0`.
fn native_mutable_buffer_caselessmatch(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let start = required_position_argument(interp, args, 0)?;
    if start
        > buffer_state(interp, receiver, b"CASELESSMATCH")?
            .bytes
            .len()
    {
        return Ok(Some(interp.counted(0)));
    }
    let other = string_method_argument(interp, args, 1)?;
    let answer = match_region(
        interp,
        receiver,
        args,
        start,
        &other,
        b"CASELESSMATCH",
        crate::builtin::string::caseless_eq,
    );
    interp.give_result_buffer(other);
    Ok(Some(interp.counted(usize::from(answer?))))
}

/// The rest of `match`'s arguments once `start` is inside the receiver: an
/// explicit offset past `other` is `0` before the length is looked at, and a
/// length past `other` is `0` -- measured, oracle rc 0: `~match(1, 'abc', 4,
/// -1)` is `0`. `None` is those two answers; `Some` is the region of `other`
/// to compare.
///
/// **Shared by both receivers, and reading these arguments is all it does** --
/// the receiver is not touched here, which is what lets a `String` and a
/// `MutableBuffer` reach the same code with the borrow each of them needs.
pub(super) fn match_region_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    other_len: usize,
) -> Result<Option<(usize, usize)>, Failure> {
    let offset = optional_position_argument(interp, args, 2)?;
    if offset.is_some_and(|offset| offset > other_len) {
        return Ok(None);
    }
    let offset = offset.unwrap_or(1);
    let length = optional_length_argument(interp, args, 3)?.unwrap_or(other_len + 1 - offset);
    if length == 0 || offset.saturating_add(length) - 1 > other_len {
        return Ok(None);
    }
    Ok(Some((offset, length)))
}

/// [`match_region_arguments`]' comparison, once the receiver's bytes are in
/// hand: `primitiveMatch` (`classes/MutableBufferClass.cpp:1615`) compares the
/// two regions, and `matches` is what separates the two spellings.
pub(super) fn match_region_over(
    haystack: &[u8],
    start: usize,
    other: &[u8],
    region: Option<(usize, usize)>,
    matches: fn(&[u8], &[u8]) -> bool,
) -> bool {
    let Some((offset, length)) = region else {
        return false;
    };
    haystack
        .get(start - 1..(start - 1).saturating_add(length))
        .is_some_and(|region| matches(region, &other[offset - 1..offset - 1 + length]))
}

/// [`match_region_arguments`] and [`match_region_over`] over a buffer's
/// contents.
fn match_region(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    start: usize,
    other: &[u8],
    name: &[u8],
    matches: fn(&[u8], &[u8]) -> bool,
) -> Result<bool, Failure> {
    let region = match_region_arguments(interp, args, other.len())?;
    let state = buffer_state(interp, receiver, name)?;
    Ok(match_region_over(
        &state.bytes,
        start,
        other,
        region,
        matches,
    ))
}

/// `MutableBuffer::matchChar` (`classes/MutableBufferClass.cpp:1668`): a
/// position past the contents is `0` before the set is looked at -- measured,
/// oracle rc 0: `.MutableBuffer~new('abcabc')~matchChar(99, .nil)` is `0`.
fn native_mutable_buffer_matchchar(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = required_position_argument(interp, args, 0)?;
    if position > buffer_state(interp, receiver, b"MATCHCHAR")?.bytes.len() {
        return Ok(Some(interp.counted(0)));
    }
    let set = string_method_argument(interp, args, 1)?;
    let state = buffer_state(interp, receiver, b"MATCHCHAR")?;
    let answer = state
        .bytes
        .get(position - 1)
        .is_some_and(|byte| set.contains(byte));
    interp.give_result_buffer(set);
    Ok(Some(interp.counted(usize::from(answer))))
}

/// `MutableBuffer::caselessMatchChar` (`classes/MutableBufferClass.cpp:1705`):
/// `MATCHCHAR`'s scan with both sides folded, and the same order -- measured,
/// oracle rc 0: `.MutableBuffer~new('aBcaBc')~caselessMatchChar(99, .nil)` is
/// `0`.
fn native_mutable_buffer_caselessmatchchar(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = required_position_argument(interp, args, 0)?;
    if position
        > buffer_state(interp, receiver, b"CASELESSMATCHCHAR")?
            .bytes
            .len()
    {
        return Ok(Some(interp.counted(0)));
    }
    let set = string_method_argument(interp, args, 1)?;
    let state = buffer_state(interp, receiver, b"CASELESSMATCHCHAR")?;
    let answer = state.bytes.get(position - 1).is_some_and(|byte| {
        let byte = byte.to_ascii_uppercase();
        set.iter().any(|member| member.to_ascii_uppercase() == byte)
    });
    interp.give_result_buffer(set);
    Ok(Some(interp.counted(usize::from(answer))))
}

/// `MutableBuffer::subchar` (`classes/MutableBufferClass.cpp:914`): the one
/// byte at the position, or the null string past the end.
fn native_mutable_buffer_subchar(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = required_position_argument(interp, args, 0)?;
    let state = buffer_state(interp, receiver, b"SUBCHAR")?;
    let mut out = interp.take_result_buffer();
    out.extend(state.bytes.get(position - 1));
    Ok(Some(interp.text_built(out)))
}

/// A mutator's rebuilt contents written back over the receiver's own, the
/// capacity already raised for them.
///
/// `BufferState::ensure_capacity` has reserved at least `built.len()`, so the
/// copy cannot be what grows the allocation.
fn replace_buffer_contents(state: &mut BufferState, built: &[u8]) {
    state.bytes.clear();
    state.bytes.extend_from_slice(built);
}

/// [`BufferState::ensure_capacity`]'s refusal, the oracle's 5.1.
fn buffer_capacity(state: &mut BufferState, added: usize) -> Result<(), Failure> {
    state
        .ensure_capacity(added)
        .map_err(|_| Failure::from(Raised::system_resources()))
}

/// `changeStr`'s arguments: needle, replacement, and a count defaulting to
/// every occurrence.
pub(super) fn changestr_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, Vec<u8>, usize), Failure> {
    let needle = string_method_argument(interp, args, 0)?;
    let replacement = string_method_argument(interp, args, 1)?;
    let limit = optional_non_negative_argument(interp, args, 2)?.unwrap_or(usize::MAX);
    Ok((needle, replacement, limit))
}

/// `space`'s arguments: the gap between words, defaulting to one, and a pad.
pub(super) fn space_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(usize, u8), Failure> {
    let gap = optional_length_argument(interp, args, 0)?.unwrap_or(1);
    let pad = pad_method_argument(interp, args, 1)?.unwrap_or(b' ');
    Ok((gap, pad))
}

/// The `(start, range)` a case shift takes, from `first` onwards.
///
/// `first` is 0 for `upper` and `lower` and 3 for `translate`'s no-table form,
/// which is a case shift wearing `translate`'s argument positions.
pub(super) fn case_shift_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    first: usize,
) -> Result<(usize, Option<usize>), Failure> {
    let start = optional_position_argument(interp, args, first)?.unwrap_or(1) - 1;
    let range = optional_length_argument(interp, args, first + 1)?;
    Ok((start, range))
}

/// `translate`'s arguments: the output table, the input table, a pad, a
/// 0-based start and an optional range.
pub(super) fn translate_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, Vec<u8>, u8, usize, Option<usize>), Failure> {
    let out_table = optional_string_method_argument(interp, args, 0)?;
    let in_table = optional_string_method_argument(interp, args, 1)?;
    let pad = pad_method_argument(interp, args, 2)?.unwrap_or(b' ');
    let start = optional_position_argument(interp, args, 3)?.unwrap_or(1) - 1;
    let range = optional_length_argument(interp, args, 4)?;
    Ok((out_table, in_table, pad, start, range))
}

/// An empty input table means "every byte", which the core spells `None`.
pub(super) fn translate_in_table(in_table: &[u8]) -> Option<&[u8]> {
    if in_table.is_empty() {
        None
    } else {
        Some(in_table)
    }
}

/// `insert`'s arguments: the string, a 0-based begin defaulting to the front,
/// an optional length and a pad.
///
/// **Shared by both receivers.** Measured, oracle: a non-whole second argument
/// is 93.906 at a `String` and at a `MutableBuffer` alike, so unlike
/// [`replace_at_plan`]'s callers these two agree on every refusal.
pub(super) fn insert_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, usize, Option<usize>, u8), Failure> {
    let new = string_method_argument(interp, args, 0)?;
    let begin = optional_non_negative_argument(interp, args, 1)?.unwrap_or(0);
    let length = optional_length_argument(interp, args, 2)?;
    let pad = pad_method_argument(interp, args, 3)?.unwrap_or(b' ');
    Ok((new, begin, length, pad))
}

/// `overlay`'s arguments, which are [`insert_arguments`]' with a 1-based
/// position in place of the non-negative offset.
pub(super) fn overlay_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, usize, Option<usize>, u8), Failure> {
    let new = string_method_argument(interp, args, 0)?;
    let begin = optional_position_argument(interp, args, 1)?.unwrap_or(1) - 1;
    let length = optional_length_argument(interp, args, 2)?;
    let pad = pad_method_argument(interp, args, 3)?.unwrap_or(b' ');
    Ok((new, begin, length, pad))
}

/// `delStr`'s arguments: a 0-based begin defaulting to the front and an
/// optional range.
pub(super) fn delete_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(usize, Option<usize>), Failure> {
    let begin = optional_position_argument(interp, args, 0)?.unwrap_or(1) - 1;
    let range = optional_length_argument(interp, args, 1)?;
    Ok((begin, range))
}

/// `delWord`'s arguments: a required 1-based word position and an optional
/// count.
pub(super) fn delword_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(usize, Option<usize>), Failure> {
    let position = required_position_argument(interp, args, 0)?;
    let count = optional_length_argument(interp, args, 1)?;
    Ok((position, count))
}

/// How much of the receiver `replaceAt` overwrites, and how long the answer
/// is: `(replaced, final_length)`.
///
/// A begin past the contents replaces nothing and pads out to it; a length
/// running past the end is trimmed to what is there.
pub(super) fn replace_at_plan(
    contents: usize,
    begin: usize,
    length: Option<usize>,
    new_len: usize,
) -> (usize, usize) {
    let mut replaced = length.unwrap_or(new_len);
    if begin > contents {
        replaced = 0;
    } else if begin.saturating_add(replaced) > contents {
        replaced = contents - begin;
    }
    let kept = if begin > contents { begin } else { contents };
    (replaced, kept - replaced + new_len)
}

/// [`replace_at_plan`]'s assembly: the front padded out to `begin`, then
/// `new`, then whatever the replacement did not cover.
pub(super) fn replace_at_bytes(
    out: &mut Vec<u8>,
    bytes: &[u8],
    new: &[u8],
    begin: usize,
    replaced: usize,
    pad: u8,
) -> Result<(), Failure> {
    crate::builtin::string::substr_bytes(out, bytes, 0, Some(begin), pad)?;
    out.extend_from_slice(new);
    out.extend_from_slice(&bytes[(begin + replaced).min(bytes.len())..]);
    Ok(())
}

/// `MutableBuffer::insert` (`classes/MutableBufferClass.cpp:420`): the
/// position is a 0-based count defaulting to 0, a position past the contents
/// pads the gap, and the length pads the insertion; answers the receiver.
fn native_mutable_buffer_insert(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (new, begin, length, pad) = insert_arguments(interp, args)?;
    let mut out = interp.take_result_buffer();
    let state = buffer_state_mut(interp, receiver, b"INSERT")?;
    let insert_length = length.unwrap_or(new.len());
    let added = insert_length.saturating_add(begin.saturating_sub(state.bytes.len()));
    buffer_capacity(state, added)?;
    crate::builtin::string::insert_bytes(&mut out, &state.bytes, &new, begin, length, pad)?;
    replace_buffer_contents(state, &out);
    interp.give_result_buffer(out);
    Ok(Some(receiver))
}

/// `MutableBuffer::overlay` (`classes/MutableBufferClass.cpp:493`): the
/// position is 1-based and defaults to 1, and the capacity is raised for the
/// position plus the overlay length rather than for the result; answers the
/// receiver.
///
/// Measured, oracle rc 0: `.MutableBuffer~new('abc', 10)~overlay('Z', 60)`
/// reads `60 63`, so the capacity outruns the contents by the original
/// length.
fn native_mutable_buffer_overlay(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (new, begin, length, pad) = overlay_arguments(interp, args)?;
    let mut out = interp.take_result_buffer();
    let state = buffer_state_mut(interp, receiver, b"OVERLAY")?;
    let overlay_length = length.unwrap_or(new.len());
    buffer_capacity(state, begin.saturating_add(overlay_length))?;
    crate::builtin::string::overlay_bytes(&mut out, &state.bytes, &new, begin, length, pad)?;
    replace_buffer_contents(state, &out);
    interp.give_result_buffer(out);
    Ok(Some(receiver))
}

/// `MutableBuffer::replaceAt` (`classes/MutableBufferClass.cpp:570`): the
/// range from the 1-based position is excised and the replacement spliced in
/// whole, so the contents shift where the two lengths differ; answers the
/// receiver.
///
/// `Setup.cpp:1425` and `:1426` bind this and `bracketsEqual`
/// (`MutableBufferClass.cpp:545`), which is this with the pad left to its
/// default, and `name` is what the two rows differ in.
fn buffer_replace_at(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    name: &[u8],
) -> Result<Option<ObjRef>, Failure> {
    let new = named_string_argument(interp, args, 0, "new")?;
    let begin = named_position_argument(interp, args, 1, "position")? - 1;
    let length = optional_named_length_argument(interp, args, 2, "length")?;
    let pad = named_pad_argument(interp, args, 3, "pad")?.unwrap_or(b' ');
    let mut out = interp.take_result_buffer();
    let state = buffer_state_mut(interp, receiver, name)?;
    let (replaced, final_length) = replace_at_plan(state.bytes.len(), begin, length, new.len());
    buffer_capacity(state, final_length)?;
    out.try_reserve(final_length)
        .map_err(|_| Failure::from(Raised::system_resources()))?;
    replace_at_bytes(&mut out, &state.bytes, &new, begin, replaced, pad)?;
    replace_buffer_contents(state, &out);
    interp.give_result_buffer(out);
    Ok(Some(receiver))
}

/// `MutableBuffer~replaceAt`, [`buffer_replace_at`].
fn native_mutable_buffer_replaceat(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    buffer_replace_at(interp, receiver, args, b"REPLACEAT")
}

/// `MutableBuffer~'[]='`, [`buffer_replace_at`].
fn native_mutable_buffer_bracketsequal(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    buffer_replace_at(interp, receiver, args, b"[]=")
}

/// `MutableBuffer::changeStr` (`classes/MutableBufferClass.cpp:971`): answers
/// the receiver, and raises the capacity only on the branch where the
/// replacement is longer than the needle (`:1081`).
///
/// Measured, oracle rc 0:
/// `.MutableBuffer~new('abc', 10)~changeStr('b', copies('q', 50))` reads
/// `52 55`, where the equal-length and shorter branches leave the capacity
/// where it was.
fn native_mutable_buffer_changestr(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (needle, replacement, limit) = changestr_arguments(interp, args)?;
    let mut out = interp.take_result_buffer();
    let state = buffer_state_mut(interp, receiver, b"CHANGESTR")?;
    if !needle.is_empty() && limit > 0 && replacement.len() > needle.len() {
        let matches = crate::builtin::string::count_occurrences(&state.bytes, &needle, limit);
        if matches > 0 {
            let growth = matches.saturating_mul(replacement.len() - needle.len());
            let result_length = state.bytes.len().saturating_add(growth);
            buffer_capacity(state, result_length)?;
        }
    }
    crate::builtin::string::changestr_bytes(&mut out, &state.bytes, &needle, &replacement, limit)?;
    replace_buffer_contents(state, &out);
    interp.give_result_buffer(out);
    Ok(Some(receiver))
}

/// `MutableBuffer::caselessChangeStr` (`classes/MutableBufferClass.cpp:1136`):
/// `CHANGESTR`'s three length branches with the search folded, and the same
/// capacity rule -- only the growing branch calls `ensureCapacity`, and what
/// it passes is the result's own length. Measured, oracle rc 0:
/// `.MutableBuffer~new('aBc', 10)~caselessChangeStr('B', copies('q', 40))`
/// reads `42 45`, where a buffer built at the 256 default reads `404 512`.
fn native_mutable_buffer_caselesschangestr(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (needle, replacement, limit) = changestr_arguments(interp, args)?;
    let mut out = interp.take_result_buffer();
    let state = buffer_state_mut(interp, receiver, b"CASELESSCHANGESTR")?;
    if !needle.is_empty() && limit > 0 && replacement.len() > needle.len() {
        let matches =
            crate::builtin::string::caseless_count_occurrences(&state.bytes, &needle, limit);
        if matches > 0 {
            let growth = matches.saturating_mul(replacement.len() - needle.len());
            let result_length = state.bytes.len().saturating_add(growth);
            buffer_capacity(state, result_length)?;
        }
    }
    crate::builtin::string::caseless_changestr_bytes(
        &mut out,
        &state.bytes,
        &needle,
        &replacement,
        limit,
    )?;
    replace_buffer_contents(state, &out);
    interp.give_result_buffer(out);
    Ok(Some(receiver))
}

/// `MutableBuffer::upper` and `::lower` (`classes/MutableBufferClass.cpp:1341`,
/// `:1301`), and `::translate`'s no-table form: the bytes within the range
/// case-shifted in place, with `first` the argument index the position is
/// read from; answers the receiver.
fn buffer_case_shift(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    first: usize,
    name: &[u8],
    shift: fn(&u8) -> u8,
) -> Result<Option<ObjRef>, Failure> {
    let (start, range) = case_shift_arguments(interp, args, first)?;
    let state = buffer_state_mut(interp, receiver, name)?;
    crate::builtin::string::case_shift_bytes(&mut state.bytes, start, range, shift);
    Ok(Some(receiver))
}

/// `MutableBuffer~upper`, [`buffer_case_shift`].
fn native_mutable_buffer_upper(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    buffer_case_shift(interp, receiver, args, 0, b"UPPER", u8::to_ascii_uppercase)
}

/// `MutableBuffer~lower`, [`buffer_case_shift`].
fn native_mutable_buffer_lower(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    buffer_case_shift(interp, receiver, args, 0, b"LOWER", u8::to_ascii_lowercase)
}

/// `MutableBuffer::translate` (`classes/MutableBufferClass.cpp:1382`): each
/// byte within the range that the input table holds -- or every byte, read as
/// its own index, where that table is the null string -- becomes the output
/// table's byte at that index, or the pad past its end; answers the receiver.
///
/// **With all three table arguments omitted this is `upper`**, taking its
/// position and length from arguments four and five (`:1385`-`:1388`) --
/// measured, oracle rc 0:
/// `.MutableBuffer~new('abcdef')~translate(, , , 2, 3)` is `aBCDef`.
fn native_mutable_buffer_translate(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if args.iter().take(3).all(Option::is_none) {
        return buffer_case_shift(
            interp,
            receiver,
            args,
            3,
            b"TRANSLATE",
            u8::to_ascii_uppercase,
        );
    }
    let (out_table, in_table, pad, start, range) = translate_arguments(interp, args)?;
    let state = buffer_state_mut(interp, receiver, b"TRANSLATE")?;
    crate::builtin::string::translate_bytes(
        &mut state.bytes,
        &out_table,
        translate_in_table(&in_table),
        pad,
        start,
        range,
    );
    Ok(Some(receiver))
}

/// `MutableBuffer::space` (`classes/MutableBufferClass.cpp:1962`): the words
/// rejoined by the pad, and the capacity raised from the single-blank form's
/// length rather than from the original (`:2031`, `:2038`); answers the
/// receiver.
fn native_mutable_buffer_space(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (gap, pad) = space_arguments(interp, args)?;
    let mut out = interp.take_result_buffer();
    let state = buffer_state_mut(interp, receiver, b"SPACE")?;
    crate::builtin::string::space_bytes(&mut out, &state.bytes, gap, pad)?;
    let gaps = crate::builtin::word::word_count(&state.bytes).saturating_sub(1);
    let growth = gaps.saturating_mul(gap.saturating_sub(1));
    state.bytes.truncate(out.len() - growth);
    buffer_capacity(state, growth)?;
    replace_buffer_contents(state, &out);
    interp.give_result_buffer(out);
    Ok(Some(receiver))
}

/// `MutableBuffer::delWord` (`classes/MutableBufferClass.cpp:1901`): the words
/// from the 1-based position, with the blanks after the last of them, removed
/// in place; answers the receiver.
fn native_mutable_buffer_delword(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (position, count) = delword_arguments(interp, args)?;
    let state = buffer_state_mut(interp, receiver, b"DELWORD")?;
    crate::builtin::word::delword_bytes(&mut state.bytes, position, count);
    Ok(Some(receiver))
}

/// `Class~new(id, ...)`: the class id is required and this crate builds no
/// class from it -- `RexxClass::newRexx` (`classes/ClassClass.cpp:1776`).
///
/// The refusal past the checks is the clone at `:1789`, which makes the
/// receiver the new class's metaclass; `~subclass` is the factory this crate
/// does build. Measured, oracle: `.Class~new` is `93.901 Not enough arguments
/// for method; 1 expected.` at rc 163 and `.Class~new(.nil)` is `88.909
/// Argument class id must have a string value.` at rc 168.
fn native_class_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    if args.is_empty() {
        return Err(Raised::not_enough_method_arguments(1).into());
    }
    match args[0] {
        Some(id) => required_string_named_argument(interp, id, "class id")?,
        None => return Err(Raised::missing_named_argument("class id").into()),
    };
    Err(unbuilt_new(interp, class))
}

/// `Message~new(target, message, ...)`: a message object whose send has not
/// been made -- `MessageClass::newRexx` (`classes/MessageClass.cpp:828`).
///
/// The target, the name and the arguments are not kept, and a third argument
/// -- the `"AI"` argument-style option (`:861`) -- is refused rather than
/// checked. What the object does carry is the **absence** of an entry in
/// [`Interp::message_outcomes`], which is what tells an unsent message from a
/// completed one; [`Loud::unsent_message_result`] reads it.
///
/// Measured, oracle rc 163: `.Message~new` is `93.901 Not enough arguments
/// for method; 2 expected.`
fn native_message_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    if args.len() < 2 {
        return Err(Raised::not_enough_method_arguments(2).into());
    }
    if args[0].is_none() {
        return Err(Raised::missing_named_argument("message target").into());
    }
    decode_message_name(interp, args[1])?;
    if args.len() > 2 || class != interp.object_model().message {
        return Err(unbuilt_new(interp, class));
    }
    let object = interp.native_instance(class);
    let caller = interp.caller();
    interp.send_message(object, INIT, None, &[], caller)?;
    Ok(Some(object))
}

/// `Method~new(name, source, ...)` and `Routine~new(name, source, ...)`: both
/// arguments are required -- `BaseExecutable::processNewExecutableArgs`
/// (`execution/BaseExecutable.cpp:225`), which `MethodClass::newRexx` and
/// `RoutineClass::newRexx` share.
///
/// Measured, oracle rc 168: `.Method~new` is `88.901 Missing argument;
/// argument name is required.` and `.Method~new('m')` is `88.901 Missing
/// argument; argument source is required.`
///
/// The two-argument `Method` form is built here, through the same
/// [`compile_method_source`] that `Object~setMethod` compiles a source string
/// with. Measured, oracle rc 0: `.Method~new('MM', 'return 7')` answers a
/// `Method` whose `~scope` is `.nil` and whose `~annotations` is an empty
/// `StringTable`, and setting it into a directory answers 7.
///
/// `Routine` shares this function and still compiles nothing, and so does a
/// `Method~new` carrying the optional third argument -- a package, measured
/// rc 0, where anything else is rc 40.
fn native_executable_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let name = executable_name_argument(interp, args)?;
    let Some(source) = args.get(1).copied().flatten() else {
        return Err(Raised::missing_named_argument("source").into());
    };
    if args.len() > 2 || class != interp.method_class() {
        return Err(unbuilt_new(interp, class));
    }
    let name = interp.to_text(name).to_vec();
    let method = compile_method_source(interp, &name, source, "source")?;
    Ok(Some(method))
}

/// `Package~new(name, source, ...)`: the name is required, the source is not,
/// and this crate loads no package either way -- `PackageClass::newRexx`
/// (`classes/PackageClass.cpp:158`).
///
/// A constructor of its own rather than [`native_executable_new`]'s, because
/// an omitted source is a file to resolve and load here rather than the
/// 88.901 the other two raise -- measured, oracle rc 0, `.Package~new('p')`
/// answers `a Package`.
fn native_package_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    executable_name_argument(interp, args)?;
    Err(unbuilt_new(interp, class))
}

/// The `name` argument `Method`, `Routine` and `Package` share --
/// `stringArgument(pgmname, "name")`.
fn executable_name_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    match args.first().copied().flatten() {
        Some(name) => required_string_named_argument(interp, name, "name"),
        None => Err(Raised::missing_named_argument("name").into()),
    }
}

/// `String~new(value, ...)`: a string carrying `value`'s own bytes --
/// `RexxString::newRexx` (`classes/StringClass.cpp:2352`).
///
/// **A fresh string and not the argument**, which the C++ says is so that the
/// class can be adjusted (`:2365`-`:2368`). The `INIT` send takes the
/// arguments past the first, so `.String~new('abc', 'x')` reaches
/// `Object~init` with one argument -- measured, oracle rc 163, `93.902 Too
/// many arguments in invocation of method; 0 expected.`
///
/// A subclass of `String` refuses: `completeNewObject` would give the string
/// that subclass's behaviour, and a string this crate builds carries no class
/// of its own.
fn native_string_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let Some(Some(value)) = args.first().copied() else {
        return Err(Raised::missing_method_argument(1).into());
    };
    let text = required_string_argument(interp, value, 1)?;
    if class != interp.object_model().string {
        return Err(unbuilt_new(interp, class));
    }
    let bytes = interp.to_text(text).to_vec();
    let object = interp.text_built(bytes);
    interp.roots.push_temp(object);
    let caller = interp.caller();
    interp.send_message(object, INIT, None, &args[1..], caller)?;
    Ok(Some(object))
}

/// `Stem~new(name, ...)`: a stem object whose name is the optional argument
/// -- `StemClass::newRexx` (`classes/StemClass.cpp:92`).
///
/// The constructor sets both `stemName` and `value` from that argument and
/// leaves the stem dropped (`:118`-`:128`), which is what `default: None`
/// means here: an unset stem renders as its own name. Measured, oracle rc 0:
/// `say '[' || .Stem~new || ']'` is `[]` and the same with `('FOO.')` is
/// `[FOO.]`.
///
/// A subclass of `Stem` refuses, for [`native_string_new`]'s reason: the body
/// this builds carries no class of its own.
fn native_stem_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let name = match args.first().copied().flatten() {
        Some(value) => {
            let text = required_string_argument(interp, value, 1)?;
            interp.to_text(text).into_owned()
        }
        None => Vec::new(),
    };
    if class != interp.object_model().stem {
        return Err(unbuilt_new(interp, class));
    }
    let object = interp.alloc_with(
        rexx_core::BehaviourId::STEM,
        Body::Stem {
            name: name.into(),
            default: None,
            tails: rexx_core::NameMap::default(),
        },
    );
    interp.roots.push_temp(object);
    let caller = interp.caller();
    let rest = args.get(1..).unwrap_or_default();
    interp.send_message(object, INIT, None, rest, caller)?;
    Ok(Some(object))
}

/// `WeakReference~new(value, ...)`: an instance that does not hold the value
/// -- `WeakReference::newRexx` (`classes/WeakReferenceClass.cpp:231`).
///
/// The referent is not kept. Measured, oracle rc 163: `.WeakReference~new`
/// is `93.903 Missing argument in method; argument 1 is required.`
fn native_weak_reference_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    if args.first().copied().flatten().is_none() {
        return Err(Raised::missing_method_argument(1).into());
    }
    let object = new_instance(interp, class)?;
    let caller = interp.caller();
    let rest: Vec<Option<ObjRef>> = args.iter().skip(1).copied().collect();
    interp.send_message(object, INIT, None, &rest, caller)?;
    Ok(Some(object))
}

/// `Object~request(class)`: the receiver converted to `class`, or `.nil` --
/// `RexxObject::requestRexx` (`classes/ObjectClass.cpp:1912`).
///
/// The rule is `MAKE` + the upcased class name looked up in the receiver's own
/// behaviour and sent if it is there; failing that, the receiver itself when
/// the name matches its own class's id; failing that, `.nil`. Bug #1904's own
/// comment fixes that order -- the `MAKE` method comes first because it can do
/// more than hand back the same object.
///
/// Measured, oracle rc 0: `.K~request("STRING")` is `The NIL object` without a
/// `makeString` and `K says hello` with one; `'abc'~request("STRING")` is
/// `abc`; `5~request("STRING")` is `5`; `.environment~request("STRING")` is
/// `The NIL object`; `.Array~request("CLASS")` is `The Array class`, which is
/// the id-match limb; `.K~request("K")` and `.K~request(5)` are both
/// `The NIL object`; and the argument is case-insensitive, so
/// `.K~request("string")` answers what `.K~request("STRING")` does.
///
/// **A `MAKE` method the class dictionaries declare and this crate has no code
/// for is a loud refusal rather than `.nil`**, which is what keeps the
/// unimplemented conversions from becoming wrong answers: `.environment
/// ~request("ARRAY")` is an array on the oracle, and `Directory`'s behaviour
/// really does answer `MAKEARRAY` -- measured,
/// `.environment~hasMethod("MAKEARRAY")` is `1` where `.K~hasMethod
/// ("MAKEARRAY")` is `0`, so the lookup below tells the two apart and only the
/// second reaches the `.nil`.
fn native_request(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_method_argument(1).into());
    };
    // `stringArgument(name, ARG_ONE)->upper()`, which is `requestRexx`'s own
    // first line -- so a class name with no string value is 88.909 here and
    // not `.nil`.
    let argument = required_string_argument(interp, argument, 1)?;
    let wanted = interp.to_text(argument).to_ascii_uppercase();
    let mut make = b"MAKE".to_vec();
    make.extend_from_slice(&wanted);
    if interp.lookup(receiver, &make, None).is_some() {
        let caller = interp.caller();
        // `resultOrNil`: a `MAKE` method that returns nothing answers `.nil`
        // rather than leaving the send with no value.
        let sent = interp.send_message(receiver, &make, None, &[], caller)?;
        return Ok(Some(sent.unwrap_or(ObjRef::NIL)));
    }
    Ok(Some(match interp.receiver_class_id(receiver) {
        Some(id) if id.as_bytes().to_ascii_uppercase() == wanted => receiver,
        Some(_) | None => ObjRef::NIL,
    }))
}

/// `String~makeString`: a string is its own string value --
/// `RexxString::makeString` (`classes/StringClass.hpp`), which
/// `Object~request("STRING")` is what finds.
///
/// Measured, oracle rc 0: `'abc'~makeString` is `abc` and
/// `'abc'~hasMethod("MAKESTRING")` is `1`, where
/// `.environment~hasMethod("MAKESTRING")` is `0`.
fn native_string_make_string(
    _interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(receiver))
}

/// `String~upper([n [, length]])`: the receiver with a range of it
/// uppercased -- `RexxString::upperRexx` (`classes/StringClass.cpp:1765`),
/// bound at `memory/Setup.cpp:681` with a declared count of 2.
///
/// The body is `upperRexx`'s, and the no-op cases are its own: a start past
/// the end of the string, a zero-length range, and a range capped at what is
/// left. Measured, oracle rc 0: `'abcdef'~upper` is `ABCDEF`,
/// `'abcdef'~upper(3)` is `abCDEF`, `'abcdef'~upper(3,2)` is `abCDef`,
/// `'abcdef'~upper(9)` and `'abcdef'~upper(3,0)` are both `abcdef` unchanged,
/// and `'abcdef'~upper(,2)` is `ABcdef` -- an omitted first argument takes the
/// default rather than shifting the second.
///
/// **`to_ascii_uppercase` and not a locale fold**, which is `Utilities::
/// toUpper`: measured, `'e9'x~upper` answers its own byte back.
///
/// The receiver is a string by dispatch -- this row is in `String`'s own
/// dictionary -- so `to_text` is its value and no required-string protocol
/// runs. Measured, `1234~upper` is `1234`.
fn native_string_upper(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let start = match whole_method_argument(interp, args, 0, Raised::invalid_position)? {
        Some(value) if value > 0 => {
            usize_or_refuse(interp, args, 0, value, Raised::invalid_position)?
        }
        Some(_) => {
            return Err(refuse_method_argument(
                interp,
                args,
                0,
                Raised::invalid_position,
            ));
        }
        None => 1,
    } - 1;
    let text = interp.to_text(receiver).into_owned();
    let range = match whole_method_argument(interp, args, 1, Raised::invalid_length)? {
        Some(value) if value >= 0 => {
            usize_or_refuse(interp, args, 1, value, Raised::invalid_length)?
        }
        Some(_) => {
            return Err(refuse_method_argument(
                interp,
                args,
                1,
                Raised::invalid_length,
            ));
        }
        None => text.len(),
    };
    if start >= text.len() {
        return Ok(Some(interp.text(&text)));
    }
    let range = range.min(text.len() - start);
    if range == 0 {
        return Ok(Some(interp.text(&text)));
    }
    let mut result = text;
    for byte in &mut result[start..start + range] {
        *byte = byte.to_ascii_uppercase();
    }
    Ok(Some(interp.text_built(result)))
}

/// One `optionalPositionArgument`/`optionalLengthArgument` conversion: the
/// argument at `index` as a whole number, `None` for an omitted or absent
/// position, and `raise`'s own condition for anything that is not one.
///
/// **The 93.9xx families a *method* raises name the object's own
/// `stringValue()`, never the converted value**, which is where they part
/// from the identically-numbered conditions the builtin layer raises.
/// Measured on the oracle, three descriptors: `'abcdef'~upper('0.0')` reports
/// `found "0.0"` and `'abcdef'~upper(' -1 ')` reports `found " -1 "`, where
/// `substr('abc','0.0')` reports the converted `found "0"`. A non-string
/// object is rendered the same way -- `'abcdef'~upper(.array)` reports
/// `found "The Array class"`.
fn whole_method_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
    raise: impl Fn(&[u8]) -> Raised + Copy,
) -> Result<Option<i64>, Failure> {
    let Some(Some(value)) = args.get(index).copied() else {
        return Ok(None);
    };
    match interp.to_number(value) {
        Ok(number) => match number.whole_value(METHOD_ARGUMENT_DIGITS) {
            Some(whole) => Ok(Some(whole)),
            None => Err(refuse_method_argument(interp, args, index, raise)),
        },
        Err(_) => Err(refuse_method_argument(interp, args, index, raise)),
    }
}

/// [`whole_method_argument`]'s refusal, split out because the range checks
/// after it raise the identical condition about the identical object.
fn refuse_method_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
    raise: impl Fn(&[u8]) -> Raised,
) -> Failure {
    let found = match args.get(index).copied().flatten() {
        Some(value) => interp.string_value_text(value),
        None => Vec::new(),
    };
    raise(&found).into()
}

/// A converted argument narrowed to a `usize`, or the same refusal a bad
/// range gets. `i64` values wider than a `usize` cannot arise on the
/// platforms this builds for, and answering the refusal rather than
/// truncating is what keeps that from being an assumption.
fn usize_or_refuse(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
    value: i64,
    raise: impl Fn(&[u8]) -> Raised,
) -> Result<usize, Failure> {
    usize::try_from(value).map_err(|_| refuse_method_argument(interp, args, index, raise))
}

/// `Numerics::ARGUMENT_DIGITS`, the precision every method-argument
/// conversion runs at -- `builtin.rs`'s own constant of the same value, kept
/// separate because the two layers reach it through different call paths and
/// a shared one would tie them together for no reason beyond the number
/// agreeing today.
const METHOD_ARGUMENT_DIGITS: usize = 18;

/// `Object~isNil`: `1` for `.nil` and `0` for everything else.
fn native_is_nil(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.counted(usize::from(receiver == ObjRef::NIL))))
}

/// `String~length`: the receiver's own byte count.
fn native_length(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let length = interp.text_len(receiver);
    Ok(Some(interp.counted(length)))
}

/// `file_separator`: the file system's name separator.
///
/// **The unix answer, which is the platform this crate is checked against.**
/// `SysFileSystem::getSeparator` returns `"/"`
/// (`platform/unix/SysFileSystem.cpp:1358`-`:1361`) and the windows half of
/// the platform layer is Phase 7's, unread and unbuilt here, exactly as Task
/// 23 embeds `platform/unix/PlatformObjects.orx` and no other. Measured,
/// oracle: a class method bound to this entry answers `/`.
fn native_file_separator(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.text_built(b"/".to_vec())))
}

/// `file_path_separator`: the separator between the entries of a search path.
///
/// `SysFileSystem::getPathSeparator` returns `":"`
/// (`platform/unix/SysFileSystem.cpp:1369`-`:1372`); see
/// [`native_file_separator`] for the platform note. Measured, oracle: a class
/// method bound to this entry answers `:`.
fn native_file_path_separator(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.text_built(b":".to_vec())))
}

/// `String~makeArray`: the receiver's lines, one array element each.
///
/// Measured on the oracle: `LF` separates and a trailing one terminates
/// rather than separating, so `'a' || LF` is one element and
/// `LF` alone is one empty element; a `CR` immediately before an `LF` is
/// dropped with it, while a lone `CR` is ordinary data; and the empty string
/// has no lines at all, answering an array of zero items.
fn native_string_makearray(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let bytes = interp.to_text(receiver).to_vec();
    let lines = crate::builtin::string::line_slices(&bytes);
    let slots: Vec<Option<ObjRef>> = lines
        .into_iter()
        .map(|line| Some(interp.text_built(line.to_vec())))
        .collect();
    // Measured, oracle: `''~makeArray~dimension` is 0, so an empty result
    // carries no dimensions rather than one of size 0.
    let body = Body::Array {
        dimensions: None,
        slots,
    };
    let object = interp.alloc_with(BehaviourId::ARRAY, body);
    interp.roots.push_temp(object);
    let caller = interp.caller();
    interp.send_message(object, INIT, None, &[], caller)?;
    Ok(Some(object))
}

/// `String~reverse`: the receiver's own bytes, last to first.
fn native_reverse(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let mut bytes = interp.to_text(receiver).to_vec();
    bytes.reverse();
    Ok(Some(interp.text_built(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The caller a resolution asked for outside any send is asking as:
    /// neither a receiver nor a package, which is the state
    /// `RexxObject::checkPrivate` and `RexxObject::checkPackage` each refuse
    /// (`classes/ObjectClass.cpp:622`-`:626`, `:665`-`:669`).
    fn no_caller() -> Caller {
        Caller {
            receiver: None,
            package: CallerPackage::NoActivation,
        }
    }

    /// The object model is built on first use and not in `Interp::new`.
    ///
    /// **This is the shape of the 5.5 ms measurement, asserted rather than
    /// left to a comment.** Had `Interp::new` built it, every program in the
    /// benchmark set would pay for a class registry it never reads; had the
    /// accessor not built it, `classes` would have nothing to look a class up
    /// in.
    #[test]
    fn the_object_model_is_built_on_first_use() {
        let mut interp = Interp::new();
        assert!(interp.object_model.is_none());
        let _ = interp.classes();
        assert!(interp.object_model.is_some());
    }

    /// A program that neither declares a class nor sends a message never
    /// touches the object model.
    #[test]
    fn a_program_with_no_send_and_no_class_never_builds_the_object_model() {
        let mut interp = Interp::new();
        let program = rexx_parse::parse_program(b"say 1 + 1\n".to_vec()).expect("it parses");
        interp
            .install_directives(crate::ProgramId(0), &std::rc::Rc::new(program))
            .expect("no directives to install");
        assert!(interp.object_model.is_none());
    }

    /// An ordinary resolution answers the class the definition came from,
    /// which is not always the receiver's own class: `LENGTH` is `String`'s
    /// and `ISNIL` is `Object`'s, and one string receiver reaches both.
    #[test]
    fn an_ordinary_resolution_answers_the_defining_scope() {
        let mut interp = Interp::new();
        let receiver = interp.text(b"abc");
        let string = interp.classes().lookup("String").expect("String is native");
        let object = interp.classes().lookup("Object").expect("Object is native");

        let own = interp
            .resolve(receiver, b"LENGTH", None, no_caller())
            .expect("String answers LENGTH");
        assert_eq!(own.scope, string);
        let inherited = interp
            .resolve(receiver, b"ISNIL", None, no_caller())
            .expect("Object's ISNIL reaches a String receiver");
        assert_eq!(inherited.scope, object);
        assert_eq!(
            interp
                .resolve(receiver, b"NOSUCHMETHOD", None, no_caller())
                .err(),
            Some(Miss::NoMethod)
        );
    }

    /// **The start-scope argument, at the seam rather than through a
    /// program.**
    ///
    /// `Interp::message_term` is what a `target~name:scope` send goes
    /// through, and it validates the scope against the receiver's own
    /// behaviour first, so a program cannot ask `resolve` about a start
    /// scope the receiver does not hold. This test calls `resolve`
    /// directly and can.
    ///
    /// The pair is what makes it mean something. `findSuperMethod` searches
    /// the starting scope itself plus the scopes folded in **ahead of** it,
    /// so on a `String` receiver:
    ///
    /// * starting at `Object`, `ISNIL` (defined there) is found and `LENGTH`
    ///   (defined at `String`, folded in after `Object`) is not;
    /// * starting at `String`, both are found.
    ///
    /// A `resolve` that ignored its start scope would answer `Some` for all
    /// four, and one that always answered `None` for a start scope would
    /// answer `None` for all four.
    #[test]
    fn a_start_scope_hides_a_method_defined_after_it_and_not_one_defined_before() {
        let mut interp = Interp::new();
        let receiver = interp.text(b"abc");
        let string = interp.classes().lookup("String").expect("String is native");
        let object = interp.classes().lookup("Object").expect("Object is native");

        assert!(
            interp
                .resolve(receiver, b"ISNIL", Some(object), no_caller())
                .is_ok()
        );
        assert!(
            interp
                .resolve(receiver, b"LENGTH", Some(object), no_caller())
                .is_err()
        );
        assert!(
            interp
                .resolve(receiver, b"ISNIL", Some(string), no_caller())
                .is_ok()
        );
        assert!(
            interp
                .resolve(receiver, b"LENGTH", Some(string), no_caller())
                .is_ok()
        );
    }

    /// A name the class answers with no implementation here is **loud**, and
    /// a name it does not answer is the oracle's own 97.1. The two are
    /// different answers and the pair is what keeps them apart: a build that
    /// raised for both would let a program expecting a working `String`
    /// method pass against a gap.
    ///
    /// **The loud name is chosen from the registry rather than written
    /// down**, so that implementing any one `String` method does not retire
    /// this test by making its example answer. The assertion that one was
    /// found is what stops it going vacuous the day the last row lands: it
    /// fails then, which is when it wants rewriting.
    #[test]
    fn an_unimplemented_method_is_loud_where_an_unknown_one_is_a_condition() {
        let mut interp = Interp::new();
        let receiver = interp.text(b"abc");
        let string = interp.classes().lookup("String").expect("String is native");
        let answered = interp.classes().instance_method_names(string);
        // **The candidate is found in the registry and only then sent.**
        // Sending every name until one refuses used to do it, and that stopped
        // working the moment a numeric method landed early in the order: a
        // body that reads `NUMERIC DIGITS` wants a live activation, and this
        // test has none, so the search panicked before it reached a loud name.
        let rows: Vec<&str> = NATIVE_METHODS
            .iter()
            .chain(string::NATIVE_METHODS)
            .chain(hash::NATIVE_METHODS)
            .chain(collection::NATIVE_METHODS)
            .filter(|(class, ..)| *class == "String")
            .map(|(_, method, ..)| *method)
            .collect();
        let mut unimplemented = None;
        for name in &answered {
            if rows.contains(&name.as_str()) {
                continue;
            }
            if matches!(
                interp.send_message(receiver, name.as_bytes(), None, &[], no_caller()),
                Err(Failure::Loud(_))
            ) {
                unimplemented = Some(name.clone());
                break;
            }
        }
        assert!(
            unimplemented.is_some(),
            "every name .String's behaviour answers now has an implementation, so this test \
             has no subject left and its pair has to be rebuilt on something else"
        );
        assert!(matches!(
            interp.send_message(receiver, b"NOSUCHMETHOD", None, &[], no_caller()),
            Err(Failure::Raised(_))
        ));
    }

    /// **A class identity used as a receiver resolves against that class's
    /// own class behaviour, never through the arena.**
    ///
    /// Before class identities moved to `rexx_core::CLASS_SLOT_BASE`'s
    /// reserved range, a class handle decoded as an ordinary heap handle,
    /// `heap.get` answered whatever object held that slot, and the send
    /// resolved against **that value's** class and answered from it: a silent
    /// wrong answer. `LENGTH` is `String`'s *instance* method and no class
    /// answers it on the class side, so the correct answer is 97.1 -- and
    /// under the collision the answer is `17`, measured: the length of a
    /// string the program never named.
    ///
    /// **The arrangement below is what makes the send the catcher**, and
    /// getting it wrong is what a first version of this test did. `.String`
    /// is the third identity the registry mints, so under the old minting its
    /// handle is the arena's **slot 2** -- filling slot 0 alone leaves slot 2
    /// empty, `heap.get` answers `None`, and the send fails for the wrong
    /// reason. With the arena filled up to and including that slot, the two
    /// mintings give visibly different answers. There is deliberately no
    /// `class_id()` precondition here, so that reverting the minting fails
    /// this test at the send rather than ahead of it.
    ///
    /// **The pair is what pins it to the class side rather than to "a class
    /// object answers nothing".** `HASMETHOD` is `Object`'s instance method
    /// and reaches the class object through the metaclass merge, so it
    /// answers -- and it answers about the *class* behaviour, measured on the
    /// oracle: `::class K` plus `::method m class` gives
    /// `.K~hasMethod('M')` = `1` and `.K~hasMethod('LENGTH')` = `0`.
    #[test]
    fn a_class_identity_used_as_a_receiver_answers_from_the_class_side() {
        let mut interp = Interp::new();
        // Each is longer than a handle can hold, so each really takes a slot
        // instead of travelling inline.
        let mut last = ObjRef::NIL;
        for text in [
            &b"first-occupant!"[..],
            b"second-occupant!",
            b"third-occupant!!!",
        ] {
            last = interp.text(text);
            assert!(
                interp.heap.get(last).is_some(),
                "the occupant went inline instead of into a slot"
            );
        }
        let Decoded::Heap { slot, .. } = last.decode() else {
            panic!("an arena handle")
        };
        assert_eq!(
            slot, 2,
            "this test rests on the third occupant holding the slot .String's identity would \
             have been equal to; the arena's allocation order moved"
        );

        let class = interp.classes().lookup("String").expect("String is native");
        assert!(matches!(
            interp.send_message(class, b"LENGTH", None, &[], no_caller()),
            Err(Failure::Raised(_))
        ));
        let name = interp.text(b"LENGTH");
        assert_eq!(
            interp
                .send_message(class, b"HASMETHOD", None, &[Some(name)], no_caller())
                .expect("Object's HASMETHOD reaches a class object"),
            Some(interp.counted(0)),
            "a class object was asked about its instances' behaviour instead of its own"
        );
    }

    /// Runs `source` on both engines and hands back `(exit code, stdout,
    /// stderr)`, having first insisted the two engines agree with each other.
    ///
    /// Both arms, because a method body is entered from
    /// `Interp::message_term`, which the compiled instruction op
    /// (`crate::ir::Op::Message`) and the tree-walker's own
    /// `ExprKind::Message` arm both reach, and because the body's own clauses
    /// are then driven by whichever engine is running.
    fn both_engines(source: &str) -> (i32, String, String) {
        let mut answer = None;
        for engine in [crate::Engine::TreeWalker, crate::Engine::Ir] {
            let outcome = crate::run_program(
                "/t.rex",
                source.as_bytes().to_vec(),
                crate::Invocation::none().with_engine(engine),
            );
            let seen = (
                outcome.exit_code,
                String::from_utf8_lossy(&outcome.stdout).into_owned(),
                String::from_utf8_lossy(&outcome.stderr).into_owned(),
            );
            match &answer {
                None => answer = Some(seen),
                Some(first) => assert_eq!(first, &seen, "the two engines disagree on {source:?}"),
            }
        }
        answer.expect("at least one engine ran")
    }

    /// **D24's `SmallInt` behaviour arm is taken for a small integer
    /// receiver**, where the general path is what a receiver whose bytes are
    /// in the arena takes.
    ///
    /// The pair is what makes it mean something, and each half fails on its
    /// own kind of mistake:
    ///
    /// * the two receivers answer **different** kinds, so folding the tag
    ///   into `Primitive::String` reddens the first assertion;
    /// * they answer the **same** behaviour, so an arm that pointed the tag
    ///   at another class reddens the second -- and that half is why no
    ///   differential row moves and why the split is structural rather than
    ///   an optimisation: `RexxInteger`'s own id is `String`
    ///   (`classes/IntegerClass.cpp:2066`), and measured, `12345~class~id` is
    ///   `String` and `12345~length` is 5 on the oracle.
    #[test]
    fn a_small_integer_receiver_takes_the_small_int_arm() {
        let mut interp = Interp::new();
        let integer = ObjRef::small_int(12345).expect("12345 fits the tag");
        assert!(
            matches!(integer.decode(), Decoded::SmallInt(_)),
            "the value model stopped holding 12345 in the tag, so this test's subject is gone"
        );
        let text = interp.text(b"12345");
        assert!(
            matches!(text.decode(), Decoded::Text(_)),
            "the same digits as bytes went somewhere other than the handle's text arm"
        );

        assert_eq!(
            interp
                .receiver_kind(integer)
                .expect("a small integer answers"),
            Primitive::SmallInt,
            "a tagged integer receiver went down the general path"
        );
        assert_eq!(
            interp.receiver_kind(text).expect("a text handle answers"),
            Primitive::String
        );

        let string = interp.classes().lookup("String").expect("String is native");
        let string_behaviour = Behaviour::Instance {
            methods: interp.classes().instance_behaviour_handle(string),
            owner: string,
        };
        assert_eq!(
            interp
                .receiver_behaviour(integer)
                .expect("a small integer resolves"),
            string_behaviour,
            "the arm sends a small integer's messages somewhere other than String"
        );
        assert_eq!(
            interp
                .receiver_behaviour(text)
                .expect("a text handle resolves"),
            string_behaviour
        );

        assert_eq!(
            both_engines("say 12345~length\n"),
            (0, "5\n".to_string(), String::new())
        );
    }

    /// **D24's receiver in the calling convention**: `SELF` is the object the
    /// send was addressed to, taken from `crate::CallContext`, and not the
    /// scope the resolution found the method at.
    ///
    /// The two part exactly where a method is inherited, which is what this
    /// program is for: `m` is defined at `K`, the send is to `J`, and the
    /// oracle answers `The J class` -- measured, rc 0, empty stderr. A
    /// `SELF` re-derived from the resolution would answer `The K class`, and
    /// a program whose receiver and scope are the same class cannot tell the
    /// two apart.
    ///
    /// The second row is the convention's other state: a frame that is not a
    /// method's carries no receiver, which is
    /// `RexxActivation::getReceiver`'s `OREF_NULL`
    /// (`execution/RexxActivation.cpp:2348`).
    #[test]
    fn a_method_send_binds_self_from_the_receiver_in_the_calling_convention() {
        assert_eq!(
            both_engines(
                "say .J~m\n\
                 ::class K\n\
                 ::method m class\n  return self\n\
                 ::class J subclass K\n"
            ),
            (0, "The J class\n".to_string(), String::new()),
            "SELF is not the receiver of the send"
        );

        let interp = Interp::new();
        assert!(
            interp.caller().receiver().is_none(),
            "a frame that is not a method's carries a receiver"
        );
    }

    /// **`SELF` and `SUPER` are bound before the body's first instruction**,
    /// to the receiver and to the scope after the method's own.
    ///
    /// Measured on the oracle, one class method of `::class K`: `say self`
    /// is `The K class` and `say super` is `The Class class` -- the class
    /// behaviour folds `Object`, then `Class`, then `K`, so the scope after
    /// `K` is `Class`. A build that bound neither would print the two
    /// variables' own derived names, `SELF` and `SUPER`.
    ///
    /// The second program is the reason the binding goes through
    /// `Interp::slot_of` rather than through the plan alone: the outer body
    /// mentions neither name, so the plan carries no slot for either, and an
    /// `INTERPRET` that reads one has to find it anyway.
    #[test]
    fn self_and_super_are_bound_before_the_bodys_first_instruction() {
        assert_eq!(
            both_engines(
                "say .K~m\n\
                 ::class K\n\
                 ::method m class\n  return self '/' super\n"
            ),
            (
                0,
                "The K class / The Class class\n".to_string(),
                String::new()
            )
        );
        assert_eq!(
            both_engines(
                "say .K~m\n\
                 ::class K\n\
                 ::method m class\n  interpret \"zz = self '/' super\"\n  return zz\n"
            ),
            (
                0,
                "The K class / The Class class\n".to_string(),
                String::new()
            )
        );
    }

    /// A native method's argument that has no string value raises 88.909,
    /// and the neighbouring arguments that have one still answer.
    ///
    /// **Ungated, where the corpus witness is not.**
    /// `corpus/lang/message_send_argument_object_not_a_string.rex` runs these
    /// same shapes against the live oracle, and it only fails a run under
    /// `REXX_CORPUS_GATE`; an argument answered instead of raised passes every
    /// other gate command without this case.
    ///
    /// The pair is what pins the rule to "has a string value" rather than to
    /// "is not a heap object": a stem answers *as* its default, so the same
    /// handle shape is on both sides of the line depending on what it holds.
    #[test]
    fn an_argument_with_no_string_value_raises_where_one_with_a_string_value_answers() {
        for source in [
            "say 'abc'~hasMethod(.String)\n",
            "say 'abc'~hasMethod(.environment)\n",
            "say 'abc'~hasMethod(.nil)\n",
            "a. = .array\nsay 'abc'~hasMethod(a.)\n",
            "a. = .nil\nsay 'abc'~hasMethod(a.)\n",
            "say .K~hasMethod(.String)\n::class K\n",
        ] {
            let (code, stdout, stderr) = both_engines(source);
            assert_eq!((code, stdout.as_str()), (168, ""), "{source:?}");
            assert!(
                stderr.contains("Error 88.909:  Argument 1 must have a string value."),
                "{source:?} raised {stderr:?}"
            );
            assert!(
                stderr.contains("Compiled method \"HASMETHOD\" with scope \"Object\"."),
                "{source:?} lost the method's own traceback line: {stderr:?}"
            );
        }

        for (source, expected) in [
            ("say 'abc'~hasMethod('LENGTH')\n", "1\n"),
            ("say 'abc'~hasMethod(5)\n", "0\n"),
            ("a. = 'LENGTH'\nsay 'abc'~hasMethod(a.)\n", "1\n"),
            ("say 'abc'~hasMethod(d.)\n", "0\n"),
        ] {
            assert_eq!(
                both_engines(source),
                (0, expected.to_string(), String::new()),
                "{source:?}"
            );
        }
    }

    /// The `::METHOD` and `::ATTRIBUTE` shapes whose body this crate cannot
    /// run are **loud**, and the neighbouring shapes that it can run are not.
    ///
    /// The refusals cannot be corpus programs, which have to match the
    /// oracle; each row's comment carries what the oracle answers instead.
    /// The successes below them are what stops "refuse every directive
    /// option" from passing: `PROTECTED`, `PACKAGE` and `UNGUARDED` run on the
    /// oracle and must run here, and a `::ATTRIBUTE GET` with a body of its
    /// own is the one attribute form that is a written method rather than a
    /// generated accessor.
    ///
    /// `PRIVATE` is not a row in either list, because it is neither: the
    /// oracle answers it or refuses it depending on who is sending, and
    /// [`Interp::private_sends_are_refused_by_who_is_sending`] is the test
    /// that separates the two.
    ///
    /// [`Interp::private_sends_are_refused_by_who_is_sending`]:
    ///     private_sends_are_refused_by_who_is_sending
    #[test]
    fn a_method_body_this_crate_cannot_run_is_loud_and_its_neighbours_still_run() {
        // (source, the refusal's own text after `rexx-exec: `). The tail
        // differs by row and is not a shared suffix: `Loud::method_body`
        // names Phase 5 as the owner and `Loud::accessor_variable` names
        // none, on the reasoning that constructor's doc gives.
        let refused: &[(&str, &str)] = &[
            // A generated accessor over a variable that is not a simple name.
            // Oracle rc 0 both: the stem answers `5` for the round trip and
            // the compound answers its own derived name `a.b`.
            (
                ".K~'A.' = 5\nsay .K~'A.'\n::class K\n::attribute \"a.\" class\n",
                "a generated accessor for the attribute \"a.\" is not implemented",
            ),
            (
                "say .K~'A.B'\n::class K\n::attribute \"a.b\" class\n",
                "a generated accessor for the attribute \"a.b\" is not implemented",
            ),
            // oracle rc 0, printing `4`: `USE LOCAL` binds its list against
            // the method's own scope pool.
            (
                "say .K~m\n::class K\n::method m class\n  use local zz\n  zz = 4\n  return zz\n",
                "USE LOCAL in a ::METHOD body is not implemented (Phase 5)",
            ),
        ];
        for (source, refusal) in refused {
            let (code, stdout, stderr) = both_engines(source);
            assert_eq!(
                (code, stdout.as_str(), stderr.as_str()),
                (
                    crate::NOT_IMPLEMENTED_EXIT,
                    "",
                    format!("rexx-exec: {refusal}\n").as_str()
                ),
                "{source:?}"
            );
        }

        for (source, expected) in [
            (
                "say .K~m\n::class K\n::method m class protected\n  return 7\n",
                "7\n",
            ),
            (
                "say .K~m\n::class K\n::method m class unguarded\n  return 7\n",
                "7\n",
            ),
            (
                "say .K~m\n::class K\n::method m class package\n  return 7\n",
                "7\n",
            ),
            (
                "say .K~a\n::class K\n::attribute a class get\n  return 11\n",
                "11\n",
            ),
        ] {
            assert_eq!(
                both_engines(source),
                (0, expected.to_string(), String::new()),
                "{source:?}"
            );
        }
    }

    /// **A `DELEGATE` directive installs a forwarding method under every key
    /// it claims**, including the setter half of the pair a `ATTRIBUTE`
    /// modifier adds, and every row's bytes are the oracle's own.
    ///
    /// **These rows were the refusal list's above until `DELEGATE` was
    /// built**, and they are here rather than in the corpus because the
    /// receiver is a class object whose delegate property is never assigned:
    /// the answer is a refusal from the *delegate*, which is what says the
    /// message reached it. The message name each row reports is the one the
    /// send used, so the setter row is the only instrument anywhere for the
    /// `A=` key -- a build that installed the getter alone would make
    /// `.K~a = 5` a name miss on the class and report `Object "The K class"`
    /// instead, the oracle's status and catalogue row over a receiver the
    /// oracle does not name.
    ///
    /// Table D's two rows send through an *instance* and answer rather than
    /// refuse, so they cover the arm this one cannot and the two are not
    /// substitutes.
    #[test]
    fn a_delegate_directive_forwards_under_every_key_it_claims() {
        for (source, message) in [
            ("say .K~m\n::class K\n::method m class delegate p\n", "M"),
            ("say .K~a\n::class K\n::attribute a class delegate p\n", "A"),
            (
                "say .K~a\n::class K\n::method a class delegate p attribute\n\
                 ::attribute p class\n",
                "A",
            ),
            (
                ".K~a = 5\nsay 'stored'\n::class K\n\
                 ::method a class delegate p attribute\n::attribute p class\n",
                "A=",
            ),
        ] {
            let clause = source.lines().next().expect("a first clause");
            let expected = format!(
                "     1 *-* {clause}\n\
                 Error 97 running /t.rex line 1:  Object method not found.\n\
                 Error 97.1:  Object \"P\" does not understand message \"{message}\".\n"
            );
            assert_eq!(
                both_engines(source),
                (159, String::new(), expected),
                "{source:?}"
            );
        }
    }

    /// **A generated accessor pair reads and writes one variable in the
    /// declaring scope's pool on the receiver**, and every row is measured on
    /// the oracle.
    ///
    /// **What catches a regression here**, stated because this replaces a
    /// loud refusal and the corpus gate cannot see a refusal becoming a wrong
    /// answer: this test,
    /// `corpus/lang/method_attribute_generated.rex` and the
    /// `method_attribute_generated_*` programs beside it, and the table D rows
    /// `corpus/gate-tables/directives/attribute__class__subkeyword.rex` and
    /// `method__attribute__subkeyword.rex`. The instance reading of every row
    /// below is out of reach until something builds instances, so a wrong
    /// answer to a send whose receiver is not a class object is caught by
    /// nothing here.
    ///
    /// Both directives generate the pair, which is why the first group has a
    /// `::ATTRIBUTE` row and a `::METHOD ... ATTRIBUTE` row.
    #[test]
    fn a_generated_accessor_pair_reads_and_writes_the_declaring_scopes_pool() {
        for (source, expected) in [
            // The round trip, through each directive.
            (
                ".K~a = 5\nsay .K~a\n::class K\n::attribute a class\n",
                "5\n",
            ),
            (
                ".K~a = 5\nsay .K~a\n::class K\n::method a class attribute\n",
                "5\n",
            ),
            // An unassigned variable reads as its derived name, which is the
            // directive's name **as written** and not the accessor's upcased
            // dictionary key: `getRetriever(name)` against
            // `addMethod(internalname)`. The quoted row is what tells the two
            // apart -- an accessor keyed on the message name would answer
            // `AB` here.
            ("say .K~a\n::class K\n::attribute a class\n", "A\n"),
            ("say .K~ab\n::class K\n::attribute \"aB\" class\n", "aB\n"),
            // And it is the same pool entry `EXPOSE` reaches, in both
            // directions.
            (
                ".K~a = 'through the setter'\nsay .K~read\n::class K\n::attribute a class\n\
                 ::method read class\n  expose a\n  return a\n",
                "through the setter\n",
            ),
            (
                "x = .K~write\nsay .K~a\n::class K\n::attribute a class\n\
                 ::method write class\n  expose a\n  a = 'through EXPOSE'\n  return 1\n",
                "through EXPOSE\n",
            ),
            // **Keyed on the declaring scope and on the receiver**, which is
            // one property from both sides. `.J~a` and `.K~a` name different
            // receivers and so different pools: measured, the second answers
            // its derived name after the first was assigned.
            (
                ".J~a = 5\nsay .J~a .K~a\n::class K\n::attribute a class\n\
                 ::class J subclass K\n",
                "5 A\n",
            ),
            // A value is stored, not a rendering of one: the receiver goes in
            // and comes back out.
            (
                ".K~a = .K\nsay .K~a\n::class K\n::attribute a class\n",
                "The K class\n",
            ),
            // `GET` and `SET` each generate one half, and the generated half
            // reads the same pool a generated pair does.
            ("say .K~a\n::class K\n::attribute a class get\n", "A\n"),
            // A trailing omission leaves the getter's own bound satisfied.
            ("say .K~a(,)\n::class K\n::attribute a class\n", "A\n"),
            (
                ".K~a = 5\nsay 'stored'\n::class K\n::attribute a class set\n",
                "stored\n",
            ),
        ] {
            assert_eq!(
                both_engines(source),
                (0, expected.to_string(), String::new()),
                "{source:?}"
            );
        }

        // **Neither `GET` nor `SET` installs the other half**, so the
        // message the directive did not generate is a name miss. Measured at
        // rc 159, one program each.
        for (source, missing) in [
            (
                ".K~a = 5\nsay 'x'\n::class K\n::attribute a class get\n",
                "A=",
            ),
            ("say .K~a\n::class K\n::attribute a class set\n", "A"),
        ] {
            let (code, stdout, stderr) = both_engines(source);
            assert_eq!((code, stdout.as_str()), (159, ""), "{source:?}");
            assert!(
                stderr.contains(&format!(
                    "Error 97.1:  Object \"The K class\" does not understand message \
                     \"{missing}\"."
                )),
                "{source:?} reported {stderr:?}"
            );
        }

        // The setter answers nothing, which a program can read: measured,
        // `91.999` at rc 165 rather than a value the assignment produced.
        let (code, stdout, stderr) =
            both_engines("r = .K~'A='(9)\nsay r\n::class K\n::attribute a class\n");
        assert_eq!((code, stdout.as_str()), (165, ""));
        assert!(
            stderr.contains("Error 91.999:  Message \"A=\" did not return a result."),
            "the setter answered a value: {stderr:?}"
        );

        // The argument bounds, which are the accessor pair's own and are the
        // C++'s (`execution/CPPCode.cpp:284`, `:334`, `:339`). Each row is
        // measured at rc 163.
        for (source, catalogue) in [
            (
                "say .K~a(1)\n::class K\n::attribute a class\n",
                "Error 93.902:  Too many arguments in invocation of method; 0 expected.",
            ),
            (
                "say .K~'A='(1,2)\n::class K\n::attribute a class\n",
                "Error 93.902:  Too many arguments in invocation of method; 1 expected.",
            ),
            (
                "say .K~'A='()\n::class K\n::attribute a class\n",
                "Error 93.903:  Missing argument in method; argument 1 is required.",
            ),
            // **A trailing omission is not an argument that arrived, and a
            // leading one is.** Measured, both at rc 163: `(,)` reaches the
            // setter as no arguments at all, and `(,5)` as two. The getter's
            // side of the same rule is the `(,)` row below, which answers.
            (
                "say .K~'A='(,)\n::class K\n::attribute a class\n",
                "Error 93.903:  Missing argument in method; argument 1 is required.",
            ),
            (
                "say .K~'A='(,5)\n::class K\n::attribute a class\n",
                "Error 93.902:  Too many arguments in invocation of method; 1 expected.",
            ),
        ] {
            let (code, stdout, stderr) = both_engines(source);
            assert_eq!((code, stdout.as_str()), (163, ""), "{source:?}");
            assert!(stderr.contains(catalogue), "{source:?} reported {stderr:?}");
            // **No frame of its own**, which is what separates an accessor
            // from a `NativeMethod` taking the same refusal: measured,
            // `'abc'~length(1)` carries a `Compiled method "LENGTH"` line
            // above the sending clause and none of these do.
            assert!(
                !stderr.contains("Compiled method"),
                "{source:?} grew a traceback frame the oracle does not write: {stderr:?}"
            );
        }
    }

    /// **An `ABSTRACT` method installs and is refused when it is sent**,
    /// naming the message rather than the directive.
    ///
    /// The instrument, stated because this replaces a loud refusal: **this
    /// test and the table D rows** `method__abstract__subkeyword.rex`
    /// and `attribute__abstract__subkeyword.rex`, plus
    /// `corpus/lang/method_abstract_send.rex`. Gate table C cannot see it --
    /// its `abscla` row records that the abstract-*method* half has no arm of
    /// that section's probe -- so nothing derived covers this.
    ///
    /// Every row is measured on the oracle at rc 163. Between them they
    /// cover each directive that can install an `ABSTRACT` method, the
    /// as-written and upcased spellings of a name, and both halves of an
    /// abstract accessor pair -- the setter's message carrying the appended
    /// `=` is what shows the pair is installed rather than a single name.
    #[test]
    fn an_abstract_send_is_refused_at_the_send_naming_the_message() {
        for (source, named) in [
            ("say .K~m\n::class K\n::method m class abstract\n", "M"),
            (
                "say .K~\"MiXeD\"\n::class K\n::method \"MiXeD\" class abstract\n",
                "MIXED",
            ),
            (
                "say .K~m\n::class K\n::method m class abstract attribute\n",
                "M",
            ),
            ("say .K~a\n::class K\n::attribute a class abstract\n", "A"),
            (".K~a = 3\n::class K\n::attribute a class abstract\n", "A="),
        ] {
            let (code, stdout, stderr) = both_engines(source);
            assert_eq!((code, stdout.as_str()), (163, ""), "{source:?}");
            assert!(
                stderr.contains(&format!(
                    "Error 93.965:  Method {named} is ABSTRACT and cannot be directly invoked."
                )),
                "{source:?} reported {stderr:?}"
            );
            assert!(
                !stderr.contains("Compiled method"),
                "{source:?} grew a traceback frame the oracle does not write: {stderr:?}"
            );
        }

        // **Installing one is not sending one**: the directive is rc 0 with
        // the program's own output, which is what makes the row above a send
        // refusal rather than an install refusal. Measured on the oracle.
        assert_eq!(
            both_engines("say 'installed'\n::class K\n::method m class abstract\n"),
            (0, "installed\n".to_string(), String::new())
        );
    }

    /// **A private send is refused by who is sending, and the refusal names
    /// the scope that refused it.**
    ///
    /// This is the instrument the corpus gate cannot be, and the reason is
    /// the shape of the gate rather than a gap in the programs: a refusal
    /// this crate shares with the oracle is a corpus row, but the failure
    /// this task risks is a refusal quietly becoming an *answer*, and the
    /// moment that happens the row's exit status stops being a refusal's at
    /// all. So the refusals are asserted here by their catalogue coordinates,
    /// against the sends that must keep answering.
    ///
    /// Each refusing row is a distinct limb of `checkPrivate`: a program
    /// frame carries no receiver, a routine frame carries none either, and a
    /// sibling class is a class object whose hierarchy does not contain the
    /// declaring scope. Each answering row is a distinct allowing limb.
    ///
    /// `97.2` rather than `97.1` is what separates a real access check from a
    /// build that dropped the method and let the name-miss report stand:
    /// measured on the oracle, `CONDITION('E')` under a `SIGNAL ON SYNTAX` is
    /// `2` for a refused private send and `1` for a name the behaviour does
    /// not answer.
    ///
    /// **Every row is a class method**, because reaching an instance method
    /// needs `~new`. The instance reading of each is untested rather than
    /// confirmed.
    #[test]
    fn private_sends_are_refused_by_who_is_sending() {
        let class = "\n::CLASS K\n::METHOD m CLASS PRIVATE\n  return 'inner'\n";
        for source in [
            // The program's own frame, which has no receiver.
            &format!("say .K~m{class}"),
            // A routine's frame, which has none either.
            &format!("say r(){class}::ROUTINE r\n  return .K~m\n"),
            // A sibling class in the same package.
            &format!("say .S~poke{class}::CLASS S\n::METHOD poke CLASS\n  return .K~m\n"),
        ] {
            let (code, stdout, stderr) = both_engines(source);
            assert_eq!((code, stdout.as_str()), (159, ""), "{source:?}");
            assert!(
                stderr.contains(
                    "Error 97.2:  Object \"The K class\" cannot accept private message \
                     \"M\" from this context."
                ),
                "a refused private send must report 97.2 naming the receiver and the \
                 message, got {stderr:?} for {source:?}"
            );
        }
        for source in [
            // The declaring class's own class method, sending to itself.
            &format!("say .K~outer{class}::METHOD outer CLASS\n  return self~m\n"),
            // A subclass's own class method, sending to itself: the same
            // object, and the limb a rule written over the defining scope
            // alone would refuse.
            "say .Sub~poke\n::CLASS Base\n::METHOD m CLASS PRIVATE\n  return 'inner'\n\
             ::CLASS Sub SUBCLASS Base\n::METHOD poke CLASS\n  return self~m\n",
            // A class object whose hierarchy contains the declaring scope,
            // sending to a different object.
            "say .Sub~poke\n::CLASS Base\n::METHOD m CLASS PRIVATE\n  return 'inner'\n\
             ::CLASS Sub SUBCLASS Base\n::METHOD poke CLASS\n  return .Base~m\n",
        ] {
            assert_eq!(
                both_engines(source),
                (0, "inner\n".to_string(), String::new()),
                "{source:?}"
            );
        }
    }

    /// **The refusal is not raised at the send**: it drops the method and
    /// enters the receiver's own `UNKNOWN`, and the `UNKNOWN` lookup is not
    /// itself access-checked.
    ///
    /// The pair is what makes each half mean something. A build that raised
    /// at the send answers neither row; a build that checked the `UNKNOWN`
    /// lookup too answers the first and refuses the second.
    #[test]
    fn a_refused_send_reaches_unknown_and_the_unknown_lookup_is_not_checked() {
        assert_eq!(
            both_engines(
                "say .K~m\n::CLASS K\n::METHOD m CLASS PRIVATE\n  return 'never'\n\
                 ::METHOD unknown CLASS\n  use arg name\n  return 'unknown saw' name\n"
            ),
            (0, "unknown saw M\n".to_string(), String::new())
        );
        assert_eq!(
            both_engines(
                "say .K~zork\n::CLASS K\n::METHOD unknown CLASS PRIVATE\n\
                 \x20 use arg name\n  return 'private unknown saw' name\n"
            ),
            (0, "private unknown saw ZORK\n".to_string(), String::new())
        );
    }

    /// **`PACKAGE`'s refusing arm**, called directly with the two callers the
    /// oracle refuses: one with no activation at all, which no program
    /// reaches, and one whose package is not the method's, which a
    /// `::REQUIRES` of a file declaring the method does --
    /// `corpus/refusal-sites.tsv`'s `package_scope_method` row names that
    /// program. The allowing arm has a corpus program
    /// (`corpus/lang/method_access_package_and_protected.rex`) and is
    /// asserted here too, so a check that refused everything fails rather
    /// than passing both refusals.
    #[test]
    fn package_scope_refuses_a_caller_from_another_package() {
        let method_package = Package::Program(crate::ProgramId(0));
        assert_eq!(
            Interp::check_package(method_package, no_caller()),
            Err(Miss::PackageScope),
            "a caller with no activation is `checkPackage`'s first refusal"
        );
        assert_eq!(
            Interp::check_package(
                method_package,
                Caller {
                    receiver: None,
                    package: CallerPackage::Package(Package::Program(crate::ProgramId(1))),
                },
            ),
            Err(Miss::PackageScope),
            "a caller in another program's package must be refused"
        );
        assert_eq!(
            Interp::check_package(
                method_package,
                Caller {
                    receiver: None,
                    package: CallerPackage::Package(method_package),
                },
            ),
            Ok(()),
            "the same package must be allowed"
        );
        assert_eq!(
            Interp::check_package(
                Package::Rexx,
                Caller {
                    receiver: None,
                    package: CallerPackage::Package(Package::Program(crate::ProgramId(0))),
                },
            ),
            Err(Miss::PackageScope),
            "the interpreter's own package is not a program's"
        );
    }

    /// **`checkPrivate`'s `isInstanceOf` limb, which no program in this phase
    /// can reach either.**
    ///
    /// The limb allows a send from another *instance* of the class that
    /// declared the method, and an instance needs `~new`. A class object
    /// cannot stand in for one: its own class is the metaclass, never the
    /// user class a `::METHOD ... PRIVATE` is declared in. So the check is
    /// called directly, with a string as the sender and `.String` as the
    /// declaring scope -- the one value kind this phase has whose class is a
    /// class the registry holds.
    ///
    /// The second half is the refusal that makes the first mean something:
    /// the same sender against a scope its class is not compatible with.
    #[test]
    fn a_private_send_from_another_instance_of_the_declaring_class_is_allowed() {
        let mut interp = Interp::new();
        let sender = interp.text(b"abc");
        let receiver = interp.classes().lookup("Array").expect("Array is native");
        let string = interp.string_class();
        let caller = Caller {
            receiver: Some(sender),
            package: CallerPackage::NoActivation,
        };
        // `MethodId` is not read by the check at all -- the scope is -- so
        // the resolution below names a method that exists for the receiver
        // and nothing rests on which one it is.
        let resolution = interp
            .resolve(sender, b"LENGTH", None, no_caller())
            .expect("String answers LENGTH");
        assert_ne!(sender, receiver, "the two must not be the same object");
        assert_eq!(
            interp.check_private(
                Resolution {
                    scope: string,
                    method: resolution.method
                },
                receiver,
                caller,
            ),
            Ok(()),
            "a sender whose class is the declaring scope must be allowed"
        );
        let object = interp.classes().lookup("Array").expect("Array is native");
        assert_eq!(
            interp.check_private(
                Resolution {
                    scope: object,
                    method: resolution.method
                },
                receiver,
                caller,
            ),
            Err(Miss::Private),
            "a sender whose class is not compatible with the declaring scope must be refused"
        );
    }

    /// A `target~name:scope` override answers **when the scope is a class
    /// object the receiver's behaviour holds**, is 88.914 when the scope is
    /// not a class object at all, and is 93.957 when it is a class the
    /// receiver's behaviour was never given.
    ///
    /// The three together are what keep each refusal from swallowing the
    /// next: 88.914 is `RexxExpressionMessage::evaluate`'s own
    /// `isInstanceOf(TheClassClass)` test (`ExpressionMessage.cpp:166`) and
    /// 93.957 is `validateScopeOverride`'s, and a version raising either for
    /// both cases passes half of this.
    ///
    /// Oracle-measured, all three: `'abc'~length:.String` is `3` at rc 0,
    /// `'abc'~length:super` outside a method is 88.914 at rc 168 because
    /// `SUPER` is then an ordinary uninitialised variable, and
    /// `'abc'~length:.Array` is 93.957 at rc 163.
    #[test]
    fn a_scope_override_answers_and_has_one_refusal_for_each_bad_scope() {
        assert_eq!(
            both_engines("say 'abc'~length:.String\n"),
            (0, "3\n".to_string(), String::new())
        );
        let (code, stdout, stderr) = both_engines("say 'abc'~length:super\n");
        assert_eq!((code, stdout.as_str()), (168, ""));
        assert!(
            stderr.contains("Error 88.914:"),
            "a non-class scope must keep the oracle's own condition, got {stderr:?}"
        );
        let (code, stdout, stderr) = both_engines("say 'abc'~length:.Array\n");
        assert_eq!((code, stdout.as_str()), (163, ""));
        assert!(
            stderr.contains(
                "Error 93.957:  Target object \"abc\" is not a subclass of the message \
                 override scope (The Array class)."
            ),
            "a class the receiver's behaviour does not hold must be 93.957, got {stderr:?}"
        );
    }

    /// A receiver whose class this phase does not build is loud too, and for
    /// the same reason: the oracle answers a send to a stem.
    #[test]
    fn a_receiver_with_no_class_here_is_loud() {
        let mut interp = Interp::new();
        let stem = interp.alloc_with(
            rexx_core::BehaviourId::STEM,
            Body::Stem {
                name: b"A."[..].into(),
                default: None,
                tails: rexx_core::NameMap::default(),
            },
        );
        assert!(matches!(
            interp.send_message(stem, b"LENGTH", None, &[], no_caller()),
            Err(Failure::Loud(_))
        ));
    }

    /// `~identityHash` answers, and **the corpus cannot witness it**: the
    /// oracle's answer is derived from the object's address, so no differential
    /// row can compare the two and this test is the whole instrument.
    ///
    /// What it pins is the half deviation 4 does not license away. The method
    /// answers rather than refusing; it answers something a Rexx program can
    /// use as a whole number; and equal handles answer equally while different
    /// handles do not, which is the identity model that deviation names -- so
    /// a build answering a constant answers and is usable, and still fails the
    /// row that asks two different handles.
    ///
    /// [`string_hash`] against the oracle's own answers, byte for byte.
    ///
    /// The signed-byte rows are the ones that matter: an unsigned
    /// accumulator is right for every ASCII string and wrong above `0x7f`,
    /// so a witness of letters alone cannot see it. `c2x` of the method's
    /// answer is this value little-endian.
    #[test]
    fn the_string_hash_matches_the_oracle_including_above_7f() {
        // Measured with `c2x(<x>~hashCode)`, read back as little-endian.
        assert_eq!(string_hash(b""), 0x0000_0000_0000_0000);
        assert_eq!(string_hash(b"a"), 0x0000_0000_0000_0061);
        assert_eq!(string_hash(b"abc"), 0x0000_0000_0001_7862);
        assert_eq!(string_hash(b"5"), 0x0000_0000_0000_0035);
        assert_eq!(string_hash(b"String"), 0x0000_0000_943a_4c31);
        // Wraps the 64-bit register rather than saturating.
        assert_eq!(
            string_hash(b"abcdefghijklmnopqrstuvwxyz0123456789"),
            0xa09f_2fd2_5824_3772
        );
        // Signed: one byte of 0xff contributes -1, not 255.
        assert_eq!(string_hash(b"\xff"), 0xffff_ffff_ffff_ffff);
        assert_eq!(string_hash(b"\x80"), 0xffff_ffff_ffff_ff80);
        assert_eq!(string_hash(b"\x7f"), 0x0000_0000_0000_007f);
    }

    /// `==` and not `=`, and `numeric digits 20` rather than the default:
    /// measured, the answer is wider than nine significant digits, so a
    /// numeric comparison at the default `DIGITS` rounds two different
    /// handles' answers together and reports them equal.
    #[test]
    fn identity_hash_answers_a_number_that_follows_the_handle() {
        assert_eq!(
            both_engines(
                "numeric digits 20\n\
                 say (.Array~identityHash == .Array~identityHash)\n\
                 say (.Array~identityHash == .String~identityHash)\n\
                 say datatype(.Array~identityHash, 'W')\n"
            ),
            (0, "1\n0\n1\n".to_string(), String::new())
        );
    }

    /// Every receiver kind answers `~identityHash`, and each line is the
    /// oracle's own answer, measured 2026-09-03 on three descriptors.
    ///
    /// **The rows below are the whole of what the licence leaves standing.**
    /// The *value* diverges and cannot be closed -- see
    /// [`native_identity_hash`] -- so a build answering a per-receiver
    /// constant would satisfy the rendering and still fail the last two rows.
    #[test]
    fn identity_hash_answers_every_receiver_kind_as_the_oracle_does() {
        assert_eq!(
            both_engines(
                "numeric digits 20\n\
                 s. = 1\n\
                 o = s.\n\
                 say datatype(o~identityHash, 'W') datatype('abc'~identityHash, 'W')\n\
                 say datatype(.Object~new~identityHash, 'W') \
                     datatype(.Array~new~identityHash, 'W') \
                     datatype(.StringTable~new~identityHash, 'W') \
                     datatype(.Directory~new~identityHash, 'W')\n\
                 a = .Object~new\n\
                 b = .Object~new\n\
                 say (a~identityHash == a~identityHash) (a~identityHash == b~identityHash)\n"
            ),
            (0, "1 1\n1 1 1 1\n1 0\n".to_string(), String::new())
        );
    }

    /// Two equal short strings are **one** object here and two on the oracle,
    /// which is deviation 4's identity half rather than its rendering half.
    ///
    /// A string of up to [`rexx_core::INLINE_TEXT`] bytes lives in the handle
    /// and allocates nothing, so two separate concatenations of equal value
    /// are the same handle. Measured 2026-09-03, oracle rc 0: both rows below
    /// are `0` there. The second is the boundary control -- one byte past the
    /// inline capacity and the two sides agree -- so the divergence is pinned
    /// to the inline case rather than to `~identityHash` at large.
    #[test]
    fn two_equal_inline_strings_share_one_handle() {
        assert_eq!(
            both_engines(
                "j = 5\n\
                 say ((\"eeeeee\"||j)~identityHash == (\"eeeeee\"||j)~identityHash)\n\
                 say ((\"eeeeeee\"||j)~identityHash == (\"eeeeeee\"||j)~identityHash)\n"
            ),
            (0, "1\n0\n".to_string(), String::new())
        );
    }

    /// The reflection protocol answers on both engines for a receiver that is
    /// not a class object, which is where `~class` and `~isA` differ from the
    /// `.Class`-scope methods beside them.
    ///
    /// `corpus/lang/class_reflection.rex` runs these same shapes against the
    /// live oracle. Here to keep the split loud without `REXX_CORPUS_GATE`: a
    /// build that answered `~class` from `receiver_kind`'s class arm alone
    /// would refuse every row below, and nothing outside the gate would say so.
    #[test]
    fn the_object_protocol_answers_a_receiver_that_is_not_a_class_object() {
        assert_eq!(
            both_engines(
                "say 'abc'~class~id\n\
                 say (12345)~class~id\n\
                 say .nil~class~id\n\
                 say .Class~superClasses~class~id\n\
                 say .Array~package~class~id\n\
                 say 'abc'~isA(.String) .nil~isA(.Object) 'abc'~isA(.Array)\n"
            ),
            (
                0,
                "String\nString\nObject\nArray\nPackage\n1 1 0\n".to_string(),
                String::new()
            )
        );
    }

    /// A name in `.Class`'s own dictionary does not reach a receiver that is
    /// not a class object.
    ///
    /// This is what makes [`class_receiver`]'s non-class arm unreachable, and
    /// asserting it here rather than asserting the arm is deliberate: reaching
    /// the arm needs a [`Cleared`] token, and building one outside
    /// [`Interp::invoke`] would add a second call to the dispatch seam, which
    /// `tests/dispatch_seam.rs` bounds at one.
    ///
    /// One row per receiver kind `Interp::receiver_kind` admits that is not a
    /// class object; that function is where membership is decided. `~id` is
    /// the name that would answer if the class-side and instance-side
    /// dictionaries were one.
    #[test]
    fn a_class_scope_name_does_not_reach_a_receiver_that_is_not_a_class_object() {
        for source in [
            "say 'abc'~id\n",
            "say .nil~id\n",
            "say .Class~superClasses~id\n",
            "say .Array~package~id\n",
            "say 'abc'~superClasses\n",
        ] {
            let (code, stdout, stderr) = both_engines(source);
            assert_eq!((code, stdout.as_str()), (159, ""), "{source:?}");
            assert!(
                stderr.contains("Error 97.1:"),
                "{source:?} answered or refused instead of raising: {stderr:?}"
            );
        }
    }
    /// Every class with a `NEW` row of its own answers an instance of the
    /// class the send was addressed to, or the oracle's own refusal.
    ///
    /// The refusing rows are what stop the pair from collapsing: a constructor
    /// that skipped its argument checks would answer an instance for all of
    /// them, and one that refused everything would answer none.
    #[test]
    fn a_primitive_constructor_answers_an_instance_or_the_oracle_s_own_refusal() {
        for class in [
            "Bag",
            "Directory",
            "EventSemaphore",
            "IdentityTable",
            "List",
            "MutableBuffer",
            "MutexSemaphore",
            "Queue",
            "Relation",
            "Set",
            "Table",
        ] {
            let source = format!("say .{class}~new~class~id\n");
            assert_eq!(
                both_engines(&source),
                (0, format!("{class}\n"), String::new()),
                "{class}"
            );
        }
        for (class, status, catalogue) in [
            ("Class", 163, "Error 93.901:"),
            ("Message", 163, "Error 93.901:"),
            ("Method", 168, "Error 88.901:"),
            ("Package", 168, "Error 88.901:"),
            ("Routine", 168, "Error 88.901:"),
            ("String", 163, "Error 93.903:"),
            ("Supplier", 163, "Error 93.903:"),
            ("WeakReference", 163, "Error 93.903:"),
        ] {
            let (code, stdout, stderr) = both_engines(&format!("say .{class}~new\n"));
            assert_eq!((code, stdout.as_str()), (status, ""), "{class}");
            assert!(stderr.contains(catalogue), "{class}: {stderr:?}");
        }
    }

    /// The constructors whose argument list carries the instance's whole state
    /// answer one, and the state is either kept and read back or refused,
    /// never answered from nothing.
    ///
    /// The refusal rows are what stop this from passing over a constructor
    /// that fabricated a body: each names a method that would read what the
    /// arguments carried, and the crate holds none of it. `MutableBuffer`
    /// keeps what it is given, so its row reads the state back instead.
    #[test]
    fn a_constructor_taking_arguments_answers_an_instance_and_refuses_its_state() {
        for (program, id, unread) in [
            (".Message~new(.Object~new, 'STRING')", "Message", "o~send"),
            (
                ".WeakReference~new(.Object~new)",
                "WeakReference",
                "o~value",
            ),
        ] {
            assert_eq!(
                both_engines(&format!("o = {program}\nsay o~class~id\n")),
                (0, format!("{id}\n"), String::new()),
                "{program}"
            );
            let (code, stdout, stderr) = both_engines(&format!("o = {program}\nsay {unread}\n"));
            assert_eq!((code, stdout.as_str()), (120, ""), "{program}");
            assert!(stderr.starts_with("rexx-exec: "), "{program}: {stderr:?}");
        }
        assert_eq!(
            both_engines("o = .MutableBuffer~new('abc')\nsay o~class~id\nsay o~length\n"),
            (0, "MutableBuffer\n3\n".to_string(), String::new())
        );
        // `Supplier` joined the readback group in Phase 5g Task 1. Its `init`
        // used to validate both arrays and drop them, which is the shell this
        // test's refusal rows exist to catch; it now keeps them in the
        // receiver's own pool and walks them.
        assert_eq!(
            both_engines(
                "o = .Supplier~new(.Array~of('i'), .Array~of('x'))\n\
                 say o~class~id o~available o~item o~index\n"
            ),
            (0, "Supplier 1 i x\n".to_string(), String::new())
        );
        // `say` reaches a `MutableBuffer` through the required-string
        // protocol's `MAKESTRING`, which answers the contents; the
        // receiver-side comparison stays an identity test, which is the
        // oracle's `0` beside the `1` the other operand order answers.
        assert_eq!(
            both_engines("say .MutableBuffer~new('abc')\n"),
            (0, "abc\n".to_string(), String::new())
        );
        assert_eq!(
            both_engines(
                "buf = .MutableBuffer~new('abc')\n\
                 say (buf == 'abc') ('abc' == buf) (buf = 'abc') ('abc' = buf)\n"
            ),
            (0, "0 1 0 1\n".to_string(), String::new())
        );
        // `~result` on a message nothing has sent blocks the oracle, so this
        // is a refusal rather than an answer; `~completed` and `~hasError`
        // beside it are the oracle's own `0`.
        let (code, stdout, stderr) =
            both_engines("say .Message~new(.Object~new, 'STRING')~result\n");
        assert_eq!((code, stdout.as_str()), (120, ""));
        assert_eq!(
            stderr,
            "rexx-exec: `Message~result` on a message whose send has not been made is not \
             implemented (Phase 6)\n"
        );
        assert_eq!(
            both_engines(
                "m = .Message~new(.Object~new, 'STRING')\n\
                 say m~completed m~hasError\n"
            ),
            (0, "0 0\n".to_string(), String::new())
        );
    }

    /// A `.Directory~new` reads `.nil` for every index until something puts
    /// an entry there, and then reads it back through all three spellings.
    ///
    /// Every line is the oracle's, measured 2026-09-03 on three descriptors.
    /// Reading `.nil` is its answer for an empty directory and a wrong answer
    /// for any other, so the first line is only correct while the writes
    /// beside it land -- and the last is the control: a second directory does
    /// not see the first one's entries. `~zork` reads and writes under the
    /// upper-cased name where `~at` and `~put` do not, which is why `d~zork`
    /// stays `.nil` across the two rows that store `X` and `Y`.
    #[test]
    fn a_new_directory_reads_nil_until_an_entry_is_put_there() {
        assert_eq!(
            both_engines(
                "d = .Directory~new\n\
                 say d['X'] d~at('X') d~zork\n\
                 d~put('v','X')\n\
                 say d['X'] d~at('X') d~zork\n\
                 d['Y'] = 'w'\n\
                 say d['Y'] d~at('Y') d~zork\n\
                 d~zork = 'z'\n\
                 say d['ZORK'] d~zork d~at('ZORK')\n\
                 d2 = .Directory~new\n\
                 say d2['zork'] d2~zork\n"
            ),
            (
                0,
                "The NIL object The NIL object The NIL object\n\
                 v v The NIL object\n\
                 w w The NIL object\n\
                 z z z\n\
                 The NIL object The NIL object\n"
                    .to_string(),
                String::new()
            )
        );
    }

    /// A `Directory` subclass has a variable pool AND a store, so its
    /// `EXPOSE` answers and so do its entry writes.
    ///
    /// **The split this test recorded is closed.** It used to assert the
    /// second half REFUSING, and said so: "what the split costs and the
    /// oracle answers `1` for". `native_directory_new` gave `.Directory`
    /// itself a `Body::Native` -- which has no variable pool -- and a
    /// subclass a plain instance, so exactly one of the two rows could be
    /// green at a time. Phase 5h Task 4 gave every `Directory` a program
    /// makes the hash store, which lives in the pool, so the two are no
    /// longer exclusive. Both values below are the oracle's.
    #[test]
    fn a_directory_subclass_keeps_the_instance() {
        assert_eq!(
            both_engines(
                "o = .K~new\n\
                 say o~class~id o~isA(.Directory) o~peek\n\
                 ::class K subclass Directory\n\
                 ::method init\n\
                 \x20 expose n\n\
                 \x20 n = 5\n\
                 \x20 self~init:super\n\
                 ::method peek\n\
                 \x20 expose n\n\
                 \x20 return n\n"
            ),
            (0, "K 1 5\n".to_string(), String::new())
        );
        let (code, stdout, stderr) = both_engines(
            "o = .Directory~subclass('K')~new\n\
             o['A'] = 1\n\
             say o['A']\n",
        );
        assert_eq!((code, stdout.as_str(), stderr.as_str()), (0, "1\n", ""));
    }

    /// A stem receiver answers `Stem` and renders as its own value, and a
    /// `Stem` method with no body refuses loudly rather than answering.
    ///
    /// **The halves fail on opposite mistakes.** A stem built as a plain
    /// instance renders `a Stem` where the oracle renders the default, at
    /// rc 0 on both sides; a `~objectName` taken from the string value
    /// answers `dflt` where the oracle answers `a Stem`; and without the
    /// refusals a `Stem` method with no body would answer from nothing. Every
    /// value below is the oracle's, measured.
    #[test]
    fn a_stem_receiver_answers_stem_and_renders_its_own_value() {
        assert_eq!(
            both_engines(
                "s. = 'dflt'\n\
                 o = s.\n\
                 say o~class~id o~isA(.Stem) o~objectName o~defaultName\n\
                 say o o~string\n"
            ),
            (
                0,
                "Stem 1 a Stem a Stem\ndflt dflt\n".to_string(),
                String::new()
            )
        );
        assert_eq!(
            both_engines(
                "say '[' || .Stem~new || ']' '[' || .Stem~new('FOO.') || ']'\n\
                 say .Stem~new~class~id .Stem~new('FOO.')~string\n"
            ),
            (0, "[] [FOO.]\nStem FOO.\n".to_string(), String::new())
        );
        // **These three used to be the refusals this test pinned**, and
        // Phase 5h Task 5 gave `Stem` its collection surface. Each value is
        // the oracle's: a tail never assigned answers the stem's default,
        // and `items` counts only the tails that hold something.
        for (send, answer) in [("o~at(1)", "dflt"), ("o[1]", "dflt"), ("o~items", "0")] {
            assert_eq!(
                both_engines(&format!("s. = 'dflt'\no = s.\nsay {send}\n")),
                (0, format!("{answer}\n"), String::new()),
                "{send}"
            );
        }
        // A subclass would need the body to carry a class of its own, which
        // `Body::Stem` does not, so the constructor refuses rather than
        // answering an object whose `~class~id` is `Stem`.
        let (code, stdout, stderr) = both_engines("say .K~new~class~id\n::class K subclass Stem\n");
        assert_eq!((code, stdout.as_str()), (120, ""));
        assert_eq!(
            stderr,
            "rexx-exec: method \"NEW\" of class \"K\" is not implemented (Phase 5)\n"
        );
    }

    /// A `StringTable` subclass answers its own class and its own method set,
    /// and stores what its constructor puts in it -- `.TraceObject~new` is the
    /// one the image ships.
    ///
    /// **The pair is what makes each half mean something.** `makeString` is a
    /// name `TraceObject` declares and `StringTable` does not, so a build that
    /// read the behaviour off `.StringTable` answers `StringTable 0` for the
    /// first line; and the entries are what separate a genuine collection from
    /// an object that merely answers the right names. Every value below is the
    /// oracle's, measured.
    #[test]
    fn a_string_table_subclass_answers_its_own_class_methods_and_entries() {
        assert_eq!(
            both_engines(
                "say .TraceObject~new~class~id .TraceObject~new~hasMethod('makeString')\n\
                 say .StringTable~new~class~id .StringTable~new~hasMethod('makeString')\n"
            ),
            (
                0,
                "TraceObject 1\nStringTable 0\n".to_string(),
                String::new()
            )
        );
        assert_eq!(
            both_engines(
                "o = .TraceObject~new\n\
                 say o['OPTION'] o['NUMBER'] o~at('NUMBER') o['TIMESTAMP']~class~id\n"
            ),
            (0, "N 1 1 DateTime\n".to_string(), String::new())
        );
    }

    /// A semaphore's `UNINIT` runs at collection rather than at a send, so a
    /// class that declares one needs a row even where the finalizer does
    /// nothing.
    ///
    /// A refusal escaping the sweep reaches the program, so the instance
    /// merely going out of scope is what this asks about.
    #[test]
    fn a_semaphore_instance_runs_its_finalizer_without_refusing() {
        for class in ["EventSemaphore", "MutexSemaphore"] {
            let source = format!("o = .{class}~new\nsay 'built'\n");
            assert_eq!(
                both_engines(&source),
                (0, "built\n".to_string(), String::new()),
                "{class}"
            );
        }
    }

    /// **The one shape of `~at` that has no oracle behaviour to match**, and
    /// the instrument [`Loud::array_index_hole`]'s own doc names.
    ///
    /// `.Array~of` fills the slots from its arguments, and every line is the
    /// oracle's, measured 2026-09-03 on three descriptors.
    ///
    /// **The omission rows are what separate a real `~of` from a `~new` that
    /// took the argument count.** An interior omission is a slot with no item
    /// (`~size` 3, `~items` 2) and a trailing one is not an argument at all
    /// (`~size` 2), and an empty list fixes the shape where `.array~new()`
    /// leaves it open.
    #[test]
    fn array_of_fills_its_slots_from_its_arguments() {
        assert_eq!(
            both_engines(
                "say .array~of(1,2,3)~size .array~of(1,2,3)~items .array~of(1,2,3)~dimension\n\
                 say .array~of()~size .array~of()~items .array~of()~dimension\n\
                 say .array~of(1,,3)~size .array~of(1,,3)~items\n\
                 say .array~of(1,2,)~size .array~of(1,2,)~items\n\
                 say .array~of(4,5)[2] .array~of('x')~class~id\n\
                 a = .array~of(7,8)\n\
                 a[3] = 9\n\
                 say a~size a~toString('l', ' ')\n"
            ),
            (
                0,
                "3 3 1\n0 0 1\n3 2\n2 2\n5 Array\n3 7 8 9\n".to_string(),
                String::new()
            )
        );
    }

    /// `~of` sent to a subclass of `Array` answers an instance of that
    /// subclass -- Phase 5g Task 6, where this test used to assert the
    /// refusal.
    ///
    /// Measured, oracle rc 0: `.array~subclass('K')~of(1,2)~size` is `2` and
    /// its `~class~id` is `K`.
    #[test]
    fn array_of_on_a_subclass_answers_an_instance_of_it() {
        assert_eq!(
            both_engines("k = .array~subclass('K')\nsay k~of(1,2)~size\nsay k~of(1,2)~class~id\n"),
            (0, "2\nK\n".to_string(), String::new())
        );
    }

    /// A lone array argument is spread into the subscript list by item count
    /// with slot array, so an array whose leading slot is empty and whose item
    /// count is one hands the C++ a null subscript and it dies. The program is
    /// in `corpus/oracle-crashes.txt` and must never be run against the
    /// oracle, so a differential row cannot cover this and this test is the
    /// whole of it.
    ///
    /// **The adjacent answers are the point.** Each row beside the refusal is
    /// a shape the spread does answer, and a build that refused the whole
    /// spread -- or that read the item count as the size -- reddens one of
    /// them rather than passing.
    #[test]
    fn an_expanded_index_of_one_empty_slot_is_loud() {
        for source in [
            "a = (1,2)\nsay a~at((,2))\n",
            "a = (1,2)\nsay a~at((,,3))\n",
            "a = (1,2)\nsay a[(,2)]\n",
        ] {
            let (code, stdout, stderr) = both_engines(source);
            assert_eq!((code, stdout.as_str()), (120, ""), "{source:?}");
            assert_eq!(
                stderr, "rexx-exec: an array subscript that is an empty slot is not implemented\n",
                "{source:?}"
            );
        }
        // The neighbouring successes: one item spread from a two-slot array,
        // two items spread from a three-slot one, and an empty spread.
        assert_eq!(
            both_engines(
                "a = (1,2)\n\
                 say a~at((1,))\n\
                 say a[(2,)]\n"
            ),
            (0, "1\n2\n".to_string(), String::new())
        );
        let (code, stdout, stderr) = both_engines("a = (1,2)\nsay a~at((1,,3))\n");
        assert_eq!((code, stdout.as_str()), (163, ""));
        assert!(stderr.contains("Error 93.926:"), "{stderr:?}");
        let (code, stdout, stderr) = both_engines("a = (1,2)\nsay a~at((,))\n");
        assert_eq!((code, stdout.as_str()), (163, ""));
        assert!(stderr.contains("Error 93.901:"), "{stderr:?}");
    }

    /// The position a subscript refusal names is the subscript's place in the
    /// **method's own** argument list, which `putRexx`'s leading value moves
    /// by one -- and which index kind the subscript belongs to decides the
    /// error as well as the number, because `positionArgument` names no
    /// position at all where `requiredPositive` does.
    ///
    /// The corpus differential compares the numbers these raise; this is what
    /// compares their substitutions.
    ///
    /// Measured, oracle rc 163, `m = .array~new(2,3)`.
    #[test]
    fn a_subscript_refusal_names_the_position_its_own_method_counts_from() {
        for (source, message) in [
            (
                "m = .array~new(2,3)\nsay m[,2]\n",
                "Error 93.903:  Missing argument in method; argument 2 is required.",
            ),
            (
                "m = .array~new(2,3)\nm[,2] = 1\n",
                "Error 93.903:  Missing argument in method; argument 3 is required.",
            ),
            (
                "a = (1,2)\na~put('v',0)\n",
                "Error 93.907:  Method argument 2 must be a positive whole number; found \"0\".",
            ),
            (
                "m = .array~new(2,3)\nsay m[1,0]\n",
                "Error 93.924:  Invalid position argument specified; found \"0\".",
            ),
            (
                "say .array~new(2,'-1.0')~size\n",
                "Error 93.906:  Method argument 2 must be zero or a positive whole \
                 number; found \"-1.0\".",
            ),
            (
                "say .array~new(100000000000000001)~size\n",
                "Error 93.959:  An array cannot contain more than 100000000000000000 elements.",
            ),
        ] {
            let (code, stdout, stderr) = both_engines(source);
            assert_eq!((code, stdout.as_str()), (163, ""), "{source:?}");
            assert!(stderr.contains(message), "{source:?} {stderr:?}");
        }
    }
    /// A subclass of `Array` constructs, keeps its class, and carries both a
    /// store and an object variable pool -- Phase 5g Task 6.
    ///
    /// This test used to assert the refusal that stood here. Measured on the
    /// oracle: `.array~subclass('K')~new(2,3)~size` is `6` at rc 0.
    #[test]
    fn new_on_a_subclass_of_array_answers_an_instance_of_it() {
        assert_eq!(
            both_engines(
                "k = .array~subclass('K')\nsay k~id\nsay k~new(2,3)~size\nsay k~new~class~id\n"
            ),
            (0, "K\n6\nK\n".to_string(), String::new())
        );
    }

    /// A name a hash collection's behaviour does not answer is **not** 97.1:
    /// the forward reaches the receiver's own `UNKNOWN`, whose body reads the
    /// name as an entry -- so a missing name is `.nil` and a present one is
    /// the entry.
    ///
    /// **The `String` row is the pair that makes this a rule about `UNKNOWN`
    /// rather than about missing names.** That receiver's behaviour answers no
    /// `UNKNOWN`, so the same shape really is the condition there, and a build
    /// answering `.nil` for every miss fails it.
    #[test]
    fn a_hash_collection_forwards_a_missing_name_to_its_own_unknown() {
        assert_eq!(
            both_engines(
                "say .environment~nosuch\n\
                 say .local~nosuch\n\
                 say .methods~nosuch\n\
                 ::method z\n"
            ),
            (
                0,
                "The NIL object\nThe NIL object\nThe NIL object\n".to_string(),
                String::new()
            )
        );
        let (code, stdout, stderr) = both_engines("say 'abc'~nosuch\n");
        assert_eq!((code, stdout.as_str()), (159, ""));
        assert!(stderr.contains("Error 97.1:"), "{stderr:?}");
    }

    /// **The entry-method assignment with no value to store**, which the
    /// oracle answers by reading uninitialised memory -- see
    /// [`Loud::entry_method_without_a_value`] for the measurement.
    ///
    /// The only instrument, for the reason that constructor's doc gives: there
    /// is no oracle behaviour to agree with, so no corpus row can carry it.
    ///
    /// **The answering rows beside it are the point.** The same message name
    /// with a value stores it, and the ordinary message-assignment form
    /// reaches the same code with a value always present -- so a build that
    /// refused the whole set form fails them rather than passing.
    #[test]
    fn an_entry_method_send_with_no_value_is_loud() {
        for source in [
            ".local~\"MYTHING=\"()\n",
            ".local~\"MYTHING=\"(,)\n",
            ".methods~\"Q=\"()\n::method z\n",
        ] {
            let (code, stdout, stderr) = both_engines(source);
            assert_eq!((code, stdout.as_str()), (120, ""), "{source:?}");
            assert!(
                stderr.starts_with("rexx-exec: an entry-method assignment to ")
                    && stderr.ends_with("with no value is not implemented\n"),
                "{source:?} refused with {stderr:?}"
            );
        }
        assert_eq!(
            both_engines(
                ".local~\"MYTHING=\"('v')\n\
                 say .MYTHING\n\
                 .local~MYTHING = 'w'\n\
                 say .MYTHING\n"
            ),
            (0, "v\nw\n".to_string(), String::new())
        );
    }

    /// **A metaclass carrying its own `NEW` decides what `~subclass` builds**,
    /// and this crate has no `NEW` to run -- see [`factory_metaclass`] for why
    /// that is a loud refusal rather than a class built from `.Class`'s path.
    ///
    /// The only instrument. The oracle answers this shape at rc 0, so no
    /// corpus row can carry a refusal for it; and a build that dropped the
    /// check would answer at rc 0 as well, with a class the metaclass's own
    /// `NEW` never saw. That is a silent wrong answer, which is the outcome
    /// nothing else here catches.
    ///
    /// **The answering row beside it is the point.** The same metaclass with
    /// no `NEW` of its own builds and answers `id k`, matching the oracle, so
    /// a build that refused every named metaclass fails that row rather than
    /// passing this one.
    ///
    /// The `FORWARD` body is deliberate and is not reached: a `::METHOD new
    /// CLASS` that returns anything which is not a class object crashes the
    /// oracle, `corpus/oracle-crashes.txt` entry 8, and a program in this
    /// file is a program someone will eventually run against it.
    #[test]
    fn a_metaclass_with_its_own_new_is_loud() {
        let (code, stdout, stderr) = both_engines(
            "say 'id' .object~subclass(\"k\", .MyMeta)~id\n\
             ::CLASS MyMeta SUBCLASS Class\n\
             ::METHOD new CLASS\n\
             forward class (super)\n",
        );
        assert_eq!((code, stdout.as_str()), (120, ""));
        assert_eq!(
            stderr,
            "rexx-exec: method \"NEW\" of class \"MYMETA\" is not implemented (Phase 5)\n"
        );
        assert_eq!(
            both_engines(
                "say 'id' .object~subclass(\"k\", .MyMeta)~id\n\
                 ::CLASS MyMeta SUBCLASS Class\n"
            ),
            (0, "id k\n".to_string(), String::new())
        );
    }

    /// **`.RESOURCES` holds each `::RESOURCE` body as an `Array` of its own
    /// lines**, and this is the whole instrument for it.
    ///
    /// No corpus program can carry it: a `::RESOURCE` body is source lines
    /// that no clause span covers, and `rexx-parse`'s
    /// `every_corpus_program_tiles` requires every byte of a corpus program to
    /// be tiled by a clause node. `corpus/phase-5a.txt` says so at the Task 17
    /// block. Every row below was measured on the oracle at rc 0.
    ///
    /// The empty body is the neighbour that stops a build answering the whole
    /// file, and the lower-case index is the one that stops a build keying by
    /// the spelling the directive quoted.
    #[test]
    fn the_package_tables_hold_what_their_directives_declare() {
        assert_eq!(
            both_engines(
                "say .resources~class\n\
                 say .resources~x~class\n\
                 say .resources~x~items\n\
                 say .resources~x\n\
                 say .resources[\"X\"]~at(2)\n\
                 say .resources[\"x\"]\n\
                 say .resources~q\n\
                 ::resource \"x\"\n\
                 line one\n\
                 line two\n\
                 ::END\n"
            ),
            (
                0,
                "The StringTable class\nThe Array class\n2\nline one\nline two\n\
                 line two\nThe NIL object\nThe NIL object\n"
                    .to_string(),
                String::new()
            )
        );
        assert_eq!(
            both_engines(
                "say .resources~x~items\n\
                 say '[' || .resources~x || ']'\n\
                 ::resource x\n\
                 ::END\n"
            ),
            (0, "0\n[]\n".to_string(), String::new())
        );
    }

    /// An `~UNKNOWN` sent by hand rather than forwarded: its argument list is
    /// an `Array` on every forward and anything at all here, and the oracle
    /// converts what it is given with `requestArray`.
    ///
    /// In-crate only, because the refusal is this crate's: no `MAKEARRAY` is
    /// implemented for any receiver, so the conversion the oracle performs is
    /// the same gap `~request('ARRAY')` already reports. The answering row
    /// beside it is measured on the oracle -- `.environment~unknown('ARRAY',
    /// .Array~superClasses)` is `The Array class` at rc 0 -- and is what stops
    /// a build refusing every by-hand send from passing.
    #[test]
    fn an_unknown_sent_by_hand_needs_an_array_this_crate_does_not_convert() {
        assert_eq!(
            both_engines("say .environment~unknown('ARRAY', .Array~superClasses)\n"),
            (0, "The Array class\n".to_string(), String::new())
        );
        assert_eq!(
            both_engines("say .environment~unknown('ARRAY', 'y')\n"),
            (
                120,
                String::new(),
                "rexx-exec: method \"MAKEARRAY\" of class \"String\" is not implemented \
                 (Phase 5)\n"
                    .to_string()
            )
        );
    }

    /// An entry the oracle's own directory holds and this crate does not build
    /// is a refusal, not `.nil` -- and the refusal is per directory, because
    /// the oracle answers `.nil` for a `.local` name asked of `.environment`.
    ///
    /// The three answering rows are what stops a build that refuses every miss
    /// from passing, and the `~put` row is what stops one that refuses by name
    /// alone: an entry a program stored answers even under a name the unbuilt
    /// table holds.
    ///
    /// `ENDOFLINE` rather than a class name: the library bootstrap fills
    /// `.environment` with the classes `CoreClasses.orx` and
    /// `StreamClasses.orx` declare, so almost every entry in the unbuilt
    /// table now answers. What is left is `Setup.cpp`'s own non-class
    /// additions, of which this is one -- the line terminator read off the
    /// `RexxInfo` instance (`Setup.cpp:1741`), measured a bare newline on
    /// the oracle.
    #[test]
    fn a_directory_entry_the_oracle_has_and_this_crate_does_not_is_loud() {
        for (source, owner) in [
            ("say .environment['ENDOFLINE']\n", "Phase 5"),
            ("say .local['STDOUT']\n", "Phase 7"),
        ] {
            let (code, stdout, stderr) = both_engines(source);
            assert_eq!((code, stdout.as_str()), (120, ""), "{source:?}");
            assert!(
                stderr.starts_with("rexx-exec: directory entry ")
                    && stderr.ends_with(&format!("is not implemented ({owner})\n")),
                "{source:?} refused with {stderr:?}"
            );
        }
        assert_eq!(
            both_engines(
                "say .environment['STDOUT']\n\
                 say .local['ALARM']\n\
                 say .environment['ARRAY']~id\n"
            ),
            (
                0,
                "The NIL object\nThe NIL object\nArray\n".to_string(),
                String::new()
            )
        );
        assert_eq!(
            both_engines(
                "d = .local\n\
                 d~put('mine', 'STDOUT')\n\
                 say d['STDOUT']\n"
            ),
            (0, "mine\n".to_string(), String::new())
        );
    }
    /// **The required-string protocol answers where it can and refuses where
    /// it cannot, and the refusals are only visible here.**
    ///
    /// The corpus gate cannot see a clean refusal becoming a wrong answer: a
    /// refusal this crate does not share with the oracle is not expressible as
    /// a differential row at all. So the shapes of `~request` and
    /// `~objectName=` this task deliberately left loud are asserted by their
    /// message here, and the neighbouring answering shapes sit beside them --
    /// which is what stops "refuse every argument" passing.
    #[test]
    fn a_conversion_this_phase_does_not_model_is_loud_where_the_ones_it_models_answer() {
        for (source, message) in [
            // `MAKEARRAY` is in this behaviour's dictionary and this crate
            // has no code for it. Answering `.nil` would contradict the
            // oracle, which converts: measured, oracle rc 0 and
            // `.environment~request("ARRAY")` is an array. `String` is no
            // longer here -- `native_string_makearray` answers it, and
            // `string_makearray.rex` compares every shape against the
            // oracle.
            (
                "say .environment~request('ARRAY')\n",
                "rexx-exec: method \"MAKEARRAY\" of class \"Directory\" is not implemented \
                 (Phase 5)\n",
            ),
            // A receiver with no variable pool to keep a name in. The oracle
            // stores one and remembers it; answering rc 0 and forgetting it
            // would be a wrong answer.
            (
                "'abc'~objectName = 'x'\n",
                "rexx-exec: method \"OBJECTNAME=\" of class \"Object\" is not implemented \
                 (Phase 5)\n",
            ),
            (
                "5~objectName = 'x'\n",
                "rexx-exec: method \"OBJECTNAME=\" of class \"Object\" is not implemented \
                 (Phase 5)\n",
            ),
        ] {
            assert_eq!(
                both_engines(source),
                (120, String::new(), message.to_string()),
                "{source:?}"
            );
        }
        // The neighbours, each measured on the oracle: a `MAKE` method the
        // behaviour does not have falls through to the id match and then to
        // `.nil`, and a receiver that does have somewhere to keep a name keeps
        // it.
        for (source, expected) in [
            ("say .K~request('ARRAY')\n::class K\n", "The NIL object\n"),
            ("say .Array~request('CLASS')\n", "The Array class\n"),
            (".K~objectName = 'named'\nsay .K\n::class K\n", "named\n"),
            (
                ".environment~objectName = 'named'\nsay .environment\n",
                "named\n",
            ),
        ] {
            assert_eq!(
                both_engines(source),
                (0, expected.to_string(), String::new()),
                "{source:?}"
            );
        }
    }

    /// **A `reqstr` context whose instruction this phase refuses stays
    /// refused**, which is the honest reading of the section's own list: it
    /// bounds the row set, and a row whose instruction is another phase's is
    /// outside it.
    ///
    /// A refusal cannot be a corpus row, so this is the only instrument. Each
    /// message names the owning phase the section's own context is waiting on.
    #[test]
    fn a_reqstr_context_whose_instruction_is_another_phases_is_still_loud() {
        for (source, message) in [
            (
                "options .K\n::class K\n::method makeString class\n  return 'NOVALUE'\n",
                "rexx-exec: OPTIONS is not implemented (Phase 5)\n",
            ),
            (
                ".K\n::class K\n::method makeString class\n  return 'true'\n",
                "rexx-exec: a command is not implemented (Phase 7)\n",
            ),
            (
                "address 'SYSTEM' .K\n::class K\n::method makeString class\n  return 'true'\n",
                "rexx-exec: ADDRESS is not implemented (Phase 7)\n",
            ),
        ] {
            assert_eq!(
                both_engines(source),
                (120, String::new(), message.to_string()),
                "{source:?}"
            );
        }
    }

    /// An `Interp` whose class registry holds one class awaiting a class-side
    /// `UNINIT`, which is what a directive install leaves behind.
    fn interp_awaiting_a_class_uninit() -> Interp {
        let mut interp = Interp::new();
        let program = Rc::new(
            rexx_parse::parse_program(b"nop\n::class k\n::method uninit class\n  nop\n".to_vec())
                .expect("it parses"),
        );
        // The id `install_directives` is given has to name a program in
        // `Interp::programs`, which is what `Interp::run` pushes before it
        // installs anything; without it a finalizer's body cannot be reached.
        let id = crate::ProgramId(interp.programs.len());
        interp.programs.push(Rc::clone(&program));
        interp
            .install_directives(id, &program)
            .expect("the directives install");
        interp
    }

    /// [`Interp::run_termination_uninits`] refuses to re-enter itself, and
    /// refusing leaves its work pending rather than consuming it.
    ///
    /// **The only instrument for the termination sweep's copy of the
    /// interlock.** Its other copy, in [`Interp::run_ready_uninits`], is
    /// witnessed differentially by
    /// `corpus/lang/uninit_nested_collection_at_exit.rex`, which reaches it
    /// from inside a termination finalizer; the check-and-set below is
    /// reached only by a second entry into the termination sweep, which no
    /// program has been found to produce.
    ///
    /// **The empty answer is not the assertion**, because an empty `Vec<Loud>`
    /// is also what a sweep that ran everything successfully answers. What
    /// separates them is whether the registry still holds the class, so the
    /// un-interlocked arm below is what gives the interlocked one its meaning.
    #[test]
    fn an_interlocked_termination_sweep_runs_nothing_and_consumes_nothing() {
        let mut fresh = interp_awaiting_a_class_uninit();
        assert_eq!(
            fresh.classes().take_uninit_classes_in_sweep_order().len(),
            1,
            "the setup must really leave a class pending, or both arms below \
             are green over an empty registry"
        );

        let mut held = interp_awaiting_a_class_uninit();
        held.processing_uninits = true;
        assert!(
            held.run_termination_uninits().is_empty(),
            "an interlocked sweep answers no loud refusal because it runs nothing"
        );
        assert_eq!(
            held.classes().take_uninit_classes_in_sweep_order().len(),
            1,
            "an interlocked sweep must leave the class for the caller holding \
             the flag; an empty answer here means it swept anyway"
        );

        let mut free = interp_awaiting_a_class_uninit();
        assert!(free.run_termination_uninits().is_empty());
        assert!(
            free.classes()
                .take_uninit_classes_in_sweep_order()
                .is_empty(),
            "a sweep that is not interlocked consumes what it ran"
        );
    }
}
