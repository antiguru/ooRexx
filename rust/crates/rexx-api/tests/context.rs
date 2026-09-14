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

//! The context tables an extension calls back through, run against the
//! oracle's own compiled extension.

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use rexx_api::ffi::Contexts;
use rexx_api::handles::Table;
use rexx_api::invoke;
use rexx_api::layout::POINTER;
use rexx_api::load::{self, NativeMethodEntry};
use rexx_api::values::{Activation, CStringPool, Constants, Conversion, Failure, Host};
use rexx_core::{
    BehaviourHandle, BehaviourId, Body, Bytes, CollectStats, Heap, ObjRef, RootSet, ScopePools,
};

/// `Rexx_Error_Invalid_template` (`api/oorexxerrors.h:362`).
const INVALID_TEMPLATE: usize = 38000;

/// `Rexx_Error_Incorrect_method` (`api/oorexxerrors.h:518`).
const INCORRECT_METHOD: usize = 93000;

/// The repository root, which is three directories above this crate.
fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root is three directories above this crate")
}

/// The oracle's own build of the `rxregexp` extension, loaded and never
/// rebuilt (D5's amendment).
fn rxregexp() -> load::Library {
    let path = repository_root().join("build/lib/librxregexp.so");
    load::open_path(&path, "rxregexp")
        .expect("the extension asks for 4.0.0, which is below this interpreter")
        .expect("the extension publishes RexxGetPackage")
}

// --------------------------------------------------- the interpreter's side

/// What a stand-in interpreter should do when an extension asks it for a
/// number, which `RegExp_Parse` and `RegExp_Match` both do after they have
/// read their arguments.
enum OnWholeNumber {
    Nothing,
    /// Collect, with the objects the call itself is holding as the roots.
    Collect,
}

/// Stands in for the interpreter: a heap, one receiver whose scope pools hold
/// the object variables, and the class that is the running method's scope.
struct Interpreter {
    heap: Heap,
    receiver: ObjRef,
    scope: ObjRef,
    /// A second scope, which nothing the extension does may write into.
    other_scope: ObjRef,
    pointer_class: ObjRef,
    /// The one empty string object, which the oracle's `TheNullString` is.
    null_string: ObjRef,
    on_whole_number: OnWholeNumber,
    /// Every collection [`OnWholeNumber::Collect`] ran, in order.
    collections: Vec<CollectStats>,
    /// What a collection inside a call roots besides the receiver, named by
    /// the test rather than read off the local-reference table: a host
    /// callback runs with that table borrowed and cannot reach it.
    roots_during_call: Vec<ObjRef>,
}

impl Interpreter {
    fn new() -> Interpreter {
        let mut heap = Heap::new();
        let scope = heap.alloc(Body::Class { owned: Vec::new() });
        let other_scope = heap.alloc(Body::Class { owned: Vec::new() });
        let pointer_class = heap.alloc(Body::Class { owned: Vec::new() });
        // Immortal, as the oracle's own `TheNullString` is: every context
        // built during a test hands out a handle for it.
        let null_string = heap.alloc_immortal(
            BehaviourId::STRING,
            Body::Text {
                bytes: Bytes::from_slice(b""),
                num: None,
            },
        );
        let receiver = heap.alloc(Body::Instance {
            class: scope,
            behaviour: BehaviourHandle::new(0),
            name: None,
            pools: ScopePools::new(),
            own: None,
            native: None,
        });
        Interpreter {
            heap,
            receiver,
            scope,
            other_scope,
            pointer_class,
            null_string,
            on_whole_number: OnWholeNumber::Nothing,
            collections: Vec::new(),
            roots_during_call: Vec::new(),
        }
    }

    fn text(&mut self, bytes: &[u8]) -> ObjRef {
        self.heap.alloc(Body::Text {
            bytes: Bytes::from_slice(bytes),
            num: None,
        })
    }

