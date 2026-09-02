# Phase 5b final review — strand C: D-number coverage

**Subject.** `docs/superpowers/specs/2026-08-27-phase-5b-instances.md`, decisions D57–D69 plus D59a.
For each: does a committed witness exist, and does that witness fail when its subject is removed?

**Isolation.** Everything below ran in a `git archive 60256a8cc` extract at
`…/scratchpad/reviewC/tree` with `CARGO_TARGET_DIR=…/scratchpad/reviewC/target`. The main worktree
was read only. `oodocs/` and `ootest/` were **copied** (not symlinked) into the extract, because they
are git-ignored `svn` working copies and 26 tests read them.

**Restores.** Every mutation was applied by a Python replace against a saved copy and restored from
that copy, never `git checkout --`. After the last restore, each touched file was diffed against
`git show 60256a8cc:<path>` and reported `CLEAN`, and a re-run of `corpus`, `gate_table_c`,
`gate_table_d` and `licensed_divergences` under `REXX_CORPUS_GATE=1` exited **0** with
`327 of 327 matching`, `5b: 6 rows, 0 not yet agree`, `5b: 2 rows, 0 not yet agree`.

---

## The decision set, derived rather than typed

```
/bin/grep -aoE "\bD[0-9]+[a-z]?\b" docs/superpowers/specs/2026-08-27-phase-5b-instances.md | sort -u -V
  → D1 D41 D46 D53 D57 D58 D59 D59a D60 D61 D62 D63 D64 D65 D66 D67 D68 D69
```

D1/D41/D46/D53 are the parent spec's, cited here. The set this strand owns is **D57, D58, D59, D59a,
D60, D61, D62, D63, D64, D65, D66, D67, D68, D69**.

## Baselines (isolated tree, `REXX_CORPUS_GATE=1 memcap 8G`)

| command | result |
|---|---|
| `cargo test -p rexx-exec --test corpus` | `327 of 327 matching`, exit 0 |
| `cargo test -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | table C `5a: 135 rows, 0 not yet agree` / `5b: 6 rows, 0 not yet agree`; table D `5a: 36` / `5b: 2`, both 0 not yet agree; exit 0 |
| `cargo test -p rexx-exec --test licensed_divergences` | 15 passed, exit 0 |
| `cargo test -p rexx-exec --test ir_dual --no-fail-fast` | 9 passed, exit 0, 136 s |

The six 5b rows of table C are `objcla`, `abscla`, `usesem`, `creo`, `obdes`, `methodsbyclass`.

**Reviewer A's open discrepancy, answered in passing.** The grep for `"5b"` in `gate_table_d.rs`
answers 1 while the gate reports 2 rows because the assignment is one match arm covering two rows:
`("::METHOD" | "::ATTRIBUTE", "DELEGATE") => Some("5b")` (`gate_table_d.rs:250`). The gate's figure is
right; the second row is not spelled `"5b"` anywhere of its own.

---

## Per-decision verdicts

| D | witness | fails when its subject is removed? |
|---|---|---|
| D57 | **none of its own** — see finding 3 | n/a |
| D58 | `objcla.rex`, `class_behaviour_snapshot_{delete,inherit,uninherit,subclass}.rex` | **yes, both arms** |
| D59 | `licensed_divergences.rs` row `class-uninit-at-driven-collection`; `uninit_instance_retained.rex` | **yes** |
| D59a | consequence 1 as D59; 2 and 3 assigned to 5c; 4 licensed in prose | **yes** for the one 5b owes; see finding 2 |
| D60 | `obdes.rex` + five `uninit_class_*.rex`; the same licensed-divergence row | **yes** |
| D61 | prohibition — no witness is possible; satisfaction **measured** | see finding 4 |
| D62 | gate table D's `::METHOD DELEGATE` and `::ATTRIBUTE DELEGATE` rows | **yes, three separate ways** |
| D63 | `methodsbyclass.rex`, `array_multidimensional{,_refusals}.rex` | **yes, both controls the row names** |
| D64 | `phase-5b.txt` entries + `EXPECTED_SUBSET_5B` in `coverage.rs` | **yes** for drift |
| D65 | criteria 1–3 live; **criterion 4 has no failing witness** | see finding 1 |
| D66 | `setmethod_{private,restricted}_refusal.rex`, `setmethod_restricted_allowed.rex` | **yes, three separate ways** |
| D67 | `setmethod_{float,object}_scope.rex`, `object_run.rex`, `enhanced_unset.rex` | **yes, all three arms** |
| D68 | `object_start.rex`; `run/tests.rs`'s `a_started_method_that_raises_has_an_error_and_reraises_at_result` | **yes** for the assertable half; the prohibition half by inspection |
| D69 | `uninit_instance_retained.rex` | **yes, and uniquely** |

---

## Transcripts

Each mutation is a source edit in the extract, followed by the named harness. Only the reddening
lines are quoted.

### D58 — the behaviour snapshot, both arms

**Arm 1, the risk table's own named mutation** ("a live walk of the class graph"):
`dispatch.rs:1365` `Body::Instance { class, behaviour, .. } => Ok(Primitive::Instance { class, behaviour: *behaviour })`
→ read the class's *current* instance behaviour instead.

```
REXX_CORPUS_GATE=1 memcap 8G cargo test -p rexx-exec --test corpus
  325 of 327 matching
  gate-tables/concepts/objcla.rex: stdout differ
      rust:   "before 0\nafter 1\nfresh 1\n"   oracle: "before 0\nafter 0\nfresh 1\n"
  lang/class_behaviour_snapshot_delete.rex: stdout, stderr, exit code differ
  test result: FAILED   (exit 101)
