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
    Activation, CStringPool, Constants, Conversion, Failure as Refused, Host, Numeric,
    Raised as Condition,
};
use rexx_core::{BehaviourId, Body, Decoded, ObjRef};
use rexx_num::Number;

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
            return Err(Loud::library_procedure_gone().into());
        };
        self.native_handles.push(NativeFrame {
            owner,
            scope: resolution.scope,
            method: true,
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
            let answered = invoke::method(entry, &contexts.method(), &activation, args);
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
        let packaged = self.external_package_path(resolution.method).is_some();
        self.settle_native_call(answered, frame, packaged)
    }

    /// Runs the routine a [`Interp::package_routine`] slot names, as the call
    /// named `name` passed it.
    ///
    /// # Errors
    /// As [`Interp::run_library_routine`].
    pub(crate) fn run_package_routine(
        &mut self,
        slot: usize,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        let code = self.package_routine_code(slot);
        self.run_library_routine(code, &name.to_ascii_uppercase(), args)
    }

    /// Runs the library routine [`Interp::library_codes`] row `code` is
    /// against `args`, and answers what the extension returned.
    ///
    /// `name` is the name the traceback's `Compiled routine` line gives.
    ///
    /// # Errors
    /// The condition the extension raised, whatever converting an argument or
    /// the result refuses, and [`Loud`] for a conversion or a routine style
    /// this crate does not implement.
    pub(crate) fn run_library_routine(
        &mut self,
        code: usize,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        let key = self.library_code_key(code).clone();
        let Some(library) = self.libraries.get(&key.library).map(Rc::clone) else {
            return Err(Loud::library_procedure_gone().into());
        };
        let Some(entry) = library.routine(&key.procedure) else {
            return Err(Loud::library_procedure_gone().into());
        };
        self.native_handles.push(NativeFrame {
            owner: ObjRef::NIL,
            scope: ObjRef::NIL,
            method: false,
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
            let answered = invoke::routine(entry, &contexts.call(), &activation, args);
            (answered, activation.pending())
        };
        let frame = self
            .native_handles
            .pop()
            .expect("the frame pushed above is still the innermost");
        let package = self.library_code_package_path(code);
        let outcome = match pending {
            Some(number) => Err(condition_of(number)),
            None => self.settle_native_call(answered, frame, package.is_some()),
        };
        if outcome.is_err() {
            self.blame_native_routine(name, package);
        }
        outcome
    }

    /// What a native call answers once its frame is off the stack: the value,
    /// the condition its string conversion raised, or the boundary's own
    /// refusal. `packaged` is whether the code reports a package, which is
    /// what a refusal before the call is reported against with no line; one
    /// that reports none is reported against the caller's clause
    /// (`Activity::generateProgramInformation`, `interpreter/concurrency/Activity.cpp:1093-1113`).
    fn settle_native_call(
        &mut self,
        answered: Result<Option<ObjRef>, Refused>,
        frame: NativeFrame,
        packaged: bool,
    ) -> Result<Option<ObjRef>, Failure> {
        match answered {
            Err(Refused::Raised) => Err(frame
                .raised
                .expect("a host answering Raised holds the condition it raised")),
            Ok(value) => Ok(value),
            Err(refused) => Err(self.refusal(refused, packaged, frame.method)),
        }
    }

    /// What the boundary's own refusal reports.
    fn refusal(&mut self, refused: Refused, packaged: bool, method: bool) -> Failure {
        let mut raised = match refused {
            Refused::MissingArgument { position } => Raised::missing_native_argument(position),
            Refused::NoStringValue { position } => {
                Raised::native_argument_needs_a_string_value(position)
            }
            Refused::InvalidDouble { position, argument } => {
                let found = self.to_text(argument).into_owned();
                Raised::native_argument_not_a_double(position, &found)
            }
            Refused::NotPositive { position, argument } => {
                let found = self.to_text(argument).into_owned();
                Raised::native_argument_not_positive(position, &found)
            }
            Refused::TooManyArguments { expected } => Raised::too_many_external_arguments(expected),
            Refused::Signature | Refused::ResultSignature => {
                let number = refused
                    .error_number(method)
                    .expect("a signature refusal carries its number");
                Raised::incorrect_native_signature(number, refused == Refused::Signature)
            }
            Refused::ClassicStyle => {
                return Loud {
                    message: crate::owned_message(&format!("{refused}"), Some("Phase 10")),
                }
                .into();
            }
            Refused::Unfilled { .. }
            | Refused::UnfilledSlot { .. }
            | Refused::StaleHandle
            | Refused::Raised => {
                return Loud {
                    message: crate::owned_message(&format!("{refused}"), Some("Phase 8")),
                }
                .into();
            }
        };
        if !packaged {
            raised.delivery.lineless = false;
        }
        raised.into()
    }
}

/// The condition an extension raised, whose number is the major and the minor
/// packed as `major * 1000 + minor` (`api/oorexxerrors.h`).
fn condition_of(number: usize) -> Failure {
    let major = u16::try_from(number / 1000).unwrap_or(u16::MAX);
    let minor = u16::try_from(number % 1000).unwrap_or(0);
    Raised::syntax(major, minor, Vec::new()).into()
}

impl Host for Interp {
    fn is_method(&self) -> bool {
        self.native_handles.last().is_some_and(|frame| frame.method)
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

    fn numeric(&self) -> Numeric {
        let settings = &self.activation().settings;
        Numeric {
            digits: usize::try_from(settings.digits()).unwrap_or(usize::MAX),
            fuzz: usize::try_from(settings.fuzz()).unwrap_or(usize::MAX),
            engineering: settings.form() == rexx_num::Form::Engineering,
        }
    }

    fn double_value(&mut self, object: ObjRef) -> Result<Option<f64>, Condition> {
        if let Decoded::SmallInt(number) = object.decode() {
            #[expect(
                clippy::cast_precision_loss,
                reason = "`RexxInteger::doubleValue` is the same C conversion"
            )]
            return Ok(Some(number as f64));
        }
        let text = self.native_string_conversion(object)?;
        let bytes = self.to_text(text);
        if let Some(number) = Number::parse_bytes(&bytes) {
            return Ok(Some(double_of(&number)));
        }
        Ok(match &bytes[..] {
            b"nan" => Some(f64::NAN),
            b"+infinity" => Some(f64::INFINITY),
            b"-infinity" => Some(f64::NEG_INFINITY),
            _ => None,
        })
    }

    fn positive_whole_number(&mut self, object: ObjRef) -> Result<Option<isize>, Condition> {
        let value = match object.decode() {
            Decoded::SmallInt(number) => Some(number),
            _ => {
                let text = self.native_string_conversion(object)?;
                let bytes = self.to_text(text);
                Number::parse_bytes(&bytes).and_then(|number| number.whole_value(SIZE_DIGITS))
            }
        };
        Ok(value
            .filter(|number| (1..=MAX_WHOLENUMBER).contains(number))
            .and_then(|number| isize::try_from(number).ok()))
    }

    fn double_object(&mut self, value: f64, precision: usize) -> ObjRef {
        let object = self.text(&double_text(value, precision));
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

/// `Numerics::SIZE_DIGITS` (`interpreter/runtime/Numerics.hpp:92`), the
/// precision `objectToSignedInteger` converts at.
const SIZE_DIGITS: usize = 20;

/// `Numerics::MAX_WHOLENUMBER` (`interpreter/runtime/Numerics.hpp:86`).
const MAX_WHOLENUMBER: i64 = 999_999_999_999_999_999;

impl Interp {
    /// `requestString` for a native argument, holding a raised condition on
    /// the running native frame as [`Host::string_value`] does.
    fn native_string_conversion(&mut self, object: ObjRef) -> Result<ObjRef, Condition> {
        match self.required_string_value(object) {
            Ok(text) => {
                self.roots.push_temp(text);
                Ok(text)
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
}

/// A Rexx number as the C `double` `strtod` reads from its digits
/// (`NumberString::doubleValue`, `interpreter/classes/NumberStringClass.cpp:704`).
///
/// Rendered at as many digits as the number has, which keeps every digit.
/// `Number::format` takes the exponential form where the plain one would
/// pad the integer part with zeros or put more zeros after the point than
/// there are digits, so the text stays within about twice the digits.
fn double_of(number: &Number) -> f64 {
    let written = double_literal(number);
    written
        .parse()
        .unwrap_or_else(|_| unreachable!("a formatted Rexx number is a float literal: {written}"))
}

/// The float literal [`double_of`] reads.
fn double_literal(number: &Number) -> String {
    let digits = u64::try_from(number.digit_count()).unwrap_or(u64::MAX);
    number.format(digits)
}

/// `NumberString::newInstanceFromDouble(value, precision)`
/// (`interpreter/classes/NumberStringClass.cpp:4073`) as its string value:
/// `%.*g` at two digits past the precision, capped at sixteen, then rounded
/// to the precision and formatted at it.
fn double_text(value: f64, precision: usize) -> Vec<u8> {
    if value.is_nan() {
        return b"nan".to_vec();
    }
    if value == f64::INFINITY {
        return b"+infinity".to_vec();
    }
    if value == f64::NEG_INFINITY {
        return b"-infinity".to_vec();
    }
    let printed = percent_g(value, precision.min(16) + 2);
    let digits = u64::try_from(precision).unwrap_or(u64::MAX);
    Number::parse(&printed)
        .unwrap_or_else(|| unreachable!("%g prints a Rexx number: {printed}"))
        .into_round(digits)
        .format(digits)
        .into_bytes()
}

/// C's `%.*g` with `significant` digits: the style `%e` would take where its
/// exponent is below -4 or not below `significant`, the style `%f` takes
/// otherwise, and in either no trailing zero after the point.
fn percent_g(value: f64, significant: usize) -> String {
    let scientific = format!("{:.*e}", significant - 1, value);
    let (mantissa, exponent) = scientific
        .split_once('e')
        .unwrap_or_else(|| unreachable!("{{:e}} prints an exponent: {scientific}"));
    let exponent: i64 = exponent
        .parse()
        .unwrap_or_else(|_| unreachable!("{{:e}} prints a decimal exponent: {scientific}"));
    let (sign, mantissa) = match mantissa.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", mantissa),
    };
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    let width = i64::try_from(significant).unwrap_or(i64::MAX);
    if exponent < -4 || exponent >= width {
        let kept = digits.trim_end_matches('0');
        let kept = if kept.is_empty() { "0" } else { kept };
        let (first, rest) = kept.split_at(1);
        let point = if rest.is_empty() { "" } else { "." };
        return format!("{sign}{first}{point}{rest}e{exponent}");
    }
    if exponent >= 0 {
        let (whole, fraction) = digits.split_at(usize::try_from(exponent + 1).unwrap_or(0));
        let fraction = fraction.trim_end_matches('0');
        if fraction.is_empty() {
            return format!("{sign}{whole}");
        }
        return format!("{sign}{whole}.{fraction}");
    }
    let zeros = "0".repeat(usize::try_from(-exponent - 1).unwrap_or(0));
    format!("{sign}0.{zeros}{}", digits.trim_end_matches('0'))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::rc::Rc;

    use rexx_api::values::Failure as Refused;

    use rexx_num::Number;

    use super::{double_literal, double_of, double_text, percent_g, pool_variable_name};
    use crate::{Failure, Interp, LibraryLoad};

    /// This worktree's own `build/lib`, not the oracle checkout's: a second
    /// build of the extensions from the same sources, which the oracle never
    /// runs (D5's amendment, `docs/superpowers/specs/2026-09-14-phase-8-native-api.md`).
    /// Its libraries are loaded, never rebuilt.
    fn worktree_library_directory() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../build/lib")
            .canonicalize()
            .expect("the worktree's build/lib is three directories above this crate")
    }

    /// The oracle's `librxregexp.so`, opened by path.
    fn open_rxregexp() -> rexx_api::load::Library {
        rexx_api::load::open_path(
            &worktree_library_directory().join("librxregexp.so"),
            "rxregexp",
        )
        .expect("the extension asks for 4.0.0, which is below this interpreter")
        .expect("the extension publishes RexxGetPackage")
    }

    /// An interpreter started with an `LD_LIBRARY_PATH` naming that
    /// directory. The process variable is not touched: this is the
    /// interpreter's copy, which is the one the search reads.
    fn interp_that_can_see_rxregexp() -> Interp {
        let mut interp = Interp::new();
        interp.adopt_environment(vec![(
            b"LD_LIBRARY_PATH".to_vec(),
            worktree_library_directory()
                .into_os_string()
                .into_encoded_bytes(),
        )]);
        interp
    }

    /// **One resolution path, and the assertion is on the side effect rather
    /// than on the answer.** Two `Loaded` answers would look alike whether or
    /// not the second one re-opened the library, and so would two answers
    /// holding the same `Rc`, because a non-replacing hold hands the first one
    /// back; the attempt count is what says the `dlopen` and the package read
    /// happened once.
    #[test]
    fn a_library_named_twice_is_opened_once() {
        let mut interp = interp_that_can_see_rxregexp();
        let LibraryLoad::Loaded(first) = interp.resolve_library(b"rxregexp") else {
            panic!("librxregexp.so did not load from the worktree's build directory");
        };
        let LibraryLoad::Loaded(second) = interp.resolve_library(b"rxregexp") else {
            panic!("the second resolve did not load");
        };
        assert_eq!(
            interp.library_open_attempts, 1,
            "the same name was opened twice"
        );
        assert!(Rc::ptr_eq(&first, &second));

        // The control that says the count sees an attempt at all: a name
        // nothing is held for is looked for every time it is asked, whether or
        // not anything loads.
        assert!(matches!(
            interp.resolve_library(b"zorkolib"),
            LibraryLoad::Missing
        ));
        assert!(matches!(
            interp.resolve_library(b"zorkolib"),
            LibraryLoad::Missing
        ));
        assert_eq!(interp.library_open_attempts, 3);
    }

    /// The search directories come from the interpreter's own environment,
    /// which is what makes the name reachable at all: without the variable the
    /// same name answers nothing.
    #[test]
    fn the_search_path_is_what_makes_the_name_resolve() {
        let mut bare = Interp::new();
        bare.adopt_environment(Vec::new());
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

    /// The search is the one the interpreter started with: a directory written
    /// into its `LD_LIBRARY_PATH` afterwards, as `VALUE(..., 'ENVIRONMENT')`
    /// writes it, is not searched. The control is the same directory handed in
    /// at the start.
    #[test]
    fn a_search_directory_written_after_the_start_is_not_searched() {
        let directory = worktree_library_directory()
            .into_os_string()
            .into_encoded_bytes();
        let mut late = Interp::new();
        late.adopt_environment(Vec::new());
        late.env_set(b"LD_LIBRARY_PATH", Some(directory));
        assert!(matches!(
            late.resolve_library(b"rxregexp"),
            LibraryLoad::Missing
        ));
        let mut early = interp_that_can_see_rxregexp();
        assert!(matches!(
            early.resolve_library(b"rxregexp"),
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
        assert_eq!(
            interp.library_open_attempts, 0,
            "the held library was opened again"
        );
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
                worktree_library_directory()
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

    /// **The routine half answers the same under a collection at every
    /// allocation as under none**: a merged and a loaded routine, a double
    /// the extension builds, an argument refused after its string conversion,
    /// and a `loadExternalMethod` answer a class defines. The stdout is the
    /// oracle's, measured.
    #[test]
    fn a_library_routine_answers_the_same_under_a_collection_at_every_allocation() {
        let text = b"say 'merged' RxCalcSqrt(2) RxCalcPower(10, 10) RxCalcSqrt('nan')\n\
            signal on syntax\n\
            say RxCalcSqrt(.object~new)\n\
            syntax:\n\
            say 'refused' condition('O')~code\n\
            r = .Routine~loadExternalRoutine('r', 'LIBRARY rxmath RxCalcPi')\n\
            say 'loaded' r~call(12) r~callWith(.array~of(3))\n\
            say 'found' .context~package~findRoutine('RXCALCSQRT')~call(81)\n\
            .K~define('DOES', .Method~loadExternalMethod('m', 'LIBRARY rxregexp RegExp_Match'))\n\
            .K~define('INIT', .Method~loadExternalMethod('i', 'LIBRARY rxregexp RegExp_Init'))\n\
            say 'defined' .K~new('a*b')~does('aab')\n\
            ::requires 'rxmath' LIBRARY\n\
            ::class K\n"
            .to_vec();
        let invocation = || {
            crate::Invocation::none().with_environment(vec![(
                b"LD_LIBRARY_PATH".to_vec(),
                worktree_library_directory()
                    .into_os_string()
                    .into_encoded_bytes(),
            )])
        };
        let name = "/tmp/routine_stress.rex";

        let plain = crate::run_program(name, text.clone(), invocation());
        assert_eq!(
            plain.exit_code,
            0,
            "{}",
            String::from_utf8_lossy(&plain.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&plain.stdout),
            "merged 1.41421356 1.00000000E+10 nan\nrefused 88.921\nloaded 3.14159265359 3.14\n\
             found 9\ndefined 1\n"
        );

        let swept = crate::run_program_collect_every_alloc(name, text, invocation());
        assert_eq!(swept.exit_code, plain.exit_code);
        assert_eq!(swept.stdout, plain.stdout);
        assert_eq!(swept.stderr, plain.stderr);
        assert!(swept.collections > 0, "the stress mode did not collect");
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
            panic!("librxregexp.so did not load from the worktree's build directory");
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
                worktree_library_directory()
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
        let mut interp = Interp::new();
        let mut lineless =
            |refused: Refused, packaged: bool| match interp.refusal(refused, packaged, true) {
                Failure::Raised(raised) => raised.delivery.lineless,
                _ => panic!("every condition-shaped refusal is a raise"),
            };
        assert!(lineless(Refused::MissingArgument { position: 1 }, true));
        assert!(lineless(Refused::NoStringValue { position: 1 }, true));
        assert!(lineless(Refused::TooManyArguments { expected: 0 }, true));
        assert!(lineless(Refused::Signature, true));
        assert!(!lineless(Refused::ResultSignature, true));
        // Measured, oracle rc 168: `RxCalcSqrt()` through a routine no
        // directive has bound is `Error 88 running <the caller> line 2`.
        assert!(!lineless(Refused::MissingArgument { position: 1 }, false));
        assert!(!lineless(Refused::TooManyArguments { expected: 2 }, false));
    }

    /// A signature refusal is 93.968 in a method and 40.918 in a routine.
    /// Measured through a forged routine library, oracle rc 216: a routine
    /// declaring `CSELF` is `Error 40.918:  Invalid native function signature
    /// specification.` with no line where a directive bound it, and a result
    /// word carrying the optional bit the same with the sending clause's line.
    #[test]
    fn a_signature_refusal_is_numbered_for_a_method_or_a_routine() {
        let mut interp = Interp::new();
        let mut numbered = |refused: Refused, method: bool| {
            let frame = crate::NativeFrame {
                owner: rexx_core::ObjRef::NIL,
                scope: rexx_core::ObjRef::NIL,
                method,
                locals: rexx_api::handles::Table::new(),
                raised: None,
            };
            match interp.settle_native_call(Err(refused), frame, true) {
                Err(Failure::Raised(raised)) => {
                    (raised.number, raised.sub, raised.delivery.lineless)
                }
                _ => panic!("a signature refusal is a raise"),
            }
        };
        assert_eq!(numbered(Refused::Signature, true), (93, 968, true));
        assert_eq!(numbered(Refused::Signature, false), (40, 918, true));
        assert_eq!(numbered(Refused::ResultSignature, true), (93, 968, false));
        assert_eq!(numbered(Refused::ResultSignature, false), (40, 918, false));
    }

    /// A number past either end of a double's range converts to what
    /// `strtod` answers without being written out digit by digit: measured,
    /// oracle, `RxCalcSqrt('1E+999999999')` is `+infinity` and
    /// `RxCalcSqrt('1E-999999999')` is `0` under a 1 GiB address-space cap.
    #[test]
    fn a_double_argument_is_read_from_its_digits_and_exponent() {
        let read = |text: &str| double_of(&Number::parse(text).expect("a Rexx number"));
        assert_eq!(read("1E+999999999"), f64::INFINITY);
        assert_eq!(read("-9.99E+999999999"), f64::NEG_INFINITY);
        assert_eq!(read("1E-999999999"), 0.0);
        assert_eq!(read("4.9E-324"), 4.9e-324);
        assert_eq!(read("1.7976931348623157E+308"), f64::MAX);
        assert_eq!(
            read("123456789012345678901234567890E-10"),
            1.2345678901234567e19
        );
        assert_eq!(read("0.000000000000000000000000000000000000001"), 1e-39);
        assert_eq!(read("2.25"), 2.25);
        assert_eq!(read("-16"), -16.0);
        let written = double_literal(&Number::parse("1E+999999999").expect("a Rexx number"));
        assert_eq!(written, "1E+999999999");
    }

    /// `%g`'s two styles and its trailing zeros, against what glibc prints.
    #[test]
    fn percent_g_prints_what_c_prints() {
        assert_eq!(percent_g(4.0, 11), "4");
        assert_eq!(percent_g(2f64.sqrt(), 11), "1.4142135624");
        assert_eq!(percent_g(1e10, 11), "10000000000");
        assert_eq!(percent_g(1e10, 5), "1e10");
        assert_eq!(percent_g(1e-5, 11), "1e-5");
        assert_eq!(percent_g(1e-4, 11), "0.0001");
        assert_eq!(percent_g(-2.5, 3), "-2.5");
        assert_eq!(percent_g(9.99, 2), "10");
        assert_eq!(percent_g(0.0, 11), "0");
    }

    /// Measured on the oracle through `rxmath`, which hands its precision
    /// straight to `DoubleToObjectWithPrecision`.
    #[test]
    fn a_double_renders_as_the_oracle_renders_it() {
        let rendered = |value: f64, precision: usize| {
            String::from_utf8(double_text(value, precision)).expect("ASCII")
        };
        assert_eq!(rendered(4.0, 9), "4");
        assert_eq!(rendered(2f64.sqrt(), 9), "1.41421356");
        assert_eq!(rendered(2f64.sqrt(), 3), "1.41");
        assert_eq!(rendered(2f64.sqrt(), 16), "1.414213562373095");
        assert_eq!(rendered(1e10, 9), "1.00000000E+10");
        assert_eq!(rendered(1e10, 3), "1E+10");
        assert_eq!(rendered(1e9, 9), "1.00000000E+9");
        assert_eq!(rendered(1e8, 9), "100000000");
        assert_eq!(rendered(1e-5, 9), "0.00001");
        assert_eq!(rendered(9.99, 2), "10");
        assert_eq!(rendered(123_456_789.0, 5), "1.2346E+8");
        assert_eq!(rendered(f64::NAN, 9), "nan");
        assert_eq!(rendered(f64::INFINITY, 9), "+infinity");
        assert_eq!(rendered(0.0, 9), "0");
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
