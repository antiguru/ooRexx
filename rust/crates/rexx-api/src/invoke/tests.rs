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

use std::borrow::Cow;
use std::cell::RefCell;

use rexx_core::{BehaviourHandle, Body, Bytes, Heap, ObjRef};

use super::{MAX_NATIVE_ARGUMENTS, method, routine};
use crate::ffi::{CALL_CONTEXT, CallContext, MethodContext, Seen, forget_seen, reading_stub, seen};
use crate::handles::Table;
use crate::layout::{
    METHOD_CONTEXT_INTERFACE, MethodContextInterface, POINTER, RexxCallContext_,
    RexxMethodContext_, ValueDescriptor,
};
use crate::load::{
    NativeMethodEntry, NativeRoutineEntry, ROUTINE_CLASSIC_STYLE, ROUTINE_TYPED_STYLE, stub_entry,
    stub_routine_entry,
};
use crate::values::{
    ARGUMENT_EXISTS, ARGUMENT_TERMINATOR, Activation, CStringPool, Class, Constants, Conversion,
    Failure, Host, Numeric, OPTIONAL_ARGUMENT, Raised, code,
};

/// What the stub and the interpreter each did, in the order they did it.
#[derive(Clone, PartialEq, Eq, Debug)]
enum Event {
    /// The stub was entered, and `array` is whether it was handed a
    /// descriptor array rather than the null that asks for a signature.
    Entered { array: bool },
    /// The table asked the interpreter to convert an argument, and this
    /// is how many times it had asked by then.
    Asked(usize),
}

thread_local! {
    static EVENTS: RefCell<Vec<Event>> = const { RefCell::new(Vec::new()) };
}

fn record(event: Event) {
    EVENTS.with(|events| events.borrow_mut().push(event));
}

fn events() -> Vec<Event> {
    EVENTS.with(|events| events.borrow().clone())
}

fn forget_events() {
    EVENTS.with(|events| events.borrow_mut().clear());
}

/// An `int` result and one `CSTRING` parameter.
static ONE_CSTRING: [u16; 3] = [code::INT, code::CSTRING, ARGUMENT_TERMINATOR];

/// The longest signature the descriptor array has room for, its
/// parameters optional so that none of them needs an argument.
static LONGEST: [u16; MAX_NATIVE_ARGUMENTS + 1] = longest();

/// One parameter more than [`LONGEST`].
static TOO_LONG: [u16; MAX_NATIVE_ARGUMENTS + 2] = too_long();

const fn longest() -> [u16; MAX_NATIVE_ARGUMENTS + 1] {
    let mut types = [OPTIONAL_ARGUMENT | code::CSTRING; MAX_NATIVE_ARGUMENTS + 1];
    types[0] = code::INT;
    types[MAX_NATIVE_ARGUMENTS] = ARGUMENT_TERMINATOR;
    types
}

const fn too_long() -> [u16; MAX_NATIVE_ARGUMENTS + 2] {
    let mut types = [OPTIONAL_ARGUMENT | code::CSTRING; MAX_NATIVE_ARGUMENTS + 2];
    types[0] = code::INT;
    types[MAX_NATIVE_ARGUMENTS + 1] = ARGUMENT_TERMINATOR;
    types
}

/// The address of a published signature, which the caller reads and does
/// not write through.
fn published(types: &'static [u16]) -> *mut u16 {
    types.as_ptr().cast_mut()
}

/// Record the entry and answer `types` for a signature request.
fn answer(arguments: *mut ValueDescriptor, types: &'static [u16]) -> *mut u16 {
    if arguments.is_null() {
        record(Event::Entered { array: false });
        return published(types);
    }
    record(Event::Entered { array: true });
    std::ptr::null_mut()
}

extern "C" fn probe(
    _context: *mut RexxMethodContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    answer(arguments, &ONE_CSTRING)
}

extern "C" fn longest_probe(
    _context: *mut RexxMethodContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    answer(arguments, &LONGEST)
}

extern "C" fn too_long_probe(
    _context: *mut RexxMethodContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    answer(arguments, &TOO_LONG)
}

/// An `int` result and one parameter of a code the header does not
/// define.
static UNKNOWN_CODE: [u16; 3] = [code::INT, 9, ARGUMENT_TERMINATOR];

