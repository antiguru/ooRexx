# Pre-Phase-5 defects implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Clear the known, verified, unfixed divergences between this crate and the oracle before Phase 5 begins building on top of them.

**Architecture:** Four independent defects, one task each, ordered by severity. Task 1 is a process-aborting panic in the activation/frame machinery that Phase 5's message dispatch will build on, so it goes first. Tasks 2 and 3 are trace-stream divergences and both touch `run.rs`; they are sequential for that reason. Task 4 is a builtin's window rule and touches nothing the others do.

**Tech Stack:** Rust 2024, workspace at `rust/`. Two engines -- the tree-walker (`run.rs`, `eval.rs`) and the compiled register op stream (`ir/`) -- held byte-identical by `tests/ir_dual.rs`.

## Global Constraints

Every task's requirements implicitly include this section.

* **The C++ tree at `/home/moritz/dev/repos/ooRexx/` is the oracle and is READ-ONLY.** Never modify `interpreter/`, `samples/`, `build/`, `ootest/`.
* **Wrap every oracle invocation** as `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx /abs/path/FILE )`, run from a fresh empty directory you `mkdir` yourself, with absolute paths for every redirect. The scratchpad root is on the oracle's external-routine search path and holds stale `.rex` files that a probe will execute by accident.
* **Read stdout, stderr and exit status as three separate descriptors.** Never `2>&1` into one string: trace goes to stderr and `SAY` to stdout, and a merged comparison shows lines moved rather than changed.
* **Three programs crash the oracle. Never run them:** `select; when 1 = 0 then; when 2 = 2 then nop; end`; `say date('M','0','D')`; any `NUMERIC DIGITS` above 1000.
* **A symbol named `x` or `b` immediately followed by a quoted string parses as a hex or binary literal.** Use `n1`, `cv`, `zz`, `qq`.
* **Both engines must agree with the oracle byte for byte.** `MAX_EVAL_DEPTH` is the single accepted divergence. Every behavioural check runs under `REXX_ENGINE=tree-walker` and `REXX_ENGINE=ir`.
* **Gates, from `rust/`, never the repository root:** `cargo fmt --all --check` (not `--edition`, which is not a `cargo fmt` flag and exits before doing any work); `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --release --workspace`. Read every exit status unpiped. A test run that may allocate without bound goes under `memcap 8G`.
* **`cargo test <name>` exits 0 when it matches nothing.** Assert a non-zero run count before believing a named test passed.
* **Use `/bin/grep -a`, never bare `grep`** -- it is a ugrep wrapper that silently skips non-UTF-8 files and every gitignored path.
* **Restore files from a `cp` backup, `touch` after restoring, verify with `sha256sum -c`, and rebuild.** Never `git checkout --`, never `git checkout-index`, never `git add -A`, never `git reset --hard`, never force-push.
* **Comments:** prefer `--` over an em-dash. A comment may not name the size of a set -- name the set, never its cardinality; measurements keep their numbers. Comments say what the code does, not how it got there.
* **Commit first, then read the hash back with `git log`.** Use `git commit -F -`, not `-m`. Stage exact paths.
* **Scratch files never in the repository.**
* **A test that cannot fail is a defect.** Every fix needs a witness: break it deliberately, confirm the check goes red, restore from backup, verify, rebuild.
* **Fix the defect, not the check that noticed it.** Where an assertion caught corruption, keep the assertion.
* **Do not fix any other divergence a task happens to find.** Write it down in the report and leave it.

---

### Task 1: `PROCEDURE` first in a `::ROUTINE` panics where the oracle raises 17.1

**Background detail:** `docs/superpowers/plans/2026-08-13-procedure-in-routine-panic.md`. Read it first; it carries the mechanism as the reviewer described it.

**Files:**
- Modify: `rust/crates/rexx-exec/src/run.rs` (`exec_procedure`, and wherever `PROCEDURE` admission is decided)
- Modify: `rust/crates/rexx-exec/src/activation.rs` (`Activation::routine`, `entered_by_call`)
- Do NOT modify: `rust/crates/rexx-core/src/roots.rs:409` -- that assertion is correct and stays
- Test: a corpus program plus a differential case, both engines

**Interfaces:**
- Produces: nothing later tasks consume. This task is self-contained.

**The reproduction, verified on `328c51fcf`:**

```rexx
call sub
zz = 1
say zz
exit 0
::routine sub
procedure
say 'in sub'
```

Oracle: echoes the `procedure` line, then the `call sub` line, then

```
Error 17 running .../p.rex line 6:  Unexpected PROCEDURE.
Error 17.1:  PROCEDURE is valid only when it is the first instruction executed after an internal CALL or function invocation.
```

at **rc 239**. This crate, on **both** engines: panics at `crates/rexx-core/src/roots.rs:409` with `grow_slots on a frame that is not the top one`, **rc 101, stdout empty** -- the `say zz` never runs.

