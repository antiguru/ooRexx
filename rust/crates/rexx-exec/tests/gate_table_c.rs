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

mod gate_tables;
mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use gate_tables::{
    CLOSED_PHASES, Descriptors, Report, Structural, Verdict, assert_no_structural_failures,
    compare_raw, excerpt, is_loud, refused_construct, run_gate_probe, stdout_lines, verdict,
    verdict_is_gated, verdict_label,
};
use rexx_extract::docs::classes::NO_PROGRAM;
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

/// What a method probe's instance arm binds `o` to.
#[derive(Clone, PartialEq, Eq, Debug)]
enum Construction {
    /// The class is `covered`, and this is the committed expression the
    /// instance arm constructs with, together with the directive the probe
    /// carries below its readbacks for an expression that reads one.
    Constructs {
        program: String,
        directive: Option<String>,
    },
    /// The class is not `covered`, so the instance arm asks a bare `~new`
    /// whose raise is the row's evidence.
    Raises,
}

/// One documented class, from `class-set.txt`.
struct ClassRow {
    name: String,
    entry: String,
    cite: String,
    status: String,
    reason: String,
    construction: Construction,
    /// The row set's `method-owner` column: the phase that owes this class's
    /// method rows an `agree`.
    owner: String,
}

fn read_classes() -> Vec<ClassRow> {
    read_table("class-set.txt", 9)
        .into_iter()
        .map(|row| ClassRow {
            name: row[0].clone(),
            entry: row[1].clone(),
            cite: row[3].clone(),
            status: row[4].clone(),
            reason: row[5].clone(),
            construction: construction_of(&row[0], &row[4], &row[6], &row[7]),
            owner: row[8].clone(),
        })
        .collect()
}

