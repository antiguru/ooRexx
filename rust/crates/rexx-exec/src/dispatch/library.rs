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

//! Running a method whose body is a procedure of a loaded shared library, and
//! the [`Host`] the boundary reaches this interpreter through.
//!
//! `NativeActivation::run` (`interpreter/execution/NativeActivation.cpp:1264`)
//! is the reference for the call, and the `NativeActivation` that runs it is
//! also what serves the context, which is why one frame carries both the
//! receiver's pool and the local references.

use std::borrow::Cow;
use std::rc::Rc;

use rexx_api::ffi::Contexts;
use rexx_api::handles::Table;
use rexx_api::invoke;
use rexx_api::layout::POINTER;
use rexx_api::values::{
    Activation, CStringPool, Constants, Conversion, Failure as Refused, Host, Raised as Condition,
};
use rexx_core::{BehaviourId, Body, Decoded, ObjRef};

use super::Resolution;
use crate::error::Raised;
use crate::{Failure, Interp, LibraryBinding, Loud, NativeFrame};

/// The bytes a small integer or an inline string renders as, for a reader
/// that cannot allocate into the interpreter.
fn rendered(value: ObjRef) -> Option<Vec<u8>> {
    match value.decode() {
        Decoded::SmallInt(number) => Some(number.to_string().into_bytes()),
        Decoded::Text(inline) => Some(inline.to_vec()),
        _ => None,
    }
}

/// The pool name an extension's spelling addresses, or `None` for one
/// `getVariableRetriever` (`execution/VariableDictionary.cpp:738`) answers
/// nothing for: the write then does nothing, which is all the API can see of
/// the refusal.
fn pool_variable_name(name: &[u8]) -> Option<Vec<u8>> {
    let first = *name.first()?;
    if first.is_ascii_digit() || name.contains(&b'.') {
        return None;
    }
    Some(name.to_ascii_uppercase())
}

impl Interp {
    /// Runs `binding`'s procedure against `args` with `receiver` as its self,
    /// and answers what the extension returned.
    ///
    /// # Errors
    /// The condition the extension raised, whatever converting an argument or
    /// the result refuses, and [`Loud`] for a conversion this phase has not
    /// written.
    pub(super) fn run_library_method(
        &mut self,
        binding: &LibraryBinding,
        resolution: Resolution,
        receiver: ObjRef,
        args: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        let owner = self.pool_owner(receiver)?;
        // Cloned out of the binding before the interpreter is borrowed as the
        // host: the row is borrowed from the library, and the library has to
        // outlive that borrow without being reachable through `self`.
        let library = Rc::clone(&binding.library);
        let Some(entry) = library.method(&binding.procedure) else {
            return Err(
                Loud::external_entry_point("a library procedure that stopped resolving").into(),
            );
        };
        self.native_handles.push(NativeFrame {
            owner,
            scope: resolution.scope,
            locals: Table::new(),
            raised: None,
        });
        let mut strings = CStringPool::new();
        let (answered, pending) = {
            let activation = Activation::new(Conversion {
                host: self,
                strings: &mut strings,
            });
            let mut contexts = Contexts::new(&activation);
            let answered = invoke::method(entry, contexts.method(), &activation, args);
            (answered, activation.pending())
        };
        let frame = self
            .native_handles
            .pop()
            .expect("the frame pushed above is still the innermost");

        // The condition first, because the oracle raises it in the caller's
        // frame once the call has returned (`NativeActivation::checkConditions`,
        // `:1787`) and before it does anything with the value the extension
        // wrote. Measured, oracle: `.MyRe~new('[')` over `RegExp_Init` is
        // `Error 38 ... Invalid template or pattern.` at rc 218, and the
        // extension returned zero on that path.
        if let Some(number) = pending {
            return Err(condition_of(number));
        }
        match answered {
            Err(Refused::Raised) => Err(frame
                .raised
                .expect("a host answering Raised holds the condition it raised")),
            answered => answered.map_err(refusal),
        }
    }
}

