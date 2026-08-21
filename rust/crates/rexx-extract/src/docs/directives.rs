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

//! Table D's row set: the directive and option surface, from `dire.xml` and
//! from `DirectiveParser.cpp`, unioned.
//!
//! **The union is the row set, and a name in one source only is a row of its
//! own**, because that asymmetry is where the two authorities disagree and is
//! the thing worth a probe.
//!
//! **Position, and why the parser settles it.** The markup does not
//! distinguish an option from an option's *value*: `::OPTIONS`' `<option>`
//! occurrences include `ENGINEERING`, `SCIENTIFIC`, `INHERIT` and `NOINHERIT`,
//! which the prose says are values of `FORM` and of `NUMERIC`. The parser's
//! `switch` nesting says which is which, so the row key is
//! (directive, keyword, position).
//!
//! **What this reads on the parser side, which is wider than "the
//! `SUBDIRECTIVE_*` arms of its own switch", and the two reasons.** Those four
//! `::OPTIONS` values reach the parser as `SUBKEY_*` arms, not
//! `SUBDIRECTIVE_*` ones, so the narrower rule marks four real, implemented
//! keywords as documented-only and cross-referenced -- four false rows. And
//! `::RESOURCE`'s `END` is recognised by an `if` rather than a `case`
//! (`DirectiveParser.cpp:2295`) and documented only in a railroad SVG and an
//! example, so under the narrower rule it is not a row at all, though it is
//! both documented and implemented. This module therefore reads every
//! `SUBDIRECTIVE_*` and `SUBKEY_*` token inside a directive function and
//! records in `evidence` which form each is, so the narrower set is recovered
//! by filtering rather than lost.

use std::collections::BTreeMap;

use crate::docs::xml::{blank_comments, sections, split_revision_marker};

/// The nine directive functions, each with the `dire.xml` section that
/// documents it.
pub const DIRECTIVES: &[(&str, &str, &str)] = &[
    ("::ANNOTATE", "annotd", "annotateDirective"),
    ("::ATTRIBUTE", "attrd", "attributeDirective"),
    ("::CLASS", "clasdi", "classDirective"),
    ("::CONSTANT", "constantd", "constantDirective"),
    ("::METHOD", "methd", "methodDirective"),
    ("::OPTIONS", "optionsd", "optionsDirective"),
    ("::REQUIRES", "requ", "requiresDirective"),
    ("::RESOURCE", "resourced", "resourceDirective"),
    ("::ROUTINE", "routd", "routineDirective"),
];

/// Where a keyword sits in a directive's grammar.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Position {
    /// A subkeyword of the directive itself.
    Subkeyword,
    /// A value of the named subkeyword.
    ValueOf(String),
}

impl Position {
    pub fn as_string(&self) -> String {
        match self {
            Position::Subkeyword => "subkeyword".into(),
            Position::ValueOf(outer) => format!("value-of({outer})"),
        }
    }
}

/// Which authority produced a row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// Both the section and the directive's own parser.
    Both,
    /// The directive's own parser only.
    ParserOnly,
    /// The section only. The directive's own parser has no arm for it, which
    /// is what the spec calls a cross-reference: `::CLASS`'s `<option>` set
    /// contains `CLASS` because its `METACLASS` paragraph describes
    /// `::METHOD`'s `CLASS` option.
    CrossReference,
}

impl Side {
    pub fn as_str(self) -> &'static str {
        match self {
            Side::Both => "both",
            Side::ParserOnly => "parser-only",
            Side::CrossReference => "cross-reference",
        }
    }
}

/// One row of `directive-options.txt`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionRow {
    pub directive: String,
    pub keyword: String,
    pub position: Position,
    pub side: Side,
    pub evidence: String,
}

/// One `SUBDIRECTIVE_*`/`SUBKEY_*` token inside a directive function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserArm {
    pub keyword: String,
    pub position: Position,
    pub line: usize,
    /// `case` for a `switch` arm, `comparison` for a token tested outside one.
    pub form: &'static str,
    /// `SUBDIRECTIVE` or `SUBKEY`.
    pub family: &'static str,
}

/// One documented keyword inside a `dire.xml` section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocKeyword {
    pub keyword: String,
    pub line: usize,
    /// `<option>` or `indexterm`.
    pub form: &'static str,
}

