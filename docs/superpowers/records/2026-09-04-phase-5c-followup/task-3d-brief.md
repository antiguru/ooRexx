# 5c follow-up Task 3d — the caseless family

You are the implementer for the **fourth Task 3 commit** of
`docs/superpowers/plans/2026-09-04-phase-5c-followup.md`: the caseless family. Read the plan's head,
Global constraints, Task 2 and Task 3; then, in
`docs/superpowers/records/2026-09-04-phase-5c-followup/`: `task-1-report.md` §4.1 (the method-body
discipline), §4.2 and §5; `task-2-report.md` §1 and §2.3; `task-3a-report.md` §1.1 and §2 (the cores
and their signatures: a byte `start` is 0-based, a word `position` 1-based, an omitted length is
`Option<usize>`); `task-3b-report.md` §1 (the argument helpers), §2.3 (the measured refusal table
you extend) and §7 (argument conversion order is part of the answer); `task-3c-report.md` §2.4
(capacity growth is not one rule), §3.1 (what a falsified mutation prediction cost) and §5.
`rust/CLAUDE.md` governs; read its Gates and Method sections. **BASE is `git log -1` when you
start.** You work in the worktree `/home/moritz/dev/repos/ooRexx-rust-rewrite`, in `rust/`; it is
yours until the hand-back.

## What you bind

The eleven caseless rows of `MutableBuffer`, at the counts
`interpreter/memory/Setup.cpp:1416`-`:1477` declares -- every caseless `AddMethod` in that block,
and it holds no other:

| name (count) | C++ (`classes/MutableBufferClass.cpp`) | its case-sensitive twin |
|---|---|---|
| `CASELESSPOS` (3) | `caselessPos` | `POS` -- **but see the overrun below** |
| `CASELESSLASTPOS` (3) | `caselessLastPos` | `LASTPOS` |
| `CASELESSCOUNTSTR` (1) | `caselessCountStrRexx` | `COUNTSTR` |
| `CASELESSCONTAINS` (3) | `caselessContains` | `CONTAINS` |
| `CASELESSCONTAINSWORD` (2) | `caselessContainsWord` | `CONTAINSWORD` |
| `CASELESSWORDPOS` (2) | `caselessWordPos` | `WORDPOS` |
| `CASELESSMATCH` (4) | `caselessMatch` | `MATCH` |
| `CASELESSMATCHCHAR` (2) | `caselessMatchChar` | `MATCHCHAR` |
| `CASELESSSTARTSWITH` (1) | `caselessStartsWithRexx` | `STARTSWITH` |
| `CASELESSENDSWITH` (1) | `caselessEndsWithRexx` | `ENDSWITH` |
| `CASELESSCHANGESTR` (3) | `caselessChangeStr` | `CHANGESTR` |

Ten answer a value; **`CASELESSCHANGESTR` mutates and answers the receiver**, so it is the one row
in this family that can grow the buffer, and Task 3c §2.4's rule applies to it: the growth is
observable through `getBufferSize`, and the witness must exercise it past the current capacity.

**Not this commit**: the conversions (`makeString`, `makeArray`, `subWords`; `STRING` is already
bound, from Task 2). `MAKESTRING` stays unbound, so `say buf` stays loud and the witness must not
contain it.

## The trap in this family, measured before you were dispatched

**A caseless method is not its twin with the bytes folded.** `POS` reproduces an upstream window
overrun -- `range` bounds neither where a match fits nor where it begins -- and the oracle's
`caselessPos` does **not**, because it walks `range - needle + 1` probes one at a time. Measured on
the oracle at `7e1cfb753`, from a fresh directory, over `.MutableBuffer~new('axan')`:

```
buf~pos('an', 1, 3)          -- 3
buf~caselessPos('an', 1, 3)  -- 0
buf~caselessPos('AN', 1, 3)  -- 0
```

One search, one set of arguments, two answers. `builtin/string.rs`'s `find_forward` doc block
records the same asymmetry for the `String` class and cites DEVIATION 3.

So: **fold-then-call-the-existing-core is a hypothesis, not a design.** For every one of the eleven,
measure the oracle for the cases where the two scans could differ before you choose an
implementation, and say in the report which twins share a scan and which do not. Where they do not,
the caseless body needs its own scan rather than the case-sensitive core, and the report says so
with the probe that proves it. `LASTPOS` is clean of the overrun; do not assume that means
`caselessLastPos` is a fold away either -- measure it.

## Method bodies

Task 1 §4.1's discipline: **convert every argument first** (copying argument bytes into
`take_result_buffer()` where a `&[u8]` is needed alongside the state), **then**
`buffer_state(interp, receiver, b"NAME")?` -- `buffer_state_mut` only for `CASELESSCHANGESTR` -- and
the scan over the bytes. A `None` from `buffer_state` is the `Loud::native_method` refusal, never a
panic. Task 3b §7 is live: where the C++ returns or refuses before converting a later argument, the
body must do the same, so read each method's conversion order rather than assuming the argument
list's.

Folding is ASCII, as `case_shift_bytes` and `option_letter` already do it (`to_ascii_uppercase`);
if any of these eleven folds differently in the C++, that is a finding and belongs in the report
before it is coded around.

## Refusals

