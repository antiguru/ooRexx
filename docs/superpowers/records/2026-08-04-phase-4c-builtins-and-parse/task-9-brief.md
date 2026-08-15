### Task 9: the `ADDRESS` instruction and environment tracking

**Files:** modify `crates/rexx-exec/src/run.rs`, `crates/rexx-exec/src/activation.rs`, `crates/rexx-exec/src/lib.rs` (`instruction_owner`, `:761`), `tests/owners.rs`, `tests/loud.rs`, `tests/coverage.rs`, `rust/corpus/phase-4c.txt`

**Scope is the environment name only.**
`ast::Address` carries `environment`, `dynamic`, `command` and `io`.
This task implements `environment` and `dynamic`.
**`command` and `io` are Phase 7's under D18 and must still fail loudly naming Phase 7.**

- [ ] **Step 0: Measured, 2026-08-05 -- read before writing any code**

**(a) The constant form UPPERCASES the name; the `VALUE` form does not.** This is the cheap one to get wrong:

```
address()  before anything  ->  sh
address envC               ->  ENVC
nm = 'envC'; address value nm  ->  envC
```

Same intent, different `ADDRESS()`.

**(b) Bare `ADDRESS` is a TOGGLE, not a stack.** Measured: `envA`, `envB`, bare -> `ENVA`, bare -> `ENVB`, bare -> `ENVA`. It swaps between the current and the previous, forever. A stack implementation is wrong from the third bare `ADDRESS` onward.

**(c) It is per-activation state, like `NUMERIC` -- inherited on call, discarded on return.** Confirmed rather than inferred: a called internal routine sees the caller's environment, changes it, and the caller still sees its own after the return. **Task 13 depends on this.**

**(d) `DIGITS`/`FORM`/`FUZZ`: an internal routine inherits, a `::routine` does not.** Measured with the caller at `12 ENGINEERING 3`:

```
internal (call shownum)   ->  12 ENGINEERING 3
::routine showint         ->   9 SCIENTIFIC  0
```

`TRACE()` behaves the same way. **This is the measurement Task 13's non-inheritance table needs**, taken from the builtins' side rather than inferred from the trace.

*Not isolated, flagged:* bare `ADDRESS` with **no** prior environment was never probed -- every probe set one first.

- [ ] **Step 1: Measure the default and the swap semantics**

Measured: `say address()` with no `ADDRESS` instruction prints `sh` on this host.
**That default is platform-supplied and therefore Phase 7's**, so a corpus witness must assert the **swap**, not the initial value.

`ADDRESS` with no operand swaps to the previous environment.
Probe: the two-deep swap; the swap with no prior environment; and whether the setting survives a `CALL` and a `RETURN`.
It is per-activation state like `Settings`, so it belongs on `Activation` and not on `Interp` -- the same shape as 4b's `trace_mode` move.
**Measured (D-R): a `::routine` does *not* inherit it**, which Task 13 depends on.

- [ ] **Step 2: Split the `loud.rs` witness rather than deleting it**

`loud.rs:208`'s witness is `address cmd` -- an `ADDRESS` **with a command**, which stays Phase 7's.
Deleting the row removes the witness for a half that is still out of scope, and `assert_witness_set_is_complete` cannot catch that.
**Make the row arm-grained**, following the pattern already in `owners.rs:349-351`: the environment form moves in scope, the command and `WITH` forms keep a Phase 7 witness.

- [ ] **Step 3: Write failing tests, implement, move the `owners.rs` row**

- [ ] **Step 4: Run the shared verify block and commit**

---