/// Every keyword `dire.xml` documents, per directive.
pub fn documented(dire_xml: &str) -> BTreeMap<String, Vec<DocKeyword>> {
    let text = blank_comments(dire_xml);
    let mut out: BTreeMap<String, Vec<DocKeyword>> = BTreeMap::new();
    for section in sections(&text) {
        let Some(id) = section.id.as_deref() else {
            continue;
        };
        let Some(&(directive, _, _)) = DIRECTIVES.iter().find(|(_, s, _)| *s == id) else {
            continue;
        };
        let (_, title) = split_revision_marker(&section.title);
        assert_eq!(
            title, directive,
            "dire.xml section {id} is titled {title:?}, not {directive:?}"
        );
        let body = &text[section.body_start..section.body_end];
        let base = text[..section.body_start]
            .bytes()
            .filter(|&b| b == b'\n')
            .count();
        let mut found: Vec<DocKeyword> = Vec::new();
        for (at, name) in tagged(body, "<option>", "</option>") {
            found.push(DocKeyword {
                keyword: name,
                line: base + 1 + body[..at].bytes().filter(|&b| b == b'\n').count(),
                form: "<option>",
            });
        }
        for (at, primary) in tagged(body, "<primary>", "</primary>") {
            if let Some(name) = primary.strip_suffix(" subkeyword") {
                found.push(DocKeyword {
                    keyword: name.to_string(),
                    line: base + 1 + body[..at].bytes().filter(|&b| b == b'\n').count(),
                    form: "indexterm",
                });
            }
        }
        found.sort_by(|a, b| a.keyword.cmp(&b.keyword).then(a.line.cmp(&b.line)));
        found.dedup_by(|a, b| a.keyword == b.keyword);
        out.insert(directive.to_string(), found);
    }
    assert_eq!(
        out.len(),
        DIRECTIVES.len(),
        "dire.xml did not yield a section for every directive function"
    );
    out
}

fn tagged(text: &str, open: &str, close: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while let Some(at) = text[i..].find(open).map(|a| a + i) {
        let from = at + open.len();
        let Some(end) = text[from..].find(close).map(|e| e + from) else {
            break;
        };
        out.push((at, text[from..end].trim().to_string()));
        i = end + close.len();
    }
    out
}

/// Every `SUBDIRECTIVE_*`/`SUBKEY_*` token each directive function names.
pub fn parser_arms(source: &str) -> BTreeMap<String, Vec<ParserArm>> {
    let mut out = BTreeMap::new();
    for &(directive, _, function) in DIRECTIVES {
        let (body, base) = function_body(source, function);
        out.insert(directive.to_string(), scan_arms(body, base));
    }
    out
}

/// The braced body of `void LanguageParser::<name>()`, and the 1-based line
/// its opening brace is on.
fn function_body<'a>(source: &'a str, name: &str) -> (&'a str, usize) {
    let signature = format!("void LanguageParser::{name}()");
    let at = source
        .find(&signature)
        .unwrap_or_else(|| panic!("DirectiveParser.cpp has no {signature}"));
    let open = source[at..]
        .find('{')
        .map(|o| at + o)
        .unwrap_or_else(|| panic!("{signature} has no body"));
    let mut depth = 0usize;
    let mut scanner = Scanner::new(&source[open..]);
    let mut end = source.len() - open;
    while let Some((offset, token)) = scanner.next_token() {
        match token {
            Token::Open => depth += 1,
            Token::Close => {
                depth -= 1;
                if depth == 0 {
                    end = offset + 1;
                    break;
                }
            }
            Token::Word(_) => {}
        }
    }
    let base = source[..open].bytes().filter(|&b| b == b'\n').count() + 1;
    (&source[open..open + end], base)
}

