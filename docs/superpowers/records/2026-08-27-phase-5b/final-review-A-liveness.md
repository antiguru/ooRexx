# Phase 5b final review, strand A: witness liveness

Scope: `4c553f383..60256a8cc` (`HEAD` of the main worktree is `60256a8cc`, so
the phase end and the checkout are the same tree). Everything below was run
from `git archive 60256a8cc` extracts under
`…/scratchpad/reviewA/{tree,tree2,tree4,tree5}`, each with its own
`CARGO_TARGET_DIR`. The main worktree was read and never written. Every
restore is from a copy taken before the first mutation; no `git checkout --`
was run. At the end, `diff -rq` shows all four trees byte-identical to a fresh
`git archive 60256a8cc`, and every instrument re-runs green (see §8).

---

## 0. The gate table D discrepancy, resolved before anything was derived from it

The controller's `grep -ac '"5b"' rust/crates/rexx-exec/tests/gate_table_d.rs`
answers 1; the gate's own summary says `5b: 2 rows`. **Both are right about
different questions.** `owning_phase` assigns a phase per
`(directive, keyword)` pair through a `match`, and one arm is an or-pattern:

    gate_table_d.rs:250:        ("::METHOD" | "::ATTRIBUTE", "DELEGATE") => Some("5b"),

One source line, two rows. The rows are lines 46 and 68 of
`rust/corpus/docs/directive-options.txt` (`::ATTRIBUTE DELEGATE` and
`::METHOD DELEGATE`). Confirmed by running, not by reading: the baseline gate
table D report prints a verdict line for each,

      agree  loud=no  5b   ::ATTRIBUTE  DELEGATE  subkeyword  …  attribute__delegate__subkeyword.rex
      agree  loud=no  5b   ::METHOD     DELEGATE  subkeyword  …  method__delegate__subkeyword.rex

and ends `5b: 2 rows, 0 not yet 'agree'`. Gate table C's summary in the same
run is `5b: 6 rows, 0 not yet 'agree'`, matching the plan.

**The lesson for anyone deriving a population from this file**: a count of
source lines carrying `"5b"` is not a count of rows owned by 5b, because
`owning_phase`'s arms are or-patterns over `(directive, keyword)`. The row set
is `corpus/docs/directive-options.txt`; the phase assignment is a function over
it; the only safe denominator is the table's own printed summary, or a
re-derivation that applies the function to every row.

---

## 1. Populations, derived by command

| population | how derived | size |
|---|---|---|
| corpus programs added or touched | `git diff --name-only 4c553f383..60256a8cc -- 'rust/corpus/*.rex'` | 55 added + 3 modified = 58 |
| gate table C 5b rows | the `phase: "5b"` arms of `CONCEPTS`, cross-checked against the table's own summary | 6 |
| gate table D 5b rows | the table's own summary (§0) | 2 |
| `LICENSED_DIVERGENCES` | the `const` in `tests/licensed_divergences.rs` | 2 |
| send-surface rows of `refusal-sites.tsv` | rows whose `surface` field has a `send` tag | 55 |

**Every one of the 58 corpus programs is named by a phase subset file.** The
union of `phase-4a/4b/4c/5a/5b.txt` is 327 distinct paths; `comm -23` of the
58 against that union is empty for both the added and the modified set. 327 is
also the denominator the corpus differential prints (`327 of 327 matching`),
so nothing in the union is enumerated-but-unrun.

---

## 2. `refusal-sites.tsv`: all 55 send-surface rows are live, in both directions

Instrument:
`refusal_sites.rs::a_reached_row_carries_its_own_site_identifier_and_an_unreached_one_does_not`.
It reads both the table and `crates/rexx-exec/src` at run time, so neither
mutation below needs a rebuild; the runs drive the prebuilt test binary
directly. Baseline asserted green and a non-zero test count before every
sweep, and green again after.

