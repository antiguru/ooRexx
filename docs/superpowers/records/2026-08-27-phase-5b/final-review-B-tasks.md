# Phase 5b final review, strand B: the seven tasks with no independent reviewer

Reviewer B. Subject: Tasks **0, 2, 6, 7, 8, 9, 10**, read against **their own controls**. Tasks 1,
3, 4, 5 had independent reviewers and are out of scope.

Scope commit **`60256a8cc`**. Every figure below was produced by a command run in this review's own
isolated tree; nothing in this report is copied from a task report without re-measuring it.

---

## 0. Isolation, and the one measurement hazard that bit

```
mkdir -p <scratch>/reviewB/tree
cd /home/moritz/dev/repos/ooRexx-rust-rewrite && git archive 60256a8cc | tar -x -C <scratch>/reviewB/tree
export CARGO_TARGET_DIR=<scratch>/reviewB/target
```

The main worktree was read only throughout. `git status --porcelain` there is empty at the end of
this review, and every file I mutated in my own tree was restored from a `cp` backup taken before
the edit and re-checked by `sha256sum` **against the main worktree's copy**, not against my own:

```
OK    rust/crates/rexx-exec/src/dispatch.rs
OK    rust/crates/rexx-exec/src/error.rs
OK    rust/crates/rexx-exec/src/run.rs
OK    rust/crates/rexx-exec/src/builtin/state.rs
OK    rust/crates/rexx-classes/src/class_graph.rs
OK    rust/crates/rexx-exec/tests/licensed_divergences.rs
OK    rust/crates/rexx-exec/tests/coverage.rs
OK    rust/corpus/refusal-sites.tsv
OK    docs/superpowers/plans/phase-4-exclusions.txt
```

`/bin/grep -c "REVIEW B CONTROL"` answers 0 on each of the five source files I edited.

**A stale binary produced one false divergence in this review and is recorded so the figure is not
read as a finding.** After restoring `run.rs` and `dispatch.rs` from the Task 7 control I ran the
63-program `phase-5b.txt` sweep without rebuilding; it reported `lang/object_copy.rex` differing.
`cargo build` then moved `target/release/rexx-run` from `a236338a6aa50222` to
`d8c835797b98936f`, and the sweep re-run on the rebuilt binary is **63 of 63 agreeing**. The first
reading was the Task 7 mutant still on disk. Every probe figure in this report was taken on
`d8c835797b98936f` or, for the pre-mutation batches, on the binary built directly from the archive
before any edit.

Oracle probes: `( cd <fresh empty dir>; ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib
timeout -s KILL 20 .../build/bin/rexx <abs path> )`, stdout, stderr and exit status captured to three
separate files, never `2>&1`, and both `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker` on every
program. `rust/corpus/oracle-crashes.txt` was read first; no program in it and no shape from it was
constructed.

---

## 1. Verdict per task

| task | its named control | did I run it | result |
|---|---|---|---|
| 0 | removing `phase-5b.txt` from a guarded `SUBSET_FILES` reddens that binary | yes (`coverage.rs`) | **confirmed** |
| 0 | removing it from the `trace_oracle.rs` literal reddens nothing | not needed | **still a control that cannot fail** -- see finding B3 |
| 2 | C2: copying the instance behaviour on `~inherit` reddens the new witness and moves no gate row | yes | **confirmed exactly, including "no gate row moves"** |
| 6 | deliver the class's `UNINIT` from the driven collection: the DEVIATION assertion reddens | yes | **confirmed** |
| 6 | the marker check protects both directions | yes, both | **confirmed** |
| 7 | making `~copy` share the receiver's scope pools reddens `object_copy.rex` at rc 0 | yes | **confirmed, one program and no others** |
| 8 | transposing the index mapping reddens `methodsbyclass` | yes | **confirmed, and only the `order` line moves** |
| 9 | adding a site to the derived list without walking it reddens the test | yes | **confirmed** |
| 9 | `reached=yes`/`reached=no` are each enforced against the site's own identifiers | yes, both | **confirmed** |
| 10 | the flip gates: a red 5b row is an exit status with no `REXX_PHASE_GATE` | yes | **confirmed** |
| 10 | every `phase-5b.txt` program agrees on three descriptors on both engines | yes, re-swept | **63 of 63** |

