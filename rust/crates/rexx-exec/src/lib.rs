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

//! The executor.

use rexx_classes::MethodId;
use rexx_core::{Heap, NameMap, ObjRef, RootSet, SlotRef};
use rexx_num::Settings;
use rexx_parse::{
    CodeBody, ExprKind, InstructionKind, Operator, Program, SymbolId, SymbolTable, compound_parts,
    parse_program,
};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

// The value model: `text`/`number`/`to_text`/`to_number` on `Interp`, and the
// two rules D15 exists to enforce (a number's rendering is fixed at creation,
// and a `SmallInt` is admissible only within the DIGITS that produced it).
mod value;

// Stems and compound variables (D15a): tail resolution, the tombstone rule,
// and the "replace the object, mutate a tail in place" split.
mod stem;

// The in-process external data queue (I15) `PUSH`/`QUEUE` write to and
// `PULL`/`PARSE PULL` read back.
mod queue;
use queue::Queue;

// What a command line supplies to a top-level program: the one argument string
// it can carry, how a list of words becomes that string, and where `.input`
// reads from.
mod invocation;
pub use invocation::{Invocation, ProgramInput, Sinks, join_command_line};
pub use rexx_core::FrameBlock;

// `.input`: one line position, shared by every construct that reads a line,
// and the queue-first rule `PULL` follows on top of it.
mod input;
use input::Input;

// The per-body resolution plan (D16): `Plan`, `BodyKey`, `ProgramId`, the
// plan cache, and the full name-resolution order (plan, then `extra`, then
// growth).
mod plan;
use plan::{BodyKey, BodyKind, ClassPackage, CompoundName, Package, Plan, ProgramId};

// One activation: everything about the frame currently executing (D16).
mod activation;
use activation::{Activation, ActivationId, CallType, InstanceVar};
use clause::ClauseState;

// `Raised` (the payload of a real Rexx condition) and `Failure` (either a
// `Loud` not-implemented marker or a `Raised` condition, the one type
// `step` and everything above it propagate).
mod error;
use error::{ClauseSite, Failure, FailureSite, Raised};

// Expression evaluation (`eval`/`eval_node`): terms, arithmetic and
// concatenation.
mod eval;

// The builtin functions: the name set (read from `rexx_inventory`, never
// copied), the per-name arity, and the `dispatch` `Interp::invoke_call`
// reaches for a name `resolve_call` answered `Resolved::Builtin` for.
pub(crate) mod builtin;

// The instruction loop (D16's "Control flow"): `Flow`, and `step` and its two
// callers, together with the borrow discipline `run_activation` is written
// down to prove (Task 3's spike). Extended task by task with the branches and
// calls later tasks add; Task 9 is the first to extend it, with the seven
// instructions that do not branch.
mod clause;
mod run;
use run::Ended;

// The `PARSE` template engine: the movement cursor (source-independent, one
// struct, unit-tested against measured oracle bytes) and the driver that
// evaluates trigger operands, traces, and assigns the targets.
mod parse_template;

// The interpreter's version string, platform name and line terminator:
// what `PARSE VERSION`, `PARSE SOURCE`, `.ENDOFLINE` and `RexxInfo` answer.
mod version;

// `TRACE` (D17): the mode, the nine reachable prefixes' own byte formatting,
// and the classification a `TRACE`/`TRACE VALUE` setting goes through to
// become one. The `Op::Clause` region and its loop drivers, and
// `eval.rs`'s `eval`, own *when* to call into this module; this module owns
// only the bytes.
mod trace;
use trace::ChunkTrace;

// The register-based instruction stream (Phase 4e): `Op`, `Chunk`, and
// `compile`, the one pass that turns a body into a `Chunk`. `Interp::chunk_for`
// (`plan.rs`, beside `plan_for`) is the cache that makes compiling a body once
// rather than once per entry.
mod ir;

// Message dispatch (Phase 5a): the object model a send resolves against, the
// `resolve`/`invoke` pair D24 asks for, and the one security chokepoint D45
// asks for.
mod dispatch;

// `.environment`, `.local`, `.context` and `.methods` as objects (D33), the
// order `PackageClass::findClass` resolves a `.NAME` in, and the one directory
// chokepoint D45 asks for.
mod environment;

// `::OPTIONS` (Phase 5c): the numeric, trace and condition settings a file's
// directives leave on its own package, and which every activation of that
// package's code starts from.
mod options;
use options::PackageOptions;

// `::REQUIRES` (Phase 5d): the routes a required file's name is searched over
// and the extensions appended to it.
mod require;

// Command clauses: the handler an `ADDRESS` environment names, the
// child it runs, and the return code that becomes `RC`, `.RS` and a
// condition.
mod command;

// The condition object `CONDITION('O')` answers: the Directory a
// raise builds, whose indexes depend on the condition's own kind.
mod condition;

/// `ADDRESS ... WITH`: the permanent per-environment configuration, and the
/// per-command context that says where one command's three streams go.
mod redirect;

// The security manager (D12): the manager a package carries, and the one
// send each checkpoint makes to it.
mod security;

// The libraries a name has been resolved to, in a module of their own so
// that the map is private to the rule that a resolved name is never given up.
mod libraries;
use libraries::Libraries;

// What a program's directives say before any of them is installed.
mod directives;
use directives::{accessor_setter_name, accessor_variable, delegate_variable, method_body_gap};

// Variable storage and reads: a slot's own storage, what an `EXPOSE` bound it
// to, and the scope pools.
mod variables;

// Installing a program's directives, and the libraries and executable records
// they bind.
mod install;

/// The exit code for a construct this crate does not implement.
pub const NOT_IMPLEMENTED_EXIT: i32 = 120;

/// The status a run abandoned at its own deadline exits with
/// ([`Invocation::with_deadline`]).
pub const DEADLINE_EXIT: i32 = 121;

/// The stderr line a run abandoned at its deadline leaves behind.
pub const DEADLINE_REPORT: &[u8] = b"rexx-exec: the run exceeded its deadline\n";

/// The name the interpreter's own package answers to -- `PackageClass::
/// getProgramName`'s answer for internal code (`classes/PackageClass.hpp:147`).
pub(crate) const LIBRARY_PACKAGE_NAME: &[u8] = b"REXX";

/// The interpreter thread's stack, in bytes.
/// ```text
/// interpreter stack: 536870912 bytes, eval depth reached: 100000, span: 183998160 bytes, per frame: 1840.0 bytes
/// ```
/// ```text
/// interpreter stack: 536870912 bytes, eval depth reached: 100000, span: 47999520 bytes, per frame: 480.0 bytes
/// ```
pub const INTERPRETER_STACK_BYTES: usize = 512 * 1024 * 1024;

/// The arena size below which no ordinary run ever collects, and the floor
/// every later growth allowance is raised to (see `Interp::collect_at`).
const COLLECT_FLOOR: usize = 65_536;

/// The globals entry [`Interp::root_exit_value`] writes. A name rather than an
/// index because `RootSet::add_global` is keyed by name and replaces in place,
/// which is the behaviour wanted here.
const EXIT_VALUE_ROOT: &str = "the program's exit value";

/// What one interpreter run produced.
#[derive(Debug)]
pub struct Outcome {
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    /// What the run cost in stack. Task 11 reads this to set the
    /// evaluation-depth limit; see `StackSpan`.
    pub stack: StackSpan,
    /// How many times `Heap::collect` ran during this program.
    pub collections: u64,
    /// How many times the run declined to compile a body because it does not
    /// fit the compiled stream's index widths. Such a body raises
    /// [`Loud::chunk_refused`]; there is no second engine to run it.
    pub chunks_refused: usize,
}

/// How deep evaluation went and how much stack it took to get there.
#[derive(Copy, Clone, Debug, Default)]
pub struct StackSpan {
    /// The deepest `eval` recursion the run reached. Zero if it never
    /// evaluated an expression at all, and zero also for a run whose only
    /// values came from ops that do not enter `eval` -- see this type's own doc
    /// comment.
    pub max_depth: usize,
    /// Stack bytes between the first `eval` level and the deepest one, both
    /// on the chain that reached `max_depth`. Meaningless unless `max_depth`
    /// is above 1, which is what `bytes_per_frame` checks before dividing.
    pub bytes: usize,
}

impl StackSpan {
    /// Stack bytes one further level of `eval` costs, or `None` when the run
    /// never recursed and there is nothing to divide by.
    pub fn bytes_per_frame(self) -> Option<f64> {
        (self.max_depth > 1).then(|| self.bytes as f64 / (self.max_depth - 1) as f64)
    }
}

/// A construct this crate does not implement, on its way to becoming an exit
/// code and a line on stderr.
#[derive(Debug)]
struct Loud {
    message: String,
}

impl Loud {
    /// An instruction this crate does not execute. `keyword()` is `None` for the four
    /// clause shapes no keyword introduces, and their names come from the
    /// shape rather than from a keyword table.
    fn instruction(kind: &InstructionKind) -> Loud {
        let name = match kind {
            // The four clause shapes no keyword introduces.
            InstructionKind::Assignment { .. } => "an assignment",
            InstructionKind::Label { .. } => "a label",
            InstructionKind::Message { .. } => "a message send",
            InstructionKind::Command { .. } => "a command",
            // Everything else is named by the keyword that introduced it.
            InstructionKind::Address { .. }
            | InstructionKind::Arg { .. }
            | InstructionKind::Call { .. }
            | InstructionKind::Do { .. }
            | InstructionKind::Drop { .. }
            | InstructionKind::Else { .. }
            | InstructionKind::End { .. }
            | InstructionKind::Exit { .. }
            | InstructionKind::Expose { .. }
            | InstructionKind::Forward { .. }
            | InstructionKind::Guard { .. }
            | InstructionKind::If { .. }
            | InstructionKind::Interpret { .. }
            | InstructionKind::Iterate { .. }
            | InstructionKind::Leave { .. }
            | InstructionKind::Loop { .. }
            | InstructionKind::Nop
            | InstructionKind::Numeric { .. }
            | InstructionKind::Options { .. }
            | InstructionKind::Otherwise
            | InstructionKind::Parse { .. }
            | InstructionKind::Procedure { .. }
            | InstructionKind::Pull { .. }
            | InstructionKind::Push { .. }
            | InstructionKind::Queue { .. }
            | InstructionKind::Raise { .. }
            | InstructionKind::Reply { .. }
            | InstructionKind::Return { .. }
            | InstructionKind::Say { .. }
            | InstructionKind::Select { .. }
            | InstructionKind::Signal { .. }
            | InstructionKind::Then
            | InstructionKind::Trace { .. }
            | InstructionKind::Use { .. }
            | InstructionKind::When { .. }
            | InstructionKind::WhenCase { .. } => kind.keyword().unwrap_or("an instruction"),
        };
        Loud {
            message: owned_message(name, instruction_owner(kind)),
        }
    }

    /// An expression form this crate does not evaluate.
    fn expression(kind: &ExprKind) -> Loud {
        Loud {
            message: owned_message(&form_name(kind), expr_owner(kind)),
        }
    }

    /// A binary operator that no family of `Interp::apply_binary` claims -- an
    /// internal inconsistency, never a program error.
    fn binary_operator(op: Operator) -> Loud {
        Loud {
            message: format!("binary operator {op:?} has no implementation"),
        }
    }

    /// A call that resolved to a **builtin this crate runs nothing for**. The
    /// owner comes from the same table the exclusion itself does, so a
    /// delivered exclusion takes its blame with it.
    fn unresolved_call(name: &[u8]) -> Loud {
        const LIMIT: usize = 128;
        let shown = if name.len() > LIMIT {
            format!("{}...", String::from_utf8_lossy(&name[..LIMIT]))
        } else {
            String::from_utf8_lossy(name).into_owned()
        };
        let owner = rexx_inventory::builtins::owner_of(&shown);
        Loud {
            message: owned_message(&format!("routine \"{shown}\""), owner),
        }
    }

    /// A call that resolved to a routine one of the oracle's internal packages
    /// exports and this crate has no body for. Distinct from
    /// [`Loud::unresolved_call`] only in where the name and the owner come
    /// from; the message a program sees is the same shape.
    fn internal_routine(name: &[u8], owner: &'static str) -> Loud {
        let shown = String::from_utf8_lossy(name).into_owned();
        Loud {
            message: owned_message(&format!("routine \"{shown}\""), Some(owner)),
        }
    }

    /// A message sent to a value whose class this phase does not build, so
    /// there is no behaviour to resolve the name against at all.
    fn receiver_class(kind: &str) -> Loud {
        Loud {
            message: owned_message(&format!("a message send to {kind}"), Some("Phase 5")),
        }
    }

    /// An `ADDRESS ... WITH` redirection whose target this crate accepts in
    /// the grammar and does not yet build. `owner` is the phase that owes it,
    /// which is not always this one: a `RexxQueue` target waits on the queues
    /// themselves.
    fn redirection(what: &str, owner: &'static str) -> Loud {
        Loud {
            message: owned_message(&format!("an ADDRESS WITH {what} redirection"), Some(owner)),
        }
    }

    /// An operator whose **left** operand is an object this phase can build
    /// but send no message to: a class object, or one of the interpreter's
    /// own (`.environment`, `.local`, `.methods`, `.context`).
    fn operator_operand(op: &str, kind: &str) -> Loud {
        Loud {
            message: owned_message(
                &format!("the operator `{op}` applied to {kind}"),
                Some("Phase 5"),
            ),
        }
    }

    /// One of the objects [`Loud::operator_operand`] refuses, in a position
    /// that is not an operator's operand: a `DO` header's value, `DO OVER`'s
    /// target, a controlled loop's own control variable at the increment, or
    /// a `RAISE SYNTAX` clause's `ADDITIONAL` value.
    fn object_position(position: &str, kind: &str) -> Loud {
        Loud {
            message: owned_message(&format!("{kind} as {position}"), Some("Phase 5")),
        }
    }

    /// A `.NAME` the oracle's `.environment` or `.local` answers and this
    /// crate builds nothing for.
    fn environment_symbol(name: &[u8], owner: &'static str) -> Loud {
        let shown = String::from_utf8_lossy(name);
        Loud {
            message: owned_message(&format!("environment symbol \"{shown}\""), Some(owner)),
        }
    }

    /// A directory index the oracle's own `.environment` or `.local` has an
    /// entry for and this crate builds nothing for.
    fn environment_entry(index: &[u8], owner: &'static str) -> Loud {
        let shown = String::from_utf8_lossy(index);
        Loud {
            message: owned_message(&format!("directory entry \"{shown}\""), Some(owner)),
        }
    }

    /// A message that **resolved** to a primitive method this crate has no
    /// code for.
    fn native_method(name: &[u8], scope: &str) -> Loud {
        let shown = String::from_utf8_lossy(name);
        Loud {
            message: owned_message(
                &format!("method \"{shown}\" of class \"{scope}\""),
                Some("Phase 5"),
            ),
        }
    }

    /// An array subscript list whose only element is an empty slot, which the
    /// oracle answers by dying.
    fn array_index_hole() -> Loud {
        Loud {
            message: owned_message("an array subscript that is an empty slot", None),
        }
    }

    /// A `receiver~NAME=` entry-method send that carried no value argument.
    fn entry_method_without_a_value(index: &[u8]) -> Loud {
        let shown = String::from_utf8_lossy(index);
        Loud {
            message: owned_message(
                &format!("an entry-method assignment to \"{shown}\" with no value"),
                None,
            ),
        }
    }

    /// A collection this crate can name but cannot read: one whose entries
    /// the oracle has and this crate answers per name through
    /// [`Loud::environment_entry`] instead of building.
    fn unreadable_collection(owner: &'static str) -> Loud {
        Loud {
            message: owned_message(
                "a directory whose entries this crate does not fill",
                Some(owner),
            ),
        }
    }

    /// A library procedure that resolved when it was bound and no longer
    /// does -- an internal inconsistency, never a program error: a held
    /// library is never released.
    fn library_procedure_gone() -> Loud {
        Loud {
            message: "a library procedure that resolved when it was bound no longer resolves"
                .to_string(),
        }
    }

    /// `Package~options(name, value)` and `Package~defaultOptions(name,
    /// value)`, each of which writes a package setting rather than reading
    /// one.
    fn package_option_write() -> Loud {
        Loud {
            message: owned_message("a package settings write", Some("Phase 10")),
        }
    }

    fn method_from_source(what: &str) -> Loud {
        Loud {
            message: owned_message(what, Some("Phase 5")),
        }
    }

    /// A `SETMETHOD` or `UNSETMETHOD` whose receiver has no dictionary of
    /// its own here.
    fn object_method(what: &str) -> Loud {
        Loud {
            message: owned_message(what, Some("Phase 5")),
        }
    }

    /// A message that resolved to a `::METHOD` or `::ATTRIBUTE` directive
    /// whose body this crate cannot run -- see [`method_body_gap`], which
    /// enumerates the cases and supplies `what`.
    fn method_body(what: &str) -> Loud {
        Loud {
            message: owned_message(what, Some("Phase 5")),
        }
    }

