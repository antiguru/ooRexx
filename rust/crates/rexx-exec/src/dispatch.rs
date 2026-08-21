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
use rexx_core::{Body, Decoded, ObjRef};
use rexx_parse::Expr;

use crate::activation::{Activation, MethodIdentity, body_of};
use crate::error::{FailureSite, Raised};
use crate::plan::BodyKey;
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
type NativeMethod = fn(&mut Interp, Cleared, ObjRef, &[Option<ObjRef>]) -> Result<ObjRef, Failure>;

/// What a resolved [`MethodId`] runs.
///
/// The kinds are not interchangeable and are kept apart rather than
/// hidden behind one closure: only a Rexx body can produce a send with no
/// value, only a native method has a declared arity to check, and only a
/// native method contributes a `Compiled method` traceback line.
enum Invocable {
    Native(NativeEntry),
    Rexx(crate::InstalledMethodBody),
}

/// What [`Interp::invoke`] needs about one primitive method beyond its code.
#[derive(Copy, Clone)]
struct NativeEntry {
    /// The number of parameters the method declares, which is the number the
    /// oracle's own 93.902 names -- measured, `'abc'~length(1)` is `0
    /// expected` and `'abc'~hasMethod('a','b')` is `1 expected`.
    arity: usize,
    run: NativeMethod,
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
static NATIVE_METHODS: &[(&str, &str, usize, NativeMethod)] = &[
    ("Object", "HASMETHOD", 1, native_has_method),
    ("Object", "ISNIL", 0, native_is_nil),
    ("String", "LENGTH", 0, native_length),
    ("String", "REVERSE", 0, native_reverse),
];

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
    /// `.String`, `.Object` and `.Class`, resolved once at bootstrap. These
    /// are handles into `classes` and not a second model of it, and holding
    /// them buys two things: a send does not look its receiver's class up by
    /// name (which upcases and allocates) on every message, and neither a
    /// send nor a `::CLASS` install can be redirected by a program that
    /// declares a class of one of those three names.
    string: ObjRef,
    object: ObjRef,
    metaclass: ObjRef,
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
        ObjectModel {
            classes,
            natives,
            string,
            object,
            metaclass,
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
    /// `.nil`, measured: `.nil~class~id` is `Object`.
    Object,
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
    fn receiver_kind(&self, receiver: ObjRef) -> Result<Primitive, &'static str> {
        match receiver.decode() {
            Decoded::Nil => Ok(Primitive::Object),
            Decoded::SmallInt(_) | Decoded::Text(_) => Ok(Primitive::String),
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
                    Body::Array(_) => Err("an array"),
                    Body::Instance(_) => Err("an instance of a user class"),
                    Body::WeakRef(_) => Err("a weak reference"),
                    // `.environment`, `.local`, `.methods` and `.context`.
                    // Their classes are in the registry, so there is a
                    // behaviour to resolve against -- what is missing is a
                    // `NATIVE_METHODS` row for anything a `Directory`, a
                    // `StringTable` or a `RexxContext` answers, and answering
                    // 97.1 for a name the oracle implements is the wrong
                    // failure. Loud until a task implements those methods.
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
            Primitive::String => Behaviour::Instance(model.string),
            Primitive::Object => Behaviour::Instance(model.object),
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
    pub(crate) fn resolve(
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
    /// **`None` is a send that produced no value**, which only a Rexx body
    /// can do -- a [`NativeMethod`] returns an `ObjRef` by its own signature.
    /// Measured, `::method m class` ending in a bare `return`: as a whole
    /// clause it drops `RESULT` at rc 0, and in an expression it is 91.999 at
    /// rc 165.
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
                let outcome = if args.len() > entry.arity {
                    Err(Raised::too_many_method_arguments(entry.arity).into())
                } else {
                    (entry.run)(self, cleared, receiver, args)
                };
                if outcome.is_err() {
                    let scope = self.classes().id_string(resolution.scope).to_string();
                    self.blame_native_method(name, &scope);
                }
                outcome.map(Some)
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
        let saved_context = std::mem::replace(
            &mut self.call_context,
            crate::CallContext {
                name: name.to_vec(),
                arguments: args
                    .iter()
                    .map(|arg| arg.map(crate::Argument::Value))
                    .collect(),
            },
        );

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

    /// [`Interp::resolve`] then [`Interp::invoke`], with the oracle's 97.1
    /// for a name the receiver's behaviour does not answer.
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
    ) -> Result<Option<ObjRef>, Failure> {
        if let Err(kind) = self.receiver_kind(receiver) {
            return Err(Loud::receiver_class(kind).into());
        }
        match self.resolve(receiver, name, start_scope) {
            Some(resolution) => self.invoke(resolution, receiver, name, args),
            None => {
                let target = self.to_text(receiver).to_vec();
                Err(Raised::no_method(&target, name).into())
            }
        }
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
        let result = evaluated.and_then(|()| self.send_message(receiver, name, None, &values));
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
    /// `pub(crate)`: an arithmetic operator's left operand, and a
    /// controlled `DO` header's numeric position, can each raise from
    /// inside this same shape of frame when the receiver is a stem
    /// forwarding to a native method -- see `eval.rs`'s
    /// `Interp::blame_stem_forwarded_operator` -- and `eval.rs` is a
    /// different module from this one.
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
) -> Result<ObjRef, Failure> {
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
    Ok(interp.counted(usize::from(answers)))
}

/// `Object~isNil`: `1` for `.nil` and `0` for everything else.
fn native_is_nil(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    Ok(interp.counted(usize::from(receiver == ObjRef::NIL)))
}

/// `String~length`: the receiver's own byte count.
fn native_length(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let length = interp.text_len(receiver);
    Ok(interp.counted(length))
}

/// `String~reverse`: the receiver's own bytes, last to first.
fn native_reverse(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let mut bytes = interp.to_text(receiver).to_vec();
    bytes.reverse();
    Ok(interp.text_built(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

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
            .resolve(receiver, b"LENGTH", None)
            .expect("String answers LENGTH");
        assert_eq!(own.scope, string);
        let inherited = interp
            .resolve(receiver, b"ISNIL", None)
            .expect("Object's ISNIL reaches a String receiver");
        assert_eq!(inherited.scope, object);
        assert!(interp.resolve(receiver, b"NOSUCHMETHOD", None).is_none());
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

        assert!(interp.resolve(receiver, b"ISNIL", Some(object)).is_some());
        assert!(interp.resolve(receiver, b"LENGTH", Some(object)).is_none());
        assert!(interp.resolve(receiver, b"ISNIL", Some(string)).is_some());
        assert!(interp.resolve(receiver, b"LENGTH", Some(string)).is_some());
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
            interp.send_message(receiver, b"UPPER", None, &[]),
            Err(Failure::Loud(_))
        ));
        assert!(matches!(
            interp.send_message(receiver, b"NOSUCHMETHOD", None, &[]),
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
            interp.send_message(class, b"LENGTH", None, &[]),
            Err(Failure::Raised(_))
        ));
        let name = interp.text(b"LENGTH");
        assert_eq!(
            interp
                .send_message(class, b"HASMETHOD", None, &[Some(name)])
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
            interp.send_message(stem, b"LENGTH", None, &[]),
            Err(Failure::Loud(_))
        ));
    }
}
