# Task 1: re-grain `owners.rs` to arm granularity and assert `lib.rs` against it

**Status: done.** Commits `337217da` (the consolidation) and `f9cfb060` (three
false sentences it exposed). Tree clean.

Base `222d438d`. Verification at both commits: `cargo test --workspace` **1020
passed / 0 failed**, `cargo fmt --all --check` and
`cargo clippy --workspace --all-targets -- -D warnings` clean, corpus **9/9**,
assertions **5/5**, keyword **7/7**, `./scripts/mutate-4b.sh` **12 of 12 as
declared** -- every figure identical to the baseline.

## Step 1: the three inherited measurements

All three held exactly as given. Nothing was substituted.

1. `instruction_owner` (`src/lib.rs`) had exactly one nested match,
   `InstructionKind::Call(call) => match &**call`, splitting
   `Named`/`Dynamic`/`Trap` -> `None` from `Qualified` -> `Some("Phase 5")`.
2. `expr_owner` had none. The only `match` token in its body is the outer one;
   a second grep hit was the word "match" inside a comment, not an expression.
3. `expand_for_witnesses` (`tests/loud.rs`) had one real entry,
   `"Call" => vec!["Call::Qualified"]`, plus `other => vec![other]`.

One fact worth adding to the brief's three, because it shaped the
implementation: `InstructionKind::Call` holds a `Box<Call>`. Box patterns are
unstable, so `InstructionKind::Call(Call::Named { .. })` is not a pattern that
can be written and the four arms cannot become four ordinary rows. They are
only reachable through a nested `match` on a dereference.

## Step 2: the table

`owners.rs`'s `tags!` gains a second rule -- a trailing
`split PATTERN in (EXPR) { .. }` section -- which expands to a nested `match`
and contributes one row per arm to the generated list. `InstructionKind::Call`
takes it and becomes four rows: `Call::Named`, `Call::Dynamic`, `Call::Trap`
in scope, `Call::Qualified` -> `Phase 5`. Every other row is untouched.

Both matches stay wildcard-free, so a new variant of `InstructionKind` *or* of
`rexx_parse::Call` is still a compile error here. The section is trailing (and
its rows therefore land at the end of `INSTRUCTION_TAGS`) because a
`macro_rules` matcher cannot alternate two row shapes inside one repetition
without a token muncher; every consumer sorts or counts, so the order carries
nothing.

Consequences, all of which the harness caught rather than reasoning
predicting: `INSTRUCTION_TAGS.len()` 40 -> 43, `InScope` 28 -> 31,
`EXPECTED_OUT_OF_SCOPE`'s `Call` row -> `Call::Qualified`. The `Phase 5` count
stays 7 and the phase-owned tag count stays 12, because the split is 1:1 at
the loud end.

`Call::Named`/`Dynamic`/`Trap` becoming `InScope` means criterion 1 now demands
a corpus witness for each. Checked before implementing rather than after: the
4a+4b subset union already constructs all three (`call outer`,
`call (target)`, `call on user marker name trap_user`), and
`every_in_scope_variant_is_witnessed_by_the_phase_subsets` passes unchanged.

## Step 3: the assertion

The grains now match, so `INSTRUCTION_WITNESSES`' tags are the table's
phase-owned rows one for one and `assert_witness_set_is_complete` uses a plain
`.map`.

The assertion itself is the removal of `Witness`'s hand-written `owner` field.
A new `table_owner` reads the owner out of `owners.rs` by tag, and
`every_out_of_scope_variant_fails_loudly` then requires the running executor's
stderr to end with exactly that string. So the chain is
`owners.rs` -> `run_program` -> `lib.rs`'s `instruction_owner`/`expr_owner`,
with nothing hand-maintained in between. Previously the witness's own `owner`
literal sat in the middle, and a witness agreeing with `lib.rs` while both
disagreed with `owners.rs` would have gone unnoticed.