/// An `int` result declared with the optional bit.
static OPTIONAL_RESULT: [u16; 2] = [OPTIONAL_ARGUMENT | code::INT, ARGUMENT_TERMINATOR];

extern "C" fn unknown_code_probe(
    _context: *mut RexxMethodContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    answer(arguments, &UNKNOWN_CODE)
}

extern "C" fn optional_result_probe(
    _context: *mut RexxMethodContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    answer(arguments, &OPTIONAL_RESULT)
}

extern "C" fn routine_probe(
    _context: *mut RexxCallContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    answer(arguments, &ONE_CSTRING)
}

/// A stub that publishes no signature at all.
extern "C" fn silent_probe(
    _context: *mut RexxMethodContext_,
    arguments: *mut ValueDescriptor,
) -> *mut u16 {
    record(Event::Entered {
        array: !arguments.is_null(),
    });
    std::ptr::null_mut()
}

/// Stands in for the interpreter, recording what the table asks it for.
struct Interpreter {
    heap: Heap,
    asked: usize,
    variables: Vec<(Vec<u8>, ObjRef)>,
    locals: Table,
    numeric: Numeric,
    /// What `double_object` was asked for, in order.
    doubles: Vec<(f64, usize)>,
    /// Whether `whole_number` runs [`crate::ffi::refusing_stub`] as a
    /// native call of its own before answering.
    nest: bool,
    /// What that nested call answered.
    nested: Option<Result<Option<ObjRef>, Failure>>,
    /// What `arguments` answers.
    argument_list: ObjRef,
}

impl Interpreter {
    fn new() -> Interpreter {
        Interpreter {
            heap: Heap::new(),
            asked: 0,
            variables: Vec::new(),
            locals: Table::new(),
            numeric: Numeric {
                digits: 9,
                fuzz: 0,
                engineering: false,
            },
            doubles: Vec::new(),
            nest: false,
            nested: None,
            argument_list: ObjRef::NIL,
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
        self.asked += 1;
        record(Event::Asked(self.asked));
        Ok(Some(object))
    }

    fn string_bytes(&self, object: ObjRef) -> Option<Cow<'_, [u8]>> {
        match &self.heap.get(object)?.body {
            Body::Text { bytes, .. } => Some(Cow::Borrowed(bytes.as_slice())),
            _ => None,
        }
    }

    fn cself(&mut self) -> Option<POINTER> {
        None
    }

    fn constants(&mut self) -> Constants<ObjRef> {
        constants_of(self)
    }

    fn set_object_variable(&mut self, name: &[u8], value: Option<ObjRef>) {
        set_variable(&mut self.variables, name, value);
    }

    fn drop_object_variable(&mut self, name: &[u8]) {
        set_variable(&mut self.variables, name, None);
    }

    fn whole_number(&mut self, value: isize) -> ObjRef {
        if self.nest {
            let entry = stub_entry(b"refusing", crate::ffi::refusing_stub);
            let mut inner = Interpreter::new();
            let mut strings = CStringPool::new();
            let activation = Activation::new(Conversion {
                host: &mut inner,
                strings: &mut strings,
            });
            let mut contexts = crate::ffi::Contexts::new(&activation);
            self.nested = Some(method(&entry, &contexts.method(), &activation, &[]));
        }
        whole_number_object(&mut self.heap, value)
    }

    fn new_pointer(&mut self, value: POINTER) -> ObjRef {
        let body = Body::pointer(ObjRef::NIL, BehaviourHandle::new(0), value);
        self.heap.alloc(body)
    }

    fn numeric(&self) -> Numeric {
        self.numeric
    }

    fn double_value(&mut self, object: ObjRef) -> Result<Option<f64>, Raised> {
        let text = self.string_bytes(object).map(Cow::into_owned);
        Ok(text.and_then(|bytes| String::from_utf8(bytes).ok()?.parse().ok()))
    }

    fn signed_integer(
        &mut self,
        object: ObjRef,
        min: i64,
        max: i64,
    ) -> Result<Option<i64>, Raised> {
        let text = self.string_bytes(object).map(Cow::into_owned);
        Ok(text
            .and_then(|bytes| String::from_utf8(bytes).ok()?.parse().ok())
            .filter(|number| (min..=max).contains(number)))
    }

