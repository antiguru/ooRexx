# Task 16 report: `REPLY` and `GUARD` inside a method

## What I implemented

`GUARD` and `REPLY` run rather than being refused, on both engines. `instruction_owner`
answers `None` for both variants and `tests/owners.rs` carries them as `Owner::InScope`.

**`GUARD`** (`Interp::exec_guard`, `run.rs`). A `GUARD` outside a method invocation raises
99.911. Inside one, `GUARD ON` and `GUARD OFF` with no `WHEN` are no-ops: the instruction
reserves and releases the receiver's scope against other activities, this interpreter runs
one, and an uncontended reservation is a no-op. `GUARD ... WHEN expr` evaluates its
expression through the same `eval_condition` `IF` and `WHEN` use, with `GUARD`'s own
`>K>   "WHEN" => ...` line and `GUARD`'s own 34.902 for a value that is not exactly `0` or
`1`; if it holds, that is also a no-op. If it does not hold, the refusal is loud, because
the oracle blocks there for ever.

**`REPLY`** (`Interp::exec_reply`, `run.rs`). A `REPLY` outside a method raises 99.919. A
second `REPLY` in one invocation raises 98.935, after its own expression has been evaluated
and traced, which is where `RexxActivation::reply` asks the question. Otherwise the value's
`>>>` line and rooting are `RETURN`'s, and the instruction answers `Flow::Return(value)`
with `ReplyState::Owed` and a `pc` on the next clause set on the activation.

**Suspension** (`Interp::park_reply`, `Interp::resume_reply`,
`Interp::run_deferred_replies`, `dispatch.rs`). `enter_method_body` reads `ReplyState::Owed`
and, instead of releasing the activation, copies its frame's slots out, hands them and the
call context's own object handles to `RootSet::park`, releases the frame, and queues the
activation on `Interp::deferred`. `execute` drains that queue after the main body's report
is written and its exit status settled; each resume pushes a frame of the same length,
writes the slots back, releases the parked anchor, and runs the rest of the body. A body
queued by a resumed body is run too, which is why the queue is drained rather than iterated.

**Legality after a reply** (`Interp::returned_value`, `run.rs`). A `RETURN` carrying a value
raises 98.936 and an `EXIT` carrying one raises 98.937, both after the value's own trace
line, which is the order the C++ takes. The bare forms of either stay legal.

**Trace.** A resumed body announces a second `>I>` exactly when the first half announced
one, which is `RexxActivation::run`'s reply arm
(`traceEntryAllowed = traceEntryDone; traceEntryDone = false`,
`execution/RexxActivation.cpp:562`-`:563`, then the unconditional `traceEntry()` at `:600`).

**Root set** (`rexx-core`). `RootSet` gains a `parked` category with `park`/`release`,
`live_parked` and `frame_aliases`. A slot frame nests -- `pop_slots` asserts that the frame
it closes is the top one -- so an activation whose body is to continue later cannot keep its
frame open while the activations above it close, and the parked category is where its values
stay reachable in between.

## Which of my code is legality and which is scheduling

For Phase 6, marked in the code as well as here. **Fix round 1 finished the marking**: the
first version of this list was right about the split and wrong to claim the code already
carried all of it, and two of the four functions it named as saying SCHEDULING did not. Every
marker now in the tree, located with `grep -n 'SCHEDULING\|LEGALITY'` and resolved to the item
it documents -- the site list is the claim, not a count of it, since `Activation::reply`'s own
marker wraps across two lines and a `grep -c` line count cannot see that:

```
crates/rexx-core/src/roots.rs       RootSet::park (naming the whole parked group),
                                     RootSet::frame_aliases
crates/rexx-exec/src/activation.rs  Activation::reply, ReplyState::Owed,
                                     Activation::object_roots
crates/rexx-exec/src/dispatch.rs    park_reply, run_deferred_replies, resume_reply
crates/rexx-exec/src/lib.rs         Interp::deferred, Loud::guard_when_false,
                                     Loud::reply_inside_construct,
                                     Argument::object_roots, CallContext::object_roots
crates/rexx-exec/src/run.rs         exec_guard, exec_reply (both kinds),
                                     returned_value's check, raised_guard_not_logical
```

Every item in the two lists below sits at one of those sites.

**Legality, and Phase 6 must keep it:**

* `Interp::exec_guard`'s `method_identity.is_none()` check (99.911).
* `Interp::exec_reply`'s `method_identity.is_none()` check (99.919).
* `Interp::exec_reply`'s `ReplyState` check (98.935), and its position *after* the
  expression is evaluated.
* `Interp::returned_value`'s `ReplyState` check (98.936/98.937), and its position after the
  value's own trace line.
* `raised_guard_not_logical` (34.902) and the `>K>   "WHEN"` echo -- the `WHEN` expression is
  evaluated and traced whatever a scheduler then does with the answer.
* `ReplyState` itself, whose `None`/`Issued` distinction is what all three of the numbered
  checks read.

**Scheduling, and Phase 6 may rewrite all of it:**

* `Interp::deferred`, `Interp::park_reply`, `Interp::resume_reply`,
  `Interp::run_deferred_replies` -- the whole "run it after the main program, in reply
  order" model. Its own doc comments say SCHEDULING.
* `ReplyState::Owed` and the branch in `enter_method_body` that reads it. A real second
  activity would not park anything.
* `Loud::guard_when_false` -- a wait for another activity.
* `Loud::reply_inside_construct` and `top_level_clause` -- a construct's state cannot be
  restored from an instruction index, which is an artifact of resuming rather than of the
  language.
