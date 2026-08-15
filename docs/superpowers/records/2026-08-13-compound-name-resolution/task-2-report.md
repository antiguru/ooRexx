# Task 2 report -- the slot, resolved once

Plan: `docs/superpowers/plans/2026-08-13-compound-name-resolution.md`.
Base: `b3e8cf4d2` (Task 1). Commits: `a93bfb548` (the change), `df3c34087` (record entry 28), `c993b6479` (the fix round, on top of `b3e4f11cd`).

The record entry is a second commit on purpose: its `commit` field has to name a commit that already exists, and the record's own rule is that the hash is read back from `git log` rather than written from memory.

## What changed

A compound's variable tail piece carries the slot the upfront pass assigned its name, so joining a tail key costs an index into the frame rather than a hash of the piece's bytes.

* **`rust/crates/rexx-exec/src/plan.rs`**
  * `TailPiece::Variable(Box<[u8]>)` becomes `TailPiece::Variable { name, at: Option<usize> }`.
  * `CompoundName::split` produces `at: None` on every piece, because the split is a property of the text and a slot is a property of the plan the entry is going into.
  * `note_compound_name` keeps the answer `slot_for` was already giving it, instead of dropping it.
  * `bind` leaves an entry that is already recorded alone.
  * `build_records_a_compounds_split_under_the_compounds_own_id` gains the slots and the plan's own name-to-slot map.
  * Three new tests, below.
* **`rust/crates/rexx-exec/src/stem.rs`**
  * `join_tails` reads a piece through the slot when it has one.
  * New `Interp::read_by_name_at(name, at)`; `read_by_name` is `read_by_name_at(name, None)`, so the two paths are one function and an unset piece derives its own spelling identically either way.
  * A debug tripwire in `join_tails` on the one premise that can break.
* **`rust/crates/rexx-exec/tests/ir_dual_cases/compound-names`** -- two new stanzas, both measured against the oracle.
* **`docs/superpowers/plans/2026-08-13-compound-name-resolution.md`** -- Task 2's steps ticked, and one correction to the plan's own measurement section (below).
* **`docs/superpowers/plans/phase-4f-record.md`** -- entry 28, which the plan's own measurement section asks for after both tasks land.

**No source file outside the brief's list was touched.** The brief names `plan.rs` and `stem.rs`, and those are the only two. The case file is Step 4's own deliverable and the two documents are Steps 5 and 6's.

## The fix round

`.superpowers/sdd/2026-08-13-compound-name-resolution/task-2-review.md` returned **spec PASS, quality PASS WITH CHANGES**, on top of `b3e4f11cd`. What changed here, and what did not:

| finding | disposition |
|---|---|
| Entry 28's four control-axis figures are more precise than the instrument supports | **fixed**, in the entry and in this report: measured the instrument's own resolution and restated the claim as a bound |
| The report's percentage restatement of that range is wrong by ten at the low end | **deleted** rather than corrected -- the sentence it lived in is gone with the four figures |
| `note_compound_name`'s comment quotes profile shares without naming the commit | **fixed**: it names `5dc12a403` and says why the commit belongs with the figures |
| No test pins the `do v.i` then `say v.i` build order | **fixed**: `a_compound_seen_after_the_control_variable_still_gets_its_slots`, with the mutation run behind it below |
| A stray one-word line wrap in a `plan.rs` doc comment | fixed |
| The `exec_procedure` write-back | **adjudicated not a defect**, line left alone; the `::ROUTINE` panic behind it is filed separately |
| The `DO`-control decision's stated reason | **corrected**, and the review's proposed replacement reason **does not hold either** -- see the last section |

No behaviour changed in this round. The release binary's `.text` and `.rodata` are byte-identical across it, so every instruction figure below still describes the committed code.

## Step 1: the slot beside the piece, and what a piece without one does

