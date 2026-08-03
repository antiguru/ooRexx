# Phase 4b implementation plan: procedures and conditions

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** run a classic Rexx program that calls internal routines, isolates and exposes their variables, traps conditions and interprets text, byte-for-byte as `build/bin/rexx` runs it.

**Architecture:** 4a built one activation that runs one body. 4b makes the activation stack real: a body selector on `Activation`, one Rust frame per activation with an explicit depth counter, settings and trace mode inherited per frame, exposed names bound to caller slots at call time, and an error report that carries a stack of sites rather than one. No new crate; `rexx-exec` grows.

**Tech stack:** Rust 1.96.1, no `unsafe`, `cargo fmt` default, `clippy -D warnings`. Depends on `rexx-core`, `rexx-num`, `rexx-parse`, `rexx-inventory`.

## The governing documents, and what each is for

* **`docs/superpowers/specs/2026-08-01-phase-4bc-scoping.md`** is the material this plan was written from.
  Its 37 inherited items are reproduced in the task bodies below, because a per-task brief extraction makes anything outside a task's own section invisible.
  **Do not read it to find your requirements.** Everything a task needs is in that task.
* **`docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md`** still governs the value model, the borrow shape, and decisions D15 to D19.
  Read the section a task names; do not read the whole spec.
* **`docs/superpowers/plans/phase-4-exclusions.txt`** is the live record of what Phase 4 does not do.
  Several tasks below add rows to it. Adding a KNOWN GAP row needs no permission; removing one does.
* **`docs/superpowers/plans/phase-4a-gate.md`** is the criterion set this sub-phase's gate is derived from.

---

## Global constraints

Every task's requirements implicitly include this section.

* **The C++ tree is the oracle and is never modified.**
  `interpreter/`, `samples/`, `build/`, `ootest/` are read-only.
  Every behavioural question is settled by running `build/bin/rexx`, not by reading the ANSI standard.
* **Wrap every oracle invocation** as `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`.
  The interpreter requests gigabytes mid-range and gets OOM-killed otherwise, which has already cost a session.
* **Read stdout and stderr as separate descriptors.**
  Comparing `2>&1` as one string produced two false regressions in 4a: the interleaving of the trace sink and stdout is unobservable by design (D17), so a combined capture compares something the design does not define.
