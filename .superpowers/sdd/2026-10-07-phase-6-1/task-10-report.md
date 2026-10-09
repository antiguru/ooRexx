# Task 10 report: the seeded gate

Base `0e9d6fe6e`. Commits `39f14832b` (the gate), `902de404e` (rulings, gate programs, scratch
paths), and the commit carrying this report and the gate record. Scratch `/tmp/claude-1000/p61/t10/`.
Paths below are under `rust/crates/rexx-exec/` unless they start with `rust/` or `docs/`.

## What was built

* `tests/support/group_runner.rs`: `SwitchMode::Sim(String)` (no longer `Copy`). `run_crate_within`
  in that mode, through `run_crate_sim` and `run_crate_process`, runs `rexx-run` as a process with
  `REXX_SWITCH_MODE` set, in a process group of its own killed at the deadline (`Ended::Killed`);
  the in-process path is unchanged (`crate_environment` factored out).
* `tests/concurrency_tests.rs`, `group_runs::whole_groups::sim_gate` (`left_out_of` factored out of
  `rows_of`):
  * `the_seeded_gate` (`REXX_CORPUS_GATE`): every row of `rust/corpus/sim-gate.tsv`, seed i =
    `splitmix(fnv("group:part"), i)`, policy = the mix's entry i mod 7 with the row's k; knobs from
    the row. Each run is judged (`judge`): hang (killed), crash (signal), panic, invariant
    (`the scheduler found`), determinism (foreign post, replay divergence), design limit (unless the
    oracle set has a run that did not end), no `rexx-sim:` line, and except under `pct` and in
    rows injecting failures (`fail=`, `halt@`) a check: an ooTest failing test outside the oracle
    sets' failing tests and `DIFFERING`'s normal-mode key (R1), or a gate program's `FAIL` line.
    A check red in a `WALL_CLOCK` test is rerun once (R6/P48). Reds matching
    `rust/corpus/sim-exempt.tsv` (group, part, test, kind, refusal text) are listed, not failed.
    Each outcome's digest (stdout masked, `rexx-sim:` lines dropped, status) is compared with the
    part's committed oracle set; differences, reruns, reds and exempted reds go to the report
    (`REXX_SIM_REPORT`, else `target/tmp/sim-gate-report.txt`), with a replay line per red.
  * `a_seeded_run_repeats_in_a_fresh_process` (Step 4), `the_oracle_sets_grow_only_under_refresh`
    (`REXX_SIM_ORACLE_REFRESH=1`: 5 oracle runs, 30 where they vary or none matches seed 0 under
    `fifo`; merged into the committed set), `calibration` (`REXX_SIM_CALIBRATION=FILE`), and
    non-gated tests of the seed rule, masking, key parsing and the table's parts.
  * Env: `REXX_SIM_SEEDS` (count), `REXX_SIM_POLICY` (every seed's policy), `REXX_SIM_ONLY` (`GROUP`,
    `GROUP:PART`, or `GROUP:PART:SEED:POLICY`), `REXX_SIM_JOBS` (default 8).
  * A run's scratch path is named by its unit (`unit_dir`), since a program may read its own path:
    MethodArgs derived took 49749 or 49759 steps under one seed depending on the PID in the path
    (`/tmp/claude-1000/p61/t10/probe/ma*`); with unit paths three replays gave 49749 each.
* `rust/corpus/sim-gate.tsv` (196 rows), `rust/corpus/sim-exempt.tsv`, `rust/corpus/sim-oracle/`
  (one file per row not injecting failures).
* Gate programs `tests/sim_gate/`: `m11_stale_sleeper.rex` (`fail=wait:1`), `n1_halt_ready_once.rex`
  (`halt@30000`), `timer_post_wakes_every_waiter.rex` (a `FAIL` line where one post leaves a waiter).
