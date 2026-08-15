### Task 8: program arguments, `ARG`, `PULL`, `PARSE PULL`, `PARSE LINEIN`

**Files:** modify `crates/rexx-exec/src/run.rs`, `crates/rexx-exec/src/lib.rs` (`instruction_owner`, `:758`; `run_program`'s signature), `crates/rexx-exec/src/bin/rexx-run.rs`, `tests/owners.rs`, `tests/loud.rs`, `tests/coverage.rs`, `rust/corpus/phase-4c.txt`

**Consumes Task 7's engine unchanged and 4b's `Interp::queue`.**

- [ ] **Step 0: The input model, measured -- read before designing the plumbing**

**(a) A command line can NEVER produce more than one argument string.** This is the design input Step 1 was missing, and the answer is the plain negative.

| invocation | `ARG()` | `ARG(1)` |
|---|---|---|
| `rexx p.rex` | **0** | omitted |
| `rexx p.rex a b c` | 1 | `a b c` |
| `rexx p.rex "a b c"` | 1 | `a b c` |
| `rexx p.rex a,b,c` | 1 | `a,b,c` |
| `rexx p.rex ""` | **1** | `` (empty) |

* **`ARG(2)` is unreachable from a command line.** A single optional string suffices.
* **`rexx p.rex a b c` and `rexx p.rex "a b c"` are indistinguishable to the program** -- argv is joined with one blank, internal spacing preserved.
* **Commas are not separators**: `parse arg a1, a2` on `a,b,c` puts the whole string in `a1`.
* **An empty argument still counts.** `rexx p.rex ""` is `ARG()` = 1 with an empty string, distinct from no argument at all. **Modelling "no arguments" as `Some("")` gets this case wrong** -- it must be `None` versus `Some("")`.
* The `-e` form behaves identically. `ARG() > 1` is reachable only by invoking a program as an **external routine**, which Phase 4 does not do.

**(b) `PULL`, `PARSE PULL`, `PARSE LINEIN` and `LINEIN()` share ONE cursor, and it is `.input`.**
This is the structural finding, and it is asserted in exactly one place -- `runtime.objects/environmentEntries.testGroup:174-181`, not in any `PARSE` or `PULL` group:

```rexx
.input~destination(.ArrayStream~of("a", "b", "c", "d", "e"))
pull a;  parse pull b;  parse linein c;  d = linein();  e = .input~lineIn
self~assertSame("A b c d e", a b c d e)
```

Five constructs, one shared position, and the expected value encodes that **only `PULL` uppercases**.
Measured directly:

```
queue empty, stdin two lines:  parse pull -> line-A ; parse linein -> line-B ; parse pull -> ''
queue non-empty, same stdin:   parse pull -> the queue entry ; parse linein -> line-A
```

**`PARSE LINEIN` never consults the queue.** With a non-empty queue, adjacent `PARSE PULL` and `PARSE LINEIN` lines return different things from different places.
So the model is **not** "each instruction reads stdin"; it is one object with a position that all of them advance.

**(c) An empty queue with stdin at `/dev/null` gives the null string** -- length 0, rc 0, no error, no condition, **no hang**, and repeatable.

**(d) `PULL` uppercases; `PARSE PULL` does not.** Measured on the same run: `[LOWER ONE] [lower two]`.

**(e) `ARG template` is `PARSE UPPER ARG template`, confirmed -- but the keyword is only a keyword BEFORE `ARG`.**
`arg upper t4` consumes `UPPER` as a **template target**, not as a modifier: with arguments `mIxEd CaSe`, it assigns `UPPER='MIXED'` and `t4='CASE'`.
Bare `ARG` and bare `PARSE ARG` with no template are legal and do nothing observable.

**(f) Trace, under `trace i`.** `ARG` is the **only** source emitting no `>K>`, in both spellings. `PULL` and `LINEIN` emit it, and for `PULL` the `>K>` carries the **pre-uppercase** value while `>>>` carries the uppercased one -- the two lines differ on the same instruction.

**This survey is `trace i`-only and that is a known blind spot, not a completeness claim.**
Task 7's Step 0(g) carries the rule this one cannot show: an assigned target's value line is a **choice of prefix**, `>=>` at `trace i` and `>>>` at `trace r`, never both, and the prefixes do not share one gate (`>K>` and every `>>>` are `results`; `>L>`/`>V>`/`>C>`/`>=>`/`>.>` are `intermediates`).
**Probe both modes for the two sources this task adds**, and do not infer a gate from `DO`'s use of the same prefix.

**(g) What the suite checks.** `bif/ARG.testGroup`: 17 methods, 64 `assertSame`, **2 `expectSyntax`, both 40.14**. `bif/LINEIN.testGroup`: 6 methods, 9 `assertSame`, **0 `expectSyntax`**.
A bare-word `ARG` scan returns **835** hits against **4** for the instruction syntax -- a ~200× overstatement, and **one of those 4 is `arg = 4`**, an assignment to a variable named `arg`. Require the syntax.

**(h) Probe hygiene, and this one bit both the surveyor and me within minutes of each other.**
`a`, `b`, `c` are the natural names for parse targets and **`b` is poisoned**: `say "["a"]["b"]["c"]"` makes `b"]["` a **binary literal**, failing with `Error 15.4` or `15.6` pointing at the `say` line and naming neither the variable nor the cause.
**Use `n1`, `n2`, `n3`.** The warning was in front of me when I wrote the probe and I hit it anyway; assume you will too.

- [ ] **Step 1: Supply a top-level program's argument string**

**Nothing in the crate provides one today.** `run_program` takes no arguments and neither does `rexx-run`, yet `PARSE ARG` at top level, the `ARG` instruction and `ARG()` all read it.
This task adds the plumbing: `rexx-run` accepts trailing arguments after the program path, and `run_program` gains a parameter carrying them.

**Measure the oracle's model first.** `rexx foo.rex a b c` supplies **one** argument string, not three -- confirm that, and confirm what `ARG()` returns for it, before choosing the representation.

**The standing oracle wrapper passes no arguments**, so a corpus witness cannot exercise this unless the harness is extended.
If extending it is out of scope here, say so and record a `KNOWN GAP` row rather than shipping an untested path.

- [ ] **Step 2: Measure the queue-empty fallback**

`PULL` and `PARSE PULL` read the queue and **read stdin when the queue is empty**.
Probe with stdin at `/dev/null` and with a here-string, and record what an empty queue plus closed stdin produces.
`PARSE LINEIN` reads stdin directly.

**This is where 4b's `PUSH`/`QUEUE` gets its first differential witness.** 4b's gate records the queue as shipping with storage verified only in-crate; a corpus program that pushes and pulls closes that, and Task 15's gate should say so.

- [ ] **Step 3: Confirm `ARG template` is `PARSE UPPER ARG`**

They are the same instruction with `upper` set. Confirm rather than assume, and confirm `ARG` with no template is legal and does nothing.

- [ ] **Step 4: Write failing tests, implement, move `Arg` and `Pull` in scope**

Both `owners.rs` rows, both `EXPECTED_OUT_OF_SCOPE` rows, `lib.rs:758`'s arm, and both `loud.rs` witnesses deleted.

- [ ] **Step 5: Run the shared verify block and commit**

---