No control I tested was one a wrong implementation would also have passed. The four findings below
are about what the controls do **not** reach, and one is a false statement in a control's own stated
justification.

---

## 2. Task 0 -- `corpus/phase-5b.txt` and its wiring

**State at close, derived rather than read off the report.** All six sites carry the literal:
`SUBSET_FILES` in `corpus.rs:658`, `coverage.rs:676`, `collect_stress.rs:142`, `ir_dual.rs:1187`;
the unguarded literal in `trace_oracle.rs:689`; and `EXPECTED_SUBSET_5B` at `coverage.rs:1228` with
`phase_5b_subset_matches_the_committed_list`.

`EXPECTED_SUBSET_5B` has **63** entries, `corpus/phase-5b.txt` has **63** non-comment lines, the two
lists are **equal in order**, and every entry names a file that exists on disk (checked by script,
not by eye).

**Control, run.** Removing `"phase-5b.txt"` from `coverage.rs`'s `SUBSET_FILES`:

```
REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test coverage --no-fail-fast
thread 'the_union_reads_every_phase_subset_file'         panicked at coverage.rs:709
thread 'every_in_scope_variant_is_witnessed_by_the_phase_subsets' panicked at coverage.rs:1452
test result: FAILED. 16 passed; 2 failed
```

The directory-listing guard is live, and it takes a second test with it.

---

## 3. Task 2 -- the behaviour snapshot (D58)

**Control 2, the point of the task, reproduced.** `ClassGraph::inherit_at` made to call
`copy_instance_behaviour(class)` before `update_sub_classes`:

```
REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec \
    --test corpus --test coverage --test gate_table_c --test gate_table_d --no-fail-fast
326 of 327 matching
mismatches (1):
  [UNCLASSIFIED] lang/class_behaviour_snapshot_inherit.rex: stdout, stderr, exit code differ
table C   5a: 135 rows, 0 not yet `agree`   5b: 6 rows, 0 not yet `agree`
table D   5a:  36 rows, 0 not yet `agree`   5b: 2 rows, 0 not yet `agree`
gated by this run: 0 row(s)   (both tables)
```

Exactly one corpus program reddens, it is the witness the task added, and **not one gate row in
either table changes verdict**. That is the report's claim, measured independently.

**Probing past the row.** Ten shapes around D58 measured on all three sides from fresh empty
directories; all agree byte for byte on three descriptors:

```
d58_define_after            AGREE rc=0  has 0 1
d58_delete_super            AGREE rc=0  a 1 | b 0 0
d58_inherit_sub             AGREE rc=0  a 0 | b 1 mixin-ran
d58_setmethod_then_define   AGREE rc=0  has 0 | class-has 0 | new-has 1
```

I did **not** re-run Task 2's C1, M3--M8. C2 is the mutation no existing row can see and is the one
the task exists for; C1 is the pre-task state and the report's transcript for it is the row's own
committed `control:` string.

---

## 4. Task 6 -- the licensed divergences (D59a)

**Baseline.** `cargo test --release -p rexx-exec --test licensed_divergences` exits 0, 15 tests, and
`locate()` asserts the oracle binary exists, so the two rows are measured rather than skipped.

**Control A, run.** The class-side drain of `run_termination_uninits` copied into
`Interp::run_ready_uninits`:

```
thread 'the_licensed_divergences_still_diverge_exactly_as_recorded' panicked at
  licensed_divergences.rs:191:
[class-uninit-at-driven-collection] this crate's own stdout moved on Ir
  left: "before\nclass uninit\nafter\n"
 right: "before\nafter\nclass uninit\n"
exit 101
```

Row A reddens on the crate **starting to agree with the oracle**, which is the property the row
claims.

