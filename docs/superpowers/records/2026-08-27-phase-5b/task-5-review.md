# Task 5 review

Range: `c65b51641..d708491a7` on `plan/rust-rewrite`, four commits (`3ff1055de`, `cf85bfd67`,
`eda2a0b94`, `d708491a7`).

Reviewer scratch, everything below was run from there and nothing was written into the worktree
except this file:
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/ae5f3c99-11ad-4b29-9dfe-f3d991a1a398/scratchpad/t5review`.
Every cargo invocation used a `CARGO_TARGET_DIR` under that path. Mutations were applied to
`git archive d708491a7 | tar -x` copies.

Probe convention for every transcript below: fresh empty directory, absolute paths, three
descriptors read separately, never `2>&1`, `timeout -s KILL 20`, oracle wrapped as
`( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -s KILL 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`,
crate run on `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker` from the release build of
`d708491a7` unless a cell says debug.

---

## Verdicts

**Spec compliance: FAIL on one of the brief's named arms.** D60's characterisation is right at the
mechanism level, not just against the transcripts the report quotes -- verified below on class-id
sets the report never used, including the signed-`char` fold, a 64-bit-wrapping id and a tie-break
made after two other entries. D61 is respected by every new corpus program. D69's termination sweep
does reach a class-scope-rooted instance, and C5 is its only witness.

**What fails is the mixin arm the brief named.** `::CLASS M MIXINCLASS Object` with a class-side
`UNINIT` and `::CLASS K INHERIT M` still prints nothing for `K` here where the oracle prints
`uninit on K` -- silently, rc 0, empty stderr, both engines. The committed row for that arm cannot
see it, because the row gives `K` a `UNINIT` of its own, and deleting the `inherit` from the row
changes no output on either side. So the arm the brief said "must agree" does not, and its witness
was never able to say so.

**Quality: Changes requested.** Two Criticals -- a silent absence covering four shapes (CRIT-1) and a
non-terminating sweep (CRIT-2) -- three Importants, and seven Minors. The third Important is a
measured table in the report that does not reproduce; the fix it measures is real, and larger than
the table says.

Counts: **2 Critical, 3 Important, 7 Minor.** None of them is in the ordering rule, which held under
every probe I could design against it; all of them are in the delivery's registration surface, its
loop, or its witnesses.

---

## Findings

Finding ids are `CRIT-`/`IMP-`/`MIN-`; the task's five mutation controls keep their own
`C1`..`C5` names throughout.

| id | severity | subject | where |
|---|---|---|---|
| CRIT-1 | Critical | `check_uninit` is never re-run after an inherit -- four silent absences | below |
| CRIT-2 | Critical | the termination sweep runs to a fixed point; a `UNINIT` that allocates never returns | under "the silent-wrong-answer probes" |
| IMP-1 | Important | no re-entrancy interlock on `run_ready_uninits` | below |
| IMP-2 | Important | `uninit_class_mixin.rex` cannot witness its own subject | below |
| MIN-1 | Minor | `corpus/README.md` splice | below |
| MIN-2 | Minor | the `8`/`9` figure does not describe the committed program | below |
| MIN-3 | Minor | non-UTF-8 class id hashed after mangling (pre-existing) | below |
| MIN-4 | Minor | gate 5 unmeasured by both of us | below |
| MIN-5 | Minor | "a class object is the exception" has a second member | after CRIT-2 |
| MIN-6 | Minor | one measured sequence in three places, one of which asserts it | after CRIT-2 |
| MIN-7 | Minor | `Heap::clear_uninit` reached only from its own test | after CRIT-2 |
| IMP-3 | Important | the quadratic table's pre-fix column does not reproduce | under "the quadratic fix" |

### CRIT-1 -- `check_uninit` is never re-run after an inherit, so four `UNINIT` deliveries go missing silently

**Neither inherit path re-checks, and there are two of them.**

* `rust/crates/rexx-exec/src/dispatch.rs:4300` `native_class_inherit` ends at `:4318` with
  `interp.classes().inherit_at(...)` and no `check_uninit` -- the runtime `.K~inherit(.M)`.
* `rust/crates/rexx-exec/src/lib.rs:5444` `check_uninit(id)` runs **before** the
  `for target in &class.inherit` loop at `:5447`-`:5456`, and `inherit_mixin` (`lib.rs:5609`) calls
  `self.classes().inherit(class, mixin)` and nothing else -- the directive `::class k inherit m`.

The crate's whole set of `check_uninit` call sites, from
`/bin/grep -rn "check_uninit" --include=*.rs rust/crates/`, is `dispatch.rs:4110` (the `subclass`
factory tail), `dispatch.rs:4388` (`inheritInstanceMethods`) and that `lib.rs:5444`.

The oracle's is wider: `RexxClass::inherit` ends in `updateSubClasses()`
(`classes/ClassClass.cpp:1361`), and `updateSubClasses` calls `checkUninit()` (`:1052`) on the class
and recurses into every subclass (`:1062`). `uninherit` reaches the same site at `:1413`. **The
directive path is the same one**: `ClassDirective::install` sends `GlobalNames::INHERIT` to the
class object for each mixin (`instructions/ClassDirective.cpp:230`), so it lands in
`RexxClass::inherit` and re-checks there.

**The directive arm is the one the brief named and the one this task was asked to close**, and it is
still open. `task-5-brief.md`'s second arm is
`::CLASS M MIXINCLASS Object / ::METHOD uninit CLASS / ::CLASS K INHERIT M / ::METHOD uninit CLASS`,
with `K` carrying its own `UNINIT`; strip that and it is the real mixin arm, which fails:

```rexx
say 'main'

::class m mixinclass Object
::method uninit class
  say 'uninit on' self~id

::class k inherit m
```
```
oracle, three runs of three  rc 0  stderr empty  stdout  main / uninit on K / uninit on M
crate ir and tree-walker     rc 0  stderr empty  stdout  main / uninit on M
```

**And its instance-side twin**, an instance-side `UNINIT` reaching a class through a directive
`INHERIT`:

```rexx
say 'start'
o = .K~new
say 'built'
p1='aaaaaaaaaa'; p2='bbbbbbbbbb'; p3='cccccccccc'; p4='dddddddddd'; p5='eeeeeeeeee'
p6='ffffffffff'; p7='gggggggggg'; p8='hhhhhhhhhh'; p9='iiiiiiiiii'; p10='jjjjjjjjjj'
drop o
call gc 'force'
say 'after-gc'
exit 0
::class m mixinclass Object
::method uninit
  say 'instance uninit ran'
::class k inherit m
```
```
oracle, two runs of two   rc 0  stderr empty  stdout  start / built / instance uninit ran / after-gc
crate ir and tree-walker  rc 0  stderr empty  stdout  start / built / after-gc
```

**The runtime `~inherit`, class side.** Declaration order `QQ`, `ZED`, `MX`; `QQ` acquires its class-side `UNINIT` only at
the runtime `~inherit`, so it enters the oracle's table third and lands second in bucket 8 behind
`ZED`.

```rexx
say 'main'
.QQ~inherit(.MX)
say 'inherited'
exit 0
::class qq
::class zed
::method uninit class
  say 'u' self~id
::class mx mixinclass Object
::method uninit class
  say 'u' self~id
```

```
oracle, three runs of three   rc 0  stderr empty  stdout  main / inherited / u ZED / u QQ / u MX
crate ir                      rc 0  stderr empty  stdout  main / inherited / u ZED / u MX
crate tree-walker             rc 0  stderr empty  stdout  main / inherited / u ZED / u MX
```

The `~inherit` itself works -- adding `::method greet class` to `MX` and sending `.QQ~greet` after
the inherit answers `greet from QQ` on both sides, so the class-side method is reachable and only
the uninit-table entry is missing.

**The runtime `~inherit`, instance side -- the fourth shape, same missing call.**

```rexx
say 'start'
.K~inherit(.MX)
say 'inherited'
o = .K~new
p1='aaaaaaaaaa'; p2='bbbbbbbbbb'; p3='cccccccccc'; p4='dddddddddd'; p5='eeeeeeeeee'
p6='ffffffffff'; p7='gggggggggg'; p8='hhhhhhhhhh'; p9='iiiiiiiiii'; p10='jjjjjjjjjj'
drop o
call gc 'force'
say 'after-gc'
exit 0
::class k
::class mx mixinclass Object
::method uninit
  say 'instance uninit ran'
```

```
oracle, two runs of two   rc 0  stderr empty  stdout  start / inherited / instance uninit ran / after-gc
crate ir and tree-walker  rc 0  stderr empty  stdout  start / inherited / after-gc
```

**All four are silent**: matching rc, empty stderr on both sides, differing stdout. The two instance
arms are *absences*, which is the shape D69 exists to close and the shape the task's own C5 control was built
to catch -- C5 catches the class-scope-root path and cannot see this one.

**And the report states the opposite as a universal.** `task-5-report.md`, "The same oracle
mechanism was already analysed in this tree": "the insertion sequence is class creation order,
**which this crate already matches because `check_uninit` is called from the same places the oracle
calls it**." It is not, and the transcripts above are the falsification. That sentence is the
argument the sweep-order implementation rests on, so it needs correcting even if the fix is deferred.

**Fix.** Call `check_uninit` (and `refresh_parent_has_uninit`) after every inherit -- at the end of
`native_class_inherit`, and inside or after `install_class_at`'s `class.inherit` loop -- over the
class and its subclasses, which is `updateSubClasses`'s own shape. Then make
`corpus/lang/uninit_class_mixin.rex` the program that can see it (IMP-2), and add the runtime
`~inherit` program above as a second row: it is order-sensitive in a way none of the six added rows
is, because the entry that moves is made after two others.

### IMP-1 -- `run_ready_uninits` has no re-entrancy interlock, so a `GC('force')` inside a `UNINIT` runs the next finalizer inline

`rust/crates/rexx-exec/src/dispatch.rs:2202` `Interp::run_ready_uninits`, and
`rust/crates/rexx-exec/src/builtin/state.rs:240`, which drains from `gc()`.

The docstring cites `MemoryObject::runUninits` (`memory/RexxMemory.cpp:337`) as the model, and that
function opens with the interlock the crate does not have:

```
    if (processingUninits)
    {
        return;
    }
    processingUninits = true;
```
(`memory/RexxMemory.cpp:341`-`:347`, cleared at `:383`.)

So on the oracle a `GC('force')` reached from inside a finalizer collects and marks, and runs
nothing; the newly readied object waits. Here the nested `gc()` drains the ready list immediately.

```rexx
say 'start'
g = .G~new
say 'built'
z1='a'; z2='b'; z3='c'; z4='d'; z5='e'; z6='f'; z7='g'; z8='h'; z9='i'; z10='j'; z11='k'; z12='l'
drop g
call gc 'force'
say 'end'
exit 0
::class k
::method uninit
  say 'uninit K'
::class g
::method uninit
  say 'g uninit'
  o = .K~new
  say 'inner built'
  y1='a'; y2='b'; y3='c'; y4='d'; y5='e'; y6='f'; y7='g'; y8='h'; y9='i'; y10='j'; y11='k'; y12='l'
  drop o
  call gc 'force'
  say 'g done'
```

```
oracle, three runs of three  rc 0  stderr empty
  start / built / g uninit / inner built / g done / end / uninit K
crate ir, release and debug  rc 0  stderr empty
  start / built / g uninit / inner built / uninit K / g done / end
crate tree-walker, release and debug   identical to crate ir
```

Silent, and the finalizer's own output moves relative to the outer finalizer's. **Fix**: a
`processing_uninits` flag on `Interp`, checked and set in `run_ready_uninits`, which is the cited
function's own first four lines.

### IMP-2 -- `corpus/lang/uninit_class_mixin.rex` cannot witness its own subject

The file's first line is "A mixin's class-side UNINIT reaches the class that inherits it", and
`corpus/phase-5b.txt` describes it and `uninit_class_inherited.rex` as "the two arms `obdes.rex`
cannot see, **where the class that fires declares no `UNINIT` of its own**". That is true of
`uninit_class_inherited.rex` and false of this one: `::class k inherit m` is followed by
`::method uninit class`, so `K` declares one.

**The control, run.** Delete the `inherit m` -- the whole subject of the program -- and the output
does not move, on either side:

```
committed                  oracle rc 0   main / uninit on K / uninit on M
`::class k inherit m` replaced by `::class k`
                           oracle rc 0   main / uninit on K / uninit on M
                           crate ir and tree-walker rc 0, identical
```

`K` is bucket 7 and `M` is 9, so even the ordering is unchanged. The row is green whether the
class-side inherit propagation works, is absent, or does not exist -- and it is in fact absent (CRIT-1).
It is also redundant against its neighbour: the report's own control table shows it reddening under
C1, C2 and C3 in lockstep with `uninit_class_inherited.rex` and never without it.

**Fix**: delete `K`'s `::method uninit class` from the file, so `K` fires only if the mixin's
class-side `UNINIT` propagates. With CRIT-1 fixed the row then reads `main / uninit on K / uninit on M`,
matching the oracle above; without CRIT-1 fixed it reads `main / uninit on M` and is red, which is what a
witness for this arm is supposed to do.

### MIN-1 -- `rust/corpus/README.md:47` splices a sentence and moves what it asserts

The insert ends mid-line:

```
forbids -- measured, twenty runs of one class at bucket 16 beside one live
instance gave the instance first nineteen times and the class first once. It is not a gap this project could close by working
harder: the oracle does not agree with itself, ...
```

Before the change, "It is not a gap this project could close by working harder" attached to the
*rule* -- no program may print the iteration order of an object-keyed collection. It now attaches to
the class/instance mixing sentence, and "the oracle does not agree with itself" reads as a claim
about the class-object case, which is exactly what the new paragraph has just said is *not* true.
The line is also 123 columns where the file is hard-wrapped at 72.

**Fix**: put the new paragraph after the "It is not a gap ..." sentence, or start a new paragraph
before it, and rewrap.

### MIN-2 -- the plan's `8`/`9` figure does not describe the committed program

`docs/superpowers/plans/2026-08-27-phase-5b.md:415` ("eight padding clauses give `a c uninit` and
nine give `a uninit c`") and `rust/corpus/lang/uninit_instance_collected.rex:5`-`:6` ("eight
allocating clauses in their place leave the finalizer for the termination sweep"). Confirmed exactly (below). But the committed program has
**two** clauses between `~new` and `drop`, not nine, and it is green because
`say 'built' o~class~id` is worth more than one allocation. Measured on the committed file:

```
committed (say 'built' ... plus pad = ...)  oracle rc 0  start / built K / uninit ran / after-gc
the pad line deleted                        oracle rc 0  start / built K / uninit ran / after-gc
both clauses deleted                        oracle rc 0  start / after-gc / uninit ran
```

So the row's margin is the `pad` clause plus whatever the `say` clause is worth over nine slots, and
neither number is stated anywhere. A reader applying the "nine padding clauses" rule to this file
would conclude it is on the diverging side. **Fix**: say in the program's comment that the `say`
clause alone is enough and the `pad` line is the margin, with the two-line transcript above.

### MIN-3 -- `uninit_bucket` folds a `String`, so a non-UTF-8 class id is hashed after mangling

`rust/crates/rexx-classes/src/registry.rs:339` passes `self.id_string(class).as_bytes()`, and
`id_string` answers a `str`. Class ids reach that `str` through
`String::from_utf8_lossy` (`dispatch.rs:4099`), so a class id with a byte outside UTF-8 is stored
mangled and the bucket is computed over the mangled bytes.

This is a **pre-existing** divergence, not this task's: it shows with no `UNINIT` anywhere.

```rexx
say c2x(.Object~subclass('<0xE9>A')~id)      /* the literal holds the raw byte 0xE9 */
```
```
oracle  rc 0  stdout  E941
crate   rc 0  stdout  EFBFBD41          both engines
```

The task's new consumer inherits it: the same program with a metaclass `UNINIT` orders `41` before
`E941` on the oracle and `EFBFBD41` before `41` here. **No fix owed by this task** -- recorded so the
`uninit_bucket` doc's "over the bytes" is read as "over the bytes `id_string` still has", and so the
class-id mangling has a witness somewhere.

### MIN-4 -- gate 5 has no status in the report and I did not re-run it

`task-5-report.md`'s last section says so itself, which is the honest reading rather than a defect.
Recorded here as the one acceptance criterion neither the task nor this review measured.

---

## What I re-measured

### The order rule, against the mechanism rather than the transcripts

The controller verified the four citations; I checked that the implementation reproduces
`strhash(class id) % 17` ascending with entry order as the tie-break, on sets the report does not
use, predicting before running.

**P1 -- seven ids none of which appears in the report, chosen so that the sort is not any
alphabetical or declaration order, with one shared bucket and three ids long enough to wrap 64-bit
`size_t`.** Declared `WOMBAT`(16), `QUUX`(9), `THISISALONGCLASSNAME`(8), `ALPHABETICALLY`(6),
`ABCDEFGHIJKLMNOP`(14), `ZED`(8), `BAR`(5), each with `::METHOD uninit CLASS`. Predicted
`BAR ALPHABETICALLY THISISALONGCLASSNAME ZED QUUX ABCDEFGHIJKLMNOP WOMBAT` -- bucket ascending, and
bucket 8 in declaration order.

```
oracle, three runs of three  rc 0  stderr empty
  main / u BAR / u ALPHABETICALLY / u THISISALONGCLASSNAME / u ZED / u QUUX / u ABCDEFGHIJKLMNOP / u WOMBAT
crate ir           identical
crate tree-walker  identical
```

**P2 -- the signed-`char` half of the fold, which no transcript in the report or spec can see.**
`uninit_bucket` sign-extends (`i64::from(byte as i8)`). Two runtime classes built through a metaclass
`UNINIT`, ids `<0xE9>A` and `A`, created in that order. Signed predicts buckets 16 and 14, so `A`
first, contradicting creation order; unsigned predicts 12 and 14, so `<0xE9>A` first, agreeing with
it.

```
oracle, three runs of three  rc 0  stderr empty  stdout  main / built / u 41 / u E941
```

Signed confirmed, so the crate's fold matches the oracle's `char` signedness. (The crate's own answer
here is MIN-3's mangling, not an ordering defect.)

**P3 -- case sensitivity and runtime-built ids.** `.Object~subclass` preserves case where a directive
uppercases (`::class k` answers `K`, `.Object~subclass('mIxEd')~id` answers `mIxEd`, measured). Five
runtime classes created `aaa`(16), `mIxEd`(11), `MiXeD`(15), `Zzz`(5), `zzz`(4).

```
predicted                    zzz Zzz mIxEd MiXeD aaa
oracle, three runs of three  rc 0  main / built / u zzz / u Zzz / u mIxEd / u MiXeD / u aaa
crate ir and tree-walker     identical
```

**P4 -- the tie-break as entry order rather than declaration order.** This is CRIT-1's program; the
oracle puts `QQ` second in bucket 8 because it enters the table at the `~inherit`, and the crate does
not enter it at all. The tie-break rule is right in `uninit_classes_in_sweep_order` (a `sort_by_key`
on a `Vec` pushed in `check_uninit` order, and `sort_by_key` is stable), but the crate's entry
*points* are not the oracle's.

**Verdict on the order rule: the implementation matches the C++ mechanism, not only the
transcripts** -- the fold, its signedness, the 64-bit wrap, the modulus, the ascending walk and the
stable tie-break all reproduce on sets chosen to break any weaker rule. What does not match is where
entries are *made*, which is CRIT-1.

### D61 against every new corpus program

| program | `UNINIT`s that fire | order it depends on | D61 |
|---|---|---|---|
| `gate-tables/concepts/obdes.rex` | one class | none | fine |
| `lang/uninit_instance_collected.rex` | one instance | delivery point, not order | fine |
| `lang/uninit_instance_retained.rex` | one instance | delivery point, not order | fine |
| `lang/uninit_class_inherited.rex` | two classes | class vs class | fine, D60 characterised |
| `lang/uninit_class_mixin.rex` | two classes | class vs class | fine for D61, but see IMP-2 |
| `lang/uninit_class_sweep_order.rex` | four classes | class vs class | fine, D60 characterised |

No new program mixes an instance `UNINIT` with any other `UNINIT`. `uninit_instance_retained.rex`
declares `::method uninit` instance-side only, so its class object is not in the sweep and exactly
one finalizer runs -- the case `uninit_sweep_order.rs`'s fourth test pins at the unit level. The
instance-side 19/20 instability the task measured is therefore not reachable from any row. **D61 is
respected.**

### The five controls

Re-run at `d708491a7` (the reviewed HEAD; `cf85bfd67`, which the report used, differs from it only
in comment rewrapping). Each mutation applied to its own `git archive | tar -x` copy with its own
`CARGO_TARGET_DIR`; each copy `diff -rq`'d against the pristine copy before building, and each
touched exactly the file(s) intended. Command in every cell:

```
REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus --no-fail-fast
```

| arm | matching | status | mismatches |
|---|---|---|---|
| unmutated | **277 of 277** | 0 | -- |
| C1 `run_termination_uninits` returns immediately | **272 of 277** | 101 | `obdes`, `uninit_instance_retained`, `uninit_class_inherited`, `uninit_class_mixin`, `uninit_class_sweep_order` |
| C3 `sort_by_key` deleted | **274 of 277** | 101 | `uninit_class_inherited`, `uninit_class_mixin`, `uninit_class_sweep_order` |
| C5 the instance loop deleted from the termination sweep | **276 of 277** | 101 | `uninit_instance_retained` |
| C3 with this task's six entries removed from `phase-5b.txt` and `EXPECTED_SUBSET_5B` | **271 of 271** | 0 | -- |

Every figure matches the report. Every mismatch is silent: `exit=0` and `stderr=""` on both sides in
every cell, differing on stdout alone -- read from the harness's own
`rust: ... exit=0 / oracle: ... exit=0` lines.

**C3's load-bearing claim holds.** `obdes.rex` is **not** among C3's three mismatches, so the task's
own gate row cannot see the ordering rule, and the three order-sensitive rows are what pin it. C3's
mutation is the implementation an earlier draft of D60 called for, and it produces the exact reversal
the spec predicted (`uninit on P` before `uninit on K`, and `C B A D` for the four-class program).

**C3 reduced is green**, so nothing already in the corpus catches the ordering mutation: the three
order rows earn their place rather than merely being able to fail.

### The silent-wrong-answer probes the brief named

Each ran on the oracle and on both engines of the release build at `d708491a7`, three descriptors
read separately, fresh empty directory.

| shape | result |
|---|---|
| `UNINIT` raising (`y = 1/0` in a class `UNINIT` at termination) | agrees: oracle and crate both rc 0, stdout `main` / `in uninit`, empty stderr |
| a subclass whose parent also defines `UNINIT`, chained with `self~uninit:super` | agrees: both `uninit K` / `uninit P` before `after-gc`, rc 0 |
| the same **unchained** -- `K` and `P` both define an instance `UNINIT`, `K` does not chain | agrees: only `uninit K` runs, on both, rc 0, two runs of two |
| a class that acquires its `UNINIT` through an inherit | **diverges, CRIT-1**, in four shapes |
| an object resurrected by its own `UNINIT` (stored into a class-scope variable, then released, then a second `GC('force')`) | agrees: both run the finalizer exactly once, rc 0 |
| a `UNINIT` during a collection inside a `UNINIT` | **diverges, IMP-1** |
| a class whose `UNINIT` is defined after instances exist | agrees: `o = .K~new` then `.K~define('UNINIT', ...)` then `drop`/`GC('force')` prints nothing on the oracle, two runs of two, because the instance's behaviour snapshot predates the define |
| the reverse order -- `~define` then `~new` | the oracle runs it; the crate answers a **loud** refusal at rc 120, `method "UNINIT" of class "K" is not implemented (Phase 5)`. The same refusal appears for a non-`UNINIT` name (`.K~define('GREET', "say 'greet ran'")` then `o~greet`), so string-source `~define` installs nothing runnable here -- a pre-existing gap surfacing through the new delivery, loud, not a defect of this task |
| a `UNINIT` that allocates a new instance whose class has a `UNINIT` | **diverges, CRIT-2 below** |
| a program raising 42.3 with a class `UNINIT` | agrees byte for byte, rc 214, identical traceback on stderr |
| a program ending `exit 7` with a class `UNINIT` | agrees, rc 7, `main` / `uninit ran`, empty stderr |

### CRIT-2 -- the termination sweep runs to a fixed point where the oracle runs one pass, so a `UNINIT` that allocates does not terminate

`rust/crates/rexx-exec/src/dispatch.rs:2236`-`:2242`, inside `Interp::run_termination_uninits`
(`:2234`):

```rust
loop {
    let flagged = self.heap.take_uninit_flagged();
    if flagged.is_empty() { break; }
    self.run_uninit_batch(flagged, &mut loud);
}
```

`MemoryObject::runUninits` (`memory/RexxMemory.cpp:337`) is **one** pass over the table, and
`lastChanceUninit` (`:324`) calls it once and then `uninitTable->empty()` (`:330`) discards whatever
is left. So the oracle's sweep is bounded by the table it starts with plus what the running iterator
happens to reach; the crate's is bounded by nothing.

**Bounded witness**, a finalizer that allocates one further instance until a class-scope counter
reaches a limit:

```rexx
say 'start'
o = .K~new
drop o
say 'end'
exit 0
::class k
::method uninit
  n = .K~bump
  say 'u' n
  if n < 8 then z = .K~new
::method bump class
  expose c
  if var('c') = 0 then c = 0
  c = c + 1
  return c
```

```
oracle, three runs of three  rc 0  stderr empty  stdout  start / end / u 1 / u 2
crate ir and tree-walker     rc 0  stderr empty  stdout  start / end / u 1 ... u 8
```

The oracle answers `u 1 / u 2` with the limit at 4 as well, so its count is a property of its single
pass and not of the limit.

**Unbounded witness -- the crate does not terminate.** The same program with the counter removed, so
the finalizer always allocates:

```rexx
say 'start'
o = .K~new
drop o
say 'end'
exit 0
::class k
::method uninit
  z = .K~new
```

```
oracle  ( ulimit -v 1048576; ... timeout -s KILL 15 ... )   rc 0    stdout  start / end   stderr empty
crate   ( ulimit -v 2097152; REXX_ENGINE=ir timeout -s KILL 15 ... )
                                                            rc 137  stdout EMPTY          stderr empty
```

Stdout is empty rather than truncated because this crate buffers a program's output in `interp.out`
and writes it from `Outcome`, so a sweep that never returns loses everything the program printed.

**Fix.** Bound the sweep the way `runUninits` is bounded: take the flagged set once, run it, and do
not re-take. The oracle's `u 2` says one further entry made during the pass is still reached, so a
single `take_uninit_flagged` followed by one more is the closest match; whichever is chosen, the
`loop` has to go, and the F1 program above is the row that pins the choice.

### MIN-5 -- `corpus/README.md:34` "a class object is the exception" is a universal that has a second member

The added paragraph says "**A class object is the exception, and it is one because `RexxClass`
overrides the hash rather than because a class is special.**" The pattern that enumerates the
candidates is

```
/bin/grep -rn "HashCode .*::getHashValue" /home/moritz/dev/repos/ooRexx/interpreter/
```

which names `RexxNilObject` (`classes/ObjectClass.cpp:2916`), `RexxString`, `PointerClass`,
`RexxInteger`, `NumberString` and `RexxClass`. `RexxNilObject::getHashValue` answers a stored
constant, and `.nil` is an object that is neither a string nor a number, so it is a second exception
to the rule the paragraph is qualifying. Measured, four runs of four, oracle rc 0:

```rexx
say c2x(.nil~hashCode) c2x('AB'~hashCode) c2x(.Object~subclass('ZZ')~hashCode) c2x(.Object~new~hashCode)
```
```
EFBEADDE00000000  2108000000000000  400B000000000000  2F13DF108A80FFFF
EFBEADDE00000000  2108000000000000  400B000000000000  2F23E71E4080FFFF
EFBEADDE00000000  2108000000000000  400B000000000000  2F7390CEE480FFFF
EFBEADDE00000000  2108000000000000  400B000000000000  2F23E704AA80FFFF
```

`.nil` is `0xDEADBEEF` every run; a plain instance moves. (The same transcript is an independent
third confirmation of the fold: `'AB'` hashes to `0x0821` = 2081 = `31*65 + 66`, and the class `ZZ`
to `0x0B40` = 2880 = `31*90 + 90`, so a class object's hash really is its id string's.)

**Fix**: "a class object is *an* exception", or name the pattern above beside the sentence.

### MIN-6 -- three copies of one measured sequence, where one is asserted

`D E F G H I J K L M N O P Q A R B S C T` and `D A B C` each appear in
`rust/crates/rexx-classes/src/registry.rs:330`-`:332` (a doc comment),
`rust/crates/rexx-classes/tests/uninit_sweep_order.rs` (a doc comment **and** the assertion), and
`rust/corpus/README.md:40`-`:42`. `rust/CLAUDE.md`'s rule of 2026-08-27 is that a load-bearing
property is asserted in a test rather than restated in prose, "because a test cannot rot silently and
a comment can", and the same rule's "no record of experiments" covers
`rust/crates/rexx-core/tests/uninit.rs:127`-`:129` ("Deleting `ready_for_uninit`'s test in
`Heap::collect` reddens the second assertion, and skipping the `resurrect.push` ... reddens the
third") and `uninit_sweep_order.rs:3`-`:4` ("recorded before this crate's ordering was written"),
which is history rather than what the code does.

The measurements themselves are the allowed kind -- facts about an external system -- so this is
about the duplication and the framing, not about deleting the evidence. **Fix**: keep the sequences
in the test that asserts them, cite that test from the two prose copies, and drop the mutation
narrative and the "recorded before" clause.

### MIN-7 -- `Heap::clear_uninit` is now reached only from its own test

`cf85bfd67` replaced the per-object clear with `clear_uninit_all`. The full caller set at
`d708491a7`, from
`/bin/grep -rn "clear_uninit\b" --include=*.rs rust/crates/`, is
`rust/crates/rexx-core/tests/uninit.rs:25`, `:113`. Nothing in the interpreter calls it, so the
`retain` it performs -- and the test at `uninit.rs:101` that pins that `retain` -- now guards a path
no program takes. It is `pub` on a `pub` struct, so no lint sees it.

**Fix**: delete it and fold its test into `clear_uninit_all`'s, or say at the site why it is kept.

## Quality checks, with the command beside each

| check | command | result |
|---|---|---|
| ASCII only in added lines | `git diff c65b51641..d708491a7 -- rust/ docs/ \| /bin/grep '^+' \| LC_ALL=C /bin/grep -c -P '[^\x00-\x7F]'` | **0** of 755 added lines |
| no em-dash in added lines | same pipeline with `-P '\xe2\x80\x94\|\xe2\x80\x93'` | none |
| rustdoc intra-doc links, `rexx-classes` and `rexx-core` | `RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links" cargo doc --no-deps -p rexx-classes -p rexx-core --document-private-items` | the only two unresolved links are `MethodDict::merge` (`registry.rs:442`) and `MethodDict::replace_methods_from` (`:668`), both present at `c65b51641` (`git show c65b51641:rust/crates/rexx-classes/src/registry.rs \| /bin/grep -n "MethodDict::merge"` answers `415`). **No link this task added is unresolved.** |
| rustdoc intra-doc links, `rexx-exec` | same for `-p rexx-exec` | 20 unresolved links, none in `dispatch.rs`, `lib.rs:3669`-`:3680` or `builtin/state.rs`; the three link targets this task added (`ClassRegistry::uninit_classes_in_sweep_order`, `Interp::run_ready_uninits`, `UNKNOWN`) appear in none of them |
| the plan's `nop` claim | `say 'a'` / `o = .K~new` / nine `nop` / `drop o` / `call gc 'force'` / `say 'c'` | oracle, two runs of two, rc 0: `a` / `c` / `uninit`. **Confirmed** -- `nop` does not move the threshold |
| the plan's two-`GC` claim | the same with no padding and two `call gc 'force'` | oracle, two runs of two, rc 0: `a` / `c` / `uninit`. **Confirmed** |
| the save-stack boundary | the padded program at N in 0, 7, 8, 9, 10 | oracle `a c uninit` at 0, 7, 8 and `a uninit c` at 9, 10; crate `a uninit c` at every N on both engines. **The 8/9 boundary is exactly as reported.** |
| the `execute` comment's two claims | a program raising 42.3 with a class `UNINIT`, and one ending `exit 7` | rc 214 with an identical traceback, and rc 7, both printing `uninit ran`; crate matches the oracle on all three descriptors |

Comment rule of 2026-08-27: the new doc comments are long for it -- `uninit_classes_in_sweep_order`'s
runs to a full screen -- but every paragraph in them is either a C++ citation or an oracle
measurement, which the rule admits as "a fact about an external system that no reader can derive
from the code". MIN-6 is the part that is not admitted: a measurement restated in three places when one
of them asserts it, plus two passages of experiment narrative and one of history.

No comment added by this task states the size of a set. The nearest are `BUCKETS: u64 = 17`, which
is a value with its C++ name beside it, and "twenty entries do not reach that, measured above",
which is a measurement.

## The divergence left open: confirmed, and yes, it is a collector decision

**The measurement reproduces exactly.** `say 'a'` / `o = .K~new` / N clauses each assigning a string
literal / `drop o` / `call gc 'force'` / `say 'c'`, with `::class k` and `::method uninit` saying
`uninit`:

```
N      oracle                    crate ir and tree-walker
 0     a / c / uninit            a / uninit / c
 7     a / c / uninit            a / uninit / c
 8     a / c / uninit            a / uninit / c
 9     a / uninit / c            a / uninit / c
10     a / uninit / c            a / uninit / c
```

rc 0 and empty stderr in every cell. **The boundary is between 8 and 9**, as recorded. The plan's two
side conditions also hold: nine `nop`s in place of the padding leave the oracle at `a c uninit`, and
two `call gc 'force'` in a row with no padding do too.

The three C++ citations the plan rests it on are right: `Memory::SaveStackSize = 10`
(`memory/Memory.hpp:107`), `pushSaveStack(newObj)` on every allocation once the image is up
(`memory/RexxMemory.cpp:1053`), and `memoryObject.collectAndUninit(false);   // keep stack` for
`GC('force')` (`expression/BuiltinFunctions.cpp:3033`).

**I agree it is a collector decision, and here is the argument that settles it rather than asserting
it: the same hold is observable with no `UNINIT` anywhere.** `WeakReference`, the mechanism whose
documented purpose is to report collection, sees the identical boundary:

```rexx
o = .Object~new
w = .WeakReference~new(o)
<N padding clauses>
drop o
call gc 'force'
say 'after' (w~value == .nil)
```
```
oracle, two runs each, rc 0, empty stderr:   N = 0  ->  after 0        N = 9  ->  after 1
```

So the observable is "a driven collection reaches a just-created object", and `UNINIT` is only the
first mechanism in this crate that can see it. `WeakReference~new` is 5c's, and 5c will meet the same
divergence from the same cause -- which is the reason to decide it as a collector property in Task 6
rather than to license it under `UNINIT`. D59a's rule that a further observer needs its own decision
points the same way: this is a fifth observer, and it is not a class-object one.

The write-up in the plan's Task 6 is accurate, names both ways out, and states which side the
committed corpus program sits on. **No change requested there beyond MIN-2.**

## Acceptance criteria: what I re-measured and what I could not

**Re-measured**, with the command beside each figure above.

* `obdes.rex` agrees on both engines -- it is inside the `277 of 277 matching` unmutated corpus run,
  and it is the first of control C1's five mismatches when the sweep is skipped.
* Its probe path is in `corpus/phase-5b.txt` and in `EXPECTED_SUBSET_5B` -- removing the six entries
  from both is what turns the reduced arm into `271 of 271`, so both lists really carry them.
* `corpus/phase-5b.txt` carries a one-instance collection delivery
  (`uninit_instance_collected.rex`) and a one-instance termination delivery
  (`uninit_instance_retained.rex`, the only row control C5 reddens, re-run here).
* The order question has an answer, and the answer is right at the mechanism level rather than only
  against its own transcripts.
* `gate_table_c.rs`'s `obdes` control text is the corrected one, and **the control is recorded as run
  and re-run here**: control C1, `272 of 277` at status 101, silent on all five rows.
* **The debug abort**, on builds of `git archive c65b51641 | tar -x`:

```
uninit_instance_collected.rex

  BASE debug ir           rc 101  stdout empty
                          stderr  thread 'rexx-interp' panicked at crates/rexx-exec/src/lib.rs:6447:9:
                                  an object was resurrected for UNINIT and nothing here runs a finalizer
  BASE debug tree-walker  rc 101  identical
  BASE release ir and tw  rc 0    stdout  start / built K / after-gc            stderr empty
  HEAD debug ir and tw    rc 0    stdout  start / built K / uninit ran / after-gc   stderr empty
  HEAD release            rc 0    identical, and green in the 277 of 277 run
  oracle                  rc 0    start / built K / uninit ran / after-gc
```

  It fails on BASE both ways -- rc 101 in debug and a **silent** missing line at rc 0 in release --
  and passes after, exactly as the report says.

**Re-measured and NOT met.**

* "**the two inherited arms above agree**". `uninit_class_inherited.rex` agrees and is a real witness.
  `uninit_class_mixin.rex` agrees but is not a witness of its arm (IMP-2), and **the arm itself does
  not agree** (CRIT-1): a mixin's class-side `UNINIT` does not reach the class that inherits it here.
  The criterion is met on the row and not on the behaviour, which is exactly the split the brief's
  "record the control as run" rule exists to prevent.

**Not measured, by me or by anyone.**

* **Gate 5**, `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast`. The report claims
  no status for it either and says so plainly; I did not run it.

**Not re-measured, taken from the report.**

* **Controls C2 and C4.** The brief asked me for C1, C3 and C5, and those three reproduce exactly.
  C2's and C4's figures are the report's.
* Nothing else.

## The quadratic fix, re-timed interleaved

The machine was busy throughout (`/proc/loadavg` between 4 and 13 across the run), so both arms were
run **alternately** rather than one after the other, three rounds at each size, from one script:

```
for round in 1 2 3; do for prog in ...; do for rev in 3ff1055de cf85bfd67; do
  /usr/bin/time -f "%e" env REXX_ENGINE=ir timeout -s KILL 300 \
      "$SP/target-$rev/debug/rexx-run" "$P/$prog.rex" >/dev/null
done; done; done
```

on `do i = 1 to N ; o = .K~new ; end ; say 'main'` with `::CLASS K` / `::METHOD uninit` / `nop`, both
trees built with `cargo build --workspace --bin rexx-run` (debug) into their own `CARGO_TARGET_DIR`,
and each tree checked byte-identical to its revision (`diff <(git show REV:path) scratch/path`).

| N | `3ff1055de`, three rounds | `cf85bfd67`, three rounds | report says |
|---|---|---|---|
| 16,000 | 2.61 / 2.58 / 2.62 | 0.30 / 0.30 / 0.30 | 0.30 and 0.30 |
| 64,000 | 41.18 / 37.21 / 45.12 | 0.86 / 0.87 / 1.39 | 0.86 and 0.88 |
| 128,000 | 155.51 / 155.66 / 181.75 | 1.63 / 1.66 / 1.66 | 9.67 and 1.67 |
| 200,000 | killed at the 300 s bound | 2.60 | 49.55 and 2.48 |
| 200,000, class with no `UNINIT` | 1.27 | 1.31 | 1.27 and 1.28 |

**The fix reproduces, and the control does.** Every `cf85bfd67` cell matches the report's within
noise -- 0.30, 0.86, 1.63, 2.60 against 0.30, 0.88, 1.67, 2.48 -- and the no-`UNINIT` control reads
1.27 against 1.31, so the ordinary path really is untouched. The direction and the magnitude of the
win are not in question.

### IMP-3 -- the pre-fix column of `task-5-report.md`'s table does not reproduce, and its shape says why

At 16,000 the report's pre-fix figure is **0.30 s** and I measure **2.6 s**, three rounds of three.
At 64,000 it is **0.86 s** and I measure **37-45 s**. At 128,000 it is **9.67 s** and I measure
**155-182 s**. At 200,000 it is **49.55 s** and the run is still going at 300 s.

Load cannot be the explanation, because the *same runs, interleaved with these*, reproduce the
report's post-fix column exactly. A load factor large enough to turn 0.86 into 41 would have turned
0.86 into 41 on the other arm too.

**The shape points at the cause.** The report's pre-fix figures at 16,000 and 64,000 are its post-fix
figures -- 0.30 against 0.30, 0.86 against 0.88 -- which is what a stale binary produces, and this
task's own report already records one stale-binary batch thrown away for exactly that reason (`cp -p`
preserving mtimes). Whatever the cause, the table as committed says the quadratic term only appears
between 64,000 and 128,000, and it appears at 16,000.

**This does not weaken the fix; it understates it.** `cf85bfd67` is right on every cell.

**Fix**: re-take the pre-fix column with the interleaving above and the tree-identity check, or
delete the two columns and keep only the pair that was measured against a verified binary.
