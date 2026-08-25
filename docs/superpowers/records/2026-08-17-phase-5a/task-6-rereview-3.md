# Task 6, re-review of fix round 3

Fix base `be81ce689`, head `6572d67cc`, two commits (`56c842cb0` the fix, `6572d67cc` the
sitting rows). Read the diff package once; everything below is measured, not read off the
report.

**Round verdict: the mechanism is right and the measurements hold; the prose is not.** The
fifth adapter is correct and does not over-fire anywhere I could reach -- 48 probe programs
and the 12 corpus witnesses, both engines, every one byte-identical to the oracle or
identical to the pre-task build. The
witness, the corpus arithmetic, the control and the sitting rows all verify. What does not
verify is the writing: **eight new prose defects, NEW-1 to NEW-8 below**, and three of them
are the exact shapes this round was fixing -- a set size in a comment, a historical framing,
and a min/max collapse. On top of those, the diff made one of the code's own enumerations
stale, and the two report sentences finding 2 named still stand unedited.

---

## Finding verdicts

| # | Finding | Verdict |
|---|---------|---------|
| 1 | Comparison was the same unmet class -- wrap it | **ADDRESSED** -- `eval.rs:1303` wrapper + `eval.rs:1330` `compare_values_body`, blame at `eval.rs:1323`; witness `corpus/lang/operator_frame_stem_compare_overflow.rex`; corpus 118/118 |
| 2 | Two false performance sentences | **NOT ADDRESSED at the cited lines** -- `task-6-report.md:436` and `:724-725` still say both, verbatim and unmarked; the true statement exists only at `:836-850` |
| 3a | `eval.rs:1053` `arith_left_operand` shares the blame helper | **ADDRESSED** -- `eval.rs:1053-1056`, and verified: the five call sites are `run.rs:7298`, `eval.rs:822`, `:979`, `:1323`, `:1477`; `arith_left_operand` is not among them |
| 3b | `eval.rs:973` "exactly as before" | **ADDRESSED but re-broken** -- the phrase is gone; the replacement at `eval.rs:973-977` is itself historical (see NEW-5) |
| 3c | `eval.rs:972` "all three past ..." | **ADDRESSED** -- now "each of them past `arith_left_operand`'s own return" (`eval.rs:967`) |
| 3d | Report perf table breaking its own convention | **ADDRESSED in the table, re-broken in the prose** -- the round-3 table at `:914-939` lists all 24 cells with median, min and max and matches the TSV row for row; the prose around it breaks the same convention twice (NEW-7, NEW-8) |
| 3e | `operator_frame_stem_divide_by_zero.rex` "the divisor's own arithmetic overflow" | **ADDRESSED** -- corrected in both files, and the correction is right: measured `Error 42.3: Arithmetic overflow; divisor must not be zero`, not 42.901 |

---

## 1. Comparison's blast radius

`compare_values` has exactly two `Err` sources in its body -- `Loud::operator_operand` from
the left-operand gap check, and `rexx_num::compare_numbers` -- and it is reached from one
place, `apply_binary` (`eval.rs:1586`), which both engines enter. So the wrapper's reach is
"any comparison whose left operand is a stem and which raised". I measured 48 probe
programs, oracle vs. both engines, stdout/stderr/rc read separately, from an empty run
directory outside the repository.

**Nothing over-fired. Nothing regressed. Every program below is byte-identical between the
oracle and both engines unless marked.**

### No stem anywhere

| program | oracle | rust (tw, ir) |
|---|---|---|
| `say 'abc' > 1` | rc 0, `1`, no frame | MATCH |
| `say 1 = 'x'` | rc 0, `0`, no frame | MATCH |
| `numeric digits 1; say '9.9E999999999' > 1` | rc 214, **no frame** | MATCH |

The third is the one that matters: two literals overflow, the oracle emits no frame because
neither operand is a stem, and the wrapper stays silent.

### Stem on the right (required to stay frameless)

| program | oracle | rust |
|---|---|---|
| `numeric digits 1; s. = '9.9E999999999'; say 1 > s.` | rc 214, no frame | MATCH |
| same with `1 = s.` | rc 214, no frame | MATCH |

### Strict comparison (required to stay frameless)

`==`, `\==`, `>>`, `<<`, `>>=`, `<<=`, `\>>`, `\<<`, each with the overflowing stem on the
left: **all eight** rc 0, no frame, on the oracle and on both engines. Strict operators
never reach `compare_numbers`, so the wrapper never sees an `Err`.

### The non-numeric fallback

`s. = 'abc'; say s. > 1` -- rc 0, `1`, no frame, MATCH. `compare_strings`, not
`compare_numbers`.

