STATUS: IN PROGRESS

# Task 8 report: expression evaluation, part two -- comparison and logic

Scope: `rust/crates/rexx-exec/src/eval.rs` only, per the plan's own Files
list. `rexx-num`'s side (`compare_bytes`/`compare_decoded`, `5bf9b03d`)
already landed as Task 8a, reviewed and DONE before this task started.

Filling this in as I go, per the project's write-report-first discipline.

## Pre-flight reading

The plan doc's Task 8 section in full; `rexx-num/src/compare.rs` (the
whole file -- `CompareOp`, `compare`/`compare_bytes`/`compare_decoded`,
`numeric_order`, `string_order`); `rexx-num/src/settings.rs` for
`Settings::digits()/fuzz()/form()`; `rexx-parse/src/token.rs`'s `Operator`
enum (all thirty-one variants, to build the family split correctly);
`rexx-parse/src/ast.rs`'s `ExprKind::Logical` doc comment (the comma-list
node: built by `parseLogical` for `IF`/`WHEN`/`GUARD`/`WHILE`/`UNTIL`, "no
element may be omitted"); the current `eval.rs` end to end, especially
`eval_node`'s dispatch shape, `eval_arithmetic`'s operand-rooting pattern,
and `arith_operand`'s NotNumeric-to-41.1 conversion, which the comparison
and logical arms reuse the shape of rather than inventing a new one;
`value.rs`'s `to_number`, confirmed to already implement exactly the
tri-state-cache-respecting parse the plan asks comparison to use (it
fills `Body::Text`'s `num` cache at most once and returns an owned
`Number`, so calling it is not the "re-parse on every call" the plan
warns against -- the warning is about the `&str` *entry point*
re-parsing, not about calling `to_number` before `compare_decoded`).

## Investigation: the family split and what changed since the plan was written

The plan's Step 3 lists three families (numeric-or-string, strict,
logical) plus `ExprKind::Logical`, but `Operator`'s actual thirty-one
variants split the comparison group differently than the plan's own
transcript block suggests, and needed mapping to `CompareOp` explicitly:

* Numeric-or-string (`CompareOp`, non-strict): `Equal`, `BackslashEqual`,
  `GreaterThan`, `LessThan`, `GreaterThanEqual`, `LessThanEqual`, plus the
  three synonyms `token.rs` keeps as separate `Operator` variants for the
  same `CompareOp`: `LessThanGreaterThan`/`GreaterThanLessThan` (both
  `NotEqual`, matching `compare.rs`'s own doc comment that `\=`, `<>` and
  `><` share one C++ method pointer), and the two backslash-negated forms
  `BackslashGreaterThan` (`\>`, "not greater than" = `LessEqual`) and
  `BackslashLessThan` (`\<`, "not less than" = `GreaterEqual`).
* Strict (`CompareOp`, strict): `StrictEqual`, `StrictBackslashEqual`,
  `StrictGreaterThan`, `StrictLessThan`, `StrictGreaterThanEqual`,
  `StrictLessThanEqual`, and the two backslash-negated strict forms
  `StrictBackslashGreaterThan` (`\>>` = `StrictLessEqual`),
  `StrictBackslashLessThan` (`\<<` = `StrictGreaterEqual`).
* Logical (`Operator`, not `CompareOp` at all): `And`, `Or`, `Xor` (`&`,
  `|`, `&&`).

`CompareOp::is_strict`/`holds` are both private to `rexx-num` (confirmed
by reading `compare.rs`), so `eval.rs` cannot ask a `CompareOp` whether it
is strict. Not worth requesting a third accessor for: strictness is
decided here anyway, to choose whether `to_number` is even worth calling
(see below), so it is read directly off the *`Operator`*, before the
`CompareOp` conversion, rather than derived from the enum `compare.rs`
made private.

## Oracle verification

All under `( ulimit -v 1048576; build/bin/rexx FILE )`, run before writing
any comparison code.

**The thirteen-line transcript from the plan, unmodified:**

```
' a' = 'a'      -> 1       '09'x'a' = 'a'   -> 1      'a' = 'a'||'09'x -> 1
'a' = 'a '      -> 1       'a b' = 'a  b'   -> 0      '' = ' '         -> 1
'01' = '1'      -> 1       ' 1 ' = 1        -> 1      'a' = 1          -> 0
'10' >> '9'     -> 0       '10' > '9'       -> 1      'a' << 'a '      -> 1
'01' == '1'     -> 0
```

All thirteen matched on the first run.

**Backslash-negated and synonym forms, not in the plan's own list, needed
to pin the `Operator` -> `CompareOp` mapping above:**

```
'9' \> '10'    -> 1   ('9'<='10' via the LessEqual mapping)
'9' \< '10'    -> 0   ('9'>='10' via GreaterEqual)
'10' \> '9'    -> 0
'10' \< '9'    -> 1
'9' \>> '10'   -> 0   (strict: '9' sorts after '10' byte-wise, so strict > is true, \>> false)
'9' \<< '10'   -> 1
'9' <> '10'    -> 1
'9' >< '10'    -> 1
'9' \= '10'    -> 1
'9' \== '9'    -> 0
'9' >>= '9'    -> 1
'9' <<= '9'    -> 1
'8' >>= '9'    -> 0
'10' <<= '9'   -> 1
```

