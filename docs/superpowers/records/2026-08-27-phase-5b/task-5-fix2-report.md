# Task 5, second fix round -- report

Status: COMPLETE, including the controller-ruled guard test in the addendum. Written first and
appended as the work went, per the global constraints.

BASE: 065bceab7 on plan/rust-rewrite. Committed at 3148d8cdd and f65694c8b. Five gates green at
both, phase gate 101 by design at both.

## Scope

The brief assigns IMP-A, MIN-A, MIN-B, MIN-C, MIN-D and MIN-E. IMP-B is the controller's
and is not touched here.

## Probe convention

Every oracle transcript below: a fresh empty directory created per run with `mktemp -d`, absolute
program path, `( cd $D; ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib
timeout -s KILL 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`, stdout, stderr and exit
status captured to three separate files, never `2>&1`. The runner is
`scratchpad/t5fix2/orun.sh`.

Crate transcripts: `scratchpad/t5fix2/crun.sh`, same fresh-directory rule, `timeout -s KILL 20`,
`REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`, three descriptors separate.

Mutation arms are built in a `git archive 065bceab7` extract of `rust/` and `interpreter/` under
`scratchpad/t5fix2/tree/`, with its own `CARGO_TARGET_DIR=scratchpad/t5fix2/target`. **The live
worktree is not mutated at any point in this round.**

Oracle checked before anything else, `parse version`:
`REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026`, `bin/rexx` 62,600 bytes -- the binary
`rust/CLAUDE.md` records.

None of the probes below is one of `rust/corpus/oracle-crashes.txt`'s eight shapes: none uses an
orphaned `WHEN`, `date('M','0','D')`, a `'00'x` needle, `NUMERIC DIGITS` above 1000, a doubled
`CALL ON` name in one clause, an array with a leading empty slot as a sole subscript, `GUARD ... WHEN`,
or a metaclass `NEW`. Read at `corpus/oracle-crashes.txt` this round rather than remembered.

# IMP-A

## 1. The split, reproduced

Three arms on `Interp::run_termination_uninits`, each applied to the extract, `touch`ed, rebuilt with
`cargo build --release --bin rexx-run`, and `cmp`'d against the saved pristine binary -- all three
reported "binary changed", so no arm measured a stale build.

* **MF** -- the early-return guard `if self.processing_uninits { return Vec::new(); }` deleted, the
  `= true`/`= false` kept.
* **MSET** -- `self.processing_uninits = true;` and `self.processing_uninits = false;` deleted, the
  guard kept.
* **MF2** -- both deleted. (The reviewer's M-F and M-F2; MSET is the arm neither the review nor the
  brief had isolated, and it is the one that names which half carries the behaviour.)

Four probe shapes, all four asking the same question of the termination sweep and differing only in
scaffolding: **A** is the controller's probe verbatim; **B** is A minus `say 'inner built'`; **C** is
B minus the twelve-variable padding; **D** is A minus the padding.

Oracle, ten runs of each of A, B, C and D, three descriptors hashed separately: **one distinct stdout
each, zero-byte stderr every run, rc 0 every run.**

```
A, D  start / end / outer fin / inner built / inner ran inline? 0
B, C  start / end / outer fin / inner ran inline? 0
```

The crate at 065bceab7 matches the oracle on all three descriptors for all four shapes, on `ir` and
on `tree-walker`.

Under the arms, all four shapes on both engines:

| arm | A | B | C | D |
|---|---|---|---|---|
| MF (guard alone) | green | green | green | green |
| MSET (assignment alone) | **RED** | **RED** | **RED** | **RED** |
| MF2 (both) | **RED** | **RED** | **RED** | **RED** |

Every red cell is `rc 0`, empty stderr, and `inner ran inline? 1` -- a **silent wrong answer**, which
is the failure mode this phase's constraints call the worst one available.

**So the controller's finding reproduces exactly, and the split is confirmed independently:** the
assignment is the load-bearing half, and removing the guard alone changes nothing on any of the four
shapes or on either engine.

## 2. The padding is load-bearing, and the unpadded shape is a false witness

