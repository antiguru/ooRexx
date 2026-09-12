# Phase 7 — streams and platform: design spec

**Status: draft, 2026-09-11.** Written from six surveys under
`.superpowers/sdd/2026-09-11-phase-7/survey/` (copied to
`docs/superpowers/records/2026-09-11-phase-7/` at close) and the 2026-09-04 scoping survey
`docs/superpowers/specs/2026-09-04-phase-7-scoping.md`.

## 0. Architecture constraints

Read from the tree at `5bcb28edb`. Every area design below obeys them.

- **A1. Output is buffered in the interpreter.** `SAY` appends to `Interp.out`
  (`rust/crates/rexx-exec/src/run.rs:2079`), trace lines and error reports to `Interp.trace`;
  `run_program` returns both in `Outcome`, and `rexx-run` writes stdout, then stderr, at exit
  (`rust/crates/rexx-exec/src/bin/rexx-run.rs`). So `.STDOUT` and `.STDERR` write into those two
  buffers, a child process's stdout and stderr are captured and appended in order, and no Phase 7
  code writes to file descriptor 1 or 2 itself.
- **A2. The interpreter runs in-process, on threads, in parallel.** `tests/watchdog/mod.rs:48`
  spawns a thread calling `run_program`, and `tests/corpus.rs:70` is one of its callers. So Phase 7
  mutates no process-global state. The interpreter holds a **shadow environment** and a **shadow
  current directory**, initialised from the process unless the `Invocation` supplies them. Every
  relative path resolves against the shadow directory, and every child process receives both
  (`Command::current_dir`, `env_clear` + `envs`). `Interp::resolve_search`
  (`rust/crates/rexx-exec/src/lib.rs:2965`) reads `REXX_PATH`, `PATH` and the process's directory
  today, and moves to the shadows.
- **A3. Input has one source.** `ProgramInput` (`rust/crates/rexx-exec/src/input.rs`) is what
  `PULL`, `PARSE PULL` and `PARSE LINEIN` read; `.STDIN`, `.INPUT` and `LINEIN()` share its source
  and its position.
- **A4. Native state lives on the object.** `Body::Instance` carries `native:
  Option<Box<BufferState>>` for `MutableBuffer` (`rust/crates/rexx-core/src/body.rs:114`); a stream's
  state rides in the same slot, widened to an enum.
- **A5. A native entry point is a row.** `dispatch/native.rs`'s `deferred(name, family)` becomes
  `implemented(name, family, arity, body)`.
- **A6. A corpus program that touches the file system runs in a directory of its own**, the same
  absolute path for both interpreters and emptied between the oracle's run and the crate's, so every
  path a program prints agrees without any normalisation.

## 1. Scope

The roadmap row (`docs/superpowers/plans/2026-07-27-rust-rewrite.md:476`): "`StreamClasses.orx`
runs; stream model, `ADDRESS`, file system green on all 5 platforms; the `Sys*` subset ooTest needs
(D11) works".

**The phase's purpose is to let the ooTest suite start against this crate** (L2). The suite's
kicker, `ootest/testOORexx.rex`, reads and writes `PATH` through `VALUE(..., 'ENVIRONMENT')`,
changes directory with `DIRECTORY()`, and calls `'worker.rex'(arguments)` by file name;
`worker.rex` redirects `.OUTPUT` to a `.Stream`, and calls `SysFileExists` and `.File`. Measured
2026-09-11 at `5bcb28edb`: every one of those refuses here.

### In

