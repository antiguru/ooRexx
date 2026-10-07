# Plan review: Phase 6.1

Plan: `docs/superpowers/plans/2026-10-07-phase-6-1.md` at `b89f33392`. Spec: `docs/superpowers/specs/2026-10-07-phase-6-1-loose-ends.md` (R1-R8). Read as the controller dispatching each task to an implementer who sees only that task's text. Every check is a read or a grep on this checkout, quoted where a finding rests on it; no builds. Paths under `rust/crates/rexx-exec/` unless they say otherwise.

## Verdict

**Dispatch after fixes.** Spec coverage is complete except three section 4 clauses (policy mix, M11 through the gate, the regression form). The citations hold. What needs fixing before Task 1 goes out: Task 1 moves the condition field that is already per-activation and does not name the one that is wrong (`ActiveCondition`), and it removes the activity's seed without saying what a fresh activation's unseeded `RANDOM` draws from, which its witnesses cannot tell from a constant. Before Tasks 2-5 go out: each deletes constructors and breaks two harness tables (`refusal-sites.tsv`, `owners.rs`) the plan assigns to Task 7.

| class | count |
|---|---|
| BLOCKER | 0 |
| IMPORTANT | 9 |
| MINOR | 19 |

## Citations and names

`git diff --stat be19fd06a b89f33392 -- rust` is empty, so every line the scouts cited still holds. Every `file:line` the plan cites in Tasks 1-12 was printed with `sed -n` (two batches: the Task 1-3 lines, then the Task 4-12 lines, the harness lines, the SDD and roadmap lines) and matches its description, with one path error (M1). Every existing function, type, constant and file the plan names resolves (`fn condition_copy` is `src/condition.rs:512`, `local_route` `environment/route.rs:31`, `pinned!` `pinning.rs:19`, `linein_line` `input.rs:310`, `debug_pause_after_clause` `run/interpret.rs:202`, `required_source` `lib.rs:632`, `library_source` `lib.rs:622`, `receiver_kind` `dispatch.rs:1754`, `receiver_class` `lib.rs:480`, `scheduler_inconsistency` `lib.rs:814`, `inverted_wait` `:822`, `immovable_reply` `:834`, `next_runnable` `scheduler.rs:1591`, `cancel_wait` `:1032`, `switch_to` `:956`, `Pool::reserve` `scheduler/pool.rs:111`, `set_native_entry` `environment.rs:1241`, `ExecutableRecord` `lib.rs:1655`, `Entry::InternalCall` `activation.rs:427`, `ActivationFlags(u8)` `:495` with `FORWARDED`, `GUARDED`, `RESERVED` `:500-502`, `RegionEnd::Flowed` `ir/drive.rs:283`, `clause_region!` `:1841`, `finish_call` `run/call.rs:1013`, `Posted::Halt` `scheduler.rs:1278`, `run_crate_within` `tests/support/group_runner.rs:425`, `whole_groups` `tests/concurrency_tests.rs:2394`, the pinning report `tests/corpus.rs:1013`, `CLOSED` `tests/closed_phases.rs:38`, `PHASES` `tests/bif_assertions.rs:781` and `tests/keyword_assertions.rs:668`, `SPLIT_TABLE_PHASES` `tests/owners.rs:331`, `bggates.sh` with `W=` at `p6-gates/bggates.sh:6`, the ledger lines for Task 8 I1 (`progress.md:125-127`) and Task 21 N1 (`:302-303`), the origin failure at `bg/b6efbfd53/logs/g4-test-release.txt:2210-2213`, the queued probe texts for `reply`, `pa`, `di`). `size_of::<Op>() == 16` is `ir.rs:61` and `Op::Clause` is the enum's first variant (`:70-71`). The names the plan introduces as produced (`top_level_activation_mut`, `ELAPSED_RESET`, `DEBUG_PAUSE`, `ActivationCold`, `sim.rs`, `pauses_after`, the four new corpus tables, `phase-6-1-gate.md`) do not exist yet, as intended; three names mislead (I4 "tests/loud.rs's row", M3 "the `stress_collect` test", M15 `Constant`).

