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

//! `Setup.cpp`, read for the native-or-Rexx column and the entry-point token
//! that `collection_scopes.rs` and `introspection_scopes.rs` both join onto
//! the oracle's own scope answer.
//!
//! Native-or-Rexx and the token are keyed by (scope, name): present in that
//! scope's post-`InheritInstanceMethods` table means native, absent means
//! Rexx. The Method object cannot answer it -- `~source~items` is 0 for
//! `CircularQueue`'s Rexx `queue` and `Array`'s native `[]` alike, and
//! `~package~name` is `REXX` for both.
//!
//! **The token is a citation, not a body identity.** Two tokens can name one
//! C++ function -- `Set`'s `HasItem` is written `IdentityTable::hasIndexRexx`
//! and `IdentityTableClass.hpp` declares no such member -- and one token can
//! reach three behaviours chosen by the contents class the receiver
//! allocated. Count tokens for an upper bound on bodies and never for a count
//! of behaviours.

#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// One native method the interpreter defines: its token and arity operand.
pub type Native = (String, String);

/// Every `StartClassDefinition` block's instance and class tables, with
/// `InheritInstanceMethods` resolved.
///
/// `RemoveMethod` and `HideMethod` are deliberately not applied: a name they
/// remove is not at that scope on the oracle either, so a (scope, name) key
/// never reaches the copy they would have deleted. The module doc has why.
pub fn setup_tables() -> (
    HashMap<String, HashMap<String, Native>>,
    HashMap<String, HashMap<String, Native>>,
) {
    let path = PathBuf::from("/home/moritz/dev/repos/ooRexx/interpreter/memory/Setup.cpp");
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read the oracle's {}: {e}", path.display()));
    let mut instance: HashMap<String, HashMap<String, Native>> = HashMap::new();
    let mut class: HashMap<String, HashMap<String, Native>> = HashMap::new();
    let mut current: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        // The macros' own `#define`s name their parameters, not classes.
        if trimmed.starts_with('#') {
            continue;
        }
        if let Some(name) = between(trimmed, "StartClassDefinition(", ")") {
            current = Some(name.to_string());
            instance.entry(name.to_string()).or_default();
            class.entry(name.to_string()).or_default();
            continue;
        }
        let Some(here) = current.clone() else {
            continue;
        };
        if let Some(source) = between(trimmed, "InheritInstanceMethods(", ")") {
            let inherited = instance
                .get(source)
                .unwrap_or_else(|| {
                    panic!("InheritInstanceMethods({source}) names a block not yet defined")
                })
                .clone();
            let table = instance.entry(here).or_default();
            for (name, native) in inherited {
                table.entry(name).or_insert(native);
            }
            continue;
        }
        if let Some((name, native, is_class)) = add_method(trimmed) {
            let table = if is_class { &mut class } else { &mut instance };
            table.entry(here).or_default().insert(name, native);
        }
    }
    (instance, class)
}

fn between<'a>(line: &'a str, open: &str, close: &str) -> Option<&'a str> {
    let start = line.find(open)? + open.len();
    let rest = &line[start..];
    let end = rest.find(close)?;
    Some(&rest[..end])
}

/// `AddMethod("Name", Entry, arity)` and its Class/Protected/Private/Unguarded
/// spellings, as (uppercased name, (token, arity), is-class-method).
fn add_method(line: &str) -> Option<(String, Native, bool)> {
    let open = line.find("Method(")?;
    let prefix = &line[..open];
    let keyword = prefix.rfind("Add")?;
    let is_class = prefix[keyword..].contains("Class");
    let body = &line[open + "Method(".len()..];
    let close = body.rfind(");")?;
    let mut parts = body[..close].splitn(3, ',');
    let name = parts.next()?.trim();
    let token = parts.next()?.trim();
    let arity = parts.next()?.trim();
    if !name.starts_with('"') || !name.ends_with('"') || token.is_empty() || arity.is_empty() {
        return None;
    }
    Some((
        name.trim_matches('"').to_uppercase(),
        (token.to_string(), arity.to_string()),
        is_class,
    ))
}
