# L0 differential corpus

Each `.rex` file here is run under both interpreters by `rexx-diff`, and their
normalised stdout, stderr, and exit code must match exactly.

## The one rule: determinism

A program here must produce byte-identical output on every run of the *same*
interpreter. No `DATE()`, no `TIME()`, no process IDs, no file system state, no
directory listings, no addresses, no iteration over an unordered collection.

When the self-test (`--cpp X --rs X`, the same binary twice) reports a
divergence, the corpus is at fault, not the differ. **Fix the program; never
loosen `normalize` to make it pass** — everything normalisation strips is a
class of divergence the project can no longer detect.

## Running it

```sh
cargo build --release
./target/release/rexx-diff \
    --cpp ../build/bin/rexx --rs ../build/bin/rexx --corpus corpus
```

Expect `N programs, 0 divergences` and exit 0, where `N` is whatever the tool
itself counts (`find corpus -name '*.rex' | wc -l` agrees with it) -- do not
hard-code a number here, since it goes stale every time a program is added or
removed and nothing catches the drift. Substituting any other binary for
`--rs` should report `N` divergences and exit 1; that negative control is what
makes a zero meaningful.

## Phase 4a subset

Phase 4a's executor implements a subset of the language -- no builtin
functions, no `CALL`/`INTERPRET`/`PARSE`, no message sends, no `::`
directives, no `DO WITH` or `DO OVER` on a stem. `phase-4a.txt` lists exactly
the programs in this directory that stay inside that subset, so Task 14's
harness has something narrower to run than "every `.rex` file here" while
Phase 4a is still the only thing built. It is a plain list, one path per
line, relative to this directory; rebuild it by hand whenever a program's
scope changes rather than trusting a stale copy.

## Current programs

| File | Covers |
|---|---|
| `arith_digits.rex` | `NUMERIC DIGITS`/`FORM`, division, `**`, `//`, `%`, exponential formatting |
| `parse_template.rex` | `PARSE` with string, literal, positional and `UPPER` templates |
| `condition_syntax.rex` | `SIGNAL ON SYNTAX`, `RC`, `CONDITION()` |
| `do_variants.rex` | `DO TO/BY/count/WHILE/UNTIL/OVER`, `ITERATE`, `LEAVE` |
| `select_when.rex` | `SELECT`/`WHEN`/`OTHERWISE`, `SELECT CASE` |
| `call_procedure.rex` | `CALL`, `PROCEDURE`, `EXPOSE`, `USE ARG`, `ARG()`, recursion |
| `stem_compound.rex` | stem defaults, compound variables, tail substitution, `DROP` |
| `interpret_dynamic.rex` | `INTERPRET` of statements and assignments |
| `string_builtins.rex` | 20 string builtins including the `C2X`/`X2C`/`D2X`/`X2D` conversions |
| `trace_output.rex` | `TRACE I` output formatting, which is observable |
| `source_arg.rex` | `PARSE SOURCE`, `SOURCELINE()`, `ARG()` option forms |
| `primitive_classes.rex` | `~id` of every class reachable as an environment symbol |
| `whitespace_significant.rex` | `f(x)` vs `f (x)`, abuttal forms, and the empty-binary-literal trap |

### Phase 4a additions -- executor control flow

Written for Task 14a because, of the programs above, none exercises `LEAVE`
or `ITERATE` and most exercise the arithmetic core rather than the
executor's control flow. Every file below stays inside what Phase 4a's
executor implements (see `phase-4a.txt`): no builtin function calls, no
`CALL`/`RETURN`/`PROCEDURE`/`USE`/`SIGNAL`/`RAISE`/`INTERPRET`, no
`PARSE`/`ARG`/`PULL`/`PUSH`/`QUEUE`, no message sends, no `::` directives, no
command clauses, no `DO WITH`, and no `DO OVER` on a stem.

