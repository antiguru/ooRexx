# Task 6.13 report: commands

Clusters `redirected-command-holds-baton`, `context-rs` and `hostemu-halt`. The scclient, sfclient
and RexxContext tests pass, and `SIGNAL TEST_SIGNAL_ANY_CONDITION_HALT` is a derived `Phase 10
Extension` row. `suite-9d-set.txt` now holds no test, and the whole suite is green (exit 0).

## Commits

* `0a934624f` Phase 9d Task 6.13: commands. The fixes, both witnesses with their
  `sourceline_oracle/` files and `phase-9.txt` lines, the exclusion row, the emptied 9d set, the
  re-derived `refusal-sites.tsv` and the updated pool test.
* The records commit that follows holds `docs/superpowers/records/2026-10-18-phase-9d/t6.13/` and
  this report.

## Root causes and fixes

### redirected-command-holds-baton

`Interp::run_command` let a child's wait leave the baton only when `io.is_none()`. A redirected
command waited on the baton, so the scserver/sfserver started by `Object~start` ran alone and the
clients found a closed port. The oracle releases the kernel lock around every command handler
(`NativeActivation::run`, `NativeActivation.cpp:1691`), redirected or not.

Fix (`command.rs`):
* `Block` carries the child's input bytes, the merged pipe and whether the command is redirected.
  A redirected child is collected whole on the pool thread (input written, both streams kept); an
  unredirected one streams as before.
* The parked clause keeps its `IoContext` in `Blocked`. `Activity::object_roots` roots its target
  objects through `Blocked::object_roots` / `IoContext::object_roots` (`redirect.rs`) while the
  activity is parked; `end_blocked_command` roots them in a temps frame while the targets are
  written.
* `command_ended` is the one place that writes the targets and then the unredirected halves to this
  interpreter's sinks, for both the baton path and the resumed clause.
* An input-reader SYNTAX (`input_dispatch_syntax`) stays on the activity across the park and is
  raised once the resumed command completes, as on the baton path.

The pool unit test `a_redirected_command_stops_every_other_activity` pinned the old order against
its own note that the oracle prints `w 1` to `w 3` first (30 of 30 runs). It is now
`a_redirected_command_does_not_stop_another_activity` and expects the oracle's order. It failed
against the fix before the update, so it also witnesses the change.

### context-rs

`context_rs` (`dispatch/context.rs`) answered `.nil` always. It now reads the context's activation
`rs`, `.nil` where none is set, as `RexxActivation::getContextReturnStatus` does.

### hostemu-halt

The site that answered rc 30 FAILURE is `run_command`'s no-handler branch. It now refuses with
`Loud::loader_environment` (`lib.rs`, `#[cold]`, `Phase 10 Extension`) when the environment's name
is, without case, the package name of a held native library whose package loader ran
(`Libraries::loader_named`, `libraries.rs`). hostemu's loader registers `HostEmu` through
`RexxRegisterSubcomExe` (`extensions/hostemu/platform/unix/hostemu.cpp:836`), the classic API this
crate does not implement. Text: `rexx-exec: ADDRESS environment "hostemu" a native package loader
registers is not implemented (Phase 10 Extension)`.

A first version refused any unregistered environment once any loader had run. The group run showed
it turning `SIGNAL TEST_8` (`XYZZY`) and `TEST_SIGNAL_ANY_CONDITION_FAILURE` (`A_FAILURE`) from
passing to blocked, because the group's `::requires "hostemu" LIBRARY` holds the library for every
test. The name rule keeps them at rc 30, as the oracle answers.

`phase-9-exclusions.txt` gains `base/keyword/SIGNAL	TEST_SIGNAL_ANY_CONDITION_HALT	Phase 10
Extension	refusal: rexx-exec: ADDRESS environment "hostemu" a native package loader registers is
not implemented (Phase 10 Extension)`, in C order. The suite checked it (the row's verdict is
`Phase10(Extension)` and the group is green).

## Witnesses

`rust/corpus/phase-9.txt`, each with its `sourceline_oracle/` file. The driver ran on a copy in a
scratch directory, so the witness's prolog could not write into the tree.
`t6.13/witnesses/run.sh` writes `summary.tsv` (parent is `t6.12-r2/rexx-head`, `45bd0c3ba`, whose
code `71369fa96` carries unchanged):

