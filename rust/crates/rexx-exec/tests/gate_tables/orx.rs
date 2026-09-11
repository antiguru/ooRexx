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

//! Reading the interpreter's own bootstrap Rexx -- `CoreClasses.orx` and
//! `StreamClasses.orx` -- so a gate table can say which directive shapes the
//! bootstrap actually depends on.

use std::fs;
use std::path::PathBuf;

/// The two bootstrap files, by the names they are cited by.
pub const CORE_CLASSES: &str = "CoreClasses.orx";
pub const STREAM_CLASSES: &str = "StreamClasses.orx";

/// Where the interpreter keeps its bootstrap Rexx.
pub fn orx_root() -> PathBuf {
    PathBuf::from("/home/moritz/dev/repos/ooRexx/interpreter/RexxClasses")
}

/// One operand of a directive clause, and whether it arrived as a quoted
/// literal. The quoting matters: a quoted operand is a name, never a keyword,
/// which is what keeps `subclass 'Supplier'` from reading as a keyword called
/// `SUPPLIER`.
#[derive(Clone, Debug)]
pub struct Token {
    pub text: String,
    pub quoted: bool,
}

impl Token {
    /// Whether this token is the unquoted keyword `keyword`, compared the way
    /// the parser compares a subkeyword: case-insensitively, and never
    /// against a literal.
    pub fn is_keyword(&self, keyword: &str) -> bool {
        !self.quoted && self.text.eq_ignore_ascii_case(keyword)
    }
}

/// One directive clause, as scanned.
#[derive(Clone, Debug)]
pub struct DirectiveLine {
    /// Which of the two files, by [`CORE_CLASSES`] / [`STREAM_CLASSES`].
    pub file: &'static str,
    /// 1-based physical line, so a hit can be cited as `file:line`.
    pub line: usize,
    /// The physical line, as it appears in the file, trimmed of trailing
    /// whitespace.
    pub text: String,
    /// The directive's own name, upper-cased and with its `::`, so it
    /// compares directly against a row's directive column.
    pub name: String,
    /// The operands after the directive name, up to the clause's end.
    pub tokens: Vec<Token>,
    /// The name of the `::CLASS` this directive sits under, unquoted and
    /// upper-cased, or `None` before the file's first one.
    pub enclosing_class: Option<String>,
    /// The clause ended at a `;` and more of the program follows on the same
    /// physical line.
    pub body_on_same_line: bool,
}

/// A place in one of the two files that a scan found something at.
#[derive(Clone, Debug)]
pub struct Hit {
    pub file: &'static str,
    pub line: usize,
    pub text: String,
}

impl Hit {
    pub fn cite(&self) -> String {
        format!("{}:{}", self.file, self.line)
    }
}

fn hit(directive: &DirectiveLine) -> Hit {
    Hit {
        file: directive.file,
        line: directive.line,
        text: directive.text.clone(),
    }
}

/// Every directive clause in `file`, in file order.
pub fn scan(file: &'static str) -> Vec<DirectiveLine> {
    let path = orx_root().join(file);
    let text = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "cannot read {} -- the bootstrap Rexx this scan derives its answers \
             from is part of the read-only C++ tree, so a missing one means the \
             tree moved rather than that the shape is gone: {e}",
            path.display()
        )
    });
    scan_text(file, &text)
}

/// [`scan`], over text already in hand, so the lexer can be tested on
/// constructed input rather than only on the two real files.
pub fn scan_text(file: &'static str, text: &str) -> Vec<DirectiveLine> {
    let mut comment_depth = 0usize;
    let mut enclosing_class: Option<String> = None;
    let mut out = Vec::new();

    for (index, raw) in text.lines().enumerate() {
        let code = strip_comments(raw, &mut comment_depth);
        let trimmed = code.trim_start();
        if !trimmed.starts_with("::") {
            continue;
        }
        let (name, rest) = split_directive_name(trimmed);
        let (tokens, body_on_same_line) = tokenize_clause(rest);
        let directive = DirectiveLine {
            file,
            line: index + 1,
            text: raw.trim_end().to_string(),
            name,
            tokens,
            enclosing_class: enclosing_class.clone(),
            body_on_same_line,
        };
        if directive.name == "::CLASS" {
            enclosing_class = directive
                .tokens
                .first()
                .map(|token| token.text.to_ascii_uppercase());
        }
        out.push(directive);
    }
    out
}

