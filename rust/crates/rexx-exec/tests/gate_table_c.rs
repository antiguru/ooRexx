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

//! Gate table C: the concept and class surface.
//!
//! Four row classes over the row sets `corpus/docs/` carries, sharing gate
//! table D's harness (`tests/gate_tables/mod.rs`) for the verdict function,
//! the two-engine crate run, the structural channel and the report:
//!
//! * **concept rows**, one per `<section id>` under `provide.xml`'s `provide`
//!   chapter (`provide-sections.txt`), each naming the probe program that
//!   exercises the section's mechanism and the negative control that would
//!   redden it;
//! * **wiring rows**, one per class in `class-set.txt` -- asked what the
//!   `.environment` entry renders as and what its class is, questions any
//!   entry answers, and then `~id`, `~class`, `~superClass`, `~superClasses`,
//!   `~metaClass` and `~isA(.Class)`, which only a class object answers --
//!   and one per documented edge in `hierarchy-edges.txt`;
//! * **the `ArgUtil` assertion**, which is a wiring row with **no verdict
//!   channel** for the reason [`argutil_assertion`] gives in full;
//! * **method rows**, one per (class, method, arm) in `class-methods.txt`,
//!   owned by 5c and reported here rather than gated.
//!
//! # The table types no expected bytes, and one expected count
//!
//! As in table D, no oracle *answer* is recorded anywhere in this file or
//! beside it. Every row's oracle column is produced by launching the oracle
//! on the run that reports it, so a row cannot pass by agreeing with a
//! recording of itself.
//!
//! The one exception is [`Concept::oracle_lines`], a line count per concept
//! row. It is a shape and not an answer -- it holds no byte the oracle
//! produced, and it can only make a row **fail** -- and the concept family is
//! the one whose probes nothing derives, so it is where the bound has to come
//! from somewhere. Its own doc carries the rest.
//!
//! # A non-`agree` row is not a failing test
//!
//! A verdict failure is an exit status only under [`gate_tables::CORPUS_GATE_ENV`]
//! and only for a row whose owning phase is closing
//! ([`gate_tables::PHASE_GATE_ENV`]) or already closed
//! ([`gate_tables::CLOSED_PHASES`]). Outside that, this table is a progress
//! report: a row that does not `agree` is a row waiting on the task that owns
//! it, and the number of non-`agree` rows per phase is what each task reports
//! against its predecessor's. A **structural** failure is red in every mode
//! and is never one of those.
//!
//! # A verdict says the two sides agree; it does not say either answered
//!
//! Before any row gets a verdict, the oracle's `stdout` is checked against
//! the shape the row's probe could produce while asking its question --
//! [`OracleShape`], bounded by the derived probe text for the families whose
//! text is derived and by a committed count for the concept family, whose
//! probes are hand-written. For a class row that bound is read through the
//! `entry` column, and it is paired with [`check_entry_present`], because a
//! count alone cannot tell an entry from a name nothing defines: an
//! unresolved environment symbol answers the opening questions as a string.
//! Two interpreters that fail identically at a
//! probe's first instruction agree on all three descriptors, so without this
//! a row over a class neither of them has reads `agree` and is counted as a
//! satisfied row of its phase. A row that fails the check keeps its place in
//! the table and is reported as `unanswered`, never as `agree`: dropping it
//! would lower the gated count, which is a close criterion satisfiable by
//! removing evidence.
//!
//! # The probe corpus is derived from the row set, and checked in both
//! directions
//!
//! Table D derives each row's probe **path** from the row. The class, edge and
//! method families here go further and derive the probe's **text**:
//! `class_probe_text`, `edge_probe_text` and `method_probe_text` are the sole
//! definition of what those programs contain, and every run compares the
//! committed file against the re-derivation. A path-only check cannot see a
//! probe that asks about the wrong class -- `classes/string.rex` asking
//! `.Array~id` would agree with the oracle for the wrong reason, and nothing
//! about the row would notice. The concept probes are hand-written, because
//! nothing derives a program that exercises a documented mechanism, and they
//! keep table D's path-only property.
//!
//! # Method rows share a program, and their stdout channel is one line of it
//!
//! One oracle launch per (class, arm) rather than per row, for the reason the
//! spec gives: every row costs a process launch. A row's `status` and
//! `stderr` channels are therefore the whole program's, which is correct --
//! a program that died did not answer any of its rows -- and its `stdout`
//! channel is its own line of the program's output. [`method_row_stdout`]
//! carries the exact rule, including why a line-count mismatch reddens every
//! row rather than only the rows past the mismatch.
//!
//! # What this table cannot see
//!
//! * **Whether a method is defined at a class's own scope.** Measured,
//!   `.Array~hasMethod("ID")` and `.Array~hasMethod("DEFINE")` are both 1
//!   though `id` and `define` are `Class`'s, and `.Array~method("ID")` raises
//!   because `~method` reads the instance dictionary. A build that moved a
//!   class method between scopes is not caught here. The plan builds no scope
//!   row class -- the documentation supplies no expected answer for the scope
//!   question -- and the instrument is instead the `~method` corpus programs
//!   Task 9 commits.
//! * **The operator-frame traceback line.** The spec records it as a
//!   mechanism with no documented section, and this table's concept rows are
//!   one per `provide.xml` section id, so it has no row here. Its instrument
//!   is Task 6's corpus programs.
//! * **A section's mechanism beyond what its one probe reaches.** A concept
//!   row is one program; a claim of the section that program does not
//!   exercise is outside the row. Each concept arm's `control` field names
//!   the mutation that would redden the row, which is the honest statement of
//!   what the row does reach.
//! * **A row filed under the wrong phase.** As in table D, the owning phase
//!   is a committed assignment read by a human out of a diff, and a row filed
//!   under the wrong one escapes gating.

mod gate_tables;
mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use gate_tables::{
    Descriptors, Report, Structural, Verdict, assert_no_structural_failures, compare_raw, excerpt,
    is_loud, refused_construct, run_on_both_engines, stdout_lines, verdict, verdict_is_gated,
    verdict_label,
};
use support::oracle::{CppOutcome, did_not_finish, wrapped_exit_code};

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus")
}

/// Where each family's probe programs live, relative to the corpus root.
const CONCEPT_SUBDIR: &str = "gate-tables/concepts";
const CLASS_SUBDIR: &str = "gate-tables/classes";
const EDGE_SUBDIR: &str = "gate-tables/hierarchy";
const METHOD_SUBDIR: &str = "gate-tables/methods";

/// Reads one committed row file as tab-separated fields.
///
/// Comment and blank lines are dropped, the same way every other reader of a
/// corpus list drops them, and a row with the wrong number of fields is a
/// panic rather than a silently short row: these files are derived and
/// re-derived in both directions by `rexx-extract/tests/extract_docs.rs`, so
/// a malformed line here means the file this table's denominator comes from
/// is not the file that check polices.
fn read_table(name: &str, fields: usize) -> Vec<Vec<String>> {
    let path = corpus_dir().join("docs").join(name);
    let text =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let mut rows = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let row: Vec<String> = line.split('\t').map(str::to_string).collect();
        assert_eq!(
            row.len(),
            fields,
            "{}: row {line:?} has the wrong number of fields",
            path.display()
        );
        rows.push(row);
    }
    assert!(
        !rows.is_empty(),
        "{} named no rows -- that is a defect in the row file, not an empty pass",
        path.display()
    );
    rows
}

/// One `provide.xml` section, from `provide-sections.txt`.
struct Section {
    id: String,
    depth: String,
    line: String,
    parent: String,
    title: String,
}

fn read_sections() -> Vec<Section> {
    read_table("provide-sections.txt", 5)
        .into_iter()
        .map(|row| Section {
            id: row[0].clone(),
            depth: row[1].clone(),
            line: row[2].clone(),
            parent: row[3].clone(),
            title: row[4].clone(),
        })
        .collect()
}

/// One documented class, from `class-set.txt`.
struct ClassRow {
    name: String,
    entry: String,
    cite: String,
    status: String,
    reason: String,
}

fn read_classes() -> Vec<ClassRow> {
    read_table("class-set.txt", 6)
        .into_iter()
        .map(|row| ClassRow {
            name: row[0].clone(),
            entry: row[1].clone(),
            cite: row[3].clone(),
            status: row[4].clone(),
            reason: row[5].clone(),
        })
        .collect()
}

/// One documented hierarchy edge, from `hierarchy-edges.txt`.
struct Edge {
    child: String,
    parent: String,
    line: String,
}

fn read_edges() -> Vec<Edge> {
    read_table("hierarchy-edges.txt", 4)
        .into_iter()
        .map(|row| Edge {
            child: row[0].clone(),
            parent: row[1].clone(),
            line: row[3].clone(),
        })
        .collect()
}

/// One documented (class, method, arm), from `class-methods.txt`.
struct MethodRow {
    class: String,
    method: String,
    arm: String,
    status: String,
    origin: String,
}

fn read_method_rows() -> Vec<MethodRow> {
    read_table("class-methods.txt", 7)
        .into_iter()
        .map(|row| MethodRow {
            class: row[0].clone(),
            method: row[1].clone(),
            arm: row[2].clone(),
            status: row[3].clone(),
            origin: row[5].clone(),
        })
        .collect()
}

