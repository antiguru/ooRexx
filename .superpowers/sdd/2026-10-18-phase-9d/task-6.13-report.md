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

Every own delta is under +0.05% on both measures, and every whole-process figure against the pin is
under +0.5%. rexxcps's +2.24 M Ir is all `Interp::exec_instruction`'s own instructions (cgdiff of
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
