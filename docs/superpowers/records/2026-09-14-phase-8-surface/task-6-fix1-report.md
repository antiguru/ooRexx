# Task 6 fix round 1 report

Base `b06b278ce`. One commit: `ea4a6d194` Escalate every handler condition the oracle does, and fix
Task 6's records. Scratch `$S` = `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/fix-s6/`.

Instruments: `$S/cmp.sh PROG...` runs the oracle under the standard wrapper and this crate's release
`rexx-run` (`$S/target`, or `$B`) each from a fresh `mktemp -d`, with the oracle's `build/lib` and
`$S/forge` (the forge rebuilt with its own `build.sh`: NEEDED libgcc_s.so.1, libc.so.6, ld-linux; no
undefined Rexx symbol) on `LD_LIBRARY_PATH`, and compares stdout, stderr and rc separately. Outputs
are in `$S/out/`. Predictions, each written before its run, are in `$S/predictions.txt`. The
baseline binary (`b06b278ce`, `Compiling rexx-exec` present) is kept as `$S/rexx-run-base`. After the
last source edit, the fixed binary was checked to be newer than `command.rs`.

## I1: a handler's other conditions under `::OPTIONS ... SYNTAX`

**Done: matched to the oracle.** `rust/crates/rexx-exec/src/command.rs`:
- The command clause's pre-RC escalation now also covers a held handler condition named `LOSTDIGITS`,
  `NOSTRING` or `NOTREADY` under that condition's own `::OPTIONS`. That is `Activity::raiseCondition`'s
  arms inside the callback, which have no `NOVALUE` arm. The test is `name != NOVALUE &&
  condition_raises_syntax(name)`.
- `raise_handler_condition` now escalates any condition but `FAILURE` under `ERROR SYNTAX` ahead of the
  traps. This is `RexxActivation::command`'s `!failureCondition && isErrorSyntaxEnabled()`. `FAILURE`
  under `FAILURE SYNTAX` behaves as before.
- The free `escalated` maps each condition to its SYNTAX error: 98.971, 98.972, 98.973, 98.974, else
  98.970. `Interp::handler_escalation` supplies the `RESULT` text only for `ERROR` and `FAILURE`.

Evidence. These are new forge probes, committed under `task-6-forge/probes/`:
- `opts_error.rex` (`::options error syntax`): USER, USER with no RESULT, NOTREADY, HALT, LOSTDIGITS,
  NOVALUE and NOSTRING. Also USER with `CALL ON USER` armed, USER under `trace c`, and USER with `CALL ON
  ERROR` armed.
- `opts_callback.rex` (`::options notready syntax lostdigits syntax nostring syntax novalue syntax`,
  with `.RS` left at 1 by `exit 3`): NOTREADY, LOSTDIGITS, NOSTRING, USER, ERROR and NOVALUE, plus
  NOTREADY with `CALL ON NOTREADY` armed.
- `opts_both.rex` (`error syntax notready syntax`): NOTREADY.
- `opts_failure.rex` (`failure syntax` alone): USER and NOTREADY.

Oracle, measured:
- Under ERROR SYNTAX, every case is 98.970 `External command "desc" ended with return code res.`. With
  no RESULT the message ends `return code .`. An armed `CALL ON USER` does not stop the escalation. An
  armed `CALL ON ERROR` does: the USER reaches the handler.
- The callback arms give 98.974, 98.972 and 98.973 with `.RS` still 1 and RC 98.
- USER and NOVALUE are not promoted by those options, and neither is ERROR without ERROR SYNTAX.
  NOTREADY with a NOTREADY trap armed reaches the trap.
- NOTREADY under both options is 98.974.
- FAILURE SYNTAX alone promotes nothing.

Runs:
- Baseline: opts_error differs on stdout and stderr, opts_callback and opts_both differ on stdout,
  opts_failure is identical. All as predicted.
- Fixed: all four are identical on all three descriptors, rc 0.
- A stderr difference in the first opts_error was the recorded ADDRESS-instruction TRACE C echo
  (`38 *-* address xd ...` missing). That case now issues a command clause after `address xd`.

The review's `opts_user.rex` and `opts_notready.rex` are identical after the fix. With the fixed
binary, `$S/cmp.sh` over every forge probe and the three `library_command_*` programs is identical,
except the recorded address_trace and stream_state, and interleave and conflict_syntax (I2, M1).