    /// One of the interpreter's own embedded `.orx` sources will not parse.
    fn library_source(name: &str, error: &str) -> Loud {
        Loud {
            message: owned_message(
                &format!("{name} does not parse here: {error}"),
                Some("Phase 5"),
            ),
        }
    }

    /// A file a `::REQUIRES` found will not parse.
    fn required_source(path: &str, error: &str) -> Loud {
        Loud {
            message: owned_message(
                &format!("{path} does not parse here: {error}"),
                Some("Phase 5"),
            ),
        }
    }

    /// One of the two methods `Setup.cpp` puts on `.Class` for the image
    /// build and `removeSetupMethods` deletes, given something it cannot
    /// use.
    fn setup_method(what: &str) -> Loud {
        Loud {
            message: owned_message(what, Some("Phase 5")),
        }
    }

    /// `EXPOSE` in a method whose receiver is neither a class object nor an
    /// instance.
    fn expose_receiver() -> Loud {
        Loud {
            message: owned_message("EXPOSE on an object with no variable pool", Some("Phase 5")),
        }
    }

    /// `USE LOCAL` as a `::METHOD`'s first instruction, which is the one
    /// placement the oracle runs (measured, rc 0).
    fn use_local_in_a_method() -> Loud {
        Loud {
            message: owned_message("USE LOCAL in a ::METHOD body", Some("Phase 5")),
        }
    }

    /// `EXPOSE` or `PROCEDURE EXPOSE` naming a single compound tail.
    fn compound_expose(keyword: &str, name: &[u8]) -> Loud {
        Loud {
            message: format!(
                "{keyword} of the single compound tail \"{}\" is not implemented",
                String::from_utf8_lossy(name)
            ),
        }
    }

    /// A generated `::METHOD ATTRIBUTE`/`::ATTRIBUTE` accessor whose
    /// variable is a stem or a single compound tail.
    fn accessor_variable(name: &[u8]) -> Loud {
        Loud {
            message: format!(
                "a generated accessor for the attribute \"{}\" is not implemented",
                String::from_utf8_lossy(name)
            ),
        }
    }

    /// A `DELEGATE` whose variable is a stem or a single compound tail, the
    /// same storage gap [`Loud::accessor_variable`] refuses reached from a
    /// different directive.
    fn delegate_variable(name: &[u8]) -> Loud {
        Loud {
            message: format!(
                "a DELEGATE to the variable \"{}\" is not implemented",
                String::from_utf8_lossy(name)
            ),
        }
    }

    /// A builtin's option letter whose answer this crate cannot produce.
    fn builtin_option_object(routine: &str, option: u8, why: &str) -> Loud {
        Loud {
            message: format!(
                "{routine} option \"{}\" answers {why}, which is not implemented",
                option.escape_ascii()
            ),
        }
    }

    /// A `PARSE` template trigger that needs an operand and has none.
    fn parse_trigger_operand() -> Loud {
        Loud {
            message: "a PARSE template trigger carries no position operand".to_string(),
        }
    }

    /// An activation's body selector named something that is not a routine
    /// body -- an internal inconsistency, never a program error.
    fn missing_body() -> Loud {
        Loud {
            message: "an activation's body selector names no routine body".to_string(),
        }
    }

    /// A chunk's instruction map is shorter than the body it was compiled
    /// from -- an internal inconsistency, never a program error.
    fn chunk_map_too_short() -> Loud {
        Loud {
            message: "a compiled chunk has no op for an instruction of its own body".to_string(),
        }
    }

    /// A body the compiler refused.
    fn chunk_refused() -> Loud {
        Loud {
            message: "a body does not fit the compiled stream's index widths, and there is no \
                      longer a second engine to run it"
                .to_string(),
        }
    }

    /// The driver reached an op it has no arm for.
    fn op_not_driven(what: &'static str) -> Loud {
        Loud {
            message: format!("a compiled {what} op has no driver arm"),
        }
    }

    /// A compiled jump names an op past the end of the range it is running in
    /// -- an internal inconsistency, never a program error.
    fn jump_out_of_range() -> Loud {
        Loud {
            message: "a compiled jump leaves the range it is running in".to_string(),
        }
    }

    /// A register a branch op reads holds something that is not a Rexx
    /// logical value -- an internal inconsistency, never a program error.
    fn register_not_logical() -> Loud {
        Loud {
            message: "a compiled branch read a register holding no logical value".to_string(),
        }
    }

    /// A compiled `SELECT` op does not describe the `SELECT` it was emitted
    /// for -- an internal inconsistency, never a program error.
    fn select_op_off_its_node() -> Loud {
        Loud {
            message: "a compiled SELECT op does not name a SELECT of its own body".to_string(),
        }
    }

    /// A compiled `DO`/`LOOP` op does not describe the loop it was emitted for
    /// -- an internal inconsistency, never a program error.
    fn loop_op_off_its_node() -> Loud {
        Loud {
            message: "a compiled DO/LOOP op does not name a DO/LOOP of its own body".to_string(),
        }
    }

    /// A compiled `Signal` op carries an operand its instruction's own form
    /// does not have, or lacks the one it does -- an internal inconsistency,
    /// never a program error.
    fn signal_op_off_its_node() -> Loud {
        Loud {
            message: "a compiled SIGNAL op does not match the form of the SIGNAL it names"
                .to_string(),
        }
    }

    /// A compiled `Store` op does not describe the assignment it was emitted
    /// for -- an internal inconsistency, never a program error.
    fn store_op_off_its_node() -> Loud {
        Loud {
            message: "a compiled Store op does not name an assignment of its own body".to_string(),
        }
    }

    /// A compiled op that runs or traces a call does not describe the call it
    /// was emitted for -- an internal inconsistency, never a program error.
    fn call_op_off_its_node() -> Loud {
        Loud {
            message: "a compiled call op does not name a call of its own body".to_string(),
        }
    }

    /// A compiled `Const` op names a constant its own chunk does not carry --
    /// an internal inconsistency, never a program error.
    fn constant_out_of_range() -> Loud {
        Loud {
            message: "a compiled Const op names no constant of its own chunk".to_string(),
        }
    }

    /// A `GUARD ... WHEN` whose expression is false.
    fn guard_when_false() -> Loud {
        Loud {
            message: "a GUARD that has to wait for another activity to make its WHEN \
                      expression true is not implemented (Phase 6)"
                .to_string(),
        }
    }

    /// `Message~result` on a message whose send has not been made.
    fn unsent_message_result() -> Loud {
        Loud {
            message: "`Message~result` on a message whose send has not been made is not \
                      implemented (Phase 6)"
                .to_string(),
        }
    }

    /// A `REPLY` that is not a clause of its method body's own top level.
    fn reply_inside_construct() -> Loud {
        Loud {
            message: "a REPLY inside a DO, SELECT or IF is not implemented (Phase 6)".to_string(),
        }
    }

    // **There is no `Loud::parse`, and its absence is the fix.** A fragment
    // that does not parse raises the oracle's own 27.901 at rc 229, through
    // `impl From<&ParseError> for Raised` (`error.rs`), which `run_fragment`
    // uses. What the *top level* can and cannot take from it is written out
    // at `execute`'s own parse arm, below.
}

/// Names an expression form in **bounded** text, for a loud failure to quote.
fn form_name(kind: &ExprKind) -> String {
    let name = match kind {
        ExprKind::Literal(_) => "a literal",
        ExprKind::Constant(_) => "a constant symbol",
        ExprKind::Variable(_) => "a simple variable",
        ExprKind::Stem(_) => "a stem",
        ExprKind::Compound(_) => "a compound variable",
        ExprKind::DotVariable(_) => "an environment symbol",
        // The two operator forms name the operator, because "a dyadic
        // operator is not implemented" does not tell a reader which one to
        // go and implement. Each asks its own operator type for the spelling,
        // which is where the canonical bytes live and what the trace line for
        // that operator carries.
        ExprKind::Prefix { op, .. } => {
            return format!("the prefix operator `{}`", op.spelling());
        }
        ExprKind::Binary { op, .. } => {
            return format!("the operator `{}`", op.spelling());
        }
        ExprKind::Call { .. } => "a function call",
        ExprKind::QualifiedCall { .. } => "a namespace-qualified call",
        ExprKind::ClassResolver { .. } => "a namespace-qualified class lookup",
        ExprKind::Message { .. } => "a message send",
        ExprKind::List(_) => "a parenthesised list",
        ExprKind::Logical(_) => "a comma list in a condition",
        ExprKind::VariableReference(_) => "a variable reference",
    };
    name.to_string()
}

/// The `REXX` and `REXXUTIL` packages' routine names and the phase owing each
/// a body, consulted between the running package's `::ROUTINE`s and the
/// external file search.
pub(crate) mod internal_routines;

/// Turning a name a program wrote into the path this interpreter opens, against
/// the interpreter's own current directory rather than the process's.
pub(crate) mod paths;

/// Every internal-package routine name, for the test that re-derives them from
/// the C++ tree.
pub fn internal_routine_names() -> Vec<&'static str> {
    internal_routines::INTERNAL_ROUTINES
        .iter()
        .map(|row| row.name)
        .collect()
}

/// Every internal-package routine as `(name, package, owner)`, for the test
/// that re-derives the two packages from the C++ tree separately. The owner
/// is `None` for a row this crate runs.
pub fn internal_routine_rows() -> Vec<(&'static str, &'static str, Option<&'static str>)> {
    internal_routines::INTERNAL_ROUTINES
        .iter()
        .map(|row| (row.name, row.package.label(), row.owner))
        .collect()
}

/// Appends the owner phase to a loud message, `"{name} is not implemented
/// ({owner})"`, or leaves it unsuffixed (`"{name} is not implemented"`) when
/// [`instruction_owner`]/[`expr_owner`] answer `None` -- meaning "this crate
/// implements that variant", not "the owner is some particular phase".
fn owned_message(name: &str, owner: Option<&'static str>) -> String {
    match owner {
        None => format!("{name} is not implemented"),
        Some(owner) => format!("{name} is not implemented ({owner})"),
    }
}

/// Who is responsible for an `InstructionKind` this crate does not implement,
/// spelled exactly as the split table spells it
/// (`docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md`, "The
/// split") -- `None` for a variant this crate implements (see
/// [`owned_message`]'s doc for why that is `None` and not a `"4a"` string).
fn instruction_owner(kind: &InstructionKind) -> Option<&'static str> {
    match kind {
        InstructionKind::Assignment { .. }
        | InstructionKind::Label { .. }
        | InstructionKind::Do(_)
        | InstructionKind::Loop(_)
        | InstructionKind::If { .. }
        | InstructionKind::Then
        | InstructionKind::Else { .. }
        | InstructionKind::Select { .. }
        | InstructionKind::When { .. }
        | InstructionKind::WhenCase { .. }
        | InstructionKind::Otherwise
        | InstructionKind::Leave { .. }
        | InstructionKind::Iterate { .. }
        | InstructionKind::End { .. }
        | InstructionKind::Drop { .. }
        | InstructionKind::Say { .. }
        | InstructionKind::Exit { .. }
        | InstructionKind::Numeric { .. }
        | InstructionKind::Trace(_)
        | InstructionKind::Interpret { .. }
        // A `RETURN` in the main body is not a gap either: measured, it ends
        // the program with its value exactly as `EXIT` does.
        | InstructionKind::Return { .. }
        | InstructionKind::Nop => None,
        // Every arm is implemented, so any owner string here would be a false
        // statement in a table whose only job is to be true --
        // `Loud::instruction` is not reached for any of them, and an owner
        // string nothing reads is exactly how the third copy of this data
        // drifts. A named call that resolves to no internal label, no builtin
        // and no `::ROUTINE` raises the oracle's own 43.1 rather than failing
        // loudly, and the external file search behind those three runs. So
        // there is no residual claim on the `CALL` keyword here at all.
        InstructionKind::Call(_) => None,
        // `Use` is `None` even
        // though `USE LOCAL` can only ever fail here: it fails with the
        // oracle's own two errors (98.993/99.910), measured, which is an
        // implemented instruction answering the same bytes the oracle
        // answers -- not a gap. The one shape inside `Procedure` this crate
        // cannot express, `expose a.1`, fails loudly through
        // `Loud::compound_expose` rather than through this table, because it
        // is a sub-case of a variant and this table is per variant.
        InstructionKind::Procedure { .. } | InstructionKind::Use(_) => None,
        // All three `Signal` arms are implemented, so unlike
        // `Call` above this one needs no arm-grained match. `RAISE` needs none
        // either, and the shape that would have forced one is
        // `ADDITIONAL <array>`: measured on both engines, three descriptors,
        // `raise syntax 40.4 additional (1,,3)` and `... array (1,,3)` are
        // byte-identical to each other and to the oracle, because
        // `requestArray` answers an array unchanged and both spellings
        // therefore build the same substitution list. `ADDITIONAL`'s one shape
        // with no code here -- a `SYNTAX` condition whose value is a class
        // object or one of the interpreter's own -- fails loudly through
        // `Loud::object_position` rather than through this table, which is
        // where `Expose`'s own two sub-cases are refused too.
        InstructionKind::Signal(_) | InstructionKind::Raise(_) => None,
        // Both keywords are whole: `queue.rs`
        // stores every line either writes, and neither has a shape this
        // crate cannot express the way `Procedure`'s `expose a.1` does.
        InstructionKind::Push { .. } | InstructionKind::Queue { .. } => None,
        // All three spellings of the one instruction: `PARSE`, and the `ARG`
        // and `PULL` short forms, which `rexx-parse` gives variants of their
        // own but which share `exec_parse`. Every source is implemented,
        // including the two that read a line (`PARSE PULL`, `PARSE LINEIN`).
        InstructionKind::Parse(_) | InstructionKind::Arg(_) | InstructionKind::Pull(_) => None,
        // Every form is implemented, the `WITH` redirection included: the
        // one shape with no code here is a `RexxQueue` target, which fails
        // loudly through `Loud::redirection` rather than through this table.
        InstructionKind::Address(_) => None,
        // A message send as a whole clause: `exec_message` (`run.rs`)
        // evaluates it and settles `RESULT`. `None` in the same sense
        // `DotVariable` is `None` below -- the variant is implemented, and
        // the sub-cases with no code here fail loudly through `Loud::
        // receiver_class`/`Loud::native_method` rather than answering.
        InstructionKind::Message { .. } => None,
        // `EXPOSE` binds its names to the receiving object's scope pool.
        // `None` in the same sense `Message` above is: the variant executes,
        // and the two sub-cases with no code here -- a single compound tail,
        // and a receiver that is not a class object -- fail loudly through
        // `Loud::compound_expose`/`Loud::expose_receiver` rather than
        // answering.
        InstructionKind::Expose { .. } => None,
        // `GUARD` reserves and releases the receiver's scope and `REPLY`
        // hands its value to the sender and leaves the rest of the body
        // owed. `None` in the same sense `Expose` above is: both variants
        // execute, and the sub-cases with no code here -- a `GUARD ... WHEN`
        // that is false and so has to wait, a `REPLY` under a construct --
        // fail loudly through `Loud::guard_when_false`/
        // `Loud::reply_inside_construct` rather than answering.
        InstructionKind::Guard(_) | InstructionKind::Reply { .. } => None,
        // `FORWARD` is `None` in the sense `Guard` and `Reply` above are: the
        // instruction executes and every option is built, and the one
        // sub-case with no code -- an `ARGUMENTS` value whose conversion to a
        // single-dimensional array this crate does not build -- fails loudly
        // through `Loud::object_position` rather than answering.
        InstructionKind::Forward(_) => None,
        InstructionKind::Options { .. } => Some("Phase 5"),
        InstructionKind::Command { .. } => None,
    }
}

/// [`instruction_owner`]'s counterpart for `ExprKind`. See that function's
/// own doc for why this is a third copy of `owners.rs`'s `EXPR_TAGS` (there,
/// `EXPR_TAGS`), and for the completeness guarantee the exhaustive match
/// below carries.
fn expr_owner(kind: &ExprKind) -> Option<&'static str> {
    match kind {
        ExprKind::Literal(_)
        | ExprKind::Constant(_)
        | ExprKind::Variable(_)
        | ExprKind::Stem(_)
        | ExprKind::Compound(_)
        | ExprKind::DotVariable(_)
        | ExprKind::Prefix { .. }
        | ExprKind::Binary { .. }
        | ExprKind::Logical(_) => None,
        // **`ExprKind::Call` is `None`, not an owner string.** It has
        // exactly two `CallTarget` forms and this crate evaluates both, so it
        // closes outright. A
        // name that resolves to no internal label (or a `CallTarget::
        // Literal`, which never searches labels at all), no builtin and no
        // `::ROUTINE` raises the oracle's own 43.1 -- exactly the same shape
        // `InstructionKind::Call`'s own comment above describes for `CALL`.
        // `>name`/`<name` answers a `VariableReference`, which `eval.rs`'s
        // own arm builds and `run/call.rs`'s `Interp::variable_reference` binds to
        // the variable.
        ExprKind::Call { .. }
        | ExprKind::VariableReference(_)
        | ExprKind::Message { .. }
        | ExprKind::List(_)
        | ExprKind::QualifiedCall { .. }
        | ExprKind::ClassResolver { .. } => None,
    }
}