    fn unsigned_integer(&mut self, _object: ObjRef, _max: u64) -> Result<Option<u64>, Raised> {
        unreachable!("the stubs these tests call declare no unsigned integer")
    }

    fn logical(&mut self, _object: ObjRef) -> Result<Result<bool, ObjRef>, Raised> {
        unreachable!("the stubs these tests call declare no logical_t")
    }

    fn array_value(&mut self, _object: ObjRef) -> Result<Option<ObjRef>, Raised> {
        unreachable!("the stubs these tests call declare no array")
    }

    fn is_stem(&self, _object: ObjRef) -> bool {
        unreachable!("the stubs these tests call declare no stem")
    }

    fn context_stem(&mut self, _object: ObjRef) -> Result<Option<ObjRef>, Raised> {
        unreachable!("the stubs these tests call declare no stem")
    }

    fn is_instance_of(&mut self, _object: ObjRef, _class: Class) -> bool {
        unreachable!("the stubs these tests call declare no class-checked argument")
    }

    fn pointer_value(&self, _object: ObjRef) -> Option<POINTER> {
        unreachable!("the stubs these tests call declare no POINTER argument")
    }

    fn string_value_text(&mut self, _object: ObjRef) -> Vec<u8> {
        unreachable!("the stubs these tests call declare no POINTERSTRING")
    }

    fn receiver(&mut self) -> ObjRef {
        unreachable!("the stubs these tests call declare no OSELF")
    }

    fn scope(&mut self) -> ObjRef {
        unreachable!("the stubs these tests call declare no SCOPE")
    }

    fn super_scope(&mut self) -> ObjRef {
        unreachable!("the stubs these tests call declare no SUPER")
    }

    fn arguments(&mut self) -> ObjRef {
        self.argument_list
    }

    fn message_name(&mut self) -> Vec<u8> {
        b"STUB".to_vec()
    }

    fn unsigned_number(&mut self, _value: u64) -> ObjRef {
        unreachable!("the stubs these tests call return no unsigned integer")
    }

    fn new_string(&mut self, bytes: &[u8]) -> ObjRef {
        self.text(bytes)
    }

    fn double_object(&mut self, value: f64, precision: usize) -> ObjRef {
        self.doubles.push((value, precision));
        self.text(format!("{value} at {precision}").as_bytes())
    }

    fn locals(&mut self) -> &mut Table {
        &mut self.locals
    }
}

/// What a stand-in interpreter answers for the constant objects: `.nil`,
/// the boolean values as the small integers the oracle's `RexxInteger`
/// ones render as, and an empty string object.
fn constants_of(interpreter: &mut Interpreter) -> Constants<ObjRef> {
    Constants {
        nil: ObjRef::NIL,
        true_object: ObjRef::small_int(1).expect("one is a small integer"),
        false_object: ObjRef::small_int(0).expect("zero is a small integer"),
        null_string: interpreter.text(b""),
    }
}

/// Binds `name`, upper-cased as `getVariableRetriever` upper-cases it, or
/// returns it to the uninitialised state for a `value` of `None`.
fn set_variable(variables: &mut Vec<(Vec<u8>, ObjRef)>, name: &[u8], value: Option<ObjRef>) {
    let name = name.to_ascii_uppercase();
    variables.retain(|(bound, _)| *bound != name);
    if let Some(value) = value {
        variables.push((name, value));
    }
}

/// `Numerics::wholenumberToObject`: a small integer where the value fits
/// the tag, and its digits otherwise.
fn whole_number_object(heap: &mut Heap, value: isize) -> ObjRef {
    match i64::try_from(value).ok().and_then(ObjRef::small_int) {
        Some(object) => object,
        None => heap.alloc(Body::Text {
            bytes: Bytes::from_slice(value.to_string().as_bytes()),
            num: None,
        }),
    }
}

/// An address nothing in this crate produces, so a context field still
/// holding it was not written. `std::ptr::dangling_mut` will not do: a
/// caller writing that same value would be indistinguishable from one
/// writing nothing.
fn sentinel<T>() -> *mut T {
    std::ptr::without_provenance_mut(0x5eed_0000)
}

