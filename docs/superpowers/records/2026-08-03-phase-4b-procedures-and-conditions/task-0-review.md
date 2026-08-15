# Task 0 review -- shared owner table, arm-grained loud witnesses, owner in every loud message

Reviewer: independent (opus). BASE `dc87708d`, HEAD `0197b360`.
Diff read in full; every claim below was re-derived in the tree, not taken from the report.

---

## Verdicts

**1. Spec compliance: MET WITH GAPS.**
Steps 1, 2, 4 and 6 are met in full. Step 3 is met in code and not met in the plan
(the format was never recorded where Step 3 says to record it, I4). Step 5 is met in
substance but placed outside the module doc the step names, and mislabelled as being
in it (M1). Nothing in the diff exceeds the brief's scope: the six files touched are
exactly the six named, no C++ file is touched, and no `unsafe` appears.

**2. Task quality: PASS WITH CHANGES REQUIRED.**
The three mechanisms the task exists to install genuinely work -- I proved each by
mutation, below. The defects are concentrated in one place: the *new* behaviour
Step 3 adds is only partly held in place by assertions. Two deliberate,
argued-at-length design decisions (the `"4a"` carve-out, the exact message shape) are
asserted by nothing at all, and one arm of the new drift check is structurally
unable to fail.

**Counts: Critical 0 -- Important 6 -- Minor 7.**

---

## Verification performed (not taken from the report)

| Check | Command | Result |
|---|---|---|
| Full suite | `cargo test -p rexx-exec` | exit **0**, 12 binaries, 0 failed, 0 `FAILED` markers |
| Corpus | inside the above | `29 of 29 matching`, unchanged |
| Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | exit **0** |
| Format | `cargo fmt --all --check` | exit **0**, no diff |
| Format (as reported) | `cargo fmt --edition 2024 --check` | exit **2**, `error: unexpected argument '--edition' found` -- see M6 |

Exit statuses read unpiped in every case.

### Live message probe (27 witness sources + the two `run.rs` edge cases)

Built `rexx-run` and ran each witness source through it. The format
`"{name} is not implemented ({owner})"` **is** emitted live, for every out-of-scope
construct, all at rc 120:

```
call sub         -> rexx-exec: CALL is not implemented (4b)
call (x)         -> rexx-exec: CALL is not implemented (4b)
call ns:sub      -> rexx-exec: CALL is not implemented (Phase 5)
call off error   -> rexx-exec: CALL is not implemented (4b)
signal there     -> rexx-exec: SIGNAL is not implemented (4b)
signal off error -> rexx-exec: SIGNAL is not implemented (4b)
say foo(1)       -> rexx-exec: a function call is not implemented (4b)
'date'           -> rexx-exec: a command is not implemented (Phase 7)
say a~b          -> rexx-exec: a message send is not implemented (Phase 5)
parse var x y    -> rexx-exec: PARSE is not implemented (4c)
   ... (all 27 checked; owners 4b / 4c / Phase 5 / Phase 7 as declared)
do counter c 5   -> rexx-exec: DO is not implemented        <- no suffix, as reported
do with index i over x -> rexx-exec: DO is not implemented  <- no suffix, as reported
```

The report's format claim and its claim about the two `run.rs` sites are **both
accurate as descriptions of today's behaviour**. What is not true is that either is
held in place by a test (I1, I2).

### Mutation battery -- do the three copies actually stay honest?

Working tree restored and confirmed clean (`git diff HEAD --stat` empty) after each.

| # | Mutation | Expected | Actual |
|---|---|---|---|
| A | `owned_message` never appends the owner | fail | **caught** -- `every_out_of_scope_variant_fails_loudly` |
| B | `instruction_owner`: `Call::Qualified` `"Phase 5"` -> `"4b"` | fail | **caught** -- `Call::Qualified (Phase 5): stderr does not name this owner: "rexx-exec: CALL is not implemented (4b)\n"` |
| C | `expr_owner`: `VariableReference` `"4b"` -> `"Phase 7"` | fail | **NOT CAUGHT -- suite green** (I3) |
| D | `expr_owner`: `List` `"Phase 5"` -> `"4c"` | fail | **caught** -- `List (Phase 5): stderr does not name this owner: "... a parenthesised list is not implemented (4c)"` |
| E | delete the `Call::Trap` witness row | fail | **caught** -- `assert_witness_set_is_complete`, diff names `Call::Trap` |
| F | delete the `"4a"` carve-out (always append) | fail | **NOT CAUGHT -- full `cargo test -p rexx-exec` exit 0** (I1) |
| G | rewrite the shape to `"[{owner}] {name}: unimplemented"` | fail | **NOT CAUGHT -- loud/coverage/owners/corpus/assertions all green** (I2) |