**Mutation A — the row's own recorded answer.** For each of the 55 rows the
`answer` column became `ZZQQ7731X`, a token no constructor produces, one row
at a time.

* All **50** `reached=yes` rows went red **and the failure named that row**.
* All **5** `reached=no` rows stayed green, which is the correct direction for
  them: their check is `!admits`, and the token is not admitted.

**Mutation B — the other direction, for the five `reached=no` rows.** Each
row's `answer` became an identifier its own constructor really does produce:
`array_index_hole` → its own message text; `insufficient_stack` → `11.1` from
its `syntax(11, 1)`; `missing_body` → its own message text;
`package_scope_method` → `97.3`; `setup_method` → `Phase 5`, which its
`owned_message(what, Some("Phase 5"))` carries. **All five went red and named
themselves.**

**Mutation C — the constructor's own source.** For every `reached=yes` row
whose constructor body contains a `syntax(` call, `syntax(` inside that body
alone became `syntaxq(`, emptying the derived number set for that constructor.
**All 37 such rows went red and named themselves.** The other 13 `reached=yes`
rows build no `syntax(M, N)` at all — their `answer` is prose that the check
admits through the definition/sites text — so this mutation is undefined for
them; mutation A covers them.

**No send-surface row survives a mutation of its subject, and no mutation
reddened a row other than the one mutated.**

Transcript: `scratchpad/reviewA/runs/refusal_mut.log`.

### 2a. Finding A1 — two pairs of rows the check cannot tell apart

`admits` accepts a `reached=yes` row's `answer` if it is one of the
constructor's `syntax(M, N)` numbers **or** appears in its definition/sites
text. Two answers are carried by two rows each:

| answer | rows |
|---|---|
| `88.909` | `argument_needs_a_string_value` (`o~hasMethod(o)`), `named_argument_needs_a_string_value` (`self~setMethod('Z', "return 1", self)`) |
| `88.914` | `argument_not_a_class` (`o~isA(5)`), `scope_override_not_a_class` (`o~m:v with v holding 5`) |

Measured, not inferred: swapping each pair's `answer` **and** `witness`
columns leaves **all four tests in `refusal_sites.rs` green** (4 passed,
0 failed, both swaps), and the table restores green afterwards.

    swap argument_needs_a_string_value <-> named_argument_needs_a_string_value: passed=4 failed=0
    swap argument_not_a_class <-> scope_override_not_a_class: passed=4 failed=0

This is `invalid_position`'s shape exactly — a row carrying its neighbour's
real error number, with nothing distinguishing it by inspection — except that
here it is a *latent* possibility rather than a present error: each of the
four rows is correct today. **The instrument 5c inherits cannot see a
transposition inside either pair.** Everything else is discriminated: the
sweep planted every other `reached=yes` row's `answer` into every
`reached=yes` row, one plant at a time, skipping only the pairs above (whose
answers are equal, so the plant is a no-op). **No plant survived** — every
foreign answer reddened the row it was planted in.

Transcript: `scratchpad/reviewA/runs/cross_answers.log`.

---

## 3. The `sourceline_oracle` expectations (outside the assigned population, measured anyway)

The phase also added 55 files under
`rust/crates/rexx-parse/tests/sourceline_oracle/`, one per new
`corpus/lang/*.rex`. `sourceline_oracle.rs` enumerates `corpus/lang/*.rex` and
*panics* if a program has no expectation file, so none can be orphaned, and
each file's recorded lines are compared to `ProgramSource::line(n)`.

Measured: one byte appended to the first recorded source line of each of the
55 files, one file at a time. **55 of 55 went red and the failure named that
file.** Post-restore run green.

Transcript: `scratchpad/reviewA/runs/sourceline.log`.

---

## 4. Corpus programs: all 58 are live

