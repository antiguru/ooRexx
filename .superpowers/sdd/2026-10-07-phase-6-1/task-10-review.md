# Task 10 review: the seeded gate

Range `0e9d6fe6e..7dc679e36` (ledger commit `920b7cd4f` ignored). Read-only review: diff, brief,
carry-ins, rulings, spec section 4, and focused checks outside the diff (`scheduler.rs` invariants,
`sim.rs` refusal texts, `command.rs` child spawn, the revert scripts under
`/tmp/claude-1000/p61/t10/bin/`, `git show 7eafa3b58`). Nothing built; no probe run. Paths are under
`rust/crates/rexx-exec/` unless they say otherwise. Line numbers are at `7dc679e36`.

### Spec Compliance

- ✅ Spec compliant on the brief's steps and the carry-ins: `SwitchMode::Sim`, the seed rule (FNV-1a
  then splitmix64), the 7-policy mix with k, `REXX_SIM_SEEDS`/`POLICY`/`ONLY`/`ORACLE_REFRESH`, the
  committed table, exempt table and oracle sets, the process-level deadline with a group kill, masking,
  the report file with replay lines, the self-test, the revert checks, the origin REPLY run, and the gate
  record under `## Task 10`. The scheduler test programs are left out and recorded as the brief says.
- ✅ Rulings R1-R6 and the TRACE key move are implemented as ruled: the R1 known set is read from
  `DIFFERING` (`failing_unswitched`) and unioned with the oracle sets' failing tests; the STREAM rows
  carry `clock=real`; children omit the `rexx-sim:` line; MutexSemaphore TEST_EXCLUSION runs as a
  `single` row with a design-limit exemption keyed on its refusal text; `message_notify.rex` is keyed on
  `a wait that nothing left to run can end`; the WALL_CLOCK rerun is limited to runs whose every red is
  a check in a WALL_CLOCK test, and every rerun goes to the report.
- ❌ One red the spec lists is not judged for runs that end early, see Important 1
  (`tests/concurrency_tests.rs:4134-4180`).
- ⚠️ Cannot verify from the diff: the reported counts (13720 runs in 126 s, 60/24/53 revert reds,
  128/128 self-test, 300/300 REPLY). The revert scripts match the described hunks.

### Strengths

- Exemptions match on group, part, test, kind and a line text. Each committed row is as narrow as its
  ruling: SysSleep is `*` only in part, MutexSemaphore is `single` + `TEST_EXCLUSION` + design-limit +
  refusal text, `message_notify.rex` is program + `-` + design-limit + refusal text. A different red
  kind or refusal in an exempted part (an invariant in MutexSemaphore single, a hang in
  `message_notify.rex`, any other SysSleep test) still fails the gate.
- The revert hunks (`mutate.py`) match the original defects. I1: the pre-`7eafa3b58` timer had a
  single `waiter` slot that a second wait overwrote, and the earlier waiter kept its sleeper entry. The
  revert drops all but the last waiter from the timer and leaves their sleepers too. N1 and M11 are the
  exact guards the brief names.
- The I1 explanation is correct. `scheduler.rs` does check "a parked activity with no wake source"
  (around `:1127`), but `wakes` counts the parked activity's sleeper entry. Every
  `ticker_waitTimer` waiter has a deadline (`20000` in the program, and the oracle signature
  requires one), so the dropped waiter keeps a wake source and only its output changes. A gate program
  with a `FAIL` line is the right route, and the gate reads it from stdout, not from status.
- R3's child detection is sound. `REXX_SIM_CHILD` is set only where `child_switch_mode()` returns
  `Some`, which happens only when the parent runs in sim mode (`sim.rs:1256-1260`). A top-level run
  started by the harness never gets it unless the invoking environment already has it. In that case
  the gate goes red as `no-report`, so the failure is loud, not silent.
- Pinning scratch paths per unit (`unit_dir`) instead of per PID is backed by a measured step-count
  difference (MethodArgs 49749 vs 49759). Without it, replay lines would not reproduce.

### Issues

#### Critical (Must Fix)

None.

#### Important (Should Fix)

