# Scoped re-review: 4d-1 Task 3, fix round 1

Range 93e285d1..10c95815

10c95815 Correct the retention diagnosis: cause B has two sites, not one

 docs/superpowers/plans/phase-4-exclusions.txt | 177 ++++++++++++++++----------
 docs/superpowers/plans/phase-4d-diagnosis.md  |  26 +++-
 docs/superpowers/plans/phase-4d-retention.md  | 168 ++++++++++++++++++++----
 3 files changed, 276 insertions(+), 95 deletions(-)

## Diff
diff --git a/docs/superpowers/plans/phase-4-exclusions.txt b/docs/superpowers/plans/phase-4-exclusions.txt
index 85ac9a1d..ccd6de8b 100644
--- a/docs/superpowers/plans/phase-4-exclusions.txt
+++ b/docs/superpowers/plans/phase-4-exclusions.txt
@@ -1777,20 +1777,129 @@ above.
 
   AND THE GENERAL HAZARD, RECORDED BECAUSE IT IS DECIDED RATHER THAN
   OVERSIGHT. A rebuilt oracle silently reprices every differential result in
   this project, and nothing stores a build identity beside any number: Task
   6's sweep moved from 18 mismatches to 12 without its own code changing,
   and no harness noticed. Adding a fingerprint to the corpus harness was
   considered and DECLINED -- rebuilds are rare and are Moritz's own -- so the
   cost is that any recorded figure is a claim about the binary present when
   it was taken. Re-measure rather than trust a number across a rebuild.
 
+
+  KNOWN GAP: A LOOP'S MEMORY GROWS WITHOUT BOUND, FROM TWO INDEPENDENT
+  CAUSES, AND A LONG-RUNNING PROGRAM ABORTS. Measured 2026-08-08 at
+  c9a90906, /usr/bin/time -v peak resident set, each probe from a fresh
+  empty directory with its exit status and stdout checked. The full
+  diagnosis, the massif call stacks and the prototype figures are in
+  docs/superpowers/plans/phase-4d-retention.md.
+
+  WHAT A USER SEES. Under the project's standard `ulimit -v 1048576` two of
+  the five runnable benchmark axes die:
+
+    bench-programs/arith.rex     rc 134, stdout empty,
+    bench-programs/strings.rex   stderr `memory allocation of 402653184
+                                 bytes failed`
+
+  No Rexx condition is raised, so SIGNAL ON SYNTAX cannot catch it; no
+  traceback is printed; output already produced is lost.
+
+  ONE OF THOSE TWO IS THE CAP RATHER THAN THE MEMORY, which rust/CLAUDE.md
+  requires every `ulimit -v` finding to state. `ulimit -v` charges reserved
+  address space, and this crate reserves INTERPRETER_STACK_BYTES = 512 MiB
+  (lib.rs:309) before any program runs. Raising the cap by exactly that, to
+  `ulimit -v 1572864`: arith goes rc 134 -> rc 0 with correct output, and
+  strings still dies (`memory allocation of 805306368 bytes failed`). So
+  with the reservation free ONE axis dies, not two. The ruling stands on the
+  linearity rather than the count: growth is linear and unbounded on every
+  shape measured, so any finite limit is reached by a long enough run, and
+  the failure is a function of how long a program runs rather than of how it
+  is written. A four-line loop correct today fails after enough iterations.
+
+  CAUSE A -- NOTHING TRIGGERS A COLLECTION, so every heap object a program
+  allocates is retained for the process's whole life. Retention is 128
+  bytes per short string, 144 per wide decimal, of which 96 is the
+  `rexx_core::heap::Slot` the arena vector grows by (massif, at
+  `Heap::alloc_with_uncollected`). `Heap.slots` is only ever pushed and
+  indexed -- no truncate, no shrink_to_fit -- so the arena is a permanent
+  high-water mark. That bounds rather than defeats a trigger policy: the
+  free list IS reused, and measured on one 8,000,000-iteration program,
+  peak RSS tracks the collection interval linearly -- 1,002,700 KB with no
+  collection, 15,796 KB forcing one every 100,000 iterations, 4,036 KB
+  every 10,000.
+
+  CAUSE B -- A LOOP HEADER'S TEMPS ARE ROOTED UNTIL THE LOOP ENDS, which is
+  a root leak and NOT fixable by a collector. TWO push sites, and naming
+  only the first is the error this row was first filed with:
+
+    Interp::loop_advance   (run.rs) the control variable of DO i = ...
+    Interp::eval_condition (run.rs:6016) every WHILE or UNTIL test
+
+  Both run inside the SAME single step_in_temps_frame -- the one belonging
+  to the whole DO instruction -- so both accumulate 8 bytes a pass for the
+  loop's entire run, and a loop with both pays both. The tree already
+  documented the second: step_in_temps_frame's own doc at run.rs:4073-4082
+  says "a `do while` running 10^7 passes holds ~10^7 dead-but-rooted temps
+  in one frame", classifying it as a cost rather than a defect, which was
+  right while nothing collected.
+
+  MEASURED, 8,000,000 iterations. `do n; nop; end` is flat at 2,532 KB, with
+  no control variable and no condition. `do i = 1 to n; nop; end` reaches
+  64,268 KB. `do n while zz; nop; end` with zz=1 -- no control variable, and
+  nothing on either side of the test that allocates -- reaches 64,468 KB,
+  which isolates the second site with the first absent. UNTIL matches
+  (63,704 KB). When the rooted values are not tagged small integers the
+  roots pin heap objects too: `do i = 1.5 to n by 1` with a nop body costs
+  1,064,416 KB.
+
+  A FIX MUST COVER BOTH SITES. All four runnable benchmark axes are
+  `do i = 1 to n`, so a fix to loop_advance alone turns every axis green and
+  leaves DO WHILE and DO UNTIL growing without bound -- the shape a
+  long-running service loop is actually written in. `do n while zz; nop;
+  end` is the acceptance test, not varlookup.rex.
+
+  AND A WARNING ABOUT THE INSTRUMENT: massif attributes the realloc, not the
+  leaking push. Both sites push into one RootSet.temps vector, so massif
+  names whichever push crosses the capacity boundary. Measured on `do while
+  k < n; k = k + 1; end` it names eval_arithmetic (eval.rs:620), the BODY's
+  push, while the entries piling up are eval_condition's. Isolate a suspect
+  push by removing the others (a nop body), not by reading the deepest
+  frame.
+
+  NOT A BROKEN COLLECTOR. Both prototypes were built, measured and reverted;
+  with both in place the whole test suite passes, 1,317 tests, 0 failures.
+  Peak RSS falls 42x on arith, 16x on compound, 298x on strings and 60x on
+  varlookup, and strings gets 16% FASTER. Prototype B covered loop_advance
+  only, so those figures bound the counted form and say nothing about what
+  closing the WHILE/UNTIL site is worth.
+
+  NO OWNER IS ASSIGNED HERE. 4d-2 is the phase unit that would take it, and
+  the diagnosis document is written for whoever does. A trigger policy must
+  first sweep rexx-exec for the under-rooted window `Heap::collect`'s own doc
+  comment records, because the corpus is too small to hit one.
+
+  WHY THE DIFFERENTIAL SUITE CANNOT SEE THIS, AND THIS LIMB IS THE ONE THAT
+  GENERALISES. Neither the corpus harness nor the trace harness reads peak
+  resident set, getrusage, or any memory figure at all: a program that
+  retains a gigabyte and prints the right bytes passes. And every corpus
+  program is far too small to accumulate a visible amount -- the largest
+  loop bound anywhere in rust/corpus is `to 1010`, which at 8 bytes an
+  iteration retains about 8 KB. The two compound, so closing the gap needs
+  both a program whose iteration count lets the linear term dominate the
+  fixed cost AND a bound on its peak resident set rather than on its output.
+  Adding either alone catches nothing. The benchmark suite carries a
+  narrower version of the same gap -- no axis is a conditional loop -- so
+  whatever memory regression test 4d-2 adds needs a DO WHILE in it or it
+  inherits the blind spot that made this row wrong the first time. That is a
+  gap in the instrument this project's whole definition of correctness rests
+  on, and it is why unbounded growth survived three phases of differential
+  testing green.
+
 CLOSED DEFECTS -- measured wrong, measured again, now matching
 -------------------------------------------------------------
 
 A fourth status, and the reason it is a section rather than a deletion: a
 divergence that was found, fixed and verified leaves nothing behind for a
 reader to trip over EXCEPT the temptation to "simplify" the fix. Each row says
 what the oracle does, what we used to do, and which test would go red if the
 fix were removed. Removing a row here needs the same amendment a KNOWN GAP does.
 
   THE *-* CLAUSE ECHO SATURATES AT 40 COLUMNS OF INDENT (fixed, 4b Task 2).
