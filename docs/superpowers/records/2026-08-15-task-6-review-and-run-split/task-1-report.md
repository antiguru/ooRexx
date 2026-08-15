# Task 1 report: close the `INTERPRET` regression `e74780054` caused

**Commit:** `11638b91e` -- *Give an INTERPRET fragment the condition queue the oracle gives it*
**Tree at start:** `ac09e3abe` (which contains `56d9d1c86`).
**Status:** DONE_WITH_CONCERNS. The brief's prescribed mechanism was built, measured, and rejected;
a different mechanism closes the divergence and every sibling. Details in *Deviation from the brief*.

## What I implemented

Not the suppression the brief's Step 2 names. That suppression was built and measured first and is
wrong (evidence below). What landed instead reconstructs the thing the brief's own rationale
describes: the fragment's separate condition queue.

* `Interp::fragment_depth` (`rust/crates/rexx-exec/src/lib.rs`) counts how many `INTERPRET`
  fragments are running. The `Interpret` arm of `step` increments it around `run_fragment` and
  decrements it after.
* `PendingTrap::fragment_depth` records that counter at the moment the condition was queued.
* `Interp::deliver_pending_traps` matches a queued condition's depth against the depth the boundary
  is reached at, alongside the activation identity that was already there.
* The `Interpret` arm drops any entry still queued at the fragment's own depth when the fragment
  ends, on the failing path as well as the succeeding one.

Nothing from `e74780054` was reverted: the plain `DO`'s header clause and `END` clause stay exactly
as that commit left them.

## Deviation from the brief, and why

Step 2 says: give the new header and `END` clauses no boundary when `clause_line_override` is in
force. I built exactly that (`enter_clause` / `leave_clause_without_boundary` at both sites, gated
on `self.clause_line_override.is_some()`), built the release binary, and measured. It closes the
task's program and keeps the empty-fragment shape -- and it **regresses** a shape that agrees today:

Program (`p6`), a condition queued **inside** the fragment:

```rexx
call on user c1 name h1
call on user c2 name h2
interpret 'zq = raiser(); do; say ''body''; end'
say 'after' zq
exit 0
raiser:
raise user c1 return 5
h1:
say 'h1' sigl
raise user c2 return 1
h2:
say 'h2' sigl
return
```

| | stdout | stderr | rc |
|---|---|---|---|
| oracle | `h1 3` / `h2 3` / `body` / `after 5` | empty | 0 |
| `ac09e3abe`, both engines | `h1 3` / `h2 3` / `body` / `after 5` | empty | 0 |
| brief's Step 2 suppression, both engines | `h1 3` / `body` / `h2 3` / `after 5` | empty | 0 |

So the oracle *does* deliver at the `DO`'s header clause inside a fragment, when the condition was
queued inside that fragment. The fragment's boundaries are live; what is separate is the queue. The
brief's Step 3 says a suppression that breaks a control is the wrong suppression, and this is that
case with a different control than the one Step 3 named.

The suppression was reverted (`git checkout --` after copying the file aside) before the landed work
began; it is in no commit.

## Every transcript I measured

All programs share this surrounding text, with only the `interpret` clause differing (`p6`--`p10`
have their own text, given inline below):

```rexx
call on user c1 name h1
call on user c2 name h2
zr = raiser()
<the interpret clause>
say 'after' zr
exit 0
raiser:
raise user c1 return 5
h1:
say 'h1' sigl
raise user c2 return 1
h2:
say 'h2' sigl
return
```

Oracle runs: `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib
/home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`, each from its own fresh directory, three
descriptors read separately. Engine runs: `REXX_ENGINE=tree-walker|ir target/release/rexx-run FILE`.
**stderr was empty and rc was 0 in every row of every table below**, oracle and both engines.

### `p1` -- the task's program, `interpret 'do; say ''body''; end'`

| | stdout |
|---|---|
| oracle | `h1 3` / `body` / `h2 4` / `after 5` |
| `ac09e3abe`, both engines | `h1 3` / `h2 4` / `body` / `after 5` (diverges -- reproduced) |
| `11638b91e`, both engines | `h1 3` / `body` / `h2 4` / `after 5` (matches) |

