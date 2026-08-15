STATUS: DONE

# Task 6 report: the resolution plan

## Pre-flight reading

`task-6-brief.md` (stale: still has the old wrong `Plan { slots, len }`
Interfaces) and the plan document's own Task 6 section (corrected, matches
the team lead's second message: `Plan { names, by_symbol }`, matching what
Task 3 already built). D16 in full. The current `rexx-exec/src/lib.rs`
(`Plan`, `BodyKey`, `ProgramId`, `Activation`, `Interp::plan_for`/`slot_of`,
in full, again, since this task moves them). `rexx-parse/src/token.rs`
(`SymbolId`, `SymbolTable::intern`/`name`) to check the `SymbolId::index()`
question with real evidence rather than guessing.

## The three oracle transcripts, verified (does not depend on the open
## questions below)

All under `( ulimit -v 1048576; build/bin/rexx FILE )`.

| transcript | result | matches D16 |
|---|---|---|
| `b=2; say a.b; a.2='hit'; say a.b` | `A.2` / `hit` | yes |
| `v='X'; x=1; drop (v); say x` | `X` | yes |
| `i='abc'; v.i='val'; say v.i v.ABC` | `val V.ABC` | yes (same transcript D15a used) |

## Two design questions, sent before writing `plan.rs`/`activation.rs`

**1. `SymbolId::index()`, with the reasoning asked for.** Checked
`token.rs` rather than guessing: `SymbolTable::intern` assigns
`SymbolId(u32(self.names.len()))` (`token.rs:122`) and `SymbolTable::name`
indexes `self.names[id.0 as usize]` directly (`token.rs:132`) -- so a
`SymbolId`'s raw value is *already* a dense, table-local, zero-based `Vec`
index in every table that exists, including a fragment's fresh one. Nothing
about exposing it changes an invariant; it only stops being the one thing
`rexx-parse` isn't saying out loud. Given that, and given the 8.1%/32.2%
figure is D16's own stated reason for the whole design, my recommendation
is yes: ask for `SymbolId::index() -> usize` (or `u32`) and turn
`Plan::by_symbol` into a `Vec<Option<usize>>`, sized incrementally during
the same upfront walk that builds `names` (no `SymbolTable::len()` needed).
**Not blocking on this**: implementing with the `HashMap<SymbolId, usize>`
already in the tree first, since that is what Task 3 built and what "you
are inheriting its shape, not inventing one" asks for, and the `Vec` swap
is a mechanical follow-up once (if) the accessor lands. Flagging the
recommendation rather than quietly taking the hash, per the ask.

**2. `blocks: Vec<Block>` on `Activation`.** The Interfaces list carries it,
but D16 (this task's own spec section) never mentions blocks at all, and
`Block`'s only real definition anywhere is the design doc's DO/LOOP
passage ("a per-activation `Vec<Block>` holding the control variable's
slot, the `to`, `by` and `for` values, the iteration counter, the block's
label and its `end` index") -- which is Task 11's territory (`DO`/`LOOP`),
not Task 6's. Inventing a placeholder `Block` now that Task 11 replaces
wholesale is the same shape of throwaway scaffolding Task 4's brief
correction explicitly ruled out for `eval_str`. Proceeding **without**
`blocks` on `Activation` for this task; Task 11 adds it when it can define
what a block actually holds. `settings: Settings` is different and stays
in: `Settings` already exists (`rexx-num`), needs no invention, and is a
field-only addition with a well-defined default.

## Implementation

Moved (not rewritten) from Task 3's spike in `lib.rs` into two new files,
per the crate layout:

* `rust/crates/rexx-exec/src/plan.rs`: `ProgramId`, `BodyKey`, `Plan` (with
  `build`/`note`/`bind`/`len`, all unchanged), and the `Interp` methods
  that operate on them -- `plan_for` (the cache), `slot_of` (the full
  three-source resolution: plan, then `extra`, then growth) and
  `fragment_plan` (a fragment's ids resolved against the enclosing frame).
  One small addition beyond a straight move: `Plan::slot_of(&self, name:
  &[u8]) -> Option<usize>`, a thin wrapper over `self.names.get(name)`,
  so `Interp::slot_of`'s first step reads as `activation.plan.slot_of(name)`
  rather than reaching into `plan.names` directly -- this is the literal
  vocabulary the brief's own pseudocode uses
  (`plan.slot_of(name).or_else(|| extra.get(name))`), not a new design.
* `rust/crates/rexx-exec/src/activation.rs`: `Activation` (all its
  existing fields, plus the new `settings: Settings`), and `Interp::
  activation`/`activation_mut`. Added `Activation::new(program, plan,
  frame)`, a constructor that default-initialises `extra`, `pc` and
  `settings` -- `Interp::run` (staying in `lib.rs`) now calls this instead
  of writing the struct literal out, so `lib.rs` never needs to know
  `Settings::default()` exists at all.

Every moved type's fields and the moved methods are `pub(crate)`, since
`Interp`'s own struct definition (staying in `lib.rs`, the parent module)
constructs/reads them directly (`ProgramId(self.programs.len())`,
`BodyKey { program: id, directive: None }`, `slots: &plan.by_symbol` in
`run_activation`) -- private (module-only) visibility runs the wrong
direction for a child module's items to be usable from its parent.

`rust/crates/rexx-exec/src/lib.rs`: the moved code deleted, `mod plan;`/
`mod activation;` added, `Interp::run`'s `Activation { .. }` literal
replaced with `Activation::new(..)`, and an unused `SlotFrame` import
dropped (it now only appears inside `activation.rs`'s own field type).

`rust/crates/rexx-exec/src/stem.rs`: its test-only `activate` helper
updated to call `Activation::new(..)` too, since the struct literal it
built now needs (and shouldn't need to know about) `settings`.

## Tests

`cargo test -p rexx-exec`: **20 unit tests** (4 new in `plan::tests` --
one per Step 1 test name, straight from the transcripts above -- plus the
16 already in `value::tests`/`stem::tests`, unchanged), plus the 10
pre-existing `tests/spike.rs` integration tests and 2 doctests, all still
passing -- confirming the move lost nothing. `cargo clippy -p rexx-exec
--all-targets -- -D warnings`: clean. `cargo fmt -p rexx-exec -- --check`:
clean after one `cargo fmt` pass.

One test bug worth recording, caught before it could hide a real defect:
my first draft of `names_are_keyed_upcased_but_tail_values_are_not` called
`interp.slot_of(b"i")` and `interp.slot_of(b"I")` directly and asserted
equal slots, which failed (0 vs 1). `slot_of` does not itself upcase
anything -- upcasing happens once, upstream, in `SymbolTable::intern`,
before a `SymbolId` even exists, so by the time any name reaches `Plan`
it is already-upcased text, and feeding it a hand-written lowercase byte
string tests a rule `slot_of` was never responsible for enforcing.
Rewrote the test to check the real fact instead: `program.symbols.name(id)`
for a compound written `v.i` is already `"V.I"`, confirmed directly,
before ever reaching `slot_of`.

`#[allow(dead_code, reason = "...")]` on `Activation::settings`: nothing
reads or mutates it yet (Task 9's `NUMERIC` is the first to).

## Commit

`ca005497`, "Resolve a body's variables once, keyed by name, cached on
Interp". Staged and committed directly: `rust/crates/rexx-exec/src/plan.rs`
(new), `src/activation.rs` (new), `src/lib.rs`, `src/stem.rs` only --
confirmed via `git status --porcelain` before staging. Re-verified at HEAD
after commit: 20 unit + 10 integration + 2 doctests, clippy clean.
