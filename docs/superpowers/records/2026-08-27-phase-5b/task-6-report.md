# Task 6 report: the licensed divergences (D59a)

**Status: COMPLETE.** Every figure is quoted beside the command that produced it; a line with no
command beside it has not been run.

**BASE:** `6eb234598`, tree clean at start.

## Log

* Read the brief, `global-constraints.md`, the 5a constraints, `rust/CLAUDE.md`, the plan's Task 6
  section and the spec's D59/D59a. Read `rust/corpus/oracle-crashes.txt` before writing any probe.

## Re-measurement of the brief's claims

Every run below used the standard oracle wrapper from a fresh empty directory per run, with the
three descriptors read separately, and the crate side under `timeout -s KILL 20` on both engines.
The two drivers are in the scratchpad:
`scratchpad/task6/t6-probe.sh` and `scratchpad/task6/t6-summary.sh`, both of which do

```
( cd "$D" && ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  timeout -s KILL 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$D/p.rex" \
  >"$D/out" 2>"$D/err"; echo $? >"$D/rc" )
```

for the oracle and

```
( cd "$D" && REXX_ENGINE=$SIDE timeout -s KILL 20 \
  /home/moritz/dev/repos/ooRexx-rust-rewrite/rust/target/release/rexx-run "$D/p.rex" \
  >"$D/out" 2>"$D/err"; echo $? >"$D/rc" )
```

for the crate, with `$SIDE` each of `ir` and `tree-walker`.

### Precondition (brief's claim that Task 5's delivery 3 landed) -- HOLDS

`/bin/grep -n 'run_termination_uninits' rust/crates/rexx-exec/src/dispatch.rs rust/crates/rexx-exec/src/lib.rs`
prints six lines, four of them doc-comment or test references. The two the brief names are among
them, at the line numbers it gives:
`dispatch.rs:2641:    pub(crate) fn run_termination_uninits(&mut self) -> Vec<Loud> {` and
`lib.rs:7071:    for loud in interp.run_termination_uninits() {`.

### Divergence A -- REPRODUCES, five runs each side

`t6-probe.sh scratchpad/task6/progs/divA.rex 5`. rc 0 and zero stderr bytes on all fifteen runs.

```
oracle       (5/5)  before / class uninit / after
ir           (5/5)  before / after / class uninit
tree-walker  (5/5)  before / after / class uninit
```

Checked against `rust/corpus/oracle-crashes.txt` entry 8 before running: that entry is a metaclass
whose `NEW` answers a non-class. This program defines no `NEW`.

### Divergence B -- REPRODUCES, five runs each side, threshold at N=9

`t6-summary.sh scratchpad/task6/progs/divB-n<N>.rex 5` for N in 0, 1, 2, 8, 9, 10. rc 0 and zero
stderr bytes on all ninety runs.

```
N        oracle (5/5)                      ir and tree-walker (5/5 each)
0,1,2,8  start / after-gc / uninit ran     start / uninit ran / after-gc
9,10     start / uninit ran / after-gc     start / uninit ran / after-gc
```

### The window counts allocations, not clauses -- CONFIRMED, and twice over

`t6-summary.sh scratchpad/task6/progs/divBc-n<N>.rex 5`, padding clauses `z<i> = 'p'||'q'||'r'||'s'`,
oracle side, rc 0 and zero stderr bytes on all twenty runs:

```
N=1  start / after-gc / uninit ran      N=3  start / uninit ran / after-gc
N=2  start / after-gc / uninit ran      N=4  start / uninit ran / after-gc
```

Three concatenating clauses reach the same threshold nine plain assignments do, which is three
allocations per clause and nine either way.

The plan's second control holds too: `t6-summary.sh scratchpad/task6/progs/divBnop-n9.rex 3` and
`...-n20.rex 3` are `start / after-gc / uninit ran` on the oracle at both counts, three runs each,
so twenty `nop` clauses do not move the threshold at all.

### C++ citations, re-read in the read-only tree

* `interpreter/memory/Memory.hpp:107` -- `static const size_t SaveStackSize = 10;`
* `interpreter/memory/RexxMemory.cpp:306` -- `void MemoryObject::collectAndUninit(bool clearStack)`,
  whose body clears the save stack only when `clearStack` is true.
* `interpreter/memory/RexxMemory.cpp:1053` -- `pushSaveStack(newObj);`, in `newObject`, so the push
  is per allocation.
* `interpreter/expression/BuiltinFunctions.cpp:3033` -- `memoryObject.collectAndUninit(false);
  // keep stack`, which is `GC`'s own call.

## The measurement that decides Divergence B

The controller's recommendation is to license, on the reasoning that a ten-deep hold cannot buy
agreement because the oracle's window counts *its own* allocations. That reasoning is right, and
what follows is the measurement rather than the argument, because the argument alone would have been
a mechanism claim about two allocators.

**This section records why the hold is not built, and it was written before Moritz ruled.** It is no
longer the reason the difference is *allowed* -- see "Moritz's ruling" below, which replaced that
half of the argument with the language's documented contract and demoted everything here to a
supporting fact. Nothing measured here was withdrawn.

