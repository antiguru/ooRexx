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

use rexx_classes::{ClassKind, InheritRefusal, MethodId};
use rexx_core::{Body, Heap, NameMap, ObjRef, RootSet, SlotFrame, SlotRef};
use rexx_num::Settings;
use rexx_parse::{
    Access, AnnotationTarget, AttributeDirective, AttributeStyle, ClassDirective, ClassRef,
    CodeBody, ConstantDirective, ConstantValue, Directive, DirectiveKind, Expr, ExprKind,
    GuardOption, InstructionKind, MethodDirective, Operator, Program, Protection, SymbolId,
    SymbolTable, compound_parts, parse_program,
};
use std::collections::{BTreeMap, HashMap, VecDeque};
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
pub use invocation::{Invocation, ProgramInput, join_command_line};

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
mod builtin;

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

    /// A call that resolved to a **builtin this crate runs nothing for**.
    fn unresolved_call(name: &[u8]) -> Loud {
        const LIMIT: usize = 128;
        let shown = if name.len() > LIMIT {
            format!("{}...", String::from_utf8_lossy(&name[..LIMIT]))
        } else {
            String::from_utf8_lossy(name).into_owned()
        };
        Loud {
            message: owned_message(&format!("routine \"{shown}\""), Some("4c")),
        }
    }

    /// A message sent to a value whose class this phase does not build, so
    /// there is no behaviour to resolve the name against at all.
    fn receiver_class(kind: &str) -> Loud {
        Loud {
            message: owned_message(&format!("a message send to {kind}"), Some("Phase 5")),
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

    /// A method compiled from source text, in one of the shapes or places
    /// [`compile_method_source`] does not take.
    fn executable_context() -> Loud {
        Loud {
            message: owned_message("a newFile package context", Some("Phase 7")),
        }
    }

    /// `loadExternalMethod` and `loadExternalRoutine` for an entry point this
    /// phase cannot resolve.
    fn external_entry_point(what: &'static str) -> Loud {
        Loud {
            message: owned_message(what, Some("Phase 7")),
        }
    }

    fn security_manager() -> Loud {
        Loud {
            message: owned_message("a security manager", Some("D12, Phase 7")),
        }
    }

    /// `Package~options(name, value)` and `Package~defaultOptions(name,
    /// value)`, each of which writes a package setting rather than reading
    /// one.
    fn package_option_write() -> Loud {
        Loud {
            message: owned_message("a package settings write", Some("D12, Phase 7")),
        }
    }

    /// `Package~loadPackage(name, source)`, whose second argument builds a
    /// package out of source lines under a name that is not a file.
    fn package_from_source() -> Loud {
        Loud {
            message: owned_message("a loadPackage source array", Some("Phase 7")),
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

    /// `VALUE`'s three-argument form: a *present* third argument selects an
    /// external pool rather than this crate's own local variables
    /// (`expression/BuiltinFunctions.cpp:1848`-`1913`).
    fn value_selector() -> Loud {
        Loud {
            message: "VALUE's external-selector form is not implemented".to_string(),
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

/// Every class this `::CLASS` names: its `SUBCLASS`/`MIXINCLASS` target (one
/// slot, `mixin` telling the two apart), its `METACLASS`, and each entry of
/// its `INHERIT` list.
fn class_references(class: &ClassDirective) -> impl Iterator<Item = &ClassRef> {
    class
        .subclass
        .iter()
        .chain(class.metaclass.iter())
        .chain(class.inherit.iter())
}

/// The gap a `::` directive declares at install time, or `None` for one this
/// crate can install.
fn directive_gap(kind: &DirectiveKind) -> Option<Loud> {
    let gap = |name: &str, owner: &'static str| {
        Some(Loud {
            message: owned_message(name, Some(owner)),
        })
    };
    match kind {
        // Binds an entry point before `main` and whether or not the routine
        // is ever called. **Every one of its forms stays here, the
        // `LIBRARY REXX` one included**, and that is worth saying because the
        // `::METHOD` arm below moves exactly that spelling: a routine
        // resolves against `rexx_routines[]`, which
        // `dispatch::native`'s registry is not
        // (`runtime/InternalPackage.cpp:230`, from `NativeFunctions.h`).
        // Measured, oracle: `::routine r external "LIBRARY nosuchlib
        // nosuchfn"` and the same without the third word are both 98.903 rc
        // 158 with stdout empty; `"LIBRARY REXX file_separator"` is 90.999 rc
        // 166, naming a method as a routine it cannot find; `"LIBRARY REXX
        // Filespec"` is rc 0 and the routine runs.
        DirectiveKind::Routine(routine) if routine.external.is_some() => {
            gap("::ROUTINE EXTERNAL", "Phase 7")
        }
        // **The `EXTERNAL` forms this phase binds are the ones whose library
        // is `REXX`**, and `dispatch::native::method_external` and its
        // `::ATTRIBUTE` half are what decide that -- read here and again by
        // `Interp::install_directives`, so the forms that bind and the forms
        // that are refused cannot come apart. An entry point the `REXX`
        // package does not export is 90.998 in that walk and not a gap here:
        // the oracle answers it, so it is a differential row rather than a
        // refusal.
        DirectiveKind::Method(method) => match dispatch::native::method_external(method) {
            None
            | Some(
                dispatch::native::MethodExternal::LibraryRexx(_)
                | dispatch::native::MethodExternal::Attribute(_),
            ) => None,
            // Loads a shared library, which is Phase 7's, exactly as
            // `::ROUTINE EXTERNAL` above does.
            Some(dispatch::native::MethodExternal::OtherLibrary) => gap(
                "::METHOD EXTERNAL naming a library other than REXX",
                "Phase 7",
            ),
        },
        DirectiveKind::Attribute(attribute) => {
            match dispatch::native::attribute_external(attribute) {
                Some(dispatch::native::MethodExternal::OtherLibrary) => gap(
                    "::ATTRIBUTE EXTERNAL naming a library other than REXX",
                    "Phase 7",
                ),
                _ => None,
            }
        }
        // Loads a shared library rather than a package file, which is Phase
        // 7's exactly as `::ROUTINE EXTERNAL` above is.
        DirectiveKind::Requires(requires) if requires.library => {
            gap("::REQUIRES LIBRARY", "Phase 7")
        }
        // `::OPTIONS` installs (`Interp::install_directives`' own walk): it
        // resolves no name, runs no code, and every setting it writes is one
        // an activation of this package's code starts from.
        DirectiveKind::Annotate(_)
        | DirectiveKind::Class(_)
        | DirectiveKind::Constant(_)
        | DirectiveKind::Options(_)
        | DirectiveKind::Requires(_)
        | DirectiveKind::Resource(_)
        | DirectiveKind::Routine(_) => None,
    }
}

/// The `LIBRARY REXX` entry point a directive's `EXTERNAL` names and the
/// `REXX` package does not export -- the one the oracle reports 90.998 for --
/// or `None` for a directive whose `EXTERNAL` resolves and for one carrying
/// none.
fn unresolved_external(kind: &DirectiveKind) -> Option<Vec<u8>> {
    let external = match kind {
        DirectiveKind::Method(method) => dispatch::native::method_external(method),
        DirectiveKind::Attribute(attribute) => dispatch::native::attribute_external(attribute),
        _ => None,
    };
    dispatch::native::unresolved_entry(external.as_ref()).map(<[u8]>::to_vec)
}

/// The refusal a directive stage owes, or `None` when every directive the
/// stage selects installs.
/// ```text
/// ::routine/::method/::attribute EXTERNAL  vs a failing ::CLASS  98.903 rc 158, the EXTERNAL line
/// ::routine/::method/::attribute EXTERNAL  vs a ::CLASS cycle    98.903 rc 158, the EXTERNAL line
/// ::routine EXTERNAL                       vs ::requires         98.903 rc 158, the EXTERNAL line
/// ::annotate routine nosuch                vs a failing ::CLASS  99.945 rc 157, the ::ANNOTATE line
/// ::annotate routine nosuch                vs a ::CLASS cycle    99.945 rc 157, the ::ANNOTATE line
/// ::annotate routine nosuch                vs ::routine EXTERNAL whichever is FIRST in the file
/// ::requires 'nosuch.rex'                  vs a failing ::CLASS  43.901 rc 213, the ::REQUIRES line
/// ::requires 'nosuch.rex'                  vs a ::CLASS cycle    98.911 rc 158, the cycle's root
/// ::options digits 12                      vs a failing ::CLASS  98.909 rc 158, the ::CLASS line
/// ::class q metaclass zzz                  vs a failing ::CLASS  98.908 or 98.909, whichever is first
/// ```
/// ```text
/// ::class a / ::constant kk (1/0) / ::routine r external
///                              'LIBRARY REXX Filespec'                 42.3 rc 214
/// ```
/// ```text
/// ::routine r / ::annotate routine r / ::class a subclass zzznotaclass  98.909 rc 158
/// ::class a / ::constant kk (1/0) / ::routine r / ::annotate routine r  42.3 rc 214
/// ::routine r / ::annotate routine r / a duplicate ::ROUTINE pair       99.903 rc 157
/// ::routine r / ::annotate routine r / a class-less ::constant (1/0)    99.906 rc 157
/// ```
fn staged_gap(program: &Program, stage: fn(&DirectiveKind) -> bool) -> Option<Loud> {
    program
        .directives
        .iter()
        .filter(|directive| stage(&directive.kind))
        .find_map(|directive| directive_gap(&directive.kind))
}

/// The classes of the file being installed that a `::CLASS`'s own reference
/// can resolve against: the index of every `::CLASS` the file declares, and
/// the object each of the ones installed so far became.
struct FileClasses<'a> {
    declared: &'a HashMap<Box<[u8]>, usize>,
    installed: &'a HashMap<usize, ObjRef>,
}

/// `directive`'s own line number and clause text, as a traceback echoes them.
fn directive_clause(program: &Rc<Program>, directive: &Directive) -> (usize, Vec<u8>) {
    let line = program.source.line_of(directive.clause_span.start);
    let text = program
        .source
        .join_span(directive.clause_span.clone())
        .map_or_else(
            || b"<clause span outside the retained source>".to_vec(),
            |bytes| bytes.into_owned(),
        );
    (line, text)
}

/// The directives this one must be installed after: every class it names
/// that this file also declares, unqualified.
fn class_dependencies<'a>(
    class: &'a ClassDirective,
    declared: &'a HashMap<Box<[u8]>, usize>,
) -> impl Iterator<Item = usize> + 'a {
    class_references(class)
        .filter(|target| target.namespace.is_none())
        .filter_map(|target| declared.get(target.name.as_ref()).copied())
}

/// The order a file's `::CLASS` directives install in: each class after every
/// class it names, when that target is declared in the same file.
/// ```text
/// blame the LAST ::CLASS directive in the file:
///     directive_constant_blames_the_last_installed_class.rex differs, 105 of 106
/// blame the FIRST ::CLASS directive in the file:
///     directive_constant_expression_blames_the_last_class.rex differs, 105 of 106
/// ```
fn class_install_order(
    program: &Program,
    declared: &HashMap<Box<[u8]>, usize>,
) -> Result<Vec<usize>, usize> {
    /// Where a directive is in the walk. Absent from the map is "not yet
    /// looked at".
    enum Visit {
        OnStack,
        Done,
    }
    let mut state: HashMap<usize, Visit> = HashMap::new();
    let mut order = Vec::new();
    for root in 0..program.directives.len() {
        if !matches!(program.directives[root].kind, DirectiveKind::Class(_)) {
            continue;
        }
        // The walk from this root, innermost pending class last.
        let mut stack = vec![root];
        while let Some(index) = stack.last().copied() {
            if matches!(state.get(&index), Some(Visit::Done)) {
                stack.pop();
                continue;
            }
            let DirectiveKind::Class(class) = &program.directives[index].kind else {
                state.insert(index, Visit::Done);
                stack.pop();
                continue;
            };
            // The classes this one names that the file declares. A target the
            // file does not declare is the registry's and is nobody's
            // predecessor here.
            let mut waiting = false;
            for target in class_dependencies(class, declared) {
                match state.get(&target) {
                    // Already on this walk without having finished, so every
                    // class from it up to here is waiting on it. A class
                    // naming itself reaches this on its second visit, which
                    // is the one-entry case of the same thing.
                    Some(Visit::OnStack) => return Err(root),
                    Some(Visit::Done) => {}
                    None => {
                        stack.push(target);
                        waiting = true;
                    }
                }
            }
            if waiting {
                state.insert(index, Visit::OnStack);
            } else {
                state.insert(index, Visit::Done);
                order.push(index);
                stack.pop();
            }
        }
    }
    Ok(order)
}

/// The dictionary keys a `::METHOD` directive claims, each with what a send
/// reaching it runs.
fn method_dictionary_keys(method: &MethodDirective) -> Vec<(Vec<u8>, Option<GeneratedKind>)> {
    let upper = method.name.to_ascii_uppercase();
    if method.delegate.is_some() {
        let delegate = Some(GeneratedKind::Delegate);
        if method.attribute {
            vec![(accessor_setter_name(&upper), delegate), (upper, delegate)]
        } else {
            vec![(upper, delegate)]
        }
    } else if method.attribute {
        let setter = accessor_setter_name(&upper);
        let (get, set) = if method.abstract_ {
            (Some(GeneratedKind::Abstract), Some(GeneratedKind::Abstract))
        } else if method.external.is_some() {
            (None, None)
        } else {
            (Some(GeneratedKind::Getter), Some(GeneratedKind::Setter))
        };
        vec![(upper, get), (setter, set)]
    } else if method.abstract_ {
        vec![(upper, Some(GeneratedKind::Abstract))]
    } else {
        vec![(upper, None)]
    }
}

/// The dictionary keys an `::ATTRIBUTE` directive claims -- the plain name
/// for a getter, the name with `=` appended for a setter, both for the
/// default (neither `GET` nor `SET`) style.
fn attribute_dictionary_keys(
    attribute: &AttributeDirective,
) -> Vec<(Vec<u8>, Option<GeneratedKind>)> {
    let upper = attribute.name.to_ascii_uppercase();
    let setter_name = accessor_setter_name(&upper);
    let generated = if attribute.abstract_ {
        Some(GeneratedKind::Abstract)
    } else if attribute.delegate.is_some() {
        Some(GeneratedKind::Delegate)
    } else if attribute.external.is_some() || attribute.body.is_some() {
        None
    } else {
        // The one place what an accessor is depends on which half of the pair
        // it is.
        Some(GeneratedKind::Getter)
    };
    // The setter's half of that one place.
    let setter = match generated {
        Some(GeneratedKind::Getter) => Some(GeneratedKind::Setter),
        other => other,
    };
    match attribute.style {
        AttributeStyle::Both => vec![(upper, generated), (setter_name, setter)],
        AttributeStyle::Get => vec![(upper, generated)],
        AttributeStyle::Set => vec![(setter_name, setter)],
    }
}

/// The dictionary keys one member directive claims, each with the side it
/// claims them on -- `true` for the class dictionary.
fn member_dictionary_keys(kind: &DirectiveKind) -> Vec<(Vec<u8>, bool)> {
    match kind {
        DirectiveKind::Method(method) => method_dictionary_keys(method)
            .into_iter()
            .map(|(name, _)| (name, method.class_method))
            .collect(),
        DirectiveKind::Attribute(attribute) => attribute_dictionary_keys(attribute)
            .into_iter()
            .map(|(name, _)| (name, attribute.class_method))
            .collect(),
        DirectiveKind::Constant(constant) => {
            let upper = constant.name.to_ascii_uppercase();
            vec![(upper.clone(), false), (upper, true)]
        }
        _ => Vec::new(),
    }
}

/// Which directives each `::CLASS` in `program` owns, keyed by that
/// `::CLASS`'s own index and in source order within a class.
fn class_members(program: &Program) -> HashMap<usize, Vec<usize>> {
    let mut members: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut current: Option<usize> = None;
    for (index, directive) in program.directives.iter().enumerate() {
        // A synthetic directive belongs to no class -- see the skip in
        // `Interp::install_directives`, which carries why an empty clause
        // span is what tells one from a written directive. This one is what
        // keeps a loaded file's main section out of that file's last class.
        if directive.clause_span.is_empty() {
            continue;
        }
        match &directive.kind {
            DirectiveKind::Class(_) => current = Some(index),
            DirectiveKind::Method(_) | DirectiveKind::Attribute(_) | DirectiveKind::Constant(_) => {
                if let Some(class) = current {
                    members.entry(class).or_default().push(index);
                }
            }
            _ => {}
        }
    }
    members
}

/// The [`rexx_core::RootSet::add_global`] key one `::CONSTANT`'s value is
/// held under.
fn constant_root_key(ProgramId(program): ProgramId, directive: usize) -> String {
    format!("constant:{program}:{directive}")
}

/// What an `::ANNOTATE` names, as the first install walk can express it.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum AnnotatedSite {
    /// `::ANNOTATE PACKAGE`, which names the running package and no
    /// directive.
    Package,
    /// The `::CLASS` or `::ROUTINE` directive named.
    Directive(usize),
    /// One dictionary entry of a method-shaped directive: the directive that
    /// declares it and the name it is filed under.
    Member(usize, Box<[u8]>),
}

/// An `::ANNOTATE` target the accumulated package does not hold: the
/// keyword's own spelling for the message, and the name that resolved to
/// nothing.
struct MissingTarget<'a> {
    kind: &'static str,
    name: &'a [u8],
}