/// A context addressing the refusing table, which no probe calls back
/// into.
fn method_context(table: &'static MethodContextInterface) -> RexxMethodContext_ {
    RexxMethodContext_ {
        threadContext: std::ptr::null_mut(),
        functions: std::ptr::from_ref(table).cast_mut(),
        arguments: sentinel(),
    }
}

/// What one run produced: its outcome, the events in order, how many
/// `CSTRING` copies the call interned, and the array left on the context.
struct Run {
    outcome: Result<Option<ObjRef>, Failure>,
    events: Vec<Event>,
    interned: usize,
    left_on_context: *mut ValueDescriptor,
}

/// Run `entry` against `arguments` freshly made strings.
fn run(entry: &NativeMethodEntry, arguments: usize) -> Run {
    forget_events();
    forget_seen();
    let mut interpreter = Interpreter::new();
    let supplied: Vec<Option<ObjRef>> = (0..arguments)
        .map(|at| Some(interpreter.text(format!("argument {at}").as_bytes())))
        .collect();
    let mut strings = CStringPool::new();
    let mut context = method_context(&METHOD_CONTEXT_INTERFACE);
    let outcome = {
        let activation = Activation::new(Conversion {
            host: &mut interpreter,
            strings: &mut strings,
        });
        method(
            entry,
            &MethodContext::bare(&mut context),
            &activation,
            &supplied,
        )
    };
    Run {
        outcome,
        events: events(),
        interned: strings.len(),
        left_on_context: context.arguments,
    }
}

/// [`run`] for a routine, over a call context no `Contexts` built.
fn run_routine(entry: &NativeRoutineEntry, arguments: usize) -> Run {
    forget_events();
    forget_seen();
    let mut interpreter = Interpreter::new();
    let supplied: Vec<Option<ObjRef>> = (0..arguments)
        .map(|at| Some(interpreter.text(format!("argument {at}").as_bytes())))
        .collect();
    let mut strings = CStringPool::new();
    let mut context = RexxCallContext_ {
        threadContext: std::ptr::null_mut(),
        functions: std::ptr::from_ref(&CALL_CONTEXT).cast_mut(),
        arguments: sentinel(),
    };
    let outcome = {
        let activation = Activation::new(Conversion {
            host: &mut interpreter,
            strings: &mut strings,
        });
        routine(
            entry,
            &CallContext::bare(&mut context),
            &activation,
            &supplied,
        )
    };
    Run {
        outcome,
        events: events(),
        interned: strings.len(),
        left_on_context: context.arguments,
    }
}

/// A typed routine runs the method's two calls in the method's order, and
/// publishes its array for exactly the length of the call.
#[test]
fn a_routine_runs_the_two_calls_a_method_runs() {
    let entry = stub_routine_entry(ROUTINE_TYPED_STYLE, b"routine", routine_probe);
    let run = run_routine(&entry, 1);
    assert_eq!(
        run.events,
        vec![
            Event::Entered { array: false },
            Event::Asked(1),
            Event::Entered { array: true },
        ]
    );
    assert!(run.outcome.is_ok(), "{:?}", run.outcome);
    assert_eq!(run.interned, 1);
    assert!(run.left_on_context.is_null());
}

/// A classic row's address is a function of another C signature, so it is
/// refused before either call and its stub is never entered.
#[test]
fn a_classic_routine_is_refused_before_its_stub_is_entered() {
    let entry = stub_routine_entry(ROUTINE_CLASSIC_STYLE, b"classic", routine_probe);
    let run = run_routine(&entry, 1);
    assert_eq!(run.outcome, Err(Failure::ClassicStyle));
    assert_eq!(run.events, Vec::new());
    assert_eq!(run.outcome.unwrap_err().error_number(false), None);
}