**What it does not cover, stated because overclaiming here would be the
defect.** `Loud::instruction` is only reached for a variant the executor
declines, so the `None` arms are unreachable data: a phase string written onto
an implemented variant is invisible to any assertion. `corpus.rs`'s
byte-for-byte differential run covers that direction, by failing the moment an
implemented construct starts printing a gap. Both files now say this rather
than implying full equality.

## Step 4: what the compiler said was dead

Determined by removing and compiling, not by reading call sites. Three things:

* **`expand_for_witnesses`** -- the expected casualty.
* **`instruction_arm`** -- did *not* survive. Its display role is gone:
  `owners::instruction_tag` now answers the arm-grained tag directly, so
  `assert_constructs` calls it instead. (It also named `Signal`'s three arms,
  which no witness ever looked up.)
* **`loud.rs`'s `SPLIT_TABLE_PHASES` import**, which the compiler flagged
  unused once the per-witness phase check went. That check was strictly
  redundant: `owners.rs`'s own `assert_owner_strings_are_split_table_phases`
  holds every `Owner::Phase` row of all seven tables to that set, including
  the rows `loud.rs` never asks about.

## Step 5: the falsification

The gate predicted this consolidation would *move* the duplication into the
reconciler. Three mutations, each restored from a scratchpad copy rather than
from git.

**M1 -- a fully coherent edit on the `owners.rs` side.** `Address` 4c ->
Phase 5, with `EXPECTED_OUT_OF_SCOPE` and both phase counts updated to match:
exactly what a wrong-but-internally-consistent plan amendment looks like.
Every other test in the harness stayed **green** -- the pinned literal, the
counts, the coverage walk, `assert_witness_set_is_complete` -- and only the new
assertion went red, naming both sides:

```
Address (Phase 5): stderr does not end with " is not implemented (Phase 5)":
  "rexx-exec: ADDRESS is not implemented (4c)\n"
```

That isolation is the result that matters. The assertion is comparing against
`lib.rs`, not against another copy inside the test tree.

**M2 -- the production side.** `lib.rs`'s six `Some("Phase 5")` instruction
owners -> `"4c"`, `owners.rs` untouched. Caught, six rows at once.

**M3 -- the arm the split exists for.** `lib.rs`'s
`Call::Qualified => Some("Phase 5")` -> `Some("4c")`. Caught, naming
`Call::Qualified`, which proves the newly split row is itself load-bearing and
not decoration.

In every case the test ran (non-zero count, named in the output), so this is
not `cargo test` matching nothing.

## Step 6: the frozen message

All 16 witness programs were run through `rexx-run` before any edit and again
after, capturing stderr and exit status. `diff` of the two captures is
**empty** -- byte-identical message text and rc 120 throughout. The keyword
gate, whose 790 derived rows depend on that text, is 7/7 at both commits.

## The second commit

A sweep for ownership claims in files this task did not otherwise open found
three false sentences.

* `loud.rs`'s "no `owner: "4b"` witness remains in this list" -- falsified by
  this task, since the field is gone.
* `coverage.rs`: "two of `rexx_parse::Call`'s four arms (`Trap`, `Qualified`)
  are still loud" -- already false before this task; only `Qualified` is.
* `coverage.rs`: "The five that remain (... `VariableReference` ...) split
  across two phases, `4b` and `Phase 5`" -- already false; there are four,
  and all four are Phase 5's.

The last two are pre-existing and `coverage.rs` is Task 2's file, but they are
about exactly the data being consolidated and both figures are asserted, so
correcting them here was cheaper than handing on a known-false statement.

`loud.rs`'s separate "five `ExprKind` assignments that are a Task 16 gate-time
judgement call" was checked and **left alone**: it is true, and it cites
`phase-4-exclusions.txt`, which records five of the original six rows as that
call. It counts a decision, not a current table.

## For the 4c planner

Two rows of `phase-4b-gate.md`'s deferred-minors table are now moot, their
subjects deleted: **T0-M2** (`expand_for_witnesses` not exhaustiveness-checked
against `Call`/`Signal`) and **T0-M4** (`instruction_arm` returns `String`
where `&'static str` suffices).

