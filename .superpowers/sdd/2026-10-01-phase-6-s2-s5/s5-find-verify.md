# S5 finding: criterion 10, the *(verify)* items of spec section 6 (agent `verify`)

Tree measured: `git archive 0144bdac5` (rust/ and interpreter/) built to
`/tmp/claude-1000/p6-s5-verify/target`, `cargo build --release -p rexx-exec --bin rexx-run`
(log line `Compiling rexx-exec v0.1.0`). Evidence: `s5-evidence/verify/` (probes, per-run logs,
`run.sh`, `summary.txt`, `logs/cargo-lib-uninit.log`). Load average 12.6 at the time of the runs.

## 1. Enumeration of the items

Command, from the repo root:

```
grep -n -i 'verify' docs/superpowers/specs/2026-09-29-phase-6-concurrency-design.md
```

Output:

```
352:Oracle behaviour is Read or Measured in the reviews unless marked *(verify)*, which the plan settles
426:  (`concurrency/Activity.cpp:249`, `:321-324`; `InterpreterInstance.cpp:571`) *(verify the exact
514:10. Every *(verify)* item of section 6 is settled against the oracle.
```

Line 352 is the definition and 514 is the criterion, so section 6 (lines 350-431) holds one item:
line 425-427, **UNINIT** "runs on the ending activity when it ends, and at interpreter termination
... *(verify the exact ordering)*". The search is case-insensitive on `verify` alone, so a marker
split across lines by the hard wrap is still found (line 426 is such a split).

History, for the lead only: the first draft (9385003fa) had three other items (whether the oracle
detects a guard deadlock; how it reports an untrapped error in a started send; which started
activities program end waits for). Review round 559a3e0c5 removed them and the spec now states each
as Read or Measured (`Activity::checkDeadLock` and 98.905; `MessageDispatcher.cpp:64-72`,
"measured"; the Program end bullet, "measured"). Command:
`git log -p --format='COMMIT %h %s' -- docs/superpowers/specs/2026-09-29-phase-6-concurrency-design.md | grep -n -i -E '^COMMIT|^[-+].*verify' | grep -B1 -i verify`.
They are not section 6 *(verify)* items at HEAD.

## 2. Prior settlement

`docs/superpowers/plans/phase-6-gate.md` `## S2`, `### UNINIT ordering (spec section 6, the
*(verify)* item)` (lines 117-150) settled it in Task 9, probes committed in
`docs/superpowers/records/2026-10-01-phase-6-s2-s5/uninit-ordering-probes.md`. Rulings: P39
(program end does not wait for activities termination UNINITs start), the standing GC-ordering
licence (Moritz 2026-09-01, `docs/superpowers/plans/2026-08-27-phase-5b.md:646`). Open queue item:
`.superpowers/sdd/queued/2026-10-02-uninit-reply-at-termination.md` (u5).

## 3. Re-run at HEAD (30 runs per side, every probe)

Probes extracted byte for byte from the record:
`awk '/^## /{n=$2} /^```rexx/{f=n".rex"; inb=1; next} /^```/{inb=0; next} inb{print > f}' uninit-ordering-probes.md`.
Runner: `bash run.sh oracle|ours|every <probe> 30`, from an empty directory, oracle under the
rules' wrapper, ours `rexx-run`, `every` = `REXX_SWITCH_MODE=every rexx-run`. Each row: stdout
lines `|`-joined, rc, stderr (all stderr empty).

| probe | oracle (30) | ours (30) | every (30) |
|---|---|---|---|
| u1 | 30 `main end\|activity end\|uninit live` rc 0 | same 30 | same 30 |
| u2 | 30 `activity end\|main end\|uninit dropped` rc 0 | same 30 | same 30 |
| u3 | 30 `activity end\|main end\|uninit dropped 1` | 30 `uninit dropped 2\|activity end\|main end` | as ours, 30 |
| u4 | 30 `activity loop done\|uninit dropped\|activity end\|main end` | 30 `activity loop done\|activity end\|uninit dropped\|main end` | as ours, 30 |
| u5 | 28 `main end\|uninit before reply\|uninit after reply`, 2 without the third line | 30 without the third line | 30 without |
| t3 | 30 `main end\|uninit starts\|uninit after start` rc 7 | same 30 | same 30 |
| t7 | 30 `main end\|uninit starts a poller` rc 0, max 0.01 s | same 30, max 0.03 s | same 30, max 0.03 s |
| `corpus/lang/uninit_after_every_activity.rex` | 30 `main end\|started activity end\|uninit of the object the started activity dropped` | same 30 | same 30 |