/// The code a step is executing, all of it borrowed from the caller's local
/// `Rc` and none of it from `self`.
struct Code<'a> {
    body: &'a CodeBody,
    symbols: &'a SymbolTable,
    /// Slot per `SymbolId` **of this code's own table**. For a fragment those
    /// ids are the fragment's, resolved against the enclosing frame, which is
    /// why this is a field of `Code` rather than something read back off the
    /// activation.
    slots: &'a [Option<usize>],
    /// The upfront pass's answers about **this** body: its clause indents
    /// (`Plan::indents`), the line each of its clauses sits on
    /// (`Plan::lines`), and how each of its compound names splits
    /// (`Plan::compounds`).
    plan: Option<&'a Plan>,
}

impl<'a> Code<'a> {
    /// The slot this body's upfront pass bound `id` to, or `None` when it
    /// bound it none.
    pub(crate) fn slot_for(&self, id: SymbolId) -> Option<usize> {
        let at = self.slots.get(id.index()).copied().flatten();
        debug_assert!(
            at.is_none()
                || self.plan.is_none_or(|plan| {
                    plan.names.get(self.symbols.name(id).as_bytes()).copied() == at
                }),
            "the plan's slot for {} is {at:?}, which its own name map does not agree with",
            self.symbols.name(id),
        );
        at
    }

    /// How the compound `id` names splits, from the upfront pass when this
    /// body had one, and by splitting the interned spelling when it did not.
    fn compound(&self, id: SymbolId) -> Option<&'a CompoundName> {
        self.plan?.compound(id)
    }

    /// The stem half of the compound `id` names, its trailing period
    /// included -- the name whose slot holds the stem object a tail is read
    /// out of or written into -- **and that slot, when the entry carries
    /// one**.
    fn stem(&self, id: SymbolId) -> (&'a [u8], Option<usize>) {
        match self.compound(id) {
            Some(entry) => (&entry.stem, entry.stem_at),
            None => (compound_parts(self.symbols.name(id)).0.as_bytes(), None),
        }
    }
}

/// A program's main body as a `Code`, carrying the plan the upfront pass
/// builds for it -- which is what `run.rs` hands every step.
#[cfg(test)]
fn planned_code<'a>(program: &'a Program, plan: &'a Plan) -> Code<'a> {
    Code {
        body: &program.main,
        symbols: &program.symbols,
        slots: &plan.by_symbol,
        plan: Some(plan),
    }
}

/// Whether a variable read found a value or derived one from the name.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Novalue {
    Set,
    Unset,
}

/// The condition a running handler was entered for, kept so `RAISE
/// PROPAGATE` can re-raise it.
/// ```text
///      8 *-*   say 1/0          <- line 8 raised; line 12 propagated
///      3 *-* call fun
/// Error 42:  Arithmetic overflow/underflow.
/// ```
struct ActiveCondition {
    raised: Raised,
    site: Option<FailureSite>,
    sites: Vec<FailureSite>,
}

/// A condition waiting for the current clause to finish before its `CALL ON`
/// handler runs.
struct PendingTrap {
    condition: Box<[u8]>,
    rc: Option<Vec<u8>>,
    /// `RAISE ... DESCRIPTION`'s rendered value, held for the same reason
    /// `rc` is: the raising clause's temps frame is long gone by the time
    /// the handler reads it back through `CONDITION('D')`.
    description: Option<Vec<u8>>,
    /// `CONDITION('O')`'s directory, built when this was **queued** rather
    /// than when the handler runs. Everything in it is a raise-time fact that
    /// delivery cannot recover: `POSITION` is the raising clause's own line
    /// (measured 7 for a command inside a routine, not the caller's `call`
    /// line), `STACKFRAMES` is the stack as it stood then, and `RESULT` is
    /// carried by a command's condition and by no other.
    ///
    /// An `ObjRef` reachable only through this queue, so
    /// [`Interp::object_roots`] names it: that destructure guards `Interp`'s
    /// own fields and would not have caught one added inside a `VecDeque`.
    object: Option<ObjRef>,
    /// The activation this may be delivered to: the raising activation's
    /// **caller**, which is the one whose trap table matched.
    activation: ActivationId,
    /// Whether this was queued by a **handler** running at a clause boundary
    /// rather than by that clause's own work.
    queued_during_delivery: bool,
    /// [`Interp::fragment_depth`] as it stood when this was queued: which
    /// `INTERPRET` fragment, if any, was running.
    fragment_depth: usize,
}

