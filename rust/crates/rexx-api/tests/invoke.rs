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

//! The two-call protocol, run against the oracle's own compiled extension.

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};

use rexx_api::ffi::Contexts;
use rexx_api::handles::Table;
use rexx_api::invoke;
use rexx_api::layout::{POINTER, wholenumber_t};
use rexx_api::load::{self, NativeMethodEntry, NativeRoutineEntry};
use rexx_api::values::{
    Activation, CStringPool, Constants, Conversion, Failure, Host, Numeric, OPTIONAL_ARGUMENT,
    Raised, code,
};
use rexx_core::{BehaviourHandle, Body, Bytes, Heap, ObjRef};

/// `Rexx_Error_Invalid_template` (`api/oorexxerrors.h:362`), which
/// `RegExp_Init` raises for an expression it cannot parse
/// (`extensions/rxregexp/rxregexp.cpp:83`).
const INVALID_TEMPLATE: usize = 38000;

/// `Rexx_Error_Incorrect_method` (`api/oorexxerrors.h:518`), which
/// `RegExp_Parse` raises for a match type it does not know
/// (`extensions/rxregexp/rxregexp.cpp:130`).
const INCORRECT_METHOD: usize = 93000;

/// The repository root, which is three directories above this crate.
fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root is three directories above this crate")
}

/// The oracle's own build of the `rxregexp` extension, which D5's amendment
/// makes the instrument: it is loaded, never rebuilt.
fn rxregexp() -> load::Library {
    let path = repository_root().join("build/lib/librxregexp.so");
    load::open_path(&path, "rxregexp")
        .expect("the extension asks for 4.0.0, which is below this interpreter")
        .expect("the extension publishes RexxGetPackage")
}

/// The oracle's own build of the `rxmath` extension, whose routines are all
/// typed (`extensions/rxmath/rxmath.cpp:617-638`).
fn rxmath() -> load::Library {
    let path = repository_root().join("build/lib/librxmath.so");
    load::open_path(&path, "rxmath")
        .expect("the extension asks for 4.0.0, which is below this interpreter")
        .expect("the extension publishes RexxGetPackage")
}

// ------------------------------------------- what the extension called back

thread_local! {
    /// The pointers handed to `NewPointer`, in order.
    static POINTERS: RefCell<Vec<POINTER>> = const { RefCell::new(Vec::new()) };
    /// The condition each call left pending, in order.
    static RAISED: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
    /// The numbers handed to `WholeNumberToObject`, in order.
    static NUMBERS: RefCell<Vec<wholenumber_t>> = const { RefCell::new(Vec::new()) };
    static VARIABLES_SET: Cell<usize> = const { Cell::new(0) };
    static VARIABLES_DROPPED: Cell<usize> = const { Cell::new(0) };
}

fn forget_callbacks() {
    POINTERS.with(|seen| seen.borrow_mut().clear());
    RAISED.with(|seen| seen.borrow_mut().clear());
    NUMBERS.with(|seen| seen.borrow_mut().clear());
    VARIABLES_SET.set(0);
    VARIABLES_DROPPED.set(0);
}

// --------------------------------------------------- the interpreter's side

/// Stands in for the interpreter: text objects in a heap, and whatever
/// `CSELF` the receiver last stored.
struct Interpreter {
    heap: Heap,
    cself: Option<POINTER>,
    variables: Vec<(Vec<u8>, ObjRef)>,
    locals: Table,
    digits: usize,
    /// What `DoubleToObjectWithPrecision` was asked for, in order.
    doubles: Vec<(f64, usize)>,
}

impl Interpreter {
    fn new() -> Interpreter {
        Interpreter {
            heap: Heap::new(),
            cself: None,
            variables: Vec::new(),
            locals: Table::new(),
            digits: 9,
            doubles: Vec::new(),
        }
    }

    fn text(&mut self, bytes: &[u8]) -> ObjRef {
        self.heap.alloc(Body::Text {
            bytes: Bytes::from_slice(bytes),
            num: None,
        })
    }
}

impl Host for Interpreter {
    fn is_method(&self) -> bool {
        true
    }