* `RootSet::park`/`release`/`live_parked`/`frame_aliases` and
  `Activation::object_roots`/`CallContext::object_roots` exist only because a suspended
  activation has no frame. A design that keeps the frame open needs none of them.
* The `>I>`/`<I<` handling in `resume_reply` follows the C++ line for line, but the C++ line
  it follows is on the *reply resume* path, so it moves with the scheduling.
* The no-op-ness of `GUARD ON`/`GUARD OFF` themselves.

## What I tested

### The two transcripts in the brief, both engines, three descriptors read separately

The harness ran the oracle wrapped in `( ulimit -v 1048576; LD_LIBRARY_PATH=... timeout -s
KILL 10 build/bin/rexx <abs> )` and the crate as `( memcap 1G env REXX_ENGINE=<e> timeout -s
KILL 10 target/release/rexx-run <abs> )`, from a fresh empty directory, and diffed stdout,
stderr and exit status separately. `AGREE` means all three matched on both engines.

```
method_guard_instruction.rex AGREE rc=0
method_guard_outside_method.rex AGREE rc=157
method_reply.rex AGREE rc=0
method_reply_outside_method.rex AGREE rc=157
method_reply_twice.rex AGREE rc=0
method_reply_no_result.rex AGREE rc=165
method_reply_exit_status.rex AGREE rc=7
method_reply_chain.rex AGREE rc=0
```

`method_reply.rex` is the exit-status-0-with-a-traceback shape. Its actual bytes, from the
crate, both engines:

```
  ir           rc=0 stdout=[replied|after|] stderr=[    15 *-* return 'returned'|Error 98 running .../method_reply.rex line 15:  Execution error.|Error 98.936:  RETURN cannot return a value after a REPLY.|]
  tree-walker  rc=0 stdout=[replied|after|] stderr=[    15 *-* return 'returned'|Error 98 running .../method_reply.rex line 15:  Execution error.|Error 98.936:  RETURN cannot return a value after a REPLY.|]
```

Other shapes checked the same way and agreeing on all three descriptors, both engines, not
added to the corpus: `guard off`/`on`/`off`, `guard on when <true>`, `guard off when
<true>`, a bad `WHEN` value (34.902, rc 222), `guard on`/`reply` in the main body and in a
`::ROUTINE`, a bare `reply`, `reply` as a method's last clause, `reply` with `USE ARG`
arguments read back in the resumed half, `reply` in a method called from a `::ROUTINE`, a
reply chained out of an owed body, an exposed variable written before and read after a
reply, `exit <value>` after a reply (98.937), and a `trace r` transcript carrying the
second `>I>`.

### Determinism, and why some measured shapes are not corpus rows

The oracle's own answer for a replied body is racy whenever the main body produces output
*after* the send. 25 runs each of the eight corpus programs gave one distinct
(rc, stdout, stderr) signature apiece; `say .K~m` / `say 'main-end'` over a replying method
gave two orders; and two class methods each replying gave two orders in 30 runs, 18 and 12:

```
for i in $(seq 1 30); do ( ulimit -v 1048576; LD_LIBRARY_PATH=.../lib \
  timeout -s KILL 10 build/bin/rexx $D/two.rex ) 2>/dev/null | tr '\n' '|'; echo; done \
  | sort | uniq -c | sort -rn

     18 A-replied|A-after|B-replied|B-after|main-end|
     12 A-replied|A-after|B-replied|main-end|B-after|
```

**Fix round 2: `two.rex` itself was never recorded, only the command that ran it.** It was a
scratch file and is gone; what follows is a reconstruction of the same shape -- two classes,
each with one class method that replies and then prints two lines -- not the original bytes:

```
.ClassA~m
.ClassB~m
say 'main-end'

::class ClassA
::method m class
  reply
  say 'A-replied'
  say 'A-after'

::class ClassB
::method m class
  reply
  say 'B-replied'
  say 'B-after'
```

Run 30 times this sitting, under the same wrapper: **five** distinct orders, not two, and `B`
ahead of `A` throughout rather than behind it --

```
      8 B-replied|B-after|main-end|A-replied|A-after|
      8 B-replied|B-after|A-replied|main-end|A-after|
      7 B-replied|A-replied|main-end|B-after|A-after|
      4 B-replied|B-after|A-replied|A-after|main-end|
      3 main-end|A-replied|B-replied|A-after|B-after|
```

**Fix round 3: round 2's own sentence here said more than this measurement supports, and it is
retracted rather than reworded.** Round 2 called the five-order result above "a second,
independent demonstration that this shape family answers differently depending on what exactly
is written." It is not that. This reconstruction claims to be the same shape that gave 18/12,
and it gives five orders instead of two. Either the reconstruction is not that program, or the
original 30-run reading was undersampled -- and because the original file is gone, nobody can
tell which. So this measurement stands beside the 18/12 reading as a second, independent data
point and nothing more. No sentence here says why the two differ.

**Fix round 1 narrowed what this establishes, and the narrower claim is the true one.** The
report and `Interp::deferred`'s doc both said the oracle printed "an order no single-threaded
scheduling produces". That is false: a scheduler that yielded at the `REPLY` and returned to
the sender would produce some of these orders. What no single schedule produces is *all* of
them. The order I originally quoted came from an *instance*-method version of this shape, which
this crate cannot run at all and which is therefore a poor witness to have quoted, and this
task's review ran its own two-object shape 30 times and got **five** distinct orders where I
got two. The review re-ran its own five-order shape from a second, independent sitting and got
five orders again, three of the same rows -- so that shape's own order count is stable across
its own two sittings. Whether the same would hold for the 18/12 shape is exactly what this
round's reconstruction could not settle, for the reason above. What all of this establishes,
and all the exclusion decision needs, is that **the oracle has no single answer for such a
program**, so it cannot be a differential row. The doc comment now says that and nothing
stronger.

