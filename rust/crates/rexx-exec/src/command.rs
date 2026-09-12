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

//! Issuing a command clause: the handler the `ADDRESS` environment names, the
//! child it runs, and the return code that becomes `RC`, `.RS` and a
//! condition.

use std::ffi::OsStr;
use std::io::Read;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::process::ExitStatusExt;
use std::process::Stdio;

use rexx_parse::{Expr, Instruction, ProgramSource};

use crate::error::{Failure, Raised};
use crate::run::Flow;
use crate::{Code, Interp};

/// `SYSSHELLPATH` for every unix but AIX
/// (`interpreter/platform/unix/PlatformDefinitions.h:66`-`:70`).
const SHELL_DIRECTORY: &str = "/bin";

/// `MAX_COMMAND_ARGS`: the most arguments `ADDRESS PATH` splits a command
/// into, beyond which the split fails
/// (`interpreter/platform/unix/SystemCommands.cpp:70`).
const MAX_COMMAND_ARGS: usize = 400;

/// `UNKNOWN_COMMAND`: what a shell answers for a command it could not find,
/// and the one non-zero code raising `FAILURE` rather than `ERROR`.
const UNKNOWN_COMMAND: i32 = 127;

/// `RXSUBCOM_NOTREG` (`api/rexxapidefs.h:91`): the return code for an
/// environment no handler is registered for.
const NOT_REGISTERED: i32 = 30;

/// What a command's return code did, and so what `.RS` answers.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum ReturnStatus {
    Normal,
    Error,
    Failure,
}

impl ReturnStatus {
    /// `.RS`'s own value -- measured 0 after `true`, 1 after `exit 3` and
    /// -1 after `exit 127`.
    pub(crate) fn code(self) -> i32 {
        match self {
            ReturnStatus::Normal => 0,
            ReturnStatus::Error => 1,
            ReturnStatus::Failure => -1,
        }
    }

    /// The condition name this status raises, or `None` where it raises
    /// none.
    pub(crate) fn condition(self) -> Option<&'static str> {
        match self {
            ReturnStatus::Normal => None,
            ReturnStatus::Error => Some("ERROR"),
            ReturnStatus::Failure => Some("FAILURE"),
        }
    }
}

/// What running one command left behind.
pub(crate) struct CommandOutcome {
    /// The return code `RC` is assigned from.
    pub(crate) rc: i32,
    pub(crate) status: ReturnStatus,
}

impl CommandOutcome {
    /// `ioCommandHandler`'s mapping of a return code onto a condition: 127
    /// alone is a `FAILURE`, every other non-zero an `ERROR`.
    fn of(rc: i32) -> CommandOutcome {
        let status = match rc {
            0 => ReturnStatus::Normal,
            UNKNOWN_COMMAND => ReturnStatus::Failure,
            _ => ReturnStatus::Error,
        };
        CommandOutcome { rc, status }
    }
}

/// What an environment name resolves to.
enum Handler {
    /// A shell, run as `<SHELL_DIRECTORY>/<name> -c <command>`.
    Shell(String),
    /// `PATH`: no shell, so the command string is split here.
    Path,
}

/// The handler registered for `environment`, or `None` for a name nothing
/// answers. `registerCommandHandlers`
/// (`interpreter/platform/unix/SystemCommands.cpp:1072`) is the registered
/// set; the lookup upcases, and the name an activation stores keeps the
/// spelling it was written with.
fn handler_for(environment: &[u8]) -> Option<Handler> {
    let upper = environment.to_ascii_uppercase();
    match upper.as_slice() {
        // `ioCommandHandler` maps these onto `sh` by name rather than by
        // lowercasing what it was given.
        b"" | b"COMMAND" | b"SYSTEM" => Some(Handler::Shell("sh".to_string())),
        b"SH" | b"KSH" | b"CSH" | b"BSH" | b"BASH" | b"TCSH" | b"ZSH" => Some(Handler::Shell(
            String::from_utf8_lossy(&upper).to_ascii_lowercase(),
        )),
        b"PATH" => Some(Handler::Path),
        _ => None,
    }
}