## IMPORTANT

### I1. Task 1 moves the condition field that is already right and does not name the wrong one

Plan, Task 1 Interfaces: "a cold box `Option<Box<ActivationCold>>` holding `condition` (moved from its inline field), `random_seed` and `locals`". Step 3: "`nested` (internal call, `CALL ON`) copies anchor, cached clock, reset flag and condition from the caller".

Evidence: `activation.rs:364` `condition: Option<TrappedCondition>` is already per-activation and `nested` already copies it (`activation.rs:681` `condition: inherited.condition`; `Inherited` `:940`). The field the `cond1`/`cond2` witnesses show on the wrong side is `activity.rs:120` `active_condition: Option<ActiveCondition>` (`lib.rs:1130`: `raised: Raised, site, sites`), which the plan never names. `run/condition.rs:365-369` keeps the two apart on purpose: "Written here and not onto `active_condition` below, because the two have different lifetimes: this one dies with the activation (`TrappedCondition`), while `active_condition` is the interpreter's one slot for `RAISE PROPAGATE`." The spec said "the propagated condition reuses the existing `condition` field where T1 shows them equivalent"; the plan carries the reuse and drops the showing.

Fix: Interfaces names `ActiveCondition` as what moves into the cold box beside `TrappedCondition` (or merges `raised`, `site`, `sites` into `TrappedCondition`, stating the equivalence check Step 3 must make: the `Raised` a `CALL ON` handler's activation holds is the one `RAISE PROPAGATE` re-raises, `run/condition.rs:398`, `:506-616`, `:988`). Step 3's copy rule names it. Files: `run/condition.rs` lines `:398`, `:506-616`, `:988`.

### I2. Task 1 removes the activity's seed and leaves a fresh activation's first draw undefined

Plan, Task 1 Files: "`activity.rs` (remove ... `random_seed` `:258` ...)". Step 3: "program, routine, method, external and started activations start fresh".

Evidence: `builtin/numeric.rs:561-575` `next_seed` seeds lazily with `interp.activity.random_seed.get_or_insert_with(initial_seed)`, `initial_seed` `:577-582` (clock nanoseconds and pid). Spec section 2: `random_seed` "own, seeded from the activity's generator on first use (the oracle seeds at creation, `RexxActivation.cpp:174`, `:312` ... the activity seed is clock- and pid-derived)". With `Activity.random_seed` gone the plan has no activity-level generator, and "fresh" for the seed could be read as a constant start. The `rnd` witness seeds every activation explicitly (7, 11, 13: `876 457 517 735`), so a constant start passes it. Task 8 Step 4 ("RANDOM's unseeded default ... come from their streams") then has no activity-level state to route.

Fix: Task 1 keeps an activity-level generator (`Activity.random_seed`, or a renamed `random_source`) that each top-level activation's cold box seeds from on its first unseeded draw, advancing it once (the oracle's `Activity::getRandomSeed`, `Activity.cpp:469-475`). Add a predicate witness: two fresh activations (a method called twice, or a method and a routine) print `random(1,1000000)` unseeded and the two values differ, on both engines. Task 8 Step 4 routes that generator's `initial_seed` to the sim stream.

### I3. Tasks 2-5 delete constructors; only Task 7 re-derives `refusal-sites.tsv`

Plan, Task 7 Files: "`corpus/refusal-sites.tsv` (regenerated)". Tasks 2, 3, 4, 5 Step 2: "Delete the refusal", "Delete each refusal's constructor and its pins", "Delete each refusal and its pins", "Delete the `required_source` and `library_source`-style refusals".

