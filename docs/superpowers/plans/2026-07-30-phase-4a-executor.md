# Phase 4a executor implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** run a classic Rexx program with no procedures, no `PARSE` and no builtin calls, byte-for-byte as `build/bin/rexx` runs it.

**Architecture:** a new `rexx-exec` crate. `Interp` owns the heap, root set, activation stack, plan cache and sinks, and holds programs as `Rc<Program>` so `&Expr` borrows never collide with `&mut self`. Expression evaluation is recursive over the AST with a trace event per step; instruction execution is a program counter over Phase 3's flat, index-linked instruction list.

**Tech stack:** Rust 1.96.1, no `unsafe`, `cargo fmt` default, `clippy -D warnings`. Depends on `rexx-core`, `rexx-num`, `rexx-parse`, `rexx-inventory`.

**The spec is `docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md` and it governs.** It carries the measured transcripts every task below is checked against, and its decision blocks D15 to D19 are binding. Read the section a task names; do not read the whole spec.

## Global constraints

* **Test a private subject with a `#[cfg(test)] mod tests` beside it, not an integration test.**
  `Interp` and its methods are private to `rexx-exec`, and an integration test under `tests/`
  can only reach what is `pub`. Choosing one is what forced Task 3's spike to expose a public
  entry point that now carries `#[doc(hidden)]` and a note telling a later sub-phase to delete
  it. Integration tests are for the public surface: the runner, the corpus harness, the gate
  harnesses. An earlier draft of this plan named an integration test for every module task,
  which is why two implementers asked the same question.

* **The C++ tree is the oracle and is never modified.** `interpreter/`, `samples/`, `build/`, `ootest/` are read-only. Every behavioural question is settled by running `build/bin/rexx`, not by reading the ANSI standard.
* **Wrap every oracle invocation** as `( ulimit -v 1048576; build/bin/rexx FILE )`. The interpreter requests gigabytes mid-range and gets OOM-killed otherwise, which has already cost a session.
* **Never set `NUMERIC DIGITS` above 1000** in a probe.
* **Never instantiate `.Package~new`** on a file inside the repository: it executes that file's prolog and has written untracked files into the tree.
* Scratch files go in the session scratchpad, never in the repository.
* **No `unsafe`.** If a task appears to need it, stop and report BLOCKED.
* **Never `git add -A`.** Stage the exact paths the task names. Do not run `git reset --hard`, do not force-push.
* Comments state the contract at the top and the reasoning at the decision point. Never delete an existing comment to make a change easier. Prefer `--` over an em-dash, matching what the tree already does everywhere.
  > **The "no structuring semicolons" rule was imported here by mistake and is withdrawn.** It belongs to a different repository's conventions, not this one. It was enforced through several task reviews before Moritz corrected it, and enforcing it did measurable harm: a comment-only fix round rewriting semicolons into separate sentences introduced three false statements, one of them a new structuring semicolon inside the very comment being rewritten. Reviewers should not raise it, and nothing already written needs changing. Kept as a note rather than deleted, because the reviews that cited it are on disk and a reader meeting one needs to know it was withdrawn rather than forgotten.
* A value's rendering is fixed when the value is created. Any code that formats a number with `settings.digits()` or `settings.form()` instead of the value's own captured pair is wrong; see D15.
* Anything 4a does not implement **fails loudly**: a dedicated exit code outside 157..253 (where `256 - major` lives) and a message naming the construct and the owning sub-phase. Never a plausible Rexx condition.

## File structure

```
rust/crates/rexx-exec/
  Cargo.toml
  src/lib.rs          Interp, the public entry point, the plan cache
  src/value.rs        value model, conversions, string and number identity
  src/stem.rs         stems, tail resolution, derived names
  src/plan.rs         the per-body resolution pass
  src/activation.rs   one frame: slot handle, block stack, pc, Settings, Rc<Plan>
  src/eval.rs         expression evaluation and the operators
  src/run.rs          the instruction loop and control flow
  src/trace.rs        trace events and prefix formatting
  src/error.rs        Raised, the condition payload, the message catalogue
  src/bin/rexx-run.rs the runner the differential tests drive
  tests/              per-area integration tests, plus the gate harnesses
rust/corpus/phase-4a.txt     the named L0 subset
rust/corpus/lang/*.rex       new 4a programs
docs/superpowers/plans/phase-4-exclusions.txt
```

Two earlier crates are amended, and those amendments are Tasks 1 and 2 rather than incidental edits: `rexx-parse` gains a `CodeBody` for the main body, and `rexx-core` gains the value bodies and root-set slot frames.

---

### Task 1: `rexx-parse` gives the main body a `CodeBody`

**Spec:** "The borrow shape", the paragraph beginning "One `rexx-parse` change is required".

**Files:**
- Modify: `rust/crates/rexx-parse/src/lib.rs`
- Modify: `rust/crates/rexx-parse/tests/program.rs`, and any test that names `Program::instructions` or `Program::labels`

**Interfaces:**
- Produces: `Program { source, main: CodeBody, directives, symbols }` and `Fragment { source, body: CodeBody, symbols }`.
- `CodeBody { instructions: Vec<Instruction>, labels: BTreeMap<Box<[u8]>, usize> }` is unchanged and keeps its `Clone, PartialEq, Eq, Debug, Default` derives.

**Why:** `fn eval(&mut self, body: &CodeBody, …)` cannot be called for the body 4a actually runs. `Program` holds `instructions` and `labels` as sibling fields and `Fragment` has no label table at all, so there is no borrowed `CodeBody` view to hand the evaluator, and behind an `Rc` you cannot make one without cloning both vectors per call.

- [ ] **Step 1: Measure whether an `INTERPRET` fragment may contain a label**

Run, wrapped: a program whose body is `interpret "lab: nop"` and a second whose body is `interpret "signal lab; lab: nop"`. Record the exact error number and text if either is rejected.

Expected from Phase 3's own note: a label inside `INTERPRET` text is error 47.1, so `Fragment`'s label table is always empty. **Confirm it rather than assuming it**, and put the transcript in the task report. If it turns out labels are legal, say so and stop: the `Fragment` field is then load-bearing and 4b needs to know.

- [ ] **Step 2: Change the two structs**

`Program::instructions` and `Program::labels` become `Program::main: CodeBody`. `Fragment::instructions` becomes `Fragment::body: CodeBody`. Keep every doc comment: move each field's comment onto the corresponding `CodeBody` field's use site, and do not drop the paragraph explaining why labels are keyed by value rather than by `SymbolId`.

In `parse_program` and `parse_interpret`, `parsed.main` already *is* a `CodeBody`, so both become a move rather than a field split.

- [ ] **Step 3: Update callers and tests**

`cargo test -p rexx-parse` names every caller. Prefer `program.main.instructions` over destructuring, so the diff stays mechanical.

- [ ] **Step 4: Verify**

Run: `cd rust && cargo test -p rexx-parse && cargo clippy -p rexx-parse --all-targets -- -D warnings && cargo fmt --check`
Expected: the same test count as before the change, zero failures. A changed count means a test was lost, not that the change worked.

- [ ] **Step 5: Commit**

```bash
git add rust/crates/rexx-parse
git commit -m "Give the main program body a CodeBody, so the executor can borrow one"
```

---

### Task 2: `rexx-core` gains the value bodies and root-set slot frames

**Spec:** D15 "Where these variants live, and what that costs", D15a, and D16's `RootSet` bullet.

**Files:**
- Modify: `rust/crates/rexx-core/src/body.rs`, `src/roots.rs`, `src/heap.rs` (tests), `Cargo.toml`
- Test: `rust/crates/rexx-core/tests/collect.rs`

**Interfaces:**
- Produces:
  ```rust
  pub enum Body {
      Text { bytes: Vec<u8>, num: Option<Result<Box<Number>, NotNumeric>> },
      Num { value: Number, created_digits: u32, created_form: Form, text: Option<Vec<u8>> },
      Stem { name: Box<[u8]>, default: Option<ObjRef>, tails: HashMap<Vec<u8>, Option<ObjRef>> },
      Array(Vec<ObjRef>),
      Instance(Vec<(String, ObjRef)>),
      WeakRef(ObjRef),
  }
  ```
  `Body::String` is deleted. `BehaviourId::STEM` is added.
- Produces on `RootSet`: `push_slots(initial_len: usize) -> SlotFrame`, `pop_slots(SlotFrame)`, `slot(SlotFrame, usize) -> Option<ObjRef>`, `set_slot(&mut self, SlotFrame, usize, ObjRef)`, `grow_slots(&mut self, SlotFrame) -> usize` returning the new index.

**Why:** the values 4a manipulates are heap objects, and `Body` lives here. And an activation's variables must be reachable from the collector: `RootSet` is globals plus temps, so as it stands the first collection sweeps every local.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn a_stems_tails_and_default_are_traced() {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let tail = heap.alloc(Body::Text { bytes: b"kept".to_vec(), num: None });
    let default = heap.alloc(Body::Text { bytes: b"dflt".to_vec(), num: None });
    let mut tails = HashMap::new();
    tails.insert(b"1".to_vec(), Some(tail));
    // A tombstone: present, and reaching nothing.
    tails.insert(b"2".to_vec(), None);
    let stem = heap.alloc_with(BehaviourId::STEM, Body::Stem {
        name: b"A.".to_vec().into_boxed_slice(),
        default: Some(default),
        tails,
    });
    roots.add_global("a.", stem);
    heap.collect(&roots);
    assert_eq!(heap.get(tail).is_some(), true, "a live tail was swept");
    assert_eq!(heap.get(default).is_some(), true, "the stem default was swept");
}