Instrument: `REXX_CORPUS_GATE=1 memcap 8G cargo test --offline -p rexx-exec
--test corpus` (STRICT mode, so a mismatch is a non-zero exit), classified out
of the report's own `N of M matching` line rather than out of the exit status
alone. A run with no such line, or with a total of 0, is an infrastructure
failure and never a result. The unmutated baseline is `327 of 327 matching`
before the first mutation and again after the last restore.

**Method.** 56 one-site mutations were applied, one at a time, across
`dispatch.rs`, `run.rs`, `activation.rs`, `class_graph.rs` and `registry.rs`;
each `OLD` pattern was required to occur exactly once before it was applied,
so a pattern that stopped matching is a hard failure and never a skip. Of the
56, **45 diverged, 9 left the corpus at 327 of 327, and 2 were infrastructure
failures**: `methodsbyclass-transpose-index`, whose failure mode is diagnosed
in §5a and which was replaced by `multidim-transpose-offset`, and
`array-single-index-no-extend`, whose cause was not investigated because the
row it targeted (`array_multidimensional.rex`) is reddened by two other
mutations. Neither is counted as a result. Every other mutation's red set is
the set of corpus rows that stopped matching the oracle under it.

**Result: every one of the 58 corpus programs in the phase diff was reddened
by at least one mutation.** No corpus witness in the population is dead. For
57 of the 58, at least one reddening mutation is of the construct the program
is named for; the exception is `instance_self_reassigned.rex`, whose only
reddening mutation is of its setup — see finding A4.