/// `scan_cmd` (`interpreter/platform/unix/SystemCommands.cpp:639`):
/// `ADDRESS PATH` has no shell, so it splits its own command string. Blanks
/// and tabs separate. A `"` opens a run that only a `"` **followed by
/// whitespace or the end** closes, so a quote inside a run does not end it.
/// `None` where the string holds more arguments than [`MAX_COMMAND_ARGS`].
fn scan_command(command: &[u8]) -> Option<Vec<Vec<u8>>> {
    let mut args: Vec<Vec<u8>> = Vec::new();
    let mut at = 0;
    while at < command.len() {
        while at < command.len() && matches!(command[at], b' ' | b'\t') {
            at += 1;
        }
        if at >= command.len() {
            break;
        }
        if args.len() == MAX_COMMAND_ARGS {
            return None;
        }
        let quoted = command[at] == b'"';
        if quoted {
            at += 1;
        }
        let start = at;
        let mut end = at;
        if quoted {
            while end < command.len()
                && !(end > start
                    && command[end - 1] == b'"'
                    && matches!(command[end], b' ' | b'\t'))
            {
                end += 1;
            }
        } else {
            while end < command.len() && !matches!(command[end], b' ' | b'\t') {
                end += 1;
            }
        }
        // The closing quote is dropped where the run has one; an
        // unterminated run keeps every byte it holds.
        let stop = if quoted && end > start && command[end - 1] == b'"' {
            end - 1
        } else {
            end
        };
        args.push(command[start..stop].to_vec());
        at = end + 1;
    }
    Some(args)
}

/// Whether `command` carries a redirection or sequencing character outside a
/// double-quoted run, which is what stops `handleCommandInternally`
/// (`interpreter/platform/unix/SystemCommands.cpp:721`) handling it here. A
/// backslash escapes the byte after it.
fn shell_metacharacter(command: &[u8]) -> bool {
    let mut in_quotes = false;
    let mut escape = false;
    for &byte in command {
        if escape {
            escape = false;
        } else if byte == b'\\' {
            escape = true;
        } else if byte == b'"' {
            in_quotes = !in_quotes;
        } else if !in_quotes && matches!(byte, b'<' | b'>' | b'|' | b'&' | b';') {
            return true;
        }
    }
    false
}

/// Which of the three environment-writing spellings a command used.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Assignment {
    Set,
    Unset,
    Export,
}

/// `st` after the keyword, with the blanks the C++ skips removed.
fn after_keyword(command: &[u8], keyword: usize) -> &[u8] {
    let rest = &command[keyword..];
    let at = rest
        .iter()
        .position(|byte| *byte != b' ')
        .unwrap_or(rest.len());
    &rest[at..]
}

/// `cd`, `set`, `unset` and `export` against this interpreter's own directory
/// and environment rather than in a child. `None` is "not handled here", which
/// hands the command to a shell. A leading blank or a quoted spelling is
/// deliberately not recognised, matching `handleCommandInternally`.
fn run_internally(interp: &mut Interp, command: &[u8]) -> Option<CommandOutcome> {
    if shell_metacharacter(command) {
        return None;
    }
    if command == b"cd" || command.starts_with(b"cd ") {
        return Some(change_directory(interp, command));
    }
    if command.starts_with(b"set ") {
        return assign_environment(interp, after_keyword(command, 3), Assignment::Set);
    }
    if command.starts_with(b"unset ") {
        return assign_environment(interp, after_keyword(command, 5), Assignment::Unset);
    }
    if command.starts_with(b"export ") {
        return assign_environment(interp, after_keyword(command, 6), Assignment::Export);
    }
    None
}

/// `unquote` (`interpreter/platform/unix/SystemCommands.cpp:470`): a wrapping
/// pair of double quotes is removed and everything else is kept.
fn unquote(text: &[u8]) -> Vec<u8> {
    if text.len() >= 2 && text.first() == Some(&b'"') && text.last() == Some(&b'"') {
        return text[1..text.len() - 1].to_vec();
    }
    text.to_vec()
}

/// `sys_process_cd`: moves this interpreter's own directory.
///
/// **A `chdir` that fails is still handled here** -- the C++ raises `ERROR`
/// with `errno` as the return code and answers `true`, so the command never
/// reaches a shell. Measured: `cd nosuchdir` is `rc 2`, `.RS 1`, the
/// directory unmoved and **nothing on stderr**.
fn change_directory(interp: &mut Interp, command: &[u8]) -> CommandOutcome {
    let rest = after_keyword(command, 2);
    let target = if rest.is_empty() {
        match interp.env_get(b"HOME") {
            Some(home) => home.to_vec(),
            // No `HOME` is the one shape the C++ hands back to the shell,
            // which this cannot express here; a shell would fail too.
            None => return CommandOutcome::of(1),
        }
    } else {
        unquote(rest)
    };
    let path = crate::paths::normalize(&String::from_utf8_lossy(&target), &interp.cwd_text());
    match std::fs::metadata(&path) {
        Ok(meta) if meta.is_dir() => {
            interp.set_cwd(std::path::PathBuf::from(path));
            CommandOutcome::of(0)
        }
        // `errno` is the return code, and `ENOTDIR` and `ENOENT` are the two
        // this can produce.
        Ok(_) => CommandOutcome::of(20),
        Err(error) => CommandOutcome::of(error.raw_os_error().unwrap_or(2)),
    }
}