### Implicit comparison -- the cases no earlier round touched

These are the ones a wrapper on the comparison path could change without a visible operator
in the program, and they are the reason this attack was worth running. All measured with
`numeric digits 1`, `s. = '9.9E999999999'`:

| context | program shape | oracle | rust |
|---|---|---|---|
| `IF` | `if s. > 1 then ... else ...` | rc 214, **frame**, `3 *-* if s. > 1` | MATCH |
| `SELECT`/`WHEN` | `select; when s. > 1 then ...; otherwise ...; end` | rc 214, **frame** | MATCH |
| `DO WHILE` | `do while s. > 1 ... end` | rc 214, **frame** | MATCH |
| `DO UNTIL` | `do until s. > 1 ... end` | rc 214, **frame**, raised at the `end` clause after one body pass | MATCH |
| `IF a, b` comma list | `if 1 = 1, s. > 1 then ...` | rc 214, **frame** | MATCH |
| `DO i = 1 TO n WHILE` | `do i = 1 to 3 while s. > 1` | rc 214, **frame** | MATCH |
| `DO i = 1 TO n` own termination test | `do i = 1 to 5; i = '9.9E999999999'; end` | rc 214, **no frame** | MATCH |
| `SELECT CASE` | `select case s.; when 1 then ...; otherwise ...; end` | rc 0, `other` | MATCH |

The seventh row is the structurally interesting one and it comes out clean for a reason
worth recording: the controlled loop's own bound test does **not** go through
`compare_values`. `loop_advance` answers it with the integer fast path or
`controlled_within_wide` (`run.rs:8916-8925`), and the increment goes through
`arith_operand`, so the comparison wrapper cannot fire there at all. The eighth row is
clean because `SELECT CASE` compares strictly (`run/tests.rs:1397`), which never reaches
`compare_numbers`.

### The receiver-side asymmetry, tested both ways

`numeric digits 1; s. = 1; say s. > '9.9E999999999'` -- stem receiver converts fine, the
**right** operand is what overflows. Oracle: rc 214 **with** the frame. Rust: MATCH. Same
for `=`. So the round-2 rule ("blame on any failure past the receiver") is the oracle's rule
for comparison too, and the wrapper implements it in the right place.

### Every comparison operator, swept

All eighteen `is_comparison` operators against the overflowing stem, oracle vs. both engines:

```
>  <  >=  <=  =  \=  \>  \<  <>  ><      rc 214, frame, MATCH   (ten, non-strict)
== \== >> << >>= <<= \>> \<<             rc 0,  no frame, MATCH (eight, strict)
```

**This is where the diff's prose is wrong.** Ten non-strict comparison operators reach the
numeric path and carry the frame, not six -- `\>`, `\<`, `<>` and `><` do too, measured
above. See NEW-1.

### Adversarial shapes around `is_stem_receiver`

| program | oracle | rust | note |
|---|---|---|---|
| `t. = '9.9E999999999'; s. = t.; say s. > 1` (nested stem default) | rc 214, frame | MATCH | the redirect is chased correctly |
| `numeric digits 1; s.a = '9.9E999999999'; say s.a > 1` (compound, not stem) | rc 214, **no frame** | MATCH | receiver is a `String`, no forward |
| `say b. == .array` (unset stem, strict, class right) | rc 0, `0` | MATCH | |
| `s. = 1; say s. > .array` (class on the right) | rc 0, `0` | MATCH | |
| `numeric fuzz 0` + `<=` overflow | rc 214, frame | MATCH | |
| `s. = .environment; say s. > 1` | rc 0 | **DIVERGE** (loud Phase-5 refusal) | pre-existing, identical on the pinned pre-task build |
| `s. = .array; say s. > 1` | rc 159, 97.1 | **DIVERGE** (loud Phase-5 refusal) | pre-existing, identical on pinned |
| `s. = .nil; say s. > 1` | rc 159, 97.1 | **DIVERGE** (rc 0, `1`) | pre-existing, identical on pinned -- see out-of-scope |

## 2. The escape hatch

Verified by measurement, not by reading the report. The report's four-row escape-hatch table
reproduces exactly:

```
s. == 1   rc 0, "0", no frame     (measured, oracle and both engines)
s. >> 1   rc 0, "1", no frame
s. = 'abc'; s. > 1   rc 0, "1", no frame
1 > s.    rc 214, no frame        (raises, but is_stem_receiver(1) is false)
```

The hatch genuinely did not need to trigger: `is_stem_receiver` discriminates on the
receiver, and every frameless case is frameless either because it never raises (strict, and
the string fallback) or because its receiver is not a stem (`1 > s.`).