| area | what |
|---|---|
| S | the stream model: `.Stream`, `StreamSupplier`, the stream builtins `CHARIN` `CHAROUT` `CHARS` `LINEIN` `LINEOUT` `LINES` `STREAM` `QUALIFY`, the `NOTREADY` condition |
| F | `.File` and its `file_*` natives |
| M | the standard streams `.STDIN` `.STDOUT` `.STDERR`, the monitors `.INPUT` `.OUTPUT` `.ERROR` `.TRACEOUTPUT` `.DEBUGINPUT`, `.SYSCARGS`, and the routing of `SAY`, trace output, error reports, `PULL` and the default-stream builtins through them |
| C | commands: the command instruction, `ADDRESS` issuing a command, `ADDRESS ... WITH`, `RC`, `ERROR`/`FAILURE`, the `+++` RC trace line |
| E | the process environment and the platform functions: `VALUE(..., 'ENVIRONMENT')`, `SETLOCAL`/`ENDLOCAL`, `DIRECTORY`, `USERID`, and the rest of the unix external-function table |
| R | external routine resolution by file, and package loading from files |
| U | the RexxUtil subset, from a derived and committed list |
| D | interactive `TRACE ?` |
| K | the security manager's hooks for commands and streams (D12's Phase 7 half) |

### Out, each with its destination

| what | destination | why |
|---|---|---|
| `RexxQueue`, the `rexx_*_queue` natives, `.STDQUE` | Phase 10 | the external-queue API is the RXAPI daemon's (D-P7-3) |
| native shared libraries: `::REQUIRES ... LIBRARY` and `EXTERNAL` naming a library other than `REXX` | Phase 8 | loading a library means calling it through the native API Phase 8 builds (D-P7-6) |
| the timer natives behind `.Alarm` and `.Ticker` | Phase 6 | already so (`dispatch/native.rs` `Family::Timer`) |
| the RexxUtil remainder | Phase 10 | D11 — and it must refuse **loudly**, which it does not today (§U) |
| `RXFUNCADD` `RXFUNCDROP` `RXFUNCQUERY` `RXQUEUE` | Phase 10 | `phase-4-exclusions.txt` |
| platforms other than Linux | assessed as CANNOT ASSESS | no other platform is available to this crate's gates |

## 2. Rulings on the 2026-09-04 open decisions

- **D-P7-1 is reversed: the monitors are built in Phase 7.** The earlier recommendation left
  `.OUTPUT` and `Monitor` loud because no program could observe the difference. The ooTest framework
  is such a program: `worker.rex:129` redirects `.output~destination(...)`, and `ooTest.frm` sends
  `.traceoutput~destination` and `.error~say`. `SAY` keeps its direct write on the fast path while
  `.OUTPUT` is unredirected; §M defines the exact check.
- **D-P7-3 stands**: `RexxQueue` moves to Phase 10.
- **D-P7-5 stands**: the excluded-builtin owner message is fixed in the first task, with an
  assertion.
- **D-P7-6 (new): native libraries move to Phase 8.**
- **D-P7-7 (new): no `unsafe`.** `std::env::set_var` is `unsafe` in edition 2024, so the process
  environment is shadowed in the interpreter (§E).

## 3. Gate

1. `rust/corpus/method-bodies.txt`: every `File`, `Stream` and `StreamSupplier` row moves to
   `answers`, and `rust/corpus/docs/class-set.txt`'s three rows owned by `7` gain construction
   programs (D-P7-4).
2. A corpus program per stream operation and per area above, byte-identical on all three
   descriptors, filed in `rust/corpus/phase-7.txt`.
3. `docs/superpowers/plans/phase-4-exclusions.txt`: every Phase 7 row is deleted as delivered, or
   re-homed by a ruling in this spec.
4. No refusal in the crate names Phase 7, asserted by a test.
5. `gate_tables::CLOSED_PHASES` names `7`.
6. The four standard gates are green at the closing commit.
7. Reported, not gated: how far `ootest/testOORexx.rex` gets on this crate.
8. Linux measured; the other four platforms recorded as CANNOT ASSESS.

## 4. Area designs

Each area's evidence -- the measured oracle behaviour, the C++ citations and the alternatives
weighed -- is in its survey report under `.superpowers/sdd/2026-09-11-phase-7/survey/`, committed to
`docs/superpowers/records/2026-09-11-phase-7/` at the close. This section carries the decisions.

### 4.F `.File` (survey `B-file.md`)

