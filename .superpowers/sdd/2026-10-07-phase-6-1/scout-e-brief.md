# Scout E: deferred items for Task 11a

Moritz (2026-10-09) asked which deferred items still belong in Phase 6.1. This scout re-probes each
candidate at HEAD and sizes it, so the controller can write Task 11a. Read-only: you edit nothing in
the repository except your report file, and you do not commit.

Read `.superpowers/sdd/2026-10-07-phase-6-1/global-constraints.md` first. Its shell, memory and
oracle-run rules bind you:
* every interpreter run goes under `memcap` with `timeout` inside it;
* no `bash -c`;
* `rm` only with literal absolute paths;
* oracle runs use the exact command given there, from a fresh empty directory;
* `rust/corpus/oracle-crashes.txt` is read first;
* concurrent programs get at least 5 runs per engine, with counts stated.

## Binary

Archive the commit with `git archive f7170918c rust interpreter` into your scratch `src/`. Touch the
tree, then build with `CARGO_INCREMENTAL=0 memcap 8G cargo build --release -j 4 -p rexx-exec
--bin rexx-run`, using its own target dir, and require a `Compiling rexx-exec` line. Record the
binary's sha256. Another agent is editing `rust/crates/rexx-core/src/heap.rs` in the shared tree, so
never build from the shared tree. `/tmp` is inode-tight: check `df -i /tmp` and keep to one source
copy and one target dir.

## Items

Each queued item is `.superpowers/sdd/queued/<name>.md`. A "ledger" item is a line in
`.superpowers/sdd/2026-10-07-phase-6-1/progress.md` that starts `- Task 12 queue add:` (grep the
quoted words).

## For every item

1. Re-run the item's own probe, or write the smallest one from its description, on the crate binary
   and on the oracle.
2. Verdict: STILL REPRODUCES, FIXED (name the commit if `git log -S` or the item's text shows it),
   or CHANGED (describe how).
3. Root cause in the crate: file:line and one sentence. Do not fix anything.
4. Size: S (one site, under a day), M (several sites or a design choice), L (a phase's worth). Name
   the files a fix touches.
5. Risk: anything a fix would interact with in 6.1's work (the sim mode, the seeded gate, R9's truth
   judgment, R10's byte trigger, the heapshape round).
6. For perf items, give callgrind or wall-clock figures for the crate before (HEAD) and, where the
   item names a scratch fix, the expected gain. Do not land the fix.

Write `.superpowers/sdd/2026-10-07-phase-6-1/<your-report-name>.md`. Start it with one summary
table (item, verdict, size, files, risk) and follow with a section per item. Each section quotes
the probe text and the exact commands, and gives stdout, stderr and status for both engines. Return
only the summary table.
