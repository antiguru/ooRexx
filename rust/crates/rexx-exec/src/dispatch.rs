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
//! and is neither `Copy` nor `Clone`, and both invocable kinds take one by
//! value -- a [`NativeMethod`] by its signature and
//! [`Interp::enter_method_body`] by its parameter list -- so neither runs
//! without a value produced inside [`mod seam`](seam). The compiler refuses
//! the token's tuple-struct constructor written anywhere else,
//! `error[E0423]`.
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
//! A resolved [`rexx_classes::MethodId`] names either a [`NativeMethod`] or a
//! `::METHOD` directive's own Rexx body ([`Interp::method_bodies`]).
//! [`Interp::invoke`] picks between them **after** the seam has been passed,
//! and both entry points take the [`Cleared`] token by value, so neither can
//! run without one. [`Interp::enter_method_body`] is written here rather than
//! beside the other activation machinery in `run.rs` for exactly that reason:
//! `Cleared` is private to this module, so a function that takes one has to
//! live here.

use std::collections::HashMap;
use std::rc::Rc;

use rexx_classes::{ClassRegistry, MethodId};
use rexx_core::{BehaviourId, Body, Decoded, NativeObject, ObjRef};
use rexx_parse::{Access, Expr};

use crate::activation::{Activation, MethodIdentity, body_of};
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
    /// passes here, native or Rexx-bodied, and a manager installed in a later
    /// phase gets its hook in this function's body.
    ///
    /// The oracle asks its manager only for a *protected* method:
    /// `RexxObject::messageSend` routes one through
    /// `processProtectedMethod` (`classes/ObjectClass.cpp:886`-`:889`), and
    /// every other method reaches `method->run` with nothing asked
    /// (`:896`-`:899`). So the question is asked here, which is the one place
    /// both invocable kinds pass, and [`Interp::check_protected_method`] is
    /// the manager's own half of it.
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
/// hidden behind one closure: only a native method has a declared argument
/// count for the send to check, and only a native method contributes a
/// `Compiled method` traceback line.
enum Invocable {
    Native(NativeEntry),
    Rexx(crate::InstalledMethodBody),
}

/// What [`Interp::invoke`] needs about one primitive method beyond its code.
#[derive(Copy, Clone)]
struct NativeEntry {
    arity: Arity,
    run: NativeMethod,
}

/// How many arguments a primitive method's own entry admits, which is the
/// second half of every `AddMethod` row in `memory/Setup.cpp`.
#[derive(Copy, Clone)]
enum Arity {
    /// A numeric count. `CPPCode::run` refuses more than this with 93.902
    /// before the body is entered (`execution/CPPCode.cpp:151`-`:154`) and
    /// pads a shorter list out with nulls -- measured, `'abc'~length(1)` is
    /// `0 expected` and `'abc'~hasMethod('a','b')` is `1 expected`.
    Fixed(usize),
    /// `A_COUNT` (`execution/CPPCode.hpp:48`), which hands the body the whole
    /// argument list and count and refuses nothing here
    /// (`execution/CPPCode.cpp:144`-`:148`). The body's own check is what
    /// reports, and its error is not always 93.902 -- measured,
    /// `(1,2)~at(1,2)` is 93.926 `Too many subscripts for array; 1 expected.`
    /// where `.environment~at(1,2)`, whose row is a count, is 93.902.
    Counted,
}

