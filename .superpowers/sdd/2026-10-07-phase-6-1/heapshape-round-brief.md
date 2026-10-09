# Heapshape perf round 1

The Phase 6.1 perf budget is +0.5% against the 6.1 base `e6af1198b` on every program. heapshape is
at +1.1558%. `heapshape-bisect.md` in this directory attributes the overrun to Task 5a (R10, the
byte-aware collection trigger). It is added work: `live_bytes += object.body.held_bytes()` runs for
every survivor in both mark loops of `Heap::collect` (+17.83M Ir), and the `Array`/`Buffer` arms of
`Body::held_bytes` lengthen that per-survivor match (+6.94M Ir). Read the bisect report and
`global-constraints.md` first: memcap on every run with `timeout` inside, no `bash -c`, `rm` only
with literal absolute paths, stage by path, commit with `-F` and the attribution lines.

## Change

Stop summing live bytes over survivors at collection time. Keep live bytes as a running figure
instead: charged bytes are added where R10 charges them today, and the bytes of every body the
sweep frees are subtracted, along with any other path that releases a charged body or shrinks a
charged one. The observable stays R10's: a collection triggers on the same byte condition as
today. Find every place that changes a body's held bytes, and name the enumeration command in the
report.

* **Debug check.** In debug builds, `Heap::collect` still computes the survivor sum and asserts it
  equals the running figure, with a message naming both. Invert it once to show it fires: drop
  one subtraction and run a test that reaches it. Record that run.
* **Tests.** The existing R10 trigger tests stay green unchanged. Add one crate test in which a
  charged body shrinks or is freed between collections (array shrink, buffer truncation, or
  dropped strings) and the trigger point is the same as before the change. Run that test against
  the base too, and say in the report whether it passes there.
* **Perf.** Use callgrind on the eight programs (`pingmsg pingguard pingsem alloc alloc4c heapshape
  rexxcps emptyloop`) with `rust/bench-programs/callgrind.sh`, against three columns:
  * HEAD before this round, `7aedf9501`, the current HEAD you start from;
  * base61, `/tmp/claude-1000/p61/t1/bin/base/rexx-run`; verify its sha256 against the gate
    record's `## Task 1`;
  * this round's binary.

  Interleave wall clock as the constraint says. Target: heapshape within +0.5% of base61, and no
  other program's running total over +0.5%. If the target is missed, attribute what remains with
  `cgdiff.py` and say what a second round would change. Do not start a second round.

Per-task check: fmt, clippy `-D warnings`, plain workspace test, and the `REXX_CORPUS_GATE=1`
corpus pair. Also run the seeded gate in release (`REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4
--release -p rexx-exec --test concurrency_tests the_seeded_gate`), because collection timing feeds
its step counts. Build in `/tmp/claude-1000/p61/hsr/` with one target dir. `/tmp` is inode-tight,
so check `df -i /tmp` first.

Report to `heapshape-round-report.md` in this directory. Record the perf table under a new
`### Heapshape round 1` heading in the gate record's `## Task 9` section, or the section that holds
running totals. Commit code, report and record. Return only: status, commit shas, perf line
(heapshape and the worst other program against base61), and concerns.
