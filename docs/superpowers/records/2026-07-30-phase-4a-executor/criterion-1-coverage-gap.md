STATUS: DONE

# Criterion 1 coverage gap analysis

Does `rust/corpus/phase-4a.txt` satisfy criterion 1's coverage property: every
`InstructionKind`, `ExprKind`, `LoopKind`, `PrefixOp`, `EndStyle` and `Trace`
variant in 4a's scope, and every `Operator`, constructed by at least one program
in the subset?

**No. Nineteen in-scope variants are unconstructed, and separately the subset
contains three programs that 4a cannot run at all.** The second is the more
serious and is not a coverage question.

Both are closable cheaply: **three new programs** cover every missing variant.

## Revised after three rulings

This report was first written before the team lead ruled on the two questions it
raised, and the count above reflects the rulings:

* **The three `List` programs come out of `phase-4a.txt`** and stay in
  `corpus/lang/` and `corpus/num/` for 4b or 4c. **Re-measured with them
  excluded: the gap list does not change at all.** The only difference between
  the 26-program and 23-program runs is that `List` disappears from the
  `ExprKind` set; every other variant in every other enum is still covered by
  the remaining 23. So the three were entirely redundant for coverage, which
  strengthens the case for dropping rather than rewriting them.
* **All four `Trace` variants are in 4a's scope**, so `Default`, `Skip` and
  `Value` are real gaps rather than the ambiguity this report first recorded.
  That takes the count from sixteen to nineteen and adds a third witness
  program. `Skip` is observable: `trace 5` raises **24.901** at rc 232, not a
  silent no-op.
* **`Operator::Backslash` is excluded with an owner string**, not a witness,
  because it cannot appear in a `Binary` node.

## Method

Measured, not read. A scratch analyser parses exactly the 26 listed programs
with `rexx-parse` itself and records every variant constructed, walking each
`Expr` recursively. Built and run from a `git archive` export; nothing in the
repository was modified and no corpus program was written.

Three things that reading would have got wrong, and did:

* **`keyword()` conflates `When` and `WhenCase`** — `ast.rs:912` maps both to
  `"WHEN"`. My first run used it and reported `WHEN` covered, leaving
  `WhenCase` indeterminate. Split explicitly, **`WhenCase` is covered.** This is
  a trap for whoever writes the enumerating test: identity must come from the
  variant, not from `keyword()`, or `WhenCase` is silently satisfied by any
  `WHEN`.
* `Debug` on `Expr` is still recursive with a cliff near 2,000, and
  `deep_nested_expr.rex` is 3,000 terms, so `{:?}` on a node would have aborted
  the analyser. Variant names are matched, not formatted.
* The programs recommended below were themselves put through the analyser
  rather than trusted, since "this program obviously constructs X" is the exact
  guess this analysis exists to replace.

---

## The finding that is not about coverage: three subset programs cannot run under 4a

`num/digits_rounding.rex`, `num/exponential.rex` and `num/operators.rex` — all
three inherited from Phase 2 — construct **`ExprKind::List`**, from a comma in a
`SAY` expression such as `operators.rex:8`:

```rexx
say (-2) ** 3 , (-2) ** 2
```

The oracle runs it, printing `-8` and `4` on two lines, rc 0. The spec, line
371: "`Call`, `QualifiedCall`, `Message`, `ClassResolver`, **`List`** and
`VariableReference` are not 4a's and **fail loudly**."

So under 4a as specified these three exit with the not-implemented code where
the oracle prints two lines. **Criterion 1 requires zero divergences over the
subset, so it cannot pass as the subset stands**, independent of any coverage
gap. The subset's own header claims every entry "was read and checked against
what Phase 4a's executor implements", so this is a checking miss rather than a
known compromise.

Three ways out, and the choice is not mine:

1. **Drop the three programs.** Cheapest. Costs some numeric coverage, though
   `num/comparison.rex` and `num/notation_thresholds.rex` remain and
   `arith_digits.rex` is in scope.
2. **Rewrite the three** to split the comma clauses into separate `SAY`s. Keeps
   the numeric coverage; changes inherited programs, which have their own
   provenance.
3. **Bring `List` into 4a** for the `SAY` case. I would not: the general form
   builds an array, which is Phase 5's object model, and a partial `List` is the
   kind of half-implemented surface criterion 5 exists to catch.

---

## Coverage, per enum

