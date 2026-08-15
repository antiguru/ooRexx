### Task 4: `POS` bounds a match differently from the oracle

**Files:**
- Modify: `rust/crates/rexx-exec/src/builtin/string.rs` (`find_forward`)
- Do NOT change `find_backward` without establishing its own rule from the oracle first
- Test: `rust/crates/rexx-exec/tests/`, both engines

**Interfaces:**
- Independent of Tasks 1 through 3.

**The divergence, found 2026-08-14 by a search-primitive sweep and present in every earlier build tested:**

```rexx
say pos('an', 'banana bandana abracadabra', 6, 4)
```

Oracle answers **9**. This crate answers **0**. The four-argument form's window is positions 6 through 9; the match beginning at 9 runs into position 10, outside the window. **The oracle bounds where a match may begin; `find_forward` requires the whole match to fit.**

**`find_backward` carries the opposite rule for `LASTPOS` in its own doc comment**, measured against the oracle when it landed: "the match has to end within the window, not merely begin there". So the two builtins genuinely differ, and only one of them is right. **Do not assume a fix for `POS` applies to `LASTPOS`.**

- [ ] **Step 1: establish both rules from the oracle, not from either doc comment.**

Sweep `POS` and `LASTPOS` over a haystack that repeats its needle, across start positions and window lengths that put a match astride each window boundary -- beginning inside and ending outside, beginning outside and ending inside, exactly fitting, and one byte too long. Include the empty needle and a needle longer than the window. Write both tables into your report.

The existing search probe is a starting point but was written before this was known; extend it rather than trusting its coverage.

- [ ] **Step 2: write the failing test from the captured table**, run it, confirm it fails on the rows the sweep says diverge.

- [ ] **Step 3: fix `find_forward`.**

- [ ] **Step 4: decide `find_backward` from Step 1's table**, and say in the report which way it went and why. If its doc comment is right, leave the code and say so; if the comment is wrong, correct the comment as well as the code. **A comment that states something false must be corrected or removed, not hedged.**

- [ ] **Step 5: run the full search sweep against the oracle** and confirm every row agrees, including the rows that already agreed.

- [ ] **Step 6: the mutation witness**, then gates, corpus sweep and commit, as Task 1 Steps 7 and 8.