    /// The receiver's scope pools.
    fn pools(&mut self) -> &mut ScopePools {
        match &mut self
            .heap
            .get_mut(self.receiver)
            .expect("the receiver is live")
            .body
        {
            Body::Instance { pools, .. } => pools,
            other => panic!("the receiver stopped being an instance: {other:?}"),
        }
    }

    /// What `name` holds in the running method's scope.
    fn variable(&mut self, name: &[u8]) -> Option<ObjRef> {
        let scope = self.scope;
        self.pools().get(scope, name)
    }

    /// The address a `.Pointer` object holds, or `None` for anything else.
    fn address_of(&self, object: ObjRef) -> Option<POINTER> {
        match &self.heap.get(object)?.body {
            Body::Instance {
                native: Some(state),
                ..
            } => state.pointer(),
            _ => None,
        }
    }
}

impl Host for Interpreter {
    fn is_method(&self) -> bool {
        true
    }

    fn string_value(&mut self, object: ObjRef) -> Option<ObjRef> {
        Some(object)
    }

    fn string_bytes(&self, object: ObjRef) -> Option<Cow<'_, [u8]>> {
        match &self.heap.get(object)?.body {
            Body::Text { bytes, .. } => Some(Cow::Borrowed(bytes.as_slice())),
            _ => None,
        }
    }

    fn cself(&mut self) -> Option<POINTER> {
        let held = self.variable(b"CSELF")?;
        self.address_of(held)
    }

    fn constants(&mut self) -> Constants<ObjRef> {
        Constants {
            nil: ObjRef::NIL,
            true_object: ObjRef::small_int(1).expect("one is a small integer"),
            false_object: ObjRef::small_int(0).expect("zero is a small integer"),
            null_string: self.null_string,
        }
    }

    fn set_object_variable(&mut self, name: &[u8], value: Option<ObjRef>) {
        let scope = self.scope;
        let name = name.to_ascii_uppercase();
        match value {
            Some(value) => self.pools().set(scope, &name, value),
            None => self.pools().clear(scope, &name),
        }
    }

    fn drop_object_variable(&mut self, name: &[u8]) {
        self.set_object_variable(name, None);
    }

    fn whole_number(&mut self, value: isize) -> ObjRef {
        if matches!(self.on_whole_number, OnWholeNumber::Collect) {
            let mut roots = RootSet::new();
            roots.add_global(".RECEIVER", self.receiver);
            for (index, object) in self.roots_during_call.iter().enumerate() {
                roots.add_global(&format!(".HANDLE{index}"), *object);
            }
            let stats = self.heap.collect(&roots);
            self.collections.push(stats);
        }
        ObjRef::small_int(i64::try_from(value).expect("a position fits an i64"))
            .expect("a position is a small integer")
    }

    fn new_pointer(&mut self, value: POINTER) -> ObjRef {
        let body = Body::pointer(self.pointer_class, BehaviourHandle::new(0), value);
        self.heap.alloc(body)
    }
}

/// One activation's state, reused across the calls of one test the way an
/// object's methods reuse the object.
struct Session {
    interpreter: Interpreter,
    locals: Table,
    strings: CStringPool,
}

impl Session {
    fn new() -> Session {
        Session {
            interpreter: Interpreter::new(),
            locals: Table::new(),
            strings: CStringPool::new(),
        }
    }

    fn text(&mut self, bytes: &[u8]) -> Option<ObjRef> {
        Some(self.interpreter.text(bytes))
    }

    /// Runs `entry` through the real tables, and answers what it returned
    /// beside the condition it raised.
    fn call(
        &mut self,
        entry: &NativeMethodEntry,
        arguments: &[Option<ObjRef>],
    ) -> (Result<Option<ObjRef>, Failure>, Option<usize>) {
        let activation = Activation::new(Conversion {
            host: &mut self.interpreter,
            locals: &mut self.locals,
            strings: &mut self.strings,
        });
        let mut contexts = Contexts::new(&activation);
        let outcome = invoke::method(entry, contexts.method(), &activation, arguments);
        (outcome, activation.pending())
    }