### `p2` -- the empty-fragment shape, `interpret 'do; end'` (brief Step 3)

| | stdout |
|---|---|
| oracle | `h1 3` / `h2 4` / `after 5` |
| `ac09e3abe`, both engines | `h1 3` / `h2 4` / `after 5` (matches) |
| brief's Step 2 suppression, both engines | `h1 3` / `h2 4` / `after 5` (matches) |
| `11638b91e`, both engines | `h1 3` / `h2 4` / `after 5` (matches) |

**Correction to the brief.** Step 3 says the reviewer established that this shape "diverges
identically before and after `e74780054`". At the tree I was given it does not diverge at all: the
oracle and both engines agree. The reviewer's measurement predates `56d9d1c86` (the drain), which is
the likeliest absorber; I did not go back and confirm that, because the shape agrees now and my
change leaves it agreeing.

### `p3` -- sibling, `interpret 'if 1 = 1 then say ''body'''` (brief Step 4)

| | stdout |
|---|---|
| oracle | `h1 3` / `body` / `h2 4` / `after 5` |
| `ac09e3abe`, both engines | `h1 3` / `h2 4` / `body` / `after 5` (diverges) |
| `11638b91e`, both engines | `h1 3` / `body` / `h2 4` / `after 5` (**closed**) |

### `p4` -- sibling, `interpret 'do zi = 1 to 1; say ''body''; end'` (brief Step 4)

| | stdout |
|---|---|
| oracle | `h1 3` / `body` / `h2 4` / `after 5` |
| `ac09e3abe`, both engines | `h1 3` / `h2 4` / `body` / `after 5` (diverges) |
| `11638b91e`, both engines | `h1 3` / `body` / `h2 4` / `after 5` (**closed**) |

Both named siblings are closed by the same mechanism, which is what the controller's resolution 3
allows.

### `p5` -- no construct in the fragment at all, `interpret 'say ''a''; say ''b'''`

| | stdout |
|---|---|
| oracle | `h1 3` / `a` / `b` / `h2 4` / `after 5` |
| `ac09e3abe`, both engines | `h1 3` / `a` / `h2 4` / `b` / `after 5` (diverges) |
| `11638b91e`, both engines | `h1 3` / `a` / `b` / `h2 4` / `after 5` (**closed**) |

This one is worth flagging: it is the same divergence with no `DO`, no `IF` and no loop anywhere,
which is what says the defect was never about the `DO`'s clauses.

### `p6` -- condition queued **inside** the fragment (the adjacent success)

Program text is in *Deviation from the brief* above.

| | stdout |
|---|---|
| oracle | `h1 3` / `h2 3` / `body` / `after 5` |
| `ac09e3abe`, both engines | `h1 3` / `h2 3` / `body` / `after 5` (matches) |
| brief's Step 2 suppression, both engines | `h1 3` / `body` / `h2 3` / `after 5` (**regresses**) |
| `11638b91e`, both engines | `h1 3` / `h2 3` / `body` / `after 5` (matches) |

### `p7` -- the same without the `DO`, `interpret 'zq = raiser(); say ''body'''`

Surrounding program has no `zr = raiser()` line; the `interpret` is on line 3.

| | stdout |
|---|---|
| oracle | `h1 3` / `body` / `h2 3` / `after 5` |
| `ac09e3abe`, both engines | `h1 3` / `body` / `h2 3` / `after 5` (matches) |
| `11638b91e`, both engines | `h1 3` / `body` / `h2 3` / `after 5` (matches) |

### `p8` -- a condition left queued when the fragment ends, `interpret 'zq = raiser()'`

| | stdout |
|---|---|
| oracle | `h1 3` / `after 5` |
| `ac09e3abe`, both engines | `h1 3` / `h2 3` / `after 5` (diverges) |
| `11638b91e`, both engines | `h1 3` / `after 5` (**closed**) |

The oracle drops the requeue outright. This is the other half of the same fact, and it is what makes
the key an equality rather than an "at or outside" comparison.

