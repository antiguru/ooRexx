# Task 5 fix round -- review of `d708491a7..9c0007723`

**0 Critical, 2 Important, 5 Minor.** Nothing I ran found a wrong answer in the shipped behaviour.
Every row this round adds or rewrites reddens when the behaviour it witnesses is deleted, which is
the thing the brief asked for above all others. The two Importants are about the round's record, not
its code: the termination sweep's copy of the re-entrancy interlock has no corpus witness, and the
fix report ends mid-sentence with no gate status for the round. Verdicts and findings are at the
bottom.

Reviewer working copy: a `git archive 9c0007723` extract under my own scratch dir with its own
`CARGO_TARGET_DIR`. Nothing was written to the live worktree except this file. Written first and
appended as the work went, per the global constraints.

## Probe convention

Every oracle transcript below: a **fresh empty directory created per run** with `mktemp -d`, absolute
program path, `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -s KILL 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`,
stdout, stderr and exit status captured to three separate files and printed separately, never `2>&1`.
Crate transcripts: the release `rexx-run` built from `9c0007723` in my own `CARGO_TARGET_DIR`,
`REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`, same fresh-directory rule, `timeout -s KILL 20`.
Probe sources are under my scratch at `t5fr/progs/`, never in the run directory.

None of the probes below is one of `rust/corpus/oracle-crashes.txt`'s eight shapes.

## 1. The three fixed behaviours, against my own probes

All eight probes are mine; none is the report's. Every cell is rc 0 with **empty stderr** on all
three of oracle, crate `ir` and crate `tree-walker`, and stdout is byte-identical across the three.

| probe | what it is | oracle stdout | crate ir | crate tree-walker |
|---|---|---|---|---|
| `p1_directive_class` | `::class mid inherit base` plus `::class leaf subclass mid`, class-side `UNINIT` on `BASE` only | `main / fin BASE / fin LEAF / fin MID` | same | same |
| `p2_directive_instance` | instance-side `UNINIT` reached through a directive `INHERIT` | `start / built WIDGET / instance fin / after-gc` | same | same |
| `p3_runtime_class` | `.alpha~inherit(.beta)`, class-side | `main / inherited / fin ALPHA / fin BETA` | same | same |
| `p4_runtime_instance` | `.holder~inherit(.trait)` then `~new`, `drop`, `gc 'force'` | `start / inherited / built HOLDER / instance fin / after-gc` | same | same |
| `p5_uninherit_class` | `~inherit` then `~uninherit`, with an `::method unknown class` on the receiver | `main / inherited / uninherited / fin BETA` | same | same |
| `p6_alloc_limit5` | finalizer allocating one more instance up to a self-imposed limit of **5** | `start / end / f 1 / f 2` | same | same |
| `p7_alloc_unbounded` | finalizer that **always** allocates, no limit | `start / end` | same | same |
| `p8_nested` | `GC('force')` inside a finalizer, printing only whether the inner finalizer ran inline | `start / built / outer fin / inner built / inner ran inline? 0 / end` | same | same |

Oracle runs: three each for `p1`-`p6`, two for `p7`, **twelve for `p8`** (one distinct stdout in
twelve, one distinct stderr).

**CRIT-1 agrees, and on more than the brief asked.** `p1` is the brief's "fires once per class that
has it": `BASE` declares the class-side `UNINIT`, `MID` inherits it through the directive and `LEAF`
gets it through the subclass cascade, so three classes fire where one declares. That exercises the
cascade the fix inherits by sitting in `update_sub_classes`, which no probe in either the review or
the fix report covers. `p2`, `p3` and `p4` are the other three arms CRIT-1 named; all four agree.

**CRIT-1b agrees.** `p5`'s receiver carries `::method unknown class`, so a sweep that sent `UNINIT`
without testing first would print an `unknown UNINIT` line. Neither side prints one, and `BETA` still
fires.

**CRIT-2 agrees, in both directions.** `p6` uses a limit of 5, which neither the review nor the report
used (they used 8 and 4), and both sides stop at `f 2`, so the count is a property of the two passes
and not of the limit. `p7` is the non-terminating case from the review: it now exits rc 0 with
`start / end` on both engines, matching the oracle. Before this round it was rc 137 with empty stdout.

**IMP-1 agrees, and its answer is stable.** `p8`'s `inner ran inline? 0` was identical in twelve oracle
runs and on both engines.