/// One concept row's committed assignment: which phase owes it an `agree`,
/// and the negative control that would redden it.
///
/// **The control is a sentence, not a mechanism, and that is deliberate.** A
/// mutation control demonstrates a row going from green to red, and a row
/// that is already red demonstrates nothing -- measured on the commit that
/// creates this table, every row here but the ones listed as agreeing is
/// already red. So each control names the change that would falsify the row
/// and the task that can first run it, which is the task where the row first
/// reads `agree`. Two of them are D50's required controls and are carried in
/// the "Done when" of the tasks that owe them.
struct Concept {
    /// The `provide.xml` section id, which is also the probe's file stem.
    id: &'static str,
    /// The phase that owes this row an `agree`.
    phase: &'static str,
    /// The authority for that assignment.
    authority: &'static str,
    /// The change that would redden the row, and who can first run it.
    control: &'static str,
    /// How many lines of `stdout` this section's probe prints on the oracle.
    ///
    /// **The only committed expectation in this table, and it is a shape
    /// rather than an answer.** The other families derive their bound from
    /// the probe text, because that text is derived too; a concept probe is
    /// hand-written against a page of prose and nothing derives it, so
    /// without a number here a concept row that stopped reaching its
    /// mechanism entirely would read `agree` the moment both interpreters
    /// fell over alike. It can only make a row **fail** -- no line count can
    /// turn a divergence into agreement -- and it holds no byte the oracle
    /// answered.
    ///
    /// It also makes editing a concept probe a visible act: changing what the
    /// program prints means changing this, in the same diff, which is the
    /// nearest thing this family has to the derivation the others get.
    oracle_lines: usize,
}

/// One arm per `provide.xml` section id.
///
/// The assignment is checked against `provide-sections.txt` in both
/// directions by [`concept_rows`]: a section this table has no arm for is
/// structural, and so is an arm naming a section the row set does not carry.
///
/// The authorities are `docs/superpowers/specs/2026-08-17-phase-5-object-model.md`'s
/// mechanism enumeration, which files most of these sections by id, and
/// `docs/superpowers/plans/2026-08-17-phase-5a.md`'s handover, which is what
/// decides a section the enumeration has no row for. **The sections in that
/// second class** -- `xcremet`, `usingcl`, `methna`, `classmeth`, `chi` and
/// `methodsbyclass` -- are the concept-row denominator doing the job the spec
/// says it is there for: the enumeration is a hand-made list with no
/// denominator of its own, and a mechanism it missed surfaces here because
/// the section set is derived.
const CONCEPTS: &[Concept] = &[
    Concept {
        id: "typcla",
        phase: "5a",
        authority: "spec enumeration: the four kinds themselves are 5a, with enforcement \
                    split into its own rows",
        control: "make `::CLASS ... METACLASS` a no-op, so the declared class is an \
                  instance of `.Class` like a plain one -- Task 9. The section's abstract \
                  kind has no control here and is not owed one: this probe asks each class \
                  its `~class~id`, which an abstract class answers exactly as a plain one \
                  does, and abstract enforcement is `abscla`'s row, at 5b because the check \
                  lives inside `~new` -- a discriminator added here would duplicate that row \
                  rather than cover a gap",
        oracle_lines: 4,
    },
    Concept {
        id: "objcla",
        phase: "5b",
        authority: "the section's distinguishing claim is that an object acquires its \
                    class's methods at the time of its creation, which needs `~new`; the \
                    plan's handover puts instance construction in 5b",
        control: "rebuild an existing instance's method lookup from its class on every \
                  send, so a method defined after the instance was created answers -- 5b",
        oracle_lines: 3,
    },
    Concept {
        id: "xmixin",
        phase: "5a",
        authority: "spec enumeration: a mixin's base class, and who may inherit it, is 5a",
        control: "answer `~baseClass` with the class itself rather than with the mixin's \
                  first non-mixin superclass, or merge an INHERIT target's methods in front \
                  of the class's own -- Task 7",
        oracle_lines: 4,
    },
    Concept {
        id: "abscla",
        phase: "5b",
        authority: "spec enumeration: abstract-*class* enforcement is 5b, because the check \
                    lives inside `~new`; the abstract-*method* half is 5a and has no arm of \
                    this section's probe",
        control: "drop `checkAbstract` from the `~new` path, so an abstract class \
                  constructs -- 5b",
        oracle_lines: 1,
    },
    Concept {
        id: "xmetac",
        phase: "5a",
        authority: "spec enumeration: the metaclass graph and its circularity is 5a",
        control: "ignore `METACLASS`, so a class declared with one is still an instance of \
                  `.Class` -- Task 9, which is where this probe's `~id` and `~class` land; \
                  `::CLASS ... METACLASS` itself installs from Task 8",
        oracle_lines: 3,
    },
    Concept {
        id: "xcremet",
        phase: "5a",
        authority: "no row in the spec enumeration; the probe needs `::CLASS` install with a \
                    quoted identifier and a `::METHOD ... CLASS` body, which are Task 18's \
                    and Task 15's",
        control: "install a quoted `::CLASS` identifier under a different name, or give the \
                  new class a superclass other than Object -- Task 18",
        oracle_lines: 3,
    },
    Concept {
        id: "usingcl",
        phase: "5a",
        authority: "spec enumeration: `mixinClass()` and `inherit()` are 5a, and `~subclass` \
                    is the same factory protocol reached by message rather than by directive",
        control: "answer `~subclass` with a class whose superclass is Object rather than the \
                  receiver -- Task 7",
        oracle_lines: 4,
    },
    Concept {
        id: "xscope",
        phase: "5a",
        authority: "spec enumeration: the method dictionary's one entry per scope, and \
                    `Method~scope`, are 5a",
        control: "answer `~method` from the flattened all-scopes dictionary, so an inherited \
                  name answers where the oracle raises -- Task 9",
        oracle_lines: 2,
    },
    Concept {
        id: "usesem",
        phase: "5b",
        authority: "spec enumeration: per-object methods -- SETMETHOD and ENHANCED, and the \
                    scope they create -- are 5b",
        control: "put a `setMethod` definition in the class's dictionary rather than in the \
                  object's own scope, so it reaches the class's other instances -- 5b",
        oracle_lines: 4,
    },
    Concept {
        id: "methna",
        phase: "5a",
        authority: "no row in the spec enumeration; the probe needs `~define` and `~method`, \
                    which the enumeration files under 5a and which are Task 21's and Task 9's",
        control: "stop uppercasing a method name as it is added, so a name defined in lower \
                  case is not found by the message that names it -- Task 21",
        oracle_lines: 4,
    },
    Concept {
        id: "xmeths",
        phase: "5a",
        authority: "spec enumeration: the complete method search order is 5a with the \
                    per-object arm 5b; this probe deliberately omits that arm, which is \
                    `usesem`'s row",
        control: "search a superclass before the class's own dictionary, or drop the \
                  `UNKNOWN` step so a miss goes straight to NOMETHOD -- Task 12",
        oracle_lines: 3,
    },
    Concept {
        id: "unkno",
        phase: "5a",
        authority: "spec enumeration: the complete method search order, of which `UNKNOWN` \
                    is a step, is 5a",
        control: "**delete the `UNKNOWN` step** and record that this row reddens. One of \
                  D50's two required controls; runnable only once the step exists, so \
                  **Task 12** owes it and carries it in its own \"Done when\"",
        oracle_lines: 1,
    },
    Concept {
        id: "chsrod",
        phase: "5a",
        authority: "spec enumeration: changing the search order -- `~m:scope`, `~m:super` -- \
                    is 5a",
        control: "start a scope-override send at the receiver's own class rather than at the \
                  named scope, which makes `self~type:super` recurse or answer the \
                  subclass's method -- Task 10",
        oracle_lines: 1,
    },
    Concept {
        id: "pubpri",
        phase: "5a",
        authority: "spec enumeration: PUBLIC / PACKAGE / PRIVATE as three access scopes is 5a",
        control: "let a PRIVATE method answer a send from outside the object, so the \
                  outside-send line answers instead of raising 97.2 -- Task 13",
        oracle_lines: 2,
    },
    Concept {
        id: "creo",
        phase: "5b",
        authority: "spec enumeration: initialization of an instance -- `~new`, `init`, \
                    `self~init:super` -- is 5b",
        control: "stop calling `init` after `~new` builds the object, or pass `~new`'s \
                  arguments to the wrong `init` -- 5b",
        oracle_lines: 3,
    },
    Concept {
        id: "obdes",
        phase: "5b",
        authority: "spec enumeration: object destruction and uninitialization -- `UNINIT` \
                    and its propagation flags -- is 5b, with the flags carried by 5a's code",
        control: "stop running `UNINIT` before the object's storage is reclaimed, which is \
                  **silent**: measured today the row is rc 0 with empty stderr on both \
                  sides and differs on stdout alone -- 5b",
        oracle_lines: 2,
    },
    Concept {
        id: "reqstr",
        phase: "5a",
        authority: "spec enumeration: required string values -- request(\"STRING\") to \
                    makeString to NOSTRING to defaultName -- is 5a",
        control: "**delete the `makeString` limb** and record that this row reddens (the \
                  second of D50's two required controls), and **answer `makeString` with \
                  the wrong string**, which reddens the same row at rc 0 with empty stderr \
                  on both sides -- that second one is table C's mutation 3. **Task 14** \
                  owes both and carries them in its own \"Done when\"",
        oracle_lines: 6,
    },
    Concept {
        id: "concurr",
        phase: "5a",
        authority: "spec enumeration: GUARDED/UNGUARDED, REPLY and GUARD legality is 5a, \
                    with the semantics Phase 6's",
        control: "refuse `REPLY` inside a method body, or stop returning its expression to \
                  the sender -- Task 16",
        oracle_lines: 3,
    },
    Concept {
        id: "classmeth",
        phase: "5a",
        authority: "no row in the spec enumeration; the probe asks `~id` of every class \
                    reachable as an environment symbol, which is Task 9's protocol over \
                    Task 21's registry",
        control: "drop a class from the registry, so its `~id` line cannot answer -- \
                  Task 9, and again for the deferred classes at Task 21",
        oracle_lines: 31,
    },
    Concept {
        id: "chi",
        phase: "5a",
        authority: "no row in the spec enumeration as a section; the hierarchy list is the \
                    wiring rows' own authority and the probe asks Task 9's `~superClass` \
                    and `~superClasses`",
        control: "answer `~superClass` with the whole superclass list's last element rather \
                  than the class's direct superclass -- Task 9",
        oracle_lines: 8,
    },
    Concept {
        id: "methodsbyclass",
        phase: "5b",
        authority: "no row in the spec enumeration; the probe needs a multidimensional Array \
                    instance for the section's own `matrix[2, 3] = 0` note, and the plan's \
                    handover puts instance construction in 5b",
        control: "route `matrix[2, 3] = 0` to a single-index `[]=`, so the element read back \
                  is not the one written -- 5b",
        oracle_lines: 4,
    },
];