**What does distinguish the two readings, without saying why: `A-after` follows `B-replied` in
8 of 30 runs of the review's five-order shape and in 0 of 30 of the 18/12 reading** (both of
18/12's own orders have `A-after` before `B-replied`), which is about 1e-4 if the two readings
were draws from the same distribution. That is a fact about the two recorded samples, not a
claim about which property of either program causes it.

The exclusion decision itself is unchanged, and the review found a third racy shape supporting
it (a `SIGNAL ON SYNTAX` trap firing inside an owed body, splitting 15/15) plus one clean shape
I had not measured (a main body that raises after the send agrees on all three descriptors at
rc 214 on both engines).

Only the deterministic shapes are corpus rows. This is disclosed in `Interp::deferred`'s doc
and in the exclusions entry, not hidden.

### Corpus

`REXX_CORPUS_GATE=1 cargo test --release --test corpus`: **185 of 185 matching**, up from
177 of 177. Eight programs added, listed in `corpus/phase-5a.txt`;
`crates/rexx-exec/tests/coverage.rs`'s `EXPECTED_SUBSET_5A` and `collect_stress.rs`'s
`NO_ALLOCATION_PROGRAMS` updated, and a `sourceline_oracle` expectation generated for each
with the driver in that test's own module comment (each verified byte-identical to its
program). `method_reply.rex` and `method_reply_twice.rex` are also in
`RAW_STDERR_COMPARISON`.

### The control the brief requires: returning a value after `REPLY` without raising

Deleting the 98.936/98.937 raise from `Interp::returned_value` and rebuilding.

Before, both engines, `method_reply.rex`:

```
  ir           rc=0 stdout=[replied|after|] stderr=[    15 *-* return 'returned'|Error 98 running ... line 15:  Execution error.|Error 98.936:  RETURN cannot return a value after a REPLY.|]
  tree-walker  rc=0 stdout=[replied|after|] stderr=[    15 *-* return 'returned'|Error 98 running ... line 15:  Execution error.|Error 98.936:  RETURN cannot return a value after a REPLY.|]
```

After:

```
  ir           rc=0 stdout=[replied|after|] stderr=[]
  tree-walker  rc=0 stdout=[replied|after|] stderr=[]
```

**Exit status and stdout are unchanged and only stderr moves**, which is exactly why a gate
keyed on the status would not see this.

`REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` under that mutation:

```
183 of 185 matching
mismatches (2):
  [UNCLASSIFIED] lang/method_reply.rex: stderr differ
  [UNCLASSIFIED] lang/method_reply_exit_status.rex: stderr differ
test corpus_differential ... FAILED
```

**Fix round 1 corrected what this run proves.** I first wrote that `corpus_differential` was
the only red test in the whole workspace. It was, on the tree I ran it on -- which did not yet
contain the in-crate tests the same report goes on to add. Re-run on the shipped tree, still
gated and still `--no-fail-fast`, the mutation reddens two things:

```
REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast   # 98.936/98.937 deleted
  grep -E '^test .* FAILED$' | sort -u

test corpus_differential ... FAILED
test run::tests::a_value_returned_after_a_reply_reports_the_oracles_own_98_936 ... FAILED
  corpus: 183 of 185 matching
```

`method_reply_twice.rex` stayed green, correctly: it is 98.935, a different check.

**And the 98.935 mutation is 184, not 183.** Same command with that raise deleted instead:

```
test corpus_differential ... FAILED
test run::tests::a_second_reply_reports_the_oracles_own_98_935 ... FAILED
  corpus: 184 of 185 matching
```

One mismatch, `lang/method_reply_twice.rex`. So the two mutations redden disjoint in-crate
tests and different corpus counts, and neither figure stands for the other.

The **ungated** run of the same mutated build reports the same two mismatches and still
exits 0 (`183 of 185 matching -- REPORT MODE, NOT THE GATE`, `test corpus_differential ...
ok`). So without an in-crate test the legality check would have had no instrument in gates 1
through 3, and that is why I added
`a_value_returned_after_a_reply_reports_the_oracles_own_98_936` and
`a_second_reply_reports_the_oracles_own_98_935` to `run/tests.rs`. Each mutation reddens
exactly its own test and leaves the other green:

```
=== 98.936 deleted:
test run::tests::a_value_returned_after_a_reply_reports_the_oracles_own_98_936 ... FAILED
test run::tests::a_second_reply_reports_the_oracles_own_98_935 ... ok
=== 98.935 deleted:
test run::tests::a_second_reply_reports_the_oracles_own_98_935 ... FAILED
test run::tests::a_value_returned_after_a_reply_reports_the_oracles_own_98_936 ... ok
=== restored:
test result: ok. 2 passed; 0 failed
```

The adjacent success is in the same test: a **bare** `RETURN` after a `REPLY` must be legal,
stdout `replied\ntail\n`, empty stderr, rc 0 on both engines. An implementation that raised
on the reply rather than on the value carried by the return reddens there.

### The guard surface, and its three mutations

`the_guard_instructions_answers_and_the_phase_6_refusals` (`run/tests.rs`) carries the corpus
program's stdout, the 99.911 refusal, the 34.902 raise, and the two loud refusals, on both
engines. Each of three mutations reddens it:

```
=== M1: guard WHEN-false loud removed (always a no-op)
test run::tests::the_guard_instructions_answers_and_the_phase_6_refusals ... FAILED
=== M2: 99.911 check removed
test run::tests::the_guard_instructions_answers_and_the_phase_6_refusals ... FAILED
=== M3: top-level check removed (nested REPLY silently accepted)
test run::tests::the_guard_instructions_answers_and_the_phase_6_refusals ... FAILED
=== restored
test result: ok. 1 passed; 0 failed
```

M1 is the one the corpus cannot see at all: the oracle blocks for ever on a false `WHEN`, so
there is no transcript to compare and a silent no-op there is a wrong answer with no
differential row possible.

### Rooting

`a_parked_reply_keeps_its_variables_across_a_collection` (`collect_stress.rs`) runs a
replying method under `run_program_collect_every_alloc` on both engines, with every value
the resumed half prints wide enough to need a heap slot and the sender allocating after the
reply. Two mutations, one at a time:

* `RootSet::iter`'s `self.parked` chain removed: both engines panic at `a live value`.
* `park_reply`'s `context.object_roots(&mut anchor)` removed: both engines panic at `a live
  value`.

Under each, the plain (non-stress) run of the same program still prints all five lines
correctly on both engines, so nothing but the collector sees either one.

### What the checks could not see

* The `object_roots` destructurings are exhaustive with no `..`, so a **field** added to
  `Activation`, `MethodIdentity`, `InstanceVar`, `CallContext` or `Argument` is a compile
  error. They cannot see an `ObjRef` appearing inside `Settings`, `TrapMap`, `AddressState`
  or `TrappedCondition`, none of which holds one today. The instrument for that is the
  stress mode, which reaches a missed root as a wrong answer rather than as luck.
* `park_reply`'s `frame_aliases` assertion is a `debug_assert`, so it fires in gate 5 and
  not in gates 3 and 4. Had it been false, gate 5 would have panicked there; gates 3 and 4
  would have printed a wrong answer for a shared exposed variable, silently.
* `method_reply.rex` and `method_reply_twice.rex` being in `RAW_STDERR_COMPARISON` asserts
  the indent an owed body's report sits at. Had that indent been wrong, the default
  normalised comparison would have collapsed the difference and passed. I did **not**
  separately perturb that indent to watch raw mode redden, so what I have for that
  particular claim is the harness's own
  `raw_fails_where_normalized_passes_on_a_two_column_indent_difference`, which is about the
  comparison and not about these two programs.
* The `INTERPRET`-carrying-a-`GUARD`/`REPLY` refusals (99.912, 99.924) are `rexx-parse`'s
  and predate this task. They diverge from the oracle in the traceback's **first** line: the
  oracle echoes the fragment's own clause and this crate does not. Measured to be
  pre-existing and unrelated -- `interpret "::routine zz"` (99.914),
  `interpret "use local q"` (99.915) and `interpret "say )("` (37.2) all lose the same line
  -- and `execute`'s own parse arm documents that class of gap. Not a corpus row, and not
  mine to close.

## What instrument catches a regression for each refusal removed

| Refusal removed | Instrument |
| --- | --- |
| `GUARD` was `rexx-exec: GUARD is not implemented (Phase 5)` | The gated corpus (`method_guard_instruction.rex`, `method_guard_outside_method.rex`) **and** `the_guard_instructions_answers_and_the_phase_6_refusals`, which runs in every `cargo test`. Mutations M1-M3 proved the second live. |
| `REPLY` was `rexx-exec: REPLY is not implemented (Phase 5)` | The gated corpus (six programs) **and** the `run/tests.rs` reply tests. Deleting the 98.936/98.937 raise reddens the gated corpus at **183** of 185 plus `a_value_returned_after_a_reply_reports_the_oracles_own_98_936`; deleting the 98.935 raise reddens it at **184** of 185 plus `a_second_reply_reports_the_oracles_own_98_935`. Both are measured on the shipped tree; the earlier claim that either mutation gives 183 of 185 was wrong. The ungated corpus run reddens for neither, which is why the in-crate tests exist. |
| `Loud::guard_when_false` (new) | `the_guard_instructions_answers_and_the_phase_6_refusals` only. The corpus cannot carry this row -- the oracle blocks for ever, so there is no transcript. Proved live by M1. The program is now `corpus/oracle-crashes.txt` entry 7 and the `Loud`'s own doc names both the file and this instrument, which is `Loud::array_index_hole`'s convention and which fix round 1 added. |
| `Loud::reply_inside_construct` (new) | The same test only, for the reason inverted: a nested `REPLY` has an oracle answer, but the answer needs construct state this design cannot restore, so a corpus row would have to be a divergence. Proved live by M3. Fix round 1 named this instrument in the code -- the `Loud`'s doc and the test's own doc both say the row is its sole coverage, so a task splitting that test has to carry the row. |
| The parked root set | `a_parked_reply_keeps_its_variables_across_a_collection` under the stress mode, plus the three Task 16 corpus programs that allocate and so collect. Both mutations above proved it live. |

## Sitting

**Instrument `instructions:u` throughout.** Every figure below comes from
`bench-baselines/phase-5a-arms.tsv` with its `instrument` column kept, and is the median of
5 rounds **except the four `dispatchclass` contribution cells, which are derived from `min`**
for the reason set out under that table. Cycles were recorded too and are not read here: a
cycles figure is not a result on its own.

**Two sittings, because one of them cannot answer the 1% question.** The brief's command
compares `pinned` (`bench-baselines/pinned/rexx-run-15a1ffa98`) against `head`, which is the
*accumulated* ratio over every task since that pin. My own contribution needs the build
immediately before this task, so the second sitting is `base` (`b1016203c`, built in a
throwaway worktree) against `changed` (this tree), interleaved in one sitting of its own --
Task 15's `pinned>base`/`pinned>head` pair asks the same question with a third build. Every
ratio below is computed inside the sitting that produced it, and no per-pass or absolute
figure is compared across them. The one cross-sitting arithmetic is the reconciliation at the
end -- Task 15's accumulated *ratio* multiplied by this task's contribution *ratio* against
the accumulated ratio read here -- which is a consistency check on three ratios rather than a
measurement, and it is what found the contaminated median.

```
./target/release/rexx-arms --build pinned=bench-baselines/pinned/rexx-run-15a1ffa98 \
                           --build head=target/release/rexx-run \
    --axis alloc4c --axis arith --axis compound --axis emptyloop --axis strings \
    --axis varlookup --axis dispatchclass --axis bench-rexxcps/rexxcps.rex \
    --rounds 5 --task 16 --commit 1022bc885 --baseline bench-baselines/phase-5a-arms.tsv

