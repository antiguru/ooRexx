# Task 21 re-review 5 (fix round 5, `26acf6f83..001855da5`)

Reviewer: s4-t21-rr5. HEAD `001855da5`. Scratch: `p6-scratch/t21rr5/` (deleted at the end).

Binaries: `rexx-run.head` built at HEAD in `t21rr5/target`; `rexx-run.base` built from a `git
archive bf76afa1d` copy, every file touched, in `t21rr5/target-base` (one `Compiling rexx-exec`
line in each log). Driver: re-review 4's `drv.py` (fresh run directory per run, `env
--default-signal=INT,TERM,HUP`, oracle under `ulimit -v 1048576`), copied and re-pointed at
these binaries. "Identical" means rc, stdout and stderr.

## Summary

- Verdicts: 1 PASS, 2 PASS, 3 PASS, 4 PASS, 5 PASS.
- New findings: Low 1 (L-1, from fix round 2, not this round). No Critical, High or Medium.
- Mutants: 6 run, 6 red (2 of them only as an unbounded hang, L-1).
- Final verdict: **approved**.

## 1. The removal is complete: PASS

`git diff bf76afa1d 001855da5 -- rust/crates/rexx-exec/src` has eight files, and every hunk is
a kept item:

| File | Hunk | Kept item |
| --- | --- | --- |
| `command.rs:463-473` | `read_all` retries `Interrupted` | EINTR retry (P67) |
| `input.rs:79-84` | `read_stdin_chunk` retries `Interrupted` | EINTR retry (P67) |
| `input.rs:379-380` | the chunk read under `signal::unblocked` | P67 as amended |
| `lib.rs:2935-2939` | `install_signal_handlers` blocks in the caller | P67 |
| `lib.rs:3127-3130` | the interpreter thread calls `signal::receive` | P67 amendment |
| `run/condition.rs:513-530`, `:593` | `delayed` key, `ANY` where the condition has no entry | R1/R3 |
| `run/tests/conditions.rs` | `a_call_on_any_handler_holds_its_any_trap` | R3 witness |
| `scheduler.rs:660-664` | the command wait under `signal::unblocked` | P67 |
| `scheduler/pool.rs:178`, `timer.rs:440-441` | pool and timer threads block | P67 |
| `signal.rs` | `halting`/`mask`/`block`/`receive`/`unblocked`; the `alone` helper and `a_halt_readies_a_set_aside_activity_once` | P67; R2 witness |

`fill_stdin` / `fill_stdin_keeping` (`input.rs:353-398`) are `bf76afa1d`'s text plus the
`unblocked` wrap. Nothing else under `rust/` differs from `bf76afa1d` outside `rexx-exec`'s
`tests/` (`signals.rs`, `stdin_contention.rs` and its `cases`), and outside `rust/` only
`phase-4-exclusions.txt` (entry 13) differs.

Leftovers searched (`git grep -nE
"stdin_turn|input_readers|bootstrap_stdin|ParkReason::Stdin|ParkReason::Input|Failure::Parked|end_stdin_turn"
001855da5 -- . ':!.superpowers'`): two hits, both in `docs/superpowers/records/2026-09-12-phase-7/ledger.md`
(Phase 7's own `bootstrap_stdin`, a historical record). `grep -rnE "P66|P68|guard queue|guard
turn|..."` over `rust/crates`, `phase-6-gate.md` and `phase-4-exclusions.txt`: one hit,
`scheduler.rs:1018` "guard queues", which is the method-guard queue in `cancel_wait` and is
unchanged since `bf76afa1d`. `phase-6-gate.md` is untouched since Task 18 (its S4 table is a
dated Task 18 record) and names neither ruling. The report's fix round 3 and 4 sections describe
P66 and the guard queue as what those rounds did; the fix round 5 section says both are
withdrawn. `stdin_contention.rs`'s module doc describes the harness only.

## 2. Stdin-contention matrix: PASS

All 63 cells of re-review 4's matrix (`mx_R_S`, readers R 0-6 x second accesses S 0-8), one run
each on the oracle, `rexx-run.base` and `rexx-run.head`, input `one`/`two`/`three` at 1.0 s:

- head identical to base in 63/63;
- head identical to the oracle in 56/63; the 7 others are column 1 (`QUERY EXISTS` answers `''`,
  the oracle `STDIN`; N-5, the same on base);
