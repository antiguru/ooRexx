# Task 1 report: `rexx-parse` gives the main body a `CodeBody`

Status: DONE

## Step 1: Measure whether an INTERPRET fragment may contain a label

Ran, wrapped in `( ulimit -v 1048576; build/bin/rexx FILE )`:

Probe 1 -- `/tmp/.../scratchpad/interp_label1.rex`:
```
signal on syntax
interpret "lab: nop"
say "no error"
exit
syntax:
say "condition code:" condition('o')~code
say "condition message:" condition('o')~message
```
Output:
```
condition code: 47.1
condition message: INTERPRET data must not contain labels; found "LAB".
```

Probe 2 -- `/tmp/.../scratchpad/interp_label2.rex`:
```
signal on syntax
interpret "signal lab; lab: nop"
say "no error"
exit
syntax:
say "condition code:" condition('o')~code
say "condition message:" condition('o')~message
```
Output:
```
condition code: 47.1
condition message: INTERPRET data must not contain labels; found "LAB".
```

Both probes are rejected with error 47.1, confirming Phase 3's note. A label inside `INTERPRET` text is
never legal, so **`Fragment`'s label table is always empty**. This is already enforced in the current
codebase at `rust/crates/rexx-parse/src/instruction.rs:520` (`self.error(47, 1)`, doc comment above it
citing the same measurement), so this step confirms existing behaviour rather than surfacing new
behaviour to encode. Proceeding with the brief's design: `Fragment` gets `body: CodeBody` (whose
`labels` map will simply always end up empty), not a load-bearing label table.

## Step 2: Change the two structs

`Program::instructions` and `Program::labels` collapsed into `Program::main: CodeBody`.
`Fragment::instructions` became `Fragment::body: CodeBody`. `parse_program` and `parse_interpret`
both became a move of `parsed.main` (as expected from the context note), not a field split.

Doc comments: nothing was dropped, but content moved to the most accurate site rather than being
copy-pasted onto `Program::main`/`Fragment::body`, because inspecting `translate_block`'s own doc
comment (`block.rs:1022`-`1028`) showed the "ends at the first `::` directive clause or EOF" property
and the "label is local to the body that declares it" property are true of every `CodeBody`, not just
the main one -- `translate_block` is the same function that builds a directive body's `CodeBody`.  So:

* `CodeBody::instructions`'s and `CodeBody::labels`'s own doc comments in `ast.rs` absorbed the fuller
  content that used to live on `Program::instructions`/`Program::labels` (the `::` boundary sentence,
  and the full `Box<[u8]>`-not-`Box<str>` rationale with the Task 3.3 citation and the first-occurrence-
  wins rationale). Nothing here was reworded, only relocated and, where two versions differed in
  detail, replaced with the fuller one.
* `Program::main`'s own field doc is now a short pointer to `CodeBody`'s own doc, plus the one fact
  that's actually specific to `Program`: a directive's own body is a `CodeBody` too, held in
  `directives` rather than in `main`.
* `Program`'s struct-level doc comment used to say the main body's fields were "spelled out here rather
  than held as a `CodeBody` ... because they are this type's public surface and predate it" -- this
  sentence described the OLD design this very task reverses, so it was rewritten to state the NEW
  design and its reason (the borrow-shape rationale from the spec: an evaluator can borrow one
  `&CodeBody` rather than cloning two sibling fields).
* `Fragment`'s struct-level doc used to say "No `directives` and no `labels` fields ... a `Fragment`
  that exists at all has neither." This is no longer true at the type level (`body.labels` exists,
  it's just always empty), so it was corrected to say exactly that, citing both error numbers and
  Task 1's own measurement from Step 1.
* Added a second `debug_assert!` in `parse_interpret`, mirroring the existing one for
  `parsed.directives.is_empty()`, asserting `parsed.main.labels.is_empty()` -- belt-and-suspenders
  for the Step 1 finding, in the same idiom the function already used for directives.

## Step 3: Update callers and tests

