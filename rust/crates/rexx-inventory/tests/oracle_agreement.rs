//! Checks the generated message table against the oracle's own generated
//! header, message for message.
//!
//! `interpreter/messages/RexxErrorMessages.h` is produced from the same
//! `rexxmsg.xml` by `RexxErrorMessages.xsl`, so it is an independent rendering
//! of exactly the text this crate must reproduce. Comparing against it
//! validates every markup rule -- `<q>`, `<sq/>`, `<dq/>`, `<Sub>`, entity
//! unescaping -- across all 704 messages at once, rather than the handful a
//! hand-written test can assert.
//!
//! This is the test that would have caught treating `<q>` as documentation-only
//! markup: 363 messages use it, and dropping its quotes fails 363 comparisons.

use rexx_inventory::errors;
use std::collections::HashMap;

const HEADER: &str = "../../../interpreter/messages/RexxErrorMessages.h";

/// Pulls `MESSAGE(Symbol, "text")` pairs out of the generated C header.
///
/// The only escape the header uses is `\"` (727 occurrences; verified by
/// scanning the file), so the unescaper handles that and backslash itself and
/// rejects anything else rather than silently mis-decoding it.
fn oracle_messages() -> HashMap<String, String> {
    let src =
        std::fs::read_to_string(HEADER).unwrap_or_else(|e| panic!("cannot read {HEADER}: {e}"));
    let mut out = HashMap::new();
    for line in src.lines() {
        let Some(rest) = line.trim_start().strip_prefix("MESSAGE(") else {
            continue;
        };
        let Some((symbol, rest)) = rest.split_once(',') else {
            continue;
        };
        let symbol = symbol.trim();
        // `Table_end` is a sentinel terminating the C table, not a message.
        if symbol == "Table_end" {
            continue;
        }
        let Some(open) = rest.find('"') else { continue };
        let mut text = String::new();
        let mut chars = rest[open + 1..].chars();
        loop {
            match chars.next() {
                None => panic!("unterminated string for {symbol}"),
                Some('"') => break,
                Some('\\') => match chars.next() {
                    Some('"') => text.push('"'),
                    Some('\\') => text.push('\\'),
                    other => panic!("unhandled escape \\{other:?} for {symbol}"),
                },
                Some(c) => text.push(c),
            }
        }
        out.insert(symbol.to_string(), text);
    }
    out
}

#[test]
fn every_message_renders_exactly_as_the_oracle_renders_it() {
    let oracle = oracle_messages();
    assert_eq!(
        oracle.len(),
        704,
        "the oracle header lost or gained a message"
    );

    let ours: HashMap<&str, &str> = errors::MESSAGES
        .iter()
        .map(|m| (m.symbol, m.text))
        .collect();
    assert_eq!(ours.len(), 704, "symbols must be unique across the table");

    let mut mismatches = Vec::new();
    for (symbol, expected) in &oracle {
        match ours.get(symbol.as_str()) {
            None => mismatches.push(format!("{symbol}: missing from our table")),
            Some(got) if got != expected => mismatches.push(format!(
                "{symbol}:\n  oracle: {expected:?}\n  ours:   {got:?}"
            )),
            Some(_) => {}
        }
    }
    assert!(
        mismatches.is_empty(),
        "{} of {} messages disagree with the oracle:\n{}",
        mismatches.len(),
        oracle.len(),
        mismatches
            .iter()
            .take(10)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}
