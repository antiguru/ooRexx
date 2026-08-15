# Task 14a report -- Phase 4a differential corpus

Scope: write and verify the Rexx programs for Phase 4a's corpus (this task),
and produce `rust/corpus/phase-4a.txt` naming the full 4a-eligible subset —
the existing programs plus these new ones. No Rust code touched; the harness
that runs the subset is a separate task.

Every program below was run under the oracle wrapped exactly as
`( ulimit -v 1048576; build/bin/rexx FILE )`, twice each, and the two runs'
stdout, stderr and exit code were diffed byte-for-byte. All 16 are
deterministic. Design was empirical throughout: several assumptions from the
brief turned out to be wrong in ways only running against the oracle caught
(see "Corrections" at the end) — this is consistent with the project's
pattern of pre-flights finding real defects.

**Follow-up round:** the team lead corrected Correction 1 below (partially —
see the amended text) and asked for one more program, `do_label.rex`,
covering the explicit `DO LABEL name` form that Correction 1's first draft
missed entirely. Added, verified the same way, and folded into
`phase-4a.txt` and `README.md`. Also expanded `README.md`'s segfault writeup
with the exact minimal repro, `rexxc`'s exit code, and a 3/3 determinism
check on both the crashing and non-crashing variants, per the team lead's
request that the report be self-contained for whoever eventually reads it.

## Plan

1. Read `do_variants.rex` and the existing 28 corpus programs; confirm which
   of the "10 known-clean" list are actually free of anything outside
   Phase 4a's scope, and check the other 18 too rather than trusting the
   list.
2. Write ~15 new programs covering: a 4a-only cut of `do_variants.rex`; four
   control-flow shapes that expose a mis-wired jump index (nested
   `LEAVE`, `ITERATE` from a nested `SELECT`, an `IF`/`ELSE IF` chain, and
   `SELECT`/`WHEN` bodies with side effects); the `WHEN`-as-`THEN`-of-`WHEN`
   parser trap; bare and named `LEAVE`/`ITERATE`; `DROP` (single-tail vs
   whole-stem); stem aliasing (three cases); `EXIT` with and without a
   value; number identity under `DIGITS`/`FORM`; the four comparison
   families; a deep expression; and `TRACE R`.
3. Verify every new program against the oracle: correct exit code, and two
   runs producing byte-identical stdout and stderr.
4. Write `rust/corpus/phase-4a.txt` (existing qualifying programs + all 15
   new ones) and fix the hard-coded program count in `README.md`.

## Existing corpus: which of the 28 qualify

Read all 28 files under `rust/corpus/lang/` and `rust/corpus/num/`. Confirmed
the 10 named in the brief are the full set that qualifies — every other file
uses at least one of `CALL`, `PROCEDURE`, `USE`, `SIGNAL`, `INTERPRET`,
`PARSE`, a builtin function call, a message send (`~`), or a `::` directive:

* Qualify (verified clean): `arith_digits.rex`, `no_trailing_newline.rex`,
  `select_when.rex`, `stem_compound.rex`, `trace_output.rex`,
  `num/comparison.rex`, `num/digits_rounding.rex`, `num/exponential.rex`,
  `num/notation_thresholds.rex`, `num/operators.rex`.
* Excluded, checked and confirmed why: `call_procedure.rex` (`CALL`,
  `PROCEDURE`, `USE ARG`), `condition_syntax.rex` (`SIGNAL`, `CONDITION()`),
  `gate_variants.rex` (`::` directives, message sends, `DO WITH`),
  `interpret_dynamic.rex` (`INTERPRET`), `keyword_as_variable.rex` (`PARSE`,
  `CALL`), `parse_template.rex` (`PARSE`), `primitive_classes.rex` (message
  sends), `source_arg.rex` (`PARSE`, `CALL`, builtins), `string_builtins.rex`
  (builtin calls), `whitespace_significant.rex` (`::routine`, builtin
  calls), `num/canonical_form.rex` (`.array~of`, message send),
  `num/datatype_num.rex`/`num/format_trunc.rex` (builtin calls),
  `num/errors.rex`/`num/settings.rex` (`CALL`, `INTERPRET`, `SIGNAL`),
  `num/form_notation.rex` (`form()` builtin), `num/fuzz.rex` (`fuzz()`
  builtin).
* `do_variants.rex` itself is excluded by its one `DO OVER .array~of(...)`
  line; `do_loop_forms.rex` below is the 4a-only cut of everything else it
  covers.

## The 16 new programs

### `do_loop_forms.rex`
Covers: `DO TO/BY`, a repetition count, `WHILE`, `UNTIL`, and inline
`ITERATE`/`LEAVE` — every non-`OVER` block of `do_variants.rex`, unchanged.
Oracle exit code: 0. Two runs: byte-identical stdout (14 lines), empty
stderr both times.