/// A row of any style but typed publishes no signature and calls nothing,
/// whatever its caller checked: its address is a function of the
/// `RXSTRING` signature (`api/oorexxapi.h:209`).
#[test]
fn a_classic_row_publishes_no_signature_and_calls_nothing() {
    forget_events();
    let entry = stub_routine_entry(ROUTINE_CLASSIC_STYLE, b"classic", routine_probe);
    let mut context = RexxCallContext_ {
        threadContext: std::ptr::null_mut(),
        functions: std::ptr::from_ref(&CALL_CONTEXT).cast_mut(),
        arguments: std::ptr::null_mut(),
    };
    let bare = CallContext::bare(&mut context);
    assert_eq!(entry.signature(&bare, MAX_NATIVE_ARGUMENTS + 1), None);
    let mut descriptors: [ValueDescriptor; MAX_NATIVE_ARGUMENTS] =
        std::array::from_fn(|_| super::empty());
    let _ = entry.call(&bare, &mut descriptors, None);
    assert_eq!(
        events(),
        Vec::new(),
        "the classic row's address was entered"
    );
}

/// The too-many check is the one both calls share.
#[test]
fn an_argument_a_routine_does_not_consume_is_refused() {
    let entry = stub_routine_entry(ROUTINE_TYPED_STYLE, b"routine", routine_probe);
    let run = run_routine(&entry, 2);
    assert_eq!(run.outcome, Err(Failure::TooManyArguments { expected: 1 }));
    assert_eq!(
        run.events,
        vec![Event::Entered { array: false }, Event::Asked(1)]
    );
}

/// A routine's stub reaches the calling activation's `NUMERIC` settings
/// through the call-context table and builds a number through the thread
/// table its call context links, and the object that builds is the
/// routine's answer.
#[test]
fn a_routine_reads_numeric_settings_and_builds_a_double_through_its_contexts() {
    let entry = stub_routine_entry(ROUTINE_TYPED_STYLE, b"numeric", crate::ffi::numeric_stub);
    let mut interpreter = Interpreter::new();
    interpreter.numeric = Numeric {
        digits: 7,
        fuzz: 2,
        engineering: true,
    };
    let mut strings = CStringPool::new();
    let outcome = {
        let activation = Activation::new(Conversion {
            host: &mut interpreter,
            strings: &mut strings,
        });
        let mut contexts = crate::ffi::Contexts::new(&activation);
        routine(&entry, &contexts.call(), &activation, &[])
    };
    assert_eq!(crate::ffi::numeric_seen(), Some((7, 2, 1)));
    assert_eq!(interpreter.doubles, vec![(crate::ffi::STUB_DOUBLE, 7)]);
    let answered = outcome
        .expect("the routine answers")
        .expect("the routine answers an object");
    assert_eq!(
        interpreter.string_bytes(answered).as_deref(),
        Some(&b"1.5 at 7"[..])
    );
}

/// **No argument exists when the signature is asked for.** The stub is
/// handed no descriptor array, and the table has not yet been asked to
/// convert anything, so there is nothing for the array channel to carry.
/// `GetArgument` is a channel of its own and reaches the activation's own
/// list, which this says nothing about; the oracle builds its context
/// before the signature call too (`NativeActivation.cpp:1289-1291`).
#[test]
fn the_signature_call_runs_before_any_argument_is_touched() {
    let entry = stub_entry(b"probe", probe);
    let run = run(&entry, 1);
    assert_eq!(
        run.events,
        vec![
            Event::Entered { array: false },
            Event::Asked(1),
            Event::Entered { array: true },
        ],
        "the signature call has to come first and be handed no array"
    );
    assert!(run.outcome.is_ok(), "{:?}", run.outcome);
    assert_eq!(
        run.interned, 1,
        "the argument the signature call could not see is there for the \
         call that follows it"
    );
}

/// The signature call leaves the array channel alone, so an extension
/// reading it through the context (`api/oorexxapi.h:4276`) sees whatever
/// was there before rather than an argument of this call.
#[test]
fn the_signature_call_publishes_no_array() {
    forget_events();
    let entry = stub_entry(b"probe", probe);
    let mut context = method_context(&METHOD_CONTEXT_INTERFACE);
    let before = context.arguments;
    let types = super::signature(&entry, &MethodContext::bare(&mut context));
    assert_eq!(types, Ok(vec![code::INT, code::CSTRING]));
    assert_eq!(events(), vec![Event::Entered { array: false }]);
    assert_eq!(
        context.arguments, before,
        "the context was built holding an array, and this call left it alone"
    );
}

