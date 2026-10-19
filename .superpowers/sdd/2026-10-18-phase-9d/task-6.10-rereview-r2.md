# Task 6.10 re-review: fix round 2

Scoped re-review of `3fd3801a7..13ccea027`, code `24da8a29b`. I built these, each in its own scratch worktree and
target dir, `cargo build --release --bin rexx`, exit 0:
* HEAD `13ccea027`;
* the task's parent `d2e55befa`;
* for attribution, round 1's code `9a3431a0b` and round 0's `aa7e2a079`.

Probes ran from fresh directories: the oracle under the standard wrapper, the crate under
`memcap 2G timeout -s KILL 10` (or 40/60 where noted). stdout, stderr and rc were compared separately. The probes
were in `scratchpad/r2-probes.6AHK/`, which is not durable, so the deciding lines are quoted.

"All 13 modes" means: wall, `sim:1`, `sim:7`, `sim:42`, `sim:3,uniform:0.2`, `sim:9,uniform:0.2`, `sim:5,uniform:1`,
`sim:8,uniform:0.01`, `sim:7,pct:3,k=40`, `sim:11,pct:3,k=200`, `sim:4,pct:2,k=1000`, `sim:2,pre:2,k=40` and
`sim:6,pre:1,k=200`.

No macro is added or changed. The code diff is `activity.rs`, one line of `route.rs`, and `scheduler.rs`'s
`serve_requests`, `delivery_may_yield` and `count_delivery_clause`.

## Checked and holding

* **The re-review's finding 1 is closed.**
  * `sp1` (a waiting LINEOUT calling an unguarded `\self~ready`) prints `done 1`, rc 0, in all 13 modes.
  * `sp3` (the loop calls an internal label on each pass) prints `done 1`, rc 0, in all 13 modes.
  * Their witnesses each give one digest over 8 oracle runs: `trace_destination_waits_calling_a_method` and
    `trace_destination_waits_calling_a_label`.
  * Their seeded rows each give exit 0, 70 of 70 `agrees with`.
* **The earlier witnesses still hold.**
  * `st1` prints `done 1` in all 13 modes.
  * `two1` prints `lines A 30 B 30` in all 13 modes.
  * `two2` gives 366 stderr lines in all 13 modes.
  * `trace_reply_continuation` gives one digest in all 13 modes. It is `610ab418…`, the digest of the oracle's
    10 runs in the round 0 review.
  * `--test trace_delivery_schedule` (test_forwarded3 under 60 modes): exit 0.
  * The rows `trace_destination_waits_on_thread`, `trace_destinations_two_threads` and `trace_reply_continuation`:
    70 of 70 each.
* **Alternating activations.** `alt1` is a LINEOUT that loops on `self~a`, where `a` returns `self~b`, so each
  activation runs 1 or 2 clauses per pass. The oracle prints `done 1` on 8 of 8 runs, and HEAD prints `done 1` in
  all 13 modes. LINEOUT's own count survives its calls, as intended. Without a common caller, alternation needs
  recursion (below).
* **Deep recursion, REPLY and START inside a destination.** In `deep1`, each LINEOUT recurses 300 deep through a
  method, starts an activity and waits for its result, and sends a REPLY whose rest runs 60 clauses. The oracle
  prints `lines 21 max 300` on 8 of 8 runs. HEAD prints the same in all 13 modes.
  * `delivery_counts` grows to one entry per activation that runs a clause during the delivery.
  * It is cleared at each delivery's start and when a slice falls due, and it lives on the activity, so a
    started or continued activity has its own.
  * `count_delivery_clause`'s `rposition` finds the running activation at the end in the normal case.
* **No behaviour change for untraced concurrent programs.**
  * Every `program` row of `corpus/sim-gate.tsv`: exit 0, `REXX_SIM_ONLY=<row>:program`, one invocation per row.
    That is 141 rows, counted by `awk -F'\t' '$2=="program"'`.
  * I did not run the `whole` and `derived` rows.
  * The count runs only behind `slice_deferred && routing_trace`.
* **Untraced perf of the count itself.** In one callgrind run (3 rounds, every spread at most 0.0001%), HEAD against
  the parent:
  * `nop` -0.0000%, `assign` -0.0000%, `sendloop` +0.0000%, `fibcall` -0.0008%, `dispatch` -0.0000%,
    `emptyloop` -0.0000%, `rexxcps` +0.0135%.
  * Against the pin, rexxcps is +0.4916%.
  * Round 2 itself is neutral: HEAD against `9a3431a0b` is +0.0004% on `pingguard`, `pingmsg` and `pingsem`.