/// Expands `$NAME` in the value half of an assignment against this
/// interpreter's environment, as `sys_process_export` does before storing it.
///
/// **A name ends only at `/`, `:`, `$` or the end of the value** -- not at a
/// blank and not at punctuation, so `$ZZSET-tail` looks up a variable called
/// `ZZSET-tail`. A name nothing answers contributes nothing at all, which is
/// why that example stores the empty string rather than the shell's
/// `two-tail`; measured, and the terminator itself is left in the stream for
/// the next segment to copy.
fn expand(interp: &Interp, value: &[u8]) -> Vec<u8> {
    // The C++ sets its `HitFlag` on seeing a `$` rather than on resolving
    // one, and copies the value untouched only when there is none.
    if !value.contains(&b'$') {
        return value.to_vec();
    }
    let mut out = Vec::with_capacity(value.len());
    let mut at = 0;
    while let Some(offset) = value[at..].iter().position(|byte| *byte == b'$') {
        let dollar = at + offset;
        out.extend_from_slice(&value[at..dollar]);
        let start = dollar + 1;
        let mut end = start;
        while end < value.len() && !matches!(value[end], b'/' | b':' | b'$') {
            end += 1;
        }
        if let Some(found) = interp.env_get(&value[start..end]) {
            out.extend_from_slice(found);
        }
        at = end;
    }
    if at < value.len() {
        out.extend_from_slice(&value[at..]);
    }
    out
}

/// `sys_process_export`: `set`, `unset` and `export` against this
/// interpreter's own environment. `None` hands the command to a shell.
fn assign_environment(
    interp: &mut Interp,
    rest: &[u8],
    assignment: Assignment,
) -> Option<CommandOutcome> {
    if rest.is_empty() && assignment != Assignment::Unset {
        return None;
    }
    let equals = rest.iter().position(|byte| *byte == b'=');
    if assignment == Assignment::Unset {
        // An `=` in an `unset` is handed on so that the shell produces its
        // own error message.
        if equals.is_some() {
            return None;
        }
        interp.env_set(rest, None);
        return Some(CommandOutcome::of(0));
    }
    let Some(equals) = equals else {
        // No assignment operator at all: the C++ treats this as no command,
        // except where a pipe or a redirection means a shell should see it.
        if rest.contains(&b'|') || rest.contains(&b'>') {
            return None;
        }
        return Some(CommandOutcome::of(0));
    };
    let value = expand(interp, &rest[equals + 1..]);
    interp.env_set(&rest[..equals], Some(value));
    Some(CommandOutcome::of(0))
}

/// The return code the oracle reports for a finished child: the exit status
/// where there is one, and the negated signal number where the child died of
/// one (`ioCommandHandler`'s `WIFEXITED`/`WTERMSIG` pair, whose `-1` clamp
/// this keeps).
fn exit_code(status: std::process::ExitStatus) -> i32 {
    match status.code() {
        Some(code) => code,
        None => {
            let rc = -status.signal().unwrap_or(0);
            if rc == 1 { -1 } else { rc }
        }
    }
}

