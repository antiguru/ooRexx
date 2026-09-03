## Task 1 -- the construction route, and surface B

**BASE:** `f3b637fde`. Tree clean.

**Read first, in this order:**
1. `docs/superpowers/plans/2026-09-02-phase-5c.md` -- especially "The premise, in one paragraph" and
   the Global constraints. This plan is the SECOND version; the first had a false premise.
2. `docs/superpowers/specs/2026-09-02-phase-5c-class-methods.md`, the section
   **"What table C actually measures -- the correction"**. Sections M2-M6 there are SUPERSEDED --
   do not build from them.
3. `.superpowers/sdd/2026-09-02-phase-5c/plan-review-report.md` -- F1, F3 and F5 are yours.
4. `rust/CLAUDE.md`.

---

## Your subject

**The construction route, and the eight surface-B classes whose `~new` needs arguments.**

`gate_table_c.rs:815` states your obligation in its own words:

> Every instance-arm program here opens with a bare `~new`, and there is no route to a committed
> construction program because none exists to route to. **The task that commits the first one has to
> add that route here in the same change**; what it must not do is flip a class to `covered` and
> leave the derived probe on a bare `~new` that raises.

You are that task. Build the route.

**Surface B, measured at BASE by running every probe** -- these eight are rc 216, meaning `~new`
reaches the class's `init` and raises there because it wants arguments:

```
alarm__instance   caselesscolumncomparator__instance   columncomparator__instance
file__instance    invertingcomparator__instance        ticker__instance
timespan__instance                                     (file is 5d's -- see below)
```

`file__instance` is **5d's**, not yours. Leave it. The comparators take 1, 2 and 2 arguments
respectively (measured: `93.901 Not enough arguments for method; N expected`).

`.TimeSpan` is the worked case and the cheapest proof the route works: `.TimeSpan~new(1)` **already
answers `0.000001` on this crate**, byte-identical to the oracle. Its 47 rows need the program and
nothing else.

## The thing to be careful about

`covered`'s second limb is the hand-written `CONSTRUCTION` const at
`crates/rexx-extract/src/docs/classes.rs:117`, and `status_of` reaches `Covered` only through it.
**So flipping a class to `covered` is a string edit, and the status by itself is evidence of
nothing.** `method_probe_text` (`gate_table_c.rs:824`) emits `o = .X~new` regardless of status --
that is exactly the gap you are closing.

Whatever you build must make "this class is `covered`" mean "the derived probe constructs an
instance". In your report, say **how you made that true, and what would still let someone flip a
class without it.**

## Done when

* The route exists and the derived probe for a `covered`-by-program class opens with that program
  rather than a bare `~new`.
* Each of surface B's seven (excluding `file`) is **rc 0 and matches the oracle on three descriptors,
  both engines**.
* `tests/extract_docs.rs` still re-derives and compares in both directions.
* **The control has been run**: flip a class to `covered` with no committed program and confirm
  something reddens. If nothing does, say so plainly -- that is a finding, not a failure.
* The table C not-yet-agreeing count is reported before and after (it is 868 at BASE).

## Rules

Every Global constraint in the plan binds you. The ones that bite here:

* **D74**: a doc comment is one sentence plus at most a short paragraph. No results, no reasoning,
  no alternatives -- those go in your report. `964dc68ba` is the worked trim.
* Oracle probes from a **fresh empty directory**; three descriptors read separately, never `2>&1`;
  both engines; capture exit status immediately, not after an intervening command.
* `/bin/grep -a` for counts over data files; a `grep -c` over source misses multi-line entries.
* Gate-table report output goes to **stderr**, and `cargo test` captures it without `-- --nocapture`.
* A gate run does not survive the turn that starts it. Tree hash before the first gate and after the
  last.
* No `unsafe`. `oodocs/`, `ootest/` and the C++ tree are read-only.
* `git commit -F <file>`, paths named explicitly; `Cargo.lock` unstaged.

Report to `.superpowers/sdd/2026-09-02-phase-5c/task-1-report.md`. **Say plainly what you did not do.**
