STATUS: DONE

# Task 6 fix report: Plan::note/build must be exhaustive, and pinned by a content test

Scope: strict, `rust/crates/rexx-exec/src/plan.rs` only, as dispatched.
Stayed inside it entirely -- one Minor (below) needed a change in
`activation.rs` and is flagged rather than reached for.

## What was wrong

`Plan::note` handled `Variable`/`Prefix`/`Binary` then `_ => {}`;
`Plan::build` handled `Assignment`/`Say`/`Interpret` then `_ => {}`. No
`Stem`/`Compound` was ever visited, so a body containing one got an empty
plan and every one of its names fell back to the lazy, one-at-a-time
`grow_slots` path D16 exists to replace -- worst exactly where D16's own
32.2% figure was measured, stem-heavy code.

## Investigation: what actually needed a slot, read-only against `eval.rs`/`stem.rs`

Before writing anything, traced how each `ExprKind` and `VariableRef`
shape is actually *resolved* at run time, reading (not touching) `eval.rs`
and `stem.rs` -- both real code, not guessed at:

* `eval_node`'s `Variable(id) | Stem(id)` arm calls `self.read(code, *id)`,
  which tries `code.slots.get(&id)` (the `by_symbol`-derived fast path)
  before falling back to `self.slot_of(code.symbols.name(id).as_bytes())`.
  So a bare `Stem` needs exactly the same treatment `Variable` already
  got: bind the id *and* the name.
* `eval_node`'s `Compound(id)` arm never calls `read` at all -- it derives
  `stem_name` via `compound_parts(code.symbols.name(*id))`, resolves the
  tail via `tail_key`, and calls `stem_get(stem_name.as_bytes(), &key)`.
  `stem_get` (`stem.rs`) itself calls `self.slot_of(stem_name)` -- a
  **byte-name** lookup, never an id one. So a `Compound`'s own id is never
  looked up anywhere; only its *decomposed* stem name matters.