| program | oracle | parent | head |
|---|---|---|---|
| `context_rs_command` | rc 0 | stdout differs | identical |
| `command_redirected_off_baton` | rc 0 | stdout differs | identical |

Read stdout:
* `context_rs_command`: `.nil` before any command; 0, 1 and -1 after `true`, `exit 3` and
  `exit 127`, with `.rs` beside each; after a redirected `address ""` it is 0 and equal to `.rs`;
  an internal routine starts with the caller's 0, sets 1, and the caller keeps 0; a `::routine`
  starts at `.nil` while its caller's context object reads 0, and a method context the same,
  `.nil 1 0`. The parent prints `The NIL object` on every line.
* `command_redirected_off_baton`: a started activity runs a redirected shell child that touches
  `started` and waits for `answer`; main polls with `SysSleep`, writes `answer`, and the child
  prints `got hello from rexx` (the input line arrives too). The parent prints `0 gave up`: main
  never ran while the child waited.

Oracle stability: 8 of 8 runs of `command_redirected_off_baton` gave one output; 8 of 8 on head
too.

**Simulation mode cannot run this witness.** In the simulation mode the pool's bound is 0
(`sim.rs:956`), so every command waits on the baton by design and one waiting past `block=`
refuses: `REXX_SWITCH_MODE=sim:1` gives `a command in the simulation mode that waits longer than its
bound is not implemented`, rc 120. The witness runs in the real mode; its output does not depend
on the schedule, because main writes `answer` only after the child has written `started`.

The refusal has no corpus witness: the corpus gate compares against the oracle, which halts there.
`t6.13/probes/hostemu/` (run by `t6.13/probes/run.sh`, with the oracle's library directory as the
suite harness gives it) records it beside its adjacent success:
* `hostemu_halt`, `signal_any_hostemu`: oracle HALT; parent rc 30 FAILURE; head the refusal, rc 120.
* `other_environment`: `address "xyzzy"` after `::requires "hostemu" LIBRARY` is rc 30, `.rs` -1 on
  all three sides; the head then refuses at `hostemu`.
* `no_requires`: without the library, `hostemu` and `nosuch` are rc 30 on all three sides.

## The tests

`suite-9d-set.txt` loses `base/class/RexxContext TESTRS01`, `base/keyword/SIGNAL
TEST_SIGNAL_ANY_CONDITION_HALT`, `samples/scclient TEST_04_SCCLIENT.REX` and `samples/sfclient
TEST_04_SFCLIENT.REX`, its last lines.

## Changed rows

Against `t6.12/r2/suite/tests.tsv`, by the command at the top of `t6.13/suite/changed-rows.txt`:
* `RexxContext TESTRS01`, `scclient TEST_04_SCCLIENT.REX`, `sfclient TEST_04_SFCLIENT.REX`:
  `failed` to `passed`.
* `SIGNAL TEST_SIGNAL_ANY_CONDITION_HALT`: row `-` to `Phase10(Extension)`, `failed` to blocked by
  the refusal.
* `extensions/hostemu/hostemu`'s rows that reach `address hostemu` (`TEST_EXECIO_DISKR_*_STEM`,
  `TEST_EXECIO_DISKR_SKIP_THEN_STEM`, `TEST_EXECIO_DISKR_STEM_SKIP`, `TEST_EXECIO_DISKW_STEM`,
  `TEST_HI`, `TEST_TE`): already `Phase10(Extension)` `group` rows; the crate column goes from
  `failed` to blocked by the same refusal. Same mechanism, already owned by Phase 10.

No other row changed. The `SysFileTree TEST_TREE_DIRECTORY` flake did not show.

## Queued item

`.superpowers/sdd/queued/2026-10-18-command-waits-for-background-child.md` is **not closed**. Its
probe is an unredirected command, a path this fix does not change: head still waits for the
timeout, rc 124, while the oracle prints `v=[abc] READY` rc 0 (`t6.13/probes/bgcmd/`). That is
`start`'s piping of an unredirected command's stdout and stderr, which waits for EOF the
background child holds. Not widened.

## Checks

From `rust/`, `CARGO_TARGET_DIR=<scratchpad>/t613-target`, `CARGO_INCREMENTAL=0`, each exit status
read unpiped, on the tree committed as `0a934624f`:

