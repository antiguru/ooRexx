# Task 2 review: the prefix operators as a native op

Reviewed `2a7daea6e..1ccb81199` against `task-2-brief.md` and `task-2-report.md`,
on the committed tree.

## Verdicts

**Spec compliance: pass.** Every interface the brief names is present with the
signature it names, every step was taken, and the two deviations the report
declares (`rexx-parse` touched; the golden test grown from one row to three) are
both improvements on the brief rather than departures from it. One deviation is
incompletely executed -- see finding 4.

**Code quality: changes requested.** The Rust is clean and I could not find a
behavioural defect in it: the error-path argument holds when read against
`error.rs` and every arm of `apply_prefix`, the trace split is byte-identical,
the adjacency assertion really checks all three properties, and the corpus
restatement is genuinely independent of `compile`. All seven new oracle stanzas
re-capture byte-for-byte. The findings are all in prose, and the first of them
is a flat falsehood in the case file's own statement of what its rows are worth
-- it names the wrong engine, and having named it, draws the wrong conclusion
about what the new row adds.

## What I verified independently

* **All seven new stanzas re-captured from the oracle**
  (`/home/moritz/dev/repos/ooRexx/build/bin/rexx`, fresh `mktemp -d` per run,
  stdout/stderr/status as three descriptors). Every one is byte-identical to the
  recorded rows, including `rc> 222`/`rc> 215` and the full `trace i` transcript
  with its two `>P>   "-"` lines in post-order. The captures are genuine.
* **`PrefixOp::spelling`'s "canonical" claim, measured.** The doc argues from the
  parse that `\`, `0xAA` and `0xAC` collapse to one spelling. I ran the oracle on
  `trace i ; say <0xAC>za` and `say <0xAA>za`: both print `>P>   "\" => "0"`. The
  claim is true of the oracle and not only of our parser. The two citations hold
  -- `scanner.rs:672`'s `b'\\' | 0xAA | 0xAC` arm falls through to
  `Operator::Backslash` at `:692`, and `expr.rs:801`'s
  `Operator::Backslash => PrefixOp::Not` is inside `message_subterm`
  (`expr.rs:788`).
* **The error path.** `Failure::Exited(Option<ObjRef>)` (`error.rs:1442`) is the
  only variant carrying an `ObjRef`; `Raised` carries `Vec<Substitution>` (bytes)
  and `Loud` carries no ref. `apply_prefix`'s arms reach exactly three failure
  constructions -- `arith_operand` (`eval.rs:870`, builds `Raised::nonnumeric`),
  `Raised::from(ArithError)` on `Number::zero().add/sub`, and
  `Raised::not_logical` -- and none is `Exited`. `Number::pow` is not on this
  path at all. The unconditional `pop_frame` is safe, and the substitution bytes
  are copied out of the operand before the pop.
* **The trace split is byte-identical.** `trace_intermediate` computes
  `let indent = self.clause_state.current_value_indent;` at its top and guards on
  `tracing_intermediates()`; `echo_prefix_op` recomputes the identical indent from
  the identical field and guards on the identical predicate
  (`tracing_intermediates()` *is* `trace_mode().intermediates`, `trace.rs:683`).
  `drive.rs`'s `Op::TracePrefix` arm calls the same `echo_prefix_op`. The two
  emissions cannot disagree about tag or indent.
* **`assert_prefix_echoes_follow_their_op`** is called unconditionally from
  `compile` (`compile.rs:712`), beside the other four, and its guard is
  `matches!(before, Op::Prefix { op: applied, dst, .. } if applied == echoed && dst == src)`
  at `at - 1` -- position, register **and** operator, as the brief required.
* **`corpus_shape_tests`'s restatement is independent.** `root_of` and `native`
  are local functions over `rexx_parse` types; the only `super::compile` call in
  the file is the one producing the chunk being compared against. `Root::Prefix`
  in the anti-vacuity list is real: the sweep passes with it there.
* **`const _: () = assert!(size_of::<Op>() == 12)`** is unchanged at
  `ir/mod.rs:65` and the workspace compiles, which is what that assertion is.
* **No new exhaustive `match` was missed.** Every site naming `Op::TraceOperator`
  has a `TracePrefix` counterpart, including `assert_region_ops_name_their_clause`'s
  `None` arm. `assert_trace_ops_open_a_clause_region` concerns only
  `Op::TraceClause` and needed nothing.
* **No task numbers, no "used to", no em-dashes** in any added line.
* **The `trace r` stanza does exercise a live gate.** `ChunkTrace` carries only
  `clauses`/`labels` (`trace.rs:252`), so `Op::TracePrefix` is compiled into the
  stream regardless of the trace setting and `echo_prefix_op`'s own early return
  is what suppresses the line on both engines. The report undersold this as
  "argued rather than measured"; the mechanism is checkable by reading.