The two shapes that drop the twelve-variable padding (C and D) redden under MSET on the crate, so on
redness alone the smaller program would do. **On the oracle they answer 0 for a different reason**,
and that makes the unpadded shape a row whose comment would be false.

The control that shows it: the finalizer's body run as an **ordinary class method**, where no sweep
is in progress and the interlock is not held, so the answer is 1 exactly when the nested collection
really readied `K`'s instance.

```
E  ::method probe class  o = .K~new ; <twelve-variable padding> ; drop o ; call gc 'force' ; return .K~ran
F  the same with the padding deleted

oracle, ten runs each, one distinct stdout each, rc 0, stderr 0 bytes
  E   start / ran inline? 1
  F   start / ran inline? 0
crate at 065bceab7, ir and tree-walker
  E   ran inline? 1        F   ran inline? 1
```

So without the padding the **oracle never collects `K`'s instance at all** -- it is still reachable
from a stale slot -- while the crate does collect it. In the unpadded probe the two sides print the
same `0` for opposite reasons: the crate because the interlock refused, the oracle because there was
nothing to run. The row would agree with the oracle and would still redden under MSET, and its
comment would nonetheless be false about what the oracle is doing.

**The row therefore keeps the padding.** The one clause dropped from the controller's probe is
`say 'inner built'`, whose information is implied by the answer line: `inner ran inline? N` cannot
print unless `.K~new` succeeded and the finalizer ran to its last clause. `say 'outer fin'` stays --
it is the only mark of the finalizer's *entry*, which nothing else in the transcript carries -- and
`start`/`end` stay, because "the finalizer is reached because the program ends" is read off the
answer line falling after `end`.

That is shape **B**, and it is the row.

## 3. Nothing in the committed corpus sees MSET, and the sweep is not blind

The brief asks for the "adds coverage" check extended from the ten `UNINIT` rows to the corpus. Run
over the **union of `corpus/phase-4a.txt`, `phase-4b.txt`, `phase-4c.txt`, `phase-5a.txt` and
`phase-5b.txt` at 065bceab7 -- 286 entries, every one of which resolves to a file** -- on both
engines, 12-way parallel, each program run from its own parent directory with `Stdio::null` standard
input, which is what `tests/support/oracle.rs`'s `wrapped()` and `run_with` do (`.current_dir(
path.parent()...)`, `Stdio::null()`), so the sweep sees what `tests/corpus.rs` sees.

**Stability first**: two consecutive sweeps of the pristine binary are byte-identical across all
286 x 2 x 3 descriptor files, so a difference in the arms is the arm.

| arm | corpus programs whose crate output changes, either engine |
|---|---|
| MF (guard alone) | none |
| MSET (assignment alone) | none |
| MF2 (both) | none |

**Negative controls on the sweep itself**, because "nothing changed" reads exactly like a sweep that
cannot see:

* **S1**, the reviewer's M-A -- `self.check_uninit(class);` deleted from
  `ClassGraph::update_sub_classes` -- changes exactly `lang/uninit_class_mixin.rex` and
  `lang/uninit_class_inherit_runtime.rex`, on both engines and nothing else in the 286.
* **S2**, the reviewer's M-H -- `sweep.reverse();` inserted before the stable `sort_by_key` in
  `ClassRegistry::take_uninit_classes_in_sweep_order` -- changes exactly
  `lang/uninit_class_inherit_runtime.rex`, on both engines and nothing else in the 286.

Both reproduce the reviewer's arms, and both prove the sweep detects a stdout change where one
exists. So **no committed corpus program witnesses the termination sweep's copy of the interlock**,
which is IMP-A, measured over the whole corpus rather than over the ten `UNINIT` rows.

# MIN-A -- the runtime row's non-redundancy, with a control that separates the two rows

*Placed here, in the middle of IMP-A, because the control that settles it is section 3's S2 and this
report was written as the work went. IMP-A continues at section 4.*

The arm the round's report cited (its C3, the bucket sort deleted) reddens both rows together, so it
cannot separate them. S2 above does, and against the whole corpus rather than the ten rows.
Oracle run for each of the two rows from the row's own directory:

```
lang/uninit_class_mixin.rex             oracle rc 0, stderr 0 bytes, stdout  main / uninit on K / uninit on M
  pristine  ir, tree-walker   green
  S1        ir, tree-walker   RED   main / uninit on M
  S2        ir, tree-walker   green