    /// [`Session::call`] for a test that expects neither a refusal nor a
    /// condition.
    fn ran(&mut self, entry: &NativeMethodEntry, arguments: &[Option<ObjRef>]) -> Option<ObjRef> {
        let (outcome, pending) = self.call(entry, arguments);
        assert_eq!(pending, None, "the extension raised");
        outcome.expect("the call was refused")
    }

    /// The roots a collection between two calls sees: the receiver, which the
    /// interpreter is holding for the send, and nothing else.
    fn between_calls(&mut self) -> CollectStats {
        let mut roots = RootSet::new();
        roots.add_global(".RECEIVER", self.interpreter.receiver);
        self.interpreter.heap.collect(&roots)
    }
}

/// The small integer an `int` result of `value` answers.
fn returned(value: i64) -> Option<ObjRef> {
    Some(ObjRef::small_int(value).expect("a small integer"))
}

// ------------------------------------------------------ one per function

/// `NewPointer` mints a `.Pointer` and `SetObjectVariable` binds it under
/// `CSELF` in the running method's own scope.
#[test]
fn new_pointer_and_set_object_variable_store_the_cself_block() {
    let library = rxregexp();
    let init = library.method(b"RegExp_Init").expect("RegExp_Init");
    let mut session = Session::new();

    assert_eq!(session.ran(init, &[]), returned(0));

    let held = session
        .interpreter
        .variable(b"CSELF")
        .expect("CSELF was not bound");
    let address = session
        .interpreter
        .address_of(held)
        .expect("CSELF holds something that is not a .Pointer");
    assert!(!address.is_null(), "the extension allocated nothing");
    assert_eq!(
        session.interpreter.pointer_class,
        match &session.interpreter.heap.get(held).expect("live").body {
            Body::Instance { class, .. } => *class,
            other => panic!("{other:?}"),
        },
        "the object bound under CSELF is not of the class the host mints"
    );

    let uninit = library.method(b"RegExp_Uninit").expect("RegExp_Uninit");
    session.ran(uninit, &[]);
}

/// The write lands in the method's own scope and in no other, which is
/// `getObjectVariables(getScope())` (`NativeActivation.cpp:1878`) and what a
/// subclass method's `expose CSELF` measures against the oracle.
#[test]
fn the_write_reaches_the_methods_scope_and_no_other() {
    let library = rxregexp();
    let init = library.method(b"RegExp_Init").expect("RegExp_Init");
    let uninit = library.method(b"RegExp_Uninit").expect("RegExp_Uninit");
    let mut session = Session::new();

    session.ran(init, &[]);

    let other = session.interpreter.other_scope;
    assert_eq!(
        session.interpreter.pools().get(other, b"CSELF"),
        None,
        "a scope the method does not belong to was written into"
    );
    session.ran(uninit, &[]);
}

/// `DropObjectVariable` returns the name to the uninitialised state, which is
/// how `RegExp_Uninit` keeps from freeing an automaton twice.
#[test]
fn drop_object_variable_unbinds_the_name() {
    let library = rxregexp();
    let init = library.method(b"RegExp_Init").expect("RegExp_Init");
    let uninit = library.method(b"RegExp_Uninit").expect("RegExp_Uninit");
    let mut session = Session::new();

    session.ran(init, &[]);
    assert!(session.interpreter.variable(b"CSELF").is_some());

    session.ran(uninit, &[]);
    assert_eq!(
        session.interpreter.variable(b"CSELF"),
        None,
        "CSELF is still bound, so a second UNINIT would free the automaton again"
    );
}

