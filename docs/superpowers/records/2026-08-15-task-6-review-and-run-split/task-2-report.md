# Task 2 report: witness the two `settle_block_indent` arguments

Commit: `dc76f9cdfb28d2f1521bff3acf732d307f3ed82d`
Base: `f6dcfcc91`

## Call sites confirmed

Found by the surrounding `LoopKind::Simple` arm of `run_loop_with_header`
(not by line number; the brief's `run_loop` was wrong, and the brief's line
numbers had moved):

* `crates/rexx-exec/src/run.rs:6222` -- `it.settle_block_indent(true, do_indent);`
  inside the `DO`'s own header clause.
* `crates/rexx-exec/src/run.rs:6291` -- `it.settle_block_indent(false, do_indent);`
  inside the `END` clause.

## What was added, and where

One new case file: `rust/crates/rexx-exec/tests/ir_dual_cases/do-block-handler-indent`,
two stanzas.

Why there. `corpus/lang/do_clause_boundaries.rex` carries no `TRACE` by design
and the controller's resolution 2 forbids adding one. `loop-header-boundaries`
and `interpret-condition-queue` are owned by other tasks. A `trace_oracle` case
would have required registering the witness in that file's `WITNESS_PREFIXES`
and `PREFIX_COVERAGE` tables for prefixes it does not newly reach. An
`ir_dual_cases` stanza records the tree-walker's exact stdout/stderr/rc *and*
asserts both engines agree, which is exactly what this witness needs, and the
directory is walked by `datadriven::walk` so no registration is involved.

## Transcripts measured

Program A (`wa.rex`, the header-clause witness) and program B (`wb.rex`, the
`END`-clause witness) are the two stanza programs verbatim.

### Unmutated build, program A

Oracle, `( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx wa.rex )`,
rc 0.

stdout:

```
h ran 4
g ran 5
body
after 1
```

stderr:

```
     2 *-* call on user zx name h
     3 *-* call on user zy name g
     4 *-* zn = ra()
    10 *-*   ra:
    11 *-*   raise user zx return 1
       >K>     "RESULT" => "1"
       >>>   "1"
    12 *-*   h:
    13 *-*   say 'h ran' sigl
       >>>     "h ran 4"
    14 *-*   raise user zy return 1
       >K>     "RESULT" => "1"
     5 *-* do
    15 *-*     g:
    16 *-*     say 'g ran' sigl
       >>>       "g ran 5"
    17 *-*     return
     6 *-*   say 'body'
       >>>     "body"
     7 *-* end
     8 *-* say 'after' zn
       >>>   "after 1"
     9 *-* exit 0
       >>>   "0"
```

`REXX_ENGINE=tree-walker` and `REXX_ENGINE=ir` (release build at `f6dcfcc91`):
identical to the oracle on stdout, stderr and rc. `diff` reported SAME on all
six comparisons.

### Unmutated build, program B

Oracle, rc 0.

stdout:

```
h ran 5
g ran 6
after 1
```

stderr:

```
     2 *-* call on user zx name h
     3 *-* call on user zy name g
     4 *-* do
     5 *-*   zn = ra()
     9 *-*     ra:
    10 *-*     raise user zx return 1
       >K>       "RESULT" => "1"
       >>>     "1"
    11 *-*     h:
    12 *-*     say 'h ran' sigl
       >>>       "h ran 5"
    13 *-*     raise user zy return 1
       >K>       "RESULT" => "1"
     6 *-* end
    14 *-*   g:
    15 *-*   say 'g ran' sigl
       >>>     "g ran 6"
    16 *-*   return
     7 *-* say 'after' zn
       >>>   "after 1"
     8 *-* exit 0
       >>>   "0"
```

Both engines identical to the oracle on all three descriptors.

### Flip A build (`true` -> `false` at run.rs:6222)

Rebuilt `--release --bin rexx-run`. Program A diverges from the oracle on
stderr on **both** engines, identically; stdout and rc unchanged:

