# Task 1 fix-round re-review: `a9420630..ebbfb3d7`

Scope: only the four findings from `task-1-review.md` (I1, I2, M1, M2), plus
anything new introduced by the fix. Commit reviewed: `ebbfb3d7` only.
`6829ca32` and `cf7bb05c` touch only `docs/superpowers/plans/...` (confirmed:
`git show --stat ebbfb3d7` names no `docs/` file) and are out of scope, as
instructed.

All gates re-run here, unpiped, tree clean before and after every experiment:

```
cargo fmt --all --check                                  -> 0
cargo clippy --workspace --all-targets -- -D warnings     -> 0
cargo test --workspace                                    -> 0  (853 passed, 0 failed, 4 ignored)
REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus corpus_differential -> 0
                                                            "mode: STRICT (the gate)" / "30 of 30 matching"
git status --porcelain                                    -> empty, throughout
```

853/0/4 matches the report exactly (`grep -c FAILED`, `grep -c failures:` both
0 on the full `cargo test --workspace` log). Assertion table `4224 of 4259`,
matching.

---

## I1 -- ADDRESSED (half a fully; half b's deferral holds up under attack)

**(a), re-measured independently, not taken from the report.**

`trace r` / `x = 'nop'` / `interpret x`, oracle vs. `rexx-run`, three
descriptors read separately:

* stdout: both empty -- MATCH
* rc: both 0 -- MATCH
* stderr: oracle has one more line than this crate --
  ```
        3 *-* nop
  ```
  everything above it, including `>>>   "nop"` on the `interpret x` clause,
  is now byte-identical.

