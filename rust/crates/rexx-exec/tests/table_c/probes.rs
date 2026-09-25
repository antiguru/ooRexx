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

//! The probe programs gate table C derives from its rows: where each
//! family's programs live, and the text each row's program holds.

use super::rows::Construction;

/// Where each family's probe programs live, relative to the corpus root.
pub(crate) const CONCEPT_SUBDIR: &str = "gate-tables/concepts";
pub(crate) const CLASS_SUBDIR: &str = "gate-tables/classes";
pub(crate) const EDGE_SUBDIR: &str = "gate-tables/hierarchy";
pub(crate) const METHOD_SUBDIR: &str = "gate-tables/methods";

/// The probe program for a concept row, as a corpus-relative path.
pub(crate) fn concept_probe(id: &str) -> String {
    format!("{CONCEPT_SUBDIR}/{id}.rex")
}

/// The probe program for a class wiring row.
pub(crate) fn class_probe(name: &str) -> String {
    format!("{CLASS_SUBDIR}/{}.rex", name.to_ascii_lowercase())
}

/// The probe program for a hierarchy edge row.
pub(crate) fn edge_probe(child: &str, parent: &str) -> String {
    format!(
        "{EDGE_SUBDIR}/{}__{}.rex",
        child.to_ascii_lowercase(),
        parent.to_ascii_lowercase()
    )
}

/// The probe program a (class, arm) group of method rows shares.
pub(crate) fn method_probe(class: &str, arm: &str) -> String {
    format!("{METHOD_SUBDIR}/{}__{arm}.rex", class.to_ascii_lowercase())
}

/// The marker a class probe prints the `.environment` entry itself under.
pub(crate) const ENTRY_MARKER: &str = "entry ";

/// The questions a class wiring row asks that **any** `.environment` entry
/// answers, whether it is a class object or an instance.
pub(crate) fn class_probe_entry_questions(name: &str) -> Vec<String> {
    vec![
        format!("say '{}' .{name}\n", ENTRY_MARKER.trim_end()),
        format!("say 'class-of-entry' .{name}~class~id\n"),
    ]
}

/// The questions a class wiring row asks that only a **class object** answers.
pub(crate) fn class_probe_class_questions(name: &str) -> Vec<String> {
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
pub(crate) fn class_probe_text(name: &str) -> String {
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

/// The text of a hierarchy edge row's probe.
pub(crate) fn edge_probe_text(child: &str, parent: &str) -> String {
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
pub(crate) fn method_probe_text(
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

/// How many lines a derived probe prints when every `say` in it is reached.
pub(crate) fn derived_say_lines(text: &str) -> usize {
    text.lines().filter(|line| line.starts_with("say ")).count()
}

/// The marker an edge probe's derived text prints its documented-edge answer
/// under.
pub(crate) const DOCUMENTED_EDGE_MARKER: &str = "documented-edge ";