## Findings, most severe first

### 1. The case file's header names the wrong engine, and then draws the wrong conclusion from it

`crates/rexx-exec/tests/ir_dual_cases/operators`, header:

> **A prefix operator's line is `>P>` and never `>O>`**, and those bytes
> are already pinned for the tree-walker by `tests/trace_oracle`'s own
> `prefix_operators` witness, which runs on that engine alone. What the `trace
> i` prefix row adds is the compiled clause: that an op stream emits the line
> at all, and where it falls among the operand's own lines.

Both sentences are false.

`check_witness` (`crates/rexx-exec/tests/trace_oracle.rs:229`) runs
`run_program(..., rexx_exec::Invocation::none())`. `Invocation::none()` sets
`engine: Engine::DEFAULT` (`invocation.rs:203`), and

```rust
pub const DEFAULT: Engine = Engine::Ir;   // invocation.rs:160
```

So the `prefix_operators` witness runs on the **compiled** engine, not the
tree-walker. `tests/corpus.rs:236` uses `Invocation::none()` too. Grepping
`Engine::TreeWalker` across `crates/rexx-exec/tests/` names only `ir_dual.rs`,
`spike.rs` and `collect_stress.rs` -- so this case file, through
`ir_dual.rs`'s `render_both_engines`, is very nearly the *only* thing in the
crate that pins the tree-walker's `>P>` bytes at all.

The consequence is that the credit is assigned exactly backwards. What
`trace_oracle`'s witness already pins is the compiled clause -- `say +5` under
`trace i` is a promoted clause emitting `Op::TracePrefix` after this task -- and
what the new `trace i` row adds is the tree-walker's bytes and the two engines'
agreement on them. The header tells a reader the opposite, and it is the
sentence a future task would rely on when deciding whether this row is
redundant.