1. **`tests/concurrency_tests.rs:4134-4180` (`judge`): a run that ends early on an unlisted
   refusal, or exits abnormally with no failing-test lines, passes the gate.**
   * After hang, crash and panic, `judge` reds a `rexx-exec:` line only when it starts with
     `the scheduler found ` or contains a `DETERMINISM` or `DESIGN_LIMIT` text.
   * Any other refusal falls through. Examples are a `... is not implemented` reached only under some
     interleaving, or `scheduler.rs:1992`/`:2031`-style messages that do not start with the invariant
     prefix. The next check reads the sim line, which is present. The last check reads `failing(run)`,
     which holds only `[failure]`/`[error]` detail lines. A driver killed mid-test prints neither, so
     the run is green.
   * The exit status is never judged. An rc 1 with a `pass` summary is the shape of the origin REPLY
     failure the gate exists for (`pass, assertions 18, rc 1`), and it is green too.
   * Live instance: Section1 whole refuses on every seed with
     `method "OBJECTNAME=" ... is not implemented` (its `DIFFERING` normal key). The close run lists
     it only under oracle differences, with no red. That refusal is legitimate there, but the
     mechanism accepts any refusal in any part, including parts with no `DIFFERING` row.
   * Why it matters: the spec's red list includes "a nonzero ooTest failure or error count". A run
     that dies inside a test has an errored test but no count to read. The gate silently reports a
     schedule-dependent abort as an oracle difference, and the ruling D7 chain never sees it.
   * Fix: add a red kind (`abort`, or reuse `check`). Fire it when a group part ends without the
     ooTest summary, or with a `rexx-exec:` refusal or exit status that neither the part's `DIFFERING`
     normal-mode key nor any committed oracle outcome has. Programs get the same rule against their
     oracle set's status. Exempt the known Section1 case through the R1 known-set mechanism, not a
     wildcard.

#### Minor (Nice to Have)

1. **`tests/support/group_runner.rs:535` and `src/command.rs:565`: sim children escape the
   deadline kill (implementer concern 1), and the doc comment says they do not.**
   * Every command a sim run starts is put in a group of its own: `child_switch_mode()` is `Some` for
     any command in sim mode, not only `rexx`. `kill_process_group` on the `rexx-run` group therefore
     never reaches them.
   * On the `Killed` path, a child still running is orphaned and keeps running after the gate ends.
     If it inherited the parent's stdout, `recv_timeout(10 s)` also discards the parent's whole
     output, which is a 10 s stall per such run.
   * It cannot hang the gate: the readers are bounded and a hang is already red. It is not a defect
     this task must fix.
   * The sentence "a child it started included" is false and must go (prose constraint).
   * Fix options, either one: have the harness start `rexx-run` with `setsid` and kill every process
     of that session (`/proc/*/stat` sid) at the deadline; or have `command.rs` keep the parent's
     group when `REXX_SIM_CHILD` is already set in the parent's own environment.
2. **`tests/concurrency_tests.rs:4323` with `run_unit`: a passing WALL_CLOCK rerun on a row without
   `clock=real` hides a determinism breach.**
   * Outside `clock=real`, the same seed and mode must reproduce the run. If the first run fails a
     WALL_CLOCK check and the rerun passes, the run was not deterministic, but the result is recorded
     only as "rerun passed".
   * The only expected source of variation is the wall-clock `block=` watchdog.
   * Fix: compare the two runs' `trace=` fields, and red as `determinism` when they differ on a row
     without `clock=real`. Alternatively, limit the rerun to `clock=real` rows, which are the only
     ones R6's evidence covers.
3. **`tests/concurrency_tests.rs:4288` (`unit_dir`): scratch paths now collide across concurrent
   gate runs and depend on the target directory.**
   * `CARGO_TARGET_TMPDIR` is `<target>/tmp` for every profile, and debug seeds are a prefix of
     release seeds. A debug and a release gate run at once on one target dir share unit directories,
     and each run's `remove` deletes the other's.
   * A replay from another target dir (the revert copies used `rv-target`) runs at a different path.
     For path-sensitive parts (MethodArgs) that gives a different step count, so the replay does not
     reproduce.
   * Fix: include the profile in the name, and print the target tmp dir in the replay line.
4. **Environment as a host input (risk 4).** Both `run_here` for programs and `crate_environment` pass
   the harness's whole environment through, `REXX_SIM_*` included. A replay therefore runs with
   `REXX_SIM_ONLY` set where the gate run had none. No current part enumerates the environment, so
   this changes nothing today. Filtering `REXX_SIM_*` (and `REXX_SIM_CHILD`) out of the child's
   environment closes it. No other host input was found: time is virtual outside `clock=real`, and
   scratch paths are pinned.
5. **`tests/support/group_runner.rs:591`: the stdin write happens before the deadline loop.** A program
   that neither reads standard input nor exits, given a sidecar stdin larger than the pipe buffer,
   blocks the harness past any deadline. Current sidecars are small. A writer thread fixes it.
6. **Gate programs N1 and M11 print lines nothing reads** (`slept`, `result ... after`). Their route is
   the invariant, which is correct under the injecting knobs. A comment saying the stdout is not
   judged would stop a reader from assuming it is.

### Assessment

**Task quality:** Needs fixes

**Reasoning:** The gate matches the brief, the carry-ins and every ruling. The exemptions are keyed as
narrowly as ruled, and an exempted part cannot hide a different red kind or refusal. The three reverts
undo the original defects and fail the gate through sound routes, and the I1 explanation is correct.
One hole remains in what the gate judges. A run that ends early on an unlisted refusal, or with an
abnormal status and no failing-test lines, passes green and appears only in the oracle report. That is
the shape of the origin failure the gate was built for. Fixing it is a small change to `judge`, plus a
known-set entry for Section1's normal-mode refusal. The process-group leak is real but bounded and
only on an already-red path. Its doc comment needs correcting.
