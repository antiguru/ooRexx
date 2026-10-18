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
-0.0001% whole process. The binaries differ (sha256 in the file). The new construction stays in the
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
