# Phase 5d Task 6 — the flip, and 5d's close

**BASE `ec2d2c216`.** Tree held alone. Plan `docs/superpowers/plans/2026-09-03-phase-5d.md`
(Task 6), spec `docs/superpowers/specs/2026-09-03-phase-5d-foundation.md`.

**Committed before the gates**, per the plan's 2026-09-04 rule. The gate section below carries
placeholders and nothing else; the statuses are read from the background run's own file.

---

## The flip

`CLOSED_PHASES` gains `"5d"`. `corpus/phase-5d.txt` is created naming Phase 5d's three witness
programs and is wired into every list that names a phase subset file. `corpus/unfiled.txt` keeps its
header and names no program. Two of the three interim witness binaries are deleted; the third is
trimmed rather than deleted, for the reason below.

`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus`, at BASE and after:

```text
BASE ec2d2c216   354 of 354 matching
after the flip   357 of 357 matching
```

**The count is the side effect, not the exit status.** Both runs exit 0; a `corpus/phase-5d.txt`
created and wired into nothing would also exit 0, and would print 354. Every one of the three new
rows is a program run against the oracle on three descriptors, and the controls below show each of
them can fail.

---

## Where my enumeration disagreed with the brief, and won

The brief gave its own reading of the tree as a starting point to check. Three things came out
differently.

### 1. `tests/package_requires.rs` is not only a witness, and deleting it would delete four tests

The brief's step 4 is "each interim witness's own test binary is deleted, because the differential
now runs the program". That premise holds for `tests/operator_methods.rs` and
`tests/package_namespace.rs`, which hold one test each — the corpus program against the oracle on
both engines, plus assertions counting lines of the same `stdout` the byte comparison already
covers.

`tests/package_requires.rs` holds **five** tests, and only the first is that:

| test | what it needs | replaced by the differential |
|---|---|---|
| `the_package_requires_program_answers_the_oracle` | the corpus program | **yes** |
| `each_of_the_four_search_routes_finds_the_required_file` | five temporary directory trees, a `cwd` and `REXX_PATH`/`PATH` per row | no |
| `the_earliest_route_wins_when_every_one_of_them_holds_a_copy` | the same, with every route holding a copy | no |
| `a_requires_cycle_is_the_oracles_own_report` | a three-file cycle written into a fresh directory | no |
| `noprolog_suppresses_a_required_files_prologue_and_not_a_programs_own` | one file run twice, as a program and as a requirement | no |

The four that stay each **set a working directory and two environment variables**, which is the
whole subject of the four-route search — and a corpus program cannot express it, since the
differential runs every program the same way. So the first test goes and the binary stays, on
`tests/directive_options.rs`'s precedent: that binary was likewise kept beside `corpus/phase-5c.txt`
for what the differential does not do.

What the deletion costs, stated rather than left to be discovered: the prologue-once property loses
its **named** assertion (`"lib prologue"` appearing exactly once). It does not lose its detector —
a prologue running twice differs from the oracle on every line below it, which is the byte
comparison the differential makes. `corpus/phase-5d.txt`'s own entry says the binary is not deleted
and why.

### 2. `tests/method_bodies.rs` and `tests/unreachable_classes.rs` stay, and the plan said otherwise

The plan's Task 6 says "every interim witness from Tasks 2–5 moves into it and its binary is
deleted." Task 2 committed no interim witness: those two files are the phase's instrument. Both are
untouched, and `corpus/method-bodies.txt` was not rewritten — its sha256 is
`a9586070733fa4029a28a1cd678b3837f33a80cc1c9f8280a560bc2a1d617c9d` before and after, 1347 rows,
`loud` 661 / `answers` 677 / `diverge` 7 / `unstable` 2. The plan is corrected rather than this
report alone.

### 3. The assertion this whole ordering rests on could not see 5d

This is the finding worth more than the flip.

`gate_table_c.rs`'s `every_closed_phase_this_table_owns_rows_for_is_gated` is what the plan's
"ordering constraint" section is about: a committed `corpus/phase-<id>.txt` obliges `CLOSED_PHASES`
to name that phase, so a phase's witnesses cannot enter a subset file before it closes. **Its
enumeration is `method_row_owners()` plus `CONCEPTS` plus `WIRING_PHASE` — the owners table C's own
rows carry.** Table C owns no `5d` row; 5d's single gated row is table D's
`requires__namespace__subkeyword`. And `gate_table_d.rs` had no such assertion.

So at BASE, committing `corpus/phase-5d.txt` obliged nothing. Measured (control D1): with the file
committed and `"5d"` removed from `CLOSED_PHASES`, `gate_table_c.rs`'s assertion **passes**.

`gate_table_d.rs` now carries the twin, keyed on `owning_phase` run over the committed row set the
same way table C's is keyed on `method_row_owner`. With it, the same mutation is red:

