## Task 5: `UNINIT` (D60, D61, D69), and characterising the sweep order

**Goal.** `corpus/gate-tables/concepts/obdes.rex` agrees, and instance `UNINIT` is delivered.

**BASE:** the commit named in your dispatch. Read
`.superpowers/sdd/2026-08-27-phase-5b/global-constraints.md` and the 5a constraints it points at,
`rust/CLAUDE.md`, the plan's Task 5 section, and D59, D59a, D60, D61 and D69 in the spec.

**You run out of plan order, and here is why.** The plan runs Tasks 2, 3 and 4 first. Task 1's review
found that `native_new` sets `Object::has_uninit` while `Interp::collect`'s
`debug_assert!(stats.pending_uninit.is_empty(), ...)` still asserts nothing does -- so **a debug build
now aborts at rc 101 once an instance whose class defines `UNINIT` becomes garbage**, where the oracle
is rc 0. Verified: 200,000 `.K~new` with `::METHOD uninit` gives rc 101 on both engines in debug and
rc 0 in release. The gate is green only because no committed program has that shape, which makes the
next task to write one redden for a reason not its own. Your delivery 1 is the fix, so you were moved
ahead of Tasks 2, 3 and 4. Nothing you consume comes from them.

**Measured, and to be re-measured by you before you build.** `obdes.rex` is the phase's only silent
row: rc 0 and empty stderr on both sides, oracle stdout `main` / `uninit ran`, crate stdout `main`.

**What is already there. Every sentence here is a claim to re-measure, not a premise** -- four of these
paragraphs were wrong on the previous plan, all in the direction of making the task look smaller.
`Heap::collect` returns `CollectStats::pending_uninit`, the unreachable objects whose flag is set,
resurrected rather than swept; `Heap::set_uninit` is the only writer of `true` and `Heap::clear_uninit`
is how a caller reports the finalizer ran. `Interp::collect` carries the `debug_assert!` above, whose
comment names itself as the site that owes delivery. `ClassGraph::check_uninit` and `parent_has_uninit`
are 5a's half of the propagation, **but they answer about a class's *instances*, which is deliveries 1
and 2 only**. Delivery 3 has no predicate at all: `class_graph.rs`'s own doc records that the
class-object side, where a `::METHOD uninit CLASS` lands, needs the table the oracle's `requiresUninit`
fills, and that "this crate has no such table". That table is yours to build, and `obdes.rex` -- your
only gate row -- is exactly that side.

**Build three deliveries, not one.**

1. **From a collection, for an instance.** Replace `Interp::collect`'s assertion with the delivery, and
   `clear_uninit` after the finalizer runs. Reproducible shape, five runs of five: `drop` then
   `call gc 'force'`, one instance.
2. **At termination, over every live object carrying the flag** (D69). This is not the same as (1) and
   cannot be built out of it: under D59 a class-scope `EXPOSE` roots an instance for ever, so it is
   never unreachable and a collection-driven delivery never runs its `UNINIT` **at all**. That would be
   an *absence*, not a late delivery, which is the silent wrong answer D69 exists to close. The oracle
   fires those at termination, measured, and the spec prints the program.
3. **At termination, for class objects** (D60). Never from a collection, because D59 means a class
   object is never collected.

**Characterise the order, which the spec deliberately does not.** D60 records that the oracle's class
`UNINIT` order at termination is reproducible and is **not** creation order: a three-class program
fires a class declared second after one declared third, three runs of three, and a mixed program fires
runtime-built classes before directive-installed ones. Two witnesses are not a rule. Find the rule, or
establish that it cannot be characterised cheaply and say so; **either outcome is this task's output**,
and D61 stays in force over class objects until the first one lands. The oracle source is readable at
`/home/moritz/dev/repos/ooRexx/interpreter/` -- read what it actually does rather than inferring a rule
from transcripts, then confirm the reading by running it.

**5a's Task 7 handed three class-`UNINIT` programs here and no task had named them**
(`docs/superpowers/records/2026-08-17-phase-5a/task-7-report.md:333`-`:348`). `obdes.rex` has one class
declaring its own `UNINIT` and cannot see either of the other two arms. Both are silent wrong answers
on this crate today, rc 0 with empty stderr on both engines:

```
::CLASS P / ::METHOD uninit CLASS / ::CLASS K SUBCLASS P
   oracle:  main / uninit on K / uninit on P     ten runs of ten
   crate:   main

::CLASS M MIXINCLASS Object / ::METHOD uninit CLASS / ::CLASS K INHERIT M / ::METHOD uninit CLASS
   oracle:  main / uninit on K / uninit on M
   crate:   main
```