```
oracle                         flip A (both engines)
    15 *-*     g:                  15 *-*   g:
    16 *-*     say 'g ran' sigl    16 *-*   say 'g ran' sigl
       >>>       "g ran 5"            >>>     "g ran 5"
    17 *-*     return              17 *-*   return
```

Program B under flip A: `diff` SAME against the oracle on stdout and stderr,
rc 0, on both engines.

So the brief's recorded two-column loss survived Task 1's change, on the
handler delivered at the `DO`'s header clause.

### Flip B build (`false` -> `true` at run.rs:6291)

Program B diverges from the oracle on stderr on **both** engines, identically;
stdout and rc unchanged:

```
oracle                         flip B (both engines)
    14 *-*   g:                    14 *-*     g:
    15 *-*   say 'g ran' sigl      15 *-*     say 'g ran' sigl
       >>>     "g ran 6"              >>>       "g ran 6"
    16 *-*   return                16 *-*     return
```

Program A under flip B: `diff` SAME against the oracle on stdout and stderr,
rc 0, on both engines.

## Mutation evidence, per argument

Baseline, unmutated, `memcap 8G cargo test --release --workspace --no-fail-fast`:
status 0, 1509 `test ... ok` lines, no `FAILED`.

### Flip A

`memcap 8G cargo test --release --workspace --no-fail-fast` -- status 101,
1508 ok. Every test that caught it, complete:

* `both_engines_agree_on_every_case_file` (`tests/ir_dual.rs`)

Nothing else. The datadriven failure names
`tests/ir_dual_cases/do-block-handler-indent:36`, which is the first stanza
(the header-clause row).

Per-row selectivity, run separately because datadriven stops at the first
failing stanza:

* file replaced by the **second stanza only**:
  `memcap 8G cargo test --release -p rexx-exec --test ir_dual both_engines_agree_on_every_case_file --no-fail-fast`
  -> `ok. 1 passed; 0 failed`.
* file replaced by the **first stanza only**: same command -> `FAILED. 0 passed; 1 failed`.

Coverage-versus-can-fail check: with the whole new file **deleted**,
`memcap 8G cargo test --release --workspace --no-fail-fast` under flip A ->
status 0, 1509 ok, no `FAILED`. So this file is the only catcher in the
workspace.

Corpus gate under flip A:
`REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus --no-fail-fast`
-> status 0, `mode: STRICT`, `55 of 55 matching`. The oracle differential does
not see this defect; `tests/support/mod.rs` normalises trace indent width, and
`support::tests::two_clause_lines_differing_only_in_indent_width_normalise_equal`
asserts that normalisation.

### Flip B

`memcap 8G cargo test --release --workspace --no-fail-fast` -- status 101,
1508 ok. Every test that caught it, complete:

* `both_engines_agree_on_every_case_file` (`tests/ir_dual.rs`)

Nothing else. The failure names
`tests/ir_dual_cases/do-block-handler-indent:89`, which is the second stanza
(the `END`-clause row).

Per-row selectivity:

* file replaced by the **first stanza only** -> `ok. 1 passed; 0 failed`.
* file replaced by the **second stanza only** -> `FAILED. 0 passed; 1 failed`.

Coverage check: whole file deleted, flip B, full workspace `--no-fail-fast` ->
status 0, 1509 ok, no `FAILED`.

Corpus gate under flip B: status 0, 10 passed, 0 failed.

Every mutation run used a byte copy of `run.rs` for restore, never
`git checkout --`, and every run count asserted above is non-zero.

## What these witnesses do NOT distinguish

Stated in the file's own header as well:

* **Which argument carries the level.** Only the settled
  `current_value_indent` reaches the handler, so
  `settle_block_indent(false, do_indent + 2)` at the header site would produce
  byte-identical output. These rows pin the resulting indent, not the calling
  convention that produces it.