**The instrument.** `rexx_exec::run_program_collect_every_alloc` collects on every allocation and
`Outcome::collections` counts collections during the program, so under that entry point
`collections` **is** the program's arena allocation count. It is reached from a throwaway crate
outside this repository -- `scratchpad/task6/alloccount/` with a path dependency on `rexx-exec` and
its own `CARGO_TARGET_DIR` -- so nothing in the shared tree was mutated to take these numbers.

**The instrument is live, proved by inverting it.** Same binary, same command
(`scratchpad/task6/alloccount/target/release/t6-alloccount <programs>`):

```
say 'x'                                              allocations=0
do i = 1 to 1  ; z = .K~new ; end ; say 'x'          allocations=1
do i = 1 to 5  ; z = .K~new ; end ; say 'x'          allocations=5
do i = 1 to 20 ; z = .K~new ; end ; say 'x'          allocations=20
```

so it tracks allocation one for one and reads zero when there is none.

**What the padding clauses cost on each side.** Same command over the probe programs:

```
divB-n0 .. divB-n10   (0, 1, 2, 3, 4, 8, 9, 10 plain padding clauses)   allocations=4, every one
divBc-n1 .. divBc-n4  (1, 2, 3, 4 concatenating padding clauses)        allocations=4, every one
do i = 1 to 50 ; z = 'p'||'q'||'r'||'s' ; end ; say 'x'                 allocations=0
```

The oracle's window advances one slot per plain padding clause and three per concatenating one;
**this crate's arena allocation count does not move at all for either.** Both padding shapes are a
literal the compiled stream already interned and, for the concatenating one, a result short enough
to live inline in the value.

**So a hold would move this divergence rather than close it, and the corpus program is where it
would land.** `corpus/lang/uninit_instance_collected.rex` is on the *agreeing* side today. Its
allocation profile, by the same command over a four-step prefix series
(`scratchpad/task6/progs/d1.rex` .. `d4.rex`):

```
o = .K~new                                                      allocations=1
+ say 'built' o~class~id                                        allocations=1
+ pad = '<51-byte literal>'                                     allocations=2
+ drop o ; call gc 'force'                                      allocations=3
```

so **exactly one** arena allocation happens between `.K~new` and `drop o` in that program, against
the nine or more the oracle needs to evict it. A ten-deep hold rooted from `Interp::alloc_with`
would therefore still be holding that instance at the `GC('force')`, its `UNINIT` would move to the
termination sweep, and the committed corpus row would flip from

```
start / built K / uninit ran / after-gc     (oracle, and this crate today)
```

to `start / built K / after-gc / uninit ran`. **Everything in this paragraph from "would therefore"
onward was a prediction from the allocation counts at the time it was written**; it was then run as
Control B, and "The controls, recorded as run" below has what the run actually did, which was that
flip and three more besides.

**Decision: license it.** The row is DEVIATION 6 below, and it carries a runnable witness in the
same shape as Divergence A's.

## What was built

### 1. The DEVIATION rows, and the file choice

**`docs/superpowers/plans/phase-4-exclusions.txt`'s DEVIATIONS section, as rows 5 and 6**, rather
than a Phase 5 successor file.

The plan flags the choice because "that file's set is read by a Phase 4 builtin gate". Re-measured,
and the hazard does not reach the DEVIATIONS section:

* `rust/crates/rexx-exec/tests/builtin_status.rs` was the only code in the tree that read the file
  before this task added a second reader.
  `/bin/grep -rn 'plans/phase-4-exclusions.txt' --include='*.rs' rust/` finds the path spelled out
  in many places and all but two are prose in a doc comment; the two that build a `PathBuf` are
  `builtin_status.rs:137` and, now, `licensed_divergences.rs:137`.
  `/bin/grep -n 'exclusions_path' rust/crates/rexx-exec/tests/builtin_status.rs` gives four lines:
  the constructor at `:136` and three uses inside `every_divergent_row_has_a_known_gap`
  (`:696`, `:697`, `:707`). That test does `exclusions.contains(&format!("KNOWN GAP: {name}"))` on
  the whole text. It parses no sections and counts no rows.
* The `excluded == 15` assertion at `:497` counts `Status::Excluded` rows in a run's own derived
  table against `rexx_inventory::builtins::wholly_excluded().len()`. Both sides of it are Rust; the
  file is not consulted.

So the Phase 4 gate's *set* is the builtins-exclusion set, held in `rexx-inventory`, and the file's
text reaches a gate only through `KNOWN GAP:` markers. A DEVIATION row naming no builtin cannot
touch either.

The positive reason for that file rather than a new one is the file's own definition: "An EXCLUSION
is work assigned to a later phase... A DEVIATION is a permanent difference chosen on purpose."
A permanent difference has no owning phase, and the section is already not a Phase 4 artifact --
DEVIATION 3 and DEVIATION 4 both landed in the pre-Phase-5 defect round, and the row texts say so.
Against that, a new file nothing reads and nothing points at is the failure this project has hit
before: the crate's own source cites `phase-4-exclusions.txt` as the deviations register from many
sites, and a second register would have to win that traffic before it was worth having.