So: the cross-copy drift check the report leans on is real and discriminating (A, B, D, E),
which answers the lead's question affirmatively -- with the two holes C and F, and the
shape itself unpinned (G).

---

## Important findings

### I1 -- The `"4a"` carve-out is asserted by nothing; deleting it leaves the whole suite green

`rust/crates/rexx-exec/src/lib.rs:577-581` (`owned_message`).

Mutation F: replacing the body with an unconditional
`format!("{name} is not implemented ({owner})")` leaves **`cargo test -p rexx-exec` at
exit 0, zero failures**. The two sites this carve-out exists for --
`rust/crates/rexx-exec/src/run.rs:1712` (`COUNTER`/`DO WITH`) and
`rust/crates/rexx-exec/src/run.rs:1882` (stem-target `DO OVER`) -- are covered by
`do_with_takes_the_loud_path` (`run.rs:5069`),
`do_counter_takes_the_loud_path_regardless_of_which_other_kind_it_rides_on`
(`run.rs:5078`), `do_over_a_stem_target_takes_the_loud_path` (`run.rs:5200`) and
`do_over_a_parenthesised_stem_target_is_also_caught` (`run.rs:5218`) -- and every one
of those four asserts only `let Failure::Loud(_) = failure else { panic! }`. None
inspects the message. No corpus program reaches those paths either (29 of 29 match, so
no corpus program takes a loud path at all).

The 22 lines of doc comment on `owned_message` defending this decision therefore
describe behaviour that a later task can delete without a single test noticing.

**Fix:** add one assertion beside the existing four, e.g. in `run.rs`'s test module

```rust
let Failure::Loud(loud) = failure else { panic!(...) };
assert_eq!(loud.message, "DO is not implemented",
    "a construct 4a does implement must not be attributed to a phase; see owned_message");
```