### `p9` -- the same one fragment deeper, `interpret 'interpret "zq = raiser()"'`

| | stdout |
|---|---|
| oracle | `h1 3` / `after 5` |
| `ac09e3abe`, both engines | `h1 3` / `h2 3` / `after 5` (diverges) |
| `11638b91e`, both engines | `h1 3` / `after 5` (**closed**) |

### `p10` -- a second fragment after one that left a condition queued

```rexx
call on user c1 name h1
call on user c2 name h2
interpret 'zq = raiser()'
interpret 'say ''second'''
say 'after' zq
exit 0
... same handlers ...
```

| | stdout |
|---|---|
| oracle | `h1 3` / `second` / `after 5` |
| `ac09e3abe`, both engines | `h1 3` / `h2 3` / `second` / `after 5` (diverges) |
| `11638b91e`, both engines | `h1 3` / `second` / `after 5` (**closed**) |

This is the shape that makes the discard observable rather than merely tidy: without it, the first
fragment's leftover is delivered inside the *second* fragment, which runs at the same depth.

## The witness

`rust/crates/rexx-exec/tests/ir_dual_cases/interpret-condition-queue`, a new case file with its own
header, per the controller's resolution 1. `loop-header-boundaries` was not touched. Seven stanzas:
`p1`, `p6`, `p5`, `p4`, `p8`, `p10`, `p9`. Every expected block is the oracle's bytes, measured
before it was written.

**Which harness gates it.** `both_engines_agree_on_every_case_file` in `ir_dual`, under a plain
`cargo test --release --workspace`. `corpus_differential` does **not** gate it in either mode -- see
the mutation table, where the STRICT corpus gate exits 0 under both mutations.

## Mutation evidence

Two mutations, each reverting one behavioural half of the fix, applied to the committed source.
Command in every case:

```
memcap 8G cargo test --release --workspace --no-fail-fast -j 2
```

plus, separately, `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus`.

**M1 -- delete the depth half of the delivery key.** In `deliver_pending_traps`, the predicate
becomes `pending.activation == here` alone.

* Suite exit 101. **Every test that caught it: `ir_dual::both_engines_agree_on_every_case_file`, and
  nothing else.** The failing stanza is the `p1` one, `interpret-condition-queue:33`, actual
  `h1 3 / h2 4 / body / after 5` against expected `h1 3 / body / h2 4 / after 5`.
* STRICT corpus gate under M1: **exit 0** -- does not catch it.
* **"Can fail" versus "adds coverage":** the same mutation with the new case file *deleted* runs the
  whole workspace suite to **exit 0**, no failures. So the file adds coverage; nothing existing
  reaches this.

**M2 -- delete the discard at the fragment's exit.** The `retain` in the `Interpret` arm is removed;
the depth counting and the delivery key stay.

* Suite exit 101. **Every test that caught it: `ir_dual::both_engines_agree_on_every_case_file`, and
  nothing else.** The failing stanza is the `p10` one, `interpret-condition-queue:161`, actual
  `h1 3 / second / h2 4 / after 5` against expected `h1 3 / second / after 5`.
* STRICT corpus gate under M2: **exit 0** -- does not catch it.
* With the new case file deleted: **exit 0**, no failures. Adds coverage.