/// Replaces every comment region of one physical line with a space, carrying
/// `depth` across lines so a `/* */` spanning several of them does not leave
/// its body looking like code.
fn strip_comments(line: &str, depth: &mut usize) -> String {
    let bytes: Vec<char> = line.chars().collect();
    let mut out = String::new();
    let mut index = 0usize;
    while index < bytes.len() {
        if *depth > 0 {
            if bytes[index] == '*' && bytes.get(index + 1) == Some(&'/') {
                *depth -= 1;
                index += 2;
                out.push(' ');
                continue;
            }
            if bytes[index] == '/' && bytes.get(index + 1) == Some(&'*') {
                *depth += 1;
                index += 2;
                continue;
            }
            index += 1;
            continue;
        }
        match bytes[index] {
            '/' if bytes.get(index + 1) == Some(&'*') => {
                *depth += 1;
                index += 2;
                out.push(' ');
            }
            '-' if bytes.get(index + 1) == Some(&'-') => break,
            quote @ ('\'' | '"') => {
                out.push(quote);
                index += 1;
                while index < bytes.len() {
                    out.push(bytes[index]);
                    index += 1;
                    if bytes[index - 1] == quote {
                        break;
                    }
                }
            }
            other => {
                out.push(other);
                index += 1;
            }
        }
    }
    out
}

/// Splits `::NAME rest` into the upper-cased `::NAME` and the rest.
fn split_directive_name(text: &str) -> (String, &str) {
    let after = &text[2..];
    let end = after
        .find(|c: char| c.is_whitespace() || c == ';')
        .unwrap_or(after.len());
    (
        format!("::{}", after[..end].to_ascii_uppercase()),
        &after[end..],
    )
}

/// The operand tokens of a directive clause, and whether the program
/// continues on the same physical line after the `;` that ended it.
fn tokenize_clause(text: &str) -> (Vec<Token>, bool) {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut index = 0usize;
    let mut body_on_same_line = false;

    while index < chars.len() {
        if chars[index].is_whitespace() {
            index += 1;
            continue;
        }
        if chars[index] == ';' {
            body_on_same_line = chars[index + 1..].iter().any(|c| !c.is_whitespace());
            break;
        }
        if chars[index] == '\'' || chars[index] == '"' {
            let quote = chars[index];
            index += 1;
            let mut literal = String::new();
            while index < chars.len() && chars[index] != quote {
                literal.push(chars[index]);
                index += 1;
            }
            index += 1;
            tokens.push(Token {
                text: literal,
                quoted: true,
            });
            continue;
        }
        let mut word = String::new();
        while index < chars.len() && !chars[index].is_whitespace() && chars[index] != ';' {
            word.push(chars[index]);
            index += 1;
        }
        tokens.push(Token {
            text: word,
            quoted: false,
        });
    }
    (tokens, body_on_same_line)
}

/// The token index a directive's options can start at. See the module doc for
/// the two groups and why the split is where it is.
fn first_option_index(directive: &str) -> usize {
    match directive {
        "::OPTIONS" | "::ANNOTATE" => 0,
        _ => 1,
    }
}

/// How many tokens a keyword takes with it, so the walk over a clause's option
/// positions steps past an operand rather than reading it as another keyword.
pub fn operands_consumed(directive: &str, keyword: &str) -> usize {
    match (directive, keyword) {
        ("::CLASS", "INHERIT") => usize::MAX,
        ("::CLASS", "METACLASS" | "MIXINCLASS" | "SUBCLASS") => 1,
        ("::METHOD" | "::ATTRIBUTE", "DELEGATE" | "EXTERNAL") => 1,
        ("::ROUTINE", "EXTERNAL") => 1,
        ("::REQUIRES", "NAMESPACE") => 1,
        ("::RESOURCE", "END") => 1,
        ("::ANNOTATE", "ATTRIBUTE" | "CLASS" | "CONSTANT" | "METHOD" | "ROUTINE") => 1,
        (
            "::OPTIONS",
            "ALL" | "DIGITS" | "ERROR" | "FAILURE" | "FORM" | "FUZZ" | "LOSTDIGITS" | "NOSTRING"
            | "NOTREADY" | "NOVALUE" | "NUMERIC" | "TRACE",
        ) => 1,
        _ => 0,
    }
}