/// The fields `covered` is spread across, read back as one value.
fn construction_of(name: &str, status: &str, program: &str, directive: &str) -> Construction {
    let directive = match directive {
        NO_PROGRAM => None,
        directive => Some(directive.to_string()),
    };
    match (status, program) {
        ("covered", NO_PROGRAM) => panic!(
            "class-set.txt records {name} as `covered` and carries no construction program \
             for it. `covered` is exactly the claim that one is committed"
        ),
        ("covered", program) => Construction::Constructs {
            program: program.to_string(),
            directive,
        },
        (_, NO_PROGRAM) => {
            assert!(
                directive.is_none(),
                "class-set.txt records {name} as `{status}` and carries a construction \
                 directive for it. Only a `covered` row has a route to an instance"
            );
            Construction::Raises
        }
        (status, program) => panic!(
            "class-set.txt records {name} as `{status}` and carries the construction program \
             {program:?} for it. Only a `covered` row has a route to an instance"
        ),
    }
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
struct Concept {
    /// The `provide.xml` section id, which is also the probe's file stem.
    id: &'static str,
    /// The phase that owes this row an `agree`.
    phase: &'static str,
    /// The authority for that assignment.
    authority: &'static str,
    /// The change that would redden the row.
    control: &'static str,
    /// How many lines of `stdout` this section's probe prints on the oracle.
    oracle_lines: usize,
}

/// One arm per `provide.xml` section id.
const CONCEPTS: &[Concept] = &[
    Concept {
        id: "typcla",
        phase: "5a",
        authority: "spec enumeration: the four kinds themselves are 5a, with enforcement \
                    split into its own rows",
        control: "make `::CLASS ... METACLASS` a no-op, so the declared class is an \
                  instance of `.Class` like a plain one. The section's abstract \
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
                  of the class's own",
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
                  `.Class`",
        oracle_lines: 3,
    },
    Concept {
        id: "xcremet",
        phase: "5a",
        authority: "no row in the spec enumeration; the probe needs `::CLASS` install with a \
                    quoted identifier and a `::METHOD ... CLASS` body",
        control: "install a quoted `::CLASS` identifier under a different name, or give the \
                  new class a superclass other than Object",
        oracle_lines: 3,
    },
    Concept {
        id: "usingcl",
        phase: "5a",
        authority: "spec enumeration: `mixinClass()` and `inherit()` are 5a, and `~subclass` \
                    is the same factory protocol reached by message rather than by directive",
        control: "answer `~subclass` with a class whose superclass is Object rather than the \
                  receiver",
        oracle_lines: 4,
    },
    Concept {
        id: "xscope",
        phase: "5a",
        authority: "spec enumeration: the method dictionary's one entry per scope, and \
                    `Method~scope`, are 5a",
        control: "answer `~method` from the flattened all-scopes dictionary, so an inherited \
                  name answers where the oracle raises",
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
                    which the enumeration files under 5a",
        control: "stop uppercasing a method name as it is added, so a name defined in lower \
                  case is not found by the message that names it. Both sites have to go \
                  together, measured: `dispatch.rs`'s own `method_name_pair`, and \
                  `MethodDict::replace_method`. Either one alone leaves this row `agree` \
                  and the corpus untouched",
        oracle_lines: 4,
    },
    Concept {
        id: "xmeths",
        phase: "5a",
        authority: "spec enumeration: the complete method search order is 5a with the \
                    per-object arm 5b; this probe deliberately omits that arm, which is \
                    `usesem`'s row",
        control: "search a superclass before the class's own dictionary, or drop the \
                  `UNKNOWN` step so a miss goes straight to NOMETHOD",
        oracle_lines: 3,
    },
    Concept {
        id: "unkno",
        phase: "5a",
        authority: "spec enumeration: the complete method search order, of which `UNKNOWN` \
                    is a step, is 5a",
        control: "**delete the `UNKNOWN` step** and record that this row reddens. One of \
                  D50's two required controls; runnable only once the step exists",
        oracle_lines: 1,
    },
    Concept {
        id: "chsrod",
        phase: "5a",
        authority: "spec enumeration: changing the search order -- `~m:scope`, `~m:super` -- \
                    is 5a",
        control: "start a scope-override send at the receiver's own class rather than at the \
                  named scope, which makes `self~type:super` recurse or answer the \
                  subclass's method",
        oracle_lines: 1,
    },
    Concept {
        id: "pubpri",
        phase: "5a",
        authority: "spec enumeration: PUBLIC / PACKAGE / PRIVATE as three access scopes is 5a",
        control: "let a PRIVATE method answer a send from outside the object, so the \
                  outside-send line answers instead of raising 97.2",
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
        control: "skip the termination sweep, since under D59 a class object's storage is \
                  never reclaimed and no collection ever reaches one. Measured as run: the \
                  row reddens **silently**, rc 0 with empty stderr on both sides, differing \
                  on stdout alone -- 5b",
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
                  on both sides -- that second one is table C's mutation 3",
        oracle_lines: 6,
    },
    Concept {
        id: "concurr",
        phase: "5a",
        authority: "spec enumeration: GUARDED/UNGUARDED, REPLY and GUARD legality is 5a, \
                    with the semantics Phase 6's",
        control: "refuse `REPLY` inside a method body, or stop returning its expression to \
                  the sender",
        oracle_lines: 3,
    },
    Concept {
        id: "classmeth",
        phase: "5a",
        authority: "no row in the spec enumeration; the probe asks `~id` of every class \
                    reachable as an environment symbol",
        control: "drop a class from the registry, so its `~id` line cannot answer",
        oracle_lines: 31,
    },
    Concept {
        id: "chi",
        phase: "5a",
        authority: "no row in the spec enumeration as a section; the hierarchy list is the \
                    wiring rows' own authority and the probe asks `~superClass` \
                    and `~superClasses`",
        control: "answer `~superClass` with the whole superclass list's last element rather \
                  than the class's direct superclass",
        oracle_lines: 8,
    },
    Concept {
        id: "methodsbyclass",
        phase: "5b",
        authority: "no row in the spec enumeration; the probe needs a multidimensional Array \
                    instance for the section's own `matrix[2, 3] = 0` note, and the plan's \
                    handover puts instance construction in 5b",
        control: "route `matrix[2, 3] = 0` to a single-index `[]=`, so the element read back \
                  is not the one written; or transpose the index mapping, which the `order` \
                  line reads and no write-then-read pair can. Transpose the **offset \
                  accumulation alone**, leaving `multi_dimension_position`'s bounds pass \
                  pairing each subscript with its own dimension: reversing that pairing does \
                  not terminate, because a subscript past the dimension it was mispaired with \
                  re-enters `array_extend_multi` and the interpreter thread overflows its \
                  stack at rc 134, which reads as an infrastructure failure and not as a \
                  reddened row -- 5b",
        oracle_lines: 5,
    },
];

/// The phase that owes every wiring row an `agree`.
const WIRING_PHASE: &str = "5a";

/// The owner of a method row that is never expected to agree, deliberately not
/// spelled like a phase so it can never match [`gate_tables::PHASE_GATE_ENV`]
/// or sit in [`gate_tables::CLOSED_PHASES`].
const NEVER_AGREES: &str = "never-expected-to-agree";

/// The `class-set.txt` status [`method_row_owner`] pairs with
/// [`INSTANCE_ARM`] to read a row as one no run can ever move.
const UNREACHABLE_STATUS: &str = "unreachable";

/// The `class-methods.txt` arm in that pair.
const INSTANCE_ARM: &str = "instance";

/// The phase that owes one method row an `agree`: its class's `method-owner`
/// column, except for a row that can never agree.
fn method_row_owner<'a>(class: &'a ClassRow, arm: &str, status: &str) -> &'a str {
    if status == UNREACHABLE_STATUS && arm == INSTANCE_ARM {
        NEVER_AGREES
    } else {
        &class.owner
    }
}

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
const ENTRY_MARKER: &str = "entry ";

