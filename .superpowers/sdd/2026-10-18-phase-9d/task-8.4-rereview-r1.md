# Phase 9d Task 8.4 re-review, fix round 1

Package `65f000af0..b3875ce42` (code base `9194fef58`; code commits `402a6c753`, `2ea743b36`; no file
under `rust/` changes after `2ea743b36`). HEAD built `--release`, `CARGO_INCREMENTAL=0`, own target
dir; `9194fef58` built the same way from a scratch worktree. Probes ran from a fresh directory per
probe and per side under `t84-rereview-scratch/`, oracle under `ulimit -v 1048576`, crate under
`memcap 2G timeout 20`; stdout, stderr and rc compared separately, the run directory normalised out
of stderr. Concurrent probes ran several times per side.

**Verdict: CHANGES REQUESTED** (1 Critical, 2 Important, 0 Minor). Every new problem is in the M4 /
queue candidate 2 fix: the re-runnable `::CONSTANT` method.

## Finding verdicts

**I1 (`::CONSTANT` level `traceline`): ADDRESSED.** `k4.rex` and `tb1.rex` (the review's shapes)
are identical to the oracle on all three descriptors; `tl1` (`.Package~new` source) and `tl2`
(`.Routine~new` source) now echo `3 *-* ::constant k (.c~f)` / `2 *-* ::class c` as the oracle does
and differ only by the oracle's extra level, which is the report's concern 3 (queued into 8.7):
```
tl1 oracle only:  METHOD The NIL object
                             *-* Compiled method "NEW" with scope "Package".
```
`tb2` (untrapped error in a method the expression calls) identical. The re-run level's frames echo
the directive too (`tb3.rex`, apart from I2 below). Witness `corpus/lang/constant_expression_traceline.rex`:
identical to the oracle 3 of 3 runs at HEAD, stdout differs at `9194fef58`.

**M1 (`parse-errors.tsv` header): ADDRESSED.** `derive.sh` re-derives the file from the oracle,
rc 0, `diff` against the committed file empty. The false "must ACCEPT" and "decision unchanged"
sentences are gone.

**M2 (Deviation candidate B, witness, comment): ADDRESSED.** The block has title, rule, measurement,
`dire.xml:535`-`:536` (checked: "A ::CONSTANT directive defines a method that returns a constant
value.") and "Witnessed by". `tests::a_constant_expressions_method_answers_its_own_clause_as_source`
passes at HEAD. The `directive_clause_lines` comment states the ruled answer.

**M3 (records for the two statements): ADDRESSED.** `r1/candidate-a-oracle-20.txt` holds the 20-run
count and the report sentence reads it. Re-measured here: `r1.rex` oracle 10 of 10 `line 3 line 3`,
crate 10 of 10 `line 8 line 3`. `r1/debug-witnesses.txt` exists and names each witness.

**M4 + queue candidate 2 (method runnable, guard): PARTIAL.** What now matches the oracle:
`define`d and sent as an expression (91.999 `Message "ZZ" did not return a result.`) and as an
instruction (`RESULT` unset) (`m1.rex`, `m3.rex`, the corpus witness 3 of 3, red at `9194fef58`
with rc 120); `define` into an unrelated class (`m3`); `SUPER`/`SELF` in a re-run (`s1.rex`);
`.object~new~run(e)` from outside is 97.2; the install-time deadlock `g2.rex` is 98.905 rc 158 (7 of
7 runs, oracle 7 of 7); an uncontended re-run holds the receiver's guard against another activity
(`g12.rex`, 3 of 3 runs identical); same-activity reentrance (`g1`, `g3`, `g5`, `g13`) identical;
memory of the minted identities is bounded (below). What does not: `run`, `enhanced` and
`setMethod` panic (C1), a contended guard is not waited for (I1), and the re-run level has the wrong
name (I2).

**Deviation 67 HALT witness: ADDRESSED.**
`run::tests::conditions::a_started_messages_halt_report_names_the_line_that_raised` passes; `r2.rex`
crate 10 of 10 `line 9 line 2` with stderr equal to the test's expected text and rc 252; oracle 10
of 10 `line 2 line 2`, rc 252.

## New problems in the fix diff

### Critical

**C1. Running the `::CONSTANT` expression method under a scope that is not a class panics.**
`run_constant_expression` passes `resolution.scope` as the scope, and `eval_constant_expression`
computes `SUPER` with `self.classes().class_super_scope(class, class)` (`install.rs:2748`), which
indexes the class graph with the scope. `self~run(m)`, `.object~enhanced(...)` and `setMethod`
install under a scope that is not a class, so the lookup panics:
```
p1.rex:  o = .c~new; o~go; say 'done' / ::class c / ::constant e (.context~executable)
         ::method go / self~run(.c~e) / say 'run instr' var('result')
oracle   rc 0: "run instr 0" "done"
head     rc 101: thread 'rexx-interp' panicked at crates/rexx-classes/src/class_graph.rs:593:32:
         no entry found for key   (backtrace: class_super_scope <- eval_constant_expression
         <- run_constant_expression <- run_other <- ... <- native_run)
9194fef58 rc 120: rexx-exec: a Method or Routine object with no executable record
```
Same panic for `say self~run(.c~e)` (`p2.rex`, oracle 91.999 `Message "RUN" did not return a
result.` rc 165), `p = .object~enhanced(.directory~of(('ZZ', .c~e))); p~zz` (`p3.rex`, oracle runs
it, then 91.999 `Message "ZZ"` for `say p~zz`) and `self~setMethod('YY', self~e); self~yy` in a class
method (`g14.rex`, oracle rc 0). The fix brief names `~run` and `enhanced`; the base answered with a
loud refusal, HEAD crashes. `SUPER` should come from the receiver's behaviour and the scope the way
an ordinary method level computes it (`Interp::begin_method`), which also answers `.nil`-scoped
methods; the corpus witness exercises `run` only from outside, where 97.2 stops before the level.

### Important

**I1. A re-run takes the receiver's guard only when it is free; contended, it runs without it.**
`eval_constant_expression` calls `reserve_for_method` (`install.rs:2737`). When the guard is held
by another activity, `reserve_for_method` only queues a `GuardWait` for this activation, which
`serve_guard_wait` serves at a clause visit in that same activation. The `::CONSTANT` level runs no
clauses, so the wait is never served; the expression evaluates, and `guard_off_at_end` discards the
wait. The method reports `isGuarded` 1 and the oracle waits:
```
g4.rex:  .c~define('ZZ', .c~e); o = .c~new; m = o~start('hold'); call syssleep 0.3
         o~zz; say 'zz returned'; m~result
         ::constant e (.c~note(.context~executable))   (note says 'note ran', returns its arg)
         ::method hold: say 'hold start'; call syssleep 1; say 'hold end'
oracle:  note ran / hold start / hold end / note ran / zz returned
head:    note ran / hold start / note ran / zz returned / hold end
g11.rex  (guarded instance method go does self~start('zz'), sleeps 0.3, says 'go still holds'):
oracle:  note ran / go still holds / go done / note ran
head:    note ran / note ran / go still holds / go done
```
Both shapes are new with the fix (`9194fef58`: rc 120 refusal). The install-time level has the same
code path, but its receiver is the class being installed, which no other activity can hold, so only
the re-run is reachable. `tests::a_constant_expression_holds_its_classs_guard` covers only the
deadlock, the one shape that never waits. This is the report's concern 4: contradicted by running.

**I2. The re-run level is named `::CONSTANT`, not the message it was sent as.**
`eval_constant_expression` always names its activation `::CONSTANT` (`install.rs:2700`); the oracle
names a re-run level by its message, as for any method:
```
tb3.rex  (re-run of .c~define('ZZ', .c~e), condition-free .context~stackframes in a method it calls)
oracle:  METHOD ZZ 6            head:  METHOD ::CONSTANT 6
m4.rex   (::OPTIONS TRACE R, o~zz)
oracle:  >I> Method "ZZ" with scope "C" ...   <I< Method "ZZ" with scope "C" ...
head:    >I> Method "::CONSTANT" with scope "C" ...   <I< Method "::CONSTANT" with scope "C" ...
```
Silent wrong answers on the newly runnable shape; everything else in `tb3` and `m4` matches.

## Implementer concerns

1. **g2's deadlock reports (ruled inside Deviation 67).** The started report fits 67 (it names line
   8, the started method's clause). The waiter's report does not have 67's shape: it names line 3,
   the `::CONSTANT` clause, and carries the constant's and the class's echoes, where 67's crate answer
   for a waiter is its own `result` clause (line 6, `return m~result`). The plain-method control
   shows 67's shape:
   ```
   d1.rex (same deadlock, a guarded class method in place of the constant):
     head waiter: "9 *-* say 'g ran'" / "Error 98 running d1.rex line 7"   (7 = return m~result)
   g2.rex head waiter: "8 *-* say 'g ran'" / "3 *-* ::constant k (.c~u)" / "2 *-* ::class c" /
                       "Error 98 running g2.rex line 3"
   g9.rex (no guard: started method does 1/0, the constant's method waits on ~result):
     head waiter: "Error 42 running g9.rex line 4" (the ::CONSTANT clause; waiter is line 7)
     oracle 'line 7' twice; g10.rex, the same without the constant: head 'line 8' then 'line 6'
   ```
   The cause is the install-time error path (`install.rs:1094`-`:1106`: `blame_directive` and the
   `::CONSTANT` site frame applied to any failure leaving the expression), and `g9` is identical at
   `9194fef58`, so it predates the fix and is not a fix-round blocker. It does contradict the ruling's
   premise "same mechanism": the waiter's line here comes from the constant's error path, not from
   67's re-raise. Queue candidate, or the one question to Moritz the ruling foresees.
2. **Oracle re-run evaluates another constant's expression (ruled inside the shared-source bug).**
   Consistent with that bug, with one correction for the Deviation 69 text: the oracle re-runs
   every `::CONSTANT` expression of the class in order, not one other. `run3.rex` (constants `a`,
   `e`, `z`, `e` the re-run): oracle prints `note a / keep / note z` at install and again for the
   re-run; head prints `keep` for the re-run.
3. **METHOD NEW level (8.7).** Observed as the report says (`tl1`, `tl2`).
4. **Guard on the re-run receiver.** See I1: taken uncontended (`g12`), not waited for contended.

## Perf

`callgrind-r1.txt` is the run of `402a6c753` (`pingmsg` +0.0734% libc subtracted, over the
+0.05% limit); it is superseded by `callgrind-r1final.txt`, which measures
`rexx-head-2ea743b36` against `rexx-base-f0164456d`, every program, exit 0: largest libc-subtracted
rises startup +0.0263%, pingmsg +0.0252%, sayloop +0.0140%, all under +0.05%. `2ea743b36` is the
last code commit, so HEAD's code is measured. The C1/I1/I2 fixes touch `eval_constant_expression`
only, which no benchmark program reaches; the install path runs it per `::CONSTANT` expression.

## Memory

Peak RSS (`/usr/bin/time -f %M`), head:
```
mem1 .Package~new with two ::CONSTANTs (one expression method minted each), 10000 / 100000:  48104 / 48288 KB
mem2 o~zz re-run, 100000 / 1000000 / 3000000:  35336 / 34616 / 34992 KB
mem3 control, an ordinary defined method:      19848 / 19468 / 20428 KB
```
No growth; the minted identity is freed with its object (`prune_dead_object_rows` ->
`forget_method_identity`).

## Checks run

`cargo test -j 4 --release -p rexx-exec --lib`: 1182 passed, rc 0 (the fix round's new `tests::` tests and
the `a_started_messages_*` tests among them). `derive.sh` as above.

## Out of scope

Nothing beyond concern 1's queue candidate.

## New oracle defects

None beyond candidate B's, whose description should read "every expression of the class" (concern 2).

Probe sources and outputs were in `t84-rereview-scratch/` (deleted at the end of the review; the
deciding lines are quoted above).