| File | Covers |
|---|---|
| `do_loop_forms.rex` | `do_variants.rex` minus its one `DO OVER` line -- `TO`/`BY`, a repetition count, `WHILE`, `UNTIL`, inline `ITERATE`/`LEAVE` |
| `do_label.rex` | the explicit `DO LABEL name` form -- on a plain block and on a controlled loop, `LEAVE`/`ITERATE` by that label from a nested loop; the only program constructing `Loop::label` |
| `leave_nested_outer.rex` | nested `DO` with `LEAVE` naming the outer loop's control variable |
| `iterate_from_select.rex` | `ITERATE` from inside a `SELECT` nested in a loop |
| `if_else_chain.rex` | an `IF`/`ELSE IF`/`ELSE` chain with different-length bodies, so a wrong then-exit or false-target is visible |
| `select_when_bodies.rex` | `SELECT`/`WHEN` bodies several instructions long, so a wrong exit lands inside a neighbouring `WHEN` |
| `select_when_absorption.rex` | a `WHEN` whose `THEN` instruction is itself a `WHEN` clause -- it is never collected into the `SELECT`'s list |
| `leave_iterate_variants.rex` | bare and outer-naming `LEAVE`/`ITERATE`, matrixed so mis-wiring either one is visible |
| `drop_stem_tail.rex` | single-tail vs whole-stem `DROP`, including the tombstone rendering of a dropped compound |
| `stem_aliasing.rex` | `b. = a.` shares a.'s table; assigning a bare stem into a scalar copies its default; an unset stem renders its own name |
| `exit_with_value.rex` | `EXIT` with an expression sets the process exit code |
| `exit_no_value.rex` | bare `EXIT` exits 0 |
| `number_identity.rex` | a number's `DIGITS`/`FORM` are fixed when it is created, not when it is displayed |
| `comparison_families.rex` | the four comparison-operator families (simple, strict, simple ordering, strict ordering) and the cases that tell them apart |
| `deep_nested_expr.rex` | a 3000-term expression, deep enough to matter and far below the oracle's ~150,000-term stack cliff |
| `trace_results.rex` | `TRACE R` output on stderr, distinct from `SAY`'s stdout |

### `num/` — Phase 2, the numeric core

| File | Covers |
|---|---|
| `digits_rounding.rex` | `NUMERIC DIGITS` 1-12 and 40; round-half-up at the boundary |
| `form_notation.rex` | `SCIENTIFIC` vs `ENGINEERING`; `FORM()` |
| `operators.rex` | `+ - * / % // **` across sign combinations |
| `comparison.rex` | numeric vs strict comparison, `<<` and `>>` |
| `fuzz.rex` | `NUMERIC FUZZ` altering `=` but not `==` |
| `format_trunc.rex` | `FORMAT()` in all argument forms, `TRUNC()` |
| `datatype_num.rex` | `DATATYPE` `N`/`W`/default |
| `exponential.rex` | E-notation thresholds, which are asymmetric |
| `errors.rex` | error numbers 42, 41 and 26 |
| `canonical_form.rex` | trailing-zero preservation; zero collapsing |
| `notation_thresholds.rex` | the E-notation boundaries, which use *different* exponents per side |

### `expr/` — Phase 3, the expression grammar

Not a `rexx-diff` corpus. `precedence.tsv` is a table of generated expressions
with the value or error number `build/bin/rexx` gave for each, read by
`rexx-parse`'s differential test so that `cargo test` needs no built C++
interpreter. Its own header records how it was generated and what the columns
mean.

### `errors/` — Phase 3, the parse-error gate

Not a `rexx-diff` corpus either.
`parse-errors.tsv` holds one row per program with the answer `build/bin/rexxc` gave for it: the major number, sub-number and reported line when it refused, and `ok` when it translated.
`rexx-parse`'s `tests/errors.rs` reads it and checks both directions, so a parser that rejected nothing and one that rejected everything both fail.

The `class` field is the one worth knowing about.
It says whether the oracle refused to *translate* the program or rejected it later, while installing the package, and that is recorded per row rather than derived from the error number.
The two install-time classes are `98.903` and `90.999`, so a rule keyed on a `98.9xx` prefix would have covered half of them and read as correct.

