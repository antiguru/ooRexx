### Task 4b-M: Build a control that works, then name the driver's cost -- BLOCKS EVERYTHING

**Decided 2026-08-09 by Moritz, on both halves: fix the instrument first, then name the mechanism, before any further promotion.**

Task 4b's op-level driver measures about **+7.0% on `emptyloop`** and **+2.7% on `varlookup`** against the parent binary, with **no named mechanism**. Only `branchloop` pays for its promotion. Exit criterion 4 is that the IR arm is not slower than the tree-walker arm, so as things stand this phase fails its own performance criterion, and every task after 4b builds on the driver that costs it.

**The instrument comes first, because the mechanism cannot be attributed to a number that cannot be measured.**
Task 4b's negative control read +4.3%, +2.5% and +1.8% where it should read within half a per cent, and the tree-walker arm moved by up to 6% between binaries running identical code. A 7% signal sitting barely above a 6% layout artifact is not a measurement, and Task 4a's within-binary estimator is already dead for the same reason.

**Why the control failed, stated as a hypothesis to test rather than a conclusion.** It reverted the *promotion* but not the *driver*, so it never controlled for the thing being measured. A within-binary control can only read zero if the arm being held fixed is genuinely unchanged -- and if the engine choice touches the tree-walker's own hot path, it is not. Whether that is what happened is the first thing to establish.

**Established in Task 4b's fix round 1: the second half of that hypothesis is confirmed, and it is stronger than "the control was reverted wrongly".** The tree-walker arm does not run unchanged code in any of the four binaries, because three of Task 4b's changes are on the path *both* arms take: `absorb` is a call taking a 64-byte `Flow` by value and returning one, once per clause, and `run_bounded_instructions` decides through it too; `step_in_temps_frame_with` now routes through the generic `in_stepped_clause` with a closure; and `if_targets` computes `skip_else` on both of `If`'s paths where the old arm computed it only when the condition held. So **no within-binary control over these binaries could have read zero**, whatever it reverted -- which means 4b-M's first deliverable is not a better revert but a control whose two arms genuinely share no changed code, or a second instrument that does not need one. A profile taken alongside that round corroborates the same thing from the other side: `_memcpy_avx512_unaligned_erms` triples on an identical stack through `run_repeating` in both arms.

**A second instrument is available and has different failure modes.** Instruction and cycle counts are insensitive to the code layout and frequency effects breaking the wall-clock control. They are a cross-check, not a replacement: a cycle ratio equals a wall-clock ratio only if both sides run at the same average frequency.

**This task ends with two things, and neither is a promotion:** a control that demonstrably reads zero when nothing changed, and a named, evidenced cause for the driver's cost. If the cause turns out to be inherent to the two-level op-driver shape, say so -- that is a finding about the design, and the design came from a spike that measured no speedup either.

## Global Constraints

* **The C++ tree at `/home/moritz/dev/repos/ooRexx/` is the oracle and is read-only.** Never modify `interpreter/`, `samples/`, `build/`, `ootest/`.
* **Wrap every oracle invocation:** `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx /abs/path/FILE )`. Use `ulimit -v 8388608` for benchmark workloads.
* **Three programs crash the oracle; never run them:** `select; when 1 = 0 then; when 2 = 2 then nop; end`; `say date('M','0','D')`; any `NUMERIC DIGITS` above 1000.
* **Run every oracle probe from a fresh empty subdirectory you `mkdir` yourself, with absolute paths.** The scratchpad root is on the oracle's external-routine search path and a leftover `.rex` file gets called as an external routine.
* **A symbol named `x` or `b` followed by a quoted string parses as a hex or binary literal.** Use `n1`, `cv`, `zz` in probes.
* **Use `/bin/grep -a`, never bare `grep`** -- the wrapper is ugrep with `-I` and silently skips binary files.
* **Read stdout, stderr and exit status as separate descriptors. Never `2>&1`.** Never read a cargo exit code from a pipeline.
* **Never run cargo from the repo root.** Run from `rust/`.
* **`cargo fmt --all --check`**, not `cargo fmt --edition 2024 --check`, which is an error rather than a check.
* **`cargo clippy --workspace --all-targets -- -D warnings`.** A warm target directory makes a green provisional; run from a clean target at the phase gate.
* **`cargo test <name>` exits 0 when it matches nothing.** Read the run count.
* **`cargo test --release` is a distinct gate** -- `lto = "fat"` changes behaviour, which `5253a674` recorded.
* **Mutation runs need `--no-fail-fast`**, or the suite stops at the first catcher and "nothing else caught it" is unmeasured.
* **No `unsafe`.** The workspace sets `unsafe_code = "forbid"`.
* **No em-dashes in comments; use `--`.** No counts of mutable in-repo aggregates in prose.
* **Markdown: one sentence per line, `*` bullets, first-word-only capitalisation including after a colon.**
* **Back up with `cp`, restore from the backup, verify with `sha256sum -c`.** Never `git checkout --`, never `git add -A`, never `git reset --hard`, never force-push.
* **Benchmark comparisons interleave between arms within one sitting.** Never read a comparison across two separate runs.
* **Commit first, then read the hash back with `git log`, then quote it.**