**An `Option` on the piece, because `note_compound_name` and `bind` disagree about slots.**
`note_compound_name` assigns a slot to every variable piece and now keeps it.
`bind` assigns none, deliberately, for the reason Task 1 recorded: `do a.i = 1 to 5` binds the whole dotted `A.I` to one slot, and giving `A.` and `I` slots of their own would move every later slot number in the body.
So an entry existing does not imply its pieces carry slots, and the difference is in the type rather than in a comment: `TailPiece::Variable`'s `at` is an `Option<usize>` and `join_tails` cannot read it without deciding what `None` means.

That spelling is not new here. `Interp::read_stem_at(name, at: Option<usize>)` and `Interp::read_at(code, id, at: Option<usize>)` already carry exactly this shape for exactly this reason -- a slot a compiler resolved earlier, with `None` meaning "resolve it the ordinary way" -- and `read_by_name_at` joins them.

**`bind` no longer overwrites an entry that exists.**
One symbol can reach both: `say v.i` takes `V.I` through `note_compound_name`, and `do v.i = 1 to 2` takes **the same id** through `bind`.
Either entry holds the identical split, since they split the identical spelling, and they differ only in whether the pieces carry slots -- so overwriting changes no answer, and throws the slots away for every reference in the body, in whichever order the pass happened to reach them.
`a_control_variable_does_not_take_the_slots_off_a_compound_already_seen` is that case, and it checks that the `SAY` and the `DO` really do name one `SymbolId` rather than assuming it.

**The other order rests on the other rule, and is pinned separately.**
`do v.i = 1 to 2` then `say v.i` reaches `bind` first, and what replaces its slotless entry is `note_compound_name`'s **unconditional** write. That is a different line deciding the same outcome, so `a_compound_seen_after_the_control_variable_still_gets_its_slots` asserts it; between them the two rules are pinned in the direction each one decides. Neither order can be seen from output, which is why both are entry assertions rather than programs.

**Frame layout is unchanged**, which was the constraint on this step. `slot_for` is called exactly where and as often as it was; only its return value stops being discarded.

## Step 3: the three-source rule -- the answer, and the program that reaches it

**Yes. A compound tail piece can be bound in `extra` rather than in the plan, and the shape that reaches it is a compound `DO` control variable.**

`do za.zi = 1 to 3` binds `ZA.ZI` whole and binds nothing named `ZI` or `ZA.`.
At run time the loop resolves its tail on every pass, so `ZI` is looked up, misses the plan, misses `extra`, grows the frame, and is recorded in `extra` -- where every later pass finds it.

Measured, not argued. An interpreter instrumented to print each growth into `extra` and each hit against it, run on `do za.zi = 1 to 3; nop; end`:

```
PROBE grow-into-extra "ZI" slot 1
PROBE grow-into-extra "ZA." slot 2
PROBE hit-extra "ZI" slot 1
PROBE hit-extra "ZA." slot 2
... (once more per pass, including the pass that fails the test)
```

**That case is exactly the entry class `bind` fills with no slots**, so the fallback is required rather than defensive, and it is why `at` is an `Option`.
`plan::tests::a_tail_piece_with_no_plan_slot_binds_in_extra` is the permanent form of that probe: it asserts the plan holds no name `ZI`, asserts the entry's piece carries `at: None`, runs `tail_key`, and asserts `ZI` ends up in the activation's `extra`.

**A piece that does carry a slot cannot shadow anything, and the reason is the resolution order itself.**
`Interp::slot_of` reads the plan's own name map first and `extra` only after it misses.
A slot lands on a piece only because `Plan::slot_for` put that piece's name into that same map.
So for a piece carrying a slot, `extra` was already unreachable before this task existed: `slot_of` would have returned the plan's answer, which is the number now stored on the piece.
A precomputed slot is therefore not a fourth source; it is the first source, read without hashing.

