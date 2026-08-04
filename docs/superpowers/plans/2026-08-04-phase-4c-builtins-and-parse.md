# Phase 4c implementation plan: builtins, PARSE, and the rest of the call chain

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** finish classic Rexx.
After 4c, a program that calls builtins, parses strings, reads the queue, tracks an `ADDRESS` environment and calls a `::routine` runs byte-for-byte as `build/bin/rexx` runs it, and Phase 4's parent row closes.

**Architecture:** 4b made the activation stack real.
4c hangs the builtin table off the one call-resolution chokepoint 4b built (`resolve_and_run_call`, `run.rs:3218`), adds a `PARSE` template engine, and closes the call chain's last step with `::routine` dispatch.
The 66 builtins live in a new `builtin/` module tree, one file per family, so no two tasks touch one file.
No new crate; `rexx-exec` grows.

**Tech stack:** Rust 1.96.1, no `unsafe`, `cargo fmt` default, `clippy -D warnings`.
Depends on `rexx-core`, `rexx-num`, `rexx-parse`, `rexx-inventory`.

## The governing documents, and what each is for

* **`docs/superpowers/plans/phase-4-exclusions.txt`** is the live record of what Phase 4 does not do.
  Adding a `KNOWN GAP` row needs no permission; removing one does.
  **Three of its rows are corrected by this plan's Task 1** and must not be read as authority until then: `+++`'s owner, `>I>`/`<I<`'s justification, and the `TRACE ?` row's missing owner.
* **`docs/superpowers/plans/phase-4b-gate.md`** is the criterion set 4c's gate derives from, under D14.
* **`docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md`** still governs the value model, the borrow shape, and D15 to D19.
  Read the section a task names.
  **Its line 71 is amended by this plan** -- see D-R below.
* **`docs/superpowers/specs/2026-08-01-phase-4bc-scoping.md`** is groundwork, not requirements.
  **Do not read it to find your requirements.**
  Its `trace r` probe for `>.>` could not have seen the prefix, and three of its inherited items are superseded here.

---

## Global constraints

Every task's requirements implicitly include this section.
**It is not extracted into task briefs, so a task that depends on one of these lines restates it.**

* **The C++ tree is the oracle and is never modified.** `interpreter/`, `samples/`, `build/`, `ootest/` are read-only.
* **Wrap every oracle invocation** as `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`.
  Without the ulimit the interpreter requests gigabytes mid-range and is OOM-killed, which has already cost a session and the machine's memory.
* **Read stdout, stderr and exit status as separate descriptors.** Comparing `2>&1` as one string produced two false regressions in 4a.
  Read exit status unpiped: a shell pipeline reports the last command's status.
* **Never set `NUMERIC DIGITS` above 1000** in a probe.
  This binds 4c harder than it bound 4b: `FORMAT`, `TRUNC`, `D2X` and `X2D` all take a digits-shaped argument, and the natural "try a big one" probe is the one that kills the machine.