/// The interpreter. Owns the heap, the root set, the activation stack, the
/// plan cache and the two sinks, and **does not own the AST**.
struct Interp {
    heap: Heap,
    roots: RootSet,
    /// A buffer lent out for building a compound's tail key, and handed back.
    key_buffer: Vec<u8>,
    /// A buffer lent out for a builtin call's evaluated argument values.
    value_buffer: Vec<Option<ObjRef>>,
    /// Buffers lent out for a `PARSE` instruction's source strings, and
    /// handed back when the template walk is done with them.
    parse_buffers: Vec<Vec<u8>>,
    /// Where an inline string's bytes, or a tagged integer's rendering, are
    /// put so that [`Interp::to_text`] can hand back a borrow of them.
    text_scratch: [u8; crate::value::TEXT_SCRATCH],
    /// The parse cache a handle-inline string has nowhere to keep. See
    /// `value::TextNumbers`, which owns the rule and the measurement.
    text_numbers: crate::value::TextNumbers,
    /// A buffer lent out for building a builtin's result, and handed back.
    result_buffer: std::cell::Cell<Vec<u8>>,
    /// The environment this interpreter reads and writes, initialised from the
    /// process and never written back. `std::env::set_var` is `unsafe` in this
    /// edition, and the test harnesses run interpreters on threads in one
    /// process, so a write reaching the process would reach every other run.
    env: Vec<(Vec<u8>, Vec<u8>)>,
    /// The directories `LD_LIBRARY_PATH` named in the environment this
    /// interpreter was started with, which a bare library name is looked for
    /// in before the undecorated `dlopen`.
    ///
    /// Taken once, as the process loader takes the variable once: measured,
    /// oracle rc 0, a program that writes a directory holding a copy of the
    /// library into `LD_LIBRARY_PATH` through `VALUE` still loads nothing
    /// from it.
    library_search: Vec<std::path::PathBuf>,
    /// The directory relative paths resolve against, for the same reason:
    /// `std::env::set_current_dir` is process-wide. `DIRECTORY()` moves this
    /// and nothing else.
    cwd: std::path::PathBuf,
    /// What each outstanding `SETLOCAL` saved, innermost last: the directory
    /// and the whole environment, which `ENDLOCAL` puts back. The oracle keeps
    /// this on the top-level activation and an internal routine's `SETLOCAL`
    /// therefore outlives its return (`platform/unix/ExternalFunctions.cpp`,
    /// and `funct.xml` says otherwise -- measured, the file is wrong).
    locals: Vec<(std::path::PathBuf, Vec<(Vec<u8>, Vec<u8>)>)>,
    /// The activation running right now, held in a field of its own rather
    /// than at the top of [`Interp::suspended`].
    running: Option<Box<Activation>>,
    /// The `TRACE` setting of whatever [`Interp::running`] holds, kept beside
    /// it rather than read through it.
    trace_cache: crate::trace::TraceCache,
    /// The activations that entered before [`Interp::running`], oldest
    /// first, so `suspended.last()` is the running activation's own caller.
    #[expect(
        clippy::vec_box,
        reason = "the box is the same one `running` holds, so suspending and \
                  resuming move a pointer instead of the activation"
    )]
    suspended: Vec<Box<Activation>>,
    /// Boxes whose activations have ended, kept for the next push rather than
    /// returned to the allocator. `Interp::recycle_activation` is what fills
    /// it and `Interp::push_activation` what drains it; both carry the
    /// reasoning.
    #[expect(
        clippy::vec_box,
        reason = "the box is the allocation being kept, so unboxing here would \
                  return the very thing this parks"
    )]
    spare_activations: Vec<Box<Activation>>,
    /// The next [`ActivationId`] to hand out. Monotonic, never reset, never
    /// reused -- see that type for the two defects that needed an identity a
    /// stack depth could not supply.
    next_activation_id: u64,
    /// The counter `RexxContext~invocation` mints from --
    /// `RexxActivation.cpp:94`'s file-scope `counter`.
    next_invocation: u32,
    /// Every program the loader has issued an id for, indexed by that id.
    programs: Vec<Rc<Program>>,
    /// What each program's `::OPTIONS` directives left on its package,
    /// entered by `Interp::install_directives` and read by every activation
    /// of that program's code.
    package_options: HashMap<ProgramId, PackageOptions>,
    /// The security manager each package carries, written by all three
    /// `setSecurityManager` setters and read by every checkpoint.
    security_managers: HashMap<ProgramId, ObjRef>,
    /// **`NameHasher` and not `RandomState`**, for the reason that alias's own
    /// doc gives and with the same shape of key behind it: a `BodyKey` is a
    /// pair of small integers this interpreter mints itself, so the
    /// chosen-collision resistance `RandomState` buys is resistance to an
    /// input nothing outside can choose, and SipHash's setup dominates a key
    /// this short. Measured on a probe whose loop is one `CALL` into a label
    /// that returns at once, the two lookups here and on `chunks` cost 9.7% of
    /// the program between them.
    plans: NameMap<BodyKey, Rc<Plan>>,
    /// The wall-clock bound `Interp::count_clause_against_deadline` honours,
    /// or `None` for the unbounded run every shipped caller asks for.
    deadline: Option<crate::clause::Deadline>,
    /// Clauses left before `Interp::countdown_reached` runs.
    clause_countdown: u32,
    /// The chunk cache (Phase 4e): D16's discipline applied to a second cache
    /// rather than invented afresh for it, under `plans`' own `BodyKey`
    /// **paired with the trace setting the chunk was compiled under**.
    chunks: NameMap<(BodyKey, ChunkTrace), Rc<crate::ir::Chunk>>,
    /// How many times `chunk_for` has refused a body because it does not fit
    /// the index widths the compiled stream commits to (`ChunkTooLarge`) --
    /// never because a body contains a construct the compiler does not
    /// know, which does not exist (D21: every instruction compiles).
    chunks_refused: usize,
    /// Method bodies a `REPLY` has left owed, oldest first.
    deferred: std::collections::VecDeque<crate::activation::DeferredReply>,
    /// Every `::ROUTINE` a program installs, keyed by the program and then by
    /// the routine's **upcased** name, holding its index in
    /// `Program::directives`.
    routines: HashMap<ProgramId, HashMap<Box<[u8]>, InstalledRoutine>>,
    /// The subset of [`routines`] a `::ROUTINE ... PUBLIC` filed -- the
    /// oracle's `publicRoutines`, a second table beside `routines` exactly as
    /// [`package_public_classes`] is beside [`package_classes`].
    package_public_routines: HashMap<ProgramId, HashMap<Box<[u8]>, InstalledRoutine>>,
    /// The routines a program imported -- `mergedPublicRoutines`: every
    /// `::REQUIRES ... LIBRARY`'s library routines, then each `::REQUIRES`'s
    /// public and imported routines, a name keeping its first entry
    /// (`PackageClass::mergeLibrary` and `::mergeRequired`,
    /// `classes/PackageClass.cpp:693-772`).
    merged_public_routines: HashMap<ProgramId, HashMap<Box<[u8]>, MergedRoutine>>,
    /// The public classes a program's `::REQUIRES` directives imported --
    /// `mergedPublicClasses`, merged alongside the routines above and read by
    /// `PackageClass::findClass` between the package's own installed classes
    /// and `.local`.
    merged_public_classes: NameMap<ProgramId, NameMap<Box<[u8]>, ObjRef>>,
    /// `.NAME` answers [`Interp::rexx_package_class`] has already found,
    /// keyed by the bare uppercased name.
    rexx_class_cache: NameMap<Box<[u8]>, ObjRef>,
    /// The packages a program's `::REQUIRES ... NAMESPACE` directives
    /// registered, under the upcased qualifier -- `PackageClass::addNamespace`
    /// (`classes/PackageClass.cpp:2152`), whose key is `name->upper()`.
    package_namespaces: HashMap<ProgramId, HashMap<Box<[u8]>, Package>>,
    /// The `Directory` `Package~local` answers, per package, built on first
    /// ask -- `PackageClass::getPackageLocal` (`classes/PackageClass.cpp:2131`
    /// region), which creates it lazily too.
    package_locals: HashMap<Package, ObjRef>,
    /// The object model message dispatch resolves against: `Setup.cpp`'s
    /// native classes, whatever `::CLASS`/`::METHOD`/`::ATTRIBUTE` have
    /// installed beside them, and the primitive methods this crate
    /// implements. `rexx-classes`' own
    /// [`rexx_classes::ClassRegistry`] is the one class model here (R9), not
    /// one of two.
    object_model: Option<dispatch::ObjectModel>,
    /// `.environment`, `.local` and what `.context`/`.methods` are built from
    /// (D33). `None` until a `.NAME` is resolved, for the reason
    /// [`Interp::object_model`] is: building it forces the native class set.
    environment: Option<environment::EnvironmentModel>,
    /// The arena object holding each class object's own variable pools, by
    /// class identity.
    class_variables: HashMap<ObjRef, ObjRef>,
    /// The classes each program's own `::CLASS` directives installed, keyed by
    /// the uppercased name -- `PackageClass`'s installed-class table, which
    /// `.NAME` resolution consults ahead of `.local` and `.environment`.
    package_classes: NameMap<ProgramId, NameMap<Box<[u8]>, ObjRef>>,
    /// The subset of [`package_classes`] a `::CLASS ... PUBLIC` directive or
    /// `~addPublicClass` filed -- the oracle's `installedPublicClasses`,
    /// which is a second table beside `installedClasses` and not a flag on
    /// the entries of one (`classes/PackageClass.cpp:1406`-`:1419`).
    /// `~publicClasses` is what reads it.
    package_public_classes: NameMap<ProgramId, NameMap<Box<[u8]>, ObjRef>>,
    /// Which program's `::CLASS` directive created a class -- the other
    /// direction of [`package_classes`], which `~package` reads.
    class_packages: HashMap<ObjRef, ClassPackage>,
    /// The one empty argument list every call that has none shares.
    empty_arguments: Rc<[Option<ObjRef>]>,
    /// The package objects `~package` answers, keyed by the package itself
    /// -- see [`crate::plan::Package`] for why the interpreter's own is a
    /// variant rather than an absent program id.
    package_objects: HashMap<Package, ObjRef>,
    /// The one `Routine` object standing for a program's own main section --
    /// what `RexxContext~executable` answers from a `PROGRAM` or
    /// `INTERNALCALL` context.
    program_routine_objects: HashMap<ProgramId, ObjRef>,
    /// The `.METHODS`/`.ROUTINES`/`.RESOURCES` tables, keyed by the program
    /// whose directives fill them and by which of those names it answers.
    package_tables: HashMap<(ProgramId, environment::PackageTable), ObjRef>,
    /// The one `Routine` object standing for each installed routine.
    routine_objects: HashMap<InstalledRoutine, ObjRef>,
    /// The one `Routine` object standing for each library routine an
    /// imported-routine table holds, by its [`Interp::library_codes`] row:
    /// `mergeLibrary` merges the library package's own routine objects, so
    /// every package importing one answers the same object.
    library_routine_objects: HashMap<usize, ObjRef>,
    /// The packages each program has imported, in the order they were added
    /// -- `PackageClass`'s `loadedPackages`, which `~importedPackages`
    /// answers a copy of.
    package_imports: HashMap<ProgramId, Vec<Package>>,
    /// The package that built each program, for one an executable created
    /// rather than one loaded in its own right: `newFile`'s and
    /// `Package~new`'s parent context. A routine lookup that misses a
    /// program's own table and its imports walks this.
    package_parents: HashMap<ProgramId, Package>,
    /// The value each `::CONSTANT` accessor answers, keyed by the directive
    /// that declared it.
    constant_values: HashMap<(ProgramId, usize), ObjRef>,
    /// The `StringTable` each annotated thing's `~annotations` answers, keyed
    /// by the thing -- see [`environment::Annotated`] for the key space and
    /// [`Interp::annotation_table`] for why the table is kept rather than
    /// rebuilt.
    annotations: HashMap<environment::Annotated, ObjRef>,
    /// How many methods have been compiled from source text, which is what
    /// [`environment::Annotated::Compiled`] counts -- see that variant for
    /// why a compiled method's annotation table is keyed by a count and not
    /// by the dictionary entry it is installed into.
    compiled_methods: usize,
    /// The `Method` object `Class~method` answers, keyed by the class and the
    /// instance dictionary name -- see [`Interp::method_object`] for the two
    /// oracle answers that make one object per entry observable.
    method_objects: HashMap<(ObjRef, Box<[u8]>), ObjRef>,
    /// Which `(program, directive)` a [`rexx_classes::MethodId`] `install_directives`
    /// minted names -- the "bodies are stored" half of R9, addressed by the
    /// same identity `ClassRegistry::add_instance_method`/`add_class_method`
    /// already returns.
    method_bodies: NameMap<MethodId, InstalledMethodBody>,
    /// Whether the interpreter's own Rexx-written library is running.
    library_bootstrap: bool,
    /// How many collections the heap had performed when the library
    /// bootstrap finished, which is what `Outcome::collections` is counted
    /// from -- see `Interp::bootstrap_library` for why the boundary is
    /// there and not at process start.
    collections_before_program: u64,
    /// The programs the library bootstrap loaded, in load order.
    library_programs: Vec<ProgramId>,
    /// The name a program compiled from method source text reports under.
    compiled_method_names: HashMap<ProgramId, Box<[u8]>>,
    /// Whether any object has been given a method of its own, which is what
    /// keeps the per-object dictionary off a send's path in a program that
    /// never sends `SETMETHOD` -- see `Interp::own_method_entry`.
    object_methods: bool,
    /// Which directive is the body of a `Method` object this crate handed
    /// out through `.METHODS` or compiled from source text.
    table_method_bodies: NameMap<ObjRef, InstalledMethodBody>,
    /// What each `Method` and `Routine` object this crate has handed out
    /// reports on -- see [`ExecutableSource`], which carries why this is not
    /// [`Interp::table_method_bodies`] with more rows in it.
    executable_sources: HashMap<ObjRef, ExecutableRecord>,
    /// What `Method`'s four setters have written on each object they have
    /// been sent to, over what its directive declared -- see
    /// [`dispatch::executable::MethodFlagWrites`].
    method_flag_writes: HashMap<ObjRef, dispatch::executable::MethodFlagWrites>,
    /// What the send behind each `Message` object this crate has built ended
    /// with -- `MessageClass`'s `flagResultReturned` and `flagRaiseError`
    /// (`classes/MessageClass.hpp:61`-`:62`) and its `condition` field (`:135`).
    message_outcomes: HashMap<ObjRef, Option<Box<Raised>>>,
    /// The methods a directive implements itself -- see [`GeneratedMethod`]
    /// for why these are not rows of [`method_bodies`], which is a
    /// measurement rather than a taxonomy.
    generated_methods: HashMap<MethodId, GeneratedMethod>,
    /// Which `LIBRARY REXX` entry point each `::METHOD ... EXTERNAL` bound
    /// to, keyed by the identity [`Interp::install_one_method`] minted for
    /// its dictionary key.
    native_externals: HashMap<MethodId, &'static dispatch::native::NativeExternal>,
    /// The native activations on the stack, innermost last. Each carries the
    /// objects an extension's handles name (D5), rooted by
    /// [`Interp::object_roots`] for exactly as long as the frame is on this
    /// stack, so a handle that outlives its activation resolves to nothing.
    native_handles: Vec<NativeFrame>,
    /// Frames popped off [`Interp::native_handles`] with the buffers that held
    /// objects cleared, whose allocations the next native call reuses.
    native_spares: Vec<NativeFrame>,
    /// Every object an extension made a global reference, which a handle to
    /// it resolves to from any native call for as long as the interpreter
    /// runs (`InterpreterInstance::addGlobalReference`).
    global_references: rexx_api::handles::Table,
    /// The terminated copies `StringData` and its kin answered, by the object
    /// they copy. Not roots: a collection drops the copy of an object it
    /// frees, and a handle-carried value, which no collection frees, keeps
    /// its copy as [`Interp::kept_holders`] says.
    kept_strings: std::collections::HashMap<ObjRef, Box<[u8]>>,
    /// The terminated copies of message and routine names answered, by name.
    kept_names: std::collections::HashMap<Box<[u8]>, Box<[u8]>>,
    /// For each handle-carried value in [`Interp::kept_strings`], how many
    /// native calls in flight asked for its copy. The copy goes when the
    /// last of them ends and no global reference holds the value.
    kept_holders: std::collections::HashMap<ObjRef, usize>,
    /// Every native library a name has loaded, by the name it was resolved
    /// under -- `PackageManager::packages`
    /// (`interpreter/package/PackageManager.cpp:229-248`). A miss is not held,
    /// because the oracle removes the package on a failed load and asks again;
    /// a library whose version check refused is, because that check raises
    /// between the `put` and the `remove`.
    libraries: Libraries,
    /// The thread context every native call and package hook is handed,
    /// which an extension may keep for as long as this interpreter runs.
    thread: rexx_api::ffi::ThreadContext,
    /// How many times [`Interp::resolve_library`] has asked `load::open` for a
    /// name nothing held, whether or not a library loaded, which is what a
    /// test reads to see that a held one is not asked for again.
    #[cfg(test)]
    library_open_attempts: usize,
    /// Which library procedure each `::METHOD`/`::ATTRIBUTE ... EXTERNAL
    /// "LIBRARY <name>"` bound to, keyed by the identity
    /// [`Interp::install_one_method`] minted for its dictionary key.
    library_externals: HashMap<MethodId, LibraryBinding>,
    /// Which package declared each `EXTERNAL` binding, keyed the same way.
    /// A raise the native boundary makes for itself is reported against this
    /// package rather than against the running program.
    external_packages: HashMap<MethodId, ProgramId>,
    /// The package of each library procedure's shared code object, `None`
    /// until a directive binds it, indexed by [`ExecutableSource::Loaded`].
    ///
    /// `NativeCode::setPackageObject` (`execution/NativeCode.cpp:130-140`)
    /// sets the package in place on the first binding and copies on every
    /// later one, so an object a `loadExternal*` send answered earlier
    /// reports the first binder's package from then on. A `::ROUTINE`
    /// directive discards that copy and stores the shared routine
    /// (`parser/DirectiveParser.cpp:2691-2693`), so a later routine binder's
    /// own routine reports the first binder too.
    library_codes: Vec<Option<ProgramId>>,
    /// Which [`Interp::library_codes`] row each procedure owns.
    library_code_rows: HashMap<LibraryCodeKey, usize>,
    /// The procedure each [`Interp::library_codes`] row is, by row.
    library_code_keys: Vec<LibraryCodeKey>,
    /// The [`Interp::library_codes`] row each library-backed `::ROUTINE`
    /// directive bound, which is the package its routine reports.
    library_routine_codes: HashMap<InstalledRoutine, usize>,
    /// The `REXX` package routine each `::ROUTINE ... EXTERNAL "LIBRARY REXX"`
    /// directive bound.
    rexx_routine_rows: HashMap<InstalledRoutine, &'static internal_routines::InternalRoutine>,
    /// The `REXX` package routine each `loadExternalRoutine` answer over that
    /// package is.
    rexx_routine_objects: HashMap<ObjRef, &'static internal_routines::InternalRoutine>,
    /// The [`Interp::library_codes`] row of each method a `~define` installed
    /// from a `loadExternalMethod` answer, which is the package it reports.
    defined_library_codes: HashMap<MethodId, usize>,
    /// `PackageManager::packageRoutines`
    /// (`interpreter/package/PackageManager.cpp:524`): every routine a loaded
    /// library exports, by upcased name, as the index of its slot in
    /// [`Interp::package_routine_codes`]. A later library exporting the same
    /// name replaces the slot's row and keeps its index, so a call site that
    /// resolved to the slot calls the replacement.
    package_routines: HashMap<Vec<u8>, usize>,
    /// The [`Interp::library_codes`] row each [`Interp::package_routines`]
    /// slot calls.
    package_routine_codes: Vec<usize>,
    /// Moved by every write to a routine table an existing call site's
    /// lookup reads: a library registering routines
    /// ([`Interp::register_package_routines`]), a merge that adds a name
    /// ([`Interp::merge_routines`]), and `~addRoutine`/`~addPublicRoutine`.
    /// See `Resolved::kept_until_routines_change`.
    pub(crate) routine_generation: u32,
    /// The access scope and protection of every method that has one -- the
    /// oracle's `isSpecial()` set, which is what `RexxObject::messageSend`
    /// consults before it runs anything.
    special_methods: Vec<Option<dispatch::AccessScope>>,
    /// The output sink. `SAY` writes here and `Outcome::stdout` is what it
    /// becomes.
    out: Vec<u8>,
    /// The trace sink, which becomes `Outcome::stderr`.
    trace: Vec<u8>,
    /// `current_value_indent` and `current_clause_line`, bundled -- see
    /// `ClauseState`'s own doc comment for what the two share, the property
    /// that decides what belongs alongside them, and why they are one field
    /// rather than two.
    clause_state: ClauseState,
    /// **SPIKE.** The flattened `DO`/`LOOP`s the op driver has open, innermost
    /// last.
    #[expect(
        clippy::vec_box,
        reason = "the box is the point: a pass boundary takes the top out to hand it a &mut Interp beside it, and moves a pointer rather than the header's Numbers"
    )]
    flat_loops: Vec<Box<crate::run::FlatLoop>>,
    /// **SPIKE.** The innermost open flat loop, held apart from the stack of
    /// the ones enclosing it.
    flat_top: Option<Box<crate::run::FlatLoop>>,
    /// **SPIKE.** The constructs the op driver has open, innermost last, across
    /// every level of it at once.
    frames: Vec<crate::ir::drive::Frame>,
    /// **SPIKE.** Boxes a finished loop handed back, so that entering a loop
    /// is a write into an allocation this interpreter already owns. A loop
    /// entered once per two passes is common enough -- `samples/rexxcps.rex`
    /// enters one 140,000 times to run 280,000 passes -- that an allocation
    /// per entry is charged against a saving per pass.
    #[expect(
        clippy::vec_box,
        reason = "these are allocations handed back for reuse, so the box is what is being kept"
    )]
    flat_spares: Vec<Box<crate::run::FlatLoop>>,
    /// A condition raised by `RAISE` whose `CALL ON` handler has not run yet.
    pending_traps: VecDeque<PendingTrap>,
    /// The condition whose handler is running, for `RAISE PROPAGATE` to
    /// re-raise.
    active_condition: Option<ActiveCondition>,
    /// The object a `RAISE ... ADDITIONAL` named, held from the raise until
    /// the condition object is built. **The raise's own value and not a
    /// rebuild of it**: measured, `additional 'JUSTONE'` puts a `String` in
    /// the directory and `additional (.array~new)` an empty `Array`, where
    /// reconstructing from the substitution list gives a one-item `Array` and
    /// nothing at all. It lives on `Interp` rather than on `Raised` because a
    /// `Raised` travels inside a `Failure` on the Rust stack, where no
    /// destructure can root it, and this field is covered by the exhaustive
    /// match in `object_roots`.
    pending_additional: Option<ObjRef>,
    /// The `RESULT` a condition an extension raised names, held from the
    /// raise until the condition object is built, as
    /// [`Interp::pending_additional`] is.
    pending_result: Option<ObjRef>,
    /// **F3, found by review.** The innermost `SELECT CASE`'s own evaluated
    /// `case` text, or `None` inside a plain `SELECT` (or before any
    /// `SELECT`/`SELECT CASE` has run at all) -- the one piece of state an
    /// **absorbed** `WhenCase` needs that nothing else threads to it: a
    /// *listed* `WhenCase` gets `case_text` handed to it directly by
    /// `Select`'s own explicit arm (`run.rs`), but an absorbed one (a
    /// `WhenCase` reached only through ordinary `Op::Clause`'s region
    /// stepping, because it is itself the `THEN` consequence of a
    /// preceding `WHEN`/`WHEN CASE`, `ast.rs`'s own doc comment on
    /// `whens`) has no such hand-off -- it is stepped like any other
    /// instruction, with nothing carrying its enclosing `SELECT CASE`'s
    /// own comparison value along.
    current_case_text: Option<Vec<u8>>,
    /// **F3's own perimeter, found by review -- and corrected twice more,
    /// each correction found by re-verifying the previous one rather than
    /// trusting it.** When an absorbed `WhenCase` (`run.rs`'s own doc
    /// comment on that arm) takes its `Flow::Goto(false_target)` branch,
    /// whatever it lands on -- `END`'s own 7.3, or (F-EX1) `OTHERWISE`'s
    /// own marker *and its whole body*, redirected through `run_
    /// otherwise` -- reports every indent it computes **`self` spaces
    /// higher** than its own ordinary `static_indent` would give, for as
    /// long as this stays non-zero.
    indent_offset: usize,
    /// The absolute printed indent every clause of the **current activation
    /// level** starts from -- `0` for a program's own body, and an
    /// `INTERPRET` fragment's enclosing clause's own printed indent for the
    /// life of that fragment.
    activation_indent: usize,
    /// The clause a `Raised` condition escaped from, as the 1-based line and
    /// the bytes `TRACE` would echo, or `None` if nothing raised.
    failure_site: Option<FailureSite>,
    /// The levels that have already finished failing, innermost first --
    /// `Raised::report`'s echo stack minus its last entry.
    failure_sites: Vec<FailureSite>,
    /// The line number every clause echo prints while an `INTERPRET`
    /// fragment is running, overriding the clause's own line in its own
    /// source.
    clause_line_override: Option<usize>,
    /// How many `INTERPRET` fragments are running, counted from zero outside
    /// any of them.
    fragment_depth: usize,
    /// Whether a line typed at an interactive-debug pause is running.
    /// `RexxActivation::noTracing` includes this, so a pause's own fragment
    /// traces nothing and pauses nowhere. Written only through
    /// [`Interp::replace_debug_pause`], which keeps `trace_cache` in step.
    pub(crate) debug_pause: bool,
    /// Task 16's collect-on-every-allocation gate criterion (4a exit gate,
    /// criterion 4): when true, [`Interp::alloc_with`] calls `Heap::collect`
    /// after every allocation instead of never. Off by default, and the off
    /// path is untouched by this field's existence -- `alloc_with` reads it
    /// once, in an `if`, and does nothing else differently; nothing upstream
    /// of that one check changed at all. Named for what it does rather than
    /// for the criterion, since a later, permanent collector would want the
    /// same flag and should not have to rename it away from a gate task's
    /// number.
    stress_collect: bool,
    /// The objects a collection found unreachable and flagged for `UNINIT`,
    /// oldest first, awaiting [`Interp::run_ready_uninits`] -- oracle's
    /// `setReadyForUninit` list (`memory/RexxMemory.cpp:274`).
    uninit_ready: Vec<ObjRef>,
    /// Whether a `UNINIT` sweep is running -- oracle's `processingUninits`
    /// (`memory/RexxMemory.cpp:341`-`:347`, cleared at `:383`).
    processing_uninits: bool,
    /// Whether a trace line is being delivered to `.TRACEOUTPUT`. A traced
    /// clause inside that delivery writes to the buffer instead of routing
    /// again: the oracle SIGSEGVs in the one shape that reaches this
    /// (`corpus/oracle-crashes.txt` entry 14), and this crate must terminate.
    routing_trace: bool,
    /// The `.STDERR` the bundle minted, for recognising a trace route no
    /// program has redirected: the monitor chain still ends here, and a direct
    /// write is then the same bytes a delivery would produce.
    bootstrap_stderr: Option<ObjRef>,
    /// The `.STDOUT` the bundle minted, which is the same recognition for
    /// `SAY`'s own route. Measured, delivering every `SAY` through the monitor
    /// costs 16,360 instructions a line where writing costs none of it.
    bootstrap_stdout: Option<ObjRef>,
    /// Bumped by every write that could move a route's far end: a `.local`
    /// entry, a directory put or removal, and an array write, which is how a
    /// monitor's destination queue changes.
    route_generation: u64,
    /// What a `dispatch::hash::StoreView` is valid against.
    /// `hash::bump_store_generation` moves it.
    store_generation: u64,
    /// `SAY`'s route as of [`Interp::route_generation`]: `None` writes
    /// straight to the buffer. Deciding it afresh costs 764 instructions a
    /// line -- measured, `sayloop` at 1.40x -- and both halves of that are
    /// hash lookups, so the decision is cached rather than either half.
    output_route: Option<(u64, Option<ObjRef>)>,
    /// The arena size at which [`Interp::alloc_with`] collects, and half of
    /// this crate's trigger policy. The other half is `Heap::will_grow`.
    collect_at: usize,
    /// Current `eval` recursion depth, and the deepest it has reached.
    depth: usize,
    max_depth: usize,
    /// The depth-1 address of the chain currently being evaluated, kept aside
    /// until that chain turns out to be the deepest one.
    stack_entry: usize,
    /// The two ends of the span, both from the chain that reached
    /// `max_depth`, written together so they can never disagree.
    stack_first: usize,
    stack_deepest: usize,
    /// Whether the instruction about to be stepped is allowed to be a
    /// `PROCEDURE` -- and, read the other way, whether it is the first
    /// instruction executed in its activation.
    procedure_permitted: bool,
    /// What [`Interp::procedure_permitted`] held when the running
    /// [`crate::ir::Op::Clause`] region opened, for the one op that needs it.
    region_procedure_permitted: bool,
    /// The call that entered the running activation: what `USE ARG` reads.
    call_context: CallContext,
    /// The in-process external data queue (I15): every line
    /// `PUSH`/`QUEUE` has written and `PULL`/`PARSE PULL` have not yet
    /// removed. See `queue.rs`'s own module doc for the LIFO/FIFO split, and
    /// `Interp::pull_line` (`input.rs`) for the queue-first-then-`.input`
    /// rule that reads it.
    queue: Queue,
    /// `.input`'s position: the one line cursor `PULL`, `PARSE PULL` and
    /// `PARSE LINEIN` all advance. See `input.rs` for the shared-position
    /// measurement and the line rule; `ProgramInput`'s own doc has why the
    /// process's real standard input is never what this holds by default.
    input: Input,
    /// Whether each standard descriptor -- input, output, error, in that
    /// order -- is transient, which decides `QUERY STREAMTYPE` and whether
    /// positioning a standard stream is 93.958. Taken from the `Invocation`,
    /// never from this process: see `Invocation::standard_transient`.
    standard_transient: [bool; 3],
    /// Where the two output buffers go before a read that would wait on a
    /// live standard input, or `None` for an embedding that reads them out of
    /// the `Outcome` at the end. See [`crate::Sinks`].
    sinks: Option<crate::invocation::Sinks>,
    /// The command-line words `.SYSCARGS` answers, before they become the one
    /// `Array` `.local` holds -- see `Invocation::words` for why the joined
    /// argument string cannot supply them.
    command_words: Vec<Vec<u8>>,
    /// `RANDOM`'s generator state: the seed the next call will scramble, or
    /// `None` before any call has drawn one.
    random_seed: Option<u64>,
    /// `TIME('E')`/`TIME('R')`'s anchor: the clock reading (`builtin::
    /// datetime`'s microseconds-since-0001-01-01 unit) elapsed time is
    /// measured from, or `None` before any `E`/`R` call has run.
    /// ```text
    /// zz=time('E'); call burn; call sub; say 'after' time('E')   [sub does n2 = time('R')]
    ///   oracle:  inside 0.725271  inside-after-R 0.000004  after 0.725387
    ///   crate:   inside 33.889432 inside-after-R 0.000005  after 0.000013
    /// ```
    elapsed_anchor: Option<i64>,
    /// Whether a `TIME('R')` (or a clock read going backward) is waiting
    /// to move [`elapsed_anchor`] the next time the clock cache next
    /// refreshes -- `RexxActivation`'s own `elapsedReset` state flag
    /// (`execution/ActivationSettings.hpp:121`), consumed by
    /// `builtin::datetime::now_base_time`'s cache-miss path. See
    /// [`elapsed_anchor`]'s own doc for why the reset is lazy at all.
    pending_elapsed_reset: bool,
    /// Whether the required-string protocol can answer anything other than
    /// the value it was handed -- `Interp::required_string_value`'s gate.
    reqstr_armed: bool,
    /// Whether any program in this run installed `::OPTIONS ... LOSTDIGITS
    /// SYNTAX`, which is the gate on [`Interp::lostdigits_check`].
    lostdigits_armed: bool,
    /// The running program's own location, as `PARSE SOURCE`'s third word.
    program_path: String,
    /// The resolved location of each program a `::REQUIRES` loaded, which is
    /// what that program's own `PARSE SOURCE`, `~package~name` and traceback
    /// report in place of [`Interp::program_path`].
    required_paths: HashMap<ProgramId, Box<str>>,
    /// The package each `::REQUIRES` name has already loaded, keyed both by
    /// the name as written and by the file it resolved to.
    required_packages: HashMap<Box<[u8]>, ProgramId>,
    /// The programs whose first walk of [`Interp::install_directives`], which is
    /// this crate's translation, started and did not finish. The oracle gives
    /// the package of a translation that raised no routine, method or resource
    /// table (`parser/LanguageParser.cpp:1893-1908`) and no prolog (`:656-665`).
    untranslated: std::collections::HashSet<ProgramId>,
    /// The resolved paths whose `::REQUIRES` directives are still installing
    /// -- `Activity`'s own `requiresTable` (`concurrency/Activity.hpp:308`).
    requires_installing: Vec<Box<str>>,
}