```text
["5d"] own rows in gate table D and have a committed corpus subset file, so their programs
agree with the oracle, but CLOSED_PHASES does not name them -- a verdict of theirs can move
and every gate still exits 0
```

**This adds one test to `gate_table_d`**, so G6 and G7 report `16 passed` for that binary where
every earlier task in this phase reported `15`. A rising count is the new assertion; a falling one
would be tests not running.

### And the brief's own open question, answered

*"That check covers `corpus.rs` alone. Whether the other four lists have an equivalent is the first
thing to find out."* Enumerated from the tree and then measured, one list at a time, by removing
`phase-5d.txt` from it and running all five binaries:

| list | guard | removing `phase-5d.txt` from it |
|---|---|---|
| `corpus.rs` `SUBSET_FILES` | `the_differential_reads_every_phase_subset_file` | red, **and the headline falls to 354** |
| `coverage.rs` `SUBSET_FILES` | `the_union_reads_every_phase_subset_file` | red |
| `ir_dual.rs` `SUBSET_FILES` | `the_dual_harness_reads_every_phase_subset_file` | red |
| `collect_stress.rs` `SUBSET_FILES` | `the_stress_subset_reads_every_phase_subset_file` | red |
| `trace_oracle.rs`'s literal | **none, at `63aaeb665`** | **all five binaries green, exit 0** |

`trace_oracle.rs`'s own doc said it was the one call site the four guards do not cover and that a
future phase's file must be added by hand. The measurement above is what that sentence was worth: a
list nothing compares with the directory, and an instruction to a future reader in place of a check.
**A second commit gives it the guard** — see "The follow-up commit" below, which is where that row
changes to red.

`tests/directive_options.rs:14`'s reference to `corpus/phase-5c.txt` is a doc comment about the
`directive_options*` programs, which are still that file's. It is unaffected and was not touched.

---

## Verified by running it, in a `git archive` extract

**Tree `33387533aeaf1e87a085e6936d5fa3b0ae5d2bf0`** — `git write-tree` over the staged index,
extracted to `/home/moritz/dev/repos/claude-build-scratch/task-6/head/` with `CARGO_TARGET_DIR` at
`…/task-6/head-target`. The two deleted binaries are genuinely absent from that directory, which is
the failure mode 5c's flip had: a flip that passes only where the old files still exist on disk.

```text
$ ls head/rust/crates/rexx-exec/tests/ | grep -E 'operator_methods|package_namespace|package_requires|method_bodies|unreachable'
method_bodies.rs
package_requires.rs
unreachable_classes.rs
```

Statuses, each written unpiped as it landed
(`…/scratchpad/task-6/flip/logs/extract/status.txt`):

```text
fmt 0
clippy-clean-target 0
harnesses 101      <- see below; ir_dual alone, and environmental
g6 0
g7 0
```

**`clippy-clean-target 0` is the phase-boundary run `rust/CLAUDE.md` asks for**, which Tasks 3, 4
and 5 each declined because none of them was a phase boundary. This one is. The extract's
`CARGO_TARGET_DIR` was empty when it started, so the linter examined every crate.

**The `harnesses 101` is `ir_dual` and it is an artifact of the archive**, not of the flip:

```text
cannot read …/task-6/head/rust/crates/rexx-exec/../../../ootest/ooRexx/base/bif: No such file
or directory (os error 2)
```

`ootest/` and `oodocs/` are untracked checkouts inside this worktree, so `git archive` cannot carry
them. Symlinked into the extract and re-run, `ir_dual` is `9 passed; 0 failed`. Every other harness
was green on the first run: `corpus` `357 of 357`, `coverage`, `collect_stress`,
`directive_options`, `package_requires`, `trace_oracle`, `method_bodies` and `unreachable_classes`.

### The four expected-after numbers, each with its command

`REXX_PHASE_GATE=5d REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test
gate_table_d --no-fail-fast`, run in the extract, **exit 0**:

```text
5a: 135 rows, 0 not yet `agree`      6:  13 rows, 13 not yet `agree`
5b:   6 rows, 0 not yet `agree`      7:  94 rows, 82 not yet `agree`
5c: 1225 rows, 0 not yet `agree`     deferred-rexxcontext-stackframes: 10 rows, 10 not yet `agree`
                                     never-expected-to-agree: 5 rows, 5 not yet `agree`
gated by this run: 0 row(s)

5a: 36 rows, 0 not yet `agree`       5d: 1 rows, 0 not yet `agree`
5b:  2 rows, 0 not yet `agree`       7:  2 rows, 2 not yet `agree`
5c: 36 rows, 0 not yet `agree`       deferred-parse-error-rendering: 2 rows, 2 not yet `agree`
gated by this run: 0 row(s)
```

* **both tables `gated by this run: 0 row(s)`** — quoted above.
* **`REXX_PHASE_GATE=5d` is exit 0.** It was already 0 at BASE: Task 5 closed
  `requires__namespace__subkeyword`, and this task's reading is the second green one, not the first.
  Task 4 and earlier ran it at 101.
