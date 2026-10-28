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

use super::rexxutil::{self, stems::MAX_WHOLENUMBER};
use super::{Args, arg, required_string};
use crate::environment::EnvScope;
use crate::error::Raised;
use crate::{Failure, Interp};

/// `QUALIFY(name)`: the name resolved against the interpreter's current
/// directory and reduced to one spelling, whether or not it exists
/// (`expression/BuiltinFunctions.cpp:2988` through
/// `SysFileSystem::qualifyStreamName`).
pub(crate) fn qualify(
    interp: &mut Interp,
    _name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let text = required_string(interp, args, 1);
    let qualified = qualify_name(interp, &text);
    Ok(interp.text_built(qualified))
}

/// `SysFileSystem::qualifyStreamName`: `name` up to its first NUL, as the C
/// string the C++ is given, with a leading `~` or `~user` resolved, made
/// absolute against the interpreter's current directory and reduced by
/// [`crate::paths::normalize_bytes`]. The null string where the C++ fails: an
/// empty name, a `~user` naming no account, or a home directory that is not
/// absolute.
pub(crate) fn qualify_name(interp: &Interp, name: &[u8]) -> Vec<u8> {
    let name = match name.iter().position(|byte| *byte == 0) {
        Some(end) => &name[..end],
        None => name,
    };
    if name.is_empty() {
        return Vec::new();
    }
    if name.first() == Some(&b'~') {
        return match expand_tilde(interp, name) {
            Some(expanded) if expanded.first() == Some(&b'/') => {
                crate::paths::normalize_bytes(&expanded, b"/")
            }
            _ => Vec::new(),
        };
    }
    crate::paths::normalize_bytes(name, interp.cwd_bytes())
}

/// `SysFileSystem::resolveTilde`: a leading `~` replaced by the environment's
/// `HOME`, a leading `~user` by that account's home directory, and `None`
/// where no account has that name. Accounts are read from `/etc/passwd`, so
/// one that lives only in NSS (LDAP and the like) is not found.
pub(crate) fn expand_tilde(interp: &Interp, name: &[u8]) -> Option<Vec<u8>> {
    let rest = &name[1..];
    if rest.is_empty() || rest[0] == b'/' {
        let mut out = interp.env_get(b"HOME").unwrap_or_default().to_vec();
        out.extend_from_slice(rest);
        return Some(out);
    }
    let slash = rest
        .iter()
        .position(|byte| *byte == b'/')
        .unwrap_or(rest.len());
    let mut out = passwd_home(&rest[..slash])?;
    out.extend_from_slice(&rest[slash..]);
    Some(out)
}

