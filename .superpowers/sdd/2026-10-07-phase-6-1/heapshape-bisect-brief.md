# Heapshape running-total bisect

Phase 6.1's perf budget is +0.5% against the 6.1 base `e6af1198b`. The Task 9 review measured
heapshape at +1.0266% already at `6358ca7a6` (end of Task 8), before Task 9's +0.128%. No earlier
gate measured heapshape. Find which commit(s) carried the +1.03% and why. Measurement only: no
edits to the repository except the report.

Read first: `.superpowers/sdd/2026-10-07-phase-6-1/global-constraints.md` (memcap on every run with
`timeout` inside; no `bash -c`; `rm` only literal absolute paths, no globs, no variables), and
check 5 of `.superpowers/sdd/2026-10-07-phase-6-1/task-9-review.md` (the callgrind command).

Scratch: `/tmp/claude-1000/p61/hsb/`. Check `df -h /tmp` first. Build each commit from
`git archive <sha> rust interpreter | tar -x -C /tmp/claude-1000/p61/hsb/src-<short>` (fresh
directory per commit), `CARGO_INCREMENTAL=0 memcap 8G cargo build --release -j 4 -p rexx-run`
with one target directory per commit or a shared one: in either case touch the tree after
extracting (archived mtimes fool cargo) and require a `Compiling rexx-exec` line in the build log,
then copy the binary to `/tmp/claude-1000/p61/hsb/bin/<short>/rexx-run` and record its sha256.

1. Checkpoints, in order (task closes from the ledger): `e6af1198b` (base61; its binary already
   exists at `/tmp/claude-1000/p61/t1/bin/base/rexx-run`, verify its sha256 against the gate
   record's `## Task 1` and reuse it), `b4d847074` (T1), `7b84c819d` (T2), `ec97e4190` (T3),
   `66d9eac18` (T4), `5e59d66b3` (T4a), `37d874a36` (T5), `1fd487d02` (T5a), `072e536ac` (T6),
   `b71606f16` (T7), `6358ca7a6` (T8; binary at `/tmp/claude-1000/p61/t9/perf/bin/base/rexx-run`).
   Run `rust/bench-programs/callgrind.sh` on heapshape alone with all checkpoint binaries in one
   invocation (`-p heapshape`, `-r 2`), under `memcap 8G`. Table: Ir per checkpoint and the step
   against the previous one and against base61.
2. For each step over +0.2%, bisect its commits (`git log --oneline A..B -- rust`) the same way to
   the single commit. Then attribute with `callgrind_annotate` (or `rust/bench-programs/cgdiff.py`)
   on that commit's before/after outputs: which functions gained Ir, and whether the calls rose
   (added work) or the per-call cost rose (codegen). For added work, name the source line that does
   it and whether it is the R10 byte-aware collection trigger charge (Task 5a), the R9 truth
   judgment (Task 4a) or something else.
3. Also run the full eight programs (`pingmsg pingguard pingsem alloc alloc4c heapshape rexxcps
   emptyloop`) on base61 against the T8 binary once, to confirm the review's Tasks 1-8 column.

Write `.superpowers/sdd/2026-10-07-phase-6-1/heapshape-bisect.md`: the exact commands, the
checkpoint table, the bisect steps, the attribution with figures, and a one-paragraph conclusion
(which commit, added work or codegen, and the smallest change that would recover it if added work).
Do not commit (another agent commits in the same tree); the controller commits it.
Return only: the commit(s) found, Ir delta each, added-work vs codegen, and
the report path.