Legend: **covered** / **MISSING** (in 4a's scope, no witness) / owned elsewhere.

### `InstructionKind` — 18 of 20 in scope covered

Covered: `Assignment`, `Do`, `Drop`, `Else`, `End`, `Exit`, `If`, `Iterate`,
`Leave`, `Nop`, `Numeric`, `Otherwise`, `Say`, `Select`, `Then`, `Trace`,
`When`, `WhenCase`.

* **MISSING: `Loop`.** Distinct from `Do` — the `LOOP` keyword is its own
  variant. No subset program uses `LOOP`.
* **MISSING: `Label`.** 4a executes it as a traced no-op; nothing constructs one.

The other 20 variants are owned elsewhere (9 in 4b, 4 in 4c, 6 in Phase 5, 1 in
Phase 7), matching criterion 1's own arithmetic.

### `ExprKind` — 7 of 9 in scope covered

Covered: `Literal`, `Constant`, `Variable`, `Stem`, `Compound`, `Prefix`,
`Binary`.

* **MISSING: `DotVariable`.** `.nil`, `.true`, `.false` are the three 4a admits.
* **MISSING: `Logical`.** The comma list in a condition, `if a, b then`, which
  is an AND of its parts and is the one of the six child-holding forms 4a
  evaluates.

`List` is constructed but is out of scope — see above. `Call`, `QualifiedCall`,
`ClassResolver`, `Message` and `VariableReference` are correctly absent.

### `LoopKind` — 4 of 5 in scope covered

Covered: `Simple`, `Forever`, `Count`, `Controlled`.

* **MISSING: `Over`.** The witness must be a **non-stem** target, both because
  `DO OVER` on a stem is an excluded deviation and because criterion 1 says so.
  Measured: `do i over 'abc'` iterates once yielding `abc`.

`With` is Phase 5's (`DO WITH` sends `SUPPLIER`).

### `PrefixOp` — 1 of 3 covered

Covered: `Minus`.

* **MISSING: `Plus`** — `say +5` gives `5`.
* **MISSING: `Not`** — `say \1` gives `0`.

All three are in scope; this is the enum with the largest proportional gap.

### `EndStyle` — 4 of 6 covered

Covered: `Do`, `LabeledDo`, `Loop`, `Otherwise`.

* **MISSING: `Select`** — a `SELECT` **without** `OTHERWISE`. Every `SELECT` in
  the subset has one. Needs a `WHEN` that matches, or the `END` raises 7.3.
* **MISSING: `LabeledOtherwise`** — a labelled `SELECT` that has an `OTHERWISE`.

Your prediction was right that this enum needed the hardest look, though it is
not the worst: `PrefixOp` and `Trace` are proportionally worse. No phase has
ever gated `EndStyle`, and it is the only enum here whose variants are decided
entirely by *block structure* rather than by a keyword, which is why reading for
it is unreliable.

### `Trace` — 1 of 4 covered, all four in scope

Covered: `Setting` (`TRACE R` and similar).

* **MISSING: `Default`** — bare `trace`. Measured: runs clean, rc 0, no trace
  output.
* **MISSING: `Value`** — `trace value 'N'`. Measured: rc 0, no extra output.
* **MISSING: `Skip`** — `trace 5`. Measured: **`Error 24.901`, "Numeric TRACE
  requests are valid only from interactive debugging.", rc 232**, with the
  clause echo `2 *-* trace 5` on stderr first.

`Skip` being a raiser rather than a silent no-op is what makes it testable at
all, and it has a consequence for whoever writes the witness: **it terminates
the program**, so it must be the last statement of its own program, and that
program's expectation includes two stderr lines and rc 232 rather than rc 0.

### `Operator` — 21 of 31 reachable covered

Covered: `Plus`, `Subtract`, `Multiply`, `Divide`, `IntDiv`, `Remainder`,
`Power`, `Abuttal`, `Concatenate`, `Blank`, `Equal`, `BackslashEqual`,
`GreaterThan`, `LessThan`, `GreaterThanEqual`, `LessThanEqual`, `StrictEqual`,
`StrictBackslashEqual`, `StrictGreaterThan`, `StrictLessThan`.

**MISSING, ten:**

| variant | spelling | measured |
|---|---|---|
| `BackslashGreaterThan` | `\>` | `1 \> 2` -> 1 |
| `BackslashLessThan` | `\<` | `1 \< 2` -> 0 |
| `StrictBackslashGreaterThan` | `\>>` | `'1' \>> '2'` -> 1 |
| `StrictBackslashLessThan` | `\<<` | `'1' \<< '2'` -> 0 |
| `StrictGreaterThanEqual` | `>>=` | `'1' >>= '1'` -> 1 |
| `StrictLessThanEqual` | `<<=` | `'1' <<= '1'` -> 1 |
| `LessThanGreaterThan` | `<>` | `1 <> 2` -> 1 |
| `GreaterThanLessThan` | `><` | `1 >< 2` -> 1 |
| `Or` | `\|` | `1 \| 0` -> 1 |
| `Xor` | `&&` | `1 && 0` -> 1 |

`Backslash` is the 32nd and is **unreachable in a `Binary` node by design** —
the spec says it is prefix-only and "correctly absent", and a `\` in dyadic
position is error 35.1. It must be excluded from the enumeration or carry an
owner string, or criterion 1 demands a witness that cannot exist. That is the
same shape as the `LoopKind::With` escape the criterion already handles.

---

## The programs that close the gaps

Three, and all three were put through the same analyser rather than reasoned
about, since "this obviously constructs X" is the guess this analysis exists to
replace. Measured together they construct **every** missing variant and no
out-of-scope `ExprKind`.

### Program A — `corpus/lang/prefix_dotvar_logical_over_label.rex`

Closes `InstructionKind::Loop`, `InstructionKind::Label`,
`ExprKind::DotVariable`, `ExprKind::Logical`, `LoopKind::Over`,
`PrefixOp::Plus`, `PrefixOp::Not`, `EndStyle::Select`, `Trace::Default`,
`Trace::Value`.

```rexx
trace
say +5
say \1
say .nil
if 1 = 1, 2 = 2 then say 'both'
do i over 'abc'; say i; end
select; when 1 = 1 then say 'sel'; end
lab: nop
trace value 'N'
loop 2; say 'lp'; end
```

Oracle, verified: `5`, `0`, `The NIL object`, `both`, `abc`, `sel`, `lp`, `lp`,
rc 0. Neither `TRACE` form emits trace output, so adding them does not disturb
the expectation. `do i over 'abc'` iterates **once** yielding the string itself,
and `lab:` produces nothing.

### Program B — `corpus/lang/comparison_operators_remaining.rex`

Closes the ten missing `Operator` variants and `EndStyle::LabeledOtherwise`.

```rexx
say 1 \> 2
say 1 \< 2
say '1' \>> '2'
say '1' \<< '2'
say '1' >>= '1'
say '1' <<= '1'
say 1 <> 2
say 1 >< 2
say 1 | 0
say 1 && 0
select label s
  when 1 = 0 then nop
  otherwise say 'lo'
end
```

Oracle, verified: `1 0 1 0 1 1 1 1 1 1` on separate lines, then `lo`, rc 0.

### Program C — `corpus/lang/trace_numeric_request.rex`

Closes `Trace::Skip`, and **must be its own program** because it terminates.

```rexx
say 0
trace 5
```

Oracle, verified: `0` on stdout, then on stderr

```
     2 *-* trace 5
Error 24 running <path> line 2:  Invalid TRACE request.
Error 24.901:  Numeric TRACE requests are valid only from interactive debugging.
```

and **rc 232**. Note the clause echo appears with trace off, the same shape the
spec records for 7.3, and the path is absolute — `normalize` masks the cwd, so
this is comparable.

### Measured effect of all three together

```
INSTR     DO END IF LOOP Label NOP OTHERWISE SAY SELECT THEN TRACE When
EXPR      Binary Constant DotVariable Literal Logical Prefix Variable
LOOP      Count Over
PREFIX    Not Plus
ENDSTYLE  LabeledOtherwise Loop Select
TRACE     Default Skip Value
OP        BackslashGreaterThan BackslashLessThan GreaterThanLessThan
          LessThanGreaterThan Or StrictBackslashGreaterThan
          StrictBackslashLessThan StrictGreaterThanEqual
          StrictLessThanEqual Xor
```

Every one of the nineteen missing variants, and no out-of-scope `ExprKind`.

## Summary for whoever acts on this

1. **Drop the three `List` programs from `phase-4a.txt`.** Ruled. They are a
   correctness blocker for criterion 1, not a coverage one, and re-measurement
   confirms removing them costs no coverage at all.
2. **Add the three programs above** to `corpus/lang/` and to `phase-4a.txt`.
   Being in `corpus/lang/` alone does not count; the criterion quantifies over
   the subset. Program C must stay separate, since `trace 5` terminates.
3. **Give `Operator::Backslash` an owner string** rather than a witness. Ruled.
4. **Add 24.901 to the recorded raiser families.** `trace 5` raises it and
   nothing in the spec lists it.
5. **Tell whoever writes the enumerating test** that variant identity must not
   come from `keyword()`, which conflates `When` and `WhenCase` — and that
   `EndStyle` is the enum a reading-based check fails on, because its variants
   are decided by block structure rather than by a keyword. `PrefixOp` at 1 of 3
   and `Trace` at 1 of 4 were the largest proportional gaps; `EndStyle` at 4 of
   6 was the least visible one.
