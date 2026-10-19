# Task 6.10 re-review: fix round 1

Scoped re-review of `4137247f5..847b16fbc` (final code `9a3431a0b`). I built HEAD `847b16fbc` and the task's parent
`d2e55befa`, each in its own scratch `git worktree` with its own target dir (`rr610-target`, `rr610-ptarget`),
`cargo build --release --bin rexx`, exit 0. Probes ran from fresh directories: the oracle under the standard
wrapper, the crate under `memcap 2G timeout -s KILL 10` (or 20). A process inside a delivery loop ignores SIGTERM
(round 0 review), so a bare `timeout` does not bound it. stdout, stderr and rc were compared separately. Probes were
in `scratchpad/rr610-probes.HuFP/`, which is not durable, so the deciding lines are quoted.

"All 13 modes" below means: wall, `sim:1`, `sim:7`, `sim:42`, `sim:3,uniform:0.2`, `sim:9,uniform:0.2`,
`sim:5,uniform:1`, `sim:8,uniform:0.01`, `sim:7,pct:3,k=40`, `sim:11,pct:3,k=200`, `sim:4,pct:2,k=1000`,
`sim:2,pre:2,k=40` and `sim:6,pre:1,k=200`.

No new macro is in the range. `ir/drive.rs` changes only the function `empty_continuation`.

## Checked and holding

* **Deviation 50 and its witness.**
  * `undelivered-trace-clause-runs-first` (in `tests/licensed_divergences.rs`) pins the crate's `c=1` against
    the oracle's `c=0`. `--test licensed_divergences`: exit 0, 22 passed. Deviation 50's row carries the marker.
  * **The clause's own effects are the only divergence.** I reran the round 0 shapes on HEAD and the oracle:
    `stale1`, `stale4`, `sxa`, `sxb`, `sxd`, `sxe`, `sxf`, `mid4`, `mid7`, plus a new `pos1`. Every one has
    identical rc and identical stderr, with paths masked. stdout differs only in the failing clause's own
    effect.
  * `pos1` sets `c` in the failing clause and `d` in the clause after it, and its handler prints
    `sigl`, `position`, the message, `trace()` and the destination. The oracle prints
    `syntax 97.1 sigl 3 pos 1457 c=0 d=0|msg Object "" does not understand message "LINEOUT".|trace now N dest Object|more 2`.
    HEAD prints the same except `c=1`. So the 97.1 itself, its position, and the later program state (`d`, which
    the next clause would have set) all match.
* **Important 2: the spinning destination and the deadlock.**
  * `st1` (an unguarded destination spinning `do while .local~flag = 0; end`) prints `done 1`, rc 0, in all 13
    modes.
  * `two1` (two threads, a guarded class counter) prints `lines A 30 B 30` in all 13 modes, so the deadlock stays
    gone.
  * `two2` (two threads, default route) gives the oracle's sorted stderr digest (`d63f345c`) in all 13 modes.
  * The two new seeded rows each give exit 0 and 70 of 70 `agrees with`:
    `corpus/lang/trace_destination_waits_on_thread.rex:program` and
    `corpus/lang/trace_destinations_two_threads.rex:program`. So does `trace_reply_continuation`, 70 of 70.
  * The two witnesses, 8 oracle runs each, give one digest each.
* **Important 3: the call path.** One callgrind run, 3 rounds, `-B 0.5`, exit 0, every spread 0.0000 except sayloop
  0.0012 and parse 0.0004:

  | program | parent | head | own delta (head vs parent) | head vs pin |
  |---|---|---|---|---|
  | sendloop | 14595035861 | 14595033392 | -0.0000% | +0.0693 |
  | dispatch | 22035821899 | 22035815257 | -0.0000% | -0.1355 |
  | fibcall | 8812465605 | 8812399145 | -0.0008% | -3.1341 |
  | intcall | 754651330 | 754348848 | -0.0401% | -10.1717 |
  | extcall | 7785078015 | 7785075626 | -0.0000% | -13.0932 |
  | nop / assign / emptyloop | | | 0.0000% each | |
  | rexxcps | 19369753026 | 19372369374 | +0.0135% | +0.4915 |

  Every own delta is within +0.05%, and every figure is under the 0.5% pin budget. rexxcps sits at +0.49%,
  which leaves little headroom for the next task.
* **The Minor.** The error report stays on stdout, and the oracle's fallback lines are on stdout too. On
  entry 51's program the oracle writes `6 *-* a = 1 ... routine end|not reached` to stdout with rc 139 and an
  empty stderr. HEAD writes the 97.1 report to stdout with rc 159 and an empty stderr. Reverting to stdout is
  right.
* **`oracle-crashes.txt` entry 51** matches the run exactly: the program, the stdout contents, the empty stderr
  and rc 139.