All fourteen matched the mapping above on the first run -- no
implementation existed yet to have biased the transcript.

**Logical family truth tables and the four error-triggering values:**

```
1 & 1    -> 1        1 & 0   -> 0        0 | 0   -> 0
1 && 1   -> 0         1 && 0  -> 1        \1      -> 0        \0 -> 1

' 1 ' & 1 -> Error 34.901, found " 1 "
'01'  & 1 -> Error 34.901, found "01"
'1.0' & 1 -> Error 34.901, found "1.0"
''    & 1 -> Error 34.901, found ""
```

Confirms `&`/`|`/`&&` share the exact 34.901 message `\`'s prefix already
raises (Task 7) -- `Raised::not_logical` is reused as-is, no new
constructor for this family.

**Two findings not in the plan, both load-bearing for the implementation:**

1. **`&`/`|` do not short-circuit.** `say (0 & 'x')` and `say (1 | 'x')`
   both raise 34.901 on `"x"` even though the overall AND/OR result is
   already determined by the first operand alone. So both operands are
   always evaluated and checked, unconditionally -- the same
   "unconditional both sides" shape `eval_arithmetic` already uses, not a
   new one.
2. **`&`/`|` check the left operand first when both are bad.**
   `say ('y' & 'x')` reports `found "y"`, not `"x"` -- confirms left is
   evaluated and checked before right, matching every binary operator
   already implemented (arithmetic's base-before-exponent, concatenation's
   left-then-right).

**`ExprKind::Logical` (the comma list): the sub-number split, and the
short-circuit that makes it different from `&`.**

```
if 'x' then nop                    -> Error 34.1  (single condition, not a list)
if 1, 'x' then nop                 -> Error 34.6, found "x"
if 'x', 1 then nop                 -> Error 34.6, found "x"  (checks left-to-right)
select; when 'x' then nop; ...     -> Error 34.2
do while 'x' ... end                -> Error 34.3
do until 'x' ... end                -> Error 34.4
if 0, 'x' then nop / say 'reached' -> prints "reached", no error at all
if 1, 0, 'x' then nop / else ...   -> "else-taken", no error at all
if 1, 1, 1 then ... else ...       -> the then-branch (all-true)
if 1, 0, 1 then ... else ...       -> the else-branch (short-circuited false)
```

This resolves the plan's own six-sub-number list (`34.1 IF, 34.2 WHEN,
34.3 WHILE, 34.4 UNTIL, 34.6 the comma list, 34.901 &/|/\`) into an actual
rule rather than five arbitrary numbers: **34.1/34.2/34.3/34.4 fire when
the condition is a single expression, evaluated directly by an
instruction that does not exist in this phase yet (`IF`/`WHEN`/`WHILE`/
`UNTIL` are Tasks 9-11's); 34.6 fires when the bad value is one element of
a multi-element comma list**, regardless of which of the five keywords
(`IF`/`WHEN`/`GUARD`/`WHILE`/`UNTIL`) the list belongs to -- the list
itself does not know or care which keyword built it. So `eval_node`'s own
`ExprKind::Logical` arm, which is all Task 8 can build (no instruction
context exists to hand it a different sub-number), always raises 34.6.
34.1-34.4 (and presumably 34.5 for `GUARD`, not measured -- `GUARD` is
outside 4a's scope) are Tasks 9-11's to raise, when their own condition
evaluation calls `eval` on a *non-Logical* `Expr` directly and checks its
own result rather than delegating to this arm.

**And the comma list short-circuits, unlike `&`.** `if 0, 'x' then` never
evaluates `'x'` at all (`say 'reached'` runs with no error), and `if 1, 0,
'x' then` never reaches the third element either. So `ExprKind::Logical`
is evaluated and checked left to right, stopping at the first element
that checks out `false` -- the remaining elements are neither evaluated
nor checked. This is the opposite of `&`'s own unconditional-both-sides
rule measured above, and both are implemented exactly as measured rather
than made to agree with each other.

## Implementation

(pending)

---

## Controller's note, 2026-07-31

**The implementer's session ended here.** Everything above is its own work; the
`Implementation` section was never written and STATUS never moved off IN
PROGRESS. Neither is being back-filled by me: an invented account of decisions I
did not make is worse than an admitted gap, and the measurements above are the
part that matters to a reviewer anyway.

What existed on disk when the session ended: `eval.rs` complete and uncommitted,
`plan.rs` carrying Task 6's fix, both green. Committed as `9843c90d` and
`d7633695` respectively, which is the review range.

**Independently verified before committing**, from a case list built separately
from the tests above so agreement means something: 22 programs against
`build/bin/rexx`, comparing value and exit code, later extended to full stderr
once the reporting path was wired. Agreement on all of them, including
`'01' == '1'` = 0 against `01 = 1` = 1, `'1 ' = 1` = 1 (the leading-blank strip),
`'A' < 'a'`, `0 & (1/0)` raising 42.3 rather than short-circuiting, and `2 & 1`
raising 34.901.

**What a reviewer should treat as NOT covered by that:** the comma-list
short-circuit and the 34.6-versus-34.901 split, both of which are the report's
own claims tested by the report's own tests. The independent set exercised `&`
and `|`, not `ExprKind::Logical`, because 4a has no instruction that builds one
yet -- `IF` is Task 10's. That is the weakest point in this task's evidence.
