# Task 21, re-review 4 (closing scoped check on fix round 4)

Range `f08353cd3..HEAD` (`a80d5816c`, `4bc9ebe61`). Scope: only the prose this round added or
changed. The task itself is not re-reviewed. Nothing was rebuilt; the tree was not modified except
this file.

**Verdict: CHANGES REQUIRED.** One sentence this round added is false of the tree as it stands, and
it is false under both readings of its own subject. One further item breaks the plan's own
comment rule. Everything else this round wrote survives being run, including the two items that had
never been executed before (the F1 exhibit and the reply row's stability figure).

## The false sentence

`rust/crates/rexx-exec/tests/dispatch_seam.rs:230`-`:232`, inside
`heap_collect_is_called_from_collect_now_alone`:

```
    // Whitespace-collapsed, so a wrapped call is still one match. Comment
    // lines go first, for the reason `code_occurrences` strips them: the
    // needle is an ordinary phrase and this file's own prose uses it.
```

The offending clause is **"the needle is an ordinary phrase and this file's own prose uses it"**,
offered as the reason the loop drops comment lines. It is false either way you read "this file":

* **"this file" = `dispatch_seam.rs`** (which is what it means three lines above, in "so this file's
  own text does not contribute to the count it takes"). The loop iterates `source_files()`, which is
  `CARGO_MANIFEST_DIR/src` recursively -- `rexx-exec/src/` alone. `tests/dispatch_seam.rs` is never
  read, so its prose cannot reach the count whether comments are stripped or not. This file's own
  module doc says so in as many words at `:93`: *"The scan reads `src/` and never this file, so
  nothing here can satisfy its own assertion."* The two sentences contradict each other inside one
  file.
* **"this file" = each scanned `src/` file.** Measured: no comment anywhere under
  `rexx-exec/src/` contains `heap.collect(`. I ran the scan both ways over the tree as it stands --
  comments stripped and comments kept -- and both give exactly `1`, at `lib.rs`. The stripping
  changes nothing today, so no scanned file's prose uses the needle either.

The decision to strip comments is right (prospective defence, and the safe direction is a false
red). The reason written beside it is not a fact about this tree. This is the round's own dominant
failure mode -- a sound decision shipped with a justification that does not survive being run --
recurring inside the fix for it.

## Rule violation

`dispatch_seam.rs:220` names the size of a set: **"Two spellings that would have escaped a narrower
needle do **not** escape this one"**. `global-constraints.md:172` is flat about it -- *"A comment may
not name the size of a set. Name the set; true counts included."* -- and this round's own brief
applied that same rule to `phase-5a.txt`'s "Three rows here are...", which likewise enumerated its
members straight afterwards. Both members are named in the same sentence, so the count carries
nothing and the set of spellings a narrower needle would have missed is open (the reviewer named
five; two are now caught).

Recorded as a judgment call rather than a certainty for one reason: the same file's module doc
already carries the same shape at `:50` -- "What it counts is two literal token spellings and the
items inside one brace-matched region of one file" -- pre-existing and unflagged through several
rounds. If that one is licensed, so is this one; if it is not, this round added a second.

## Observations (not blocking, both pre-existing in origin)

* `rust/corpus/phase-5a.txt:635` now opens **"The `class_context_*` rows beside them are the context
  object's own identity rather than the package's."** The glob matches four rows in this file, and
  one of them -- `lang/class_context_package.rex` -- is a package row, the `RexxContext~package` row
  the previous sentence describes. "beside them" does exclude it, so the sentence is true as
  written; but the name chosen for the set picks out a superset containing the row it contrasts
  against, which is what the cardinality fix was supposed to avoid. The old text ("Three rows here")
  had the count wrong and the membership right; the new text has the membership expressible only via
  the qualifier.
* The same `phase-5a.txt` paragraph describes five things for six rows:
  `lang/class_package_addition_refused.rex` has no description in it. Its twin in `coverage.rs` does
  have one ("the refusal the REXX package gives an addition"). The omission is in the sentence this
  round did **not** touch, so it is not this round's, but the two comments now describe different
  sets.

## What I verified, and how

### By execution

* **F1's replacement exhibit reproduces exactly as written.** Oracle:
  `a = "-140404878001713"; b = "-140404878167489"` then `say (a = b) (a == b)` prints `1 0`, rc 0.
  Both engines agree byte for byte (`tree-walker` `1 0`, `ir` `1 0`, rc 0, empty stderr).
* **The added sentence about the literal form is true.** `say (-140404878001713 ==
  -140404878167489)` prints `1` on the oracle and on both engines; `say -140404878001713` and
  `say -140404878167489` each print `-1.40404878E+14`, so "compares two copies of `-1.40404878E+14`"
  is right and "it is not this rule" follows.
* **"which is how they arrive from `~identityHash`" is true.** Two fresh `.object~new` on the oracle:
  `length(ha) length(hb)` is `16 16` (so "more than nine digits"), and `(ha = hb) (ha == hb)` is
  `1 0` -- the exhibit's own shape, taken from `~identityHash` rather than from literals.
* **`class_context_reply.rex`'s new positive figure is true.** "this program gives one hash over
  twenty runs of the oracle and twenty of each engine": 20 oracle runs, 20 `tree-walker`, 20 `ir` --
  all 60 produce the identical stdout hash `81ab900e2b48`, rc 0, empty stderr, output
  `replied / 1 / 1 / main done`.
* **The dropped count was right to drop.** I reconstructed the loud variant (parked body ending
  `say .context~objectName`) and ran it 20 times on the oracle: **three** distinct outputs, split
  9/4/7. The re-reviewer measured three split 12/6/2; the round before recorded two. So "more than
  one distinct output over twenty oracle runs, reproducibly, and how many is not a stable quantity
  to write down" is exactly what the measurements support.
* **The widening is real, and the blind-spot bullets are accurate.** I reimplemented both needles
  (old: `heap.collect(&` per line; new: `heap.collect(` over whitespace-collapsed, comment-stripped
  text) and ran them over a copy of `rexx-exec/src` with each bypass spelling injected as an extra
  file:
  * `interp.heap.collect(roots)` (argument already a reference): old **1** = green, new **2** =
    red. Caught, as claimed.
  * the same call wrapped across three lines: old **1** = green, new **2** = red. Caught, as
    claimed.
  * `rexx_core::Heap::collect(&mut interp.heap, &interp.roots)`: both **1** = green. Escapes, as the
    UFCS bullet says.
  * `let h = &mut interp.heap; h.collect(&interp.roots);`: both **1** = green. Escapes, as the
    rebinding bullet says.
  * Baseline with no mutation: **1**, at `lib.rs`, under both needles.
* **The `collect_now` arm still fires under the new needle.** Simulated the second assertion: the
  signature `    fn collect_now(&mut self) {` is found at `lib.rs:6010`, the brace rule takes a
  2312-byte body, and that body contains `heap.collect(`.
* **The scan's reach.** The walker is `rexx-exec/src` only, so the third bullet
  ("`rexx-core`'s own code ... is invisible to it") is true; `rexx-core/src/heap.rs:531` is the only
  other `heap.collect(` in a crate source, and it is inside `#[cfg(test)] mod retire_tests`, so it
  is not a live door.
* **F2's replacement clause: no corpus program reaches any remaining door, because there is no
  remaining door.** The one call to `Heap::collect` in `rexx-exec` is `lib.rs:6034`, inside
  `collect_now`; `rexx-core`'s only other call is in a test module.
* **The sourceline fixture.** `class_context_reply.rex` is 32 lines (trailing newline present), the
  fixture header reads `count 32`, and the fixture body is byte-identical to the corpus file.
* **The sitting's figures are the recorded rows.** `across_builds`/`instructions:u`/`ir`/`small` for
  `21-fixround-5-comments`: `alloc4c` 1.003705, `arith` 0.988671, `strings` 1.011944 -- identical to
  six decimals to `21-fixround-4-comments` -- and `rexxcps` 1.019037 against 1.019048. All four
  match the report. The 172 added rows are all tagged `21-fixround-5-comments` at `a80d5816c`,
  continuing the existing one-ahead tag convention (`21-fixround-3` was added by "Record fix round
  2's sitting").
* **ASCII.** No non-ASCII byte in any line this round touched. `lib.rs` has four non-ASCII lines
  (`:269`, `:293`, `:500`, `:2391`), all pre-existing and far from the changed region.

### By reading code

* `Heap::collect(&mut self, roots: &RootSet)` at `rexx-core/src/heap.rs:137`, `pub use
  heap::{CollectStats, Heap}` at `rexx-core/src/lib.rs:36`, and `Interp { heap: Heap, roots:
  RootSet }` at `rexx-exec/src/lib.rs:2645`-`:2646` -- disjoint field borrows, so
  `rexx_core::Heap::collect(&mut interp.heap, &interp.roots)` is a well-formed in-crate expression.
  I did **not** rebuild, so "compiles" and "reddens `class_context_gc.rex`" are read, not run; what I
  did run is that it passes the assertion **green**, which is the half that matters for the doc's
  honesty.
* F3's "Can be non-zero" is consistent with both neighbours: `COLLECT_FLOOR`'s doc 40 lines above
  ("a program that allocates a few hundred values -- which is most of the corpus -- never collects
  at all") and `class_context_identity.rex`'s header ("the collector's floor is far above 300 slots,
  so no collection happens here"). The added "Most programs never reach the watermark and do read
  `0`" says the same thing in the same direction. `collect_policy.rs`'s churn program does read 6 --
  the file states it twice, at its doc and in the failure message, and asserts `> 0` and `<= 12`
  around it. The stranded `What` is gone.
* `coverage.rs:1141`-`:1148` describes exactly the six rows it precedes: `RexxContext~package`, the
  two class tables, the refusal, one per activation, running-or-suspended, parked-by-REPLY. The
  "running or suspended" wording matches `class_context_gc.rex`'s own header, which calls the
  suspended half "the half the single-activation rows cannot reach", and matches `collect_now`'s
  body, which chains `self.running` and `self.suspended`.
* No historical framing was added in `src/`. The one historical clause in the round's new prose
  ("the door that produced those two rows was found by hand rather than by a gate") is in `tests/`
  and preserves the content of the sentence it replaced.

### On trust

* That `class_context_gc.rex` is red, and that `GC('Force')` reaching `Heap::collect` directly
  refuses the next `.CONTEXT` send at rc 120 against the oracle's rc 0. Both are fix round 2's
  measurements against a build I am not permitted to make. The mechanism is consistent with
  `collect_now`'s body and with the row's own header, but I did not reproduce it.
* That the UFCS bypass compiles and reddens the corpus row (see above).
* The five gates, the 232-of-232 corpus, and the 98 `test result: ok` lines, which the controller
  ran.
