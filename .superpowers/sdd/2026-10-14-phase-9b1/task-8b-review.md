# Phase 9b1 Task 8b review: the three remaining leaks (efbf0fa1d..aaf302608)

Reviewer probes and the probe runner are in `task-8b-review-files/` beside this file (`refs/` for
Step 3, `behaviours/` for Step 2, `review_probes.rs`). The runner is the Task 8 reviewer's
(`run_program_collect_every_alloc` per `.rex` in `PROBE_DIR`), plus `PROBE_SWITCH=1` for
`SwitchMode::EveryOpportunity`. I built it in a git-archive copy of `aaf302608` (`rust/` and
`interpreter/`) at `/tmp/t8brev/tree` with `CARGO_TARGET_DIR=/tmp/t8brev/target`, under `memcap 8G`,
`-j 4` and `CARGO_INCREMENTAL=0`. The copy and its target dir are deleted. Oracle runs used the
standard wrapper, 3 runs each, from a fresh empty directory per program. Every program ran 3 of 3
identical on the oracle except where stated. Crate runs were under `memcap 4G`. The filler is 300
passes of `copies('x', 100000)` on both sides. The head binary I built is kept at
`/home/moritz/dev/repos/ooRexx-p9-pin/cg-9b1/t8b-review/rexx-run-head`.

### Spec Compliance

- ✅ Spec compliant. Each of the four steps is implemented as briefed:
  - Step 1: `MethodRows` (`rexx-exec/src/method_rows.rs`), pruned by `forget_method_identity`
    (`dispatch.rs`) and `prune_program_rows` (`lib.rs`).
  - Step 2: `ClassGraph::free_unheld_behaviours` plus `Interp::free_unheld_behaviours`, called from
    `collect_now`.
  - Step 3: `Body::Cell`, traced from `Body::VarRef` and from `ActivityRoots::iter` aliases, released
    by generation in `RootSet::release_cells`.
  - Step 4: the bench set.

  The rexx-classes files and the index-table `RootSet::cells` are covered by the rulings in
  `progress.md`. Both loop witnesses and the perf budget reproduce (below).
- ⚠️ Cannot verify from the diff: the native-API cell path (`dispatch/library/surface.rs`,
  `variable_reference` with `object == false`) cannot be reached from Rexx with the oracle's test
  libraries. `grep -n "VariableReference" /home/moritz/dev/repos/ooRexx/testbinaries/*.cpp` finds
  only `TestGetObjectVariableReference`, `TestIsVariableReference` and
  `TestSetVariableReferenceValue` in `orxmethod.cpp`. Those three paths were probed and match the
  oracle (`refs/q11_native.rex`).

### Strengths

- Step 3 fails loudly. A holder the collector does not see now panics on its next read, from the
  generation check in `RootSet::cell` (`rexx-core/src/roots.rs`), instead of reading another
  variable. My mutant run confirms it (below): the missed holder became
  `a variable's cell CellRef {..} was collected while something still held it`, not a wrong value.
- Step 2's held set comes from a scan of every live heap object (`Heap::live_objects`), not from
  roots. So any instance kept alive by anything, including pending-`UNINIT` objects, idle
  activities and subclass-of-builtin instances, keeps its behaviour.
- `cell-holders.md` commits the grep that found the holders and the mutation table. Its mutation
  table reproduced exactly on the one mutant I re-ran.
- `MethodRows::get` keeps the dense path a single bounds check, and the map is consulted only when it
  is non-empty.

### Checks run

