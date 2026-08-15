STATUS: DONE

# Task 7 report: expression evaluation, part one

## Pre-flight reading

`task-7-brief.md`; the spec's "Expression evaluation", "Control flow" (to
know what's NOT this task), and "Errors, and the reporting subsystem" in
full; the current `rexx-exec/src/lib.rs` (`eval`/`eval_node`, `Code<'a>`,
`Loud`, `form_name`, `step`, `run_activation`, `run`, `execute`) at HEAD
(`7a628261`); `rexx-num`'s `Number::add/sub/mul/div/pow`, `ArithError`
(`code`/`additional`/`message`), `DivOp`.

## One thing checked and dissolved: `body: &CodeBody` vs `code: &Code<'_>`

The brief's signature reads `fn eval(&mut self, body: &CodeBody, expr:
&Expr) -> Result<ObjRef, Raised>`, but the tree's `eval` (built by Task 3,
untouched since) takes `code: &Code<'_>`. This is not a new conflict:
`Code`'s own doc comment already names this exact abstraction gap --
"This is the design's `fn eval(&mut self, body: &CodeBody, expr: &Expr)`
with the two things a body needs beyond its instructions folded in" --
i.e. the spec states the signature at the abstract level and `Code<'a>`
is Task 3's concrete answer, already reconciled. Keeping `code: &Code<'_>`
as-is; not touching this.

## The real open question: `Raised` does not exist yet, and `eval`'s own
## return type names it

`Raised` is the spec's error type ("Errors, and the reporting subsystem"):
"carries the condition name, the number and sub-number, and the
substitution values." The crate layout puts it in `src/error.rs`, and the
plan's own Task 12 ("Errors, the message catalogue, and the exit code")
is the task that creates that file and delivers "a message catalogue for
4a's four raiser families... Only arithmetic's text exists today" (true:
`rexx-num::ArithError` already has `code()`/`additional()`/`message()`).
Task 12 has not run.

This is not the same shape as Task 6's `blocks: Vec<Block>` question,
where the field could simply be omitted because nothing needed it yet.
Here, `Raised` is the literal return type of the one function this task's
Interfaces line names, and arithmetic is explicitly one of "4a's raisers"
in the spec's own list -- so `1/0` and `'abc' + 1` are real cases Task 7's
own tests will want to exercise, and `Loud` is the wrong type for them by
the codebase's own stated distinction ("Not a Rexx condition and never
convertible into one... Task 12 gives the real errors their own type,
Raised, which is a different thing entirely"). Encoding an arithmetic
error as `Loud` would give it `NOT_IMPLEMENTED_EXIT` (120) instead of the
oracle's `256 - 42` for a divide-by-zero, which is not merely imprecise,
it is the wrong exit code family entirely.

Two live options, and the choice changes more than `eval.rs`:

1. Build a minimal, real `Raised` now (a new `error.rs`, holding just
   `condition`/`number`/`sub`/`additional`, with a `From<ArithError>`
   or similar), thread it through as the one place a real condition
   surfaces, and give `step`/`run_activation`/`run`/`execute` a way to
   carry *either* `Loud` or `Raised` up to the top (a small enum, or two
   fields, or `Loud` growing a `Raised(Raised)` variant). Task 12 then
   extends this `Raised` with the other raiser families and the exact
   oracle-matching two-line render + exit-code map, rather than inventing
   the type from nothing. This is real, incremental building rather than
   throwaway scaffolding, because the struct's *shape* is already
   reasonably pinned by the spec and won't need to change under Task 12,
   only grow a renderer around it.
2. Keep `eval` returning `Result<ObjRef, Loud>` for this task, treat every
   arithmetic error as a loud failure for now (wrong exit code, flagged
   honestly as a known gap Task 12 closes), and defer `Raised`'s
   existence entirely.

Recommendation: (1). Option (2) means a divide-by-zero test can only
assert "fails loudly", not "raises 42.003 with rc 214", which is a real
gap in what this task can prove given arithmetic errors are explicitly
in its own spec section's raiser list. Sent before writing `eval.rs`,
since this changes `lib.rs`'s existing `step`/`run_activation`/`run`/
`execute` chain, not only the new file.

## Two decisions carried in from the last round, applied without re-asking

* **The `u32`/`u64` `DIGITS` narrowing** (flagged by Task 4, mine to
  decide): `number()` takes `created_digits: u32`, `Settings::digits()`
  returns `u64`. Deciding to **saturate** (`u32::try_from(digits)
  .unwrap_or(u32::MAX)`) rather than reject or panic: no corpus program
  goes near `DIGITS` anywhere close to `u32::MAX` (4.29 billion), so the
  cheap answer is also the honest one, and saturating rather than
  panicking keeps a pathological `NUMERIC DIGITS 99999999999` from
  aborting the interpreter over a value nothing in-scope will ever set to
  that range on purpose. Will state this at the call site, not bury it.
* **Comparison stays out.** The brief's own arithmetic list is
  `+ - * / % // **` plus concatenation; comparison (`=`, `==`, etc.) is
  explicitly Task 8's, waiting on `rexx-num`'s byte-slice `compare` entry
  point. Not reaching for `compare` anywhere in this task.