`phase-4b-gate.md`'s Step 3b still reads, in the present tense, that a straight
equality assertion "is therefore **not available**". That is now false of the
tree, and it is left as written because the gate is a dated record and the
pre-4c plan already states the reason expired. A 4c planner reading the gate
directly should know the sentence has been overtaken.

## Scope note

`src/lib.rs` was edited, though the plan lists only the two test files. Its
`instruction_owner` doc paragraph described this consolidation as costed and
declined, which doing it made false, and one line said the `Call` split matched
`loud.rs`'s witness table when it now matches `owners.rs`'s. Both are
corrections to statements this task falsified, not prose culling; Task 2's 73
lines in that file are untouched.

---

# Fix round 1 (`7900224e`)

Review verdict was PASS/PASS with 3 Important + 4 Minor. **All seven addressed;
three needed no edit because Task 2's cull had already removed their subjects.**

**The tree had moved.** The review's line numbers were against `f9cfb060`, but
Task 2 landed eight commits on top (`fd0bcee6`..`96ad0d15`), rewriting
`owners.rs`, `loud.rs`, `coverage.rs` and `lib.rs`. Every finding was therefore
re-derived against HEAD rather than applied by line number.

## I3 -- the one that mattered, and where the proposed fix was also wrong

My sentence "corpus.rs covers that direction" contradicted its own preceding
clause. The review is right that it is false, and right that an overclaiming
disclaimer is worse than none.

Measured before rewriting:

| mutation to `instruction_owner` | `cargo test --workspace` |
|---|---|
| `InstructionKind::Say` -> `Some("Phase 5")` | **1020 passed / 0 failed** |
| `Do`/`Loop` -> `Some("Phase 5")` | **4 failed** (`run.rs`) |

So nothing covers the `Say` case, exactly as the review measured. **But the
proposed replacement -- "nothing covers it, and nothing needs to, because no
path reads it" -- is false as a blanket statement.** `run_loop` does reach
`instruction_owner` for `Do`/`Loop`, through the two documented edge cases
where the instruction is implemented and only the specific reason is not, and
four `run.rs` tests assert the exact unsuffixed message: `do_with_takes_the_
loud_path`, `do_counter_takes_the_loud_path_regardless_of_which_other_kind_it_
rides_on`, `do_over_a_stem_target_takes_the_loud_path`, `do_over_a_
parenthesised_stem_target_is_also_caught` (all four confirmed present at HEAD).

All three sites now state the unreachability, name the one exception, and point
at no test that is not in fact watching.

## The rest

* **I1** `corpus/phase-4b.txt:93-102` -- fixed. Confirmed live and confirmed
  false: the header is the standing rule for the 4b subset and `f9cfb060`'s
  sweep did not reach it. The caveat has no subject left, since there is no
  coarse `Call` tag to misread, so the rule is now read off the table directly.
* **M2** `owners.rs` -- fixed. "One row per owner" was literally false (43 rows,
  five owners); the sense is one row per separately owned unit.
* **M3** `phase-4b-gate.md` -- forward pointer added at Step 3b naming
  `337217da` and the three present-tense claims that no longer hold, plus
  T0-M2/T0-M4 as moot. The assessment itself left exactly as written.
* **I2, M1, M4** -- no edit needed; Task 2's `514051f1` and `8c9321c2` had
  already removed all three subjects. Verified by grep at HEAD rather than
  assumed.

## Verification

1020 passed / 0 failed, fmt and clippy clean, corpus 9/9, assertions 5/5,
keyword 7/7, `mutate-4b.sh` 12 of 12 as declared.

`mutate-4b.sh` aborted twice on a transient toolchain fault (`rustc -vV`
failing, `cargo`/`rustc` briefly "not applicable to the stable toolchain" --
most likely a concurrent `rustup` operation). Recorded because the script's
refusal to score those runs is the correct behaviour and worth not mistaking
for a result: it reported `INFRA_FAILURE` and stopped rather than reporting a
row as caught. The toolchain was healthy on re-check and the third run
completed.