**Row B's own liveness, run.** Task 6's Control B (building the ten-deep hold) is expensive; I ran
the cheap equivalent instead -- `builtin/state.rs`'s `gc` no longer calling `run_ready_uninits`, so
the instance's finalizer moves to termination:

```
[driven-collection-reaches-a-new-object] this crate's own stdout moved on Ir
  left: "start\nafter-gc\nuninit ran\n"
 right: "start\nuninit ran\nafter-gc\n"
exit 101
```

Row A is unmoved by that mutation and row B is unmoved by Control A, so the two rows are
independently live and neither is riding the other.

**The marker check, both directions, run.** This is the check the review plan lists as one of the
phase's eight defects (one-directional while its doc claimed both halves). It is fixed:

```
# marker deleted from phase-4-exclusions.txt
the_prose_rows_and_this_table_name_the_same_divergences  FAILED
  left: {"driven-collection-reaches-a-new-object"}
 right: {"class-uninit-at-driven-collection", "driven-collection-reaches-a-new-object"}

# row deleted from LICENSED_DIVERGENCES instead
the_prose_rows_and_this_table_name_the_same_divergences  FAILED
  left: {"class-uninit-at-driven-collection", "driven-collection-reaches-a-new-object"}
 right: {"class-uninit-at-driven-collection"}
```

**Citations spot-checked.** DEVIATION 5's "The 5b spec's Risks table names `D59's licence widens by
drift`" resolves: `specs/2026-08-27-phase-5b-instances.md:1088` carries that exact row. DEVIATION 6's
"THIS IS NOT D59a's LICENCE" paragraph is present. The two SCOPE bullet lists in DEVIATIONS 5 and 6
are **textually identical**, which is the property the rows claim so a diff shows drift; checked by
script.

---

## 5. Task 7 -- `~copy`, `~run`, `~send`/`~sendWith`, `~start`/`~startWith`

**Control 1, run.** `pool_owner` made to consult `class_variables` before its instance arm, and
`native_copy` made to record `copy -> receiver` there, so the copy's `EXPOSE` reads the receiver's
pools:

```
REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus --no-fail-fast
326 of 327 matching
mismatches (1):
  [UNCLASSIFIED] lang/object_copy.rex: stdout differ
    rust:   ... one-off one-off | receiver changed float-changed | ... | uninit changed | ...
    oracle: ... one-off one-off | receiver orig FV        | ... | uninit orig    | ...
    exit=0 on both sides
exit 101
```

The `same 0 1` and `copy-init orig float FV` lines are byte-identical under the mutation, so the
write-through lines are the ones doing the work -- the plan's claim, confirmed. One program reddens
and no others, at rc 0 with empty stderr, so the witness is the only catcher.

**The `~start` witness meets its Done-when.** `corpus/lang/object_start.rex` reads `~result` first
and `~completed`/`~hasError` after it on every one of its four messages, carries the array form's
starting-class override in the literal spelling, and no started method writes to stdout. That is
D68's bound honoured rather than described.

**Probing past the rows.** Ten shapes measured on all three sides. Everything that diverges is a
**loud** refusal at rc 120, never a silent wrong answer:

```
c_copy_name              AGREE  rc=0   named | a K
c_send_scope             AGREE  rc=0   plain sub | array base | with m2 7 8
c_start_result           AGREE  rc=0   result got 5 | completed 1 | haserror 0
c_message_new            AGREE  rc=0   Message
c_run_expose_pool        AGREE  rc=0   saw V
c_copy_writethrough      rc 120 rexx-exec: the operator `==` applied to an instance ... (Phase 5)
c_run_forms              rc 120 rexx-exec: reporting a method source that does not parse ... (Phase 5)
```

Two probes came back diverging at rc 0 and **both are already licensed**, checked rather than
assumed:

* a copy's `identityHash` compared with the original's -- oracle `1`, crate `0`. This is not
  `~copy`'s and not 5b's: `'abcdefghij'~identityHash = 'klmnopqrst'~identityHash` is oracle `1`
  against crate `0` with no 5b feature in the program at all. DEVIATION 4 licenses it and names the
  trap by name ("`~identityHash` IS ADDRESS-DERIVED", "A TRAP THIS PROJECT FELL INTO TWICE").
* `c = o~copy` / `drop c` / `call gc 'force'` -- oracle `after / uninit ran / uninit ran`, crate
  `uninit ran / after / uninit ran`. Only the finalizer's own line moves, which is exactly
  DEVIATION 6's SCOPE.

I did **not** re-run Task 7's controls 2 through 8.

---

## 6. Task 8 -- multidimensional `Array`, and `methodsbyclass` (D63)

**Control 2, run -- the one the brief said the previous probe could not express.**
`multi_dimension_position`'s subscript/dimension walk reversed (`.enumerate()` ->
`.enumerate().rev()`):

```
REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --no-fail-fast
  diverge-stdout loud=no  5b   methodsbyclass Class Library Notes  ... methodsbyclass.rex
      oracle rc=0  out="element z23 a12 a21\norder a21 a12 z23\ndimensions 2 2 3\n..."
      crate  rc=0  out="element z23 a12 a21\norder a12 a21 z23\ndimensions 2 2 3\n..."
gated by this run: 1 row(s)
exit 101
```

The `element` line is **identical** under a transposed mapping and only the `order` line moves. That
is Task 8's whole argument in one transcript, and it is the transcript its report gives -- reproduced
here on a build of my own. The row's committed `control:` string now names both mutations and
`oracle_lines` is `5`.

**Probing past the row.** Seven `Array` shapes on all three sides, all agreeing on three descriptors:

```
a_new_shapes      AGREE rc=0   0 1 1 | 2 3 | 6 0 | 0 0
a_reshape         AGREE rc=0   6 3 2 | a b
a_spread          AGREE rc=0   x | x
a_read_past       AGREE rc=0   The NIL object | The NIL object | 6
a_single_extend   AGREE rc=0   5 1 1 5
a_of_hasmethod    AGREE rc=0   1 1 1 | Array
a_dim_zero        AGREE rc=163
```

Entry 6 of `oracle-crashes.txt` is directly in this subject and no probe of mine constructed an array
with a leading empty slot as the sole subscript of `~at`.

---

## 7. Task 9 -- the instance-side reading of 5a's limits

The largest instrument in the phase, and the one I spent most of the budget on.

**Control: a site added to the source without a row.** A `Raised` constructor appended at the **end**
of `impl Raised` so no existing definition's line number moves -- the first attempt inserted it mid
file and reddened the test through line drift, which would have been a control that could not tell
me which mechanism fired:

```
cargo test --release -p rexx-exec --test refusal_sites --no-fail-fast
the_table_holds_every_constructor_the_source_defines  FAILED
  in the source, not the table:
    ("Raised", "review_b_unwalked_site", "", "crates/rexx-exec/src/error.rs:2244")
  in the table, not the source: []
exit 101
```

**Control: the `reached` column, both directions.** These are the two that separate this instrument
from the `invalid_position` shape the review plan lists. Table-only edits, no rebuild:

```
# abstract_class, answer 98.989 -> 93.907 (a neighbour's number), reached left at yes
abstract_class: reached=yes, but "93.907" is none of this constructor's own identifiers
  (its syntax(M, N) numbers are ["98.989"]) ... so the probe reached a different check

# abstract_class, reached yes -> no, answer left at 98.989
abstract_class: reached=no, but "98.989" is one of this constructor's own identifiers,
  so the probe did reach it
```

Both red. A row cannot claim a site it did not reach, and cannot disclaim one it did.

**The table at close, derived by command rather than read off the report:**