* `tail_key` resolves each `Tail::Variable` piece via
  `self.read_by_name(name.as_bytes())`, which itself calls
  `self.slot_of(name)` -- also byte-name only, because (per `ast.rs`'s own
  `Compound` doc comment, and `stem.rs`'s own comment on `read_by_name`) a
  tail piece is a borrowed slice of the compound's one interned spelling,
  never a token the scanner saw, so it has no `SymbolId` of its own to
  bind.

This settled the design before any code was written: `Stem` binds `(id,
name)` exactly like `Variable`; `Compound` registers the decomposed stem
name and every `Tail::Variable` piece's name, by name alone, with nothing
bound to the compound's own id (nothing ever looks it up). The same
name-only path applies to `Redirection::Stem` (`ADDRESS ... WITH STEM
name.`, resolved by `stem_get` the identical way) and to a
`Drop`/`Expose`/`Procedure`/`Use Local` target once its spelling is known
to be compound- or stem-shaped.

## `note`: exhaustive over `ExprKind`, no catch-all

Added `Stem`, `Compound`, `Call`/`QualifiedCall` (args), `Message`
(target/super_class/args), `List`, `Logical`, `VariableReference`. Left
`Literal`/`Constant`/`DotVariable`/`ClassResolver` as one explicit empty
arm, each with the specific reason it names nothing (a constant's value is
its own spelling, a `.name` symbol resolves through the class/environment
lookup, a `ClassResolver` names a class not a variable).

`for_each_child` (`rexx-parse`'s `ast.rs`), which already walks this exact
shape, is `pub(crate)` to `rexx-parse` and unreachable from `rexx-exec` --
confirmed by trying to call it and getting a privacy error before writing
the match by hand. `rexx-parse/tests/gate_walk/mod.rs`'s `children_of`
reimplements the identical shape for the identical reason, one crate over,
which is cited in `note`'s own doc comment so a future reader does not
wonder why this isn't shared code.

## `build`: exhaustive over `InstructionKind`, no catch-all

Registers every name an instruction's fields *could* name, not only the
three kinds `eval`/`run` execute today (`Assignment`/`Say`/`Interpret`).
Most `InstructionKind` variants still fail loudly in the current
interpreter and gain real behaviour only in Tasks 9-14, but pre-registering
their names now is free (an unread slot is simply unread) and means this
file does not need a second fix round the day one of them stops failing
loudly -- the alternative, matching only the currently-executed kinds and
writing "Task N will handle the rest" comments on the others, is exactly
the anti-pattern the dispatch named: "both `_ => {}` arms carry comments
... promising that *Task 6* would close them, so they now read as the task
promising itself."

Extracted five small helpers (`note_loop`, `note_parse`, `note_call`,
`note_variable_ref`, `note_compound_name`) plus two trivial ones
(`note_opt`, `note_args`) rather than one very long `match`, since several
`InstructionKind`/nested-struct shapes (`Loop`, `Parse`, `Call`,
`VariableRef`) needed more than a one-line arm. Field names and shapes
for every nested struct (`Loop`, `LoopKind`, `Controlled`,
`LoopConditional`, `Parse`, `ParseSource`, `ParseTrigger`, `Call`,
`Signal`, `Guard`, `Forward`, `Raise`, `RaiseResult`, `Use`, `UseTarget`,
`Address`, `AddressIo`, `Redirection`, `Trace`) were read directly from
`rexx-parse`'s `ast.rs` rather than recalled, specifically to avoid
transcribing a field name wrong and discovering it as a run of compile
errors instead of up front.

One instruction-kind subtlety worth recording: `Do`/`Loop`'s own `label`,
`Select`'s own `label`, and `Leave`/`Iterate`/`End`'s `name` are **block**
labels, matched against `LEAVE`/`ITERATE`/`END`, never data variables --
none of the three is bound. `DO COUNTER name` and every controlled/`OVER`/
`WITH` loop's control variable(s) *are* data variables and are bound.

## `VariableRef` (`Drop`/`Expose`/`Procedure`/`Use Local`)

`Indirect(id)`'s `id` is the *wrapper* variable read at run time to learn
the real target's name (`DROP (v)` reads `v` itself) -- the target itself
is not statically knowable, so nothing more is registered for it, and this
is exactly the case the pre-existing `a_runtime_name_grows_the_frame` test
already covers and still expects to fall through to `extra`/`grow_slots`
(confirmed still passing, unmodified).

`Direct(id)`'s spelling can be a plain variable, a stem or a compound with
no tag distinguishing them (`VariableRef`'s own doc comment says so).
Dispatched on whether the spelling contains a `.` at all -- the same test
`compound_parts` itself requires before it can be called without
panicking -- routing to the compound-name path or a plain `bind`
accordingly.

## The content test, and the mutation that proves it

Added `build_registers_every_name_a_stem_or_compound_touches`
(`plan.rs`'s own `tests` module), asserting `plan.names` **contents**
directly -- which byte strings are keys -- for six bodies, four straight
from the dispatch's own measured repro (`say a.b`, `a.1 = 'x'`, `q. = 1`,
`say v`) plus two more exercising `build`'s own fix specifically rather
than only `note`'s (`drop a.b.c`, and `do i = 1 to 5 / end`, neither of
which the pre-fix `build` ever reached at all, compound target or not).

**Mutation, run on a scratch copy exactly as asked**, not merely narrated:
copied `rust/` to scratch, neutered `Plan::build` to
`Plan::default()` unconditionally (the reviewer's own mutation the
dispatch names), symlinked `interpreter/` in (needed for
`rexx-inventory`'s build script; confirmed with `git status` before and
after that the real repository was never touched), and ran
`cargo test -p rexx-exec --no-fail-fast` unpiped:

```
Without the fix:  37 passed, 1 failed (a_fragments_plan_resolves_against_the_enclosing_frame)
With the mutation: 36 passed, 2 failed (the fragment test AND
                    build_registers_every_name_a_stem_or_compound_touches,
                    which fails immediately: "say a.b" must build a
                    non-empty plan)
```

So without the new test, the mutation's only symptom is exactly the one
fragment test the dispatch already named as inadequate on its own; with
it, the regression is caught directly, by content, on the very first
case -- the bar the dispatch set. Every other test (36 of 38) still passes
against the mutant, confirming the new test is not merely duplicating
something else that would have caught this.

## The two Minors

**`by_symbol`'s "pending" comment (in scope, fixed).** `SymbolId::index()`
landed at `180875a9`, before Task 6's own commit, and the struct doc
comment still called switching to it a decision "pending" that later
prose. Corrected: the accessor now exists and switching is available, not
made in this fix round because this round is scoped to exhaustiveness, not
to the representation, and the `HashMap`-to-`Vec` trade is worth its own
measurement (variable lookup is 8.1%/32.2% of runtime) rather than a side
effect here.

**The `blocks` deferral's stated reason (out of scope, flagged rather than
reached for).** This lives in `activation.rs`, not `plan.rs` -- confirmed
by grep before assuming otherwise. Per the dispatch's own strict-scope
instruction ("if you believe the fix needs a change outside `plan.rs`,
stop and tell me rather than reaching"), **not touched.** The exact
correction, ready for whoever does touch that file: `activation.rs`'s
module doc comment currently says `Block`'s "only real definition anywhere
is the DO/LOOP passage of the design doc ... which is Task 11's to write,
not Task 6's to guess at" -- but that same sentence already states the
definition (the control variable's slot, `to`/`by`/`for`, the counter, the
label and the `end` index), so the comment both asserts and states the
"missing" definition in the same breath. The decision to defer is still
right; only the reason is wrong. Corrected reason, matching what the
dispatch itself gives and I agree with: not "the shape is unknown" but "no
reader exists yet, and Task 11's first real use should pick the
representation" -- a premature-commitment reason, not a missing-information
one.

## Verification

* `cargo build -p rexx-exec`: exit 0.
* `cargo test -p rexx-exec --no-fail-fast`: exit 0, **38/38** (37
  pre-existing, unmodified, + 1 new), plus 10 integration tests and 2
  doctests, all unchanged and passing.
* `cargo clippy -p rexx-exec --all-targets -- -D warnings`: exit 0.
* `cargo fmt -p rexx-exec -- --check`: exit 0.
* `cargo build --workspace`: exit 0.
* `cargo test --workspace --no-fail-fast`: exit 0, every `test result:`
  line `0 failed` (checked by grepping for any that were not, zero hits).
* Mutation run, above: exit 101 (as expected -- two real failures),
  confirming the new test's coverage rather than merely asserting it.

## Files touched

`rust/crates/rexx-exec/src/plan.rs` only. Nothing under `eval.rs`,
`error.rs`, `lib.rs`, `activation.rs`, or any other crate.

## Commit

`3cf3f03c`, "Make Plan::note/build exhaustive over ExprKind/InstructionKind,
no catch-all". Staged and committed exactly `rust/crates/rexx-exec/src
/plan.rs`. `git status` after the commit is clean.