/// The primitive methods this phase implements, as (class id, method name,
/// declared parameter count, implementation).
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
    // two agree on every shape `array_index.rex` and
    // `array_index_refusals.rex` ask.
    ("Array", "[]", Arity::Counted, native_array_at),
    ("Array", "AT", Arity::Counted, native_array_at),
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
    ("Class", "BASECLASS", Arity::Fixed(0), native_base_class),
    ("Class", "ID", Arity::Fixed(0), native_id),
    (
        "Class",
        "ISSUBCLASSOF",
        Arity::Fixed(1),
        native_is_subclass_of,
    ),
    ("Class", "METACLASS", Arity::Fixed(0), native_metaclass),
    ("Class", "METHOD", Arity::Fixed(1), native_method),
    ("Class", "PACKAGE", Arity::Fixed(0), native_package),
    ("Class", "SUPERCLASS", Arity::Fixed(0), native_superclass),
    (
        "Class",
        "SUPERCLASSES",
        Arity::Fixed(0),
        native_superclasses,
    ),
    // `HashCollection::getRexx` under both of its names and
    // `HashCollection::putRexx`, donated to `Directory` by
    // `InheritInstanceMethods(StringTable)` (`memory/Setup.cpp:933`) out of
    // `IdentityTable`'s own rows (`:825`, `:828`, `:831`). The donation puts
    // them in `Directory`'s **own** dictionary, which is what the scope in the
    // traceback says: measured, `.environment~at()` reports `Compiled method
    // "AT" with scope "Directory".`, not `IdentityTable`.
    ("Directory", "[]", Arity::Fixed(1), native_directory_at),
    ("Directory", "AT", Arity::Fixed(1), native_directory_at),
    ("Directory", "PUT", Arity::Fixed(2), native_directory_put),
    ("Object", "CLASS", Arity::Fixed(0), native_class),
    ("Object", "HASMETHOD", Arity::Fixed(1), native_has_method),
    (
        "Object",
        "IDENTITYHASH",
        Arity::Fixed(0),
        native_identity_hash,
    ),
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
    ("Object", "STRING", Arity::Fixed(0), native_string),
    ("Package", "NAME", Arity::Fixed(0), native_package_name),
    ("String", "LENGTH", Arity::Fixed(0), native_length),
    (
        "String",
        "MAKESTRING",
        Arity::Fixed(0),
        native_string_make_string,
    ),
    ("String", "REVERSE", Arity::Fixed(0), native_reverse),
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
    directory: ObjRef,
}

impl ObjectModel {
    /// `Setup.cpp`'s native class set, plus the lookup from each implemented
    /// method's minted identity to its code.
    ///
    /// **Built on first use and not in `Interp::new`.** Measured at 5.5 ms
    /// per build, which every program that never sends a message and
    /// declares no class would otherwise pay.
    fn bootstrap() -> ObjectModel {
        let classes = rexx_classes::native_classes();
        let mut natives = HashMap::new();
        for (class_id, method_name, arity, run) in NATIVE_METHODS {
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
        let string = classes.lookup("String").expect("String is a native class");
        let object = classes.lookup("Object").expect("Object is a native class");
        let metaclass = classes.lookup("Class").expect("Class is a native class");
        let array = classes.lookup("Array").expect("Array is a native class");
        let package = classes
            .lookup("Package")
            .expect("Package is a native class");
        let method = classes.lookup("Method").expect("Method is a native class");
        let directory = classes
            .lookup("Directory")
            .expect("Directory is a native class");
        ObjectModel {
            classes,
            natives,
            string,
            object,
            metaclass,
            array,
            package,
            method,
            directory,
        }
    }
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
    /// A `Body::Native` whose class is `.Directory` -- `.environment` and
    /// `.local`. Measured, `.environment~class~id` is `Directory`.
    ///
    /// **`.methods`, `.routines`, `.resources` and `.context` are not this**,
    /// even though a `StringTable` answers the names this module's `Directory`
    /// rows answer, out of the same donated `IdentityTable` rows: this crate
    /// populates none of those tables (`environment.rs`'s
    /// `package_string_table`), so answering `~at` on one would answer `.nil`
    /// for an index the oracle has an entry for. They keep the loud arm.
    Directory,
    /// The receiver **is** a class object, so its messages resolve against
    /// that class's own class behaviour rather than against any class's
    /// instance behaviour. Measured, `::class K` plus `::method m class`:
    /// `.K~m` runs the body, `.K~hasMethod('M')` is `1` and
    /// `.K~hasMethod('LENGTH')` is `0`.
    Class(ObjRef),
}

/// The behaviour a receiver's messages resolve against.
///
/// An enum rather than a bare `ObjRef`, because a class object's messages
/// are answered by its **class** behaviour while every other receiver's are
/// answered by its class's **instance** behaviour, and those dictionaries
/// hold different names for the same class.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Behaviour {
    Instance(ObjRef),
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
        self.object_model.get_or_insert_with(ObjectModel::bootstrap)
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
            Behaviour::Instance(class) => class,
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

