# Phase 9d Task 8.4 re-review, fix round 2

Package `761fa71f9..10fe190cb` (code commit `d2e226538` on `b3875ce42`; `git diff --stat d2e226538 HEAD
-- rust` is empty). HEAD built `--release`, `CARGO_INCREMENTAL=0`, target dir
`/tmp/claude-1000/t84-rr2-target`; the fix base `b3875ce42` built the same way from a `git archive` of
`rust/` and `interpreter/` with its own target dir. Probes ran from a fresh directory per probe, per
side and per run under `t84-rereview2-scratch/`, oracle under `ulimit -v 1048576`, crate under `memcap
2G timeout 20`; stdout, stderr and rc compared separately with the run directory normalised out of
both streams. Concurrent probes ran several times per side.

**Verdict: CHANGES REQUESTED** (0 Critical, 1 Important, 2 Minor). The Important finding predates
this round (`b3875ce42` answers the same) but lies in the task's own re-runnable `::CONSTANT` method
and in the position rule item 4 rules; round 1's re-review did not probe a failing re-run.

## Fix brief items

**1. C1 (`run`, `enhanced`, `setMethod` panic): ADDRESSED.** `p1`, `p2`, `p3`, `g14` and the corpus
witness `constant_expression_method_scopes.rex` are identical to the oracle on stdout, stderr and rc
(rc 0, 165, 165, 0, 0). The scope hunt `s1.rex` (below) is identical too: a run's `SELF` and `SUPER`
under `setMethod` with `FLOAT`, `OBJECT` and the default scope, `define` into the class, its superclass
and its mixin, `Class~enhanced`, a class-side `setMethod` and `self~run` from a subclass instance:
```
s1 oracle = head, rc 0:
Class / The C class super The Class class          (install)
SUB / a SUB super The SUB class                    (setMethod FLOAT)
SUB / a SUB super The BASE class                   (setMethod OBJECT)
SUB / a SUB super The SUB class                    (setMethod, default scope)
SUB / a SUB super The BASE class                   (.sub~define)
SUB / a SUB super The Object class                 (.mix~define)
SUB / a SUB super The MIX class                    (.base~define)
SUB / enhanced SUB super The SUB class             (.sub~enhanced)
Class / The SUB class super The Class class        (class-side setMethod)
SUB / a SUB super The SUB class                    (self~run)
```

**2. I1 (contended guard not waited for): ADDRESSED.** `g4` and `g11`, 5 runs each side, identical every
run (`g11`: `note ran / go still holds / go done / note ran`).
`tests::a_constant_expressions_method_waits_for_a_held_guard` passed 5 of 5 filtered runs, and the
whole `rexx-exec --lib` run is 1184 passed, rc 0. No hang or deadlock the oracle does not have:
`q1` (guarded `go` sends `self~zz` and `self~run(.c~e)` on the same activity), `q2` (`o~start('zz')`,
`m~wait`), `q3` (a run whose expression sends another constant-expression method to the same receiver,
the constants in separate classes) and `q4` (an install-time constant through `.Package~new` whose
expression calls a guarded class method another activity holds, 3 runs) are identical on stdout, and
on stderr and rc where no finding below names them.

**3. I2 (re-run level named `::CONSTANT`): ADDRESSED.** `tb3` (`METHOD ZZ 4`) and `m4` (`>I> Method
"ZZ" with scope "C"`) identical; corpus witness `constant_expression_method_trace.rex` identical.
`q5` / `q5c`: `.context~name` answers `::CONSTANT` at install and `ZZ` on the run on both sides.
Install-time `condition~stackframes` (`n1.rex`, trapped `.Package~new`) is identical, `METHOD
::CONSTANT 2` included.