/// The call publishes its array on the context, which is where
/// `argumentExists` reads it (`api/oorexxapi.h:4276`), and what it
/// publishes is the array the stub was handed with the argument's flags
/// in it.
#[test]
fn the_call_publishes_its_array_on_the_context() {
    let entry = stub_entry(b"reading", reading_stub);
    let run = run(&entry, 1);
    assert!(run.outcome.is_ok(), "{:?}", run.outcome);
    assert_eq!(
        seen(),
        Seen {
            same_array: true,
            flags: ARGUMENT_EXISTS,
        }
    );
}

/// The descriptor array does not outlive the call it was made for.
#[test]
fn the_context_holds_no_array_once_the_call_is_over() {
    let entry = stub_entry(b"probe", probe);
    let run = run(&entry, 1);
    assert!(run.outcome.is_ok(), "{:?}", run.outcome);
    assert!(
        run.left_on_context.is_null(),
        "the context was built holding a dangling array, so a null here \
         is this call's doing"
    );
}

/// A stub reaches the activation that called it through the method
/// context a `Contexts` hands out, which is where `ffi::owner_of` reads
/// the owner back past the public struct.
#[test]
fn a_stub_reaches_its_activation_through_the_context_it_was_handed() {
    let entry = stub_entry(b"dropping", crate::ffi::dropping_stub);
    let mut interpreter = Interpreter::new();
    let held = interpreter.text(b"a pointer's stand-in");
    interpreter.variables.push((b"CSELF".to_vec(), held));
    let mut strings = CStringPool::new();
    let outcome = {
        let activation = Activation::new(Conversion {
            host: &mut interpreter,
            strings: &mut strings,
        });
        let mut contexts = crate::ffi::Contexts::new(&activation);
        method(&entry, &contexts.method(), &activation, &[])
    };
    assert_eq!(outcome, Ok(Some(ObjRef::small_int(0).expect("zero"))));
    assert!(
        interpreter.variables.is_empty(),
        "the drop did not reach the activation's host"
    );
}

/// A stub reaches the activation through the thread context the method
/// context links, which is where `ffi::owner_of` reads the owner back past
/// the thread wrapper's public struct.
#[test]
fn a_stub_reaches_its_activation_through_the_thread_context() {
    let entry = stub_entry(b"thread_table", crate::ffi::thread_table_stub);
    let mut interpreter = Interpreter::new();
    let argument = interpreter.text(b"hello");
    let mut strings = CStringPool::new();
    let (outcome, pending) = {
        let activation = Activation::new(Conversion {
            host: &mut interpreter,
            strings: &mut strings,
        });
        let mut contexts = crate::ffi::Contexts::new(&activation);
        let outcome = method(&entry, &contexts.method(), &activation, &[Some(argument)]);
        (outcome, activation.pending())
    };
    assert_eq!(outcome, Ok(Some(ObjRef::small_int(0).expect("zero"))));
    assert_eq!(pending, Some(crate::ffi::STUB_CONDITION));
    let variable = |name: &[u8]| {
        interpreter
            .variables
            .iter()
            .find(|(bound, _)| bound == name)
            .map(|(_, value)| *value)
    };
    assert_eq!(variable(b"LENGTH"), ObjRef::small_int(5));
    assert_eq!(variable(b"FIRST"), ObjRef::small_int(i64::from(b'h')));
    let pointer = variable(b"POINTER").expect("NewPointer's object reached the host");
    let address = match interpreter.heap.get(pointer).map(|object| &object.body) {
        Some(Body::Instance {
            native: Some(state),
            ..
        }) => state.pointer(),
        _ => None,
    };
    assert_eq!(
        address,
        Some(std::ptr::without_provenance_mut(crate::ffi::STUB_POINTER))
    );
}

/// A stub reaches the instance through the thread context, and the
/// instance answers the version and level the frozen header names:
/// measured, oracle, `orxmethod`'s `TestInterpreterVersion` is `328448` and
/// `TestLanguageLevel` `1542`.
#[test]
fn a_stub_reaches_the_instance_through_the_thread_context() {
    let entry = stub_entry(b"instance", crate::ffi::instance_stub);
    let mut interpreter = Interpreter::new();
    let mut strings = CStringPool::new();
    let outcome = {
        let activation = Activation::new(Conversion {
            host: &mut interpreter,
            strings: &mut strings,
        });
        let mut contexts = crate::ffi::Contexts::new(&activation);
        method(&entry, &contexts.method(), &activation, &[])
    };
    assert_eq!(outcome, Ok(Some(ObjRef::small_int(0).expect("zero"))));
    let variable = |name: &[u8]| {
        interpreter
            .variables
            .iter()
            .find(|(bound, _)| bound == name)
            .map(|(_, value)| *value)
    };
    assert_eq!(variable(b"VERSION"), ObjRef::small_int(328_448));
    assert_eq!(variable(b"LEVEL"), ObjRef::small_int(1542));
}

