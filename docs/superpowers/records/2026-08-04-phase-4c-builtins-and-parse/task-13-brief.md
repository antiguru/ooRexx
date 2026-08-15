### Task 13: `::routine` dispatch, `>I>` and `<I<`

**Files:** modify `crates/rexx-exec/src/run.rs`, `crates/rexx-exec/src/plan.rs`, `crates/rexx-exec/src/lib.rs`, `crates/rexx-exec/src/eval.rs`, `crates/rexx-exec/src/error.rs`, `tests/coverage.rs`, `tests/trace_oracle.rs`, `docs/superpowers/plans/phase-4-exclusions.txt`, `rust/corpus/phase-4c.txt`

**Why last of the implementation tasks.** Builtins shadow `::routine`s, so the resolution order cannot be verified until the table is complete.

- [ ] **Step 1: Set `BodyKey::directive` at the production site**

`plan.rs:79`'s `directive: Option<usize>` is `None` at **nine** construction sites, eight of which are test-only.

**The production site is the `BodyKey` literal inside `Interp::run_program`'s `plan_for` call** -- `lib.rs:1690-1697`, with `directive: None` at `:1693`, building the plan for the main body. Re-derive it rather than trusting the line: classify each site by whether it sits after its file's last `#[cfg(test)]`, and exactly one comes out production.

`plan.rs:631` is inside a `#[cfg(test)] mod tests` (the `cfg` is at `:615`), and the first revision of this plan sent an implementer to that fixture.

*(Re-counted 2026-08-07. The earlier "seven sites, six test-only, production at `lib.rs:1422`" was measured before Tasks 11 and 12 added fixtures and moved `lib.rs`; `:1422` is now a doc comment about `Invocation`, which is what a stale line number looks like when it still lands on plausible-looking code.)*

- [ ] **Step 2: Give a `::routine` activation its own pool, and five other non-inheritances**

D-R's table lists six measured differences from an internal label's activation.
**`Activation::nested` inherits `NUMERIC`, `ADDRESS` and the condition traps, and a `::routine` inherits none of the three** -- two of those three failures are silent.

The constructor is `activation.rs:586-619` and it takes an `Inherited` struct: `settings`, `trace_mode`, `address`, `traps`, `condition`. Its own doc already records which fields are deliberately *not* inherited and why, so add your reasoning there rather than beside the call site. *(Corrected 2026-08-07: this step used to cite `run.rs:3304-3313`, which is the `raise propagate` condition-restoration block and never inherited anything. The fact was right and the address was wrong.)*

**This is not 4b's `PROCEDURE` isolation reused.** A `::routine` has a different `CodeBody`, therefore a different `Plan`, therefore a different name-to-slot map, so the slot-index-identity property `PROCEDURE EXPOSE`'s alias bitset rests on does **not** hold across bodies.

- [ ] **Step 3: Place it third, and record what sits in front of 43.1**

Internal label, then builtin, then `::routine`.
Measured: `call max 1, 9` with a `::routine max` present returns 9.
**A quoted target is a second order:** `call 'ZORKOLO'` skips the internal label but still finds the `::routine`, while the builtin still wins.
Routine lookup **upcases both sides**, unlike `CodeBody::labels`.

**Replacing `Loud::unresolved_call` with an unconditional 43.1 is wrong, and this is the correction that matters most in this task.**
The oracle searches for an **external file** before raising 43.1: measured, with `zorkolo.rex` in the current directory, `call zorkolo` runs it at rc 0, and with a `::routine zorkolo` present the routine wins.
4c does not implement external routine resolution.
So this task must:

* raise 43.1 when the name resolves to none of label, builtin or `::routine`;
* add an `EXCLUSIONS` row -- **external routine resolution, Phase 7** -- carrying both transcripts;
* add a corpus rule that no corpus program may depend on external routine resolution, and note that the scratchpad's stale `.rex` files make this the easiest probe error in the phase;
* settle the contradiction between `lib.rs:501` ("the builtin table and then external resolution, and **both are 4c's**") and `eval.rs:484` ("builtin second (4c), external third (Phase 7)") in the same commit. *(The `lib.rs` line was cited as `:479` before Tasks 11-12 moved it.)*

`eval.rs:507`'s own comment argues against an unconditional substitution here -- read it before changing it.

- [ ] **Step 4: Fail on directive RESOLUTION failure, not on directive presence**

An earlier revision of this step said "add a loud failure naming Phase 5 for any directive that is not `::ROUTINE`".
**That is too aggressive and would diverge from the oracle on programs it runs fine.** Measured 2026-08-05:

```
::class foo                              -> rc 0, "main ran"
::class foo + ::method bar               -> rc 0, "main ran"
::class foo + ::attribute baz            -> rc 0, "main ran"
::requires 'helper.rex'   (file present) -> rc 0, "main ran"
a loose ::method with no ::class         -> rc 0, "main ran"

::class foo subclass zzznotaclass        -> 98.909, rc 158, stdout EMPTY
::requires 'no_such_file_zz.rex'         -> 43.901, rc 213, stdout EMPTY
```

**The oracle fails only when a directive fails to RESOLVE, never on mere presence**, and it fails **before `main` runs** -- stdout is empty in both failing cases.

So the rule is: a program that *contains* a well-formed directive it never uses must run identically on both interpreters.
A program that *uses* one must not silently do the wrong thing.
The boundary case that shows the difference: `::requires 'helper.rex'` where the helper holds `::routine helperfn public` makes `helperfn()` callable and returning `HELPED` -- so `::REQUIRES` **imports names into the resolution chain**, and ignoring it changes which routine a call finds.

**Detect use, not presence.** Failing loudly on presence rejects valid programs; ignoring resolution failure runs a program the oracle refuses.