**Both rows land subject to Moritz's veto, and each says so in its own text.** DEVIATION 3 and 4
name a licensor and a date; neither I nor the controller has that authority, so the rows carry the
measurement and the reasoning and not a licensor line.

### 2. The runnable witness

`rust/crates/rexx-exec/tests/licensed_divergences.rs`, new.

* `LICENSED_DIVERGENCES`, a `const` table in `KNOWN_DIVERGENCES`' shape, with the non-empty-table
  assertion reused verbatim in intent.
* `the_licensed_divergences_still_diverge_exactly_as_recorded` runs each row's program through
  `support::oracle`'s `locate()`/`Oracle::run` and through **both** crate engines
  (`Invocation::none().with_engine(Engine::Ir)` and `Engine::TreeWalker`), asserts stdout, stderr
  and exit status on each of the three sides against the row, asserts the two crate engines'
  three descriptors are equal to each other, asserts `oracle_stdout != crate_stdout`, and asserts
  `oracle.invocations() == LICENSED_DIVERGENCES.len()` so a table that classified rather than ran
  could not pass.
* `the_prose_rows_and_this_table_name_the_same_divergences` holds the set of names after
  `LICENSED DIVERGENCE WITNESS: ` in `phase-4-exclusions.txt` **equal** to the set of `name`s in the
  table, so neither half of a licence can be deleted alone and a typo in either copy fails both
  directions. Each marker is one line by construction: that file is hard-wrapped and a name long
  enough to wrap could not be found by a line scan. (This started as a one-directional
  `contains` check; see "The review round" below for the finding and the three controls.)
* Not a `datadriven` case file, and the reason is mechanical. `datadriven::TestFile::run` branches
  on `env::var("REWRITE")` and rewrites each file's expected block in place
  (`datadriven-0.9.0/src/lib.rs:501`, read in the local registry cache). A table whose whole claim
  is "this must not be updated in place, a fix deletes it" cannot live in a medium with a supported
  command for updating it in place.
* Not gated on `REXX_CORPUS_GATE`, unlike most oracle harnesses here. `parse_version_oracle.rs`'s
  own module doc records that the protection the gate buys is already unavailable in this crate --
  `builtin_status.rs` invokes the oracle on a plain `cargo test` with no gate -- so gating restores
  nothing and costs the witness three of the five gate commands.

Green as committed:

```
cargo test --release -p rexx-exec --test licensed_divergences
  15 passed; 0 failed  (13 of those are support::* unit tests linked into the binary)
```

### 3. The controls, recorded as run

Both mutations were applied to the tree, run, and restored **from a scratchpad copy taken before
the edit** (`scratchpad/task6/t6-backup/`), never with `git checkout --`. `git status --porcelain`
after each restore showed only my own intended changes, and
`/bin/grep -rn 'TASK 6 CONTROL' rust/crates/` exits 1 with no output.

**Control A -- deliver the class's UNINIT from the driven collection.** The class-side drain of
`run_termination_uninits` added to `Interp::run_ready_uninits` (`dispatch.rs`), which is the
function `builtin/state.rs`'s `gc` calls after its `collect_now`:

```rust
let classes = self.classes().take_uninit_classes_in_sweep_order();
for class in classes {
    loud.extend(self.run_one_uninit(class));
}
```

`cargo test --release -p rexx-exec --test licensed_divergences`, exit 101:

```
thread 'the_licensed_divergences_still_diverge_exactly_as_recorded' panicked at
crates/rexx-exec/tests/licensed_divergences.rs:176:5:
assertion `left == right` failed: [class-uninit-at-driven-collection] this crate's own stdout moved on Ir
  left: "before\nclass uninit\nafter\n"
 right: "before\nafter\nclass uninit\n"
```

The crate started **agreeing with the oracle** and the row went red, which is the property the row
claims: a fix reddens and should delete the row rather than update it.

**Control B -- build the hold.** A ring of the ten most recently allocated handles, a
`VecDeque<ObjRef>` field on `Interp`, filled in `Interp::alloc_with` after the allocation and
pushed as temps in `Interp::collect_now` before `heap.collect`.

`cargo test --release -p rexx-exec --test licensed_divergences`, exit 101:

```
assertion `left == right` failed: [driven-collection-reaches-a-new-object] this crate's own stdout moved on Ir
  left: "start\nafter-gc\nuninit ran\n"
 right: "start\nuninit ran\nafter-gc\n"
```

**And what else it reddened, which is the part that was a prediction until it was run.**
`REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` reports `315 of 315 matching`
at exit 0 on the tree as committed, and under Control B exits 101 with
`4 of 315 corpus programs disagree`, every one on stdout at rc 0 with empty stderr:

```
lang/uninit_instance_collected.rex
  crate  start / built K / after-gc / uninit ran      oracle  start / built K / uninit ran / after-gc
lang/uninit_nested_collection.rex
  crate  start / built / end / g uninit / inner built / K finalizer ran during the nested collection? 0
  oracle start / built / g uninit / inner built / K finalizer ran during the nested collection? 0 / end
lang/class_behaviour_snapshot_delete.rex
  crate  a 1 / b still-here / c 1 / d 0 / e done / uninit-ran
  oracle a 1 / b still-here / c 1 / d 0 / uninit-ran / e done
lang/setmethod_uninit.rex
  crate  first done / second done / one-off uninit    oracle  one-off uninit / first done / second done
```