**Run against that, not only reasoned.** The same instrumented build was extended to print whenever a name found in the plan *also* had an entry in `extra` -- a shadow -- and run over every program under `rust/corpus/` and `rust/bench-programs/` on both engines, 80 programs x 2 engines: **zero shadows**.
The probe site was shown to execute before that sweep, by a control run that printed unconditionally at the same line.
`PROCEDURE EXPOSE`'s own `extra` construction turns out to hold the same rule explicitly (`exec_procedure` inserts a name only `if plan.slot_of(&name).is_none()`), and `procedure-extra` fired zero times over the sweep in any case.

**The premise that can break is a different one, and it now has a tripwire.**
A stored slot is right only while the entry and the running activation come from **one plan**.
`Code::plan` is what pairs them, and the pairing is held by construction: outside `#[cfg(test)]`, a `Code` is built in `run_activation`, which takes `Rc::clone(&self.activation().plan)`, and in `run_fragment`, which passes `None` and so reads no stored slot at all.
`join_tails` now carries a `debug_assert!` that a piece's `at` equals what this activation's plan answers for its name, so every debug run of the suite and of the corpus sweep checks it.
M4 below is the measured witness that it can fail; M4' is the measured statement that it catches nothing the rest of the suite would miss, and why it is kept anyway.

## Step 2: reading the value by slot

`read_by_name_at` is `read_by_name`'s body with the lookup made optional, and `read_by_name` now calls it.
Everything after the lookup is the one implementation it always was: a set slot yields its value, an unset one derives the piece's own (already upcased) spelling through `self.text(name)`, and `join_tails` renders with `to_text` and joins with `.` exactly as before.
Constant pieces are untouched.

## Step 4: oracle captures

Fresh empty directory, programs at absolute paths, `</dev/null`, stdout/stderr/status read separately, every invocation

```
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  /home/moritz/dev/repos/ooRexx/build/bin/rexx /abs/path/FILE )
```

**All twelve stanzas of `compound-names` were re-captured from the oracle at this commit**, not only the two new ones: each program was extracted from the case file, run, and its stdout compared with `cmp` against the bytes the file records.
All twelve exited `rc 0` with empty stderr and matched byte for byte, so Task 1's ten transcripts still hold and the two new ones join them on the same footing.

### The two new stanzas

**11 -- a tail piece introduced by an `INTERPRET`, on both sides of the slot question.**

```rexx
interpret "zi = 4"
zv.zi = 'via fragment'
say zv.zi
say zv.4
interpret "zk = 9"
do za.zk = 1 to 2
  nop
end
say za.9
```

Oracle: `via fragment` / `via fragment` / `3`.

The two halves are deliberately opposite. `ZI` is a piece the enclosing plan **does** hold a slot for, because `zv.zi` is an expression of the body; the fragment's `zi = 4` resolves that same name through `fragment_plan` and writes that same slot, and the read after it sees 4 through the precomputed slot.
`ZK` is a piece the plan holds **no** slot for, because `do za.zk` binds only the whole `ZA.ZK`; the fragment's `zk = 9` binds `ZK` in `extra`, and the loop's tail resolves through `extra` on every pass.
`3` rather than `2` because a controlled loop leaves its control variable holding the value that failed the test, and that final write goes to the same tail -- measured, not predicted.

**12 -- one compound read either side of a `DROP` of its tail variable, and of its own tail.**

```rexx
zi = 1
zv.zi = 'one'
say zv.zi
zn = 'ZI'
drop (zn)
say zv.zi
zi = 1
say zv.zi
drop zv.zi
say zv.zi
```

Oracle: `one` / `ZV.ZI` / `one` / `ZV.1`.

`drop (zn)` is the run-time-named target the brief points at. Its target here is `ZI`, which the plan **does** hold, so the drop clears that slot rather than binding anything new -- and the read after it finds the piece unset, derives `ZI`, and keys a tail that was never written. Setting `zi` again reaches the original tail, so the drop moved the key and not the tail.
The last pair is the tail's own `DROP`: a tombstone, which derives the resolved name rather than falling back to a stem default.

