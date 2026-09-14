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

use rexx_api::handles::Table;
use rexx_api::invoke;
use rexx_api::layout::{
    CSTRING, MethodContextInterface, POINTER, RexxMethodContext_, RexxObjectPtr, RexxPointerObject,
    RexxThreadContext_, RexxThreadInterface, wholenumber_t,
};
use rexx_api::load::{self, NativeMethodEntry};
use rexx_api::values::{
    Activation, CStringPool, Constants, Conversion, Failure, Host, OPTIONAL_ARGUMENT, Raised, code,
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

// ------------------------------------------- what the extension called back

thread_local! {
    /// The pointers handed to `NewPointer`, in order.
    static POINTERS: RefCell<Vec<POINTER>> = const { RefCell::new(Vec::new()) };
    /// The error numbers handed to `RaiseException0`, in order.
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

/// An object handle the extension only ever hands back to us.
fn opaque<T>() -> *mut T {
    std::ptr::dangling_mut()
}

extern "C" fn new_pointer(_context: *mut RexxThreadContext_, value: POINTER) -> RexxPointerObject {
    POINTERS.with(|seen| seen.borrow_mut().push(value));
    opaque()
}

extern "C" fn whole_number_to_object(
    _context: *mut RexxThreadContext_,
    value: wholenumber_t,
) -> RexxObjectPtr {
    NUMBERS.with(|seen| seen.borrow_mut().push(value));
    opaque()
}

extern "C" fn raise_exception0(_context: *mut RexxThreadContext_, number: usize) {
    RAISED.with(|seen| seen.borrow_mut().push(number));
}

extern "C" fn set_object_variable(
    _context: *mut RexxMethodContext_,
    _name: CSTRING,
    _value: RexxObjectPtr,
) {
    VARIABLES_SET.set(VARIABLES_SET.get() + 1);
}

extern "C" fn drop_object_variable(_context: *mut RexxMethodContext_, _name: CSTRING) {
    VARIABLES_DROPPED.set(VARIABLES_DROPPED.get() + 1);
}

/// Run `body` with a method context whose tables answer the members
/// `rxregexp` calls and refuse the rest.
fn with_context<R>(body: impl FnOnce(&mut RexxMethodContext_) -> R) -> R {
    forget_callbacks();
    let mut thread_table = RexxThreadInterface::REFUSING;
    thread_table.NewPointer = new_pointer;
    thread_table.WholeNumberToObject = whole_number_to_object;
    thread_table.RaiseException0 = raise_exception0;
    let mut thread = RexxThreadContext_ {
        instance: std::ptr::null_mut(),
        functions: &raw mut thread_table,
    };
    let mut method_table = MethodContextInterface::REFUSING;
    method_table.SetObjectVariable = set_object_variable;
    method_table.DropObjectVariable = drop_object_variable;
    let mut context = RexxMethodContext_ {
        threadContext: &raw mut thread,
        functions: &raw mut method_table,
        arguments: std::ptr::null_mut(),
    };
    body(&mut context)
}

// --------------------------------------------------- the interpreter's side

/// Stands in for the interpreter: text objects in a heap, and whatever
/// `CSELF` the receiver last stored.
struct Interpreter {
    heap: Heap,
    cself: Option<POINTER>,
    variables: Vec<(Vec<u8>, ObjRef)>,
    locals: Table,
}

impl Interpreter {
    fn new() -> Interpreter {
        Interpreter {
            heap: Heap::new(),
            cself: None,
            variables: Vec::new(),
            locals: Table::new(),
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
        let name = name.to_ascii_uppercase();
        self.variables.retain(|(bound, _)| *bound != name);
        if let Some(value) = value {
            self.variables.push((name, value));
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
        let body = Body::pointer(ObjRef::NIL, BehaviourHandle::new(0), value);
        self.heap.alloc(body)
    }

    fn locals(&mut self) -> &mut Table {
        &mut self.locals
    }
}

/// One activation's conversion state.
struct Session {
    interpreter: Interpreter,
    strings: CStringPool,
}

impl Session {
    fn new() -> Session {
        Session {
            interpreter: Interpreter::new(),
            strings: CStringPool::new(),
        }
    }

    fn text(&mut self, bytes: &[u8]) -> Option<ObjRef> {
        Some(self.interpreter.text(bytes))
    }

    fn call(
        &mut self,
        entry: &NativeMethodEntry,
        context: &mut RexxMethodContext_,
        arguments: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        let activation = Activation::new(Conversion {
            host: &mut self.interpreter,
            strings: &mut self.strings,
        });
        invoke::method(entry, context, &activation, arguments)
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
    with_context(|context| {
        assert_eq!(
            invoke::signature(init, context),
            Ok(vec![
                code::INT,
                OPTIONAL_ARGUMENT | code::CSTRING,
                OPTIONAL_ARGUMENT | code::CSTRING,
            ])
        );
        assert_eq!(
            invoke::signature(parse, context),
            Ok(vec![
                code::INT,
                code::CSELF,
                code::CSTRING,
                OPTIONAL_ARGUMENT | code::CSTRING,
            ])
        );
    });
}

#[test]
fn regexp_init_with_no_arguments_answers_zero_and_allocates_an_automaton() {
    let library = rxregexp();
    let init = library.method(b"RegExp_Init").expect("RegExp_Init");
    let uninit = library.method(b"RegExp_Uninit").expect("RegExp_Uninit");
    let mut session = Session::new();

    with_context(|context| {
        assert_eq!(session.call(init, context, &[]), returned(0));
        let automaton = allocated();
        assert!(!automaton.is_null(), "the extension allocated nothing");
        assert_eq!(VARIABLES_SET.get(), 1, "CSELF was not stored");
        assert!(RAISED.with(|seen| seen.borrow().is_empty()));

        // Hand the pointer back as `CSELF` so that the automaton is freed.
        session.interpreter.cself = Some(automaton);
        assert_eq!(session.call(uninit, context, &[]), returned(0));
        assert_eq!(VARIABLES_DROPPED.get(), 1);
    });
}

#[test]
fn regexp_init_with_one_argument_parses_it() {
    let library = rxregexp();
    let init = library.method(b"RegExp_Init").expect("RegExp_Init");
    let uninit = library.method(b"RegExp_Uninit").expect("RegExp_Uninit");
    let mut session = Session::new();
    let expression = session.text(b"a*b");

    with_context(|context| {
        assert_eq!(session.call(init, context, &[expression]), returned(0));
        assert!(
            RAISED.with(|seen| seen.borrow().is_empty()),
            "`a*b` parses, so nothing is raised"
        );
        session.interpreter.cself = Some(allocated());
        assert_eq!(session.call(uninit, context, &[]), returned(0));
    });
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

    with_context(|context| {
        assert_eq!(session.call(init, context, &[expression]), returned(0));
        assert_eq!(
            RAISED.with(|seen| seen.borrow().clone()),
            vec![INVALID_TEMPLATE]
        );
        assert_eq!(VARIABLES_SET.get(), 1, "the work after the raise still ran");
        session.interpreter.cself = Some(allocated());
        assert_eq!(session.call(uninit, context, &[]), returned(0));
    });
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

    with_context(|context| {
        assert_eq!(session.call(init, context, &[]), returned(0));
        session.interpreter.cself = Some(allocated());

        assert_eq!(session.call(parse, context, &[bad, bogus]), returned(3));
        assert_eq!(
            RAISED.with(|seen| seen.borrow().clone()),
            vec![INCORRECT_METHOD]
        );

        assert_eq!(session.call(uninit, context, &[]), returned(0));
    });
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

    with_context(|context| {
        assert_eq!(session.call(init, context, &[]), returned(0));
        session.interpreter.cself = Some(allocated());

        assert_eq!(session.call(parse, context, &[good]), returned(0));
        assert_eq!(
            session.call(parse, context, &[bad, minimal]),
            returned(3),
            "the oracle answers 3 for this template"
        );
        assert_eq!(
            NUMBERS.with(|seen| seen.borrow().len()),
            2,
            "each parse stores !POS"
        );

        assert_eq!(session.call(uninit, context, &[]), returned(0));
    });
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

    with_context(|context| {
        assert_eq!(session.call(init, context, &[]), returned(0));
        session.interpreter.cself = Some(allocated());

        let outcome = session.call(parse, context, &[]);
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

        assert_eq!(session.call(uninit, context, &[]), returned(0));
    });
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

    with_context(|context| {
        let outcome = session.call(init, context, &[expression, matchtype, extra]);
        assert_eq!(outcome, Err(Failure::TooManyArguments { expected: 2 }));
        assert_eq!(outcome.unwrap_err().error_number(true), Some(88922));
        assert_eq!(
            VARIABLES_SET.get(),
            0,
            "the extension must not have run at all"
        );
    });
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
