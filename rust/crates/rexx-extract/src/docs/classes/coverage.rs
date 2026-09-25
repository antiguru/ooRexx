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

//! A class row's coverage: the construction measurements, read together
//! with the reference's own sentences saying a class cannot be constructed.

use std::collections::BTreeMap;

use super::{
    Book, CONSTRUCTION, CONSTRUCTION_DIRECTIVES, CONSTRUCTION_PROGRAMS, Coverage, Route, Status,
    UNCONSTRUCTIBLE, UNCONSTRUCTIBLE_BOOK,
};

/// Re-reads each [`UNCONSTRUCTIBLE`] quotation at the lines it cites, so a
/// citation that has gone stale reddens rather than being carried.
pub(super) fn unconstructible_index(
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
pub(super) fn coverage_of(
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
    let directive = CONSTRUCTION_DIRECTIVES
        .iter()
        .find(|(n, _)| *n == upper)
        .map(|(_, directive)| (*directive).to_string());
    assert!(
        program.is_none() || construction != Some("new"),
        "{name} has a committed construction program and a bare ~new that constructs on \
         the oracle, so the program is a route to nothing the bare ~new does not already reach"
    );
    assert!(
        directive.is_none() || program.is_some(),
        "{name} has a committed construction directive and no construction program, so \
         nothing derives a probe that would carry the directive"
    );
    let route = |expression: &str| Route {
        expression: expression.to_string(),
        directive: directive.clone(),
    };
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
                Coverage::Covered(route(program)),
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
            Coverage::Covered(route(program)),
            format!(
                "a committed construction program answers an instance; a bare ~new raises \
                 {code} on the oracle"
            ),
        );
    }
    match construction {
        Some("new") => (
            Coverage::Covered(route(&format!(".{name}~new"))),
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