fn scan_arms(body: &str, base_line: usize) -> Vec<ParserArm> {
    /// One `{ ... }` block, and whether it is a `switch` body.
    struct Frame {
        is_switch: bool,
        current_case: Option<String>,
    }
    let mut frames: Vec<Frame> = Vec::new();
    let mut next_is_switch = false;
    let mut pending_case = false;
    let mut out = Vec::new();
    let mut scanner = Scanner::new(body);
    while let Some((offset, token)) = scanner.next_token() {
        match token {
            Token::Open => {
                frames.push(Frame {
                    is_switch: next_is_switch,
                    current_case: None,
                });
                next_is_switch = false;
            }
            Token::Close => {
                frames.pop();
            }
            Token::Word(word) => {
                if word == "switch" {
                    next_is_switch = true;
                    continue;
                }
                if word == "case" {
                    pending_case = true;
                    continue;
                }
                let family = if word.starts_with("SUBDIRECTIVE_") {
                    "SUBDIRECTIVE"
                } else if word.starts_with("SUBKEY_") {
                    "SUBKEY"
                } else {
                    pending_case = false;
                    continue;
                };
                let keyword = word
                    .split_once('_')
                    .map(|(_, rest)| rest.to_string())
                    .unwrap_or_default();
                let is_case = std::mem::take(&mut pending_case);
                if is_case && let Some(frame) = frames.last_mut() {
                    frame.current_case = Some(keyword.clone());
                }
                // The enclosing switch, if this is a case, is the innermost
                // frame; the position comes from the *next* switch outwards,
                // whose current case is the keyword this one is a value of.
                let skip = usize::from(is_case);
                let position = frames
                    .iter()
                    .rev()
                    .filter(|f| f.is_switch)
                    .nth(skip)
                    .and_then(|f| f.current_case.clone())
                    .map_or(Position::Subkeyword, Position::ValueOf);
                out.push(ParserArm {
                    keyword,
                    position,
                    line: base_line + body[..offset].bytes().filter(|&b| b == b'\n').count(),
                    form: if is_case { "case" } else { "comparison" },
                    family,
                });
            }
        }
    }
    out
}

enum Token<'a> {
    Open,
    Close,
    Word(&'a str),
}

/// A C++ scanner that yields braces and identifiers and nothing else, with
/// comments and literals skipped.
///
/// Counting braces off raw text would be enough for this file today; it is
/// done properly because a brace inside a string or a comment is a silent
/// off-by-one in the nesting that decides every `value-of(...)` position.
struct Scanner<'a> {
    text: &'a str,
    at: usize,
}

impl<'a> Scanner<'a> {
    fn new(text: &'a str) -> Self {
        Scanner { text, at: 0 }
    }

    fn next_token(&mut self) -> Option<(usize, Token<'a>)> {
        let bytes = self.text.as_bytes();
        while self.at < bytes.len() {
            let start = self.at;
            let rest = &self.text[start..];
            if rest.starts_with("//") {
                self.at = rest.find('\n').map_or(bytes.len(), |n| start + n);
                continue;
            }
            if let Some(after) = rest.strip_prefix("/*") {
                self.at = after.find("*/").map_or(bytes.len(), |n| start + 2 + n + 2);
                continue;
            }
            if rest.starts_with('"') || rest.starts_with('\'') {
                self.at = start + literal_len(rest);
                continue;
            }
            let b = bytes[start];
            if b == b'{' {
                self.at = start + 1;
                return Some((start, Token::Open));
            }
            if b == b'}' {
                self.at = start + 1;
                return Some((start, Token::Close));
            }
            if b.is_ascii_alphabetic() || b == b'_' {
                let len = rest
                    .bytes()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == b'_')
                    .count();
                self.at = start + len;
                return Some((start, Token::Word(&rest[..len])));
            }
            self.at = start + 1;
        }
        None
    }
}

fn literal_len(rest: &str) -> usize {
    let quote = rest.as_bytes()[0];
    let bytes = rest.as_bytes();
    let mut i = 1usize;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b if b == quote => return i + 1,
            _ => i += 1,
        }
    }
    bytes.len()
}