/// The phase that owes every wiring row an `agree`.
///
/// One value rather than a per-row assignment: the spec's class-set criterion
/// replaces the roadmap's "32 classes exist and respond" with exactly this
/// half, and the plan makes Task 9 the task that moves it and Task 21 the one
/// that finishes it for the deferred classes. There is no class in
/// `class-set.txt` the plan files anywhere else.
const WIRING_PHASE: &str = "5a";

/// The phase that owes every method row an `agree`.
///
/// The spec files method rows under 5c, and their instance arm additionally
/// depends on 5b having landed `~new`. They are reported here and never gated
/// by a 5a run.
const METHOD_PHASE: &str = "5c";

/// The probe program for a concept row, as a corpus-relative path.
fn concept_probe(id: &str) -> String {
    format!("{CONCEPT_SUBDIR}/{id}.rex")
}

/// The probe program for a class wiring row.
fn class_probe(name: &str) -> String {
    format!("{CLASS_SUBDIR}/{}.rex", name.to_ascii_lowercase())
}

/// The probe program for a hierarchy edge row.
fn edge_probe(child: &str, parent: &str) -> String {
    format!(
        "{EDGE_SUBDIR}/{}__{}.rex",
        child.to_ascii_lowercase(),
        parent.to_ascii_lowercase()
    )
}

/// The probe program a (class, arm) group of method rows shares.
fn method_probe(class: &str, arm: &str) -> String {
    format!("{METHOD_SUBDIR}/{}__{arm}.rex", class.to_ascii_lowercase())
}

/// The marker a class probe prints the `.environment` entry itself under.
///
/// [`check_entry_present`] reads the oracle's answer back by it, so it is
/// spelled once and both sides of that pair are in this file.
const ENTRY_MARKER: &str = "entry ";

/// The questions a class wiring row asks that **any** `.environment` entry
/// answers, whether it is a class object or an instance.
///
/// Split out from the rest because the split is the row's bound: an `entry`
/// column reading `class` says every question below answers, and one reading
/// `instance` says only these do. See [`class_probe_shape`].
fn class_probe_entry_questions(name: &str) -> Vec<String> {
    vec![
        format!("say '{}' .{name}\n", ENTRY_MARKER.trim_end()),
        format!("say 'class-of-entry' .{name}~class~id\n"),
    ]
}

/// The questions a class wiring row asks that only a **class object** answers.
fn class_probe_class_questions(name: &str) -> Vec<String> {
    vec![
        format!("say 'id' .{name}~id\n"),
        format!("say 'class' .{name}~class\n"),
        format!("say 'superclass' .{name}~superClass\n"),
        format!("say 'superclasses' .{name}~superClasses~makeString('L', ' ')\n"),
        format!("say 'metaclass' .{name}~metaClass\n"),
        format!("say 'isa-class' .{name}~isA(.Class)\n"),
    ]
}

/// The text of a class wiring row's probe.
///
/// **This function is the definition of the program**, not a copy of it: the
/// committed file is compared against this on every run, in both directions,
/// so a probe cannot ask about a class other than its row's.
///
/// One text for every row, whatever its `entry` column says; what the column
/// decides is how many of these lines the entry can answer, which is
/// [`class_probe_shape`]'s job.
fn class_probe_text(name: &str) -> String {
    let mut text = format!(
        "/* Table C wiring row: the .{name} environment entry, asked what it\n\
         \x20  renders as and what its class is -- questions any entry answers --\n\
         \x20  and then the questions the class surface is wired by: ~id, ~class,\n\
         \x20  ~superClass, ~superClasses, ~metaClass and ~isA(.Class). Derived\n\
         \x20  from corpus/docs/class-set.txt by\n\
         \x20  crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on\n\
         \x20  every run and compares it in both directions. */\n"
    );
    for line in class_probe_entry_questions(name) {
        text.push_str(&line);
    }
    for line in class_probe_class_questions(name) {
        text.push_str(&line);
    }
    text
}

/// How many lines a class wiring row's probe prints on the oracle.
///
/// **This is the arm the `entry` column decides, and it must not be zero.**
/// A bound of zero is satisfied by a program that produced nothing at all, so
/// a row filed as an instance entry over a name the build does not have would
/// read `agree` on the two interpreters raising alike -- the same defect the
/// class arm's bound exists to catch, reintroduced through the other side of
/// the same `if`. The instance shape therefore opens with questions any entry
/// answers, and its bound is how many of those there are.
fn class_probe_shape(name: &str, entry: &str) -> OracleShape {
    let entry_questions = class_probe_entry_questions(name).len();
    match entry {
        "class" => OracleShape::Exactly(entry_questions + class_probe_class_questions(name).len()),
        _ => OracleShape::Exactly(entry_questions),
    }
}

/// The `entry` values [`class_probe_shape`] and [`check_edge_endpoints`] know
/// how to read.
///
/// **Read at two decision sites and validated nowhere is how a typo exempts a
/// row instead of reddening it**: an unrecognised value falls to the
/// instance arm's bound and drops out of the edge families' referential
/// check, both silently. Recognised here, so a fourth kind is a structural
/// failure that names the row.
const ENTRY_KINDS: &[&str] = &["class", "instance"];

