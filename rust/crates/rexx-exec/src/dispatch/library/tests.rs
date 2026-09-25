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
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/lang/library_method_external.rex");
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

/// **The conversion rows answer the same under a collection at every
/// allocation as under none**: integers at their ends, a float, a double,
/// a logical, a `CSTRING` both ways, the `size_t` result, an array made by
/// conversion, a class, pointers, stems by object and by name, the special
/// arguments, and a refusal. The stdout is the oracle's, measured. It reads
/// a refusal's `code` and not its `message`: `signal on syntax`, `y = 1/0`,
/// then `say condition('O')~message` in the syntax handler panics in this
/// mode with no native call, at the assertion `collect_stress`'s L0 test
/// fails at.
#[test]
fn the_conversion_rows_answer_the_same_under_a_collection_at_every_allocation() {
    let text = b"t = .T~new\n\
        say 'int' t~int8(-128) t~uint64('18446744073709551615') t~int64('-9223372036854775808') t~size('1E19')\n\
        say 'num' t~float(1.1) t~double(2/3) t~logical(1) t~cstring('abc') t~version\n\
        say 'obj' t~array('abc')~items t~array(.list~of(1, 2))~items t~classarg(.string) t~object(t)~class\n\
        say 'ptr' t~pointerarg(t~pointervalue) t~nullpointerstring t~pointerstringarg(t~pointerstringvalue)\n\
        y.7 = 'seven'\n\
        say 'stem' TestStemArg('y')[7] t~stem(y.)[7]\n\
        drop qq.\n\
        say 'unset' TestStemArg('qq')~class symbol('QQ.')\n\
        al = t~arglist(1, , 3)\n\
        say 'special' al~size al~items (t~oself == t) t~scope t~super t~name TestNameArg() TestArglistArg(1, 2)~items\n\
        say 'rxmath ['MathLoadFuncs()']'\n\
        signal on syntax\n\
        say t~int8(128)\n\
        exit\n\
        syntax:\n\
        say 'refused' condition('O')~code\n\
        ::requires 'orxfunction' LIBRARY\n\
        ::requires 'rxmath' LIBRARY\n\
        ::class Base\n\
        ::class T subclass Base\n\
        ::method int8 external \"LIBRARY orxmethod TestInt8Arg\"\n\
        ::method uint64 external \"LIBRARY orxmethod TestUint64Arg\"\n\
        ::method int64 external \"LIBRARY orxmethod TestInt64Arg\"\n\
        ::method size external \"LIBRARY orxmethod TestSizeArg\"\n\
        ::method float external \"LIBRARY orxmethod TestFloatArg\"\n\
        ::method double external \"LIBRARY orxmethod TestDoubleArg\"\n\
        ::method logical external \"LIBRARY orxmethod TestLogicalArg\"\n\
        ::method cstring external \"LIBRARY orxmethod TestCstringArg\"\n\
        ::method version external \"LIBRARY orxmethod TestInterpreterVersion\"\n\
        ::method array external \"LIBRARY orxmethod TestArrayArg\"\n\
        ::method classarg external \"LIBRARY orxmethod TestClassArg\"\n\
        ::method object external \"LIBRARY orxmethod TestObjectArg\"\n\
        ::method pointervalue external \"LIBRARY orxmethod TestPointerValue\"\n\
        ::method pointerarg external \"LIBRARY orxmethod TestPointerArg\"\n\
        ::method pointerstringvalue external \"LIBRARY orxmethod TestPointerStringValue\"\n\
        ::method pointerstringarg external \"LIBRARY orxmethod TestPointerStringArg\"\n\
        ::method nullpointerstring external \"LIBRARY orxmethod TestNullPointerStringValue\"\n\
        ::method stem external \"LIBRARY orxmethod TestStemArg\"\n\
        ::method arglist external \"LIBRARY orxmethod TestArglistArg\"\n\
        ::method oself external \"LIBRARY orxmethod TestOSelfArg\"\n\
        ::method scope external \"LIBRARY orxmethod TestScopeArg\"\n\
        ::method super external \"LIBRARY orxmethod TestSuperArg\"\n\
        ::method name external \"LIBRARY orxmethod TestNameArg\"\n\
        "
    .to_vec();
    let invocation = || {
        crate::Invocation::none().with_environment(vec![(
            b"LD_LIBRARY_PATH".to_vec(),
            worktree_library_directory()
                .into_os_string()
                .into_encoded_bytes(),
        )])
    };
    let name = "/tmp/conversion_stress.rex";

    let plain = crate::run_program(name, text.clone(), invocation());
    assert_eq!(
        plain.exit_code,
        0,
        "{}",
        String::from_utf8_lossy(&plain.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&plain.stdout),
        "int -128 18446744073709551615 -9223372036854775808 10000000000000000000\n\
         num 1.10000002 0.666666667 1 abc 328448\n\
         obj 1 2 The String class The T class\n\
         ptr 1 0x0 1\n\
         stem seven seven\n\
         unset The Stem class VAR\n\
         special 3 2 1 The T class The BASE class NAME TESTNAMEARG 2\n\
         rxmath []\n\
         refused 88.907\n"
    );

    let swept = crate::run_program_collect_every_alloc(name, text, invocation());
    assert_eq!(swept.exit_code, plain.exit_code);
    assert_eq!(swept.stdout, plain.stdout);
    assert_eq!(swept.stderr, plain.stderr);
    assert!(swept.collections > 0, "the stress mode did not collect");
}