And the padding family under a Control B build of `rexx-run`, two runs each on both engines, rc 0
with empty stderr throughout: N = 8, 9 and 10 all answer `start / after-gc / uninit ran`. That is
the oracle's answer at N = 8 and not its answer at N = 9 or 10, so the hold does not close the
divergence -- it moves it to the other half of the range and takes four corpus rows with it.

I predicted one corpus regression from the allocation counts and there were four. The prediction
was right about the mechanism and understated the blast radius, which is the direction that matters
here: the recommendation to license is stronger than the argument that reached it.

## Corrections made to the plan and the spec

**`docs/superpowers/plans/2026-08-27-phase-5b.md`, Task 6's section.** Its closing paragraph read
"Two ways out and this task picks one... Do not leave it unstated" and would have stayed that way
after the task picked. A paragraph recording the choice, the measurement behind it and where the
row lives is appended to it. The paragraph it follows is left as written -- it is the question, and
the question was not wrong.

**Nothing in the plan's Task 6 section needed correcting on the facts.** Its Divergence B table
(`0..8` oracle `a c uninit`, `9..` oracle `a uninit c`, crate `a uninit c` throughout) reproduces
exactly, five runs a side. Its `nop` control and its C++ citations -- `memory/RexxMemory.cpp:1053`,
`memory/Memory.hpp:107` value 10 -- were re-read in the read-only tree and are right.

**The spec was not edited, deliberately.** D59a's licence is stated over the observable "a class
object was collected" and its "a fourth/fifth observer ... needs its own decision" sentence is about
observers of *class* collection. Divergence B is about an instance and about the collector's own
reachability, so it is not a fifth observer of that observable and a D-number beside D59a would say
it was. The decision lives in the deviations register, which is where a permanent difference lives,
and the plan's Task 6 section points at it.

## What this task did not do

* **It did not build the hold.** DEVIATION 6 licenses the divergence instead, and the measurements
  above are why. Control B is the only place the hold exists, and it was reverted.
* **It did not build witnesses for D59a's other three consequences, and none was owed here.**
  `~subclasses` keeping a dropped class and a `WeakReference` still answering are 5c's -- the spec's
  D59a table already names 5c as the owner of each, and there is no 5c plan or spec file yet to
  record them in (`ls docs/superpowers/plans/` and `docs/superpowers/specs/` have none). The OOM
  asymmetry is stated in D59a with its measurement and the plan says it needs no runnable witness.
  I added nothing for any of the three and re-read the spec to confirm all three are already
  recorded there.
* **It added no corpus program and moved no gate-table row**, so `rust/corpus/phase-5b.txt` is
  unchanged. The phase's rule about adding to that file in the same commit had nothing to bind.
* **It ran no performance sitting.** The change is one new file under `tests/` and two document
  files; nothing in `src/` of `rexx-exec`, `rexx-core`, `rexx-classes` or `rexx-lib` is touched in
  the commit, so the release binary the axes measure is byte-identical and the sitting would measure
  noise. That is the 5a constraints' own stated exemption.
* **It reached for no `unsafe`**, and no site here suggested it.
* **It did not re-measure D59a's `~subclasses`, `WeakReference` or OOM transcripts.** Those are the
  spec's own figures for rows this task does not own, and nothing here rests on them.
* **It did not sweep the whole suite under Control A.** Control A was run against the witness only.
  Control B was run against the witness *and* the corpus differential, because that is where the
  decision's cost was predicted to land; the same sweep was not owed for A, whose mutation is the
  "fix" direction of a row this crate has decided not to take.

## The margin of the Divergence B shape, re-measured rather than inherited

The brief warns that the controller's first two reconstructions did not reproduce the divergence
because they put an extra `say` between `~new` and `drop`. That is a claim about the shape and the
witness rests on it, so it was measured rather than carried over.
`t6-summary.sh scratchpad/task6/progs/marginA.rex 3` and `.../marginB.rex 3` -- the witness's own
eight-clause program with one `say 'built'` inserted immediately after `.K~new`, and with it
inserted immediately before `drop o` -- rc 0 and zero stderr bytes on all eighteen runs:

```
marginA (say before the padding)   oracle, ir, tree-walker all  start / built / uninit ran / after-gc
marginB (say after the padding)    oracle, ir, tree-walker all  start / built / uninit ran / after-gc
```

One `say` in either position makes all three sides agree, so the divergence the row records is gone.
That margin is in the row's own source comment.

## Also re-measured, because the row states it

`k~setMethod('UNINIT', ...)` from a program context, three runs of three, standard oracle wrapper,
fresh empty directory: rc 159, stdout empty, stderr

```
     2 *-* k~setMethod('UNINIT', 'say ''class uninit''')
Error 97 running <path> line 2:  Object method not found.
Error 97.2:  Object "The K class" cannot accept private message "SETMETHOD" from this context.
```