Evidence: `tests/refusal_sites.rs:502-528` `the_table_holds_every_constructor_the_source_defines` asserts the committed table equals the derived one on every `cargo test`: "corpus/refusal-sites.tsv disagrees with crates/rexx-exec/src; re-derive it rather than editing the disagreeing row"; the refresh runs only under `REXX_REFUSAL_SITES_REFRESH` (`:503`). Each per-task check of Tasks 2-5 is red until the table is re-derived; Task 7's relabels and Task 8's new sim constructors do the same.

Fix: every task that adds, deletes or relabels a `Loud` constructor (2, 3, 4, 5, 7, 8, 9) names `corpus/refusal-sites.tsv` in Files and re-derives it in its last step with `REXX_REFUSAL_SITES_REFRESH=1 cargo test -p rexx-exec --test refusal_sites`.

### I4. Task 3's OPTIONS removal breaks `owners.rs` and `loud.rs`, which it does not name; "tests/loud.rs's row" does not exist

Plan, Task 3 Step 2: "b1 evaluates and traces the expression and does nothing else (remove `Options` from `instruction_owner` and `tests/loud.rs`'s row)". Task 7 Step 3: "prune `"Phase 5"` from the `PHASES` and `SPLIT_TABLE_PHASES` vocabularies and `owners.rs`'s count".

Evidence: `tests/owners.rs:154` `InstructionKind::Options { .. } => ("Options", Owner::Phase("Phase 5"))`, `:324` `("InstructionKind", "Options", "Phase 5")`, `:468-473` asserts exactly one `Phase 5` instruction tag. `tests/loud.rs` has no rows of its own: `table_owner` (`:40-55`) reads the owner from `owners.rs`'s table, and `every_out_of_scope_variant_fails_loudly` (`:251`) requires each Phase-owned instruction to refuse. Once OPTIONS runs, `loud.rs` fails if the tag stays and `owners.rs:468-473` fails if it goes. Task 2 has the same shape with `LoopKind::With` (`owners.rs:213`, `:325`; Task 2 names the tag, not `:325`).

Fix: Task 3 Files add `tests/owners.rs` (`:154`, `:324`, `:468-473`); Step 2 reads "remove the Options tag from `owners.rs`'s `INSTRUCTION_TAGS` and its `:324` row, and set the Phase 5 count assertion to 0", dropping "tests/loud.rs's row". Task 2 adds `:325`. Task 7 Step 3 keeps the `PHASES` and `SPLIT_TABLE_PHASES` prunes only.

### I5. Pins of `DO is not implemented` outside the files Task 2 names

Plan, Task 2 Files: "`run/tests/loops.rs:196-228`, `tests/concurrency_tests.rs` whole-group expectations naming `DO is not implemented`, `tests/owners.rs`".

Evidence (`git grep -n 'DO is not implemented' -- src tests`): `src/run/tests/loops.rs:575` and `:600` (two more mutation-kill asserts, "a construct 4a does implement must not be attributed to a phase"); `src/scheduler/tests.rs:1286-1316` (a refusal raised inside an UNINIT during collection, built from `do counter c over .array~of(1)`, counted at `:1294` and `:1316`); `tests/ir_recorded_cases/loop-refusals` (three recorded cases with `rc> 120`, `err> rexx-exec: DO is not implemented`, "the three DO/LOOP forms this crate refuses").

Fix: name all three. The scheduler test needs a refusal that survives 6.1 to keep its subject (one of the LIMIT constructors); the recorded cases become ordinary cases with the oracle's output or are deleted.

### I6. Task 10 runs no policy mix

Spec, section 4 Gate: "Policy mix: `pre:1`, `pre:2`, `pre:3`, `pct:3`, `uniform:0.01`, `uniform:0.2`, `uniform:1`"; R5 puts `pct` in the mix, judged by invariants only. Plan, Task 10: `REXX_SIM_POLICY` is an override; no step says what each seed runs under. An implementer running every seed under `pre:1` has met Task 10 as written and not criterion 6.

