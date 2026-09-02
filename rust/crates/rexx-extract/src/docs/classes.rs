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

//! Table C's class rows and method rows: the documented class set, and one
//! row per (class, method, arm, status).
//!
//! **How a method is keyed to a class.** Each `cls*` section opens with a
//! generated class table whose `<member>`s name the class's own `mth*`
//! sections and whose `xi:include`s pull in the method sets it receives from a
//! mixin. Neither of the two rules a reader reaches for first is correct
//! alone: keying by the class's own section span loses every included set --
//! measured, `.Array~new~hasMethod("UNION")` is `1` and `UNION` is not among
//! `clsArray`'s own titles -- and keying by an `mth<Class>` name prefix
//! collides, since `mthString*` matches `StringTable`'s set. The table is the
//! book's own answer to the question and is what this module reads.
//!
//! **Where the method's name comes from.** From its `mth*` section's `<title>`
//! in every case but one: a group heading (`Comparison Methods` and its three
//! siblings) documents operator methods that have no title anywhere, and the
//! class table spells those out in the text after the `<xref>` --
//! `mthObjectComparisonMethods` is followed by `= == &lt;> >&lt; \= \==`. A
//! title-keyed row set omits every one of them, and they are real: measured,
//! `"a"~hasMethod("=")` is `1`.

use std::collections::{BTreeMap, BTreeSet};

use crate::docs::xml::{
    Section, blank_comments, decode_predefined, line_of, members, sections, split_revision_marker,
    xincludes,
};

/// The class the book documents but which this build cannot load: it is
/// delivered by `::requires "rxregexp.cls"`, and that file is not on this
/// build's search path -- measured, `43.901 Could not find file
/// "rxregexp.cls" for ::REQUIRES`, rc 213. It is out of the class set
/// entirely, so it has neither a class row nor a method row.
pub const EXCLUDED_CLASS: &str = "RegularExpression";

/// The class documented as a class whose `.environment` entry is an
/// *instance*: `say .RexxInfo` is `a RexxInfo`, and
/// `EndSpecialClassDefinition(RexxInfo)` (`Setup.cpp:1285`) routes the class
/// to `addToSystem`, whose target is not an environment symbol.
pub const INSTANCE_ENTRY_CLASS: &str = "RexxInfo";

/// The `.environment` class the books document nowhere. `provide.xml:838`
/// carries the reason beside the hierarchy list.
pub const UNDOCUMENTED_CLASS: &str = "ArgUtil";

/// Which arm of the readback a method row is asked on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Arm {
    /// `.X~hasMethod("M")`.
    Class,
    /// `.X~new~hasMethod("M")`.
    Instance,
}

impl Arm {
    pub fn as_str(self) -> &'static str {
        match self {
            Arm::Class => "class",
            Arm::Instance => "instance",
        }
    }
}

/// The three row statuses, and what each one may claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    /// A bare `~new` constructs the class, or a construction program is
    /// committed for it.
    Covered,
    /// No construction program is committed. **A statement about this
    /// project's coverage, carrying no claim about the oracle.**
    NotCovered,
    /// The reference itself states that instances come only from native code.
    Unreachable,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Covered => "covered",
            Status::NotCovered => "not-covered",
            Status::Unreachable => "unreachable",
        }
    }
}

/// A class's status together with, for a `covered` class, the Rexx expression
/// its instance arm binds `o` to.
///
/// The expression rides inside the status because it is what the status
/// claims: there is no way to reach [`Coverage::Covered`] without naming the
/// route the derived probe then constructs with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Coverage {
    Covered(String),
    NotCovered,
    Unreachable,
}

impl Coverage {
    pub fn status(&self) -> Status {
        match self {
            Coverage::Covered(_) => Status::Covered,
            Coverage::NotCovered => Status::NotCovered,
            Coverage::Unreachable => Status::Unreachable,
        }
    }

    /// The construction expression, or `None` for a class that has no route.
    pub fn program(&self) -> Option<&str> {
        match self {
            Coverage::Covered(program) => Some(program),
            _ => None,
        }
    }
}

/// What a bare `~new` does on the oracle, per `.environment` class entry.
///
/// **Measured, not derived**, and committed here for the same reason
/// `extract_bif.rs`'s `PER_GROUP` commits its counts: nothing in `oodocs/`
/// says it, and the derivation needs it. The sweep iterates `.environment`,
/// sends `~new` to every entry answering `~isA(.Class)`, and traps `SYNTAX`
/// in a `::routine` -- a label inside the loop would be error 47.2. Run
/// 2026-08-21 against the pinned 5.3 oracle: rc 0, empty stderr, 62 class
/// entries, 38 constructing, 24 raising, none hanging inside a 60 s bound.
///
/// `new` means a bare `~new` returns an instance; anything else is the
/// `condition('O')~code` it raised.
///
/// **Nothing re-measures this**, by decision: the spec rules that a class's
/// membership of the constructible set is not re-checked by the gate tables,
/// and that the review instrument is a diff of the committed row set. What
/// *is* checked on every run is that these names are exactly the class set the
/// books produce, which is what would catch the books and the image drifting
/// apart.
pub const CONSTRUCTION: &[(&str, &str)] = &[
    ("ALARM", "93.901"),
    ("ALARMNOTIFICATION", "new"),
    ("ARGUTIL", "new"),
    ("ARRAY", "new"),
    ("BAG", "new"),
    ("BUFFER", "93.967"),
    ("CASELESSCOLUMNCOMPARATOR", "93.901"),
    ("CASELESSCOMPARATOR", "new"),
    ("CASELESSDESCENDINGCOMPARATOR", "new"),
    ("CIRCULARQUEUE", "93.901"),
    ("CLASS", "93.901"),
    ("COLLECTION", "new"),
    ("COLUMNCOMPARATOR", "93.901"),
    ("COMPARABLE", "new"),
    ("COMPARATOR", "new"),
    ("DATETIME", "new"),
    ("DESCENDINGCOMPARATOR", "new"),
    ("DIRECTORY", "new"),
    ("EVENTSEMAPHORE", "new"),
    ("FILE", "93.901"),
    ("IDENTITYTABLE", "new"),
    ("INPUTOUTPUTSTREAM", "new"),
    ("INPUTSTREAM", "new"),
    ("INVERTINGCOMPARATOR", "93.901"),
    ("LIST", "new"),
    ("MAPCOLLECTION", "new"),
    ("MESSAGE", "93.901"),
    ("MESSAGENOTIFICATION", "new"),
    ("METHOD", "88.901"),
    ("MONITOR", "new"),
    ("MUTABLEBUFFER", "new"),
    ("MUTEXSEMAPHORE", "new"),
    ("NUMERICCOMPARATOR", "new"),
    ("OBJECT", "new"),
    ("ORDERABLE", "new"),
    ("ORDEREDCOLLECTION", "new"),
    ("OUTPUTSTREAM", "new"),
    ("PACKAGE", "88.901"),
    ("POINTER", "93.967"),
    ("PROPERTIES", "new"),
    ("QUEUE", "new"),
    ("RELATION", "new"),
    ("REXXCONTEXT", "93.967"),
    ("REXXQUEUE", "new"),
    ("ROUTINE", "88.901"),
    ("SET", "new"),
    ("SETCOLLECTION", "new"),
    ("SINGLETON", "93.901"),
    ("STACKFRAME", "93.967"),
    ("STEM", "new"),
    ("STREAM", "93.901"),
    ("STREAMSUPPLIER", "97.1"),
    ("STRING", "93.903"),
    ("STRINGTABLE", "new"),
    ("SUPPLIER", "93.903"),
    ("TABLE", "new"),
    ("TICKER", "93.901"),
    ("TIMESPAN", "93.901"),
    ("TRACEOBJECT", "new"),
    ("VALIDATE", "new"),
    ("VARIABLEREFERENCE", "93.967"),
    ("WEAKREFERENCE", "93.903"),
];