Both stanzas produce byte-identical stdout under this crate on **both** engines (`REXX_ENGINE=tree-walker` and `REXX_ENGINE=ir`), compared with `cmp` against the oracle capture, with empty stderr and rc 0. The debug build carrying the tripwire is what ran them.

## Step 5: gates

Run from `rust/`, each exit status read unpiped.

| gate | result |
|---|---|
| `cargo fmt --all --check` | exit 0 (after `cargo fmt --all`; the diff was in test code this task added, and the release binary rebuilt byte-identical afterwards) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, after `rm -rf target/debug/.fingerprint/rexx-exec-*` so the changed crate was re-linted |
| `memcap 8G cargo test --workspace --no-fail-fast` | exit 0, **1480 passed, 0 failed, 4 ignored** at the fix round's final state |

**Test counts.** Task 1's report records 1477 passed, 0 failed, 4 ignored at `b3e8cf4d2`, which is the brief's stated baseline and is not re-measured here. The commit measured 1479; the fix round's new test takes it to **1480 passed, 0 failed, 4 ignored**.
The three are `plan::tests::a_control_variable_does_not_take_the_slots_off_a_compound_already_seen`, `plan::tests::a_compound_seen_after_the_control_variable_still_gets_its_slots` and `plan::tests::a_tail_piece_with_no_plan_slot_binds_in_extra`; the two new case-file stanzas run inside the existing `both_engines_agree_on_every_case_file`, which is one test.

## Mutations

Whole workspace, `--no-fail-fast`, under `memcap 8G`, each applied by script and restored from a private backup afterwards, verified with `sha256sum -c` and rebuilt (`touch` before the rebuild, for a reason below).

**Every row below was re-run at the final state of the fix round.** The first pass of M1 to M4' was taken before `a_compound_seen_after_the_control_variable_still_gets_its_slots` existed, and that test **is** a new catcher for two of them -- M1 went from two failures to three and M4 from 20 to 21 -- which is exactly the case the re-run rule exists for. The eight rows here are one sitting at one tree state.

| id | mutation | what reddened |
|---|---|---|
| M1 | `note_compound_name` computes each piece's slot and stores `None` -- the answer dropped again, as before this task | **only** the three entry tests: `build_records_a_compounds_split_under_the_compounds_own_id`, `a_control_variable_does_not_take_the_slots_off_a_compound_already_seen`, `a_compound_seen_after_the_control_variable_still_gets_its_slots` (1477 passed, 3 failed) |
| M2 | `join_tails` passes `None` for every piece, so a stored slot is never used | **nothing** -- 1480 passed |
| M3 | `bind` overwrites an entry that already exists instead of leaving it alone | **only** `a_control_variable_does_not_take_the_slots_off_a_compound_already_seen` (1479 passed, 1 failed) |
| M3' | M3, with that test removed from compilation | **nothing** -- 1479 passed |
| M4 | `note_compound_name` stores `slot_for(name) + 1`: a slot the plan does not give the piece's name | 21 failures across 20 distinct tests; the tripwire in `join_tails` panicked 18 times |
| M4' | M4 with the tripwire deleted | 21 failures, the **same 20 test names**, 0 panics |
| M5 | `note_compound_name` stops overwriting (`get_or_insert`), so `bind`'s slotless entry survives | **only** `a_compound_seen_after_the_control_variable_still_gets_its_slots` (1479 passed, 1 failed) |
| M5' | M5, with that test removed from compilation | **nothing** -- 1479 passed |

### What these say, and what they do not