1. **Step 3 holder hunt.** Every shape ran under collect-every-allocation on the crate, and 3 of 3 on
   the oracle. The crate matched the oracle on every shape except the two pre-existing divergences
   in Minor m1 and m2.

   | probe | shape | oracle = crate |
   |---|---|---|
   | `p01` | reference held past its frame, and a second one made after it | `p01 40 abab` / `p01b 40 42` |
   | `p02` | `>x` passed as an argument, stored by the callee, used after both returned; `use arg >z` alias written while the caller's frame lives | `p02a 30 qzq`, `p02c 26 ddd` |
   | `p03` | the same reference in an Array, a Directory and a stem tail, past the procedure | all `34 mn`, `same 1` |
   | `p04` | variable in a frame of another activity parked in `GUARD WHEN`, only `.local` and then nothing else holding the reference | `38 hhh` twice |
   | `p05` | reference passed to a started method running on another activity while the caller drops its own | `1 Y`, `22 vvv` |
   | `p06` | `PROCEDURE EXPOSE` of a referenced variable, a reference taken through `EXPOSE`, then `DROP` | `28 ppp 28 ppp`, `32 zzz 32 zzz`, `LIT 1` |
   | `p07` | `USE ARG >z` then `DROP z` then reassign, and `r = >w; drop r` | `VAR 24 ubu`, `20 waw`, `LIT` |
   | `p08` | `>st.` past its procedure, `>s.` after `drop s.`, `>t.k` (stem tail) | `Stem 30 sts 1`, `Stem S.2`, Error 20.930 on both |
   | `p09` | `>y` made inside `INTERPRET`, `INTERPRET 'call q >x'` | `24 ini`, `22 qzq`, `LIT 1` |
   | `p10` | `expose v; return >v` with the object dropped (read only) | `26 ovo` x3 |
   | `q11` | orxmethod `GetObjectVariableReference` with the object dropped; `SetVariableReferenceValue` on a `>y` cell | `36 nvn 1`, `28 svs` |
   | `q12` | 40 references made and dropped with every third kept, so cells are reused | `13 0` |
   | `q13` | a cell reference held only by a `WeakReference`, and one held strongly | `1`, `20 w2` |
   | `q14` | a cell reference in a `RAISE ... ADDITIONAL` condition object | both print nothing, rc 0 |
   | `q15` | a cell reference in a `.Message` argument array sent later, and a `~copy` of one | `26 msm`, `26 cpc Y` |
   | `q17` | an instance-variable reference passed to `USE ARG >z` with the owner dropped | `18 q7q` |
   | `r18` | a reference to a variable of a `REPLY` continuation's frame, read after the continuation writes it | `28 rpr`, `30 rqr 30` |

   With `PROBE_SWITCH=1` (switch at every opportunity), `p04`, `p05`, `q12`, `q15` and `r18` gave the
   same output.

   A new shape also crashes the oracle (Minor m3), so I ran it on the crate only. It assigns
   through an instance-variable reference whose object was dropped, then collects:
   `refs/p11_objpool_assign_oracle_segv.rex`. Oracle: SIGSEGV, rc 139, 3 of 3. Crate:
   `p10b 14 nwn`, rc 0. I never ran entry 49's shape on the oracle.

2. **Step 2 holder rule against the oracle.** Probes are in `behaviours/`, run under
   collect-every-allocation and 3 of 3 on the oracle.
   - **Retired method object held in a variable** (`t1`): `old~foo` answers `v1 aaaaaaaaaa`,
     `m1~source` keeps v1's, and `.k~define('BAR', m1)` answers v1's result through a new instance.
     These match. `t1b` differs, see the next bullet.
   - **Old instance and `instanceMethod`** (`t1b`, `t2b`): the probes diverge, and the divergence is
     **pre-existing** (Minor m4). The oracle answers `1 1` and `1 return "v1" ...`. The crate answers
     `0 0` and `0 return "v2"` on head, and also on `rexx-run-base` (`efbf0fa1d`) and
     `rexx-run-step2`, in plain runs. So `instanceMethod` on an old instance never reads its retired
     behaviour, and the holder rule cannot be judged through it.
   - **Through `.context~executable` instead** (`t5`): an old instance's retired method keeps its
     identity across 300 redefinitions and 600 fills. Oracle and crate both answer `t5 1 return w300`.
   - **Running activation on another activity** (`t4`): the activity is parked in a
     `SysSleep` poll while its class's method is redefined 50 times and collected. Oracle and crate,
     with and without `PROBE_SWITCH`, both answer
     `t4 exe 1 0 1   e0 = .context~executable~identityHash new 50`.
   - **Subclass instances** (`t3`): `t3 new old new` on both.
   - **Builtin subclasses** (`u1`): instances of Directory, Array and String subclasses survive 400
     redefinitions each plus 20 strangers' defines. `u1 d-old a-old s-old` / `u1b 0 0 0 400` on both,
     with 15625 collections on the crate.

3. **Step 1 MethodRows.** Correct as written. A row goes to the map only when `sealed` and
   `index >= dense.len()`, and the vector never grows after `seal`. So no id has rows in both halves,
   and `get`/`get_mut`/`remove` may decide by `dense.get` alone. Both halves are pruned:
   - `remove` from `forget_method_identity`;
   - `retain` from `prune_program_rows`, for `special_methods`;
   - `natives`' minted rows are filed only by `copy_method_identity` and dropped by
     `forget_method_identity`.

   The vector's dead rows are no longer popped. That is bounded, because its length is fixed at
   `seal`. Seal points: `ObjectModel::build` for natives, then `bootstrap_library` (`lib.rs`) and
   `build_object_model` for `special_methods`. The `copypriv` and `methnewpriv` witnesses in
   `memory.md` are the extent evidence. I did not re-run them.

4. **Perf.** `bench-programs/callgrind.sh -B 0.5 -j 12 -p "varlookup assign compound startup churn
   interpret" pin=<pin> head=<my build> step4a=<implementer's>` exited 0, run with `timeout 590`.
   Outputs: `/home/moritz/dev/repos/ooRexx-p9-pin/cg-9b1/t8b-review/cg6`.

   | program | head whole-process d% | step4a d% | head -libc d% |
   |---|---|---|---|
   | varlookup | +0.0031 | +0.0031 | +0.0022 |
   | assign | +0.0024 | +0.0024 | +0.0016 |
   | compound | +0.0009 | +0.0009 | +0.0033 |
   | startup | +0.3302 | +0.3305 | +0.5039 |
   | churn | +0.2336 | +0.2336 | +0.2392 |
   | interpret | +0.2935 | +0.2935 | -0.1048 |

   All are within budget and agree with the report: startup +0.33%, churn +0.23%, interpret +0.29%.

