# Task 1 report -- the stem's slot, resolved once

Plan: `docs/superpowers/plans/2026-08-13-stem-slot-resolution.md`.
Base: `9513c8b13`.
Commits: `5eaa3c8e8` (the change), `73d470474` (record entry 30), `4bfabd00f` (false comments corrected, no behaviour change), `0e3659c8c` (the fix round: the rest of that closed set, plus record entry 31).

The record entry is a second commit for the same reason Task 2 of the previous plan made it one: its `commit` field has to name a commit that already exists, and the record's rule is that the hash is read back from `git log`.

## What changed

A compound's stem carries the slot the upfront pass assigned its name, so finding the stem object at a reference costs an index into the frame rather than a hash of the stem's bytes.

* **`rust/crates/rexx-exec/src/plan.rs`**
  * `CompoundName` gains `stem_at: Option<usize>`.
  * `CompoundName::split` produces `stem_at: None`, for the reason it produces `at: None` on every piece.
  * `note_compound_name` keeps the answer `slot_for(&entry.stem)` was already giving it.
  * `build_records_a_compounds_split_under_the_compounds_own_id` gains the stem slots.
  * One new test, `a_stem_with_no_plan_slot_binds_in_extra`.
* **`rust/crates/rexx-exec/src/stem.rs`**
  * New `Interp::stem_slot(stem_name, at)`: the one place the three `_at` accessors decide a stem's slot, carrying the debug tripwire.
  * `stem_get_at`, `stem_set_at`, `stem_drop_tail_at`; the three old names are now one-line wrappers passing `None`.
* **`rust/crates/rexx-exec/src/lib.rs`** -- `Code::stem_name` becomes `Code::stem`, returning `(&'a [u8], Option<usize>)`.
* **`rust/crates/rexx-exec/src/eval.rs`** -- the compound read path passes the slot; the `>C>` echo takes the name and explicitly discards the slot.
* **`rust/crates/rexx-exec/src/run.rs`** -- the compound write (`assign_expr_target`), the controlled loop's own re-test, and `DROP` of a compound tail.
* **`rust/crates/rexx-exec/src/parse_template.rs`** -- a compound `PARSE VAR`/`PARSE VALUE` target's read.
* **`rust/crates/rexx-exec/tests/ir_dual_cases/compound-names`** -- three new stanzas, all measured against the oracle.
* **`docs/superpowers/plans/2026-08-13-stem-slot-resolution.md`** -- steps ticked, and one correction to the plan's own file list (below).
* **`docs/superpowers/plans/phase-4f-record.md`** -- entry 30, appended.

### A correction to the plan, written into the plan

The plan's Files list said `eval.rs` carries "the compound read **and write** paths".
**The write path is not in `eval.rs`**: it is `run.rs`'s `assign_expr_target`, and it is what `tab.i = i` goes through on both engines.
Reading `stem.rs`'s callers -- which Step 2 asks for -- also turned up two more callers holding an entry (`run.rs`'s controlled-loop re-test and `drop_variable`) and one in `parse_template.rs`.
The plan's list now names `lib.rs`, `run.rs` and `parse_template.rs` alongside the three it had, with a parenthetical saying what was wrong and how it was found.

## Step 2: every `slot_of(stem_name)` in `stem.rs`, and which ones changed

Found by reading the file, not from a list. **There were five**, and I changed three.

| site | reached by | disposition |
|---|---|---|
| `stem_get` | a compound *read* | **changed** -- `stem_get_at` |
| `stem_set` | a compound *write* | **changed** -- `stem_set_at` |
| `stem_drop_tail` | `DROP a.i` | **changed** -- `stem_drop_tail_at` |
| `stem_assign` | `a. = expr`, a **bare stem** write | left, and it is a finding -- below |
| `replace_stem` | `stem_assign`'s wrap branch and `stem_drop` | left, for the same reason |