```
/bin/grep -av "^#" corpus/refusal-sites.tsv | awk -F'\t' '{print $3"|"$5"|"$6}' | sort | uniq -c
  106 body|off-send-surface|-        27 send|agrees|yes        11 body+send|agrees|yes
   10 send|diverges|yes               9 ir|off-send-surface|-    4 body+ir|off-send-surface|-
    1 send|recorded|yes               1 send|not-run|no          1 send|diverges|no
    1 send|agrees|no                  1 body+send|recorded|no    1 body+send|diverges|yes
    1 body+send|agrees|no
```

55 send-surface rows, **50 `reached=yes` against 5 `no`**. The "17 of 55 reached a different check"
the review plan names was reduced to 5 by better probes, not by relabelling: the five remaining each
carry a stated reason (two are `oracle-crashes.txt` shapes that must never be run; three say why no
route exists).

**Verdicts spot-checked against the oracle, which no test does.** Thirteen `agrees` rows re-probed
from fresh empty directories, three descriptors, both engines. **All thirteen agree**, byte for byte,
including the two the review plan flags by name:

```
r_invalid_position           AGREE rc=163   93.924 "1.5"
r_not_enough_subscripts      AGREE rc=163   93.925
r_too_many_subscripts        AGREE rc=163   93.926
r_array_too_big              AGREE rc=163   93.959
r_bad_metaclass              AGREE rc=157   99.927
r_argstring                  AGREE rc=168   r_isa5  AGREE rc=168
r_copy_not_supported         AGREE rc=163   r_scope_override_not_a_scope AGREE rc=163
r_message_array_shape        AGREE rc=163   r_message_name_shape        AGREE rc=163
r_private_method             AGREE rc=159   r_nomethod  AGREE rc=0  trapped NOMETHOD
```

`message_array_shape` is the row Task 9 caught carrying a verdict measured on a different program and
repaired; the repaired row agrees.

**The four open silent divergences the walk found, all four reproduced.** See finding B1.

---

## 8. Task 10 -- the flip

**The flip's own control, reproduced by a different route than the task's.** Task 10's A/B arms
differ only in `CLOSED_PHASES`. I used a real crate mutation instead -- Task 8's transposed index
mapping, which reddens a 5b **table C** row where Task 10's mutation reddened table D rows -- and ran
with **no `REXX_PHASE_GATE` set at all**:

```
REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --no-fail-fast
gated by this run: 1 row(s) whose owning phase is closing or closed and whose verdict is not `agree`
panicked: 1 row(s) of gate table C owned by a closing or closed phase do not `agree` ...
  ["gate-tables/concepts/methodsbyclass.rex"]
exit 101
```

`CLOSED_PHASES` is `&["5a", "5b"]` at `gate_tables/mod.rs:344`, and it is what turns a red 5b row
into an exit status in both tables. The flip gates.

**D65 criterion 2, re-swept independently.** All 63 `phase-5b.txt` programs, oracle wrapper from a
fresh empty directory per program, both crate engines, three descriptors compared raw:

```
SWEEP: 63 programs, 0 differ  (binary sha256 d8c835797b98936f)
```

**Gates, re-run in my own tree.**

| gate | status |
|---|---|
| `cargo fmt --all --check` | **0** |
| `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | 101, **one failure, environmental** |

The single failure is `rexx-bench-suite`'s `every_blocked_axis_still_fails_on_this_crate`, which
resolves `target/release/rexx-run` **relative to the crate**; my `CARGO_TARGET_DIR` is outside the
tree. With `rust/target` symlinked, `REXX_CORPUS_GATE=1 cargo test --release -p rexx-bench` exits
**0**. My first attempt had nine failing test binaries; the eight that are not `rexx-bench` were all
`cannot read .../ootest/ooRexx/base/...`, because `ootest/` and `oodocs/` are gitignored and so absent
from a `git archive` extract. With both symlinked in they are gone. The figures the run produced,
which are the ones the phase closes on:

```
mode: STRICT (the gate) -- REXX_CORPUS_GATE is set, verdicts gated for 5a, 5b
327 of 327 matching
table C   5b: 6 rows, 0 not yet `agree`
table D   5b: 2 rows, 0 not yet `agree`
104 "test result: ok" blocks
```

**The pin, and the brief's wrong claim about it.** The review plan names "Task 10's brief claimed
`dispatch.rex` had no prior baseline figure when the pinned binary runs it at rc 0" as one of the
phase's defect instances. Task 10 found it and corrected the plan and the brief in place. I verified
the correction rather than the claim:

```
sha256sum bench-baselines/pinned/rexx-run-f558ea501
  857787f797e88dd94ab839d7b66f1e08cd1a08d22c9e7978d05cbfd49f70494e   (= PINNED.md)