Fix: Step 1 (or a step of its own) states the mix per seed, carried in `sim-gate.tsv`; Step 2's "(not for `pct`)" then has rows to apply to; the self-test of Step 4 names which policy it reruns.

### I7. M11 reds a crate test in the plan; the spec requires the seeded gate to red on it

Spec, section 4 "The gate can fail": "Criterion 6 requires the seeded gate itself, not a crate test, to go red on re-introduced defects ... And M11 in a group-shaped program under `fail=wait:K`: the stale sleeper readies an activity parked on a result, which the invariant catches when written as 'a ready activity holds no park reason'". Criterion 6: "the gate went red on each re-introduced defect of section 4". Plan, Task 10 Step 5 lists the two ledger reverts and the alternate only; M11 is Task 11 Step 1, a crate test.

Fix: Task 10 Step 5 adds a gate program for M11 (a `sim-gate.tsv` row with `fail=wait:K`, excluded from the oracle report per Step 3) and records the gate's red under the `sleepers.retain` deletion (`scheduler.rs:1047-1049`) with its replay line; Task 11 Step 1 stays as the regression form.

### I8. "The scheduler test programs" have no route to a seed

Plan, Task 10 Step 1: "One seed per group part of Phase 6 criterion 1, plus `corpus/phase-6.txt` and the scheduler test programs, in both builds".

Evidence: the scheduler tests build their programs inline and pick a mode in code: `src/scheduler/tests.rs:347` `fn run_with(source: &str, invocation: Invocation)`, `.with_switch_mode(SwitchMode::EveryOpportunity)` at `:365`, `:447`, `:502` and on; no environment hook. The `phase-6.txt` programs have one (`tests/corpus.rs:54` `SWITCH_ENV`); the group parts have `run_crate_within`. The third set cannot be run per seed without a mechanism the plan does not name.

Fix: either a committed directory of those programs' texts listed in `sim-gate.tsv` and run through `run_crate_within`, or drop the set from the gate with the reason recorded in the gate record and the spec's Gate paragraph amended at Task 12.

### I9. The regression form for a found bug is missing

Spec, section 4: "A found bug lands as a crate test with explicit switch points and an assertion that it reaches the defect's site, so a later commit that moves the schedule fails loudly rather than passing." Plan, Task 10 Step 2: "a defect to fix in this task"; Task 11 Step 3: "A defect either shows is fixed here". Neither says how the fix is witnessed.

Fix: add the sentence to both steps.

## MINOR

M1. `phase-4-exclusions.txt` is cited bare in Tasks 2, 5 and 7, and Task 5's Files puts it beside `corpus/errors/parse-errors.tsv`. It is `docs/superpowers/plans/phase-4-exclusions.txt` (`tests/closed_phases.rs:325`); `rust/corpus/` has no such file. Lines 530, 844, 1895 and 1925 match at the docs path.

M2. Task 7 Interfaces: "Task 9 adds LIMIT rows for its sim refusals". Task 8 Step 4 adds them. Say Task 8.

M3. Task 8 Files and Step 4: "`gc=q` inside the existing `stress_collect` test". `stress_collect` is a field (`lib.rs:1545`), read by `collect_if_due` (`:2645`), set by `enable_stress_collect` (`:2626`). Say "the `stress_collect` branch of `collect_if_due`".

M4. Task 8 Files lists `clippy.toml (disallowed_methods)` among existing files. No `clippy.toml` exists in the crate, under `rust/` or at the root. Say it is new and where it goes (`rust/clippy.toml` for the workspace).

M5. Task 8 Step 3's command, run as the plan words it, also prints `src/run.rs:573` (`#[cfg(test)]`) and `src/sync.rs:233-255` (inside `#[test]` functions in `sync.rs`), which `clippy --all-targets` lints and the plan's allow-list omits; and `src/builtin/numeric.rs:578`, `src/builtin/datetime.rs:760`, which the seam replaces rather than allows. Say which of the printed sites the seam takes and which are allowed.