/// Where one installed `::ROUTINE` lives: which loaded program, and which of
/// its directives.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct InstalledRoutine {
    program: ProgramId,
    directive: usize,
}

/// One entry of [`Interp::merged_public_routines`].
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum MergedRoutine {
    /// A `::ROUTINE` a required package declared or imported.
    Installed(InstalledRoutine),
    /// A library routine, as its [`Interp::library_codes`] row.
    Library(usize),
}

impl MergedRoutine {
    /// What a call that finds this entry runs.
    pub(crate) fn resolved(self) -> run::Resolved {
        match self {
            MergedRoutine::Installed(installed) => run::Resolved::Routine(installed),
            MergedRoutine::Library(code) => run::Resolved::MergedLibraryRoutine(code),
        }
    }
}

/// One `Method` or `Routine` object this crate has handed out, as its own
/// readers see it.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) struct ExecutableRecord {
    /// What the seven flags, `~source` and `~package` report on.
    pub(crate) source: ExecutableSource,
    /// The dictionary entry this object *is*, which is what makes
    /// `Method~setPrivate` change how a send resolves. `None` for an object
    /// no class has taken -- a `.METHODS` entry, or one compiled from source
    /// text.
    pub(crate) installed: Option<MethodId>,
    /// The `::ROUTINE` directive `Routine~call` enters, which is not always
    /// the directive [`ExecutableRecord::source`] names: a `Routine`
    /// compiled from source text reports the whole of its own program as its
    /// source and runs the sole directive that program carries.
    pub(crate) routine: Option<(ProgramId, usize)>,
}

/// What a `Method` or `Routine` object's own readers report on -- its seven
/// flags, its `~source` and its `~package`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum ExecutableSource {
    /// A directive of a program: a `::METHOD`, an `::ATTRIBUTE`, a
    /// `::CONSTANT` or a `::ROUTINE`.
    Directive {
        program: ProgramId,
        directive: usize,
    },
    /// A program's own main section, which is what a body compiled from
    /// source text is: `translateBlock` starts such a block at line 1
    /// (`parser/LanguageParser.cpp:1193`) where a directive's starts after
    /// the directive clause.
    Main { program: ProgramId },
    /// A primitive, which has no directive to report on:
    /// `BaseCode::getSource` answers an empty array
    /// (`execution/BaseCode.cpp:120`) and `BaseCode::setSecurityManager`
    /// answers `0` (`:133`).
    Native,
    /// An `EXTERNAL` binding a directive of `program` installed, which reads
    /// as [`ExecutableSource::Native`] except that its package is `program`.
    External { program: ProgramId },
    /// A `loadExternalMethod` or `loadExternalRoutine` answer over a library
    /// procedure, which reads as [`ExecutableSource::Native`] except that its
    /// package is whatever [`Interp::library_codes`] row `code` holds when
    /// asked.
    Loaded { code: usize },
}

/// The code object a library shares between every binding of one procedure.
///
/// A method's is cached per spelling asked for
/// (`LibraryPackage::resolveMethod`, `package/LibraryPackage.cpp:374-400`); a
/// routine's is built once per routine table entry when the library loads,
/// and found by its exact spelling or else by the first entry matching
/// without regard to case (`LibraryPackage::loadRoutines` and
/// `::resolveRoutine`, `:270-299`, `:410-435`), as `Library::routine` finds it.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct LibraryCodeKey {
    /// The library name byte for byte, as [`Interp::libraries`] keys it.
    pub(crate) library: Vec<u8>,
    /// A method's procedure name byte for byte as asked, or the name a
    /// routine table entry declares.
    pub(crate) procedure: Vec<u8>,
    /// Whether this is the routine table's entry rather than the method
    /// table's.
    pub(crate) routine: bool,
}

/// What a namespace qualifier resolved to.
#[derive(Copy, Clone)]
enum Namespace {
    Rexx,
    Package(ProgramId),
}

/// Where one installed `::METHOD`/`::ATTRIBUTE` accessor's own body lives --
/// the same shape as [`InstalledRoutine`], for the same reason: a
/// [`rexx_classes::MethodId`] alone names neither the program nor the
/// directive, and both are needed to find the [`rexx_parse::CodeBody`]
/// (or its absence, for a generated accessor or an `ABSTRACT`/`DELEGATE`
/// method) later.
#[derive(Copy, Clone)]
struct InstalledMethodBody {
    program: ProgramId,
    directive: usize,
}

/// An equality rather than a bound, for the reason `crate::ir::Op`'s own
/// width assertion is one: every send to a `::METHOD` body copies one of
/// these out of [`Interp::method_bodies`], and a field added here costs that
/// path -- measured, `bench-programs/dispatchclass.rex` at +1.88% and 121
/// `instructions:u` per send when a [`GeneratedKind`] discriminant sat
/// beside `program` and `directive` (see [`GeneratedMethod`]).
const _: () = assert!(size_of::<InstalledMethodBody>() == 16);

/// One installed method that the *directive* implements rather than a body:
/// a generated accessor, or an `ABSTRACT` declaration.
#[derive(Copy, Clone)]
struct GeneratedMethod {
    program: ProgramId,
    directive: usize,
    kind: GeneratedKind,
}

/// Which method a directive generated.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum GeneratedKind {
    /// A generated getter: it answers the attribute's variable in the
    /// declaring scope's pool on the receiver.
    Getter,
    /// A generated setter: it assigns that variable and answers nothing.
    Setter,
    /// `ABSTRACT`, on either directive and on either half of a generated
    /// accessor pair: the send is 93.965 whatever the arguments are.
    Abstract,
    /// `DELEGATE`, on either directive and on both halves of the pair a
    /// `::ATTRIBUTE` or a `::METHOD ... ATTRIBUTE` generates: the message is
    /// re-sent, under the name it arrived under and with the arguments it
    /// arrived with, to the value of the delegate variable in the declaring
    /// scope's pool on the receiver.
    Delegate,
    /// A `::CONSTANT` accessor: it answers the value
    /// [`Interp::constant_values`] holds for the directive.
    Constant,
}

/// The directories `LD_LIBRARY_PATH` names in `environment`.
///
/// The process loader read that variable before any interpreter existed, so a
/// value an interpreter is handed reaches the search no other way. Writing it
/// back to the process is forbidden here and would reach every other
/// interpreter in the process besides.
fn library_search_of(environment: &[(Vec<u8>, Vec<u8>)]) -> Vec<std::path::PathBuf> {
    use std::os::unix::ffi::OsStrExt;
    let Some((_, value)) = environment
        .iter()
        .find(|(name, _)| name.as_slice() == b"LD_LIBRARY_PATH")
    else {
        return Vec::new();
    };
    value
        .split(|byte| *byte == b':')
        .filter(|part| !part.is_empty())
        .map(|part| std::path::PathBuf::from(std::ffi::OsStr::from_bytes(part)))
        .collect()
}

/// A native library the interpreter has tried to resolve.
pub(crate) enum LibraryLoad {
    Loaded(Rc<rexx_api::load::Library>),
    /// No shared object of that name loaded, or one loaded and published no
    /// package entry. `LibraryPackage::load` (`package/LibraryPackage.cpp:147`)
    /// answers false for both, because `getPackageTable` returns null for each
    /// (`:204`, `:216`), and its callers cannot tell them apart.
    Missing,
    /// The package entry asks for a newer interpreter than this one. This ask
    /// raises; the library is held, and every later ask answers it loaded.
    Version,
    /// The package loader raised or refused. This ask raises the failure; the
    /// library is held, and every later ask answers it loaded.
    Raised(Failure),
}

/// One dictionary key bound to one procedure of one loaded library.
#[derive(Clone)]
pub(crate) struct LibraryBinding {
    pub(crate) library: Rc<rexx_api::load::Library>,
    pub(crate) procedure: Vec<u8>,
}

/// One native activation: what an extension writes its object variables
/// through, and the handles it has been given.
struct NativeFrame {
    /// The object whose [`rexx_core::ScopePools`] the write lands in, which
    /// is [`Interp::pool_owner`] of the receiver.
    owner: ObjRef,
    /// The running method's own scope, not the receiver's class --
    /// `receiver->getObjectVariables(getScope())`
    /// (`execution/NativeActivation.cpp:1878`).
    scope: ObjRef,
    /// A method's activation rather than a routine's, which is what decides
    /// whether a signature may ask for the receiver's state.
    method: bool,
    /// The method's receiver, `.nil` for a routine.
    receiver: ObjRef,
    /// The name the method was sent by or the routine called by.
    name: Vec<u8>,
    /// The call's arguments, an omitted one `None`.
    arguments: Vec<Option<ObjRef>>,
    /// The array [`NativeFrame::arguments`] became, once a conversion asked.
    argument_list: Option<ObjRef>,
    locals: rexx_api::handles::Table,
    /// The condition an argument's string conversion or a callback raised,
    /// held for the call to raise once it has returned.
    raised: Option<Failure>,
    /// The held condition's `ADDITIONAL` object, where the raise named one.
    additional: Option<ObjRef>,
    /// The held condition's `RESULT` object, where the raise named one.
    result: Option<ObjRef>,
    /// The held condition's object, once a callback asked for it.
    condition: Option<ObjRef>,
    /// A routine's [`Interp::library_codes`] row, `None` for a method or a
    /// package hook.
    code: Option<usize>,
    /// The handle-carried values this call asked a kept `CSTRING` of, each
    /// counted once in [`Interp::kept_holders`].
    kept: std::collections::HashSet<ObjRef>,
}