/// Whether a member directive's methods are the *attribute* methods
/// `::ANNOTATE ATTRIBUTE` will accept -- `MethodClass::isAttribute()`.
fn is_attribute_method(kind: &DirectiveKind) -> bool {
    match kind {
        DirectiveKind::Attribute(_) => true,
        DirectiveKind::Method(method) => method.attribute,
        _ => false,
    }
}

/// `annotateDirective`'s target resolution (`parser/DirectiveParser.cpp:1940`):
/// which directives an `::ANNOTATE` annotates, or the target it could not
/// find.
fn annotation_target<'a>(
    program: &Program,
    target: &'a AnnotationTarget,
    current_class: Option<usize>,
    claimed: &HashMap<(Option<usize>, bool, Vec<u8>), usize>,
    declared_classes: &HashMap<Vec<u8>, usize>,
    declared_routines: &HashMap<Vec<u8>, usize>,
) -> Result<Vec<AnnotatedSite>, MissingTarget<'a>> {
    // `findMethod`'s own order, instance dictionary before class.
    let member = |name: &[u8]| -> Option<usize> {
        [false, true]
            .into_iter()
            .find_map(|side| claimed.get(&(current_class, side, name.to_vec())).copied())
    };
    // One half of an accessor pair, on the side asked and only where the
    // directive that claimed the name is an attribute directive.
    let accessor = |name: &[u8], side: bool| -> Option<usize> {
        let directive = claimed
            .get(&(current_class, side, name.to_vec()))
            .copied()?;
        is_attribute_method(&program.directives[directive].kind).then_some(directive)
    };
    match target {
        AnnotationTarget::Package => Ok(vec![AnnotatedSite::Package]),
        AnnotationTarget::Class(name) => declared_classes
            .get(&name.to_vec())
            .map(|index| vec![AnnotatedSite::Directive(*index)])
            .ok_or(MissingTarget {
                kind: "class",
                name,
            }),
        AnnotationTarget::Routine(name) => declared_routines
            .get(&name.to_vec())
            .map(|index| vec![AnnotatedSite::Directive(*index)])
            .ok_or(MissingTarget {
                kind: "routine",
                name,
            }),
        AnnotationTarget::Method(name) => member(name)
            .map(|index| vec![AnnotatedSite::Member(index, name.clone())])
            .ok_or(MissingTarget {
                kind: "method",
                name,
            }),
        AnnotationTarget::Constant(name) => member(name)
            .filter(|index| matches!(program.directives[*index].kind, DirectiveKind::Constant(_)))
            .map(|index| vec![AnnotatedSite::Member(index, name.clone())])
            .ok_or(MissingTarget {
                kind: "constant",
                name,
            }),
        AnnotationTarget::Attribute(name) => {
            let setter: Box<[u8]> = accessor_setter_name(name).into();
            let mut found = Vec::new();
            for side in [false, true] {
                for half in [name, &setter] {
                    if let Some(index) = accessor(half, side) {
                        found.push(AnnotatedSite::Member(index, half.clone()));
                    }
                }
                if !found.is_empty() {
                    break;
                }
            }
            if found.is_empty() {
                return Err(MissingTarget {
                    kind: "attribute",
                    name,
                });
            }
            Ok(found)
        }
    }
}

/// Why a resolved method's directive cannot be entered, or `None` when it
/// can -- the gate `Interp::enter_method_body` (`dispatch.rs`) takes before
/// it pushes anything.
fn method_body_gap(kind: &DirectiveKind) -> Option<Loud> {
    match kind {
        DirectiveKind::Method(method) => {
            if method.body.is_none() {
                Some(Loud::method_body("a ::METHOD with no body of its own"))
            } else {
                None
            }
        }
        DirectiveKind::Attribute(attribute) => {
            if attribute.body.is_none() {
                Some(Loud::method_body("a ::ATTRIBUTE with no body of its own"))
            } else {
                None
            }
        }
        other => Some(Loud::method_body(&format!(
            "a method installed by ::{}",
            other.keyword()
        ))),
    }
}

/// The dictionary key of a generated setter: the getter's key with `=`
/// appended.
pub(crate) fn accessor_setter_name(upper: &[u8]) -> Vec<u8> {
    let mut setter = upper.to_vec();
    setter.push(b'=');
    setter
}

/// The variable a generated accessor reads and writes: the directive's name
/// **as written**, which is not the accessor's own dictionary key.
fn accessor_variable(kind: &DirectiveKind) -> Option<&[u8]> {
    match kind {
        DirectiveKind::Attribute(attribute) => Some(&attribute.name),
        DirectiveKind::Method(method) => Some(&method.name),
        _ => None,
    }
}

