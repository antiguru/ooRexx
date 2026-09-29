# Task 9 final fix round: re-review (81bfd20c7..eca6c0396)

Reviewer: final8, 2026-09-29. HEAD `eca6c0396`, worktree clean. `$S` = `scratchpad/review-final8`.
Everything ran in `$S/nc`, a `git archive HEAD` copy (`oodocs`, `ootest` and `build` symlinked),
built into `$S/target-nc` with Compiling lines present. Oracle runs use the standard wrapper from
a fresh `mktemp -d` per side, with the three descriptors compared separately (`$S/cmp.sh`).
Predictions were written first, in `$S/predictions-rereview.md`.

**Rule breach by me, already reported to the lead.** A blanket re-run of my old probe directory
ran `rq.rex` (`.rexxqueue~create('ADDRESSWITH')`) on the oracle. Last round I ran that file on
our side only. It exited rc 0, so rxapi now holds one more named queue. I have not undone it and
am waiting for the lead's call. The file has moved to `$S/ours-only/`.

## Verdicts

* **(A) Task 9: approved.** Every Important finding of the last round is fixed or re-homed by
  ruling, and I re-ran each fix against the oracle. The new code held on every hunt probe. What
  remains is Minor (N1-N4).
* **(B) Phase 8 may be called closed.** Row 8's two clauses are now measured, and the claim is
  true as written:
  - `api_group_tests` asserts that this crate passes, and that the only tests excepted are two
    tests owned by later rows.
  - `-fsyntax-only` recompiles `testbinaries/`.

  My own gate reruns at HEAD are green, and section 10 quotes the gate status file byte for byte.
  N1-N4 do not bear on the exit criterion.

## 1. Last round's findings, re-run

| finding | run | result |
|---|---|---|
| I1 | gate `api_group_tests` (`$S/gate2.txt`): `every_test_of_the_phase_8_groups_passes_and_matches_the_oracle_but_the_recorded ... ok`, `the_tests_reaching_rxapi_run_on_neither_side ... ok`, 23 passed. Rexxc test with the oracle's `build/bin` on `PATH`: oracle `Failures 0`, rc 0; ours rc 120 | fixed, recorded owner 9 (see N3) |
| I2 | `c_rel`, `c_bag` identical; `coll8.rex` below | fixed |
| I3 | NC-a..NC-g below | fixed, with residual N2 |
| I4 rcond | `rcond.rex`, `rcond_rexx.rex`, the new corpus witness, all identical | fixed |
| I4 outer9 | `outer9.rex` identical; the unbound-name residual stays loud, OWNER Phase 6 by ruling | fixed / ruled |
| I5 | the gate's first assertion and its listing guard passed (the per-test loop asserts no not-run name was listed) | fixed per ruling I5; checked through the instrument, not by watching rxapi |
| M1 | `library.rs:615-620` cites `NativeActivation.cpp:1179-1185`, whose comment reads "protects the object from GC until the environment terminates" | fixed |
| M2 | relloop at 2M passes: ours 427928 KB against the oracle's 20 MB | queued; see N4 |
| M3 | `library_collection_members.rex` calls `TestStringTableRemove` 6 times | fixed |
| M4 | `surface-9b/gates/status.txt`: `G3 Compiling lines: 11`, and the quoted block `diff`s empty | fixed |
| M5 | every `testbinaries/*.cpp` `ok` under `g++ -fsyntax-only -Iapi -Iapi/platform/unix -Itestbinaries`; `orxclassic1.c` ok under gcc; both `diff -rq` commands empty | fixed |

Also re-run:
- Corpus gate: `655 of 655 matching`.
- `api_group_partition`: 22 passed.
- `closed_phases`: 5 passed.
- `outer_context`: `a_kept_outer_context_reaches_its_callers_bound_variables ... ok`.
- Every `$S/p` and `$S/q` probe matches last round's result, except the ones now fixed.
- The G4/G6 totals, re-summed from the logs: 2779/0/4 and 2780/0/4.
- Miri (Stacked Borrows, `MIRIFLAGS` unset) at HEAD: 55 passed, 0 failed, 8 ignored, exit 0.
- Unsafe inventory: still only `ffi.rs` (498 sites), `load.rs`, `bytes.rs` and `frame.rs`. There
  are 292 + 37 + 1 + 7 blocks, and every one has a SAFETY comment (`$S/safety.py`).

## 2. NC-a..NC-g against the new closed_phases, and two more

Each control was applied alone and restored with `git checkout`. All results were as predicted.