## 2. Every new corpus row, checked by deleting the behaviour it witnesses

Not a sample: the four rows this round adds, plus the row it rewrote, plus the four `UNINIT` rows and
`obdes` that were already there, under **nine** mutations. Each arm is the `9c0007723` extract with
one edit, `touch`ed, rebuilt with `CARGO_TARGET_DIR=<mine> cargo build --release --bin rexx-run`, and
the binary compared against the saved pristine one (`cmp` reported "changed" on every arm, so no arm
measured a stale binary). Each row is then run under both engines and compared against the oracle on
all three descriptors -- which is what `tests/corpus.rs` itself does, live, per row.

A cell is `RED` where the crate's three descriptors stopped matching the oracle's, blank where the row
stayed green.

| row | M-A | M-B | M-C | M-D | M-E | M-F | M-F2 | M-G | M-H |
|---|---|---|---|---|---|---|---|---|---|
| `uninit_class_mixin` (rewritten) | **RED** | | | | **RED** | | | **RED** | |
| `uninit_class_inherit_runtime` (new) | **RED** | | | | **RED** | | | **RED** | **RED** |
| `uninit_class_uninherit` (new) | | **RED** | | | | | | **RED** | |
| `uninit_allocating_finalizer` (new) | | | **RED** | | | | | | |
| `uninit_nested_collection` (new) | | | | **RED** | | | | | |
| `uninit_instance_collected` | | | | | | | | | |
| `uninit_instance_retained` | | | | | | | | | |
| `uninit_class_inherited` | | | | | **RED** | | | **RED** | |
| `uninit_class_sweep_order` | | | | | **RED** | | | **RED** | |
| `obdes` | | | | | | | | **RED** | |
| *(my probe `p18`, not a corpus row)* | | | | | | | **RED** | | |

The mutations:

* **M-A** -- `self.check_uninit(class);` deleted from `ClassGraph::update_sub_classes` (`class_graph.rs:580`). This is CRIT-1's whole fix.
* **M-B** -- the `if !self.answers_uninit(object) { return None; }` guard deleted from `Interp::run_one_uninit`. This is CRIT-1b's whole fix.
* **M-C** -- `const SWEEPS: usize = 2;` changed to `1`. This is half of CRIT-2's fix; the other half (the `loop` becoming a bounded `for`) is what M-C cannot express, and my `p7` covers it.
* **M-D** -- the `if self.processing_uninits { return Vec::new(); }` guard deleted from `Interp::run_ready_uninits`, the `= true` kept. This is IMP-1's fix.
* **M-E** -- `sweep.sort_by_key(...)` deleted from `ClassRegistry::take_uninit_classes_in_sweep_order`.
* **M-F** -- the same guard deleted from `Interp::run_termination_uninits`, the `= true` kept.
* **M-F2** -- the guard **and** the `= true`/`= false` deleted from `Interp::run_termination_uninits`.
* **M-G** -- `ClassGraph::take_uninit_classes` returns `self.uninit_classes.clone()` instead of `std::mem::take`, so the sweep no longer drains. This is the other half of CRIT-2's fix.
* **M-H** -- `sweep.reverse();` inserted before the (stable) `sort_by_key`, which reverses the order of entries inside one bucket and changes nothing else.

**Every row this round adds or rewrites reddens under the deletion of the behaviour it witnesses.**
IMP-2's specific failure -- a row that stayed green when its subject was deleted -- does not recur:
`uninit_class_mixin` is red under M-A, where the version it replaced was green. That is the one thing
the brief asked for above all others, and it holds for all five rows.

Two things the mutation set says that the report's does not.

**The runtime row is not redundant, and M-H is the mutation that shows it.** The report's argument for
keeping `uninit_class_inherit_runtime.rex` beside `uninit_class_mixin.rex` is that "C3 reddens the
runtime row over an ordering the directive row cannot express" -- but the report's own C3 row lists
*both* among the rows it reddens, and my M-E reproduces that (both, plus `class_inherited` and
`class_sweep_order`). So the report's stated argument is not carried by any arm the report ran: in
every one of its ten arms the two rows redden together. M-H separates them. It is the one mutation in
the whole set that reddens exactly one row, and that row is the runtime one -- the tie-break inside a
shared bucket is a property no other corpus program can see. The conclusion the report reaches is
right; the evidence it gives for it is not, and the evidence is above.