lang/uninit_class_inherit_runtime.rex   oracle rc 0, stderr 0 bytes, stdout  main / inherited / u ZED / u QQ / u MX
  pristine  ir, tree-walker   green
  S1        ir, tree-walker   RED   main / inherited / u ZED / u MX
  S2        ir, tree-walker   RED   main / inherited / u QQ / u ZED / u MX
```

**S2 reddens `uninit_class_inherit_runtime.rex` and leaves `uninit_class_mixin.rex` green**, so the
runtime row sees the tie-break inside a shared bucket that no other committed program can see. That
is the row's non-redundancy evidence, and it replaces the report's C3 sentence.

## 4. The guard half: I looked and did not find a witness, and here is what I ran

Deleting the guard (MF) changes nothing, but that is consistent with two different worlds -- the
guard never fires, or it fires and the re-entrant call is harmless. A deletion cannot tell them
apart, so the instrument here is not a deletion. **The taken branch was made observable**: the guard's
body replaced by `panic!("GUARD FIRED in run_termination_uninits")`, and the same done separately to
`run_ready_uninits`' guard as a positive control.

**The positive control fires**, which is what makes the instrument evidence rather than decoration:
`GPANIC_R` reddens `lang/uninit_nested_collection.rex` on both engines with `GUARD FIRED in
run_ready_uninits` on stderr at rc 101, and the sweep picked up the change on all three descriptors
(`.out`, `.err` and `.rc`), so the sweep is not blind to a stderr-only or status-only change either.

**`GPANIC_T` never fired on anything I ran:**

* the whole corpus union, 286 programs x 2 engines -- no output change on any descriptor, and no
  `GUARD FIRED` on any stderr;
* eight probes written specifically to try to re-enter the termination sweep, each in its own
  directory containing only that probe's files, oracle and crate both run from that directory:

| probe | shape | oracle | crate `ir` / `tree-walker` | `GPANIC_T` | `GPANIC_R` |
|---|---|---|---|---|---|
| g1 | the candidate row: forced collection inside a termination finalizer | `start / end / outer fin / inner ran inline? 0`, 3 runs alike, rc 0 | matches | silent | **fires** |
| g2 | termination finalizer that `CALL`s an external `.rex` which drops and forces | `start / end / outer fin`, rc 0 | matches | silent | silent |
| g3 | the same driven through three `INTERPRET`s | `start / end / outer fin / after interpret 0`, rc 0 | matches | silent | **fires** |
| g4 | termination finalizer that `EXIT`s, beside a second instance finalizer | **D61-unstable**: 5 of 12 `g fin / h fin`, 7 of 12 `h fin / g fin` | one of the two, deterministically | silent | silent |
| g5 | termination finalizer that `RAISE`s, beside a second | **D61-unstable**: 8 of 12 / 4 of 12 | one of the two, deterministically | silent | silent |
| g6 | **class-side** termination finalizer forcing a collection | `start / end / class fin / inner ran inline? 0`, rc 0 | matches | silent | **fires** |
| g7 | termination finalizer allocating 200,000 times, no forced collection | `start / end / outer fin / inner ran inline? 0`, rc 0 | matches | silent | silent |
| g8 | termination finalizer stashing a new instance in a class-scope `EXPOSE` (D59) | `start / end / outer fin / k fin`, rc 0 | matches | silent | silent |

g4 and g5 are the racy shape D61 names -- two instance `UNINIT`s at termination -- and the crate
answers one of the oracle's two orders deterministically. That is not a divergence and is why neither
can be a row; they are here only for the guard question, which does not depend on the oracle.

**So: no witness found for the guard half.** That is "looked and did not find", not "unreachable" --
the secondary reading, that `run_termination_uninits` has exactly one call site
(`crates/rexx-exec/src/lib.rs:6911`, in the top-level driver after the program body and the deferred
bodies) and that nothing a Rexx program can write reaches that line a second time, is a mechanism
argument and this project's record on those is bad. **The guard is not deleted**, per the brief. What
changes is that its doc now says it has no witness, which is the honest state.

Two things worth naming that the battery turned up and no row covers:

* **g7**: an *unforced* collection inside a termination finalizer also runs no finalizer inline, on
  both sides. The interlock covers the unforced path as well as `GC('force')`.
* **g2**: `CALL` to an external `.rex` from inside a finalizer produces nothing after the finalizer's
  own output on **either** side -- the call raises and the raise is discarded, per
  `run_one_uninit`'s documented behaviour. The two agree, so it is not a divergence, but nothing in
  the corpus reaches an external routine from a finalizer.

## 5. The row

`rust/corpus/lang/uninit_nested_collection_at_exit.rex`, 36 lines, added to
`rust/corpus/phase-5b.txt` and to `EXPECTED_SUBSET_5B` in `crates/rexx-exec/tests/coverage.rs` in the
same commit, with its `crates/rexx-parse/tests/sourceline_oracle/uninit_nested_collection_at_exit.txt`
expectation captured by the documented `.Package~new` driver
(`sourceline_oracle.rs`'s module doc) run from the repository root: `count 36`, matching `wc -l`.

It is shape B: the controller's probe minus `say 'inner built'`, with the padding kept for the reason
in section 2.

### D61

Ten oracle runs of the committed file, a fresh empty directory created per run, three descriptors
hashed separately:

```
distinct stdout in 10   1  (08c75b616a09c1998a7e07efd66d6785)
stderr                  0 bytes, every run
rc                      0, every run
stdout                  start / end / outer fin / inner ran inline? 0
```

and the same stdout from `corpus/lang/`, which is the directory `tests/corpus.rs` actually runs it
from. The row prints only *whether* the inner finalizer ran inline, never when: the "when" is the
bimodal answer `run_ready_uninits`' own doc records, and `uninit_nested_collection.rex` omits it for
the same reason.

### It reddens, in both directions

The row added to the extract's `corpus/lang/` and `phase-5b.txt`, oracle reference taken from
`corpus/lang/` with `Stdio::null` standard input:

```
oracle                      rc 0, stderr 0 bytes, start / end / outer fin / inner ran inline? 0
pristine   ir, tree-walker  GREEN
MF         ir, tree-walker  GREEN
MSET       ir, tree-walker  RED   rc 0, stderr empty, ... / inner ran inline? 1
MF2        ir, tree-walker  RED   rc 0, stderr empty, ... / inner ran inline? 1
```

Mutated red, restored green, both engines, both directions.

### It adds coverage, not merely the ability to fail

Section 3 measured the arms against the 286-entry union **without** the row and found no program that
changes. With the row present the union is 287, and the same three arms give:

| arm | of the 287, the programs whose output changes |
|---|---|
| MF | none |
| MSET | `lang/uninit_nested_collection_at_exit.rex`, `ir` and `tree-walker`, and nothing else |
| MF2 | the same, and nothing else |

So the row is the **only** thing in the corpus that catches the termination sweep's copy of the
interlock, and it catches it on both engines.

## 6. The guard: what I did NOT change, and why

The review's IMP-A fix also said to "say at the `run_termination_uninits` check that the check itself
has no witness, or delete it". **I did neither**, and this is a stated assumption rather than an
oversight:

* The brief's own item 4 asks for the report to say I looked and did not find one, and says not to
  delete the guard. It does not ask for a source change.
* `global-constraints.md` says "'Record' means the report and the ledger, **not a comment in the
  source**", and `rust/CLAUDE.md` puts "no corpus row witnesses this" squarely in the class it
  forbids in a comment: a mutable in-repo aggregate that rots without the sentence being reread. A
  comment saying "this has no witness" becomes false the moment somebody writes one, silently.
* The doc at the site does not currently claim the guard *has* a witness -- it does not mention the
  interlock at all -- so there is no false sentence there to correct.

**The option I did not take unilaterally**, and which the controller may want: an in-crate
`#[cfg(test)]` test that sets `processing_uninits` and asserts `run_termination_uninits` returns
empty and runs nothing. That would convert "no witness" into "witnessed by an in-crate test only",
which is what this phase's constraint asks for where a differential row is impossible. I did not add
it because (a) it is beyond the brief, and (b) it would pin an implementation detail that MF proves
has no observable consequence, which is close to a test written only to make a mutation redden. Say
the word and it is a few lines.

