# Task 1, fix round 3 — rulings on re-review 2

Re-review at `.superpowers/sdd/2026-08-17-phase-5a/task-1-rereview-2.md`. **Every ruled finding from
round 2 is closed** — 8 of 8, and the two that mattered closed the way the corrected ruling asked
for. The verdict is REWORK only because four new low defects landed, three of them in prose and one
in a diagnostic. This round is small; it is not a re-litigation of anything.

Read the re-review — each finding carries its own fix, and in two places the fix is the exact
replacement text.

---

## D1 — **Ruled: fix.** A comment naming `"106 of 106 matching"` breaks the global constraint outright,
in the file whose own `SUBSET_FILES` doc says "a number a reader might eyeball is not a check", and it
is the class I ruled "fix" one round ago as N4. Name the set, not the count.

**The reviewer found a second defect inside the same sentence and it is the more interesting one:** a
non-finish is pushed to `mismatches` and never counted in `matched`, so such a run prints
`105 of 106`. The comment names **the one headline that cannot accompany the event it describes.**
Fix both — the constraint violation and the false example.

## D2 — **Ruled: fix.** "The pre-channel code named the path in exactly this case
(`.join().unwrap_or_else(...)`)" is history, added by the round that was fixing history. It fails the
ruling's own test: strike it and the block still says the same thing, because the last sentence
carries the whole argument. Delete the sentence and its parenthetical.

**Say in your report that your own collapsed-comment pass missed this**, and if you can tell why, say
that too. Round 2's report claimed the pass found no history framing beyond N3–N5; the reviewer's
pass over the same six files found this one. That is worth more to me than the fix.

## D3 — **Ruled: fix, with the per-file counter.**

The label `format!("{filename}#{checked}")` uses the **running total across every file**
`datadriven::walk` reads, while `render_oracle`'s doc says "`label` identifies the stanza". A hang in
the single-stanza `ir_dual_cases/rexxcps` reports `#152`, sending the operator to look for a 152nd
stanza in a one-stanza file.

**The decisive half is not the misleading number, it is that the number is not reproducible.**
`datadriven` 0.9.0's `test_files` walks `fs::read_dir` with **no sort**, so file order is filesystem
order: the same stanza gets a different label on another machine, and adding a case file renumbers
everything after it. A diagnostic that differs between two people looking at the same failure is
worse than no diagnostic.

Take the per-file counter reset inside the `walk` closure, leaving `checked` to go on counting
invocations for its assertion. Do **not** reach for `TestCase::line_number` — the re-review confirmed
it is private in 0.9.0.

## D4 — **Ruled: fix both halves, and one of them is mine to own.**

* **The comment.** `ir_dual_oracle.rs`'s "the shape **every** oracle-invoking harness in this crate
  uses" is checkable and false. Fix it.
* **The sixth site.** `input_oracle.rs:556`/`:567` reaches `expect_exit_code()` inside an `assert_eq!`
  with no `did_not_finish` in front, so a non-finish there panics from inside `support/oracle.rs`
  naming no program — **exactly the complaint M3 was raised about.** The re-review is right that this
  is not your omission: my round-2 brief named two "also in scope" sites and this was not one of
  them. **I am ruling it in now.** Leaving one site out is what made "everywhere" false, and the same
  event should not have a sixth treatment. Two lines, same shape as its sibling at `:449`.

**What I am not asking you to fix: the commit message.** `63a49c9b9`'s subject says "everywhere" and
its body says "everywhere the same event can occur", and one site falsified both when it was written.
That copy is uneditable and it stays. Do not amend or rebase reviewed work to correct prose. The
ledger records that the subject overreached and where the sixth site was closed — which is the only
durable place a later reader can learn it.

---

## Verification for this round

All five gate commands, each with **its own** exit status, plus the phase-gate command. Corpus stays
**106 of 106**. Both `oracle_deadline` tests pass under the gate — **two, not three; my earlier brief
said three and was wrong, and you were right to flag it rather than invent one.**

Run the collapsed-comment-block diff over every file you touch, and this time over the files you
touched in earlier rounds too, since D2 shows the pass can miss.

Append to `task-1-report.md` under "Fix round 3".

**Return only:** status, commit SHAs, one line per finding, and anything you could not close.
