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

//! Lifts individual test methods out of ooTest `.testGroup` files so they can
//! run as standalone programs long before the ooTest framework itself works.
//!
//! This is a heuristic, and deliberately conservative: a method that touches
//! fixture state set up by `setUp` cannot stand alone, so it is flagged and
//! skipped rather than mis-extracted into a silently-passing test.

/// The set of `self~` messages that are assertions rather than fixture access.
const ASSERTIONS: &[&str] = &[
    "assertequals",
    "assertnotequals",
    "asserttrue",
    "assertfalse",
    "assertnull",
    "assertnotnull",
    "assertsame",
    "assertnotsame",
    "expectsyntax",
    "assertlistequals",
    "assertarrayequals",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestMethod {
    pub name: String,
    pub body: String,
    /// True when the body reads or writes `self~<something>` that is not an
    /// assertion, meaning it depends on fixture state and cannot stand alone.
    pub uses_fixture: bool,
}

pub fn extract(source: &str) -> Vec<TestMethod> {
    let mut out = Vec::new();
    let mut current: Option<(String, Vec<&str>)> = None;
    for line in source.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = strip_directive(trimmed, "::method") {
            if let Some((name, body)) = current.take() {
                push_if_test(&mut out, name, body);
            }
            let name = rest
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_matches('"');
            current = Some((name.to_string(), Vec::new()));
        } else if trimmed.starts_with("::") {
            if let Some((name, body)) = current.take() {
                push_if_test(&mut out, name, body);
            }
        } else if let Some((_, body)) = current.as_mut() {
            body.push(line);
        }
    }
    if let Some((name, body)) = current.take() {
        push_if_test(&mut out, name, body);
    }
    out
}

fn strip_directive<'a>(line: &'a str, directive: &str) -> Option<&'a str> {
    let lower = line.to_ascii_lowercase();
    lower
        .starts_with(directive)
        .then(|| line[directive.len()..].trim_start())
}

fn push_if_test(out: &mut Vec<TestMethod>, name: String, body: Vec<&str>) {
    if !name.to_ascii_lowercase().starts_with("test") {
        return;
    }
    let body = body.join("\n");
    out.push(TestMethod {
        uses_fixture: touches_fixture(&body),
        name,
        body,
    });
}

fn touches_fixture(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    let mut rest = lower.as_str();
    while let Some(at) = rest.find("self~") {
        rest = &rest[at + "self~".len()..];
        let message: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '.')
            .collect();
        if !ASSERTIONS.contains(&message.as_str()) {
            return true;
        }
    }
    false
}
