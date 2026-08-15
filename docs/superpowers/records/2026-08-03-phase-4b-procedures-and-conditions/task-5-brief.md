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
* **Where the redirect lives is this task's decision, made with a measurement.** Either `Interp`'s slot resolution returns a `(SlotFrame, usize)` pair and every site changes -- adding a check to a path that is 8.1%/32.2% of runtime -- or the indirection goes into `RootSet`, which amends `rexx-core`. **Measure the hot-path cost before choosing**, and report both the choice and the number.

  **Count the sites yourself; the recorded figure has already gone stale once.** At `344677e6` there are **26** non-comment sites resolving the frame from the top activation -- `eval.rs` 3, `stem.rs` 12, `plan.rs` 4, `lib.rs` 2, `run.rs` 5. An earlier revision of this plan said twelve with seven in `stem.rs`, which was true before the activation stack existed. `grep -c` overcounts by including doc comments; filter them.
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