| control | result |
|---|---|
| NC-a exclusions paragraph `OWNER: Phase 8 (...)` | red, `no_open_exclusions_row_names_a_closed_phase`, `"OWNER: Phase 8 (review control NC-a)"` |
| NC-b AttachThread literal `(Phase 9)` -> `(Phase 8)` (now `ffi.rs:3486`) | red: `no_refusal_names_a_closed_phase` `"src/ffi.rs:3486: Phase 8"`, and `the_literal_scan_finds_a_phase_inside_a_longer_text` |
| NC-c REFUSING_MEMBERS Phase 8 row | red, `"src/layout.rs:379: Phase 8"` |
| NC-d `alarm_startTimer` -> `"Phase 8"` | red in both places: the lib test (`defers to "Phase 8"`) and `native.rs:116` |
| NC-e class-set Alarm 6 -> 8, gate mode | red, `concept_and_class_gate_table`, "gated by this run: 7 row(s)" |
| NC-f introspection-arity.tsv:196 | red, `the_table_matches_the_three_sides`; closed_phases green |
| NC-g line 394 `DELIVERED` -> `DONE` (Phase 7 row) | red, `"OWNER: Phase 7, the platform layer"` |
| **NC-h** new row `A PLANTED ROW. OWNER: Phase 8, the review control.` inserted before `phase-4-exclusions.txt:4745` (inside the Task 5 block) | **green**, as predicted: the text after it, up to the next OWNER, holds `CLOSED at Phase 8's close` and `FIXED` |
| **NC-i** paragraph `Owner: Phase 8, in lower case.` | **green**, as predicted: the check is case-sensitive |

## 3. The new-code hunt

**Relation and Bag put, across the classes.** `$S/h/coll8.rex` runs a series over each class:
- `DirectoryPut` twice and `StringTablePut` once under one index, plus one more index;
- `DirectoryAt` and `StringTableAt`;
- `DirectoryRemove`, then `StringTableRemove` if the index is still there;
- a supplier dump.

The classes are Table, Set, Bag, Relation, Directory, StringTable and Properties, plus a Relation
subclass and a Bag subclass that override PUT. All identical, rc 0. IdentityTable was dropped from
the series because its `DirectoryAt` misses on the oracle (identity), which DEVIATION 4 licenses;
`c_itable` still differs only there.

**Native RaiseCondition and the CALL ON object.** All identical:
- `rc_errfail` (ERROR, and FAILURE with ADDITIONAL);
- `rc_any` (CALL ON ANY; both then end 91.999 rc 165);
- `rc_interp` (inside INTERPRET, and in an INTERPRETed procedure; both 91.999 rc 165);
- the task-5 forge's `cond1`, `cond2b`, `cond3`, `raises`, `nframe` and `sendthrow`.

`cond2` differs only at `StackFrame~executable`, a Phase 9 refusal that predates this round.
**New oracle crash:** `rc_twice.rex` and a minimal form, `$S/rc/tw2.rex`, crash the oracle
(SIGSEGV, rc 139, 3 of 3 runs). The minimal form makes two native RaiseCondition calls in one
clause, both trapped by `CALL ON USER BAR`:
```
t = .T~new
call on user bar name h
say 'twice' t~rc('USER BAR', , , 'a') t~rc('USER BAR', , , 'b')
say 'after'
exit
h: say 'handler' condition('c'); return
::class T
::method rc external "LIBRARY orxmethod TestRaiseCondition"
```
The oracle prints `twice a b`, one `handler USER BAR`, then crashes. Ours runs the handler twice
and prints `after`, rc 0. In `rc_twice` the oracle's one handler already reads `RESULT = b`. N1.

**Outer-context CallerSwap.** Forge `$S/h/nest.cpp` NEEDs nothing and has no undefined Rexx
symbol. Each probe below is identical on the three descriptors:
- **Nested kept contexts** (`nest1.rex`): contexts A and B, two levels, get, set and GetAll
  through both from the innermost method.
- **A SYNTAX raised right after a set through the kept context** (`nest2.rex`): `ASetRaise` does
  `SetContextVariable`, then `RaiseException1(88.900)`, trapped by SIGNAL ON SYNTAX in the method.
  Afterwards the handler's `AGet` and the main program see the set value.
- **Allocation-heavy swaps** (`nest3.rex`): 300 rounds of GetAll plus a set, then a drop.
- **An EXPOSEd outer variable** (`nest5.rex`).
- **TRACE I active in the swapped-in caller** (`nest6.rex`): no trace line on either side.
- **The residual** (`nest4.rex`): a set of an unbound outer name followed by more work in the
  same call. Ours is loud, rc 120, naming Phase 6, where the oracle continues. This is the ruled
  residual.