    fn string_value(&mut self, object: ObjRef) -> Result<Option<ObjRef>, Raised> {
        Ok(Some(object))
    }

    fn string_bytes(&self, object: ObjRef) -> Option<Cow<'_, [u8]>> {
        match &self.heap.get(object)?.body {
            Body::Text { bytes, .. } => Some(Cow::Borrowed(bytes.as_slice())),
            _ => None,
        }
    }

    fn cself(&mut self) -> Option<POINTER> {
        self.cself
    }

    fn constants(&mut self) -> Constants<ObjRef> {
        Constants {
            nil: ObjRef::NIL,
            true_object: ObjRef::small_int(1).expect("one is a small integer"),
            false_object: ObjRef::small_int(0).expect("zero is a small integer"),
            null_string: self.text(b""),
        }
    }

    fn set_object_variable(&mut self, name: &[u8], value: Option<ObjRef>) {
        VARIABLES_SET.set(VARIABLES_SET.get() + 1);
        let name = name.to_ascii_uppercase();
        self.variables.retain(|(bound, _)| *bound != name);
        if let Some(value) = value {
            self.variables.push((name, value));
        }
    }

    fn drop_object_variable(&mut self, name: &[u8]) {
        VARIABLES_DROPPED.set(VARIABLES_DROPPED.get() + 1);
        let name = name.to_ascii_uppercase();
        self.variables.retain(|(bound, _)| *bound != name);
    }

    fn whole_number(&mut self, value: isize) -> ObjRef {
        NUMBERS.with(|seen| seen.borrow_mut().push(value as wholenumber_t));
        match i64::try_from(value).ok().and_then(ObjRef::small_int) {
            Some(object) => object,
            None => self.text(value.to_string().as_bytes()),
        }
    }

    fn new_pointer(&mut self, value: POINTER) -> ObjRef {
        POINTERS.with(|seen| seen.borrow_mut().push(value));
        let body = Body::pointer(ObjRef::NIL, BehaviourHandle::new(0), value);
        self.heap.alloc(body)
    }

    fn numeric(&self) -> Numeric {
        Numeric {
            digits: self.digits,
            fuzz: 0,
            engineering: false,
        }
    }

    fn double_value(&mut self, object: ObjRef) -> Result<Option<f64>, Raised> {
        let text = self.string_bytes(object).map(Cow::into_owned);
        Ok(text.and_then(|bytes| String::from_utf8(bytes).ok()?.parse().ok()))
    }

    fn positive_whole_number(&mut self, object: ObjRef) -> Result<Option<isize>, Raised> {
        let text = self.string_bytes(object).map(Cow::into_owned);
        Ok(text
            .and_then(|bytes| String::from_utf8(bytes).ok()?.parse().ok())
            .filter(|number| *number >= 1))
    }

    fn double_object(&mut self, value: f64, precision: usize) -> ObjRef {
        self.doubles.push((value, precision));
        self.text(format!("{value}").as_bytes())
    }

    fn locals(&mut self) -> &mut Table {
        &mut self.locals
    }
}

/// One interpreter's conversion state, which each call runs through the
/// contexts a `Contexts` hands out.
struct Session {
    interpreter: Interpreter,
    strings: CStringPool,
}

impl Session {
    fn new() -> Session {
        forget_callbacks();
        Session {
            interpreter: Interpreter::new(),
            strings: CStringPool::new(),
        }
    }

    fn text(&mut self, bytes: &[u8]) -> Option<ObjRef> {
        Some(self.interpreter.text(bytes))
    }

    /// Runs `entry`, recording the condition it left pending.
    fn call(
        &mut self,
        entry: &NativeMethodEntry,
        arguments: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        let activation = Activation::new(Conversion {
            host: &mut self.interpreter,
            strings: &mut self.strings,
        });
        let mut contexts = Contexts::new(&activation);
        let outcome = invoke::method(entry, &contexts.method(), &activation, arguments);
        if let Some(number) = activation.pending() {
            RAISED.with(|seen| seen.borrow_mut().push(number));
        }
        outcome
    }

