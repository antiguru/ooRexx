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
//! and is neither `Copy` nor `Clone`, and a [`NativeMethod`] takes one by
//! value, so no primitive method runs without a value produced inside
//! [`mod seam`](seam) -- the compiler refuses the token's tuple-struct
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

use std::collections::HashMap;

use rexx_classes::{ClassRegistry, MethodId};
use rexx_core::{Body, Decoded, ObjRef};
use rexx_parse::Expr;

use crate::error::{FailureSite, Raised};
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
    /// that can produce a value of this type, and a
    /// [`NativeMethod`](super::NativeMethod) takes one, so no native method
    /// runs without the seam having been passed first.
    pub(super) struct Cleared(());

    /// **The dispatch chokepoint (D45, site one).** Every native method
    /// invocation passes here, and a manager installed in a later phase gets
    /// its hook in this function's body.
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

    /// `.Object` and `.Class`: the superclass and metaclass
    /// `RexxClass::subclass` defaults to, which is what a bare `::CLASS`
    /// declares (`ClassClass.cpp:1562`).
    pub(crate) fn root_and_metaclass(&mut self) -> (ObjRef, ObjRef) {
        let model = self.object_model();
        (model.object, model.metaclass)
    }

    /// Which native class a value answers to, or the value's own shape when
    /// this phase builds no class for it.
    ///
    /// A stem answers `Stem` on the oracle, and
    /// `rexx_classes::deferred_classes` does not build that class (its
    /// `Setup.cpp` block hides the comparison methods, and `MethodDict`
    /// models no removal), so a stem receiver resolves nothing and fails
    /// loudly rather than answering from the wrong class. Every other heap
    /// shape gets a refusal naming itself rather than a shared one, so
    /// whichever task makes one reachable as a receiver gets a message that
    /// says which.
    fn receiver_kind(&self, receiver: ObjRef) -> Result<Primitive, &'static str> {
        match receiver.decode() {
            Decoded::Nil => Ok(Primitive::Object),
            Decoded::SmallInt(_) | Decoded::Text(_) => Ok(Primitive::String),
            // **Asked before the arena is**, which is the whole point of
            // `rexx_core::CLASS_SLOT_BASE`: a class identity is heap-tagged
            // and names no slot, so reaching for the arena with one answers
            // from whatever object happens to hold that index.
            Decoded::Heap { .. } if receiver.class_id().is_some() => Err("a class object"),
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
                },
            },
        }
    }

    /// The class object a value answers to, or the value's own shape when
    /// this phase builds no class for it.
    fn class_of_receiver(&mut self, receiver: ObjRef) -> Result<ObjRef, &'static str> {
        let kind = self.receiver_kind(receiver)?;
        let model = self.object_model();
        Ok(match kind {
            Primitive::String => model.string,
            Primitive::Object => model.object,
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
        let class = self.class_of_receiver(receiver).ok()?;
        // Borrowed rather than owned wherever the name is UTF-8, which every
        // name a program can write is: `from_utf8_lossy` allocates only for
        // the bytes it has to replace.
        let name = String::from_utf8_lossy(name);
        let (scope, method) = match start_scope {
            None => self.classes().lookup_instance_method(class, &name)?,
            Some(start) => self
                .classes()
                .lookup_instance_method_from_scope(class, &name, start)?,
        };
        Some(Resolution { scope, method })
    }

    /// **Step two of a send** (D24): run what [`Interp::resolve`] found.
    ///
    /// The seam is passed first and the argument count is checked after,
    /// which is the oracle's order: `messageSend` asks the security manager
    /// before `method->run`, and the count check is part of the method's own
    /// entry (`NativeActivation::run`), not of the send.
    pub(crate) fn invoke(
        &mut self,
        resolution: Resolution,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<ObjRef, Failure> {
        // The scope's `~id` is rendered only where it is printed -- a
        // successful send has no use for it, and every send would otherwise
        // pay for the copy.
        let Some(entry) = self.object_model().natives.get(&resolution.method).copied() else {
            let scope = self.classes().id_string(resolution.scope).to_string();
            return Err(Loud::native_method(name, &scope).into());
        };
        let cleared = seam::clear(self, receiver, name, args)?;
        let outcome = if args.len() > entry.arity {
            Err(Raised::too_many_method_arguments(entry.arity).into())
        } else {
            (entry.run)(self, cleared, receiver, args)
        };
        if outcome.is_err() {
            let scope = self.classes().id_string(resolution.scope).to_string();
            self.blame_native_method(name, &scope);
        }
        outcome
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
    ) -> Result<ObjRef, Failure> {
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
    ) -> Result<ObjRef, Failure> {
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
            let _scope = self.eval(code, super_class)?;
            // `RexxExpressionMessage::evaluate`'s
            // `_super->isInstanceOf(TheClassClass)`. **This crate has no
            // value that is a class object**: the only environment symbols
            // `ExprKind::DotVariable` answers are `.NIL`, `.TRUE` and
            // `.FALSE`, and `~class` resolves to a primitive method with no
            // implementation here, so every value that can reach this line
            // is a string, a number or `.nil` and none of them is an
            // instance of `.Class`. `Interp::resolve`'s own start-scope
            // argument is what a class object would be handed to, and it
            // is exercised by `dispatch::tests` rather than from a program.
            return Err(Raised::scope_override_not_a_class().into());
        }

        let mut values = self.take_value_buffer();
        let evaluated = self.evaluate_message_arguments(code, args, assigned, &mut values);
        let result = evaluated.and_then(|()| self.send_message(receiver, name, None, &values));
        self.give_value_buffer(values);
        let sent = result?;

        // `~~` replaces the result with the target **before** the send is
        // traced, so `>M>` shows the target for a cascade -- measured,
        // `'abc'~~length` traces `>M>   "LENGTH" => "abc"`
        // (`ExpressionMessage.cpp:202-219`).
        let value = if cascade { receiver } else { sent };
        // **The only `>M>` site.** Emitted here rather than from `eval`'s own
        // post-order `trace_intermediate` hook, where every other value
        // prefix lives, because the message-assignment form
        // (`Interp::exec_message`) never evaluates its term as an expression
        // and so never reaches that hook. Emitting from the one function
        // both forms do pass through is what keeps them from disagreeing.
        // The position is the same either way: this is the last thing the
        // term does before returning its value.
        if let Some(rendered) = self.intermediate_text(value) {
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
    fn blame_native_method(&mut self, name: &[u8], scope: &str) {
        if self.failure_site.is_some() {
            return;
        }
        let line = Raised::compiled_method_line(name, scope);
        self.failure_site = Some(FailureSite::Rendered(line));
        self.seal_site_level();
    }
}

/// `Object~hasMethod(name)`: whether the receiver's behaviour answers `name`.
///
/// The argument is upcased before the lookup, measured:
/// `'abc'~hasMethod('length')` is `1`. A value with no string form -- `.nil`
/// is the only one a program can write -- is 88.909 rather than an answer of
/// `0`, measured.
fn native_has_method(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_method_argument(1).into());
    };
    if argument == ObjRef::NIL {
        return Err(Raised::argument_needs_a_string_value(1).into());
    }
    let name = String::from_utf8_lossy(&interp.to_text(argument).to_ascii_uppercase()).into_owned();
    // **A receiver with no class here is loud, not `0`.** `send_message`
    // screens for it before any method runs, so this arm is unreachable
    // today; answering `0` from it anyway would make this the one place in
    // the module where a gap becomes an answer, and the answer would be one
    // the oracle contradicts.
    let class = match interp.class_of_receiver(receiver) {
        Ok(class) => class,
        Err(kind) => return Err(Loud::receiver_class(kind).into()),
    };
    let answers = interp.object_model().classes.has_method(class, &name);
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
    let length = interp.to_text(receiver).len();
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

    /// **A class identity used as a receiver is refused, not resolved through
    /// the arena.**
    ///
    /// No expression yields a class object as a value in this phase, so this
    /// is unreachable from a program -- and it is the arm that decides what
    /// happens on the first day one does. Before class identities moved to
    /// `rexx_core::CLASS_SLOT_BASE`'s reserved range, a class handle decoded
    /// as an ordinary heap handle, `heap.get` answered whatever object held
    /// that slot, and the send resolved against **that value's** class and
    /// answered from it: a silent wrong answer. With the range, `class_id()`
    /// is `Some` and this is loud.
    ///
    /// **The arrangement below is what makes the send the catcher**, and
    /// getting it wrong is what a first version of this test did. `.String`
    /// is the third identity the registry mints, so under the old minting its
    /// handle is the arena's **slot 2** -- filling slot 0 alone leaves slot 2
    /// empty, `heap.get` answers `None`, and the send is loud for the wrong
    /// reason. With the arena filled up to and including that slot, the old
    /// minting answers `17`, measured: the length of a string the program
    /// never named. There is deliberately no `class_id()` precondition here,
    /// so that reverting the minting fails this test at the send rather than
    /// ahead of it.
    #[test]
    fn a_class_identity_used_as_a_receiver_is_loud() {
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
            Err(Failure::Loud(_))
        ));
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
                tails: std::collections::HashMap::new(),
            },
        );
        assert!(matches!(
            interp.send_message(stem, b"LENGTH", None, &[]),
            Err(Failure::Loud(_))
        ));
    }
}