* **Table C still shows 110 rows open**: `13 + 82 + 10 + 5`, identical to Tasks 2–5's readings, every
  one under `6`, `7`, `deferred-rexxcontext-stackframes` or `never-expected-to-agree`, and **none**
  under a phase in `CLOSED_PHASES` — `5a`, `5b`, `5c` and `5d` all read `0 not yet agree`.
* **The method-body table is complete and unchanged**: `method_bodies` `16 passed; 0 failed` in the
  extract, and the file's sha256 is the same before and after.

Test counts, because a green from tests not running looks identical to a green from tests passing:
`gate_table_c` **15**, `gate_table_d` **16** (Task 5 measured 15 and 15).

---

## Controls

**Predictions written before the first run**, in
`…/scratchpad/task-6/flip/predictions.md`, and every one is marked below. Each mutation was applied
to the wired tree, run, and reverted — the restores are `git show :<path> > <path>` followed by
`touch`, for the reason the section after this gives.

| # | mutation | prediction | outcome |
|---|---|---|---|
| D1 | `"5d"` out of `CLOSED_PHASES`, `phase-5d.txt` committed | table C's assertion **passes** (the gap); table D's new one reddens | **CONFIRMED** both halves; `gate_table_c` `1 passed`, `gate_table_d` `1 failed` naming `["5d"]` |
| D3 | `::REQUIRES … NAMESPACE` refuses again (`directive_gap` arm restored) | `REXX_CORPUS_GATE=1` **alone** on `gate_table_d` exits non-zero; `tests/corpus` names `lang/package_namespace.rex` | **CONFIRMED**: rc 101, `gated by this run: 1 row(s)`, `5d: 1 rows, 1 not yet agree`; corpus `356 of 357`, the one mismatch `[::REQUIRES NAMESPACE] lang/package_namespace.rex` |
| D4 | `lang/operator_methods.rex` dropped from `phase-5d.txt` | `phase_5d_subset_matches_the_committed_list` and `every_lang_program_is_run_or_named_unfiled` redden | **CONFIRMED**, exactly those two, and the headline falls to `356 of 356` |
| D5 | that program in `phase-5d.txt` **and** in `unfiled.txt` | `every_lang_program_is_run_or_named_unfiled` reddens with "named by a phase subset file and by unfiled.txt" | **CONFIRMED**, that message, and only that test |
| C1 | `phase-5d.txt` out of `corpus.rs`'s list | its guard reddens | **CONFIRMED**, plus the headline back at `354 of 354` |
| C1 | out of `coverage.rs`'s list | its guard reddens, alone | **CONFIRMED** |
| C1 | out of `ir_dual.rs`'s list | its guard reddens, alone | **CONFIRMED** |
| C1 | out of `collect_stress.rs`'s list | its guard reddens, alone | **CONFIRMED** |
| C2 | out of `trace_oracle.rs`'s literal | **no** guard; all five binaries stay green | **CONFIRMED**, exit 0 — and the follow-up commit below is what turns this row red |

D3 is the one that answers "did the flip actually do anything". At BASE that program is named by no
subset file, so the differential's `354` could not have counted it whatever the crate answered;
after the flip the same mutation makes the run name it. The baseline run of the same five binaries,
with nothing mutated, is `exit 0` and `357 of 357`.

**No control weakened `every_lang_program_is_run_or_named_unfiled`**, and it did not resist: D4 and
D5 are its two arms and both went red exactly as the brief predicted they should.

---

## A revert that did not revert, and four failures that were not real

**`cp -a` restores the source file's original mtime, so cargo does not rebuild and the mutated
artifacts survive the revert.** Measured here:

```text
crates/rexx-exec/src/lib.rs        13:33:44   (mtime restored by `cp -a`)
target/release/rexx-run            14:34:17   (built during the D3 mutation)
```

Cargo compares the source's mtime against the artifact's; a source *older* than its output is up to
date, so the D3 mutation stayed linked in. Four subsequent runs of `collect_stress` reported
`the_l0_subset_passes_again_under_collect_on_every_allocation` failing with
`lang/package_namespace.rex` in the observed zero-collection set — which is exactly what a program
whose `::REQUIRES NAMESPACE` refuses at install time looks like: it never allocates.

Diagnosed by running the program through the binary directly: `rc 120`,
`rexx-exec: ::REQUIRES NAMESPACE is not implemented (Phase 5)` — a message that had been deleted from
the source twenty minutes earlier. `touch` on the source plus a rebuild makes it `rc 0`, and
`collect_stress` is `8 passed; 0 failed`. **The extract was green on `collect_stress` throughout**,
because its `CARGO_TARGET_DIR` had never seen the mutation — which is the reason the brief asks for
the extract, arriving from an unexpected direction.