`.File` is ordinary Rexx (`interpreter/RexxClasses/StreamClasses.orx:506`) over 24 deferred
`file_*` natives, none of which needs anything but strings, integers and logicals across the
boundary. `MutableBuffer` no longer blocks it; **`file_qualify` alone does**, because `init`
qualifies eagerly, so every instance method is unreachable until that one lands.

- **Qualification is a port of `SysFileSystem::normalizePathName`** (`SysFileSystem.cpp:687`):
  lexical, single pass, `..` collapsed textually and floored at the root, trailing separator
  stripped except at the root, and it must work for paths that do not exist. Not
  `fs::canonicalize` (resolves symlinks, requires existence) and not `Path::components` (does not
  collapse `..` against a preceding segment). Relative names resolve against the shadow directory
  (A2), which `DIRECTORY()`, the streams, `Sys*` and command spawning share.
- **`canRead`/`canWrite` are `access(2)`**, not mode bits, and additionally require existence.
- **Timestamps** are `.DateTime` basetime values: getters from `std::fs::Metadata` re-encoded the
  way `utcToLocal` (`SysFileSystem.cpp:942`) does, `.nil` on a missing file; setters through
  `filetime`, preserving the other stamp. `length` on a missing file is `0`, not `.nil`.
- **`isHidden`** is `exists && qualified path contains "/."` -- an ancestor's dot hides a
  descendant.
- **`renameTo` refuses a same-path and an existing target before calling `rename(2)`**, and
  `deleteFile` requires `access(W_OK)` on the file itself before `unlink(2)`. A straight delegation
  to `std::fs` diverges on both, and the rename case overwrites data the oracle refuses to touch.
- **`list` keeps `read_dir` order** -- `readdir(3)` order on both sides. Sorting it for determinism
  is a divergence, not a fix.
- **`setReadOnly`/`setWritable` answer nothing**, so an expression context is 91.999; the witness
  carries that shape as well as the side effect.
- `.File`'s row in `rust/corpus/docs/class-set.txt` gains the construction `.File~new('.')`.

### 4.C Commands, `ADDRESS` and the platform layer (survey `D-commands-env.md`)

- **`Interp::run_command(environment, command, io)`** over `std::process::Command`, always with
  `env_clear().envs(shadow)` and `.current_dir(shadow)`. stdout and stderr are piped and drained
  concurrently, then appended to `Interp.out` / `Interp.trace` at the point the oracle's own writes
  fall -- after the child under `TRACE N`/`E`/`F`, after the `>>>` line under `C`/`A`/`R`/`I`.
  Inheriting the descriptors instead prints `b a c` for `say 'a'; 'echo b'; say 'c'`.
  Exit status: a signal is `-signal`, otherwise the code; 127 is `FAILURE`, any other non-zero is
  `ERROR`, and a spawn failure is `FAILURE` 127.
- **The handler table** is keyed by the upcased name: `""`, `COMMAND`, `SYSTEM`, `SH` run
  `/bin/sh -c`; `KSH`, `CSH`, `BSH`, `BASH`, `TCSH`, `ZSH` run `/bin/<name lowercased> -c`; `PATH`
  splits the string itself (blanks and tabs, a double-quoted run is one argument, 400 arguments
  maximum); anything else is rc 30 and `FAILURE`. The name is stored as written and only the lookup
  upcases.
- **`cd`, `set`, `unset` and `export NAME=value` are handled in the interpreter**, as the oracle
  does (`platform/unix/SystemCommands.cpp`), against the shadows, and only when the command carries
  no unquoted `< > | & ;`.
- **The shadow environment** is an ordered `Vec<(Vec<u8>, Vec<u8>)>` on `Interp`, initialised from
  the process unless `Invocation` supplies one. Every reader is a witness: `VALUE`, child
  processes, `Interp::resolve_search`'s `PATH`/`REXX_PATH`, `SysTempFileName`'s `TMPDIR`, `HOME`.
  `VALUE(name, new, 'ENVIRONMENT')` truncates a value at the first NUL, silently declines a name
  that is empty or holds `=`, and removes on `.nil`.
