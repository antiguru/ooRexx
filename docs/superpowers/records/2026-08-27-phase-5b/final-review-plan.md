## Phase 5b whole-phase review

**Scope.** `4c553f383..60256a8cc` — 51 commits, 171 files, +12,562/−586 under `rust/`.

**This is not a general read, and the reason is measured.** The last consolidated review on this
project found **zero** code defects across 52 files and 3,110 added lines, and **five** prose defects;
prose fix rounds here have introduced new false statements at 5, 0, 4, 0 across four rounds. A
read-everything pass would spend its budget on the comment layer that the comment-policy tightening
at 5b's close will rewrite anyway. **A prose or comment pass is explicitly out of scope.**

**What the phase's own defects have in common, and what this review hunts.** Every real defect found
in 5b after a task believed itself finished was one class: **a check that passes over the absence, or
the wrong instance, of its subject.** Eight instances, none caught by rereading and all caught by
running something:

| instance | how it passed |
|---|---|
| two D62 `::ATTRIBUTE DELEGATE` gate rows | exercised the delegating setter, never the getter; a build with a plain getter left the corpus at 315 of 315 |
| `accessor_variable` | green over nothing; actually a divergence |
| `message_array_shape` | marked `diverges` from one program, `answer` measured on a different one |
| `invalid_position` | carried its *neighbour's* real error number; nothing distinguishes it by inspection |
| the DEVIATION marker check | one-directional while its doc claimed both halves protected |
| the tree-hash guard | covered three named files while claiming the tree |
| Task 8's index discriminator | a transposed mapping transposes write and read alike, so the pair swaps together |
| Task 9's send-surface walk | 17 of 55 rows reached a different check that fires first |

Four of the eight were found by mutating a subject and watching what did **not** go red. That is the
method this review generalises.

**Timing.** Before 5c, which inherits these instruments directly: `corpus/refusal-sites.tsv`,
`tests/refusal_sites.rs`, `tests/licensed_divergences.rs`, and both gate tables.

---

## The three strands

Independent, no shared state, run in parallel. **Each reviewer works from its own `git archive`
extract with its own `CARGO_TARGET_DIR`; the main worktree is read-only to all three.** Mutation is
the method here, so three agents mutating one tree would invalidate each other's runs.

### A. Witness liveness

For every witness 5b added or touched, mutate its subject and confirm it reddens. The population, to
be derived by command rather than typed: the 62 files under `rust/corpus/` in the phase diff, the 6
`5b` rows in gate table C, the `5b` rows in gate table D, the 2 rows in `LICENSED_DIVERGENCES`, and
the send-surface rows of `corpus/refusal-sites.tsv`.

**Note a discrepancy to resolve first**: the controller's grep for `"5b"` in `gate_table_d.rs`
answers 1, while the phase gate's own summary reports `5b: 2 rows`. The gate's figure is the
authoritative one and the grep is wrong; find out how the second row is spelled before deriving any
population from a pattern like it.

A witness that cannot be reddened by any mutation of its subject is a finding. So is one whose
mutation reddens *a different* witness instead — that is `message_array_shape`'s shape.

### B. The seven tasks with no independent reviewer

Tasks **0, 2, 6, 7, 8, 9, 10** were reviewed by the controller, who shares the plan's assumptions —
and those assumptions were themselves the defect three times (Task 8's discriminator, Task 10's
pin/baseline claim, the D59a citation in the Task 6 boundary message). Tasks 1, 3, 4, 5 had
independent reviewers and are out of scope.

Read each against **its own controls**, not for style: is the named control the one that could fail?
Was it run? Would the task's acceptance have passed over a wrong implementation? Every task's report
is committed under `docs/superpowers/records/2026-08-27-phase-5b/`.

### C. D-number coverage

The 5b spec defines **D57 through D69, including D59a**. For each: does a committed witness exist,
and does that witness fail when its subject is removed? A decision with no witness is a finding; a
decision whose witness cannot fail is a worse one.

D59a's licence is the special case — two of its four consequences are 5c's and are *supposed* to have
no 5b witness. Check that the ones 5b owes are present and that the ones it does not are recorded
against the phase that owes them, rather than merely absent.

---

## Rules for all three reviewers

* **Report findings; do not fix.** The controller rules on each and dispatches any fix round.
* A finding needs a transcript. A claim that something "could" fail is not a finding until a
  mutation shows it does or does not.
* Never run a program in `rust/corpus/oracle-crashes.txt` and never construct one of those shapes.
* Oracle probes from a fresh empty directory; three descriptors read separately; both engines.
* `cargo test <name>` exits 0 when it matches nothing — assert a non-zero run count, or a mutation
  harness cannot tell "passed" from "does not exist".
* `cargo test` stops at the first failing binary without `--no-fail-fast`, so "nothing else caught
  it" is unmeasured without it.
* Run mutations under `memcap 8G` where available: a mutation's whole purpose is to execute code
  known to be wrong, and wrong code has no bound on what it allocates.
* Restore from your own copy, never `git checkout --`.