**The mechanism:** `Activation::routine` sets `entered_by_call: true`, so this crate admits a `PROCEDURE` the oracle rejects. `exec_procedure` pushes a second frame onto an activation whose `owns_frame` is already true; `invoke_call` pops one on the way out; the caller's next `slot_of` miss reaches `grow_slots` with a frame that is no longer the top. **The wrong decision is admitting the instruction. Fix the admission, not the assertion.**

- [ ] **Step 1: capture the oracle's actual rule, case by case.**

Do not reason from 17.1's sentence. Capture one oracle transcript per case, into a fresh empty directory, and write the table into your report:

  * an internal label reached by `CALL`, `PROCEDURE` first
  * an internal label reached as a function call, `PROCEDURE` first
  * a `::ROUTINE` reached by `CALL`, `PROCEDURE` first
  * a `::ROUTINE` reached as a function call, `PROCEDURE` first
  * the main program, `PROCEDURE` first
  * each of the above with some other instruction executed before the `PROCEDURE`
  * a `::METHOD`, `PROCEDURE` first

A `::METHOD` is Phase 5 and out of scope for the fix. **Capture it anyway**, so the exclusion is a recorded decision rather than a gap.

- [ ] **Step 2: find every construction of `entered_by_call` and map each to a Step 1 case.**

`/bin/grep -rn "entered_by_call" rust/crates/`. For each construction site, say in your report which Step 1 case it covers and whether the oracle grants the property it claims. The defect is that one of them claims a property the oracle does not grant.

- [ ] **Step 3: write the failing differential test.**

Add the reproduction above as a corpus program, or as a test that runs it under both engines and compares against the oracle bytes you captured in Step 1. It must fail before the fix with the panic, and it must check stdout, stderr and exit status separately.

Run it and confirm it fails with `rc 101`.

- [ ] **Step 4: raise 17.1 where the oracle raises it.**

The clause echo ordering is part of the expected bytes: the oracle echoes the `procedure` line and **then** the `call sub` line, which is the caller's frame being reported.

- [ ] **Step 5: run the test and confirm it passes**, on both engines, byte for byte against the oracle on all three descriptors.

- [ ] **Step 6: sweep for the same shape, and write down what came back clean.**

Any other instruction whose legality depends on how the activation was entered, and any other place `owns_frame` is set true twice. Report what you checked including the clean results -- a sweep that reports only its finds is indistinguishable from one that stopped early.

- [ ] **Step 7: the mutation witness.**

Break the fix deliberately (re-admit the instruction) and confirm the new test goes red. Restore from a `cp` backup, `touch`, `sha256sum -c`, rebuild, re-run.

- [ ] **Step 8: gates, corpus sweep, commit.**

`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `memcap 8G cargo test --release --workspace --no-fail-fast`. Report how many corpus programs changed output. Commit with `git commit -F -`, then read the hash back with `git log`.

---

### Task 2: a trap handler's clauses echo at the wrong indent

**Background detail:** `docs/superpowers/plans/2026-08-13-handler-clause-indent.md`. Read it first -- it carries a hypothesis stated specifically so that Step 3 can refute it.

**Files:**
- Modify: `rust/crates/rexx-exec/src/run.rs` (`clause_state.current_value_indent` save/restore across a trap transfer)
- Possibly modify: `rust/crates/rexx-exec/src/ir/drive.rs`
- Test: transcripts under `rust/crates/rexx-exec/tests/`, both engines

**Interfaces:**
- Consumes: nothing from Task 1.
- Produces: nothing Task 3 consumes, but **both tasks change trace output and both touch `run.rs`**, so Task 3 starts from this task's committed tree.

**Instance one, verified on `328c51fcf`:** `trace r`, `signal on syntax name bad`, a `call sub` whose callee raises. The handler's clauses echo **four spaces too deep**:

```
oracle                          this crate
   11 *-* bad:                     11 *-*     bad:
   12 *-* say 'trapped'            12 *-*     say 'trapped'