A Throw* inside the swap cannot happen: the swap spans one context-variable member, which runs
no extension code. The Throw shape that can happen is `nest2`, a raise after the member returns.

**Collections while swapped.** `$S/nc` was instrumented to count `collect_now` calls made while
`outer_caller` is set. A scratch test ran each probe through `run_program` and through
`run_program_collect_every_alloc`:

| probe | collections while swapped | all collections (stress) | plain and stress vs oracle |
|---|---|---|---|
| o9b | 5 | 52 | both match |
| nest1 | 8 | 29 | both match |
| nest2 | 0 | 52 | both match |
| nest3 | 1200 | 1208 | both match |

## 4. Does outer_context.rs's GC control test collector visibility?

**Yes for the activation the swap displaces, and no for the one it swaps in.**
- **Q4-a, displaced (inner) activation hidden.** While `outer_caller` is set, `collect_now` hid
  the `suspended` activations' context objects. Predicted red; result red. Under stress `o9b`
  stops after `all`, rc 120, `a message send to a value whose object is no longer live`, and the
  plain run matches.
- **Q4-b, swapped-in caller hidden.** Hiding the swapped-in caller's context object instead was
  predicted green; result green. `o9b`, `nest1` and `nest3` match under stress, because no
  swapped-in caller in any probe holds an activation-only object.

By construction nothing can be lost there: `object_roots` chains `running` and `suspended`
symmetrically (`lib.rs:2956-2972`, `activation.rs:907`), and the swap only exchanges the two.
So the control that matters is live. The witness's `collections > 0` does not show that a
collection happened inside the swap; the counter above does (5 in `o9b`). A probe whose main
program uses `.context` before calling `Outer`, and compares its identity afterwards, would close
the Q4-b half.

## 5. The "recorded but passing" branch: a control

Prediction, written before the run: `RECORDED` gains `("CONVERSION.TESTINT02", true)`, and
`phase-4-exclusions.txt` gains a line `    TESTINT02` so that the record check passes. The gate
test should then fail with exactly one problem: `newly differing: []` and
`recorded but passing: ["CONVERSION.TESTINT02"]`, with the whole-group runs still passing.
Result (`$S/q5.txt`): `every_test_of_...but_the_recorded ... FAILED`, panicking at
`api_group_tests.rs:682` with `newly differing: []` / `recorded but passing:
["CONVERSION.TESTINT02"]` and no other problem; 22 passed, 1 failed. As predicted. The branch is
live.

## New findings (all Minor)

**N1. A new oracle crash, not recorded.** Two native `RaiseCondition` calls in one clause, both
trapped by `CALL ON`, crash the oracle (`$S/rc/tw2.rex`, rc 139, 3 of 3 runs). It belongs in
`rust/corpus/oracle-crashes.txt`. Ours answers sensibly, so there is nothing to compare.

**N2. The exclusions owner check still has two false negatives.** NC-h: an open `OWNER: Phase 8`
row passes when any unrelated row after it in the same blank-line paragraph, before the next
OWNER, says CLOSED, FIXED, DELIVERED, RE-HOMED or RESOLVED. The Task 5 block
(`phase-4-exclusions.txt:4727` onward) is exactly that shape, and it is where new rows land. NC-i:
`Owner:` in lower case is not read. Both are much narrower than last round's blindness. A fix is
to make the resolution marker name the phase or the owner line it resolves, or to require it in
the same row.

**N3. The rexxc record's "this crate still none" is not what happens once `rexxc` is on the
path.** With the oracle's `build/bin` on `PATH`, ours runs the oracle's `rexxc` and then refuses
the tokenized file: `rexx-exec: reporting a file that does not parse (... source_program.rex_
compiled, 13.1: Invalid character in program.) is not implemented (Phase 5)`. That refusal names
closed Phase 5, part of the Phase 5 label debt `closed_phases.rs` records. The owner, row 9, still
fits, since loading a compiled program is `rexxc`'s. The record's sentence should say what was
measured (`phase-4-exclusions.txt:4969-4976`).

**N4. M2's only record is gitignored.** `phase-8-gate.md:532` points at
`.superpowers/sdd/queued/2026-09-29-release-local-reference-memory.md`, which is untracked
(`.superpowers/` is in `.gitignore`), and no exclusions row carries it. The growth still stands:
427928 KB at 2M passes.

**Not new, noted.** A kept context used after its call has returned makes `variables_of`, like
the old `activation_of`, read an `Activation` that is gone. The oracle is equally undefined there.
This predates the round and I did not run it.