/// **A call that reached an unwritten member refuses naming it, and a
/// native call nested inside it keeps its own record.** The outer stub
/// reaches `NewStringFromAsciiz`, then a member whose host runs a native
/// call reaching `HaltThread`, then raises a condition; the outer call
/// answers its own first member and not the inner one, the inner call
/// answers its own, and the refusal forgets the condition raised after it.
#[test]
fn a_refused_member_is_the_calls_answer_and_a_nested_call_keeps_its_own() {
    let entry = stub_entry(b"nesting", crate::ffi::nesting_stub);
    let mut interpreter = Interpreter::new();
    interpreter.nest = true;
    let mut strings = CStringPool::new();
    let (outcome, pending) = {
        let activation = Activation::new(Conversion {
            host: &mut interpreter,
            strings: &mut strings,
        });
        let mut contexts = crate::ffi::Contexts::new(&activation);
        let outcome = method(&entry, &contexts.method(), &activation, &[]);
        (outcome, activation.pending())
    };
    assert_eq!(
        outcome,
        Err(Failure::UnfilledSlot {
            entry: "RexxThreadInterface.NewStringFromAsciiz"
        })
    );
    assert_eq!(
        interpreter.nested,
        Some(Err(Failure::UnfilledSlot {
            entry: "RexxThreadInterface.HaltThread"
        }))
    );
    assert_eq!(
        pending, None,
        "the condition raised after the refusal survived it"
    );
}

/// Runs the method `stub` against `arguments` freshly made strings, over
/// the contexts a `Contexts` builds, and answers the outcome beside the
/// bytes of the object it answered, if any.
fn run_stub(
    stub: crate::load::NativeMethod,
    arguments: &[&[u8]],
) -> (Result<Option<ObjRef>, Failure>, Option<Vec<u8>>) {
    let entry = stub_entry(b"stub", stub);
    let mut interpreter = Interpreter::new();
    let supplied: Vec<Option<ObjRef>> = arguments
        .iter()
        .map(|bytes| Some(interpreter.text(bytes)))
        .collect();
    let mut strings = CStringPool::new();
    let outcome = {
        let activation = Activation::new(Conversion {
            host: &mut interpreter,
            strings: &mut strings,
        });
        let mut contexts = crate::ffi::Contexts::new(&activation);
        method(&entry, &contexts.method(), &activation, &supplied)
    };
    let bytes = match &outcome {
        Ok(Some(object)) => interpreter
            .string_bytes(*object)
            .map(std::borrow::Cow::into_owned),
        _ => None,
    };
    (outcome, bytes)
}

/// A `CSTRING` result is copied out of the extension's memory when the
/// call returns and stops at the first NUL, and a null one is no object.
#[test]
fn a_cstring_result_is_copied_up_to_its_first_nul() {
    let (outcome, bytes) = run_stub(crate::ffi::cstring_stub, &[]);
    assert!(matches!(outcome, Ok(Some(_))), "{outcome:?}");
    assert_eq!(bytes.as_deref(), Some(&b"answered"[..]));
    let (outcome, _) = run_stub(crate::ffi::null_cstring_stub, &[]);
    assert_eq!(outcome, Ok(None));
}

/// A special code as the result type is refused through the call, whose
/// descriptor holds a `CSTRING` the refusal does not turn into a string.
#[test]
fn a_special_code_as_the_result_is_refused() {
    let (outcome, _) = run_stub(crate::ffi::name_result_stub, &[]);
    assert_eq!(outcome, Err(Failure::ResultSignature));
}