pinned ir           rc=0 out=5000000
pinned tree-walker  rc=0 out=5000000
head   ir           rc=0 out=5000000
head   tree-walker  rc=0 out=5000000
```

The pin runs `dispatch.rex`. So `dispatch` is a real comparison and Task 10's correction is right.

**The sitting's figures check against the committed rows.** I did not re-measure the sitting; I
recomputed every cell of its table from `bench-baselines/phase-5b-arms.tsv`:

```
awk -F'\t' 'NR>1 && $2=="a9bf7d023" && $5=="across_builds" && $8=="instructions:u"'
  dispatch  tw small 1.001383   ir small 1.002252   tw large 1.001373   ir large 1.002244
  varlookup ir large 1.006998        emptyloop tw large 0.997770
```

432 rows, all stamped `task=10 commit=a9bf7d023`, nine axes at 52 rows and `rexxcps` at 16, zero
duplicate keys. Every cell of the report's table matches to the digit, and so does its counted
observation: the `ir` arm is positive on **9 of 9** axes and the `tw` arm negative on **6**, positive
on **3** (`dispatch`, `dispatchclass`, `rexxcps`). The arm ratios that decide the `dispatchclass`
retention question are in the file as reported -- `dispatch` 0.951400/0.951178 against
`dispatchclass` 0.990930/0.990292.

---

## Findings

### B1 (highest) -- four silent wrong answers close the phase with no instrument anywhere near them, and their owner is not a phase

**Reproduced, all four, both engines, three descriptors, fresh empty directories:**

```
o~sendWith('M', .array~new(2,2))
  oracle rc=168  Error 88.913: Argument message arguments must be a single-dimensional array.
  ir / tw rc=0   stdout "a seen 0\nr\n"   stderr empty

forward message('N') arguments (.array~new(2,2))
  oracle rc=158  Error 98.946: FORWARD arguments must be a single-dimensional array of values.
  ir / tw rc=0   stdout "a seen 0\nr\n"   stderr empty

o~startWith('M', .array~new(2,2))
  oracle rc=158  Error 98.913: Unable to convert object "an Array" to a single-dimensional array value.
  ir / tw rc=0   stdout "a seen 0\nr\n"   stderr empty

self~run("...", 'A', .array~new(2,2))
  oracle rc=168  Error 88.913: Argument argument array must be a single-dimensional array.
  ir / tw rc=0   stdout "a seen 0\nr\n"   stderr empty
