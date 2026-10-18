# Task 6.3 review: numeric conditions

Reviewed `4f62b0d43..9095627aa` (crate, witnesses, `phase-9.txt`) and the report at `64823a51a`.
The crate was built from the worktree at `64823a51a`. `git diff --stat 9095627aa 64823a51a -- rust/`
shows only `rust/corpus/suite-9d-set.txt` changing. Every probe ran against the oracle under the
standard wrapper from a fresh empty run directory, with stdout, stderr and rc compared separately.
The probes are in
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/rev63/p/`.

## Spec compliance

Issues found. One LOSTDIGITS raise site of the cluster is missing.

* The cluster `lostdigits-not-raised` is "`LOSTDIGITS` is never raised for an operand wider than
  `NUMERIC DIGITS`" (`clusters.md:51`). The oracle has a fourth raise site the report does not
  list: `NumberString::maxMin` (`interpreter/classes/NumberStringMath.cpp:277`) runs every argument
  after the first through `prepareOperatorNumber`, which reports LOSTDIGITS. The crate does not
  raise there, under a trap or under `::OPTIONS LOSTDIGITS SYNTAX`. See finding I1.
* Cannot verify from the diff alone: none. The two cluster tests left failing are owned by Task 6.9,
  as the dispatch says, and are not a finding here.

## Strengths

* `lostdigits_raise_with` (`run/condition.rs:279`) keeps the oracle's observable order. A `SIGNAL
  ON` trap disables the option through `disable_for`, so checking the trap before the option is
  equivalent to `Activity::raiseCondition`. Probe `opt1.rex` and `opt2.rex` confirm this for `SIGNAL
  ON` followed by `SIGNAL OFF`, for `CALL ON ANY` with the option, and for `SIGNAL ON ANY` with
  the option.
* Arming only on `SIGNAL ON` matches `TrapHandler::canHandle`. `CALL ON LOSTDIGITS` is 25.1 on
  both sides (`callon.rex`).
* DATATYPE `W` now has the oracle's shape (add zero at the running DIGITS, then `is_integer`), and
  the error propagates from both callers (`builtin/datatype.rs:215`, `dispatch/string.rs:1593`).
* The witnesses are dense and each path prints. They cover the as-written description with blanks,
  `CALL ON ANY` not taking the raise, `SIGNAL ON ANY`, `SIGNAL OFF`, a `DO` header and SmallInt
  operands. A fix that arms on `CALL ON ANY`, normalises the description, or skips `ANY` turns them
  red.

## Probes run (all identical to the oracle unless stated)

* `ops.rex`: every comparison (`<>`, `>=`, `\=`, `><`, `>`, `<=`), strict comparisons (silent),
  `**`, `/`, `*`, prefix `-()`, a function result on either side, message forms `~"+"`, decimal
  and exponent operands, and `1E+1000000+1`. `abs`, `sign`, `trunc`, `format`, `~abs` and the
  first argument of `max` stay silent on both sides.
* `loops.rex`: `DO` with a wide `BY` and `TO`, the increment crossing DIGITS, stem, `PARSE VALUE`,
  `.local`, Array item, `"..."~"="`. It differs only on the repetition counts in M2.
* `opt1.rex`, `opt2.rex`: `::OPTIONS LOSTDIGITS SYNTAX` with `SIGNAL ON`/`OFF`, `CALL ON ANY`,
  `SIGNAL ON ANY`, a method, a nested routine with its own `NUMERIC DIGITS 5` raising to the caller's
  trap (SIGL and `DIGITS()` checked), and a routine changing DIGITS between operations.
* `opt3.rex`: condition object fields `C`, `D`, `I`, SIGL, `CONDITION('A')`, `CONDITION('E')` and
  `~description`, `~rc`, `~code`, `~additional` (NIL on the oracle), `~result`, `~propagated`,
  `~instruction`, and `~traceback~items`.
* `prop.rex`: a method, a `::ROUTINE` and a `.Routine~new` with no trap, under the caller's `SIGNAL
  ON LOSTDIGITS`. Both sides stay silent, because `Activity::raiseCondition` stops at the first Rexx
  activation.
* `interp.rex`: `SIGNAL ON LOSTDIGITS` and `SIGNAL ON ANY` set inside INTERPRET.
* `dtw.rex`: DATATYPE `W` over 24 values at DIGITS 9, 20, 21, 40 and 100. The values include
  `1E+20`, `12345678901234567890.0`, negatives, `-0.0`, `0E+5`, `1e+999999999`, `1E-999999999`,
  `.5`, `5.`, a 59-digit integer and blanks. The probe also checks the 42.901 overflow message.
* `dtw2.rex`: DATATYPE `W` LOSTDIGITS descriptions (blanks, exponent, a NumberString value),
  underflow-shaped values (silent on both sides), `~dataType('W')` under `SIGNAL ON ANY`, and
  98.972 under the option.

## Issues

### Critical

None.

### Important

* **I1. `MAX`/`MIN` arguments after the first never raise LOSTDIGITS.** Oracle
  `NumberStringMath.cpp:277` (`compobj->prepareOperatorNumber(saveDigits, saveDigits, ROUND)`).
  Probe `max.rex` at DIGITS 9 under `SIGNAL ON LOSTDIGITS`: `max(1.5, 1111111111)`,
  `min(1.5, 1111111111)`, `1111111111~max(2222222222)`, `1.5~min(2222222222)` and
  `max(1.5, 1, 3333333333)`. The oracle answers `LOSTDIGITS [<arg>]` for each, and the crate answers
  `none <value>`. Probe `maxopt.rex`: `say max(1.5, 1111111111)` under `::OPTIONS LOSTDIGITS SYNTAX`
  is 98.972 at rc 158 on the oracle and prints `1.11111111E+9` at rc 0 here. This falls in the
  cluster's own definition. The report's root cause names `prepareOperatorNumber` but does not
  follow its callers. Fix: call `lostdigits_check` on each non-first argument in the MAX/MIN
  NumberString path, and leave the RexxInteger receiver path alone. On the oracle,
  `max(1, 1111111111)` and `1~max(1111111111)` stay silent, which is the queued
  `max-integer-unrounded` shape. Add the probe lines to the witness.

### Minor

* **M1. Coverage gap.** No witness shows that `ABS`, `FORMAT`, `TRUNC`, `SIGN` or `MAX`'s first
  argument stay silent under a trap (`prepareNumber`, `NumberStringMath.cpp:517`). All agree today
  (`ops.rex`). A witness line would catch a later fix that routes them through the operator check.
* **M2. Pre-existing, outside this diff.** `do 1111111111` and `do i = 1 for 1111111111` at DIGITS
  9 are 26.2 and 26.3 at rc 230 here. The oracle runs the loop, and it raises no LOSTDIGITS even
  under a trap (probes `rep.rex` and `rep2.rex`). The diff does not touch `run/loops.rs` count
  handling. The item needs queuing.
* **M3. Report wording.** The report says of `t6.3/callgrind-arith.txt` that "the binaries differ
  (sha256 in the file)". That file records only the base's sha256. The head's hash (`94e0d552...`)
  is in `t6.3/callgrind.txt` for the same `rexx-head` path. The figures in the report's perf table
  match `t6.1/callgrind-r2.txt` and `t6.3/callgrind.txt` row for row. The base hash `ae65dc12...`
  equals the t6.1-r2 head, which is consistent with `git diff --stat ade6fd223 4f62b0d43 --
  rust/crates` being empty.

## Fast path

The diff adds nothing to the small-integer paths. The only change on an unarmed run is one compare
in `exec_condition_trap` (`run/condition.rs:42`), which runs per `SIGNAL ON`/`CALL ON` and not per
operation. `lostdigits_raise` stays `#[cold]`. The new `lostdigits_raise_with` is reached only from
it and from `lostdigits_check_text`, which DATATYPE `W` calls behind the same gate. DATATYPE `W` now
pays a `Number::add` per call where it used `whole_value` before. That cost is off the arithmetic
path, and the brief does not gate it.

## Assessment

**Spec compliance:** issues found (I1).

**Task quality:** needs fixes. The trap mechanism is right in every shape I probed, but one oracle
raise site of the cluster (MAX/MIN arguments) stays silent under both a trap and the option.