**The `p6` stanza is the adjacent success and is red under a different mutation**: it stays green
under M1 and M2, and it is what the brief's Step 2 suppression reddens (`h1 3 / body / h2 3 /
after 5` against the oracle's `h1 3 / h2 3 / body / after 5`, measured on a built binary, both
engines). It is in the file to keep a future "just suppress the boundary" fix from passing.

Note on granularity: `datadriven` stops at the first failing stanza in a file, so the transcripts
above name the first stanza each mutation breaks, not all of them. The reported catcher set is at
test granularity, which is what was asked.

## Before/after sweep (brief Step 6)

Two binaries built from the same tree, one at `ac09e3abe`'s sources and one at the committed
sources, then every `.rex` under `rust/corpus/` and `rust/bench-programs/` run under both
`REXX_ENGINE` values, stdout, stderr and exit status each captured to its own file.

* 84 programs x 2 engines x 3 descriptors = 504 captured files per arm.
* `diff -rq before after`: **no output. Nothing moved, not one program, on either engine.**
* Re-run against the final committed binary after the comment corrections: **identical to `before`**
  again.
* Negative control that the sweep instrument is live: the same two binaries on `p1` under
  `REXX_ENGINE=ir` print `h1 3 / h2 4 / body / after 5` and `h1 3 / body / h2 4 / after 5`
  respectively. The binaries differ; the corpus cannot see it.

That last point is the whole content of Step 7: this sweep is blind to the shape in both directions.

## Step 7: the previous plan's sweep sentence

`docs/superpowers/plans/2026-08-14-pre-phase-5-defects.md`, the Task 6 OUTCOME "Witnesses"
paragraph. The original sentence is kept (it is true as measured) and the claim it invites is
corrected after it: the sweep runs only what those two directories hold, it did not witness the
regression this task created, and Task 1's own sweep over the same directories likewise moves
nothing -- which is the measurement, not an assertion, that the sweep cannot see the shape. No
number I did not measure is restated.

## Gates

Each run unpiped from `rust/`, exit status read on its own.

| gate | command | exit |
|---|---|---|
| fmt | `cargo fmt --all --check` | 0 |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| clippy, clean target | `cargo clean` then the same command | 0, and no `warning`/`error` line in the log |
| tests | `memcap 8G cargo test --release --workspace --no-fail-fast -j 2` | 0, no `FAILED` |
| corpus gate | `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus` | 0, log reports `mode: STRICT (the gate)` |

**Clippy was run from a fully clean target directory** (`cargo clean`, 4.4 GiB removed) and exited 0
with zero warning or error lines. It was re-run after the final comment corrections, exit 0.

A note on the harness: `memcap 8G cargo test --release --workspace` OOM-kills at the cap during
*compilation* (fat LTO, parallel codegen), not during the tests. `--no-run -j 2` first, or `-j 2` on
the combined run, stays under it. The target is `corpus`, not `corpus_differential` --
`corpus_differential` is a test *inside* `tests/corpus.rs`; `--test corpus_differential` exits 101
with "no test target named".

## Files changed

* `rust/crates/rexx-exec/src/lib.rs` -- `PendingTrap::fragment_depth`, `Interp::fragment_depth`, its
  initialiser.
* `rust/crates/rexx-exec/src/run.rs` -- the depth around `run_fragment`, the discard at the
  fragment's exit, the depth half of the delivery key, and the queue site.
* `rust/crates/rexx-exec/src/clause.rs` -- one paragraph in the module doc saying the separation is
  reconstructed at the delivery rather than by withholding boundaries.
* `rust/crates/rexx-exec/tests/ir_dual_cases/interpret-condition-queue` -- new.
* `docs/superpowers/plans/2026-08-14-pre-phase-5-defects.md` -- Step 7's correction.

## Self-review findings, fixed before the commit

* **Two comments named the size of a set** (`the three above`, `which has the two transcripts`) and a
  third said `both transcripts`. All three rewritten to name the set instead, per `rust/CLAUDE.md`.
* **The plan-file sentence was garbled** on first writing ("and one this task created it did not").
  Rewritten.
* **Checked for em-dashes** in every file I touched: none.
* **Checked the binary was not stale** before each measurement round, by timestamp against the
  sources and by re-running `p1`.
* **Checked the discard cannot underflow or be skipped**: the `+= 1` and the `-= 1` are in one
  straight-line block with no `?` and no early return between them; `run_fragment`'s result is bound,
  not propagated, so the discard runs on the failing path too.

## Issues and concerns

1. **The brief's Step 2 mechanism is wrong and the plan should say so.** Recorded here and in the
   commit message, and the code comments state the measurement. The plan text itself I have not
   edited, because a later task in this plan owns that file and two owners is what the controller
   warned about; the controller may want to correct Step 2 in place so a future fix round does not
   receive the same instruction.
2. **Step 3's premise about `interpret 'do; end'` does not hold at this tree** -- it agrees
   everywhere. Written up above.
3. **Wider scope than the brief names, and it is two separate widenings, not one.** *(Corrected in
   fix round 1; the original text here claimed all of it was mechanically forced, which my own
   mutation M2 refutes.)*
   * **Forced by the delivery key.** `p5` is closed by the same predicate that closes `p1`: both are
     a condition queued outside the fragment being offered a boundary inside it, and no version of
     the key closes one and leaves the other open.
   * **Forced by the delivery key, second measurement.** `p8` and `p9` too. *(Corrected in fix
     round 2: round 1's text attributed these to the discard, and building the no-discard binary
     shows otherwise.)* With the discard removed, `p8` prints `h1 3 / after 5` and `p9` prints
     `h1 3 / after 5`, both the oracle's bytes on both engines -- the key alone leaves a deeper
     entry unmatched at every outer level, so it never gets delivered whether or not it is dropped.
   * **Separable, and taken deliberately: `p10` alone.** It is the only measured shape the *discard*
     closes. M2 is the proof: it keeps the delivery key and removes the discard, and `p10` is the
     stanza that fails, printing `h1 3 / second / h2 4 / after 5` against the oracle's
     `h1 3 / second / after 5`. The discard stays because that closure has an oracle transcript and
     a stanza, and because leaving undeliverable entries queued is a slow leak in a program that
     runs `INTERPRET` in a loop. But it is scope taken on purpose, not scope the fix forced.

### Found and NOT fixed

Nothing. Every divergence I found in this family is closed by the commit, and I found none outside
it. For completeness, the shapes I probed that already agreed and still agree: `p2`, `p6`, `p7`.

**One thing I did not probe and a reviewer may want to:** a condition queued inside a fragment whose
*target activation* is not the fragment-running one, in combination with a nested fragment. The
depth is not saved across a call, which is stated and reasoned at the field, and `p6`/`p7` measure
the ordinary call-from-inside-a-fragment case; the deeper cross of call and nesting is untested.

---

# Fix report, round 1

**Commit:** `e45dccf86` -- *Pin the nesting level the fragment queue is keyed on, and say what each
row pins*. All four Important findings and all four Minors addressed. No finding required a code
behaviour change; one added an assertion.

## I1 -- the "depth rather than a flag" claim, and the program that actually pins it

**The finding is right that the claim was false, and its proposed program does not fix it.** I
settled this by construction rather than by tracing, as asked: I built the boolean design the
finding describes -- `std::mem::replace(&mut self.fragment_depth, 1)` on entry, the same discard,
the enclosing value restored on exit, which is exactly a saved/restored `in_fragment` flag encoded
in the existing field -- built the release binary from it, and ran every stanza's program plus both
candidates under it and under the committed design.

| program | oracle | depth design | flag design | discriminates? |
|---|---|---|---|---|
| `p1` `interpret 'do; say ''body''; end'` | `h1 3 / body / h2 4 / after 5` | same | same | no |
| `p6` `'zq = raiser(); do; say ''body''; end'` | `h1 3 / h2 3 / body / after 5` | same | same | no |
| `p5` `'say ''a''; say ''b'''` | `h1 3 / a / b / h2 4 / after 5` | same | same | no |
| `p4` `'do zi = 1 to 1; say ''body''; end'` | `h1 3 / body / h2 4 / after 5` | same | same | no |
| `p3` `'if 1 = 1 then say ''body'''` | `h1 3 / body / h2 4 / after 5` | same | same | no |
| `p8` `'zq = raiser()'` | `h1 3 / after 5` | same | same | no |
| `p10` two fragments | `h1 3 / second / after 5` | same | same | no |
| `p9` `'interpret "zq = raiser()"'` | `h1 3 / after 5` | same | same | **no** |
| `r1` `'zq = raiser(); interpret "say 1"; say 2'` | `h1 3 / 1 / h2 3 / 2 / after 5` | same | same | **no** |
| `r2` `'zq = raiser(); interpret "say 1; say 2"; say 3'` | `h1 3 / 1 / 2 / h2 3 / 3 / after 5` | matches oracle | `h1 3 / 1 / h2 3 / 2 / 3 / after 5` | **YES** |

Both engines agree with each other on every cell; oracle stderr empty and `rc 0` throughout.

**`r1` is the finding's proposed program, and it does not discriminate.** The reason is that under a
flag the delivery lands at the *first* boundary inside the inner fragment, and under a depth it
lands at the enclosing clause's boundary -- and when the inner fragment has exactly one clause,
those two moments have the same content printed before them. The inner fragment needs a second
clause for the delivery to fall *between* them. `r2` is `r1` with `say 1` split into `say 1; say 2`,
and that is the whole difference.

**Harness-level confirmation, which is stronger than the table.** With the flag design applied and
the amended case file in place, `both_engines_agree_on_every_case_file` fails at
`interpret-condition-queue:252`, the `r2` stanza, showing actual `h1 3 / 1 / h2 3 / 2 / 3 / after 5`
against expected `h1 3 / 1 / 2 / h2 3 / 3 / after 5`. `datadriven` stops at the first failing
stanza, and `r2` is the **last** stanza in the file -- so that single failure is also the statement
that every other row passed under the flag design. That is the by-construction answer to "which
design does each row distinguish": all of them agree with a flag, and only `r2` does not.

`r2` is added as the file's last stanza with a comment saying what it pins and recording that the
one-clause version does not. The `p9` row's comment no longer claims to make the depth/flag
distinction; it now says what it does pin (the discard reaching a nested fragment) and that it
agrees with a flag.

## I2 -- the same false justification in `Interp::fragment_depth`

Replaced. The doc now cites `r2` as the program that separates the designs, states why the inner
fragment needs more than one clause, and says that every shorter shape agrees under both and that
this was established by building the flag and running it.

## I3 -- the compiled-engine sentence on the controlled-loop row

Verified the finding: `run_fragment` calls `run_bounded(..., BodyEngine::TreeWalker)`, and
`BodyEngine`'s own doc says a fragment is never `Chunk`. So no loop inside an `INTERPRET` reaches
`Op::LoopRun` on either arm. The sentence is gone. The row's comment now says what is still true --
a controlled loop's header and per-pass re-test clauses are boundaries a plain `DO` does not have,
opened by `run_repeating` rather than the `Simple` arm.

## I4 -- the report's justification for the extra scope

Corrected in *Issues and concerns* item 3 above, not in code, per the controller's ruling. The
claim "there is no version of this fix that closes `p1` correctly and leaves them open" was refuted
by my own M2. It now separates the two widenings: `p5` is forced by the delivery key, and `p8`,
`p9`, `p10` are closed by the discard, which is separable and taken deliberately. The first commit's
message carries the older framing and cannot be edited; this report and the fix commit's message
carry the correction.

## Minors

* **M5** -- "three parts" removed; the parts are named, and a fourth (the nesting level) is named
  with them.
* **M6** -- the ordinal "the second row" is replaced by the row's program shape,
  `zq = raiser(); do; say 'body'; end`.
* **M7** -- `debug_assert!` added at the discard: every surviving entry has
  `fragment_depth < depth`. **Proved live rather than assumed:** with the `retain` removed, the debug
  build panics `a condition queued inside a fragment outlived that fragment's own exit` at
  `run.rs:1709`, exit 101. With the `retain` in place the whole debug workspace suite is green and it
  never fires.
