# Task 8 re-review 2 -- fix round C (scoped)

Reviewer: review-s8. Range `a9fd1a7ec..f24f208ad` (`50ec3eb98`..`f24f208ad`). `$R` =
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/review-s8/`.

* **Build:** `git archive f24f208ad` into `$R/h3`, with `ootest/` copied in and every file
  touched, and its own `CARGO_TARGET_DIR=$R/tgt3` (`Compiling rexx-exec` seen).
* **Probe harness:** probes now run through `$R/cmp2.sh` and `$R/cmpd2.sh`, which give each side
  its own fresh `mktemp -d` (see N6). Everything else is as before: the standard wrapper, stdin
  `/dev/null`, three descriptors compared separately, temporary paths masked.
* **Binaries compared:** `a9fd1a7ec` is fix-s8b's `rexx-run-FINAL`; `391242b7e` is surface-8's
  `base-target`.

## Verdict

**Approved, with three Minor findings and three residuals for the lead to place.**

* **Round C is correct as measured.** N5, N1 to N4, the three I2-shaped residuals and `~empty` all
  agree with the oracle.
* **N6 was my mistake,** not a defect.
* **The new general semantics held under my mutation hunt,** except one untrapped shape (M7).
* **Performance:** rexxcps is flat, every trap loop is within +10% of `391242b7e`, and I judge the
  expose loop's +9.0% acceptable.
* **Gate re-run in my own build:** the instrument is green (51.53 s) and the corpus reads
  `652 of 652 matching` (`$R/r3-base.out`).

## Open findings

No Critical or Important finding.

* **M7 (Minor).** A `RAISE PROPAGATE` issued inside INTERPRET in a routine's handler, then
  untrapped (`$R/p3/g3.rex`):
  * **Traceback:** the crate adds `6 *-* interpret 'raise propagate'`, and its report says
    `line 6`; the oracle has no such line and reports `line 1`, the caller's.
  * **Before round C:** at `a9fd1a7ec` the extra line was already there and the report had no
    `running ... line` at all.
  * **Trapped** (`g6`) and **from an internal call** (`g7`): both agree.
* **M8 (Minor, pre-existing).** A SYNTAX error raised through an INTERPRET whose fragment contains
  a DO loop, then trapped (`$R/p3/l6.rex`): the traceback's indentation lands on the other line.
  * **Oracle:** `2 *-*       call bad;` (indented) and `2 *-* interpret '...'`.
  * **Crate:** the fragment line flush and the `interpret` line indented six.
  * **Also affected:** the live-frame tracelines, in `l2.rex`.
  * **Not new:** the same at `a9fd1a7ec`. Untrapped (`l5`) agrees.
  * **Not recorded:** the entries I could find are about a loop header's indent, a different
    line.
* **M9 (Minor, withdrawn N6).** Round C is right: my `cmp.sh` ran both sides in one directory,
  and `sr4.rex` appends to `in.txt`, so the crate read both runs' lines.
  * **Per-side directories:** `sr4` is identical at `f24f208ad` and at `391242b7e` alike.
  * **One shared directory:** the old doubling reappears (`$R/p/sr4.rex.*`).
  * **Other probes:** none of my earlier probes but `sr3`/`sr4` write files.
  * **Harness fix:** `cmp2.sh`/`cmpd2.sh` now use one directory per side.

## 1. The findings re-run

All at `f24f208ad` through `cmp2.sh` (`$R/p/`) and `cmpd2.sh` (`$R/q/`):

| Finding | Probes | Result |
|---|---|---|
| N5 reader ends | `sq1`-`sq7`, `sr1`, `sr2`, `sr3` | identical (`sq1`, `sq3`, `sq6` rc 0 where they hung; `sq7` rc 0) |
| N1 external program level | `q/d2`, `q/d3`, `q/d4` | identical (also `d3`'s untrapped caller line, the pre-existing `exclusions:5137` shape) |
| N2 PROPAGATED | `pr1`, `q/d6`, `q/d7` | identical |
| N3 RAISE SYNTAX in a callback | `nr1` | identical |
| N4 CallRoutine | `nr1`, `nr2`, `nr4` | identical |
| N6 | `sr4` | a harness artefact (M9) |
| residual 1, INTERPRET frame | `q/r1` | identical |
| residual 2, program as routine | `q/r2` | identical |
| residual 3, directive install | `q/r3` | identical |
| `~empty` under an exposed tail | `q/r4` | identical |

The earlier rounds' probes all stay identical (`co*`, `gr*`, `rec1`-`rec4`, `sf*`, `tb*`, `rs*`,
`ex*`, `cn*`, `pk2`, `pk4`, `op*`, `ms1`, `ra*`, `pm1`-`pm3`, `q/d1`, `d5`), with two exceptions,
neither new:

* `cn3` differs only because its `~right(8)` takes in part of the temporary path, which the
  masking cannot reach.
* `pk1` is the pre-existing, recorded parent-package class lookup, NO OWNER.

## 2. Mutation hunt over the new semantics

Probes in `$R/p3/` and `$R/q/e1`-`e4`. Every one below is identical on three descriptors unless
named.

* **Tail-less and EXIT RAISE offered to the caller** (`a1`-`a8`):
  * **Raise sites:** a routine, a method's internal call, nested INTERPRET in a routine and in
    main, an internal-call chain, and a main-level internal call nobody traps (silent end, rc 0).
  * **Call styles:** function versus CALL, `EXIT 'value'` reaching the caller's variable, and the
    44.1 when a function answers nothing.
  * **Traps and conditions:** CALL ON versus SIGNAL ON, the method's own CALL ON passed over,
    ERROR/FAILURE/NOTREADY/NOVALUE, and `condition('I')` and `sigl`.
  * **Trace:** `trace r`'s ordering of the returned value and the handler.
* **Untrapped HALT and NOMETHOD as SYNTAX** (`h1`-`h16`):
  * HALT with and without DESCRIPTION.
  * `SIGNAL ON SYNTAX` and `CALL ON HALT`, a routine's HALT with RETURN trapped by main, and a
    method's own trap.
  * NOMETHOD with ADDITIONAL, DESCRIPTION, both, SIGNAL ON NOMETHOD.
  * `trace i`, SIGNAL ON HALT from an internal call, CALL ON ANY, and SIGNAL ON ANY over a
    routine.
* **RAISE PROPAGATE** (`g1`, `g2`, `g4`-`g7`, `pe*`, `tr2`):
  * From a routine's, a method's and a CALL ON handler.
  * A USER condition propagated to SIGNAL ON ANY, a method propagating through another method,
    and the main program's own propagate.
  * The traced run, from INTERPRET trapped, and from an internal call in a handler.
  * Only the untrapped INTERPRET form differs (M7).
* **A program called as a routine** (`q/e1`-`e4`):
  * PARSE SOURCE's call type, the frame type, `.context~name`, and `EXIT value` into `RESULT`.
  * A `::REQUIRES`d routine's `>I>`/`<I<` naming its own package under `trace r`, and a called
    program's untraced run under the caller's `trace i`.
  * A CALL ON handler for a raise from the called program, and `trace r` in a method calling one.
  * Recursion through an external program with the trapped frames, and a called program with a
    `::ROUTINE`.
* **Live native and INTERPRET frames** (`l1`-`l5`, `tr1`, `tr3`):
  * `.context~stackframes` from a method reached by `TestSendMessage0`, and nested INTERPRET in a
    routine.
  * A trapped error through a nested INTERPRET inside a DO.
  * `trace i` over nested INTERPRET.
  * Two differences:
    * `l1`: the CALL ON handler never runs for `raise ... return` inside INTERPRET in an internal
      call, which is round C's listed `ri1`.
    * `l2`: the tracelines' indentation, M8.
* **EXPOSE weak cell, clear and detach** (`w1`-`w3`):
  * Two methods exposing tails, emptied twice with a 20000-array churn between.
  * 300 short-lived exposers collected before an `~empty`, and a tail-less `PUT` (`s.[] = v`)
    under a live exposure.
  * `drop s.` in the object, then `~empty` of the new stem.

## 3. Performance

Callgrind `summary`, `$R/cg2/run.sh`, fresh directory per run. The `391242b7e` and `a9fd1a7ec`
columns are from the previous re-review, except where marked (`$R/cg2/*.line` and profiles).

| program | 391242b7e | a9fd1a7ec | f24f208ad | against 391242b7e | against a9fd1a7ec |
|---|---|---|---|---|---|
| `t.rex` | 629,364,129 | 550,497,395 | 552,006,340 | -12.29% | +0.27% |
| `tu.rex` | 1,232,651,879 | 1,176,832,181 | 1,180,242,225 | -4.25% | +0.29% |
| `tm.rex` | 642,961,614 | 682,846,832 | 684,643,637 | +6.48% | +0.26% |
| `td.rex` | 5,364,233,244 | 5,446,879,828 | 5,488,127,372 | +2.31% | +0.76% |
| `rexxcps.rex` | -- | 19,450,620,453 (this run) | 19,467,336,777 | -- | +0.09% |
| `em3.rex` (round C's) | -- | 2,570,850,571 (this run) | 2,802,258,964 | -- | +9.00% |

* **The report's table reproduces:** every figure is within 0.1 point.
* **rexxcps is flat.** Its +0.09% is inside the band this project measured for layout alone.
* **Every trap loop is within +10% of `391242b7e`.** The largest is the one-method loop, at
  +6.48%.
* **em3's +9.0% is acceptable, in my judgement.**
  * **What the loop measures:** round C's own microbenchmark, 200000 method sends that do nothing
    but `EXPOSE` a tail, with an `~empty` every 1000.
  * **What the cost is:** one weak-reference cell per EXPOSE of a single tail. EXPOSE of a whole
    stem or a simple variable pays nothing, and no bench axis and no suite test is shaped like
    this loop.
  * **What it buys:** the oracle's detach semantics, which I found no cheaper representation
    for. Before round B this EXPOSE refused loudly, so no program that ran before pays it.
  * **One condition:** if Task 9's gate adds a stem-heavy OO axis, this loop's shape should be on
    it.

## 4. Round C's three unrecorded pre-existing divergences

Each run through `cmp2.sh` at `f24f208ad` and at `391242b7e`. None changes any test's outcome,
assertion count or detail in the three groups (the instrument is green at `-V 2`).

* **`ri1`: `RAISE ... RETURN` inside INTERPRET goes to the wrong activation.** It is offered to
  the caller of the activation running the fragment rather than to that activation.
  * **Real:** the oracle prints `rr2 handler`; the crate does not.
  * **Pre-existing:** the same at `391242b7e`. `p3/l1` is a second instance, an internal call's
    `interpret 'raise user u return'` whose main-level CALL ON handler never runs.
  * **Plausible owner:** Phase 9. It is core condition semantics, reached by no API group.
* **`if1`: `Routine~call` from Rexx lacks its own level.** The oracle lists a `METHOD [CALL]`
  level and names the routine's frame `ROUTINE [rr]`; the crate has `ROUTINE [CALL]` and no
  METHOD level.
  * **Real and pre-existing** (same at `391242b7e`). Its INTERPRET half is now fixed.
  * **More generally:** no live frame is listed for a built-in method's level.
  * **Plausible owner:** Phase 9, as `.context~stackframes` conformance.
  * **Why not Phase 8:** the native API's CallRoutine, which is Phase 8's, now agrees (N4).
* **The empty PROGRAM frame name.** Round C left this unexplained; I reproduced it and
  narrowed it.
  * **Trigger:** once a SYNTAX error raised inside INTERPRET has been trapped by `SIGNAL ON`, the
    oracle names the main program's frame with the null string for the rest of the run
    (`$R/p3/pf1.rex`: `.context~stackframes[1]~name` answers `[]`, as do later conditions'
    PROGRAM frames and a method's view of the stack).
  * **Not triggered** by a SIGNAL out of INTERPRET with no error (`pf2`), nor by any earlier
    trap in the witness (`pb1`-`pb6`: only removing the INTERPRET trap `s6` removes it).
  * **Pre-existing:** the same at `391242b7e`.
  * **What it looks like:** state the oracle's trap leaves behind rather than a rule. The name
    does not come back.
  * **Plausible home:** a candidate for the "upstream accident" licence the exclusions already use
    for identity inconsistencies, otherwise Phase 9. Recording it with this narrowing is enough
    for now.

## Housekeeping

`$R/tgt3` was deleted after the runs. `$R/h3`, the run logs and all probes stay in `$R`.
Nothing in the repository was edited.
