# Task 3.10 review: parse throughput on the bootstrap files

Reviewed: brief, report, and `review-cf5f7650..c1f84b0a.diff` (commits `9cd1752f`, `c1f84b0a`).

## Verdicts

* **Spec compliance: PASS.**
  All five brief steps are satisfied: `criterion = "0.8.2"` is a dev-dependency with a
  `[[bench]] name = "parse" harness = false` stanza and the required criterion settings
  (`sample_size(10)`, 500 ms warm-up, 30 s ceiling); the benchmark parses the two real
  shipped `.orx` files via `include_bytes!`, not synthetic input; a node-count assertion
  runs inside the timed closure; the measurement is written into `d10-decision.md` next
  to the D10 spike's numbers with an explicit non-cold-start disclaimer, matching the
  corrected Step 4 text from `c1f84b0a`; the work is committed (`9cd1752f`).
* **Task quality: NOT APPROVED.**
  One Critical: the deliverable repeatedly misdescribes the provenance of four of its
  six asserted numbers as reused from an independent pin, when they are not. This is
  exactly the honesty trap the task brief calls out, and it appears in the benchmark's
  own doc comment, its panic message, and in `d10-decision.md`'s prose, not just in
  the report. See C1 below. The fix is a small wording change, not a re-measurement,
  but it should happen before this ships as-is.

## Findings

### C1 (Critical): "already pin" is false for 4 of the 6 asserted numbers

`src/directive/tests.rs` (`core_classes_parses`, `the_other_shipped_packages_parse`) pins
exactly:
* `directives.len() == 347` for `CoreClasses.orx` (a direct, literal pin).
* Per-kind counts for `StreamClasses.orx`: 7 classes, 139 methods, 5 attributes, 2
  constants (which sum to 153, but 153 itself is never written as a literal anywhere
  in that file).

It contains **no** assertion anywhere about a main-body instruction count or a
nested-instruction total. I grepped the file for `41`, `2390`, `610`, and `153` as
literals; only the per-kind numbers that sum to 153 are present. `main_instructions`
(41, 7) and `nested_instructions` (2390, 610) do not exist in `tests.rs` in any form —
they were measured for the first time by the implementer's own throwaway pre-flight
test (report, "What went wrong," item 1) and then hardcoded into `benches/parse.rs`'s
`CASES` table.

Despite this, the deliverable states in three places that these are reused, pinned
values:

* `rust/crates/rexx-parse/benches/parse.rs` module doc comment: "the same counts
  `src/directive/tests.rs`'s `core_classes_parses` and
  `the_other_shipped_packages_parse` pin, reused here rather than re-derived."
* The benchmark's own `assert_eq!` panic message: "...the benchmark and the
  acceptance test in `src/directive/tests.rs` have diverged, or the parser stopped
  early" — there is no such acceptance test to diverge from for `main_instructions`
  or `nested_instructions`, so this message describes a divergence check that cannot
  happen for two of the three counts.
* `docs/superpowers/plans/d10-decision.md`, "The assertion" paragraph: "reusing the
  numbers `src/directive/tests.rs`'s `core_classes_parses` and
  `the_other_shipped_packages_parse` already pin."

The task brief's own framing anticipated exactly this ("a change-detector assertion
whose expected value came from the code under test is still useful, but it must not
be described as a pin from elsewhere"). A change-detector for `main_instructions` and
`nested_instructions` is legitimate and useful — it still catches a parser that starts
dropping directive bodies after this commit — but describing it as pulled from an
independent pin overstates its assurance: nothing outside this benchmark has ever
independently verified that 2,390 and 610 are the *correct* nested-instruction counts,
only that they are *today's* counts. The fix is wording, in all three locations above:
say plainly that `directives.len()` for `CoreClasses.orx` (347) is cross-checked
against `src/directive/tests.rs`, that the `StreamClasses.orx` directive count (153) is
consistent with that file's per-kind pins, and that `main_instructions` and
`nested_instructions` are this benchmark's own first-measured baseline, asserted as a
regression lock rather than pinned elsewhere.

### I1 (Important): the triple catches the intended trap but not every parser regression