## Happy-path oracle transcripts, gathered while waiting for the answer
## (needed regardless of which option is chosen)

One program, all under `( ulimit -v 1048576; build/bin/rexx FILE )`:

```
say 123        -> 123          say 2*3        -> 6
say 1e5        -> 1E5          say 7/2        -> 3.5
say -5         -> -5           say 7%2        -> 3
say +5         -> 5            say 7//2       -> 1
say \1         -> 0            say 2**3       -> 8
say \0         -> 1            say 'a'||'b'   -> ab
say 1+2        -> 3            say 'a' 'b'    -> a b   (Blank)
say 5-3        -> 2            x='a'; say x'b' -> ab   (Abuttal)

w.='wd'; say w.  -> wd     (ExprKind::Stem through eval)
a.1='x'; say a.1 -> x      (ExprKind::Compound through eval)
say .nil   -> The NIL object
say .true  -> 1
say .false -> 0
```

All match the spec's stated rules exactly (`\` as prefix logical-not on a
0/1 operand, `Abuttal` and `Blank` both concatenating like `||`).

## Error transcripts, oracle-verified (the reasoning that shaped `Raised`)

All under `( ulimit -v 1048576; build/bin/rexx FILE )`.

* `say 1/0` -> Error 42.3, rc 214.
* `say 1//0` -> Error 42.3, rc 214 (the same `DivideByZero`).
* `say 'abc'+1` -> Error 41.1, `Nonnumeric value ("abc")`.
* `say 2**'x'` -> Error 26.8, `found "x"`.
* `say 2**2.5` -> Error 26.8, `found "2.5"` -- confirms the same 26.8 fires
  for a value that *parses* but is not whole, not only for one that does
  not parse at all.
