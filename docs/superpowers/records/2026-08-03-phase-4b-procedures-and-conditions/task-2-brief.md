### Task 2: The error report carries a stack of sites, and the clause echo saturates at 40

**Files:**
- Modify: `rust/crates/rexx-exec/src/error.rs` (`Raised` and `Raised::report`)
- Modify: `rust/crates/rexx-exec/src/run.rs` (`record_failure_site` at **`:1387`**, its callers at **`:882`**, **`:904`** and **`:1357`**, and the `*-*` formatting site)
- Modify: `rust/crates/rexx-exec/src/lib.rs` if `failure_site` lives there
- Create: a corpus program nesting past depth 20
- Modify: `docs/superpowers/plans/phase-4-exclusions.txt`

**Interfaces:**
- Produces: `Raised` carries a stack of sites, innermost first, each entry carrying its own line number and its own **absolute printed indent**.
- Consumes: `static_indent` from `run.rs`. Its signature does not change and the clamp does not go inside it.
- Consumes: **`Interp::indent_offset` (`src/lib.rs:888`), which already exists.** 4a's F-EX1 work added it and the call sites already read `static_indent(...) + self.indent_offset`. `run.rs:1045` carries a "**Do not clear `indent_offset` here**" note with its reasoning. The activation base this task adds is the same quantity, so extend that mechanism rather than introducing a parallel one, and read its doc comment before touching it.

**Why:** every raise inside a routine differs from the oracle on stderr until this exists, which is most of what a 4b differential corpus contains. Building it after `CALL` means every corpus program written in between is unverifiable.

**Read D2r above in full before starting.** It carries the measured indent rule, and the first revision of this plan stated that rule wrongly. In particular: the base is the **calling clause's printed indent**, not two times the depth, and it cannot be derived from the depth counter.

**The 40-column cap is a pre-existing 4a divergence and this task closes it.** Measured with nested `DO`s and no calls: the oracle caps the `*-*` echo at 40 columns from depth 20 onward; our binary is uncapped and prints 50 at depth 25. The cap applies to `*-*` **only** -- at depth 25 under `trace r`, `>>>` value lines run to 52. Clamp at the `*-*` formatting site.

**Inherited items this task pays for:**

* **I12.** The KNOWN GAP row at `phase-4-exclusions.txt` records only "one echo per nesting level, innermost first". Amend it to carry the measured rule; the `INTERPRET` half closes here and the `CALL` half in Task 3.
* **I11.** `Interp::failure_site` is set first-call-wins, and its guard `self.failure_site.is_none()` is documented at `run.rs:1295-1298`. It matters only once a trap can resume after a raise, which is Task 7. **Do not fix it here and do not remove the guard.** Leave a comment on the new stack naming Task 7 as the owner of the clearing.

- [ ] **Step 1: Capture the oracle expectations you will assert against**

At minimum: the two-level `INTERPRET` case; a raise inside a `DO` inside a called routine; a call nested two `DO`s deep whose callee is flat (the probe that discriminates the indent rule); and a 25-deep `DO` nest with no call at all (the cap). Commit them in the shape `tests/trace_oracle.rs` uses -- its module doc carries the regeneration command.

- [ ] **Step 2: Run the new expectations and watch the right ones fail**

The cap expectation fails today. Say so in the report: it is a 4a defect this task closes, not a regression this task introduced.

- [ ] **Step 3: Change `Raised` to carry a stack**

Each entry carries the line to print and its **absolute** printed indent. `Raised::report` walks innermost-first. The existing single-site behaviour must fall out as the one-element case byte-identically -- 4a byte-verified the report on eleven programs and all eleven must still pass.

Do **not** resolve the stack at report time by walking `Interp::activations`. `run` pops the activation before `execute` sees the error, which is why `failure_site` exists at all.

- [ ] **Step 4: Push a fragment entry at `INTERPRET` entry, with delta 0**

Measured: the fragment shares its caller's indent and carries the enclosing clause's line.