I confirmed the model: `Program::instructions` and `CodeBody::instructions` are flat,
source-ordered clause lists — `If`, `Do`/`Loop`, and `Select` do not own a nested
`Vec<Instruction>` for their bodies; they carry control-flow target *indices* into the
same flat vector (`InstructionKind::If { false_target: Option<usize>, .. }`,
`Select { whens: Vec<usize>, otherwise: Option<usize>, end: Option<usize>, .. }`, etc.,
in `rust/crates/rexx-parse/src/ast.rs`). Because of this, a clause dropped anywhere in a
body, including inside a loop or an `IF`/`SELECT` branch, does change the relevant
count, so the triple genuinely defends against the Phase-2-shaped trap it was written
for (early stopping, or directive bodies built with no content).

It does **not** defend against two other regression shapes, both concrete:

1. **Control-flow wiring corruption with unchanged counts.** `If::false_target`,
   `Else::then_exit`, `Select::whens`/`otherwise`/`end`, `When::false_target`/`exit` are
   plain indices into the flat instruction list. A parser bug that computes one of
   these wrong — e.g. an `IF`'s `false_target` pointing one instruction short or past
   the matching `ELSE`, or a `WHEN` linked to the wrong `SELECT` — changes nothing that
   `instructions.len()`, `directives.len()`, or the nested sum observe: the same
   instructions exist, in the same order, just wired to execute in the wrong sequence.
   `parse_program` returns `Ok`, all three counts match, and the benchmark posts a good
   number over a tree with corrupted control flow.
2. **Cross-directive misattribution with an unchanged sum.** `nested_instructions` is a
   single sum across all 347 (or 153) directives. A boundary-detection bug that moves a
   clause from directive N's body into directive N+1's body — for instance an
   off-by-one on where one `::METHOD` body ends and the next directive's begins —
   leaves `directives.len()`, the main count, and the *sum* all unchanged, while the
   wrong method now owns the wrong code.
3. **Expression-level corruption.** The brief's own example: a wrong operator, a
   dropped operand, or wrong precedence inside an `Assignment`, `Message`, or `Command`
   expression changes nothing that any of the three counts observe, since none of them
   look inside an `Expr`.

A fourth count would close (2) but not (1) or (3): comparing the **per-directive
vector** of body lengths (or a cheap hash of it) against a pinned vector, rather than
comparing only its sum, would catch a body-boundary misattribution that a sum cannot.
Closing (1) or (3) needs something that observes tree *shape*, not node counts —
options in that direction include hashing the sequence of `InstructionKind` discriminant
tags together with every control-flow target field, or summing each node's byte-span
length as a structural checksum. Any of these would still be cheap enough to sit inside
the timed closure (it's the same order of work as the existing `nested` computation),
but none is a "count" in the sense the brief asked about, and full protection against
(1)-(3) really requires an equality oracle over the parsed tree, which is a materially
bigger undertaking than this task's scope. I read this as a real, pre-existing gap
shared with `src/directive/tests.rs`'s own acceptance tests (which also only assert
per-kind directive counts, not structure) rather than a regression this task
introduced — but the brief asked me to judge sufficiency, and my judgment is: sufficient
against the specific trap named, not a general structural regression detector.

### Clone placement and cost — confirmed as claimed, materially negligible either way

The clone is genuinely outside the timed region: `bencher.iter_batched(|| case.text.to_vec(), |text| { ... }, BatchSize::SmallInput)` —
criterion's `iter_batched` runs the first closure (the clone) untimed as setup and times
only the second closure (parse + node-count computation + assertion). Code and prose
agree on this placement.

I measured the clone's own cost directly rather than only estimating it: cloning a
141,049-byte `Vec<u8>` 200,000 times in a release-mode Rust binary on this machine
averaged **1.01 µs/clone**. Against a 2.6 ms parse, that is about **0.04%** — immaterial
whether counted or not. The report's stated reason for excluding it (the interpreter's
real cold-start path never clones, so charging one here would overstate the isolated
cost) is directionally correct, but the practical stakes are near zero either way; the
prose's framing ("would have overstated the cost") is a bit more dramatic than the
~0.04% actually at issue.

### M1 (Minor): "only `parse_program` itself is inside the timed region" is not quite true

