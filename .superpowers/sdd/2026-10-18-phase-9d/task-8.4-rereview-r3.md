# Phase 9d Task 8.4 re-review, fix round 3

Package `a87a5c8fa..75066653a` (code commit `6c8644ad5` on `d2e226538`; `git diff --stat 75066653a HEAD
-- rust` is empty at HEAD `0c51373c7`). HEAD built `--release`, `CARGO_INCREMENTAL=0`, target dir
`/tmp/claude-1000/t84-rr3-target`. The previous code head is the pinned
`ooRexx-p9-pin/cg-9d/t8.4/rexx-head-d2e226538`, run as a third side on every probe; the round 2 base is
`rexx-r2base-b3875ce42`. Every probe ran from a fresh directory per side and per run under
`t84-rereview3-scratch/`, oracle under `ulimit -v 1048576`, crate under `memcap 4G timeout 20`;
stdout, stderr and rc compared separately with the run directory normalised out. Concurrent probes ran
3 times per side.

**Verdict: APPROVED** (0 Critical, 0 Important, 0 Minor).

## Fix brief items

**1. I1 (a failing run records no site for its level): ADDRESSED.** `q3`, `q6`, `q7`, `q8`, `c3`, `c5`
(`r3/probes/`) are identical to the oracle on stdout, stderr and rc (rc 165, 0, 214, 214, 0, 214), and
each differs from `d2e226538`. `q8` at head:
```
     6 *-* ::constant e (.c~keep(.context~executable, 1 / .c~d))
     4 *-* o~zz
Error 42 running q8.rex line 6:  Arithmetic overflow/underflow.
```
The code is shared, not copied: `Interp::seal_constant_level` (`install.rs:2832`-`:2856`) holds the
`Reraised` early return, `blame_directive`, `capture_site_frame` and `seal_site_level`; the install
path (`install.rs:1098`) calls it with `::CONSTANT` and no arguments and then adds the `::CLASS` echo,
the run path (`install.rs:2820`-`:2829`) with the message name and its arguments and no `::CLASS` echo.
The install path's directive argument changed from the loop's `directive` to
`&program.directives[index]`, the same element (`install.rs:1062`). Witness
`corpus/lang/constant_expression_method_fails.rex` (single-threaded; the expression's own failure, a
failure in a method it calls, and a RAISE PROPAGATE into the level, each trapped, then the first
untrapped): identical to the oracle on all three descriptors at head, and stdout and stderr both differ
at `b3875ce42`. Its `sourceline_oracle` file is in place; `rexx-parse` tests pass.

**2. M1 (`>I>` after the guard wait): ADDRESSED.** `trace_package_invocation_entry` moved before
`reserve_for_method` (`install.rs:2737`). `t1` identical to the oracle 5 of 5 runs (2 here plus 3
before the third side was added); against `d2e226538` the `>I> Method "ZZ" with scope "C" in package
"t1.rex".` line moves from after the holder's lines to before them, as on the oracle. `t1c` identical
3 of 3. `t2` (deadlock under TRACE R): head 3 of 3 one answer, oracle the same answer 1 of 3 and an
interleaving of its two reports 2 of 3; the rest is the queued deadlock report shape. The witness
`tests::a_constant_expressions_method_traces_its_entry_before_the_guard_wait` passed 5 of 5 filtered
runs.

**3. M2: ADDRESSED.** `task-8.4-report.md` round 1 sentence now reads "the oracle evaluates every
`::CONSTANT` expression of the class, in order, when the method is run", matching the candidate B
block.

**4. q5d in candidate B: ADDRESSED.** Candidate B's measurement names `r3/probes/q5d.rex` and its
answers; re-measured, oracle `0 1`, head `1 0`, 3 of 3 runs each, rc 0 both, stderr identical.

## Hunt for breakage from the shared `seal_constant_level`

Every probe below is identical to the oracle on stdout, stderr and rc unless stated. Sources in
`t84-rereview3-scratch/src/`.

