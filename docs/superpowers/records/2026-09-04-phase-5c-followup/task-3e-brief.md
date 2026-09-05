# 5c follow-up Task 3e — the conversions, and the commit that flips `say buf`

You are the implementer for the **fifth and last Task 3 commit** of
`docs/superpowers/plans/2026-09-04-phase-5c-followup.md`: the conversions. Read the plan's head,
Global constraints, Task 2 and Task 3; then, in
`docs/superpowers/records/2026-09-04-phase-5c-followup/`: `task-1-report.md` §4.1 (the method-body
discipline); `task-2-report.md` §1 and §2.3; `task-3b-report.md` §1, §2.3 and §7; `task-3c-report.md`
§2.4 (capacity growth is not one rule) and §3.1; `task-3d-report.md` §3 (the mutation instrument and
the M1b control shape) and its open questions. `rust/CLAUDE.md` governs; read its Gates and Method
sections. **BASE is `git log -1` when you start.** You work in the worktree
`/home/moritz/dev/repos/ooRexx-rust-rewrite`, in `rust/`; it is yours until the hand-back.

## What you bind

Four rows, at the counts `interpreter/memory/Setup.cpp:1416`-`:1477` declares:

| name (count) | C++ | answers |
|---|---|---|
| `MAKESTRING` (0) | `MutableBuffer::makeStringRexx` | a `String` of the contents |
| `MAKEARRAY` (1) | `MutableBuffer::makeArrayRexx` | an `Array`, the contents split on line ends |
| `SUBWORDS` (2) | `MutableBuffer::subWords` | an **`Array`** -- see below |
| `SETTEXT` (1) | `MutableBuffer::setTextRexx` | the receiver, contents replaced |

`STRING` is already bound (Task 2) and stays as it is. **`SETTEXT` is in no family list in the
plan** -- it is a mutator that Task 3c's list did not name and no later task claimed. It is here
because it is the last unbound instance row, and the report should note the plan gap rather than
leave the next reader to wonder which task was supposed to have taken it.

## Three measurements taken before you were dispatched

Each on the oracle at `cb49df49d`, from a fresh directory. They are the reason this task is not the
shape the plan's one-line sketch suggests.

**1. `subWords` answers an Array, not a string.** Over `.MutableBuffer~new('a b  c')`:

```
b2~subWords(2)~class          -- The Array class
'[' || b2~subWords(2) || ']'  -- [b\nc]   (two lines: the Array's own rendering)
'[' || b2~subWords(2,1) || ']' -- [b]
```

A body written as "the word range as text" is wrong. `word_slices` (`builtin/word.rs:167`) is the
core to build from, and `native_string_makearray` (`dispatch.rs:9665`) is the model for handing back
an `Array` -- but note that String's `MAKEARRAY` is count 0 and takes no argument where
MutableBuffer's is count 1, so the separator argument is yours to measure and implement, not to
inherit.

**2. `makeArray` splits on line ends.** `.MutableBuffer~new('a b  c')~makeArray~items` is 1 and its
first element is the whole `a b  c`; `.MutableBuffer~new('one'||'0a'x||'two')~makeArray~items` is 2.
Measure what the count-1 argument does before you write it.

**3. The receiver-side comparison stays an identity compare, for `=` as well as `==`.** The plan
records `buf == 'abc'` as `0`; measured now, **`buf = 'abc'` is also `0`**, while `'abc' == buf` is
`1` and `length(buf)` is `3`. So binding `MAKESTRING` must not make either comparison request the
buffer's string when the buffer is the left operand. The plan names only `==`; both belong in your
witness, and if the crate today answers `1` for either after `MAKESTRING` binds, that is the
regression this task exists to avoid.

## What flipping `MAKESTRING` does to everything else

`say buf` routes through `MAKESTRING`, which is why every witness so far has been forbidden from
containing it. **This commit is where that ends**, and it is also the commit that closes
`corpus/method-bodies.txt` for this class. The rows that are `loud` today, with their evidence:

* at `MAKESTRING` -- the class row `new`, and `delete`, `delStr`, `lower`, `space`, `translate`,
  `upper`: their sends already succeed and only the probe's own `say` refuses.
* at their own name -- `makeArray`, `makeString`, `setText`, `subWords`, the four you bind.

**Predict, before the refresh, what each of those eleven rows does**, and say which reach `answers`
and which do not. **The gate: no row moves to `diverge`**, and the file's `diverge` total (7) does
not change. If the prediction is that every one of the eleven becomes `answers`, say so and let the
refresh judge it -- Task 3d's brief carried a `loud`-at-`MAKESTRING` expectation for
`caselessChangeStr` that the oracle falsified, so do not reason from a sibling's row.

## Method bodies and refusals

Task 1 §4.1's discipline: convert every argument first, then `buffer_state` -- `buffer_state_mut`
for `SETTEXT` only -- and the core over the bytes. A `None` from `buffer_state` is the
`Loud::native_method` refusal, never a panic. Task 3b §7 is live: read each method's C++ conversion
order rather than assuming the argument list's. **Measure every refusal on the oracle first** (a
two-line program per case, fresh directory, three descriptors), as Task 3b's §2.3 and Task 3d's
tables do. The argument helpers already exist; add one only where the C++ refuses in a shape none of
them covers, and say which C++ call it mirrors. `SETTEXT` can grow the buffer, so Task 3c §2.4
applies to it: the growth is observable through `getBufferSize`, and the witness must exercise it
past the current capacity.