/// The questions a class wiring row asks that **any** `.environment` entry
/// answers, whether it is a class object or an instance.
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
fn class_probe_shape(name: &str, entry: &str) -> OracleShape {
    let entry_questions = class_probe_entry_questions(name).len();
    match entry {
        "class" => OracleShape::Exactly(entry_questions + class_probe_class_questions(name).len()),
        _ => OracleShape::Exactly(entry_questions),
    }
}

/// The `entry` values [`class_probe_shape`] and [`check_edge_endpoints`] know
/// how to read.
const ENTRY_KINDS: &[&str] = &["class", "instance"];

/// Every concept row's committed line count is one a probe can be evidence
/// for.
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
fn method_name_literal(name: &str) -> &str {
    match name {
        "(abuttal)" => "",
        "(blank)" => " ",
        other => other,
    }
}

/// The text of a (class, arm) method probe.
fn method_probe_text(
    class: &str,
    arm: &str,
    status: &str,
    reason: &str,
    construction: &Construction,
    names: &[&str],
) -> String {
    let mut text = match (arm, construction) {
        ("class", _) => format!(
            "/* Table C method rows: {class}, class arm -- .{class}~hasMethod(\"M\") for\n\
             \x20  every method corpus/docs/class-methods.txt documents on this arm,\n\
             \x20  one line per row and in the row set's own order. Derived by\n\
             \x20  crates/rexx-exec/tests/gate_table_c.rs, which re-derives this file on\n\
             \x20  every run and compares it in both directions. */\n"
        ),
        (_, Construction::Constructs { program, directive }) => {
            let carries = match directive {
                None => String::new(),
                Some(directive) => format!(
                    "\x20  `{directive}` below the readbacks is that row's `directives` \
                     field,\n\
                     \x20  which is what the expression reads.\n"
                ),
            };
            format!(
                "/* Table C method rows: {class}, instance arm -- one line per method\n\
                 \x20  corpus/docs/class-methods.txt documents on this arm, asked of the\n\
                 \x20  instance `{program}` answers, in the row set's own order. That\n\
                 \x20  expression is corpus/docs/class-set.txt's committed construction\n\
                 \x20  program for this class, and carrying one is what `covered` claims.\n\
                 {carries}\
                 \x20  Derived by crates/rexx-exec/tests/gate_table_c.rs, which re-derives\n\
                 \x20  this file on every run and compares it in both directions. */\n"
            )
        }
        (_, Construction::Raises) => format!(
            "/* Table C method rows: {class}, instance arm. corpus/docs/class-set.txt\n\
             \x20  records this class as `{status}`, because\n\
             \x20  {reason}.\n\
             \x20  So ~new raises and no line below it is reached; the row's evidence\n\
             \x20  is that raise, which is what the row set says there is to have.\n\
             \x20  Derived by crates/rexx-exec/tests/gate_table_c.rs, which re-derives\n\
             \x20  this file on every run and compares it in both directions. */\n"
        ),
    };
    if arm == "class" {
        for name in names {
            text.push_str(&format!(
                "say 'class' .{class}~hasMethod(\"{}\")\n",
                method_name_literal(name)
            ));
        }
    } else {
        match construction {
            Construction::Constructs { program, .. } => {
                text.push_str(&format!("o = {program}\n"));
            }
            Construction::Raises => text.push_str(&format!("o = .{class}~new\n")),
        }
        for name in names {
            text.push_str(&format!(
                "say 'instance' o~hasMethod(\"{}\")\n",
                method_name_literal(name)
            ));
        }
        if let Construction::Constructs {
            directive: Some(directive),
            ..
        } = construction
        {
            text.push_str(&format!("{directive}\n"));
        }
    }
    text
}

