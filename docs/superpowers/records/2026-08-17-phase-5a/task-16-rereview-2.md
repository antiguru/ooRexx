# Task 16 re-review: fix rounds 2 and 3

Scoped re-review of `08965a822..52682fe94` -- four commits: `51103413a` (D, the corpus witness),
`5bc6aca0a` (B, C, E, J), `49e4c9e27` (the corpus-total comment), `52682fe94` (fix round 3's
retraction). Read against `task-16-rereview.md`'s defects B through J,
`task-16-fixround-2-report.md` including its "Fix round 3" section, `task-16-report.md` and
`global-constraints.md`.

**Verdict: CHANGES REQUESTED.** Two defects, both false numbers, both one-line fixes. Every one
of B through J is genuinely addressed and I found no new false statement introduced by the
fixes themselves. Defect 1 below is a C++ citation the diff *keeps* -- it predates this round --
but the brief asks for citations added or kept, the paragraph carrying one of the two copies was
reordered by this round's own J fix, and the previous re-review certified it as correct.
Defect 2 is the fix for H reproducing the shape H was about.

All three gates are green (see **Gates**). The tree is untouched: I mutated nothing and restored
nothing, because nothing needed mutating.

## Defect 1. `GuardInstruction.cpp:168` is a `{`; the call cited is at `:167` (source comment, must-change)

Two shipped doc comments cite `truthValue(Error_Logical_value_guard)` at line 168 of
`interpreter/instructions/GuardInstruction.cpp`:

* `crates/rexx-exec/src/run.rs:3543` (`Interp::exec_guard`): "The truth test is `WHEN`'s own
  (`truthValue(Error_Logical_value_guard)`, `:168`), which is 34.902 and not `IF`'s 34.1."
* `crates/rexx-exec/src/run.rs:11911` (`raised_guard_not_logical`): "`truthValue(Error_Logical_
  value_guard)` at `instructions/GuardInstruction.cpp:168` is what selects this sub-number over
  `IF`'s and `WHEN`'s."

Read at the tree the plan names, `/home/moritz/dev/repos/ooRexx` clean at `4b66b3558`:

```
$ /bin/grep -n 'truthValue(Error_Logical_value_guard)' \
    /home/moritz/dev/repos/ooRexx/interpreter/instructions/GuardInstruction.cpp
167:        if (!result->truthValue(Error_Logical_value_guard))
184:            } while (!result->truthValue(Error_Logical_value_guard));

$ awk 'NR==167||NR==168{printf "%d:[%s]\n", NR, $0}' .../GuardInstruction.cpp
167:[        if (!result->truthValue(Error_Logical_value_guard))]
168:[        {]
```

The call has exactly two sites, `:167` and `:184`. Line 168 is an opening brace. The
substantive claim -- that `truthValue(Error_Logical_value_guard)` is what selects 34.902 over
`IF`'s and `WHEN`'s sub-numbers -- is true; only the line number is wrong.

**What makes this more than a typo is that the same paragraph contradicts itself.**
`exec_guard`'s doc says, three sentences earlier, that the C++ "waits for another activity to
change one of the exposed variables the expression names (`:167`-`:185`)". That bracket is
right: `:167` is `if (!result->truthValue(...))` and `:185` is its closing `}`, so it brackets
the false-branch loop exactly. So one doc block names the same C++ line as both `:167` and
`:168`, and a reader following the second lands on a brace.

`corpus/oracle-crashes.txt` entry 7 -- which defect E's fix now points `exec_guard` at -- cites
`:167`-`:185` and `:147`-`:150`, both of which I checked and both of which land. It does not
carry the `:168` form, so the two run.rs sites are the whole of it.

**Fix:** `:168` -> `:167` at both sites. Nothing else in the neighbourhood moves.

## Defect 2. "19 warnings total" is 21 (report, must-change)

`task-16-report.md:688`, the fix for defect H:

> `cargo doc --workspace --no-deps` at 19 warnings total, 4 unresolved links and 15 links to
> private items, none of the 4 in `rexx-core` or `rexx-exec`

Run twice on the shipped tree, stable both times:

```
$ cargo doc --workspace --no-deps          # exit 0
21 warnings:
  15  links to private item
   4  unresolved link       (rexx-bench x3, rexx-classes x1)
   2  redundant explicit link target
