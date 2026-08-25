# Task 10, fix round 2 -- three documentation fixes, no code behaviour

**Round 1 is substantially solid.** The four-arm receiver decision was confirmed against the oracle by
the reviewer's own probes, and the test reddened structurally when the `Trap` arm was mutated in a
sandbox. The `allow` control is **real**: stripping both attributes names exactly
`methods 'receiver' and 'package' are never used` at the accessors' `impl` block, not the fields. None
of round 1's four false statements survives anywhere in the report, checked for orphan copies as well
as for the wrong figures. All five C++ citations correct. And **the +18 per `CALL` was independently
reproduced** by `perf stat` against a sha256-verified rebuild: 17.999847, against your 18.00.

Three fixes. None changes behaviour.

## 1. A false sentence in new code

`plan.rs:53`-`:54`. The new `Package` doc says `Interp::class_packages` keys on `Package`. It does not:
`class_packages` is `HashMap<ObjRef, ProgramId>` at `lib.rs:2347`, untouched by your diff and still
keyed on `ObjRef`. Say what it is keyed on, and if the intent was that it *should* be rekeyed, say that
as a separate sentence naming who would do it -- do not let a plan for later read as a fact about now.

## 2. A stale claim in the report, one paragraph from the one you closed

The pre-round "What I could not close" section still asserts that `resolve`'s `debug_assert` stands
behind `Caller`'s fields. **Your own item 2 deleted that assert.** The neighbouring paragraph in the
same section was explicitly closed out; this one was not.

Worth naming rather than just fixing: you corrected four false statements **in place** in that same
file, and this one sat beside them. A section headed "what I could not close" is the one place a
reader trusts to be current, and it is the section least likely to be re-read when the thing it
describes gets closed.

## 3. The `task` column's two spellings, and my framing of it was wrong

Confirmed labels in `phase-5a-arms.tsv`: `6`, `7`, `8`, `9`, `9-fixround-1`, `10`, `10-fixround-1`.

**My "one sitting versus several" framing was imprecise** -- task `9` bare also holds two commits, a
base and a pre-review performance fix. The real asymmetry is narrower and worth stating exactly:
**only the reviewer-driven fix round moved to a `-fixround-N` suffix, and only for Tasks 9 and 10;
Tasks 6, 7 and 8 recorded theirs under the bare number.**

**Neither `rust/bench-baselines/README.md` nor `PINNED.md` documents either spelling.**

**Ruling: document both in `README.md`, and do not touch the committed rows.** The rows are
measurements; rewriting them to a uniform spelling would be editing data to match a convention decided
afterwards. What a reader needs is to be told that a task's sittings may appear under the bare number
or under a suffix, that the **commit** column is the reliable discriminator in either case, and which
tasks used which. State it as what the file contains rather than as a rule for the future -- a later
plan can decide its own convention.

## Verification this round owes

* The five gate commands, each status read **unpiped** and as this shell's own child -- the re-review
  could not confirm gate 5's exit code because it ran as a background job, and that is worth not
  repeating.
* No sitting: state which side of the `.text` predicate you land on, with the two sha256 sums if you
  claim it does not change. Comment-only edits plus a Markdown file should leave `.text` identical, and
  that is now a claim with a measured pass and a measured failure behind it, so it is checkable rather
  than assumed.
* Report section **before** the commit; send me the SHA. This is the last round -- Task 10 closes on it.