# MIN-B -- the stale names

Each line read before changing it. `Heap::clear_uninit` was **deleted** by the previous round, not
renamed; `clear_uninit_all` (`crates/rexx-core/src/heap.rs:382`) carries its contract.
`ClassRegistry::take_uninit_classes_in_sweep_order` (`crates/rexx-classes/src/registry.rs:339`) is
the live name.

| file:line | said | now says |
|---|---|---|
| `plans/2026-08-27-phase-5b.md:398` | ``Heap::clear_uninit`` | ``Heap::clear_uninit_all`` |
| `plans/2026-08-27-phase-5b.md:410` | ``clear_uninit`` | ``clear_uninit_all`` |
| `plans/2026-08-27-phase-5b.md:474` | ``ClassRegistry::uninit_classes_in_sweep_order`` | ``ClassRegistry::take_uninit_classes_in_sweep_order`` |

**`specs/2026-08-27-phase-5b-instances.md:447` still says `Heap::clear_uninit` and I did not touch
it.** The review's own fix says the spec is binding and is not a task's to edit. So the previous
round's sentence "No reference to the old name is left in the workspace" is **still false**, now on
the strength of that one line alone, and closing it is a spec amendment for the controller. Hits
under `docs/superpowers/records/` and `.superpowers/sdd/` are history and are correctly left alone.