`read_stem_at`'s own `None => self.slot_of(name)` is a sixth occurrence of the *pattern* and not a sixth site: it is already the `_at` form this task copies.

**The two I left are left by choice, and the choice is a finding rather than an omission.**
They are a **bare stem's** own operations. Their name is the whole spelling of an `ExprKind::Stem` symbol or a run-time string, not a compound's stem half, so `CompoundName::stem_at` has nothing to say about them -- there is no compound entry involved at all.
`stem_slot`'s doc comment says exactly this, so the boundary is written where a reader of the file meets it.

**But the slot they resolve is already computed, and nothing uses it.** That is the finding, and it is measured rather than argued -- see the last section.

## Step 1: the slot beside the stem, and what a stem without one does

**An `Option`, because `note_compound_name` and `bind` disagree about slots, and because `CompoundName::split` is entered with no plan at all.**
`note_compound_name` assigns the stem a slot and now keeps it.
`Plan::bind` assigns none, deliberately: `do a.i = 1 to 5` binds the whole dotted `A.I` to one slot, and giving `A.` and `I` slots of their own would move every later slot number in the body.
A fragment carries no plan, so `Code::stem` answers `None` for every compound in an `INTERPRET`.

**The difference is in the type at both ends, and the compiler enforced it.**
`stem_at` is a named field on a struct the tests build by literal, so adding it produced three `E0063: missing field stem_at` errors and nothing else -- each one a place that had to decide what this entry's stem slot is.
`Code::stem` returns the name and the slot **together** rather than from two methods, so a caller reaching for the name has to say what it does about the slot; the one site that wants only the name (`echo_symbol_read`'s `>C>` line) binds `_` and says why in a comment.

**Frame layout is unchanged.** `slot_for` is called exactly where and as often as it was; only its return value stops being discarded. `build_records_a_compounds_split_under_the_compounds_own_id` asserts the plan's whole name-to-slot map by hand, and that assertion did not move.

## Step 4: can a stem be bound in `extra`? **Yes**, and here is the program

Run, not reasoned. `Interp::slot_of` instrumented to print each of its three sources, release build, run from a fresh empty directory under `REXX_ENGINE=ir`.

**The table below was taken on two arms, and the arm matters for one row of it.**
The first version of this report printed the whole table without naming an arm, which the review caught; the three `extra` rows were taken on both and are identical on both, and the control row **cannot** come from head.

**Three different shapes reach `extra`, and none of them carries a slot** -- identical output on the base arm `9513c8b13` and on head:

| program | probe output, both arms |
|---|---|
| `do za.zi = 1 to 3 / nop / end` | `grow-into-extra "ZI" slot 1`, `grow-into-extra "ZA." slot 2`, then `hit-extra` for both, six times each |
| `interpret "zq.1 = 7"` | `grow-into-extra "ZQ." slot 0`, then `hit-extra "ZQ." slot 0` |
| `zn = 'ZR.1'` then `drop (zn)` | `grow-into-extra "ZR." slot 1` |

and the control, an ordinary compound whose stem the plan does hold, **where the two arms differ and that is the point**:

| program | base `9513c8b13` | head |
|---|---|---|
| `zi = 1 / zb.zi = 5 / say zb.zi` | `plan "ZB." slot 1`, twice -- the plan's own map, never `extra` | **nothing at all**: `slot_of` is never entered |

The base row is what answers the question the step asks, which is about a resolution mechanism that predates the change.
**The head row is the stronger evidence and I ran it only after the review asked for it**: silence there is direct evidence the precomputed slot is being used, and it is the one thing the suite cannot see -- M2 below shows no test in the workspace catches "the slot is not being used".

The first row is the compound `DO` control variable: `bind` binds `ZA.ZI` whole and binds nothing named `ZA.`, so the loop's own read misses the plan, misses `extra`, grows the frame, and finds `ZA.` in `extra` on every later pass.
That is exactly the entry class `bind` fills with no slot, so `stem_at`'s `None` arm is **required** rather than defensive.
The second and third rows are the fragment and the run-time name, which carry no entry at all.

`plan::tests::a_stem_with_no_plan_slot_binds_in_extra` is the permanent form of the first row: it asserts the plan holds no name `ZA.`, asserts the entry's `stem_at` is `None`, asserts `Code::stem` hands back that `None`, runs `stem_get_at`, and asserts `ZA.` ends up in the activation's `extra`.

**A stem that does carry a slot cannot shadow an `extra` binding**, and the reason is the resolution order.
`Interp::slot_of` reads the plan's own name map first and `extra` only after it misses.
A slot lands on `stem_at` only because `Plan::slot_for` put that stem's name into that same map.
So for a stem carrying a slot, `extra` was already unreachable before this task existed: `slot_of` would have returned the plan's answer, which is the number now stored on the entry.
That argument is checked by Step 5 rather than left as an argument.

## Step 5: the accessor-level probe, and its negative control

Inside `Interp::stem_slot`'s `Some(slot)` arm, an `assert_eq!` of the precomputed slot against the **full three-source resolution** -- `self.slot_of(stem_name)`, which is plan then `extra` then growth, not just the plan's map the committed tripwire compares against.

| run | result |
|---|---|
| `memcap 8G cargo test --workspace --no-fail-fast` | **1481 passed, 0 failed, 4 ignored, zero firings** |
| the same probe **inverted** to `assert_ne!` | **1446 passed, 35 failed**, and it fires **35 times** |

So the zero is a live zero. The passing run covers the corpus differential, the `ootest` population sweep, the dual-engine case files, the collector tests and every unit test.

**Extended to the programs the suite does not run**, because `rexxcps` and the bench programs are the measurement subjects and are in no test:

* `assert_eq!` form, release build, every `bench-programs/*.rex` plus `samples/rexxcps.rex` on **both** engines -- 22 runs, **zero firings**.
* Negative control on the same programs: the site **fires** on `compound`, `alloc4c` and `rexxcps` under both engines and is **silent** on `arith`, `emptyloop`, `strings` and `varlookup`.

That second line is worth keeping: it is an independent, mechanical confirmation of the previous plan's correction that `compound` and `alloc4c` are subject axes and the other four are the control. The probe cannot fire on a program that never reaches a compound's stem.

## Step 6: oracle captures

Fresh empty directory, absolute paths, `</dev/null`, stdout/stderr/status read as three descriptors, every invocation

```
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  /home/moritz/dev/repos/ooRexx/build/bin/rexx /abs/path/FILE </dev/null )
```

**Every stanza of `compound-names` was re-captured at this commit**, not only the new ones: each program was extracted from the committed file, run, and its stdout compared with `cmp` against the recorded bytes. All matched byte for byte at rc 0 with empty stderr, so the earlier transcripts still hold and the new ones join them on the same footing.

### The three new stanzas

All three are **labelled transcripts** and claim nothing about what they catch; the mutation table below is where the coverage claims are.

**A stem's own operations, with compound reads through it on either side.**

```rexx
zs. = 'dflt'
zi = 1
say zs.zi
zs.zi = 'one'
say zs.zi
say zs.
drop zs.
say zs.zi
say zs.
zs.2 = 'two'
say zs.2 zs.9
```

Oracle: `dflt` / `one` / `dflt` / `ZS.1` / `ZS.` / `two ZS.9`.

`say zs.` after `zs. = 'dflt'` prints **`dflt`, not `ZS.`** -- a stem holding a default renders as that default. That was measured rather than predicted, and it is the line I would have got wrong.

**`PROCEDURE EXPOSE` of a stem, written through on both sides of the call.**

```rexx
zi = 3
zp.zi = 'outer'
call zsub
say zp.3 zp.4
exit
zsub:
procedure expose zp.
zj = 4
zp.zj = 'inner'
say zp.3
return
```

Oracle: `outer` / `outer inner`.

The exposed name is the stem alone, so the callee's own plan holds `ZP.` and gets its own `stem_at` for it, pointing at the slot `exec_procedure` aliased to the caller's. The tail is resolved separately in each body against that body's own variable.

**A compound inside an `INTERPRET`, on both sides of the stem's slot question.**

```rexx
interpret "zq.1 = 'frag'"
say zq.1
say zq.
interpret "zr.2 = 'only'"
interpret "say zr.2"
```

Oracle: `frag` / `ZQ.` / `only`.

`ZQ.` is a stem the enclosing body names for itself, so the enclosing plan holds it and the fragment's write resolves to the same slot the body's own `stem_at` names. `ZR.` is named by no line of the enclosing body, so the plan never holds it and both fragments reach it through `extra` -- the second row of the Step 4 table, in a form the harness runs on every build.

**The plan's list of eight behaviours**: a stem read, a stem write, a bare stem, `DROP` of a stem and a compound `INTERPRET` are the three stanzas above; `PROCEDURE EXPOSE` of a stem is the second; a stem whose tail changes between references and a compound `DO` control variable were already stanzas of this file before this task, and I did not duplicate them. All are in the file, all measured against the oracle.

## Step 7: gates

Run from `rust/`, each exit status read unpiped, at the final state of `4bfabd00f`.

| gate | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, after `rm -rf target/debug/.fingerprint` so every crate including `rexx-exec` was genuinely re-linted (`Checking rexx-exec` confirmed in the log) |
| `memcap 8G cargo test --workspace --no-fail-fast` | exit 0, **1481 passed, 0 failed, 4 ignored** |

The brief's baseline is 1480. The one new test is `plan::tests::a_stem_with_no_plan_slot_binds_in_extra`; the three new case-file stanzas run inside the existing `both_engines_agree_on_every_case_file`, which is one test.

## Mutations

Whole workspace, `--no-fail-fast`, under `memcap 8G`, each applied by script and restored from a private backup, `touch`ed and rebuilt afterwards, with the restore verified by `cmp` against the backup on every run.
A per-run `timeout 3000` was set for the reason the plan gives; no run came near it.
**Every row was run at the final code state of `5eaa3c8e8`.** The two later commits change one record document and three comments and cannot add a catcher.

| id | mutation | what reddened |
|---|---|---|
| M1 | `note_compound_name` computes the stem's slot and stores `None` -- the answer dropped again, as before this task | **only** `plan::tests::build_records_a_compounds_split_under_the_compounds_own_id` (1480 passed, 1 failed) |
| M2 | `stem_slot` ignores `at` and always resolves the name, so a stored slot is never used | **nothing** -- 1481 passed |
| M3 | `note_compound_name` stores `slot_for(&entry.stem) + 1`: a slot the plan does not give the stem's name | 36 failures across 35 tests; the tripwire panicked **35** times |
| M3' | M3, with the tripwire deleted | 29 failures across **28** tests, 0 panics |
| M4 | `bind` gives the stem the whole dotted name's slot, so an entry that must carry none carries one | 13 failures, including `a_stem_with_no_plan_slot_binds_in_extra`; the tripwire panicked 11 times |

### What these say, and what they do not

* **M1 is caught by the entry test and by nothing else in the workspace.** A dropped stem slot is not a wrong answer -- the accessor falls back to `slot_of`, which is what every stem accessor did before this task -- so no output comparison anywhere can see it. Only reading the entry back can. That single-test result is the measured "only" claim, and it is what makes the change to that test load-bearing rather than decorative.
* **M2 is caught by nothing, and that is correct rather than a gap.** Making `stem_slot` ignore every stored slot disables the optimisation and changes no answer, because the fallback is required to agree with the slot. The instrument that sees M2 is the instruction counter, not the suite. **No test here catches "the slot is not being used".**
* **M3 shows the tripwire can fail**, which is the thing a `debug_assert` has to be shown to do before its comment claims anything.
* **M3' is the interesting one, and it comes out opposite to the previous task's.** Task 2's tail-piece tripwire was measured to add **no** coverage: the same test names failed with and without it. This one **does**. M3 reddens 35 distinct tests and M3' reddens 28, so seven tests catch a wrong stem slot **only** because the tripwire is there: `builtin::datatype::tests::symbol_reads_the_variable_pool_for_names_and_compounds`, `builtin::datatype::tests::var_answers_the_documented_example_line_for_line`, `run::tests::do_over_a_parenthesised_stem_target_is_also_caught`, `run::tests::do_over_a_stem_target_takes_the_loud_path`, `run::tests::drop_of_the_indirect_form_is_a_subsidiary_list_of_words`, `run::tests::novalue_fires_for_a_simple_variable_and_a_compound_but_not_a_bare_stem`, `run::tests::use_arg_alias_reports_the_kind_mismatch_before_the_uninitialised_target`. In those seven a wrong stem slot lands somewhere whose contents happen not to change the printed bytes. **I did not import the previous task's rationale for keeping a tripwire; I ran the pair and got a different answer.**
* **M4 reddens the new test along with twelve others**, so `a_stem_with_no_plan_slot_binds_in_extra` is **not** a unique catcher and I do not claim it adds coverage. It earns its place as the permanent, re-run form of the Step 4 measurement -- the `extra` assertion is the part nothing else in the workspace states -- and it can fail, which M4 shows.

## Step 8: instructions

`perf stat -e instructions:u`, `REXX_ENGINE=ir`, from a fresh empty working directory, **every arm staged at the one fixed path** `.../task1-stem/bench/armbinary` so `argv[0]` is byte-identical between arms.
Arms interleaved per axis: three rounds on the subject axes, six on the control axes, ten on `strings`.
Every `rexxcps` run self-calibrated to `100 x 100`, checked on each arm's own `Averaged:` line.

| axis | base `9513c8b13` | head | difference |
|---|---:|---:|---:|
| `compound` | 27,798,509,899 | 25,052,491,453 | **-9.88%** |
| `alloc4c` | 8,456,675,211 | 8,189,737,101 | **-3.16%** |
| `rexxcps` | 25,858,071,136 | 25,297,898,179 | **-2.17%** |
| `arith` | 20,403,949,862 | 20,415,989,999 | **+0.06%** |
| `strings` | 44,928,588,505 | 44,928,588,385 | -120 instructions |
| `varlookup` | 44,840,637,218 | 44,840,636,876 | -342 instructions |
| `emptyloop` | 27,150,610,000 | 27,150,610,122 | +122 instructions |

Minimum of each arm's runs. `compound` loses **2.746 billion** instructions, `rexxcps` **0.560 billion**, `alloc4c` **0.267 billion**.

**The instrument's own spread, measured on every axis before any small figure was read**, as the span one binary produces against itself in the same sitting:

| axis | span, base | span, head |
|---|---:|---:|
| `compound` | 1,370,916 | 1,571,750 |
| `alloc4c` | 58,490 | 14,892 |
| `rexxcps` | 4,075,851 | 11,206,860 |
| `arith` | 1,439 | 1,250 |
| `strings` | 1,999 | 1,036 |
| `varlookup` | 607 | 954 |
| `emptyloop` | 1,430 | 1,456 |

So `strings`, `varlookup` and `emptyloop` carry a **bound and not a difference**: under about a thousand instructions out of twenty-seven to forty-five billion, which is this instrument's resolution there. No signed figure survives on those three.

**`arith` is not inside the spread, and it is the cost side.**
It moved **+12,040,137 instructions**, about eight thousand times its own span, with the six runs of each arm not overlapping at all.
It holds no compound variable -- measured, not read: the Step 5 probe is silent on it under both engines.
So it is codegen drift on a program this change cannot otherwise touch. **I offer no attribution beyond that**: entry 27 measured layout alone moving these axes between -5.12% and +4.55% on wall clock, and separating drift from layout needs a do-nothing control I did not build.

**Three `strings` runs came back 36 to 51 million instructions high** -- two on the base arm and one on the head arm. That is the same signature the previous task's reviewer recorded on that axis and attributed to external interference; seeing it on **both** arms here is what rules out its being a property of either binary. Those three are excluded; every other run of every axis is used.

**Byte-identity between the arms.** `compound`, `alloc4c`, `varlookup`, `emptyloop`, `arith` and `strings` produce byte-identical stdout **and** stderr and exit 0 under both binaries on **both** engines. `rexxcps` differs only in its own two timing lines, with `100 x 100` and empty stderr on every arm.

**The measured binary is the committed one.** Rebuilt from the sources at `4bfabd00f` and compared with `objcopy` plus `sha256sum`: `.text` and `.rodata` are byte-identical to the binary every figure above was taken with.

### The bucket this task existed for

`perf record -F 999`, `REXX_ENGINE=ir`, over `samples/rexxcps.rex`, both arms staged at the one fixed path, **three runs per arm**, one sitting, bucketed by self time.
The bucket is named rather than counted: `hash_one::<&[u8]>`, SipHash's own `write`, `Interp::slot_of`, `hash_one::<&SymbolId>` and `HashMap<&str, ()>::contains_key::<str>`.

| | base `9513c8b13` | head |
|---|---|---|
| **bucket** | **7.56% / 7.42% / 7.75%** | **5.34% / 4.68% / 5.24%** |
| `hash_one::<&[u8]>` | 2.56% / 2.66% / 3.04% | 2.07% / 1.61% / 2.06% |
| SipHash `write` | 2.79% / 2.49% / 2.48% | 1.63% / 1.25% / 1.43% |
| `Interp::slot_of` | 1.47% / 1.55% / 1.25% | 0.73% / 1.04% / 0.92% |
| `hash_one::<&SymbolId>` | 0.56% / 0.72% / 0.76% | 0.80% / 0.74% / 0.68% |

**The bucket fell**, which is the whole reason the task exists. The two arms' ranges do not overlap on the bucket or on its three largest members. `hash_one::<&SymbolId>` -- the id-keyed `by_symbol`/`Code::slots` lookup, which this task does not touch -- overlaps, as it should.

**These shares are not comparable with the plan's 8.61%/9.33% table.** That table was taken at a different sitting with a different `perf` invocation, and a sampled share is a share of whatever else was running. The comparison that means anything is base against head above, measured in one sitting with one method. A first pass of this profile taken with `--call-graph=dwarf` put the same bucket at 7.42% and 5.74%: different absolute numbers, same direction and roughly the same gap, which is the reason to report the pair rather than either number alone.
A share is also a share of a total that itself fell 2.17%, so the absolute hashing work fell by rather more than the share difference suggests.

## Things I am not sure about, or left undone

* **A bare stem write already has its slot resolved, and nothing uses it. Six comments said the slot does not exist, and they were wrong.**
  This is the finding behind the third and fifth commits; the closed set is in the fix round below. `PlanSlot`'s doc said a stem *write* "is a name and not a slot at all"; `write_slot` said neither a stem nor a compound target "writes the symbol's own frame slot"; `assign_expr_target` said "only the first one writes a slot by name at all".
  The compound half of all three is right. The stem half is not: `Plan::note`'s `ExprKind::Variable(id) | ExprKind::Stem(id)` arm calls `bind(id, symbols.name(id))`, and `assign_expr_target`'s `Stem` arm calls `stem_assign(symbols.name(id))` -- the **same spelling on the same id**, so `by_symbol[id]` is the number `stem_assign` then hashes for.
  **Measured rather than reasoned**, which matters because the reasoning depends on the assignment target actually being walked by `note`: `assign_expr_target` instrumented to print both numbers, on `zs. = 'one'; zi = 2; zs.zi = 'two'; zt. = 'x'`, prints `by_symbol=Some(0) slot_of=0` and `by_symbol=Some(2) slot_of=2` under both engines.
  The comments are corrected in `4bfabd00f` and `0e3659c8c`. **The optimisation is not taken**: it is a different path from a compound's stem, it wants its own measurement, and folding it into a change whose whole claim is that behaviour did not move would be wrong. `rexxcps`' inner loop writes compound tails, not bare stems, so it would not have shown up in this task's numbers either way.
* **A compound `DO` control variable's stem and tail pieces still resolve by name on every pass**, because `bind` assigns them no slots and this task did not change that. The reason is unchanged from the previous plan's report: adding them moves the body's frame layout. `a_stem_with_no_plan_slot_binds_in_extra` is the shape that pays it.
* **The tripwire costs a hash per compound reference in debug builds**, which is what the release path used to cost. Test-suite wall clock was not measured before and after adding it. Unlike the previous task's, this one is a measured net catcher (M3 against M3'), so it is kept for both reasons rather than only for where it fails.
* **`Code::plan` pairing the plan with the activation is still an invariant held by construction**, not by a type. The tripwire makes a violation loud in debug rather than preventing it.
* **`arith`'s +0.06% is real and unattributed.** It is above that axis's own spread by a factor of thousands, it holds no compound, and the do-nothing control that would separate codegen drift from binary layout was not built. If a later task wants to know whether this change costs anything on compound-free code, that control is the thing to build.
* **I used `git checkout-index -f --` once**, to drop a temporary probe from `run.rs` after the file had already been committed. `git status` was empty immediately afterwards, so nothing uncommitted was at risk, but the plan's restore rule is a backup copy and that is what every other restore in this task used. Recording it rather than leaving it in the transcript.
* **The `strings` axis has a recurring 36-to-51-million-instruction excursion** that has now been seen by three different sittings on this axis and nothing else. It landed on both arms here, so it is not a property of either binary, but nothing explains it and it will keep costing runs until something does.