```

**Arm 2, the other half the spec names** ("a copy on both families"): `class_graph.rs`'s
`inherit_at` gains `self.copy_instance_behaviour(class);` before `update_sub_classes`.

```
  326 of 327 matching
  lang/class_behaviour_snapshot_inherit.rex: stdout, stderr, exit code differ
  test result: FAILED
```

The two arms redden **disjoint** rows, which is what makes them two witnesses rather than one.

### D59 / D59a-1 / D60 — the class object's `UNINIT`

**Mutation** (the control `phase-4-exclusions.txt` DEVIATION 5 records as run): copy
`run_termination_uninits`'s class-side drain into `Interp::run_ready_uninits`.

```
cargo test -p rexx-exec --test licensed_divergences
  the_licensed_divergences_still_diverge_exactly_as_recorded ... FAILED
  [class-uninit-at-driven-collection] this crate's own stdout moved on Ir
    left: "before\nclass uninit\nafter\n"   right: "before\nafter\nclass uninit\n"
```

The recorded control reproduces exactly. This one row is the committed witness for three decisions at
once: D59's observable ("a class object was collected" is unobservable here), D59a's first
consequence, and D60's "never from a collection".

**D60's other half**, the `obdes` row's own control text ("skip the termination sweep"): delete the
class-side loop from `run_termination_uninits`.

```
  321 of 327 matching
  gate-tables/concepts/obdes.rex: stdout differ
  lang/uninit_class_inherited.rex / _mixin.rex / _inherit_runtime.rex / _sweep_order.rex / _uninherit.rex
  test result: FAILED
```

`obdes` reddens on **stdout alone**, which is the signature its `control` field claims ("the row
reddens silently, rc 0 with empty stderr on both sides, differing on stdout alone"). The control's
own wording is accurate.

### D69 — every live object with a finalizer, whether or not it was ever unreachable

Two mutations, because the coarse one does not separate D69's clause from D60's.

**Coarse** (drop the flagged-object batch from the termination sweep): `uninit_instance_retained.rex`,
`uninit_allocating_finalizer.rex`, `uninit_nested_collection_at_exit.rex` red.

**Sharp** (termination delivers only what a *collection* readies — `collect_now()` then drain
`uninit_ready`), which is precisely the "delivery driven by collection alone" D69 rules insufficient:

```
  326 of 327 matching
  lang/uninit_instance_retained.rex: stdout differ
  test result: FAILED
