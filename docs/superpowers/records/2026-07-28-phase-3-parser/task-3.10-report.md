# Task 3.10 report: parse throughput on the bootstrap files

Status: DONE.
Commits: `9cd1752f` (`Task 3.10: measure rexx-parse's throughput on the two shipped .orx files`), `c0e1321a` (fix round, below).

## Pre-flight findings

Five, against the brief and against `docs/superpowers/plans/d10-decision.md`.

1. **The brief's Step 4 wording contradicts the task's own correction of it.**
   Step 4 says "Write the measurement into `d10-decision.md` and say plainly
   whether it fits."
   The surrounding task text then says the opposite: "A sentence saying the
   budget fits would be a defect even if the number is small."
   The second instruction wins -- it is the explicit, reasoned correction, and
   it matches D10's own history of being corrected for claiming parser
   throughput "sets cold-start time directly."
   The addition to `d10-decision.md` states the combined number, states that
   the other cold-start components are unmeasured, and stops there.
2. **`Program.instructions` alone is too weak an assertion for these two
   files.**
   The "what earlier tasks built" section names `instructions` as "what you
   assert a count over," but for both files nearly all content lives inside
   `::METHOD`, `::ATTRIBUTE` and `::ROUTINE` bodies, not the main body.
   Measured before writing the benchmark, with a throwaway test:
   `CoreClasses.orx`'s main body holds 41 instructions against 2,390 nested
   inside its 347 directives; `StreamClasses.orx` holds 7 against 610 nested
   inside 153 directives.
   A parser that stopped right after the main body, or that built directives
   with every body dropped, would still pass a bare `instructions.len()`
   check -- exactly the Phase-2-shaped trap the brief itself names two
   paragraphs later.
   The benchmark asserts all three counts together, reusing the numbers
   `src/directive/tests.rs`'s `core_classes_parses` and
   `the_other_shipped_packages_parse` already pin rather than re-deriving
   them.
3. Line counts checked against `wc -l`: `CoreClasses.orx` 4,193, `StreamClasses.orx`
   1,010.
   Both match the brief verbatim.
4. Criterion version checked against `rust/Cargo.lock`: `0.8.2` is already
   locked, used by `rexx-num` and `rexx-bench`.
   Matches the brief verbatim; adding it to `rexx-parse` needed no network
   access.
5. `d10-decision.md`'s existing Axis 3 text says the whole-file re-measurement
   "must be re-measured as a whole-file parse when Task 3.11 exists," naming a
   different task number than the one this brief assigns (3.10).
   Not fixed -- renumbering the parent plan is not this task's job -- but
   flagged, because the new section this task adds now sits right next to a
   sentence naming a different task for the same measurement.

## What was built

* `rust/crates/rexx-parse/benches/parse.rs`: a criterion benchmark, harness
  `false`, one `benchmark_group("parse")` with a `bench_function` per file.
  `sample_size(10)`, 500 ms warm-up, 30 s measurement ceiling, matching
  `rexx-bench`'s `interpreter.rs` and `rexx-num`'s `arith.rs`.
* `rust/crates/rexx-parse/Cargo.toml`: `criterion = "0.8.2"` as a
  dev-dependency, `[[bench]] name = "parse" harness = false`.
* `docs/superpowers/plans/d10-decision.md`: a new subsection, "Task 3.10's
  later, different measurement: the shipped parser," placed directly after
  the paragraph that says the spike's number "is not the D2 cold-start
  number" and must be re-measured over whole files later -- this is that
  later measurement.

## The clone