@@ -2455,78 +2564,10 @@ fix were removed. Removing a row here needs the same amendment a KNOWN GAP does.
   `a_compound_control_variables_tail_re_resolves_every_pass`,
   `a_compound_controls_to_bound_survives_a_masking_stem_default`,
   `a_stem_control_variable_binds_through_stem_assign`,
   `a_simple_control_variable_still_binds_through_the_fast_slot_path`,
   `leave_and_iterate_by_name_still_reach_a_compound_controlled_loop`, and
   `a_compound_control_variable_traces_its_own_c_line`. The seven ooTest
   bodies this unblocked (`DO::test_DO_standardTest2A`, `2B`, `2P`, `2Q`,
   `5-69`, `ITERATE::test_12`, `LEAVE::test_11`) are no longer in
   `rust/corpus/keyword-exempt.txt`; `the_exempt_set_matches_the_current_
   failures` is the witness that all seven now pass.
-
-
-  KNOWN GAP: A LOOP'S MEMORY GROWS WITHOUT BOUND, FROM TWO INDEPENDENT
-  CAUSES, AND A LONG-RUNNING PROGRAM ABORTS. Measured 2026-08-08 at
-  c9a90906, /usr/bin/time -v peak resident set, each probe from a fresh
-  empty directory with its exit status and stdout checked. The full
-  diagnosis, the massif call stacks and the prototype figures are in
-  docs/superpowers/plans/phase-4d-retention.md.
-
-  WHAT A USER SEES. Under the project's standard `ulimit -v 1048576` two of
-  the five runnable benchmark axes die:
-
-    bench-programs/arith.rex     rc 134, stdout empty,
-    bench-programs/strings.rex   stderr `memory allocation of 402653184
-                                 bytes failed`
-
-  No Rexx condition is raised, so SIGNAL ON SYNTAX cannot catch it; no
-  traceback is printed; output already produced is lost. The failure is a
-  function of how long a program runs rather than of how it is written, so
-  a four-line loop that is correct today fails after enough iterations.
-
-  CAUSE A -- NOTHING TRIGGERS A COLLECTION, so every heap object a program
-  allocates is retained for the process's whole life. Retention is 128
-  bytes per short string, 144 per wide decimal, of which 96 is the
-  `rexx_core::heap::Slot` the arena vector grows by (massif, at
-  `Heap::alloc_with_uncollected`). `Heap.slots` is only ever pushed and
-  indexed -- no truncate, no shrink_to_fit -- so the arena is a permanent
-  high-water mark. That bounds rather than defeats a trigger policy: the
-  free list IS reused, and measured on one 8,000,000-iteration program,
-  peak RSS tracks the collection interval linearly -- 1,002,700 KB with no
-  collection, 15,796 KB forcing one every 100,000 iterations, 4,036 KB
-  every 10,000.
-
-  CAUSE B -- A COUNTED LOOP'S CONTROL TEMP IS ROOTED UNTIL THE LOOP ENDS,
-  which is a root leak and NOT fixable by a collector. `Interp::loop_advance`
-  (run.rs) calls `roots.push_temp` on the control variable's previous value
-  and pops nothing; the only release is the enclosing `step_in_temps_frame`'s
-  pop, and the instruction that frame belongs to is the whole DO. So
-  `RootSet.temps` grows 8 bytes an iteration for the loop's whole life
-  (massif, at `push_temp` under `loop_advance`). Measured, 8,000,000
-  iterations: `do n; nop; end` is flat at 2,568 KB and `do i = 1 to n; nop;
-  end` reaches 64,268 KB, which is the negative control that separates the
-  two causes. When the control values are not tagged small integers the
-  roots pin heap objects too: `do i = 1.5 to n by 1` with a `nop` body costs
-  1,064,416 KB.
-
-  NOT A BROKEN COLLECTOR. Both prototypes were built, measured and reverted;
-  with both in place the whole test suite passes, 1,317 tests, 0 failures.
-  Peak RSS falls 42x on arith, 16x on compound, 298x on strings and 60x on
-  varlookup, at a wall cost between -16% and +6% -- strings gets FASTER.
-
-  NO OWNER IS ASSIGNED HERE. 4d-2 is the phase unit that would take it, and
-  the diagnosis document is written for whoever does. A trigger policy must
-  first sweep rexx-exec for the under-rooted window `Heap::collect`'s own doc
-  comment records, because the corpus is too small to hit one.
-
-  WHY THE DIFFERENTIAL SUITE CANNOT SEE THIS, AND THIS LIMB IS THE ONE THAT
-  GENERALISES. Neither the corpus harness nor the trace harness reads peak
-  resident set, getrusage, or any memory figure at all: a program that
-  retains a gigabyte and prints the right bytes passes. And every corpus
-  program is far too small to accumulate a visible amount -- the largest
-  loop bound anywhere in rust/corpus is `to 1010`, which at 8 bytes an
-  iteration retains about 8 KB. The two compound, so closing the gap needs
-  both a program whose iteration count lets the linear term dominate the
-  fixed cost AND a bound on its peak resident set rather than on its output.
-  Adding either alone catches nothing. That is a gap in the instrument this
-  project's whole definition of correctness rests on, and it is why
-  unbounded growth survived three phases of differential testing green.
diff --git a/docs/superpowers/plans/phase-4d-diagnosis.md b/docs/superpowers/plans/phase-4d-diagnosis.md
index 4105db58..f2952988 100644
--- a/docs/superpowers/plans/phase-4d-diagnosis.md
+++ b/docs/superpowers/plans/phase-4d-diagnosis.md
@@ -1,21 +1,25 @@
 # Phase 4d diagnosis -- where the interpreter's time and memory go
 
 Measured 2026-08-08 at `9bcbfeda`, release build with the pinned profile, this machine.
 Driven directly rather than through the task plan, because the work is iterative and a linear task list was the wrong shape for it.
 
 **Cause 3 and its two open questions are superseded by `phase-4d-retention.md`, measured at
 `c9a90906`.**