---

# Fix round -- `0e3659c8c`

`.superpowers/sdd/2026-08-13-stem-slot-resolution/task-1-review.md` returned **spec PASS, code quality PASS WITH CHANGES**, on top of `4bfabd00f`. Every change wanted was in comments; no code change was required and none was made. Gates re-run at the final state: `fmt` exit 0, `clippy -D warnings` exit 0, `memcap 8G cargo test --workspace --no-fail-fast` exit 0, **1481 passed, 0 failed, 4 ignored**.

## Finding 1: the closed set for "a bare stem write has no slot"

`4bfabd00f` corrected the instances the investigation happened to have in hand and swept nothing. Swept now, by searching the crate for the claim's vocabulary rather than for the sentences the review named -- `no slot`, `not a slot`, `through a name`, `by name rather`, `resolved ahead`, `under a name` -- and reading every hit.

**Instances of the claim. All are now true.**

| site | wording | fixed in |
|---|---|---|
| `ir/mod.rs`, `PlanSlot`'s doc | "a stem *write* is `stem_assign`, which is a name and not a slot at all" | `4bfabd00f` |
| `ir/compile.rs`, `write_slot`'s doc | "Neither writes the symbol's own frame slot, so there is no slot here for them to carry" | `4bfabd00f` |
| `run.rs`, `assign_expr_target`'s doc | "only the first one writes a slot by name at all" | `4bfabd00f` |
| `run.rs`, `control_slot`'s doc | "so **neither** has a slot that could be resolved ahead of the pass that uses it" -- **measurably false** | `0e3659c8c` |
| `run.rs`, `bind_control`'s doc | "a stem or compound control writes through a name rather than through a slot" | `0e3659c8c` |
| `run.rs`, `bind_control`'s `NameShape::Stem` arm | "a stem write is `stem_assign` under a name, so there is no slot for it to be the slot of" | `0e3659c8c` |