---

## What is already established, so you do not re-derive it

Two independent investigations converged on the mechanism after this task was written: the
controller profiled with samply and analysed with pollard, and Task 4b's reviewer read the diff.
**The mechanism is named. Your job is to confirm it with a trustworthy instrument and remove it,
not to search for it.**

Profile, `emptyloop` at `n = 25000000`, release with symbols, one run per arm, wall 3013 ms
tree-walker against 3097 ms IR:

| function | tree-walker self | IR self |
|---|---:|---:|
| `run_bounded_instructions` | 49 ms | gone |
| `run_ops` | -- | **96 ms** |
| `step_in_temps_frame_with` | 617 ms | **674 ms** |
| its closure | 151 ms | 111 ms |
| `memcpy` in `run_repeating` | 17 ms | **53 ms** |

**About 84% of the regression is on the dispatch path itself.** The `memcpy` is on an *identical*
stack in both arms -- same code, three times the time -- which is what a larger value copied per
clause looks like.

Named candidates, sharpest first:

1. **`run_bounded_from_chunk`'s `op_at(start)` and `run_ops`' `op_at(end)` add two bounds-checked
   `op_of` loads and an extra call frame per `DO`-body pass.** On a 25e6-pass axis that is 50e6
   loads and 25e6 frames. At `736bf080` the chunk arm looked up `op_of` once per clause and never
   looked up `end` at all.
2. `absorb` moves a 64-byte value across a call boundary per clause -- `size_of::<Flow>() ==
   size_of::<Absorbed>() == 64` -- where the range test used to be an inline match on a local.
3. `step_in_temps_frame_with` routes through the generic `in_stepped_clause` with a per-clause
   closure that now also captures `engine`.
4. `BodyEngine` grew 8 to 16 bytes and is copied on every `step`/`run_loop`/`run_bounded` hop.
5. `run_ops` takes eight arguments where the old loop had five values in scope.
6. `granting.grants()` is a new branch per clause.
7. Tree-walker only, per `IF`: `if_targets` computes `skip_else` on both paths where the old arm
   computed it only when the condition held.

## Why no within-binary control could read zero, which is your first deliverable

**Candidates 2, 3 and 7 sit on the path *both* arms take.** So a control that reverts the promotion
does not hold the reference fixed -- the tree-walker arm changed too. That is why Task 4a's
estimator died and Task 4b's control read +4.3%, +2.5% and +1.8% where it should read within half a
per cent. It is not a flawed revert; **no revert over these binaries could have worked.**

Your first deliverable is a control whose arms share no changed code, **or an instrument that needs
no control**. The second is likely cheaper and is the recommended route: **instruction counts are
insensitive to the code layout and frequency effects breaking wall-clock here**, and the machine's
governor could not be fixed, which is a property of the machine rather than an omission. `perf stat
-e instructions` and `valgrind --tool=callgrind` are both available routes.

**State the instrument's own limit when you use it.** A cycle or instruction ratio equals a
wall-clock ratio only if both sides run at the same average frequency and the same instructions
retire at the same rate. Instruction counts are a proxy chosen because it is trustworthy at this
magnitude, not because it is the quantity the gate cares about.

## What ends this task

1. **An instrument that demonstrably reads zero when nothing changed.** Show it, do not assert it.
2. **Each named candidate confirmed or refuted with a number**, not an argument.
3. **The confirmed ones removed**, with the removal measured on the same instrument.
4. **A wall-clock re-measurement at the end**, against `phase-4e-anchor.md`, so the phase knows
   where it stands on the quantity exit criterion 4 actually names.

**This task promotes nothing.** If removing a cause requires changing what a promotion emits, say so
and stop rather than promoting while measuring.

**If a cause turns out to be inherent to the two-level op-driver shape, that is a finding about the
design and you should say it plainly.** The shape came from a spike that measured no speedup either,
and the phase would rather learn that now than after four more promotions.
