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
use std::rc::Rc;

use rexx_parse::{AddressIo, Expr, Instruction, ProgramSource};

use rexx_core::ObjRef;

use rexx_api::load::CommandHandler;
use rexx_api::redirect::Redirector;

use crate::error::{Failure, Raised};
use crate::redirect::IoContext;
use crate::run::Flow;
use crate::scheduler::{ExecOutcome, Scheduler};
use crate::security::{key, message};
use crate::{Code, Interp};

/// A return code's whole-number value under the default precision, or `None`
/// where it has none -- `RexxObject::numberValue(wholenumber_t &)`, which is
/// what the `+++ "RC(n)"` line is gated on.
fn whole_value(text: &[u8]) -> Option<i32> {
    let number = rexx_num::Number::parse(std::str::from_utf8(text).ok()?)?;
    i32::try_from(number.whole_value(9)?).ok()
}

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
    /// The return code's whole-number value, or `0` for one that is not a
    /// whole number: what the `+++ "RC(n)"` line is gated on and what a
    /// condition carries.
    pub(crate) rc: i32,
    pub(crate) status: ReturnStatus,
    /// The object a security manager or a registered handler supplied,
    /// which is assigned to `RC` verbatim and rendered by the trace line.
    /// `None` for a command a shell ran, whose code is
    /// [`CommandOutcome::rc`].
    pub(crate) supplied: Option<Supplied>,
    /// The condition a registered handler raised, which the command raises
    /// in place of the one its status would.
    pub(crate) condition: Option<Box<HandlerCondition>>,
}

/// A condition a registered handler raised through its thread context, as
/// `RexxActivation::command` finds it (`execution/RexxActivation.cpp`).
pub(crate) struct HandlerCondition {
    pub(crate) name: Vec<u8>,
    pub(crate) description: Option<Vec<u8>>,
    pub(crate) additional: Option<ObjRef>,
    pub(crate) result: Option<ObjRef>,
}

/// The `RC` a security manager set, as both halves are needed: the object
/// the variable is assigned, and the text the `+++ "RC(n)"` line renders.
pub(crate) struct Supplied {
    pub(crate) object: ObjRef,
    pub(crate) text: Vec<u8>,
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
        CommandOutcome {
            rc,
            status,
            supplied: None,
            condition: None,
        }
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
        return change_directory(interp, command);
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

/// `unquote` (`interpreter/platform/unix/SystemCommands.cpp:453`): every `"`
/// dropped, and a backslash dropped with the byte after it kept. A trailing
/// backslash ends the text.
fn unquote(text: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len());
    let mut bytes = text.iter();
    while let Some(&byte) = bytes.next() {
        match byte {
            b'\\' => match bytes.next() {
                Some(&escaped) => out.push(escaped),
                None => break,
            },
            b'"' => {}
            _ => out.push(byte),
        }
    }
    out
}

/// `sys_process_cd` (`interpreter/platform/unix/SystemCommands.cpp:494`): the
/// target after the leading spaces, unquoted, is where this interpreter's own
/// directory moves. Nothing is the home directory. `~`, `~/rest` and `~ rest`
/// are `HOME` with a `/` after it, the rest kept only after `~/`. `~user` and
/// `~user/rest` are that account's home directory. `None` hands the command
/// to the shell, as the C++ does where `HOME` is unset or the account is
/// unknown. With `HOME` unset the oracle dies on `cd ~`
/// (`corpus/oracle-crashes.txt` entry 44), and that goes to the shell too.
///
/// The directory is resolved as `chdir` resolves it, `..` after any link, and
/// the new directory is its physical path, as `getcwd` answers. A failure is
/// still handled here: `rc` is the `errno` and nothing reaches stderr.
fn change_directory(interp: &mut Interp, command: &[u8]) -> Option<CommandOutcome> {
    let rest = after_keyword(command, 2);
    let home = || interp.env_get(b"HOME").map(<[u8]>::to_vec);
    let target = if rest.is_empty() {
        home()?
    } else if rest[0] == b'~' && matches!(rest.get(1), None | Some(b'/' | b' ')) {
        let mut target = home()?;
        target.push(b'/');
        if rest.get(1) == Some(&b'/') {
            target.extend_from_slice(&rest[2..]);
        }
        target
    } else if let Some(account) = rest.strip_prefix(b"~") {
        let (user, tail) = match account.iter().position(|byte| *byte == b'/') {
            Some(slash) => (&account[..slash], &account[slash + 1..]),
            None => (account, &b""[..]),
        };
        let mut target = crate::builtin::platform::passwd_home(user)?;
        target.push(b'/');
        target.extend_from_slice(tail);
        target
    } else {
        rest.to_vec()
    };
    let target = unquote(&target);
    let target = match target.iter().position(|byte| *byte == 0) {
        Some(end) => &target[..end],
        None => &target[..],
    };
    Some(CommandOutcome::of(
        match physical_directory(interp, target) {
            Ok(directory) => {
                interp.set_cwd(directory);
                0
            }
            Err(errno) => errno,
        },
    ))
}