- **The shadow directory** is a `PathBuf` on `Interp`. `DIRECTORY(new)` qualifies, requires a
  directory, stores the canonicalised path and answers it; a failure answers `""` and changes
  nothing. One helper resolves every relative path in the phase against it.
- **`USERID()`** reads the effective uid from `/proc/self/status` and the name from `/etc/passwd`,
  falling back to the shadow `USER`/`LOGNAME`. No new dependency, and no corpus program can carry
  the value.
- **`.RS`** is `Option<i8>` in the activation settings beside the address pair: inherited by
  internal calls, fresh for methods and external programs, read after `.local` and `.environment`
  and before the `.RS` string fallback. `RC` is the ordinary variable, assigned before any trace.
- **`SETLOCAL`/`ENDLOCAL`** push and pop `(directory, environment)` snapshots on the top-level
  activation; a restore re-puts the saved names and never removes a name added since. Both oracle
  crashes are licensed divergences (§4a).
- **`ERROR`/`FAILURE`** build the measured condition directory, queue it through the existing trap
  machinery, rename an untrapped `FAILURE` to `ERROR` and re-raise; `::OPTIONS ERROR|FAILURE SYNTAX`
  is checked where the raise happens *and* after the conversion, and an armed trap disables it. The
  `+++ "RC(n)"` line is the `>>>` emitter at the same indent, only when the clause was traced and
  the rc is numeric and non-zero.
- **`ADDRESS ... WITH`** keeps a permanent per-name configuration in the activation settings and
  builds a per-command context by evaluating sources and targets at issue time. Stem, stream,
  collection and string redirectors follow the oracle's REPLACE/APPEND rules, and a `RexxQueue`
  target refuses loudly (Phase 10).