./target/release/rexx-arms --build base=/tmp/.../rexx-run-base-b1016203c \
                           --build changed=target/release/rexx-run \
    <the same eight axes> --rounds 5 --task 16-contribution --commit 1022bc885 \
    --baseline bench-baselines/phase-5a-arms.tsv
```

### My own contribution: `base>changed`, `instructions:u`

```
axis           arm  size   ratio      [min..max]
alloc4c        tw   small  1.000595   [1.000594..1.000595]
alloc4c        ir   small  1.000001   [1.000001..1.000002]
alloc4c        tw   large  1.000579   [1.000579..1.000579]
alloc4c        ir   large  1.000001   [1.000001..1.000001]
arith          tw   small  1.000191   [1.000190..1.000191]
arith          ir   small  1.000001   [1.000001..1.000001]
arith          tw   large  1.000190   [1.000190..1.000190]
arith          ir   large  1.000001   [1.000001..1.000001]
compound       tw   small  1.000729   [1.000729..1.000729]
compound       ir   small  1.000000   [1.000000..1.000000]
compound       tw   large  1.000729   [1.000729..1.000729]
compound       ir   large  1.000000   [1.000000..1.000000]
emptyloop      tw   small  1.000000   [1.000000..1.000000]
emptyloop      ir   small  1.000000   [1.000000..1.000000]
emptyloop      tw   large  1.000000   [1.000000..1.000000]
emptyloop      ir   large  1.000000   [1.000000..1.000000]
strings        tw   small  1.000549   [1.000549..1.004936]
strings        ir   small  1.000001   [1.000001..1.000001]
strings        tw   large  1.000549   [1.000549..1.000549]
strings        ir   large  1.000001   [1.000001..1.000001]
varlookup      tw   small  1.001140   [1.001140..1.001140]
varlookup      ir   small  1.000000   [1.000000..1.000000]
varlookup      tw   large  1.001140   [1.001140..1.001140]
varlookup      ir   large  1.000000   [1.000000..1.000000]
dispatchclass  tw   small  1.002757   [see below -- min/min]
dispatchclass  ir   small  1.004191   [see below -- median/min]
dispatchclass  tw   large  1.002765   [see below -- min/min]
dispatchclass  ir   large  1.004202   [see below -- median/min]
rexxcps        tw   small  0.999679   [0.999265..0.999697]
rexxcps        ir   small  1.000193   [1.000174..1.000202]
```

**The `dispatchclass` rows are not read straight off the median, and fix round 1 corrected
them.** The four figures I first reported there (tw 1.002761/1.002765, ir 1.002169/0.999541)
read the median throughout, and exactly one arm of one axis of this sitting has a median that
is not the same statistic as the rest. Projecting median against min for every `absolute` cell
of `16-contribution` on `instructions:u`:

```
awk -F'\t' '$1=="16-contribution" && $5=="absolute" && $8=="instructions:u" \
  {sp=($9-$10)/$10*100; printf "%-14s %-8s %-3s %-6s med=%.0f min=%.0f spread=%.4f%%\n",\
   $3,$4,$6,$7,$9,$10,sp}' bench-baselines/phase-5a-arms.tsv

dispatchclass  base     ir  small  med=12916149038 min=12890142050 spread=0.2018%
dispatchclass  base     ir  large  med=25850879966 min=25730852836 spread=0.4665%
   ... every other cell of every axis: spread <= 0.0018%