/// The directory `chdir(target)` would enter, by its physical path, or the
/// `errno` it would fail with.
fn physical_directory(interp: &Interp, target: &[u8]) -> Result<std::path::PathBuf, i32> {
    const ENOENT: i32 = 2;
    const ENOTDIR: i32 = 20;
    if target.is_empty() {
        return Err(ENOENT);
    }
    let joined = if target[0] == b'/' {
        target.to_vec()
    } else {
        [interp.cwd_bytes(), b"/", target].concat()
    };
    let directory = std::fs::canonicalize(OsStr::from_bytes(&joined))
        .map_err(|error| error.raw_os_error().unwrap_or(ENOENT))?;
    if !directory.is_dir() {
        return Err(ENOTDIR);
    }
    rustix::fs::access(&directory, rustix::fs::Access::EXEC_OK)
        .map_err(|errno| errno.raw_os_error())?;
    Ok(directory)
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

/// The `SYNTAX` error the `::OPTIONS` for `condition` makes of it: 98.971
/// for `FAILURE`, the `LOSTDIGITS`, `NOSTRING` and `NOTREADY` ones for a
/// handler's condition of that name (`Activity::raiseCondition`,
/// `concurrency/Activity.cpp:596-620`), and 98.970 for any other.
fn escalated(condition: &[u8], description: &[u8], rc: &[u8]) -> Raised {
    match condition {
        b"FAILURE" => Raised::failure_syntax(description, rc),
        b"LOSTDIGITS" => Raised::lostdigits(description),
        b"NOSTRING" => Raised::nostring_syntax(description),
        b"NOTREADY" => Raised::notready_syntax(description),
        _ => Raised::error_syntax(description, rc),
    }
}

/// A command clause's run: its outcome, or `Left` with its child's wait off
/// the baton under the token its end is posted with.
pub(crate) enum Commanded {
    Done(CommandOutcome),
    Left(u64),
}

/// What one child left behind: its return code and each stream it wrote.
pub(crate) struct Waited {
    rc: i32,
    out: Vec<u8>,
    err: Vec<u8>,
}

/// A command's child, whose wait runs off the baton
/// ([`crate::scheduler::Scheduler::exit_for_block`]).
pub(crate) struct Block {
    running: std::process::Child,
    input: Option<Vec<u8>>,
    merged: Option<std::io::PipeReader>,
    /// Whether standard output (or both streams, merged) goes to a target.
    captures_output: bool,
    /// Whether standard error goes to a target of its own.
    captures_error: bool,
}

/// Where a piece a child off the baton wrote goes.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum Piece {
    /// This interpreter's standard output.
    Out,
    /// This interpreter's standard error.
    Err,
    /// The output target, written by the parked clause as it arrives.
    Captured,
}

impl Block {
    /// Waits for the child, handing each piece it writes to `written` as it
    /// arrives, except standard error going to a target, which the answer
    /// holds whole (the oracle's error reader writes its target once the
    /// output has ended).
    pub(crate) fn stream(self, written: &(dyn Fn(Piece, Vec<u8>) + Sync)) -> Waited {
        let out = match self.captures_output {
            true => Piece::Captured,
            false => Piece::Out,
        };
        let err = match self.captures_error {
            true => None,
            false => Some(Piece::Err),
        };
        let input = self.input;
        collect(self.running, input, self.merged, Some((written, out, err)))
    }
}

/// A child's pieces' destination: the sink and where standard output and
/// standard error go, `None` for standard error kept whole.
type Routes<'a> = (&'a (dyn Fn(Piece, Vec<u8>) + Sync), Piece, Option<Piece>);

/// Reads `reader` to its end, into `kept`, or piece by piece into `written`
/// as `piece` where there is one.
fn read_all(
    mut reader: impl Read,
    kept: &mut Vec<u8>,
    written: Option<(&(dyn Fn(Piece, Vec<u8>) + Sync), Piece)>,
) {
    let Some((written, as_piece)) = written else {
        let _ = reader.read_to_end(kept);
        return;
    };
    let mut piece = [0u8; 4096];
    loop {
        match reader.read(&mut piece) {
            Ok(0) => return,
            Ok(count) => written(as_piece, piece[..count].to_vec()),
            Err(failed) if failed.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => return,
        }
    }
}

/// A command clause whose child runs off the baton, on its activity's record
/// until the clause runs again with what the child left.
pub(crate) struct Blocked {
    command: Vec<u8>,
    indent: usize,
    token: u64,
    io: Option<IoContext>,
    /// Output pieces that arrived for the output target and are not yet
    /// written.
    captured: Vec<u8>,
    /// The first condition a target raised, which the clause raises once
    /// the child has ended.
    raised: Option<Failure>,
    ended: Option<Waited>,
}

impl Blocked {
    /// Records an output piece for the target, under the token the wait was
    /// left with; answers whether it was this wait's.
    pub(crate) fn capture(&mut self, token: u64, bytes: Vec<u8>) -> bool {
        if token != self.token || self.ended.is_some() {
            return false;
        }
        self.captured.extend_from_slice(&bytes);
        true
    }

    /// Every object the redirection's targets hold.
    pub(crate) fn object_roots(&self, out: &mut Vec<ObjRef>) {
        if let Some(io) = &self.io {
            io.object_roots(out);
        }
    }

    /// Records what the child left, under the token the wait was left with;
    /// answers whether it was this wait's. A clause whose target write
    /// raised has left its wait, and the next command's wait has a token of
    /// its own.
    pub(crate) fn end(&mut self, token: u64, ended: Waited) -> bool {
        if token != self.token {
            return false;
        }
        self.ended = Some(ended);
        true
    }

    /// Ends the wait for a child still running, which a halt abandons there,
    /// answering its token: `waitpid` interrupted leaves the oracle's status
    /// unset, and this answers [`INTERRUPTED`].
    pub(crate) fn abandon(&mut self) -> Option<u64> {
        if self.ended.is_some() {
            return None;
        }
        self.ended = Some(Waited {
            rc: INTERRUPTED,
            out: Vec::new(),
            err: Vec::new(),
        });
        Some(self.token)
    }
}

