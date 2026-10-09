### Finding Verdicts
- **I1 silent replay divergence** — ADDRESSED. `sim.rs` `Replaying` (`point`, `pick`, `finish`, `diverge`) refuses out-of-range picks, kind mismatches, passed points, decisions left at the end, and decisions needed after the file ends (`pick` returns via `diverge` on `get(next)` None); `sim_finish` raises it, `sim_is_stuck` stops the wait. Header `CONFIG hash=H` is written in `sim_finish` and checked in `read_replay` before the run. Tests named in the report cover the reviewer's cases with exact stderr. Read, not re-run.
- **I2 quadratic invariant check** — ADDRESSED. `scheduler.rs` `check_invariants` builds `parked`/`ready`/`free` vectors by handle in one pass; `guards.rs` `inconsistency` uses one set. Messages and check order preserved by reading (park-reason set equals the old `holds_park_reason`; `guards.waiters()` = keys of `waiting`). Nine corruptions are permanent variants. Scaling n=1000 3.07s to 0.13s, n=2000 20.7s to 0.35s; still superlinear in n (per switch linear in handles), acceptable.
- **I3 record only** — ADDRESSED. Gate record has the base61 column and running totals for all eight programs; heapshape +1.1558% is stated and left to the bisect and ruling.
- **M1 repeated refusal** — ADDRESSED. `sim_check_switch` returns once `invariant_broken`; test asserts exact stderr.
- **M2 trace-file edges** — ADDRESSED. `nonempty_path`, second `trace=`/`replay=` refused, `comma_in_path` names the path, unwritable trace is `Loud::sim_trace_unwritten` (rc 120). Limit: `trace=/a,fifo` parses as path plus knob, unavoidable.
- **M3 dead `pub fn trace_hash`** — ADDRESSED. Private, used by `read_replay`; `lib.rs` re-export removed.
- **M4 exploration test** — ADDRESSED. `sim/tests.rs` asserts the set equals all three interleavings over seeds 1-40.

### Concern 1 ruling
Acceptable, not worth blocking. Reaching it needs a deliberate edit plus a recomputed hash: accidental truncation or damage is caught by the header hash, and a different program almost always hits a pick or point mismatch or leaves decisions unconsumed. The residual is a replay that runs past the file's last decision with no further preemptions or collections. Closing it is cheap if wanted: add the final contended-step and allocation counts to the header (inside the hashed or checked line) and compare in `finish`. Optional Minor.

### New Breakage in the Fix Diff
None.

### Out-of-Scope Observations
* `sim_finish` runs `finish()` even when the run ended for another reason (halt, other refusal), so a replay cut short by an unrelated refusal may add a second "decisions left" line. Harmless, not exercised.

### Verdict
**Fix round:** All findings addressed, no new Critical/Important breakage.