* **M8** -- `p3` (`if 1 = 1 then say 'body'` in a fragment) added as a stanza of its own.

## Tests covering the amended code

| what | command | result |
|---|---|---|
| the case file's harness, release | `memcap 8G cargo test --release -p rexx-exec --test ir_dual both_engines_agree_on_every_case_file` | `ok. 1 passed; 0 failed` |
| the same, **debug** (the new assertion is live there) | `memcap 8G cargo test -p rexx-exec --test ir_dual -j 2` | `ok. 9 passed; 0 failed` |
| whole workspace, **debug**, for the assertion | `memcap 8G cargo test --workspace -j 2` | exit 0, no `FAILED`, assertion never fired |
| whole workspace, release | `memcap 8G cargo test --release --workspace --no-fail-fast -j 2` | exit 0, no `FAILED` |
| corpus gate | `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus` | exit 0, `mode: STRICT (the gate)` |
| fmt | `cargo fmt --all --check` | exit 0 |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |

**Mutations re-run against the amended file**, since the stanza set changed:

* **M1** (delete the depth half of the delivery key): suite exit 101, **only**
  `ir_dual::both_engines_agree_on_every_case_file`; first failing stanza now
  `interpret-condition-queue:42` (`p1`). STRICT corpus gate exit 0, still does not catch it.
