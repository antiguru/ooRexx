STATUS: DONE

# Review of Task 4, commit 75990fc9 (the value model)

## Verdicts

**Spec compliance: PASS.** All five plan steps done, and the mid-task
question (the brief's own illustrative test code calling methods that
don't exist and can't without pulling Task 6/7 forward) was the right
thing to escalate rather than guess at -- it is a real API-surface
mismatch, not an invented problem, and the ruling that came back (unit
tests inside `src/value.rs`, no public `Interp`) is what the plan now
says. **Code quality: PASS.** 0 blocking, 0 major, 0 minor. This is the
strongest commit I've reviewed in this family so far: every design choice
the report calls out as a discovery survives independent tracing through
the code it depends on, not just the report's own narration of it.

Reviewed the commit's own diff and content, not the current working tree:
by the time I started, `rexx-exec` had moved on twice more (Task 5's
stems commit `28a62383`, then an audit-fixes commit `9b11836d`), both
touching `value.rs`/`lib.rs`/`body.rs`. Built and tested `75990fc9` in
isolation, via `git archive` into a scratch copy, rather than the
integrated HEAD, so nothing later is mixed into this verdict.

## Inputs

- `docs/superpowers/plans/2026-07-30-phase-4a-executor.md`, "Task 4" section
  (current text, which the report says was corrected in place -- confirmed
  below).
- D15 and D15a in `docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md`.
- `.superpowers/sdd/2026-07-30-phase-4a-executor/task-4-report.md`.
- `git show 75990fc9` (full diff: `Cargo.lock`, `rexx-exec/Cargo.toml`,
  `src/lib.rs`, `src/value.rs`).
- Neighbouring source read directly rather than taken from the report's
  description of it: `rexx-num/src/format.rs` (`format_form`/`format_with`,
  the exponential-trigger logic), `rexx-core/src/body.rs` (`Body`,
  `NotNumeric`), `rexx-core/src/handle.rs` (`ObjRef`, `Decoded`,
  `SMALL_INT_MIN`/`MAX`), `rexx-num/src/settings.rs` (`Settings::digits`,
  `MAX_WHOLENUMBER`).

## Pressure point 1: is the `SmallInt` check genuinely equivalent to the spec's digit-count rule?

D15's rule, in full: admissible only when the exact result is whole, inside
the tag's range, and its decimal digit count is at most the `DIGITS` that
produced it, "form-independent" because "under the digit-count condition
the rendering carries no exponent, so engineering and scientific agree."
The implementation decides all of this in one shot: render with
`value.format_form(created_digits, Form::Scientific)` and admit only if
the result contains neither `.` nor `E`, then range-check the parsed `i64`.

Traced rather than accepted, because "renders without a `.` or an `E`" and
"is whole and narrow enough" are only the same thing if two further facts
both hold, and I checked both against the actual arithmetic rather than
against the report's restatement of them:

- **Does the oracle's own "whole" really mean "renders with no decimal
  point", not "is mathematically an integer"?** Re-ran the report's own
  `t6.rex` against the oracle myself: `x = 1.00 + 1 ; say x` -> `2.00`,
  `x == 2` -> `0`, `y = 20.00 + 0 ; say y` -> `20.00`. Confirmed. This is
  what rules out `Number::whole_value` (which would say yes to `20.00`)
  and is exactly the trap a literal reading of D15's word "whole" sets:
  the spec's own phrase is ambiguous between the two readings, and only
  the oracle measurement resolves it. The render-based check gets this
  right for a structural reason, not by accident: `format_form`'s
  plain-decimal branch reproduces a value's own stored significant digits
  (that is the entire point of decimal arithmetic preserving trailing
  zeros), so a value with a real stored decimal place renders with a `.`
  regardless of whether it is mathematically whole.
- **Can probing in `Form::Scientific` admit or refuse something that
  `Form::Engineering` would have decided the other way?** Read
  `format_with`'s trigger logic directly (`rexx-num/src/format.rs`): the
  boolean that decides exponential-vs-plain (`a as i64 >= expt || (a < 0
  && ...)`, computing `triggered`) is calculated **before** `group(form,
  a)` is ever called, and none of the inputs to that comparison (`a`,
  `expt`, both derived from `n1`/`digits`, never `form`) reference `form`
  at all -- `form` only selects how an *already-triggered* exponent is
  grouped. So the trigger is provably form-independent, matching D15's own
  claim, and probing with `Scientific` cannot bias whether the `.`/`E`
  check fires versus probing with `Engineering` would have.