-Three statements below are false at that commit, and the corrections are there with their evidence:
-this is a missing trigger policy **and also** a root leak, in `Interp::loop_advance`, which a
-collector cannot fix; the standard `ulimit -v 1048576` now aborts two of the five axes rather than
-four; and both "what is not diagnosed" items about the arena and the 128-byte figure are answered.
+Two statements below are false at that commit and one is answered rather than false; each carries an
+inline marker where it appears, so a reader arriving by search sees the warning beside the claim.
+False: "not a root leak" -- it is one, in `Interp::loop_advance` and `Interp::eval_condition`, which
+a collector cannot fix.
+False: four of five axes abort under the standard cap -- two do, and one of those two survives the
+cap being raised by the 512 MiB this crate merely reserves.
+Answered: both "what is not diagnosed" items, about the arena being a high-water mark and about the
+128-byte figure.
 Everything else here, including the four causes and the per-axis attribution, stands as measured at
 `9bcbfeda`.
 
 ## Summary
 
 **Four causes, three of them compounding.**
 The headline per-axis ratios -- `varlookup` 23x, `strings` 14x, `compound` 12x, `arith` 3.5x -- are mostly one cause wearing four names.
 
 | cause | cost | evidence |
 |---|---|---|
@@ -87,23 +91,37 @@ Measured at one million iterations, isolating each effect:
 * `Interp::alloc_with` (`rexx-exec/src/lib.rs:2091`), **gated on `self.stress_collect`**, which is the test-only stress mode.
 * the user-callable `GC('Force')` builtin (`builtin/state.rs:234`).
 
 There is no allocation-count threshold, no heap-size threshold, and no other trigger.
 A normal program never collects and the heap grows monotonically until the process dies.
 
 **The collector works.** Two million iterations of `x = x + 1; y = x`, both runs exiting 0 with correct output: 423,892 KB plain against 122,880 KB when forcing `gc('Force')` every hundred thousand iterations.
 
 This is a **missing trigger policy, not a broken collector and not a root leak.**
 