* **Never set `NUMERIC DIGITS` above 1000** in a probe.
* **Never instantiate `.Package~new`** on a file inside the repository: it executes that file's prolog and has written untracked files into the tree.
* **Never probe `select; when 1 = 0 then; when 2 = 2 then nop; end`.** It segfaults the oracle (upstream SF #2018).
* Scratch files go in the session scratchpad, never in the repository.
* **No `unsafe`.** The workspace sets `unsafe_code = "forbid"`. If a task appears to need it, stop and report BLOCKED.
* **Never `git add -A`.** Stage the exact paths the task names. Do not run `git reset --hard`, do not force-push.
* Comments state the contract at the top and the reasoning at the decision point.
  Never delete an existing comment to make a change easier.
  Prefer `--` over an em-dash, matching what the tree already does everywhere.
  The "no structuring semicolons" rule does **not** apply to this repository; it was imported by mistake in 4a, enforced through several reviews, and withdrawn.
  Reviewers should not raise it.
* A value's rendering is fixed when the value is created.
  Any code that formats a number with `settings.digits()` or `settings.form()` instead of the value's own captured pair is wrong; see D15.
* Anything Phase 4b does not implement **fails loudly**: `NOT_IMPLEMENTED_EXIT`, outside 157..253 where `256 - major` lives, and a message naming the construct and the owning sub-phase.
  Never a plausible Rexx condition.
* **Test a private subject with a `#[cfg(test)] mod tests` beside it, not an integration test.**
  `Interp` and its methods are private to `rexx-exec`.
  Integration tests are for the public surface: the runner, the corpus harness, the gate harnesses.
* **Every new allocation site goes through `Interp::alloc_with`.**
  `Heap::alloc_with_uncollected` and `Heap::alloc` both bypass the collect-on-every-allocation stress hook, and the first is named that way so a site written the natural way announces the bypass at the call site.
  "Exactly four call sites, verified by grep" was true on the day it was written and is exactly the kind of fact that goes stale silently.
* **Commit first, then read the hash back, then record it.**
  Two ledger entries in 4a carried invented commit hashes because the hash was written from memory before the commit existed.
* `rustfmt` needs `--edition 2024`.
  The bare form defaults to 2015 and rejects let-chains.
* A shell pipeline reports the *last* command's status.
  `cargo clippy | tail` in an `&&` chain reports `tail`'s success. Read exit status unpiped.

---

## Decisions taken for 4b

The scoping document left fourteen decisions open.
Nine bind 4b and are settled here.
Five bind only 4c (D4 builtin exclusions, D7 `ExprKind::List`, D8 the `rexxcps` gate, D11 `RANDOM`/`DATE`/`TIME` determinism, D12 the L1 extractors) and are deferred to the 4c plan, with one exception noted under D7 below.

### D1 — the Task 3 spike is replaced, not extended

`run_program_interpret_spike` and `Interp::interpret_spike` are deleted, and `tests/spike.rs`'s three fragment tests move onto `run_program`.
This is what the field's own doc says: "4b's first move here is to delete this field and the branch that reads it".
Keeping it would be two entry points into one machinery.
Task 1 owns this.

### D2 — the error report carries a stack of sites, and it is built before `CALL`

Measured 2026-08-03, and these bytes exist nowhere else in the tree.
A raise inside a routine called from line 1:

```
     4 *-*   say 2 & 1
     1 *-* call sub
Error 34 running /path/p9.rex line 4:  Logical value not 0 or 1.
Error 34.901:  Logical value must be exactly "0" or "1"; found "2".
```

Three levels:

```
     7 *-*     say 2 & 1
     4 *-*   call inner
     1 *-* call outer
```

Two properties, both of which the recorded KNOWN GAP omits:

* **Each echo carries its own activation's line number**, innermost first.
  This differs from `INTERPRET`, where both echoes carry the enclosing clause's line.
* **Each activation adds two spaces of indent**, on top of the lexical indent `static_indent` already computes.
  Measured separately: a raise inside a `DO` body inside one called routine prints at indent 4, which is `static_indent`'s 2 plus one activation's 2.
  Source leading whitespace is irrelevant — every probe above was written with two leading spaces in the callee and the indents still came out 0/2/4 by depth.

So the activation base is **added outside `static_indent`**, and `static_indent` keeps its property of being a pure function of the flat instruction list.
Task 2 builds the stack; Task 3 pushes activation entries onto it.

### D3 — the `DO OVER` on a stem deviation stands, and its corpus rule carries into 4b

The deviation is about traversal order: the oracle walks a balanced tree, we use a hash map.
`PROCEDURE EXPOSE` changes stem *identity*, not iteration order, so it does not touch the deviation's premise.
**No corpus program may contain `DO OVER` on a stem**, in 4b's subset as in 4a's.

### D5 — 4b runs strictly before 4c, in one lane

4c depends on 4b in six named places; 4b depends on 4c nowhere.
Both sub-phases' code lands in `eval.rs`, `run.rs` and `lib.rs`, and every scheduling collision recorded in the 4a ledger was two agents in one file.
Never dispatch two implementers in parallel against this plan.

### D6 — a new corpus subset file per sub-phase

`rust/corpus/phase-4b.txt` is created beside `phase-4a.txt`.
The three harnesses that read a subset (`tests/corpus.rs`, `tests/coverage.rs`, `tests/collect_stress.rs`) each hardcode the 4a filename at their call site; all three change to read a list of subset files and use the union.
Growing `phase-4a.txt` instead would destroy the ability to say which sub-phase a regression belongs to.
Task 10 owns this.

### D9 — exposed names bind to caller slots at call time, and this is measured rather than chosen

The scoping document offered two options and recommended slot binding on invariant-preservation grounds.
Measured 2026-08-03, slot binding is **forced**, because `EXPOSE` aliases the caller's *variable entry* and not the stem *object*:

| Probe | Callee body | Caller prints | What it settles |
|---|---|---|---|
| p3 | `a.1 = 'changed'` | `changed` | tail writes propagate |
| p2 | `a. = 'wiped'` | `wiped` | whole-stem assignment propagates |
| p1 | `drop a.` | `A.1` | **drop propagates** |
| p7 | `b. = a.` in caller, `drop a.` in callee | `A.1 orig` | the drop rebinds the entry; `b.` still holds the old object |
| p4 | caller never mentions `zzz`; callee `procedure expose zzz; zzz = 'set-by-callee'` | `set-by-callee` | the slot must live in the caller and survive the return |

p1 is the discriminating probe.
Under an object-sharing model the callee's `drop` would rebind only the callee's own name and the caller would still print `kept`; it prints `A.1`.
p7 then shows the drop does **not** clear the shared object, because `b.` — bound to the same object by an earlier `b. = a.` — still reads `orig`.
So `stem_drop`'s existing shape (`replace_stem(name, None)`, put a fresh stem in the slot) is correct under exposure and must not change.

p4 is what makes slot binding non-trivial: the caller's plan has no slot for a name the caller never mentions.
The binding therefore happens **at the `CALL` instruction, while the caller's frame is still the top frame**, so `RootSet::grow_slots`'s top-frame invariant survives untouched.
The callee's plan carries its exposed-name list; the `CALL` resolves it against the caller and grows the caller's frame as needed before pushing the callee's.

The indirect form resolves against the caller's pool.
Measured: `v = 'zzz'` and `zzz = 'caller-value'` in the caller, `sub: procedure expose (v)` in the callee, prints `inside caller-value` then `after callee-set`.
**Two hypotheses fit those bytes** — "the name list is read from the caller before isolation" and "`PROCEDURE` executes in the callee before the pool is isolated" — and no probe distinguishes them, because `PROCEDURE` must be the first instruction of a routine so the callee can never have its own `v`.
They agree operationally. Implement either; do not claim the measurement chose one.

### D10 — D19's per-activation Rust recursion is confirmed

One Rust frame per activation plus an explicit counter, raising 11.1 at rc 245.
Reopening this reopens the `Rc<Program>` risk D19 closed: the flat-loop variant is where the program local must be re-derived at every frame transition.
This is recorded here because it is the decision most likely to be re-litigated by an implementer who has not read D19.
Task 3 owns the counter, and owes a measurement of the *combined* budget, because `run_bounded` already costs a Rust frame per source nesting level and nobody has measured the two together.

### D13 — 4b gets its own gate

Its own plan, its own gate document, its own corpus subset.
This matches how 4a ran and keeps D6 coherent.

### D14 — the 4a criterion set carries forward with two amendments

* Criterion 2's wording contemplates a blocked assertion row being unblocked by "4b or 4c" and never names Phase 5.
  All 35 exempt rows need Phase 5.
  **4b's gate must state this up front rather than discover it**, and must not promise movement in `tests/assertions.rs`.
* Criterion 3's trace table has no measure of its own coverage.
  Four divergences were found in 4a by probing adjacent shapes rather than by the table.
  4b adds trace prefixes to that unmeasured surface, so criterion 3 gains a coverage measure of its own.

---

## Corrections to inherited items, made by measurement for this plan

Three inherited items are wrong or overstated.
They are corrected here rather than in the scoping document, because that document is a snapshot and this plan is what gets executed.

### `>I>` and `<I<` are not 4b's, on the evidence available

Item I14 assigns six trace prefixes to 4b and 4c by reading `RexxActivation.cpp:3567-3588`.
Measured 2026-08-03 at `trace i`, an internal `CALL` emits **no `>I>` and no `<I<`**:

```
     2 *-* call sub 1
       >L>   "1"
       >A>   "1"
     4 *-*   sub:
     4 *-*   procedure
     5 *-*   use arg a
       >>>     "1"
       >=>     A <= "1"
     6 *-*   return a
       >V>     A => "1"
       >>>     "1"
       >>>   "1"
     3 *-* exit
```

An expression-form function call emits `>F>` and still no `>I>`/`<I<`:

```
     2 *-* say f(1)
       >L>   "1"
       >A>   "1"
     4 *-*   f:
...
       >F>   F => "2"
       >>>   "2"
```

So on this evidence 4b owns `>A>` (ARGUMENT), `>F>` (FUNCTION, expression-call form only) and `>R>` (ALIAS, unmeasured).
`>I>`/`<I<` INVOCATION/EXIT are most likely method invocation, which is Phase 5's.
**Task 9 must settle this by measurement and write the answer into the exclusions file**, either as a 4b obligation or as a reassignment.
Reading the C++ enum told us the prefixes exist; it did not tell us which construct emits them.

### I17's premise is false, and its conclusion is unproven

I17 says a mutant rerouting `stem_drop` to a slot clear is a genuine equivalent mutant "until something can hold a second reference to the old stem object", and that nothing in 4a can.

Measured 2026-08-03 on both interpreters: `a.1 = 'orig'; b. = a.; a.1 = 'new'; say a.1 b.1` prints `new new`.
Stem assignment shares the object, in 4a, with no `CALL` anywhere.
So the premise is false.

But the conclusion does not follow either way: `a.1 = 'orig'; b. = a.; drop a.; say a.1 b.1` prints `A.1 orig` under both the current code and the mutant, because "slot holds a fresh empty stem" and "slot is unset" are not distinguishable through a second reference.
**4b must not carry I17 forward as "becomes pinnable when 4b lands".**
Task 5 either produces a program that distinguishes the two, or reclassifies the mutant as genuinely equivalent with the reason written down.
Reporting it as an expected survivor with a false explanation is worse than either.

### The corpus files contradict the ruling about `ExprKind::List`

`corpus/phase-4a.txt:18` and `corpus/README.md:108-109` both say the three dropped `num/` programs return "for 4b or 4c, once `List` exists".
The later Task 16 ruling in `phase-4-exclusions.txt:176-179` assigns `List` to **Phase 5**.
The ruling is correct — measured, `(1, 2)` is an `Array` instance — and the corpus files are what a 4b author reads first.
Task 10 corrects both comments to say Phase 5.
This is the one 4c-adjacent decision resolved in this round, because leaving it would send a 4b implementer looking for work that is not theirs.

---

## File structure

```
rust/crates/rexx-exec/
  src/lib.rs          Interp, the plan cache; trace_mode leaves here (Task 3)
  src/activation.rs   gains a body selector, a settings-inheriting constructor,
                      trace_mode, and the exposed-name binding
  src/plan.rs         BodyKey::directive gets its first setter; a body's
                      exposed-name list is precomputed here
  src/error.rs        Raised gains a site stack; Raised::condition gains a reader
  src/run.rs          CALL, RETURN, PROCEDURE, USE, SIGNAL, RAISE, INTERPRET,
                      PUSH, QUEUE, the condition-trap table, the depth counter
  src/eval.rs         ExprKind::Call's internal-routine front,
                      ExprKind::VariableReference
  src/queue.rs        NEW: the in-process external data queue
  src/trace.rs        >A>, >F>, >R>, and the activation indent base
  tests/              corpus.rs, coverage.rs, loud.rs, collect_stress.rs,
                      trace_oracle.rs all gain 4b rows; spike.rs loses its
                      three fragment tests to lib.rs
  tests/owners.rs     NEW: the owner table coverage.rs and loud.rs share
rust/corpus/phase-4b.txt   the named 4b subset
rust/corpus/proc/*.rex     new 4b programs
rust/scripts/mutate-4b.sh  4b's mutation set, reusing 4a's guard mechanism
docs/superpowers/plans/phase-4b-gate.md
```

`rexx-core` is amended in no task.
That is a deliberate outcome of D9: binding exposed names at call time keeps `RootSet::grow_slots`'s top-frame invariant, so its `4a invariant` panic and the `#[should_panic(expected = "4a invariant")]` test in `crates/rexx-core/tests/collect.rs:163-181` both stand unchanged.
If a task finds it must relax that panic, that is a plan change: stop and report BLOCKED rather than editing the message.

---

### Task 1: A real `INTERPRET`, replacing the Task 3 spike

**Spec:** design spec's "The borrow shape", the paragraph on the fragment lifetime.

**Files:**
- Modify: `rust/crates/rexx-exec/src/lib.rs` (delete `run_program_interpret_spike` around line 696 and `Interp::interpret_spike` around line 1005; add a `#[cfg(test)] mod tests` for the fragment-lifetime proofs)
- Modify: `rust/crates/rexx-exec/src/run.rs` (the `InstructionKind::Interpret` arm)
- Modify: `rust/crates/rexx-exec/src/plan.rs` (fragment plan construction)
- Modify: `rust/crates/rexx-exec/tests/spike.rs` (remove the three fragment tests at lines 122, 150, 300; change the loud fixture at line 187)
- Modify: `rust/crates/rexx-exec/tests/coverage.rs`, `tests/loud.rs` (the `Interpret` owner arm and its witness)

**Interfaces:**
- Consumes: `Fragment { source, body: CodeBody, symbols }` from `rexx-parse`; `Interp::alloc_with`; `step_in_temps_frame`.
- Produces: nothing public. `run_program` gains the behaviour; no new public item.

**Why:** the spike's public entry point exists only because a private subject needed an integration test, and its doc comment records that a `#[cfg(test)] mod tests` inside `lib.rs` could prove the same lifetime with no public surface at all. That trade is remade here rather than inherited (D1).

**Inherited items this task pays for:**

* **I7.** `run_program_interpret_spike` and `Interp::interpret_spike` are deleted, and the three tests at `tests/spike.rs:122,150,300` move to a `#[cfg(test)] mod tests` in `lib.rs`.
* **I8.** Fragment plans are built, used and dropped, and **they stay that way.** Revision 6's `(enclosing body, fragment id)` cache key was withdrawn as "sound and useless": fragment text varies per execution, so every lookup misses while every entry is retained. A cache keyed by fragment *text* is permitted only if this task measures a hit rate on a real program and reports it. Absent that measurement, do not add one.
* **I9.** `Fragment::body.labels` is always empty and needs no label table. Measured and settled in 4a's Task 1: a label inside `INTERPRET` text is error 47.1 both ways. Do not add label handling to the fragment path.
* **I16.** `step_in_temps_frame` is the single chokepoint that heals six `push_frame` sites in `eval.rs` which skip their `pop_frame` on the `?` path, and it is the **only** caller of `step` in the crate. The 4a investigation's conclusion that `SIGNAL ON SYNTAX` cannot accumulate temps leaks rests entirely on that: a trap acts at instruction-loop level, and the wrapper has already truncated before the `Failure` reaches the loop's `Err` arm. **If this task moves execution off that chokepoint, say so in the report** — the whole analysis must then be redone rather than assumed, and Task 7 depends on it.
* **I21.** Any allocation this task adds goes through `Interp::alloc_with`, not `Heap::alloc_with_uncollected` and not `Heap::alloc`.
* **I22.** `pop_frame`'s truncation semantics are load-bearing. **No assert may be added there** without balancing the six `eval.rs` sites first. An optional debug tripwire in `step_in_temps_frame` asserting temps balance on the `Ok` path was scheduled in 4a and not built; it needs a `temps_len()` accessor. Building it is welcome; changing `pop_frame` is not.

- [ ] **Step 1: Move the three fragment-lifetime tests into `lib.rs`**

Read `tests/spike.rs:122`, `:150` and `:300`. Each proves something about a fragment's lifetime against the borrow shape. Reproduce each as a `#[test]` inside a `#[cfg(test)] mod tests` at the bottom of `lib.rs`, driving `Interp` directly rather than the public entry point. Keep each test's existing doc comment verbatim: they record *why* the lifetime is what it is.

- [ ] **Step 2: Change the loud fixture at `tests/spike.rs:187` before it breaks**

That test uses `call "sub"` and asserts both `NOT_IMPLEMENTED_EXIT` and that stderr contains `"CALL"`. Its own doc comment says "`CALL` is 4b's" — it breaks on Task 3. This is the fourth occurrence in this project of a witness implemented out from under a test.

Replace the fixture with a **message send** (`q~append(1)`), which the spec assigns to Phase 5 outright rather than by ruling, and update the stderr assertion to match. Add a one-line comment saying why a message send and not `PARSE` or `ADDRESS`: those are 4c's and would break again in three weeks.

- [ ] **Step 3: Run the two tests to see them pass before anything else changes**

Run: `cargo test -p rexx-exec --test spike`
Expected: PASS. This is a refactor-in-place checkpoint, not a TDD red.

- [ ] **Step 4: Write the failing `INTERPRET` test**

In `run.rs`'s `#[cfg(test)] mod tests`, or as a corpus program if the shape is better as one:

```rust
#[test]
fn interpret_binds_a_name_the_enclosing_body_never_mentions() {
    // Measured on the oracle in 4a: the binding outlives the fragment.
    let out = run_source(b"interpret \"zork = 42\"\ninterpret \"say zork\"\n");
    assert_eq!(out.stdout, b"42\n");
    assert_eq!(out.exit, 0);
}
```

- [ ] **Step 5: Run it and watch it fail**

Run: `cargo test -p rexx-exec interpret_binds_a_name`
Expected: FAIL, with the loud not-implemented message naming `Interpret` and `4b`.

- [ ] **Step 6: Implement the `InstructionKind::Interpret` arm**

The expression is evaluated to a string, parsed as a `Fragment`, given a plan, and executed against the **current** activation: same frame, same slots, same settings. A name the enclosing plan never saw goes into `Activation::extra`, which exists for exactly this and which `DROP (v)` will also use.

The fragment's instruction list is a separate flat list, so `pc` cannot simply continue into it. Execute the fragment through the same bounded sub-loop shape `run_bounded` uses for a block, and forward an unowned `Flow` outward: a `SIGNAL` out of a fragment, a `LEAVE` targeting an enclosing loop, and `EXIT` all have to escape it. `RETURN` inside a fragment is Task 3's problem, not this task's — leave it failing loudly and say so in the report.

- [ ] **Step 7: Run the test, and the whole suite**

Run: `cargo test -p rexx-exec`
Expected: the new test passes; `loud.rs`'s `Interpret` row now fails because the construct works. Fix that row in the next step, not by weakening the test.

- [ ] **Step 8: Move `Interpret` from out-of-scope to in-scope in both owner tables**

`coverage.rs` and `loud.rs` each carry the owner table **by hand** (I36). Both change. `coverage.rs`'s in-scope tag then requires a witness program in the subset, or `every_in_scope_variant_is_witnessed_by_the_phase_4a_subset` fails; `variant_counts_match_the_audited_split` asserts 20/9/4/6/1 for `InstructionKind`, so those numbers move too. Task 10 makes the shared table that stops these two drifting; until then, edit both and check they agree.

- [ ] **Step 9: Verify the exclusions file**

`phase-4-exclusions.txt` has an EXPRKIND OWNERSHIP section and a KNOWN GAPS section. `INTERPRET`'s one-echo-per-nesting-level gap stays open until Task 2 — do not close it here.

- [ ] **Step 10: Commit**

```bash
git add rust/crates/rexx-exec/src/lib.rs rust/crates/rexx-exec/src/run.rs rust/crates/rexx-exec/src/plan.rs rust/crates/rexx-exec/tests/spike.rs rust/crates/rexx-exec/tests/coverage.rs rust/crates/rexx-exec/tests/loud.rs
git commit -F <message file>
```

---

### Task 2: The error report carries a stack of sites

**Files:**
- Modify: `rust/crates/rexx-exec/src/error.rs` (`Raised`, around lines 36-55, and `Raised::report`)
- Modify: `rust/crates/rexx-exec/src/run.rs` (`record_failure_site`, and its callers around lines 1199, 1316, 1368)
- Modify: `rust/crates/rexx-exec/tests/` — a new differential witness

**Interfaces:**
- Produces: `Raised` carries `sites: Vec<FailureSite>` rather than one `Option<FailureSite>`, innermost first, each entry carrying its own line number and its own indent base. `Raised::report` walks it.
- Consumes: `static_indent` from `run.rs`, unchanged.

**Why:** every raise inside a routine differs from the oracle on stderr until this exists, which is most of what a 4b differential corpus contains. Building it after `CALL` means every corpus program written in between is unverifiable.

**The measured shape, which exists nowhere else in the tree.** For `INTERPRET`, both echoes carry the *enclosing clause's* line. For `CALL`, each echo carries *its own activation's* line, and each activation adds two spaces of indent:

```
     7 *-*     say 2 & 1
     4 *-*   call inner
     1 *-* call outer
Error 34 running /path/p10.rex line 7:  Logical value not 0 or 1.
Error 34.901:  Logical value must be exactly "0" or "1"; found "2".
```

Line numbers are right-aligned in a six-column field, then ` *-* `, then the indent. The `Error NN running` line names the **innermost** line. rc 222.

**The indent is `static_indent` plus an activation base, and the base is added outside `static_indent`.** Measured: a raise inside a `DO` body inside one called routine prints at indent 4 — `static_indent`'s 2 for the block, plus 2 for the one activation. Source leading whitespace contributes nothing; every probe above had two leading spaces in the callee's source and still produced 0/2/4 by depth. Task 11 of 4a built `static_indent` as a pure function of the flat instruction list, and that property survives only if the base stays outside it.

**Inherited items this task pays for:**

* **I12.** The KNOWN GAP at `phase-4-exclusions.txt:224-245` records only "one echo per nesting level, innermost first". It is closed here for `INTERPRET` and by Task 3 for `CALL`. Do not close the row until Task 3 lands; amend it to say which half is done.
* **I11.** `Interp::failure_site` is never cleared mid-run, and it is set first-call-wins (`self.failure_site.is_none()` guards both callers). It matters only once a condition trap can resume execution after a raise, which is Task 7. **Do not fix it here and do not delete the guard** — changing the first-call-wins rule without a resuming caller changes which site an untrapped error reports. Leave a comment on the new stack saying Task 7 owns the clearing.

- [ ] **Step 1: Write the failing test from the measured `INTERPRET` transcript**

Take the two-level `INTERPRET` case from the KNOWN GAP row. Capture the oracle's exact stderr with the wrapper, commit it as an expectation in the same shape `tests/trace_oracle.rs` uses (its module doc carries the regeneration command), and assert byte equality.

- [ ] **Step 2: Run it and watch it fail on the missing second echo**

Expected: FAIL, one echo where two are expected.

- [ ] **Step 3: Change `Raised` to carry a stack**

`FailureSite` gains the fields the echo needs: the line number to print and the indent base to add. `Raised::report` walks innermost-first. The existing single-site behaviour must fall out as the one-element case, byte-identically — 4a byte-verified the report on eleven programs and all eleven must still pass.

Do **not** resolve the stack at report time by walking `Interp::activations`. `run` pops the activation before `execute` sees the error, which is the reason `failure_site` exists at all; walking it would need the pops to stop or the stack to be snapshotted, which is this design with extra steps.

- [ ] **Step 4: Push a fragment entry at `INTERPRET` entry**

The fragment shares its caller's indent — measured, in the three-deep probe the `INTERPRET` level's indent equalled the `CALL` level's below it — and carries the enclosing clause's line.

- [ ] **Step 5: Run the new test and the eleven existing report witnesses**

Run: `cargo test -p rexx-exec`
Expected: all pass.

- [ ] **Step 6: Amend the KNOWN GAP row**

Say the `INTERPRET` half is closed at this commit and the `CALL` half is Task 3's. Record the two measured properties — per-activation line number, `+2` indent per activation — **in the row**, because they are what the next reader needs and rediscovering an indent rule cost 4a's Task 11 a whole fix round.

- [ ] **Step 7: Commit**

---

### Task 3: The body selector, `CALL` to an internal label, and `RETURN`

**Spec:** design spec's "The borrow shape" (line 305 on `Activation::new`), and D19 at line 284/522.

**Files:**
- Modify: `rust/crates/rexx-exec/src/activation.rs` (the body selector beside `program` at lines 44-52; a sibling constructor to `Activation::new` at lines 83-100; `trace_mode`)
- Modify: `rust/crates/rexx-exec/src/plan.rs` (`BodyKey::directive` at lines 64-66)
- Modify: `rust/crates/rexx-exec/src/lib.rs` (delete `Interp::trace_mode`, lines 586-602)
- Modify: `rust/crates/rexx-exec/src/run.rs` (`run_activation`'s hardcoded `&program.main`; the `Call` and `Return` arms; the depth counter)
- Modify: `rust/crates/rexx-exec/src/error.rs` (push an activation site)
- Modify: `rust/crates/rexx-exec/tests/coverage.rs`, `tests/loud.rs`

**Interfaces:**
- Produces: `Activation::for_call(program, plan, frame, caller_settings, caller_trace) -> Activation`, and a body selector field whose `None` is the main body and whose `Some(i)` is `directives[i]`'s body — the same shape `BodyKey::directive` already carries.
- Produces: `Interp::depth: usize`, and the 11.1 raiser.
- Consumes: Task 2's site stack.

**Why:** `run_activation` hardcodes `&program.main`. That is true for every activation 4a can build and false the moment a callee runs, and the failure is silent and with the right program.

**Inherited items this task pays for:**

* **I1.** The body selector goes beside `Activation::program` at `activation.rs:44-52`. That field's doc already describes the gap; update it rather than leaving a stale note.
* **I2.** `BodyKey::directive` at `plan.rs:64-66` is `Some(index)`-shaped and nothing has ever set it. **Decide it together with I1 or the two will disagree** — the activation's selector and the plan cache's key must denote the same thing.
* **I3.** `Activation::new` unconditionally defaults `Settings`. Measured in 4a: with `numeric digits 7`, an internal `call sub` sees 7, sets its own to 3, and after `return` the caller still reports 7. The sibling constructor inherits from the caller. Its doc says explicitly that folding both into one function would need a parameter that is `None` on every 4a path — that is no longer true, but keep them separate anyway: a fresh top-level run and a nested call begin from different starting settings and the two-constructor shape says so.
* **I4.** `Interp::trace_mode` moves onto `Activation` and is deleted from `Interp`. A deliberate 4a-only simplification, not an oversight: 4a has one frame, and measured, a callee's `trace off` does not survive its `return`. The field's own doc at `lib.rs:586-602` names this as "4b's first move here".
* **I6 and D10.** One Rust frame per activation plus an explicit counter. Measured: unbounded `CALL` recursion on the oracle gives `Error 11.1`, "Insufficient control stack space", rc 245 — a reportable condition, not a crash. Do **not** reopen D19 in favour of a flat loop over the activation stack: that variant is where the `Rc<Program>` local must be re-derived at every frame transition, and D19 closed that risk.
* **I12's `CALL` half.** Each activation pushes a site entry carrying its own line and an indent base two greater than its caller's.

**The measured `RESULT` semantics, which no document records.** `call sub` where `sub: return 42` sets `RESULT` to `42`. `call sub2` where `sub2: return` **drops** `RESULT` — a program that assigns `result = 'before'` and then calls a value-less routine prints the derived name `RESULT`, not `before`. So `CALL` unconditionally drops `RESULT` and re-assigns it only when the routine returned a value.

**The measured argument semantics.** `call sub 1,,3` with `use arg p, q, r` gives `[1] [Q] [3]`: an omitted middle argument leaves its target unset, so reading it yields the derived name. `arg()` returns 3 — the omitted position still counts toward the argument count.

- [ ] **Step 1: Measure the combined depth budget before writing the counter**

D19 chose per-activation Rust recursion; 4a's `run_bounded` already costs a Rust frame per source nesting level. Nobody has measured the two together, and `INTERPRETER_STACK_BYTES`'s doc names four consumers of which the fourth is an admitted unmeasured gap.

Measure both directions on our binary: recursion depth to abort with no nesting, and with each activation containing a nested `DO`. Compare against the oracle's 11.1 depth. Report both numbers. If our native abort arrives before the counter fires, the counter is decoration and the task must say so rather than ship it.

- [ ] **Step 2: Write the failing test for a callee running the right body**

```rust
#[test]
fn a_called_label_runs_its_own_clauses_not_the_main_body() {
    let out = run_source(b"call sub\nsay 'main'\nexit\nsub: say 'callee'\nreturn\n");
    assert_eq!(out.stdout, b"callee\nmain\n");
}
```

An internal `CALL` targets a **label in the same body**, not a directive, so this test does not exercise `BodyKey::directive`. Add a second test that does, once a `::routine` is reachable; if it is not reachable in 4b, say so in the report and leave `Some(index)` unset with its doc updated to name the phase that sets it.

- [ ] **Step 3: Run it and watch it fail loudly**

Expected: FAIL with the not-implemented message naming `Call` and `4b`.

- [ ] **Step 4: Add the body selector and the settings-inheriting constructor**

- [ ] **Step 5: Move `trace_mode` onto `Activation`**

Delete the `Interp` field. Every reader changes to read the current activation's. The callee inherits the caller's value at call time and does not write back on return.

- [ ] **Step 6: Implement `Call::Named` and `Return`**

`Call` has four forms and **only three are 4b's**:

* `Named { name, literal, args }` — `literal` is true for `CALL "name"`, which bypasses the internal label search.
* `Dynamic { target, args }` — the target is known only at run time.
* `Trap(ConditionTrap)` — `CALL ON`/`CALL OFF`, which is **Task 7's**. Leave it failing loudly until then.
* `Qualified { namespace, name, args }` — `CALL ns:name`. **Phase 5's.** It must keep failing loudly with owner `Phase 5` after this task.

That last point matters for the coverage harness: it counts the *variant*, so implementing `Call` ticks the box while `Qualified` stays loud. **The witness program in the subset must not be a `Qualified` call**, or the witness passes for the wrong reason.

Resolution order for a named call is internal label, then builtin, then external. 4b builds the front. The fallback must fail loudly naming `4c` for a name that is not an internal label, because that is exactly what a reader at the end of 4b will meet.

- [ ] **Step 7: Implement the depth counter and the 11.1 raise**

Raise 11.1, "Insufficient control stack space", rc 245. Take the exact message text from the catalogue, not from this plan.

- [ ] **Step 8: Push an activation site entry, and verify the three-deep transcript byte-for-byte**

The expectation is the `p10` transcript in the D2 section above. Capture it fresh from the oracle rather than copying it from here.

- [ ] **Step 9: Implement `RESULT`**

Drop it on every `CALL`; set it only when the callee returned a value. The test is the measured `1: 42` / `2: RESULT` pair above.

- [ ] **Step 10: Run the full suite and both gates**

Run: `cargo test -p rexx-exec` then `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus`
Expected: 4a's 29 corpus programs still match byte-for-byte. If any moved, the site stack or the indent base is wrong; do not adjust the expectation.

- [ ] **Step 11: Update both owner tables, close the `CALL` half of the KNOWN GAP row, commit**

---

### Task 4: `ExprKind::Call` — the internal-function form

**Files:**
- Modify: `rust/crates/rexx-exec/src/eval.rs` (a new `ExprKind::Call` arm)
- Modify: `rust/crates/rexx-exec/tests/coverage.rs`, `tests/loud.rs`

**Interfaces:**
- Consumes: Task 3's activation machinery, unchanged.
- Produces: the `ExprKind::Call` arm, whose fallback is where 4c hangs the builtin table.

**Why:** I25. `ExprKind::Call`'s owner string is `4b`, and the owner named is the phase after which the variant stops failing loudly *for some target*. Whichever sub-phase runs first has to build the arm; the other inherits it. 4b runs first.

**Inherited items this task pays for:**

* **I25.** Write the split **into `eval.rs`'s arm as a comment**, not only into `phase-4-exclusions.txt`. It is currently recorded in the exclusions file and two test-file comments, and a 4c implementer reading `eval.rs` sees none of them. The comment says: internal routine first (4b), builtin second (4c), external third (Phase 7), and that a name reaching the fallback must fail loudly naming `4c`.

**Measured semantics.** A function call and a `CALL` differ in one visible way beyond the result: `say f(1)` traces `>F>   F => "2"` at the caller's indent before the `SAY` expression's own `>>>`. `call sub 1` traces no `>F>`. Both trace `>A>` per argument at the call site. Task 9 implements the prefixes; this task must not emit them.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn an_internal_function_returns_its_value_into_an_expression() {
    let out = run_source(b"say f(1) + 1\nexit\nf: return 41\n");
    assert_eq!(out.stdout, b"42\n");
}
```

- [ ] **Step 2: Run it and watch it fail loudly on `ExprKind::Call`**

- [ ] **Step 3: Implement the arm**

A function call is an activation like `CALL`'s, with one difference: a routine that returns no value is an error in the expression form. Measure the oracle's error number and text and use it; do not guess.

- [ ] **Step 4: Verify the fallback still fails loudly for a builtin name**

```rust
#[test]
fn a_builtin_name_still_fails_loudly_naming_4c() {
    let out = run_source(b"say length('abc')\n");
    assert_eq!(out.exit, NOT_IMPLEMENTED_EXIT);
    assert!(String::from_utf8_lossy(&out.stderr).contains("4c"));
}
```

This test is the whole point of the task's boundary. It must exist and it must name `4c`.

- [ ] **Step 5: Run the suite, update both owner tables, commit**

---

### Task 5: `PROCEDURE`, `PROCEDURE EXPOSE`, `USE ARG` and `USE LOCAL`

**Files:**
- Modify: `rust/crates/rexx-exec/src/plan.rs` (a body's exposed-name list, precomputed)
- Modify: `rust/crates/rexx-exec/src/run.rs` (the `Procedure` and `Use` arms; the binding step inside `CALL`)
- Modify: `rust/crates/rexx-exec/src/eval.rs` (`ExprKind::VariableReference`)
- Modify: `rust/crates/rexx-exec/src/stem.rs` (comments only, unless a measurement says otherwise)
- Modify: `rust/crates/rexx-exec/tests/coverage.rs`, `tests/loud.rs`

**Interfaces:**
- Consumes: `RootSet::grow_slots` (`crates/rexx-core/src/roots.rs:186-206`), unchanged; `Activation::extra`.
- Produces: the exposed-name binding, performed at the `CALL` instruction while the caller's frame is still the top frame.

**Why:** D9, and it is measured rather than chosen. See the D9 table above; the five probes are reproduced here so the implementer does not have to find them.

**The five measured transcripts.** Caller sets `a.1 = 'kept'` and calls `sub: procedure expose a.`:

| Callee body | Caller prints | rc |
|---|---|---|
| `a.1 = 'changed'` | `changed` | 0 |
| `a. = 'wiped'` | `wiped` | 0 |
| `drop a.` | `A.1` | 0 |

With `b. = a.` added in the caller before the call, and `drop a.` in the callee, the caller prints `A.1 orig` — the drop rebinds the entry, and `b.` still holds the old object.

With the caller never mentioning the name at all:

```rexx
call sub
say zzz          /* prints set-by-callee */
exit
sub: procedure expose zzz
zzz = 'set-by-callee'
return
```

The same holds for a stem the caller never mentions.

**What that forces.** `EXPOSE` aliases the caller's *variable entry*, not the stem *object*. The slot must live in the caller's frame and survive the callee's return, and the caller's plan may have no slot for it. So the binding happens **at the `CALL` instruction, before the callee's frame is pushed**, while the caller is still the top frame. `RootSet::grow_slots` keeps its top-frame invariant, its `4a invariant` panic message stands, and the `#[should_panic(expected = "4a invariant")]` test at `crates/rexx-core/tests/collect.rs:163-181` is not touched. **If this task finds it must relax that panic, stop and report BLOCKED** — that is a plan change, not an implementation detail.

**The indirect form.** `procedure expose (v)` resolves `v` against the caller's pool. Measured: caller has `v = 'zzz'` and `zzz = 'caller-value'`, callee prints `inside caller-value`, caller afterwards prints `after callee-set`. **Two hypotheses fit those bytes** — the list is read from the caller before isolation, or `PROCEDURE` runs in the callee before the pool is isolated — and no probe distinguishes them, because `PROCEDURE` must be the first instruction of a routine so the callee can never have its own `v`. They agree operationally. Implement either; do not report that the measurement chose one.

**Inherited items this task pays for:**

* **I5.** `RootSet::grow_slots` panics on a non-top frame. Under this design that invariant stays true. Say so in the `PROCEDURE` arm's comment, with the reason, so the next reader does not conclude the panic was overlooked.
* **I18.** `RootSet::clear_slot` exists so the read path can tell "unset" from every other value, for `NOVALUE`. `stem_drop` deliberately does **not** use it, and `stem.rs:288-305`'s doc comment explains why a stem's slot is not "empty or not" the way a simple variable's is. Task 7 needs both halves of that distinction; do not collapse them here.
* **I17, corrected.** The scoping document says a mutant rerouting `stem_drop` to a slot clear "becomes pinnable exactly when 4b lands". **That is wrong twice.** A second reference to a stem object exists in 4a already (`b. = a.` shares the object, measured `new new` on both interpreters), and the drop-vs-clear distinction still is not observable through it (measured `A.1 orig` under both the current code and the mutant). This task must either produce a program that distinguishes the two, or reclassify the mutant as genuinely equivalent with the reason written into `mutate-4b.sh`. Do not carry the old explanation forward.

**Measured `USE ARG` semantics.**

* `call sub 1,2,3` with `use arg p` succeeds and binds `p = 1`. Extra arguments are ignored.
* `use strict arg p` with three arguments is Error 40.4, rc 216: "Too many arguments in invocation of SUB2; maximum expected is 1."
* `call sub 1,,3` with `use arg p, q, r` gives `[1] [Q] [3]`.
* `use arg >q` requires the caller to pass a variable reference. `call sub p` with a plain symbol is Error 88.928, rc 168: "The 1 argument must be a VariableReference instance; found "caller"." `call sub >p` works: the callee's `q = 'aliased'` makes the caller's `p` read `aliased`.

That last pair is why `ExprKind::VariableReference` is in this task and not elsewhere — it is the argument-side half of `USE ARG >`, and neither is testable without the other.

- [ ] **Step 1: Write the failing isolation test with values that can distinguish the hypotheses**

A callee setting `x = 1` where the caller also has `x = 1` cannot distinguish isolation from sharing. A probe whose exposed variable holds a value equal to its own derived name cannot distinguish exposure from non-exposure, because an unexposed unset read yields the name. Use values that are neither.

```rust
#[test]
fn procedure_isolates_and_expose_aliases_the_caller_entry() {
    let out = run_source(
        b"v = 'caller-v'\ncall sub\nsay v w\nexit\n\
          sub: procedure expose w\nv = 'callee-v'\nw = 'callee-w'\nreturn\n",
    );
    assert_eq!(out.stdout, b"caller-v callee-w\n");
}
```

- [ ] **Step 2: Run it and watch it fail loudly on `Procedure`**

- [ ] **Step 3: Precompute each body's exposed-name list in `plan.rs`**

`PROCEDURE` must be the first instruction of a routine, so the list is a property of the body and belongs in its plan. The indirect form's names are not known until run time; carry the `SymbolId` to read and resolve it at call time.

- [ ] **Step 4: Bind at the `CALL` instruction**

Grow the caller's frame for any exposed name it has no slot for, record the name in the caller's `Activation::extra`, then push the callee's frame with those slots aliased.

- [ ] **Step 5: Implement `Use::Arg` and `Use::Local`**

`strict` and `allow_optionals` (the trailing `...`) are both fields on `Use::Arg` and both change the arity check. `UseTarget::default` is `USE ARG a = 1`; `UseTarget::alias` is `>a`. Take every error number and text from the oracle.

- [ ] **Step 6: Implement `ExprKind::VariableReference`**

- [ ] **Step 7: Write the stem-exposure tests from the five transcripts above**

All five, as corpus programs if the differential harness covers them better than unit tests. The `drop` pair is the one that pins the design.

- [ ] **Step 8: Settle I17 one way or the other and write the answer into `mutate-4b.sh`**

- [ ] **Step 9: Run the suite and the corpus gate, update both owner tables, commit**

---

### Task 6: `SIGNAL` to a label, and `SIGNAL VALUE`

**Files:**
- Modify: `rust/crates/rexx-exec/src/run.rs` (the `Signal` arm)
- Modify: `rust/crates/rexx-exec/tests/coverage.rs`, `tests/loud.rs`

**Interfaces:**
- Consumes: the label table `CodeBody` already carries, and `Flow`'s existing variants.
- Produces: nothing new for later tasks except the arm's shape; `Signal::Trap` is Task 7's.

**Why:** `SIGNAL` needs only the label table 4a has. It is placed here rather than first because it unblocks nothing, and putting it before `CALL` would have meant designing `Flow`'s escape for a body with no callers.

`Signal` has three forms. `Label(Box<[u8]>)` and `Value(Expr)` are this task's; `Trap(ConditionTrap)` is Task 7's and keeps failing loudly until then. As with `Call::Qualified`, the coverage harness counts the variant, so **the witness program must use `SIGNAL label`, not `SIGNAL ON`.**

`SIGNAL` out of a `DO`, a `SELECT` or an `INTERPRET` fragment must unwind the block stack. 4a's `pop_search_frame` and the `Flow` forwarding already do this shape for `LEAVE`; reuse rather than re-derive. `SIGNAL` from inside a routine to a label in that routine's body stays in the activation; the oracle's behaviour for a `SIGNAL` naming a label that is not in the current body must be **measured**, not assumed.

- [ ] **Step 1: Measure `SIGNAL` out of a nested block, and `SIGNAL` to a label not in the current body**

- [ ] **Step 2: Write the failing tests from the transcripts**

- [ ] **Step 3: Implement `Signal::Label` and `Signal::Value`**

- [ ] **Step 4: Run the suite, update both owner tables, commit**

---

### Task 7: Condition traps, `RAISE`, and `NOVALUE`

**Files:**
- Modify: `rust/crates/rexx-exec/src/error.rs` (`Raised::condition` gains its first reader)
- Modify: `rust/crates/rexx-exec/src/run.rs` (the trap table on `Activation`; `SIGNAL ON`/`CALL ON`; the `Raise` arm; `failure_site` clearing)
- Modify: `rust/crates/rexx-exec/src/lib.rs` (`Novalue::Unset`'s first reader, lines 544-556)
- Modify: `rust/crates/rexx-exec/tests/coverage.rs`, `tests/loud.rs`

**Interfaces:**
- Consumes: `Raised`, `FailureSite`, `record_failure_site`, `Raised::report`, the message catalogue and the 256-major exit rule — all built and byte-verified in 4a.
- Produces: a per-activation trap table; `RAISE`'s raiser reusing the existing families.

**Why:** `RAISE` is the cheapest instruction in 4b's list because the raiser families already exist. The expensive half is resumption: a trap that transfers control after a raise is the first caller that makes `failure_site`'s never-cleared state matter.

**Inherited items this task pays for:**

* **I10.** `Raised::condition` at `error.rs:36-55` has no reader. It is carried as a field rather than hardcoded because `NOVALUE`, `NOMETHOD` and friends need to set it to something else, and it is `#[expect(dead_code)]` rather than `#[allow]` **on purpose, so the day 4b reads it the annotation asks to be deleted.** Delete it.
* **I11.** `Interp::failure_site` is never cleared mid-run and is set first-call-wins (`self.failure_site.is_none()` guards both callers at `run.rs:1199,1316,1368`). A second raise after a trapped first one would report the first site. **This task is the caller that makes it matter.** Clear it when a trap resumes execution, and write a test with two raises where the second is the one reported.
* **I13.** `Novalue::Unset` at `lib.rs:544-556` is produced by the read path and read by nothing. D16 required the flag from the start rather than retrofitting a raise into the hottest path. `SIGNAL ON NOVALUE` is its first reader, and 4c's gate program uses `signal on novalue`.
* **I14, the `+++` half.** Measured 2026-08-01: a trapped `SIGNAL ON SYNTAX` under `trace r` emits **no `+++` and no error report at all**; the trap label's own clause is echoed as an ordinary `*-*`. So condition traps do **not** bring `+++` into 4b. `+++` is command errors and failures, Phase 7's under D18.
* **I16, revisited.** 4a's investigation concluded `SIGNAL ON SYNTAX` cannot accumulate temps leaks, and that conclusion rests entirely on `step_in_temps_frame` being the single chokepoint: the trap acts at instruction-loop level and the wrapper has already truncated before the `Failure` reaches the loop's `Err` arm. **This is the task that makes the trap real.** Re-verify the conclusion against the implementation rather than assuming it, and if Task 1 moved execution off the chokepoint, redo the analysis. `.superpowers/sdd/2026-07-30-phase-4a-executor/temps-frame-investigation.md` has the original.

**The vacuity hazard specific to this task.** A trap criterion that asserts "the handler ran" by checking that the program exited 0 is satisfied by a program that never raised. Assert a **value the handler sets**, and choose one that is neither the variable's derived name nor its unset rendering — an unset read yields the derived name, so a flag left unset renders as something that looks like data.

- [ ] **Step 1: Measure the trap transcripts**

`SIGNAL ON SYNTAX` with a raise, trapped; `CALL ON ERROR NAME handler` with no command (measured 2026-08-01: handler not invoked, rc 0); `SIGNAL ON NOVALUE` reading an unset variable; `RAISE SYNTAX 40.4`; `RAISE ... RETURN` and `RAISE ... EXIT`; `RAISE PROPAGATE` from inside a trap. Capture stdout, stderr and rc separately for each.

- [ ] **Step 2: Write the failing tests, asserting handler-set values rather than exit codes**

- [ ] **Step 3: Add the trap table to `Activation`**

Per activation, not per interpreter: measured in 4a that a callee's settings do not survive its return, and the trap table follows the same rule unless a measurement says otherwise. **Measure it.**

- [ ] **Step 4: Implement `SIGNAL ON`/`OFF` and `CALL ON`/`OFF`**

`SIGNAL ON` transfers control and does not return; `CALL ON` calls the handler and resumes. The two differ in exactly the way that makes `failure_site` clearing necessary.

- [ ] **Step 5: Implement `Raise`**

`Raise` carries `condition`, `propagate`, `rc`, `description`, `additional`, `array` and `result`. `RaiseResult { exit, value }` is the `RETURN`/`EXIT` tail.

- [ ] **Step 6: Wire `Novalue::Unset` to `SIGNAL ON NOVALUE`, and delete the `#[expect(dead_code)]` on `Raised::condition`**

- [ ] **Step 7: Clear `failure_site` on trap resumption, with a two-raise test**

- [ ] **Step 8: Re-verify the temps-frame conclusion against the real trap**

Report the verification in the task report, with the shape of the program used. "It still holds" without a program is not a verification.

- [ ] **Step 9: Run the suite and the corpus gate, update both owner tables, commit**

---

### Task 8: `PUSH`, `QUEUE`, and the in-process queue

**Files:**
- Create: `rust/crates/rexx-exec/src/queue.rs`
- Modify: `rust/crates/rexx-exec/src/lib.rs` (own the queue), `src/run.rs` (the `Push` and `Queue` arms)
- Modify: `rust/crates/rexx-exec/tests/coverage.rs`, `tests/loud.rs`
- Modify: `docs/superpowers/plans/phase-4-exclusions.txt`

**Interfaces:**
- Produces: an in-process LIFO/FIFO queue. 4c's `QUEUED()` and `PARSE PULL` read it.

**Why:** I15. The queue is 4b's; `QUEUED()` is a partial 4c exclusion resting on it.

**The differential caveat, and it is live rather than theoretical.** The oracle's queue is backed by `rxapi`, a daemon. Confirmed on this host 2026-08-01: `rxapi` is running as pid 857 and `rxqueue('G')` returns `SESSION`. Cross-process sharing will never match, so **a differential run involving the queue is single-program only** — one program, both interpreters, no state carried in from outside. Write that rule into `phase-4-exclusions.txt` beside the `QUEUED` row, not only into this task.

- [ ] **Step 1: Measure `PUSH` and `QUEUE` ordering in a single program**

- [ ] **Step 2: Write the failing test**

- [ ] **Step 3: Implement the queue and the two arms**

- [ ] **Step 4: Add the single-program rule to the exclusions file**

- [ ] **Step 5: Run the suite, update both owner tables, commit**

---

### Task 9: Trace for calls, and the Controlled-loop `>>>` gap

**Files:**
- Modify: `rust/crates/rexx-exec/src/trace.rs` (`>A>`, `>F>`, `>R>`, the activation indent base)
- Modify: `rust/crates/rexx-exec/src/run.rs` (the two missing `>>>` lines on a Controlled loop's re-tested pass)
- Modify: `rust/crates/rexx-exec/tests/trace_oracle.rs` (new committed expectations)
- Modify: `docs/superpowers/plans/phase-4-exclusions.txt`

**Interfaces:**
- Consumes: Task 3's activation depth; `static_indent`, unchanged.

**Why:** the trace surface is where 4a's four late divergences were found, all by probing adjacent shapes rather than by the table. 4b adds prefixes to it.

**The measured transcripts.** At `trace r`, a call with two arguments:

```
     2 *-* call sub 1, 2
     5 *-*   sub:
     5 *-*   procedure
     6 *-*   use arg a, b
       >>>     "1"
       >>>     "2"
     7 *-*   return a + b
       >>>     "3"
       >>>   "3"
     3 *-* say 'done'
       >>>   "done"
done
     4 *-* exit
```

Four things to read off it: the callee's `sub:` **label clause is echoed**, and so is `procedure`; callee clauses sit at indent 2 and their value lines at the matching deeper column; `use arg` emits one `>>>` per argument; **the return value is traced twice**, once at the callee's indent and once at the caller's.

At `trace i`, the same call adds `>L>` and `>A>` at the call site and `>=>`/`>V>` inside the callee, and emits **no `>I>` and no `<I<`**. An expression-form function call adds `>F>   F => "2"` at the caller's indent, named for the routine. Both full transcripts are in the "Corrections to inherited items" section above.

**Inherited items this task pays for:**

* **I14, corrected.** 4b's prefixes are `>A>` (ARGUMENT, both call forms), `>F>` (FUNCTION, expression form only) and `>R>` (ALIAS, `USE ARG >`, **unmeasured — measure it**). `>I>`/`<I<` produce nothing for an internal call at `trace i`. **Settle their owner by measurement and write the answer into `phase-4-exclusions.txt`**, either as a 4b obligation or as a reassignment to Phase 5. Reading `RexxActivation.cpp:3567-3588` told us the prefixes exist; it did not tell us which construct emits them, and that is the mistake this item inherited.
* **I31.** A Controlled (`TO`-style) loop's re-tested pass omits two `>>>` value lines. Measured, cause read from `DoBlock::checkControl` (`DoBlock.cpp:182`) rather than inferred, and costed at about twenty lines plus re-verification of bound-before-test, `FOR` and `ITERATE` — half a day, not a rewrite. **Close it here.** An overstated cost in a gap row is how a cheap fix stays open, and this row was corrected once for exactly that.

- [ ] **Step 1: Regenerate the five existing trace expectations to prove the harness still round-trips**

`tests/trace_oracle.rs`'s module doc carries the regeneration command, and all five were verified byte-identical in 4a.

- [ ] **Step 2: Measure `>R>` with `call sub >p` / `use arg >q` under `trace i` and `trace r`**

- [ ] **Step 3: Measure whether `>I>`/`<I<` appear for any 4b construct at any trace level**

If they do not, that is the finding. Record it.

- [ ] **Step 4: Commit the new expectations, then implement**

- [ ] **Step 5: Add the activation indent base, outside `static_indent`**

`static_indent` stays a pure function of the flat instruction list. Task 11 of 4a built it that way deliberately and the error report (Task 2) depends on the same separation.

- [ ] **Step 6: Close I31's two missing `>>>` lines, and re-verify bound-before-test, `FOR` and `ITERATE`**

- [ ] **Step 7: Add a coverage measure to the trace table**

D14's criterion 3 amendment. The honest statement today is that the five witnesses verify what they cover and the trace surface's coverage is measured by nothing. Produce a number: which of the nineteen prefixes have a committed expectation, which do not, and which are out of scope with an owner.

- [ ] **Step 8: Run the suite, remove I31's KNOWN GAP row, commit**

Removing a KNOWN GAP row needs the gap actually closed and a witness in the tree. Adding rows needs no permission; removing them does.

---

### Task 10: The 4b corpus, the shared owner table, and the collector

**Files:**
- Create: `rust/corpus/phase-4b.txt`, `rust/corpus/proc/*.rex`
- Create: `rust/crates/rexx-exec/tests/owners.rs`
- Modify: `rust/crates/rexx-exec/tests/corpus.rs:186`, `tests/coverage.rs:586`, `tests/collect_stress.rs:96` (each has its own `read_subset` copy with the filename hardcoded at the call site)
- Modify: `rust/corpus/phase-4a.txt:18`, `rust/corpus/README.md:108-109`
- Modify: `rust/crates/rexx-exec/src/lib.rs` if the collector sweep finds a second under-rooting site

**Interfaces:**
- Produces: `phase-4b.txt`; a subset **union** read by all three harnesses.

**Why:** D6, I36, I19, I20, and the corpus rules that bind every 4b program.

**The corpus rules for 4b, which bind every program in `phase-4b.txt`:**

* **No `DO OVER` on a stem.** The oracle walks a balanced tree and we use a hash map; measured, tails 1, 2, 3, 10, ZZ, B yield `1 B 3 2 ZZ 10`. Such a program could never pass. (I37, D3.)
* **No builtin calls.** 4b depends on 4c nowhere, and a corpus program written naturally will reach for builtins to make a routine do something observable. `say`, assignment and arithmetic are enough. This is corpus discipline, not a dependency — but a 4b program calling `length()` fails loudly and tells you nothing about 4b.
* **No `PARSE`.** 4c's.
* **No queue state carried in from outside the program.** The oracle's queue is `rxapi`-backed (Task 8).

**Inherited items this task pays for:**

* **I36.** `coverage.rs` and `loud.rs` duplicate the owner table by hand, because an integration test cannot `mod` another test binary's directory and no shared module was in scope in 4a. 4b and 4c edit both on every variant they deliver, and a divergence between them is caught by nothing. **Make `tests/owners.rs` the single table both include**, via `#[path]` or a shared module both `mod`-include from a common file. Then add a test that the two tables are the same object rather than two lists that happen to agree.
* **I19.** `EXIT`'s result is under-rooted from the temps-frame pop to `exit_code_for` — under-rooting, the direction that breaks when a collector lands, and longer than any window the crate documents. Harmless today only because nothing between that pop and `exit_code_for` calls `alloc_with`. The pointer was deliberately placed on `Heap::collect` in `rexx-core` rather than at the leak site, because the person who turns this into a use-after-free is whoever wires a collector into the interpreter — **and anyone doing 4b work in `rexx-exec` will never see it.** That instruction also says to sweep `rexx-exec` for the same shape first. **Do the sweep**, and report what it found, including "nothing" if that is the answer.
* **I20.** The collect-on-every-allocation mode has never seen a call frame. Criterion 4 passed on 29 programs, all of them 4a-shaped. The gate says so: "4b's body-calls-body recursion, argument passing, and everything Phase 5's object model eventually adds are all untested by this mode as it stands." `collect_stress` must run the **union** of the subsets, and 4b's programs must include calls, arguments and exposure.
* **I26 and D7's documentation half.** `corpus/phase-4a.txt:18` and `corpus/README.md:108-109` both say three `num/` programs return "for 4b or 4c, once `List` exists"; `phase-4-exclusions.txt:176-179` assigns `List` to Phase 5. The ruling is right — measured, `(1, 2)` is an `Array` instance whose `~items` is 2. Correct both comments to say Phase 5. A 4b author reads the corpus files first and would otherwise go looking for work that is not theirs.

- [ ] **Step 1: Make `tests/owners.rs` and have both harnesses read it**

Do this first. Every later step edits the owner table, and doing it last means doing it twice.

- [ ] **Step 2: Change the three `read_subset` call sites to take a list**

`&[&Path]`, union semantics, in all three. Keep each harness's own copy of the reader — factoring those together is a separate change and not this task's.

- [ ] **Step 3: Write the 4b corpus programs**

One per construct at minimum, and at least one that combines two: a raise inside a routine inside a loop, an exposed stem mutated by a callee, a trap that resumes and then raises again. **The combinations are the point.** 4a's whole-branch review found two Criticals that had survived 824 tests, a 29-of-29 byte-identical corpus, nine per-task reviews, seven gate criteria and a nine-mutation script — because the coverage criterion enumerates *variants* and asserts nothing about *combinations*. `Stem` had a witness. Arithmetic had witnesses. `Stem` as an arithmetic operand had none, and `a. = 5; say a. + 1` aborted the process.

- [ ] **Step 4: Run the corpus in report mode, then in strict mode**

Run: `cargo test -p rexx-exec --test corpus` then `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus`
The report names which owner each remaining failure belongs to. A failure naming `4c` is expected; a failure naming `4b` is this plan's work.

- [ ] **Step 5: Sweep `rexx-exec` for the I19 under-rooting shape, and report the result**

- [ ] **Step 6: Run `collect_stress` over the union**

- [ ] **Step 7: Correct the two `List` comments, commit**

---

### Task 11: The `base/keyword` L1 table

**Files:**
- Modify: `rust/crates/rexx-extract/src/lib.rs` (a third extractor beside `extract` at line 53 and `extract_assertions` at line 233)
- Create: `rust/crates/rexx-exec/tests/keyword_assertions.rs`
- Modify: `docs/superpowers/plans/l1-coverage.md`

**Interfaces:**
- Produces: a `base/keyword` row table in the same shape `tests/assertions.rs` consumes, with an `EXEMPT` list committed beside it.

**Why:** I28. The split table names `base/keyword` as 4b's L1 obligation and `base/bif` as 4c's, and **nothing extracts either today.** `rexx-extract` has `extract` (test methods) and `extract_assertions` (the `base/expressions` table) and nothing else.

**D12 is settled for 4b as Option A: a third extractor, not a generalisation of `extract_assertions`.** That function is specific to `base/expressions`'s `assertSame` shape and already needed two modelling corrections — single-quoted method names, and `expectSyntax` markers changing what a later `assertSame` *means*. Both corrections were about a group's mechanics, and the same shape will recur. 4c makes its own call for `base/bif`.

**The conservation invariant is mandatory.** `rows + dropped == calls`, asserted, with the dropped rows enumerated and committed. This is the single most valuable thing 4a's Task 15a produced: a percentage cannot notice a missing population; a conservation law can. An extractor that silently drops a third of the group and reports 100% on the rest is the exact failure mode L1 already had once — Phase 0's harness rendered each method inside `::routine main public`, so an extracted program's main body was empty and executed **nothing at all**, and two interpreters agreed on nothing.

**What this task must not promise.** `tests/assertions.rs`'s 35 exempt rows will not move. All 35 are `unblocked_by: "Phase 5"` — verified, `grep -c` returns 35 of 35 — and the two whose first-observed blocker is a 4b construct re-block on a message send one line later in the same prelude. Implementing `Call` moves the blocker; it does not unblock the row. What 4b owes that harness is nothing, except that `the_exempt_set_matches_the_current_blocked_rows` fails if a listed row starts passing, so an accidental improvement shows as a red test rather than silently.

- [ ] **Step 1: Check whether `base/keyword` uses a shape `extract_assertions` already models**

Ten minutes, and it decides whether Option A is doing unnecessary work. Report the answer either way.

- [ ] **Step 2: Write the conservation test first**

Before the extractor works, write the test that asserts `rows + dropped == calls` and watch it fail on a count of zero.

- [ ] **Step 3: Write the extractor**

- [ ] **Step 4: Commit the row table and the `EXEMPT` list**

- [ ] **Step 5: Run it in report mode and record the pass rate**

Report mode first. A strict gate on a table nobody has looked at turns a measurement into a blocker.

- [ ] **Step 6: Add a `REXX_KEYWORD_GATE=1` strict mode matching the existing harnesses' convention, commit**

---

### Task 12: The 4b gate

**Files:**
- Create: `docs/superpowers/plans/phase-4b-gate.md`
- Create: `rust/scripts/mutate-4b.sh`
- Modify: `docs/superpowers/plans/phase-4-exclusions.txt`

**Why:** D13 and D14.

**The criterion set carries forward from `phase-4a-gate.md` with the two amendments D14 names, plus what 4b adds.** Write each criterion so it can fail. Four criteria in 4a could not, and each was caught late:

* criterion 6's predecessor was satisfied by `/bin/true`;
* criterion 4 had no way to fail at all until it was rewritten, then no *subject* until the mode it named was built — the collect-on-every-allocation mode had zero lines of implementation anywhere when the criterion was strengthened to depend on it;
* `strict_comparison_never_calls_to_number` returned before reading either operand, so no result could observe the property;
* the CONCATENATION rows would have passed while testing nothing.

**And the mutation script itself was vacuous.** `mutate-4a.sh` reported 9 of 9 caught **with the oracle absent**, because any non-zero exit counted as a catch. It has since gained `require_baseline_pass` and a `subset_status` that distinguishes PASSED, DIVERGED and INFRA_FAILURE. `mutate-4b.sh` reuses that guard mechanism; the mutations themselves are not reusable.

**Three 4b-specific vacuity shapes to write against:**

* **A trap criterion asserting the program exited 0.** A program that never raised also exits 0. Assert a value the handler set, and pick one that is neither the flag variable's derived name nor its unset rendering.
* **Criterion 4 carried forward verbatim.** It passes today with zero call frames exercised (I20). Carried unchanged to 4b it is a criterion that cannot fail *for the thing 4b adds*. It needs the subset union from Task 10 **and an activation-shaped negative control**: 4a's control deletes `eval_arithmetic`'s `push_temp(left_value)`. A 4b control must delete a root a *call* holds — the argument list between evaluation and the callee's `USE` — or it re-tests 4a and reports a pass that means nothing.
* **A coverage criterion that enumerates variants.** It says nothing about combinations, which is how 4a's two Criticals survived everything. State the limit in the criterion's own text rather than leaving a reader to infer coverage the instrument does not provide.

**What the gate must state up front rather than discover:** criterion 2's exempt list cannot light up at this gate or at 4c's, for the measured reason in Task 11.

**I27 is not this gate's criterion.** The 342 expected trace-output lines in `TRACE.testGroup` are "4b's and 4c's to satisfy", but an ooTest group is not runnable as extracted, and the same file yields 239, 342, 374, 393 and 437 under five different defensible anchorings — three recounts have already gone astray. If this gate uses any figure from that file, **state which scan produced it**. Prefer a named, measured subset over the whole group.

- [ ] **Step 1: Write the gate document before running anything**

Criteria first, results second. A criterion written after the measurement is a description of what happened.

- [ ] **Step 2: Write `mutate-4b.sh` with 4a's guard mechanism and 4b-shaped mutations**

Include the activation-shaped negative control. Include I17's resolution from Task 5, with its real explanation.

- [ ] **Step 3: Run every gate: full suite, `REXX_CORPUS_GATE=1`, `REXX_ASSERTIONS_GATE=1`, `REXX_KEYWORD_GATE=1`, `clippy -D warnings`, `cargo fmt --check`, the mutation script**

Read each exit status unpiped.

- [ ] **Step 4: Assess each criterion honestly, including the ones met weakly**

4a's gate recorded five met, one met with an inherited criterion defect, and one met weakly with an open gap. That is what an honest gate looks like. A seven-of-seven with no qualifications, after a sub-phase this size, is a claim about the instruments rather than the code.

- [ ] **Step 5: Commit the gate document, then read the hash back and record it in the ledger**

---

## Explicitly not in scope, and not promised

* **`tests/assertions.rs` moves not at all.** All 35 exempt rows need Phase 5. Measured; see Task 11.
* **`Plan::by_symbol` stays a `HashMap`.** D16's shape wants a `Vec` index, and `SymbolId::index()` landed at `180875a9` so the swap is now this crate's decision. Variable lookup is 8.1%/32.2% of runtime, so it deserves its own measurement rather than arriving as a side effect of 4b. `Option<usize>` is still required because keywords, labels and constants share the `SymbolTable`, so a dense `Vec` has holes.
* **The parser's recursion cliffs.** Prefix-operator chains recurse in `message_subterm` outside the shared depth budget, aborting between 1,150 and 1,200 levels on a default 2 MiB thread, and the oracle's cliff for that construct has never been measured. `Debug`, `PartialEq` and `Clone` on `Expr` are still recursive with cliffs at 2,000/2,050 and 2,100/2,200; the trigger for scheduling them is the first test that formats or compares a deep tree. The depth counter protects a sized caller only — on a default 2 MiB thread the native abort arrives at 331 parens or 341 calls, long before any counter at 50,000 fires, and the long-term answer is a documented minimum stack or a sized entry point in `rexx-parse`. **Task 3's Step 1 measures 4b's contribution to this; it does not fix it.**
* **`Heap::alloc`'s friendly name.** It still bypasses the stress hook. Parked in 4a by choice.
* **A `DO`/`LOOP` temps frame growing for its whole run.** Parked in 4a by choice.
* **Everything 4c owns**, which is `PARSE` in all its forms, the 66 in-scope builtins, `ADDRESS`, `VALUE`'s variable-access form, `QUEUED()`, `ARG()`, `CONDITION()`, and the `rexxcps.rex` gate question (D8: it auto-adjusts its loop count from measured wall-clock time, so it cannot be the byte-for-byte differential the parent plan assumes).
* **The five 4c-only open decisions**: D4 (which of the 15 excluded builtins are genuinely blocked — measured 11 of 15, with `QUALIFY` not blocked at all and `USERID`/`SETLOCAL`/`ENDLOCAL` blocked because `std::env::set_var` is `unsafe` in edition 2024 and the workspace forbids it), D7 (`ExprKind::List`, whose documentation half only is fixed in Task 10), D8, D11 (`RANDOM`/`DATE`/`TIME` determinism — measured, unseeded `RANDOM` is deterministic across separate processes, so "the values differ between runs" is not evidence of randomness), D12 for `base/bif`.