/// What the oracle's `stdout` must look like for a row's probe to have asked
/// the question the row is about.
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
fn derived_say_lines(text: &str) -> usize {
    text.lines().filter(|line| line.starts_with("say ")).count()
}

/// Records a structural failure when the oracle's output is not a shape the
/// probe could have produced while asking its row's question.
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
    phase: String,
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

    let crate_side = run_gate_probe(&abs);
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
/// for the sequences that would end the comment early or split the line.
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
        let Construction::Constructs { program, directive } = &row.construction else {
            continue;
        };
        for (field, text) in [
            ("construction", Some(program)),
            ("directives", directive.as_ref()),
        ] {
            let Some(text) = text else { continue };
            if text.contains("*/") || text.contains('\n') || text.trim() != text.as_str() {
                structural.push(Structural {
                    subject: format!("class-set.txt row {}", row.name),
                    detail: format!(
                        "its `{field}` is not a single trimmed line free of `*/`, and the \
                         derived probe writes it into both a Rexx block comment and its own \
                         line: {text:?}"
                    ),
                });
            }
        }
    }
}

/// Every hierarchy edge names, at both ends, a class the class row set carries
/// as a class object.
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

/// Every owner the method rows carry.
fn method_row_owners() -> BTreeSet<String> {
    let classes = read_classes();
    let by_name: BTreeMap<&str, &ClassRow> =
        classes.iter().map(|row| (row.name.as_str(), row)).collect();
    read_method_rows()
        .iter()
        .filter_map(|row| by_name.get(row.class.as_str()).map(|class| (class, row)))
        .map(|(class, row)| method_row_owner(class, &row.arm, &row.status).to_string())
        .collect()
}