**Hits that are not instances of the claim**, read and left alone because each is about a compound or a run-time name and each is true: `eval.rs`'s `SymbolRead::Compound` doc ("a compound is the kind a compiled read has no slot for"); `ir/golden_tests.rs`'s "`ZA.ZI`'s own symbol has no slot to carry"; `run.rs`'s two `exec_procedure`/`PROCEDURE EXPOSE` notes about a name appearing in no instruction of either routine; `ir/mod.rs`'s `resolved`, which is a call-site cache and unrelated; and `plan.rs`/`lib.rs`'s statements that `Plan::bind` assigns the **stem prefix** no slot, which is about `CompoundName::stem_at` rather than about a symbol's own slot.

`control_slot`'s was the false one and it is measured, by me and not taken from the review: `assign_expr_target`'s `Stem` arm instrumented to print `code.slots.get(id)` beside `self.slot_of(name)`, release, on `do cv. = 1 to 3`, prints `BARESTEM "CV." by_symbol=Some(0) slot_of=0` four times under `ir` and four times under `tree-walker`. Its doc now says the slot exists and is not carried because nothing reads it yet, which is the same disposition `write_slot`'s correction records.

`bind_control`'s `Stem` arm was the corrected `assign_expr_target` sentence verbatim in meaning, and its "`None`, and not `at`" framed a distinction that does not exist: `at` **is** `control_slot`'s answer, which is `None` for a stem, so the two spellings are the same value. The comment now says that.

