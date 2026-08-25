# Task 5, fix round 2 -- re-review

Scope: `938916aa2..772cb9852`, one commit, read from
`.superpowers/sdd/2026-08-17-phase-5a/review-938916aa2..772cb9852.diff` in three passes (README and
the 63 regenerated class probes; `gate_table_c.rs`; `gate_table_d.rs` and `gate_tables/mod.rs`). The
63 probe hunks are one template with the name substituted, so they were read as a template plus a
spot check of the long-name and short-name ends.

Every mutation was applied to a copy of `rust/` at `.../scratchpad/t5rr2/repo/rust`, with symlinks
supplying `interpreter/`, `oodocs/` and `ootest/`. The copy's `target/` was deleted and the two test
binaries rebuilt inside it, because `env!("CARGO_MANIFEST_DIR")` is baked in at compile time and a
reused binary would have read the **real** corpus; verified after the build that the binary contains
the scratchpad path and not the repository path. The real repository was never modified: `git status`
was clean at the start, no `git` command other than `show`/`status`/`log` was run, and the copy's
`corpus/` was `diff -r`-clean against the repository's after every mutation.

**Verdict: ACCEPT, with four documentation corrections.** All six findings are closed, all three
controls redden independently, and I could not defeat either new bound. The four new items are
prose-only -- none of them can turn a row green or drop the gated count -- but one of them
(**A**) is the answer to the first branch of the N1 ruling stated as an impossibility that is
measurably false, which is worth correcting before the task closes.

## The six findings

| # | | |
|---|---|---|
| N1 | **ADDRESSED** | Table C: `class_probe_shape` (`gate_table_c.rs:621`) returns `Exactly(8)` or `Exactly(2)` and can never return zero; `check_entry_present` (`:668`) carries the discrimination a bound cannot; `ENTRY_KINDS`/`check_entry_kinds` (`:637`, `:640`) make an unrecognised `entry` structural. Table D: `refusal_answered` (`gate_table_d.rs:307`), gated at `:515` and folded into the verdict at `:531`. All three controls re-run below. |
| N2 | **ADDRESSED** | `README.md:42-67`. The lead sentence is now true for every family, the per-directory bound is named, `classes/` is stated as gated on `entry`, and the `unanswered`-still-counts consequence is spelled out. One new inaccuracy in the same paragraph -- finding **B**. |
| N3 | **ADDRESSED** | `gate_table_c.rs:1884` sums `method_measured` entries whose verdict `is_none()`, which is exactly what the ruling asked for. Measured 497, matching the `unanswered: 497` column beside it. |
| N4 | **ADDRESSED** | `UNANSWERED`, `verdict_label` and `stdout_lines` are defined once each, in `gate_tables/mod.rs:377`, `:385`, `:397`. Grepped the whole of `crates/`: no second definition of any of the three, and `stdout_line_count` is gone. |
| N5 | **ADDRESSED** | Both sentences struck (`check_probe_text`'s error arm at `gate_table_c.rs:1127ff`, `first_difference`'s doc at `:1197ff`), and `Measured::verdict`'s "dropping it would ..." kept at `:978` as ruled. The same round put a new sentence of that shape into `gate_table_d.rs` -- finding **C**. |
| N6 | **ADDRESSED** | `gate_table_c.rs:30` heading now reads "and one expected count"; `:36-41` names `Concept::oracle_lines` as the exception and says why the concept family is where a bound has to be committed. |

## The three controls, re-run independently

Each run was `REXX_PHASE_GATE=5a REXX_CORPUS_GATE=1` on the rebuilt binary in the copy, status read
unpiped, stdout and stderr captured to separate files.

**Baseline first**, so the controls have something to move: table C exits **101** at
`5a: 135 rows, 135 not yet 'agree'`, table D exits **101** at `5a: 36 rows, 10 not yet 'agree'`.
Neither baseline run has a `structural failures` section -- both panic on the verdict count. That
matches the report's `145 of 171`.