* **Tests.**
  * Corpus gate: `1114 of 1114 matching`.
  * `licensed_divergences`: 22 passed.
  * `--lib`: 1154 passed, 1 failed. The failure is environmental: `a_move_across_file_systems_copies_and_unlinks`
    needs two filesystems, and my scratch worktree's `target/tmp` shares the tmpfs with `/tmp`.

## Issues

### Important

1. **Untraced concurrent programs pay for round 1, and `pingguard` is over the pin budget.** These are the
   `pingpong/` programs of `callgrind.sh`, with no TRACE in them. Same run, 3 rounds, spreads at most 0.0001%:

   | program | parent | HEAD | own delta | HEAD vs pin |
   |---|---|---|---|---|
   | pingguard | 1317816143 | 1322918408 | +0.3872% | **+0.6299% OVER** |
   | pingmsg | 7518256884 | 7545324001 | +0.3600% | -0.4940% |
   | pingsem | 1301398739 | 1305099257 | +0.2843% | +0.4242% |

   `callgrind.sh` exited 1 (`OVER pingguard head +0.6299% > 0.5%`).

   Bisection (`-r 1`), round 0 then round 1 then HEAD, against the parent:
   * pingguard +0.1746%, +0.3863%, +0.3867%;
   * pingmsg +0.0032%, +0.3588%, +0.3598%;
   * pingsem +0.0693%, +0.2839%, +0.2844%.

   So most of it arrived in round 1 (`9a3431a0b`), and round 2 adds nothing. `cgdiff.py` of round 0 against round
   1 puts it in `Interp::switch_to`'s self cost: +3.0M instructions on pingguard and on pingsem, and +2.4M on
   pingmsg, which also adds `run_round` +0.96M and `Activity::new` +0.32M.

   That fits the `Activity` fields round 1 added (`delivery_floor`, `delivery_streak` (now `delivery_counts`) and
   `trace_refusal`), moved with the struct on every switch. This is a hypothesis; I did not confirm it with a
   mutant. If it holds, moving the three behind one boxed cold field (or into the existing cold state) should
   return the switch path to the parent's cost.

   No round's perf table measured the `ping*` programs. The round 0 report and both re-reviews ran
   call-path and per-clause programs only, so this was not seen.

### Minor

2. **Recursion to the control-stack limit inside a destination is OOM-killed, since round 0.** `rec3` is a
   LINEOUT that runs `call spin`, where `spin` recurses with no end. HEAD, round 1 and round 0 are each killed at
   the 2G cap after about 8 s (`memcap: OOM-killed at the 2G cap (peak 2.0G)`). The parent prints `after`, rc 0.
   The oracle segfaults (rc 139).
   * Bounded depths show the cliff. `dn1000` and `dn5000` pass on both HEAD and the parent. `dn20000` passes on
     the parent (`after`, 77 MB peak) and is OOM-killed on HEAD. The crate's control-stack limit is about 9998
     (`depth.rex`: `max 9998`).
   * The same recursion done only on the first LINEOUT terminates. Trapped, it gives `trapped 11.1 calls 3`.
     Untrapped, it ends at rc 245 with a long report.
   * So the blow-up appears when every delivery hits 11.1. A likely cause, not confirmed, is the
     about-10000-line error report being delivered line by line through the same destination, each line
     recursing to the limit again.
   * `rec1`/`rec2` (a recursing destination waiting on a worker) behave the same way: OOM-killed at about 20 s,
     where the parent prints `done 1`. The oracle segfaults on these too, so the oracle does not yield in this
     shape either.

   This is pathological, and the oracle crashes on it. But the crate should terminate, and the parent did.
   Queue it, together with an `oracle-crashes.txt` candidate for `rec3`.

## Assessment

**Verdict: CHANGES REQUESTED** (finding 1). The re-review's finding 1 is closed, the witnesses and schedules hold,
the count is free on the per-clause and call paths, and round 2 is perf-neutral. But untraced concurrent programs
have carried a +0.28% to +0.39% cost since round 1, and `pingguard` is over the 0.5% pin budget. Finding 2 can be
queued.
