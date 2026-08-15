### Task 11: The `base/keyword` L1 table

**Files:**
- Modify: `rust/crates/rexx-extract/src/lib.rs` (a third extractor)
- Create: `rust/crates/rexx-exec/tests/keyword_assertions.rs`
- Modify: `docs/superpowers/plans/l1-coverage.md`

**Why:** I28. The split table names `base/keyword` as 4b's L1 obligation and `base/bif` as 4c's, and **nothing extracts either today.** `rexx-extract` has `extract` (test methods) and `extract_assertions` (the `base/expressions` table) and nothing else.

**D12 is settled for 4b as a third extractor, not a generalisation.** `extract_assertions` is specific to `base/expressions`'s `assertSame` shape and already needed two modelling corrections -- single-quoted method names, and `expectSyntax` markers changing what a later `assertSame` *means*. Both were about a group's mechanics, and the same shape will recur. 4c makes its own call for `base/bif`.

**Step 1 of this task's first revision was "spend ten minutes checking whether `base/keyword` uses a shape `extract_assertions` already models". That check has been run, and its answer changes the task.** The numbers below were measured before dispatch, at ooTest r13178, and are the starting facts rather than something to re-derive:

* Pointed at `ootest/ooRexx/base/keyword`, the existing extractor yields **54 rows from 2,561 `self~assertSame` calls -- 2.1%**. Twenty-six of the 39 groups yield **zero**.
* It does not merely under-perform, it **breaks its own conservation law**: `rexx-extract-assertions --suite .../base/keyword` panics on the second group with `0 rows + 39 dropped != 265 assertSame calls`.
* The cause of that gap is real, not a counting artifact. **405 of the 2,561 calls are not at the start of a line** and `scan_method_for_assertions` only ever tests `trimmed.starts_with("self~assertsame")`. `base/keyword` writes multi-clause lines (`id=0010; ABBREV     =0010; self~assertSame(abbrev, id)`, `ASSIGNMENT`, 226 of them) and trailing assertions (`When i=0 Then self~assertSame(...)`, `IF`, 126 of them). `base/expressions` never does, so the blindness has never shown.
* A second latent defect surfaces on this group and not on the last one. `lower.starts_with("self~assertsame")` also matches the **120 `self~assertSameList` calls**, which `parse_assert_same` then rejects (its next char is `L`, not `(`) -- so each one calls `block()` and poisons every later `assertSame` in its enclosing method. `base/expressions` contains zero `assertSameList`, which is the only reason this has never mattered.

**The row model is what does not transfer, and the fix is a different shape of extractor.** `base/expressions` rows are `prelude` (a list of simple assignments) plus a self-contained expression. `base/keyword` tests *statements*: the assertion sits after a procedural body of loops, `IF`s and `SIGNAL`s that `simple_assignment` rejects by construction, which is where the other ~2,000 drops come from. **Extract whole method bodies, not preludes.**

**That shape is reachable in 4b, and this is the estimate that says so.** Of 26 candidate statement-shaped groups, 18 contain `assertSame`-bearing `test_*` methods, and in those **1,008 of 1,033 such methods use no message send at all other than `self~assert*`** -- they are plain classic Rexx. Those 1,008 methods carry **1,850 of 2,020 `assertSame` calls (91.6%)**. So a body-shaped extractor needs no message dispatch, and Phase 5 is not a prerequisite. Two caveats, both load-bearing. The estimate came from a throwaway script that split on `::method` and looked for a surviving `~` after deleting `self~assert*`, so it is approximate and worth redoing properly. And "no message send" is necessary, not sufficient -- a body free of `~` can still use a BIF we lack or an environment symbol -- so treat 1,850 as the ceiling to report against, never as a promised row count.

**A worked mechanism, offered so the task is not open-ended; take it or beat it.** Rewrite each extracted body into a standalone program in which `self~assertSame(a, b)` becomes an ordinary comparison our own interpreter can already run and report. That needs no new interpreter feature and no `self` object. If you find a cheaper shape that keeps the same population, use it and say why in the report.

**The conservation invariant alone is a tautology, and the first revision required it alone.** `rows + dropped == calls` holds at `0 + 0 == 0` (nothing scanned) and at `0 + N == N` (everything dropped). What made it non-vacuous in 4a was the *companion* test pinning absolute totals -- `total_rows`, `total_dropped`, per-group counts -- plus a non-empty assertion. Require all four:

1. `rows + dropped == calls`, where `calls` counts **every** occurrence including mid-line ones. The existing harness would satisfy a line-start-only `calls` while silently ignoring 405 assertions, so the law must be stated over the wider population or it certifies the blindness it exists to catch.
2. **Absolute committed literals**: the file count scanned, `calls`, `rows`, `dropped`, and the per-group row counts. Commit the file list the extractor scans, the way `phase-4a.txt` is pinned.
3. A floor: the row set is not empty. Given the ceiling above, make the floor a real number rather than `> 0` -- a body-shaped extractor that yields 54 rows has failed, and `> 0` would call that a pass.
4. The ooTest revision the literals were measured at, in the failure message. See the provenance note below for why.

**State the denominator's assertion spelling in the criterion's own text, and note that the obvious spelling over-counts.** Measured on `ootest/ooRexx/base/keyword`: 39 `.testGroup` files, **4,567** `self~assert*` occurrences of all spellings, case-insensitively. Of those, **2,561 match the prefix `self~assertSame`** but only **2,441 are exactly `assertSame`** -- the other 120 are `assertSameList`, a different method that the prefix test swallows. Pick one and say which: 2,441 leaves 2,126 assertions (46.6% of the group) outside the population, 2,561 leaves 2,006 (43.9%) but silently classifies 120 `assertSameList` calls as dropped `assertSame` calls. Say how many assertions are deliberately outside the population either way.

**`ootest/` is not checked-in test data, and two comments in the tree say it is.** It is git-ignored (`.gitignore:6`), has **zero** tracked files, and exists only as an SVN working copy of `svn.code.sf.net/p/oorexx/code-0/test/trunk` at **r13178** -- the main ooRexx worktree has no `ootest` at all. `tests/assertions.rs`'s `suite_root` doc comment ("this is checked-in test data in the same repository, not an external build a machine might be missing") and `rexx-extract-assertions.rs`'s ("they are already checked into the tree") are both false. Correct both while you are here, and pin the revision your literals were measured at so a red absolute-literal test is diagnosable as `svn up` rather than as a regression.

**What this task must not promise.** `tests/assertions.rs`'s 35 exempt rows will not move. All 35 are `unblocked_by: "Phase 5"`, verified 35 of 35, and the two whose first-observed blocker is a 4b construct re-block on a message send one line later in the same prelude. What 4b owes that harness is nothing, except that `the_exempt_set_matches_the_current_blocked_rows` fails if a listed row starts passing, so an accidental improvement shows as a red test.

- [ ] **Step 1: Reproduce the two measurements above before changing anything**

Run `cargo run -p rexx-extract --bin rexx-extract-assertions -- --suite ../ootest/ooRexx/base/keyword` and watch it panic on `ASSIGNMENT`, and count the spellings yourself. You are inheriting numbers, and a number in a brief gets propagated rather than checked. If any of them disagree with what you measure, **stop and report the disagreement** -- do not quietly use yours.

- [ ] **Step 2: Write the four tests first -- conservation, absolute literals, a real floor, revision pinning**

Write the absolute-literal test with the numbers from Step 1 and watch **that** one fail. The conservation test passes at zero and cannot be the red.

- [ ] **Step 3: Write the extractor**

Whole-body, per the model above. Route `assertSameList` to the inert-assertion arm at `lib.rs:379` rather than letting the `assertsame` prefix test claim it, and fix the same prefix hazard in whatever detector the new extractor uses.

- [ ] **Step 4: Commit the row table, the scanned file list, and the `EXEMPT` list**

- [ ] **Step 5: Run it in report mode and record the pass rate**

Report mode first. A strict gate on a table nobody has looked at turns a measurement into a blocker. Record the per-group pass rate in `l1-coverage.md`, and name the groups that yield nothing so 4c and Phase 5 inherit a list rather than a number.

- [ ] **Step 6: Add a `REXX_KEYWORD_GATE=1` strict mode matching the existing convention, commit**

The convention is `corpus.rs`'s `REXX_CORPUS_GATE` and `assertions.rs`'s `REXX_ASSERTIONS_GATE`: a `GATE_ENV` constant, set-and-non-`"0"` to enable, report mode otherwise.

---