- no cell refuses: rc 0 in 63/63, stderr empty.

This covers re-review 4's 27 N-1 cells (rows 1-5, columns 0-3, 5 and 8, minus P68's three).
Repeated 15 times per side, interleaved: `mx_2_3` (`charin(,,1)` both sides) 15/15 `main [n]` /
`r [o]` on all three; `mx_1_2` (`linein(,,1)` both sides) 15/15 `main [two]` / `r [one]` on all
three.

## 3. Concern 1 (`mhalt1`, `mhalt3`): PASS

15 runs per side, interleaved, input at 1.0 s:

| Program | Oracle | Base `bf76afa1d` | Head |
| --- | --- | --- | --- |
| `mhalt1` (PARSE PULL) | 15/15 `halt sent 1` / `r halted from main` / `r read [one]` / `r next` / `r done` | 15/15 `r read [one]` / `r next` / `halt sent 1` / `r done` | the same as base, 15/15 |
| `mhalt3` (`linein()`) | the same as `mhalt1` | the same as `mhalt1` | the same as base, 15/15 |

So the difference predates P66 and is the starvation; entry 13 names `mhalt1` with these lines.

## 4. Kept behaviour: PASS

Single activity, head against the oracle, signal at 0.5 s, `hi` at 2.0 s:

| Program | Signal | Runs per side | Result |
| --- | --- | --- | --- |
| `parse pull v` | SIGINT by PID, by group | 5, 5 | identical: 4.1 with the `Monitor` frame at 1457 and `2 *-* parse pull v` |
| `v = linein()` | SIGINT by PID, by group | 5, 5 | identical, 4.1 |
| `address system 'sleep 3'` | SIGINT by PID, by group | 5, 5 | identical, 4.1 at line 2 |
| `parse pull v` | SIGTERM, SIGHUP | 3, 3 | identical, 4.1 |
| `address system 'sleep 3'` | SIGTERM, SIGHUP | 3, 3 | identical, 4.1 |
| CALL ON HALT, then `parse pull v` | SIGINT by PID, by group | 5, 5 | identical: `halted 3` / `b []` |

Group program (`say 'a'` / `address system 'sleep 2'` / ..., SIGINT to the group at 0.5 s), head,
under fix round 3's `load.sh 48` (load average 26.6 by the end): 60/60 rc 252, `a`, 4.1 at line
2.

`n3any` (SIGINT at 0.5 and 1.2 s), 10 runs per side: identical 10/10, `ready` / `main after` /
`h in` / `h out`. R3's program (`call on any`; the handler runs `exit 2`), 5 runs per side:
identical, `h in ERROR` / `h out 2` / `main after`.

Signals binary (`target/release/deps/signals-e26401c7bf9c40c4`, from the gate build), 20 runs
with `--nocapture`, load average 1.4-2.3: 37 passed in each, and no `run again` line in any
log.

## 5. DEVIATIONS 13: PASS

Each measured sentence, re-run:

- The stamped probe (8 lines 0.25 s apart, `hi` at 2 s; stamps from `TIME('F')` under NUMERIC
  DIGITS 20, passed to the activity), 3 runs per side: oracle `w 1 0.0` ... `w 8 1.8` then `main
  hi 2.0`; head and base `main hi 2.0` then `w 1 2.0` ... `w 8 3.8` (`w 2` 2.2 or 2.3).
- `keptend` (SIGINT at 1 s, `hi` at 2 s), 5 runs per side: oracle `w 3` / `wh 3` / `mh` / `main
  []` / `end`; head and base `mh` / `main []` / `w 3` / `end`, every run.
- `mhalt1`: as in point 3.

The scope "a started activity that has not yet run" is right: an activity that ran and parked
before the read is halted. Probe (the activity says `w in` and sleeps 3 s; main reads after 0.2
s; SIGINT at 1 s; `hi` at 2 s), 5 runs per side: identical stdout on all three, `w in` / `mh` /
`main []` / `w out` / `wh` / `end` (head and base end at about 2 s, the oracle at 3 s: P59's
licensed SysSleep cut, DEVIATIONS 10). The two named witnesses exist in `tests/signals.rs` and
assert the starvation order.

## New findings

