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
mod seam {
    use super::{Failure, Interp, MethodId, ObjRef};

    /// Evidence that a message send passed the dispatch security seam.
    pub(super) struct Cleared(());

    /// **The dispatch chokepoint (D45, site one).** Every method invocation
    /// passes here, whatever kind [`super::Invocable`] finds it to be, and a
    /// manager installed in a later phase gets its hook in this function's
    /// body.
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

// `RexxInfo`'s readers, chained the same way.
mod rexx_info;

// `Method`'s and `Routine`'s own readers, chained the same way.
pub(crate) mod executable;

// `Class`'s graph readers and `Object`'s three, chained the same way.
mod introspection;

// `RexxContext`'s and `StackFrame`'s readers, chained the same way.
mod context;

// `Package`'s readers and its four writes, chained the same way.
mod package;

/// One primitive method's implementation.
type NativeMethod =
    fn(&mut Interp, Cleared, ObjRef, &[Option<ObjRef>]) -> Result<Option<ObjRef>, Failure>;

/// What a resolved [`MethodId`] runs.
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

/// Files one native method under its own id, growing the table to reach it.
fn put_native(natives: &mut Vec<Option<NativeEntry>>, method: MethodId, entry: NativeEntry) {
    let index = method.0 as usize;
    if index >= natives.len() {
        natives.resize(index + 1, None);
    }
    natives[index] = Some(entry);
}

/// What [`Interp::invoke`] needs about one primitive method beyond its code.
#[derive(Copy, Clone)]
struct NativeEntry {
    arity: Arity,
    run: NativeMethod,
}

/// How many arguments an entry admits.
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
    // The comparison operators `memory/Setup.cpp`'s `Class` block declares
    // (`:485`-`:490`), each at count 1 and each `RexxClass::equal`,
    // `strictEqual` or `notEqual` -- the same identity test `Object`'s rows
    // below carry, bound
    // again here because `Class` names its own and a name resolves to the
    // more specific scope. `Interp::operator_message_receiver` is the send,
    // so these answer the expression `(.Array = .Array)` and the message
    // `.Array~'='(.Array)` alike. Measured, oracle rc 0: `.array = .array` is
    // `1`, `.array = 'The Array class'` is `0` -- identity, not a comparison
    // of renderings.
    ("Class", "=", Arity::Fixed(1), native_object_identical),
    ("Class", "==", Arity::Fixed(1), native_object_identical),
    ("Class", "\\=", Arity::Fixed(1), native_object_different),
    ("Class", "\\==", Arity::Fixed(1), native_object_different),
    ("Class", "<>", Arity::Fixed(1), native_object_different),
    ("Class", "><", Arity::Fixed(1), native_object_different),
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
    // `QueueClass::initRexx` (`memory/Setup.cpp:777`) and the donation
    // `InheritInstanceMethods(IdentityTable)` makes at `Relation` (`:958`).
    ("Relation", "INIT", Arity::Fixed(1), native_capacity_init),
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
    // `AddMethod("Value", WeakReference::value, 0)`, `memory/Setup.cpp:1686`.
    (
        "WeakReference",
        "VALUE",
        Arity::Fixed(0),
        native_weak_reference_value,
    ),
];

/// The primitive methods bound to a class's **class** dictionary rather than
/// its instance one -- `memory/Setup.cpp`'s `AddClassMethod` rows.
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
    (
        "Method",
        "LOADEXTERNALMETHOD",
        Arity::Fixed(2),
        native_load_external,
    ),
    ("Method", "NEW", Arity::Counted, native_executable_new),
    ("Method", "NEWFILE", Arity::Fixed(2), native_new_file),
    ("Package", "NEW", Arity::Counted, native_package_new),
    (
        "Routine",
        "LOADEXTERNALROUTINE",
        Arity::Fixed(2),
        native_load_external,
    ),
    ("Routine", "NEW", Arity::Counted, native_executable_new),
    ("Routine", "NEWFILE", Arity::Fixed(2), native_new_file),
    (
        "WeakReference",
        "NEW",
        Arity::Counted,
        native_weak_reference_new,
    ),
    // The classes whose `newRexx` is the refusal and nothing else, because
    // their instances come only from native code (`utilityclasses.xml:429`,
    // `:6910`).
    ("Buffer", "NEW", Arity::Counted, native_unsupported_new),
    ("Pointer", "NEW", Arity::Counted, native_unsupported_new),
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
pub(crate) const ARRAY_DEFAULT_NAME: &[u8] = b"an Array";

/// The message the search order's last step sends -- `GlobalNames::UNKNOWN`.
const UNKNOWN: &[u8] = b"UNKNOWN";

/// The message name a `~run` body is entered under --
/// `GlobalNames::UNNAMED_METHOD` (`memory/GlobalNames.h:240`), which
/// `RexxObject::run` passes to `methobj->run`
/// (`classes/ObjectClass.cpp:2245`).
const UNNAMED_METHOD: &[u8] = b"*UNNAMED*";

/// The entry a `Message` object keeps the value its send answered under.
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
pub(crate) struct ObjectModel {
    classes: ClassRegistry,
    /// Indexed by [`MethodId`]'s own number rather than keyed by it.
    natives: Vec<Option<NativeEntry>>,
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
    /// The scope [`WEAK_REFERENT`] is bound in, so that a subclass's instance
    /// keeps its referent where `WeakReference~value` looks for it.
    weak_reference: ObjRef,
    /// The class `RexxContext~stackFrames` builds its elements from. Reached
    /// here rather than by name at every build, exactly as `context` beside
    /// it is.
    stack_frame: ObjRef,
}