/// What one just-installed dictionary key resolves to, handed to
/// [`Interp::install_one_method`] by whichever installer minted it.
#[derive(Clone)]
enum InstallBody {
    /// The directive's own Rexx body: a row of [`Interp::method_bodies`].
    Written,
    /// A method the directive implements itself: a row of
    /// [`Interp::generated_methods`].
    Generated(GeneratedKind),
    /// A procedure of a loaded shared library: a row of
    /// [`Interp::library_externals`].
    Library(LibraryBinding),
    /// A `LIBRARY REXX` entry point: a row of [`Interp::native_externals`].
    Native(&'static dispatch::native::NativeExternal),
}

/// The name, arguments and receiver of one call in progress.
#[derive(Default)]
struct CallContext {
    /// The resolved routine name, as errors 40.3 and 40.4 spell it --
    /// measured, `Not enough arguments in invocation of SUB2`, the label's
    /// own upcased spelling.
    name: Vec<u8>,
    /// The arguments in source order, an omitted position (`call sub 1,,3`)
    /// left as `None` rather than closed up. Measured: that call into `use
    /// arg p, q, r` gives `[1] [Q] [3]`, so an omission holds its place.
    arguments: Rc<[Option<ObjRef>]>,
    /// **The receiver, which is part of the calling convention** (D24): the
    /// object a message send was addressed to, and `None` for a call that has
    /// none.
    receiver: Option<ObjRef>,
}

impl Interp {
    /// `values` as the shared slice a [`CallContext`] carries, without
    /// allocating for an empty one -- see [`Interp::empty_arguments`].
    pub(crate) fn shared_arguments(&self, values: &[Option<ObjRef>]) -> Rc<[Option<ObjRef>]> {
        if values.is_empty() {
            return Rc::clone(&self.empty_arguments);
        }
        Rc::from(values)
    }
}

/// Where a variable lives: a frame slot, or a name in a scope pool on some
/// object.
#[derive(Clone, Debug)]
enum VarHome {
    Slot(SlotRef),
    Instance(Box<InstanceVar>),
}

impl CallContext {
    /// Appends every `ObjRef` this convention holds to `out`.
    fn object_roots(&self, out: &mut Vec<ObjRef>) {
        let CallContext {
            name: _,
            arguments,
            receiver,
        } = self;
        out.extend(arguments.iter().flatten().copied());
        out.extend(*receiver);
    }
}

impl Interp {
    /// **Every field a caller might want to vary starts at a fixed value
    /// here and is set after construction**, so this signature does not grow
    /// a parameter for each one. `stress_collect` and `engine` are both of
    /// that shape: `execute` sets each from what it was handed, and every
    /// other caller -- the unit tests throughout this crate, which is nearly
    /// all of them -- gets the value below.
    fn new() -> Interp {
        let env: Vec<(Vec<u8>, Vec<u8>)> = {
            use std::os::unix::ffi::OsStrExt;
            std::env::vars_os()
                .map(|(name, value)| (name.as_bytes().to_vec(), value.as_bytes().to_vec()))
                .collect()
        };
        Interp {
            heap: Heap::new(),
            roots: RootSet::new(),
            key_buffer: Vec::new(),
            value_buffer: Vec::new(),
            parse_buffers: Vec::new(),
            text_scratch: [0; crate::value::TEXT_SCRATCH],
            text_numbers: crate::value::TextNumbers::new(),
            result_buffer: std::cell::Cell::new(Vec::new()),
            library_search: library_search_of(&env),
            env,
            cwd: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/")),
            locals: Vec::new(),
            running: None,
            suspended: Vec::new(),
            spare_activations: Vec::new(),
            programs: Vec::new(),
            package_options: HashMap::new(),
            security_managers: HashMap::new(),
            plans: NameMap::default(),
            deadline: None,
            clause_countdown: crate::clause::Deadline::NO_DEADLINE_SPACING,
            chunks: NameMap::default(),
            chunks_refused: 0,
            deferred: std::collections::VecDeque::new(),
            routines: HashMap::new(),
            package_public_routines: HashMap::new(),
            merged_public_routines: HashMap::new(),
            merged_public_classes: NameMap::default(),
            rexx_class_cache: NameMap::default(),
            package_namespaces: HashMap::new(),
            package_locals: HashMap::new(),
            object_model: None,
            class_variables: HashMap::new(),
            environment: None,
            package_classes: NameMap::default(),
            package_public_classes: NameMap::default(),
            class_packages: HashMap::new(),
            empty_arguments: Rc::from(&[][..]),
            package_objects: HashMap::new(),
            program_routine_objects: HashMap::new(),
            package_tables: HashMap::new(),
            routine_objects: HashMap::new(),
            library_routine_objects: HashMap::new(),
            package_imports: HashMap::new(),
            package_parents: HashMap::new(),
            constant_values: HashMap::new(),
            annotations: HashMap::new(),
            compiled_methods: 0,
            method_objects: HashMap::new(),
            library_bootstrap: false,
            collections_before_program: 0,
            library_programs: Vec::new(),
            method_bodies: NameMap::default(),
            compiled_method_names: HashMap::new(),
            object_methods: false,
            table_method_bodies: NameMap::default(),
            executable_sources: HashMap::new(),
            method_flag_writes: HashMap::new(),
            message_outcomes: HashMap::new(),
            generated_methods: HashMap::new(),
            native_externals: HashMap::new(),
            libraries: Libraries::new(),
            thread: rexx_api::ffi::ThreadContext::new(),
            #[cfg(test)]
            library_open_attempts: 0,
            library_externals: HashMap::new(),
            external_packages: HashMap::new(),
            library_codes: Vec::new(),
            library_code_rows: HashMap::new(),
            library_code_keys: Vec::new(),
            library_routine_codes: HashMap::new(),
            defined_library_codes: HashMap::new(),
            rexx_routine_rows: HashMap::new(),
            rexx_routine_objects: HashMap::new(),
            package_routines: HashMap::new(),
            package_routine_codes: Vec::new(),
            routine_generation: 0,
            native_handles: Vec::new(),
            native_spares: Vec::new(),
            global_references: rexx_api::handles::Table::new(),
            kept_strings: std::collections::HashMap::new(),
            kept_names: std::collections::HashMap::new(),
            kept_holders: std::collections::HashMap::new(),
            special_methods: Vec::new(),
            out: Vec::new(),
            trace: Vec::new(),
            clause_state: ClauseState::new(),
            flat_loops: Vec::new(),
            flat_top: None,
            flat_spares: Vec::new(),
            frames: Vec::new(),
            pending_traps: VecDeque::new(),
            active_condition: None,
            pending_additional: None,
            pending_result: None,
            next_activation_id: 0,
            next_invocation: 0,
            current_case_text: None,
            indent_offset: 0,
            activation_indent: 0,
            failure_site: None,
            failure_sites: Vec::new(),
            clause_line_override: None,
            fragment_depth: 0,
            debug_pause: false,
            stress_collect: false,
            uninit_ready: Vec::new(),
            processing_uninits: false,
            routing_trace: false,
            bootstrap_stderr: None,
            bootstrap_stdout: None,
            route_generation: 0,
            store_generation: 0,
            output_route: None,
            collect_at: COLLECT_FLOOR,
            depth: 0,
            max_depth: 0,
            stack_entry: 0,
            stack_first: 0,
            stack_deepest: 0,
            procedure_permitted: false,
            region_procedure_permitted: false,
            call_context: CallContext::default(),
            queue: Queue::new(),
            // Nothing to read, which is what makes it impossible for a unit
            // test to reach the harness's own standard input: only `execute`
            // ever replaces this, and only from an `Invocation` that named a
            // source. `ProgramInput`'s own doc has the argument.
            input: Input::new(ProgramInput::Nothing),
            // All transient, for the same reason `input` reads nothing: only
            // `execute` replaces this, and only from an `Invocation` whose
            // caller owns real descriptors.
            standard_transient: [true; 3],
            sinks: None,
            command_words: Vec::new(),
            random_seed: None,
            elapsed_anchor: None,
            pending_elapsed_reset: false,
            reqstr_armed: false,
            lostdigits_armed: false,
            program_path: String::new(),
            required_paths: HashMap::new(),
            required_packages: HashMap::new(),
            untranslated: std::collections::HashSet::new(),
            requires_installing: Vec::new(),
            trace_cache: crate::trace::TraceCache::of(crate::trace::TraceMode::OFF, false),
        }
    }

    // ---- loading and the activation stack ----

    /// Loads `program`, runs its main body in a fresh activation, and tears
    /// the activation down again.
    fn run(&mut self, program: Program) -> Result<Option<ObjRef>, Failure> {
        let program = Rc::new(program);
        let program_id = ProgramId(self.programs.len());
        self.programs.push(Rc::clone(&program));
        self.run_loaded(program, program_id, CallType::Command, None, None)
    }

    /// Runs the interpreter's own Rexx-written library -- `Setup.cpp:1786`'s
    /// `resolveProgramName(BASEIMAGELOAD)` and the `runProgram` at `:1795`,
    /// which hands the entry program `TheRexxPackage` as its one argument.
    pub(crate) fn bootstrap_library(&mut self) -> Result<(), Failure> {
        debug_assert!(
            self.object_model.is_none(),
            "the library bootstrap must build the object model, so nothing may have forced \
             the shipped one before it runs"
        );
        let model = dispatch::ObjectModel::bootstrap_for_library(&mut || self.heap.mint_class());
        self.install_object_model(model);
        self.library_bootstrap = true;
        #[cfg(test)]
        ir::drive::suspend_counters();
        let entry = rexx_lib::lookup(rexx_lib::ENTRY)
            .unwrap_or_else(|| panic!("{} is embedded", rexx_lib::ENTRY));
        let outcome = self.enter_library_program(entry, None);
        self.library_bootstrap = false;
        rexx_classes::remove_setup_methods(self.classes());
        if outcome.is_ok() {
            self.mint_local_directory();
        }
        // **Every per-run instrument reads from here, not from process
        // start.** `Outcome::collections` and `Outcome::chunks_refused`
        // answer a question about the program, and so do the compiled
        // engine's own counters; the bootstrap is the interpreter starting
        // up. Without this, `run_program_collect_every_alloc`'s
        // "collections non-zero" criterion is met by the library allocating
        // rather than by the program, which is the criterion measuring
        // nothing.
        self.collections_before_program = self.heap.collections_performed();
        self.chunks_refused = 0;
        #[cfg(test)]
        ir::drive::resume_counters();
        outcome.map(|_| ())
    }

    /// The file the external search resolves `name` to, or `None` when no
    /// route holds one. `&self`, which is what lets the call resolver use it.
    pub(crate) fn external_program(&self, name: &[u8]) -> Option<String> {
        let program = self.running_activation()?.program_id;
        self.resolve_search(Some(self.package_path(program)), name, false)
    }

    /// One external Rexx file, entered by a call: read, parsed, registered and
    /// run, with `arguments` as its calling convention.
    ///
    /// **Registered afresh every time, and never in the requires cache.**
    /// Measured: a file rewritten between two calls runs both versions, so
    /// nothing here may serve the first parse to the second call, and a later
    /// `::REQUIRES` of the same file must still run its prologue.
    fn enter_external_program(
        &mut self,
        name: &[u8],
        arguments: Vec<Option<ObjRef>>,
        call_type: CallType,
    ) -> Result<Option<ObjRef>, Failure> {
        let Some(resolved) = self.external_program(name) else {
            return Err(Raised::routine_not_found(name).into());
        };
        // Resolved and then unreadable is 3.1 naming the file, not the 43.1
        // of a name that resolved to nothing -- measured on a mode-000 file.
        let Ok(text) = std::fs::read(&resolved) else {
            return Err(Raised::executable_file_unreadable(resolved.as_bytes()).into());
        };
        let parsed = match parse_program(text) {
            Ok(parsed) => Rc::new(parsed),
            Err(error) => {
                return Err(Loud::required_source(&resolved, &format!("{error}")).into());
            }
        };
        let caller = self
            .running_activation()
            .map(|activation| activation.program_id);
        let program_id = ProgramId(self.programs.len());
        self.programs.push(Rc::clone(&parsed));
        // Its own path, so the callee's `PARSE SOURCE`, `~package~name` and
        // traceback name the file rather than the program that called it.
        self.required_paths
            .insert(program_id, resolved.as_str().into());
        let address = self.activation().address.clone();
        let settings = self.activation().settings.clone();
        let saved = std::mem::replace(
            &mut self.call_context,
            CallContext {
                name: name.to_vec(),
                arguments: Rc::from(arguments),
                receiver: None,
            },
        );
        let outcome = self.run_loaded(parsed, program_id, call_type, Some(address), Some(settings));
        self.call_context = saved;
        let value = outcome?;
        // What the callee made public becomes the caller's, and transitively
        // what the callee itself required.
        if let Some(caller) = caller {
            self.merge_required(caller, program_id);
        }
        Ok(value)
    }

    /// Parses one embedded library program, registers it, and runs its body
    /// with `arguments` as its calling convention.
    fn enter_library_program(
        &mut self,
        program: &'static rexx_lib::Program,
        arguments: Option<Vec<Option<ObjRef>>>,
    ) -> Result<Option<ObjRef>, Failure> {
        let parsed = match parse_program(program.source.to_vec()) {
            Ok(parsed) => parsed,
            // A parse failure here is the embedded source, not a program's:
            // `rexx-lib` pins each file's sha256, so this can only fire on a
            // file this crate cannot yet parse.
            Err(error) => {
                return Err(Loud::library_source(program.name, &format!("{error}")).into());
            }
        };
        let parsed = Rc::new(parsed);
        let program_id = ProgramId(self.programs.len());
        self.programs.push(Rc::clone(&parsed));
        // `rexx_package_class` searches this list, so a new library package
        // can turn a miss into a hit and shadow an earlier answer.
        self.invalidate_rexx_class_cache();
        self.library_programs.push(program_id);
        let arguments = arguments.unwrap_or_else(|| {
            let package = self.package_object(Package::Program(program_id));
            // Rooted for the whole of the bootstrap: `call_context` is not
            // walked by the collector, and the prologue allocates before it
            // reads `rexxPackage`.
            self.roots.push_temp(package);
            vec![Some(package)]
        });
        let saved = std::mem::replace(
            &mut self.call_context,
            CallContext {
                name: program.name.as_bytes().to_vec(),
                arguments: Rc::from(arguments),
                receiver: None,
            },
        );
        let outcome = self.run_loaded(parsed, program_id, CallType::Command, None, None);
        self.call_context = saved;
        outcome
    }

    /// What `program`'s own `::OPTIONS` directives left on its package, or
    /// `None` for a program carrying none.
    fn options_of(&self, program: ProgramId) -> Option<&PackageOptions> {
        self.package_options.get(&program)
    }

    /// The numeric settings the running activation's own package declares --
    /// what a bare `NUMERIC DIGITS`, `FUZZ` or `FORM` there resets to, and
    /// the language defaults for a package carrying no `::OPTIONS`.
    pub(crate) fn package_default_numeric(&self) -> Settings {
        self.options_of(self.activation().program_id)
            .map_or_else(Settings::default, |options| options.numeric.clone())
    }

    /// Starts a freshly built activation from its own package's `::OPTIONS`.
    fn start_from_package(&self, activation: &mut Activation, caller: Option<&Settings>) {
        let Some(options) = self.options_of(activation.program_id) else {
            return;
        };
        activation.settings = match caller {
            Some(caller) if options.numeric_inherit => caller.clone(),
            _ => options.numeric.clone(),
        };
        activation.condition_syntax = options.syntax;
        if let Some(trace) = options.trace {
            activation.trace_mode = trace;
        }
    }

    /// Whether the running activation turns an untrapped raise of `condition`
    /// into a SYNTAX error -- `::OPTIONS <condition> SYNTAX`, and `ALL` for
    /// all six at once.
    pub(crate) fn condition_raises_syntax(&self, condition: &[u8]) -> bool {
        self.running_activation()
            .is_some_and(|activation| activation.condition_syntax.raises(condition))
    }

    /// Installs `program`'s directives and runs its main body, for a program
    /// already registered under `program_id`.
    ///
    /// `address` is the environment pair the body starts with, `None` for one
    /// that starts at the platform default: a file reached by a **call**
    /// inherits its caller's, and measured, nothing else does.
    /// `caller_settings` is what `::OPTIONS NUMERIC INHERIT` inherits from,
    /// `None` where there is no call site above the body.
    fn run_loaded(
        &mut self,
        program: Rc<Program>,
        program_id: ProgramId,
        call_type: CallType,
        address: Option<crate::activation::AddressState>,
        caller_settings: Option<Settings>,
    ) -> Result<Option<ObjRef>, Failure> {
        // **Before the first clause, and its failures print nothing on
        // stdout.** That is the oracle's own shape rather than a choice
        // here: measured, `say 'main ran'` followed by `::requires
        // 'no_such_file_zz.rex'` is 43.901 at rc 213 with stdout EMPTY, and
        // `::class foo subclass zzznotaclass` is 98.909 at rc 158, likewise
        // empty. A program whose directives all install runs its main body
        // exactly as one with no directives does.
        self.install_directives(program_id, &program)?;

        // **`::OPTIONS NOPROLOG` suppresses the leading code section of a
        // package a `::REQUIRES` loaded, and of nothing else** --
        // `PackageClass::runProlog` installs and stops where
        // `isPrologEnabled()` is false (`classes/PackageClass.cpp:2131`),
        // and the top-level program never goes through it. Measured, oracle
        // rc 0: the same file prints its first clause when it is the program
        // and does not when another file requires it, its public routines and
        // classes reachable either way.
        if call_type == CallType::Requires
            && self
                .options_of(program_id)
                .is_some_and(|options| options.suppress_prolog)
        {
            return Ok(None);
        }

        // Note what does *not* happen here: the plan is looked up through
        // `&program.main`, a borrow of the local `Rc`, while `self` is
        // borrowed mutably by `plan_for`. Reaching the same body through
        // `self.programs[id.0].main` instead would be the `E0502` that
        // `run_activation` writes out.
        let plan = self.plan_for(
            BodyKey {
                program: program_id,
                directive: None,
            },
            &program.main,
            &program.symbols,
            &program.source,
        );

        let frame = self.roots.push_slots(plan.len());
        let id = self.next_activation_id();
        let mut main = Activation::new(id, Rc::clone(&program), program_id, plan, frame);
        main.call_type = call_type;
        if let Some(address) = address {
            main.address = address;
        }
        // With no call site above a main body, `::OPTIONS NUMERIC INHERIT` has
        // nothing to inherit and the package's own settings stand -- measured,
        // `::options digits 12 numeric inherit` alone in a file reports 12.
        // A file reached by a call has one, and passes it.
        self.start_from_package(&mut main, caller_settings.as_ref());
        // `SysInterpreterInstance::setupProgram`
        // (`platform/unix/SysInterpreterInstance.cpp:105`-`:112`), which runs
        // for every top-level program activation and not only the first --
        // measured, a called external program and a `::REQUIRES` prologue
        // each trace and prompt under `RXTRACE=ON` too. **Not the library
        // bootstrap**, whose programs are this crate's stand-in for
        // `Setup.cpp` and have no activation on the oracle to trace.
        let external_trace = !self.library_bootstrap && self.external_trace_enabled();
        self.push_activation(main);
        // Through the same route a `TRACE` instruction takes, because
        // `enableExternalTrace` is `setTrace` and that calls `traceEntry()`:
        // a called program announces itself with `>I>` where the top-level
        // one, which has nothing to announce, prints the debug banner
        // instead.
        if external_trace {
            self.set_trace_mode(crate::trace::TraceMode {
                debug: true,
                ..crate::trace::TraceMode::RESULTS
            });
            self.trace_invocation_entry();
        }

        // `Returned` and `Exited` are the same thing at the top: measured,
        // `return 5` in a main body with no active call exits 5, exactly like
        // `exit 5`, and a bare `return` there exits 0. `Ended` keeps the two
        // apart because a *callee* has to tell them apart, not because the
        // program's own exit value ever depends on which arrived.
        let exit = self.run_activation().map(Ended::value);

        // Popped whether or not the body raised, so the root set is left the
        // way it was found even on the failure path.
        let activation = self.pop_activation().expect("the frame just pushed");
        self.roots.pop_slots(activation.frame);
        exit
    }

    /// The value of one environment variable, or `None` for a name the
    /// interpreter's environment does not hold.
    pub(crate) fn env_get(&self, name: &[u8]) -> Option<&[u8]> {
        self.env
            .iter()
            .find(|(held, _)| held == name)
            .map(|(_, value)| value.as_slice())
    }