    /// Runs the routine `entry` through a call context.
    fn call_routine(
        &mut self,
        entry: &NativeRoutineEntry,
        arguments: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        let activation = Activation::new(Conversion {
            host: &mut self.interpreter,
            strings: &mut self.strings,
        });
        let mut contexts = Contexts::new(&activation);
        invoke::routine(entry, &contexts.call(), &activation, arguments)
    }

    fn signature(&mut self, entry: &NativeMethodEntry) -> Result<Vec<u16>, Failure> {
        let activation = Activation::new(Conversion {
            host: &mut self.interpreter,
            strings: &mut self.strings,
        });
        let mut contexts = Contexts::new(&activation);
        invoke::signature(entry, &contexts.method())
    }
}

/// The small integer an `int` result of `value` converts back to.
fn returned(value: i64) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(ObjRef::small_int(value).expect("a small integer")))
}

/// The pointer the extension asked us to wrap, which is the automaton it
/// allocated.
fn allocated() -> POINTER {
    POINTERS.with(|seen| {
        let seen = seen.borrow();
        assert_eq!(seen.len(), 1, "NewPointer was not called exactly once");
        seen[0]
    })
}

// ------------------------------------------------------------- the protocol

/// The signature call answers what `extensions/rxregexp/rxregexp.cpp:55` and
/// `:105-109` declare, read out of the oracle's own compiled binary.
#[test]
fn the_extension_publishes_the_signatures_its_source_declares() {
    let library = rxregexp();
    let init = library.method(b"RegExp_Init").expect("RegExp_Init");
    let parse = library.method(b"RegExp_Parse").expect("RegExp_Parse");
    let mut session = Session::new();
    assert_eq!(
        session.signature(init),
        Ok(vec![
            code::INT,
            OPTIONAL_ARGUMENT | code::CSTRING,
            OPTIONAL_ARGUMENT | code::CSTRING,
        ])
    );
    assert_eq!(
        session.signature(parse),
        Ok(vec![
            code::INT,
            code::CSELF,
            code::CSTRING,
            OPTIONAL_ARGUMENT | code::CSTRING,
        ])
    );
}

#[test]
fn regexp_init_with_no_arguments_answers_zero_and_allocates_an_automaton() {
    let library = rxregexp();
    let init = library.method(b"RegExp_Init").expect("RegExp_Init");
    let uninit = library.method(b"RegExp_Uninit").expect("RegExp_Uninit");
    let mut session = Session::new();

    assert_eq!(session.call(init, &[]), returned(0));
    let automaton = allocated();
    assert!(!automaton.is_null(), "the extension allocated nothing");
    assert_eq!(VARIABLES_SET.get(), 1, "CSELF was not stored");
    assert!(RAISED.with(|seen| seen.borrow().is_empty()));

    // Hand the pointer back as `CSELF` so that the automaton is freed.
    session.interpreter.cself = Some(automaton);
    assert_eq!(session.call(uninit, &[]), returned(0));
    assert_eq!(VARIABLES_DROPPED.get(), 1);
}

#[test]
fn regexp_init_with_one_argument_parses_it() {
    let library = rxregexp();
    let init = library.method(b"RegExp_Init").expect("RegExp_Init");
    let uninit = library.method(b"RegExp_Uninit").expect("RegExp_Uninit");
    let mut session = Session::new();
    let expression = session.text(b"a*b");

    assert_eq!(session.call(init, &[expression]), returned(0));
    assert!(
        RAISED.with(|seen| seen.borrow().is_empty()),
        "`a*b` parses, so nothing is raised"
    );
    session.interpreter.cself = Some(allocated());
    assert_eq!(session.call(uninit, &[]), returned(0));
}

/// A native method that raises runs to completion: the work after the raise
/// still happens (`interpreter/api/ThreadContextStubs.cpp:1863-1885`).
///
/// `RegExp_Init` answers zero on every path, so this says nothing about
/// element zero; `a_call_that_raised_still_returns_what_it_wrote` does.
#[test]
fn an_extension_that_raises_still_finishes() {
    let library = rxregexp();
    let init = library.method(b"RegExp_Init").expect("RegExp_Init");
    let uninit = library.method(b"RegExp_Uninit").expect("RegExp_Uninit");
    let mut session = Session::new();
    let expression = session.text(b"[");

    assert_eq!(session.call(init, &[expression]), returned(0));
    assert_eq!(
        RAISED.with(|seen| seen.borrow().clone()),
        vec![INVALID_TEMPLATE]
    );
    assert_eq!(VARIABLES_SET.get(), 1, "the work after the raise still ran");
    session.interpreter.cself = Some(allocated());
    assert_eq!(session.call(uninit, &[]), returned(0));
}