* **M1, M3 and M5 are caught by the entry assertions and by nothing else in the workspace.** A dropped, overwritten or preserved-when-it-should-be-replaced slot is not a wrong answer -- the piece falls back to `read_by_name`, which is what every piece did before this task -- so no output comparison anywhere can see any of them. Only reading the entry back can.
* **M3'/M5' are the measured "adds coverage" results, one per test.** With `a_control_variable_does_not_take_the_slots_off_a_compound_already_seen` removed, M3 goes green across the whole workspace; with `a_compound_seen_after_the_control_variable_still_gets_its_slots` removed, M5 goes green. Each is the only catcher of the rule it pins, and the two rules are different lines: `bind`'s `get_or_insert_with` and `note_compound_name`'s unconditional write.
* **M2 is caught by nothing, and that is correct rather than a gap.** Making `join_tails` ignore every stored slot disables the optimisation and changes no answer, because the fallback is required to agree with the slot. The instrument that sees M2 is the instruction counter below, not the suite. **No test here catches "the slot is not being used".**
* **M4 shows the tripwire can fail**, which is the one thing a `debug_assert` has to be shown to do before its comment claims anything: a slot the plan does not give the name panics at the assertion, 18 times in that run, naming the invariant at the site rather than at whatever read the wrong value.
* **M4' shows the tripwire adds no coverage**, and that is reported rather than glossed. With the assertion deleted, the same 20 test names fail -- a wrong slot reads another variable's value, which the compound unit tests, the corpus differential, the dual-engine sweep, the population sweep and the collector tests all see on their own. **The tripwire is kept for where it fails, not for whether it fails**: it names the invariant at `join_tails` instead of surfacing as a wrong byte somewhere downstream, and it is the same instrument `Op::Load`'s own `at` already carries in `ir/drive.rs` for the same relationship.
* **None of the three entry tests changes any output**, which is why all three are assertions about the table rather than programs. That is the whole reason this task needed mutations at all.

## Step 6: instructions