* `say 'y'**2` -> Error 41.1 (the **base's** ordinary nonnumeric path).
* `say 'y'**'x'` -> Error 41.1 (base checked before exponent, still).
* `say \'abc'` -> Error 34.901, `found "abc"`.

The `**` asymmetry (exponent failures unify under 26.8 including "not a
number at all"; base failures are always the ordinary 41.1; base checked
first) is not a simplification, it is the fact `eval_arithmetic`/
`Raised::power_exponent_not_whole` exist to reproduce -- verified before
writing any code, not after a test failed.

## Implementation

* `rust/crates/rexx-exec/src/error.rs` (new): `Raised` (condition/number/
  sub/additional -- payload only, no message catalogue, no two-line
  stderr format, no exit-code map, per the team lead's explicit
  boundary), five constructors (`nonnumeric` 41.1, `power_exponent_not_
  whole` 26.8, `not_logical` 34.901, plus the private `syntax` helper),
  and `From<ArithError> for Raised`. `Failure` (`Loud | Raised`) is the
  one type `step` and everything above it now propagate.
* `rust/crates/rexx-exec/src/eval.rs` (new): `eval`/`eval_node`/
  `stack_span` moved from `lib.rs` unchanged in logic, extended with
  `Stem` (merged into the `Variable` arm -- same slot read, D15a's own
  rule), `Compound` (via `compound_parts` + `tail_key` + `stem_get`),
  `DotVariable`'s three names, `Prefix`, the seven arithmetic operators,
  and `Abuttal`/`Blank` beside `||` (one shared `concat` helper,
  separator `b""` or `b" "`). `arith_operand` centralises the
  to-number-or-41.1 conversion every arithmetic operator and prefix `+`/
  `-` shares.
* `rust/crates/rexx-exec/src/lib.rs`: `mod error;`/`mod eval;` added;
  `step`/`step_in_temps_frame`/`run_activation`/`run`/`run_fragment` all
  changed their error type from `Loud` to `Failure` (three direct `Loud`
  constructions needed an explicit `.into()`, since `?` only applies one
  level of `From` and these are not behind `?`); `execute`'s top-level
  match gained a `Failure::Raised` arm, printing the condition/number/
  sub/additional without pretending to be the oracle's format (matching
  the existing "wrong in the details, right in never being mistaken for
  success" rule the parse-error arm already followed).
* `rust/crates/rexx-exec/src/value.rs`: removed the two `#[allow(dead_code,
  ...)]` on `number`/`to_number`, now genuinely called by `eval.rs`, and
  updated the one doc comment that said "raising is a later task's job" to
  name `eval.rs`/`Raised::nonnumeric` now that this task is that later
  task.
* `rust/crates/rexx-exec/tests/spike.rs`: two pre-existing tests needed
  fixing, both natural consequences of implementing arithmetic rather than
  defects in this task's own code --
  - `a_loud_failure_message_does_not_grow_with_the_expression` used `+` as
    its example of "a form Task 3's spike does not evaluate"; since Task 7
    now evaluates `+`, `say 1 + 1` succeeds instead of failing loudly.
    Switched to `=` (comparison), still Task 8's.
  - `records_the_stack_cost_of_one_eval_frame`: see below.

## A real finding: `eval_node`'s stack frame roughly doubled, and the
## existing test caught it correctly

Measured on the **unchanged** `'a'||''||''...` (100,000-term) stress
program: 784 -> **1600 bytes per `eval` level in debug**, 192 -> **624 in
release**. `eval_node` grew from four match arms to fifteen; in an
unoptimised debug build the compiler does not appear to reuse stack slots
across mutually exclusive match arms as aggressively as release does, so a
dispatch function's own frame size tracks how many forms it *names*, not
only what the one arm actually taken does -- confirmed rather than
guessed, by re-running the concatenation-only stress test whose own logic
never changed and watching the number move anyway.

The test's own comment anticipated exactly this ("anything above a
kilobyte per frame would mean the stack size below needs recomputing
rather than the test relaxing"), so I did not just widen the bound.
Recomputed: survivable depth at 1600 bytes/level is `512 MiB / 1600 ≈
335,000`, still more than three times D19's 100,000 minimum, so
`INTERPRETER_STACK_BYTES` stays at 512 MiB rather than growing to chase a
number that will keep moving as Task 8 and later tasks add more `ExprKind`
arms. Recorded a new paragraph in `INTERPRETER_STACK_BYTES`'s doc comment
(not a rewrite -- the existing table and its own "measurement was correct
when taken" framing stay, exactly as that comment's own history already
shows happening once before) and raised the test's sanity bound from 1024
to 4096, both with the reasoning above rather than a bare number change.
**This is a probe reading, not a re-bisection** -- said explicitly in both
places, since the earlier table's own two rows for `eval` (783 bisected,
784.0 probed) only agreed to within 0.2% by actually doing the bisection,
and Task 11 is who needs to confirm that still holds at this size when it
sets the real depth limit.

## Tests

`cargo test -p rexx-exec`: **37 unit tests** (17 new in `eval::tests`,
20 unchanged from Tasks 4-6), plus the 10 `tests/spike.rs` integration
tests (2 updated as above, 8 unchanged) and 2 doctests, all passing.
`cargo test -p rexx-exec --release`: same 37 + 10 + 2, all passing (this
is the run Step 4 asks for specifically, since the temps discipline is
what `debug_assert!` cannot check). `cargo clippy -p rexx-exec
--all-targets -- -D warnings`: clean. `cargo fmt -p rexx-exec -- --check`:
clean after one `cargo fmt` pass. `cargo build --workspace --exclude
rexx-parse`: clean, confirming nothing outside this crate broke.