`parse_program(text: Vec<u8>)` takes ownership, because `Program::source`
retains the buffer for every node's span.
Each timed call needs its own owned copy, so the benchmark uses criterion's
`iter_batched`: the setup closure clones the file's `&'static [u8]` into a
fresh `Vec<u8>`, untimed, and only the `parse_program` call inside the routine
closure is measured.
This was a deliberate choice, not the only one available: the interpreter's
own cold-start path reads a file's bytes exactly once and parses them exactly
once, so it never pays for a clone, and folding one into the timed region
here would have overstated the cost this benchmark is trying to isolate.

## Measured numbers

Machine: `AMD RYZEN AI MAX+ 395`, Linux, `rustc 1.96.1`.
Command: `cargo bench --offline -p rexx-parse --bench parse`.
Run twice; the second run's criterion output reported "No change in
performance detected" against the first as baseline for both files, so the
ranges below are the union of both runs.

| File | Lines | Bytes | Time per parse | Throughput |
|---|---|---|---|---|
| `CoreClasses.orx` | 4,193 | 141,049 | 2.619-2.636 ms | 51.0-51.4 MiB/s |
| `StreamClasses.orx` | 1,010 | 37,603 | 651.75-659.11 us | 54.4-55.0 MiB/s |

Combined: 3.27-3.30 ms to parse both files this benchmark asserts the
interpreter parses at every start.

## What this excludes

This is `parse_program`'s own cost, nothing else.
Bootstrap execution, heap setup and class construction are not measured
anywhere in the tree yet, so this number is **not** cold-start time, only a
component of the ~55 ms cold-start budget under D2.
No conclusion about whether the budget fits is drawn here or in
`d10-decision.md`, because the data cannot support one: none of the other
components has a number yet.

## Verification

* `cargo build --offline -p rexx-parse --benches`: clean.
* `cargo clippy --offline --all-targets -- -D warnings`: clean.
* `cargo fmt --all -- --check`: clean (no diff after the earlier `cargo fmt` pass).
* `cargo test --offline --workspace --no-fail-fast`: every crate green, 0
  failed (`rexx-parse`'s own suites: `program.rs` 32, `directive/tests.rs` 253,
  `errors.rs` 23, `scanner.rs` 35, `sourceline.rs` 25, `tokens.rs` 9, plus unit
  tests; 3 pre-existing `#[ignore]`d tests elsewhere in the workspace, untouched).
* `cargo bench --offline -p rexx-parse --bench parse`: run twice, agreed
  within criterion's own noise band; numbers above.

## What went wrong

* My first instinct, before measuring, was to follow the parent task's
  literal instruction and assert only `Program.instructions.len()`.
  A one-line throwaway test (deleted before the real benchmark was written)
  showed that number is 41 and 7 for the two files -- true, but so weak a
  check that a parser dropping every directive body would still pass it.
  Caught before it shipped, not after; the fix is finding 2 above.
* While drafting the `d10-decision.md` addition, an `Edit` call partially
  matched and left one sentence duplicated with two slightly different
  wordings of the same combined-time figure (`"on the order of 3.3 ms"` next
  to `"3.27-3.30 ms"`).
  Caught by re-reading the file before finalizing rather than by a linter --
  markdown has no tool that would have caught it, so this is a reminder that
  a multi-step `Edit` sequence on prose needs the same re-read discipline as
  a multi-step code edit.
* `d10-decision.md`'s own text names "Task 3.11" for the measurement this
  task performs under the number 3.10 (pre-flight finding 5).
  Left unresolved because it is the parent plan's numbering to fix, not a
  defect in this task's own output, but a later reader of that document
  should not be surprised to find Task 3.10's measurement sitting under a
  sentence that predicted Task 3.11 would supply it.
  (Fixed independently by the coordinator in `c1f84b0a`, landed between this
  task's original commit and its fix round.)

## Fix round, 2026-07-29

Review verdict: spec PASS, quality NOT APPROVED on one Critical, addressed in
commit `c0e1321a`.
Everything measurable reproduced clean on the reviewer's own run (2.6317-2.6429 ms
and 651.15-658.63 us, p = 0.40 and 0.62 against the stored baseline); the
finding was entirely about what the artefact claimed, not what it measured.