**Measure every refusal on the oracle first** (a two-line program per case, fresh directory, three
descriptors), as Task 3b's §2.3 and Task 3c's tables do. The helpers exist:
`required_position_argument`, `optional_position_argument`, `required_length_argument`,
`optional_length_argument`, `required_string_argument`, `optional_string_method_argument`,
`string_method_argument`, `pad_method_argument`, `option_method_argument`,
`optional_non_negative_argument`, and the named family
(`named_string_argument`, `named_position_argument`, `optional_named_length_argument`,
`named_pad_argument`) with its 88.910/88.911/88.912 raisers. Task 3c found that `replaceAt` uses the
C++ *named* argument overloads where its neighbours use positional ones: check which overload each
caseless method uses rather than copying its twin's refusals.

## The witness: `rust/corpus/lang/mutablebuffer_caseless.rex`

**Written and confirmed on the oracle before you touch the crate** -- rc 0, stderr empty, both
engines byte-identical after. Mixed-case data throughout, and for each name at least: a match that
only the caseless scan finds, a match both find, and a miss. The `caselessPos` overrun pair above,
in both directions. The `CASELESSCHANGESTR` growth case, read back through `getBufferSize`. Contents
read back with a reader (`buf~string` is available, Task 2 bound it; Task 3c's brief said otherwise
and was wrong). `buf`, never `b`. **No `say buf`, no concatenation of a buffer, no `makeString`.**

Task 3c's lesson is the one to carry: **a line that reaches a path is not a line that observes it.**
Write down, per method, which printed value would change if that method's scan were wrong, and check
each claim against the witness text you actually committed -- not against your exploration program.

**File it in this commit**: `corpus/phase-5c.txt` and `coverage.rs`'s `EXPECTED_SUBSET_5C`, **not**
`corpus/unfiled.txt`; plus the `rexx-parse/tests/sourceline_oracle/mutablebuffer_caseless.txt`
companion, generated with the driver in that test's module comment. The pin's inversion -- the line
absent from `EXPECTED_SUBSET_5C` -- must redden `phase_5c_subset_matches_the_committed_list`.

## `corpus/method-bodies.txt`

**Predict first, in your report**: which `MutableBuffer` instance rows move under the receiver
`.MutableBuffer~new('abc')` with a zero-argument send, and the evidence each carries -- measure the
oracle's refusal per name. `CASELESSCHANGESTR` answers the receiver, so its probe's own `say`
reaches `MAKESTRING` and its row stays `loud` with the evidence moving to `MAKESTRING`, as five of
Task 3c's did; predict that per row rather than assuming it. Then
`REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p rexx-exec --test method_bodies` and read the
diff row by row against the prediction. **The gate: no row moves to `diverge`**, and the file's
`diverge` total (7) does not change.

## Controls, each predicted before it runs

* The witness against BASE's binary (a pristine `git archive <BASE> rust interpreter` extract under
  `/home/moritz/dev/repos/claude-build-scratch/5c-followup-task3d/`, own `CARGO_TARGET_DIR`; the
  archive needs `interpreter/` as well as `rust/`, two build scripts read it): rc 120 at the first
  caseless send, stderr naming it.
* **The corpus control that can go red is `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec
  --test corpus`**, read with its `N of M matching` line. The plain `--test corpus` binary is report
  mode and exits 0 on a divergence. Never cite it as a control.
* At least one mutation per distinct scan you write, plus one that changes the folding, each
  predicted (which test or program catches it, by what reading), run with `--profile mutation`
  (`target/mutation/`; the release binary is untouched and its sha256 must not change), restored
  from a copy, `touch`ed, sha confirmed. **Run the suite without the new witness wherever the new
  witness is the claimed catcher.** Task 3b's M4 is the precedent for a mutation going red by a
  panic rather than a mismatch, and Task 3c's M2 for a prediction that was simply false: if one
  comes back green, that is a finding about the witness, not a mutation to replace.
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
(`measured, oracle rc 163: …`) may stay; no history, no set sizes. Scratch under
`…/scratchpad/task3d/`, builds under `claude-build-scratch/5c-followup-task3d/`.

## Report and hand-back — commit-before-gating, no exceptions

Report `docs/superpowers/records/2026-09-04-phase-5c-followup/task-3d-report.md`: skeleton first;
every claim **measured** (with the command or file) or **inferred**; the witness's text and the
oracle's three descriptors; every refusal measured, in a table like Task 3b's §2.3; **which twins
share a scan and which do not, with the probe for each**; the predicted and measured
`method-bodies.txt` moves; each control's prediction and reading; files created outside the tree;
open questions. Fast checks yourself -- `fmt`, `clippy`, `--lib`, STRICT corpus, `--test
refusal_sites` (constructor definitions are cited by line; an insertion in `error.rs` moves every
row below it, so re-derive from the test's own panic by script, never by hand), `--test
method_bodies`, `--test coverage`, `cargo test --release -p rexx-parse --test sourceline_oracle`,
run counts asserted, **in the foreground with a bounded timeout** where possible. **Commit code and
report together** with the Gates table carrying `**G1**`–`**G7**` (copy from
`task-3c-report.md`), start the seven gates in the background from `rust/` writing
`…/scratchpad/task3d/gates/status.txt` (commit sha its first line, pidfile beside it, `finished`
its last), final message `committed at <sha>, gates running, statuses at <path>`, stop. **If you
background any job and wait on it, send the controller its pidfile and status-file paths before you
go idle** -- notifications were lost repeatedly this phase, and one agent died on a usage limit with
its work uncommitted.