/// Every concept row's committed line count is one a probe can be evidence
/// for.
///
/// **Zero is not.** A bound of zero lines is satisfied by a program that
/// produced nothing at all, so a concept row committed at zero would read
/// `agree` the moment both interpreters fell over alike -- the same defect
/// the bound exists to catch, through the one arm of it that a committed
/// number rather than a derivation decides.
fn check_concept_line_counts(
    concepts: &[(&Section, &'static Concept)],
    structural: &mut Vec<Structural>,
) {
    for (section, concept) in concepts {
        if concept.oracle_lines == 0 {
            structural.push(Structural {
                subject: format!("CONCEPTS arm {}", section.id),
                detail: "its `oracle_lines` is zero, and a bound of zero lines is met by a \
                         program that produced nothing at all. A concept probe exercises a \
                         mechanism and prints what it observed; one that prints nothing is \
                         not evidence, and the row would read `agree` on both interpreters \
                         failing alike"
                    .to_string(),
            });
        }
    }
}

/// Every class row's `entry` column is a value this file knows how to read.
fn check_entry_kinds(classes: &[ClassRow], structural: &mut Vec<Structural>) {
    for row in classes {
        if !ENTRY_KINDS.contains(&row.entry.as_str()) {
            structural.push(Structural {
                subject: format!("class-set.txt row {}", row.name),
                detail: format!(
                    "its `entry` column reads {:?}, which is not one of {ENTRY_KINDS:?}. \
                     Both the bound on its probe's output and its place in the hierarchy \
                     rows' referential check are decided by that column, so an \
                     unrecognised value exempts the row from each rather than reddening",
                    row.entry
                ),
            });
        }
    }
}

/// Records a structural failure when the oracle does not resolve the row's
/// name to an `.environment` entry at all.
///
/// **What separates a real entry from a name nothing defines**, and why a
/// line count cannot do it alone: an unresolved environment symbol evaluates
/// to its own name as a string, so `.Zork` renders as `.ZORK` and answers
/// `~class~id` with `String` -- one line either way, and both interpreters
/// agree on it. Measured, `.Array` renders as `The Array class` and
/// `.RexxInfo` as `a RexxInfo`. So the discriminator is the rendering, and it
/// is checked here rather than in the probe because the probe cannot fail;
/// it is the row's own claim that this name is an entry.
fn check_entry_present(
    probe: &str,
    name: &str,
    oracle_stdout: &[u8],
    structural: &mut Vec<Structural>,
) -> bool {
    let text = String::from_utf8_lossy(oracle_stdout);
    let answer = text
        .lines()
        .find_map(|line| line.strip_prefix(ENTRY_MARKER));
    let unresolved = format!(".{}", name.to_ascii_uppercase());
    if let Some(answer) = answer
        && answer != unresolved
    {
        return true;
    }
    structural.push(Structural {
        subject: probe.to_string(),
        detail: format!(
            "the oracle does not resolve .{name} to an `.environment` entry: its \
             `{ENTRY_MARKER}` answer is {answer:?}, which is how an unresolved \
             environment symbol renders. The row claims this name is an entry; where the \
             shipped build has no such name, both interpreters raise alike on every \
             question below and the row would read `agree` over a class that does not \
             exist"
        ),
    });
    false
}

/// The text of a hierarchy edge row's probe.
///
/// The documented claim is that the parent is **present in** the child's
/// `~superClasses`, never that it is the whole answer: the hierarchy list
/// gives one edge per class by construction and says so in its own prose. So
/// the probe walks that list, prints whether the parent is on it, prints the
/// whole list beside that for the report, and no line of it asserts the
/// list's length.
///
/// **The walk compares `~id`, not the objects themselves, and that is the
/// fidelity the rest of the row already has.** `~superClasses~makeString`
/// renders each member as its `~id` inside `The ... class`, and the class
/// wiring row for either endpoint asks `~id` directly, so nothing in this
/// family distinguishes two class objects that answer the same `~id`.
/// Asking membership by identity instead would put a **different**
/// mechanism inside a wiring row: whether two class objects are one object,
/// which is `==` sent to a class object. Measured, this crate refuses that
/// one loudly, at rc 120 naming the operator and the operand's shape, and
/// `crates/rexx-exec/src/eval.rs`'s
/// `an_operator_sent_to_an_object_is_loud` is what pins the refusal. A
/// wiring row phrased over it could not be answered until that mechanism
/// lands, which would make this phase's class-set criterion wait on a later
/// phase.
///
/// What the walk therefore cannot see: a `~superClasses` holding some other
/// object whose `~id` is the documented parent's. The `superclasses` line is
/// what stands behind it, the whole list rendered and compared byte for byte
/// against the oracle's.
fn edge_probe_text(child: &str, parent: &str) -> String {
    let mut text = format!(
        "/* Table C wiring row: provide.xml's class hierarchy list indents\n\
         \x20  {child} below {parent}, which claims {parent} is PRESENT IN\n\
         \x20  {child}'s ~superClasses and never that it is the whole of it.\n\
         \x20  Derived from corpus/docs/hierarchy-edges.txt by\n\
         \x20  crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on\n\
         \x20  every run and compares it in both directions. */\n"
    );
    text.push_str(&format!("say 'child' .{child}~id\n"));
    text.push_str(&format!("say 'parent' .{parent}~id\n"));
    text.push_str(&format!("supers = .{child}~superClasses\n"));
    text.push_str("edge = 0\n");
    text.push_str("do at = 1 to supers~items\n");
    text.push_str(&format!(
        "  if supers[at]~id == .{parent}~id then edge = 1\n"
    ));
    text.push_str("end\n");
    // The marker is spelled once, in `DOCUMENTED_EDGE_MARKER`, because
    // `check_documented_edge` reads the oracle's answer back by it.
    text.push_str(&format!(
        "say '{}' edge\n",
        DOCUMENTED_EDGE_MARKER.trim_end()
    ));
    text.push_str(&format!(
        "say 'superclasses' .{child}~superClasses~makeString('L', ' ')\n"
    ));
    text
}

/// The two method names that have no printable spelling, written in the row
/// set as the placeholders the class tables use.
///
/// Measured: the names are the empty string and a single blank, and
/// `.Object~method("")` and `.Object~method(" ")` both answer
/// `The Method class`. A name this function does not recognise is passed
/// through, which is right for every other name in the row set -- they are
/// literal.
fn method_name_literal(name: &str) -> &str {
    match name {
        "(abuttal)" => "",
        "(blank)" => " ",
        other => other,
    }
}

/// The text of a (class, arm) method probe.
///
/// **Which shape this takes comes from the committed `status` column, and the
/// column is a fact about the row set rather than a claim about the oracle.**
/// The class arm asks `.X~hasMethod`, which needs no instance. The instance
/// arm needs one, and `status` says whether the row set offers a way to get
/// it: `covered` is "a bare `~new` constructs an instance on the oracle **or**
/// a construction program is opted in", and `not-covered` says only that no
/// construction program is committed -- its own header adds, in capitals,
/// that it CARRIES NO CLAIM ABOUT THE ORACLE.
///
/// **This derives only the first limb of `covered`.** Every instance-arm
/// program here opens with a bare `~new`, and there is no route to a
/// committed construction program because none exists to route to. The task
/// that commits the first one has to add that route here in the same change;
/// what it must not do is flip a class to `covered` and leave the derived
/// probe on a bare `~new` that raises. What it would see if it did is not a
/// wrong verdict -- [`OracleShape::AllOrNothing`] admits a group that answered
/// nothing -- but a group whose every row reads `agree` on the two sides
/// raising alike, which is the standing hole the report counts on every run.
fn method_probe_text(class: &str, arm: &str, status: &str, reason: &str, names: &[&str]) -> String {
    let mut text = if arm == "class" {
        format!(
            "/* Table C method rows: {class}, class arm -- .{class}~hasMethod(\"M\") for\n\
             \x20  every method corpus/docs/class-methods.txt documents on this arm,\n\
             \x20  one line per row and in the row set's own order. Derived by\n\
             \x20  crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on\n\
             \x20  every run and compares it in both directions. */\n"
        )
    } else if status == "covered" {
        format!(
            "/* Table C method rows: {class}, instance arm -- one line per method\n\
             \x20  corpus/docs/class-methods.txt documents on this arm, asked of a bare\n\
             \x20  ~new instance, in the row set's own order. Derived by\n\
             \x20  crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on\n\
             \x20  every run and compares it in both directions. */\n"
        )
    } else {
        format!(
            "/* Table C method rows: {class}, instance arm. corpus/docs/class-set.txt\n\
             \x20  records this class as `{status}`, because\n\
             \x20  {reason}.\n\
             \x20  So ~new raises and no line below it is reached; the row's evidence\n\
             \x20  is that raise, which is what the row set says there is to have.\n\
             \x20  Derived by crates/rexx-exec/tests/gate_table_c.rs, which re-derives\n\
             \x20  this file on every run and compares it in both directions. */\n"
        )
    };
    if arm == "class" {
        for name in names {
            text.push_str(&format!(
                "say 'class' .{class}~hasMethod(\"{}\")\n",
                method_name_literal(name)
            ));
        }
    } else {
        text.push_str(&format!("o = .{class}~new\n"));
        for name in names {
            text.push_str(&format!(
                "say 'instance' o~hasMethod(\"{}\")\n",
                method_name_literal(name)
            ));
        }
    }
    text
}

/// What the oracle's `stdout` must look like for a row's probe to have asked
/// the question the row is about.
///
/// **The check every family here needs, and the reason it is not optional.**
/// A verdict is a comparison of two sides; it says nothing about whether
/// either side answered. Two interpreters that fail identically at a probe's
/// first instruction agree on all three descriptors, so the row reads
/// `agree` and is counted as satisfied while nothing was asked -- measured, a
/// row naming a class neither interpreter has produces byte-identical `97.1`
/// on both sides, empty `stdout`, identical exit status. `class-set.txt`'s
/// header records `RegularExpression` being removed from the row set by hand
/// for exactly that reason, so the case is not hypothetical: had it stayed,
/// its wiring row would read `agree` today over a class this build does not
/// ship, satisfying a "Done when" no interpreter can meet.
///
/// The bound comes from the probe's own text, never from a recording of what
/// the oracle said. It can only make a row **fail**; nothing here can turn a
/// divergence into agreement.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum OracleShape {
    /// Exactly this many lines, because every `say` the program holds is
    /// reached.
    Exactly(usize),
    /// Every line or none at all, and nothing between: the shape of a program
    /// that constructs something first and asks its questions of the result,
    /// where the construction either answers or raises.
    AllOrNothing(usize),
}

impl OracleShape {
    /// Whether `lines` is a count this shape admits.
    fn admits(self, lines: usize) -> bool {
        match self {
            OracleShape::Exactly(want) => lines == want,
            OracleShape::AllOrNothing(want) => lines == want || lines == 0,
        }
    }

    /// What this shape demands, for the failure message.
    fn describe(self) -> String {
        match self {
            OracleShape::Exactly(want) => format!("exactly {want}"),
            OracleShape::AllOrNothing(want) => format!("{want} or none"),
        }
    }
}

/// How many lines a derived probe prints when every `say` in it is reached.
///
/// Read off the derived text rather than counted by the caller, so the bound
/// and the program cannot drift: they are the same string.
fn derived_say_lines(text: &str) -> usize {
    text.lines().filter(|line| line.starts_with("say ")).count()
}

/// Records a structural failure when the oracle's output is not a shape the
/// probe could have produced while asking its row's question.
///
/// Returns whether the shape held, so a caller with a second check over the
/// same output can skip it rather than report twice about a program that
/// plainly did not run.
fn check_oracle_shape(
    probe: &str,
    subject: &str,
    shape: OracleShape,
    oracle_stdout: &[u8],
    structural: &mut Vec<Structural>,
) -> bool {
    let lines = stdout_lines(oracle_stdout).len();
    if shape.admits(lines) {
        return true;
    }
    structural.push(Structural {
        subject: probe.to_string(),
        detail: format!(
            "the oracle answered {lines} line(s) where this row's probe asks for {}. The \
             row for {subject} therefore has no answer to compare -- and two sides that \
             both fail to answer agree on all three descriptors, so without this the row \
             would read `agree` for a question nobody asked",
            shape.describe()
        ),
    });
    false
}

/// Whether one method row's own `stdout` channel differs.
///
/// The program's `stdout` is shared by every row of its group, so a row's
/// share of it is its own line. Two rules, both of which exist to stop a row
/// reading `agree` off a program that did not answer it:
///
/// * if the two sides' whole `stdout` agrees, no row's line can differ, and
///   that is answered directly rather than by indexing;
/// * otherwise a row agrees only if the two sides produced the **same number
///   of lines** and this row's line matches. Without the count, a crate that
///   printed more lines than the oracle would let the rows before the extra
///   line read `agree` while the program's output as a whole is wrong, and a
///   crate that printed fewer would let the rows past the end compare
///   nothing against nothing.
fn method_row_stdout(oracle: &[Vec<u8>], crate_side: &[Vec<u8>], index: usize) -> bool {
    if oracle.len() != crate_side.len() {
        return true;
    }
    match (oracle.get(index), crate_side.get(index)) {
        (Some(left), Some(right)) => left != right,
        // Reached only where the group expects no line at all -- a class with
        // no instance -- and then the two sides' whole `stdout` has already
        // been compared by the caller.
        (None, None) => false,
        _ => true,
    }
}

/// The marker an edge probe's derived text prints its documented-edge answer
/// under.
const DOCUMENTED_EDGE_MARKER: &str = "documented-edge ";

/// Records a structural failure when the oracle does not confirm the edge the
/// row claims.
///
/// **The row's own claim, asked of the oracle.** A hierarchy row says the
/// documented parent is present in the child's `~superClasses`; if it is not,
/// every instrument in this table still works and the row still reads `agree`
/// whenever the two interpreters answer `0` alike -- a row that has become a
/// statement about neither the book nor the build. What it is here to catch
/// is a row whose claim the shipped interpreter contradicts, which is a
/// finding for a maintainer rather than a verdict for a phase, so it is
/// structural.
///
/// The marker is the one [`edge_probe_text`] writes, in the same file, so the
/// two cannot drift.
fn check_documented_edge(
    probe: &str,
    subject: &str,
    oracle_stdout: &[u8],
    structural: &mut Vec<Structural>,
) {
    let text = String::from_utf8_lossy(oracle_stdout);
    let answer = text
        .lines()
        .find_map(|line| line.strip_prefix(DOCUMENTED_EDGE_MARKER));
    if answer == Some("1") {
        return;
    }
    structural.push(Structural {
        subject: probe.to_string(),
        detail: format!(
            "the oracle does not confirm the documented edge {subject}: its \
             `{DOCUMENTED_EDGE_MARKER}` answer is {answer:?}, not \"1\". The row claims the \
             parent is present in the child's ~superClasses; where the shipped interpreter \
             says otherwise, both sides can answer `0` alike and the row would read `agree` \
             while asserting nothing about either the book or the build"
        ),
    });
}

/// One finished measurement, whatever row class it came from.
struct Measured {
    kind: &'static str,
    /// The row's identity, as the report prints it.
    subject: String,
    /// Everything the row's key does not carry: a citation, a phase, a title.
    detail: String,
    probe: String,
    phase: &'static str,
    /// `None` where the oracle did not answer the row's question at all, so
    /// no comparison of the two sides means anything. **The row stays in the
    /// table**: dropping it would leave one row fewer, a lower gated count,
    /// and a close criterion phrased over that count satisfiable by removing
    /// evidence. It is reported as `unanswered`, is never `agree`, and its
    /// own structural failure is what makes the run red.
    verdict: Option<Verdict>,
    loud: bool,
    refused: Option<String>,
    oracle_exit: i32,
    oracle_stdout: Vec<u8>,
    oracle_stderr: Vec<u8>,
    crate_exit: i32,
    crate_stdout: Vec<u8>,
    crate_stderr: Vec<u8>,
}

/// A probe's two runs -- this crate on both engines, and the oracle -- or the
/// structural failure that stopped them.
struct Ran {
    crate_exit: i32,
    crate_stdout: Vec<u8>,
    crate_stderr: Vec<u8>,
    crate_loud: bool,
    crate_refused: Option<String>,
    oracle_exit: i32,
    oracle_stdout: Vec<u8>,
    oracle_stderr: Vec<u8>,
    differs: Descriptors,
}

/// Runs one probe on both crate engines and on the oracle, or records why it
/// could not be run.
///
/// `on_disk` is the probe subdirectory's own listing, and the guard on it is
/// table D's: a row whose probe was simply deleted has already been reported
/// by the set comparison, and reporting it again here -- with a message
/// stating that the name **is** in the listing -- would be a second, false
/// report of the same fact. What this arm is for is the probe that is listed
/// and still cannot be resolved, a dangling symlink being the case that
/// produces it.
fn run_probe(
    oracle: &support::oracle::Oracle,
    corpus: &Path,
    probe: &str,
    subject: &str,
    on_disk: &BTreeSet<String>,
    structural: &mut Vec<Structural>,
) -> Option<Ran> {
    let abs = match fs::canonicalize(corpus.join(probe)) {
        Ok(abs) => abs,
        Err(error) => {
            if on_disk.contains(probe) {
                structural.push(Structural {
                    subject: probe.to_string(),
                    detail: format!(
                        "the probe is listed in the directory but cannot be resolved: \
                         {error}. The row for {subject} therefore has no program to run, \
                         which is structural -- it is never a row the table drops"
                    ),
                });
            }
            return None;
        }
    };

    let crate_side = run_on_both_engines(&abs);
    let cpp: CppOutcome = oracle.run(&abs);
    if did_not_finish(&cpp) {
        structural.push(Structural {
            subject: probe.to_string(),
            detail: format!(
                "the oracle did not finish: {:?}. Whatever it left on its descriptors is \
                 where it was interrupted, not an answer to compare",
                cpp.termination
            ),
        });
        return None;
    }

    let differs = compare_raw(&crate_side, &cpp);
    Some(Ran {
        crate_exit: wrapped_exit_code(crate_side.exit_code),
        crate_loud: is_loud(&crate_side),
        crate_refused: refused_construct(&crate_side.stderr),
        crate_stdout: crate_side.stdout,
        crate_stderr: crate_side.stderr,
        oracle_exit: cpp.expect_exit_code(),
        oracle_stdout: cpp.stdout,
        oracle_stderr: cpp.stderr,
        differs,
    })
}

/// Compares a probe family's derived path set against the directory's own
/// listing, in both directions, and returns the listing.
///
/// A row with no probe is structural, and so is a probe no row names: an
/// orphan program is one nothing ever runs, which is the shape a deleted row
/// leaves behind.
fn probe_set(
    corpus: &Path,
    subdir: &str,
    expected: &BTreeSet<String>,
    structural: &mut Vec<Structural>,
) -> BTreeSet<String> {
    let dir = corpus.join(subdir);
    let mut on_disk: BTreeSet<String> = BTreeSet::new();
    for entry in fs::read_dir(&dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
    {
        let name = entry
            .unwrap_or_else(|e| panic!("cannot read an entry of {}: {e}", dir.display()))
            .file_name();
        match name.clone().into_string() {
            Ok(name) if name.ends_with(".rex") => {
                on_disk.insert(format!("{subdir}/{name}"));
            }
            Ok(_) => {}
            // A name no row can ever derive, because every derived path is
            // built from ASCII. Dropping it would make the file invisible to
            // the orphan half of the check below -- present in the directory,
            // named by nothing, and never reported.
            Err(_) => structural.push(Structural {
                subject: format!("{subdir}/{}", name.to_string_lossy()),
                detail: "a file in the probe directory whose name is not valid UTF-8. No \
                         row derives such a path, so nothing runs it and the orphan check \
                         cannot name it"
                    .to_string(),
            }),
        }
    }
    for missing in expected.difference(&on_disk) {
        structural.push(Structural {
            subject: missing.clone(),
            detail: "a row has no probe program. A row without one is structural and is \
                     never skipped: the table reddens on the missing program rather than \
                     shrinking to the rows that still have one"
                .to_string(),
        });
    }
    for orphan in on_disk.difference(expected) {
        structural.push(Structural {
            subject: orphan.clone(),
            detail: "a probe program no row names, so nothing runs it".to_string(),
        });
    }
    on_disk
}

/// Compares one derived probe program against its committed file, in both
/// directions.
fn check_probe_text(
    corpus: &Path,
    probe: &str,
    derived: &str,
    on_disk: &BTreeSet<String>,
    structural: &mut Vec<Structural>,
) {
    let path = corpus.join(probe);
    let committed = match fs::read_to_string(&path) {
        Ok(committed) => committed,
        Err(error) => {
            // **Returning quietly here is only right for one of the ways this
            // read fails, and the others leave the file in place.**
            // `read_to_string` fails on invalid UTF-8 and on a permission
            // error as well as on a missing file, and in both of those the
            // program still exists, still runs, and still produces a verdict
            // -- so a silent return turns off the only thing standing between
            // a row's verdict and a program about a different subject.
            //
            // Guarded on the directory listing for the reason `run_probe`'s
            // own arm is: a genuinely missing file is the set check's case and
            // has already been reported once there.
            if on_disk.contains(probe) {
                structural.push(Structural {
                    subject: probe.to_string(),
                    detail: format!(
                        "the probe is listed in the directory but its bytes cannot be \
                         read as text: {error}. The derivation check is what keeps a \
                         probe from asking about a subject other than its row's, and it \
                         cannot run on a file it cannot read -- so this is structural \
                         rather than a check that quietly does not happen"
                    ),
                });
            }
            return;
        }
    };
    if committed != derived {
        // The **first differing line**, not the head of either file. Every
        // program in these families opens with a comment naming its own row,
        // so a bounded excerpt from the top is the same text on both sides
        // and shows a reader nothing at all.
        let (at, left, right) = first_difference(derived, &committed);
        structural.push(Structural {
            subject: probe.to_string(),
            detail: format!(
                "the committed probe is not what its row derives. The derivation in \
                 gate_table_c.rs is the definition of this program, so a difference means \
                 the file asks something other than what its row says it asks -- which a \
                 comparison against the oracle cannot see, because a probe asking about \
                 the wrong subject agrees for the wrong reason.\n    line {at} derived:   \
                 {}\n    line {at} committed: {}",
                excerpt(left.as_bytes()),
                excerpt(right.as_bytes()),
            ),
        });
    }
}

/// The one-based number of the first line at which two texts differ, and that
/// line from each side, **terminator included**. A line present on one side
/// only is reported as the empty string on the other.
///
/// `split_inclusive`, not `lines`. `lines` strips the terminator and a
/// trailing `\r` with it, and cannot see a missing final newline at all, so a
/// file differing from its derivation *only* in line endings is a difference
/// this function has to be able to point at. This repository carries no
/// `.gitattributes`, so a CRLF checkout puts every derived probe through this
/// path at once. The caller escapes both sides, so a `\r` and a missing `\n`
/// are visible in the message.
fn first_difference(left: &str, right: &str) -> (usize, String, String) {
    let mut lefts = left.split_inclusive('\n');
    let mut rights = right.split_inclusive('\n');
    let mut at = 0usize;
    loop {
        at += 1;
        match (lefts.next(), rights.next()) {
            (None, None) => return (at, String::new(), String::new()),
            (a, b) if a != b => {
                return (
                    at,
                    a.unwrap_or_default().to_string(),
                    b.unwrap_or_default().to_string(),
                );
            }
            _ => {}
        }
    }
}

/// The concept rows, with [`CONCEPTS`] checked against the committed section
/// set in both directions.
fn concept_rows<'a>(
    sections: &'a [Section],
    structural: &mut Vec<Structural>,
) -> Vec<(&'a Section, &'static Concept)> {
    let assigned: BTreeSet<&str> = CONCEPTS.iter().map(|concept| concept.id).collect();
    let derived: BTreeSet<&str> = sections.iter().map(|section| section.id.as_str()).collect();
    for missing in derived.difference(&assigned) {
        structural.push(Structural {
            subject: format!("provide.xml section {missing}"),
            detail: "no arm of `CONCEPTS` covers this section, so nothing owes it an \
                     `agree`, no gate could ever read it, and no control is named for it"
                .to_string(),
        });
    }
    for extra in assigned.difference(&derived) {
        structural.push(Structural {
            subject: format!("CONCEPTS arm {extra}"),
            detail: "names a section `provide-sections.txt` does not carry, so this arm's \
                     row does not exist and its control is about nothing"
                .to_string(),
        });
    }
    let mut rows = Vec::new();
    for section in sections {
        if let Some(concept) = CONCEPTS.iter().find(|arm| arm.id == section.id) {
            rows.push((section, concept));
        }
    }
    rows
}

/// The row-set text a derived probe copies into a Rexx block comment, checked
/// for the one sequence that would end the comment early.
///
/// `class-set.txt`'s `reason` is free text and [`method_probe_text`] writes it
/// between `/*` and `*/`. A reason containing `*/` closes the comment where it
/// stands and leaves the rest of the header being parsed as Rexx -- and
/// **nothing else here would notice**: the derivation is the definition, so
/// the committed file matches it, and an instance group whose program now
/// fails to parse answers no lines, which [`OracleShape::AllOrNothing`]
/// admits. The fix belongs in the row file, so this names the row rather than
/// escaping the text and carrying on.
fn check_interpolated_text(classes: &[ClassRow], structural: &mut Vec<Structural>) {
    for row in classes {
        if row.reason.contains("*/") {
            structural.push(Structural {
                subject: format!("class-set.txt row {}", row.name),
                detail: format!(
                    "its `reason` contains `*/`, which ends the Rexx block comment the \
                     derived probe writes it into: {:?}",
                    row.reason
                ),
            });
        }
    }
}

/// Every hierarchy edge names, at both ends, a class the class row set carries
/// as a class object.
///
/// **This is what makes the class wiring row the existence check for the other
/// families.** A class row asks whether `.X` answers at all; an edge row and a
/// method group both name a class, and neither asks that question of its own
/// accord. Tying both back to `class-set.txt` means a name this build does not
/// have is caught once, at its class row, rather than once per family or not
/// at all. The method families are tied by the membership check the grouping
/// does; this is the edge families' half.
fn check_edge_endpoints(classes: &[ClassRow], edges: &[Edge], structural: &mut Vec<Structural>) {
    let class_objects: BTreeSet<&str> = classes
        .iter()
        .filter(|row| row.entry == "class")
        .map(|row| row.name.as_str())
        .collect();
    for edge in edges {
        for (end, name) in [("child", &edge.child), ("parent", &edge.parent)] {
            if !class_objects.contains(name.as_str()) {
                structural.push(Structural {
                    subject: format!("{} <- {}", edge.child, edge.parent),
                    detail: format!(
                        "its {end}, {name}, is not a `class` row of class-set.txt, so no \
                         wiring row asks whether this build has it -- and an edge row \
                         whose endpoints neither interpreter can answer for reads `agree` \
                         on the two of them raising alike"
                    ),
                });
            }
        }
    }
}

/// The `ArgUtil` assertion, carried here as a wiring row.
///
/// **It is the one row here with no verdict channel, and the reason is that a
/// verdict could not see it.** `provide.xml` comments `ArgUtil`'s member out
/// of the class hierarchy list, so an extractor that read the `<member>`s
/// without stripping XML comments first emits an `ArgUtil Object` edge -- and
/// that edge is **true**: `.ArgUtil~superClasses` does hold `The Object
/// class`, measured. So an edge row for it would read `agree`, the end-to-end
/// oracle run would stay at zero failures, and the wrong member set would be
/// invisible to every instrument in this table. What can see it is an
/// assertion over the committed files themselves, which is what this is, and
/// it is structural because there is nothing for a gate mode to relax.
fn argutil_assertion(
    classes: &[ClassRow],
    edges: &[Edge],
    structural: &mut Vec<Structural>,
) -> String {
    const NAME: &str = "ArgUtil";
    let in_class_set = classes.iter().any(|row| row.name == NAME);
    let edge: Vec<String> = edges
        .iter()
        .filter(|edge| edge.child == NAME || edge.parent == NAME)
        .map(|edge| {
            format!(
                "{} <- {} (provide.xml:{})",
                edge.child, edge.parent, edge.line
            )
        })
        .collect();
    if !in_class_set {
        structural.push(Structural {
            subject: "the ArgUtil assertion".to_string(),
            detail: "`ArgUtil` is not in class-set.txt. It is an `.environment` class the \
                     books document nowhere, carried by the row set with the XML comment at \
                     provide.xml:838 as its citation; without it the class denominator is \
                     the documented set rather than the reachable one"
                .to_string(),
        });
    }
    if !edge.is_empty() {
        structural.push(Structural {
            subject: "the ArgUtil assertion".to_string(),
            detail: format!(
                "hierarchy-edges.txt carries an edge naming `ArgUtil`: {}. `provide.xml` \
                 comments that member out, so an edge here means the derivation read the \
                 `<member>`s without blanking XML comments first. **No verdict row can see \
                 this**: the edge it emits is one the oracle confirms, so the table would \
                 be green over a wrong member set",
                edge.join(", ")
            ),
        });
    }
    match (in_class_set, edge.is_empty()) {
        (true, true) => "held: ArgUtil is a class row and emits no hierarchy edge".to_string(),
        _ => "FAILED -- see the structural failures above".to_string(),
    }
}

#[test]
fn concept_and_class_gate_table() {
    let oracle = support::oracle::locate();
    let corpus = corpus_dir();
    let mut structural = Vec::new();

    let sections = read_sections();
    let classes = read_classes();
    let edges = read_edges();
    let method_rows = read_method_rows();

    let concepts = concept_rows(&sections, &mut structural);

    // Method rows, grouped into the (class, arm) programs they share, in the
    // row set's own order on both axes.
    let mut method_groups: Vec<(String, String, String, String, Vec<usize>)> = Vec::new();
    let mut group_index: BTreeMap<(String, String), usize> = BTreeMap::new();
    let class_status: BTreeMap<&str, (&str, &str)> = classes
        .iter()
        .map(|row| {
            (
                row.name.as_str(),
                (row.status.as_str(), row.reason.as_str()),
            )
        })
        .collect();
    for (index, row) in method_rows.iter().enumerate() {
        let key = (row.class.clone(), row.arm.clone());
        let Some(&(status, reason)) = class_status.get(row.class.as_str()) else {
            structural.push(Structural {
                subject: format!("{} {} ({})", row.class, row.method, row.arm),
                detail: "class-methods.txt names a class class-set.txt does not carry, so \
                         the row's probe shape -- which comes from that class's `status` -- \
                         cannot be derived"
                    .to_string(),
            });
            continue;
        };
        assert_eq!(
            row.status.as_str(),
            status,
            "class-methods.txt and class-set.txt disagree on {}'s status",
            row.class
        );
        match group_index.get(&key) {
            Some(&at) => method_groups[at].4.push(index),
            None => {
                group_index.insert(key.clone(), method_groups.len());
                method_groups.push((
                    row.class.clone(),
                    row.arm.clone(),
                    status.to_string(),
                    reason.to_string(),
                    vec![index],
                ));
            }
        }
    }

    // Both directions between every family's derived path set and its
    // directory, before anything runs.
    let concept_paths: BTreeSet<String> = concepts
        .iter()
        .map(|(section, _)| concept_probe(&section.id))
        .collect();
    let class_paths: BTreeSet<String> = classes.iter().map(|row| class_probe(&row.name)).collect();
    let edge_paths: BTreeSet<String> = edges
        .iter()
        .map(|edge| edge_probe(&edge.child, &edge.parent))
        .collect();
    let method_paths: BTreeSet<String> = method_groups
        .iter()
        .map(|(class, arm, _, _, _)| method_probe(class, arm))
        .collect();
    let concept_on_disk = probe_set(&corpus, CONCEPT_SUBDIR, &concept_paths, &mut structural);
    let class_on_disk = probe_set(&corpus, CLASS_SUBDIR, &class_paths, &mut structural);
    let edge_on_disk = probe_set(&corpus, EDGE_SUBDIR, &edge_paths, &mut structural);
    let method_on_disk = probe_set(&corpus, METHOD_SUBDIR, &method_paths, &mut structural);

    // And the derived text against the committed file, for the three families
    // whose programs a row defines.
    for row in &classes {
        check_probe_text(
            &corpus,
            &class_probe(&row.name),
            &class_probe_text(&row.name),
            &class_on_disk,
            &mut structural,
        );
    }
    for edge in &edges {
        check_probe_text(
            &corpus,
            &edge_probe(&edge.child, &edge.parent),
            &edge_probe_text(&edge.child, &edge.parent),
            &edge_on_disk,
            &mut structural,
        );
    }
    for (class, arm, status, reason, rows) in &method_groups {
        let names: Vec<&str> = rows
            .iter()
            .map(|&at| method_rows[at].method.as_str())
            .collect();
        check_probe_text(
            &corpus,
            &method_probe(class, arm),
            &method_probe_text(class, arm, status, reason, &names),
            &method_on_disk,
            &mut structural,
        );
    }

    check_concept_line_counts(&concepts, &mut structural);
    check_entry_kinds(&classes, &mut structural);
    check_interpolated_text(&classes, &mut structural);
    check_edge_endpoints(&classes, &edges, &mut structural);
    let argutil = argutil_assertion(&classes, &edges, &mut structural);

    let mut measured: Vec<Measured> = Vec::new();

    for (section, concept) in &concepts {
        let probe = concept_probe(&section.id);
        let subject = format!("{} ({})", section.id, section.title);
        let Some(ran) = run_probe(
            &oracle,
            &corpus,
            &probe,
            &subject,
            &concept_on_disk,
            &mut structural,
        ) else {
            continue;
        };
        let answered = check_oracle_shape(
            &probe,
            &subject,
            OracleShape::Exactly(concept.oracle_lines),
            &ran.oracle_stdout,
            &mut structural,
        );
        measured.push(Measured {
            kind: "concept",
            subject: format!("{} {}", section.id, section.title),
            detail: format!(
                "depth {} parent {} provide.xml:{}",
                section.depth, section.parent, section.line
            ),
            probe,
            phase: concept.phase,
            verdict: answered.then(|| verdict(ran.differs)),
            loud: ran.crate_loud,
            refused: ran.crate_refused,
            oracle_exit: ran.oracle_exit,
            oracle_stdout: ran.oracle_stdout,
            oracle_stderr: ran.oracle_stderr,
            crate_exit: ran.crate_exit,
            crate_stdout: ran.crate_stdout,
            crate_stderr: ran.crate_stderr,
        });
    }

    for row in &classes {
        let probe = class_probe(&row.name);
        let Some(ran) = run_probe(
            &oracle,
            &corpus,
            &probe,
            &row.name,
            &class_on_disk,
            &mut structural,
        ) else {
            continue;
        };
        // Two checks, and neither subsumes the other. The bound says the
        // program printed what a row filed this way prints; the presence
        // check says the name is an entry at all, which no line count can
        // decide because an unresolved environment symbol answers the
        // opening questions as a string.
        let answered =
            check_oracle_shape(
                &probe,
                &row.name,
                class_probe_shape(&row.name, &row.entry),
                &ran.oracle_stdout,
                &mut structural,
            ) && check_entry_present(&probe, &row.name, &ran.oracle_stdout, &mut structural);
        measured.push(Measured {
            kind: "class",
            subject: row.name.clone(),
            detail: format!("{} {} {}", row.entry, row.status, row.cite),
            probe,
            phase: WIRING_PHASE,
            verdict: answered.then(|| verdict(ran.differs)),
            loud: ran.crate_loud,
            refused: ran.crate_refused,
            oracle_exit: ran.oracle_exit,
            oracle_stdout: ran.oracle_stdout,
            oracle_stderr: ran.oracle_stderr,
            crate_exit: ran.crate_exit,
            crate_stdout: ran.crate_stdout,
            crate_stderr: ran.crate_stderr,
        });
    }

    for edge in &edges {
        let probe = edge_probe(&edge.child, &edge.parent);
        let subject = format!("{} <- {}", edge.child, edge.parent);
        let Some(ran) = run_probe(
            &oracle,
            &corpus,
            &probe,
            &subject,
            &edge_on_disk,
            &mut structural,
        ) else {
            continue;
        };
        let asked = derived_say_lines(&edge_probe_text(&edge.child, &edge.parent));
        let answered = check_oracle_shape(
            &probe,
            &subject,
            OracleShape::Exactly(asked),
            &ran.oracle_stdout,
            &mut structural,
        );
        if answered {
            check_documented_edge(&probe, &subject, &ran.oracle_stdout, &mut structural);
        }
        measured.push(Measured {
            kind: "edge",
            subject,
            detail: format!("provide.xml:{}", edge.line),
            probe,
            phase: WIRING_PHASE,
            verdict: answered.then(|| verdict(ran.differs)),
            loud: ran.crate_loud,
            refused: ran.crate_refused,
            oracle_exit: ran.oracle_exit,
            oracle_stdout: ran.oracle_stdout,
            oracle_stderr: ran.oracle_stderr,
            crate_exit: ran.crate_exit,
            crate_stdout: ran.crate_stdout,
            crate_stderr: ran.crate_stderr,
        });
    }

    // One measurement per method row, from one run per (class, arm).
    let mut method_measured: Vec<(usize, Option<Verdict>, bool, Option<String>)> = Vec::new();
    let mut method_programs: Vec<MethodProgram> = Vec::new();
    for (class, arm, _, _, rows) in &method_groups {
        let probe = method_probe(class, arm);
        let subject = format!("{class} ({arm} arm)");
        let Some(ran) = run_probe(
            &oracle,
            &corpus,
            &probe,
            &subject,
            &method_on_disk,
            &mut structural,
        ) else {
            continue;
        };
        let oracle_lines: Vec<Vec<u8>> = stdout_lines(&ran.oracle_stdout)
            .into_iter()
            .map(<[u8]>::to_vec)
            .collect();
        let crate_lines: Vec<Vec<u8>> = stdout_lines(&ran.crate_stdout)
            .into_iter()
            .map(<[u8]>::to_vec)
            .collect();
        // The class arm asks its questions of the class object directly, so
        // every `say` is reached. The instance arm constructs first, and that
        // construction either answers -- one line per row -- or raises, and
        // no count between the two is reachable. **Neither is read off the
        // row set's `status` column**: that column says whether a
        // construction program is committed, and its own header says
        // `not-covered` "CARRIES NO CLAIM ABOUT THE ORACLE".
        //
        // What this does not check is that the construction *succeeded*: a
        // group whose `~new` raises answers none of its names, and every row
        // of it can read `agree` on the two sides raising alike. The row that
        // catches a class this build does not have at all is the class wiring
        // row, which every method row reaches by the class-set membership
        // check above; what stays uncovered is a class the build has whose
        // constructor stopped constructing, and the report says how many rows
        // sit there on every run.
        let shape = if arm == "class" {
            OracleShape::Exactly(rows.len())
        } else {
            OracleShape::AllOrNothing(rows.len())
        };
        let shape_held =
            check_oracle_shape(&probe, &subject, shape, &ran.oracle_stdout, &mut structural);
        let whole_stdout_differs = ran.differs.stdout;
        for (at, &index) in rows.iter().enumerate() {
            let differs = Descriptors {
                status: ran.differs.status,
                stdout: whole_stdout_differs && method_row_stdout(&oracle_lines, &crate_lines, at),
                stderr: ran.differs.stderr,
            };
            // **A row whose own line is absent from both sides was asked of
            // neither, and has no verdict.** That is the group whose `~new`
            // raised: the program answered none of its names, so `agree`
            // would be a claim about a question nobody put and `diverge`
            // would be a claim about the constructor rather than about this
            // row. The group's own report line still carries the two sides'
            // exit statuses and `stderr`, which is where the constructor's
            // divergence is visible.
            //
            // One side answering and the other not is a real divergence and
            // keeps its verdict.
            let asked = oracle_lines.get(at).is_some() || crate_lines.get(at).is_some();
            // Kept, not dropped, when the shape check failed: a group that
            // vanished would take its rows out of every count, including the
            // gated one.
            method_measured.push((
                index,
                (shape_held && asked).then(|| verdict(differs)),
                ran.crate_loud,
                ran.crate_refused.clone(),
            ));
        }
        method_programs.push(MethodProgram {
            class: class.clone(),
            arm: arm.clone(),
            probe,
            rows: rows.clone(),
            loud: ran.crate_loud,
            refused: ran.crate_refused,
            oracle_exit: ran.oracle_exit,
            oracle_stdout: ran.oracle_stdout,
            oracle_stderr: ran.oracle_stderr,
            crate_exit: ran.crate_exit,
            crate_stdout: ran.crate_stdout,
            crate_stderr: ran.crate_stderr,
        });
    }
    let method_verdicts: BTreeMap<usize, Option<Verdict>> = method_measured
        .iter()
        .map(|(index, verdict, _, _)| (*index, *verdict))
        .collect();

    // ---------------------------------------------------------------- report

    let mut report = Report::new(
        "rexx-exec gate table C -- the concept and class surface \
         (corpus/docs/provide-sections.txt, class-set.txt, hierarchy-edges.txt, \
         class-methods.txt)",
    );
    report.line(&format!(
        "concept rows: {}, one probe each under corpus/{CONCEPT_SUBDIR}/. \
         class wiring rows: {}, under corpus/{CLASS_SUBDIR}/. hierarchy edge rows: {}, \
         under corpus/{EDGE_SUBDIR}/. method rows: {}, sharing {} program(s) under \
         corpus/{METHOD_SUBDIR}/. stderr is compared raw on every row.",
        concepts.len(),
        classes.len(),
        edges.len(),
        method_measured.len(),
        method_programs.len(),
    ));
    report.line("");
    report.line(&format!("the ArgUtil assertion: {argutil}"));

    report.line("");
    report.line("concept rows (verdict / loud / owning phase / section / probe):");
    for row in measured.iter().filter(|row| row.kind == "concept") {
        emit_row(&mut report, row, CONCEPT_SUBDIR);
    }
    report.line("");
    report.line("the negative control each concept row carries, and who can first run it:");
    for (section, concept) in &concepts {
        report.line(&format!(
            "  {:<16} {:<4} {}",
            section.id, concept.phase, concept.control
        ));
        report.line(&format!(
            "      owning-phase authority: {}",
            concept.authority
        ));
    }

    report.line("");
    report.line("class wiring rows (verdict / loud / owning phase / class / probe):");
    for row in measured.iter().filter(|row| row.kind == "class") {
        emit_row(&mut report, row, CLASS_SUBDIR);
    }

    report.line("");
    report.line("hierarchy edge rows (verdict / loud / owning phase / edge / probe):");
    for row in measured.iter().filter(|row| row.kind == "edge") {
        emit_row(&mut report, row, EDGE_SUBDIR);
    }

    report.line("");
    report.line(
        "method rows, by the program their group shares. A group whose rows do not all \
         read the same verdict is listed row by row:",
    );
    for program in &method_programs {
        let mut by_verdict: BTreeMap<Option<Verdict>, usize> = BTreeMap::new();
        for &index in &program.rows {
            *by_verdict.entry(method_verdicts[&index]).or_insert(0) += 1;
        }
        let summary: Vec<String> = by_verdict
            .iter()
            .map(|(verdict, count)| format!("{}={count}", verdict_label(*verdict)))
            .collect();
        report.line(&format!(
            "  {:<28} {:<9} loud={:<3} {:<4} {:<4} row(s): {:<40} {}",
            program.class,
            program.arm,
            if program.loud { "yes" } else { "no" },
            METHOD_PHASE,
            program.rows.len(),
            summary.join(" "),
            program
                .probe
                .strip_prefix(METHOD_SUBDIR)
                .and_then(|rest| rest.strip_prefix('/'))
                .unwrap_or(&program.probe),
        ));
        if by_verdict.len() > 1 {
            for &index in &program.rows {
                report.line(&format!(
                    "      {:<14} {} ({})",
                    verdict_label(method_verdicts[&index]),
                    method_rows[index].method,
                    method_rows[index].origin,
                ));
            }
        }
        if by_verdict
            .keys()
            .any(|verdict| *verdict != Some(Verdict::Agree))
        {
            report.line(&format!(
                "      oracle rc={:<4} out={} err={}",
                program.oracle_exit,
                excerpt(&program.oracle_stdout),
                excerpt(&program.oracle_stderr),
            ));
            report.line(&format!(
                "      crate  rc={:<4} out={} err={}{}",
                program.crate_exit,
                excerpt(&program.crate_stdout),
                excerpt(&program.crate_stderr),
                match &program.refused {
                    Some(construct) => format!("   [{construct}]"),
                    None => String::new(),
                },
            ));
        }
    }

    let mut by_verdict: BTreeMap<&str, usize> = BTreeMap::new();
    let mut by_phase: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    let mut by_construct: BTreeMap<String, usize> = BTreeMap::new();
    let mut loud_rows = 0usize;
    let mut gated: Vec<String> = Vec::new();
    let mut record = |verdict: Option<Verdict>,
                      phase: &'static str,
                      loud: bool,
                      refused: &Option<String>,
                      probe: &str| {
        *by_verdict.entry(verdict_label(verdict)).or_insert(0) += 1;
        let entry = by_phase.entry(phase).or_insert((0, 0));
        entry.0 += 1;
        // `unanswered` is not `agree`, so it counts as open and, on a gated
        // phase, as gated. That is the safe direction: a row nothing could
        // answer must never make the gated count smaller.
        if verdict != Some(Verdict::Agree) {
            entry.1 += 1;
            if verdict_is_gated(phase) {
                gated.push(probe.to_string());
            }
        }
        if loud {
            loud_rows += 1;
        }
        if let Some(construct) = refused {
            *by_construct.entry(construct.clone()).or_insert(0) += 1;
        }
    };
    for row in &measured {
        record(row.verdict, row.phase, row.loud, &row.refused, &row.probe);
    }
    for (index, verdict, loud, refused) in &method_measured {
        record(
            *verdict,
            METHOD_PHASE,
            *loud,
            refused,
            &method_probe(&method_rows[*index].class, &method_rows[*index].arm),
        );
    }

    report.line("");
    report.line("verdicts, over every row of this table:");
    for (label, count) in &by_verdict {
        report.line(&format!("  {label}: {count}"));
    }
    report.line(&format!(
        "  loud (this crate declined rather than answered): {loud_rows}"
    ));

    // Said here, in the summary a reader meets the counts in, rather than
    // only in a report nobody reads at the moment they see green. These are
    // the method rows no documented name was asked of: their group's probe
    // raised before reaching them, so they are `unanswered` above rather than
    // `agree`, and no run of this table can move them until an instance
    // exists.
    //
    // Counted from the verdicts themselves rather than from the oracle's own
    // output, because the sentence and the column have to agree: a group
    // whose oracle raised while this crate answered has rows that **were**
    // asked on one side, and an oracle-only predicate would put them in this
    // line while the table reported them as divergences.
    let unasked = method_measured
        .iter()
        .filter(|(_, verdict, _, _)| verdict.is_none())
        .count();
    report.line(&format!(
        "  method rows no documented name was asked of, on either side, because their \
         group's probe raised first: {unasked}"
    ));

    report.line("");
    report.line("by owning phase -- rows, and rows not yet `agree`:");
    for (phase, (total, open)) in &by_phase {
        report.line(&format!("  {phase}: {total} rows, {open} not yet `agree`"));
    }

    if !by_construct.is_empty() {
        report.line("");
        report.line("what the loud rows are waiting on:");
        for (construct, count) in &by_construct {
            report.line(&format!("  {construct}: {count}"));
        }
    }

    report.line("");
    report.line(&format!(
        "gated by this run: {} row(s) whose owning phase is closing or closed and whose \
         verdict is not `agree`",
        gated.len()
    ));

    gate_tables::emit_uncaptured(&report.finish());

    // Unconditional and first, as in table D: a structural failure is red in
    // every mode, and the gate mode exists to relax a *verdict* comparison
    // rather than to decide whether a run that produced nothing to compare
    // gets noticed.
    assert_no_structural_failures(&structural);

    let named: Vec<&String> = gated.iter().take(20).collect();
    assert!(
        gated.is_empty(),
        "{} row(s) of gate table C owned by a closing or closed phase do not `agree` with \
         the oracle; the first of them are {named:?}. See the report above for each row's \
         three descriptors on both sides.",
        gated.len(),
    );
}

/// One (class, arm) program and the rows that share it.
struct MethodProgram {
    class: String,
    arm: String,
    probe: String,
    rows: Vec<usize>,
    loud: bool,
    refused: Option<String>,
    oracle_exit: i32,
    oracle_stdout: Vec<u8>,
    oracle_stderr: Vec<u8>,
    crate_exit: i32,
    crate_stdout: Vec<u8>,
    crate_stderr: Vec<u8>,
}

/// One row of the report, with the three descriptors on both sides when the
/// row is not `agree`.
fn emit_row(report: &mut Report, row: &Measured, subdir: &str) {
    report.line(&format!(
        "  {:<14} loud={:<3} {:<4} {:<40} {:<34} {}",
        verdict_label(row.verdict),
        if row.loud { "yes" } else { "no" },
        row.phase,
        row.subject,
        row.detail,
        row.probe
            .strip_prefix(subdir)
            .and_then(|rest| rest.strip_prefix('/'))
            .unwrap_or(&row.probe),
    ));
    if row.verdict != Some(Verdict::Agree) {
        report.line(&format!(
            "      oracle rc={:<4} out={} err={}",
            row.oracle_exit,
            excerpt(&row.oracle_stdout),
            excerpt(&row.oracle_stderr),
        ));
        report.line(&format!(
            "      crate  rc={:<4} out={} err={}{}",
            row.crate_exit,
            excerpt(&row.crate_stdout),
            excerpt(&row.crate_stderr),
            match &row.refused {
                Some(construct) => format!("   [{construct}]"),
                None => String::new(),
            },
        ));
    }
}