**4. Position rule (concern 1): ADDRESSED for the install-time level it names.** rexxref checked:
POSITION is "The line number in source code at which a condition was raised" (`condtra.xml:769`-`:771`),
"A ::CONSTANT directive defines a method" (`dire.xml:535`). Measured, 5 runs per side:
```
g2   oracle 5/5 "line 6 line 6"   head 5/5 "line 8 line 6"   rc 158 both
g9   oracle 4/5 "line 7 line 7", 1/5 "line 9 line 7"   head 5/5 "line 9 line 7"   rc 214 both
g10  oracle 5/5 "line 6 line 6"   head 5/5 "line 8 line 6"   rc 214 both
```
Each residual is the started activity's report naming its own line, Deviation 67's race: the oracle's
own fifth `g9` run took the crate's answer byte for byte. `k2`, `k3`, `t_err` identical on all
descriptors. The shapes the rule reaches beyond the brief's probes match the oracle except where
Deviation 67 already licenses: `k1` (the expression's own clause does `~result` on a failing started
message), `k1t` (the same through `.Package~new`), `k2r` (the same on a run), 3 runs each, the
waiter's report the oracle's every run; `c1` (SIGNAL ON in a called method, install and run), `c2`
(CALL ON NOTREADY in a called method, install and run) and `c4` (RAISE PROPAGATE through the
install-time level, untrapped) identical. A failing run does not follow the rule: new finding I1.

**5. Candidate B's measurement text: ADDRESSED.** The block reads "evaluates every `::CONSTANT`
expression of the class in order on the oracle (`r2/probes/run3.rex` ...)". Re-measured: `run3` oracle
`note a / keep / note z` at install and again for the run, head `keep` for the run. See Minor M2 for a
stale sentence elsewhere in the report.

## New problems

### Important

**I1. A run of the `::CONSTANT` method records no site for its level when it fails: the directive
clause is neither echoed nor blamed.** `run_constant_expression` (`install.rs:2792`-`:2829`) returns
`eval_constant_expression`'s failure unchanged; the install path's handling (`install.rs:1083`-`:1118`,
`blame_directive` plus `capture_site_frame` and `seal_site_level`) has no run-time counterpart. A
condition raised in the expression's own clause is reported at the sender's line, and one raised below
loses the level's echo and its `condition~stackframes` entry:
```
q8.rex:  .c~define('ZZ', .c~e) / o = .c~new / .local~div = 0 / o~zz   (line 4)
         ::class c / ::constant e (.c~keep(.context~executable, 1 / .c~d))   (line 6)
         (.c~d answers 1 at install, .local~div afterwards; keep returns arg(1))
oracle rc 214:  "     6 *-* ::constant e (.c~keep(.context~executable, 1 / .c~d))"
                "     4 *-* o~zz"   "Error 42 running q8.rex line 6: ..."
head   rc 214:  "     4 *-* o~zz"   "Error 42 running q8.rex line 4: ..."
q7.rex   (the division in a class method the expression calls, line 10)
oracle:  "10 *-* x = 1/d" / "6 *-* ::constant e (.c~boom(.context~executable))" / "4 *-* o~zz" / line 10
head:    "10 *-* x = 1/d" / "4 *-* o~zz" / line 10
q6.rex   (q7 trapped by SIGNAL ON SYNTAX in main; position, stackframes, traceback)
oracle:  42.3 17 / METHOD BOOM 17 / METHOD ZZ 13 / PROGRAM q6.rex 5 / 17 *-* ... / 13 *-* ::constant ... / 5 *-* o~zz
head:    42.3 17 / METHOD BOOM 17 / PROGRAM q6.rex 5 / 17 *-* ... / 5 *-* o~zz
c5.rex   (RAISE PROPAGATE from the called method's SYNTAX handler, untrapped)
oracle:  ... "5 *-* ::constant e (.c~prop(.context~executable))" ... "Error 42 running c5.rex line 5"
head:    no ::constant echo, "Error 42 running c5.rex line 3"
c3.rex   (c5 trapped in main): position oracle 11 (the directive), head 4; tracebacks identical
q3.rex   (91.999 raised by the expression's own `self~yy`): oracle line 8 with the ::constant echo,
         head line 4 without it
```
The oracle's answers are the rule's: POSITION is where the condition is raised (`condtra.xml:769`),
the expression's own failure and a RAISE PROPAGATE into the level are raised in the directive's clause,
and the level is a method level that a traceback and `stackframes` list. The ::CLASS echo is
install-only on the oracle and should stay so on a run. `b3875ce42` gives the same answers for
`q3`, `q6`, `q7`, `q8`, `c3` and `c5`, so this predates the round; no corpus witness or test has a failing
run. Silent wrong positions and tracebacks on the shape this task made runnable.