* **Anything about repeating loops.** Neither program contains one, so neither
  row says anything about the three `settle_block_indent` calls outside the
  `LoopKind::Simple` arm (`run.rs:5603`, `:6596`, `:6706`). Those were not
  mutated and are not covered here.
* **Whether the two boundaries exist at all.** I did not measure removing
  either clause, so I make no claim about whether these rows would catch that.
  What they were measured against is the `open` argument only.
* **The oracle.** These are committed bytes, not a live oracle run. They were
  captured from the oracle before being written down and re-verified after,
  but the test itself does not re-run the oracle. That is `corpus.rs`'s job,
  and `corpus.rs` is blind to this particular defect, which is the gap this
  file fills.

## Files changed

* `rust/crates/rexx-exec/tests/ir_dual_cases/do-block-handler-indent` (new, only file)

`crates/rexx-exec/src/run.rs` was mutated four times during measurement and
restored byte-for-byte from a copy each time; `diff` against the pre-mutation
copy is empty, and the commit contains only the new file.

## Gates

Each run unpiped, exit status read on its own:

* `cargo fmt --all --check` -> 0
* `memcap 8G cargo clippy --workspace --all-targets -- -D warnings` -> 0
* `memcap 8G cargo test --release --workspace` -> 0, 1509 ok, no `FAILED`
* `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus -- --nocapture`
  -> 0, `mode: STRICT`, `55 of 55 matching`

## Self-review findings, fixed before commit

1. The header named the enclosing function `run_loop`. The `LoopKind::Simple`
   arm is in `run_loop_with_header`. Corrected against the tree. The brief
   carries the same wrong name; it is stated correctly in the file and here.
