# Phase 9b1 gate record

Spec: `docs/superpowers/specs/2026-10-14-phase-9b1-uniform-object-model.md`. Plan:
`docs/superpowers/plans/2026-10-14-phase-9b1.md`. Ledger and reviews:
`.superpowers/sdd/2026-10-14-phase-9b1/` (`final-review.md`, `final-rereview.md`).

## Close run

At `d58e12600`, by `docs/superpowers/records/2026-10-14-phase-9b1/close/run.sh`:

```sh
cd rust
REXX_CORPUS_GATE=1 CARGO_INCREMENTAL=0 timeout 2400 memcap 8G cargo test -j 4 --workspace --release --no-fail-fast
REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../close/suite/table.tsv CARGO_INCREMENTAL=0 \
    memcap 8G cargo test --release -j 4 -p rexx-exec --test suite l3_core_suite -- --nocapture
```

Status file (`close/status.txt`): start 2026-10-15T13:20:07Z, workspace exit 0 at 13:52:37Z, suite
exit 101 at 13:57:11Z, `finished`. The suite's exit is its whole-suite assertion (46 of 402 groups
not green, as at the 9b close); the logs and `close/suite/` hold the table, verdicts and per-test
record.

**Close command exit status:** 0. It includes `collect_stress`, `representation_refusals` and the
corpus differential under `REXX_CORPUS_GATE=1`.

## Criteria (spec section 6)

**Every leak in scout-leaks' ranked table is fixed or licensed.** Met.

| leak | disposition | where |
|---|---|---|
| L1 WeakReference to a held inline value cleared | fixed | Task 2 |
| L2 operators on a String subclass not sent, DO setup `+` too | fixed | Task 2 |
| L3 class-side `STRING` not sent | fixed | Task 2 |
| L4 value identity and `objectName=` | licensed, Deviation 36 (spec section 1) | Task 1 |
| L5 Package subclass copy refused | fixed | Task 4 |
| L6 REXX package copy refused | fixed | Task 4 |
| L7 packages never collected | fixed, including a package defining a class | Task 5 |
| L8 Method/Routine/setMethod/Package~new never freed | fixed | Task 6 |
| L9 Message/Method/Routine side-map rows never pruned | fixed | Task 7, Task 8 |
| L10 internal-row Routine copy refused by addRoutine | fixed | Task 3 |
| L11 native routine frame missing from stackFrames | fixed | Task 3 |
| L12 SysStemSort writes back by tail after mid-sort mutation | licensed, Deviation 37 | Task 1 |

L7 is fixed, not licensed. A Task 5 ledger ruling said a package defining a class is never
collected, under D59. That is false: such packages are collected on both sides
(`corpus/lang/package_class_collected.rex`; final review M4).

Every map the scout listed ends in one of the spec's three states, in
`docs/superpowers/records/2026-10-14-phase-9b1/side-maps.tsv`. Task 8b moved the remaining leaks
(minted method ids, retired behaviours, variable-reference cells) to heap objects or pruned tables.

**The refusal grep holds only permanent rows.** Met. `rexx-exec/tests/representation_refusals.rs`
greps `rust/crates` for its phrases and fails on any literal not on `ALLOWED`. Every row's owner is
Deviation 28, Deviation 32, Deviation 36, Phase 10, or "internal guard" (a refusal that no Rexx
program reaches while the collector's holders are right). The final review added "whose program was
freed" to the phrases. The sites that answered a default on a freed program's id now refuse loudly
(`8fa4a5694`).

**Section 1 witness.** Met. `corpus/lang/value_based_identity.rex` pins held-value `identityHash`
stability through reads, aliases, arguments, returns and collections, and full identity plus
`objectName=` on subclass and enhanced strings. It is in the corpus differential, which matches the
oracle.

**`base/class/Package` `TESTRECLAIM01` passes within its own wait.** Met. It was `blocked: timeout
after 60 s` at the 9b close and is `passed` here.

**No test regression against the 9b close.** Met. A join of `close/suite/tests.tsv` with
`2026-10-13-phase-9b/close2/suite/tests.tsv` on group and test changes exactly one row:
`base/class/Package TESTRECLAIM01`, timeout to passed. Groups: 356 of 402 green, as at the 9b close.

**Memory: nothing grows with the iteration count.** Met. Peak RSS (kB), `close/memory/memory.txt`,
by `close/memory/memloop.sh` from that directory, head binary
`/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9b1-d58e12600`:

| loop | oracle 20000 / 80000 | head 20000 / 80000 | head 80000 vs 20000 |
|---|---|---|---|
| `Method~new` | 20888 / 20624 | 45692 / 45880 | +0.4% |
| `Routine~new` | 20692 / 20576 | 45500 / 45820 | +0.7% |
| source-string `setMethod` (Directory) | 20816 / 20628 | 45732 / 45192 | -1.2% |
| `Package~new` | 20608 / 20804 | 54928 / 54796 | -0.2% |
| `call ext` | 20932 / 20304 | 55384 / 55400 | +0.0% |
| message loop | 18924 / 20584 | 33056 / 32872 | -0.6% |

The message loop and `setMethod` were also run at 320000: head 32556 and 45100, oracle 20628 and
20584. Before 9b1, 80000 `Method~new` peaked at 1.70 GB.

The absolute gap to the oracle is the collector's 32 MB floor (`COLLECT_BYTES_FLOOR`), licensed as
GC timing (spec section 6, controller ruling 2026-10-15). Task 7 measured an 8 MB floor: it cost +50%
on `churn`.

**Performance: every bench program within +0.5% of the Phase 9 pin.** See below.

## Callgrind

Met. `close/callgrind.txt`, run from the repo's `rust/` (final review M5: a `/tmp` git-archive copy
moves `compound`'s whole-process figure by layout alone):

```sh
bench-programs/callgrind.sh -B 0.5 -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9b1/close \
    pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run head=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9b1-d58e12600
```

Exit 0, every spread under 0.01%. The pin is sha256 `c4b4e8c4…`. The head binary is `cargo build
--release --bin rexx-run` at `d58e12600`, sha256 `0a9a562c…`. The binary the test run left in
`target/release` was not used, because the build recompiled `rexx-exec`. The largest whole-process
rises are `startup` +0.335%, `interpret` +0.292%, `sayloop` +0.242%, `churn` +0.231% and `sendloop`
+0.140%. `startup` is +0.508% with libc subtracted, but the budget judges the whole process. Falls:
`extcall` -13.2%, `intcall` -10.1%, `pkglocal` -6.5%, `dispatchclass`, `fibcall` and `fibfunc`
-3.4% to -3.5%. `interpret`'s whole-process rise is mostly glibc: it is -0.105% with libc
subtracted.

## Carried forward

- Queued, priority: `Routine~start('CALL')` panics
  (`.superpowers/sdd/queued/2026-10-14-routine-start-call-panic.md`). It predates 9b1.
- Queued: `.superpowers/sdd/queued/2026-10-15-routines-table-silent-none.md` (the final re-review's
  two minors), `2026-10-15-9b1-task-8b-review-minors.md`, `2026-10-15-isguarded-copy-and-additional.md`,
  `2026-10-15-executable-flags-and-result.md`, `2026-10-14-native-method-frames.md`,
  `2026-10-14-class-side-defaultname.md`, `2026-10-14-filespec-string-arg.md`.
- The deferred SysFork and named-semaphore refusals keep `Owner::Phase9`, owner undecided
  (`phase-9b-gate.md`).
- Oracle bugs found while witnessing are not copied: the oracle clears a WeakReference to a held
  `.true` (Task 2), and oracle-crashes entry 49 (Task 8b).
