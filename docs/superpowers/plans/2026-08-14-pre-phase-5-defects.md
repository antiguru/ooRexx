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
* **The exception, and its limit.** Where the assigned fix *mechanically* forces a second site to change -- a shared field whose meaning you are correcting, a reader that must now answer for a new type -- correcting that site is part of the fix, not a second one. Preserving a known-wrong answer in a reader you are actively rewriting is not in scope discipline's gift. **What the rule is protecting is attribution**, so the test is whether the differential still says which programs saw what: give each half its own test and its own mutation witness, and say plainly in the report that you did. A divergence you could have left untouched, in a site the fix does not force, still goes on the found-and-not-fixed list.

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

**Instance one, verified on `328c51fcf`:** `trace r`, `signal on syntax name bad`, a `call sub` whose callee raises. The handler's clauses echo **two spaces too deep**:

```
oracle                          this crate
    7 *-* bad:                       7 *-*   bad:
    8 *-* say 'in handler'           8 *-*   say 'in handler'
```

**The depth is one level per enclosing call, and this plan said four.** That figure came from a controller probe carrying two call levels -- `call sub`, then `sub:` calling `inner:`, which raises -- and was written beside a description of the one-level program, which measures two. Corrected after Task 2 measured the minimal form; the four-column shape is real and needs the raise a further call down.

**It needs the callee.** The same trap raised in the main program agrees byte for byte on both engines. Both engines emit identical bytes here, so it is not the compiled form.

**Instance two, recorded since phase 4e** in `phase-4e-gate.md` and `2026-08-09-phase-4e-ir.md`: a `CALL ON` handler delivered at a promoted loop header's boundary indents its own clauses **two spaces less** than the oracle -- `13 *-*   h:` against `13 *-*     h:`.

**Instance three, found by Task 1 and reproduced independently on `3cf9fcaba`: no trap is needed at all.** A plain `SIGNAL` inside a `CALL`ed label, with no condition handler anywhere in the program:

```rexx
trace r
call sub
exit 0
sub:
signal onward
onward:
zz = 1 / 0
```

Both engines echo the clauses after the `SIGNAL` **two spaces too deep**, and agree with the oracle on stdout and on `rc 214`:

```
oracle                          this crate
    6 *-* onward:                   6 *-*   onward:
    7 *-* zz = 1 / 0                7 *-*   zz = 1 / 0
```

**This is the sharpest form of the defect and it reframes Step 3's hypothesis**: handlers are not the subject. A `SIGNAL` that leaves a called label does not restore the indent, whether or not a condition raised it. Start from this case -- it has the fewest moving parts of the three.

**The instances are in different directions**, which is why they are one investigation and not separate fixes.

### The harnesses cannot see this defect, and that governs how the task is tested

`phase-4-exclusions.txt`'s **DEVIATION 0** is the one normalisation the differential harnesses apply to `stderr`, shared by `tests/corpus.rs` and `tests/trace_oracle.rs`: it collapses the run of ASCII spaces between a trace line's prefix marker and its content down to a single space. **That is exactly this defect's signature.**

Two consequences, both binding:

* **The test for this fix must compare raw, un-normalised `stderr`.** A test that goes through the shared harness will pass before the fix and after it, which is the "test that cannot fail" this project treats as a defect.
* **The corpus-sweep count in Step 5 will read zero changed programs even if the fix moves many transcripts.** Report the raw count as well, by comparing un-normalised bytes across the corpus before and after. A sweep count taken through the normalising harness is not evidence here and must not be reported as though it were.

Do not change DEVIATION 0 or its normalisation in this task. It is a recorded, accepted deviation with its own justification; whether it should survive this fix is a question for the report, not an edit.

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

### Task 5: a `SELECT CASE` expression's clause boundary delivers a handler two columns short

**Files:**
- Modify: `rust/crates/rexx-exec/src/run.rs`, at the same call site Task 2 settled for a loop header's boundary
- Test: `rust/crates/rexx-exec/tests/trace_indent.rs`, the raw-stderr harness Task 2 created
- Re-capture: the note in `rust/crates/rexx-exec/tests/ir_dual_cases/loop-header-boundaries` that records this gap becomes false when it closes