```

The per-crate roll-ups sum to the same 21: `rexx-bench` 1 + 3, `rexx-classes` 5, `rexx-core` 1,
`rexx-exec` 2, `rexx-extract` 6, `rexx-num` 2, `rexx-parse` 1.

19 is exactly `4 + 15` -- the total was computed by adding the two categories that were looked
at rather than by counting the command's warnings, which is the shape defect H was raised for.
The two warnings it misses are `redundant explicit link target` at
`crates/rexx-exec/src/lib.rs:391` and `:392`, i.e. in the very crate the sentence goes on to
clear. They are not this round's: `git log -L 391,392:rust/crates/rexx-exec/src/lib.rs` puts
them at `533c48a57`, 2026-08-09.

The half of the claim that carries weight survives -- all four unresolved links are in
`rexx-bench` and `rexx-classes`, so this task's new intra-doc links do all resolve. Only the
total is wrong.

**Fix:** either "21 warnings, of which 4 unresolved links and 15 links to private items", or
drop the total and keep the two categories that were actually counted, which is the same remedy
F took.

## Not blocking, recorded once

`task-16-report.md:228`-`229`, the `### Corpus` section, still reads "`REXX_CORPUS_GATE=1 cargo
test --release --test corpus`: **185 of 185 matching** ... Eight programs added". D's corpus
program makes that nine and 186 today. The round-1 Gates block below it is explicitly stamped
"all on the committed tree at `8265ead58`", where 185 was right, and
`task-16-fixround-2-report.md` carries 186 with its own gates -- so this is layering, not a
claim about the current tree, and I am not asking for it. It is worth one line only because
`49e4c9e27` was made on the reasoning that a corpus total goes stale, and the neighbourhood
sweep that commit records covered source comments and not this file.

## B through J, each checked

| # | Verdict | Basis |
| --- | --- | --- |
| B | **CLOSED** | `activation.rs:564`-`:566` now reads "The readers are a set the tree can enumerate, so a number here is a fact that goes stale with nothing to notice." The enforcement clause is gone and nothing replaced it with a second unsupported claim. The four reader names above it still resolve to the four sites. |
| C | **CLOSED** | The causal clause is gone from `Interp::deferred` and from both places in the report's Determinism section. What is left is checkable and checks out -- see below. |
| D | **CLOSED** | Third independent measurement agrees: rc 222, byte-identical on all three descriptors, oracle and both engines. See below. |
| E | **CLOSED** | `exec_guard` now says "the program and its measured effect are `corpus/oracle-crashes.txt` entry 7, which must not be run"; the inline program and the 6-second figure are both gone. Entry 7 does carry that program and its effect (rc 137 at a 10 s deadline, nothing on either descriptor, 0.00 s CPU over 5.00 s elapsed). One record now, and it is the one with the do-not-run rule. |
| F | **CLOSED** | Counts deleted; the report presents the site list alone. I re-derived it: `grep -n 'SCHEDULING\|LEGALITY'` over the five files gives 19 lines / 18 markers, and every one resolves to an item the report's list names, with nothing left over. `Activation::reply`'s marker does wrap (`activation.rs:568`-`:569`), which is the report's stated reason for dropping the line count. |
| G | **CLOSED** | Tags now `min/min` (tw) and `median/min` (ir), and the TSV agrees exactly: `changed/ir` medians 12944164395 and 25838965683 (mins 12944091129, 25838847950), `changed/tw` mins 13084018670 and 26118851434, all four over `base` mins. The four quoted ratios recompute to the quoted four decimals. |
| H | **CLOSED on the substance, OPEN on the total** | Defect 2 above. |
| I | **CLOSED** | Row 6 of the fix table now names "the pair of probes", "The last two rows", "the four legality refusals, one fatal each" and the test's own name, and says the name carried a false universal rather than a count. No "five more". |
| J | **CLOSED** | `raised_guard_not_logical`'s summary line and catalogue text are one paragraph again with LEGALITY following, matching `raised_if_not_logical` six lines above. |

### D, measured a third time

Fresh empty directory, absolute paths, three descriptors read separately, never `2>&1`, both
sides bounded:

```
oracle       rc=222  stdout empty  stderr md5 58024db9a56f2c1d4e062b25a628529f
crate ir     rc=222  stdout empty  stderr md5 58024db9a56f2c1d4e062b25a628529f
crate tw     rc=222  stdout empty  stderr md5 58024db9a56f2c1d4e062b25a628529f