## Finding 2: this task's own sentence

`stem_slot`'s doc said "no caller **has** a slot to hand them" of `stem_assign`/`replace_stem`. The true statement is narrower -- no caller **passes** one -- and the sentence now says so and points at `control_slot` for the measurement. That instance was authored by `5eaa3c8e8`, three commits before the same author corrected the same claim elsewhere.

## Finding 3: entry 30's `arith` sentence

Corrected by **appending entry 31**, never by editing entry 30, which is left exactly as `73d470474` committed it (the append is 40 insertions, 0 deletions, one hunk at the end of the file). Entry 31 withdraws "so this is codegen drift" and keeps the bound: the evidence reaches "not the change's semantics" and stops, because the control separating drift from layout was not built. It also states that `5eaa3c8e8`'s commit message carries the same claim and cannot be edited, and that entry 31 is the correction of record for it.

## Finding 4: the Step 4 table

Labelled, and the missing arm added -- see that section above. I ran the head arm rather than quoting the review's: the control program produces **nothing at all** from `slot_of` at head, which is direct evidence the precomputed slot is used and is the one property M2 shows no test can see.

## Finding 5: the cardinality

`write_slot`'s rewritten sentence read "the two other target shapes". It now names them -- "a stem target and a compound target" -- and so does `bind_control`'s doc, where my first attempt at this round's fix reintroduced "the other two shapes" and I caught it before committing.

## Finding 6: the forward pointer

In entry 31 rather than in entry 30, for finding 3's reason. It states that the plan's `8.61%`/`9.33%` shares and entry 30's bucket table may not be subtracted from one another, with the `--call-graph=dwarf` measurement as the evidence for how far an invocation alone moves a share.

## Not fixed, and why

* **The bare-stem-write optimisation itself.** Confirmed real by the review and being filed separately; not taken here. The review named a third site I had not: `bind_control`'s stem arm, reached on **every pass** of a stem-controlled `DO`, where the slot `control_slot` declines to answer is `Some(0)`. That is now in `control_slot`'s doc and in entry 31, so the next reader of either meets it.
* **`lib.rs`'s and `plan.rs`'s "`Plan::bind` assigns the stem no slot".** True of `CompoundName::stem_at`, which is what those sentences are about, and not an instance of the swept claim. For a **stem-shaped** spelling `bind` does give the *name* a slot while leaving `stem_at` `None`; that entry is never read, since `Code::stem` is reached only from `ExprKind::Compound`. Left alone rather than hedged, because rewriting a true sentence to pre-empt a misreading is where correction rounds add false statements.