/// A native call's frame is reused by the next call with nothing of the
/// last one left in it: a handle the last call registered resolves to
/// nothing, and its arguments, argument array, name and condition are
/// gone.
#[test]
fn a_reused_native_frame_holds_nothing_of_the_call_before() {
    let mut interp = Interp::new();
    let object = interp.text(b"held by the first call");
    interp.push_native_frame(
        rexx_core::ObjRef::NIL,
        rexx_core::ObjRef::NIL,
        Some(object),
        b"FIRST",
        &[Some(object), None],
    );
    let handle = interp.native_frame_mut().locals.register(object);
    interp.native_frame_mut().argument_list = Some(object);
    interp.native_frame_mut().raised = Some(crate::Loud::library_procedure_gone().into());
    let (raised, method) = interp.pop_native_frame();
    assert!(raised.is_some() && method);

    interp.push_native_frame(
        rexx_core::ObjRef::NIL,
        rexx_core::ObjRef::NIL,
        None,
        b"SECOND",
        &[],
    );
    assert_eq!(
        interp.native_spares.len(),
        0,
        "the spare frame was not reused"
    );
    let frame = interp.native_frame_mut();
    assert_eq!(frame.locals.resolve(handle), None);
    assert_eq!(frame.name, b"SECOND");
    assert!(frame.arguments.is_empty());
    assert_eq!(frame.argument_list, None);
    assert!(frame.raised.is_none());
    assert!(!frame.method);
    assert_eq!(frame.receiver, rexx_core::ObjRef::NIL);
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
    let mut numbered = |refused: Refused, method: bool| match interp.settle_native_call(
        Err(refused),
        None,
        method,
        true,
    ) {
        Err(Failure::Raised(raised)) => (raised.number, raised.sub, raised.delivery.lineless),
        _ => panic!("a signature refusal is a raise"),
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

/// The cases [`unloaders_run_in_the_oracles_table_order`] reads.
const UNLOAD_ORDER_CASES: &str = "tests/unload_order_cases";

/// The order an interpreter holds its libraries in for termination after the
/// loads in `input`, one name per line, `missing` before a name that loaded
/// nothing. Every name that loads is the same shared object.
fn unload_order(input: &str) -> String {
    let mut interp = Interp::new();
    for line in input.lines() {
        match line.strip_prefix("missing ") {
            Some(name) => {
                assert!(matches!(
                    interp.settle_library(name.as_bytes(), Ok(None)),
                    LibraryLoad::Missing
                ));
            }
            None => {
                assert!(matches!(
                    interp.settle_library(line.as_bytes(), Ok(Some(open_rxregexp()))),
                    LibraryLoad::Loaded(_)
                ));
            }
        }
    }
    let order: Vec<String> = interp
        .libraries
        .in_unload_order()
        .into_iter()
        .map(|(name, _)| String::from_utf8(name).expect("a case's names are ASCII"))
        .collect();
    format!("{}\n", order.join(" "))
}

#[test]
fn unloaders_run_in_the_oracles_table_order() {
    let mut cases = 0;
    datadriven::walk(UNLOAD_ORDER_CASES, |file| {
        file.run(|case| {
            cases += 1;
            assert_eq!(
                case.directive, "loads",
                "unknown directive {:?}",
                case.directive
            );
            unload_order(&case.input)
        });
    });
    assert!(cases > 0, "{UNLOAD_ORDER_CASES} ran no case");
}

thread_local! {
    /// Which recording hook ran, in order, and whether it was handed a
    /// thread context.
    static HOOKS_RAN: std::cell::RefCell<Vec<(&'static str, bool)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

extern "C" fn recording_loader(thread: *mut rexx_api::layout::RexxThreadContext_) {
    HOOKS_RAN.with(|ran| ran.borrow_mut().push(("loader", !thread.is_null())));
}

extern "C" fn recording_unloader(thread: *mut rexx_api::layout::RexxThreadContext_) {
    HOOKS_RAN.with(|ran| ran.borrow_mut().push(("unloader", !thread.is_null())));
}

/// **The interpreter runs a library's loader when the library loads and its
/// unloader when it terminates**, each handed a thread context, through the
/// same paths an opened shared object takes. No library this tree may load
/// declares a hook that does anything observable, so the entry is in memory.
#[test]
fn a_loaded_librarys_loader_and_unloader_run() {
    HOOKS_RAN.with(|ran| ran.borrow_mut().clear());
    let mut interp = Interp::new();
    let library = rexx_api::load::hooks_only(Some(recording_loader), Some(recording_unloader));
    assert!(matches!(
        interp.settle_library(b"hooked", Ok(Some(library))),
        LibraryLoad::Loaded(_)
    ));
    assert_eq!(
        HOOKS_RAN.with(|ran| ran.borrow().clone()),
        vec![("loader", true)],
        "loading ran no loader"
    );
    assert!(interp.terminate().is_empty());
    assert_eq!(
        HOOKS_RAN.with(|ran| ran.borrow().clone()),
        vec![("loader", true), ("unloader", true)],
        "terminating ran no unloader"
    );
    let LibraryLoad::Loaded(held) = interp.resolve_library(b"hooked") else {
        panic!("the library is held");
    };
    assert!(!held.is_open(), "terminating left the library open");
}

/// Which of [`program_loader`] and [`program_unloader`] ran, in order, on
/// whichever thread ran them.
static PROGRAM_HOOKS: std::sync::Mutex<Vec<&'static str>> = std::sync::Mutex::new(Vec::new());

extern "C" fn program_loader(_thread: *mut rexx_api::layout::RexxThreadContext_) {
    PROGRAM_HOOKS.lock().expect("unpoisoned").push("loader");
}

extern "C" fn program_unloader(_thread: *mut rexx_api::layout::RexxThreadContext_) {
    PROGRAM_HOOKS.lock().expect("unpoisoned").push("unloader");
}

/// **A program's end runs the unloader of a library it required**, through
/// the whole run, not only through [`Interp::terminate`].
#[test]
fn a_programs_end_runs_its_librarys_unloader() {
    PROGRAM_HOOKS.lock().expect("unpoisoned").clear();
    let outcome = crate::on_interpreter_thread(|| {
        crate::install::offer_library(b"hooked", || {
            rexx_api::load::hooks_only(Some(program_loader), Some(program_unloader))
        });
        crate::execute(
            "hooked.rex",
            b"say 'main'\n::requires 'hooked' LIBRARY\n".to_vec(),
            false,
            crate::Invocation::none(),
        )
    });
    assert_eq!(
        (
            outcome.exit_code,
            String::from_utf8_lossy(&outcome.stdout).into_owned()
        ),
        (0, "main\n".to_owned()),
        "{}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(
        *PROGRAM_HOOKS.lock().expect("unpoisoned"),
        vec!["loader", "unloader"]
    );
}
