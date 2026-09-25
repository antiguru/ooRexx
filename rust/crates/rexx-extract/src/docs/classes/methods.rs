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

//! A method row's names and arm: each class-table member's target section,
//! the names the book displays for it, and the arm each name is asked on.

use std::collections::BTreeMap;

use crate::docs::xml::{decode_predefined, members, split_revision_marker};

use super::{Arm, Book, OPERATOR_PLACEHOLDERS};

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

/// The `xrefstyle` under which a class-table member displays the target
/// section's own `<title>`, which is what makes the title the method's name.
const TITLE_XREFSTYLE: &str = "select:title";

/// Class-table members whose `xrefstyle` **overrides** the displayed name, and
/// the text it displays instead.
const TEMPLATE_MEMBERS: &[(&str, &str)] = &[
    ("mthDateTimeInit", "template:new (Inherited Class Method)"),
    ("mthRexxQueueNew", "template:new (Inherited Class Method)"),
    ("mthTimeSpanInit", "template:new (Inherited Class Method)"),
];

/// The `mth*` section of every id the class tables name, keyed by id.
pub(super) fn method_sections(books: &[Book]) -> BTreeMap<String, (String, usize, String)> {
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

/// One `<member>` of a class table, as the row derivation needs it.
pub(super) struct Listed {
    /// The `mth*` section the member links to.
    pub(super) target: String,
    /// The `<xref>`'s `xrefstyle`, which decides what the book displays.
    pub(super) xrefstyle: Option<String>,
    /// Text after the `<xref/>`, where a group heading spells out its
    /// operators.
    pub(super) trailing: String,
    /// The file the member was read from: a book, or the `*classmethods.xml`
    /// a class table includes.
    pub(super) source: String,
    /// The member's own line in that file. This is the citation a row carries
    /// when the member's `xrefstyle` is where its name came from.
    pub(super) line: usize,
}

pub(super) fn listed_members(text: &str, source: &str, base_line: usize) -> Vec<Listed> {
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
pub(super) fn displayed_names(
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
pub(super) fn names_of(
    section: &str,
    title: &str,
    trailing: &str,
    source: &str,
) -> Vec<(String, Arm)> {
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
const TITLE_PARENTHETICALS: &[(&str, Option<Arm>)] = &[
    ("class method", Some(Arm::Class)),
    ("inherited class method", Some(Arm::Class)),
    ("abstract method", Some(Arm::Instance)),
    ("private method", Some(Arm::Instance)),
    ("attribute", Some(Arm::Instance)),
    ("inline if", None),
];

/// Splits a trailing `(...)` off a title and reads the arm it carries, if any.
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