## A collision in the plan file: the controller wrote into it, and withdrew

For part of this task `docs/superpowers/plans/2026-08-27-phase-5b.md` carried two uncommitted edits.
Mine was the Task 6 decision paragraph. The other was `## At the close of 5b: the comment policy
tightens`, quoting Moritz on 2026-09-01, appended at the end of the file.
`ls -la --time-style=+%H:%M:%S` on it read `23:26:18` against a wall clock of `23:35:23`, so it had
been written nine minutes earlier, long after my own edit.

I did not commit the file and did not rewrite it: committing would have put another agent's
unreviewed, possibly still-in-progress work into a commit whose message says it is Task 6's, and
rewriting it to drop that section would have clobbered live work. I messaged the controller and kept
working.

**The section was the controller's**, appended to the plan while this task was editing the same
file, and they withdrew it on being told: extracted to their own scratchpad, the file truncated back
to my hunk alone, and `git diff` checked before handing the file back to me. They will land it as a
separate commit after this one. So the plan file **is** in this task's commit.

An earlier version of this section said the removal was "by someone other than me" and that where the
section went was "not something I can see from here". Both were true when written and neither is now,
and the controller's objection to leaving them is right: **a report recording an unexplained
disappearance from a tracked file is worse than one recording a named mistake.** Verified after the
fact rather than taken on the message alone: `/bin/grep -n 'comment policy tightens'` on the file
exits 1, `git log --oneline -1` is still `6eb234598` so nothing was committed under it, and
`git diff docs/superpowers/plans/2026-08-27-phase-5b.md | /bin/grep '^@@'` reports a single hunk.

## Moritz's ruling, and what it changed in what was already built

Two messages arrived from the controller after the rows and the witness were written and after both
controls had been run. Nothing behavioural changed; the rows' **reasoning** did, and so did their
SCOPE.

**The ruling.** *"anything that depends on gc ordering is inherently unpredictable, and we shouldn't
aim for equal behavior. it just needs to be one of the many correct orderings."* -- Moritz,
2026-09-01, given on Divergence B, relayed with the direction that Divergence A be written the same
way. Divergence B is licensed and the hold is not to be built, which is what this task had already
decided and measured; the ruling makes it a decision rather than a recommendation.

**Both documentation citations re-read before writing anything, in the git-ignored working copies.**
`svn info oodocs/rexxref` and `svn info oodocs/rexxpg` both report **r13198**, which is the revision
`global-constraints.md` stamps, so the working copies are the ones the constraints describe.
`/bin/grep -n 'id="obdes"' oodocs/rexxref/en-US/provide.xml` puts that section at **line 709**.
`sed -n '709,722p'` prints it; below is its prose verbatim, with the XML tags and one intervening
paragraph (about releasing system resources) elided at the `...`:

```
Object destruction is implicit. When an object is no longer in use, Rexx
automatically reclaims its storage.
...
An object requiring uninitialization
should define an UNINIT method. If this method is defined, Rexx runs it before
reclaiming the object's storage. If an object has more than one UNINIT method
(defined in several classes), each UNINIT method is responsible for sending
the UNINIT method up the object hierarchy.
```

`sed -n '440,460p' oodocs/rexxpg/en-US/classes.xml` puts the one-line form at **line 447**: `If an
object has an UNINIT method, Rexx runs it before reclaiming the object's storage.` Both citations as
the controller gave them.

The controller's warning about the inheritance sentence holds and is why nothing here rests on it:
`rexxref`'s "sending the UNINIT method up the object hierarchy" and `rexxpg`'s `self~uninit:super`
form are both about the chain *within one object's* class hierarchy, not about ordering *between*
objects. Neither row mentions it.

**What changed in DEVIATION 5.**

* Its `LANDING SUBJECT TO MORITZ'S VETO` paragraph is now `LICENSED BY MORITZ, 2026-09-01`, with the
  note that the ruling was given on row 6 and relayed as applying here because it is the same
  question.
* `WHY EXACT AGREEMENT IS NOT AVAILABLE` -- which argued from D59 being expensive to reverse -- is
  demoted to `A SUPPORTING FACT AND NOT THE REASON`, with a sentence saying it is why the ordering
  is not changed and not why it is allowed to differ. In its place, `WHY THIS IS A CORRECT ANSWER
  RATHER THAN A SHORTFALL` leads with the two quotations and the ruling.
* A paragraph was added for the tension the controller asked not to be papered over: under D59 a
  class object's storage is never reclaimed here, so the document's trigger never fires for one and
  the sweep runs its UNINIT for storage nothing will reclaim. That is more than the document
  requires, not less, and it satisfies both "before reclaiming" (vacuously) and D69.
* Its `SCOPE` now states the boundary as a list of what is **not** licensed -- an UNINIT that never
  runs, one that runs while the object is reachable, any exit-status or stderr difference, any
  stdout difference beyond the finalizer's own line -- in DEVIATION 0's "deliberately narrow, not a
  precedent" register.