### `do_label.rex`
Covers: the explicit `DO LABEL name` form — added in the follow-up round.
`DO LABEL` on a plain non-repetitive block (`LEAVE` by that label exits it
early); `DO LABEL` on a controlled loop with `LEAVE` by that label from a
nested loop (the outer control variable keeps its value at the moment of
the `LEAVE`, e.g. `after leave-outer loop 1 2`); and `ITERATE` by that label
from a nested loop. This is the only program in the corpus that constructs
the parser's `Loop::label` field — none of the original 15 used `DO LABEL`.
Oracle exit code: 0. Two runs: byte-identical stdout (7 lines: `blk-a` /
`after blk` / `leave-outer 1 1` / `after leave-outer loop 1 2` /
`iterate-outer 1 1` / `iterate-outer 2 1` / `iterate-outer 3 1`), empty
stderr both times.

### `leave_nested_outer.rex`
Covers: nested `DO` where `LEAVE` names the outer loop's control variable,
built so a `LEAVE` wired to only the inner loop would visibly print more
than the correct wiring does. Oracle exit code: 0. Two runs: byte-identical
stdout (`inner 1 1` / `after outer`), empty stderr both times.

### `iterate_from_select.rex`
Covers: `ITERATE` from inside a `SELECT` nested in a `DO` loop. Oracle exit
code: 0. Two runs: byte-identical stdout (`keep 1` / `skip 2` / `keep 3` /
`keep 4`), empty stderr both times.

### `if_else_chain.rex`
Covers: an `IF`/`ELSE IF`/`ELSE IF`/`ELSE` chain with bodies of length 2, 1,
3 and 1 instructions, so a then-exit or false-target wired to the wrong
offset lands visibly in the wrong branch. Oracle exit code: 0. Two runs:
byte-identical stdout (11 lines), empty stderr both times.

### `select_when_bodies.rex`
Covers: `SELECT`/`WHEN` with multi-instruction bodies (3, 2, 1, 2
instructions) with visible side effects, so a wrong exit landing inside a
later `WHEN` shows up directly. Oracle exit code: 0. Two runs:
byte-identical stdout (12 lines), empty stderr both times.

### `select_when_absorption.rex`
Covers: `when 1 = 1 then` / `when 2 = 2 then n = 42` — the second `WHEN` is
never collected into the `SELECT`'s clause list, so with the first
condition true neither assignment runs and `n` stays 0. Oracle exit code: 0.
Two runs: byte-identical stdout (`0`), empty stderr both times. **Note:**
confirmed only for the true-first-condition case; see Corrections below for
why the false-condition variant is deliberately not in the corpus.

### `leave_iterate_variants.rex`
Covers: bare `LEAVE`, bare `ITERATE`, `LEAVE` naming the outer loop's
control variable, and `ITERATE` naming the outer loop's control variable,
in a matrix built so each form's output is distinguishable from what a
mis-wired (inner-loop-only) version would produce. Oracle exit code: 0. Two
runs: byte-identical stdout (24 lines), empty stderr both times.

### `drop_stem_tail.rex`
Covers: `DROP` of a single compound tail (tombstones just that tail —
`u.1` renders `U.1`, not the stem default) vs `DROP` of the whole stem
(clears the default too, so every tail reverts to its own tombstone name).
Oracle exit code: 0. Two runs: byte-identical stdout (`U.1` / `d` / `S.1` /
`S.2` / `S.3`), empty stderr both times.

### `stem_aliasing.rex`
Covers three cases, each measured against the oracle before being written
into the file (see Corrections): `b. = a.` shares the same underlying table
as `a.` (`a.1 = 2` afterward is visible through `b.1` too — both print `2`);
assigning a bare stem into a plain scalar (`u = r.`) copies its current
default as an ordinary value, so `drop r.` afterward does not affect `u`;
and an unset stem (`q.`) renders as its own upcased name including the
trailing period. Oracle exit code: 0. Two runs: byte-identical stdout
(`2 2 1 1` / `rd` / `R.1` / `Q.`), empty stderr both times.

### `exit_with_value.rex`
Covers: `EXIT` with an expression sets the process exit code. Oracle exit
code: 42 (both runs). Two runs: byte-identical stdout (`before`), empty
stderr both times.

### `exit_no_value.rex`
Covers: bare `EXIT` exits 0, same as falling off the end of the program.
Oracle exit code: 0 (both runs). Two runs: byte-identical stdout (`before`),
empty stderr both times.

### `number_identity.rex`
Covers: a number's rendering is fixed at creation for both `DIGITS`
(`numeric digits 9; y = 1/3; numeric digits 3; say y` prints
`0.333333333`) and `FORM` (`numeric form engineering; x = 1e10+0; say x`
prints `10E+9` and still prints `10E+9` after `numeric form scientific`).
Oracle exit code: 0. Two runs: byte-identical stdout (`0.333333333` /
`10E+9` / `10E+9`), empty stderr both times.