/// The committed construction program for a class whose bare `~new` raises:
/// the Rexx expression the instance arm binds `o` to, taken from the book's
/// own syntax for that class's constructor.
///
/// A name here makes the class `covered`, and `covered` means nothing else --
/// the derived probe opens with this expression rather than with a bare
/// `~new`.
pub const CONSTRUCTION_PROGRAMS: &[(&str, &str)] = &[
    (
        "CASELESSCOLUMNCOMPARATOR",
        ".CaselessColumnComparator~new(3, 100)",
    ),
    ("COLUMNCOMPARATOR", ".ColumnComparator~new(3, 100)"),
    (
        "INVERTINGCOMPARATOR",
        ".InvertingComparator~new(.Comparator~new)",
    ),
    ("TIMESPAN", ".TimeSpan~new(1)"),
];

/// What `class-set.txt`'s construction field holds for a class that is not
/// `covered`, and so has no route to an instance.
pub const NO_PROGRAM: &str = "-";

/// A sentence of the reference saying the user cannot construct a class, with
/// the line it is on and the status it grounds.
///
/// Each row is re-read on every run -- [`class_rows`] asserts the sentence is
/// still at the cited line -- so a citation that goes stale reddens instead of
/// being carried.
///
/// **`unreachable` belongs to a row whose sentence says instances come only
/// from native code**, and to no other. A row whose sentence says the user
/// cannot construct one and names a Rexx-level route to obtain one makes a
/// different claim; those are `not-covered`, with a documented and trivial
/// opt-in construction program available and none committed. The `Status` in
/// each row below is which of the two it is.
pub const UNCONSTRUCTIBLE: &[(&str, usize, usize, &str, Status)] = &[
    (
        "Buffer",
        429,
        429,
        "can only be created using the native code application programming interfaces.",
        Status::Unreachable,
    ),
    (
        "Pointer",
        6910,
        6910,
        "can only be created using the native code application programming interfaces.",
        Status::Unreachable,
    ),
    (
        "RexxContext",
        7545,
        7545,
        "They cannot be directly created by the user.",
        Status::NotCovered,
    ),
    (
        "RexxInfo",
        7942,
        7942,
        "other instances cannot be created or copied.",
        Status::NotCovered,
    ),
    (
        "StackFrame",
        9407,
        9407,
        "StackFrame instances cannot be directly created by the user.",
        Status::NotCovered,
    ),
    (
        "VariableReference",
        12556,
        12557,
        "Calling the new method to create a VariableReference instance is not allowed.",
        Status::NotCovered,
    ),
];

/// The file every [`UNCONSTRUCTIBLE`] sentence is in.
pub const UNCONSTRUCTIBLE_BOOK: &str = "utilityclasses.xml";

/// The two `mth*` sections whose title is a bare `new` and which are
/// nonetheless class-side. A short named exception list, not a mechanism:
/// measured, `.Class~hasMethod("NEW")` is `1` while
/// `.Class~new("Foo")~hasMethod("NEW")` is `0`.
const BARE_NEW_CLASS_SIDE: &[&str] = &["mthClassNew", "mthRexxQueueNew"];

/// Titles that name a group of operator methods rather than a method. The
/// operators themselves have no title anywhere and come from the class
/// table's own text.
const GROUP_HEADINGS: &[&str] = &[
    "Comparison Methods",
    "Arithmetic Methods",
    "Concatenation Methods",
    "Logical Methods",
];

/// Placeholders the class tables use for the two concatenation operators that
/// have no printable spelling. Measured on the oracle: the method names are
/// the empty string and a single blank -- `.Object~method("")` and
/// `.Object~method(" ")` both answer `The Method class`.
const OPERATOR_PLACEHOLDERS: &[(&str, &str)] = &[("(abuttal)", ""), ("(blank)", " ")];

/// The literal a generated class table prints for a class that contributes no
/// methods of its own. Both classes carrying it still receive method rows,
/// from the mixin sets their tables `xi:include`.
const NO_OWN_METHODS: &str = "(no class or instance methods)";

/// The `xrefstyle` under which a class-table member displays the target
/// section's own `<title>`, which is what makes the title the method's name.
const TITLE_XREFSTYLE: &str = "select:title";