```

One row, and exactly the right one.

### D62 — the two `DELEGATE` gate rows

Three mutations, each reddening under `REXX_CORPUS_GATE=1 … --test gate_table_d`:

| mutation | `::METHOD` row | `::ATTRIBUTE` row |
|---|---|---|
| the delegated send reads the **method's own name** as the variable instead of the `DELEGATE` symbol | `diverge-stdout` | `diverge-both` |
| the `::ATTRIBUTE DELEGATE` **getter** is a plain accessor, the setter still delegates — *the historical defect this phase recorded* | `agree` | `diverge-stdout` |
| the **setter** is a plain accessor, the getter still delegates | `agree` | `diverge-stdout` |

`5b: 2 rows, 2 not yet agree` / `1 not yet agree`, `test result: FAILED` in each case. Both halves of
the generated pair are now load-bearing; a build with a plain getter no longer passes.

### D63 — `methodsbyclass` and multidimensional `Array`

Both mutations the row's `control` field names were run.

```
transpose the index mapping (last subscript fastest-moving)
  325 of 327 matching — methodsbyclass.rex, array_multidimensional.rex
  gate_table_c: 5b: 6 rows, 1 not yet `agree`   test result: FAILED

route `[]=` to a single-index put (`&args[1..2]`)
  323 of 327 matching — methodsbyclass.rex, array_multidimensional.rex,
                        array_multidimensional_refusals.rex, array_allocation_refused.rex
  gate_table_c: 5b: 6 rows, 1 not yet `agree`   test result: FAILED
```

The `order` line is doing the work the row's comment claims: the transposition is invisible to every
write/read pair and visible to `~toString('l', ' ')`.

### D66 — the fourth access check

| mutation | result |
|---|---|
| `check_restricted_method` always allows | 325/327 — `setmethod_restricted_refusal.rex`, `object_run_refusals.rex` |
| `check_restricted_method` always refuses | 316/327 — 11 rows, including `usesem.rex` and `setmethod_restricted_allowed.rex` |
| `Raised::restricted_method` raises `97.2` instead of `98.991` | 325/327 — `setmethod_restricted_refusal.rex` (stderr, exit code), `object_run_refusals.rex` |

The third is the one that matters for D66's actual claim, and the error identity is pinned. Read
directly from the built binary, from a fresh empty directory, both engines, three descriptors read
separately:

```
setmethod_private_refusal.rex      rc 159   stdout "a"   stderr: no method frame,
                                   Error 97.2: … cannot accept private message "SETMETHOD" …
setmethod_restricted_refusal.rex   rc 158   stdout ""    stderr opens
                                   `*-* Compiled method "SETMETHOD" with scope "Object".`,
                                   Error 98.991: Method SETMETHOD may only be invoked from …
```

identical on `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`. That is D66's sentence, byte for byte,
and the corpus differential holds both against the oracle.

### D67 — `setMethod`'s third argument

| mutation | red rows |
|---|---|
| the third argument is ignored; every one-off gets the class's pool | `setmethod_float_scope.rex`, `setmethod_object_scope.rex`, `enhanced_unset.rex`, `object_run.rex` |
| `OBJECT` resolves to the `FLOAT` pool | `setmethod_object_scope.rex` |
| `FLOAT` becomes **one pool per method** (`scope = interp.text(&name)`) | `setmethod_float_scope.rex`, `enhanced_unset.rex`, `object_copy.rex`, `object_run.rex` |

The third is the reading D67 calls out as "silently wrong at rc 0". It reddens. `setmethod_float_scope.rex`
is the only row that separates it from the correct one, exactly as its own header claims.

### D68 — `~completed`

`native_message_completed` forced to `0`:

```
  326 of 327 matching
  lang/object_start.rex: stdout differ
  test result: FAILED