**Interfaces:**
- Consumes: Task 2's fix and its `trace_indent.rs` harness. **Do not start this before Task 2 has landed.**

**Found by Task 2, deliberately not fixed there** because it is a different site that the assigned fix does not force, and recorded in commit `1638a0ea4`. Reproduced independently by the controller on that commit.

`SELECT` is a block instruction, so the oracle's level counter is already one deeper when a `SELECT CASE` expression's clause ends. This crate settles that boundary at the level the clause echoed at:

```rexx
trace r
call on user zx name h
zv = 'unset'
select case raiser()
  when 1 then say 'one'
  otherwise say 'other'
end
say 'after' zv
exit
raiser:
raise user zx return 2
h:
zv = 'set'
return
```

```
oracle                              this crate
   12 *-*     h:                       12 *-*   h:
   13 *-*     zv = 'set'               13 *-*   zv = 'set'
      >>>       "set"                     >>>     "set"
   14 *-*     return                   14 *-*   return
```

Both engines agree with each other; stdout and `rc 0` agree with the oracle. Only the trace stream differs, by the same two columns Task 2 fixed for a loop header.

**A trap queued by a `WHEN` condition instead agrees with the oracle**, because a `WHEN` is not a block instruction and its clause ends at the level it echoed. That is the adjacent success and it is what pins the rule to *block* instructions rather than to conditions or to `SELECT`.

- [ ] **Step 1: capture the baseline for the case above and for the `WHEN` case beside it**, both engines, raw un-normalised stderr, all three descriptors, before changing anything. The `WHEN` case must be shown already agreeing -- a fix that moves it has broken something.

- [ ] **Step 2: establish whether `SELECT CASE` is the last of them.**

Task 2 settled `DO`/`LOOP` and left this. Ask the same question of every remaining block instruction the oracle counts a level for, and write down what came back clean as well as what did not. **"Came back clean" does not distinguish a repaired probe from one that never had the defect** -- run each against a build without Task 2's fix as well, which is the control Task 2 used for its own hiding places.

- [ ] **Step 3: write the failing test in `trace_indent.rs`**, which compares raw bytes. A test routed through `tests/corpus.rs` or `tests/trace_oracle.rs` passes before and after, because DEVIATION 0 collapses exactly this signature. Run it and confirm it fails.

- [ ] **Step 4: fix, and confirm the `WHEN` case did not move.**

- [ ] **Step 5: correct the note in `ir_dual_cases/loop-header-boundaries`.** It currently records this gap as measured and open; when it closes, that comment states something false and must be corrected rather than hedged.

- [ ] **Step 6: the mutation witness, the raw blast-radius sweep and the gates**, as Task 2 did them: the raw count is the evidence and the harness count is not.

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

---

### Task 6: a plain `DO`'s clause boundary runs after its body, so a requeued condition is delivered late

**Files:**
- Modify: `rust/crates/rexx-exec/src/run.rs`, the `InstructionKind::Do` arm and whatever it shares with `InstructionKind::Select`
- Test: `rust/crates/rexx-exec/tests/` -- **not** `trace_indent.rs`. This defect is observable on stdout with no `TRACE` in the program, so it belongs with the ordinary behavioural tests, and a test that needs tracing to see it has tested the wrong thing.
- Re-capture: the `DO` entry added to `rust/crates/rexx-exec/tests/ir_dual_cases/loop-header-boundaries` by Task 5's fix round 2 records this as open; it becomes false when this closes.

**Interfaces:**
- Consumes: Task 5's recording of the defect and the requeue probe shape it names. **Do not start before Task 5 has landed.**

**Found during Task 5's re-review, out of that task's scope, and the only defect in this plan that needs no `TRACE` to see.** Every other task here moves a trace column. This one moves program output.

A `CALL ON` handler that ends in `raise ... return` leaves a second trapped condition pending, and the next clause boundary delivers it. The oracle delivers it at the plain `DO`'s own boundary, before the body runs. This crate's boundary for that clause is not missing -- it is open across the body, so the body's first clause reaches its own boundary first and delivers there instead.

