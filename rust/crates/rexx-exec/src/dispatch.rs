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

use crate::activation::{Activation, MethodIdentity, Replied, ReplyState, TraceEntry, body_of};
use crate::error::{FailureSite, Raised};
use crate::plan::{BodyKey, Package, ProgramId};
use crate::run::{MAX_ACTIVATION_DEPTH, Started};
use crate::scheduler::Scheduler;
use crate::{Failure, Interp, Loud};

/// The dispatch security seam.
mod seam {
    use super::{Failure, Interp, MethodId, ObjRef};

    /// Evidence that a message send passed the dispatch security seam.
    pub(super) struct Cleared(());

    /// What the seam decided about one send.
    pub(super) enum Clearance {
        /// Run the method.
        Cleared(Cleared),
        /// Run the method, which the security manager was asked about and
        /// did not answer: code it ran may have changed the receiver.
        Checked(Cleared),
        /// The security manager answered the send itself, and the method
        /// never runs (`RexxObject::processProtectedMethod`,
        /// `classes/ObjectClass.cpp:976`-`:988`).
        Answered(Option<ObjRef>),
    }

    /// **The dispatch chokepoint (D45, site one).** Every method invocation
    /// passes here, whatever kind [`super::Invocable`] finds it to be, and
    /// the security manager's `METHOD` checkpoint is asked here for a method
    /// the send has found to be `PROTECTED`.
    pub(super) fn clear(
        interp: &mut Interp,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
        method: MethodId,
    ) -> Result<Clearance, Failure> {
        let cleared = Cleared(());
        if interp.method_is_protected(method) {
            if let Some(result) = interp.check_protected_method(receiver, name, args)? {
                return Ok(Clearance::Answered(result));
            }
            return Ok(Clearance::Checked(cleared));
        }
        Ok(Clearance::Cleared(cleared))
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
// `ObjectModel::build` beside `NATIVE_METHODS` rather than merged into it,
// except the rows `NATIVE_METHODS` itself holds.
mod string;
use string::{
    native_length, native_reverse, native_string_make_string, native_string_makearray,
    native_string_sign, native_string_upper,
};

// `Stream`'s `LIBRARY REXX` entry points. A child of this module because a
// body takes the seam's `Cleared`, which cannot be named outside it.
pub(crate) mod stream;

// `.Alarm`'s and `.Ticker`'s `LIBRARY REXX` entry points.
mod time_support;

// `EventSemaphore`'s and `MutexSemaphore`'s methods.
mod semaphore;
use semaphore::{
    native_event_is_posted, native_event_post, native_event_reset, native_event_wait,
    native_mutex_acquire, native_mutex_release, native_mutex_uninit,
};

/// `.File`'s entry points, whose Rexx half hands each one an already-qualified
/// path.
mod files;
use files::{native_file_path_separator, native_file_separator};

// The collection classes' primitive methods, chained the same way.
mod collection;
pub(crate) mod hash;
use hash::{native_hash_at, native_hash_put, native_hash_unknown};

// `RexxInfo`'s readers, chained the same way.
mod rexx_info;

// `Method`'s and `Routine`'s own readers, chained the same way.
pub(crate) mod executable;

// `Class`'s graph readers and `Object`'s three, chained the same way.
mod introspection;

// `RexxContext`'s and `StackFrame`'s readers, chained the same way.
pub(crate) mod context;

// `Package`'s readers and its four writes, chained the same way.
mod package;
use package::{native_package_local, native_package_name};

// Running a procedure of a loaded shared library, and the `Host` the boundary
// reaches this interpreter through.
pub(crate) mod library;

// Array slot and subscript arithmetic, and `Array`'s primitive methods.
mod array;
use array::{
    IndexUse, MAX_FIXED_ARRAY_SIZE, array_dimensions, array_position, array_size_argument,
    array_slots, array_slots_owned, is_whole_method_argument, native_array_at, native_array_at_for,
    native_array_dimension, native_array_items, native_array_make_string, native_array_new,
    native_array_of, native_array_put, native_array_size, positive_index, request_array,
    unconverted_array_argument, unsigned_index,
};

// `MutableBuffer`'s primitive methods, and the argument and byte-search helpers
// the other primitive methods share, chained the same way.
mod buffer;
use buffer::{
    abbrev_arguments, array_of_texts, backward_search, backward_search_arguments, bit_arguments,
    case_shift_arguments, changestr_arguments, compare_arguments, compare_to_arguments,
    conversion_length_argument, copies_argument, count_method_argument, datatype_option_argument,
    delete_arguments, delword_arguments, ends_with, equals_argument, forward_search,
    forward_search_arguments, insert_arguments, match_region_arguments, match_region_over,
    native_mutable_buffer_new, overlay_arguments, pad_arguments, replace_at_bytes, replace_at_plan,
    space_arguments, starts_with, strip_arguments, substr_arguments, translate_arguments,
    translate_in_table, verify_arguments, wordpos_arguments,
};

// The method-argument parsers the primitive methods share.
mod method_arguments;
use method_arguments::{
    named_string_argument, option_method_argument, optional_length_argument,
    optional_position_argument, pad_method_argument, refuse_method_argument,
    required_position_argument, string_method_argument, usize_or_refuse, whole_method_argument,
};

// `Class`'s own methods: its readers, the mutators, the class factory.
mod class_protocol;
use class_protocol::{
    SourceTaker, begin_init, class_argument, class_receiver, compile_method_source,
    compile_method_source_in, compile_routine_source, is_enhanced_instance, method_name_argument,
    method_source_lines, native_annotation, native_annotations, native_base_class,
    native_class_copy, native_class_default_name, native_class_inherit, native_define,
    native_define_class_method, native_define_methods, native_delete, native_enhanced, native_id,
    native_inherit_instance_methods, native_is_subclass_of, native_metaclass, native_method,
    native_mixin_class_factory, native_new, native_new_class, native_package,
    native_package_add_class, native_package_add_public_class, native_scope, native_subclass,
    native_superclass, native_superclasses, native_uninherit, new_instance, scoped_method,
};

// The native constructors, and `Pointer`'s and `WeakReference`'s methods,
// chained the same way.
mod construct;
use construct::{
    native_capacity_init, native_directory_new, native_executable_new, native_hash_collection_new,
    native_load_external, native_message_new, native_new_file, native_package_new, native_stem_new,
    native_string_new, native_unsupported_new, native_weak_reference_new, pointer_address,
};

// `Object`'s own methods, `run`/`send`/`start`, and the `Message` readers.
mod object_protocol;
use object_protocol::{
    ArrayArgument, array_argument, decode_message_name, hash_value, invoke_executable,
    native_class, native_copy, native_default_name, native_has_method, native_hash_code,
    native_identity_hash, native_is_a, native_is_nil, native_message_arguments,
    native_message_completed, native_message_error_condition, native_message_halt,
    native_message_has_error, native_message_has_result, native_message_message_complete,
    native_message_name, native_message_notify, native_message_reply, native_message_reply_with,
    native_message_result, native_message_send, native_message_send_with, native_message_start,
    native_message_start_with, native_message_target, native_message_wait, native_no_op,
    native_object_concat, native_object_concat_blank, native_object_different,
    native_object_identical, native_object_name, native_object_name_set, native_request,
    native_run, native_send, native_send_with, native_set_method, native_start, native_start_with,
    native_string, native_unset_method, operator_argument, string_hash,
};

// The required-string protocol and the string conversion behind it.
mod reqstr;
pub(crate) use reqstr::MAKESTRING;
use reqstr::{DEFAULTNAME, OBJECTNAME, required_string_argument, required_string_named_argument};

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
    /// A `::METHOD`/`::ATTRIBUTE ... EXTERNAL 'LIBRARY <name>'` bound at
    /// install to a procedure of a loaded shared library.
    Library(crate::LibraryBinding),
}

/// Files one native method under its own id, growing the table to reach it.
fn put_native(natives: &mut Vec<Option<NativeEntry>>, method: MethodId, entry: NativeEntry) {
    let index = method.0 as usize;
    if index >= natives.len() {
        natives.resize(index + 1, None);
    }
    natives[index] = Some(entry);
}

/// The begin half of a primitive method that runs a Rexx activation or parks
/// the activity, which answers [`NativeStarted::Entered`] with that activation
/// pushed or the park on [`crate::activity::Activity::native_park`].
type NativeBegin =
    fn(&mut Interp, Cleared, ObjRef, &[Option<ObjRef>]) -> Result<NativeStarted, Failure>;

/// A `LIBRARY REXX` entry point that reads or writes the receiver's variables
/// in the method's scope, its fourth parameter, and may park as a
/// [`NativeBegin`] does.
type ScopedBegin =
    fn(&mut Interp, Cleared, ObjRef, ObjRef, &[Option<ObjRef>]) -> Result<NativeStarted, Failure>;

/// What [`Interp::invoke`] needs about one primitive method beyond its code.
#[derive(Copy, Clone)]
struct NativeEntry {
    arity: Arity,
    body: NativeBody,
}

/// One primitive method's code.
#[derive(Copy, Clone)]
enum NativeBody {
    Run(NativeMethod),
    Begin(NativeBegin),
}

/// What a [`NativeBegin`] left.
pub(crate) enum NativeStarted {
    Ran(Option<ObjRef>),
    /// A Rexx activation is pushed, or the activity's park recorded, and
    /// `Then` is the native's own work once it has ended or woken.
    Entered(Then),
}

/// A primitive method's work after the Rexx activation it entered has ended.
#[derive(Clone, Copy)]
pub(crate) enum Then {
    /// The activation's answer.
    Pass,
    /// This object, whatever the activation answered.
    Answer(ObjRef),
    /// This started message, with the activation's outcome recorded on it.
    Started(ObjRef),
    /// The activation's answer, with its outcome recorded on this message.
    Held(ObjRef),
    /// Nothing, with the activation's outcome recorded on this message.
    Triggered(ObjRef),
}

impl Then {
    /// Whether this records an outcome on `message`.
    fn records(&self, message: ObjRef) -> bool {
        matches!(self, Then::Started(object) | Then::Held(object) | Then::Triggered(object) if *object == message)
    }
}

/// A [`Then`] parked on [`Activity::native_tails`] until the activation whose
/// [`CallTail`] sits at `depth` ends.
///
/// [`Activity::native_tails`]: crate::activity::Activity
/// [`CallTail`]: crate::run::CallTail
pub(crate) struct NativeTail {
    /// `Activity::call_tails`' length with that tail on top.
    depth: usize,
    then: Then,
    /// The method's traceback line for a failure, where a send made it.
    blame: Option<NativeBlame>,
}

/// What [`Interp::blame_native_method`] names for one send.
struct NativeBlame {
    name: Box<[u8]>,
    scope: ObjRef,
    receiver: ObjRef,
    args: Box<[Option<ObjRef>]>,
}

impl Then {
    /// Appends the `ObjRef` this holds to `out`.
    pub(crate) fn object_roots(self, out: &mut Vec<ObjRef>) {
        match self {
            Then::Pass => {}
            Then::Answer(object)
            | Then::Started(object)
            | Then::Held(object)
            | Then::Triggered(object) => out.push(object),
        }
    }
}

impl NativeBlame {
    fn object_roots(&self, out: &mut Vec<ObjRef>) {
        out.extend([self.scope, self.receiver]);
        out.extend(self.args.iter().flatten().copied());
    }
}

impl NativeTail {
    /// Appends every `ObjRef` this tail holds to `out`.
    pub(crate) fn object_roots(&self, out: &mut Vec<ObjRef>) {
        self.then.object_roots(out);
        if let Some(blame) = &self.blame {
            blame.object_roots(out);
        }
    }

