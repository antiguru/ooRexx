# Task 6 review: AddCommandEnvironment, the exit context and the I/O redirector

Reviewer: review-s6. Range `362fd648c..b06b278ce`, HEAD `b06b278ce`, clean tree. Release `rexx-run` built
into my own target dir (`Compiling rexx-api`, `Compiling rexx-exec` lines present). Scratch `$S` =
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/review-s6/`.
`$S/cmp.sh` runs the oracle under the standard wrapper and the crate, each from a fresh `mktemp -d`, and compares
stdout, stderr and rc separately. The forge is rebuilt with its own `build.sh` into `$S/forge` (NEEDED
libgcc_s.so.1, libc.so.6, ld-linux; no undefined Rexx symbol). `liborxfunction.so` NEEDs libc.so.6 only.
Probes are in `$S/probes/`, outputs in `$S/out/`, and the predictions, written before each run, are in
`$S/predictions.txt`. The parent commit `362fd648c` was built from `git archive` (touched, own target dir,
`Compiling rexx-exec` present) as `$S/target-base/release/rexx-run`.

## Verdict

**Spec compliance: met. Quality: changes requested: two Important findings (I1, I2) and Minor ones (M1-M7).**
Every step of the brief is built. Both corpus witnesses and every forge probe the report calls identical
reproduce as identical. Miri under Stacked Borrows is green. The boundary is clean: `unsafe` is only in
`ffi.rs` and `load.rs`, each block has a SAFETY note, and the notes I checked are true. The C-unwind set
is exactly the Throw slots and the handler entry call, and the ABI matches the header. The failure-path
probes found two unrecorded silent divergences in how a handler's condition meets `::OPTIONS ... SYNTAX`.
They also found that one recorded divergence's reason is false: the ledger says only the handler can
observe it, and a plain Rexx program can.

## A. The boundary

- **unsafe placement.** Per-file count of added lines containing `unsafe` over the range: ffi.rs 49,
  load.rs 7, and 0 in every other `.rs` file, `rexx-api/src/redirect.rs` included. Only `ffi.rs` and
  `load.rs` carry `#![allow(unsafe_code)]` in `rexx-api/src`. Every added `unsafe {` block has a
  `SAFETY:` comment in the three lines above it except one, `load.rs` `CommandHandler::call`
  (`catch_unwind(... || unsafe {`), whose note is the eight-line comment directly above the statement.