```

The prohibition half ("`~start`'s interleaving is not a property any check may assert") was checked by
inspection rather than mutation — see "what I could not establish".

### D64 — the corpus subset

One line (`lang/setmethod_float_scope.rex`) deleted from `corpus/phase-5b.txt`:

```
--test corpus     326 of 326 matching        test result: ok      ← silent
--test coverage   phase_5b_subset_matches_the_committed_list ... FAILED
```

The drift pin is the *only* thing that sees it, which is what its own doc comment says. A change that
edited both the file and `EXPECTED_SUBSET_5B` would still be silent; that is inherent and documented.

### D61 — measured rather than witnessed

D61 is a prohibition on rows, so nothing can go red when it is violated. What can be measured is
whether the tree already depends on the order it forbids. `run_termination_uninits` runs the instance
group before the class group and its own doc says "no check may depend on that". Swapping the two
groups:

```
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast
  327 of 327 matching
  failure set identical to the baseline run — 26 tests, all of them
  rexx-extract / rexx-inventory / rexx-bench tests that need oodocs/ootest,
  which that pair of runs did not have. baseline-only: []  mutation-only: []
```

So the group order is genuinely unobserved, across the whole workspace and not just the corpus.

### D65 — the exit criterion, item by item

1. **every 5b row `agree`, no structural failure.** Live: proved by every gate-table mutation above.
2. **`phase-5b.txt` agrees on all three descriptors on both engines.** Live by composition:
   `corpus.rs` runs `Invocation::none()` — the **IR engine only**, since `Invocation::none` sets
   `engine: Engine::DEFAULT` and `Engine::DEFAULT = Engine::Ir` (`invocation.rs:204`, `:160`) — against
   the oracle on all three
   descriptors, and `ir_dual.rs` reads the same `SUBSET_FILES` (including `phase-5b.txt`) and holds
   tree-walker byte-identical to IR. Both green. No 5b program is exempted: `KNOWN_DIVERGENCES` holds
   two inline condition-handler programs and no corpus file.
3. **D59a's 5b-owned licensed divergence has a committed witness the harness runs, as a numbered
   DEVIATION.** Live: DEVIATION 5 in `phase-4-exclusions.txt` plus `licensed_divergences.rs`, whose
   `the_prose_rows_and_this_table_name_the_same_divergences` holds the two sets equal in both
   directions. The mutation above reddens it.
4. **`CLOSED_PHASES` gains `"5b"`.** *Not live* — finding 1.
5. **the five gates pass with commands quoted.** Task 10's record; reviewer B's strand, not measured
   here.

---

## Findings

### Finding 1 — D65 criterion 4 has no failing witness, and its absence is silent (medium)

`CLOSED_PHASES = &["5a", "5b"]` (`gate_tables/mod.rs:344`) is what makes a 5b regression red rather
than merely reported. Removing `"5b"` from it changes nothing that any check can see:

```
pub const CLOSED_PHASES: &[&str] = &["5a"];

REXX_CORPUS_GATE=1 memcap 8G cargo test -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast
  mode: STRICT (the gate) -- REXX_CORPUS_GATE is set, verdicts gated for 5a
  5b: 6 rows, 0 not yet `agree`     test result: ok
  5b: 2 rows, 0 not yet `agree`     test result: ok
```

and with a 5b row actually broken underneath it (the D62 method-name mutation), the gate still
exits 0:

```
  diverge-both    5b   ::ATTRIBUTE DELEGATE subkeyword   attribute__delegate__subkeyword.rex
  diverge-stdout  5b   ::METHOD    DELEGATE subkeyword   method__delegate__subkeyword.rex
  5b: 2 rows, 2 not yet `agree`
  test result: ok. 15 passed; 0 failed        EXIT=0
