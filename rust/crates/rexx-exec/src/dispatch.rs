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
//! `exec_call`'s relation to the call pair exactly.
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
use rexx_parse::Expr;

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
    use super::{Failure, Interp, ObjRef};

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
    /// The oracle asks its manager only for a *protected* method
    /// (`RexxObject::messageSend`'s `isSpecial`/`isProtected` branch), and
    /// this phase models no method visibility at all -- `Setup.cpp`'s
    /// `AddProtectedMethod`/`AddPrivateMethod` distinctions are not carried
    /// into `rexx-classes`. So the seam is passed unconditionally and
    /// refuses nothing; what a later phase adds here is the visibility test
    /// and the manager call, not a second seam.
    pub(super) fn clear(
        interp: &mut Interp,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<Cleared, Failure> {
        let _ = (interp, receiver, name, args);
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
    ("Package", "NAME", Arity::Fixed(0), native_package_name),
    ("String", "LENGTH", Arity::Fixed(0), native_length),
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
    /// Read by the access-scope checks, which are Task 13's: `PRIVATE`
    /// compares this against the receiver of the send. Kept out of
    /// `dead_code` by the `allow` below rather than by an assertion that
    /// cannot fail -- the accessor has no caller in the non-test build, and
    /// pretending otherwise with a `debug_assert` no in-tree path can
    /// falsify would put a check where a reader expects one and find nothing.
    /// `cargo clippy --all-targets` compiles this crate once with `cfg(test)`
    /// off, which is the compilation the lint fires in; an `expect` would be
    /// unfulfilled in the library-as-test one, which is a warning of its own
    /// (`rexx-parse/src/lib.rs`'s own note on the same choice).
    #[allow(dead_code, reason = "read by Task 13's PRIVATE check")]
    pub(crate) fn receiver(self) -> Option<ObjRef> {
        self.receiver
    }

    /// The package the send is written in -- `PACKAGE`'s input, Task 13's to
    /// read, and alive for the reason [`Caller::receiver`] gives.
    #[allow(dead_code, reason = "read by Task 13's PACKAGE check")]
    pub(crate) fn package(self) -> CallerPackage {
        self.package
    }
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
    pub(crate) fn not_in_arena(&self, value: ObjRef) -> &[u8] {
        assert!(value.class_id().is_some(), "a live value");
        self.class_default_name(value)
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
    /// receiver, and from which scope.
    ///
    /// `start_scope` is the `target~name:scope` override. `None` is the
    /// ordinary lookup, `RexxBehaviour::methodLookup`; `Some` is
    /// `RexxObject::superMethod(msgname, startscope)`, which searches only
    /// the starting scope itself and the scopes folded in ahead of it.
    ///
    /// Nothing here is cached, per D28. The answer depends on the receiver's
    /// behaviour as it stands at this instant, and a `~define` between two
    /// sends of the same name at the same call site must change the second
    /// one's answer.
    ///
    /// `caller` is the sending side, which the access scopes read and the
    /// receiver does not carry -- [`Caller`] has each one's C++ site.
    pub(crate) fn resolve(
        &mut self,
        receiver: ObjRef,
        name: &[u8],
        start_scope: Option<ObjRef>,
        caller: Caller,
    ) -> Option<Resolution> {
        // Nothing reads `caller` here: the access scopes are what read it and
        // none is implemented in this crate, which
        // `rexx-classes/src/registry.rs:414`-`:416` records for the native
        // side. It is a parameter now so that the sites that will pass it are
        // already passing it.
        let _ = caller;
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
        let cleared = seam::clear(self, receiver, name, args)?;
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
            Some(resolution) => self.invoke(resolution, receiver, name, args),
            None => self.unknown_or_nomethod(receiver, name, args, caller),
        }
    }

    /// **The last two steps of the documented search order**: `UNKNOWN` on the
    /// receiver's own behaviour, and the `NOMETHOD` condition beneath it when
    /// the behaviour answers no `UNKNOWN` either -- `RexxObject::processUnknown`
    /// (`classes/ObjectClass.cpp:1002`), reached from `messageSend` at `:904`.
    ///
    /// **The forward's two arguments are the missed message name and an
    /// `Array` of the send's own arguments**, in that order (`:1013`, then
    /// `:1018`-`:1019`).
    /// The array is the argument list as the send holds it, omissions
    /// included: measured, `.k~zork(1,,3)` reaches an `UNKNOWN` whose
    /// `arguments~items` is `2` and whose `~size` is `3`, while `.k~zork()`
    /// answers `0` for both.
    ///
    /// **The `UNKNOWN` lookup is the ordinary one and carries no start
    /// scope**, whatever the missed send's own scope override was:
    /// `processUnknown` asks `behaviour->methodLookup(GlobalNames::UNKNOWN)`
    /// on both of `messageSend`'s paths.
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
        caller: Caller,
    ) -> Result<Option<ObjRef>, Failure> {
        let Some(resolution) = self.resolve(receiver, UNKNOWN, None, caller) else {
            return Err(self.nomethod(receiver, name));
        };
        // **Both of the forward's arguments are rooted, and only one of the
        // two roots has a witness.** `alloc_with` collects *before* it
        // allocates, so the array's own root is what carries it across the
        // `text` below -- a message name too long for a handle allocates
        // there, and `corpus/lang/message_send_unknown_forward.rex`'s last
        // row is that name. Measured with `collect_stress.rs`'s
        // collect-on-every-allocation and this root removed: that row panics
        // in `to_text` and a name of seven bytes or fewer does not, because
        // then nothing allocates between the two.
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
    /// neither the message nor `UNKNOWN`, and **which of two conditions it is
    /// depends on what is armed**.
    ///
    /// `reportNomethod` (`concurrency/ActivityManager.hpp:509`) offers a
    /// `NOMETHOD` condition to the activation stack first and raises the
    /// 97.1 syntax error only when nothing took it, so the two are separate
    /// answers a program can tell apart. Measured, `say 'abc'~nosuchmsg`:
    /// under `signal on nomethod` it traps with `CONDITION('C')` `NOMETHOD`,
    /// `CONDITION('D')` `NOSUCHMSG`, `CONDITION('E')` the null string and
    /// `RC` untouched; under `signal on syntax` alone it traps with `C`
    /// `SYNTAX`, `D` the null string, `E` `1` and `RC` `97`; with a
    /// `SIGNAL ON SYNTAX` inside a routine and a `SIGNAL ON NOMETHOD` in its
    /// caller, the **caller's** `NOMETHOD` handler runs, so the offer really
    /// does cross activations before the degradation happens.
    ///
    /// That is why the choice is made here rather than left to
    /// [`Interp::offer_to_trap`]: that function sees one activation at a
    /// time as the failure unwinds, and by the time the outermost one has
    /// declined, the inner `SYNTAX` traps that the degraded error is owed
    /// have already been passed.
    fn nomethod(&mut self, receiver: ObjRef, name: &[u8]) -> Failure {
        let target = self.message_target_text(receiver);
        if self.trapped_anywhere(b"NOMETHOD") {
            Raised::nomethod(&target, name).into()
        } else {
            Raised::no_method(&target, name).into()
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

/// Whether a value has **no** string value, which is what the oracle raises
/// 88.909 for at an argument that must be text.
///
/// `stringArgument` (`runtime/MethodArguments.hpp:136`) reaches
/// `RexxInternalObject::requiredString` (`classes/ObjectClass.cpp:1373`),
/// which asks the argument for `makeString()` and raises when that answers
/// `.nil`. Only the string-valued primitives answer it, so every value shape
/// a program can put in this argument position is 88.909 there except a
/// string, a number, and a stem standing for one. Measured, three descriptors
/// against the oracle:
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
/// **Not [`Interp::operator_operand_gap`], and the difference is `.nil`.**
/// That predicate passes `.nil` through as text on purpose, because an
/// operator here compares its rendering; `requiredString` refuses it. The
/// stem redirect is the same in both and for the same reason -- `to_text`
/// answers a stem *as* its default, so a test stopping at the stem handle
/// would let the last two rows above through.
///
/// **Scoped to this argument rather than to native arguments in general**:
/// the surface where each native checks its own is owned by the phase named
/// against the argument row in `docs/superpowers/plans/phase-4-exclusions.txt`.
fn lacks_a_string_value(interp: &Interp, value: ObjRef) -> bool {
    match value.decode() {
        Decoded::Nil => true,
        Decoded::SmallInt(_) | Decoded::Text(_) => false,
        // Asked before the arena is, for the reason `receiver_kind` gives:
        // a class identity is heap-tagged and names no slot.
        Decoded::Heap { .. } if value.class_id().is_some() => true,
        Decoded::Heap { .. } => match interp.heap.get(value) {
            None => false,
            Some(object) => match &object.body {
                Body::Native(_) => true,
                Body::Stem {
                    default: Some(default),
                    ..
                } => lacks_a_string_value(interp, *default),
                _ => false,
            },
        },
    }
}

/// `Object~hasMethod(name)`: whether the receiver's behaviour answers `name`.
///
/// The argument is upcased before the lookup, measured:
/// `'abc'~hasMethod('length')` is `1`. An argument with no string value is
/// 88.909 rather than an answer of `0`, measured -- see
/// [`lacks_a_string_value`] for which shapes those are.
fn native_has_method(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_method_argument(1).into());
    };
    if lacks_a_string_value(interp, argument) {
        return Err(Raised::argument_needs_a_string_value(1).into());
    }
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
    if lacks_a_string_value(interp, argument) {
        return Err(Raised::named_argument_needs_a_string_value("method name").into());
    }
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
    if lacks_a_string_value(interp, argument) {
        return Err(Raised::named_argument_needs_a_string_value("index").into());
    }
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
            if lacks_a_string_value(interp, argument) {
                return Err(Raised::argument_needs_a_string_value(1).into());
            }
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
            if lacks_a_string_value(interp, argument) {
                return Err(Raised::argument_needs_a_string_value(2).into());
            }
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
        assert!(
            interp
                .resolve(receiver, b"NOSUCHMETHOD", None, no_caller())
                .is_none()
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
                .is_some()
        );
        assert!(
            interp
                .resolve(receiver, b"LENGTH", Some(object), no_caller())
                .is_none()
        );
        assert!(
            interp
                .resolve(receiver, b"ISNIL", Some(string), no_caller())
                .is_some()
        );
        assert!(
            interp
                .resolve(receiver, b"LENGTH", Some(string), no_caller())
                .is_some()
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
    /// option" from passing: `PROTECTED` and `UNGUARDED` run on the oracle
    /// and must run here, and a `::ATTRIBUTE GET` with a body of its own is
    /// the one attribute form that is a written method rather than a
    /// generated accessor.
    #[test]
    fn a_method_body_this_crate_cannot_run_is_loud_and_its_neighbours_still_run() {
        // (source, the phrase the refusal must name)
        let refused: &[(&str, &str)] = &[
            // oracle 97.2 at rc 159, `cannot accept private message "M" from
            // this context` -- which caller's scope may send it is not
            // modelled here at all, so raising 97.2 for every private send
            // would refuse the ones the oracle allows.
            (
                "say .K~m\n::class K\n::method m class private\n  return 7\n",
                "a PRIVATE ::METHOD",
            ),
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
            // uninitialised class-scope instance variable, which is Task 8's.
            (
                "say .K~a\n::class K\n::attribute a class\n",
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
}