impl ObjectModel {
    /// The hash-collection class a `DO OVER` target may be, for a caller
    /// asking whether an object is one.
    pub(crate) fn iterable_collection_class(&self) -> ObjRef {
        self.string_table
    }

    /// `Setup.cpp`'s native class set, plus the lookup from each implemented
    /// method's minted identity to its code.
    fn bootstrap(mint: &mut dyn FnMut() -> ObjRef) -> ObjectModel {
        ObjectModel::build(rexx_classes::native_classes(mint), &[])
    }

    /// [`ObjectModel::bootstrap`] with the two setup-only methods present --
    /// the state the interpreter's own library runs in, and the one
    /// `Interp::bootstrap_library` closes with
    /// `rexx_classes::remove_setup_methods`.
    pub(crate) fn bootstrap_for_library(mint: &mut dyn FnMut() -> ObjRef) -> ObjectModel {
        ObjectModel::build(
            rexx_classes::native_classes_for_bootstrap(mint),
            SETUP_METHODS,
        )
    }

    fn build(
        classes: rexx_classes::ClassRegistry,
        extra: &[(&str, &str, Arity, NativeMethod)],
    ) -> ObjectModel {
        let mut natives: Vec<Option<NativeEntry>> = Vec::new();
        for (class_id, method_name, arity, run) in NATIVE_METHODS
            .iter()
            .chain(string::NATIVE_METHODS)
            .chain(hash::NATIVE_METHODS)
            .chain(collection::NATIVE_METHODS)
            .chain(rexx_info::NATIVE_METHODS)
            .chain(executable::NATIVE_METHODS)
            .chain(introspection::NATIVE_METHODS)
            .chain(context::NATIVE_METHODS)
            .chain(package::NATIVE_METHODS)
            .chain(extra)
        {
            // **The kernel directory as well as the environment one**, since
            // `RexxInfo` is registered only in the former: `ClassRegistry`'s
            // two tables are disjoint, so the fallback cannot reach a
            // different class than the row names, and a name in neither still
            // panics.
            let class = classes
                .lookup(class_id)
                .or_else(|| classes.system_lookup(class_id))
                .unwrap_or_else(|| {
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
            put_native(
                &mut natives,
                method,
                NativeEntry {
                    arity: *arity,
                    run: *run,
                },
            );
        }
        for (class_id, method_name, arity, run) in NATIVE_CLASS_METHODS
            .iter()
            .chain(context::NATIVE_CLASS_METHODS)
            .chain(package::NATIVE_CLASS_METHODS)
        {
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
            put_native(
                &mut natives,
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
        let weak_reference = classes
            .lookup("WeakReference")
            .expect("WeakReference is a native class");
        let stack_frame = classes
            .lookup("StackFrame")
            .expect("StackFrame is a native class");
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
            weak_reference,
            stack_frame,
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
    Routine,
    /// A `Body::Native` whose class is `.Directory` -- `.environment` and
    /// `.local`. Measured, `.environment~class~id` is `Directory`.
    Directory,
    /// A `Body::Native` whose class is `.StringTable` or a subclass of it --
    /// `.methods`, `.routines` and `.resources`, and `.TraceObject~new`.
    /// Measured, `.methods~class` is `The StringTable class` and
    /// `.TraceObject~new~class~id` is `TraceObject`.
    StringTable(ObjRef),
    /// A `Body::Native` whose class is `.RexxContext` -- what `.context`
    /// answers. Measured, `.context~class` is `The RexxContext class`.
    Context,
    /// A `Body::Native` whose class is `.StackFrame` -- what
    /// `RexxContext~stackFrames` fills its array with. Measured,
    /// `.context~stackFrames[1]~class~id` is `StackFrame`.
    StackFrame,
    /// A `Body::Native` whose class is the `RexxInfo` one -- the single
    /// pre-built instance `.RexxInfo` answers (`Setup.cpp:1735`-`:1737`).
    /// Measured, `.RexxInfo~class~id` is `RexxInfo` and
    /// `.RexxInfo~isA(.Class)` is `0`.
    RexxInfo,
    /// A `Body::Native` whose class is `.Message` -- what `~start` and
    /// `~startWith` answer. Measured, oracle rc 0: `o~start('M', 5)~class~id`
    /// is `Message` and `~string` is `a Message`.
    Message,
    /// A `Body::VarRef` -- the object a `>name` term answers. Measured,
    /// `vr = 5; o = >vr; say o~class~id` is `VariableReference`.
    VariableReference,
    /// A `Body::Stem` -- what a bare stem read answers and what `.Stem~new`
    /// builds. Measured, oracle rc 0: `s. = 'd'; o = s.; say o~class~id` is
    /// `Stem`.
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
    Instance {
        class: ObjRef,
        behaviour: BehaviourHandle,
    },
}

/// The behaviour a receiver's messages resolve against.
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
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) struct Caller {
    /// The receiver of the call in progress, from the calling convention.
    receiver: Option<ObjRef>,
    /// The package the sending code was translated in, which is the running
    /// activation's program.
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
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) struct AccessScope {
    /// `PRIVATE` or `PACKAGE`. `Access::Default` or `Access::Public` for a
    /// method whose row exists for `PROTECTED` alone.
    pub(crate) access: Access,
    /// `MethodClass::isProtected`, which is what the security-manager seam
    /// asks about.
    pub(crate) protected: bool,
    /// The package the method's own directive was translated in.
    pub(crate) package: Package,
}

/// Why a send's own lookup produced no method to run.
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
#[derive(Copy, Clone)]
pub(crate) struct Resolution {
    pub(crate) scope: ObjRef,
    pub(crate) method: MethodId,
}

impl Interp {
    /// The [`Caller`] a send written in the running activation resolves as.
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
            let model = ObjectModel::bootstrap(&mut || self.heap.mint_class());
            self.install_object_model(model);
        }
        self.object_model
            .as_mut()
            .expect("the object model was just installed")
    }

    /// Stores a freshly built object model and files the access scopes of
    /// the natives `Setup.cpp` declares private.
    pub(crate) fn install_object_model(&mut self, model: ObjectModel) {
        debug_assert!(
            self.special_methods.is_empty(),
            "the natives' access scopes are the first rows filed, so nothing may have filed one \
             before the model that mints them exists"
        );
        for &method in model.classes.private_native_methods() {
            *self.special_method_row_mut(method) = Some(AccessScope {
                access: Access::Private,
                protected: false,
                package: Package::Rexx,
            });
        }
        self.object_model = Some(model);
    }

    /// The class registry: the one class model in this crate (R9), holding
    /// `Setup.cpp`'s native classes and whatever `::CLASS` directives have
    /// installed beside them.
    pub(crate) fn classes(&mut self) -> &mut ClassRegistry {
        &mut self.object_model().classes
    }

    /// The rendering of a handle the arena does not hold.
    pub(crate) fn not_in_arena(&self, value: ObjRef) -> &[u8] {
        assert!(self.heap.is_class(value), "a live value");
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
    pub(crate) fn is_base_class(&mut self, value: ObjRef) -> bool {
        let Some(class) = self.class_of_value(value) else {
            return true;
        };
        let id = self.classes().id_string(class).to_string();
        self.classes().system_lookup(&id) == Some(class)
    }

    pub(crate) fn receiver_class_id(&mut self, receiver: ObjRef) -> Option<String> {
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

    /// `.Routine`: the class of the objects `.ROUTINES` holds and
    /// `Routine~new` answers.
    pub(crate) fn routine_class(&mut self) -> ObjRef {
        self.object_model().routine
    }

    /// Which native class a value answers to, or the value's own shape when
    /// this phase builds no class for it.
    fn receiver_kind(&self, receiver: ObjRef) -> Result<Primitive, &'static str> {
        match receiver.decode() {
            Decoded::Nil => Ok(Primitive::Object),
            Decoded::SmallInt(_) => Ok(Primitive::SmallInt),
            Decoded::Text(_) => Ok(Primitive::String),
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
                    // The class test folds in here rather than guarding the
                    // arm above: this match already had to fetch the object.
                    Body::Class { .. } => Ok(Primitive::Class(receiver)),
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
                    // One element of `RexxContext~stackFrames`, in the same
                    // position `.context` above is: the class is in the
                    // registry with `Setup.cpp`'s whole instance set, so a
                    // name it does not hold is 97.1 and a name it holds with
                    // no row here is this crate's own gap.
                    Body::Native(native)
                        if self.object_model.as_ref().map(|model| model.stack_frame)
                            == Some(native.class()) =>
                    {
                        Ok(Primitive::StackFrame)
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
            Primitive::StackFrame => model.stack_frame,
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
    pub(crate) fn lookup(
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
        // `from_utf8` rather than `from_utf8_lossy`: both scan, but this one
        // has the word-at-a-time ASCII fast path and returns a borrow with no
        // `Cow`. A name that is not UTF-8 answers `None` here, which is the
        // same miss the lossy form reached by building a name with
        // replacement characters that no dictionary holds.
        let name = std::str::from_utf8(name).ok()?;
        let (scope, method) = match (behaviour, start_scope) {
            (Behaviour::Instance { methods, .. }, None) => {
                self.classes().lookup_at(methods, name)?
            }
            (Behaviour::Instance { methods, .. }, Some(start)) => {
                self.classes().lookup_from_scope_at(methods, name, start)?
            }
            (Behaviour::ClassSide(class), None) => {
                self.classes().lookup_class_method(class, name)?
            }
            (Behaviour::ClassSide(class), Some(start)) => self
                .classes()
                .lookup_class_method_from_scope(class, name, start)?,
        };
        Some(Resolution { scope, method })
    }

    /// What `receiver`'s own dictionary answers for `name`: `None` when it
    /// holds no entry under it, `Some(None)` for a name it hides, and
    /// `Some(Some(method))` for one it defines.
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
    fn lookup_for_refusal(&mut self, receiver: ObjRef, name: &[u8]) -> Option<String> {
        let resolution = self.lookup(receiver, name, None)?;
        Some(self.classes().id_string(resolution.scope).to_string())
    }

    /// The access scope and protection of a resolved method, for the methods
    /// that have one.
    fn access_scope_of(&self, method: MethodId) -> Option<AccessScope> {
        // One bounds check and one load. The table is indexed by the id, so
        // there is no order to get wrong and no search to check against a
        // scan -- the assertion that used to stand here guarded a failure mode
        // this shape does not have.
        self.special_methods
            .get(method.0 as usize)
            .copied()
            .flatten()
    }

    /// `MethodClass::isProtected` (`classes/MethodClass.hpp:116`), asked by
    /// the security-manager seam and by nothing else.
    fn method_is_protected(&self, method: MethodId) -> bool {
        self.access_scope_of(method)
            .is_some_and(|scope| scope.protected)
    }

    /// The security manager's `checkProtectedMethod`, asked for a method the
    /// send has found to be `PROTECTED`.
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
    fn class_of_value(&mut self, value: ObjRef) -> Option<ObjRef> {
        match self.receiver_behaviour(value).ok()? {
            Behaviour::Instance { owner, .. } => Some(owner),
            Behaviour::ClassSide(class) => Some(self.classes().class_of(class)),
        }
    }

    /// `isOfClassType(Class, value)`: whether the value is a class object.
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
    fn invocable(&mut self, resolution: Resolution, name: &[u8]) -> Result<Invocable, Failure> {
        if let Some(entry) = self
            .object_model()
            .natives
            .get(resolution.method.0 as usize)
            .copied()
            .flatten()
        {
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
        let arguments = self.shared_arguments(args);
        let saved_context = std::mem::replace(
            &mut self.call_context,
            crate::CallContext {
                name: name.to_vec(),
                arguments,
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
            // **Back to the pool**, which until now only `Interp::invoke_call`
            // fed. The pool is drained by every push and was filled by the
            // `CALL` path alone, so a program of method sends missed it every
            // time: measured on `bench-programs/dispatch.rex`, 5,000,089
            // misses and no hits, each one a `Box::new` of a 416-byte
            // `Activation` and the free that follows.
            self.recycle_activation(callee);
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
        debug_assert!(
            abandoned || self.roots.live_parked() == 0,
            "a replied method body's values are still parked with nothing owing them"
        );
        failures
    }

    /// Sends `UNINIT` to `object` and answers a loud refusal if one escaped.
    fn run_one_uninit(&mut self, object: ObjRef) -> Option<Loud> {
        if !self.answers_uninit(object) {
            return None;
        }
        // A class is on two lists: the heap's flag, which a collection
        // readies, and `rexx-classes`' pending list, which the termination
        // sweep drains. Dropping it from the second here is what keeps a
        // collection-driven finalizer from running a second time at exit --
        // measured, `class-uninit-at-driven-collection` printed its line
        // twice without it.
        if self.heap.is_class(object) {
            self.classes().forget_uninit_class(object);
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
        // Back to the pool, for the reason the send path above states.
        self.recycle_activation(callee);
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
    fn message_target_text(&mut self, receiver: ObjRef) -> Vec<u8> {
        self.string_value_text(receiver)
    }

    /// One whole `target~name(...)` term: the receiver, the scope override,
    /// the arguments, the send, and `~~`'s replacement of the result by the
    /// target.
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
                if !self.heap.is_class(scope) {
                    return Err(Raised::scope_override_not_a_class().into());
                }
                self.validate_scope_override(receiver, Some(scope))?;
                Some(scope)
            }
        };

        let (mut values, mark) = self.take_value_buffer();
        let evaluated = self.evaluate_message_arguments(code, args, assigned, &mut values);
        let caller = self.caller();
        let result = evaluated
            .and_then(|()| self.send_message(receiver, name, start_scope, &values[mark..], caller));
        self.give_value_buffer(values, mark);
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
enum StringConversion {
    /// The object that *is* the string value: the value itself, or a stem's
    /// own default value, or what a `makeString` answered.
    Object(ObjRef),
    /// The string value as bytes no object holds -- an array's items joined.
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
    #[inline]
    pub(crate) fn required_string_value(&mut self, value: ObjRef) -> Result<ObjRef, Failure> {
        // **A string and a small integer are their own string value**, and
        // that is the answer whether the latch is armed or clear:
        // `classify_string_conversion`'s first arm returns
        // `StringConversion::Object(value)` for both, before it touches
        // `self`, allocates, or looks at the heap. Answering here rather than
        // three calls deeper is what keeps an armed interpreter from paying
        // `required_string_dispatch` + `required_string_answer` +
        // `classify_string_conversion` for every operand of every comparison.
        if matches!(value.decode(), Decoded::SmallInt(_) | Decoded::Text(_)) {
            return Ok(value);
        }
        if self.reqstr_armed {
            // **A heap string or number is its own string value as much as an
            // inline one is** -- the same `classify_string_conversion` arm
            // answers `Object(value)` for `Body::Text` and `Body::Num`. It
            // costs the heap lookup that arm makes anyway, and it keeps the
            // `push_temp`, because unlike the two decodings above this value
            // does live in the arena and the caller's rooting of it is not
            // this function's to assume.
            if matches!(
                self.heap.get(value).map(|object| &object.body),
                Some(Body::Text { .. } | Body::Num { .. })
            ) {
                self.roots.push_temp(value);
                return Ok(value);
            }
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
        // **Scanned before anything is built.** `None` means "the arguments
        // stand as they are", so a call whose every argument is already its
        // own string value can answer that instead of allocating a vector to
        // hold copies of what the caller already has. An armed interpreter
        // otherwise allocated one `Vec` per builtin call for the whole run,
        // and being armed is not rare: any instance of a Rexx class arms it.
        if !args.iter().enumerate().any(|(index, argument)| {
            argument.is_some_and(|value| {
                !raw.contains(&(index + 1))
                    && !matches!(value.decode(), Decoded::SmallInt(_) | Decoded::Text(_))
            })
        }) {
            return Ok(None);
        }
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
    fn classify_string_conversion(&mut self, value: ObjRef) -> StringConversion {
        let redirect = match value.decode() {
            Decoded::Nil => return self.make_string_or_none(value),
            // `RexxString::primitiveMakeString` and
            // `RexxInteger::primitiveMakeString`: a string and a number are
            // their own string value.
            Decoded::SmallInt(_) | Decoded::Text(_) => {
                return StringConversion::Object(value);
            }
            Decoded::Heap { .. } if self.heap.is_class(value) => {
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
    fn make_string_or_none(&mut self, value: ObjRef) -> StringConversion {
        match self.lookup(value, MAKESTRING, None) {
            Some(_) => StringConversion::MakeString,
            None => StringConversion::None,
        }
    }

    /// Sends `makeString` on the receiver's behalf.
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
    fn blame_request(&mut self) {
        self.blame_native_method(b"REQUEST", "Object");
    }
}

/// `RexxInternalObject::requiredString(position)`
/// (`classes/ObjectClass.cpp:1373`): a method argument the method needs as
/// text, converted through the required-string protocol, or 88.909 for a
/// value that has no string value at all.
/// ```text
/// 'abc'~hasMethod(5)                oracle `0` rc 0    a number has one
/// 'abc'~hasMethod(a.)               oracle `0` rc 0    an unset stem is its own name
/// 'abc'~hasMethod(.nil)             oracle 88.909 rc 168
/// 'abc'~hasMethod(.String)          oracle 88.909 rc 168
/// 'abc'~hasMethod(.environment)     oracle 88.909 rc 168
/// a. = .array; 'abc'~hasMethod(a.)  oracle 88.909 rc 168
/// a. = .nil;   'abc'~hasMethod(a.)  oracle 88.909 rc 168
/// ```
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
fn native_no_op(
    _interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(None)
}

/// `Object~hasMethod(name)`: whether the receiver's behaviour answers `name`.
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
fn class_argument(interp: &Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("class").into());
    };
    match interp.heap.is_class(argument) {
        true => Ok(argument),
        false => Err(Raised::argument_not_a_class("class").into()),
    }
}

/// `Class~id`: the name the class was declared with, case unmodified --
/// `RexxClass::getId` (`classes/ClassClass.cpp:385`).
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
fn native_default_name(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(class) = native_class(interp, cleared, receiver, &[])? else {
        return Err(Loud::receiver_class("a value with no class of its own").into());
    };
    // **An enhanced object takes no article**, returning before the `a`/`an`
    // choice -- `RexxObject::defaultName` (`classes/ObjectClass.cpp:1763`).
    // Measured, oracle rc 0: `.Object~subclass('K')~enhanced(t)~defaultName`
    // is `enhanced K`.
    let enhanced = is_enhanced_instance(interp, receiver);
    let id = interp.classes().id_string(class);
    let name = if enhanced {
        format!("enhanced {id}")
    } else {
        crate::environment::default_object_name(id)
    };
    Ok(Some(interp.text_built(name.into_bytes())))
}

/// `Class~metaClass`: `RexxClass::getMetaClass` (`classes/ClassClass.cpp:419`).
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
        Primitive::StackFrame => model.stack_frame,
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
fn native_is_a(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let other = class_argument(interp, args)?;
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
    let other = class_argument(interp, args)?;
    let class = class_receiver(interp, receiver)?;
    let answer = interp.classes().is_a(class, other);
    Ok(Some(interp.counted(usize::from(answer))))
}

/// `Object~identityHash`.
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
fn operator_argument(args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    match args.first().copied() {
        Some(Some(argument)) => Ok(argument),
        _ => Err(Raised::missing_method_argument(1).into()),
    }
}

/// `Object~"="` and `Object~"=="`: `RexxObject::equal` and
/// `RexxObject::strictEqual` (`classes/ObjectClass.cpp:464`, `:448`), each a
/// direct identity test rather than a comparison of renderings.
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
        Some(MethodSlot::Defined { scope, method }) => {
            let record = crate::ExecutableRecord {
                source: interp.installed_executable_source(method),
                installed: Some(method),
                routine: None,
            };
            Ok(Some(interp.method_object(class, &name, scope, record)))
        }
    }
}

/// `Method~scope`: the class the method object was defined at, `.nil` for a
/// method object no class has taken -- [`Interp::method_scope`] carries the
/// C++ and the measurements.
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
fn is_source_line(interp: &Interp, value: ObjRef) -> bool {
    matches!(
        interp.receiver_kind(value),
        Ok(Primitive::String | Primitive::SmallInt)
    )
}

/// `MethodClass::newMethodObject`'s compiling arm
/// (`classes/MethodClass.cpp:462`-`:485`): the `Method` object a source text
/// becomes, carrying no scope, for a caller that is about to install it.
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

/// `RoutineClass::newRexx`'s compiling arm (`classes/RoutineClass.cpp:379`):
/// [`compile_method_source`] answering a `Routine` instead.
fn compile_routine_source(
    interp: &mut Interp,
    name: &[u8],
    source: ObjRef,
    position: &'static str,
) -> Result<ObjRef, Failure> {
    let lines = method_source_lines(interp, source, position)?;
    let borrowed: Vec<&[u8]> = lines.iter().map(Vec::as_slice).collect();
    let parsed = rexx_parse::parse_lines(&borrowed).map_err(|error| {
        Failure::from(Loud::method_from_source(&format!(
            "reporting a routine source that does not parse ({}, {error})",
            String::from_utf8_lossy(name)
        )))
    })?;
    // The oracle installs them and this crate does not: measured, oracle rc
    // 0, `.Routine~new('R', <four lines with a ::ROUTINE among them>)~call`
    // resolves the declared routine and answers. Refused loudly here rather
    // than dropped, and it is the refusal [`compile_method_source`] already
    // makes over the same shape.
    if !parsed.directives.is_empty() {
        return Err(Loud::method_from_source("a routine source that carries a directive").into());
    }
    let routine_class = interp.routine_class();
    let object = interp.native_instance(routine_class);
    let site = crate::environment::Annotated::Compiled(interp.compiled_methods);
    interp.compiled_methods += 1;
    interp.attach_annotations(object, site);
    interp.record_compiled_routine(object, name, parsed);
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
fn native_mixin_class_factory(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    class_factory(interp, receiver, args, ClassKind::Mixin)
}

/// `Object~new`: what every class that declares no `NEW` of its own answers.
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
fn class_factory(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    kind: ClassKind,
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let metaclass = factory_metaclass(interp, class, args)?;
    let name = class_id_argument(interp, args)?;
    let id = interp.mint_class();
    interp.classes().define_unregistered_class(
        id,
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
    interp.flag_class_uninit(id);
    let caller = interp.caller();
    interp.send_message(id, INIT, None, &[], caller)?;
    interp.classes().refresh_parent_has_uninit(id);
    Ok(Some(id))
}

/// The metaclass a class factory builds from: the second argument, or the
/// receiver's own where the send omits it (`classes/ClassClass.cpp:1566`-
/// `:1569`).
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
fn class_id_argument(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<Vec<u8>, Failure> {
    let outcome = required_class_id(interp, args);
    if outcome.is_err() {
        interp.blame_native_method(b"NEW", "Class");
    }
    outcome
}

/// [`class_id_argument`] without the frame, so that every way of failing
/// takes it.
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
    let Some(source) = interp.heap.is_class(argument).then_some(argument) else {
        return Err(
            Loud::setup_method("inheritInstanceMethods with a value that is not a class").into(),
        );
    };
    let source = class_receiver(interp, source)?;
    interp.classes().inherit_instance_methods(class, source);
    interp.classes().check_uninit(class);
    interp.flag_class_uninit(class);
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

/// `Package~addClass(name, class)` and `Package~addPublicClass(name, class)`
/// -- `PackageClass::addClassRexx` (`classes/PackageClass.cpp:1926`) and
/// `addPublicClassRexx` (`:1944`), which differ only in the flag they hand
/// `addInstalledClass`.
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
    if !interp.heap.is_class(class) {
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

/// An array receiver's own slots, borrowed, or the refusal for a receiver that
/// is not one.
fn array_slots(interp: &Interp, receiver: ObjRef) -> Result<&[Option<ObjRef>], Failure> {
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
fn array_slots_owned(interp: &Interp, receiver: ObjRef) -> Result<Vec<Option<ObjRef>>, Failure> {
    Ok(array_slots(interp, receiver)?.to_vec())
}

/// An array receiver's dimensions array as an owned copy, or the refusal
/// [`array_slots`] gives for a receiver that is not an array.
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
fn is_whole_method_argument(interp: &mut Interp, value: ObjRef) -> bool {
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
fn referenced_receiver(interp: &mut Interp, receiver: ObjRef) -> Result<ObjRef, Failure> {
    interp
        .referenced_object(receiver)
        .ok_or_else(|| Loud::receiver_class("a value that is not a variable reference").into())
}

/// `RexxObject::requestArray` (`runtime/MethodArguments.hpp:683`): the value
/// itself when it already is an array, and its `MAKEARRAY` otherwise.
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
        // The object's own `rendered` bytes rather than `string_value_text`,
        // which for this receiver answers the **traceback line**:
        // `StackFrameClass` overrides `stringValue()` and leaves
        // `defaultName()` alone, the split `NativeObject::string_value`
        // carries. Measured, oracle rc 0, six renderings of one frame:
        // `say f`, `~string`, `~makeString` and an `Array` join are the
        // traceback line where `~objectName` and `~defaultName` are
        // `a StackFrame`. Read rather than derived, so a name
        // `~objectName=` has set still wins -- measured, `f~objectName =
        // 'tagged'` then `f~objectName` is `tagged` while `say f` is
        // unchanged.
        Primitive::StackFrame => match interp.heap.get(receiver).map(|held| &held.body) {
            Some(Body::Native(native)) => native.rendered().to_vec(),
            _ => return Err(Loud::receiver_class("a stack frame this crate did not build").into()),
        },
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
        | Primitive::StackFrame
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
    // `enhanced_object->behaviour->setEnhanced()` (`:1478`), which
    // `RexxObject::defaultName` reads. **Unconditional, and not folded into
    // the walk above**: an empty table installs no method and still renders
    // `enhanced <id>` -- measured, oracle rc 0, `k~enhanced(.StringTable~new)`
    // answers `enhanced K` for `~string` and `~defaultName` alike.
    mark_enhanced_instance(interp, object);
    // `dummy_subclass->sendMessage(GlobalNames::NEW, args + 1, ...)`
    // (`:1470`), whose `INIT` send is `completeNewObject`'s -- the enhancing
    // `INIT` by then, since it is already in the behaviour.
    let caller = interp.caller();
    interp.send_message(object, INIT, None, &args[1..], caller)?;
    Ok(Some(object))
}

/// Marks `object` as one [`native_enhanced`] built, creating its
/// [`ObjectMethods`] when an empty enhancing table left it without one.
fn mark_enhanced_instance(interp: &mut Interp, object: ObjRef) {
    if let Some(Object {
        body: Body::Instance { own, .. },
        ..
    }) = interp.heap.get_mut(object)
    {
        own.get_or_insert_with(|| Box::new(ObjectMethods::new()))
            .mark_enhanced_instance();
    }
}

/// Whether `Class~enhanced` built `value`, which
/// [`crate::environment::default_object_name`] is not given because the
/// answer takes no article.
fn is_enhanced_instance(interp: &Interp, value: ObjRef) -> bool {
    matches!(
        interp.heap.get(value).map(|object| &object.body),
        Some(Body::Instance { own: Some(own), .. }) if own.is_enhanced_instance()
    )
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
pub(super) fn conversion_length_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<Option<usize>, Failure> {
    optional_length_argument(interp, args, 0)
}

/// A count argument that is zero or a positive whole number, answered as the
/// `i64` the numeric cores take.
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
pub(super) fn bit_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, Option<u8>), Failure> {
    let other = optional_string_method_argument(interp, args, 0)?;
    let pad = pad_method_argument(interp, args, 1)?;
    Ok((other, pad))
}

/// `DATATYPE`'s option letter, or `None` when it is omitted.
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
pub(super) fn strip_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(u8, Option<Vec<u8>>), Failure> {
    let option = option_method_argument(interp, args, 0, "BLT")?.unwrap_or(b'B');
    let set = optional_string_or_none_argument(interp, args, 1)?;
    Ok((option, set))
}

/// `EQUALS`'s and `CASELESSEQUALS`'s one operand, as its **string value**.
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
pub(super) fn abbrev_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, Option<usize>), Failure> {
    let info = string_method_argument(interp, args, 0)?;
    let minimum = optional_length_argument(interp, args, 1)?;
    Ok((info, minimum))
}

/// `COMPARE`'s other string and its pad.
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
    let routine = class == interp.routine_class();
    if args.len() > 2 || !(routine || class == interp.method_class()) {
        return Err(unbuilt_new(interp, class));
    }
    let name = interp.to_text(name).to_vec();
    let object = if routine {
        compile_routine_source(interp, &name, source, "source")?
    } else {
        compile_method_source(interp, &name, source, "source")?
    };
    Ok(Some(object))
}

/// `Method~newFile(name [, context])` and `Routine~newFile(name [, context])`:
/// the executable a file's own text becomes -- `MethodClass::newFileRexx`
/// (`classes/MethodClass.cpp:521`) and `RoutineClass::newFileRexx`
/// (`classes/RoutineClass.cpp:341`).
fn native_new_file(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let name = executable_name_argument(interp, args)?;
    let routine = class == interp.routine_class();
    if !routine && class != interp.method_class() {
        return Err(unbuilt_new(interp, class));
    }
    if args.get(1).copied().flatten().is_some() {
        return Err(Loud::executable_context().into());
    }
    let name = interp.to_text(name).to_vec();
    interp.new_file_executable(&name, routine).map(Some)
}

/// `MethodClass::loadExternalMethod(name, descriptor)` and
/// `RoutineClass::loadExternalRoutine` (`memory/Setup.cpp:1091`, `:1130`).
fn native_load_external(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let name = executable_name_argument(interp, args)?;
    let routine = class == interp.routine_class();
    if !routine && class != interp.method_class() {
        return Err(unbuilt_new(interp, class));
    }
    let Some(descriptor) = args.get(1).copied().flatten() else {
        return Err(Raised::missing_named_argument("descriptor").into());
    };
    let descriptor = required_string_named_argument(interp, descriptor, "descriptor")?;
    let descriptor = interp.to_text(descriptor).to_vec();
    let name = interp.to_text(name).to_vec();
    let Some((library, entry)) = external_specification(&descriptor, &name) else {
        return Err(Raised::bad_external_specification(&descriptor).into());
    };
    if routine {
        return Err(Loud::external_entry_point("loadExternalRoutine").into());
    }
    // **The keyword is case-insensitive and the library name is not**, which
    // is measured rather than assumed: oracle rc 0,
    // `library REXX file_separator` and `LiBrArY REXX file_separator` each
    // answer a `Method`, while `LIBRARY rexx file_separator` and
    // `LIBRARY Rexx file_separator` answer `.nil`.
    if library != b"REXX" {
        return Err(Loud::external_entry_point(
            "loadExternalMethod naming a library other than REXX",
        )
        .into());
    }
    if native::entry_point(&entry).is_none() {
        return Ok(Some(ObjRef::NIL));
    }
    let object = interp.native_instance(class);
    interp.record_native_executable(object);
    Ok(Some(object))
}

/// A `loadExternal*` descriptor split into its library and its entry point,
/// or `None` for one that is not an external name specification.
fn external_specification(descriptor: &[u8], name: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    let mut words = descriptor
        .split(|byte| byte.is_ascii_whitespace())
        .filter(|word| !word.is_empty());
    if !words.next()?.eq_ignore_ascii_case(b"LIBRARY") {
        return None;
    }
    let library = words.next()?.to_vec();
    let entry = words.next().map_or_else(|| name.to_vec(), <[u8]>::to_vec);
    if words.next().is_some() {
        return None;
    }
    Some((library, entry))
}

/// `Package~new(name, source, ...)`: the name is required, the source is not,
/// and this crate loads no package either way -- `PackageClass::newRexx`
/// (`classes/PackageClass.cpp:158`).
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

/// `Pointer~new` and `Buffer~new`: the raise that is the whole body of
/// `PointerClass::newRexx` (`classes/PointerClass.cpp:139`-`:143`) and
/// `BufferClass::newRexx` (`classes/BufferClass.cpp:86`-`:90`).
fn native_unsupported_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let id = interp.class_id_text(class).as_bytes().to_vec();
    Err(Raised::unsupported_new_method(&id).into())
}

/// The entry `WeakReference`'s scope pool binds the referent cell to, in the
/// position [`COLLECTION_STORES`]' entries are in.
const WEAK_REFERENT: &[u8] = b"REFERENT";

/// The cell `WEAK_REFERENT` holds: a `Body::WeakRef` allocated for `referent`.
fn weak_referent_cell(interp: &mut Interp, referent: ObjRef) -> ObjRef {
    let cell = interp.alloc_with(rexx_core::BehaviourId::OBJECT, Body::WeakRef(referent));
    interp.roots.push_temp(cell);
    cell
}

/// `WeakReference~new(value, ...)`: a reference that does not keep `value`
/// alive -- `WeakReference::newRexx` (`classes/WeakReferenceClass.cpp:231`).
fn native_weak_reference_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let Some(referent) = args.first().copied().flatten() else {
        return Err(Raised::missing_method_argument(1).into());
    };
    let scope = interp.object_model().weak_reference;
    let object = new_instance(interp, class)?;
    let cell = weak_referent_cell(interp, referent);
    interp.set_pool_variable(object, scope, WEAK_REFERENT, cell);
    let caller = interp.caller();
    let rest: Vec<Option<ObjRef>> = args.iter().skip(1).copied().collect();
    interp.send_message(object, INIT, None, &rest, caller)?;
    Ok(Some(object))
}

/// `WeakReference~value`: the referent, or `.nil` once the collector has
/// cleared it -- `WeakReference::value` (`classes/WeakReferenceClass.cpp:217`),
/// whose whole body is `resultOrNil(referentObject)`.
fn native_weak_reference_value(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let scope = interp.object_model().weak_reference;
    let cell = interp
        .pools_of(receiver)
        .and_then(|pools| pools.get(scope, WEAK_REFERENT));
    let referent = match cell.and_then(|cell| interp.heap.get(cell).map(|object| &object.body)) {
        Some(Body::WeakRef(referent)) => *referent,
        _ => ObjRef::NIL,
    };
    Ok(Some(referent))
}

/// `Object~request(class)`: the receiver converted to `class`, or `.nil` --
/// `RexxObject::requestRexx` (`classes/ObjectClass.cpp:1912`).
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
fn native_file_separator(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.text_built(b"/".to_vec())))
}

/// `file_path_separator`: the separator between the entries of a search path.
fn native_file_path_separator(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.text_built(b":".to_vec())))
}

/// `String~makeArray`: the receiver's lines, one array element each.
fn native_string_makearray(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let bytes = interp.to_text(receiver).to_vec();
    let lines = crate::builtin::string::line_slices(&bytes);
    // **Each line is rooted as it is built.** The `Vec` gathering them is a
    // Rust local and invisible to the collector, so a line built earlier is
    // swept by the allocation of the next one -- and, for a single-line
    // string, by the `alloc_with` below. Measured under
    // `run_program_collect_every_alloc`: `'.RESOURCES'~makeArray` then
    // `a[1]` panicked at `not_in_arena`'s "a live value", while `'abc'`
    // survived only because a short string is inline and never a heap object
    // at all.
    let mut slots: Vec<Option<ObjRef>> = Vec::with_capacity(lines.len());
    for line in lines {
        let text = interp.text_built(line.to_vec());
        interp.roots.push_temp(text);
        slots.push(Some(text));
    }
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
    fn both_engines(source: &str) -> (i32, String, String) {
        let mut answer = None;
        let outcome = crate::run_program(
            "/t.rex",
            source.as_bytes().to_vec(),
            crate::Invocation::none(),
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
        answer.expect("at least one engine ran")
    }

    /// **D24's `SmallInt` behaviour arm is taken for a small integer
    /// receiver**, where the general path is what a receiver whose bytes are
    /// in the arena takes.
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

    /// A `Stem` answers a message it has no method for by forwarding it to its
    /// VALUE -- `StemClass::unknownRexx`.
    #[test]
    fn a_stem_forwards_a_message_it_has_no_method_for_to_its_value() {
        let mut interp = Interp::new();
        let stem = interp.alloc_with(
            rexx_core::BehaviourId::STEM,
            Body::Stem {
                name: b"A."[..].into(),
                default: None,
                tails: rexx_core::NameMap::default(),
            },
        );
        let answered = interp
            .send_message(stem, b"LENGTH", None, &[], no_caller())
            .expect("a stem forwards an unknown message to its value")
            .expect("LENGTH answers a value");
        assert_eq!(interp.to_text(answered).as_ref(), b"2");
    }

    /// `~identityHash` answers, and **the corpus cannot witness it**: the
    /// oracle's answer is derived from the object's address, so no differential
    /// row can compare the two and this test is the whole instrument.
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
            ("Buffer", 163, "Error 93.967:"),
            ("Class", 163, "Error 93.901:"),
            ("Message", 163, "Error 93.901:"),
            ("Pointer", 163, "Error 93.967:"),
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
    #[test]
    fn a_constructor_taking_arguments_answers_an_instance_and_refuses_its_state() {
        let message = ".Message~new(.Object~new, 'STRING')";
        assert_eq!(
            both_engines(&format!("o = {message}\nsay o~class~id\n")),
            (0, "Message\n".to_string(), String::new())
        );
        let (code, stdout, stderr) = both_engines(&format!("o = {message}\nsay o~send\n"));
        assert_eq!((code, stdout.as_str()), (120, ""));
        assert!(stderr.starts_with("rexx-exec: "), "{stderr:?}");
        assert_eq!(
            both_engines("o = .MutableBuffer~new('abc')\nsay o~class~id\nsay o~length\n"),
            (0, "MutableBuffer\n3\n".to_string(), String::new())
        );
        // `WeakReference` keeps its referent and reads it back, so it is a
        // readback row rather than a refusal one. That its reference is *weak*
        // is not visible here and cannot be -- nothing on this path collects;
        // `tests/collect_stress.rs` carries that half.
        assert_eq!(
            both_engines(
                "k = .Object~new\n\
                 o = .WeakReference~new(k)\n\
                 say o~class~id (o~value == k) o~value~class~id\n"
            ),
            (0, "WeakReference 1 Object\n".to_string(), String::new())
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