**Task 1 tried the obvious version of this and proved it wrong, and an independent reviewer reproduced the failure by building it.** Passing `Some(&fragment.source)` into the fragment's execution does **not** supplement the enclosing echo -- it *replaces* it, and it moves the reported error line off the one the oracle names. Measured: on a raise inside fragment text the oracle reports line 3 and the naive fix reports line 1. Two independent causes, and both must be handled here:

* The fragment's own source carries the fragment's line numbering, where the oracle prints the **enclosing `INTERPRET` clause's** line for both echoes.
* `record_failure_site` is **first-wins** (`self.failure_site.is_none()` guards both callers). A fragment entry that records first therefore takes the report off the enclosing clause. The site stack is what lets both exist without racing.

So this step is not "pass the source down". It is the reason the stack exists. Do not reach for the one-line version -- it has already been built twice and it fails both ways.

The missing clause echo for a traced `INTERPRET` is the other half of a divergence Task 1 half-fixed: Task 1 landed the `>>>` on the interpreted text, and this step lands the `*-*` echo. Under `trace r` with `x='nop'` and `interpret x`, the oracle prints `>>> "nop"` then `3 *-* nop`.

- [ ] **Step 5: Clamp the `*-*` echo at 40 columns, and only the `*-*` echo**

**There are two `*-*` formatters, not one, and the tree already says they are one quantity.** `Raised::report` (`src/error.rs:272`) writes the error report's echo; `push_clause` (`src/trace.rs:231`) writes the trace echo. `push_clause`'s own doc calls this "one quantity with two formatters, not two quantities, and this is the second formatter D17's own retrofit note names". So the clamp is one rule applied at both, via a shared helper or a shared constant -- not a number typed twice.

Measured, the cap is on the **total** printed indent and is reached at nesting depth 20: depth 18 gives 36, depth 19 gives 38, depth 20 gives 40, depths 21 and 25 give 40. Value lines are **not** capped: at depth 25 under `trace r`, `*-*` tops out at 40 while `>>>` runs to 52. A clamp inside `static_indent`, or one applied to `push_indent` generally, is therefore wrong in both directions.

- [ ] **Step 5b: Raise the oracle's condition when a fragment fails to parse**

Found by Task 1 and measured there: a fragment whose text does not parse raises **27.901 at rc 229** on the oracle, and we exit 120 with a loud not-implemented message instead. Like the echo stack, this was unreachable before Task 1 gave `run_program` a real `INTERPRET`, so it is a newly reachable divergence rather than a regression.

The fix is a `ParseError`-to-`Raised` conversion. **A top-level syntax error wants the same conversion**, so build it once and check whether the top-level path can use it -- if it cannot, say why in the report rather than duplicating the mapping.

The starting point exists: `Loud::parse(error: &ParseError)` at **`src/lib.rs:491`**, and the doc comment immediately above it at **`:488`** already names this gap and says closing it needs exactly this conversion. Read that comment first -- it was written by the person who chose to defer it, and it records why.

- [ ] **Step 6: Run the full suite and the corpus gate**

Run: `cargo test -p rexx-exec`, then `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus`

- [ ] **Step 7: Add the deep-nesting program to the 4a corpus subset**

It has no 4b construct in it -- 25 nested `DO`s around a failing clause -- so it belongs in `phase-4a.txt` rather than `phase-4b.txt`. It pins a rule 4a should always have had.

**This is an authorised plan amendment, and it takes two edits, not one.** `phase_4a_subset_matches_the_committed_list` (`tests/coverage.rs:514`) asserts the file against the `EXPECTED_SUBSET` literal (`:481`), and its message says adding or removing a line "is a plan amendment, and must change both the file and this list together, so a line cannot be silently dropped or silently added". Change both in the same commit. Do not widen or delete the assertion.

Expect the corpus figure to move from `30 of 30` to `31 of 31`. Confirm the new program is genuinely **compared** and not merely counted: append a program you know diverges, check the report names it, then revert. Task 1 established that pattern after finding that a count moving by one does not by itself prove the new slot was exercised.

- [ ] **Step 8: Amend the KNOWN GAP row, add a DEVIATIONS or fixed-defect note for the cap, commit**

---