for at least `do_counter_...` and `do_over_a_stem_target_...`. (`Loud.message` is
crate-private and `run.rs`'s tests are in-crate, so this needs no new API.)

### I2 -- The message shape is a documented cross-task contract that nothing pins

`rust/crates/rexx-exec/tests/loud.rs:581` -- `if !stderr.contains(witness.owner)`.

`contains` is satisfied by any string mentioning the owner anywhere. Mutation G
rewrote the message to `[4b] CALL: unimplemented` and `loud`, `coverage`, `owners`,
`corpus` and `assertions` **all stayed green**. Two consequences:

* The report and `progress.md` both tell later tasks the shape is
  `"{name} is not implemented ({owner})"` and to rely on it. Nothing enforces that.
* `rust/crates/rexx-exec/tests/corpus.rs:283` (`owner_from_stderr`) and
  `rust/crates/rexx-exec/tests/assertions.rs:291` (`construct_from_stderr`) both key on
  the literal `" is not implemented"`. Under mutation G both silently returned `None`,
  so every `RuntimeBlocked` row in `assertions.rs` (~4,259 rows, 35 exempt) was labelled
  `"<unnamed>"` (`assertions.rs:674`, `:731`, `unwrap_or_else`) and every corpus
  mismatch would have been reported as an unclassified divergence -- with no test
  failing. The classifiers are diagnostic-only, so the damage is to diagnosis, not to
  pass/fail; but it is exactly the "measurement quietly goes to zero" shape the
  `EXPECTED_SUBSET` pin was added to prevent.

**Fix:** make the assertion pin the shape, not just the substring --

```rust
let want = format!(" is not implemented ({})", witness.owner);
if !stderr.trim_end().ends_with(&want) { ... }
```

This subsumes the current check and would have failed mutations A, B, D and G.

### I3 -- `expr_owner`'s `VariableReference` arm cannot be checked by the new assertion

`rust/crates/rexx-exec/src/lib.rs:677` and `rust/crates/rexx-exec/tests/loud.rs:377-385`.

The `VariableReference` witness source is `"call sub >x\n"`. That program fails loudly
on the **`CALL` instruction**, not on the expression, so the stderr the new assertion
inspects is `rexx-exec: CALL is not implemented (4b)` -- produced by
`instruction_owner`, never by `expr_owner`. Mutation C set
`expr_owner(ExprKind::VariableReference)` to `"Phase 7"` -- not even a legal owner for
it -- and the suite stayed green.

`loud.rs:380-382`'s comment is now partly false:

```rust
// `Call` itself is 4b's too, so the loudness this produces is not
// confounded: nothing here depends on `VariableReference` doing
// anything differently for the exit code to be `NOT_IMPLEMENTED_EXIT`.
```

That argument holds for the **exit code** (its stated subject) and does not transfer to
the owner, which this task just made the second thing that witness asserts. The two
copies agreeing here is coincidence, not verification.

**Fix:** no new mechanism is available today -- `VariableReference`'s only legal position
is a call argument list, so `CALL` will always fire first until Task 3 lands. Record
that honestly beside the witness and hand it to Task 3:

```rust
// The owner half of this witness is confounded, unlike the exit-code half
// above: `CALL` fails first, so the stderr checked below comes from
// `instruction_owner`, never `expr_owner`. Task 3 (which implements
// `Call::Named`) makes this witness reach the expression and the check real.
```

and add a line to `owners.rs`'s pinned-items block naming
`expr_owner(VariableReference)` as the one arm the drift check does not cover.

### I4 -- Step 3's "record it here" was not done: the plan still carries no format

`docs/superpowers/plans/2026-08-03-phase-4b-procedures-and-conditions.md:251`.

Step 3 reads: *"take the exact format from what reads best beside the existing text, and
**record it here** so later tasks assert the same shape."* The plan file is unchanged by
this commit; line 251 still contains the instruction with no format filled in, and
line 38 still says the emitted text is `format!("{name} is not implemented")` and names
no phase. Line 38's prohibition is correctly conditional (*"until Task 0 lands"*), so it
is not actively wrong -- but a Task 3/6/7 implementer working from the plan alone has no
way to know what shape to assert. The format currently lives only in
`task-0-report.md` and in the lead's `progress.md` ledger entry.

**Fix:** amend plan line 251 (and line 38's measured-text sentence) to state
`rexx-exec: {name} is not implemented ({owner})` with a live example, and note the
owner-less exception at `run.rs:1712`/`:1882`.

### I5 -- `read_subset`'s union and de-duplication -- the only new runtime logic -- has no test

`rust/crates/rexx-exec/tests/coverage.rs:410`, `corpus.rs:194`, `collect_stress.rs:105`.

All four call sites pass a one-element slice
(`coverage.rs:484`, `:506`; `corpus.rs:478`; `collect_stress.rs:126`). The
multi-file loop, the `HashSet` de-duplication and the first-seen ordering are therefore
never executed with more than one path in any test. A defect in that path -- wrong order,
a dropped entry, de-duplicating across files when it should not -- lands silently on the
first later task that passes two files, which is precisely the task that will be busy
debugging its own new code.

**Fix:** one test in `coverage.rs` (the harness that already owns the pinned subset list)
writing two temp list files with an overlapping entry and asserting the union's exact
contents and order. About ten lines.

### I6 -- `"4a"` doubles as the sentinel for "this crate implements it", which traps 4b/4c

`rust/crates/rexx-exec/src/lib.rs:577` (`owner == "4a"`) and `:626-673`
(`instruction_owner`'s doc: *"`"4a"` for a variant 4a already implements"*).

The carve-out keys on a phase name, but the property it means is "already implemented
here". When Task 3 implements `CALL`, the implementer has two bad options: label
`InstructionKind::Call` `"4a"` (false -- 4b implemented it, and it puts a 4b variant in a
group named "4a's own twenty"), or leave it `"4b"` and ship
`rexx-exec: CALL is not implemented (4b)` from any 4b-local blocked site -- the exact
self-contradiction the carve-out exists to prevent, now with no carve-out covering it.
This is the same class of forward trap as the three mechanisms Task 0 was written to
remove, and it is cheap to close now and awkward to close later.

**Fix:** make the sentinel say what it means. Either

```rust
fn instruction_owner(kind: &InstructionKind) -> Option<&'static str>  // None == implemented here
```

with `owned_message` taking `Option`, or a named constant
`const IMPLEMENTED_HERE: &str = "";` used in both `instruction_owner` and `expr_owner`
in place of `"4a"`. Either removes the phase-name coupling and survives 4b, 4c and
Phase 5 unchanged.

---

## Minor findings

### M1 -- Step 5's block is at the end of the file, not in the module doc, and is called the module doc

`rust/crates/rexx-exec/tests/owners.rs:476-517` (a trailing `//` comment block) and
`owners.rs:1671-1672`-region doc on `EXPECTED_OUT_OF_SCOPE`, which says
*"this file's own module doc (below, 'What is pinned here')"*.

Step 5 says the module doc, and gives the reason: *"This is the only place a later
implementer will look."* The block is at the bottom of a 517-line file, the module doc
(`owners.rs:12-50`) never mentions it, and the one pointer to it calls it the module doc.
A `//` block after the last `#[test]` is also invisible to `cargo doc`.

**Fix:** move the five-item block into the `//!` module doc as a `# What is pinned here`
section, next to the two sections already there, and correct the `EXPECTED_OUT_OF_SCOPE`
pointer.

### M2 -- `expand_for_witnesses` is not exhaustiveness-checked against `rexx_parse::Call`/`Signal`

`rust/crates/rexx-exec/tests/loud.rs:1071-1082`.

It matches on `&'static str` with a `other => vec![other]` catch-all. Dropping an arm is
caught (`assert_eq!(expected_instructions.len(), 24)` and the set comparison both bite --
mutation E). **Adding** one is not: a new `rexx_parse::Call` arm forces a compile error in
`instruction_arm` (`loud.rs:1039`) and in `instruction_owner` (`lib.rs:648`), both
exhaustive, but `expand_for_witnesses` compiles unchanged, the expected count stays 24
because it is derived from `expand_for_witnesses` itself, and the new arm gets no
loudness witness. That is the brief's Mechanism 1 one level down.

Answering the lead's question directly: `instruction_arm` and `expand_for_witnesses` were
both genuinely needed, but the hand-maintained string list is not the smallest *safe*
form -- the repo already ships the device that fixes it.

**Fix:** apply the existing `tags!` macro (`owners.rs:1473`) to `rexx_parse::Call` and
`rexx_parse::Signal`, producing `CALL_TAGS`/`SIGNAL_TAGS` with no wildcard arm, and derive
both `instruction_arm` and `expand_for_witnesses` from them.

### M3 -- `the_two_harnesses_include_this_exact_file` is defeated by the regression it names

`rust/crates/rexx-exec/tests/owners.rs:1858-1874`.

It asserts `coverage.rs`/`loud.rs` contain the literal `#[path = "owners.rs"]` anywhere in
their source. A file that re-added a hand-copied `Owner`/`tags!` block *and* kept the
attribute (or merely mentioned the string in a comment) passes. The named failure mode is
"a future edit quietly reverted one of the two back to a hand-copied table" -- exactly the
case a text search for the *presence* of one line cannot detect.

**Fix:** also assert absence of the copied artefacts:
`assert!(!text.contains("macro_rules! tags"))` and `assert!(!text.contains("enum Owner"))`,
with a message naming I36.

### M4 -- `instruction_arm` returns `String` where `&'static str` suffices

`rust/crates/rexx-exec/tests/loud.rs:1039-1053` -- seven `.to_string()` calls, all on
static literals. I changed the signature to `-> &'static str`, deleted every
`.to_string()`, and `cargo test -p rexx-exec --test loud` passed 8/8. It is smaller and
avoids the allocation in `assert_constructs`'s inner `.any(...)` closure.

**Fix:** apply that change (verified to compile and pass).

### M5 -- the "unavoidably a third copy" claim overstates the constraint

`rust/crates/rexx-exec/src/lib.rs:626-635` and `owners.rs:1907-1914` (pinned item 5),
both asserting the third copy is unavoidable because *"production code cannot depend on
anything under `tests/`"*.

The stated reason is true; it does not establish unavoidability, because the dependency
runs the other way perfectly well: `loud.rs` already does
`use rexx_exec::{NOT_IMPLEMENTED_EXIT, run_program};`. `rexx_exec` could export the owner
data and `owners.rs` could derive its tables from it -- one copy, not three. That is a
real option with a real cost (the crate's public API is deliberately 7 items;
test-only exports would widen it), so the *decision* is defensible. The *comment* is not:
it presents a chosen trade-off as a forced one, which is how the next reader stops looking.

**Fix:** reword to "deliberately a third copy: merging it would mean exporting
test-only ownership data from this crate's public API, which is kept to
`run_program`/`Outcome`/`NOT_IMPLEMENTED_EXIT` and their kin. The cost of that
outweighed the cost of the third copy, which `loud.rs`'s stderr assertion keeps honest."

### M6 -- the reported `cargo fmt` invocation does not run

`task-0-report.md:153-155` claims `cargo fmt --edition 2024 --check` was "clean on every
changed file", read unpiped. That command exits **2** with
`error: unexpected argument '--edition' found` on this toolchain (cargo/rustfmt 1.9.0).
`cargo fmt --all --check -- --edition 2024` exits 1 with
`Option 'edition' given more than once` (the edition is already set in the manifest).

Formatting itself is fine: `cargo fmt --all --check` exits 0 with no diff, so there is
nothing to correct in the code. The finding is that a claimed verification step was not
actually performed as described.

**Fix:** none in code. The working invocation for this tree is `cargo fmt --all --check`;
worth correcting in the brief's global constraints so later tasks do not re-report a
command that errors as "clean".

### M7 -- two comments now describe a stderr shape that no longer exists

* `rust/crates/rexx-exec/tests/corpus.rs:58`: *"exits `NOT_IMPLEMENTED_EXIT` with
  `rexx-exec: X is not implemented` on stderr, naming the construct."*
* `rust/crates/rexx-exec/tests/corpus.rs:282` and
  `rust/crates/rexx-exec/tests/assertions.rs:285`: *"Pulls `X` out of a
  `rexx-exec: X is not implemented` line."*

The line now ends `... is not implemented ({owner})`. Both extractors still work (they
`find` the suffix and take the prefix -- verified), so this is comment rot only, but
`corpus.rs:58` is the module doc a later reader consults to learn the marker's shape, and
`corpus.rs:334`'s report string re-emits `"{construct} is not implemented"`, discarding
the owner this task just added.

**Fix:** update both to `rexx-exec: X is not implemented (OWNER)`, and consider carrying
the owner through into `corpus.rs:334`'s mismatch line, which is the report the phase's
progress is read from.

---

## Requirements table

| Step | Requirement | Verdict |
|---|---|---|
| 1 | `tests/owners.rs` created as the single owner table | **met** |
| 1 | Both harnesses read it (`#[path]`) | **met** -- verified in both files; tests run 3x as documented |
| 1 | A test that the two see the *same* table | **met**, weakly -- M3 |
| 2 | `Call::Named` / `Call::Dynamic` own rows, owner `4b` | **met** |
| 2 | `Call::Qualified` own row, `Phase 5` | **met** -- live message confirms |
| 2 | `Call::Trap` own row | **met** |
| 2 | `Signal::Trap` own row | **met** |
| 2 | Each witness verified to construct the arm it claims | **met** -- `assert_constructs` uses `instruction_arm`; mutation E shows completeness bites |
| 3 | Owner in the loud message | **met** -- emitted live, all 27 witnesses |
| 3 | `every_out_of_scope_variant_fails_loudly` asserts the owner in stderr | **met** -- mutations A/B/D caught; holes I3 |
| 3 | Record the format for later tasks | **not met** in the plan -- I4 |
| 4 | `read_subset(&[&Path])`, union semantics | **met** in signature and body; untested -- I5 |
| 4 | All four call sites (2 in `coverage.rs`) | **met** -- `coverage.rs:484`, `:506`, `corpus.rs:478`, `collect_stress.rs:126` |
| 4 | Each harness keeps its own copy | **met** |
| 5 | Pinned literals documented in `owners.rs`'s module doc | **partially met** -- M1 |
| 6 | Full suite green, no behaviour change | **met** -- re-run independently; 29 of 29 unchanged |
| -- | No `unsafe` | **met** |
| -- | C++ tree untouched | **met** -- diff is `rust/crates/rexx-exec/` only |
| -- | No existing comment deleted to ease a change | **met** -- the `coverage.rs`/`loud.rs` comments moved into `owners.rs`, adapted where the audience changed, none dropped |
| -- | `--` not em-dashes | **met** |
| -- | clippy / fmt clean | **met** (fmt via the working invocation -- M6) |
