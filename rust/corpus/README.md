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

Expect `24 programs, 0 divergences` and exit 0. Substituting any other binary
for `--rs` should report 24 divergences and exit 1; that negative control is
what makes a zero meaningful.

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
| `notation_thresholds.rex` | the exact positive and negative E-notation boundaries |

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