/// `WholeNumber` turns the automaton's position into an object, which
/// `RegExp_Parse` binds under `!POS`. Measured against the oracle: parsing
/// `aaa` leaves the position at 3.
#[test]
fn whole_number_to_object_carries_the_position_the_extension_computed() {
    let library = rxregexp();
    let init = library.method(b"RegExp_Init").expect("RegExp_Init");
    let parse = library.method(b"RegExp_Parse").expect("RegExp_Parse");
    let uninit = library.method(b"RegExp_Uninit").expect("RegExp_Uninit");
    let mut session = Session::new();
    let template = session.text(b"aaa");

    session.ran(init, &[]);
    assert_eq!(session.ran(parse, &[template]), returned(0));
    assert_eq!(
        session.interpreter.variable(b"!POS"),
        returned(3),
        "the oracle answers 3 for this template"
    );

    session.ran(uninit, &[]);
}

/// `StringData` and `StringLength` are the whole of what `RegExp_Match` reads
/// its subject through, so a match that succeeds is a witness for both.
#[test]
fn string_data_and_string_length_hand_over_the_subject() {
    let library = rxregexp();
    let init = library.method(b"RegExp_Init").expect("RegExp_Init");
    let matches = library.method(b"RegExp_Match").expect("RegExp_Match");
    let uninit = library.method(b"RegExp_Uninit").expect("RegExp_Uninit");
    let mut session = Session::new();
    let template = session.text(b"a*b");
    let hit = session.text(b"aab");
    let miss = session.text(b"aac");

    session.ran(init, &[template]);
    assert_eq!(session.ran(matches, &[hit]), returned(1));
    assert_eq!(session.ran(matches, &[miss]), returned(0));

    session.ran(uninit, &[]);
}

/// The oracle answers one address per string object, because it hands out an
/// interior pointer to an object that does not move. Two reads of one object
/// make one copy here and two objects make two.
#[test]
fn string_data_answers_one_address_for_one_object() {
    let library = rxregexp();
    let init = library.method(b"RegExp_Init").expect("RegExp_Init");
    let matches = library.method(b"RegExp_Match").expect("RegExp_Match");
    let uninit = library.method(b"RegExp_Uninit").expect("RegExp_Uninit");
    let mut session = Session::new();
    let template = session.text(b"a*b");
    let subject = session.text(b"aab");
    let another = session.text(b"aab");

    session.ran(init, &[template]);
    assert_eq!(session.strings.len(), 1, "the template was interned");

    session.ran(matches, &[subject]);
    session.ran(matches, &[subject]);
    assert_eq!(
        session.strings.len(),
        2,
        "the same object was copied more than once"
    );

    session.ran(matches, &[another]);
    assert_eq!(
        session.strings.len(),
        3,
        "a second object with the same bytes must get its own address"
    );

    session.ran(uninit, &[]);
}

/// `RaiseException0` records the condition and lets the extension run on
/// (`interpreter/api/ThreadContextStubs.cpp:1863-1885`), and a second raise
/// overwrites the first as `setConditionInfo` does.
#[test]
fn raise_exception0_records_the_condition_and_the_call_finishes() {
    let library = rxregexp();
    let init = library.method(b"RegExp_Init").expect("RegExp_Init");
    let parse = library.method(b"RegExp_Parse").expect("RegExp_Parse");
    let uninit = library.method(b"RegExp_Uninit").expect("RegExp_Uninit");
    let mut session = Session::new();
    let bad = session.text(b"[");
    let bogus = session.text(b"BOGUS");

    let (outcome, pending) = session.call(init, &[bad]);
    assert_eq!(outcome, Ok(returned(0)));
    assert_eq!(pending, Some(INVALID_TEMPLATE));
    assert!(
        session.interpreter.variable(b"CSELF").is_some(),
        "the work after the raise did not run"
    );

    // `RegExp_Parse` raises for the match type, parses anyway, and answers
    // what the parse said (`extensions/rxregexp/rxregexp.cpp:130`, `:135`).
    let (outcome, pending) = session.call(parse, &[bad, bogus]);
    assert_eq!(outcome, Ok(returned(3)));
    assert_eq!(pending, Some(INCORRECT_METHOD));

    session.ran(uninit, &[]);
}