/// The union, one row per (directive, keyword, position).
pub fn option_rows(
    documented: &BTreeMap<String, Vec<DocKeyword>>,
    arms: &BTreeMap<String, Vec<ParserArm>>,
) -> Vec<OptionRow> {
    let mut out = Vec::new();
    for &(directive, section, function) in DIRECTIVES {
        let doc = documented
            .get(directive)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let parser = arms.get(directive).map(Vec::as_slice).unwrap_or_default();
        let mut rows: BTreeMap<(String, String), OptionRow> = BTreeMap::new();
        for arm in parser {
            let documented_here = doc.iter().find(|d| d.keyword == arm.keyword);
            let side = if documented_here.is_some() {
                Side::Both
            } else {
                Side::ParserOnly
            };
            let evidence = match documented_here {
                Some(d) => format!(
                    "DirectiveParser.cpp:{} {} {}_{}; dire.xml:{} {}",
                    arm.line, arm.form, arm.family, arm.keyword, d.line, d.form
                ),
                None => format!(
                    "DirectiveParser.cpp:{} {} {}_{}; no <option> or subkeyword indexterm in \
                     dire.xml's {section} section",
                    arm.line, arm.form, arm.family, arm.keyword
                ),
            };
            rows.entry((arm.keyword.clone(), arm.position.as_string()))
                .or_insert(OptionRow {
                    directive: directive.to_string(),
                    keyword: arm.keyword.clone(),
                    position: arm.position.clone(),
                    side,
                    evidence,
                });
        }
        for keyword in doc {
            if parser.iter().any(|a| a.keyword == keyword.keyword) {
                continue;
            }
            // The directive's own parser has no arm for it. Say where one does
            // handle it, because that is what makes "cross-reference" a
            // finding rather than a shrug.
            let elsewhere: Vec<String> = DIRECTIVES
                .iter()
                .filter(|(d, _, _)| *d != directive)
                .filter_map(|(d, _, _)| {
                    arms.get(*d)?
                        .iter()
                        .find(|a| a.keyword == keyword.keyword)
                        .map(|a| format!("{d} (DirectiveParser.cpp:{})", a.line))
                })
                .collect();
            let where_handled = if elsewhere.is_empty() {
                format!(
                    "no directive function names {}_{}",
                    "SUBDIRECTIVE", keyword.keyword
                )
            } else {
                format!("handled by {}", elsewhere.join(", "))
            };
            rows.entry((keyword.keyword.clone(), Position::Subkeyword.as_string()))
                .or_insert(OptionRow {
                    directive: directive.to_string(),
                    keyword: keyword.keyword.clone(),
                    position: Position::Subkeyword,
                    side: Side::CrossReference,
                    evidence: format!(
                        "dire.xml:{} {}; {function} has no arm for it, {where_handled}",
                        keyword.line, keyword.form
                    ),
                });
        }
        out.extend(rows.into_values());
    }
    out
}

/// `directive<TAB>keyword<TAB>position<TAB>side<TAB>evidence`.
pub fn rows(rows: &[OptionRow]) -> Vec<String> {
    rows.iter()
        .map(|r| {
            format!(
                "{}\t{}\t{}\t{}\t{}",
                r.directive,
                r.keyword,
                r.position.as_string(),
                r.side.as_str(),
                r.evidence
            )
        })
        .collect()
}