/// Class-table members whose `xrefstyle` **overrides** the displayed name, and
/// the text it displays instead.
///
/// `template:<text>` replaces the rendered link text outright, so the book's
/// own table shows a name the target section's `<title>` does not carry:
/// `clsDateTime`'s constructor entry displays `new (Inherited Class Method)`
/// and points at `mthDateTimeInit`, whose title is `init`. Measured,
/// **both names answer** -- `.DateTime~hasMethod("NEW")` is `1` and
/// `.DateTime~new~hasMethod("INIT")` is `1` -- so the row set carries both,
/// the title's name and the displayed one.
///
/// **This is a named exception list and [`method_rows`] asserts against it**,
/// in the shape `hierarchy`'s `ArgUtil` assertion already uses. Rule 1 there
/// and this rule here are the same hazard: an extractor misreading the book in
/// a direction nothing downstream contradicts. The both-directions check is
/// structurally blind to both, because a row that was never derived has no arm
/// to disagree about and the committed file agrees perfectly with its absence.
///
/// The style text is part of the key, so an upstream edit to the displayed
/// name reddens rather than being read as the old one.
const TEMPLATE_MEMBERS: &[(&str, &str)] = &[
    ("mthDateTimeInit", "template:new (Inherited Class Method)"),
    ("mthRexxQueueNew", "template:new (Inherited Class Method)"),
    ("mthTimeSpanInit", "template:new (Inherited Class Method)"),
];

/// One book, comment-blanked, with its section index.
pub struct Book {
    pub name: String,
    pub text: String,
    pub sections: Vec<Section>,
}

impl Book {
    pub fn new(name: &str, source: &str) -> Self {
        let text = blank_comments(source);
        let sections = sections(&text);
        Book {
            name: name.to_string(),
            text,
            sections,
        }
    }
}

/// One row of `class-set.txt`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassRow {
    pub name: String,
    pub section: String,
    pub book: String,
    pub line: usize,
    /// `class` for a name whose `.environment` entry is the class object,
    /// `instance` for the one whose entry is an instance.
    pub entry: &'static str,
    pub coverage: Coverage,
    pub reason: String,
}

/// One row of `class-methods.txt`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodRow {
    pub class: String,
    pub method: String,
    pub arm: Arm,
    pub status: Status,
    /// The `mth*` section the name came from.
    pub section: String,
    /// `file:line` of that section.
    pub origin: String,
    pub reason: String,
}

/// Every `cls*` section across the books, in book order then document order.
pub fn class_sections(books: &[Book]) -> Vec<(&Book, &Section)> {
    let mut out = Vec::new();
    for book in books {
        for section in &book.sections {
            if section
                .id
                .as_deref()
                .is_some_and(|id| id.starts_with("cls"))
            {
                out.push((book, section));
            }
        }
    }
    out
}

/// The class set: every `cls*` section, minus [`EXCLUDED_CLASS`], plus
/// [`UNDOCUMENTED_CLASS`].
///
/// `argutil_citation` is the `file:line` of the XML comment that stands in for
/// `ArgUtil`'s missing section, which is the only citation it has.
pub fn class_rows(books: &[Book], argutil_citation: &str) -> Vec<ClassRow> {
    let sentences = unconstructible_index(books);
    let mut out = Vec::new();
    for (book, section) in class_sections(books) {
        let id = section.id.clone().unwrap_or_default();
        let name = id.strip_prefix("cls").unwrap_or(&id).to_string();
        if name == EXCLUDED_CLASS {
            continue;
        }
        let entry = if name == INSTANCE_ENTRY_CLASS {
            "instance"
        } else {
            "class"
        };
        let (coverage, reason) = coverage_of(&name, sentences.get(name.as_str()).copied());
        out.push(ClassRow {
            name,
            section: id,
            book: book.name.clone(),
            line: section.open_line,
            entry,
            coverage,
            reason,
        });
    }
    let (coverage, reason) = coverage_of(UNDOCUMENTED_CLASS, None);
    out.push(ClassRow {
        name: UNDOCUMENTED_CLASS.to_string(),
        section: "-".into(),
        book: "provide.xml".into(),
        line: argutil_citation
            .rsplit(':')
            .next()
            .and_then(|n| n.parse().ok())
            .unwrap_or_else(|| panic!("ArgUtil citation {argutil_citation} names no line")),
        entry: "class",
        coverage,
        reason,
    });
    let derived: BTreeSet<String> = out
        .iter()
        .filter(|r| r.entry == "class")
        .map(|r| r.name.to_ascii_uppercase())
        .collect();
    let measured: BTreeSet<String> = CONSTRUCTION.iter().map(|(n, _)| (*n).to_string()).collect();
    assert_eq!(
        derived, measured,
        "the class set the books produce and the swept .environment class entries disagree; \
         CONSTRUCTION is a measurement of the image and this is where the two would drift apart"
    );
    let named: BTreeSet<String> = out.iter().map(|r| r.name.to_ascii_uppercase()).collect();
    for (class, program) in CONSTRUCTION_PROGRAMS {
        assert!(
            named.contains(*class),
            "CONSTRUCTION_PROGRAMS commits {program} for {class}, which the class set does not \
             carry -- a program for a name no row derives is never reached and never run"
        );
    }
    out
}

/// Re-reads each [`UNCONSTRUCTIBLE`] quotation at the lines it cites, so a
/// citation that has gone stale reddens rather than being carried.
///
/// **A row's span is the lines its own quoted text occupies**, not the lines
/// the book's whole sentence occupies. `Buffer`'s sentence begins on a line
/// above the one cited for it; the row quotes the clause, so the row cites the
/// clause's line. A span written as a range is one whose quotation crosses a
/// line break.
///
/// The span is joined, tag-stripped and whitespace-collapsed before the
/// substring test, because a quotation may cross a line break and may carry
/// inline markup -- `VariableReference`'s carries a `<methodname>`. A raw
/// substring test on one line could then only be written for the part of a
/// quotation that happens to fit, which is a citation to part of a claim.
fn unconstructible_index(
    books: &[Book],
) -> BTreeMap<&'static str, (usize, usize, &'static str, Status)> {
    let book = books
        .iter()
        .find(|b| b.name == UNCONSTRUCTIBLE_BOOK)
        .unwrap_or_else(|| panic!("{UNCONSTRUCTIBLE_BOOK} is not among the books"));
    let lines: Vec<&str> = book.text.lines().collect();
    let mut out = BTreeMap::new();
    for &(class, first, last, sentence, status) in UNCONSTRUCTIBLE {
        let span = lines
            .get(first - 1..last)
            .unwrap_or_else(|| panic!("{UNCONSTRUCTIBLE_BOOK} has no lines {first}-{last}"))
            .join(" ");
        let plain = collapse(&strip_tags(&span));
        assert!(
            plain.contains(sentence),
            "{UNCONSTRUCTIBLE_BOOK}:{first}-{last} no longer reads {sentence:?}; it reads \
             {plain:?}. This citation grounds {class}'s status, and a status grounded in a \
             sentence that moved is grounded in nothing"
        );
        out.insert(class, (first, last, sentence, status));
    }
    out
}