```

`dispatchclass/base/ir` is the only contaminated cell: at least three of its five rounds
landed in a high mode, so its median sits above its own mode while the `changed` arm's does
not, and dividing one by the other understates the rise. Recomputed with each side's own
uncontaminated statistic -- `base`'s `min` in all four cells; `changed`'s `min` on the
tree-walker rows and `changed`'s `median` on the compiled-engine rows, since `changed/ir`'s
own median is not contaminated the way `base/ir`'s is:

```
ir small: 12944164395 / 12890142050 = 1.004191
ir large: 25838965683 / 25730852836 = 1.004202
tw small: 13084018670 / 13048048967 = 1.002757
tw large: 26118851434 / 26046836168 = 1.002765
```

**The reusable part is which statistic to trust, not the figure.** A median is the right
default and it failed here because the sitting is five rounds and a bimodal arm needs more
than five to put the median in the mode; `min` cannot be dragged upward by a slow round at
all, and on every clean cell it agrees with the median to four decimal places, so it costs
nothing to read both and notice when they part.

**No axis is over the 1% line on my own contribution.** The widest is `dispatchclass`, +0.28%
on the tree-walker and **+0.42%** on the compiled engine, and it is the axis I would expect to
move: it is the method-send axis, and `Activation` gained a field while
`enter_method_body` gained a branch and one `mem::replace`. `emptyloop`'s ratio is 1.000000
on both arms -- its absolute counts still differ in the last two digits (tw small
5,450,613,897 base against 5,450,613,987 changed), so the right statement is that it does not
move at this resolution, not that it is byte-for-byte identical, which is what an earlier
version of this sentence claimed and what the confinement argument was leaning on.

**The tw arm does not move further than the ir arm on every axis.** Re-derived from the table
above: it does on `alloc4c`, `arith`, `compound`, `strings` and `varlookup`; `emptyloop` is
equal at 1.000000 on both; `dispatchclass` is the other way round (+0.42% ir against +0.28%
tw) once the min-derived figures are used; and `rexxcps` moves in opposite directions (tw
0.999679 down, ir 1.000193 up). So the pattern holds on five of the eight axes and I have not
attributed it to anything -- a layout control is what that would need.

### Accumulated: `pinned>head`, `instructions:u`

```
axis           arm  size   ratio
alloc4c        tw   small  1.003574
alloc4c        ir   small  1.004806
alloc4c        tw   large  1.003483
alloc4c        ir   large  1.004622
arith          tw   small  0.999969
arith          ir   small  0.993067
arith          tw   large  0.999934
arith          ir   large  0.993017
compound       tw   small  1.006965
compound       ir   small  1.010472
compound       tw   large  1.006966
compound       ir   large  1.010473
emptyloop      tw   small  0.995434
emptyloop      ir   small  0.992022
emptyloop      tw   large  0.995434
emptyloop      ir   large  0.992022
strings        tw   small  1.008736
strings        ir   small  1.013411
strings        tw   large  1.008736
strings        ir   large  1.013410
varlookup      tw   small  0.997161
varlookup      ir   small  0.994260
varlookup      tw   large  0.997161
varlookup      ir   large  0.994260
dispatchclass  tw   small  1.015379
dispatchclass  ir   small  1.016492
dispatchclass  tw   large  1.015399
dispatchclass  ir   large  1.016525
rexxcps        tw   small  1.015097
rexxcps        ir   small  1.018793
```

**The accumulated disclosure, and it is wider than I was told.** My brief disclosed two axes
already over 1% accumulated -- `dispatchclass` at about 1.0123, which crossed during Task 14
and which Task 14 owns, and `rexxcps` at about 1.0186 on ir and 1.0154 on tw, an
unattributable position the plan licenses because that axis joined the guard at Task 15.
Both still read over 1% here: `dispatchclass` at 1.0154/1.0165 and `rexxcps` at
1.0151/1.0188. **My own contribution against those two is not nil and I am not saying it
is**: `dispatchclass` is +0.28% mine on tw and +0.42% on ir, and `rexxcps` is 0.9997/1.0002
mine, which is nil within the axis's own spread.

The reconciliation now closes on both arms, which it did not with the median figures --
`1.012279 x 1.002169 = 1.014453` against the 1.016525 read was the gap that found the
contaminated median. Task 15-fixround-2's own accumulated `dispatchclass` rows are 1.012572
(tw small), 1.012257 (ir small), 1.012597 (tw large) and 1.012279 (ir large), and with the
min-derived contribution:

```
ir  large  1.012279 x 1.004202 = 1.016533   against 1.016525 read
ir  small  1.012257 x 1.004191 = 1.016499   against 1.016492 read
tw  large  1.012597 x 1.002765 = 1.015397   against 1.015399 read
```

Four decimal places on every cell, so nothing about this axis's accumulated position is
unattributed.

**Two further axes read over 1% accumulated and were not in my disclosure**: `compound` on
ir at 1.0105 and `strings` on ir at 1.0134. Against both, **my own contribution is nil**:
`compound` is 1.0000 on ir and +0.073% on tw, and `strings` is 1.0000 on ir and +0.055% on
tw. So neither position is this task's, and I am recording them here rather than leaving a
later task to rediscover them -- whichever task owns the accumulated position on those two
axes, it is not this one.


## Commits

* `ebe91e39b` Run GUARD and REPLY inside a method instead of refusing them
* `1022bc885` Put GUARD and REPLY in the corpus, and pin their reports without the oracle
* `7eabdfb59` Record Task 16's two arms sittings
* `8265ead58` Fix round 1: record the hanging GUARD, and correct six claims about the checks

The corpus programs are their own commit, as the plan asks. The three in-crate tests go with
them rather than with the implementation, because each reads its program through
`corpus_source!` and so cannot compile before the programs exist. The working tree is clean
at `7eabdfb59`; this report itself is under `.superpowers/`, which `.gitignore` excludes, and
copying the run's records into `docs/superpowers/records/` is not this task's step.

## Files changed

New:

* `rust/corpus/lang/method_guard_instruction.rex`
* `rust/corpus/lang/method_guard_outside_method.rex`
* `rust/corpus/lang/method_reply.rex`
* `rust/corpus/lang/method_reply_outside_method.rex`
* `rust/corpus/lang/method_reply_twice.rex`
* `rust/corpus/lang/method_reply_no_result.rex`
* `rust/corpus/lang/method_reply_exit_status.rex`
* `rust/corpus/lang/method_reply_chain.rex`
* one `rust/crates/rexx-parse/tests/sourceline_oracle/<name>.txt` per program above

Edited:

* `rust/crates/rexx-core/src/roots.rs`, `rust/crates/rexx-core/src/lib.rs` -- the parked
  root category, `frame_aliases`, the `Parked` export.
* `rust/crates/rexx-exec/src/activation.rs` -- `ReplyState`, `Activation::reply`,
  `Activation::object_roots`, `DeferredReply`.
* `rust/crates/rexx-exec/src/dispatch.rs` -- the park branch in `enter_method_body`,
  `park_reply`, `resume_reply`, `run_deferred_replies`.
* `rust/crates/rexx-exec/src/run.rs` -- `exec_guard`, `exec_reply`, `top_level_clause`,
  `raised_guard_not_logical`, `returned_value`'s new signature and check, the two `step`
  arms, `trace_invocation_entry` made crate-visible.
* `rust/crates/rexx-exec/src/error.rs` -- 99.911, 99.919, 98.935, 98.936, 98.937.
* `rust/crates/rexx-exec/src/lib.rs` -- `Interp::deferred`, the deferred drain in
  `execute`, `Loud::guard_when_false`, `Loud::reply_inside_construct`,
  `instruction_owner`, `Argument::object_roots`, `CallContext::object_roots`.
* `rust/crates/rexx-exec/src/ir/drive.rs` -- `Op::Return`'s failure now leaves through
  `leave_stepped_clause` instead of `?`.
* `rust/crates/rexx-exec/src/run/tests.rs`, `tests/collect_stress.rs`, `tests/corpus.rs`,
  `tests/coverage.rs`, `tests/loud.rs`, `tests/owners.rs` -- the new tests and the tables.
* `rust/corpus/phase-5a.txt` -- the eight rows.
* `docs/superpowers/plans/phase-4-exclusions.txt` -- the `GUARD` limb retired.

Fix round 1 added `rust/corpus/oracle-crashes.txt` (entry 7 and the header's third failure
mode) and touched `roots.rs`, `activation.rs`, `dispatch.rs`, `lib.rs`, `run.rs`,
`run/tests.rs` and `tests/coverage.rs` for comments and one test rename.

## Self-review findings, and what I did about them

1. **The IR lost the failing clause.** `Op::Return`'s arm used `?` on
   `returned_value`, which propagates past `leave_stepped_clause` and so past the site
   recording. Measured: `0 *-* <no failing clause recorded>` on ir against the correct
   `15 *-* return 'returned'` on the tree-walker. `returned_value` could not fail before
   this task, so nothing had exercised that path. Fixed by `break 'cold Err(failure)`.