**The cause, traced rather than inferred, because the first attribution written here was wrong.** `step`'s `InstructionKind::Do` arm calls `run_loop`, which runs the whole loop before returning, and it is called from inside the clause unit `step_in_temps_frame` opened -- so `leave_clause`, the boundary that delivers, does not run until after `END`. The oracle's `RexxInstructionSimpleDo::execute` (`interpreter/instructions/SimpleDoInstruction.cpp:71`) traces the instruction, opens the block, handles the debug pause and returns -- it never runs the body -- so its clause ends before any body instruction does. **This account is written from the tree-walker's path and the defect is on both engines**; the compiled path is different code and the same shape, `crate::ir::Op::LoopRun` sitting inside the clause region and calling `run_loop_with_header`, which resolves the loop before that region closes. Fix and measure both. The fix shape follows and is not the one the first draft of this task implied: close the header clause before the body runs, the way `InstructionKind::Select`'s arm already does. That clause currently owns the body's temps frame, so this is larger than it sounds.

```rexx
call on user zx name h
call on user zy name g
zv = 'unset'
zw = raiser()
do
  say 'body'
end
say 'after' zv zw
exit
raiser:
raise user zx return 5
h:
zv = 'set'
raise user zy return 1
g:
say 'G ran' sigl
return
```

```
oracle          this crate (both engines)
G ran 5         body
body            G ran 6
after set 5     after set 5
```

Wrong delivery order and a different `SIGL`, on stdout, `rc 0` on both sides. `do label zl` behaves the same. `if ... then` is clean on this route, and a plain `SELECT` is *not* -- see below.

**The last line of that transcript is the trap in this task.** `after set 5` agrees on both sides because its `5` is `zw`, what `raiser()` returned, not a `SIGL`. A reviewer reconstructing this program with `say 'after' zv sigl` as the last statement reproduces the oracle's transcript exactly and gets `after set 6` from this crate -- and correctly reported the record as wrong. Two programs, one oracle transcript, two crate transcripts. Name the program in anything you record.

- [x] **Step 1: capture the baseline** for the program above, for the `sigl` variant of it, and for the **empty-body variant** (`do` / `end` with nothing between), both engines, all three descriptors, with and without `trace r`, before changing anything. The `sigl` variant tells a fix that moved delivery from one that moved only the printed value. The empty-body variant is the control that pins the cause and it **agrees with the oracle today** -- `G ran 5` on both sides, `SIGL` naming the `do` line rather than the `end` line or the one after it. A fix that breaks this one has moved the boundary somewhere new rather than earlier.

- [x] **Step 1b: a second instance of the mechanism, and a shape that runs the other way.** Nested plain `DO`s -- `do` / `do` / `end` / `end` and nothing else, outer on line 5 and inner on line 6 -- give oracle `G ran 5` and this crate `G ran 6` on both engines. That is the outer `DO`'s boundary arriving after the inner one, and it excludes both alternatives on its own: an absent boundary would deliver at the `say` after the loop, and a correctly placed one would deliver at line 5. Capture it.

  **A `DO UNTIL` whose condition raises delivers EARLIER here than on the oracle, which no other shape in this plan does.** With `do until raiser1() > 0` on line 4, `nop` on 5, `end` on 6 and `say 'after' zv` on 7, where `raiser1` raises the first trapped condition and its handler requeues a second: the oracle prints `after set` then `G ran 7`, and both engines here print `G ran 6` then `after set`. Not the same direction as everything above, so do not assume one fix moves it -- measure it, and if this task does not close it, say so.

  **CORRECTED: this line read `after unset` when it was written, and the oracle prints `after set`.** `unset` is not reachable on this program: the handler that requeues is the one that assigns `zv`, so anything that prints `after` at all prints it after the assignment. Measured with a marker added to that handler, the oracle's whole transcript is `H ran 6` / `after set` / `G ran 7` -- the *first* delivery agrees with this crate at line 6 and only the second moves, which the uncorrected line hid by making both look wrong.

- [x] **Step 1c: the second mechanism, which has its own named program and is NOT the one above.** The whole-branch review found it and the controller reproduced it byte for byte. An empty `do` / `end` with a *double* requeue -- a handler that requeues, whose own handler requeues again -- with `do` on line 5, `end` on 6 and `say 'after' zn` on 7:

```rexx
call on user zx name h
call on user zy name g
call on user zw name k
zn = raiser1()
do
end
say 'after' zn
exit
raiser1:
raise user zx return 5
h:
raise user zy return 1
g:
say 'G ran' sigl
raise user zw return 1
k:
say 'K ran' sigl
return
```

```
oracle       G ran 5 / K ran 6 / after 5
this crate   G ran 5 / after 5 / K ran 7     (both engines)
```

The first delivery agrees. The **second** is owed at the `END`'s own clause -- `K ran 6` names it -- and this crate has no boundary there at all, so it falls past `say 'after'` and lands at line 7, after that line has printed. **This is the elided-`END` mechanism: a boundary that is genuinely absent, not misplaced.** So `DO` carries two distinct defects, and the empty-body control in Step 1 pins only the first: it agrees for one requeue and diverges for two.

Pre-existing, confirmed by a control build with every behavioural change of this plan reverted. **Decide whether this task closes one mechanism or both, and say which.** A fix for the header clause's position does nothing for a missing `END` boundary.

- [x] **Step 2: decide whether the plain `SELECT` case is the same fix or a different one.** The two are inverted, which is why they are probably two fixes: a plain `SELECT` has its boundary in the right *place* and settles it at the wrong *level* (`SIGL` already names the right clause; only the indent moves), because `settle_block_indent` lives in `Interp::select_case` and a caseless `SELECT` never reaches it. A plain `DO` has the right level and the wrong place. Task 5 recorded both. **These may be one defect or two, and Task 5's own experience is that the "same defect one construct over" hypothesis was refuted once already.** Measure before assuming; if they are one, both transcripts must move together, and if they are two, say which this task closes.

- [x] **Step 3: write the failing test**, comparing stdout and exit status with no `TRACE` in the program. Run it and confirm it fails.

- [x] **Step 4: fix, and confirm `if ... then` and the `WHEN`-condition case did not move.** They are the adjacent successes for this route.

- [x] **Step 5: correct the two records** -- the `DO` entry in `ir_dual_cases/loop-header-boundaries`, and the plain-`SELECT` entry if Step 2 closed it too. They state open divergences and become false.

- [x] **Step 6: the mutation witness, the raw blast-radius sweep and the gates.** The sweep here must include stdout, not only stderr: this is the one defect in this plan whose blast radius is program output.

**OUTCOME, `e74780054`. This task closed BOTH mechanisms, and the answer to Step 2 is that the plain `SELECT` is a different fix and is untouched.**

The rule the fix lands: a plain `DO` has a clause at its header and a clause at its `END`, and the step that spans the whole construct has neither. `run_loop_with_header`'s `Simple` arm opens the header's clause and closes it before `run_bounded`, and opens `END`'s on the pass that reaches it; `leave_stepped_clause` gives a `DO`/`LOOP` step no boundary of its own, because a boundary there is one the oracle does not have and it delivers at whichever line the last inner clause left behind.

**That third part is what the `DO UNTIL` needed, and Step 1b's prediction that it might not follow was right to make.** Its `END` boundary is already served by the `UNTIL` test, so the requeue is owed to the clause *after* the loop; only removing the step's own boundary moves it. The first two changes alone leave it wrong.

**What the measurements said that this task's text did not.**