5. **Loop witnesses.** I ran `task-8/memory/memloop.sh` from a fresh directory with my head build.
   `/usr/bin/time -f %M` wraps the binary itself.

   | program | n | head | oracle |
   |---|---|---|---|
   | varref3 | 20000 / 80000 / 320000 | 47000 / 50196 / 50044 | 20928 / 20588 / 20072 |
   | define | 20000 / 80000 / 320000 | 93240 / 93236 / 92052 | 20468 / 20636 / 20324 |

   `varref3` is +6.8% at 80000 and +6.5% at 320000. `define` is flat. Both are within the 10% bar.
   At 320000 the stdout is `done` and `1`, rc 0.

6. **Mutant.** In the archive copy I replaced `VarRefHome::Cell(slot) => out.extend(slot.cell()),`
   with `VarRefHome::Cell(_slot) => {}` (`rexx-core/src/body.rs:1374`), then ran
   `collect_stress reference variable_another`. These failed:
   - `a_reference_in_a_collection_outlives_its_procedure`;
   - `a_reference_outlives_its_procedure`;
   - `a_finalized_objects_reference_keeps_its_variable`;
   - `a_variable_another_activity_references_survives`.

   `a_reference_argument_aliases_the_callers_variable` and `a_reference_outlives_its_dropped_variable`
   passed. That is exactly `cell-holders.md`'s row. The failure text is the generation panic at
   `roots.rs:360`. I reversed the mutant by the same exact edit, and
   `git show aaf302608:rust/crates/rexx-core/src/body.rs | diff - <copy>` was empty.

### Issues

#### Critical (Must Fix)

None.

#### Important (Should Fix)

None found in the task's code. m4 below is a pre-existing silent wrong answer in the area the holder
rule touches, and it needs a ruling and queueing.

#### Minor (Nice to Have)

- **m1 (pre-existing).** `.context~variables` lists a `USE ARG >z` alias.
  `refs/r16_vars.rex`: oracle `r16v Object 14 The NIL object`, so the oracle's
  directory has no `Z`. Crate `r16v String 30 vcv...`, on head and on `rexx-run-base`. This is not
  about cell lifetime. Queue it.
- **m2 (pre-existing, scheduling).** In `refs/q18_reply.rex`, after `o~release = 1` and a fill, the
  oracle has already run the `REPLY` continuation's write (`q18b 30 rqr`). The crate has not
  (`q18b 28 rpr`), on head and base. With an explicit wait (`r18`) both agree. This looks like
  `.superpowers/sdd/queued/2026-10-01-reply-continuation-state-visibility.md`. Confirm it is that
  item.
- **m3 (oracle crash, sibling of entry 49).** The crash shape is: assign through an
  instance-variable reference whose object was dropped, then collect
  (`refs/p11_objpool_assign_oracle_segv.rex`). Oracle: SIGSEGV, rc 139, 3 of 3. Crate:
  `p10b 14 nwn`, rc 0. Extend entry 49 or add entry 50, so no one runs it on the oracle.
- **m4 (pre-existing, silent wrong answer).** On an instance made before a `define`,
  `old~instanceMethod('FOO')` answers the class's current Method object, not the one `old~foo` runs.
  - `behaviours/t2_old_instance_only.rex`: oracle `t2b 1 return "v1" ...`, crate
    `t2b 0 return "v2"`.
  - `t1`: oracle `1 1`, crate `0 0`, so the identity is also wrong.
  - Both reproduce on `rexx-run-base`, so it is not Task 8b's. Queue it.
  - Until it is fixed, `instanceMethod` cannot observe Step 2's holder rule. `.context~executable`
    (`t5`) can, and it matches.
- **m5.** `ClassGraph::free_unheld_behaviours` (`rexx-classes/src/class_graph.rs`) builds a
  `std::collections::HashSet` (SipHash) over every behaviour's dict and every class's own
  dictionaries on each collection that frees a retired behaviour. It is cold, and the crate uses
  `FxHash` elsewhere. Its instruction cost on a `define` loop is unmeasured: no bench program runs
  `define`. That is not a budget breach, but nothing guards it.
- **m6.** `MethodRows`' central invariant has no assertion and no unit test of its own: a minted id
  is never below `dense.len()` once sealed. It holds by construction in `insert`. A
  `debug_assert!(index >= self.dense.len())` in the map arm of `get_mut`/`remove` would cost nothing
  in release.

### Assessment

**Task quality:** Approved

**Reasoning:** I probed every holder shape on the list under collect-every-allocation, with and
without forced switching. Each matched the oracle except two pre-existing, non-lifetime divergences.
A missed holder now panics through the generation check instead of answering a wrong variable, and
the one Step 3 mutant I re-ran went red exactly as recorded. The perf budget and both loop
witnesses reproduce. The `instanceMethod` divergence on old instances (m4) predates the task and
needs queueing.
