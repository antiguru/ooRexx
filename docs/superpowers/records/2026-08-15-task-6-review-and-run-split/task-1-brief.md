### Task 1: close the `INTERPRET` regression `e74780054` caused

`e74780054` gave a plain `DO` a header clause and an `END` clause. Inside an `INTERPRET` fragment
those are boundaries the oracle does not have, and they take a delivery that the oracle leaves for
the `INTERPRET` clause's own boundary. This is review finding **C1**, and the disposition is
**close it**, decided by Moritz on 2026-08-15.

#### The program and what it does now

```rexx
call on user c1 name h1
call on user c2 name h2
zr = raiser()
interpret 'do; say ''body''; end'
say 'after' zr
exit 0
raiser:
raise user c1 return 5
h1:
say 'h1' sigl
raise user c2 return 1
h2:
say 'h2' sigl
return
```

Measured by the controller at `56d9d1c86`, `rc 0` everywhere, stderr empty:

| | stdout |
|---|---|
| oracle | `h1 3` / `body` / `h2 4` / `after 5` |
| `1f4176b47`, both engines | `h1 3` / `body` / `h2 4` / `after 5` (matched) |
| `56d9d1c86`, both engines | `h1 3` / `h2 4` / `body` / `after 5` (diverges) |

The rule the oracle follows is already written down in `crates/rexx-exec/src/clause.rs`'s module
doc, with the measurement behind it: an `INTERPRET` fragment runs in an activation whose condition
queue is separate, so a condition pending when the `INTERPRET` clause runs is offered no boundary
*inside* the fragment.

#### Steps

1. **Reproduce** the program above on both engines and confirm the divergence still stands at the
   tree you find. If it does not, stop and report that instead of fixing nothing.

2. **Fix it.** CORRECTED 2026-08-15, after Task 1 built the mechanism this step first named and
   measured it wrong. This step read "give the new header and `END` clauses no boundary when a
   fragment's `clause_line_override` is in force". That suppression is too broad: it regresses
   `interpret 'zq = raiser(); do; say ''body''; end'`, where the oracle **does** deliver at the
   `DO` header inside the fragment. Measured by the controller at `11638b91e`, oracle and both
   engines, `rc 0`: `h1 3` / `h2 3` / `body` / `after 5`. The distinction is not whether the
   boundary is inside a fragment, it is **which fragment queued the condition**: a condition
   pending when the `INTERPRET` clause ran waits for that clause's own boundary, and a condition
   queued inside the fragment is delivered at the fragment's own boundaries. The suppression must
   reach both engines: the compiled engine
   never calls `run_loop`, `Op::LoopRun` enters `run_loop_with_header` directly
   (`crates/rexx-exec/src/ir/drive.rs:1337`), which is why `e74780054`'s own step suppression had to
   go in `leave_stepped_clause`.

3. **Measure the suppression against the empty-fragment shape before you keep it.** Construct
   `interpret 'do; end'` under a double requeue, measure it on the oracle and on both engines
   before and after your change, and report all three. A suppression that breaks it is the wrong
   suppression.

   CORRECTED 2026-08-15: this step was written from the reviewer's statement that the shape
   "diverges *identically* before and after `e74780054`". Task 1 measured it and it does not
   diverge at all: the oracle and both engines agree, before and after. The step stands as a
   control that must keep agreeing; its stated premise does not.

4. **Measure the two siblings** the reviewer identified as pre-existing divergences of the same
   family, unchanged by `e74780054`:
   * `interpret 'if 1 = 1 then say ''body'''`
   * `interpret 'do zi = 1 to 1; say ''body''; end'`
   in the same surrounding program. Report, for each, whether your suppression closes it. If it
   does, that is in scope: it is the same mechanism, not another divergence. If it does not, record
   it as found-and-not-fixed with its transcripts and do not chase it.

5. **Witness it.** At least one `ir_dual_cases` stanza that reddens when the suppression is removed.
   Mutation-test with `--no-fail-fast` under `memcap 8G` and report every catcher. Note in the report
   which harness actually gates it: under a plain `cargo test --release --workspace`,
   `corpus_differential` is in REPORT mode.

6. **Sweep.** Raw before/after over every `.rex` under `corpus/` and `bench-programs/`, both engines,
   all three descriptors. Report exactly what moved. Nothing but your new case should.

7. **Correct the sweep sentence in the previous plan.** `docs/superpowers/plans/2026-08-14-pre-phase-5-defects.md`
   carries a Task 6 OUTCOME claim that the before/after sweep covers `corpus/` and `bench-programs/`
   and moved only one program. That sweep cannot see an `INTERPRET` shape, and this divergence is
   one the task *created* rather than encountered. Correct the sentence so it says what the sweep
   covers and what it therefore cannot witness. Do not restate a number you have not measured.

8. **Gates** as in Global Constraints, plus the corpus gate under `REXX_CORPUS_GATE=1`. Run clippy
   from a clean target directory at least once and say so.

---