### `comparison_families.rex`
Covers all four comparison-operator families with the discriminating cases
from the brief: `' a' = 'a'` → 1, `'a b' = 'a  b'` → 0, `'10' >> '9'` → 0
vs `'10' > '9'` → 1, `'a' << 'a '` → 1, plus `1 = 1.0` → 1 vs `1 == 1.0` →
0, `' 1' == '1'` → 0, `'1.0' = '1'` → 1 vs `'1.0' == '1'` → 0. Every
expression is wrapped in `say (...)` because a bare expression statement
with no assignable left side is a command clause, which is out of scope.
Oracle exit code: 0. Two runs: byte-identical stdout (`1 0 0 1 1 1 0 0 1
0`, one value per line), empty stderr both times.

### `deep_nested_expr.rex`
Covers: a 3000-term `+`-chained expression (generated, not hand-typed —
see Corrections). Oracle exit code: 0. Two runs: byte-identical stdout
(`3000`), empty stderr both times.

### `trace_results.rex`
Covers: `TRACE R` output. Oracle exit code: 0. Two runs: byte-identical
stdout (`big` / `done 6` on stdout) and byte-identical stderr (the trace
lines), confirming trace output stays on stderr while `SAY` stays on
stdout.

## `phase-4a.txt` and `README.md`

`rust/corpus/phase-4a.txt` lists all 26 qualifying programs (10 existing +
16 new), one relative path per line. `README.md` gained a "Phase 4a subset"
section pointing at it, a "Phase 4a additions" table describing the 16 new
files, three "Things this corpus learned the hard way" entries (below,
expanded in the follow-up round), and the count-rot fix: "Expect `24
programs, 0 divergences`" now reads "Expect `N programs, 0 divergences` ...
where `N` is whatever the tool itself counts ... do not hard-code a number
here." The actual current count is 44 `.rex` files (28 existing + 16 new);
the README no longer asserts any specific number, since the old "24" was
already stale before this task (the corpus had already grown to 28 without
anyone updating it) — exactly the count-rot pattern this fix is for.

## Corrections found while verifying against the oracle

1. **`LEAVE`/`ITERATE` reject a *clause* label — but that turned out to be
   only half the story, per the team lead's correction.** First finding: a
   clause label immediately before the `DO` it's meant to name is rejected
   with error 47.2 ("Labels are not allowed within a DO/LOOP block"), and a
   clause label placed anywhere else compiles but `LEAVE`/`ITERATE` still
   refuse it with 28.3 ("must either match the label of a current loop or
   block instruction") — a clause label is a `SIGNAL` target, not a loop
   name. I read this as "`LEAVE`/`ITERATE` only accept a control variable,"
   which is true but incomplete: the explicit **`DO LABEL name`** form also
   works (`do label outer i = 1 to 3` / `leave outer`), and unlike a control
   variable it works on a plain non-repetitive block too. This matters
   beyond wording, because the parser's `Loop::label` field is set only by
   the `LABEL` keyword and Task 16's coverage criterion requires every
   field-bearing AST form to be constructed by at least one corpus program
   — none of the original 15 used `DO LABEL`, so that field was
   unconstructed until `do_label.rex` was added in the follow-up round.
   `leave_nested_outer.rex` and `leave_iterate_variants.rex` still use plain
   control variables (both are legitimate, independent forms), and
   `do_label.rex` now covers the `DO LABEL` form specifically.

2. **The `select_when_absorption.rex` false-condition variant segfaults the
   oracle; the team lead independently reproduced it and confirmed it is a
   run-time defect, not a parse one.** Minimal repro:
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
   `build/bin/rexxc` accepts this file and exits 0. Running it under
   `build/bin/rexx` segfaults deterministically: exit 139, 3/3 runs
   (confirmed independently in the follow-up round, not just by the team
   lead). Changing the first line's `2` to a `1` — so the first `WHEN`
   matches and control never reaches the orphaned second `WHEN` — removes
   the crash entirely: output `0`, exit 0, also checked 3/3 stable. That is
   exactly `select_when_absorption.rex` as shipped. This is a real defect in
   the oracle itself, in the fallthrough path past an unmatched `WHEN`
   specifically (not in parsing, not in `SELECT` generally). The team lead
   is surfacing it to their human partner with this repro; per their
   instruction, it stays unfiled and is not this task's decision. It is
   documented in `README.md`'s "learned the hard way" section with the
   minimal program, the `rexxc` result, and both determinism checks, and
   deliberately not built into a corpus program — the corpus's one rule is
   byte-identical output between the two interpreters under test, and a
   memory-safe Rust reimplementation neither can nor should reproduce a
   SIGSEGV.

3. **`deep_nested_expr.rex` first draft was 50 terms, not 3000.** Hand-typing
   `1 + 1 + 1 + ...` and eyeballing the count undercounted by a factor of
   60; the file's own header comment claimed 3000 while the program printed
   `50`. Caught only because the verification step actually reads the
   printed value instead of trusting the file. Regenerated the term list
   with a script instead of typing it by hand; now verified to print `3000`.

## Constraints observed

`NUMERIC DIGITS` never exceeds 9 in any new program (well under the 1000
ceiling). No file instantiates `.Package~new` or anything else via `~`. No
file under `interpreter/`, `samples/`, `build/`, or `ootest/` was modified.
All scratch/probe files used during verification live under
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/`
and are not part of the commit.