- **The security manager** (D12's Phase 7 half) is stored per executable, reached through one
  `check(message, entries)` that must reach `UNKNOWN`, rejects a missing result (91.999) and a
  non-logical one (34.903), and hooks `COMMAND` before the handler, `STREAM` in the stream-name
  resolver, `LOCAL`/`ENVIRONMENT` in `env_seam::admit`, `METHOD` in `check_protected_method`, and
  `CALL`/`REQUIRES` at their resolution sites.
- **`DIRECTORY`, `FILESPEC` and `BEEP` are not builtins**: they are native routines of the oracle's
  internal `REXX` package, so the crate answers 43.1 -- a wrong answer rather than a refusal. An
  internal-routine table consulted at `rust/crates/rexx-exec/src/run.rs:3549`, between the builtin
  step and the external file search, makes them refuse loudly until implemented; it is the same slot
  the `Sys*` library needs.

### 4.S The stream model (survey `A-streams.md`)

The Rexx halves of `Stream`, the three mixins and `StreamSupplier` already run -- `rexx-lib`
embeds `StreamClasses.orx` and every `EXTERNAL 'LIBRARY REXX ...'` method binds to a deferred entry.
Phase 7 fills in those entries and the eight builtins.

- **A stream's state rides in the object.** `Body::Instance`'s `native: Option<Box<BufferState>>`
  widens to `Option<Box<NativeState>>` with a `Buffer` and a `Stream` arm; `StreamState` keeps the
  oracle's own field names (`StreamNative.hpp:158`) -- name, qualified path, kind, state, the mode
  flags, the four positions plus the two line-character positions, `stream_line_size`, `reclength`,
  a read-ahead buffer. It holds no `ObjRef`, so the collector is unaffected. `stream_uninit` closes
  and sets `native` to `None`; every entry point then raises 48.1, including the eight that
  segfault the oracle.
- **The eight builtins become message sends**, as they are in the C++: `resolve_stream` maps an
  omitted name or `STDIN`/`STDOUT`/`STDERR` to the standard streams, qualifies anything else, looks
  it up in a stream table keyed by the **qualified** name, and otherwise sends `NEW` to `Stream`.
  The table belongs to a program or method activation and is borrowed by reference by internal
  calls, `PROCEDURE`, `::ROUTINE` and `INTERPRET` frames; the builtins that the C++ passes `&added`
  insert on a miss, `STATE`, `DESCRIPTION` and a non-OPEN/CLOSE/SEEK command do not. On a program
  or method activation's exit every entry is sent `CLOSE`.
- **Files are write-through** in the first cut -- every measured observable (`QUERY SIZE` counting
  unflushed bytes, `QUERY POSITION SYS`) comes out right for free, and only `close` has to flush.
  Reads take a 4 KiB read-ahead with one unget byte, which is what the CR-before-LF peek and
  `hasData` need. Switching between reading and writing re-seeks the descriptor to the logical
  position.
- **`.STDOUT` and `.STDERR` write into `Interp.out` and `Interp.trace`** and are never locally
  buffered, so `SAY`, `lineout(,...)`, `lineout('STDOUT',...)` and `.stdout~charout` interleave in
  program order by construction. `.STDIN` reads through `ProgramInput`, which gains a byte read and
  a `has_data`; `has_data` answers 1 without peeking, since `FIONREAD` needs `unsafe`.
- **NOTREADY** is raised with the stream name as description, the stream object as `additional` and
  the call's own answer as `result`; `PendingTrap` gains those two fields and they are rooted. An
  untrapped NOTREADY changes nothing but the state.
- **The `LINES('C')` count cache is ported with its off-by-one.** The oracle's second `LINES('C')`
  after a `CHARIN` answers one short; a port that recounts every time diverges, and ooTest's stream
  tests count lines in loops. Parity wins over cleanliness here.
- **The option tokenizer** is a port of `StreamCommandParser`, including the per-option minimum
  abbreviation lengths and the first-match-in-table-order rule; every conflicting or repeated option
  is a bare 93.
- **`HANDLE:n` refuses loudly**: reaching an already-open descriptor needs `from_raw_fd`, and the
  oracle segfaults on the only interesting use of it anyway.
- `chrono` (already a workspace dependency) renders `QUERY DATETIME`'s local `ctime` string, which
  the Rexx half re-parses.

### 4.R External routines and package loading (survey `E-external-resolution.md`)

- **The file search is the last step before 43.1**, after the label, the builtin, the `::ROUTINE`s
  and the library routines -- exactly where `run.rs`'s `None =>` arm sits today.
- **An external file call reuses the `::ROUTINE` call path** rather than growing a second one; the
  three differences are flags on the activation: the file route inherits the caller's `ADDRESS`, the
  call type is `FUNCTION` or `SUBROUTINE`, and `merge_required` runs after a normal return. The file
  is **re-parsed on every call** -- the oracle caches nothing here -- and must not enter the
  requires cache.
- **A callee that does not parse raises the callee's own error**, with the two-frame traceback, not
  a loud refusal; `::REQUIRES` gets the same conversion in the same change.
- **`require.rs` has three measured infidelities**: it treats `.hid` as carrying an extension where
  the oracle never examines byte 0; it keeps scanning after a path entry that exists but is not a
  regular file, where the oracle abandons that spelling and extension; and it never expands `~`.
  The candidate list becomes groups, one per spelling-and-extension, so the abandon rule has
  something to abandon.
- **The search reads the shadows.** A `SearchContext { parent_dir, parent_ext, cwd, rexx_path,
  sys_path }` is built by the caller, so the function is pure and unit-testable and the process
  environment is never consulted. This is exactly what the ooTest kicker needs.
- **`Package~new(name)` searches the global context** -- no parent directory, no parent extension --
  where the crate uses the caller's; `newFile` and `loadPackage(name, source)` gain the parent
  package chain that routine, class and program lookup fall back to, which lifts both refusals.
- **Witnesses that depend on the environment get a sidecar.** A corpus program may carry
  `<name>.env` (`PATH=`, `REXX_PATH=`, `CWD=`) and a `<name>.d/` fixture directory; the harness
  feeds both sides from it through `Oracle::run_in` and the crate's shadow environment. A pinned
  `PATH` also keeps the scratchpad off the oracle's search path.

### 4.M The standard streams, the monitors, and interactive debug (survey `C-monitors.md`)

`.local` holds exactly the ten names `ORACLE_LOCAL` already lists, built the way
`LocalServer~initInstance` builds them (`interpreter/RexxClasses/CoreClasses.orx:988`): three
Streams, five Monitors each stacking the next, `.STDQUE`, and `.SYSCARGS` from the launcher.

- **`.STDIN`, `.STDOUT`, `.STDERR` are ordinary `Stream` instances** whose backing is the
  interpreter rather than a file: writes append to `Interp.out` / `Interp.err`, reads take the next
  line of `ProgramInput`. `.Stream~new('STDOUT')` still makes a different object, and a closed
  `.STDOUT` swallows every later `SAY` at rc 0, which the oracle does too.
- **`.SYSCARGS`** is an `Array` of the separate command-line words, so `Invocation` carries them
  beside the joined argument string; absent, the entry is absent.
- **`.STDQUE`** stays unbuilt and its refusal names **Phase 10**, not Phase 7.
- **Every route is a message send, and the survey measured which**: `SAY` sends `SAY` with one
  string; `LINEOUT()`/`CHAROUT()` with the name omitted send `LINEOUT`/`CHAROUT` with the arguments
  given and their reply is the builtin's result; `LINEOUT('STDERR', ...)` goes to `.ERROR`;
  every trace line *and every line of the error report* is one `LINEOUT` of a `TraceObject` to
  `.TRACEOUTPUT`; `PULL`, `PARSE PULL`, `PARSE LINEIN`, `LINEIN()`, `CHARIN()` send `LINEIN` /
  `CHARIN` with no arguments to `.INPUT`, `LINES()` sends `LINES('NORMAL')`, and the queue is
  consulted before `.INPUT`.
- **The fast path (A1's performance constraint) re-derives the route rather than caching a flag.**
  Only two things can move it, because `~define` on a REXX class is 98.985 and `~setmethod` on a
  monitor is 98.991: the `.local` entry, and the bootstrap Monitor's destination stack. So `SAY`
  reads `.local`'s `OUTPUT` slot behind a generation counter on that one Directory, pointer-compares
  it with the bootstrap Monitor, peeks its destination queue, pointer-compares that with the
  bootstrap `.STDOUT`, and writes directly only when all of it holds. Any other state -- a replaced
  entry, `.nil`, a removed entry, a pushed destination -- takes the send. Trace output takes the
  same shape against `.TRACEOUTPUT`.
- **A missing entry or `.nil` under `OUTPUT` writes the line to stdout directly**; any other object
  gets the message, and its failure propagates as the oracle's through-the-Monitor 97 report, which
  names `REXX line 1457` and carries a `(no source available)` traceback line.
- **End of input raises `NOTREADY` on every read, `PULL` included** (against `iostrms.xml:681`),
  with description `STDIN`, silent when untrapped. The counts (`LINES()`, `CHARS()`) raise nothing.
- **`rexx-run` drains before a blocking read.** The interpreter keeps buffering; the binary installs
  a pair of sinks that flush `Interp.out` and `Interp.err` to the real descriptors when
  `Input::read_line` is about to block on the process's stdin, so a prompt precedes its answer. The
  harness never installs them, and no differential can see the difference -- so it needs its own
  `rexx-run`-level test.
- **Interactive `TRACE ?`** gains per-activation state -- debug, pause, skip and suppressed, bypass,
  prompt-issued, source-traced -- inherited by internal calls and restored on return. The banner is
  the source string once before the first traced clause; the prompt line is emitted once per
  activation; a null line continues, `=` re-executes the clause, anything else runs as an INTERPRET
  fragment with no tracing, no `RC`, no `.RS`, whose SYNTAX error prints the two
  `+++ Interactive trace.` lines and is swallowed; a fragment that changes the flow or sets a trace
  setting ends the pause. `TRACE` instructions are ignored while in debug and the TRACE builtin is
  not; numeric `TRACE` outside a pause stays 24.901; `RXTRACE=ON` arms `?R` on each program's
  top-level activation.
- **What ooTest needs from this area**: `worker.rex` pushes a file stream on `.OUTPUT` and pops it;
  `OOREXXUNIT.CLS` pushes the defaults back onto all three output monitors around every test, so the
  stacks grow and a design that caps them or folds a re-push of the default breaks; and its
  `.NullOutput` case pushes an object with only `lineout` and `say`.

### 4.U The RexxUtil subset (survey `F-rexxutil.md`)

**The subset is derived, and the derivation is committed with it.** D11 ordered the whole suite's
`Sys*` calls by raw count; the survey re-derived it the way the phase needs -- what the framework
calls, plus what test groups *outside* RexxUtil's own 30 groups call, ranked by the number of
distinct `.testGroup` files rather than by call sites, with the commands quoted so the derivation
re-runs. The culling is where the work was: one whole cluster of apparent singletons belongs to
`rxunixsys`, a native library (Phase 8); several more are Windows-gated at their own call site; and
two are placeholder names a security-manager mock calls for the trigger rather than the answer.

**Phase 7 builds nine**: `SysFileExists`, `SysFileTree`, `SysSleep`, `SysVersion`, `SysLinVer`,
`SysFileDelete`, `SysIsFile`, `SysRmDir`, `SysMkDir`.

- **They are not builtins and must not join the builtin table.** On the oracle `REXXUTIL` and `REXX`
  are two internal packages consulted at the same point, after `::ROUTINE`s and before the external
  file search. One internal-routine table at `rust/crates/rexx-exec/src/run.rs:3549` serves both,
  with the package as a property of the row; `DIRECTORY`, `FILESPEC` and `BEEP` are its other
  members. Their arity errors are the native-routine `88.9xx` shapes, not the builtins' `40.x`.
- **The Phase 10 remainder refuses loudly from a derived list.** Every name RexxUtil registers on
  Linux, minus the nine, is a row that refuses naming Phase 10, sourced from
  `interpreter/runtime/RexxUtilCommon.cpp`'s table plus the platform `#include` so it cannot drift.
  Today all of them -- and `DIRECTORY` -- answer 43.1, which a program cannot tell from its own
  typo.
- **`SysFileTree`'s globbing is hand-ported**, not delegated to the `glob` crate: the doc fixes the
  grammar it needs (`*`, `?`, `[abc]`, `[!abc]`, `[a-z]`), and `glob`'s bracket semantics against
  `fnmatch(3)` are an unanswered question, where a hand-port's are known before it ships. The
  witness carries a `[a-z]` and a `[!...]` filename either way.
- **`SysFileDelete` answers 13 for a missing file**, because the oracle pre-checks `access(W_OK)`
  and returns a constant `EACCES` for any pre-check failure; `std::fs::remove_file` alone answers 2.
  The other error numbers fall out of `std::fs` correctly.
- **`SysSleep`'s bound is 2147483**, not the documented `999999999`.
- **`SysMkDir`'s mode argument** goes through `DirBuilderExt::mode`, which a plain `create_dir`
  would silently ignore.
- **`SysVersion` and `SysLinVer` read `uname(2)`**, which is host-dependent and therefore fine: the
  gate runs both interpreters on the same host.
- Every path argument resolves through the same shadow-directory helper `.File` and `DIRECTORY` use.

### 4.D Dependencies

Two new direct dependencies, both already in the offline registry, both a safe API over a syscall
`std` does not expose. `chrono` is the only external dependency `src/` has today, and its
`Cargo.toml` entry carries the reasons at the site; these two follow that pattern.

| crate | why | used by |
|---|---|---|
| `rustix` | `access(2)` -- which mode bits cannot answer -- and `uname(2)` | `canRead`, `canWrite`, `deleteFile` and `SysFileDelete`'s pre-check, `SysVersion`, `SysLinVer` |
| `filetime` | setting atime and mtime, which `std` cannot | `lastModified=`, `lastAccessed=` |

`chrono` already covers the local-time re-encoding for file timestamps and `QUERY DATETIME`. `nix`
is rejected because `rustix` answers both of the calls it was wanted for; `whoami` and `glob` are
rejected because a `/proc` read and a small hand-port answer theirs with no dependency at all.

## 4a. Licensed divergences

Each one names what cannot be matched and why, in the shape
`rust/crates/rexx-exec/tests/licensed_divergences.rs` already uses.

- **`ENDLOCAL` past the outstanding `SETLOCAL`s.** The oracle dies with SIGSEGV rc 139: its guard
  tests `TheNilObject` while an emptied queue answers `OREF_NULL`. This crate answers `0`, which is
  what the oracle itself answers when no `SETLOCAL` ever ran.
- **A second `ENDLOCAL` that restores.** The oracle aborts, `free(): invalid pointer` rc 134, even
  fully paired: `restoreEnvironment` `putenv`s pointers into the saved Rexx buffer and the next
  restore frees them, so one restore per process is all that survives. This crate restores as many
  times as a program asks.

  `rust/corpus/oracle-crashes.txt` entry 10 carries both chains, the measured programs and the
  control. No differential case may run either program.
- **`~copy` of a `Stream`.** The oracle aborts at termination, `double free or corruption (out)`
  rc 134: `RexxObject::copy` carries `CSELF` into the copy and both objects' `uninit` destruct the
  one `StreamInfo`. No open and no existing file are needed. This crate's state block is cloned
  with the body, so a copy is an independent, unopened stream that answers its own name and
  `UNKNOWN`. `rust/corpus/oracle-crashes.txt` entry 13 carries the program, the four positive
  shapes and the two negative controls; no differential case may copy a stream.
- **A default I/O route with nothing at the end of it.** `rust/corpus/oracle-crashes.txt` entry 11:
  the `OUTPUT` or `INPUT` entry removed from `.local` and then a name-less `LINEOUT`/`LINEIN`, and a
  traced clause while `.TRACEOUTPUT`'s destination is `.nil`, are all SIGSEGV rc 139 on the oracle.
  This crate answers 97.1 for the first two -- the answer the oracle itself gives when the entry is
  `.nil` rather than absent -- and for the third writes the trace line to stdout, as the oracle does
  before dying, and exits normally.

## 5. Corpus and harness

- **Run directory (A6).** Every program `rust/corpus/phase-7.txt` lists runs with its working
  directory at `$CARGO_TARGET_TMPDIR/corpus-run/<its path, "/" spelled "__">/`, created empty
  before the oracle's run and emptied again before the crate's, so both sides see one absolute path.
  The program file stays where it is and is named by its absolute path, so a search that starts at
  the program's own directory still starts in `rust/corpus/lang/`.
- **The determinism rule is narrowed, not dropped.** `rust/corpus/README.md` forbids file-system
  state. A Phase 7 program may create, read and delete files inside its run directory, and read the
  fixtures beside it in `rust/corpus/` that it names. It prints no timestamp, no size of a file it
  did not write, no directory listing it did not create, and no host variable's value.
- **Standard input.** A program `lang/p.rex` with a sibling `lang/p.stdin` reads that file's bytes
  on standard input on both sides: the oracle through its stdin, the crate through
  `Invocation::with_input(ProgramInput::Bytes(..))`. Without one, both read nothing, as today.
- **Environment.** Both sides receive `REXX_CORPUS_VAR=corpus-value` on top of the inherited
  environment; a witness reads that variable, never a host one.
- **Helpers.** A program that calls or requires a second file names one beside it in
  `rust/corpus/lang/`, spelled `.cls` (outside every scan) or `.rex` (a corpus program of its own),
  which is the README's existing rule. The rule at `phase-4-exclusions.txt:342` -- no corpus program
  may depend on external routine resolution -- is lifted for the programs `phase-7.txt` lists.
- **Commands** in a witness are POSIX utilities every Linux host has: `echo`, `printf`, `true`,
  `false`, `sh -c 'exit N'`, and `cat` of a file in the run directory.