## The witness: `rust/corpus/lang/mutablebuffer_conversion.rex`

**Written and confirmed on the oracle before you touch the crate** -- rc 0, stderr empty, both
engines byte-identical after. It must contain, at least: `say buf` itself, now that it works; the
four comparison lines from measurement 3, in both operand orders; `makeString` against `string`;
`makeArray` on contents with and without line ends, and its count-1 argument; `subWords` with and
without its second argument, and its `~class`; `setText` growing the buffer past `getBufferSize`;
and a buffer that is empty. `buf`, never `b`.

**File it in this commit**: `corpus/phase-5c.txt` and `coverage.rs`'s `EXPECTED_SUBSET_5C`, **not**
`corpus/unfiled.txt`; plus the `rexx-parse/tests/sourceline_oracle/mutablebuffer_conversion.txt`
companion, generated with the driver in that test's module comment. The pin's inversion -- the line
absent from `EXPECTED_SUBSET_5C` -- must redden `phase_5c_subset_matches_the_committed_list`.

Task 3c's lesson holds: **a line that reaches a path is not a line that observes it.** Write down,
per method, which printed value would change if that method were wrong, and check each claim against
the witness text you commit rather than against your exploration program.

## Controls, each predicted before it runs

* The witness against BASE's binary (a pristine `git archive <BASE> rust interpreter` extract under
  `/home/moritz/dev/repos/claude-build-scratch/5c-followup-task3e/`, own `CARGO_TARGET_DIR`; the
  archive needs `interpreter/` as well as `rust/`): rc 120 at the first unbound send, stderr naming
  it.
* **The corpus control that can go red is `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec
  --test corpus`**, read with its `N of M matching` line, never the exit status alone.
* **The without-witness control uses Task 3d's M1b shape**: take the line out of `phase-5c.txt`
  **and name it in `corpus/unfiled.txt`**, so `every_lang_program_is_run_or_named_unfiled`
  (`corpus.rs:845`) is satisfied and the run's exit status and its matching line say the same thing.
  Merely deleting the line exits 101 on bookkeeping and reads as a contradiction.
* One mutation per method at least, plus one that defeats the identity compare in measurement 3,
  each predicted -- which test or program catches it, and **which line of the witness output moves**
  -- run with `--profile mutation`. **The per-mutant sha must be the sha256 of the corpus test
  executable read after the corpus run**, never `target/mutation/rexx-run` after `--lib`: that
  binary is not what the differential runs (`corpus.rs`'s `run_rust` is in process) and `--lib` does
  not rebuild it. Show two mutants give two shas before quoting any of them. Restore from a copy,
  `touch`, and confirm the release binary's sha256 is unchanged.
* Run the suite without the new witness wherever the new witness is the claimed catcher.
* `cargo test <name>` matches nothing at exit 0 -- assert run counts.

## Rules that bite

No `unsafe`. Never `git checkout -- <path>` on an edited file. No `rm` with a glob or computed path;
delete nothing you did not create, your own scratch directory included. Never `git add -A`; never
amend; `git commit -F <file>` naming paths; `Cargo.lock` not staged. Oracle runs from a fresh empty
directory under the wrapper in the plan; three descriptors, never `2>&1`; both engines
`REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`; never run anything in
`rust/corpus/oracle-crashes.txt`; never `NUMERIC DIGITS` above 1000. `cargo fmt --all --check` (not
`--edition`). `grep` is `ugrep -I`; use `/bin/grep -a` for counts and `-F` for data patterns.
Comments minimal: one sentence per body naming the C++ function; measurements that state a property
may stay; no history, no set sizes. Scratch under `…/scratchpad/task3e/`, builds under
`claude-build-scratch/5c-followup-task3e/`.

## Report and hand-back — commit-before-gating, no exceptions

Report `docs/superpowers/records/2026-09-04-phase-5c-followup/task-3e-report.md`: skeleton first;
every claim **measured** (with the command or file) or **inferred**; the witness's text and the
oracle's three descriptors; every refusal measured, in a table like Task 3d's; what the count-1
`makeArray` argument and the count-2 `subWords` arguments do; the predicted and measured
`method-bodies.txt` moves for all eleven rows; each control's prediction and reading; the plan gap
that left `SETTEXT` in no family; files created outside the tree; open questions. Fast checks
yourself -- `fmt`, `clippy`, `--lib`, STRICT corpus, `--test refusal_sites` (constructor definitions
are cited by line; re-derive moved rows from the test's own panic by script, never by hand), `--test
method_bodies`, `--test coverage`, `cargo test --release -p rexx-parse --test sourceline_oracle`,
run counts asserted, **in the foreground with a bounded timeout** where possible. **Commit code and
report together** with the Gates table carrying `**G1**`–`**G7**` (copy from `task-3d-report.md`),
start the seven gates in the background from `rust/` writing `…/scratchpad/task3e/gates/status.txt`
(commit sha its first line, pidfile beside it, `finished` its last), final message `committed at
<sha>, gates running, statuses at <path>`, stop. **If you background any job and wait on it, send
the controller its pidfile and status-file paths before you go idle** -- and if that job mutates the
tree, say where the pristine copies are, as Task 3d did.
