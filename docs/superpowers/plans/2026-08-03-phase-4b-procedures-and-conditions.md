# Phase 4b implementation plan: procedures and conditions

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** run a classic Rexx program that calls internal routines, isolates and exposes their variables, traps conditions and interprets text, byte-for-byte as `build/bin/rexx` runs it.

**Architecture:** 4a built one activation that runs one body. 4b makes the activation stack real: one Rust frame per activation with an explicit depth counter, settings and trace mode inherited per frame, a variable pool that is *shared* by default and isolated only by `PROCEDURE`, and an error report that carries a stack of sites rather than one. No new crate; `rexx-exec` grows.

**Tech stack:** Rust 1.96.1, no `unsafe`, `cargo fmt` default, `clippy -D warnings`. Depends on `rexx-core`, `rexx-num`, `rexx-parse`, `rexx-inventory`.

**Revision note.** This plan was reviewed adversarially before execution by four independent reviewers, and the first revision was substantially wrong. Two of its four "corrections to inherited items" were themselves wrong, its activation-indent rule was refuted, and it omitted an entire construct. Everything below marked *measured 2026-08-03* was re-measured after that review. The reports are in the session scratchpad and are not required reading; what they found is folded in here.

## The governing documents, and what each is for

* **`docs/superpowers/specs/2026-08-01-phase-4bc-scoping.md`** is the material this plan was written from. Twenty-nine of its 37 inherited items are reproduced in the task bodies below. The other eight (I23, I24, I29, I30 are 4c's; I32 to I35 are unowned) are in "Explicitly not in scope" at the end. **Do not read the scoping document to find your requirements** -- everything a task needs is in that task, and the scoping document contains claims this plan has since refuted.
* **`docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md`** still governs the value model, the borrow shape, and decisions D15 to D19. Read the section a task names.
* **`docs/superpowers/plans/phase-4-exclusions.txt`** is the live record of what Phase 4 does not do. Adding a KNOWN GAP row needs no permission; removing one does.
* **`docs/superpowers/plans/phase-4a-gate.md`** is the criterion set 4b's gate derives from.

---

## Global constraints

Every task's requirements implicitly include this section. **It is not extracted into task briefs, so a task that depends on one of these lines restates it.**

* **The C++ tree is the oracle and is never modified.** `interpreter/`, `samples/`, `build/`, `ootest/` are read-only.
* **Wrap every oracle invocation** as `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`. Without the ulimit the interpreter requests gigabytes mid-range and is OOM-killed, which has already cost a session and the machine's memory.
* **Read stdout, stderr and exit status as separate descriptors.** Comparing `2>&1` as one string produced two false regressions in 4a: the interleaving of the trace sink and stdout is undefined by design (D17). Read exit status unpiped -- a shell pipeline reports the last command's status.
* **Never set `NUMERIC DIGITS` above 1000** in a probe.
* **Never instantiate `.Package~new`** on a file inside the repository: it executes that file's prolog and has written untracked files into the tree.
* **Never probe `select; when 1 = 0 then; when 2 = 2 then nop; end`.** It segfaults the oracle (upstream SF #2018).
* **Beware Rexx literal syntax in probes.** A symbol named `x` or `b` immediately followed by a quoted string parses as a hex or binary literal, so `say '['x']'` is error 15.3, not concatenation. This cost a probe in planning. Use other names.
* Scratch files go in the session scratchpad, never in the repository.
* **No `unsafe`.** The workspace sets `unsafe_code = "forbid"`. If a task appears to need it, stop and report BLOCKED.
* **Never `git add -A`.** Stage the exact paths the task names. Do not run `git reset --hard`, do not force-push.
* Comments state the contract at the top and the reasoning at the decision point. Never delete an existing comment to make a change easier. Prefer `--` over an em-dash. The "no structuring semicolons" rule does **not** apply to this repository; it was imported by mistake in 4a and withdrawn. Reviewers should not raise it.
* A value's rendering is fixed when the value is created. Any code that formats a number with `settings.digits()` or `settings.form()` instead of the value's own captured pair is wrong; see D15.
* Anything Phase 4b does not implement **fails loudly**: `NOT_IMPLEMENTED_EXIT`, outside 157..253 where `256 - major` lives. Never a plausible Rexx condition. **Note what this does *not* say.** Measured 2026-08-03, the emitted text is `format!("{name} is not implemented")` (`src/lib.rs:444` and `:468`) and names **no phase**. The 4a plan claimed loud messages name the owning sub-phase; they never have. Task 0 makes that true; until Task 0 lands, do not write a test asserting a phase name appears in stderr.
* **Every new allocation site goes through `Interp::alloc_with`** (`src/lib.rs`, the wrapper around `Heap::alloc_with_uncollected`). `Heap::alloc_with_uncollected` and `Heap::alloc` both bypass the collect-on-every-allocation stress hook, and the first is named that way so a site written the natural way announces the bypass at the call site.
* **Commit first, then read the hash back, then record it.** Two ledger entries in 4a carried invented hashes because the hash was written from memory before the commit existed.
* `rustfmt` needs `--edition 2024`. The bare form defaults to 2015 and rejects let-chains.

---

## Decisions taken for 4b

Nine decisions bind 4b and are settled here. Five bind only 4c (D4 builtin exclusions, D7 `ExprKind::List`, D8 the `rexxcps` gate, D11 `RANDOM`/`DATE`/`TIME` determinism, D12 the `base/bif` extractor) and are deferred, with one documentation half pulled forward under D7.

### D1 -- the `INTERPRET` spike is replaced, not extended

`run_program_interpret_spike` (`src/lib.rs:1114`) is deleted, along with the `interpret_spike` **field** on `Interp` (`src/lib.rs:784`), its constructor parameter (`:830`), and the `execute` parameter (`:1169`). **There is no method named `Interp::interpret_spike`** -- the first revision of this plan told an implementer to delete one, and none has ever existed.

### D9r -- the variable pool is shared by default, and `PROCEDURE` isolates it

**This replaces the first revision's D9 entirely.** That version had the caller grow its own frame at `CALL` time and push the callee's frame "with those slots aliased". Three things were wrong with it: there is no aliasing in `RootSet`, its justifying probe did not discriminate, and it omitted the default case.

**The default case, which the first revision never mentioned in twelve tasks.** Measured 2026-08-03:

```rexx
v = 'caller-v'
call sub
say 'caller sees:' v w
exit
sub:
say 'callee sees v:' v
w = 'callee-w'
return
```

```
callee sees v: caller-v
caller sees: caller-v callee-w
```

A routine **without** `PROCEDURE` shares the caller's entire variable pool: it reads the caller's variables and its writes survive the return. This is the default for internal routines in classic Rexx, and it is most of what 4b's corpus will contain.

**Why the first revision's storage argument was wrong.** `Plan::build` (`src/plan.rs`) walks the whole flat instruction list of one `CodeBody`. An internal routine is a label *inside that same body*, so caller and callee share one `CodeBody`, one `Plan`, and one name-to-slot map. The caller's frame is sized `plan.len()`, so it already has a slot for every name mentioned anywhere in the file -- including names mentioned only inside a callee. The probe offered as discriminating (a caller that "never mentions" an exposed name) does not discriminate, because the name appears in the callee's own instructions and those are in the caller's plan.

**The design.**

* A callee **without** `PROCEDURE` gets **no new slot frame**. Reuse the caller's `SlotFrame` (it is `Copy`), save and restore `pc`, and do not `pop_slots` on return. Settings and trace mode are still per activation.
* A callee **with** `PROCEDURE` gets a fresh frame of the **same size**, so slot indices are identical between the two frames. An exposed name at plan index *i* in the callee aliases index *i* in the target frame. The redirect is therefore a bitset over slot indices plus one target `SlotFrame` -- not a name-keyed map.
* **Exposure is transitive.** Measured: `a` exposes `n` to `b`, `b` exposes the same `n` to `c`, `c` writes it, and `a` sees `set-by-c`. So the bind must **chase the caller's own alias table**: binding `c` resolves through `b`'s alias to `a`'s frame. Binding `c` to `b`'s frame gives a silently wrong value two levels up, which is exactly the failure `grow_slots`'s doc comment wants to prevent.
* `RootSet::grow_slots`'s top-frame-only invariant is untouched by this design, so its `4a invariant` panic message and the `#[should_panic(expected = "4a invariant")]` test in `crates/rexx-core/tests/collect.rs` stand. The residual cases that could still need a grow are a computed `expose (v)` naming a symbol that appears in no instruction, and a name introduced by `INTERPRET`. Route both through `Plan::slot_of` (`src/plan.rs:548`), which checks plan then `extra` then grows and is therefore **idempotent** -- an implementer who instead writes "if not in plan, grow" leaks one slot per call, and `do 100000; call sub; end` leaks 100,000 rooted slots.

**`rexx-core` may need amending after all.** Twelve non-test sites resolve the frame implicitly from the top activation (`self.activation().frame`), seven of them in `stem.rs`. Whether the redirect lives in `Interp` (returning a `(SlotFrame, usize)` pair, touching all twelve, adding a check to a path that is 8.1%/32.2% of runtime) or in `RootSet` (amending `rexx-core`) is **Task 5's decision, made with a measurement**. The first revision asserted `rexx-core` is amended in no task and listed `stem.rs` as "comments only"; both were wrong.

**The indirect form is plural, and it also exposes its own selector.** Measured: with `list = 'ALPHA BETA'`, `sub: procedure expose (list)` exposes `ALPHA` *and* `BETA`; and with `v = 'zzz'`, `procedure expose (v)` exposes **`v` itself as well as** `ZZZ`. The value is a blank-delimited list of names, not one name. `run.rs`'s `DROP (v)` arm took this exact correction in 4a -- read its doc comment before implementing.

**What the five stem transcripts still establish.** `EXPOSE` aliases the caller's *variable entry*, not the stem *object*. With the caller holding `a.1 = 'kept'` and calling `sub: procedure expose a.`:

| Callee body | Caller prints |
|---|---|
| `a.1 = 'changed'` | `changed` |
| `a. = 'wiped'` | `wiped` |
| `drop a.` | `A.1` |

and with `b. = a.` added in the caller beforehand, `drop a.` in the callee leaves the caller printing `A.1 orig` -- the drop rebinds the entry, and `b.` still holds the old object. So `stem_drop`'s existing shape (`replace_stem(name, None)`) is correct under exposure and must not change.

**The two indirect-form hypotheses remain undistinguished.** "The name list is read from the caller before isolation" and "`PROCEDURE` executes in the callee before the pool is isolated" both fit every probe, because `PROCEDURE` must be a routine's first instruction so the callee can never have its own selector variable. They agree operationally. Implement either; do not report that a measurement chose one.

### D2r -- the error report carries a stack of sites, and the indent base is the caller's *printed* indent

**This replaces the first revision's D2 indent rule, which was refuted.** That version said each activation adds two spaces on top of `static_indent`. It was measured only at caller lexical indent 0, where the wrong rule and the right one coincide -- the fourth time in this project an indentation rule has been got wrong by probing a single shape.

Measured 2026-08-03. A call nested inside one `DO`, callee flat:

```
     6 *-*     say 2 & 1        <- indent 4
     2 *-*   call sub           <- indent 2
```

A call nested two `DO`s deep inside a called routine:

```
    11 *-*         say 2 & 1    <- indent 8
     6 *-*       call inner     <- indent 6
     1 *-* call outer           <- indent 0
```

The rule that fits every probe:

```
indent(frame) = indent of the calling clause AS PRINTED
              + delta
              + static_indent(the clause in this frame)

delta = 2 for a CALL or function-call activation
delta = 0 for an INTERPRET fragment
```

The oracle keeps one running counter that blocks and activations both push onto, so a callee inherits the caller's *whole* current indent, lexical part included. The base **cannot be computed from the depth counter** and must be carried on the site stack, taken from the calling clause's own printed indent. The architectural half survives: the base is still added outside `static_indent`, and `static_indent` stays a pure function of the flat instruction list.

**The clause echo saturates at 40 columns, and this is a live 4a defect.** Measured on the oracle with nested `DO`s and no calls at all: depth 18 gives 36, depth 19 gives 38, depth 20 gives 40, and depths 21 and 25 give 40. Our binary is uncapped -- at 25 nested `DO`s the oracle prints the echo at 40 and `rexx-run` prints it at 50, verified by `diff` with the echo as the only differing line. **This ships today**; 4a's gate passed because none of the 29 corpus programs nests past 20.

**The cap applies to `*-*` clause echoes only.** Measured at nesting depth 25 under `trace r`: `*-*` indents top out at 40 while `>>>` value lines run to 52. Any implementation that clamps inside `static_indent`, or that treats the clause echo and the value lines as one rule, is wrong. The clamp goes at the `*-*` formatting site.

The two properties, per activation line number and the additive base, are also what the KNOWN GAP row at `phase-4-exclusions.txt` must record, because the row currently says only "one echo per nesting level".

### D10 -- D19's per-activation Rust recursion is confirmed

One Rust frame per activation plus an explicit counter, raising 11.1 at rc 245. `Raised::insufficient_stack()` **already exists** at `src/error.rs:109` -- do not write a second raiser. Reopening this reopens the `Rc<Program>` risk D19 closed: the flat-loop variant is where the program local must be re-derived at every frame transition. Recorded explicitly because it is the decision most likely to be re-litigated by an implementer who has not read D19.

### D3, D5, D6, D13, D14 -- unchanged from the first revision

* **D3.** The `DO OVER`-on-a-stem deviation stands; **no corpus program may contain `DO OVER` on a stem**, in 4b's subset as in 4a's. The oracle walks a balanced tree and we use a hash map; measured, tails 1, 2, 3, 10, ZZ, B yield `1 B 3 2 ZZ 10`. `PROCEDURE EXPOSE` changes stem identity, not iteration order, so it does not touch the deviation's premise.
* **D5.** 4b runs strictly before 4c, in one lane. 4c depends on 4b in six named places; 4b depends on 4c nowhere. Both sub-phases' code lands in `eval.rs`, `run.rs` and `lib.rs`, and every scheduling collision in the 4a ledger was two agents in one file. **Never dispatch two implementers in parallel against this plan.**
* **D6.** `rust/corpus/phase-4b.txt` is created beside `phase-4a.txt`; the harnesses read the union. Growing `phase-4a.txt` would destroy the ability to say which sub-phase a regression belongs to.
* **D13.** 4b gets its own gate document.
* **D14.** The 4a criterion set carries forward with two amendments: criterion 2's wording must name Phase 5 (all 35 exempt assertion rows are `unblocked_by: "Phase 5"`, verified 35 of 35), and criterion 3 gains a coverage measure of its own.

---

## Corrections to inherited items, and to this plan's own first revision

### `>I>` and `<I<` are real, and they belong to `::routine` and `::method` under `TRACE LABELS`

The first revision claimed they are "most likely method invocation, Phase 5's", on the strength of a `trace i` probe. **That repeated the original error's shape**: concluding an owner from an instrument that could never have produced the answer. Measured 2026-08-03:

```rexx
call myrtn 1
say result
exit
::routine myrtn
trace l
use arg a
return a + 1
```

```
       >I> Routine "MYRTN" in package "/path/i1.rex".
       <I< Routine "MYRTN" in package "/path/i1.rex".
```

The C++ gate is `tracingLabels() && isMethodOrRoutine()`. An **internal label** call with `trace l` as its first clause emits nothing, with or without `PROCEDURE`; `trace l` in the *caller* targeting a `::routine` emits nothing, because the caller's trace setting does not reach a `::routine` activation. So whether these are 4b's depends entirely on whether 4b calls a `::routine`, which Task 3 leaves open. The exclusions row must say "`::routine`/`::method` under TRACE LABELS", not "method invocation".

4b's other prefixes, measured: `>A>` (ARGUMENT) in both call forms; `>F>` (FUNCTION) in the expression form only; `>R>` (ALIAS) in the callee for `USE ARG >`, and it is a **RESULTS**-level prefix, not intermediates-only -- at `trace i` the call site shows `>O>   ">" => "PP"` and `>A>   "orig"` (the *value*, not the name), then the callee shows `>R>     "PP" => "Q"`.

### I17 is reclassified: the `stem_drop` mutant is genuinely equivalent

The scoping document says the mutant becomes pinnable when 4b lands, because nothing in 4a can hold a second reference to a stem object. Measured, `b. = a.` shares the object in 4a on both interpreters (`new new`). So the premise is false. But the conclusion fails too: `a.1 = 'orig'; b. = a.; drop a.; say a.1 b.1` prints `A.1 orig` under both the current code and the mutant, because "slot holds a fresh empty stem" and "slot is unset" are not distinguishable through a second reference. **Record it as genuinely equivalent, with that mechanism written down**, rather than carrying it as an expected survivor with a false explanation. Task 5 owns the wording; it is not asked to find a distinguishing program, because none exists.

### `RESULT` is dropped on return, not at the call

Measured: a caller sets `result = 'before'`, calls a routine with no `PROCEDURE`, and the callee prints `inside result= before`. So the drop happens when the routine returns without a value, not when the call begins. `call sub` where `sub: return 42` sets `RESULT` to `42`; after a bare `return`, `RESULT` reads as the derived name `RESULT`. **The expression form does not touch `RESULT` at all.**

### The corpus files contradict the ruling about `ExprKind::List`

`corpus/phase-4a.txt:18` and `corpus/README.md:108-109` say three dropped `num/` programs return "for 4b or 4c, once `List` exists"; `phase-4-exclusions.txt:176-179` assigns `List` to Phase 5. The ruling is right -- measured, `(1, 2)` is an `Array` instance. Task 10 corrects both comments. Pulled forward because a 4b author reads the corpus files first.

---

## File structure

```
rust/crates/rexx-exec/
  src/lib.rs          Interp; the spike field and its plumbing are deleted;
                      trace_mode leaves here for Activation
  src/activation.rs   body selector, settings-inheriting constructor,
                      trace_mode, the exposure alias bitset
  src/plan.rs         BodyKey::directive's first setter; each body's
                      PROCEDURE/EXPOSE list, precomputed
  src/error.rs        Raised gains a site stack; Raised::condition gains a
                      reader; insufficient_stack() already exists
  src/run.rs          CALL, RETURN, PROCEDURE, USE, SIGNAL, RAISE, INTERPRET,
                      PUSH, QUEUE, the trap table, the depth counter,
                      the 40-column clause-echo clamp
  src/eval.rs         ExprKind::Call's internal-routine front,
                      ExprKind::VariableReference
  src/queue.rs        NEW: the in-process external data queue
  src/trace.rs        >A>, >F>, >R>, and the activation indent base
  tests/owners.rs     NEW (Task 0): the owner table coverage.rs and loud.rs share
rust/corpus/phase-4b.txt   the named 4b subset
rust/corpus/proc/*.rex     new 4b programs
rust/scripts/mutate-4b.sh  4b's mutation set, reusing 4a's guard mechanism
docs/superpowers/plans/phase-4b-gate.md
```

---

### Task 0: The shared owner table, the subset union, and an owner in every loud message

**This task runs first.** Every later task edits the owner tables, and three later tasks assert a property the tree cannot currently express.

**Files:**
- Create: `rust/crates/rexx-exec/tests/owners.rs`
- Modify: `rust/crates/rexx-exec/tests/coverage.rs`, `tests/loud.rs`
- Modify: `rust/crates/rexx-exec/src/lib.rs` (the two loud-message sites, `:444` and `:468`)
- Modify: `rust/crates/rexx-exec/tests/corpus.rs`, `tests/coverage.rs`, `tests/collect_stress.rs` (the `read_subset` call sites)

**Why:** three separate mechanisms in the tree defeat the later tasks as written.

**Inherited item I36.** `coverage.rs` and `loud.rs` duplicate the owner table **by hand**, because an integration test cannot `mod` another test binary's directory and no shared module was in scope in 4a. 4b and 4c edit both on every variant they deliver, and a divergence between them is caught by nothing.

**The three mechanisms, all verified in the tree:**

1. **`loud.rs`'s witness set is variant-grained and asserted complete.** `assert_witness_set_is_complete` requires exactly one witness per out-of-scope `InstructionKind` variant, "no more and no fewer". So the moment `InstructionKind::Call` moves in scope, the `call sub` witness must be **deleted** -- and `Call::Qualified` (Phase 5's) and `Call::Trap` (Task 7's) are left with no loudness witness anywhere. Same shape for `Signal::Trap` between Tasks 6 and 7. The first revision caught the `coverage.rs` half of this and missed the `loud.rs` half.
2. **No loud message names a phase.** `every_out_of_scope_variant_fails_loudly` compares `exit_code` and nothing else; `Witness::owner` is only printed on failure. Measured, the emitted text is `rexx-exec: CALL is not implemented`. So "must keep failing loudly with owner `Phase 5`" is a property nothing can express, and any test asserting `stderr.contains("4c")` fails today.
3. **`EXPECTED_OUT_OF_SCOPE` (`coverage.rs:606`) and four hardcoded counts** are pinned literals that every variant-flipping task breaks. They are named in no later task.

- [ ] **Step 1: Create `tests/owners.rs` as the single owner table, and have both harnesses read it**

Use `#[path]` or a shared file both `mod`-include. Then add a test asserting the two harnesses see the *same* table, not two lists that happen to agree.

- [ ] **Step 2: Make the witness table arm-grained for the split variants**

`Call::Named` in scope from Task 3; `Call::Dynamic` in scope from Task 3; `Call::Qualified`, `Call::Trap` and `Signal::Trap` each their own witness row with its own owner. A variant whose inner forms have different owners needs one row per form, or implementing the outer variant silently drops coverage of the rest.

- [ ] **Step 3: Give the loud message an owner, and assert it**

Add an `owner: &'static str` to the loud payload sourced from the shared table, so the message reads `rexx-exec: CALL is not implemented (Phase 5)` or similar -- take the exact format from what reads best beside the existing text, and record it here so later tasks assert the same shape. Then make `every_out_of_scope_variant_fails_loudly` assert the emitted stderr contains the witness's owner.

- [ ] **Step 4: Change the `read_subset` call sites to take a list of subset files**

`&[&Path]`, union semantics. There are **two** call sites in `coverage.rs`, not one, plus `corpus.rs` and `collect_stress.rs`. Keep each harness's own copy of the reader -- factoring those together is a separate change.

- [ ] **Step 5: Document `EXPECTED_OUT_OF_SCOPE` and the four hardcoded counts in `tests/owners.rs`'s module doc**

Name each one and say that any task moving a variant in scope must update it. This is the only place a later implementer will look.

- [ ] **Step 6: Run the full suite, commit**

Expected: no behaviour change, all tests green. This task ships zero interpreter functionality by design.

---

### Task 1: A real `INTERPRET`, replacing the Task 3 spike

**Files:**
- Modify: `rust/crates/rexx-exec/src/lib.rs` -- delete `run_program_interpret_spike` (**`:1114`**), the `interpret_spike` field (**`:784`**), its constructor parameter (**`:830`**), and `execute`'s parameter (**`:1169`**); add a `#[cfg(test)] mod tests` for the fragment-lifetime proofs
- Modify: `rust/crates/rexx-exec/src/run.rs` (the `InstructionKind::Interpret` arm)
- Modify: `rust/crates/rexx-exec/tests/spike.rs`
- Modify: `rust/crates/rexx-exec/tests/owners.rs`

**Interfaces:**
- Consumes: `Fragment { source, body: CodeBody, symbols }`; `Interp::alloc_with`; `step_in_temps_frame`; Task 0's shared owner table.
- Produces: nothing public. No new public item.

**Why:** D1. The spike's public entry point exists only because a private subject needed an integration test, and its own doc records that a `#[cfg(test)] mod tests` inside `lib.rs` could prove the same lifetime with no public surface. That trade is remade here.

**Check the tree before writing anything.** A private fragment-running helper may already do most of what Step 6 describes. Find it and reuse it rather than writing a second one.

**Inherited items this task pays for:**

* **I7.** Delete the spike surface; move `tests/spike.rs`'s three fragment tests into `lib.rs`.
* **I8.** Fragment plans are built, used and dropped, and **stay that way**. Revision 6's `(enclosing body, fragment id)` cache key was withdrawn as "sound and useless": fragment text varies per execution, so every lookup misses while every entry is retained. A cache keyed by fragment *text* is permitted only if this task measures a hit rate on a real program and reports it. Absent that measurement, do not add one.
* **I9.** `Fragment::body.labels` is always empty. Measured and settled in 4a's Task 1: a label inside `INTERPRET` text is error 47.1 both ways. Do not add label handling.
* **I16.** `step_in_temps_frame` is the single chokepoint healing six `push_frame` sites in `eval.rs` that skip their `pop_frame` on the `?` path, and it is the **only** caller of `step` in the crate. 4a's conclusion that `SIGNAL ON SYNTAX` cannot accumulate temps leaks rests entirely on that. **If this task moves execution off that chokepoint, say so in the report** -- Task 7 depends on the analysis.
* **I21.** Allocations go through `Interp::alloc_with`.
* **I22.** `pop_frame`'s truncation semantics are load-bearing. **No assert may be added there** without balancing the six `eval.rs` sites first. A debug tripwire in `step_in_temps_frame` asserting temps balance on the `Ok` path was scheduled in 4a and not built; it needs a `temps_len()` accessor. Building it is welcome; changing `pop_frame` is not.

- [ ] **Step 1: Move the three fragment-lifetime tests into `lib.rs`**

Reproduce each as a `#[test]` in a `#[cfg(test)] mod tests`, driving `Interp` directly. Keep each test's doc comment verbatim: they record why the lifetime is what it is.

- [ ] **Step 2: Change the loud fixtures in `spike.rs` before they break**

At least two tests there use constructs 4b implements and assert `NOT_IMPLEMENTED_EXIT`. One uses `call "sub"` and asserts stderr contains `"CALL"`; find the others by running the suite after Step 6 and reading what fails. This is the **fourth** occurrence in this project of a witness implemented out from under a test.

Replace each fixture with a **message send** (`q~append(1)`), which the spec assigns to Phase 5 outright rather than by ruling. Give the exact expected stderr text in the test, taken from a run. Add a one-line comment saying why a message send and not `PARSE` or `ADDRESS`: those are 4c's and would break again in weeks.

- [ ] **Step 3: Run the suite to see the moved tests pass before anything else changes**

- [ ] **Step 4: Write the failing `INTERPRET` test**

```rust
#[test]
fn interpret_binds_a_name_the_enclosing_body_never_mentions() {
    // Measured on the oracle in 4a: the binding outlives the fragment.
    let out = run_source(b"interpret \"zork = 42\"\ninterpret \"say zork\"\n");
    assert_eq!(out.stdout, b"42\n");
    assert_eq!(out.exit, 0);
}
```

`run_source` is shorthand: use whatever helper the surrounding tests already use, or drive `Interp` directly.

- [ ] **Step 5: Run it and watch it fail**

- [ ] **Step 6: Implement the `InstructionKind::Interpret` arm**

Evaluate the expression to a string, parse it as a `Fragment`, give it a plan, execute it against the **current** activation: same frame, same slots, same settings. A name the enclosing plan never saw goes through `Plan::slot_of` into `Activation::extra`.

The fragment's instruction list is separate, so `pc` cannot continue into it. Execute through the same bounded sub-loop shape `run_bounded` uses, and forward an unowned `Flow` outward.

**Two escapes must be measured against the oracle before you choose their semantics, and the first revision of this plan got one wrong by asserting it:**

* `LEAVE` naming a loop that encloses the `INTERPRET` instruction, from inside the fragment.
* `RETURN` inside a fragment, both in the main body and inside a called routine.

Measure both, record the transcripts in the report, and implement what the oracle does. If either needs machinery Task 3 has not built, leave it failing loudly and say so -- do not invent a plausible answer.

- [ ] **Step 7: Run the suite**

- [ ] **Step 8: Move `Interpret` in scope in `tests/owners.rs`, and update `EXPECTED_OUT_OF_SCOPE` and the four counts Task 0's module doc names**

The in-scope tag requires a witness program in the subset, or `every_in_scope_variant_is_witnessed_by_the_phase_4a_subset` fails.

- [ ] **Step 9: Commit**

---

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

- [ ] **Step 5: Clamp the `*-*` echo at 40 columns, and only the `*-*` echo**

- [ ] **Step 6: Run the full suite and the corpus gate**

Run: `cargo test -p rexx-exec`, then `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus`

- [ ] **Step 7: Add the deep-nesting program to the 4a corpus subset**

It has no 4b construct in it, so it belongs in `phase-4a.txt` -- it pins a rule 4a should always have had.

- [ ] **Step 8: Amend the KNOWN GAP row, add a DEVIATIONS or fixed-defect note for the cap, commit**

---

### Task 3: The body selector, `CALL`, `RETURN`, and the shared variable pool

**Spec:** design spec's "The borrow shape" and D19.

**Files:**
- Modify: `rust/crates/rexx-exec/src/activation.rs` (body selector; a sibling constructor to `Activation::new`; `trace_mode`)
- Modify: `rust/crates/rexx-exec/src/plan.rs` (`BodyKey::directive`)
- Modify: `rust/crates/rexx-exec/src/lib.rs` (delete `Interp::trace_mode`, field at **`:640`**, doc at **`:625-639`**)
- Modify: `rust/crates/rexx-exec/src/run.rs` (`run_activation`'s hardcoded `&program.main`; the `Call` and `Return` arms; the depth counter; `Flow`)
- Modify: `rust/crates/rexx-exec/src/error.rs` (push an activation site)
- Modify: `rust/crates/rexx-exec/tests/owners.rs`

**Interfaces:**
- Produces: a body selector field on `Activation` whose `None` is the main body and whose `Some(i)` is `directives[i]`'s -- the same shape `BodyKey::directive` carries. A sibling constructor to `Activation::new` that inherits `settings` and `trace_mode` from the caller. `Interp::depth: usize`. A `Flow` variant for `RETURN`.
- Consumes: Task 2's site stack; `Raised::insufficient_stack()` (`src/error.rs:109`), which **already exists**.

**Why:** `run_activation` hardcodes `&program.main`. True for every activation 4a can build, false the moment a callee runs, and the failure is silent and with the right program.

**Read D9r above in full.** The default is a *shared* pool: a callee with no `PROCEDURE` reads and writes the caller's variables and its writes survive the return. **This task implements that default**, and Task 5 adds isolation. A callee without `PROCEDURE` gets **no new slot frame** -- reuse the caller's `SlotFrame`, save and restore `pc`, do not `pop_slots` on return.

**Your witness must have variables in it.** The first revision's witnesses were `sub: say 'callee'` and `f: return 41` -- neither has a variable, so both pass against an implementation that wrongly isolates every callee. That is 4a's own pathology: `Stem` witnessed, arithmetic witnessed, `Stem`-as-arithmetic-operand not, and a process abort shipped.

**Inherited items this task pays for:**

* **I1.** The body selector goes beside `Activation::program`. That field's doc describes the gap; update it rather than leaving a stale note.
* **I2.** `BodyKey::directive` is `Some(index)`-shaped and nothing has ever set it. **Decide it together with I1** or the activation's selector and the plan cache's key will denote different things.
* **I3.** `Activation::new` unconditionally defaults `Settings`. Measured: with `numeric digits 7`, an internal `call sub` sees 7, sets its own to 3, and after `return` the caller still reports 7. Keep two constructors: a fresh top-level run and a nested call begin from different starting settings.
* **I4.** `Interp::trace_mode` moves onto `Activation`. A deliberate 4a-only simplification: 4a has one frame, and measured, a callee's `trace off` does not survive its `return`. The field's doc names this as 4b's first move.
* **I6 and D10.** One Rust frame per activation plus an explicit counter. Measured: unbounded `CALL` recursion gives `Error 11.1`, "Insufficient control stack space", rc 245 -- a reportable condition, not a crash. Do not reopen D19.
* **I34.** The counter protects a sized caller only. On a default 2 MiB thread the native abort arrives at **331 parens or 341 calls**, long before any counter at 50,000 fires. The long-term answer is a documented minimum stack or a sized entry point in `rexx-parse`. This task measures 4b's contribution; it does not fix it.
* **I12's `CALL` half.** Each activation pushes a site entry carrying its own line and the indent base D2r defines.

**Measured semantics this task must reproduce:**

* **`RESULT` is dropped on return, not at the call.** A caller sets `result = 'before'`, calls a no-`PROCEDURE` routine, and the callee prints `inside result= before`. After `sub: return 42`, `RESULT` is `42`; after a bare `return`, `RESULT` reads as the derived name `RESULT`.
* **Omitted arguments.** `call sub 1,,3` with three `USE ARG` targets gives `[1] [Q] [3]` -- the omitted position leaves its target unset, and `arg()` still returns 3.
* **`CALL "SUB"` with `sub:` present is Error 43.1, rc 213**, `Could not find routine "SUB"`. The `literal` flag bypasses the internal label search, so 4b's correct answer is the loud 4c/Phase 7 fallback -- do not wire the literal form into the label table for symmetry.

- [ ] **Step 1: Measure the combined depth budget before writing the counter**

D19 chose per-activation Rust recursion; `run_bounded` already costs a Rust frame per source nesting level; Phase 5's dispatch will add a third. Measure both directions on our binary -- recursion depth to abort with no nesting, and with each activation containing a nested `DO` -- and compare against the oracle's 11.1 depth. **If our native abort arrives before the counter fires, the counter is decoration and the task must say so** rather than ship it silently.

- [ ] **Step 2: Write the failing tests, both with variables**

```rust
#[test]
fn a_routine_without_procedure_shares_the_callers_pool() {
    let out = run_source(
        b"v = 'caller-v'\ncall sub\nsay 'caller sees:' v w\nexit\n\
          sub:\nsay 'callee sees v:' v\nw = 'callee-w'\nreturn\n",
    );
    assert_eq!(out.stdout, b"callee sees v: caller-v\ncaller sees: caller-v callee-w\n");
}

#[test]
fn a_called_label_runs_its_own_clauses_not_the_main_body() {
    let out = run_source(b"call sub\nsay 'main'\nexit\nsub: say 'callee'\nreturn\n");
    assert_eq!(out.stdout, b"callee\nmain\n");
}
```

- [ ] **Step 3: Run them and watch them fail loudly**

- [ ] **Step 4: Add the body selector and the settings-inheriting constructor**

An internal `CALL` targets a **label in the same body**, so it does not exercise `BodyKey::directive`. Add a `::routine` test if one is reachable; **if it is not reachable in 4b, say so in the report** and leave `Some(index)` unset with its doc naming the phase that sets it. Task 9's `>I>`/`<I<` scope decision depends on this answer.

- [ ] **Step 5: Move `trace_mode` onto `Activation`**

The callee inherits the caller's value at call time and does not write back on return.

- [ ] **Step 6: Implement `Call::Named` and `Call::Dynamic`, and `Return`**

`Call` has four forms:

* `Named { name, literal, args }` -- this task's, including the `literal` fallback above.
* `Dynamic { target, args }` -- **this task's.** The first revision named it as 4b's and gave it no step. The target is an expression evaluated at run time, then resolved as a name.
* `Trap(ConditionTrap)` -- **Task 7's.** Keeps failing loudly until then.
* `Qualified { namespace, name, args }` -- **Phase 5's.** Must keep failing loudly with owner `Phase 5`.

Task 0 gave the witness table one row per form, so the last two are still covered after this task. Do not delete their witnesses.

Resolution order for a named call is internal label, then builtin, then external. 4b builds the front; the fallback fails loudly naming `4c`.

`RETURN` needs its own `Flow` variant -- the existing variants do not express "unwind to the activation boundary". Say what happens to a `RETURN` in the **main body** with no active call: measure it, because `loud.rs` may use that shape as its own witness.

- [ ] **Step 7: Implement the depth counter, calling the existing `Raised::insufficient_stack()`**

- [ ] **Step 8: Push an activation site entry using D2r's rule, and verify the three-deep transcript**

Capture it fresh from the oracle. **Include a probe where the caller is lexically nested**, or the expectation cannot distinguish D2r's rule from the wrong one the first revision shipped.

- [ ] **Step 9: Implement `RESULT`, dropped on return**

- [ ] **Step 10: Run the full suite and both gates; 4a's 29 corpus programs must still match byte-for-byte**

If any moved, the site stack or the indent base is wrong. Do not adjust the expectation.

- [ ] **Step 11: Update `tests/owners.rs` and the pinned literals, close the `CALL` half of the KNOWN GAP row, commit**

---

### Task 4: `ExprKind::Call` -- the internal-function form

**Files:**
- Modify: `rust/crates/rexx-exec/src/eval.rs` (a new `ExprKind::Call` arm)
- Modify: `rust/crates/rexx-exec/tests/owners.rs`

**Interfaces:**
- Consumes: Task 3's activation machinery, unchanged.
- Produces: the `ExprKind::Call` arm, whose fallback is where 4c hangs the builtin table.

**Why:** I25. `ExprKind::Call`'s owner string is `4b`, and the owner named is the phase after which the variant stops failing loudly *for some target*. Whichever sub-phase runs first builds the arm; 4b runs first.

**Check `ExprKind::Call`'s own target field before writing the arm.** If it carries a target enum with forms owned by different phases, the same one-row-per-form treatment Task 0 gave `InstructionKind::Call` applies here, and the arm must fail loudly per form.

**I25's split goes into `eval.rs`'s arm as a comment**, not only into `phase-4-exclusions.txt`. It is currently in the exclusions file and two test-file comments, and a 4c implementer reading `eval.rs` sees none of them. The comment says: internal routine first (4b), builtin second (4c), external third (Phase 7), and that a name reaching the fallback fails loudly naming `4c`.

**Measured:** the expression form does **not** touch `RESULT`. A function call at `trace i` emits `>F>   F => "2"` at the caller's indent before the enclosing expression's own `>>>`; `call sub 1` emits no `>F>`. Both emit `>A>` per argument at the call site. Task 9 implements the prefixes; this task must not emit them.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn an_internal_function_returns_its_value_into_an_expression() {
    let out = run_source(b"say f(1) + 1\nexit\nf: return 41\n");
    assert_eq!(out.stdout, b"42\n");
}
```

- [ ] **Step 2: Run it and watch it fail loudly**

- [ ] **Step 3: Implement the arm**

A routine that returns no value is an error in the expression form. **Measure the oracle's error number and text**; do not guess.

- [ ] **Step 4: Verify the fallback still fails loudly for a builtin name**

```rust
#[test]
fn a_builtin_name_still_fails_loudly_naming_4c() {
    let out = run_source(b"say length('abc')\n");
    assert_eq!(out.exit, NOT_IMPLEMENTED_EXIT);
    assert!(String::from_utf8_lossy(&out.stderr).contains("4c"));
}
```

This assertion depends on Task 0 having put the owner into the message. If Task 0's format differs from `4c`, use Task 0's format.

- [ ] **Step 5: Run the suite, update `tests/owners.rs` and the pinned literals, commit**

---

### Task 5: `PROCEDURE`, `PROCEDURE EXPOSE`, `USE ARG` and `USE LOCAL`

**Files:**
- Modify: `rust/crates/rexx-exec/src/plan.rs` (each body's `PROCEDURE`/`EXPOSE` list, precomputed)
- Modify: `rust/crates/rexx-exec/src/run.rs` (the `Procedure` and `Use` arms; the isolation decision inside `CALL`)
- Modify: `rust/crates/rexx-exec/src/activation.rs` (the alias bitset and its target frame)
- Modify: `rust/crates/rexx-exec/src/eval.rs` (`ExprKind::VariableReference`)
- Modify: `rust/crates/rexx-exec/src/stem.rs` -- **not comments only.** Seven of the twelve sites that resolve a frame from the top activation are here.
- Possibly modify: `rust/crates/rexx-core/src/roots.rs` -- see below.
- Modify: `rust/crates/rexx-exec/tests/owners.rs`

**Interfaces:**
- Consumes: Task 3's shared-pool default; `Plan::slot_of` (`src/plan.rs:548`), which is idempotent.
- Produces: isolation plus the exposure redirect.

**Why:** D9r. **Read it in full before starting** -- the first revision of this plan specified a mechanism that does not exist, and this task's file list is the corrected one.

**The design, restated so it cannot be got wrong:**

* Caller and callee share one `CodeBody`, one `Plan`, and one name-to-slot map, so **slot indices are identical between frames**. A `PROCEDURE` callee gets a fresh frame of the same size; an exposed name at index *i* aliases index *i* in the target frame. The redirect is a bitset over slot indices plus one target `SlotFrame`, not a name-keyed map.
* **Exposure is transitive.** Measured: `a` exposes `n` to `b`, `b` exposes the same `n` to `c`, `c` writes it, and `a` sees `set-by-c`. Binding `c` must **chase `b`'s alias** to `a`'s frame. Binding to `b`'s frame gives a silently wrong value two levels up.
* **`expose (list)` is plural, and exposes its own selector.** Measured: with `list = 'ALPHA BETA'`, `procedure expose (list)` exposes `ALPHA` and `BETA`; with `v = 'zzz'`, `procedure expose (v)` exposes **`v` itself as well as** `ZZZ`. The value is a blank-delimited list of names. `run.rs`'s `DROP (v)` arm took this exact correction in 4a -- read its doc comment first.
* **Where the redirect lives is this task's decision, made with a measurement.** Twelve non-test sites resolve the frame from the top activation, seven in `stem.rs`. Either `Interp`'s slot resolution returns a `(SlotFrame, usize)` pair and all twelve change -- adding a check to a path that is 8.1%/32.2% of runtime -- or the indirection goes into `RootSet`, which amends `rexx-core`. **Measure the hot-path cost before choosing**, and report both the choice and the number.
* `RootSet::grow_slots` keeps its top-frame-only invariant under either choice. Its panic message pins the string `4a invariant` and `crates/rexx-core/tests/collect.rs` pins the wording with `#[should_panic(expected = "4a invariant")]`. **If this task finds it must relax that panic, stop and report BLOCKED** -- that is a plan change.
* Two `rexx-core` comments predict a relaxation this design does not perform. Correcting them is in scope for this task; leaving them is not.

**Measured `USE ARG` semantics:**

* `call sub 1,2,3` with `use arg p` succeeds and binds `p = 1`. Extra arguments are ignored.
* `use strict arg p` with three arguments is Error 40.4, rc 216: `Too many arguments in invocation of SUB2; maximum expected is 1.`
* `call sub 1,,3` with `use arg p, q, r` gives `[1] [Q] [3]`.
* `use arg >q` requires the caller to pass a variable reference. `call sub p` with a plain symbol is Error 88.928, rc 168: `The 1 argument must be a VariableReference instance; found "caller".` `call sub >p` works: the callee's `q = 'aliased'` makes the caller's `p` read `aliased`.

That last pair is why `ExprKind::VariableReference` is in this task: it is the argument-side half of `USE ARG >`, and neither is testable without the other.

**Inherited items this task pays for:**

* **I5.** `RootSet::grow_slots` panics on a non-top frame. Under this design the invariant stays true. Say so in the `PROCEDURE` arm's comment, with the reason, so the next reader does not think the panic was overlooked.
* **I18.** `RootSet::clear_slot` exists so the read path can tell "unset" from every other value, for `NOVALUE`. `stem_drop` deliberately does not use it, and the doc at **`src/stem.rs:361`** explains why a stem's slot is not "empty or not" the way a simple variable's is. Task 7 needs both halves; do not collapse them.
* **I17, reclassified.** The `stem_drop`-to-slot-clear mutant is **genuinely equivalent**, and its old explanation is false in both directions. `b. = a.` shares the object in 4a (measured `new new` on both interpreters), so the "nothing in 4a can hold a second reference" premise is wrong; and `a.1='orig'; b. = a.; drop a.; say a.1 b.1` prints `A.1 orig` under both the current code and the mutant, so the distinction is not observable through that reference either. Write that mechanism into `mutate-4b.sh`'s comment. **You are not asked to find a distinguishing program; none exists.**
* **D3, restated because this task writes stem programs.** No corpus program may contain `DO OVER` on a stem.

- [ ] **Step 1: Write the failing isolation test with discriminating values**

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

- [ ] **Step 2: Precompute each body's `PROCEDURE`/`EXPOSE` list in `plan.rs`**

`PROCEDURE` must be a routine's first instruction, so the list is a property of the body. The indirect form's names are not known until run time; carry what is needed to resolve them at bind time.

- [ ] **Step 3: Implement isolation and the exposure redirect, with transitivity**

- [ ] **Step 4: Write the transitivity and plural-selector tests**

```rexx
n = 'from-a'
call b
say 'a sees:' n
exit
b: procedure expose n
call c
say 'b sees after c:' n
return
c: procedure expose n
n = 'set-by-c'
return
```

Expected, measured: `b sees after c: set-by-c` then `a sees: set-by-c`.

- [ ] **Step 5: Implement `Use::Arg` and `Use::Local`**

`strict` and `allow_optionals` (the trailing `...`) both change the arity check. `UseTarget::default` is `USE ARG a = 1`; `UseTarget::alias` is `>a`. Take every error number and text from the oracle.

- [ ] **Step 6: Implement `ExprKind::VariableReference`**

- [ ] **Step 7: Write the five stem-exposure tests**

The `drop` pair is the one that pins the design.

- [ ] **Step 8: Record I17's reclassification, run the suite and the corpus gate, update `tests/owners.rs`, commit**

---

### Task 6: `SIGNAL` to a label, and `SIGNAL VALUE`

**Files:**
- Modify: `rust/crates/rexx-exec/src/run.rs` (the `Signal` arm, and `Flow` if a variant is needed)
- Modify: `rust/crates/rexx-exec/tests/owners.rs`

**Why:** `SIGNAL` needs only the label table `CodeBody` already carries. It is placed here rather than first because it unblocks nothing.

`Signal` has three forms. `Label(Box<[u8]>)` and `Value(Expr)` are this task's; `Trap(ConditionTrap)` is **Task 7's** and keeps failing loudly, with its own witness row from Task 0. **The witness program for the in-scope tag must use `SIGNAL label`, not `SIGNAL ON`.**

**Do not assume `Flow`'s existing variants suffice.** `SIGNAL` out of a `DO`, a `SELECT` or an `INTERPRET` fragment must unwind the block stack, and `SIGNAL` from inside a routine has to be measured before its semantics are chosen. `pop_search_frame` and the `Flow` forwarding do a similar shape for `LEAVE`; reuse where it fits, and add a variant where it does not.

- [ ] **Step 1: Measure, and put the transcripts in the report**

At minimum: `SIGNAL` out of a nested block; `SIGNAL` to a label not in the current body; `SIGNAL` from inside a called routine to a label in the caller's body; `SIGNAL` out of an `INTERPRET` fragment; `SIGNAL VALUE` where the value is not a label.

- [ ] **Step 2: Write the failing tests from those transcripts, asserting exact stdout, stderr and rc**

- [ ] **Step 3: Implement `Signal::Label` and `Signal::Value`**

- [ ] **Step 4: Run the suite, update `tests/owners.rs`, commit**

---

### Task 7: Condition traps, `RAISE`, and `NOVALUE`

**Files:**
- Modify: `rust/crates/rexx-exec/src/error.rs` (`Raised::condition` gains its first reader; delete its `#[expect(dead_code)]`)
- Modify: `rust/crates/rexx-exec/src/run.rs` (the trap table; `SIGNAL ON`/`CALL ON`; the `Raise` arm; `failure_site` clearing)
- Modify: `rust/crates/rexx-exec/src/lib.rs` (`Novalue::Unset`, enum at **`:591`**, produced at **`:914`**)
- Modify: `rust/crates/rexx-exec/tests/owners.rs`

**Interfaces:**
- Consumes: `Raised`, `FailureSite`, `record_failure_site` (**`src/run.rs:1387`**), `Raised::report`, the catalogue and the 256-major exit rule -- all built and byte-verified in 4a.
- Produces: a per-activation trap table; `RAISE` reusing the existing raiser families.

**Why:** `RAISE` is the cheapest instruction in 4b's list because the raiser families already exist. The expensive half is resumption: a trap that transfers control after a raise is the first caller that makes `failure_site`'s never-cleared state matter.

**Inherited items this task pays for:**

* **I10.** `Raised::condition` has no reader. It is a field rather than a hardcoded value because `NOVALUE`, `NOMETHOD` and friends need to set it to something else, and it is `#[expect(dead_code)]` rather than `#[allow]` **on purpose, so the day 4b reads it the annotation asks to be deleted.** Delete it.
* **I11.** `Interp::failure_site` is set first-call-wins; the guard is documented at `run.rs:1295-1298` and the callers are at `:882`, `:904` and `:1357`. A second raise after a trapped first one would report the first site. **This task is the caller that makes it matter.** Clear it on trap resumption, and write a test with two raises where the second is the one reported.
* **I13.** `Novalue::Unset` is produced by the read path and read by nothing. D16 required the flag from the start rather than retrofitting a raise into the hottest path. `SIGNAL ON NOVALUE` is its first reader, and 4c's gate program uses `signal on novalue`.
* **I14, the `+++` half.** Measured: a trapped `SIGNAL ON SYNTAX` under `trace r` emits **no `+++` and no error report at all**; the trap label's own clause is echoed as an ordinary `*-*`. Condition traps do not bring `+++` into 4b. `+++` is command errors and failures, Phase 7's under D18.
* **I16, revisited.** 4a concluded `SIGNAL ON SYNTAX` cannot accumulate temps leaks, resting entirely on `step_in_temps_frame` being the single chokepoint: the trap acts at instruction-loop level and the wrapper has truncated before the `Failure` reaches the loop's `Err` arm. **This task makes the trap real.** Re-verify against the implementation; if Task 1 moved execution off the chokepoint, redo the analysis. `.superpowers/sdd/2026-07-30-phase-4a-executor/temps-frame-investigation.md` has the original.

**The vacuity hazard specific to this task.** A trap criterion asserting "the handler ran" by checking that the program exited 0 is satisfied by a program that never raised. Assert a **value the handler sets**, and pick one that is neither the flag's derived name nor its unset rendering -- an unset read yields the derived name, so a flag left unset renders as plausible data.

- [ ] **Step 1: Measure the trap transcripts, stdout, stderr and rc separately**

At minimum: `SIGNAL ON SYNTAX` with a raise, trapped; `CALL ON ERROR NAME handler` with no command (measured, handler not invoked, rc 0); `SIGNAL ON NOVALUE` reading an unset variable; `RAISE SYNTAX 40.4`; `RAISE ... RETURN` and `RAISE ... EXIT`; `RAISE PROPAGATE` from inside a trap; a trap in a **caller** for a condition raised in a **callee**; `SIGNAL ON` inside an `INTERPRET` fragment.

- [ ] **Step 2: Write the failing tests, asserting handler-set values rather than exit codes**

- [ ] **Step 3: Add the trap table to `Activation`, with dispatch decided by the Step 1 measurements**

Whether traps inherit into a callee, and whether a condition raised in a callee propagates up to a caller's trap, are both measured in Step 1 rather than assumed. Do not carry over the first revision's guess.

- [ ] **Step 4: Implement `SIGNAL ON`/`OFF` and `CALL ON`/`OFF`**

`SIGNAL ON` transfers control and does not return; `CALL ON` calls the handler and resumes. The two differ in exactly the way that makes `failure_site` clearing necessary.

- [ ] **Step 5: Implement `Raise`**

`Raise` carries `condition`, `propagate`, `rc`, `description`, `additional`, `array` and `result`. `RaiseResult { exit, value }` is the `RETURN`/`EXIT` tail.

- [ ] **Step 6: Wire `Novalue::Unset` to `SIGNAL ON NOVALUE`; delete `Raised::condition`'s `#[expect(dead_code)]`**

- [ ] **Step 7: Clear `failure_site` on trap resumption, with a two-raise test**

- [ ] **Step 8: Re-verify the temps-frame conclusion against the real trap**

Report the verification with the program used. "It still holds" without a program is not a verification.

- [ ] **Step 9: Run the suite and the corpus gate, update `tests/owners.rs`, commit**

---

### Task 8: `PUSH`, `QUEUE`, and the in-process queue

**Files:**
- Create: `rust/crates/rexx-exec/src/queue.rs`
- Modify: `rust/crates/rexx-exec/src/lib.rs`, `src/run.rs`
- Modify: `rust/crates/rexx-exec/tests/owners.rs`
- Modify: `docs/superpowers/plans/phase-4-exclusions.txt`

**Why:** I15. The queue is 4b's; 4c's `QUEUED()` and `PARSE PULL` read it.

**This task ships with zero differential coverage, and must say so.** Every construct that can read the queue -- `PULL`, `PARSE PULL`, `QUEUED()` -- is 4c's. So a 4b program containing `PUSH`/`QUEUE` produces byte-identical output whether the queue stores anything or nothing, and `Push | Queue => Ok(Flow::Next)` would satisfy every instrument this plan schedules. Measured: a program whose whole body is `push "X"` / `queue "Y"` produces empty stdout, empty stderr, rc 0.

**So the coverage is a unit test, not a differential.** Test the queue type directly with a `#[cfg(test)] mod tests` beside `queue.rs`, asserting the interleaved order against values measured from the oracle in a **4c-shaped** probe (`push a; queue b; push c` then three pulls). The probe program is not a corpus program and does not go in the subset; it is how you learn the expected order.

**And the exclusions file must carry a KNOWN GAP row** stating that Task 8 shipped with no differential witness, and that its first one is 4c's first `PARSE PULL` corpus program. Without that row, a reader sees a green task and infers coverage that does not exist.

**The cross-process caveat, corrected.** The scoping document says the oracle's `rxapi`-backed queue makes cross-process differential runs impossible, and calls it live rather than theoretical. Measured 2026-08-03: a following separate `rexx` process reports `queued()` as `0`, so the queue is **not** shared across processes on this host. Record the single-program rule anyway -- it is right for a different reason, and depends on the host's `rxapi` state.

- [ ] **Step 1: Measure `PUSH`/`QUEUE` interleaving with a 4c-shaped probe, and record the expected order**

- [ ] **Step 2: Write the failing unit test against the queue type**

- [ ] **Step 3: Implement `queue.rs` and the two arms**

- [ ] **Step 4: Add the KNOWN GAP row and the single-program rule to the exclusions file**

- [ ] **Step 5: Run the suite, update `tests/owners.rs`, commit**

---

### Task 9: Trace for calls, and the Controlled-loop `>>>` gap

**Files:**
- Modify: `rust/crates/rexx-exec/src/trace.rs` (`>A>`, `>F>`, `>R>`, the activation indent base)
- Modify: `rust/crates/rexx-exec/src/run.rs` (the two missing `>>>` lines on a Controlled loop's re-tested pass)
- Modify: `rust/crates/rexx-exec/tests/trace_oracle.rs` -- including **`CLAIMED_PREFIXES` at `:233`**, which is asserted against the witness union and breaks the moment a prefix lands
- Modify: `docs/superpowers/plans/phase-4-exclusions.txt`

**Interfaces:**
- Consumes: Task 3's activation stack; `static_indent`, whose signature does not change; Task 2's clamp, which this task must not duplicate or contradict.

**Why:** the trace surface is where 4a's four late divergences were found, all by probing adjacent shapes rather than by the table.

**The measured `trace r` transcript for a two-argument call:**

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

Four things to read off it: the callee's `sub:` **label clause is echoed**, and so is `procedure`; callee clauses sit at the caller's indent plus two; `use arg` emits one `>>>` per argument; and **the return value is traced twice**, once at the callee's indent and once at the caller's.

**Inherited items this task pays for:**

* **I14, corrected twice.** 4b's prefixes are `>A>` (ARGUMENT, both call forms), `>F>` (FUNCTION, expression form only) and `>R>` (ALIAS). `>R>` is a **RESULTS**-level prefix, not intermediates-only: at `trace i` the call site shows `>O>   ">" => "PP"` and `>A>   "orig"` (the *value*, not the name), then the callee shows `>R>     "PP" => "Q"`; at `trace r` the `>R>` line is still emitted; at `trace l` only the label clause appears. **`>I>`/`<I<` are real and belong to `::routine`/`::method` under `TRACE LABELS`** -- the C++ gate is `tracingLabels() && isMethodOrRoutine()`. An internal-label call with `trace l` first emits nothing, with or without `PROCEDURE`; `trace l` in the caller targeting a `::routine` emits nothing. So their owner follows Task 3 Step 4's answer about whether 4b reaches a `::routine`. **Write the answer into `phase-4-exclusions.txt` as "`::routine`/`::method` under TRACE LABELS", not "method invocation".**
* **I31.** A Controlled (`TO`-style) loop's re-tested pass omits two `>>>` value lines. Measured, cause read from `DoBlock::checkControl` (`interpreter/.../DoBlock.cpp`) rather than inferred, and costed at about twenty lines plus re-verification of bound-before-test, `FOR` and `ITERATE` -- half a day, not a rewrite. **Close it here.** An overstated cost is how a cheap fix stays open, and this row was corrected once for exactly that.

- [ ] **Step 1: Regenerate the five existing trace expectations to prove the harness still round-trips**

`tests/trace_oracle.rs`'s module doc carries the regeneration command, and all five were verified byte-identical in 4a.

- [ ] **Step 2: Probe `trace l` on a `::routine` and on a `::method`, and settle `>I>`/`<I<`'s owner**

Not `trace i` on an internal label -- that instrument cannot produce them, which is how the first revision of this plan reached a wrong conclusion.

- [ ] **Step 3: Commit the new expectations, and update `CLAIMED_PREFIXES`**

- [ ] **Step 4: Implement `>A>`, `>F>` and `>R>`**

- [ ] **Step 5: Add the activation indent base, per D2r**

The base is the **calling clause's printed indent** plus the delta, not two times the depth. `static_indent` stays a pure function of the flat instruction list. Task 2 clamped the `*-*` echo at 40 columns and the value lines are **not** clamped -- measured, at nesting depth 25 `*-*` tops out at 40 while `>>>` runs to 52. Do not extend the clamp to value lines and do not re-implement it.

- [ ] **Step 6: Close I31's two missing `>>>` lines, and re-verify bound-before-test, `FOR` and `ITERATE`**

- [ ] **Step 7: Add a coverage measure to the trace table**

D14's criterion 3 amendment. The honest statement today is that the five witnesses verify what they cover and the trace surface's coverage is measured by nothing. Produce a number: which of the nineteen prefixes have a committed expectation, which do not, and which are out of scope with an owner. **A printed number that no assertion reads cannot fail** -- assert the count against a committed literal.

- [ ] **Step 8: Run the suite, remove I31's KNOWN GAP row, commit**

Removing a KNOWN GAP row needs the gap closed and a witness in the tree.

---

### Task 10: The 4b corpus and the collector

**Files:**
- Create: `rust/corpus/phase-4b.txt`, `rust/corpus/proc/*.rex`
- Modify: `rust/corpus/phase-4a.txt:18`, `rust/corpus/README.md:108-109`
- Modify: `rust/crates/rexx-exec/src/lib.rs` if the collector sweep finds a second under-rooting site

**Interfaces:**
- Consumes: Task 0's subset union.

**The corpus rules for 4b, binding every program in `phase-4b.txt`:**

* **No `DO OVER` on a stem.** The oracle walks a balanced tree and we use a hash map; measured, tails 1, 2, 3, 10, ZZ, B yield `1 B 3 2 ZZ 10`. Such a program could never pass.
* **No builtin calls.** 4b depends on 4c nowhere, and a program written naturally will reach for builtins to make a routine do something observable. `say`, assignment and arithmetic are enough.
* **No `PARSE`.** 4c's.
* **No `::routine` or other directive**, unless `assert_program_has_no_directives` (`tests/coverage.rs:331`) is amended first -- it rejects any corpus program with a directive, and that is a deliberate property of the 4a subset.

**Inherited items this task pays for:**

* **I19.** `EXIT`'s result is under-rooted from the temps-frame pop to `exit_code_for` -- under-rooting, the direction that breaks when a collector lands, and longer than any window the crate documents. Harmless today only because nothing between that pop and `exit_code_for` calls `alloc_with`. The pointer was deliberately placed on `Heap::collect` in `rexx-core` rather than at the leak site, because the person who turns this into a use-after-free is whoever wires a collector in -- **and anyone doing 4b work in `rexx-exec` will never see it.** That instruction also says to sweep `rexx-exec` for the same shape first. **Do the sweep** and report what it found, including "nothing".
* **I20.** The collect-on-every-allocation mode has never seen a call frame. Criterion 4 passed on 29 programs, all 4a-shaped. `collect_stress` must run the **union**, and 4b's programs must include calls, arguments and exposure.
* **I26 and D7's documentation half.** Correct both `List` comments to say Phase 5.

- [ ] **Step 1: Write the 4b corpus programs**

One per construct at minimum, and **at least three that combine two**: a raise inside a routine inside a loop; an exposed stem mutated by a callee; a trap that resumes and then raises again. The combinations are the point. 4a's whole-branch review found two Criticals that survived 824 tests, a 29-of-29 byte-identical corpus, nine per-task reviews, seven gate criteria and a nine-mutation script -- because the coverage criterion enumerates variants and asserts nothing about combinations. `Stem` had a witness. Arithmetic had witnesses. `Stem` as an arithmetic operand had none, and `a. = 5; say a. + 1` aborted the process.

- [ ] **Step 2: Run the corpus in report mode, then strict mode**

Run: `cargo test -p rexx-exec --test corpus`, then `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus`. A failure naming `4c` is expected; one naming `4b` is this plan's work.

- [ ] **Step 3: Sweep `rexx-exec` for the I19 under-rooting shape, and report the result**

- [ ] **Step 4: Run `collect_stress` over the union**

- [ ] **Step 5: Correct the two `List` comments, commit**

---

### Task 11: The `base/keyword` L1 table

**Files:**
- Modify: `rust/crates/rexx-extract/src/lib.rs` (a third extractor)
- Create: `rust/crates/rexx-exec/tests/keyword_assertions.rs`
- Modify: `docs/superpowers/plans/l1-coverage.md`

**Why:** I28. The split table names `base/keyword` as 4b's L1 obligation and `base/bif` as 4c's, and **nothing extracts either today.** `rexx-extract` has `extract` (test methods) and `extract_assertions` (the `base/expressions` table) and nothing else.

**D12 is settled for 4b as a third extractor, not a generalisation.** `extract_assertions` is specific to `base/expressions`'s `assertSame` shape and already needed two modelling corrections -- single-quoted method names, and `expectSyntax` markers changing what a later `assertSame` *means*. Both were about a group's mechanics, and the same shape will recur. 4c makes its own call for `base/bif`.

**The conservation invariant alone is a tautology, and the first revision required it alone.** `rows + dropped == calls` holds at `0 + 0 == 0` (nothing scanned) and at `0 + N == N` (everything dropped). What made it non-vacuous in 4a was the *companion* test pinning absolute totals -- `total_rows`, `total_dropped`, per-group counts -- plus a non-empty assertion. Require all three:

1. `rows + dropped == calls`.
2. **Absolute committed literals**: the file count scanned, `calls`, `rows`, `dropped`, and the per-group row counts. Commit the file list the extractor scans, the way `phase-4a.txt` is pinned.
3. A floor: the row set is not empty.

**State the denominator's assertion spelling in the criterion's own text.** Measured on `ootest/ooRexx/base/keyword`: 39 `.testGroup` files, **2,561** `self~assertSame` occurrences but **4,567** `self~assert*` occurrences of all spellings. If `assertSame` is the denominator, 2,006 assertions -- 44% of the group -- sit outside the conservation population entirely while the law holds exactly. Say which spelling counts, and say how many assertions are deliberately outside it.

**What this task must not promise.** `tests/assertions.rs`'s 35 exempt rows will not move. All 35 are `unblocked_by: "Phase 5"`, verified 35 of 35, and the two whose first-observed blocker is a 4b construct re-block on a message send one line later in the same prelude. What 4b owes that harness is nothing, except that `the_exempt_set_matches_the_current_blocked_rows` fails if a listed row starts passing, so an accidental improvement shows as a red test.

- [ ] **Step 1: Check whether `base/keyword` uses a shape `extract_assertions` already models**

Ten minutes, and it decides whether a third extractor is doing unnecessary work. Report the answer either way.

- [ ] **Step 2: Write the three tests first -- conservation, absolute literals, non-empty**

Write the absolute-literal test with the numbers you measured in Step 1 and watch **that** one fail. The conservation test passes at zero and cannot be the red.

- [ ] **Step 3: Write the extractor**

- [ ] **Step 4: Commit the row table, the scanned file list, and the `EXEMPT` list**

- [ ] **Step 5: Run it in report mode and record the pass rate**

Report mode first. A strict gate on a table nobody has looked at turns a measurement into a blocker.

- [ ] **Step 6: Add a `REXX_KEYWORD_GATE=1` strict mode matching the existing convention, commit**

---

### Task 12: The 4b gate

**Files:**
- Create: `docs/superpowers/plans/phase-4b-gate.md`
- Create: `rust/scripts/mutate-4b.sh`
- Modify: `docs/superpowers/plans/phase-4-exclusions.txt`

**Why:** D13 and D14.

**Write each criterion so it can fail.** Four criteria in 4a could not, and each was caught late: criterion 6's predecessor was satisfied by `/bin/true`; criterion 4 had no way to fail at all until it was rewritten, then no *subject* until the mode it named was built; `strict_comparison_never_calls_to_number` returned before reading either operand; the CONCATENATION rows would have passed while testing nothing. **And `mutate-4a.sh` itself reported 9 of 9 caught with the oracle absent**, because any non-zero exit counted as a catch. It has since gained `require_baseline_pass` and a `subset_status` distinguishing PASSED, DIVERGED and INFRA_FAILURE. Reuse that guard; the mutations are not reusable.

**Four 4b-specific vacuity shapes to write against:**

* **A trap criterion asserting the program exited 0.** A program that never raised also exits 0. Assert a value the handler set, neither the flag's derived name nor its unset rendering.
* **Criterion 4 carried forward verbatim.** It passes today with zero call frames exercised. Carried unchanged it cannot fail *for the thing 4b adds*. It needs the subset union and an **activation-shaped negative control**: 4a's control deletes `eval_arithmetic`'s `push_temp(left_value)`. A 4b control must delete a root a *call* holds -- the argument list between evaluation and the callee's `USE` -- or it re-tests 4a and reports a pass that means nothing.
* **A coverage criterion that enumerates variants** says nothing about combinations, which is how 4a's two Criticals survived everything. State that limit in the criterion's own text.
* **A queue criterion.** Task 8 has no differential witness at all. Any criterion covering it must assert unit-level order against oracle-measured values, and the gate must record that the construct ships undifferentiated.

**State up front rather than discover:** criterion 2's exempt list cannot light up at this gate or 4c's.

**I27 is not this gate's criterion.** The 342 expected trace-output lines in `TRACE.testGroup` are 4b's and 4c's to satisfy, but the group is not runnable as extracted, and the same file yields 239, 342, 374, 393 and 437 under five defensible anchorings -- three recounts have already gone astray. If this gate uses any figure from that file, **state which scan produced it**. Prefer a named, measured subset.

- [ ] **Step 1: Write the gate document before running anything**

Criteria first, results second. A criterion written after the measurement is a description of what happened.

- [ ] **Step 2: Write `mutate-4b.sh` with 4a's guard and 4b-shaped mutations**

Include the activation-shaped negative control, and I17's reclassification with its real mechanism.

- [ ] **Step 3: Run every gate, reading each exit status unpiped**

Full suite; `REXX_CORPUS_GATE=1`; `REXX_ASSERTIONS_GATE=1`; `REXX_KEYWORD_GATE=1`; `cargo clippy -- -D warnings`; `cargo fmt --check`; the mutation script.

- [ ] **Step 4: Assess each criterion honestly, including the ones met weakly**

4a's gate recorded five met, one met with an inherited criterion defect, and one met weakly with an open gap. That is what an honest gate looks like. A seven-of-seven with no qualifications, after a sub-phase this size, is a claim about the instruments rather than the code.

- [ ] **Step 5: Commit the gate document, read the hash back, record it in the ledger**

---

## Explicitly not in scope, and not promised

* **`tests/assertions.rs` moves not at all.** All 35 exempt rows need Phase 5.
* **`Plan::by_symbol` stays a `HashMap`** (I35). D16's shape wants a `Vec` index, and `SymbolId::index()` landed so the swap is this crate's decision. Variable lookup is 8.1%/32.2% of runtime, so it deserves its own measurement rather than arriving as a side effect. `Option<usize>` is still required because keywords, labels and constants share the `SymbolTable`, so a dense `Vec` has holes.
* **The parser's recursion cliffs** (I32, I33, I34). Prefix-operator chains recurse in `message_subterm` outside the shared depth budget, aborting between 1,150 and 1,200 levels on a default 2 MiB thread, and the oracle's cliff for that construct has never been measured. `Debug`, `PartialEq` and `Clone` on `Expr` are still recursive, cliffs at 2,000/2,050 and 2,100/2,200; the trigger is the first test that formats or compares a deep tree. The depth counter protects a sized caller only -- the native abort arrives at 331 parens or 341 calls. Task 3 Step 1 measures 4b's contribution; it does not fix them.
* **`Heap::alloc`'s friendly name.** Still bypasses the stress hook. Parked in 4a by choice.
* **A `DO`/`LOOP` temps frame growing for its whole run.** Parked in 4a by choice.
* **Everything 4c owns** (I23, I24, I29, I30): `PARSE` in all forms, the 66 in-scope builtins, `ADDRESS` and `ADDRESS()`, `VALUE`'s variable-access form, `QUEUED()`, `ARG()`, `CONDITION()`, and the `rexxcps.rex` gate question -- D8, it auto-adjusts its loop count from measured wall-clock time, so it cannot be the byte-for-byte differential the parent plan assumes.
* **The five 4c-only open decisions**: D4 (11 of 15 excluded builtins genuinely blocked; `QUALIFY` not blocked at all; `USERID`/`SETLOCAL`/`ENDLOCAL` blocked because `std::env::set_var` is `unsafe` in edition 2024 and the workspace forbids it), D7, D8, D11 (measured, unseeded `RANDOM` is deterministic across separate processes, so "the values differ between runs" is not evidence of randomness), D12 for `base/bif`.