/// The variable a `DELEGATE` method reads to find its target: the directive's
/// `DELEGATE` symbol, **not** the directive's own name.
fn delegate_variable(kind: &DirectiveKind) -> Option<SymbolId> {
    match kind {
        DirectiveKind::Attribute(attribute) => attribute.delegate,
        DirectiveKind::Method(method) => method.delegate,
        _ => None,
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
        // loudly; the one step behind those three that this crate skips is
        // the external file search, which is Phase 7's. So there is no
        // residual claim on the `CALL` keyword here at all.
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
        // **Arm-grained, the second variant in this match that is.** The three
        // forms that only name an environment -- `ADDRESS env`, `ADDRESS VALUE
        // expr` and the bare toggle -- are implemented and answer `None`.
        // `ADDRESS env command` issues a command, and a `WITH` redirection
        // configures where a command's streams go; both need the command
        // dispatch `InstructionKind::Command` needs, so they carry that same
        // owner rather than one of their own (D18).
        InstructionKind::Address(address) => {
            if address.command.is_some() || address.io.is_some() {
                Some("Phase 7")
            } else {
                None
            }
        }
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
        InstructionKind::Command { .. } => Some("Phase 7"),
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
        // `InstructionKind::Call`'s own comment above describes for `CALL`,
        // with the external file search behind those three being Phase 7's.
        // `>name`/`<name` answers a `VariableReference`, which `eval.rs`'s
        // own arm builds and `run.rs`'s `Interp::variable_reference` binds to
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
    /// The activation running right now, held in a field of its own rather
    /// than at the top of [`Interp::suspended`].
    running: Option<Box<Activation>>,
    /// The `TRACE` setting of whatever [`Interp::running`] holds, kept beside
    /// it rather than read through it.
    trace_cache: crate::trace::TraceMode,
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
    /// The public routines a program's `::REQUIRES` directives imported --
    /// `mergedPublicRoutines`, filled by `PackageClass::mergeRequired`
    /// (`classes/PackageClass.cpp:693`).
    merged_public_routines: HashMap<ProgramId, HashMap<Box<[u8]>, InstalledRoutine>>,
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
    /// The packages each program has imported, in the order they were added
    /// -- `PackageClass`'s `loadedPackages`, which `~importedPackages`
    /// answers a copy of.
    package_imports: HashMap<ProgramId, Vec<Package>>,
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
    /// A primitive or an `EXTERNAL` binding, which has no directive to report
    /// on: `BaseCode::getSource` answers an empty array
    /// (`execution/BaseCode.cpp:120`) and `BaseCode::setSecurityManager`
    /// answers `0` (`:133`).
    Native,
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

/// What one just-installed dictionary key resolves to, handed to
/// [`Interp::install_one_method`] by whichever installer minted it.
#[derive(Copy, Clone)]
enum InstallBody {
    /// The directive's own Rexx body: a row of [`Interp::method_bodies`].
    Written,
    /// A method the directive implements itself: a row of
    /// [`Interp::generated_methods`].
    Generated(GeneratedKind),
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
        Interp {
            heap: Heap::new(),
            roots: RootSet::new(),
            key_buffer: Vec::new(),
            value_buffer: Vec::new(),
            parse_buffers: Vec::new(),
            text_scratch: [0; crate::value::TEXT_SCRATCH],
            text_numbers: crate::value::TextNumbers::new(),
            result_buffer: std::cell::Cell::new(Vec::new()),
            running: None,
            suspended: Vec::new(),
            spare_activations: Vec::new(),
            programs: Vec::new(),
            package_options: HashMap::new(),
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
            package_imports: HashMap::new(),
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
            next_activation_id: 0,
            next_invocation: 0,
            current_case_text: None,
            indent_offset: 0,
            activation_indent: 0,
            failure_site: None,
            failure_sites: Vec::new(),
            clause_line_override: None,
            fragment_depth: 0,
            stress_collect: false,
            uninit_ready: Vec::new(),
            processing_uninits: false,
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
            random_seed: None,
            elapsed_anchor: None,
            pending_elapsed_reset: false,
            reqstr_armed: false,
            lostdigits_armed: false,
            program_path: String::new(),
            required_paths: HashMap::new(),
            required_packages: HashMap::new(),
            requires_installing: Vec::new(),
            trace_cache: crate::trace::TraceMode::OFF,
        }
    }

    // ---- loading and the activation stack ----

    /// Loads `program`, runs its main body in a fresh activation, and tears
    /// the activation down again.
    fn run(&mut self, program: Program) -> Result<Option<ObjRef>, Failure> {
        let program = Rc::new(program);
        let program_id = ProgramId(self.programs.len());
        self.programs.push(Rc::clone(&program));
        self.run_loaded(program, program_id, CallType::Command)
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
        let outcome = self.run_loaded(parsed, program_id, CallType::Command);
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
    fn run_loaded(
        &mut self,
        program: Rc<Program>,
        program_id: ProgramId,
        call_type: CallType,
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
        // No call site above a main body, so `::OPTIONS NUMERIC INHERIT` has
        // nothing to inherit and the package's own settings stand -- measured,
        // `::options digits 12 numeric inherit` alone in a file reports 12.
        self.start_from_package(&mut main, None);
        self.push_activation(main);

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

    /// Resolves every `::` directive of `program`, filling [`Interp::routines`]
    /// and refusing the ones this crate cannot resolve.
    /// ```text
    /// ::class foo
    /// ::class foo + ::method bar
    /// ::class foo + ::attribute baz
    /// ::constant kk 5
    /// ::resource foo ... ::END
    /// ::annotate package author 'me'
    /// ::annotate <target> <name>, with the target declared above it
    /// a loose ::method with no ::class
    /// ```
    /// ```text
    /// ::class foo subclass zzznotaclass     98.909 rc 158
    /// ::class foo metaclass zzznotaclass    98.908 rc 158
    /// ::class bar inherit zzznotaclass      98.909 rc 158
    /// ::requires 'no_such_file_zz.rex'      43.901 rc 213
    /// ::routine z external "LIBRARY nosuchlib nosuchfn"   98.903 rc 158
    /// ::method m external "LIBRARY nosuchlib nosuchfn"    98.903 rc 158
    /// ::method m external "LIBRARY REXX nosuchentry"      90.998 rc 166
    /// ::annotate routine nosuchrtn          99.945 rc 157
    /// duplicate ::routine of the same name  99.903 rc 157
    /// ```
    fn install_directives(&mut self, id: ProgramId, program: &Rc<Program>) -> Result<(), Failure> {
        // **The oracle's first walk, and everything it can answer is answered
        // here in source order** -- the duplicate names (`::CLASS`,
        // `::ROUTINE`, `::RESOURCE`, and a member directive's own dictionary
        // keys), a parenthesised `::CONSTANT` with no `::CLASS` before it, a
        // `CLASS` keyword with no `::CLASS` before it, an `::ANNOTATE` target
        // and an `EXTERNAL` library. Measured, `::constant sep (1+2)` alone in
        // a file is rc 157 with `Error 99.906`, where the identical directive
        // under a preceding `::CLASS` reaches the install-time evaluation
        // below instead; and see `staged_gap` for the stage order this walk is
        // the first of, and for every probe placing a form in it.
        let mut saw_class = false;
        // The class a member directive's keys are claimed against, and the
        // keys claimed so far. `None` is `LanguageParser`'s `unattachedMethods`
        // table, which is one table for the whole file rather than one per
        // class (`parser/DirectiveParser.cpp:518`).
        let mut current_class: Option<usize> = None;
        // Which directive claimed each key, and not merely that one did:
        // `::ANNOTATE ATTRIBUTE` and `::ANNOTATE CONSTANT` accept only a key
        // whose claimant is of the matching kind, which is `isAttribute()`
        // and `isConstant()` on the method object the C++ finds.
        let mut claimed: HashMap<(Option<usize>, bool, Vec<u8>), usize> = HashMap::new();
        // The other two tables the duplicate checks keep, each keyed by the
        // upcased name and separate from the others, so that a `::CLASS` and a
        // `::ROUTINE` of one name are not a collision. See
        // `Raised::duplicate_class` for the probes on both halves.
        // The class and routine tables carry the declaring directive's index
        // as well, because an `::ANNOTATE CLASS` or `::ANNOTATE ROUTINE`
        // resolves its target against exactly these two -- `classDependencies`
        // and `routines`, the same tables `findClassDirective` and
        // `findRoutine` read (`parser/DirectiveParser.cpp:230`, `:258`).
        let mut declared_classes: HashMap<Vec<u8>, usize> = HashMap::new();
        let mut declared_routines: HashMap<Vec<u8>, usize> = HashMap::new();
        let mut declared_resources: std::collections::HashSet<Vec<u8>> =
            std::collections::HashSet::new();
        // What each `::ANNOTATE` recorded, keyed by what its target names.
        // Filled in this walk and converted below, because a target's own
        // object does not exist yet: a `::CLASS` has no class object until
        // the install pass creates one.
        let mut staged: BTreeMap<AnnotatedSite, Vec<(Box<[u8]>, Box<[u8]>)>> = BTreeMap::new();
        for (index, directive) in program.directives.iter().enumerate() {
            // **A synthetic directive installs nothing**, which is what lets
            // `Interp::new_file_executable` file a loaded file's main section
            // as a directive of its own without also declaring it under a
            // name a program could call. The whole set of them is the ones
            // this crate builds, and each carries an empty clause span where
            // a written directive's spans at least `::method x` --
            // `no_written_directive_has_an_empty_clause_span` asserts that
            // over every corpus program, so this is a narrow rule rather than
            // a trap that silently drops a real directive.
            if directive.clause_span.is_empty() {
                continue;
            }
            // **Before the arms below, because the oracle checks before it
            // adds.** `constantDirective` calls `checkDuplicateMethod` ahead
            // of `createConstantGetterMethod`, which is what raises 99.906
            // (`parser/DirectiveParser.cpp:1926`, `:1933`), and the two part:
            // measured, `::constant c 5` then `::constant c (1+2)` with no
            // `::CLASS` in the file is 99.932 and not 99.906.
            self.check_member_keys(program, directive, current_class, &mut claimed, index)?;
            match &directive.kind {
                DirectiveKind::Class(class) => {
                    if declared_classes
                        .insert(class.name.to_ascii_uppercase(), index)
                        .is_some()
                    {
                        self.blame_directive(program, directive);
                        return Err(Raised::duplicate_class().into());
                    }
                    saw_class = true;
                    current_class = Some(index);
                }
                DirectiveKind::Resource(resource) => {
                    if !declared_resources.insert(resource.name.to_ascii_uppercase()) {
                        self.blame_directive(program, directive);
                        return Err(Raised::duplicate_resource().into());
                    }
                }
                DirectiveKind::Constant(constant) => {
                    if matches!(constant.value, ConstantValue::Expression(_)) && !saw_class {
                        self.blame_directive(program, directive);
                        return Err(Raised::constant_needs_class().into());
                    }
                }
                DirectiveKind::Routine(routine) => {
                    // Resolves nothing outside this file and runs nothing:
                    // the body is already assembled in the AST, so installing
                    // it is recording a name.
                    let name: Box<[u8]> = routine.name.to_ascii_uppercase().into();
                    declared_routines.insert(name.to_vec(), index);
                    let installed = InstalledRoutine {
                        program: id,
                        directive: index,
                    };
                    if routine.access == Access::Public {
                        self.package_public_routines
                            .entry(id)
                            .or_default()
                            .insert(name.clone(), installed);
                    }
                    if self
                        .routines
                        .entry(id)
                        .or_default()
                        .insert(name, installed)
                        .is_some()
                    {
                        // A *translation* error on the oracle, not an install
                        // one: measured, two `::routine zork` directives give
                        // `Error 99.903: Duplicate ::ROUTINE directive
                        // instruction.` at rc 157, echoing the second
                        // directive's own clause. `rexx-parse` does not
                        // detect it, so it is detected here, where the
                        // accumulated table is what answers.
                        self.blame_directive(program, directive);
                        return Err(Raised::duplicate_routine().into());
                    }
                }
                DirectiveKind::Annotate(annotate) => {
                    let target = match annotation_target(
                        program,
                        &annotate.target,
                        current_class,
                        &claimed,
                        &declared_classes,
                        &declared_routines,
                    ) {
                        Ok(target) => target,
                        Err(missing) => {
                            self.blame_directive(program, directive);
                            return Err(Raised::missing_annotation_target(
                                missing.kind,
                                missing.name,
                            )
                            .into());
                        }
                    };
                    // **Accumulative, and the last write to a name wins.**
                    // Each arm of `annotateDirective` reaches for its
                    // target's own table and `processAnnotation` puts into
                    // it (`parser/DirectiveParser.cpp:2259`), so a second
                    // `::ANNOTATE` of one target adds to the first's pairs.
                    // Measured, oracle rc 0: `::annotate class K a 1` beside
                    // `::annotate class K b 2` leaves `~annotations~items` 2,
                    // and `::annotate class K a 1 a 2` leaves it 1 with `A`
                    // answering `2`.
                    for site in target {
                        let pairs = staged.entry(site).or_default();
                        for annotation in &annotate.annotations {
                            let name = program.symbols.name(annotation.name).as_bytes();
                            pairs.retain(|(held, _)| **held != *name);
                            pairs.push((name.into(), annotation.value.clone()));
                        }
                    }
                }
                // **Applied in this walk, so its own refusal is in source
                // order with the rest.** Measured: the 33.1 below wins over a
                // duplicate `::ROUTINE` pair standing after it and loses to
                // one standing before it, and it wins over a `::CLASS` that
                // cannot resolve on either side of it -- the class pass is
                // the second walk. `::OPTIONS` itself never resolves a name
                // and never runs code, so this is the whole of installing it.
                DirectiveKind::Options(options) => {
                    let outcome = {
                        let package = self.package_options.entry(id).or_default();
                        options.iter().try_for_each(|option| package.apply(option))
                    };
                    if let Err(error) = outcome {
                        self.blame_directive(program, directive);
                        return Err(run::raised_from_settings(error).into());
                    }
                    // The required-string protocol's third arming route:
                    // `::OPTIONS NOSTRING SYNTAX` turns a rendering into a
                    // raise exactly as a `NOSTRING` trap does. See
                    // [`Interp::reqstr_armed`] for why this only ever sets.
                    if self
                        .options_of(id)
                        .is_some_and(PackageOptions::escalates_nostring)
                    {
                        self.reqstr_armed = true;
                    }
                    // `Interp::lostdigits_armed`'s only write, and the same
                    // set-once rule: the arithmetic path's gate.
                    if self
                        .options_of(id)
                        .is_some_and(PackageOptions::escalates_lostdigits)
                    {
                        self.lostdigits_armed = true;
                    }
                }
                _ => {}
            }

            // **After the arms above, not before them**, because when one
            // directive is both a duplicate `::ROUTINE` and an `EXTERNAL`
            // the oracle answers the duplicate: measured, `::routine dup`
            // followed by `::routine dup external "LIBRARY nosuchlib
            // nosuchfn"` is 99.903 rc 157 echoing the second directive, not
            // 98.903. Reverse that pair and the `EXTERNAL` comes first in the
            // file and wins, which the walk gives.
            if matches!(
                directive.kind,
                DirectiveKind::Annotate(_)
                    | DirectiveKind::Routine(_)
                    | DirectiveKind::Method(_)
                    | DirectiveKind::Attribute(_)
            ) && let Some(loud) = directive_gap(&directive.kind)
            {
                return Err(loud.into());
            }

            // **The eager bind** (D37), in this walk because the oracle does
            // it while the directive is being translated:
            // `createNativeMethod` raises from inside `methodDirective`
            // (`parser/DirectiveParser.cpp:1385`), so the file is refused
            // before its own first clause runs. Measured, oracle: a file
            // opening `say "prolog ran"` and carrying one `::METHOD
            // EXTERNAL` on a missing entry point is rc 166 with stdout empty,
            // and the same file naming `file_separator` is rc 0 printing the
            // prologue.
            if let Some(missing) = unresolved_external(&directive.kind) {
                self.blame_directive(program, directive);
                return Err(Raised::external_method_not_found(&missing).into());
            }
        }

        // **A second pass, because the oracle's own translation-time
        // refusals above happen before every install-time one below**
        // (98.9xx/43.901/the `::CONSTANT` expression evaluation), so a
        // program with both gets the translation error -- which is what
        // running the whole first pass before any of this reproduces.
        let mut declared: HashMap<Box<[u8]>, usize> = HashMap::new();
        for (index, directive) in program.directives.iter().enumerate() {
            if let DirectiveKind::Class(class) = &directive.kind {
                declared
                    .entry(class.name.to_ascii_uppercase().into())
                    .or_insert(index);
            }
        }

        // **The order the file's classes are installed in**, which is not
        // source order once a `SUBCLASS` names a class declared later. See
        // `class_install_order`; a cycle is 98.911 and never reaches the
        // installs below.
        let order = match class_install_order(program, &declared) {
            Ok(order) => order,
            Err(blame) => {
                self.blame_directive(program, &program.directives[blame]);
                let path = self.program_path.clone();
                return Err(Raised::cyclic_inheritance(&path).into());
            }
        };

        // A `::REQUIRES` file is opened after the cycle check and before any
        // class is created -- measured, 43.901 against a file whose first
        // directive is a `::CLASS` that fails to resolve, and 98.911 against
        // one whose classes form a cycle. See `staged_gap`.
        if let Some(loud) = staged_gap(program, |kind| matches!(kind, DirectiveKind::Requires(_))) {
            return Err(loud.into());
        }
        self.load_required_packages(id, program)?;

        // **The failing-`::CONSTANT` blame target is the class the oracle
        // installed LAST, not the last one in the file and not the nearest
        // preceding one, and every part of that is measured.** `::class A` /
        // `::constant x (1/0)` / `::class B` blames `B`, and a third
        // `::class C` after it blames `C`, so the blame does not depend on
        // which class the constant is lexically under. `::class b subclass a`
        // / `::constant c (1/0)` / `::class a` blames **`b`**, which source
        // order reaches first, and `::class c subclass b` / `::class a` /
        // `::constant x (1/0)` / `::class b subclass a` blames `c`, first in
        // the file. Which positional rules the corpus excludes, and which
        // witness excludes which, is in `class_install_order`'s own doc.
        // Tracked separately from `class_members` below, which is R9's
        // registry attachment and a genuinely different rule: a `::METHOD` or
        // `::ATTRIBUTE` attaches to the class positionally nearest above it.
        let last_class_directive = order.last().map(|index| &program.directives[*index]);

        // **Which class each `::METHOD`, `::ATTRIBUTE` and `::CONSTANT`
        // attaches to**, taken positionally (R9), so that the pass below can
        // install a class's own members while it is constructing that class.
        let members = class_members(program);

        // **Install walks the class list once per pass, and each pass
        // finishes before the next begins** (`PackageClass::processInstall`):
        // create every class (`classes/PackageClass.cpp:1281`), then resolve
        // every `::CONSTANT` expression (`:1290`), then send `ACTIVATE` to
        // every class (`:1299`). Each pass walks `order` rather than the
        // file, because `processInstall`'s own list is the class list.
        let mut classes: HashMap<usize, ObjRef> = HashMap::new();
        for index in &order {
            let attached = members.get(index).map_or(&[][..], Vec::as_slice);
            let class =
                self.install_class_at(id, program, *index, &declared, &classes, attached)?;
            classes.insert(*index, class);
            // **After the install and not inside it**, which is where
            // `ClassDirective::install` puts `setAnnotations`
            // (`instructions/ClassDirective.cpp:243`): the class is built and
            // has been sent `INIT` by then. Measured, oracle rc 0: a
            // class-side `init` saying `self~annotation("A")` prints `The NIL
            // object` under an `::ANNOTATE CLASS` that the main body reads
            // back as the annotation's value.
            self.attach_directive_annotations(program, &mut staged, *index, class, attached);
        }

        // What is left names no class object: the package, the file's
        // `::ROUTINE`s, and the method-shaped directives ahead of its first
        // `::CLASS`. `annotation_target` produces a `Directive` key for a
        // `::CLASS` and a `::ROUTINE` alone, and the loop above removed every
        // `::CLASS`'s, because `order` holds every `::CLASS` in the file.
        for (target, pairs) in staged {
            let site = match target {
                AnnotatedSite::Package => environment::Annotated::Package(Package::Program(id)),
                AnnotatedSite::Directive(directive) => {
                    environment::Annotated::Routine(id, directive)
                }
                AnnotatedSite::Member(_, name) => environment::Annotated::Unattached(id, name),
            };
            self.record_annotations(&[site], &pairs);
        }

        // The gap forms whose stage is after the classes are created; see
        // `staged_gap` for the probe behind each. Measured, `::options digits
        // 12` beside a failing `::CLASS` is the oracle's `::CLASS` line, so
        // this walk cannot move ahead of the pass above.
        for directive in &program.directives {
            if let Some(loud) = directive_gap(&directive.kind) {
                return Err(loud.into());
            }
        }

        for index in &order {
            let attached = members.get(index).map_or(&[][..], Vec::as_slice);
            self.resolve_constants(id, program, classes[index], attached, last_class_directive)?;
        }

        for index in &order {
            // `Some` inside this loop by construction: `last_class_directive`
            // is `order.last()` and the loop body runs only for a non-empty
            // `order`.
            let blame = last_class_directive.expect("a non-empty install order has a last class");
            self.send_directive_message(id, program, classes[index], dispatch::ACTIVATE, blame)?;
        }
        Ok(())
    }

    /// The file a package was loaded from: the program's own path, or the
    /// resolved name a `::REQUIRES` found it under.
    fn package_path(&self, id: ProgramId) -> &str {
        match self.required_paths.get(&id) {
            Some(path) => path,
            None => &self.program_path,
        }
    }

    /// Loads every package this program's `::REQUIRES` directives name, in
    /// source order, and merges each one's public routines and classes in.
    fn load_required_packages(
        &mut self,
        id: ProgramId,
        program: &Rc<Program>,
    ) -> Result<(), Failure> {
        if !program
            .directives
            .iter()
            .any(|directive| matches!(directive.kind, DirectiveKind::Requires(_)))
        {
            return Ok(());
        }
        self.requires_installing.push(self.package_path(id).into());
        let mut outcome = Ok(());
        for directive in &program.directives {
            let DirectiveKind::Requires(requires) = &directive.kind else {
                continue;
            };
            match self.load_requires(id, &requires.name) {
                Ok(required) => {
                    self.add_imported_package(id, Package::Program(required));
                    self.merge_required(id, required);
                    // `RequiresDirective::install`
                    // (`instructions/RequiresDirective.cpp:137`): the
                    // registration is what the directive does *after* the
                    // load and the merge, so a namespace neither narrows the
                    // merge nor replaces it.
                    if let Some(namespace) = requires.namespace {
                        let name = program.symbols.name(namespace).as_bytes().into();
                        self.package_namespaces
                            .entry(id)
                            .or_default()
                            .insert(name, Package::Program(required));
                    }
                }
                Err(failure) => {
                    self.seal_site_level();
                    self.blame_directive_in(id, program, directive);
                    outcome = Err(failure);
                    break;
                }
            }
        }
        self.requires_installing.pop();
        outcome
    }

    /// The package `name` names, loaded and its prologue run if this is the
    /// first `::REQUIRES` to reach it.
    fn load_requires(&mut self, id: ProgramId, name: &[u8]) -> Result<ProgramId, Failure> {
        if let Some(&loaded) = self.required_packages.get(name) {
            self.check_not_installing(loaded)?;
            return Ok(loaded);
        }
        let resolved = self.resolve_requires(id, name);
        if let Some(resolved) = &resolved
            && let Some(&loaded) = self.required_packages.get(resolved.as_bytes())
        {
            self.check_not_installing(loaded)?;
            self.required_packages.insert(name.into(), loaded);
            return Ok(loaded);
        }
        let Some(resolved) = resolved else {
            return Err(Raised::requires_file_not_found(name).into());
        };
        // The search already answered that this names a regular file, so a
        // read failing here is a permission or a race rather than a miss --
        // and the oracle reports the same 43.901 for it, since
        // `PackageManager::loadRequires` answers `OREF_NULL` either way.
        let Ok(text) = std::fs::read(&resolved) else {
            return Err(Raised::requires_file_not_found(name).into());
        };
        let parsed = match parse_program(text) {
            Ok(parsed) => Rc::new(parsed),
            Err(error) => {
                return Err(Loud::required_source(&resolved, &format!("{error}")).into());
            }
        };
        let required = ProgramId(self.programs.len());
        self.programs.push(Rc::clone(&parsed));
        self.required_paths
            .insert(required, resolved.as_str().into());
        // **Cached before the prologue runs, not after**, which is
        // `addRequiresFile` standing ahead of `runProlog` at
        // `InterpreterInstance.cpp:1060`: a name reached again from inside
        // that prologue must find this entry, or the circularity check has
        // nothing to fire on.
        self.required_packages.insert(name.into(), required);
        self.required_packages
            .insert(resolved.as_bytes().into(), required);
        self.run_loaded(parsed, required, CallType::Requires)?;
        Ok(required)
    }

    /// 98.952 when `loaded`'s own `::REQUIRES` directives are still
    /// installing, and `Ok` otherwise -- `Activity::checkRequires`
    /// (`concurrency/Activity.cpp:3702`).
    fn check_not_installing(&self, loaded: ProgramId) -> Result<(), Failure> {
        let path = self.package_path(loaded);
        if self.requires_installing.iter().any(|open| &**open == path) {
            return Err(Raised::circular_requires(path).into());
        }
        Ok(())
    }

    /// `PackageClass::addPackage` (`classes/PackageClass.cpp:1367`): records
    /// `from` as one of `into`'s imports, once.
    fn add_imported_package(&mut self, into: ProgramId, from: Package) -> bool {
        let held = self.package_imports.entry(into).or_default();
        if held.contains(&from) {
            return false;
        }
        held.push(from);
        true
    }

    /// [`Interp::merge_required`] for either kind of package.
    fn merge_package(&mut self, into: ProgramId, from: Package) {
        let from = match from {
            Package::Program(program) => return self.merge_required(into, program),
            Package::Rexx => self.rexx_package_class_table(true),
        };
        self.invalidate_rexx_class_cache();
        let target = self.merged_public_classes.entry(into).or_default();
        for (name, class) in from {
            target.entry(name).or_insert(class);
        }
    }

    /// The public routines and classes `from` contributes to `into`: its own
    /// first, then the ones it imported.
    fn merge_required(&mut self, into: ProgramId, from: ProgramId) {
        let routines: Vec<(Box<[u8]>, InstalledRoutine)> = self
            .package_public_routines
            .get(&from)
            .into_iter()
            .chain(self.merged_public_routines.get(&from))
            .flatten()
            .map(|(name, installed)| (name.clone(), *installed))
            .collect();
        let target = self.merged_public_routines.entry(into).or_default();
        for (name, installed) in routines {
            target.entry(name).or_insert(installed);
        }
        let classes: Vec<(Box<[u8]>, ObjRef)> = self
            .package_public_classes
            .get(&from)
            .into_iter()
            .chain(self.merged_public_classes.get(&from))
            .flatten()
            .map(|(name, class)| (name.clone(), *class))
            .collect();
        self.invalidate_rexx_class_cache();
        let target = self.merged_public_classes.entry(into).or_default();
        for (name, class) in classes {
            target.entry(name).or_insert(class);
        }
    }

    /// The package a namespace qualifier written in `package` names, or `None`
    /// when nothing registered it.
    fn find_namespace(&self, package: ProgramId, upper: &[u8]) -> Option<Namespace> {
        if upper == LIBRARY_PACKAGE_NAME {
            return Some(Namespace::Rexx);
        }
        self.package_namespaces
            .get(&package)?
            .get(upper)
            .copied()
            .map(|package| match package {
                Package::Rexx => Namespace::Rexx,
                Package::Program(program) => Namespace::Package(program),
            })
    }

    /// The class `namespace:name` names from `package`, or the oracle's own
    /// refusal for either half missing.
    fn namespace_class(
        &mut self,
        package: ProgramId,
        namespace: &[u8],
        name: &[u8],
    ) -> Result<ObjRef, Failure> {
        let Some(target) = self.find_namespace(package, namespace) else {
            let path = self.package_path(package).to_owned();
            return Err(Raised::namespace_not_found(namespace, &path).into());
        };
        let found = match target {
            Namespace::Rexx => self.rexx_package_class(name),
            Namespace::Package(program) => self.public_class_of(program, name),
        };
        found.ok_or_else(|| Raised::namespace_class_not_found(name, namespace).into())
    }

    /// `findPublicClass` for one package: its own `::CLASS ... PUBLIC`
    /// declarations, then the ones it imported (`classes/PackageClass.cpp:760`
    /// region). Measured, oracle rc 0: a class a *required* file of the
    /// namespace package declares public is reachable through the qualifier.
    fn public_class_of(&self, program: ProgramId, upper: &[u8]) -> Option<ObjRef> {
        if let Some(found) = self
            .package_public_classes
            .get(&program)
            .and_then(|table| table.get(upper))
        {
            return Some(*found);
        }
        self.merged_public_classes
            .get(&program)
            .and_then(|table| table.get(upper))
            .copied()
    }

    /// The routine `namespace:name` names from `package`, or the oracle's own
    /// refusal for either half missing.
    fn namespace_routine(
        &self,
        package: ProgramId,
        namespace: &[u8],
        name: &[u8],
    ) -> Result<InstalledRoutine, Failure> {
        let Some(target) = self.find_namespace(package, namespace) else {
            let path = self.package_path(package).to_owned();
            return Err(Raised::namespace_not_found(namespace, &path).into());
        };
        let found = match target {
            Namespace::Rexx => None,
            Namespace::Package(program) => self
                .package_public_routines
                .get(&program)
                .and_then(|table| table.get(name))
                .or_else(|| {
                    self.merged_public_routines
                        .get(&program)
                        .and_then(|table| table.get(name))
                })
                .copied(),
        };
        found.ok_or_else(|| Raised::namespace_routine_not_found(name, namespace).into())
    }

    /// The file a `::REQUIRES` of `name` in package `id` resolves to, or
    /// `None` when no route holds one.
    fn resolve_requires(&self, id: ProgramId, name: &[u8]) -> Option<String> {
        self.resolve_search(Some(self.package_path(id)), name, true)
    }

    /// [`Interp::resolve_requires`] for a caller that names the searching
    /// package's path itself and chooses the resolve type.
    pub(crate) fn resolve_search(
        &self,
        program: Option<&str>,
        name: &[u8],
        requires: bool,
    ) -> Option<String> {
        let name = std::str::from_utf8(name).ok()?;
        let entries = require::search_entries(
            program.and_then(require::program_directory),
            std::env::var("REXX_PATH").ok().as_deref(),
            std::env::var("PATH").ok().as_deref(),
        );
        let cwd = std::env::current_dir().ok()?;
        let cwd = cwd.to_str()?;
        let extension = program.and_then(require::program_extension);
        for candidate in require::candidates(name, &entries, extension, requires) {
            let resolved = require::normalize(&candidate, cwd);
            if std::fs::metadata(&resolved).is_ok_and(|meta| meta.is_file()) {
                return Some(resolved);
            }
        }
        None
    }

    /// `PackageClass::loadPackageRexx`'s load: the same
    /// [`Interp::load_requires`] a `::REQUIRES` performs, which is what the
    /// C++ calls too (`classes/PackageClass.cpp:1842`).
    pub(crate) fn load_package(
        &mut self,
        program: ProgramId,
        name: &[u8],
    ) -> Result<ProgramId, Failure> {
        self.load_requires(program, name)
    }

    /// `PackageClass::newRexx`'s in-memory form: a package compiled from
    /// source lines, its directives installed and its prologue run.
    pub(crate) fn package_from_source(
        &mut self,
        name: &[u8],
        lines: &[Vec<u8>],
    ) -> Result<ProgramId, Failure> {
        let borrowed: Vec<&[u8]> = lines.iter().map(Vec::as_slice).collect();
        let parsed = rexx_parse::parse_lines(&borrowed).map_err(|error| {
            Failure::from(Loud::required_source(
                &String::from_utf8_lossy(name),
                &format!("{error}"),
            ))
        })?;
        let parsed = Rc::new(parsed);
        let id = ProgramId(self.programs.len());
        self.programs.push(Rc::clone(&parsed));
        self.compiled_method_names.insert(id, name.into());
        self.run_loaded(parsed, id, CallType::Requires)?;
        Ok(id)
    }

    /// Moves what the first walk recorded for one `::CLASS` and for the
    /// members attached to it onto the class object that has just been built.
    fn attach_directive_annotations(
        &mut self,
        program: &Rc<Program>,
        staged: &mut BTreeMap<AnnotatedSite, Vec<(Box<[u8]>, Box<[u8]>)>>,
        index: usize,
        class: ObjRef,
        attached: &[usize],
    ) {
        if let Some(pairs) = staged.remove(&AnnotatedSite::Directive(index)) {
            self.record_annotations(&[environment::Annotated::Class(class)], &pairs);
        }
        for &member in attached {
            // The sides one name is filed under, which is more than one for a
            // `::CONSTANT` and is why the keys are collected before the table
            // is built: both sides must answer one table.
            // Ordered for [`AnnotatedSite`]'s reason: the two halves of an
            // accessor pair are two names of one directive, and which of
            // them gets its table first must not depend on a hash seed.
            let mut sides: BTreeMap<Vec<u8>, Vec<bool>> = BTreeMap::new();
            for (name, class_side) in member_dictionary_keys(&program.directives[member].kind) {
                sides.entry(name).or_default().push(class_side);
            }
            for (name, sides) in sides {
                let key = AnnotatedSite::Member(member, name.clone().into());
                let Some(pairs) = staged.remove(&key) else {
                    continue;
                };
                let keys: Vec<environment::Annotated> = sides
                    .into_iter()
                    .map(|class_side| {
                        environment::Annotated::Member(class, class_side, name.clone().into())
                    })
                    .collect();
                self.record_annotations(&keys, &pairs);
            }
        }
    }

    /// `LanguageParser::checkDuplicateMethod`
    /// (`parser/DirectiveParser.cpp:507`-`:530`): every dictionary key a
    /// member directive is about to claim, refused if the class it attaches
    /// to has already been given that key on that side.
    fn check_member_keys(
        &mut self,
        program: &Rc<Program>,
        directive: &Directive,
        current_class: Option<usize>,
        claimed: &mut HashMap<(Option<usize>, bool, Vec<u8>), usize>,
        index: usize,
    ) -> Result<(), Failure> {
        let constant = matches!(directive.kind, DirectiveKind::Constant(_));
        for (name, class_side) in member_dictionary_keys(&directive.kind) {
            if current_class.is_none() {
                if constant && class_side {
                    continue;
                }
                if class_side {
                    self.blame_directive(program, directive);
                    return Err(Raised::class_keyword_needs_class().into());
                }
            }
            if claimed
                .insert((current_class, class_side, name), index)
                .is_some()
            {
                self.blame_directive(program, directive);
                return Err(Raised::duplicate_member(&directive.kind).into());
            }
        }
        Ok(())
    }

    /// Records the value of every literal `::CONSTANT` among `attached`.
    fn record_literal_constants(
        &mut self,
        id: ProgramId,
        program: &Rc<Program>,
        attached: &[usize],
    ) {
        for &index in attached {
            let DirectiveKind::Constant(constant) = &program.directives[index].kind else {
                continue;
            };
            let value = match &constant.value {
                ConstantValue::Name => self.interned_literal(&constant.name),
                ConstantValue::Text(text) => self.interned_literal(text),
                ConstantValue::Expression(_) => continue,
            };
            self.record_constant_value(id, index, value);
        }
    }

    /// Evaluates the `::CONSTANT` expressions among `attached`, in source
    /// order, and records what each answered.
    fn resolve_constants(
        &mut self,
        id: ProgramId,
        program: &Rc<Program>,
        class: ObjRef,
        attached: &[usize],
        last_class: Option<&Directive>,
    ) -> Result<(), Failure> {
        for &index in attached {
            let directive = &program.directives[index];
            let DirectiveKind::Constant(constant) = &directive.kind else {
                continue;
            };
            let ConstantValue::Expression(expr) = &constant.value else {
                continue;
            };
            match self.eval_constant_expression(id, program, class, expr) {
                Ok(value) => self.record_constant_value(id, index, value),
                Err(failure) => {
                    // Two clause echoes, innermost first, matching the
                    // oracle's own report exactly (measured, `::class K` /
                    // `::constant c (1/0)`):
                    // ```text
                    //      4 *-* ::constant c (1/0)
                    //      3 *-* ::class K
                    // ```
                    // ```text
                    //      5 *-* return 1/0
                    //      6 *-* ::constant c (self~m)
                    //      3 *-* ::class K
                    // ```
                    self.blame_directive(program, directive);
                    self.seal_site_level();
                    self.blame_directive(
                        program,
                        last_class.expect(
                            "the first pass already refused an expression with no \
                             preceding ::CLASS",
                        ),
                    );
                    return Err(failure);
                }
            }
        }
        Ok(())
    }

    /// Records what a `::CONSTANT` accessor answers, and roots it.
    fn record_constant_value(&mut self, program: ProgramId, directive: usize, value: ObjRef) {
        self.constant_values.insert((program, directive), value);
        self.roots
            .add_global(&constant_root_key(program, directive), value);
    }

    /// The value a `::CONSTANT` accessor answers, or `None` for an expression
    /// form whose pass has not run yet -- [`Interp::constant_values`] has
    /// which of those is reachable.
    pub(crate) fn constant_value(&self, generated: GeneratedMethod) -> Option<ObjRef> {
        self.constant_values
            .get(&(generated.program, generated.directive))
            .copied()
    }

    /// The constant's name as its accessor was installed under, which is what
    /// a 97.4 report names.
    pub(crate) fn constant_name(&self, generated: GeneratedMethod) -> Result<Vec<u8>, Failure> {
        let program = &self.programs[generated.program.0];
        // `get` rather than an index, and a refusal rather than a panic, for
        // the reason `Interp::enter_method_body`'s own reads carry.
        let Some(directive) = program.directives.get(generated.directive) else {
            return Err(Loud::missing_body().into());
        };
        let DirectiveKind::Constant(constant) = &directive.kind else {
            return Err(Loud::missing_body().into());
        };
        Ok(constant.name.to_ascii_uppercase())
    }

    /// One message the install machinery sends a class object, run in a
    /// throwaway activation carrying the installing program.
    fn send_directive_message(
        &mut self,
        id: ProgramId,
        program: &Rc<Program>,
        class: ObjRef,
        name: &[u8],
        blame: &Directive,
    ) -> Result<(), Failure> {
        let frame = self.push_directive_activation(id, program);
        let caller = self.caller();
        let sent = self.send_message(class, name, None, &[], caller);
        self.pop_directive_activation(frame);
        let Err(failure) = sent else {
            return Ok(());
        };
        self.seal_site_level();
        self.blame_directive(program, blame);
        Err(failure)
    }

    /// Pushes the activation an install-time evaluation or send runs in, and
    /// answers the frame [`Interp::pop_directive_activation`] takes back.
    fn push_directive_activation(&mut self, id: ProgramId, program: &Rc<Program>) -> SlotFrame {
        let frame = self.roots.push_slots(0);
        let activation_id = self.next_activation_id();
        self.push_activation(Activation::new(
            activation_id,
            Rc::clone(program),
            id,
            Rc::new(Plan::default()),
            frame,
        ));
        frame
    }

    /// Tears down what [`Interp::push_directive_activation`] pushed.
    fn pop_directive_activation(&mut self, frame: SlotFrame) {
        self.pop_activation();
        self.roots.pop_slots(frame);
    }

    /// `::CLASS`'s own R9 install: a class object in [`Interp::classes`],
    /// carrying the name as written, and an entry in the running package's own
    /// class table under the uppercased one.
    fn install_class(
        &mut self,
        program: ProgramId,
        class: &ClassDirective,
        superclass: ObjRef,
        metaclass: ObjRef,
    ) -> ObjRef {
        let name = String::from_utf8_lossy(&class.name).into_owned();
        // `define_unregistered_class`, not `define_class`: an installed class
        // does not go into `.environment`, and registering it there would let
        // `::class array` displace the environment's own `Array` for every
        // later lookup rather than only for this package's.
        let kind = if class.mixin {
            ClassKind::Mixin
        } else {
            ClassKind::Regular
        };
        let id = self.mint_class();
        self.classes()
            .define_unregistered_class(id, &name, Some(superclass), kind, metaclass);
        // **A class the interpreter's own library declares is a class in the
        // image**, and `RexxClass::liveGeneral` sets `REXX_DEFINED` on every
        // class in the image under `PREPARINGIMAGE` (`ClassClass.cpp:136`-
        // `:142`). Measured: the oracle refuses `.Alarm~inherit(.Comparable)`
        // with 98.985, the same refusal it gives `.Array~inherit()`.
        if self.library_bootstrap {
            self.classes().set_rexx_defined(id);
        }
        // The package's own installed-class table, which is what `.NAME`
        // resolution reads first -- see `environment.rs`'s
        // `record_package_class` for why the registry's flat table is not
        // that.
        // `ClassDirective::install` passes the directive's own `isPublic()`
        // to `addInstalledClass` (`instructions/ClassDirective.cpp:209`),
        // which files a public class in both of the package's tables and
        // every other class in one (`classes/PackageClass.cpp:1410`-`:1419`).
        // That is the difference `~publicClasses` reads back.
        self.record_package_class(program, &class.name, id, class.access == Access::Public);
        id
    }

    /// Installs the `::CLASS` at `index`, whose declared targets
    /// [`class_install_order`] has already put before it.
    fn install_class_at(
        &mut self,
        program_id: ProgramId,
        program: &Rc<Program>,
        index: usize,
        declared: &HashMap<Box<[u8]>, usize>,
        installed: &HashMap<usize, ObjRef>,
        attached: &[usize],
    ) -> Result<ObjRef, Failure> {
        let directive = &program.directives[index];
        if let Some(loud) = directive_gap(&directive.kind) {
            return Err(loud.into());
        }
        let DirectiveKind::Class(class) = &directive.kind else {
            return Err(Loud::missing_body().into());
        };
        let file_classes = FileClasses {
            declared,
            installed,
        };
        let named_metaclass = match &class.metaclass {
            None => None,
            Some(target) => Some(self.resolve_class_target(
                program_id,
                program,
                directive,
                target,
                &file_classes,
                Raised::metaclass_not_found,
            )?),
        };
        let superclass = match &class.subclass {
            None => self.root_and_metaclass().0,
            Some(target) => self.resolve_class_target(
                program_id,
                program,
                directive,
                target,
                &file_classes,
                Raised::class_not_found,
            )?,
        };
        // `RexxClass::subclass`'s own opening (`ClassClass.cpp:1566`-
        // `:1575`): a directive naming no `METACLASS` derives from the
        // superclass's own, and either way the value has to be a metaclass
        // before anything is built from it. Measured, `::CLASS S MIXINCLASS
        // Class METACLASS Object` raises this even though deriving from
        // `.Class` then discards the named metaclass -- the test is on what
        // the directive named, not on what the class ends up with.
        let metaclass = named_metaclass.unwrap_or_else(|| self.classes().metaclass(superclass));
        if !self.classes().is_metaclass(metaclass) {
            let name = self.class_default_name(metaclass).to_vec();
            self.blame_directive(program, directive);
            return Err(Raised::bad_metaclass(&name).into());
        }
        let id = self.install_class(program_id, class, superclass, metaclass);
        self.record_literal_constants(program_id, program, attached);
        // **The class-side members are the enhancing methods the class is
        // built with**: `ClassDirective::install` hands `classMethods` to
        // `mixinClass`/`subclass` (`ClassDirective.cpp:200`, `:205`), which
        // merges them into the class method dictionary before it builds
        // either behaviour and therefore before it sends `INIT`
        // (`ClassClass.cpp:1602`-`:1607`, then `:1613` builds the behaviour and
        // `:1631` sends the message).
        self.install_class_members(program_id, program, id, attached, true);
        // `RexxClass::subclass`'s own tail, in its order: `checkUninit`
        // (`ClassClass.cpp:1628`), the `INIT` send (`:1631`), then the
        // parent's `UNINIT` propagation (`:1634`-`:1637`). Reading a finished
        // parent is what `order` buys: a class is constructed after every
        // class it names.
        self.classes().check_uninit(id);
        self.send_directive_message(program_id, program, id, dispatch::INIT, directive)?;
        self.classes().refresh_parent_has_uninit(id);
        for target in &class.inherit {
            let mixin = self.resolve_class_target(
                program_id,
                program,
                directive,
                target,
                &file_classes,
                Raised::class_not_found,
            )?;
            self.inherit_mixin(program, directive, id, mixin)?;
        }
        self.install_class_members(program_id, program, id, attached, false);
        // `RexxClass::makeAbstract` (`ClassClass.cpp:1754`-`:1761`): a
        // metaclass cannot be made abstract, and any other class takes the
        // keyword by setting the flag `~new`'s `checkAbstract` reads.
        if class.abstract_ {
            if self.classes().is_metaclass(id) {
                let class_id = self.class_id_text(id).as_bytes().to_vec();
                self.blame_directive(program, directive);
                return Err(Raised::abstract_metaclass(&class_id).into());
            }
            self.classes().make_abstract(id);
        }
        Ok(id)
    }

    /// One side of a class's own members: the `::METHOD` and `::ATTRIBUTE`
    /// directives whose `CLASS` keyword matches `class_side`, and every
    /// `::CONSTANT`, which installs on both.
    fn install_class_members(
        &mut self,
        program_id: ProgramId,
        program: &Rc<Program>,
        class: ObjRef,
        attached: &[usize],
        class_side: bool,
    ) {
        for &index in attached {
            match &program.directives[index].kind {
                DirectiveKind::Method(method) if method.class_method == class_side => {
                    self.install_method(program_id, index, class, method);
                }
                DirectiveKind::Attribute(attribute) if attribute.class_method == class_side => {
                    self.install_attribute(program_id, index, class, attribute);
                }
                DirectiveKind::Constant(constant) => {
                    self.install_constant(program_id, index, class, constant, class_side);
                }
                _ => {}
            }
        }
    }

    /// `::CONSTANT`'s own R9 install: the upcased name lands in one of
    /// `class`'s dictionaries as a [`GeneratedKind::Constant`] accessor.
    fn install_constant(
        &mut self,
        program: ProgramId,
        directive: usize,
        class: ObjRef,
        constant: &ConstantDirective,
        class_method: bool,
    ) {
        let upper = constant.name.to_ascii_uppercase();
        self.install_one_method(
            program,
            directive,
            class,
            &upper,
            InstallBody::Generated(GeneratedKind::Constant),
            class_method,
            Access::Default,
            Protection::Default,
        );
    }

    /// One class reference on a `::CLASS`, resolved against the file's own
    /// `::CLASS` names and then the registry.
    fn resolve_class_target(
        &mut self,
        installing: ProgramId,
        program: &Rc<Program>,
        directive: &Directive,
        target: &ClassRef,
        classes: &FileClasses<'_>,
        not_found: fn(&[u8]) -> Raised,
    ) -> Result<ObjRef, Failure> {
        if let Some(namespace) = target.namespace {
            let namespace = program.symbols.name(namespace).as_bytes().to_vec();
            let found = self.namespace_class(installing, &namespace, &target.name);
            if found.is_err() {
                self.blame_directive(program, directive);
            }
            return found;
        }
        match classes.declared.get(target.name.as_ref()) {
            // Already installed, because `class_install_order` put it ahead
            // of this one; a `None` here would be that ordering and its
            // caller's loop disagreeing, which is an internal inconsistency
            // and gets this crate's loud refusal rather than a panic.
            Some(other) => match classes.installed.get(other) {
                Some(id) => Ok(*id),
                None => Err(Loud::missing_body().into()),
            },
            None => match self.directive_class(installing, &target.name) {
                Some(id) => Ok(id),
                None => {
                    self.blame_directive(program, directive);
                    Err(not_found(&target.name).into())
                }
            },
        }
    }

    /// One `INHERIT` entry, as the send the oracle makes for it.
    fn inherit_mixin(
        &mut self,
        program: &Rc<Program>,
        directive: &Directive,
        class: ObjRef,
        mixin: ObjRef,
    ) -> Result<(), Failure> {
        let Err(refusal) = self.classes().inherit(class, mixin) else {
            return Ok(());
        };
        // `Interp::class_default_name` borrows out of the registry, so the
        // substitutions are taken before the blaming below reborrows it.
        let mixin_name = self.class_default_name(mixin).to_vec();
        let raised = match refusal {
            InheritRefusal::NotAMixin => Raised::inherit_needs_a_mixinclass(&mixin_name),
            InheritRefusal::Recursive => {
                let class_name = self.class_default_name(class).to_vec();
                Raised::recursive_inherit(&class_name, &mixin_name)
            }
            InheritRefusal::BaseClass(base) => {
                let class_name = self.class_default_name(class).to_vec();
                let base_name = self.class_default_name(base).to_vec();
                Raised::inherit_base_class(&class_name, &mixin_name, &base_name)
            }
            // The `INHERIT` keyword carries no position, and this refusal is
            // the position's alone (`ClassClass.cpp:1350`), so a directive
            // cannot reach it. Answered rather than unreachable-panicked,
            // this crate's rule for a case the type admits and the caller
            // does not produce.
            InheritRefusal::NotInherited(other) => {
                let class_name = self.class_default_name(class).to_vec();
                let other_name = self.class_default_name(other).to_vec();
                Raised::not_inherited(&class_name, &other_name)
            }
        };
        let scope = self.root_and_metaclass().1;
        let scope = self.classes().id_string(scope).to_string();
        self.blame_native_method(b"INHERIT", &scope);
        self.blame_directive(program, directive);
        Err(raised.into())
    }

    /// `::METHOD`'s own R9 install: the name lands in `class`'s instance
    /// dictionary, or its class dictionary for `::METHOD ... CLASS`, and
    /// [`Interp::record_method_body`] records which directive to read its
    /// body from later (Task 7's, not entered here).
    fn install_method(
        &mut self,
        program: ProgramId,
        directive: usize,
        class: ObjRef,
        method: &MethodDirective,
    ) {
        // The bind `Interp::install_directives`' walk already resolved, asked
        // again rather than staged: `method_external` is a pure function of
        // the directive, and a staging map keyed by directive index would be
        // a second place for the answer to live. Every entry point it names
        // resolved, because that walk returned 90.998 for the first that did
        // not and this install never ran.
        let external = dispatch::native::method_external(method);
        for (name, generated) in method_dictionary_keys(method) {
            let native = dispatch::native::bound_entry(external.as_ref(), &name);
            debug_assert!(
                native.is_none() || generated.is_none(),
                "a ::METHOD bound to a LIBRARY REXX entry point also generated a method \
                 for {}, so one of them is lost",
                String::from_utf8_lossy(&name)
            );
            let body = match (generated, native) {
                (Some(kind), _) => InstallBody::Generated(kind),
                (None, Some(entry)) => InstallBody::Native(entry),
                (None, None) => InstallBody::Written,
            };
            self.install_one_method(
                program,
                directive,
                class,
                &name,
                body,
                method.class_method,
                method.access,
                method.protection,
            );
        }
    }

    /// `::ATTRIBUTE`'s own R9 install: one or two accessor names, per
    /// [`AttributeStyle`] -- the plain name for a getter, the name with `=`
    /// appended for a setter, both for the default (neither `GET` nor `SET`)
    /// style -- landing in `class`'s instance or class dictionary the same
    /// way [`Interp::install_method`] does.
    fn install_attribute(
        &mut self,
        program: ProgramId,
        directive: usize,
        class: ObjRef,
        attribute: &AttributeDirective,
    ) {
        // Read for the reason `Interp::install_method`'s is, and asked per
        // dictionary key because the two accessors of an `EXTERNAL` attribute
        // name two different procedures.
        let external = dispatch::native::attribute_external(attribute);
        for (name, generated) in attribute_dictionary_keys(attribute) {
            let body = match (
                generated,
                dispatch::native::bound_entry(external.as_ref(), &name),
            ) {
                (Some(kind), _) => InstallBody::Generated(kind),
                (None, Some(entry)) => InstallBody::Native(entry),
                (None, None) => InstallBody::Written,
            };
            // Both accessors of a `Both`-style attribute carry the
            // directive's own access scope, which is the oracle's own shape:
            // `attributeDirective` builds the getter and the setter and calls
            // `setAttributes(accessFlag, protectedFlag, guardFlag)` on each
            // (`parser/DirectiveParser.cpp:1683`, `:1690`).
            self.install_one_method(
                program,
                directive,
                class,
                &name,
                body,
                attribute.class_method,
                attribute.access,
                attribute.protection,
            );
        }
    }

    /// Adds one name to `class`'s instance or class dictionary and records
    /// what the name resolves to -- the tail every `::METHOD` and
    /// `::ATTRIBUTE` install shares, once per dictionary key rather than once
    /// per directive.
    #[allow(clippy::too_many_arguments)]
    fn install_one_method(
        &mut self,
        program: ProgramId,
        directive: usize,
        class: ObjRef,
        name: &[u8],
        body: InstallBody,
        class_method: bool,
        access: Access,
        protection: Protection,
    ) {
        self.arm_reqstr_for(name);
        let name = String::from_utf8_lossy(name).into_owned();
        let method_id = if class_method {
            self.classes().add_class_method(class, &name)
        } else {
            self.classes().add_instance_method(class, &name)
        };
        self.record_method_body(method_id, program, directive, body);
        self.record_access_scope(method_id, program, access, protection);
    }

    /// Arms `Interp::reqstr_armed` for a method name the required-string
    /// protocol would send, called from every directive install that adds a
    /// name to a class's dictionary.
    fn arm_reqstr_for(&mut self, installed: &[u8]) {
        if installed == dispatch::MAKESTRING {
            self.reqstr_armed = true;
        }
    }

    /// Records which `(program, directive)` a just-minted
    /// [`rexx_classes::MethodId`] names, immediately after the call that
    /// minted it -- see [`Interp::method_bodies`]'s own doc for what the key
    /// is.
    fn record_method_body(
        &mut self,
        method: MethodId,
        program: ProgramId,
        directive: usize,
        body: InstallBody,
    ) {
        let previous = match body {
            InstallBody::Written => self
                .method_bodies
                .insert(method, InstalledMethodBody { program, directive })
                .is_some(),
            InstallBody::Generated(kind) => self
                .generated_methods
                .insert(
                    method,
                    GeneratedMethod {
                        program,
                        directive,
                        kind,
                    },
                )
                .is_some(),
            InstallBody::Native(entry) => self.native_externals.insert(method, entry).is_some(),
        };
        debug_assert!(
            !previous,
            "a MethodId was recorded twice, so one of the two bodies is lost"
        );
    }

    /// Files a body compiled from method source text as a program of its own
    /// and hangs it on the `Method` object, so a send can enter it.
    fn record_compiled_body(&mut self, object: ObjRef, name: &[u8], parsed: Program) {
        let Program {
            source,
            main,
            symbols,
            ..
        } = parsed;
        let program = Rc::new(Program {
            source,
            main: CodeBody::default(),
            directives: vec![Directive {
                kind: DirectiveKind::Method(Box::new(MethodDirective {
                    name: name.into(),
                    class_method: false,
                    attribute: false,
                    abstract_: false,
                    access: Access::default(),
                    protection: Protection::default(),
                    guard: GuardOption::default(),
                    external: None,
                    delegate: None,
                    body: Some(main),
                })),
                clause_span: 0..0,
            }],
            symbols,
        });
        let program_id = ProgramId(self.programs.len());
        self.programs.push(program);
        self.compiled_method_names.insert(program_id, name.into());
        self.table_method_bodies.insert(
            object,
            InstalledMethodBody {
                program: program_id,
                directive: 0,
            },
        );
        self.executable_sources.insert(
            object,
            ExecutableRecord {
                source: ExecutableSource::Main {
                    program: program_id,
                },
                installed: None,
                routine: None,
            },
        );
    }

    /// [`Interp::record_compiled_body`] for a `Routine`: the same program of
    /// its own, carrying a `::ROUTINE` rather than a `::METHOD`.
    fn record_compiled_routine(&mut self, object: ObjRef, name: &[u8], parsed: Program) {
        let Program {
            source,
            main,
            symbols,
            ..
        } = parsed;
        let program = Rc::new(Program {
            source,
            main: CodeBody::default(),
            directives: vec![Directive {
                kind: DirectiveKind::Routine(Box::new(rexx_parse::RoutineDirective {
                    name: name.into(),
                    access: Access::default(),
                    external: None,
                    body: Some(main),
                })),
                clause_span: 0..0,
            }],
            symbols,
        });
        let program_id = ProgramId(self.programs.len());
        self.programs.push(program);
        self.compiled_method_names.insert(program_id, name.into());
        self.executable_sources.insert(
            object,
            ExecutableRecord {
                source: ExecutableSource::Main {
                    program: program_id,
                },
                installed: None,
                routine: Some((program_id, 0)),
            },
        );
    }

    /// `MethodClass::newFileRexx` and `RoutineClass::newFileRexx`
    /// (`classes/MethodClass.cpp:521`, `classes/RoutineClass.cpp:341`): the
    /// executable a file's own text becomes.
    pub(crate) fn new_file_executable(
        &mut self,
        name: &[u8],
        routine: bool,
    ) -> Result<ObjRef, Failure> {
        let path = String::from_utf8_lossy(name).into_owned();
        let Ok(text) = std::fs::read(&path) else {
            return Err(Raised::executable_file_unreadable(name).into());
        };
        let parsed = match rexx_parse::parse_program(text) {
            Ok(parsed) => parsed,
            Err(error) => {
                return Err(Loud::method_from_source(&format!(
                    "reporting a file that does not parse ({path}, {error})"
                ))
                .into());
            }
        };
        let Program {
            source,
            main,
            mut directives,
            symbols,
        } = parsed;
        let body = directives.len();
        directives.push(Directive {
            kind: if routine {
                DirectiveKind::Routine(Box::new(rexx_parse::RoutineDirective {
                    name: name.into(),
                    access: Access::default(),
                    external: None,
                    body: Some(main),
                }))
            } else {
                DirectiveKind::Method(Box::new(MethodDirective {
                    name: name.into(),
                    class_method: false,
                    attribute: false,
                    abstract_: false,
                    access: Access::default(),
                    protection: Protection::default(),
                    guard: GuardOption::default(),
                    external: None,
                    delegate: None,
                    body: Some(main),
                }))
            },
            clause_span: 0..0,
        });
        let program = Rc::new(Program {
            source,
            main: CodeBody::default(),
            directives,
            symbols,
        });
        let id = ProgramId(self.programs.len());
        self.programs.push(Rc::clone(&program));
        self.required_paths.insert(id, path.into());
        self.install_directives(id, &program)?;
        let class = if routine {
            self.routine_class()
        } else {
            self.method_class()
        };
        let object = self.native_instance(class);
        let site = environment::Annotated::Compiled(self.compiled_methods);
        self.compiled_methods += 1;
        self.attach_annotations(object, site);
        self.executable_sources.insert(
            object,
            ExecutableRecord {
                source: ExecutableSource::Main { program: id },
                installed: None,
                routine: routine.then_some((id, body)),
            },
        );
        if !routine {
            self.table_method_bodies.insert(
                object,
                InstalledMethodBody {
                    program: id,
                    directive: body,
                },
            );
        }
        Ok(object)
    }

    /// Records a `Method` or `Routine` object whose body is a primitive, so
    /// that its readers answer `BaseCode`'s: an empty `~source`, the `REXX`
    /// package, and `0` from `~setSecurityManager`.
    pub(crate) fn record_native_executable(&mut self, object: ObjRef) {
        self.executable_sources.insert(
            object,
            ExecutableRecord {
                source: ExecutableSource::Native,
                installed: None,
                routine: None,
            },
        );
    }

    /// What the `Method` object for one installed method reports on: the
    /// directive that declared it, or [`ExecutableSource::Native`] for a
    /// primitive and for an `EXTERNAL` binding, neither of which has one.
    pub(crate) fn installed_executable_source(&self, method: MethodId) -> ExecutableSource {
        if let Some(installed) = self.method_bodies.get(&method) {
            return ExecutableSource::Directive {
                program: installed.program,
                directive: installed.directive,
            };
        }
        if let Some(generated) = self.generated_methods.get(&method) {
            return ExecutableSource::Directive {
                program: generated.program,
                directive: generated.directive,
            };
        }
        ExecutableSource::Native
    }

    /// `Method~setPrivate`'s half that a send can see: the dictionary entry
    /// this object *is* stops answering a sender outside its scope.
    pub(crate) fn make_method_private(&mut self, object: ObjRef) {
        let Some(method) = self
            .executable_sources
            .get(&object)
            .and_then(|record| record.installed)
        else {
            return;
        };
        let package = match self.installed_executable_source(method) {
            ExecutableSource::Directive { program, .. } | ExecutableSource::Main { program } => {
                plan::Package::Program(program)
            }
            ExecutableSource::Native => plan::Package::Rexx,
        };
        let row = self.special_method_row(method);
        match row {
            Some(existing) => existing.access = Access::Private,
            None => {
                *self.special_method_row_mut(method) = Some(dispatch::AccessScope {
                    access: Access::Private,
                    protected: false,
                    package,
                });
            }
        }
    }

    /// The access-scope row `method` already has, if it has one.
    pub(crate) fn special_method_row(
        &mut self,
        method: MethodId,
    ) -> Option<&mut dispatch::AccessScope> {
        self.special_methods.get_mut(method.0 as usize)?.as_mut()
    }

    /// The slot `method`'s row lives in, growing the table to reach it.
    pub(crate) fn special_method_row_mut(
        &mut self,
        method: MethodId,
    ) -> &mut Option<dispatch::AccessScope> {
        let index = method.0 as usize;
        if index >= self.special_methods.len() {
            self.special_methods.resize(index + 1, None);
        }
        &mut self.special_methods[index]
    }

    /// One `::ROUTINE` run over the arguments given, for `Routine~call` and
    /// the two rows beside it.
    pub(crate) fn enter_installed_routine(
        &mut self,
        program: ProgramId,
        directive: usize,
        arguments: Vec<Option<ObjRef>>,
    ) -> Result<Option<ObjRef>, Failure> {
        let installed = InstalledRoutine { program, directive };
        self.call_over_installed_routine(installed, arguments)
    }

    /// `PARSE SOURCE`'s third word: a compiled method's own name, the file a
    /// `::REQUIRES` loaded this package from, or the running program's path.
    pub(crate) fn program_display_name(&self, program: ProgramId) -> &[u8] {
        match self.compiled_method_names.get(&program) {
            Some(name) => name,
            None => self.package_path(program).as_bytes(),
        }
    }

    /// Records a just-minted method's access scope and protection, for the
    /// methods the oracle calls *special*.
    fn record_access_scope(
        &mut self,
        method: MethodId,
        program: ProgramId,
        access: Access,
        protection: Protection,
    ) {
        let protected = protection == Protection::Protected;
        let scoped = matches!(access, Access::Private | Access::Package);
        if !protected && !scoped {
            return;
        }
        *self.special_method_row_mut(method) = Some(dispatch::AccessScope {
            access,
            protected,
            package: Package::Program(program),
        });
    }

    /// Evaluates a `::CONSTANT` directive's parenthesised expression in the
    /// second install pass, and answers what it produced.
    fn eval_constant_expression(
        &mut self,
        id: ProgramId,
        program: &Rc<Program>,
        class: ObjRef,
        expr: &Expr,
    ) -> Result<ObjRef, Failure> {
        let empty_body = CodeBody::default();
        let frame = self.push_directive_activation(id, program);
        // The oracle's own message name for this run, which is the string
        // `GlobalNames::CONSTANT_DIRECTIVE` holds (`memory/GlobalNames.h:84`).
        // The failure path this crate has a witness for does not print it:
        // measured, a condition raised inside a class method the expression
        // calls echoes that method's clause above the `::CONSTANT` and the
        // `::CLASS`, and names no method.
        let saved_context = std::mem::replace(
            &mut self.call_context,
            CallContext {
                name: b"::CONSTANT".to_vec(),
                arguments: Rc::from(&[][..]),
                receiver: Some(class),
            },
        );
        let self_slot = self.slot_of(b"SELF");
        self.set_variable(frame, self_slot, class);
        let super_slot = self.slot_of(b"SUPER");
        // `.nil` for the topmost scope, which is what `superScope` answers
        // there and what `Interp::enter_method_body` writes for it.
        let super_scope = self.classes().class_super_scope(class, class);
        self.set_variable(frame, super_slot, super_scope.unwrap_or(ObjRef::NIL));
        let code = Code {
            body: &empty_body,
            symbols: &program.symbols,
            slots: &[],
            plan: None,
        };
        let result = self.eval(&code, expr);
        self.call_context = saved_context;
        self.pop_directive_activation(frame);
        result
    }

    /// Records `directive`'s own clause as the site a directive-time
    /// condition is reported against, so its report carries the same echo
    /// line the oracle prints above the two `Error` lines.
    fn blame_directive(&mut self, program: &Rc<Program>, directive: &Directive) {
        let (line, text) = directive_clause(program, directive);
        self.failure_site = Some(FailureSite::Clause {
            line,
            text,
            indent: 0,
        });
    }

    /// [`Interp::blame_directive`] for a directive in the package `id`, whose
    /// report names that package's own file when a `::REQUIRES` loaded it.
    fn blame_directive_in(&mut self, id: ProgramId, program: &Rc<Program>, directive: &Directive) {
        let (line, text) = directive_clause(program, directive);
        self.failure_site = Some(match self.required_paths.get(&id) {
            Some(path) => FailureSite::Named {
                line,
                indent: 0,
                text,
                name: path.as_bytes().to_vec(),
            },
            None => FailureSite::Clause {
                line,
                text,
                indent: 0,
            },
        });
    }

    // `plan_for` and `activation`/`activation_mut` live in `plan.rs`/
    // `activation.rs` (Task 6), beside the types they operate on.

    // `fragment_plan` and `slot_of` live in `plan.rs` (Task 6), beside
    // `Plan` itself.

    /// The variable slot `slot` of `frame` names: the frame's own storage,
    /// unless an `EXPOSE` in this activation bound that slot to a pool on the
    /// receiving object.
    #[inline(always)]
    fn variable(&self, frame: SlotFrame, slot: usize) -> Option<ObjRef> {
        if self.activation_exposes(frame) {
            return self.exposed_variable(frame, slot);
        }
        self.roots.frame_slot(frame, slot)
    }

    /// Assigns the variable slot `slot` of `frame` names.
    #[inline(always)]
    /// `pub(crate)` for `crate::ir::Op::Store`'s own fast path, which writes
    /// a simple target's slot without going through `assign_evaluated`.
    pub(crate) fn set_variable(&mut self, frame: SlotFrame, slot: usize, value: ObjRef) {
        if self.activation_exposes(frame) {
            return self.set_exposed_variable(frame, slot, value);
        }
        self.roots.set_frame_slot(frame, slot, value);
    }

    /// Returns the variable slot `slot` of `frame` names to the uninitialised
    /// state, which is what `DROP` does. Measured on the oracle: a class
    /// method that exposes `v`, assigns it and drops it leaves a later `expose
    /// v` reading the derived name `V`.
    #[inline(always)]
    fn clear_variable(&mut self, frame: SlotFrame, slot: usize) {
        if self.activation_exposes(frame) {
            return self.clear_exposed_variable(frame, slot);
        }
        self.roots.clear_frame_slot(frame, slot);
    }

    /// Whether the running activation has bound any name at all to a scope
    /// pool over `frame` -- the whole of what the three accessors above test
    /// before taking the frame, and the reason each of them is two functions.
    #[inline(always)]
    fn activation_exposes(&self, frame: SlotFrame) -> bool {
        let activation = self.activation();
        !activation.exposed.is_empty() && activation.frame == frame
    }

    /// [`Interp::variable`] for a **named** activation rather than the running
    /// one -- what `RexxContext~variables` reads a suspended context's pool
    /// through.
    pub(crate) fn variable_in(&self, activation: &Activation, slot: usize) -> Option<ObjRef> {
        let frame = activation.frame;
        match Interp::exposure_in(activation, frame, slot) {
            Some(var) => self
                .pools_of(var.owner)
                .and_then(|pools| pools.get(var.scope, &var.name)),
            None => self.roots.frame_slot(frame, slot),
        }
    }

    /// [`Interp::variable`]'s exposed half. A slot the list does not name is
    /// still an ordinary local, so this falls back rather than answering
    /// unset.
    #[cold]
    #[inline(never)]
    fn exposed_variable(&self, frame: SlotFrame, slot: usize) -> Option<ObjRef> {
        match self.exposure(frame, slot) {
            Some(var) => self
                .pools_of(var.owner)
                .and_then(|pools| pools.get(var.scope, &var.name)),
            None => self.roots.frame_slot(frame, slot),
        }
    }

    /// [`Interp::set_variable`]'s exposed half.
    #[cold]
    #[inline(never)]
    fn set_exposed_variable(&mut self, frame: SlotFrame, slot: usize, value: ObjRef) {
        // Field by field rather than through `Interp::exposure`, because the
        // name borrowed out of the activation has to stay live across the
        // `&mut self.heap` below; disjoint fields borrow independently where a
        // method taking `&self` would not.
        let activation = self.running.as_deref().expect("an activation is running");
        let Some(var) = Interp::exposure_in(activation, frame, slot) else {
            self.roots.set_frame_slot(frame, slot, value);
            return;
        };
        let pools = self
            .heap
            .get_mut(var.owner)
            .map(|object| &mut object.body)
            .and_then(|body| match body {
                Body::Instance { pools, .. } => Some(pools),
                _ => None,
            })
            .expect("an exposed variable's owner is a rooted Body::Instance");
        pools.set(var.scope, &var.name, value);
    }

    /// [`Interp::clear_variable`]'s exposed half.
    #[cold]
    #[inline(never)]
    fn clear_exposed_variable(&mut self, frame: SlotFrame, slot: usize) {
        let activation = self.running.as_deref().expect("an activation is running");
        let Some(var) = Interp::exposure_in(activation, frame, slot) else {
            self.roots.clear_frame_slot(frame, slot);
            return;
        };
        let pools = self
            .heap
            .get_mut(var.owner)
            .map(|object| &mut object.body)
            .and_then(|body| match body {
                Body::Instance { pools, .. } => Some(pools),
                _ => None,
            })
            .expect("an exposed variable's owner is a rooted Body::Instance");
        pools.clear(var.scope, &var.name);
    }

    /// Assigns one name in one scope's pool on `owner`, outside any
    /// activation's exposure list -- what a generated `::ATTRIBUTE` setter
    /// writes through.
    fn set_pool_variable(&mut self, owner: ObjRef, scope: ObjRef, name: &[u8], value: ObjRef) {
        let pools = self
            .heap
            .get_mut(owner)
            .map(|object| &mut object.body)
            .and_then(|body| match body {
                Body::Instance { pools, .. } => Some(pools),
                _ => None,
            })
            .expect("Interp::pool_owner answers a rooted Body::Instance");
        pools.set(scope, name, value);
    }

    /// What an `EXPOSE` bound slot `slot` of `frame` to, if anything.
    fn exposure(&self, frame: SlotFrame, slot: usize) -> Option<&InstanceVar> {
        Interp::exposure_in(self.activation(), frame, slot)
    }

    /// [`Interp::exposure`] over one activation, so that a caller holding a
    /// `&mut` borrow of another `Interp` field can still ask.
    fn exposure_in(activation: &Activation, frame: SlotFrame, slot: usize) -> Option<&InstanceVar> {
        if activation.exposed.is_empty() || activation.frame != frame {
            return None;
        }
        activation
            .exposed
            .iter()
            .find(|(at, _)| *at == slot)
            .map(|(_, var)| var)
    }

    /// The scope pools `owner` holds, for a reader.
    pub(crate) fn pools_of(&self, owner: ObjRef) -> Option<&rexx_core::ScopePools> {
        match self.heap.get(owner).map(|object| &object.body) {
            Some(Body::Instance { pools, .. }) => Some(pools),
            _ => None,
        }
    }

    /// Reads a variable, resolving its slot here.
    fn read(&mut self, code: &Code<'_>, id: SymbolId) -> (ObjRef, Novalue) {
        self.read_at(code, id, None)
    }

    /// Reads a variable, by the slot the plan already resolved its id to, or
    /// by `at` when a compiler resolved the same thing earlier.
    pub(crate) fn read_at(
        &mut self,
        code: &Code<'_>,
        id: SymbolId,
        at: Option<usize>,
    ) -> (ObjRef, Novalue) {
        let slot = match at {
            Some(slot) => slot,
            None => match code.slot_for(id) {
                Some(slot) => slot,
                None => self.slot_of(code.symbols.name(id).as_bytes()),
            },
        };
        let frame = self.activation().frame;
        match self.variable(frame, slot) {
            Some(value) => (value, Novalue::Set),
            None => (self.derived_name(code, id), Novalue::Unset),
        }
    }

    /// What an uninitialised read yields: the derived name, which for a
    /// simple variable is its own upcased spelling.
    #[cold]
    #[inline(never)]
    fn derived_name(&mut self, code: &Code<'_>, id: SymbolId) -> ObjRef {
        let derived = code.symbols.name(id).as_bytes();
        self.text(derived)
    }

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
            // `.environment` and `.local` are globals; the rest are classes.
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
            package_imports: _,
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
            pending_traps: _,
            active_condition: _,
            current_case_text: _,
            indent_offset: _,
            activation_indent: _,
            failure_site: _,
            failure_sites: _,
            clause_line_override: _,
            fragment_depth: _,
            stress_collect: _,
            // The collector's own resurrection flag holds each object until
            // its finalizer clears it.
            uninit_ready: _,
            processing_uninits: _,
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
            requires_installing: _,
        } = self;
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
    /// The phase that owes the family a body, spelled as every other owner
    /// string in this crate is.
    pub owner: &'static str,
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
    let (argument, program_input, deadline) = invocation.into_parts();
    interp.input = Input::new(program_input);
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
            // (`run.rs`, the `push_temp(argument.value())` beside the argument
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
            interp.trace.extend_from_slice(&raised.report(&site));
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
                interp.trace.extend_from_slice(&raised.report(&site));
            }
        }
    }

    // `MemoryObject::lastChanceUninit` (`memory/RexxMemory.cpp:324`), reached
    // from `Interpreter::terminateInterpreter` (`runtime/Interpreter.cpp:279`)
    // -- after everything the program and its replied bodies do, and reached
    // whatever the program's own outcome was. Measured, oracle: a program
    // whose main body raises 42.3 still prints its class `UNINIT` and exits
    // 214, and one ending `exit 7` prints it and exits 7.
    for loud in interp.run_termination_uninits() {
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
mod tests {
    use super::{Interp, ProgramId, form_name, parse_program, run_program};
    use rexx_parse::{DirectiveKind, Expr, ExprKind, Operator, PrefixOp, Program};
    use std::rc::Rc;

    /// An empty clause span is what tells a directive this crate synthesised
    /// from one a program wrote, and three walks skip on it --
    /// [`Interp::install_directives`], `class_members` and
    /// `environment.rs`'s `package_table_entries`.
    #[test]
    fn no_written_directive_has_an_empty_clause_span() {
        let corpus = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
        let mut directives = 0usize;
        let mut directories = vec![corpus];
        while let Some(directory) = directories.pop() {
            for entry in std::fs::read_dir(&directory).expect("a readable corpus directory") {
                let path = entry.expect("a readable directory entry").path();
                if path.is_dir() {
                    directories.push(path);
                    continue;
                }
                if path.extension().and_then(|extension| extension.to_str()) != Some("rex") {
                    continue;
                }
                let bytes = std::fs::read(&path).expect("a readable corpus program");
                let Ok(program) = parse_program(bytes) else {
                    continue;
                };
                for directive in &program.directives {
                    assert!(
                        !directive.clause_span.is_empty(),
                        "{} has a written directive whose clause span is empty, so the three \
                         walks that skip on an empty span would stop installing it",
                        path.display()
                    );
                    directives += 1;
                }
            }
        }
        assert!(
            directives > 100,
            "the walk found {directives} directives, which is too few for the corpus to have \
             been read at all"
        );
    }

    /// The path these tests report programs under.
    const TEST_PATH: &str = "/nonexistent/lib-test-program.rex";

    /// A program with a body per directive kind that carries one, which is what
    /// [`super::render_ir`] walks.
    const EVERY_BODY: &[u8] = b"\
say 1
::class k
::method m
  return 1
::attribute a get
  return 3
::routine r
  return 2
";

    /// [`super::render_ir`] renders every body a program has, each under its
    /// own heading -- not the main body alone.
    #[test]
    fn the_ir_render_covers_every_body_and_only_the_ones_that_exist() {
        let rendered = super::render_ir(EVERY_BODY.to_vec(), b"n").expect("the program parses");
        let headings: Vec<&str> = rendered
            .lines()
            .filter(|line| line.starts_with("==="))
            .collect();
        assert_eq!(
            headings,
            vec![
                "=== main ===",
                "=== ::METHOD ===",
                "=== ::ATTRIBUTE ===",
                "=== ::ROUTINE ==="
            ],
            "the render walked the wrong set of bodies\n{rendered}"
        );
        // Each body's own ops are there, not just its heading: the three
        // directive bodies each return a literal, so each owes a load and a
        // `Return`, and the main body owes a `Say`.
        assert_eq!(rendered.matches("Return").count(), 3, "{rendered}");
        assert_eq!(rendered.matches("Say").count(), 1, "{rendered}");
    }

    /// **The `TRACE` setting is an input to compilation, not a display
    /// option** (D23), and the render shows the difference.
    #[test]
    fn the_ir_render_compiles_under_the_setting_it_is_given() {
        let untraced = super::render_ir(EVERY_BODY.to_vec(), b"n").expect("the program parses");
        let traced = super::render_ir(EVERY_BODY.to_vec(), b"r").expect("the program parses");
        assert!(
            !untraced.contains("TraceClause"),
            "a chunk compiled under N carries a clause echo op\n{untraced}"
        );
        assert!(
            traced.contains("TraceClause"),
            "a chunk compiled under R carries no clause echo op\n{traced}"
        );
    }

    /// **A clause-opening op carries the source clause it stands for, and an
    /// op inside a region does not.**
    #[test]
    fn the_ir_render_names_the_clause_a_region_opens_and_not_its_inner_ops() {
        // `DROP` delegates and `SAY` has compiled operands, so this program
        // has a region of each shape: one holding a single `Exec`, one holding
        // the ops that compute the value.
        let rendered = super::render_ir(b"drop zn\nsay 'x'\n".to_vec(), b"n").expect("parses");
        let lines: Vec<&str> = rendered.lines().collect();
        assert!(
            lines
                .iter()
                .any(|line| line.starts_with("0: Clause") && line.ends_with("; 1: drop zn")),
            "the Clause op does not name the clause it opens\n{rendered}"
        );
        assert!(
            lines
                .iter()
                .any(|line| line.starts_with("2: Clause") && line.ends_with("; 2: say 'x'")),
            "the second Clause op does not name the clause it opens\n{rendered}"
        );
        assert!(
            lines
                .iter()
                .any(|line| line.starts_with("1: Exec") && !line.contains(';')),
            "the Exec op inside a region repeats its region's clause\n{rendered}"
        );
        assert!(
            lines
                .iter()
                .filter(|line| line.starts_with("3: ") || line.starts_with("4: "))
                .all(|line| !line.contains(';')),
            "an op inside a region repeats its region's clause\n{rendered}"
        );
    }

    /// A setting that is not a `TRACE` setting is reported, rather than
    /// silently compiling under some other one.
    #[test]
    fn the_ir_render_refuses_a_setting_that_is_not_one() {
        let refused = super::render_ir(EVERY_BODY.to_vec(), b"zz");
        assert!(
            refused.is_err_and(|report| report.contains("not a TRACE setting")),
            "an unusable TRACE setting was accepted"
        );
    }

    fn literal() -> Expr {
        Expr::new(ExprKind::Literal(Box::from(&b"1"[..])), 0..1)
    }

    fn nest(depth: usize) -> Expr {
        let mut node = literal();
        for _ in 0..depth {
            node = Expr::new(
                ExprKind::Binary {
                    op: Operator::Plus,
                    left: Box::new(node),
                    right: Box::new(literal()),
                },
                0..1,
            );
        }
        node
    }

    /// `Loud::expression`'s size contract, tested on the two arms that can
    /// break it.
    #[test]
    fn the_two_formatting_arms_do_not_grow_with_the_subtree() {
        let deep = nest(200);
        let shallow = nest(1);
        assert_eq!(form_name(&deep.kind), form_name(&shallow.kind));
        assert_eq!(form_name(&deep.kind), "the operator `+`");

        let deep = Expr::new(
            ExprKind::Prefix {
                op: PrefixOp::Minus,
                operand: Box::new(nest(200)),
            },
            0..1,
        );
        let shallow = Expr::new(
            ExprKind::Prefix {
                op: PrefixOp::Minus,
                operand: Box::new(literal()),
            },
            0..1,
        );
        assert_eq!(form_name(&deep.kind), form_name(&shallow.kind));
        assert_eq!(form_name(&deep.kind), "the prefix operator `-`");
    }

    // ---- the fragment's lifetime (I7) ----

    /// Step 4's test, and the one property `INTERPRET` has that no other
    /// instruction does: a name bound inside fragment text outlives the
    /// fragment, so a *later, separate* fragment reads it back.
    #[test]
    fn interpret_binds_a_name_the_enclosing_body_never_mentions() {
        let outcome = run_program(
            TEST_PATH,
            b"interpret \"zork = 42\"\ninterpret \"say zork\"\n".to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
        assert_eq!(outcome.stdout, b"42\n");
    }

    /// Step 2, and the reason the spike exists in the shape it does.
    /// ```text
    /// zzz = 'from the enclosing frame'
    /// interpret "say zzz"
    /// interpret "zork = 42"
    /// interpret "say zork"
    /// zzz = zzz || '!'
    /// interpret "say zzz"
    /// ```
    /// ```text
    /// from the enclosing frame
    /// 42
    /// from the enclosing frame!
    /// ```
    #[test]
    fn a_fragment_shares_the_enclosing_frames_variable_pool() {
        let program = b"zzz = 'from the enclosing frame'\n\
                        interpret \"say zzz\"\n\
                        interpret \"zork = 42\"\n\
                        interpret \"say zork\"\n\
                        zzz = zzz || '!'\n\
                        interpret \"say zzz\"\n";
        let outcome = run_program(TEST_PATH, program.to_vec(), crate::Invocation::none());
        assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
        assert_eq!(
            outcome.stdout,
            b"from the enclosing frame\n42\nfrom the enclosing frame!\n"
        );
    }

    /// `EXIT` inside a fragment ends the *program*, not the fragment, so control
    /// leaves the nested loop and the enclosing one together and both `Rc` locals
    /// drop in order.
    /// ```text
    /// say 'before'
    /// interpret "say 'inside'"
    /// interpret "exit"
    /// say 'after'
    /// ```
    #[test]
    fn an_exit_inside_a_fragment_ends_the_program() {
        let program = b"say 'before'\n\
                        interpret \"say 'inside'\"\n\
                        interpret \"exit\"\n\
                        say 'after'\n";
        let outcome = run_program(TEST_PATH, program.to_vec(), crate::Invocation::none());
        assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
        assert_eq!(outcome.stdout, b"before\ninside\n");
    }

    /// A condition raised inside an `INTERPRET` fragment reports
    /// **both** clauses, and this is the whole report, byte for byte, at the
    /// one level `run_program` can see it.
    /// ```text
    ///      3 *-*       say 2 & 1;
    ///      3 *-*     interpret "do jj = 1 to 1; say 2 & 1; end"
    /// Error 34 running <path> line 3:  Logical value not 0 or 1.
    /// Error 34.901:  Logical value must be exactly "0" or "1"; found "2".
    /// ```
    #[test]
    fn a_raise_inside_a_fragment_reports_both_clauses() {
        let program = b"do kk = 1 to 1\n\
                        do mm = 1 to 1\n\
                        interpret \"do jj = 1 to 1; say 2 & 1; end\"\n\
                        end\n\
                        end\n";
        let outcome = run_program(TEST_PATH, program.to_vec(), crate::Invocation::none());
        assert_eq!(outcome.exit_code, 222);
        assert_eq!(outcome.stdout, b"");
        assert_eq!(
            String::from_utf8(outcome.stderr).unwrap(),
            format!(
                concat!(
                    "     3 *-*       say 2 & 1;\n",
                    "     3 *-*     interpret \"do jj = 1 to 1; say 2 & 1; end\"\n",
                    "Error 34 running {path} line 3:  Logical value not 0 or 1.\n",
                    "Error 34.901:  Logical value must be exactly \"0\" or \"1\"; found \"2\".\n",
                ),
                path = TEST_PATH
            )
        );
    }

    /// Review round 1, F1 and its neighbours: the activation base survives
    /// every construct inside the fragment that writes an indent of its own.
    #[test]
    fn a_fragments_activation_base_survives_every_indent_writer_inside_it() {
        // (program, expected stderr with `{path}` for the program's path)
        let rows: &[(&str, &str)] = &[
            (
                "do z = 1 to 1\n\
                 interpret \"select; when 1 = 0 then nop; otherwise nop; end; say 1/0\"\n\
                 end\n",
                concat!(
                    "     2 *-*   say 1/0\n",
                    "     2 *-*   interpret \"select; when 1 = 0 then nop; otherwise nop; \
                     end; say 1/0\"\n",
                    "Error 42 running {path} line 2:  Arithmetic overflow/underflow.\n",
                    "Error 42.3:  Arithmetic overflow; divisor must not be zero.\n",
                ),
            ),
            (
                "do z = 1 to 1\n\
                 interpret \"do jj = 1 to 1; leave zz; end\"\n\
                 end\n",
                concat!(
                    "     2 *-*   leave zz;\n",
                    "     2 *-*   interpret \"do jj = 1 to 1; leave zz; end\"\n",
                    "Error 28 running {path} line 2:  Invalid LEAVE or ITERATE.\n",
                    "Error 28.3:  Symbol following LEAVE (\"ZZ\") must either match the \
                     label of a current loop or block instruction.\n",
                ),
            ),
            (
                "do z = 1 to 1\n\
                 select case 2\n\
                 \x20 when 2 then\n\
                 \x20   when 3 then nop\n\
                 \x20 otherwise interpret \"do jj = 1 to 1; say 1/0; end\"\n\
                 end\n\
                 end\n",
                concat!(
                    "     5 *-*             say 1/0;\n",
                    "     5 *-*           interpret \"do jj = 1 to 1; say 1/0; end\"\n",
                    "Error 42 running {path} line 5:  Arithmetic overflow/underflow.\n",
                    "Error 42.3:  Arithmetic overflow; divisor must not be zero.\n",
                ),
            ),
        ];
        for (index, (program, expected)) in rows.iter().enumerate() {
            let outcome = run_program(
                TEST_PATH,
                program.as_bytes().to_vec(),
                crate::Invocation::none(),
            );
            assert_eq!(
                String::from_utf8(outcome.stderr).unwrap(),
                expected.replace("{path}", TEST_PATH),
                "row {index}"
            );
        }
    }

    /// Review round 1, F2: the `WHEN` scan's own echo carries the offsets too.
    #[test]
    fn a_when_scan_inside_a_fragment_echoes_at_the_fragments_own_indent() {
        let program = "trace r\n\
                       do\n\
                       interpret \"select; when 1 = 1 then nop; end; nop\"\n\
                       end\n";
        let outcome = run_program(
            TEST_PATH,
            program.as_bytes().to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(outcome.exit_code, 0);
        assert_eq!(outcome.stdout, b"");
        assert_eq!(
            String::from_utf8(outcome.stderr).unwrap(),
            concat!(
                "     2 *-* do\n",
                "     3 *-*   interpret \"select; when 1 = 1 then nop; end; nop\"\n",
                "       >>>     \"select; when 1 = 1 then nop; end; nop\"\n",
                "     3 *-*   select;\n",
                "     3 *-*     when 1 = 1 \n",
                "       >>>       \"1\"\n",
                "     3 *-*       then\n",
                "     3 *-*         nop;\n",
                "     3 *-*   nop\n",
                "     4 *-* end\n",
            )
        );
    }

    /// A fragment that does not parse raises the oracle's own condition
    /// instead of failing loudly.
    #[test]
    fn a_fragment_that_does_not_parse_raises_the_oracles_condition() {
        let outcome = run_program(
            TEST_PATH,
            b"say 1\ninterpret \"do forever then\"\n".to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(outcome.exit_code, 229);
        assert_eq!(outcome.stdout, b"1\n");
        assert_eq!(
            String::from_utf8(outcome.stderr).unwrap(),
            format!(
                concat!(
                    "     2 *-* interpret \"do forever then\"\n",
                    "Error 27 running {path} line 2:  Invalid DO or LOOP syntax.\n",
                    "Error 27.901:  Incorrect data following FOREVER keyword on the loop; \
                     found \"&1\".\n",
                ),
                path = TEST_PATH
            )
        );

        let outcome = run_program(
            TEST_PATH,
            b"say 1\ninterpret \"if\"\n".to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(outcome.exit_code, 221);
    }

    /// The reported span comes from one call chain, so what else the program
    /// evaluated cannot change it.
    #[test]
    fn the_stack_span_does_not_depend_on_what_else_the_program_evaluated() {
        let mut alone = b"say 'a'".to_vec();
        for _ in 1..1_000 {
            alone.extend_from_slice(b"||''");
        }
        alone.push(b'\n');

        let mut then_a_fragment = alone.clone();
        then_a_fragment.extend_from_slice(b"interpret \"say 'b'\"\n");

        let on_eval = || crate::Invocation::none();
        let alone = run_program(TEST_PATH, alone, on_eval());
        let then_a_fragment = run_program(TEST_PATH, then_a_fragment, on_eval());

        assert_eq!(alone.exit_code, 0, "stderr: {:?}", alone.stderr);
        assert_eq!(
            then_a_fragment.exit_code, 0,
            "stderr: {:?}",
            then_a_fragment.stderr
        );
        assert_eq!(
            alone.stack.max_depth, then_a_fragment.stack.max_depth,
            "the fragment's own evaluation is shallow, so it must not move the maximum"
        );
        assert_eq!(
            alone.stack.bytes, then_a_fragment.stack.bytes,
            "the span must come from the chain that reached the maximum, not from the last \
             top-level evaluation to start"
        );
    }

    // ---- R9: ::CLASS/::METHOD/::ATTRIBUTE create and record in
    // `Interp::classes`. No corpus program can see this yet -- reading a
    // class object back needs a message send, and `ExprKind::Message` still
    // fails loudly (Task 5's), so this is witnessed here, against the
    // registry `install_directives` leaves behind, instead. ----

    /// Parses `text` and installs its directives against a fresh `Interp`,
    /// under `ProgramId(0)`, handing both back so a test can read the
    /// registry state left behind.
    fn installed(text: &[u8]) -> (Interp, Rc<Program>) {
        let program = Rc::new(parse_program(text.to_vec()).expect("test program parses"));
        let mut interp = Interp::new();
        interp
            .install_directives(ProgramId(0), &program)
            .expect("test program installs without a raised condition");
        (interp, program)
    }

    /// The class `::CLASS name` installed, read out of the package's own
    /// table.
    fn installed_class(interp: &Interp, name: &str) -> rexx_core::ObjRef {
        interp.package_classes[&ProgramId(0)][name.as_bytes()]
    }

    /// Had `::CLASS` not created anything, the package's own table would hold
    /// nothing and the index below would panic.
    #[test]
    fn a_bare_class_directive_creates_a_class_object() {
        let (interp, _program) = installed(b"say 'main ran'\n::class Foo\n");
        let id = installed_class(&interp, "FOO");
        assert!(interp.heap.is_class(id), "a class identity, not a value");
    }

    /// **An installed class is not an environment entry**, which is the whole
    /// of why `install_class` does not register the name.
    #[test]
    fn an_installed_class_does_not_displace_the_environments_own_entry() {
        let (mut interp, _program) = installed(b"say 'main ran'\n::class array\n");
        let installed = installed_class(&interp, "ARRAY");
        let native = interp.classes().lookup("Array").expect("Array is native");
        assert_ne!(installed, native);
        // Upcased, because the directive names the class with a symbol and
        // the scanner interns a symbol upcased -- which is why the oracle
        // prints `The ARRAY class` for it and `The Array class` for the
        // environment's own.
        assert_eq!(interp.classes().id_string(installed), "ARRAY");
        assert_eq!(interp.classes().id_string(native), "Array");
    }

    /// A bare `::CLASS` is `subclass Object`, `metaclass Class`, which is
    /// what `RexxClass::subclass`'s own defaults give it. Had `install_class`
    /// left the superclass out, or named some other class, this would
    /// catch it.
    #[test]
    fn a_bare_class_directive_subclasses_object() {
        let (mut interp, _program) = installed(b"say 'main ran'\n::class Foo\n");
        let id = installed_class(&interp, "FOO");
        let object = interp.classes().lookup("Object").unwrap();
        let class = interp.classes().lookup("Class").unwrap();
        assert_eq!(interp.classes().superclass(id), Some(object));
        assert_eq!(interp.classes().metaclass(id), class);
    }

    /// **`~metaClass` and `~class` are two different fields of a class
    /// object, and they part iff the superclass is a metaclass and is not the
    /// named-or-inherited metaclass.**
    /// ```text
    /// ::class S  MIXINCLASS Class         ~metaClass Class  ~class Class   same   stated, cannot fail
    /// ::class M1 MIXINCLASS Class         ~metaClass Class  ~class Class   same
    /// ::class T  SUBCLASS S METACLASS M1  ~metaClass S      ~class M1      part   asserted
    /// ::class T2 SUBCLASS S               ~metaClass S      ~class Class   part   asserted
    /// ::class K  METACLASS M1             ~metaClass M1     ~class M1      same   asserted
    /// ::class P                           ~metaClass Class  ~class Class   same   asserted
    /// ```
    #[test]
    fn a_class_objects_metaclass_and_its_class_are_separate_fields() {
        let (mut interp, _program) = installed(
            b"say 'main ran'\n\
              ::class M1 mixinclass class\n\
              ::class S mixinclass class\n\
              ::class T subclass S metaclass M1\n\
              ::class T2 subclass S\n\
              ::class K metaclass M1\n\
              ::class P\n",
        );
        let class = interp.classes().lookup("Class").unwrap();
        let m1 = installed_class(&interp, "M1");
        let s = installed_class(&interp, "S");
        let t = installed_class(&interp, "T");
        let t2 = installed_class(&interp, "T2");
        let k = installed_class(&interp, "K");
        let p = installed_class(&interp, "P");

        // Derived from a metaclass and naming one: the two fields disagree,
        // and each holds what the other does not.
        assert_eq!(interp.classes().metaclass(t), s, "T~metaClass");
        assert_eq!(interp.classes().class_of(t), m1, "T~class");
        // Derived from a metaclass, naming none: they disagree here too, so
        // the split is not an artifact of writing METACLASS down.
        assert_eq!(interp.classes().metaclass(t2), s, "T2~metaClass");
        assert_eq!(interp.classes().class_of(t2), class, "T2~class");
        // Derived from a metaclass and yet the two agree, which is the row
        // that refutes "the fields part wherever a class derives from a
        // metaclass": deriving from one is necessary and is not sufficient.
        assert_eq!(interp.classes().metaclass(s), class, "S~metaClass");
        assert_eq!(interp.classes().class_of(s), class, "S~class");
        // Naming one under a superclass that is not a metaclass, and naming
        // none at all: nothing overrides, and the two agree. Without these
        // the test would admit a build that simply answered different things.
        assert_eq!(interp.classes().metaclass(k), m1, "K~metaClass");
        assert_eq!(interp.classes().class_of(k), m1, "K~class");
        assert_eq!(interp.classes().metaclass(p), class, "P~metaClass");
        assert_eq!(interp.classes().class_of(p), class, "P~class");
    }

    /// The other half of the same install: a user class inherits `.Object`'s
    /// own instance methods through the flattened cascade, which is what a
    /// send to one of its instances would resolve against. Had
    /// `install_class` recorded no superclass, this set would hold `BAR`
    /// alone.
    #[test]
    fn a_user_class_inherits_objects_instance_methods() {
        let (mut interp, _program) =
            installed(b"say 'main ran'\n::class Foo\n::method bar\n  return 1\n");
        let id = installed_class(&interp, "FOO");
        let names = interp.classes().instance_method_names(id);
        assert!(names.contains("BAR"));
        assert!(names.contains("HASMETHOD"));
    }

    /// Had `::METHOD` landed in the class dictionary instead of the instance
    /// one (or nowhere), one side of this pair would be wrong.
    #[test]
    fn a_method_directive_lands_in_the_classs_instance_dictionary() {
        let (mut interp, _program) =
            installed(b"say 'main ran'\n::class Foo\n::method bar\n  return 1\n");
        let id = installed_class(&interp, "FOO");
        assert!(
            interp
                .classes()
                .own_instance_method_names(id)
                .contains("BAR")
        );
        assert!(!interp.classes().own_class_method_names(id).contains("BAR"));
    }

    /// `::METHOD ... CLASS`'s own side of the same pair.
    #[test]
    fn a_class_method_directive_lands_in_the_classs_class_dictionary() {
        let (mut interp, _program) =
            installed(b"say 'main ran'\n::class Foo\n::method bar class\n  return 1\n");
        let id = installed_class(&interp, "FOO");
        assert!(interp.classes().own_class_method_names(id).contains("BAR"));
        assert!(
            !interp
                .classes()
                .own_instance_method_names(id)
                .contains("BAR")
        );
    }

    /// The `UNINIT` flags, **through the directive path** -- which is the
    /// half `rexx-classes`' own graph-API test cannot reach.
    #[test]
    fn the_uninit_flags_are_set_for_the_classes_a_file_declares() {
        let (mut interp, _program) = installed(
            b"say 'main ran'
              ::class Base
              ::method uninit
  return
              ::class Kid subclass Base
              ::class Grandkid subclass Kid
              ::class Plain
              ::class Plainkid subclass Plain
",
        );
        let base = installed_class(&interp, "BASE");
        let kid = installed_class(&interp, "KID");
        let grandkid = installed_class(&interp, "GRANDKID");
        let plain = installed_class(&interp, "PLAIN");
        let plainkid = installed_class(&interp, "PLAINKID");

        // The class that declares it, and the ones that reach it through the
        // flattened behaviour the oracle's `checkUninit` reads.
        assert!(interp.classes().has_uninit(base), "the declaring class");
        assert!(interp.classes().has_uninit(kid), "its subclass");
        assert!(
            interp.classes().has_uninit(grandkid),
            "and one generation further down"
        );

        // The separate flag, propagated rather than looked up.
        assert!(!interp.classes().parent_has_uninit(base));
        assert!(interp.classes().parent_has_uninit(kid));
        assert!(interp.classes().parent_has_uninit(grandkid));

        // The negative rows: an ancestry with no UNINIT in it leaves both
        // flags clear, so a build that set them unconditionally fails here.
        assert!(!interp.classes().has_uninit(plain));
        assert!(!interp.classes().has_uninit(plainkid));
        assert!(!interp.classes().parent_has_uninit(plainkid));
    }

    /// A class-side `::METHOD uninit CLASS` sets neither flag, and that is
    /// the oracle's answer rather than a gap.
    #[test]
    fn a_class_side_uninit_sets_neither_flag() {
        let (mut interp, _program) = installed(
            b"say 'main ran'
              ::class K
              ::method uninit class
  return
",
        );
        let id = installed_class(&interp, "K");
        assert!(!interp.classes().has_uninit(id));
        assert!(!interp.classes().parent_has_uninit(id));
        assert!(
            interp
                .classes()
                .own_class_method_names(id)
                .contains("UNINIT"),
            "the method did install, on the class side"
        );
    }

    /// Neither `GET` nor `SET`: both accessor names install. Had the `=`
    /// suffix been on the wrong name, or missing, one side of this pair
    /// would fail.
    #[test]
    fn an_attribute_with_no_style_installs_both_accessor_names() {
        let (mut interp, _program) = installed(b"say 'main ran'\n::class Foo\n::attribute baz\n");
        let id = installed_class(&interp, "FOO");
        let names = interp.classes().own_instance_method_names(id);
        assert!(names.contains("BAZ"));
        assert!(names.contains("BAZ="));
    }

    /// `GET` alone installs only the getter -- had `install_attribute`
    /// always installed both names regardless of style, `BAZ=` would be
    /// present here too.
    #[test]
    fn an_attribute_get_installs_only_the_getter() {
        let (mut interp, _program) =
            installed(b"say 'main ran'\n::class Foo\n::attribute baz get\n");
        let id = installed_class(&interp, "FOO");
        let names = interp.classes().own_instance_method_names(id);
        assert!(names.contains("BAZ"));
        assert!(!names.contains("BAZ="));
    }

    /// `::METHOD ... ATTRIBUTE` generates the same pair `::ATTRIBUTE` does,
    /// and each half is recorded as the half it is.
    #[test]
    fn a_method_attribute_installs_a_getter_and_a_setter() {
        let (mut interp, _program) =
            installed(b"say 'main ran'\n::class Foo\n::method baz attribute\n");
        let id = installed_class(&interp, "FOO");
        let names = interp.classes().own_instance_method_names(id);
        assert!(names.contains("BAZ"));
        assert!(names.contains("BAZ="));
        assert_eq!(generated_kinds(&interp), vec!["Getter", "Setter"]);
        assert!(
            interp.method_bodies.is_empty(),
            "a generated accessor is not a row of the body table"
        );
    }

    /// `ABSTRACT` under `ATTRIBUTE` replaces both halves rather than one, on
    /// either directive.
    #[test]
    fn an_abstract_accessor_pair_is_abstract_on_both_halves() {
        for source in [
            b"say 'main ran'\n::class Foo\n::method baz class abstract attribute\n".to_vec(),
            b"say 'main ran'\n::class Foo\n::attribute baz class abstract\n".to_vec(),
        ] {
            let (interp, _program) = installed(&source);
            assert_eq!(
                generated_kinds(&interp),
                vec!["Abstract", "Abstract"],
                "{:?}",
                String::from_utf8_lossy(&source)
            );
        }
    }

    /// A `DELEGATE` method is a row of the generated table and not of the
    /// body one, and under `ATTRIBUTE` it is two rows rather than one.
    #[test]
    fn a_delegate_method_is_a_generated_method() {
        let (interp, _program) =
            installed(b"say 'main ran'\n::class Foo\n::method baz class delegate p\n");
        assert_eq!(generated_kinds(&interp), vec!["Delegate"]);
        assert!(
            interp.method_bodies.is_empty(),
            "a delegate method is not a row of the body table"
        );

        for source in [
            b"say 'main ran'\n::class Foo\n::method baz class delegate p attribute\n".to_vec(),
            b"say 'main ran'\n::class Foo\n::attribute baz class delegate p\n".to_vec(),
        ] {
            let (mut interp, _program) = installed(&source);
            assert_eq!(
                generated_kinds(&interp),
                vec!["Delegate", "Delegate"],
                "{:?}",
                String::from_utf8_lossy(&source)
            );
            let id = installed_class(&interp, "FOO");
            let names = interp.classes().own_class_method_names(id);
            assert!(names.contains("BAZ"), "{names:?}");
            assert!(names.contains("BAZ="), "{names:?}");
        }
    }

    /// Every generated method `install_directives` recorded, as its `Debug`
    /// spelling, in sorted order. A `Vec` rather than a set so a pair
    /// recorded as halves of the same kind reads differently from one of
    /// each.
    fn generated_kinds(interp: &Interp) -> Vec<String> {
        let mut kinds: Vec<String> = interp
            .generated_methods
            .values()
            .map(|generated| format!("{:?}", generated.kind))
            .collect();
        kinds.sort();
        kinds
    }

    /// A loose `::METHOD` with no preceding `::CLASS` installs on the oracle
    /// (measured, rc 0 "main ran") and has nothing here to attach to -- had
    /// `install_directives` recorded it against a stale or default class id
    /// instead of skipping it, `method_bodies` would be non-empty here.
    #[test]
    fn a_loose_method_with_no_preceding_class_is_not_recorded() {
        let (interp, _program) = installed(b"say 'main ran'\n::method bar\n  return 1\n");
        assert!(interp.method_bodies.is_empty());
    }

    /// The "bodies are stored" half of R9: `method_bodies` names the exact
    /// directive a method's own body came from, keyed by the identity the
    /// registry minted for it. Had `record_method_body` recorded the wrong
    /// directive index, or keyed the two methods the same way, this would
    /// catch it; had a mint been recorded twice, the `debug_assert!` inside
    /// `record_method_body` catches that first, in every debug build
    /// including the workspace's own gate.
    #[test]
    fn method_bodies_names_each_directive_by_its_minted_identity() {
        let (mut interp, program) = installed(
            b"say 'main ran'\n::class Foo\n::method bar\n  return 1\n::method baz class\n  return 2\n",
        );
        let index_of = |name: &[u8]| -> usize {
            program
                .directives
                .iter()
                .position(|d| matches!(&d.kind, DirectiveKind::Method(m) if &*m.name == name))
                .expect("the ::METHOD directive is in the program")
        };
        let id = installed_class(&interp, "FOO");
        // A bare symbol's own name is already upcased by the scanner (a
        // quoted literal is the shape that would keep the source case), so
        // `bar`/`baz class` in the source above are `BAR`/`BAZ` here.
        let (_, bar) = interp
            .classes()
            .lookup_instance_method(id, "BAR")
            .expect("::method bar resolves");
        let (_, baz) = interp
            .classes()
            .lookup_class_method(id, "BAZ")
            .expect("::method baz class resolves");
        assert_eq!(interp.method_bodies.len(), 2);
        assert_eq!(interp.method_bodies[&bar].program, ProgramId(0));
        assert_eq!(interp.method_bodies[&bar].directive, index_of(b"BAR"));
        assert_eq!(interp.method_bodies[&baz].program, ProgramId(0));
        assert_eq!(interp.method_bodies[&baz].directive, index_of(b"BAZ"));
    }

    /// **Every class the interpreter's own library declares carries
    /// `REXX_DEFINED`** -- asserted over the set the bootstrap leaves behind
    /// rather than on the classes a corpus row happens to name.
    #[test]
    fn every_class_the_library_declares_carries_the_rexx_defined_flag() {
        let mut interp = Interp::new();
        interp
            .bootstrap_library()
            .expect("the library bootstrap runs");
        let programs = interp.library_programs.clone();
        let mut open: Vec<String> = Vec::new();
        let mut checked = 0;
        let mut non_public = 0;
        for program in programs {
            // `get`, not an index: an embedded file declaring no `::CLASS`
            // has no table at all, which `PlatformObjects.orx` is.
            let Some(table) = interp.package_classes.get(&program) else {
                continue;
            };
            let classes: Vec<(Vec<u8>, rexx_core::ObjRef)> = table
                .iter()
                .map(|(name, id)| (name.to_vec(), *id))
                .collect();
            let public = interp
                .package_public_classes
                .get(&program)
                .cloned()
                .unwrap_or_default();
            for (name, id) in classes {
                checked += 1;
                if !public.contains_key(name.as_slice()) {
                    non_public += 1;
                }
                if !interp.classes().is_rexx_defined(id) {
                    open.push(String::from_utf8_lossy(&name).into_owned());
                }
            }
        }
        open.sort_unstable();
        assert!(
            open.is_empty(),
            "the library declared classes a program can still mutate: {open:?}"
        );
        // Anti-vacuity, both halves. An empty table, or a bootstrap that
        // installed only public classes, would pass the loop above by
        // having nothing in it to fail.
        assert!(checked > 0, "the library declared no classes at all");
        assert!(
            non_public > 0,
            "no class without PUBLIC was seen, so the half the corpus cannot \
             reach is not what this test read"
        );
    }
}