/// Whether this clause uses `keyword` at an option position. See the module
/// doc for what an option position is.
fn uses_at_an_option_position(line: &DirectiveLine, keyword: &str) -> bool {
    let mut index = first_option_index(line.name.as_str());
    while index < line.tokens.len() {
        let token = &line.tokens[index];
        if token.is_keyword(keyword) {
            return true;
        }
        if token.quoted {
            index += 1;
            continue;
        }
        let consumed = operands_consumed(&line.name, &token.text.to_ascii_uppercase());
        if consumed == usize::MAX {
            return false;
        }
        index += consumed + 1;
    }
    false
}

/// Every directive clause in `lines` that uses `directive` with `keyword` at
/// `position`. See the module doc for the exact predicate.
pub fn usage(lines: &[DirectiveLine], directive: &str, keyword: &str, position: &str) -> Vec<Hit> {
    let mut hits = Vec::new();
    for line in lines {
        if line.name != directive {
            continue;
        }
        let found = match value_of_target(position) {
            Some(owner) => line
                .tokens
                .windows(2)
                .any(|pair| pair[0].is_keyword(owner) && pair[1].is_keyword(keyword)),
            None => uses_at_an_option_position(line, keyword),
        };
        if found {
            hits.push(hit(line));
        }
    }
    hits
}

/// The keyword `X` of a `value-of(X)` position, or `None` for `subkeyword`.
fn value_of_target(position: &str) -> Option<&str> {
    position
        .strip_prefix("value-of(")
        .and_then(|rest| rest.strip_suffix(')'))
}

/// A `::CLASS` whose own name is a quoted literal, carrying both
/// `MIXINCLASS` and `INHERIT`, whose `INHERIT` names more than one class.
pub fn quoted_mixin_inheriting_several(lines: &[DirectiveLine]) -> Vec<Hit> {
    let mut hits = Vec::new();
    for line in lines {
        if line.name != "::CLASS" || !line.tokens.first().is_some_and(|token| token.quoted) {
            continue;
        }
        if !line
            .tokens
            .iter()
            .any(|token| token.is_keyword("MIXINCLASS"))
        {
            continue;
        }
        let Some(at) = line
            .tokens
            .iter()
            .position(|token| token.is_keyword("INHERIT"))
        else {
            continue;
        };
        // Everything after INHERIT up to the next keyword of this directive
        // is the class list; on these files nothing follows it, so the count
        // is the tail's length.
        if line.tokens[at + 1..].len() > 1 {
            hits.push(hit(line));
        }
    }
    hits
}

/// A `::CLASS` whose `SUBCLASS` target is a quoted literal rather than a
/// symbol.
pub fn quoted_subclass_target(lines: &[DirectiveLine]) -> Vec<Hit> {
    let mut hits = Vec::new();
    for line in lines {
        if line.name != "::CLASS" {
            continue;
        }
        let uses_quoted_target = line
            .tokens
            .windows(2)
            .any(|pair| pair[0].is_keyword("SUBCLASS") && pair[1].quoted);
        if uses_quoted_target {
            hits.push(hit(line));
        }
    }
    hits
}

/// A `::CONSTANT` whose value expression sends a message to its own class,
/// naming a method that class declares `PRIVATE CLASS`.
pub fn constant_sending_an_own_private_class_method(lines: &[DirectiveLine]) -> Vec<Hit> {
    let mut hits = Vec::new();
    for line in lines {
        if line.name != "::CONSTANT" {
            continue;
        }
        let Some(class) = line.enclosing_class.as_deref() else {
            continue;
        };
        let expression: String = line.tokens[1..]
            .iter()
            .map(|token| token.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
            .to_ascii_uppercase();
        let sends_own_private_class_method = lines.iter().any(|candidate| {
            candidate.name == "::METHOD"
                && candidate.enclosing_class.as_deref() == Some(class)
                && candidate.file == line.file
                && candidate.tokens.iter().any(|t| t.is_keyword("PRIVATE"))
                && candidate.tokens.iter().any(|t| t.is_keyword("CLASS"))
                && candidate.tokens.first().is_some_and(|name| {
                    expression.contains(&format!(".{class}~{}", name.text.to_ascii_uppercase()))
                })
        });
        if sends_own_private_class_method {
            hits.push(hit(line));
        }
    }
    hits
}

/// A directive whose clause ended at a `;` with more of the program after it
/// on the same physical line.
pub fn body_on_the_same_physical_line(lines: &[DirectiveLine]) -> Vec<Hit> {
    lines
        .iter()
        .filter(|line| line.body_on_same_line)
        .map(hit)
        .collect()
}