/// Everything outside a `<...>`, with the tags replaced by nothing.
fn strip_tags(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find('<') {
        out.push_str(&rest[..at]);
        rest = match rest[at..].find('>') {
            Some(end) => &rest[at + end + 1..],
            None => "",
        };
    }
    out.push_str(rest);
    out
}

fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A class's coverage and the sentence `class-set.txt` states it in.
///
/// A committed [`CONSTRUCTION_PROGRAMS`] entry is what makes a class
/// `covered` where a bare `~new` raises; it may not be committed for a class
/// whose sentence makes it [`Status::Unreachable`], which is D73's guard.
fn coverage_of(
    name: &str,
    sentence: Option<(usize, usize, &'static str, Status)>,
) -> (Coverage, String) {
    let upper = name.to_ascii_uppercase();
    let construction = CONSTRUCTION
        .iter()
        .find(|(n, _)| *n == upper)
        .map(|(_, outcome)| *outcome);
    let program = CONSTRUCTION_PROGRAMS
        .iter()
        .find(|(n, _)| *n == upper)
        .map(|(_, program)| *program);
    assert!(
        program.is_none() || matches!(construction, Some(code) if code != "new"),
        "{name} has a committed construction program and a bare ~new that does not raise on \
         the oracle, so the program is a route to nothing the bare ~new does not already reach"
    );
    if let Some((first, last, text, status)) = sentence {
        let at = if first == last {
            format!("{first}")
        } else {
            format!("{first}-{last}")
        };
        let cited = format!("{UNCONSTRUCTIBLE_BOOK}:{at} \"{text}\"");
        if status == Status::Unreachable {
            assert!(
                program.is_none(),
                "{name} is unreachable and has a committed construction program"
            );
            return (
                Coverage::Unreachable,
                format!("the reference says instances come only from native code -- {cited}"),
            );
        }
        return match program {
            Some(program) => (
                Coverage::Covered(program.to_string()),
                format!(
                    "a committed construction program answers an instance; the reference \
                     says the user cannot create one and names a Rexx-level route instead \
                     -- {cited}"
                ),
            ),
            None => (
                Coverage::NotCovered,
                format!(
                    "no construction program is committed; the reference says the user \
                     cannot create one and names a Rexx-level route instead -- {cited}"
                ),
            ),
        };
    }
    if let Some(program) = program {
        let code = construction.unwrap_or_default();
        return (
            Coverage::Covered(program.to_string()),
            format!(
                "a committed construction program answers an instance; a bare ~new raises \
                 {code} on the oracle"
            ),
        );
    }
    match construction {
        Some("new") => (
            Coverage::Covered(format!(".{name}~new")),
            "a bare ~new constructs an instance on the oracle".into(),
        ),
        Some(code) => (
            Coverage::NotCovered,
            format!(
                "no construction program is committed; a bare ~new raises {code} on the oracle"
            ),
        ),
        None => (
            Coverage::NotCovered,
            "no construction program is committed; the name is not an .environment class entry"
                .into(),
        ),
    }
}

/// The `mth*` section of every id the class tables name, keyed by id.
fn method_sections(books: &[Book]) -> BTreeMap<String, (String, usize, String)> {
    let mut out = BTreeMap::new();
    for book in books {
        for section in &book.sections {
            let Some(id) = section.id.as_deref() else {
                continue;
            };
            if !id.starts_with("mth") {
                continue;
            }
            assert!(
                section.title_is_immediate,
                "{}:{} -- {id} has no title of its own",
                book.name, section.open_line
            );
            let previous = out.insert(
                id.to_string(),
                (section.title.clone(), section.open_line, book.name.clone()),
            );
            assert!(previous.is_none(), "{id} is defined in two places");
        }
    }
    out
}

/// The method rows, one per (class, method, arm).
///
/// `includes` maps a `*classmethods.xml` file name to its text.
pub fn method_rows(
    books: &[Book],
    includes: &BTreeMap<String, String>,
    class_rows: &[ClassRow],
) -> Vec<MethodRow> {
    let titles = method_sections(books);
    let status: BTreeMap<&str, (Status, &str)> = class_rows
        .iter()
        .map(|r| (r.name.as_str(), (r.coverage.status(), r.reason.as_str())))
        .collect();
    let mut out = Vec::new();
    for (book, section) in class_sections(books) {
        let id = section.id.clone().unwrap_or_default();
        let name = id.strip_prefix("cls").unwrap_or(&id).to_string();
        if name == EXCLUDED_CLASS {
            continue;
        }
        let head = section.head(&book.text);
        let mut listed: Vec<Listed> = Vec::new();
        // `head` is a slice of the book, so a member's line inside it counts
        // from the section's opening tag rather than from the file.
        let head_base = line_of(&book.text, section.body_start) - 1;
        listed.extend(listed_members(head, &book.name, head_base));
        for (href, line) in xincludes(head) {
            if !href.ends_with("classmethods.xml") {
                continue;
            }
            let text = includes.get(&href).unwrap_or_else(|| {
                panic!("{}:{line} includes {href}, which was not read", book.name)
            });
            // An include is read whole, so its member lines are already the
            // file's.
            listed.extend(listed_members(text, &href, 0));
        }
        assert!(
            !listed.is_empty() || head.contains(NO_OWN_METHODS),
            "{}:{} -- {id}'s class table names no methods and does not say {NO_OWN_METHODS:?}",
            book.name,
            section.open_line
        );
        // A class table can name the same `mth*` section twice -- a class that
        // includes both `collectionclassmethods.xml` and a set that repeats
        // one of its entries. Rows are per (class, method), so the second
        // sighting is the same row and not a duplicate to emit.
        let mut seen: BTreeSet<(String, Arm)> = BTreeSet::new();
        let (row_status, reason) = status
            .get(name.as_str())
            .copied()
            .unwrap_or_else(|| panic!("{name} has a method set and no class row"));
        for entry in listed {
            let Listed {
                target,
                xrefstyle,
                trailing,
                source,
                line: member_line,
            } = entry;
            let (title, line, file) = titles
                .get(&target)
                .unwrap_or_else(|| panic!("{source} names {target}, which has no <section>"))
                .clone();
            for (displayed, from_xrefstyle) in
                displayed_names(&target, xrefstyle.as_deref(), &title, &source)
            {
                // A row cites where its own name came from. Normally that is
                // the `mth*` section whose title it is; for a name the class
                // table's `xrefstyle` displays instead, following the
                // section's line would land on a title that does not carry
                // the name at all.
                let origin = if from_xrefstyle {
                    format!("{source}:{member_line}")
                } else {
                    format!("{file}:{line}")
                };
                for (method, arm) in names_of(&target, &displayed, &trailing, &source) {
                    if !seen.insert((method.clone(), arm)) {
                        continue;
                    }
                    out.push(MethodRow {
                        class: name.clone(),
                        method,
                        arm,
                        status: row_status,
                        section: target.clone(),
                        origin: origin.clone(),
                        reason: reason.to_string(),
                    });
                }
            }
        }
    }
    out
}

/// One `<member>` of a class table, as the row derivation needs it.
struct Listed {
    /// The `mth*` section the member links to.
    target: String,
    /// The `<xref>`'s `xrefstyle`, which decides what the book displays.
    xrefstyle: Option<String>,
    /// Text after the `<xref/>`, where a group heading spells out its
    /// operators.
    trailing: String,
    /// The file the member was read from: a book, or the `*classmethods.xml`
    /// a class table includes.
    source: String,
    /// The member's own line in that file. This is the citation a row carries
    /// when the member's `xrefstyle` is where its name came from.
    line: usize,
}

fn listed_members(text: &str, source: &str, base_line: usize) -> Vec<Listed> {
    members(text)
        .into_iter()
        .filter(|m| m.linkend.as_deref().is_some_and(|t| t.starts_with("mth")))
        .map(|m| Listed {
            target: m.linkend.unwrap_or_default(),
            xrefstyle: m.xrefstyle,
            trailing: m.trailing,
            source: source.to_string(),
            line: base_line + m.line,
        })
        .collect()
}

/// Every name the book **displays** for one class-table member, each with
/// whether it came from the `xrefstyle` rather than from the target's title.
///
/// Normally exactly one, the target section's own `<title>`. A member whose
/// `xrefstyle` overrides the displayed text yields two: the title's name and
/// the displayed one, because measured on the oracle both answer.
///
/// **Panics on any other `xrefstyle`**, which is the guard: a `template:`
/// member added upstream, or one whose displayed text changes, reddens instead
/// of being read as its target's title. Nothing downstream can see that
/// mistake -- the row it should have produced was never derived, so it has no
/// arm to disagree about and the both-directions check compares the extractor
/// with itself.
fn displayed_names(
    section: &str,
    xrefstyle: Option<&str>,
    title: &str,
    source: &str,
) -> Vec<(String, bool)> {
    let style = xrefstyle.unwrap_or_else(|| {
        panic!("{source} names {section} in a <member> whose <xref> carries no xrefstyle")
    });
    if style.starts_with(TITLE_XREFSTYLE) {
        return vec![(title.to_string(), false)];
    }
    let template = TEMPLATE_MEMBERS
        .iter()
        .find(|(id, expected)| *id == section && *expected == style)
        .map(|(_, expected)| expected.trim_start_matches("template:"))
        .unwrap_or_else(|| {
            panic!(
                "{source} names {section} with xrefstyle {style:?}, which neither displays the \
                 target section's own title nor is a member this extractor has decided about. \
                 The book displays something other than {title:?} here, so deriving the name from \
                 the title would emit a name the book does not show and silently omit the one it \
                 does"
            )
        });
    vec![(title.to_string(), false), (template.to_string(), true)]
}

/// The method names one class-table member yields, and the arm each takes.
///
/// Every hard case the spec names is a branch here, in the order they have to
/// be tried.
fn names_of(section: &str, title: &str, trailing: &str, source: &str) -> Vec<(String, Arm)> {
    // The revision-marker entity is glued to the name with no space, and no
    // DTD-resolving escape is available -- see `xml`'s own doc.
    let (_, title) = split_revision_marker(title);
    let (title, marker) = split_marker(title);

    // A group heading is not a method name. The operators it documents have no
    // title anywhere; the class table's own text is where they are written.
    if GROUP_HEADINGS.contains(&title) {
        assert!(
            !trailing.is_empty(),
            "{source} names the group heading {section} with no operator list after the <xref>, \
             so this class's operator methods would be silently absent"
        );
        return decode_predefined(trailing)
            .split_whitespace()
            .map(|token| {
                let name = OPERATOR_PLACEHOLDERS
                    .iter()
                    .find(|(placeholder, _)| *placeholder == token)
                    .map_or(token, |(_, actual)| *actual);
                (name.to_string(), Arm::Instance)
            })
            .collect();
    }

    // Two `New` sections carry a bare `new` title and are class-side anyway.
    let arm = if BARE_NEW_CLASS_SIDE.contains(&section) {
        Arm::Class
    } else {
        marker.and_then(|m| m.arm).unwrap_or(Arm::Instance)
    };

    // `center/centre`, `canceled/cancelled`, `delete / delStr`: one section,
    // two names, and the oracle answers both.
    title
        .split('/')
        .map(|name| (name.trim().to_string(), arm))
        .filter(|(name, _)| !name.is_empty())
        .collect()
}

struct Marker {
    arm: Option<Arm>,
}

/// Every parenthetical a `mth*` title ends with, lowercased, and the arm it
/// carries.
///
/// The suffix's case wobbles -- `Class method` beside `Class Method` -- so the
/// match is case-insensitive. `(Inherited Class Method)` occurs once and is
/// class-side like the rest of its family. **`(inline if)` is not a kind
/// marker**: it says nothing about the arm and is a gloss on the `?` operator.
/// It still has to come off the name, because `?` is what the oracle answers
/// and `? (inline if)` is what a title-verbatim extractor would ask for.
const TITLE_PARENTHETICALS: &[(&str, Option<Arm>)] = &[
    ("class method", Some(Arm::Class)),
    ("inherited class method", Some(Arm::Class)),
    ("abstract method", Some(Arm::Instance)),
    ("private method", Some(Arm::Instance)),
    ("attribute", Some(Arm::Instance)),
    ("inline if", None),
];

/// Splits a trailing `(...)` off a title and reads the arm it carries, if any.
///
/// Panics on a parenthetical this list does not know, rather than guessing:
/// an unknown one is either a new kind marker whose arm nobody has decided or
/// a gloss that has to come off the name, and both are wrong to absorb.
fn split_marker(title: &str) -> (&str, Option<Marker>) {
    let trimmed = title.trim_end();
    let Some(open) = trimmed.rfind('(') else {
        return (trimmed, None);
    };
    if !trimmed.ends_with(')') {
        return (trimmed, None);
    }
    let inside = trimmed[open + 1..trimmed.len() - 1].to_ascii_lowercase();
    let arm = TITLE_PARENTHETICALS
        .iter()
        .find(|(text, _)| *text == inside)
        .map(|(_, arm)| *arm)
        .unwrap_or_else(|| {
            panic!(
                "the title {title:?} ends with a parenthetical this extractor does not know. \
                 Either it is a kind marker whose arm nobody has decided, or it is a gloss that \
                 has to come off the name; guessing gives a name the oracle has on neither arm"
            )
        });
    (trimmed[..open].trim_end(), Some(Marker { arm }))
}

/// Every `mth*` section the class tables never name, which is the method-row
/// denominator's other half: each such section is either a row or a named
/// exception.
pub fn unreferenced_sections(books: &[Book], rows: &[MethodRow]) -> Vec<(String, String, usize)> {
    let referenced: BTreeSet<&str> = rows.iter().map(|r| r.section.as_str()).collect();
    let mut out = Vec::new();
    for book in books {
        for section in &book.sections {
            let Some(id) = section.id.as_deref() else {
                continue;
            };
            if id.starts_with("mth") && !referenced.contains(id) {
                out.push((id.to_string(), book.name.clone(), section.open_line));
            }
        }
    }
    out
}

/// Every class in the class set that has no method row, with the reason its
/// method set is empty.
///
/// Derived rather than described, because the alternative is a reader with the
/// two committed files and no report -- which is every reader until the plan
/// closes -- finding a class with zero method rows and no stated reason, and
/// having to decide whether it is deliberate or a row that went missing.
pub fn classes_without_method_rows(
    class_rows: &[ClassRow],
    method_rows: &[MethodRow],
) -> Vec<(String, String)> {
    let with_rows: BTreeSet<&str> = method_rows.iter().map(|r| r.class.as_str()).collect();
    class_rows
        .iter()
        .filter(|r| !with_rows.contains(r.name.as_str()))
        .map(|r| {
            let why = if r.section == "-" {
                format!(
                    "the books document it nowhere; its only citation is {}:{}",
                    r.book, r.line
                )
            } else {
                format!("{} names no methods, {}:{}", r.section, r.book, r.line)
            };
            (r.name.clone(), why)
        })
        .collect()
}

/// `name<TAB>entry<TAB>section<TAB>book:line<TAB>status<TAB>reason`.
pub fn class_set_rows(rows: &[ClassRow]) -> Vec<String> {
    rows.iter()
        .map(|r| {
            format!(
                "{}\t{}\t{}\t{}:{}\t{}\t{}\t{}",
                r.name,
                r.entry,
                r.section,
                r.book,
                r.line,
                r.coverage.status().as_str(),
                r.reason,
                r.coverage.program().unwrap_or(NO_PROGRAM)
            )
        })
        .collect()
}

/// `class<TAB>method<TAB>arm<TAB>status<TAB>section<TAB>origin<TAB>reason`.
pub fn method_set_rows(rows: &[MethodRow]) -> Vec<String> {
    rows.iter()
        .map(|r| {
            format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                r.class,
                render_method(&r.method),
                r.arm.as_str(),
                r.status.as_str(),
                r.section,
                r.origin,
                r.reason
            )
        })
        .collect()
}