* **Never instantiate `.Package~new`** on a file inside the repository: it executes that file's prolog and has written untracked files into the tree.
* **Never probe `select; when 1 = 0 then; when 2 = 2 then nop; end`.** It segfaults the oracle (upstream SF #2018).
* **Run every oracle probe from a fresh empty subdirectory of the scratchpad, not the scratchpad root**, using absolute paths.
  The scratchpad is on the oracle's **external-routine search path** and holds hundreds of stale `.rex` files.
  Measured: `say "f"(1)` with an internal `f:` reports 44.1 rc 212 in the root and 43.1 rc 213 in a clean directory -- different error, different rc, different meaning.
  **This bites 4c hardest of any sub-phase**, because a builtin name that is not yet implemented falls through to exactly that external search.
* **Beware Rexx literal syntax in probes.** A symbol named `x` or `b` immediately followed by a quoted string parses as a hex or binary literal, so `say '['x']'` is error 15.3.
  Use other names.
  **`b2x`, `x2b`, `x2c`, `c2x`, `d2x`, `x2d` probes are the live hazard here**: `say x2d('ff')` is fine, but `y = 'ff'; say x2d(y)` and `say x'41'` are different programs.
* **No `unsafe`.** The workspace sets `unsafe_code = "forbid"`.
  If a task appears to need it, stop and report BLOCKED.
  **`SETLOCAL`/`ENDLOCAL` are excluded partly for this reason** (`std::env::set_var` is `unsafe` in edition 2024, verified by compiling it); do not route around it.
* **Never `git add -A`.** Stage the exact paths the task names.
  Do not run `git reset --hard`, do not force-push.
* Comments state the contract at the top and the reasoning at the decision point.
  Never delete a true comment to make a change easier.
  Prefer `--` over an em-dash.
  The "no structuring semicolons" rule does **not** apply to this repository.
* **A comment may say what the oracle does and what was measured. It may not state where the implemented/not-implemented boundary sits.**
  "Owned by 4c", "not yet implemented", "the twelve builtins this module has so far" are all facts about a boundary that moves every task.
  **Assert the boundary or do not write it down** -- Task 1 builds the mechanism that makes asserting it free.
* A value's rendering is fixed when the value is created.
  Any code that formats a number with `settings.digits()` instead of the value's own captured pair is wrong; see D15.
* Anything Phase 4c does not implement **fails loudly**: `NOT_IMPLEMENTED_EXIT` (120), outside `157..=253` where `256 - major` lives.
  Never a plausible Rexx condition.
* **Every new allocation site goes through `Interp::alloc_with`**, the wrapper around `Heap::alloc_with_uncollected`.
  **4c adds more allocation sites than 4a and 4b combined** -- every builtin returning a string allocates -- so this is the constraint most likely to be broken by volume rather than by misunderstanding.
* **Commit first, then read the hash back, then record it.**
* **Check formatting with `cargo fmt --all --check`, from `rust/`.**
  Not `cargo fmt --edition 2024 --check`: `cargo fmt` has no `--edition` flag and that spelling exits 2 without doing any work.
* **Run clippy from a clean target directory at each task's end, not just `cargo clippy`.**
  Measured at the pre-4c boundary: the identical command reported exit 0 across a whole session at two commits that fail it on a cold target.
  A same-session green is provisional.

---

## Decisions taken for 4c

Five decisions were deferred by 4b's plan and are settled here with measurements.
Two more (D-R, D-P) are new, and both correct an attribution the tree currently machine-asserts.

### D4 -- the fifteen excluded builtins stay excluded, and `QUALIFY` with them

**Measured 2026-08-04.** `grep -E '\bqualify *\(' ootest/ooRexx/base/bif/*` returns hits in **`QUALIFY.testGroup` only** -- zero calls from any other group.
The scoping document's stated condition for pulling `QUALIFY` in ("needed by an L1 `base/bif` group 4c must pass") is therefore measured **false**, and option A stands unchanged: `EXCLUDED_BUILTINS` keeps its 18 rows, `in_scope` stays 66.

**What Task 1 owes this decision:** the exclusions file gains **one sentence per excluded row saying why it is blocked**, which the scoping document recommended regardless of the outcome.
Three rows need a reason that is not "the platform layer":

* `USERID` -- Rust `std` has no username API; `getpwuid` needs libc and the workspace forbids `unsafe`.
  Its value is host-dependent, so a differential run would be single-machine.
* `SETLOCAL` / `ENDLOCAL` -- they save and restore the process environment, and `std::env::set_var` is `unsafe` in edition 2024 (verified: `error[E0133]`).
  Phase 7 needs either a shadow environment or a lint exception, and **that is a decision, not a task**.

### D11 -- no `RANDOM`, `DATE` or `TIME` in any differential corpus program

**Measured 2026-08-04, and the result reverses the reason the question was asked.**
`RANDOM.testGroup` asserts only *properties*, never a generator output:

* 100 reps per range checked for `mi <= x <= ma`, then `self~assertTrue(.true)`.
* A seeded sequence generated twice **inside one process** and compared to itself.
* Three degenerate ranges where the answer is forced regardless of the PRNG: `random(1,1)` is 1, `random(0)` is 0, `random(,0)` is 0.

So **reproducing ooRexx's PRNG and default seed is not forced**, and the scoping document's changed-by condition ("if `base/bif` pins `RANDOM(1,100,42) = 89`") is measured false.
Option A stands: these three are excluded from the corpus by rule and pinned by unit tests against properties.

**But the seeded-reproducibility body is a real requirement and it is easy to miss.**
`random(mi, ma, se)` followed by 99 unseeded calls must produce the identical sequence when the same seed is re-supplied.
That constrains *our* generator to be seedable and deterministic; it does not constrain it to match the oracle's.
**The corpus rule goes in `corpus/README.md` next to the `DO OVER` rule**, because that is where the next author looks.

### D12 -- `base/bif` reuses Task 11's whole-body extractor; no third extractor is written

**Measured 2026-08-04 across `ootest/ooRexx/base/bif`'s 78 files and 31,162 lines:**

| assertion method | calls | files |
|---|---|---|
| `assertSame` | 5,441 | 66 |
| `expectSyntax` | 1,021 | 55 |
| `assertTrue` | 186 | 17 |
| `assertEquals` | 106 | 18 |
| `assertFalse` | 51 | 8 |

The shape is `assertSame`-dominated exactly as `base/keyword` was, so `rexx-extract`'s `keyword.rs` whole-body extractor is the right instrument and a third extractor is not written.
**Reuse over writing, and the conservation invariant comes with it**: `rows + dropped == calls`, the property that caught the Phase 0 defect, is already in `keyword.rs` and must be preserved rather than re-derived.

**The size, stated like for like, because the two obvious comparisons are not comparable.**
`base/bif` has **4,420 `::method` bodies in 78 files**; `base/keyword` has **2,105 in 39**.
`base/keyword`'s **896** is a different quantity -- bodies the extractor *emitted*, a 43% yield on its methods -- so 4,420 against 896 compares an input to an output.
At `base/keyword`'s yield, `base/bif` would emit roughly 1,900 bodies; **that is an extrapolation and Task 15 measures the real number.**

**The case trap that cost `base/keyword` 510 rows does not exist here, and this is measured rather than assumed.**
`base/keyword` contains **510 `AssertSame`** with a capital `A` alongside 1,931 lowercase, which is how its true total reaches 2,441 -- a case-sensitive count silently drops a fifth of the table.
`base/bif` contains **5,441 `self~assertSame` and zero capital-`A` occurrences**.
Match the token case-insensitively anyway: it is correct in both groups, and it costs nothing here.
Separately, `assertSameList` appears **5** times and is a different method -- match the token, never the prefix.

**Two things that will not transfer, and Task 14 measures them rather than assuming:**

* `expectSyntax` at 1,021 calls is a fifth of the volume, and in `base/expressions` an `expectSyntax` marker changed what a *later* `assertSame` meant.
  Whether that coupling exists here is a measurement, not an inheritance.
* `assertTrue`/`assertEquals`/`assertFalse` total 343 calls in 25 files.
  Extracting `assertSame` alone drops them.
  **That is acceptable and must be stated in the header** -- a dropped call is counted by the conservation invariant, so the number is visible rather than silent.

### D8 -- `rexxcps.rex` is a run-to-completion smoke test; the enumerated dependency list is the gate

Read in full 2026-08-01, and it is not byte-comparable for two independent reasons: it prints wall-clock timings, and **its loop count is auto-adjusted from measured elapsed time** (`count=(1%total + 1) * count`, repeated until `total>1`), so the number of output lines and the control flow depend on host speed.

Options A and C together: keep it as a smoke test asserting **rc 0 and completion**, and make the real gate the dependency list, each item with its own differential witness.
Its dependencies are: `parse var`, `parse version`, `parse value`, `parse upper`, `parse source`, `trace value`, `trace off`, `signal on novalue`, one internal `call subroutine`, the `call time 'R'` call-to-builtin **instruction** form, `address value` with `ADDRESS()`, and eight builtins -- `TIME`, `SUBSTR`, `FORMAT`, `WORD`, `TRACE`, `LENGTH`, `LEFT`, `ADDRESS`.

### D7 -- closed in 4b, recorded here so it is not reopened

`ExprKind::List` is Phase 5's; the three `num/` corpus programs return in Phase 5.
Both halves of the two-file contradiction were fixed during 4b: `corpus/phase-4a.txt:18` and `corpus/README.md:119` now say Phase 5.
**Nothing in 4c touches this.**

### D-R (new) -- `::routine` is 4c's, and the design spec's line 71 is amended

**The tree contradicts itself and this decision resolves it.**
`2026-07-30-phase-4a-executor-design.md:71` says "**every directive** ... [is] Phase 5's", and `phase-4-exclusions.txt:540` leans on that sentence to place `QualifiedCall` in Phase 5.
But `phase-4-exclusions.txt:88` says `::routine` is 4c's ("which 4c will have to meet"), and `trace_oracle.rs:542,546` **machine-assert** `Coverage::Owned("4c")` for `>I>` and `<I<`.
`::routine` is a directive. Both cannot hold.

**Ruling: 4c owns `::routine` dispatch.** The spec's line 71 is amended to read "every directive except `::ROUTINE`".
Three reasons, in order of weight:

1. **The reason 4b deferred it is discharged exactly here.** The exclusions row says 4b declined "because getting builtin-colliding names right needs 4c's table".
   4c is where that table lands.
2. **It is one mechanism, and splitting it strands `eval_call` across a phase boundary.** Rexx resolves internal label, then builtin, then `::routine`/external.
   4b built step 1; 4c builds step 2; leaving step 3 to Phase 5 means `resolve_and_run_call` has a hole in the middle of one chain for a whole phase, which is the boundary skew this plan exists to reduce.
3. **Otherwise Phase 4 closes with a construct the oracle runs at rc 0 still failing loudly.** Measured: `call zorkolo` with a `::routine zorkolo` present runs on the oracle.

**Three measured facts a `::routine` activation must satisfy, which an internal label's does not:**

* **It has its own variable pool.** Measured: a caller with `nn = 5` calling a `::routine` that says `nn` prints the derived name `NN`, not 5.
  This is *not* 4b's `PROCEDURE` isolation reused -- a `::routine` has a different `CodeBody`, so it also has a different `Plan`, and the slot-index-identity assumption `PROCEDURE EXPOSE` rests on does **not** hold.
* **Builtins shadow it.** Measured: `call max 1, 9` with a `::routine max` present sets `RESULT` to 9 and the routine never runs.
* **Trace does not cross into it.** Measured: a caller's `trace r` echoes its own clauses and none of the routine's.
  An internal label's clauses *are* echoed under the same setting.

**`BodyKey::directive` is the field this finally uses.** It is `Option<usize>` at `plan.rs:79` and is set to `None` at its one construction site (`plan.rs:631`); I2 predicted 4b would be the first to need `Some(index)` and 4b never did.
4c is.

**`::method` does not travel with it.** The C++ gate is one predicate (`isMethodOrRoutine`), but `::method` needs the object model and stays Phase 5's.

### D-P (new) -- `+++` moves to Phase 7, and the `TRACE ?` row gets an owner

**`trace_oracle.rs:529` asserts `("+++", Coverage::Owned("4c"))` and that is wrong.**
`TRACE_PREFIX_ERROR` has exactly two emission sites in the C++, read directly rather than inferred:

* `RexxActivation.cpp:4468` -- a command's non-zero return code, formatted `RC(n)`.
  Measured live: `address sh` then `'exit 3'` under `trace r` prints `+++   "RC(3)"`.
  **Command dispatch is Phase 7's under D18.**
* `RexxActivation.cpp:4024` -- `traceSourceString()`, whose only caller is guarded by `if (inDebug() && !settings.wasSourceTraced())` at `:4305`.
  **Interactive debug**, reached through `TRACE ?`.

Neither is a 4c construct.
**`+++` becomes `Coverage::Owned("Phase 7")`.**

**And the second site is a gap that has drifted unowned since 4a, which this decision closes rather than passes on.**
`phase-4-exclusions.txt:989-1011` records that `TRACE ?` is silently ignored -- `mode_from_setting` accepts the letter and drops the `?` -- while the oracle, measured with stdin at `/dev/null`, emits two stderr lines this crate does not:

```
+++ "LINUX COMMAND <absolute path>"
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
```

That row ends "**Owner unassigned**". It is the same shape as the compound-`DO` gap the 4b gate ruled on, and it gets the same treatment.

**Ruling: `TRACE ?` is assigned to Phase 7, alongside the rest of interactive debug and the other `+++` site.**
Not 4c, and the reason is that the cheap half is the dishonest half: reproducing two banner lines without the debug pause claims interactivity the crate cannot deliver, and `TRACE ?` genuinely changes control flow on a tty.
An owner naming the subsystem that will build the pause is the honest record.
**Task 1 writes the owner into the row and flips the `Coverage` constant, and those two edits must land together** -- the assertion is what stops the row drifting again.

**Consequence for the gate, stated now rather than discovered:** 4c's trace-prefix target is **16 of 19**, not 17.
The three 4c adds are `>.>`, `>I>` and `<I<`.
`+++`, `>M>` and `>N>` remain, owned by Phase 7, Phase 5 and Phase 5.

### D6, D3, D5, D13, D14 -- carried from 4b unchanged

* **D6.** `rust/corpus/phase-4c.txt` is created beside the other two; the harnesses read the union of all three.
* **D3.** No corpus program may contain `DO OVER` on a stem.
  **Measured and still true for 4c: no in-scope builtin exposes stem traversal order**, so `DO OVER` remains the only exposure through the end of Phase 4.
* **D5.** One lane. **Never dispatch two implementers in parallel against this plan.**
  Every scheduling collision in the 4a ledger was two agents in one file.
  This plan's file structure is designed so that constraint costs nothing.
* **D13.** 4c gets its own gate document.
* **D14.** The criterion set carries forward with its amendments.

---

## The anti-skew design, and why this plan is sliced the way it is

4b spent a large share of its commits correcting prose about where the implemented/not-implemented boundary sat.
The diagnosis, measured: a **796-row derived file** that computes those facts at runtime and polices them in both directions needed **zero** corrections across thirteen tasks, while a **one-line count comment** stating the same kind of fact rotted **four times**.
Same data, different medium, opposite outcome.

**4c moves that boundary 66 times.** Every builtin that lands changes which names resolve, and unblocks some of `keyword-exempt.txt`'s 790 `4c` rows.
Left alone, that is 4b's defect multiplied.

Three structural countermeasures, all in Task 1, all before a single builtin lands:

1. **The implemented set is derived, never listed.**
   A test enumerates `rexx_inventory::builtins::NAMES`, calls each name through the interpreter, classifies it implemented or loud, and compares the result to a committed file.
   No task body, comment, or gate row ever states how many builtins exist yet.
   This is `keyword-exempt.txt`'s mechanism, reused because it is the one thing in this project with a clean record.
2. **One file per family, and no task touches another's.**
   The 66 builtins split into seven modules under `builtin/`.
   A family task's diff is one new file plus one line in `builtin/mod.rs`'s dispatch, so review scope stays small and D5's one-lane rule costs nothing.
3. **The two wrong attributions are fixed first.**
   `+++`'s owner and the `TRACE ?` row (D-P), and the spec's directive sentence (D-R), are Task 1's, not a cleanup at the end.
   An attribution that is wrong while eleven tasks build against it is how the compound-`DO` gap survived two phases.

**What is deliberately *not* done:** the phases are not re-sliced.
The one genuine slicing artifact 4b found was an enum whose arms landed in different phases, and the fix was the *granularity of the table describing it*, not a different slice.
4c's coupling (builtins before `PARSE PULL`, `ADDRESS` before `ADDRESS()`, `::routine` after the builtin table) is forced by the language, not by the plan.

---

## File structure

**Created:**

| file | responsibility |
|---|---|
| `crates/rexx-exec/src/builtin/mod.rs` | name -> function dispatch, the arity table, the 40.x raisers, `BuiltinResult` |
| `crates/rexx-exec/src/builtin/string.rs` | 23: `ABBREV` `CENTER` `CENTRE` `CHANGESTR` `COMPARE` `COPIES` `COUNTSTR` `DELSTR` `INSERT` `LASTPOS` `LEFT` `LENGTH` `OVERLAY` `POS` `REVERSE` `RIGHT` `SPACE` `STRIP` `SUBSTR` `TRANSLATE` `VERIFY` `LOWER` `UPPER` |
| `crates/rexx-exec/src/builtin/word.rs` | 7: `DELWORD` `SUBWORD` `WORD` `WORDINDEX` `WORDLENGTH` `WORDPOS` `WORDS` |
| `crates/rexx-exec/src/builtin/convert.rs` | 12: `B2X` `BITAND` `BITOR` `BITXOR` `C2D` `C2X` `D2C` `D2X` `X2B` `X2C` `X2D` `XRANGE` |
| `crates/rexx-exec/src/builtin/numeric.rs` | 7: `ABS` `FORMAT` `MAX` `MIN` `RANDOM` `SIGN` `TRUNC` |
| `crates/rexx-exec/src/builtin/datatype.rs` | 4: `DATATYPE` `SYMBOL` `VALUE` `VAR` |
| `crates/rexx-exec/src/builtin/datetime.rs` | 2: `DATE` `TIME` |
| `crates/rexx-exec/src/builtin/state.rs` | 11: `ADDRESS` `ARG` `CONDITION` `DIGITS` `ERRORTEXT` `FORM` `FUZZ` `GC` `QUEUED` `SOURCELINE` `TRACE` |
| `crates/rexx-exec/src/parse_template.rs` | the `PARSE` template engine, source-independent |
| `crates/rexx-exec/tests/builtin_status.rs` | the derived implemented-set harness |
| `crates/rexx-extract/src/bif.rs` | `base/bif` extraction, reusing `keyword.rs`'s whole-body machinery |
| `crates/rexx-exec/tests/bif_assertions.rs` | the `base/bif` L1 harness, `REXX_BIF_GATE` |
| `rust/corpus/builtin-status.txt` | committed implemented/loud classification, derived |
| `rust/corpus/bif-exempt.txt` | committed `base/bif` exempt set, derived |
| `rust/corpus/phase-4c.txt` | 4c's differential subset |
| `rust/scripts/mutate-4c.sh` | 4c-shaped mutations |
| `docs/superpowers/plans/phase-4c-gate.md` | the gate |

23 + 7 + 12 + 7 + 4 + 2 + 11 = **66**, which is `EXCLUDED_BUILTINS`' own derived `in_scope` figure and is asserted in Task 1 rather than trusted from this table.

**Modified:** `src/run.rs` (the resolution chokepoint, `PARSE`/`ARG`/`PULL`/`ADDRESS` arms, `bind_control`), `src/eval.rs` (nothing structural -- `eval_call` already delegates), `src/lib.rs` (`instruction_owner`, the loud fallback), `src/plan.rs` (`BodyKey::directive`), `tests/owners.rs`, `tests/loud.rs`, `tests/coverage.rs`, `tests/trace_oracle.rs`, `docs/superpowers/plans/phase-4-exclusions.txt`, `docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md`.

---

## Task list

Fifteen tasks. The order is forced by the language in five places and free elsewhere.

| # | task | why here |
|---|---|---|
| 1 | Boundary infrastructure and the three attribution fixes | before anything moves the boundary |
| 2 | Builtin dispatch, arity table, the 40.x error family | every family task depends on it |
| 3 | `builtin/string.rs` (23) | free |
| 4 | `builtin/word.rs` (7) | free |
| 5 | `builtin/convert.rs` (12) | free |
| 6 | `builtin/numeric.rs` (7) | free |
| 7 | `PARSE` template engine + `VAR`/`VALUE`/`ARG`/`SOURCE`/`VERSION`, `UPPER`/`LOWER`/`CASELESS`, `>.>` | needs nothing from 2-6 |
| 8 | `ARG` instruction, `PULL`, `PARSE PULL`, `PARSE LINEIN` | needs 7's engine and 4b's queue |
| 9 | `ADDRESS` instruction and environment tracking | before `ADDRESS()` |
| 10 | `builtin/state.rs` (11) | `ADDRESS()` needs 9; `ARG()` needs 8's argument model |
| 11 | `builtin/datatype.rs` (4) | `VALUE`'s variable-access form needs the pool, which exists |
| 12 | `builtin/datetime.rs` (2) | `TIME('R')` is stateful; isolate it |
| 13 | `::routine` dispatch, `>I>`/`<I<` | needs the builtin table complete, for shadowing |
| 14 | The compound-`DO` control-variable fix | independent; scheduled late so its L1 rows move once |
| 15 | `base/bif` L1 harness, the 4c corpus subset, `mutate-4c.sh` | measures everything above |

---

### Task 1: Boundary infrastructure and the three attribution fixes

**Files:**
* Create: `crates/rexx-exec/tests/builtin_status.rs`, `rust/corpus/builtin-status.txt`
* Modify: `crates/rexx-exec/tests/trace_oracle.rs`, `docs/superpowers/plans/phase-4-exclusions.txt`, `docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md`

**Interfaces:**
* Produces: `corpus/builtin-status.txt`, one row per builtin name in `rexx_inventory::builtins::NAMES` order, format `NAME<TAB>STATUS` where `STATUS` is `implemented`, `loud`, or `excluded`.
  Every later task's only obligation to this file is to **re-run the test and commit the rows it flips**.

**Why this task is first.**
Tasks 2 through 14 each move the implemented/not-implemented boundary.
This task builds the one place that records where it sits, and fixes the two places that currently record it wrongly.

- [ ] **Step 1: Write the failing status harness**

`tests/builtin_status.rs` enumerates `rexx_inventory::builtins::NAMES` and, for each name, runs a one-line program through `run_program` that calls it with zero arguments, then classifies:

* stderr ends with `" is not implemented (4c)"` and exit is `NOT_IMPLEMENTED_EXIT` -> `loud`
* the name is in `coverage.rs`'s `EXCLUDED_BUILTINS` -> `excluded`
* anything else (including a `40.3` "not enough arguments" raise) -> `implemented`

**A zero-argument call is the right probe and the reason is not obvious.**
Most builtins raise `40.3` on it -- measured, `say substr('abc')` gives `Error 40.3: Not enough arguments in invocation of SUBSTR; minimum expected is 2`, rc 216.
That raise is *evidence the builtin exists*: an unimplemented name never gets far enough to count its arguments.
So the classifier keys on the **loud** message, which only an unimplemented name produces, and treats every other outcome as implemented.

Assert the derived set equals the committed file **in both directions**, with the two failure messages worded so a reader knows which way it went:

* a name that is `implemented` but committed as `loud` -> "`NAME` now resolves and is still committed as loud -- re-run and commit `corpus/builtin-status.txt`"
* a name that is `loud` but committed as `implemented` -> "`NAME` no longer resolves"

- [ ] **Step 2: Run it and confirm every in-scope name is currently `loud`**

Expected before any builtin lands: 66 `loud`, 15 `excluded`, 0 `implemented`.
**Assert the total is 81 and that `implemented + loud == 66`**, so the file cannot silently shrink.

- [ ] **Step 3: Falsify it**

Delete one row from `corpus/builtin-status.txt` and confirm the test fails **by name**.
Then hand-edit one row from `loud` to `implemented` and confirm the other direction fails.
Both must be observed; a set assertion checked in one direction is half a check.

- [ ] **Step 4: Fix `+++`'s owner (D-P)**

In `tests/trace_oracle.rs`, change `("+++", Coverage::Owned("4c"))` to `Coverage::Owned("Phase 7")`.
`OWNER_PHASES` already admits `"Phase 7"`.

In `phase-4-exclusions.txt`, correct the paragraph at `:84` -- it currently reads "Four of the six -- +++ and >.> (4c), >M> and >N> (Phase 5)".
The corrected statement, with its evidence:

> `+++` is Phase 7's. `TRACE_PREFIX_ERROR` has two emission sites in
> `RexxActivation.cpp`: `:4468`, a command's non-zero `RC`, formatted
> `RC(n)` and measured live as `+++   "RC(3)"` after `address sh` and
> `'exit 3'`; and `:4024`, `traceSourceString`, whose only caller is
> guarded by `inDebug()` at `:4305`. Command dispatch is Phase 7's under
> D18 and interactive debug is Phase 7's under the `TRACE ?` row below.
> Neither is a 4c construct.

- [ ] **Step 5: Give the `TRACE ?` row an owner (D-P)**

The row at `:989-1011` ends "Owner unassigned".
Replace that with `Owner: Phase 7, with the rest of interactive debug.` and one sentence of reason: reproducing the two banner lines without the debug pause would claim interactivity the crate cannot deliver, and `TRACE ?` genuinely changes control flow on a tty.

**This row and Step 4's constant must land in the same commit.**
The assertion is what stops the row drifting a third time.

- [ ] **Step 6: Amend the design spec's directive sentence (D-R)**

`docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md:71` reads "`Message`, `Guard`, `Reply`, `Forward`, every directive, and environment symbols beyond `.nil`, `.true` and `.false` are Phase 5's".
Change "every directive" to "every directive except `::ROUTINE`, which is 4c's (see the 4c plan's D-R)".

Correct `phase-4-exclusions.txt:88`'s row to cite D-R rather than leaving the contradiction with `:540` standing, and add one line to `:540`'s `QualifiedCall` row noting that its "every directive" citation now carries the `::ROUTINE` carve-out and that `QualifiedCall` is unaffected, because namespaces come from `::REQUIRES`.

- [ ] **Step 7: Verify and commit**

`cargo test --offline --workspace`, `cargo fmt --all --check`, clippy from a clean target.
Stage exactly the six paths above.

---

### Task 2: Builtin dispatch, the arity table, and the 40.x error family

**Files:**
* Create: `crates/rexx-exec/src/builtin/mod.rs`, `crates/rexx-exec/src/builtin/string.rs` (holding `LENGTH` alone, see Step 3)
* Modify: `crates/rexx-exec/src/run.rs` (`resolve_and_run_call`), `crates/rexx-exec/src/lib.rs`, `crates/rexx-exec/src/error.rs`

**Interfaces:**
* Produces:
  * `pub(crate) fn dispatch(interp: &mut Interp, name: &[u8], args: &[Option<ObjRef>]) -> Option<Result<ObjRef, Failure>>` -- `None` means "not a builtin name", which is what lets resolution fall through to `::routine` in Task 13.
  * `Raised::bad_argument_count`, `Raised::bad_argument_type` -- the 40.x raisers.
* Consumes: `resolve_and_run_call`'s existing argument evaluation, unchanged.
  It already produces `Vec<Option<Argument>>`, where `Argument` is `lib.rs:1340`'s private two-variant enum.
  **Pass `Argument::value()`, not the `Argument`**: that method exists precisely for this and its own doc names `ARG()` as a caller, the `Reference` variant's alias data is `USE ARG >`'s business, and no builtin takes a variable reference.
  An omitted position stays `None` rather than being closed up -- the same rule 4b established for `call sub 1,,3`.

**Where it hooks in, exactly.**
`run.rs:3218` currently reads:

```rust
let Some(target) = target else {
    return Err(Loud::unresolved_call(name).into());
};
```

The builtin step goes **between** the label lookup and that loud fallback.
Do not add a second resolution path in `eval_call`: `eval.rs:529` already delegates to this function, and 4b's own comment records that a second path is what makes `INTERPRET` fragments resolve against the wrong body.

- [ ] **Step 1: Measure the 40.x family**

Probe the oracle for each shape below and commit the exact bytes as the test table.
Measured 2026-08-04, rc **216** in every case, stderr shape:

```
     1 *-* say substr('abc')
Error 40 running <path> line 1:  Incorrect call to routine.
Error 40.3:  Not enough arguments in invocation of SUBSTR; minimum expected is 2.
```

Three sub-codes already measured:

| probe | sub-code | secondary text |
|---|---|---|
| `substr('abc')` | 40.3 | `Not enough arguments in invocation of SUBSTR; minimum expected is 2.` |
| `substr('abc','x')` | 40.12 | `SUBSTR argument 2 must be a whole number; found "x".` |
| `substr('abc',2,3,'pq')` | 40.23 | `SUBSTR argument 4 must be a single character; found "pq".` |

**Probe at least these further shapes, because the sub-code is what the test pins and guessing it is how a whole family ships wrong:** too many arguments; a missing *required* argument in a middle position (`substr('abc',,2)`); a negative where non-negative is required; a non-numeric where a number is required in a builtin whose name is not `SUBSTR`, to confirm the name is interpolated rather than fixed.

**The builtin name in the message is uppercased.**
Confirm what a lowercase call site produces (`say substr(...)` already gives `SUBSTR`), and confirm the `*-*` echo carries the source spelling.

- [ ] **Step 2: Write the failing dispatch test**

A test asserting that `say length('abc')` prints `3` and exits 0, plus one asserting `say length()` produces the measured 40.3 bytes.
Both fail: `dispatch` does not exist.

- [ ] **Step 3: Build `builtin/mod.rs`**

The dispatch is a `match` on the upcased name returning `Option`.
The arity table lives beside it as `(min, max)` per name, with `max` as `Option<usize>` for the variadic ones.

**Assert the dispatch's name set equals `rexx_inventory::builtins::NAMES` minus `EXCLUDED_BUILTINS`.**
That assertion is what makes Task 1's status file redundant *as documentation* while keeping it as the boundary record -- a name added to the C++ table and not here is a compile-or-test failure, not a silent gap.

**Only `LENGTH` is implemented in this task**, and it goes straight into `builtin/string.rs` -- which this task therefore creates -- rather than living in `mod.rs` and being moved later.
It is the arity-check witness and nothing more; Task 3 adds that file's other 22.
A one-builtin file is not a placeholder: `dispatch` needs one real name to prove the chain end to end, and staging it through `mod.rs` would buy a rename and a diff that says nothing.

- [ ] **Step 4: Wire the fallthrough**

`resolve_and_run_call` tries the label table, then `builtin::dispatch`, then falls through to `Loud::unresolved_call`.
**Leave the loud fallback in place** -- Task 13 replaces it with 43.1.

- [ ] **Step 5: Update the derived status file**

Re-run Task 1's harness. `LENGTH` flips to `implemented`. Commit the flipped row.

- [ ] **Step 6: Verify and commit**

---

### Tasks 3-6, 10-12: the seven builtin families

These seven tasks share one shape, and it is stated once here rather than seven times.
**Each task's brief carries this section verbatim plus its own name list.**

**Files (per task):** create one `crates/rexx-exec/src/builtin/<family>.rs`; modify `builtin/mod.rs` (dispatch arms and arity rows only) and `corpus/builtin-status.txt` (the flipped rows).
**Task 3 is the one exception**: `string.rs` already exists, created by Task 2 to hold `LENGTH`, so Task 3 modifies it.
**No task in this group touches any other's file.**

- [ ] **Step 1: Build the probe table before writing any code**

For each builtin in the family, probe the oracle for:

* the documented base case;
* **every optional argument position, separately** -- a one-argument probe cannot distinguish a builtin that honours its optional arguments from one that ignores them;
* at least one probe with a **non-default pad character** where the builtin takes one;
* the empty string and the empty-argument (`f(,2)`) forms;
* the boundary values the ooTest group for that name uses.

Commit the table as the test's expected values, with the probe program beside each row.

- [ ] **Step 2: Read the ooTest group for each name**

`ootest/ooRexx/base/bif/<NAME>.testGroup` is the reference for edge cases and it is the same file Task 15's L1 harness will run.
**Reading it is cheaper than rediscovering its cases by probe**, and a case it covers that the implementation misses will surface at Task 15 as an exempt row that should not be there.

- [ ] **Step 3: Write failing tests from the table, then implement**

- [ ] **Step 4: Re-run Task 1's status harness and commit the flipped rows**

- [ ] **Step 5: Verify and commit**

**Per-task name lists and the traps specific to each:**

**Task 3 -- `string.rs`, 23 names.**
`ABBREV CENTER CENTRE CHANGESTR COMPARE COPIES COUNTSTR DELSTR INSERT LASTPOS LEFT LENGTH OVERLAY POS REVERSE RIGHT SPACE STRIP SUBSTR TRANSLATE VERIFY LOWER UPPER`.
`CENTER` and `CENTRE` are the same function under two names and must dispatch to one implementation.
**`LENGTH` is already in this file from Task 2 and is not rewritten**; this task adds the other 22 around it.
`STRIP` and `VERIFY` both take option letters -- probe each letter and an invalid one.

**Task 4 -- `word.rs`, 7 names.**
`DELWORD SUBWORD WORD WORDINDEX WORDLENGTH WORDPOS WORDS`.
The blank-delimiter rule is shared; factor it once.
Probe leading, trailing and repeated blanks, and tabs.

**Task 5 -- `convert.rs`, 12 names.**
`B2X BITAND BITOR BITXOR C2D C2X D2C D2X X2B X2C X2D XRANGE`.
**`NUMERIC DIGITS` interacts with `C2D`, `D2C`, `D2X` and `X2D`** -- probe each at a non-default `DIGITS`, and never above 1000.
The bit builtins take a pad character.
`XRANGE`'s arguments are single characters, so 40.23 is its live error.

**Task 6 -- `numeric.rs`, 7 names.**
`ABS FORMAT MAX MIN RANDOM SIGN TRUNC`.
`FORMAT` is the largest single builtin in the group and `FORMAT.testGroup` has 767 `::method` bodies, the biggest in `base/bif` -- budget for it accordingly.
**`RANDOM` must be seedable and deterministic under a seed** (D11), and must *not* appear in any corpus program.
Every result here obeys D15: rendering is fixed at creation.

**Task 10 -- `state.rs`, 11 names.** Depends on Tasks 8 and 9.
`ADDRESS ARG CONDITION DIGITS ERRORTEXT FORM FUZZ GC QUEUED SOURCELINE TRACE`.
`ADDRESS()` reads Task 9's tracked environment name.
`ARG()` and `ARG(n)` read 4b's argument model; `ARG(n,'E'|'O')` are separate options to probe.
`CONDITION()` reads 4b's trap state -- probe every option letter inside a live handler, not outside one.
`QUEUED()` reads 4b's in-process queue, and **its differential is single-program only**: `rxapi` is running on this host and `rxqueue('G')` returns `SESSION`, so a cross-process comparison can never match.
`GC()` returns 0, measured.
`TRACE()` with no argument returns the current setting, which `rexxcps.rex` depends on.

**Task 11 -- `datatype.rs`, 4 names.**
`DATATYPE SYMBOL VALUE VAR`.
`DATATYPE`'s option letters are its whole surface -- probe every one.
**`VALUE` is split**: the variable-access form (`value('name')`, `value('name', newval)`) is 4c's; the external-selector form (`value(name, , 'ENVIRONMENT')`) is Phase 7's and must fail loudly naming Phase 7, not silently ignore the third argument.
`VALUE`'s two-argument form *writes* the pool, so it needs `Plan::slot_of`'s idempotent path -- the same one 4b's `EXPOSE` uses -- and not a bare grow.

**Task 12 -- `datetime.rs`, 2 names.**
`DATE TIME`.
Neither may appear in a corpus program (D11).
**`TIME('R')` is stateful** -- it resets an elapsed-time clock and `rexxcps.rex` depends on it -- so its unit test must pin the reset semantics, not a value.
Two probes inside one second cannot distinguish a live clock read from a cached one; construct the probe so they can.

---

### Task 7: the `PARSE` template engine

**Files:**
* Create: `crates/rexx-exec/src/parse_template.rs`
* Modify: `crates/rexx-exec/src/run.rs` (the `InstructionKind::Parse` arm), `tests/owners.rs`, `tests/loud.rs`, `tests/coverage.rs`, `tests/trace_oracle.rs`

**The AST is already complete and this task writes no parser code.**
`rexx_parse::ast::Parse` carries `source`, `upper`, `lower`, `caseless`, and `template: Vec<Option<ParseTrigger>>` where `None` is the comma fence between templates.
`ParseTrigger` carries `kind`, an optional `value` expression, and `targets: Vec<Option<Expr>>` where `None` is a `.` placeholder.
`TriggerKind` has all eight variants: `End`, `Plus`, `Minus`, `Absolute`, `MinusLength`, `PlusLength`, `String`, `Mixed`.

**Sources in this task:** `Var`, `Value`, `Arg`, `Source`, `Version`.
`Pull` and `LineIn` are Task 8's.

- [ ] **Step 1: Measure the trace shape, because it is not what the scoping document says**

Measured 2026-08-04 -- `trace i`, not `trace r`:

```
     2 *-* parse value 'a b c' with p . q
       >L>   "a b c"
       >K>   "VALUE" => "a b c"
       >>>   "a b c"
       >=>   P <= "a"
       >.>   "b"
       >=>   Q <= "c"
```

Four things to take from this and none of them is optional:

* **`>.>` fires at `trace i` and not at `trace r`.** The scoping document probed `trace r`, saw no `>.>`, and recorded the absence; the instrument could not have shown it.
* **`>.>` carries the value the placeholder consumed**, not a marker.
* **`>=>` (ASSIGNMENT) is emitted per *assigned* target**, in template order, with the `NAME <= "value"` shape.
  A `.` placeholder gets `>.>` instead, so the two prefixes partition the targets rather than overlapping.
* **`>K>` carries a `=>` continuation** here (`>K>   "VALUE" => "a b c"`), where 4a's and 4b's `>K>` lines carry a bare value.
  **Measure whether the existing `trace.rs` keyword path emits the continuation before writing any of it**, and say which it was -- a hedge in either direction is what this step exists to remove.

Probe every `TriggerKind` under `trace i` and commit the expectations.
Eight variants, and the ones a `trace r`-only probe will miss are the same ones `>.>` was missed by: `Plus`, `Minus`, `MinusLength`, `PlusLength`.

- [ ] **Step 2: Measure `PARSE SOURCE` and `PARSE VERSION` on this host**

Both are host-dependent -- `SOURCE` carries an absolute path.
**Their witnesses belong in the live corpus, not in a committed `.expected` file**, for the same reason `>I>`/`<I<`'s do.

- [ ] **Step 3: Write the failing engine tests, then implement**

The engine is source-independent: it takes a byte string and a template and produces assignments.
Keep it that way -- Task 8 adds two sources and must add no engine code.

**The comma fence is a template boundary, not a trigger.** `parse arg a, b` is two templates over two argument strings.
A `None` entry in `template` advances to the next source string.

- [ ] **Step 4: Move the four `owners.rs` rows and delete the `loud.rs` witnesses**

`InstructionKind::Parse` moves from `Owner::Phase("4c")` to `Owner::InScope` in `tests/owners.rs:165` and its `SPLIT_TABLE` row at `:352`; `src/lib.rs:761`'s arm loses `Parse`.
Its `loud.rs` witness must be **deleted**, not left stale, or `assert_witness_set_is_complete` fails the other way.

**`Arg` and `Pull` stay `4c` until Task 8.** They are separate `InstructionKind` variants and move separately.

- [ ] **Step 5: Add a `phase-4c.txt` witness and verify**

---

### Task 8: `ARG`, `PULL`, `PARSE PULL`, `PARSE LINEIN`

**Files:** modify `src/run.rs`, `tests/owners.rs`, `tests/loud.rs`, `tests/coverage.rs`, `corpus/phase-4c.txt`

**Interfaces:** consumes Task 7's engine unchanged and 4b's `Interp::queue`.

- [ ] **Step 1: Measure the queue-empty fallback**

`PULL` and `PARSE PULL` read the queue, and **read stdin when the queue is empty**.
Probe both with stdin at `/dev/null` and with a here-string, and record what an empty queue plus closed stdin produces.
`PARSE LINEIN` reads stdin directly.

**This is where 4b's `PUSH`/`QUEUE` gets its first differential witness.** The 4b gate records the queue as shipping with storage verified only in-crate; a corpus program that pushes and pulls closes that, and the gate should say so.

- [ ] **Step 2: Measure `ARG template` against `PARSE UPPER ARG`**

They are the same instruction with `upper` set.
Confirm rather than assume, and confirm that `ARG` with no template is legal and does nothing.

- [ ] **Step 3: Write failing tests, implement, move the two `owners.rs` rows, delete the two `loud.rs` witnesses**

- [ ] **Step 4: Verify and commit**

---

### Task 9: the `ADDRESS` instruction and environment tracking

**Files:** modify `src/run.rs`, `src/activation.rs`, `tests/owners.rs`, `tests/loud.rs`, `tests/coverage.rs`

**Scope is the environment name only.**
`ast::Address` carries `environment`, `dynamic`, `command` and `io`.
This task implements `environment` and `dynamic`; **`command` and `io` are Phase 7's under D18 and must still fail loudly naming Phase 7**, not be silently dropped.

- [ ] **Step 1: Measure the default and the swap semantics**

Measured: `say address()` with no `ADDRESS` instruction prints `sh` on this host.
**That default is platform-supplied and therefore Phase 7's**, which means the corpus witness cannot assert `sh` -- it must assert the *swap*, not the initial value.

`ADDRESS` with no operand swaps to the previous environment.
Probe: the two-deep swap, the swap with no prior environment, and whether the setting survives a `CALL` and a `RETURN` (it is per-activation state, like `Settings`, so it belongs on `Activation` and not on `Interp` -- the same shape as 4b's `trace_mode` move).

- [ ] **Step 2: Write failing tests, implement, move the `owners.rs` row, delete the `loud.rs` witness**

- [ ] **Step 3: Verify and commit**

---

### Task 13: `::routine` dispatch, `>I>` and `<I<`

**Files:** modify `src/run.rs`, `src/plan.rs`, `src/lib.rs`, `src/error.rs`, `tests/trace_oracle.rs`, `corpus/phase-4c.txt`

**Why last of the implementation tasks.** Builtins shadow `::routine`s, so the resolution order cannot be verified until the table is complete.

- [ ] **Step 1: Set `BodyKey::directive`**

`plan.rs:79`'s `directive: Option<usize>` is `None` at its one construction site (`:631`).
This task is its first `Some(index)` -- the field was carried for 4b, which never needed it.

- [ ] **Step 2: Give a `::routine` activation its own pool**

Measured: a caller with `nn = 5` calling a `::routine` that says `nn` prints `NN`.
**This is not 4b's `PROCEDURE` isolation reused.** A `::routine` has a different `CodeBody`, therefore a different `Plan`, therefore a different name-to-slot map -- the slot-index-identity property `PROCEDURE EXPOSE`'s alias bitset rests on does **not** hold across bodies.
An implementer who reuses the `PROCEDURE` path here gets a silently wrong pool.

- [ ] **Step 3: Place it third in the resolution order**

Internal label, then builtin, then `::routine`, then Error 43.1.
Measured: `call max 1, 9` with a `::routine max` present returns 9 and the routine never runs.

**Replace `Loud::unresolved_call` with a real 43.1 raise.**
Measured from a clean directory: `Error 43.1 rc 213, "Routine not found"`.
`unresolved_call`'s existing 128-byte truncation of a `Call::Dynamic` target was a deliberate divergence justified by the loud path; **re-derive it against 43.1's own behaviour** rather than carrying it -- the oracle's 43.1 does not truncate, and that comment says so.

- [ ] **Step 4: Trace does not cross into it**

Measured: a caller's `trace r` echoes its own clauses and none of the routine's.
An internal label's clauses *are* echoed under the same setting, which is what 4b implements -- so this is a deliberate difference between two paths that share a function.

- [ ] **Step 5: `>I>`/`<I<` under the two-condition gate**

`tracingLabels() && isMethodOrRoutine()` (`RexxActivation.cpp:3655`).
Measured:

* `trace l` in the **caller**, targeting a `::routine` -> nothing. The caller's setting does not cross.
* `::options trace labels` in the package, **or** a non-dynamic `trace l` as the routine's own first instruction -> both lines fire.
* The content is **not** a clause echo. Verbatim:

```
>I> Routine "ZORKOLO" in package "<absolute path>".
<I< Routine "ZORKOLO" in package "<absolute path>".
```

The absolute path makes any committed expectation host-dependent, so **the witness lives in the live corpus, not in `tests/trace_oracle/`**.

`::options` is a directive and is **not** in scope -- if the routine's own `trace l` is the only reachable route, say so in the exclusions file rather than implementing `::options`.

- [ ] **Step 6: Flip `>I>`/`<I<` to `Witnessed` and verify**

---

### Task 14: the compound-`DO` control-variable fix

**Files:** modify `src/run.rs` (`bind_control`), `crates/rexx-parse/src/ast.rs`, `corpus/keyword-exempt.txt`, `docs/superpowers/plans/phase-4-exclusions.txt`

**This gap was assigned to 4c by the 4b gate's Step 3c ruling** and is the one 4c inherits with a fix already scoped.

**What the divergence is.** `do cv.j = 1 to 5` is legal Rexx and the oracle iterates it, assigning the compound `CV.J` on every pass.
`bind_control` writes the control variable through `slot_of`, a flat name-to-slot lookup, so `CV.J` becomes the literal name of one simple variable and no tail is resolved -- while the same executor resolves the same name correctly in `say cv.j` one line later.
**It is not a parse gap**: `cv.j` is a single symbol token and the parser interns `"CV.J"` whole.
The `LEAVE`/`ITERATE`/`END` forms naming a compound all dispatch correctly.

**Recorded cost:** a `rexx-parse` signature change -- `Controlled::control` carrying the `VariableRef` shape an assignment target already does -- plus roughly twenty lines and re-verification of bound-before-test, `FOR` and `ITERATE`.

- [ ] **Step 1: Reproduce all three narrowing probes before changing anything**

- [ ] **Step 2: Fix, then re-run `REXX_KEYWORD_GATE=1`**

Six `base/keyword` bodies turn on this and they are the **only assertion failures in the whole table** -- every other non-passing body fails loudly.
When the fix lands, all six start passing and `the_exempt_set_matches_the_current_failures` goes red until they are removed from `keyword-exempt.txt`.
**That red test is this task's success signal**, and it is automatic.

- [ ] **Step 3: Remove the six rows, move the exclusions row to `CLOSED DEFECTS`, verify, commit**

---

### Task 15: the `base/bif` L1 harness, the 4c corpus subset, and `mutate-4c.sh`

**Files:**
* Create: `crates/rexx-extract/src/bif.rs`, `crates/rexx-exec/tests/bif_assertions.rs`, `rust/corpus/bif-exempt.txt`, `rust/scripts/mutate-4c.sh`
* Modify: `rust/corpus/phase-4c.txt`, `rust/corpus/README.md`, `crates/rexx-exec/tests/coverage.rs`

- [ ] **Step 1: Extract `base/bif` by reusing `keyword.rs`**

D12: no third extractor.
Preserve `keyword.rs`'s conservation invariant, `rows + dropped == calls`, and its `DropReason` detail field.
**Match the token, never the prefix, and match it case-insensitively.**
A prefix match swallows `assertSameList` (5 occurrences here), which is a different method.
A case-sensitive match is safe in this group and is not safe in general: `base/keyword` carries 510 capital-`A` `AssertSame` calls beside 1,931 lowercase, and a case-sensitive count there drops a fifth of the table.
Measured population for `base/bif`: **5,441 `assertSame`, zero capital-`A`, across 4,420 `::method` bodies in 78 files.**

State in the file's header that `assertTrue` (186), `assertEquals` (106) and `assertFalse` (51) are dropped, and let the conservation invariant carry the number rather than a comment.

- [ ] **Step 2: Measure whether `expectSyntax` couples to a later `assertSame` here**

It does in `base/expressions` -- an `expectSyntax` marker changed what a later `assertSame` meant.
1,021 calls here.
**Measure it; do not inherit the answer either way.**

- [ ] **Step 3: Build the harness with both-direction policing**

`REXX_BIF_GATE=1`, the same shape as `REXX_KEYWORD_GATE`.
A body that starts passing must be as red as one that starts failing.
The exempt set's attribution is **derived from the loud message**, not hand-written.

- [ ] **Step 4: Build `corpus/phase-4c.txt`**

The union of all three subset files is what every harness reads.
Add `phase_4c_subset_matches_the_committed_list` to `coverage.rs`, the pin 4b's gate found missing for `phase-4b.txt` -- **nine of its twelve entries were deletable with everything green**, including one criterion's only witness.

**Corpus rules for 4c, written into `corpus/README.md` beside the `DO OVER` one:**
no `RANDOM`, no `DATE`, no `TIME` (D11); `QUEUED()` single-program only; no `DO OVER` on a stem (D3).

- [ ] **Step 5: Write `mutate-4c.sh`**

Carry 4a's and 4b's guard mechanism -- exact-match, exactly-once application; a baseline before the first mutation and after the last restore; three-way `PASSED`/`DIVERGED`/`INFRA_FAILURE` that never folds an infrastructure failure into either bucket; a non-zero test-run count per target.
**Declare each mutation's expected outcome per instrument in advance**, so an unexpected catch fails as loudly as an unexpected survival.

Mutations must target 4c's own code. Suggested shapes, each with a stated declaration:
a builtin's optional argument ignored; an arity bound off by one; a `PARSE` trigger's boundary off by one; a `.` placeholder assigning instead of discarding; the comma fence treated as a trigger; `ADDRESS`'s swap keeping the old name; `::routine` resolved before the builtin table; `TIME('R')` not resetting.

- [ ] **Step 6: Write `docs/superpowers/plans/phase-4c-gate.md`**

**Write the criteria before running anything**, per 4b's Step 1.
Carry 4b's ten forward with these amendments, each already known rather than to be discovered:

* **Criterion 2** (`tests/assertions.rs`) will report **the same 4,224 of 4,259 with 35 RUNTIME-BLOCKED** that 4a and 4b reported.
  All 35 are `unblocked_by: "Phase 5"`. **That sameness is the correct result**, and saying so before measuring is what makes it distinguishable from a stall.
* **Criterion 3**'s target is **16 of 19 prefixes**, not 17: `>.>`, `>I>` and `<I<` are 4c's; `+++` is Phase 7's under D-P.
* **Criterion 10**'s 790 `4c` rows in `keyword-exempt.txt` are **designed to fire here**.
  But **790 is an upper bound on what landing 4c fixes, not a measure of 4c's remaining surface**: four bodies are known to differ for reasons 4c cannot fix -- three `CALL` bodies fail **under the C++ oracle itself** with `Error 43, Routine not found`, and `NUMERIC::test_42` exits 3 because its body falls through into its own `dig:` label.
* **A new criterion for the derived builtin-status file**, policed in both directions, whose falsification is Task 1's Step 3.
* **A new criterion for `base/bif`**, reported as a measurement and **not gated on a threshold** -- a strict gate on a table nobody has looked at turns a measurement into a blocker.

**Every criterion gets the same question asked of it before it is written: what degenerate implementation satisfies this, and would deleting its subject leave it green?**
Three concrete traps for 4c specifically:

* **"Each of the 66 names is recognised" is satisfied by a stub returning `''` for all 66.**
  The criterion must assert a **value per builtin**, captured from the oracle, or it is `/bin/true` with 66 rows.
* **A `PARSE` criterion asserting "the program exited 0" passes for a program that parsed nothing.**
  Assert the assigned values, and choose them so an unset target renders as its own derived name and is recognisably wrong.
* **Criterion 4's collector control must delete a root that a *builtin* holds** -- an argument between evaluation and the builtin's own use.
  Re-running 4b's activation-shaped control here re-tests 4b.

- [ ] **Step 7: Run everything, record every figure with its command and unpiped exit status**

---

## Explicitly not in scope

* **`ExprKind::List`** -- Phase 5's (D7). The three `num/` corpus programs return then.
* **`::method`, `::class`, `::requires`, `::attribute`, `::options`** -- Phase 5's. Only `::ROUTINE` is carved out (D-R).
* **`QualifiedCall`** (`ns:name(...)`) -- Phase 5's; namespaces come from `::REQUIRES`.
* **Command dispatch, `ADDRESS ... WITH` redirection, the platform-supplied default environment, and `+++`** -- Phase 7's (D18, D-P).
* **`TRACE ?`'s interactive pause and its two banner lines** -- Phase 7's (D-P).
* **The fifteen excluded builtins** (D4), and `VALUE`'s external-selector form.
* **I32-I35**, unowned and unchanged: prefix-operator recursion outside the depth budget; recursive `Debug`/`PartialEq`/`Clone` on `Expr`; the depth counter protecting a sized caller only; `Plan::by_symbol` as a `HashMap` where D16 wants a `Vec` index.
* **The `DO OVER`-on-a-stem traversal-order deviation** (D3), which no in-scope 4c builtin exposes.