* **M2** (delete the discard): suite exit 101, **only**
  `ir_dual::both_engines_agree_on_every_case_file`; first failing stanza now
  `interpret-condition-queue:194` (`p10`). STRICT corpus gate exit 0.
* **The flag design** (a third mutation, added this round): case-file test exit 101, first and only
  failing stanza `interpret-condition-queue:252` (`r2`).

**Sweep re-run** against the final binary: every `.rex` under `corpus/` and `bench-programs/`, both
engines, three descriptors, `diff -rq` against the pre-change capture -- **identical, nothing moved.**

## Files changed this round

* `rust/crates/rexx-exec/src/lib.rs` -- I2.
* `rust/crates/rexx-exec/src/run.rs` -- M7.
* `rust/crates/rexx-exec/tests/ir_dual_cases/interpret-condition-queue` -- I1, I3, M5, M6, M8.
* this report -- I4.

## Concerns from this round

1. **The finding's proposed program was insufficient and I substituted a different one.** Reported
   here in full with the measurement, because a reviewer checking I1 against the finding's text will
   otherwise see a program they did not ask for. `r1` is retained in the file as a comment, so the
   next reader cannot shorten `r2` back into it without meeting the reason.
2. **Every row except `r2` is invariant under the depth/flag choice**, which is worth knowing when
   judging what this file pins: the delivery key's *nesting* half rests on one row. Its `p1`-shaped
   and `p10`-shaped halves each rest on several.