Error 34.902:  Value of expression following GUARD keyword must be exactly "0" or "1";
found "x".
```

`cmp` on all three descriptor pairs is byte-identical. So the third measurement agrees with the
two already on record.

The claim the corpus program exists to make true is now true. The guard test's doc says the
oracle-answer rows are `method_guard_instruction.rex`, the 99.911 raise and the 34.902 raise,
and that "the gated corpus is the stronger comparison for those": all three have a corpus
program -- `lang/method_guard_instruction.rex`, `lang/method_guard_outside_method.rex`
(a `::ROUTINE`'s `guard on`, 99.911) and now `lang/method_guard_when_not_logical.rex`. Registered
in `corpus/phase-5a.txt` as the last entry and in `EXPECTED_SUBSET_5A` in the same position, and
`phase_5a_subset_matches_the_committed_list` is green. The generated
`sourceline_oracle/method_guard_when_not_logical.txt` is `count 15` over a body byte-identical
to the program, and `wc -l` of the program is 15.

### C, checked against the two sittings it rests on

`Interp::deferred`'s surviving text makes three checkable claims and all three hold:

* "A two-object shape gave five distinct stdout orders over 30 runs, reproduced at five across
  two independent sittings, three of the same rows." `task-16-review.md:144`-`152` records
  19/8/1/1/1 and `task-16-rereview.md` records 13/9/6/1/1. Five each. Three orders appear in
  both lists (`A-after|B-after|B-replied|main-end`, `B-after|A-after|B-replied|main-end`,
  `B-after|B-replied|main-end|A-after`, each after a leading `A-replied`). Exactly three, not
  four.
* "no figure here is *the* distribution" now rests on that shape alone, and it can: the two
  sittings give five orders each with different counts (19/8/1/1/1 against 13/9/6/1/1), so the
  counts are not a property of the program.
* "that program was never preserved, so nothing here says why the two readings differ" -- the
  doc offers no cause, and the report's Determinism section offers none either. Its one
  surviving distinguishing statistic recomputes: `A-after` follows `B-replied` in 6+1+1 = 8 of
  the re-review's 30 runs and in 0 of 30 of the 18/12 reading, and `(22/30)^30 = 9.1e-5`,
  which is the "about 1e-4" quoted.

### `49e4c9e27`, the corpus total replaced by names

The rewritten doc on `a_value_returned_after_a_reply_reports_the_oracles_own_98_936` names
`lang/method_reply.rex` and `lang/method_reply_exit_status.rex` and quotes no total. Those two
names are what both prior mutation sittings measured (the re-review's at 185, round 2's at 186),
which is why I did not mutate a third time. The structural half of the sentence I did check:
`corpus.rs` runs `corpus_differential` in REPORT MODE without `REXX_CORPUS_GATE` and exits 0
while printing its mismatches (`corpus.rs:518`, `:525`, `:555`), so "the ungated run reports the
same two mismatches and still exits 0" is the harness's documented behaviour and "no instrument
in gates 1 through 3" follows.

## Binding constraints

* **No `unsafe`, ASCII only, no em-dashes.** Zero matches for `unsafe`, zero bytes outside
  `\x00-\x7F` and zero U+2014/U+2013 across every added line of the diff.
* **No set sizes in comments.** The numbers in the added comment lines are `five`, `two`, `30
  runs`, `18 and 12`, `three of the same rows`, `two independent sittings`, `34.902`, `34.1`,
  `34.2` -- measurements, error numbers and phase numbers. The one new count-shaped phrase,
  "the same two mismatches" in `run/tests.rs`, is a measurement of a mutation run and sits
  beside the two names it counts. Nothing added names the size of a set the code could
  enumerate.
* **No historical framing.** The added comments say what the code and the oracle do; the
  provenance caveat in `Interp::deferred` is about the evidence for a contract, not about what
  this crate used to do.
* **Performance guard.** The diff touches `src/` only in `///` lines, plus `corpus/`, `tests/`
  and a generated fixture, so the release binary is byte-identical and no sitting is owed. The
  report says so.

## Gates

From `rust/`, on the shipped tree at `52682fe94`, `git status --porcelain` empty:

```
cargo fmt --all --check                                             -> exit 0, no output
cargo clippy --workspace --all-targets -- -D warnings               -> exit 0, no warnings
REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast  -> exit 0
    186 of 186 matching
    grep -c '\.\.\. FAILED' over the whole log: 0
```

`cargo doc --workspace --no-deps` also exits 0, at the 21 warnings of defect 2. It is not a
gate.

## Tree state

**Unmodified.** I mutated nothing, so nothing was restored: `git status --porcelain` is empty
and `md5sum rust/crates/rexx-exec/src/run.rs` is `dedf3ec52fd9034ba17696aa7b7e1983` both before
and after this review. `target/release/rexx-run` is the current source's
(`cargo build --release --bin rexx-run` was a 0.11 s no-op before the probes), and the only
build artifacts this review created are `target/doc/`.

All oracle probes ran from a freshly created empty directory with absolute paths, bounded in
time and memory on both sides, three descriptors read separately and never `2>&1`. I did **not**
run entry 7's program or any neighbour of it that blocks; its readings are taken as given.