**What changed in DEVIATION 6.** The same `LICENSED BY MORITZ` line, the same `SCOPE` boundary, and
a `WHY THIS IS A CORRECT ANSWER` paragraph pointing at row 5's quotations and adding the sentence
that matters here: this crate reclaims an instance and runs its UNINIT before doing so, which is the
whole of what is specified, and the oracle's save stack delays its own reclamation as its
collector's choice. The allocation-counting measurement and the measured corpus regression under the
hold are kept and relabelled `A SUPPORTING FACT AND NOT THE REASON`, because they say why the hold
is not built rather than why the difference is allowed.

**The boundary list is written out in full in both rows rather than cross-referenced.** A pointer
would be one edit away from a reader of row 6 never seeing it, which is the drift the spec's Risks
table names. The duplication's own hazard is answered by making the two lists byte-identical:
`python3` counting the exact block in the file reports **2** occurrences of the bulleted list and
**2** of the paragraph introducing it, so a diff shows any drift between them.

**The witness/licence tension is stated in three places.** Both DEVIATION rows and the test's module
doc now carry: a change on the **crate's** side, inside the SCOPE boundary, is a deliberate edit to
the row by whoever re-timed the finalizer and not a defect the row caught; a change on the
**oracle's** side, or any exit-status or stderr change on either side, is a real finding. Without
that sentence the next person to legitimately re-time a finalizer reads a red test as a regression.

**What did not change.** The programs, the recorded transcripts, the assertions, both controls and
their results. The witness still pins exact bytes on both sides, which the controller asked for
explicitly.

**Also corrected while writing this.** My first draft of the SCOPE cited "D59a's own risk table"
for the drift hazard. `/bin/grep -n -i 'widens by drift' docs/superpowers/specs/2026-08-27-phase-5b-instances.md`
puts it at `:1088`, in a section headed `## Risks` at `:1082`, and the row reads "D59's licence
widens by drift", not D59a's. The row now cites the spec's Risks table and quotes it as written.

## The review round: one real finding, one reordering, one question answered

### The marker check was one-directional and its doc claimed both. Fixed.

The controller's reading is right, and I checked it before acting: the file was 317 lines with two
`#[test]` functions, `/bin/grep -n 'LICENSED DIVERGENCE WITNESS'` on it found the marker string used
once (`:308`), and the same grep over `rust/`, `docs/` and `.superpowers/` found no third instrument
that could close the other direction -- only the two DEVIATION rows and one line of this report.

`every_row_here_is_licensed_by_a_deviation_row` did `exclusions.contains(marker)` per table row. So
deleting a DEVIATION row reddened it, and **deleting a row from `LICENSED_DIVERGENCES` reddened
nothing**: the prose would go on claiming a witness that no longer ran, which is the state the file
was in before this task and the state its DEVIATION 4 transcripts are still in. The doc sentence
said it stopped "either being deleted alone", which overstated it in the direction that made the
weaker half look covered.

Replaced by `the_prose_rows_and_this_table_name_the_same_divergences`: scan the exclusions text for
every line containing `LICENSED DIVERGENCE WITNESS: `, take the rest of that line as a name, and
assert the resulting `BTreeSet` **equals** the set of `name`s in the table. The doc now says what
the code does.

**Three controls, each run, each restored from a scratchpad copy** (`t6-backup/`), never with
`git checkout --`. Command each time:
`cargo test --release -p rexx-exec --test licensed_divergences`.

```
baseline                                        exit 0    15 passed; 0 failed
delete the second row from LICENSED_DIVERGENCES exit 101  the_prose_rows_... FAILED
delete one marker line from the exclusions file exit 101  the_prose_rows_... FAILED
typo one marker name (colection for collection) exit 101  the_prose_rows_... FAILED
restored                                        exit 0    15 passed; 0 failed
```

The first and third are the ones the old `contains` check could not see. `git status --porcelain`
after restoring shows only my own three paths.

### DEVIATION 6's WHY now leads with the measured control

