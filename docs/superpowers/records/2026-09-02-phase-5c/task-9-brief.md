## Task 9 -- the real 98.972, and `String~MAKEARRAY`

**BASE:** `76b61b3e8`. Tree clean, all five gates 0 at `d769a847c`. Table C's 5c count is **110**
(the floor) and table D's is **2**. **No row count should move.** Both subjects close a divergence.

**Read first:**
1. `docs/superpowers/plans/2026-09-02-phase-5c.md` -- **"What comes into 5c at the close"** for why
   these two and not the eight that hand over, and the Global constraints including **"Working on
   one worktree"**.
2. `.superpowers/sdd/2026-09-02-phase-5c/task-6-report.md` -- it built the LOSTDIGITS detection and
   measured what the real condition needs. Its measurements are your starting point.
3. `.superpowers/sdd/2026-09-02-phase-5c/task-4-report.md` -- the `>x` fix, which is what made
   `String~MAKEARRAY` reachable.
4. `rust/CLAUDE.md`.

---

## Part 1 -- raise 98.972 instead of refusing

`::OPTIONS LOSTDIGITS SYNTAX` currently detects the lost digit and refuses loudly. That was the right
first step and it is not the answer: the oracle raises `98.972`. Measured at `231a2bb71`:

```
numeric digits 3 / say 1.23456789 + 0 / ::options lostdigits syntax
  crate   rc 120  loud refusal
  oracle  rc 158  Error 98.972
```

**Task 6 measured what this needs and you should confirm rather than re-derive it:** the
substitution is the operand's **plain rendering** -- `1.23456789e2` reports `1.23456789E2`, not the
source text and not the rounded value -- so it needs `string_value_text` and nothing new. And where
the refusal fires, `98.972` is unambiguous, because arming a trap clears the escalation. Closing the
trap route is the same three-way shape `novalue_raised` already has.

Task 6 also measured the surface on 47 single-operation programs, **46 of which line up**: both
operands of `+ - * / // %`, the base of `**`, prefix `+`/`-`, non-strict comparison where both
operands are numeric, and a controlled `DO`'s `Initial`/`TO`/`BY` raise; strict comparison,
concatenation, `abs`/`trunc`/`format`/`max`/`sign` and whole-number argument conversions do not. The
one residue is `do 123456789`, where we answer 26.2 and the oracle 98.972 -- and 26.2 is what
**both** sides answer without the directive. **Re-run that sweep against your change**; it is the
regression test for this part.

**`CONDITION` is the other spelling and Task 6 found it clean** -- it turns the escalation off, rc 0
on both sides. Check it still is.

**The detection's cost is already paid and accepted** (+0.26% ir / +0.19% tree-walker on
`arith.rex`, nothing elsewhere; Moritz accepted it 2026-09-03). **If rendering the condition instead
of refusing costs measurably more than that, stop and tell me the figure** rather than absorbing it
-- interleaved, separate target directories, SHA-256 beside each number.

## Part 2 -- `String~MAKEARRAY`

Measured by the controller at `15fc08c72` on a hashed binary:

```
say 'abc'~makearray~items     crate rc 120  method "MAKEARRAY" of class "String"
                              oracle rc 0   1
```

A string's `makeArray` is a one-item array. **Measure the surface before building**: what the oracle
answers for an empty string, for a string containing line separators (this is `makeArray` -- check
whether it splits), for a stem reference, and what `~request('ARRAY')` answers beside it. Task 4
recorded that `~request('ARRAY')` and `FORWARD ARGUMENTS` disagree with each other on references, so
do not assume these two routes agree.

This is the divergence `>x` made newly reachable: `o~unknown('LENGTH','notanarray')` under a syntax
trap is rc 0 on the oracle and rc 120 here. **That program is the witness** -- it must go from
divergent to agreeing, and you should run it before and after.

## Done when

* `98.972` is raised where the oracle raises it, byte-identical on three descriptors on both
  engines, and the 47-program sweep is re-run with its result stated.
* `String~MAKEARRAY` matches the oracle on the shapes you measured, or the ones you did not
  implement refuse loudly and are listed.
* The `~unknown` witness agrees, with its before-and-after transcript.
* A corpus witness exists for each part that **fails before your change and passes after** -- run
  both ways and record both.
* Table C's 5c count is still **110** and table D's still **2**. If either moves, say why.
* Five gates 0 and the corpus headline at its full count.

## Rules

Every Global constraint in the plan binds you -- **read the "Working on one worktree" section**. The
ones that bite here:

* **D74**: one sentence plus at most a short paragraph per doc comment. No results, no reasoning, no
  alternatives -- those go in your report.
* **No `rm` with a glob or a computed path** -- in a subagent that prompt cannot be answered and
  deadlocks you while you still report as running. A named literal path you created is fine. Name
  anything large you leave behind.
* **`/tmp` is a RAM disk.** Small files there; any isolated build tree on real disk under
  `/home/moritz/dev/repos/claude-build-scratch/task-9/`. Check `df -h /tmp` first.
* **Separate revisions need separate target directories**, and every figure carries the SHA-256 of
  the binary that produced it. One `CARGO_TARGET_DIR` gave two revisions the same hash this phase,
  which reads as a clean 1.0000x.
* **Interleave any A/B comparison.** Non-interleaved runs have twice produced a difference that was
  machine load.
* **Rebuild before any sweep following a revert or a mutation run** -- `cargo test --release`
  relinks `rexx-run`, so a mutated binary outlives its revert with `git status` clean.
* **Never wait with `pgrep -f`** -- it matches your own polling shell.
* **Run the three test gates with `--no-fail-fast`.** Gate 5 is slow; that is normal.
* **`directive_options.rs` carries a licensed flake** on `directive_options_trace_reply.rex` -- if
  you see two threads' trace lines out of order, that is Deviation 7 and not yours.
* Oracle probes from a **fresh empty directory**; three descriptors separately, never `2>&1`; both
  engines (`REXX_ENGINE=ir`, `REXX_ENGINE=tree-walker`); exit status captured **immediately**.
* Never run a program in `rust/corpus/oracle-crashes.txt`. Never `NUMERIC DIGITS` above 1000. No
  `unsafe`. `oodocs/`, `ootest/` and the C++ tree are read-only.
* `git commit -F <file>`, paths named explicitly; `Cargo.lock` unstaged.

Report to `.superpowers/sdd/2026-09-02-phase-5c/task-9-report.md`. **Say plainly what you did not do.**