## 3. The witness and the corpus arithmetic

* Entries: `phase-5a.txt` 62 -> 63; `phase-4a/4b/4c` unchanged at 31/12/12. 55 + 62 = **117**
  before, 55 + 63 = **118** after. Sorted set diff of the subset file between base and head
  is exactly one added line and **zero removals** -- nothing left the set.
* `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus corpus_differential`
  run by me: `118 of 118 matching`, `ok`. The report's `118 (117 + 1)` is right.
* The new witness measured **directly**, not through the harness excerpt:
  `corpus/lang/operator_frame_stem_compare_overflow.rex` is rc 214 on the oracle, stderr
  opening `       *-* Compiled method ">" with scope "String".` then the 42/42.901 pair, and
  both `REXX_ENGINE=tree-walker` and `REXX_ENGINE=ir` reproduce all three descriptors byte
  for byte.
* `operator_frame_stem_divide_by_zero.rex` re-measured after its comment edit: unchanged
  behaviour, rc 214, `Error 42.3: Arithmetic overflow; divisor must not be zero`. The
  corrected comment is factually right and the old one was wrong.
* Both `sourceline_oracle` fixtures check out: `count 17` / `count 10` match the files' own
  line counts, and each fixture body is byte-identical to its `.rex`.
  `cargo test -p rexx-parse --test sourceline_oracle` passes.
* `cargo test --release -p rexx-exec --test coverage` (17 tests) and `--test corpus`
  non-gate (17 tests, 1 ignored) pass, so `EXPECTED_SUBSET_5A` and `RAW_STDERR_COMPARISON`
  agree with the subset file.

## 4. The control, twice

**The pinned pre-task build (the stronger control the brief pointed at).**
`bench-baselines/pinned/rexx-run-15a1ffa98` against the oracle on all twelve
`operator_frame_stem_*` programs: **twelve of twelve differ by exactly one stderr line, and
that line is the `Compiled method ... with scope "String".` frame**; stdout identical, exit
status identical, on every one. So every program in the set -- including the new one -- was
divergent before the task and is fixed by it, and the fix's effect on them is only the frame
line.

**The report's own mutation, reproduced independently.** I copied `rust/` outside the
repository, symlinked `interpreter/`, `oodocs/` and `ootest/`, built fresh, and put the
disable behind an environment variable so both arms come from *one* binary:

* mutation off: `118 of 118 matching`, `corpus_differential ... ok` -- the copy is sound.
* mutation on: `106 of 118 matching`, `corpus_differential ... FAILED`, and the twelve
  `[UNCLASSIFIED]` programs are exactly the twelve `operator_frame_stem_*` entries. The
  other 106 stayed green, so the mutation disables the mechanism and nothing adjacent.
* the mutated `rexx-run` diffed directly against the oracle on all twelve: one stderr line
  each, and it is the frame line, stdout and rc identical.

The report's transcript is real failing output and its `106 of 118` is exact.

## 5. The sitting, read off the rows

`rust/bench-baselines/phase-5a-arms.tsv` rows citing `56c842cb0`: **312 rows appended, zero
deleted, every appended row citing that one commit**, `n=5` throughout, six axes -- matching
the recorded invocation. Pin staleness re-checked: `15a1ffa98` is an ancestor of HEAD and
`git log 15a1ffa98..HEAD -- rust/crates rust/Cargo.toml rust/Cargo.lock Cargo.toml` lists
`56c842cb0` on top of this phase's own commits, none foreign.

**The 24-row table in the report matches the TSV cell for cell, including min and max.** The
finding-5 shape is fixed *in the table*. It is not fixed in the prose around it:

| claim | line | truth |
|---|---|---|
| "again min equal to max on all four rows" | `:845-846` | **False.** `arith/ir/small` is min `1.001283`, max `1.001284`, in this round's sitting -- and in round 2's, so the clause is false about both sittings it covers |
| "the same four values to six decimal places, not a further increase" | `:947-948` | **False for one of the four.** `arith/ir/small`'s median is `1.001283` at `c351fa473` and `1.001284` at `56c842cb0`. Three medians are identical; this one moved by exactly one unit in the sixth decimal |
| "Only `alloc4c/ir/small` differs from an exact `1.000000`" | `:941` | **False as written** -- four `arith` rows in the table directly above differ. The intent ("of the other axes") is recoverable from the next paragraph, but the sentence is not true |
| "`arith` went from `1.000000` (min equal to max on all four rows)" for `211763aaa` | `:844-845` | True -- all four rows are `1.000000/1.000000/1.000000` |
| "Every ratio stays under the 1% threshold" | `:952` | Holds on medians, including the `cycles:u` arms (worst median `compound/ir/large` `1.007748`). Individual `cycles:u` maxima reach `1.110049`, which is the same noise earlier rounds already characterised |