* `cargo fmt --all --check`: exit 0.
* `memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings`: exit 0.
* `memcap 16G cargo test -j 4 --release -p rexx-exec --lib`: exit 0, 1155 passed.
* `REXX_REFUSAL_SITES_REFRESH=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test
  refusal_sites`: exit 0; `refusal-sites.tsv` gains the `loader_environment` row, the rest are line
  shifts.
* `memcap 16G cargo test -j 4 --release -p rexx-exec --test refusal_sites --test owners --test
  closed_phases --test phase_ten_reasons --test representation_refusals --test
  refusal_dispositions --test coverage --test rexxc --test licensed_divergences`: exit 0.
* `memcap 16G cargo test -j 4 --release -p rexx-parse`: exit 0.
* `memcap 16G cargo test -j 4 --release -p rexx-core --test unsafe_sites`: exit 0.
* `memcap 16G cargo test -j 4 --release -p rexx-exec --test concurrency_tests --test sim_processes
  --test rexx_command --test signals --test frame_block --test stdin_contention --test
  program_end`: exit 101, one failure, `sim_gate::the_table_holds_the_parts_the_spec_names`, which
  predates this task (Concerns 3).
* `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus`: exit 0,
  1135 of 1135 matching.
* `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=docs/superpowers/records/2026-10-18-phase-9d/t6.13/suite/table.tsv
  memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite`: **exit 0**, 402
  groups, all green, 250 s.

## Perf

`t6.13/callgrind.txt`: pin, parent (`t6.12-r2/rexx-head`) and head (`0a934624f`) in one run, exit
0. Own delta is head d% minus parent d% in the same run:

| program | head raw vs pin | head -libc vs pin | own -libc | own raw |
|---|---|---|---|---|
| startup | +0.0741 | -0.1200 | +0.0001 | +0.0038 |
| interpret | -1.7697 | -4.6275 | +0.0022 | -0.0015 |
| parse | -0.1206 | -0.1416 | 0.0000 | +0.0003 |
| churn | +0.1332 | +0.1068 | 0.0000 | +0.0006 |
| sayloop | +0.0473 | -0.0636 | +0.0001 | +0.0028 |
| rexxcps | +0.4765 | +0.5318 | +0.0126 | +0.0117 |
| sendloop | -0.2049 | -0.2176 | 0.0000 | 0.0000 |
| dispatch | -0.3161 | -0.1454 | 0.0000 | 0.0000 |

Every own delta is under +0.05% on both measures. Against the pin, every raw whole-process figure
is under +0.5%; with libc subtracted rexxcps is +0.53% (+0.5318), past it (Ruled 14 accepts it). rexxcps's +2.24 M Ir is all `Interp::exec_instruction`'s own instructions (cgdiff of
the r1 profiles); rexxcps runs no command, so it is the command arm's larger code inlined there
rather than work done. The new refusal is `#[cold]`.

## Concerns

1. **`.context~rs` inside INTERPRET: oracle departs from the documented rule; the crate follows
   the rule.** rexxref (`utilityclasses.xml:7821`): `rs` is "the context current return status
   value ... equivalent to using the .RS environment symbol". On the oracle, inside an INTERPRET
   after `exit 4`, `.rs` is 1 and `.context~rs` is still the caller's earlier 0, and they agree
   again once the INTERPRET ends (`t6.13/probes/interpret/`). The interpret activation keeps its own
   settings and copies them back at its end, while `.context` is the caller's object. Here both
   read 1. This is a Deviation candidate for Moritz; no Deviation or licence row is written, and the
   witness leaves the INTERPRET case out. No ooTest test reaches it.
2. **The hostemu refusal keys on the package name.** The crate cannot see what a loader registers
   through the classic API, so it refuses an unregistered environment named as a loader-run native
   package. That covers hostemu, the one non-Windows extension calling `RexxRegisterSubcom*` (`grep -rln 'RexxRegisterSubcom' extensions/ | grep -v windows`). An
   extension registering under another name still answers rc 30. A loader that runs also calls the
   oracle's `librexxapi.so` (which libhostemu NEEDs) inside this process: Phase 10's subject.