/// The header the committed file carries, with its D56 stamp.
pub fn header(stamp: &str) -> Vec<String> {
    vec![
        "Table D's row set: the directive and option surface, from dire.xml and".into(),
        "from interpreter/parser/DirectiveParser.cpp, unioned.".into(),
        String::new(),
        "One `directive<TAB>keyword<TAB>position<TAB>side<TAB>evidence` per".into(),
        "line, grouped by directive in the order DIRECTIVES lists and sorted".into(),
        "within a directive by (keyword, position).".into(),
        String::new(),
        "`position` is `subkeyword` or `value-of(NAME)`. The markup does not".into(),
        "distinguish an option from an option's value -- ::OPTIONS' <option>".into(),
        "occurrences include ENGINEERING and SCIENTIFIC, which the prose says".into(),
        "are values of FORM -- so the parser's switch nesting settles it.".into(),
        String::new(),
        "`side` is which authority produced the row. `cross-reference` means the".into(),
        "section documents the name and the directive's own parser has no arm".into(),
        "for it; the evidence says where one does handle it. ::CLASS's CLASS row".into(),
        "is the worked case: its METACLASS paragraph describes ::METHOD's CLASS".into(),
        "option.".into(),
        String::new(),
        "The parser side reads every SUBDIRECTIVE_* and SUBKEY_* token inside a".into(),
        "directive function, not only the SUBDIRECTIVE_* arms of its own switch,".into(),
        "and `evidence` records which form each is. Two reasons, both measured:".into(),
        "::OPTIONS' FORM and NUMERIC values reach the parser as SUBKEY_* arms,".into(),
        "so the narrower rule marks four implemented keywords as documented-only;".into(),
        "and ::RESOURCE's END is recognised by an `if` rather than a `case`, so".into(),
        "the narrower rule gives it no row at all though it is both documented".into(),
        "and implemented.".into(),
        String::new(),
        "DirectiveParser.cpp is tracked in this repository, so it needs no".into(),
        "revision stamp of its own -- a change to it lands in the same history as".into(),
        "this file. The stamp below is dire.xml's.".into(),
        String::new(),
        "Derived by `cargo run -p rexx-extract --bin rexx-extract-docs`, whose".into(),
        "module is `src/docs/directives.rs`. Re-derived and compared in both".into(),
        "directions by `tests/extract_docs.rs`.".into(),
        String::new(),
        stamp.to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = r#"
void LanguageParser::optionsDirective()
{
    for (;;)
    {
        switch (token->subDirective())
        {
            // ::OPTIONS DIGITS nnnn -- a brace in a comment { does not count
            case SUBDIRECTIVE_DIGITS:
            {
                syntaxError(Error_x, "an unbalanced { in a literal");
                break;
            }

            case SUBDIRECTIVE_FORM:
            {
                switch (token->subKeyword())
                {
                    case SUBKEY_SCIENTIFIC:
                        break;
                    case SUBKEY_ENGINEERING:
                        break;
                }
                break;
            }

            case SUBDIRECTIVE_NOVALUE:
            {
                switch (token->subDirective())
                {
                    case SUBDIRECTIVE_ERROR:
                    case SUBDIRECTIVE_SYNTAX:
                        break;
                }
                break;
            }
        }
    }
}

void LanguageParser::resourceDirective()
{
    if (token->subDirective() != SUBDIRECTIVE_END)
    {
        syntaxError(Error_Invalid_subkeyword_resource, token);
    }
}
"#;

    fn arms_of(function: &str) -> Vec<ParserArm> {
        let (body, base) = function_body(SOURCE, function);
        scan_arms(body, base)
    }

    #[test]
    fn a_nested_switch_arm_is_a_value_of_the_arm_it_sits_under() {
        let arms = arms_of("optionsDirective");
        let seen: Vec<(String, String)> = arms
            .iter()
            .map(|a| (a.keyword.clone(), a.position.as_string()))
            .collect();
        assert_eq!(
            seen,
            [
                ("DIGITS".into(), "subkeyword".to_string()),
                ("FORM".into(), "subkeyword".into()),
                ("SCIENTIFIC".into(), "value-of(FORM)".into()),
                ("ENGINEERING".into(), "value-of(FORM)".into()),
                ("NOVALUE".into(), "subkeyword".into()),
                ("ERROR".into(), "value-of(NOVALUE)".into()),
                ("SYNTAX".into(), "value-of(NOVALUE)".into()),
            ]
        );
        assert!(arms.iter().all(|a| a.form == "case"));
    }

    /// The four `::OPTIONS` value keywords are `SUBKEY_*`, and recording the
    /// family is what lets the narrower "`SUBDIRECTIVE_*` arms" set be
    /// recovered from the committed file by filtering.
    #[test]
    fn the_family_of_each_arm_is_recorded() {
        let arms = arms_of("optionsDirective");
        let subkeys: Vec<&str> = arms
            .iter()
            .filter(|a| a.family == "SUBKEY")
            .map(|a| a.keyword.as_str())
            .collect();
        assert_eq!(subkeys, ["SCIENTIFIC", "ENGINEERING"]);
    }

    /// A token tested outside a `switch` is still an arm of the grammar.
    #[test]
    fn a_comparison_outside_a_switch_is_recorded_as_one() {
        let arms = arms_of("resourceDirective");
        assert_eq!(arms.len(), 1);
        assert_eq!(arms[0].keyword, "END");
        assert_eq!(arms[0].form, "comparison");
        assert_eq!(arms[0].position, Position::Subkeyword);
    }

    /// A brace inside a comment or a string literal would shift every nesting
    /// decision after it. `optionsDirective`'s sample carries one of each.
    #[test]
    fn braces_in_comments_and_literals_do_not_move_the_nesting() {
        let arms = arms_of("optionsDirective");
        let form = arms
            .iter()
            .find(|a| a.keyword == "SCIENTIFIC")
            .expect("SUBKEY_SCIENTIFIC");
        assert_eq!(form.position, Position::ValueOf("FORM".into()));
    }
}
