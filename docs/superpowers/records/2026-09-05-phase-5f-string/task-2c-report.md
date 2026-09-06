# Phase 5f Task 2c — `?`, the operator row that is not an operator

Plan: `docs/superpowers/plans/2026-09-05-phase-5f-string.md`, Task 2, D85.
BASE `ef5f2d845`. Landed at `5f6321db8`.

The thirty-third operator row. `?` is registered beside the operators —
`AddMethod("?", RexxString::choiceRexx, 2)` (`memory/Setup.cpp:678`) — but the
language has no `?` operator, so there is no expression form for the message to
agree with. It is the one row in `dispatch/string.rs` with a single dispatch.

String now reads 91 `answers`, 6 `uncomparable`, 38 `loud`: **74 of the phase's
112 bound**, 38 left — 25 (Task 3) and 13 (Task 4).

## 1. The body is three lines, and their order is the behaviour

```cpp
requiredArgument(trueResult, "true value");
requiredArgument(falseResult, "false value");
return truthValue(Error_Logical_value_method) ? trueResult : falseResult;
```

Both arguments are required and *named* — 88.901, not the 93.903 the rest of
`String`'s argument layer raises — and the receiver is read as a logical value
only once both are there. Measured on the oracle, the whole of it:

| send | oracle |
|---|---|
| `'1'~"?"('yes','no')` / `'0'~…` | `yes` / `no` |
| `'1'~"?"(.nil,'no')` | `The NIL object` — `.nil` is an ordinary argument |
| `'1'~"?"()` | 88.901 `argument true value is required` |
| `'1'~"?"('y')` | 88.901 `argument false value is required` |
| `'1'~"?"(,'n')` | 88.901 `true value` — omitted is omitted, not positional |
| `'abc'~"?"('y','n')` | 34.901 `found "abc"` |
| **`'abc'~"?"()`** | **88.901 `true value`** — the argument check comes first |
| `'01'`, `''`, `' 1 '`, `'1.0'` | 34.901 — logical is exactly `0` or `1`, as text |
| `'1'~"?"(1,2,3)` | 93.902, `2 expected` |

The last row of that table is the one the plan could not have predicted from
reading: a receiver that is not logical *and* no arguments produces the
argument's error, not the receiver's.

## 2. The plan's premise for deferring this was wrong

Task 2c was held back for "a logical-value reader that does not exist as a
helper". It does: `eval::logical_value` is `pub(crate)`, is the single
definition of what counts as logical for prefix `\`, `&`/`|`/`&&` and
`ExprKind::Logical`, and `Raised::not_logical` is the 34.901 beside it.
`Raised::missing_named_argument` is the 88.901. The body is those three plus
two `args` reads.

## 3. Two witnesses, because the trap cannot see a message

The two 88.901s differ only in the argument they name, and **no trap can show
that**: `CONDITION("O")` answers a Directory and `CONDITION("A")` an Array,
both of which this crate refuses on purpose, and `CONDITION("D")` is empty for
SYNTAX. A handler can print `88.901` and nothing more, so a body that named
both arguments `true value` would pass a single trapped witness.

So each named argument is the untrapped tail of its own program:

- `corpus/lang/string_choice.rex`, rc 168, 20 lines — the four answering sends,
  then untrapped `'abc'~"?"()`. That tail is doing two jobs: it is the ordering
  claim, and its stderr is where `true value` is compared as bytes.
- `corpus/lang/string_choice_refusals.rex`, rc 168, 43 lines — every refusal by
  number through a trap, then untrapped `'1'~"?"('y')` for `false value`.

Both byte-identical to the oracle on stdout, stderr and exit status, on
`REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`.

## 4. `?` puts 34.901 on the send surface

`corpus/refusal-sites.tsv` reddened: `Raised::not_logical` was a `body`
constructor and this is the first construction site under `dispatch/`. The
census is a census of *where a refusal is constructed*, so the honest fix is to
walk the row, not to hide the site behind an `eval.rs` helper — that would have
left the table accurate about construction sites and blind to the fact that a
send can now produce this refusal.

Probed, and the row records it: `.String~new("abc")~"?"("y", "n")` agrees on
both engines and all three descriptors. The witness column says why the obvious
nearby spelling will not do — `say \'abc'` is the identical 34.901 from the
`eval.rs` site and reaches nothing under `dispatch/`, which is exactly the
"tries the obvious one, sees byte-identical output, deletes the row" failure
that column exists to stop. No `SHARED_ANSWERS` change: no other send-surface
row answers 34.901.

## 5. The control — three mutations, predicted before running

**D — the two argument names swapped.** Predicted: both files red on stderr,
each naming the other argument; no stdout row moves, because the trap prints
only the number. Measured exactly that. **Confirmed.**

**E — the receiver read before the arguments.** Predicted: `string_choice.rex`
red on stderr *and* rc 222 instead of 168; `string_choice_refusals.rex` red on
stdout row 5 alone — rows 1–3 have a logical receiver, row 4 and rows 6–9
already raise 34.901, row 10's arity check is outside the body, and the tail's
receiver is logical so its stderr does not move. Measured exactly that.
**Confirmed, every part.**

**F — the two results swapped.** Predicted: `string_choice.rex` four stdout rows
red, `string_choice_refusals.rex` fully green, since every row there raises
before a result is chosen. Measured exactly that. **Confirmed.**

E is the one that matters: it is the only mutation that makes the ordering claim
false, and it is caught in two places at once, on two different descriptors.

## 6. Gates

| | |
|---|---|
| G1 `cargo fmt --all --check` | rc 0 |
| G2 `cargo clippy --workspace --all-targets -- -D warnings` | rc 0, zero warnings |
| G3 `cargo test --release --workspace --no-fail-fast` | rc 0 |
| G4 same with `REXX_CORPUS_GATE=1` | rc 0 |
| G5 `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace` | rc 0 |
| G6 `REXX_PHASE_GATE=5c` | rc 0 |
| G7 `REXX_PHASE_GATE=5d` | rc 0 |

Pre-commit chain at the committed tree: method-bodies refresh rc 0 (one row,
`?` `loud` -> `answers`), fmt rc 0, clippy rc 0, strict corpus 381 of 381,
full workspace test rc 0.