/// `ARGLIST` lifts the check for arguments the signature does not consume,
/// in a method and in a routine, which share it (ruling S2); the
/// signature's other parameters still convert. Measured, oracle:
/// `ForgeRoutineIntThenArglist(7, 2, 3)` answers `7` and
/// `TestIntArg(1, 2)` is 88.922.
#[test]
fn an_argument_list_parameter_takes_arguments_nothing_else_consumes() {
    let (outcome, bytes) = run_stub(crate::ffi::arglist_stub, &[b"7", b"8", b"9"]);
    assert_eq!(outcome, Ok(ObjRef::small_int(7)), "{bytes:?}");

    let entry = stub_routine_entry(
        ROUTINE_TYPED_STYLE,
        b"arglist",
        crate::ffi::arglist_routine_stub,
    );
    let mut interpreter = Interpreter::new();
    let supplied: Vec<Option<ObjRef>> = [&b"5"[..], b"6"]
        .iter()
        .map(|bytes| Some(interpreter.text(bytes)))
        .collect();
    let mut strings = CStringPool::new();
    let outcome = {
        let activation = Activation::new(Conversion {
            host: &mut interpreter,
            strings: &mut strings,
        });
        let mut contexts = crate::ffi::Contexts::new(&activation);
        routine(&entry, &contexts.call(), &activation, &supplied)
    };
    assert_eq!(outcome, Ok(ObjRef::small_int(5)));
}

#[test]
fn a_signature_that_fills_the_array_is_run() {
    let entry = stub_entry(b"longest", longest_probe);
    let run = run(&entry, 0);
    assert_eq!(run.outcome, Ok(Some(ObjRef::small_int(0).expect("zero"))));
    assert_eq!(
        run.events,
        vec![
            Event::Entered { array: false },
            Event::Entered { array: true },
        ]
    );
}

/// A signature longer than the array is refused rather than grown. The
/// oracle answers 93.968 in a method (`NativeActivation.cpp:238-241`).
#[test]
fn a_signature_longer_than_the_array_is_refused_and_not_called() {
    let entry = stub_entry(b"too_long", too_long_probe);
    let run = run(&entry, 0);
    assert_eq!(run.outcome, Err(Failure::Signature));
    assert_eq!(
        run.events,
        vec![Event::Entered { array: false }],
        "the stub must not be entered a second time"
    );
}

/// A parameter code nobody defines consumes an argument, and its absence
/// is checked first: measured through a forged extension, oracle 88.901
/// with no argument and 93.968 with one. Neither reaches the call.
#[test]
fn an_unknown_parameter_code_is_an_argument_position() {
    let entry = stub_entry(b"unknown", unknown_code_probe);
    let missing = run(&entry, 0);
    assert_eq!(
        missing.outcome,
        Err(Failure::MissingArgument { position: 1 })
    );
    let supplied = run(&entry, 1);
    assert_eq!(supplied.outcome, Err(Failure::Signature));
    assert_eq!(supplied.events, vec![Event::Entered { array: false }]);
}

/// A result word carrying the optional bit is refused once the call has
/// run, as `valueToObject` refuses it inside the call's `try`
/// (`NativeActivation.cpp:1302-1310`).
#[test]
fn a_result_word_carrying_the_optional_bit_is_refused_after_the_call() {
    let entry = stub_entry(b"optional_result", optional_result_probe);
    let run = run(&entry, 0);
    assert_eq!(run.outcome, Err(Failure::ResultSignature));
    assert_eq!(
        run.events,
        vec![
            Event::Entered { array: false },
            Event::Entered { array: true },
        ],
        "the stub runs before the result is refused"
    );
}

#[test]
fn a_stub_that_publishes_no_signature_is_refused() {
    let entry = stub_entry(b"silent", silent_probe);
    let run = run(&entry, 0);
    assert_eq!(run.outcome, Err(Failure::Signature));
    assert_eq!(run.events, vec![Event::Entered { array: false }]);
}

/// An argument the signature does not consume is 88.922, measured against
/// the oracle.
#[test]
fn an_argument_the_signature_does_not_consume_is_refused() {
    let entry = stub_entry(b"probe", probe);
    let run = run(&entry, 2);
    assert_eq!(run.outcome, Err(Failure::TooManyArguments { expected: 1 }));
    assert_eq!(
        run.events,
        vec![Event::Entered { array: false }, Event::Asked(1)],
        "the stub must not be entered with an argument list that was \
         refused"
    );
}