/// A method name for the file. The two concatenation operators are the empty
/// string and a single blank, neither of which survives a tab-separated line,
/// so they are written as the placeholder the book uses and read back the same
/// way.
pub fn render_method(name: &str) -> String {
    match OPERATOR_PLACEHOLDERS
        .iter()
        .find(|(_, actual)| *actual == name)
    {
        Some((placeholder, _)) => (*placeholder).to_string(),
        None => name.to_string(),
    }
}

/// The inverse of [`render_method`], for a harness reading the committed file.
pub fn parse_method(field: &str) -> String {
    match OPERATOR_PLACEHOLDERS
        .iter()
        .find(|(placeholder, _)| *placeholder == field)
    {
        Some((_, actual)) => (*actual).to_string(),
        None => field.to_string(),
    }
}

/// The header `class-set.txt` carries, with its D56 stamp.
pub fn class_set_header(stamp: &str, argutil_citation: &str) -> Vec<String> {
    vec![
        "Table C's class row set: every <section id=\"cls...\"> across".into(),
        "fundclasses.xml, collclasses.xml, utilityclasses.xml and".into(),
        "streamclasses.xml.".into(),
        String::new(),
        "One `name<TAB>entry<TAB>section<TAB>book:line<TAB>status<TAB>reason<TAB>construction`"
            .into(),
        "per line, in book order then document order.".into(),
        String::new(),
        format!(
            "Three departures from the sections, each named rather than \
             mechanical. {EXCLUDED_CLASS} is out of the set: it is delivered by \
             `::requires \"rxregexp.cls\"` and that file is not on this build's \
             search path, measured 43.901 rc 213. {INSTANCE_ENTRY_CLASS} is an \
             `instance` row rather than a `class` row: its .environment entry \
             is an instance, not the class object. {UNDOCUMENTED_CLASS} is \
             added: it is an .environment class the books document nowhere, and \
             its only citation is the XML comment at {argutil_citation}."
        ),
        String::new(),
        "`status` is the spec's three row statuses. `covered` is a class a bare".into(),
        "~new constructs, or one opted in with a committed construction".into(),
        "program. `not-covered` says no construction program is committed and".into(),
        "CARRIES NO CLAIM ABOUT THE ORACLE. `unreachable` is grounded in a".into(),
        "sentence of the reference itself and belongs to the classes whose".into(),
        "sentence says instances come only from native code; each such".into(),
        "sentence is re-read at the line it cites on every run.".into(),
        String::new(),
        format!(
            "`construction` is the Rexx expression a method row's instance arm \
             binds `o` to, and it is what `covered` claims: every `covered` row \
             carries one and every other row carries `{NO_PROGRAM}`. A class \
             whose bare ~new constructs carries that bare ~new; one opted in by \
             CONSTRUCTION_PROGRAMS carries the book's own constructor syntax. \
             gate_table_c.rs derives the instance-arm probe from this field, so \
             a `covered` status and the program the probe runs cannot say \
             different things."
        ),
        String::new(),
        "Derived by `cargo run -p rexx-extract --bin rexx-extract-docs`, whose".into(),
        "module is `src/docs/classes.rs`. Re-derived and compared in both".into(),
        "directions by `tests/extract_docs.rs`.".into(),
        String::new(),
        stamp.to_string(),
    ]
}