    /// Sets, replaces or (with `None`) removes one environment variable.
    /// `setenv` declines a name that is empty or holds `=` and stores nothing,
    /// which the oracle passes through as a silent no-op -- measured.
    pub(crate) fn env_set(&mut self, name: &[u8], value: Option<Vec<u8>>) {
        if name.is_empty() || name.contains(&b'=') {
            return;
        }
        let at = self.env.iter().position(|(held, _)| held == name);
        match (at, value) {
            (Some(at), None) => {
                self.env.remove(at);
            }
            (Some(at), Some(value)) => self.env[at].1 = value,
            (None, Some(value)) => self.env.push((name.to_vec(), value)),
            (None, None) => {}
        }
    }

    /// The directory this interpreter resolves relative paths against.
    pub(crate) fn cwd_text(&self) -> String {
        self.cwd.to_string_lossy().into_owned()
    }

    /// Moves it. `DIRECTORY(new)` is the only caller: nothing else in the
    /// interpreter changes where relative paths resolve from.
    pub(crate) fn set_cwd(&mut self, cwd: std::path::PathBuf) {
        self.cwd = cwd;
    }

    /// Appends bytes to what the program has written on standard output --
    /// the same buffer `SAY` appends to, so anything written here interleaves
    /// with it in program order.
    pub(crate) fn write_out(&mut self, bytes: &[u8]) {
        self.out.extend_from_slice(bytes);
    }

    /// [`Interp::write_out`] for the other descriptor: the trace sink, which
    /// becomes `Outcome::stderr`. `.STDERR`'s writes land here, so they
    /// interleave with trace output and never with `SAY`'s.
    pub(crate) fn write_err(&mut self, bytes: &[u8]) {
        self.trace.extend_from_slice(bytes);
    }

    /// `SETLOCAL`: saves the directory and the whole environment, and answers
    /// whether it saved one.
    pub(crate) fn push_local_environment(&mut self) -> bool {
        self.locals.push((self.cwd.clone(), self.env.clone()));
        true
    }

    /// `ENDLOCAL`: restores the innermost saved pair, answering whether there
    /// was one. **A restore puts back the names it saved and removes nothing
    /// added since** -- measured on the oracle, and `restoreEnvironment` only
    /// re-`putenv`s what it holds.
    pub(crate) fn pop_local_environment(&mut self) -> bool {
        let Some((cwd, saved)) = self.locals.pop() else {
            return false;
        };
        self.cwd = cwd;
        for (name, value) in saved {
            let at = self.env.iter().position(|(held, _)| *held == name);
            match at {
                Some(at) => self.env[at].1 = value,
                None => self.env.push((name, value)),
            }
        }
        true
    }

    /// `RXTRACE=ON` in the interpreter's own environment, which starts every
    /// top-level program under `TRACE ?R` -- `setExternalTrace` is
    /// `setTraceResults` plus the debug flag
    /// (`execution/TraceSetting.hpp`'s own definition).
    ///
    /// Caseless, and only that word: `SysInterpreterInstance::initialize`
    /// compares against `ON` with `strCaselessCompare`, so any other value
    /// leaves tracing alone.
    fn external_trace_enabled(&self) -> bool {
        self.env
            .iter()
            .find(|(name, _)| name == b"RXTRACE")
            .is_some_and(|(_, value)| value.eq_ignore_ascii_case(b"ON"))
    }

    /// One variable of the interpreter's own environment, as text. `None` for
    /// a name it does not hold or a value that is not UTF-8 -- a path this
    /// crate cannot spell is a path it cannot search.
    fn shadow_var(&self, name: &[u8]) -> Option<String> {
        self.env
            .iter()
            .find(|(held, _)| held == name)
            .and_then(|(_, value)| String::from_utf8(value.clone()).ok())
    }

    // `plan_for` and `activation`/`activation_mut` live in `plan.rs`/
    // `activation.rs` (Task 6), beside the types they operate on.

    // `fragment_plan` and `slot_of` live in `plan.rs` (Task 6), beside
    // `Plan` itself.

    /// Converts `EXIT`'s result into the raw exit code, before `rexx-run`'s
    /// own 8-bit truncation (`bin/rexx-run.rs`) narrows it to a process exit
    /// status.
    fn exit_code_for(&mut self, value: Option<ObjRef>) -> i32 {
        let Some(value) = value else { return 0 };
        let Ok(number) = self.to_number(value) else {
            return 0;
        };
        match number.whole_value(rexx_num::ARGUMENT_DIGITS) {
            Some(whole) => i32::try_from(whole).unwrap_or(0),
            None => 0,
        }
    }

    /// Roots a value that has to outlive the clause that produced it, all the
    /// way to [`Interp::exit_code_for`].
    fn root_exit_value(&mut self, value: ObjRef) {
        self.roots.add_global(EXIT_VALUE_ROOT, value);
    }

    /// Turns on Task 16's collect-on-every-allocation stress mode. Only
    /// `execute`'s `collect_every_alloc` arm calls this, right after
    /// construction and before `run`; nothing else needs to flip it, and
    /// nothing can un-flip it once a run has started.
    fn enable_stress_collect(&mut self) {
        self.stress_collect = true;
    }

    /// The one allocation entry point every value/stem constructor in this
    /// crate goes through, so that Task 16's stress mode has exactly one
    /// place to hook rather than one per call site.
    pub(crate) fn alloc_with(
        &mut self,
        behaviour: rexx_core::BehaviourId,
        body: rexx_core::Body,
    ) -> ObjRef {
        self.collect_if_due();
        self.heap.alloc_with_uncollected(behaviour, body)
    }

    /// The collection decision every allocation site makes, without the
    /// allocation.
    fn collect_if_due(&mut self) {
        if self.stress_collect
            || (self.heap.will_grow() && self.heap.slot_capacity() >= self.collect_at)
        {
            self.collect_now();
        }
    }

    /// Appends every `ObjRef` the interpreter must hand the collector to
    /// `out`, and names every field that does not need to be handed over.
    fn object_roots(&self, out: &mut Vec<ObjRef>) {
        let Interp {
            heap: _,
            roots: _,
            key_buffer: _,
            // Every push site roots the value as a temp before it lands here.
            value_buffer: _,
            parse_buffers: _,
            text_scratch: _,
            // Handle-inline strings, which are their own bytes and have no
            // slot to recycle.
            text_numbers: _,
            result_buffer: _,
            running,
            trace_cache: _,
            suspended,
            // A finished activation's leftovers, overwritten at reuse and read
            // by nothing in between.
            spare_activations: _,
            next_activation_id: _,
            next_invocation: _,
            programs: _,
            package_options: _,
            security_managers,
            plans: _,
            deadline: _,
            clause_countdown: _,
            // A chunk's interned literals are allocated immortal.
            chunks: _,
            chunks_refused: _,
            // `Interp::park_reply` hands each entry to `RootSet::park`.
            deferred: _,
            routines: _,
            package_public_routines: _,
            merged_public_routines: _,
            merged_public_classes,
            // **Not a root, deliberately.** Its values are class handles, and
            // rooting them would pin every class a `.NAME` ever resolved --
            // which is exactly the pinning Phase 5j removed. A cached handle
            // is instead dropped before it can go stale:
            // `Interp::collect_now` clears the cache on every collection, and
            // every write to a table it is derived from clears it too.
            rexx_class_cache: _,
            package_namespaces: _,
            // `RootSet::add_global`, under `package_local_root_key`.
            package_locals: _,
            // Handles into the class registry, so class identities again.
            object_model: _,
            // `.environment`, `.local` and the owed placeholders are globals;
            // the rest are classes. A `StoreView`'s store arrays are held by
            // its directory's own pool while its generation is current, and a
            // view whose generation is not current is not read.
            environment: _,
            // A lookup index. Both halves are held by the class itself --
            // the key is the class, the value is in its `owned` list.
            class_variables: _,
            package_classes,
            package_public_classes,
            // Keyed by class identity, and `ClassPackage` holds no `ObjRef`.
            class_packages: _,
            // Zero length.
            empty_arguments: _,
            // `RootSet::add_global`, under `package_root_key`.
            package_objects: _,
            // `RootSet::add_global`, under `program_routine_root_key`.
            program_routine_objects: _,
            // `RootSet::add_global`, under `package_table_root_key`.
            package_tables: _,
            // Rooted by the `.ROUTINES` table each entry is also in.
            routine_objects: _,
            // `RootSet::add_global`, under `library_routine_root_key`.
            library_routine_objects: _,
            package_imports: _,
            // Program identities and nothing else.
            package_parents: _,
            // `RootSet::add_global`, under `constant_root_key`.
            constant_values: _,
            // A lookup index. A class-owned site's table is held by that
            // class; every other site's is a global under
            // `annotation_root_key`.
            annotations: _,
            compiled_methods: _,
            // A lookup index. The `Method` object is held by the class whose
            // dictionary entry it answers for.
            method_objects: _,
            method_bodies: _,
            library_bootstrap: _,
            collections_before_program: _,
            library_programs: _,
            compiled_method_names: _,
            object_methods: _,
            // Keyed by an object and holding none: a swept key is a lookup
            // miss, because the generation bump makes the handle unequal.
            table_method_bodies: _,
            executable_sources: _,
            method_flag_writes: _,
            message_outcomes: _,
            generated_methods: _,
            native_externals: _,
            // Library handles and procedure names, no `ObjRef` in either.
            libraries: _,
            // Handles for constants that are not heap objects, and the
            // innermost native call, whose frame `native_handles` roots.
            thread: _,
            #[cfg(test)]
                library_open_attempts: _,
            library_externals: _,
            // Program identities, not objects.
            external_packages: _,
            library_codes: _,
            library_code_rows: _,
            library_code_keys: _,
            library_routine_codes: _,
            defined_library_codes: _,
            rexx_routine_rows: _,
            // Keyed by objects a generation bump makes unequal once swept.
            rexx_routine_objects: _,
            // Routine names and row indices.
            package_routines: _,
            package_routine_codes: _,
            routine_generation: _,
            native_handles,
            // A pop clears the buffers that held objects, and a push
            // overwrites the handle fields before anything reads them, so a
            // spare names nothing to root.
            native_spares: _,
            global_references,
            // Copies keyed by object, dropped with it rather than keeping it.
            kept_strings: _,
            kept_names: _,
            kept_holders: _,
            special_methods: _,
            out: _,
            trace: _,
            clause_state: _,
            // The `DO OVER` snapshot sits in a register held for the loop's
            // lifetime, and `RootSet` reaches a register as a temp.
            flat_loops: _,
            flat_top: _,
            frames: _,
            // Overwritten at reuse, as `spare_activations` is.
            flat_spares: _,
            pending_traps,
            active_condition: _,
            pending_additional,
            pending_result,
            current_case_text: _,
            indent_offset: _,
            activation_indent: _,
            failure_site: _,
            failure_sites: _,
            clause_line_override: _,
            fragment_depth: _,
            debug_pause: _,
            stress_collect: _,
            // The collector's own resurrection flag holds each object until
            // its finalizer clears it.
            uninit_ready: _,
            processing_uninits: _,
            routing_trace: _,
            // `.local` holds these streams and is a global root, so the
            // handles here root nothing of their own; they are only compared.
            bootstrap_stderr: _,
            bootstrap_stdout: _,
            route_generation: _,
            store_generation: _,
            // The route it names is `.local`'s own entry, which that global
            // root holds; this caches the decision, not the object.
            output_route: _,
            collect_at: _,
            depth: _,
            max_depth: _,
            stack_entry: _,
            stack_first: _,
            stack_deepest: _,
            procedure_permitted: _,
            region_procedure_permitted: _,
            // A running call's arguments and receiver are the caller's temps;
            // `CallContext::object_roots` is the parked case's other route.
            call_context: _,
            queue: _,
            input: _,
            random_seed: _,
            elapsed_anchor: _,
            pending_elapsed_reset: _,
            reqstr_armed: _,
            lostdigits_armed: _,
            program_path: _,
            required_paths: _,
            required_packages: _,
            untranslated: _,
            requires_installing: _,
            // Bytes and paths, no `ObjRef` in any of them: the interpreter's
            // own environment, current directory and `SETLOCAL` snapshots are
            // not the collector's.
            env: _,
            library_search: _,
            cwd: _,
            locals: _,
            // The embedding's own writers, which take bytes and hold nothing
            // of this interpreter's.
            sinks: _,
            // Three flags describing the embedding's descriptors, and the
            // command-line words as bytes: no `ObjRef` in either, so nothing
            // here is reachable from the collector.
            standard_transient: _,
            command_words: _,
        } = self;
        // The raise's own `ADDITIONAL`, alive between the raise and the
        // condition object that will hold it.
        out.extend(*pending_additional);
        out.extend(*pending_result);
        out.extend(global_references.roots());
        // Everything a native call has been handed, and the receiver it is
        // writing object variables through. Held here rather than by the
        // collector's other routes because an extension's handle is the only
        // reference to it: nothing on the Rexx side names an object a native
        // method allocated and has not returned yet.
        for frame in native_handles {
            out.extend(frame.locals.roots());
            out.extend([frame.owner, frame.scope, frame.receiver]);
            out.extend(frame.arguments.iter().copied().flatten());
            out.extend(frame.argument_list);
            out.extend(
                [frame.additional, frame.result, frame.condition]
                    .into_iter()
                    .flatten(),
            );
        }
        // A manager is an ordinary program object held by nothing else: the
        // package that carries it is a plan, not an object with a slot.
        out.extend(security_managers.values().copied());
        // Each queued trap's condition object. Destructured rather than
        // reached by field: the match above guards `Interp`'s own fields, and
        // an `ObjRef` added inside this `VecDeque` would otherwise arrive
        // unrooted with nothing to say so.
        for PendingTrap {
            condition: _,
            rc: _,
            description: _,
            object,
            activation: _,
            queued_during_delivery: _,
            fragment_depth: _,
        } in pending_traps
        {
            out.extend(*object);
        }
        // The context objects of the activations on the stack. **The one
        // object an activation owns outright**: everything else it holds is
        // rooted by its slot frame, by `Interp::class_variables`, or -- a
        // send's receiver -- by the temporary `Interp::message_term` takes
        // over the sending clause. A `RexxContext` is created by
        // `Interp::context_object` and stored on the activation, and nothing
        // else refers to it. Handed over here rather than kept rooted per
        // activation because the alternative is a global root whose key has
        // to be minted, replaced and retired as activations come and go, and
        // this pays only when a collection actually happens.
        // `Activation::object_roots` is the same objects' other route, for an
        // activation a `REPLY` has parked.
        // **A package pins the classes it declares**, which is the oracle's
        // behaviour and not a convenience: measured, a `::CLASS` class
        // survives a forced collection there, because nothing can drop the
        // binding. Since Phase 5j a class is an ordinary object, so these
        // tables hold arena handles and an unrooted one would dangle.
        for table in [
            merged_public_classes,
            package_classes,
            package_public_classes,
        ] {
            out.extend(table.values().flat_map(|names| names.values().copied()));
        }
        out.extend(
            running
                .iter()
                .map(std::ops::Deref::deref)
                .chain(suspended.iter().map(Box::as_ref))
                .filter_map(|activation| activation.context_object),
        );
        // The trapped condition's object, which a `CALL ON` handler's
        // activation and every callee that inherits its `CONDITION()` hold
        // once the queue has handed it over.
        out.extend(
            running
                .iter()
                .map(std::ops::Deref::deref)
                .chain(suspended.iter().map(Box::as_ref))
                .filter_map(|activation| activation.condition.as_ref()?.object),
        );
    }

    /// The collection itself, kept out of [`Interp::collect_if_due`]'s body so
    /// that what an allocation pays when nothing is due is the test alone.
    #[inline(never)]
    fn collect_now(&mut self) {
        // Everything the interpreter holds outside `RootSet`, handed to the
        // collector as temporaries for the length of the sweep.
        let mut anchor: Vec<ObjRef> = Vec::new();
        self.object_roots(&mut anchor);
        let frame = self.roots.push_frame();
        for object in anchor {
            self.roots.push_temp(object);
        }
        let stats = self.heap.collect(&self.roots);
        self.roots.pop_frame(frame);
        let heap = &self.heap;
        self.kept_strings.retain(|object, _| {
            !matches!(object.decode(), rexx_core::Decoded::Heap { .. })
                || heap.get(*object).is_some()
        });
        // A sweep can free a class the registry named, which unlinks its row
        // and leaves any `.NAME` answer derived from it naming nothing.
        self.invalidate_rexx_class_cache();
        // `pending_uninit` is what the collector resurrected so a finalizer
        // could run against a whole graph. The finalizer is not sent from
        // here: the oracle's collector only marks
        // (`MemoryObject::checkUninit`), and `runUninits` is reached from
        // `GC('force')` and from the termination sweep.
        self.uninit_ready.extend(stats.pending_uninit);
        // The rows `rexx-classes` keys by a class go with the class. Done
        // here rather than in a pass of its own: the sweeper already knows
        // which slots it freed, and a scan of the registry per collection
        // would cost the whole class population to find the few that died.
        if !stats.freed_classes.is_empty() {
            let dead = stats.freed_classes;
            self.classes().expunge(&dead);
            for class in &dead {
                self.class_variables.remove(class);
                self.class_packages.remove(class);
            }
            self.method_objects
                .retain(|(class, _), _| !dead.contains(class));
        }
        // **Not raised for the stress mode**, which collects on every
        // allocation by definition and must not have its watermark moved
        // out from under it.
        if !self.stress_collect {
            self.collect_at = COLLECT_FLOOR.max(stats.live.saturating_mul(2));
        }
    }