M6. Task 3 Step 1's globs over-reach and "Each refuses today" is false for the extras: `b_do_*` includes `b_do_header_env`, `b_do_over_env`, `b_do_over_env2` (same on both engines, scout A results lines 1699-1702; b11 GUARD); `b2_do_*` includes `b2_do_for_env`, `b2_do_rep_inst` (same); `b3_forward_args_*` includes `b3_forward_args_weak` (same); `b_forward_args_env` and `b_forward_args_inst` differ as 35.1 parse errors (c8's class), not b10. List scout A's b10 names.

M7. Task 4 Step 1: "scout D's class-method probes: `self~setMethod`, `self~unsetMethod`, EXPOSE in a class method; oracle `42`, `42 5`, `ok`". Scout D's probes are `cls_setm` (`42` `42`), `cls_setmobj` (`42 5`), `cls_unset` (`ok`); none is an EXPOSE-in-a-class-method probe (the EXPOSE sits inside `cls_setmobj`'s OBJECT-scope method). Name the three; give a text if a fourth is wanted.

M8. Task 4 Step 1 lists `b6_define_array_subclass_with_primitive`, `b10_define_constant_method`, `b10_define_attr_method` under b5. Scout A section 4 puts their refusal in class (a) (`method "TEST1" ... (Phase 9)`) with b5 as the "likely" root, "not confirmed". Step 3 "Witnesses agree" has no branch for them still refusing after b5. Say: if they still refuse with a Phase 9 label after b5, record them under row 9, not fix.

M9. Task 1 Step 1's `debug_pause` witness and the per-field REPLY probes have no text. Give the shapes: the `dbgcall` probe with the pause lines the oracle prints; for REPLY, a method that (a) `call time 'r'`, replies, then prints a `time('e')` predicate, (b) seeds `random`, replies, draws again, (c) `setlocal`, replies, `endlocal` (`sl2` covers), (d) traps a condition, replies, `raise propagate` in the continuation, (e) `select case` with a REPLY inside a WHEN and an absorbed WHEN after it; each run on the oracle 5 times.

M10. Task 1 Step 3 never says when `DEBUG_PAUSE` is set. Spec table: "a typed debug line: true for its duration" (`run/interpret.rs:256` `replace_debug_pause(true)`, `trace.rs:57`); INTERPRET proper and every callee: false. Add the sentence; Task 6 reads the bit.

M11. Task 9 Step 1: "`uniform:1` matches `EveryOpportunity`'s outcome on the switch-mode tests". With the one-event seeded pick on and virtual time, a legal outcome can differ from `every`'s. Say `uniform:1,order=fifo` and exclude tests whose output reads the clock.

M12. Task 10's committed oracle outcome sets (`rust/corpus/sim-oracle/`) have no populating step; Step 3 only says they grow under `REXX_SIM_ORACLE_REFRESH=1`. Say Step 1 writes them from the harness's `ORACLE_RUNS` runs (5, 30 when unsettled) under the refresh variable.

M13. Task 10 Step 5: Task 8 I1's fix is one commit with I2 and I3 (`progress.md:127` "7eafa3b58 (I1 waiter list, I2 CANCELED from waiter, I3 32-bit remainder, minors)"), and Task 21 N1's fix sits among F1r, N2, N5-N7 in `f9fb61990`/`2e50fc410` (`:303`). Reverting the commit reintroduces the others and the red is unattributed. Say "revert the fix's hunk, recorded by commit and file".

M14. Task 11 Step 4: "judged by the counter" has no expected values. Say: 0 unmutated; above 0 under the mutant kills it; 0 under the mutant with the program named records it equivalent.