2. "each row is only sensitive to its own [boundary]" claimed more than the
   two flips I ran. Reworded to state the construction ("each row is built so
   that only its own boundary delivers anything") and let the measurement
   sentence carry the result.
3. "The settled indent is all that is observable" was a general observability
   claim. Replaced with the concrete equivalent call that would produce the
   same bytes.
4. An enumeration `DO i = 1 TO n`, `DO WHILE`, `DO FOREVER` read as an
   exhaustive list of repeating kinds and is not one. Replaced with
   "a repeating `DO`".
5. No em-dashes: `/bin/grep -n "—"` on the file returns nothing.

## Concerns

* **`clippy` ran warm.** The commit touches no Rust source, so there is
  nothing new for the linter to see, but CLAUDE.md's rule that a same-session
  green is provisional still applies. I did not run it from a clean target
  directory.
* **The two rows share one test name.** `both_engines_agree_on_every_case_file`
  covers every case file, and datadriven panics at the first failing stanza, so
  a single run cannot show which of the two rows moved when both do. The
  per-row selectivity above was therefore measured by holding each stanza out
  in turn rather than read off one run. A future defect that moves both rows
  will report only the first.
* **No divergence found and left unfixed.** Both engines match the oracle on
  both programs at the committed tree.
* **Untracked file not mine:** `docs/superpowers/specs/2026-08-15-phase-5-inherited-surface.md`
  was already untracked in the working tree when I started and I did not touch
  or stage it.

---

# Fix round 1

Commit: `64d0ff54fc0c4d1e58cc5f72f98c023c2c838cc6`
Files changed: `rust/crates/rexx-exec/tests/ir_dual_cases/do-block-handler-indent` only.
`crates/rexx-exec/src/run.rs` was mutated ten more times and restored from a
byte copy each time; `diff` against the pre-mutation copy is empty and the
commit contains only the case file.

Every line number, per-row result and hold-out result below was re-measured
against the **committed** bytes, after the last comment edit, so nothing here is
carried over from a run against an earlier revision of the file.

## I1 -- the header row's note was one level off

Confirmed against the recorded bytes. In that row `do` echoes at column 0, the
body's `say 'body'` at 2, and `g:` at 4. The note said `g:` echoes "two columns
in from the `DO`", which is the body's column, not the handler's.

Cause, and it is the same one as I2: the settled block indent is not the column
the handler's clauses echo at. The header boundary settles 2 and the handler
activation echoes a level in from what it inherits, so `g:` lands at 4.

Fixed by stating both numbers: "that boundary settles 2 and `g:` echoes at
column 4, ahead of the body's `say` at column 2". Every number in it is a count
of the spaces between `*-*` and the clause text in the block below, and the file
header now says so explicitly and states the settled-vs-echoed relation once so
neither row note has to.

## I2 -- the `END` row's note stated the opposite of its bytes

Confirmed. In that row `do` echoes at 0, the body clause `zn = ra()` at 2, and
`g:` at 2. The note said `g:` echoes "at the `DO`'s own level rather than the
body's"; it echoes at exactly the body clause's column.

Fixed the same way: "that boundary settles 0 -- the `DO`'s own -- and `g:`
echoes at column 2, the body clause's own column." The settled value and the
echoed column are now both named and neither is left for the reader to derive.

## I3 -- the nested row, and what it buys

Added a third stanza whose plain `DO` sits inside another block, so `do_indent`
is 2 rather than 0. Measured against the live oracle from a fresh empty
directory, rc 0, three descriptors read separately.

Program:

```
trace r
call on user zx name h
call on user zy name g
call on user zw name k
call on user zv name m
do
zn = ra()
do
zp = rb()
end
end
say 'after' zn zp
exit 0
ra:
raise user zx return 1
rb:
raise user zw return 1
h:
say 'h ran' sigl
raise user zy return 1
g:
say 'g ran' sigl
return
k:
say 'k ran' sigl
raise user zv return 1
m:
say 'm ran' sigl
return
```

Oracle stdout:

```
h ran 7
g ran 8
k ran 9
m ran 10
after 1 1
```

Oracle stderr (the four lines the row exists for, in context):

```
     8 *-*   do
    21 *-*       g:
...
    10 *-*   end
    27 *-*     m:
```

The inner `do` and `end` echo at column 2, `g:` at 6 (settled 4), `m:` at 4
(settled 2). `REXX_ENGINE=tree-walker` and `REXX_ENGINE=ir` are byte-identical
to the oracle on stdout, stderr and rc.

**Nothing here contradicts the two existing rows.** Both rows and the nested one
fit the same rule: the header boundary settles `do_indent + 2`, the `END`
boundary settles `do_indent`, and a handler echoes two columns deeper than the
settled value. The top-level rows are that rule at `do_indent == 0`.

## Mutation evidence, all five mutants, against the committed file

Baseline: `memcap 8G cargo test --release --workspace` -> status 0, 1509 ok.

Full workspace, `memcap 8G cargo test --release --workspace --no-fail-fast`, one
build per mutant. "caught by" is the complete list of failing tests, not the
first:

| mutation | status | ok | caught by | stanza datadriven named |
|---|---|---|---|---|
| `settle_block_indent(false, do_indent)` at the header site | 101 | 1508 | `both_engines_agree_on_every_case_file` | `do-block-handler-indent:57` |
| `settle_block_indent(true, do_indent)` at the `END` site | 101 | 1508 | `both_engines_agree_on_every_case_file` | `do-block-handler-indent:111` |
| `settle_block_indent(true, 0)` at the header site | 101 | 1508 | `both_engines_agree_on_every_case_file` | `do-block-handler-indent:167` |
| `settle_block_indent(false, 0)` at the `END` site | 101 | 1508 | `both_engines_agree_on_every_case_file` | `do-block-handler-indent:167` |
| `settle_block_indent(false, end_indent)` at the `END` site | 0 | 1509 | nothing | -- |

Lines 57, 111 and 167 are the three `program` directives in the committed file,
in order: header row, `END` row, nested row.

Per-row selectivity, each row placed in the directory alone,
`memcap 8G cargo test --release -p rexx-exec --test ir_dual both_engines_agree_on_every_case_file --no-fail-fast`.
Every cell is a run with a non-zero count (`1 passed` or `1 failed`), never
`0 passed; 0 failed`:

| mutation | header row alone | `END` row alone | nested row alone |
|---|---|---|---|
| header `open` flipped | FAILED | ok | FAILED |
| `END` `open` flipped | ok | FAILED | FAILED |
| header `do_indent` -> `0` | ok | ok | FAILED |
| `END` `do_indent` -> `0` | ok | ok | FAILED |
| `END` `do_indent` -> `end_indent` | ok | ok | ok |

So the per-argument selectivity from the first round is intact: no row reddens
for the argument it is not about. The nested row reddens for either `open` flip,
which is why the file says it is not selective between them.

Direction of the literal-`0` divergence, read out of the failure blocks:

```
expected   21 *-*       g:      (column 6)     27 *-*     m:      (column 4)
zeroA      21 *-*     g:        (column 4)     27 *-*     m:      (unchanged)
zeroB      21 *-*       g:      (unchanged)    27 *-*   m:        (column 2)
```

Each literal `0` moves only its own site's handler, two columns out. That is
what the nested row's note now says.

"Can fail" versus "adds coverage", full workspace `--no-fail-fast` per row:

* header `open` flipped, **whole file** deleted: status 0, 1509 ok, nothing failed.
* `END` `open` flipped, **whole file** deleted: status 0, 1509 ok, nothing failed.
* header `do_indent` -> `0`, **nested row only** held out: status 0, 1509 ok, nothing failed.
* `END` `do_indent` -> `0`, **nested row only** held out: status 0, 1509 ok, nothing failed.

The last two are the ruling's premise measured directly: with the two top-level
rows in the tree and the nested row absent, both literal-`0` mutants pass the
entire workspace. The nested row is the only catcher.

Corpus gate under each of the five mutants,
`REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus --no-fail-fast`:
status 0 every time, `10 passed; 0 failed`. The oracle differential sees none of
this, which the file now states as the reason the witness lives here.

## M4, M5, M6

* **M4.** "ir_dual.rs refuses to run at all under REWRITE" replaced by
  "`both_engines_agree_on_every_case_file` refuses to run under REWRITE for that
  reason". The assert is inside that test.
* **M5.** The recipe now shows the redirect (`>out 2>err`, never merged) and
  says to write the block as `rc> <exit status>`, then every line of `out`
  tagged `out>`, then every line of `err` tagged `err>`.
* **M6.** Correct. The first round's mutation runs were made against the file as
  it stood before three self-review comment edits, where the `program`
  directives sat at 36 and 89; the committed file put them at 41 and 94, and I
  wrote down what the runs printed without re-checking it against what I
  committed. To close the class rather than the instance, every mutation in this
  round was re-run against the committed bytes after the last comment edit, and
  `datadriven`'s reported lines (57, 111, 167) are the `program` directives of
  the committed file, checked with `grep -n '^program$'`.

## What the rows still do not distinguish, restated for the third row

* **`do_indent` from `end_indent` at the `END` site.** Measured: passing the
  `end_indent` computed two lines above for the `END`'s own echo leaves all
  three rows and the whole workspace byte-identical. This is now disclosed in
  the file. Both are the `DO`'s own indent, computed by different routes; I did
  not find a program that separates them and make no claim that none exists.
* **Which argument carries the level.** `settle_block_indent(false, do_indent + 2)`
  at the header site produces the same bytes.
* **Repeating loops.** No row contains one, so nothing here covers the
  `settle_block_indent` calls outside the `LoopKind::Simple` arm.
* **Whether either boundary exists at all.** Not measured, and not claimed.
* **Nesting deeper than one level, and a `DO` nested in something other than a
  plain block.** The nested row uses one enclosing plain `DO`. `do_indent` at 2
  is enough to tell it from the constant 0, which is what the ruling asked for;
  it is not evidence about 4, or about a `DO` inside a `SELECT` or an `IF`.

## Gates, re-run after every mutation was restored

Each unpiped, status read on its own:

* `cargo fmt --all --check` -> 0
* `memcap 8G cargo clippy --workspace --all-targets -- -D warnings` -> 0
* `memcap 8G cargo test --release --workspace` -> 0, 1509 ok, no `FAILED`
* `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus -- --nocapture`
  -> 0, `mode: STRICT`, `55 of 55 matching`

`/bin/grep -n "—"` on the case file returns nothing.

## Self-review of this round's own prose

Three sentences were rewritten again after the mutations came back, because the
first draft of each was broader than the run:

1. "a site passing a literal `0` for `do_indent` puts both of those handlers two
   columns out" -- I mutated the two sites separately, and each moves only its
   own handler. Now: "replacing `do_indent` with a literal `0` at either site
   moves that site's own handler two columns out -- `g:` to 4, or `m:` to 2".
2. "`corpus_differential` in STRICT passes under every mutation named above" was
   a universal over a list living in the same file. Now names the closed set it
   was measured on: either `open` flip, and a literal `0` at either site.
3. "The nested row separates the arguments from the constant they happen to
   equal at top level" -- the arguments do not equal a constant, `do_indent`
   does. Now says `do_indent`.

## Concerns

* **`clippy` ran warm** in this round too. The commit changes no Rust source, so
  there is nothing new to lint, but the same-session-green rule still applies.
* **The nested row is deliberately not selective between the two arguments.**
  If both `open` arguments regress at once, `datadriven` stops at the first
  failing stanza and the operator sees the header row only. The per-row
  hold-out method above is what separates them, and it is not something a
  single run does for you.
* **The `end_indent` residue is disclosed, not closed.** I did not construct a
  program that tells the two computations apart, and I do not assert that none
  exists.

---

# Fix round 2

Commit: `d5bc4b8a01c659e1c1c001da7bdba09f9c18094f`
Files changed: `rust/crates/rexx-exec/tests/ir_dual_cases/do-block-handler-indent` only.
`crates/rexx-exec/src/run.rs` was mutated five more times and restored from a
byte copy each time; `diff` against the pre-mutation copy is empty.

## N1 -- the instruction for reading the columns was off by one

Reproduced. Measured the space run between `*-*` and the clause text on every
clause line in the file:

```
origin space-run for a column-0 clause: 1
```

Every clause line is one space plus its indent, so the counting rule the
sentence gave was one more than the columns the notes use, in every case. The
reviewer's five examples all reproduce, and the `end` case is the clearest: an
unindented clause has a one-space run, so the rule would have called it
"column 1".

Fixed by replacing the counting rule with an origin, so there is no arithmetic
left to be off by one:

> Column 0 is where the `call on` clauses at the top of each row echo, outside
> every block, and the columns each row's note names are read against those.

The `call on` clauses are present in all three rows and sit outside every block
in all three. **The notes were not renumbered.** Checked mechanically: taking a
`call on` line's space run as the origin, every column the notes name comes out
exactly as written --

```
row 1   do 0    g: 4    say 'body' 2   end 0
row 2   do 0    zn = ra() 2            g: 2    end 0
row 3   do 0    inner do 2    g: 6     inner end 2    m: 4    outer end 0
```

Row 2's note says `g:` echoes "at column 2, the body clause's own column", and
the body clause `zn = ra()` is at 2. Row 3's note says the inner `DO` echoes at
2, `g:` at 6, `m:` at 4.

I considered deleting the sentence, as the method note suggested. I kept it in
the anchored form for one reason: the notes name absolute columns, and without
an origin "column 4" is a bare number a reader has to guess the base of. The
re-review named this exact alternative ("define the column as the distance from
where an unindented clause echoes") as an acceptable fix, and it is a
comparison rather than a count, so it has no off-by-one to get wrong.

## N2 -- the recipe modelled a relative redirect

Fixed. The recipe now says to run from a fresh empty directory and use an
absolute path for the program and for every redirect:

```
  ( ulimit -v 1048576; LD_LIBRARY_PATH=<ooRexx>/build/lib \
    <ooRexx>/build/bin/rexx <absdir>/PROGRAM.rex </dev/null ) \
    > <absdir>/out 2> <absdir>/err
```

Rather than assert the recipe works, I ran it. Extracted the nested stanza's
program verbatim from the committed file into a fresh empty directory, followed
the recipe literally with the placeholders substituted, rendered the result as
`rc>` / `out>` / `err>` in that order, and diffed against the stanza's recorded
expected block:

```
recipe output matches the recorded block: True
```

So the instruction someone will follow does produce the bytes this file records.

## Re-run after the amendment

The change is comment-only, but comments in this file are part of what
`datadriven` parses and they shift every stanza's line number, which is the
exact class of staleness M6 was. So the whole matrix was re-run against the
amended bytes rather than reasoned about.

Full workspace, `memcap 8G cargo test --release --workspace --no-fail-fast`,
one build per mutant:

| mutation | status | ok | caught by | stanza |
|---|---|---|---|---|
| header site `open` -> `false` | 101 | 1508 | `both_engines_agree_on_every_case_file` | `:60` (header row) |
| `END` site `open` -> `true` | 101 | 1508 | `both_engines_agree_on_every_case_file` | `:114` (`END` row) |
| header site `do_indent` -> `0` | 101 | 1508 | `both_engines_agree_on_every_case_file` | `:170` (nested row) |
| `END` site `do_indent` -> `0` | 101 | 1508 | `both_engines_agree_on_every_case_file` | `:170` (nested row) |
| `END` site `do_indent` -> `end_indent` | 0 | 1509 | nothing | -- |

Per-row, each row alone in the directory,
`memcap 8G cargo test --release -p rexx-exec --test ir_dual both_engines_agree_on_every_case_file --no-fail-fast`,
every cell a run with a non-zero count:

| mutation | header row | `END` row | nested row |
|---|---|---|---|
| header site `open` -> `false` | FAILED | ok | FAILED |
| `END` site `open` -> `true` | ok | FAILED | FAILED |
| header site `do_indent` -> `0` | ok | ok | FAILED |
| `END` site `do_indent` -> `0` | ok | ok | FAILED |
| `END` site `do_indent` -> `end_indent` | ok | ok | ok |

Identical to round 1's matrix in all fifteen cells. 60, 114 and 170 are the
three `program` directives of the committed file, confirmed with
`grep -n '^program$'` after the commit; the previous round's 57, 111 and 167
were correct for the previous commit and moved because the recipe block grew by
three lines.

## Gates

Each unpiped, status read on its own, after every mutation was restored:

* `cargo fmt --all --check` -> 0
* `memcap 8G cargo clippy --workspace --all-targets -- -D warnings` -> 0
* `memcap 8G cargo test --release --workspace` -> 0, 1509 ok, no `FAILED`
* `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus -- --nocapture`
  -> 0, `mode: STRICT`, `55 of 55 matching`

`/bin/grep -n "—"` on the case file returns nothing.

## Concerns

* **`clippy` ran warm again.** No Rust source changed in this round either.
* **Stanza line numbers have now moved twice** and were reported wrong once.
  They are a property of the file, not of the behaviour, and the only reason
  they appear in these reports is so a reviewer can locate what `datadriven`
  named. Anyone re-running should take them from `grep -n '^program$'` at the
  revision they are on rather than from this report.
* **The `end_indent` residue is still open**, unchanged from round 1: nothing
  here separates `do_indent` from `end_indent` at the `END` site, and I do not
  claim nothing could.
* **The two out-of-scope observations were not acted on**, per the message: the
  `corpus_differential` sentence stands, and the nested stanza remains the
  directory's largest.