One `DO` deeper (`trace r` / `do kk = 1 to 1` / `interpret "nop"` / `end`):
the `>>>     "nop"` line (two extra spaces, the construct's own indent) is
present and correct on both sides. The transcript still diverges from the
oracle by two *other*, already-disclosed lines: the same missing fragment
echo, and a pre-existing 4a `DO`/`TRACE` gap (a re-echoed `>>>` pair on loop
exit for the control variable). I isolated the second one with a control --
the same program with `INTERPRET` replaced by a bare `nop` diverges
identically -- confirming it is not this task's and not new.

**(b) deferral argument -- I tried to refute it and failed; it survives.**

I built the "obvious wrong fix" myself: changed `run_fragment`'s
`run_bounded(&code, 0, len, None)` to `Some(&fragment.source)`, rebuilt, and
ran `interpret "say 2 & 1"` three lines into a program (so the fragment's own
line 1 and the enclosing clause's line 3 are distinguishable). Result:

* the reported error line moved from the correct `3` to the wrong `1`
* the clause echo did not gain the fragment's `1 *-* say 2 & 1` line as a
  new, additional line -- it **replaced** the enclosing `3 *-* interpret
  "say 2 & 1"` echo, which vanished entirely

Both are exactly the two failure modes the report predicts (wrong line number
via `record_failure_site`'s first-wins race, and the wrong clause winning).
The "fix" is strictly worse than the shipped state, not merely incomplete.
Reverted; tree confirmed clean before rebuilding onward.

## I1(b) is genuinely deferred, not swept under a plausible-sounding excuse -- CONFIRMED

---

## I2 -- ADDRESSED, and independently re-derived, not just re-read

Instrumented `Interp::slot_of`'s growth branch (`plan.rs:556-559`) with an
`eprintln!`, three variants, one binary, reverted after each measurement
(`git status --porcelain` empty afterward):

| variant | `Activation::extra` hits |
|---|---|
| original (`interpret_dynamic.rex` before this fix) | 0 |
| prescribed (`interpret "zork = 42"` + bare `say zork`) | 0 |
| shipped (`interpret "zork = 42"` + `interpret "say zork"`) | **1, `ZORK`** |

Stronger than what was asked: I ran the *entire* 30-program differential
corpus (all of `phase-4a.txt` + `phase-4b.txt`) through the same
instrumentation in one pass. Total hits across all 30 programs: **1**, the
same `ZORK`. Nothing else in the corpus reaches `Activation::extra` (no
program uses indirect `DROP (v)`, the only other path into that branch), so
the new `tests.rs` comment's claim "the `Activation::extra` path, which
nothing in the differential corpus reached before" is true in the strong
sense, not merely true of the one file it was measured against.

Byte-for-byte check on the shipped `corpus/lang/interpret_dynamic.rex`
against the oracle: rc 0/0, stdout MATCH, stderr MATCH (empty on both).

---

## M1 -- ADDRESSED

`INSTRUCTION_WITNESSES` (`loud.rs:194-351`) counted mechanically: **23**
`Witness {` entries. `InstructionKind`'s `Owner::Phase` arms in
`owners.rs`: **19** (`Command`, `Call`, `Return`, `Procedure`, `Use`,
`Signal`, `Raise`, `Push`, `Queue`, `Parse`, `Arg`, `Pull`, `Address`,
`Expose`, `Options`, `Message`, `Guard`, `Reply`, `Forward`). `Call` expands
to 4 witness rows, `Signal` to 2, everything else 1:1 -- 19 + 3 + 1 = 23.
The new doc's arithmetic is exact.

---

## M2 -- ADDRESSED

The "two tasks still to land" sentence is gone; the paragraph now says every
task lands and re-runs the differential. The replacement claim -- "every
figure this runner reports is computed from `subset.len()` at run time" --
checked against the actual code: `corpus_differential` (`corpus.rs:498-530`)
computes `total = subset.len()` and accumulates `matched` by running
`check_case` per entry; no hardcoded total anywhere in the function. True.

---

## Corpus figure: 30 of 30, not 31 -- CONFIRMED

STRICT gate: exit 0, `mode: STRICT (the gate)`, `30 of 30 matching`. Only
`rust/corpus/lang/interpret_dynamic.rex` and `rust/corpus/phase-4b.txt`
changed under `rust/corpus/` between `a9420630` and `ebbfb3d7`
(`git diff --name-only`); `phase-4a.txt` and its 29 listed programs are
untouched, so the STRICT pass necessarily re-certifies all 29 plus the one
grown 4b witness -- 30 total, matching the dispatch's own correction that 31
would have required a new file, which nothing here adds.

---

## `rexx-parse` pinned expectations -- ADDRESSED, and regeneration verified independently

* `the_corpus_programs_parse` (instruction-count table): passes; count is
  `10` in the source, matching what I got parsing the shipped file.
* `sourceline_matches_the_interpreter_for_every_corpus_program`: passes.
* Independently regenerated the SOURCELINE expectation myself, using the
  driver documented in `sourceline_oracle.rs`'s module doc, against a
  **scratch copy** of `interpret_dynamic.rex`
  (`$SCRATCH/interpret_dynamic_scratchcopy.rex`) rather than the repository
  file -- the driver instantiates `.Package~new`, which the standing rule
  forbids on a repo file. Output: `count 10` plus the ten lines, **byte-for-byte
  identical** (`diff`, not eye) to the committed
  `sourceline_oracle/interpret_dynamic.txt`. `git status --porcelain` stayed
  empty throughout -- no stray file landed in the tree.

---

## Comments checked for truth (every one the diff adds or changes)

All measured, not read:

* `run_activation`'s corrected comment (`run.rs:445-460`), the `zz = 'nop'`
  transcript quoted verbatim -- reproduced exactly, character for character,
  including the trailing `3 *-* nop` line marked as oracle-only.
* The `Interpret` arm's new doc comment (`run.rs:762-812`) -- both quoted
  transcripts (top level and one `DO` deeper) reproduced exactly; the "before
  `run_fragment`, not after" claim confirmed by actually moving the call and
  watching the indent go from `>>>     "nop"` (2 spaces, correct) to
  `>>>   "nop"` (0 spaces, wrong).
* `interpret_traces_the_text_it_is_about_to_run`'s doc -- both named
  mutations carried out by hand: deleting the `trace_result` call and moving
  it after `run_fragment`. Both FAILED the test, as claimed.
* `phase-4b.txt`'s header -- covered under I2 above; true and understated if
  anything (the corpus-wide check found zero collateral hits, not just zero
  in this one file).
* `loud.rs`'s M1 doc -- arithmetic verified by direct count.
* `corpus.rs`'s M2 doc and the new dated row at `a9420630: 30 of 30 matching`
  -- both true; the dated-figure addition follows the original review's own
  ruling (append, don't replace) to the letter.
* `instruction/tests.rs`'s comment on the `8 -> 10` count -- true, and (see
  I2) provably the *only* program in the corpus that reaches that path.

No false or unverified claim found among the comments this fix round adds or
changes.

---

## New findings

None of Critical or Important severity. One cosmetic note, not worth a
finding: the "one `DO` deeper" oracle transcript in `run.rs`'s new doc
comment and in the new test's doc quotes only the fragment's own `>>>` line
in isolation; a reader diffing the *full* transcript against the oracle by
hand (as I did) will also see the pre-existing, already-disclosed 4a
`DO`/`TRACE` re-echo gap and could momentarily mistake it for something this
fix left broken. The report's Concern text already discloses that gap by name
elsewhere, so this is not a missing disclosure -- just a place a future
reader might have to reconstruct the isolation the implementer already did.
Not filed as a finding.

---

## Verdicts

| Finding | Verdict |
|---|---|
| I1 | ADDRESSED -- half (a) fixed and re-measured; half (b)'s deferral survives an attempted refutation (built and ran the "obvious fix," it breaks line attribution and clause selection exactly as predicted) |
| I2 | ADDRESSED -- 0/0/1 hit counts reproduced independently, plus a corpus-wide control showing the shipped witness is the *only* differential program reaching `Activation::extra`; byte-identical to oracle on all three descriptors |
| M1 | ADDRESSED -- 23-entry count and the 19+3+1 arithmetic both verified by direct count |
| M2 | ADDRESSED -- stale forward reference removed; replacement claim (every figure computed from `subset.len()`) verified against the code |

Corpus: **30 of 30**, confirmed correct (not a shortfall against the
dispatch's mistaken "31"). STRICT mode certifies all 30 on all three
descriptors. All 29 original 4a programs untouched and still matching.