Reordered as the controller asked, and the reasoning is his: "this change would break four committed
programs and move the divergence rather than close it" is checkable and tells a later reader what it
would cost them, where "we cannot match the allocation count" invites someone to try harder. The row
now opens `WHY THE HOLD IS NOT BUILT, AND THIS IS MEASURED RATHER THAN ARGUED`, names all four
programs, and gives the inversion (`N` = 8, 9 and 10 all answering `start / after-gc / uninit ran`
under the hold, which is the oracle's answer at 8 and not at 9 or 10). The allocation counts follow
under `A SUPPORTING FACT AND NOT THE REASON`, with a sentence saying why they are the weaker
argument. **The prediction is recorded beside the measurement**: one corpus regression was predicted
from the allocation figures and four were measured, and the row says so.

The controller's second note needed no change: the sentence explaining that this row is not D59a's
-- that licence is stated over a class object being collected and this is an instance -- is kept
verbatim.

### The boundary row sits at N=8 by choice, and it is stable there

`N` = 8 is one below the threshold, which is the most sensitive point in the range, and it was
chosen for that: a row at `N` = 0 would also diverge but would go on diverging under any change to
either side's window, where a row at the boundary reddens if the oracle's save stack, or this
crate's allocation on the path before `~new`, ever moves by one.

Measured on the **witness's own program bytes**, extracted from the `const` table by a script rather
than retyped, ten runs on each of the three sides
(`t6-summary.sh scratchpad/task6/progs/witness-row2.rex 10`):

```
30 of 30 runs   rc=0  stderr=0 bytes
oracle       10/10  start / after-gc / uninit ran
ir           10/10  start / uninit ran / after-gc
tree-walker  10/10  start / uninit ran / after-gc
```

No flake at the boundary in thirty runs.

### Four things the controller asked be preserved, and were

`oracle.invocations()` asserted against the table length; the engine-agreement assertion taken from
the two actual runs rather than inferred from the single `crate_stdout` field; the non-empty-table
guard; and the `datadriven` exception reasoning. All four are untouched by this round. The last one
is in the module doc and in "The runnable witness" above: `datadriven::TestFile::run` branches on
`env::var("REWRITE")` and rewrites each file's expected block in place
(`datadriven-0.9.0/src/lib.rs:501`, read in the local registry cache), and a table whose whole claim
is "this must not be updated in place" cannot live in a medium with a supported command for doing
exactly that.

## A detached gate run dies at a turn boundary, and an idle notice hides it

**This is the finding, and I had it wrong first.** When the controller reported my gate run stopped,
I answered that their `ps` snapshot and mine disagreed because a run between commands shows no cargo
-- true in general, and not what happened. The timestamps refute it and I checked them rather than
argue:

```
stat -c '%n birth=%w mtime=%y' on the status files
  final-gates.status   birth 23:34:11   mtime 23:36:52   holds GATE1=0 GATE2=0 GATE3=0
  def-gates.status     birth 23:47:00
```

Their `ps` ran at **23:43:07** and found no cargo and no rustc. `final-gates.status`'s last write is
**23:36:52**, the line for gate 3; gate 4 -- `REXX_CORPUS_GATE=1 cargo test --release --workspace`,
which takes minutes -- had started and never wrote its own line. So at 23:43 that run had been dead
for some part of six minutes. **My own `TaskStop` on it came later**, immediately before
`def-gates.status` was created at 23:47:00, so the run was not alive for me to stop: it had already
gone. And the `2149870 cargo` I quoted back at them was from the run started *after* their message,
so it was never evidence about 23:43.

What actually happened is structural rather than anyone's mistake: **a gate run started with
`run_in_background` does not survive the end of the turn that started it**, and it dies wherever it
happens to be -- here mid-gate-4, with three green lines already on disk and no fourth. From inside
the turn it was in flight, so that is what I reported, and an idle notice describing a pending run is
indistinguishable from one describing a run that has since died. This is the second time in this
task that a status nobody read stood in for a gate.

**What that means for how a gate run is used here**, and it is the reason the status file and the
sha pair above are worth inheriting: the run is something to return to and *read*, never something to
report as pending. A green line is only a gate once someone has looked at it, and a run that is
"still going" has to be re-checked for a process, not assumed.

Separately and for a different reason: **the run that was alive when the controller wrote was stopped
and discarded on purpose**, because the marker finding and the DEVIATION reordering both changed
files it had already built, and a run spanning an edit certifies neither tree. The statuses below are
from a single run started after every edit in this report had landed.

## Two things about the gate wrapper, for whoever runs the next task

Both are answers to hazards this task hit rather than good practice in the abstract, and the
controller asked that they be written down so the next task inherits them.

**Each gate writes its status to a file rather than into a pipe.** The wrapper is a brace block that
runs the six commands in sequence and appends `GATEn=$?` to one status file after each, finishing
with `ALL-DONE`:

```
cargo fmt --all --check > $SP/n-gate1.out 2>&1; echo "GATE1=$?" >> $SP/final.status
cargo clippy --workspace --all-targets -- -D warnings > $SP/n-gate2.out 2>&1; echo "GATE2=$?" >> $SP/final.status
... and so on for the three test gates and the phase gate
echo "ALL-DONE" >> $SP/final.status
```

`cmd | tail` inside an `&&` chain reports tail's status, which is the rule this phase already
carries. Writing the status to a file goes one step further: it makes "a status nobody read" not a
state the run can be in, because the status outlives the turn that started the run. This mattered
here -- the run was started in the background three times and each attempt outlived a turn.

**A sha256 pair taken before the first gate and after the last** -- `final-sha-before.txt` and
`final-sha-after.txt`. A gate run that spans someone else's write, or my own, measures a tree that
existed at no commit and certifies nothing; comparing the two files detects that rather than
promising not to edit.

**Its coverage is three files, not the tree, and an earlier draft of this section claimed the
general property.** The controller caught it. What the pair actually hashes is
`crates/rexx-exec/tests/licensed_divergences.rs`, `phase-4-exclusions.txt` and
`2026-08-27-phase-5b.md` -- the three paths in this commit, which is where tonight's collision
landed. **It does not cover `src/`, any other test, or anything else this run certifies.** A write
there during the run would pass it silently.

The claim is narrowed to match rather than the instrument widened, because widening it means
restarting a run already past gate 3 and the two intervals would then not join up. One partial
widening was free and was taken: a hash of `git status --porcelain`, `git diff` and the untracked
test file together, which covers **every tracked modification** with no list to keep in sync.
It was taken mid-run at 23:58:13 against a run that began 23:53:35, so the honest statement of
coverage is two intervals with a named hole:

* the three named files, from the first gate to the last;
* every tracked modification, from 23:58:13 to the last gate;
* **uncovered: a write to any file outside those three, between 23:53:35 and 23:58:13.**

**What the next task should do instead**, and it is the controller's suggestion rather than mine:
hash the repository's own view of what has changed -- `git status --porcelain` and `git diff`
together, plus each untracked path -- as the *before* and *after*. That covers every tracked
modification, needs no upkeep as paths move, and removes the failure mode where a third file joins
the commit and nobody adds it to the wrapper. **A file list that has to be kept in sync with a
commit is one that will eventually disagree with it.**

**This is the second instrument in this task whose description outran what it did**, the first
being the one-directional marker check. Two in one task is a pattern, and the common cause is
worth naming: both descriptions were written from what the check was *for* rather than from what it
*does*. The check itself was sound both times; the sentence next to it was the defect. Writing the
description from the code -- "this hashes these three paths", "this asserts `contains` per row" --
would have caught both without a reviewer.

## The gates, from one run started after every edit above had landed

From `rust/`, statuses appended to `final.status` by the wrapper rather than read from a pipe, and
each read unpiped from that file:

```
cargo fmt --all --check                                                 GATE1=0
cargo clippy --workspace --all-targets -- -D warnings                   GATE2=0
cargo test --release --workspace                                        GATE3=0
REXX_CORPUS_GATE=1 cargo test --release --workspace                     GATE4=0
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast      GATE5=0
ALL-DONE
```

All five exit 0. `command -v memcap` answers `/home/moritz/.local/bin/memcap`, so gate 5 ran under
the cap rather than the `ulimit -v` substitute. Gates 3, 4 and 5 each report 104 `test result: ok`
lines and no `FAILED`; gates 4 and 5 each print `315 of 315 matching` from the corpus differential.

**The tree did not move under the run.** `diff final-sha-before.txt final-sha-after.txt` is empty,
exit 0 -- the three named files are byte-identical either side. The wide hash over
`git status --porcelain` + `git diff` + the untracked test file reads
`57aab0fd46397b756ca3b4258b8cfd632031711cdca697dfe98048fd44c65195` both at 23:58:13 and after
`ALL-DONE`, so no tracked file was modified across that interval either. The hole named above
remains the hole: a write outside the three files between 23:53:35 and 23:58:13 would not have been
seen.

### The phase gate

```
REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec \
    --test gate_table_c --test gate_table_d --no-fail-fast              PHASEGATE=101
```

**Exit 101 is the row the brief names as Task 8's, and nothing of this task's.** The panic reads
`1 row(s) of gate table C owned by a closing or closed phase do not agree with the oracle; the first
of them are ["gate-tables/concepts/methodsbyclass.rex"]`, and the plan's Task 8 section
(`## Task 8: multidimensional Array, and methodsbyclass (D63)`) owns it.

Not-agree counts, both tables, as the two harnesses print them:

```
table C   5a: 135 rows, 0 not yet agree     gated by this run: 1 row
          5b:   6 rows, 1 not yet agree     (methodsbyclass, Task 8's)
          5c: 1347 rows, 912 not yet agree
          test result: FAILED. 13 passed; 1 failed

table D   5a:  36 rows, 0 not yet agree     gated by this run: 0 rows
          5b:   2 rows, 0 not yet agree
          5c:  38 rows, 35 not yet agree
          7:    1 rows, 1 not yet agree
          deferred-parse-error-rendering: 2 rows, 2 not yet agree
          test result: ok. 15 passed; 0 failed
```

`obdes` reads `agree loud=no 5b` in table C's listing, which is D60's row and the one DEVIATION 5's
SCOPE says stays gated rather than re-filed.

## The commit

`09d2e2e57c9e89d1d04f33a59ffd5648a05a0dd4`, read back with `git log -1 --format=%H` after
committing rather than quoted from memory.

```
git commit -F <message file>
 3 files changed, 665 insertions(+)
 docs/superpowers/plans/2026-08-27-phase-5b.md      |  28 ++
 docs/superpowers/plans/phase-4-exclusions.txt      | 303 +++++++++++++++++++
 rust/crates/rexx-exec/tests/licensed_divergences.rs | 334 +++++++++++++++++++++
```

**Three tracked paths, not the four I told the controller to expect.** This report is the fourth and
it is not in the commit, because `.superpowers/` is git-ignored -- `git check-ignore -v` names
`.gitignore:30` -- and no sibling report on this plan was ever tracked (`git log -- .superpowers` is
empty; task-0 through task-5's reports all sit beside this one, untracked). That is the arrangement
the brief assumed when it named this path, not an omission.

`git diff --cached --stat` was re-read immediately before committing and **`Cargo.lock` is not in
it**; `git status --porcelain` showed it unmodified as well, so the three gate runs that had just
finished did not rewrite it. `git status --porcelain` after the commit is empty.