| corpus program | mutations that reddened its row |
|---|---|
| `gate-tables/concepts/methodsbyclass.rex` | `multidim-transpose-offset` |
| `gate-tables/directives/attribute__delegate__subkeyword.rex` | `creo-skip-init`, `delegate-name-not-value` |
| `gate-tables/directives/method__delegate__subkeyword.rex` | `creo-skip-init`, `delegate-name-not-value` |
| `lang/array_allocation_refused.rex` | `array-alloc-wrong-refusal` |
| `lang/array_multidimensional_refusals.rex` | `array-subscript-count-swap`, `position-index-wrong-error`, `array-new-zero-keeps-no-shape` |
| `lang/array_multidimensional.rex` | `array-new-zero-keeps-no-shape`, `multidim-transpose-offset` |
| `lang/class_behaviour_snapshot_delete.rex` | `objcla-rebuild-behaviour`, `uninit-skip-registration` |
| `lang/class_behaviour_snapshot_inherit.rex` | `inherit-copies-behaviour` |
| `lang/class_behaviour_snapshot_subclass.rex` | `update-sub-classes-no-cascade` |
| `lang/class_behaviour_snapshot_uninherit.rex` | `uninherit-copies-behaviour` |
| `lang/delegate_no_frame.rex` | `creo-skip-init`, `delegate-name-not-value` |
| `lang/delegate_private.rex` | `creo-skip-init`, `delegate-name-not-value` |
| `lang/delegate_variable.rex` | `creo-skip-init`, `delegate-name-not-value` |
| `lang/enhanced_scope.rex` | `creo-skip-init`, `usesem-hide-own-methods`, `enhanced-drop-init-args` |
| `lang/enhanced_unset.rex` | `usesem-hide-own-methods`, `enhanced-entries-are-setmethod` |
| `lang/forward_after_reply.rex` | `forward-no-after-reply` |
| `lang/forward_arguments_converted.rex` | `forward-always-continue` |
| `lang/forward_arguments_not_an_array.rex` | `forward-always-continue` |
| `lang/forward_class_scope.rex` | `forward-no-phantom`, `forward-always-continue`, `forward-skip-scope-validate` |
| `lang/forward_class_super.rex` | `forward-always-continue`, `forward-message-default-wrong` |
| `lang/forward_class_trace.rex` | `forward-always-continue`, `forward-class-check-after-trace` |
| `lang/forward_continue.rex` | `forward-always-continue` |
| `lang/forward_frame.rex` | `creo-skip-init`, `forward-message-default-wrong` |
| `lang/forward_options.rex` | `forward-always-continue`, `forward-no-arg-trim`, `forward-drop-default-args`, `forward-message-default-wrong` |
| `lang/forward_outside_method.rex` | `forward-outside-wrong-error` |
| `lang/forward_phantom_trap.rex` | `creo-skip-init`, `forward-no-phantom` |
| `lang/instance_named_operands.rex` | `instance-name-not-stored` |
| `lang/instance_naming_overrides.rex` | `objectname-no-defaultname-send`, `instance-name-not-stored` |
| `lang/instance_naming_raises.rex` | `objectname-no-defaultname-send` |
| `lang/instance_naming.rex` | `instance-name-not-stored` |
| `lang/instance_self_reassigned.rex` | `creo-skip-init` |
| `lang/method_source_reported_name.rex` | `usesem-hide-own-methods`, `compiled-body-fixed-name` |
| `lang/object_copy_class_refusal.rex` | `class-copy-answers` |
| `lang/object_copy.rex` | `usesem-hide-own-methods`, `uninit-skip-registration`, `copy-no-uninit-registration`, `instance-name-not-stored` |
| `lang/object_run_refusals.rex` | `run-restricted-check-first`, `restricted-class-arm-always-allows` |
| `lang/object_run.rex` | `creo-skip-init`, `usesem-hide-own-methods`, `restricted-drops-class-arm`, `compiled-body-fixed-name` |
| `lang/object_send_name_refusal.rex` | `send-name-shape-wrong-error` |
| `lang/object_send_refusals.rex` | `sendwith-array-before-name`, `message-count-is-slot-count`, `message-scope-unchecked`, `send-name-shape-wrong-error`, `send-array-scope-dropped` |
| `lang/object_send.rex` | `send-array-scope-dropped` |
| `lang/object_start.rex` | `send-array-scope-dropped`, `start-drops-result` |
| `lang/setmethod_float_scope.rex` | `creo-skip-init`, `usesem-hide-own-methods` |
| `lang/setmethod_hidden.rex` | `usesem-hide-own-methods` |
| `lang/setmethod_object_scope.rex` | `creo-skip-init`, `usesem-hide-own-methods`, `setmethod-force-float-scope` |
| `lang/setmethod_precedence.rex` | `usesem-hide-own-methods` |
| `lang/setmethod_private_refusal.rex` | `natives-not-private` |
| `lang/setmethod_restricted_allowed.rex` | `usesem-hide-own-methods`, `restricted-drops-class-arm` |
| `lang/setmethod_restricted_refusal.rex` | `restricted-class-arm-always-allows` |
| `lang/setmethod_uninit.rex` | `usesem-hide-own-methods` |
| `lang/uninit_allocating_finalizer.rex` | `obdes-skip-termination-sweep`, `uninit-skip-registration` |
| `lang/uninit_class_inherited.rex` | `obdes-skip-termination-sweep`, `check-uninit-class-side-off`, `sweep-order-reversed` |
| `lang/uninit_class_inherit_runtime.rex` | `obdes-skip-termination-sweep`, `check-uninit-class-side-off`, `sweep-order-reversed` |
| `lang/uninit_class_mixin.rex` | `obdes-skip-termination-sweep`, `check-uninit-class-side-off`, `sweep-order-reversed` |
| `lang/uninit_class_sweep_order.rex` | `obdes-skip-termination-sweep`, `check-uninit-class-side-off`, `sweep-order-reversed` |
| `lang/uninit_class_uninherit.rex` | `obdes-skip-termination-sweep`, `check-uninit-class-side-off` |
| `lang/uninit_instance_collected.rex` | `uninit-skip-registration` |
| `lang/uninit_instance_retained.rex` | `creo-skip-init`, `obdes-skip-termination-sweep`, `uninit-skip-registration` |
| `lang/uninit_nested_collection_at_exit.rex` | `obdes-skip-termination-sweep`, `uninit-skip-registration`, `uninit-no-interlock` |
| `lang/uninit_nested_collection.rex` | `uninit-skip-registration`, `uninit-no-interlock` |