/// A call that raised still writes element zero, and the value is read back
/// whatever the raise did. `RegExp_Parse` raises for an unknown match type
/// (`extensions/rxregexp/rxregexp.cpp:130`) and then returns what the parse
/// answered (`:135`), which is not the zero the descriptor started at.
#[test]
fn a_call_that_raised_still_returns_what_it_wrote() {
    let library = rxregexp();
    let init = library.method(b"RegExp_Init").expect("RegExp_Init");
    let parse = library.method(b"RegExp_Parse").expect("RegExp_Parse");
    let uninit = library.method(b"RegExp_Uninit").expect("RegExp_Uninit");
    let mut session = Session::new();
    let bad = session.text(b"[");
    let bogus = session.text(b"BOGUS");

    assert_eq!(session.call(init, &[]), returned(0));
    session.interpreter.cself = Some(allocated());

    assert_eq!(session.call(parse, &[bad, bogus]), returned(3));
    assert_eq!(
        RAISED.with(|seen| seen.borrow().clone()),
        vec![INCORRECT_METHOD]
    );

    assert_eq!(session.call(uninit, &[]), returned(0));
}

/// The whole chain: a `CSELF` the first call produced, a required `CSTRING`,
/// an optional one left out, and a non-zero result the extension wrote into
/// element zero.
#[test]
fn a_second_method_reads_the_cself_the_first_produced() {
    let library = rxregexp();
    let init = library.method(b"RegExp_Init").expect("RegExp_Init");
    let parse = library.method(b"RegExp_Parse").expect("RegExp_Parse");
    let uninit = library.method(b"RegExp_Uninit").expect("RegExp_Uninit");
    let mut session = Session::new();
    let good = session.text(b"a*b");
    let bad = session.text(b"[");
    let minimal = session.text(b"MINIMAL");

    assert_eq!(session.call(init, &[]), returned(0));
    session.interpreter.cself = Some(allocated());

    assert_eq!(session.call(parse, &[good]), returned(0));
    assert_eq!(
        session.call(parse, &[bad, minimal]),
        returned(3),
        "the oracle answers 3 for this template"
    );
    assert_eq!(
        NUMBERS.with(|seen| seen.borrow().len()),
        2,
        "each parse stores !POS"
    );

    assert_eq!(session.call(uninit, &[]), returned(0));
}

/// A required argument nobody supplied is refused before the extension is
/// called a second time.
#[test]
fn a_missing_required_argument_stops_the_call() {
    let library = rxregexp();
    let init = library.method(b"RegExp_Init").expect("RegExp_Init");
    let parse = library.method(b"RegExp_Parse").expect("RegExp_Parse");
    let uninit = library.method(b"RegExp_Uninit").expect("RegExp_Uninit");
    let mut session = Session::new();

    assert_eq!(session.call(init, &[]), returned(0));
    session.interpreter.cself = Some(allocated());

    let outcome = session.call(parse, &[]);
    assert_eq!(outcome, Err(Failure::MissingArgument { position: 1 }));
    assert_eq!(
        outcome.unwrap_err().error_number(true),
        Some(88901),
        "measured against the oracle for `~parse()`"
    );
    assert_eq!(
        NUMBERS.with(|seen| seen.borrow().len()),
        0,
        "`RegExp_Parse` stores !POS, so nothing of it ran"
    );

    assert_eq!(session.call(uninit, &[]), returned(0));
}