```

**This is 5b's own worst-defect class, arriving inside 5b.** All four were loud rc 120 refusals until
Task 8 landed `Array~new`; landing it turned them into silent wrong answers. Task 9's walk found them
and the plan records them accurately, with the C++ mechanism, which I audited:
`runtime/MethodArguments.hpp:675`-`:717` are the two `arrayArgument` overloads and both reject
`array->isMultiDimensional()`. The citation is right.

What I am reporting is not that they were missed. It is what stands over them at close:

* no gated gate-table row in either table -- necessarily so, since every 5a and 5b row `agree`s and
  a row containing this shape could not, and 5c's rows are reported rather than gated;
* no corpus program (there cannot be one -- the corpus is `327 of 327 matching`);
* no `LICENSED_DIVERGENCES` row and no DEVIATION;
* **and `refusal-sites.tsv` carries nothing about it either**, which is worth spelling out because
  the reason differs per member. Three of the four have no constructor at all to enumerate: there is
  no `syntax(88, 913` anywhere in `error.rs` (`~sendWith` and `~run 'A'`), and the only 98.913
  mentions are two unit-test tables at `error.rs:2880` and `:2925` (`~startWith`). The fourth does
  have one -- `Raised::forward_arguments`, `error.rs:2161`, whose **own doc names the missing half**:
  "The oracle tests `requestArray`'s answer for `TheNilObject` or a multi-dimensional array ..., and
  `.nil` is the reachable half here." Its table row is

  ```
  Raised  forward_arguments  body  crates/rexx-exec/src/error.rs:2161  off-send-surface  -  -  -
  ```

  which is mechanically correct under the table's own rule (`surface` is decided by which file the
  construction site is in, and `FORWARD`'s is `run.rs`), and which means the row carries `-` in
  every verdict column. The site is enumerated and nothing is asserted about it.
  `Raised::message_array_shape` is *not* this -- its 93.946 is the message-*name* array's check and
  it agrees (`r_message_array_shape AGREE rc=163` above).

So nothing in the tree reddens if this spreads to a fifth caller of `Interp::array_slots_of`, or
changes shape, or is silently half-fixed. The only record is prose in a plan file for a phase that is
now closed.

**And the owner is circular.** Every other row on Task 9's list names something that will exist:
"5c's", "Phase 7's", "whoever lands `.Method~new`". This one says "whoever owns the `Array`
argument-conversion surface" -- a surface named by the divergence itself, that no phase plan owns and
no file schedules. Compare `Loud::accessor_variable`, which the same report says "carries no owner
string by design" and then explains what would have to be built; that one is at least loud.

**Recommendation for the controller.** This is a ruling, not a fix I can make. The cheapest instrument
that would hold is a row in `LICENSED_DIVERGENCES`' shape -- crate against oracle, three descriptors,
both engines, red if either side moves -- marked as a *recorded* divergence rather than a licensed
one, or a named 5c entry with the four programs attached. The plan text alone is the state
`phase-4-exclusions.txt`'s DEVIATION 4 transcripts are in, which Task 6's own brief describes as
"prose nobody executes".

### B2 -- `refusal-sites.tsv`'s header states a false reason for its weakest link

`corpus/refusal-sites.tsv:39`:

```
# The test does not re-run a probe -- nothing in this crate can -- so `answer`
# ties the row to the source, not to the transcript.
```

**"nothing in this crate can" is false.** Twelve integration-test binaries under
`crates/rexx-exec/tests/` run the oracle through `support::oracle`:

```
bootstrap_install_oracle.rs  builtin_status.rs  corpus.rs  gate_table_c.rs  gate_table_d.rs
input_oracle.rs  ir_dual_oracle.rs  licensed_divergences.rs  native_entries.rs
oracle_deadline.rs  parse_version_oracle.rs  state_builtin_oracle.rs
```

One of them, `licensed_divergences.rs`, is a harness **this same phase built** for exactly the job
the sentence says cannot be done: run a program through the oracle and through both crate engines
from a test and assert all three descriptors.

Two adjacent copies of the same sentence are correct, which is what makes the `.tsv` one a slip
rather than a belief: `refusal_sites.rs:28` says "It does not re-run a probe ... `tests/corpus.rs` is
what sees that", and Task 9's report says "nothing in this crate reaches the oracle **from a unit
test**". The real obstacle is that the `witness` column is prose (`say .K~new over ::class K
abstract`), not a runnable program -- a good reason, and a different one.

Severity is low because the decision it justifies is right. It is reported because a reader deciding
whether to close the gap in 5c will read this line, conclude the tooling does not exist, and be
wrong.

### B3 -- the `verdict` column is the one column nothing holds against anything, and the phase already knows the check is productive

`every_send_surface_row_is_walked` checks `verdict` for membership in a four-value set.
`a_reached_row_carries_its_own_site_identifier_and_an_unreached_one_does_not` checks `answer` against
the constructor's source. **Nothing checks that an `agrees` row still agrees or a `diverges` row still
diverges.**

This is disclosed in both the file header and the report, so it is not a hidden gap. What makes it
worth a finding is that the phase has already measured the check's yield: Task 9 re-ran all twelve
`diverges` witnesses by hand at the committed tree and **one of the twelve had a verdict measured on
a different program** (`message_array_shape`) -- the `message_array_shape` instance on the review
plan's list of eight. A check with a demonstrated one-in-twelve hit rate, run once by hand and never
again, is the shape this review exists to name.

I ran the missing half by hand for the other side of the table: thirteen `agrees` rows re-probed
against the oracle on both engines, **all thirteen still agree** (section 7). So the table is
currently accurate. Nothing keeps it so.

### B4 (observation, not a defect) -- Task 0's second control arm still cannot fail at phase close

The brief asked Task 0 to record that removing `"phase-5b.txt"` from the **unguarded**
`trace_oracle.rs` literal reddens nothing, "which is the property that makes the last one worth
naming". Task 0's report says plainly that the arm was unfalsifiable at the time because the file was
empty.

**It is still unfalsifiable at close, and for a different reason.** The literal is only read by
`every_live_witness_emits_its_prefix_and_is_run_by_the_corpus`, which iterates
`Coverage::WitnessedLive` rows. There is exactly one such row --
`LIVE_INVOCATION_WITNESS = "lang/routine_dispatch.rex"` -- and it lives in a phase-4 subset. No 5b
program is a `WitnessedLive` witness, so dropping `phase-5b.txt` from that literal changes nothing
that test looks at, today or at any point during 5b.

Nothing was done wrong here; the brief predicted the outcome and the report stated it. It is recorded
because the site is the one the whole task exists to name, and it will stay unfalsifiable until some
phase's subset file carries a live trace witness -- which is worth knowing before 5c inherits the
literal.

---

## What I covered thoroughly, and what I did not

**Thoroughly** -- named control run in my own tree, plus independent probing:

* **Task 6.** Both `LICENSED_DIVERGENCES` rows shown independently live, the marker check shown live
  in both directions, citations spot-checked, DEVIATION 5/6 SCOPE lists shown textually identical.
* **Task 8.** The transposition control reproduced with its own transcript, the discriminating line
  identified, seven adjacent `Array` shapes swept.
* **Task 9.** The derived-list control run cleanly (after discarding a first attempt that reddened
  through line drift), both `reached` directions run, the table's counts re-derived, thirteen
  `agrees` verdicts re-probed against the oracle, and all four members of the open divergence
  reproduced.
* **Task 10.** The flip control run by an independent route with no `REXX_PHASE_GATE`, the 63-program
  raw sweep re-run, gates 1/2/4 re-run, the pin verified by sha256 and by running `dispatch.rex`
  through it, and every sitting cell recomputed from the committed `.tsv`.

**Partly:**

* **Task 0.** One guarded site's mutation run; the other three guarded literals were checked by
  reading, not by mutating. `EXPECTED_SUBSET_5B` verified equal to the file, in order, with every
  path existing.
* **Task 2.** C2 run, including its "no gate row moves" half. C1 and M3--M8 not run.
* **Task 7.** Control 1 run, ten adjacent shapes swept, `object_start.rex` read against its
  Done-when. Controls 2 through 8 not run.

**Not covered at all:**

* **Gate 5** (`REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace`, the debug build) was not run --
  `memcap` is present on this host, but the debug-build sweep is a multi-hour run and I spent the
  budget on mutations instead.
* **The performance sitting was not re-measured.** I verified its rows against the committed `.tsv`
  and verified the pin, which checks arithmetic and provenance, not the measurement.
* **Comment and prose quality** -- out of scope by the review plan, except where a false statement is
  a control's own justification (finding B2).
* **`body`-surface refusal sites** (106 of the 174 rows). Task 9 says plainly it did not sweep them
  for a receiver-dependent substitution and that the mechanism claim behind leaving them is unrun; I
  did not run it either, so that risk is unchanged by this review.