- **Does the discarded `Scientific` rendering leak into an object whose
  real `created_form` is `Engineering`?** Read `number`'s allocation on
  refusal: `Body::Num { value, created_digits, created_form, text: None }`
  -- `text` is explicitly `None`, not the rejected rendering. `to_text`'s
  own lazy fill later computes `format_form(created_digits, created_form)`
  from the object's *real* form. No leak, confirmed by reading the
  allocation site rather than trusting the doc comment's claim about it.

Conclusion: genuinely equivalent, and for a reason stronger than the
report states modestly ("this is not merely a way around a wall"): by
reusing the exact function `to_text` calls for a `Body::Num`, a
`SmallInt`'s admissibility and a `Body::Num`'s rendering are the same
computation asked once, not two independently-maintained rules that
happen to agree today and could silently drift apart on the next change
to either. This is a better design than a hand-rolled digit-count
check would have been, not merely an adequate substitute forced by
`Number`'s private fields.

## Pressure point 2: the tri-state `num` cache

Grepped every touch of `Body::Text`'s `num` field in the diff: exactly two
sites, `text()` (sets `None` on creation) and `to_number()`'s
`get_or_insert_with` (fills it exactly once, on first ask, from whichever
of the two failure causes actually applies). Nothing else reads or writes
it, so there is no second code path that could misread `None` as "not a
number" -- the tri-state discipline has exactly one gate. `get_or_insert_with`
also gives the right update-once semantics for free: a second call against
the same object, at a different `DIGITS`, reads back the same stored exact
parse rather than re-parsing or re-rounding it, which is what
`the_text_cache_holds_the_exact_parse_not_a_rounded_one` (a test written
beyond the plan's own five) actually exercises, and what I independently
re-verified against the oracle (`t4b.rex`/`t4c.rex`, below).

## Pressure point 3: does nothing read ambient settings, structurally and not just today?

Grepped the whole diff for "settings": every hit is inside a comment
explaining the deliberate *absence* of one ("there is no `Settings` in
scope to reach for by mistake"); no code path touches any such field.
Checked further than the diff: `Interp` at this commit (`git show
75990fc9:.../lib.rs`, `struct Interp`) has no `settings` field at all --
`heap`, `roots`, `activations`, `programs`, `plans`, `out`, `trace`,
`interpret_spike`, nothing else -- so the rule is enforced by the
codebase's actual shape at this point, not merely by discipline. And it
stays enforced by construction going forward: `number`/`to_text` take
`created_digits`/`created_form` as plain parameters rather than reaching
through `self` for anything, so a future caller (Task 7) has no ambient
value to reach for even once `Settings` exists on `Activation` -- it has
to be threaded in explicitly at the call site, which is the discipline
D15 asks for made structural rather than promised.

## Independent re-verification of the oracle transcripts

Re-ran every transcript the design leans on myself, wrapped
(`ulimit -v 1048576`), rather than trusting the report's paste:

```
t1 (DIGITS 9->3, y=1/3):      0.333333333 / 0.333               -- match
t2 (FORM engineering->sci):    10E+9 / 10E+9 / 1E+10             -- match
t3 (DIGITS 1, SmallInt):       2E+1 / 3E+1 / 2E+1                -- match
t4b (say x, DIGITS 5->20):     1.234567890123456789 (both times) -- match
t4c (say x+0, DIGITS 5->20):   1.2346 / 1.234567890123456789     -- match
t6 (20.00+0 keeps its point):  2.00 / 0 / 20.00                  -- match
t7 (.nil + 1):                 Error 97.1, rc 159                -- match
```

All exact matches. `t6` and `t7` are the two the report calls out as
"directly load-bearing for `small_int_for`'s design" and "beyond what the
brief asked for" respectively -- re-running them myself rather than only
`t1`-`t5` (the plan's own five) is what actually presses on the two
design decisions this review was asked to judge.

## The flagged type mismatch (`u32` vs `u64` digits)

Confirmed real: `rexx_num::Settings::digits() -> u64` and
`MAX_WHOLENUMBER = 999_999_999_999_999_999` (`settings.rs`), both far past
`u32::MAX` (~4.3 billion), while `number`'s `created_digits` parameter and
`Body::Num::created_digits` are both `u32`. Confirmed this is not
something Task 4 introduced or could fix unilaterally: `Body::Num`'s shape
(`created_digits: u32`) is Task 2's already-committed Interfaces entry,
and Task 4's own plan section fixes `number`'s signature to match it, so
the type was chosen two tasks upstream of here. Correctly scoped in the
report as a heads-up for Task 7 (whoever narrows a live `Settings::digits()`
to call `number()` has to decide how, deliberately) rather than worked
around with a silent cast in this task, which would have been the wrong
place to make that decision anyway.

## Verification, on the commit in isolation

Extracted `75990fc9` with `git archive` into a scratch copy (which
includes the whole tree at that commit, `interpreter/` included -- no
symlink needed, learned from a mistake in the previous review), built and
tested there rather than against the current, further-modified working
tree:

- `cargo test -p rexx-exec --no-fail-fast`: exit 0. **8/8** new
  `value::tests` pass, the pre-existing **10** `tests/spike.rs` and **2**
  doctests untouched and passing -- matches the report's count exactly.
- `cargo clippy -p rexx-exec --all-targets -- -D warnings`: exit 0.
- `cargo fmt -p rexx-exec -- --check`: exit 0.
- Confirmed the two `#[allow(clippy::wrong_self_convention, ...)]` and two
  `#[allow(dead_code, ...)]` annotations are load-bearing, not decoration:
  the interface names (`to_text`/`to_number` taking `&mut self`) are fixed
  by D15's own Interfaces list, and the `dead_code` allows match an
  existing project precedent verbatim (`lib.rs:572`,
  `#[allow(dead_code, reason = "Tasks 10 and 11 build the jumps that
  produce it")]` for `Flow::Goto`) rather than inventing a new exemption
  style.
- Confirmed the plan's Files list was actually corrected: Task 4's section
  now reads "Test: a `#[cfg(test)] mod tests` inside .../src/value.rs"
  (not the stale `tests/value.rs`), and no such file exists in the tree
  (`rexx-exec/tests/` holds only the pre-existing `spike.rs`) -- the report
  and the plan agree with each other and with what was actually built.
- `git show 75990fc9 --stat`: exactly `Cargo.lock`, `rexx-exec/Cargo.toml`,
  `src/lib.rs`, `src/value.rs` -- nothing from concurrently active agents.

## Code quality notes (none blocking)

- The eight tests go beyond the plan's five in a way that matters, not
  just in count: `small_int_admissibility_is_checked_once_against_the_
  producing_operations_digits` asserts the actual `ObjRef` shape
  (`Decoded::SmallInt` vs `Decoded::Heap`), not only the rendered bytes --
  which is the right test for exactly this design, since a wrongly-admitted
  `SmallInt` and a correctly-refused `Body::Num` can render identical bytes
  if `to_text`'s two branches happen to agree, and only the tag itself can
  tell them apart. This is the same discipline as "defeat the mechanism":
  a test that only checks rendered output would not have caught a bug in
  the admissibility check itself.
- `to_text`/`to_number`'s `other => unreachable!(...)` arms are correctly
  scoped to what exists *at this commit* (only `Body::Text`/`Body::Num` are
  ever produced here) rather than defensively handling variants no caller
  can produce yet -- and the very next commit in this crate's history
  (Task 5, `28a62383`) is exactly what extends `to_text` to cover
  `Body::Stem`, confirming the boundary was drawn in the right place, not
  left for later to discover as a bug.
- Reusing `BehaviourId::STRING` for both `Body::Text` and `Body::Num`
  matches the language's own model (a Rexx number is a string that also
  carries a validated numeric value, not a separate class) rather than
  inventing a distinct behaviour id no class dispatch could ever reach.

## Scratch cleanup

All work under this session's scratchpad (`.../scratchpad/task4review/`),
a single `git archive` extraction, no commits/branches/worktrees against
the real repository. `git status` on the real repository shows only
other agents' own in-flight files, nothing from this review.
