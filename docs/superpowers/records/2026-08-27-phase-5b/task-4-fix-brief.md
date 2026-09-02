# Task 4 fix round -- brief

`task-4-review.md`: 3 Critical (two silent wrong answers), 2 Important, 2 Minor, 1 observation. BASE
is the commit named in your dispatch. Every arm the review claims reproduces; the failure is coverage
past the rows, not a control that lied. **Read the C++ yourself; the review's citations are claims.**

## CRIT-2 first: a trap that should not fire. Verified by the controller with its own probes

Loud face -- `signal on syntax` in a method that then does a non-continuing
`forward to (d) message('FAIL')` over a `1/0`:

```
oracle           rc 214   stdout empty        stderr 321 bytes, three-line traceback
ir, tree-walker  rc 0     stdout a INNER-handler   stderr empty
```

**Silent face**, with a trap in the caller as well -- both sides rc 0, both stderr empty, one word of
stdout apart:

```
oracle           a OUTER-handler
ir, tree-walker  a INNER-handler
```

The oracle unwinds **past** the forwarding method; this crate traps inside it. The mechanism is
explicit in the C++ and the review cites it: `settings.setForwarded(true)` before the send
(`RexxActivation.cpp` around `:1372`), and `RexxActivation::trap` reads that flag first and drills to
the previous non-forwarded frame. A forwarded activation is a phantom for condition delivery too, not
only for its result. Four adjacent arms bound it to exactly that send -- the same trap with
`CONTINUE`, the same failure through `DELEGATE`, a FORWARD option's own `98.946`/`88.914` under a
trap, and the caller-only trap -- so do not widen the fix past the non-continuing case.

## CRIT-3: `FORWARD ARGUMENTS` over a stem, silent

```
s.0=3; s.1='p'; s.2='q'; s.3='r' ; forward message('SEEN') arguments (s.)
  oracle          rc 0  a seen 4        (controller's own probe)
  ir, tree-walker rc 0  a seen 1
```

`a. = 'dflt'` then `arguments (a.)` is oracle `seen 0` against crate `seen 1`. Cause per the review:
`forward_arguments` reuses `operator_operand_gap` (`eval.rs:1635`), which answers "what no operator
can take" rather than "what `requestArray` cannot answer"; the stem arm is where those two differ and
nothing refuses to make it visible. **A witness must compare `arg()` and not the items** -- the
oracle's items are stem tails in hash order, which
[D61: no check may depend on an order the oracle does not reproduce] forbids relying on.

## CRIT-1: `FORWARD CLASS (x)` skips the scope-override validation

`forward class (.Other) message('M')` in `K subclass Base` where `.Other` is not an ancestor: oracle
rc 163 `Error 93.957: Target object "a K" is not a subclass of the message override scope`, crate
rc 159 `Error 97.1 ... does not understand message "M"`. **This task's, not pre-existing**: the same
bad scope through `o~m:.Other` is byte-identical on all three sides. `Interp::message_term` checks
`class_id().is_none()` **and** `receiver_has_scope`; `Interp::exec_forward` checks only the first.
The C++ keeps both inside `RexxObject::messageSend`'s override arm, whose comment says so. Valid
ancestors are unaffected, which is why `forward_class_super.rex` cannot see it.

## IMP-1, a record defect: the `98.937` witness has no recorded control

`acb015bd4`'s row is absent from the report's arm table, which stops at `d8e48d353`'s eleven. The
review ran the two missing arms and both behave: deleting the check gives 310 of 311 with
`forward_after_reply.rex` alone and `stderr differ`; keying it to the `REPLY` keyword instead of the
replied value gives the same, so the file's `bare` arm does separate them. **Record them; do not
re-derive them as new work.**

## IMP-2: six loud divergences on paths this task built, none on any list

`arguments (self)` and `arguments (.String)` are oracle `98.946` at rc 158 against crate rc 120 --
and `98.946` is the error this function *already* raises for `.nil`, so the answer exists and the
route to it does not. `arguments (a .StringTable)` is oracle rc 0 against rc 120. Two more sit behind
`Array~new`/`List~new`. And `::method m delegate a.b` (or `delegate a.`) is oracle rc 0 against
rc 120, whose text names "a generated accessor for the attribute" and no longer describes the site.
Close what is cheap, and put the rest on **Task 9's list** with measurements.

## OBS-1: take this seriously, it questions a controller ruling

Under arm M6 (MESSAGE discarded), `forward_after_reply.rex` does **not terminate** -- rc 137 after
`timeout -s KILL 60`, both descriptors empty, where the unmutated build is well under a second. Under
M6 that program's `bare` method becomes a self-forward reached after a `reply`.

The licensed divergence says this crate answers the self-forward shape with a clean rc 245. **That
licence is the controller's and this is evidence a member of the shape may hang instead**, which
would make the licence wrong rather than merely narrow. The reviewer did not construct the unmutated
forbidden program and neither may you: **never construct or run a `FORWARD` without `CONTINUE` that
resolves back to its own method**, and never hand one to the oracle.

What to do instead: establish, on the **crate alone**, that the refusal terminates for the *family*
and not just for the one probe already measured. Reach the shape by routes that are not the forbidden
literal -- a `reply` before it, a delegate chain, an option that renames the message -- and if any
member hangs, say so plainly. If the licence needs narrowing or a bound, say what it should read.

## Minors

`trace i` writes `>K> "CLASS" => "5"` for an invalid `FORWARD CLASS` that the oracle does not, because
`ForwardInstruction.cpp` raises `88.914` between evaluating and tracing; the good-path transcript
agrees byte for byte. The plan at `:426`-`:427` still says the spec is the controller's to correct --
that was done at `1aa54204b` and the plan should now say so. The `::ATTRIBUTE ... DELEGATE` getter is
never sent by its gate row, only by a `dispatch.rs` unit test. Four comments state a set size.

## Standing constraints

All of `global-constraints.md`. Oracle probes from a fresh empty directory; three descriptors read
separately; both engines; nothing from `rust/corpus/oracle-crashes.txt`; no `unsafe`. Mutation arms in
`git archive` extracts with their own `CARGO_TARGET_DIR`, and **`touch` the sources after a `cp -a`
restore** or cargo rebuilds nothing and you measure the previous arm. Compare failing **sets**, not
counts. Five gates from `rust/`, each status read unpiped from its own file, never chained, plus the
phase-gate command.

**A sitting is owed and currently impossible:** this machine's PMU will not schedule `cycles:u` and
`instructions:u` together, confirmed repeatedly by the controller at low load with no competing
`perf`. Task 4 already owes one for `acb015bd4`. Do not attempt a sitting; record what your change
would need one for, and the controller will take both when the counters recover.

Report to `task-4-fix-report.md`, file first, append as you go. Message the controller when you finish.
