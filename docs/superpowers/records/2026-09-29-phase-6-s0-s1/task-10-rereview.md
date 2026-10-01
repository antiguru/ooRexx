# Task 10 re-review (0f5aef01c..1f9be8ea5)

Reviewer: p6-t10-review. Read-only; gates not re-run. Builds: `git archive` of 1f9be8ea5 and
92ac5c054, touched, own target dirs (deleted), both logs carry `Compiling rexx-exec`.

## Findings

- **I1 / P20: Addressed.** `scheduler.rs` trait `Scheduler` declares only `spawn(&mut self) -> bool`,
  called by `exec_suspends` in `ir/drive.rs`. `ActivityId`, `run_until_park`, `park`, `unpark`,
  `yield_at_slice`, `exit_for_native`, `exit_for_block`, `post_completion`, `request_baton`,
  `stop_the_world` are gone (grep over `rust/crates` finds none). No `expect`/`allow` on the trait.
  The report lists every left-out method. The three `cfg_attr(not(test), expect(dead_code))` stay,
  as ruled. Note (not a finding): `SingleActivity::spawn` answers `false` always, which is its
  single-activity meaning and is what the Split path branches on.
- **I2 / M1: Addressed.** The "no bench program has a plain DO" sentences are deleted from
  `phase-6-perf.md`; the `split_continuation` doc sentence is deleted (function replaced by
  `exec_suspends`, whose doc states the actual behaviour).
- **I3 / P21 option b: Addressed.**
  - Park loud on every path: `exec_suspends` answers `op_not_driven("an activity park")`; the
    `$top`/pinned distinction is gone, so top-level and nested (`do label l`) both refuse.
  - Cold and out of line: the `Op::Exec` arm handles `Done` inline and sends everything else to
    `#[cold] #[inline(never)] exec_suspends`.
  - `Deliver::Wake`, `Exit::Suspended`, the `woken` epilogue match and the `at: match deliver`
    are gone; `grep` for `Wake|Suspended|woken` finds none.
  - Split goes on with its carried flow (`Ok(RegionEnd::Flowed(flow))` when `spawn()` is false).
  - `a_parked_exec_is_loud` asserts rc != 0, stderr contains "an activity park", stdout is only
    `before`, splits 0, for a driven region and for `do label l`; each program also runs clean
    unscripted. `a_split_exec_goes_on_with_its_flow` kept.
  - Perf, committed state (a7e53d4c0, `-r 3`, spreads 0.0000%): per-clause programs nop, assign,
    compound, emptyloop, varlookup are +0.00 vs 92ac5c054; largest rise arith +0.24; fibcall and
    fibfunc -2.32; rexxcps -0.96. The numbers in the report equal those in `phase-6-perf.md`. The
    doc-only commit 1f9be8ea5 cannot change them.

## Requested checks

1. **No production path can produce `ExecOutcome::Park`.** The only constructors are
   `run.rs:574-579`, inside `#[cfg(test)] if let Some(scripted) = take_scripted()`. The
   production match arms for GUARD and REPLY answer `Done` (via `exec_guard`/`exec_reply`). The
   `dead_code` expects on `Park`/`Split`/`Guard` are consistent with that.
2. **REPLY and GUARD unchanged.** Probes run from a fresh dir against the oracle, 1f9be8ea5 and
   92ac5c054: `reply` (REPLY then RETURN value: error 98.936), `reply2` (valued REPLY with a
   background loop), `guard` (unguarded method, GUARD OFF/ON WHEN), `guard2`. 1f9be8ea5 is
   byte-identical to 92ac5c054 on stdout, stderr and rc in all four. Both match the oracle except
   `reply2`, where the oracle interleaves `bg` lines before `done` and both Rust builds print
   `done` first (pre-existing on 92ac5c054, not from this change).
3. **False or future-mechanism sentences.** One class, in the report (`task-10-report.md`):
   - Seam section: "each landing with its first caller" (left-out methods) and "The driver's Park
     path lands with S2's first real parker" describe a future mechanism. Delete both clauses.
   Nothing else found: `exec_suspends` doc, `ExecOutcome::Park` doc ("would park ... refuses
   loudly"), lib.rs seam comment, `scheduler.rs` module doc and the perf-doc fix-round section
   match the code. The report's "largest rises were x7 +2.65, v5 +2.12, x1 +0.51" matches the
   variant table.

## Verdict

Approved, with one report-prose deletion (item 3). No code findings.
