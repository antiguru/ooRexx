### Task 1: Boundary infrastructure and the four attribution fixes

**Files:**
* Create: `crates/rexx-exec/tests/builtin_status.rs`, `rust/corpus/builtin-status.txt`, `rust/corpus/builtin-probes.txt`
* Modify: `crates/rexx-exec/tests/trace_oracle.rs`, `docs/superpowers/plans/phase-4-exclusions.txt`, `docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md`

**Interfaces produced:** `corpus/builtin-status.txt`, one row per name in `rexx_inventory::builtins::NAMES` order, `NAME<TAB>STATUS` with `STATUS` in `implemented` / `loud` / `divergent` / `excluded`.
Every later task's obligation to this file is to re-run the harness and commit the rows it flips.

**Why first.** Tasks 2-14 each move the boundary. This builds the one place that records where it sits, and fixes the four places that record it wrongly.

- [ ] **Step 1: Write `corpus/builtin-probes.txt`**

One line per **in-scope** builtin: the name, a tab, and a **meaningful** one-line Rexx program that calls it and prints a non-empty result -- `say substr('abcdef',2,3)`, not `say substr()`.

**Why meaningful and not zero-argument.** A zero-argument probe compares two error messages.
That is a real comparison but it tests the 40.x family rather than the builtin, and it is satisfied by a builtin that exists and computes nothing.
The gate criterion this file underwrites is "assert a *value* per builtin, captured from the oracle", and a value is what this file must elicit.

66 rows. The 15 whole exclusions get no probe.

- [ ] **Step 2: Write the differential status harness**

For each name in `NAMES`:

1. **If the name is one of the 15 whole exclusions, classify `excluded` and run nothing.**
   Derive the 15 as `EXCLUDED_BUILTINS` minus the three partial rows (`VALUE`, `ADDRESS`, `QUEUED`), which are **in scope and must be probed**.
   Writing "in `EXCLUDED_BUILTINS`" gives 18 and makes the count assertions in Step 3 fail on a correct implementation.
2. Otherwise run its probe through **both** `build/bin/rexx` and `run_program`, comparing stdout, stderr and exit status as three descriptors.
   * ours exits `NOT_IMPLEMENTED_EXIT` -> `loud`
   * all three match the oracle -> `implemented`
   * anything else -> `divergent`

**`EXCLUDED_BUILTINS` is a private `const` in `tests/coverage.rs` and cannot be reached from another test binary or from `src/`.**
Move it to `crates/rexx-inventory/src/lib.rs` as `pub const EXCLUDED: &[&str]` alongside `NAMES`, and have `coverage.rs` read it from there.
That is this task's one production-side edit and it is what makes the list shareable with Task 2's dispatch.

**Reuse `corpus.rs`'s oracle machinery** -- `oracle_root()`, the `ulimit` wrapper, the three-descriptor comparison -- rather than writing a second copy.
It already fails hard when the oracle binary is absent, which is the guard that stops a vacuous "0 of 0".

- [ ] **Step 3: Assert, including that the harness ran**

Four assertions, and the third is the one that stops a classifier that consults only name tables:

1. The derived set equals the committed file, **in both directions**, with messages that say which way it went.
2. `excluded` is exactly 15 and the total is 81, so `implemented + loud + divergent == 66`.
3. **The oracle was invoked exactly 66 times.** Count the invocations and assert the count.
   Without this, a classifier of the form `if EXCLUDED.contains(n) {excluded} else if DISPATCHED.contains(n) {implemented} else {loud}` satisfies every other check while running no program at all.
4. `divergent` is empty unless the row is committed as such.
   A divergent builtin is a defect, not a status; committing one requires a `KNOWN GAP` row naming it.

- [ ] **Step 4: Falsify it three ways, two of them against the interpreter**

The first revision falsified only by editing the committed file, which cannot detect a classifier that never runs anything.

1. Delete a row from `corpus/builtin-status.txt`; the test must fail **by name**.
2. Hand-edit a row from `loud` to `implemented`; the other direction must fail.
3. **Mutate the interpreter, not the data:** at Task 2's completion this step is re-run by deleting `LENGTH`'s dispatch arm and confirming its row flips `implemented` -> `loud` on its own.
   Record in this task that Step 4.3 is owed by Task 2, because it cannot run before a builtin exists.

- [ ] **Step 5: Fix `+++`'s owner (D-P)**

In `tests/trace_oracle.rs`, change `("+++", Coverage::Owned("4c"))` at `:529` to `Coverage::Owned("Phase 7")`.
`OWNER_PHASES` already admits `"Phase 7"`.
**`WITNESSED_PREFIX_COUNT` and `OUT_OF_SCOPE_PREFIX_COUNT` at `:551` and `:555` are not touched here** -- the prefix moves owner, not witnessed-ness.

In `phase-4-exclusions.txt`, correct the paragraph at `:84`, which currently reads "Four of the six -- +++ and >.> (4c)". The corrected statement, with its evidence:

> `+++` is Phase 7's. A `+++`-prefixed line has four producers in the C++:
> `RexxActivation.cpp:4468`, a command's non-zero `RC`, measured live as
> `+++   "RC(3)"` after `address sh` and `'exit 3'`; `:4024`
> (`traceSourceString`, guarded by `inDebug()` at `:4305`), the first banner
> line; `:4237`, the debug prompt, the second; and `Activity.cpp:1496`,
> `debug_error`. Three are interactive debug and one is command dispatch,
> both Phase 7's under D18 and under the `TRACE ?` row below. The reason is
> that split rather than the absence of any path: `AddressInstruction.cpp
> :163` is a 4c instruction and one of `command()`'s two callers, so it is
> `ADDRESS`'s own 4c/Phase 7 division that keeps the prefix out of reach.

- [ ] **Step 6: Give the `TRACE ?` row an owner and correct two of its claims (D-P)**

Replace "Owner unassigned" at `:1009` with `Owner: Phase 7, with the rest of interactive debug.`, and add:

* the reason -- measured, **with non-empty stdin `trace ?r` drains stdin and issues each line as a shell command**, so a following `PULL` reads `""`; reproducing the banner alone would be byte-exact at `/dev/null` and wrong on stdout for every `PULL` program;
* that the row's "only stderr differs" holds **only** at `/dev/null`;
* that the same path is reached by **`RXTRACE=ON`** with no `TRACE` instruction in the program.

**This row and Step 5's constant land in the same commit.** The assertion is what stops the row drifting a third time.

- [ ] **Step 7: Amend the design spec and correct the `::routine` citations (D-R)**

In `2026-07-30-phase-4a-executor-design.md:71`, change "every directive" to "every directive except `::ROUTINE`, which is 4c's (see the 4c plan's D-R)".

In `phase-4-exclusions.txt`, correct the `>I>`/`<I<` row so the ownership claim cites `:99-100` rather than `:124` (`:124` is about the trace gate), and add one line to `:540`'s `QualifiedCall` row noting that its "every directive" citation now carries the `::ROUTINE` carve-out and that `QualifiedCall` is unaffected because namespaces come from `::REQUIRES`.

- [ ] **Step 8: Add the D4 reason sentences**

One sentence per excluded row in `phase-4-exclusions.txt` saying why it is blocked, with the three non-obvious ones (`USERID`, `SETLOCAL`, `ENDLOCAL`) as D4 states them.

- [ ] **Step 9: Verify and commit**

Run the shared verify block. Stage exactly the paths this task names.

---