**The termination half of the interlock is load-bearing and no corpus row can see it.** M-F2 leaves
all ten `UNINIT` rows green and every probe of mine green except `p18`, which is
`uninit_nested_collection.rex`'s shape moved to the *termination* sweep -- the outer finalizer runs
because the program ends, not because the program called `GC('force')`:

```
p18: say 'start' / o = .outer~new / say 'end' / exit 0
     ::class outer ::method uninit -> builds a .inner, drops it, calls gc 'force',
                                     then prints whether INNER's finalizer ran inline
oracle, ten runs of ten   rc 0  stderr empty  start / end / outer fin / inner built / inner ran inline? 0
crate at 9c0007723, both engines                         identical
crate under M-F2, both engines                           ... / inner ran inline? 1
```

`uninit_nested_collection.rex` drives its outer finalizer from a program-level `GC('force')`, so it
enters through `run_ready_uninits` and witnesses only that function's copy of the flag. The copy in
`run_termination_uninits` is a second delivery point with the same silent failure mode, and nothing in
the corpus reddens when it goes. See IMP-A below.

## 3. IMP-3, the stale-binary measurement: re-measured, and the correction holds

The fix round's own report carries **no** performance figures. The corrected table lives in
`task-5-report.md:514`-`:537` (mtime 15:11, after the review), marked "CORRECTED at the fix round".
That is the claim I re-measured.

Both revisions extracted with `git archive` into their own trees, each checked file-by-file against
`git show REV:path` by `sha1sum` before building (`heap.rs` and `dispatch.rs` both matched), built
debug with their own `CARGO_TARGET_DIR`, binaries timestamped 22:51:53 and 22:51:58 against sources
timestamped 11:22 and 11:39, so neither binary predates its source. Arms **alternated inside one
loop**, `REXX_ENGINE=ir`, `/usr/bin/time -f %e`, `/proc/loadavg` 8-10 throughout.

Program: `do i = 1 to N ; o = .K~new ; end ; say 'main'` with `::CLASS K` / `::METHOD uninit` / `nop`.
Control: the same program with the method named `other` instead of `uninit`.

| N | `3ff1055de` (pre-fix) | `cf85bfd67` (post-fix) | the corrected table says |
|---|---|---|---|
| 16,000 | 2.53 / 2.52 / 2.53 | 0.29 / 0.29 / 0.30 | 2.57 x3 and 0.30 x3 |
| 16,000, class with no `UNINIT` | 0.20 / 0.20 / 0.20 | 0.20 / 0.20 / 0.20 | (control at 200,000: 1.34 and 1.29) |
| 64,000 | 36.55 / 36.62 | 0.86 / 0.87 | 37.01 / 37.07 / 36.93 and 0.86 x3 |