    /// Whether this records its activation's outcome on `message`.
    pub(crate) fn records(&self, message: ObjRef) -> bool {
        self.then.records(message)
    }
}

impl Interp {
    /// The `StackFrame` of the primitive method that entered the running
    /// activation (`InternalActivationFrame::createStackFrame`,
    /// `concurrency/ActivationFrame.cpp:100`) and its `Method` object (`.nil`
    /// where this crate has none), where one did.
    pub(crate) fn entering_native_frame(&mut self) -> Option<(ObjRef, ObjRef)> {
        let depth = self.activity.call_tails.len();
        let tail = self.activity.native_tails.last()?;
        let blame = tail.blame.as_ref().filter(|_| tail.depth == depth)?;
        let (name, scope, receiver, args) = (
            blame.name.clone(),
            blame.scope,
            blame.receiver,
            blame.args.clone(),
        );
        let scope_name = self.scope_id(scope);
        let trace_text = Raised::compiled_method_line(&name, &scope_name);
        let frame =
            context::build_native_level_frame(self, true, &name, receiver, &args, &trace_text);
        let executable = self.method_executable(scope, &name).unwrap_or(ObjRef::NIL);
        self.roots.activity_mut().push_temp(executable);
        Some((frame, executable))
    }
}

/// What a primitive method that parked answers once its activity wakes.
pub(crate) type NativeResume = fn(&mut Interp, ObjRef) -> Result<Option<ObjRef>, Failure>;

/// A send to a guarded method that is not a Rexx body, kept while its
/// activity waits for the guard lock.
pub(crate) struct GuardedSend {
    key: crate::guards::GuardKey,
    invocable: Invocable,
    cleared: Cleared,
    resolution: Resolution,
    receiver: ObjRef,
    name: Box<[u8]>,
    args: Vec<Option<ObjRef>>,
}

impl GuardedSend {
    pub(crate) fn object_roots(&self, out: &mut Vec<ObjRef>) {
        out.push(self.receiver);
        out.extend(self.args.iter().flatten().copied());
    }
}

/// The send a guard wait kept, run once the lock is granted.
fn resume_guarded_send(interp: &mut Interp, _receiver: ObjRef) -> Result<Option<ObjRef>, Failure> {
    let Some(send) = interp.activity.guarded_send.take() else {
        return Err(Loud::scheduler_inconsistency("a guard wait with no send recorded").into());
    };
    let GuardedSend {
        key,
        invocable,
        cleared,
        resolution,
        receiver,
        name,
        args,
    } = *send;
    let frame = interp.roots.activity_mut().push_frame();
    interp.roots.activity_mut().push_temp(receiver);
    for arg in args.iter().flatten() {
        interp.roots.activity_mut().push_temp(*arg);
    }
    let started = interp.run_other_guarded(
        Some(key),
        invocable,
        cleared,
        resolution,
        receiver,
        &name,
        &args,
    );
    interp.roots.activity_mut().pop_frame(frame);
    match started? {
        Started::Ran(value) => {
            if let Some(value) = value {
                interp.roots.activity_mut().push_temp(value);
            }
            Ok(value)
        }
        Started::Entered => Err(Loud::scheduler_inconsistency(
            "a guarded method that parked again after its guard wait",
        )
        .into()),
    }
}

/// A primitive method's park: why, its continuation, and the work of each
/// primitive method between it and the driver, innermost first.
pub(crate) struct NativePark {
    reason: crate::scheduler::ParkReason,
    resume: NativeResume,
    receiver: ObjRef,
    thens: Vec<(Then, Option<NativeBlame>)>,
}

impl NativePark {
    pub(crate) fn reason(&self) -> crate::scheduler::ParkReason {
        self.reason
    }

    pub(crate) fn set_reason(&mut self, reason: crate::scheduler::ParkReason) {
        self.reason = reason;
    }

    /// Whether a primitive method between the park and the driver records
    /// its outcome on `message`.
    pub(crate) fn records(&self, message: ObjRef) -> bool {
        self.thens.iter().any(|(then, _)| then.records(message))
    }