- **SAFETY truth, spot-checked.**
  - `CommandHandler::call` says a handler "is still mapped: the only holder is the interpreter's handler
    table, which it clears before its first library close, and a library closes nowhere else". `Mapping::close`
    (load.rs:143) is the only place the handle is dropped while the Rc lives.
    `run_package_unloaders` (dispatch/library.rs) clears the table when any `close()` answers true. A library
    whose loader raises stays held (`settle_library`, install.rs:1932, "a loader that raises leaves the
    library held"). A `RegisterLibrary` entry that loses its name race is dropped, but it is mapped over
    `Library::this()`, so the drop unmaps nothing. The claim holds.
  - `redirector_of`: `owner` is `from_ref(redirector).cast_mut()` and is only read back as `&Redirector`, and
    every mutation goes through `Cell`/`RefCell`/`OnceCell`. `ReadInput`'s and `ReadInputBuffer`'s addresses
    point into `Box<[u8]>`s that never move for the `Redirector`'s life, as the doc says.
- **ABI and layout.** `ExitContextInterface` and `IORedirectorInterface` in layout.rs match
  `api/oorexxapi.h:749-784` member for member. `add_command_environment(instance, CSTRING, REXXPFN, c_int)`
  matches `AddCommandEnvironment`. The layout test now includes both tables in the abort/return-shape check,
  the interface-version check, the refusing-members derivation and the `Owned` offset check.
  `DIRECT_COMMAND_ENVIRONMENT`/`REDIRECTING_COMMAND_ENVIRONMENT` are 1 and 2, as in the header.
- **Unwind discipline.** The exit context's Throw members are `throw_exception*::<RexxExitContext_>`, the
  Task 5 C-unwind generics, and they are marked `unwinds` in the layout table. Every other new member
  (the variable members, `GetCallerContext`, every redirector member and `add_command_environment`)
  is `extern "C"`. The handler types `DirectHandler`/`RedirectingHandler` are `extern "C-unwind"`, which is
  the extension entry-point call. `CommandHandler::call` catches only `crate::ffi::Thrown` and
  `resume_unwind`s anything else. **The exit context's Throw members no longer abort.** `throws.rex`
  (each member with a C++ local whose destructor logs) and `redirect_raise.rex` (`WTHROW`) are identical to
  the oracle, the destructors included. This is right: the header gives the exit context its own Throw
  members, and orxfunction-shaped handlers call them. The report's control C9 shows that the catch is live.
- **No process-global state.** The new statics `EXIT_CONTEXT` and `IO_REDIRECTOR` are immutable tables. The diff adds no
  `print!`/`eprint!`/`println!`/`eprintln!`/`dbg!`/`stdout()`/`stderr()`/`libc::write` (grep over the added lines, exit 1).
  The handler table is `Interp::command_handlers`. No new code writes to fd 0/1/2: an unredirected
  stream's writes have no sink and are dropped, which is what `redirect.rex`'s no-`WITH` lines show
  identically on both sides.
- **Miri.** ffi.rs and load.rs changed only in `949b15f84`. I reran the report's command
  (`RUSTUP_HOME=<surface-4>/rustup-home cargo +nightly miri test -p rexx-api --lib --offline`, no
  `MIRIFLAGS`, so Stacked Borrows) at HEAD into my own target dir. Result: exit 0, 54 passed, 0 failed,
  8 ignored (`$S/miri.txt`, 9 Compiling lines). The new ffi tests (`a_command_environment_registers_a_handler_of_either_type`,
  `a_direct_handler_binds_the_callers_variable_and_answers`, `an_exit_context_throw_leaves_the_handler`,
  `a_redirecting_handler_reads_and_writes_each_stream`) and every `redirect::tests` test ran and passed. The
  ignored ones are the pre-existing process-spawning and running-image tests.
- **Refusal-sites rows.** `escalated` (command.rs:865) and `redirection_not_supported` (error.rs:1581) are
  `body`/`off-send-surface`. Every construction site is in command.rs: lines 616, 802-803, 870, 891 and
  952 (grep for `escalated(` and `redirection_not_supported(`). `run_command` is reached only from
  `checked_command`, whose one caller is the command clause at command.rs:770, plus the library test.
  `run_command_handler` is in dispatch/, but it constructs neither. So neither row is on the send
  surface. See M5 for a derivation gap this range opened.

## B. Behaviour against the oracle

### Reruns

`$S/cmp.sh` over `task-6-forge/probes/*.rex` and `rust/corpus/lang/library_command_*.rex`: exitvars, raises,
raises2, redirect, redirect_raise, returns and throws are identical, as are library_command_environments and
library_command_redirect, all at rc 0. address_trace differs on stderr only (the missing `4 *-* address
system "exit 3"` and `5 *-*` lines). stream_state differs on all three descriptors (oracle `handler 2 2` /
`shell 0 2` rc 0; crate 97.1 STATE rc 159). Both match the ledger text.

**"Found, not made" is true for both.** On the parent build, address_trace gives the same stderr as HEAD, and
`$S/probes/stream_shell.rex` (stream_state's shell line alone, no extension) gives the same 97.1 STATE on
both builds.

### Failure paths the report does not probe

Identical on all three descriptors:
- `handler_rc.rex`: orxfunction's `rc-direct` answers -1 under `CALL ON ERROR`, `CALL ON FAILURE` and each
  of `TRACE E`, `TRACE F`, `TRACE N`. The result is `-1 0` with no trap and no trace line.
- `direct_normal.rex`: a direct handler under `WITH INPUT NORMAL`, `OUTPUT NORMAL` and `ERROR NORMAL` each
  gives 98.921. A redirecting one under all-`NORMAL` answers `10000` (requested, nothing redirected).
- `names.rex`: a 250-character name registers and answers. At 251 characters, `ADDRESS VALUE` gives 29.1 on
  both, and the registration itself is accepted on both. The empty name registers and answers. Re-registering
  replaces the handler, direct to redirecting and back. An undefined type leaves the old handler in place.
- `opts_failure_error.rex`: a handler's untrapped FAILURE under `::OPTIONS ERROR SYNTAX` alone gives
  98.970 with `desc`/`res`.
- `streams.rex` without its Stream-object line, and `streamobj.rex`: stream, stem and array
  targets, replace and append. The input stream is also the output stream (in place). One `.Stream` object
  serves as both targets.

### Findings

**I1. Important: a handler's condition other than ERROR and FAILURE escapes `::OPTIONS ... SYNTAX`.**
`rust/crates/rexx-exec/src/command.rs:798-806` (the clause's escalation) and `:884-928`
(`raise_handler_condition`) escalate only when the condition is ERROR or FAILURE. The oracle has two
further arms:
- `RexxActivation::command` (`execution/RexxActivation.cpp`, the block after `setReturnStatus`) raises
  98.970 for **any** handler condition that is not FAILURE when ERROR SYNTAX is enabled. The test is
  `!failureCondition && isErrorSyntaxEnabled()`, and it comes before the trap check.
- `Activity::raiseCondition` (`concurrency/Activity.cpp:596-620`) escalates NOTREADY, and also NOSTRING
  and LOSTDIGITS, inside the callback.

Runs (prediction written first, both confirmed):
- `$S/probes/opts_user.rex` (`::options error syntax`, handler `RAISE USER FOO`): oracle
  `syntax 98.970 External command "desc" ended with return code res. 98`, crate `after user res 0`.
  Both rc 0.
- `$S/probes/opts_notready.rex` (`::options notready syntax`, handler `RAISE NOTREADY`): oracle
  `syntax 98.974 Stream "desc" is not ready. 98`, crate `after notready res 0`. Both rc 0.

Both are silent wrong answers and are not in the ledger. By the C++ ordering, a USER trapped by
`CALL ON USER` under ERROR SYNTAX also escalates on the oracle. That is read, not run.

**I2. Important: the ledger's write-timing entry says only the handler can observe it, and a plain Rexx
program can.** `docs/superpowers/plans/phase-4-exclusions.txt:4762-4768` says "Only the handler can tell"
and "Read, not measured". `$S/probes/interleave.rex` uses no exit-context read: the output and error
targets are `OutputStream` subclasses whose `LINEOUT` says its line, and the forge's `ECHO` handler writes
`o:` then `e:` per input line. The prediction was a difference, and it was confirmed. The oracle prints
`out o:a`, `err e:a`, `out o:b`, `err e:b`. The crate prints `out o:a`, `out o:b`, `err e:a`, `err e:b`.
Both are rc 0 with an empty stderr. Any stream-object target with a side effect sees it, and that is the
FUNCTION group's own `ArrayOutputStream` shape. The divergence may stay recorded with owner none, but its
reason must say it is observable from Rexx and must cite a measurement.

**M1. Minor: a SYNTAX after a write keeps the line on this crate, but not on the oracle when the target is
also the input source.** The same ledger entry says "a SYNTAX the handler raises after writing leaves the
written line in place on both (redirect_raise.rex)". `CommandHandler::call` (`concurrency/CommandHandler.cpp`,
the REDIRECTING arm) runs `ioContext->cleanup()` only when `activity->run` returns, so a SYNTAX skips the flush
of the conflict buffer that `resolveConflicts` inserted. `$S/probes/conflict_syntax.rex` (`WSYNTAX` with input
stem `s.` and output stem `s.`; predicted to differ): oracle `stem 93.900 2 s1 s2`, crate
`stem 93.900 1 before S.2`. The array form (`using (arr)` both ways) is identical, `1 before The NIL object`.
The code is command.rs:621-622, which writes the lines before looking at the SYNTAX. Narrow the ledger
sentence and record the shape.

**M2. Minor: an inserted struct took over `Popped`'s doc comment.** `rust/crates/rexx-exec/src/dispatch/library.rs:567`:
`/// What [Interp::pop_native_frame] answers.` now sits directly above `HandledCommand`'s own doc
(lines 568-569), so rustdoc joins the two on `HandledCommand`. `struct Popped` (line 577) is left
undocumented. This is visible in the diff hunk at `review-362fd648c..b06b278ce.diff:4101-4111`.

**M3. Minor: a comment states a set's size.** `rust/crates/rexx-api/src/ffi.rs:3595`: "One of the four write
members." This breaks the global constraint on comments.

**M4. Minor: the redirect module doc cites the oracle for behaviour the oracle does not have.**
`rust/crates/rexx-api/src/redirect.rs:13-15` says the lines "reach their targets after it
(`interpreter/instructions/CommandIOContext.cpp`)". That file writes as it goes (I2). The citation should
land on this crate's own choice, or on the ledger entry.

**M5. Minor: two constructors named `escalated`, one row.** `rust/crates/rexx-exec/src/command.rs:406` (free fn
returning `Raised`) and `:865` (`Interp::escalated`, returning `Raised`). `tests/refusal_sites.rs`
`definitions` keys by name (`found.insert(name, ..)`), so the later definition wins. The table has one
`escalated` row at `:865`, and the free function has none, although the header promises "Every constructor".
Both are body-only, so the verdict column is unaffected. Renaming one restores a row per constructor.

**M6. Minor: a null name is not recorded.** `add_command_environment` (ffi.rs) registers nothing for a null
name. The oracle's `addCommandHandler` calls `new_upper_string(name)`
(`runtime/InterpreterInstance.cpp:913-917`), which does not guard null. That is read, not run. The ledger's
null-handler entry covers the handler argument only.

**M7. Minor, not this task's: a Stream object's writes are visible by name before it is closed.**
`$S/probes/streamflush.rex` has no handler and no extension. It writes through `.stream~new('sh.txt')` (by
a shell command) and through `.stream~new('s2.txt')~lineout`, then reads each by name. The oracle gives
`0 0` / `0` and the crate `0 1` / `1`. The parent build gives the crate's answer too, so the difference is
pre-existing. I found no ledger entry mentioning it (`/bin/grep -a -i` for flush/buffer phrasing). It is
what made `streams.rex`'s Stream-object line differ. Recorded here for the controller.

## Notes, no finding

- The gate status file `surface-6/gates/status.txt` reads `b06b278ce`, G1-G6 exit 0, finished.
- Sourceline expectations: the `count` headers (70, 130) equal `wc -l` of the two corpus files.
- The report's concern about `GetCallerContext`'s refusal name on a host with no surface stands as written.
  Only the fake host can reach it.

## Cleanup

`$S/target`, `$S/target-base`, `$S/target-miri` and `$S/base` deleted after this review was written.