Both the report and `d10-decision.md` say the timed region contains only the
`parse_program` call. In the actual routine closure
(`rust/crates/rexx-parse/benches/parse.rs`), the `nested` sum (an O(directive count)
loop over up to 347 directives, cheap match arms, no allocation) and the triple
`assert_eq!` also run inside that same timed closure. I confirmed this adds no material
cost — it's the same computation criterion's own benchmarking loop already re-runs once
per sample regardless of harness, and its cost is dwarfed by the parse itself — but the
prose overstates what is literally being timed. Should say "parse_program and the
node-count check that guards it" rather than "only parse_program."

### M2 (Minor): "comparable with `perf-baseline.md`" overstates what exists there

`d10-decision.md`'s new section and the benchmark's own comment both say matching
`rexx-bench`'s and `rexx-num`'s criterion settings "keeps the numbers comparable with
`perf-baseline.md`." I checked `docs/superpowers/plans/perf-baseline.md`: it documents
the *settings* (`sample_size = 10`, 500 ms warm-up, 30 s ceiling) that this benchmark
matches, but it has no row for `rexx-parse`, `parse_program`, `CoreClasses.orx`, or
`StreamClasses.orx` to actually compare against. The claim is true in the narrow sense
of "methodologically consistent with the settings recorded there," not in the sense of
"there is an existing comparison." Worth a word change, not worth blocking on.

### Numbers I could cheaply check, and did

* Line counts: `wc -l` gives `CoreClasses.orx` 4,193, `StreamClasses.orx` 1,010, total
  5,203 — matches the brief, the report, and every place these appear in the diff and
  in `2026-07-28-phase-3-parser.md`.
* Byte counts: `wc -c` gives `CoreClasses.orx` 141,049, `StreamClasses.orx` 37,603 —
  matches the report and `d10-decision.md` table exactly.
* `criterion` is a `[dev-dependencies]` entry (not a normal dependency), the
  `[[bench]]` stanza sets `harness = false`, and `rust/Cargo.lock` gained exactly one
  line (`"criterion"` added to `rexx-parse`'s dependency list) — `criterion 0.8.2` was
  already locked in the workspace via `rexx-num`/`rexx-bench`, so no new transitive
  packages were pulled. Confirmed by inspecting the lockfile directly.
* No existing comments were deleted in the `Cargo.toml` diff; the new stanza is purely
  additive.
* `docs/superpowers/plans/2026-07-28-phase-3-parser.md` genuinely has twelve tasks
  (3.1 through 3.10, including 3.7b and 3.7c), ending at 3.10 — confirms the
  `c1f84b0a` fix commit's claim that "Task 3.11" was never real, and that the plan-text
  edit does not contradict anything.
* The Step 4 rewording in `c1f84b0a` no longer contradicts the surrounding correction
  paragraph (it previously said "say plainly whether it fits"; now says "say plainly
  what share of that budget parsing accounts for, and what is still unmeasured").
* The report's claimed duplicate-sentence bug ("on the order of 3.3 ms" beside
  "3.27-3.30 ms") is not present in the final `d10-decision.md` text — only one phrasing
  survives. The report's own account of catching and fixing it checks out.
* The new `d10-decision.md` section explicitly disambiguates itself from the D10 spike
  ("The number above is the spike's, and the spike is gone... recorded here because it
  sits next to the spike's numbers and a later reader should not confuse the two"), and
  no sentence in the new section claims or implies a cold-start verdict — the closing
  paragraph explicitly states the combined figure and then says "Nothing here says
  whether the other components fit, because none of them has been measured."

### Cannot verify from this diff

* The machine identity and `rustc`/`criterion` version strings in the new
  `d10-decision.md` section are not independently re-verifiable from the diff alone,
  but this is covered by the task's "Already verified" section: I reran the benchmark
  myself and reproduced the stated ranges within noise, which is the strongest
  available check on measurement authenticity.
* The per-file test counts listed in the report's Verification section (`program.rs`
  32, `directive/tests.rs` 253, `errors.rs` 23, `scanner.rs` 35, `sourceline.rs` 25,
  `tokens.rs` 9) were not individually re-derived; the aggregate 561-passed/0-failed
  claim is already confirmed in the task's "Already verified" section.
