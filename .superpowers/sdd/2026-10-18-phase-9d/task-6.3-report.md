# Task 6.3 report: numeric conditions

Clusters `lostdigits-not-raised` and `datatype-whole-wide-numbers`. Status: DONE_WITH_CONCERNS.
14 of the 16 cluster tests pass. The other 2 now fail on causes owned elsewhere (see "Tests").

## Commits

* `9095627aa` Phase 9d Task 6.3: LOSTDIGITS reaches a SIGNAL ON trap; DATATYPE W at any width.
  This commit holds the code, the two witnesses, their `sourceline_oracle` files and `phase-9.txt`.
* `a65279c10` records: holds `suite-9d-set.txt`, `docs/superpowers/records/2026-10-18-phase-9d/t6.3/`
  and this report.

## Root causes

1. **LOSTDIGITS was only ever a SYNTAX escalation.** `Interp::lostdigits_raise`
   (`rust/crates/rexx-exec/src/run/condition.rs`) returned at once unless `::OPTIONS LOSTDIGITS
   SYNTAX` was in force, and its gate `lostdigits_armed` was set only by that option. The oracle's
   `NumberString::checkLostDigits`, `addSub` and `prepareOperatorNumber` call
   `reportCondition(LOSTDIGITS)`, and a `SIGNAL ON LOSTDIGITS` or `SIGNAL ON ANY` trap takes it.
   The fix makes `SIGNAL ON LOSTDIGITS` and `SIGNAL ON ANY` arm the gate in
   `exec_condition_trap`, in the same way NOSTRING arms `reqstr_armed`. The armed half now checks the
   digit count, then a `SIGNAL ON` trap (raising the LOSTDIGITS condition with the operand as
   written as its description), then the option (98.972).
   * `CALL ON ANY` does not take it on the oracle (measured, `t6.3/probes`, and the witness), so
     `CALL ON` neither arms the gate nor takes the raise.
   * Where the check runs: every site that already called `lostdigits_check` (`arith_general_body`,
     `apply_prefix_body`, `compare_values_body`'s numeric arm, `header_number_body`). That covers
     literals, variables, strings, INTERPRET-built values and integers held as SmallInt. The small
     integer fast paths (`small_int_arith`, `small_int_compare`, the loop increments) decline any
     operand that `within_digits` rejects, so an operand wider than DIGITS always reaches the general
     path. The witness's `w+0`, `w<1` and `w-v` hold a SmallInt `1000000000` at DIGITS 9.
2. **`DATATYPE(x, 'W')` was bounded by an i64.** It used `Number::whole_value`, which answers `None`
   above the i64 range, so a 19-digit whole number answered 0. The oracle
   (`StringUtil::dataType`, `classes/support/StringUtil.cpp:1076`) computes `x + 0` at the running
   DIGITS and asks `NumberString::isInteger` of the result. The fix does the same with `Number::add`
   and `Number::is_integer`. That `+` is an operator, so the oracle also raises LOSTDIGITS and
   42.901 from it. Both now happen here (`lostdigits_check_text`, `Raised::from`).
   `datatype_matches` now takes `&mut Interp` and returns `Result`, and both callers (`DATATYPE`
   and `String~dataType`) propagate the failure.

## Witnesses, red then green

Both witnesses were run against the oracle and against a crate binary built at `4f62b0d43` (red),
then against one built at `9095627aa` (green). Outputs are in
`docs/superpowers/records/2026-10-18-phase-9d/t6.3/witnesses/{red,green}/<name>/`, with oracle and
crate stdout, stderr and rc kept as separate files.

* `rust/corpus/lang/lostdigits_signal_trap.rex`. This witness covers each arithmetic operator,
  prefix `+`/`-`, `=` and `<`, a string operand with blanks, a variable, and SmallInt variables. It
  includes `==`, a non-numeric `=` and `||`, which do not raise, and the boundary at DIGITS 8 and 10.
  All of these run through INTERPRET (Review Focus 4). It also covers an untrapped operand,
  `CALL ON ANY` (which does not take it), `SIGNAL ON ANY`, an internal routine with no trap raising
  to the caller's trap, a method's own trap through INTERPRET, a `DO` header operand, and
  `SIGNAL OFF`.
  * Red: stdout differs from the first line on, and the crate dies at `1111111111%1` with 26.11 at
    rc 230 where the oracle ends at rc 0.
  * Green: stdout, stderr and rc are identical. I read the stdout and every path above prints its
    line.
* `rust/corpus/lang/datatype_whole_wide.rex`. This witness covers `W` at DIGITS 19, 20 and 30,
  including rounding, exponent forms and a fractional part. It covers `W` at DIGITS 9 on the
  boundary cases, `String~dataType('W')`, the LOSTDIGITS raise, and the 42.901 overflow.
  * Red: the first three lines answer all 0 where the oracle answers `1 1 0 1 1`, `1 1 1 0` and
    `1 0 1 1 0 0`, and the crate prints `not raised` for both raises.
  * Green: stdout, stderr and rc are identical.

## Tests

The run used the gated whole suite, against `t6.2/suite/tests.tsv`:

* Now pass: `base/keyword/LOSTDIGITS` (10 tests), `base/class/Validate`
  `TEST_VALIDATE_NON_NEGATIVE_WHOLE_NUMBER`, `TEST_VALIDATE_POSITIVE_WHOLE_NUMBER` and
  `TEST_VALIDATE_WHOLE_NUMBER`, and `base/directives/OPTIONS TEST_LOSTDIGITS_SYNTAX_OVERRIDE`. These
  14 lines leave `rust/corpus/suite-9d-set.txt`.
* Still failing, on a cause outside these clusters, so both stay in `suite-9d-set.txt`:
  * `base/bif/CONDITION TEST_CONDITION_SIGNAL`. The failure moved from line 170 (LOSTDIGITS, this
    cluster) to line 203, `CONDITION(A)` expected `object`, actual the NIL object. The NOMETHOD
    condition object has no `ADDITIONAL` here. The probe `t6.3/probes/condsig.rex` shows the
    LOSTDIGITS step now identical to the oracle. NOMETHOD, NOSTRING and NOTREADY all lack
    `ADDITIONAL`, and NOTREADY also lacks `RESULT`. `clusters.md` assigns NOTREADY's `ADDITIONAL`
    to `stream-notready-names`. No cluster names NOMETHOD's or NOSTRING's.
  * `base/keyword/SIGNAL TEST_SIGNAL_ANY_CONDITION_LOSTDIGITS`. LOSTDIGITS now reaches the
    method's `SIGNAL ON ANY` handler, but its `RAISE PROPAGATE` reaches the caller as SYNTAX
    (`t6.3/probes/anyprop.rex`). That is `raise-propagate-from-method`, as `clusters.md` already
    notes. Its NOSTRING sibling fails the same way.
* No queued item or exclusion row is closed. `queued-inventory.tsv` has no LOSTDIGITS or DATATYPE
  row, and the one queued file mentioning LOSTDIGITS
  (`2026-10-10-raise-novalue-return-into-call-on-any.md`) is about `RAISE ... RETURN`.

## Changed rows

`diff` of `t6.2/suite/tests.tsv` against `t6.3/suite/tests.tsv` (sorted, header dropped) shows only
these rows:

* The 14 tests listed above, which move from oracle passed / crate failed to passed / passed.
* `base/rexxutil/SysFileTree TEST_TREE_DIRECTORY`, oracle `failed` to `passed`. This is the known
  flake with a queued item. I am ignoring it as instructed.

`table.tsv` and `verdicts.tsv` change only for `base/class/Validate` and `base/keyword/LOSTDIGITS`
(red to green), `base/directives/OPTIONS` (one fewer failing test), and the SysFileTree oracle row.

## Perf

`t6.3/callgrind.txt` holds the whole-process figures against the pin, and the task delta below is
measured against `t6.1/callgrind-r2.txt`:

| program | t6.1-r2 vs pin | t6.3 vs pin | task delta |
| --- | --- | --- | --- |
| startup | +0.0173 | +0.0130 | -0.0043 |
| interpret | -2.1860 | -2.1836 | +0.0024 |
| parse | -0.0877 | -0.0879 | -0.0002 |
| churn | +0.1525 | +0.1262 | -0.0263 |
| sayloop | +0.0331 | +0.0297 | -0.0034 |

Every delta is under +0.05%, and every figure against the pin is under +0.5%.

The arithmetic fast path was not touched. To measure that the fix costs nothing when nothing is
trapped, I ran `arith`, `rexxcps` and `emptyloop` with this task's parent binary against the head
(`t6.3/callgrind-arith.txt`). The results were arith -0.0000%, rexxcps -0.0000% and emptyloop
-0.0001% whole process. The binaries differ: `t6.3/callgrind-arith.txt` records the base's sha256 (`ae65dc12...`), and the head's (`94e0d552...`) is in `t6.3/callgrind.txt` for the same `rexx-head` path. The new construction stays in the
`#[cold]` `lostdigits_raise`, and the only new cost on the unarmed path is one compare in
`exec_condition_trap`.

## Checks

All commands ran from `rust/` with `CARGO_TARGET_DIR=<scratchpad>/target-9d-t6` and
`CARGO_INCREMENTAL=0`:

| command | exit |
| --- | --- |
| `timeout 590 memcap 8G cargo build --release -j 4 -p rexx-exec --bin rexx` | 0 |
| `cargo fmt --all --check` | 0 |
| `timeout 590 memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` | 0 |
| `timeout 600 memcap 8G cargo test -j 4 --release -p rexx-exec --test refusal_sites --test refusal_dispositions --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test licensed_divergences` | 0 |
| `REXX_CORPUS_GATE=1 timeout 600 memcap 8G cargo test -j 4 --release -p rexx-exec --test corpus` | 0 (29 passed, 1 ignored) |
| `timeout 600 memcap 8G cargo test -j 4 --release -p rexx-parse --test sourceline_oracle` | 0 |
| `timeout 600 memcap 8G cargo test -j 4 --release -p rexx-exec --lib datatype` | 0 (17 passed) |
| `timeout 600 memcap 8G cargo test -j 4 --release -p rexx-exec --test suite` (row checks, after the set edit) | 0 (52 passed) |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=docs/superpowers/records/2026-10-18-phase-9d/t6.3/suite/table.tsv timeout 600 memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` | 101 (expected) |
| `bench-programs/callgrind.sh -B 0.5 -p "startup interpret parse churn sayloop" ...` | 0 |
| `bench-programs/callgrind.sh -p "arith rexxcps emptyloop" base=... head=...` | 0 |

I made the `sourceline_oracle` files with the driver in `sourceline_oracle.rs`'s module comment,
using the oracle.

## Concerns

* `TEST_CONDITION_SIGNAL` now fails on NOMETHOD's (and NOSTRING's) missing `ADDITIONAL`, which no
  cluster owns. It needs an owner.
* Not in this cluster, seen while probing: `2~max(1111111111)` at DIGITS 9 answers `1111111111` on
  the oracle and `1.11111111E+9` here (`max(2, 1111111111)` agrees). This looks like a `MAX`
  counterpart of Deviation 44's `RexxInteger::Min` representation leak. I did not investigate it.

## Fix round 1

Review: `task-6.3-review.md`. This round fixes I1 and M1, corrects the M3 sentence above, and
leaves M2 to the queued item.

### Commits

* `cc1285116` Phase 9d Task 6.3 fix round 1: MAX and MIN arguments raise LOSTDIGITS. It holds the
  code, the witness `lostdigits_max_min.rex`, its `sourceline_oracle` file, and `phase-9.txt`.
* The records commit holds `t6.3/witnesses/{red,green}/lostdigits_max_min/`,
  `t6.3/suite-r1/`, `t6.3/callgrind-r1.txt`, and this section.

### I1: MAX and MIN arguments

`NumberString::maxMin` runs each argument after the first through `prepareOperatorNumber`
(`classes/NumberStringMath.cpp:277`), which reports LOSTDIGITS. `max_min_over`
(`builtin/numeric.rs`) is the general path for the builtin and the message alike. It now calls
`lostdigits_check_with` on each argument after the first, before rounding. That covers a
`SIGNAL ON` trap and `::OPTIONS LOSTDIGITS SYNTAX` (98.972, rc 158).

* The builtin's integer path (`integer_path`) already returns before `max_min_over`, so
  `max(1, 1111111111)` stays silent. The message form had no integer path. `string_max_min` now
  asks `integer_path_applies`, the same test of an integer target valid under DIGITS and an integer
  in every argument, and checks LOSTDIGITS only when it fails and the gate is armed. So
  `1~max(1111111111)` stays silent. The value either path answers is unchanged (queued
  `2026-10-18-max-integer-unrounded.md`), and the witness prints no value for those cases.
* The description is not the argument as written. `prepareOperatorNumber` clones the number, so
  the oracle renders it from the digits: `' 4444444444 '` names `4444444444`, and
  `44444444440E-1` names `4.4444444440E+9` (probe `t6.3/probes/maxdesc.rex`, whose cases the
  witness repeats). The new
  `Number::render_unrounded` (`rexx-num/src/lib.rs`) is `NumberString::stringValue`
  (`classes/NumberStringClass.cpp:338`) on an unrounded number. An exponent of zero renders as
  plain digits. Otherwise the exponent trigger is the DIGITS in force, with ENGINEERING's multiples
  of 3. `lostdigits_check_text` became `lostdigits_check_with`, which takes the description as a
  closure that runs only on a raise. DATATYPE `W` passes its text through it.

### M1

`lostdigits_max_min.rex` also shows that `ABS`, `~abs`, `FORMAT`, `TRUNC`, `SIGN`, `~sign`, and
`MAX`'s and `MIN`'s first argument (builtin and message) raise nothing under a trap.

### Witness, red then green

`rust/corpus/lang/lostdigits_max_min.rex` covers the review's `max.rex` lines, `maxopt.rex` (as a
`.Routine` built with `::options lostdigits syntax`, last in the program), the description shapes
under SCIENTIFIC and ENGINEERING, the integer-path silences and the M1 lines.

* Red, against the binary at `9095627aa`: every non-integer-path MAX/MIN line answers `none`
  where the oracle answers `LOSTDIGITS`, and the program ends at rc 0 where the oracle ends at
  98.972, rc 158.
* Green, at `cc1285116`: stdout, stderr and rc are identical. I read the stdout and every line
  prints.

The other two witnesses still match the oracle against the new binary.

### Changed rows

The whole suite (`t6.3/suite-r1/`) against `t6.3/suite/` shows no difference in `tests.tsv`,
`table.tsv` or `verdicts.tsv`.

### Perf

`t6.3/callgrind-r1.txt`, against the pin, with the delta from `t6.3/callgrind.txt`:

| program | t6.3 vs pin | r1 vs pin | delta |
| --- | --- | --- | --- |
| startup | +0.0130 | +0.0144 | +0.0014 |
| interpret | -2.1836 | -2.1791 | +0.0045 |
| parse | -0.0879 | -0.0879 | 0.0000 |
| churn | +0.1262 | +0.1265 | +0.0003 |
| sayloop | +0.0297 | +0.0310 | +0.0013 |

Every delta is under +0.05%, and every figure against the pin is under +0.5%.

### Checks

All commands ran from `rust/` with the task's `CARGO_TARGET_DIR` and `CARGO_INCREMENTAL=0`:

| command | exit |
| --- | --- |
| `timeout 590 memcap 8G cargo build --release -j 4 -p rexx-exec --bin rexx` | 0 |
| `cargo fmt --all --check` (after `cargo fmt --all` reflowed two lines) | 0 |
| `timeout 590 memcap 8G cargo clippy -j 4 -p rexx-exec -p rexx-num --all-targets -- -D warnings` | 0 |
| the refusal-table command of the first round | 0 (7 binaries ok) |
| `timeout 600 memcap 8G cargo test -j 4 --release -p rexx-num` | 0 (193 passed) |
| `timeout 600 memcap 8G cargo test -j 4 --release -p rexx-exec --lib builtin` | 0 (203 passed) |
| `timeout 600 memcap 8G cargo test -j 4 --release -p rexx-parse --test sourceline_oracle` | 0 |
| `REXX_CORPUS_GATE=1 timeout 600 memcap 8G cargo test -j 4 --release -p rexx-exec --test corpus` | 0 (29 passed, 1 ignored) |
| `timeout 600 memcap 8G cargo test -j 4 --release -p rexx-exec --test suite` | 0 (52 passed) |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../t6.3/suite-r1/table.tsv ... l3_core_suite` | 101 (expected) |
| `bench-programs/callgrind.sh -B 0.5 -p "startup interpret parse churn sayloop" ...` | 0 |

### Concern

The integer-path test (`integer_object`) reads the value's spelling, where the oracle reads whether
the object is a `RexxInteger`. At DIGITS 9 the oracle stays silent for `max(1, 1111111111)`, a
literal, and raises for `max(1, '1111111111')`, a string. This crate answers both by spelling and
stays silent for both (probe `t6.3/probes/maxstr.rex`). That is the representation question
queued in `2026-10-18-max-integer-unrounded.md`.