---

# Fix report, round 2

**Commit:** `f6dcfcc91` -- *Bound the flag claims to what was run, and stop a row claiming what it
does not pin*. N1--N5 and both non-gating minors addressed. **No code behaviour changed this round;
every edit is prose, and every one of N1--N4 was deleted rather than restated**, per the round's
method note.

I re-measured each counterexample myself before editing anything, building three release binaries
from the current tree: the committed design, the boolean-flag design, and the design with the
discard removed.

## N1 -- "every shorter shape gives the same bytes either way"

**Confirmed false.** The cited program minus its last clause:

program: `interpret 'zq = raiser(); interpret "say 1; say 2"'` in the usual surround, `rc 0`,
stderr empty everywhere.

| build | stdout |
|---|---|
| oracle | `h1 3` / `1` / `2` / `h2 3` / `after 5` |
| committed, both engines | `h1 3` / `1` / `2` / `h2 3` / `after 5` |
| flag design, both engines | `h1 3` / `1` / `h2 3` / `2` / `after 5` |

So a *shorter* shape does separate the designs. The universal is deleted. What replaces it is only
what the experiment covered: the flag was built and run against this file's rows, and the doc
comment now cites the discriminating program and points at the case file for the neighbouring one
that does not separate them.

## N2 -- "the inner fragment needs more than one clause"