Every control in the table above was **re-taken** after that, with the restore changed to
`git show :<path>` plus `touch`. The four bad readings are discarded rather than reported; the
table's rows are from the second pass, whose baseline row is green.

The project's existing note on this is about a `git status` that says nothing about `target/`. This
is a second mechanism for the same defect, and it is worse: the tree really was clean, `cmp` against
the pre-mutation copy really was silent, and the artifact was still wrong.

---

## Performance

**No file the `rexx-run` binary is built from changed**, and that is the claim rather than the
sitting. `git diff --name-only ec2d2c216 33387533a` is twelve paths, every one under
`crates/rexx-exec/tests/` or `corpus/`; **none under any `src/` directory and no Cargo manifest**. A
benchmark cannot show a behaviour change here because there is no behaviour change available to it.

The sitting was taken anyway, as the brief asks. Two builds with their own `CARGO_TARGET_DIR`, both
from `git archive` extracts — `base` of `ec2d2c216` and `head` of the staged tree — each copied to a
path no rebuild can reach:

```text
base  d5dbca2cd88e1f086b7e58bcbefbb738628bd7bcf219d6ad4bbe26da02fa2348
head  4c95cc38c9922ad5f9573dff65752d4b241552c9394efad38c1b0de76b8bf07f
```

Two distinct hashes of two identically sized files: the only difference the compiler can see is the
extract's own path, embedded because `[profile.release]` sets `debug = true`. That distinctness is
worth stating — one `CARGO_TARGET_DIR` for two revisions has given this project a spurious `1.0000x`
before.

Five rounds, both engine arms, both problem sizes, nine axes, committed as task `6` rows in
`bench-baselines/phase-5d-arms.tsv` (432 rows, the same shape Tasks 4 and 5 have). The `commit`
column holds `ec2d2c216`, the base of the comparison, per `bench-baselines/README.md`.

| axis | worst `instructions:u` cell of its four | that same cell's own round-to-round spread | `cycles:u` range over the four |
|---|---|---|---|
| `alloc4c` | +0.002% | 0.024% | -2.85% to +3.08% |
| `arith` | **-0.147%** | **0.332%** | -1.09% to -0.67% |
| `compound` | -0.001% | 0.006% | -2.03% to +1.99% |
| `dispatch` | -0.001% | 0.223% | -0.61% to +0.08% |
| `dispatchclass` | +0.002% | 0.342% | -0.59% to +2.65% |
| `emptyloop` | -0.005% | 0.007% | -1.20% to +0.58% |
| `rexxcps` | +0.001% | 0.004% | -0.22% to -0.13% |
| `strings` | -0.000% | 0.002% | -4.29% to +0.32% |
| `varlookup` | +0.001% | 0.003% | +0.47% to +4.81% |