### Critical: the provenance claim was false for two of the three counts

`src/directive/tests.rs` pins `directives.len() == 347` for `CoreClasses.orx`
as a literal, and per-kind counts for `StreamClasses.orx` (7/139/5/2, summing
to 153, though 153 is never itself written there).
It pins **nothing** about `main_instructions` (41, 7) or `nested_instructions`
(2390, 610) -- those were measured for the first time by my own throwaway
pre-flight test and hardcoded into the benchmark, then written up in four
places as if `tests.rs` had independently pinned all three.
Fixed at all four sites: the module doc's "The assertion" section, the `Case`
struct's doc comment, the panic message (renamed to point at which count
actually diverged, since "the benchmark and the acceptance test have
diverged" was itself imprecise when there is no acceptance test for two of
the three counts), and `d10-decision.md`'s matching paragraph.
The corrected text states plainly: `directives.len()` is cross-checked
against `tests.rs`, 153 is consistent with `tests.rs`'s per-kind pins, and
`main_instructions`/`nested_instructions` are this benchmark's own baseline, a
change detector against today's output, not an independent pin.

### Assertion left unexpanded, limit recorded in prose instead

Per instruction, the triple was not widened into a structural checksum.
Added a new module-doc section and a matching `d10-decision.md` paragraph
naming the three things it cannot observe, as given: corrupted control-flow
wiring (a jump index pointing at the wrong instruction with every count
unchanged), a body-boundary bug moving a clause between adjacent directives
while the cross-directive sum holds, and anything inside an `Expr`.
Stated as a limit shared with `tests.rs`'s own acceptance tests, not one this
benchmark introduces.

### Two minors

* "only `parse_program` itself is inside the timed region" was not literally
  true -- the routine closure also runs the nested-instruction sum and the
  triple assertion, both cheap but present.
  Reworded to "the timed region holds `parse_program` plus the cheap
  node-count check."
* "comparable with `perf-baseline.md`" overstated what that file supports: it
  has no row for `rexx-parse` or either `.orx` file, so nothing here is
  checked against a value recorded there.
  Reworded to say the methodology (criterion settings) matches, not the
  number.

### The clone framing, addressed though not formally flagged

The reviewer's own measurement (1.01 us for the 141,049-byte buffer, about
0.04% of the parse) showed my "would have overstated the cost" framing was
more dramatic than the number supports, though this was noted in the
"confirmed clean" preamble rather than listed as a Critical or Minor.
Reworded in both `benches/parse.rs` and `d10-decision.md`: the clone is
excluded on principle -- the interpreter's own cold-start path never pays for
one -- rather than because including it would distort the result, and the
measured magnitude (about 1 us, under 0.1%) is now stated rather than implied
to be larger than it is.

### Verification after the fix round

* `cargo build --offline -p rexx-parse --benches`: clean.
* `cargo clippy --offline --all-targets -- -D warnings`: clean.
* `cargo fmt --all -- --check`: clean.
* `cargo test --offline --workspace --no-fail-fast`: every crate green, 0 failed.

### What went wrong, fix round

* The provenance error was mine, not a hallucination laundered through review:
  I had the true counts (41/7 main, 2390/610 nested) in hand from my own
  pre-flight measurement and still wrote the doc comments as though
  `tests.rs` had pinned them, apparently by analogy with the one count
  (`directives.len()`) that actually is pinned there.
  Writing "reused here rather than re-derived" for a value I had, in fact,
  just derived is the same shape of error the report's own "What went wrong"
  section should have caught and did not -- the pre-flight finding that
  distinguished pinned from first-measured evaporated somewhere between
  finding it and writing the artefact.
* I did not independently re-verify the reviewer's 1.01 us clone measurement
  or the 0.04% figure before using them; the coordinator's message said this
  was already confirmed clean and not to re-check it, so both are cited as
  the reviewer's numbers, not re-derived here.