/// The four members that are data rather than functions carry handles for the
/// objects the interpreter names (`Activity.cpp:1841-1849`).
#[test]
fn the_thread_table_carries_the_four_constant_objects() {
    let mut session = Session::new();
    let expected = session.interpreter.constants();
    let activation = Activation::new(Conversion {
        host: &mut session.interpreter,
        locals: &mut session.locals,
        strings: &mut session.strings,
    });
    let handles = Contexts::new(&activation).constants();

    let cx = activation.conversion();
    for (handle, object, name) in [
        (handles.nil, expected.nil, "RexxNil"),
        (handles.true_object, expected.true_object, "RexxTrue"),
        (handles.false_object, expected.false_object, "RexxFalse"),
        (
            handles.null_string,
            expected.null_string,
            "RexxNullString-shaped",
        ),
    ] {
        assert!(!handle.is_null(), "{name} is still null");
        assert_eq!(cx.locals.resolve(handle), Some(object), "{name}");
    }
}

// -------------------------------------------------------- across a collection

/// **The gate criterion.** An object's `CSELF` survives a collection between
/// two calls on it, so the second call reaches the same block the first
/// allocated.
#[test]
fn cself_survives_a_collection_between_two_calls() {
    let library = rxregexp();
    let init = library.method(b"RegExp_Init").expect("RegExp_Init");
    let parse = library.method(b"RegExp_Parse").expect("RegExp_Parse");
    let uninit = library.method(b"RegExp_Uninit").expect("RegExp_Uninit");
    let mut session = Session::new();

    session.ran(init, &[]);
    let before = session.interpreter.cself().expect("CSELF after INIT");

    // The local-reference table is not a root between two calls, and the
    // strings the first call interned are not objects, so the pointer object
    // is reachable only through the receiver's own pool.
    session.locals.clear();
    session.interpreter.text(b"garbage nothing holds");
    let stats = session.between_calls();
    assert!(
        stats.swept > 0,
        "the collection reclaimed nothing, so it witnesses nothing"
    );

    let after = session.interpreter.cself().expect("CSELF after collecting");
    assert_eq!(after, before, "the block moved or was reallocated");

    let template = session.text(b"aaa");
    assert_eq!(session.ran(parse, &[template]), returned(0));
    assert_eq!(session.interpreter.variable(b"!POS"), returned(3));

    session.ran(uninit, &[]);
}

/// The control for the test above: with the receiver unreachable, the same
/// collection takes the pointer object with it, so the survival rests on the
/// pool holding it and not on the collector sweeping nothing.
#[test]
fn the_cself_object_is_swept_when_the_receiver_is_not_a_root() {
    let library = rxregexp();
    let init = library.method(b"RegExp_Init").expect("RegExp_Init");
    let mut session = Session::new();

    session.ran(init, &[]);
    let held = session.interpreter.variable(b"CSELF").expect("CSELF");
    let address = session.interpreter.address_of(held).expect("a .Pointer");

    session.locals.clear();
    let stats = session.interpreter.heap.collect(&RootSet::new());
    assert!(stats.swept > 0);
    assert!(
        session.interpreter.heap.get(held).is_none(),
        "the pointer object outlived a collection that rooted nothing"
    );

    // The automaton itself is the extension's, and nothing here frees it: the
    // block leaks with the object, which is the state this control is showing
    // is reachable and the reason a `.Pointer` has to be rooted.
    assert!(!address.is_null());
}