The substantive conclusion -- wrapping `compare_values` did not move `arith` -- survives all
of this: the four `arith` medians are `1.001006 / 1.001284 / 1.001036 / 1.001294`, at most
one sixth-decimal unit from round 2's, and the mechanism argument (`arith.rex` never reaches
`compare_values`) is correct. It is the sentences describing the rows that are wrong.

## 6. Comment prose

House method: every contiguous comment block collapsed to one line for base and for head,
the two collapsed files diffed, and **every new line read in full** -- no keyword filter
standing in for the reading. Files: `eval.rs`, `tests/corpus.rs`, `tests/coverage.rs`,
`corpus/phase-5a.txt`, both `.rex` fixtures and both `sourceline_oracle` twins.

Verified true by measurement: everything in the new `compare_values` comment
(`eval.rs:1310-1322`) -- the overflow claim, the `s. > 1` frame, the `1 > s.` frameless
result, and the strict/fallback exemption. Verified true by reading the tree: the
`eval.rs:1053-1056` rewording (five call sites, `arith_left_operand` not one of them) and
the `compare_values_body` doc's parallel to `arith_general_body`.

ASCII-only holds in every changed file and in both commit messages. No `unsafe` anywhere in
the diff. `cargo fmt --all --check` exit 0 and
`cargo clippy --workspace --all-targets -- -D warnings` exit 0, both run by me, unpiped,
status read directly.

What the pass found is below.

---

## New breakage

**NEW-1 (moderate, and the headline). `rust/corpus/phase-5a.txt:214-215` -- a false set
size, in the round that was fixing a set size.** The comment reads "One program stands for
the **six** non-strict comparison operators that reach `compare_values`'s numeric path (`>`
here; `<`, `>=`, `<=`, `=` and `\=` were measured to carry the identical frame)". Measured:
**ten** non-strict comparison operators reach that path and carry the frame -- the six named
plus `\>`, `\<`, `<>` and `><`, each rc 214 with `*-* Compiled method "<op>" with scope
"String".` on the oracle and matched by both engines. `is_comparison` (`eval.rs:1903`)
enumerates them and `compare_op` (`eval.rs:1847`) translates all ten. So the sentence is
both a banned set-size phrase and a false one. The round's own report says of the commit
title that "nothing in this task establishes six families or six of anything" -- while the
committed comment asserts exactly such a six.

**NEW-2 (minor). `rust/corpus/phase-5a.txt:213-214` -- "`Interp::compare_values` joins the
same blame-on-any-failure shape the other **four** adapters already have".** The count is
true (`arith_general`, `apply_prefix`, `logical_values`, `header_number`) and the phrase is
still a set size in a comment, which the constraint forbids including for true counts.

**NEW-3 (minor). `rust/corpus/lang/operator_frame_stem_compare_overflow.rex:6-9` and its
twin `rust/crates/rexx-parse/tests/sourceline_oracle/operator_frame_stem_compare_overflow.txt:7-10`.**
"One program stands for the family: `>` here, and `<`, `>=`, `<=`, `=` and `\=` all reach
the identical `compare_values` path" -- no number, but the enumeration is offered as *the
family* and omits the same four operators as NEW-1. A reader takes the list for the set.
Note the twin is generated from the `.rex`, so fixing the `.rex` and regenerating fixes both.

**NEW-4 (moderate). `rust/crates/rexx-exec/src/eval.rs:1102-1106` -- an enumeration this
diff made stale and did not update.** `blame_stem_forwarded_operator`'s own doc says
"[`Interp::arith_general`], [`Interp::apply_prefix`] and [`Interp::logical_values`] each
call this on any failure of their own, and `Interp::header_number` (`run.rs`) does the same
for **the other** receiver of a real unary `+` this crate models" -- a complete accounting of
its callers, accurate at `be81ce689`. This diff added a fifth caller at `eval.rs:1323` and
left the list alone, so the helper's own doc now omits one of the sites that calls it. The
"the other" construction is what makes it read as exhaustive.