#[test]
fn slot_frames_keep_locals_alive_and_release_them_on_pop() {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let frame = roots.push_slots(2);
    let v = heap.alloc(Body::Text { bytes: b"local".to_vec(), num: None });
    roots.set_slot(frame, 0, v);
    heap.collect(&roots);
    assert!(heap.get(v).is_some(), "a live local was swept");
    roots.pop_slots(frame);
    let stats = heap.collect(&roots);
    assert_eq!(stats.swept, 1, "the local outlived its frame");
}

#[test]
fn a_slot_frame_grows_for_a_name_the_plan_never_saw() {
    let mut roots = RootSet::new();
    let frame = roots.push_slots(1);
    let index = roots.grow_slots(frame);
    assert_eq!(index, 1);
}
```

- [ ] **Step 2: Run them to watch them fail**

Run: `cd rust && cargo test -p rexx-core`
Expected: compile errors — `Body::Text`, `Body::Stem`, `push_slots` do not exist.

- [ ] **Step 3: Add the variants and extend `trace`**

`Body::trace` must gain arms for `Text` (reaches nothing) and `Stem` (reaches `default` and every `Some` tail). **It has no wildcard arm and must not gain one**: that exhaustive match is the whole of Phase 1's GC-safety argument, so a new variant has to be a compile error here rather than a use-after-free later.

`rexx-core/Cargo.toml` gains `rexx-num`, because `Body::Text` holds a `Number`. That edge is new and points the object model at the arithmetic core; it is declared in the spec and is not an accident to be quietly avoided.

Delete `Body::String` and update `heap.rs`'s `retire_tests`, which construct it.

- [ ] **Step 4: Implement slot frames**

Only the **top** frame ever grows **in 4a**, which has one frame, and for `INTERPRET`, which runs inside the activation that created it. Assert it: `grow_slots` on a frame that is not the top one is a panic with a message saying so, because a silent wrong answer here is a variable that lands in another routine's pool.

Write in the doc comment that this is a **4a invariant and not a general one**, and why: measured, `sub: procedure expose zzz` makes a callee write into its caller's pool while the callee's frame is on top, so 4b either grows a non-top frame or binds exposed names to caller slots at call time. A panic that a later sub-phase must remove is the right shape here; a silent allowance it would inherit is not.

`iter()` must yield globals, temps **and** every assigned slot, so `collect`'s signature does not change.

- [ ] **Step 5: Verify**

Run: `cd rust && cargo test -p rexx-core && cargo clippy -p rexx-core --all-targets -- -D warnings`
Expected: all pass, including the three new tests.

- [ ] **Step 6: Commit**

```bash
git add rust/crates/rexx-core
git commit -m "Add the executor's value bodies and root-set slot frames"
```

---

### Task 2b: `RootSet` cannot express an unset slot

**Spec:** D16's `RootSet` bullet. **Files:** `rust/crates/rexx-core/src/roots.rs`, `tests/collect.rs`.

**Why:** `set_slot` takes an `ObjRef`, so there is no way to write "unset" back into a slot that has been written. `ObjRef::NIL` cannot stand in for it: `x = .nil` is legal Rexx and `.nil` is a value, which D16 already settled when it rejected storing slots in `temps`.

Found by Task 5's design pre-flight rather than by anything looking for it. It does not block Task 5, because a stem's `DROP` replaces the object with `default: None, tails: {}`, which is observationally identical to never-touched — measured: after `drop x.`, both `x.1` and `x.` render derived names. It **does** block plain `DROP a` on a simple variable in Task 9, where the read afterwards must yield the derived name, and in 4b must fire `NOVALUE`.

- [ ] **Step 1: Write the failing test** — set a slot, clear it, and assert the read answers "unset" rather than any value, including that it is distinguishable from a slot holding `.nil`.
- [ ] **Step 2: Add the clearing operation.** Either `set_slot` takes an `Option<ObjRef>` or a `clear_slot` sits beside it; pick one and say why in the doc comment. Whichever you choose, `RootSet::iter` must keep yielding exactly the live slots, so a cleared slot stops being a root — that is the half a naive `Option` wrapper gets wrong, and a swept-too-late value is invisible until a collection happens at the wrong moment.
- [ ] **Step 3: Check the neighbours.** `grow_slots` and `pop_slots` both touch the same storage; say in the report whether a cleared slot interacts with either, particularly whether growth can reuse a cleared index.
- [ ] **Step 4: Verify** — `cargo test -p rexx-core`, then the workspace, clippy unpiped.
- [ ] **Step 5: Commit.**

---

### Task 3: The borrow-shape spike

**Spec:** "Architecture / The borrow shape", and D19.

**Files:**
- Create: `rust/crates/rexx-exec/Cargo.toml`, `src/lib.rs`, `src/bin/rexx-run.rs`
- Create: `rust/crates/rexx-exec/tests/spike.rs`
- Modify: `rust/Cargo.toml` (workspace members)

**Interfaces:**
- Produces: `Interp` owning `Heap`, `RootSet`, `Vec<Activation>`, `plans: HashMap<BodyKey, Rc<Plan>>`, an output sink and a trace sink; `pub fn run_program(text: Vec<u8>) -> Outcome`, executed on a dedicated thread.

**Why:** this is the phase's one unsolved architectural question and the parent plan says to spike it first. The deliverable is a proof that compiles, not a design note.

- [ ] **Step 1: Prove the shape with the smallest possible interpreter**

Enough of `Interp` to execute `say 'hello'` and nothing else. The discipline being proven:

```rust
// The instruction loop clones the Rc into a local on entry, and every
// &CodeBody and &Expr derives from that local. The activation's own Rc is a
// liveness anchor and is never borrowed through.
let program = Rc::clone(&self.activations.last().expect("a frame").program);
let body = &program.main;
while let Some(instruction) = body.instructions.get(self.pc()) {
    // self.eval(...) takes &mut self, which only compiles because `body`
    // borrows `program`, a local, rather than borrowing self.
    self.step(body, instruction)?;
}
```

Put the version that does **not** compile in a comment beside it, with its `E0502`, because the next phase to touch this will want to know which shape is wrong:

```rust
// Does not compile: borrows self, then calls &mut self.
//   let body = &self.activations.last().unwrap().program.main;
//   self.step(body, ...);           // E0502
```

- [ ] **Step 2: Prove it survives a fragment created mid-instruction**

A test that parses a fragment at run time with `parse_interpret`, executes its body inside the current activation, and returns. The fragment's `Rc` is a local that outlives the nested loop. This is 4a building the machinery; the `INTERPRET` *instruction* is 4b's and still fails loudly.

- [ ] **Step 3: Run the interpreter on its own thread**

**The sized thread belongs to `rexx-exec`'s public entry point, not to the `rexx-run` binary**, because the L0 harness and the assertion-table harness both run in process and a `cargo test` thread's default stack is far smaller than the one the depth limit is calibrated against. Put it in the binary only, and every in-process caller sits on the cliff the depth policy exists to keep them off.

That entry point spawns a thread with an explicit stack size, and **that thread owns everything from `parse_program` onward** — bytes in, an outcome out. `Rc<Program>` is `!Send`, so a program parsed on the main thread cannot be handed across, and getting this wrong is a compile error on day one rather than a subtle bug.

Record the chosen stack size and the measured per-frame cost in the task report; Task 11 sets the depth limit from them.

- [ ] **Step 4: Verify**

Run: `cd rust && cargo test -p rexx-exec && cargo clippy -p rexx-exec --all-targets -- -D warnings`
Expected: the spike tests pass, `say 'hello'` prints `hello`.

- [ ] **Step 5: Commit**

```bash
git add rust/crates/rexx-exec rust/Cargo.toml
git commit -m "Spike the executor's borrow shape: Interp owns everything but the AST"
```

---

### Task 3b: `rexx-parse` drops a deep expression by recursion

**Spec:** D19, "Phase 3's parser has the same exposure and it is unverified".

**Files:**
- Modify: `rust/crates/rexx-parse/src/ast.rs`
- Test: `rust/crates/rexx-parse/tests/deep.rs`

**Why:** D19 flagged this as a check rather than an assumption, and Phase 4a's corpus confirmed it at a tenth of the guessed depth. `rust/corpus/lang/deep_nested_expr.rex`, a 3000-term `1 + 1 + ...` chain that the oracle evaluates without complaint, makes `cargo test -p rexx-parse --test program` abort with a stack overflow.

**`rexx-parse` has three separate recursions with three very different cliffs**, measured on a default 2 MiB stack:

| recursion | cliff | fix |
|---|---|---|
| `block.rs::visit_expr`, a hand-written walk called from `add_clause` per clause | **2,450 terms** | iterative, this task |
| compiler drop glue for a `Box<Expr>` chain | ~10,000-20,000 | iterative, this task |
| `subterm`'s parenthesis descent | ~85,000 debug, sized thread | a counter raising 11.1, Task 3c |

The first is what blocks the corpus test: it runs *during* parsing, so `parse_program` aborts before a `Program` value exists for anyone to drop.

**An earlier draft of this task asserted the drop glue was the cause. It was wrong**, and the way it was caught is the method to reuse: `mem::forget` on the parsed result left the cliff exactly where it was, building the same tree without the parser showed `Drop` surviving six to eight times deeper, and a gdb backtrace named `visit_expr` directly. The original evidence — a stack overflow, plus the spike's 512 MiB thread parsing the file fine — is consistent with *any* deep-recursion hypothesis and therefore distinguishes none of them.

This is not cosmetic. A stack overflow aborts the process with no message and no exit code, which is the one outcome D19's "failing loudly" rule most wants to exclude, and it is reachable from ordinary user code that the oracle handles.

- [ ] **Step 1: Find the actual cliff, and which recursion owns it, before changing anything.** Measured: 2,449 terms parse and 2,450 abort. Separating the candidates is the work — `mem::forget` the result to take `Drop` out of the picture and see whether the cliff moves, build the same tree without the parser to measure `Drop` alone, and get a backtrace. Keep `examples/depth_probe.rs`, the instrument that settles it, and commit it with a doc comment recording the three cliffs as of today.

- [ ] **Step 2: Make `block.rs::visit_expr` iterative.** This is the recursion that blocks the corpus test, and it runs *during* parsing, so `parse_program` aborts before a `Program` exists to drop. It populates a `referenced` name set per clause that `GUARD` and exposed-variable handling read, so **preserve exactly which nodes it visits** — a walk collecting a different set is a behaviour change wearing a refactor's clothes. If iteration changes visit order, establish whether order is observable rather than assuming.

- [ ] **Step 3: Write the failing test** — parse an expression deep enough to abort today, on a normal test thread, and drop the result. Then also assert the case the oracle handles: 100,000 terms.

- [ ] **Step 4: Implement an iterative `Drop` for `Expr`.** The standard shape: take each child out with `std::mem::replace` into a worklist and drain it, so unwinding is a loop rather than a recursion. `Expr` owns children through `Box` and `Vec`, so every child-holding variant participates. Do not change the tree's shape and do not add `unsafe`.

- [ ] **Step 5: Check the neighbours.** `PartialEq`, `Debug` and any other derive that walks children recursively has the same exposure. Say in the report which of them you tested at depth and which you did not, rather than leaving the boundary to be inferred.

- [ ] **Step 6: Verify with `--no-fail-fast`**, and report all three affected binaries by name: `program.rs::the_corpus_exercises_at_least_one_directive_with_a_body`, `tiling.rs::every_corpus_program_tiles`, `variants.rs::every_variant_is_constructed_by_the_corpus_and_samples`. Cargo stops at the first failing binary by default and that has produced a false green here before. The suite must be green on a default stack with the 3000-term corpus program present and unmodified.

- [ ] **Step 7: Commit.**

```bash
git add rust/crates/rexx-parse
git commit -m "Walk and drop a deep expression tree iteratively, not by recursion"
```

---

### Task 3c: A depth counter on the parser's subexpression recursion

**Spec:** D19, "There are two cliffs, not one".

**Files:**
- Modify: `rust/crates/rexx-parse/src/expr.rs`, `src/error.rs`
- Test: `rust/crates/rexx-parse/tests/deep.rs` (created by Task 3b)

**Why:** nested parentheses recurse in the parser and nothing counts them. Measured against the oracle:

| `say ((((…'a'…))))` | oracle | ours today |
|---|---|---|
| 38,000 | rc 0 | rc 0 |
| 40,000 | **rc 245, Error 11.1** | rc 0 |
| 85,000 | rc 245 | rc 0 |
| 90,000 | rc 245 | **rc 134, SIGABRT, no message** |

We diverge in both directions, and the abort is the outcome the phase's failing-loudly rule exists to prevent. Unlike the evaluation-depth limit, this one is **parity, not a deviation**: the oracle raises 11.1 here, so raising 11.1 matches it.

Do this **after Task 3b**, not beside it: both touch `rexx-parse` and 3b's iterative `Drop` changes how deep trees behave.

- [ ] **Step 1: Pin the oracle's cliff more precisely than the bracket above.** It is between 38,000 and 40,000 parens. Bisect it, and record the number and the fact that it is a C++ stack artifact rather than a language constant, so nobody later mistakes it for a specification.

- [ ] **Step 2: Write the failing test** — a parenthesis nesting that aborts our parser today, asserting it instead raises 11.1.

- [ ] **Step 3: Add the counter** to `subterm`'s recursion, raising 11.1 at a limit set below our own abort cliff (measured at 90,000 in debug on a 512 MiB thread, so debug is what binds) and near the oracle's. **Exact depth parity is not achievable** — both cliffs are stack artifacts of two different implementations — so pick a limit inside the oracle's own reporting range and say in the doc comment that programs within a few thousand levels of it may diverge, and that no corpus program goes near either cliff.

- [ ] **Step 4: Check the other recursive descents in the parser** for the same exposure: nested `DO`/`SELECT` block structure, and any other place that recurses per source construct. Report which you tested and which you did not.

- [ ] **Step 5: Verify** — `cargo test --workspace` green, clippy clean, and the counter's limit recorded in the report alongside Task 3's 512 MiB / 784-bytes-per-level figures.

- [ ] **Step 6: Commit.**

---

### Task 3d: The nested-call recursion, which reaches the sized path

**Spec:** D19. **Files:** `rust/crates/rexx-parse/src/expr.rs`, `tests/deep.rs`, `examples/depth_probe.rs`.

**Why this and not the prefix-chain gap:** Task 3c deferred two unguarded recursions as equals. They are not. Nested calls, `f(f(f(…)))`, descend through `arg_list` rather than the grouping-paren arm `MAX_EXPR_DEPTH` guards, and Task 3c's review measured the oracle's side of it:

| depth | oracle | ours, default 2 MiB | ours, sized 512 MiB |
|---|---|---|---|
| 10,000 | parses, rc 213 (43.1 at run time) | abort | parses |
| 39,900 | **rc 245, Error 11.1** | abort | parses |
| 50,001 | rc 245, 11.1 | abort | parses, counter does not apply |
| 92,187 | rc 245, 11.1 | abort | **abort, rc 134, no message** |

So above roughly 92,000 the **sized** path — the one the executor actually runs on — aborts silently where the oracle reports a condition. That is the precise failure D19 exists to remove, and it is reachable today.

- [ ] **Step 1: Bisect our sized-path cliff and the oracle's**, unpiped, checking the build's exit status before trusting any number. Two figures in this phase were wrong because a pipe swallowed a failed build and the bisection measured a stale binary.
- [ ] **Step 2: Write the failing test** — a nesting deep enough to abort on the sized path today, asserting 11.1 instead.
- [ ] **Step 3: Count the `arg_list` descent**, sharing one depth budget with `MAX_EXPR_DEPTH` if a shared counter is defensible, or a second counter if the recursions genuinely differ. Say which and why.
- [ ] **Step 4: Fold in Task 3c's two corrections**, both from its review. The default-thread cliff is **331/332**, not the 337/338 stated in four places — the counter's own field and check cost about six levels, so the fix made the unprotected case slightly worse, and one of those four places is a test whose job is to document the gap accurately. And "nested calls are shallower than plain parens" is **backwards** in both the report and `depth_probe.rs`: measured like-for-like on one binary, parens are 331 and calls are 349. The priority it argued for is still right, since calls are the shallowest *unguarded* recursion; only the comparison is wrong.
- [ ] **Step 5: Two minors from the same review.** `parse_constant_expression`'s own `(` is uncounted, so `RAISE`, `FORWARD`, `USE ARG` and `ADDRESS WITH` effectively get 50,001 — harmless, worth a clause. And the paren test covers only the sized path, so nothing would notice `MAX_EXPR_DEPTH` being raised above the native cliff; a plain const comparison pins it.
- [ ] **Step 6: Verify and commit.** `cargo test -p rexx-parse --no-fail-fast`, clippy unpiped, and the whole workspace.

**Do not attempt a stack-aware counter.** Task 3c's review costed it: a remaining-stack query has no safe stable API, and a per-frame byte estimate is a class of number this phase has got wrong twice in one day. The long-term answer is a documented minimum stack or a sized entry point in `rexx-parse` itself.

---

### Task 4: The value model

**Spec:** D15 in full, including every transcript.

**Files:**
- Create: `rust/crates/rexx-exec/src/value.rs`
- Test: a `#[cfg(test)] mod tests` inside `rust/crates/rexx-exec/src/value.rs`

**Interfaces:**
- Produces: `Value` constructors and accessors on `Interp` — `text(&mut self, &[u8]) -> ObjRef`, **`number(&mut self, Number, created_digits: u32, created_form: Form) -> ObjRef`** (which applies the `SmallInt` admissibility rule), `to_text(&mut self, ObjRef) -> Cow<'_, [u8]>`, `to_number(&mut self, ObjRef) -> Result<Number, NotNumeric>`.

**`number` takes the digits and form explicitly, and that is the design rather than a convenience.** D15's whole point is that a value captures the pair in force *at the operation that produced it*, so the operation supplies them. Three consequences worth knowing before you start: the value model needs no ambient `Settings`, so there is nothing to seed before an activation exists; when Task 6 puts `Settings` on the `Activation`, nothing here changes; and `to_text` formats through the value's own captured pair, never the current one, which is the rule this task exists to enforce.

**Test with a `#[cfg(test)] mod tests` inside `src/value.rs`, not an integration test**, so `Interp` stays private and no public surface is widened for testing. Task 3's review landed exactly this lesson from the other direction: choosing an integration test there is what forced a public entry point.

**Do not build any expression evaluation.** An earlier draft of this task sketched its tests as `interp.eval_str("1 / 3")`, which does not exist and cannot without pulling Task 6's activation and Task 7's arithmetic forward — throwaway scaffolding Task 7 would then have to unwind. Construct the `Number` directly through `rexx-num`, which Phase 2 already tested, and let this task own only what happens to it afterwards. The transcripts below remain the source of truth: they are oracle-measured facts about **rendering**, which is this task's subject.

**Why:** every later task manipulates values through these four functions, and the two rules they enforce are the ones the oracle makes observable everywhere.

- [ ] **Step 1: Write the failing tests, straight from the spec's transcripts**

```rust
#[test]
fn a_numbers_rendering_is_fixed_when_it_is_created() {
    // numeric digits 9 ; y = 1/3 ; numeric digits 3 ; say y  ->  0.333333333
    let mut interp = Interp::new();
    interp.settings_mut().set_digits_str("9").unwrap();
    let y = interp.eval_str("1 / 3").unwrap();
    interp.settings_mut().set_digits_str("3").unwrap();
    assert_eq!(&*interp.to_text(y), b"0.333333333");
}

#[test]
fn numeric_form_is_captured_at_creation_too() {
    // numeric form engineering ; x = 1e10+0 -> 10E+9, and stays 10E+9.
    let mut interp = Interp::new();
    interp.settings_mut().set_form_str("ENGINEERING").unwrap();
    let x = interp.eval_str("1e10 + 0").unwrap();
    interp.settings_mut().set_form_str("SCIENTIFIC").unwrap();
    assert_eq!(&*interp.to_text(x), b"10E+9");
    let y = interp.eval_str("1e10 + 0").unwrap();
    assert_eq!(&*interp.to_text(y), b"1E+10");
}

#[test]
fn a_small_int_is_only_admissible_within_the_digits_of_its_own_operation() {
    // numeric digits 1 ; x = 15 + 0 ; x is 20, so x + 6 is 3E+1 while 15 + 6 is 2E+1.
    let mut interp = Interp::new();
    interp.settings_mut().set_digits_str("1").unwrap();
    let x = interp.eval_str("15 + 0").unwrap();
    assert_eq!(&*interp.to_text(x), b"2E+1");
    let sum = interp.eval_with("x + 6", &[("X", x)]).unwrap();
    assert_eq!(&*interp.to_text(sum), b"3E+1");
    let direct = interp.eval_str("15 + 6").unwrap();
    assert_eq!(&*interp.to_text(direct), b"2E+1");
}

#[test]
fn text_keeps_its_own_spelling_and_caches_an_exact_parse() {
    // x = '007' ; say x -> 007 ; say x + 0 -> 7
    // and the cache is exact, so it survives a DIGITS change:
    // x = '1.234567890123456789'; digits 5 -> 1.2346 ; digits 20 -> the whole thing
    let mut interp = Interp::new();
    let x = interp.text(b"007");
    assert_eq!(&*interp.to_text(x), b"007");
    let converted = interp.eval_with("x + 0", &[("X", x)]).unwrap();
    assert_eq!(&*interp.to_text(converted), b"7");
}

#[test]
fn nil_has_a_string_value_and_the_booleans_are_plain_strings() {
    // say .nil -> The NIL object ; .true is "1" ; .false is "0"
    let mut interp = Interp::new();
    assert_eq!(&*interp.to_text(ObjRef::NIL), b"The NIL object");
}
```

- [ ] **Step 2: Run them to watch them fail**

Run: `cd rust && cargo test -p rexx-exec value`
Expected: compile errors, no constructors yet.

- [ ] **Step 3: Implement**

Conversions are total. Text to number is `std::str::from_utf8` then `Number::parse`, and both failures collapse into `NotNumeric` because a Rexx number's characters are ASCII by definition. Number to text is `format_form(created_digits, created_form)` — **never** `settings.digits()`.

The `num` cache is tri-state and **holds the exact parse, never a rounded one**. Rounding belongs to the operation, which is what makes the cache safe across a settings change.

`number()` admits a `SmallInt` only when the value is whole, inside `SMALL_INT_MIN..=SMALL_INT_MAX`, and its decimal digit count is at most the `DIGITS` of the operation that produced it. The check happens once, at creation, and is never re-derived.

- [ ] **Step 4: Verify against the oracle, not just against the tests**

For each of the five transcripts, run the equivalent `.rex` under `( ulimit -v 1048576; build/bin/rexx … )` and paste both outputs into the task report. The tests encode my transcripts; this step checks my transcripts.

- [ ] **Step 5: Commit**

```bash
git add rust/crates/rexx-exec/src/value.rs rust/crates/rexx-exec/tests/value.rs
git commit -m "The value model: text keeps its spelling, a number keeps its rendering"
```

---

### Task 5: Stems and compound variables

**Spec:** D15a in full.

**Files:**
- Create: `rust/crates/rexx-exec/src/stem.rs`
- Test: a `#[cfg(test)] mod tests` inside `rust/crates/rexx-exec/src/stem.rs`

**Interfaces:**
- Produces: `tail_key(&mut self, &Plan, frame, SymbolId) -> Vec<u8>` resolving a compound's tail pieces; `stem_get`, `stem_set`, `stem_drop_tail`, `stem_assign`, `stem_drop`.

**Why:** four measured behaviours that the obvious model gets wrong, and the derived-name rule that every uninitialised read depends on.

- [ ] **Step 1: Write the failing tests from the six transcripts**

Each of these is a measured oracle transcript and the test asserts the same bytes:

```rust
// u. = 'd' ; u.1 = 'one' ; drop u.1  ->  u.1 is U.1, u.2 is d
// a. = 1 ; b. = a. ; a.1 = 2         ->  b.1 is 2      (one shared object)
// r. = 'rd' ; u = r. ; drop r.       ->  u is rd       (drop rebinds)
// s. = 'def' ; t = s. ; s. = 'other' ->  t is def      (assign rebinds)
// say q.                             ->  Q.            (name, with the period)
// i = 'abc' ; v.i = 'val'            ->  v.ABC is V.ABC (keys are verbatim)
// i = 1 ; j = 2 ; a.i.j = 'deep'     ->  a.1.2 is deep  (pieces joined by '.')
```

- [ ] **Step 2: Run them to watch them fail**

- [ ] **Step 3: Implement**

A dropped tail is `Some(key) -> None`, a **tombstone** that does not take the default; an absent key does. `stem_assign` and `stem_drop` **replace the Stem object** and rebind the variable, leaving the old object for anything that aliased it; a tail assignment mutates in place. Tail keys are the resolved piece values verbatim and case-sensitively, joined with `.` for a multi-level tail.

- [ ] **Step 4: Verify against the oracle** — same rule as Task 4, all seven transcripts re-run and pasted into the report.

- [ ] **Step 5: Commit**

```bash
git add rust/crates/rexx-exec/src/stem.rs rust/crates/rexx-exec/tests/stem.rs
git commit -m "Stems: tombstones, aliasing, and a name the object carries itself"
```

---

### Task 6: The resolution plan

**Spec:** D16 in full.

**Files:**
- Create: `rust/crates/rexx-exec/src/plan.rs`, `src/activation.rs`
- Test: a `#[cfg(test)] mod tests` inside `rust/crates/rexx-exec/src/plan.rs`

**Interfaces:**
- Produces: the `Plan` **already in the tree at `lib.rs:424`**, which Task 3 built:
  ```rust
  struct Plan {
      names: HashMap<Box<[u8]>, usize>,      // arrives as text: tail pieces, DROP (v), fragments
      by_symbol: HashMap<SymbolId, usize>,   // arrives as a SymbolId: the common case
  }
  ```
  plus `Plan::build(&CodeBody, &SymbolTable)` and, on `Interp`, a cache keyed by `(program_id, body_index)` where the loader assigns `program_id`.

  **Both maps point at the same slot index**, which is what makes a tail piece and a same-named variable share a slot rather than merely agree. An earlier draft of this task specified `Plan { slots, len }` with no `by_symbol` at all, which would have put a byte-string hash on the access path D16 motivates the whole design by costing at 8.1% of runtime and 32.2% on stem-heavy code.

  **The open choice, which `lib.rs:418` records rather than settles:** `by_symbol` is a `HashMap` where D16's shape wants an array index, and it cannot be one yet because `SymbolId` is a newtype over a private `u32` with no accessor, so nothing outside `rexx-parse` can index a `Vec` by one. Either ask for `SymbolId::index()` as a `rexx-parse` amendment, or keep the hash deliberately. Make the choice and record the reasoning; do not inherit it.
- `Activation { plan: Rc<Plan>, extra: HashMap<Box<[u8]>, usize>, frame: SlotFrame, blocks: Vec<Block>, pc: usize, settings: Settings, program: Rc<Program> }`.

**`extra` is not optional and Task 3 already proved why.** The plan is an `Rc`, shared and immutable, built by an upfront pass that never saw a name introduced at run time — and such names exist in 4a: `DROP (v)` names its target at run time, and an interpreted fragment's bindings are visible to the enclosing body's own later clauses (measured: `interpret "newvar = 7"` then `say newvar + 1` prints 8). Resolution is `plan.slot_of(name).or_else(|| extra.get(name))`; allocation writes `extra` and calls `grow_slots`. Task 3 built this; you are inheriting its shape, not inventing one.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn a_tail_piece_and_a_plain_variable_share_one_slot() {
    // b = 2 ; say a.b -> A.2 ; a.2 = 'hit' ; say a.b -> hit
    // An implementer who gives tail pieces their own slots gets A.B.
}

#[test]
fn a_runtime_name_grows_the_frame() {
    // v = 'X' ; x = 1 ; drop (v) ; say x  ->  X
    // X may not appear in the body at all, so the plan cannot have a slot for it.
}

#[test]
fn names_are_keyed_upcased_but_tail_values_are_not() {
    // The two rules live in different decision blocks and are easy to swap.
}
```

- [ ] **Step 2: Run them to watch them fail**

- [ ] **Step 3: Implement the upfront pass**

One walk over the body's AST, collecting every referenced name and every `Tail::Variable` piece from `compound_parts`, assigning dense indices. **Not lazy**: a lazy design threads a "seen this name?" check through every site that touches a variable, which is a different algorithm and the wrong one. Run-time growth is the exception, not the normal path.

`Settings` lives on the `Activation`, inherited from the caller at call time. Measured: an internal call sees the caller's `DIGITS`, changes its own, and the caller is unaffected after `return`.

- [ ] **Step 4: Verify** — `cargo test -p rexx-exec plan`, plus the three oracle transcripts.

- [ ] **Step 5: Commit**

```bash
git add rust/crates/rexx-exec/src/plan.rs rust/crates/rexx-exec/src/activation.rs rust/crates/rexx-exec/tests/plan.rs
git commit -m "Resolve a body's variables once, keyed by name, cached on Interp"
```

---

### Task 7: Expression evaluation, part one — terms, arithmetic, concatenation

**Spec:** "Expression evaluation", the arithmetic and concatenation groups.

**Files:**
- Create: `rust/crates/rexx-exec/src/eval.rs`
- Test: a `#[cfg(test)] mod tests` inside `rust/crates/rexx-exec/src/eval.rs`

**Interfaces:**
- Produces: `fn eval(&mut self, body: &CodeBody, expr: &Expr) -> Result<ObjRef, Raised>`.

- [ ] **Step 1: Write the failing tests** — `Literal`, `Constant` (`say 1e5` is `1E5`), `Variable`, `Stem`, `Compound`, `DotVariable` for the three admissible names, `Prefix` (`+ - \`), arithmetic `+ - * / % // **` through `rexx-num`, and `Abuttal` / `Blank` / `||`.

- [ ] **Step 2: Run to watch them fail**

**`Raised` is built here, not in Task 12, and the boundary matters.** Task 7's own Interfaces name it as `eval`'s error type, and arithmetic is one of 4a's raisers, so a divide-by-zero test that can only assert "fails loudly" is asserting the same thing an *unimplemented* construct produces — it cannot tell a working raiser from an absent one.

Build the payload, the propagation and `From<ArithError>`. Do **not** build the message catalogue, the two-line stderr format, the clause echo or the exit-code mapping: those are Task 12's, they need `rexx-inventory`'s generated table and oracle-captured expectations, and they are where the work is. Tests assert the **condition data** — that `1/0` raises 42.3 — not the rendered text or the process exit code.

Measured, so the tests assert something real: `1/0` and `1//0` give **42.3 at rc 214**, `'abc'+1` gives **41.1 at rc 215**, and `2**'x'` gives **26.8 at rc 230**. Arithmetic raisers therefore span majors 26, 41 and 42, not the two the error section names most often, and `256 - major` holds for all three — which is Task 12's rule to implement, not this one's.

- [ ] **Step 3: Implement**

Push every intermediate to `RootSet::push_temp` before any allocation that could collect while it is live. A value held only in a Rust local across an allocation is the defect class the root set exists to remove.

Every unimplemented `ExprKind` — `Call`, `QualifiedCall`, `Message`, `ClassResolver`, `List`, `VariableReference`, and any `DotVariable` beyond the three — takes the loud-failure path with the owning sub-phase named.

- [ ] **Step 4: Verify** — plus a `--release` run, because the temps discipline is what `debug_assert`s cannot check.

- [ ] **Step 5: Commit**

```bash
git add rust/crates/rexx-exec/src/eval.rs rust/crates/rexx-exec/tests/eval_arith.rs
git commit -m "Evaluate terms, arithmetic and concatenation"
```

---

### Task 8: Expression evaluation, part two — comparison and logic

**Spec:** "Expression evaluation", the comparison and logical groups, with their transcripts.

**Files:**
- Modify: `rust/crates/rexx-exec/src/eval.rs`
- Test: a `#[cfg(test)] mod tests` inside `rust/crates/rexx-exec/src/eval.rs`

- [ ] **Step 1: Write the failing tests, from the measured line**

```
' a' = 'a'  -> 1     '09'x'a' = 'a' -> 1     'a' = 'a'||'09'x -> 1
'a' = 'a '  -> 1     'a b' = 'a  b' -> 0     '' = ' '         -> 1
'01' = '1'  -> 1     ' 1 ' = 1      -> 1     'a' = 1          -> 0
'10' >> '9' -> 0     '10' > '9'     -> 1     'a' << 'a '      -> 1     '01' == '1' -> 0
```

The first row is the one that matters. An earlier draft of this plan said the string rule was "blank-pad the shorter on the right", which is wrong, and no test in the second and third rows can tell the two rules apart.

- [ ] **Step 2: Run to watch them fail**

- [ ] **Step 3: Implement all four families**

* Numeric-or-string `= \= <> >< > < >= <= \> \<`: **call `rexx-num`'s comparison and do not write a string comparison at all.**

**Step 3a comes first: amend `rexx-num` with a byte-slice entry point.** Today `compare` takes `&str` and re-parses both operands on every call. A Rexx string can hold bytes that are not valid UTF-8 (D14), so `&str` cannot carry one, and re-parsing defeats D15's cache, whose whole stated purpose is that a non-numeric string is not re-parsed on every comparison. Add an entry taking byte slices and already-decoded operands, keep the existing one, and do not duplicate `string_order`. It is the whole algorithm, string fallback included — `string_order` at `rust/crates/rexx-num/src/compare.rs:173`, ported from `RexxString::stringComp` (`StringClass.cpp:795`), which strips leading blanks *and tabs*, compares the shared prefix, and decides a leftover tail against a space. Writing a second one here means writing a divergent one.
* Strict `== \== >> << >>= <<= \>> \<<`: no padding, shorter is less.
* Logical `& | &&`: a logical value is **exactly** the one-character string `0` or `1`. Measured, `' 1 '`, `'01'`, `'1.0'` and `''` are each error 34.
* `ExprKind::Logical`, the comma list, is an AND of its parts under the same check.

- [ ] **Step 4: Verify** — every line above re-run under the oracle and pasted into the report.

- [ ] **Step 5: Commit**

```bash
git add rust/crates/rexx-exec/src/eval.rs rust/crates/rexx-exec/tests/eval_compare.rs
git commit -m "Comparison in two families, and logic that coerces nothing"
```

---

### Task 9: The instruction loop — assignment, SAY, DROP, NUMERIC, EXIT, LABEL, NOP

**Spec:** "Control flow", "Output and trace sinks".

**Files:**
- Create: `rust/crates/rexx-exec/src/run.rs`
- Test: a `#[cfg(test)] mod tests` inside `rust/crates/rexx-exec/src/run.rs`

**Interfaces:**
- Produces: `enum Flow { Next, Goto(usize), Exit(Option<ObjRef>) }` and `fn step(&mut self, body: &CodeBody, index: usize) -> Result<Flow, Raised>`.

- [ ] **Step 1: Write the failing tests** — assignment to a variable, a stem and a compound; `SAY` of each value kind and of an omitted expression (a blank line); `DROP` of a variable, a tail, a whole stem and the `(v)` indirect form, **using `RootSet::clear_slot`, which Task 2b added expressly for this**; `NUMERIC DIGITS`/`FUZZ`/`FORM` including the `VALUE` spellings; `EXIT` with and without an expression; a `LABEL` as a traced no-op; `NOP`.

- [ ] **Step 2: Run to watch them fail**

- [ ] **Step 3: Implement**

**Upcase an indirect name before resolving it.** `slot_of` deliberately does not upcase — that happens once upstream in `SymbolTable::intern`, before a `SymbolId` exists — but `DROP (v)` never goes through the scanner. Measured: `v = 'x'; x = 1; drop (v); say x` prints `X`, so the *value* is upcased before it names a variable. A resolution path that passes the raw bytes to `slot_of` misses an existing slot and silently allocates a second one for the same variable, which is the aliasing failure Task 2b's `growth_does_not_recycle_a_cleared_slot` exists to prevent, arriving by a different route.

**Do not write `ObjRef::NIL` to mean "dropped".** `x = .nil` is legal Rexx and `.nil` is a value, so the two states are observationally distinct: measured, `y = .nil; drop y; say y` prints `Y`, the derived name, while `x = .nil; say x` prints `The NIL object`. `clear_slot` exists because of exactly this.

`SAY` writes to the output sink, default stdout. Trace goes to the **trace sink, default stderr** — the two are separate descriptors, so their interleaving is not observable and two independently buffered sinks are safe.

- [ ] **Step 4: Verify** — `cargo test`, plus each instruction run under both interpreters through `rexx-run`.

- [ ] **Step 5: Commit**

```bash
git add rust/crates/rexx-exec/src/run.rs
git commit -m "The instruction loop, and the seven instructions that do not branch"
```

---

### Task 10: `IF`, `SELECT`, `SELECT CASE`

**Spec:** "Control flow", and the `WhenCase` rule.

**Files:**
- Modify: `rust/crates/rexx-exec/src/run.rs`
- Test: a `#[cfg(test)] mod tests` inside `rust/crates/rexx-exec/src/run.rs`

- [ ] **Step 1: Write the failing tests**

Include the two shapes that discriminate a wrong jump target, because Phase 3 cannot see either: an `IF`/`ELSE` chain where the false target and the then-exit differ, and a `SELECT` whose `WHEN` bodies are several instructions long with visible side effects, so a wrong exit lands inside a later `WHEN`'s body. Also `when 1 = 1 then` followed by `when 2 = 2 then nop`, where the second `WHEN` is the first's `THEN` instruction and is never collected into `whens`.

`SELECT CASE` compares with `==`: measured, `select case '007'` does not match `when 7`.

**A `WhenCase`'s comma is a value list, an OR of `==` tests, and this is the opposite of a plain `WHEN`'s comma.** Measured: `select case 2` with `when 1, 2 then say 'hit'` prints `hit`, while a plain `when 1, 2` on a non-logical value raises 34.6. The two commas parse into the same-looking node and mean opposite things, so an implementer who handles one and reuses it for the other gets a silently wrong answer rather than a failure. `ast.rs:801-815` records the distinction.

**Do not check a comma-list condition yourself.** A single-expression condition that is not `0` or `1` raises **34.1** under `IF` and **34.2** under `WHEN`. A comma list raises **34.6** from inside `eval_logical_list`, which already does it, and re-checking the result would replace 34.6 with 34.1. Measured across all four keywords, and the rule is that the sub-number is decided by the clause being a list at all, not by which element failed: `if 'x', 1 then` is 34.6, not 34.1.

A `SELECT` that reaches its `END` with no `WHEN` taken is **7.3**, and **the clause it echoes is the `END`, not the `SELECT`** (measured, rc 249). Raise it from the wrong arm and stdout and the exit code still match; only the stderr echo shows it.

`when 1 = 1 then` followed by `when 2 = 2 then nop` is **accepted, rc 0** (`ast.rs:776`, re-confirmed at run time). Do not confuse it with the *false*-condition variant, which segfaults the oracle and is upstream bug SF #2018, not ours to reproduce.

**The clause echo is indented by block nesting depth, two spaces per level, and `Raised::report` does not do this yet.** Measured: one enclosing `DO` gives two spaces, two gives four, three gives six; a `SELECT` contributes as well, and `do i = 1 to 'x'` gets none because the control expression is evaluated before the block is entered. This is the same indentation `TRACE` applies, so Task 13 shares it. It is unreachable today only because no block instruction exists; **this task is what makes it reachable**, and the first corpus program with a failure inside a block will diverge on stderr. Characterise the per-construct counting during implementation, since it is not simply one level per keyword.

- [ ] **Step 2: Run to watch them fail**

- [ ] **Step 3: Implement**

- [ ] **Step 4: Verify** — `cargo test`, plus each construct run under both interpreters through `rexx-run`, comparing stdout, stderr and exit code.

- [ ] **Step 5: Commit**

```bash
git add rust/crates/rexx-exec/src/run.rs
git commit -F <message-file>
```

---

### Task 11: `DO` and `LOOP` in every variant

**Spec:** "Control flow", D19's depth policy.

**Files:**
- Modify: `rust/crates/rexx-exec/src/run.rs`
- Modify: `rust/crates/rexx-exec/src/eval.rs` — **the evaluation-depth counter belongs here, not in `run.rs`.** `eval.rs:70-71` already says "Task 11 adds the limit check to this function", and the recursion the limit guards is `eval`'s.
- Modify: `rust/crates/rexx-exec/src/error.rs` — the 11.1 catalogue-family test lives beside the other catalogue tests. 11.1 is present in the generated catalogue as "Insufficient control stack space".
- Test: a `#[cfg(test)] mod tests` inside each file it touches

- [ ] **Step 1: Write the failing tests** — `Simple`, `Forever`, `Count`, `Controlled` with every combination of `TO`/`BY`/`FOR`, and `Over` on a **non-stem** target (measured: a string and a number each iterate once, yielding themselves). `LoopKind::With` is Phase 5's and takes the loud-failure path, because `DO WITH` sends `SUPPLIER` and nothing in 4a answers a message.

**`DO OVER` on a stem is out of scope and no test may use it.** That is not a judgement to re-make here: it is DEVIATION 1 in `phase-4-exclusions.txt`, which states the rule as "no corpus program may contain `DO OVER` on a stem", because the oracle walks a balanced tree and we use a hash map, so the orders are two different deterministic orders. `DO OVER` on a stem *does* work on the oracle and iterates in hash order, so it is easy to write a test for by accident.

**`COUNTER` and `OVER ... FOR` are in the AST and absent from this list.** Decide each explicitly, implement or take the loud path, and say which in the report. Do not leave them to fall through a catch-all.

**`WHILE` and `UNTIL` are missing from this list and are yours.** With them come **34.3** (`WHILE`) and **34.4** (`UNTIL`) for a single-expression condition that is not `0` or `1`. As with `IF` and `WHEN`, do not check a comma-list condition yourself: it raises **34.6** from inside `eval_logical_list`.

**Which clause the report echoes distinguishes a correct loop from one that evaluates its condition in the wrong place, and nothing else does.** Measured: `do until 'x'; end` echoes **`end`**, because `UNTIL` is tested at the bottom, while `do while 'x'; end` echoes the **`do`** line. A loop that evaluates `UNTIL` eagerly still produces the right values and the right exit code, and only the echoed clause reveals it.

Control expressions are evaluated in `Controlled::order`, which Phase 3 recorded because an expression can have side effects.

`LEAVE` and `ITERATE`, bare and by label, including from inside a `SELECT` nested in a loop. **Measured, and it is not what classic-Rexx intuition suggests:**

* An **ordinary clause label does not name a loop.** `outer: do i = 1 to 3` followed by `leave outer` is **28.3**, and `iterate outer` is **28.4**.
* What does work is `DO LABEL name`, and the **control variable's automatic label**: `leave i` or `iterate i` from inside a nested loop reaches the *outer* loop and unwinds the inner one.
* Bare `leave` in a simple `DO` block is **28.1**, but a **labelled** simple block is leavable.
* `leave sel`, where `sel` labels a `SELECT`, exits the `SELECT`.

**That last one is a coordination point with Task 10, and Task 10 has since shipped, so here is what it actually left you.** `LEAVE` must find and unwind through Task 10's `SELECT`s by label. Read the committed `run.rs` rather than reasoning from this plan; the three facts below were established after this section was first written, and two of them were mutation-verified.

* **`run_bounded` forwards a `Flow` it does not own, outward and unchanged** (`other => return Ok(other)`, and both the `If` and `Select` callers forward untouched). That is settled, not a contingency: a mutation that swallowed an unowned variant was applied and killed. So a `Flow` variant you add for `LEAVE`/`ITERATE` will propagate out of a nested `IF`/`SELECT` without either arm interfering.
* **But `run_bounded` absorbs any `Goto` whose target lands INSIDE its own range**, forwards or backwards, and does not tell the arm that produced it. Its doc guarantees the propagation case, which is the case that does not bite. The one that bites is yours: a `DO` inside an `IF`'s `THEN`, with an `ITERATE` in the loop body, computes a jump to the loop top, which is inside the `IF`'s range, so the `IF`'s `run_bounded` takes it directly and your `Do` arm re-enters as a **first entry** with its counter reset. That is exactly criterion 6's "`LEAVE` unwinds one block too few" mutation arriving by a route no document mentions. **So the invariant you must hold is: every block-stack change a `LEAVE`/`ITERATE` implies must be complete before the `Goto` is returned**, because you cannot rely on seeing that `Goto` again. State in your report which of your tests pins it.
* **`error.rs` is in your Files list for two reasons, not one.** The 11.1 catalogue entry is the obvious one. The other is `Raised::report`, which is where the clause-echo indentation belongs, and which nothing else points you at.

**The clause-echo indentation is yours, and it was recorded in four places with four different owners before it was recorded here.** Both other named owners are closed tasks, so this is the only statement of it that will reach an implementer. What the oracle does, measured, and re-verified by a reviewer including the two rows that were originally inferred:

* Two spaces per open block frame. One enclosing `DO` gives two, two gives four, three gives six.
* An `IF`'s `THEN` counts as **two** frames: `if 1=1 then say 2 & 1` indents four.
* A `SELECT` adds one more, so a `say` inside a true `WHEN`'s `THEN` indents six. `SELECT CASE`'s `THEN` is also six, and an `ELSE IF` chain is eight.
* `do i = 1 to 'x'` gets **none**, because its control expression is evaluated before the block is entered.

`Raised::report` emits no indentation today, so every corpus program that raises inside a block currently differs from the oracle on stderr, and that is the largest single source of remaining divergence. Two warnings. **The quantity is shared with `TRACE`'s own `*-*` indentation (Task 13), so put it somewhere Task 13 can use rather than inline in the report formatter** -- one quantity, two formatters. And Task 10's report concludes the depth is **derivable from the AST statically, with no runtime block stack**; every probe behind the measured rule was a raising `say`, and the claim that `TRACE` indents identically is asserted rather than measured, so **measure the trace half yourself before sharing the mechanism**.

**Two more things this section did not say.** `MAX_EXPR_DEPTH` in `rexx-parse` is 50,000 and raises the **same** 11.1 from the parser, so a depth test built from nested parentheses goes green without `eval`'s counter ever firing: build the depth test from a left-deep operator chain, not from parentheses. And `End`/`EndStyle` for `DO`/`LOOP` are yours and appear nowhere in the steps below.

- [ ] **Step 2: Measure the `DO` control error family before implementing it**

Measured, and note the paraphrase an earlier draft used was wrong in a way that would have pointed your probes at the one case that never raises:

| clause | oracle |
|---|---|
| `do i = 'a' to 3`, `do i = 1 to 'x'`, `do i = 1 by 'x'` | 41.1 |
| `do i = 1 to 3 for 'x'`, `for -1`, `for 1.5` | **26.3**, the `FOR` count |
| `do 'a'`, `do -1`, `do 2.5` | **26.2**, the `DO` repetitor |
| `do i = 1.5 to 3` | **no error** — a non-whole *control* value is legal |
| `do i = 1 by 0 to 3` | **no error**, loops forever — behaviour to reproduce, not an error to catalogue |

Re-run each row yourself and put the table in the report; two accounts of this family have already disagreed.

- [ ] **Step 3: Implement**, including the depth counter D19 requires.

Its limit is bounded on **both** sides: at least 100,000, the oracle's largest measured passing depth, and below what Task 3's stack size and per-frame cost allow. An upper bound alone is satisfied by a limit of 20,000, which diverges on every program between there and 100,000.

**Task 3 measured these and the answer is not the one this plan first recorded, so inherit the corrected numbers.** The interpreter thread is **512 MiB**. **`eval` binds the stack, and the figure moves whenever `eval`'s shape does — treat the rule as the deliverable and the number as perishable.** After Task 7 grew `eval_node` to fifteen match arms it went from ~783 to **~1600 bytes per level in debug**, roughly 335,000 survivable levels, still clearing D19's 100,000 minimum by more than three times. That is the **fourth** value this figure has taken in two days: ~820, then ~850, then ~783 once three tree walks became iterative, now ~1600. Every one was correct for the code it measured. **Re-measure at implementation and do not quote a predecessor's number.**

The historical figure, superseded: ~783 bytes per level with roughly 685,000 levels in debug. Measured on the current tree and independently confirmed: 600,000 and 684,000 levels pass, 700,000 aborts at rc 134.

| what runs | deepest surviving | bytes/level |
|---|---|---|
| parse and drop only | no cliff below 4,000,000 | under 134 |
| parse, plan and drop | 3,354,442 | ~160 |
| all four, including `eval` | 684,618 | **~783** |

`Plan::note` is now the crate's own remaining recursion at ~160 bytes per level, and is a candidate for the same worklist treatment if anything ever needs it.

**This number has moved twice in a day, and the reason is worth more than the number.** It was first recorded as ~820, then corrected to ~850 by a finer bisection, and both are now void — because Task 3b made three tree walks iterative and thereby *removed the recursions those measurements were measuring*. A measurement of the code is only valid for the code it measured. **Re-run the bisection when you implement this**, and treat the table above as the shape of the answer rather than the answer.

Two lessons recorded alongside it. The ~820 figure was wrong in a way visible without re-measuring: its table had parse-and-drop costing *less* per level than parse-plan-and-drop, so adding a phase appeared to make each level cheaper, which no model of sequential phases produces — incoherence in a table is a finding even before you check the arithmetic. And `eval` binding again at ~783 is now the strongest of the three figures, because an independent in-`eval` probe reported 784 and the external bisection agrees to 0.2 per cent, which is two methods rather than one.

**The recommendation of 100,000 is unchanged; only its justification moved**, and the headroom is now about 6.5x rather than 6x.

**Set the limit at exactly 100,000, raising 11.1 for anything deeper**, and mind the off-by-one: a 100,000-term expression *reaches* depth 100,000, so the check must fire **above** 100,000 rather than at it, or the one depth the oracle is known to survive becomes the first one we refuse. Higher limits reproduce nothing that exists and only widen the window where we succeed while the oracle SIGSEGVs.

**And know what this limit cannot do.** A counter in `eval` does not close the abort path, because a program can reach a deep tree without evaluating it at all: `exit` followed by a 700,000-term expression aborts inside the `Drop`, with nothing evaluated and no counter in `rexx-exec` in a position to see it. That path is closed by Task 3b's iterative `Drop`, not here. Do not write a doc comment claiming this limit makes deep expressions safe.

- [ ] **Step 4: Verify** — plus an expression at a depth the oracle handles comfortably, and a **test that reaches the 11.1 raise**, since no differential program can cross our limit without also crossing the oracle's cliff, and without that test the depth path is untested by construction.

> **That test cannot be a plain `#[cfg(test)]` unit test, and writing one is the trap this step used to set.** The only sized stack in the workspace is inside `run_program` (`lib.rs`), the public entry point; a libtest thread gets 2 MiB by default. At roughly the per-level cost quoted above, `eval` dies natively somewhere around a thousand levels, far below the limit you are trying to trigger, and it dies as a **guard-page abort with no message**, which is precisely the silent death D19's limit exists to prevent. Worse, the cheapest-looking way to make a failing test pass is to lower the limit, which would quietly defeat the whole decision.
>
> So drive it through `run_program`, which is what puts you on the sized thread. `tests/spike.rs` and `tests/corpus.rs` both already do this. If that means this one test is an integration test while the rest of your work is unit-tested in place, that is correct and not a constraint violation: the constraint forbids integration-testing a **private** subject, and `run_program` is public cross-crate surface. Say in your report how you confirmed the test reaches your counter rather than the parser's 50,000 or the guard page.

The oracle's cliff is between **100,000 and 150,000** terms, not at 200,000: 100,000 prints its answer, and both 150,000 and 200,000 exit 139. A corpus rule phrased against 200,000 would admit a 150,000-term program that SIGSEGVs.

- [ ] **Step 5: Commit**

```bash
git add rust/crates/rexx-exec/src/run.rs rust/crates/rexx-exec/src/eval.rs rust/crates/rexx-exec/src/error.rs
git commit -F <message-file>
```

---

### Task 12: Errors, the message catalogue, and the exit code

**Spec:** "Errors, and the reporting subsystem".

**Files:**
- Create: `rust/crates/rexx-exec/src/error.rs`
- Modify: `rust/crates/rexx-exec/Cargo.toml` (add `rexx-inventory`)
- Test: a `#[cfg(test)] mod tests` inside `rust/crates/rexx-exec/src/error.rs`

**Why:** criterion 1 compares stderr and the exit code byte for byte, so "terminates with the oracle's message" is a subsystem, not a sentence.

- [ ] **Step 1: Enumerate the raiser families against the oracle before writing anything**

The list is open, not closed, and an earlier draft of this plan named four families and missed two. Confirmed so far: arithmetic; 7.3; the logical-value checks, which are **six** sub-numbers — 34.1 `IF`, 34.2 `WHEN`, 34.3 `WHILE`, 34.4 `UNTIL`, 34.6 the comma list, 34.901 for `&`/`|`/`\`; the `NUMERIC` instruction's **26.5**, **26.6** and **33.1**, where major 33 is a family the earlier draft did not have at all; the `DO` control conversions (41.1 confirmed, 26.2 and 26.3 reported and unconfirmed); and `do i over .nil`, which is **98.913** at rc 158 from two constructs both in 4a's scope. Walk 4a's instruction and expression surface for raisers rather than trusting this list, and put the table in the report. And one more, measured and recorded nowhere until now: the `TRACE` instruction's numeric form. `trace 5` outside interactive debugging is **24.901**, "Numeric TRACE requests are valid only from interactive debugging", so `Trace::Skip` is a raiser rather than the silent no-op it resembles — which also settles that all four `Trace` variants are 4a's, a scope question the spec never answered.

- [ ] **Step 2: Capture the oracle's exact output for each family**

Measured for 7.3, and this is the format to reproduce exactly, two spaces after each colon:

```
     3 *-* end
Error 7 running /abs/path/vB.rex line 3:  WHEN or OTHERWISE expected.
Error 7.3:  All WHEN expressions of SELECT are false; OTHERWISE expected.
rc=249
```

Note the clause echo appears **with trace off**. Capture the same for 34.1 (measured rc 222), the arithmetic family, and the `DO` control numbers from Task 11.

- [ ] **Step 2: Write the failing tests** against those captures.

- [ ] **Step 3: Implement**

The message text comes from `rexx-inventory`'s generated table — Phase 0 already generates `errors.rs` from `rexxmsg.xml`, 704 messages. Do not hand-transcribe text the tree already generates. Arithmetic's text comes from `rexx-num`'s `ArithError::message()`.

`exit code = 256 - major`. The **loud-failure** code for unimplemented constructs must sit outside 157..253, where `256 - major` lives, or a not-implemented failure is indistinguishable from error 11; state the chosen code in a doc comment.

- [ ] **Step 4 and 5** as before, committing `src/error.rs`, the test and the manifest.

---

### Task 13: Trace

**Spec:** D17, and exit criterion 3.

**Files:**
- Create: `rust/crates/rexx-exec/src/trace.rs`
- Modify: `rust/crates/rexx-exec/src/eval.rs` — the value-line events (`>L>`, `>V>`, `>O>`, `>=>`) are emitted from expression evaluation, which means threading an event through roughly eighteen `eval_node` arms.
- Modify: `rust/crates/rexx-exec/src/run.rs` — the `*-*` clause event, and `InstructionKind::Trace`'s own `step` arm, which no step below mentions and which nothing implements today.
- Test: a `#[cfg(test)] mod tests` inside `rust/crates/rexx-exec/src/trace.rs`, with the committed expectations under `rust/crates/rexx-exec/tests/trace_oracle/`

> **There are zero trace emission points in `eval.rs` or `run.rs` today**, and the only writer to the trace sink is `execute`'s error path. So this task adds the hook to code that already works, rather than finding it in place.
>
> **An earlier version of this note called that a breach of D17 and a process failure. That verdict was wrong and is withdrawn.** D17 does say "emit the event from the start", but its stated purpose is to forbid constant folding and expression fusion so the evaluator is not designed twice, and the shipped `eval_node` does neither, so the harm D17 names never occurred. This plan has also scheduled trace at Task 13 since revision 1, which means Tasks 7 to 9 followed the governing document exactly; a brief carrying all of D17 would not have changed what they built. The note is kept rather than deleted because recording a sound decision as unimplemented invites a later reader to reopen it.
>
> **The retrofit is also far smaller than that note claimed.** It said roughly eighteen `eval_node` arms; there are ten top-level `ExprKind` arms, and more to the point `eval` was already split from `eval_node` precisely so that every exit path including the `?` ones goes through one place. A post-order value event with the value in hand is therefore **one** insertion point in `eval`, not a threading job across arms.
>
> Two things from that note stand on their own merits and are not withdrawn. Sequence this task **after** Task 11, because `run.rs` is the most contended file in the phase, and **read the committed `run.rs`** rather than designing against this plan.

- [ ] **Step 1: Build the prefix table from the oracle's side, not from what we emit**

All 19 prefixes at `RexxActivation.hpp:90`-`110`. Each row is either a witness program 4a emits, or the sub-phase that first emits it. Measured reachable from pure-4a code: `*-*`, `>>>`, `>=>`, `>L>`, `>V>`, `>O>`, `>K>`, `>C>`, and a prefix-operator line.

- [ ] **Step 2: Capture the oracle's exact bytes for each witness** — spacing, quoting and indentation are unspecified anywhere but the oracle. Commit the expectations the way `rexx-parse/tests/sourceline_oracle/` does, with a regeneration command named in the reading test, so `cargo test` alone is the gate.

> **This is the opposite strategy from the corpus runner, deliberately, and both are right.** `tests/corpus.rs` runs the oracle **live** and compares in process, because it exists to track progress across tasks and a committed expectation would have to be regenerated on every task that changes behaviour. Trace expectations are committed instead, because a trace witness is a fixed artefact whose whole value is that it cannot drift silently, and because capturing it requires a running oracle that a machine checking out this tree may not have. Neither section knew about the other when they were written, so the difference looked like an inconsistency; it is a real distinction between an instrument and an expectation. Do not "unify" them.

- [ ] **Step 3: Implement**, emitting to the trace sink (stderr) per evaluation step, gated on the setting.

- [ ] **Step 4: Verify** — `trace r` and `trace i` byte for byte against every committed expectation.

- [ ] **Step 5: Commit**

```bash
git add rust/crates/rexx-exec/src/trace.rs rust/crates/rexx-exec/src/eval.rs rust/crates/rexx-exec/src/run.rs rust/crates/rexx-exec/tests/trace_oracle
git commit -m "Trace value lines, quantified from the oracle's nineteen prefixes"
```

---

### Task 14: The corpus, and the L0 differential harness

**Spec:** "L0 differential corpus".

**Files:**
- **`rust/corpus/phase-4a.txt` ALREADY EXISTS** with 26 entries, amended during the phase. Add to it; do not create it. Plus 12 to 15 new `rust/corpus/lang/*.rex`
- **`rust/crates/rexx-exec/tests/corpus.rs` ALREADY EXISTS** and is the differential runner, built early on purpose so every later task can see its own effect. It runs the oracle **live** rather than against committed expectations, and reports in a non-gate mode by default with a strict mode the gate flips. `tests/corpus_oracle/` was never created and is not needed: see the note under Task 13's Step 2 about which strategy applies where
- Modify: `rust/corpus/README.md`

**Why:** of the 28 existing corpus programs only 10 are 4a-clean, **none of the 10 contains a `LEAVE` or an `ITERATE`**, and seven are numeric, so the inherited corpus mostly re-tests `rexx-num` through a new front end.

- [ ] **Step 1: Write the programs**

Start with a 4a-only cut of `do_variants.rex`, which is excluded today by the single line `do i over .array~of("x", "y")`. Then: the four control-flow shapes from criterion 1; `LEAVE`/`ITERATE` by label; whole-stem versus tail `DROP` including the tombstone; `EXIT` with an expression; the created-digits and created-form transcripts; the stem-aliasing transcripts; an expression at a safe depth.

**No program may contain `DO OVER` on a stem** — the iteration order is a recorded deviation (D15a).

- [ ] **Step 2: Fix `corpus/README.md`'s hard-coded "24 programs"** to report rather than assert. It is the count-rot this project warns about, in the document the corpus rules come from.

- [ ] **Step 3: Build the harness as a `cargo test`** with the oracle's expected output committed, so `cargo test` alone is the gate and a script regenerates. Report the program count; never assert it.

- [ ] **Step 4: Run it**, and separately under collect-on-every-allocation.

- [ ] **Step 5: Commit.**

---

### Task 15: The `base/expressions` assertion table

**Spec:** "L1, and why it is table-driven", exit criterion 2.

**Files:**
- Modify: `rust/crates/rexx-extract/src/lib.rs`, `src/bin/rexx-extract.rs`
- Create: `rust/crates/rexx-exec/tests/assertions.rs`

**Why:** `rexx-extract`'s current rendering produces programs whose main body is empty, so they execute nothing at all — verified under the oracle, which prints nothing and exits 0. The route is data, not programs.

- [ ] **Step 1: Add an extraction mode emitting one row per assertion** — the expression text, the expected value, and the `NUMERIC DIGITS` in force.

Those files change the setting throughout, from 1 to 100, so the extractor **scans sequentially and carries the setting**. Getting this wrong silently tests the wrong precision and still passes, which is the worst available outcome, so it gets its own test against a file that changes the setting mid-way.

- [ ] **Step 2: Include `PRECEDENCE` (1,226), which is self-contained literal arithmetic.** Phase 2 excluded it because it had no parser; 4a has one.

- [ ] **Step 3: `CONCATENATION` (388) needs a prelude, and adding it naively passes while testing nothing**

Every assertion in that group references variables `a` through `g` assigned at the top of its test method. A row of (expression, expected, digits) cannot carry them, and **part** of the resulting failure is silent: with `a`..`g` unset, each renders as a distinct single-character name, so any row whose expected value happens to match that all-distinct pattern passes while testing nothing. Measured, that is the 56 strict `==` and `\==` rows of the 388. The other 332 use non-strict `=` and turn on real blank-padding equalities between different variables, so they fail visibly instead — line 71 expects `0 0 1 1 0 1 0`, which unset operands do not produce.

Every row still needs the real prelude to mean anything. The reason to state the split precisely is that "all 388 would pass silently" was the original claim, and it is wrong: a reader who checks it, finds 332 loud failures, and concludes the hazard was imagined would then add the group naively.

So a row carries the method's **assignment prelude**, and any assertion whose prelude cannot be represented is listed as blocked rather than quietly included.

- [ ] **Step 4: Compare byte for byte, never numerically** — a numeric comparison would hide the entire created-digits and created-form story across thousands of rows.

- [ ] **Step 5: Prove the table can fail.** Perturb an expected value and confirm that row fails. A table that cannot fail is exactly the defect this criterion already had once, when it quantified over extracted programs that executed nothing.

- [ ] **Step 6: Report the row count and list rows blocked on 4b or 4c** with the sub-phase that unblocks each.

- [ ] **Step 5: Commit.**

---

### Task 16: The gate harnesses

**Spec:** the 4a exit gate, all seven criteria.

**Files:**
- Create: `rust/crates/rexx-exec/tests/coverage.rs`, `tests/loud.rs`, `rust/scripts/mutate-4a.sh`
- **`docs/superpowers/plans/phase-4-exclusions.txt` ALREADY EXISTS** and is ahead of this plan; Step 4 below says what is still owed and why writing it would regress it

- [ ] **Step 1: The coverage enumeration** — a macro-generated match with **no wildcard arm** over `InstructionKind`, `ExprKind`, `LoopKind`, `PrefixOp`, `EndStyle`, `Trace` and `Operator`. Every variant carries either a witness program in the subset or the phase that owns it, and the test fails on a variant carrying neither.

**Take variant identity from the variant, never from `keyword()`.** `InstructionKind::keyword()` maps **both** `When` and `WhenCase` to `"WHEN"` (`ast.rs:912`), so a test keyed on it lets any `WHEN` silently satisfy `WhenCase` — the coverage number stays green while a variant goes unwitnessed. Found by a gap analysis whose own first run made exactly that mistake.

**`Operator::Backslash` carries an owner string, not a witness.** It cannot appear in a `Binary` node by design: `\` is prefix-only and a dyadic one is error 35.1. Demanding a witness would demand a program that cannot exist, which is the `LoopKind::With` shape one enum over.

The owner arm is an escape unless it is policed, so: the owner string must be one of the phases named in the spec's split table or its "assigned elsewhere" paragraph, and the **set** of out-of-4a variants is asserted, the way the exclusions file is. Otherwise a variant that turns out hard can be relabelled Phase 5's instead of getting a witness. The assignment is complete today — 40 `InstructionKind` variants (20 in 4a, 9 in 4b, 4 in 4c, 6 in Phase 5, 1 in Phase 7) and 15 `ExprKind` variants (9 in scope, 6 failing loudly) — so the assertion costs nothing to add now. Without the owner arm this criterion demands a witness for `LoopKind::With`, which needs Phase 5, and the criterion written to close a blindness finding would itself be unsatisfiable.

- [ ] **Step 2: The loud-failure enumeration** — for every `InstructionKind` and `ExprKind` variant, either 4a executes it or it produces the not-implemented exit code and names its owner. One test closes a surface larger than 4a's own.

- [ ] **Step 3: The mutation control** — a committed list of one-line mutations, each of which the subset must catch: off-by-one on `If::false_target`, on `When::exit`, on `Loop::end`; `Controlled::order` in fixed To/By/For order; `Abuttal` as `Blank`; `=` as `==`; `LEAVE` unwinding one block too few; formatting with the current digits, and with the current form, instead of the created pair.

The script **exits non-zero on an unapplied pattern**. That guard fired in four separate Phase 3 tasks, and without it a stale pattern reports coverage that does not exist. This is the one criterion a `cargo test` cannot be, since it edits the source it tests.

- [ ] **Step 4: `phase-4-exclusions.txt` ALREADY EXISTS and is ahead of this step. Do not write it, and do not follow this step literally.** It holds the 15 whole exclusions, the 3 partial rows, a **separate deviations section** (a deviation is permanent and chosen, an exclusion is work assigned to a later phase, and filing one as the other is how a deviation stops being reviewed), **and** a KNOWN GAPS section this step never described, pinned asymmetrically on purpose: adding a row needs no permission, removing one needs an owner or a measurement. The file has gained rows during the phase, including a second deviation and several gaps. Writing it from this step would **regress** it.
>
> What this task still owes the file is the **set assertion in the harness**, so the enumerated exclusions cannot drift from the file, and syncing the 66-of-81 phrasing. Nothing else.

- [ ] **Step 5: Assess every criterion, writing `docs/superpowers/plans/phase-4a-gate.md`.** That file does **not** exist and is this step's output, not its input; the seven criteria live in the design spec's "4a exit gate" section. Follow the shape of `phase-3-gate.md`: state what was measured, and where a criterion is met but weak, say so. A gate that reports only "met" is worth less than one that says which of its criteria could not have failed.

- [ ] **Step 6: Commit.**