M15. Task 6 Interfaces lists `Constant` among the instruction kinds that pause. `::CONSTANT` is a directive (`interpreter/instructions/ConstantDirective.cpp`, caught by scout B's `grep` over that directory); ours has no such clause kind. Drop it.

M16. Review Focus 4 says no per-clause branch is added, and the memory it rests on prices one at 1.25% on `emptyloop` against 0.5% on `rexxcps`. Task 9 Step 4's list has `rexxcps` and not `emptyloop`. Add it.

M17. Spec section 3: "A GUARD keeps its constructor with owner `None` and a doc comment citing its probes." Task 7 Step 2 puts the probe names in the tsv only. Add the doc comment to the GUARD relabels.

M18. Task 7 Step 2 "DEVIATION: ... crashing rows in `oracle-crashes.txt` (List ITEMS, ...)" reads as citing an entry that does not exist yet (`grep -n 'ITEMS' rust/corpus/oracle-crashes.txt` prints nothing; the file's last entry is 12b). Say Task 7 Step 2 writes the entry and the exclusions row from Task 4 Step 3's probe.

M19. Task 8 Interfaces and Step 5 print `trace=H` (`SimReport.trace_hash`) before Task 9 Step 4 defines the decision trace. Say Task 8 hashes the decisions it already draws (clock quanta, order picks) or leaves the field to Task 9.

## Pairs sharing a file or interface

| pair | shared | found |
|---|---|---|
| Task 1, Task 6 | `ir/drive.rs`, `run/interpret.rs`, `run/call.rs`, `trace.rs`; `DEBUG_PAUSE` | Task 1 deletes `ir/drive.rs:2984-2986`; Task 6's `:1905`, `:1930`, `:1899-1901` sit above the deletion. The rule for setting the bit is in neither task (M10). Order holds. |
| Task 1, Task 8 | `builtin/datetime.rs`, `builtin/numeric.rs`, `lib.rs` | Task 1 removes the activity seed Task 8 Step 4 must route (I2). Task 8's seam replaces `datetime.rs:760` and `numeric.rs:578`, inside ranges Task 1 rewrites (`:777-788`, `:857-876`, `:560-582`); Task 8's citations drift after Task 1, harmless since it names the functions. |
| Task 1, Task 11 | `scheduler/tests.rs`, `activity.rs` | Task 1 rewrites `:1058` and adds concurrent witnesses; Task 11 appends M11. `activity.rs:607` is untouched by Task 1. Consistent. |
| Task 2, Task 3 | `run/loops.rs`, `lib.rs` (`instruction_owner`), `tests/owners.rs` | Task 3's `run/loops.rs:716`, `:2233` drift after Task 2's loop rewrite (named by constructor, so findable). Both prune `owners.rs`; the count assertion is assigned to Task 7 (I4). |
| Tasks 2-5, Task 7 | `corpus/refusal-sites.tsv`, `tests/owners.rs`, `tests/concurrency_tests.rs` expectation lines | I3, I4. The expectation lines each task rewrites are verified only by `whole_groups` under `REXX_CORPUS_GATE=1` (Q1). |
| Task 3, Task 4 | `dispatch.rs`, `run.rs`, `lib.rs` | Disjoint sites (`dispatch.rs:3059`, `:3078` vs `:2082-2083`, `:1102`; `run.rs:1856`, `:1197`, `:1620`, `:2281` vs `:1536`). Both delete `lib.rs` constructors (I3). |
| Task 4, Task 7 | `lib.rs` `receiver_class`, `oracle-crashes.txt`, exclusions row | Task 4 Step 3's probe feeds Task 7 Step 2's row; who writes the entry is unstated (M18). Order holds. |
| Task 5, Task 7 | `phase-4-exclusions.txt:530`, `:1895-1925`; `lib.rs` `required_source`, `library_source` | Task 5 rewrites both rows; Task 7 keeps `library_source` (b17) as GUARD. Consistent; path wrong in both (M1). |
| Task 5, Task 6 | `run/interpret.rs` | Task 5 edits `run_fragment` (`:28-37`); Task 6's INTERPRET pause ("before an INTERPRET fragment's first clause") lands in the same function after Task 5. Order 5 before 6 holds. |
| Task 7, Task 8 | `corpus/refusal-dispositions.tsv`, `tests/refusal_dispositions.rs` | Task 7's test reds on Task 8's new ownerless constructors until the LIMIT rows land in the same commit; Task 7 says Task 9 adds them (M2). |
| Task 8, Task 9 | `sim.rs`, `scheduler.rs` (`:1990-1999`, `next_runnable`), `Policy`, `SimReport` | Task 8 parses `fifo`, Task 9 fills `Policy`: consistent. `trace_hash` printed by Task 8 before Task 9 defines the trace (M19). The fairness floor's figure comes from Task 8 Step 2: consistent. |
| Task 8, Task 10 | `invocation.rs` `SwitchMode` vs `tests/support/group_runner.rs:75` `SwitchMode` | Two enums; Task 10 adds `Sim` to the test-side one and maps it. Consistent. |
| Task 9, Task 11 | `scheduler.rs` (`cancel_wait :1034-1049`, `switch_to :976-977`), `scheduler/tests.rs` | Task 9 rewrites `next_runnable` and the readying sites; Task 11's sites are elsewhere. The invariant Task 11 relies on ("a ready activity holds no park reason") is Task 9 Step 3's. Consistent. |
| Task 10, Task 11 | `tests/concurrency_tests.rs`, `phase-6-1-gate.md` | Both append; I7 moves M11's gate red into Task 10 and leaves the crate test in Task 11. |
| Task 10, Task 12 | the seeded gate, `phase-6-1-gate.md`, `whole_groups` | Task 10 Step 6 runs the gate in release; Task 12 Step 4 repeats it with the debug subset. Consistent. |

## Checks that could not fail, steps that need guessing

None of the checks is vacuous: Task 1 Step 2's assert, Task 7 Step 4's grep, Task 10 Step 4's fresh-process hash, Task 10 Step 5's reverts, Task 11 Step 1's deletion and Step 2's count change can each fail. Steps needing a guess are I1, I2, I6, I8 and the MINOR items M9, M10, M12, M14. Every Review Focus line has a test in a task (RF1: Task 1 Step 1 and Step 3; RF2: Task 1 Step 1 and the `fibcall` figure; RF3: Task 2 Steps 2-3; RF4: Task 8 Step 5 and Task 9 Step 4, with M16; RF5: Task 9 Step 1 and Task 10 Step 4).

## Questions only Moritz can answer

| # | Question | Recommendation |
|---|---|---|
| Q1 | The per-task check is `REXX_CORPUS_GATE=1 ... cargo test --workspace` in debug. That variable gates the test binaries `concurrency_tests` (`tests/support/group_runner.rs:50`), `corpus` (`:40`), `oracle_deadline` (`:23`), `parse_version_oracle` (`:25`), `input_oracle` (`:25`), `outer_context` (`:35`), `ir_recorded_oracle` (`:40`) and `bootstrap_install_oracle` (`:26`); `whole_groups` is in the first, and that binary alone took 825.39 s in release at the S5 close (`.superpowers/sdd/2026-10-01-phase-6-s2-s5/bg/42b29493c/logs/g4-test-release.txt:2069`). The 2026-10-04 ruling was fmt, clippy and plain `cargo test` per task, full gates at stage closes. Does 6.1 run `whole_groups` on every task? | Per task: `REXX_CORPUS_GATE=1` on `--test corpus` and `--test ir_recorded_oracle` (the witnesses need the strict mode) plus plain `cargo test --workspace`; `whole_groups` in release at the close of Tasks 3 and 5 (the tasks that rewrite expectation lines), Task 10 (the sim gate) and Task 12. |