Found every caller with `grep -rn '\.instructions\b|\.labels\b'` across the whole `rust/` workspace
(only `rexx-parse` itself has any; no other crate depends on it yet). Distinguished `Program`/`Fragment`
field access from `CodeBody` field access (e.g. a directive's `body.instructions`, which is unaffected)
by checking the binding's origin in each file before editing, since `Block`'s own internal fields are
also named `instructions`/`labels` and must NOT be touched.

Files touched beyond `lib.rs`:
* `tests/program.rs` -- the primary target named in the brief. Mechanical field-access renames
  (`p.instructions` -> `p.main.instructions`, `p.labels` -> `p.main.labels`, `f.instructions` ->
  `f.body.instructions`, same for `f1`/`f2`). Two doc comments that named `Program::labels` or
  `Fragment`'s field shape explicitly were corrected for accuracy (not reworded for convenience):
  the `// ---- Program::labels ----` section header and the doc comment on
  `a_label_inside_a_directive_body_is_not_in_the_programs_own_label_map`, and the doc comment on
  `interpret_accepts_ordinary_text_with_no_directive_and_no_label`, which also gained a
  `f.body.labels.is_empty()` assertion so the corrected claim is actually checked, not just asserted
  in prose.
* `src/block/tests.rs` -- 20+ `program.instructions`/`program.labels` sites (including one `nested =
  ok(...)` binding, also `Program`-typed) renamed the same way. Left `body.instructions`/`body.labels`
  sites alone throughout (those come from `routine.body.as_ref()`, a `CodeBody`, not `Program`).
* `tests/gate_walk/mod.rs`, `tests/tiling.rs`, `tests/sourceline.rs` -- one or two `Program`/`Fragment`
  site(s) each, same rename. `sourceline.rs` had one `fragment.instructions` (not `program.`) that
  needed `fragment.body.instructions` instead.
* `benches/parse.rs` -- `program.instructions.len()` -> `program.main.instructions.len()`. Not named in
  the brief's file list but required for `cargo clippy --all-targets` to compile the bench target.
* `src/lib.rs` -- removed the now-unused `use std::collections::BTreeMap;` (was only there for the two
  struct fields that no longer exist on `Program`/`Fragment` directly).

## Step 4: Verify

Baseline (before the change, via `git stash`): `cargo test -p rexx-parse` = **392 passed, 0 failed**
across all targets (254 unit + 32 errors + 23 program + 1 samples + 35 scanner + 25 sourceline + 1
sourceline_oracle + 11 tiling + 9 tokens + 1 variants + 0 doctests).

After the change (`git stash pop` to restore): **392 passed, 0 failed** -- identical count. `program.rs`
still reports 23 (the new assertion was added inside an existing test, not as a new `#[test]` fn), so
this is a real "same count" match, not a coincidence from a lost test cancelling a gained one.

`cargo clippy -p rexx-parse --all-targets -- -D warnings`: clean, zero warnings.

`cargo fmt --check`: ONE diff, in `benches/parse.rs:150` -- the `program.instructions.len()` ->
`program.main.instructions.len()` rename pushed that line past rustfmt's width limit, so rustfmt wants
to break the tuple onto multiple lines. Per the brief's global constraint ("Run cargo fmt only as its
own commit if needed, never mixed into a substantive one"), this is fixed in a SEPARATE commit after
the substantive one below, not folded into it.

## Step 5: Commit

Substantive commit `5a5feadc`: "Give the main program body a CodeBody, so the executor can borrow
one" -- staged exactly `rust/crates/rexx-parse`, message verbatim from the brief.

Follow-up commit `cb80e2a9`: `cargo fmt` on its own, fixing the one line-width diff `benches/parse.rs`
picked up from the rename (see Step 4). Not mixed into the substantive commit, per the global
constraint.

Final verification re-run against the committed tree: `cargo test -p rexx-parse` 392/392 passed,
0 failed; `cargo clippy -p rexx-parse --all-targets -- -D warnings` clean; `cargo fmt --check` clean.

## Summary for the team lead

* Step 1 confirmed: an `INTERPRET` fragment can never contain a label (measured 47.1 both ways),
  matching Phase 3's note and the existing enforcement at `instruction.rs:520`. No design change
  needed; proceeded with the brief as written.
* `Program::instructions`/`Program::labels` -> `Program::main: CodeBody`.
  `Fragment::instructions` -> `Fragment::body: CodeBody`. `CodeBody`'s own derives untouched.
* All doc comments preserved; the `Program`/`Fragment`-specific paragraphs that turned out to be
  true of every `CodeBody` (confirmed against `translate_block`'s own doc) were consolidated onto
  `CodeBody`'s own fields in `ast.rs` rather than duplicated per site.
* Test count unchanged (392/392) before and after; clippy and fmt both clean on the final tree.
* No `unsafe`, no oracle files touched, no `.superpowers`/scratchpad boundary crossed.