`perf stat -e instructions:u`, `REXX_ENGINE=ir`, from a fresh empty working directory, **every arm staged at the one fixed path** `.../scratchpad/task2/bench/armbinary` so `argv[0]` is byte-identical between arms (entry 27's artifact).
One run per arm, arms interleaved per axis.
Every `rexxcps` run self-calibrated to `100 x 100`, checked on each arm's own output line, so every arm did the same work.

### This task alone, `b3e8cf4d2` to head

| axis | base `b3e8cf4d2` | head | difference |
|---|---:|---:|---:|
| `rexxcps` | 26,550,115,428 | 25,858,073,812 | **-2.61%** |
| `compound` | 30,588,189,449 | 27,798,509,709 | **-9.12%** |
| `alloc4c` | 8,735,759,297 | 8,456,745,357 | **-3.19%** |
| `strings` | 44,928,589,202 | 44,928,589,154 | below the instrument's resolution |
| `varlookup` | 44,840,637,921 | 44,840,637,964 | below the instrument's resolution |
| `emptyloop` | 27,150,611,005 | 27,150,610,102 | below the instrument's resolution |
| `arith` | 20,403,950,333 | 20,403,950,838 | below the instrument's resolution |

`rexxcps` loses **0.692 billion** instructions, `compound` **2.790 billion**, `alloc4c` **0.279 billion**.

**The four axes holding no compound variable carry a bound and not a difference, and the bound is measured.**
Those four rows are one run per arm, and one run per arm cannot say anything at this size. Three runs per arm, both arms staged at the one fixed path, interleaved, one sitting:

| axis | same-binary span, base | same-binary span, head | arm to arm, minimum against minimum |
|---|---:|---:|---:|
| `emptyloop` | 1,333 | 1,078 | -419 |
| `strings` | 652 | 881 | +6 |
| `varlookup` | 332 | 474 | +523 |
| `arith` | 258 | 33 | +192 |

Every arm-to-arm figure is the size of the spread one binary produces against itself. Taken with the one-run figures above and with the reviewer's independent two-run replication (+229, -558, -506, -699), `strings`, `varlookup` and `arith` have each come out with **both signs** across the three sittings; `emptyloop` read -903, -506 and -419, negative every time, but every one of those is smaller than the 1,078 to 1,333 the same binary spans against itself on that axis.
So what these axes support is: **Task 2 moves them by under about a thousand instructions out of twenty to forty-five billion**, which is this instrument's resolution here, and no signed figure survives.
Task 1's movement on the same four is 7.3 million to 266 million instructions, between 5,500 and 200,000 times that bound, and the combined table below reproduces Task 1's own four percentages exactly (+0.04%, +0.18%, +0.31%, +0.60%) on arms built and run today. That drift is Task 1's, and this task's share of it is unmeasurable rather than small.

**How the first version of this was wrong**, because it is the defect this project produces most: I measured arm-internal spread for `rexxcps`, where the effect is 2.61% and the spread decides nothing, and did not measure it for the four axes where the claimed effect was one part in a billion and the spread decides everything. The four signed figures were three digits from an instrument that has none at that size. The conclusion did not move; only the form did.

**Arm-internal spread**, four runs of each arm on `rexxcps`, interleaved: base 26,546,060,171 / 26,546,068,336 / 26,546,060,492 / 26,550,115,428, a range of **0.0153%**; head 25,858,073,812 / 25,862,126,352 / 25,872,773,451 / 25,863,549,258, a range of **0.0568%**.
Both are far below the -2.61% effect, and the head arm's is the wider of the two.

### Both tasks, the plan's own base `cb4f27d1c` to head

The table the plan's "Measurement, after both land" section asks for, measured as a pair of arms rather than composed from Task 1's table and this one.
`cb4f27d1c`'s arm was built by writing that commit's `eval.rs`, `lib.rs`, `parse_template.rs`, `plan.rs`, `run.rs` and `stem.rs` into `rexx-exec/src` and rebuilding; the head sources were then restored from a private backup, rebuilt, and the head binary came back **byte-identical** to the one already staged.

| axis | base `cb4f27d1c` | head | difference |
|---|---:|---:|---:|
| `rexxcps` | 28,594,847,258 | 25,865,205,907 | **-9.55%** |
| `compound` | 37,163,096,047 | 27,800,230,124 | **-25.19%** |
| `alloc4c` | 9,392,724,256 | 8,456,718,073 | **-9.97%** |
| `varlookup` | 44,574,614,522 | 44,840,637,363 | +0.60% |
| `strings` | 44,787,587,435 | 44,928,589,222 | +0.31% |
| `emptyloop` | 27,100,609,164 | 27,150,610,085 | +0.18% |
| `arith` | 20,396,605,792 | 20,403,950,467 | +0.04% |

**No wall-clock figure is claimed**, for the reason the brief and the plan both give: entry 27 measured layout alone moving these axes between -5.12% and +4.55%.

### A correction to the plan, written into the plan

The plan's measurement section said `compound` was "the only one holding compound variables at all", making every other loop axis a control.
**That is false: `alloc4c`'s inner loop is `tab.i = i`**, a compound with a variable tail piece, which is precisely what this plan changes.
It was found by measuring rather than by reading -- `alloc4c` moved 3.19% on this task alone, which under the plan's rule would have been reported as codegen drift three times the size of anything Task 1 saw -- and then confirmed by reading the program.
Task 1's table does not carry an `alloc4c` row, which is why the claim survived it.
The plan's paragraph now names `compound` and `alloc4c` as subject axes and the remaining four as the control.

## Byte-identity between the arms

One run each, `REXX_ENGINE=ir`: `compound`, `alloc4c`, `varlookup`, `emptyloop`, `arith` and `strings` produce byte-identical stdout **and** stderr and exit 0 under all three binaries, `cb4f27d1c`'s included.
`rexxcps` differs only in its own two timing lines (elapsed seconds and clauses per second), with `100 x 100` on every arm and empty stderr on every arm.

**The measured head binary and the committed one have byte-identical `.text` and `.rodata`.** Comment wording was tightened after the measurement, which changes DWARF line tables and so changes the file; the code sections were compared with `objcopy` and `sha256sum` and are the same bytes, so the instruction counts above are the committed binary's.

## Things I am not sure about, or left undone

* **A compound `DO` control variable still resolves its tail by name on every pass**, because `bind` assigns its pieces no slots and this task did not change that. The decision stands; the reason I first gave for it did not, and the reason the review offered instead does not either.
  * **What I wrote was overstated.** "It changes what `Code::slots`, `Op::Load`'s `at` and `PROCEDURE EXPOSE`'s alias indices all mean" is wrong: all three are derived from the one `Plan`, so a consistent renumbering keeps them consistent. What changes is the numbers, not the meanings.
  * **The review's replacement reason does not hold either, and I checked rather than importing it.** It said `ir` carries slot indices as `u16`/`u32` with `ChunkTooLarge` refusing an over-wide chunk, so a large body could fall back to the tree-walker. `PlanSlot` is a `u32` and `PlanSlot::of` answers `UNRESOLVED` rather than failing, with its own doc comment saying why: refusing a whole body for something that is only an optimisation would be wrong. The `u16::try_from(slot)` that does raise `ChunkTooLarge` (`ir/compile.rs`) is an *expression* slot in `Op::CallExpr`, not a frame slot.
  * **What is left is frame growth, and it is smaller than either.** `push_slots(plan.len())` sizes every activation's frame from the plan, so each new name costs a slot in every activation of that body. But `note_compound_name` already registers the stem prefix and every piece name of any compound the body reads or writes, so the growth is only for a name that appears *solely* as a compound `DO` control variable and nowhere else.
  * So the honest form is that the cost is not knowable without its own measurement, which is the reason not to fold it into a change whose whole claim is that behaviour did not move. `plan::tests::a_tail_piece_with_no_plan_slot_binds_in_extra` is the shape that pays it, resolving through `extra` after the plan's own miss.
* **The stem's slot is still resolved by name at every reference.** `stem_get`/`stem_set`/`stem_assign`/`stem_drop` all open with `self.slot_of(stem_name)`, and `CompoundName::stem` carries no slot. That is the same optimisation as this one for the other half of a compound reference, it is not in this plan's text, and it would want its own measurement -- the stem is one lookup per reference where the tail can be several.
* **The debug tripwire costs a hash per variable piece in debug builds**, which is what the release path used to cost. Test-suite wall clock was not measured before and after adding it.
* **`Code::plan` pairing the plan with the activation is still an invariant held by construction**, not by a type. The tripwire makes a violation loud in debug rather than preventing it, and M4' measured that it catches nothing the suite would otherwise miss.
* **A `cp -p` restore does not rebuild.** Restoring the head sources from a backup preserved their older mtimes, cargo saw nothing to do, and `target/release/rexx-run` stayed the *other* arm's binary while reporting a successful build. It was caught by comparing the rebuilt binary against the staged one rather than by trusting the build, and every restore in this task `touch`es afterwards. This is the recorded stale-binary hazard, reached by a route the recorded version does not name.
* **The `exec_procedure` write-back was adjudicated and is not a defect.** I had flagged `caller.extra = resolved` as replacing the caller's map rather than merging into it. On the only path the oracle permits -- `PROCEDURE` as the first instruction after an internal `CALL` -- the callee's `extra` is a clone of the caller's taken at the call, nothing has run in the callee since, and `exec_procedure` only adds to it, so `resolved` is a superset and the assignment *is* a merge. It would wipe on the `::ROUTINE` path, but that path panics before the wipe is reachable, which is filed as its own task (`docs/superpowers/plans/2026-08-13-procedure-in-routine-panic.md`, and committed as `b3e4f11cd`). The line is left alone.