/// An argument the signature does not consume is 88.922, measured against the
/// oracle as `.RegularExpression~new('a*','MAXIMAL','extra')`.
#[test]
fn an_argument_the_signature_does_not_consume_is_refused() {
    let library = rxregexp();
    let init = library.method(b"RegExp_Init").expect("RegExp_Init");
    let mut session = Session::new();
    let expression = session.text(b"a*");
    let matchtype = session.text(b"MAXIMAL");
    let extra = session.text(b"extra");

    let outcome = session.call(init, &[expression, matchtype, extra]);
    assert_eq!(outcome, Err(Failure::TooManyArguments { expected: 2 }));
    assert_eq!(outcome.unwrap_err().error_number(true), Some(88922));
    assert_eq!(
        VARIABLES_SET.get(),
        0,
        "the extension must not have run at all"
    );
}

// ------------------------------------------------------------ the routines

/// `RxCalcSqrt` over the routine protocol: no precision argument, so the
/// extension asks the call context for the digits
/// (`extensions/rxmath/rxmath.cpp:118-123`) and formats the root at them.
#[test]
fn rxcalcsqrt_formats_at_the_callers_digits_when_no_precision_is_given() {
    let library = rxmath();
    let sqrt = library.routine(b"RxCalcSqrt").expect("RxCalcSqrt");
    let mut session = Session::new();
    session.interpreter.digits = 12;
    let sixteen = session.text(b"16");

    session
        .call_routine(sqrt, &[sixteen])
        .expect("the routine answers");
    assert_eq!(session.interpreter.doubles, vec![(4.0, 12)]);
}

/// A precision argument that exists wins over the digits, which is
/// `argumentExists(2)` read through the array the context publishes, and the
/// extension caps it at 16 (`extensions/rxmath/rxmath.cpp:124-127`).
#[test]
fn rxcalcsqrt_reads_a_precision_argument_that_exists() {
    let library = rxmath();
    let sqrt = library.routine(b"RxCalcSqrt").expect("RxCalcSqrt");
    let mut session = Session::new();
    let two = session.text(b"2");
    let three = session.text(b"3");
    let twenty = session.text(b"20");

    session
        .call_routine(sqrt, &[two, three])
        .expect("a precision of three");
    session
        .call_routine(sqrt, &[two, twenty])
        .expect("a precision of twenty");
    assert_eq!(
        session.interpreter.doubles,
        vec![(2f64.sqrt(), 3), (2f64.sqrt(), 16)]
    );
}

/// The argument errors are refused before the extension runs, measured
/// against the oracle as 88.901 and 88.922.
#[test]
fn rxcalcsqrt_refuses_a_missing_and_an_extra_argument_before_running() {
    let library = rxmath();
    let sqrt = library.routine(b"RxCalcSqrt").expect("RxCalcSqrt");
    let mut session = Session::new();
    let one = session.text(b"1");

    assert_eq!(
        session.call_routine(sqrt, &[]),
        Err(Failure::MissingArgument { position: 1 })
    );
    assert_eq!(
        session.call_routine(sqrt, &[one, one, one]),
        Err(Failure::TooManyArguments { expected: 2 })
    );
    assert_eq!(session.interpreter.doubles, Vec::new());
}

/// Every row of `rxmath`'s routine table is typed, so none is refused as
/// classic.
#[test]
fn every_rxmath_routine_is_typed() {
    let library = rxmath();
    assert!(!library.routines().is_empty());
    for row in library.routines() {
        assert_eq!(
            row.style,
            load::ROUTINE_TYPED_STYLE,
            "{}",
            String::from_utf8_lossy(&row.name)
        );
    }
}

/// The descriptor array is what bounds a signature, and this is the length
/// the oracle passes (`interpreter/execution/NativeActivation.hpp:209`).
#[test]
fn the_array_is_as_long_as_the_oracles() {
    assert_eq!(invoke::MAX_NATIVE_ARGUMENTS, 16);
    let header = std::fs::read_to_string(
        repository_root().join("interpreter/execution/NativeActivation.hpp"),
    )
    .expect("the oracle's header is readable");
    let declared = header
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("static const size_t MaxNativeArguments = ")?
                .strip_suffix(';')?
                .parse::<usize>()
                .ok()
        })
        .expect("MaxNativeArguments is declared in that header");
    assert_eq!(invoke::MAX_NATIVE_ARGUMENTS, declared);
}