/// Runs `command` in a child and appends what it wrote to this interpreter's
/// own sinks.
///
/// **Both pipes are drained concurrently**: a child filling one while this
/// thread reads only the other deadlocks once a pipe buffer fills. Standard
/// input is inherited, because the oracle spawns with no file actions until
/// an `ADDRESS ... WITH` asks for them.
fn spawn(interp: &mut Interp, handler: &Handler, command: &[u8]) -> CommandOutcome {
    let mut builder = match handler {
        Handler::Shell(shell) => {
            let mut builder = std::process::Command::new(format!("{SHELL_DIRECTORY}/{shell}"));
            builder.arg("-c").arg(OsStr::from_bytes(command));
            builder
        }
        Handler::Path => {
            let Some(args) = scan_command(command) else {
                return CommandOutcome::of(UNKNOWN_COMMAND);
            };
            let Some((program, rest)) = args.split_first() else {
                return CommandOutcome::of(UNKNOWN_COMMAND);
            };
            let mut builder = std::process::Command::new(OsStr::from_bytes(program));
            for argument in rest {
                builder.arg(OsStr::from_bytes(argument));
            }
            builder
        }
    };
    builder.env_clear();
    for (name, value) in &interp.env {
        builder.env(OsStr::from_bytes(name), OsStr::from_bytes(value));
    }
    builder.current_dir(interp.cwd_text());
    builder.stdout(Stdio::piped());
    builder.stderr(Stdio::piped());
    let Ok(mut running) = builder.spawn() else {
        return CommandOutcome::of(UNKNOWN_COMMAND);
    };
    let mut out = Vec::new();
    let stdout = running.stdout.take();
    let stderr = running.stderr.take();
    let err = std::thread::scope(|scope| {
        let reader = scope.spawn(|| {
            let mut bytes = Vec::new();
            if let Some(mut stderr) = stderr {
                let _ = stderr.read_to_end(&mut bytes);
            }
            bytes
        });
        if let Some(mut stdout) = stdout {
            let _ = stdout.read_to_end(&mut out);
        }
        reader.join().unwrap_or_default()
    });
    let rc = match running.wait() {
        Ok(status) => exit_code(status),
        Err(_) => UNKNOWN_COMMAND,
    };
    interp.write_out(&out);
    interp.write_err(&err);
    CommandOutcome::of(rc)
}

impl Interp {
    /// Runs one already-evaluated command string against the `ADDRESS`
    /// environment in force, answering its return code and status.
    ///
    /// An environment no handler is registered for runs nothing and answers
    /// [`NOT_REGISTERED`] with a `FAILURE` -- measured, the command is
    /// evaluated and never executed.
    pub(crate) fn run_command(
        &mut self,
        environment: &[u8],
        command: &[u8],
    ) -> Result<CommandOutcome, Failure> {
        let Some(handler) = handler_for(environment) else {
            return Ok(CommandOutcome {
                rc: NOT_REGISTERED,
                status: ReturnStatus::Failure,
            });
        };
        // `PATH` names no shell, so a `cd` under it is a program to find
        // rather than a directory to move to.
        if matches!(handler, Handler::Shell(_))
            && let Some(outcome) = run_internally(self, command)
        {
            return Ok(outcome);
        }
        Ok(spawn(self, &handler, command))
    }

    /// One command clause: evaluates the string, echoes what the setting
    /// asks, runs it, and settles `RC`, `.RS` and any condition.
    ///
    /// `environment` names the handler for this clause alone. `ADDRESS env
    /// command` leaves the activation's own pair untouched -- measured,
    /// `ADDRESS()` is unchanged afterwards and a later bare `ADDRESS`
    /// toggles to the alternate the one-off never wrote.
    pub(crate) fn exec_command(
        &mut self,
        code: &Code<'_>,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
        expression: &Expr,
        environment: Option<&[u8]>,
    ) -> Result<Flow, Failure> {
        let indent = self.clause_state.current_value_indent;
        let value = self.eval(code, expression)?;
        self.roots.push_temp(value);
        let value = self.required_string_value(value)?;
        let command = self.to_text(value).into_owned();
        // Before the command runs, and observably so: under `TRACE C` a
        // command writing to its own standard error shows this line first.
        self.trace_command_value(indent, &command);

        let name = match environment {
            Some(name) => name.to_vec(),
            None => self
                .activation()
                .address
                .current
                .as_deref()
                .unwrap_or(crate::builtin::state::DEFAULT_ENVIRONMENT)
                .to_vec(),
        };
        let outcome = self.run_command(&name, &command)?;

        // `RC` before anything else, which is where the C++ puts it too
        // (`RexxActivation::command`).
        let assigned = self.text(outcome.rc.to_string().as_bytes());
        self.assign_by_name(b"RC", assigned);

        let mode = self.trace_mode();
        let echoed = mode.all || mode.commands;
        let retrace = match outcome.status {
            ReturnStatus::Error => mode.errors,
            ReturnStatus::Failure => mode.failures,
            ReturnStatus::Normal => false,
        };
        if retrace
            && !echoed
            && let Some((line, text)) = self.clause_site(source, instruction)
        {
            self.trace_command_retrace(line, indent, &text, &command);
        }
        if (echoed || retrace) && outcome.rc != 0 {
            self.trace_command_rc(indent, outcome.rc);
        }
        self.activation_mut().rs = Some(outcome.status.code());

        if let Some(condition) = outcome.status.condition() {
            self.raise_command_condition(condition, &command, outcome.rc)?;
        }
        Ok(Flow::Next)
    }