Crate tests, `cargo test --release -p rexx-exec --lib -- uninit` (log
`logs/cargo-lib-uninit.log`): 8 passed, 0 failed, including
`scheduler::tests::an_activity_a_termination_uninit_starts_is_not_waited_for` (t3/t7 shape),
`scheduler::tests::an_uninit_a_collection_readied_runs_when_its_activity_ends` (the ending
activity), `scheduler::tests::every_refusal_of_the_uninits_at_an_activitys_end_is_reported`.

Oracle source lines re-read: `concurrency/Activity.cpp:248-249` (`checkUninitQueue` after a
dispatch), `:321-324` (on deactivate when inactive), `runtime/InterpreterInstance.cpp:560-582`
(`terminationSem.wait()`, then `collectAndUninit`). The spec's citations hold.

## 4. Verdict

Holds, with the differences already licensed:

- The ordering itself (u1, u2, t3, t7, corpus witness): identical, 30/30, both modes. Termination
  waits for every activity, then sweeps; readied UNINITs run when an activity ends; activities the
  sweep's UNINITs start are not waited for.
- u3, u4: ours runs the UNINIT at the collection on the started activity (thread 2), the oracle
  at termination (u3) or one activation return later (u4). Collection timing: GC-ordering licence
  (2026-09-01), as the S2 record already says. Not a defect.
- u5: ours 0/30 third line, oracle 28/30 here (Task 9 review: 7/30 at load 29, 30/30 at load 14).
  Ours always takes an outcome the oracle produced (2/30 here). Settled by P39; the queue item
  stays open. The S2 record calls it a race and gives no frequency, which is still correct.

## 5. What the implementer must do

Nothing in code. In `phase-6-gate.md`'s S5 section, criterion 10: paste the table below, cite the
S2 section for the probe sources, and the evidence directory for the HEAD re-run. Optionally
update the S2 bullet for t3/t7 from "10" to note the 30-run re-run at S5 (do not rewrite the S2
figures; add the S5 run beside them).

Ready-to-paste criterion 10 table:

```
### Criterion 10: the *(verify)* items of spec section 6

Enumeration: `grep -n -i 'verify' docs/superpowers/specs/2026-09-29-phase-6-concurrency-design.md`
answers lines 352 (the definition), 426 (the item) and 514 (this criterion); section 6 holds one item.

| item (spec line) | claim | settled by | verdict |
|---|---|---|---|
| UNINIT ordering (425-427) | UNINIT runs on the ending activity when it ends, and at interpreter termination after every activity has ended; activities a termination UNINIT starts are not waited for | S2 `### UNINIT ordering` (probes `docs/superpowers/records/2026-10-01-phase-6-s2-s5/uninit-ordering-probes.md`); re-run at 0144bdac5, 30 oracle / 30 unswitched / 30 `every` per probe (`.superpowers/sdd/2026-10-01-phase-6-s2-s5/s5-evidence/verify/summary.txt`); `corpus/lang/uninit_after_every_activity.rex` 30/30 on all three; `cargo test --release -p rexx-exec --lib -- uninit` 8 passed | holds. u1, u2, t3, t7 and the corpus witness identical in both modes. u3, u4 differ in collection timing (GC-ordering licence, 2026-09-01). u5's REPLY continuation never runs here; the oracle runs it in 28 of 30 (load-dependent race with process exit), ruling P39, queued `2026-10-02-uninit-reply-at-termination.md` |
```

## 6. Open questions for the lead

- u5 stands on P39 plus a queue item; there is no Phase 6 divergence row naming it (searched:
  `grep -rn -l 'uninit after reply\|uninit-reply-at-termination\|P39' docs rust/crates rust/corpus
  .superpowers/sdd/queued` finds only the gate record, the probes file, `lib.rs` and the queue item).
  If S5 keeps a licensed-divergence list (the S4 section has `#### Rulings and licensed divergences
  of S4`, `phase-6-gate.md:829`), add a u5 row there citing P39.
