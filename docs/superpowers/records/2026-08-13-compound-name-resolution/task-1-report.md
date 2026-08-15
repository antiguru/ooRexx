# Task 1 report -- the split, computed once

Plan: `docs/superpowers/plans/2026-08-13-compound-name-resolution.md`.
Base: `cb4f27d1c`. Commit: `b3e8cf4d2`. Fix round: `67927f216`.

## What changed

`Plan` gains `compounds: Box<[Option<CompoundName>]>`, filled by the pass that already walked every compound name, and `Interp::tail_key` reads it instead of calling `rexx_parse::ast::compound_parts` on the interned spelling at every reference.

* **`rust/crates/rexx-exec/src/plan.rs`**
  * New `CompoundName { stem, tails }` and `TailPiece::{Constant, Variable}`, owned byte strings.
  * New `CompoundName::split(&str)`, the **single** definition of the split, entered both by the upfront pass and by `tail_key`'s fallback -- so the two cannot come to disagree.
  * New field `Plan::compounds`, sized in `build` from `symbols.len()` and written by id.
  * New `Plan::compound(id)`.
  * `note_compound_name` takes the compound's id and `&str`, keeps the `CompoundName` it built, and assigns exactly the slots it assigned before.
  * `bind` takes `&str` and records the split of a compound-shaped spelling, assigning no new slots.
* **`rust/crates/rexx-exec/src/lib.rs`**
  * `Code::indents` renamed to `Code::plan` -- the field was already `Option<&Plan>` and now serves two tables, so the old name was false.
  * New `Code::compound(id)` (the table, or `None`) and `Code::stem_name(id)` (the table's stem, or `compound_parts`' stem).
  * New `#[cfg(test)] planned_code(program, plan)`.