/// The return code of a command whose wait a halt abandoned: `EINTR`, negated
/// as a signal's code is. DEVIATIONS entry 10.
const INTERRUPTED: i32 = -4;

/// A started child, or what a command that started none answers.
enum Child {
    Ran(Waited),
    Running {
        running: std::process::Child,
        input: Option<Vec<u8>>,
        merged: Option<std::io::PipeReader>,
    },
}

/// Starts `command` in a child.
///
/// Standard input is inherited unless `io` supplies one, which is the
/// oracle's own rule -- it spawns with no file actions until an `ADDRESS ...
/// WITH` asks for them.
///
/// `switch_mode`, where there is one, is the child's `REXX_SWITCH_MODE`.
fn start(
    interp: &Interp,
    handler: &Handler,
    command: &[u8],
    io: Option<&IoContext>,
    switch_mode: Option<String>,
) -> Result<Child, Failure> {
    let nothing = |rc| {
        Child::Ran(Waited {
            rc,
            out: Vec::new(),
            err: Vec::new(),
        })
    };
    let mut builder = match handler {
        Handler::Shell(shell) => {
            let mut builder = std::process::Command::new(format!("{SHELL_DIRECTORY}/{shell}"));
            builder.arg("-c").arg(OsStr::from_bytes(command));
            builder
        }
        Handler::Path => {
            let Some(args) = scan_command(command) else {
                return Ok(nothing(UNKNOWN_COMMAND));
            };
            let Some((program, rest)) = args.split_first() else {
                return Ok(nothing(UNKNOWN_COMMAND));
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
    if let Some(mode) = switch_mode {
        builder.env("REXX_SWITCH_MODE", mode);
        builder.env(crate::SIM_CHILD_ENV, "1");
        // A group of its own, which a refused wait kills whole.
        std::os::unix::process::CommandExt::process_group(&mut builder, 0);
    }
    builder.current_dir(interp.cwd_text());
    let input = io.and_then(IoContext::input_bytes);
    if input.is_some() {
        builder.stdin(Stdio::piped());
    }
    // **Output and error on one target share one pipe**, which is the
    // `adddup2` of the child's standard error onto its standard output the
    // unix handler performs before the spawn. Two pipes read separately
    // could not put the two streams back in the order the child wrote them.
    let merged = match io.is_some_and(IoContext::shares_one_target) {
        true => Some(std::io::pipe().map_err(|error| pipe_failed(&error))?),
        false => None,
    };
    match &merged {
        Some((_, writer)) => {
            let out = writer.try_clone().map_err(|error| pipe_failed(&error))?;
            let err = writer.try_clone().map_err(|error| pipe_failed(&error))?;
            builder.stdout(Stdio::from(out));
            builder.stderr(Stdio::from(err));
        }
        None => {
            builder.stdout(Stdio::piped());
            builder.stderr(Stdio::piped());
        }
    }
    let Ok(running) = builder.spawn() else {
        return Ok(nothing(UNKNOWN_COMMAND));
    };
    // **Every writing half of a merged pipe closes here**, and there are
    // three: the one this scope holds and the two the builder still owns,
    // which the spawn duplicated rather than consumed. The read below waits
    // for end-of-file, and any one of them left open never gives it.
    drop(builder);
    Ok(Child::Running {
        running,
        input,
        merged: merged.map(|(reader, _writer)| reader),
    })
}

/// Collects what `running` writes and its return code, writing `input` to
/// it first where there is one.
///
/// **Both pipes are drained concurrently**: a child filling one while this
/// thread reads only the other deadlocks once a pipe buffer fills.
fn collect(
    mut running: std::process::Child,
    input: Option<Vec<u8>>,
    merged: Option<std::io::PipeReader>,
    routes: Option<Routes<'_>>,
) -> Waited {
    let stdin = running.stdin.take();
    let stdout = running.stdout.take();
    let stderr = running.stderr.take();
    let mut out = Vec::new();
    let out_route = routes.map(|(written, out, _)| (written, out));
    let err_route = routes.and_then(|(written, _, err)| Some((written, err?)));
    let err = std::thread::scope(|scope| {
        if let (Some(mut stdin), Some(bytes)) = (stdin, input.as_deref()) {
            // A write error is dropped: a child exiting before it reads its
            // input gives `EPIPE`, which the oracle's own writer ignores.
            scope.spawn(move || {
                use std::io::Write;
                let _ = stdin.write_all(bytes);
            });
        }
        let collector = scope.spawn(|| {
            let mut bytes = Vec::new();
            if let Some(stderr) = stderr {
                read_all(stderr, &mut bytes, err_route);
            }
            bytes
        });
        match merged {
            Some(merged) => read_all(merged, &mut out, out_route),
            None => {
                if let Some(stdout) = stdout {
                    read_all(stdout, &mut out, out_route);
                }
            }
        }
        collector.join().unwrap_or_default()
    });
    let rc = match running.wait() {
        Ok(status) => exit_code(status),
        Err(_) => UNKNOWN_COMMAND,
    };
    Waited { rc, out, err }
}

/// 98.923, worded from the system's own description of the failure.
fn pipe_failed(error: &std::io::Error) -> Failure {
    Raised::redirection_failed(&error.to_string()).into()
}

impl Interp {
    /// Runs one already-evaluated command string against the `ADDRESS`
    /// environment in force, answering its return code and status.
    ///
    /// A handler an extension registered answers first, a built-in name's
    /// included, as `InterpreterInstance::resolveCommandHandler` finds it. An
    /// environment no handler is registered for runs nothing and answers
    /// [`NOT_REGISTERED`] with a `FAILURE` -- measured, the command is
    /// evaluated and never executed. An environment a held native package's
    /// loader registers through the classic subcommand API is refused instead
    /// ([`crate::Loud::loader_environment`]).
    ///
    /// `io`'s targets are begun before the handler runs
    /// (`CommandHandler::call`), so a target that cannot be begun stops the
    /// command before it starts.
    ///
    /// A child's wait leaves the baton, answering `Left`, outside a park's
    /// continuation (`NativeActivation::run`,
    /// `execution/NativeActivation.cpp:1691`, releases the kernel lock around
    /// a command handler), where a halt can abandon it (ruling P60).
    pub(crate) fn run_command(
        &mut self,
        environment: &[u8],
        command: &[u8],
        io: Option<&IoContext>,
    ) -> Result<Commanded, Failure> {
        let registered = self
            .command_handlers
            .get(environment.to_ascii_uppercase().as_slice())
            .map(Rc::clone);
        if let Some(handler) = registered {
            return self
                .run_registered_command(&handler, environment, command, io)
                .map(Commanded::Done);
        }
        let Some(handler) = handler_for(environment) else {
            if self.libraries.classic_environment(environment) {
                return Err(crate::Loud::loader_environment(environment).into());
            }
            return Ok(Commanded::Done(CommandOutcome {
                rc: NOT_REGISTERED,
                status: ReturnStatus::Failure,
                supplied: None,
                condition: None,
            }));
        };
        // `PATH` names no shell, so a `cd` under it is a program to find
        // rather than a directory to move to.
        //
        // **A redirected command never takes this path either.**
        // `handleCommandInternally` sits in the branch `ioCommandHandler`
        // reaches only when nothing is redirected, so a `cd` issued under an
        // `ADDRESS ... WITH` is a child like any other command and moves no
        // directory of this interpreter's.
        if io.is_none()
            && matches!(handler, Handler::Shell(_))
            && let Some(outcome) = run_internally(self, command)
        {
            return Ok(Commanded::Done(outcome));
        }
        if let Some(context) = io {
            context.begin(self)?;
        }
        self.hand_over_output();
        let switch_mode = self.child_switch_mode();
        let spawned = match start(self, &handler, command, io, switch_mode)? {
            Child::Ran(spawned) => spawned,
            Child::Running {
                running,
                input,
                merged,
            } if !self.activity.resuming => {
                let block = Block {
                    running,
                    input,
                    merged,
                    captures_output: io.is_some_and(IoContext::redirects_output),
                    captures_error: io.is_some_and(|context| {
                        context.redirects_error() && !context.shares_one_target()
                    }),
                };
                match self.exit_for_block(block) {
                    Ok(token) => return Ok(Commanded::Left(token)),
                    Err(block) => {
                        self.collect_on_baton(block.running, block.input, block.merged)?
                    }
                }
            }
            Child::Running {
                running,
                input,
                merged,
            } => self.collect_on_baton(running, input, merged)?,
        };
        self.command_ended(io, &spawned, None).map(Commanded::Done)
    }

    /// [`collect`] on the baton. In the simulation mode no other activity
    /// runs meanwhile, so a wait past [`Interp::sim_block_bound`]'s real time
    /// is abandoned, its child's process group killed, and refused.
    fn collect_on_baton(
        &self,
        running: std::process::Child,
        input: Option<Vec<u8>>,
        merged: Option<std::io::PipeReader>,
    ) -> Result<Waited, Failure> {
        let Some(bound) = self.sim_block_bound() else {
            return Ok(collect(running, input, merged, None));
        };
        let pid = running.id();
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            crate::signal::block();
            let waited = crate::signal::unblocked(|| collect(running, input, merged, None));
            let _ = sender.send(waited);
        });
        match receiver.recv_timeout(bound) {
            Ok(waited) => Ok(waited),
            Err(_) => {
                let group = i32::try_from(pid)
                    .ok()
                    .and_then(rustix::process::Pid::from_raw);
                if let Some(group) = group {
                    let _ =
                        rustix::process::kill_process_group(group, rustix::process::Signal::KILL);
                }
                Err(crate::Loud::sim_blocked_command().into())
            }
        }
    }

    /// What a command whose child ended answers: its redirected streams
    /// handed to `io`'s targets, the others written to this interpreter's
    /// sinks, and its return code.
    ///
    /// `held` is the condition a target raised while the child ran, which
    /// the clause raises in place of settling.
    fn command_ended(
        &mut self,
        io: Option<&IoContext>,
        ended: &Waited,
        held: Option<Failure>,
    ) -> Result<CommandOutcome, Failure> {
        let finished = match io {
            Some(context) => context.finish(self, &ended.out, &ended.err, held),
            None => Ok(()),
        };
        if io.is_none_or(|context| !context.redirects_output()) {
            self.write_out(&ended.out);
        }
        if io.is_none_or(|context| !context.redirects_error()) {
            self.write_err(&ended.err);
        }
        finished.map(|()| CommandOutcome::of(ended.rc))
    }

    /// `CommandHandler::call` (`concurrency/CommandHandler.cpp:98-151`) for a
    /// handler an extension registered: a direct one refuses any redirection
    /// with 98.921, and a redirecting one reads and writes through `io`,
    /// whose lines reach their targets once it returns, a `SYNTAX` it raised
    /// notwithstanding.
    ///
    /// `RC` is the object the handler returned, or `.false` for none, and
    /// the status is `Normal` unless it raised `ERROR` or `FAILURE`.
    fn run_registered_command(
        &mut self,
        handler: &CommandHandler,
        environment: &[u8],
        command: &[u8],
        io: Option<&IoContext>,
    ) -> Result<CommandOutcome, Failure> {
        if io.is_some() && !handler.redirects() {
            return Err(Raised::redirection_not_supported(environment).into());
        }
        if let Some(context) = io {
            context.begin(self)?;
        }
        let redirector = io.map_or_else(Redirector::unrequested, IoContext::redirector);
        let handled = self.run_command_handler(handler, environment, command, &redirector)?;
        if let Some(context) = io {
            let (out, err) = redirector.finish();
            context.finish_lines(self, &out, &err)?;
        }
        let condition = match handled.raised {
            None => None,
            Some(Failure::Raised(held)) if !held.condition.eq_ignore_ascii_case("SYNTAX") => {
                Some(Box::new(HandlerCondition {
                    name: held.condition.to_ascii_uppercase().into_bytes(),
                    description: held.description,
                    additional: handled.additional,
                    result: handled.result,
                }))
            }
            Some(raised) => {
                if handled.additional.is_some() {
                    self.activity.pending_additional = handled.additional;
                }
                return Err(raised);
            }
        };
        // `RexxActivation::command`: the condition's `RESULT` stands in for
        // its missing `RC`, and then for the handler's answer.
        let object = match condition.as_ref().and_then(|held| held.result) {
            Some(result) => result,
            None => match handled.value {
                Some(value) => value,
                None => self.counted(0),
            },
        };
        self.roots.activity_mut().push_temp(object);
        let text = self.string_value_text(object);
        let status = match condition.as_ref().map(|held| held.name.as_slice()) {
            Some(b"FAILURE") => ReturnStatus::Failure,
            Some(b"ERROR") => ReturnStatus::Error,
            _ => ReturnStatus::Normal,
        };
        Ok(CommandOutcome {
            rc: whole_value(&text).unwrap_or(0),
            status,
            supplied: Some(Supplied { object, text }),
            condition,
        })
    }

    /// The security manager's `COMMAND` checkpoint, and then the command
    /// itself where the manager did not take it.
    ///
    /// `Activity::callCommandExit` (`concurrency/Activity.cpp:2782`-`2792`)
    /// runs **before the handler is resolved**, so an environment nothing
    /// answers still reaches the manager.
    fn checked_command(
        &mut self,
        environment: &[u8],
        command: &[u8],
        io: Option<&IoContext>,
    ) -> Result<Commanded, Failure> {
        if self.effective_security_manager().is_none() {
            return self.run_command(environment, command, io);
        }
        let address = self.text(environment);
        self.roots.activity_mut().push_temp(address);
        let issued = self.text(command);
        self.roots.activity_mut().push_temp(issued);
        let entries = [(key::COMMAND, issued), (key::ADDRESS, address)];
        let Some(info) = self.security_check(message::COMMAND, &entries)? else {
            return self.run_command(environment, command, io);
        };
        self.command_from_manager(info).map(Commanded::Done)
    }

    /// What `SecurityManager::checkCommand` (`execution/SecurityManager.cpp:
    /// 244`-`281`) reads back out of the info directory once the manager has
    /// handled a command: the return code, and which condition to raise.
    fn command_from_manager(&mut self, info: ObjRef) -> Result<CommandOutcome, Failure> {
        let status = if self.security_entry(info, key::FAILURE)?.is_some() {
            ReturnStatus::Failure
        } else if self.security_entry(info, key::ERROR)?.is_some() {
            ReturnStatus::Error
        } else {
            ReturnStatus::Normal
        };
        // No `RC` entry is `IntegerZero`. Either object is what `RC` and a
        // raised condition's `RC` carry.
        let object = match self.security_entry(info, key::RC)? {
            Some(object) => object,
            None => self.counted(0),
        };
        self.roots.activity_mut().push_temp(object);
        let text = self.required_string_value(object)?;
        let text = self.to_text(text).into_owned();
        let rc = whole_value(&text).unwrap_or(0);
        Ok(CommandOutcome {
            rc,
            status,
            supplied: Some(Supplied { object, text }),
            condition: None,
        })
    }

    /// One command clause: evaluates the string, echoes what the setting
    /// asks, runs it, and settles `RC`, `.RS` and any condition.
    ///
    /// `environment` names the handler for this clause alone. `ADDRESS env
    /// command` leaves the activation's own pair untouched -- measured,
    /// `ADDRESS()` is unchanged afterwards and a later bare `ADDRESS`
    /// toggles to the alternate the one-off never wrote.
    ///
    /// `io` is the `WITH` configuration the issuing `ADDRESS` instruction
    /// carried, which merges with whatever the environment name has stored.
    ///
    /// Where the child's wait leaves the baton, the clause parks its activity
    /// and [`Interp::end_blocked_command`] settles it once the child is done.
    #[expect(
        clippy::too_many_arguments,
        reason = "the clause's own index and node beside the command's parts, each one the caller holds"
    )]
    pub(crate) fn exec_command(
        &mut self,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
        expression: &Expr,
        environment: Option<&[u8]>,
        io: Option<&AddressIo>,
    ) -> Result<ExecOutcome, Failure> {
        let indent = self.activity.clause_state.current_value_indent;
        if matches!(instruction.kind, rexx_parse::InstructionKind::Address(_)) {
            self.echo_address_command(source, instruction, indent);
        }
        let value = self.eval(code, expression)?;
        self.roots.activity_mut().push_temp(value);
        let value = self.required_string_or_nil(value)?;
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
        // **The redirections are evaluated here**, after the command string
        // and its `>>>` -- measured under `trace i`, a one-off carrying both
        // shows `>L>` and `>>>` for the command and only then each target's
        // own echo and `>K>` line. The frame holds every object they resolve
        // to for as long as the command runs and the lines are written back.
        let frame = self.roots.activity_mut().push_frame();
        let context = pinned!(
            self,
            crate::pinning::PinKind::RedirectWrapper,
            self.io_context(code, &name, io)
        );
        let (outcome, context) = match context {
            Ok(context) => (
                self.checked_command(&name, &command, context.as_ref()),
                context,
            ),
            Err(failure) => (Err(failure), None),
        };
        self.roots.activity_mut().pop_frame(frame);
        let outcome = match outcome {
            Ok(Commanded::Left(token)) => {
                self.activity.blocked = Some(Box::new(Blocked {
                    command,
                    indent,
                    token,
                    io: context,
                    captured: Vec::new(),
                    raised: None,
                    ended: None,
                }));
                return Ok(ExecOutcome::Park(crate::scheduler::ParkReason::Block));
            }
            Ok(Commanded::Done(outcome)) => Ok(outcome),
            Err(failure) => Err(failure),
        };
        let kept = self.activity.input_dispatch_syntax.take();
        let outcome = outcome?;
        self.serve_signal_now();
        // The SYNTAX condition the input reader's dispatcher took, raised now
        // that the command has completed: measured, oracle rc 0, a `LINEIN`
        // failing on its second line leaves `cat`'s one line of output in
        // the target and `RC` 41 in the trap.
        if let Some(kept) = kept {
            self.reraise_failure_levels();
            return Err(kept);
        }
        self.settle_command(index, instruction, source, &command, indent, &outcome)
            .map(ExecOutcome::Done)
    }

    /// The `*-*` line of an `ADDRESS` clause issuing a command where the
    /// setting echoes commands but not every clause (`TRACE C`), which the
    /// oracle echoes as it does a command clause. Every other setting that
    /// echoes it has done so before the clause ran.
    fn echo_address_command(
        &mut self,
        source: Option<&ProgramSource>,
        instruction: &Instruction,
        indent: usize,
    ) {
        let trace = self.chunk_trace();
        if trace.echoes(false, true)
            && !trace.echoes(false, false)
            && let Some((line, text)) = self.clause_site(source, instruction)
        {
            self.trace_issued_clause(line, indent, &text);
        }
    }

    /// Whether the running activity's command clause has a child off the
    /// baton, which [`Interp::end_blocked_command`] takes up.
    pub(crate) fn blocked_command_woken(&self) -> bool {
        self.activity.blocked.is_some()
    }

    /// The command clause whose child waits off the baton, run again: the
    /// output that arrived is written to the target, and the clause parks
    /// again until its child has ended, when it is settled as
    /// [`Interp::exec_command`] settles one whose child it waited for. A run
    /// with neither, after a wake that was not the command's, parks again.
    pub(crate) fn end_blocked_command(
        &mut self,
        index: usize,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
    ) -> Result<ExecOutcome, Failure> {
        let Some(mut blocked) = self.activity.blocked.take() else {
            return Err(
                crate::Loud::scheduler_inconsistency("a command resumed with no child").into(),
            );
        };
        let frame = self.roots.activity_mut().push_frame();
        let mut held = Vec::new();
        blocked.object_roots(&mut held);
        for object in held {
            self.roots.activity_mut().push_temp(object);
        }
        let captured = std::mem::take(&mut blocked.captured);
        if let Some(context) = &blocked.io {
            // A condition waits for the child, as the oracle's handler
            // catches it and raises it once it returns; anything else (an
            // EXIT, a refusal) ends the clause now.
            let mut raised = blocked.raised.take();
            let written = self.write_holding_last(&mut raised, |interp| {
                context.write_output(interp, &captured)
            });
            if let Err(failure) = written {
                self.roots.activity_mut().pop_frame(frame);
                return Err(failure);
            }
            blocked.raised = raised;
        }
        if blocked.ended.is_none() {
            self.roots.activity_mut().pop_frame(frame);
            self.activity.blocked = Some(blocked);
            return Ok(ExecOutcome::Park(crate::scheduler::ParkReason::Block));
        }
        let Blocked {
            command,
            indent,
            io,
            raised,
            ended,
            ..
        } = *blocked;
        let outcome = match ended {
            Some(ended) => self.command_ended(io.as_ref(), &ended, raised),
            None => Err(crate::Loud::scheduler_inconsistency(
                "a command resumed before its child ended",
            )
            .into()),
        };
        self.roots.activity_mut().pop_frame(frame);
        let kept = self.activity.input_dispatch_syntax.take();
        let outcome = outcome?;
        if let Some(kept) = kept {
            self.reraise_failure_levels();
            return Err(kept);
        }
        self.settle_command(index, instruction, source, &command, indent, &outcome)
            .map(ExecOutcome::Done)
    }

    /// `RC`, `.RS`, the trace lines, any condition and the debug pause of a
    /// command clause, instruction `index`, that ran `command` to `outcome`.
    fn settle_command(
        &mut self,
        index: usize,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
        command: &[u8],
        indent: usize,
        outcome: &CommandOutcome,
    ) -> Result<Flow, Failure> {
        if let Some(supplied) = &outcome.supplied {
            // The frame that held it is gone and `RC` is assigned below,
            // after allocations of this clause's own.
            self.roots.activity_mut().push_temp(supplied.object);
        }

        // **`::OPTIONS ERROR|FAILURE SYNTAX` escalates where the condition is
        // raised, not where it is delivered**: `Activity::raiseCondition`
        // reports the syntax error from inside the handler's own
        // `RaiseCondition`, so `RexxActivation::command` never regains control
        // and neither `RC`, `.RS` nor the failure retrace happens. Measured on
        // three descriptors -- `::OPTIONS FAILURE SYNTAX` with an untrapped
        // 127 shows the clause echo the error report writes and nothing else,
        // where the same program without the directive shows `>>>` and
        // `+++ "RC(127)"` first; `trace e` with `::OPTIONS ERROR SYNTAX` is
        // the same for an `exit 3`, and its control without the directive
        // keeps the retrace. `RC` in a `SIGNAL ON SYNTAX` handler is 98, the
        // raise's own, rather than the command's code or a stale one.
        //
        // The **original** condition is what is tested here. A `FAILURE` with
        // only `ERROR SYNTAX` set escalates at neither point: it traces, finds
        // no trap, is renamed, and is caught by the check in
        // `raise_command_condition` -- the C++'s own second test.
        //
        // A handler's `LOSTDIGITS`, `NOSTRING` and `NOTREADY` escalate here
        // too, under their own `::OPTIONS`: `Activity::raiseCondition` has an
        // arm for each and none for `NOVALUE`.
        match &outcome.condition {
            Some(held) => {
                if held.name != b"NOVALUE" && self.condition_raises_syntax(&held.name) {
                    return Err(self.handler_escalation(held, &held.name));
                }
            }
            None => {
                if let Some(condition) = outcome.status.condition()
                    && self.condition_raises_syntax(condition.as_bytes())
                {
                    let rc = outcome.rc.to_string().into_bytes();
                    return Err(escalated(condition.as_bytes(), command, &rc).into());
                }
            }
        }

        // A command typed at a debug pause sets neither `RC` nor `.RS`,
        // traces nothing and raises nothing (`RexxActivation.cpp:4442`).
        if self.debug_pause() {
            return Ok(Flow::Next);
        }

        // `RC` before anything else, which is where the C++ puts it too
        // (`RexxActivation::command`).
        //
        // **A manager's own `RC` is assigned as the object it set**, not as
        // a rendering of it: measured, `info~rc = 'abc'` leaves `RC` reading
        // `abc` with no `+++` line, because `numberValue` fails and the line
        // is gated on it.
        let assigned = match &outcome.supplied {
            Some(supplied) => supplied.object,
            None => self.text(outcome.rc.to_string().as_bytes()),
        };
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
            self.trace_command_retrace(line, indent, &text, command);
        }
        if (echoed || retrace) && outcome.rc != 0 {
            match &outcome.supplied {
                Some(supplied) => {
                    let text = supplied.text.clone();
                    self.trace_command_rc(indent, &text);
                }
                None => {
                    let text = outcome.rc.to_string().into_bytes();
                    self.trace_command_rc(indent, &text);
                }
            }
        }
        self.activation_mut().rs = Some(outcome.status.code());

        match &outcome.condition {
            Some(held) => {
                let name = held.name.clone();
                self.raise_handler_condition(held, &name)?;
            }
            None => {
                if let Some(condition) = outcome.status.condition() {
                    let supplied = outcome.supplied.as_ref().map(|supplied| supplied.object);
                    self.raise_command_condition(condition, command, outcome.rc, supplied)?;
                }
            }
        }
        if self.debug_pause_after_command(outcome.status)? {
            return Ok(Flow::Goto(index));
        }
        Ok(Flow::Next)
    }

    /// The `SYNTAX` error the `::OPTIONS` for `condition` makes of a
    /// handler's condition: its description and, for `ERROR` and `FAILURE`
    /// alone, its `RESULT` as the `RC` substitution.
    fn handler_escalation(&mut self, held: &HandlerCondition, condition: &[u8]) -> Failure {
        let rc = match held.result {
            Some(result) if matches!(condition, b"ERROR" | b"FAILURE") => {
                self.string_value_text(result)
            }
            _ => Vec::new(),
        };
        escalated(condition, held.description.as_deref().unwrap_or(b""), &rc).into()
    }

    /// Offers a condition a registered handler raised to the traps in force
    /// as `RexxActivation::command` does: under its own name and with the
    /// handler's description, `ADDITIONAL` and `RESULT`, its `RESULT` also
    /// its `RC`, which an `ERROR` or `FAILURE` carries even with no `RESULT`.
    /// An untrapped `FAILURE` is re-raised as `ERROR`; any other untrapped
    /// condition is silent.
    fn raise_handler_condition(
        &mut self,
        held: &HandlerCondition,
        name: &[u8],
    ) -> Result<(), Failure> {
        // **Any condition but `FAILURE` meets `ERROR SYNTAX` ahead of the
        // traps** (`RexxActivation::command`'s `!failureCondition` test):
        // measured, a handler's `USER` under `::OPTIONS ERROR SYNTAX` is
        // 98.970 with a `CALL ON USER` armed.
        let command_status = matches!(name, b"ERROR" | b"FAILURE");
        let syntax: &[u8] = if name == b"FAILURE" {
            b"FAILURE"
        } else {
            b"ERROR"
        };
        if self.condition_raises_syntax(syntax) {
            return Err(self.handler_escalation(held, syntax));
        }
        let raised = Raised {
            description: held.description.clone(),
            ..Raised::condition(std::borrow::Cow::Owned(
                String::from_utf8_lossy(name).into_owned(),
            ))
        };
        let rc = match held.result {
            Some(result) => Some(result),
            None if command_status => Some(ObjRef::NIL),
            None => None,
        };
        match self.trap_for(name) {
            Some(trap) if trap.call => {
                self.activity.pending_rc = rc;
                self.activity.pending_additional = held.additional;
                self.activity.pending_result = held.result;
                let object = self.build_condition_object(&raised, Some(true))?;
                self.activity.pending_traps.push_back(crate::PendingTrap {
                    condition: name.into(),
                    rc: None,
                    description: held.description.clone(),
                    object: Some(object),
                    activation: self.activation().id,
                    queued_during_delivery: false,
                    request: false,
                    fragment_depth: self.activity.fragment_depth,
                    trace_failure: None,
                });
                Ok(())
            }
            Some(_) => {
                self.activity.pending_rc = rc;
                self.activity.pending_additional = held.additional;
                self.activity.pending_result = held.result;
                Err(raised.into())
            }
            None if name == b"FAILURE" => self.raise_handler_condition(held, b"ERROR"),
            None => Ok(()),
        }
    }

    /// Offers `ERROR` or `FAILURE` to the traps in force.
    ///
    /// **An untrapped `FAILURE` is re-raised as `ERROR`** and an untrapped
    /// `ERROR` is silent -- measured, a command exiting 127 reaches a
    /// `SIGNAL ON ERROR` handler with no failure trap anywhere, and one
    /// exiting 3 reaches a lone `SIGNAL ON FAILURE` handler not at all.
    ///
    /// `supplied` is the `RC` a security manager set: the condition carries
    /// that object as its `RC` and no `RESULT`, as `checkCommand`'s
    /// `createConditionObject` builds it.
    fn raise_command_condition(
        &mut self,
        condition: &'static str,
        command: &[u8],
        rc: i32,
        supplied: Option<ObjRef>,
    ) -> Result<(), Failure> {
        // **Ahead of the traps**, which is where `RexxActivation::command`
        // checks it. That cannot overtake a handler: arming one clears the
        // escalation first (`ConditionSyntax::disable_for` turns ERROR and
        // FAILURE off for `CALL ON` as well as `SIGNAL ON`), and measured,
        // `::OPTIONS ERROR SYNTAX` with `signal on error` armed runs the
        // handler and exits 0. The reraise below re-enters here, which is the
        // C++'s second check after a FAILURE becomes an ERROR.
        if self.condition_raises_syntax(condition.as_bytes()) {
            return Err(
                escalated(condition.as_bytes(), command, &rc.to_string().into_bytes()).into(),
            );
        }
        let rendered = rc.to_string().into_bytes();
        // One `Raised` for both branches. `result` beside `rc` is what parts
        // a command's condition from every other: measured, its directory
        // carries `RESULT` under `SIGNAL ON` and under `CALL ON` alike, where
        // `raise error 5` carries `RC` alone.
        let raised = Raised {
            // A supplied `RC` is already the variable's value, and
            // `pending_rc` carries it into the condition object.
            rc: supplied.is_none().then_some(rendered),
            result_is_rc: true,
            description: Some(command.to_vec()),
            ..Raised::condition(std::borrow::Cow::Borrowed(condition))
        };
        match self.trap_for(condition.as_bytes()) {
            Some(trap) if trap.call => {
                self.activity.pending_rc = supplied;
                // Built now, not at delivery: by then this clause has
                // finished and `POSITION` and `STACKFRAMES` no longer exist
                // to be read.
                let object = self.build_condition_object(&raised, Some(true))?;
                self.activity.pending_traps.push_back(crate::PendingTrap {
                    condition: condition.as_bytes().into(),
                    rc: raised.rc.clone(),
                    description: Some(command.to_vec()),
                    object: Some(object),
                    activation: self.activation().id,
                    queued_during_delivery: false,
                    request: false,
                    fragment_depth: self.activity.fragment_depth,
                    trace_failure: None,
                });
                Ok(())
            }
            Some(_) => {
                self.activity.pending_rc = supplied;
                Err(raised.into())
            }
            None if condition == "FAILURE" => {
                self.raise_command_condition("ERROR", command, rc, supplied)
            }
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

    /// `corpus/oracle-crashes.txt` entry 44: the oracle dies on `cd ~` with
    /// `HOME` unset. Here the shell answers it, as the C++ hands over a bare
    /// `cd` or `cd ~/x` with `HOME` unset, and the directory stays. The rc is
    /// the shell's own and is not asserted.
    #[test]
    fn cd_home_with_home_unset_goes_to_the_shell() {
        let source = b"here = directory()\n\
            'unset HOME'\n\
            'cd ~'\n\
            say 'stayed' (directory() == here)\n";
        let outcome = crate::run_program("/t.rex", source.to_vec(), crate::Invocation::none());
        assert_eq!(outcome.exit_code, 0);
        assert_eq!(String::from_utf8_lossy(&outcome.stdout), "stayed 1\n");
    }

    /// `unquote`'s byte rules, the C++ loop's answers.
    #[test]
    fn unquote_drops_quotes_and_escapes_one_byte() {
        assert_eq!(unquote(b"\"a\"b"), b"ab");
        assert_eq!(unquote(b"a\\ b"), b"a b");
        assert_eq!(unquote(b"a\\\"b"), b"a\"b");
        assert_eq!(unquote(b"a\\\\b"), b"a\\b");
        assert_eq!(unquote(b"ab\\"), b"ab");
    }
}
