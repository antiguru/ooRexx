## Task 10: the flip

**Goal.** `5b` gates.

**BASE:** `5eb74f6a3`, the commit named in your dispatch. Tree clean. Read
`.superpowers/sdd/2026-08-27-phase-5b/global-constraints.md` and the 5a constraints it points at,
`rust/CLAUDE.md`, the plan's Task 10 section
(`docs/superpowers/plans/2026-08-27-phase-5b.md:995` onward), and D65 in the spec.

**This is the last task of Phase 5b.** Tasks 0 through 9 are closed and every 5b gate row already
agrees -- the controller re-ran the phase gate at `dcd8b468d` and again through Task 9's own run, exit
0 both times. So the flip should be small. **What makes it not routine is that D65's criteria must all
hold at *one* commit**, and the sitting.

---

## Verified by the controller against the tree at BASE. Re-measure all of it

* **`CLOSED_PHASES` is `&["5a"]`** at `crates/rexx-exec/tests/gate_tables/mod.rs:344`. Your build is
  adding `"5b"` to it.
* **`corpus/phase-5b.txt` carries 63 non-comment entries** (`/bin/grep -avc "^#\|^$"`).
* **The pin is present and matches its record.** `bench-baselines/pinned/rexx-run-f558ea501`,
  sha256 `857787f797e88dd94ab839d7b66f1e08cd1a08d22c9e7978d05cbfd49f70494e`, identical to the value
  `bench-baselines/PINNED.md` records for it.
* **The three parked bench programs are where the plan predicted**, run at BASE on `ir`:

```
dispatch.rex    rc 0
alloc.rex       rc 120   method "OF" of class "Array" is not implemented (Phase 5)
heapshape.rex   rc 120   method "NEW" of class "Directory" is not implemented (Phase 5)
```

`heapshape` has moved one stage -- Task 8 cleared its `.array~new` and it now stops at
`.directory~new`, which 5b does not owe. `alloc` needs `Array~of` and `String~new`, neither 5b's.
**So exactly one program goes live: `dispatch.rex`.**

---

## The pin was corrected on 2026-09-02 and the wrong one is still on disk

The plan named `rexx-run-15a1ffa98` until `5eb74f6a3`. That is the **retired** 5a pin, 153 commits
stale when it was replaced. `pinned/` still holds both binaries, so measuring against the wrong one
produces a plausible table rather than an error, and that table would re-measure 5a's drift and bill
it to 5b. **Use `rexx-run-f558ea501` and verify its sha256 before you measure.**

## The sitting

Re-run the eight axes against that pin and record the result. Three rules, each of which has already
cost this project a wrong number:

* **Interleave the arms.** Two suite runs, one per side, once invented a 5% regression that was
  really a 5% improvement. `rexx-arms` interleaves; do not hand-roll two runs.
* **`rexx-arms` appends** to whatever `--baseline` names. It has previously appended a partial
  sitting to a tracked `.tsv`. Write to a new file or check what you are appending to.
* **Report instructions, not cycles.** Instructions are deterministic here to about seven
  significant figures; the `cycles` column moves several percent on a do-nothing control and is not
  readable at this resolution.

**`dispatch.rex` cannot show a regression against a baseline in which it did not run.** Its first
crate figure is a **new baseline row**, not a comparison, and must be recorded as one. "No regression
on N axes" where one of them has no prior figure is a ratio flattered by its denominator.

**If `perf stat` fails to count both `instructions:u` and `cycles:u` together, that is the sandbox
boundary and not a machine property.** Inside the agent sandbox exactly one hardware counter slot is
available: each event counts alone and any pair drops the second. Outside it, on the same machine the
same minute, both count. This has already been recorded once as a machine-wide shortage and left a
commit permanently unmeasured. **Do not diagnose it, do not work around it, and do not record the
sitting as impossible** -- say plainly that the pair would not schedule, and the controller will ask
Moritz to run it outside and paste the rows back. A failing pair is also evidence about the moment
rather than the instrument, so retry once before reporting it.

**The drift is known, ruled, and not yours.** Roughly twenty 5a steps each passing a half-percent gate
summed to +32.58% on the worst axis. Moritz ruled on 2026-09-01 that this is handled by a non-SDD
performance round after Phase 5, not by widening any gate now. Record what your sitting says and do
not re-litigate it.

## The stand-in decision

`dispatchclass`'s header says it "does not replace dispatch.rex or relieve whichever task unblocks
it". `dispatch.rex` is now unblocked, so **decide and record** whether `dispatchclass` stays an axis
beside `dispatch` or is retired -- leaving both unexamined double-counts a dimension in every later
comparison. `alloc4c`'s header says it is not a substitute once message sends land, but `alloc.rex`
does not go live in this phase, so `alloc4c` stays; say so rather than leaving it unmentioned.

---

## Done when

D65's criteria hold **at one commit**:

* every `5b` row in both tables `agree`, with no structural failure;
* every program in `corpus/phase-5b.txt` agrees on three descriptors on both engines;
* Task 6's DEVIATION witness runs (`rexx-exec/tests/licensed_divergences.rs`);
* `CLOSED_PHASES` holds `"5b"`;
* the five gates pass, **with the command that printed each figure quoted beside the figure**.

Plus the sitting recorded, the new-baseline row marked as one, and the `dispatchclass` decision
written down.

**After the flip, a red 5b row is a gate failure for everyone with no env var set.** Check that the
gates you run after adding `"5b"` are the ones that would catch it -- a flip that gates nothing is
the failure mode here, and the cheap control is to redden one 5b row deliberately, confirm a plain
`REXX_CORPUS_GATE=1 cargo test` fails on it without `REXX_PHASE_GATE`, and restore.

## Rules

* Correct the plan or spec where you find it wrong. Tasks 8 and 9 both did; the pin above is one the
  controller already fixed.
* Never run a program in `rust/corpus/oracle-crashes.txt` and never construct one of those shapes.
* No `unsafe`. Stop and say so rather than reach for it.
* Three descriptors read separately, never `2>&1`. Both engines.
* **A gate run does not survive the turn that starts it.** Write each status to a file, arm a waiter
  that exits on the process vanishing as well as on completion, and read the statuses in the same turn
  you commit. Three of the last four tasks' runs finished green and sat unread.
* Tree hash before the first gate and after the last, covering untracked files' **bytes**. Task 8's
  and Task 9's instruments do this correctly; copy one.
* **Audit every `interpreter/` citation you add.** Task 7 found seven wrong, Task 8 four.
* Commit with `git commit -F <file>`, naming paths explicitly. Never `git add -A`, never amend, never
  a bare `git stash`, never `rm` with a star glob, never `git checkout --` on a file you have edited.
  Re-read `git diff --cached --stat` and confirm `Cargo.lock` is absent -- a gate run rewrote it
  between review and commit on this plan once and ~30 unreviewed dependency bumps shipped.
* Write your report to `.superpowers/sdd/2026-08-27-phase-5b/task-10-report.md` (git-ignored). Say
  plainly what you did not do.