3. **Pre-existing red unit test, not this task's.**
   `concurrency_tests::...::sim_gate::the_table_holds_the_parts_the_spec_names` fails because
   `corpus/sim-gate.tsv` holds `trace_destination_waits_calling_a_label`,
   `trace_destination_waits_calling_a_method`, `trace_destination_waits_on_thread`,
   `trace_destinations_two_threads` and `trace_reply_continuation`, which `phase-6.txt` does not
   list. This task touched neither file (`sim-gate.tsv` last changed at `24da8a29b`, Task 6.10 fix
   round 2).
4. **A halt during a redirected command** now abandons it as an unredirected one is abandoned
   (rc -4, `INTERRUPTED`), and the targets are written with no lines. Not compared with the oracle.
5. The witness of the off-baton wait cannot run in the simulation mode (above). The
   `sim_blocked_command` refusal is that mode's designed answer.

## Fix round 1

Review `task-6.13-review.md`; rulings at the end of `progress.md`. Code commits `bdec52856` and
`7e42bba78`; the records commit that follows holds `t6.13/r1/` and this section.

### Important 1: the hostemu refusal

`Libraries::classic_environment` (`libraries.rs`) replaces `loader_named`. It refuses only the
environments in `CLASSIC_ENVIRONMENTS`, each a package name and the environment its loader
registers through `RexxRegisterSubcomExe`, while that package is held with its loader run. The table
holds hostemu's `HostEmu` (`hostemu.cpp:836`), derived by `grep -rln 'RexxRegisterSubcom'
extensions/ | grep -v windows`. The refusal text is unchanged, so the exclusion row still derives
(the suite's `TEST_SIGNAL_ANY_CONDITION_HALT` row is unchanged, `Phase10(Extension)`).

Witness `lang/command_library_environments_unregistered.rex` with its `.env` (the oracle's library
directory): after `::requires "rxsock" LIBRARY` and `"rxunixsys" LIBRARY`, `address rxsock`,
`RXSOCK`, `rxunixsys` and `HostEmu` (hostemu not loaded) are each rc 30 with FAILURE, `.rs` -1. The
round-0 binary refuses at the first, rc 120. `r1/probes/libraries/` has the three probes.

### Important 3: the targets during the command

rexxref (`instrc.xml`, ADDRESS WITH) gives the order: a `REPLACE` stem's lines go to `stem.1`
upwards with the count in `stem.0`, a `REPLACE` collection is emptied with `empty` "before any
output lines are added", a `REPLACE` stream "truncated to zero length before any output lines are
written". It does not say when, relative to another activity (the note saying Linux output is
available while the command runs is commented out of the source). So this follows the oracle:
`CommandIOContext::init` before the handler runs, `OutputRedirector::writeBuffer` as output arrives,
the error target written once the output has ended, and `resolveConflicts`' buffering of a target
that is also the input.

`redirect.rs`:
* `IoContext::begin`, called by `run_command` before the child starts and before a registered
  redirecting handler runs: a `REPLACE` stem is emptied (its default kept) with `stem.0` 0, an
  `APPEND` stem reads its `stem.0`, a `REPLACE` collection is sent `EMPTY`. A target that cannot be
  begun stops the command before it starts: `stem.0 = 'x'` with `APPEND` is 26.904 and the command
  never ran (`r1/probes/targets/begin_error_first`, the oracle the same; round 0 ran it first).
* `IoContext::write_output` writes each whole line of a piece as it arrives and keeps a part line;
  each stem line sets `stem.i` and then `stem.0` (`StemOutputTarget::writeLine`).
* `deferred`: the output target that is also the input source (a stem or stream object by
  identity, a named stream by name; never a collection, whose input kind differs) is begun and
  written only at the end, else the error target (`resolveConflicts`).
* `finish` writes the last part line, then the error lines, then closes each named stream.

`command.rs` and `scheduler.rs`: a parked redirected child's standard output reaches the activity
as `Posted::Captured` pieces under the wait's token; `Blocked::capture` keeps them and the activity
is unparked. The clause runs again, writes them (`end_blocked_command`), and parks again until the
child has ended. Standard error going to a target is kept whole and written at the end; a stream
that is not redirected streams to this interpreter's own as before.

Witness `lang/command_redirected_targets_mid_command.rex`: a started activity runs three redirected
commands, each child waiting for a file main writes once it has read the targets. Main sees `output
1 one error 0 E` (the error stem emptied, its default `E` kept), `array 1 three` and `same 1 in` (an
input stem that is also the output, untouched), and at the end the full targets. Oracle and head
gave one output each in 8 runs (`r1/witnesses/summary.tsv`); the round-0 binary differs. sim mode
cannot run it (commands stay on the baton there).

### Important 2: a halt during a redirected command

rexxref, HALT (`condtra.xml`): "if Ctrl+Break is pressed while a command ... is processing, the
command ends", and the condition is raised at the end of the clause. The crate follows that: the
halt abandons the wait (rc -4, as Deviation 10 gives an unredirected one), the target keeps the
lines already written, and HALT is raised. `r1/probes/halt/` (SIGINT by PID after 0.6 s):
* oracle: `HALT rc 98 stem 1 first line 10`: 98.923 "Interrupted system call" first, its SYNTAX
  trap runs, and the pending HALT is taken in the handler;
* head: `HALT rc -4 stem 1 first line 5`; round 0: `stem 0 S.1`.

The oracle's 98.923 here is the departure Deviation 12 already records ("a redirected command under
a process-group SIGINT is 98.923 ... where this crate ... raises 4.1 at the same clause"), so no
new candidate. Deviation 12's text now needs one correction (Concerns 1). Witness
`tests/signals.rs` `sigint_keeps_what_a_redirected_command_wrote`: `b -4 1 first`, `halted 5`;
its doc says the bytes are this crate's.

### Found in the round: a command's end under another wait

The first round-1 suite run had `ADDRESS` `TEST_PPREXXREF_EXAMPLE`, `TEST_SIMPLE_WITH` and
`TEST_STEM_INPUT` red. A target that raises while its child still runs
(`TEST_INPUT_OUTPUT_STREAM_ERROR`) leaves the clause, and its child's later `Unblocked` was
recorded by the next redirected command's wait, which settled with the first child's empty output.
`Blocked::end` now takes an end only under its own token, as `capture` does, and the scheduler
drops any other. Witness `lang/command_redirected_target_raises.rex`: an `OutputStream` subclass
whose `lineout` raises, then a command whose stem must hold `c d`; `bdec52856` gives `next 0 0`
(8 of 8), the oracle and head `next 0 2 c d` (8 of 8). A run of the clause with no output and no end
(a wake that was not the command's) now parks again rather than issuing the command a second time.

### Minor 1: the rooting witness

`lang/command_redirected_targets_collected.rex`: a started activity's command writes to two Array
subclass targets only the `IoContext` holds, while main allocates until it completes. Identical to
the oracle; with `blocked.object_roots(out)` removed from `Activity::object_roots` it is `rexx-exec:
a message send to a value whose object is no longer live`, rc 120, 3 of 3 (mutant built, run,
reverted by an exact copy, `git diff --stat` empty).

### Minor TRACE C

`TRACE C` now echoes an `ADDRESS` clause that issues a command. The echo is
`Interp::echo_address_command` in `exec_command` (command path only), which prints the clause line
where the setting echoes commands but not every clause. Putting it in the stepped clause gate cost
+0.06% to +0.34% on parse and rexxcps in three shapes measured (`cg-try1`..`cg-try4`, scratch), so
that path is unchanged. Witness `lang/trace_commands_address_clause.rex`: identical to the oracle,
the round-0 binary misses the `*-*` lines; a `DO` body, an INTERPRET and a `TRACE R` switch were
probed identical too.

### Deviation 51's witness

`LICENSED DIVERGENCE WITNESS: context-rs-in-interpret` in its entry, and the
`tests/licensed_divergences.rs` row: oracle `in 1 0` / `out 1 1`, crate `in 1 1` / `out 1 1`.

### The sim gate table test

`phase-6.txt` lists the five trace-destination programs `sim-gate.tsv` holds
(`the_table_holds_the_parts_the_spec_names` reads `phase-6.txt`); the harnesses read the union of
the phase files, so each still runs once. `concurrency_tests`: exit 0, 54 passed.

### Checks, on `7e42bba78`

From `rust/`, `CARGO_TARGET_DIR=<scratchpad>/t613-target`, `CARGO_INCREMENTAL=0`, each status read
unpiped:
* `cargo fmt --all --check`: exit 0.
* `memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings`: exit 0.
* `memcap 16G cargo test -j 4 --release -p rexx-exec --lib`: exit 0, 1155 passed.
* `REXX_REFUSAL_SITES_REFRESH=1 ... --test refusal_sites`: exit 0 (line shifts in `bdec52856`).
* `memcap 16G cargo test -j 4 --release -p rexx-exec --no-fail-fast --test refusal_sites --test
  owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test
  refusal_dispositions --test coverage --test rexxc --test licensed_divergences --test
  concurrency_tests --test sim_processes --test rexx_command --test signals --test frame_block
  --test stdin_contention --test program_end`: exit 0, no failures.
* `memcap 16G cargo test -j 4 --release -p rexx-parse`: exit 0.
* `memcap 16G cargo test -j 4 --release -p rexx-core --test unsafe_sites`: exit 0.
* `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus`: exit 0,
  1140 of 1140 matching.
* The whole suite with the table at `t6.13/r1/suite/table.tsv`: exit 0, 402 groups green, 288 s.
  Against `t6.13/suite/tests.tsv` no row changed (`t6.13/r1/suite/changed-rows.txt` holds only its
  command).

### Perf

`t6.13/r1/callgrind.txt`: pin, round 0 (`0a934624f`) and `7e42bba78` in one run, exit 0. With libc
and ld-linux subtracted, head minus parent is at most +0.0001% (sendloop, dispatch, startup, sayloop),
and interpret -0.0023, rexxcps -0.0016. Raw it is at most +0.0108% (interpret). rexxcps against the
pin is +0.4781 raw and +0.5302 with libc subtracted (Ruled 14).

### Concerns

1. **Deviation 12's text is now false in one clause.** It says a halt does not end "a redirected
   command's wait, which runs on the baton holder's thread ... the halt is served when the wait
   returns". A redirected wait now runs off the baton and a halt abandons it as it does an
   unredirected one (`sigint_keeps_what_a_redirected_command_wrote`). The 98.923 sentence still
   holds. The Deviation is the controller's to correct.
2. **The oracle drops a target's SYNTAX when the command ends non-zero.** With an output target
   whose `lineout` raises, `'echo a; echo b; exit 3'` runs on past the clause on the oracle (no
   42.3), where `exit 0` raises 42.3; this crate raises 42.3 in both. Not copied, not in a
   witness (the witness exits 0). A Deviation candidate if Moritz wants it recorded.
3. `ctrl_c_halts_a_redirected_command`'s doc said the wait ran on the baton; corrected.
4. `phase-6-gate.md:472` and P64 still describe the off-baton path as unredirected only, for the
   9d gate's pointer (controller ruling).

## Fix round 2

Re-review `task-6.13-rereview.md`. Code commit `54958edd3`; the records commit that follows holds
`t6.13/r2/` and this section.

### Important 1: a raising target waits for the child

rexxref is silent on when a target's condition is raised, so this follows the oracle, measured on
the re-review's `raise1.rex`, `raise2.rex` and two shapes of my own (`r2/probes/held/`, each on
the oracle, round 1 and head; head is identical to the oracle on every one, round 1 on none):
* the clause stays parked until the child ends; later pieces are still offered to the target
  (`lineout b` after the first `lineout` raised);
* a line the target raises on ends its piece: that piece's later lines and its part line are
  dropped (`r3`: `calls 3`, `part` never offered);
* the error target is still written (`err 1 e1`);
* the condition raised is the **last** one: with a 42.3 and then a 41.1 the oracle raises 41.1
  (`r4`, `r4u`);
* it is raised again at the command's clause, which the report names (`running ... line 5`,
  traceback the last raise's method line then the command), with RC and `.RS` unset and the
  cleanup skipped: no part line flushed, no named stream closed.

`Interp::write_holding_last` (`run.rs`) runs a target write where a condition may already be held:
a condition it raises replaces the held one, with the failure levels the earlier one left taken
out first (its objects pushed as temps) so the report and `TRACEBACK` are the last raise's alone;
a write that succeeds puts them back. Any other failure (an EXIT, a refusal) ends the clause at
once. `end_blocked_command` holds the condition in `Blocked::raised` across parks.
`IoContext::finish` takes the held condition, writes in the oracle's order (last output piece,
standard error, then the cleanup's part line and closes), and on a condition calls
`reraise_failure_levels` and answers it. `IoContext::write_output` clears the part line on a raise.

Witness `lang/command_redirected_target_raises_held.rex`: `trapped 41.1 calls 3 lines  b` /
`rc 41 .rs .RS error 1 e1`; oracle and head one output each in 8 runs, round 1 `trapped 42.3
calls 1`. `lang/command_redirected_target_raises.rex`'s opening comment said the condition ends
the clause while the child runs; it now says the clause raises it once the child has ended (its
`sourceline_oracle/` file regenerated). Its output is unchanged. With the clause now waiting for
its own child, that program no longer reaches `Blocked::end`'s token check, and no witness of
this round does; whether any program still reaches it was not measured.

The halt shapes (`r2/probes/halt/`, SIGINT by PID):
* `h1`: a raising target under a halt: the oracle raises 98.923 (Deviation 12), head the held 42.3.
* `h3`, `h5`, `h6`, `sigint2`: **a pre-existing defect, outside this cluster.** After a SYNTAX is
  trapped by `SIGNAL ON SYNTAX`, the pending HALT taken at the label reports `0 *-* <no failing
  clause recorded>` (untrapped) or `Error 0 ... <no message 0.0 in the catalogue>` (with
  `SIGNAL ON HALT`), where the oracle names the label clause (`3 *-* raised:`) or runs the HALT
  trap (`HALT 41 4`). `h5` and `h6` reach it with no command at all (`x = SysSleep(1.13) + 'a'`),
  and the round-1 binary gives the same (`h6.parent.txt`). Not widened; proposed as a queued
  item.

### Deviation 52's witness

`LICENSED DIVERGENCE WITNESS: target-error-whatever-the-rc` in its entry, and the
`tests/licensed_divergences.rs` row: `exit 3` with a target whose `lineout` raises, oracle
`ran on 3`, crate `trapped 42.3 42` (oracle measured 3 of 3).

### oracle-crashes.txt

Entry 56, the error stem that is also the input stem with no output target: SIGSEGV, rc 139, 3 of
3, nothing written (the re-review's `errin.rex` line). Entry 57, `two.rex` from the re-review:
an indefinite block in 3 of 6 runs at a 20 s deadline; the crate gives the four results and
`parallel 1` in 6 of 6. Both programs are in `r2/probes/oracle-crashes/`.

### Checks, on `54958edd3`

From `rust/`, `CARGO_TARGET_DIR=<scratchpad>/t613-target`, `CARGO_INCREMENTAL=0`, each status read
unpiped:
* `cargo fmt --all --check`: exit 0.
* `memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings`: exit 0.
* `memcap 16G cargo test -j 4 --release -p rexx-exec --lib`: exit 0, 1155 passed.
* `REXX_REFUSAL_SITES_REFRESH=1 ... --test refusal_sites`: exit 0, no change.
* The round-1 test-binary set (refusal tables, `licensed_divergences`, `concurrency_tests`,
  `sim_processes`, `rexx_command`, `signals`, `frame_block`, `stdin_contention`, `program_end`),
  `--no-fail-fast`: exit 0, no failures.
* `memcap 16G cargo test -j 4 --release -p rexx-parse`: exit 0.
* `memcap 16G cargo test -j 4 --release -p rexx-core --test unsafe_sites`: exit 0.
* `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus`: exit 0,
  1141 of 1141 matching.
* The whole suite with the table at `t6.13/r2/suite/table.tsv`: exit 0, 402 groups green, 277 s.
  Against `t6.13/r1/suite/tests.tsv` no row changed (`t6.13/r2/suite/changed-rows.txt`).

### Perf

`t6.13/r2/callgrind.txt`: pin, round 1 (`7e42bba78`) and `54958edd3` in one run, exit 0. Head minus
parent with libc and ld-linux subtracted: at most +0.0011% (interpret); raw at most +0.0009%
(sayloop). rexxcps against the pin: +0.4767 raw, +0.5287 with libc subtracted (Ruled 14).

### Concerns

1. The pending-HALT-after-a-SIGNAL report defect above (`h5`, `h6`), pre-existing and not a
   command's; proposed as a queued item.
2. `Blocked::end`'s token check is reached by no witness of this round (see Important 1).