+> **FALSE at `c9a90906`, in its last clause.** It *is* also a root leak, in two places:
+> `Interp::loop_advance` for a counted loop's control variable, and `Interp::eval_condition` for
+> every `WHILE`/`UNTIL` test. Both push a `RootSet` temp per iteration that nothing pops until the
+> whole `DO` clause ends, and no collector can reclaim through a live root. Measured and located in
+> `phase-4d-retention.md`.
+
 **Consequence for the standard memory cap.** Under the project's `ulimit -v 1048576` this crate aborts on four of the five runnable benchmark axes, completing only `startup`.
 The previously recorded set of seven SIGABRT programs is therefore not an exotic large-string edge case: the crate exhausts memory on ordinary loops, and any memory finding taken under that cap was measured on an interpreter already out of room.
 
+> **STALE at `c9a90906`: two of the five, not four.** `compound` and `varlookup` now complete under
+> that cap. And the count itself is partly an artifact of the cap rather than of memory used, which
+> `rust/CLAUDE.md` requires every `ulimit -v` finding to say: raising the cap by exactly the 512 MiB
+> `INTERPRETER_STACK_BYTES` reserves takes `arith` from rc 134 to rc 0 with correct output, leaving
+> `strings` as the only axis that dies with the reservation free. The sentence that follows this row
+> still holds -- ordinary loops do exhaust memory, and growth is linear and unbounded, so any cap is
+> eventually hit.
+
 ## Cause 4 -- the residual
 
 The repeat-count loop isolates interpretation with no control variable and no allocation: **5.2x**, at a flat 2,816 KB.
 That is the floor the other three sit on top of, and it is the only one of the four this diagnosis does not explain further.
 
 ## What this predicts, so it can be wrong
 
 * Fixing cause 1 alone should move `varlookup` from about 23x to near the repeat-loop's 5.2x, because `varlookup.rex`'s body is two assignments against a counted loop's overhead.
 * Fixing cause 3 alone should bound RSS without moving wall time much, since the allocations still happen.
 * Fixing cause 2 should move `strings` more than the other axes.
diff --git a/docs/superpowers/plans/phase-4d-retention.md b/docs/superpowers/plans/phase-4d-retention.md
index 9aa06b96..1490177a 100644
--- a/docs/superpowers/plans/phase-4d-retention.md
+++ b/docs/superpowers/plans/phase-4d-retention.md
@@ -15,32 +15,36 @@ No optimisation is proposed here and none survives this commit.
 **A loop's memory still grows without bound, and there are two independent causes, not one.**
 The per-iteration constant has collapsed since it was last measured -- 8 bytes on the loop shape the
 task brief names, down from about 216 -- but the growth is still linear and still unbounded, and one
 of the two causes is a genuine root leak that a garbage collector would not touch.
 
 * **Cause A, the arena is never collected.**
   Every heap object a program allocates is retained for the process's whole life, because nothing
   triggers a collection.
   A retained object costs 96 bytes of arena slot plus its own payload -- 128 bytes for a short
   string, 144 for a wide decimal.
-* **Cause B, a counted loop's control temp is rooted until the loop ends.**
-  `DO i = 1 TO n` pushes one root per iteration in `Interp::loop_advance` and pops none of them
-  until the enclosing `DO` clause finishes, so the root set grows 8 bytes an iteration.
-  A collector cannot reclaim through it: the roots are live by definition, so when the control
-  variable's values are heap objects the loop pins every one of them.
+* **Cause B, a loop header's temps are rooted until the loop ends.**
+  Two sites push one root per pass and pop none of them until the enclosing `DO` clause finishes:
+  `Interp::loop_advance` for a counted loop's control variable, and `Interp::eval_condition` for
+  every `WHILE` or `UNTIL` test.
+  Either grows the root set 8 bytes a pass, and a loop with both pays both.
+  A collector cannot reclaim through them: the roots are live by definition, so when the values are
+  heap objects the loop pins every one of them.
 
 Both are defects rather than performance properties, and both are recorded in
 `phase-4-exclusions.txt` under KNOWN GAPS.
 
-**Fixing both is a win on time as well as memory.**
+**Fixing them is a win on time as well as memory.**
 Measured by prototype, then reverted: peak RSS falls 42x on `arith`, 60x on `varlookup` and 298x on
 `strings`, and `strings` gets 16% *faster*.
+Those figures cover cause A and cause B's `loop_advance` site; the `eval_condition` site was located
+and measured but not prototyped, so what closing it is worth is unquantified.
 
 ## The stale figures, and what replaces them
 
 The task brief quoted retention of roughly 216 bytes per iteration on
 `do i = 1 to n; x = x + 1; y = x; end`, 213 MB at one million iterations, and peak RSS 51 to 218
 times the oracle's.
 Those were measured at or before `107febcd`.
 Five speedups landed afterwards, two of them (`b6b1d8a9`, `e1d50dda`) directly on that loop shape.
 
 | figure | at or before `107febcd` | at `c9a90906` |