    pub(crate) fn object_roots(&self, out: &mut Vec<ObjRef>) {
        out.push(self.receiver);
        for (then, blame) in &self.thens {
            then.object_roots(out);
            if let Some(blame) = blame {
                blame.object_roots(out);
            }
        }
    }
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
    (
        "EventSemaphore",
        "ISPOSTED",
        Arity::Fixed(0),
        native_event_is_posted,
    ),
    ("EventSemaphore", "POST", Arity::Fixed(0), native_event_post),
    (
        "EventSemaphore",
        "RESET",
        Arity::Fixed(0),
        native_event_reset,
    ),
    // `EventSemaphoreClass::close` (`memory/Setup.cpp:1330`), which leaves
    // the post as it was: measured, a posted semaphore still answers
    // `isPosted` with `1` after its `UNINIT`.
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
        "ARGUMENTS",
        Arity::Fixed(0),
        native_message_arguments,
    ),
    (
        "Message",
        "COMPLETED",
        Arity::Fixed(0),
        native_message_completed,
    ),
    (
        "Message",
        "ERRORCONDITION",
        Arity::Fixed(0),
        native_message_error_condition,
    ),
    ("Message", "HALT", Arity::Fixed(1), native_message_halt),
    (
        "Message",
        "HASERROR",
        Arity::Fixed(0),
        native_message_has_error,
    ),
    (
        "Message",
        "HASRESULT",
        Arity::Fixed(0),
        native_message_has_result,
    ),
    (
        "Message",
        "MESSAGENAME",
        Arity::Fixed(0),
        native_message_name,
    ),
    ("Message", "NOTIFY", Arity::Fixed(1), native_message_notify),
    ("Message", "REPLY", Arity::Counted, native_message_reply),
    (
        "Message",
        "REPLYWITH",
        Arity::Fixed(2),
        native_message_reply_with,
    ),
    ("Message", "START", Arity::Counted, native_message_start),
    (
        "Message",
        "STARTWITH",
        Arity::Fixed(2),
        native_message_start_with,
    ),
    ("Message", "TARGET", Arity::Fixed(0), native_message_target),
    ("Method", "ANNOTATION", Arity::Fixed(1), native_annotation),
    ("Method", "ANNOTATIONS", Arity::Fixed(0), native_annotations),
    // `MethodClass::getScopeRexx`, `memory/Setup.cpp:1113`. `Routine` and
    // `Package` carry no such row: `Setup.cpp` binds `Scope` at `Method`
    // alone, and the scope is a `MethodClass` field rather than a
    // `BaseExecutable` one (`classes/MethodClass.hpp:168`).
    ("Method", "SCOPE", Arity::Fixed(0), native_scope),
    (
        "MutexSemaphore",
        "RELEASE",
        Arity::Fixed(0),
        native_mutex_release,
    ),
    (
        "MutexSemaphore",
        "UNINIT",
        Arity::Fixed(0),
        native_mutex_uninit,
    ),
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
        Arity::Fixed(1),
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

/// The primitive methods that send a message the receiver may answer with a
/// Rexx body, or that park the activity, entered by their begin halves.
static RESUMABLE_METHODS: &[(&str, &str, Arity, NativeBegin)] = &[
    // `MessageClass::messageCompleted`, bound under both names
    // (`memory/Setup.cpp:1073`-`:1074`).
    (
        "Message",
        "MESSAGECOMPLETE",
        Arity::Fixed(1),
        native_message_message_complete,
    ),
    (
        "Message",
        "TRIGGERED",
        Arity::Fixed(1),
        native_message_message_complete,
    ),
    ("Message", "RESULT", Arity::Fixed(0), native_message_result),
    ("Message", "SEND", Arity::Counted, native_message_send),
    (
        "Message",
        "SENDWITH",
        Arity::Fixed(2),
        native_message_send_with,
    ),
    ("Object", "SEND", Arity::Counted, native_send),
    ("Object", "SENDWITH", Arity::Fixed(2), native_send_with),
    ("Object", "START", Arity::Counted, native_start),
    ("Object", "STARTWITH", Arity::Fixed(2), native_start_with),
    ("Message", "WAIT", Arity::Fixed(0), native_message_wait),
    ("EventSemaphore", "WAIT", Arity::Fixed(1), native_event_wait),
    (
        "MutexSemaphore",
        "ACQUIRE",
        Arity::Fixed(1),
        native_mutex_acquire,
    ),
];

/// [`RESUMABLE_METHODS`] for the class dictionary: every `NEW` that sends
/// `INIT`.
static RESUMABLE_CLASS_METHODS: &[(&str, &str, Arity, NativeBegin)] = &[
    // `AddClassMethod("New", RexxObject::newRexx, A_COUNT)`,
    // `memory/Setup.cpp:514`, reached by every class whose own class
    // behaviour declares no `NEW` of its own, and the classes below whose
    // `newRexx` allocates a body this crate does not model.
    ("Object", "NEW", Arity::Counted, native_new),
    ("EventSemaphore", "NEW", Arity::Counted, native_new),
    ("List", "NEW", Arity::Counted, native_new),
    ("MutexSemaphore", "NEW", Arity::Counted, native_new),
    ("Queue", "NEW", Arity::Counted, native_new),
    ("Supplier", "NEW", Arity::Counted, native_new),
    // `AddClassMethod("New", ArrayClass::newRexx, A_COUNT)`,
    // `memory/Setup.cpp:708`. A row of its own for the reason
    // `StringTable`'s below is one: the answer is a body this crate builds
    // rather than an instance.
    ("Array", "NEW", Arity::Counted, native_array_new),
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
    ("Bag", "NEW", Arity::Counted, hash::native_hash_new),
    // `TableClass::newRexx` and `IdentityTable::newRexx` take an optional
    // initial capacity, and it is observable: it decides the bucket count,
    // which decides the order every iteration answers in.
    (
        "IdentityTable",
        "NEW",
        Arity::Counted,
        hash::native_hash_new,
    ),
    ("Relation", "NEW", Arity::Counted, hash::native_hash_new),
    ("Set", "NEW", Arity::Counted, hash::native_hash_new),
    ("Table", "NEW", Arity::Counted, hash::native_hash_new),
    // `AddClassMethod("New", RexxClass::newRexx, A_COUNT)`,
    // `memory/Setup.cpp:450`, the one class method `.Class` adds.
    ("Class", "NEW", Arity::Counted, native_new_class),
    ("Message", "NEW", Arity::Counted, native_message_new),
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

/// The primitive methods bound to a class's **class** dictionary rather than
/// its instance one -- `memory/Setup.cpp`'s `AddClassMethod` rows.
static NATIVE_CLASS_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    // `AddClassMethod("Of", ArrayClass::ofRexx, A_COUNT)`,
    // `memory/Setup.cpp:709`. The same body as `Array~new`'s, filled from the
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
    // `AddClassMethod("Of", BagClass::ofRexx, A_COUNT)`, which unlike
    // `Set~of` keeps the duplicates.
    ("Bag", "OF", Arity::Counted, hash::native_bag_of),
    // `AddClassMethod("Of", SetClass::ofRexx, A_COUNT)`: `Set`'s own, unlike
    // the mapped classes whose `of` is `MapCollection~OF` in Rexx.
    ("Set", "OF", Arity::Counted, hash::native_set_of),
    // The classes whose own `newRexx` checks its arguments and then builds
    // something this crate does not -- a compiled executable, a loaded
    // package (`memory/Setup.cpp:1091`, `:1128`, `:1156`). Each checks
    // exactly what the C++ checks before that point and refuses loudly past
    // it.
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
    // The classes whose `newRexx` is the refusal and nothing else, because
    // their instances come only from native code (`utilityclasses.xml:429`,
    // `:6910`) or from the `>name` operator
    // (`classes/VariableReference.cpp:97`, `memory/Setup.cpp:1294`).
    ("Buffer", "NEW", Arity::Counted, native_unsupported_new),
    ("Pointer", "NEW", Arity::Counted, native_unsupported_new),
    (
        "VariableReference",
        "NEW",
        Arity::Counted,
        native_unsupported_new,
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

/// The entry a `Message` object keeps the condition object its send failed
/// with under.
pub(crate) const MESSAGE_CONDITION: &[u8] = b"ERRORCONDITION";

/// The entries a `Message~new` object keeps its unsent message in:
/// `MessageClass`'s `receiver`, `message`, `startscope` and `args`.
const MESSAGE_TARGET: &[u8] = b"TARGET";
const MESSAGE_NAME: &[u8] = b"MESSAGENAME";
const MESSAGE_SCOPE: &[u8] = b"SCOPE";
const MESSAGE_ARGUMENTS: &[u8] = b"ARGUMENTS";

/// The entry a `Message` object keeps the `Array` of objects `~notify` named
/// under -- `MessageClass`'s `interestedParties`.
const MESSAGE_PARTIES: &[u8] = b"NOTIFY";

/// The message a class construction sends the class it just built --
/// `GlobalNames::INIT`, sent by `RexxClass::subclass`
/// (`classes/ClassClass.cpp:1631`). Upper case for [`UNKNOWN`]'s reason.
pub(crate) const INIT: &[u8] = b"INIT";

/// The message `SAY` sends whatever `.OUTPUT` holds -- `Activity::sayOutput`
/// (`concurrency/Activity.cpp:3214`), whose reply is dropped. Upper case for
/// [`UNKNOWN`]'s reason.
pub(crate) const SAY: &[u8] = b"SAY";

/// The message a trace line and an error report line send whatever
/// `.TRACEOUTPUT` holds -- `Activity::traceOutput`
/// (`concurrency/Activity.cpp:3171`). Upper case for [`UNKNOWN`]'s reason.
pub(crate) const LINEOUT: &[u8] = b"LINEOUT";

/// The message a line read with no stream name sends whatever `.INPUT` holds.
/// Measured, `PARSE PULL` sends this too rather than a `PULL` of its own.
/// Upper case for [`UNKNOWN`]'s reason.
pub(crate) const LINEIN: &[u8] = b"LINEIN";

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
    /// The scope a mapped collection's store is bound under.
    table: ObjRef,
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
    /// The scope [`construct::WEAK_REFERENT`] is bound in, so that a subclass's instance
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
        for (class_id, method_name, arity, body) in NATIVE_METHODS
            .iter()
            .chain(buffer::NATIVE_METHODS)
            .chain(construct::NATIVE_METHODS)
            .chain(string::NATIVE_METHODS)
            .chain(hash::NATIVE_METHODS)
            .chain(hash::relation::NATIVE_METHODS)
            .chain(hash::stem::NATIVE_METHODS)
            .chain(collection::list::NATIVE_METHODS)
            .chain(collection::queue::NATIVE_METHODS)
            .chain(array::sort::NATIVE_METHODS)
            .chain(array::surface::NATIVE_METHODS)
            .chain(collection::supplier::NATIVE_METHODS)
            .chain(rexx_info::NATIVE_METHODS)
            .chain(executable::NATIVE_METHODS)
            .chain(introspection::NATIVE_METHODS)
            .chain(context::NATIVE_METHODS)
            .chain(package::NATIVE_METHODS)
            .chain(extra)
            .map(|(class_id, method_name, arity, run)| {
                (class_id, method_name, arity, NativeBody::Run(*run))
            })
            .chain(
                RESUMABLE_METHODS
                    .iter()
                    .chain(executable::RESUMABLE_METHODS)
                    .map(|(class_id, method_name, arity, begin)| {
                        (class_id, method_name, arity, NativeBody::Begin(*begin))
                    }),
            )
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
                    body,
                },
            );
        }
        for (class_id, method_name, arity, body) in NATIVE_CLASS_METHODS
            .iter()
            .chain(context::NATIVE_CLASS_METHODS)
            .chain(package::NATIVE_CLASS_METHODS)
            .map(|(class_id, method_name, arity, run)| {
                (class_id, method_name, arity, NativeBody::Run(*run))
            })
            .chain(
                RESUMABLE_CLASS_METHODS
                    .iter()
                    .chain(package::RESUMABLE_CLASS_METHODS)
                    .map(|(class_id, method_name, arity, begin)| {
                        (class_id, method_name, arity, NativeBody::Begin(*begin))
                    }),
            )
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
                    body,
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
        let table = classes.lookup("Table").expect("Table is a native class");
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
            table,
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
    /// A `Body::Native` whose class is `.Directory`, such as a condition
    /// object.
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
            receiver: self.activity.call_context.receiver,
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
    /// unmodified -- and `.NIL` for the `.nil` scope a method `setMethod` or
    /// `run` installed carries, which is how the oracle's `>I>` line and
    /// traceback name it: measured, oracle rc 0, `>I> Method "HM" with scope
    /// ".NIL"`.
    pub(crate) fn class_id_text(&self, class: ObjRef) -> &str {
        if class == ObjRef::NIL {
            return ".NIL";
        }
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
    /// `environment/identities.rs` builds one of per package.
    pub(crate) fn package_class(&mut self) -> ObjRef {
        self.object_model().package
    }

    /// `.Method`: the class of the object `Class~method` answers, which
    /// `environment/identities.rs` builds one of per instance dictionary entry a program
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

    /// [`Interp::resolve_in`] with the receiver's behaviour looked up.
    #[cfg(test)]
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
        self.admit(resolution, receiver, caller)
    }

    /// **Step one of a send** (D24): which method a name reaches on this
    /// receiver, whose behaviour is `behaviour`, and from which scope, or why
    /// the send has none to run.
    fn resolve_in(
        &mut self,
        receiver: ObjRef,
        behaviour: Behaviour,
        name: &[u8],
        start_scope: Option<ObjRef>,
        caller: Caller,
    ) -> Result<Resolution, Miss> {
        let found = match self.own_lookup(receiver, name, start_scope) {
            Some(found) => found,
            None => self.lookup_behaviour(behaviour, name, start_scope),
        };
        let Some(resolution) = found else {
            return Err(Miss::NoMethod);
        };
        self.admit(resolution, receiver, caller)
    }

    /// `resolution` once its method's access admits a send from `caller`.
    fn admit(
        &mut self,
        resolution: Resolution,
        receiver: ObjRef,
        caller: Caller,
    ) -> Result<Resolution, Miss> {
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
        if let Some(found) = self.own_lookup(receiver, name, start_scope) {
            return found;
        }
        let behaviour = self.receiver_behaviour(receiver).ok()?;
        self.lookup_behaviour(behaviour, name, start_scope)
    }

    /// What `receiver`'s own methods answer for `name` ahead of its
    /// behaviour: `None` where they do not decide it.
    fn own_lookup(
        &self,
        receiver: ObjRef,
        name: &[u8],
        start_scope: Option<ObjRef>,
    ) -> Option<Option<Resolution>> {
        if start_scope.is_none()
            && let Some(entry) = self.own_method_entry(receiver, name)
        {
            return Some(
                entry.map(|ObjectMethod { method, scope, .. }| Resolution { scope, method }),
            );
        }
        None
    }

    /// [`Interp::lookup`] in `behaviour`, past the receiver's own methods.
    fn lookup_behaviour(
        &mut self,
        behaviour: Behaviour,
        name: &[u8],
        start_scope: Option<ObjRef>,
    ) -> Option<Resolution> {
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
            Body::Class { .. } => self.class_own_entry(receiver, name),
            _ => None,
        }
    }

    /// [`Interp::own_method_entry`] for a class object, whose own methods
    /// sit on the object holding its variable pools ([`Interp::pool_owner`]).
    #[cold]
    #[inline(never)]
    fn class_own_entry(&self, class: ObjRef, name: &[u8]) -> Option<Option<ObjectMethod>> {
        let owner = self.class_variables.get(&class)?;
        match &self.heap.get(*owner)?.body {
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
        let holder = match self.receiver_kind(receiver) {
            Ok(Primitive::Instance { .. }) => receiver,
            // Measured, oracle rc 0: a class method's `self~setMethod('m2',
            // 'return 42')` then `.K~m2` answers `42`, and `unsetMethod`
            // from one answers.
            Ok(Primitive::Class(_)) => self.pool_owner(receiver)?,
            Ok(_) => return Err(Loud::object_method("a receiver with no scope of its own").into()),
            Err(kind) => return Err(Loud::unknown_receiver(kind).into()),
        };
        let Some(Object {
            body: Body::Instance { own, .. },
            ..
        }) = self.heap.get_mut(holder)
        else {
            unreachable!("an instance receiver and a class's pool owner are Body::Instance")
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
        Some(self.scope_id(resolution.scope))
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

    /// The security manager's `checkProtectedMethod`
    /// (`execution/SecurityManager.cpp:172`-`:194`), asked for a method the
    /// send has found to be `PROTECTED`.
    ///
    /// `Some(result)` where the manager handled the send, the method then
    /// never running; `None` where it did not, or where no manager is
    /// installed.
    fn check_protected_method(
        &mut self,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<Option<Option<ObjRef>>, Failure> {
        if self.effective_security_manager().is_none() {
            return Ok(None);
        }
        let message = self.text(name);
        self.roots.activity_mut().push_temp(message);
        let arguments = self.security_arguments_array(args);
        let entries = [
            (crate::security::key::OBJECT, receiver),
            (crate::security::key::NAME, message),
            (crate::security::key::ARGUMENTS, arguments),
        ];
        let Some(info) = self.security_check(crate::security::message::METHOD, &entries)? else {
            return Ok(None);
        };
        let result = self.security_entry(info, crate::security::key::RESULT)?;
        Ok(Some(result))
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
    pub(crate) fn class_of_value(&mut self, value: ObjRef) -> Option<ObjRef> {
        match self.receiver_behaviour(value).ok()? {
            Behaviour::Instance { owner, .. } => Some(owner),
            Behaviour::ClassSide(class) => Some(self.classes().class_of(class)),
        }
    }

    /// Mints a method identity that runs what `template` runs, with its
    /// access scope and guard: the identity a copy `MethodClass::newScope`
    /// makes is installed under, so the copy's own flags stay its own.
    pub(crate) fn copy_method_identity(&mut self, template: MethodId) -> MethodId {
        let method = self.classes().mint_method_id();
        let natives = &mut self.object_model().natives;
        if let Some(entry) = natives.get(template.0 as usize).copied().flatten() {
            put_native(natives, method, entry);
        }
        if let Some(body) = self.method_bodies.get(&template).copied() {
            self.method_bodies.insert(method, body);
        }
        if let Some(generated) = self.generated_methods.get(&template).copied() {
            self.generated_methods.insert(method, generated);
        }
        if let Some(entry) = self.native_externals.get(&template).copied() {
            self.native_externals.insert(method, entry);
        }
        if let Some(binding) = self.library_externals.get(&template).cloned() {
            self.library_externals.insert(method, binding);
        }
        if let Some(program) = self.external_packages.get(&template).copied() {
            self.external_packages.insert(method, program);
        }
        if let Some(code) = self.defined_library_codes.get(&template).copied() {
            self.defined_library_codes.insert(method, code);
        }
        if self.guarded_externals.contains(&template) {
            self.guarded_externals.insert(method);
        }
        if let Some(row) = self.special_method_row(template).copied() {
            *self.special_method_row_mut(method) = Some(row);
        }
        method
    }

    /// A method scope's `~id`, [`Interp::class_id_text`]'s rendering:
    /// measured, oracle rc 163, a primitive `setMethod` or `run` installed
    /// under the `.nil` scope is `Compiled method "HM" with scope ".NIL"`.
    pub(crate) fn scope_id(&mut self, scope: ObjRef) -> String {
        self.object_model();
        self.class_id_text(scope).to_string()
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
        self.super_scope_of(receiver, resolution.scope)
    }

    /// [`Interp::super_scope_for`] for a method whose scope is `scope`.
    pub(crate) fn super_scope_of(&mut self, receiver: ObjRef, scope: ObjRef) -> Option<ObjRef> {
        let behaviour = self.receiver_behaviour(receiver).ok()?;
        self.super_scope_in(behaviour, scope)
    }

    /// [`Interp::super_scope_of`] in `behaviour`, the receiver's.
    fn super_scope_in(&mut self, behaviour: Behaviour, scope: ObjRef) -> Option<ObjRef> {
        match behaviour {
            Behaviour::Instance { methods, .. } => self.classes().super_scope_at(methods, scope),
            Behaviour::ClassSide(class) => self.classes().class_super_scope(class, scope),
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
        if let Some(binding) = self.library_externals.get(&resolution.method) {
            return Ok(Invocable::Library(binding.clone()));
        }
        // The scope's `~id` is rendered only where it is printed -- a
        // successful send has no use for it, and every send would otherwise
        // pay for the copy.
        let scope = self.scope_id(resolution.scope);
        Err(Loud::native_method(name, &scope).into())
    }

    /// **Step two of a send** (D24): run what [`Interp::resolve_in`] found.
    pub(crate) fn invoke(
        &mut self,
        resolution: Resolution,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        let started = self.begin_invoke(resolution, None, receiver, name, args)?;
        self.complete_send(started)
    }

    /// [`Interp::invoke`] up to the point a Rexx body it enters would run,
    /// with `behaviour` the receiver's where the caller has it.
    fn begin_invoke(
        &mut self,
        resolution: Resolution,
        behaviour: Option<Behaviour>,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<Started<Option<ObjRef>>, Failure> {
        let invocable = self.invocable(resolution, name)?;
        let (cleared, behaviour) = match seam::clear(self, receiver, name, args, resolution.method)?
        {
            seam::Clearance::Cleared(cleared) => (cleared, behaviour),
            seam::Clearance::Checked(cleared) => (cleared, None),
            seam::Clearance::Answered(result) => return Ok(Started::Ran(result)),
        };
        if let Invocable::Rexx(installed) = invocable {
            // **Neither this path nor `Generated` blames the method**,
            // measured twice over.
            // An untrapped `1/0` inside a `::METHOD` body reports
            // the method's own failing clause and then the sending clause,
            // with no `Compiled method` line between them, and
            // [`Interp::finish_call`] seals the method's own level for that,
            // exactly as it does for a call. A generated accessor and
            // an `ABSTRACT` send report the *sending* clause and nothing
            // above it: measured, `.K~a(1)` on `::attribute a class` is
            // `93.902` with the sending clause alone on `stderr`, where
            // `'abc'~length(1)` -- a [`NativeMethod`] taking the same
            // refusal -- carries a `Compiled method "LENGTH"` line. The two
            // differ in the C++ because `AttributeGetterCode::run` and
            // `AbstractCode::run` raise directly where `CPPCode::run` raises
            // from inside a `NativeActivation` of its own
            // (`execution/CPPCode.cpp:280`, `:526`).
            self.begin_method(
                cleared, installed, resolution, behaviour, receiver, name, args,
            )?;
            return Ok(Started::Entered);
        }
        let outcome = match invocable {
            Invocable::Native(entry) => {
                let outcome = match (entry.arity, entry.body) {
                    (Arity::Fixed(arity), _) if args.len() > arity => {
                        Err(Raised::too_many_method_arguments(arity).into())
                    }
                    (_, NativeBody::Run(run)) => {
                        let ran = pinned!(
                            self,
                            crate::pinning::PinKind::native(name),
                            run(self, cleared, receiver, args)
                        );
                        self.serve_posted_halt();
                        ran.map(NativeStarted::Ran)
                    }
                    (_, NativeBody::Begin(begin)) => begin(self, cleared, receiver, args),
                };
                match outcome {
                    Ok(NativeStarted::Ran(value)) => Ok(value),
                    Ok(NativeStarted::Entered(then)) => {
                        let blame = NativeBlame {
                            name: name.into(),
                            scope: resolution.scope,
                            receiver,
                            args: args.into(),
                        };
                        match self.activity.native_park.as_mut() {
                            Some(park) => park.thens.push((then, Some(blame))),
                            None => self.push_native_tail(then, Some(blame)),
                        }
                        return Ok(Started::Entered);
                    }
                    Err(failure) => {
                        let scope = self.scope_id(resolution.scope);
                        self.blame_native_method(name, &scope, receiver, args);
                        Err(failure)
                    }
                }
            }
            other => {
                return self.begin_invoke_other(other, cleared, resolution, receiver, name, args);
            }
        };
        outcome.map(Started::Ran)
    }

    /// [`Interp::begin_invoke`] for a method whose body is neither Rexx nor
    /// native, holding its guard lock where it is guarded.
    #[inline(never)]
    fn begin_invoke_other(
        &mut self,
        invocable: Invocable,
        cleared: Cleared,
        resolution: Resolution,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<Started<Option<ObjRef>>, Failure> {
        let guard = self.other_guard(&invocable, resolution, receiver);
        if let Some(key) = guard
            && let crate::guards::Reserve::Contended(owner) = self.take_guard(key)
        {
            let send = GuardedSend {
                key,
                invocable,
                cleared,
                resolution,
                receiver,
                name: name.into(),
                args: args.to_vec(),
            };
            return self.wait_for_other_guard(owner, send);
        }
        self.run_other_guarded(guard, invocable, cleared, resolution, receiver, name, args)
    }

    /// The guard lock a send to `invocable` reserves: a generated getter,
    /// setter or `DELEGATE` and an external method, where guarded
    /// (`execution/CPPCode.cpp:288-298`, `:344-358`, `:612-620`;
    /// `NativeActivation::methodVariables`).
    fn other_guard(
        &self,
        invocable: &Invocable,
        resolution: Resolution,
        receiver: ObjRef,
    ) -> Option<crate::guards::GuardKey> {
        let guarded = match invocable {
            Invocable::Generated(generated) => {
                matches!(
                    generated.kind,
                    crate::GeneratedKind::Getter
                        | crate::GeneratedKind::Setter
                        | crate::GeneratedKind::Delegate
                ) && self.programs[generated.program.0]
                    .directives
                    .get(generated.directive)
                    .is_some_and(|directive| crate::guards::directive_guarded(&directive.kind))
            }
            Invocable::External(_) | Invocable::Library(_) => {
                self.guarded_externals.contains(&resolution.method)
            }
            Invocable::Rexx(_) | Invocable::Native(_) => false,
        };
        let guarded = match invocable {
            Invocable::Generated(_) | Invocable::External(_) | Invocable::Library(_) => {
                self.sends_guarded(resolution.method, guarded)
            }
            Invocable::Rexx(_) | Invocable::Native(_) => false,
        };
        guarded.then_some(crate::guards::GuardKey {
            object: receiver,
            scope: resolution.scope,
        })
    }

    /// `send`'s wait for its guard lock, owned by `owner`: 98.905 where the
    /// waits it would join lead back here, else parked until a release
    /// grants it, then run.
    #[cold]
    #[inline(never)]
    fn wait_for_other_guard(
        &mut self,
        owner: crate::scheduler::ActivityId,
        send: GuardedSend,
    ) -> Result<Started<Option<ObjRef>>, Failure> {
        if self.deadlocks(owner) {
            return Err(Raised::deadlock().into());
        }
        let (key, receiver) = (send.key, send.receiver);
        let me = self.running_activity();
        self.activities.guards.enqueue(key, me);
        self.activity.guarded_send = Some(Box::new(send));
        match self.park_native(
            crate::scheduler::ParkReason::Guard(key),
            resume_guarded_send,
            receiver,
        ) {
            Ok(NativeStarted::Ran(value)) => Ok(Started::Ran(value)),
            Ok(NativeStarted::Entered(_)) => Ok(Started::Entered),
            Err(failure) => {
                self.activity.guarded_send = None;
                if self.granted(key) {
                    self.release_guard(key);
                }
                Err(failure)
            }
        }
    }

    /// [`Interp::run_other`] holding `guard`, which it releases: a
    /// `DELEGATE` method's once it has read its target, before the send
    /// (`DelegateCode::run`), anything else's once it has run.
    #[expect(
        clippy::too_many_arguments,
        reason = "the send's own parts, and the lock it holds"
    )]
    fn run_other_guarded(
        &mut self,
        guard: Option<crate::guards::GuardKey>,
        invocable: Invocable,
        cleared: Cleared,
        resolution: Resolution,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<Started<Option<ObjRef>>, Failure> {
        let Some(key) = guard else {
            return self.run_other(invocable, cleared, resolution, receiver, name, args, false);
        };
        if let Invocable::Generated(generated) = &invocable
            && generated.kind == crate::GeneratedKind::Delegate
        {
            let target = self.delegate_target(*generated, resolution, receiver);
            self.release_guard(key);
            return self
                .send_to_delegate_target(target?, name, args)
                .map(Started::Ran);
        }
        // A library method's own frame holds the lock, which `SetGuardOff`
        // may give up, and releases it where it still holds it.
        let library = matches!(invocable, Invocable::Library(_));
        let outcome = self.run_other(invocable, cleared, resolution, receiver, name, args, true);
        if !library {
            self.release_guard(key);
        }
        outcome
    }

    /// The body of a method that is neither Rexx nor native.
    #[expect(
        clippy::too_many_arguments,
        reason = "the send's own parts, and whether it holds its guard lock"
    )]
    fn run_other(
        &mut self,
        invocable: Invocable,
        cleared: Cleared,
        resolution: Resolution,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
        reserved: bool,
    ) -> Result<Started<Option<ObjRef>>, Failure> {
        let outcome = match invocable {
            // **The refusal is the ordinary outcome here**, so unlike the
            // `Native` arm of `begin_invoke` this one does not blame the
            // method for it:
            // a `Loud` is a report about this crate and carries no traceback
            // at all. The implemented arm does blame, because its refusals
            // are the oracle's own -- measured, `.k~sep(1)` on a class method
            // bound to `file_separator` is `88.922 Too many arguments in
            // invocation; 0 expected.` at rc 168, under a `Compiled method
            // "SEP" with scope "K".` line.
            Invocable::External(entry) => match &entry.body {
                native::ExternalBody::Deferred { owner } => {
                    Err(native::deferred_send(entry, owner).into())
                }
                native::ExternalBody::Implemented { arity, run } => {
                    self.run_external(*arity, resolution, receiver, name, args, |interp| {
                        pinned!(
                            interp,
                            crate::pinning::PinKind::native(name),
                            run(interp, cleared, receiver, args)
                        )
                    })
                }
                native::ExternalBody::Scoped { arity, begin } => {
                    let started =
                        self.run_external(*arity, resolution, receiver, name, args, |interp| {
                            begin(interp, cleared, receiver, resolution.scope, args)
                        })?;
                    match started {
                        NativeStarted::Ran(value) => Ok(value),
                        NativeStarted::Entered(then) => {
                            match self.activity.native_park.as_mut() {
                                Some(park) => park.thens.push((then, None)),
                                None => self.push_native_tail(then, None),
                            }
                            return Ok(Started::Entered);
                        }
                    }
                }
            },
            Invocable::Rexx(_) | Invocable::Native(_) => return Err(Loud::missing_body().into()),
            Invocable::Library(binding) => {
                let outcome = match self
                    .run_library_method(&binding, resolution, receiver, name, args, reserved)
                {
                    Ok(Started::Entered) => return Ok(Started::Entered),
                    Ok(Started::Ran(value)) => Ok(value),
                    Err(failure) => Err(failure),
                };
                if outcome.is_err() {
                    let scope = self.scope_id(resolution.scope);
                    let method = resolution.method;
                    let reraised = std::mem::take(&mut self.activity.native_reraise);
                    self.blame_external_method(name, &scope, method, receiver, args, reraised);
                }
                outcome
            }
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
        };
        outcome.map(Started::Ran)
    }

    /// A `LIBRARY REXX` entry's `run` after the arity check `CPPCode`-style
    /// externals make, with the method blamed for a failure.
    fn run_external<T>(
        &mut self,
        arity: Arity,
        resolution: Resolution,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
        run: impl FnOnce(&mut Interp) -> Result<T, Failure>,
    ) -> Result<T, Failure> {
        let outcome = match arity {
            Arity::Fixed(arity) if args.len() > arity => {
                Err(Raised::too_many_external_arguments(arity).into())
            }
            Arity::Fixed(_) | Arity::Counted => run(self),
        };
        if let Err(failure) = &outcome {
            let scope = self.scope_id(resolution.scope);
            let method = resolution.method;
            let reraised = failure.reraised_by_native_call();
            self.blame_external_method(name, &scope, method, receiver, args, reraised);
        }
        outcome
    }

    /// Runs a started send's Rexx body on this Rust stack, and answers the
    /// send's value.
    pub(crate) fn complete_send(
        &mut self,
        started: Started<Option<ObjRef>>,
    ) -> Result<Option<ObjRef>, Failure> {
        match started {
            Started::Ran(value) => Ok(value),
            Started::Entered => {
                if self.activity.native_park.is_some() {
                    return Err(self.refuse_unrooted_park());
                }
                let ended = self.run_activation();
                self.finish_send(ended)
            }
        }
    }

    /// Answers a park for `reason`, whose continuation is `resume` on
    /// `receiver`. Where a Rust frame pins the activity, the wait runs the
    /// other activities here and the continuation's answer is the native's.
    pub(crate) fn park_native(
        &mut self,
        reason: crate::scheduler::ParkReason,
        resume: NativeResume,
        receiver: ObjRef,
    ) -> Result<NativeStarted, Failure> {
        if self.activity.pin_depth > 0 {
            let frame = self.roots.activity_mut().push_frame();
            self.roots.activity_mut().push_temp(receiver);
            let mut reason = reason;
            let sent = loop {
                if let Some(failure) = self.pinned_wait(reason) {
                    break Err(failure);
                }
                match self.retest_wait(reason) {
                    Some(again) => reason = again,
                    None => break resume(self, receiver),
                }
            };
            self.roots.activity_mut().pop_frame(frame);
            return sent.map(NativeStarted::Ran);
        }
        self.activity.native_park = Some(Box::new(NativePark {
            reason,
            resume,
            receiver,
            thens: Vec::new(),
        }));
        Ok(NativeStarted::Entered(Then::Pass))
    }

    /// A park for `reason` by an instruction its op runs again once the
    /// activity wakes, recorded on the activity for the root driver.
    #[cold]
    #[inline(never)]
    pub(crate) fn park_instruction(&mut self, reason: crate::scheduler::ParkReason) {
        self.activity.native_park = Some(Box::new(NativePark {
            reason,
            resume: |_, _| Ok(None),
            receiver: ObjRef::NIL,
            thens: Vec::new(),
        }));
    }

    /// A park for `reason` by a routine whose answer, once woken, is what
    /// `resume` answers for `receiver`: that answer after a pinned wait, or
    /// `receiver` with the park recorded on the activity.
    pub(crate) fn park_routine_with(
        &mut self,
        reason: crate::scheduler::ParkReason,
        resume: NativeResume,
        receiver: ObjRef,
    ) -> Result<ObjRef, Failure> {
        Ok(match self.park_native(reason, resume, receiver)? {
            NativeStarted::Ran(answer) => answer.unwrap_or(receiver),
            NativeStarted::Entered(_) => receiver,
        })
    }

    /// `park`'s primitive methods, run once their activity has woken: the
    /// continuation, or `failure` in its place, then each method's work after
    /// it.
    pub(crate) fn resume_native_park(
        &mut self,
        park: NativePark,
        failure: Option<Failure>,
    ) -> Result<Option<ObjRef>, Failure> {
        let NativePark {
            resume,
            receiver,
            thens,
            ..
        } = park;
        let frame = self.roots.activity_mut().push_frame();
        let mut held = Vec::new();
        for (then, blame) in &thens {
            then.object_roots(&mut held);
            if let Some(blame) = blame {
                blame.object_roots(&mut held);
            }
        }
        for object in held {
            self.roots.activity_mut().push_temp(object);
        }
        let mut sent = match failure {
            Some(failure) => {
                self.abandon_native_call(receiver);
                Err(failure)
            }
            None => {
                self.activity.resuming = true;
                let sent = resume(self, receiver);
                self.activity.resuming = false;
                sent
            }
        };
        for (then, blame) in thens {
            if let Ok(Some(value)) = sent {
                self.roots.activity_mut().push_temp(value);
            }
            sent = self.apply_then(then, sent);
            if sent.is_err()
                && let Some(blame) = blame
            {
                let scope = self.scope_id(blame.scope);
                self.blame_native_method(&blame.name, &scope, blame.receiver, &blame.args);
            }
        }
        self.roots.activity_mut().pop_frame(frame);
        sent
    }

    /// A send's value once the Rexx body it entered has ended `ended`: the
    /// sender's level state back, and the work of every primitive method
    /// between the sender and that body.
    pub(crate) fn finish_send(
        &mut self,
        ended: Result<crate::run::Ended, Failure>,
    ) -> Result<Option<ObjRef>, Failure> {
        let depth = self.activity.call_tails.len();
        let mut sent = self.finish_call(ended).map(crate::run::Ended::value);
        if self
            .activity
            .native_tails
            .last()
            .is_some_and(|tail| tail.depth == depth)
        {
            sent = self.finish_native_tails(depth, sent);
        }
        if !self.uninit_ready.is_empty()
            && let Ok(value) = sent
        {
            self.run_uninits_at_return(value)?;
        }
        sent
    }

    /// [`Interp::finish_send`]'s primitive methods, innermost first.
    #[cold]
    #[inline(never)]
    fn finish_native_tails(
        &mut self,
        depth: usize,
        mut sent: Result<Option<ObjRef>, Failure>,
    ) -> Result<Option<ObjRef>, Failure> {
        let first = self
            .activity
            .native_tails
            .iter()
            .rposition(|tail| tail.depth != depth)
            .map_or(0, |below| below + 1);
        let tails: Vec<NativeTail> = self.activity.native_tails.drain(first..).collect();
        for tail in tails {
            sent = self.apply_then(tail.then, sent);
            if sent.is_err()
                && let Some(blame) = tail.blame
            {
                let scope = self.scope_id(blame.scope);
                self.blame_native_method(&blame.name, &scope, blame.receiver, &blame.args);
            }
        }
        sent
    }

    /// Parks `then` beside the [`crate::run::CallTail`] on top.
    fn push_native_tail(&mut self, then: Then, blame: Option<NativeBlame>) {
        let depth = self.activity.call_tails.len();
        self.activity
            .native_tails
            .push(NativeTail { depth, then, blame });
    }

    /// A [`NativeBegin`]'s outcome, with the activation it entered run on
    /// this Rust stack: the whole primitive method, for a caller that is not
    /// a send.
    pub(crate) fn complete_native(
        &mut self,
        started: NativeStarted,
    ) -> Result<Option<ObjRef>, Failure> {
        match started {
            NativeStarted::Ran(value) => Ok(value),
            NativeStarted::Entered(then) => {
                if self.activity.native_park.is_some() {
                    return Err(self.refuse_unrooted_park());
                }
                self.push_native_tail(then, None);
                let ended = self.run_activation();
                self.finish_send(ended)
            }
        }
    }

    /// What a primitive method answers once the activation it entered
    /// answered `sent`.
    pub(crate) fn apply_then(
        &mut self,
        then: Then,
        sent: Result<Option<ObjRef>, Failure>,
    ) -> Result<Option<ObjRef>, Failure> {
        match then {
            Then::Pass => sent,
            Then::Answer(object) => sent.map(|_| Some(object)),
            Then::Started(message) => object_protocol::record_started(self, message, sent),
            Then::Held(message) => object_protocol::record_held(self, message, sent),
            Then::Triggered(message) => {
                object_protocol::record_held(self, message, sent).map(|_| None)
            }
        }
    }

    /// A generated getter: the value the attribute's variable holds in the
    /// declaring scope's pool on the receiver ([`Interp::pool_value`]).
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
        Ok(Some(self.pool_value(owner, resolution.scope, &variable)))
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
        self.set_pool_value(owner, resolution.scope, &variable, value);
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
        let target = self.delegate_target(generated, resolution, receiver)?;
        self.send_to_delegate_target(target, name, args)
    }

    /// The value of a `DELEGATE` method's variable, rooted as a temp.
    fn delegate_target(
        &mut self,
        generated: crate::GeneratedMethod,
        resolution: Resolution,
        receiver: ObjRef,
    ) -> Result<ObjRef, Failure> {
        let variable = self.delegate_variable(generated)?;
        let owner = self.pool_owner(receiver)?;
        let target = self.pool_value(owner, resolution.scope, &variable);
        self.roots.activity_mut().push_temp(target);
        Ok(target)
    }

    /// A `DELEGATE` method's send to `target`.
    fn send_to_delegate_target(
        &mut self,
        target: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        let caller = self.caller();
        pinned!(
            self,
            crate::pinning::PinKind::Delegate,
            self.send_message(target, name, None, args, caller)
        )
    }

    /// The variable a `DELEGATE` method addresses.
    fn delegate_variable(&self, generated: crate::GeneratedMethod) -> Result<Box<[u8]>, Failure> {
        let program = &self.programs[generated.program.0];
        let Some(directive) = program.directives.get(generated.directive) else {
            return Err(Loud::missing_body().into());
        };
        let Some(symbol) = crate::delegate_variable(&directive.kind) else {
            return Err(Loud::missing_body().into());
        };
        Ok(program.symbols.name(symbol).as_bytes().into())
    }

    /// The variable a generated accessor addresses.
    fn accessor_variable(&self, generated: crate::GeneratedMethod) -> Result<Box<[u8]>, Failure> {
        let program = &self.programs[generated.program.0];
        // `get` rather than an index, and `None` rather than a panic, for the
        // reason `Interp::begin_method`'s own reads carry.
        let Some(directive) = program.directives.get(generated.directive) else {
            return Err(Loud::missing_body().into());
        };
        let Some(variable) = crate::accessor_variable(&directive.kind) else {
            return Err(Loud::missing_body().into());
        };
        Ok(variable.into())
    }

    /// Pushes one `::METHOD` body's activation and the [`CallTail`] its
    /// sender's level state goes into, up to the point the body would run.
    ///
    /// [`CallTail`]: crate::run::CallTail
    #[expect(
        clippy::too_many_arguments,
        reason = "the send's own parts, and the receiver's behaviour where the send has it"
    )]
    fn begin_method(
        &mut self,
        _cleared: Cleared,
        installed: crate::InstalledMethodBody,
        resolution: Resolution,
        behaviour: Option<Behaviour>,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<(), Failure> {
        let program = Rc::clone(&self.programs[installed.program.0]);
        // Both reads are `get`, not an index: an `InstalledMethodBody`
        // always names a real directive with a body, but this crate's rule for an internal
        // inconsistency is a loud refusal rather than a panic, and
        // `body_of`'s own `?` already follows it.
        let Some(directive) = program.directives.get(installed.directive) else {
            return Err(Loud::missing_body().into());
        };
        let Some(body) = body_of(&program, Some(installed.directive)) else {
            return Err(Loud::missing_body().into());
        };
        let guarded = self.sends_guarded(
            resolution.method,
            crate::guards::directive_guarded(&directive.kind),
        );
        // D19/I6, the same guard `Interp::invoke_call` takes and for the same
        // reason: a method that sends itself a message is an unbounded
        // recursion, and it must become a reportable condition rather than a
        // native abort.
        let probe = 0u8;
        if self.activation_depth() >= MAX_ACTIVATION_DEPTH || self.stack_exhausted(&raw const probe)
        {
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
        let (self_slot, super_slot) = (plan.self_slot, plan.super_slot);
        let reserve_at_once =
            guarded && !plan.traces_entry_early && !self.activities.guards.is_live();
        let callee_id = self.next_activation_id();
        let super_scope = match behaviour {
            Some(behaviour) => self.super_scope_in(behaviour, resolution.scope),
            None => self.super_scope_for(receiver, resolution),
        };
        // **The calling convention, entered before anything reads it.** The
        // receiver goes in here and is read back out below, so the identity
        // the activation carries, the `SELF` the body reads and the caller a
        // send inside the body resolves as all name one field rather than
        // each taking its own copy of this function's argument.
        let arguments = self.shared_arguments(args);
        let name = self.activity.invocation_name(name);
        let saved_context = std::mem::replace(
            &mut self.activity.call_context,
            crate::CallContext {
                name: Rc::clone(&name),
                arguments,
                receiver: Some(receiver),
            },
        );
        // **Shadows the parameter**, which is what makes the routing a
        // property of scope rather than of a comment: past this line the name
        // `receiver` is the convention's own field, so the bindings below
        // cannot read the argument even by accident.
        let receiver = self
            .activity
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
                name,
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
        // With one activity the lock is the activation's flag alone, taken
        // here unless a package-traced `>I>` has to come first.
        let reserve_at_once = reserve_at_once && !callee.trace_mode.labels;
        callee.flags = crate::activation::ActivationFlags::method(guarded, reserve_at_once);
        self.push_activation(callee);
        self.trace_package_invocation_entry();
        if guarded && !reserve_at_once {
            self.reserve_for_method(crate::guards::GuardKey {
                object: receiver,
                scope: resolution.scope,
            });
        }

        // `SELF` and `SUPER`, the two locals `RexxActivation::run` sets on a
        // method activation before its first instruction
        // (`RexxActivation.cpp:535`-`536`). Measured in a class method of
        // `::class K`: `say self` is `The K class` and `say super` is
        // `The Class class`.
        let self_slot = match self_slot {
            Some(slot) => slot,
            None => self.slot_of(b"SELF"),
        };
        self.set_variable(frame, self_slot, receiver);
        let super_slot = match super_slot {
            Some(slot) => slot,
            None => self.slot_of(b"SUPER"),
        };
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
        let saved_base = std::mem::replace(&mut self.activity.activation_indent, 0);
        let saved_offset = std::mem::take(&mut self.activity.indent_offset);
        let saved_line = std::mem::take(&mut self.activity.clause_line_override);
        self.activity.call_tails.push(crate::run::CallTail::method(
            saved_context,
            saved_clause_state,
            saved_base,
            saved_offset,
            saved_line,
        ));
        Ok(())
    }

    /// [`Interp::finish_call`]'s release of a method activation that has
    /// just been popped, with `saved_context` the sender's convention. A
    /// `REPLY`'s activation answers its value, and moves with its slot frame
    /// to the continuation its split filed, if the split ran.
    pub(crate) fn release_method_activation(
        &mut self,
        mut callee: Box<Activation>,
        saved_context: crate::CallContext,
    ) -> Option<Option<ObjRef>> {
        // Unconditionally, where `Interp::invoke_call` asks `owns_frame`
        // first: a method activation always owns its frame and nothing can
        // change that under it, because the one instruction that swaps a
        // frame in is `PROCEDURE` and `Entry::Method` is 17.1 for it.
        debug_assert!(callee.owns_frame, "a method activation owns its frame");
        let callee_context = std::mem::replace(&mut self.activity.call_context, saved_context);
        let Some(replied) = callee.replied.take() else {
            self.roots.activity_mut().pop_slots(callee.frame);
            if callee.exposed.capacity() > 0 {
                let mut exposed = std::mem::take(&mut callee.exposed);
                exposed.clear();
                self.activity.spare_exposed = exposed;
            }
            // **Back to the pool**, which until now only `Interp::invoke_call`
            // fed. The pool is drained by every push and was filled by the
            // `CALL` path alone, so a program of method sends missed it every
            // time: measured on `bench-programs/dispatch.rex`, 5,000,089
            // misses and no hits, each one a `Box::new` of a 416-byte
            // `Activation` and the free that follows.
            self.recycle_activation(callee);
            return None;
        };
        let Replied {
            value,
            continuation,
        } = *replied;
        match continuation {
            Some(continuation) => self.spawn_continuation(callee, callee_context, continuation),
            // The body ended before its next clause, so nothing is left to
            // move. The oracle still spawns the empty rest, which numbers the
            // replier (`Activity.cpp:1206`) and takes the oldest pooled
            // thread back to the pool's tail.
            None => {
                self.activity_number();
                self.rotate_pooled();
                self.activity.splits_owed -= 1;
                self.roots.activity_mut().pop_slots(callee.frame);
                self.recycle_activation(callee);
            }
        }
        Some(value)
    }

    /// Files `continuation`, the record a `REPLY`'s split left, with
    /// `activation` and its slot frame moved into it. The oracle's replier
    /// yields only to a thread already waiting (`ActivityManager.hpp:293`),
    /// which the continuation's is not: the sender yields at its next clause
    /// boundary where another activity is ready, and otherwise keeps running
    /// (rulings P34, P36).
    #[cold]
    #[inline(never)]
    fn spawn_continuation(
        &mut self,
        mut activation: Box<Activation>,
        context: crate::CallContext,
        mut continuation: Box<crate::scheduler::Idle>,
    ) {
        let mut anchor = Vec::new();
        activation.object_roots(&mut anchor);
        context.object_roots(&mut anchor);
        continuation.roots.park(anchor);
        activation.frame = self
            .roots
            .activity_mut()
            .move_frame(activation.frame, &mut continuation.roots);
        activation.reply = ReplyState::Issued;
        activation.trace_entry = if activation.trace_entry == TraceEntry::Done {
            TraceEntry::Allowed
        } else {
            TraceEntry::Spent
        };
        activation.first_instruction_pending = false;
        self.guards_go_live(Some(&activation));
        let transfer = self.transfer_on_reply(&mut activation);
        let resumed = &mut continuation.activity;
        if let crate::guards::Transfer::Again(wait) = transfer {
            resumed.guard_waits.push(wait);
        }
        resumed.trace_cache =
            crate::trace::TraceCache::of(activation.trace_mode, activation.flags.debug_pause());
        resumed.running = Some(activation);
        resumed.call_context = context;
        resumed.call_tails.push(crate::run::CallTail::method(
            crate::CallContext::default(),
            self.save_clause_state(),
            0,
            0,
            None,
        ));
        resumed.first = Some(crate::scheduler::First::Reply);
        resumed.root_then = Some(Then::Pass);
        let others_ready = self.any_ready();
        let spawned = self.spawn(continuation);
        if let crate::guards::Transfer::Moved(key) = transfer {
            self.activities.guards.transfer(key, spawned);
        }
        if others_ready {
            self.timer.requests().set(crate::timer::SLICE);
        }
        // The split's own visit served no other request, so the sender's next
        // clause does: it arms the timer for the continuation now ready.
        self.clause_countdown = 1;
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
        let outcome = pinned!(
            self,
            crate::pinning::PinKind::Uninit,
            self.send_message(object, UNINIT, None, &[], caller)
        );
        self.clear_failure_levels();
        match outcome {
            // A deadline is discarded here with the rest, and `execute`'s own
            // guard is what keeps that from being a silent answer: the flag
            // the check set outlives this match. Every later clause of this
            // run fails the same way, so the bounded sweep above finishes at
            // once rather than running the remaining finalizers.
            // No slice ends under the pin this send holds.
            Ok(_)
            | Err(Failure::Raised(_) | Failure::Exited(_) | Failure::Deadline | Failure::Slice) => {
                None
            }
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
        let parked = self.roots.activity_mut().park(batch.clone());
        for object in batch {
            loud.extend(self.run_one_uninit(object));
        }
        self.roots.activity_mut().release(parked);
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

    /// [`Interp::run_ending_uninits`] for an activation returning `value`,
    /// which stays rooted while they run.
    #[cold]
    #[inline(never)]
    pub(crate) fn run_uninits_at_return(&mut self, value: Option<ObjRef>) -> Result<(), Failure> {
        let parked = self.roots.activity_mut().park(value.into_iter().collect());
        let drained = self.run_ending_uninits();
        self.roots.activity_mut().release(parked);
        drained
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

    /// [`Interp::resolve_in`] then [`Interp::invoke`], with
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
        let started = self.begin_send(receiver, name, start_scope, args, caller)?;
        self.complete_send(started)
    }

    /// [`Interp::send_message`] up to the point a Rexx body it enters would
    /// run.
    pub(crate) fn begin_send(
        &mut self,
        receiver: ObjRef,
        name: &[u8],
        start_scope: Option<ObjRef>,
        args: &[Option<ObjRef>],
        caller: Caller,
    ) -> Result<Started<Option<ObjRef>>, Failure> {
        let behaviour = match self.receiver_behaviour(receiver) {
            Ok(behaviour) => behaviour,
            Err(kind) => return Err(Loud::unknown_receiver(kind).into()),
        };
        match self.resolve_in(receiver, behaviour, name, start_scope, caller) {
            Ok(resolution) => self.begin_invoke(resolution, Some(behaviour), receiver, name, args),
            Err(miss) => self
                .unknown_or_nomethod(receiver, name, args, miss)
                .map(Started::Ran),
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
        let frame = self.roots.activity_mut().push_frame();
        let arguments = self.alloc_charged(BehaviourId::ARRAY, Body::array(args.to_vec()));
        self.roots.activity_mut().push_temp(arguments);
        let missed = self.text(name);
        self.roots.activity_mut().push_temp(missed);
        let forwarded = pinned!(
            self,
            crate::pinning::PinKind::Unknown,
            self.invoke(
                resolution,
                receiver,
                UNKNOWN,
                &[Some(missed), Some(arguments)],
            )
        );
        self.roots.activity_mut().pop_frame(frame);
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
        self.roots.activity_mut().push_temp(receiver);

        let start_scope = match super_class {
            None => None,
            Some(super_class) => {
                // Evaluated for its own trace lines and its own failures
                // before either check below, which is the oracle's order.
                let scope = self.eval(code, super_class)?;
                self.roots.activity_mut().push_temp(scope);
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
            self.trace_message(
                self.activity.clause_state.current_value_indent,
                name,
                &rendered,
            );
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
                    self.trace_argument(self.activity.clause_state.current_value_indent, b"");
                    values.push(None);
                }
                Some(expr) => values.push(Some(self.eval_traced_argument(code, expr)?)),
            }
        }
        Ok(())
    }

    /// Records the traceback line a failing native method contributes, and
    /// closes the level so the sending clause records its own.
    pub(crate) fn blame_native_method(
        &mut self,
        name: &[u8],
        scope: &str,
        target: ObjRef,
        arguments: &[Option<ObjRef>],
    ) {
        let level = NativeLevel {
            method: true,
            name,
            target,
            arguments,
            activation: None,
        };
        self.blame_native_level(Raised::compiled_method_line(name, scope), None, level);
    }

    /// [`Interp::blame_native_method`] for a method an `EXTERNAL` directive
    /// bound, whose level carries the package that directive was written in.
    /// `failure` is what the method answered.
    /// `reraised` is whether the call's failure is the condition it raised
    /// while running.
    pub(crate) fn blame_external_method(
        &mut self,
        name: &[u8],
        scope: &str,
        method: MethodId,
        target: ObjRef,
        arguments: &[Option<ObjRef>],
        reraised: bool,
    ) {
        let package = self.external_package_path(method);
        let level = NativeLevel {
            method: true,
            name,
            target,
            arguments,
            activation: Some(NativeActivationLevel {
                package: self.external_package_program(method).map(Package::Program),
                reraised,
            }),
        };
        self.blame_native_level(Raised::compiled_method_line(name, scope), package, level);
    }

    /// The same for a routine of an internal package, whose traceback line
    /// names the routine alone and upcased -- measured, `filespec('D')` is
    /// reported for `Filespec` as `"FILESPEC"`.
    pub(crate) fn blame_internal_routine(
        &mut self,
        name: &[u8],
        arguments: &[Option<ObjRef>],
        failure: &Failure,
    ) {
        let level = NativeLevel {
            method: false,
            name,
            target: ObjRef::NIL,
            arguments,
            activation: Some(NativeActivationLevel {
                package: Some(Package::Rexx),
                reraised: failure.reraised_by_native_call(),
            }),
        };
        self.blame_native_level(Raised::compiled_routine_line(name), None, level);
    }

    /// The same for a routine of a loaded library, whose level carries the
    /// package its shared code reports, if any.
    pub(crate) fn blame_native_routine(
        &mut self,
        name: &[u8],
        program: Option<ProgramId>,
        arguments: &[Option<ObjRef>],
    ) {
        let package = program.map(|program| self.package_path(program).as_bytes().to_vec());
        let level = NativeLevel {
            method: false,
            name,
            target: ObjRef::NIL,
            arguments,
            activation: Some(NativeActivationLevel {
                package: program.map(Package::Program),
                reraised: std::mem::take(&mut self.activity.native_reraise),
            }),
        };
        self.blame_native_level(Raised::compiled_routine_line(name), package, level);
    }

    /// Records `text` as a native level's whole traceback line, first call
    /// wins, and closes the level so the sending clause records its own.
    fn blame_native_level(
        &mut self,
        text: Vec<u8>,
        package: Option<Vec<u8>>,
        level: NativeLevel<'_>,
    ) {
        if self.activity.failure_site.is_some() {
            return;
        }
        let frame = crate::dispatch::context::build_native_level_frame(
            self,
            level.method,
            level.name,
            level.target,
            level.arguments,
            &text,
        );
        self.activity.failure_frame = Some(frame);
        match level.activation {
            // `NativeActivation::checkConditions` (`execution/
            // NativeActivation.cpp:1787`) re-raises a SYNTAX condition in the
            // caller through `Activity::reraiseException`, whose `POSITION`
            // and program are the caller's.
            Some(NativeActivationLevel { reraised: true, .. }) => self.reraise_failure_levels(),
            // Anything else keeps the object built where it was raised, and
            // a condition raised here names this frame's package, which has
            // no line.
            Some(NativeActivationLevel {
                package: Some(package),
                reraised: false,
            }) => {
                self.activity.failure_origin.get_or_insert((package, None));
            }
            Some(_) | None => {}
        }
        self.activity.failure_site = Some(FailureSite::Rendered { text, package });
        self.seal_site_level();
    }
}

/// What the `StackFrame` of a failing native level is built from; see
/// [`context::build_native_level_frame`].
struct NativeLevel<'a> {
    method: bool,
    name: &'a [u8],
    target: ObjRef,
    arguments: &'a [Option<ObjRef>],
    /// `None` for an `InternalActivationFrame`, which neither has a package nor
    /// re-raises.
    activation: Option<NativeActivationLevel>,
}

/// A level the oracle runs as a `NativeActivation`.
struct NativeActivationLevel {
    /// What `NativeActivationFrame::getPackage` answers.
    package: Option<Package>,
    /// Whether the condition leaves it re-raised: a SYNTAX condition the
    /// boundary did not raise for itself.
    reraised: bool,
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

#[cfg(test)]
mod tests;
