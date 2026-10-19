# Task 6.10 re-review: fix round 3

Scoped re-review of `d151615d0..492766170`, code `414fa3bf4`. Before building, `df -h /tmp` showed 11G free. I built
these in scratch worktrees, each with its own target dir, `cargo build --release --bin rexx`, exit 0:
* HEAD `492766170`;
* the task's parent `d2e55befa`;
* round 2's `13ccea027`;
* for attribution, round 0's `aa7e2a079` and round 1's `9a3431a0b`.

Every build and worktree was removed afterwards. Probes ran from fresh directories: the oracle under the standard
wrapper, the crate under `memcap 2G timeout -s KILL 10` (or 20). stdout, stderr and rc were compared separately.
The probes were in `scratchpad/r3-probes.agGm/`, which is not durable, so the deciding lines are quoted.

"All 13 modes" means: wall, `sim:1`, `sim:7`, `sim:42`, `sim:3,uniform:0.2`, `sim:9,uniform:0.2`, `sim:5,uniform:1`,
`sim:8,uniform:0.01`, `sim:7,pct:3,k=40`, `sim:11,pct:3,k=200`, `sim:4,pct:2,k=1000`, `sim:2,pre:2,k=40` and
`sim:6,pre:1,k=200`.

## Checked and holding

* **The round 2 Important is closed.** I ran callgrind on the three programs alone (3 rounds, exit 0, spread at
  most 0.0050%):

  | program | parent | HEAD | own delta | HEAD vs pin |
  |---|---|---|---|---|
  | pingguard | 1317813078 | 1317926878 | +0.0086% | +0.2502% |
  | pingmsg | 7518263190 | 7517545462 | -0.0095% | -0.8604% |
  | pingsem | 1301398303 | 1301699456 | +0.0231% | +0.1624% |

  In the wider run (same binaries; it exited 1 only on the `SPREAD` flags, which pin and parent got as well
  while my probes ran alongside), the other programs against the parent were:
  * `nop` -0.0000%, `assign` -0.0000%, `sendloop` -0.0000%, `fibcall` -0.0003%, `dispatch` -0.0000%,
    `sayloop` -0.0009%, `rexxcps` +0.0212%.
  * Against the pin, `rexxcps` is +0.4993%. That is under the budget, with almost no room left.
* **The pinned-delivery claim holds on every shape I could build.**
  * The push and the pop of `Interp::deliveries` sit in one frame of `deliver_trace_line_as`, around a
    `pinned!` block that cannot return early.
  * Another activity runs only inside that pinned frame, so the Vec mirrors the Rust stack.
  * `routing_trace` stays per-activity, so an activity that has finished its own delivery inside another's
    yield reads `routing_trace == false` and never consults the outer `Delivery`.
  * Probes, each against 8 oracle runs:

  | probe | shape | oracle (8 of 8) | HEAD |
  |---|---|---|---|
  | `m1` | two started activities each trace 20 passes into one unguarded destination; each LINEOUT sends a REPLY, starts a message and runs 60 clauses | `lines 246 results A B` | same, all 13 modes |
  | `m3` | the destination's LINEOUT raises SYNTAX while a started worker runs; the traced caller traps it | `trapped 97.1 B result B lines The NIL object` | same, all 13 modes |
  | `m4` | `trace i` in main and in a started activity; the destination runs 80 clauses per line, so deliveries yield to each other; each `>V>`/`>=>` object's `VARIABLE` name is checked against its own line | `bad 0` | `bad 0`, all 13 modes (the parent: `bad 6` on 6 modes) |
  | `m7` | a trapped failed delivery, then a spinning destination waiting on a worker | `... done 1` | same, all 13 modes |
  | `deep1` | recursion 300 deep, START and REPLY inside a destination | `lines 21 max 300` | same, all 13 modes |
  | `alt1` | alternating callees | `done 1` | same, all 13 modes |

  `m7` is the leak check. A delivery that failed leaves no state that stops the next one yielding.
* **HALT during a delivery.**
  * `m6b` halts a started message whose destination is mid-loop. The oracle segfaults, 8 of 8 (rc 139). HEAD
    raises HALT in that activity, `result error 4.1`, and main goes on tracing (`main lines 1`), rc 0.
  * With a shorter loop (`m6`), the oracle's message finishes before the halt (`result ran`, 8 of 8). HEAD gives
    `result ran` on 8 of 8 wall runs. In the sim modes the halt lands mid-delivery on 8 of 13 modes, because
    virtual time differs. That is a legal schedule, not a defect.
  * SIGINT during a delivery's loop (`m5b`) is ignored by the oracle and by HEAD alike (both killed at
    `-k`). It is pre-existing, and the SIGTERM case is already queued.
* **Witnesses under all 13 modes.** One digest each, equal to the oracle's:
  * `trace_destination_waits_calling_a_method`, `trace_destination_waits_calling_a_label` and
    `trace_destination_waits_on_thread`: `8057c6dd…`;
  * `trace_destinations_two_threads`: `c7bba77b…`;
  * `trace_reply_continuation`: `610ab418…`.

  `sp1`, `sp3`, `st1` and `two1` likewise.
* **Tests.**
  * `--test trace_delivery_schedule` (test_forwarded3 under 60 modes): ok.
  * `--test licensed_divergences`: 22 passed.
  * `--lib`: 1154 passed and 1 failed. The failure is environmental, as in rounds 1 and 2:
    `a_move_across_file_systems_copies_and_unlinks` needs two filesystems, and the scratch worktree has one.
* **Seeded gate.**
  * Every `program` row whose name contains `trace`, `reply` or `guard` exits 0 (49 rows, counted by
    `awk -F'\t' '$2=="program" && ($1 ~ /trace|reply|guard/)'`).
  * `base/keyword/REPLY.testGroup` whole and derived: exit 0, no reds.
  * `base/keyword/TRACE.testGroup` whole and derived: exit 0, no reds.

## Issues

### Important

None.

### Minor

1. **Two delivering activities that share a guarded destination which blocks now end with a refusal.** In `m2`, two
   started activities trace two lines each into one destination whose LINEOUT is guarded (the default) and calls
   `syssleep 0.01`.
   * The oracle prints `lines 10 results A B`, 8 of 8.
   * HEAD, round 2, round 1 and round 0 all end with rc 120 under all 13 modes (wall, `sim:1` and
     `sim:3,uniform:0.2` for rounds 0 and 1):
     `rexx-exec: a pinned wait for a guard that only an activity pinned below it can end is not implemented`.
   * The parent printed `lines 5`, rc 0, with activity B's five lines written to stdout. It too failed B's
     deliveries; the difference is that it swallowed the failure. So this is the pre-existing pinned-guard-wait
     limitation, made loud by round 1's "a refusal is never dropped", not a regression of round 3.
   * The shape is ordinary: a default-guarded logging destination, two threads, and any blocking call inside
     LINEOUT. It should be appended to
     `.superpowers/sdd/queued/2026-10-19-guarded-trace-destination-spin.md`, which has the same root.
2. **The oracle segfaults when HALT lands in a delivery** (`m6b`, rc 139 on 8 of 8). This is an
   `oracle-crashes.txt` candidate. HEAD's answer (HALT raised in the halted activity) needs no change.

## Assessment

**Verdict: APPROVED.** The untraced concurrent cost is back within +0.05% of the parent, and `pingguard` is at +0.25%
against the pin. Moving the delivery state to `Interp` survived every concurrent delivery shape I built across all
13 modes, with no state carried into the next delivery or activity. Both Minors are queue entries.