# MIN-C -- the rustdoc link that resolved to a private field

Verified at HEAD before changing: `uninit_classes` is the private field at
`crates/rexx-classes/src/class_graph.rs:262`, the public accessor is `take_uninit_classes` at `:411`,
and `check_uninit` is `pub fn`, so its doc was public documentation linking to a private item.

Re-measured at all three revisions myself, each a `git archive` extract with its own
`CARGO_TARGET_DIR`, `RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links" cargo doc -p rexx-classes
--no-deps --document-private-items`:

| revision | "links to private item" warnings | mentions `uninit_classes` |
|---|---|---|
| `d708491a7` | 8 | no |
| `9c0007723` | 9 | yes, `check_uninit` -> `Self::uninit_classes` |
| `065bceab7` (HEAD, before my fix) | 9 | yes, at `:386` |
| HEAD with the fix | **8** | **no** |

**The brief's figures reproduce exactly**, and the warning was still live at HEAD. The fix is to drop
the link brackets and name the field as plain code -- `` the `uninit_classes` list `` -- rather than
to point the link at `take_uninit_classes`, because the sentence is about *entering* a class into the
list and the taker is not what it means.

**A separate finding, not mine to fix.** The same build has **three** unresolved-link `error:` lines
at HEAD where it had two at `9c0007723`. The new one is

```
error: unresolved link to `Self::lookup_instance_method_from_scope`
   --> crates/rexx-classes/src/registry.rs:527:39
```

introduced by `f558ea501` ("Give an instance the behaviour its class held when it was built"), which
is Task 2's. `ClassRegistry::lookup_instance_method` exists at `registry.rs:460` and is the likely
intended target, but which item the sentence means is Task 2's judgement, not mine. The reviewer's
"two `error:` lines ... present at both revisions" was correct at their revision.

# MIN-D -- the void run's figure and its exemption

Re-measured from the log at
`scratchpad/task5/g5-void-d708491a7.log` (180,647 bytes, mtime 2026-08-31 14:12):

```
/bin/grep -c '^     Running'                 36
/bin/grep -c '^test result:'                 35
/bin/grep -c 'test result: FAILED'            0
/bin/grep -cE '[0-9]{2}:[0-9]{2}:[0-9]{2}'    0
tail -1   test both_engines_agree_across_every_population has been running for over 60 seconds
```