/// The header `class-methods.txt` carries, with its D56 stamp.
pub fn class_methods_header(stamp: &str) -> Vec<String> {
    vec![
        "Table C's method row set: one row per (class, method, arm, status).".into(),
        String::new(),
        "One `class<TAB>method<TAB>arm<TAB>status<TAB>section<TAB>origin<TAB>reason`".into(),
        "per line. `section` is the mth* section that documents the method, and".into(),
        "`origin` is the file:line WHERE THE NAME CAME FROM -- so every row".into(),
        "cites the book, and following the citation lands on the name.".into(),
        String::new(),
        "Those two are usually the same place and for a few rows they are not,".into(),
        "because the book does not always display a section's own title. A".into(),
        "class table's `<member>` links to its `mth*` section through an".into(),
        "`<xref>` whose `xrefstyle` decides the rendered text: `select:title`".into(),
        "renders that section's `<title>`, and `template:<text>` replaces it".into(),
        "with `<text>` outright. Where a member overrides the title, the row".into(),
        "set carries BOTH names -- the title's and the displayed one -- because".into(),
        "measured on the oracle both answer, and the row whose name came from".into(),
        "the override cites the member rather than the section.".into(),
        String::new(),
        "clsDateTime's constructor entry is the worked case: the member at".into(),
        "utilityclasses.xml:1281 displays `new (Inherited Class Method)` and".into(),
        "points at mthDateTimeInit, whose title is `init`. So `DateTime init`".into(),
        "cites the section at :1775 and `DateTime new` cites the member at".into(),
        ":1281. Measured, .DateTime~hasMethod(\"NEW\") is 1 with the instance arm".into(),
        "0, and .DateTime~hasMethod(\"INIT\") and .DateTime~new~hasMethod(\"INIT\")".into(),
        "are both 1. clsTimeSpan is the same shape at :10492.".into(),
        String::new(),
        "An xrefstyle that neither renders the title nor is one the extractor".into(),
        "has decided about is a hard error, not a fallback to the title: the".into(),
        "row it should have produced would never be derived, and a check".into(),
        "comparing the derivation against this file cannot see a row that is".into(),
        "absent from both sides.".into(),
        String::new(),
        "`arm` is which readback instrument the row is asked on -- `class` is".into(),
        "`.X~hasMethod(\"M\")` and `instance` is `.X~new~hasMethod(\"M\")`.".into(),
        "Neither instrument answers both questions: measured,".into(),
        "`.Array~hasMethod(\"OF\")` is 1 and `.Array~new~hasMethod(\"OF\")` is 0.".into(),
        String::new(),
        "The completeness evidence over this file is that every `covered` row".into(),
        "answers on ONE ARM OR THE OTHER on the oracle. The scope to `covered`".into(),
        "is what makes it achievable: a `not-covered` class has no instance, so".into(),
        "measured, all three of `.Alarm~hasMethod(\"CANCEL\")`,".into(),
        "`(\"TRIGGERED\")` and `(\"SCHEDULEDTIME\")` are 0 while `.Alarm~new`".into(),
        "raises 93.901.".into(),
        String::new(),
        "The two concatenation operators have no printable spelling, so they \
         are written as the placeholders the class tables use. Measured, the \
         names are the empty string and a single blank: `.Object~method(\"\")` \
         and `.Object~method(\" \")` both answer `The Method class`."
            .into(),
        String::new(),
        format!("{EXCLUDED_CLASS} has no rows here, as it has none in class-set.txt."),
        String::new(),
        "Derived by `cargo run -p rexx-extract --bin rexx-extract-docs`, whose".into(),
        "module is `src/docs/classes.rs`. Re-derived and compared in both".into(),
        "directions by `tests/extract_docs.rs`.".into(),
        String::new(),
        stamp.to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_group_heading_yields_the_operators_and_not_its_own_title() {
        let out = names_of(
            "mthObjectComparisonMethods",
            "Comparison Methods",
            r"= == &lt;> >&lt; \= \==",
            "fundclasses.xml",
        );
        let names: Vec<&str> = out.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["=", "==", "<>", "><", r"\=", r"\=="]);
    }

    #[test]
    fn the_two_concatenation_placeholders_become_the_names_the_oracle_has() {
        let out = names_of(
            "mthObjectConcatenationMethods",
            "Concatenation Methods",
            "(abuttal) || (blank)",
            "fundclasses.xml",
        );
        let names: Vec<&str> = out.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["", "||", " "]);
        assert_eq!(render_method(""), "(abuttal)");
        assert_eq!(parse_method("(blank)"), " ");
    }

    #[test]
    fn an_entity_prefix_is_not_part_of_the_name() {
        let out = names_of("mthQueueSize", "&added50;size", "", "collclasses.xml");
        assert_eq!(out, [("size".to_string(), Arm::Instance)]);
    }

    #[test]
    fn the_arm_suffix_carries_the_arm_case_insensitively() {
        let class = |section: &str, title: &str| names_of(section, title, "", "x")[0].1;
        assert_eq!(class("mthArrayNew", "new (Class Method)"), Arm::Class);
        assert_eq!(class("mthPackageLoad", "load (Class method)"), Arm::Class);
        assert_eq!(
            class("mthStreamNew", "new (Inherited Class Method)"),
            Arm::Class
        );
        assert_eq!(
            class("mthArrayAppend", "append (Abstract Method)"),
            Arm::Instance
        );
        assert_eq!(class("mthObjectRun", "run (Private Method)"), Arm::Instance);
        assert_eq!(
            class("mthFileLastAccessed", "lastAccessed (Attribute)"),
            Arm::Instance
        );
    }

    /// The one parenthetical that is not a kind marker. Keeping it out of the
    /// name would leave `?` intact anyway; what the case pins is that it does
    /// not silently become a class-side row.
    #[test]
    fn the_inline_if_parenthetical_is_not_a_marker_and_is_not_part_of_the_name() {
        let out = names_of(
            "mthStringIif",
            "&added50;? (inline if)",
            "",
            "fundclasses.xml",
        );
        assert_eq!(out, [("?".to_string(), Arm::Instance)]);
    }

    /// A parenthetical nobody has classified is a name the oracle would not
    /// answer, so it is loud rather than absorbed.
    #[test]
    #[should_panic(expected = "does not know")]
    fn an_unknown_title_parenthetical_is_loud() {
        names_of("mthX", "frobnicate (Ceremonial Method)", "", "x");
    }

    #[test]
    fn a_bare_new_title_is_still_class_side_for_the_two_that_carry_one() {
        assert_eq!(names_of("mthClassNew", "new", "", "x")[0].1, Arm::Class);
        assert_eq!(names_of("mthRexxQueueNew", "new", "", "x")[0].1, Arm::Class);
        // And the exception is a list of two, not a rule about the word.
        assert_eq!(names_of("mthObjectNew", "new", "", "x")[0].1, Arm::Instance);
    }

    #[test]
    fn a_slashed_title_names_two_methods() {
        let names = |title: &str| -> Vec<String> {
            names_of("mthX", title, "", "x")
                .into_iter()
                .map(|(n, _)| n)
                .collect()
        };
        assert_eq!(names("center/centre"), ["center", "centre"]);
        assert_eq!(names("delete / delStr"), ["delete", "delStr"]);
        assert_eq!(
            names("&added50;canceled/cancelled"),
            ["canceled", "cancelled"]
        );
    }

    #[test]
    fn an_ordinary_member_displays_its_targets_own_title() {
        assert_eq!(
            displayed_names("mthArrayAppend", Some("select:title"), "append", "x"),
            [("append".to_string(), false)]
        );
    }

    /// A `template:` member displays a name its target's title does not carry,
    /// so the row set needs both. Measured, both answer:
    /// `.DateTime~hasMethod("NEW")` is `1` and `.DateTime~new~hasMethod("INIT")`
    /// is `1`.
    #[test]
    fn a_template_member_yields_the_displayed_name_beside_the_title() {
        let out = displayed_names(
            "mthDateTimeInit",
            Some("template:new (Inherited Class Method)"),
            "init",
            "utilityclasses.xml",
        );
        assert_eq!(
            out,
            [
                ("init".to_string(), false),
                ("new (Inherited Class Method)".to_string(), true)
            ]
        );
        // The second name is flagged as coming from the xrefstyle, which is
        // what makes its row cite the member rather than the section whose
        // title does not carry it.
        let names: Vec<(String, Arm)> = out
            .iter()
            .flat_map(|(d, _)| names_of("mthDateTimeInit", d, "", "utilityclasses.xml"))
            .collect();
        assert_eq!(
            names,
            [
                ("init".to_string(), Arm::Instance),
                ("new".to_string(), Arm::Class)
            ]
        );
    }

    /// The guard. A sixth `template:` member added upstream must redden, not
    /// drift: the row it should produce would never be derived, and a check
    /// comparing the extractor with itself cannot see a row that does not
    /// exist on either side.
    #[test]
    #[should_panic(expected = "which neither displays the target section's own title")]
    fn an_unrecognised_xrefstyle_is_loud() {
        displayed_names(
            "mthArrayAppend",
            Some("template:add"),
            "append",
            "collclasses.xml",
        );
    }

    /// The displayed text is part of the key, so an upstream edit to it is a
    /// different member and reddens too.
    #[test]
    #[should_panic(expected = "which neither displays the target section's own title")]
    fn a_template_member_whose_displayed_text_changed_is_loud() {
        displayed_names(
            "mthDateTimeInit",
            Some("template:new (Class Method)"),
            "init",
            "utilityclasses.xml",
        );
    }

    #[test]
    #[should_panic(expected = "carries no xrefstyle")]
    fn a_member_with_no_xrefstyle_is_loud() {
        displayed_names("mthArrayAppend", None, "append", "collclasses.xml");
    }

    #[test]
    #[should_panic(expected = "with no operator list after the <xref>")]
    fn a_group_heading_with_no_operator_list_is_loud() {
        names_of(
            "mthStringLogicalMethods",
            "Logical Methods",
            "",
            "fundclasses.xml",
        );
    }
}