/// Every phase this table owns rows for whose corpus subset file exists is in
/// [`CLOSED_PHASES`].
#[test]
fn every_closed_phase_this_table_owns_rows_for_is_gated() {
    let corpus = corpus_dir();
    let method_owners = method_row_owners();
    // The method family's owners come from a file, so an enumeration that read
    // none of them would leave the filters below iterating the concept and
    // wiring phases alone and still pass.
    assert!(
        !method_owners.is_empty(),
        "no method row named an owner, so this check no longer covers the family it was \
         widened for"
    );
    let owners: BTreeSet<String> = CONCEPTS
        .iter()
        .map(|concept| concept.phase.to_string())
        .chain([WIRING_PHASE.to_string()])
        .chain(method_owners)
        .collect();
    let ungated: Vec<&String> = owners
        .iter()
        .filter(|phase| corpus.join(format!("phase-{phase}.txt")).is_file())
        .filter(|phase| !CLOSED_PHASES.contains(&phase.as_str()))
        .collect();
    assert!(
        ungated.is_empty(),
        "{ungated:?} own rows in gate table C and have a committed corpus subset file, so \
         their programs agree with the oracle, but CLOSED_PHASES does not name them -- a \
         verdict of theirs can move and every gate still exits 0"
    );
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
    let mut method_groups: Vec<MethodGroup> = Vec::new();
    let mut group_index: BTreeMap<(String, String), usize> = BTreeMap::new();
    let class_row: BTreeMap<&str, &ClassRow> =
        classes.iter().map(|row| (row.name.as_str(), row)).collect();
    for (index, row) in method_rows.iter().enumerate() {
        let key = (row.class.clone(), row.arm.clone());
        let Some(class) = class_row.get(row.class.as_str()) else {
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
            row.status, class.status,
            "class-methods.txt and class-set.txt disagree on {}'s status",
            row.class
        );
        match group_index.get(&key) {
            Some(&at) => method_groups[at].rows.push(index),
            None => {
                group_index.insert(key.clone(), method_groups.len());
                method_groups.push(MethodGroup {
                    class: row.class.clone(),
                    arm: row.arm.clone(),
                    status: class.status.clone(),
                    reason: class.reason.clone(),
                    construction: class.construction.clone(),
                    owner: method_row_owner(class, &row.arm, &row.status).to_string(),
                    rows: vec![index],
                });
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
        .map(|group| method_probe(&group.class, &group.arm))
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
    for group in &method_groups {
        let names: Vec<&str> = group
            .rows
            .iter()
            .map(|&at| method_rows[at].method.as_str())
            .collect();
        check_probe_text(
            &corpus,
            &method_probe(&group.class, &group.arm),
            &method_probe_text(
                &group.class,
                &group.arm,
                &group.status,
                &group.reason,
                &group.construction,
                &names,
            ),
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
            phase: concept.phase.to_string(),
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
            phase: WIRING_PHASE.to_string(),
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
            phase: WIRING_PHASE.to_string(),
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
    let mut method_measured: Vec<(usize, Option<Verdict>, bool, Option<String>, String)> =
        Vec::new();
    let mut method_programs: Vec<MethodProgram> = Vec::new();
    for group in &method_groups {
        let MethodGroup {
            class, arm, rows, ..
        } = group;
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
        // every `say` is reached. An instance arm with a committed
        // construction program claims that program answers an instance, so
        // every `say` below it is reached too, and a group whose oracle
        // construction raised is a structural failure -- **this is what makes
        // `covered` mean the probe constructs**, rather than a status a
        // string edit can move. An instance arm without one asks a bare
        // `~new` the row set says nothing about, and no count between all and
        // none is reachable.
        let shape = match (arm.as_str(), &group.construction) {
            ("class", _) | (_, Construction::Constructs { .. }) => OracleShape::Exactly(rows.len()),
            (_, Construction::Raises) => OracleShape::AllOrNothing(rows.len()),
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
            let asked = oracle_lines.get(at).is_some() || crate_lines.get(at).is_some();
            // Kept, not dropped, when the shape check failed: a group that
            // vanished would take its rows out of every count, including the
            // gated one.
            method_measured.push((
                index,
                (shape_held && asked).then(|| verdict(differs)),
                ran.crate_loud,
                ran.crate_refused.clone(),
                group.owner.clone(),
            ));
        }
        method_programs.push(MethodProgram {
            class: class.clone(),
            arm: arm.clone(),
            owner: group.owner.clone(),
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
        .map(|(index, verdict, _, _, _)| (*index, *verdict))
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
    report.line("the negative control each concept row carries:");
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
            "  {:<28} {:<9} loud={:<3} {:<32} {:<4} row(s): {:<40} {}",
            program.class,
            program.arm,
            if program.loud { "yes" } else { "no" },
            program.owner,
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
    let mut by_phase: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut by_construct: BTreeMap<String, usize> = BTreeMap::new();
    let mut loud_rows = 0usize;
    let mut gated: Vec<String> = Vec::new();
    let mut record = |verdict: Option<Verdict>,
                      phase: &str,
                      loud: bool,
                      refused: &Option<String>,
                      probe: &str| {
        *by_verdict.entry(verdict_label(verdict)).or_insert(0) += 1;
        let entry = by_phase.entry(phase.to_string()).or_insert((0, 0));
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
        record(row.verdict, &row.phase, row.loud, &row.refused, &row.probe);
    }
    for (index, verdict, loud, refused, owner) in &method_measured {
        record(
            *verdict,
            owner,
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
    let unasked = method_measured
        .iter()
        .filter(|(_, verdict, _, _, _)| verdict.is_none())
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

/// One (class, arm) group of method rows, with everything its shared program
/// is derived from.
struct MethodGroup {
    class: String,
    arm: String,
    status: String,
    reason: String,
    construction: Construction,
    /// The phase that owes every row of this group an `agree`, from
    /// [`method_row_owner`]. One value per group: the exception it derives is
    /// keyed on the class's status and the arm, and a group is one (class, arm).
    owner: String,
    rows: Vec<usize>,
}

/// One (class, arm) program and the rows that share it.
struct MethodProgram {
    class: String,
    arm: String,
    owner: String,
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