**Confirmed false**, in both places it appeared (`lib.rs` and the case file's last row).

program: `interpret 'zq = raiser(); interpret "interpret ""say 1; say 2"""; say 3'`, `rc 0`, stderr
empty everywhere. Its inner fragment is one clause.

| build | stdout |
|---|---|
| oracle | `h1 3` / `1` / `2` / `h2 3` / `3` / `after 5` |
| committed, both engines | `h1 3` / `1` / `2` / `h2 3` / `3` / `after 5` |
| flag design, both engines | `h1 3` / `1` / `h2 3` / `2` / `3` / `after 5` |

The requirement is not restated in the true form either. I have now got this general rule wrong
twice in two rounds, and the thing it was protecting -- a reader shortening the stanza -- is served
by the measurement alone. What stays is: neighbouring shapes do not all separate the designs, so do
not shorten this program by eye, and here is the one that was measured and does not.

## N3 -- "what this row pins is the discard reaching a nested fragment"

**Confirmed false.** With the discard removed and the delivery key kept:

| program | oracle | no-discard build, both engines |
|---|---|---|
| `p9` `interpret 'interpret "zq = raiser()"'` | `h1 3` / `after 5` | `h1 3` / `after 5` (still right) |
| `p8` `interpret 'zq = raiser()'` | `h1 3` / `after 5` | `h1 3` / `after 5` (still right) |
| `p10` two fragments | `h1 3` / `second` / `after 5` | `h1 3` / `second` / `h2 4` / `after 5` (**wrong**) |

The row's comment now says what is true and bounded: it is green under the delivery-key mutation,
the discard mutation and the flag design alike, so it holds the oracle's answer for a nested
fragment rather than a mechanism the other rows leave uncovered.

**This measurement also refuted a claim in my own round-1 fix report**, which attributed `p8`, `p9`
and `p10` to the discard. Only `p10` is. *Issues and concerns* item 3 above is corrected, with the
transcripts.

## N4 -- the controlled-loop row contradicting the plain-`DO` row

Correct: the `Simple` arm opens the plain `DO`'s own header clause, and the same file's `p1` row
says so. Only the per-pass re-test is a boundary a plain `DO` lacks, and that is all the comment
claims now.

## N5 and the two non-gating minors

* **N5** -- "each row needing a different one" deleted. The bullets stand on their own.
* **the fourth part** -- replaced by "the nesting-level part", removing the ordinal into the bullet
  list, the same fix M6 got for the ordinal into the row list.
* **`run.rs:1707` -> `run.rs:1709`** -- corrected in the round-1 fix report's M7 entry; the
  committed `debug_assert!` is at 1709.

## Tests covering the amended code

| what | command | result |
|---|---|---|
| the case file's harness and the rest of `ir_dual` | `memcap 8G cargo test --release -p rexx-exec --test ir_dual` | `ok. 9 passed; 0 failed` |
| whole workspace, release | `memcap 8G cargo test --release --workspace --no-fail-fast -j 2` | exit 0, no `FAILED` |
| whole workspace, debug (the `debug_assert`) | `memcap 8G cargo test --workspace -j 2` | exit 0, no `FAILED`, assertion never fired |
| corpus gate | `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus` | exit 0, `mode: STRICT (the gate)` |
| fmt | `cargo fmt --all --check` | exit 0 |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |

**The header's flag claim re-verified against the amended file**, since it is a claim about the
current row set: with the flag design applied, `both_engines_agree_on_every_case_file` exits 101
failing at `interpret-condition-queue:250`, which is the `interpret "say 1; say 2"` stanza and the
file's last. `datadriven` stops at the first failure, so that is also the statement that every
earlier row passed under the flag.

## Files changed this round

* `rust/crates/rexx-exec/src/lib.rs` -- N1, N2.
* `rust/crates/rexx-exec/tests/ir_dual_cases/interpret-condition-queue` -- N1, N2, N3, N4, N5, the
  ordinal minor.
* this report -- the `p8`/`p9`/`p10` attribution and the assert's line number.

## Concerns from this round

1. **Every finding this round was in prose I rewrote in the previous round, and none was in code.**
   That is the measured pattern, and it held again. Where a claim survived this round it is because
   it names exactly one experiment and what that experiment ran against; the three general rules I
   had written about the shape of a discriminating program are gone, not repaired.
2. **My own round-1 fix report carried the same defect** -- `p8`/`p9`/`p10` attributed to the
   discard when only `p10` is -- and it took building the no-discard binary to see it. Corrected
   above. The scope claim is now: the delivery key closes `p1`, `p5`, `p8` and `p9`; the discard
   closes `p10` and nothing else measured.