* R3 fix: `src/command.rs` sets `REXX_SIM_CHILD` (`sim::SIM_CHILD_ENV`) for a sim child, and
  `src/bin/rexx-run.rs` writes no `rexx-sim:` line where it is set. Crate test
  `tests/sim_processes.rs` `a_child_writes_no_sim_line_into_output_its_parent_captures` (captured
  lines `1`, `child sim:`; the parent's own line stays); with the filter replaced by `true` it fails
  with 2 captured lines, the second the child's `rexx-sim:` line.
* `docs/superpowers/records/2026-10-07-phase-6-1/notify-window.{rex,md}` (R5 evidence).
* The scheduler test programs are not in the gate: they choose their switch mode in code
  (`src/scheduler/tests.rs`).

## Calibration (Step 1)

`REXX_SIM_CALIBRATION=FILE ... cargo test [--release] -p rexx-exec --test concurrency_tests
sim_gate::calibration`: seed 0 of each part under `fifo`, both builds, jobs 8. k is the run's
`contended=`; the k column of `sim-gate.tsv` is the release run's (debug identical on every row).
Rows: 196 (59 with k 0). Sum of calibrated wall times: release 14.2 s, debug 19.5 s; largest
DateTime whole 5.0 s / 8.1 s, then `sleeper_wakes_busy_main.rex` 0.47 s / 0.84 s. The full table is
the committed `sim-gate.tsv` (k, both times per row). Seeds: 70 per row in release (10 per policy);
debug subset 14 per row on the 128 rows that are derived parts, programs with k at least 1, the
gate programs and MutexSemaphore single. Deadline per run: calibrated time x 10, at least 60 s.

Budget: release gate 10 min, debug subset 2 min. Measured: release 13720 runs in 124 s (whole
`sim_gate::` binary 2:05 wall, max RSS 78 MB); debug 1792 runs in 12 s (22 s wall).

## Reds found while building the gate (Step 2)

Found by the exploratory release gate, 7 seeds per part (one per policy), reports
`/tmp/claude-1000/p61/t10/gate1.txt` and `gate2.txt`. Each sent to the controller for a ruling;
none exempted or fixed before the ruling.

### R1. The crate's own normal-mode failures (ruling R6 read literally)

Parts: STREAM whole, Class whole, Method rest, Object rest, RexxContext rest, ATTRIBUTE whole and
derived, CONSTANT rest, METHOD rest and derived, RAISE whole and derived, TRACE whole,
TRACE_TraceObject whole and derived. Every seed of every non-pct policy reds on the same tests,
e.g. `check base/keyword/RAISE.testGroup:derived:...:pre:1,k=3 in TEST_RAISE_INSERT_CRLF`. Each failing
set is the one `whole_groups`' `DIFFERING` lists for the part in normal mode (the shipped scheduler):
these are not scheduling outcomes and fail without sim. Proposed (implemented, pending ruling): R6's
known set is the union of the oracle outcomes' failing tests and the `DIFFERING` normal-mode key's
failing tests of that part (`failing_unswitched`). With it, gate2 has none of these reds.

**Ruling (controller): accepted** with two conditions, both held: a failure outside the union stays
red, and the key is read from `DIFFERING` itself (`failing_unswitched`), not copied.

### R2. STREAM: virtual clock against real file timestamps

`base/bif/STREAM.testGroup` whole and derived, every seed: TEST_QUERYDIR_EXISTS (and in whole also
TEST_QUERYFILE_EXISTS_NOTOPEN) fail. The tests compare `.DateTime~new` with the file system's
timestamp of a file they just made (`STREAM.testGroup:603-621`, `:855-876`); under sim the clock is
virtual from a seeded 2026 epoch, the file system real. Replay:
`REXX_CORPUS_GATE=1 REXX_SIM_ONLY='base/bif/STREAM.testGroup:whole:6583656226591860901:pre:1,k=1' memcap 8G cargo test -j 4 --release -p rexx-exec --test concurrency_tests the_seeded_gate -- --nocapture`
(profile release, stack 536870912). Test-environment assumption, not a defect. Proposed: the spec's
own `clock=real` knob on both STREAM rows (row configuration, not an exemption). With it, 21 seeds of
each part: whole gives the normal-mode key (assertions 195, `DIFFERING`'s failing set), derived passes
21 of 21 (`/tmp/claude-1000/p61/t10/stream-real.txt`).

**Ruling: accepted**, the row's reason naming both tests (a comment above the STREAM rows of
`sim-gate.tsv`). Self-test on STREAM derived under `clock=real`: two fresh processes, trace
`cbf29ce484222325` both (the row's k is 0, so no decision is taken) and identical outcomes.

### R3. bug2003_guard_when: a child's `rexx-sim:` line inside the program's own output

`regressions/bug2003_guard_when.testGroup` whole and derived, TEST_GUARD_WHEN_1, every non-pct seed.
The test runs `rexx guard_when.rex` through `issueCmd`, which appends the child's stderr to the same
array as its stdout (`ootest/framework/FileUtils.cls:71-84`), then asserts 10 lines. Under sim the
child `rexx-run` prints its `rexx-sim:` line on stderr, so the array holds 11: the -V 2 detail reads
`Failed: assertEquals / Expected: 10 / Actual: 11` (manual run of the group copy under
`sim:11026779930089115553,pre:1,k=1`). Probe: `address system "rexx c.rex" with output append using
(o) error append using (o)` under `sim:5` gives `o~items` = 2, the second item the child's
`rexx-sim: seed=12053802520126558431 ...` line. Replay:
`REXX_CORPUS_GATE=1 REXX_SIM_ONLY='regressions/bug2003_guard_when.testGroup:whole:11026779930089115553:pre:1,k=1' memcap 8G cargo test -j 4 --release -p rexx-exec --test concurrency_tests the_seeded_gate -- --nocapture`.
The harness masks `rexx-sim:` lines in the stream it reads; it cannot mask a line the program itself
captured. Not a scheduler defect: the sim report perturbs what the program observes. Options: (a) a
child prints no `rexx-sim:` line (the command path that sets the child's `REXX_SWITCH_MODE`,
`command.rs:561`, also marks it as a child, and `rexx-run` omits the line for one), a `src/` change
with a crate test; (b) an exempt row, which takes the bug 2003 GUARD WHEN regression test out of the
gate. Recommended: (a).

**Ruling: option (a)**, done as described under "What was built" (`REXX_SIM_CHILD`, crate test
with a mutation shown failing). With it, bug2003_guard_when passes on every seed of the close run.

### R4. MutexSemaphore: P46's deadlock, named by the last started test

`base/class/MutexSemaphore.testGroup` whole (uniform:0.2 seed 13676852389019854076, uniform:1 seed
10163017869986652146) and derived (uniform:1 seed 1363244776685834026): `rexx-exec: a wait in the
simulation mode that only a signal can end is not implemented`, rc 120, last started
TEST_RELEASE_ONE_ARG. Replay:
`REXX_CORPUS_GATE=1 REXX_SIM_ONLY='base/class/MutexSemaphore.testGroup:whole:13676852389019854076:uniform:0.2' memcap 8G cargo test -j 4 --release -p rexx-exec --test concurrency_tests the_seeded_gate -- --nocapture`.
Attribution: 60 seeds under `uniform:1` of the whole part deadlock 60 of 60; the same 60 seeds with
TEST_EXCLUSION also left out pass 60 of 60 (`mutex-whole-uniform:1.txt`, `mutex-rest-u1.txt`). The
worker TEST_EXCLUSION starts waits forever on the mutex main took (ruling P46), so main deadlocks at
the program's end, after later tests started. The P46 row as briefed (test TEST_EXCLUSION) cannot match:
the red names the last started test. Proposed: the P46 exempt row with test `*`, kind
`design-limit`, this evidence added.

**Ruling: narrowed.** MutexSemaphore whole and derived leave TEST_EXCLUSION out (the row's
`left_out` column, the mechanism the rest rows use) and a `single` row runs it alone (`-t
TEST_EXCLUSION`); the exempt row is that single row, test TEST_EXCLUSION, kind design-limit, keyed
on `a wait in the simulation mode that only a signal can end`, with the 60/60 against 0/60
evidence. Close run: the single row ended so on 20 of 70 seeds, all exempted; whole and derived
(now with no contended step) pass.

### R5. message_notify.rex: `~notify` racing the notification loop

`corpus/lang/message_notify.rex`, pre:3 seed 1293537187515384856 (hangs at the `m~start` section) and
uniform:0.2 seed 11946169623404229269 (at the `r = m~reply` section): `rexx-exec: a wait that nothing
left to run can end is not implemented`, rc 120. Replay:
`REXX_CORPUS_GATE=1 REXX_SIM_ONLY='corpus/lang/message_notify.rex:program:1293537187515384856:pre:3,k=12' memcap 8G cargo test -j 4 --release -p rexx-exec --test concurrency_tests the_seeded_gate -- --nocapture`.
Cause: main's `m~notify(w)` lands while the started message's activity is inside its notification
loop, running a party's Rexx `messageComplete`. The oracle has the same window:
`MessageClass::notify` appends and notifies at once only if `allNotified()`
(`interpreter/classes/MessageClass.cpp:217-229`), and `sendNotification` fixes the count before the
sends and sets all-notified after them (`:666-685`), so a party appended in between is never sent
`messageComplete`; `notify_parties` (`src/dispatch/object_protocol.rs:1346-1375`) does the same. Oracle
twin, a probe forcing that interleaving (`/tmp/claude-1000/p61/t10/probe/mn2/notify_window.rex`: the
first party's `messageComplete` sleeps 1 s, main notifies `w` after 0.3 s and waits on it): the oracle
hangs 5 of 5 (killed at the timeout, `notified 1` printed, `waited` never); the crate in normal mode
refuses with the same message, rc 120. A program race the oracle's 24 ms slice almost never shows, not
a crate defect. Proposed: an exempt row (`corpus/lang/message_notify.rex`, `program`, `-`,
`design-limit`) with the probe committed as its evidence.

**Ruling: accepted, narrowed** to the refusal `a wait that nothing left to run can end`. The probe
and its result are committed as `docs/superpowers/records/2026-10-07-phase-6-1/notify-window.rex`
and `.md` (oracle command quoted; rerun 2026-10-09: rc 137 on 5 of 5, stdout `notified 1`). Close
run: 14 of 70 seeds, all exempted.

### R6. STREAM under `clock=real`: the virtual sleep crosses a second the file system does not

With `clock=real`, STREAM whole failed TEST_QUERYDIR_EXISTS on 3 of 70 seeds (uniform:0.2
9007989191373000343, pre:1,k=1 4638698144989352312, uniform:0.01 7856179057164274178) and the replay
of one passed 6 of 6. The test (`STREAM.testGroup:862-876`) sleeps 0.05 s while the time is past
.95 of a second and then compares the directory's mtime second with that time; the virtual clock
jumps the sleep while real time does not, so a run starting in a second's last 50 ms crosses into the
next. `clock=real` sets only where virtual time starts (spec section 4, the clock row: virtual time
"advances a seeded quantum per clause and jumps to the next deadline when every activity waits",
`clock=real` among its origin knobs), so the sleep's jump is the specified behaviour, not a sim
inconsistency. **Ruling: (a)**: a check red in a `WALL_CLOCK` test (read from that list) is rerun
once with the same seed and mode, only a second failure red, never for another kind; every rerun is
written to the report. Close run: STREAM whole 2 and derived 7 first failures, all passed on rerun;
SysSleep's reruns are the exempt TEST_SLEEP_DURATION.

### whole_groups: TRACE whole (found at the Task 10 close run)

`whole_groups` at `902de404e` was red on TRACE whole, both modes: assertions 129, failing [DROP EXIT
EXPOSE IGNORED LABEL_WITH_FORWARD OTHER_ENTRYPOINT PROCEDURE], where `DIFFERING` listed 120 with
TEST_TRACE_?, ?A, ?I, ?R, ?_OPTION and NUMERIC_DEBUG failing too. Base `0e9d6fe6e` gives 129 as
well. **Ruling: rewrite both keys and bisect.** Bisect over the commits of `0d18b0045..0e9d6fe6e`
touching `rust/`, each a `git archive` copy built in one reused target dir (tree touched, one
`Compiling rexx-exec` line each), TRACE.testGroup whole in normal mode, stdin `/dev/null`
(`/tmp/claude-1000/p61/t10/bin/trace-at.sh`): `72287a744` (Task 5a, last) 120;
`f062f791e` (Task 6, "debug pause placement and .DebugInput") 123, TEST_TRACE_? and ?_OPTION pass;
`004312db8`, `ca3a558b6`, `ac6cb27f6`, `5495d5a08` 123; `072e536ac` (Task 6, "handler EXIT witness,
banner per activation") 129, ?A, ?I, ?R and NUMERIC_DEBUG pass; `58a558ff1` (Task 7) 129. So Task 6
moved the row, in two commits. Oracle: the whole group 5 of 5 rc 0, 147 assertions, nothing failing;
each moved test alone (`-t`) on the oracle and on HEAD's release `rexx-run`: identical stdout (timing
lines masked), stderr and status, rc 0 on both, for all of them. Keys rewritten in `b8bba37a2`;
`whole_groups` then exit 0, 14 passed.

## Step 4: self-test

`a_seeded_run_repeats_in_a_fresh_process` over the 128 debug-subset rows, seed 0 under `pre:2`,
each run twice as its own `rexx-run` process at the same unit path: 128 of 128 same trace hash and
same outcome (`REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 --release -p rexx-exec --test
concurrency_tests sim_gate::a_seeded_run_repeats -- --nocapture`, exit 0).

## Step 5: the gate can fail

Each revert on a `git archive` copy of HEAD (`902de404e`) in `/tmp/claude-1000/p61/t10/rv/`, one
hunk (`/tmp/claude-1000/p61/t10/bin/mutate.py`, `revert-gate.sh`), the release gate run whole, one
`Compiling rexx-exec` line each. The hunks, found with `grep -n 'Task 8\|Task 21\|Task 4'
.superpowers/sdd/2026-10-01-phase-6-s2-s5/progress.md` (lines 125-127 and 302-303):

| defect | commit, file | revert | gate |
|---|---|---|---|
| Task 8 I1, one timer post wakes only the last waiter | `7eafa3b58`, `src/scheduler.rs` `post_timer_at` | the post takes the last waiter only; the others stay parked | red, 60 reds, `check timer_post_wakes_every_waiter.rex`, `FAIL one post woke 0 1` |
| Task 21 N1, a halt readies an already-ready timed waiter | `f9fb61990`, `src/scheduler.rs` `make_ready` | readying pushes unconditionally | red, 24 reds, `invariant n1_halt_ready_once.rex`, `the scheduler found an activity both running and ready` |
| M11, `cancel_wait` keeps the sleeper | `src/scheduler.rs` `cancel_wait` (`:1201-1203` at HEAD) | the `sleepers.retain` deleted | red, 53 reds, `invariant m11_stale_sleeper.rex`, `the scheduler found a ready activity holding a park reason` |

Replay lines (release, stack 536870912), each on its reverted copy:

* I1: `REXX_CORPUS_GATE=1 REXX_SIM_ONLY='crates/rexx-exec/tests/sim_gate/timer_post_wakes_every_waiter.rex:program:7759006598199263188:pre:1,k=4' memcap 8G cargo test -j 4 --release -p rexx-exec --test concurrency_tests the_seeded_gate -- --nocapture` (`REXX_SWITCH_MODE=sim:7759006598199263188,pre:1,k=4`)
* N1: `REXX_CORPUS_GATE=1 REXX_SIM_ONLY='crates/rexx-exec/tests/sim_gate/n1_halt_ready_once.rex:program:12889984058210536791:pre:1,k=90394' ...` (`sim:12889984058210536791,pre:1,k=90394,halt@30000`)
* M11: `REXX_CORPUS_GATE=1 REXX_SIM_ONLY='crates/rexx-exec/tests/sim_gate/m11_stale_sleeper.rex:program:7462166462821798847:pre:1,k=1' ...` (`sim:7462166462821798847,pre:1,k=1,fail=wait:1`)

The first gate runs on I1 and N1 (before the gate programs existed) were green, 13580 runs each: no
part of the ooTest groups or `phase-6.txt` reaches either revert observably. For I1 the spec's "a
parked activity with no wake source" does not hold in the current shape: the waiter left out keeps
its sleeper's deadline, so the revert changes output (`after one post 0 1` on the corpus witness
`timer_post_wakes_every_waiter.rex`, `phase-8.txt`, under `sim:1` and `every`) and breaks no
invariant; its gate program is the corpus witness with a `FAIL` assertion line (oracle: `one post
woke both`). N1 needs a halt landing while a deadline-woken waiter is ready: the gate program runs
under `halt@30000`; probe on the N1 copy, 8 seeds each, red 8/8 under pre:1 and pre:3, 0/8 under
pct and uniform. The alternate (Task 4 I2) was not needed.

Origin failure (REPLY derived under every opportunity, rc 1, `bg/b6efbfd53/logs/g4-test-release.txt:2210-2213`:
`pass, assertions 18, rc 1`): under `uniform:1`, 300 seeds each of REPLY derived and whole at HEAD:
derived 300/300 `pass, assertions 18, rc 0`, whole 300/300 `pass, assertions 19, rc 0`
(`REXX_SIM_ONLY=base/keyword/REPLY.testGroup:derived REXX_SIM_POLICY=uniform:1 REXX_SIM_SEEDS=300`).
Not reproduced.

## Step 6: per-task check

* `cargo fmt --all --check`: exit 0 (at `902de404e` and `b8bba37a2`).
* `memcap 8G cargo clippy -j 4 --workspace --all-targets -- -D warnings`: exit 0 at `902de404e`'s
  tree (warm target dir; `rexx-exec`'s test targets were re-linted after each edit).
* `memcap 8G cargo test -j 4 --workspace --no-fail-fast` (debug) at `902de404e`: exit 0, 3119
  passed, 0 failed, 4 ignored (`/tmp/claude-1000/p61/t10/logs/ws.txt`).
* `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec --test corpus --test
  ir_recorded_oracle`: exit 0, corpus 29 passed 1 ignored, ir_recorded_oracle 21 passed.
* `REXX_REFUSAL_SITES_REFRESH=1 cargo test -p rexx-exec --test refusal_sites`: re-derived (line
  shifts from `lib.rs`'s `pub use sim`); then `refusal_sites` 5 passed, `refusal_dispositions` 3
  passed. No `Loud` constructor added.
* Release gate and `whole_groups`: `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 1 --release -p
  rexx-exec --test concurrency_tests whole_groups` at `b8bba37a2`: exit 0, 14 passed, 5:20 wall, the
  seeded gate 13720 runs in 126 s, no red, exempted reds SysSleep 99, MutexSemaphore single 20,
  `message_notify.rex` 14 (`/tmp/claude-1000/p61/t10/gate-final.txt`).
* Debug gate: `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec --test concurrency_tests
  sim_gate::` at `902de404e`: exit 0, 8 passed, 1792 runs in 12 s.

## Concerns

1. A sim child (`rexx` run as a command) is started in a process group of its own
   (`command.rs`), so the gate's deadline kill of the run's group does not reach it; a hung child
   is ended only by the sim watchdog (`block=`).
2. The oracle report compares digests; a program part's key holds only status and line counts, so
   a difference has to be read through its replay line.
3. The k column comes from a calibration run at a PID-named path; with unit-named paths the step
   count of path-sensitive parts (MethodArgs) differs by a few steps. k only bounds where `pre`
   and `pct` draw their points.
4. `/tmp` ran out of inodes mid-task (my gate leaked an empty directory per run, fixed; other
   tasks' trees were the bulk).

## Fix round 1

Review: `task-10-review.md` (Needs fixes). Code commit `705dfb767`; this section in the commit after
it. Line numbers at `705dfb767`, paths under `rust/crates/rexx-exec/`.

* **I1, early end.** `judge` (`tests/concurrency_tests.rs:4171-4176`) calls `early_end` (`:4223`), red
  kind `abort`, for every row not injecting failures. It reds a refusal that neither the part's
  oracle outcomes nor its `DIFFERING` normal-mode key (`differing_keys`, `:4210`) names, every policy.
  Except under `pct`, it also reds a group part with no ooTest summary, and a status none of them
  has in a run naming no failing test (the origin's `pass, rc 1`). Failing tests stay with the check
  route. A design-limit refusal whose oracle twin hangs now ends judging (`:4163`), so it is not
  re-judged as an abort. Pinned by `a_run_ending_where_no_listed_outcome_ends_is_red` (`:4877`):
  - a pass summary at rc 1 is red, at rc 0 is not, and under `pct` is not;
  - an empty stdout is red;
  - Section1's listed `OBJECTNAME=` refusal is not red;
  - an unlisted `COPY` refusal is red in Section1 (pre and pct) and in a part with no `DIFFERING`
    row.

  With the call disabled (`if false && ...`) the test fails (`left: []`).
  Section1 whole does not go red: its refusal is its `DIFFERING` normal key's, the known-set route the
  review names. The release gate at `705dfb767` showed no new red, so none went to the controller.
* **Minor 1, deadline kill.** The doc's false claim is deleted (`tests/support/group_runner.rs:536-538`
  now says a command a sim run starts is in a group of its own, which the kill does not reach). Not
  fixed in code: it is bounded (readers time out, a hang is already red); recorded as concern 1.
* **Minor 2, WALL_CLOCK rerun.** In `run_unit`, a rerun that passes on a row without `clock=real`
  (`Row::real_clock`, `:3674`) is a `determinism` red, with the failed test named. Close run: the
  reruns were STREAM (`clock=real`) and SysSleep (rerun failed, exempt), so no such red.
* **Minor 3, scratch.** `unit_dir` (`:4349`) is `<target tmp>/sim-gate-<profile>/<unit hash>/run`
  (`profile`, `:3562`). The replay line prints that path (`:3761`).
* **Minor 4, environment.** `REXX_SIM_*` variables are dropped from the program's environment, both
  for group parts (`crate_environment`, `group_runner.rs:481`) and for programs (`:3892`). No child
  needs them: a sim child gets `REXX_SWITCH_MODE` and `REXX_SIM_CHILD` from the command that starts it
  (`src/command.rs`).
* **Minor 5, stdin.** The sidecar's stdin is written from a thread of its own
  (`group_runner.rs:590-597`).
* **Minor 6, gate programs.** `m11_stale_sleeper.rex` and `n1_halt_ready_once.rex` say in their
  header comment that the gate reads only the invariants, not their stdout.

Per-task check at `705dfb767`:
* `cargo fmt --all --check` exit 0.
* `memcap 8G cargo clippy -j 4 --workspace --all-targets -- -D warnings` exit 0.
* `memcap 8G cargo test -j 4 --workspace --no-fail-fast` exit 0: 3120 passed, 0 failed, 4 ignored
  (`/tmp/claude-1000/p61/t10/logs/ws2.txt`).
* `REXX_CORPUS_GATE=1 ... --test corpus --test ir_recorded_oracle`: 29 passed and 1 ignored; 21
  passed.
* `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 1 --release -p rexx-exec --test concurrency_tests
  whole_groups`: exit 0, 15 passed, 4:08 wall. The seeded gate inside it ran 13720 runs in 126 s,
  with no red (`/tmp/claude-1000/p61/t10/gate-f1-final.txt`).
* Debug `sim_gate::`: 9 passed, 1792 runs in 12 s.