/// A collection running inside the call, while the extension is holding a
/// `StringData` pointer, leaves the call able to finish and leaves the
/// receiver's `CSELF` where it was.
///
/// The roots are named by the test: a host callback runs with the
/// local-reference table borrowed and cannot read it, which is the open point
/// this task's report records for the phase that drives a real collection.
#[test]
fn a_collection_inside_the_call_leaves_it_able_to_finish() {
    let library = rxregexp();
    let init = library.method(b"RegExp_Init").expect("RegExp_Init");
    let matches = library.method(b"RegExp_Match").expect("RegExp_Match");
    let uninit = library.method(b"RegExp_Uninit").expect("RegExp_Uninit");
    let mut session = Session::new();
    let template = session.text(b"a*b");
    let subject = session.text(b"aab");

    session.ran(init, &[template]);
    let before = session.interpreter.cself().expect("CSELF after INIT");
    session.interpreter.text(b"garbage nothing holds");
    session.interpreter.roots_during_call = subject.into_iter().collect();
    session.interpreter.on_whole_number = OnWholeNumber::Collect;

    assert_eq!(session.ran(matches, &[subject]), returned(1));

    session.interpreter.on_whole_number = OnWholeNumber::Nothing;
    let stats = std::mem::take(&mut session.interpreter.collections);
    assert_eq!(stats.len(), 1, "the collection hook did not run once");
    assert!(
        stats[0].swept > 0,
        "the collection inside the call reclaimed nothing, so it witnesses nothing"
    );
    assert_eq!(
        session.interpreter.cself(),
        Some(before),
        "the block the receiver holds did not survive the collection"
    );

    session.ran(uninit, &[]);
}

/// The copy the pool owns outlives the object it was made from, which is what
/// makes a `CSTRING` safe for the length of a call in a heap that reallocates
/// its slots.
#[test]
fn the_object_keyed_copy_outlives_the_object_it_came_from() {
    let mut session = Session::new();
    let subject = session
        .interpreter
        .text(b"long enough to reach the heap rather than the handle");
    let handle = session.locals.register(subject);

    let pointer = {
        let activation = Activation::new(Conversion {
            host: &mut session.interpreter,
            locals: &mut session.locals,
            strings: &mut session.strings,
        });
        let first = activation.string_data(handle);
        assert_eq!(
            activation.string_data(handle),
            first,
            "one object must answer one address"
        );
        assert_eq!(activation.string_length(handle), 52);
        first
    };

    session.locals.clear();
    let stats = session.interpreter.heap.collect(&RootSet::new());
    assert!(stats.swept > 0);
    assert!(session.interpreter.heap.get(subject).is_none());

    assert_eq!(
        session.strings.bytes_at(pointer),
        Some(&b"long enough to reach the heap rather than the handle"[..]),
        "the pool stopped owning the bytes it handed out"
    );
}

/// A handle the activation does not hold reads as an object with no bytes,
/// which is the `NULL` and the zero the stubs answer on an exception.
#[test]
fn a_handle_this_activation_does_not_hold_reads_as_nothing() {
    let mut session = Session::new();
    let stranger: rexx_api::layout::RexxObjectPtr = std::ptr::without_provenance_mut(0x5eed_0000);
    let activation = Activation::new(Conversion {
        host: &mut session.interpreter,
        locals: &mut session.locals,
        strings: &mut session.strings,
    });
    assert!(activation.string_data(stranger).is_null());
    assert_eq!(activation.string_length(stranger), 0);
}

/// The state behind a `.Pointer` is an address and nothing else, so the only
/// object a collection reaches through one is its class.
#[test]
fn a_pointer_object_reaches_only_its_class() {
    let mut session = Session::new();
    let address: POINTER = std::ptr::without_provenance_mut(0x1234_5678);
    let object = session.interpreter.new_pointer(address);
    let mut reached = Vec::new();
    session
        .interpreter
        .heap
        .get(object)
        .expect("live")
        .body
        .trace(&mut reached);
    assert_eq!(reached, vec![session.interpreter.pointer_class]);
    assert_eq!(session.interpreter.address_of(object), Some(address));
}