```

**It needs the callee.** The same trap raised in the main program agrees byte for byte on both engines. Both engines emit identical bytes here, so it is not the compiled form.

**Instance two, recorded since phase 4e** in `phase-4e-gate.md` and `2026-08-09-phase-4e-ir.md`: a `CALL ON` handler delivered at a promoted loop header's boundary indents its own clauses **two spaces less** than the oracle -- `13 *-*   h:` against `13 *-*     h:`.

**The two instances are in opposite directions**, which is why they are one investigation and not two fixes.

- [ ] **Step 1: reproduce both, side by side, and write the transcripts down before changing anything.**

Both are pre-existing, so a capture taken now is the baseline a fix has to move in exactly the two expected places. Capture under `trace r` and `trace i`, both engines, all three descriptors.

- [ ] **Step 2: find where the indent is saved and restored across an activation boundary, and where a trap transfer bypasses it.**

`current_value_indent` lives in `clause_state`. `run.rs` already carries a comment about a handler's own indent near the `CALL ON` delivery.

- [ ] **Step 3: decide whether it is one defect, and say so in the report either way, with the evidence.**

The hypothesis to refute, from the detail file: *this crate keeps whatever indent is live at the transfer, where the oracle sets it from the kind of transfer* -- `SIGNAL ON` unwinds so the handler belongs at the program's indent; `CALL ON` invokes the handler as a subroutine so its clauses belong one level deeper. It predicts that a fix which merely restores an indent fixes the `SIGNAL` case and leaves the `CALL ON` case exactly as wrong.

**A fix that lands without answering this is the failure mode.** The second instance has been recorded for weeks precisely because nobody asked whether it generalised.

**Read the transcripts against the possibility that the oracle is wrong.** This project has found a genuine upstream trace-indent defect already: two loop-exit paths in the C++, one restoring the indent and one bare-decrementing it, so a completed loop under-indents later clauses. That one is not this one, but "the oracle is wrong here" is live and must be decided from the transcripts rather than assumed away.

- [ ] **Step 4: write the failing test, run it, confirm it fails.**

Both engines, all three descriptors, against oracle bytes captured in Step 1.

- [ ] **Step 5: fix, and measure the blast radius.**

Corpus sweep, both engines, and a count of how many corpus programs change output. **Trace-indent changes are the class most likely to move transcripts far from the construct being fixed**, so the count is the finding, not a formality.

- [ ] **Step 6: check the third place this could hide.**

A raise inside a callee inside a loop body; a `SIGNAL ON` and a `CALL ON` handler in the same program; and a handler that itself calls a routine. Report each, including the ones that came back clean.

- [ ] **Step 7: the mutation witness**, then gates and commit, as Task 1 Steps 7 and 8.

---

### Task 3: the `>K>` line a `DO OVER ... FOR` does not print

**Background detail:** `docs/superpowers/plans/2026-08-13-over-for-keyword.md`.

**Files:**
- Modify: `rust/crates/rexx-exec/src/run.rs` (`HeaderRole::keyword()`, and its doc comment in the same edit)
- Verify, and modify only if needed: `rust/crates/rexx-exec/src/ir/compile.rs`
- Re-capture: `ir_dual_cases/loop-header-values`, and any `trace_oracle` transcript holding a `DO OVER ... FOR`

**Interfaces:**
- Consumes: Task 2's committed tree, since both change trace output in `run.rs`.

**The divergence, captured 2026-08-13** on `trace r`, program `zs = 'a b c'` then `do qq over zs for 1`:

```
oracle                              this crate
  >K>   "OVER" => "a b c"             >K>   "OVER" => "a b c"
  >K>   "FOR" => "1"                  (nothing)
```

`HeaderRole::OverFor::keyword()` answers `None`, so no `Op::TraceKeyword` is emitted and the tree-walker echoes nothing either. **Both engines agree with each other**, so this is the role table withholding a keyword, not anything the compiled form does.

**A note on probing this construct:** a probe reaching for `.array~of` hits the declared Phase 4 message-send gap and exits 120, which says nothing about the defect. Build the probe from values Phase 4 can produce -- a blank-delimited string, a number.

**This is a defect and not an exclusion.** `phase-4-exclusions.txt` holds work assigned to a later phase and permanent chosen differences; this is a difference nobody chose, in a construct otherwise in scope and implemented. Do not add a row there.

- [ ] **Step 1: find out whether `OverFor` is alone.**

Capture the oracle for **every** `HeaderRole` variant under both `trace i` and `trace r`, one program per variant, and write the transcripts down before changing anything.

`Initial` is the one other variant documented as echoing no `>K>`, and that is recorded as measured rather than assumed. **Confirm it rather than trusting the comment** -- the comment beside `OverFor` made the same kind of claim and was wrong. A second withheld keyword found here changes the shape of the fix from one variant to a table.

- [ ] **Step 2: write the failing test, run it, confirm it fails.**

- [ ] **Step 3: give the role its keyword, on the tree-walker.**

`HeaderRole::keyword()` in `run.rs`. **Correct its doc comment in the same edit:** it currently says `None` is "for the roles the oracle echoes nothing for", which is the sentence that made this gap invisible.

- [ ] **Step 4: let the compiled engine follow.**

`compile.rs` emits `Op::TraceKeyword` where `role.keyword().is_some()`, so the compiled side should need no change of its own. **Verify that rather than assume it**, and if it does need one, say so in the report.

- [ ] **Step 5: re-capture the transcripts from the oracle, never by hand.**

`ir_dual_cases/loop-header-values` pins the gap on purpose and its rows now change.

- [ ] **Step 6: run the corpus sweep and the gates**, and say in the report how many corpus programs changed output. **A construct this narrow should move few; a surprise there means Step 1 missed something.**

- [ ] **Step 7: the mutation witness**, then commit, as Task 1 Steps 7 and 8.

---

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