/// The condition an extension raised, whose number is the major and the minor
/// packed as `major * 1000 + minor` (`api/oorexxerrors.h`).
fn condition_of(number: usize) -> Failure {
    let major = u16::try_from(number / 1000).unwrap_or(u16::MAX);
    let minor = u16::try_from(number % 1000).unwrap_or(0);
    Raised::syntax(major, minor, Vec::new()).into()
}

/// What the boundary's own refusal reports.
fn refusal(refused: Refused) -> Failure {
    match refused {
        Refused::MissingArgument { position } => Raised::missing_native_argument(position).into(),
        Refused::NoStringValue { position } => {
            Raised::native_argument_needs_a_string_value(position).into()
        }
        Refused::TooManyArguments { expected } => {
            Raised::too_many_external_arguments(expected).into()
        }
        Refused::Signature => Raised::incorrect_method_signature().into(),
        Refused::ResultSignature => Raised::incorrect_method_result_signature().into(),
        Refused::Unfilled { .. } | Refused::StaleHandle | Refused::Raised => Loud {
            message: crate::owned_message(&format!("{refused}"), Some("Phase 8")),
        }
        .into(),
    }
}

impl Host for Interp {
    fn is_method(&self) -> bool {
        true
    }

    fn string_value(&mut self, object: ObjRef) -> Result<Option<ObjRef>, Condition> {
        match self.blamed_string_conversion(object) {
            Ok(converted) => {
                if let Some(text) = converted {
                    self.roots.push_temp(text);
                }
                Ok(converted)
            }
            Err(failure) => {
                self.native_handles
                    .last_mut()
                    .expect("a native activation is running")
                    .raised = Some(failure);
                Err(Condition)
            }
        }
    }