```

The only trace is the report's own `verdicts gated for 5a` line and the `2 not yet agree` tally, and
nothing asserts on either. This is the same shape `corpus.rs`'s own `SUBSET_FILES` doc comment
records for its headline ("a number a reader might eyeball is not a check") — and that one was given
a check (`the_differential_reads_every_phase_subset_file`) while this one was not.

The cheap fix has the same shape as that one: assert `CLOSED_PHASES` against something derived rather
than typed — e.g. that every phase with a `corpus/phase-*.txt` subset file whose rows are all `agree`
is in it, or simply that it contains every phase named in the two tables' `owning_phase`
assignments that the project has declared closed. I am reporting the gap, not prescribing the fix.

### Finding 2 — D59a's two 5c-owed witnesses are assigned, but the assignment lives in exactly one place (low)

The brief asks whether the consequences 5b does not owe are *recorded against the phase that owes
them* rather than merely absent. They are recorded. The record is:

* the mechanism-set table, spec line 95: "`~subclasses` and `WeakReference` observing a class's
  collection … **licensed to diverge** by D59a, and 5c owes the witnesses";
* the licence table, spec lines 618–620, with an explicit `who owes the witness` column reading
  "5c, which lands `~subclasses`" and "5c, which lands `WeakReference~new`".

So this is "assigned elsewhere", not "absent". What it is **not** is carried anywhere 5c will
necessarily look:

* `/bin/grep -arln "D59a" docs/superpowers/ rust/` names the 5b spec, the 5b plan, records under
  `records/2026-08-27-phase-5b/`, and `phase-4-exclusions.txt` — **nothing under `rust/` at all**, and
  the exclusions-file hit is a *disclaimer* at `:1163`-`:1166` ("THIS IS NOT D59a's LICENCE"), not an
  entry for it;
* the **parent** spec `2026-08-17-phase-5-object-model.md`, which a 5c spec will be written against,
  contains neither `D59a` nor `subclasses` nor `WeakReference` in this connection;
* nothing in `rust/` carries a marker. `ClassRegistry::subclasses` exists
  (`rexx-classes/src/registry.rs:384`) with no mention of the licence; no `Class~subclasses` method is
  exposed to Rexx, and `WeakReference` appears only as a name in `environment.rs:254` and
  `native_classes.rs:171`.

Both consequences are unreachable today (loud, rc 120), so there is nothing to witness yet and no
ledger entry is possible — that part is correct. The exposure is that when 5c makes them reachable,
the only thing that will point at the licence is a sentence in a table in the previous phase's spec.

The fourth consequence — the OOM asymmetry at 100,000 `~subclass` calls — is licensed **explicitly in
D59a** by decision ("licensed explicitly here rather than by reference") and deliberately has no
runnable witness. Checked: it appears nowhere in `phase-4-exclusions.txt`. Given the spec's own
wording that is the intended state, and I record it as checked rather than as a finding.

### Finding 3 — D57 has no witness of its own (informational; I do not think it needs one)

D57 says 5b is the phase in which an instance exists, over the enumerated mechanism set, taking
nothing back from 5a and deferring nothing beyond the named splits. Searched for a witness: no file
in `rust/` cites D57, and there is no check over "the mechanism set". Its content is discharged
piecewise and each piece is witnessed elsewhere:

* every mechanism it enumerates is owned by one of D58–D69 or by a table-C/table-D 5b row, and those
  are live above;
* "takes nothing back from 5a" is witnessed continuously: `"5a"` is in `CLOSED_PHASES` and both
  tables report `5a: 135 rows / 36 rows, 0 not yet agree` on every gate run, with 5a's corpus
  programs in the same union `corpus_differential` walks;
* of the spec's "Handover from 5a" list, the items with a code site are discharged in the tree —
  `receiver_kind` now answers `Ok(Primitive::Instance { … })` (`dispatch.rs:1365`) where 5a's
  `Body::Instance(_) => Err(…)` stood; `Interp::collect`'s `debug_assert!` on `pending_uninit` is
  replaced by `self.uninit_ready.extend(stats.pending_uninit)` (`lib.rs:6651`); and the `SELF`-rooting
  instrument the first item names, `run_program_collect_every_alloc`, is in use in `collect_stress.rs`.
  The fourth item is Task 9's audit and is reviewer B's.

I report it because the brief says a decision with no witness is a finding. I do not think a witness
is constructible for a pure scoping decision, and I would rule it not-a-defect.

### Finding 4 — D61's literal text is contradicted by a committed row; D60's conditional clause is what licenses it (informational)

D61 reads: "No row of any gate table, and no program in `corpus/phase-5b.txt`, may depend on the order
in which `UNINIT` runs at termination — … for class objects, because D60 has not characterised it
yet." `corpus/lang/uninit_class_sweep_order.rex` is in `phase-5b.txt` and its whole subject is that
order ("Declaration order here is C, B, A, D and the sweep runs D, A, B, C"), backed by
`rexx-classes/tests/uninit_sweep_order.rs`.

This is licensed, not a defect: D60's clause is conditional ("**until** it is characterised"), Task 5
characterised the order (bucket over the hash of the class's id string) and its report and review both
record the lapse explicitly — `task-5-report.md:192`, `:648`; `task-5-review.md:404`-`:418`
tabulates D61 against every new row. I checked the row's own claim and it holds: declaration order is
`C, B, A, D`, sweep order `D, A, B, C`, and every one of the four classes changes position.

What is left is a prose residue that this review's scope excludes: the spec's "What I could not check"
still says "the plan owes the characterisation", and D61 still says "because D60 has not characterised
it yet". A reader of D61 alone now reads a false clause. Flagging it for the controller to rule on,
not as a code defect.

---

## What I established firmly, and what I did not

**Firm.** D58, D59, D59a's 5b-owned consequence, D60, D62, D63, D66, D67, D69, and D68's assertable
half. Each has at least one committed witness and at least one mutation transcript showing it red;
D58, D62, D63, D66 and D67 have two or three mutations each, chosen so that they redden disjoint row
sets. D64's drift pin and D65's criteria 1–3 likewise.

**Established as a measurement rather than a witness.** D61. A prohibition cannot have a failing
witness; what I could show is that the tree does not currently depend on the ordering D61 forbids, for
the instance-group/class-group axis, across the whole workspace.

**Not established.**

* **D61 for instance-against-instance order.** I did not add a D61-violating row and demonstrate that
  nothing catches it. The claim I can support is only the group-order one above.
* **D68's prohibition half.** Checked by inspection, not by mutation: of the phase-5b subset, only
  `object_start.rex` and `object_send_refusals.rex` mention `~start`, the latter only in refusal
  shapes; `object_start.rex` reads `~result` before `~completed`/`~hasError` on all four messages and
  none of its started methods writes to stdout. I did not build a lazy-`~start` mutation to prove the
  interleaving is unobserved.
* **D59's "the registry stays monotone" and "`Interp::class_variables` stays a permanent root"
  literally.** I reached them only through their observables (the licensed divergence, and
  `uninit_instance_retained.rex`). Removing the global root directly is a use-after-free rather than a
  clean red, so I did not run it.
* **D65 criterion 5.** The five gates and their quoted commands are Task 10's record and reviewer B's
  strand.
* **A full green `cargo test --workspace` in my extract.** The individual binaries I needed were green;
  the whole-workspace run with `oodocs`/`ootest` present was killed for time after ~15 minutes in
  `collect_stress`. The workspace-wide comparison I do rely on (D61) used two runs in the *same*
  environment, so the 26 environmental failures cancel.

## One process note, since it produced a wrong reading before I caught it

`cp -a` preserves mtime, so restoring a mutated source from a backup can leave cargo believing the
crate is unchanged. My first "restore" of `class_graph.rs` did exactly that, and the next run —
mutating a *different* crate, which forced a rebuild — reported `class_behaviour_snapshot_inherit.rex`
red and would have been reported as a D60 consequence. The restore helper now `touch`es the file, and
the run was repeated: 6 red rows, not 7. Every transcript above is from a post-fix run.