@@ -110,39 +114,58 @@ per-iteration column subtracts the 2,500 KB floor a trivial program occupies.
 | `do i = 1 to n by 1` | `nop` | 64,700 KB | 8.0 |
 | `do i = 1 to n` | `yy = 7` | 65,176 KB | 8.0 |
 | `do n` | `yy = 'abc'` | 1,002,964 KB | 128.1 |
 | `do n` | `yy = (k = 5)` | 1,002,736 KB | 128.0 |
 | `do n` | `if k = 5 then nop` | 1,001,700 KB | 127.9 |
 | `do n` | `yy = 12345678901234567890123456789` | 1,127,088 KB | 144.0 |
 | `do n` | `yy = 'a' \|\| 'b'` | 3,002,748 KB | 384.0 |
 | `do i = 1 to n` | `if i = n then nop` | 1,063,640 KB | 135.8 |
 | `do i = 1.5 to n by 1` | `nop` | 1,064,416 KB | 135.9 |
 | `do while k < n` | `k = k + 1` | 1,063,580 KB | 135.8 |
+| `do while k < n` | `k = k + 1; nop` | 1,063,848 KB | 135.9 |
 | `do forever` | `k = k + 1; if k = n then leave` | 1,001,456 KB | 127.9 |
+| `do n while zz`, `zz = 1` | `nop` | 64,468 KB | 7.9 |
+| `do n until zz`, `zz = 0` | `nop` | 63,704 KB | 7.8 |
+| `do n while zz`, `zz = 1` | `k = k + 1` | 63,956 KB | 7.9 |
 
 Read the table as four constants: **0, 8, 128 and 384**, plus 8-and-128 added together as 136.
 
 * **0 is the negative control that matters most.**
-  `do n` -- a repeat-count loop with no control variable -- retains nothing at all, at any body that
-  allocates nothing.
+  `do n` -- a repeat-count loop with no control variable and no condition -- retains nothing at all,
+  at any body that allocates nothing.
   Eight million iterations, eight million clause steps, and a flat 2.5 MB.
   So neither "iterating" nor "stepping a clause" retains anything, and any explanation that says
   otherwise is refuted by this row.
-* **8 is the counted loop's control variable**, and only the control variable.
-  It appears whenever the header is `DO i = ...` and never when it is `DO n`, `DO WHILE` or
-  `DO FOREVER`, and it does not move when the body is doubled from one `nop` to two.
+* **8 is one loop-header temp per pass, and there are two independent sources of it.**
+  A counted header contributes one, for the control variable.
+  A `WHILE` or `UNTIL` test contributes one, whatever the header around it.
+  The last three rows isolate the second source with the first absent: `do n while zz` on a bare
+  variable allocates nothing on either side of the test, has no control variable at all, and still
+  retains 7.9 bytes a pass against the same loop without the condition at 0.0.
+  `UNTIL` behaves identically, and adding a body clause does not move it.
+* **A loop pays 8 for each source it has.** `do i = 1 to n while ...` would pay 16; no row here
+  measures that combination, and the arithmetic is stated rather than measured.
 * **128 is one retained string object**, and it does not care what produced it: a literal
   assignment, a comparison's `0`, an `IF`'s test.
   `if 1 then nop` and `if k then nop` cost nothing because a literal `1` is inlined (`3799692d`) and
   a variable read allocates nothing, which is what isolates the 128 to the *comparison's result*
   rather than to `IF`.
 * **384 is three of them**: `'a'`, `'b'` and the concatenation.
+* **136 is 128 and 8 added, and reading it that way is what corrects the row above it.**
+  `do while k < n` costs 135.8 because the comparison's result is a retained heap object (128, cause
+  A) *and* the `WHILE` test leaves a rooted temp behind (8, cause B).
+  `do forever` with the same counting body costs 127.9 -- the 128 only -- because it has no `WHILE`
+  test, and the `IF` in its body does not leak: an `IF` is its own instruction, so the
+  `step_in_temps_frame` around it pops every pass.
+  `do n; if k = 5 then nop; end` at 127.9 is the same statement from the other side.
+  **The leak is `eval_condition` called from a loop header, not `eval_condition` as such**, and the
+  difference is which frame the caller runs inside.
 * **`zz = zz + 1` costs nothing** inside a `do n` loop, which is `b6b1d8a9` visible from the memory
   side: a small-integer sum is a tagged immediate and never reaches the heap.
   Give it an operand that cannot be one -- `do i = 1.5 to n`, or a 25-digit addend -- and the 128
   comes back.
 
 **The two causes are released at different times, which is the cleanest evidence that they are two
 things rather than one.**
 Eight outer iterations of a 1,000,000-iteration inner loop:
 
 | program | peak RSS |
@@ -177,53 +200,93 @@ useful heap and puts **12,582,912 of those bytes in one allocation site**:
 12,582,912 bytes over a 131,072-element capacity is **96 bytes per `Slot`**, which with the string's
 own `Vec<u8>` and the allocator's per-block overhead is the 128 bytes the table measures.
 This is `Heap.slots` in `crates/rexx-core/src/heap.rs`, and nothing ever sweeps it because nothing
 ever calls `Heap::collect`.
 
 The root that holds these objects is **nothing**.
 They are not reachable from `RootSet` at all -- the per-clause temps frame that rooted them was
 popped when the clause ended, and the variable they were bound to has been rebound many times since.
 They are retained because the sweep that would notice never runs.
 
