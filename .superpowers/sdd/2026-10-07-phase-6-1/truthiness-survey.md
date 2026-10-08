# Truthiness and comparison-answer survey (input for a new phase)

Surveyed 2026-10-08, read-only, at committed `HEAD` 485319e20 on `plan/rust-rewrite`.
The working tree then carried 237 uncommitted changed lines in
`rust/crates/rexx-exec/src/run/loops.rs` (Phase 6.1 Task 3 fix round 3), so the
`loops.rs` rows below describe HEAD, not that work in progress. Re-check them
against the tree before planning.

## Question

Every place that decides "is this value logically true" (or "did this operator
answer mean yes") must agree. The oracle bug behind Deviation 25 was one such
judgment implemented twice: `DoBlock.cpp:213` tests `== TheTrueObject` while
WHILE/UNTIL fall back to `truthValue()`. Ruling (Moritz, 2026-10-08): `'1'` in
any form is true, `'0'` false, anything else raises Error 34.

## The canonical predicate

`rexx-exec/src/eval.rs:1436` `logical_value(text: &[u8]) -> Option<bool>`:
exactly `b"0"` / `b"1"`, no coercion. Callers choose their own Error 34
sub-number on `None`.

Sites that route through it correctly: prefix `\` (`eval.rs:674`),
`&`/`|`/`&&` (`eval.rs:1166-1169`), comma list elements (`eval.rs:1356`),
IF/WHEN/WHILE/UNTIL via `condition_value` -> `condition_holds` (`run.rs:3723`),
DO WITH `AVAILABLE` (`run/loops.rs:2802`), `dispatch/string.rs:1256`,
security manager answer (`security.rs:124`), native API `logical`
(`dispatch/library.rs:1498`).

## Sites that do not route through it

| # | Site (HEAD) | Behaviour | Assessment |
|---|---|---|---|
| 1 | `run/loops.rs:2733` DO TO termination test | `is_true_object(answer)` (`loops.rs:2988`): `answer == LOGICAL_TRUE`, a handle identity on the inline `'1'`. A heap `'1'` does not end the loop; `'banana'` raises nothing | Violates the ruling. Fix round 3 presumably replaces it |
| 2 | `run/loops.rs:988` BY sign check (`answer < 0`) in `accept_object_header` | Same `is_true_object` | Same class as #1. Confirm round 3 fixes BY as well as TO |
| 3 | `dispatch/collection.rs:215` `same_item` | `logical_value(..).unwrap_or(false)`: a user `==` answering a non-logical value is silently "not equal" | New candidate defect. Oracle behaviour NOT run; expectation (unverified) is Error 34. Callers: `dispatch/array/surface.rs:104,121,178`, `dispatch/collection/list.rs:309,543,561`, `dispatch/hash.rs:585,1219,1388` |
| 4 | `ir/drive.rs:1477` IR IF/WHEN quick path | Handle compare against `LOGICAL_TRUE`/`LOGICAL_FALSE`, then `SmallInt(1/0)`; otherwise the general `condition_value` | Sound: any miss falls back to the full check. A refinement-proof target |
| 5 | `ir/drive.rs:3911` `register_holds` (used at `drive.rs:1510`) | Same handle tests; anything else is `Loud::register_not_logical` (internal inconsistency) | Sound only if every op that writes the register validated first. A requirement on the compiler, not on the value |
| 6 | `run.rs:3720` `condition_holds(checked = true)` | `text == b"1"`, no `0` check; `checked` is set when the condition is `ExprKind::Logical` (comma list, `run.rs:3629`) | Sound only because each element was already checked at `eval.rs:1356`. Same kind of requirement as #5 |

Adjacent, not a truth check, unverified: `run.rs:3732` `test_case_when`
compares the SELECT CASE value with WHEN values by text byte equality and sends
no `==` message. If the oracle sends `==` to a CASE object, this is the same
"operators are messages" skip. Check `WhenCaseInstruction` in the oracle
source before planning on it.

## Proposed phase shape

1. **One definition with a type wall.** A `Logical`/`truth(answer) ->
   Result<bool, Raised>` that only `logical_value` can produce, so
   `is_true_object` and `unwrap_or(false)` stop being writable. Removes #1-#3.
2. **Fast-path refinement checks for #4-#6.** At minimum
   `debug_assert_eq!(fast, logical_value(text))` at each site (cheap, catches a
   broken compiler requirement). Optionally Kani harnesses over bounded values:
   `fast(v).is_some() => fast(v) == logical(v)`.
3. **Metamorphic truthiness table (datadriven).** Values `'1'`, `1`, `0+1`,
   `'1'||''`, `left('12',1)`, `1~string`, `.true~copy`, `'0'`, `'banana'`, an
   object, each fed through IF, WHEN, WHILE, UNTIL, DO TO (user `>` answering
   the value), BY (user `<`), and `hasItem` (user `==`). Invariant: all contexts
   agree on true/false/Error 34 (sub-numbers may differ per keyword). Judged by
   the crate's own invariant, not oracle equality.
4. **Spec record.** One row per judgment (truth, comparison answer, string
   coercion, ...) citing ANSI X3.274 section, oodocs page and ruling
   (Deviation 25 becomes a clause).

## Re-deriving the site list

Run at the tree being planned against (it was run at HEAD 485319e20 here):

```
git grep -n -E 'logical_value\(|is_true_object\(|LOGICAL_TRUE|LOGICAL_FALSE|unwrap_or\(false\)' HEAD -- rust/crates/rexx-exec/src
git grep -n -E 'apply_binary\(' HEAD -- rust/crates/rexx-exec/src
```

The second command finds operator answers consumed as booleans outside
`eval.rs`. It does not find truth decisions made on a value that never passed
through `apply_binary` or these constants. The list above covers what these
patterns match; it is not proven complete.