2. **The trace entry on resume was wrong the first way I wrote it.** I set
   `TraceEntry::Pending` and got no second `>I>` and no final `<I<`, because `>I>` is
   announced by the `TRACE` instruction here and the resumed half has none.
   `RexxActivation.cpp:562` has the real rule -- allowed *because* it was done -- and the
   announcement has to be asked for at the resume. Caught by the `trace r` probe, not by
   any gate.
3. **`live_parked` had no user**, so its "for asserting that a park is matched by a
   release" doc described an instrument that did not exist. It now backs a `debug_assert`
   at the end of `run_deferred_replies`.
4. **An alias in a parked frame would silently stop sharing.** Copying the values out and
   writing them back as plain slots loses an alias's redirection. Both routes to one are
   refused inside a method body -- measured, `PROCEDURE` there is 17.1 at rc 239 and
   `USE ARG >q` is 88.928 at rc 168, both matching the oracle -- but that is a fact about
   somewhere else, so it is now a `debug_assert` on `RootSet::frame_aliases` rather than a
   sentence.
5. **`resume_reply` popped a frame it had not proved was the top one.** A resumed body
   cannot park again, because `exec_reply` raises 98.935 before it can set the state -- but
   that is an inference, so there is a `debug_assert_ne!` on it now.
6. **Order churn on the hot path.** My first version moved `pop_slots` after the four
   indent restores in `enter_method_body`, which is a reordering every method send pays for
   on a path with no reply in it. The context swap moved earlier instead, and `pop_slots`
   is back where it was.
7. **A false claim in a comment I wrote.** `corpus/phase-5a.txt` said `method_reply.rex` was
   "compared raw" before I had added it to `RAW_STDERR_COMPARISON`; and
   `collect_stress.rs`'s new comment named a mechanism for why
   `method_guard_instruction.rex` allocates that I had not measured. Both corrected --
   the first by adding the entry, the second by deleting the mechanism.
8. **`Interp::deferred` is drained by `execute` alone.** A unit test driving `Interp::run`
   directly leaves an owed body unrun and its values parked for ever, with nothing
   complaining. Recorded on the field.

## Fix round 1

Seven findings, all addressed. The diff is **comments, one crashes-file entry and one `#[test]`
function rename** -- nothing that executes and nothing reachable from `rexx-run`, so **no
sitting was run**; the numbers in the Sitting section above are re-read from the same
committed TSV rows rather than re-measured, and finding 2 is an arithmetic correction to how
those rows were projected, not a new measurement.