-### Cause B -- `RootSet.temps`, grown by `loop_advance`, popped only at loop exit
+### Cause B -- `RootSet.temps`, grown by a loop header, popped only at loop exit
+
+**Two push sites, not one, and any explanation that names only the counted form is incomplete.**
+`Interp::loop_advance` pushes the control variable's previous value; `Interp::eval_condition`
+(`run.rs:6016`) pushes the result of every `WHILE`/`UNTIL` test.
+Both run inside the *same single* `step_in_temps_frame` -- the one belonging to the whole `DO`
+instruction -- so both accumulate for the loop's entire run.
+
+**The tree already said so, and this document's first version missed it.**
+`step_in_temps_frame`'s own doc comment at `run.rs:4073`-`4082` reads: "everything pushed per pass
+(`eval_condition`'s own `push_temp` for every `WHILE`/`UNTIL` test, one `ObjRef` per iteration)
+accumulates for the loop's whole run rather than one iteration's ... a `do while` running 10^7
+passes holds ~10^7 dead-but-rooted temps in one frame."
+It classifies this as a cost rather than a correctness defect, which was right when nothing
+collected and is the thing that changes once something does.
 
 For `do i = 1 to n; nop; end` at 100,000 iterations, massif's peak snapshot has a 1,075,070-byte
 useful heap and puts **1,048,576 of those bytes in one allocation site**:
 
 ```
 1,048,576B   <alloc::raw_vec::RawVec<rexx_core::handle::ObjRef>>::grow_one
              <- push_temp (roots.rs:146)
              <- loop_advance (run.rs:5654)
              <- run_repeating (run.rs:5168)
              <- step (run.rs:1734) <- step_in_temps_frame (run.rs:4206)
              <- <rexx_exec::Interp>::run_activation (run.rs:882)
 ```
 
+The `WHILE` form gives the matching profile.
+For `do n while zz; nop; end` with `zz = 1` at 100,000 passes -- no control variable, and nothing on
+either side of the test that allocates -- the peak snapshot is a 1,074,768-byte useful heap with the
+same **1,048,576 bytes** in the same vector, this time under `eval_condition (run.rs:6017)` called
+from `run_repeating (run.rs:5168)`.
+
 1,048,576 bytes over a 131,072-element capacity is **8 bytes per `ObjRef`**, which is the table's
-8-byte constant exactly.
+8-byte constant exactly, and it is the same constant on both profiles.
 
 `Interp::loop_advance` reads the control variable's previous value, calls
 `self.roots.push_temp(previous)` to root it across the arithmetic that produces the next value, and
 never pops it.
 The only thing that releases it is the enclosing `step_in_temps_frame`'s unconditional
 `pop_frame` -- and the instruction that frame belongs to is the whole `DO`, so the release happens
 when the loop finishes rather than when the iteration does.
 That is precisely the nested-loop result above: the inner loop's temps go when the inner loop ends.
 
 **This one is a root leak, and a collector cannot fix it.**
 The entries are live roots, so a mark phase must trace them.
 When the control values are tagged small integers they cost only the 8 bytes of the vector entry;
 when they are not -- `do i = 1.5 to n by 1` -- each one pins a heap object as well, which is why
 that row costs 136 bytes and not 8.
 `RootSet::pop_frame` truncates to a watermark and its own doc comment says a caller may open a frame
-and rely on the outer truncation, so the fix is a frame taken and popped inside `loop_advance`, not
-a change to the root set.
+and rely on the outer truncation, so the fix is a frame taken and popped at the push site, not a
+change to the root set.
+
+**The fix must cover both sites, and a fix to `loop_advance` alone is worse than none.**
+It would close the shape all four benchmark axes happen to use -- every one of them is
+`do i = 1 to n` -- and leave `DO WHILE` and `DO UNTIL` growing without bound, which is the shape a
+long-running service loop is actually written in.
+The suite would go green on the axes and the defect would survive in the form that matters most.
+Whoever takes this in 4d-2 should treat `do n while zz; nop; end` as the acceptance test, not
+`varlookup.rex`.
+
+**A warning about the instrument, because 4d-2 will reuse it: massif attributes the `realloc`, not
+the leaking push.**
+Both sites push into one `RootSet.temps` vector, so the frame massif names is whichever push happens
+to cross the capacity boundary -- not the one whose entries are accumulating.
+Measured on `do while k < n; k = k + 1; end`: the temps vector's growth is attributed to
+`eval_arithmetic (eval.rs:620)`, the *body's* push, while the entries piling up are
+`eval_condition`'s.
+Cause B's attribution here is sound only because each profile was taken on a shape where the leaking
+push is the only one -- a `nop` body -- and that is the discipline to copy.
+The general rule: a growing shared buffer names its last grower, so isolate the suspect push by
+removing the others rather than by reading the deepest frame.
 
 ## Whether the arena is a high-water mark
 
 **It is, and that turns out not to be the objection it looked like.**
 `Heap::collect` marks, then replaces each unreachable `Slot::Live` with a `Slot::Free` threaded onto
 a free list.
 `Heap.slots` is only ever pushed and indexed -- there is no `truncate`, no `shrink_to_fit`, no
 `clear` -- so the vector's length never falls and the pages are never returned to the allocator.
 Peak RSS is therefore a measure of the largest interval between collections rather than of live
 data, which is what Step 2 of the brief hypothesised.
@@ -258,91 +321,136 @@ program's allocation and the structure around it is irrelevant.
 ## What the fix costs, by prototype
 
 Two throwaway prototypes were built, measured, and reverted in the commit that publishes this file.
 Both are gone from the tree; `sha256sum -c` against copies taken before the edits confirms the two
 touched files are byte-identical to their pre-prototype state, and the rebuilt `rexx-run` has the
 same sha256 as the baseline binary.
 
 * **Prototype A, a trigger policy.**
   A watermark on `Heap::live_count()` in `Interp::alloc_with`: collect before allocating once the
   live count passes it, then set the next watermark to twice what survived, floor 65,536.
-* **Prototype B, the loop temp.**
+* **Prototype B, the loop temp -- `loop_advance` only.**
   `push_frame` before `loop_advance`'s `push_temp` and `pop_frame` after the increment stores its
   result.
+  **It does not touch `eval_condition`**, so every figure below bounds the counted form and says
+  nothing about what closing the `WHILE`/`UNTIL` site is worth.
+  All four benchmark axes are `do i = 1 to n`, so no axis here could have shown the difference.
 
 Every measurement below is interleaved -- base, prototype, base, prototype -- because two separate
 suite runs on this machine have invented a 5% effect that was not there.
 Every run exited 0 and printed the expected bytes.
 
+**One measurement was discarded, and it is named here so a re-measurement can be reconciled rather
+than believed.**
+A non-interleaved first pass read 18.94 s for prototype A on `strings` against 9.20 s for the base.
+Interleaved, the same comparison is 7.79 s against 9.24 s -- the opposite sign.
+The 18.94 s run was taken immediately after a base run whose 3.7 GB resident set was still resident,
+and it is contaminated by that rather than by anything in the prototype.
+No figure in this document derives from it.
+
 **Prototype A alone**, three interleaved repetitions:
 
 | axis | base RSS | A RSS | base wall | A wall |
 |---|---|---|---|---|
 | `arith` | 439,440 / 439,440 / 439,952 KB | 21,516 / 21,664 / 22,024 KB | 3.12 / 3.12 / 3.11 s | 3.27 / 3.29 / 3.23 s |
 | `compound` | 41,072 / 40,144 / 40,204 KB | 40,852 / 40,844 / 40,636 KB | 6.90 / 6.92 / 6.94 s | 6.78 / 6.85 / 6.82 s |
 | `strings` | 3,728,072 / 3,728,100 / 3,728,036 KB | 82,808 / 82,324 / 82,560 KB | 9.23 / 9.04 / 9.33 s | 9.39 / 10.14 / 10.82 s |
 | `varlookup` | 150,208 / 148,740 / 149,688 KB | 150,192 / 149,956 / 150,740 KB | 5.24 / 5.25 / 5.27 s | 5.41 / 5.42 / 8.05 s |
 
 `varlookup` and `compound` do not move at all under A, and that is the finding rather than a
 disappointment: their retention is cause B, which A does not touch.
 
+**Read the wall columns in that table with the outliers in view, because two of them will not carry
+a claim.**
+`varlookup`'s third repetition is 8.05 s against 5.41 and 5.42, and `strings`' three rise
+monotonically at 9.39 / 10.14 / 10.82, which is a drifting machine rather than a distribution.
+At that dispersion and n=3, **`arith` at +5.5% and `varlookup` at +3.4% are not distinguishable from
+noise**, and neither is `compound`'s -1%.
+What the data does carry is every RSS column, which is stable to well under a percent across
+repetitions and moves by factors rather than percents, and `strings`' wall win, which is measured
+below at seven interleaved repetitions rather than three.
+No claim in this document rests on a sub-10% wall difference.
+
 **Both prototypes**, interleaved, two repetitions except `strings` at seven:
 
 | axis | base RSS | A+B RSS | base wall (median) | A+B wall (median) |
 |---|---|---|---:|---:|
 | `arith` | 439,468 / 439,732 KB | 10,452 / 9,876 KB | 3.10 s | 3.27 s |
 | `compound` | 40,324 / 40,136 KB | 2,472 / 2,388 KB | 6.83 s | 6.78 s |
 | `strings` | 3,726,796 - 3,728,796 KB | 12,372 - 13,072 KB | 9.24 s | 7.79 s |
 | `varlookup` | 150,272 / 150,284 KB | 2,500 / 2,436 KB | 5.24 s | 5.42 s |
 | `startup` | 2,548 KB | 2,512 KB | -- | -- |
 
 So the two together are worth **42x on `arith`, 16x on `compound`, 298x on `strings` and 60x on
-`varlookup`** in peak resident set, at a wall cost between -16% and +6%.
+`varlookup`** in peak resident set.
 `varlookup` lands at 2,436 KB against the oracle's 20,992 KB, and `compound` at 2,388 KB against
 20,720 KB.
+On wall time the only movement the data supports is `strings`, below; the other three axes sit
+within the noise the previous table's outliers imply, so they are reported as unchanged rather than
+as a small cost.
 
 **`strings` gets faster, by 16%, and that is the headline for 4d-2.**
 Its base peak is 3.7 GB; holding 3.7 GB of never-reused arena destroys locality, and collecting into
 a free list that fits in cache is cheaper than the allocator's next fresh page.
 The smoke profile that put about a third of `arith`'s self time in the glibc malloc family was
 reading the same thing from the allocator's side.
 
 **The whole test suite passes with both prototypes in place**: 1,317 tests, 0 failures, exit 0.
 That is evidence and not a safety proof.
 `Heap::collect`'s own doc records one known under-rooted window -- `EXIT`'s result, from the
 wrapper's `pop_frame` through to `exit_code_for` -- and argues that nothing in that window
 allocates.
 A real trigger policy in 4d-2 must sweep `rexx-exec` for that shape rather than inherit this
 prototype's silence, because every corpus program is small enough that a rare unrooted window need
 not be hit even once.
 
-## The ruling: this is a defect
+## The ruling: This is a defect
 
 **Both causes are defects, not performance properties, and a user sees them as an abort.**
 A Rexx program whose live set is two integers should not exhaust memory, and this one does: at
 `c9a90906` under the project's standard `ulimit -v 1048576`, `arith.rex` and `strings.rex` both die
 with
 
 ```
 memory allocation of 402653184 bytes failed
 ```
 
 on stderr, exit status 134, and nothing at all on stdout.
 No Rexx condition is raised, so `SIGNAL ON SYNTAX` cannot catch it, no traceback is printed, and the
 partial output the program had already produced is lost.
-`compound.rex`, `varlookup.rex` and `startup.rex` now complete under that cap; at `107febcd` four of
-the five aborted, so the symptom has narrowed without going away.
+`compound.rex`, `varlookup.rex` and `startup.rex` complete under that cap; at `107febcd` four of the
+five aborted, so the symptom has narrowed without going away.
+
+**One of those two aborts is the cap's doing rather than the memory's, and `rust/CLAUDE.md` requires
+this to be said.**
+`ulimit -v` charges a process for address space it merely reserves, and this crate reserves
+`INTERPRETER_STACK_BYTES` -- 512 MiB, `lib.rs:309` -- for the sized interpreter thread before any
+program runs.
+Raising the cap by exactly that, to `ulimit -v 1572864`, was measured:
+
+| axis | `ulimit -v 1048576` | `ulimit -v 1572864` |
+|---|---|---|
+| `arith` | rc 134, `memory allocation of 402653184 bytes failed` | **rc 0, `4629643519330627.7808`** |
+| `strings` | rc 134, `memory allocation of 402653184 bytes failed` | rc 134, `memory allocation of 805306368 bytes failed` |
+| `compound`, `varlookup`, `startup` | rc 0 | rc 0 |
+
+So with the reservation free, **one axis dies rather than two**.
+
+**The ruling is unchanged by that, and the reason is the linearity rather than the axis count.**
+`strings` genuinely exceeds 1.5 GiB of real resident set, and growth on every shape measured here is
+linear and unbounded, so any finite limit is reached by a long enough run.
+What the correction costs is the "two of five" figure, not the finding.
 
 The severity is a function of runtime, not of program size.
 A four-line loop that is correct today fails after enough iterations, and the iteration count at
-which it fails depends on how much the body allocates -- 8 bytes an iteration for a counted loop
-that allocates nothing, 128 for one that touches a string.
+which it fails depends on how much the body allocates -- 8 bytes a pass for a counted or conditional
+loop that allocates nothing, 128 for one that touches a string.
 
 Both are recorded in `docs/superpowers/plans/phase-4-exclusions.txt` under KNOWN GAPS.
 
 ## The coverage gap that let this reach Phase 4
 
 **Nothing in the differential suite can see unbounded growth, and this is the project's primary
 instrument.**
 Two independent reasons, either of which alone is sufficient:
 
 * **The harness compares output, not memory.**
@@ -352,35 +460,49 @@ Two independent reasons, either of which alone is sufficient:
 * **Every corpus program is far too small.**
   The largest loop bound anywhere in `rust/corpus` is `to 1010`.
   At the 8 bytes an iteration measured above, that loop retains about 8 KB -- four orders of
   magnitude below anything a peak-RSS check could distinguish from noise, even if one existed.
 
 The two compound: adding an RSS assertion to the existing corpus would catch nothing, and adding a
 long-running program without an RSS assertion would catch nothing either.
 Closing this needs both -- a program whose iteration count is large enough for the linear term to
 dominate the fixed cost, and a bound on its peak resident set rather than on its output.
 
+**The benchmark suite has a narrower version of the same gap, and it nearly cost this task its main
+finding.**
+All four runnable axes are `do i = 1 to n`.
+None is a `DO WHILE`, a `DO UNTIL` or a `DO FOREVER`, so `eval_condition`'s push site is invisible
+to every one of them, and the first version of this document concluded from those four axes that the
+8-byte constant belonged to counted loops alone.
+A memory regression test built from the benchmark axes would inherit that blind spot exactly: it
+would go green on a fix that closed `loop_advance` and left `eval_condition` open.
+Whatever 4d-2 adds needs a conditional loop in it.
+
 That is a gap in the instrument the whole project's definition of correctness rests on, and it is
 why unbounded growth survived three phases of differential testing with a green suite.
 
 ## Reproducing
 
 The probe programs are throwaway and are not committed.
 Each is the loop in the tables above, written into a fresh empty directory, run with an absolute
 path:
 
 ```sh
 ( ulimit -v 8388608; /usr/bin/time -v /abs/path/rexx-run /abs/path/probe.rex )
 ```
 
 Check the exit status and stdout of every run before reading the resident set.
 A program that dies on line 1 reports a small, stable and entirely meaningless figure, and that
 mistake has already been made once against this very finding.
 
-The two allocation sites:
+The allocation sites:
 
 ```sh
 valgrind --tool=massif --stacks=no --detailed-freq=1 --massif-out-file=out /abs/path/rexx-run /abs/path/probe.rex
 ms_print --threshold=1.0 out
 ```
 
+Run each site on a probe where it is the only push into the buffer in question -- a `nop` body for
+either root-set site -- or massif names the last grower rather than the leaker, per the warning
+under Cause B.
+
 100,000 iterations is enough for both, because the growth is linear from there up.