    fn string_bytes(&self, object: ObjRef) -> Option<Cow<'_, [u8]>> {
        if let Some(bytes) = rendered(object) {
            return Some(Cow::Owned(bytes));
        }
        match &self.heap.get(object)?.body {
            Body::Text { bytes, .. } => Some(Cow::Borrowed(bytes.as_slice())),
            Body::Num { text, .. } => text.as_deref().map(Cow::Borrowed),
            _ => None,
        }
    }

    fn cself(&mut self) -> Option<POINTER> {
        let frame = self.native_handles.last()?;
        let (owner, scope) = (frame.owner, frame.scope);
        let held = self.pools_of(owner)?.get(scope, b"CSELF")?;
        match &self.heap.get(held)?.body {
            Body::Instance {
                native: Some(state),
                ..
            } => state.pointer(),
            _ => None,
        }
    }

    fn constants(&mut self) -> Constants<ObjRef> {
        Constants {
            nil: ObjRef::NIL,
            true_object: self.counted(1),
            false_object: self.counted(0),
            null_string: self.text(b""),
        }
    }

    fn set_object_variable(&mut self, name: &[u8], value: Option<ObjRef>) {
        let Some(frame) = self.native_handles.last() else {
            return;
        };
        let (owner, scope) = (frame.owner, frame.scope);
        let Some(name) = pool_variable_name(name) else {
            return;
        };
        match value {
            Some(value) => self.set_pool_variable(owner, scope, &name, value),
            None => self.clear_pool_variable(owner, scope, &name),
        }
    }

    fn drop_object_variable(&mut self, name: &[u8]) {
        self.set_object_variable(name, None);
    }

    fn whole_number(&mut self, value: isize) -> ObjRef {
        match i64::try_from(value).ok().and_then(ObjRef::small_int) {
            Some(object) => object,
            None => self.text(value.to_string().as_bytes()),
        }
    }

    fn new_pointer(&mut self, value: POINTER) -> ObjRef {
        let class = self
            .classes()
            .lookup("Pointer")
            .expect("Pointer is a native class");
        let behaviour = self.classes().instance_behaviour_handle(class);
        let body = Body::pointer(class, behaviour, value);
        let object = self.alloc_with(BehaviourId::OBJECT, body);
        self.roots.push_temp(object);
        object
    }

    fn locals(&mut self) -> &mut Table {
        &mut self
            .native_handles
            .last_mut()
            .expect("a native activation is running")
            .locals
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::rc::Rc;

    use rexx_api::values::Failure as Refused;

    use super::{pool_variable_name, refusal};
    use crate::{Failure, Interp, LibraryLoad};

    /// The oracle's own build directory, whose `librxregexp.so` D5's amendment
    /// makes the instrument: it is loaded, never rebuilt.
    fn oracle_library_directory() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../build/lib")
            .canonicalize()
            .expect("the oracle's build directory is four above this crate")
    }

    /// The oracle's `librxregexp.so`, opened by path.
    fn open_rxregexp() -> rexx_api::load::Library {
        rexx_api::load::open_path(
            &oracle_library_directory().join("librxregexp.so"),
            "rxregexp",
        )
        .expect("the extension asks for 4.0.0, which is below this interpreter")
        .expect("the extension publishes RexxGetPackage")
    }

    /// An interpreter whose own `LD_LIBRARY_PATH` names that directory. The
    /// process variable is not touched: this is the interpreter's copy, which
    /// is the one the search reads.
    fn interp_that_can_see_rxregexp() -> Interp {
        let mut interp = Interp::new();
        let directory = oracle_library_directory()
            .into_os_string()
            .into_encoded_bytes();
        interp.env_set(b"LD_LIBRARY_PATH", Some(directory));
        interp
    }

    /// **One resolution path, and the assertion is on the side effect rather
    /// than on the answer.** Two `Loaded` answers would look alike whether or
    /// not the second one re-opened the library; the same `Rc` says the
    /// `dlopen` and the package read happened once.
    #[test]
    fn a_library_named_twice_is_opened_once() {
        let mut interp = interp_that_can_see_rxregexp();
        let LibraryLoad::Loaded(first) = interp.resolve_library(b"rxregexp") else {
            panic!("librxregexp.so did not load from the oracle's build directory");
        };
        let LibraryLoad::Loaded(second) = interp.resolve_library(b"rxregexp") else {
            panic!("the second resolve did not load");
        };
        assert!(
            std::rc::Rc::ptr_eq(&first, &second),
            "the same name answered two different libraries, so it was opened twice"
        );

        // The control that says the pointer comparison can tell two opens
        // apart at all: a second interpreter opens its own.
        let mut other = interp_that_can_see_rxregexp();
        let LibraryLoad::Loaded(elsewhere) = other.resolve_library(b"rxregexp") else {
            panic!("librxregexp.so did not load for the second interpreter");
        };
        assert!(
            !std::rc::Rc::ptr_eq(&first, &elsewhere),
            "two interpreters shared one library object, so ptr_eq is not measuring an open"
        );
    }

    /// The search directories come from the interpreter's own environment,
    /// which is what makes the name reachable at all: without the variable the
    /// same name answers nothing.
    #[test]
    fn the_search_path_is_what_makes_the_name_resolve() {
        let mut bare = Interp::new();
        bare.env_set(b"LD_LIBRARY_PATH", None);
        assert!(
            matches!(bare.resolve_library(b"rxregexp"), LibraryLoad::Missing),
            "librxregexp.so is on the process loader's own path, so the test above \
             is not reading the directory it supplies"
        );
        let mut seeing = interp_that_can_see_rxregexp();
        assert!(matches!(
            seeing.resolve_library(b"rxregexp"),
            LibraryLoad::Loaded(_)
        ));
    }

    /// A name that resolved to nothing is not held, so the next ask opens
    /// again: `PackageManager::loadLibrary` removes a package whose load
    /// answered false (`interpreter/package/PackageManager.cpp:240-244`).
    #[test]
    fn a_name_that_resolves_to_nothing_is_not_held() {
        let mut interp = Interp::new();
        assert!(matches!(
            interp.settle_library(b"zorkolib", Ok(None)),
            LibraryLoad::Missing
        ));
        assert!(interp.libraries.get(b"zorkolib").is_none());
    }

    /// A library whose version check refused raises on the ask that opened it
    /// and is held, so every later ask answers it loaded without opening
    /// anything: measured on the oracle with a forged package entry asking
    /// for 6.0.0, `loadLibrary` is 98.982 and then `1`, and a method of the
    /// library binds and runs afterwards.
    #[test]
    fn a_version_refused_library_raises_once_and_is_held() {
        let mut interp = Interp::new();
        let refused = rexx_api::load::Refused {
            failure: rexx_api::load::Failure::LibraryVersion("forgever".to_owned()),
            library: Box::new(open_rxregexp()),
        };
        assert!(matches!(
            interp.settle_library(b"forgever", Err(refused)),
            LibraryLoad::Version
        ));
        // No `forgever` is on any search path, so an ask that opened again
        // would answer `Missing`.
        let LibraryLoad::Loaded(held) = interp.resolve_library(b"forgever") else {
            panic!("the refused library was not held");
        };
        assert!(held.method(b"RegExp_Parse").is_some());
    }

    /// **The end-to-end witness answers the same under a collection at every
    /// allocation as it does under none.** The stress harness cannot say this:
    /// `collect_stress` reads `phase-8.txt` last and aborts on a pre-existing
    /// panic before it gets there.
    ///
    /// **What it does not say**, measured: with `Interp::object_roots`
    /// dropping `native_handles` entirely this still passes, because the
    /// receiver is rooted by the send's own temps and `Host::new_pointer`
    /// pushes what it mints as a temp before handing it back. Whether the
    /// frame's own rooting is live is
    /// `a_native_activations_local_references_are_roots_only_while_it_lives`,
    /// which does fail under that edit.
    #[test]
    fn a_library_call_answers_the_same_under_a_collection_at_every_allocation() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../corpus/lang/library_method_external.rex");
        let text = std::fs::read(&path).expect("the corpus witness is readable");
        let invocation = || {
            crate::Invocation::none().with_environment(vec![(
                b"LD_LIBRARY_PATH".to_vec(),
                oracle_library_directory()
                    .into_os_string()
                    .into_encoded_bytes(),
            )])
        };
        let name = path.to_string_lossy().into_owned();

        let plain = crate::run_program(&name, text.clone(), invocation());
        // The value the comparison rests on, so that two identical failures
        // cannot pass for agreement.
        assert_eq!(
            plain.exit_code,
            0,
            "{}",
            String::from_utf8_lossy(&plain.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&plain.stdout),
            "parse 0\npos 3\nmatch 1\nlastpos 3\nreparse 0\nmatch 1\nminimal 0\nfind 3\nfindpos 5\n"
        );

        let swept = crate::run_program_collect_every_alloc(&name, text, invocation());
        assert_eq!(swept.exit_code, plain.exit_code);
        assert_eq!(swept.stdout, plain.stdout);
        assert_eq!(swept.stderr, plain.stderr);
    }

    /// A name that has loaded keeps the library it loaded, which is what stops
    /// a second write dropping the interpreter's own reference to it.
    #[test]
    fn a_held_library_is_not_replaced_by_a_later_answer() {
        let (first, second) = (Rc::new(open_rxregexp()), Rc::new(open_rxregexp()));
        let mut libraries = crate::Libraries::new();
        assert!(Rc::ptr_eq(
            &libraries.hold(b"rxregexp", Rc::clone(&first)),
            &first
        ));
        assert!(Rc::ptr_eq(
            &libraries.hold(b"rxregexp", Rc::clone(&second)),
            &first
        ));
        assert!(Rc::ptr_eq(
            libraries.get(b"rxregexp").expect("a library was held"),
            &first
        ));
        // The control that says the second library was refused rather than
        // never built: under a name nothing is held for it is taken.
        assert!(Rc::ptr_eq(
            &libraries.hold(b"zorkolib", Rc::clone(&second)),
            &second
        ));
    }

    /// A loaded library survives a collection and the finalizer sweep, which
    /// is what an object still holding a `CSELF` at termination needs: its
    /// `UNINIT` is an address inside the mapping.
    #[test]
    fn a_loaded_library_outlives_the_finaliser_sweep() {
        let mut interp = interp_that_can_see_rxregexp();
        let LibraryLoad::Loaded(library) = interp.resolve_library(b"rxregexp") else {
            panic!("librxregexp.so did not load from the oracle's build directory");
        };
        let watch = std::rc::Rc::downgrade(&library);
        drop(library);
        interp.collect_now();
        assert!(interp.run_termination_uninits().is_empty());
        assert!(
            watch.upgrade().is_some(),
            "the interpreter gave up its library across a collection and the sweep"
        );

        // The control, and what says the assertion above is about the
        // interpreter's hold rather than some other one: the interpreter is
        // the only holder, so the watch reports a release as soon as it goes.
        drop(interp);
        assert!(
            watch.upgrade().is_none(),
            "something outside the interpreter holds the library, so the watch \
             above cannot see a release"
        );
    }

    /// **A finalizer allocates**, and nothing about the sweep it runs in
    /// forbids that: a collection only readies an object, and the sweep is
    /// reached afterwards from `GC('force')` and from termination. The
    /// allocating part of this witness is the Rexx subclass finalizer around
    /// the native one, which sends and says; `RegExp_Uninit` itself allocates
    /// nothing. The witness answers the same with a collection at every
    /// allocation as with none.
    #[test]
    fn the_native_finaliser_answers_the_same_under_a_collection_at_every_allocation() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../corpus/lang/library_uninit_collected.rex");
        let text = std::fs::read(&path).expect("the corpus witness is readable");
        let invocation = || {
            crate::Invocation::none().with_environment(vec![(
                b"LD_LIBRARY_PATH".to_vec(),
                oracle_library_directory()
                    .into_os_string()
                    .into_encoded_bytes(),
            )])
        };
        let name = path.to_string_lossy().into_owned();

        let plain = crate::run_program(&name, text.clone(), invocation());
        assert_eq!(
            plain.exit_code,
            0,
            "{}",
            String::from_utf8_lossy(&plain.stderr)
        );
        // The oracle's own bytes, so that two identical failures cannot pass
        // for agreement: `sub after String` is `DropObjectVariable` reaching
        // the pool, which only the extension's own code writes.
        assert_eq!(
            String::from_utf8_lossy(&plain.stdout),
            "live Pointer\nsub before Pointer\nsub after String\ndone\n"
        );

        let swept = crate::run_program_collect_every_alloc(&name, text, invocation());
        assert_eq!(swept.exit_code, plain.exit_code);
        assert_eq!(swept.stdout, plain.stdout);
        assert_eq!(swept.stderr, plain.stderr);
        assert!(swept.collections > 0, "the stress mode did not collect");
    }

    /// Which refusals are reported against the declaring package with no
    /// line. Measured, oracle rc 163 and 168: 93.968 for a parameter code and
    /// 88.909 for an argument with no string value name the package; 93.968
    /// for a return word carrying `OPTIONAL` names the sending clause's line.
    #[test]
    fn a_refusal_before_the_call_is_lineless_and_one_after_it_is_not() {
        let lineless = |refused: Refused| match refusal(refused) {
            Failure::Raised(raised) => raised.delivery.lineless,
            _ => panic!("every condition-shaped refusal is a raise"),
        };
        assert!(lineless(Refused::MissingArgument { position: 1 }));
        assert!(lineless(Refused::NoStringValue { position: 1 }));
        assert!(lineless(Refused::TooManyArguments { expected: 0 }));
        assert!(lineless(Refused::Signature));
        assert!(!lineless(Refused::ResultSignature));
    }

    /// The spellings `getVariableRetriever` answers nothing for, beside
    /// `CSELF` and `!POS`, which `rxregexp` really writes.
    #[test]
    fn a_pool_name_is_upcased_and_a_compound_one_is_refused() {
        assert_eq!(pool_variable_name(b"CSELF").as_deref(), Some(&b"CSELF"[..]));
        assert_eq!(pool_variable_name(b"!POS").as_deref(), Some(&b"!POS"[..]));
        assert_eq!(pool_variable_name(b"cself").as_deref(), Some(&b"CSELF"[..]));
        assert_eq!(pool_variable_name(b""), None);
        assert_eq!(pool_variable_name(b"a.b"), None);
        assert_eq!(pool_variable_name(b"1x"), None);
    }
}