One test-writing mistake caught before it shipped, worth recording:
`eval_text`/`eval_source` always call `activate`, which pushes a *fresh*
frame. A test that needs to bind state first (`stem_assign`, `settings`)
and then evaluate against it cannot call `eval_source` afterward --
pushing a second activation shadows the first, and the bound state
becomes invisible. This is the exact same trap Task 5's
`a_multi_level_tail_joins_its_pieces_with_a_period` test hit and fixed;
four of my own first-draft tests hit it independently before I generalised
the fix into `eval_in_place`/`eval_in_place_text`, which evaluate against
whatever activation is already on top rather than pushing a new one.

Also caught before it shipped: a planned Abuttal test used `'a'('b')`,
which measured against the oracle as `Error 43.1, Could not find routine
"a"` -- a quoted literal directly followed by `(...)` is call syntax
(`CallTarget::Literal`), not concatenation. Replaced with the actual
Abuttal shape (`x'b'`, a variable directly followed by a literal),
matching the oracle transcript already in this report.

## Resolved while finishing this report: `ArithError::sub_code()` accessor

Landed as `4a320f1c` (`Make ArithError::sub_code public, and pin the pow
asymmetry that is rexx-num's to own`) while this task's implementation was
otherwise complete and pending its report. `error.rs`'s `From<ArithError>`
originally shipped the flagged two-variant stopgap (hand-mapping only
`DivideByZero` and `PowerExponentNotWhole`, `sub: 0` elsewhere) exactly as
proposed; once the accessor landed I deleted the stopgap match and
replaced it with `error.sub_code()` directly, so every `ArithError`
variant now carries its real sub-number, not only the two this task's own
tests exercise. Re-ran the full suite after the swap (below) -- no
behaviour change for the two verified variants, and the other six are no
longer a documented gap.

## Tests, re-verified after the `sub_code` swap

`cargo test -p rexx-exec`: 38 unit tests (the 39th-looking count includes
one test from an unrelated, concurrently in-progress `plan.rs` change
sitting uncommitted in this shared worktree -- see "Working-tree hygiene"
below; the count from this task's own code is unchanged) + 10 integration
+ 2 doctests, all green in debug. `cargo test -p rexx-exec --release`:
same, all green. `cargo clippy -p rexx-exec --all-targets -- -D warnings`:
clean. `cargo fmt -p rexx-exec -- --check`: clean. `cargo build --workspace
--exclude rexx-parse`: clean.

## Working-tree hygiene: `plan.rs` is not this task's

At commit time `git status` also showed `rust/crates/rexx-exec/src/plan.rs`
modified, unstaged, and unrelated to Task 7 -- a large, apparently
in-progress rewrite of `Plan::build`/`Plan::note` toward an exhaustive
match over every `InstructionKind` variant (its own doc comment names this
as a fix for the original `_ => {}` catch-all, which left stems and
compounds unregistered). Nothing in this task's brief or Interfaces
touches `plan.rs`, and the diff's own voice and dated reasoning (citing
`SymbolId::index()` at `180875a9` as already landed, same as this task's
own report) reads as a concurrent, separate piece of work rather than
anything left over from Task 6. Left untouched and unstaged: not added to
this commit, not reverted, not evaluated for correctness -- it is not this
task's to judge or to claim. Flagged to the team lead below so it is not
lost track of in a shared, non-worktree-isolated tree.

## Commit

`3a9d6446`, "Task 7: eval grows terms, arithmetic and concatenation, and
Raised propagates". Exactly the five files touched: `src/error.rs`,
`src/eval.rs` (both new), `src/lib.rs`, `src/value.rs`, `tests/spike.rs`
(all modified). `src/plan.rs`'s unrelated concurrent change was left
unstaged, per "Working-tree hygiene" above.