The review's figures reproduce exactly. `task-5-fix-report.md`'s sentence is corrected in place with
the correction labelled, which is the shape the previous round used for its own false sentence: the
figure becomes 35 of 36 started, and the exemption clause ("because those binaries read their inputs
and completed while the tree was pristine") is **deleted** rather than reworded, since with zero
timestamps the log cannot date any binary against the 14:06:23 write. The void ruling itself stands
and is untouched.

# MIN-E -- the plan prose the fix overtook, and two sentences nobody named

The brief warned that this project's fix-round prose rots one sentence further than the one anybody
names. It did, twice.

**The named one, `plans/2026-08-27-phase-5b.md:481`.** Measured at HEAD, oracle and crate from
`corpus/lang/`, `Stdio::null` standard input:

```
lang/uninit_class_inherited.rex  oracle rc 0, stderr 0b  main / uninit on K / uninit on P
                                 crate ir, tree-walker   identical, AGREES
lang/uninit_class_mixin.rex      oracle rc 0, stderr 0b  main / uninit on K / uninit on M
                                 crate ir, tree-walker   identical, AGREES
```

So "Both are silent wrong answers on this crate today" and both `crate:` lines are false. **The crate
lines are deleted and the oracle lines kept**, which is what the reviewer said the section actually
needs; dating them instead would have meant writing a figure for `c65b51641` that I have not
measured.

**Two more in the same section, neither named by the review nor by the brief:**

* **`:394`**, in the `**Measured.**` block: "crate stdout `main`" for `obdes.rex`. Measured at HEAD,
  `obdes.rex` agrees on both engines -- oracle and crate both `main / uninit ran`, rc 0, empty
  stderr. Rewritten to past tense as what the task *found*, cited to
  `task-5-report.md:456` and `:614` (which independently record `rust "main\n"` and "`diverge-stdout`
  at BASE"), so the sentence is a claim about the task's starting point rather than about the tree.
* **`:404`**: the block quotes `class_graph.rs`'s doc as saying "this crate has no such table". That
  sentence is no longer in `class_graph.rs` -- `/bin/grep -n "no such table"` on it answers nothing
  -- because this task built the table. The dead quotation is deleted and the claim it carried kept.

Checked and left alone in the same section: `:452`'s account of a first implementation calling
`check_uninit` only where a class is declared (history, and a plan is where history belongs);
`:478`'s "5a's Task 7 handed three class-`UNINIT` programs" (a quoted, dated statement about a
record, with the citation beside it); and the "Under D61 neither can be a corpus row until the order
question is answered" instruction, which the parenthetical immediately below it already records as
discharged.

# The commit

`3148d8cdd`, "Witness the termination sweep's copy of the UNINIT interlock", 6 files, +100/-20.
The corpus program, its `phase-5b.txt` entry and its `EXPECTED_SUBSET_5B` entry are in that one
commit, as the constraint requires, along with its `sourceline_oracle` expectation, the MIN-C doc
fix and the plan edits. Paths were staged by name; nothing was amended.

# Performance guard: no sitting, with a measurement rather than an argument

The round's only `src/` change is one doc-comment line in
`crates/rexx-classes/src/class_graph.rs`, replacing one line with one line (file stays at 1223
lines). Rather than argue the binary is unaffected, it was built both ways in the extract with the
same `CARGO_TARGET_DIR`:

```
pristine  sha256 a0b3de004df4cc4d6917ec32886a7fc1b69d4b88c6ccffc061d1b835720b915c
with fix  sha256 a0b3de004df4cc4d6917ec32886a7fc1b69d4b88c6ccffc061d1b835720b915c   cmp: identical
```

and the second build's log shows `Compiling rexx-classes` followed by `Compiling rexx-exec`, so the
compiler really re-ran rather than reusing a cached artifact. **The release binary the axes measure
is byte-identical, so a sitting would measure noise.** No row was written to
`bench-baselines/phase-5b-arms.tsv`, which stays as it was.

# Gates

All six run from `rust/` at `3148d8cdd`, each command written to its own log with its status written
to its own file and read back unpiped, none chained with `&&`.

| # | command | status |
|---|---|---|
| 1 | `cargo fmt --all --check` | **0** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| 3 | `cargo test --release --workspace` | **0** |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace` | **0** |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0** |
| phase | `REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **101**, by design while 5b rows are red |

`command -v memcap` answers `/home/moritz/.local/bin/memcap`, so gate 5 ran under it rather than
under the `ulimit -v` substitute.

**The corpus figure moved with the row**: gate 4 prints

```
rexx-exec differential corpus report -- rust/corpus/phase-4a.txt + phase-4b.txt + phase-4c.txt + phase-5a.txt + phase-5b.txt
mode: STRICT (the gate) -- REXX_CORPUS_GATE is set
287 of 287 matching
```

against 286 before this commit, and the count is read from that line rather than carried from
memory.

Gate 5's own counts, stated as measured rather than reconciled: `test result: FAILED` **0**,
`^test result:` 103, `^     Running` 92, `^   Doc-tests` 10. The 92 + 10 headers against 103 result
lines do not reconcile exactly and I did not chase it; what the gate turns on is the exit status and
the zero failures, both above. Gate 3 prints the same 103 against 0 failures.

**Phase gate detail**, from `pg.log`: `obdes` -- this task's row -- reads `agree`. Table C, 5b: 6
rows, 2 not yet `agree`, neither of them this task's. Table D, 5b: 2 rows, 0 not yet `agree`. The
101 is table C's `test result: FAILED. 13 passed; 1 failed`, which is the gate doing its job over
5c's rows.

`git status --porcelain` is empty after all six, so no gate left a stray file in the tree.

# What I did not verify

* The whole-corpus sweep was run against MF, MSET, MF2, S1, S2, `GPANIC_T` and `GPANIC_R`. I did not
  re-run the previous round's other arms (M-B, M-C, M-D, M-E, M-G).
* Whether the guard is genuinely unreachable, as opposed to unwitnessed by everything I ran. See
  section 4.
* The registry.rs:527 broken doc link is Task 2's and I did not fix or further investigate it.
* `specs/2026-08-27-phase-5b-instances.md:447` still names the deleted `Heap::clear_uninit`. The spec
  is binding and I did not edit it.

---

# Addendum: the guard-contract test (controller ruling, 2026-09-01)

The controller ruled that the `cfg(test)` test I had flagged and declined to add unilaterally should
be added, affirming the reasoning for not putting the claim in a source comment. Committed at
`f65694c8b` on top of `3290d5763`, one file, +67.

`crates/rexx-exec/src/dispatch.rs`,
`dispatch::tests::an_interlocked_termination_sweep_runs_nothing_and_consumes_nothing`.

## The contract was reachable, but not the way the ruling assumed

`run_termination_uninits` is `pub(crate)` and `Interp`'s fields are visible to `dispatch::tests`, so
calling it with the flag already set was direct. What was **not** enough was
`install_directives` alone: the un-interlocked arm panicked at
`dispatch.rs:1946`, `index out of bounds: the len is 0 but the index is 0`, on
`self.programs[installed.program.0]`. The `ProgramId` handed to `install_directives` has to name a
program in `Interp::programs`, and `Interp::run` pushes it before installing. The helper does the
same thing in the same order. That is completing the documented setup, not scaffolding around the
type, so I did not stop.

## The empty answer is deliberately not the assertion

An empty `Vec<Loud>` is what a sweep that ran everything **successfully** answers too, so a test
reading only `is_empty()` would pass over an implementation with no guard at all -- the vacuous
witness this phase has now shipped four times. What separates the two cases is whether the sweep
**consumed** its work, since `take_uninit_flagged` and `take_uninit_classes_in_sweep_order` drain.
Hence three arms:

1. the setup really leaves a class pending (else both arms below are green over an empty registry);
2. interlocked, the sweep answers empty **and** the class is still pending;
3. not interlocked, the same call consumes it.

Arm 3 is what gives arm 2 its meaning, which is this project's "pair a refusal with its adjacent
success".

## It reddens, both directions, transcripts

Run in a `git archive 3290d5763` extract with the working-tree `dispatch.rs` overlaid and its own
`CARGO_TARGET_DIR`; the live worktree was not mutated. Restores are from a saved copy, never
`git checkout --`.

```
guard intact
  test dispatch::tests::an_interlocked_termination_sweep_runs_nothing_and_consumes_nothing ... ok
  test result: ok. 1 passed; 0 failed                                          exit 0

early-return guard deleted from run_termination_uninits
  test dispatch::tests::an_interlocked_termination_sweep_runs_nothing_and_consumes_nothing ... FAILED
  test result: FAILED. 0 passed; 1 failed                                      exit 101
  panicked at crates/rexx-exec/src/dispatch.rs:7010:9:
    assertion `left == right` failed: an interlocked sweep must leave the class for
    the caller holding the flag; an empty answer here means it swept anyway
      left: 0
     right: 1

restored from the copy
  test dispatch::tests::an_interlocked_termination_sweep_runs_nothing_and_consumes_nothing ... ok
  test result: ok. 1 passed; 0 failed                                          exit 0
```

It fails on the assertion it was written for, with `left: 0` against `right: 1` -- the registry
drained, which is the sweep having run.

## Nothing else catches it

`cargo test --workspace --no-fail-fast` in the extract, both arms. The extract carries 31 failing
tests either way, because a `git archive` has no `ootest/` or `oodocs/` -- those are git-ignored
working copies at the repository root -- so the comparison is of the failing **sets**, not of a
count against zero:

```
failing tests, guard intact         31
failing tests, guard deleted        32
fail only under the deletion        dispatch::tests::an_interlocked_termination_sweep_runs_nothing_and_consumes_nothing
fail only when intact               (none)
```

One test in the whole workspace catches the guard, and it is this one.

## The two instruments are complementary, checked in both directions

| deletion | `uninit_nested_collection_at_exit.rex` | this unit test |
|---|---|---|
| the early-return guard | green (and every other corpus row, all 287, both engines) | **RED** |
| the `processing_uninits` assignment | **RED**, both engines | green |

Measured, not inferred: the unit test was run under the assignment deletion and passes, because the
arm sets the flag by hand so the guard fires either way. So neither instrument is redundant, and
between them both halves of the interlock now have a witness.

## Perf guard again: byte-identical

`#[cfg(test)]` is compiled out of a release build, and the addition is at the end of the file so no
non-test line moves. Measured rather than argued, in the extract with one `CARGO_TARGET_DIR`:

```
with the test     sha256 ace6286b5049b00184ed8914144e21949a2752ab09afb41daa2fdb47a71a3a6c
without it        sha256 ace6286b5049b00184ed8914144e21949a2752ab09afb41daa2fdb47a71a3a6c   cmp: identical
```

and the second build's log carries `Compiling rexx-exec`, so the compiler re-ran. No sitting taken,
nothing written to `bench-baselines/phase-5b-arms.tsv`.

## The three items the controller had already handled

Not redone, and re-read rather than taken on trust: `registry.rs:527` now reads
`Self::lookup_from_scope_at` and the rustdoc build's unresolved-link errors are back to the two that
predate this phase; `specs/2026-08-27-phase-5b-instances.md:447` now names `Heap::clear_uninit_all`.
The gate-5 count is resolved -- one `Doc-tests` header covers two sections, so 103 sections against
103 `test result:` lines and nothing ran silently.

## Gates at `f65694c8b`

Each status read unpiped from its own file, none chained with `&&`.

| # | command | status |
|---|---|---|
| 1 | `cargo fmt --all --check` | **0** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| 3 | `cargo test --release --workspace` | **0**, zero `test result: FAILED` |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace` | **0**, `287 of 287 matching` |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **0**, zero `test result: FAILED` |
| phase | `REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **101**, by design |

Gate 5's counts this time: `^running` sections 103 and `^test result:` lines 103, which is the
controller's resolution of last round's caveat reproducing -- every section reports and nothing ran
silently.

Phase gate: `obdes` reads `agree`. Table C 5b, 6 rows, 2 not yet `agree`, neither this task's; table
D 5b, 2 rows, 0 not yet `agree`. `git status --porcelain` empty after all six.