Controls. Each mutation was made on a copy, restored, and checked with `cmp`:

| # | Mutation | Prediction | Result |
|---|----------|------------|--------|
| C1 | clause-level arm tests `ERROR`/`FAILURE` only | opts_callback, opts_both differ; opts_error, opts_failure identical | as predicted |
| C2 | the `ERROR SYNTAX` test in `raise_handler_condition` gated on `command_status` again | opts_error differs; the other three and raises/raises2 identical | as predicted |

**Corpus witness: only the non-promotion shape is possible through orxfunction.**
- orxfunction's handlers (`dHandler`, `rHandler`, `ioHandler`, `testbinaries/orxfunction.cpp:530-640`)
  call no Raise or Throw member. The corpus loads only the oracle's `build/lib`, and orxfunction is the
  only extension there that calls `AddCommandEnvironment` (grep of the oracle tree's `.c`/`.cpp`).
- So no corpus program can show a promoted handler condition. The promotions are witnessed by the forge
  probes above, not by the STRICT corpus.
- `rust/corpus/lang/library_command_options.rex` (+ `.env`, the phase-8.txt entry, and the sourceline
  expectation recorded with the module comment's procedure: `count 28`, and the lines equal the file)
  runs each orxfunction handler under `::options error syntax failure syntax notready syntax lostdigits
  syntax nostring syntax`. Nothing escalates. A command's own `exit 3` still gives 98.970.
- It is identical to the oracle, rc 0. It passes on the baseline too and stays green under C1 and C2:
  it can only catch an escalation invented where no condition was raised.

## I2: the write-timing ledger entry

**Done: corrected, not matched.** Matching the order would take a redirector context that reaches the
interpreter on every write, which is not cheap.
- The entry in `docs/superpowers/plans/phase-4-exclusions.txt`, "A REDIRECTING HANDLER'S LINES REACH
  THEIR TARGETS WHEN IT RETURNS...", no longer says "Only the handler can tell" or "Read, not
  measured".
- It now states what `interleave.rex` shows from plain Rexx, with both outputs. The oracle prints
  `out o:a`, `err e:a`, `out o:b`, `err e:b`. Ours prints both out lines, then both err lines. Both are
  rc 0 with stderr empty.
- The probe is now committed as `task-6-forge/probes/interleave.rex`, and the entry names `compare.sh`
  and the date. Re-run: `$S/cmp.sh .../task-6-forge/probes/interleave.rex` gives "interleave: out
  differs", with the outputs above (`$S/out/interleave.*`).

## M1: SYNTAX after a write when the target is also the input source

**Done: ledger sentence rewritten, not matched.** Matching is not local:
- `needsBuffering` is `type() == in->type() && target() == in->target()` over the oracle's
  RedirectionType classes (`OutputRedirector.cpp:63`, `:527`), and that type is not recorded on the
  input side here.
- `resolveConflicts` has an error-conflict arm that wraps `output`, not `error`
  (`CommandIOContext.cpp:128-132`). A faithful port would need its own probes. An array as input and
  output is not a conflict on the oracle.

The same ledger entry now says the written line stays on both except where the output target is also
the input source. It gives `conflict_syntax.rex`'s two lines: stem `2 s1 s2` on the oracle against
`1 before S.2` here, and the identical array line. It also gives the reason, `cleanup()` running only
when the handler returns (`CommandHandler.cpp`). The probe is committed under
`task-6-forge/probes/`. Re-run: stem line differs, array line identical, as the review found.

## M2

The stray `/// What [Interp::pop_native_frame] answers.` is moved back onto `struct Popped`
(`dispatch/library.rs`). `HandledCommand` keeps its own two-line doc.

## M3

`ffi.rs`: "One of the four write members." is now "One of the write members."

## M4

`redirect.rs`'s module doc now cites the ledger (`docs/superpowers/plans/phase-4-exclusions.txt`) for
the write timing instead of `CommandIOContext.cpp`, which does the opposite.

## M5: two `escalated` constructors

**Done, differently from the literal ruling. Justification:**
- Renaming the method (tried first as `handler_escalated`) and refreshing gave a new row with an
  **empty column 3** (`$S/refusal-sites.rename-attempt.tsv`, line 146:
  `Raised handler_escalated <empty> command.rs:876 off-send-surface`). No other row in the table has one.
- The reason is that the derivation (`refusal_sites.rs`, `surface`) treats a constructor defined outside
  `error.rs`/`lib.rs` as a free function. It skips a name preceded by `.`, so it cannot see
  `self.handler_escalated(` call sites.
- A method constructor there is outside what the table can derive. So the constructor became one free
  function, `escalated` (command.rs:408), which now also maps LOSTDIGITS, NOSTRING and NOTREADY.
- The method that remains, `handler_escalation`, returns `Failure`, not `Raised`. So it is not a
  constructor by the table's definition ("return type is Loud or Raised"), and its construction site is
  the `escalated(` call inside it, which the scanner does see.
- The result is one row per constructor and no row with an undetermined surface.

Re-derivation: `REXX_REFUSAL_SITES_REFRESH=1 cargo test --release -p rexx-exec --test refusal_sites --
--test-threads=1`, run from the committed `b06b278ce` table. It gave 5 passed, and then the normal run
passes. `diff` against the committed table:
- `escalated`: column 4 only, `command.rs:865` -> `command.rs:408`. The row now describes the free
  function. Column 3 stays `body`. The verdict stays `off-send-surface`, carried by the refresh as for
  every body row.
- `nostring_syntax`: **column 3 changes, `send` -> `body+send`.** This is justified: `escalated` in
  command.rs now constructs it for a handler's NOSTRING, a body site. The verdict `agrees` and the
  other measured columns are about its send-surface site, which is unchanged. The refresh carries them
  because the row is still on the send surface.
- No new row.

## M6

A new ledger entry, "A NULL COMMAND ENVIRONMENT NAME REGISTERS NOTHING", is in the Task 6 block. It
records the divergence as accepted, with the reason "the oracle dereferences NULL (read from source, not
run)". It cites `interpreter/runtime/InterpreterInstance.cpp:915` (`new_upper_string(name)`) and
`interpreter/classes/StringClass.hpp:887` (`strlen(string)`, unguarded). It notes that no Rexx program
can send a null name. Owner none.

## M7

Not fixed. `.superpowers/sdd/queued/2026-09-27-stream-write-visibility.md` holds the probe and both
outputs, re-run by this round: oracle `0 0` / `0`, crate `0 1` / `1`, both rc 0 with stderr empty. It
states no mechanism, because none was read.

## Also changed

The existing entry "::OPTIONS ERROR SYNTAX AND FAILURE SYNTAX OVER A HANDLER'S RaiseCondition ESCALATE
WHEN THE HANDLER RETURNS" now names LOSTDIGITS, NOSTRING and NOTREADY too. Those also escalate here
after the handler returns, where the oracle escalates inside the callback. It cites
`Activity.cpp:596-620` and adds opts_callback.rex and opts_both.rex to its evidence (message, RC 98 and
`.RS` identical).

## Miri

`ffi.rs` and `redirect.rs` changed (comments only). The command was `RUSTUP_HOME=<surface-4>/rustup-home
CARGO_TARGET_DIR=$S/target-miri cargo +nightly miri test -p rexx-api --lib --offline`, Stacked Borrows
(no `MIRIFLAGS`), on the committed tree. Result: exit 0, 54 passed, 0 failed, 8 ignored, 9 Compiling
lines (`$S/miri.txt`).

## Local checks before the gates

`cargo fmt --all --check` exit 0. `cargo clippy -p rexx-exec -p rexx-api --all-targets -- -D warnings`
exit 0 (`$S/clippy.txt`).

## Gates

`$S/gates.sh` is `surface-6/gates.sh` with `S` pointed at `fix-s6`. Status is in `$S/gates/status.txt`,
started at `ea4a6d194`.

GATES_PENDING

## Gates (controller-appended, from scratchpad/fix-s6/gates/status.txt)
G1 fmt exit 0
G2 clippy(empty target) exit 0
G3 release build (test --no-run) exit 0
G4 release test exit 0
G5 debug build (test --no-run) exit 0
G6 debug test exit 0
finished 2026-09-27T21:43:14+02:00