    /// Which native class a value answers to, or the value's own shape when
    /// this phase builds no class for it.
    ///
    /// A stem answers `Stem` on the oracle, and
    /// `rexx_classes::deferred_classes` does not build that class (its
    /// `Setup.cpp` block hides the comparison methods, and `MethodDict`
    /// models no removal), so a stem receiver resolves nothing and fails
    /// loudly rather than answering from the wrong class. Every other heap
    /// shape that has no class here gets a refusal naming itself rather than
    /// a shared one, so whichever task makes one reachable as a receiver gets
    /// a message that says which.
    ///
    /// A **class object** is the one heap-tagged handle that answers, and it
    /// answers as itself rather than as an instance of anything: its
    /// behaviour is that class's class behaviour, which is where `::METHOD
    /// ... CLASS` installs.
    ///
    /// An **array** answers, because `~superClasses` puts one in a program's
    /// hands. A name `.Array`'s behaviour here does not hold is the oracle's
    /// 97.1, which is the same answer a `String` receiver already gets for a
    /// name the prologue donates and this crate has not: `CoreClasses.orx:93`
    /// and `:97` are the same `~inherit` and the gap belongs to whichever task
    /// runs that file, not to one value kind.
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
                // A handle whose slot is gone. Not reachable from a running
                // program -- a receiver is rooted by the term that evaluated
                // it -- and answered rather than panicked for the reason
                // every error path here is: an implementation gap must not
                // become a crash.
                None => Err("a value whose object is no longer live"),
                Some(object) => match &object.body {
                    Body::Text { .. } | Body::Num { .. } => Ok(Primitive::String),
                    Body::Stem { .. } => Err("a stem"),
                    Body::Array(_) => Ok(Primitive::Array),
                    Body::Instance(_) => Err("an instance of a user class"),
                    Body::WeakRef(_) => Err("a weak reference"),
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
                    Body::Native(native)
                        if self.object_model.as_ref().map(|model| model.directory)
                            == Some(native.class()) =>
                    {
                        Ok(Primitive::Directory)
                    }
                    // `.methods`, `.routines`, `.resources` and `.context`.
                    // Their classes are in the registry, so there is a
                    // behaviour to resolve against -- what is missing is a
                    // `NATIVE_METHODS` row for anything a `StringTable` or a
                    // `RexxContext` answers, and answering 97.1 for a name the
                    // oracle implements is the wrong failure. Loud until a
                    // task implements those methods.
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
        Ok(match kind {
            // Both arms answer one behaviour, which is what makes
            // `Primitive::SmallInt` a distinction without a divergence: the
            // oracle gives `RexxInteger` the id `String`
            // (`classes/IntegerClass.cpp:2066`), so a small integer's messages
            // resolve against `String`'s instance behaviour exactly as a
            // literal's do. This fold is one of the two sites that would
            // change if that ever stopped being true.
            Primitive::String | Primitive::SmallInt => Behaviour::Instance(model.string),
            Primitive::Object => Behaviour::Instance(model.object),
            Primitive::Array => Behaviour::Instance(model.array),
            Primitive::Package => Behaviour::Instance(model.package),
            Primitive::Directory => Behaviour::Instance(model.directory),
            Primitive::Class(class) => Behaviour::ClassSide(class),
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
        let behaviour = self.receiver_behaviour(receiver).ok()?;
        // Borrowed rather than owned wherever the name is UTF-8, which every
        // name a program can write is: `from_utf8_lossy` allocates only for
        // the bytes it has to replace.
        let name = String::from_utf8_lossy(name);
        let (scope, method) = match (behaviour, start_scope) {
            (Behaviour::Instance(class), None) => {
                self.classes().lookup_instance_method(class, &name)?
            }
            (Behaviour::Instance(class), Some(start)) => self
                .classes()
                .lookup_instance_method_from_scope(class, &name, start)?,
            (Behaviour::ClassSide(class), None) => {
                self.classes().lookup_class_method(class, &name)?
            }
            (Behaviour::ClassSide(class), Some(start)) => self
                .classes()
                .lookup_class_method_from_scope(class, &name, start)?,
        };
        Some(Resolution { scope, method })
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
    /// **The refusing arm needs a second package and so is not reachable in
    /// this phase**: a caller in another package needs `::REQUIRES`, which is
    /// refused here. The same-package arm is what a program can run, and an
    /// in-crate test is the whole instrument for the other.
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
            Behaviour::Instance(class) => Some(class),
            Behaviour::ClassSide(class) => Some(self.classes().class_of(class)),
        }
    }

    /// `isOfClassType(Class, value)`: whether the value is a class object.
    ///
    /// Asked through `receiver_kind` rather than off the handle's own tag,
    /// because a heap-tagged handle with a class id is the one shape that
    /// names no arena slot and that distinction is that function's.
    fn is_class_object(&self, value: ObjRef) -> bool {
        matches!(self.receiver_kind(value), Ok(Primitive::Class(_)))
    }

    /// The scope a method found at `resolution` sees as `SUPER`, or `None`
    /// when it is the topmost scope in the receiver's behaviour --
    /// `RexxObject::superScope(scope)`, the second of the two locals
    /// `RexxActivation::run` sets on a method activation
    /// (`RexxActivation.cpp:535`-`536`).
    fn super_scope_for(&mut self, receiver: ObjRef, resolution: Resolution) -> Option<ObjRef> {
        match self.receiver_behaviour(receiver).ok()? {
            Behaviour::Instance(class) => {
                self.classes().instance_super_scope(class, resolution.scope)
            }
            Behaviour::ClassSide(class) => {
                self.classes().class_super_scope(class, resolution.scope)
            }
        }
    }

    /// What a resolved [`MethodId`] runs, or the refusal for one this phase
    /// implements neither way.
    ///
    /// Asked **before** the seam rather than after, so the seam stays a
    /// single call site with both invocable kinds behind it.
    fn invocable(&mut self, resolution: Resolution, name: &[u8]) -> Result<Invocable, Failure> {
        if let Some(entry) = self.object_model().natives.get(&resolution.method).copied() {
            return Ok(Invocable::Native(entry));
        }
        if let Some(installed) = self.method_bodies.get(&resolution.method).copied() {
            return Ok(Invocable::Rexx(installed));
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
    /// **`None` is a send that produced no value**, and either kind can do it.
    /// Measured, `::method m class` ending in a bare `return`: as a whole
    /// clause it drops `RESULT` at rc 0, and in an expression it is 91.999 at
    /// rc 165. `.environment~put('v','q')` answers the identical pair from a
    /// [`NativeMethod`].
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
            // **No `blame_native_method`**, measured: an untrapped `1/0`
            // inside a `::METHOD` body reports the method's own failing
            // clause and then the sending clause, with no `Compiled method`
            // line between them. [`Interp::enter_method_body`] seals its own
            // level for that, exactly as `Interp::invoke_call` does.
            Invocable::Rexx(installed) => {
                self.enter_method_body(cleared, installed, resolution, receiver, name, args)
            }
        }
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
                arguments: args
                    .iter()
                    .map(|arg| arg.map(crate::Argument::Value))
                    .collect(),
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
        self.push_activation(Activation::method(
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
        ));

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
        self.roots.pop_slots(callee.frame);
        self.activation_indent = saved_base;
        self.indent_offset = saved_offset;
        self.clause_line_override = saved_line;
        self.restore_clause_state(saved_clause_state);
        self.call_context = saved_context;

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
        let arguments = self.alloc_with(BehaviourId::ARRAY, Body::Array(args.to_vec()));
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

        if let Some(super_class) = super_class {
            // Evaluated for its own trace lines and its own failures before
            // the refusal below, which is the oracle's order.
            let scope = self.eval(code, super_class)?;
            self.roots.push_temp(scope);
            // `RexxExpressionMessage::evaluate`'s
            // `_super->isInstanceOf(TheClassClass)`. A value that is **not**
            // a class object gets the oracle's own 88.914, and one that is
            // gets a refusal: the override needs the receiver's own scope
            // chain checked against `scope` (93.957, `Target object "abc" is
            // not a subclass of the message override scope (The Array
            // class).`) before `Interp::resolve`'s start-scope argument may
            // be used, and this phase implements neither that check nor the
            // `SUPER` sends that would want it.
            return Err(if scope.class_id().is_some() {
                Loud::scope_override(&String::from_utf8_lossy(self.class_default_name(scope)))
                    .into()
            } else {
                Raised::scope_override_not_a_class().into()
            });
        }

        let mut values = self.take_value_buffer();
        let evaluated = self.evaluate_message_arguments(code, args, assigned, &mut values);
        let caller = self.caller();
        let result =
            evaluated.and_then(|()| self.send_message(receiver, name, None, &values, caller));
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
            values.push(Some(self.eval_traced_argument(code, expr)?.value()));
        }
        for arg in args {
            match arg {
                // An omitted position traces an empty value line, not no
                // line -- the same rule a call's own argument list follows.
                None => {
                    self.trace_argument(self.clause_state.current_value_indent, b"");
                    values.push(None);
                }
                Some(expr) => values.push(Some(self.eval_traced_argument(code, expr)?.value())),
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
        if self.trap_for(b"NOSTRING").is_some_and(|trap| !trap.call) {
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
        // `sendMessage(STRING)`, which for every value this phase builds is
        // `stringValue()` -- `Interp::string_value_text`'s own doc says which
        // question that is.
        let readable = self.string_value_text(value);
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
                    Body::Array(_) => None,
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
        Behaviour::Instance(class) => classes.has_method(class, &name),
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
/// Measured, `.array~id` is `Array` and `::class Foo` makes `.Foo~id` `Foo`.
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
        interp.alloc_with(BehaviourId::ARRAY, Body::Array(items)),
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
        Primitive::Directory => model.directory,
        Primitive::Class(class) => model.classes.class_of(class),
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
/// message: identity in this crate is handle equality, and what its answers
/// *mean* is 5c's. The oracle's own answer is derived from the object's
/// address, so no differential row can compare the two -- the corpus cannot
/// witness this method and `dispatch.rs`'s own tests are the instrument.
fn native_identity_hash(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let bits = receiver.bits().to_string().into_bytes();
    Ok(Some(interp.text_built(bits)))
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
    let found = interp
        .classes()
        .has_own_instance_method(class, &String::from_utf8_lossy(&name));
    if !found {
        let target = interp.class_default_name(class).to_vec();
        return Err(Raised::no_method(&target, &name).into());
    }
    let method_class = interp.object_model().method;
    let rendered = crate::environment::default_object_name(interp.class_id_text(method_class));
    Ok(Some(interp.alloc_with(
        BehaviourId::OBJECT,
        Body::Native(Box::new(NativeObject::new(
            method_class,
            rendered.as_bytes(),
        ))),
    )))
}

/// An array receiver's own slots, borrowed, or the refusal for a receiver that
/// is not one.
///
/// The refusal is unreachable -- every row that reaches one of these callers
/// is in `.Array`'s own dictionary, and the only receiver whose behaviour that
/// dictionary reaches is a `Body::Array`. Loud rather than a panic, this
/// crate's rule for an internal inconsistency.
fn array_slots(interp: &Interp, receiver: ObjRef) -> Result<&[Option<ObjRef>], Failure> {
    interp
        .array_slots(receiver)
        .ok_or_else(|| Loud::receiver_class("a value that is not an array").into())
}

/// [`array_slots`] as an owned copy, for a caller that renders the slots and
/// so needs `interp` back.
fn array_slots_owned(interp: &Interp, receiver: ObjRef) -> Result<Vec<Option<ObjRef>>, Failure> {
    Ok(array_slots(interp, receiver)?.to_vec())
}

/// The one subscript `.Array`'s `[]`/`AT` were given, 1-based, or the refusal
/// for a subscript list that does not name one.
///
/// `ArrayClass::validateIndex` (`classes/ArrayClass.cpp:1211`) then
/// `validateSingleDimensionIndex` (`:1258`), under `IndexAccess`, which is
/// `RaiseBoundsTooMany` alone (`classes/ArrayClass.hpp:62`). So a subscript
/// past the end of the array is **not** an error -- measured,
/// `(1,2)~at(100000000000000001)` answers `The NIL object` even though that is
/// past `MaxFixedArraySize` -- and only the count and the conversion raise.
///
/// **A lone array argument is the subscript list**, spread by taking its item
/// count alongside its slot array (`:1219`-`:1226`). Measured, that is item
/// count and not size: `(1,2)~at((1,))` answers `1` where `(1,2)~at((1,2))` is
/// 93.926 and `(1,2)~at((,))` is 93.901.
fn array_index(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<usize, Failure> {
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
    match subscripts {
        [] => Err(Raised::not_enough_method_arguments(1).into()),
        [Some(only)] => positive_index(interp, *only),
        // Only the spread above can produce this: an argument list of its own
        // drops a trailing omission, so `~at(,)` arrives as no argument at all
        // and is 93.901 above.
        [None] => Err(Loud::array_index_hole().into()),
        _ => Err(Raised::too_many_subscripts(1).into()),
    }
}

/// One subscript as `RexxInternalObject::requiredPositive`
/// (`classes/ObjectClass.cpp:1564`) reads it: a whole number of at least 1,
/// converted under `Numerics::ARGUMENT_DIGITS` rather than under the
/// activation's own `NUMERIC DIGITS`.
///
/// The fixed precision is measured rather than read off the default argument:
/// `numeric digits 3; say (1,2)~at(1000000)` answers `The NIL object` where a
/// conversion at 3 digits would have rounded the subscript.
fn positive_index(interp: &mut Interp, value: ObjRef) -> Result<usize, Failure> {
    // A tagged integer already is the answer when it is narrow enough that
    // the rounding rule would change nothing, the same shortcut
    // `builtin::whole_number` takes against the same width.
    if let Decoded::SmallInt(small) = value.decode()
        && let Some(whole) = rexx_num::whole_i64(small, rexx_num::ARGUMENT_DIGITS)
        && let Ok(index) = usize::try_from(whole)
        && index > 0
    {
        return Ok(index);
    }
    if let Ok(number) = interp.to_number(value)
        && let Some(whole) = number.whole_value(rexx_num::ARGUMENT_DIGITS)
        && let Ok(index) = usize::try_from(whole)
        && index > 0
    {
        return Ok(index);
    }
    let found = interp.string_value_text(value);
    Err(Raised::method_argument_not_positive(1, &found).into())
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
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // The subscript first, so the slots are borrowed rather than copied: the
    // conversion needs `&mut interp` and the read does not, and reading one
    // slot must not cost a copy of the whole array.
    let index = array_index(interp, args)?;
    Ok(Some(match array_slots(interp, receiver)?.get(index - 1) {
        Some(Some(item)) => *item,
        Some(None) | None => ObjRef::NIL,
    }))
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

/// The index argument a directory method was given, as the bytes it is stored
/// and looked up under.
///
/// `stringArgument(index, "index")` (`runtime/MethodArguments.hpp:161`), which
/// raises 88.901 for an omitted argument and 88.909 for a value with no string
/// value. Measured at rc 168: `.environment~at()` reports `Missing argument;
/// argument index is required.` and `.environment~at(.nil)` reports `Argument
/// index must have a string value.`
///
/// **The bytes are not upcased.** A directory index is stored and matched
/// verbatim -- measured, `d~put('v','kk')` leaves `d['kk']` `v` and `d['KK']`
/// `The NIL object`, and `.environment['array']` is `The NIL object` where
/// `.environment['ARRAY']` is `The Array class`. The entries `Setup.cpp`
/// registers are uppercase because `completeSystemClass` upcases the *name it
/// registers*, not because a lookup folds case.
fn directory_index(
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

/// `Directory~at(index)` and `Directory~[index]`: the entry stored under
/// `index`, or `.nil` -- `HashCollection::getRexx`, donated to `.Directory`
/// by `InheritInstanceMethods(StringTable)`.
///
/// Measured, `.environment['ARRAY']` is `The Array class` and
/// `.environment['x']` is `The NIL object`.
fn native_directory_at(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let index = directory_index(interp, args, 1)?;
    Ok(Some(interp.directory_entry_read(receiver, &index)?))
}

/// `Directory~put(item, index)`: stores `item` under `index`, replacing
/// whatever was there -- `HashCollection::putRexx`.
///
/// **The item is argument one and the index argument two**, which is the order
/// `CoreClasses.orx:66` writes (`.environment~put(class, name)`). Measured at
/// rc 168, the two refusals in the order the C++ checks them:
/// `.environment~put()` reports `Missing argument; argument item is required.`
/// and `.environment~put('a')` reports `... argument index is required.`
///
/// **Answers no value**, measured: `.environment~put('v','q')` is rc 0 as a
/// whole clause and 91.999 at rc 165 under `say`.
fn native_directory_put(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(item)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("item").into());
    };
    let index = directory_index(interp, args, 2)?;
    interp.directory_entry_write(receiver, &index, item)?;
    Ok(None)
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
        Primitive::Object
        | Primitive::Array
        | Primitive::Class(_)
        | Primitive::Package
        | Primitive::Directory => interp.string_value_text(receiver),
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
        Primitive::Package | Primitive::Directory => {
            let Some(object) = interp.heap.get_mut(receiver) else {
                return Err(Loud::receiver_class("a value whose object is no longer live").into());
            };
            match &mut object.body {
                Body::Native(native) => native.set_rendered(&name),
                other => {
                    unreachable!("a package or directory receiver is Body::Native, got {other:?}")
                }
            }
        }
        Primitive::String | Primitive::SmallInt | Primitive::Object | Primitive::Array => {
            return Err(Loud::native_method(b"OBJECTNAME=", "Object").into());
        }
    }
    Ok(None)
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

    /// **The start-scope argument, which no program can reach yet.**
    ///
    /// `target~name:scope` needs a class object as a value and this phase
    /// produces none, so every scope override a program can write is refused
    /// at 88.914 before `resolve` is called. This is the test that exercises
    /// the argument itself.
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
    /// raised for both would let a program expecting the oracle's working
    /// `~upper` pass against a gap.
    #[test]
    fn an_unimplemented_method_is_loud_where_an_unknown_one_is_a_condition() {
        let mut interp = Interp::new();
        let receiver = interp.text(b"abc");
        assert!(matches!(
            interp.send_message(receiver, b"UPPER", None, &[], no_caller()),
            Err(Failure::Loud(_))
        ));
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
        assert_eq!(
            interp
                .receiver_behaviour(integer)
                .expect("a small integer resolves"),
            Behaviour::Instance(string),
            "the arm sends a small integer's messages somewhere other than String"
        );
        assert_eq!(
            interp
                .receiver_behaviour(text)
                .expect("a text handle resolves"),
            Behaviour::Instance(string)
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
        // (source, the phrase the refusal must name)
        let refused: &[(&str, &str)] = &[
            // oracle 93.965 at rc 163, `Method M is ABSTRACT and cannot be
            // directly invoked.`
            (
                "say .K~m\n::class K\n::method m class abstract\n",
                "a ::METHOD with no body of its own",
            ),
            // oracle 97.1 at rc 159 naming `"P"`: the message is forwarded to
            // the delegate property's value.
            (
                "say .K~m\n::class K\n::method m class delegate p\n",
                "a ::METHOD with no body of its own",
            ),
            // oracle rc 0, printing `A`: the generated getter reads an
            // uninitialised class-scope instance variable, which this phase
            // does not build.
            (
                "say .K~a\n::class K\n::attribute a class\n",
                "a generated ::ATTRIBUTE accessor",
            ),
            // The same accessor behind a `PRIVATE` attribute, sent from a
            // caller the access check ALLOWS, which is the arm that reaches
            // the gap rather than the refusal. Oracle rc 0, printing `A`.
            // The pair matters because the access scope and the missing
            // instance variable are two refusals over one send: an access
            // check that refused this caller would report 97.2 instead, and
            // the corpus row for the callers it does refuse
            // (`corpus/lang/method_access_private_attribute.rex`) is what
            // catches that from the other side.
            (
                "say .K~poke\n::class K\n::method poke class\n  return self~a\n\
                 ::attribute a class private\n",
                "a generated ::ATTRIBUTE accessor",
            ),
            // oracle rc 0, printing `4`: `USE LOCAL` binds its list against
            // the method's own scope pool.
            (
                "say .K~m\n::class K\n::method m class\n  use local zz\n  zz = 4\n  return zz\n",
                "USE LOCAL in a ::METHOD body",
            ),
        ];
        for (source, phrase) in refused {
            let (code, stdout, stderr) = both_engines(source);
            assert_eq!(
                (code, stdout.as_str(), stderr.as_str()),
                (
                    crate::NOT_IMPLEMENTED_EXIT,
                    "",
                    format!("rexx-exec: {phrase} is not implemented (Phase 5)\n").as_str()
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

    /// **`PACKAGE`'s refusing arm, which no program in this phase can
    /// reach.**
    ///
    /// A caller in a second package needs `::REQUIRES`, so the check is
    /// called directly with the callers the oracle refuses: one with no
    /// activation at all, and one whose package is not the method's. The
    /// allowing arm has a corpus program
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

    /// A `target~name:scope` override is **loud when the scope really is a
    /// class object** and the oracle's own 88.914 when it is not.
    ///
    /// The pair is what keeps the refusal from swallowing the condition:
    /// before Task 6 made `.NAME` resolve, no value was a class object and
    /// 88.914 was the only answer this term could give -- so
    /// `'abc'~length:.String`, which the oracle answers `3` at rc 0, was a
    /// Rexx condition a program could trap where the oracle succeeded.
    #[test]
    fn a_scope_override_is_loud_on_a_class_object_and_88_914_on_anything_else() {
        assert_eq!(
            both_engines("say 'abc'~length:.String\n"),
            (
                crate::NOT_IMPLEMENTED_EXIT,
                String::new(),
                "rexx-exec: a message scope override on \"The String class\" is not \
                 implemented (Phase 5)\n"
                    .to_string()
            )
        );
        let (code, stdout, stderr) = both_engines("say 'abc'~length:super\n");
        assert_eq!((code, stdout.as_str()), (168, ""));
        assert!(
            stderr.contains("Error 88.914:"),
            "a non-class scope must keep the oracle's own condition, got {stderr:?}"
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
    /// **The one shape of `~at` that has no oracle behaviour to match**, and
    /// the instrument [`Loud::array_index_hole`]'s own doc names.
    ///
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

    /// A name a directory's behaviour does not answer is **not** 97.1: the
    /// forward reaches `Directory`'s own `UNKNOWN`, whose body reads the name
    /// as an entry and which this phase does not implement -- so the refusal
    /// names that method rather than the search step.
    ///
    /// **The only instrument for these bytes**, and it has to be an in-crate
    /// one: a refusal the oracle does not share is not expressible as a
    /// corpus row, and no corpus program sends a missing name to a directory.
    /// Measured, `.environment~nosuch` is `The NIL object` at rc 0 on the
    /// oracle, so answering 97.1 here would be a wrong answer a program could
    /// trap -- which is what the `String` row below is paired against: that
    /// receiver's behaviour answers no `UNKNOWN`, so the same missing name
    /// really is the condition there.
    #[test]
    fn a_directory_forwards_a_missing_name_to_an_unknown_this_phase_lacks() {
        for source in ["say .environment~nosuch\n", "say .local~nosuch\n"] {
            let (code, stdout, stderr) = both_engines(source);
            assert_eq!((code, stdout.as_str()), (120, ""), "{source:?}");
            assert_eq!(
                stderr,
                "rexx-exec: method \"UNKNOWN\" of class \"Directory\" is not implemented \
                 (Phase 5)\n",
                "{source:?}"
            );
        }
        let (code, stdout, stderr) = both_engines("say 'abc'~nosuch\n");
        assert_eq!((code, stdout.as_str()), (159, ""));
        assert!(stderr.contains("Error 97.1:"), "{stderr:?}");
    }

    /// An entry the oracle's own directory holds and this crate does not build
    /// is a refusal, not `.nil` -- and the refusal is per directory, because
    /// the oracle answers `.nil` for a `.local` name asked of `.environment`.
    ///
    /// The three answering rows are what stops a build that refuses every miss
    /// from passing, and the `~put` row is what stops one that refuses by name
    /// alone: an entry a program stored answers even under a name the unbuilt
    /// table holds.
    #[test]
    fn a_directory_entry_the_oracle_has_and_this_crate_does_not_is_loud() {
        for (source, owner) in [
            ("say .environment['ALARM']\n", "Phase 5"),
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
            // `MAKEARRAY` is in these behaviours' dictionaries and this crate
            // has no code for it. Answering `.nil` here would contradict the
            // oracle, which converts: measured, oracle rc 0,
            // `.environment~request("ARRAY")` is an array and
            // `'abc'~request("ARRAY")` is an Array of one line.
            (
                "say .environment~request('ARRAY')\n",
                "rexx-exec: method \"MAKEARRAY\" of class \"Directory\" is not implemented \
                 (Phase 5)\n",
            ),
            (
                "say 'abc'~request('ARRAY')\n",
                "rexx-exec: method \"MAKEARRAY\" of class \"String\" is not implemented \
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
}