**The corrected column reproduces**, at both sizes and on both arms, and the control is flat. Three
independent measurements now agree on the pre-fix cells (the review's 2.61/2.58/2.62 and 37-45, the
corrected report's 2.57 and 37.0, mine) where the original table's 0.30 and 0.86 agreed with none of
them. IMP-3 is closed.

Two things the corrected table does that are worth naming as done properly: it says in the table
which cells are its own and which are the reviewer's rather than presenting one column, and it says
why the 128,000 and 200,000 rows were not re-taken. I did not re-take those either.

## 4. D61: every new row is stable on the oracle, and byte-identical on the crate

Ten oracle runs of each new row, from a fresh empty directory each time, three descriptors hashed
separately:

| row | distinct stdout in 10 | stderr | rc |
|---|---|---|---|
| `uninit_class_mixin` | 1 (`89c37809`) | 0 bytes, every run | 0 |
| `uninit_class_inherit_runtime` | 1 (`c97398d6`) | 0 bytes | 0 |
| `uninit_class_uninherit` | 1 (`aae3f1dc`) | 0 bytes | 0 |
| `uninit_allocating_finalizer` | 1 (`2ef3fd2b`) | 0 bytes | 0 |
| `uninit_nested_collection` | 1 (`959d821b`) | 0 bytes | 0 |

Five crate runs of each row on **each** engine give the same five stdout hashes and an empty stderr,
so the agreement is byte-for-byte and neither side varies.

None of the five prints the order of two `UNINIT`s that D61 leaves unreproducible.
`uninit_nested_collection.rex` prints only *whether* the inner finalizer ran inline, which the report
says and which its ten stable oracle runs confirm; `uninit_allocating_finalizer.rex` prints a
sequence `u 1` / `u 2` that is generation order inside one chain and not table order; the three class
rows print class-against-class order only, which D60 characterises. I found nothing in the five that
depends on an order the oracle does not reproduce.

**One shape that is not stable, and is not a row.** My own `p16` -- an instance finalizer that
registers a new class with `.late~inherit(.mx)` while the sweep is running -- answered
`trigger fin / fin MX / fin LATE` in two oracle runs of three and `fin MX / trigger fin / fin LATE` in
the third. That is the instance-against-class mixing `corpus/README.md` forbids, and it is the reason
it cannot be a row. Both sides do run `fin LATE`, so a class registered *during* the sweep is still
reached; the crate answers the majority order deterministically, ten runs of ten across both engines.
Recorded because the two-pass design makes that reachable and no row covers it.

## 5. The two void runs

I read the whole fix report looking for a figure taken from either. **Exactly one figure comes from
the void gate 5, and it is loudly labelled**: "36 test binaries reported, zero `test result: FAILED`,
with `corpus_differential ... ok`". Every other measurement in the report is either a C++ source
reading, a fresh oracle probe, or one of the `git archive` mutation arms with its own
`CARGO_TARGET_DIR`. The mutation table's counts check out independently: the union of
`corpus/phase-*.txt` at `9c0007723` is **281** entries, and the task's ten rows leave **271**, which
are the report's two column figures.

Two problems with that one figure, both in MIN-D below: the log has 36 `Running` lines and **35**
`test result:` lines, not 36, and it carries no timestamps at all
(`/bin/grep -cE "[0-9]{2}:[0-9]{2}:[0-9]{2}"` answers 0), so the report's reason for exempting those
binaries -- "those binaries read their inputs and completed while the tree was pristine" -- is not
something the log can be read to say.

The mutation arms were taken at `e43d2c228` rather than at the head of the range. That is sound:
`9c0007723` is `git show --stat`-confirmed to touch only
`docs/superpowers/plans/2026-08-27-phase-5b.md` and `rust/corpus/README.md`, so it cannot move a
mutation arm. My own arms were all taken at `9c0007723`.

## "Adds coverage", checked over the whole corpus for one arm

The row table in section 2 is over the ten `UNINIT` rows. For **M-A**, CRIT-1's fix, I ran the union
of `corpus/phase-*.txt` -- 281 programs -- under both engines against the oracle, three descriptors
each, from a fresh directory per program:

```
# 281 programs (the union of corpus/phase-*.txt) x 2 engines, 12-way parallel,
# each from a fresh empty directory, three descriptors compared
DIVERGE lang/uninit_class_inherit_runtime.rex [ir] STDOUT
DIVERGE lang/uninit_class_inherit_runtime.rex [tree-walker] STDOUT
DIVERGE lang/uninit_class_mixin.rex [ir] STDOUT
DIVERGE lang/uninit_class_mixin.rex [tree-walker] STDOUT
```

Exactly the two rows, and **nothing else in the corpus catches CRIT-1's fix** -- so those two rows
earn their place rather than merely being able to fail. The same sweep on the pristine binary reports
only `lang/pull_queue.rex`, which is my harness giving a `PULL` program no stdin and not a defect;
it does not appear under M-A, so it is intermittent under parallel load either way.

**Not verified by me:** the equivalent whole-corpus sweep for the other eight arms. For those I
checked the ten `UNINIT` rows plus my own probes only, and the report's `reduced` column is the
only evidence that nothing else in the corpus catches them.

---

# Verdicts

**Spec compliance: PASS.** CRIT-1, CRIT-1b, CRIT-2 and IMP-1 all agree with the oracle on probes I
wrote myself, on both engines, byte for byte on all three descriptors. CRIT-1 agrees on a shape
neither the review nor the report tested -- a subclass of the class that inherits the mixin, which
the fix's placement inside `update_sub_classes` picks up through the cascade. CRIT-2's
non-terminating case now exits rc 0 with the oracle's own output. IMP-2's replacement row reddens
when its subject is deleted, where its predecessor did not. D61 holds for every new row.

**Quality: Changes requested, on the record rather than on the code.** No Critical. Two Important,
both about what the round wrote down rather than what it built; five Minor. **Nothing I ran found a
wrong answer in the shipped behaviour.**

Counts: **0 Critical, 2 Important, 5 Minor.**

| id | severity | subject |
|---|---|---|
| IMP-A | Important | the termination half of the interlock is load-bearing and no corpus row can see it |
| IMP-B | Important | the fix report ends mid-sentence: no gate status for the round, and IMP-3 has no section |
| MIN-A | Minor | the runtime row's non-redundancy is not carried by any arm the report ran |
| MIN-B | Minor | "no reference to the old name is left in the workspace" is false |
| MIN-C | Minor | a new rustdoc warning: `check_uninit`'s doc link now resolves to a private field |
| MIN-D | Minor | the void run's figure is 35, not 36, and its exemption is not readable from the log |
| MIN-E | Minor | the plan section the round edited records a crate output that is now false |

## IMP-A -- the termination sweep's half of the interlock is load-bearing and unwitnessed

`Interp::run_termination_uninits` got the same `processing_uninits` check-and-set as
`run_ready_uninits`. The corpus witnesses only the second one.

Deleting the check **and** the set from `run_termination_uninits` (M-F2) leaves all ten `UNINIT` rows
green, `obdes` included, and leaves every probe of mine green except one:

```
p18   say 'start' / o = .outer~new / say 'end' / exit 0
      ::class outer's UNINIT builds a .inner, drops it, calls gc 'force',
      then prints whether INNER's finalizer ran inline

oracle, ten runs of ten   rc 0  stderr empty
   start / end / outer fin / inner built / inner ran inline? 0
crate at 9c0007723, ir and tree-walker, five runs each   identical
crate under M-F2, ir and tree-walker                     ... / inner ran inline? 1
```

`uninit_nested_collection.rex` drives its outer finalizer from a program-level `GC('force')`, so it
enters through `run_ready_uninits` and sees only that function's flag. `p18` differs by one line --
no program-level `GC('force')`, the finalizer runs because the program ends -- and it is the shape
that reaches the other delivery point. It is stable on the oracle and deterministic on both engines,
so it is a legitimate row.

Deleting only the *check* and keeping the set (M-F) reddens nothing I ran, `p18` included, which says
the set is the load-bearing half and the check guards a re-entry into the termination sweep that I
could not construct. That is a defensible line to keep for symmetry with `runUninits`, but it should
say at the site that it has no witness rather than read as though it does.

This phase's global constraint 2 is "a witness must be run against a control that makes it fail ...
record the control as run, with its transcript, or the acceptance is not met." The round's C9 is
"the interlock removed" and reddens `nested_collection`; that control covers one of the two copies.

**Fix**: add `p18`'s shape to `corpus/lang/` and to `corpus/phase-5b.txt` and
`EXPECTED_SUBSET_5B`, with M-F2 recorded as its control; and say at the `run_termination_uninits`
check that the check itself has no witness, or delete it.

## IMP-B -- the fix report has no gate status for the round, and stops mid-sentence

`task-5-fix-report.md` ends:

> A green line from it would have been indistinguishable from a real one at a glance, and it would
> have replaced the expected 101 with a 0 -- the more attractive number. The gate line below is from
> the documented command with `REXX_CORPUS_GATE=1` and `--no-fail-fast`.

There is no gate line below it. The file ends there. So the round's record carries **no** exit status
for any of the five gates or for the phase gate, after ruling two earlier runs void -- which is
exactly the state the void ruling was supposed to be recovered from.

The same section is missing for **IMP-3**, which is named in the report's scope line and in its edit
table ("re-take the pre-fix column interleaved, rebuilt between arms, binaries checked") and has no
section. The work was in fact done, and done well, but it landed in `task-5-report.md:514`-`:537`
(mtime 15:11) and the fix report never says so. A reader of the fix report alone concludes IMP-3 was
dropped.

I re-measured IMP-3 myself and it reproduces (section 3). The gate status is separately established:
the controller reports all five gates green at `2d5614e63`, a superset of this round, and I did not
re-run them.

**Fix**: finish the sentence with the gate lines actually run, and add a line saying where IMP-3's
re-take lives.

## MIN-A -- the runtime row's non-redundancy is argued, not measured

The report keeps `uninit_class_inherit_runtime.rex` beside `uninit_class_mixin.rex` on this argument:

> The directive row and the runtime row are one mutation apart from each other and neither is
> redundant: C3 reddens the runtime row over an ordering the directive row cannot express

But the report's own C3 row lists *both* among the rows it reddens, and my M-E reproduces that: the
bucket sort deleted reddens `class_mixin`, `class_inherit_runtime`, `class_inherited` and
`class_sweep_order` together. In all ten of the report's arms and in eight of my nine, the two rows
redden together and never apart, so no arm either of us ran separates them.

**The conclusion is right and I found the arm that shows it.** M-H -- `sweep.reverse()` inserted
before the stable `sort_by_key`, which reorders entries inside one bucket and changes nothing else --
reddens `uninit_class_inherit_runtime.rex` and **nothing else in the ten**. It is the only row whose
`QQ` and `ZED` collide in a bucket with `QQ` entered later, so it is the only row that can see the
tie-break.

**Fix**: replace the sentence with M-H, which is a control that actually separates the two rows.

## MIN-B -- "no reference to the old name is left in the workspace" is false

The report's MIN-7 paragraph ends "No reference to the old name is left in the workspace." Run over
the workspace rather than over `rust/crates/*.rs`:

```
/bin/grep -rnw "clear_uninit" --include=*.rs --include=*.md rust/ docs/
  docs/superpowers/specs/2026-08-27-phase-5b-instances.md:447
  docs/superpowers/plans/2026-08-27-phase-5b.md:398
  docs/superpowers/plans/2026-08-27-phase-5b.md:410
  (plus four hits in docs/superpowers/records/, which are history and fine)
```

The plan is a live document this very round edited twice, and the spec is binding. The same round's
own rename leaves a second one:

```
/bin/grep -rn "uninit_classes_in_sweep_order" --include=*.rs --include=*.md rust/ docs/ \
    | /bin/grep -v take_
  docs/superpowers/plans/2026-08-27-phase-5b.md:474
```

The claim is true of the pattern that was almost certainly run (`--include=*.rs` under
`rust/crates/`) and false of the set it names. **Fix**: either narrow the sentence to the pattern, or
update `plans/...:398`, `:410` and `:474`. The spec is binding and is not a task's to edit.

## MIN-C -- a rustdoc link now resolves to a private field, and no gate sees it

`crates/rexx-classes/src/class_graph.rs:392`, inside `check_uninit`'s doc:

```
/// [`Self::uninit_classes`] when its **class** behaviour answers `UNINIT`
```

The round renamed the public accessor `uninit_classes()` to `take_uninit_classes()`, so
`Self::uninit_classes` no longer names the accessor; it now resolves to the private **field** of the
same name. Measured, both trees extracted and documented with their own `CARGO_TARGET_DIR`:

```
RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links" cargo doc -p rexx-classes --no-deps --document-private-items
  d708491a7   8 "public documentation for ... links to private item" warnings, none about uninit_classes
  9c0007723   9, the extra one being
     warning: public documentation for `check_uninit` links to private item `Self::uninit_classes`
        --> crates/rexx-classes/src/class_graph.rs:392:11
```

This is the hazard the global constraints name -- a doc link is not a compile error and no gate sees
it -- in its other form: the link still resolves, to the wrong item. **Fix**:
`[`Self::take_uninit_classes`]`, or name the field explicitly.

(The two `error:` lines in the same build, `MethodDict::merge` at `registry.rs:444` and
`MethodDict::replace_methods_from` at `:670`, are present at both revisions and are not this round's.)

## MIN-D -- the void run's figure is 35, not 36, and its exemption is not readable from the log

`scratchpad/task5/g5-void-d708491a7.log`:

```
/bin/grep -c '^     Running'      36
/bin/grep -c '^test result:'      35
/bin/grep -c 'test result: FAILED' 0
tail -1                            test both_engines_agree_across_every_population has been running for over 60 seconds
/bin/grep -cE "[0-9]{2}:[0-9]{2}:[0-9]{2}"   0
```

So 36 binaries were started and **35** reported, the missing one being `ir_dual`, which is what the
report itself says was still running. "36 test binaries reported" should be 35.

More importantly, the report exempts those results from its own void ruling on the ground that "those
binaries read their inputs and completed while the tree was pristine". The log carries no timestamps,
so there is nothing in it that says which binaries finished before 14:06:23; `corpus_differential`
in particular reads `phase-5b.txt` at run time, which is the exact reason the run was ruled void.

The ruling itself is right and the figure is loudly labelled as not a gate line, so this is a Minor.
**Fix**: drop the sentence that exempts them, or say the exemption is an inference the log cannot
support.

## MIN-E -- the plan paragraph the round edited now records a state that is false at both ends

`docs/superpowers/plans/2026-08-27-phase-5b.md:481` still opens "Both are silent wrong answers
on this crate today", and the round updated the second block's crate line to `main / uninit on M`.
That output was the crate's between `d708491a7` and `b4266cbd6` only: at the plan's own "today"
(`c65b51641`) it was `main`, and at `9c0007723` both arms agree with the oracle -- measured, my `p1`
and the committed rows. The first block's `crate: main` is now false as well.

The parenthetical two paragraphs down does say both are corpus rows now, so a careful reader
recovers. **Fix**: this is the boundary prose that rots; either date the block ("at
`c65b51641`, before this task") or delete the crate lines and keep the oracle ones, which is what the
section actually needs.

## Things I checked that hold

* **MIN-2's transcript in `uninit_instance_collected.rex`'s comment reproduces exactly**: as
  committed and with the `pad` line deleted, both `start / built K / uninit ran / after-gc`; with the
  `say 'built'` clause deleted too, `start / after-gc / uninit ran`. Oracle, rc 0, empty stderr.
* **MIN-3's measured comment in `uninit_bucket` reproduces exactly**:
  `say c2x(.Object~subclass('<0xE9>A')~id)` is `E941` on the oracle and `EFBFBD41` on both engines.
* **MIN-5's documented grep runs**, from `rust/corpus/` in the real worktree, and names the six
  `getHashValue` overrides the README's paragraph relies on. The relative path `../../../ooRexx/`
  resolves there (it does not in a `git archive` extract, which is my sandbox and not a defect).
* **MIN-6's de-duplication kept the assertion**: `crates/rexx-classes/tests/uninit_sweep_order.rs`
  passes, including the new `a_second_sweep_of_the_same_registry_is_empty`, and
  `crates/rexx-core/tests/uninit.rs` passes.
* **`coverage.rs` and `sourceline_oracle.rs` pass** on my copy, and `sourceline_oracle.rs` is not
  blind to the new files: it `read_dir`s `corpus/lang` and **panics** on a program with no
  expectation file, so the four new `.txt` files are actually read.
* **The drain is well witnessed.** M-G -- `take_uninit_classes` cloning instead of taking -- reddens
  all six class rows including `obdes`.
* **`cargo fmt --all --check` exits 0** on the extract.
* **The corrected false sentence in `task-5-report.md:631` is accurate**, is labelled as a
  correction, and states the weaker claim that is true.
* **Probing past the rows found no new silent answer.** A class-side finalizer that allocates an
  instance (`main / maker fin / thing fin`), a three-class allocation chain (`a1 fin / a2 fin`, and
  `a3` never), an instance built between an `~inherit` and its `~uninherit` (no finalizer), a private
  `::method uninit` and a private `::method uninit class` (neither fires), a class-side `UNINIT` that
  comes from a metaclass's *instance* methods (fires), and a finalizer that raises inside itself
  (`aone fin / btwo fin`, rc 0, empty stderr) all agree on both engines, oracle-stable where I
  checked stability. `~setMethod('UNINIT', ...)` from inside a method is a shape the oracle answers
  and this crate refuses **loudly** at rc 120 ("method \"SETMETHOD\" of class \"Object\" is not
  implemented (Phase 5)"), which is licensed, not a silent wrong answer -- worth naming because
  `Interp::answers_uninit` resolves only class-level methods and would be wrong for it if the message
  ever lands.

## What I did not verify

* The five gates and the phase gate for this round. The brief says not to; the controller reports
  them green at `2d5614e63`.
* The whole-corpus sweep for eight of my nine mutation arms (M-A only, above).
* The report's C1-C5 and C10 arms from the first round, which this round re-ran; I ran my own
  nine instead.
* The 128,000 and 200,000 cells of the quadratic table, for the reason its own text gives.
* Anything in a debug build. Every crate transcript above is release.