- [ ] **Step 5: Trace does not cross into a `::routine`**

Measured: a caller's `trace r` echoes its own clauses and none of the routine's, while an internal label's clauses *are* echoed under the same setting.
A deliberate difference between two paths that share a function.

- [ ] **Step 6: `>I>`/`<I<` under the two-condition gate**

`tracingLabels() && isMethodOrRoutine()` (`RexxActivation.cpp:3655`).
Measured:

* `trace l` in the **caller**, targeting a `::routine` -> nothing. The caller's setting does not cross.
* The routine's **own** non-dynamic trace instruction fires both lines, and `earlyTraceEntry` accepts **A, I, L and R** -- so `trace r` in the routine body emits them too.
  Any routine-body trace witness therefore emits them whether it intends to or not.
* The content is **not** a clause echo. It carries a **7-space leading indent**:

```
       >I> Routine "ZORKOLO" in package "<absolute path>".
       <I< Routine "ZORKOLO" in package "<absolute path>".
```

**The exact bytes, confirmed with `cat -A` so trailing whitespace is visible:**

```
       >I> Routine "RTN" in package "/abs/path/own_a.rex".$
       <I< Routine "RTN" in package "/abs/path/own_a.rex".$
```

**Seven leading spaces**, the routine name **uppercased** and double-quoted, the package an **absolute path**, a **trailing period outside the quotes**, and no trailing whitespace. `<I<` is identical in form. All on **stderr**.

Fires for exactly **A, I, L, R** -- verified by running the same routine under all nine letters; `n`, `c`, `e`, `f`, `o` produce zero stderr.

The absolute path makes any committed expectation host-dependent, so **the witness lives in the live corpus, not in `tests/trace_oracle/`**.
The package field is **one `String` on `Interp`**, not a package object.

**`::options trace labels` IS a second reachable route, and an earlier revision of this step said it was not.**
Measured: a routine with **no `trace` instruction of its own**, in a file carrying `::options trace labels`, emits both lines with identical bytes.
So "the routine's own trace is the only path" is false; record the `::options` route rather than implementing it, but do not write that it does not exist.

- [ ] **Step 7: Let a `::routine` program into the corpus**

`tests/coverage.rs`'s `assert_program_has_no_directives` (`:150`) panics on any `::` directive in the subset union, and this task's witness is the first corpus program to carry one.
Relax it to permit `::ROUTINE` and keep the panic for the rest.
**Without this, keeping the witness out of the subset file is the path of least resistance, and then the `>I>`/`<I<` witness is read by no harness and the "`::routine` before builtins" mutation loses its catcher.**

- [ ] **Step 8: Flip `>I>`/`<I<` to `Witnessed` and move both counts**

`WITNESSED_PREFIX_COUNT` and `OUT_OF_SCOPE_PREFIX_COUNT` each move by two.

Both live in **`tests/trace_oracle.rs`**, at `:621` and `:625`, currently **14** and **5**; the assertions that read them are at `:693`-`:696`, and the third one checks the sum. *(This step used to place them in `tests/coverage.rs` at `:551`/`:555`, where neither name occurs.)*

- [ ] **Step 9: Stop rendering an argument nobody will print**

**Inherited from Task 3's fix round, and assigned here because this task restructures `resolve_and_run_call` anyway and is the last 4c task whose file list already contains `run.rs`.**

`resolve_and_run_call` renders every evaluated argument to an owned `Vec` purely to hand it to `trace_argument`, which discards it unless the trace mode asks for intermediates.
A value of any size is therefore copied whether or not anything will print it, the copy is unguarded, and it **aborts the process** rather than raising.
Measured at the project's own `ulimit -v 1048576`: `say length(copies('a',400000000))` is `400000000` at rc 0 on the oracle and **SIGABRT at rc 134** here.

**Measured:** with the argument render placed behind `if self.trace_mode().intermediates`, `say length(copies('a',N))` at both 300000000 and 400000000 returns rc 0 with the oracle's answer.
That experiment was run and reverted, not shipped.

**It is not the whole of the cause, and an earlier revision of this step said it was.**
With the same experiment applied, four one-line neighbours still SIGABRT at 400 MB where the oracle returns rc 0:

```rexx
say length(strip(copies('a',400000000)))
say length(reverse(copies('a',400000000)))
say length(copies('a',400000000) || 'x')
x = copies('a',400000000)
```

The first two are `required_string`'s infallible `into_owned()`, the third is `Interp::concat`, the fourth the assignment path's own render.
**You are looking for a shape -- an unguarded owned copy on a path that may discard it -- not for one line.**
The false generalisation came from probing a single expression shape (`length` of a `copies`); vary the surrounding expression before believing any fix is complete.

**It costs real memory, not only address space, and that is the stronger reason to fix it.**
Measured at a 4 GiB limit where both sides succeed, peak RSS for `say length(copies('a',500000000))` is **978,460 kB here against the oracle's 495,860 kB** -- the oracle holds one copy of the result and this crate holds two.
482 MB of the 483 MB difference is one redundant copy of a 500 MB string.
**Re-measure that pair after your fix and expect parity.**
That expectation is arithmetic rather than a measurement, so treat a non-parity result as a finding rather than as noise.

**What the measurement does not say is which of the roughly fifteen similar sites need the same guard, and a wrong guard silently drops a trace line.**
So: **owe a trace witness for every site you change**, and change no site you cannot witness.
The assignment path's own `>>>` render is the same shape.

The **second** cause in that gap row -- `ulimit -v` limiting address space while `INTERPRETER_STACK_BYTES` reserves 512 MiB of it -- is **not yours and not closable in Phase 4**. It follows from D19's sized-thread choice. Do not attempt it, and do not let its presence stop you closing the first.

- [ ] **Step 10: Run the shared verify block and commit**

---