Nothing in the file is our parser's own answer.
That is the point: a row records only what the oracle said, so a divergence shows up as a test failure rather than as an expected value.
The header records how the rows were collected, what each class means, and what the escaping covers.

## Things this corpus learned the hard way

`say a"|"b` does not concatenate three values. `"|"b` is read as a **binary
string literal** — the `b` suffix binds to the preceding quote — and the
program dies with error 15.4. `parse_template.rex` uses explicit `||` for
exactly this reason.

Worse, the same rule usually fails *silently*. `say a''b` looks like the
classic idiom for blank-free concatenation but prints `x`, not `xy`: `''b` is
an empty binary literal, so the line concatenates `a` with `""` and never
reads `b`. This was written into `whitespace_significant.rex` as a comment
asserting the wrong output, and only caught by running it.

`.integer` is not an environment symbol; `Integer` is internal and unexposed.
`.rexxinfo` is an *instance*, not a class, so it has no `~id`. Both were in
the first draft of `primitive_classes.rex` and both failed.

`LEAVE name`/`ITERATE name` accept two different kinds of name, and a
*clause* label is neither of them. `outer: do i = 1 to 3` then `leave outer`
fails, and it fails the **same way** regardless of where the clause label
sits relative to the `DO` it was meant to name: `rexxc` accepts the program
(it is not a translate-time error), and at run time `LEAVE`/`ITERATE` refuse
the name with error 28.3 ("must either match the label of a current loop or
block instruction") — a clause label is a `SIGNAL` target, not a loop name,
and 28.3 does not distinguish "wrong kind of name" from "right kind, wrong
loop". Error 47.2 ("Labels are not allowed within a DO/LOOP block") is
unrelated: it is a translate-time rejection of a clause label written
*inside* a loop's body, between `DO` and `END`, which has nothing to do with
naming the loop at all. (An earlier revision of this entry conflated the
two — measure the claim that only lands in documentation, not just the one
that decides what a program contains.) What does work is either the loop's
own **control variable** (`do outer = 1 to 3` / `leave outer`, used
throughout `leave_nested_outer.rex` and `leave_iterate_variants.rex`) or the
explicit **`DO LABEL name`** form (`do label outer i = 1 to 3` / `leave
outer`, which also works on a plain non-repetitive block and is the only
corpus program that constructs the parser's `Loop::label` field — see
`do_label.rex`). First draft of `leave_nested_outer.rex` used a clause label
and failed with 28.3 before this was measured.

A `WHEN` whose `THEN` instruction is itself a `WHEN` clause
(`select_when_absorption.rex`) parses and is accepted, and the second `WHEN`
is never added to the enclosing `SELECT`'s clause list — it is silently
swallowed as the first `WHEN`'s (empty) consequence. That is confirmed only
for the case where the first `WHEN`'s condition is *true*, which is all the
shipped program uses. The *false* variant is an orphaned-`WHEN` landmine, not
a corpus program, and this is its full writeup:

```rexx
n = 0
select
  when 1 = 2 then
    when 2 = 2 then n = 42
  otherwise
    n = 99
end
say n
```

`build/bin/rexxc` accepts this file and exits 0 — it is not a parse defect.
Running it under `build/bin/rexx` segfaults deterministically: exit 139,
3/3 runs (measured 2026-07-30). Changing only the first line's `2` to a `1`
(so the first `WHEN`'s condition is true, control never reaches the orphaned
second `WHEN`, and this is exactly `select_when_absorption.rex`) removes the
crash entirely — output `0`, exit 0, also 3/3 stable. So the crash is
specifically in the fallthrough path past an unmatched `WHEN` whose `THEN`
was itself absorbed, not in parsing or in the SELECT construct generally.
This is a real defect in the oracle itself. It is being surfaced upstream by
the team lead's human partner with this repro; it is deliberately not built
into a corpus program, because the corpus's one rule is byte-identical
output between the two interpreters under test, and a memory-safe Rust
reimplementation neither can nor should reproduce a C++ segfault.