/// The home directory `/etc/passwd` gives the account `user`.
pub(crate) fn passwd_home(user: &[u8]) -> Option<Vec<u8>> {
    let passwd = std::fs::read("/etc/passwd").ok()?;
    passwd.split(|byte| *byte == b'\n').find_map(|line| {
        let fields: Vec<&[u8]> = line.split(|byte| *byte == b':').collect();
        (fields.len() > 5 && fields[0] == user).then(|| fields[5].to_vec())
    })
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

/// `DIRECTORY([new])`: a native routine of the `REXX` package, not a builtin
/// (`runtime/NativeFunctions.h:48`), which is why its arity errors are the
/// 88.9xx family. With an argument it moves the interpreter's own current
/// directory and answers the new one; a change it cannot make answers the null
/// string and moves nothing. With none it answers where it is.
pub(crate) fn directory(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    if args.len() > 1 {
        return Err(Raised::too_many_internal_arguments(1).into());
    }
    let Some(text) = rexxutil::optional_text(interp, args, 1)? else {
        let here = interp.cwd_bytes().to_vec();
        return Ok(interp.text_built(here));
    };
    if text.is_empty() {
        return Ok(interp.text(b""));
    }
    let qualified = qualify_name(interp, &text);
    let candidate = crate::paths::os_path(&qualified);
    if !std::fs::metadata(candidate).is_ok_and(|meta| meta.is_dir()) {
        return Ok(interp.text(b""));
    }
    // `getcwd` answers the symlink-resolved path, so the answer is what the
    // filesystem calls the directory rather than the way it was named.
    let resolved = std::fs::canonicalize(candidate).unwrap_or_else(|_| candidate.to_path_buf());
    let answer = resolved.as_os_str().as_encoded_bytes().to_vec();
    interp.set_cwd(resolved);
    Ok(interp.text_built(answer))
}

/// The `FILESPEC` options, in the order the oracle's own message lists them:
/// `DELNP`.
const FILESPEC_OPTIONS: &[u8] = b"DELNP";

/// `FILESPEC(option, name)`: the piece of `name` that `option`'s first letter
/// selects (`runtime/InternalPackage.cpp:87`). Purely textual -- the file need
/// not exist, and nothing here touches the file system. On unix the drive is
/// always the null string.
pub(crate) fn filespec(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    if args.len() > 2 {
        return Err(Raised::too_many_internal_arguments(2).into());
    }
    rexxutil::required(args, 1)?;
    rexxutil::required(args, 2)?;
    let option = rexxutil::required_c_string(interp, args, 1)?;
    let letter = option.first().map(u8::to_ascii_uppercase);
    let Some(letter) = letter.filter(|byte| FILESPEC_OPTIONS.contains(byte)) else {
        return Err(Raised::argument_not_in_list(
            b"FILESPEC",
            1,
            &String::from_utf8_lossy(FILESPEC_OPTIONS),
            &option,
        )
        .into());
    };
    let spec = rexxutil::required_c_string(interp, args, 2)?;
    let cut = spec.iter().rposition(|byte| *byte == b'/');
    let answer: Vec<u8> = match letter {
        // Unix has no drive letter, and the oracle answers the null string.
        b'D' => Vec::new(),
        b'L' | b'P' => match cut {
            None => Vec::new(),
            Some(at) => spec[..=at].to_vec(),
        },
        b'N' => match cut {
            None => spec.clone(),
            Some(at) => spec[at + 1..].to_vec(),
        },
        // What follows the last dot of the *name*, wherever that dot sits:
        // measured, `filespec('E','/a/.hidden')` is `hidden`, not the null
        // string. `.File~extension` has the leading-dot rule this does not.
        b'E' => {
            let name = match cut {
                None => &spec[..],
                Some(at) => &spec[at + 1..],
            };
            match name.iter().rposition(|byte| *byte == b'.') {
                None => Vec::new(),
                Some(at) => name[at + 1..].to_vec(),
            }
        }
        other => unreachable!("{other} is not one of the options just checked"),
    };
    Ok(interp.text_built(answer))
}

/// `BEEP`'s bounds (`runtime/InternalPackage.cpp:160`-`:163`).
const MIN_FREQUENCY: i64 = 37;
const MAX_FREQUENCY: i64 = 32767;
const MIN_DURATION: i64 = 0;
const MAX_DURATION: i64 = 60000;

/// `BEEP(frequency, duration)`: writes one bell byte and answers the null
/// string. The oracle sounds the speaker through `SysProcess::beep` and, on
/// this platform, that reaches the terminal as a single `0x07` on standard
/// output -- measured, `beep(440,10)` writes it and answers `''`.
pub(crate) fn beep(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    if args.len() > 2 {
        return Err(Raised::too_many_internal_arguments(2).into());
    }
    // Both conversions, then frequency's range before duration's, and all
    // before the bell: `beep(1,1)` raises on the frequency and sounds nothing.
    let frequency = whole_number(interp, args, 1)?;
    let duration = whole_number(interp, args, 2)?;
    in_range(frequency, "frequency", MIN_FREQUENCY, MAX_FREQUENCY)?;
    in_range(duration, "duration", MIN_DURATION, MAX_DURATION)?;
    interp.write_out(&[0x07]);
    Ok(interp.text(b""))
}

/// A required `wholenumber_t` argument as the routine's marshalling converts
/// it (`NativeActivation::signedIntegerValue`): 88.901 where it is omitted,
/// 88.907 over the whole-number range where it is not a whole number.
fn whole_number(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    position: usize,
) -> Result<i64, Failure> {
    let value = rexxutil::required(args, position)?;
    if let Some(number) = rexxutil::stems::whole_value(interp, value, -MAX_WHOLENUMBER)? {
        return Ok(number);
    }
    let found = interp.string_value_text(value);
    Err(rexxutil::marshalled(Raised::native_argument_out_of_range(
        &position.to_string(),
        -MAX_WHOLENUMBER,
        MAX_WHOLENUMBER,
        &found,
    )))
}

/// `BEEP`'s own range check of a converted argument, which names it.
fn in_range(number: i64, argument: &str, minimum: i64, maximum: i64) -> Result<(), Failure> {
    if (minimum..=maximum).contains(&number) {
        return Ok(());
    }
    Err(Raised::native_argument_out_of_range(
        argument,
        minimum,
        maximum,
        number.to_string().as_bytes(),
    )
    .into())
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
    // The write allocates, and `old` may be a fresh `.NAME` string nothing
    // else holds.
    interp.roots.activity_mut().push_temp(old);
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

#[cfg(test)]
mod tests {
    /// **A `~` with `HOME` unset qualifies to the null string, and `~/x` to
    /// `/x`**, the empty home read as no directory at all. The oracle runs
    /// `strlen` on the null `getenv("HOME")` there (`corpus/oracle-crashes.txt`
    /// entry 48), so this is the crate's own answer. The adjacent success is
    /// the same names with `HOME` set.
    #[test]
    fn a_tilde_with_home_unset_qualifies_without_a_home() {
        let source = b"call value 'HOME', '/h', 'ENVIRONMENT'\n\
            say qualify('~') qualify('~/x')\n\
            'unset HOME'\n\
            say '['qualify('~')']' qualify('~/x')\n";
        let outcome = crate::run_program("/t.rex", source.to_vec(), crate::Invocation::none());
        assert_eq!(outcome.exit_code, 0);
        assert_eq!(String::from_utf8_lossy(&outcome.stdout), "/h /h/x\n[] /x\n");
    }
}
