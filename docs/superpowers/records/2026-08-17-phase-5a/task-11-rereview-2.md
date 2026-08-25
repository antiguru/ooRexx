# Task 11 fix round 2 -- scoped re-review

Reviewing `fefe33597..0ef493ae7` (`3983105e1`, `0ef493ae7`) against the two
questions only.

## Question 1: does the rendering change break anything the corpus does not reach?

**No divergence found.** The change (`run.rs`'s `RAISE ... ARRAY` arm) replaces
`self.to_text(value).to_vec()` with `self.string_value_text(value)`. Read
`string_value_text` (`value.rs:694`): it special-cases `Body::Array` to answer
the constant `ARRAY_DEFAULT_NAME` ("an Array") and **falls through to
`self.to_text(value).to_vec()` for every other body kind, unchanged** -- so for
a string, a number, `.nil`, a stem, or a class object the new call is
byte-for-byte the same computation the old call was. The only kind of value
whose rendering can differ from before is a nested `Body::Array`, which is
exactly the fix.

`string_value_text` also already had six other call sites before this round
(`trace.rs`'s `>>>`/`>=>` lines, `dispatch.rs`'s `UNKNOWN` receiver name,
`eval.rs`'s several trace/quote sites, and the `RAISE ADDITIONAL` arm's own
line and slot loop) -- this round adds a seventh caller to an existing,
already-battle-tested function rather than introducing a new renderer, so
those sites cannot be affected by this diff.

Probed the neighbourhood directly: 12 programs (string, number, `.nil`, a
stem, a class object, and an omitted slot, each in both the `ARRAY` and the
`ADDITIONAL` spelling), traced (`trace i`), untrapped, run fresh-directory
against the oracle and both crate engines, three descriptors read separately.
**All 12 byte-identical on all three descriptors on both engines** -- includes
the `>A>`/`>K>` trace lines for a plain (non-nested) list, which is exactly
the "does a plain list's trace line still render as before" question. Sample
(string element, `ARRAY` spelling): oracle/ir/tw all emit
```
>L>   "40.4"  >K>   "SYNTAX" => "40.4"  >L>   "hi"  >A>   "hi"  >A>   "hi"
>L>   "5"  >A>   "5"  >A>   "5"  >K>   "ARRAY" => "an Array"
```
followed by `Error 40.4:  Too many arguments in invocation of hi; maximum
expected is 5.` at rc 216, identically on all three.

Programs are in
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/cd8e0d76-f9df-4fc9-9792-2e8e29a6d20b/scratchpad/probe11/p_*.rex`
(scratchpad, not the repository).

## Question 2: audit of the round-2 prose

### Checked and true

* **`an_object_as_a_raise_syntax_substitution_is_loud`** (`eval.rs:3669`) exists
  exactly as fix round 2 describes: runs `additional (.array)` and
  `additional (.environment)` through `both_engines`, asserts `(120, "")`, and
  asserts stderr contains `"a RAISE ADDITIONAL value"`. `git log -S` on that
  name finds one hit, `e5cce2546`, which is far older in `git log --oneline`
  than `f4b21eadb` (task 11's own first commit) -- **it predates this task**,
  as claimed.
* **`do_over_for_0_skips_the_single_non_stem_iteration`** (`run/tests.rs`)
  does now assert the control variable's post-loop value (the third block,
  `x = 'pre'` / `do x over 'hello' for 0` / `say x`, asserting `b"hello\n"`).
  **Load-bearing, verified by mutation**: reordered `run.rs`'s
  `LoopState::OverItems` arm in `loop_advance` to consult `remaining` (the
  `FOR` budget) before `bind_control`, reproducing the old bug. Ran
  `cargo test --release -p rexx-exec do_over_for_0_skips_the_single_non_stem_iteration`:
  1 test run (not 0-matched), **FAILED**, `left: "pre\n" right: "hello\n"`.
  Restored from a pre-mutation copy, `diff -q` reported no difference,
  `git status --short` clean, rebuilt, and re-ran the same test: 1 passed.
* **Every "no differential row" sentence** (identityHash, the oracle crasher,
  the three added refusals) reads as structurally true and unchanged by this
  round; not independently re-derived beyond what round 2's own audit table
  already re-checked, since none of these is the "written from memory instead
  of a search" shape the task called out.
* **The methodology numbers.** Recomputed every figure the "One methodology
  finding" paragraph cites directly from `rust/bench-baselines/phase-5a-arms.tsv`
  (`/bin/grep`/`awk` on the tab-separated file, not the ugrep wrapper):
  `arith`/`ir`/`per_pass`/`pinned`/`instructions:u` is 23764.522156 at
  `task=11,commit=f4b21eadb` and 24059.573752 at `task=11-fixround-2` (matches
  "23764.5222" / "24059.5738"); `alloc4c`/`ir` is 3591.397682 and 3594.684960
  (matches "3591.3984" / "3594.6850"); the within-sitting `head - pinned` on
  `arith`/`ir` is +2.998080 and +3.000848 (matches "+2.9981" / "+3.0008"), and
  on `alloc4c`/`ir` is +8.998500 and +9.001168 (matches "+8.9985" / "+9.0012").
  The `11-bisection` task label is one commit tag (`6f3434e88`) for all six
  builds, confirming it is a single sitting as claimed.
* **The renderer/slot-accessor claim.** `array_slots_of` is called exactly
  once in `run.rs`, inside the `ADDITIONAL` arm (line 4580), and not at all in
  the `ARRAY` arm (the `if let Some(items) = &raise.array` block starting at
  line 4607 iterates the parsed expression list directly). This is forced by
  the grammar rather than a style choice: an `ARRAY` list's elements are
  parser output with no backing array object to hold slots, while
  `ADDITIONAL`'s one value is converted to a real array whose slots are the
  substitution list. Confirmed by reading the code, not inferred.

### Found false

**1. "Every figure is the round-1 sitting's to three decimal places."**
False for 2 of the 12 published `across_builds`/`pinned>head`/`instructions:u`
percentages. Recomputed both tables' underlying `small`-size ratios from the
TSV:

| axis/arm | round-1 sitting (`6f3434e88`) | round-2 sitting (`3983105e1`) | printed round-1 | printed round-2 |
|---|---|---|---|---|
| `alloc4c`/`ir` | 1.002706 | 1.002703 | +0.271% | +0.270% |
| `arith`/`tw` | 1.001508 | 1.001500 | +0.151% | +0.150% |

Both ratios drifted by roughly three parts in a million between sittings --
the same order of magnitude as the drift the paragraph itself documents for
the *absolute* `arith`/`ir` and `alloc4c`/`ir` per-pass figures -- and that
drift happens to straddle a rounding boundary in the percentage display for
these two cells, so the two published tables (section 4's and fix round 2's
own) disagree in the third decimal place on both of them. This does not
change any conclusion the report draws (both are still far under the 1%
threshold), but the literal claim that every figure matches to three decimal
places is checkably wrong for two of twelve.

**2. "So that class of sentence has now been wrong three times on this task --
once in the round-1 prose about the send-path instrument, once about the
`RAISE` refusal, once about `DO OVER ... FOR`."**
The "send-path instrument" is not part of Task 11's history at all. The term
appears verbatim only in `task-10-report.md` ("## Item 3: the send-path
figure, taken", a performance measurement of native/method/internal-`CALL`
dispatch cost) and `task-10-rereview.md` ("## Finding 3: the send-path
figure"), both Task 10 records. Task 10's rereview verdict on that item is
**"ADDRESSED, and independently reproduced"** -- it was never found to be a
false "no test covers X" statement; grepping both Task 10 files for that
pattern turns up nothing matching it. So this sentence borrows an unrelated
episode from a different task, misattributes it to "this task," and
mischaracterizes its own outcome (calling it an instance of the "wrote from
memory instead of searching" defect when the actual Task 10 finding was a
clean pass). The count of two confirmed instances on Task 11 (the `RAISE`
refusal, corrected per `task-11-rereview-1.md`'s Question 3 ruling; `DO OVER
... FOR`, corrected in this round) stands; "three times on this task" does
not.

## Files

* Re-review: `.superpowers/sdd/2026-08-17-phase-5a/task-11-rereview-2.md` (this file)
* Probed: `.superpowers/sdd/2026-08-17-phase-5a/task-11-report.md`,
  `review-fefe33597..0ef493ae7.diff`
* Code read: `rust/crates/rexx-exec/src/value.rs` (`string_value_text`,
  `to_text`), `rust/crates/rexx-exec/src/run.rs` (`RAISE` arms ~4560-4685,
  `loop_advance`'s `LoopState::OverItems` arm ~8677-8720),
  `rust/crates/rexx-exec/src/eval.rs:3669` (`an_object_as_a_raise_syntax_substitution_is_loud`),
  `rust/crates/rexx-exec/src/run/tests.rs` (`do_over_for_0_skips_the_single_non_stem_iteration`)
* Data checked: `rust/bench-baselines/phase-5a-arms.tsv` (tasks `11`,
  `11-bisection`, `11-fixround-2`)
* Scratch probes (not in the repository):
  `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/cd8e0d76-f9df-4fc9-9792-2e8e29a6d20b/scratchpad/probe11/`
* Tree left clean: mutation to `run.rs` (both the `RAISE` renderer probe area,
  untouched, and the `loop_advance` reorder) was reverted from a pre-mutation
  copy, `diff -q` confirmed identical, `git status --short` reports nothing,
  and the release binary was rebuilt afterward.