* Install-time shapes unchanged: `r2/probes/` (`k2`, `k3`, `t_err` among them), the round 2
  re-review's sources, `t8.4/constant/probes/` and every `corpus/lang/constant_expression*.rex`. The
  ones that differ from the oracle differ identically at `d2e226538` (`d2`, `g2`, `g9`, `g10`, `k1`,
  `k1t`, `k2r`: Deviation 67 or the queued deadlock shape; `run3`, `q5`, `q5d`, `f3`, `f4`, `f7`, `f8`:
  candidate B; `t_r`), except two: `cf` differs only by its `~right(8)` of a random directory name, and
  `g15` (queued shape) gains the waiter's `6 *-* ::constant e (.context~executable)` echo, which the
  oracle prints for both reports; head moved toward the oracle.
* `h1`: SIGNAL ON SYNTAX in the sender method: `42.3 8 / METHOD ZZ 8 0 / METHOD GO 16 0 / PROGRAM
  h1.rex 5 0` and the three-line traceback with the directive's echo.
* `h2`: CALL ON USER in main, the run's expression calls a routine that RAISE PROPAGATEs USER into the
  level (rc 212, 44.1 "No data returned from function "PR"" with the directive echoed, both sides).
* `h3`: nested runs (a's run's expression runs b's method, which fails), trapped then untrapped:
  `METHOD ZZ 21 / METHOD STEP 18 / METHOD ZZ 16 / PROGRAM h3.rex 7`, both directive echoes in the
  traceback and the untrapped report, `Error 42 running h3.rex line 21`.
* `h4`: a failing run in `o~start('zz', 'x', 7)`, trapped at `m~result`, then `m~result` untrapped, 3
  runs: stdout identical (`42.3 7 / METHOD ZZ 17 / 17 *-* ::constant ... / 1 42.3`); stderr differs
  only by the started activity's report naming its own line (`line 17`, oracle `line 7`), Deviation
  67's licensed race. `d2e226538` printed no directive echo and no `METHOD ZZ` frame here.
* `h5`: INTERPRET, trapped then untrapped: `METHOD ZZ 16 2 / INTERPRET  6 0 / PROGRAM h5.rex 6 0`.
* `h6` under TRACE I, R, O, A, N and `h6p` under `::OPTIONS TRACE R`: a failing run with four arguments,
  one omitted; the `>A>` lines, the directive echo and `line 8`.
* `h6s`: the arguments in `stackframes`: `METHOD ZZ 14 3 4 two`.
* `h7`: EXIT in a routine the expression calls (returns, as an external routine's EXIT does).
* `h8`, `h9`: a trapped failing run (twice in a row in `h9`), then an unrelated error: no stale
  `::constant` echo or frame in the later condition (`h9`: `13 1 1`, traceback `13 *-* say 1/0`).
* `h11`: the failure trapped inside the method the expression calls, then an untrapped error in main.
* `h12`: the run two internal calls below a SIGNAL ON in an internal routine.
* `h13`: a native method's error in the run's own clause (`'abc'~left(-1)`), trapped then untrapped
  from an internal routine: `93.923 18`, `*-* Compiled method "LEFT"` above the directive echo.
* `h14`: the constant in a `::REQUIRES`d file, trapped then untrapped through `~send`: position 2 in
  `lib1.cls`, `Error 42 running lib1.cls line 2`.
* `h15`: the run reached by FORWARD and by `~sendWith`.
* `h16`: a failing run after REPLY on the new activity, 3 runs.
* `h17`: the constant in a package built by `.Package~new`: `2 pk`, `Error 42 running pk line 2`.

## Perf

`r3/callgrind.txt` measures `rexx-head-6c8644ad5` against `rexx-base-f0164456d`, every program, exit 0.
That binary carries the fix (`q8` gives the oracle's echo and `line 6`). Largest libc-subtracted
rises: startup +0.0261%, pingmsg +0.0245%, sayloop +0.0139%, churn +0.0043%, define +0.0040%; all under
+0.05%. The new code is in the failure arms of `resolve_constants` and `run_constant_expression`; the
moved trace call is inside `eval_constant_expression`, reached only by installing a `::CONSTANT`
expression and by the `GeneratedKind::ConstantExpression` dispatch arm. No hot path is touched.

## Checks run

`cargo fmt --all --check`: 0. `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/claude-1000/t84-rr3-target
memcap 8G cargo test -j 4 --release -p rexx-exec --lib`: 1185 passed, rc 0. The four
`constant_expressions_method` tests filtered, 5 runs: 4 passed each run. `cargo test --release -p
rexx-parse`: 413 passed, 0 failed.

## Out of scope

None.

## New oracle defects

None.