Transcripts: `scratchpad/reviewA/runs/batch1.log`, `runs4/batch3.log`,
`runs4/batch4a.log`, `runs4b/batch4b.log`, `runs7/batch7.log`,
`runs8/batch8.log`.


## 5. Gate tables C and D: every 5b row is reddened by its own declared control

Instrument: `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test gate_table_c
--test gate_table_d --test licensed_divergences --no-fail-fast`, with each
watched row's **verdict read out of the table's own printed report** rather
than out of the exit status. A run whose report has no verdict line for a
watched row is an infrastructure failure, never a pass; the baseline asserts
all eight watched rows read `agree` and that three `test result:` lines were
produced, and every mutated run is required to produce the same three.

| mutation (gate table C's own `control` text, as a source edit) | rows whose verdict moved |
|---|---|
| `abscla-skip-check` — drop the abstract check from `new_instance` | `abscla` → `diverge-both` |
| `creo-skip-init` — `~new` does not send `INIT` | `creo`, and both `DELEGATE` rows (their probes build instances) |
| `creo-init-wrong-arguments` — `INIT` sent with no arguments | `creo` alone |
| `objcla-rebuild-behaviour` — resolve an instance's methods from its class on every send | `objcla` alone |
| `usesem-hide-own-methods` — `own_method_entry` answers `None` | `usesem` alone |
| `obdes-skip-termination-sweep` — `run_termination_uninits` makes no pass | `obdes`, and the licensed row |
| `multidim-transpose-offset` — transpose the multidimensional index mapping | `methodsbyclass` alone (see §5a) |
| `delegate-break-setter-only` | `::ATTRIBUTE DELEGATE` **only** |
| `delegate-break-getter-only` | **both** `DELEGATE` rows |
| `delegate-name-not-value` — the delegate sends to the variable's derived name | both `DELEGATE` rows |
| `uninit-ready-runs-nothing` | no gate row; the licensed row |
| `termination-sweep-skips-classes` | `obdes`, and the licensed row |
| `uninit-skip-registration` | no gate row; the licensed row |

Two things this says beyond "the rows are live":

* **The D62 defect is closed and the closure is visible from the outside.**
  The defect was that both `::ATTRIBUTE DELEGATE` rows exercised the
  delegating setter and never the getter. Breaking only the *setter* now moves
  the `::ATTRIBUTE` row and leaves the `::METHOD` row alone — correct, the
  `::METHOD` probe has no setter — and breaking only the *getter* moves both.
  So each row is bound to the half it is supposed to cover.
* **No row was moved by a control declared for a different row**, with one
  explainable exception: `creo-skip-init` stops every `INIT`, and the two
  `DELEGATE` probes construct an instance, so they move with `creo`. That is a
  dependency, not a misfiled control.

Transcripts: `scratchpad/reviewA/runs/batch2.log` and `runs/batch2b.log`.

### 5a. Finding A2 — the `methodsbyclass` row's control, written literally, does not terminate

The row's `control` reads "route `matrix[2, 3] = 0` to a single-index `[]=`
… or **transpose the index mapping**, which the `order` line reads and no
write-then-read pair can". Spelled the obvious way — `subscripts.iter()
.rev().zip(dimensions)` in `multi_dimension_position` — the mutation does not
terminate. Measured directly, `rexx-run` on `methodsbyclass.rex` from a fresh
empty directory under that build:

    rc 134, stdout empty, stderr:
      thread 'rexx-interp' has overflowed its stack
      fatal runtime error: stack overflow, aborting

The reversed pair puts a subscript past its dimension, which takes the
`IndexUse::Put` arm into `array_extend_multi` and back into
`multi_dimension_position` with the *unreversed* subscripts. Under the two
harnesses this reads as an infrastructure failure, not a catch: the corpus run
prints no `N of M matching` line at all, and the gate-table run starts 2 test
binaries where the baseline starts 3.

The control **is** runnable in a form that transposes only the offset
accumulation and leaves the bounds pass exactly as it was
(`multidim-transpose-offset`), and in that form it reddens `methodsbyclass`
and nothing else in gate table C, plus `array_multidimensional.rex` in the
corpus. So the row is live and its control is right about *what* to perturb;
what the control text does not say is that the obvious spelling of it hangs.
Whoever runs this control next needs that sentence, or they will read an
infrastructure failure as a catch.

## 6. `LICENSED_DIVERGENCES`: both rows are individually live

Each row was reddened by a mutation of the finalizer timing its own SCOPE
names, and the assertion message names that row:

* `class-uninit-at-driven-collection`, under `termination-sweep-skips-classes`
  (the sweep runs the flagged instances and skips the class group):

      [class-uninit-at-driven-collection] this crate's own stdout moved on Ir
        left: "before\nafter\n"   right: "before\nafter\nclass uninit\n"

* `driven-collection-reaches-a-new-object`, under `uninit-ready-runs-nothing`
  (`run_ready_uninits` returns immediately):

      [driven-collection-reaches-a-new-object] this crate's own stdout moved on Ir
        left: "start\nafter-gc\nuninit ran\n"   right: "start\nuninit ran\nafter-gc\n"

Both runs are `14 passed; 1 failed`, so no other test in the binary moved with
them, and the tree restores to `15 passed; 0 failed`.

## 7. Findings

**Nothing in the assigned population is a dead witness.** All 58 corpus
programs, all 6 gate table C 5b rows, both gate table D 5b rows, both
`LICENSED_DIVERGENCES` rows and all 55 send-surface `refusal-sites.tsv` rows
reddened under a mutation, and wherever the instrument names a row, the row it
named was the one mutated. A4 is the one place where the reddening mutation is
not of the property the witness is filed under. The findings below are about
instruments that cannot see a particular perturbation, not about witnesses
that never fire.

The nine mutations that left the corpus at `327 of 327` are accounted for in
full: two are A3's rooting pair, five are A5's table, and two are caught
outside the corpus (last subsection).

### A1 — `refusal-sites.tsv`: two pairs of rows the `reached` check cannot tell apart

See §2a for the transcript. `argument_needs_a_string_value` /
`named_argument_needs_a_string_value` both record `88.909`, and
`argument_not_a_class` / `scope_override_not_a_class` both record `88.914`;
transposing either pair's `answer` **and** `witness` leaves all four tests in
`refusal_sites.rs` green. Both rows of each pair are correct today, so this is
a hole in the instrument rather than a wrong row — but it is precisely
`invalid_position`'s shape, and 5c inherits this file. Every other ordered
pair is discriminated.

### A2 — the `methodsbyclass` control aborts in its obvious spelling

See §5a. The row is live; the control text needs the sentence that the naive
`.rev()` re-enters the growth path until the interpreter thread overflows its
stack, because an infrastructure failure and a catch are not distinguishable
from a non-zero exit status alone.

### A3 — `Activation`'s GC root walk is not witnessed by anything in the workspace

`Activation`'s root walk pushes the running method's `receiver`
(`activation.rs:1351`) and, for each exposed variable, that variable's `owner`
(`activation.rs:1362`). The mutations below delete exactly those two lines and
leave the two neighbouring `out.push(*scope)` calls (`:1350`, `:1363`) alone.
Measured on a `git archive` extract, with `--no-fail-fast` and the binary
count asserted equal:

| run | binaries | passed | failed | distinct failing tests |
|---|---|---|---|---|
| unmutated baseline | 104 | 1910 | 31 | 26 |
| **both lines deleted** | 104 | 1910 | 31 | the same 26 |

The failing-set difference is empty in both directions. The corpus
differential is `327 of 327 matching` under `out.push(*receiver)` deleted,
under `out.push(*owner)` deleted, and under both deleted at once.

The 26 baseline failures are environmental — an extract has no `.git`, so the
revision-stamp and extractor-re-derivation tests cannot pass — which is why
the claim is a *set difference against a baseline measured the same way*
rather than "the suite is green".

I am reporting the observable and not a diagnosis: **no test in this workspace
distinguishes a build that roots those two references from one that does
not.** Whether that is safe (the objects reachable another way in every
committed program) or a latent use-after-collection is a question for whoever
owns the collector; it is not something this review can settle, and I did not
try to construct a program that crashes without them.

### A4 — `instance_self_reassigned.rex` does not witness the property it is filed under

`corpus/phase-5b.txt` introduces it as "a method body assigning over `SELF`
and then allocating, **the shape the receiver's rooting turns on**". It is a
live witness — `creo-skip-init` (`~new` does not send `INIT`) reddens it — but
that is its *setup*, not its stated subject, and it is the only one of the 58
whose sole reddening mutation is of a construct it does not name. Under A3's
mutations it stays green, so the sentence in `phase-5b.txt` claims a coverage
the row does not have.

### A5 — five 5b behaviours no test in this workspace distinguishes

Each row below is a one-site mutation that left the corpus differential at
`327 of 327 matching` **and** left the workspace's failing set identical to
the baseline's, under `--no-fail-fast`, with the binary count equal (104 in
every run). None is a dead witness; each is a behaviour with no witness.

| mutation | what it changes | why the nearest row cannot see it |
|---|---|---|
| `setmethod-restricted-before-scope` | `native_set_method` runs `check_restricted_method` before `set_method_scope` | the two corpus rows that reach the restricted check (`setmethod_restricted_refusal.rex`, `setmethod_restricted_allowed.rex`) pass no third argument, so no option is read either way. `native_run`'s analogous order **is** witnessed — `object_run_refusals.rex` pins it and `run-restricted-check-first` reddens that row — so this is an asymmetry between sibling checks |
| `startwith-wrong-missing-position` | `native_start_with`'s `missing_method_argument(2)` becomes `(3)` | `object_send_refusals.rex` traps every case and prints `rc`, the error number and the sub-number, never the message text; the position is a substitution inside the message. The row's own claim — `startWith` reports the missing argument where `sendWith` reports the name's `93.972` — is intact and witnessed |
| `class-copy-allowed` | `native_copy`'s non-instance arm answers the receiver instead of `Loud::native_method(b"COPY", "Object")` | `object_copy_class_refusal.rex` reaches `native_class_copy`, the refusal `Setup.cpp` installs *over* `Object~copy`, so a class receiver never arrives at this arm; and no committed program sends `~copy` to a string or an array |
| `start-no-scope-validate` | `started_message` no longer calls `validate_scope_override` | `object_start.rex` starts no message with a scope the receiver is not an instance of, and `object_send_refusals.rex`'s `~start`/`~startWith` cases fail earlier, in the name decode |
| `restricted-allows-no-receiver` | `check_restricted_method` answers `Ok(())` for a caller with no receiver, instead of `Raised::restricted_method` | this is what the code's own comment beside it predicts: "the private check refuses first", and `setmethod_private_refusal.rex` measures exactly that — 97.2 at dispatch, before the body. So the arm may have no reachable caller at all, which is a different thing from being untested, and separating the two needs a program this review did not build |

### Not findings: mutations the corpus missed and something else caught

Two mutations survived the corpus and were caught elsewhere, which is the
right outcome for both:

* `array-new-no-subclass-refusal` (never raise `Loud::array_subclass_new`) —
  caught by `dispatch::tests::new_on_a_subclass_of_array_is_loud`. No corpus
  row could carry it: it is a refusal where the oracle answers, so it is a
  divergence, and its other witness is the `array_subclass_new` row of
  `refusal-sites.tsv`, which §2 shows live.
* `check-uninit-instance-off` (the instance-side arm of
  `ClassGraph::check_uninit`) — caught by
  `tests::the_uninit_flags_are_set_for_the_classes_a_file_declares` in
  `rexx-exec` and by
  `uninit_propagates_through_all_three_constructors_at_the_graph_api` in
  `rexx-classes`.

### What this strand did not cover

* **The 5a and 5c rows of both gate tables, and the pre-5b corpus.** Only the
  5b rows and the phase diff's own programs were mutated. Where a 5b mutation
  reddened an older row (`abscla.rex`, `creo.rex`, `obdes.rex`, `objcla.rex`,
  `usesem.rex` are pre-existing files that 5b added to `phase-5b.txt`), that
  is recorded above but their liveness was not the target.
* **`ir_dual`.** The corpus differential runs `Invocation::none()`, which is
  the IR engine, so every corpus result above is about the compiled engine.
  The tree-walker is pinned by `ir_dual.rs`, which this strand did not mutate.
* **Two 5b-touched checks beside the assigned population.** `gate_table_d.rs`'s
  `ORACLE_REFUSES` list and its `expected_oracle_lines` guard were not mutated
  (neither `DELEGATE` row is named in that list, so no 5b row exercises it),
  and neither was `coverage.rs`'s `EXPECTED_SUBSET_5B`, which Task 0 added
  beside `EXPECTED_SUBSET_5A`.
* **No new probe was run against the C++ oracle.** Every oracle invocation
  here came from a committed harness running committed programs, so no shape
  from `corpus/oracle-crashes.txt` was constructed or run. The one probe this
  strand ran directly (§5a) was `rexx-run` on a committed corpus program, from
  a fresh empty directory, with no oracle involved.
* **`unsafe` was not used anywhere**, and nothing was committed.

## 8. Controls, and one harness defect this review had to correct

**The first three campaigns' post-restore controls failed, and the cause was
the harness rather than the tree.** `shutil.copytree` preserves the backup's
mtimes, which are *older* than the artifacts cargo built from the mutated
copy, so cargo's fingerprint reads the restored tree as up to date and keeps
the mutated rlib. Diagnosed rather than assumed: after each of those campaigns
`diff -rq` against a never-mutated `git archive` extract reports **no
differences at all**, and a forced rebuild (`touch` every `.rs`, then rebuild)
puts every instrument back to green —

* batch 1 tree: `327 of 327 matching`
* batch 3 tree: `327 of 327 matching`
* batch 2 tree: gate table C `14 passed; 0 failed`, gate table D `15 passed;
  0 failed`, licensed divergences `15 passed; 0 failed`

The project's own committed `rust/scripts/mutate-4*.sh` do **not** have this
defect: their `restore` is `cp` without `-p`, which stamps the current time.
This is a note about this review's harness, not a finding against the tree.

**One consequence had to be corrected rather than noted.** Batch 3's rows
after the first `rexx-classes` mutation ran against a `rexx-classes` rlib
still carrying `sweep-order-reversed`, because every later row mutated only
`rexx-exec` and so never forced `rexx-classes` to rebuild. Those rows were
re-run with a restore that stamps the current time on every file, and §4 uses
the re-run. The re-run changed three verdicts — `restricted-allows-no-receiver`,
`start-no-scope-validate` and `class-copy-allowed` each read as reddening four
`uninit_class_*` rows in the contaminated batch and read as `327 of 327` when
run clean — which is why the contaminated rows are excluded rather than
merely annotated.

**Every campaign after the fix passed its own post-restore control** —
batches 4a, 4b, 7 and 8 all end `post-restore baseline: PASSED 327/327` — and
at the end of the review `diff -rq` shows all four extracts byte-identical to
a fresh `git archive 60256a8cc`.