The same error is in `task-2-report.md`'s mutation-3 paragraph
("`prefix_operators_covers_plus_and_backslash` and
`controlled_loop_covers_the_control_variables_own_value_lines` (both
`trace_oracle` witnesses, which run the tree-walker alone)"). The mutation
*result* is unaffected -- both engines route through `echo_prefix_op`, so both
tests catch it either way -- but the characterisation is wrong, and it is the
characterisation that got written into the committed header.

### 2. Stanza 1 names a row as discriminating that cannot discriminate

`ir_dual_cases/operators`, first new stanza:

> The two rows that could distinguish a sign flip on the operand's own digits
> from the subtraction `-` actually is are `-0`, which answers `0` and not
> `-0`, and `-'7.50'`, which keeps the operand's trailing zero -- measured,
> `say 0 - 0` and `say 0 - 7.50` give the identical `0` and `-7.50`.

Measured on the oracle: `0 - 7.50` is `-7.50`. A sign flip on the operand's own
digits is *also* `-7.50` -- keeping the trailing zero is what a sign flip does
most obviously of all. The `-'7.50'` row therefore cannot separate the two
implementations; it shows that they agree. (`-0` does separate them under a
textual sign flip, though not under a `Number` negate, which also answers `0`.)

The row that actually separates them is in the third stanza: `-12345` at
`numeric digits 1` answers `-1E+4`, where any sign flip answers `-12345`. The
comment has the evidence available two stanzas down and cites the wrong rows.
This is the "a comment asserting a separation its rows cannot make" shape the
brief flagged, and the numeral "two" is wrong along with it.

### 3. The `\` control row in the `NUMERIC DIGITS` stanza cannot fail

Header:

> A `\` row at the same setting is that stanza's control, because the text
> check reads no setting at all.

and the stanza:

> **The `\` row is the control**, and it says the setting cannot reach a text
> check.

The row is `say \0` under `numeric digits 1`. A `\` that *did* read `NUMERIC
DIGITS` would answer `1` here as well: the logical set is exactly `"0"` and
`"1"`, and no precision setting changes either one's rounded value or its
rendering. No implementation of the kind the comment contrasts changes this
row's output, so the row cannot redden for the stated reason and the sentence
claims something no row of that shape could show.

What actually says `\` is a text check and not a numeric one is the fourth
stanza (`\'abc'` -> 34.901, not 41.1), and the file already has it.

### 4. `lib.rs` still open-codes the three spellings the new method exists to own

`crates/rexx-exec/src/lib.rs:926-934`, in `form_name`:

```rust
        // The two operator forms name the operator, because "a dyadic
        // operator is not implemented" does not tell a reader which one to
        // go and implement. Both spellings are `&'static`.
        ExprKind::Prefix { op, .. } => {
            return format!(
                "the prefix operator `{}`",
                match op {
                    PrefixOp::Plus => "+",
                    PrefixOp::Minus => "-",
                    PrefixOp::Not => "\\",
                }
            );
        }
        ExprKind::Binary { op, .. } => {
            return format!("the operator `{}`", op.spelling());
        }
```

The `Binary` arm immediately below already calls `op.spelling()`; the `Prefix`
arm is now the last open-coded copy of the exact bytes `PrefixOp::spelling()`
answers, and it would read `op.spelling()` with no other change. The report's
justification for touching `rexx-parse` -- "Rather than a second copy of the
match (there was already one in `trace_intermediate` and one in `ast.rs`'s
test-only `shape()`...)" -- enumerates the existing copies and misses this one,
so the deviation's stated payoff ("one home for the three bytes") is not
actually collected.

For the record, the other two are correctly judged: `trace_intermediate`'s copy
is deleted by this diff, and `ast.rs`'s `shape()` is `#[cfg(test)]` and spells
them `u+`/`u-`/`u\` deliberately, so it genuinely cannot use the method. No
conflict there.

### 5. The `trace i` stanza claims a source-span discrimination its rows do not make

> The tags are the operators' own spellings, and `\`'s is the one an
> implementation reading a source span would get wrong.

Every `\` in the stanza is written as ASCII `\`, so the source span and the
canonical spelling are the same bytes in every row; a span-reading
implementation passes. The distinction is real -- I measured the oracle printing
`>P>   "\"` for both `0xAC` and `0xAA` prefixes -- but no row exercises it, and a
row that did would have to contain one of those bytes.

Contrast the sentence this one is modelled on, about the abuttal and the blank:
that one is true, because those tags have no source text at all.

### 6. The header cites a measurement the file cannot show

> at DIGITS 1 the oracle answers `-1E+4` for `-12345` and the identical `-1E+4`
> for `0 - 12345`.

I re-ran it: true. But `0 - 12345` at DIGITS 1 is not a row in this file -- the
report says the control probe was deliberately kept out. A reader cannot check
the file's central claim about `-` from the file. Adding the control as a row
would cost three lines and make the stanza self-supporting; as it stands the
comment leans on an uncommitted probe. Stanza 1's `say 0 - 0` / `say 0 - 7.50`
citation has the same shape.

### 7. Nit: the report misplaces the new method

`task-2-report.md` says `PrefixOp` gains `spelling()` "beside
`Operator::spelling()`". `Operator::spelling` is at `token.rs:243`; the new one
is at `ast.rs:82`, next to the `PrefixOp` enum. The placement is the right one
-- a method belongs next to its type -- but the symmetry argument as written
describes a file layout that does not exist.

### 8. Nit: a numeral doing quantifier duty

`PrefixOp::spelling`'s doc: "Canonical for [`Operator::spelling`]'s reason and in
the one case that has it". The set is named immediately afterwards (`\`), so
nothing rots, but "the case that has it" is the phrasing the rule asks for.

## Gates, re-run on the committed tree

Run from `rust/`, unpiped, exit statuses read directly rather than through a
pipe.

| command | exit | result |
| --- | --- | --- |
| `cargo test -p rexx-exec --lib` | 0 | 596 passed, 0 failed |
| `memcap 8G cargo test -p rexx-exec --test ir_dual` | 0 | 9 passed, 0 failed |
| `memcap 8G cargo test --workspace --no-fail-fast` | 0 | 82 binaries, 1461 passed, 0 failed, 4 ignored |
| `cargo fmt --all --check` | 0 | -- |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | -- |

The workspace totals match the report's exactly (82/1461/0/4). Statuses here were
read from a redirect with `$?` taken directly, never through a pipe, and the
per-binary lines were summed from the log rather than eyeballed.

Both new tests exist and run under their own names:
`ir::golden_tests::a_prefix_operator_compiles_to_a_native_op_and_its_own_echo`
and
`ir::corpus_shape_tests::every_corpus_body_compiles_the_minimum_promotion_set_to_its_own_ops`.

## Things I looked for and did not find

* No relaxation of the `Op` size assertion, and no widened payload.
* No second implementation of anything `eval.rs` owns: `Op::Prefix`'s driver arm
  enters `apply_prefix`, the same function `eval_prefix` enters, with the operand
  already in a register.
* No computing op that emits, and no echo carrying the wrong operator type.
* No test whose subject could be deleted leaving it green, apart from the `\0`
  control noted in finding 3. The golden test's three rows differ in the operator
  alone over a full-stream `assert_eq!`, which is what defeats a compiler that
  writes one fixed operator into every echo.
* No `REWRITE` use, no oracle-crashing form among the new programs, and no stanza
  whose recorded bytes disagree with the oracle.