### Minor

**M1. Under TRACE, a contended run traces its `>I>` after the guard wait; the oracle traces it before.**
`serve_guard_wait` (`install.rs:2750`) runs before `trace_package_invocation_entry`
(`install.rs:2761`), and an ordinary guarded method traces its entry early (control `t1c.rex`, the same
program with a plain `::method zz`, identical 2 of 2 runs):
```
t1.rex (::OPTIONS TRACE R; o~zz while a started `hold` holds o's guard), 3 of 3 runs each side:
oracle:  5 *-* o~zz / >I> Method "ZZ" ... / >>> "0" / 11 *-* say 'hold end' / ... / <I< Method "HOLD" / 8 *-* ::constant e ...
head:    5 *-* o~zz / >>> "0" / 11 *-* say 'hold end' / ... / <I< Method "HOLD" / >I> Method "ZZ" ... / 8 *-* ::constant e ...
```
The same cause drops the run's `>I>`/`<I<` pair when the wait deadlocks (`t2.rex`, `g15`'s shape under
TRACE R: the oracle prints `>I> Method "ZZ"` and `<I< Method "ZZ"` before the first report, head
neither). The rest of `t2`'s difference is the queued deadlock report shape and is not counted.

**M2. A round-1 sentence in the report still says the oracle evaluates "another" constant's
expression.** `task-8.4-report.md:212`-`:213` ("the oracle evaluates another `::CONSTANT`'s expression
when the method is run (`run2.rex` prints `note ran` again ...)") contradicts the corrected candidate B
block (`:249`-`:252`, every expression of the class, its own included).

## Perf

`r2/callgrind.txt` measures `rexx-head-d2e226538` against `rexx-base-f0164456d`, every program, exit 0.
That binary carries the fix (it runs `p1` to `done`, rc 0, and traces `m4` as `>I> Method "ZZ"`), and
no `rust/` file changes after `d2e226538`. Largest libc-subtracted rises: startup +0.0269%, pingmsg
+0.0259%, sayloop +0.0143%, churn +0.0045%; all under +0.05%. `eval_constant_expression`'s only callers
are `resolve_constants` (`install.rs:1082`, install) and `run_constant_expression`
(`install.rs:2820`), reached only from the `GeneratedKind::ConstantExpression` dispatch arm
(`dispatch.rs:2940`, an argument added); the `blame_first_clause` addition is on the `#[cold]` deadlock
path.

## Checks run

`CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/claude-1000/t84-rr2-target memcap 8G cargo test -j 4
--release -p rexx-exec --lib`: rc 0, 1184 passed. The two new `tests::` tests filtered, 5 runs: 2 passed
each run.

## Out of scope

None beyond the new oracle defect below.

## New oracle defects

**A run of a `::CONSTANT` expression's method replaces the constant's value on the oracle.** `q5d.rex`:
`m = .c~e; .c~define('ZZ', m); .c~new~zz; say (m == .c~e) (.c~e == .c~method('ZZ'))` with `::constant e
(.context~executable)`: oracle `0 1`, head `1 0`. After the run the oracle's accessor answers the run's
executable, the `define`d copy; rexxref has the directive define "a method that returns a constant
value" (`dire.xml:535`). It fits candidate B's shared-directive bug (the run re-evaluates and records
every expression of the class) and belongs in its description; the crate's answer is the rule's.

Probe sources are in `t84-rereview2-scratch/src/` until the scratch directory is removed; the deciding
lines are quoted above.