The first is the inherited arm: `K` declares no `UNINIT` and fires one anyway, **before** the parent
that declares it. That also constrains the order question -- `P` is declared first and fires second.
Both must agree. Under D61 neither can be a corpus row until the order question is answered, so if it
is not, **record them as still open with their transcripts rather than dropping them**.

**Done when**

* `obdes` agrees on both engines under
  `REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast`,
  and its probe path is in `corpus/phase-5b.txt` in the same commit;
* the two inherited arms above agree, or are recorded open with their transcripts;
* `corpus/phase-5b.txt` carries a one-instance collection delivery and a one-instance termination
  delivery;
* the order question has an answer recorded either way;
* `gate_table_c.rs`'s `obdes` control text is corrected from "stop running `UNINIT` before the object's
  storage is reclaimed" to "skip the termination sweep", since under D59 a class object's storage is
  never reclaimed;
* and **the control is recorded as run**, with its transcript: skipping the termination sweep reddens
  `obdes`, silently, at rc 0 with empty stderr on both sides;
* and **the debug abort is gone.** Add a case that would have caught it -- an instance whose class
  defines `UNINIT` becoming garbage under a collection -- and confirm it fails on BASE and passes after.

**Hazards this phase keeps hitting.**

* **D61 is a hard constraint, not advice.** No row and no corpus program may depend on the order two
  `UNINIT`s run in at termination: the oracle does not reproduce it for instances (two instances, two
  orders over twenty runs). A program needing more than one `UNINIT` to fire must force each with
  `drop` then `call gc 'force'` (reproducible five runs of five), or must not exist.
* **"Can fail" is not "adds coverage."** Run each mutation against the suite *without* your new
  program before claiming the program earns its place.
* **A witness that cannot fail.** Task 1 found two: a caller variable roots the instance, and a
  five-byte value never becomes a heap object. Check your witness's value can actually move.
* **An absence reads exactly like a pass.** A `UNINIT` that never fires prints nothing, and so does a
  program that never built the object. Make each witness print something only the delivery can produce.
* **No comment states the size of a set.** ASCII only, no em-dashes, no historical framing.

**Then run all five gates**, from `rust/`, each status read unpiped, plus the phase-gate command.
Write your report file first and append as you go; do not poll long commands on a short interval;
message the controller when you finish.

---

## Controller pre-dispatch check, run against the tree at c65b51641

Every claim below was measured, not copied. They are still yours to re-measure; this records that
none of them was missing at dispatch.

* **The uninit API the task consumes is present.** `rexx-core/src/heap.rs:48`
  `pub pending_uninit: Vec<ObjRef>`, filled at `:229` and returned at `:280`; `:314`
  `pub fn set_uninit`; `:339` `pub fn clear_uninit`. `class_graph.rs:376` `has_uninit`, `:395`
  `check_uninit`, `:435` `parent_has_uninit`.
* **The missing table is documented at the site.** `class_graph.rs:386`-`:389` cites
  `if (hasUninitMethod()) requiresUninit();` at `ClassClass.cpp:1222` and ends "This crate has no
  such table." That is delivery 3's subject.
* **`obdes`'s control text is still the wording you must correct**, at `gate_table_c.rs:457`:
  "stop running `UNINIT` before the object's storage is reclaimed, which is **silent**".
* **The debug abort is live and is your "fails on BASE" witness.** Measured on the debug binary at
  this commit, from a fresh empty directory, three descriptors read separately:

```
do i = 1 to 200000 ; o = .K~new ; end ; say 'main'   ::CLASS K / ::METHOD uninit / nop

  debug ir            rc=101   thread 'rexx-interp' panicked at crates/rexx-exec/src/lib.rs:6447:9
  debug tree-walker   rc=101   same
  oracle              rc=0     main
```

  Release is rc 0, so this is a debug/release split as well as a divergence. The case you add for it
  must fail at `c65b51641` and pass after; run it both ways and put both transcripts in the report.

## What landed between the plan being written and your dispatch

Task 1 (`f7c38531c`), a prose commit (`c245dc418`), and Task 1's fix round (`f06142745` plus the
sitting at `c65b51641`). The fix round is the reason `heap_to_number` refuses an array and an
instance outright and why `operator_operand_gap` is asked ahead of the truth test in
`logical_values_body` and `PrefixOp::Not` -- if a `UNINIT` probe of yours puts an object in an
operand position it will meet a loud refusal, and that is current, licensed behaviour rather than a
defect to work around.

Tasks 2, 3 and 4 have **not** run. `objcla`, `usesem` and `methodsbyclass` are red and are theirs;
do not fix them and do not treat their rows as evidence about your own.