| control | observed exit | observed verdict cell | observed counts |
|---|---|---|---|
| A' -- `Zork` row filed `entry = instance`, correctly derived `classes/zork.rex` committed | **101** | row reads **`unanswered`** | `5a: 136 rows, 136 not yet 'agree'`; **one** structural failure, and it is the right one |
| B' -- `Array` filed `entry = klass` | **101** | row reads **`unanswered`** | `5a: 135 rows, 135 not yet 'agree'`; **three** structural failures |
| D3' -- `directives/attribute__external__subkeyword.rex` replaced by `nop` | **101** | row reads **`unanswered`** | `5a: 36 rows, 10 not yet 'agree'` -- the open count does **not** fall to 9; one structural failure |

The messages, verbatim from the runs:

* A': `gate-tables/classes/zork.rex: the oracle does not resolve .Zork to an 'environment' entry: its
  'entry ' answer is Some(".ZORK"), which is how an unresolved environment symbol renders. ...`
  The row's own report line shows the oracle's stdout as `"entry .ZORK\nclass-of-entry String\n"` --
  two lines, so the `instance` bound of 2 is satisfied and the presence check is what catches it,
  which is the design the report claims.
* B': all three consequences fire, in this order --
  `class-set.txt row Array: its 'entry' column reads "klass", which is not one of ["class",
  "instance"] ...`; `Array <- OrderedCollection: its child, Array, is not a 'class' row of
  class-set.txt ...`; `gate-tables/classes/array.rex: the oracle answered 8 line(s) where this row's
  probe asks for exactly 2 ...`.
* D3': `gate-tables/directives/attribute__external__subkeyword.rex: the oracle refuses this row's
  probe, so its answer is the report it writes on 'stderr' -- and it wrote none. ...`

All three match the report's claims exactly, including which of the three consequences B' shows
(`check_entry_present` does not fire there because `&&` short-circuits after the bound fails, and
the report does not claim it does).

## Attacking the new bounds

**Table C.** I could not construct an input that reads `agree` or lowers the 5a open count with the
row's subject absent or wrong.

* The bound alone is defeated in the `instance` arm -- `.Zork` answers the two opening questions --
  and that is exactly why the presence check exists. Measured on the oracle from the run's own
  report: `.Array` -> `entry The Array class`, `.RexxInfo` -> `entry a RexxInfo`, `.Zork` ->
  `entry .ZORK` with `class-of-entry String`. Every measured claim in `check_entry_present`'s doc is
  true.
* Miscategorisation in the other direction reddens too: a real class filed `instance` answers 8
  lines against a bound of 2 (this is what B' shows for `Array`), and a real instance filed `class`
  answers 2 against a bound of 8.
* The `ENTRY_MARKER` hole the report names is **not** a real hole. The marker is one `const` used by
  both the derivation (`class_probe_entry_questions`) and the reader (`check_entry_present`), and
  `check_probe_text` compares every committed probe against the derivation in both directions -- so a
  probe that stopped printing the marker line is a structural failure before the presence check ever
  runs. Nothing outside the file pins the *spelling*, but nothing needs to: changing the const
  changes both sides and regenerates the probes. Two independent checks, and the bound running
  alongside is adequate.
* The edge family is not reachable through this: `edge_probe_text`'s bound is non-zero, so an edge
  over an absent class reddens on its own bound, and `check_edge_endpoints` additionally requires
  both endpoints to be `class` rows that carry the presence check.

**Table D.** I tested the report's central claim -- that a non-zero `stdout` bound provably does not
exist for the seven -- rather than accepting it. It is **almost** right, and the "almost" is finding
**A** below. Separately I confirmed the admitted wrong-subject gap is real but inert today:
substituting `class__class__subkeyword.rex`'s body into `attribute__external__subkeyword.rex` gives
exit **101** with **zero** structural failures and the row still `diverge-both` at `5a: 36 rows, 10
not yet 'agree'`. So the new bound does admit a wrong-subject probe, as the doc says it does, but no
such body turns the row green on this build, because this crate is loud on all seven refusals and
`agree` would require it to reproduce the oracle's report byte for byte.

## New defects the fix introduced

### A. LOW-MEDIUM -- the impossibility claim that answers the ruling's first branch is false, measured

`gate_table_d.rs:59-62`: *"Those rows print nothing on `stdout` -- measured, and it is not a probe
that could be written differently: the refusal is a translate-time or install-time failure, and both
precede the program's own first clause, so a line printed "before" the directive does not exist."*
And `:300-302`: *"**A non-zero `stdout` bound does not exist for these rows, and that was measured
rather than assumed.**"*

The premise is true and the conclusion does not follow. A program's own clauses do not run, but a
`::REQUIRES` installed *before* the refusing directive runs the required file's main code, and that
code prints. Measured on the oracle, each run from a fresh directory with the three descriptors read
separately:

```
say 'main'
::requires 'helper.rex'          /* helper.rex is: say 'helper-ran' */
::requires 'zzznofile.rex' namespace ns
```

exits **213**, stdout `helper-ran\n`, stderr `Error 43.901: Could not find file "zzznofile.rex" for
::REQUIRES.` -- a line on `stdout`, printed before the directive that refuses, for the
`::REQUIRES NAMESPACE` row.

I scoped the negative half rather than leaving the pattern open: the same construction prints nothing
for `::ATTRIBUTE EXTERNAL` (exit 166), `::METHOD EXTERNAL` (166), `::ROUTINE EXTERNAL` (158),
`::REQUIRES LIBRARY` (158), `::CLASS CLASS` (231) and `::RESOURCE LIBRARY` (231) -- six resolve or
fail before any `::REQUIRES` file is installed. So the claim holds for six of the seven and fails for
one.

Why it matters more than a prose slip: the ruling's first branch was *"first ask whether a non-zero
bound exists for those seven"*, and this sentence is the recorded answer. The `stderr` bound is a
good instrument and I am not asking for it to be replaced -- the fix should narrow the claim to what
was measured (these committed probes print nothing; a preceding `::REQUIRES` of a printing file is
the one shape that would, and it is policed in both directions so it cannot go unnoticed).

Note also that the harness would reject the helper file itself: `gate_table_d.rs:434` reports an
orphan in `directives/`, so the shape is available to the language but not, today, to this probe
corpus. That is a reason the current bound is fine; it is not a reason the sentence is true.

### B. LOW -- a new false universal about the probes, in two places

`gate_table_d.rs:301`: *"Every probe here already opens with `say 'main'`"*. `README.md:61-62`:
*"they open with `say 'main'` like the rest"*.

Measured over `corpus/gate-tables/directives/`: **75 of 79** probes contain `say 'main'`. The four
that do not are `options__digits__subkeyword.rex` (`say digits()`),
`options__engineering__value_of_form.rex` and `options__form__subkeyword.rex` (`say form()`) and
`options__fuzz__subkeyword.rex` (`say fuzz()`). They each print one line, so the *substantive*
property -- "every probe here prints one line" -- is intact; the universal about the specific clause
is not. This is the same shape as N2: a measured-sounding claim in the two artifacts a reader
consults, one of them tracked.

### C. LOW -- history reintroduced by the round that struck it

`gate_table_d.rs:296-298`: *"Measured, replacing such a row's probe with a program reading `nop` did
exactly that and took the 5a open count down by one."*

Apply the deciding test: strike it, and the two sentences before it (*"A bound of zero lines is
satisfied by a program that produced nothing at all ... a refusing row's two sides both print nothing
on `stdout`, so `compare_raw` agrees and the row reads `agree` with nothing asked"*) still state the
contract in full. It is a past-tense account of a mutation run against the code **before** this
change -- under the code as it is, that mutation reddens, which is control D3'. That is exactly the
shape of `check_probe_text`'s *"Measured: one `0xff` byte appended ... left the table exiting 0"*,
which N5 struck in this same commit. The distinction the tree draws elsewhere holds up: measuring
what the oracle or the code does **now** (`check_entry_present`'s `.Array` / `.RexxInfo` / `.Zork`
renderings) is not this shape, and those stay.

### D. LOW -- three enumerations the round made incomplete and did not update

The round added two questions to every class probe and a second check to the class family, and
updated the 63 probe headers and the README's bound paragraph. Three older enumerations of the same
facts were left behind:

* `gate_table_c.rs:22-24` -- *"**wiring rows**, one per class in `class-set.txt` -- asked `~id`,
  `~class`, `~superClass`, `~superClasses`, `~metaClass` and `~isA(.Class)`"*. The probe now asks two
  more questions first, and they are the ones the round's whole argument rests on.
* `gate_table_c.rs:56-59` -- the `# A verdict says the two sides agree; it does not say either
  answered` section says the check is *"[`OracleShape`], bounded by the derived probe text ... and by
  a committed count for the concept family"*. That is now a proper subset: the class family has a
  second, non-count check, and this section is the file's own account of what protects a verdict --
  the exact role the README paragraph N2 was about plays for a reader.
* `README.md:33` -- *"`classes/` -- one per row of `../docs/class-set.txt`, asking the questions the
  class surface is wired by"*. Two of the eight questions are now precisely **not** that.

None is false as a bare statement; all three are the round's own change not carried through. Grouped
as one item because one edit pass closes them.

## Smaller notes, no action asked

* **N3's replacement carries a causal clause that its predicate does not support.**
  `gate_table_c.rs:1889` prints *"method rows no documented name was asked of, on either side,
  because their group's probe raised first"*, counted from `verdict.is_none()`. A group whose
  **shape check** failed also has `None` verdicts and would be counted here with the wrong reason
  given. That run is structurally red, so a reader cannot meet the sentence on a green run -- which
  is the property the ruling asked for. The count itself is what was ruled and is right.
* **`class_probe_shape` takes `name` only to call `.len()` on two vectors whose lengths do not depend
  on it.** Harmless; clippy is clean on it.
* **The first line of the class-probe header comment exceeds ~72 columns for the long class names**
  (89 for `CaselessDescendingComparator`). Not a regression -- the base line was 95 for the same
  file -- and the template's fixed break points are what make the derivation name-independent.

## Verification of the report's own claims

I did not re-run the suite, as instructed. What I checked:

* **Confirmed independently.** `cargo fmt --all --check` exit **0** (empty output); `cargo clippy
  --workspace --all-targets -- -D warnings` exit **0**; both gate tables' 5a counts and exit statuses
  (135 of 135 and 10 of 36, both **101**), matching `145 of 171`; the `unanswered: 497` column and the
  `497` in the summary sentence agreeing; zero structural failures on both baseline runs, which is
  also the standing check that no probe made the two engines disagree.
* **Corroborated indirectly.** "237 probes, 0 engine disagreements" -- `run_on_both_engines` asserts
  this before any verdict exists, and both baselines reached the verdict panic, so no probe in the
  set tripped it.
* **Not verified.** `cargo test --release --workspace` = 0, the two corpus-gate runs at 106 of 106,
  and `rexx-diff` at 440 programs / 0 divergences. The report asserts these as exit statuses without
  pasting output; that is the same form the previous rounds used.

## Out-of-Scope Observations

* **`Concept::oracle_lines` has no non-zero validation.** It is the one remaining place in table C
  where a committed `0` would reconstruct finding 1 -- `OracleShape::Exactly(0)` is satisfied by a
  probe that produced nothing. All 21 committed values are between 1 and 31 today, so nothing is
  wrong now, and the field is untouched by this fix. If the plan wants the class of defect closed
  rather than the two instances, an assertion that every committed count is at least one is the
  cheap version.
* **Table D derives only the probe path, so a wrong-subject body is invisible to every check.**
  Measured above (exit 101, zero structural failures). This is table-wide, not specific to the seven
  refusing rows -- any row's probe can be swapped for any other one-line-printing program -- and it
  is pre-existing. `gate_table_d.rs:63-70` now names it with the task that could close it, which is
  what the N1 ruling's fallback asked for.
* The permission-denied limb of finding 2 and `run_probe`'s two row-dropping paths are unchanged from
  the previous re-review's observations.