* **The empty-body control in Step 1 agrees for a reason weaker than it looks.** With one requeue and an empty body the delivery lands on the `DO`'s own line because there is no body clause to update the clause line first, not because the boundary was correctly placed. The same block with one body clause reports the body's line. So the control pins "a boundary exists and carries a line", not "the header clause ends before the body".
* **Every loop shape measured on this route was wrong, not only `Simple`, and `run_repeating`'s per-pass clause is not what makes the repeating ones right.** Re-measured 2026-08-15 against the oracle, on both engines, every row diverging at `1f4176b47` and agreeing at `d0b7504a5`:

  | program | oracle, and `d0b7504a5` on both engines | `1f4176b47`, both engines |
  |---|---|---|
  | `do zi = 1 to 1 / zr = raiser() / end / say 'after' zr` | `h1 5` `h2 6` `after 5` `h3 7` | `h1 5` `h2 6` `h3 6` `after 5` |
  | `do 1 / zr = raiser() / end / say 'after' zr` | `h1 5` `h2 6` `after 5` `h3 7` | `h1 5` `h2 6` `h3 6` `after 5` |
  | `do while zn < 1 / zn = zn + 1 / zr = raiser() / end / say 'after' zr` | `h1 7` `h2 8` `after 5` `h3 9` | `h1 7` `h2 8` `h3 8` `after 5` |
  | `do forever / zr = raiser() / leave / end / say 'after' zr` | `h1 5` `h2 6` `after 5` `h3 8` | `h1 5` `h2 6` `h3 6` `after 5` |
  | `zr = raiser() / do label zl / say 'body' / end / say 'after' zr` | `h1 3` `h2 4` `body` `after 5` | `h1 3` `body` `h2 5` `after 5` |

  In the first four rows the raise happens inside the loop body, and `h2` leaves `c3` queued behind it. `run_repeating`'s per-pass clause is not what makes them right: it was already there at `1f4176b47`, in the shape it has now (`run_repeating`'s `header_line` match on `HeaderClause`), and those rows still diverged. A clause boundary drains what is queued when it runs, so the condition a handler requeues is owed to the *next* boundary -- and at `1f4176b47` that next boundary was the step's own, because `leave_stepped_clause` ran `leave_clause` for every construct. That is what those rows read back: `h3` lands on the line the loop's last inner clause left behind, ahead of the `SAY` the oracle delivers it at.

  The last row is a different mechanism and is not a repeating loop at all. `do label zl` with no header parses as `LoopKind::Simple` through `create_loop`'s bare at-end-of-clause arm (`crates/rexx-parse/src/instruction.rs:960-975`), so it never reaches `run_repeating`. Its handler chain stops at `h2`, so the only delivery that moves is `h2`'s: the oracle owes it to the `DO`'s own line, and at `1f4176b47` the `Simple` arm opened no clause at all -- neither at the `DO`'s line nor at `END`'s -- so it fell through to the first clause inside the block.
* **`if ... then do` diverged too, and it is this defect rather than a neighbour.** Both deliveries moved, and both now agree. Plain `if ... then` with no block, and `select` / `when ... then` / `end`, agreed before and after.

**Found and NOT fixed, all confirmed pre-existing by a before/after control build.**

1. **The plain `SELECT` boundary recorded in `ir_dual_cases/loop-header-boundaries` is unchanged**, verified by re-running its own program after this fix landed: still two columns short on both engines, stdout and `rc` agreeing. Two defects, as that record already said.
2. **A handler delivered at an `IF`'s or a `SELECT`'s own step boundary traces too deep**, which is the same indent axis as (1) with the opposite sign. `if 1 = 1 then zr = raiser()` under `trace r` puts the handler's activation four columns in from the oracle's, and the `select` / `when ... then` shape six. `SIGL` and stdout agree in both; only the trace indent differs. Same structural cause as this task's -- a step boundary the oracle does not have -- but on constructs this task did not touch.
3. **A stem `DO OVER` is refused where the oracle runs it.** `do zi over zz.` exits `rexx-exec: DO is not implemented` at `rc 120` against the oracle's `rc 0`. `loop_header_plan` answers `None` for it by design and `run_loop_with_header` raises there for both engines; this is the known refusal rather than a new divergence, and it is written down because a probe reached it.

**Witnesses.** Each of the three changes was mutated separately, keeping the indent and the echo so that only the boundary moved. Every one reddens `corpus_differential` and `ir_dual`; the suite with `lang/do_clause_boundaries.rex` removed from the subset catches none of them, so all three are load-bearing and the program adds coverage rather than merely being able to fail. A raw before/after sweep of every program under `corpus/` and `bench-programs/`, on both engines and all three descriptors, moves nothing but the new program. **That sweep runs only the programs those two directories hold, so it can witness a regression only in a shape one of them already contains.** It did not witness the one this task created: inside an `INTERPRET` fragment the new header and `END` clauses are boundaries where the oracle offers a condition queued before the fragment none at all, and that shape was found by review instead. It is closed by the 2026-08-15 plan's Task 1, whose own before/after sweep over the same two directories, on both engines and all three descriptors, likewise moves nothing -- which is the measurement saying this sweep cannot see the shape in either direction.
