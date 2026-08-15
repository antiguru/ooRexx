# Task 4 report: the value model

Status: **DONE.**

## Pre-flight reading

`task-4-brief.md`; the plan's Task 4 and Task 5/6/7 (to see what is explicitly
assigned elsewhere); the spec's D15 and D15a in full; `progress.md` for what
Tasks 1-3b/3c have already landed; `rexx-core/src/lib.rs`, `body.rs`,
`handle.rs` (`ObjRef`, `Decoded`, `SMALL_INT_MIN/MAX`, `Body::Text`/`Num`,
`NotNumeric`); `rexx-num/src/lib.rs`, `settings.rs`, `format.rs` (`Number`,
`Settings`, `Form`, `format_form`, `add`/`sub`/`mul`/`div`); `rexx-parse/src/
lib.rs`'s public surface; `rexx-exec/src/lib.rs` in full (the ~1100-line
spike from Task 3).

## Question sent before implementing

The brief's Step 1 test code (verbatim from the plan) calls
`Interp::new()`, `interp.settings_mut()`, `interp.eval_str("1 / 3")`,
`interp.eval_with("x + 6", &[("X", x)])`. None of the four are in Task 4's
own "Interfaces" list (`text`, `number`, `to_text`, `to_number` on `Interp`),
and none exist in the tree today:

* `Interp` is a private, non-`pub` `struct Interp` in `rexx-exec/src/lib.rs`
  (Task 3's spike). An external test in `tests/value.rs` cannot name it at
  all, let alone call a no-arg `new()` -- today's is
  `fn new(interpret_spike: bool) -> Interp`.
* `eval_str`/`eval_with` need to parse and evaluate a bare Rexx expression
  against live interpreter state. `rexx-parse`'s only run-time parse entry
  point is `parse_interpret(Vec<u8>) -> Result<Fragment, ParseError>`, which
  parses a sequence of *instructions*, not a bare expression; there is no
  `parse_expr`. So the transcripts (`"1 / 3"`, `"1e10 + 0"`, `"x + 6"`,
  `"15 + 6"`) can only run through an assignment fragment plus a variable
  read, or a new expression-only parser entry point.
* All four transcripts that need arithmetic (`/`, `+`) need `Binary` operator
  evaluation through `rexx-num`'s `add`/`div`. Task 3's spike only evaluates
  `Literal`, `Constant`, `Variable` and `||` (`Concatenate`) --
  `eval_node`'s match has no arithmetic arm at all, and every other form
  fails loudly via `Loud::expression`. The plan's own Task 7,
  "Expression evaluation, part one -- terms, arithmetic, concatenation", is
  named as where "arithmetic `+ - * / % // **` through `rexx-num`" is built,
  and it is sequenced *after* Task 6 (the resolution plan / `Activation`
  with `Settings`).
* `settings_mut()` implies a `Settings` reachable with **no activation on
  the stack** (`Interp::new()` pushes no frame). D16 and the Task 3 spike's
  own doc comment both say `Settings` lives per-`Activation`, not per-
  `Interp` ("the field arrives with Task 9" per `lib.rs`'s `Activation`
  doc comment) -- Task 6's Interfaces list confirms `settings: Settings` is
  a field Task 6 adds to `Activation`. Nothing names an ambient, pre-
  activation `Settings` on `Interp` that a fresh `eval_str` call would seed
  a throwaway activation from.

So implementing the brief's tests literally means Task 4 also has to (a)
make `Interp` public with a public no-arg constructor, (b) add some form of
expression-only or throwaway-fragment evaluation path, (c) add at least `+`
and `/` through `rexx-num` (Task 7's stated scope), and (d) decide where an
ambient `Settings` lives before any activation exists (touching Task 6's
design). That is a lot of Task 6/7 surface pulled forward into Task 4, and
building it now risks being redone or conflicting once Task 6/7 land in
their planned order.

Question sent to the team lead: which of these is intended --
(1) build the minimal slice of Task 6/7 needed (arithmetic `+`/`-`/`*`/`/`
    only, a throwaway per-call activation, `Interp` made `pub`) as
    Task 4 scaffolding that Tasks 6/7 will later replace/absorb, or
(2) test the value model directly against `Number`/`Body` without going
    through expression parsing at all (i.e. treat the brief's `eval_str`/
    `eval_with` calls as illustrative shorthand for the spec's transcripts,
    not literal required test code), or
(3) something else the team lead prefers.

Awaiting an answer before writing `value.rs` or `tests/value.rs`.

## Oracle verification, done while waiting (does not depend on the answer)

All under `( ulimit -v 1048576; build/bin/rexx FILE )`, scratch files in
`/tmp/claude-1000/.../scratchpad/task4/`.

**Created-digits transcript** (`t1.rex`):
```
numeric digits 9 ; y = 1 / 3 ; numeric digits 3 ; say y ; z = 1 / 3 ; say z
```
-> `0.333333333` then `0.333`. Matches D15 exactly.

**Created-digits, the 1e10 half** (`t1b.rex`): `1E+10`, `1E+10` (unchanged
after `digits 20`), `10000000000`. Matches.

**Created-form transcript** (`t2.rex`):
```
numeric form engineering ; x = 1e10 + 0 ; say x
numeric form scientific  ;               say x
                           y = 1e10 + 0 ; say y
```
-> `10E+9`, `10E+9`, `1E+10`. Matches D15 exactly.

**SmallInt admissibility transcript** (`t3.rex`):
```
numeric digits 1 ; x = 15 + 0 ; say x ; say x + 6 ; say 15 + 6
```
-> `2E+1`, `3E+1`, `2E+1`. Matches D15 exactly.

**Text keeps its own spelling** (`t4.rex`): `x = '007' ; say x` -> `007`,
`say x + 0` -> `7`. Matches.

**The exact-parse cache across a DIGITS change** -- this is the one place
the brief's transcript comment is imprecise and worth recording. The
brief's comment reads `x = '1.234567890123456789'; digits 5 -> 1.2346 ;
digits 20 -> the whole thing`, which on a first read looks like it means
`say x` after each `NUMERIC DIGITS`. Measured (`t4b.rex`, plain `say x`
both times): **`say x` prints the full 19-digit literal unchanged at both
DIGITS 5 and DIGITS 20.** That is correct and expected -- `x`'s identity is
its bytes (D15), so displaying it never converts it at all. Re-measured
with `x + 0` instead (`t4c.rex`): `say x + 0` gives `1.2346` at DIGITS 5
and the full `1.234567890123456789` at DIGITS 20, from the same stored
exact parse. So the comment means "converting x to a number", not "saying
x", and the brief's actual Rust test code never asserts the bare-`say`
half -- only `x + 0` (as `"7"` for `'007'`) is in the real assertion. No
defect in the test code itself, just a comment that reads more broadly
than the oracle supports; noting it so nobody implements `to_text` to
round a `Body::Text` value's own bytes by the current `DIGITS`, which
`t4b.rex` proves is wrong.

**`.nil`/`.true`/`.false`** (`t5.rex`): `say .nil` -> `The NIL object`;
`say .true` -> `1`; `say .false` -> `0`; `say (.true == 1)` -> `1`. Matches
D15 exactly.

All five of D15's measured transcripts (plus the sixth, `.nil`/booleans)
reproduce exactly as the spec states. No spec defect found here, unlike
the API-surface question above.

Two more, run while designing the `SmallInt` admissibility check, both
directly load-bearing for `small_int_for`'s design (below):

**`20.00 + 0` keeps its trailing decimal places** (`t6.rex`): `x = 1.00 + 1 ;
say x` -> `2.00` (not `2`), and `(x == 2)` -> `0`. `y = 20.00 + 0 ; say y` ->
`20.00`. This is the case that rules out `Number::whole_value` for the
admissibility check: `whole_value` answers "does this convert to a whole
number", which is yes for `20.00`, but the oracle prints `20.00`, not `20`,
so admitting it as `SmallInt(20)` would be a real, observable bug -- a
`SmallInt` can only ever render as a bare integer with no decimal point at
all, however many of its stored decimal digits are zero.

**`.nil + 1` fails at message dispatch, not at numeric conversion**
(`t7.rex`): `x = .nil + 1` is Error 97.1, `"Object \"The NIL object\" does not
understand message \"+\""`, rc 159 -- not error 41 (nonnumeric value used in
arithmetic). 4a has no general message dispatch (that is Phase 5's), so
`to_number(.nil)` answering `NotNumeric` cannot reproduce 97.1 and is not
trying to; it is the honest answer for what this layer alone can see. Noted
in `to_number`'s doc comment so nobody reads the difference as a bug later.

## Team lead's answer

Option **(2)**: unit tests inside `#[cfg(test)] mod tests` in `src/value.rs`
itself, constructing `Number`s directly through `rexx-num` rather than
through an evaluator, so `Interp` stays private and no public surface is
widened for testing (the same lesson Task 3's review drew from the other
direction). `number` takes `created_digits: u32, created_form: Form` as
**explicit arguments**, not read from anywhere ambient -- which also
dissolves the `Settings`-before-any-activation question: there is no
`Settings` in this module at all, and nothing changes here when Task 6 adds
one to `Activation`. The plan doc was corrected in place (Task 4's section,
`docs/superpowers/plans/2026-07-30-phase-4a-executor.md`).

## Implementation

`rust/crates/rexx-exec/src/value.rs` (new), `impl Interp` block with four
`pub(crate)` methods:

* `text(&mut self, &[u8]) -> ObjRef` -- allocates `Body::Text { bytes,
  num: None }`. Replaces the spike's private `alloc_text`.
* `number(&mut self, Number, created_digits: u32, created_form: Form) ->
  ObjRef` -- applies the `SmallInt` admissibility check (`small_int_for`,
  below) and falls back to a heap `Body::Num` carrying the same pair.
* `to_text(&mut self, ObjRef) -> Cow<'_, [u8]>` -- `.nil`/`SmallInt` render
  fresh (no backing bytes to look up); a `Body::Num`'s `text` cache is filled
  once, through `format_form(created_digits, created_form)`, **never**
  `settings.digits()`/`.form()` (there is nothing of the kind in scope to
  reach for by mistake).
* `to_number(&mut self, ObjRef) -> Result<Number, NotNumeric>` -- `Body::Num`
  clones its own `value`; `Body::Text`'s tri-state `num` cache is filled at
  most once via `std::str::from_utf8` then `Number::parse`, both failures
  collapsing to `NotNumeric`, and the cache is never rounded at fill time.

`rust/crates/rexx-exec/src/lib.rs`: added `mod value;`; removed the old
private `alloc_text`/`text_of` (superseded); updated every call site (`Say`,
the fragment-spike `Interpret` arm, `read`'s uninitialised-variable path,
`eval_node`'s `Literal`/`Constant`/`Concatenate` arms) to call
`self.text(...)`/`self.to_text(...)` instead. Trimmed `BehaviourId`/`Body`
from `lib.rs`'s `use rexx_core::{...}` since they are now used only from
`value.rs`.

`rust/crates/rexx-exec/Cargo.toml`: added the `rexx-num` dependency (needed
for `Number`/`Form`/`format_form`, not previously a dependency of this
crate). `rust/Cargo.lock` picks up the one new edge; diffed to confirm it is
*only* that edge, nothing from the parser/corpus agents' concurrent work.

**A real discovery while implementing, not merely a workaround:**
`Number`'s fields (`digits`, `exponent`, `negative`) are `pub(crate)` to
`rexx-num`, so `small_int_for` cannot inspect them directly -- the first
draft (checking `exponent >= 0` and hand-computing the magnitude from
`digits`) does not compile outside `rexx-num`. Rewrote it to decide by
**rendering**: call `value.format_form(created_digits, Form::Scientific)`
and admit only when the result has no `.` and no `E`, then parse those
digits back into `i64` and range-check against `SMALL_INT_MIN..=
SMALL_INT_MAX`. This is not merely a way around the field wall, it is the
better design: `format_form` is exactly what `to_text` calls for a
`Body::Num`, so a `SmallInt`'s rendering and a `Body::Num`'s rendering are
now provably the same computation asked the same question, rather than two
independently-written rules that happen to agree today. `Form::Scientific`
in the probe is arbitrary -- D15 states the two forms agree on plain
rendering, and an exponential result is refused regardless of which form
produced its exponent grouping, so the probe form cannot bias the answer.
On refusal by `E` (an exponential result), the rendered string is thrown
away rather than seeded into `Body::Num.text`: it was rendered in
`Scientific`, and if the object's real `created_form` is `Engineering` that
string would be wrong (D15's own `1E+10`/`10E+9` pair) -- `to_text`'s own
lazy fill, keyed off the object's real `created_form`, is what has to
produce it.

Verified this against `t6.rex` (`20.00 + 0` keeping its point) before
trusting it: `value.format_form(9, Scientific)` on that `Number` renders
`"20.00"`, contains `.`, correctly refused.

Two `#[allow(clippy::wrong_self_convention, reason = "...")]` were needed:
clippy's `to_*`-implies-`&self` convention fires on `to_text`/`to_number`
because both take `&mut self` (load-bearing for the lazy cache fill) under
names the design's interface list fixes, not a naming choice this task is
free to change. Two `#[allow(dead_code, reason = "...")]` on `number`/
`to_number`, matching the existing `Flow::Goto` precedent in `lib.rs`:
nothing outside this module's own tests calls them yet, since the
arithmetic evaluator that will is Task 7's `eval.rs`, which does not exist.

## Tests

`cargo test -p rexx-exec`: **8 new unit tests in `value::tests`, all
passing**, plus the 10 pre-existing integration tests in `tests/spike.rs`
and 2 doctests, untouched and still passing. `cargo clippy -p rexx-exec
--all-targets -- -D warnings`: clean. `cargo fmt -p rexx-exec -- --check`:
clean after one `cargo fmt` pass (reformatted `value.rs`'s longer call
sites into multi-line form).

The four tests straight from the spec's transcripts (`a_numbers_rendering_
is_fixed_when_it_is_created`, `numeric_form_is_captured_at_creation_too`,
`a_small_int_is_only_admissible_within_the_digits_of_its_own_operation`,
`text_keeps_its_own_spelling_and_caches_an_exact_parse`), plus `nil_has_a_
string_value_and_the_booleans_are_plain_strings`. Three more added beyond
the brief, in the spirit of Task 3's review (which wrote its own test for a
path neither of the brief's two tests could see):

* `small_int_admissibility_is_checked_once_against_the_producing_operations_
  digits` -- checks the actual `ObjRef` shape (`Decoded::SmallInt` vs
  `Decoded::Heap`) for D15's own discriminating pair (`a = 20+0` at
  `DIGITS 9`, `b = 15+0` at `DIGITS 1`), not only the rendered bytes.
* `the_text_cache_holds_the_exact_parse_not_a_rounded_one` -- the
  1.234567890123456789 transcript, reading the same `Body::Text`'s cache
  twice under two different `DIGITS`.
* `nonnumeric_text_and_nil_both_collapse_to_not_numeric` -- `to_number` on
  non-numeric text and on `.nil` both answer `Err(NotNumeric)`.

I did **not** create `rust/crates/rexx-exec/tests/value.rs`. The plan's
"Files" list still names it, but the team lead's ruling (unit tests inside
`src/value.rs`, `Interp` stays private) makes an integration test file
pointless here: there is nothing in the public API for it to reach. Worth
the plan's Files list being corrected alongside the rest, since a future
reader following it literally would go looking for a file this task
deliberately does not create.

## Concerns for the team lead

* **A type mismatch nobody has to resolve yet, but will:** `rexx_num::
  Settings::digits()` returns `u64`; `Body::Num::created_digits` (Task 2,
  already committed) and `number()`'s new `created_digits: u32` parameter
  are both `u32`. Whatever calls `number()` from a real `Settings` (Task 7's
  `eval.rs`) has to narrow `u64` to `u32` at that call site. `NUMERIC
  DIGITS` can legally be set up to `999_999_999_999_999_999` (`Settings`'s
  own `MAX_WHOLENUMBER`), far past `u32::MAX`, so that narrowing needs a
  deliberate decision (saturate? truncate and let the `SmallInt` width
  check reject it downstream? panic, since no corpus program goes near
  it?) rather than a silent `as u32`. Not this task's field to redesign --
  `Body::Num`'s shape was Task 2's decided interface -- but flagging it now
  since Task 7 will hit it on day one.
* Confirmed the `Files` list discrepancy above; recommend it gets folded
  into the same plan correction that already landed for this task.

## Commit

`75990fc9`, "Add the value model: text and number identity, fixed at
creation" -- committed by the team lead, on my behalf, after my session hit
a usage limit right at the commit step with the four files already staged
and green (`git add` had run; `git commit` had not). Confirmed independently
after the fact: `git show --stat 75990fc9` touches exactly the four files
this task built (`rust/Cargo.lock`, `rust/crates/rexx-exec/Cargo.toml`,
`src/lib.rs`, `src/value.rs`), nothing from the concurrently active
`rexx-parse`/`rexx-extract` agents leaked in.