| # | What it was | What was done |
| --- | --- | --- |
| 1 | The hanging `GUARD ... WHEN` had no `corpus/oracle-crashes.txt` entry, and `Loud::guard_when_false` skipped the convention `Loud::array_index_hole` set | Entry 7 added with the program, effect and cause; the file's header grew a **third failure mode** for an indefinite block, since neither existing label fits 0.00 s of CPU over 5.00 s elapsed; `Loud::guard_when_false` now names the crashes file and its instrument, as `array_index_hole` does |
| 2 | `dispatchclass` ir contribution reported as +0.22% | It is **+0.42%**. `dispatchclass/base/ir` is the sitting's only cell whose median and min disagree (0.2018% and 0.4665% spread against <= 0.0018% everywhere else), so the median was the artifact; recomputed from `min` the four cells reconcile with the accumulated figure to four decimal places. The false reconciliation sentence is replaced by the arithmetic that closes |
| 3 | Three false universals | Each re-derived on the shipped tree: the 98.936 mutation reddens **two** tests not one; the 98.935 mutation gives **184** of 185 not 183; "tw moves further than ir on every axis" holds on five of eight and is narrowed to those |
| 4 | "An order no single-threaded scheduling produces", in `Interp::deferred`'s doc as well as the report | Re-measured, 30 runs: **two** orders, 18 and 12, where the review's own shape gave five. Both doc and report now claim only that the oracle has no single answer, which is all the exclusion decision needs, and say why no exact distribution should be written down |
| 5 | The Phase 6 marking was claimed complete and was not | Markers added at `returned_value`'s check, `raised_guard_not_logical`, `Activation::reply`, `ReplyState::Owed`, `resume_reply`, `run_deferred_replies` and the whole parked group in `roots.rs`, `activation.rs` and `lib.rs`; the report's claim is now the site list itself, located by `grep -n` |
| 6 | `Activation::reply`'s doc said "Three readers" and there were four | The readers are named instead of counted, and the diff was swept for the shape: "the pair of probes", "The last two rows" and "the four legality refusals, one fatal each" were reworded too, along with the test's own name, which carried a false universal rather than a count |
| 7 | The guard test's doc contradicted itself and held an unrecorded instrument | Doc rewritten to separate the oracle-answer rows from the refusal rows by what they carry rather than by position; both `Loud`s name the test, and the test names `reply_inside_construct`'s row as its sole coverage. Renamed to `the_guard_instructions_answers_and_the_phase_6_refusals` |
| 8 | Two parked prose defects | Fixed anyway, both being false sentences of mine: "byte-for-byte unchanged" is now "does not move at this resolution" with the differing absolute counts quoted, and `coverage.rs`'s "four legality refusals, one fatal each" no longer calls a rc-0 program fatal |

**Gates, all on the committed tree at `8265ead58`.** `cargo fmt --all --check` and
`cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo doc --workspace
--no-deps` at 21 warnings, of which 4 unresolved links, 15 links to private items and 2
redundant explicit link targets, none of
the 4 in `rexx-core` or `rexx-exec` -- so this round's own new intra-doc links all resolve --
and the three test gates `98 ok / 0 failed` each with the corpus at **185 of 185**:

```
cargo test --release --workspace --no-fail-fast                     -> exit 0
REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast  -> exit 0
REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast  -> exit 0
```

They were re-run after the last comment edit rather than before it: an earlier pass of the
same three was three `///`-only hunks behind, which is cheap to close and not worth an
argument about what a comment can affect.

**What this round could not check.** Finding 1's entry rests on a hang I did not re-run and must
not: the file's own rule is that entries are never re-run to confirm them, and the CPU-time
reading is the review's. If that reading were wrong -- if the oracle were spinning rather than
blocking -- the entry's mode would be wrong and the header's new third label would have been
added for nothing; nothing in this tree would notice.

## Concerns

* **The scheduling matches the oracle only where the oracle is deterministic**, and the
  oracle is not deterministic for any program whose main body writes after the send. I have
  measured that directly (two orders in 30 runs of a two-object shape, 18 and 12, where this
  task's review got five orders from its own) and kept those shapes out of the corpus. A future
  differential sweep that generates such a program will see an intermittent divergence that
  is the oracle's own race, not a defect here. Phase 6 owns it.
* **A false `GUARD ... WHEN` refuses where the oracle hangs.** That is a divergence by
  choice; there is no transcript to match. It is the only place I knowingly answer something
  the oracle does not. Fix round 1 recorded it as `corpus/oracle-crashes.txt` **entry 7**,
  which the first pass had not -- the most expensive kind of program to rediscover is one
  with no end, and that file exists so nobody has to. The entry needed the file's header to
  grow a **third failure mode**: it admitted a memory-safety defect or a resource-exhaustion
  hazard, and a block at 0.00 s of CPU over 5.00 s elapsed is neither. The header now names
  the mode and says why an entry in it can only be characterised by killing it.
* **A nested `REPLY` refuses where the oracle answers.** `top_level_clause` is a real
  narrowing of D55 and it is loud rather than wrong. The design that would close it is a
  continuation for the enclosing construct, which is Phase 6's shape rather than a fix here.
* **`GuardOption` on the directive is still read by nobody**, and the exclusions entry's
  "what makes it unobservable" argument had to be rewritten: it used to rest on `REPLY` and
  the `GUARD` instruction being loud, and both run now. What is left is that neither creates
  a second activity. `~start` is still loud and re-measured as such.
* **A `Loud` inside an owed body moves the exit status to 120** where a `Raised` there does
  not. That is deliberate -- a message with an unchanged status is the silent gap the loud
  rule exists to exclude -- but it means a program whose owed body reaches a gap reports 120
  where the oracle would have reported the main body's own status. No such program exists in
  the corpus.