    /// Flags a class carrying a class-side `UNINIT` so the collector
    /// resurrects it rather than freeing it, exactly as it does for an
    /// instance.
    pub(crate) fn flag_class_uninit(&mut self, class: ObjRef) {
        if self.classes().has_pending_class_uninit(class) {
            self.heap.set_uninit(class);
        }
    }

    /// Records that `class` keeps `object` alive.
    fn class_owns(&mut self, class: ObjRef, object: ObjRef) {
        match self.heap.get_mut(class).map(|held| &mut held.body) {
            Some(rexx_core::Body::Class { owned }) => owned.push(object),
            _ => panic!("class_owns on a handle that is not a live class object"),
        }
    }

    /// A class object a program made, which the collector may take.
    fn mint_class(&mut self) -> ObjRef {
        let class = self.alloc_with(
            rexx_core::BehaviourId::OBJECT,
            rexx_core::Body::Class { owned: Vec::new() },
        );
        self.roots.push_temp(class);
        class
    }

    /// [`alloc_with`], for an object the collector must never take.
    fn alloc_immortal_with(
        &mut self,
        behaviour: rexx_core::BehaviourId,
        body: rexx_core::Body,
    ) -> ObjRef {
        self.collect_if_due();
        self.heap.alloc_immortal(behaviour, body)
    }

    // ---- values ----

    // `eval`/`eval_node`/`stack_span` live in `eval.rs` (Task 7), beside the
    // operators they evaluate. `depth`/`max_depth`/`stack_entry`/
    // `stack_first`/`stack_deepest` stay here, on `Interp`'s own struct
    // definition, exactly like every other field a sibling module's
    // `impl Interp` block reaches into.
}

// ---- the public entry point ----

/// Runs a Rexx program and returns what it produced.
pub fn run_program(path: &str, text: Vec<u8>, invocation: Invocation) -> Outcome {
    let path = path.to_string();
    on_interpreter_thread(move || execute(&path, text, false, invocation))
}

/// `run_program`, except that `Heap::collect` runs after every allocation
/// instead of never. The 4a exit gate's criterion 4: the named L0 subset
/// has to pass again under this mode, with the mode proved to have actually
/// collected (`Outcome::collections` non-zero) rather than merely having
/// been requested.
#[doc(hidden)]
pub fn run_program_collect_every_alloc(
    path: &str,
    text: Vec<u8>,
    invocation: Invocation,
) -> Outcome {
    let path = path.to_string();
    on_interpreter_thread(move || execute(&path, text, true, invocation))
}

/// Every body of `text`, compiled to a chunk and rendered as text: what the
/// `rexx-ir` binary prints.
pub fn render_ir(text: Vec<u8>, setting: &[u8]) -> Result<String, String> {
    let mode = trace::mode_from_setting(setting).map_err(|byte| {
        format!(
            "not a TRACE setting: {} (at {:?})",
            String::from_utf8_lossy(setting),
            char::from(byte)
        )
    })?;
    let trace = trace::ChunkTrace::of(mode);
    let program = rexx_parse::parse_program(text).map_err(|error| format!("{error:?}"))?;

    let mut out = String::new();
    for (body, what, kind) in ir_bodies(&program) {
        out.push_str(&format!("=== {what} ===\n"));
        let plan = plan::Plan::build(body, &program.symbols, Some(&program.source), kind);
        match ir::compile(body, &plan, trace) {
            Ok(chunk) => out.push_str(&ir::render_annotated(&chunk, body, &program.source)),
            Err(error) => out.push_str(&format!("(refused: {error:?})\n")),
        }
    }
    Ok(out)
}

// ---- the registry projection ----

/// One row of the `LIBRARY REXX` entry-point registry (D37), for a caller
/// that walks it.
pub struct NativeEntryPoint {
    /// The name the `REXX` package exports the entry point under, spelled as
    /// `interpreter/runtime/NativeMethods.h` spells it. The lookup that
    /// matches it is caseless.
    pub entry: &'static str,
    /// The family it belongs to, lower case: the interpreter subsystem whose
    /// C++ translation unit defines it.
    pub family: &'static str,
    /// The phase that owes this entry point a body, or `None` for one this
    /// crate runs. Spelled as every other owner string in this crate is.
    pub owner: Option<&'static str>,
    /// Whether this phase runs the entry point rather than refusing a send to
    /// it. A bind succeeds either way -- what it decides is whether the
    /// declaring *file* installs.
    pub implemented: bool,
}

/// Every entry point a `::METHOD ... EXTERNAL 'LIBRARY REXX name'` can bind
/// to. See [`NativeEntryPoint`].
pub fn native_entry_points() -> Vec<NativeEntryPoint> {
    dispatch::native::entry_points().collect()
}

/// Every body [`render_ir`] compiles, in source order, each with the name it is
/// printed under.
fn ir_bodies(program: &rexx_parse::Program) -> Vec<(&rexx_parse::CodeBody, String, BodyKind)> {
    use rexx_parse::DirectiveKind;

    let mut out = vec![(&program.main, "main".to_string(), BodyKind::Plain)];
    for directive in &program.directives {
        let (body, what, kind) = match &directive.kind {
            DirectiveKind::Method(method) => (method.body.as_ref(), "::METHOD", BodyKind::Method),
            DirectiveKind::Attribute(attribute) => {
                (attribute.body.as_ref(), "::ATTRIBUTE", BodyKind::Method)
            }
            DirectiveKind::Routine(routine) => {
                (routine.body.as_ref(), "::ROUTINE", BodyKind::Plain)
            }
            DirectiveKind::Annotate(_)
            | DirectiveKind::Class(_)
            | DirectiveKind::Constant(_)
            | DirectiveKind::Options(_)
            | DirectiveKind::Requires(_)
            | DirectiveKind::Resource(_) => (None, "", BodyKind::Plain),
        };
        if let Some(body) = body {
            out.push((body, what.to_string(), kind));
        }
    }
    out
}

/// Runs `body` on a thread with `INTERPRETER_STACK_BYTES` of stack.
fn on_interpreter_thread(body: impl FnOnce() -> Outcome + Send + 'static) -> Outcome {
    let interpreter = std::thread::Builder::new()
        .name("rexx-interp".to_string())
        .stack_size(INTERPRETER_STACK_BYTES)
        .spawn(body)
        .expect("spawning the interpreter thread");
    match interpreter.join() {
        Ok(outcome) => outcome,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

/// Everything that happens on the interpreter thread: parse, run, report.
fn execute(
    path: &str,
    text: Vec<u8>,
    collect_every_alloc: bool,
    invocation: Invocation,
) -> Outcome {
    let program = match parse_program(text) {
        Ok(program) => program,
        // **A top-level parse failure stays loud, and that was checked rather
        // than assumed either way.** The `ParseError`-to-`Raised` conversion
        // exists and `INTERPRET` uses it
        // (`run_fragment`). This arm can have the *mapping* -- it is one
        // `impl From` and nothing about it is fragment-specific -- but not
        // the *report*, and the obstacle is concrete rather than a
        // preference: `Raised::report`'s major line names a source line, the
        // line comes from `ParseError::line(&source)`, and `parse_program`
        // takes `text` by value and returns only the `ParseError` on the
        // failure path, so by the time this arm runs the `ProgramSource` that
        // could answer has been built and dropped inside the parser. There is
        // no way back to it from here: `rexx-parse` exposes `ProgramSource::
        // new` and `scan`, but the composition that turns a `&ProgramSource`
        // into a `Program` is private, so the only route is to clone the
        // whole program text before every parse to serve a path that runs
        // only on syntax errors. Closing it properly is a `rexx-parse`
        // signature change -- hand the source back alongside the error, or
        // make the `parse(&ProgramSource)` composition public -- which is
        // outside the file list Task 2 was given, so it is written down here
        // rather than half-done. The second gap `INTERPRET` shares is the
        // clause echo: the failing clause never became an `Instruction`, and
        // `ParseError` carries the clause's *start* byte with no end, so
        // there is no span to echo at either level.
        Err(error) => {
            return Outcome {
                exit_code: NOT_IMPLEMENTED_EXIT,
                stdout: Vec::new(),
                stderr: format!("rexx-exec: {error}\n").into_bytes(),
                stack: StackSpan::default(),
                collections: 0,
                chunks_refused: 0,
            };
        }
    };

    let mut interp = Interp::new();
    interp.program_path = path.to_string();
    if collect_every_alloc {
        interp.enable_stress_collect();
    }
    // The command line is the top-level program's caller, so what it supplied
    // goes into the same `call_context` a `CALL` fills -- see that field's own
    // doc for what reads it and for the three measured invocations that tell
    // "no argument" from "one empty argument" apart.
    interp.call_context.name = path.as_bytes().to_vec();
    let parts = invocation.into_parts();
    interp.roots.set_frame_block(parts.frame_block);
    let (argument, deadline) = (parts.argument, parts.deadline);
    interp.input = Input::new(parts.input);
    interp.standard_transient = parts.standard_transient;
    interp.sinks = parts.sinks;
    interp.command_words = parts.words;
    if let Some(directory) = parts.directory {
        interp.cwd = directory;
    }
    if let Some(environment) = parts.environment {
        interp.adopt_environment(environment);
    }
    // Armed here rather than in `Interp::new`, and after the parse, so that
    // what it bounds is the running of this program. `Interp::bootstrap_library`
    // below runs clauses of its own and is inside the bound, which is what a
    // caller asking for a bounded run wants: a bootstrap that did not finish
    // is a run that did not finish.
    interp.deadline = deadline.map(crate::clause::Deadline::starting_now);
    if interp.deadline.is_some() {
        interp.clause_countdown = crate::clause::Deadline::CLAUSES_PER_CHECK;
    }
    // **The library bootstrap runs before the command line's own program is
    // installed or run, and before its argument string exists.** The program
    // has already been *parsed* above, which is where a syntax error is
    // reported from and is why that report does not wait on this. The
    // library's classes have to be
    // in `.environment` by the time the program's first clause runs --
    // `Interp::bootstrap_library` carries why that is at start rather than on
    // demand -- and running it first leaves nothing of this program's to keep
    // reachable across the allocation it does.
    let result = interp.bootstrap_library().and_then(|()| {
        if let Some(argument) = argument {
            let value = interp.text(&argument);
            // Rooted with a `push_temp` taken before `run`, which is what makes it
            // outlive every clause: `Op::Clause`'s region truncates the
            // temporaries stack back to a watermark it takes on entry, and every
            // such watermark sits above this push. This is the same mechanism
            // `Interp::invoke_call` uses to keep a call's own arguments reachable
            // (`run/call.rs`, the `push_temp(argument.value())` beside the argument
            // list it builds); `call_context` itself is not walked by the
            // collector, so without this the value is unreachable the first time
            // anything allocates.
            interp.roots.push_temp(value);
            interp.call_context.arguments = Rc::from(&[Some(value)][..]);
        }
        interp.run(program)
    });
    // The whole echo stack, innermost first: the levels `seal_site_level`
    // already closed, then the level that was still unwinding when the
    // condition reached the top. See `Interp::failure_sites` for why the two
    // are separate fields, and `Raised::report` for what the order means.
    let mut failure_sites = std::mem::take(&mut interp.failure_sites);
    failure_sites.extend(interp.failure_site.take());
    // `exit_code_for` needs `&mut interp` (`to_number` fills a lazy cache),
    // so this has to run before `interp.trace`/`interp.out` move out of
    // `interp` below -- a partial move of one field ends `interp`'s usability
    // as a whole value, and every other call above this one only reads or
    // takes a single field, never the whole struct.
    let mut exit_code = match result {
        // `Failure::Exited` is not a failure -- it is `EXIT` (or falling off
        // the routine's own end) reached through `ExprKind::Call`'s
        // expression form, tunnelled here through `Err`/`?` only because
        // `eval`'s own return type has no `Flow` to carry it through instead
        // (`Failure::Exited`'s own doc, `error.rs`, has the full argument).
        // Treated exactly like an ordinary `Ok(value)`: same exit-code rule,
        // no stderr report, because it is not one.
        Ok(value) | Err(Failure::Exited(value)) => interp.exit_code_for(value),
        Err(Failure::Loud(loud)) => {
            interp
                .trace
                .extend_from_slice(format!("rexx-exec: {}\n", loud.message).as_bytes());
            NOT_IMPLEMENTED_EXIT
        }
        Err(Failure::Raised(raised)) => {
            // `run_activation` records the site on the way out. An empty
            // stack here would mean a condition escaped without passing an
            // instruction loop, which nothing in this crate can do; it
            // renders visibly rather than panicking, on the error path's
            // standing rule that a reportable condition must never become a
            // crash.
            if failure_sites.is_empty() {
                failure_sites.push(FailureSite::Clause {
                    line: 0,
                    text: b"<no failing clause recorded>".to_vec(),
                    indent: 0,
                });
            }
            let site = ClauseSite {
                path,
                sites: &failure_sites,
            };
            interp.write_trace_report(&raised.report(&site));
            raised.exit_code()
        }
        // No report and no status here: the run may still have deferred
        // bodies and `UNINIT`s to abandon, and the one place that says a
        // deadline fired is the guard below them.
        Err(Failure::Deadline) => 0,
    };

    // **After the main body's own report and after its exit status is
    // settled**, which is the order the oracle produces: the main activity
    // writes its traceback when it fails and the replied remainder runs on
    // afterwards. Measured, oracle rc 7 on a program ending `exit 7` whose
    // replied method then raises 98.936 -- the traceback is on stderr and the
    // status is the main body's, so a raise here only writes.
    for (failure, mut sites) in interp.run_deferred_replies() {
        match failure {
            // `Interp::resume_reply` answers `Ok` for this variant, exactly as
            // `Interp::enter_method_body` does; the arm is what makes this
            // match exhaustive and nothing else.
            Failure::Exited(_) => {}
            // The guard below the `UNINIT` sweep is what reports this, for
            // the reason the main body's own arm gives.
            Failure::Deadline => {}
            Failure::Loud(loud) => {
                interp
                    .trace
                    .extend_from_slice(format!("rexx-exec: {}\n", loud.message).as_bytes());
                exit_code = NOT_IMPLEMENTED_EXIT;
            }
            Failure::Raised(raised) => {
                if sites.is_empty() {
                    sites.push(FailureSite::Clause {
                        line: 0,
                        text: b"<no failing clause recorded>".to_vec(),
                        indent: 0,
                    });
                }
                let site = ClauseSite {
                    path,
                    sites: &sites,
                };
                interp.write_trace_report(&raised.report(&site));
            }
        }
    }

    // `MemoryObject::lastChanceUninit` (`memory/RexxMemory.cpp:324`), reached
    // from `Interpreter::terminateInterpreter` (`runtime/Interpreter.cpp:279`)
    // -- after everything the program and its replied bodies do, and reached
    // whatever the program's own outcome was. Measured, oracle: a program
    // whose main body raises 42.3 still prints its class `UNINIT` and exits
    // 214, and one ending `exit 7` prints it and exits 7.
    for loud in interp.terminate() {
        interp
            .trace
            .extend_from_slice(format!("rexx-exec: {}\n", loud.message).as_bytes());
        exit_code = NOT_IMPLEMENTED_EXIT;
    }

    // **The one place a deadline becomes an answer**, below everything that
    // runs clauses, so that a run abandoned part-way cannot report the status
    // its main body happened to reach. It is here rather than in the three
    // arms above because one of the paths between them discards failures on
    // purpose: `Interp::run_one_uninit` throws away a raised condition and an
    // `EXIT` because the oracle's own dispatcher does, and a deadline reaching
    // it would otherwise vanish. `Interp::deadline_expired` reads the flag the
    // check itself set, not the clock, so a run that finished inside its bound
    // is untouched however narrow the margin was.
    if interp.deadline_expired() {
        interp.trace.extend_from_slice(DEADLINE_REPORT);
        exit_code = DEADLINE_EXIT;
    }

    // Read after the deferred bodies above, so a collection or a refused chunk
    // inside one is counted: `run_program_collect_every_alloc` decides that its
    // mode ran from `collections`, and a resumed body allocates like any other.
    let stack = interp.stack_span();
    let collections = interp.heap.collections_performed() - interp.collections_before_program;
    let chunks_refused = interp.chunks_refused;

    Outcome {
        exit_code,
        stdout: interp.out,
        stderr: interp.trace,
        stack,
        collections,
        chunks_refused,
    }
}

#[cfg(test)]
mod tests;