* **`rust/crates/rexx-exec/src/stem.rs`** -- `tail_key` reads `Code::compound`; the join moves to `Interp::join_tails`, entered identically by both paths.
* **`rust/crates/rexx-exec/src/eval.rs`** -- the two stem-name sites (`read_symbol`'s `Compound` arm and `echo_symbol_read`) use `Code::stem_name`.
* **`rust/crates/rexx-exec/src/run.rs`** -- `Code` construction renamed; the same stem-name substitution at `assign_expr_target`'s `Compound` arm, the controlled-loop step's `NameShape::Compound` arm, and `drop_variable`'s `Direct` arm.
* **`rust/crates/rexx-exec/src/parse_template.rs`** -- the same substitution at `exec_parse`'s `NameShape::Compound` arm.
* **`rust/crates/rexx-exec/tests/ir_dual_cases/compound-names`** -- new, ten stanzas, every expected byte measured against the oracle.

### Files beyond the brief's list, and why

The brief names `plan.rs`, `stem.rs`, `eval.rs`.
`lib.rs` is where `Code` lives, so the field and its two accessors could go nowhere else.
`run.rs` had to change with it (the field is constructed and read there).

Having `Code::stem_name` in hand, the **four** other sites that were splitting the name for the stem alone were converted too: three in `run.rs` and one in `parse_template.rs`.
They are the same one-line substitution the brief's two `eval.rs` sites are, and one of them -- `assign_expr_target` -- is the compound *write* path, at least as hot as the read.
Leaving them would have left `compound_parts` on the hot path the plan exists to take it off.

## Step 1: how the table is addressed, and what the density measurement found

**Established from `rexx-parse`'s interner, not assumed.**
`SymbolTable::intern` (`token.rs:149`) assigns `SymbolId(u32::try_from(self.names.len())...)` before pushing, and `SymbolTable::name` indexes `self.names[id.0 as usize]` directly, so ids are **dense and zero-based within the table that interned them**.
`SymbolId::index()` already exists and its doc comment states exactly that guarantee; `rexx-parse`'s own `symbol_ids_are_dense_and_zero_based` pins it, including that re-interning a different spelling of the same name consumes no index.

The density is per **table**, and a table is per **program** while a `Plan` is per **body**, so a body's plan sees a sparse subset of a dense range.
That makes a `Vec` indexed by id *correct* but potentially wasteful, and the waste is what I measured rather than guessed.

Throwaway probe (added to `plan.rs`'s test module, run, and removed): for every program that parses under `rust/corpus/`, `rust/bench-programs/` and `/home/moritz/dev/repos/ooRexx/samples/` -- **381 programs** -- sum `symbols.len()` over the program's code bodies (main plus every `::ROUTINE` with a body), which is the total number of `Option<CompoundName>` slots a `Vec`-addressed table would allocate for that program.

* Largest single program: **2,244** entries (`samples/windows/ole/apps/MSAccessDemo_32bit_only.rex`, 204 symbols x 11 bodies).
* Total over all 381: **85,428** entries.

At `size_of::<Option<CompoundName>>()` = 32 bytes (two boxed slices, the `Option` in the niche), the worst program allocates about 72 KB across all its bodies.

**Chose the `Vec`** (`Box<[Option<CompoundName>]>` indexed by `SymbolId::index`).
The measured waste is negligible, an index beats a hash, and the profile already shows `hash_one::<&SymbolId>` at about 1%.
A denser addressing -- a compact `Vec<CompoundName>` behind an id-to-position map -- would reintroduce exactly the lookup this removes.

## Step 2: which callers have an id

**Every caller of `note_compound_name` has one, so no "name reached without an id" case exists there.**

* `note`'s `ExprKind::Compound(id)` arm.
* `note_variable_ref`, for both `VariableRef::Direct(id)` and `VariableRef::Indirect(id)`.

The `Indirect` case is the one worth stating: that id names the *wrapper* variable, so it reaches `note_compound_name` only when the wrapper is itself compound-shaped (`drop (a.b)`), and in that case `run.rs`'s `drop_variable` reads the wrapper through `tail_key` under that same id.
So the entry is addressed by the id the reader will present.

**But there was a caller I did not have to guess about, and running found it.**
`note_loop` does not go through `note_compound_name` at all: it calls `bind` on a `DO` control variable, an `OVER` control, a `WITH` index/item and a `COUNTER`, binding the whole dotted symbol to one slot.
A compound-shaped control (`do a.i = 1 to 5`) is legal, its tail is re-resolved on **every pass** (`run::tests::a_compound_control_variables_tail_re_resolves_every_pass` is that shape), and it therefore reached `tail_key` with no entry.

I found this by putting a temporary `debug_assert!(entry.is_some())` in `Code::compound` and running the whole workspace: **10 tests failed**, naming `A.I`, `AA.II`, `ZA.ZI`, `I.J` and `CV.J`, all from controlled-loop tests.
The fix is in `bind` rather than at the five `note_loop` call sites, so that recording a split is a property of binding a name: `bind` records the split and **assigns no slot to the stem or to a piece**, because the whole name is already bound and adding slots for its parts would move every later slot number in the body -- a frame-layout change, not a split.
With that in, the same probe over the whole workspace found **no remaining miss**.

The `debug_assert` was **not kept**. A missing entry is not a wrong answer -- `Code::compound` falls back and produces the identical key -- so the assert would turn a harmless miss on some shape the suite does not reach into a debug-build panic. `Plan::compounds`' doc states the property instead ("an entry is an optimisation and never a requirement"), which is checkable by reading `Code::compound`.

## Step 5: what happens for an `INTERPRET` fragment

**A fragment carries `Code::plan = None` and splits its own spelling, through the same `CompoundName::split`.**

The alternative -- handing the fragment the local `Plan` that `fragment_plan` already builds -- would be *correct for the split* (the fragment's ids index the fragment's own table, and the split is pure text), and it is still the wrong thing to do: that plan's `by_symbol`/`names` slots are **local to the fragment**, while `Code::slots` holds the translation into the enclosing frame. Putting the two beside each other invites a later reader to take a slot from the wrong one, and Task 2 is specifically about putting slots into these entries.

The hazard the brief names -- keying a fragment's compound by the *enclosing* plan's ids -- is real and quiet, and I made it loud rather than arguing about it. `compound-names`' ninth stanza arranges the in-range case: `za.1` is the enclosing body's first interned symbol and the fragment's own first symbol is its compound, so a build that hands the fragment the enclosing plan finds `ZA.`'s split under the fragment's compound id and writes through the wrong stem. Measured under exactly that mutation: `ZB.5 / in / ZC.1 / ZC.2 / 2` against the oracle's `in / first / 1 / 2 / first` -- every line wrong.

An `INTERPRET` inside a loop is stanza 8 and stanza 9 both.

## Step 6: oracle captures

Fresh empty directory `.../scratchpad/oracle/run`, programs at absolute paths under `.../scratchpad/oracle/progs`, `</dev/null`, stdout/stderr/status read separately. Every invocation was

```
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  /home/moritz/dev/repos/ooRexx/build/bin/rexx /abs/path/FILE )
```

All ten exited `rc 0` with **empty stderr**, and all ten produce byte-identical stdout under this crate on **both** engines (`REXX_ENGINE=tree-walker` and `REXX_ENGINE=ir`), compared with `cmp`.

| # | shape | program | oracle stdout |
|---|---|---|---|
| 1 | constant tail, set and unset | `zc.5 = 'five'; say zc.5; say zc.9` | `five` / `ZC.9` |
| 2 | variable tail, two ways to the same tail | `zi = 7; zv.zi = 'val'; say zv.zi; say zv.7` | `val` / `val` |
| 3 | several tails | `zi=1; zj=2; za.zi.zj='deep'; say za.zi.zj; say za.1.2; say za.zi.3.zj` | `deep` / `deep` / `ZA.1.3.2` |
| 4 | unset tail variable, and a bare stem | `zu.zk='set'; say zu.zk; say zu.ZK; say zu.` | `set` / `set` / `ZU.` |
| 5 | tail variable changing between references | `zi=1; zm.zi='one'; zi=2; zm.zi='two'; zi=1; say zm.zi; zi=2; say zm.zi` | `one` / `two` |
| 6 | bare stem: default, inherited tail, overridden tail, untouched stem | `zs.='def'; say zs.; say zs.1; zs.1='x'; say zs.1; say zt.` | `def` / `def` / `x` / `ZT.` |
| 7 | `DROP` of a compound, with and without a default | see the case file | `gone` / `ZD.3` / `ZD.4` / `dflt` |
| 8 | compound built inside `INTERPRET`, and `INTERPRET` in a loop | see the case file | `in` / `10` / `20` / `30` / `10 20 30` |
| 9 | a fragment's compound against the enclosing split table | see the case file | `in` / `first` / `1` / `2` / `first` |
| 10 | a `DO` control variable that is itself a compound | `do za.zi = 1 to 3 ... zi='k'; do zb.zi = 1 to 2; say zb.zi zb.k; end` | `1` / `2` / `3` / `1 ZB.K` / `2 ZB.K` |

All ten are committed as `rust/crates/rexx-exec/tests/ir_dual_cases/compound-names`, which runs each on both engines, compares them against each other, and compares the tree-walker against the recorded oracle bytes.

## Step 7: gates

Run from `rust/`, each exit status read unpiped.

| gate | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, after `rm -rf target/debug/.fingerprint/rexx-exec-*` so the crate was re-linted |
| `memcap 8G cargo test --workspace --no-fail-fast` | exit 0 |

**Test counts.** Baseline at `cb4f27d1c`, measured here rather than taken on faith: **1476 passed, 0 failed, 4 ignored**. Final: **1477 passed, 0 failed, 4 ignored**. The one new test is `plan::tests::build_records_a_compounds_split_under_the_compounds_own_id`; the ten new case-file stanzas run inside the existing `both_engines_agree_on_every_case_file`, which is one test.

## Mutations

Whole workspace, `--no-fail-fast`, under `memcap 8G`, each restored from a private backup afterwards and verified with `sha256sum -c`. The four whose result carries an "only" claim were **re-run at the final state of the commit** and gave the identical answer both times.

| id | mutation | what reddened |
|---|---|---|
| M1 | `CompoundName::split` classifies a `Tail::Constant` piece as `TailPiece::Variable` | **only** `build_records_a_compounds_split_under_the_compounds_own_id` |
| M2 | `Code::compound` always answers `None` (the table is never consulted) | **nothing** -- 1477 passed |
| M3 | `bind` stops recording a compound-shaped spelling's split | **only** `build_records_a_compounds_split_under_the_compounds_own_id` |
| M4 | `note_compound_name` computes the split and drops it | **only** `build_records_a_compounds_split_under_the_compounds_own_id` |
| M5 | `join_tails` stops emitting the `.` between pieces | `both_engines_agree_on_every_case_file`, `run::tests::drop_of_the_indirect_form`, `stem::tests::a_multi_level_tail_joins_its_pieces_with_a_period`, `the_exempt_set_matches_the_current_failures` |
| M6 | `tail_key`'s no-plan arm returns an empty key | `both_engines_agree_on_every_case_file`, `eval::tests::a_compound_read_resolves_its_tail_and_looks_it_up` |
| M7 | `Code::stem_name` returns the whole compound spelling instead of the stem | 21 tests, **and the run was killed** -- see below |
| M8 | a fragment's `Code` carries the **enclosing** body's plan | `both_engines_agree_on_every_case_file`, `tests::a_when_scan_inside_a_fragment_echoes_at_the_fragments_own_indent` |
| M9 | M8 with `printed_indent` held at its fallback, so only the compound half moves | **only** `both_engines_agree_on_every_case_file` |
| M9' | M9 with `tests/ir_dual_cases/compound-names` moved out of the directory | **nothing** -- 1477 passed |

### What these say, and what they do not

* **M1, M3 and M4 are caught by the new unit test and by nothing else in the workspace**, which is what that test exists for: a missing or misaddressed entry is not a wrong answer, because `Code::compound` falls back and produces the identical key, so no output comparison anywhere can see it. Only reading the table back can.
* **M1 is an output-equivalent mutation.** A constant piece read as a variable goes through `read_by_name`, and a name that starts with a digit can never be set, so it derives its own spelling and the key is unchanged. No behavioural test *could* catch it. The test's claim is about the table's contents, not about program output, and that is exactly the claim it makes.
* **M2 is caught by nothing, and that is correct rather than a gap.** Making `Code::compound` answer `None` disables the optimisation and changes no answer -- the fallback is required to agree with the table. The instrument that sees M2 is the instruction counter below, not the suite. **I am not claiming any test catches "the table is not being used".**
* **M9 against M9' is the one measured "adds coverage" result here.** The compound half of the cross-table hazard is caught by the new case file and, with that file removed, by nothing in the workspace. M8 alone is *also* caught by a pre-existing indent test, so the new file is not the only catcher of the wider mutation -- which is why M9 was built: it holds the indent half still.
* **M5 and M6 are caught by pre-existing tests as well as by the new file, so the new stanzas add no coverage against those two.** They are kept as measured oracle transcripts.
* **M7's run was killed after about 25 minutes**, hung in `both_engines_agree_across_every_population` (a wrong stem name evidently sends some corpus program into a non-terminating loop). Its 21 failures are a **lower bound**, not a complete list, and the run is recorded as aborted.

Two comment-only edits (in `stem.rs` and `lib.rs`) were applied while M1's final run was in flight and re-applied afterwards from a saved copy; they cannot add or remove a catcher, and the four "only" results were taken with the code otherwise byte-identical to the commit.

## Step 8: instructions

`perf stat -e instructions:u`, `REXX_ENGINE=ir`, from a fresh empty working directory, **every arm staged at the one fixed path** `.../scratchpad/bench/armbinary` so `argv[0]` is byte-identical between arms (entry 27's artifact). One run per arm, arms interleaved per axis. `rexxcps` self-calibrated to `100 x 100` on both arms, so both did the same work.

| axis | base `cb4f27d1c` | head | difference |
|---|---:|---:|---:|
| `rexxcps` | 28,592,601,793 | 26,550,118,216 | **-7.14%** |
| `compound` | 37,165,275,681 | 30,588,188,899 | **-17.70%** |
| `strings` | 44,787,586,837 | 44,928,589,444 | +0.31% |
| `varlookup` | 44,574,614,754 | 44,840,637,490 | +0.60% |
| `emptyloop` | 27,100,608,229 | 27,150,610,604 | +0.18% |
| `arith` | 20,396,605,620 | 20,403,950,881 | +0.04% |

`rexxcps` loses **2.042 billion instructions**; `compound` loses **6.577 billion**.
Those are the two axes holding compound variables, and they are the only two that move by more than 1%.

**The `rexxcps` row is a re-measurement, and the figure it replaces was wrong by its own arm's noise.**
This report first gave -7.19%, from a single pair whose base run came out higher than all three base runs taken minutes earlier -- the review caught it.
Re-measured, three interleaved rounds at the one fixed path: base 28,598,071,071 / 28,592,601,793 / 28,588,567,089 against head 26,550,119,275 / 26,550,118,216 / 26,546,064,392, medians in the table above.
The reviewer's own independent pair reads **-7.12%**, and Task 2's arms imply -7.16%.

**Arm-internal spread**, taken over every run of each arm across both sittings.
Base: 28,588,534,968 / 28,588,542,497 / 28,588,567,089 / 28,591,624,395 / 28,592,601,793 / 28,598,071,071, a range of **0.033%**.
Head: 26,546,057,740 / 26,546,064,317 / 26,546,064,392 / 26,550,118,216 / 26,550,119,275 / 26,552,499,092, a range of **0.024%**.
Both are two orders of magnitude below the effect, and the spread is why the three independent readings of it differ in the second decimal.
The 0.011% this report first quoted for the base arm was three runs rather than six.

**The cost side, part one.** The four axes this task cannot touch rose 0.04% to 0.60% -- shared-driver codegen drift, the same size entry 27 measured (0.24% to 0.54%) for a change of this shape.

**The cost side, part two: `INTERPRET` got slower, and this report left it unmeasured.**
`fragment_plan` calls `Plan::build`, so from this commit onward it also builds and discards a `compounds` table on every execution of every `INTERPRET`; and the fragment, which carries no plan on purpose, now allocates an owned `CompoundName` at each compound reference where the old code borrowed slices out of the interned name.
**Measured by the reviewer and not by me**, `perf stat -e instructions:u`, three interleaved runs per arm at one fixed binary path, on `do i = 1 to 200000; interpret "x = i + 1"; end`: base 48.48 / 48.64 / 48.66 G against head 49.02 / 49.13 / 49.16 G, the arms non-overlapping, **+1.0%**.
The same loop with the `INTERPRET` taken out moved **+0.45%**, which is this commit's codegen drift on a program it cannot otherwise touch, so roughly half a percent is attributable to the fragment path.
Nothing corpus-shaped sees it: no axis in the table above holds an `INTERPRET`.
Recorded rather than fixed, and carried into the record's entry 28 as the plan's cost side.

**No wall-clock figure is claimed**, for the reason the brief gives: entry 27 measured layout alone moving these axes between -5.12% and +4.55%.

**Byte-identity between the arms**, one run each, `REXX_ENGINE=ir`: `compound`, `varlookup`, `emptyloop` and `arith` produce byte-identical stdout **and** stderr and exit 0 under both binaries. `rexxcps` differs only in its own two timing lines (elapsed seconds and clauses per second), with `100 x 100` on both, and empty stderr on both.

## Things I am not sure about, or left undone

* **`compounds` is sized by `symbols.len()` per body, so a multi-body program allocates the table once per body.** Measured worst case over 381 real programs is 2,244 entries (about 72 KB), which is why I did not build a denser addressing. A program far larger than anything in the corpus would pay more, and nothing in the code bounds it.
* **`fragment_plan` now builds a compound table for the fragment and discards it**, because it calls `Plan::build`. **I did not measure this and should have; the reviewer did** -- +1.0% on an `INTERPRET`-saturated loop against +0.45% drift on the same loop without one, quoted above and in entry 28. Left unfixed. The cheap form, if it is ever worth taking, is a build mode that skips `compounds` for a fragment plan, since `fragment_plan` keeps only the id-to-slot translation out of the plan it builds.
* **`bind` records a split for any compound-shaped spelling it binds, including a bare stem's own name** (`A.` contains a period, and `compound_parts` gives it stem `A.` with one empty constant tail). Those entries are never read: `stem_name`/`tail_key` are only entered for compound-shaped symbols. The cost is one small allocation per stem symbol at plan-build time.
* **Slot allocation is unchanged**, deliberately. `note_compound_name` assigns exactly the slots it assigned before, and `bind`'s new branch assigns none. I did **not** route a compound-shaped `DO` control through `note_compound_name`, because that would add `A.` and `I` to `names` and move every later slot number in the body -- a frame-layout change that belongs to nobody's task right now.
* **Task 2's work was not done and was cleanly separable.** Slot resolution is still by name: `join_tails` calls `read_by_name` for every variable piece, exactly as before. `TailPiece::Variable` is where Task 2's slot goes.
* **`Code::plan` pairing `symbols` with the plan built from the same body is an invariant held by construction, not by a check.** `Plan::compound`'s doc says so and names the quiet failure mode; M9 is the measured witness that a violation is loud on output, but only for a program arranged to make it so.
* **M7's mutation run did not finish**, so its list of catchers is a lower bound.

## Fix round, against `task-1-review.md`

The review's verdicts were spec **PASS** and code quality **PASS WITH CHANGES**, with no code change required.
Nothing in this round touches behaviour: three comment claims were wrong or unmeasured, and the measurement figures were corrected.
Done on top of `df3c34087`, with Task 2 already landed; nothing was rebased.

### 1. "A missing **or misaddressed** entry is not a wrong answer" -- the second half was false

The fallback argument covers a *missing* entry only. A misaddressed one has no fallback at all: the table answers, with another symbol's split.
The doc comment on `build_records_a_compounds_split_under_the_compounds_own_id` now separates the two, and the second half is measured rather than reasoned.

**N1**, both fillers writing every entry under `(id.index() + 1) % len`, whole workspace, `--no-fail-fast`, at the final state of this commit: **15 failures over 14 distinct tests**.
`plan::tests::build_records_a_compounds_split_under_the_compounds_own_id`, `plan::tests::a_tail_piece_with_no_plan_slot_binds_in_extra`, `plan::tests::a_control_variable_does_not_take_the_slots_off_a_compound_already_seen`, `run::tests::assignment_to_a_variable_a_stem_and_a_compound`, four `run::tests::drop_*`, three `run::tests::use_arg_alias_*`, `run::tests::an_exposed_stem_aliases_the_callers_entry_not_the_object`, `both_engines_agree_on_every_case_file`, and `the_exempt_set_matches_the_current_failures` in two binaries.
So a misaddressed entry is loud at output level, and the comment now says so.

### 2. "the **three** syntactic positions" -- a cardinality, and the wrong one

`PARSE VAR a.i` is a further position: `note_parse`'s `ParseSource::Var` arm binds the whole dotted symbol and `parse_template`'s `read_parse_var` reads it back through `tail_key`.
Reproduced here rather than taken from the review -- fresh empty directory, absolute path, the standard wrapper: `i = 3; a.i = 'p q'; parse var a.i x y; say x y` prints `p q` on the oracle at rc 0 and on both engines.
`note_variable_ref` likewise carries `EXPOSE`, `PROCEDURE EXPOSE` and `USE LOCAL` targets, not only `DROP`.

The count is gone. The comment now names what the rows are and which filler each reaches, and adds the `PARSE VAR` fact as the reason the control-variable fix went into `bind` -- **without** claiming to enumerate the positions, which is the claim that was wrong in the first place.

### 3. The case file's header claimed unique coverage for rows where it was not measured

"The rows a precomputed split can get wrong in a way nothing else notices" headed three bullets; only the fragment bullet had been measured that way.
The header now labels every row a transcript except the fragment row, and says for the other two why they are not witnesses: `run.rs`'s `a_compound_control_variables_tail_re_resolves_every_pass` predates this file and already covered the `DO`-control shape, and the changing-tail bullet describes an implementation this crate does not have (a table caching the resolved key), so there is nothing here to mutate into it.

The one claim kept was re-measured at the final state of this commit:

* **N2** -- a fragment's `Code` carrying the enclosing body's plan, with `printed_indent` pinned to its fallback so only the compound half moves: **exactly one** red test in the workspace, `both_engines_agree_on_every_case_file`, failing at `tests/ir_dual_cases/compound-names:169` with `ZB.5 / in / ZC.1 / ZC.2 / 2` at rc 0.
* **N2'** -- the same mutation with `compound-names` moved out of the directory: **1479 passed, 0 failed**.

### 4. The `rexxcps` figure was its own arm's high outlier

Corrected from -7.19% to **-7.14%**, with the six-run spread that explains it, in the Step 8 section above.
The record's entry 28 carried the -7.19% and is corrected to match, with the re-measurement and its runs written into the entry.

### 5. The `INTERPRET` cost, recorded rather than fixed

Added to the Step 8 section above and to entry 28 as the plan's cost side, quoted with the reviewer's provenance since they took it and I did not.

### Not changed, and why

* **`bind` recording splits for stem-shaped names nothing reads.** The review accepted it: the uniformity is what makes "recording a split is a property of binding a name" hold, and it is one small allocation per stem symbol at plan-build time.
* **No code changed at all this round.** The review asked for none, and every fix is a comment or a figure.

### Gates, at the final state of this commit

`cargo fmt --all --check` exit 0; `cargo clippy --workspace --all-targets -- -D warnings` exit 0 from a cleared `rexx-exec` fingerprint; `memcap 8G cargo test --workspace --no-fail-fast` **1479 passed, 0 failed, 4 ignored**, unchanged from the baseline this round started at.