* **Rule (a), refusals inside a delivery.** The destination's `n`th LINEOUT sends `ITEMS` to a Stem built without
  its allocation. Each of these probes gives rc 120 with exactly one stderr line,
  `rexx-exec: method "ITEMS" of class "Stem" is not implemented (Phase 9)`, and no further output:
  * `rf1`: a routine's `<I<`, with the caller under `signal on syntax`. The trap does not take it, as is right
    for a refusal.
  * `rf2`: every line refuses, so a second refusal is met. The message still appears once.
  * `rf3`: a started method's `<I<` while main sleeps. Its output is identical to the control `rf3c`, where the
    started method meets the refusal directly.
  * `rf4`: a refusal inside a routine's INTERPRET.
  * `rf5`: the main body, mid-clause, under `signal on syntax`.
  * `rf6`: an empty REPLY rest's `>I>`.
  * `rf7`: `sortWith`'s comparator method's lines, below a native frame.

  I found no case where it surfaces twice or in another activity.
* **Rule (b) does not touch the untraced path.** The count sits behind `slice_deferred && routing_trace` in
  `serve_requests`, which is reached only on the clause-countdown path (`clause.rs:317`). The table above
  confirms it.
* **Tests.**
  * `REXX_CORPUS_GATE=1 --test corpus`: exit 0, `1112 of 1112 matching`.
  * `--test trace_delivery_schedule`: exit 0.
  * `--lib`: 1154 passed and 1 failed. The failure is `a_move_across_file_systems_copies_and_unlinks`, which
    is environmental: my scratch worktree's `target/tmp` and `/tmp` are on one tmpfs, `this test needs ... on
    different file systems`.

## Issues

### Important

1. **Rule (b) hangs a destination that spins with a call in its loop.** `delivery_streak` restarts whenever a
   different activation runs (`count_delivery_clause`: `_ => Some((running, 1))`). A wait loop that calls
   anything therefore never reaches 51 clauses in a row, and never yields. Two probes, each an `st1` variant:
   * `sp1`: an unguarded LINEOUT runs `do while \self~ready; end`, where `ready` is an unguarded method returning
     `.local~flag = 1`.
   * `sp3`: the loop is `do while .local~flag = 0; call nothing; end`, with `nothing: return`.

   Results:
   * The oracle prints `done 1`, rc 0, on 8 of 8 runs of each.
   * The parent prints `done 1` under wall, `sim:1`, `sim:3,uniform:0.2` and `sim:7,pct:3,k=40`.
   * HEAD is killed (rc 137) in all 13 modes for `sp1`, and in all four modes tried for `sp3`.

   The rule does not model the oracle. `instructionCount` is a member of each `RexxActivation`, and calling
   another activation does not reset the caller's count. The waiting LINEOUT activation in `sp1` and `sp3`
   passes 50 instructions within a few iterations on the oracle, and its native calls (`.local~flag`) can also
   release the kernel. Fix: count clauses per activation, for example a field on the activation or a small
   stack keyed by activation id, kept across calls into callees and restarted when the slice falls due. Then
   rerun `sp1`, `sp3`, `two1`, `st1` and `trace_delivery_schedule` under the modes above. A seeded-gate row for
   `sp1` would pin it.

### Minor

2. **Concern 10 (a guarded destination waiting on another thread) is a regression against the parent, not round 0
   behaviour only.** `gw1` is `st1` with the default (guarded) `LINEOUT`.
   * The oracle prints `done 1` on 8 of 8 runs.
   * The parent prints `done 1` under wall, `sim:1`, `sim:3,uniform:0.2` and `sim:7,pct:3,k=40`.
   * HEAD is killed (rc 137) in all 13 modes.

   An ordinary Rexx destination is guarded unless it says `unguarded`, so this is the default shape of a
   destination that spins. What hangs is the spin. Blocking waits in a guarded destination (`guard on when`,
   `syssleep`) terminate (round 0 review, `st2`/`st3`). The suppression exists because a pinned yield while
   another delivery waits for the same guard reaches the "pinned wait for a guard that only an activity pinned
   below it can end" refusal. Fixing it means lifting that pinning limitation, which is not this task's scope.
   **Queue it** with the pinned-guard-wait limitation, recorded as a regression of this task against
   `d2e55befa` with `gw1`, rather than as a parent defect.
3. **Deviation 50 names only 97.1.** `mid4`'s destination raises 40.1 on `>>>` lines, and the clause runs first by
   the same mechanism (HEAD stdout `said|syntax 40.1 ...`, oracle `syntax 40.1 ...`). The row's text says
   "raises the same 97.1". One phrase ("the condition the delivery raised, such as 97.1") would make the licence
   cover what it licenses.

## Assessment

**Verdict: CHANGES REQUESTED** (finding 1). Important 3, the Minor, entry 51, Deviation 50's pinning and rule (a)
all hold. Important 2's two witnessed shapes hold, but rule (b)'s count, which restarts per activation, hangs any
spinning destination whose loop makes a call. That is a hang the parent and the oracle do not have, and it is
fixable here. Findings 2 and 3 can be queued or folded in.