    /// Offers `ERROR` or `FAILURE` to the traps in force.
    ///
    /// **An untrapped `FAILURE` is re-raised as `ERROR`** and an untrapped
    /// `ERROR` is silent -- measured, a command exiting 127 reaches a
    /// `SIGNAL ON ERROR` handler with no failure trap anywhere, and one
    /// exiting 3 reaches a lone `SIGNAL ON FAILURE` handler not at all.
    fn raise_command_condition(
        &mut self,
        condition: &'static str,
        command: &[u8],
        rc: i32,
    ) -> Result<(), Failure> {
        let rendered = rc.to_string().into_bytes();
        match self.trap_for(condition.as_bytes()) {
            Some(trap) if trap.call => {
                self.pending_traps.push_back(crate::PendingTrap {
                    condition: condition.as_bytes().into(),
                    rc: Some(rendered),
                    description: Some(command.to_vec()),
                    activation: self.activation().id,
                    queued_during_delivery: false,
                    fragment_depth: self.fragment_depth,
                });
                Ok(())
            }
            Some(_) => Err(Raised {
                rc: Some(rendered),
                description: Some(command.to_vec()),
                ..Raised::condition(std::borrow::Cow::Borrowed(condition))
            }
            .into()),
            None if condition == "FAILURE" => self.raise_command_condition("ERROR", command, rc),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `scan_cmd`'s own split, against the C++ loop's answers.
    #[test]
    fn path_splits_on_blanks_and_keeps_quoted_runs_whole() {
        let split = |text: &str| scan_command(text.as_bytes()).expect("under the argument cap");
        assert_eq!(
            split("echo one two"),
            [b"echo".to_vec(), b"one".to_vec(), b"two".to_vec()]
        );
        assert_eq!(split("a\tb"), [b"a".to_vec(), b"b".to_vec()]);
        assert_eq!(split("  a   b  "), [b"a".to_vec(), b"b".to_vec()]);
        assert_eq!(split("\"a b\" c"), [b"a b".to_vec(), b"c".to_vec()]);
        assert_eq!(split("\"a b\""), [b"a b".to_vec()]);
        // A quote that closes nothing keeps every byte after it.
        assert_eq!(split("\"a b"), [b"a b".to_vec()]);
        assert_eq!(split(""), Vec::<Vec<u8>>::new());
    }

    /// More arguments than the cap is a failed split, not a truncated one.
    #[test]
    fn a_command_past_the_argument_cap_does_not_split() {
        let text = vec![b'a'; MAX_COMMAND_ARGS * 2]
            .iter()
            .map(|byte| (*byte as char).to_string())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(scan_command(text.as_bytes()).is_none());
    }

    /// The guard that decides whether a command can be handled without a
    /// shell.
    #[test]
    fn a_metacharacter_outside_quotes_needs_a_shell() {
        assert!(!shell_metacharacter(b"cd sub"));
        assert!(shell_metacharacter(b"cd sub ; true"));
        assert!(shell_metacharacter(b"echo a | cat"));
        // Quoted and escaped ones do not count.
        assert!(!shell_metacharacter(b"echo \"a ; b\""));
        assert!(!shell_metacharacter(b"echo a \\; b"));
    }

    /// Every registered environment name, and the shell each resolves to.
    #[test]
    fn the_registered_environments_resolve_to_their_shells() {
        let shell = |name: &[u8]| match handler_for(name) {
            Some(Handler::Shell(shell)) => shell,
            _ => panic!("{} named no shell", String::from_utf8_lossy(name)),
        };
        for name in [b"".as_slice(), b"COMMAND", b"SYSTEM", b"sh", b"SH"] {
            assert_eq!(shell(name), "sh");
        }
        assert_eq!(shell(b"BASH"), "bash");
        assert_eq!(shell(b"tcsh"), "tcsh");
        assert!(matches!(handler_for(b"path"), Some(Handler::Path)));
        assert!(handler_for(b"nosuchhandler").is_none());
    }
}
