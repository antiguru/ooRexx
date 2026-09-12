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

//! The builtins that read or move the interpreter's own platform state:
//! `QUALIFY`, `USERID`, `SETLOCAL`, `ENDLOCAL`, and the external-pool halves of
//! `VALUE`. Every one of them goes through `Interp`'s environment and current
//! directory rather than the process's -- see `tests/no_process_state.rs`.

use rexx_core::ObjRef;

use super::{Args, arg, required_string};
use crate::environment::EnvScope;
use crate::error::Raised;
use crate::{Failure, Interp};

/// `QUALIFY(name)`: the name resolved against the interpreter's current
/// directory and reduced to one spelling, whether or not it exists
/// (`expression/BuiltinFunctions.cpp:2988` through
/// `SysFileSystem::qualifyStreamName`). An empty name answers the null string,
/// because `canonicalizeName` refuses it and the caller resets to `""`.
pub(crate) fn qualify(
    interp: &mut Interp,
    _name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let text = required_string(interp, args, 1);
    if text.is_empty() {
        return Ok(interp.text(b""));
    }
    let expanded = expand_tilde(interp, &text);
    let cwd = interp.cwd_text();
    let qualified = crate::paths::normalize(&String::from_utf8_lossy(&expanded), &cwd);
    Ok(interp.text_built(qualified.into_bytes()))
}

/// A leading `~` or `~/` replaced by the environment's `HOME`, as
/// `SysFileSystem::resolveTilde` does. `~user` is left alone: it needs the
/// passwd entry of another account, which nothing in this phase reads.
fn expand_tilde(interp: &Interp, name: &[u8]) -> Vec<u8> {
    if name.first() != Some(&b'~') {
        return name.to_vec();
    }
    let rest = &name[1..];
    if !rest.is_empty() && rest.first() != Some(&b'/') {
        return name.to_vec();
    }
    match interp.env_get(b"HOME") {
        None => name.to_vec(),
        Some(home) => {
            let mut out = home.to_vec();
            out.extend_from_slice(rest);
            out
        }
    }
}

/// `USERID()`: the effective user's login name
/// (`platform/unix/UseridFunction.cpp:58`, `getpwuid(geteuid())`). Read from
/// `/proc/self/status` and `/etc/passwd` rather than through libc, because
/// every libc route is an `unsafe` call. An account that lives only in NSS
/// (LDAP and the like) is not in `/etc/passwd`, and answers the environment's
/// own `USER` or `LOGNAME` instead; the oracle would answer the account name.
pub(crate) fn userid(
    interp: &mut Interp,
    _name: &'static [u8],
    _args: Args<'_>,
) -> Result<ObjRef, Failure> {
    if let Some(name) = passwd_name(effective_uid()) {
        return Ok(interp.text_built(name));
    }
    for fallback in [b"USER".as_slice(), b"LOGNAME".as_slice()] {
        if let Some(value) = interp.env_get(fallback) {
            let value = value.to_vec();
            return Ok(interp.text_built(value));
        }
    }
    Ok(interp.text(b""))
}

/// The effective uid, from `/proc/self/status`'s `Uid:` line, whose second
/// field is the effective one.
fn effective_uid() -> Option<u32> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|line| line.starts_with("Uid:"))?;
    line.split_whitespace().nth(2)?.parse().ok()
}

/// The login name `/etc/passwd` gives `uid`.
fn passwd_name(uid: Option<u32>) -> Option<Vec<u8>> {
    let uid = uid?;
    let passwd = std::fs::read_to_string("/etc/passwd").ok()?;
    for line in passwd.lines() {
        let mut fields = line.split(':');
        let name = fields.next()?;
        let _password = fields.next();
        let Some(entry) = fields.next() else { continue };
        if entry.parse::<u32>() == Ok(uid) {
            return Some(name.as_bytes().to_vec());
        }
    }
    None
}

/// `SETLOCAL()`: saves the directory and the environment, answering 1.
pub(crate) fn setlocal(
    interp: &mut Interp,
    _name: &'static [u8],
    _args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let saved = interp.push_local_environment();
    Ok(interp.counted(usize::from(saved)))
}

/// `ENDLOCAL()`: restores the innermost saved pair, answering 1 when there was
/// one and 0 when there was not.
///
/// **The oracle crashes past the end and this crate does not**
/// (`corpus/oracle-crashes.txt` entry 10): an `ENDLOCAL` with nothing
/// outstanding dereferences null there once any `SETLOCAL` has run, and a
/// second restore in one process aborts in the allocator. Answering 0 is what
/// the oracle itself answers before its first `SETLOCAL`, so that is the answer
/// here, and no differential case may run either program.
pub(crate) fn endlocal(
    interp: &mut Interp,
    _name: &'static [u8],
    _args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let restored = interp.pop_local_environment();
    Ok(interp.counted(usize::from(restored)))
}

/// `VALUE(name, [new], selector)`: the two external pools this phase answers.
/// The empty selector names `.environment` itself; `ENVIRONMENT`, caselessly,
/// names the process environment this interpreter holds. Any other selector is
/// 40.914, which is also what the oracle answers on unix for every selector its
/// own `valueFunction` declines (`platform/unix/ValueFunction.cpp:66`).
pub(crate) fn value_selector(
    interp: &mut Interp,
    args: Args<'_>,
    selector: &[u8],
) -> Result<ObjRef, Failure> {
    let name = required_string(interp, args, 1);
    let new = arg(args, 2);
    if selector.is_empty() {
        return environment_directory(interp, &name, new);
    }
    if selector.eq_ignore_ascii_case(b"ENVIRONMENT") {
        return process_environment(interp, &name, new);
    }
    Err(Raised::syntax(40, 914, vec![selector.to_vec()]).into())
}

/// The `''` selector: `.environment`'s own entry, under the upper-cased name
/// the directory stores. A name it does not hold reads as `.NAME` -- the same
/// answer `.NAME` itself gives, because that is the path it takes.
fn environment_directory(
    interp: &mut Interp,
    name: &[u8],
    new: Option<ObjRef>,
) -> Result<ObjRef, Failure> {
    let upper = name.to_ascii_uppercase();
    let mut dotted = Vec::with_capacity(upper.len() + 1);
    dotted.push(b'.');
    dotted.extend_from_slice(&upper);
    let old = interp.dot_variable(&dotted)?;
    if let Some(value) = new {
        interp.set_directory_entry(EnvScope::Environment, &upper, value)?;
    }
    Ok(old)
}

/// The `ENVIRONMENT` selector: the interpreter's own environment. A name it
/// does not hold reads as the null string; `.nil` removes; any other value is
/// stored as its string, truncated at the first NUL because `setenv` takes a
/// C string and the oracle's own read-back stops there.
fn process_environment(
    interp: &mut Interp,
    name: &[u8],
    new: Option<ObjRef>,
) -> Result<ObjRef, Failure> {
    let old = interp.env_get(name).unwrap_or_default().to_vec();
    if let Some(value) = new {
        if value == ObjRef::NIL {
            interp.env_set(name, None);
        } else {
            let text = interp.to_text(value).into_owned();
            let stored = match text.iter().position(|byte| *byte == 0) {
                Some(at) => text[..at].to_vec(),
                None => text,
            };
            interp.env_set(name, Some(stored));
        }
    }
    Ok(interp.text_built(old))
}