**NEW-5 (minor). `rust/crates/rexx-exec/src/eval.rs:973-977` -- the replacement for the
struck "exactly as before" is itself historical framing.** It now reads
"`blame_stem_forwarded_operator`'s own predicate is unchanged, but this call site is not: a
non-stem receiver **now** reaches it on any failure here, where **the removed inline call**
inside `arith_left_operand` only reached it from that function's own narrower branch." The
deciding test: strike the historical framing and the sentence has nothing left to say about
the code as it is -- "unchanged", "now", and "the removed inline call" are all relative to a
past state, and the last names code that is not in the tree. This is the same defect class
as the phrase it replaced, which is why it is worth naming rather than waving through.
(Related, weaker: `eval.rs:1311-1312`'s "corrected from this task's own earlier premise"
narrates the *plan's* history rather than the code's; borderline, listed for the record and
not as a defect.)

**NEW-6 (moderate). `.superpowers/sdd/2026-08-17-phase-5a/task-6-report.md:436` and
`:724-725` -- finding 2's two false sentences are still standing.** Both are present
verbatim: ":436 ... and nothing on any path that succeeds", ":724-725 Nothing this round's
diff added runs on any path `arith`'s benchmark exercises". The round-3 section at `:836`
quotes them in the past tense -- "`:436` **said** ..." -- and states the true version, but
the text was never edited and carries no forward pointer, and the round-2 section that holds
them never mentions round 3. The report is untracked here (`.gitignore:30` ignores
`.superpowers/`), so unlike a commit message there was no obstacle to editing them in place;
"Recorded, not fixable" was correctly reserved for the commit message and does not cover
these. A reader reaching `:436` still reads a claim the round itself has shown to be false.

**NEW-7 (moderate). `task-6-report.md:845-846` -- "again min equal to max on all four
rows".** False: `arith/ir/small` is min `1.001283` / max `1.001284` in this round's sitting,
and the same cell was min `1.001283` / max `1.001284` in round 2's. This is the *third*
consecutive round in which this report's prose collapses a differing min/max -- round 2's
own finding 5 was titled "the sitting's false 'min equal to max' claim" (`:674`) -- and this
instance is inside the paragraph fixing that very finding.

**NEW-8 (minor). `task-6-report.md:947-948` -- "the same four values to six decimal
places".** `arith/ir/small`'s median is `1.001283` at `c351fa473` and `1.001284` at
`56c842cb0`: three of the four medians are identical, the fourth differs by one unit in the
sixth decimal. The conclusion ("not a further increase") is sound; the precision claim is
not. Related, same paragraph: `:941` "Only `alloc4c/ir/small` differs from an exact
`1.000000`" is false as written, since four `arith` rows in the table above it differ.

None of NEW-1..NEW-8 is a behavioural defect. The interpreter is correct on every program I
could construct.

---

## Out-of-scope observations

These are outside the fix diff, confirmed identical on the pre-task pinned build
`rexx-run-15a1ffa98`, and do not block.

* **`s. = .nil` with a comparison silently answers where the oracle raises.**
  `s. = .nil; say s. > 1` is rc 159, `97.1 Object "The NIL object" does not understand
  message ">"` on the oracle; this crate prints `1` and exits 0, on both engines and on the
  pinned build. The arithmetic sibling `s. = .nil; say s. + 1` at least fails -- but with
  41.1 `Nonnumeric value ("The NIL object")` where the oracle gives 97.1. `is_stem_receiver`'s
  own doc (`eval.rs:1130-1137`) records the oracle's 97.1 for the `+` case, so the
  arithmetic half is known; the comparison half is a *silent wrong answer* rather than a
  wrong error, which is the worse of the two shapes. Whether either belongs to Phase 5b is
  not this task's call.
* **`eval.rs:3272`** -- "`apply_prefix`, `arith_general` and `compare_values`, all four of
  which ..." is a set-size phrase in a test-module doc. The count is right (with
  `apply_binary`, four names) and it predates this diff.
* `s. = .environment; say s. > 1` and `s. = .array; say s. > 1` both take the declared
  Phase-5 loud refusal (`the operator > applied to ... is not implemented (Phase 5)`),
  unchanged by this round.

## What I ran

48 probe programs plus the twelve corpus witnesses, each against the oracle
(`ulimit -v 1048576`, `timeout -s KILL 10`, three descriptors read separately, from an empty
run directory outside the repository) and against both `REXX_ENGINE=tree-walker` and
`REXX_ENGINE=ir`. `cargo fmt --all --check` (0), `cargo clippy --workspace --all-targets --
-D warnings` (0), `REXX_CORPUS_GATE=1 ... --test corpus corpus_differential` (118/118),
`--test coverage`, `--test corpus` non-gate, `-p rexx-parse --test sourceline_oracle`. One
mutation build outside the repository, both arms. The full suite was not re-run, per the
brief. The working tree, index, HEAD and branch of this checkout were not modified.