### L-1 (Low, from fix round 2; predates this round). The posts a read keeps aside are witnessed only by a hang

**Where.** `input.rs:386-390` keeps every post other than `Input` and `Halt` in `kept`, and
`input.rs:357` requeues them after the read. The one test that reaches it is
`tests/signals.rs:1004` `a_stdin_read_waits_without_spinning`, where a started activity's
command ends (`Posted::Unblocked`) while main reads. That test waits with
`wait_with_output()` and no deadline, and asserts stdout and CPU ticks but not the exit status.

**Failure scenario.** Mutant M1 (`other => drop(other)`) and M2 (`drop(kept)` for the requeue):
the started activity's command end is lost, so after `v=[hi]` the program waits for that
activity forever. The suite does not fail; it hangs. M1's run took 1085 s and finished only
because I killed the child `rexx-run`; the test then passed (`v=[hi]` was already written).
`stdin_contention` passes under both: none of its cases ends a command during a read.

**Fix direction.** Give the test a deadline (kill and fail after a few seconds) and assert the
exit status, or add a `stdin_contention` case whose started activity runs a command while main
reads. Not a blocker: the code is `bf76afa1d`'s and is right.

## Mutants

Mutations made with the Edit tool and reverted the same way; `cmp` against a copy saved before
the first mutation after each revert, and `git status` clean of tracked edits at the end. Run:
`memcap 8G cargo test --profile mutation -p rexx-exec --lib --test signals --test stdin_contention
--no-fail-fast` (M1 used `--release`), one `Compiling rexx-exec` line each, with a watchdog that
kills a `rexx-run` child older than 90 s (M2 onward). Unmutated under the mutation profile: lib
1000, signals 37, stdin_contention 1, all pass.

| Mutant | Site | Result |
| --- | --- | --- |
| M1 posts kept during a read are dropped | `input.rs:389` | **hang** in `a_stdin_read_waits_without_spinning` (L-1); nothing fails |
| M2 kept posts not requeued | `input.rs:357` | **hang**, the same (watchdog kill after 94 s); nothing fails |
| M3 a halted read reads on | `input.rs:395` (`return false` removed) | red: signals 7 (`sigint_ends_a_parse_pull`, `_a_linein`, `_a_charin`, `_a_stdin_linein`, `call_on_halt_sees_an_interrupted_read`, `a_halted_read_loses_no_input`, `a_halt_during_a_read_misses_an_activity_that_has_not_run`), stdin_contention 1 |
| M4 the trap's own key not delayed (`true` arm answers `ANY`) | `run/condition.rs:520` | red: lib 2 (`a_call_handler_reports_call_and_delay_and_leaves_nothing_behind`, `a_message_halt_inside_its_handler_is_dropped`), signals 2 (`a_second_signal_ends_a_handlers_sleep`, `a_second_signal_in_a_call_on_halt_handler_is_dropped`) |
| M5 `unblocked` does not restore the mask | `signal.rs:171` | red: `only_the_interpreter_and_its_waits_take_the_halting_signals` |
| M6 P66 returns | `git archive 4a20439fa` (after P66, before the guard queue) with HEAD's `tests/signals.rs`, own target dir | red: `no_activity_runs_while_main_reads` (oracle order `w 1`..`w 8` / `main hi`) and `a_halt_during_a_read_misses_an_activity_that_has_not_run` (`w 3` / `wh 3` / `mh` / ...) |

So the two starvation witnesses go red if the run-others path comes back, and pass at HEAD.

## Checks (P51)

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0, one `Checking rexx-exec` line.
- `cargo test --workspace --release --no-run`: exit 0. Then `memcap 8G cargo test --workspace
  --release` in the foreground: exit 0, 144 `test result` lines, 3030 passed, 0 failed, 4
  ignored.
- All statuses read from `$?` of the command itself, unpiped, in `t21rr5/target`.

## Final verdict

**Approved.** P66 and P68 are gone with nothing left behind, every remaining difference from
`bf76afa1d` is a kept item, the 63-cell matrix is identical to `bf76afa1d` with no refusal,
concern 1 is true at `bf76afa1d`, the kept signal behaviour matches the oracle, and DEVIATIONS 13
is true as worded. L-1 is a witness gap from fix round 2 and can be queued.