**Nothing moved, and `arith` is the row that says so rather than the row that contradicts it.** The
brief's rule is that any instruction movement is a finding, so `arith`'s -0.147% is stated first and
then read: **it is smaller than that same cell's own spread across its five rounds.** `arith tw
large` reads `13,895,840,011 … 13,942,063,449` on the base side alone, and the head side's five
rounds sit inside that interval — `13,896,508,350 … 13,918,017,870`. The third column is that
comparison for every axis, and no axis's between-build gap reaches its own within-build spread.

That third column is the part worth keeping: `arith`, `dispatch` and `dispatchclass` are **not**
instruction-deterministic on this machine at 0.2-0.3%, where `emptyloop`, `strings`, `varlookup`,
`rexxcps` and `compound` reproduce to five or six significant figures. A future task reading a
0.1% instructions move on one of those three has read noise, and the way to tell is the axis's own
`value_min`/`value_max` columns, which are in the committed file.

`cycles:u` is the noisy column as it was for Tasks 3, 4 and 5, spanning both directions on six of
the nine axes while the instructions column spans a thousandth of a percent. Its whole range is in
the committed file.

**`rexxcps` contributes two cells where every other axis contributes four**: `rexx-arms` generated
only a `small` workload for it, so its row above is a min/max over two numbers rather than four.
Tasks 4 and 5's rows have the same shape.

**No do-nothing control was built and none could have said anything here.** The usual reason a
control is needed is to separate a change's effect from code layout; there is no change to separate,
because no source file the binary is built from differs. This sitting *is* the layout floor.

**`rexx-arms --workdir` writes its generated axis programs into that directory**, and I pointed it at
`rust/`, which put seventeen `<axis>-<size>.rex` files into the repository. They were removed by name
before the commit — `git status` is clean of them and none was staged — and the run they came from is
the one reported above. The tool's own default is a temp directory; passing `--workdir` at the repo
root is what put them there, and a later sitting should pass a scratch path instead.

---

## Committed text this change falsified, corrected rather than left

* **`crates/rexx-exec/tests/package_requires.rs`'s module doc** said "Whoever closes 5d moves the
  line into the subset file and deletes this file." The line moved and the file stays; the doc now
  says what the binary holds that the differential does not, and names the working directory and the
  two environment variables as the reason.
* **`crates/rexx-exec/tests/operator_methods.rs` and `tests/package_namespace.rs`** each carried the
  same sentence. Both files are deleted, so it is gone with them.
* **`corpus/unfiled.txt`'s three entries** each said the program is "Run by
  `crates/rexx-exec/tests/<name>.rs` until 5d closes and `corpus/phase-5d.txt` exists to name it."
  All three are removed. The file's header stays: it is the mechanism the next phase's interim
  witnesses will use, and `corpus.rs` reads it whether or not it names anything.
* **`docs/superpowers/plans/2026-09-03-phase-5d.md`, Task 6** said every interim witness's binary is
  deleted. Corrected in place with what Task 2's two files are and what `package_requires.rs` holds.
* **The same plan's "ordering constraint"** stated the `CLOSED_PHASES` obligation as a general rule.
  It reaches only the phases table C owns rows for; corrected, with a pointer to the table D twin.
* **The same plan's target table** gave table D's close as `3` rows not `agree`, naming one Phase 7
  row. Phase 7 owns two there. Corrected to `4`, with the measurement beside it.
* **`corpus/phase-5c.txt`'s header** names the five subset files that existed when it was written and
  is **not** updated. That is the convention every earlier file follows — `phase-4c.txt` names two,
  `phase-5a.txt` three, `phase-5b.txt` four — and `phase-5d.txt` follows it too. Left alone
  deliberately rather than missed.

---

## What I did not do

* **I did not delete `crates/rexx-exec/tests/package_requires.rs`**, and the section above says why.
  Its four remaining tests are unchanged; only the differential test and the module doc moved.
* **I did not touch `tests/method_bodies.rs`, `tests/unreachable_classes.rs` or
  `corpus/method-bodies.txt`.** No row moved and the table was not refreshed.
* **I did not weaken, widen or otherwise edit `every_lang_program_is_run_or_named_unfiled`**, nor any
  other assertion in `corpus.rs`. No file under any `src/` directory changed, and no Cargo manifest.
* **I added an assertion rather than only reporting the gap.** `gate_table_d.rs`'s new
  `every_closed_phase_this_table_owns_rows_for_is_gated` is the one thing in this change that is not
  bookkeeping. It is shown to fail (D1) and shown to be non-vacuous (its own `owners.is_empty()`
  guard, copied from table C's).
* **I did not check that a table D row is filed under the *right* phase.** That file's own module
  doc says nothing does, and this assertion does not change it: it checks the obligation a committed
  subset file creates, not the correctness of the filing.
* **`corpus/README.md` is untouched.** Its "Two shapes work" section already names both programs that
  take the `.cls` shape and stays true; the four `.cls` helpers are not in `phase-5d.txt`, have no
  `sourceline_oracle` expectation, and are outside every scan by design.
* **I did not re-derive `corpus/refusal-sites.tsv`** or any other generated file. `Cargo.lock` is
  unmodified and unstaged.
* **I ran no oracle probe of my own.** Every oracle run in this task came from a committed harness.
* **I did not measure `startup`, `alloc`, `heapshape` or the `bench-control` axes** — the sitting
  covers the nine `phase-5d-arms.tsv` already carries.
* **I did not investigate the seven `diverge` rows, the two `unstable` ones, or any open item below.**

---

## What Phase 5d leaves open

So the next phase inherits a list rather than a search. Nothing here is new work this task found;
each is named by the task that met it.

### Owned by a later phase

* **Phase 6.** `Alarm` 7 rows and `Ticker` 6, plus the timer entry points. And a live defect rather
  than absent work: **`REPLY` continues the method body at program end here and on a preemptable
  activity in the oracle**, a deterministic stdout-ordering divergence at rc 0 with empty stderr on
  both sides. It is why `Alarm` and `Ticker` cannot be stubbed.
* **Phase 7.** `File` 50 rows, `Stream` 24, `StreamSupplier` 8; `stream_*`, `file_*` and `qualify`;
  the stream read and write paths and `~open`'s option grammar; the stream and platform BIFs; and
  **`::REQUIRES … LIBRARY`**, which needs a native library loader and refuses in `directive_gap`
  ahead of any file search, exactly as it did at BASE.
* **The `RexxContext` work.** `StackFrame`'s 10 rows, needing a `RexxContext` method body and a
  `StackFrame` object model.

### Owned by nobody, and each a real divergence

* **`DATE()` and `TIME()` answer UTC where the oracle answers local time** — Phase 4's, found
  2026-09-04. A silent wrong answer in two of the most-used builtins, and very likely the single
  cause under six of the method-body table's seven `diverge` rows (`DateTime`'s `date`,
  `timeOfDay`, `toTimezone`, `toUtcTime`, `utcDate`, `utcIsoDate`, plus `today`). Whoever fixes it
  should re-run the method-body sweep and expect those rows to move together; if they do not, the
  offset handling is a second defect. Its witness must not be able to flake at midnight — Task 2
  built the instrument for exactly that.
* **`RootSet::promote` leaks one cell per referenced variable instance.**
* **`~unknown`'s argument list does not send `MAKEARRAY`.**
* **A `Directory` subclass's entry writes refuse where the oracle answers.**
* **`identityHash` is licensed rather than matched.**

### Named by Task 5 and deliberately left

* **`Package~findNamespace` and `~namespaces` are still `loud`**, a decision rather than an omission:
  each needs its own oracle differential and its own control, and each would move a method-body row.
* **`Package`'s other `loud` rows** — `addPackage`, `new`, `classes`, `routines`, `source`,
  `sourceLine`, `definedMethods`, `findClass`, `findRoutine`, `importedRoutines`, `loadPackage`,
  `digits`, `form`, `fuzz`, `trace` and the rest. **`rexxpg` step 3's `addPackage` half is
  unreachable** while `Package~new` is `loud`.
* **`directive_class` still has no package-local step**, unobservable rather than absent: a directive
  installs before its package's first clause, so nothing can have written to that directory by then.
  What would separate the two implementations is a route into another package's local at install
  time, and Task 5 did not find one.
* **A namespace qualifier on anything but a class reference, a call and a `::CLASS` keyword was not
  swept** against the oracle for a fifth spelling `rexx-parse` might reject.
* **`Package~local`'s interaction with the collector was not stress-tested.** No program in
  `collect_stress` sends `~local`.
* **The `REXX` namespace's public *routines* are modelled as empty rather than as a table**, checked
  on three likely candidates rather than enumerated.

### Named by Task 4 and deliberately left

* **A required file whose source does not parse is a loud refusal**, not the oracle's own report
  under the requiring `::REQUIRES` clause. Closing it is a `rexx-parse` signature change.
* **`::OPTIONS`'s other settings across a package boundary were not swept**; only `NOPROLOG` was.
* **`REXX_PATH`'s compile-time default (`ORX_REXXPATH`) is not implemented**, which changes no answer
  on this build because it is defined as the empty string.
* **A leading `~` in a required name is not expanded**, and **`require::normalize` resolves no
  symlink**, and **a `REXX_PATH` or `PATH` whose bytes are not UTF-8 drops the whole variable** —
  the last being a narrowed search, left as it is deliberately and stated rather than guarded.
* **`phase-4-exclusions.txt`'s `::OPTIONS IS REFUSED FOR THE SAME REASON` entry is stale.** 5c
  implemented `::OPTIONS` and did not mark it.

### Named by Task 3 and deliberately left

* **`String`'s operator method rows and `Class`'s** — the operator sent as an explicit message to a
  primitive receiver, `'abc'~'+'(1)` — stay `loud`. A different surface from the operator syntax
  Task 3 fixed, with an arity matrix of its own. Every one is a safe rc-120 refusal.
* **The `DO` header's three positions and the controlled-loop control variable** stay loud, because
  the oracle keeps the bound as an object and `header_number` returns a `Number`.
* **`Body::Native`, class objects and arrays** keep `Loud::operator_operand`.
* **`.DateTime~new - .TimeSpan~…` cannot print through `~string`**, because `TimeSpan~string`
  reaches `String~right`, which is unimplemented.

### About the instrument itself

* **Table D's row set has no expected-output column.** A probe rewritten to fail for an unrelated
  reason still writes a report on `stderr` and is not caught; `gate_table_d.rs`'s own module doc
  names the task that would close it.
* **The method-body table's `answers` verdict is a weaker claim than it looks** for a method needing
  arguments: both sides are sent the same wrong ones. The gated rule does not rest on it.

---

## Files

* `rust/corpus/phase-5d.txt` (new) — the three witnesses, one section per task
* `rust/corpus/unfiled.txt` — the three entries removed, header kept
* `rust/crates/rexx-exec/tests/gate_tables/mod.rs` — `CLOSED_PHASES` gains `"5d"`
* `rust/crates/rexx-exec/tests/gate_table_d.rs` — `every_closed_phase_this_table_owns_rows_for_is_gated`,
  and `CLOSED_PHASES` added to its imports
* `rust/crates/rexx-exec/tests/corpus.rs`, `coverage.rs`, `ir_dual.rs`, `collect_stress.rs`,
  `trace_oracle.rs` — `phase-5d.txt` wired into each list; `coverage.rs` also gains
  `EXPECTED_SUBSET_5D` and `phase_5d_subset_matches_the_committed_list`
* `rust/crates/rexx-exec/tests/package_requires.rs` — the differential test removed, module doc
  rewritten; the four route, cycle and `NOPROLOG` tests unchanged
* `rust/crates/rexx-exec/tests/operator_methods.rs`, `tests/package_namespace.rs` — **deleted**, each
  by an explicit `git rm` with the path named in full
* `rust/bench-baselines/phase-5d-arms.tsv` — task `6` rows
* `docs/superpowers/plans/2026-09-03-phase-5d.md` — the three corrections above
* `docs/superpowers/records/2026-09-03-phase-5d/task-6-report.md` (this file)

**Scratch left behind, not deleted:**
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/ae5f3c99-11ad-4b29-9dfe-f3d991a1a398/scratchpad/task-6/flip/`
(`predictions.md`, `extract-verify.sh`, `controls2.sh`, `logs/` with every run's three descriptors
and the two status files, `pre-control/` with the thirteen source files as they stood at BASE, and
`bench/` with the sitting's raw rows) and `/home/moritz/dev/repos/claude-build-scratch/task-6/` on
real disk (`base/` and `head/`, the two `git archive` extracts; `base-target/` and `head-target/`,
their build trees; `base-bin/` and `head-bin/`, the copied release binaries).

---

## Gates

Run from `rust/` by a background job writing each status **unpiped** to a file as it goes, with a
pidfile. Started after the commit below; the controller reads the statuses and fills this table in.

| # | command | exit |
|---|---|---|
| 1 | `cargo fmt --all --check` | **0** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| 3 | `cargo test --release --workspace --no-fail-fast` | **0** |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | **0** |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0** |
| 6 | `REXX_PHASE_GATE=5c REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **0** |
| 7 | `REXX_PHASE_GATE=5d REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **0** |

**All seven are 0, and the counts the paragraph below predicted are the counts the run produced**
-- `gate_table_c` 15, `gate_table_d` 16, both tables `gated by this run: 0 row(s)`. G7's own
per-owner breakdown, from the run's output, is what says the phase closed rather than merely passing:

```text
table C   5a: 135 rows, 0 not yet `agree`      6:  13 rows, 13 not yet `agree`
          5b:   6 rows, 0 not yet `agree`      7:  94 rows, 82 not yet `agree`
          5c: 1225 rows, 0 not yet `agree`     deferred-rexxcontext-stackframes: 10 rows, 10
                                               never-expected-to-agree: 5 rows, 5

table D   5a:  36 rows, 0 not yet `agree`      7:   2 rows, 2 not yet `agree`
          5b:   2 rows, 0 not yet `agree`      deferred-parse-error-rendering: 2 rows, 2
          5c:  36 rows, 0 not yet `agree`
          5d:   1 rows, 0 not yet `agree`
```

`13 + 82 + 10 + 5 = 110` open rows in table C, every one owned by `6`, `7`, the `StackFrame` owner or
never-agrees, and **none under a phase in `CLOSED_PHASES`** -- which is this task's expected-after,
measured rather than asserted. Table D's `5d: 1 rows, 0 not yet agree` is the row Task 5 closed,
still closed with 5d now named in `CLOSED_PHASES`, which is the direction that could have broken.

**G7 had to be 0, and the reading to compare it against is Task 5's, not Task 4's.** Task 5 was the
first green G7 this phase; Tasks 2, 3 and 4 each ran it at 101 on
`requires__namespace__subkeyword`. **The test counts are `15` for `gate_table_c` and `16` for
`gate_table_d`** — the 16th is this task's new assertion, and a 15 there would mean it did not run.
The status file's lines are written from each gate's own output rather than from an expectation.

A pre-commit run of the same two gate commands over the `git archive` extract of this tree read
`g6 0` and `g7 0`, with both tables at `gated by this run: 0 row(s)`; that run is quoted above and
is not what the table here reports.

---

# The follow-up commit: `trace_oracle.rs` gets the guard

**BASE `3c4615ecf`**, the commit that filled the gate table above. This is a second commit with a
gate run of its own, because it is test code rather than documentation.

**What the first commit shipped was a comment where a check belonged.** The measurement in "the
brief's own open question" found that `trace_oracle.rs`'s subset list is the one of five with no
directory-listing guard, and the file's response to that was a sentence asking a future task to
remember. An instruction is not a control: on this project a *dispatch warning* has failed to
prevent the very defect it warned about three times running, and a doc comment is weaker than a
dispatch warning, because nobody is required to read it at the moment it matters. One finding
earlier the same session, `gate_table_c.rs`'s assertion could not see table D and the answer was to
write the twin rather than a note — same situation, and the only thing that made this one different
is that it lives in a file whose subject is trace prefixes, which is why it was missed rather than a
reason to leave it.

## The change

* the inline literal becomes `const SUBSET_FILES`, with the measurement above in its doc;
* `phase_subset_files_on_disk()` — a fifth copy of the helper the other four binaries carry, for the
  reason each of them states: these are integration-test binaries and none can `mod` another;
* `the_prefix_table_reads_every_phase_subset_file`, which pins the one against the other;
* `every_live_witness_emits_its_prefix_and_is_run_by_the_corpus` now reads `SUBSET_FILES` rather
  than its own literal. **That substitution is half the change**: a pin on a constant nothing
  consumes would pass forever while the test kept a private copy, which is the same defect class as
  the assertion that could not see its subject;
* the "must be added by hand" sentence is gone; the explanation stays, because it says why the guard
  exists.

**The new test asserts the directory listing is non-empty before comparing it.** The non-empty
literal would have made an empty read fail anyway, but inheriting a property from the line next to
it is exactly what the `gate_table_c.rs` finding was about, and the assertion says *what happened*
where a bare `assert_eq!` prints a diff against an empty vector.

**The insertion goes above the whole doc block**, not on the `fn` line: anchoring on a `fn` silently
reassigns the doc above it to the new item and both `fmt` and `clippy` pass.

## The inversion control

**Predicted before the run**, in `…/scratchpad/task-6/flip/predictions-guard.md`, written after the
patch was prepared and before it was applied.

| # | prediction | outcome |
|---|---|---|
| P0 | `for name in SUBSET_FILES` binds `&&str` and `join` resolves it through the blanket `AsRef` impl, so it builds unchanged | **CONFIRMED** — `fmt` 0, `clippy` 0, no `for &name` or `join(*name)` needed |
| P1 | with the guard in, `trace_oracle` is exit 0 at **34** tests where the committed tree has 33 | **CONFIRMED** — `ok. 34 passed; 0 failed` |
| P2a | drop `phase-5d.txt` from `SUBSET_FILES`: `trace_oracle` reddens on `the_prefix_table_reads_every_phase_subset_file` **specifically** | **CONFIRMED** — `FAILED. 33 passed; 1 failed`, that test named |
| P2b | `corpus`, `coverage`, `ir_dual` and `collect_stress` stay green, their own lists untouched | **CONFIRMED** — 18, 20, 9 and 8 passed, 0 failed |
| P2c | no *other* `trace_oracle` test reddens, because both `Coverage::WitnessedLive` rows name `lang/routine_dispatch.rex`, which no 5d file names — so this is a pure list mismatch rather than a genuinely narrowed union | **CONFIRMED**, and it is the weaker of the two outcomes; stated in advance for that reason |

```text
assertion `left == right` failed: this file does not read every phase subset file in rust/corpus/.
  left: ["phase-4a.txt", …, "phase-5c.txt"]
 right: ["phase-4a.txt", …, "phase-5c.txt", "phase-5d.txt"]
```

**The contrast is the point.** The identical mutation at `63aaeb665` left all five binaries green at
exit 0 — that reading is in the table earlier in this report. Restored from the index with
`git show :<path>` plus a `touch`, `trace_oracle` is back to `ok. 34 passed`.

## What this commit does not do

* **No corpus program, no subset file and no `CLOSED_PHASES` entry changed.** One file.
* **The guard checks that the list matches the directory, not that a subset file's contents are
  right** — `coverage.rs`'s `phase_*_subset_matches_the_committed_list` tests are what pin contents,
  and this adds no sixth copy of those.
* **It does not make `trace_oracle.rs` read the subset files the way `corpus.rs` does.** The union
  here is still a membership set for `Coverage::WitnessedLive` paths and nothing more.

## Files

* `rust/crates/rexx-exec/tests/trace_oracle.rs` — `SUBSET_FILES`, `phase_subset_files_on_disk`,
  `the_prefix_table_reads_every_phase_subset_file`, and the doc paragraph rewritten
* `docs/superpowers/records/2026-09-03-phase-5d/task-6-report.md` — this section, plus the two
  places above where the first commit's prose said the gap was open

## Gates

Same seven commands and the same protocol: committed first, run in the background, statuses written
unpiped to `…/scratchpad/task-6/flip/logs/gates2/status.txt` with the commit sha as its first line.

| # | command | exit |
|---|---|---|
| 1 | `cargo fmt --all --check` | **0** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| 3 | `cargo test --release --workspace --no-fail-fast` | **0** |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | **0** |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0** |
| 6 | `REXX_PHASE_GATE=5c REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **0** |
| 7 | `REXX_PHASE_GATE=5d REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **0** |

**The count that moves this time is `trace_oracle`'s: 33 -> 34**, and it did. Across the two runs'
own G3 output, `33 passed` appears once at `63aaeb665` and not at all at `b5f8898e0`, where
`34 passed` appears instead -- so the new test is present and running rather than merely counted.

`gate_table_c` stayed at 15 and `gate_table_d` at 16, and G7's per-owner breakdown is identical to
the first run's line for line: table C `13 + 82 + 10 + 5 = 110` open under `6`, `7`, the
`StackFrame` owner and never-agrees, table D `5d: 1 rows, 0 not yet agree`. Neither table's row set
is touched by this commit, so identical was the expected reading and a *changed* one would have
meant the guard reached further than one file.

Filled by the controller from the second run's status file after `finished`. This is Phase 5d's last
commit.
